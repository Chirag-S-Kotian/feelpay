use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

/// Middleware to validate API Keys for Server-to-Server requests
pub struct ApiKeyMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct ApiKeyMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let api_key = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty());

            let Some(api_key) = api_key else {
                return Err(actix_web::error::ErrorUnauthorized(
                    "Missing x-api-key header",
                ));
            };

            let Some(pool) = req.app_data::<web::Data<sqlx::PgPool>>() else {
                return Err(actix_web::error::ErrorInternalServerError(
                    "Database pool unavailable",
                ));
            };

            let merchant = sqlx::query_as::<_, (Uuid,)>(
                "SELECT id FROM merchants WHERE api_key = $1 AND status = 'active' LIMIT 1",
            )
            .bind(api_key)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to validate api key")
            })?;

            let Some((merchant_id,)) = merchant else {
                return Err(actix_web::error::ErrorUnauthorized("Invalid API key"));
            };

            req.extensions_mut().insert(merchant_id);

            service.call(req).await
        })
    }
}

/// Middleware to validate JWT for Dashboard requests
pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService { service }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .unwrap_or_default()
            .to_string();

        let token = auth_header
            .strip_prefix("Bearer ")
            .or_else(|| auth_header.strip_prefix("bearer "))
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string);

        let Some(token) = token else {
            return Box::pin(async {
                Err(actix_web::error::ErrorUnauthorized(
                    "Missing Authorization bearer token",
                ))
            });
        };

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        let claims = match decoded {
            Ok(v) => v.claims,
            Err(_) => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                });
            }
        };

        let merchant_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid merchant identity in token"))
                });
            }
        };

        // touch claims fields to keep validation context explicit
        let _ = (claims.exp, claims.iat);

        req.extensions_mut().insert(merchant_id);
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

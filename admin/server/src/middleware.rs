use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::future::{ready, Ready};
use uuid::Uuid;
use sqlx::PgPool;

pub struct AdminAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddlewareService { service }))
    }
}

pub struct AdminAuthMiddlewareService<S> {
    service: S,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddlewareService<S>
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
                Err(actix_web::error::ErrorUnauthorized("Missing Authorization bearer token"))
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
                    Err(actix_web::error::ErrorUnauthorized("Invalid identity in token"))
                });
            }
        };

        let pool = req.app_data::<web::Data<PgPool>>().cloned();

        let fut = self.service.call(req);

        Box::pin(async move {
            let pool = pool.ok_or_else(|| actix_web::error::ErrorInternalServerError("DB pool missing"))?;
            
            // Check if is_admin
            let is_admin = sqlx::query("SELECT is_admin FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(|e| {
                    tracing::error!("Auth check error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?;

            match is_admin {
                Some(row) => {
                    use sqlx::Row;
                    if !row.get::<bool, _>("is_admin") {
                        return Err(actix_web::error::ErrorForbidden("Forbidden: Admin access required"));
                    }
                }
                None => return Err(actix_web::error::ErrorUnauthorized("Merchant not found")),
            }

            fut.await
        })
    }
}

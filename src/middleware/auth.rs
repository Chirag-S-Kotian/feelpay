use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::models::user::Claims;
pub struct MerchantId(pub Uuid);
pub struct TenantId(pub Uuid);
pub struct UserId(pub Uuid);
pub struct UserRole(pub String);

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
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
            return ready(Err(actix_web::error::ErrorUnauthorized(
                "Missing Authorization bearer token",
            )));
        };

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        match decoded {
            Ok(v) => ready(Ok(v.claims)),
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))),
        }
    }
}

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

            // Unified check for API Keys
            let merchant = sqlx::query_as::<_, (Uuid, Uuid)>(
                "SELECT id, tenant_id FROM merchants WHERE api_key = $1 AND status = 'active' LIMIT 1",
            )
            .bind(api_key)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to validate api key")
            })?;

            let Some((merchant_id, tenant_id)) = merchant else {
                return Err(actix_web::error::ErrorUnauthorized("Invalid API key"));
            };

            // Set context for RLS and logging using wrapper types
            req.extensions_mut().insert(MerchantId(merchant_id));
            req.extensions_mut().insert(TenantId(tenant_id));

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

        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid user identity in token"))
                });
            }
        };

        // Insert context into request extensions using wrapper types
        req.extensions_mut().insert(UserId(user_id));
        if let Some(tid) = claims.tenant_id {
            req.extensions_mut().insert(TenantId(tid));
        }
        if let Some(mid) = claims.merchant_id {
            req.extensions_mut().insert(MerchantId(mid));
        }
        req.extensions_mut().insert(UserRole(claims.role.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}


/// Middleware that accepts EITHER an API Key OR a JWT token.
/// Useful for resources shared between the dashboard and API integrators.
pub struct UnifiedAuth;

impl<S, B> Transform<S, ServiceRequest> for UnifiedAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UnifiedAuthService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UnifiedAuthService {
            service: Rc::new(service),
        }))
    }
}

pub struct UnifiedAuthService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for UnifiedAuthService<S>
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
            // 1. Try API Key Auth first
            let api_key = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty());

            if let Some(key) = api_key {
                if let Some(pool) = req.app_data::<web::Data<sqlx::PgPool>>() {
                    let merchant = sqlx::query_as::<_, (Uuid, Uuid)>(
                        "SELECT id, tenant_id FROM merchants WHERE api_key = $1 AND status = 'active' LIMIT 1",
                    )
                    .bind(key)
                    .fetch_optional(pool.get_ref())
                    .await
                    .map_err(|_| actix_web::error::ErrorInternalServerError("Auth validation failed"))?;

                    if let Some((mid, tid)) = merchant {
                        req.extensions_mut().insert(MerchantId(mid));
                        req.extensions_mut().insert(TenantId(tid));
                        return service.call(req).await;
                    }
                }
            }

            // 2. Try JWT Auth second
            let auth_header = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .unwrap_or_default();

            if auth_header.to_lowercase().starts_with("bearer ") {
                let token = auth_header[7..].trim();
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
                
                if let Ok(token_data) = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::default(),
                ) {
                    let claims = token_data.claims;
                    if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                        req.extensions_mut().insert(UserId(user_id));
                        if let Some(tid) = claims.tenant_id {
                            req.extensions_mut().insert(TenantId(tid));
                        }
                        if let Some(mid) = claims.merchant_id {
                            req.extensions_mut().insert(MerchantId(mid));
                        }
                        req.extensions_mut().insert(UserRole(claims.role));
                        return service.call(req).await;
                    }
                }
            }

            Err(actix_web::error::ErrorUnauthorized(
                "Authentication required: Provide a valid x-api-key or Bearer token",
            ))
        })
    }
}

/// Create a new JWT token for a user or merchant
pub fn create_token(id: Uuid, _email: &str) -> Result<String, crate::error::FeelPayError> {
    let exp = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: id.to_string(),
        tenant_id: None,
        merchant_id: Some(id), // For merchant login, the ID is the merchant ID
        role: "merchant".to_string(),
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| crate::error::FeelPayError::Internal(format!("JWT error: {}", e)))
}

use crate::models::user::Claims;
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use chrono::{Duration, Utc};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;
pub struct MerchantId(pub Uuid);
pub struct TenantId(pub Uuid);
pub struct UserId(pub Uuid);
pub struct UserRole(pub String);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentMode {
    Test,
    Live,
}
pub struct RequestEnvironment(pub EnvironmentMode);

pub type JwtClaims = Claims;

pub fn jwt_secret() -> Result<String, actix_web::Error> {
    std::env::var("JWT_SECRET")
        .map_err(|_| actix_web::error::ErrorInternalServerError("JWT_SECRET is not configured"))
}

pub fn jwt_secret_for_token() -> Result<String, crate::error::FeelPayError> {
    std::env::var("JWT_SECRET")
        .map_err(|_| crate::error::FeelPayError::Internal("JWT_SECRET is not configured".into()))
}

fn request_environment(req: &ServiceRequest) -> EnvironmentMode {
    let value = req
        .headers()
        .get("x-feelpay-environment")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "test".to_string());
    if value == "live" {
        EnvironmentMode::Live
    } else {
        EnvironmentMode::Test
    }
}

fn bearer_token_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn bearer_token_from_header(value: &str) -> Option<String> {
    value
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| value.trim().strip_prefix("bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

pub fn bearer_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(bearer_token_from_header)
}

async fn is_bearer_token_revoked(req: &ServiceRequest, token: &str) -> bool {
    let Some(state) = req.app_data::<web::Data<crate::state::AppState>>() else {
        return false;
    };
    let key = format!("revoked:jwt:{}", bearer_token_hash(token));
    let Ok(mut conn) = state.redis.get_connection() else {
        return false;
    };
    redis::cmd("EXISTS")
        .arg(key)
        .query::<i32>(&mut conn)
        .map(|exists| exists == 1)
        .unwrap_or(false)
}

pub fn revoke_bearer_token(
    state: &crate::state::AppState,
    token: &str,
    ttl_seconds: usize,
) -> bool {
    let key = format!("revoked:jwt:{}", bearer_token_hash(token));
    let Ok(mut conn) = state.redis.get_connection() else {
        return false;
    };
    redis::cmd("SETEX")
        .arg(key)
        .arg(ttl_seconds)
        .arg("1")
        .query::<()>(&mut conn)
        .is_ok()
}

async fn has_dual_api_key_columns(pool: &sqlx::PgPool) -> Result<bool, actix_web::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'merchants'
          AND column_name IN ('api_key_test', 'api_key_live')
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to inspect schema"))?;
    Ok(count == 2)
}

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

        let secret = match jwt_secret() {
            Ok(secret) => secret,
            Err(err) => return ready(Err(err)),
        };
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        match decoded {
            Ok(v) => ready(Ok(v.claims)),
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid or expired token",
            ))),
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

        if req.method() == actix_web::http::Method::OPTIONS {
            return Box::pin(async move { service.call(req).await });
        }

        Box::pin(async move {
            let env_mode = request_environment(&req);
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

            let dual_cols = has_dual_api_key_columns(pool.get_ref()).await?;
            let merchant = if dual_cols {
                let env_name = match env_mode {
                    EnvironmentMode::Live => "live",
                    EnvironmentMode::Test => "test",
                };
                sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
                    r#"
                    SELECT id, tenant_id
                    FROM merchants
                    WHERE status = 'active'
                      AND (
                        ($2 = 'live' AND COALESCE(api_key_live, api_key) = $1)
                        OR
                        ($2 = 'test' AND COALESCE(api_key_test, api_key) = $1)
                      )
                    LIMIT 1
                    "#,
                )
                .bind(api_key)
                .bind(env_name)
                .fetch_optional(pool.get_ref())
                .await
            } else {
                sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
                    "SELECT id, tenant_id FROM merchants WHERE api_key = $1 AND status = 'active' LIMIT 1",
                )
                .bind(api_key)
                .fetch_optional(pool.get_ref())
                .await
            }
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to validate api key"))?;

            let Some((merchant_id, tenant_id)) = merchant else {
                return Err(actix_web::error::ErrorUnauthorized("Invalid API key"));
            };

            // Set context for RLS and logging using wrapper types
            req.extensions_mut().insert(MerchantId(merchant_id));
            if let Some(tid) = tenant_id {
                req.extensions_mut().insert(TenantId(tid));
            }
            req.extensions_mut().insert(RequestEnvironment(env_mode));

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
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
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
        let service = self.service.clone();
        if req.method() == actix_web::http::Method::OPTIONS {
            return Box::pin(async move { service.call(req).await });
        }

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

        Box::pin(async move {
            if is_bearer_token_revoked(&req, &token).await {
                return Err(actix_web::error::ErrorUnauthorized(
                    "Session has been logged out",
                ));
            }

            let secret = jwt_secret()?;
            let decoded = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            );

            let claims = decoded
                .map(|v| v.claims)
                .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;

            let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
                actix_web::error::ErrorUnauthorized("Invalid user identity in token")
            })?;

            req.extensions_mut().insert(UserId(user_id));
            if let Some(tid) = claims.tenant_id {
                req.extensions_mut().insert(TenantId(tid));
            }
            if let Some(mid) = claims.merchant_id {
                req.extensions_mut().insert(MerchantId(mid));
            }
            req.extensions_mut().insert(UserRole(claims.role.clone()));

            service.call(req).await
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

        if req.method() == actix_web::http::Method::OPTIONS {
            return Box::pin(async move { service.call(req).await });
        }

        Box::pin(async move {
            let env_mode = request_environment(&req);
            // 1. Try API Key Auth first
            let api_key = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty());

            if let Some(key) = api_key {
                if let Some(pool) = req.app_data::<web::Data<sqlx::PgPool>>() {
                    let dual_cols = has_dual_api_key_columns(pool.get_ref()).await?;
                    let merchant = if dual_cols {
                        let env_name = match env_mode {
                            EnvironmentMode::Live => "live",
                            EnvironmentMode::Test => "test",
                        };
                        sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
                            r#"
                            SELECT id, tenant_id
                            FROM merchants
                            WHERE status = 'active'
                              AND (
                                ($2 = 'live' AND COALESCE(api_key_live, api_key) = $1)
                                OR
                                ($2 = 'test' AND COALESCE(api_key_test, api_key) = $1)
                              )
                            LIMIT 1
                            "#,
                        )
                        .bind(key)
                        .bind(env_name)
                        .fetch_optional(pool.get_ref())
                        .await
                    } else {
                        sqlx::query_as::<_, (Uuid, Option<Uuid>)>(
                            "SELECT id, tenant_id FROM merchants WHERE api_key = $1 AND status = 'active' LIMIT 1",
                        )
                        .bind(key)
                        .fetch_optional(pool.get_ref())
                        .await
                    }
                    .map_err(|_| actix_web::error::ErrorInternalServerError("Auth validation failed"))?;

                    if let Some((mid, tid)) = merchant {
                        req.extensions_mut().insert(MerchantId(mid));
                        if let Some(tenant_id) = tid {
                            req.extensions_mut().insert(TenantId(tenant_id));
                        }
                        req.extensions_mut().insert(RequestEnvironment(env_mode));
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
                if is_bearer_token_revoked(&req, token).await {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Session has been logged out",
                    ));
                }
                let secret = jwt_secret()?;

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
    let expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(24);
    let exp = Utc::now() + Duration::hours(expiry_hours);
    let claims = Claims {
        sub: id.to_string(),
        tenant_id: None,
        merchant_id: Some(id), // For merchant login, the ID is the merchant ID
        role: "merchant".to_string(),
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    let secret = jwt_secret_for_token()?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| crate::error::FeelPayError::Internal(format!("JWT error: {}", e)))
}

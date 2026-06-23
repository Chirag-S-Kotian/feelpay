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

fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
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

fn is_safe_method(method: &actix_web::http::Method) -> bool {
    matches!(
        *method,
        actix_web::http::Method::GET
            | actix_web::http::Method::HEAD
            | actix_web::http::Method::OPTIONS
    )
}

fn merchant_role_allows_request(role: &str, method: &actix_web::http::Method, path: &str) -> bool {
    if !role.starts_with("merchant_") {
        return true;
    }
    if matches!(role, "merchant_owner" | "merchant_admin") {
        return true;
    }
    if path == "/v1/merchants/logout" {
        return true;
    }

    let safe = is_safe_method(method);
    match role {
        "merchant_read_only" | "merchant_viewer" => safe,
        "merchant_developer" => {
            path.starts_with("/v1/developers")
                || path.starts_with("/v1/merchants/keys")
                || path.starts_with("/v1/merchants/webhook-secret")
                || path.contains("/webhooks/")
                || (safe && path.starts_with("/v1/merchants/settings/security"))
        }
        "merchant_finance" => {
            path.starts_with("/v1/payments")
                || path.starts_with("/v1/refunds")
                || path.starts_with("/v1/settlements")
                || path.starts_with("/v1/reports")
                || path.starts_with("/v1/payouts")
                || path.contains("/exports")
                || (safe && path.starts_with("/v1/analytics"))
        }
        "merchant_support" | "merchant_staff" => {
            if !safe {
                return path.starts_with("/v1/disputes/");
            }
            path.starts_with("/v1/payments")
                || path.starts_with("/v1/refunds")
                || path.starts_with("/v1/customers")
                || path.starts_with("/v1/disputes")
                || path.starts_with("/v1/notifications")
        }
        _ => false,
    }
}

fn path_is_admin(path: &str) -> bool {
    path == "/admin"
        || path.starts_with("/admin/")
        || path == "/v1/admin"
        || path.starts_with("/v1/admin/")
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

async fn merchant_from_hashed_api_key(
    pool: &sqlx::PgPool,
    api_key: &str,
    env_mode: EnvironmentMode,
) -> Result<Option<(Uuid, Option<Uuid>)>, actix_web::Error> {
    let environment = match env_mode {
        EnvironmentMode::Live => "live",
        EnvironmentMode::Test => "test",
    };
    let row = sqlx::query_as::<_, (Uuid, Option<Uuid>, Uuid)>(
        r#"
        SELECT m.id, m.tenant_id, k.id
        FROM developer_api_keys k
        JOIN merchants m ON m.id = k.merchant_id
        WHERE m.status = 'active'
          AND ($2 <> 'live' OR (m.onboarding_status = 'approved' AND COALESCE(m.review_status, 'draft') = 'approved'))
          AND k.status = 'active'
          AND k.environment = $2
          AND k.encrypted_key_hash = $1
          AND (k.expires_at IS NULL OR k.expires_at > NOW())
        LIMIT 1
        "#,
    )
    .bind(hash_api_key(api_key))
    .bind(environment)
    .fetch_optional(pool)
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to validate api key"))?;

    if let Some((merchant_id, tenant_id, key_id)) = row {
        let _ = sqlx::query("UPDATE developer_api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(pool)
            .await;
        return Ok(Some((merchant_id, tenant_id)));
    }
    Ok(None)
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

            let merchant = merchant_from_hashed_api_key(pool.get_ref(), api_key, env_mode).await?;

            let Some((merchant_id, tenant_id)) = merchant else {
                return Err(actix_web::error::ErrorUnauthorized("Invalid API key"));
            };
            if path_is_admin(req.path()) {
                return Err(actix_web::error::ErrorForbidden(
                    "Merchant credentials cannot access admin APIs",
                ));
            }

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
            req.extensions_mut().insert(RequestEnvironment(request_environment(&req)));

            if !merchant_role_allows_request(&claims.role, req.method(), req.path()) {
                return Err(actix_web::error::ErrorForbidden(
                    "Merchant role cannot access this operation",
                ));
            }

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
                    let merchant =
                        merchant_from_hashed_api_key(pool.get_ref(), key, env_mode).await?;

                    if let Some((mid, tid)) = merchant {
                        if path_is_admin(req.path()) {
                            return Err(actix_web::error::ErrorForbidden(
                                "Merchant credentials cannot access admin APIs",
                            ));
                        }
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
                        let role = claims.role;
                        if path_is_admin(req.path())
                            && (claims.merchant_id.is_some() || role.starts_with("merchant_"))
                        {
                            return Err(actix_web::error::ErrorForbidden(
                                "Merchant sessions cannot access admin APIs",
                            ));
                        }
                        req.extensions_mut().insert(UserRole(role.clone()));
                        req.extensions_mut().insert(RequestEnvironment(env_mode));
                        if !merchant_role_allows_request(&role, req.method(), req.path()) {
                            return Err(actix_web::error::ErrorForbidden(
                                "Merchant role cannot access this operation",
                            ));
                        }
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

#[cfg(test)]
mod tests {
    use super::{merchant_role_allows_request, path_is_admin};
    use actix_web::http::Method;

    #[test]
    fn merchant_role_matrix_blocks_sensitive_mutations() {
        assert!(!merchant_role_allows_request(
            "merchant_read_only",
            &Method::POST,
            "/v1/payments/abc/refund"
        ));
        assert!(!merchant_role_allows_request(
            "merchant_support",
            &Method::POST,
            "/v1/merchants/keys/roll"
        ));
        assert!(!merchant_role_allows_request(
            "merchant_finance",
            &Method::POST,
            "/v1/merchants/team/invite"
        ));
    }

    #[test]
    fn merchant_role_matrix_allows_expected_workflows() {
        assert!(merchant_role_allows_request(
            "merchant_developer",
            &Method::POST,
            "/v1/merchants/keys/roll"
        ));
        assert!(merchant_role_allows_request(
            "merchant_finance",
            &Method::POST,
            "/v1/payments/abc/refund"
        ));
        assert!(merchant_role_allows_request(
            "merchant_support",
            &Method::POST,
            "/v1/disputes/abc/evidence"
        ));
        assert!(merchant_role_allows_request(
            "merchant_read_only",
            &Method::GET,
            "/v1/settlements"
        ));
    }

    #[test]
    fn admin_path_detection_covers_mounted_admin_scopes() {
        assert!(path_is_admin("/admin/users"));
        assert!(path_is_admin("/v1/admin/tenants"));
        assert!(!path_is_admin("/v1/merchants/profile"));
    }
}

/// Create a new JWT token for a user or merchant
pub fn create_token(id: Uuid, _email: &str) -> Result<String, crate::error::FeelPayError> {
    create_merchant_token(id, id, None, "merchant_owner")
}

pub fn create_merchant_token(
    merchant_id: Uuid,
    user_id: Uuid,
    tenant_id: Option<Uuid>,
    role: &str,
) -> Result<String, crate::error::FeelPayError> {
    let expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(24);
    let exp = Utc::now() + Duration::hours(expiry_hours);
    let claims = Claims {
        sub: user_id.to_string(),
        tenant_id,
        merchant_id: Some(merchant_id),
        role: role.to_string(),
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

use actix_web::body::EitherBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, HttpResponse};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::Commands;
use serde::Deserialize;
use std::rc::Rc;

use crate::models::user::Claims;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize)]
struct MaintenanceConfig {
    enabled: bool,
    message: Option<String>,
    // feature -> enabled
    features: Option<std::collections::HashMap<String, bool>>,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            message: None,
            features: None,
        }
    }
}

fn is_allowlisted_path(path: &str) -> bool {
    path == "/health"
        || path == "/v1/health"
        || path.starts_with("/swagger-ui")
        || path.starts_with("/api-docs")
}

fn feature_from_path(path: &str) -> Option<String> {
    // /v1/<feature>/...
    let p = path.trim_start_matches('/');
    let mut parts = p.split('/');
    let first = parts.next()?;
    if first != "v1" {
        return None;
    }
    parts.next().map(|s| s.to_lowercase())
}

fn is_admin_bypass(req: &ServiceRequest) -> bool {
    // Allow bypass for admin roles if valid JWT present.
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or_default();

    let Some(token) = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
    else {
        return false;
    };

    let Ok(secret) = crate::middleware::auth::jwt_secret() else {
        return false;
    };
    let Ok(decoded) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) else {
        return false;
    };

    let role = decoded.claims.role.to_lowercase();
    role.contains("admin") || role.contains("super_admin")
}

fn load_maintenance_config(state: &AppState) -> MaintenanceConfig {
    // Best-effort: if redis down / key missing / invalid JSON -> maintenance disabled.
    let Ok(mut conn) = state.redis.get_connection() else {
        return MaintenanceConfig::default();
    };

    let value: redis::RedisResult<String> = conn.get("system_config:maintenance_mode");
    let Ok(raw) = value else {
        return MaintenanceConfig::default();
    };

    serde_json::from_str::<MaintenanceConfig>(&raw).unwrap_or_default()
}

pub struct MaintenanceMode;

impl<S, B> Transform<S, ServiceRequest> for MaintenanceMode
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = MaintenanceModeMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MaintenanceModeMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct MaintenanceModeMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for MaintenanceModeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            if is_allowlisted_path(&path) {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            let Some(state) = req.app_data::<web::Data<AppState>>() else {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            };

            let cfg = load_maintenance_config(state.get_ref());
            if !cfg.enabled {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            if is_admin_bypass(&req) {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            // Feature gating: if features map present, only block those true.
            if let Some(features) = cfg.features {
                if let Some(feature) = feature_from_path(&path) {
                    let is_blocked = features.get(&feature).copied().unwrap_or(false);
                    if !is_blocked {
                        return service
                            .call(req)
                            .await
                            .map(ServiceResponse::map_into_left_body);
                    }
                } else {
                    // Non /v1 path while global maintenance enabled -> block.
                }
            }

            let message = cfg
                .message
                .unwrap_or_else(|| "Service temporarily unavailable for maintenance".into());

            Ok(req.into_response(
                HttpResponse::ServiceUnavailable()
                    .json(serde_json::json!({
                        "success": false,
                        "error": { "code": "maintenance_mode", "message": message },
                    }))
                    .map_into_right_body(),
            ))
        })
    }
}

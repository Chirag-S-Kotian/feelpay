use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, StatusCode},
    web, Error, HttpMessage, HttpResponse, HttpResponseBuilder,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use sqlx::PgPool;
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct AdminAuthMiddleware;

fn jwt_secret() -> Result<String, Error> {
    std::env::var("JWT_SECRET")
        .map_err(|_| actix_web::error::ErrorInternalServerError("JWT_SECRET is not configured"))
}

fn allowed_origins() -> Vec<String> {
    std::env::var("ADMIN_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|origins| {
            origins
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_else(|| {
            [
                "http://localhost:3000",
                "http://127.0.0.1:3000",
                "http://localhost:5173",
                "http://127.0.0.1:5173",
                "http://localhost:5174",
                "http://127.0.0.1:5174",
            ]
            .iter()
            .map(|origin| origin.to_string())
            .collect()
        })
}

fn request_origin(req: &ServiceRequest) -> Option<String> {
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())?;
    allowed_origins()
        .into_iter()
        .find(|allowed| allowed == origin)
}

fn apply_cors(builder: &mut HttpResponseBuilder, origin: Option<String>) {
    if let Some(origin) = origin {
        builder.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));
        builder.insert_header((header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true"));
        builder.insert_header((
            header::VARY,
            "Origin, Access-Control-Request-Method, Access-Control-Request-Headers",
        ));
    }
    builder.insert_header((
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    ));
    builder.insert_header((
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "authorization, content-type, accept, x-api-key, x-feelpay-environment, idempotency-key, x-request-id, x-admin-role, x-csrf-token",
    ));
    builder.insert_header((header::ACCESS_CONTROL_EXPOSE_HEADERS, "x-request-id"));
    builder.insert_header((header::ACCESS_CONTROL_MAX_AGE, "3600"));
}

fn cors_no_content<B>(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
    let origin = request_origin(&req);
    let mut builder = HttpResponse::NoContent();
    apply_cors(&mut builder, origin);
    req.into_response(builder.finish().map_into_right_body())
}

fn cors_error<B>(
    req: ServiceRequest,
    status: StatusCode,
    message: &'static str,
) -> ServiceResponse<EitherBody<B>> {
    let origin = request_origin(&req);
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut builder = HttpResponse::build(status);
    apply_cors(&mut builder, origin);
    builder.insert_header(("x-request-id", request_id.clone()));
    req.into_response(
        builder
            .json(serde_json::json!({ "message": message, "request_id": request_id }))
            .map_into_right_body(),
    )
}

fn cors_actix_error(
    status: StatusCode,
    message: &'static str,
    origin: Option<String>,
    request_id: String,
) -> Error {
    let mut builder = HttpResponse::build(status);
    apply_cors(&mut builder, origin);
    builder.insert_header(("x-request-id", request_id.clone()));
    actix_web::error::InternalError::from_response(
        message,
        builder.json(serde_json::json!({ "message": message, "request_id": request_id })),
    )
    .into()
}

impl<S, B> Transform<S, ServiceRequest> for AdminAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
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

#[derive(Clone, Debug)]
pub struct AdminClaims {
    pub admin_id: Uuid,
    pub role: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Claims {
    sub: String,
    role: Option<String>,
    permissions: Option<Vec<String>>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let method = req.method().as_str().to_string();
        if path.ends_with("/bootstrap-token") || path.ends_with("/login") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) });
        }

        if req.method() == actix_web::http::Method::OPTIONS {
            return Box::pin(async move { Ok(cors_no_content(req)) });
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
            return Box::pin(async move {
                Ok(cors_error(
                    req,
                    actix_web::http::StatusCode::UNAUTHORIZED,
                    "Missing Authorization bearer token",
                ))
            });
        };

        let secret = match jwt_secret() {
            Ok(secret) => secret,
            Err(err) => return Box::pin(async { Err(err) }),
        };
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        let claims = match decoded {
            Ok(v) => v.claims,
            Err(_) => {
                return Box::pin(async move {
                    Ok(cors_error(
                        req,
                        actix_web::http::StatusCode::UNAUTHORIZED,
                        "Invalid or expired token",
                    ))
                });
            }
        };

        let merchant_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => {
                return Box::pin(async move {
                    Ok(cors_error(
                        req,
                        actix_web::http::StatusCode::UNAUTHORIZED,
                        "Invalid identity in token",
                    ))
                });
            }
        };

        let origin = request_origin(&req);
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let pool = req.app_data::<web::Data<PgPool>>().cloned();

        let req = req;
        req.extensions_mut().insert(AdminClaims {
            admin_id: merchant_id,
            role: claims.role.clone(),
        });
        let fut = self.service.call(req);

        Box::pin(async move {
            let pool =
                pool.ok_or_else(|| actix_web::error::ErrorInternalServerError("DB pool missing"))?;

            // Check if is_admin
            let merchant = sqlx::query("SELECT is_admin, name FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(|e| {
                    tracing::error!("Auth check DB error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?;

            match merchant {
                Some(row) => {
                    use sqlx::Row;
                    let is_admin: bool = row.get("is_admin");
                    let name: String = row.get("name");
                    tracing::info!("Auth check for {}: is_admin={}", name, is_admin);

                    if !is_admin {
                        return Err(cors_actix_error(
                            StatusCode::FORBIDDEN,
                            "Forbidden: Admin access required",
                            origin,
                            request_id,
                        ));
                    }
                }
                None => {
                    tracing::warn!("Merchant ID {} not found in auth check", merchant_id);
                    return Err(cors_actix_error(
                        StatusCode::UNAUTHORIZED,
                        "Merchant not found",
                        origin,
                        request_id,
                    ));
                }
            }

            let required_permission = match required_permission_for_path(&path, &method) {
                Some(permission) => permission,
                None => {
                    tracing::warn!("Admin RBAC denied unmapped route: {} {}", method, path);
                    return Err(cors_actix_error(
                        StatusCode::FORBIDDEN,
                        "Forbidden: Admin route is not permission mapped",
                        origin,
                        request_id,
                    ));
                }
            };

            let role = claims.role.as_deref().unwrap_or("Super Admin");
            let has_claim_permission = claims
                .permissions
                .as_ref()
                .map(|permissions| {
                    permissions
                        .iter()
                        .any(|permission| permission == required_permission)
                })
                .unwrap_or(false);

            if !role_has_permission(role, required_permission) && !has_claim_permission {
                tracing::warn!(
                    "Admin RBAC denied role={} permission={}",
                    role,
                    required_permission
                );
                return Err(cors_actix_error(
                    StatusCode::FORBIDDEN,
                    "Forbidden: Admin permission required",
                    origin,
                    request_id,
                ));
            }

            fut.await.map(ServiceResponse::map_into_left_body)
        })
    }
}

fn required_permission_for_path(path: &str, method: &str) -> Option<&'static str> {
    let path = path
        .strip_prefix("/api/v1/admin")
        .or_else(|| path.strip_prefix("/v1/admin"))
        .or_else(|| path.strip_prefix("/api/admin"))
        .or_else(|| path.strip_prefix("/admin"))?;

    if path.starts_with("/stats")
        || path.starts_with("/operations")
        || path.starts_with("/payments/dashboard")
        || path.starts_with("/payment-ops/dashboard")
    {
        return Some("payment_ops.view");
    }
    if path.starts_with("/merchants") {
        return Some("payment_ops.view");
    }
    if path.contains("/business-accounts") {
        return Some(if method == "GET" {
            "payment_ops.view"
        } else {
            "business_accounts.manage"
        });
    }
    if path.starts_with("/transactions")
        || path.starts_with("/payments/transactions")
        || path.starts_with("/payment-ops/transactions")
    {
        return Some(if method == "GET" {
            "transactions.view"
        } else {
            "transactions.action"
        });
    }
    if path.contains("/refunds") {
        if method == "GET" {
            return Some("refunds.request");
        }
        if path.ends_with("/approve") || path.ends_with("/reject") {
            return Some("refunds.approve");
        }
        if path.ends_with("/initiate") || path.ends_with("/retry") {
            return Some("refunds.initiate");
        }
        return Some("refunds.request");
    }
    if path.starts_with("/settlements") || path.contains("/settlements") {
        return Some("settlements.manage");
    }
    if path.contains("/ledger") {
        return Some(if path.ends_with("/export") {
            "ledger.export"
        } else {
            "ledger.view"
        });
    }
    if path.starts_with("/disputes") || path.contains("/disputes") {
        return Some("disputes.manage");
    }
    if path.contains("/risk-rules")
        || path.contains("/risk-reviews")
        || path.contains("/risk-signals")
        || path.contains("/risk-decisions")
        || path.contains("/risk-lists")
        || path.contains("/risk/simulate")
    {
        return Some("risk_rules.manage");
    }
    if path.contains("/reconciliation") {
        return Some("reconciliation.manage");
    }
    if path.contains("/developer") {
        if path.contains("/request-logs") {
            return Some("api_logs.view");
        }
        if path.contains("/api-keys") && method != "GET" {
            return Some("api_keys.manage");
        }
        if path.contains("/webhook-events") || path.contains("/webhook-test") {
            return Some(if method == "GET" {
                "developer_tools.view"
            } else {
                "webhook_replay.manage"
            });
        }
        return Some("developer_tools.view");
    }
    if path.contains("/readiness-scorecard") {
        return Some(if method == "GET" {
            "compliance.view"
        } else {
            "compliance.manage"
        });
    }
    if path.contains("/compliance") {
        return Some(if method == "GET" {
            "compliance.view"
        } else {
            "compliance.manage"
        });
    }
    if path.contains("/security-review") {
        return Some(if method == "GET" {
            "security_review.view"
        } else {
            "security_review.manage"
        });
    }
    if path.contains("/go-live") {
        return Some(if method == "GET" {
            "go_live.view"
        } else {
            "go_live.manage"
        });
    }
    if path.starts_with("/provider-credentials")
        || path.contains("/providers")
        || path.contains("/credentials")
        || path.contains("/connectors")
    {
        return Some(if method == "GET" {
            "providers.view"
        } else if path.contains("/credentials") || path.starts_with("/provider-credentials") {
            "credentials.rotate"
        } else {
            "providers.manage"
        });
    }
    if path.contains("/bank-integrations") || path.starts_with("/bank-accounts") {
        return Some("bank_integrations.manage");
    }
    if path.contains("/routing-rules")
        || path.contains("/routing/simulate")
        || path.contains("/routing/provider-performance")
    {
        return Some("routing.manage");
    }
    if path.contains("/money-movement") {
        return Some("settlements.manage");
    }
    if path.contains("/white-label-config")
        || path.contains("/white-label/")
        || path.contains("/webhook-configs")
        || path.contains("/webhook-events")
        || path.contains("/webhooks")
        || path.contains("/webhook-processing-attempts")
        || path.contains("/webhook-retry-policies")
    {
        return Some("webhook_config.manage");
    }
    if path.starts_with("/exports") || path.starts_with("/revenue") || path.contains("/reports") {
        return Some("reports.export");
    }
    if path.contains("/audit-logs") {
        return Some("audit_logs.view");
    }
    if path.contains("/admin-roles") {
        return Some(if method == "GET" {
            "roles.view"
        } else {
            "roles.manage"
        });
    }
    if path.starts_with("/users") {
        return Some(if method == "GET" {
            "admin_users.view"
        } else {
            "admin_users.manage"
        });
    }
    if path.starts_with("/system-health") || path.starts_with("/payment-ops/system-health") {
        return Some(if method == "GET" {
            "system_health.view"
        } else {
            "system_health.manage"
        });
    }
    if path.starts_with("/notifications") {
        return Some(if method == "GET" {
            "notifications.view"
        } else {
            "notifications.manage"
        });
    }
    if path.starts_with("/platform-settings") {
        return Some(if method == "GET" {
            "platform_settings.view"
        } else {
            "platform_settings.manage"
        });
    }
    None
}

fn role_has_permission(role: &str, permission: &str) -> bool {
    if matches!(role, "Super Admin" | "super_admin" | "admin") {
        return true;
    }
    permissions_for_role(role).contains(&permission)
}

fn permissions_for_role(role: &str) -> &'static [&'static str] {
    match role {
        "Payment Ops Admin" => &[
            "payment_ops.view",
            "transactions.view",
            "transactions.action",
            "refunds.request",
            "providers.view",
            "providers.manage",
            "credentials.rotate",
            "routing.manage",
            "webhook_config.manage",
            "system_health.view",
            "notifications.view",
            "ledger.view",
            "developer_tools.view",
            "api_keys.manage",
            "api_logs.view",
            "webhook_replay.manage",
        ],
        "Finance Admin" => &[
            "payment_ops.view",
            "transactions.view",
            "refunds.request",
            "refunds.approve",
            "refunds.initiate",
            "settlements.manage",
            "reconciliation.manage",
            "reports.export",
            "notifications.view",
            "ledger.view",
            "ledger.export",
        ],
        "Risk Admin" => &[
            "payment_ops.view",
            "transactions.view",
            "risk_rules.manage",
            "disputes.manage",
            "notifications.view",
            "business_accounts.manage",
            "business_accounts.kyc_review",
            "business_accounts.risk_review",
        ],
        "Support Admin" => &[
            "payment_ops.view",
            "transactions.view",
            "transactions.action",
            "refunds.request",
            "disputes.manage",
            "notifications.view",
            "notifications.manage",
            "business_accounts.manage",
        ],
        "Support Agent" => &[
            "payment_ops.view",
            "transactions.view",
            "refunds.request",
            "notifications.view",
        ],
        "Compliance Admin" => &[
            "payment_ops.view",
            "transactions.view",
            "disputes.manage",
            "audit_logs.view",
            "notifications.view",
            "business_accounts.kyc_review",
            "business_accounts.risk_review",
            "compliance.view",
            "compliance.manage",
            "security_review.view",
            "security_review.manage",
            "go_live.view",
            "go_live.manage",
            "security_events.view",
        ],
        "Auditor" | "Auditor / Read Only" => &[
            "payment_ops.view",
            "transactions.view",
            "providers.view",
            "audit_logs.view",
            "reports.export",
            "ledger.view",
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_routes_deny_unmapped_by_default() {
        assert_eq!(required_permission_for_path("/admin/unknown", "GET"), None);
    }

    #[test]
    fn reports_and_developer_routes_map_permissions() {
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/reports/generate", "POST"),
            Some("reports.export")
        );
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/developer/api-keys", "POST"),
            Some("api_keys.manage")
        );
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/developer/request-logs", "GET"),
            Some("api_logs.view")
        );
    }

    #[test]
    fn payment_ops_system_health_routes_map_permissions() {
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/system-health/metrics", "GET"),
            Some("system_health.view")
        );
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/system-health/slos/refresh", "POST"),
            Some("system_health.manage")
        );
    }

    #[test]
    fn connector_and_routing_routes_map_permissions() {
        assert_eq!(
            required_permission_for_path("/admin/connectors", "GET"),
            Some("providers.view")
        );
        assert_eq!(
            required_permission_for_path("/admin/payment-ops/routing/provider-performance", "GET"),
            Some("routing.manage")
        );
        assert!(role_has_permission("Payment Ops Admin", "providers.view"));
        assert!(role_has_permission("Auditor", "providers.view"));
        assert!(!role_has_permission("Auditor", "providers.manage"));
    }

    #[test]
    fn finance_admin_can_export_reports() {
        assert!(role_has_permission("Finance Admin", "reports.export"));
        assert!(!role_has_permission("Support Agent", "reports.export"));
    }
}

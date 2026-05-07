use actix_web::body::{to_bytes, BoxBody, EitherBody, MessageBody};
use actix_web::dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{header, Method, StatusCode};
use actix_web::{web, Error, HttpMessage, HttpResponse};
use chrono::{Duration, Utc};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use futures_util::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};
use uuid::Uuid;

use crate::middleware::auth::{MerchantId, RequestEnvironment, TenantId, UserId, UserRole};
use crate::models::user::Claims;

const CURRENT_VERSION: &str = "2026-05-05";
const DEPRECATED_VERSION: &str = "2025-12-01";

#[derive(Clone, Debug)]
pub struct ExternalApiKeyContext {
    pub api_key_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub environment: String,
}

#[derive(Clone)]
struct IdempotencyValue {
    request_hash: String,
    status: StatusCode,
    body: web::Bytes,
}

#[derive(Clone, Default)]
pub struct ExternalApiMemory {
    idempotency: Arc<Mutex<HashMap<String, IdempotencyValue>>>,
    rate_limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Clone)]
pub struct ExternalApiGateway {
    memory: ExternalApiMemory,
    max_requests: usize,
}

impl ExternalApiGateway {
    pub fn new() -> Self {
        Self {
            memory: ExternalApiMemory::default(),
            max_requests: 120,
        }
    }

    #[cfg(test)]
    fn with_max_requests(max_requests: usize) -> Self {
        Self {
            memory: ExternalApiMemory::default(),
            max_requests,
        }
    }
}

impl Default for ExternalApiGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for ExternalApiGateway
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = ExternalApiGatewayService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ExternalApiGatewayService {
            service: Rc::new(service),
            memory: self.memory.clone(),
            max_requests: self.max_requests,
        }))
    }
}

pub struct ExternalApiGatewayService<S> {
    service: Rc<S>,
    memory: ExternalApiMemory,
    max_requests: usize,
}

impl<S, B> Service<ServiceRequest> for ExternalApiGatewayService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let memory = self.memory.clone();
        let max_requests = self.max_requests;

        Box::pin(async move {
            let started = Instant::now();
            let request_id = request_id(&req);
            let path = req.path().to_string();
            let method = req.method().clone();

            if method == Method::OPTIONS {
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            let Some(api_key) = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
            else {
                if try_jwt_context(&req).await {
                    let res = service.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
                return external_error(
                    req,
                    StatusCode::UNAUTHORIZED,
                    &request_id,
                    "invalid_api_key",
                    "Missing x-api-key header",
                );
            };

            let version = req
                .headers()
                .get("x-api-version")
                .and_then(|v| v.to_str().ok())
                .unwrap_or(CURRENT_VERSION)
                .trim()
                .to_string();
            let version_status = api_version_status(&req, &version).await;
            if version_status == "unsupported" {
                return external_error(
                    req,
                    StatusCode::BAD_REQUEST,
                    &request_id,
                    "unsupported_api_version",
                    "Unsupported API version",
                );
            }

            let auth = match authenticate_api_key(&req, &api_key).await {
                Ok(ctx) => ctx,
                Err(status) => {
                    return external_error(
                        req,
                        status,
                        &request_id,
                        "invalid_api_key",
                        "Invalid API key",
                    );
                }
            };

            if !scope_allowed(&auth.scopes, &method, &path) {
                return external_error(
                    req,
                    StatusCode::FORBIDDEN,
                    &request_id,
                    "scope_denied",
                    "API key scope does not allow this operation",
                );
            }

            if !rate_limit_allowed(&req, &memory, auth.api_key_id, &api_key, max_requests).await {
                log_external_request(
                    req.request(),
                    &auth,
                    &request_id,
                    started,
                    429,
                    Some("rate_limit_exceeded"),
                    None,
                )
                .await;
                return external_error(
                    req,
                    StatusCode::TOO_MANY_REQUESTS,
                    &request_id,
                    "rate_limit_exceeded",
                    "Request rate limit exceeded",
                );
            }

            let body = read_payload(req.take_payload()).await?;
            let request_hash = hash_bytes(&body);
            let idempotency_key = req
                .headers()
                .get("idempotency-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);

            if is_write(&method) {
                let Some(key) = idempotency_key.clone() else {
                    return external_error(
                        req,
                        StatusCode::BAD_REQUEST,
                        &request_id,
                        "idempotency_key_required",
                        "Idempotency-Key header required",
                    );
                };

                if let Some(replay) = find_idempotency(
                    &req,
                    &memory,
                    auth.api_key_id,
                    &api_key,
                    &key,
                    &request_hash,
                )
                .await
                {
                    log_external_request(
                        req.request(),
                        &auth,
                        &request_id,
                        started,
                        replay.status.as_u16() as i32,
                        None,
                        idempotency_key.as_deref(),
                    )
                    .await;
                    let mut response = HttpResponse::build(replay.status);
                    response.insert_header(("x-request-id", request_id));
                    response.insert_header(("x-idempotent-replay", "true"));
                    return Ok(req.into_response(response.body(replay.body).map_into_right_body()));
                }

                if idempotency_conflict(
                    &req,
                    &memory,
                    auth.api_key_id,
                    &api_key,
                    &key,
                    &request_hash,
                )
                .await
                {
                    log_external_request(
                        req.request(),
                        &auth,
                        &request_id,
                        started,
                        409,
                        Some("idempotency_conflict"),
                        idempotency_key.as_deref(),
                    )
                    .await;
                    return external_error(
                        req,
                        StatusCode::CONFLICT,
                        &request_id,
                        "idempotency_conflict",
                        "Idempotency key reused with different payload",
                    );
                }
            }

            req.set_payload(Payload::from(body.clone()));
            let mut res = service.call(req).await?;
            res.headers_mut().insert(
                header::HeaderName::from_static("x-request-id"),
                header::HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("invalid")),
            );
            res.headers_mut().insert(
                header::HeaderName::from_static("x-api-version"),
                header::HeaderValue::from_str(&version)
                    .unwrap_or_else(|_| header::HeaderValue::from_static(CURRENT_VERSION)),
            );
            if version_status == "deprecated" {
                res.headers_mut().insert(
                    header::HeaderName::from_static("x-api-version-warning"),
                    header::HeaderValue::from_static(
                        "Deprecated API version; migrate before sunset",
                    ),
                );
            }

            let status = res.status();
            let (req_head, res) = res.into_parts();
            let response_body = to_bytes(res.into_body()).await.map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to read response body")
            })?;
            if is_write(&method) {
                if let Some(key) = idempotency_key.as_deref() {
                    store_idempotency(
                        &req_head,
                        &memory,
                        auth.api_key_id,
                        &api_key,
                        key,
                        &request_hash,
                        status,
                        response_body.clone(),
                    )
                    .await;
                }
            }
            let error_code = if status.is_success() {
                None
            } else {
                Some("external_api_error")
            };
            log_external_request(
                &req_head,
                &auth,
                &request_id,
                started,
                status.as_u16() as i32,
                error_code,
                idempotency_key.as_deref(),
            )
            .await;
            Ok(ServiceResponse::new(
                req_head,
                HttpResponse::build(status)
                    .body(response_body)
                    .map_into_right_body(),
            ))
        })
    }
}

fn is_write(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

fn request_id(req: &ServiceRequest) -> String {
    req.headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("req_{}", Uuid::new_v4().simple()))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn hash_secret(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn external_error<B>(
    req: ServiceRequest,
    status: StatusCode,
    request_id: &str,
    code: &str,
    message: &str,
) -> Result<ServiceResponse<EitherBody<B, BoxBody>>, Error>
where
    B: MessageBody + 'static,
{
    let mut response = HttpResponse::build(status);
    response.insert_header(("x-request-id", request_id.to_string()));
    Ok(req.into_response(
        response
            .json(json!({
                "request_id": request_id,
                "error": {
                    "code": code,
                    "message": message,
                    "http_status": status.as_u16()
                }
            }))
            .map_into_right_body(),
    ))
}

async fn read_payload(mut payload: Payload) -> Result<web::Bytes, Error> {
    let mut body = actix_web::web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body.freeze())
}

async fn api_version_status(req: &ServiceRequest, version: &str) -> String {
    if let Some(state) = req.app_data::<web::Data<crate::state::AppState>>() {
        if let Ok(row) = sqlx::query("SELECT status FROM api_versions WHERE version = $1")
            .bind(version)
            .fetch_optional(&state.db)
            .await
        {
            return row
                .map(|row| row.get::<String, _>("status"))
                .unwrap_or_else(|| "unsupported".to_string());
        }
    }
    match version {
        CURRENT_VERSION => "current".to_string(),
        DEPRECATED_VERSION => "deprecated".to_string(),
        "2026-04-01" => "supported".to_string(),
        _ => "unsupported".to_string(),
    }
}

async fn authenticate_api_key(
    req: &ServiceRequest,
    api_key: &str,
) -> Result<ExternalApiKeyContext, StatusCode> {
    let environment = req
        .headers()
        .get("x-feelpay-environment")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("test")
        .trim()
        .to_ascii_lowercase();

    if let Some(state) = req.app_data::<web::Data<crate::state::AppState>>() {
        let hash = hash_secret(api_key);
        let key_row = sqlx::query(
            r#"
            SELECT id, merchant_id, scopes, environment, status, expires_at
            FROM developer_api_keys
            WHERE encrypted_key_hash = $1
            LIMIT 1
            "#,
        )
        .bind(&hash)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(row) = key_row {
            let status: String = Row::get(&row, "status");
            let expires_at: Option<chrono::DateTime<Utc>> = Row::get(&row, "expires_at");
            if status != "active" || expires_at.is_some_and(|expires| expires <= Utc::now()) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let key_env: String = Row::get(&row, "environment");
            if key_env != environment && !(key_env == "sandbox" && environment == "test") {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let api_key_id: Uuid = Row::get(&row, "id");
            let merchant_id: Option<Uuid> = Row::try_get(&row, "merchant_id").ok();
            let scopes_json: Value = Row::get(&row, "scopes");
            let scopes = scopes_json
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            let _ = sqlx::query("UPDATE developer_api_keys SET last_used_at = NOW() WHERE id = $1")
                .bind(api_key_id)
                .execute(&state.db)
                .await;
            req.extensions_mut().insert(ExternalApiKeyContext {
                api_key_id: Some(api_key_id),
                merchant_id,
                scopes: scopes.clone(),
                environment: environment.clone(),
            });
            return Ok(ExternalApiKeyContext {
                api_key_id: Some(api_key_id),
                merchant_id,
                scopes,
                environment,
            });
        }

        let merchant = sqlx::query(
            "SELECT id, tenant_id FROM merchants WHERE status = 'active' AND api_key = $1 LIMIT 1",
        )
        .bind(api_key)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if let Some(row) = merchant {
            let merchant_id: Uuid = row.get("id");
            let tenant_id: Option<Uuid> = row.get("tenant_id");
            req.extensions_mut().insert(MerchantId(merchant_id));
            if let Some(tid) = tenant_id {
                req.extensions_mut().insert(TenantId(tid));
            }
            req.extensions_mut()
                .insert(RequestEnvironment(if environment == "live" {
                    crate::middleware::auth::EnvironmentMode::Live
                } else {
                    crate::middleware::auth::EnvironmentMode::Test
                }));
            let scopes = vec!["*".to_string()];
            req.extensions_mut().insert(ExternalApiKeyContext {
                api_key_id: None,
                merchant_id: Some(merchant_id),
                scopes: scopes.clone(),
                environment: environment.clone(),
            });
            return Ok(ExternalApiKeyContext {
                api_key_id: None,
                merchant_id: Some(merchant_id),
                scopes,
                environment,
            });
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn try_jwt_context(req: &ServiceRequest) -> bool {
    let Some(token) = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
                .map(str::trim)
        })
        .filter(|token| !token.is_empty())
    else {
        return false;
    };

    let Ok(secret) = crate::middleware::auth::jwt_secret() else {
        return false;
    };
    let Ok(token_data) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) else {
        return false;
    };
    let claims = token_data.claims;
    let Ok(user_id) = Uuid::parse_str(&claims.sub) else {
        return false;
    };
    req.extensions_mut().insert(UserId(user_id));
    if let Some(tid) = claims.tenant_id {
        req.extensions_mut().insert(TenantId(tid));
    }
    if let Some(mid) = claims.merchant_id {
        req.extensions_mut().insert(MerchantId(mid));
    }
    req.extensions_mut().insert(UserRole(claims.role));
    true
}

fn scope_allowed(scopes: &[String], method: &Method, path: &str) -> bool {
    if scopes.iter().any(|scope| scope == "*") {
        return true;
    }
    let resource = if path.starts_with("/v1/payment-links") {
        "payment_links"
    } else if path.starts_with("/v1/payments") {
        "payments"
    } else if path.starts_with("/v1/refunds") || path.contains("/refund") {
        "refunds"
    } else if path.starts_with("/v1/payment-sessions") {
        "payment_sessions"
    } else if path.starts_with("/v1/webhook-test") {
        "webhooks"
    } else {
        "api"
    };
    let action = if *method == Method::GET {
        "read"
    } else {
        "write"
    };
    scopes.iter().any(|scope| {
        scope == &format!("{resource}.{action}")
            || scope == &format!("{resource}.*")
            || scope == &format!("*.{action}")
    })
}

async fn rate_limit_allowed(
    req: &ServiceRequest,
    memory: &ExternalApiMemory,
    api_key_id: Option<Uuid>,
    api_key: &str,
    default_max: usize,
) -> bool {
    if let (Some(state), Some(id)) = (
        req.app_data::<web::Data<crate::state::AppState>>(),
        api_key_id,
    ) {
        if let Ok(Some(row)) = sqlx::query(
            "SELECT id, max_requests, current_count, reset_at FROM api_rate_limits WHERE api_key_id = $1 ORDER BY reset_at DESC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        {
            let row_id: Uuid = row.get("id");
            let max_requests: i32 = row.get("max_requests");
            let mut current_count: i32 = row.get("current_count");
            let reset_at: chrono::DateTime<Utc> = row.get("reset_at");
            if Utc::now() > reset_at {
                current_count = 0;
                let _ = sqlx::query("UPDATE api_rate_limits SET current_count = 0, reset_at = NOW() + (window_seconds || ' seconds')::interval WHERE id = $1")
                    .bind(row_id)
                    .execute(&state.db)
                    .await;
            }
            if current_count >= max_requests {
                return false;
            }
            let _ = sqlx::query("UPDATE api_rate_limits SET current_count = current_count + 1 WHERE id = $1")
                .bind(row_id)
                .execute(&state.db)
                .await;
            return true;
        }
    }

    let key = api_key_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| hash_secret(api_key));
    let mut map = match memory.rate_limits.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    let now = Instant::now();
    let window = StdDuration::from_secs(60);
    let hits = map.entry(key).or_default();
    hits.retain(|hit| now.duration_since(*hit) < window);
    if hits.len() >= default_max {
        return false;
    }
    hits.push(now);
    true
}

async fn find_idempotency(
    req: &ServiceRequest,
    memory: &ExternalApiMemory,
    api_key_id: Option<Uuid>,
    api_key: &str,
    key: &str,
    request_hash: &str,
) -> Option<IdempotencyValue> {
    let memory_key = format!(
        "{}:{key}",
        api_key_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| hash_secret(api_key))
    );
    if let Ok(map) = memory.idempotency.lock() {
        if let Some(value) = map.get(&memory_key) {
            if value.request_hash == request_hash {
                return Some(value.clone());
            }
        }
    }

    if let (Some(state), Some(id)) = (
        req.app_data::<web::Data<crate::state::AppState>>(),
        api_key_id,
    ) {
        if let Ok(Some(row)) = sqlx::query(
            "SELECT request_hash, response_status, response_body FROM api_idempotency_records WHERE idempotency_key = $1 AND api_key_id = $2 AND expires_at > NOW()",
        )
        .bind(key)
        .bind(id)
        .fetch_optional(&state.db)
        .await
        {
            let existing_hash: String = row.get("request_hash");
            if existing_hash == request_hash {
                let status = StatusCode::from_u16(row.get::<i32, _>("response_status") as u16).unwrap_or(StatusCode::OK);
                let body_value: Option<Value> = row.try_get("response_body").ok();
                let body = body_value
                    .map(|value| web::Bytes::from(value.to_string()))
                    .unwrap_or_else(|| web::Bytes::from_static(b"{}"));
                return Some(IdempotencyValue { request_hash: existing_hash, status, body });
            }
        }
    }
    None
}

async fn idempotency_conflict(
    req: &ServiceRequest,
    memory: &ExternalApiMemory,
    api_key_id: Option<Uuid>,
    api_key: &str,
    key: &str,
    request_hash: &str,
) -> bool {
    let memory_key = format!(
        "{}:{key}",
        api_key_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| hash_secret(api_key))
    );
    if let Ok(map) = memory.idempotency.lock() {
        if let Some(value) = map.get(&memory_key) {
            return value.request_hash != request_hash;
        }
    }
    if let (Some(state), Some(id)) = (
        req.app_data::<web::Data<crate::state::AppState>>(),
        api_key_id,
    ) {
        if let Ok(Some(existing_hash)) = sqlx::query_scalar::<_, String>(
            "SELECT request_hash FROM api_idempotency_records WHERE idempotency_key = $1 AND api_key_id = $2 AND expires_at > NOW()",
        )
        .bind(key)
        .bind(id)
        .fetch_optional(&state.db)
        .await
        {
            return existing_hash != request_hash;
        }
    }
    false
}

async fn store_idempotency(
    req: &actix_web::HttpRequest,
    memory: &ExternalApiMemory,
    api_key_id: Option<Uuid>,
    api_key: &str,
    key: &str,
    request_hash: &str,
    status: StatusCode,
    body: web::Bytes,
) {
    let memory_key = format!(
        "{}:{key}",
        api_key_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| hash_secret(api_key))
    );
    if let Ok(mut map) = memory.idempotency.lock() {
        map.entry(memory_key).or_insert(IdempotencyValue {
            request_hash: request_hash.to_string(),
            status,
            body: body.clone(),
        });
    }
    if let (Some(state), Some(id)) = (
        req.app_data::<web::Data<crate::state::AppState>>(),
        api_key_id,
    ) {
        let response_json = serde_json::from_slice::<Value>(&body)
            .unwrap_or_else(|_| json!({ "body": String::from_utf8_lossy(&body) }));
        let _ = sqlx::query(
            r#"
            INSERT INTO api_idempotency_records
                (idempotency_key, api_key_id, request_hash, response_hash, response_status, response_body, resource_type, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6::jsonb, 'external_api', $7)
            ON CONFLICT (idempotency_key, api_key_id) DO NOTHING
            "#,
        )
        .bind(key)
        .bind(id)
        .bind(request_hash)
        .bind(hash_bytes(&body))
        .bind(status.as_u16() as i32)
        .bind(response_json)
        .bind(Utc::now() + Duration::hours(24))
        .execute(&state.db)
        .await;
    }
}

fn redact_headers(req: &actix_web::HttpRequest) -> Value {
    let mut object = serde_json::Map::new();
    for (name, value) in req.headers() {
        let key = name.as_str().to_ascii_lowercase();
        let value = if key.contains("authorization")
            || key.contains("api-key")
            || key.contains("secret")
            || key.contains("token")
        {
            "redacted".to_string()
        } else {
            value.to_str().unwrap_or_default().to_string()
        };
        object.insert(key, json!(value));
    }
    Value::Object(object)
}

async fn log_external_request(
    req: &actix_web::HttpRequest,
    auth: &ExternalApiKeyContext,
    request_id: &str,
    started: Instant,
    status_code: i32,
    error_code: Option<&str>,
    idempotency_key: Option<&str>,
) {
    if let Some(state) = req.app_data::<web::Data<crate::state::AppState>>() {
        let latency_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
        let _ = sqlx::query(
            r#"
            INSERT INTO api_request_logs
                (environment, merchant_id, api_key_id, method, path, status_code, latency_ms, idempotency_key, request_id, error_code, request_metadata, response_summary)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::jsonb, $12::jsonb)
            "#,
        )
        .bind(&auth.environment)
        .bind(auth.merchant_id)
        .bind(auth.api_key_id)
        .bind(req.method().as_str())
        .bind(req.path())
        .bind(status_code)
        .bind(latency_ms)
        .bind(idempotency_key)
        .bind(request_id)
        .bind(error_code)
        .bind(json!({ "headers": redact_headers(req), "query": req.query_string() }))
        .bind(json!({ "status_code": status_code }))
        .execute(&state.db)
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sensitive_headers() {
        let req = actix_web::test::TestRequest::get()
            .insert_header(("x-api-key", "fp_test_secret"))
            .insert_header(("authorization", "Bearer token"))
            .insert_header(("x-request-id", "req_ok"))
            .to_http_request();
        let redacted = redact_headers(&req);
        assert_eq!(redacted["x-api-key"], "redacted");
        assert_eq!(redacted["authorization"], "redacted");
        assert_eq!(redacted["x-request-id"], "req_ok");
    }

    #[test]
    fn scopes_match_resource_action() {
        assert!(scope_allowed(
            &["payments.write".to_string()],
            &Method::POST,
            "/v1/payments"
        ));
        assert!(!scope_allowed(
            &["payments.read".to_string()],
            &Method::POST,
            "/v1/payments"
        ));
        assert!(scope_allowed(
            &["*".to_string()],
            &Method::DELETE,
            "/v1/payment-links/1"
        ));
    }

    #[test]
    fn hash_changes_with_body() {
        assert_eq!(hash_bytes(br#"{"a":1}"#), hash_bytes(br#"{"a":1}"#));
        assert_ne!(hash_bytes(br#"{"a":1}"#), hash_bytes(br#"{"a":2}"#));
    }

    #[test]
    fn gateway_test_constructor_sets_limit() {
        assert_eq!(ExternalApiGateway::with_max_requests(1).max_requests, 1);
    }
}

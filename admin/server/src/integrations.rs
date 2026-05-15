use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::env;
use uuid::Uuid;

use crate::middleware::AdminClaims;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationKind {
    EmailProvider,
    SmsProvider,
    OtpProvider,
    BankPayoutProvider,
    KycProvider,
    NotificationProvider,
    ReconciliationProvider,
}

pub trait IntegrationAdapter {
    fn kind(&self) -> IntegrationKind;
    fn supports_live_mode(&self) -> bool;
}

struct GenericAdapter {
    kind: IntegrationKind,
}

impl IntegrationAdapter for GenericAdapter {
    fn kind(&self) -> IntegrationKind {
        self.kind.clone()
    }

    fn supports_live_mode(&self) -> bool {
        !matches!(self.kind, IntegrationKind::BankPayoutProvider)
    }
}

#[derive(Debug, Deserialize)]
struct UpsertProviderRequest {
    id: Option<Uuid>,
    provider_kind: String,
    name: String,
    code: String,
    environment: String,
    status: Option<String>,
    is_default: Option<bool>,
    priority: Option<i32>,
    timeout_ms: Option<i32>,
    max_retries: Option<i32>,
    retry_backoff_ms: Option<i32>,
    config: Option<Value>,
    live_enable_approved: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SecretRequest {
    secret_name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TestActionRequest {
    recipient: Option<String>,
    phone: Option<String>,
    otp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StatusRequest {
    enabled: bool,
    live_enable_approved: Option<bool>,
}

#[derive(Debug, Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_name: String,
    from_email: String,
    reply_to: Option<String>,
    security: String,
}

#[derive(Debug, Clone)]
struct GenericSmsConfig {
    base_url: String,
    method: String,
    headers_template: serde_json::Map<String, Value>,
    body_template: Value,
    sender_id: String,
    route_type: Option<String>,
    template_mapping: serde_json::Map<String, Value>,
    auth_token: String,
}

#[derive(Debug, Serialize)]
struct ProviderResponse {
    id: Uuid,
    provider_kind: String,
    name: String,
    code: String,
    environment: String,
    status: String,
    is_default: bool,
    priority: i32,
    timeout_ms: i32,
    max_retries: i32,
    retry_backoff_ms: i32,
    health_status: String,
    last_success_at: Option<DateTime<Utc>>,
    last_failure_at: Option<DateTime<Utc>>,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
    config: Value,
    masked_secrets: Vec<Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

const KINDS: &[&str] = &[
    "email",
    "sms",
    "otp",
    "bank_payout",
    "kyc",
    "notification",
    "reconciliation",
];

fn adapter_for_kind(kind: &str) -> Option<Box<dyn IntegrationAdapter>> {
    let kind = match kind {
        "email" => IntegrationKind::EmailProvider,
        "sms" => IntegrationKind::SmsProvider,
        "otp" => IntegrationKind::OtpProvider,
        "bank_payout" => IntegrationKind::BankPayoutProvider,
        "kyc" => IntegrationKind::KycProvider,
        "notification" => IntegrationKind::NotificationProvider,
        "reconciliation" => IntegrationKind::ReconciliationProvider,
        _ => return None,
    };
    Some(Box::new(GenericAdapter { kind }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/integrations")
            .route("/providers", web::get().to(list_providers))
            .route("/providers", web::post().to(upsert_provider))
            .route("/providers/{id}", web::put().to(update_provider))
            .route("/providers/{id}/status", web::post().to(set_status))
            .route("/providers/{id}/default", web::post().to(set_default))
            .route("/providers/{id}/secrets", web::post().to(upsert_secret))
            .route(
                "/providers/{id}/secrets/{secret_name}/rotate",
                web::post().to(rotate_secret),
            )
            .route(
                "/providers/{id}/test-connection",
                web::post().to(test_connection),
            )
            .route(
                "/providers/{id}/test-email",
                web::post().to(send_test_email),
            )
            .route(
                "/providers/default/email/{environment}/test-email",
                web::post().to(send_default_test_email),
            )
            .route("/providers/{id}/test-sms", web::post().to(send_test_sms))
            .route("/providers/{id}/test-otp", web::post().to(send_test_otp))
            .route(
                "/providers/{id}/test-bank-payout",
                web::post().to(test_bank_payout),
            )
            .route("/providers/{id}/logs", web::get().to(list_provider_logs))
            .route("/logs", web::get().to(list_logs)),
    );
}

fn actor(request: &HttpRequest) -> Option<Uuid> {
    request
        .extensions()
        .get::<AdminClaims>()
        .map(|claims| claims.admin_id)
}

fn mask_secret(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    format!(
        "****{}",
        chars[chars.len() - 4..].iter().collect::<String>()
    )
}

fn encrypt_secret(value: &str) -> Result<(String, String), String> {
    let raw_key = env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY is required to save integration secrets".to_string())?;
    let key = hex::decode(raw_key.trim())
        .map_err(|_| "VAULT_ENCRYPTION_KEY must be 64 hex characters".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string())?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), value.as_bytes())
        .map_err(|_| "Credential encryption failed".to_string())?;
    Ok((BASE64.encode(encrypted), BASE64.encode(nonce_bytes)))
}

fn decrypt_secret(encrypted_value: &str, nonce: &str) -> Result<String, String> {
    let raw_key = env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY is required to read integration secrets".to_string())?;
    let key = hex::decode(raw_key.trim())
        .map_err(|_| "VAULT_ENCRYPTION_KEY must be 64 hex characters".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string())?;
    let encrypted = BASE64
        .decode(encrypted_value)
        .map_err(|_| "Stored credential is invalid".to_string())?;
    let nonce = BASE64
        .decode(nonce)
        .map_err(|_| "Stored credential nonce is invalid".to_string())?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
        .map_err(|_| "Credential decryption failed".to_string())?;
    String::from_utf8(decrypted).map_err(|_| "Stored credential is not valid UTF-8".to_string())
}

async fn load_secret(
    pool: &PgPool,
    provider_id: Uuid,
    secret_name: &str,
) -> Result<String, String> {
    let row = sqlx::query(
        "SELECT encrypted_value, nonce FROM integration_provider_secrets WHERE provider_id = $1 AND secret_name = $2",
    )
    .bind(provider_id)
    .bind(secret_name)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Failed to load integration secret".to_string())?;
    let Some(row) = row else {
        return Err(format!("Missing required secret: {secret_name}"));
    };
    decrypt_secret(row.get("encrypted_value"), row.get("nonce"))
}

fn required_string(config: &Value, key: &str) -> Result<String, String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required config: {key}"))
}

async fn smtp_config(
    pool: &PgPool,
    provider_id: Uuid,
    config: &Value,
) -> Result<SmtpConfig, String> {
    let port = config
        .get("smtp_port")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(587);
    let security = config
        .get("smtp_security")
        .and_then(Value::as_str)
        .unwrap_or("starttls")
        .to_string();
    if !matches!(security.as_str(), "starttls" | "tls" | "none") {
        return Err("smtp_security must be starttls, tls, or none".to_string());
    }
    Ok(SmtpConfig {
        host: required_string(config, "smtp_host")?,
        port,
        username: required_string(config, "smtp_user")?,
        password: load_secret(pool, provider_id, "password").await?,
        from_name: required_string(config, "from_name")?,
        from_email: required_string(config, "from_email")?,
        reply_to: config
            .get("reply_to")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        security,
    })
}

fn smtp_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = match config.security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host),
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &config.host,
        )),
        _ => unreachable!(),
    }
    .map_err(|_| "Invalid SMTP host or TLS configuration".to_string())?;
    Ok(builder
        .port(config.port)
        .credentials(Credentials::new(
            config.username.clone(),
            config.password.clone(),
        ))
        .build())
}

fn parse_mailbox(value: &str) -> Result<Mailbox, String> {
    value
        .parse::<Mailbox>()
        .map_err(|_| "Invalid email address".to_string())
}

fn mask_phone(value: &str) -> String {
    let digits = value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.len() <= 4 {
        return "****".to_string();
    }
    format!("******{}", &digits[digits.len() - 4..])
}

fn validate_sms_url(environment: &str, value: &str) -> Result<(), String> {
    if value.starts_with("https://") {
        return Ok(());
    }
    if environment == "test"
        && (value.starts_with("http://localhost") || value.starts_with("http://127.0.0.1"))
    {
        return Ok(());
    }
    Err("sms_base_url must use https, except localhost in test mode".to_string())
}

fn render_sms_value(
    value: &Value,
    phone: &str,
    message: &str,
    sender_id: &str,
    route_type: Option<&str>,
    template_id: Option<&str>,
    auth_token: &str,
) -> Result<Value, String> {
    match value {
        Value::String(input) => {
            let rendered = input
                .replace("{{phone}}", phone)
                .replace("{{message}}", message)
                .replace("{{sender_id}}", sender_id)
                .replace("{{route_type}}", route_type.unwrap_or_default())
                .replace("{{template_id}}", template_id.unwrap_or_default())
                .replace("{{token}}", auth_token);
            if rendered.contains("{{") || rendered.contains("}}") {
                return Err("Unsupported SMS template placeholder".to_string());
            }
            Ok(Value::String(rendered))
        }
        Value::Array(items) => Ok(Value::Array(
            items
                .iter()
                .map(|item| {
                    render_sms_value(
                        item,
                        phone,
                        message,
                        sender_id,
                        route_type,
                        template_id,
                        auth_token,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Object(map) => Ok(Value::Object(
            map.iter()
                .map(|(key, value)| {
                    Ok((
                        key.clone(),
                        render_sms_value(
                            value,
                            phone,
                            message,
                            sender_id,
                            route_type,
                            template_id,
                            auth_token,
                        )?,
                    ))
                })
                .collect::<Result<serde_json::Map<_, _>, String>>()?,
        )),
        _ => Ok(value.clone()),
    }
}

async fn generic_sms_config(
    pool: &PgPool,
    provider_id: Uuid,
    environment: &str,
    config: &Value,
) -> Result<GenericSmsConfig, String> {
    let base_url = required_string(config, "sms_base_url")?;
    validate_sms_url(environment, &base_url)?;
    let method = required_string(config, "sms_http_method")?.to_uppercase();
    if !matches!(method.as_str(), "POST" | "PUT") {
        return Err("sms_http_method must be POST or PUT".to_string());
    }
    let headers_template = config
        .get("sms_headers_template")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| "Missing required SMS config: sms_headers_template".to_string())?;
    for value in headers_template.values() {
        if !value.is_string() {
            return Err("sms_headers_template values must be strings".to_string());
        }
    }
    let body_template = config
        .get("sms_body_template")
        .cloned()
        .ok_or_else(|| "Missing required SMS config: sms_body_template".to_string())?;
    if !body_template.is_object() {
        return Err("sms_body_template must be a JSON object".to_string());
    }
    Ok(GenericSmsConfig {
        base_url,
        method,
        headers_template,
        body_template,
        sender_id: required_string(config, "sender_id")?,
        route_type: config
            .get("route_type")
            .and_then(Value::as_str)
            .map(str::to_string),
        template_mapping: config
            .get("template_mapping")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default(),
        auth_token: load_secret(pool, provider_id, "auth_token").await?,
    })
}

async fn send_generic_sms(
    config: &GenericSmsConfig,
    phone: &str,
    message: &str,
    template_key: &str,
) -> Result<(), String> {
    let template_id = config
        .template_mapping
        .get(template_key)
        .and_then(Value::as_str);
    let rendered_body = render_sms_value(
        &config.body_template,
        phone,
        message,
        &config.sender_id,
        config.route_type.as_deref(),
        template_id,
        &config.auth_token,
    )?;
    let client = reqwest::Client::new();
    let mut request = match config.method.as_str() {
        "POST" => client.post(&config.base_url),
        "PUT" => client.put(&config.base_url),
        _ => unreachable!(),
    };
    for (name, value) in &config.headers_template {
        let rendered = render_sms_value(
            value,
            phone,
            message,
            &config.sender_id,
            config.route_type.as_deref(),
            template_id,
            &config.auth_token,
        )?;
        request = request.header(
            name,
            rendered
                .as_str()
                .ok_or_else(|| "Rendered SMS header must be a string".to_string())?,
        );
    }
    let response = request
        .json(&rendered_body)
        .send()
        .await
        .map_err(|_| "SMS provider request failed".to_string())?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("SMS provider returned {}", response.status()))
    }
}

async fn ensure_internal_admin(
    _pool: &PgPool,
    request: &HttpRequest,
) -> Result<Uuid, HttpResponse> {
    let Some(actor_id) = actor(request) else {
        return Err(HttpResponse::Unauthorized().json(json!({ "message": "Admin auth required" })));
    };
    let role = request
        .extensions()
        .get::<AdminClaims>()
        .and_then(|claims| claims.role.clone())
        .unwrap_or_default();
    if !matches!(role.as_str(), "Super Admin" | "super_admin" | "admin") {
        return Err(HttpResponse::Forbidden().json(json!({
            "message": "Internal admin or super admin required"
        })));
    }
    Ok(actor_id)
}

async fn audit(
    pool: &PgPool,
    request: &HttpRequest,
    action: &str,
    entity_id: Option<Uuid>,
    summary: Value,
) {
    let actor_id = actor(request);
    let role = request
        .extensions()
        .get::<AdminClaims>()
        .and_then(|claims| claims.role.clone())
        .unwrap_or_else(|| "admin".into());
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs_company (actor_admin_id, actor_role, action, entity_type, entity_id, new_value_summary)
        VALUES ($1, $2, $3, 'integration_provider', $4, $5::jsonb)
        "#,
    )
    .bind(actor_id)
    .bind(role)
    .bind(action)
    .bind(entity_id)
    .bind(summary)
    .execute(pool)
    .await;
}

async fn write_log(
    pool: &PgPool,
    provider_id: Option<Uuid>,
    request: &HttpRequest,
    action: &str,
    request_type: &str,
    status: &str,
    latency_ms: i32,
    error_code: Option<&str>,
    error_message: Option<&str>,
    request_summary: Value,
    response_summary: Value,
) {
    let _ = sqlx::query(
        r#"
        INSERT INTO integration_logs (
            provider_id, action, request_type, status, latency_ms, error_code,
            error_message, actor_admin_id, request_summary, response_summary
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10::jsonb)
        "#,
    )
    .bind(provider_id)
    .bind(action)
    .bind(request_type)
    .bind(status)
    .bind(latency_ms)
    .bind(error_code)
    .bind(error_message)
    .bind(actor(request))
    .bind(request_summary)
    .bind(response_summary)
    .execute(pool)
    .await;
}

async fn provider_json(pool: &PgPool, id: Uuid) -> Result<Option<ProviderResponse>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, provider_kind, name, code, environment, status, is_default, priority,
               timeout_ms, max_retries, retry_backoff_ms, health_status, last_success_at,
               last_failure_at, last_error_code, last_error_message, config, created_at, updated_at
        FROM integration_providers
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let masked_secrets = sqlx::query(
        "SELECT secret_name, masked_value, rotated_at FROM integration_provider_secrets WHERE provider_id = $1 ORDER BY secret_name",
    )
    .bind(id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|secret| json!({
        "secret_name": secret.get::<String, _>("secret_name"),
        "masked_value": secret.get::<String, _>("masked_value"),
        "rotated_at": secret.get::<DateTime<Utc>, _>("rotated_at"),
    }))
    .collect();
    Ok(Some(ProviderResponse {
        id: row.get("id"),
        provider_kind: row.get("provider_kind"),
        name: row.get("name"),
        code: row.get("code"),
        environment: row.get("environment"),
        status: row.get("status"),
        is_default: row.get("is_default"),
        priority: row.get("priority"),
        timeout_ms: row.get("timeout_ms"),
        max_retries: row.get("max_retries"),
        retry_backoff_ms: row.get("retry_backoff_ms"),
        health_status: row.get("health_status"),
        last_success_at: row.get("last_success_at"),
        last_failure_at: row.get("last_failure_at"),
        last_error_code: row.get("last_error_code"),
        last_error_message: row.get("last_error_message"),
        config: row.get("config"),
        masked_secrets,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

async fn list_providers(pool: web::Data<PgPool>, request: HttpRequest) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let ids = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM integration_providers ORDER BY provider_kind, environment, priority, name",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(ids) => ids,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let mut items = Vec::new();
    for id in ids {
        if let Ok(Some(provider)) = provider_json(pool.get_ref(), id).await {
            items.push(provider);
        }
    }
    HttpResponse::Ok().json(json!({ "items": items, "adapter_kinds": KINDS }))
}

async fn upsert_provider(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    body: web::Json<UpsertProviderRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    save_provider(pool, request, body.into_inner()).await
}

async fn update_provider(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    mut body: web::Json<UpsertProviderRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    body.id = Some(path.into_inner());
    save_provider(pool, request, body.into_inner()).await
}

async fn save_provider(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    body: UpsertProviderRequest,
) -> HttpResponse {
    if !KINDS.contains(&body.provider_kind.as_str())
        || !matches!(body.environment.as_str(), "test" | "live")
    {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Invalid provider kind or environment" }));
    }
    let status = body.status.unwrap_or_else(|| "disabled".into());
    let requested_default = body.is_default.unwrap_or(false);
    if body.provider_kind == "bank_payout"
        && body.environment == "live"
        && status == "enabled"
        && body.live_enable_approved != Some(true)
    {
        return HttpResponse::BadRequest().json(json!({
            "message": "Live bank payout providers require explicit internal approval"
        }));
    }
    let config = body.config.unwrap_or_else(|| json!({}));
    let id = match body.id {
        Some(id) => {
            sqlx::query_scalar::<_, Uuid>(
                r#"
            UPDATE integration_providers
            SET name = $2, code = $3, environment = $4, status = $5, is_default = $6,
                priority = $7, timeout_ms = $8, max_retries = $9, retry_backoff_ms = $10,
                config = $11::jsonb, updated_at = NOW()
            WHERE id = $1
            RETURNING id
            "#,
            )
            .bind(id)
            .bind(&body.name)
            .bind(&body.code)
            .bind(&body.environment)
            .bind(&status)
            .bind(false)
            .bind(body.priority.unwrap_or(100))
            .bind(body.timeout_ms.unwrap_or(10000))
            .bind(body.max_retries.unwrap_or(3))
            .bind(body.retry_backoff_ms.unwrap_or(1000))
            .bind(&config)
            .fetch_one(pool.get_ref())
            .await
        }
        None => {
            sqlx::query_scalar::<_, Uuid>(
                r#"
            INSERT INTO integration_providers (
                provider_kind, name, code, environment, status, is_default, priority,
                timeout_ms, max_retries, retry_backoff_ms, config
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::jsonb)
            RETURNING id
            "#,
            )
            .bind(&body.provider_kind)
            .bind(&body.name)
            .bind(&body.code)
            .bind(&body.environment)
            .bind(&status)
            .bind(false)
            .bind(body.priority.unwrap_or(100))
            .bind(body.timeout_ms.unwrap_or(10000))
            .bind(body.max_retries.unwrap_or(3))
            .bind(body.retry_backoff_ms.unwrap_or(1000))
            .bind(&config)
            .fetch_one(pool.get_ref())
            .await
        }
    };
    match id {
        Ok(id) => {
            if requested_default {
                let mut tx = match pool.begin().await {
                    Ok(tx) => tx,
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                };
                let _ = sqlx::query(
                    "UPDATE integration_providers SET is_default = false, updated_at = NOW() WHERE provider_kind = $1 AND environment = $2",
                )
                .bind(&body.provider_kind)
                .bind(&body.environment)
                .execute(&mut *tx)
                .await;
                let _ = sqlx::query(
                    "UPDATE integration_providers SET is_default = true, updated_at = NOW() WHERE id = $1",
                )
                .bind(id)
                .execute(&mut *tx)
                .await;
                let _ = tx.commit().await;
            }
            audit(
                pool.get_ref(),
                &request,
                if body.id.is_some() {
                    "integration_provider.updated"
                } else {
                    "integration_provider.created"
                },
                Some(id),
                json!({
                    "provider_kind": body.provider_kind,
                    "environment": body.environment,
                    "status": status,
                    "code": body.code,
                }),
            )
            .await;
            match provider_json(pool.get_ref(), id).await {
                Ok(Some(provider)) => HttpResponse::Ok().json(provider),
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(error) => {
            tracing::error!("Save integration provider failed: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn set_status(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<StatusRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let id = path.into_inner();
    let row =
        sqlx::query("SELECT provider_kind, environment FROM integration_providers WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().finish();
    };
    let kind: String = row.get("provider_kind");
    let environment: String = row.get("environment");
    if kind == "bank_payout"
        && environment == "live"
        && body.enabled
        && body.live_enable_approved != Some(true)
    {
        return HttpResponse::BadRequest().json(json!({
            "message": "Live bank payout providers require explicit internal approval"
        }));
    }
    let status = if body.enabled { "enabled" } else { "disabled" };
    let _ = sqlx::query(
        "UPDATE integration_providers SET status = $2, live_enabled_approved_at = CASE WHEN $3 THEN NOW() ELSE live_enabled_approved_at END, live_enabled_approved_by = CASE WHEN $3 THEN $4 ELSE live_enabled_approved_by END, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(kind == "bank_payout" && environment == "live" && body.enabled)
    .bind(actor(&request))
    .execute(pool.get_ref())
    .await;
    audit(
        pool.get_ref(),
        &request,
        if body.enabled {
            "integration_provider.enabled"
        } else {
            "integration_provider.disabled"
        },
        Some(id),
        json!({ "status": status }),
    )
    .await;
    provider_json(pool.get_ref(), id)
        .await
        .ok()
        .flatten()
        .map_or_else(
            || HttpResponse::InternalServerError().finish(),
            |provider| HttpResponse::Ok().json(provider),
        )
}

async fn set_default(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let id = path.into_inner();
    let row =
        sqlx::query("SELECT provider_kind, environment FROM integration_providers WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().finish();
    };
    let kind: String = row.get("provider_kind");
    let environment: String = row.get("environment");
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let _ = sqlx::query("UPDATE integration_providers SET is_default = false, updated_at = NOW() WHERE provider_kind = $1 AND environment = $2")
        .bind(&kind)
        .bind(&environment)
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query(
        "UPDATE integration_providers SET is_default = true, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await;
    let _ = tx.commit().await;
    audit(
        pool.get_ref(),
        &request,
        "integration_provider.default_changed",
        Some(id),
        json!({
            "provider_kind": kind,
            "environment": environment
        }),
    )
    .await;
    provider_json(pool.get_ref(), id)
        .await
        .ok()
        .flatten()
        .map_or_else(
            || HttpResponse::InternalServerError().finish(),
            |provider| HttpResponse::Ok().json(provider),
        )
}

async fn upsert_secret(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<SecretRequest>,
) -> impl Responder {
    save_secret(
        pool,
        request,
        path.into_inner(),
        body.into_inner(),
        "integration_secret.saved",
    )
    .await
}

async fn rotate_secret(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<(Uuid, String)>,
    body: web::Json<SecretRequest>,
) -> impl Responder {
    let (provider_id, secret_name) = path.into_inner();
    let mut body = body.into_inner();
    body.secret_name = secret_name;
    save_secret(
        pool,
        request,
        provider_id,
        body,
        "integration_secret.rotated",
    )
    .await
}

async fn save_secret(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    provider_id: Uuid,
    body: SecretRequest,
    action: &str,
) -> HttpResponse {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    if body.secret_name.trim().is_empty() || body.value.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "secret_name and value are required" }));
    }
    let (encrypted_value, nonce) = match encrypt_secret(body.value.trim()) {
        Ok(value) => value,
        Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
    };
    let masked = mask_secret(body.value.trim());
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO integration_provider_secrets (
            provider_id, secret_name, encrypted_value, nonce, masked_value, rotated_by_admin_id
        )
        VALUES ($1,$2,$3,$4,$5,$6)
        ON CONFLICT (provider_id, secret_name)
        DO UPDATE SET encrypted_value = EXCLUDED.encrypted_value,
                      nonce = EXCLUDED.nonce,
                      masked_value = EXCLUDED.masked_value,
                      rotated_by_admin_id = EXCLUDED.rotated_by_admin_id,
                      rotated_at = NOW(),
                      updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(provider_id)
    .bind(body.secret_name.trim().to_lowercase())
    .bind(encrypted_value)
    .bind(nonce)
    .bind(&masked)
    .bind(actor(&request))
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                pool.get_ref(),
                &request,
                action,
                Some(provider_id),
                json!({
                    "secret_id": id,
                    "secret_name": body.secret_name,
                    "masked_value": masked
                }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "masked_value": masked }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn provider_action(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    provider_id: Uuid,
    action: &str,
    request_type: &str,
    body: Option<TestActionRequest>,
) -> HttpResponse {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let provider = sqlx::query(
        "SELECT provider_kind, environment, status, config FROM integration_providers WHERE id = $1",
    )
    .bind(provider_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(provider)) = provider else {
        return HttpResponse::NotFound().finish();
    };
    let provider_kind: String = provider.get("provider_kind");
    let environment: String = provider.get("environment");
    let status: String = provider.get("status");
    let allowed = match action {
        "test_email" => matches!(provider_kind.as_str(), "email" | "notification"),
        "test_sms" => provider_kind == "sms",
        "test_otp" => matches!(provider_kind.as_str(), "sms" | "otp"),
        "test_bank_payout" => provider_kind == "bank_payout",
        _ => true,
    };
    if !allowed {
        return HttpResponse::BadRequest().json(json!({
            "message": "Action is not supported for this provider kind"
        }));
    }
    let adapter = adapter_for_kind(&provider_kind);
    if action == "test_bank_payout" && environment != "test" {
        write_log(
            pool.get_ref(),
            Some(provider_id),
            &request,
            action,
            request_type,
            "blocked",
            0,
            Some("live_bank_test_blocked"),
            Some("Bank payout tests are allowed only in safe test mode"),
            json!({}),
            json!({}),
        )
        .await;
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Bank payout tests are allowed only in safe test mode" }));
    }
    let config: Value = provider.get("config");
    let mut success = status == "enabled";
    let mut error_code = if success {
        None
    } else {
        Some("provider_disabled")
    };
    let mut error_message = if success {
        None
    } else {
        Some("Provider is disabled".to_string())
    };
    if success && action == "test_email" {
        let recipient = match body.as_ref().and_then(|value| value.recipient.as_deref()) {
            Some(value) if !value.trim().is_empty() => value.trim(),
            _ => {
                return HttpResponse::BadRequest()
                    .json(json!({ "message": "recipient is required" }));
            }
        };
        let send_result = async {
            let smtp = smtp_config(pool.get_ref(), provider_id, &config).await?;
            let from = parse_mailbox(&format!("{} <{}>", smtp.from_name, smtp.from_email))?;
            let to = parse_mailbox(recipient)?;
            let mut builder = Message::builder()
                .from(from)
                .to(to)
                .subject("FeelPay integration test email");
            if let Some(reply_to) = smtp.reply_to.as_deref() {
                builder = builder.reply_to(parse_mailbox(reply_to)?);
            }
            let message = builder
                .body("This is a FeelPay SMTP integration test email.".to_string())
                .map_err(|_| "Failed to build test email".to_string())?;
            smtp_transport(&smtp)?
                .send(message)
                .await
                .map_err(|_| "SMTP delivery failed".to_string())?;
            Ok::<(), String>(())
        }
        .await;
        if let Err(message) = send_result {
            success = false;
            error_code = Some("smtp_delivery_failed");
            error_message = Some(message);
        }
    }
    if success && matches!(action, "test_sms" | "test_otp") {
        let phone = match body.as_ref().and_then(|value| value.phone.as_deref()) {
            Some(value) if !value.trim().is_empty() => value.trim(),
            _ => {
                return HttpResponse::BadRequest().json(json!({ "message": "phone is required" }));
            }
        };
        let message = if action == "test_otp" {
            "Your FeelPay test OTP is 000000. Do not share."
        } else {
            "FeelPay SMS provider test message."
        };
        let send_result = async {
            let sms =
                generic_sms_config(pool.get_ref(), provider_id, &environment, &config).await?;
            send_generic_sms(
                &sms,
                phone,
                message,
                if action == "test_otp" { "otp" } else { "test" },
            )
            .await
        }
        .await;
        if let Err(message) = send_result {
            success = false;
            error_code = Some("sms_delivery_failed");
            error_message = Some(message);
        }
    }
    let _ = sqlx::query(
        r#"
        UPDATE integration_providers
        SET health_status = $2,
            last_success_at = CASE WHEN $3 THEN NOW() ELSE last_success_at END,
            last_failure_at = CASE WHEN $3 THEN last_failure_at ELSE NOW() END,
            last_error_code = $4,
            last_error_message = $5,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(provider_id)
    .bind(if success { "healthy" } else { "failed" })
    .bind(success)
    .bind(error_code)
    .bind(error_message.as_deref())
    .execute(pool.get_ref())
    .await;
    let summary = json!({
        "provider_kind": provider_kind,
        "adapter_kind": adapter.as_ref().map(|item| format!("{:?}", item.kind())),
        "adapter_supports_live_mode": adapter.as_ref().map(|item| item.supports_live_mode()),
        "environment": environment,
        "recipient": body.as_ref().and_then(|value| value.recipient.as_ref()).map(|_| "redacted"),
        "phone": body.as_ref().and_then(|value| value.phone.as_deref()).map(mask_phone),
        "otp": body.as_ref().and_then(|value| value.otp.as_ref()).map(|_| "redacted"),
    });
    write_log(
        pool.get_ref(),
        Some(provider_id),
        &request,
        action,
        request_type,
        if success { "success" } else { "failed" },
        1,
        error_code,
        error_message.as_deref(),
        summary.clone(),
        json!({ "ok": success }),
    )
    .await;
    audit(
        pool.get_ref(),
        &request,
        &format!("integration_provider.{action}"),
        Some(provider_id),
        summary,
    )
    .await;
    HttpResponse::Ok().json(json!({
        "provider_id": provider_id,
        "status": if success { "success" } else { "failed" },
        "request_type": request_type,
        "safe_test_mode": environment == "test"
    }))
}

async fn test_connection(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    provider_action(
        pool,
        request,
        path.into_inner(),
        "test_connection",
        "connection",
        None,
    )
    .await
}

async fn send_test_email(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TestActionRequest>,
) -> impl Responder {
    provider_action(
        pool,
        request,
        path.into_inner(),
        "test_email",
        "email",
        Some(body.into_inner()),
    )
    .await
}

async fn send_default_test_email(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<String>,
    body: web::Json<TestActionRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let environment = path.into_inner();
    if !matches!(environment.as_str(), "test" | "live") {
        return HttpResponse::BadRequest().json(json!({ "message": "Invalid environment" }));
    }
    let provider_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM integration_providers WHERE provider_kind = 'email' AND environment = $1 AND is_default = true ORDER BY priority, name LIMIT 1",
    )
    .bind(&environment)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(provider_id)) = provider_id else {
        return HttpResponse::NotFound().json(json!({
            "message": "No default email provider configured"
        }));
    };
    provider_action(
        pool,
        request,
        provider_id,
        "test_email",
        "email",
        Some(body.into_inner()),
    )
    .await
}

async fn send_test_sms(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TestActionRequest>,
) -> impl Responder {
    provider_action(
        pool,
        request,
        path.into_inner(),
        "test_sms",
        "sms",
        Some(body.into_inner()),
    )
    .await
}

async fn send_test_otp(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TestActionRequest>,
) -> impl Responder {
    provider_action(
        pool,
        request,
        path.into_inner(),
        "test_otp",
        "otp",
        Some(body.into_inner()),
    )
    .await
}

async fn test_bank_payout(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    provider_action(
        pool,
        request,
        path.into_inner(),
        "test_bank_payout",
        "bank_payout",
        None,
    )
    .await
}

async fn list_provider_logs(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    list_logs_for(pool, Some(path.into_inner())).await
}

async fn list_logs(pool: web::Data<PgPool>, request: HttpRequest) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    list_logs_for(pool, None).await
}

async fn list_logs_for(pool: web::Data<PgPool>, provider_id: Option<Uuid>) -> HttpResponse {
    let rows = sqlx::query(
        r#"
        SELECT id, provider_id, action, request_type, status, latency_ms, error_code,
               error_message, actor_admin_id, request_summary, response_summary, created_at
        FROM integration_logs
        WHERE ($1::uuid IS NULL OR provider_id = $1)
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(provider_id)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "items": rows.into_iter().map(|row| json!({
                "id": row.get::<Uuid, _>("id"),
                "provider_id": row.get::<Option<Uuid>, _>("provider_id"),
                "action": row.get::<String, _>("action"),
                "request_type": row.get::<String, _>("request_type"),
                "status": row.get::<String, _>("status"),
                "latency_ms": row.get::<i32, _>("latency_ms"),
                "error_code": row.get::<Option<String>, _>("error_code"),
                "error_message": row.get::<Option<String>, _>("error_message"),
                "actor_admin_id": row.get::<Option<Uuid>, _>("actor_admin_id"),
                "request_summary": row.get::<Value, _>("request_summary"),
                "response_summary": row.get::<Value, _>("response_summary"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at"),
            })).collect::<Vec<_>>()
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_secrets_without_revealing_prefix() {
        assert_eq!(mask_secret("abcd"), "****");
        assert_eq!(mask_secret("secret-value"), "****alue");
    }

    #[test]
    fn bank_payout_adapter_is_not_live_enabled_by_default() {
        let adapter = adapter_for_kind("bank_payout").expect("bank adapter");
        assert!(matches!(
            adapter.kind(),
            IntegrationKind::BankPayoutProvider
        ));
        assert!(!adapter.supports_live_mode());
    }

    #[test]
    fn sms_templates_mask_phone_and_render_only_known_placeholders() {
        assert_eq!(mask_phone("+919876543210"), "******3210");
        let rendered = render_sms_value(
            &json!({"to": "{{phone}}", "token": "{{token}}"}),
            "+919876543210",
            "hello",
            "FEELPY",
            Some("transactional"),
            Some("OTP01"),
            "secret",
        )
        .expect("template");
        assert_eq!(rendered["to"], "+919876543210");
        assert_eq!(rendered["token"], "secret");
        assert!(render_sms_value(
            &json!({"bad": "{{unsafe}}"}),
            "+919876543210",
            "hello",
            "FEELPY",
            None,
            None,
            "secret",
        )
        .is_err());
    }
}

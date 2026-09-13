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
use feelpay::engine::services::bank_payout_provider::{
    map_provider_status, BankPayoutCallback, BankPayoutProviderAdapter, TestBankPayoutAdapter,
};

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

#[derive(Debug, Deserialize)]
struct UpsertNotificationTemplateRequest {
    event_type: String,
    channel: String,
    environment: String,
    subject: Option<String>,
    body: String,
    sms_body: Option<String>,
    enabled: bool,
    default_email_provider_id: Option<Uuid>,
    default_sms_provider_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct TemplatePreviewRequest {
    sample_data: Value,
}

#[derive(Debug, Deserialize)]
struct TemplateTestRequest {
    recipient: Option<String>,
    phone: Option<String>,
    sample_data: Value,
    live_test_approved: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct KycTestRequest {
    verification_type: String,
    scenario: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BankPayoutCallbackRequest {
    payout_id: Option<Uuid>,
    provider_event_id: String,
    provider_reference: Option<String>,
    provider_status: String,
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
    provider_code: String,
    base_url: String,
    method: String,
    headers_template: serde_json::Map<String, Value>,
    body_template: Value,
    sender_id: String,
    route_type: Option<String>,
    template_mapping: serde_json::Map<String, Value>,
    auth_token: String,
    account_sid: Option<String>,
    from_number: Option<String>,
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
    approved_for_live: bool,
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
const TEMPLATE_TYPES: &[&str] = &[
    "onboarding",
    "otp",
    "kyc_approved",
    "kyc_rejected",
    "payout_processed",
    "payout_failed",
    "refund_created",
    "refund_succeeded",
    "refund_failed",
    "dispute_opened",
    "dispute_deadline",
    "api_key_created",
    "api_key_revoked",
    "webhook_failure",
    "settlement_processed",
    "settlement_failed",
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

fn action_allowed(provider_kind: &str, action: &str) -> bool {
    match action {
        "test_email" => matches!(provider_kind, "email" | "notification"),
        "test_sms" => provider_kind == "sms",
        "test_otp" => matches!(provider_kind, "sms" | "otp"),
        "test_bank_payout" => provider_kind == "bank_payout",
        "test_kyc" => provider_kind == "kyc",
        _ => true,
    }
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
            .route("/providers/{id}/test-kyc", web::post().to(test_kyc))
            .route(
                "/providers/{id}/test-bank-payout",
                web::post().to(test_bank_payout),
            )
            .route(
                "/providers/{id}/bank-payout-callbacks",
                web::post().to(record_bank_payout_callback),
            )
            .route(
                "/providers/{id}/bank-payout-callbacks",
                web::get().to(list_bank_payout_callbacks),
            )
            .route(
                "/bank-payout/callbacks",
                web::get().to(list_all_bank_payout_callbacks),
            )
            .route(
                "/bank-payout-callbacks",
                web::get().to(list_all_bank_payout_callbacks),
            )
            .route("/providers/{id}/logs", web::get().to(list_provider_logs))
            .route("/logs", web::get().to(list_logs))
            .route("/templates", web::get().to(list_notification_templates))
            .route("/templates", web::post().to(create_notification_template))
            .route(
                "/templates/{id}",
                web::put().to(update_notification_template),
            )
            .route(
                "/templates/{id}/status",
                web::post().to(set_notification_template_status),
            )
            .route(
                "/templates/{id}/preview",
                web::post().to(preview_notification_template),
            )
            .route(
                "/templates/{id}/test",
                web::post().to(test_notification_template),
            )
            .route(
                "/dispatch-logs",
                web::get().to(list_notification_dispatch_logs),
            )
            .route(
                "/dispatch-logs/{id}/retry",
                web::post().to(retry_notification_dispatch),
            ),
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

fn extract_template_variables(values: &[Option<&str>]) -> Vec<String> {
    let regex = regex::Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").expect("template regex");
    let mut variables = values
        .iter()
        .flatten()
        .flat_map(|value| regex.captures_iter(value))
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<Vec<_>>();
    variables.sort();
    variables.dedup();
    variables
}

fn render_notification_text(
    template: &str,
    allowed_variables: &[String],
    sample_data: &Value,
) -> Result<String, String> {
    let variables = extract_template_variables(&[Some(template)]);
    let unknown = variables
        .iter()
        .filter(|variable| !allowed_variables.contains(variable))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(format!("Unknown variables: {}", unknown.join(", ")));
    }
    let mut rendered = template.to_string();
    for variable in allowed_variables {
        let value = sample_data
            .get(variable)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Missing variable: {variable}"))?;
        rendered = rendered.replace(&format!("{{{{{variable}}}}}"), value);
        rendered = rendered.replace(&format!("{{{{ {variable} }}}}"), value);
    }
    if rendered.contains("{{") || rendered.contains("}}") {
        return Err("Template contains unresolved variables".to_string());
    }
    Ok(rendered)
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
    let provider_code = config
        .get("provider_code")
        .and_then(Value::as_str)
        .unwrap_or("generic_http")
        .to_lowercase();
    let base_url = if provider_code == "twilio" {
        String::new()
    } else {
        let value = required_string(config, "sms_base_url")?;
        validate_sms_url(environment, &value)?;
        value
    };
    let method = if provider_code == "twilio" {
        "POST".to_string()
    } else {
        required_string(config, "sms_http_method")?.to_uppercase()
    };
    if !matches!(method.as_str(), "POST" | "PUT") {
        return Err("sms_http_method must be POST or PUT".to_string());
    }
    let headers_template = if provider_code == "twilio" {
        serde_json::Map::new()
    } else {
        config
            .get("sms_headers_template")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(|| "Missing required SMS config: sms_headers_template".to_string())?
    };
    for value in headers_template.values() {
        if !value.is_string() {
            return Err("sms_headers_template values must be strings".to_string());
        }
    }
    let body_template = if provider_code == "twilio" {
        json!({})
    } else {
        config
            .get("sms_body_template")
            .cloned()
            .ok_or_else(|| "Missing required SMS config: sms_body_template".to_string())?
    };
    if !body_template.is_object() {
        return Err("sms_body_template must be a JSON object".to_string());
    }
    Ok(GenericSmsConfig {
        provider_code: provider_code.clone(),
        base_url,
        method,
        headers_template,
        body_template,
        sender_id: if provider_code == "twilio" {
            config
                .get("sender_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        } else {
            required_string(config, "sender_id")?
        },
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
        account_sid: config
            .get("account_sid")
            .and_then(Value::as_str)
            .map(str::to_string),
        from_number: config
            .get("from_number")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

#[derive(Debug, Clone)]
struct SmsDeliveryResult {
    provider_message_id: Option<String>,
    provider_status: Option<String>,
}

async fn send_generic_sms(
    config: &GenericSmsConfig,
    phone: &str,
    message: &str,
    template_key: &str,
) -> Result<SmsDeliveryResult, String> {
    if config.provider_code == "twilio" {
        return send_twilio_sms(config, phone, message).await;
    }
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
        Ok(SmsDeliveryResult {
            provider_message_id: None,
            provider_status: Some("accepted".to_string()),
        })
    } else {
        Err(format!("SMS provider returned {}", response.status()))
    }
}

async fn send_twilio_sms(
    config: &GenericSmsConfig,
    phone: &str,
    message: &str,
) -> Result<SmsDeliveryResult, String> {
    let account_sid = config
        .account_sid
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Missing required config: account_sid".to_string())?;
    let from_number = config
        .from_number
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Missing required config: from_number".to_string())?;
    let url = format!("https://api.twilio.com/2010-04-01/Accounts/{account_sid}/Messages.json");
    let response = reqwest::Client::new()
        .post(url)
        .basic_auth(account_sid, Some(&config.auth_token))
        .form(&[("To", phone), ("From", from_number), ("Body", message)])
        .send()
        .await
        .map_err(|_| "Twilio request failed".to_string())?;
    let status = response.status();
    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| "Twilio response was not valid JSON".to_string())?;
    if !status.is_success() {
        let code = payload
            .get("code")
            .and_then(Value::as_i64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(format!("Twilio returned {status} (code {code})"));
    }
    Ok(SmsDeliveryResult {
        provider_message_id: payload
            .get("sid")
            .and_then(Value::as_str)
            .map(str::to_string),
        provider_status: payload
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
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
               last_failure_at, last_error_code, last_error_message, approved_for_live, config, created_at, updated_at
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
        approved_for_live: row.get("approved_for_live"),
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
    {
        if body.live_enable_approved != Some(true) {
            return HttpResponse::BadRequest().json(json!({
                "message": "Live bank payout providers require explicit internal approval"
            }));
        }
        return HttpResponse::BadRequest().json(json!({
            "message": "No official live bank payout adapter is configured yet"
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
                config = $11::jsonb, approved_for_live = CASE WHEN $4 = 'live' AND $5 = 'enabled' AND $12 THEN true ELSE approved_for_live END, updated_by_admin_id = $13, updated_at = NOW()
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
            .bind(body.live_enable_approved == Some(true))
            .bind(actor(&request))
            .fetch_one(pool.get_ref())
            .await
        }
        None => {
            sqlx::query_scalar::<_, Uuid>(
                r#"
            INSERT INTO integration_providers (
                provider_kind, name, code, environment, status, is_default, priority,
                timeout_ms, max_retries, retry_backoff_ms, config, approved_for_live,
                created_by_admin_id, updated_by_admin_id
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::jsonb,$12,$13,$13)
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
            .bind(body.environment == "live" && body.live_enable_approved == Some(true))
            .bind(actor(&request))
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
    {
        if body.live_enable_approved != Some(true) {
            return HttpResponse::BadRequest().json(json!({
                "message": "Live bank payout providers require explicit internal approval"
            }));
        }
        return HttpResponse::BadRequest().json(json!({
            "message": "No official live bank payout adapter is configured yet"
        }));
    }
    let status = if body.enabled { "enabled" } else { "disabled" };
    let _ = sqlx::query(
        "UPDATE integration_providers SET status = $2, approved_for_live = CASE WHEN $3 THEN true ELSE approved_for_live END, live_enabled_approved_at = CASE WHEN $3 THEN NOW() ELSE live_enabled_approved_at END, live_enabled_approved_by = CASE WHEN $3 THEN $4 ELSE live_enabled_approved_by END, updated_by_admin_id = $4, updated_at = NOW() WHERE id = $1",
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
    if kind == "bank_payout" && environment == "live" && body.enabled {
        audit(
            pool.get_ref(),
            &request,
            "integration_provider.live_approved",
            Some(id),
            json!({ "status": status }),
        )
        .await;
    }
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
    let allowed = action_allowed(&provider_kind, action);
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
    let mut response_summary = json!({ "ok": success });
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
        let delivery_result = match send_result {
            Ok(result) => Some(result),
            Err(message) => {
                success = false;
                error_code = Some("sms_delivery_failed");
                error_message = Some(message);
                None
            }
        };
        if let Some(result) = delivery_result {
            response_summary = json!({
                "ok": true,
                "provider_message_id": result.provider_message_id,
                "provider_status": result.provider_status,
            });
        }
    }
    if success && action == "test_bank_payout" {
        let result = TestBankPayoutAdapter.test_connection().await;
        success = result.error_code.is_none();
        if !success {
            error_code = Some("bank_payout_test_failed");
            error_message = Some("Bank payout test failed".to_string());
        }
        response_summary = result.summary_redacted;
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
        response_summary,
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

async fn record_bank_payout_callback(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<BankPayoutCallbackRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let provider_id = path.into_inner();
    let provider = sqlx::query(
        "SELECT environment, status FROM integration_providers WHERE id = $1 AND provider_kind = 'bank_payout'",
    )
    .bind(provider_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(provider)) = provider else {
        return HttpResponse::NotFound().finish();
    };
    if provider.get::<String, _>("status") != "enabled" {
        return HttpResponse::Conflict().json(json!({ "message": "Bank payout provider is disabled" }));
    }
    if provider.get::<String, _>("environment") != "test" {
        return HttpResponse::Conflict().json(json!({ "message": "No official live bank payout adapter is configured yet" }));
    }
    let adapter = TestBankPayoutAdapter;
    let result = adapter
        .handle_callback(BankPayoutCallback {
            provider_event_id: body.provider_event_id.clone(),
            provider_reference: body.provider_reference.clone(),
            provider_status: body.provider_status.clone(),
        })
        .await;
    let mapped_status = map_provider_status(&result.status);
    let inserted = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO bank_payout_provider_callbacks (
            provider_id, payout_id, provider_event_id, provider_reference,
            mapped_status, payload_summary_redacted
        )
        VALUES ($1,$2,$3,$4,$5,$6::jsonb)
        ON CONFLICT (provider_id, provider_event_id) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(provider_id)
    .bind(body.payout_id)
    .bind(body.provider_event_id.trim())
    .bind(body.provider_reference.as_deref())
    .bind(mapped_status)
    .bind(&result.summary_redacted)
    .fetch_optional(pool.get_ref())
    .await;
    match inserted {
        Ok(Some(callback_id)) => {
            if let Some(payout_id) = body.payout_id {
                if matches!(mapped_status, "paid" | "failed" | "reversed") {
                    let previous = sqlx::query_scalar::<_, String>(
                        "SELECT status FROM payouts_company WHERE id = $1",
                    )
                    .bind(payout_id)
                    .fetch_optional(pool.get_ref())
                    .await
                    .ok()
                    .flatten();
                    if let Some(previous) = previous {
                        let _ = sqlx::query(
                            "UPDATE payouts_company SET status = $2, provider_reference = COALESCE($3, provider_reference), provider_response_summary = $4::jsonb, processed_at = CASE WHEN $2 IN ('paid','failed','reversed') THEN NOW() ELSE processed_at END WHERE id = $1",
                        )
                        .bind(payout_id)
                        .bind(mapped_status)
                        .bind(body.provider_reference.as_deref())
                        .bind(&result.summary_redacted)
                        .execute(pool.get_ref())
                        .await;
                        let _ = sqlx::query(
                            "INSERT INTO payout_status_history (payout_id, from_status, to_status, reason, reference_kind, reference_value) VALUES ($1,$2,$3,'provider callback','provider',$4)",
                        )
                        .bind(payout_id)
                        .bind(previous)
                        .bind(mapped_status)
                        .bind(body.provider_reference.as_deref())
                        .execute(pool.get_ref())
                        .await;
                    }
                }
                if let Some(reference) = body.provider_reference.as_deref() {
                    let _ = sqlx::query(
                        "UPDATE payouts_company p SET reconciliation_item_id = ri.id, reconciliation_batch_id = ri.batch_id FROM reconciliation_items ri WHERE p.id = $1 AND ri.provider_reference = $2 AND p.reconciliation_item_id IS NULL",
                    )
                    .bind(payout_id)
                    .bind(reference)
                    .execute(pool.get_ref())
                    .await;
                }
            }
            write_log(
                pool.get_ref(),
                Some(provider_id),
                &request,
                "bank_payout_callback",
                "callback",
                "success",
                0,
                None,
                None,
                json!({
                    "event_id_present": !body.provider_event_id.trim().is_empty(),
                    "provider_reference_present": body.provider_reference.is_some(),
                    "environment": provider.get::<String, _>("environment")
                }),
                result.summary_redacted,
            )
            .await;
            audit(
                pool.get_ref(),
                &request,
                "integration_provider.bank_payout_callback",
                Some(provider_id),
                json!({ "callback_id": callback_id, "mapped_status": mapped_status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": callback_id, "status": mapped_status }))
        }
        Ok(None) => HttpResponse::Ok().json(json!({ "status": "duplicate" })),
        Err(error) => {
            tracing::error!("Record bank payout callback failed: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn list_bank_payout_callbacks(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let rows = sqlx::query(
        "SELECT id, payout_id, provider_event_id, provider_reference, mapped_status, processing_status, created_at FROM bank_payout_provider_callbacks WHERE provider_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(path.into_inner())
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    HttpResponse::Ok().json(json!({
        "items": rows.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id"),
            "payout_id": row.get::<Option<Uuid>, _>("payout_id"),
            "provider_event_id": row.get::<String, _>("provider_event_id"),
            "provider_reference": row.get::<Option<String>, _>("provider_reference"),
            "mapped_status": row.get::<String, _>("mapped_status"),
            "processing_status": row.get::<String, _>("processing_status"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
        })).collect::<Vec<_>>()
    }))
}

#[derive(Debug, Deserialize)]
pub struct BankPayoutCallbacksQuery {
    pub provider_id: Option<Uuid>,
}

async fn list_all_bank_payout_callbacks(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    query: web::Query<BankPayoutCallbacksQuery>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let rows = if let Some(provider_id) = query.provider_id {
        sqlx::query(
            "SELECT id, payout_id, provider_event_id, provider_reference, mapped_status, processing_status, created_at FROM bank_payout_provider_callbacks WHERE provider_id = $1 ORDER BY created_at DESC LIMIT 100",
        )
        .bind(provider_id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    } else {
        sqlx::query(
            "SELECT id, payout_id, provider_event_id, provider_reference, mapped_status, processing_status, created_at FROM bank_payout_provider_callbacks ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    };
    HttpResponse::Ok().json(json!({
        "items": rows.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id"),
            "payout_id": row.get::<Option<Uuid>, _>("payout_id"),
            "provider_event_id": row.get::<String, _>("provider_event_id"),
            "provider_reference": row.get::<Option<String>, _>("provider_reference"),
            "mapped_status": row.get::<String, _>("mapped_status"),
            "processing_status": row.get::<String, _>("processing_status"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
        })).collect::<Vec<_>>()
    }))
}

async fn test_kyc(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<KycTestRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let provider_id = path.into_inner();
    let provider = sqlx::query(
        "SELECT environment, status FROM integration_providers WHERE id=$1 AND provider_kind='kyc'",
    )
    .bind(provider_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(provider)) = provider else {
        return HttpResponse::NotFound().finish();
    };
    let environment: String = provider.get("environment");
    if environment != "test" {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "KYC tests require a test-mode provider" }));
    }
    if provider.get::<String, _>("status") != "enabled" {
        return HttpResponse::BadRequest().json(json!({ "message": "KYC provider is disabled" }));
    }
    if !matches!(
        body.verification_type.as_str(),
        "pan" | "gstin" | "business"
    ) {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "verification_type must be pan, gstin, or business" }));
    }
    let status = match body.scenario.as_deref().unwrap_or("safe_test") {
        "safe_test" => "verified",
        "failed" => "failed",
        "pending" => "pending",
        "manual_review" => "manual_review_required",
        _ => "provider_error",
    };
    let healthy = status != "provider_error";
    let _ = sqlx::query(
        r#"
        UPDATE integration_providers
        SET health_status = $2,
            last_success_at = CASE WHEN $3 THEN NOW() ELSE last_success_at END,
            last_failure_at = CASE WHEN $3 THEN last_failure_at ELSE NOW() END,
            last_error_code = CASE WHEN $3 THEN NULL ELSE 'kyc_test_failed' END,
            last_error_message = CASE WHEN $3 THEN NULL ELSE 'KYC test scenario failed' END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(provider_id)
    .bind(if healthy { "healthy" } else { "failed" })
    .bind(healthy)
    .execute(pool.get_ref())
    .await;
    write_log(
        pool.get_ref(),
        Some(provider_id),
        &request,
        "test_kyc",
        &body.verification_type,
        if healthy { "success" } else { "failed" },
        1,
        if healthy {
            None
        } else {
            Some("kyc_test_failed")
        },
        if healthy {
            None
        } else {
            Some("KYC test scenario failed")
        },
        json!({
            "verification_type": body.verification_type,
            "scenario": body.scenario,
            "identifier": "redacted"
        }),
        json!({
            "status": status,
            "provider_reference_present": healthy
        }),
    )
    .await;
    audit(
        pool.get_ref(),
        &request,
        "integration_provider.test_kyc",
        Some(provider_id),
        json!({
            "verification_type": body.verification_type,
            "identifier": "redacted"
        }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "status": status }))
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

async fn list_notification_templates(
    pool: web::Data<PgPool>,
    request: HttpRequest,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let rows = sqlx::query(
        r#"
        SELECT id, event_type, channel, environment, subject, body, sms_body, variables, is_active,
               default_email_provider_id, default_sms_provider_id, created_at, updated_at
        FROM notification_templates
        ORDER BY event_type, environment, channel
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    let items = rows.into_iter().map(|row| json!({
        "id": row.get::<Uuid, _>("id"),
        "event_type": row.get::<String, _>("event_type"),
        "channel": row.get::<String, _>("channel"),
        "environment": row.get::<String, _>("environment"),
        "subject": row.get::<Option<String>, _>("subject"),
        "body": row.get::<String, _>("body"),
        "sms_body": row.get::<Option<String>, _>("sms_body"),
        "allowed_variables": row.get::<Vec<String>, _>("variables"),
        "enabled": row.get::<bool, _>("is_active"),
        "default_email_provider_id": row.get::<Option<Uuid>, _>("default_email_provider_id"),
        "default_sms_provider_id": row.get::<Option<Uuid>, _>("default_sms_provider_id"),
        "created_at": row.get::<DateTime<Utc>, _>("created_at"),
        "updated_at": row.get::<DateTime<Utc>, _>("updated_at"),
    })).collect::<Vec<_>>();
    HttpResponse::Ok().json(json!({ "items": items, "template_types": TEMPLATE_TYPES }))
}

fn validate_template_request(
    body: &UpsertNotificationTemplateRequest,
) -> Result<Vec<String>, String> {
    if !TEMPLATE_TYPES.contains(&body.event_type.as_str()) {
        return Err("Unsupported template type".to_string());
    }
    if !matches!(body.channel.as_str(), "email" | "sms" | "both") {
        return Err("channel must be email, sms, or both".to_string());
    }
    if !matches!(body.environment.as_str(), "test" | "live") {
        return Err("environment must be test or live".to_string());
    }
    if matches!(body.channel.as_str(), "email" | "both")
        && body.subject.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err("subject is required for email templates".to_string());
    }
    if matches!(body.channel.as_str(), "sms" | "both")
        && body.sms_body.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err("sms_body is required for sms templates".to_string());
    }
    Ok(extract_template_variables(&[
        body.subject.as_deref(),
        Some(body.body.as_str()),
        body.sms_body.as_deref(),
    ]))
}

async fn create_notification_template(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    body: web::Json<UpsertNotificationTemplateRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let variables = match validate_template_request(&body) {
        Ok(value) => value,
        Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
    };
    let created = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO notification_templates (
            event_type, channel, environment, subject, body, sms_body, variables, is_active,
            default_email_provider_id, default_sms_provider_id
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING id
        "#,
    )
    .bind(&body.event_type)
    .bind(&body.channel)
    .bind(&body.environment)
    .bind(&body.subject)
    .bind(&body.body)
    .bind(&body.sms_body)
    .bind(&variables)
    .bind(body.enabled)
    .bind(body.default_email_provider_id)
    .bind(body.default_sms_provider_id)
    .fetch_one(pool.get_ref())
    .await;
    match created {
        Ok(id) => {
            audit(
                pool.get_ref(),
                &request,
                "notification_template.created",
                Some(id),
                json!({
                    "event_type": body.event_type,
                    "channel": body.channel,
                    "environment": body.environment,
                    "allowed_variables": variables,
                }),
            )
            .await;
            HttpResponse::Created().json(json!({ "id": id }))
        }
        Err(error) => HttpResponse::BadRequest().json(json!({ "message": error.to_string() })),
    }
}

async fn update_notification_template(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpsertNotificationTemplateRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let variables = match validate_template_request(&body) {
        Ok(value) => value,
        Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
    };
    let id = path.into_inner();
    let result = sqlx::query(
        r#"
        UPDATE notification_templates
        SET event_type=$2, channel=$3, environment=$4, subject=$5, body=$6, sms_body=$7,
            variables=$8, is_active=$9, default_email_provider_id=$10,
            default_sms_provider_id=$11, updated_at=NOW()
        WHERE id=$1
        "#,
    )
    .bind(id)
    .bind(&body.event_type)
    .bind(&body.channel)
    .bind(&body.environment)
    .bind(&body.subject)
    .bind(&body.body)
    .bind(&body.sms_body)
    .bind(&variables)
    .bind(body.enabled)
    .bind(body.default_email_provider_id)
    .bind(body.default_sms_provider_id)
    .execute(pool.get_ref())
    .await;
    match result {
        Ok(done) if done.rows_affected() > 0 => {
            audit(
                pool.get_ref(),
                &request,
                "notification_template.updated",
                Some(id),
                json!({
                    "event_type": body.event_type,
                    "channel": body.channel,
                    "environment": body.environment,
                    "allowed_variables": variables,
                }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(_) => HttpResponse::NotFound().finish(),
        Err(error) => HttpResponse::BadRequest().json(json!({ "message": error.to_string() })),
    }
}

async fn set_notification_template_status(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<StatusRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let id = path.into_inner();
    let result =
        sqlx::query("UPDATE notification_templates SET is_active=$2, updated_at=NOW() WHERE id=$1")
            .bind(id)
            .bind(body.enabled)
            .execute(pool.get_ref())
            .await;
    match result {
        Ok(done) if done.rows_affected() > 0 => {
            audit(
                pool.get_ref(),
                &request,
                if body.enabled {
                    "notification_template.enabled"
                } else {
                    "notification_template.disabled"
                },
                Some(id),
                json!({ "enabled": body.enabled }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "enabled": body.enabled }))
        }
        Ok(_) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn notification_template_row(
    pool: &PgPool,
    id: Uuid,
) -> Result<
    Option<(
        String,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        Vec<String>,
        bool,
        Option<Uuid>,
        Option<Uuid>,
    )>,
    sqlx::Error,
> {
    sqlx::query_as(
        r#"
        SELECT event_type, channel, environment, subject, body, sms_body, variables, is_active,
               default_email_provider_id, default_sms_provider_id
        FROM notification_templates
        WHERE id=$1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

async fn preview_notification_template(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TemplatePreviewRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let Some((_, _, _, subject, template_body, sms_body, variables, _, _, _)) =
        notification_template_row(pool.get_ref(), path.into_inner())
            .await
            .ok()
            .flatten()
    else {
        return HttpResponse::NotFound().finish();
    };
    let subject = subject
        .as_deref()
        .map(|value| render_notification_text(value, &variables, &body.sample_data))
        .transpose();
    let body_text = render_notification_text(&template_body, &variables, &body.sample_data);
    let sms = sms_body
        .as_deref()
        .map(|value| render_notification_text(value, &variables, &body.sample_data))
        .transpose();
    match (subject, body_text, sms) {
        (Ok(subject), Ok(body_text), Ok(sms)) => HttpResponse::Ok()
            .json(json!({ "subject": subject, "body": body_text, "sms_body": sms })),
        (Err(message), _, _) | (_, Err(message), _) | (_, _, Err(message)) => {
            HttpResponse::BadRequest().json(json!({ "message": message }))
        }
    }
}

async fn test_notification_template(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TemplateTestRequest>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let id = path.into_inner();
    let Some((
        event_type,
        channel,
        environment,
        subject,
        template_body,
        sms_body,
        variables,
        enabled,
        email_provider_id,
        sms_provider_id,
    )) = notification_template_row(pool.get_ref(), id)
        .await
        .ok()
        .flatten()
    else {
        return HttpResponse::NotFound().finish();
    };
    if !enabled {
        return HttpResponse::BadRequest().json(json!({ "message": "Template is disabled" }));
    }
    if environment == "live" && body.live_test_approved != Some(true) {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Live template tests require explicit internal approval" }));
    }
    let rendered_subject = subject
        .as_deref()
        .map(|value| render_notification_text(value, &variables, &body.sample_data))
        .transpose();
    let rendered_body = render_notification_text(&template_body, &variables, &body.sample_data);
    let rendered_sms = sms_body
        .as_deref()
        .map(|value| render_notification_text(value, &variables, &body.sample_data))
        .transpose();
    let (Ok(rendered_subject), Ok(rendered_body), Ok(rendered_sms)) =
        (rendered_subject, rendered_body, rendered_sms)
    else {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Template variables are incomplete or invalid" }));
    };
    let mut sent = Vec::new();
    if matches!(channel.as_str(), "email" | "both") {
        let Some(provider_id) = email_provider_id else {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Missing default email provider" }));
        };
        let Some(recipient) = body.recipient.as_deref() else {
            return HttpResponse::BadRequest().json(json!({ "message": "recipient is required" }));
        };
        let provider = sqlx::query(
            "SELECT environment, status, config FROM integration_providers WHERE id=$1",
        )
        .bind(provider_id)
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten();
        let Some(provider) = provider else {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Email provider not found" }));
        };
        let provider_environment: String = provider.get("environment");
        if provider_environment != "test"
            && !(environment == "live" && body.live_test_approved == Some(true))
        {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Template tests require a test-mode email provider" }));
        }
        if provider.get::<String, _>("status") != "enabled" {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Email provider is disabled" }));
        }
        let config: Value = provider.get("config");
        let send = async {
            let smtp = smtp_config(pool.get_ref(), provider_id, &config).await?;
            let from = parse_mailbox(&format!("{} <{}>", smtp.from_name, smtp.from_email))?;
            let to = parse_mailbox(recipient)?;
            let message = Message::builder()
                .from(from)
                .to(to)
                .subject(rendered_subject.clone().unwrap_or_default())
                .body(rendered_body.clone())
                .map_err(|_| "Failed to build test email".to_string())?;
            smtp_transport(&smtp)?
                .send(message)
                .await
                .map_err(|_| "SMTP delivery failed".to_string())
        }
        .await;
        if let Err(message) = send {
            return HttpResponse::BadRequest().json(json!({ "message": message }));
        }
        sent.push("email");
    }
    if matches!(channel.as_str(), "sms" | "both") {
        let Some(provider_id) = sms_provider_id else {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Missing default SMS provider" }));
        };
        let Some(phone) = body.phone.as_deref() else {
            return HttpResponse::BadRequest().json(json!({ "message": "phone is required" }));
        };
        let provider = sqlx::query(
            "SELECT environment, status, config FROM integration_providers WHERE id=$1",
        )
        .bind(provider_id)
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten();
        let Some(provider) = provider else {
            return HttpResponse::BadRequest().json(json!({ "message": "SMS provider not found" }));
        };
        let provider_environment: String = provider.get("environment");
        if provider_environment != "test"
            && !(environment == "live" && body.live_test_approved == Some(true))
        {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "Template tests require a test-mode SMS provider" }));
        }
        if provider.get::<String, _>("status") != "enabled" {
            return HttpResponse::BadRequest()
                .json(json!({ "message": "SMS provider is disabled" }));
        }
        let config: Value = provider.get("config");
        let sms = match generic_sms_config(
            pool.get_ref(),
            provider_id,
            &provider.get::<String, _>("environment"),
            &config,
        )
        .await
        {
            Ok(value) => value,
            Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
        };
        if let Err(message) = send_generic_sms(
            &sms,
            phone,
            rendered_sms.as_deref().unwrap_or(&rendered_body),
            &event_type,
        )
        .await
        {
            return HttpResponse::BadRequest().json(json!({ "message": message }));
        }
        sent.push("sms");
    }
    audit(
        pool.get_ref(),
        &request,
        "notification_template.tested",
        Some(id),
        json!({
            "event_type": event_type,
            "channels": sent,
            "recipient": body.recipient.as_ref().map(|_| "redacted"),
            "phone": body.phone.as_deref().map(mask_phone),
        }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "status": "success", "channels": sent }))
}

async fn list_notification_dispatch_logs(
    pool: web::Data<PgPool>,
    request: HttpRequest,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let rows = sqlx::query(
        r#"
        SELECT id, merchant_id, event_type, channel, template_id, provider_id, recipient_masked,
               status, error_code, error_message, latency_ms, retry_count, max_retries, retryable,
               next_retry_at, sent_at, created_at
        FROM notification_dispatch_logs
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    HttpResponse::Ok().json(json!({
        "items": rows.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id"),
            "merchant_id": row.get::<Option<Uuid>, _>("merchant_id"),
            "event_type": row.get::<String, _>("event_type"),
            "channel": row.get::<String, _>("channel"),
            "template_id": row.get::<Option<Uuid>, _>("template_id"),
            "provider_id": row.get::<Option<Uuid>, _>("provider_id"),
            "recipient_masked": row.get::<String, _>("recipient_masked"),
            "status": row.get::<String, _>("status"),
            "error_code": row.get::<Option<String>, _>("error_code"),
            "error_message": row.get::<Option<String>, _>("error_message"),
            "latency_ms": row.get::<i32, _>("latency_ms"),
            "retry_count": row.get::<i32, _>("retry_count"),
            "max_retries": row.get::<i32, _>("max_retries"),
            "retryable": row.get::<bool, _>("retryable"),
            "next_retry_at": row.get::<Option<DateTime<Utc>>, _>("next_retry_at"),
            "sent_at": row.get::<Option<DateTime<Utc>>, _>("sent_at"),
            "created_at": row.get::<DateTime<Utc>, _>("created_at"),
        })).collect::<Vec<_>>()
    }))
}

async fn retry_notification_dispatch(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = ensure_internal_admin(pool.get_ref(), &request).await {
        return response;
    }
    let id = path.into_inner();
    let row = sqlx::query(
        r#"
        SELECT event_type
        FROM notification_dispatch_logs
        WHERE id=$1 AND status='failed' AND retryable=true
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Retryable delivery not found" }));
    };
    if row.get::<String, _>("event_type") == "otp" {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "OTP deliveries cannot be retried" }));
    }
    let updated =
        sqlx::query("UPDATE notification_dispatch_logs SET next_retry_at = NOW() WHERE id=$1")
            .bind(id)
            .execute(pool.get_ref())
            .await;
    match updated {
        Ok(_) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
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

    #[test]
    fn sms_provider_supports_sms_and_otp_tests() {
        assert!(action_allowed("sms", "test_sms"));
        assert!(action_allowed("sms", "test_otp"));
        assert!(action_allowed("otp", "test_otp"));
        assert!(!action_allowed("email", "test_sms"));
    }

    #[test]
    fn notification_renderer_rejects_missing_and_unknown_variables() {
        let allowed = vec!["customer_name".to_string()];
        assert_eq!(
            render_notification_text(
                "Hi {{customer_name}}",
                &allowed,
                &json!({ "customer_name": "Asha" }),
            )
            .expect("render"),
            "Hi Asha"
        );
        assert!(render_notification_text("Hi {{amount}}", &allowed, &json!({})).is_err());
        assert!(render_notification_text("Hi {{customer_name}}", &allowed, &json!({})).is_err());
    }
}

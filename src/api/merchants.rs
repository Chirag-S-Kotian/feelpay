use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::engine::services::auth::MfaService;
use crate::engine::services::notification_dispatcher::{dispatch_event, DispatchRequest};
use crate::error::FeelPayError;
use crate::models::merchant::{CreateMerchantRequest, Merchant, MerchantLoginRequest};
use crate::state::AppState;
use crate::vault::crypto::VaultCrypto;
use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::Row;

const MERCHANT_SESSION_SECONDS: i64 = 24 * 60 * 60;

fn merchant_session_expires_at() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::seconds(MERCHANT_SESSION_SECONDS)
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub merchant: MerchantResponse,
}

#[derive(Serialize, ToSchema)]
pub struct MerchantResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub business_type: String,
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateMerchantRequestInput {
    pub email: String,
    pub password: String,
    pub business_name: String,
    pub business_type: String,
    pub phone: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/merchants")
            .route("", web::post().to(create_merchant))
            .route("/login", web::post().to(login_merchant))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/mfa/verify", web::post().to(verify_mfa_merchant))
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route(
                "/profile/avatar-upload",
                web::post().to(upload_profile_avatar),
            )
            .route("/keys", web::get().to(get_keys))
            .route("/keys/roll", web::post().to(roll_keys))
            .route("/settings/security", web::get().to(get_security_settings))
            .route("/dashboard-signals", web::get().to(get_dashboard_signals))
            .route(
                "/account-settings-hub",
                web::get().to(get_account_settings_hub),
            )
            .route("/checkout-settings", web::get().to(get_checkout_settings))
            .route(
                "/checkout-settings",
                web::put().to(update_checkout_settings),
            )
            .route(
                "/checkout-settings/audit",
                web::get().to(get_checkout_settings_audit),
            )
            .route("/audit-logs", web::get().to(list_merchant_audit_logs))
            .route("/webhook-secret/roll", web::post().to(roll_webhook_secret))
            .route("/{merchant_id}", web::get().to(get_merchant))
            // Team management
            .route("/team", web::get().to(list_team_members))
            .route("/team/invite", web::post().to(invite_team_member))
            .route("/team/{user_id}", web::delete().to(remove_team_member))
            .route(
                "/team/{user_id}/role",
                web::put().to(update_team_member_role),
            )
            .route("/payment-methods", web::get().to(list_payment_methods))
            .route(
                "/payment-methods/{method_type}",
                web::put().to(update_payment_method),
            )
            .route("/request-activation", web::post().to(request_activation))
            .route(
                "/{merchant_id}/kyc-status",
                web::put().to(update_kyc_status),
            ),
    );
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub name: Option<String>,
    pub business_name: Option<String>,
    pub business_type: Option<String>,
    pub webhook_url: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
    pub confirm_password: Option<String>,
    pub settlement_account: Option<serde_json::Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCheckoutSettingsRequest {
    pub styling: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
    pub payment_configuration: Option<serde_json::Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct AvatarUploadRequest {
    pub filename: String,
    pub content_type: String,
    pub data_base64: String,
}

#[derive(Serialize, ToSchema)]
pub struct AccountSettingsHubResponse {
    pub sections: Vec<AccountSettingsHubSection>,
}

#[derive(Serialize, ToSchema)]
pub struct AccountSettingsHubSection {
    pub key: String,
    pub title: String,
    pub icon: String,
    pub links: Vec<AccountSettingsHubLink>,
}

#[derive(Serialize, ToSchema)]
pub struct AccountSettingsHubLink {
    pub key: String,
    pub label: String,
    pub route: String,
    pub enabled: bool,
}

#[utoipa::path(
    post,
    path = "/v1/merchants",
    request_body = CreateMerchantRequest,
    responses(
        (status = 201, description = "Merchant created successfully"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Conflict - email already registered")
    )
)]
pub async fn create_merchant(
    state: web::Data<AppState>,
    body: web::Json<CreateMerchantRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let id = Uuid::new_v4();
    let api_key = format!("fp_test_{}", Uuid::new_v4().simple());
    let api_secret = format!("fp_secret_{}", Uuid::new_v4().simple());
    let webhook_secret = format!("whsec_{}", Uuid::new_v4().simple());
    let webhook_secret_hint = format!(
        "{}...{}",
        &webhook_secret[..6],
        &webhook_secret[webhook_secret.len() - 4..]
    );
    let (webhook_secret_ciphertext, webhook_secret_nonce) =
        encrypt_webhook_secret(state.get_ref(), webhook_secret.clone())?;

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    let secret_salt = SaltString::generate(&mut OsRng);
    let api_secret_hash = argon2
        .hash_password(api_secret.as_bytes(), &secret_salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    // 1. Insert Merchant
    sqlx::query(
        r#"INSERT INTO merchants (
               id, tenant_id, name, email, password_hash, api_key, api_secret_hash, webhook_url,
               webhook_secret, webhook_secret_ciphertext, webhook_secret_nonce, webhook_secret_hint,
               business_name, business_type
           )
           VALUES ($1,$2,$3,$4,$5,NULL,$6,$7,NULL,$8,$9,$10,$11,$12)"#,
    )
    .bind(id)
    .bind(body.tenant_id)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&api_secret_hash)
    .bind(&body.webhook_url)
    .bind(&webhook_secret_ciphertext)
    .bind(&webhook_secret_nonce)
    .bind(&webhook_secret_hint)
    .bind(&body.business_name)
    .bind(&body.business_type)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            FeelPayError::Conflict("Email already registered".into())
        } else {
            FeelPayError::Database(e.to_string())
        }
    })?;

    // 2. Insert User
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO users (id, tenant_id, merchant_id, email, password_hash, full_name, status) 
           VALUES ($1, $2, $3, $4, $5, $6, 'active')"#,
    )
    .bind(user_id)
    .bind(body.tenant_id)
    .bind(id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.name)
    .execute(&mut *tx)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // 3. Assign Role (merchant_owner)
    let role_row = sqlx::query("SELECT id FROM roles WHERE name = 'merchant_owner'")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| FeelPayError::Internal("Default role 'merchant_owner' not found".into()))?;

    let role_id: Uuid = role_row.get(0);

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    store_active_api_key(&mut *tx, id, "test", &api_key).await?;

    tx.commit()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let allow_secret_reveal =
        std::env::var("ALLOW_SECRET_REVEAL").unwrap_or_else(|_| "false".into()) == "true";

    let mut resp = serde_json::json!({
        "id": id, "name": body.name, "email": body.email,
        "api_key": api_key,
        "message": "Merchant created. Use key rotation or support flow to set secrets."
    });

    if allow_secret_reveal {
        resp["api_secret"] = serde_json::json!(api_secret);
        resp["webhook_secret"] = serde_json::json!(webhook_secret);
        resp["message"] = serde_json::json!("Store secrets safely. They may not be shown again.");
    }

    Ok(HttpResponse::Created().json(resp))
}

pub async fn get_merchant(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let id = path.into_inner();
    let row = sqlx::query(
        "SELECT id, tenant_id, name, email, api_key, business_name, status FROM merchants WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some(row) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": row.get::<Uuid, _>(0),
            "name": row.get::<String, _>(2),
            "email": row.get::<String, _>(3),
            "api_key": row.get::<Option<String>, _>(4)
                .map(|key| masked_key(&key)["display"].clone()),
            "business_name": row.get::<Option<String>, _>(5),
            "status": row.get::<String, _>(6),
        }))),
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

fn tenant_id_from_req(req: &HttpRequest) -> Option<Uuid> {
    req.extensions()
        .get::<crate::middleware::auth::TenantId>()
        .map(|tenant| tenant.0)
}

fn actor_id_from_req(req: &HttpRequest, fallback_merchant_id: Uuid) -> String {
    req.extensions()
        .get::<crate::middleware::auth::UserId>()
        .map(|user| user.0.to_string())
        .unwrap_or_else(|| fallback_merchant_id.to_string())
}

fn request_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info()
        .realip_remote_addr()
        .map(|value| value.to_string())
}

fn request_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

fn request_environment(req: &HttpRequest) -> &'static str {
    match req
        .headers()
        .get("x-feelpay-environment")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("live") => "live",
        _ => "test",
    }
}

fn masked_key(key: &str) -> serde_json::Value {
    let prefix_len = key
        .find('_')
        .map(|idx| (idx + 1).min(key.len()))
        .unwrap_or(7);
    let prefix = &key[..prefix_len.min(key.len())];
    let last4 = if key.len() >= 4 {
        &key[key.len() - 4..]
    } else {
        key
    };
    serde_json::json!({
        "display": format!("{prefix}...{last4}"),
        "prefix": prefix,
        "last4": last4,
    })
}

fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

async fn store_active_api_key<'e, E>(
    executor: E,
    merchant_id: Uuid,
    environment: &str,
    api_key: &str,
) -> Result<(), FeelPayError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let key = masked_key(api_key);
    sqlx::query(
        r#"
        INSERT INTO developer_api_keys (
            merchant_id, name, environment, key_prefix, encrypted_key_hash, masked_key, scopes, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, '["payments:write","payments:read","webhooks:write"]'::jsonb, 'active')
        "#,
    )
    .bind(merchant_id)
    .bind(format!("merchant-{environment}-default"))
    .bind(environment)
    .bind(key["prefix"].as_str().unwrap_or_default())
    .bind(hash_api_key(api_key))
    .bind(key["display"].as_str().unwrap_or_default())
    .execute(executor)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    Ok(())
}

fn encrypt_webhook_secret(
    state: &AppState,
    secret: String,
) -> Result<(String, String), FeelPayError> {
    VaultCrypto::new(
        &state.config.vault_encryption_key,
        state.config.vault_key_id.clone(),
    )?
    .encrypt(secret)
}

fn normalize_phone(raw: &str) -> Result<String, FeelPayError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(FeelPayError::BadRequest("Phone is required".into()));
    }
    let mut normalized = String::new();
    for (idx, ch) in trimmed.chars().enumerate() {
        if ch == '+' && idx == 0 {
            normalized.push(ch);
        } else if ch.is_ascii_digit() {
            normalized.push(ch);
        } else if ch == ' ' || ch == '-' || ch == '(' || ch == ')' {
            continue;
        } else {
            return Err(FeelPayError::BadRequest(
                "Phone contains invalid characters".into(),
            ));
        }
    }
    let digit_count = normalized.chars().filter(|c| c.is_ascii_digit()).count();
    if !(8..=15).contains(&digit_count) {
        return Err(FeelPayError::BadRequest(
            "Phone must contain 8 to 15 digits".into(),
        ));
    }
    Ok(normalized)
}

fn validate_webhook_url(raw: &str, env: &str) -> Result<(), FeelPayError> {
    let url = Url::parse(raw.trim())
        .map_err(|_| FeelPayError::BadRequest("Webhook URL must be a valid URL".into()))?;
    let host = url.host_str().unwrap_or_default();
    let localhost =
        matches!(host, "localhost" | "127.0.0.1" | "::1") || host.ends_with(".localhost");
    if env == "test" && localhost && matches!(url.scheme(), "http" | "https") {
        return Ok(());
    }
    if url.scheme() != "https" {
        return Err(FeelPayError::BadRequest(
            "Webhook URL must be HTTPS in live mode; localhost is allowed only in test mode".into(),
        ));
    }
    Ok(())
}

fn validate_business_type(raw: &str) -> Result<(), FeelPayError> {
    match raw.trim() {
        "individual" | "sole_proprietorship" | "partnership" | "corporation" | "llc" => Ok(()),
        _ => Err(FeelPayError::BadRequest("Invalid business type".into())),
    }
}

async fn write_audit_log(
    pool: &sqlx::PgPool,
    merchant_id: Uuid,
    action: &str,
    new_state: serde_json::Value,
) {
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs (entity_type, entity_id, action, new_state, actor_type, actor_id)
        VALUES ('merchant', $1, $2, $3::jsonb, 'merchant_user', $1)
        "#,
    )
    .bind(merchant_id)
    .bind(action)
    .bind(new_state)
    .execute(pool)
    .await;
}

async fn write_merchant_audit_log(
    pool: &sqlx::PgPool,
    req: &HttpRequest,
    merchant_id: Uuid,
    action: &str,
    target: &str,
    metadata: serde_json::Value,
) {
    let tenant_id = tenant_id_from_req(req);
    let actor_id = actor_id_from_req(req, merchant_id);
    let ip_address = request_ip(req);
    let user_agent = request_user_agent(req);

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs (
            tenant_id, merchant_id, entity_type, entity_id, action, target,
            metadata, new_state, actor_type, actor_id, ip_address, user_agent
        )
        VALUES ($1, $2, 'merchant', $2, $3, $4, $5::jsonb, $5::jsonb, 'merchant_user', $6, $7, $8)
        "#,
    )
    .bind(tenant_id)
    .bind(merchant_id)
    .bind(action)
    .bind(target)
    .bind(metadata)
    .bind(actor_id)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await;
}

const MERCHANT_TEAM_ROLES: &[&str] = &[
    "merchant_owner",
    "merchant_admin",
    "merchant_developer",
    "merchant_finance",
    "merchant_support",
    "merchant_read_only",
    // Backward-compatible legacy roles.
    "merchant_staff",
    "merchant_viewer",
];

fn ensure_valid_team_role(role: &str) -> Result<(), FeelPayError> {
    if MERCHANT_TEAM_ROLES.contains(&role) {
        Ok(())
    } else {
        Err(FeelPayError::BadRequest(format!(
            "Invalid role. Must be one of: {}",
            MERCHANT_TEAM_ROLES.join(", ")
        )))
    }
}

fn require_team_admin(req: &HttpRequest) -> Result<(), FeelPayError> {
    let extensions = req.extensions();
    let role = extensions
        .get::<crate::middleware::auth::UserRole>()
        .map(|role| role.0.as_str())
        .unwrap_or("merchant_owner");
    match role {
        "merchant_owner" | "merchant_admin" => Ok(()),
        _ => Err(FeelPayError::Forbidden(
            "Only owner or admin can manage merchant team access".into(),
        )),
    }
}

async fn merchant_session_context(
    pool: &sqlx::PgPool,
    merchant_id: Uuid,
) -> Result<(Uuid, Option<Uuid>, String), FeelPayError> {
    let row = sqlx::query(
        r#"
        SELECT u.id, u.tenant_id, COALESCE(r.name, 'merchant_owner') AS role
        FROM users u
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.merchant_id = $1 AND u.status = 'active'
        ORDER BY
            CASE COALESCE(r.name, 'merchant_owner')
                WHEN 'merchant_owner' THEN 1
                WHEN 'merchant_admin' THEN 2
                ELSE 3
            END,
            u.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(match row {
        Some(row) => (
            row.get::<Uuid, _>("id"),
            row.get::<Option<Uuid>, _>("tenant_id"),
            row.get::<String, _>("role"),
        ),
        None => (merchant_id, None, "merchant_owner".to_string()),
    })
}

async fn ensure_team_roles_exist(pool: &sqlx::PgPool) -> Result<(), FeelPayError> {
    let role_specs = [
        ("merchant_owner", "Owner access to all merchant features"),
        (
            "merchant_admin",
            "Admin access to merchant operations and settings",
        ),
        (
            "merchant_developer",
            "Developer access to API keys, webhooks, logs, and docs",
        ),
        (
            "merchant_finance",
            "Finance access to transactions, settlements, refunds, and exports",
        ),
        (
            "merchant_support",
            "Support access to customers, transactions, and disputes",
        ),
        (
            "merchant_read_only",
            "Read-only access to merchant dashboard data",
        ),
        ("merchant_staff", "Legacy operational merchant role"),
        ("merchant_viewer", "Legacy read-only merchant role"),
    ];

    for (name, description) in role_specs {
        sqlx::query(
            "INSERT INTO roles (name, description, is_system) VALUES ($1, $2, true) ON CONFLICT DO NOTHING",
        )
        .bind(name)
        .bind(description)
        .execute(pool)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;
    }

    Ok(())
}

async fn has_merchant_column(pool: &sqlx::PgPool, column_name: &str) -> Result<bool, FeelPayError> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'merchants'
              AND column_name = $1
        )
        "#,
    )
    .bind(column_name)
    .fetch_one(pool)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(exists)
}

#[utoipa::path(
    get,
    path = "/v1/merchants/profile",
    responses(
        (status = 200, description = "Returns merchant profile", body = MerchantResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Merchant not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row = sqlx::query(
        r#"
        SELECT id, tenant_id, name, email, api_key, business_name, webhook_url, status, onboarding_status, daily_limit, monthly_limit
             , phone
             , metadata
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": row.get::<Uuid, _>(0),
        "name": row.get::<String, _>(2),
        "email": row.get::<String, _>(3),
        "api_key": row.get::<Option<String>, _>(4)
            .map(|key| masked_key(&key)["display"].clone()),
        "business_name": row.get::<Option<String>, _>(5),
        "webhook_url": row.get::<Option<String>, _>(6),
        "status": row.get::<String, _>(7),
        "onboarding_status": row.get::<String, _>(8),
        "daily_limit": row.get::<i64, _>(9),
        "monthly_limit": row.get::<i64, _>(10),
        "phone": row.get::<Option<String>, _>(11),
        "avatar_url": row
            .get::<Option<serde_json::Value>, _>(12)
            .and_then(|m| m.get("avatar_url").cloned())
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    })))
}

#[utoipa::path(
    put,
    path = "/v1/merchants/profile",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = MerchantResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn update_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UpdateSettingsRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let env = request_environment(&req);

    let name = body
        .name
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let business_name = body
        .business_name
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if body.name.is_some() && name.is_none() {
        return Err(FeelPayError::BadRequest("Name is required".into()));
    }
    if body.business_name.is_some() && business_name.is_none() {
        return Err(FeelPayError::BadRequest("Business name is required".into()));
    }

    if let Some(url) = body.webhook_url.as_ref() {
        if !url.trim().is_empty() {
            validate_webhook_url(url, env)?;
        }
    }

    let phone = body
        .phone
        .as_ref()
        .map(|p| normalize_phone(p))
        .transpose()?;

    if let Some(value) = body.business_type.as_ref() {
        validate_business_type(value)?;
    }

    if let Some(url) = body.avatar_url.as_ref() {
        Url::parse(url.trim())
            .map_err(|_| FeelPayError::BadRequest("Avatar URL must be a valid URL".into()))?;
    }

    if body.current_password.is_some() || body.new_password.is_some() {
        let current = body
            .current_password
            .as_deref()
            .ok_or_else(|| FeelPayError::BadRequest("Current password required".into()))?;
        let new_password = body
            .new_password
            .as_deref()
            .ok_or_else(|| FeelPayError::BadRequest("New password required".into()))?;
        if new_password.len() < 8 {
            return Err(FeelPayError::BadRequest(
                "New password must be at least 8 characters".into(),
            ));
        }
        if let Some(confirm) = body.confirm_password.as_deref() {
            if confirm != new_password {
                return Err(FeelPayError::BadRequest(
                    "Confirm password does not match".into(),
                ));
            }
        }

        let row = sqlx::query("SELECT password_hash FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let Some(row) = row else {
            return Err(FeelPayError::NotFound("Merchant not found".into()));
        };

        let stored_hash: String = row.get(0);
        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| FeelPayError::Unauthorized("Stored password hash invalid".into()))?;

        Argon2::default()
            .verify_password(current.as_bytes(), &parsed_hash)
            .map_err(|_| FeelPayError::Unauthorized("Current password incorrect".into()))?;

        let salt = SaltString::generate(&mut OsRng);
        let new_hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
            .to_string();

        sqlx::query("UPDATE merchants SET password_hash = $2, updated_at = NOW() WHERE id = $1")
            .bind(merchant_id)
            .bind(new_hash)
            .execute(&state.db)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        write_audit_log(
            &state.db,
            merchant_id,
            "merchant_password_updated",
            serde_json::json!({"environment": env}),
        )
        .await;
    }

    sqlx::query(
        r#"
        UPDATE merchants
        SET
            name = COALESCE($2, name),
            business_name = COALESCE($3, business_name),
            webhook_url = COALESCE($4, webhook_url),
            settlement_account = COALESCE($5, settlement_account),
            business_type = COALESCE($6, business_type),
            phone = COALESCE($7, phone),
            metadata = CASE
                WHEN $8::text IS NULL THEN metadata
                ELSE jsonb_set(COALESCE(metadata, '{}'::jsonb), '{avatar_url}', to_jsonb($8::text), true)
            END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&name)
    .bind(&business_name)
    .bind(&body.webhook_url)
    .bind(&body.settlement_account)
    .bind(&body.business_type)
    .bind(&phone)
    .bind(&body.avatar_url)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    write_audit_log(
        &state.db,
        merchant_id,
        "merchant_profile_updated",
        serde_json::json!({
            "updated_fields": {
                "name": body.name.is_some(),
                "business_name": body.business_name.is_some(),
                "business_type": body.business_type.is_some(),
                "webhook_url": body.webhook_url.is_some(),
                "phone": body.phone.is_some(),
                "avatar_url": body.avatar_url.is_some()
            },
            "environment": env
        }),
    )
    .await;

    let merchant_email =
        sqlx::query_scalar::<_, Option<String>>("SELECT email FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .flatten();
    let mut revoked_variables = std::collections::BTreeMap::new();
    revoked_variables.insert("environment".to_string(), env.to_string());
    let _ = dispatch_event(
        &state.db,
        DispatchRequest {
            merchant_id: Some(merchant_id),
            event_type: "api_key_revoked".to_string(),
            environment: env.to_string(),
            recipient_email: merchant_email.clone(),
            recipient_phone: None,
            variables: revoked_variables,
        },
    )
    .await;
    let mut created_variables = std::collections::BTreeMap::new();
    created_variables.insert("environment".to_string(), env.to_string());
    let _ = dispatch_event(
        &state.db,
        DispatchRequest {
            merchant_id: Some(merchant_id),
            event_type: "api_key_created".to_string(),
            environment: env.to_string(),
            recipient_email: merchant_email,
            recipient_phone: None,
            variables: created_variables,
        },
    )
    .await;

    get_profile(req, state).await
}

pub async fn upload_profile_avatar(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AvatarUploadRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let content_type = body.content_type.trim().to_ascii_lowercase();
    let allowed = ["image/png", "image/jpeg", "image/webp"];
    if !allowed.contains(&content_type.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Only PNG/JPEG/WEBP allowed".into(),
        ));
    }
    let expected_ext = match content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        _ => "webp",
    };
    let filename = body.filename.trim().to_ascii_lowercase();
    if !(filename.ends_with(expected_ext) || (expected_ext == "jpg" && filename.ends_with("jpeg")))
    {
        return Err(FeelPayError::BadRequest(
            "Avatar filename extension must match image type".into(),
        ));
    }

    let raw = base64::engine::general_purpose::STANDARD
        .decode(body.data_base64.trim())
        .map_err(|_| FeelPayError::BadRequest("Invalid base64 payload".into()))?;

    if raw.len() < 100 * 1024 {
        return Err(FeelPayError::BadRequest(
            "Avatar must be at least 100KB".into(),
        ));
    }
    if raw.len() > 5 * 1024 * 1024 {
        return Err(FeelPayError::BadRequest("Avatar must be <= 5MB".into()));
    }
    let magic_ok = match content_type.as_str() {
        "image/png" => raw.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => raw.starts_with(b"\xff\xd8\xff"),
        "image/webp" => raw.len() > 12 && raw.starts_with(b"RIFF") && &raw[8..12] == b"WEBP",
        _ => false,
    };
    if !magic_ok {
        return Err(FeelPayError::BadRequest(
            "Avatar file content does not match image type".into(),
        ));
    }

    let ext = expected_ext;
    let file_id = format!("{}_{}", merchant_id, Uuid::new_v4().simple());
    let dir = std::path::Path::new("uploads/avatars");
    std::fs::create_dir_all(dir).map_err(|e| FeelPayError::Internal(format!("Create dir: {e}")))?;
    let path = dir.join(format!("{}.{}", file_id, ext));
    std::fs::write(&path, &raw).map_err(|e| FeelPayError::Internal(format!("Write file: {e}")))?;

    let avatar_url = format!("/uploads/avatars/{}.{}", file_id, ext);

    sqlx::query(
        r#"
        UPDATE merchants
        SET metadata = jsonb_set(COALESCE(metadata, '{}'::jsonb), '{avatar_url}', to_jsonb($2::text), true),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&avatar_url)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "avatar_url": avatar_url,
        "size_bytes": raw.len(),
        "message": "Avatar uploaded"
    })))
}

pub async fn get_account_settings_hub(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let dynamic = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_config WHERE key = 'merchant_account_settings_hub'",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let default = serde_json::json!({
      "sections": [
        {
          "key": "website_app",
          "title": "Website and App Settings",
          "icon": "monitor",
          "links": [
            { "key": "api_keys", "label": "API keys", "route": "/developers", "enabled": true },
            { "key": "webhooks", "label": "Webhooks", "route": "/webhooks", "enabled": true },
            { "key": "checkout_preview", "label": "Checkout preview", "route": "/checkout-preview", "enabled": true }
          ]
        },
        {
          "key": "business",
          "title": "Business Settings",
          "icon": "briefcase",
          "links": [
            { "key": "account_details", "label": "Account details", "route": "/onboarding", "enabled": true },
            { "key": "business_details", "label": "Business details", "route": "/settings", "enabled": true },
            { "key": "manage_team", "label": "Manage team", "route": "/settings", "enabled": true }
          ]
        },
        {
          "key": "payments",
          "title": "Payments and Refunds",
          "icon": "wallet",
          "links": [
            { "key": "payments", "label": "Payments", "route": "/payments", "enabled": true },
            { "key": "settlements", "label": "Settlements", "route": "/settlements", "enabled": true },
            { "key": "disputes", "label": "Disputes", "route": "/disputes", "enabled": true }
          ]
        },
        {
          "key": "notifications",
          "title": "Notification Settings",
          "icon": "bell",
          "links": [
            { "key": "notification_channels", "label": "Email / SMS / Alerts", "route": "/notifications", "enabled": true },
            { "key": "user_preferences", "label": "User preferences", "route": "/notifications", "enabled": true }
          ]
        }
      ]
    });

    let payload = dynamic.unwrap_or(default);

    let sections = payload
        .get("sections")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();

    let sections: Vec<AccountSettingsHubSection> = sections
        .iter()
        .map(|raw| {
            let links = raw
                .get("links")
                .and_then(|l| l.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|link| AccountSettingsHubLink {
                    key: link
                        .get("key")
                        .and_then(|v| v.as_str())
                        .unwrap_or("link")
                        .to_string(),
                    label: link
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Link")
                        .to_string(),
                    route: link
                        .get("route")
                        .and_then(|v| v.as_str())
                        .unwrap_or("/")
                        .to_string(),
                    enabled: link
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                })
                .collect();
            AccountSettingsHubSection {
                key: raw
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("section")
                    .to_string(),
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Section")
                    .to_string(),
                icon: raw
                    .get("icon")
                    .and_then(|v| v.as_str())
                    .unwrap_or("grid")
                    .to_string(),
                links,
            }
        })
        .collect();

    let _ = merchant_id;
    Ok(HttpResponse::Ok().json(AccountSettingsHubResponse { sections }))
}

#[utoipa::path(
    get,
    path = "/v1/merchants/settings/security",
    responses(
        (status = 200, description = "Merchant security settings snapshot"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_security_settings(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row = sqlx::query(
        r#"
        SELECT
            webhook_secret,
            mfa_enabled,
            settlement_cycle,
            per_transaction_limit,
            daily_limit,
            monthly_limit,
            allowed_payment_methods,
            updated_at
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let webhook_secret = row.get::<Option<String>, _>(0);
    let webhook_hint = webhook_secret
        .as_deref()
        .map(|s| {
            if s.len() > 8 {
                format!("{}...{}", &s[..6], &s[s.len() - 4..])
            } else {
                "configured".to_string()
            }
        })
        .unwrap_or_else(|| "not_set".to_string());

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "mfa_enabled": row.get::<bool, _>(1),
        "settlement_cycle": row.get::<String, _>(2),
        "per_transaction_limit": row.get::<i64, _>(3),
        "daily_limit": row.get::<i64, _>(4),
        "monthly_limit": row.get::<i64, _>(5),
        "allowed_payment_methods": row.get::<serde_json::Value, _>(6),
        "webhook_secret_hint": webhook_hint,
        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>(7)
    })))
}

pub async fn get_dashboard_signals(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let announcements: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT value->'announcements' FROM system_config WHERE key = 'merchant_dashboard_signals'",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .and_then(|v: serde_json::Value| v.as_array().cloned())
    .unwrap_or_else(|| {
        vec![
            serde_json::json!({
                "id": "default-test-mode",
                "title": "Test Mode Active",
                "message": "Real money disabled. Validate checkout, webhooks, and settlement flows safely.",
                "severity": "info"
            }),
            serde_json::json!({
                "id": "default-webhooks",
                "title": "Webhook Hygiene",
                "message": "Keep webhook endpoints HTTPS and rotate webhook secret periodically.",
                "severity": "warning"
            }),
        ]
    });

    let provider_health: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT jsonb_build_object(
            'name', display_name,
            'success_rate', success_rate,
            'avg_latency_ms', avg_latency_ms,
            'is_active', is_active,
            'circuit_breaker_state', circuit_breaker_state
        )
        FROM payment_providers
        ORDER BY success_rate DESC, avg_latency_ms ASC
        LIMIT 5
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let merchant_status: Option<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT jsonb_build_object(
            'onboarding_status', onboarding_status,
            'status', status,
            'registration_step', registration_step,
            'updated_at', updated_at
        )
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "announcements": announcements,
        "ecosystem_health": provider_health,
        "merchant_status": merchant_status
    })))
}

pub async fn get_checkout_settings(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let settings: Option<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(registration_data->'checkout_settings', '{}'::jsonb)
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let fallback = serde_json::json!({
        "styling": {
            "background_color": "#C63201",
            "border_style": "rounded",
            "font_family": "IBM Plex Sans",
            "show_sidebar_graphic": false,
            "show_trusted_badge": true
        },
        "features": {
            "show_mandate_summary": false,
            "default_language": "English",
            "collect_email": false,
            "show_message_banner": false,
            "flash_checkout": true
        },
        "payment_configuration": {
            "preset_name": "Standard payment options",
            "allow_custom_configuration": true
        }
    });

    Ok(HttpResponse::Ok().json(settings.unwrap_or(fallback)))
}

pub async fn update_checkout_settings(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UpdateCheckoutSettingsRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let actor_id = req
        .extensions()
        .get::<crate::middleware::auth::UserId>()
        .map(|v| v.0.to_string())
        .unwrap_or_else(|| "merchant".to_string());

    let previous_settings: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT COALESCE(registration_data->'checkout_settings', '{}'::jsonb)
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    .unwrap_or_else(|| serde_json::json!({}));

    let payload = serde_json::json!({
        "styling": body.styling.clone().unwrap_or_else(|| serde_json::json!({})),
        "features": body.features.clone().unwrap_or_else(|| serde_json::json!({})),
        "payment_configuration": body.payment_configuration.clone().unwrap_or_else(|| serde_json::json!({}))
    });

    sqlx::query(
        r#"
        UPDATE merchants
        SET registration_data = jsonb_set(
            COALESCE(registration_data, '{}'::jsonb),
            '{checkout_settings}',
            $2::jsonb,
            true
        ),
        updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&payload)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs (entity_type, entity_id, action, old_state, new_state, actor_type, actor_id)
        VALUES ('merchant_checkout_settings', $1, 'checkout_settings_updated', $2::jsonb, $3::jsonb, 'merchant_user', $4)
        "#,
    )
    .bind(merchant_id)
    .bind(previous_settings)
    .bind(payload.clone())
    .bind(actor_id)
    .execute(&state.db)
    .await;

    write_merchant_audit_log(
        &state.db,
        &req,
        merchant_id,
        "checkout_settings_updated",
        "merchant_checkout_settings",
        payload.clone(),
    )
    .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Checkout settings updated",
        "settings": payload
    })))
}

pub async fn get_checkout_settings_audit(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let rows = sqlx::query(
        r#"
        SELECT created_at, actor_id, old_state, new_state
        FROM audit_logs
        WHERE entity_type = 'merchant_checkout_settings'
          AND entity_id = $1
          AND action = 'checkout_settings_updated'
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>(0),
                "actor_id": row.get::<Option<String>, _>(1),
                "old_state": row.get::<Option<serde_json::Value>, _>(2),
                "new_state": row.get::<Option<serde_json::Value>, _>(3)
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({ "items": items })))
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ListMerchantAuditLogsQuery {
    pub action: Option<String>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/v1/merchants/audit-logs",
    params(
        ListMerchantAuditLogsQuery
    ),
    responses(
        (status = 200, description = "Merchant audit logs")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_merchant_audit_logs(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListMerchantAuditLogsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let tenant_id = tenant_id_from_req(&req);
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let action = query
        .action
        .as_deref()
        .filter(|value| !value.eq_ignore_ascii_case("all"));

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            COALESCE(merchant_id, entity_id) AS scoped_merchant_id,
            tenant_id,
            action,
            COALESCE(target, entity_type || ':' || entity_id::TEXT) AS target,
            COALESCE(metadata, new_state, '{}'::jsonb) AS metadata,
            actor_type,
            actor_id,
            ip_address,
            user_agent,
            created_at
        FROM audit_logs
        WHERE (merchant_id = $1 OR (merchant_id IS NULL AND entity_id = $1))
          AND ($2::UUID IS NULL OR tenant_id IS NULL OR tenant_id = $2)
          AND ($3::TEXT IS NULL OR action = $3)
        ORDER BY created_at DESC
        LIMIT $4
        "#,
    )
    .bind(merchant_id)
    .bind(tenant_id)
    .bind(action)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "merchant_id": row.get::<Uuid, _>("scoped_merchant_id"),
                "action": row.get::<String, _>("action"),
                "target": row.get::<String, _>("target"),
                "metadata": row.get::<serde_json::Value, _>("metadata"),
                "actor_type": row.get::<Option<String>, _>("actor_type"),
                "actor_id": row.get::<Option<String>, _>("actor_id"),
                "ip_address": row.get::<Option<String>, _>("ip_address"),
                "user_agent": row.get::<Option<String>, _>("user_agent"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({ "items": items })))
}

#[utoipa::path(
    post,
    path = "/v1/merchants/webhook-secret/roll",
    responses(
        (status = 200, description = "Webhook secret rotated successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn roll_webhook_secret(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let new_secret = format!("whsec_{}", Uuid::new_v4().simple());
    let hint = format!(
        "{}...{}",
        &new_secret[..6],
        &new_secret[new_secret.len() - 4..]
    );
    let (ciphertext, nonce) = encrypt_webhook_secret(state.get_ref(), new_secret.clone())?;

    sqlx::query(
        "UPDATE merchants SET webhook_secret = NULL, webhook_secret_ciphertext = $2, webhook_secret_nonce = $3, webhook_secret_hint = $4, updated_at = NOW() WHERE id = $1",
    )
    .bind(merchant_id)
    .bind(ciphertext)
    .bind(nonce)
    .bind(&hint)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let allow_secret_reveal =
        std::env::var("ALLOW_SECRET_REVEAL").unwrap_or_else(|_| "false".into()) == "true";

    let mut response = serde_json::json!({
        "message": "Webhook secret rotated. Update receiver verification key.",
        "webhook_secret_hint": hint
    });

    if allow_secret_reveal {
        response["webhook_secret"] = serde_json::json!(new_secret);
    }

    write_merchant_audit_log(
        &state.db,
        &req,
        merchant_id,
        "webhook_secret_rotated",
        "webhook_secret",
        serde_json::json!({ "webhook_secret_hint": hint }),
    )
    .await;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/v1/merchants/keys",
    responses(
        (status = 200, description = "Returns merchant API keys"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_keys(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let env = request_environment(&req);
    let row = sqlx::query(
        r#"
        SELECT masked_key, created_at, last_used_at
        FROM developer_api_keys
        WHERE merchant_id = $1
          AND environment = $2
          AND status = 'active'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .bind(env)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        let should_generate = if env == "live" {
            let approved: bool = sqlx::query_scalar(
                "SELECT onboarding_status = 'approved' AND COALESCE(review_status, 'draft') = 'approved' FROM merchants WHERE id = $1",
            )
            .bind(merchant_id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(false);
            approved
        } else {
            true
        };

        if should_generate {
            let api_key = if env == "live" {
                format!("fp_live_{}", Uuid::new_v4().simple())
            } else {
                format!("fp_test_{}", Uuid::new_v4().simple())
            };
            
            if let Ok(_) = store_active_api_key(&state.db, merchant_id, env, &api_key).await {
                let display = masked_key(&api_key)["display"].as_str().unwrap_or_default().to_string();
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "api_key": display,
                    "key": { "display": display },
                    "environment": env,
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "last_used_at": null,
                    "scopes": ["payments:write", "payments:read", "webhooks:write"],
                    "status": "active"
                })));
            }
        }

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "api_key": null,
            "key": null,
            "environment": env,
            "created_at": null,
            "last_used_at": null,
            "scopes": [],
            "status": "none"
        })));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": row.get::<String, _>("masked_key"),
        "key": { "display": row.get::<String, _>("masked_key") },
        "environment": env,
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        "last_used_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_used_at"),
        "scopes": ["payments:write", "payments:read", "webhooks:write"],
        "status": "active"
    })))
}

#[utoipa::path(
    post,
    path = "/v1/merchants/keys/roll",
    responses(
        (status = 200, description = "API keys rolled successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn roll_keys(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let env = request_environment(&req);
    if env == "live" {
        let approved: bool = sqlx::query_scalar(
            "SELECT onboarding_status = 'approved' AND COALESCE(review_status, 'draft') = 'approved' FROM merchants WHERE id = $1",
        )
        .bind(merchant_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
        if !approved {
            return Err(FeelPayError::Forbidden(
                "Live API keys require explicit merchant approval".into(),
            ));
        }
    }
    let api_key = if env == "live" {
        format!("fp_live_{}", Uuid::new_v4().simple())
    } else {
        format!("fp_test_{}", Uuid::new_v4().simple())
    };
    let api_secret = format!("fp_secret_{}", Uuid::new_v4().simple());

    let argon2 = Argon2::default();
    let secret_salt = SaltString::generate(&mut OsRng);
    let api_secret_hash = argon2
        .hash_password(api_secret.as_bytes(), &secret_salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;
    sqlx::query("UPDATE merchants SET api_secret_hash = $2, updated_at = NOW() WHERE id = $1")
        .bind(merchant_id)
        .bind(&api_secret_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    sqlx::query(
        "UPDATE developer_api_keys SET status = 'revoked', updated_at = NOW() WHERE merchant_id = $1 AND environment = $2 AND status = 'active'",
    )
    .bind(merchant_id)
    .bind(env)
    .execute(&mut *tx)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    store_active_api_key(&mut *tx, merchant_id, env, &api_key).await?;
    tx.commit()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let resp = serde_json::json!({
        "api_key": masked_key(&api_key)["display"].as_str().unwrap_or_default(),
        "key": masked_key(&api_key),
        "environment": env,
        "api_secret": api_secret,
        "api_key_once": api_key,
        "message": "Store your new API secret safely. It may not be shown again."
    });

    write_audit_log(
        &state.db,
        merchant_id,
        "merchant_api_key_rotated",
        serde_json::json!({"environment": env, "key": masked_key(&api_key)}),
    )
    .await;

    Ok(HttpResponse::Ok().json(resp))
}

#[utoipa::path(
    post,
    path = "/v1/merchants/login",
    request_body = MerchantLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login_merchant(
    state: web::Data<AppState>,
    body: web::Json<MerchantLoginRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant = sqlx::query(
        r#"
        SELECT id, name, email, password_hash, api_key
        FROM merchants
        WHERE email = $1
        "#,
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let m = merchant.ok_or(FeelPayError::Unauthorized("Invalid credentials".into()))?;
    let merchant_id: Uuid = m.get("id");
    let name: String = m.get("name");
    let email: String = m.get("email");
    let password_hash: String = m.get("password_hash");
    let api_key: Option<String> = m.get("api_key");

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|_| FeelPayError::Unauthorized("Invalid credentials".into()))?;

    argon2
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| FeelPayError::Unauthorized("Invalid credentials".into()))?;

    let mfa_enabled = if has_merchant_column(&state.db, "mfa_enabled").await? {
        sqlx::query_scalar::<_, bool>(
            "SELECT COALESCE(mfa_enabled, false) FROM merchants WHERE id = $1",
        )
        .bind(merchant_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    } else {
        false
    };

    if mfa_enabled {
        let mfa_token = create_mfa_token(merchant_id, &body.email)?;
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "mfa_required",
            "mfa_token": mfa_token,
            "merchant_id": merchant_id
        })));
    }

    let (user_id, tenant_id, role) = merchant_session_context(&state.db, merchant_id).await?;
    let token =
        crate::middleware::auth::create_merchant_token(merchant_id, user_id, tenant_id, &role)?;
    let expires_at = merchant_session_expires_at();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "token": token,
        "merchant_id": merchant_id,
        "user_id": user_id,
        "role": role,
        "name": name,
        "email": email,
        "api_key": api_key.map(|key| masked_key(&key)["display"].clone()),
        "expires_in": MERCHANT_SESSION_SECONDS,
        "expires_at": expires_at.to_rfc3339()
    })))
}

pub async fn logout_merchant(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let revoked = crate::middleware::auth::bearer_token_from_request(&req)
        .map(|token| {
            crate::middleware::auth::revoke_bearer_token(
                state.get_ref(),
                &token,
                MERCHANT_SESSION_SECONDS as usize,
            )
        })
        .unwrap_or(false);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out",
        "session_status": "terminated",
        "server_revoked": revoked
    })))
}

fn create_mfa_token(id: Uuid, _email: &str) -> Result<String, FeelPayError> {
    // Generate a short-lived token for MFA
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = crate::models::user::Claims {
        sub: id.to_string(),
        tenant_id: None,
        merchant_id: Some(id),
        role: "merchant_mfa_pending".to_string(),
        exp: now + 300, // 5 mins
        iat: now,
    };

    let secret = crate::middleware::auth::jwt_secret_for_token()?;
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| FeelPayError::Internal(e.to_string()))
}

#[utoipa::path(
    post,
    path = "/v1/merchants/mfa/verify",
    request_body = crate::api::auth::MfaVerifyRequest,
    responses(
        (status = 200, description = "MFA verified successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn verify_mfa_merchant(
    state: web::Data<AppState>,
    body: web::Json<crate::api::auth::MfaVerifyRequest>,
) -> Result<HttpResponse, FeelPayError> {
    if let Some(mfa_token) = &body.mfa_token {
        let secret_key = crate::middleware::auth::jwt_secret_for_token()?;
        let token_data = jsonwebtoken::decode::<crate::models::user::Claims>(
            mfa_token,
            &jsonwebtoken::DecodingKey::from_secret(secret_key.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| FeelPayError::Unauthorized("Invalid MFA token".into()))?;

        if token_data.claims.role != "merchant_mfa_pending" {
            return Err(FeelPayError::Unauthorized("Invalid MFA token role".into()));
        }

        let merchant_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| FeelPayError::Internal("Invalid ID in token".into()))?;

        let merchant: Option<Merchant> = sqlx::query_as("SELECT * FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        let m = merchant.ok_or(FeelPayError::Unauthorized("Merchant not found".into()))?;

        if let Some(mfa_secret) = &m.mfa_secret {
            if MfaService::verify_token(mfa_secret, &body.code)? {
                let (user_id, tenant_id, role) = merchant_session_context(&state.db, m.id).await?;
                let token = crate::middleware::auth::create_merchant_token(
                    m.id, user_id, tenant_id, &role,
                )?;
                let expires_at = merchant_session_expires_at();
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "token": token,
                    "merchant_id": m.id,
                    "user_id": user_id,
                    "role": role,
                    "name": m.name,
                    "email": m.email,
                    "api_key": m.api_key.map(|key| masked_key(&key)["display"].clone()),
                    "expires_in": MERCHANT_SESSION_SECONDS,
                    "expires_at": expires_at.to_rfc3339()
                })))
            } else {
                Err(FeelPayError::Unauthorized("Invalid MFA code".into()))
            }
        } else {
            Err(FeelPayError::BadRequest(
                "MFA not enabled for this merchant".into(),
            ))
        }
    } else {
        Err(FeelPayError::BadRequest("Missing mfa_token".into()))
    }
}

#[utoipa::path(
    post,
    path = "/v1/merchants/reset-password",
    responses(
        (status = 200, description = "Password reset successful")
    )
)]
pub async fn reset_password() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"message": "Password reset successful (simulated)"})))
}

#[utoipa::path(
    post,
    path = "/v1/merchants/forgot-password",
    responses(
        (status = 200, description = "Instructions sent")
    )
)]
pub async fn forgot_password() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"message": "Password reset instructions sent (simulated)"})))
}

// ============ Team Management ============

#[derive(Deserialize, ToSchema)]
pub struct InviteTeamMemberRequest {
    pub email: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub role: String,
}

/// List team members for the current merchant
/// List team members for the current merchant
#[utoipa::path(
    get,
    path = "/v1/merchants/team",
    responses(
        (status = 200, description = "List of team members", body = [Object]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_team_members(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let rows = sqlx::query(
        r#"
        SELECT u.id, u.email, u.full_name, u.status, u.created_at, r.name as role_name
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE u.merchant_id = $1
        ORDER BY u.created_at DESC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let members: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>(0),
                "email": row.get::<String, _>(1),
                "full_name": row.get::<Option<String>, _>(2),
                "status": row.get::<String, _>(3),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>(4),
                "role": row.get::<String, _>(5),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(members))
}

/// Invite a new team member
/// Invite a new team member
#[utoipa::path(
    post,
    path = "/v1/merchants/team/invite",
    request_body = InviteTeamMemberRequest,
    responses(
        (status = 201, description = "Team member invited successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn invite_team_member(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<InviteTeamMemberRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    require_team_admin(&req)?;
    ensure_team_roles_exist(&state.db).await?;
    ensure_valid_team_role(&body.role)?;

    // Get tenant_id from merchant
    let merchant_row = sqlx::query("SELECT tenant_id FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let tenant_id: Option<Uuid> = merchant_row.get(0);

    // Check if user already exists
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if existing.is_some() {
        return Err(FeelPayError::Conflict(
            "User with this email already exists".into(),
        ));
    }

    // Generate a temporary password (never returned)
    let temp_password = format!("temp_{}", Uuid::new_v4().simple());
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(temp_password.as_bytes(), &salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    // Get role ID
    let role_row = sqlx::query("SELECT id FROM roles WHERE name = $1")
        .bind(&body.role)
        .fetch_one(&state.db)
        .await
        .map_err(|_| FeelPayError::Internal(format!("Role '{}' not found", body.role)))?;

    let role_id: Uuid = role_row.get(0);

    // Create user
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, tenant_id, merchant_id, email, password_hash, full_name, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        "#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(merchant_id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.full_name)
    .execute(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Assign role
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    write_merchant_audit_log(
        &state.db,
        &req,
        merchant_id,
        "team_member_invited",
        "team_member",
        serde_json::json!({
            "user_id": user_id,
            "email": body.email,
            "role": body.role
        }),
    )
    .await;

    // In a real app, send invitation email here (password setup link).
    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": user_id,
        "email": body.email,
        "full_name": body.full_name,
        "role": body.role,
        "status": "pending",
        "message": "Invitation created. User must set password via reset flow."
    })))
}

/// Remove a team member
/// Remove a team member
#[utoipa::path(
    delete,
    path = "/v1/merchants/team/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Team member removed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn remove_team_member(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    require_team_admin(&req)?;
    let user_id = path.into_inner();

    // Verify user belongs to this merchant
    let row = sqlx::query("SELECT merchant_id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some(row) => {
            let user_merchant_id: Option<Uuid> = row.get(0);
            if user_merchant_id != Some(merchant_id) {
                return Err(FeelPayError::Forbidden(
                    "Cannot remove users from other merchants".into(),
                ));
            }

            // Soft delete by setting status to 'removed'
            sqlx::query("UPDATE users SET status = 'removed', updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .execute(&state.db)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            write_merchant_audit_log(
                &state.db,
                &req,
                merchant_id,
                "team_member_removed",
                "team_member",
                serde_json::json!({ "user_id": user_id }),
            )
            .await;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Team member removed"
            })))
        }
        None => Err(FeelPayError::NotFound("User not found".into())),
    }
}

/// Update a team member's role
/// Update a team member's role
#[utoipa::path(
    put,
    path = "/v1/merchants/team/{user_id}/role",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User can not be updated")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn update_team_member_role(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    require_team_admin(&req)?;
    let user_id = path.into_inner();
    ensure_team_roles_exist(&state.db).await?;
    ensure_valid_team_role(&body.role)?;

    // Verify user belongs to this merchant
    let row = sqlx::query("SELECT merchant_id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some(row) => {
            let user_merchant_id: Option<Uuid> = row.get(0);
            if user_merchant_id != Some(merchant_id) {
                return Err(FeelPayError::Forbidden(
                    "Cannot update users from other merchants".into(),
                ));
            }

            // Get new role ID
            let role_row = sqlx::query("SELECT id FROM roles WHERE name = $1")
                .bind(&body.role)
                .fetch_one(&state.db)
                .await
                .map_err(|_| FeelPayError::Internal(format!("Role '{}' not found", body.role)))?;

            let role_id: Uuid = role_row.get(0);

            // Update role
            sqlx::query("UPDATE user_roles SET role_id = $1 WHERE user_id = $2")
                .bind(role_id)
                .bind(user_id)
                .execute(&state.db)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            write_merchant_audit_log(
                &state.db,
                &req,
                merchant_id,
                "team_member_role_updated",
                "team_member",
                serde_json::json!({
                    "user_id": user_id,
                    "role": body.role
                }),
            )
            .await;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Role updated",
                "new_role": body.role
            })))
        }
        None => Err(FeelPayError::NotFound("User not found".into())),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateKycStatusRequest {
    pub status: String,
    pub message: Option<String>,
}

/// Update KYC status and notify merchant (admin action)
/// Update KYC status and notify merchant (admin action)
#[utoipa::path(
    put,
    path = "/v1/merchants/{merchant_id}/kyc-status",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    request_body = UpdateKycStatusRequest,
    responses(
        (status = 200, description = "KYC status updated successfully"),
        (status = 404, description = "Merchant not found")
    )
)]
pub async fn update_kyc_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateKycStatusRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = path.into_inner();

    // 1. Update merchant table
    let result = sqlx::query(
        r#"
        UPDATE merchants 
        SET onboarding_status = $1, 
            status = CASE WHEN $1 = 'approved' THEN 'active' ELSE status END,
            updated_at = NOW() 
        WHERE id = $2 
        RETURNING email
        "#,
    )
    .bind(&body.status)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = result else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let merchant_email: String = row.get(0);

    // 2. Create InApp notification
    let notification_title = match body.status.as_str() {
        "approved" => "KYC Approved",
        "rejected" => "KYC Rejected",
        "resubmit_required" => "KYC Documents Required",
        _ => "KYC Status Updated",
    };

    let notification_message = body
        .message
        .clone()
        .unwrap_or_else(|| match body.status.as_str() {
            "approved" => {
                "Your KYC has been approved! You can now start accepting payments.".to_string()
            }
            "rejected" => {
                "Your KYC application has been rejected. Please contact support.".to_string()
            }
            _ => format!("Your KYC status has been updated to {}.", body.status),
        });

    sqlx::query(
        r#"
        INSERT INTO notification_queue (
            notification_type, recipient_type, recipient_id, recipient_email,
            subject, body, status, entity_type, entity_id, metadata
        )
        VALUES ('in_app', 'merchant', $1, $2, $3, $4, 'sent', 'merchant', $1, $5)
        "#,
    )
    .bind(merchant_id)
    .bind(&merchant_email)
    .bind(notification_title)
    .bind(&notification_message)
    .bind(serde_json::json!({
        "status": body.status,
        "type": "kyc_update"
    }))
    .execute(&state.db)
    .await?;

    // 3. Send Email notification (simulated via email service)
    let _ = crate::engine::services::email::send_kyc_status_email(
        &state.db,
        &merchant_email,
        &body.status,
        body.message.as_deref(),
    )
    .await;
    let event_type = match body.status.as_str() {
        "approved" => Some("kyc_approved"),
        "rejected" => Some("kyc_rejected"),
        _ => None,
    };
    if let Some(event_type) = event_type {
        let mut variables = std::collections::BTreeMap::new();
        variables.insert("status".to_string(), body.status.clone());
        let _ = dispatch_event(
            &state.db,
            DispatchRequest {
                merchant_id: Some(merchant_id),
                event_type: event_type.to_string(),
                environment: if state.config.is_live() {
                    "live"
                } else {
                    "test"
                }
                .to_string(),
                recipient_email: Some(merchant_email.clone()),
                recipient_phone: None,
                variables,
            },
        )
        .await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "KYC status updated and notifications queued",
        "status": body.status
    })))
}

#[derive(Deserialize)]
pub struct UpdatePaymentMethodBody {
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}

pub async fn list_payment_methods(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = extract_merchant_id_internal(&req)?;

    let methods: Vec<crate::models::merchant::MerchantPaymentMethod> = sqlx::query_as(
        "SELECT method_type, enabled, config FROM merchant_payment_methods WHERE merchant_id = $1",
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await?;

    let response: Vec<_> = methods
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "method_type": m.method_type,
                "enabled": m.enabled,
                "config": m.config
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn update_payment_method(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdatePaymentMethodBody>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = extract_merchant_id_internal(&req)?;
    let method_type = path.into_inner();

    sqlx::query!(
        r#"
        INSERT INTO merchant_payment_methods (merchant_id, method_type, enabled, config)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (merchant_id, method_type) 
        DO UPDATE SET enabled = $3, config = $4, updated_at = NOW()
        "#,
        merchant_id,
        method_type,
        body.enabled,
        body.config
    )
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "updated" })))
}

pub async fn request_activation(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = extract_merchant_id_internal(&req)?;

    // Move into the explicit internal-review lifecycle once the merchant submits.
    let res = sqlx::query(
        "UPDATE merchants SET onboarding_status = 'submitted', review_status = 'submitted' WHERE id = $1 AND onboarding_status IN ('draft', 'resubmit_required', 'pending') RETURNING onboarding_status"
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match res {
        Some(_) => Ok(HttpResponse::Ok().json(
            serde_json::json!({ "status": "submitted", "message": "Activation request submitted" }),
        )),
        None => Err(FeelPayError::BadRequest(
            "KYC already submitted or approved".into(),
        )),
    }
}

fn extract_merchant_id_internal(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

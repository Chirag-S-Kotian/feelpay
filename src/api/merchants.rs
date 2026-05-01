use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::engine::services::auth::MfaService;
use crate::error::FeelPayError;
use crate::models::merchant::{CreateMerchantRequest, Merchant, MerchantLoginRequest};
use crate::state::AppState;
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
            .route("/keys", web::get().to(get_keys))
            .route("/keys/roll", web::post().to(roll_keys))
            .route("/settings/security", web::get().to(get_security_settings))
            .route(
                "/webhook-secret/roll",
                web::post().to(roll_webhook_secret),
            )
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
    pub settlement_account: Option<serde_json::Value>,
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
    let api_key = format!("fp_live_{}", Uuid::new_v4().simple());
    let api_secret = format!("fp_secret_{}", Uuid::new_v4().simple());
    let webhook_secret = format!("whsec_{}", Uuid::new_v4().simple());

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
        r#"INSERT INTO merchants (id, tenant_id, name, email, password_hash, api_key, api_secret_hash, webhook_url, webhook_secret, business_name, business_type) 
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#
    )
    .bind(id).bind(body.tenant_id).bind(&body.name).bind(&body.email).bind(&password_hash)
    .bind(&api_key).bind(&api_secret_hash).bind(&body.webhook_url)
    .bind(&webhook_secret).bind(&body.business_name).bind(&body.business_type)
    .execute(&mut *tx).await
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
            "tenant_id": row.get::<Option<Uuid>, _>(1),
            "name": row.get::<String, _>(2),
            "email": row.get::<String, _>(3),
            "api_key": row.get::<String, _>(4),
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

async fn has_dual_api_key_columns(pool: &sqlx::PgPool) -> Result<bool, FeelPayError> {
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
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    Ok(count == 2)
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
        "tenant_id": row.get::<Option<Uuid>, _>(1),
        "name": row.get::<String, _>(2),
        "email": row.get::<String, _>(3),
        "api_key": row.get::<String, _>(4),
        "business_name": row.get::<Option<String>, _>(5),
        "webhook_url": row.get::<Option<String>, _>(6),
        "status": row.get::<String, _>(7),
        "onboarding_status": row.get::<String, _>(8),
        "daily_limit": row.get::<i64, _>(9),
        "monthly_limit": row.get::<i64, _>(10)
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

    if let Some(url) = body.webhook_url.as_ref() {
        if !url.starts_with("https://") {
            return Err(FeelPayError::BadRequest(
                "Webhook URL must start with https://".into(),
            ));
        }
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
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&body.name)
    .bind(&body.business_name)
    .bind(&body.webhook_url)
    .bind(&body.settlement_account)
    .bind(&body.business_type)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    get_profile(req, state).await
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

    sqlx::query("UPDATE merchants SET webhook_secret = $2, updated_at = NOW() WHERE id = $1")
        .bind(merchant_id)
        .bind(&new_secret)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let allow_secret_reveal =
        std::env::var("ALLOW_SECRET_REVEAL").unwrap_or_else(|_| "false".into()) == "true";
    let hint = format!("{}...{}", &new_secret[..6], &new_secret[new_secret.len() - 4..]);

    let mut response = serde_json::json!({
        "message": "Webhook secret rotated. Update receiver verification key.",
        "webhook_secret_hint": hint
    });

    if allow_secret_reveal {
        response["webhook_secret"] = serde_json::json!(new_secret);
    }

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
    let dual_cols = has_dual_api_key_columns(&state.db).await?;
    let row = if dual_cols {
        sqlx::query("SELECT api_key, api_key_test, api_key_live FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await
    } else {
        sqlx::query("SELECT api_key FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await
    }
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let selected = if dual_cols {
        let api_key_legacy = row.get::<String, _>(0);
        let api_key_test = row.get::<Option<String>, _>(1);
        let api_key_live = row.get::<Option<String>, _>(2);
        let mut selected = if env == "live" {
            api_key_live.unwrap_or_else(|| api_key_legacy.clone())
        } else {
            api_key_test.unwrap_or_else(|| api_key_legacy.clone())
        };

        // Self-heal legacy backfill mismatch: keep env-prefixed keys consistent.
        let expected_prefix = if env == "live" { "fp_live_" } else { "fp_test_" };
        if !selected.starts_with(expected_prefix) {
            let regenerated = if env == "live" {
                format!("fp_live_{}", Uuid::new_v4().simple())
            } else {
                format!("fp_test_{}", Uuid::new_v4().simple())
            };

            let sql = if env == "live" {
                "UPDATE merchants SET api_key_live = $2, updated_at = NOW() WHERE id = $1"
            } else {
                "UPDATE merchants SET api_key_test = $2, updated_at = NOW() WHERE id = $1"
            };

            sqlx::query(sql)
                .bind(merchant_id)
                .bind(&regenerated)
                .execute(&state.db)
                .await
                .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

            selected = regenerated;
        }

        selected
    } else {
        row.get::<String, _>(0)
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": selected,
        "environment": env
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
    let dual_cols = has_dual_api_key_columns(&state.db).await?;
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

    if dual_cols && env == "live" {
        sqlx::query(
            "UPDATE merchants SET api_key_live = $2, api_secret_hash = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(merchant_id)
        .bind(&api_key)
        .bind(&api_secret_hash)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    } else if dual_cols {
        sqlx::query(
            "UPDATE merchants SET api_key_test = $2, api_secret_hash = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(merchant_id)
        .bind(&api_key)
        .bind(&api_secret_hash)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    } else {
        sqlx::query(
            "UPDATE merchants SET api_key = $2, api_secret_hash = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(merchant_id)
        .bind(&api_key)
        .bind(&api_secret_hash)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    }

    let allow_secret_reveal =
        std::env::var("ALLOW_SECRET_REVEAL").unwrap_or_else(|_| "false".into()) == "true";

    let mut resp = serde_json::json!({
        "api_key": api_key,
        "environment": env,
        "message": "API key rotated."
    });

    if allow_secret_reveal {
        resp["api_secret"] = serde_json::json!(api_secret);
        resp["message"] = serde_json::json!("Store your new API secret safely. It may not be shown again.");
    }

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
    let merchant: Option<Merchant> = sqlx::query_as("SELECT * FROM merchants WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let m = merchant.ok_or(FeelPayError::Unauthorized("Invalid credentials".into()))?;

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&m.password_hash)
        .map_err(|_| FeelPayError::Internal("Hash error".into()))?;

    argon2
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| FeelPayError::Unauthorized("Invalid credentials".into()))?;

    if m.mfa_enabled {
        let mfa_token = create_mfa_token(m.id, &body.email)?;
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "mfa_required",
            "mfa_token": mfa_token,
            "merchant_id": m.id
        })));
    }

    let token = crate::middleware::auth::create_token(m.id, &body.email)?;
    let expires_at = merchant_session_expires_at();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "token": token,
        "merchant_id": m.id,
        "name": m.name,
        "email": m.email,
        "api_key": m.api_key,
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

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
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
        let secret_key =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
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
                let token = crate::middleware::auth::create_token(m.id, &m.email)?;
                let expires_at = merchant_session_expires_at();
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "token": token,
                    "merchant_id": m.id,
                    "name": m.name,
                    "email": m.email,
                    "api_key": m.api_key,
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
    pub role: String, // merchant_admin, merchant_staff, merchant_viewer
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

    // Validate role
    let valid_roles = ["merchant_admin", "merchant_staff", "merchant_viewer"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Invalid role. Must be one of: merchant_admin, merchant_staff, merchant_viewer".into(),
        ));
    }

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
    let user_id = path.into_inner();

    // Validate role
    let valid_roles = ["merchant_admin", "merchant_staff", "merchant_viewer"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Invalid role. Must be one of: merchant_admin, merchant_staff, merchant_viewer".into(),
        ));
    }

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

    // Update status to 'pending' if it was 'draft' or 'resubmit_required'
    let res = sqlx::query(
        "UPDATE merchants SET onboarding_status = 'pending' WHERE id = $1 AND onboarding_status IN ('draft', 'resubmit_required') RETURNING onboarding_status"
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match res {
        Some(_) => Ok(HttpResponse::Ok().json(
            serde_json::json!({ "status": "pending", "message": "Activation request submitted" }),
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

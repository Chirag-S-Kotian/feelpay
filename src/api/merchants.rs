use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

use crate::engine::services::auth::MfaService;
use crate::error::FeelPayError;
use crate::models::merchant::{CreateMerchantRequest, Merchant, MerchantLoginRequest};
use crate::state::AppState;
use sqlx::Row;

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
            .route("/{merchant_id}", web::get().to(get_merchant))
            // Team management
            .route("/team", web::get().to(list_team_members))
            .route("/team/invite", web::post().to(invite_team_member))
            .route("/team/{user_id}", web::delete().to(remove_team_member))
            .route("/team/{user_id}/role", web::put().to(update_team_member_role)),
    );
}

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

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": id, "name": body.name, "email": body.email,
        "api_key": api_key, "api_secret": api_secret,
        "webhook_secret": webhook_secret,
        "message": "Store your API secret safely."
    })))
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
        Some(row) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": row.get::<Uuid, _>(0),
                "tenant_id": row.get::<Option<Uuid>, _>(1),
                "name": row.get::<String, _>(2),
                "email": row.get::<String, _>(3),
                "api_key": row.get::<String, _>(4),
                "business_name": row.get::<Option<String>, _>(5),
                "status": row.get::<String, _>(6),
            })))
        }
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

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

    let Some(row) = row
    else {
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

pub async fn update_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<crate::models::merchant::UpdateMerchantRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    sqlx::query(
        r#"
        UPDATE merchants
        SET
            name = COALESCE($2, name),
            business_name = COALESCE($3, business_name),
            webhook_url = COALESCE($4, webhook_url),
            settlement_account = COALESCE($5, settlement_account),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&body.name)
    .bind(&body.business_name)
    .bind(&body.webhook_url)
    .bind(&body.settlement_account)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    get_profile(req, state).await
}

pub async fn get_keys(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row = sqlx::query("SELECT api_key FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": row.get::<String, _>(0)
    })))
}


pub async fn roll_keys(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let api_key = format!("fp_live_{}", Uuid::new_v4().simple());
    let api_secret = format!("fp_secret_{}", Uuid::new_v4().simple());

    let argon2 = Argon2::default();
    let secret_salt = SaltString::generate(&mut OsRng);
    let api_secret_hash = argon2
        .hash_password(api_secret.as_bytes(), &secret_salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    sqlx::query(
        "UPDATE merchants SET api_key = $2, api_secret_hash = $3, updated_at = NOW() WHERE id = $1",
    )
    .bind(merchant_id)
    .bind(&api_key)
    .bind(&api_secret_hash)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": api_key,
        "api_secret": api_secret,
        "message": "Store your new API secret safely. It will not be shown again."
    })))
}

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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "token": token,
        "merchant_id": m.id
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
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "token": token,
                    "merchant_id": m.id
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

pub async fn reset_password() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"message": "Password reset successful (simulated)"})))
}

pub async fn forgot_password() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok()
        .json(serde_json::json!({"message": "Password reset instructions sent (simulated)"})))
}

// ============ Team Management ============

use serde::Deserialize;

#[derive(Deserialize)]
pub struct InviteTeamMemberRequest {
    pub email: String,
    pub full_name: String,
    pub role: String, // merchant_admin, merchant_staff, merchant_viewer
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

/// List team members for the current merchant
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
        return Err(FeelPayError::Conflict("User with this email already exists".into()));
    }

    // Generate a temporary password
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

    // In a real app, send invitation email here
    // For now, return the temp password (in production, this would be a reset link)
    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": user_id,
        "email": body.email,
        "full_name": body.full_name,
        "role": body.role,
        "status": "pending",
        "message": "Invitation sent. User must set their password on first login.",
        "temp_password": temp_password // Remove in production
    })))
}

/// Remove a team member
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

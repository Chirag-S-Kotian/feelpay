use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::merchant::{CreateMerchantRequest, MerchantLoginRequest};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/merchants")
            .route("", web::post().to(create_merchant))
            .route("/login", web::post().to(login_merchant))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route("/keys", web::get().to(get_keys))
            .route("/keys/roll", web::post().to(roll_keys))
            .route("/{merchant_id}", web::get().to(get_merchant)),
    );
}

pub async fn create_merchant(
    state: web::Data<AppState>,
    body: web::Json<CreateMerchantRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let pool = state.db.clone();
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

    sqlx::query(
        r#"INSERT INTO merchants (id, name, email, password_hash, api_key, api_secret_hash, webhook_url, webhook_secret, business_name, business_type) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"#
    )
    .bind(&id).bind(&body.name).bind(&body.email).bind(&password_hash)
    .bind(&api_key).bind(&api_secret_hash).bind(&body.webhook_url)
    .bind(&webhook_secret).bind(&body.business_name).bind(&body.business_type)
    .execute(&pool).await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            FeelPayError::Conflict("Email already registered".into())
        } else {
            FeelPayError::Database(e.to_string())
        }
    })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": id, "name": body.name, "email": body.email,
        "api_key": api_key, "api_secret": api_secret,
        "webhook_secret": webhook_secret,
        "message": "Store your API secret safely."
    })))
}

pub async fn login_merchant(
    state: web::Data<AppState>,
    body: web::Json<MerchantLoginRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let pool = state.db.clone();
    let row: Option<(Uuid, String, String, String)> = sqlx::query_as(
        "SELECT id, name, password_hash, api_key FROM merchants WHERE email = $1 AND status = 'active'"
    ).bind(&body.email).fetch_optional(&pool).await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let (id, name, pw_hash, api_key) =
        row.ok_or_else(|| FeelPayError::NotFound("Merchant not found".into()))?;

    let parsed =
        PasswordHash::new(&pw_hash).map_err(|e| FeelPayError::Internal(format!("{}", e)))?;
    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed)
        .map_err(|_| FeelPayError::Unauthorized("Invalid credentials".into()))?;

    let token = generate_jwt(&id.to_string(), &name)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token, "merchant_id": id, "name": name, "api_key": api_key,
    })))
}

pub async fn get_merchant(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let id = path.into_inner();
    let pool = state.db.clone();
    let row: Option<(Uuid, String, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, name, email, api_key, business_name, status FROM merchants WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some((id, name, email, api_key, biz, status)) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id, "name": name, "email": email,
                "api_key": api_key, "business_name": biz, "status": status,
            })))
        }
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
}

fn generate_jwt(merchant_id: &str, name: &str) -> Result<String, FeelPayError> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        name: String,
        exp: usize,
        iat: usize,
    }
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: merchant_id.into(),
        name: name.into(),
        exp: now + 86400,
        iat: now,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| FeelPayError::Internal(format!("JWT: {}", e)))
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

pub async fn get_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row: Option<(
        Uuid,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        i64,
        i64,
    )> = sqlx::query_as(
        r#"
        SELECT id, name, email, api_key, business_name, webhook_url, status, onboarding_status, daily_limit, monthly_limit
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((id, name, email, api_key, business_name, webhook_url, status, onboarding_status, daily_limit, monthly_limit)) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "name": name,
        "email": email,
        "api_key": api_key,
        "business_name": business_name,
        "webhook_url": webhook_url,
        "status": status,
        "onboarding_status": onboarding_status,
        "daily_limit": daily_limit,
        "monthly_limit": monthly_limit
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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    get_profile(req, state).await
}

pub async fn get_keys(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row: Option<(String,)> = sqlx::query_as("SELECT api_key FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((api_key,)) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": api_key
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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "api_key": api_key,
        "api_secret": api_secret,
        "message": "Store your new API secret safely. It will not be shown again."
    })))
}

pub async fn forgot_password(
    state: web::Data<AppState>,
    body: web::Json<crate::models::merchant::ForgotPasswordRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let pool = state.db.clone();
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM merchants WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((merchant_id,)) = row else {
        // Return success even if not found to avoid email enumeration
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "If an account exists with that email, a password reset link has been sent."
        })));
    };

    let token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    sqlx::query("INSERT INTO password_reset_tokens (token, merchant_id, expires_at) VALUES ($1, $2, $3)")
        .bind(&token)
        .bind(&merchant_id)
        .bind(&expires_at)
        .execute(&pool)
        .await
        .map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    // In production, send email here. For simulation, we log it and return it in dev mode if needed.
    println!("PASSWORD RESET TOKEN FOR {}: {}", body.email, token);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "If an account exists with that email, a password reset link has been sent.",
        "debug_token": token // Remove in real production
    })))
}

pub async fn reset_password(
    state: web::Data<AppState>,
    body: web::Json<crate::models::merchant::ResetPasswordRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let pool = state.db.clone();
    
    let row: Option<(Uuid,)> = sqlx::query_as(
        "DELETE FROM password_reset_tokens WHERE token = $1 AND expires_at > NOW() RETURNING merchant_id"
    )
    .bind(&body.token)
    .fetch_optional(&pool)
    .await
    .map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((merchant_id,)) = row else {
        return Err(FeelPayError::BadRequest("Invalid or expired token".into()));
    };

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(body.new_password.as_bytes(), &salt)
        .map_err(|e| FeelPayError::Internal(format!("Hash: {}", e)))?
        .to_string();

    sqlx::query("UPDATE merchants SET password_hash = $2, updated_at = NOW() WHERE id = $1")
        .bind(&merchant_id)
        .bind(&password_hash)
        .execute(&pool)
        .await
        .map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password successfully reset."
    })))
}

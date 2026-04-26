use actix_web::{web, HttpResponse};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng, PasswordHash};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::merchant::{CreateMerchantRequest, MerchantLoginRequest};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/merchants")
            .route("", web::post().to(create_merchant))
            .route("/login", web::post().to(login_merchant))
            .route("/{merchant_id}", web::get().to(get_merchant))
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

    let (id, name, pw_hash, api_key) = row
        .ok_or_else(|| FeelPayError::NotFound("Merchant not found".into()))?;

    let parsed = PasswordHash::new(&pw_hash)
        .map_err(|e| FeelPayError::Internal(format!("{}", e)))?;
    Argon2::default().verify_password(body.password.as_bytes(), &parsed)
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
        "SELECT id, name, email, api_key, business_name, status FROM merchants WHERE id = $1"
    ).bind(&id).fetch_optional(&pool).await
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
    struct Claims { sub: String, name: String, exp: usize, iat: usize }
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims { sub: merchant_id.into(), name: name.into(), exp: now + 86400, iat: now };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| FeelPayError::Internal(format!("JWT: {}", e)))
}

pub async fn get_profile() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_keys() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn roll_keys() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok().finish())
}

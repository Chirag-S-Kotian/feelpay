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
    let mut tx = state.db.begin().await.map_err(|e| FeelPayError::Database(e.to_string()))?;
    
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
    .bind(&id).bind(&body.tenant_id).bind(&body.name).bind(&body.email).bind(&password_hash)
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
           VALUES ($1, $2, $3, $4, $5, $6, 'active')"#
    )
    .bind(&user_id).bind(&body.tenant_id).bind(&id).bind(&body.email).bind(&password_hash).bind(&body.name)
    .execute(&mut *tx).await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // 3. Assign Role (merchant_owner)
    let role: (Uuid,) = sqlx::query_as("SELECT id FROM roles WHERE name = 'merchant_owner'")
        .fetch_one(&mut *tx).await
        .map_err(|_| FeelPayError::Internal("Default role 'merchant_owner' not found".into()))?;

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(&user_id).bind(&role.0)
        .execute(&mut *tx).await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    tx.commit().await.map_err(|e| FeelPayError::Database(e.to_string()))?;

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
    let pool = state.db.clone();
    let row: Option<(Uuid, Option<Uuid>, String, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, tenant_id, name, email, api_key, business_name, status FROM merchants WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some((id, tenant_id, name, email, api_key, biz, status)) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id, "tenant_id": tenant_id, "name": name, "email": email,
                "api_key": api_key, "business_name": biz, "status": status,
            })))
        }
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
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
        Option<Uuid>,
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
        SELECT id, tenant_id, name, email, api_key, business_name, webhook_url, status, onboarding_status, daily_limit, monthly_limit
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((
        id,
        tenant_id,
        name,
        email,
        api_key,
        business_name,
        webhook_url,
        status,
        onboarding_status,
        daily_limit,
        monthly_limit,
    )) = row
    else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "tenant_id": tenant_id,
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


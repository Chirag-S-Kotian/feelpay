use actix_web::{web, HttpRequest, HttpResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde::Deserialize;
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AdminUserRequest {
    pub email: String,
    pub password: Option<String>,
    pub name: String,
    pub role: String,
    pub tenant_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub totp_secret: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/users")
            .route("", web::get().to(list_admin_users))
            .route("", web::post().to(create_admin_user))
            .route("/{id}", web::put().to(update_admin_user))
            .route("/{id}", web::delete().to(deactivate_admin_user)),
    );
}

pub async fn list_admin_users(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(t) - 'password_hash' - 'totp_secret'
        FROM admin_users t
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn create_admin_user(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AdminUserRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let actor = admin_id_from_claims(&req)?;
    let password = body
        .password
        .as_deref()
        .ok_or_else(|| FeelPayError::BadRequest("Password required".into()))?;
    let password_hash = hash_password(password)?;
    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        INSERT INTO admin_users (email, password_hash, name, role, tenant_id, is_active, totp_secret)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, TRUE), $7)
        RETURNING to_jsonb(admin_users.*) - 'password_hash' - 'totp_secret'
        "#,
    )
    .bind(&body.email)
    .bind(password_hash)
    .bind(&body.name)
    .bind(&body.role)
    .bind(body.tenant_id)
    .bind(body.is_active)
    .bind(&body.totp_secret)
    .fetch_one(&state.db)
    .await?;
    let id = Uuid::parse_str(row["id"].as_str().unwrap_or_default()).ok();
    log_audit(
        &state.db,
        actor,
        "admin_user_create",
        "admin_user",
        id,
        None,
        Some(row.clone()),
        None,
    )
    .await?;
    Ok(HttpResponse::Created().json(row))
}

pub async fn update_admin_user(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<AdminUserRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let actor = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    let previous: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(admin_users.*) - 'password_hash' - 'totp_secret' FROM admin_users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let password_hash = match body.password.as_deref() {
        Some(password) => Some(hash_password(password)?),
        None => None,
    };
    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        UPDATE admin_users
        SET email = $2, name = $3, role = $4, tenant_id = $5,
            is_active = COALESCE($6, is_active),
            totp_secret = COALESCE($7, totp_secret),
            password_hash = COALESCE($8, password_hash),
            updated_at = NOW()
        WHERE id = $1
        RETURNING to_jsonb(admin_users.*) - 'password_hash' - 'totp_secret'
        "#,
    )
    .bind(id)
    .bind(&body.email)
    .bind(&body.name)
    .bind(&body.role)
    .bind(body.tenant_id)
    .bind(body.is_active)
    .bind(&body.totp_secret)
    .bind(password_hash)
    .fetch_one(&state.db)
    .await?;
    log_audit(
        &state.db,
        actor,
        "admin_user_update",
        "admin_user",
        Some(id),
        previous,
        Some(row.clone()),
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(row))
}

pub async fn deactivate_admin_user(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let actor = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    sqlx::query("UPDATE admin_users SET is_active = FALSE, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    log_audit(
        &state.db,
        actor,
        "admin_user_deactivate",
        "admin_user",
        Some(id),
        None,
        None,
        None,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

fn hash_password(password: &str) -> Result<String, FeelPayError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| FeelPayError::Internal(format!("Password hash failed: {}", e)))
}

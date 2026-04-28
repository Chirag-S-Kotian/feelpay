use actix_web::{web, HttpRequest, HttpResponse};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CredentialUpsertRequest {
    pub provider: String,
    pub key_name: String,
    pub value: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CredentialResponse {
    pub id: Uuid,
    pub provider: String,
    pub key_name: String,
    pub masked_value: String,
    pub last_validated: Option<chrono::DateTime<chrono::Utc>>,
    pub is_valid: Option<bool>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/provider-credentials")
            .route("", web::get().to(list_credentials))
            .route("", web::post().to(upsert_credential))
            .route("/{id}/validate", web::post().to(validate_credential)),
    );
}

fn mask_credential(value: &str) -> String {
    if value.len() <= 4 {
        return "****".to_string();
    }
    format!("****{}", &value[value.len() - 4..])
}

fn cipher(state: &AppState) -> Result<Aes256Gcm, FeelPayError> {
    let key = hex::decode(&state.config.vault_encryption_key)
        .map_err(|_| FeelPayError::Internal("Invalid encryption key".into()))?;
    Aes256Gcm::new_from_slice(&key)
        .map_err(|_| FeelPayError::Internal("Invalid encryption key length".into()))
}

fn encrypt_value(state: &AppState, value: &str) -> Result<(String, String), FeelPayError> {
    let cipher = cipher(state)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), value.as_bytes())
        .map_err(|_| FeelPayError::Internal("Credential encryption failed".into()))?;
    Ok((BASE64.encode(ciphertext), BASE64.encode(nonce_bytes)))
}

pub async fn list_credentials(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    let rows = sqlx::query_as::<_, CredentialResponse>(
        r#"
        SELECT id, provider, key_name, '****' AS masked_value, last_validated, is_valid
        FROM provider_credentials
        ORDER BY provider, key_name
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(rows))
}

pub async fn upsert_credential(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CredentialUpsertRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let (encrypted_value, nonce) = encrypt_value(&state, &body.value)?;
    let masked = mask_credential(&body.value);
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO provider_credentials (provider, key_name, encrypted_value, nonce)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (provider, key_name)
        DO UPDATE SET encrypted_value = EXCLUDED.encrypted_value,
                      nonce = EXCLUDED.nonce,
                      updated_at = NOW(),
                      is_valid = TRUE
        RETURNING id
        "#,
    )
    .bind(&body.provider)
    .bind(&body.key_name)
    .bind(encrypted_value)
    .bind(nonce)
    .fetch_one(&state.db)
    .await?;

    log_audit(
        &state.db,
        admin_id,
        "provider_credential_upsert",
        "provider_credential",
        Some(row.0),
        None,
        Some(serde_json::json!({ "provider": body.provider, "key_name": body.key_name, "masked": masked })),
        None,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": row.0, "masked_value": masked })))
}

pub async fn validate_credential(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    sqlx::query("UPDATE provider_credentials SET last_validated = NOW(), is_valid = TRUE, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    log_audit(
        &state.db,
        admin_id,
        "provider_credential_validate",
        "provider_credential",
        Some(id),
        None,
        Some(serde_json::json!({ "is_valid": true })),
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id, "is_valid": true })))
}

use crate::error::FeelPayError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Failed,
    Pending,
    ProviderError,
    ManualReviewRequired,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Failed => "failed",
            Self::Pending => "pending",
            Self::ProviderError => "provider_error",
            Self::ManualReviewRequired => "manual_review_required",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub status: VerificationStatus,
    pub provider_id: Option<Uuid>,
    pub provider_reference: Option<String>,
    pub response_summary: Value,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
struct ProviderRow {
    id: Uuid,
    environment: String,
    status: String,
    config: Value,
}

pub async fn verify_pan(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    environment: &str,
    pan: &str,
) -> Result<VerificationResult, FeelPayError> {
    verify_identifier(pool, merchant_id, environment, "pan", pan, None).await
}

pub async fn verify_gstin(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    environment: &str,
    gstin: &str,
) -> Result<VerificationResult, FeelPayError> {
    verify_identifier(pool, merchant_id, environment, "gstin", gstin, None).await
}

pub async fn verify_business(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    environment: &str,
    legal_name: &str,
) -> Result<VerificationResult, FeelPayError> {
    verify_identifier(pool, merchant_id, environment, "business", legal_name, None).await
}

pub async fn test_connection(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<VerificationResult, FeelPayError> {
    let provider = provider_by_id(pool, provider_id).await?;
    if provider.environment != "test" {
        return Err(FeelPayError::BadRequest(
            "KYC provider tests require a test-mode provider".into(),
        ));
    }
    verify_with_provider(
        pool,
        None,
        &provider,
        "pan",
        "ABCDE1234F",
        Some("safe_test"),
    )
    .await
}

async fn verify_identifier(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    environment: &str,
    verification_type: &str,
    identifier: &str,
    scenario: Option<&str>,
) -> Result<VerificationResult, FeelPayError> {
    let Some(provider) = default_provider(pool, environment).await? else {
        let result = VerificationResult {
            status: VerificationStatus::ManualReviewRequired,
            provider_id: None,
            provider_reference: None,
            response_summary: json!({ "reason": "provider_missing" }),
            error_code: Some("provider_missing".to_string()),
            error_message: Some("No enabled default KYC provider configured".to_string()),
        };
        persist_attempt(
            pool,
            merchant_id,
            environment,
            verification_type,
            identifier,
            &result,
            0,
        )
        .await?;
        return Ok(result);
    };
    verify_with_provider(
        pool,
        merchant_id,
        &provider,
        verification_type,
        identifier,
        scenario,
    )
    .await
}

async fn verify_with_provider(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    provider: &ProviderRow,
    verification_type: &str,
    identifier: &str,
    scenario: Option<&str>,
) -> Result<VerificationResult, FeelPayError> {
    let started = Instant::now();
    if provider.status != "enabled" {
        let result = VerificationResult {
            status: VerificationStatus::ManualReviewRequired,
            provider_id: Some(provider.id),
            provider_reference: None,
            response_summary: json!({ "reason": "provider_disabled" }),
            error_code: Some("provider_disabled".to_string()),
            error_message: Some("KYC provider is disabled".to_string()),
        };
        persist_attempt(
            pool,
            merchant_id,
            &provider.environment,
            verification_type,
            identifier,
            &result,
            0,
        )
        .await?;
        return Ok(result);
    }
    let outcome = if provider
        .config
        .get("provider_code")
        .and_then(Value::as_str)
        .unwrap_or("simulator")
        == "simulator"
    {
        simulator_result(provider.id, verification_type, scenario)
    } else {
        send_generic_request(pool, provider, verification_type, identifier).await
    };
    let latency_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
    let result = match outcome {
        Ok(result) => result,
        Err((code, message)) => VerificationResult {
            status: VerificationStatus::ProviderError,
            provider_id: Some(provider.id),
            provider_reference: None,
            response_summary: json!({ "provider_error": true }),
            error_code: Some(code),
            error_message: Some(message),
        },
    };
    persist_attempt(
        pool,
        merchant_id,
        &provider.environment,
        verification_type,
        identifier,
        &result,
        latency_ms,
    )
    .await?;
    Ok(result)
}

async fn default_provider(
    pool: &PgPool,
    environment: &str,
) -> Result<Option<ProviderRow>, FeelPayError> {
    let row = sqlx::query(
        r#"
        SELECT id, environment, status, config
        FROM integration_providers
        WHERE provider_kind = 'kyc'
          AND environment = $1
          AND is_default = true
        ORDER BY priority, name
        LIMIT 1
        "#,
    )
    .bind(environment)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    Ok(row.map(provider_from_row))
}

async fn provider_by_id(pool: &PgPool, provider_id: Uuid) -> Result<ProviderRow, FeelPayError> {
    sqlx::query(
        "SELECT id, environment, status, config FROM integration_providers WHERE id=$1 AND provider_kind='kyc'",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .map(provider_from_row)
    .ok_or_else(|| FeelPayError::NotFound("KYC provider not found".into()))
}

fn provider_from_row(row: sqlx::postgres::PgRow) -> ProviderRow {
    ProviderRow {
        id: row.get("id"),
        environment: row.get("environment"),
        status: row.get("status"),
        config: row.get("config"),
    }
}

fn simulator_result(
    provider_id: Uuid,
    verification_type: &str,
    scenario: Option<&str>,
) -> Result<VerificationResult, (String, String)> {
    let status = match scenario.unwrap_or("safe_test") {
        "safe_test" => VerificationStatus::Verified,
        "manual_review" => VerificationStatus::ManualReviewRequired,
        "pending" => VerificationStatus::Pending,
        "failed" => VerificationStatus::Failed,
        _ => VerificationStatus::ProviderError,
    };
    Ok(VerificationResult {
        status,
        provider_id: Some(provider_id),
        provider_reference: Some(format!(
            "sim_{}_{}",
            verification_type,
            Utc::now().timestamp()
        )),
        response_summary: json!({
            "adapter": "simulator",
            "verification_type": verification_type
        }),
        error_code: None,
        error_message: None,
    })
}

async fn send_generic_request(
    pool: &PgPool,
    provider: &ProviderRow,
    verification_type: &str,
    identifier: &str,
) -> Result<VerificationResult, (String, String)> {
    let base_url = required_string(&provider.config, "base_url")?;
    let token = load_secret(pool, provider.id, "auth_token").await?;
    let response = reqwest::Client::new()
        .post(format!("{base_url}/verify/{verification_type}"))
        .bearer_auth(token)
        .json(&json!({ "identifier": identifier }))
        .send()
        .await
        .map_err(|_| {
            (
                "provider_request_failed".to_string(),
                "KYC provider request failed".to_string(),
            )
        })?;
    let status = response.status();
    let payload = response.json::<Value>().await.map_err(|_| {
        (
            "provider_response_invalid".to_string(),
            "KYC provider response was invalid".to_string(),
        )
    })?;
    if !status.is_success() {
        return Err((
            "provider_http_error".to_string(),
            format!("KYC provider returned {status}"),
        ));
    }
    Ok(VerificationResult {
        status: parse_status(payload.get("status").and_then(Value::as_str)),
        provider_id: Some(provider.id),
        provider_reference: payload
            .get("reference")
            .and_then(Value::as_str)
            .map(str::to_string),
        response_summary: json!({
            "status": payload.get("status").and_then(Value::as_str).unwrap_or("provider_error"),
            "name_match": payload.get("name_match").and_then(Value::as_bool),
            "document_metadata_present": payload.get("document_metadata").is_some()
        }),
        error_code: None,
        error_message: None,
    })
}

fn parse_status(value: Option<&str>) -> VerificationStatus {
    match value {
        Some("verified") => VerificationStatus::Verified,
        Some("failed") => VerificationStatus::Failed,
        Some("pending") => VerificationStatus::Pending,
        Some("manual_review_required") => VerificationStatus::ManualReviewRequired,
        _ => VerificationStatus::ProviderError,
    }
}

fn required_string(config: &Value, key: &str) -> Result<String, (String, String)> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            (
                "provider_config_missing".to_string(),
                format!("Missing required provider config: {key}"),
            )
        })
}

async fn load_secret(
    pool: &PgPool,
    provider_id: Uuid,
    secret_name: &str,
) -> Result<String, (String, String)> {
    let row = sqlx::query(
        "SELECT encrypted_value, nonce FROM integration_provider_secrets WHERE provider_id=$1 AND secret_name=$2",
    )
    .bind(provider_id)
    .bind(secret_name)
    .fetch_optional(pool)
    .await
    .map_err(|_| ("secret_lookup_failed".to_string(), "Provider secret lookup failed".to_string()))?
    .ok_or_else(|| ("secret_missing".to_string(), format!("Missing provider secret: {secret_name}")))?;
    decrypt_secret(row.get("encrypted_value"), row.get("nonce"))
}

fn decrypt_secret(encrypted_value: String, nonce: String) -> Result<String, (String, String)> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    let raw_key = std::env::var("VAULT_ENCRYPTION_KEY").map_err(|_| {
        (
            "vault_key_missing".to_string(),
            "VAULT_ENCRYPTION_KEY is required".to_string(),
        )
    })?;
    let key = hex::decode(raw_key.trim()).map_err(|_| {
        (
            "vault_key_invalid".to_string(),
            "VAULT_ENCRYPTION_KEY is invalid".to_string(),
        )
    })?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| {
        (
            "vault_key_invalid".to_string(),
            "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string(),
        )
    })?;
    let encrypted = BASE64.decode(encrypted_value).map_err(|_| {
        (
            "secret_invalid".to_string(),
            "Stored secret is invalid".to_string(),
        )
    })?;
    let nonce = BASE64.decode(nonce).map_err(|_| {
        (
            "secret_invalid".to_string(),
            "Stored secret nonce is invalid".to_string(),
        )
    })?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
        .map_err(|_| {
            (
                "secret_decrypt_failed".to_string(),
                "Credential decryption failed".to_string(),
            )
        })?;
    String::from_utf8(decrypted).map_err(|_| {
        (
            "secret_invalid".to_string(),
            "Stored secret encoding is invalid".to_string(),
        )
    })
}

async fn persist_attempt(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    environment: &str,
    verification_type: &str,
    identifier: &str,
    result: &VerificationResult,
    latency_ms: i32,
) -> Result<(), FeelPayError> {
    sqlx::query(
        r#"
        INSERT INTO kyc_provider_attempts (
            merchant_id, provider_id, verification_type, environment, identifier_masked,
            result_status, provider_reference, response_summary, latency_ms, error_code, error_message
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(merchant_id)
    .bind(result.provider_id)
    .bind(verification_type)
    .bind(environment)
    .bind(mask_identifier(identifier))
    .bind(result.status.as_str())
    .bind(&result.provider_reference)
    .bind(&result.response_summary)
    .bind(latency_ms)
    .bind(&result.error_code)
    .bind(&result.error_message)
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    Ok(())
}

pub fn mask_identifier(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    format!(
        "****{}",
        chars[chars.len() - 4..].iter().collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_parser_is_fail_closed() {
        assert_eq!(parse_status(Some("verified")).as_str(), "verified");
        assert_eq!(parse_status(Some("unexpected")).as_str(), "provider_error");
    }

    #[test]
    fn masks_sensitive_identifiers() {
        assert_eq!(mask_identifier("ABCDE1234F"), "****234F");
        assert_eq!(mask_identifier("12"), "****");
    }
}

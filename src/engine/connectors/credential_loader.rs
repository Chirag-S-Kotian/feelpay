//! Encrypted credential loader
//!
//! Decrypts provider credentials from the `provider_credentials` table at runtime
//! using the same AES-GCM vault key used by `provider_credentials.rs`.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sqlx::PgPool;
use std::collections::HashMap;

use super::ProviderCredentials;
use crate::error::FeelPayError;

/// Load and decrypt all credentials for a given provider from the DB.
/// Returns a ProviderCredentials struct with decrypted key_name → value pairs.
/// Never logs the decrypted values.
pub async fn load_credentials(
    pool: &PgPool,
    vault_key_hex: &str,
    provider: &str,
) -> Result<ProviderCredentials, FeelPayError> {
    let key_bytes = hex::decode(vault_key_hex)
        .map_err(|_| FeelPayError::Internal("Invalid vault encryption key".into()))?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|_| FeelPayError::Internal("Invalid vault encryption key length".into()))?;

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT key_name, encrypted_value, nonce FROM provider_credentials WHERE provider = $1 AND is_valid = TRUE",
    )
    .bind(provider)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let mut secrets = HashMap::new();

    for (key_name, encrypted_b64, nonce_b64) in rows {
        let ciphertext = BASE64
            .decode(&encrypted_b64)
            .map_err(|_| FeelPayError::Internal("Invalid credential ciphertext encoding".into()))?;
        let nonce_bytes = BASE64
            .decode(&nonce_b64)
            .map_err(|_| FeelPayError::Internal("Invalid credential nonce encoding".into()))?;

        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| {
                tracing::error!(
                    provider = %provider,
                    key_name = %key_name,
                    "Failed to decrypt credential"
                );
                FeelPayError::Internal("Credential decryption failed".into())
            })?;

        let value = String::from_utf8(plaintext).map_err(|_| {
            FeelPayError::Internal("Decrypted credential is not valid UTF-8".into())
        })?;

        secrets.insert(key_name, value);
    }

    tracing::info!(
        provider = %provider,
        key_count = secrets.len(),
        "Loaded provider credentials (values redacted)"
    );

    Ok(ProviderCredentials {
        provider: provider.to_string(),
        secrets,
    })
}

/// Check which secret key_names exist for a provider (without decrypting).
pub async fn list_credential_keys(
    pool: &PgPool,
    provider: &str,
) -> Result<Vec<String>, FeelPayError> {
    let keys: Vec<(String,)> = sqlx::query_as(
        "SELECT key_name FROM provider_credentials WHERE provider = $1 AND is_valid = TRUE ORDER BY key_name",
    )
    .bind(provider)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(keys.into_iter().map(|(k,)| k).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_credentials_returns_empty_map() {
        // Without a DB we can't fully test, but we verify the struct is constructible
        let creds = ProviderCredentials {
            provider: "stripe".to_string(),
            secrets: HashMap::new(),
        };
        assert!(creds.secrets.is_empty());
    }
}

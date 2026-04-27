use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sqlx::PgPool;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::error::FeelPayError;

/// Secure card vault service. PCI-DSS scope reduction.
pub struct CardVaultService {
    db: PgPool,
    encryption_key: Key<Aes256Gcm>,
    key_id: String,
}

#[derive(Debug, Clone)]
pub struct VaultedCard {
    pub token: String,
    pub card_brand: String,
    pub last_four: String,
    pub expiry_month: i32,
    pub expiry_year: i32,
    pub holder_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct VaultLookupRow {
    encrypted_pan: String,
    nonce: String,
    encryption_key_id: String,
}

impl CardVaultService {
    pub fn new(db: PgPool, encryption_key_hex: &str, key_id: String) -> Result<Self, FeelPayError> {
        let key_bytes = hex::decode(encryption_key_hex).map_err(|_| {
            FeelPayError::Internal("Invalid vault encryption key format (not hex)".into())
        })?;

        if key_bytes.len() != 32 {
            return Err(FeelPayError::Internal(
                "Vault encryption key must be 32 bytes (AES-256)".into(),
            ));
        }

        let encryption_key = *Key::<Aes256Gcm>::from_slice(&key_bytes);

        Ok(Self {
            db,
            encryption_key,
            key_id,
        })
    }

    /// Encrypt and vault a PAN (Primary Account Number)
    pub async fn vault_card(
        &self,
        merchant_id: Uuid,
        customer_id: Option<Uuid>,
        mut pan: String,
        expiry_month: i32,
        expiry_year: i32,
        holder_name: Option<String>,
    ) -> Result<VaultedCard, FeelPayError> {
        let brand = crate::engine::processors::card::CardProcessor::detect_brand(&pan).to_string();
        let last_four = crate::engine::processors::card::CardProcessor::last_four(&pan);
        let token = format!("tok_{}", Uuid::new_v4().simple());

        // Encrypt PAN
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let encrypted_pan = cipher
            .encrypt(&nonce, pan.as_bytes())
            .map_err(|_| FeelPayError::Internal("Failed to encrypt PAN".into()))?;

        // Zeroize raw PAN from memory immediately after encryption
        pan.zeroize();

        let encrypted_pan_b64 = B64.encode(encrypted_pan);
        let nonce_b64 = B64.encode(nonce);

        // Store in DB
        sqlx::query(
            r#"
            INSERT INTO card_tokens 
                (token, merchant_id, customer_id, encrypted_pan, encryption_key_id, nonce, last_four, card_brand, expiry_month, expiry_year, holder_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&token)
        .bind(merchant_id)
        .bind(customer_id)
        .bind(&encrypted_pan_b64)
        .bind(&self.key_id)
        .bind(&nonce_b64)
        .bind(&last_four)
        .bind(&brand)
        .bind(expiry_month as i16)
        .bind(expiry_year as i16)
        .bind(&holder_name)
        .execute(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        Ok(VaultedCard {
            token,
            card_brand: brand,
            last_four,
            expiry_month,
            expiry_year,
            holder_name,
        })
    }

    /// Retrieve and decrypt a PAN (Restricted use)
    pub async fn retrieve_pan(
        &self,
        token: &str,
        merchant_id: Uuid,
    ) -> Result<String, FeelPayError> {
        let record = sqlx::query_as::<_, VaultLookupRow>(
            r#"
            SELECT encrypted_pan, nonce, encryption_key_id
            FROM card_tokens
            WHERE token = $1 AND merchant_id = $2
            "#,
        )
        .bind(token)
        .bind(merchant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
        .ok_or_else(|| FeelPayError::NotFound("Card token not found".into()))?;

        if record.encryption_key_id != self.key_id {
            return Err(FeelPayError::Internal(
                "Key rotation not fully implemented yet".into(),
            ));
        }

        let encrypted_pan = B64
            .decode(record.encrypted_pan)
            .map_err(|_| FeelPayError::Internal("Corrupt encrypted PAN payload".into()))?;
        let nonce_bytes = B64
            .decode(record.nonce)
            .map_err(|_| FeelPayError::Internal("Corrupt nonce payload".into()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new(&self.encryption_key);
        let pan_bytes = cipher
            .decrypt(nonce, encrypted_pan.as_ref())
            .map_err(|_| FeelPayError::Internal("Failed to decrypt PAN".into()))?;

        let pan = String::from_utf8(pan_bytes)
            .map_err(|_| FeelPayError::Internal("Decrypted PAN is not valid UTF-8".into()))?;

        Ok(pan)
    }
}

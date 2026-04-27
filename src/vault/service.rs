use sqlx::PgPool;
use uuid::Uuid;

use super::bin_lookup::BinLookup;
use super::crypto::VaultCrypto;
use crate::error::FeelPayError;

/// Secure card vault service. PCI-DSS scope reduction.
/// Now delegates crypto to VaultCrypto and BIN detection to BinLookup.
pub struct CardVaultService {
    db: PgPool,
    crypto: VaultCrypto,
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
        let crypto = VaultCrypto::new(encryption_key_hex, key_id)?;
        Ok(Self { db, crypto })
    }

    /// Encrypt and vault a PAN (Primary Account Number)
    pub async fn vault_card(
        &self,
        merchant_id: Uuid,
        customer_id: Option<Uuid>,
        pan: String,
        expiry_month: i32,
        expiry_year: i32,
        holder_name: Option<String>,
    ) -> Result<VaultedCard, FeelPayError> {
        let brand = BinLookup::detect_brand(&pan).to_string();
        let last_four = BinLookup::last_four(&pan);
        let fingerprint = VaultCrypto::fingerprint(&pan);
        let token = format!("tok_{}", Uuid::new_v4().simple());

        // Encrypt PAN (zeroizes the input internally)
        let (encrypted_pan_b64, nonce_b64) = self.crypto.encrypt(pan)?;

        // Store in DB
        sqlx::query(
            r#"
            INSERT INTO card_tokens 
                (token, merchant_id, customer_id, encrypted_pan, encryption_key_id, nonce, last_four, card_brand, expiry_month, expiry_year, holder_name, fingerprint)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&token)
        .bind(merchant_id)
        .bind(customer_id)
        .bind(&encrypted_pan_b64)
        .bind(&self.crypto.key_id)
        .bind(&nonce_b64)
        .bind(&last_four)
        .bind(&brand)
        .bind(expiry_month as i16)
        .bind(expiry_year as i16)
        .bind(&holder_name)
        .bind(&fingerprint)
        .execute(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

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
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
        .ok_or_else(|| FeelPayError::NotFound("Card token not found".into()))?;

        if record.encryption_key_id != self.crypto.key_id {
            return Err(FeelPayError::InternalError(
                "Key rotation not fully implemented yet".into(),
            ));
        }

        self.crypto.decrypt(&record.encrypted_pan, &record.nonce)
    }

    /// Check if a PAN fingerprint already exists for this merchant (dedup).
    pub async fn find_by_fingerprint(
        &self,
        merchant_id: Uuid,
        pan: &str,
    ) -> Result<Option<VaultedCard>, FeelPayError> {
        let fp = VaultCrypto::fingerprint(pan);

        let row: Option<(String, String, String, i16, i16, Option<String>)> = sqlx::query_as(
            r#"
            SELECT token, card_brand, last_four, expiry_month, expiry_year, holder_name
            FROM card_tokens
            WHERE merchant_id = $1 AND fingerprint = $2
            LIMIT 1
            "#,
        )
        .bind(merchant_id)
        .bind(&fp)
        .fetch_optional(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

        Ok(
            row.map(|(token, card_brand, last_four, em, ey, hn)| VaultedCard {
                token,
                card_brand,
                last_four,
                expiry_month: em as i32,
                expiry_year: ey as i32,
                holder_name: hn,
            }),
        )
    }
}

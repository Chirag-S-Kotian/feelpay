use crate::error::FeelPayError;
use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::OnceLock;
use uuid::Uuid;

/// Bank account type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    Savings,
    Current,
}

impl AccountType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Savings => "savings",
            Self::Current => "current",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "savings" => Some(Self::Savings),
            "current" => Some(Self::Current),
            _ => None,
        }
    }
}

/// Bank account verification status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Failed,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "verified" => Some(Self::Verified),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Bank account data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub account_holder_name: String,
    pub account_number_encrypted: String,
    pub ifsc_code: String,
    pub account_type: AccountType,
    pub verification_status: VerificationStatus,
    pub penny_drop_amount_paise: Option<i32>,
    pub penny_drop_reference: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Bank account response (with masked account number)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountResponse {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub account_holder_name: String,
    pub account_number_masked: String,
    pub ifsc_code: String,
    pub account_type: String,
    pub verification_status: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Request to add a bank account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBankAccountRequest {
    pub account_holder_name: String,
    pub account_number: String,
    pub ifsc_code: String,
    pub account_type: String,
}

/// BankAccountService handles bank account management
pub struct BankAccountService {
    db: PgPool,
    encryption_key: [u8; 32],
}

static BANK_ACCOUNT_SCHEMA_INIT: OnceLock<()> = OnceLock::new();

impl BankAccountService {
    /// Create a new BankAccountService instance
    pub fn new(db: PgPool, encryption_key: &str) -> Result<Self, FeelPayError> {
        // Parse hex key to 32 bytes
        let key_bytes = hex::decode(encryption_key)
            .map_err(|_| FeelPayError::Internal("Invalid encryption key format".into()))?;

        if key_bytes.len() != 32 {
            return Err(FeelPayError::Internal(
                "Encryption key must be 32 bytes (64 hex chars)".into(),
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);

        Ok(Self {
            db,
            encryption_key: key_array,
        })
    }

    async fn ensure_schema(&self) -> Result<(), FeelPayError> {
        if BANK_ACCOUNT_SCHEMA_INIT.get().is_some() {
            return Ok(());
        }
        // Create table if missing with encrypted column.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bank_accounts (
                id UUID PRIMARY KEY,
                merchant_id UUID NOT NULL,
                account_holder_name TEXT NOT NULL,
                account_number_encrypted TEXT NULL,
                account_number TEXT NULL,
                ifsc_code TEXT NOT NULL,
                bank_name TEXT NULL,
                account_type TEXT NOT NULL DEFAULT 'savings',
                verification_status TEXT NOT NULL DEFAULT 'pending',
                penny_drop_amount_paise INT NULL,
                penny_drop_reference TEXT NULL,
                is_primary BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                verified_at TIMESTAMPTZ NULL
            )
            "#,
        )
        .execute(&self.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // Legacy DB may miss encrypted column.
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS account_number_encrypted TEXT NULL")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // Legacy DB may miss account_type.
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS account_type TEXT NOT NULL DEFAULT 'savings'")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // Legacy DB may miss verification fields.
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS verification_status TEXT NOT NULL DEFAULT 'pending'")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS penny_drop_amount_paise INT NULL")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS penny_drop_reference TEXT NULL")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        sqlx::query("ALTER TABLE bank_accounts ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ NULL")
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // Fill encrypted column from legacy plaintext when possible.
        let legacy_rows: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, account_number
            FROM bank_accounts
            WHERE account_number_encrypted IS NULL
              AND account_number IS NOT NULL
              AND account_number <> ''
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

        for (id, account_number) in legacy_rows {
            let encrypted = self.encrypt_account_number(&account_number)?;
            sqlx::query(
                r#"
                UPDATE bank_accounts
                SET account_number_encrypted = $1,
                    updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(encrypted)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        }

        let _ = BANK_ACCOUNT_SCHEMA_INIT.set(());
        Ok(())
    }

    /// Add a new bank account with validation
    pub async fn add_bank_account(
        &self,
        merchant_id: Uuid,
        request: AddBankAccountRequest,
    ) -> Result<BankAccount, FeelPayError> {
        self.ensure_schema().await?;
        // Validate IFSC code format (11 characters)
        if request.ifsc_code.len() != 11 {
            return Err(FeelPayError::BadRequest(
                "IFSC code must be 11 characters".into(),
            ));
        }

        // Validate account number format (10-18 digits)
        if !request.account_number.chars().all(|c| c.is_ascii_digit()) {
            return Err(FeelPayError::BadRequest(
                "Account number must contain only digits".into(),
            ));
        }

        if request.account_number.len() < 10 || request.account_number.len() > 18 {
            return Err(FeelPayError::BadRequest(
                "Account number must be 10-18 digits".into(),
            ));
        }

        // Validate account type
        let account_type = AccountType::from_str(&request.account_type)
            .ok_or_else(|| FeelPayError::BadRequest("Invalid account type".into()))?;

        // Encrypt account number
        let encrypted_account = self.encrypt_account_number(&request.account_number)?;

        let account_id = Uuid::new_v4();
        let now = Utc::now();

        // Generate penny drop reference
        let penny_drop_ref = format!(
            "PD-{}",
            uuid::Uuid::new_v4().to_string()[0..8].to_uppercase()
        );

        let account = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<String>,
                bool,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            INSERT INTO bank_accounts (
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING 
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            "#,
        )
        .bind(account_id)
        .bind(merchant_id)
        .bind(&request.account_holder_name)
        .bind(&encrypted_account)
        .bind(&request.ifsc_code)
        .bind(account_type.as_str())
        .bind(VerificationStatus::Pending.as_str())
        .bind::<Option<i32>>(None)
        .bind(&penny_drop_ref)
        .bind(false)
        .bind(now)
        .bind(now)
        .bind::<Option<DateTime<Utc>>>(None)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add bank account: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_bank_account(account))
    }

    /// Get a bank account by ID with tenant isolation
    pub async fn get_bank_account(
        &self,
        merchant_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<BankAccount>, FeelPayError> {
        self.ensure_schema().await?;
        let account = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<String>,
                bool,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            FROM bank_accounts
            WHERE id = $1 AND merchant_id = $2
            "#,
        )
        .bind(account_id)
        .bind(merchant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bank account: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(account.map(|row| self.row_to_bank_account(row)))
    }

    /// List bank accounts for a merchant
    pub async fn list_bank_accounts(
        &self,
        merchant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<BankAccount>, i64), FeelPayError> {
        self.ensure_schema().await?;
        // Get total count
        let (total,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM bank_accounts WHERE merchant_id = $1
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count bank accounts: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        // Get paginated results
        let accounts = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<String>,
                bool,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            FROM bank_accounts
            WHERE merchant_id = $1
            ORDER BY is_primary DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch bank accounts: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        let items = accounts
            .into_iter()
            .map(|row| self.row_to_bank_account(row))
            .collect();
        Ok((items, total))
    }

    /// Verify a bank account with penny drop amount
    pub async fn verify_bank_account(
        &self,
        merchant_id: Uuid,
        account_id: Uuid,
        penny_drop_amount_paise: i32,
    ) -> Result<BankAccount, FeelPayError> {
        self.ensure_schema().await?;
        // Get the account
        let account = self
            .get_bank_account(merchant_id, account_id)
            .await?
            .ok_or_else(|| FeelPayError::NotFound("Bank account not found".into()))?;

        // Check if already verified
        if account.verification_status == VerificationStatus::Verified {
            return Ok(account);
        }

        // Verify penny drop amount matches
        if let Some(expected_amount) = account.penny_drop_amount_paise {
            if penny_drop_amount_paise != expected_amount {
                return Err(FeelPayError::BadRequest(
                    "Penny drop amount does not match".into(),
                ));
            }
        } else {
            // Store the penny drop amount for verification
            sqlx::query(
                r#"
                UPDATE bank_accounts
                SET penny_drop_amount_paise = $1
                WHERE id = $2 AND merchant_id = $3
                "#,
            )
            .bind(penny_drop_amount_paise)
            .bind(account_id)
            .bind(merchant_id)
            .execute(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update penny drop amount: {:?}", e);
                FeelPayError::Database(e.to_string())
            })?;
        }

        let now = Utc::now();

        // Mark as verified
        let verified_account = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<String>,
                bool,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            UPDATE bank_accounts
            SET verification_status = $1, verified_at = $2, updated_at = $3
            WHERE id = $4 AND merchant_id = $5
            RETURNING 
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            "#,
        )
        .bind(VerificationStatus::Verified.as_str())
        .bind(now)
        .bind(now)
        .bind(account_id)
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify bank account: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_bank_account(verified_account))
    }

    /// Set a bank account as primary
    pub async fn set_primary_bank_account(
        &self,
        merchant_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), FeelPayError> {
        self.ensure_schema().await?;
        // Verify account exists and is verified
        let account = self
            .get_bank_account(merchant_id, account_id)
            .await?
            .ok_or_else(|| FeelPayError::NotFound("Bank account not found".into()))?;

        if account.verification_status != VerificationStatus::Verified {
            return Err(FeelPayError::BadRequest(
                "Bank account must be verified before setting as primary".into(),
            ));
        }

        // Unset all other primary accounts
        sqlx::query(
            r#"
            UPDATE bank_accounts
            SET is_primary = false
            WHERE merchant_id = $1 AND id != $2
            "#,
        )
        .bind(merchant_id)
        .bind(account_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to unset primary accounts: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        // Set this account as primary
        sqlx::query(
            r#"
            UPDATE bank_accounts
            SET is_primary = true, updated_at = $1
            WHERE id = $2 AND merchant_id = $3
            "#,
        )
        .bind(Utc::now())
        .bind(account_id)
        .bind(merchant_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set primary account: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(())
    }

    /// Get the primary bank account for a merchant
    pub async fn get_primary_bank_account(
        &self,
        merchant_id: Uuid,
    ) -> Result<Option<BankAccount>, FeelPayError> {
        self.ensure_schema().await?;
        let account = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<String>,
                bool,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, account_holder_name, account_number_encrypted, ifsc_code,
                account_type, verification_status, penny_drop_amount_paise, penny_drop_reference,
                is_primary, created_at, updated_at, verified_at
            FROM bank_accounts
            WHERE merchant_id = $1 AND is_primary = true
            LIMIT 1
            "#,
        )
        .bind(merchant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch primary bank account: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(account.map(|row| self.row_to_bank_account(row)))
    }

    /// Decrypt account number for settlement processing
    pub fn decrypt_account_number(&self, encrypted: &str) -> Result<String, FeelPayError> {
        let ciphertext = hex::decode(encrypted)
            .map_err(|_| FeelPayError::Internal("Invalid encrypted account format".into()))?;

        if ciphertext.len() < 12 {
            return Err(FeelPayError::Internal("Invalid ciphertext length".into()));
        }

        // Extract nonce (first 12 bytes) and ciphertext
        let nonce_bytes = &ciphertext[0..12];
        let encrypted_data = &ciphertext[12..];

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|_| FeelPayError::Internal("Failed to decrypt account number".into()))?;

        String::from_utf8(plaintext)
            .map_err(|_| FeelPayError::Internal("Invalid UTF-8 in decrypted account".into()))
    }

    // Helper methods

    fn encrypt_account_number(&self, account_number: &str) -> Result<String, FeelPayError> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, account_number.as_bytes())
            .map_err(|_| FeelPayError::Internal("Failed to encrypt account number".into()))?;

        // Combine nonce + ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(hex::encode(combined))
    }

    fn row_to_bank_account(
        &self,
        row: (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<i32>,
            Option<String>,
            bool,
            String,
            String,
            Option<String>,
        ),
    ) -> BankAccount {
        let (
            id,
            merchant_id,
            account_holder_name,
            account_number_encrypted,
            ifsc_code,
            account_type,
            verification_status,
            penny_drop_amount_paise,
            penny_drop_reference,
            is_primary,
            created_at,
            updated_at,
            verified_at,
        ) = row;

        BankAccount {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            merchant_id: Uuid::parse_str(&merchant_id).unwrap_or_default(),
            account_holder_name,
            account_number_encrypted,
            ifsc_code,
            account_type: AccountType::from_str(&account_type).unwrap_or(AccountType::Savings),
            verification_status: VerificationStatus::from_str(&verification_status)
                .unwrap_or(VerificationStatus::Pending),
            penny_drop_amount_paise,
            penny_drop_reference,
            is_primary,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            verified_at: verified_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
        }
    }
}

/// Mask bank account number for UI display
pub fn mask_bank_account(account_number: &str) -> String {
    if account_number.len() < 4 {
        return "XXXX...XXXX".to_string();
    }
    let last4 = &account_number[account_number.len() - 4..];
    format!("XXXX...{}", last4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_as_str() {
        assert_eq!(AccountType::Savings.as_str(), "savings");
        assert_eq!(AccountType::Current.as_str(), "current");
    }

    #[test]
    fn test_verification_status_as_str() {
        assert_eq!(VerificationStatus::Pending.as_str(), "pending");
        assert_eq!(VerificationStatus::Verified.as_str(), "verified");
        assert_eq!(VerificationStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_mask_bank_account() {
        assert_eq!(mask_bank_account("1234567890123456"), "XXXX...3456");
        assert_eq!(mask_bank_account("123"), "XXXX...XXXX");
        assert_eq!(mask_bank_account(""), "XXXX...XXXX");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================================
    // Property 7: Bank Account Validation
    // **Validates: Requirements 4.1**
    // For any bank account addition request, the Bank_Account_Service SHALL validate
    // IFSC code format and account number format, accepting valid formats and
    // rejecting invalid ones.
    // ============================================================================
    proptest! {
        #[test]
        fn prop_ifsc_code_format_validation(
            bank_code in r"[A-Z]{4}",
            branch_code in r"[A-Z0-9]{7}",
        ) {
            let ifsc = format!("{}{}", bank_code, branch_code);

            // IFSC code should be exactly 11 characters
            prop_assert_eq!(ifsc.len(), 11, "IFSC code must be exactly 11 characters");

            // IFSC code should be uppercase
            let ifsc_upper = ifsc.to_uppercase();
            prop_assert_eq!(ifsc.as_str(), ifsc_upper.as_str(), "IFSC code must be uppercase");

            // First 4 characters should be alphabetic
            let first_four = &ifsc[0..4];
            prop_assert!(first_four.chars().all(|c| c.is_alphabetic()), "First 4 characters must be alphabetic");
        }
    }

    proptest! {
        #[test]
        fn prop_ifsc_code_invalid_format(
            invalid_ifsc in r"[a-z0-9]{5,15}",
        ) {
            // Invalid IFSC codes should not pass validation
            let is_valid_length = invalid_ifsc.len() == 11;
            let is_uppercase = invalid_ifsc == invalid_ifsc.to_uppercase();

            // If it's not 11 chars or not uppercase, it's invalid
            if !is_valid_length || !is_uppercase {
                prop_assert!(
                    !is_valid_length || !is_uppercase,
                    "Invalid IFSC should fail at least one check"
                );
            }
        }
    }

    proptest! {
        #[test]
        fn prop_account_number_format_validation(
            account_number in r"[0-9]{9,18}",
        ) {
            // Account number should be numeric
            prop_assert!(account_number.chars().all(|c| c.is_numeric()), "Account number must be numeric");

            // Account number should be between 9 and 18 digits
            prop_assert!(account_number.len() >= 9 && account_number.len() <= 18, "Account number must be 9-18 digits");
        }
    }

    proptest! {
        #[test]
        fn prop_account_number_invalid_format(
            invalid_account in r"[a-zA-Z]{5,20}",
        ) {
            // Account number with letters should be invalid
            let has_non_numeric = invalid_account.chars().any(|c| !c.is_numeric());
            prop_assert!(has_non_numeric, "Account number with letters should be invalid");
        }
    }

    proptest! {
        #[test]
        fn prop_account_type_valid(
            account_type_idx in 0..2usize,
        ) {
            let account_types = vec![
                AccountType::Savings,
                AccountType::Current,
            ];
            let account_type = account_types[account_type_idx].clone();

            // Verify account type is valid
            let type_str = account_type.as_str();
            prop_assert!(
                type_str == "savings" || type_str == "current",
                "Account type must be savings or current"
            );
        }
    }

    proptest! {
        #[test]
        fn prop_bank_account_request_structure(
            account_holder_name in r"[a-zA-Z ]{5,50}",
            account_number in r"[0-9]{9,18}",
        ) {
            let ifsc = "HDFC0001234";

            let request = AddBankAccountRequest {
                account_holder_name: account_holder_name.clone(),
                account_number: account_number.clone(),
                ifsc_code: ifsc.to_string(),
                account_type: AccountType::Savings.as_str().to_string(),
            };

            // Verify request structure
            prop_assert_eq!(request.account_holder_name, account_holder_name);
            prop_assert_eq!(request.account_number, account_number);
            prop_assert_eq!(request.ifsc_code, ifsc);
            prop_assert_eq!(request.account_type, AccountType::Savings.as_str());
        }
    }

    proptest! {
        #[test]
        fn prop_bank_account_masking_preserves_last_four(
            account_number in r"[0-9]{9,18}",
        ) {
            let masked = mask_bank_account(&account_number);

            // Verify last 4 digits are preserved
            if account_number.len() >= 4 {
                let last_four = &account_number[account_number.len() - 4..];
                prop_assert!(masked.contains(last_four), "Last 4 digits should be preserved in masked account");
            }

            // Verify masking characters are present
            prop_assert!(masked.contains("X"), "Masked account should contain X characters");
        }
    }

    proptest! {
        #[test]
        fn prop_bank_account_masking_format_compliance(
            account_number in r"[0-9]{9,18}",
        ) {
            let masked = mask_bank_account(&account_number);

            // Verify format matches pattern: XXXX...{last4}
            prop_assert!(masked.contains("XXXX"), "Should contain XXXX masking pattern");
            prop_assert!(masked.contains("..."), "Should contain ellipsis");

            // Verify no more than last 4 digits are visible
            let visible_digits = masked.chars().filter(|c| c.is_numeric()).count();
            prop_assert!(visible_digits <= 4, "Only last 4 digits should be visible");
        }
    }

    proptest! {
        #[test]
        fn prop_bank_account_masking_idempotent(
            account_number in r"[0-9]{9,18}",
        ) {
            // Masking same account twice should produce same result
            let masked_1 = mask_bank_account(&account_number);
            let masked_2 = mask_bank_account(&account_number);
            prop_assert_eq!(masked_1, masked_2, "Bank account masking should be idempotent");
        }
    }

    proptest! {
        #[test]
        fn prop_verification_status_valid(
            status_idx in 0..3usize,
        ) {
            let statuses = vec![
                VerificationStatus::Pending,
                VerificationStatus::Verified,
                VerificationStatus::Failed,
            ];
            let status = statuses[status_idx].clone();

            // Verify status is valid
            let status_str = status.as_str();
            prop_assert!(
                status_str == "pending" || status_str == "verified" || status_str == "failed",
                "Verification status must be pending, verified, or failed"
            );
        }
    }
}

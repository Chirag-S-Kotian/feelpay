use sqlx::PgPool;
use uuid::Uuid;
use crate::error::FeelPayError;
use crate::models::ledger::{LedgerAccount, LedgerEntry};

pub struct LedgerService {
    db: PgPool,
}

impl LedgerService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_account(&self, owner_id: Uuid, account_type: &str) -> Result<LedgerAccount, FeelPayError> {
        sqlx::query_as::<_, LedgerAccount>(
            r#"
            SELECT id, name, account_type, owner_id, currency, balance, is_active, created_at
            FROM ledger_accounts
            WHERE owner_id = $1 AND account_type = $2
            "#
        )
        .bind(owner_id)
        .bind(account_type)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))
    }

    pub async fn get_or_create_account(&self, owner_id: Uuid, account_type: &str, name: &str) -> Result<LedgerAccount, FeelPayError> {
        let existing = self.get_account(owner_id, account_type).await;
        
        if let Ok(account) = existing {
            return Ok(account);
        }

        sqlx::query_as::<_, LedgerAccount>(
            r#"
            INSERT INTO ledger_accounts (name, account_type, owner_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, account_type, owner_id, currency, balance, is_active, created_at
            "#,
        )
        .bind(name)
        .bind(account_type)
        .bind(owner_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))
    }

    pub async fn record_transaction(
        &self,
        transaction_id: Uuid,
        merchant_id: Uuid,
        amount: i64,
        fee_amount: i64,
        tx_type: &str,
    ) -> Result<(), FeelPayError> {
        let mut tx = self.db.begin().await.map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

        // Debit merchant ledger (money owed TO merchant from escrow)
        let merchant_account = sqlx::query_as::<_, LedgerAccount>(
            r#"
            SELECT id, name, account_type, owner_id, currency, balance, is_active, created_at
            FROM ledger_accounts
            WHERE owner_id = $1 AND account_type = 'merchant'
            "#
        )
        .bind(merchant_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

        if merchant_account.is_none() {
            sqlx::query(
                "INSERT INTO ledger_accounts (name, account_type, owner_id) VALUES ($1, 'merchant', $2)"
            )
            .bind(format!("Merchant {}", merchant_id))
            .bind(merchant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;
        }

        // Create debit entry (merchant account)
        sqlx::query(
            r#"
            INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description)
            SELECT $1, id, 'debit', $2, $3
            FROM ledger_accounts WHERE owner_id = $4 AND account_type = 'merchant'
            "#,
        )
        .bind(transaction_id)
        .bind(amount)
        .bind(format!("{} payment", tx_type))
        .bind(merchant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

        // Create credit entry (platform fee)
        if fee_amount > 0 {
            sqlx::query(
                r#"
                INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description)
                SELECT $1, id, 'credit', $2, 'platform_fee'
                FROM ledger_accounts WHERE account_type = 'platform'
                "#,
            )
            .bind(transaction_id)
            .bind(fee_amount)
            .execute(&mut *tx)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;
        }

        tx.commit().await.map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_entries(&self, account_id: Uuid) -> Result<Vec<LedgerEntry>, FeelPayError> {
        sqlx::query_as::<_, LedgerEntry>(
            r#"
            SELECT id, transaction_id, account_id, entry_type, amount, currency, description, created_at
            FROM ledger_entries
            WHERE account_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(account_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))
    }
}

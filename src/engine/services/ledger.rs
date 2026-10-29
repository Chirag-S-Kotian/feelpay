use crate::error::FeelPayError;
use crate::models::ledger::{LedgerAccount, LedgerEntry};
use sqlx::PgPool;
use uuid::Uuid;

pub struct LedgerService {
    db: PgPool,
}

impl LedgerService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_account(
        &self,
        owner_id: Uuid,
        account_type: &str,
    ) -> Result<LedgerAccount, FeelPayError> {
        sqlx::query_as::<_, LedgerAccount>(
            r#"
            SELECT id, name, account_type, owner_id, currency, balance, is_active, created_at
            FROM ledger_accounts
            WHERE owner_id = $1 AND account_type = $2
            "#,
        )
        .bind(owner_id)
        .bind(account_type)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))
    }

    pub async fn get_or_create_account(
        &self,
        owner_id: Uuid,
        account_type: &str,
        name: &str,
    ) -> Result<LedgerAccount, FeelPayError> {
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
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))
    }

    pub async fn record_transaction(
        &self,
        transaction_id: Uuid,
        merchant_id: Uuid,
        amount: i64,
        fee_amount: i64,
        tx_type: &str,
    ) -> Result<(), FeelPayError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        // 1. Get or Create Merchant Account
        let merchant_account = self
            .get_or_create_account(
                merchant_id,
                "merchant",
                &format!("Merchant {}", merchant_id),
            )
            .await?;

        // 2. Get Platform Fee Account
        let platform_account = sqlx::query_as::<_, LedgerAccount>(
            "SELECT * FROM ledger_accounts WHERE account_type = 'platform' LIMIT 1",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| FeelPayError::Internal("Platform account not configured".into()))?;

        // 3. Get Escrow Account
        let escrow_account = sqlx::query_as::<_, LedgerAccount>(
            "SELECT * FROM ledger_accounts WHERE account_type = 'escrow' LIMIT 1",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| FeelPayError::Internal("Escrow account not configured".into()))?;

        // Reversal (refund): callers pass a negative amount. Entries must stay positive,
        // so post the opposite direction: debit merchant, credit escrow.
        if amount < 0 {
            let reversal = -amount;

            sqlx::query("UPDATE ledger_accounts SET balance = balance - $1 WHERE id = $2")
                .bind(reversal)
                .bind(merchant_account.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            sqlx::query("INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description) VALUES ($1, $2, 'debit', $3, $4)")
                .bind(transaction_id)
                .bind(merchant_account.id)
                .bind(reversal)
                .bind(format!("Debit for {}", tx_type))
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            sqlx::query("UPDATE ledger_accounts SET balance = balance + $1 WHERE id = $2")
                .bind(reversal)
                .bind(escrow_account.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            sqlx::query("INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description) VALUES ($1, $2, 'credit', $3, $4)")
                .bind(transaction_id)
                .bind(escrow_account.id)
                .bind(reversal)
                .bind(format!("Return to escrow for {}", tx_type))
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            tx.commit()
                .await
                .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
            return Ok(());
        }

        let net_amount = amount - fee_amount;

        // --- DOUBLE ENTRY ---

        // A. Debit Escrow (Escrow balance decreases as we payout)
        sqlx::query("UPDATE ledger_accounts SET balance = balance - $1 WHERE id = $2")
            .bind(amount)
            .bind(escrow_account.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        sqlx::query("INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description) VALUES ($1, $2, 'debit', $3, $4)")
            .bind(transaction_id)
            .bind(escrow_account.id)
            .bind(amount)
            .bind(format!("Payout from escrow for {}", tx_type))
            .execute(&mut *tx)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // B. Credit Merchant (Merchant balance increases)
        sqlx::query("UPDATE ledger_accounts SET balance = balance + $1 WHERE id = $2")
            .bind(net_amount)
            .bind(merchant_account.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        sqlx::query("INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description) VALUES ($1, $2, 'credit', $3, $4)")
            .bind(transaction_id)
            .bind(merchant_account.id)
            .bind(net_amount)
            .bind(format!("Credit for {}", tx_type))
            .execute(&mut *tx)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

        // C. Credit Platform (Fee income)
        if fee_amount > 0 {
            sqlx::query("UPDATE ledger_accounts SET balance = balance + $1 WHERE id = $2")
                .bind(fee_amount)
                .bind(platform_account.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

            sqlx::query("INSERT INTO ledger_entries (transaction_id, account_id, entry_type, amount, description) VALUES ($1, $2, 'credit', $3, 'Platform fee')")
                .bind(transaction_id)
                .bind(platform_account.id)
                .bind(fee_amount)
                .execute(&mut *tx)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

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
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))
    }
}

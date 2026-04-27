use crate::error::FeelPayError;
use crate::models::settlement::Settlement;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SettlementService {
    db: PgPool,
}

impl SettlementService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_settlement(
        &self,
        merchant_id: Uuid,
        settlement_date: chrono::NaiveDate,
    ) -> Result<Settlement, FeelPayError> {
        let total_amount: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(po.amount), 0)
            FROM payment_orders po
            WHERE po.merchant_id = $1 
            AND po.status = 'captured'
            AND NOT EXISTS (
                SELECT 1
                FROM settlement_items si
                JOIN settlements s ON s.id = si.settlement_id
                WHERE si.transaction_id = po.id
                AND s.status IN ('pending', 'processing', 'completed')
            )
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let fee_amount: i64 = (total_amount.0 * 2) / 100;
        let tax_amount = (fee_amount * 18) / 100;
        let net_amount = total_amount.0 - fee_amount - tax_amount;

        let settlement = sqlx::query_as::<_, Settlement>(
            r#"
            INSERT INTO settlements (merchant_id, settlement_date, amount, fee_amount, net_amount)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id, merchant_id, settlement_date AS "settlement_date!", status, 
                amount, fee_amount, net_amount, utr_number AS "utr_number?", 
                transaction_count, processed_at AS "processed_at?", created_at
            "#,
        )
        .bind(merchant_id)
        .bind(settlement_date)
        .bind(total_amount.0)
        .bind(fee_amount)
        .bind(net_amount)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        Ok(settlement)
    }

    pub async fn get_settlements(
        &self,
        merchant_id: Uuid,
    ) -> Result<Vec<Settlement>, FeelPayError> {
        let settlements = sqlx::query_as::<_, Settlement>(
            r#"
            SELECT 
                id, merchant_id, settlement_date AS "settlement_date!", status, 
                amount, fee_amount, net_amount, utr_number AS "utr_number?", 
                transaction_count, processed_at AS "processed_at?", created_at
            FROM settlements 
            WHERE merchant_id = $1 
            ORDER BY settlement_date DESC
            "#,
        )
        .bind(merchant_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
        Ok(settlements)
    }

    pub async fn get_settlement(&self, id: Uuid) -> Result<Option<Settlement>, FeelPayError> {
        let settlement = sqlx::query_as::<_, Settlement>(
            r#"
            SELECT 
                id, merchant_id, settlement_date AS "settlement_date!", status, 
                amount, fee_amount, net_amount, utr_number AS "utr_number?", 
                transaction_count, processed_at AS "processed_at?", created_at
            FROM settlements 
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
        Ok(settlement)
    }

    pub async fn get_balance(&self, merchant_id: Uuid) -> Result<(i64, i64), FeelPayError> {
        let available: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(po.amount), 0)::BIGINT
            FROM payment_orders po
            WHERE po.merchant_id = $1 AND po.status = 'captured'
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let pending: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(po.amount), 0)::BIGINT
            FROM payment_orders po
            WHERE po.merchant_id = $1 AND po.status = 'authorized'
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        Ok((available.0 - pending.0, pending.0))
    }
}

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
        environment: &str,
    ) -> Result<Settlement, FeelPayError> {
        let total_amount: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(po.amount), 0)::BIGINT
            FROM payment_orders po
            WHERE po.merchant_id = $1 
            AND po.status = 'captured'
            AND COALESCE(po.metadata->>'environment', 'test') = $2::TEXT
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
        .bind(environment)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let fee_amount: i64 = (total_amount.0 * 2) / 100;
        let tax_amount = (fee_amount * 18) / 100;
        let net_amount = total_amount.0 - fee_amount - tax_amount;

        let settlement = sqlx::query_as::<_, Settlement>(
            r#"
            INSERT INTO settlements (merchant_id, settlement_date, amount, fee_amount, net_amount, environment)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, merchant_id, settlement_date AS "settlement_date!", status, 
                amount, fee_amount, net_amount, utr_number AS "utr_number?", 
                transaction_count, processed_at AS "processed_at?", created_at, environment
            "#,
        )
        .bind(merchant_id)
        .bind(settlement_date)
        .bind(total_amount.0)
        .bind(fee_amount)
        .bind(net_amount)
        .bind(environment)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        Ok(settlement)
    }

    pub async fn get_settlements(
        &self,
        merchant_id: Uuid,
        environment: &str,
    ) -> Result<Vec<Settlement>, FeelPayError> {
        let settlements = sqlx::query_as::<_, Settlement>(
            r#"
            SELECT 
                id, merchant_id, settlement_date AS "settlement_date!", status, 
                amount, fee_amount, net_amount, utr_number AS "utr_number?", 
                transaction_count, processed_at AS "processed_at?", created_at, environment
            FROM settlements 
            WHERE merchant_id = $1 AND environment = $2::TEXT
            ORDER BY settlement_date DESC
            "#,
        )
        .bind(merchant_id)
        .bind(environment)
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
                transaction_count, processed_at AS "processed_at?", created_at, environment
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

    pub async fn get_balance(
        &self,
        merchant_id: Uuid,
        environment: &str,
    ) -> Result<(i64, i64, i64, i64), FeelPayError> {
        let captured: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(po.amount), 0)::BIGINT
            FROM payment_orders po
            WHERE po.merchant_id = $1 AND po.status = 'captured'
              AND COALESCE(po.metadata->>'environment', 'test') = $2::TEXT
            "#,
        )
        .bind(merchant_id)
        .bind(environment)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let pending: (i64,) = sqlx::query_as(
            r#"
            SELECT
                (COALESCE((SELECT SUM(po.amount) FROM payment_orders po WHERE po.merchant_id = $1 AND po.status = 'authorized' AND COALESCE(po.metadata->>'environment', 'test') = $2::TEXT), 0)
              + COALESCE((SELECT SUM(s.net_amount) FROM settlements s WHERE s.merchant_id = $1 AND s.status IN ('scheduled','pending','processing','on_hold') AND s.environment = $2::TEXT), 0))::BIGINT
            "#,
        )
        .bind(merchant_id)
        .bind(environment)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let deductions: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE((SELECT SUM(r.amount) FROM refunds r JOIN payment_orders po ON po.id = r.payment_order_id WHERE r.merchant_id = $1 AND r.status IN ('requested','processing','processed','succeeded') AND COALESCE(po.metadata->>'environment', 'test') = $2::TEXT), 0)::BIGINT AS refunds,
                COALESCE((SELECT SUM(d.amount) FROM disputes d JOIN payment_orders po ON po.id = d.payment_order_id WHERE d.merchant_id = $1 AND d.status IN ('needs_response','under_review','accepted','lost') AND COALESCE(po.metadata->>'environment', 'test') = $2::TEXT), 0)::BIGINT AS holds,
                COALESCE((SELECT SUM(rb.amount) FROM reserve_balances rb WHERE rb.business_account_id = $1 AND rb.status = 'held'), 0)::BIGINT AS reserves,
                COALESCE((SELECT SUM(pc.amount) FROM payouts_company pc WHERE pc.business_account_id = $1 AND pc.status IN ('queued','approved','processing','paid')), 0)::BIGINT AS payout_commitments
            "#,
        )
        .bind(merchant_id)
        .bind(environment)
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

        let available =
            (captured.0 - pending.0 - deductions.0 - deductions.1 - deductions.2 - deductions.3)
                .max(0);
        Ok((available, pending.0, deductions.1, deductions.2))
    }
}

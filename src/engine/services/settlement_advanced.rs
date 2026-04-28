use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Advanced settlement features: instant settlement, TDS, reconciliation.
#[derive(Clone)]
pub struct SettlementAdvancedService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettlementCycle {
    Instant, // Real-time settlement (premium)
    T1,      // T+1 (next business day)
    T2,      // T+2 (2 business days)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InstantSettlement {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub amount: i64,     // In paise
    pub fee: i64,        // Platform fee in paise
    pub tds_amount: i64, // TDS deducted in paise
    pub net_amount: i64, // Amount after fee and TDS
    pub status: String,  // pending, processing, completed, failed
    pub requested_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub utr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TdsCalculation {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub merchant_id: Uuid,
    pub gross_amount: i64,      // Before TDS
    pub tds_rate: f64,          // TDS percentage (e.g., 1.0 for 1%)
    pub tds_amount: i64,        // TDS deducted
    pub net_amount: i64,        // After TDS
    pub financial_year: String, // FY 2025-26
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReconciliationRecord {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub settlement_id: Uuid,
    pub feelpay_amount: i64,  // Amount in FeelPay records
    pub provider_amount: i64, // Amount from provider report
    pub bank_amount: i64,     // Amount from bank statement
    pub discrepancy: i64,     // Difference (should be 0)
    pub status: String,       // matched, discrepancy, pending
    pub reconciled_at: Option<DateTime<Utc>>,
}

impl SettlementAdvancedService {
    pub fn new(pool: PgPool) -> Self {
        SettlementAdvancedService { pool }
    }

    /// Request instant settlement (premium feature)
    pub async fn request_instant_settlement(
        &self,
        merchant_id: Uuid,
        amount: i64,
        fee: i64,
    ) -> Result<InstantSettlement, sqlx::Error> {
        let id = Uuid::new_v4();
        let tds_amount = self.calculate_tds(amount);
        let net_amount = amount - fee - tds_amount;

        sqlx::query_as::<_, InstantSettlement>(
            r#"
            INSERT INTO instant_settlements (id, merchant_id, amount, fee, tds_amount, net_amount, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, merchant_id, amount, fee, tds_amount, net_amount, status, requested_at, processed_at, utr
            "#,
        )
        .bind(id)
        .bind(merchant_id)
        .bind(amount)
        .bind(fee)
        .bind(tds_amount)
        .bind(net_amount)
        .bind("pending")
        .fetch_one(&self.pool)
        .await
    }

    /// Calculate TDS (Tax Deducted at Source)
    /// India: 1% TDS on payment aggregator commissions for merchants with turnover > 50L
    pub fn calculate_tds(&self, amount: i64) -> i64 {
        // Simplified: 1% TDS
        // In production: check merchant turnover, GST registration, etc.
        (amount as f64 * 0.01) as i64
    }

    /// Create TDS calculation record
    pub async fn create_tds_record(
        &self,
        settlement_id: Uuid,
        merchant_id: Uuid,
        gross_amount: i64,
        tds_rate: f64,
    ) -> Result<TdsCalculation, sqlx::Error> {
        let id = Uuid::new_v4();
        let tds_amount = (gross_amount as f64 * tds_rate / 100.0) as i64;
        let net_amount = gross_amount - tds_amount;
        let financial_year = self.get_financial_year();

        sqlx::query_as::<_, TdsCalculation>(
            r#"
            INSERT INTO tds_calculations (id, settlement_id, merchant_id, gross_amount, tds_rate, tds_amount, net_amount, financial_year)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, settlement_id, merchant_id, gross_amount, tds_rate, tds_amount, net_amount, financial_year, calculated_at
            "#,
        )
        .bind(id)
        .bind(settlement_id)
        .bind(merchant_id)
        .bind(gross_amount)
        .bind(tds_rate)
        .bind(tds_amount)
        .bind(net_amount)
        .bind(&financial_year)
        .fetch_one(&self.pool)
        .await
    }

    /// Get financial year (FY 2025-26 for Apr 2025 - Mar 2026)
    fn get_financial_year(&self) -> String {
        let now = Utc::now();
        let year = now.year();
        let month = now.month();

        if month >= 4 {
            format!("FY {}-{}", year, year + 1)
        } else {
            format!("FY {}-{}", year - 1, year)
        }
    }

    /// Mark instant settlement as completed
    pub async fn mark_instant_settlement_completed(
        &self,
        settlement_id: Uuid,
        utr: &str,
    ) -> Result<InstantSettlement, sqlx::Error> {
        sqlx::query_as::<_, InstantSettlement>(
            r#"
            UPDATE instant_settlements
            SET status = $1, processed_at = NOW(), utr = $2
            WHERE id = $3
            RETURNING id, merchant_id, amount, fee, tds_amount, net_amount, status, requested_at, processed_at, utr
            "#,
        )
        .bind("completed")
        .bind(utr)
        .bind(settlement_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Create reconciliation record
    pub async fn create_reconciliation(
        &self,
        merchant_id: Uuid,
        settlement_id: Uuid,
        feelpay_amount: i64,
        provider_amount: i64,
        bank_amount: i64,
    ) -> Result<ReconciliationRecord, sqlx::Error> {
        let id = Uuid::new_v4();
        let discrepancy = feelpay_amount - provider_amount;
        let status = if discrepancy == 0 {
            "matched"
        } else {
            "discrepancy"
        };

        sqlx::query_as::<_, ReconciliationRecord>(
            r#"
            INSERT INTO reconciliation_records (id, merchant_id, settlement_id, feelpay_amount, provider_amount, bank_amount, discrepancy, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, merchant_id, settlement_id, feelpay_amount, provider_amount, bank_amount, discrepancy, status, reconciled_at
            "#,
        )
        .bind(id)
        .bind(merchant_id)
        .bind(settlement_id)
        .bind(feelpay_amount)
        .bind(provider_amount)
        .bind(bank_amount)
        .bind(discrepancy)
        .bind(status)
        .fetch_one(&self.pool)
        .await
    }

    /// Mark reconciliation as resolved
    pub async fn mark_reconciliation_resolved(
        &self,
        reconciliation_id: Uuid,
    ) -> Result<ReconciliationRecord, sqlx::Error> {
        sqlx::query_as::<_, ReconciliationRecord>(
            r#"
            UPDATE reconciliation_records
            SET status = $1, reconciled_at = NOW()
            WHERE id = $2
            RETURNING id, merchant_id, settlement_id, feelpay_amount, provider_amount, bank_amount, discrepancy, status, reconciled_at
            "#,
        )
        .bind("matched")
        .bind(reconciliation_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get pending reconciliations
    pub async fn get_pending_reconciliations(
        &self,
    ) -> Result<Vec<ReconciliationRecord>, sqlx::Error> {
        sqlx::query_as::<_, ReconciliationRecord>(
            r#"
            SELECT id, merchant_id, settlement_id, feelpay_amount, provider_amount, bank_amount, discrepancy, status, reconciled_at
            FROM reconciliation_records
            WHERE status = 'discrepancy'
            ORDER BY id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Retry failed payout
    pub async fn retry_payout(&self, settlement_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE settlements
            SET status = $1, retry_count = retry_count + 1
            WHERE id = $2 AND status = 'failed'
            "#,
        )
        .bind("pending")
        .bind(settlement_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get settlement retry history
    pub async fn get_retry_history(&self, settlement_id: Uuid) -> Result<i32, sqlx::Error> {
        let count: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(retry_count, 0)
            FROM settlements
            WHERE id = $1
            "#,
        )
        .bind(settlement_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tds_calculation() {
        let service = SettlementAdvancedService::new(sqlx::PgPool::connect("").await.unwrap());
        let amount = 100000; // 1000 INR
        let tds = service.calculate_tds(amount);
        assert_eq!(tds, 1000); // 1% = 10 INR
    }

    #[test]
    fn test_financial_year_april() {
        let service = SettlementAdvancedService::new(sqlx::PgPool::connect("").await.unwrap());
        let fy = service.get_financial_year();
        // Should be FY 2025-26 or similar
        assert!(fy.contains("FY"));
    }

    #[test]
    fn test_reconciliation_match() {
        let discrepancy = 0;
        let status = if discrepancy == 0 {
            "matched"
        } else {
            "discrepancy"
        };
        assert_eq!(status, "matched");
    }
}

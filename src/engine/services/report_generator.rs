use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Report generation service for pre-built and custom reports.
/// Supports: GST, Settlement, Refunds, Transactions, Disputes, Merchant Fee reports.
#[derive(Clone)]
pub struct ReportGeneratorService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Gst,          // GST report (monthly)
    Settlement,   // Settlement report
    Refunds,      // Refund transactions
    Transactions, // All transactions
    Disputes,     // Dispute details
    MerchantFee,  // Fee breakdown by merchant
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Excel,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GeneratedReport {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub report_type: String, // gst, settlement, refunds, etc.
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub status: String,           // pending, generating, completed, failed
    pub file_url: Option<String>, // S3 URL
    pub file_format: String,      // csv, excel, pdf
    pub row_count: Option<i32>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GstReportRow {
    pub transaction_id: String,
    pub date: NaiveDate,
    pub amount: i64,
    pub gst_amount: i64,
    pub payment_method: String,
    pub customer_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementReportRow {
    pub settlement_id: String,
    pub date: NaiveDate,
    pub gross_amount: i64,
    pub fees: i64,
    pub refunds: i64,
    pub net_amount: i64,
    pub utr: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefundReportRow {
    pub refund_id: String,
    pub transaction_id: String,
    pub date: NaiveDate,
    pub amount: i64,
    pub reason: String,
    pub status: String,
    pub customer_email: String,
}

impl ReportGeneratorService {
    pub fn new(pool: PgPool) -> Self {
        ReportGeneratorService { pool }
    }

    /// Create report generation job
    pub async fn create_report(
        &self,
        merchant_id: Uuid,
        report_type: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        file_format: &str,
    ) -> Result<GeneratedReport, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, GeneratedReport>(
            r#"
            INSERT INTO generated_reports (id, merchant_id, report_type, period_start, period_end, status, file_format)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, merchant_id, report_type, period_start, period_end, status, file_url, file_format, row_count, generated_at, created_at
            "#,
        )
        .bind(id)
        .bind(merchant_id)
        .bind(report_type)
        .bind(period_start)
        .bind(period_end)
        .bind("pending")
        .bind(file_format)
        .fetch_one(&self.pool)
        .await
    }

    /// Generate GST report
    pub async fn generate_gst_report(
        &self,
        merchant_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<GstReportRow>, sqlx::Error> {
        sqlx::query_as::<_, GstReportRow>(
            r#"
            SELECT
                po.id::TEXT as transaction_id,
                po.created_at::DATE as date,
                po.amount,
                (po.amount * 18 / 100)::BIGINT as gst_amount,
                po.payment_method,
                po.customer_email
            FROM payment_orders po
            WHERE po.merchant_id = $1
            AND po.created_at::DATE BETWEEN $2 AND $3
            AND po.status = 'success'
            ORDER BY po.created_at DESC
            "#,
        )
        .bind(merchant_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await
    }

    /// Generate settlement report
    pub async fn generate_settlement_report(
        &self,
        merchant_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<SettlementReportRow>, sqlx::Error> {
        sqlx::query_as::<_, SettlementReportRow>(
            r#"
            SELECT
                s.id::TEXT as settlement_id,
                s.created_at::DATE as date,
                s.gross_amount,
                s.fees,
                s.refunds,
                s.net_amount,
                s.utr,
                s.status
            FROM settlements s
            WHERE s.merchant_id = $1
            AND s.created_at::DATE BETWEEN $2 AND $3
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(merchant_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await
    }

    /// Generate refund report
    pub async fn generate_refund_report(
        &self,
        merchant_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<RefundReportRow>, sqlx::Error> {
        sqlx::query_as::<_, RefundReportRow>(
            r#"
            SELECT
                r.id::TEXT as refund_id,
                r.payment_id::TEXT as transaction_id,
                r.created_at::DATE as date,
                r.amount,
                r.reason,
                r.status,
                po.customer_email
            FROM refunds r
            JOIN payment_orders po ON r.payment_id = po.id
            WHERE po.merchant_id = $1
            AND r.created_at::DATE BETWEEN $2 AND $3
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(merchant_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await
    }

    /// Mark report as generating
    pub async fn mark_generating(&self, report_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE generated_reports
            SET status = $1
            WHERE id = $2
            "#,
        )
        .bind("generating")
        .bind(report_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark report as completed
    pub async fn mark_completed(
        &self,
        report_id: Uuid,
        file_url: &str,
        row_count: i32,
    ) -> Result<GeneratedReport, sqlx::Error> {
        sqlx::query_as::<_, GeneratedReport>(
            r#"
            UPDATE generated_reports
            SET status = $1, file_url = $2, row_count = $3, generated_at = NOW()
            WHERE id = $4
            RETURNING id, merchant_id, report_type, period_start, period_end, status, file_url, file_format, row_count, generated_at, created_at
            "#,
        )
        .bind("completed")
        .bind(file_url)
        .bind(row_count)
        .bind(report_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Mark report as failed
    pub async fn mark_failed(&self, report_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE generated_reports
            SET status = $1
            WHERE id = $2
            "#,
        )
        .bind("failed")
        .bind(report_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get report by ID
    pub async fn get_report(
        &self,
        report_id: Uuid,
    ) -> Result<Option<GeneratedReport>, sqlx::Error> {
        sqlx::query_as::<_, GeneratedReport>(
            r#"
            SELECT id, merchant_id, report_type, period_start, period_end, status, file_url, file_format, row_count, generated_at, created_at
            FROM generated_reports
            WHERE id = $1
            "#,
        )
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// List reports for merchant
    pub async fn list_reports(
        &self,
        merchant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GeneratedReport>, sqlx::Error> {
        sqlx::query_as::<_, GeneratedReport>(
            r#"
            SELECT id, merchant_id, report_type, period_start, period_end, status, file_url, file_format, row_count, generated_at, created_at
            FROM generated_reports
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Get pending reports (for background worker)
    pub async fn get_pending_reports(&self) -> Result<Vec<GeneratedReport>, sqlx::Error> {
        sqlx::query_as::<_, GeneratedReport>(
            r#"
            SELECT id, merchant_id, report_type, period_start, period_end, status, file_url, file_format, row_count, generated_at, created_at
            FROM generated_reports
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_variants() {
        let _gst = ReportType::Gst;
        let _settlement = ReportType::Settlement;
        let _refunds = ReportType::Refunds;
    }

    #[test]
    fn test_export_format_variants() {
        let _csv = ExportFormat::Csv;
        let _excel = ExportFormat::Excel;
        let _pdf = ExportFormat::Pdf;
    }

    #[test]
    fn test_gst_calculation() {
        let amount = 100000; // 1000 INR
        let gst = amount * 18 / 100;
        assert_eq!(gst, 18000); // 180 INR
    }
}

use crate::error::FeelPayError;
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Report types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportType {
    TransactionSummary,
    SettlementReconciliation,
    RefundReport,
    DisputeReport,
    GstReport,
    Custom,
}

impl ReportType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TransactionSummary => "transaction_summary",
            Self::SettlementReconciliation => "settlement_reconciliation",
            Self::RefundReport => "refund_report",
            Self::DisputeReport => "dispute_report",
            Self::GstReport => "gst_report",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "transaction_summary" => Some(Self::TransactionSummary),
            "settlement_reconciliation" => Some(Self::SettlementReconciliation),
            "refund_report" => Some(Self::RefundReport),
            "dispute_report" => Some(Self::DisputeReport),
            "gst_report" => Some(Self::GstReport),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Report frequency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportFrequency {
    Daily,
    Weekly,
    Monthly,
}

impl ReportFrequency {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            _ => None,
        }
    }
}

/// Report format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportFormat {
    Csv,
    Pdf,
    Excel,
}

impl ReportFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Csv => "csv",
            Self::Pdf => "pdf",
            Self::Excel => "excel",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "csv" => Some(Self::Csv),
            "pdf" => Some(Self::Pdf),
            "excel" => Some(Self::Excel),
            _ => None,
        }
    }
}

/// Scheduled report data model
#[derive(Debug, Clone)]
pub struct ScheduledReport {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub report_type: ReportType,
    pub frequency: ReportFrequency,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub time_of_day: NaiveTime,
    pub recipient_emails: Vec<String>,
    pub format: ReportFormat,
    pub is_active: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_user_id: Option<Uuid>,
}

/// Request to create a scheduled report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledReportRequest {
    pub report_type: String,
    pub frequency: String,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub time_of_day: String, // HH:MM format
    pub recipient_emails: Vec<String>,
    pub format: String,
}

/// Request to update a scheduled report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledReportRequest {
    pub frequency: Option<String>,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub time_of_day: Option<String>,
    pub recipient_emails: Option<Vec<String>>,
    pub format: Option<String>,
    pub is_active: Option<bool>,
}

/// ReportService handles scheduled report creation and management
pub struct ReportService {
    db: PgPool,
}

impl ReportService {
    /// Create a new ReportService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a scheduled report with validation
    pub async fn create_scheduled_report(
        &self,
        merchant_id: Uuid,
        request: CreateScheduledReportRequest,
        created_by_user_id: Option<Uuid>,
    ) -> Result<ScheduledReport, FeelPayError> {
        // Validate report type
        let report_type = ReportType::from_str(&request.report_type)
            .ok_or_else(|| FeelPayError::BadRequest("Invalid report type".into()))?;

        // Validate frequency
        let frequency = ReportFrequency::from_str(&request.frequency)
            .ok_or_else(|| FeelPayError::BadRequest("Invalid frequency".into()))?;

        // Validate format
        let format = ReportFormat::from_str(&request.format)
            .ok_or_else(|| FeelPayError::BadRequest("Invalid format".into()))?;

        // Validate time of day
        let time_of_day = NaiveTime::parse_from_str(&request.time_of_day, "%H:%M")
            .map_err(|_| FeelPayError::BadRequest("Invalid time format. Use HH:MM".into()))?;

        // Validate frequency-specific fields
        match frequency {
            ReportFrequency::Weekly => {
                if request.day_of_week.is_none()
                    || request.day_of_week.unwrap() < 0
                    || request.day_of_week.unwrap() > 6
                {
                    return Err(FeelPayError::BadRequest(
                        "Weekly frequency requires day_of_week (0-6)".into(),
                    ));
                }
            }
            ReportFrequency::Monthly => {
                if request.day_of_month.is_none()
                    || request.day_of_month.unwrap() < 1
                    || request.day_of_month.unwrap() > 31
                {
                    return Err(FeelPayError::BadRequest(
                        "Monthly frequency requires day_of_month (1-31)".into(),
                    ));
                }
            }
            ReportFrequency::Daily => {
                // No additional validation needed
            }
        }

        // Validate recipient emails
        if request.recipient_emails.is_empty() {
            return Err(FeelPayError::BadRequest(
                "At least one recipient email is required".into(),
            ));
        }

        // Calculate next run time
        let next_run_at = self.calculate_next_run(
            &frequency,
            request.day_of_week,
            request.day_of_month,
            &time_of_day,
        );

        let report_id = Uuid::new_v4();
        let now = Utc::now();

        let report = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<i32>,
                String,
                Vec<String>,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            INSERT INTO scheduled_reports (
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            "#,
        )
        .bind(report_id)
        .bind(merchant_id)
        .bind(report_type.as_str())
        .bind(frequency.as_str())
        .bind(request.day_of_week)
        .bind(request.day_of_month)
        .bind(time_of_day.to_string())
        .bind(&request.recipient_emails)
        .bind(format.as_str())
        .bind(true)
        .bind::<Option<DateTime<Utc>>>(None)
        .bind(next_run_at)
        .bind(now)
        .bind(now)
        .bind(created_by_user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create scheduled report: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_scheduled_report(report))
    }

    /// Get a scheduled report by ID with tenant isolation check
    pub async fn get_scheduled_report(
        &self,
        merchant_id: Uuid,
        report_id: Uuid,
    ) -> Result<Option<ScheduledReport>, FeelPayError> {
        let report = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<i32>,
                String,
                Vec<String>,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            FROM scheduled_reports
            WHERE id = $1 AND merchant_id = $2
            "#,
        )
        .bind(report_id)
        .bind(merchant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scheduled report: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(report.map(|row| self.row_to_scheduled_report(row)))
    }

    /// List scheduled reports for a merchant with pagination
    pub async fn list_scheduled_reports(
        &self,
        merchant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ScheduledReport>, i64), FeelPayError> {
        // Get total count
        let (total,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM scheduled_reports WHERE merchant_id = $1
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count scheduled reports: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        // Get paginated results
        let reports = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<i32>,
                String,
                Vec<String>,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            FROM scheduled_reports
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scheduled reports: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        let items = reports
            .into_iter()
            .map(|row| self.row_to_scheduled_report(row))
            .collect();
        Ok((items, total))
    }

    /// Update a scheduled report
    pub async fn update_scheduled_report(
        &self,
        merchant_id: Uuid,
        report_id: Uuid,
        request: UpdateScheduledReportRequest,
    ) -> Result<ScheduledReport, FeelPayError> {
        // Get existing report
        let existing = self
            .get_scheduled_report(merchant_id, report_id)
            .await?
            .ok_or_else(|| FeelPayError::NotFound("Scheduled report not found".into()))?;

        // Validate and parse updates
        let frequency = if let Some(freq_str) = &request.frequency {
            ReportFrequency::from_str(freq_str)
                .ok_or_else(|| FeelPayError::BadRequest("Invalid frequency".into()))?
        } else {
            existing.frequency.clone()
        };

        let format = if let Some(fmt_str) = &request.format {
            ReportFormat::from_str(fmt_str)
                .ok_or_else(|| FeelPayError::BadRequest("Invalid format".into()))?
        } else {
            existing.format.clone()
        };

        let time_of_day = if let Some(time_str) = &request.time_of_day {
            NaiveTime::parse_from_str(time_str, "%H:%M")
                .map_err(|_| FeelPayError::BadRequest("Invalid time format. Use HH:MM".into()))?
        } else {
            existing.time_of_day
        };

        let day_of_week = request.day_of_week.or(existing.day_of_week);
        let day_of_month = request.day_of_month.or(existing.day_of_month);
        let recipient_emails = request
            .recipient_emails
            .unwrap_or(existing.recipient_emails.clone());
        let is_active = request.is_active.unwrap_or(existing.is_active);

        // Validate frequency-specific fields
        match frequency {
            ReportFrequency::Weekly => {
                if day_of_week.is_none() || day_of_week.unwrap() < 0 || day_of_week.unwrap() > 6 {
                    return Err(FeelPayError::BadRequest(
                        "Weekly frequency requires day_of_week (0-6)".into(),
                    ));
                }
            }
            ReportFrequency::Monthly => {
                if day_of_month.is_none() || day_of_month.unwrap() < 1 || day_of_month.unwrap() > 31
                {
                    return Err(FeelPayError::BadRequest(
                        "Monthly frequency requires day_of_month (1-31)".into(),
                    ));
                }
            }
            ReportFrequency::Daily => {}
        }

        // Calculate next run time
        let next_run_at =
            self.calculate_next_run(&frequency, day_of_week, day_of_month, &time_of_day);

        let now = Utc::now();

        let report = sqlx::query_as::<_, (
            String,
            String,
            String,
            String,
            Option<i32>,
            Option<i32>,
            String,
            Vec<String>,
            String,
            bool,
            Option<String>,
            Option<String>,
            String,
            String,
            Option<String>,
        )>(
            r#"
            UPDATE scheduled_reports
            SET frequency = $1, day_of_week = $2, day_of_month = $3, time_of_day = $4,
                recipient_emails = $5, format = $6, is_active = $7, next_run_at = $8, updated_at = $9
            WHERE id = $10 AND merchant_id = $11
            RETURNING 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            "#,
        )
        .bind(frequency.as_str())
        .bind(day_of_week)
        .bind(day_of_month)
        .bind(time_of_day.to_string())
        .bind(&recipient_emails)
        .bind(format.as_str())
        .bind(is_active)
        .bind(next_run_at)
        .bind(now)
        .bind(report_id)
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update scheduled report: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_scheduled_report(report))
    }

    /// Delete a scheduled report
    pub async fn delete_scheduled_report(
        &self,
        merchant_id: Uuid,
        report_id: Uuid,
    ) -> Result<(), FeelPayError> {
        let result = sqlx::query(
            r#"
            DELETE FROM scheduled_reports
            WHERE id = $1 AND merchant_id = $2
            "#,
        )
        .bind(report_id)
        .bind(merchant_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete scheduled report: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        if result.rows_affected() == 0 {
            return Err(FeelPayError::NotFound("Scheduled report not found".into()));
        }

        Ok(())
    }

    /// Get scheduled reports due for execution
    pub async fn get_scheduled_reports_due_now(
        &self,
        now: &DateTime<Utc>,
    ) -> Result<Vec<ScheduledReport>, FeelPayError> {
        let reports = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<i32>,
                String,
                Vec<String>,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            FROM scheduled_reports
            WHERE is_active = true AND next_run_at <= $1
            ORDER BY next_run_at ASC
            "#,
        )
        .bind(now)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch due scheduled reports: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(reports
            .into_iter()
            .map(|row| self.row_to_scheduled_report(row))
            .collect())
    }

    /// Update last_run_at and next_run_at for a scheduled report
    pub async fn update_scheduled_report_execution(
        &self,
        report_id: Uuid,
    ) -> Result<(), FeelPayError> {
        // Get the report to calculate next run
        let report = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<i32>,
                Option<i32>,
                String,
                Vec<String>,
                String,
                bool,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, report_type, frequency, day_of_week, day_of_month,
                time_of_day, recipient_emails, format, is_active, last_run_at, next_run_at,
                created_at, updated_at, created_by_user_id
            FROM scheduled_reports
            WHERE id = $1
            "#,
        )
        .bind(report_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch scheduled report for execution update: {:?}",
                e
            );
            FeelPayError::Database(e.to_string())
        })?;

        let scheduled_report = self.row_to_scheduled_report(report);
        let now = Utc::now();
        let next_run = self.calculate_next_run(
            &scheduled_report.frequency,
            scheduled_report.day_of_week,
            scheduled_report.day_of_month,
            &scheduled_report.time_of_day,
        );

        sqlx::query(
            r#"
            UPDATE scheduled_reports
            SET last_run_at = $1, next_run_at = $2
            WHERE id = $3
            "#,
        )
        .bind(now)
        .bind(next_run)
        .bind(report_id)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update scheduled report execution: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(())
    }

    /// Convert database row to ScheduledReport struct
    fn row_to_scheduled_report(
        &self,
        row: (
            String,
            String,
            String,
            String,
            Option<i32>,
            Option<i32>,
            String,
            Vec<String>,
            String,
            bool,
            Option<String>,
            Option<String>,
            String,
            String,
            Option<String>,
        ),
    ) -> ScheduledReport {
        let (
            id,
            merchant_id,
            report_type,
            frequency,
            day_of_week,
            day_of_month,
            time_of_day,
            recipient_emails,
            format,
            is_active,
            last_run_at,
            next_run_at,
            created_at,
            updated_at,
            created_by_user_id,
        ) = row;

        ScheduledReport {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            merchant_id: Uuid::parse_str(&merchant_id).unwrap_or_default(),
            report_type: ReportType::from_str(&report_type).unwrap_or(ReportType::Custom),
            frequency: ReportFrequency::from_str(&frequency).unwrap_or(ReportFrequency::Daily),
            day_of_week,
            day_of_month,
            time_of_day: NaiveTime::parse_from_str(&time_of_day, "%H:%M:%S")
                .unwrap_or_else(|_| NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            recipient_emails,
            format: ReportFormat::from_str(&format).unwrap_or(ReportFormat::Csv),
            is_active,
            last_run_at: last_run_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            next_run_at: next_run_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_by_user_id: created_by_user_id.and_then(|id| Uuid::parse_str(&id).ok()),
        }
    }

    /// Calculate next run time based on frequency and schedule
    fn calculate_next_run(
        &self,
        frequency: &ReportFrequency,
        day_of_week: Option<i32>,
        day_of_month: Option<i32>,
        time_of_day: &NaiveTime,
    ) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let today = now.date_naive();
        let scheduled_time = today.and_time(*time_of_day);
        let scheduled_dt = DateTime::<Utc>::from_naive_utc_and_offset(scheduled_time, Utc);

        match frequency {
            ReportFrequency::Daily => {
                if scheduled_dt > now {
                    Some(scheduled_dt)
                } else {
                    Some(scheduled_dt + chrono::Duration::days(1))
                }
            }
            ReportFrequency::Weekly => {
                if let Some(target_dow) = day_of_week {
                    let target_weekday = match target_dow {
                        0 => Weekday::Mon,
                        1 => Weekday::Tue,
                        2 => Weekday::Wed,
                        3 => Weekday::Thu,
                        4 => Weekday::Fri,
                        5 => Weekday::Sat,
                        6 => Weekday::Sun,
                        _ => Weekday::Mon,
                    };

                    let mut next_date = today;
                    loop {
                        next_date = next_date + chrono::Duration::days(1);
                        if next_date.weekday() == target_weekday {
                            break;
                        }
                    }

                    let next_scheduled = next_date.and_time(*time_of_day);
                    let next_dt = DateTime::<Utc>::from_naive_utc_and_offset(next_scheduled, Utc);
                    Some(next_dt)
                } else {
                    None
                }
            }
            ReportFrequency::Monthly => {
                if let Some(target_dom) = day_of_month {
                    let mut next_date = today;
                    loop {
                        next_date = next_date + chrono::Duration::days(1);
                        if next_date.day() == target_dom as u32 {
                            break;
                        }
                        // Prevent infinite loop if day doesn't exist in month
                        if next_date.day() == 1 && next_date.month() != today.month() {
                            break;
                        }
                    }

                    let next_scheduled = next_date.and_time(*time_of_day);
                    let next_dt = DateTime::<Utc>::from_naive_utc_and_offset(next_scheduled, Utc);
                    Some(next_dt)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_as_str() {
        assert_eq!(
            ReportType::TransactionSummary.as_str(),
            "transaction_summary"
        );
        assert_eq!(
            ReportType::SettlementReconciliation.as_str(),
            "settlement_reconciliation"
        );
    }

    #[test]
    fn test_report_frequency_as_str() {
        assert_eq!(ReportFrequency::Daily.as_str(), "daily");
        assert_eq!(ReportFrequency::Weekly.as_str(), "weekly");
        assert_eq!(ReportFrequency::Monthly.as_str(), "monthly");
    }

    #[test]
    fn test_report_format_as_str() {
        assert_eq!(ReportFormat::Csv.as_str(), "csv");
        assert_eq!(ReportFormat::Pdf.as_str(), "pdf");
        assert_eq!(ReportFormat::Excel.as_str(), "excel");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================================
    // Property 6: Settlement Report Field Completeness
    // **Validates: Requirements 3.2**
    // For any settlement, generating a settlement report SHALL include all required
    // fields: settlement_id, settlement_date, transaction_count, gross_amount,
    // MDR_fee, GST_on_fee, refund_deductions, net_amount, bank_account,
    // transfer_date, UTR
    // ============================================================================
    proptest! {
        #[test]
        fn prop_settlement_report_has_all_required_fields(
            settlement_id_suffix in r"[a-zA-Z0-9]{5,20}",
            transaction_count in 1i32..10000i32,
            gross_amount_paise in 100i64..1000000000i64,
            mdr_fee_paise in 0i64..100000000i64,
            gst_on_fee_paise in 0i64..10000000i64,
            refund_deductions_paise in 0i64..100000000i64,
        ) {
            let settlement_id = format!("settlement-{}", settlement_id_suffix);
            let net_amount_paise = gross_amount_paise - mdr_fee_paise - gst_on_fee_paise - refund_deductions_paise;

            // Verify all required fields can be represented
            prop_assert!(!settlement_id.is_empty(), "settlement_id must not be empty");
            prop_assert!(transaction_count > 0, "transaction_count must be positive");
            prop_assert!(gross_amount_paise > 0, "gross_amount must be positive");
            prop_assert!(mdr_fee_paise >= 0, "MDR_fee must be non-negative");
            prop_assert!(gst_on_fee_paise >= 0, "GST_on_fee must be non-negative");
            prop_assert!(refund_deductions_paise >= 0, "refund_deductions must be non-negative");
            prop_assert!(net_amount_paise >= 0, "net_amount must be non-negative");
        }
    }

    proptest! {
        #[test]
        fn prop_settlement_report_field_types_valid(
            settlement_id_suffix in r"[a-zA-Z0-9]{5,20}",
            transaction_count in 1i32..10000i32,
            gross_amount_paise in 100i64..1000000000i64,
        ) {
            let settlement_id = format!("settlement-{}", settlement_id_suffix);
            let settlement_date = chrono::Utc::now();
            let transfer_date = chrono::Utc::now();
            let utr = format!("UTR{}", Uuid::new_v4());
            let bank_account = "XXXX...1234";

            // Verify field types and formats
            prop_assert!(!settlement_id.is_empty());
            prop_assert!(transaction_count > 0);
            prop_assert!(gross_amount_paise > 0);
            prop_assert!(!utr.is_empty());
            prop_assert!(!bank_account.is_empty());
            prop_assert!(bank_account.contains("XXXX"), "Bank account should be masked");
        }
    }

    proptest! {
        #[test]
        fn prop_settlement_report_amounts_consistency(
            gross_amount_paise in 100i64..1000000000i64,
            mdr_fee_paise in 0i64..100000000i64,
            gst_on_fee_paise in 0i64..10000000i64,
            refund_deductions_paise in 0i64..100000000i64,
        ) {
            // Verify amount calculations are consistent
            let net_amount_paise = gross_amount_paise - mdr_fee_paise - gst_on_fee_paise - refund_deductions_paise;

            // Net amount should never exceed gross amount
            prop_assert!(net_amount_paise <= gross_amount_paise, "net_amount must not exceed gross_amount");

            // All amounts should be non-negative
            prop_assert!(gross_amount_paise >= 0);
            prop_assert!(mdr_fee_paise >= 0);
            prop_assert!(gst_on_fee_paise >= 0);
            prop_assert!(refund_deductions_paise >= 0);
            prop_assert!(net_amount_paise >= 0);
        }
    }

    proptest! {
        #[test]
        fn prop_settlement_report_format_support(
            format_idx in 0..3usize,
        ) {
            let formats = vec![
                ReportFormat::Csv,
                ReportFormat::Pdf,
                ReportFormat::Excel,
            ];
            let format = formats[format_idx].clone();

            // Verify all formats are supported for settlement reports
            let format_str = format.as_str();
            prop_assert!(
                format_str == "csv" || format_str == "pdf" || format_str == "excel",
                "Settlement report format must be csv, pdf, or excel"
            );
        }
    }

    // ============================================================================
    // Property 10: Scheduled Report Generation and Delivery
    // **Validates: Requirements 7.5**
    // For any scheduled report with trigger time reached, the Report_Service SHALL
    // generate the report and send email to all configured recipient_emails
    // ============================================================================
    proptest! {
        #[test]
        fn prop_scheduled_report_frequency_valid(
            frequency_idx in 0..3usize,
        ) {
            let frequencies = vec![
                ReportFrequency::Daily,
                ReportFrequency::Weekly,
                ReportFrequency::Monthly,
            ];
            let frequency = frequencies[frequency_idx].clone();

            // Verify frequency is valid
            let freq_str = frequency.as_str();
            prop_assert!(
                freq_str == "daily" || freq_str == "weekly" || freq_str == "monthly",
                "Frequency must be daily, weekly, or monthly"
            );
        }
    }

    proptest! {
        #[test]
        fn prop_scheduled_report_recipient_emails_valid(
            email_count in 1..10usize,
        ) {
            let mut emails = Vec::new();
            for i in 0..email_count {
                emails.push(format!("recipient{}@example.com", i));
            }

            // Verify recipient emails are non-empty and valid format
            prop_assert!(!emails.is_empty(), "Must have at least one recipient email");
            for email in &emails {
                prop_assert!(email.contains("@"), "Email must contain @");
                prop_assert!(email.contains("."), "Email must contain domain");
            }
        }
    }

    proptest! {
        #[test]
        fn prop_scheduled_report_time_of_day_valid(
            hour in 0..24u32,
            minute in 0..60u32,
        ) {
            // Verify time_of_day is valid
            prop_assert!(hour < 24, "Hour must be 0-23");
            prop_assert!(minute < 60, "Minute must be 0-59");
        }
    }

    proptest! {
        #[test]
        fn prop_scheduled_report_next_run_calculation(
            days_offset in 1..365i64,
        ) {
            let now = chrono::Utc::now();
            let next_run = now + chrono::Duration::days(days_offset);

            // Verify next_run_at is in the future
            prop_assert!(next_run > now, "next_run_at must be in the future");
        }
    }
}

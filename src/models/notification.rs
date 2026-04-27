use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationQueue {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub recipient_type: RecipientType,
    pub recipient_id: Option<Uuid>,
    pub recipient_email: Option<String>,
    pub recipient_phone: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub template_id: Option<String>,
    pub template_data: serde_json::Value,
    pub status: NotificationStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub external_id: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Email,
    Sms,
    Whatsapp,
    Webhook,
    InApp,
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "email" => NotificationType::Email,
            "sms" => NotificationType::Sms,
            "whatsapp" => NotificationType::Whatsapp,
            "webhook" => NotificationType::Webhook,
            "in_app" | "inapp" => NotificationType::InApp,
            _ => NotificationType::Email,
        }
    }
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        match self {
            NotificationType::Email => "email".to_string(),
            NotificationType::Sms => "sms".to_string(),
            NotificationType::Whatsapp => "whatsapp".to_string(),
            NotificationType::Webhook => "webhook".to_string(),
            NotificationType::InApp => "in_app".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecipientType {
    Merchant,
    Customer,
    Admin,
    User,
}

impl From<&str> for RecipientType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "merchant" => RecipientType::Merchant,
            "customer" => RecipientType::Customer,
            "admin" => RecipientType::Admin,
            "user" => RecipientType::User,
            _ => RecipientType::User,
        }
    }
}

impl ToString for RecipientType {
    fn to_string(&self) -> String {
        match self {
            RecipientType::Merchant => "merchant".to_string(),
            RecipientType::Customer => "customer".to_string(),
            RecipientType::Admin => "admin".to_string(),
            RecipientType::User => "user".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
    Retrying,
}

impl From<&str> for NotificationStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => NotificationStatus::Pending,
            "sent" => NotificationStatus::Sent,
            "failed" => NotificationStatus::Failed,
            "cancelled" => NotificationStatus::Cancelled,
            "retrying" => NotificationStatus::Retrying,
            _ => NotificationStatus::Pending,
        }
    }
}

impl ToString for NotificationStatus {
    fn to_string(&self) -> String {
        match self {
            NotificationStatus::Pending => "pending".to_string(),
            NotificationStatus::Sent => "sent".to_string(),
            NotificationStatus::Failed => "failed".to_string(),
            NotificationStatus::Cancelled => "cancelled".to_string(),
            NotificationStatus::Retrying => "retrying".to_string(),
        }
    }
}

impl From<PgRow> for NotificationQueue {
    fn from(row: PgRow) -> Self {
        NotificationQueue {
            id: row.get("id"),
            notification_type: NotificationType::from(row.get::<String, _>("notification_type").as_str()),
            recipient_type: RecipientType::from(row.get::<String, _>("recipient_type").as_str()),
            recipient_id: row.get("recipient_id"),
            recipient_email: row.get("recipient_email"),
            recipient_phone: row.get("recipient_phone"),
            subject: row.get("subject"),
            body: row.get("body"),
            template_id: row.get("template_id"),
            template_data: row.get("template_data"),
            status: NotificationStatus::from(row.get::<String, _>("status").as_str()),
            retry_count: row.get("retry_count"),
            max_retries: row.get("max_retries"),
            scheduled_at: row.get("scheduled_at"),
            sent_at: row.get("sent_at"),
            error_message: row.get("error_message"),
            error_code: row.get("error_code"),
            external_id: row.get("external_id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// Report Jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJob {
    pub id: Uuid,
    pub merchant_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub report_type: String,
    pub date_from: Option<chrono::NaiveDate>,
    pub date_to: Option<chrono::NaiveDate>,
    pub filters: serde_json::Value,
    pub status: ReportJobStatus,
    pub progress: i32,
    pub file_name: Option<String>,
    pub file_url: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub file_format: Option<String>,
    pub error_message: Option<String>,
    pub requested_by: Option<Uuid>,
    pub email_on_complete: bool,
    pub email_recipients: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReportJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl From<&str> for ReportJobStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => ReportJobStatus::Pending,
            "processing" => ReportJobStatus::Processing,
            "completed" => ReportJobStatus::Completed,
            "failed" => ReportJobStatus::Failed,
            "cancelled" => ReportJobStatus::Cancelled,
            _ => ReportJobStatus::Pending,
        }
    }
}

impl ToString for ReportJobStatus {
    fn to_string(&self) -> String {
        match self {
            ReportJobStatus::Pending => "pending".to_string(),
            ReportJobStatus::Processing => "processing".to_string(),
            ReportJobStatus::Completed => "completed".to_string(),
            ReportJobStatus::Failed => "failed".to_string(),
            ReportJobStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl From<PgRow> for ReportJob {
    fn from(row: PgRow) -> Self {
        ReportJob {
            id: row.get("id"),
            merchant_id: row.get("merchant_id"),
            tenant_id: row.get("tenant_id"),
            report_type: row.get("report_type"),
            date_from: row.get("date_from"),
            date_to: row.get("date_to"),
            filters: row.get("filters"),
            status: ReportJobStatus::from(row.get::<String, _>("status").as_str()),
            progress: row.get("progress"),
            file_name: row.get("file_name"),
            file_url: row.get("file_url"),
            file_size_bytes: row.get("file_size_bytes"),
            file_format: row.get("file_format"),
            error_message: row.get("error_message"),
            requested_by: row.get("requested_by"),
            email_on_complete: row.get("email_on_complete"),
            email_recipients: row.get("email_recipients"),
            created_at: row.get("created_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub notification_type: String,
    pub recipient_type: String,
    pub recipient_id: Option<Uuid>,
    pub recipient_email: Option<String>,
    pub recipient_phone: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub template_id: Option<String>,
    pub template_data: Option<serde_json::Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub report_type: String,
    pub date_from: Option<chrono::NaiveDate>,
    pub date_to: Option<chrono::NaiveDate>,
    pub filters: Option<serde_json::Value>,
    pub file_format: Option<String>,
    pub email_on_complete: Option<bool>,
    pub email_recipients: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJobResponse {
    pub id: Uuid,
    pub report_type: String,
    pub status: String,
    pub progress: i32,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<ReportJob> for ReportJobResponse {
    fn from(job: ReportJob) -> Self {
        ReportJobResponse {
            id: job.id,
            report_type: job.report_type,
            status: job.status.to_string(),
            progress: job.progress,
            file_url: job.file_url,
            file_name: job.file_name,
            error_message: job.error_message,
            created_at: job.created_at,
            completed_at: job.completed_at,
        }
    }
}
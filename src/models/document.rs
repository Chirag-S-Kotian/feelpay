use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub document_type: String,
    pub document_purpose: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub storage_key: String,
    pub storage_bucket: Option<String>,
    pub checksum: Option<String>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UploadDocumentRequest {
    pub document_type: String,
    pub document_purpose: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub storage_key: String,
    pub storage_bucket: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub document_type: String,
    pub document_purpose: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateUploadUrlRequest {
    pub document_type: String,
    pub document_purpose: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
}

#[derive(Debug, Serialize)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub storage_key: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyDocumentRequest {
    pub status: String, // 'approved' or 'rejected'
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OtpVerification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub otp_hash: String,
    pub otp_type: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub otp_type: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub otp: String,
    pub otp_type: String,
}

#[derive(Debug, Serialize)]
pub struct OtpResponse {
    pub message: String,
    pub expires_in_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub requested_by: Uuid,
    pub request_type: String,
    pub request_data: Option<serde_json::Value>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub request_type: String,
    pub request_data: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessApprovalRequest {
    pub action: String, // 'approve' or 'reject'
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
}

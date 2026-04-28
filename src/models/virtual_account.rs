use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct VirtualAccount {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub account_number: String,
    pub ifsc_code: String,
    pub bank_name: Option<String>,
    pub beneficiary_name: Option<String>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVirtualAccountRequest {
    pub customer_id: Option<Uuid>,
    pub beneficiary_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

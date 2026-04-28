use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Mandate {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub mandate_type: String,              // upi, card, nach
    pub mandate_reference: Option<String>, // UMN for UPI
    pub status: String,                    // initiated, active, revoked, expired, paused
    pub amount_limit: i64,
    pub currency: String,
    pub frequency: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub payer_vpa: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateMandateRequest {
    pub customer_id: Option<Uuid>,
    pub mandate_type: String,
    pub amount_limit: i64,
    pub currency: Option<String>,
    pub frequency: String,
    pub end_date: Option<DateTime<Utc>>,
    pub payer_vpa: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

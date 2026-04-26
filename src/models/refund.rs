use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Refund {
    pub id: Uuid,
    pub payment_order_id: Uuid,
    pub merchant_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub reason: Option<String>,
    pub idempotency_key: Option<String>,
    pub gateway_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRefundRequest {
    pub amount: Option<i64>, // partial refund; None = full refund
    pub reason: Option<String>,
    pub idempotency_key: Option<String>,
}

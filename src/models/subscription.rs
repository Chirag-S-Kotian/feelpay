use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionInvoice {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub merchant_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_order_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeeRule {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub transaction_type: String,       // e.g., "payment", "payout"
    pub payment_method: Option<String>, // e.g., "mpesa", "card"
    pub percentage_fee: Decimal,
    pub fixed_fee: i64, // amount in minor units
    pub min_fee: Option<i64>,
    pub max_fee: Option<i64>,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

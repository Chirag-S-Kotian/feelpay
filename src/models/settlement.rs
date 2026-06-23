use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Settlement {
    pub id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub amount: Option<i64>,
    pub fee_amount: Option<i64>,
    pub net_amount: Option<i64>,
    pub status: Option<String>,
    pub settlement_date: Option<NaiveDate>,
    pub utr_number: Option<String>,
    pub transaction_count: Option<i32>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementItem {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
    pub fee: i64,
    pub created_at: DateTime<Utc>,
}

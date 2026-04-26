use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LedgerAccount {
    pub id: Uuid,
    pub name: String,
    pub account_type: String,
    pub owner_id: Option<Uuid>,
    pub currency: String,
    pub balance: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub entry_type: String,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

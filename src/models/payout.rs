use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payout {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub method: String,
    pub bank_account_id: Option<Uuid>,
    pub payout_reference: Option<String>,
    pub gateway_reference: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Option<Value>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayoutRequest {
    pub amount: i64,
    pub currency: String,
    pub method: String,
    pub bank_account_id: Option<Uuid>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BankAccount {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub account_holder_name: String,
    pub account_number: String,
    pub ifsc_code: String,
    pub bank_name: Option<String>,
    pub is_verified: bool,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountRequest {
    pub account_holder_name: String,
    pub account_number: String,
    pub ifsc_code: String,
    pub bank_name: Option<String>,
    pub is_primary: Option<bool>,
}

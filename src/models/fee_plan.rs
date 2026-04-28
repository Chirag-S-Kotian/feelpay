use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeePlan {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,

    pub card_debit_bps: i32,
    pub card_credit_bps: i32,
    pub upi_bps: i32,
    pub netbanking_bps: i32,
    pub wallet_bps: i32,
    pub emi_bps: i32,

    pub card_flat_fee: i32,
    pub upi_flat_fee: i32,
    pub netbanking_flat_fee: i32,
    pub wallet_flat_fee: i32,

    pub gst_percentage: f64,
    pub gst_inclusive: bool,

    pub min_transaction_amount: i32,
    pub max_transaction_amount: i32,

    pub is_active: bool,
    pub is_default: bool,
    pub version: i32,
    pub parent_plan_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeePlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub tenant_id: Option<Uuid>,

    pub card_debit_bps: Option<i32>,
    pub card_credit_bps: Option<i32>,
    pub upi_bps: Option<i32>,
    pub netbanking_bps: Option<i32>,
    pub wallet_bps: Option<i32>,
    pub emi_bps: Option<i32>,

    pub card_flat_fee: Option<i32>,
    pub upi_flat_fee: Option<i32>,
    pub netbanking_flat_fee: Option<i32>,
    pub wallet_flat_fee: Option<i32>,

    pub gst_percentage: Option<f64>,
    pub gst_inclusive: Option<bool>,

    pub min_transaction_amount: Option<i32>,
    pub max_transaction_amount: Option<i32>,

    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeePlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub card_debit_bps: Option<i32>,
    pub card_credit_bps: Option<i32>,
    pub upi_bps: Option<i32>,
    pub netbanking_bps: Option<i32>,
    pub wallet_bps: Option<i32>,
    pub emi_bps: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MerchantFeeAssignment {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub fee_plan_id: Uuid,

    pub custom_card_debit_bps: Option<i32>,
    pub custom_card_credit_bps: Option<i32>,
    pub custom_upi_bps: Option<i32>,
    pub custom_netbanking_bps: Option<i32>,
    pub custom_wallet_bps: Option<i32>,

    pub effective_from: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,

    pub assigned_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignFeePlanRequest {
    pub merchant_id: Uuid,
    pub fee_plan_id: Uuid,
    pub custom_card_debit_bps: Option<i32>,
    pub custom_card_credit_bps: Option<i32>,
    pub custom_upi_bps: Option<i32>,
    pub custom_netbanking_bps: Option<i32>,
    pub custom_wallet_bps: Option<i32>,
    pub effective_from: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct FeePlanResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub card_debit_bps: i32,
    pub card_credit_bps: i32,
    pub upi_bps: i32,
    pub netbanking_bps: i32,
    pub wallet_bps: i32,
    pub gst_percentage: f64,
    pub is_active: bool,
    pub is_default: bool,
}

impl From<FeePlan> for FeePlanResponse {
    fn from(plan: FeePlan) -> Self {
        Self {
            id: plan.id,
            name: plan.name,
            description: plan.description,
            card_debit_bps: plan.card_debit_bps,
            card_credit_bps: plan.card_credit_bps,
            upi_bps: plan.upi_bps,
            netbanking_bps: plan.netbanking_bps,
            wallet_bps: plan.wallet_bps,
            gst_percentage: plan.gst_percentage,
            is_active: plan.is_active,
            is_default: plan.is_default,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VerificationLog {
    pub id: Uuid,
    pub merchant_id: Option<Uuid>,
    pub verification_type: String,
    pub provider: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub verification_score: Option<i32>,
    pub verified_fields: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementBatch {
    pub id: Uuid,
    pub batch_date: chrono::NaiveDate,
    pub batch_type: String,
    pub total_merchants: i32,
    pub total_amount: i64,
    pub total_transactions: i32,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSettlementBatchRequest {
    pub batch_date: Option<chrono::NaiveDate>,
    pub batch_type: Option<String>,
}

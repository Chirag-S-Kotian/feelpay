use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Dispute {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub payment_order_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub reason_code: Option<String>,
    pub reason_message: Option<String>,
    pub evidence_due_by: Option<DateTime<Utc>>,
    pub evidence: Option<serde_json::Value>,
    pub connector_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListDisputesQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitEvidenceRequest {
    pub text: Option<String>,
    pub documents: Option<Vec<String>>,
}

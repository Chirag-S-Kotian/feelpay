use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerification {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub verification_type: String,
    pub document_number: Option<String>,
    pub document_data: Option<serde_json::Value>,
    pub status: String,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantTier {
    pub onboarding_status: String,
    pub tier: String,
    pub daily_limit: i64,
    pub monthly_limit: i64,
}

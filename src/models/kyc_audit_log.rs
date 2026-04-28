use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KycAuditLog {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub action: String,
    pub actor_id: Uuid,
    pub previous_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

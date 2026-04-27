use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub name: Option<String>,
    pub key_prefix: String,
    pub key_hash: String,
    pub is_live: bool,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub merchant_id: Uuid,
    pub name: Option<String>,
    pub is_live: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

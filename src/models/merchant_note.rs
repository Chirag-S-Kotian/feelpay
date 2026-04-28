use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantNote {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub content: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantNoteHistory {
    pub id: Uuid,
    pub note_id: Uuid,
    pub previous_content: String,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub content: String, // Max 5000 chars, markdown supported
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: String,
}

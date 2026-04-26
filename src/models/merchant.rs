use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Merchant {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub api_key: String,
    #[serde(skip_serializing)]
    pub api_secret_hash: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub status: String,
    pub business_name: Option<String>,
    pub business_type: Option<String>,
    pub settlement_account: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMerchantRequest {
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub business_name: Option<String>,
    pub business_type: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub type MerchantLoginRequest = LoginRequest;

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub merchant: MerchantPublic,
}

#[derive(Debug, Serialize)]
pub struct MerchantPublic {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub webhook_url: Option<String>,
    pub status: String,
    pub business_name: Option<String>,
    pub created_at: DateTime<Utc>,
}


impl From<Merchant> for MerchantPublic {
    fn from(m: Merchant) -> Self {
        Self {
            id: m.id,
            tenant_id: m.tenant_id,
            name: m.name,
            email: m.email,
            api_key: m.api_key,
            webhook_url: m.webhook_url,
            status: m.status,
            business_name: m.business_name,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMerchantRequest {
    pub name: Option<String>,
    pub business_name: Option<String>,
    pub webhook_url: Option<String>,
    pub settlement_account: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLink {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub link_id: String,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub short_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: PaymentLinkStatus,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub allow_partial: bool,
    pub min_partial_amount: Option<i64>,
    pub max_partial_amount: Option<i64>,
    pub view_count: i32,
    pub attempt_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentLinkStatus {
    Active,
    Expired,
    Paid,
    Cancelled,
}

impl From<&str> for PaymentLinkStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => PaymentLinkStatus::Active,
            "expired" => PaymentLinkStatus::Expired,
            "paid" => PaymentLinkStatus::Paid,
            "cancelled" => PaymentLinkStatus::Cancelled,
            _ => PaymentLinkStatus::Active,
        }
    }
}

impl ToString for PaymentLinkStatus {
    fn to_string(&self) -> String {
        match self {
            PaymentLinkStatus::Active => "active".to_string(),
            PaymentLinkStatus::Expired => "expired".to_string(),
            PaymentLinkStatus::Paid => "paid".to_string(),
            PaymentLinkStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl From<PgRow> for PaymentLink {
    fn from(row: PgRow) -> Self {
        let amount = row.get::<i32, _>("amount") as i64;
        let min_partial_amount = row
            .get::<Option<i32>, _>("min_partial_amount")
            .map(i64::from);
        let max_partial_amount = row
            .get::<Option<i32>, _>("max_partial_amount")
            .map(i64::from);

        PaymentLink {
            id: row.get("id"),
            merchant_id: row.get("merchant_id"),
            tenant_id: row.get("tenant_id"),
            link_id: row.get("link_id"),
            amount,
            currency: row.get("currency"),
            description: row.get("description"),
            short_url: row.get("short_url"),
            expires_at: row.get("expires_at"),
            status: PaymentLinkStatus::from(row.get::<String, _>("status").as_str()),
            customer_id: row.get("customer_id"),
            customer_name: row.get("customer_name"),
            customer_email: row.get("customer_email"),
            customer_phone: row.get("customer_phone"),
            allow_partial: row.get("allow_partial"),
            min_partial_amount,
            max_partial_amount,
            view_count: row.get("view_count"),
            attempt_count: row.get("attempt_count"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentLinkRequest {
    pub amount: i64,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub allow_partial: Option<bool>,
    pub min_partial_amount: Option<i64>,
    pub max_partial_amount: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaymentLinkResponse {
    pub id: Uuid,
    pub link_id: String,
    pub short_url: Option<String>,
    pub full_url: String,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl PaymentLinkResponse {
    pub fn from_link_with_base_url(pl: PaymentLink, base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/');
        let link_id = pl.link_id.clone();
        let short_url = match pl.short_url {
            Some(ref s)
                if (s.contains("localhost") || s.contains("127.0.0.1"))
                    && !base_url.contains("localhost")
                    && !base_url.contains("127.0.0.1") =>
            {
                Some(format!("{}/pay/{}", base_url, link_id))
            }
            Some(ref s)
                if !s.contains("localhost")
                    && !s.contains("127.0.0.1")
                    && (base_url.contains("localhost") || base_url.contains("127.0.0.1")) =>
            {
                Some(format!("{}/pay/{}", base_url, link_id))
            }
            Some(s) => Some(s),
            None => Some(format!("{}/pay/{}", base_url, link_id)),
        };

        PaymentLinkResponse {
            id: pl.id,
            link_id: link_id.clone(),
            short_url,
            full_url: format!("{}/pay/{}", base_url, link_id),
            amount: pl.amount,
            currency: pl.currency,
            description: pl.description,
            expires_at: pl.expires_at,
            status: pl.status.to_string(),
            created_at: pl.created_at,
        }
    }
}

impl From<PaymentLink> for PaymentLinkResponse {
    fn from(pl: PaymentLink) -> Self {
        let base_url = std::env::var("PAYMENT_LINK_BASE_URL")
            .or_else(|_| std::env::var("RENDER_EXTERNAL_URL"))
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::from_link_with_base_url(pl, &base_url)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListPaymentLinksQuery {
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

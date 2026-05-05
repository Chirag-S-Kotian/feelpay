//! Provider connector abstraction layer
//!
//! Each connector implements the ProviderConnector trait, providing a uniform interface
//! for interacting with external payment providers (Stripe, Razorpay, Adyen).
//! The MockProvider is the local default for sandbox/test mode.

pub mod adyen;
pub mod credential_loader;
pub mod mock;
pub mod razorpay;
pub mod redaction;
pub mod stripe;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use crate::error::FeelPayError;

// ── Provider Capability Enum ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    Payments,
    Captures,
    Refunds,
    Transfers,
    Reversals,
    Payouts,
    Disputes,
    Settlements,
    ReconciliationReports,
    HostedCheckout,
    Tokenization,
}

impl fmt::Display for ProviderCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Payments => "payments",
            Self::Captures => "captures",
            Self::Refunds => "refunds",
            Self::Transfers => "transfers",
            Self::Reversals => "reversals",
            Self::Payouts => "payouts",
            Self::Disputes => "disputes",
            Self::Settlements => "settlements",
            Self::ReconciliationReports => "reconciliation_reports",
            Self::HostedCheckout => "hosted_checkout",
            Self::Tokenization => "tokenization",
        };
        write!(f, "{}", s)
    }
}

// ── Normalized Payment Status ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedStatus {
    Created,
    Pending,
    Authorized,
    Captured,
    Failed,
    Cancelled,
    Refunded,
    PartiallyRefunded,
    Disputed,
    PayoutPaid,
    PayoutFailed,
    TransferProcessed,
    TransferFailed,
    SettlementProcessed,
    Unknown,
}

impl fmt::Display for NormalizedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Created => "created",
            Self::Pending => "pending",
            Self::Authorized => "authorized",
            Self::Captured => "captured",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Refunded => "refunded",
            Self::PartiallyRefunded => "partially_refunded",
            Self::Disputed => "disputed",
            Self::PayoutPaid => "payout_paid",
            Self::PayoutFailed => "payout_failed",
            Self::TransferProcessed => "transfer_processed",
            Self::TransferFailed => "transfer_failed",
            Self::SettlementProcessed => "settlement_processed",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

// ── DTOs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub idempotency_key: String,
    pub amount: i64,      // smallest currency unit (paise/cents)
    pub currency: String, // ISO 4217
    pub order_id: String,
    pub description: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub provider_session_id: String,
    pub redirect_url: Option<String>,
    pub status: NormalizedStatus,
    pub raw_status: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRequest {
    pub idempotency_key: String,
    pub provider_reference: String,
    pub amount: Option<i64>, // None = full capture
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub idempotency_key: String,
    pub provider_reference: String,
    pub amount: i64,
    pub currency: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub idempotency_key: String,
    pub source_payment_id: String,
    pub destination_account: String,
    pub amount: i64,
    pub currency: String,
    pub notes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub provider_reference: String,
    pub status: NormalizedStatus,
    pub raw_status: String,
    pub metadata: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderWebhookEvent {
    pub provider: String,
    pub event_type: String,
    pub provider_event_id: Option<String>,
    pub provider_reference: Option<String>,
    pub status: NormalizedStatus,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_event_type: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorHealthCheck {
    pub authenticated: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

// ── Decrypted credentials ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    pub provider: String,
    pub secrets: std::collections::HashMap<String, String>,
}

// ── Required secrets per provider ─────────────────────────────────────────

pub fn required_secrets(provider: &str) -> Vec<&'static str> {
    match provider {
        "stripe" => vec!["api_secret", "webhook_secret"],
        "razorpay" => vec!["api_key", "api_secret", "webhook_secret"],
        "adyen" => vec!["api_key", "merchant_account", "hmac_key"],
        "mock" => vec![],
        _ => vec![],
    }
}

/// Compute readiness score: (present_required_secrets / total_required_secrets) * 100
pub fn readiness_score(provider: &str, available_keys: &[String]) -> u8 {
    let required = required_secrets(provider);
    if required.is_empty() {
        return 100;
    }
    let present = required
        .iter()
        .filter(|k| available_keys.contains(&k.to_string()))
        .count();
    ((present as f64 / required.len() as f64) * 100.0) as u8
}

// ── Provider Connector Trait ──────────────────────────────────────────────

#[async_trait]
pub trait ProviderConnector: Send + Sync {
    /// Human-readable provider name
    fn name(&self) -> &str;

    /// Provider code used in DB (stripe, razorpay, adyen, mock)
    fn code(&self) -> &str;

    /// Capabilities this provider supports
    fn capabilities(&self) -> HashSet<ProviderCapability>;

    /// Create a hosted checkout session or payment intent
    async fn create_session(
        &self,
        creds: &ProviderCredentials,
        req: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse, FeelPayError>;

    /// Capture an authorized payment
    async fn capture_payment(
        &self,
        creds: &ProviderCredentials,
        req: &CaptureRequest,
    ) -> Result<ProviderResponse, FeelPayError>;

    /// Cancel / void an authorized payment
    async fn cancel_payment(
        &self,
        creds: &ProviderCredentials,
        idempotency_key: &str,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError>;

    /// Refund a captured payment
    async fn refund_payment(
        &self,
        creds: &ProviderCredentials,
        req: &RefundRequest,
    ) -> Result<ProviderResponse, FeelPayError>;

    /// Fetch a payment by provider reference
    async fn fetch_payment(
        &self,
        creds: &ProviderCredentials,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError>;

    /// Verify a webhook signature and parse the event
    fn verify_and_parse_webhook(
        &self,
        creds: &ProviderCredentials,
        headers: &std::collections::HashMap<String, String>,
        raw_body: &[u8],
    ) -> Result<ProviderWebhookEvent, FeelPayError>;

    /// Health check: verify authentication against provider
    async fn health_check(
        &self,
        creds: &ProviderCredentials,
    ) -> Result<ConnectorHealthCheck, FeelPayError>;

    /// Create a transfer (Route-style, Razorpay)
    async fn create_transfer(
        &self,
        _creds: &ProviderCredentials,
        _req: &TransferRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        Err(FeelPayError::NotImplemented(format!(
            "{} does not support transfers",
            self.name()
        )))
    }

    /// Reverse a transfer
    async fn reverse_transfer(
        &self,
        _creds: &ProviderCredentials,
        _idempotency_key: &str,
        _transfer_id: &str,
        _amount: Option<i64>,
    ) -> Result<ProviderResponse, FeelPayError> {
        Err(FeelPayError::NotImplemented(format!(
            "{} does not support transfer reversals",
            self.name()
        )))
    }
}

// ── Connector Factory ─────────────────────────────────────────────────────

pub fn create_connector(provider_code: &str) -> Result<Box<dyn ProviderConnector>, FeelPayError> {
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| FeelPayError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    match provider_code {
        "mock" => Ok(Box::new(mock::MockConnector::new())),
        "stripe" => Ok(Box::new(stripe::StripeConnector::new(http_client))),
        "razorpay" => Ok(Box::new(razorpay::RazorpayConnector::new(http_client))),
        "adyen" => Ok(Box::new(adyen::AdyenConnector::new(http_client))),
        _ => Err(FeelPayError::BadRequest(format!(
            "Unknown provider: {}",
            provider_code
        ))),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_secrets_stripe() {
        let secrets = required_secrets("stripe");
        assert!(secrets.contains(&"api_secret"));
        assert!(secrets.contains(&"webhook_secret"));
    }

    #[test]
    fn test_required_secrets_razorpay() {
        let secrets = required_secrets("razorpay");
        assert!(secrets.contains(&"api_key"));
        assert!(secrets.contains(&"api_secret"));
        assert!(secrets.contains(&"webhook_secret"));
    }

    #[test]
    fn test_required_secrets_adyen() {
        let secrets = required_secrets("adyen");
        assert!(secrets.contains(&"api_key"));
        assert!(secrets.contains(&"merchant_account"));
        assert!(secrets.contains(&"hmac_key"));
    }

    #[test]
    fn test_readiness_score_all_present() {
        let available = vec!["api_secret".to_string(), "webhook_secret".to_string()];
        assert_eq!(readiness_score("stripe", &available), 100);
    }

    #[test]
    fn test_readiness_score_partial() {
        let available = vec!["api_key".to_string()];
        assert_eq!(readiness_score("razorpay", &available), 33);
    }

    #[test]
    fn test_readiness_score_mock() {
        assert_eq!(readiness_score("mock", &[]), 100);
    }

    #[test]
    fn test_create_connector_mock() {
        let connector = create_connector("mock").unwrap();
        assert_eq!(connector.code(), "mock");
    }

    #[test]
    fn test_create_connector_unknown() {
        assert!(create_connector("nonexistent").is_err());
    }
}

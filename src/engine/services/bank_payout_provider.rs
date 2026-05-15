use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayoutRequest {
    pub idempotency_key: String,
    pub amount_paise: i64,
    pub currency: String,
    pub payout_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayoutResult {
    pub status: String,
    pub provider_reference: Option<String>,
    pub summary_redacted: Value,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayoutCallback {
    pub provider_event_id: String,
    pub provider_reference: Option<String>,
    pub provider_status: String,
}

#[async_trait]
pub trait BankPayoutProviderAdapter: Send + Sync {
    async fn test_connection(&self) -> BankPayoutResult;
    async fn create_payout(&self, request: BankPayoutRequest) -> BankPayoutResult;
    async fn get_payout_status(&self, provider_reference: &str) -> BankPayoutResult;
    async fn handle_callback(&self, callback: BankPayoutCallback) -> BankPayoutResult;
    async fn fetch_reconciliation_report(&self) -> Option<BankPayoutResult>;
    fn parse_provider_reference(&self, value: &str) -> Option<String>;
    fn map_provider_error(&self, code: &str) -> String;
}

pub struct TestBankPayoutAdapter;

#[async_trait]
impl BankPayoutProviderAdapter for TestBankPayoutAdapter {
    async fn test_connection(&self) -> BankPayoutResult {
        BankPayoutResult {
            status: "verified".to_string(),
            provider_reference: None,
            summary_redacted: json!({ "mode": "test", "reachable": true }),
            error_code: None,
        }
    }

    async fn create_payout(&self, request: BankPayoutRequest) -> BankPayoutResult {
        BankPayoutResult {
            status: "accepted".to_string(),
            provider_reference: Some(format!("test_payout:{}", request.idempotency_key)),
            summary_redacted: json!({
                "mode": "test",
                "amount_paise": request.amount_paise,
                "currency": request.currency,
                "payout_id_present": !request.payout_id.is_empty()
            }),
            error_code: None,
        }
    }

    async fn get_payout_status(&self, provider_reference: &str) -> BankPayoutResult {
        BankPayoutResult {
            status: if self.parse_provider_reference(provider_reference).is_some() {
                "processing"
            } else {
                "failed"
            }
            .to_string(),
            provider_reference: self.parse_provider_reference(provider_reference),
            summary_redacted: json!({ "mode": "test", "reference_present": !provider_reference.is_empty() }),
            error_code: None,
        }
    }

    async fn handle_callback(&self, callback: BankPayoutCallback) -> BankPayoutResult {
        BankPayoutResult {
            status: map_provider_status(&callback.provider_status).to_string(),
            provider_reference: callback.provider_reference,
            summary_redacted: json!({
                "mode": "test",
                "event_id_present": !callback.provider_event_id.is_empty()
            }),
            error_code: None,
        }
    }

    async fn fetch_reconciliation_report(&self) -> Option<BankPayoutResult> {
        None
    }

    fn parse_provider_reference(&self, value: &str) -> Option<String> {
        value
            .strip_prefix("test_payout:")
            .filter(|value| !value.trim().is_empty())
            .map(|_| value.to_string())
    }

    fn map_provider_error(&self, code: &str) -> String {
        format!("test_provider_{code}")
    }
}

pub struct DisabledLiveBankPayoutAdapter;

#[async_trait]
impl BankPayoutProviderAdapter for DisabledLiveBankPayoutAdapter {
    async fn test_connection(&self) -> BankPayoutResult {
        disabled_live_result()
    }
    async fn create_payout(&self, _request: BankPayoutRequest) -> BankPayoutResult {
        disabled_live_result()
    }
    async fn get_payout_status(&self, _provider_reference: &str) -> BankPayoutResult {
        disabled_live_result()
    }
    async fn handle_callback(&self, _callback: BankPayoutCallback) -> BankPayoutResult {
        disabled_live_result()
    }
    async fn fetch_reconciliation_report(&self) -> Option<BankPayoutResult> {
        None
    }
    fn parse_provider_reference(&self, _value: &str) -> Option<String> {
        None
    }
    fn map_provider_error(&self, code: &str) -> String {
        format!("live_provider_unconfigured_{code}")
    }
}

pub fn map_provider_status(status: &str) -> &'static str {
    match status {
        "paid" | "success" | "completed" => "paid",
        "failed" | "rejected" => "failed",
        "reversed" => "reversed",
        "queued" | "accepted" => "queued",
        _ => "processing",
    }
}

fn disabled_live_result() -> BankPayoutResult {
    BankPayoutResult {
        status: "failed".to_string(),
        provider_reference: None,
        summary_redacted: json!({ "mode": "live", "configured": false }),
        error_code: Some("live_provider_not_configured".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_uses_explicit_test_reference() {
        let result = TestBankPayoutAdapter
            .create_payout(BankPayoutRequest {
                idempotency_key: "idem-1".to_string(),
                amount_paise: 100,
                currency: "INR".to_string(),
                payout_id: "payout-1".to_string(),
            })
            .await;
        assert_eq!(result.status, "accepted");
        assert_eq!(
            result.provider_reference.as_deref(),
            Some("test_payout:idem-1")
        );
    }

    #[test]
    fn provider_status_mapping_is_safe() {
        assert_eq!(map_provider_status("completed"), "paid");
        assert_eq!(map_provider_status("rejected"), "failed");
        assert_eq!(map_provider_status("unknown"), "processing");
    }
}

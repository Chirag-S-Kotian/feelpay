//! Mock provider connector — local default for sandbox/test mode.
//!
//! Returns simulated success/failure responses without making any external calls.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    CaptureRequest, ConnectorHealthCheck, CreateSessionRequest, CreateSessionResponse,
    NormalizedStatus, ProviderCapability, ProviderConnector, ProviderCredentials, ProviderResponse,
    ProviderWebhookEvent, RefundRequest,
};
use crate::error::FeelPayError;

pub struct MockConnector;

impl MockConnector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderConnector for MockConnector {
    fn name(&self) -> &str {
        "Mock Provider"
    }

    fn code(&self) -> &str {
        "mock"
    }

    fn capabilities(&self) -> HashSet<ProviderCapability> {
        HashSet::from([
            ProviderCapability::Payments,
            ProviderCapability::Captures,
            ProviderCapability::Refunds,
            ProviderCapability::HostedCheckout,
        ])
    }

    async fn create_session(
        &self,
        _creds: &ProviderCredentials,
        req: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse, FeelPayError> {
        let session_id = format!("mock_sess_{}", Uuid::new_v4());
        tracing::info!(
            provider = "mock",
            session_id = %session_id,
            amount = req.amount,
            currency = %req.currency,
            "Mock: created session"
        );
        Ok(CreateSessionResponse {
            provider_session_id: session_id,
            redirect_url: Some(format!(
                "https://mock.feelpay.local/checkout?order={}",
                req.order_id
            )),
            status: NormalizedStatus::Created,
            raw_status: "created".to_string(),
            metadata: None,
        })
    }

    async fn capture_payment(
        &self,
        _creds: &ProviderCredentials,
        req: &CaptureRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        tracing::info!(
            provider = "mock",
            reference = %req.provider_reference,
            "Mock: captured payment"
        );
        Ok(ProviderResponse {
            provider_reference: req.provider_reference.clone(),
            status: NormalizedStatus::Captured,
            raw_status: "captured".to_string(),
            metadata: None,
            error_code: None,
            error_message: None,
        })
    }

    async fn cancel_payment(
        &self,
        _creds: &ProviderCredentials,
        _idempotency_key: &str,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        tracing::info!(
            provider = "mock",
            reference = %provider_reference,
            "Mock: cancelled payment"
        );
        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: NormalizedStatus::Cancelled,
            raw_status: "cancelled".to_string(),
            metadata: None,
            error_code: None,
            error_message: None,
        })
    }

    async fn refund_payment(
        &self,
        _creds: &ProviderCredentials,
        req: &RefundRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let refund_id = format!("mock_rfnd_{}", Uuid::new_v4());
        tracing::info!(
            provider = "mock",
            refund_id = %refund_id,
            amount = req.amount,
            "Mock: refunded payment"
        );
        Ok(ProviderResponse {
            provider_reference: refund_id,
            status: NormalizedStatus::Refunded,
            raw_status: "refunded".to_string(),
            metadata: None,
            error_code: None,
            error_message: None,
        })
    }

    async fn fetch_payment(
        &self,
        _creds: &ProviderCredentials,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: NormalizedStatus::Captured,
            raw_status: "captured".to_string(),
            metadata: None,
            error_code: None,
            error_message: None,
        })
    }

    fn verify_and_parse_webhook(
        &self,
        _creds: &ProviderCredentials,
        headers: &HashMap<String, String>,
        raw_body: &[u8],
    ) -> Result<ProviderWebhookEvent, FeelPayError> {
        // Mock always accepts
        let payload: serde_json::Value = serde_json::from_slice(raw_body)
            .map_err(|e| FeelPayError::BadRequest(format!("Invalid JSON: {}", e)))?;

        let event_type = payload
            .get("event")
            .or_else(|| payload.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("mock.event")
            .to_string();

        Ok(ProviderWebhookEvent {
            provider: "mock".to_string(),
            event_type: event_type.clone(),
            provider_event_id: headers.get("x-mock-event-id").cloned(),
            provider_reference: payload
                .get("reference")
                .and_then(|v| v.as_str())
                .map(String::from),
            status: NormalizedStatus::Captured,
            amount: payload.get("amount").and_then(|v| v.as_i64()),
            currency: payload
                .get("currency")
                .and_then(|v| v.as_str())
                .map(String::from),
            metadata: None,
            raw_event_type: event_type,
            timestamp: Some(chrono::Utc::now()),
        })
    }

    async fn health_check(
        &self,
        _creds: &ProviderCredentials,
    ) -> Result<ConnectorHealthCheck, FeelPayError> {
        Ok(ConnectorHealthCheck {
            authenticated: true,
            latency_ms: 1,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_create_session() {
        let connector = MockConnector::new();
        let creds = ProviderCredentials {
            provider: "mock".to_string(),
            secrets: HashMap::new(),
        };
        let req = CreateSessionRequest {
            idempotency_key: "idem_1".to_string(),
            amount: 10000,
            currency: "INR".to_string(),
            order_id: "order_1".to_string(),
            description: None,
            customer_email: None,
            customer_phone: None,
            return_url: None,
            cancel_url: None,
            metadata: None,
        };
        let resp = connector.create_session(&creds, &req).await.unwrap();
        assert!(resp.provider_session_id.starts_with("mock_sess_"));
        assert_eq!(resp.status, NormalizedStatus::Created);
    }

    #[tokio::test]
    async fn test_mock_health_check() {
        let connector = MockConnector::new();
        let creds = ProviderCredentials {
            provider: "mock".to_string(),
            secrets: HashMap::new(),
        };
        let health = connector.health_check(&creds).await.unwrap();
        assert!(health.authenticated);
    }

    #[test]
    fn test_mock_webhook_parse() {
        let connector = MockConnector::new();
        let creds = ProviderCredentials {
            provider: "mock".to_string(),
            secrets: HashMap::new(),
        };
        let body = serde_json::json!({
            "event": "payment.captured",
            "reference": "mock_ref_1",
            "amount": 5000
        });
        let headers = HashMap::new();
        let event = connector
            .verify_and_parse_webhook(&creds, &headers, serde_json::to_vec(&body).unwrap().as_slice())
            .unwrap();
        assert_eq!(event.event_type, "payment.captured");
        assert_eq!(event.provider_reference, Some("mock_ref_1".to_string()));
    }
}

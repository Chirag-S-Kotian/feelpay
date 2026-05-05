//! Razorpay provider connector — safe MVP integration
//!
//! Uses Razorpay Orders API, Payments API, Refunds API, and Route transfers.
//! Webhook verification via HMAC-SHA256. No raw card data.

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::redaction::redact_metadata;
use super::{
    CaptureRequest, ConnectorHealthCheck, CreateSessionRequest, CreateSessionResponse,
    NormalizedStatus, ProviderCapability, ProviderConnector, ProviderCredentials, ProviderResponse,
    ProviderWebhookEvent, RefundRequest, TransferRequest,
};
use crate::error::FeelPayError;

type HmacSha256 = Hmac<Sha256>;

const RAZORPAY_API_BASE: &str = "https://api.razorpay.com/v1";

pub struct RazorpayConnector {
    client: reqwest::Client,
}

impl RazorpayConnector {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn api_key(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("api_key")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Razorpay api_key".into()))
    }

    fn api_secret(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("api_secret")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Razorpay api_secret".into()))
    }

    fn webhook_secret(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("webhook_secret")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Razorpay webhook_secret".into()))
    }

    fn map_razorpay_error(status: reqwest::StatusCode, body: &serde_json::Value) -> FeelPayError {
        let desc = body
            .get("error")
            .and_then(|e| e.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("Unknown Razorpay error");
        let code = body
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        tracing::error!(
            provider = "razorpay",
            http_status = %status,
            error_code = %code,
            "Razorpay API error (message redacted)"
        );

        if status == reqwest::StatusCode::UNAUTHORIZED {
            FeelPayError::ConnectorError("Razorpay authentication failed".into())
        } else {
            FeelPayError::ConnectorError(format!("Razorpay error [{}]: {}", code, desc))
        }
    }
}

fn normalize_razorpay_status(status: &str) -> NormalizedStatus {
    match status {
        "created" => NormalizedStatus::Created,
        "authorized" => NormalizedStatus::Authorized,
        "captured" => NormalizedStatus::Captured,
        "failed" => NormalizedStatus::Failed,
        "refunded" => NormalizedStatus::Refunded,
        "partially_refunded" => NormalizedStatus::PartiallyRefunded,
        "processed" => NormalizedStatus::Captured,
        _ => NormalizedStatus::Unknown,
    }
}

fn normalize_razorpay_event(event_type: &str) -> NormalizedStatus {
    match event_type {
        "payment.captured" => NormalizedStatus::Captured,
        "payment.failed" => NormalizedStatus::Failed,
        "refund.processed" => NormalizedStatus::Refunded,
        "transfer.processed" => NormalizedStatus::TransferProcessed,
        "transfer.failed" => NormalizedStatus::TransferFailed,
        "settlement.processed" => NormalizedStatus::SettlementProcessed,
        _ => NormalizedStatus::Unknown,
    }
}

#[async_trait]
impl ProviderConnector for RazorpayConnector {
    fn name(&self) -> &str {
        "Razorpay"
    }

    fn code(&self) -> &str {
        "razorpay"
    }

    fn capabilities(&self) -> HashSet<ProviderCapability> {
        HashSet::from([
            ProviderCapability::Payments,
            ProviderCapability::Captures,
            ProviderCapability::Refunds,
            ProviderCapability::Transfers,
            ProviderCapability::Reversals,
            ProviderCapability::Settlements,
        ])
    }

    async fn create_session(
        &self,
        creds: &ProviderCredentials,
        req: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let body = serde_json::json!({
            "amount": req.amount,
            "currency": req.currency.to_uppercase(),
            "receipt": req.order_id,
            "notes": {
                "description": req.description.as_deref().unwrap_or(""),
                "customer_email": req.customer_email.as_deref().unwrap_or(""),
            }
        });

        let resp = self
            .client
            .post(format!("{}/orders", RAZORPAY_API_BASE))
            .basic_auth(key, Some(secret))
            .header("X-Razorpay-Idempotency-Key", &req.idempotency_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Razorpay request failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Razorpay parse error: {}", e)))?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let order_id = resp_body["id"].as_str().unwrap_or("").to_string();
        let raw_status = resp_body["status"]
            .as_str()
            .unwrap_or("created")
            .to_string();

        tracing::info!(
            provider = "razorpay",
            order_id = %order_id,
            status = %raw_status,
            "Razorpay: order created"
        );

        Ok(CreateSessionResponse {
            provider_session_id: order_id,
            redirect_url: None, // Razorpay uses client-side SDK checkout, not redirect
            status: normalize_razorpay_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
        })
    }

    async fn capture_payment(
        &self,
        creds: &ProviderCredentials,
        req: &CaptureRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let body = serde_json::json!({
            "amount": req.amount.unwrap_or(0),
            "currency": req.currency.to_uppercase(),
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/capture",
                RAZORPAY_API_BASE, req.provider_reference
            ))
            .basic_auth(key, Some(secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Razorpay capture failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Razorpay capture parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let raw_status = resp_body["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(ProviderResponse {
            provider_reference: req.provider_reference.clone(),
            status: normalize_razorpay_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn cancel_payment(
        &self,
        _creds: &ProviderCredentials,
        _idempotency_key: &str,
        _provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        Err(FeelPayError::NotImplemented(
            "Razorpay does not support payment cancellation — use refund instead".into(),
        ))
    }

    async fn refund_payment(
        &self,
        creds: &ProviderCredentials,
        req: &RefundRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let body = serde_json::json!({
            "amount": req.amount,
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/refund",
                RAZORPAY_API_BASE, req.provider_reference
            ))
            .basic_auth(key, Some(secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Razorpay refund failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Razorpay refund parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let refund_id = resp_body["id"].as_str().unwrap_or("").to_string();
        let raw_status = resp_body["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(ProviderResponse {
            provider_reference: refund_id,
            status: if raw_status == "processed" {
                NormalizedStatus::Refunded
            } else {
                NormalizedStatus::Pending
            },
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn fetch_payment(
        &self,
        creds: &ProviderCredentials,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let resp = self
            .client
            .get(format!(
                "{}/payments/{}",
                RAZORPAY_API_BASE, provider_reference
            ))
            .basic_auth(key, Some(secret))
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Razorpay fetch failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Razorpay fetch parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let raw_status = resp_body["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: normalize_razorpay_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn create_transfer(
        &self,
        creds: &ProviderCredentials,
        req: &TransferRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let body = serde_json::json!({
            "transfers": [{
                "account": req.destination_account,
                "amount": req.amount,
                "currency": req.currency.to_uppercase(),
                "notes": req.notes,
            }]
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/transfers",
                RAZORPAY_API_BASE, req.source_payment_id
            ))
            .basic_auth(key, Some(secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                FeelPayError::ConnectorError(format!("Razorpay transfer failed: {}", e))
            })?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Razorpay transfer parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let transfer_id = resp_body["items"][0]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ProviderResponse {
            provider_reference: transfer_id,
            status: NormalizedStatus::TransferProcessed,
            raw_status: "processed".to_string(),
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn reverse_transfer(
        &self,
        creds: &ProviderCredentials,
        _idempotency_key: &str,
        transfer_id: &str,
        amount: Option<i64>,
    ) -> Result<ProviderResponse, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;

        let mut body = serde_json::json!({});
        if let Some(amt) = amount {
            body["amount"] = serde_json::json!(amt);
        }

        let resp = self
            .client
            .post(format!(
                "{}/transfers/{}/reversals",
                RAZORPAY_API_BASE, transfer_id
            ))
            .basic_auth(key, Some(secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                FeelPayError::ConnectorError(format!("Razorpay reversal failed: {}", e))
            })?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Razorpay reversal parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_razorpay_error(status, &resp_body));
        }

        let reversal_id = resp_body["id"].as_str().unwrap_or("").to_string();

        Ok(ProviderResponse {
            provider_reference: reversal_id,
            status: NormalizedStatus::Refunded,
            raw_status: "reversed".to_string(),
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    fn verify_and_parse_webhook(
        &self,
        creds: &ProviderCredentials,
        headers: &HashMap<String, String>,
        raw_body: &[u8],
    ) -> Result<ProviderWebhookEvent, FeelPayError> {
        let whsec = Self::webhook_secret(creds)?;
        let sig = headers.get("x-razorpay-signature").ok_or_else(|| {
            FeelPayError::Unauthorized("Missing X-Razorpay-Signature header".into())
        })?;

        // Razorpay: HMAC-SHA256(webhook_secret, raw_body)
        let mut mac = HmacSha256::new_from_slice(whsec.as_bytes())
            .map_err(|_| FeelPayError::Internal("Invalid webhook secret".into()))?;
        mac.update(raw_body);
        let expected = hex::encode(mac.finalize().into_bytes());

        if expected.len() != sig.len()
            || expected
                .as_bytes()
                .iter()
                .zip(sig.as_bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                != 0
        {
            return Err(FeelPayError::Unauthorized(
                "Razorpay webhook signature mismatch".into(),
            ));
        }

        let payload: serde_json::Value = serde_json::from_slice(raw_body)
            .map_err(|e| FeelPayError::BadRequest(format!("Invalid webhook JSON: {}", e)))?;

        let event_type = payload["event"].as_str().unwrap_or("unknown").to_string();
        let entity = &payload["payload"]["payment"]["entity"];

        let provider_ref = entity["id"].as_str().map(String::from);
        let amount = entity["amount"].as_i64();
        let currency = entity["currency"].as_str().map(String::from);

        Ok(ProviderWebhookEvent {
            provider: "razorpay".to_string(),
            event_type: event_type.clone(),
            provider_event_id: payload["event_id"].as_str().map(String::from),
            provider_reference: provider_ref,
            status: normalize_razorpay_event(&event_type),
            amount,
            currency,
            metadata: Some(redact_metadata(entity)),
            raw_event_type: event_type,
            timestamp: Some(chrono::Utc::now()),
        })
    }

    async fn health_check(
        &self,
        creds: &ProviderCredentials,
    ) -> Result<ConnectorHealthCheck, FeelPayError> {
        let key = Self::api_key(creds)?;
        let secret = Self::api_secret(creds)?;
        let start = Instant::now();

        let resp = self
            .client
            .get(format!("{}/payments?count=1", RAZORPAY_API_BASE))
            .basic_auth(key, Some(secret))
            .send()
            .await;

        let latency = start.elapsed().as_millis() as u64;

        match resp {
            Ok(r) => {
                let authenticated = r.status().is_success();
                let error = if !authenticated {
                    Some(format!("HTTP {}", r.status()))
                } else {
                    None
                };
                Ok(ConnectorHealthCheck {
                    authenticated,
                    latency_ms: latency,
                    error,
                })
            }
            Err(e) => Ok(ConnectorHealthCheck {
                authenticated: false,
                latency_ms: latency,
                error: Some(format!("Connection error: {}", e)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_razorpay_status() {
        assert_eq!(
            normalize_razorpay_status("captured"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_razorpay_status("authorized"),
            NormalizedStatus::Authorized
        );
        assert_eq!(
            normalize_razorpay_status("failed"),
            NormalizedStatus::Failed
        );
    }

    #[test]
    fn test_normalize_razorpay_event() {
        assert_eq!(
            normalize_razorpay_event("payment.captured"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_razorpay_event("refund.processed"),
            NormalizedStatus::Refunded
        );
        assert_eq!(
            normalize_razorpay_event("transfer.processed"),
            NormalizedStatus::TransferProcessed
        );
    }

    #[test]
    fn test_missing_credentials_fail() {
        let creds = ProviderCredentials {
            provider: "razorpay".to_string(),
            secrets: HashMap::new(),
        };
        assert!(RazorpayConnector::api_key(&creds).is_err());
        assert!(RazorpayConnector::api_secret(&creds).is_err());
        assert!(RazorpayConnector::webhook_secret(&creds).is_err());
    }

    #[test]
    fn test_webhook_signature_invalid() {
        let connector = RazorpayConnector::new(reqwest::Client::new());
        let creds = ProviderCredentials {
            provider: "razorpay".to_string(),
            secrets: HashMap::from([("webhook_secret".to_string(), "secret_test".to_string())]),
        };
        let headers = HashMap::from([(
            "x-razorpay-signature".to_string(),
            "bad_signature".to_string(),
        )]);
        let body = b"{\"event\":\"payment.captured\",\"payload\":{\"payment\":{\"entity\":{\"id\":\"pay_123\"}}}}";
        let result = connector.verify_and_parse_webhook(&creds, &headers, body);
        assert!(result.is_err());
    }

    #[test]
    fn test_capabilities() {
        let connector = RazorpayConnector::new(reqwest::Client::new());
        let caps = connector.capabilities();
        assert!(caps.contains(&ProviderCapability::Transfers));
        assert!(caps.contains(&ProviderCapability::Reversals));
    }
}

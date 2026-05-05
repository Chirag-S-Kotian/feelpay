//! Stripe provider connector — safe MVP integration
//!
//! Uses Stripe API v1 with hosted checkout sessions and payment intents.
//! No raw card data — hosted checkout only. Webhook verification via HMAC-SHA256.

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::redaction::redact_metadata;
use super::{
    CaptureRequest, ConnectorHealthCheck, CreateSessionRequest, CreateSessionResponse,
    NormalizedStatus, ProviderCapability, ProviderConnector, ProviderCredentials, ProviderResponse,
    ProviderWebhookEvent, RefundRequest,
};
use crate::error::FeelPayError;

type HmacSha256 = Hmac<Sha256>;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

pub struct StripeConnector {
    client: reqwest::Client,
}

impl StripeConnector {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn secret_key(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("api_secret")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Stripe api_secret".into()))
    }

    fn webhook_secret(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("webhook_secret")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Stripe webhook_secret".into()))
    }

    /// Map Stripe API errors into FeelPayError with redaction
    fn map_stripe_error(status: reqwest::StatusCode, body: &serde_json::Value) -> FeelPayError {
        let error_msg = body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown Stripe error");
        let error_code = body
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        tracing::error!(
            provider = "stripe",
            http_status = %status,
            error_code = %error_code,
            "Stripe API error (message redacted)"
        );

        if status == reqwest::StatusCode::UNAUTHORIZED {
            FeelPayError::ConnectorError("Stripe authentication failed".into())
        } else {
            FeelPayError::ConnectorError(format!("Stripe error [{}]: {}", error_code, error_msg))
        }
    }
}

/// Normalize Stripe statuses to internal enum
fn normalize_stripe_status(stripe_status: &str) -> NormalizedStatus {
    match stripe_status {
        "requires_payment_method" | "requires_confirmation" | "requires_action" => {
            NormalizedStatus::Pending
        }
        "processing" => NormalizedStatus::Pending,
        "requires_capture" => NormalizedStatus::Authorized,
        "succeeded" | "complete" => NormalizedStatus::Captured,
        "canceled" | "expired" => NormalizedStatus::Cancelled,
        _ => NormalizedStatus::Unknown,
    }
}

/// Normalize Stripe webhook event types to internal status
fn normalize_stripe_event(event_type: &str) -> NormalizedStatus {
    match event_type {
        "checkout.session.completed" => NormalizedStatus::Captured,
        "payment_intent.succeeded" => NormalizedStatus::Captured,
        "payment_intent.payment_failed" => NormalizedStatus::Failed,
        "charge.refunded" => NormalizedStatus::Refunded,
        "charge.dispute.created" => NormalizedStatus::Disputed,
        "payout.paid" => NormalizedStatus::PayoutPaid,
        "payout.failed" => NormalizedStatus::PayoutFailed,
        _ => NormalizedStatus::Unknown,
    }
}

#[async_trait]
impl ProviderConnector for StripeConnector {
    fn name(&self) -> &str {
        "Stripe"
    }

    fn code(&self) -> &str {
        "stripe"
    }

    fn capabilities(&self) -> HashSet<ProviderCapability> {
        HashSet::from([
            ProviderCapability::Payments,
            ProviderCapability::Captures,
            ProviderCapability::Refunds,
            ProviderCapability::Payouts,
            ProviderCapability::Disputes,
            ProviderCapability::HostedCheckout,
            ProviderCapability::Tokenization,
        ])
    }

    async fn create_session(
        &self,
        creds: &ProviderCredentials,
        req: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse, FeelPayError> {
        let sk = Self::secret_key(creds)?;

        let mut params = vec![
            ("payment_method_types[]".to_string(), "card".to_string()),
            ("mode".to_string(), "payment".to_string()),
            (
                "line_items[0][price_data][currency]".to_string(),
                req.currency.to_lowercase(),
            ),
            (
                "line_items[0][price_data][unit_amount]".to_string(),
                req.amount.to_string(),
            ),
            (
                "line_items[0][price_data][product_data][name]".to_string(),
                req.description
                    .as_deref()
                    .unwrap_or("Payment")
                    .to_string(),
            ),
            ("line_items[0][quantity]".to_string(), "1".to_string()),
            (
                "client_reference_id".to_string(),
                req.order_id.clone(),
            ),
        ];

        if let Some(url) = &req.return_url {
            params.push(("success_url".to_string(), url.clone()));
            params.push((
                "cancel_url".to_string(),
                req.cancel_url.as_deref().unwrap_or(url).to_string(),
            ));
        }

        if let Some(email) = &req.customer_email {
            params.push(("customer_email".to_string(), email.clone()));
        }

        let resp = self
            .client
            .post(format!("{}/checkout/sessions", STRIPE_API_BASE))
            .basic_auth(sk, None::<&str>)
            .header("Idempotency-Key", &req.idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe request failed: {}", e)))?;

        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe parse error: {}", e)))?;

        if !status.is_success() {
            return Err(Self::map_stripe_error(status, &body));
        }

        let session_id = body["id"].as_str().unwrap_or("").to_string();
        let checkout_url = body["url"].as_str().map(String::from);
        let raw_status = body["status"].as_str().unwrap_or("unknown").to_string();

        tracing::info!(
            provider = "stripe",
            session_id = %session_id,
            status = %raw_status,
            "Stripe: checkout session created"
        );

        Ok(CreateSessionResponse {
            provider_session_id: session_id,
            redirect_url: checkout_url,
            status: normalize_stripe_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&body)),
        })
    }

    async fn capture_payment(
        &self,
        creds: &ProviderCredentials,
        req: &CaptureRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let sk = Self::secret_key(creds)?;

        let mut params = vec![];
        if let Some(amount) = req.amount {
            params.push(("amount_to_capture", amount.to_string()));
        }

        let resp = self
            .client
            .post(format!(
                "{}/payment_intents/{}/capture",
                STRIPE_API_BASE, req.provider_reference
            ))
            .basic_auth(sk, None::<&str>)
            .header("Idempotency-Key", &req.idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe capture failed: {}", e)))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Stripe capture parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_stripe_error(status, &body));
        }

        let raw_status = body["status"].as_str().unwrap_or("unknown").to_string();

        Ok(ProviderResponse {
            provider_reference: req.provider_reference.clone(),
            status: normalize_stripe_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn cancel_payment(
        &self,
        creds: &ProviderCredentials,
        idempotency_key: &str,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        let sk = Self::secret_key(creds)?;

        let resp = self
            .client
            .post(format!(
                "{}/payment_intents/{}/cancel",
                STRIPE_API_BASE, provider_reference
            ))
            .basic_auth(sk, None::<&str>)
            .header("Idempotency-Key", idempotency_key)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe cancel failed: {}", e)))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Stripe cancel parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_stripe_error(status, &body));
        }

        let raw_status = body["status"].as_str().unwrap_or("unknown").to_string();

        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: normalize_stripe_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn refund_payment(
        &self,
        creds: &ProviderCredentials,
        req: &RefundRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let sk = Self::secret_key(creds)?;

        let mut params = vec![
            ("payment_intent", req.provider_reference.clone()),
            ("amount", req.amount.to_string()),
        ];
        if let Some(reason) = &req.reason {
            params.push(("reason", reason.clone()));
        }

        let resp = self
            .client
            .post(format!("{}/refunds", STRIPE_API_BASE))
            .basic_auth(sk, None::<&str>)
            .header("Idempotency-Key", &req.idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe refund failed: {}", e)))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Stripe refund parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_stripe_error(status, &body));
        }

        let refund_id = body["id"].as_str().unwrap_or("").to_string();
        let raw_status = body["status"].as_str().unwrap_or("unknown").to_string();

        Ok(ProviderResponse {
            provider_reference: refund_id,
            status: if raw_status == "succeeded" {
                NormalizedStatus::Refunded
            } else {
                NormalizedStatus::Pending
            },
            raw_status,
            metadata: Some(redact_metadata(&body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn fetch_payment(
        &self,
        creds: &ProviderCredentials,
        provider_reference: &str,
    ) -> Result<ProviderResponse, FeelPayError> {
        let sk = Self::secret_key(creds)?;

        let resp = self
            .client
            .get(format!(
                "{}/payment_intents/{}",
                STRIPE_API_BASE, provider_reference
            ))
            .basic_auth(sk, None::<&str>)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Stripe fetch failed: {}", e)))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Stripe fetch parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_stripe_error(status, &body));
        }

        let raw_status = body["status"].as_str().unwrap_or("unknown").to_string();

        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: normalize_stripe_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&body)),
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
        let sig_header = headers
            .get("stripe-signature")
            .ok_or_else(|| FeelPayError::Unauthorized("Missing Stripe-Signature header".into()))?;

        // Parse Stripe signature header: t=timestamp,v1=signature
        let mut timestamp = None;
        let mut sig_hex = None;
        for part in sig_header.split(',') {
            let part = part.trim();
            if let Some((key, val)) = part.split_once('=') {
                match key {
                    "t" => timestamp = val.parse::<i64>().ok(),
                    "v1" => sig_hex = Some(val.to_string()),
                    _ => {}
                }
            }
        }

        let ts = timestamp
            .ok_or_else(|| FeelPayError::Unauthorized("Missing timestamp in signature".into()))?;
        let sig = sig_hex
            .ok_or_else(|| FeelPayError::Unauthorized("Missing v1 signature".into()))?;

        // Check timestamp freshness (5 min tolerance)
        let now = chrono::Utc::now().timestamp();
        if (now - ts).abs() > 300 {
            return Err(FeelPayError::Unauthorized(
                "Stripe webhook timestamp expired".into(),
            ));
        }

        // Compute expected signature: HMAC-SHA256(whsec, "timestamp.body")
        let signed_payload = format!("{}.{}", ts, String::from_utf8_lossy(raw_body));
        let mut mac = HmacSha256::new_from_slice(whsec.as_bytes())
            .map_err(|_| FeelPayError::Internal("Invalid webhook secret".into()))?;
        mac.update(signed_payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        // Constant-time comparison
        if expected.len() != sig.len()
            || expected
                .as_bytes()
                .iter()
                .zip(sig.as_bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                != 0
        {
            return Err(FeelPayError::Unauthorized(
                "Stripe webhook signature mismatch".into(),
            ));
        }

        // Parse event body
        let payload: serde_json::Value = serde_json::from_slice(raw_body)
            .map_err(|e| FeelPayError::BadRequest(format!("Invalid webhook JSON: {}", e)))?;

        let event_type = payload["type"].as_str().unwrap_or("unknown").to_string();
        let event_id = payload["id"].as_str().map(String::from);
        let object = &payload["data"]["object"];

        let provider_ref = object["id"]
            .as_str()
            .or_else(|| object["payment_intent"].as_str())
            .map(String::from);
        let amount = object["amount"].as_i64().or_else(|| object["amount_total"].as_i64());
        let currency = object["currency"].as_str().map(String::from);

        Ok(ProviderWebhookEvent {
            provider: "stripe".to_string(),
            event_type: event_type.clone(),
            provider_event_id: event_id,
            provider_reference: provider_ref,
            status: normalize_stripe_event(&event_type),
            amount,
            currency,
            metadata: Some(redact_metadata(object)),
            raw_event_type: event_type,
            timestamp: Some(chrono::Utc::now()),
        })
    }

    async fn health_check(
        &self,
        creds: &ProviderCredentials,
    ) -> Result<ConnectorHealthCheck, FeelPayError> {
        let sk = Self::secret_key(creds)?;
        let start = Instant::now();

        let resp = self
            .client
            .get(format!("{}/balance", STRIPE_API_BASE))
            .basic_auth(sk, None::<&str>)
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
    fn test_normalize_stripe_status() {
        assert_eq!(
            normalize_stripe_status("requires_capture"),
            NormalizedStatus::Authorized
        );
        assert_eq!(
            normalize_stripe_status("succeeded"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_stripe_status("canceled"),
            NormalizedStatus::Cancelled
        );
        assert_eq!(
            normalize_stripe_status("unknown_state"),
            NormalizedStatus::Unknown
        );
    }

    #[test]
    fn test_normalize_stripe_event() {
        assert_eq!(
            normalize_stripe_event("checkout.session.completed"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_stripe_event("charge.refunded"),
            NormalizedStatus::Refunded
        );
        assert_eq!(
            normalize_stripe_event("charge.dispute.created"),
            NormalizedStatus::Disputed
        );
        assert_eq!(
            normalize_stripe_event("payout.paid"),
            NormalizedStatus::PayoutPaid
        );
    }

    #[test]
    fn test_missing_credentials_fail() {
        let creds = ProviderCredentials {
            provider: "stripe".to_string(),
            secrets: HashMap::new(),
        };
        assert!(StripeConnector::secret_key(&creds).is_err());
        assert!(StripeConnector::webhook_secret(&creds).is_err());
    }

    #[test]
    fn test_webhook_signature_invalid() {
        let connector = StripeConnector::new(reqwest::Client::new());
        let creds = ProviderCredentials {
            provider: "stripe".to_string(),
            secrets: HashMap::from([("webhook_secret".to_string(), "whsec_test".to_string())]),
        };
        let headers = HashMap::from([(
            "stripe-signature".to_string(),
            format!("t={},v1=invalid_hex", chrono::Utc::now().timestamp()),
        )]);
        let body = b"{\"type\":\"test\"}";
        let result = connector.verify_and_parse_webhook(&creds, &headers, body);
        assert!(result.is_err());
    }
}

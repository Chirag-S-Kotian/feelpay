//! Adyen provider connector — safe MVP integration
//!
//! Uses Adyen Checkout API v71 for sessions, captures, refunds.
//! HMAC-SHA256 webhook verification. No raw card data.

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

const ADYEN_CHECKOUT_BASE: &str = "https://checkout-test.adyen.com/v71";

pub struct AdyenConnector {
    client: reqwest::Client,
}

impl AdyenConnector {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn api_key(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("api_key")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Adyen api_key".into()))
    }

    fn merchant_account(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("merchant_account")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Adyen merchant_account".into()))
    }

    fn hmac_key(creds: &ProviderCredentials) -> Result<&str, FeelPayError> {
        creds
            .secrets
            .get("hmac_key")
            .map(|s| s.as_str())
            .ok_or_else(|| FeelPayError::ConnectorError("Missing Adyen hmac_key".into()))
    }

    fn map_adyen_error(status: reqwest::StatusCode, body: &serde_json::Value) -> FeelPayError {
        let message = body["message"]
            .as_str()
            .or_else(|| body["errorType"].as_str())
            .unwrap_or("Unknown Adyen error");
        let error_code = body["errorCode"].as_str().unwrap_or("unknown");

        tracing::error!(
            provider = "adyen",
            http_status = %status,
            error_code = %error_code,
            "Adyen API error (message redacted)"
        );

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            FeelPayError::ConnectorError("Adyen authentication failed".into())
        } else {
            FeelPayError::ConnectorError(format!("Adyen error [{}]: {}", error_code, message))
        }
    }
}

fn normalize_adyen_status(result_code: &str) -> NormalizedStatus {
    match result_code {
        "Authorised" | "AuthenticationFinished" => NormalizedStatus::Authorized,
        "Captured" => NormalizedStatus::Captured,
        "Refused" | "Error" => NormalizedStatus::Failed,
        "Cancelled" => NormalizedStatus::Cancelled,
        "Pending" | "Received" | "RedirectShopper" => NormalizedStatus::Pending,
        _ => NormalizedStatus::Unknown,
    }
}

fn normalize_adyen_notification(event_code: &str) -> NormalizedStatus {
    match event_code {
        "AUTHORISATION" => NormalizedStatus::Authorized,
        "CAPTURE" => NormalizedStatus::Captured,
        "REFUND" => NormalizedStatus::Refunded,
        "CHARGEBACK" => NormalizedStatus::Disputed,
        "PAYOUT" => NormalizedStatus::PayoutPaid,
        "REPORT_AVAILABLE" => NormalizedStatus::Unknown,
        _ => NormalizedStatus::Unknown,
    }
}

#[async_trait]
impl ProviderConnector for AdyenConnector {
    fn name(&self) -> &str {
        "Adyen"
    }

    fn code(&self) -> &str {
        "adyen"
    }

    fn capabilities(&self) -> HashSet<ProviderCapability> {
        HashSet::from([
            ProviderCapability::Payments,
            ProviderCapability::Captures,
            ProviderCapability::Refunds,
            ProviderCapability::Payouts,
            ProviderCapability::Disputes,
            ProviderCapability::HostedCheckout,
            ProviderCapability::ReconciliationReports,
        ])
    }

    async fn create_session(
        &self,
        creds: &ProviderCredentials,
        req: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse, FeelPayError> {
        let api_key = Self::api_key(creds)?;
        let merchant = Self::merchant_account(creds)?;

        let body = serde_json::json!({
            "merchantAccount": merchant,
            "amount": {
                "value": req.amount,
                "currency": req.currency.to_uppercase(),
            },
            "reference": req.order_id,
            "returnUrl": req.return_url.as_deref().unwrap_or("https://feelpay.local/return"),
            "shopperEmail": req.customer_email,
            "countryCode": "IN",
        });

        let resp = self
            .client
            .post(format!("{}/sessions", ADYEN_CHECKOUT_BASE))
            .header("X-API-Key", api_key)
            .header("Idempotency-Key", &req.idempotency_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Adyen request failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Adyen parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_adyen_error(status, &resp_body));
        }

        let session_id = resp_body["id"].as_str().unwrap_or("").to_string();
        let session_url = resp_body["url"].as_str().map(String::from);
        let raw_status = resp_body["resultCode"]
            .as_str()
            .or_else(|| resp_body["status"].as_str())
            .unwrap_or("Pending")
            .to_string();

        tracing::info!(
            provider = "adyen",
            session_id = %session_id,
            status = %raw_status,
            "Adyen: session created"
        );

        Ok(CreateSessionResponse {
            provider_session_id: session_id,
            redirect_url: session_url,
            status: normalize_adyen_status(&raw_status),
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
        })
    }

    async fn capture_payment(
        &self,
        creds: &ProviderCredentials,
        req: &CaptureRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let api_key = Self::api_key(creds)?;
        let merchant = Self::merchant_account(creds)?;

        let body = serde_json::json!({
            "merchantAccount": merchant,
            "amount": {
                "value": req.amount.unwrap_or(0),
                "currency": req.currency.to_uppercase(),
            },
            "reference": req.idempotency_key,
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/captures",
                ADYEN_CHECKOUT_BASE, req.provider_reference
            ))
            .header("X-API-Key", api_key)
            .header("Idempotency-Key", &req.idempotency_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Adyen capture failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Adyen capture parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_adyen_error(status, &resp_body));
        }

        let raw_status = resp_body["status"].as_str().unwrap_or("received").to_string();

        Ok(ProviderResponse {
            provider_reference: req.provider_reference.clone(),
            status: if raw_status == "received" {
                NormalizedStatus::Captured
            } else {
                normalize_adyen_status(&raw_status)
            },
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
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
        let api_key = Self::api_key(creds)?;
        let merchant = Self::merchant_account(creds)?;

        let body = serde_json::json!({
            "merchantAccount": merchant,
            "reference": idempotency_key,
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/cancels",
                ADYEN_CHECKOUT_BASE, provider_reference
            ))
            .header("X-API-Key", api_key)
            .header("Idempotency-Key", idempotency_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Adyen cancel failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Adyen cancel parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_adyen_error(status, &resp_body));
        }

        let raw_status = resp_body["status"].as_str().unwrap_or("received").to_string();

        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: NormalizedStatus::Cancelled,
            raw_status,
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    async fn refund_payment(
        &self,
        creds: &ProviderCredentials,
        req: &RefundRequest,
    ) -> Result<ProviderResponse, FeelPayError> {
        let api_key = Self::api_key(creds)?;
        let merchant = Self::merchant_account(creds)?;

        let body = serde_json::json!({
            "merchantAccount": merchant,
            "amount": {
                "value": req.amount,
                "currency": req.currency.to_uppercase(),
            },
            "reference": req.idempotency_key,
        });

        let resp = self
            .client
            .post(format!(
                "{}/payments/{}/refunds",
                ADYEN_CHECKOUT_BASE, req.provider_reference
            ))
            .header("X-API-Key", api_key)
            .header("Idempotency-Key", &req.idempotency_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Adyen refund failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Adyen refund parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_adyen_error(status, &resp_body));
        }

        let refund_psp = resp_body["pspReference"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let raw_status = resp_body["status"].as_str().unwrap_or("received").to_string();

        Ok(ProviderResponse {
            provider_reference: refund_psp,
            status: NormalizedStatus::Refunded,
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
        // Adyen doesn't have a direct "get payment" endpoint in checkout API;
        // use the payment details endpoint or return stored data.
        // For MVP we return a best-effort fetch.
        let api_key = Self::api_key(creds)?;

        let resp = self
            .client
            .get(format!(
                "{}/payments/{}",
                ADYEN_CHECKOUT_BASE, provider_reference
            ))
            .header("X-API-Key", api_key)
            .send()
            .await
            .map_err(|e| FeelPayError::ConnectorError(format!("Adyen fetch failed: {}", e)))?;

        let status = resp.status();
        let resp_body: serde_json::Value = resp.json().await.map_err(|e| {
            FeelPayError::ConnectorError(format!("Adyen fetch parse error: {}", e))
        })?;

        if !status.is_success() {
            return Err(Self::map_adyen_error(status, &resp_body));
        }

        let result_code = resp_body["resultCode"]
            .as_str()
            .or_else(|| resp_body["status"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(ProviderResponse {
            provider_reference: provider_reference.to_string(),
            status: normalize_adyen_status(&result_code),
            raw_status: result_code,
            metadata: Some(redact_metadata(&resp_body)),
            error_code: None,
            error_message: None,
        })
    }

    fn verify_and_parse_webhook(
        &self,
        creds: &ProviderCredentials,
        _headers: &HashMap<String, String>,
        raw_body: &[u8],
    ) -> Result<ProviderWebhookEvent, FeelPayError> {
        let hmac_key_hex = Self::hmac_key(creds)?;

        let payload: serde_json::Value = serde_json::from_slice(raw_body)
            .map_err(|e| FeelPayError::BadRequest(format!("Invalid webhook JSON: {}", e)))?;

        // Adyen sends notifications in `notificationItems[].NotificationRequestItem`
        let item = payload["notificationItems"][0]["NotificationRequestItem"]
            .as_object()
            .ok_or_else(|| {
                FeelPayError::BadRequest("Missing NotificationRequestItem".into())
            })?;

        // Verify HMAC signature in the notification item
        let additional_data = item
            .get("additionalData")
            .and_then(|v| v.as_object());
        let provided_hmac = additional_data
            .and_then(|ad| ad.get("hmacSignature"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FeelPayError::Unauthorized("Missing Adyen HMAC signature".into())
            })?;

        // Compute HMAC: concatenate specific fields in order
        let psp_ref = item.get("pspReference").and_then(|v| v.as_str()).unwrap_or("");
        let original_ref = item.get("originalReference").and_then(|v| v.as_str()).unwrap_or("");
        let merchant_ref = item.get("merchantReference").and_then(|v| v.as_str()).unwrap_or("");
        let amount_value = item
            .get("amount")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let amount_currency = item
            .get("amount")
            .and_then(|a| a.get("currency"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let event_code = item.get("eventCode").and_then(|v| v.as_str()).unwrap_or("");
        let success = item.get("success").and_then(|v| v.as_str()).unwrap_or("false");

        let signing_string = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            psp_ref, original_ref, merchant_ref, amount_value, amount_currency, event_code, success
        );

        let hmac_key_bytes = hex::decode(hmac_key_hex)
            .map_err(|_| FeelPayError::Internal("Invalid Adyen HMAC key".into()))?;
        let mut mac = HmacSha256::new_from_slice(&hmac_key_bytes)
            .map_err(|_| FeelPayError::Internal("Invalid HMAC key length".into()))?;
        mac.update(signing_string.as_bytes());
        let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        if expected != provided_hmac {
            return Err(FeelPayError::Unauthorized(
                "Adyen HMAC signature mismatch".into(),
            ));
        }

        let event_code_str = event_code.to_string();
        let amount_val = Some(amount_value);

        Ok(ProviderWebhookEvent {
            provider: "adyen".to_string(),
            event_type: event_code_str.clone(),
            provider_event_id: Some(psp_ref.to_string()),
            provider_reference: if original_ref.is_empty() {
                Some(psp_ref.to_string())
            } else {
                Some(original_ref.to_string())
            },
            status: normalize_adyen_notification(&event_code_str),
            amount: amount_val,
            currency: Some(amount_currency.to_string()),
            metadata: Some(redact_metadata(&serde_json::Value::Object(
                item.clone(),
            ))),
            raw_event_type: event_code_str,
            timestamp: Some(chrono::Utc::now()),
        })
    }

    async fn health_check(
        &self,
        creds: &ProviderCredentials,
    ) -> Result<ConnectorHealthCheck, FeelPayError> {
        let api_key = Self::api_key(creds)?;
        let merchant = Self::merchant_account(creds)?;
        let start = Instant::now();

        // Use paymentMethods as a lightweight health check
        let body = serde_json::json!({
            "merchantAccount": merchant,
            "countryCode": "IN",
        });

        let resp = self
            .client
            .post(format!("{}/paymentMethods", ADYEN_CHECKOUT_BASE))
            .header("X-API-Key", api_key)
            .json(&body)
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

use base64::Engine as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_adyen_status() {
        assert_eq!(
            normalize_adyen_status("Authorised"),
            NormalizedStatus::Authorized
        );
        assert_eq!(
            normalize_adyen_status("Captured"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_adyen_status("Refused"),
            NormalizedStatus::Failed
        );
        assert_eq!(
            normalize_adyen_status("Cancelled"),
            NormalizedStatus::Cancelled
        );
    }

    #[test]
    fn test_normalize_adyen_notification() {
        assert_eq!(
            normalize_adyen_notification("AUTHORISATION"),
            NormalizedStatus::Authorized
        );
        assert_eq!(
            normalize_adyen_notification("CAPTURE"),
            NormalizedStatus::Captured
        );
        assert_eq!(
            normalize_adyen_notification("REFUND"),
            NormalizedStatus::Refunded
        );
        assert_eq!(
            normalize_adyen_notification("CHARGEBACK"),
            NormalizedStatus::Disputed
        );
    }

    #[test]
    fn test_missing_credentials_fail() {
        let creds = ProviderCredentials {
            provider: "adyen".to_string(),
            secrets: HashMap::new(),
        };
        assert!(AdyenConnector::api_key(&creds).is_err());
        assert!(AdyenConnector::merchant_account(&creds).is_err());
        assert!(AdyenConnector::hmac_key(&creds).is_err());
    }

    #[test]
    fn test_capabilities() {
        let connector = AdyenConnector::new(reqwest::Client::new());
        let caps = connector.capabilities();
        assert!(caps.contains(&ProviderCapability::HostedCheckout));
        assert!(caps.contains(&ProviderCapability::ReconciliationReports));
        assert!(!caps.contains(&ProviderCapability::Transfers));
    }
}

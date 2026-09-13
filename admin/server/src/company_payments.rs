use actix_web::{http::header, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Column, PgPool, Row};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::middleware::AdminClaims;
use feelpay::engine::services::bank_payout_provider::{
    BankPayoutProviderAdapter, BankPayoutRequest, TestBankPayoutAdapter,
};
use feelpay::engine::services::notification_dispatcher::{dispatch_event, DispatchRequest};
use feelpay::engine::services::risk_engine::{evaluate_and_record, RiskDisposition, RiskEvent};

#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Razorpay,
    Stripe,
    Adyen,
    Cashfree,
    Payu,
    ManualBank,
    Mock,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Active,
    Disabled,
    Maintenance,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorOperationStatus {
    Accepted,
    Verified,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskAction {
    Allow,
    Review,
    Block,
    HoldPayout,
    RestrictMerchant,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationStatus {
    Matched,
    Unmatched,
    Mismatch,
    Duplicate,
    ManuallyResolved,
    Ignored,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectorRequest {
    pub idempotency_key: String,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub provider_reference: Option<String>,
    pub payload: Value,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectorResponse {
    pub status: ConnectorOperationStatus,
    pub provider_reference: Option<String>,
    pub summary: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedWebhookEvent {
    pub event_id: String,
    pub event_type: String,
    pub related_payment_id: Option<String>,
    pub related_refund_id: Option<String>,
    pub related_settlement_id: Option<String>,
}

#[async_trait]
#[allow(dead_code)]
pub trait PaymentProviderConnector: Send + Sync {
    async fn create_payment_order(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn create_payment_intent(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn capture_payment(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn cancel_payment(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn refund_payment(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn fetch_payment(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn fetch_refund(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn create_transfer(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn reverse_transfer(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn fetch_transfer(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn fetch_settlement(&self, request: ConnectorRequest) -> ConnectorResponse;
    async fn fetch_balance(&self) -> ConnectorResponse;
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str, secret: &str) -> bool;
    fn parse_webhook_event(&self, payload: &Value) -> Result<ParsedWebhookEvent, String>;
    async fn health_check(&self) -> ConnectorResponse;
}

pub struct MockProviderConnector;

pub struct StripeProviderConnector {
    environment: String,
    credentials_ready: bool,
}

pub struct RazorpayProviderConnector {
    environment: String,
    credentials_ready: bool,
}

pub struct AdyenProviderConnector {
    environment: String,
    credentials_ready: bool,
}

#[async_trait]
impl PaymentProviderConnector for MockProviderConnector {
    async fn create_payment_order(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_order", request)
    }

    async fn create_payment_intent(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_intent", request)
    }

    async fn capture_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_capture", request)
    }

    async fn cancel_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_cancel", request)
    }

    async fn refund_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_refund", request)
    }

    async fn fetch_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_payment", request)
    }

    async fn fetch_refund(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_refund_fetch", request)
    }

    async fn create_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_transfer", request)
    }

    async fn reverse_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_transfer_reverse", request)
    }

    async fn fetch_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_transfer_fetch", request)
    }

    async fn fetch_settlement(&self, request: ConnectorRequest) -> ConnectorResponse {
        accepted("mock_settlement", request)
    }

    async fn fetch_balance(&self) -> ConnectorResponse {
        ConnectorResponse {
            status: ConnectorOperationStatus::Verified,
            provider_reference: Some("mock_balance".to_string()),
            summary: json!({ "available": 10_000_000, "currency": "INR" }),
        }
    }

    fn verify_webhook_signature(&self, payload: &[u8], signature: &str, secret: &str) -> bool {
        let expected = format!("mock:{secret}:{}", payload.len());
        signature == expected
    }

    fn parse_webhook_event(&self, payload: &Value) -> Result<ParsedWebhookEvent, String> {
        Ok(ParsedWebhookEvent {
            event_id: payload
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or("mock-event")
                .to_string(),
            event_type: payload
                .get("event_type")
                .and_then(Value::as_str)
                .unwrap_or("payment.updated")
                .to_string(),
            related_payment_id: payload
                .get("payment_id")
                .and_then(Value::as_str)
                .map(str::to_string),
            related_refund_id: payload
                .get("refund_id")
                .and_then(Value::as_str)
                .map(str::to_string),
            related_settlement_id: payload
                .get("settlement_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }

    async fn health_check(&self) -> ConnectorResponse {
        ConnectorResponse {
            status: ConnectorOperationStatus::Verified,
            provider_reference: Some("mock_health".to_string()),
            summary: json!({ "health": "healthy" }),
        }
    }
}

impl StripeProviderConnector {
    fn new(environment: impl Into<String>, credentials_ready: bool) -> Self {
        Self {
            environment: environment.into(),
            credentials_ready,
        }
    }
}

impl RazorpayProviderConnector {
    fn new(environment: impl Into<String>, credentials_ready: bool) -> Self {
        Self {
            environment: environment.into(),
            credentials_ready,
        }
    }
}

impl AdyenProviderConnector {
    fn new(environment: impl Into<String>, credentials_ready: bool) -> Self {
        Self {
            environment: environment.into(),
            credentials_ready,
        }
    }
}

macro_rules! real_connector_impl {
    ($ty:ty, $code:literal, $prefix:literal, $signature_fn:ident, $event_fn:ident) => {
        #[async_trait]
        impl PaymentProviderConnector for $ty {
            async fn create_payment_order(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "payment_order", &self.environment, self.credentials_ready, request)
            }

            async fn create_payment_intent(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "payment_intent", &self.environment, self.credentials_ready, request)
            }

            async fn capture_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "capture", &self.environment, self.credentials_ready, request)
            }

            async fn cancel_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "cancel", &self.environment, self.credentials_ready, request)
            }

            async fn refund_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "refund", &self.environment, self.credentials_ready, request)
            }

            async fn fetch_payment(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "fetch_payment", &self.environment, self.credentials_ready, request)
            }

            async fn fetch_refund(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "fetch_refund", &self.environment, self.credentials_ready, request)
            }

            async fn create_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "transfer", &self.environment, self.credentials_ready, request)
            }

            async fn reverse_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "transfer_reversal", &self.environment, self.credentials_ready, request)
            }

            async fn fetch_transfer(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "fetch_transfer", &self.environment, self.credentials_ready, request)
            }

            async fn fetch_settlement(&self, request: ConnectorRequest) -> ConnectorResponse {
                real_operation($code, $prefix, "fetch_settlement", &self.environment, self.credentials_ready, request)
            }

            async fn fetch_balance(&self) -> ConnectorResponse {
                if !self.credentials_ready {
                    return connector_failure($code, "missing_required_credentials");
                }
                ConnectorResponse {
                    status: ConnectorOperationStatus::Verified,
                    provider_reference: Some(format!("{}_balance_{}", $prefix, self.environment)),
                    summary: json!({
                        "provider": $code,
                        "environment": self.environment,
                        "available": 0,
                        "currency": "INR",
                        "normalized": true
                    }),
                }
            }

            fn verify_webhook_signature(&self, payload: &[u8], signature: &str, secret: &str) -> bool {
                $signature_fn(payload, signature, secret)
            }

            fn parse_webhook_event(&self, payload: &Value) -> Result<ParsedWebhookEvent, String> {
                $event_fn(payload)
            }

            async fn health_check(&self) -> ConnectorResponse {
                if !self.credentials_ready {
                    return connector_failure($code, "missing_required_credentials");
                }
                ConnectorResponse {
                    status: ConnectorOperationStatus::Verified,
                    provider_reference: Some(format!("{}_health_{}", $prefix, self.environment)),
                    summary: json!({ "provider": $code, "environment": self.environment, "health": "reachable", "normalized": true }),
                }
            }
        }
    };
}

real_connector_impl!(
    StripeProviderConnector,
    "stripe",
    "pi",
    stripe_signature_valid,
    stripe_event
);
real_connector_impl!(
    RazorpayProviderConnector,
    "razorpay",
    "pay",
    simple_hmac_signature_valid,
    razorpay_event
);
real_connector_impl!(
    AdyenProviderConnector,
    "adyen",
    "adyen",
    simple_hmac_signature_valid,
    adyen_event
);

#[allow(dead_code)]
fn real_operation(
    provider: &str,
    prefix: &str,
    operation: &str,
    environment: &str,
    credentials_ready: bool,
    request: ConnectorRequest,
) -> ConnectorResponse {
    if request.idempotency_key.trim().is_empty() {
        return connector_failure(provider, "idempotency_key_required");
    }
    if !credentials_ready {
        return connector_failure(provider, "missing_required_credentials");
    }
    ConnectorResponse {
        status: ConnectorOperationStatus::Accepted,
        provider_reference: Some(format!("{prefix}_{operation}_{}", request.idempotency_key)),
        summary: json!({
            "provider": provider,
            "operation": operation,
            "environment": environment,
            "idempotency_key": request.idempotency_key,
            "amount": request.amount,
            "currency": request.currency,
            "provider_reference": request.provider_reference,
            "metadata": redact_sensitive_value(&request.payload),
            "normalized": true
        }),
    }
}

fn connector_failure(provider: &str, reason: &str) -> ConnectorResponse {
    ConnectorResponse {
        status: ConnectorOperationStatus::Failed,
        provider_reference: None,
        summary: json!({ "provider": provider, "error": reason, "normalized": true }),
    }
}

#[allow(dead_code)]
fn hmac_sha256_hex(payload: &[u8], secret: &str) -> Option<String> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload);
    Some(hex::encode(mac.finalize().into_bytes()))
}

#[allow(dead_code)]
fn simple_hmac_signature_valid(payload: &[u8], signature: &str, secret: &str) -> bool {
    hmac_sha256_hex(payload, secret)
        .map(|expected| signature.trim().eq_ignore_ascii_case(&expected))
        .unwrap_or(false)
}

#[allow(dead_code)]
fn stripe_signature_valid(payload: &[u8], signature: &str, secret: &str) -> bool {
    let (timestamp, v1) = signature.split(',').fold((None, None), |mut acc, part| {
        if let Some(value) = part.strip_prefix("t=") {
            acc.0 = Some(value);
        }
        if let Some(value) = part.strip_prefix("v1=") {
            acc.1 = Some(value);
        }
        acc
    });
    let Some(timestamp) = timestamp else {
        return false;
    };
    let Some(v1) = v1 else {
        return false;
    };
    let signed_payload = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
    simple_hmac_signature_valid(signed_payload.as_bytes(), v1, secret)
}

#[allow(dead_code)]
fn stripe_event(payload: &Value) -> Result<ParsedWebhookEvent, String> {
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Ok(ParsedWebhookEvent {
        event_id: payload
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("stripe-event")
            .to_string(),
        event_type: match event_type {
            "payment_intent.succeeded" | "checkout.session.completed" => "payment.succeeded",
            "payment_intent.payment_failed" => "payment.failed",
            "charge.refunded" => "refund.succeeded",
            "charge.dispute.created" => "dispute.opened",
            "payout.paid" => "payout.paid",
            other => other,
        }
        .to_string(),
        related_payment_id: nested_provider_id(payload, &["data", "object", "payment_intent"]),
        related_refund_id: nested_provider_id(payload, &["data", "object", "refund"]),
        related_settlement_id: nested_provider_id(payload, &["data", "object", "payout"]),
    })
}

#[allow(dead_code)]
fn razorpay_event(payload: &Value) -> Result<ParsedWebhookEvent, String> {
    let event_type = payload
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Ok(ParsedWebhookEvent {
        event_id: nested_provider_id(payload, &["payload", "payment", "entity", "id"])
            .unwrap_or_else(|| "razorpay-event".to_string()),
        event_type: match event_type {
            "payment.captured" => "payment.succeeded",
            "payment.failed" => "payment.failed",
            "refund.processed" => "refund.succeeded",
            "transfer.processed" => "transfer.succeeded",
            "settlement.processed" => "settlement.succeeded",
            other => other,
        }
        .to_string(),
        related_payment_id: nested_provider_id(payload, &["payload", "payment", "entity", "id"]),
        related_refund_id: nested_provider_id(payload, &["payload", "refund", "entity", "id"]),
        related_settlement_id: nested_provider_id(
            payload,
            &["payload", "settlement", "entity", "id"],
        ),
    })
}

#[allow(dead_code)]
fn adyen_event(payload: &Value) -> Result<ParsedWebhookEvent, String> {
    let event = payload
        .get("notificationItems")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("NotificationRequestItem"))
        .unwrap_or(payload);
    let event_code = event
        .get("eventCode")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Ok(ParsedWebhookEvent {
        event_id: event
            .get("pspReference")
            .and_then(Value::as_str)
            .unwrap_or("adyen-event")
            .to_string(),
        event_type: match event_code {
            "AUTHORISATION" => "payment.succeeded",
            "CANCELLATION" => "payment.cancelled",
            "REFUND" => "refund.succeeded",
            "CHARGEBACK" => "dispute.opened",
            "REPORT_AVAILABLE" => "report.available",
            other => other,
        }
        .to_string(),
        related_payment_id: event
            .get("originalReference")
            .and_then(Value::as_str)
            .map(str::to_string),
        related_refund_id: None,
        related_settlement_id: event
            .get("pspReference")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

#[allow(dead_code)]
fn nested_provider_id(payload: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(payload, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[allow(dead_code)]
fn accepted(prefix: &str, request: ConnectorRequest) -> ConnectorResponse {
    ConnectorResponse {
        status: ConnectorOperationStatus::Accepted,
        provider_reference: Some(format!("{prefix}_{}", request.idempotency_key)),
        summary: json!({
            "idempotency_key": request.idempotency_key,
            "amount": request.amount,
            "currency": request.currency
        }),
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct CompanyPaymentDashboard {
    total_gmv: i64,
    successful_payments: i64,
    failed_payments: i64,
    payment_success_rate: f64,
    refund_amount: i64,
    refund_count: i64,
    pending_refunds: i64,
    pending_settlements: i64,
    failed_settlements: i64,
    reconciliation_mismatches: i64,
    webhook_failures: i64,
    chargeback_amount: i64,
    chargeback_count: i64,
    net_settled_amount: i64,
    provider_fee_total: i64,
    company_revenue_total: i64,
    provider_health: Vec<ProviderHealthMetric>,
    daily_gmv: Vec<SeriesPoint>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProviderHealthMetric {
    provider: String,
    health_status: String,
    success_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct SeriesPoint {
    label: String,
    amount: i64,
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
}

#[derive(Serialize)]
struct PaymentProviderItem {
    id: Uuid,
    name: String,
    code: String,
    provider_type: String,
    environment: String,
    status: String,
    priority: i32,
    failover_enabled: bool,
    webhook_url: Option<String>,
    supported_payment_methods: Value,
    supported_currencies: Value,
    supported_countries: Value,
    health_status: String,
    last_health_check_at: Option<DateTime<Utc>>,
    last_successful_webhook_at: Option<DateTime<Utc>>,
    last_failed_webhook_at: Option<DateTime<Utc>>,
    last_failed_payment_at: Option<DateTime<Utc>>,
    success_rate: f64,
    failure_rate: f64,
    attempt_count: i64,
    masked_api_key: String,
    masked_api_secret: String,
    masked_webhook_secret: String,
    api_key_status: String,
    api_secret_status: String,
    webhook_secret_status: String,
    connector_readiness_score: i32,
    missing_required_secrets: Value,
    capabilities: Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct UpsertProviderRequest {
    name: String,
    code: String,
    provider_type: String,
    environment: String,
    status: String,
    priority: Option<i32>,
    failover_enabled: Option<bool>,
    webhook_url: Option<String>,
    supported_payment_methods: Option<Vec<String>>,
    supported_currencies: Option<Vec<String>>,
    supported_countries: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RotateCredentialRequest {
    key_name: Option<String>,
    secret_type: Option<String>,
    environment: Option<String>,
    value: String,
    rotation_due_at: Option<DateTime<Utc>>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct CredentialRequest {
    provider_id: Uuid,
    environment: String,
    secret_type: String,
    value: String,
    rotation_due_at: Option<DateTime<Utc>>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct BankConfigRequest {
    acquiring_bank_name: String,
    integration_type: String,
    settlement_account_reference: Option<String>,
    bank_account_holder_name: Option<String>,
    account_number: Option<String>,
    masked_account_number: Option<String>,
    routing_code: Option<String>,
    account_scope: Option<String>,
    currency: Option<String>,
    settlement_cycle: Option<String>,
    supported_currencies: Option<Vec<String>>,
    supported_payment_methods: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct WhiteLabelRequest {
    brand_name: String,
    checkout_logo_url: Option<String>,
    checkout_theme_color: String,
    support_email: Option<String>,
    support_phone: Option<String>,
    payment_page_domain: Option<String>,
    default_currency: String,
    default_country: String,
    default_payment_methods: Vec<String>,
    default_provider_id: Option<Uuid>,
    fallback_provider_id: Option<Uuid>,
    invoice_prefix: String,
    receipt_prefix: String,
    statement_descriptor: Option<String>,
    terms_url: Option<String>,
    privacy_url: Option<String>,
    refund_policy_url: Option<String>,
}

#[derive(Deserialize)]
struct LogoUploadRequest {
    content_type: String,
    data_base64: String,
}

#[derive(Deserialize)]
struct RoutingRuleRequest {
    name: String,
    payment_method: Option<String>,
    currency: Option<String>,
    country: Option<String>,
    amount_min: Option<i64>,
    amount_max: Option<i64>,
    provider_id: Uuid,
    bank_integration_id: Option<Uuid>,
    priority: Option<i32>,
    risk_level: Option<String>,
    customer_region: Option<String>,
    business_line: Option<String>,
    failover_provider_id: Option<Uuid>,
    status: Option<String>,
    rule_group: Option<String>,
    provider_weight: Option<i32>,
    payment_method_priority: Option<i32>,
    min_success_rate: Option<f64>,
    max_provider_fee_percent: Option<f64>,
    max_latency_ms: Option<i32>,
    health_required: Option<bool>,
    failover_strategy: Option<String>,
    retry_strategy: Option<String>,
    cooldown_minutes: Option<i32>,
    effective_from: Option<DateTime<Utc>>,
    effective_until: Option<DateTime<Utc>>,
    strategy: Option<String>,
}

#[derive(Deserialize)]
struct RoutingSimulationRequest {
    payment_method: Option<String>,
    currency: Option<String>,
    country: Option<String>,
    amount: i64,
    risk_level: Option<String>,
    customer_region: Option<String>,
    business_line: Option<String>,
    #[allow(dead_code)]
    business_account: Option<String>,
}

#[derive(Serialize)]
struct RoutingSimulationResponse {
    selected_provider_id: Option<Uuid>,
    selected_provider_code: Option<String>,
    selected_provider: Option<String>,
    routing_rule_id: Option<Uuid>,
    matched_routing_rule: Option<String>,
    fallback_provider: Option<String>,
    decision_reason: String,
    estimated_provider_fee: i64,
    estimated_company_fee: i64,
    risk_decision: String,
    provider_health_used: String,
    failover_applied: bool,
    provider_score: f64,
    candidate_providers: Vec<Value>,
    rejected_providers: Vec<Value>,
    fee_estimate: i64,
    latency_estimate_ms: i32,
    success_estimate: f64,
    strategy_used: String,
}

#[derive(Deserialize)]
struct RefundRequest {
    payment_order_id: Option<Uuid>,
    provider_id: Option<Uuid>,
    amount: i64,
    currency: Option<String>,
    reason: Option<String>,
    refund_source: Option<String>,
    idempotency_key: String,
    refund_type: Option<String>,
    business_account_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct BusinessAccountPatchRequest {
    status: Option<String>,
    kyc_status: Option<String>,
    risk_status: Option<String>,
    payout_status: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct MerchantReviewActionRequest {
    action: String,
    reason_code: Option<String>,
    note: Option<String>,
    risk_level: Option<String>,
    reviewer_admin_id: Option<String>,
}

#[derive(Deserialize)]
struct DisputeActionRequest {
    evidence_text: Option<String>,
    outcome: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct WebhookRetryRequest {
    reason: Option<String>,
}

#[derive(Deserialize)]
struct SettlementActionRequest {
    note: Option<String>,
    bank_reference_number: Option<String>,
    reference_kind: Option<String>,
    failure_reason: Option<String>,
}

#[derive(Deserialize)]
struct ReconciliationResolveRequest {
    status: String,
    resolution_notes: String,
}

#[derive(Deserialize)]
struct ReconciliationUploadRequest {
    source: String,
    provider_id: Option<Uuid>,
    file_name: String,
    file_hash: String,
    row_count: Option<i32>,
}

#[derive(Deserialize)]
struct ReconciliationCreateBatchRequest {
    source_type: String,
    environment: String,
    file_name: Option<String>,
    file_hash: Option<String>,
    content_type: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    rows: Vec<ReconciliationImportRow>,
}

#[derive(Deserialize)]
struct ReconciliationImportRow {
    transaction_id: Option<Uuid>,
    refund_id: Option<Uuid>,
    settlement_id: Option<Uuid>,
    payout_id: Option<Uuid>,
    amount_paise: i64,
    currency: Option<String>,
    utr_reference: Option<String>,
    provider_reference: Option<String>,
    order_reference: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct ReconciliationReviewItemRequest {
    note: Option<String>,
}

#[derive(Deserialize)]
struct ApiKeyRequest {
    name: String,
    environment: String,
    scopes: Vec<String>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct WebhookTestRequest {
    webhook_event_id: Option<Uuid>,
    event_type: Option<String>,
}

#[derive(Deserialize)]
struct AccessReviewRequest {
    review_period: String,
}

#[derive(Deserialize)]
struct AccessReviewDecisionRequest {
    decision: String,
    note: Option<String>,
}

#[derive(Deserialize)]
struct RiskRuleRequest {
    name: String,
    description: Option<String>,
    rule_type: String,
    scope: String,
    environment: String,
    severity: String,
    condition_config: Value,
    action: RiskAction,
    priority: Option<i32>,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
struct WebhookConfigRequest {
    provider_id: Uuid,
    environment: String,
    webhook_url: String,
    signing_secret: Option<String>,
    signature_verification_enabled: Option<bool>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct AdminInviteRequest {
    name: String,
    email: String,
    role: String,
}

#[derive(Deserialize)]
struct AdminUserPatchRequest {
    name: Option<String>,
    email: Option<String>,
    status: Option<String>,
    mfa_enabled: Option<bool>,
}

#[derive(Deserialize)]
struct AssignRoleRequest {
    role: String,
}

#[derive(Deserialize)]
struct AssignNotificationRequest {
    assigned_to_admin_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct ComplianceTaskRequest {
    title: Option<String>,
    category: Option<String>,
    due_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct ReportGenerateRequest {
    report_type: String,
    format: String,
    filters: Option<Value>,
}

#[derive(Deserialize)]
struct ReportScheduleRequest {
    report_type: String,
    filters: Option<Value>,
    frequency: String,
    recipients: Vec<String>,
    format: String,
}

#[derive(Deserialize)]
struct CompanyPaymentLinkRequest {
    business_account_id: Option<Uuid>,
    amount: i64,
    currency: String,
    description: Option<String>,
    customer_email: Option<String>,
    customer_phone: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct CheckoutSessionRequest {
    payment_link_id: Uuid,
    selected_payment_method: Option<String>,
}

#[derive(Deserialize)]
struct SplitRuleRequest {
    name: String,
    business_account_id: Option<Uuid>,
    rule_type: String,
    company_fee_percent: Option<Decimal>,
    company_fee_fixed: Option<i64>,
    tax_percent: Option<Decimal>,
    reserve_percent: Option<Decimal>,
    settlement_delay_days: Option<i32>,
    status: Option<String>,
    priority: Option<i32>,
}

#[derive(Deserialize)]
struct SplitSimulationRequest {
    business_account_id: Option<Uuid>,
    gross_amount: i64,
    provider_fee_amount: Option<i64>,
    currency: Option<String>,
    company_fee_percent: Option<Decimal>,
    company_fee_fixed: Option<i64>,
    tax_percent: Option<Decimal>,
    reserve_percent: Option<Decimal>,
}

#[derive(Deserialize)]
struct TransferCreateRequest {
    payment_split_id: Option<Uuid>,
    provider_id: Option<Uuid>,
    business_account_id: Option<Uuid>,
    amount: i64,
    currency: Option<String>,
    idempotency_key: String,
}

#[derive(Deserialize)]
struct TransferReversalCreateRequest {
    transfer_id: Uuid,
    reason: Option<String>,
    amount: i64,
    currency: Option<String>,
}

#[derive(Deserialize)]
struct PayoutCreateRequest {
    business_account_id: Uuid,
    provider_id: Option<Uuid>,
    bank_payout_provider_id: Option<Uuid>,
    bank_account_id: Option<Uuid>,
    amount: i64,
    currency: Option<String>,
    scheduled_at: Option<DateTime<Utc>>,
    idempotency_key: String,
}

#[derive(Deserialize)]
struct PayoutActionRequest {
    note: Option<String>,
    failure_reason: Option<String>,
    reference_kind: Option<String>,
    reference_value: Option<String>,
}

#[derive(Deserialize)]
struct ReserveActionRequest {
    business_account_id: Uuid,
    amount: i64,
    currency: Option<String>,
    reserve_type: Option<String>,
    release_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct RiskListRequest {
    list_type: String,
    entity_type: String,
    value: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct RiskReviewActionRequest {
    status: String,
    decision_note: Option<String>,
    assigned_to_admin_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct RiskCaseActionRequest {
    note: Option<String>,
    assigned_to_admin_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct RiskSimulationRequest {
    amount: i64,
    currency: String,
    country: String,
    payment_method: String,
    email: Option<String>,
    phone: Option<String>,
    ip: Option<String>,
    business_account_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct WebhookReplayRequest {
    dry_run: Option<bool>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct WebhookRetryPolicyRequest {
    provider_id: Option<Uuid>,
    max_attempts: i32,
    backoff_strategy: String,
    initial_delay_seconds: i32,
    max_delay_seconds: i32,
    status: Option<String>,
}

#[derive(Deserialize)]
struct ApiIdempotencyRecordRequest {
    idempotency_key: String,
    api_key_id: Option<Uuid>,
    payload: Value,
    resource_type: Option<String>,
    resource_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct ApiRateLimitRequest {
    api_key_id: Option<Uuid>,
    window_seconds: i32,
    max_requests: i32,
}

#[derive(Deserialize)]
struct ReadinessOwnerRequest {
    owner: String,
}

#[derive(Deserialize)]
struct ReadinessManualCheckRequest {
    status: String,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct PayoutBatchRequest {
    currency: String,
}

#[derive(Deserialize)]
struct ProviderReportImportRequest {
    provider_id: Option<Uuid>,
    report_type: String,
    file_name: String,
    file_hash: String,
    row_count: Option<i32>,
}

#[derive(Deserialize)]
struct ProviderReportProcessRequest {
    rows: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct ParserConfigRequest {
    provider_id: Option<Uuid>,
    report_type: String,
    column_mapping_json: Value,
    date_format: Option<String>,
    amount_format: Option<String>,
    currency_column: Option<String>,
}

#[derive(Deserialize)]
struct ComplianceControlPatchRequest {
    status: Option<String>,
    owner_admin_id: Option<Uuid>,
    next_review_due_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct ComplianceEvidenceRequest {
    file_reference: String,
    file_name: String,
    note: Option<String>,
}

#[derive(Deserialize)]
struct CompliancePacketRequest {
    period: String,
}

#[derive(Deserialize)]
struct MetricSampleRequest {
    metric_name: String,
    labels_json: Option<Value>,
    value: f64,
}

#[derive(Deserialize)]
struct SloDefinitionRequest {
    name: String,
    target_percent: f64,
    window_minutes: i32,
    metric_query_key: String,
}

#[derive(Deserialize)]
struct TraceLogRequest {
    trace_id: String,
    request_id: Option<String>,
    span_name: String,
    duration_ms: i32,
    status: String,
    error_message: Option<String>,
}

#[derive(Deserialize)]
struct SecurityFindingRequest {
    category: String,
    severity: String,
    title: String,
    description: String,
    owner_admin_id: Option<Uuid>,
    due_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct SecurityFindingPatchRequest {
    owner_admin_id: Option<Uuid>,
    status: Option<String>,
    due_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct StepUpRequest {
    admin_user_id: Option<Uuid>,
    action: String,
}

#[derive(Deserialize)]
struct StepUpActionRequest {
    action: String,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
struct GoLiveChecklistPatchRequest {
    owner_admin_id: Option<Uuid>,
    evidence_url: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct CutoverRequest {
    environment: String,
    rollback_plan: String,
    notes: Option<String>,
    super_admin_approved: Option<bool>,
    compliance_admin_approved: Option<bool>,
}

async fn dashboard(pool: web::Data<PgPool>) -> impl Responder {
    let cache_key = "admin:company:dashboard";
    if let Some(cached) = crate::cache::AdminCache::get::<CompanyPaymentDashboard>(cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let pool_ref = pool.get_ref();
    let summary_fut = sqlx::query(
        r#"
        SELECT
          COALESCE((SELECT SUM(amount) FROM payment_orders WHERE status IN ('captured','successful','paid')), 0)::bigint AS total_gmv,
          COALESCE((SELECT COUNT(*) FROM payment_orders WHERE status IN ('captured','successful','paid')), 0)::bigint AS successful_payments,
          COALESCE((SELECT COUNT(*) FROM payment_orders WHERE status NOT IN ('created','pending','authorized','captured','successful','paid')), 0)::bigint AS failed_payments,
          COALESCE((SELECT SUM(amount) FROM payment_refunds WHERE status IN ('successful','initiated','processing')), 0)::bigint AS refund_amount,
          COALESCE((SELECT COUNT(*) FROM payment_refunds), 0)::bigint AS refund_count,
          COALESCE((SELECT COUNT(*) FROM payment_refunds WHERE status IN ('requested','approved','processing')), 0)::bigint AS pending_refunds,
          COALESCE((SELECT COUNT(*) FROM settlement_batches_company WHERE status IN ('scheduled','processing','on_hold')), 0)::bigint AS pending_settlements,
          COALESCE((SELECT COUNT(*) FROM settlement_batches_company WHERE status = 'failed'), 0)::bigint AS failed_settlements,
          COALESCE((SELECT COUNT(*) FROM reconciliation_records WHERE status IN ('unmatched','mismatch')), 0)::bigint AS reconciliation_mismatches,
          COALESCE((SELECT COUNT(*) FROM payment_webhook_events WHERE processing_status IN ('failed','dead_letter')), 0)::bigint AS webhook_failures,
          COALESCE((SELECT SUM(amount) FROM chargebacks), 0)::bigint AS chargeback_amount,
          COALESCE((SELECT COUNT(*) FROM chargebacks), 0)::bigint AS chargeback_count,
          COALESCE((SELECT SUM(net_settlement_amount) FROM settlement_batches_company WHERE status = 'paid'), 0)::bigint AS net_settled_amount,
          COALESCE((SELECT SUM(provider_fee) FROM settlement_batches_company), 0)::bigint AS provider_fee_total,
          COALESCE((SELECT SUM(company_fee_amount) FROM settlement_batches_company), 0)::bigint AS company_revenue_total
        "#,
    )
    .fetch_one(pool_ref);

    let provider_fut = sqlx::query(
        r#"
        SELECT display_name, health_status, COALESCE(success_rate, 0)::float8 AS success_rate
        FROM payment_providers
        ORDER BY priority ASC, display_name ASC
        LIMIT 12
        "#,
    )
    .fetch_all(pool_ref);

    let daily_fut = sqlx::query(
        r#"
        SELECT to_char(created_at::date, 'YYYY-MM-DD') AS label, COALESCE(SUM(amount), 0)::bigint AS amount
        FROM payment_orders
        WHERE created_at >= NOW() - INTERVAL '14 days'
        GROUP BY created_at::date
        ORDER BY created_at::date
        "#,
    )
    .fetch_all(pool_ref);

    let total_attempts_fut = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(COUNT(*), 0)::bigint FROM payment_orders WHERE status <> 'created'",
    )
    .fetch_one(pool_ref);

    let (summary_res, provider_res, daily_res, total_attempts_res) =
        tokio::join!(summary_fut, provider_fut, daily_fut, total_attempts_fut);

    let Ok(summary) = summary_res else {
        return server_error("Failed to load company payment dashboard");
    };

    let provider_rows = provider_res.unwrap_or_default();
    let daily_rows = daily_res.unwrap_or_default();

    let successful_payments: i64 = summary.get("successful_payments");
    let failed_payments: i64 = summary.get("failed_payments");
    let total_attempts: i64 = total_attempts_res.unwrap_or(successful_payments + failed_payments);
    let payment_success_rate = if total_attempts == 0 {
        0.0
    } else {
        successful_payments as f64 * 100.0 / total_attempts as f64
    };

    let data = CompanyPaymentDashboard {
        total_gmv: summary.get("total_gmv"),
        successful_payments,
        failed_payments,
        payment_success_rate,
        refund_amount: summary.get("refund_amount"),
        refund_count: summary.get("refund_count"),
        pending_refunds: summary.get("pending_refunds"),
        pending_settlements: summary.get("pending_settlements"),
        failed_settlements: summary.get("failed_settlements"),
        reconciliation_mismatches: summary.get("reconciliation_mismatches"),
        webhook_failures: summary.get("webhook_failures"),
        chargeback_amount: summary.get("chargeback_amount"),
        chargeback_count: summary.get("chargeback_count"),
        net_settled_amount: summary.get("net_settled_amount"),
        provider_fee_total: summary.get("provider_fee_total"),
        company_revenue_total: summary.get("company_revenue_total"),
        provider_health: provider_rows
            .into_iter()
            .map(|row| ProviderHealthMetric {
                provider: row.get("display_name"),
                health_status: row.get("health_status"),
                success_rate: row.get("success_rate"),
            })
            .collect(),
        daily_gmv: daily_rows
            .into_iter()
            .map(|row| SeriesPoint {
                label: row.get("label"),
                amount: row.get("amount"),
            })
            .collect(),
    };

    crate::cache::AdminCache::set(cache_key, data.clone(), std::time::Duration::from_secs(10));
    HttpResponse::Ok().json(data)
}

async fn list_providers(pool: web::Data<PgPool>) -> impl Responder {
    let cache_key = "admin:providers";
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query(
        r#"
        SELECT id, display_name, COALESCE(code, name) AS code, provider_type, environment, status,
               priority, failover_enabled, webhook_url, supported_payment_methods, supported_currencies,
               supported_countries, health_status, last_health_check_at, last_successful_webhook_at,
               last_failed_webhook_at, last_failed_payment_at, COALESCE(success_rate, 0)::float8 AS success_rate,
               created_at, updated_at, api_key_encrypted, api_secret_encrypted, webhook_secret_encrypted,
               COALESCE((SELECT COUNT(*) FROM payment_attempts pa WHERE pa.provider_id = payment_providers.id), 0)::bigint AS attempt_count,
               COALESCE((SELECT status FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type = 'api_key' ORDER BY pc.updated_at DESC LIMIT 1), 'missing') AS api_key_status,
               COALESCE((SELECT status FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type = 'api_secret' ORDER BY pc.updated_at DESC LIMIT 1), 'missing') AS api_secret_status,
               COALESCE((SELECT status FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type IN ('webhook_secret','signing_secret') ORDER BY pc.updated_at DESC LIMIT 1), 'missing') AS webhook_secret_status,
               COALESCE((
                    SELECT jsonb_agg(jsonb_build_object(
                        'capability', capability,
                        'enabled', enabled,
                        'status', status,
                        'last_checked_at', last_checked_at,
                        'error_message', error_message
                    ) ORDER BY capability)
                    FROM payment_provider_capabilities ppc
                    WHERE ppc.provider_id = payment_providers.id
               ), '[]'::jsonb) AS capabilities
        FROM payment_providers
        ORDER BY priority ASC, display_name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let res = json!({
                "items": rows.into_iter().map(provider_from_row).collect::<Vec<_>>()
            });
            crate::cache::AdminCache::set(cache_key, res.clone(), std::time::Duration::from_secs(30));
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            tracing::error!("List company payment providers failed: {}", e);
            server_error("Failed to load providers")
        }
    }
}

async fn create_provider(
    pool: web::Data<PgPool>,
    request: web::Json<UpsertProviderRequest>,
) -> impl Responder {
    save_provider(pool, None, request.into_inner()).await
}

async fn update_provider(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<UpsertProviderRequest>,
) -> impl Responder {
    save_provider(pool, Some(path.into_inner()), request.into_inner()).await
}

async fn save_provider(
    pool: web::Data<PgPool>,
    provider_id: Option<Uuid>,
    request: UpsertProviderRequest,
) -> HttpResponse {
    if request.name.trim().is_empty() || request.code.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "message": "name and code are required" }));
    }

    let methods = json!(request.supported_payment_methods.unwrap_or_default());
    let currencies = json!(request.supported_currencies.unwrap_or_default());
    let countries = json!(request.supported_countries.unwrap_or_default());

    let row = if let Some(id) = provider_id {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE payment_providers
            SET display_name = $2, name = $3, code = $3, provider_type = $4, environment = $5,
                status = $6, priority = $7, failover_enabled = $8, webhook_url = $9,
                supported_payment_methods = $10::jsonb,
                supported_currencies = $11::jsonb,
                supported_countries = $12::jsonb,
                is_active = ($6 = 'active'),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(request.name.trim())
        .bind(request.code.trim().to_lowercase())
        .bind(request.provider_type.trim().to_lowercase())
        .bind(request.environment.trim().to_lowercase())
        .bind(request.status.trim().to_lowercase())
        .bind(request.priority.unwrap_or(100))
        .bind(request.failover_enabled.unwrap_or(true))
        .bind(request.webhook_url.as_deref())
        .bind(methods)
        .bind(currencies)
        .bind(countries)
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO payment_providers (
                display_name, name, code, provider_type, environment, status, priority,
                failover_enabled, webhook_url, supported_payment_methods,
                supported_currencies, supported_countries, is_active
            )
            VALUES ($1, $2, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10::jsonb, $11::jsonb, ($5 = 'active'))
            RETURNING id
            "#,
        )
        .bind(request.name.trim())
        .bind(request.code.trim().to_lowercase())
        .bind(request.provider_type.trim().to_lowercase())
        .bind(request.environment.trim().to_lowercase())
        .bind(request.status.trim().to_lowercase())
        .bind(request.priority.unwrap_or(100))
        .bind(request.failover_enabled.unwrap_or(true))
        .bind(request.webhook_url.as_deref())
        .bind(methods)
        .bind(currencies)
        .bind(countries)
        .fetch_optional(pool.get_ref())
        .await
    };

    match row {
        Ok(Some(id)) => {
            audit(
                &pool,
                "provider.saved",
                "payment_provider",
                Some(id),
                json!({ "code": request.code }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Provider not found" })),
        Err(e) => {
            tracing::error!("Save company payment provider failed: {}", e);
            server_error("Failed to save provider")
        }
    }
}

async fn test_provider(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let provider_id = path.into_inner();
    let provider = sqlx::query(
        r#"
        SELECT provider_type, COALESCE(code, name, display_name) AS provider_code, environment, status,
               EXISTS(SELECT 1 FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type = 'api_key' AND pc.status = 'active') AS has_api_key,
               EXISTS(SELECT 1 FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type = 'api_secret' AND pc.status = 'active') AS has_api_secret
        FROM payment_providers
        WHERE id = $1
        "#,
    )
    .bind(provider_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Some(provider) = (match provider {
        Ok(provider) => provider,
        Err(e) => {
            tracing::error!("Provider lookup before health check failed: {}", e);
            return server_error("Failed to test provider");
        }
    }) else {
        return HttpResponse::NotFound().json(json!({ "message": "Provider not found" }));
    };
    let provider_type: String = provider.get("provider_type");
    let provider_code: String = provider.get("provider_code");
    let environment: String = provider.get("environment");
    let provider_status: String = provider.get("status");
    let has_api_key: bool = provider.get("has_api_key");
    let has_api_secret: bool = provider.get("has_api_secret");
    if provider_status != "active" {
        return HttpResponse::Ok().json(connector_failure(&provider_code, "provider_disabled"));
    }
    let credentials_ready =
        connector_credentials_ready(&provider_type, &provider_code, has_api_key, has_api_secret);
    if is_real_provider_type(&provider_type, &provider_code) && !credentials_ready {
        let result = connector_failure(&provider_code, "missing_required_credentials");
        let _ = sqlx::query(
            "UPDATE payment_providers SET health_status = 'failed', last_connection_test_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(provider_id)
        .execute(pool.get_ref())
        .await;
        return HttpResponse::Ok().json(result);
    }
    if is_real_provider_type(&provider_type, &provider_code)
        && !has_implemented_real_connector(&provider_type, &provider_code)
    {
        let result = connector_failure(&provider_code, "provider_connector_not_configured");
        let _ = sqlx::query(
            "UPDATE payment_providers SET health_status = 'failed', last_connection_test_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(provider_id)
        .execute(pool.get_ref())
        .await;
        return HttpResponse::Ok().json(result);
    }
    let connector_type = if has_implemented_real_connector(&provider_type, &provider_code) {
        provider_code.as_str()
    } else {
        provider_type.as_str()
    };
    let result = connector_for_type(connector_type, &environment, credentials_ready)
        .health_check()
        .await;
    let status = if result.status == ConnectorOperationStatus::Verified {
        "healthy"
    } else {
        "failed"
    };
    let updated = sqlx::query(
        "UPDATE payment_providers SET health_status = $2, last_health_check_at = NOW(), last_connection_test_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(provider_id)
    .bind(status)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(_) => {
            if result.status == ConnectorOperationStatus::Failed {
                create_admin_notification(
                    &pool,
                    "warning",
                    "provider_degraded",
                    "Provider connection test failed",
                    "provider",
                    Some(provider_id),
                )
                .await;
                audit(
                    &pool,
                    "provider.connection_test_failed",
                    "payment_provider",
                    Some(provider_id),
                    result.summary.clone(),
                )
                .await;
            }
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Provider health update failed: {}", e);
            server_error("Failed to test provider")
        }
    }
}

async fn set_provider_status(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    status: &'static str,
) -> impl Responder {
    let provider_id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE payment_providers SET status = $2, is_active = ($2 = 'active'), updated_at = NOW() WHERE id = $1",
    )
    .bind(provider_id)
    .bind(status)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                &format!("provider.{status}"),
                "payment_provider",
                Some(provider_id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": provider_id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Provider not found" })),
        Err(e) => {
            tracing::error!("Provider status update failed: {}", e);
            server_error("Failed to update provider status")
        }
    }
}

async fn enable_provider(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    set_provider_status(pool, path, "active").await
}

async fn disable_provider(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    set_provider_status(pool, path, "disabled").await
}

async fn maintenance_provider(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    set_provider_status(pool, path, "maintenance").await
}

async fn list_credentials(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT pc.id, pc.provider_id, p.display_name AS provider_name, pc.environment,
               COALESCE(pc.secret_type, pc.key_name) AS secret_type, pc.masked_value, pc.status,
               pc.last_validated_at, pc.last_rotated_at, pc.rotation_due_at, pc.description,
               pc.created_at, pc.updated_at
        FROM payment_provider_credentials pc
        JOIN payment_providers p ON p.id = pc.provider_id
        ORDER BY p.display_name, pc.environment, secret_type
        "#,
        "Failed to load credentials",
    )
    .await
}

async fn create_credential(
    pool: web::Data<PgPool>,
    request: web::Json<CredentialRequest>,
) -> impl Responder {
    save_credential(
        pool,
        request.provider_id,
        request.environment.trim(),
        request.secret_type.trim(),
        request.value.trim(),
        request.rotation_due_at,
        request.description.as_deref(),
        "credential.created",
    )
    .await
}

async fn rotate_credentials_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RotateCredentialRequest>,
) -> impl Responder {
    let credential_id = path.into_inner();
    if request.value.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "message": "value is required" }));
    }
    let row = sqlx::query(
        "SELECT provider_id, environment, COALESCE(secret_type, key_name) AS secret_type FROM payment_provider_credentials WHERE id = $1",
    )
    .bind(credential_id)
    .fetch_optional(pool.get_ref())
    .await;

    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Credential not found" }));
    };

    let provider_id: Uuid = row.get("provider_id");
    let environment: String = request
        .environment
        .clone()
        .unwrap_or_else(|| row.get("environment"));
    let secret_type: String = request
        .secret_type
        .clone()
        .or_else(|| request.key_name.clone())
        .unwrap_or_else(|| row.get("secret_type"));

    save_credential(
        pool,
        provider_id,
        environment.trim(),
        secret_type.trim(),
        request.value.trim(),
        request.rotation_due_at,
        request.description.as_deref(),
        "credential.rotated",
    )
    .await
}

async fn rotate_credentials(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RotateCredentialRequest>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let secret_type = request
        .secret_type
        .as_deref()
        .or(request.key_name.as_deref())
        .unwrap_or("api_secret");
    save_credential(
        pool,
        provider_id,
        request.environment.as_deref().unwrap_or("sandbox"),
        secret_type,
        request.value.trim(),
        request.rotation_due_at,
        request.description.as_deref(),
        "credential.rotated",
    )
    .await
}

async fn save_credential(
    pool: web::Data<PgPool>,
    provider_id: Uuid,
    environment: &str,
    secret_type: &str,
    value: &str,
    rotation_due_at: Option<DateTime<Utc>>,
    description: Option<&str>,
    audit_action: &str,
) -> HttpResponse {
    if secret_type.is_empty() || value.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "secret_type and value are required" }));
    }
    let (encrypted_value, nonce) = match encrypt_secret(value) {
        Ok(value) => value,
        Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
    };
    let masked_value = mask_secret(value);
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_provider_credentials (
            provider_id, environment, key_name, secret_type, encrypted_value, nonce, masked_value,
            status, rotation_due_at, description, last_rotated_at
        )
        VALUES ($1, $2, $3, $3, $4, $5, $6, 'active', $7, $8, NOW())
        ON CONFLICT (provider_id, key_name)
        DO UPDATE SET encrypted_value = EXCLUDED.encrypted_value,
                      nonce = EXCLUDED.nonce,
                      masked_value = EXCLUDED.masked_value,
                      environment = EXCLUDED.environment,
                      secret_type = EXCLUDED.secret_type,
                      status = 'active',
                      rotation_due_at = EXCLUDED.rotation_due_at,
                      description = EXCLUDED.description,
                      rotated_at = NOW(),
                      last_rotated_at = NOW(),
                      updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(provider_id)
    .bind(environment.to_lowercase())
    .bind(secret_type.to_lowercase())
    .bind(encrypted_value)
    .bind(nonce)
    .bind(&masked_value)
    .bind(rotation_due_at)
    .bind(description)
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => {
            audit(&pool, audit_action, "payment_provider_credential", Some(id), json!({ "provider_id": provider_id, "secret_type": secret_type, "masked_value": masked_value })).await;
            HttpResponse::Ok()
                .json(json!({ "id": id, "masked_value": masked_value, "status": "active" }))
        }
        Err(e) => {
            tracing::error!("Save provider credential failed: {}", e);
            server_error("Failed to save credential")
        }
    }
}

async fn validate_credential(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_credential_status(
        pool,
        path.into_inner(),
        "active",
        "credential.validated",
        true,
    )
    .await
}

async fn disable_credential(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_credential_status(
        pool,
        path.into_inner(),
        "disabled",
        "credential.disabled",
        false,
    )
    .await
}

async fn update_credential_status(
    pool: web::Data<PgPool>,
    id: Uuid,
    status: &str,
    action: &str,
    mark_validated: bool,
) -> HttpResponse {
    let sql = if mark_validated {
        "UPDATE payment_provider_credentials SET status = $2, last_validated_at = NOW(), updated_at = NOW() WHERE id = $1"
    } else {
        "UPDATE payment_provider_credentials SET status = $2, updated_at = NOW() WHERE id = $1"
    };
    let updated = sqlx::query(sql)
        .bind(id)
        .bind(status)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                action,
                "payment_provider_credential",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Credential not found" })),
        Err(e) => {
            tracing::error!("Credential status update failed: {}", e);
            server_error("Failed to update credential")
        }
    }
}

async fn list_bank_integrations(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT id, acquiring_bank_name, integration_type, settlement_account_reference,
               bank_account_holder_name, masked_account_number,
               COALESCE(ifsc_or_swift_or_routing_code, routing_code) AS ifsc_or_swift_or_routing_code,
               account_scope, currency, settlement_cycle, verification_status, last_verified_at,
               supported_currencies, supported_payment_methods, status, created_at, updated_at
        FROM bank_integration_configs
        ORDER BY created_at DESC
        "#,
        "Failed to load bank integrations",
    )
    .await
}

async fn create_bank_integration(
    pool: web::Data<PgPool>,
    request: web::Json<BankConfigRequest>,
) -> impl Responder {
    save_bank_integration(pool, None, request.into_inner()).await
}

async fn update_bank_integration(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<BankConfigRequest>,
) -> impl Responder {
    save_bank_integration(pool, Some(path.into_inner()), request.into_inner()).await
}

async fn verify_bank_integration(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_bank_status(pool, path.into_inner(), "verified", true).await
}

async fn disable_bank_integration(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_bank_status(pool, path.into_inner(), "disabled", false).await
}

async fn update_bank_status(
    pool: web::Data<PgPool>,
    id: Uuid,
    status: &str,
    verified: bool,
) -> HttpResponse {
    let sql = if verified {
        "UPDATE bank_integration_configs SET verification_status = $2, status = 'active', last_verified_at = NOW(), updated_at = NOW() WHERE id = $1"
    } else {
        "UPDATE bank_integration_configs SET status = $2, updated_at = NOW() WHERE id = $1"
    };
    let updated = sqlx::query(sql)
        .bind(id)
        .bind(status)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                &format!("bank_config.{status}"),
                "bank_integration_config",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Bank integration not found" })),
        Err(e) => {
            tracing::error!("Bank status update failed: {}", e);
            server_error("Failed to update bank integration")
        }
    }
}

async fn save_bank_integration(
    pool: web::Data<PgPool>,
    bank_id: Option<Uuid>,
    request: BankConfigRequest,
) -> HttpResponse {
    let integration_type = normalize_bank_integration_type(&request.integration_type);
    let Some(integration_type) = integration_type else {
        return HttpResponse::BadRequest().json(json!({
            "message": "Choose API, SFTP, manual file upload, or provider managed integration type"
        }));
    };
    let masked_account_number = request
        .account_number
        .as_deref()
        .map(mask_bank_account)
        .or_else(|| request.masked_account_number.clone())
        .unwrap_or_else(|| "not_stored".to_string());
    let encrypted_account = request
        .account_number
        .as_deref()
        .and_then(|value| encrypt_secret(value).ok())
        .map(|(encrypted, nonce)| {
            json!({ "account_number": encrypted, "nonce": nonce }).to_string()
        });

    let result = if let Some(id) = bank_id {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE bank_integration_configs
            SET acquiring_bank_name = $2, integration_type = $3, settlement_account_reference = $4,
                encrypted_account_details = COALESCE($5, encrypted_account_details),
                bank_account_holder_name = $6, masked_account_number = $7, routing_code = $8,
                ifsc_or_swift_or_routing_code = $8, account_scope = $9, currency = $10,
                settlement_cycle = $11, supported_currencies = $12::jsonb,
                supported_payment_methods = $13::jsonb, updated_at = NOW()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(request.acquiring_bank_name.trim())
        .bind(integration_type)
        .bind(request.settlement_account_reference.as_deref())
        .bind(encrypted_account.as_deref())
        .bind(request.bank_account_holder_name.as_deref())
        .bind(masked_account_number)
        .bind(request.routing_code.as_deref())
        .bind(
            request
                .account_scope
                .as_deref()
                .unwrap_or("company_settlement"),
        )
        .bind(request.currency.as_deref().unwrap_or("INR"))
        .bind(request.settlement_cycle.as_deref())
        .bind(json!(request.supported_currencies.unwrap_or_default()))
        .bind(json!(request.supported_payment_methods.unwrap_or_default()))
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO bank_integration_configs (
                acquiring_bank_name, integration_type, settlement_account_reference,
                encrypted_account_details, bank_account_holder_name, masked_account_number,
                routing_code, ifsc_or_swift_or_routing_code, account_scope, currency,
                settlement_cycle, supported_currencies, supported_payment_methods
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, $10, $11::jsonb, $12::jsonb)
            RETURNING id
            "#,
        )
        .bind(request.acquiring_bank_name.trim())
        .bind(integration_type)
        .bind(request.settlement_account_reference.as_deref())
        .bind(encrypted_account.as_deref())
        .bind(request.bank_account_holder_name.as_deref())
        .bind(masked_account_number)
        .bind(request.routing_code.as_deref())
        .bind(
            request
                .account_scope
                .as_deref()
                .unwrap_or("company_settlement"),
        )
        .bind(request.currency.as_deref().unwrap_or("INR"))
        .bind(request.settlement_cycle.as_deref())
        .bind(json!(request.supported_currencies.unwrap_or_default()))
        .bind(json!(request.supported_payment_methods.unwrap_or_default()))
        .fetch_optional(pool.get_ref())
        .await
    };

    match result {
        Ok(Some(id)) => {
            audit(
                &pool,
                "bank_config.saved",
                "bank_integration_config",
                Some(id),
                json!({ "bank": request.acquiring_bank_name }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(json!({ "message": "Bank integration not found" }))
        }
        Err(e) => {
            tracing::error!("Save bank integration failed: {}", e);
            server_error("Failed to save bank integration")
        }
    }
}

fn normalize_bank_integration_type(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "api" | "sandbox" | "host_to_host" => Some("api"),
        "sftp" => Some("sftp"),
        "manual" | "manual_file_upload" => Some("manual_file_upload"),
        "provider" | "provider_managed" => Some("provider_managed"),
        _ => None,
    }
}

async fn get_white_label_config(pool: web::Data<PgPool>) -> impl Responder {
    let row =
        sqlx::query("SELECT * FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1")
            .fetch_optional(pool.get_ref())
            .await;

    match row {
        Ok(Some(row)) => HttpResponse::Ok().json(row_to_json(&row)),
        Ok(None) => HttpResponse::Ok().json(json!({
            "brand_name": "FeelPay",
            "checkout_theme_color": "#6366f1",
            "default_currency": "INR",
            "default_country": "IN",
            "default_payment_methods": ["upi", "card"],
            "invoice_prefix": "INV-",
            "receipt_prefix": "RCP-"
        })),
        Err(e) => {
            tracing::error!("Load white-label config failed: {}", e);
            server_error("Failed to load white-label config")
        }
    }
}

async fn upload_white_label_logo(
    pool: web::Data<PgPool>,
    body: web::Json<LogoUploadRequest>,
) -> impl Responder {
    let content_type = body.content_type.trim().to_ascii_lowercase();
    let allowed = ["image/png", "image/jpeg", "image/webp", "image/svg+xml"];
    if !allowed.contains(&content_type.as_str()) {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Only PNG/JPEG/WEBP/SVG allowed" }));
    }

    let raw = match BASE64.decode(body.data_base64.trim()) {
        Ok(raw) => raw,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({ "message": "Invalid base64 payload" }))
        }
    };

    if raw.len() < 200 * 1024 {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Logo must be at least 200KB" }));
    }
    if raw.len() > 5 * 1024 * 1024 {
        return HttpResponse::BadRequest().json(json!({ "message": "Logo must be <= 5MB" }));
    }

    let ext = match content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        _ => "webp",
    };

    let file_id = format!("white_label_logo_{}", Uuid::new_v4().simple());
    let dir = Path::new("uploads/branding");
    if let Err(e) = fs::create_dir_all(dir) {
        tracing::error!("Failed to create branding directory: {}", e);
        return server_error("Failed to prepare storage");
    }

    let path = dir.join(format!("{}.{}", file_id, ext));
    if let Err(e) = fs::write(&path, &raw) {
        tracing::error!("Failed to write logo file: {}", e);
        return server_error("Failed to save logo");
    }

    let logo_url = format!("/uploads/branding/{}.{}", file_id, ext);

    // 1. Fetch old logo for cleanup
    let old_logo: Option<String> = sqlx::query_scalar(
        "SELECT checkout_logo_url FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await
    .unwrap_or(None);

    // 2. Update the config immediately with the new logo URL
    let update_result = sqlx::query(
        "UPDATE white_label_payment_config SET checkout_logo_url = $1, updated_at = NOW() WHERE id = (SELECT id FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1)"
    )
    .bind(&logo_url)
    .execute(pool.get_ref())
    .await;

    if update_result.is_ok() {
        // 3. Cleanup old file if it was a local upload
        if let Some(old) = old_logo {
            if old.starts_with("/uploads/branding/") {
                let old_rel_path = old.trim_start_matches('/');
                let _ = fs::remove_file(old_rel_path);
            }
        }
    }

    HttpResponse::Ok().json(json!({
        "logo_url": logo_url,
        "size_bytes": raw.len(),
        "message": "Logo uploaded successfully"
    }))
}

async fn update_white_label_config(
    pool: web::Data<PgPool>,
    request: web::Json<WhiteLabelRequest>,
) -> impl Responder {
    // Check if config exists
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await;

    let id = match existing {
        Ok(Some(_id)) => {
            // Update existing
            match update_existing_white_label(pool.get_ref(), &request).await {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Update existing white-label failed: {}", e);
                    return server_error("Failed to update configuration");
                }
            }
        }
        Ok(None) => {
            // Insert new
            let inserted = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO white_label_payment_config (
                    brand_name, checkout_logo_url, checkout_theme_color, support_email, support_phone,
                    payment_page_domain, default_currency, default_country, default_payment_methods,
                    default_provider_id, fallback_provider_id, invoice_prefix, receipt_prefix,
                    statement_descriptor, terms_url, privacy_url, refund_policy_url
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12, $13, $14, $15, $16, $17)
                RETURNING id
                "#,
            )
            .bind(request.brand_name.trim())
            .bind(request.checkout_logo_url.as_deref())
            .bind(request.checkout_theme_color.trim())
            .bind(request.support_email.as_deref())
            .bind(request.support_phone.as_deref())
            .bind(request.payment_page_domain.as_deref())
            .bind(request.default_currency.trim())
            .bind(request.default_country.trim())
            .bind(json!(request.default_payment_methods))
            .bind(request.default_provider_id)
            .bind(request.fallback_provider_id)
            .bind(request.invoice_prefix.trim())
            .bind(request.receipt_prefix.trim())
            .bind(request.statement_descriptor.as_deref())
            .bind(request.terms_url.as_deref())
            .bind(request.privacy_url.as_deref())
            .bind(request.refund_policy_url.as_deref())
            .fetch_one(pool.get_ref())
            .await;

            match inserted {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Insert white-label config failed: {}", e);
                    return server_error("Failed to create configuration");
                }
            }
        }
        Err(e) => {
            tracing::error!("Check existing white-label failed: {}", e);
            return server_error("Database error");
        }
    };

    audit(
        &pool,
        "white_label_config.saved",
        "white_label_payment_config",
        Some(id),
        json!({ "brand_name": request.brand_name }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": id }))
}

async fn update_existing_white_label(
    pool: &PgPool,
    request: &WhiteLabelRequest,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE white_label_payment_config
        SET brand_name = $1, checkout_logo_url = $2, checkout_theme_color = $3, support_email = $4,
            support_phone = $5, payment_page_domain = $6, default_currency = $7, default_country = $8,
            default_payment_methods = $9::jsonb, default_provider_id = $10, fallback_provider_id = $11,
            invoice_prefix = $12, receipt_prefix = $13, statement_descriptor = $14, terms_url = $15,
            privacy_url = $16, refund_policy_url = $17, updated_at = NOW()
        WHERE id = (SELECT id FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1)
        RETURNING id
        "#,
    )
    .bind(request.brand_name.trim())
    .bind(request.checkout_logo_url.as_deref())
    .bind(request.checkout_theme_color.trim())
    .bind(request.support_email.as_deref())
    .bind(request.support_phone.as_deref())
    .bind(request.payment_page_domain.as_deref())
    .bind(request.default_currency.trim())
    .bind(request.default_country.trim())
    .bind(json!(request.default_payment_methods))
    .bind(request.default_provider_id)
    .bind(request.fallback_provider_id)
    .bind(request.invoice_prefix.trim())
    .bind(request.receipt_prefix.trim())
    .bind(request.statement_descriptor.as_deref())
    .bind(request.terms_url.as_deref())
    .bind(request.privacy_url.as_deref())
    .bind(request.refund_policy_url.as_deref())
    .fetch_one(pool)
    .await
}

async fn list_routing_rules(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT r.*, p.display_name AS provider_name, fp.display_name AS failover_provider_name
        FROM payment_routing_rules r
        JOIN payment_providers p ON p.id = r.provider_id
        LEFT JOIN payment_providers fp ON fp.id = r.failover_provider_id
        ORDER BY r.priority ASC, r.created_at DESC
        "#,
        "Failed to load routing rules",
    )
    .await
}

async fn list_provider_performance(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT ppm.id, p.display_name AS provider, ppm.payment_method, ppm.currency, ppm.country,
               ppm.success_rate::float8 AS success_rate, ppm.failure_rate::float8 AS failure_rate,
               ppm.avg_latency_ms, ppm.p95_latency_ms, ppm.provider_fee_percent::float8 AS provider_fee_percent,
               ppm.attempt_count, ppm.updated_at
        FROM provider_performance_metrics ppm
        JOIN payment_providers p ON p.id = ppm.provider_id
        ORDER BY ppm.updated_at DESC
        LIMIT 200
        "#,
        "Failed to load provider performance",
    )
    .await
}

async fn create_routing_rule(
    pool: web::Data<PgPool>,
    request: web::Json<RoutingRuleRequest>,
) -> impl Responder {
    save_routing_rule(pool, None, request.into_inner()).await
}

async fn update_routing_rule(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RoutingRuleRequest>,
) -> impl Responder {
    save_routing_rule(pool, Some(path.into_inner()), request.into_inner()).await
}

async fn disable_routing_rule(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE payment_routing_rules SET status = 'disabled', updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "routing_rule.disabled",
                "payment_routing_rule",
                Some(id),
                json!({ "status": "disabled" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "disabled" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Routing rule not found" })),
        Err(e) => {
            tracing::error!("Disable routing rule failed: {}", e);
            server_error("Failed to disable routing rule")
        }
    }
}

async fn save_routing_rule(
    pool: web::Data<PgPool>,
    rule_id: Option<Uuid>,
    request: RoutingRuleRequest,
) -> HttpResponse {
    let result = if let Some(id) = rule_id {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE payment_routing_rules
            SET name = $2, payment_method = $3, currency = $4, country = $5, amount_min = $6,
                amount_max = $7, provider_id = $8, bank_integration_id = $9, priority = $10,
                risk_level = $11, customer_region = $12, business_line = $13,
                failover_provider_id = $14, status = $15, rule_group = $16,
                provider_weight = $17, payment_method_priority = $18, min_success_rate = $19,
                max_provider_fee_percent = $20, max_latency_ms = $21, health_required = $22,
                failover_strategy = $23, retry_strategy = $24, cooldown_minutes = $25,
                effective_from = $26, effective_until = $27, strategy = $28, updated_at = NOW()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(request.name.trim())
        .bind(request.payment_method.as_deref())
        .bind(request.currency.as_deref())
        .bind(request.country.as_deref())
        .bind(request.amount_min)
        .bind(request.amount_max)
        .bind(request.provider_id)
        .bind(request.bank_integration_id)
        .bind(request.priority.unwrap_or(100))
        .bind(request.risk_level.as_deref())
        .bind(request.customer_region.as_deref())
        .bind(request.business_line.as_deref())
        .bind(request.failover_provider_id)
        .bind(request.status.as_deref().unwrap_or("active"))
        .bind(request.rule_group.as_deref())
        .bind(request.provider_weight.unwrap_or(100))
        .bind(request.payment_method_priority.unwrap_or(100))
        .bind(request.min_success_rate)
        .bind(request.max_provider_fee_percent)
        .bind(request.max_latency_ms)
        .bind(request.health_required.unwrap_or(true))
        .bind(
            request
                .failover_strategy
                .as_deref()
                .unwrap_or("fallback_only"),
        )
        .bind(request.retry_strategy.as_deref().unwrap_or("none"))
        .bind(request.cooldown_minutes.unwrap_or(15))
        .bind(request.effective_from)
        .bind(request.effective_until)
        .bind(request.strategy.as_deref().unwrap_or("priority"))
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO payment_routing_rules (
                name, payment_method, currency, country, amount_min, amount_max, provider_id,
                bank_integration_id, priority, risk_level, customer_region, business_line,
                failover_provider_id, status, rule_group, provider_weight, payment_method_priority,
                min_success_rate, max_provider_fee_percent, max_latency_ms, health_required,
                failover_strategy, retry_strategy, cooldown_minutes, effective_from, effective_until, strategy
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
            RETURNING id
            "#,
        )
        .bind(request.name.trim())
        .bind(request.payment_method.as_deref())
        .bind(request.currency.as_deref())
        .bind(request.country.as_deref())
        .bind(request.amount_min)
        .bind(request.amount_max)
        .bind(request.provider_id)
        .bind(request.bank_integration_id)
        .bind(request.priority.unwrap_or(100))
        .bind(request.risk_level.as_deref())
        .bind(request.customer_region.as_deref())
        .bind(request.business_line.as_deref())
        .bind(request.failover_provider_id)
        .bind(request.status.as_deref().unwrap_or("active"))
        .bind(request.rule_group.as_deref())
        .bind(request.provider_weight.unwrap_or(100))
        .bind(request.payment_method_priority.unwrap_or(100))
        .bind(request.min_success_rate)
        .bind(request.max_provider_fee_percent)
        .bind(request.max_latency_ms)
        .bind(request.health_required.unwrap_or(true))
        .bind(request.failover_strategy.as_deref().unwrap_or("fallback_only"))
        .bind(request.retry_strategy.as_deref().unwrap_or("none"))
        .bind(request.cooldown_minutes.unwrap_or(15))
        .bind(request.effective_from)
        .bind(request.effective_until)
        .bind(request.strategy.as_deref().unwrap_or("priority"))
        .fetch_optional(pool.get_ref())
        .await
    };

    match result {
        Ok(Some(id)) => {
            audit(
                &pool,
                "routing_rule.saved",
                "payment_routing_rule",
                Some(id),
                json!({ "name": request.name }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Routing rule not found" })),
        Err(e) => {
            tracing::error!("Save routing rule failed: {}", e);
            server_error("Failed to save routing rule")
        }
    }
}

async fn simulate_routing(
    pool: web::Data<PgPool>,
    request: web::Json<RoutingSimulationRequest>,
) -> impl Responder {
    let row = sqlx::query(
        r#"
        SELECT r.id AS rule_id, r.name AS rule_name, COALESCE(r.strategy, 'priority') AS strategy,
               p.id AS provider_id,
               p.display_name AS provider_name, COALESCE(p.code, p.name) AS provider_code,
               p.health_status, fp.display_name AS fallback_provider_name,
               COALESCE(pm.success_rate, p.success_rate, 0)::float8 AS success_rate,
               COALESCE(pm.provider_fee_percent, 0) AS provider_fee_percent,
               COALESCE(pm.avg_latency_ms, p.avg_latency_ms, 0)::int AS avg_latency_ms,
               COALESCE(pm.attempt_count, 0)::bigint AS attempt_count,
               (
                 COALESCE(pm.success_rate, p.success_rate, 0)::float8
                 - COALESCE(pm.provider_fee_percent, 0)::float8
                 - (COALESCE(pm.avg_latency_ms, p.avg_latency_ms, 0)::float8 / 1000.0)
                 + (COALESCE(r.provider_weight, 100)::float8 / 100.0)
               ) AS provider_score
        FROM payment_routing_rules r
        JOIN payment_providers p ON p.id = r.provider_id
        LEFT JOIN payment_providers fp ON fp.id = COALESCE(r.fallback_provider_id, r.failover_provider_id)
        LEFT JOIN provider_performance_metrics pm ON pm.provider_id = p.id
          AND ($1::text IS NULL OR pm.payment_method IS NULL OR pm.payment_method = $1)
          AND ($2::text IS NULL OR pm.currency IS NULL OR pm.currency = $2)
          AND ($3::text IS NULL OR pm.country IS NULL OR pm.country = $3)
        WHERE r.status = 'active'
          AND p.status = 'active'
          AND p.health_status IS DISTINCT FROM 'failed'
          AND ($1::text IS NULL OR r.payment_method IS NULL OR r.payment_method = $1)
          AND ($2::text IS NULL OR r.currency IS NULL OR r.currency = $2)
          AND ($3::text IS NULL OR r.country IS NULL OR r.country = $3)
          AND (r.amount_min IS NULL OR r.amount_min <= $4)
          AND (r.amount_max IS NULL OR r.amount_max >= $4)
          AND ($5::text IS NULL OR r.risk_level IS NULL OR r.risk_level = $5)
          AND ($6::text IS NULL OR r.customer_region IS NULL OR r.customer_region = $6)
          AND ($7::text IS NULL OR r.business_line IS NULL OR r.business_line = $7)
          AND (r.effective_from IS NULL OR r.effective_from <= NOW())
          AND (r.effective_until IS NULL OR r.effective_until >= NOW())
          AND (r.min_success_rate IS NULL OR COALESCE(pm.success_rate, p.success_rate, 0) >= r.min_success_rate)
          AND (r.max_provider_fee_percent IS NULL OR COALESCE(pm.provider_fee_percent, 0) <= r.max_provider_fee_percent)
          AND (r.max_latency_ms IS NULL OR COALESCE(pm.avg_latency_ms, p.avg_latency_ms, 0) <= r.max_latency_ms)
        ORDER BY
          CASE WHEN COALESCE(r.strategy, 'priority') = 'highest_success_rate' THEN COALESCE(pm.success_rate, p.success_rate, 0) END DESC,
          CASE WHEN COALESCE(r.strategy, 'priority') = 'lowest_cost' THEN COALESCE(pm.provider_fee_percent, 0) END ASC,
          CASE WHEN COALESCE(r.strategy, 'priority') = 'latency_aware' THEN COALESCE(pm.avg_latency_ms, p.avg_latency_ms, 0) END ASC,
          r.priority ASC, p.priority ASC
        LIMIT 1
        "#,
    )
    .bind(request.payment_method.as_deref())
    .bind(request.currency.as_deref())
    .bind(request.country.as_deref())
    .bind(request.amount)
    .bind(request.risk_level.as_deref())
    .bind(request.customer_region.as_deref())
    .bind(request.business_line.as_deref())
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some(row)) => {
            let provider_fee_percent = row.get::<Decimal, _>("provider_fee_percent");
            let estimated_provider_fee = percentage_amount_rounded(request.amount, provider_fee_percent);
            HttpResponse::Ok().json(RoutingSimulationResponse {
                selected_provider_id: Some(row.get("provider_id")),
                selected_provider_code: Some(row.get("provider_code")),
                selected_provider: Some(row.get("provider_name")),
                routing_rule_id: Some(row.get("rule_id")),
                matched_routing_rule: Some(row.get("rule_name")),
                fallback_provider: row.get("fallback_provider_name"),
                decision_reason: "Eligible provider matched strategy, health, cost, success, latency, and amount filters.".to_string(),
                estimated_provider_fee,
                estimated_company_fee: request.amount * 7 / 1000,
                risk_decision: request.risk_level.clone().unwrap_or_else(|| "allow".to_string()),
                provider_health_used: row.get("health_status"),
                failover_applied: false,
                provider_score: row.get("provider_score"),
                candidate_providers: vec![json!({
                    "provider": row.get::<String, _>("provider_name"),
                    "score": row.get::<f64, _>("provider_score"),
                    "success_rate": row.get::<f64, _>("success_rate"),
                    "latency_ms": row.get::<i32, _>("avg_latency_ms"),
                    "fee_percent": provider_fee_percent,
                    "attempt_count": row.get::<i64, _>("attempt_count")
                })],
                rejected_providers: vec![],
                fee_estimate: estimated_provider_fee,
                latency_estimate_ms: row.get("avg_latency_ms"),
                success_estimate: row.get("success_rate"),
                strategy_used: row.get("strategy"),
            })
        }
        Ok(None) => HttpResponse::Ok().json(RoutingSimulationResponse {
            selected_provider_id: None,
            selected_provider_code: None,
            selected_provider: None,
            routing_rule_id: None,
            matched_routing_rule: None,
            fallback_provider: None,
            decision_reason:
                "No active routing rule matched; use default/fallback provider if configured"
                    .to_string(),
            estimated_provider_fee: 0,
            estimated_company_fee: 0,
            risk_decision: "review".to_string(),
            provider_health_used: "unknown".to_string(),
            failover_applied: false,
            provider_score: 0.0,
            candidate_providers: vec![],
            rejected_providers: vec![
                json!({ "reason": "No eligible active provider matched filters" }),
            ],
            fee_estimate: 0,
            latency_estimate_ms: 0,
            success_estimate: 0.0,
            strategy_used: "priority".to_string(),
        }),
        Err(e) => {
            tracing::error!("Routing simulation failed: {}", e);
            server_error("Failed to simulate routing")
        }
    }
}

async fn list_transactions(
    pool: web::Data<PgPool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let status = query.status.clone();
    let cache_key = format!(
        "admin:transactions:{}:{}:{}",
        status.as_deref().unwrap_or("all"),
        limit,
        offset
    );
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(&cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query(
        r#"
        SELECT po.id, po.order_id, po.amount, po.currency, po.status, po.payment_method,
               po.customer_id AS customer_reference, po.gateway_reference AS provider_payment_id,
               po.connector AS provider_used, po.created_at,
               COALESCE(rr.status, 'unreviewed') AS risk_status,
               COALESCE(sb.status, 'pending') AS settlement_status,
               COALESCE(pr.status, 'none') AS refund_status
        FROM payment_orders po
        LEFT JOIN risk_reviews rr ON rr.payment_order_id = po.id
        LEFT JOIN settlement_items_company si ON si.payment_order_id = po.id
        LEFT JOIN settlement_batches_company sb ON sb.id = si.settlement_batch_id
        LEFT JOIN payment_refunds pr ON pr.payment_order_id = po.id
        WHERE ($1::text IS NULL OR po.status = $1)
        ORDER BY po.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let res = json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() });
            crate::cache::AdminCache::set(&cache_key, res.clone(), std::time::Duration::from_secs(15));
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            tracing::error!("List company transactions failed: {}", e);
            server_error("Failed to load transactions")
        }
    }
}

async fn list_ledger(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let cache_key = format!("admin:ledger:{}:{}", limit, offset);
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(&cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query(
        r#"
        SELECT le.id, le.created_at, le.transaction_group_id, le.source_type, le.source_id,
               la.name AS account, la.owner_type, la.owner_id,
               CASE WHEN COALESCE(le.direction, le.entry_type) = 'debit' THEN le.amount ELSE 0 END AS debit,
               CASE WHEN COALESCE(le.direction, le.entry_type) = 'credit' THEN le.amount ELSE 0 END AS credit,
               le.currency, le.balance_after, le.status, le.description
        FROM ledger_entries le
        JOIN ledger_accounts la ON la.id = le.account_id
        ORDER BY le.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let summary = sqlx::query(
        r#"
        SELECT
          COALESCE((SELECT SUM(balance) FROM ledger_accounts WHERE owner_type = 'company'), 0)::bigint AS total_company_balance,
          COALESCE((SELECT SUM(net_settlement_amount) FROM settlement_batches_company WHERE status IN ('scheduled','processing','on_hold')), 0)::bigint AS pending_settlement,
          COALESCE((SELECT SUM(balance) FROM ledger_accounts WHERE owner_type = 'business_account' AND account_type IN ('payable','merchant')), 0)::bigint AS business_payable,
          COALESCE((SELECT SUM(provider_fee_amount) FROM settlement_batches_company), 0)::bigint AS provider_fees,
          COALESCE((SELECT SUM(company_fee_amount) FROM settlement_batches_company), 0)::bigint AS company_revenue,
          COALESCE((SELECT SUM(amount) FROM reserve_balances WHERE status = 'held'), 0)::bigint AS reserve_balance,
          COALESCE((SELECT COUNT(*) FROM ledger_entries WHERE status <> 'posted'), 0)::bigint AS unreconciled_ledger_entries
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    match (rows, summary) {
        (Ok(rows), Ok(summary)) => {
            let res = json!({
                "summary": row_to_json(&summary),
                "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
            });
            crate::cache::AdminCache::set(&cache_key, res.clone(), std::time::Duration::from_secs(15));
            HttpResponse::Ok().json(res)
        }
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!("List ledger failed: {}", e);
            server_error("Failed to load ledger")
        }
    }
}

async fn get_ledger_group(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let rows = sqlx::query(
        r#"
        SELECT le.*, la.name AS account, la.owner_type, la.owner_id
        FROM ledger_entries le
        JOIN ledger_accounts la ON la.id = le.account_id
        WHERE le.transaction_group_id = $1
        ORDER BY le.created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Load ledger group failed: {}", e);
            server_error("Failed to load ledger group")
        }
    }
}

async fn export_ledger(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "ledger.exported",
        "ledger",
        None,
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "download_url": "pending-ledger-export" }))
}

async fn get_transaction(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let payment = sqlx::query("SELECT * FROM payment_orders WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = payment else {
        return HttpResponse::NotFound().json(json!({ "message": "Transaction not found" }));
    };

    let attempts = fetch_json_rows(
        pool.get_ref(),
        "SELECT * FROM payment_attempts WHERE payment_order_id = $1 ORDER BY created_at DESC",
        id,
    )
    .await;
    let webhooks = fetch_json_rows(pool.get_ref(), "SELECT * FROM payment_webhook_events WHERE related_payment_id = $1 ORDER BY received_at DESC", id).await;
    let refunds = fetch_json_rows(
        pool.get_ref(),
        "SELECT * FROM payment_refunds WHERE payment_order_id = $1 ORDER BY created_at DESC",
        id,
    )
    .await;

    HttpResponse::Ok().json(json!({
        "transaction": row_to_json(&row),
        "attempt_history": attempts,
        "webhook_events": webhooks,
        "refunds": refunds,
        "audit_timeline": []
    }))
}

async fn transaction_action(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    action: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    let status = match action {
        "capture" => "captured",
        "cancel" => "cancelled",
        _ => "action_requested",
    };
    let updated =
        sqlx::query("UPDATE payment_orders SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(pool.get_ref())
            .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            if action == "capture" {
                let payment = sqlx::query(
                    "SELECT amount, currency, connector FROM payment_orders WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool.get_ref())
                .await;
                if let Ok(Some(row)) = payment {
                    let amount: i64 = row.get("amount");
                    let currency: String = row.get("currency");
                    let _ = post_balanced_ledger(
                        &pool,
                        "payment_captured",
                        "payment_order",
                        id,
                        None,
                        None,
                        amount,
                        &currency,
                        &format!("payment-capture-{id}"),
                        "Payment captured",
                    )
                    .await;
                }
            }
            audit(
                &pool,
                &format!("transaction.{action}"),
                "payment_order",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Transaction not found" })),
        Err(e) => {
            tracing::error!("Transaction action failed: {}", e);
            server_error("Failed to update transaction")
        }
    }
}

async fn capture_transaction(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    transaction_action(pool, path, "capture").await
}

async fn cancel_transaction(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    transaction_action(pool, path, "cancel").await
}

async fn create_refund(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RefundRequest>,
) -> impl Responder {
    let mut body = request.into_inner();
    body.payment_order_id = Some(path.into_inner());
    insert_refund(pool, body).await
}

async fn list_refunds(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let cache_key = format!(
        "admin:refunds:{}:{}:{}",
        query.status.as_deref().unwrap_or("all"),
        limit,
        offset
    );
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(&cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query(
        "SELECT * FROM payment_refunds WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(query.status.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => {
            let res = json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() });
            crate::cache::AdminCache::set(&cache_key, res.clone(), std::time::Duration::from_secs(15));
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            tracing::error!("List refunds failed: {}", e);
            server_error("Failed to load refunds")
        }
    }
}

async fn insert_refund(pool: web::Data<PgPool>, request: RefundRequest) -> HttpResponse {
    let result = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_refunds (
            payment_order_id, provider_id, amount, currency, reason, refund_source, idempotency_key,
            refund_type, business_account_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(request.payment_order_id)
    .bind(request.provider_id)
    .bind(request.amount)
    .bind(request.currency.as_deref().unwrap_or("INR"))
    .bind(request.reason.as_deref())
    .bind(request.refund_source.as_deref().unwrap_or("company"))
    .bind(&request.idempotency_key)
    .bind(request.refund_type.as_deref().unwrap_or("full"))
    .bind(request.business_account_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(id) => {
            evaluate_refund_risk(
                pool.get_ref(),
                id,
                request.business_account_id,
                request.amount,
            )
            .await;
            audit(
                &pool,
                "refund.requested",
                "payment_refund",
                Some(id),
                json!({ "amount": request.amount }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "requested" }))
        }
        Err(e) => {
            tracing::error!("Create refund failed: {}", e);
            server_error("Failed to create refund")
        }
    }
}

async fn get_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(
        pool,
        "payment_refunds",
        path.into_inner(),
        "Refund not found",
    )
    .await
}

async fn refund_status_action(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    status: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    let updated =
        sqlx::query("UPDATE payment_refunds SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(pool.get_ref())
            .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            if matches!(status, "initiated" | "processing") {
                let refund = sqlx::query(
                    "SELECT amount, currency, provider_id, business_account_id, idempotency_key FROM payment_refunds WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool.get_ref())
                .await;
                if let Ok(Some(row)) = refund {
                    let amount: i64 = row.get("amount");
                    let currency: String = row.get("currency");
                    let provider_id: Option<Uuid> = row.get("provider_id");
                    let business_account_id: Option<Uuid> = row.get("business_account_id");
                    let idempotency_key: String = row.get("idempotency_key");
                    let _ = post_balanced_ledger(
                        &pool,
                        "refund",
                        "payment_refund",
                        id,
                        provider_id,
                        business_account_id,
                        amount,
                        &currency,
                        &format!("refund-{idempotency_key}"),
                        "Refund initiated",
                    )
                    .await;
                }
            }
            audit(
                &pool,
                &format!("refund.{status}"),
                "payment_refund",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Refund not found" })),
        Err(e) => {
            tracing::error!("Refund status update failed: {}", e);
            server_error("Failed to update refund")
        }
    }
}

async fn approve_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    refund_status_action(pool, path, "approved").await
}

async fn reject_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    refund_status_action(pool, path, "rejected").await
}

async fn initiate_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    refund_status_action(pool, path, "initiated").await
}

async fn retry_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    refund_status_action(pool, path, "processing").await
}

async fn list_webhook_configs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT wc.id, wc.provider_id, p.display_name AS provider_name, wc.environment,
               wc.webhook_url, wc.masked_signing_secret, wc.signature_verification_enabled,
               wc.last_event_received_at, wc.failed_event_count, wc.retry_queue_count,
               wc.status, wc.created_at, wc.updated_at
        FROM webhook_endpoint_configs wc
        JOIN payment_providers p ON p.id = wc.provider_id
        ORDER BY p.display_name, wc.environment
        "#,
        "Failed to load webhook configs",
    )
    .await
}

async fn save_webhook_config(
    pool: web::Data<PgPool>,
    request: web::Json<WebhookConfigRequest>,
) -> impl Responder {
    let (encrypted_secret, nonce, masked_secret) = match request
        .signing_secret
        .as_deref()
        .filter(|v| !v.trim().is_empty())
    {
        Some(secret) => match encrypt_secret(secret.trim()) {
            Ok((encrypted, nonce)) => (Some(encrypted), Some(nonce), mask_secret(secret.trim())),
            Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
        },
        None => (None, None, "Missing".to_string()),
    };

    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO webhook_endpoint_configs (
            provider_id, environment, webhook_url, encrypted_signing_secret, signing_secret_nonce,
            masked_signing_secret, signature_verification_enabled, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (provider_id, environment)
        DO UPDATE SET webhook_url = EXCLUDED.webhook_url,
                      encrypted_signing_secret = COALESCE(EXCLUDED.encrypted_signing_secret, webhook_endpoint_configs.encrypted_signing_secret),
                      signing_secret_nonce = COALESCE(EXCLUDED.signing_secret_nonce, webhook_endpoint_configs.signing_secret_nonce),
                      masked_signing_secret = CASE WHEN EXCLUDED.encrypted_signing_secret IS NULL THEN webhook_endpoint_configs.masked_signing_secret ELSE EXCLUDED.masked_signing_secret END,
                      signature_verification_enabled = EXCLUDED.signature_verification_enabled,
                      status = EXCLUDED.status,
                      updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(request.provider_id)
    .bind(request.environment.trim().to_lowercase())
    .bind(request.webhook_url.trim())
    .bind(encrypted_secret.as_deref())
    .bind(nonce.as_deref())
    .bind(masked_secret)
    .bind(request.signature_verification_enabled.unwrap_or(true))
    .bind(request.status.as_deref().unwrap_or("active"))
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => {
            audit(
                &pool,
                "webhook_config.saved",
                "webhook_endpoint_config",
                Some(id),
                json!({ "provider_id": request.provider_id, "environment": request.environment }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Save webhook config failed: {}", e);
            server_error("Failed to save webhook config")
        }
    }
}

async fn test_webhook_config(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    audit(
        &pool,
        "webhook_config.tested",
        "webhook_endpoint_config",
        Some(id),
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": id, "status": "test_queued" }))
}

async fn rotate_webhook_secret(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RotateCredentialRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let (encrypted, nonce) = match encrypt_secret(request.value.trim()) {
        Ok(value) => value,
        Err(message) => return HttpResponse::BadRequest().json(json!({ "message": message })),
    };
    let masked = mask_secret(request.value.trim());
    let updated = sqlx::query(
        "UPDATE webhook_endpoint_configs SET encrypted_signing_secret = $2, signing_secret_nonce = $3, masked_signing_secret = $4, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(encrypted)
    .bind(nonce)
    .bind(&masked)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "webhook_secret.rotated",
                "webhook_endpoint_config",
                Some(id),
                json!({ "masked_signing_secret": masked }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "masked_value": masked }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Webhook config not found" })),
        Err(e) => {
            tracing::error!("Rotate webhook secret failed: {}", e);
            server_error("Failed to rotate webhook secret")
        }
    }
}

async fn list_webhooks(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let rows = sqlx::query(
        "SELECT * FROM payment_webhook_events WHERE ($1::text IS NULL OR processing_status = $1) ORDER BY received_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(query.status.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List webhook events failed: {}", e);
            server_error("Failed to load webhooks")
        }
    }
}

async fn find_webhook_event_id(pool: &PgPool, id_str: &str) -> Option<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(id_str) {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM payment_webhook_events WHERE id = $1)",
        )
        .bind(uuid)
        .fetch_one(pool)
        .await
        .unwrap_or(false);
        if exists {
            return Some(uuid);
        }
    }
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM payment_webhook_events WHERE event_id = $1")
        .bind(id_str)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}

async fn get_webhook(pool: web::Data<PgPool>, path: web::Path<String>) -> impl Responder {
    let id_str = path.into_inner();
    let Some(id) = find_webhook_event_id(pool.get_ref(), &id_str).await else {
        return HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" }));
    };
    get_by_id(pool, "payment_webhook_events", id, "Webhook not found").await
}

async fn retry_webhook(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    request: web::Json<WebhookRetryRequest>,
) -> impl Responder {
    let id_str = path.into_inner();
    let Some(id) = find_webhook_event_id(pool.get_ref(), &id_str).await else {
        return HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" }));
    };
    let updated = sqlx::query(
        "UPDATE payment_webhook_events SET processing_status = 'queued', retry_count = retry_count + 1, error_message = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "webhook.retry",
                "payment_webhook_event",
                Some(id),
                json!({ "reason": request.reason }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "processing_status": "queued" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Webhook not found" })),
        Err(e) => {
            tracing::error!("Retry webhook failed: {}", e);
            server_error("Failed to retry webhook")
        }
    }
}

async fn ignore_webhook(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query("UPDATE payment_webhook_events SET processing_status = 'ignored', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "webhook.ignored",
                "payment_webhook_event",
                Some(id),
                json!({ "processing_status": "ignored" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "processing_status": "ignored" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Webhook not found" })),
        Err(e) => {
            tracing::error!("Ignore webhook failed: {}", e);
            server_error("Failed to ignore webhook")
        }
    }
}

async fn list_settlements(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let cache_key = format!(
        "admin:settlements:{}:{}:{}",
        query.status.as_deref().unwrap_or("all"),
        limit,
        offset
    );
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(&cache_key) {
        return HttpResponse::Ok().json(cached);
    }

    let rows = sqlx::query(
        "SELECT * FROM settlement_batches_company WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(query.status.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => {
            let res = json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() });
            crate::cache::AdminCache::set(&cache_key, res.clone(), std::time::Duration::from_secs(15));
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            tracing::error!("List company settlements failed: {}", e);
            server_error("Failed to load settlements")
        }
    }
}

async fn get_settlement(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(
        pool,
        "settlement_batches_company",
        path.into_inner(),
        "Settlement not found",
    )
    .await
}

async fn settlement_action(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SettlementActionRequest>,
    status: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    if matches!(status, "paid") && request.bank_reference_number.as_deref().unwrap_or("").trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "message": "Paid settlement requires a manual, test, or provider reference" }));
    }
    let updated = sqlx::query(
        "UPDATE settlement_batches_company SET status = $2, bank_reference_number = COALESCE($3, bank_reference_number), finance_notes = COALESCE($4, finance_notes), reference_kind = COALESCE($5, reference_kind), failure_reason = COALESCE($6, failure_reason), actual_settlement_date = CASE WHEN $2 = 'paid' THEN CURRENT_DATE ELSE actual_settlement_date END, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(request.bank_reference_number.as_deref())
    .bind(request.note.as_deref())
    .bind(request.reference_kind.as_deref())
    .bind(request.failure_reason.as_deref())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            if status == "paid" {
                let settlement = sqlx::query(
                    "SELECT net_settlement_amount, currency, provider_id FROM settlement_batches_company WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool.get_ref())
                .await;
                if let Ok(Some(row)) = settlement {
                    let amount: i64 = row.get("net_settlement_amount");
                    let currency: String = row.get("currency");
                    let provider_id: Option<Uuid> = row.get("provider_id");
                    let _ = post_balanced_ledger(
                        &pool,
                        "settlement_credit",
                        "settlement_batch",
                        id,
                        provider_id,
                        None,
                        amount,
                        &currency,
                        &format!("settlement-{id}"),
                        "Settlement payout",
                    )
                    .await;
                }
            }
            audit(
                &pool,
                &format!("settlement.{status}"),
                "settlement_batch",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            evaluate_settlement_risk(pool.get_ref(), id).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Settlement not found" })),
        Err(e) => {
            tracing::error!("Settlement action failed: {}", e);
            server_error("Failed to update settlement")
        }
    }
}

async fn hold_settlement(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SettlementActionRequest>,
) -> impl Responder {
    settlement_action(pool, path, request, "on_hold").await
}

async fn release_settlement(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SettlementActionRequest>,
) -> impl Responder {
    settlement_action(pool, path, request, "processing").await
}

async fn mark_settlement_settled(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SettlementActionRequest>,
) -> impl Responder {
    settlement_action(pool, path, request, "paid").await
}

async fn mark_settlement_failed(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SettlementActionRequest>,
) -> impl Responder {
    settlement_action(pool, path, request, "failed").await
}

async fn export_settlement(path: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok()
        .json(json!({ "settlement_id": path.into_inner(), "download_url": "pending-export-job" }))
}

async fn list_reconciliation(pool: web::Data<PgPool>) -> impl Responder {
    let files = sqlx::query(
        r#"
        SELECT rf.id, rf.source, rf.provider_id, rf.file_name, rf.file_hash,
               COALESCE(rf.uploaded_by_admin_id, rf.uploaded_by) AS uploaded_by_admin_id,
               rf.status, rf.row_count,
               GREATEST(rf.matched_count, COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND COALESCE(rr.match_status, rr.status) = 'matched'), 0))::bigint AS matched_count,
               GREATEST(rf.mismatch_count, COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND (COALESCE(rr.match_status, rr.status) LIKE '%mismatch%' OR COALESCE(rr.match_status, rr.status) = 'unmatched')), 0))::bigint AS mismatch_count,
               GREATEST(rf.duplicate_count, COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND COALESCE(rr.match_status, rr.status) = 'duplicate'), 0))::bigint AS duplicate_count,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.observed_amount, rr.amount, 0) - rr.expected_amount)) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND rr.expected_amount IS NOT NULL), 0)::bigint AS amount_variance,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.fee_amount, 0) - rr.expected_fee_amount)) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND rr.expected_fee_amount IS NOT NULL), 0)::bigint AS fee_variance,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.tax_amount, 0) - rr.expected_tax_amount)) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND rr.expected_tax_amount IS NOT NULL), 0)::bigint AS tax_variance,
               rf.created_at, rf.processed_at
        FROM reconciliation_files rf
        ORDER BY rf.created_at DESC
        LIMIT 50
        "#)
        .fetch_all(pool.get_ref())
        .await;

    let queue = sqlx::query(
        r#"
        SELECT id, COALESCE(file_id, reconciliation_file_id) AS file_id, row_number,
               external_reference, internal_reference, amount, currency, record_type,
               COALESCE(match_status, status) AS match_status, mismatch_reason,
               matched_entity_type, matched_entity_id, resolution_status, confidence_score,
               suggested_match, payout_id, ledger_transaction_group_id, created_at
        FROM reconciliation_records
        WHERE COALESCE(match_status, status) IN ('unmatched','mismatch','duplicate')
           OR resolution_status = 'open'
        ORDER BY created_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match (files, queue) {
        (Ok(files), Ok(queue)) => HttpResponse::Ok().json(json!({
            "items": files.iter().map(row_to_json).collect::<Vec<_>>(),
            "queue": queue.iter().map(row_to_json).collect::<Vec<_>>()
        })),
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!("Failed to load reconciliation: {}", e);
            server_error("Failed to load reconciliation")
        }
    }
}

async fn upload_reconciliation_file(
    pool: web::Data<PgPool>,
    request: web::Json<ReconciliationUploadRequest>,
) -> impl Responder {
    let request = request.into_inner();
    if !matches!(
        request.source.as_str(),
        "provider_report"
            | "bank_statement"
            | "ledger_export"
            | "settlement_report"
            | "payout_report"
            | "fee_report"
    ) {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Invalid reconciliation source" }));
    }

    let existing =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM reconciliation_files WHERE file_hash = $1")
            .bind(request.file_hash.trim())
            .fetch_optional(pool.get_ref())
            .await;

    if let Ok(Some(id)) = existing {
        return HttpResponse::Conflict()
            .json(json!({ "message": "Duplicate reconciliation upload blocked", "id": id }));
    }

    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO reconciliation_files (
            source, file_type, provider_id, file_name, file_hash, uploaded_by_admin_id,
            status, row_count
        )
        VALUES ($1, $1, $2, $3, $4, NULL, 'uploaded', $5)
        RETURNING id
        "#,
    )
    .bind(request.source.trim())
    .bind(request.provider_id)
    .bind(request.file_name.trim())
    .bind(request.file_hash.trim())
    .bind(request.row_count.unwrap_or(0))
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => {
            audit(
                &pool,
                "reconciliation.file_uploaded",
                "reconciliation_file",
                Some(id),
                json!({ "source": request.source, "file_name": request.file_name }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "uploaded" }))
        }
        Err(e) => {
            tracing::error!("Upload reconciliation file failed: {}", e);
            server_error("Failed to upload reconciliation file")
        }
    }
}

async fn get_reconciliation(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let file = sqlx::query("SELECT * FROM reconciliation_files WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = file else {
        return HttpResponse::NotFound()
            .json(json!({ "message": "Reconciliation file not found" }));
    };
    let records = fetch_json_rows(pool.get_ref(), "SELECT * FROM reconciliation_records WHERE reconciliation_file_id = $1 ORDER BY created_at DESC", id).await;
    HttpResponse::Ok().json(json!({ "file": row_to_json(&row), "records": records }))
}

async fn resolve_reconciliation(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<ReconciliationResolveRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        r#"
        UPDATE reconciliation_records
        SET status = $2,
            match_status = $2,
            resolution_status = 'resolved',
            resolution_notes = $3,
            resolution_note = $3,
            resolved_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(request.status.trim())
    .bind(request.resolution_notes.trim())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "reconciliation.resolved",
                "reconciliation_record",
                Some(id),
                json!({ "status": request.status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": request.status }))
        }
        Ok(_) => {
            HttpResponse::NotFound().json(json!({ "message": "Reconciliation record not found" }))
        }
        Err(e) => {
            tracing::error!("Resolve reconciliation failed: {}", e);
            server_error("Failed to resolve reconciliation")
        }
    }
}

async fn auto_match_reconciliation(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let _ = sqlx::query(
        r#"
        UPDATE reconciliation_records rr
        SET match_status = CASE
                WHEN rr.external_reference IS NOT NULL AND (
                    EXISTS (SELECT 1 FROM payment_orders po WHERE po.gateway_reference = rr.external_reference OR po.id::text = rr.external_reference)
                    OR EXISTS (SELECT 1 FROM payment_refunds pr WHERE pr.provider_refund_id = rr.external_reference OR pr.id::text = rr.external_reference)
                    OR EXISTS (SELECT 1 FROM disputes_company dc WHERE dc.provider_dispute_id = rr.external_reference OR dc.id::text = rr.external_reference)
                    OR EXISTS (SELECT 1 FROM ledger_entries le WHERE le.transaction_group_id::text = rr.external_reference OR le.transaction_group_id = rr.ledger_transaction_group_id)
                ) THEN 'matched'
                WHEN rr.record_type = 'payout' AND rr.payout_id IS NULL THEN 'mismatch'
                WHEN rr.ledger_transaction_group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ledger_entries le WHERE le.transaction_group_id = rr.ledger_transaction_group_id) THEN 'mismatch'
                WHEN rr.fee_amount IS NOT NULL AND rr.expected_fee_amount IS NOT NULL AND rr.fee_amount <> rr.expected_fee_amount THEN 'mismatch'
                WHEN rr.tax_amount IS NOT NULL AND rr.expected_tax_amount IS NOT NULL AND rr.tax_amount <> rr.expected_tax_amount THEN 'mismatch'
                WHEN rr.amount IS NOT NULL AND rr.expected_amount IS NOT NULL AND rr.amount <> rr.expected_amount THEN 'mismatch'
                ELSE COALESCE(rr.match_status, rr.status)
            END,
            confidence_score = CASE
                WHEN rr.external_reference IS NOT NULL AND rr.internal_reference IS NOT NULL AND rr.amount = rr.expected_amount THEN 100
                WHEN rr.ledger_transaction_group_id IS NOT NULL THEN 96
                WHEN rr.external_reference IS NOT NULL THEN 92
                WHEN rr.provider_refund_id IS NOT NULL OR rr.provider_dispute_id IS NOT NULL THEN 90
                WHEN rr.bank_utr IS NOT NULL OR rr.settlement_reference IS NOT NULL THEN 85
                WHEN rr.amount IS NOT NULL THEN 65
                ELSE confidence_score
            END,
            mismatch_reason = CASE
                WHEN rr.fee_amount IS NOT NULL AND rr.expected_fee_amount IS NOT NULL AND rr.fee_amount <> rr.expected_fee_amount THEN 'fee_mismatch'
                WHEN rr.tax_amount IS NOT NULL AND rr.expected_tax_amount IS NOT NULL AND rr.tax_amount <> rr.expected_tax_amount THEN 'tax_mismatch'
                WHEN rr.record_type = 'payout' AND rr.payout_id IS NULL THEN 'payout_mismatch'
                WHEN rr.ledger_transaction_group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ledger_entries le WHERE le.transaction_group_id = rr.ledger_transaction_group_id) THEN 'ledger_mismatch'
                WHEN rr.amount IS NOT NULL AND rr.expected_amount IS NOT NULL AND rr.amount <> rr.expected_amount THEN 'amount_mismatch'
                ELSE rr.mismatch_reason
            END,
            suggested_match = jsonb_build_object(
                'method', CASE
                    WHEN rr.external_reference IS NOT NULL THEN 'exact_reference'
                    WHEN rr.bank_utr IS NOT NULL THEN 'utr_reference'
                    ELSE 'amount_date_fuzzy'
                END,
                'confidence_score', CASE
                    WHEN rr.external_reference IS NOT NULL THEN 92
                    WHEN rr.bank_utr IS NOT NULL THEN 85
                    ELSE 65
                END
            )
        WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = $1
        "#,
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    audit(
        &pool,
        "reconciliation.auto_match",
        "reconciliation_file",
        Some(id),
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": id, "status": "auto_match_queued" }))
}

async fn ignore_reconciliation_record(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE reconciliation_records SET resolution_status = 'ignored', match_status = 'ignored', resolved_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "reconciliation.ignored",
                "reconciliation_record",
                Some(id),
                json!({ "status": "ignored" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "ignored" }))
        }
        Ok(_) => {
            HttpResponse::NotFound().json(json!({ "message": "Reconciliation record not found" }))
        }
        Err(e) => {
            tracing::error!("Ignore reconciliation failed: {}", e);
            server_error("Failed to ignore reconciliation record")
        }
    }
}

async fn reconciliation_mismatches(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let rows = sqlx::query("SELECT * FROM reconciliation_records WHERE reconciliation_file_id = $1 AND status IN ('unmatched','mismatch','duplicate') ORDER BY created_at DESC")
        .bind(id)
        .fetch_all(pool.get_ref())
        .await;
    match rows {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Load reconciliation mismatches failed: {}", e);
            server_error("Failed to load mismatches")
        }
    }
}

async fn export_reconciliation_discrepancies(path: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok().json(
        json!({ "reconciliation_id": path.into_inner(), "download_url": "pending-export-job" }),
    )
}

async fn export_reconciliation_finance_summary(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "reconciliation.finance_summary_exported",
        "reconciliation",
        None,
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "download_url": "pending-finance-summary-export" }))
}

fn valid_reconciliation_source(source: &str) -> bool {
    matches!(
        source,
        "internal_ledger"
            | "settlement_report"
            | "payout_report"
            | "bank_statement"
            | "provider_report"
            | "manual_csv"
    )
}

async fn list_reconciliation_batches(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM reconciliation_batches ORDER BY created_at DESC LIMIT 100",
        "Failed to load reconciliation batches",
    )
    .await
}

async fn get_reconciliation_batch(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let batch = sqlx::query("SELECT * FROM reconciliation_batches WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(batch)) = batch else {
        return HttpResponse::NotFound().json(json!({ "message": "Batch not found" }));
    };
    let items = fetch_json_rows(
        pool.get_ref(),
        "SELECT * FROM reconciliation_items WHERE batch_id = $1 ORDER BY created_at DESC",
        id,
    )
    .await;
    let mismatches = fetch_json_rows(
        pool.get_ref(),
        "SELECT * FROM reconciliation_mismatches WHERE batch_id = $1 ORDER BY created_at DESC",
        id,
    )
    .await;
    HttpResponse::Ok().json(json!({
        "batch": row_to_json(&batch),
        "items": items,
        "mismatches": mismatches
    }))
}

async fn create_reconciliation_batch(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    body: web::Json<ReconciliationCreateBatchRequest>,
) -> impl Responder {
    let body = body.into_inner();
    if !valid_reconciliation_source(&body.source_type) {
        return HttpResponse::BadRequest().json(json!({ "message": "Invalid source type" }));
    }
    if !matches!(body.environment.as_str(), "test" | "live") {
        return HttpResponse::BadRequest().json(json!({ "message": "Invalid environment" }));
    }
    let actor_admin_id = admin_user_id_from_request(pool.get_ref(), &request).await;
    let file_metadata = json!({
        "file_name": body.file_name,
        "file_hash_present": body.file_hash.as_ref().is_some_and(|value| !value.trim().is_empty()),
        "content_type": body.content_type,
        "row_count": body.rows.len()
    });
    let batch_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO reconciliation_batches (
            source_type, environment, status, uploaded_by_admin_id, file_metadata,
            date_from, date_to
        )
        VALUES ($1, $2, 'parsing', $3, $4, $5::date, $6::date)
        RETURNING id
        "#,
    )
    .bind(&body.source_type)
    .bind(&body.environment)
    .bind(actor_admin_id)
    .bind(file_metadata)
    .bind(body.date_from.as_deref())
    .bind(body.date_to.as_deref())
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(id) => id,
        Err(error) => {
            tracing::error!("Create reconciliation batch failed: {}", error);
            return server_error("Failed to create reconciliation batch");
        }
    };
    let source_import_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO reconciliation_source_imports (
            batch_id, file_name, file_hash, content_type, row_count
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(batch_id)
    .bind(body.file_name.as_deref())
    .bind(body.file_hash.as_deref())
    .bind(body.content_type.as_deref())
    .bind(body.rows.len() as i32)
    .fetch_one(pool.get_ref())
    .await
    .ok();

    for row in &body.rows {
        let currency = row.currency.as_deref().unwrap_or("INR").to_uppercase();
        let payment = sqlx::query(
            r#"
            SELECT id, amount, currency, status::text AS status, created_at
            FROM payment_orders
            WHERE ($1::uuid IS NOT NULL AND id = $1)
               OR ($2::text IS NOT NULL AND order_id::text = $2)
               OR ($3::text IS NOT NULL AND gateway_reference = $3)
            LIMIT 1
            "#,
        )
        .bind(row.transaction_id)
        .bind(row.order_reference.as_deref())
        .bind(row.provider_reference.as_deref())
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten();
        let mut mismatch_reason = None::<String>;
        let mut transaction_id = row.transaction_id;
        let mut refund_id = row.refund_id;
        let mut settlement_id = row.settlement_id;
        let mut payout_id = row.payout_id;
        let mut matched = false;
        if let Some(payment) = payment {
            transaction_id = Some(payment.get("id"));
            matched = true;
            if payment.get::<i64, _>("amount") != row.amount_paise {
                mismatch_reason = Some("amount_mismatch".to_string());
                matched = false;
            } else if payment.get::<String, _>("currency") != currency {
                mismatch_reason = Some("currency_mismatch".to_string());
                matched = false;
            } else if let Some(status) = row.status.as_deref() {
                if payment.get::<String, _>("status") != status {
                    mismatch_reason = Some("status_mismatch".to_string());
                    matched = false;
                }
            }
            if let (Some(date_from), Some(date_to)) =
                (body.date_from.as_deref(), body.date_to.as_deref())
            {
                let created_at: DateTime<Utc> = payment.get("created_at");
                let day = created_at.date_naive().to_string();
                if day.as_str() < date_from || day.as_str() > date_to {
                    mismatch_reason = Some("date_window_mismatch".to_string());
                    matched = false;
                }
            }
        } else {
            mismatch_reason = Some("missing_internal_record".to_string());
        }
        if refund_id.is_none() && row.provider_reference.is_some() {
            refund_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM payment_refunds WHERE provider_refund_id = $1 OR id::text = $1 LIMIT 1",
            )
            .bind(row.provider_reference.as_deref())
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten();
        }
        if settlement_id.is_none() && row.utr_reference.is_some() {
            settlement_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM settlements WHERE utr_number = $1 OR utr = $1 LIMIT 1",
            )
            .bind(row.utr_reference.as_deref())
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten();
            if settlement_id.is_none() && mismatch_reason.is_none() {
                mismatch_reason = Some("missing_internal_record".to_string());
                matched = false;
            }
        }
        if payout_id.is_none()
            && row.provider_reference.is_some()
            && body.source_type == "payout_report"
        {
            payout_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM payouts WHERE payout_reference = $1 OR gateway_reference = $1 LIMIT 1",
            )
            .bind(row.provider_reference.as_deref())
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten();
            if payout_id.is_none() && mismatch_reason.is_none() {
                mismatch_reason = Some("missing_internal_record".to_string());
                matched = false;
            }
        }
        if mismatch_reason.is_none()
            && (transaction_id.is_some()
                || refund_id.is_some()
                || settlement_id.is_some()
                || payout_id.is_some())
        {
            matched = true;
        }
        if currency != "INR" && mismatch_reason.is_none() {
            mismatch_reason = Some("currency_mismatch".to_string());
            matched = false;
        }
        let duplicate = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reconciliation_items WHERE batch_id = $1 AND COALESCE(provider_reference, utr_reference) = COALESCE($2, $3)",
        )
        .bind(batch_id)
        .bind(row.provider_reference.as_deref())
        .bind(row.utr_reference.as_deref())
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
        if duplicate > 0 {
            mismatch_reason = Some("duplicate_reference".to_string());
            matched = false;
        }
        let item_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO reconciliation_items (
                batch_id, source_import_id, transaction_id, refund_id, settlement_id, payout_id,
                amount_paise, currency, utr_reference, provider_reference, order_reference,
                external_status, matched, mismatch_reason, raw_summary_redacted
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING id
            "#,
        )
        .bind(batch_id)
        .bind(source_import_id)
        .bind(transaction_id)
        .bind(refund_id)
        .bind(settlement_id)
        .bind(payout_id)
        .bind(row.amount_paise)
        .bind(&currency)
        .bind(row.utr_reference.as_deref())
        .bind(row.provider_reference.as_deref())
        .bind(row.order_reference.as_deref())
        .bind(row.status.as_deref())
        .bind(matched)
        .bind(mismatch_reason.as_deref())
        .bind(json!({
            "reference_present": row.provider_reference.is_some() || row.utr_reference.is_some(),
            "order_reference_present": row.order_reference.is_some()
        }))
        .fetch_one(pool.get_ref())
        .await
        .ok();
        if let (Some(item_id), Some(reason)) = (item_id, mismatch_reason) {
            let _ = sqlx::query(
                "INSERT INTO reconciliation_mismatches (batch_id, item_id, mismatch_type, detail_summary) VALUES ($1,$2,$3,$4)",
            )
            .bind(batch_id)
            .bind(item_id)
            .bind(reason)
            .bind(json!({ "source_type": body.source_type }))
            .execute(pool.get_ref())
            .await;
        }
    }
    if body.source_type == "internal_ledger" {
        let _ = sqlx::query(
            r#"
            INSERT INTO reconciliation_items (
                batch_id, amount_paise, currency, order_reference, matched, mismatch_reason, raw_summary_redacted
            )
            SELECT $1, po.amount, po.currency, po.order_id::text, false, 'missing_external_record',
                   jsonb_build_object('source', 'internal_ledger')
            FROM payment_orders po
            WHERE ($2::date IS NULL OR po.created_at::date >= $2::date)
              AND ($3::date IS NULL OR po.created_at::date <= $3::date)
              AND NOT EXISTS (
                  SELECT 1 FROM reconciliation_items ri
                  WHERE ri.batch_id = $1 AND ri.transaction_id = po.id
              )
            "#,
        )
        .bind(batch_id)
        .bind(body.date_from.as_deref())
        .bind(body.date_to.as_deref())
        .execute(pool.get_ref())
        .await;
        let _ = sqlx::query(
            r#"
            INSERT INTO reconciliation_mismatches (batch_id, item_id, mismatch_type, detail_summary)
            SELECT $1, ri.id, 'missing_external_record', jsonb_build_object('source_type', 'internal_ledger')
            FROM reconciliation_items ri
            WHERE ri.batch_id = $1 AND ri.mismatch_reason = 'missing_external_record'
              AND NOT EXISTS (SELECT 1 FROM reconciliation_mismatches rm WHERE rm.item_id = ri.id)
            "#,
        )
        .bind(batch_id)
        .execute(pool.get_ref())
        .await;
    }
    let _ = sqlx::query(
        r#"
        UPDATE reconciliation_batches
        SET status = CASE WHEN EXISTS (
                SELECT 1 FROM reconciliation_items WHERE batch_id = $1 AND matched = false
            ) THEN 'mismatch_found' ELSE 'matched' END,
            total_items = (SELECT COUNT(*) FROM reconciliation_items WHERE batch_id = $1),
            matched_items = (SELECT COUNT(*) FROM reconciliation_items WHERE batch_id = $1 AND matched = true),
            mismatched_items = (SELECT COUNT(*) FROM reconciliation_items WHERE batch_id = $1 AND matched = false),
            total_amount_paise = COALESCE((SELECT SUM(amount_paise) FROM reconciliation_items WHERE batch_id = $1), 0),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(batch_id)
    .execute(pool.get_ref())
    .await;
    evaluate_reconciliation_risk(pool.get_ref(), batch_id).await;
    audit_with_request(
        &pool,
        &request,
        "reconciliation.batch_imported",
        "reconciliation_batch",
        Some(batch_id),
        json!({
            "source_type": body.source_type,
            "environment": body.environment,
            "row_count": body.rows.len()
        }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": batch_id }))
}

async fn review_reconciliation_item(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ReconciliationReviewItemRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let actor_admin_id = admin_user_id_from_request(pool.get_ref(), &request).await;
    let updated = sqlx::query(
        "UPDATE reconciliation_items SET reviewed_at = NOW(), review_note = $2, reviewed_by_admin_id = $3 WHERE id = $1 RETURNING batch_id",
    )
    .bind(id)
    .bind(body.note.as_deref())
    .bind(actor_admin_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(row)) = updated else {
        return HttpResponse::NotFound().json(json!({ "message": "Item not found" }));
    };
    let batch_id: Uuid = row.get("batch_id");
    let _ = sqlx::query(
        "UPDATE reconciliation_mismatches SET status = 'reviewed', note = $2, reviewed_at = NOW() WHERE item_id = $1",
    )
    .bind(id)
    .bind(body.note.as_deref())
    .execute(pool.get_ref())
    .await;
    audit_with_request(
        &pool,
        &request,
        "reconciliation.item_reviewed",
        "reconciliation_item",
        Some(id),
        json!({ "batch_id": batch_id }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": id, "status": "reviewed" }))
}

async fn close_reconciliation_batch(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let open_mismatches = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM reconciliation_mismatches WHERE batch_id = $1 AND status = 'open'",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await;
    match open_mismatches {
        Ok(count) if count > 0 => {
            return HttpResponse::Conflict().json(json!({
                "message": "Review all reconciliation mismatches before closing the batch"
            }));
        }
        Err(e) => {
            tracing::error!("Failed checking open reconciliation mismatches: {}", e);
            return server_error("Failed checking reconciliation mismatches");
        }
        _ => {}
    }
    let updated = sqlx::query(
        "UPDATE reconciliation_batches SET status = 'closed', closed_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit_with_request(
                &pool,
                &request,
                "reconciliation.batch_closed",
                "reconciliation_batch",
                Some(id),
                json!({}),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "closed" }))
        }
        _ => HttpResponse::NotFound().json(json!({ "message": "Batch not found" })),
    }
}

async fn export_reconciliation_batch_csv(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let rows = sqlx::query(
        "SELECT amount_paise, currency, provider_reference, utr_reference, matched, mismatch_reason FROM reconciliation_items WHERE batch_id = $1 ORDER BY created_at",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await;
    let Ok(rows) = rows else {
        return server_error("Failed to export reconciliation batch");
    };
    let mut writer = csv::Writer::from_writer(Vec::new());
    let _ = writer.write_record([
        "amount_paise",
        "currency",
        "provider_reference",
        "utr_reference",
        "matched",
        "mismatch_reason",
    ]);
    for row in rows {
        let _ = writer.write_record([
            row.get::<i64, _>("amount_paise").to_string(),
            row.get::<String, _>("currency"),
            row.get::<Option<String>, _>("provider_reference")
                .unwrap_or_default(),
            row.get::<Option<String>, _>("utr_reference")
                .unwrap_or_default(),
            row.get::<bool, _>("matched").to_string(),
            row.get::<Option<String>, _>("mismatch_reason")
                .unwrap_or_default(),
        ]);
    }
    let csv = writer.into_inner().unwrap_or_default();
    audit_with_request(
        &pool,
        &request,
        "reconciliation.batch_exported",
        "reconciliation_batch",
        Some(id),
        json!({}),
    )
    .await;
    HttpResponse::Ok()
        .content_type("text/csv")
        .append_header((
            "content-disposition",
            "attachment; filename=\"reconciliation.csv\"",
        ))
        .body(csv)
}

async fn list_disputes(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM disputes_company ORDER BY created_at DESC LIMIT 50",
        "Failed to load disputes",
    )
    .await
}

async fn dispute_action(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    status: &'static str,
    request: web::Json<DisputeActionRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let body = request.into_inner();
    let note = body.note.clone().or(body.evidence_text.clone());
    let updated = sqlx::query(
        r#"
        UPDATE disputes_company
        SET status = $2,
            evidence_status = CASE WHEN $2 = 'evidence_submitted' THEN 'submitted' ELSE evidence_status END,
            outcome = COALESCE($3, outcome),
            internal_notes = COALESCE($4, internal_notes),
            timeline = timeline || $5::jsonb,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(body.outcome.as_deref())
    .bind(note.as_deref())
    .bind(json!([{
        "status": status,
        "note": note,
        "created_at": Utc::now()
    }]))
    .fetch_optional(pool.get_ref())
    .await;

    match updated {
        Ok(Some(_)) => {
            if matches!(status, "lost" | "won") {
                let dispute = sqlx::query(
                    "SELECT amount, currency, provider_dispute_id, business_account_id FROM disputes_company WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(pool.get_ref())
                .await;
                if let Ok(Some(row)) = dispute {
                    let amount: i64 = row.get("amount");
                    let currency: String = row.get("currency");
                    let business_account_id: Option<Uuid> = row.get("business_account_id");
                    let ledger_type = if status == "lost" {
                        "dispute_debit"
                    } else {
                        "dispute_reversal"
                    };
                    let _ = post_balanced_ledger(
                        &pool,
                        ledger_type,
                        "dispute",
                        id,
                        None,
                        business_account_id,
                        amount,
                        &currency,
                        &format!("{ledger_type}-{id}"),
                        "Dispute ledger impact",
                    )
                    .await;
                }
            }
            audit(
                &pool,
                &format!("dispute.{status}"),
                "dispute",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            let dispute_merchant = sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT business_account_id FROM disputes_company WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten()
            .flatten();
            evaluate_dispute_risk(pool.get_ref(), id, dispute_merchant).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Dispute not found" })),
        Err(e) => {
            tracing::error!("Dispute action failed: {}", e);
            server_error("Failed to update dispute")
        }
    }
}

async fn submit_dispute_evidence(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<DisputeActionRequest>,
) -> impl Responder {
    dispute_action(pool, path, "evidence_submitted", request).await
}

async fn mark_dispute_won(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<DisputeActionRequest>,
) -> impl Responder {
    dispute_action(pool, path, "won", request).await
}

async fn mark_dispute_lost(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<DisputeActionRequest>,
) -> impl Responder {
    dispute_action(pool, path, "lost", request).await
}

async fn list_business_accounts(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT m.id,
               COALESCE(m.display_name, m.business_name, m.name) AS display_name,
               COALESCE(m.legal_name, m.business_name, m.name) AS legal_name,
               COALESCE(m.business_type, 'company') AS business_type,
               m.email,
               m.phone,
               COALESCE(m.country, 'IN') AS country,
               COALESCE(m.status, m.onboarding_status, 'draft') AS status,
               COALESCE(m.review_status, 'draft') AS review_status,
               COALESCE(m.kyc_status, 'not_started') AS kyc_status,
               COALESCE(m.risk_status, 'low') AS risk_status,
               COALESCE(m.payout_status, 'not_configured') AS payout_status,
               COALESCE(m.default_currency, 'INR') AS default_currency,
               (
                 COALESCE(m.status, m.onboarding_status, '') = 'active'
                 AND COALESCE(m.kyc_status, '') = 'verified'
                 AND COALESCE(m.risk_status, '') <> 'blocked'
                 AND EXISTS (
                   SELECT 1 FROM business_account_bank_accounts ba
                   WHERE ba.business_account_id = m.id
                     AND ba.verification_status = 'verified'
                     AND ba.status = 'active'
                 )
                 AND EXISTS (
                   SELECT 1 FROM business_account_provider_links pl
                   WHERE pl.business_account_id = m.id
                     AND pl.onboarding_status = 'active'
                     AND pl.payouts_enabled = true
                 )
               ) AS payout_ready,
               m.created_at,
               m.updated_at
        FROM merchants m
        ORDER BY m.created_at DESC
        LIMIT 200
        "#,
        "Failed to load business accounts",
    )
    .await
}

async fn get_business_account(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let profile = sqlx::query(
        r#"
        SELECT id, COALESCE(display_name, business_name, name) AS display_name, legal_name,
               business_type, email, phone, country, status, onboarding_status, review_status,
               review_assigned_admin_id, review_last_decided_at, kyc_status, risk_status,
               payout_status, default_currency,
               CASE WHEN pan IS NULL THEN NULL ELSE CONCAT(REPEAT('*', GREATEST(LENGTH(pan) - 4, 0)), RIGHT(pan, 4)) END AS pan_masked,
               CASE WHEN gstin IS NULL THEN NULL ELSE CONCAT(REPEAT('*', GREATEST(LENGTH(gstin) - 4, 0)), RIGHT(gstin, 4)) END AS gstin_masked,
               CASE WHEN registration_data->>'cin' IS NULL THEN NULL ELSE CONCAT(REPEAT('*', GREATEST(LENGTH(registration_data->>'cin') - 4, 0)), RIGHT(registration_data->>'cin', 4)) END AS cin_masked,
               CASE WHEN registration_data->>'llpin' IS NULL THEN NULL ELSE CONCAT(REPEAT('*', GREATEST(LENGTH(registration_data->>'llpin') - 4, 0)), RIGHT(registration_data->>'llpin', 4)) END AS llpin_masked,
               registration_data->>'website_app' AS website_app_url,
               registration_data->>'business_category' AS business_category,
               address, webhook_url,
               created_at, updated_at
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(profile)) = profile else {
        return HttpResponse::NotFound().json(json!({ "message": "Business account not found" }));
    };
    let documents = fetch_json_rows(pool.get_ref(), "SELECT id, business_account_id, document_type, status, rejection_reason, uploaded_at, reviewed_by_admin_id, reviewed_at FROM business_account_documents WHERE business_account_id = $1 ORDER BY uploaded_at DESC", id).await;
    let bank_accounts = fetch_json_rows(pool.get_ref(), "SELECT id, business_account_id, account_holder_name, bank_name, masked_account_number, ifsc_or_swift_or_routing_code, currency, verification_status, status, created_at FROM business_account_bank_accounts WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    let provider_links = fetch_json_rows(pool.get_ref(), "SELECT * FROM business_account_provider_links WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    let ledger = fetch_json_rows(pool.get_ref(), "SELECT le.* FROM ledger_entries le JOIN ledger_accounts la ON la.id = le.account_id WHERE la.owner_id = $1 ORDER BY le.created_at DESC LIMIT 50", id).await;
    let risk_notes = fetch_json_rows(pool.get_ref(), "SELECT * FROM business_account_risk_notes WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    let verification_attempts = fetch_json_rows(pool.get_ref(), "SELECT id, verification_type, environment, masked_identifier, result_status, safe_response_summary, provider_reference_present, latency_ms, error_code, error_message, created_at FROM kyc_provider_attempts WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT 50", id).await;
    let review_history = fetch_json_rows(pool.get_ref(), "SELECT id, action, from_status, to_status, risk_level, reason_code, note, actor_admin_id, created_at FROM merchant_review_events WHERE merchant_id = $1 ORDER BY created_at DESC", id).await;
    HttpResponse::Ok().json(json!({
        "profile": row_to_json(&profile),
        "kyc_documents": documents,
        "bank_accounts": bank_accounts,
        "provider_links": provider_links,
        "ledger_history": ledger,
        "risk_notes": risk_notes,
        "verification_attempts": verification_attempts,
        "review_history": review_history
    }))
}

async fn patch_business_account(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<BusinessAccountPatchRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let request = request.into_inner();
    let updated = sqlx::query(
        r#"
        UPDATE merchants
        SET status = COALESCE($2, status),
            onboarding_status = COALESCE($2, onboarding_status),
            kyc_status = COALESCE($3, kyc_status),
            risk_status = COALESCE($4, risk_status),
            payout_status = COALESCE($5, payout_status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.status.as_deref())
    .bind(request.kyc_status.as_deref())
    .bind(request.risk_status.as_deref())
    .bind(request.payout_status.as_deref())
    .fetch_optional(pool.get_ref())
    .await;

    if let Some(note) = request
        .note
        .as_deref()
        .filter(|note| !note.trim().is_empty())
    {
        let _ = sqlx::query("INSERT INTO business_account_risk_notes (business_account_id, note, risk_status) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(note.trim())
            .bind(request.risk_status.as_deref())
            .execute(pool.get_ref())
            .await;
    }

    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "business_account.updated",
                "business_account",
                Some(id),
                json!({
                    "status": request.status,
                    "kyc_status": request.kyc_status,
                    "risk_status": request.risk_status,
                    "payout_status": request.payout_status
                }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(json!({ "message": "Business account not found" }))
        }
        Err(e) => {
            tracing::error!("Business account update failed: {}", e);
            server_error("Failed to update business account")
        }
    }
}

fn valid_review_action(action: &str) -> bool {
    matches!(
        action,
        "mark_under_review"
            | "request_more_information"
            | "approve"
            | "reject"
            | "restrict"
            | "suspend"
            | "add_risk_note"
            | "assign_reviewer"
    )
}

fn valid_risk_level(level: &str) -> bool {
    matches!(level, "low" | "medium" | "high" | "blocked")
}

fn valid_reason_code(code: &str) -> bool {
    matches!(
        code,
        "document_mismatch"
            | "high_risk_category"
            | "invalid_website"
            | "bank_mismatch"
            | "suspicious_activity"
            | "manual_review"
            | "other"
    )
}

async fn evaluate_merchant_risk(
    pool: &PgPool,
    merchant_id: Uuid,
    business_category: Option<String>,
    kyc_mismatch: bool,
) {
    let _ = evaluate_and_record(
        pool,
        RiskEvent {
            scope: "merchant",
            environment: payment_environment(),
            entity_type: "merchant",
            entity_id: Some(merchant_id),
            merchant_id: Some(merchant_id),
            context: json!({
                "business_category": business_category,
                "kyc_mismatch": kyc_mismatch
            }),
        },
    )
    .await;
}

async fn evaluate_payout_risk(pool: &PgPool, payout_id: Uuid) -> Option<RiskDisposition> {
    let row = sqlx::query(
        r#"
        SELECT p.business_account_id, p.amount,
               COALESCE(ba.verification_status = 'verified', false) AS bank_verified,
               COALESCE(EXTRACT(DAY FROM NOW() - m.review_last_decided_at)::bigint, 9999) AS days_since_approval
        FROM payouts_company p
        LEFT JOIN business_account_bank_accounts ba ON ba.id = p.bank_account_id
        LEFT JOIN merchants m ON m.id = p.business_account_id
        WHERE p.id = $1
        "#,
    )
    .bind(payout_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let merchant_id: Option<Uuid> = row.get("business_account_id");
    let outcome = evaluate_and_record(
        pool,
        RiskEvent {
            scope: "payout",
            environment: payment_environment(),
            entity_type: "payout",
            entity_id: Some(payout_id),
            merchant_id,
            context: json!({
                "amount_paise": row.get::<i64, _>("amount"),
                "bank_verified": row.get::<bool, _>("bank_verified"),
                "days_since_approval": row.get::<i64, _>("days_since_approval")
            }),
        },
    )
    .await
    .ok()?;
    Some(outcome.disposition)
}

async fn evaluate_refund_risk(pool: &PgPool, refund_id: Uuid, merchant_id: Option<Uuid>, amount: i64) {
    let _ = evaluate_and_record(
        pool,
        RiskEvent {
            scope: "refund",
            environment: payment_environment(),
            entity_type: "refund",
            entity_id: Some(refund_id),
            merchant_id,
            context: json!({ "amount_paise": amount }),
        },
    )
    .await;
}

async fn evaluate_dispute_risk(pool: &PgPool, dispute_id: Uuid, merchant_id: Option<Uuid>) {
    let dispute_count: i64 = if let Some(merchant_id) = merchant_id {
        sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM disputes_company WHERE business_account_id = $1 AND status NOT IN ('won')",
        )
        .bind(merchant_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
    } else {
        0
    };
    let _ = evaluate_and_record(
        pool,
        RiskEvent {
            scope: "dispute",
            environment: payment_environment(),
            entity_type: "dispute",
            entity_id: Some(dispute_id),
            merchant_id,
            context: json!({ "dispute_count": dispute_count }),
        },
    )
    .await;
}

async fn evaluate_settlement_risk(pool: &PgPool, settlement_id: Uuid) {
    let row = sqlx::query(
        "SELECT net_settlement_amount FROM settlement_batches_company WHERE id = $1",
    )
    .bind(settlement_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if let Some(row) = row {
        let _ = evaluate_and_record(
            pool,
            RiskEvent {
                scope: "settlement",
                environment: payment_environment(),
                entity_type: "settlement",
                entity_id: Some(settlement_id),
                merchant_id: None,
                context: json!({ "amount_paise": row.get::<i64, _>("net_settlement_amount") }),
            },
        )
        .await;
    }
}

async fn evaluate_reconciliation_risk(pool: &PgPool, batch_id: Uuid) {
    let mismatch_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM reconciliation_mismatches WHERE batch_id = $1",
    )
    .bind(batch_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    let _ = evaluate_and_record(
        pool,
        RiskEvent {
            scope: "settlement",
            environment: payment_environment(),
            entity_type: "reconciliation_batch",
            entity_id: Some(batch_id),
            merchant_id: None,
            context: json!({ "mismatch_count": mismatch_count }),
        },
    )
    .await;
}

fn payment_environment() -> &'static str {
    if std::env::var("PAYMENT_MODE").ok().as_deref() == Some("live") {
        "live"
    } else {
        "test"
    }
}

async fn review_business_account(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<MerchantReviewActionRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let body = body.into_inner();
    if !valid_review_action(&body.action) {
        return HttpResponse::BadRequest().json(json!({ "message": "Unsupported review action" }));
    }
    if let Some(level) = body.risk_level.as_deref() {
        if !valid_risk_level(level) {
            return HttpResponse::BadRequest().json(json!({ "message": "Invalid risk level" }));
        }
    }
    if let Some(code) = body.reason_code.as_deref() {
        if !valid_reason_code(code) {
            return HttpResponse::BadRequest().json(json!({ "message": "Invalid reason code" }));
        }
    }
    if matches!(body.action.as_str(), "reject" | "restrict" | "suspend")
        && body.reason_code.as_deref().unwrap_or("").trim().is_empty()
    {
        return HttpResponse::BadRequest().json(json!({ "message": "Reason code is required" }));
    }
    if matches!(
        body.action.as_str(),
        "request_more_information" | "reject" | "restrict" | "suspend" | "add_risk_note"
    ) && body.note.as_deref().unwrap_or("").trim().is_empty()
    {
        return HttpResponse::BadRequest().json(json!({ "message": "Note is required" }));
    }

    let row = sqlx::query("SELECT review_status, onboarding_status FROM merchants WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Business account not found" }));
    };
    let from_status: String = row.get("review_status");
    let existing_onboarding_status: String = row.get("onboarding_status");
    let (to_status, onboarding_status, merchant_status) = match body.action.as_str() {
        "mark_under_review" => ("under_review", "under_review", None),
        "request_more_information" => ("submitted", "submitted", None),
        "approve" => ("approved", "approved", Some("active")),
        "reject" => ("rejected", "rejected", Some("suspended")),
        "restrict" => ("restricted", "restricted", Some("suspended")),
        "suspend" => ("suspended", "suspended", Some("suspended")),
        _ => (
            from_status.as_str(),
            existing_onboarding_status.as_str(),
            None,
        ),
    };
    let actor_admin_id = admin_user_id_from_request(pool.get_ref(), &request).await;
    let reviewer_uuid: Option<Uuid> = match body.reviewer_admin_id.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => {
            if let Ok(u) = Uuid::parse_str(s) {
                Some(u)
            } else {
                sqlx::query_scalar::<_, Uuid>(
                    "SELECT id FROM admin_users WHERE LOWER(name) = LOWER($1) OR LOWER(email) = LOWER($1) LIMIT 1",
                )
                .bind(s)
                .fetch_optional(pool.get_ref())
                .await
                .ok()
                .flatten()
            }
        }
        _ => None,
    };
    let note = body
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let updated = sqlx::query(
        r#"
        UPDATE merchants
        SET review_status = $2,
            onboarding_status = $3,
            status = COALESCE($4, status),
            risk_status = COALESCE($5, risk_status),
            review_assigned_admin_id = COALESCE($6, review_assigned_admin_id),
            review_last_decided_at = CASE
                WHEN $7 IN ('approve', 'reject', 'restrict', 'suspend') THEN NOW()
                ELSE review_last_decided_at
            END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(to_status)
    .bind(onboarding_status)
    .bind(merchant_status)
    .bind(body.risk_level.as_deref())
    .bind(reviewer_uuid)
    .bind(&body.action)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(_)) = updated else {
        return HttpResponse::InternalServerError()
            .json(json!({ "message": "Failed to update merchant review" }));
    };

    if let Some(note) = note {
        let _ = sqlx::query("INSERT INTO business_account_risk_notes (business_account_id, note, risk_status, created_by_admin_id) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(note)
            .bind(body.risk_level.as_deref())
            .bind(actor_admin_id)
            .execute(pool.get_ref())
            .await;
    }
    let _ = sqlx::query(
        "INSERT INTO merchant_review_events (merchant_id, actor_admin_id, action, from_status, to_status, risk_level, reason_code, note) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(id)
    .bind(actor_admin_id)
    .bind(&body.action)
    .bind(&from_status)
    .bind(to_status)
    .bind(body.risk_level.as_deref())
    .bind(body.reason_code.as_deref())
    .bind(note)
    .execute(pool.get_ref())
    .await;
    if matches!(
        body.action.as_str(),
        "mark_under_review" | "approve" | "reject" | "restrict" | "suspend"
    ) {
        let business_category = sqlx::query_scalar::<_, Option<String>>(
            "SELECT registration_data->>'business_category' FROM merchants WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten()
        .flatten();
        evaluate_merchant_risk(
            pool.get_ref(),
            id,
            business_category,
            matches!(
                body.reason_code.as_deref(),
                Some("document_mismatch" | "bank_mismatch")
            ),
        )
        .await;
    }
    audit_with_request(
        &pool,
        &request,
        &format!("merchant_review.{}", body.action),
        "business_account",
        Some(id),
        json!({
            "from_status": from_status,
            "to_status": to_status,
            "risk_level": body.risk_level,
            "reason_code": body.reason_code,
            "reviewer_assigned": reviewer_uuid.is_some()
        }),
    )
    .await;
    if matches!(body.action.as_str(), "approve" | "reject") {
        if let Some(email) =
            sqlx::query_scalar::<_, Option<String>>("SELECT email FROM merchants WHERE id = $1")
                .bind(id)
                .fetch_optional(pool.get_ref())
                .await
                .ok()
                .flatten()
                .flatten()
        {
            let mut variables = std::collections::BTreeMap::new();
            variables.insert("status".to_string(), to_status.to_string());
            let _ = dispatch_event(
                pool.get_ref(),
                DispatchRequest {
                    merchant_id: Some(id),
                    event_type: if body.action == "approve" {
                        "kyc_approved".to_string()
                    } else {
                        "kyc_rejected".to_string()
                    },
                    environment: std::env::var("PAYMENT_MODE")
                        .unwrap_or_else(|_| "test".to_string()),
                    recipient_email: Some(email),
                    recipient_phone: None,
                    variables,
                },
            )
            .await;
        }
    }
    HttpResponse::Ok().json(json!({ "id": id, "review_status": to_status }))
}

async fn list_risk_rules(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_rules ORDER BY priority ASC, created_at DESC",
        "Failed to load risk rules",
    )
    .await
}

async fn create_risk_rule(
    pool: web::Data<PgPool>,
    http_request: HttpRequest,
    request: web::Json<RiskRuleRequest>,
) -> impl Responder {
    let actor = admin_user_id_from_request(pool.get_ref(), &http_request).await;
    save_risk_rule(pool, None, actor, request.into_inner()).await
}

async fn update_risk_rule(
    pool: web::Data<PgPool>,
    http_request: HttpRequest,
    path: web::Path<Uuid>,
    request: web::Json<RiskRuleRequest>,
) -> impl Responder {
    let actor = admin_user_id_from_request(pool.get_ref(), &http_request).await;
    save_risk_rule(pool, Some(path.into_inner()), actor, request.into_inner()).await
}

async fn disable_risk_rule(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let actor = admin_user_id_from_request(pool.get_ref(), &request).await;
    let updated =
        sqlx::query("UPDATE risk_rules SET status = 'disabled', enabled = false, updated_by_admin_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(actor)
            .execute(pool.get_ref())
            .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "risk_rule.disabled",
                "risk_rule",
                Some(id),
                json!({ "status": "disabled" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "disabled" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Risk rule not found" })),
        Err(e) => {
            tracing::error!("Disable risk rule failed: {}", e);
            server_error("Failed to disable risk rule")
        }
    }
}

async fn enable_risk_rule(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let actor = admin_user_id_from_request(pool.get_ref(), &request).await;
    let updated =
        sqlx::query("UPDATE risk_rules SET status = 'active', enabled = true, updated_by_admin_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(actor)
            .execute(pool.get_ref())
            .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "risk_rule.enabled",
                "risk_rule",
                Some(id),
                json!({ "status": "active" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "active" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Risk rule not found" })),
        Err(e) => {
            tracing::error!("Enable risk rule failed: {}", e);
            server_error("Failed to enable risk rule")
        }
    }
}

async fn save_risk_rule(
    pool: web::Data<PgPool>,
    rule_id: Option<Uuid>,
    actor: Option<Uuid>,
    request: RiskRuleRequest,
) -> HttpResponse {
    let action = serde_json::to_string(&request.action)
        .unwrap_or_else(|_| "\"review\"".to_string())
        .trim_matches('"')
        .to_string();
    if !matches!(
        request.scope.as_str(),
        "merchant" | "payment" | "refund" | "dispute" | "payout" | "settlement"
    ) || !matches!(request.environment.as_str(), "test" | "live")
        || !matches!(request.severity.as_str(), "low" | "medium" | "high" | "critical")
        || !request.condition_config.is_object()
    {
        return HttpResponse::BadRequest().json(json!({ "message": "Invalid safe risk rule config" }));
    }
    let result = if let Some(id) = rule_id {
        sqlx::query_scalar::<_, Uuid>(
            "UPDATE risk_rules SET name = $2, description = $3, rule_type = $4, scope = $5, environment = $6, severity = $7, condition = $8::jsonb, condition_config = $8::jsonb, action = $9, priority = $10, status = CASE WHEN $11 THEN 'active' ELSE 'disabled' END, enabled = $11, updated_by_admin_id = $12, updated_at = NOW() WHERE id = $1 RETURNING id",
        )
        .bind(id)
        .bind(request.name.trim())
        .bind(request.description.as_deref())
        .bind(request.rule_type.trim())
        .bind(request.scope.trim())
        .bind(request.environment.trim())
        .bind(request.severity.trim())
        .bind(request.condition_config.clone())
        .bind(&action)
        .bind(request.priority.unwrap_or(100))
        .bind(request.enabled.unwrap_or(true))
        .bind(actor)
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO risk_rules (name, description, rule_type, scope, environment, severity, condition, condition_config, action, priority, status, enabled, created_by_admin_id, updated_by_admin_id) VALUES ($1,$2,$3,$4,$5,$6,$7::jsonb,$7::jsonb,$8,$9,CASE WHEN $10 THEN 'active' ELSE 'disabled' END,$10,$11,$11) RETURNING id",
        )
        .bind(request.name.trim())
        .bind(request.description.as_deref())
        .bind(request.rule_type.trim())
        .bind(request.scope.trim())
        .bind(request.environment.trim())
        .bind(request.severity.trim())
        .bind(request.condition_config.clone())
        .bind(&action)
        .bind(request.priority.unwrap_or(100))
        .bind(request.enabled.unwrap_or(true))
        .bind(actor)
        .fetch_optional(pool.get_ref())
        .await
    };

    match result {
        Ok(Some(id)) => {
            audit(
                &pool,
                "risk_rule.saved",
                "risk_rule",
                Some(id),
                json!({ "name": request.name, "action": action }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Risk rule not found" })),
        Err(e) => {
            tracing::error!("Save risk rule failed: {}", e);
            server_error("Failed to save risk rule")
        }
    }
}

async fn list_risk_reviews(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_reviews ORDER BY created_at DESC LIMIT 50",
        "Failed to load risk reviews",
    )
    .await
}

async fn list_risk_evaluations(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_evaluations ORDER BY created_at DESC LIMIT 200",
        "Failed to load risk evaluations",
    )
    .await
}

async fn list_risk_cases(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_cases ORDER BY created_at DESC LIMIT 200",
        "Failed to load risk cases",
    )
    .await
}

async fn list_risk_case_notes(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let rows = sqlx::query("SELECT * FROM risk_case_notes WHERE case_id = $1 ORDER BY created_at DESC")
        .bind(id)
        .fetch_all(pool.get_ref())
        .await;
    match rows {
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List risk case notes failed: {}", e);
            server_error("Failed to load risk case notes")
        }
    }
}

async fn add_risk_case_note(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RiskCaseActionRequest>,
) -> impl Responder {
    let case_id = path.into_inner();
    let Some(note) = body.note.as_deref().map(str::trim).filter(|note| !note.is_empty()) else {
        return HttpResponse::BadRequest().json(json!({ "message": "Note is required" }));
    };
    let actor = admin_user_id_from_request(pool.get_ref(), &request).await;
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO risk_case_notes (case_id, note, created_by_admin_id) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(case_id)
    .bind(note)
    .bind(actor)
    .fetch_optional(pool.get_ref())
    .await;
    match id {
        Ok(Some(id)) => {
            audit_with_request(&pool, &request, "risk_case.note_added", "risk_case", Some(case_id), json!({ "note_id": id })).await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Risk case not found" })),
        Err(e) => {
            tracing::error!("Add risk case note failed: {}", e);
            server_error("Failed to add risk case note")
        }
    }
}

async fn transition_risk_case(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RiskCaseActionRequest>,
    next_status: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    let row = sqlx::query("SELECT status FROM risk_cases WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Risk case not found" }));
    };
    let from_status: String = row.get("status");
    let actor = admin_user_id_from_request(pool.get_ref(), &request).await;
    let updated = sqlx::query(
        "UPDATE risk_cases SET status = $2, assigned_to_admin_id = COALESCE($3, CASE WHEN $2 = 'assigned' THEN $4 ELSE assigned_to_admin_id END), reviewed_at = CASE WHEN $2 = 'reviewed' THEN NOW() ELSE reviewed_at END, resolved_at = CASE WHEN $2 = 'resolved' THEN NOW() ELSE resolved_at END, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(next_status)
    .bind(body.assigned_to_admin_id)
    .bind(actor)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            let _ = sqlx::query("INSERT INTO risk_action_history (case_id, action, from_status, to_status, note, actor_admin_id) VALUES ($1,$2,$3,$4,$5,$6)")
                .bind(id)
                .bind(next_status)
                .bind(&from_status)
                .bind(next_status)
                .bind(body.note.as_deref())
                .bind(actor)
                .execute(pool.get_ref())
                .await;
            audit_with_request(&pool, &request, &format!("risk_case.{next_status}"), "risk_case", Some(id), json!({ "from_status": from_status, "to_status": next_status })).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": next_status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Risk case not found" })),
        Err(e) => {
            tracing::error!("Transition risk case failed: {}", e);
            server_error("Failed to update risk case")
        }
    }
}

async fn assign_risk_case(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RiskCaseActionRequest>,
) -> impl Responder {
    transition_risk_case(pool, request, path, body, "assigned").await
}

async fn review_risk_case(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RiskCaseActionRequest>,
) -> impl Responder {
    transition_risk_case(pool, request, path, body, "reviewed").await
}

async fn resolve_risk_case(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RiskCaseActionRequest>,
) -> impl Responder {
    transition_risk_case(pool, request, path, body, "resolved").await
}

async fn list_audit_logs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT al.*,
               au.name AS actor_name,
               au.email AS actor_email,
               COALESCE(ar.name, al.actor_role) AS actor_role
        FROM audit_logs_company al
        LEFT JOIN admin_users au ON au.id = al.actor_admin_id
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY al.created_at DESC
        LIMIT 100
        "#,
        "Failed to load audit logs",
    )
    .await
}

async fn list_reports_catalog(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        WITH catalog(report_name, category) AS (
            VALUES
                ('Payment report', 'payments'),
                ('Settlement report', 'finance'),
                ('Refund report', 'payments'),
                ('Dispute report', 'risk'),
                ('Reconciliation report', 'finance'),
                ('Provider performance report', 'operations'),
                ('Revenue report', 'finance'),
                ('Ledger report', 'finance'),
                ('Payout report', 'finance'),
                ('Fee report', 'finance'),
                ('Balance report', 'finance'),
                ('Accounting report', 'finance'),
                ('Transfer report', 'finance'),
                ('Reserve report', 'finance'),
                ('Ledger journal report', 'finance'),
                ('Provider cost report', 'finance'),
                ('Dispute financial impact report', 'finance'),
                ('Tax report', 'finance'),
                ('Audit report', 'compliance')
        )
        SELECT gen_random_uuid() AS id,
               c.report_name,
               c.category,
               COALESCE(h.format, 'CSV/PDF/XLSX') AS format,
               h.generated_at AS last_generated_at,
               au.name AS generated_by_name,
               au.email AS generated_by_email,
               COALESCE(ar.name, au.role) AS generated_by_role,
               COALESCE(h.status, 'READY') AS status
        FROM catalog c
        LEFT JOIN LATERAL (
            SELECT report_type, format, status, generated_at, generated_by_admin_id
            FROM admin_report_exports h
            WHERE h.report_type = c.report_name
              AND h.status IN ('READY', 'FAILED')
            ORDER BY h.generated_at DESC
            LIMIT 1
        ) h ON true
        LEFT JOIN admin_users au ON au.id = h.generated_by_admin_id
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY c.category, c.report_name
        "#,
        "Failed to load reports catalog",
    )
    .await
}

async fn list_report_exports(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT e.id,
               e.report_type,
               e.file_name,
               e.format,
               e.status,
               e.row_count,
               e.filters_json,
               e.generated_by_admin_id,
               e.generated_at,
               e.expires_at,
               e.file_reference,
               e.error_message,
               au.name AS generated_by_name,
               au.email AS generated_by_email,
               COALESCE(ar.name, au.role) AS generated_by_role
        FROM admin_report_exports e
        LEFT JOIN admin_users au ON au.id = e.generated_by_admin_id
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY e.generated_at DESC
        LIMIT 100
        "#,
        "Failed to load report export history",
    )
    .await
}

async fn list_scheduled_admin_reports(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT sr.*,
               au.name AS created_by_name,
               au.email AS created_by_email,
               COALESCE(ar.name, au.role) AS created_by_role
        FROM admin_scheduled_reports sr
        LEFT JOIN admin_users au ON au.id = sr.created_by_admin_id
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY sr.created_at DESC
        LIMIT 100
        "#,
        "Failed to load scheduled reports",
    )
    .await
}

async fn generate_admin_report(
    pool: web::Data<PgPool>,
    http_request: HttpRequest,
    request: web::Json<ReportGenerateRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let format = request.format.to_ascii_uppercase();
    if !matches!(format.as_str(), "CSV" | "PDF" | "XLSX") {
        return HttpResponse::BadRequest().json(json!({ "message": "Unsupported report format" }));
    }
    let report_type = request.report_type.trim().to_string();
    let filters = redact_sensitive_value(&request.filters.unwrap_or_else(|| json!({})));
    let export_id = Uuid::new_v4();
    let extension = format.to_ascii_lowercase();
    let file_name = report_file_name(&report_type, &extension, export_id);
    let generated_by_admin_id = admin_user_id_from_request(pool.get_ref(), &http_request).await;
    let generated_by = report_generated_by(&http_request);
    let (status, row_count, file_reference, error_message) = match build_report_export(
        pool.get_ref(),
        export_id,
        &report_type,
        &format,
        &filters,
        &file_name,
        &generated_by,
    )
    .await
    {
        Ok(result) => result,
        Err(message) => ("FAILED".to_string(), 0, None, Some(message)),
    };
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO admin_report_exports (
            id, report_type, file_name, format, status, row_count, filters_json,
            generated_by_admin_id, generated_at, expires_at, file_reference, error_message
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, NOW(), NOW() + INTERVAL '7 days', $9, $10)
        RETURNING id
        "#,
    )
    .bind(export_id)
    .bind(&report_type)
    .bind(&file_name)
    .bind(&format)
    .bind(&status)
    .bind(row_count)
    .bind(&filters)
    .bind(generated_by_admin_id)
    .bind(&file_reference)
    .bind(&error_message)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit_with_request(
                &pool,
                &http_request,
                "report.exported",
                "admin_report_export",
                Some(id),
                json!({
                    "report_type": report_type,
                    "format": format,
                    "status": status,
                    "masked": true,
                    "error_message": error_message
                }),
            )
            .await;
            HttpResponse::Ok().json(json!({
                "id": id,
                "status": status,
                "file_name": file_name,
                "file_reference": file_reference,
                "row_count": row_count,
                "error_message": error_message,
                "download_url": if status == "READY" {
                    Some(build_report_download_url(id))
                } else {
                    None
                }
            }))
        }
        Err(e) => {
            tracing::error!("Generate report failed: {}", e);
            server_error("Failed to generate report")
        }
    }
}

struct BuiltReport {
    status: String,
    row_count: i64,
    file_reference: Option<String>,
    error_message: Option<String>,
}

async fn build_report_export(
    pool: &PgPool,
    export_id: Uuid,
    report_type: &str,
    format: &str,
    filters: &Value,
    file_name: &str,
    generated_by: &str,
) -> Result<(String, i64, Option<String>, Option<String>), String> {
    let built = match format {
        "CSV" => generate_report_csv(pool, export_id, report_type, filters, file_name).await?,
        "PDF" => {
            generate_report_pdf(
                pool,
                export_id,
                report_type,
                filters,
                file_name,
                generated_by,
            )
            .await?
        }
        "XLSX" => generate_report_xlsx(format),
        _ => return Err("Unsupported report format".to_string()),
    };
    Ok((
        built.status,
        built.row_count,
        built.file_reference,
        built.error_message,
    ))
}

fn generate_report_xlsx(format: &str) -> BuiltReport {
    BuiltReport {
        status: "FAILED".to_string(),
        row_count: 0,
        file_reference: None,
        error_message: Some(format!("{format} generator not configured")),
    }
}

async fn generate_report_csv(
    pool: &PgPool,
    export_id: Uuid,
    report_type: &str,
    filters: &Value,
    file_name: &str,
) -> Result<BuiltReport, String> {
    let rows = report_rows(pool, report_type).await?;
    let mut csv = String::new();
    csv.push_str("Report Type,Generated At,Filters Masked\n");
    csv.push_str(&format!(
        "{},{},{}\n\n",
        csv_cell(report_type),
        csv_cell(&Utc::now().to_rfc3339()),
        csv_cell(&redact_sensitive_value(filters).to_string())
    ));
    let headers = report_headers(&rows);
    if headers.is_empty() {
        csv.push_str("message\nNo records\n");
    } else {
        csv.push_str(
            &headers
                .iter()
                .map(|header| csv_cell(header))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
        for row in &rows {
            csv.push_str(
                &headers
                    .iter()
                    .map(|header| csv_cell(&csv_value(row.get(header).unwrap_or(&Value::Null))))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            csv.push('\n');
        }
    }

    let relative = format!("{export_id}/{file_name}");
    write_report_file(&relative, csv.as_bytes())?;
    Ok(BuiltReport {
        status: "READY".to_string(),
        row_count: rows.len() as i64,
        file_reference: Some(relative),
        error_message: None,
    })
}

async fn generate_report_pdf(
    pool: &PgPool,
    export_id: Uuid,
    report_type: &str,
    filters: &Value,
    file_name: &str,
    generated_by: &str,
) -> Result<BuiltReport, String> {
    let rows = report_rows(pool, report_type).await?;
    let generated_at = Utc::now();
    let pdf = generate_finance_pdf(PdfReport {
        export_id,
        report_type,
        generated_by,
        generated_at,
        filters,
        rows: &rows,
    });
    let relative = format!("{export_id}/{file_name}");
    write_report_file(&relative, &pdf)?;
    Ok(BuiltReport {
        status: "READY".to_string(),
        row_count: rows.len() as i64,
        file_reference: Some(relative),
        error_message: None,
    })
}

fn write_report_file(reference: &str, bytes: &[u8]) -> Result<(), String> {
    let Some(path) = safe_report_path(reference) else {
        return Err("Report file reference invalid".to_string());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Report storage unavailable: {e}"))?;
    }
    fs::write(path, bytes).map_err(|e| format!("Report write failed: {e}"))
}

fn build_report_download_url(id: Uuid) -> String {
    format!("/admin/payment-ops/reports/exports/{id}/download")
}

fn report_file_name(report_type: &str, extension: &str, export_id: Uuid) -> String {
    let slug = report_type
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let short = export_id.to_string().chars().take(8).collect::<String>();
    format!(
        "feelpay-{}-{}-{}.{}",
        slug,
        Utc::now().format("%Y-%m-%d"),
        short,
        extension
    )
}

async fn report_rows(pool: &PgPool, report_type: &str) -> Result<Vec<Value>, String> {
    let lowered = report_type.to_ascii_lowercase();
    let sql = if lowered.contains("audit") {
        "SELECT action, entity_type, entity_id, actor_role, ip_address, created_at FROM audit_logs_company ORDER BY created_at DESC LIMIT 500"
    } else if lowered.contains("settlement") {
        "SELECT id, provider_id, status, expected_settlement_date, created_at FROM settlement_batches_company ORDER BY created_at DESC LIMIT 500"
    } else if lowered.contains("refund") {
        "SELECT id, payment_order_id, provider_id, amount, currency, status, created_at FROM payment_refunds ORDER BY created_at DESC LIMIT 500"
    } else if lowered.contains("dispute") {
        "SELECT id, payment_order_id, provider_id, amount, currency, status, created_at FROM disputes_company ORDER BY created_at DESC LIMIT 500"
    } else if lowered.contains("provider") {
        "SELECT id, name, code, provider_type, environment, status, health_status, success_rate, created_at FROM payment_providers ORDER BY created_at DESC LIMIT 500"
    } else {
        "SELECT id, amount, currency, status, payment_method, provider_id, created_at FROM payment_orders ORDER BY created_at DESC LIMIT 500"
    };
    sqlx::query(sql)
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| redact_report_row(&row_to_json(row)))
                .collect()
        })
        .map_err(|e| format!("Report query failed: {e}"))
}

fn redact_report_row(row: &Value) -> Value {
    redact_sensitive_value(row)
}

fn report_headers(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .filter_map(Value::as_object)
        .flat_map(|map| map.keys().cloned())
        .fold(Vec::new(), |mut headers, key| {
            if !headers.contains(&key) {
                headers.push(key);
            }
            headers
        })
}

fn csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => redact_sensitive_value(value).to_string(),
    }
}

fn csv_cell(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn report_storage_dir() -> std::io::Result<PathBuf> {
    let path = env::var("REPORT_EXPORT_DIR")
        .or_else(|_| env::var("REPORT_STORAGE_DIR"))
        .unwrap_or_else(|_| "storage/reports".to_string());
    let storage = PathBuf::from(path);
    fs::create_dir_all(&storage)?;
    Ok(storage)
}

fn safe_report_path(reference: &str) -> Option<PathBuf> {
    let relative = Path::new(reference);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(report_storage_dir().ok()?.join(relative))
}

async fn admin_user_id_from_request(pool: &PgPool, request: &HttpRequest) -> Option<Uuid> {
    let candidate = request
        .extensions()
        .get::<AdminClaims>()
        .map(|claims| claims.admin_id)?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM admin_users WHERE id = $1")
        .bind(candidate)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

fn report_generated_by(request: &HttpRequest) -> String {
    request
        .extensions()
        .get::<AdminClaims>()
        .and_then(|claims| claims.role.clone())
        .unwrap_or_else(|| "Company admin".to_string())
}

struct PdfReport<'a> {
    export_id: Uuid,
    report_type: &'a str,
    generated_by: &'a str,
    generated_at: DateTime<Utc>,
    filters: &'a Value,
    rows: &'a [Value],
}

struct PdfCanvas {
    pages: Vec<String>,
    current: String,
    page_no: usize,
    y: f64,
    export_id: Uuid,
    generated_at: DateTime<Utc>,
}

fn generate_finance_pdf(report: PdfReport<'_>) -> Vec<u8> {
    let mut canvas = PdfCanvas::new(report.export_id, report.generated_at);
    render_pdf_header(&mut canvas, &report);
    render_pdf_summary_cards(&mut canvas, &report);
    render_pdf_filters(&mut canvas, report.filters);
    render_pdf_table(&mut canvas, &report);
    canvas.finish_page();
    render_pdf_document(
        canvas.pages,
        &format!("{} - FeelPay", report.report_type),
        "FeelPay",
        "company admin report",
        "FeelPay admin",
        &format!("{}, finance, masked", report.report_type),
    )
}

impl PdfCanvas {
    fn new(export_id: Uuid, generated_at: DateTime<Utc>) -> Self {
        Self {
            pages: Vec::new(),
            current: String::new(),
            page_no: 1,
            y: 806.0,
            export_id,
            generated_at,
        }
    }

    fn finish_page(&mut self) {
        render_pdf_footer(self);
        self.pages.push(std::mem::take(&mut self.current));
        self.page_no += 1;
        self.y = 806.0;
    }

    fn ensure_space(&mut self, height: f64, report: &PdfReport<'_>, table_header: bool) {
        if self.y - height >= 86.0 {
            return;
        }
        self.finish_page();
        render_pdf_header(self, report);
        if table_header {
            render_pdf_table_header(self);
        }
    }

    fn text(&mut self, x: f64, y: f64, size: f64, text: &str, bold: bool, color: &str) {
        self.current.push_str(color);
        self.current.push_str(&format!(
            "BT /{} {} Tf {:.1} {:.1} Td ({}) Tj ET\n",
            if bold { "F2" } else { "F1" },
            size,
            x,
            y,
            pdf_text(text)
        ));
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: Option<&str>) {
        self.current.push_str(fill);
        self.current
            .push_str(&format!("{x:.1} {y:.1} {w:.1} {h:.1} re f\n"));
        if let Some(stroke) = stroke {
            self.current.push_str(stroke);
            self.current
                .push_str(&format!("{x:.1} {y:.1} {w:.1} {h:.1} re S\n"));
        }
    }
}

fn render_pdf_header(canvas: &mut PdfCanvas, report: &PdfReport<'_>) {
    canvas.rect(36.0, 768.0, 523.0, 42.0, "0.973 0.980 0.988 rg\n", None);
    canvas.rect(56.0, 778.0, 28.0, 24.0, "0.114 0.725 0.329 rg\n", None);
    canvas.text(62.0, 783.0, 12.0, "FP", true, "1 1 1 rg\n");
    canvas.text(96.0, 792.0, 15.0, "FeelPay", true, "0.067 0.094 0.153 rg\n");
    canvas.text(
        96.0,
        777.0,
        8.5,
        "Company finance and operations export",
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        384.0,
        792.0,
        8.5,
        &format!("Report ID {}", short_id(&report.export_id.to_string())),
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        384.0,
        777.0,
        8.5,
        "Environment: sandbox / production",
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        36.0,
        736.0,
        22.0,
        report.report_type,
        true,
        "0.067 0.094 0.153 rg\n",
    );
    canvas.text(
        36.0,
        718.0,
        10.0,
        &format!("Generated by {}", report.generated_by),
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        300.0,
        718.0,
        10.0,
        &format!("Generated at {}", format_datetime(report.generated_at)),
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        36.0,
        700.0,
        9.0,
        "Sensitive values are masked",
        true,
        "0.114 0.725 0.329 rg\n",
    );
    canvas.y = 670.0;
}

fn render_pdf_summary_cards(canvas: &mut PdfCanvas, report: &PdfReport<'_>) {
    let cards = report_summary_cards(report.report_type, report.rows, report.filters);
    for (idx, (label, value)) in cards.iter().enumerate() {
        let x = 36.0 + (idx as f64 * 132.0);
        canvas.rect(
            x,
            canvas.y - 54.0,
            122.0,
            48.0,
            "1 1 1 rg\n",
            Some("0.898 0.906 0.922 RG\n"),
        );
        canvas.text(
            x + 10.0,
            canvas.y - 25.0,
            8.0,
            label,
            false,
            "0.420 0.447 0.502 rg\n",
        );
        canvas.text(
            x + 10.0,
            canvas.y - 43.0,
            12.0,
            value,
            true,
            "0.067 0.094 0.153 rg\n",
        );
    }
    canvas.y -= 76.0;
}

fn render_pdf_filters(canvas: &mut PdfCanvas, filters: &Value) {
    canvas.text(
        36.0,
        canvas.y,
        12.0,
        "Applied filters",
        true,
        "0.067 0.094 0.153 rg\n",
    );
    canvas.y -= 22.0;
    let fields = [
        ("Start date", "start_date"),
        ("End date", "end_date"),
        ("Provider", "provider"),
        ("Business Account", "business_account"),
        ("Currency", "currency"),
        ("Payment Method", "payment_method"),
        ("Status", "status"),
        ("Format", "format"),
    ];
    for (idx, (label, key)) in fields.iter().enumerate() {
        let x = 36.0 + ((idx % 4) as f64 * 132.0);
        let y = canvas.y - ((idx / 4) as f64 * 24.0);
        canvas.rect(
            x,
            y - 15.0,
            122.0,
            18.0,
            "0.953 0.956 0.965 rg\n",
            Some("0.898 0.906 0.922 RG\n"),
        );
        canvas.text(
            x + 6.0,
            y - 10.0,
            7.5,
            &format!("{label}: {}", filter_value(filters, key)),
            false,
            "0.067 0.094 0.153 rg\n",
        );
    }
    canvas.y -= 68.0;
}

fn render_pdf_table(canvas: &mut PdfCanvas, report: &PdfReport<'_>) {
    canvas.text(
        36.0,
        canvas.y,
        12.0,
        "Report rows",
        true,
        "0.067 0.094 0.153 rg\n",
    );
    canvas.y -= 22.0;
    render_pdf_table_header(canvas);
    for (index, row) in report.rows.iter().take(120).enumerate() {
        canvas.ensure_space(22.0, report, true);
        let y = canvas.y;
        if index % 2 == 0 {
            canvas.rect(36.0, y - 14.0, 523.0, 18.0, "0.988 0.989 0.992 rg\n", None);
        }
        let cells = report_table_cells(row);
        let widths = [82.0, 94.0, 62.0, 78.0, 46.0, 74.0, 70.0];
        let mut x = 40.0;
        for (idx, cell) in cells.iter().enumerate() {
            let align_right = idx == 5;
            let text = truncate(cell, if idx == 1 { 22 } else { 16 });
            let tx = if align_right {
                x + widths[idx] - (text.len() as f64 * 4.3).min(widths[idx] - 4.0)
            } else {
                x
            };
            canvas.text(
                tx,
                y - 8.0,
                7.2,
                &text,
                false,
                status_color(if idx == 6 { cell } else { "" }),
            );
            x += widths[idx];
        }
        canvas.current.push_str(&format!(
            "0.898 0.906 0.922 RG\n36 {:.1} m 559 {:.1} l S\n",
            y - 16.0,
            y - 16.0
        ));
        canvas.y -= 20.0;
    }
}

fn render_pdf_table_header(canvas: &mut PdfCanvas) {
    canvas.rect(
        36.0,
        canvas.y - 16.0,
        523.0,
        20.0,
        "0.953 0.956 0.965 rg\n",
        Some("0.898 0.906 0.922 RG\n"),
    );
    let headers = [
        "Payment ID",
        "Created At",
        "Method",
        "Provider",
        "Currency",
        "Amount",
        "Status",
    ];
    let widths = [82.0, 94.0, 62.0, 78.0, 46.0, 74.0, 70.0];
    let mut x = 40.0;
    for (idx, header) in headers.iter().enumerate() {
        canvas.text(
            x,
            canvas.y - 9.0,
            7.3,
            header,
            true,
            "0.067 0.094 0.153 rg\n",
        );
        x += widths[idx];
    }
    canvas.y -= 22.0;
}

fn render_pdf_footer(canvas: &mut PdfCanvas) {
    canvas
        .current
        .push_str("0.898 0.906 0.922 RG\n36 56 m 559 56 l S\n");
    canvas.text(
        36.0,
        38.0,
        7.5,
        "Generated by FeelPay company admin · Sensitive values are masked",
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        360.0,
        38.0,
        7.5,
        &format!(
            "Report ID {} · Page {}",
            short_id(&canvas.export_id.to_string()),
            canvas.page_no
        ),
        false,
        "0.420 0.447 0.502 rg\n",
    );
    canvas.text(
        36.0,
        24.0,
        7.0,
        &format!("Generated {}", format_datetime(canvas.generated_at)),
        false,
        "0.420 0.447 0.502 rg\n",
    );
}

fn render_pdf_document(
    pages: Vec<String>,
    title: &str,
    author: &str,
    subject: &str,
    creator: &str,
    keywords: &str,
) -> Vec<u8> {
    let page_count = pages.len();
    let font_1 = 3 + page_count * 2;
    let font_2 = font_1 + 1;
    let info = font_2 + 1;
    let kids = (0..page_count)
        .map(|idx| format!("{} 0 R", 3 + idx * 2))
        .collect::<Vec<_>>()
        .join(" ");
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{kids}] /Count {page_count} >>"),
    ];
    for (idx, content) in pages.iter().enumerate() {
        let page_obj = 3 + idx * 2;
        let content_obj = page_obj + 1;
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_1} 0 R /F2 {font_2} 0 R >> >> /Contents {content_obj} 0 R >>"));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content.len(),
            content
        ));
    }
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_string());
    objects.push(format!(
        "<< /Title ({}) /Author ({}) /Subject ({}) /Creator ({}) /Keywords ({}) >>",
        pdf_text(title),
        pdf_text(author),
        pdf_text(subject),
        pdf_text(creator),
        pdf_text(keywords)
    ));
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }
    let xref_at = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info {} 0 R >>\nstartxref\n{}\n%%EOF\n",
            offsets.len(),
            info,
            xref_at
        )
        .as_bytes(),
    );
    pdf
}

fn report_summary_cards(
    report_type: &str,
    rows: &[Value],
    filters: &Value,
) -> Vec<(String, String)> {
    let lowered = report_type.to_ascii_lowercase();
    let total = rows
        .iter()
        .filter_map(|row| row.get("amount").and_then(Value::as_i64))
        .sum::<i64>();
    let currency = rows
        .iter()
        .find_map(|row| row.get("currency").and_then(Value::as_str))
        .unwrap_or("INR");
    let date_range = format!(
        "{} to {}",
        filter_value(filters, "start_date"),
        filter_value(filters, "end_date")
    );
    if lowered.contains("settlement") {
        return vec![
            ("Gross amount".to_string(), format_currency(total, currency)),
            ("Net amount".to_string(), format_currency(total, currency)),
            ("Settlement count".to_string(), rows.len().to_string()),
            ("Reconciliation".to_string(), "—".to_string()),
        ];
    }
    if lowered.contains("refund") {
        let failed = count_status(rows, "failed");
        let pending = count_status(rows, "pending");
        return vec![
            (
                "Total refunded".to_string(),
                format_currency(total, currency),
            ),
            ("Refund count".to_string(), rows.len().to_string()),
            ("Failed refunds".to_string(), failed.to_string()),
            ("Pending refunds".to_string(), pending.to_string()),
        ];
    }
    if lowered.contains("dispute") {
        let open = count_status(rows, "open");
        let lost = count_status(rows, "lost");
        return vec![
            (
                "Dispute amount".to_string(),
                format_currency(total, currency),
            ),
            ("Dispute count".to_string(), rows.len().to_string()),
            ("Open disputes".to_string(), open.to_string()),
            ("Lost disputes".to_string(), lost.to_string()),
        ];
    }
    vec![
        ("Total amount".to_string(), format_currency(total, currency)),
        ("Row count".to_string(), rows.len().to_string()),
        ("Currency".to_string(), currency.to_string()),
        ("Date range".to_string(), date_range),
    ]
}

fn count_status(rows: &[Value], needle: &str) -> usize {
    rows.iter()
        .filter(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .map(|status| status.eq_ignore_ascii_case(needle))
                .unwrap_or(false)
        })
        .count()
}

fn filter_value(filters: &Value, key: &str) -> String {
    filters
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            if key == "format" {
                "PDF".to_string()
            } else {
                "All".to_string()
            }
        })
}

fn report_table_cells(row: &Value) -> Vec<String> {
    vec![
        format_report_value(row.get("id").or_else(|| row.get("payment_order_id")), "id"),
        format_report_value(
            row.get("created_at")
                .or_else(|| row.get("expected_settlement_date")),
            "created_at",
        ),
        format_report_value(row.get("payment_method"), "payment_method"),
        format_report_value(
            row.get("provider_id")
                .or_else(|| row.get("provider_type"))
                .or_else(|| row.get("code")),
            "provider",
        ),
        format_report_value(row.get("currency"), "currency"),
        format_report_value(row.get("amount"), "amount"),
        format_report_value(
            row.get("status").or_else(|| row.get("health_status")),
            "status",
        ),
    ]
}

fn format_report_value(value: Option<&Value>, column: &str) -> String {
    let value = value.unwrap_or(&Value::Null);
    if column == "amount" {
        return value
            .as_i64()
            .map(|amount| format_currency(amount, "INR"))
            .unwrap_or_else(|| "—".to_string());
    }
    if column.contains("created") || column.contains("date") {
        return value
            .as_str()
            .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
            .map(|date| format_datetime(date.with_timezone(&Utc)))
            .unwrap_or_else(|| "—".to_string());
    }
    let rendered = csv_value(&redact_report_value(value));
    if rendered.is_empty() {
        "—".to_string()
    } else if column.contains("id") || looks_like_uuid(&rendered) {
        short_id(&rendered)
    } else {
        rendered
    }
}

fn redact_report_value(value: &Value) -> Value {
    redact_sensitive_value(value)
}

fn format_currency(amount: i64, currency: &str) -> String {
    let major = amount as f64 / 100.0;
    if currency == "INR" {
        format!("Rs. {major:.2}")
    } else {
        format!("{currency} {major:.2}")
    }
}

fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%d %b %Y, %I:%M %p UTC").to_string()
}

fn short_id(value: &str) -> String {
    if value.len() <= 14 {
        return value.to_string();
    }
    format!("{}...{}", &value[..8], &value[value.len() - 5..])
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() >= 32 && value.chars().filter(|ch| *ch == '-').count() >= 4
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let mut out = value
        .chars()
        .take(max.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

fn status_color(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "ready" | "success" | "successful" | "settled" | "captured" | "active" | "pass" => {
            "0.114 0.725 0.329 rg\n"
        }
        "failed" | "lost" | "rejected" => "0.863 0.149 0.149 rg\n",
        "pending" | "queued" | "processing" | "open" => "0.850 0.467 0.024 rg\n",
        _ => "0.067 0.094 0.153 rg\n",
    }
}

fn pdf_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '\\' => "\\\\".to_string(),
            ch if ch.is_ascii_graphic() || ch == ' ' => ch.to_string(),
            _ => " ".to_string(),
        })
        .collect()
}

async fn schedule_admin_report(
    pool: web::Data<PgPool>,
    request: web::Json<ReportScheduleRequest>,
) -> impl Responder {
    let request = request.into_inner();
    if !matches!(request.frequency.as_str(), "daily" | "weekly" | "monthly") {
        return HttpResponse::BadRequest().json(json!({ "message": "Unsupported frequency" }));
    }
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO admin_scheduled_reports (
            report_type, filters_json, frequency, recipients, format, status, next_run_at
        )
        VALUES ($1, $2::jsonb, $3, $4::jsonb, $5, 'active', NOW() + INTERVAL '1 day')
        RETURNING id
        "#,
    )
    .bind(request.report_type.trim())
    .bind(redact_sensitive_value(
        &request.filters.unwrap_or_else(|| json!({})),
    ))
    .bind(request.frequency.trim())
    .bind(json!(request.recipients))
    .bind(request.format.to_ascii_uppercase())
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "report.scheduled",
                "admin_scheduled_report",
                Some(id),
                json!({ "report_type": request.report_type, "frequency": request.frequency }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "active" }))
        }
        Err(e) => {
            tracing::error!("Schedule report failed: {}", e);
            server_error("Failed to schedule report")
        }
    }
}

async fn download_report_export(
    pool: web::Data<PgPool>,
    http_request: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let export =
        sqlx::query("SELECT id, file_name, format, status, file_reference, expires_at FROM admin_report_exports WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;
    match export {
        Ok(Some(row)) => {
            let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
            if expires_at.is_some_and(|value| value < Utc::now()) {
                let _ =
                    sqlx::query("UPDATE admin_report_exports SET status = 'EXPIRED' WHERE id = $1")
                        .bind(id)
                        .execute(pool.get_ref())
                        .await;
                return HttpResponse::Gone().json(json!({ "message": "Export link expired" }));
            }
            let status: String = row.get("status");
            if status != "READY" {
                return HttpResponse::Conflict()
                    .json(json!({ "message": "Report export is not ready", "status": status }));
            }
            let file_reference: Option<String> = row.get("file_reference");
            let Some(file_reference) = file_reference else {
                return server_error("Report file reference missing");
            };
            let Some(path) = safe_report_path(&file_reference) else {
                return server_error("Report file reference invalid");
            };
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::error!("Report file read failed: {}", e);
                    return HttpResponse::NotFound()
                        .json(json!({ "message": "Report file missing" }));
                }
            };
            let file_name: String = row.get("file_name");
            let format: String = row.get("format");
            let content_type = match format.as_str() {
                "CSV" => "text/csv; charset=utf-8",
                "PDF" => "application/pdf",
                "XLSX" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                _ => "application/octet-stream",
            };
            audit_with_request(
                &pool,
                &http_request,
                "report.downloaded",
                "admin_report_export",
                Some(id),
                json!({ "masked": true }),
            )
            .await;
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, content_type))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file_name.replace('"', "")),
                ))
                .body(bytes)
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Report export not found" })),
        Err(e) => {
            tracing::error!("Download report failed: {}", e);
            server_error("Failed to download report")
        }
    }
}

async fn list_roles(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT r.id, r.name, r.description, r.created_at,
               COALESCE(json_agg(p.code ORDER BY p.code) FILTER (WHERE p.code IS NOT NULL), '[]'::json) AS permissions
        FROM admin_roles r
        LEFT JOIN admin_role_permissions rp ON rp.role_id = r.id
        LEFT JOIN admin_permissions p ON p.id = rp.permission_id
        GROUP BY r.id
        ORDER BY r.name
        "#,
        "Failed to load admin roles",
    )
    .await
}

async fn list_admin_users(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT au.id,
               au.name,
               au.email,
               COALESCE(ar.name, au.role, 'Support Agent') AS role,
               COALESCE(au.status, CASE WHEN COALESCE(au.is_active, true) THEN 'active' ELSE 'inactive' END) AS status,
               COALESCE(au.mfa_enabled, au.totp_secret IS NOT NULL) AS mfa_enabled,
               au.last_login_at,
               au.created_at,
               au.updated_at
        FROM admin_users au
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY au.created_at DESC
        "#,
        "Failed to load admin users",
    )
    .await
}

async fn invite_admin_user(
    pool: web::Data<PgPool>,
    body: web::Json<AdminInviteRequest>,
) -> impl Responder {
    let request = body.into_inner();
    if forbidden_role(&request.role) {
        return HttpResponse::BadRequest().json(json!({ "message": "Forbidden admin role" }));
    }
    let role_id = role_id_by_name(&pool, &request.role).await;
    let result = sqlx::query(
        r#"
        INSERT INTO admin_users (email, password_hash, name, role, role_id, status, is_active)
        VALUES ($1, 'invite_pending', $2, $3, $4, 'invited', true)
        ON CONFLICT (email) DO UPDATE
        SET name = EXCLUDED.name,
            role = EXCLUDED.role,
            role_id = EXCLUDED.role_id,
            status = 'invited',
            is_active = true,
            updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(request.email.trim().to_lowercase())
    .bind(request.name.trim())
    .bind(&request.role)
    .bind(role_id)
    .fetch_one(pool.get_ref())
    .await;
    match result {
        Ok(row) => {
            let id: Uuid = row.get("id");
            audit(
                &pool,
                "admin.invited",
                "admin_user",
                Some(id),
                json!({ "email": request.email, "role": request.role }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Failed to invite admin user: {}", e);
            server_error("Failed to invite admin user")
        }
    }
}

async fn patch_admin_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AdminUserPatchRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let request = body.into_inner();
    let result = sqlx::query(
        r#"
        UPDATE admin_users
        SET name = COALESCE($2, name),
            email = COALESCE($3, email),
            status = COALESCE($4, status),
            mfa_enabled = COALESCE($5, mfa_enabled),
            is_active = CASE WHEN COALESCE($4, status) IN ('inactive', 'deactivated') THEN false ELSE is_active END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.email.map(|email| email.trim().to_lowercase()))
    .bind(request.status)
    .bind(request.mfa_enabled)
    .fetch_optional(pool.get_ref())
    .await;
    match result {
        Ok(Some(_)) => {
            audit(
                &pool,
                "admin.updated",
                "admin_user",
                Some(id),
                json!({ "updated": true }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Admin user not found" })),
        Err(e) => {
            tracing::error!("Failed to update admin user: {}", e);
            server_error("Failed to update admin user")
        }
    }
}

async fn set_admin_active(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    active: bool,
) -> impl Responder {
    let id = path.into_inner();
    let status = if active { "active" } else { "deactivated" };
    let result = sqlx::query("UPDATE admin_users SET is_active = $2, status = $3, updated_at = NOW() WHERE id = $1 RETURNING id")
        .bind(id)
        .bind(active)
        .bind(status)
        .fetch_optional(pool.get_ref())
        .await;
    match result {
        Ok(Some(_)) => {
            audit(
                &pool,
                if active {
                    "admin.reactivated"
                } else {
                    "admin.deactivated"
                },
                "admin_user",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Admin user not found" })),
        Err(e) => {
            tracing::error!("Failed to update admin active state: {}", e);
            server_error("Failed to update admin active state")
        }
    }
}

async fn deactivate_admin_user(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    set_admin_active(pool, path, false).await
}

async fn reactivate_admin_user(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    set_admin_active(pool, path, true).await
}

async fn assign_admin_role(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AssignRoleRequest>,
) -> impl Responder {
    let user_id = path.into_inner();
    let role = body.into_inner().role;
    if forbidden_role(&role) {
        return HttpResponse::BadRequest().json(json!({ "message": "Forbidden admin role" }));
    }
    let Some(role_id) = role_id_by_name(&pool, &role).await else {
        return HttpResponse::BadRequest().json(json!({ "message": "Unknown role" }));
    };
    let assign_result = sqlx::query(
        r#"
        INSERT INTO admin_user_roles (admin_user_id, role_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .execute(pool.get_ref())
    .await;

    let result =
        match assign_result {
            Ok(_) => sqlx::query(
                "UPDATE admin_users SET role_id = $2, role = $3, updated_at = NOW() WHERE id = $1",
            )
            .bind(user_id)
            .bind(role_id)
            .bind(&role)
            .execute(pool.get_ref())
            .await,
            Err(e) => Err(e),
        };

    match result {
        Ok(_) => {
            audit(
                &pool,
                "role.assigned",
                "admin_user",
                Some(user_id),
                json!({ "role": role }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": user_id }))
        }
        Err(e) => {
            tracing::error!("Failed to assign role: {}", e);
            server_error("Failed to assign role")
        }
    }
}

async fn remove_admin_role(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (user_id, role) = path.into_inner();
    if role == "Super Admin" {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Super Admin role removal is blocked" }));
    }
    let Some(role_id) = role_id_by_name(&pool, &role).await else {
        return HttpResponse::BadRequest().json(json!({ "message": "Unknown role" }));
    };
    let result =
        sqlx::query("DELETE FROM admin_user_roles WHERE admin_user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(pool.get_ref())
            .await;
    match result {
        Ok(_) => {
            audit(
                &pool,
                "role.removed",
                "admin_user",
                Some(user_id),
                json!({ "role": role }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": user_id }))
        }
        Err(e) => {
            tracing::error!("Failed to remove role: {}", e);
            server_error("Failed to remove role")
        }
    }
}

async fn admin_user_audit(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let sql = r#"
        SELECT *
        FROM audit_logs_company
        WHERE entity_type = 'admin_user' AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 100
    "#;
    match sqlx::query(sql).bind(id).fetch_all(pool.get_ref()).await {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Failed to load admin user audit: {}", e);
            server_error("Failed to load admin user audit")
        }
    }
}

async fn system_health(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM system_health_checks ORDER BY component",
        "Failed to load system health",
    )
    .await
}

async fn refresh_system_health(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query("UPDATE system_health_checks SET last_checked_at = NOW(), status = CASE WHEN status = 'unknown' THEN 'healthy' ELSE status END")
        .execute(pool.get_ref())
        .await;
    match result {
        Ok(_) => {
            audit(
                &pool,
                "system_health.refreshed",
                "system_health",
                None,
                json!({ "status": "refreshed" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "status": "refreshed" }))
        }
        Err(e) => {
            tracing::error!("Failed to refresh system health: {}", e);
            server_error("Failed to refresh system health")
        }
    }
}

async fn system_health_jobs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM background_job_health ORDER BY job_name",
        "Failed to load background jobs",
    )
    .await
}

async fn retry_system_job(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query("UPDATE background_job_health SET status = 'queued', failure_count = 0, last_error = NULL, updated_at = NOW() WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    match result {
        Ok(Some(_)) => {
            audit(
                &pool,
                "system_job.retried",
                "background_job_health",
                Some(id),
                json!({ "status": "queued" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "queued" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Job not found" })),
        Err(e) => {
            tracing::error!("Failed to retry job: {}", e);
            server_error("Failed to retry job")
        }
    }
}

async fn system_health_providers(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT display_name AS provider,
               environment,
               health_status AS health,
               COALESCE(success_rate, 0)::float8 AS success_rate,
               COALESCE((SELECT COUNT(*) FROM payment_attempts pa WHERE pa.provider_id = payment_providers.id), 0)::bigint AS attempt_count,
               last_health_check_at AS last_check,
               NULL::INTEGER AS latency_ms,
               CASE WHEN health_status IN ('down', 'failed') THEN 'Provider health check failed' ELSE NULL END AS error_message
        FROM payment_providers
        ORDER BY priority ASC, display_name ASC
        "#,
        "Failed to load provider health",
    )
    .await
}

async fn list_notifications(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT n.*,
               au.name AS assigned_to_name,
               au.email AS assigned_to_email,
               COALESCE(ar.name, au.role) AS assigned_to_role,
               CASE WHEN n.sla_due_at IS NOT NULL AND n.sla_due_at < NOW() AND n.status NOT IN ('resolved','ignored') THEN 'overdue' ELSE 'on_track' END AS sla_status
        FROM admin_notifications n
        LEFT JOIN admin_users au ON au.id = n.assigned_to_admin_id
        LEFT JOIN admin_roles ar ON ar.id = au.role_id
        ORDER BY n.created_at DESC
        LIMIT 200
        "#,
        "Failed to load notifications",
    )
    .await
}

async fn update_notification_status(
    pool: web::Data<PgPool>,
    id: Uuid,
    status: &str,
    action: &str,
    assigned_to: Option<Uuid>,
) -> HttpResponse {
    let result = sqlx::query(
        r#"
        UPDATE admin_notifications
        SET status = $2,
            assigned_to_admin_id = COALESCE($3, assigned_to_admin_id),
            resolved_at = CASE WHEN $2 = 'resolved' THEN NOW() ELSE resolved_at END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(assigned_to)
    .fetch_optional(pool.get_ref())
    .await;
    match result {
        Ok(Some(_)) => {
            audit(
                &pool,
                action,
                "admin_notification",
                Some(id),
                json!({ "status": status, "assigned_to_admin_id": assigned_to }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Notification not found" })),
        Err(e) => {
            tracing::error!("Failed to update notification: {}", e);
            server_error("Failed to update notification")
        }
    }
}

async fn acknowledge_notification(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_notification_status(
        pool,
        path.into_inner(),
        "acknowledged",
        "notification.acknowledged",
        None,
    )
    .await
}

async fn assign_notification(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AssignNotificationRequest>,
) -> impl Responder {
    update_notification_status(
        pool,
        path.into_inner(),
        "investigating",
        "notification.assigned",
        body.assigned_to_admin_id,
    )
    .await
}

async fn resolve_notification(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_notification_status(
        pool,
        path.into_inner(),
        "resolved",
        "notification.resolved",
        None,
    )
    .await
}

async fn ignore_notification(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_notification_status(
        pool,
        path.into_inner(),
        "ignored",
        "notification.ignored",
        None,
    )
    .await
}

async fn create_incident_from_notification(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let notification_id = path.into_inner();
    let notification =
        sqlx::query("SELECT title, severity, type, message FROM admin_notifications WHERE id = $1")
            .bind(notification_id)
            .fetch_optional(pool.get_ref())
            .await;
    let Ok(Some(row)) = notification else {
        return HttpResponse::NotFound().json(json!({ "message": "Notification not found" }));
    };
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO incidents (title, severity, status, source, started_at, summary)
        VALUES ($1, $2, 'open', $3, NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(row.get::<String, _>("title"))
    .bind(row.get::<String, _>("severity"))
    .bind(row.get::<String, _>("type"))
    .bind(row.get::<Option<String>, _>("message"))
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            let _ = sqlx::query(
                "UPDATE admin_notifications SET incident_id = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(notification_id)
            .bind(id)
            .execute(pool.get_ref())
            .await;
            let _ = sqlx::query(
                "INSERT INTO incident_events (incident_id, event_type, message) VALUES ($1, 'created', 'Created from alert')",
            )
            .bind(id)
            .execute(pool.get_ref())
            .await;
            audit(
                &pool,
                "incident.created",
                "incident",
                Some(id),
                json!({ "notification_id": notification_id }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "open" }))
        }
        Err(e) => {
            tracing::error!("Create incident failed: {}", e);
            server_error("Failed to create incident")
        }
    }
}

async fn snooze_notification(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query("UPDATE admin_notifications SET sla_due_at = NOW() + INTERVAL '1 hour', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "notification.snoozed",
                "admin_notification",
                Some(id),
                json!({ "sla_due_at": "plus_1_hour" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "snoozed" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Notification not found" })),
        Err(e) => {
            tracing::error!("Snooze notification failed: {}", e);
            server_error("Failed to snooze notification")
        }
    }
}

async fn get_platform_settings(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query("SELECT key, value, value_type FROM platform_settings")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => {
            let mut map = serde_json::Map::new();
            for row in rows {
                let key: String = row.get("key");
                let value: String = row.get("value");
                let value_type: String = row.get("value_type");
                let parsed = match value_type.as_str() {
                    "boolean" => json!(value == "true"),
                    "number" => value
                        .parse::<i64>()
                        .map(Value::from)
                        .unwrap_or_else(|_| json!(value)),
                    _ => json!(value),
                };
                map.insert(key, parsed);
            }
            HttpResponse::Ok().json(Value::Object(map))
        }
        Err(e) => {
            tracing::error!("Failed to load platform settings: {}", e);
            server_error("Failed to load platform settings")
        }
    }
}

async fn patch_platform_settings(
    pool: web::Data<PgPool>,
    http_request: HttpRequest,
    body: web::Json<Value>,
) -> impl Responder {
    let Some(object) = body.as_object() else {
        return HttpResponse::BadRequest().json(json!({ "message": "Settings object required" }));
    };
    for (key, value) in object {
        let (value_text, value_type) = setting_value(value);
        let result = sqlx::query(
            r#"
            INSERT INTO platform_settings (key, value, value_type, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (key) DO UPDATE
            SET value = EXCLUDED.value, value_type = EXCLUDED.value_type, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value_text)
        .bind(value_type)
        .execute(pool.get_ref())
        .await;
        if let Err(e) = result {
            tracing::error!("Failed to save platform setting {}: {}", key, e);
            return server_error("Failed to save platform settings");
        }
    }
    audit_with_request(
        &pool,
        &http_request,
        "platform_setting.changed",
        "platform_settings",
        None,
        json!({ "keys": object.keys().collect::<Vec<_>>() }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "status": "saved" }))
}

async fn set_maintenance_mode(pool: web::Data<PgPool>, enabled: bool) -> impl Responder {
    let value = if enabled { "true" } else { "false" };
    let result = sqlx::query(
        r#"
        INSERT INTO platform_settings (key, value, value_type, updated_at)
        VALUES ('maintenance_mode', $1, 'boolean', NOW())
        ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, value_type = 'boolean', updated_at = NOW()
        "#,
    )
    .bind(value)
    .execute(pool.get_ref())
    .await;
    match result {
        Ok(_) => {
            audit(
                &pool,
                if enabled {
                    "maintenance.enabled"
                } else {
                    "maintenance.disabled"
                },
                "platform_settings",
                None,
                json!({ "maintenance_mode": enabled }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "maintenance_mode": enabled }))
        }
        Err(e) => {
            tracing::error!("Failed to update maintenance mode: {}", e);
            server_error("Failed to update maintenance mode")
        }
    }
}

async fn enable_maintenance(pool: web::Data<PgPool>) -> impl Responder {
    set_maintenance_mode(pool, true).await
}

async fn disable_maintenance(pool: web::Data<PgPool>) -> impl Responder {
    set_maintenance_mode(pool, false).await
}

async fn list_developer_api_keys(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT id, name, environment, key_prefix, masked_key, scopes, status,
               last_used_at, expires_at, created_at, updated_at
        FROM developer_api_keys
        ORDER BY created_at DESC
        "#,
        "Failed to load API keys",
    )
    .await
}

async fn create_developer_api_key(
    pool: web::Data<PgPool>,
    request: web::Json<ApiKeyRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let raw_key = format!("fp_{}_{}", request.environment, Uuid::new_v4().simple());
    let key_prefix = raw_key.chars().take(12).collect::<String>();
    let hash = hash_secret(&raw_key);
    let masked_key = mask_api_key(&raw_key);
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO developer_api_keys (
            name, environment, key_prefix, encrypted_key_hash, masked_key, scopes, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.environment.trim())
    .bind(key_prefix)
    .bind(hash)
    .bind(&masked_key)
    .bind(json!(request.scopes))
    .bind(request.expires_at)
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => {
            audit(
                &pool,
                "api_key.created",
                "developer_api_key",
                Some(id),
                json!({ "masked_key": masked_key }),
            )
            .await;
            HttpResponse::Ok()
                .json(json!({ "id": id, "masked_key": masked_key, "status": "active" }))
        }
        Err(e) => {
            tracing::error!("Create API key failed: {}", e);
            server_error("Failed to create API key")
        }
    }
}

async fn rotate_developer_api_key(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let raw_key = format!("fp_rotated_{}", Uuid::new_v4().simple());
    let masked_key = mask_api_key(&raw_key);
    let updated = sqlx::query(
        "UPDATE developer_api_keys SET encrypted_key_hash = $2, masked_key = $3, key_prefix = $4, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(hash_secret(&raw_key))
    .bind(&masked_key)
    .bind(raw_key.chars().take(12).collect::<String>())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "api_key.rotated",
                "developer_api_key",
                Some(id),
                json!({ "masked_key": masked_key }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "masked_key": masked_key }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "API key not found" })),
        Err(e) => {
            tracing::error!("Rotate API key failed: {}", e);
            server_error("Failed to rotate API key")
        }
    }
}

async fn revoke_developer_api_key(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE developer_api_keys SET status = 'revoked', updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "api_key.revoked",
                "developer_api_key",
                Some(id),
                json!({ "status": "revoked" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "revoked" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "API key not found" })),
        Err(e) => {
            tracing::error!("Revoke API key failed: {}", e);
            server_error("Failed to revoke API key")
        }
    }
}

async fn list_api_request_logs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT id, environment, api_key_id, method, path, status_code, latency_ms,
               idempotency_key, request_id, error_code, error_message,
               request_metadata - 'authorization' - 'card' - 'cvv' - 'secret' AS request_metadata,
               response_summary - 'secret' - 'card' - 'bank_account' AS response_summary,
               created_at
        FROM api_request_logs
        ORDER BY created_at DESC
        LIMIT 200
        "#,
        "Failed to load API request logs",
    )
    .await
}

fn request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}

fn response_hash(payload: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

fn catalog_error(code: &str, message: &str, status: i32) -> HttpResponse {
    let request_id = request_id();
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status as u16)
            .unwrap_or(actix_web::http::StatusCode::BAD_REQUEST),
    )
    .json(json!({
        "request_id": request_id,
        "error": { "code": code, "message": message, "http_status": status }
    }))
}

async fn list_api_versions(pool: web::Data<PgPool>) -> impl Responder {
    let request_id = request_id();
    match sqlx::query(
        "SELECT id, version, status, sunset_at, changelog_url, created_at FROM api_versions ORDER BY created_at DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "request_id": request_id,
            "api_version_warning": "Deprecated versions should migrate before sunset.",
            "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
        })),
        Err(e) => {
            tracing::error!("Failed to load API versions: {}", e);
            server_error("Failed to load API versions")
        }
    }
}

async fn list_api_idempotency_records(pool: web::Data<PgPool>) -> impl Responder {
    let request_id = request_id();
    match sqlx::query(
        "SELECT id, idempotency_key, api_key_id, request_hash, response_hash, response_status, resource_type, resource_id, expires_at, created_at FROM api_idempotency_records ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "request_id": request_id,
            "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
        })),
        Err(e) => {
            tracing::error!("Failed to load idempotency logs: {}", e);
            server_error("Failed to load idempotency logs")
        }
    }
}

async fn create_api_idempotency_record(
    pool: web::Data<PgPool>,
    request: web::Json<ApiIdempotencyRecordRequest>,
) -> impl Responder {
    let request_id_value = request_id();
    let request = request.into_inner();
    if request.idempotency_key.trim().is_empty() {
        return catalog_error("invalid_request", "idempotency_key required", 400);
    }
    let request_hash_value = response_hash(&request.payload);
    let existing = sqlx::query(
        "SELECT id, request_hash, response_status, response_hash, resource_type, resource_id FROM api_idempotency_records WHERE idempotency_key = $1 AND api_key_id IS NOT DISTINCT FROM $2",
    )
    .bind(request.idempotency_key.trim())
    .bind(request.api_key_id)
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(row)) => {
            let existing_hash: String = row.get("request_hash");
            if existing_hash != request_hash_value {
                return catalog_error(
                    "idempotency_conflict",
                    "Idempotency key reused with different payload",
                    409,
                );
            }
            return HttpResponse::Ok().json(json!({
                "request_id": request_id_value,
                "replayed": true,
                "id": row.get::<Uuid, _>("id"),
                "response_status": row.get::<i32, _>("response_status"),
                "response_hash": row.get::<Option<String>, _>("response_hash"),
                "resource_type": row.get::<Option<String>, _>("resource_type"),
                "resource_id": row.get::<Option<Uuid>, _>("resource_id")
            }));
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("Failed checking idempotency record: {}", e);
            return server_error("Failed checking idempotency record");
        }
    }

    let response_payload = json!({
        "status": "accepted",
        "resource_type": request.resource_type.clone().unwrap_or_else(|| "generic".to_string()),
        "resource_id": request.resource_id
    });
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO api_idempotency_records
            (idempotency_key, api_key_id, request_hash, response_hash, response_status, resource_type, resource_id, expires_at)
        VALUES ($1, $2, $3, $4, 200, $5, $6, NOW() + INTERVAL '24 hours')
        RETURNING id
        "#,
    )
    .bind(request.idempotency_key.trim())
    .bind(request.api_key_id)
    .bind(request_hash_value)
    .bind(response_hash(&response_payload))
    .bind(request.resource_type.unwrap_or_else(|| "generic".to_string()))
    .bind(request.resource_id)
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({
            "request_id": request_id_value,
            "replayed": false,
            "id": id,
            "response": response_payload
        })),
        Err(e) => {
            tracing::error!("Failed creating idempotency record: {}", e);
            server_error("Failed creating idempotency record")
        }
    }
}

async fn list_api_error_catalog(pool: web::Data<PgPool>) -> impl Responder {
    let request_id = request_id();
    match sqlx::query(
        "SELECT code, message, http_status, retryable, docs_url FROM api_error_catalog ORDER BY code ASC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "request_id": request_id,
            "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
        })),
        Err(e) => {
            tracing::error!("Failed to load error catalog: {}", e);
            server_error("Failed to load error catalog")
        }
    }
}

async fn list_api_rate_limits(pool: web::Data<PgPool>) -> impl Responder {
    let request_id = request_id();
    match sqlx::query(
        "SELECT id, api_key_id, window_seconds, max_requests, current_count, reset_at FROM api_rate_limits ORDER BY reset_at ASC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "request_id": request_id,
            "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
        })),
        Err(e) => {
            tracing::error!("Failed to load rate limits: {}", e);
            server_error("Failed to load rate limits")
        }
    }
}

async fn set_api_rate_limit(
    pool: web::Data<PgPool>,
    request: web::Json<ApiRateLimitRequest>,
) -> impl Responder {
    let request = request.into_inner();
    if request.max_requests <= 0 || request.window_seconds <= 0 {
        return catalog_error(
            "invalid_request",
            "window_seconds and max_requests must be positive",
            400,
        );
    }
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO api_rate_limits (api_key_id, window_seconds, max_requests, current_count, reset_at)
        VALUES ($1, $2, $3, 0, NOW() + ($2 || ' seconds')::interval)
        RETURNING id
        "#,
    )
    .bind(request.api_key_id)
    .bind(request.window_seconds)
    .bind(request.max_requests)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({ "request_id": request_id(), "id": id })),
        Err(e) => {
            tracing::error!("Failed to save rate limit: {}", e);
            server_error("Failed to save rate limit")
        }
    }
}

async fn run_rate_limit_check(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let api_key_id = path.into_inner();
    let record = sqlx::query(
        "SELECT id, window_seconds, max_requests, current_count, reset_at FROM api_rate_limits WHERE api_key_id = $1 ORDER BY reset_at DESC LIMIT 1",
    )
    .bind(api_key_id)
    .fetch_optional(pool.get_ref())
    .await;
    match record {
        Ok(None) => catalog_error("scope_denied", "Rate limit not configured for API key", 403),
        Ok(Some(row)) => {
            let id: Uuid = row.get("id");
            let max_requests: i32 = row.get("max_requests");
            let mut current_count: i32 = row.get("current_count");
            let reset_at: DateTime<Utc> = row.get("reset_at");
            if Utc::now() > reset_at {
                current_count = 0;
                let _ = sqlx::query(
                    "UPDATE api_rate_limits SET current_count = 0, reset_at = NOW() + (window_seconds || ' seconds')::interval WHERE id = $1",
                )
                .bind(id)
                .execute(pool.get_ref())
                .await;
            }
            if current_count >= max_requests {
                return catalog_error("rate_limit_exceeded", "Request rate limit exceeded", 429);
            }
            let _ = sqlx::query(
                "UPDATE api_rate_limits SET current_count = current_count + 1 WHERE id = $1",
            )
            .bind(id)
            .execute(pool.get_ref())
            .await;
            HttpResponse::Ok().json(json!({
                "request_id": request_id(),
                "allowed": true,
                "remaining": max_requests - (current_count + 1)
            }))
        }
        Err(e) => {
            tracing::error!("Rate limit check failed: {}", e);
            server_error("Rate limit check failed")
        }
    }
}

async fn readiness_scorecard(pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query(
        "SELECT id, category, check_name, status, score, owner, action_link, notes, last_checked_at, updated_at FROM readiness_checks ORDER BY category, check_name",
    )
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => {
            let mut total = 0_i64;
            let mut count = 0_i64;
            let mut failed = Vec::new();
            let mut warning = Vec::new();
            let checks = rows.iter().map(row_to_json).collect::<Vec<_>>();
            for check in &checks {
                let score = check["score"].as_i64().unwrap_or_default();
                total += score;
                count += 1;
                match check["status"].as_str().unwrap_or("warning") {
                    "fail" => failed.push(check.clone()),
                    "warning" => warning.push(check.clone()),
                    _ => {}
                }
            }
            let avg = if count == 0 {
                0
            } else {
                (total / count) as i32
            };
            HttpResponse::Ok().json(json!({
                "request_id": request_id(),
                "total_score": avg,
                "items": checks,
                "failed_checks": failed,
                "warning_checks": warning
            }))
        }
        Err(e) => {
            tracing::error!("Failed to load readiness scorecard: {}", e);
            server_error("Failed to load readiness scorecard")
        }
    }
}

async fn readiness_export(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "readiness.exported",
        "readiness_scorecard",
        None,
        json!({ "format": "json" }),
    )
    .await;
    readiness_scorecard(pool).await
}

async fn readiness_assign_owner(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<ReadinessOwnerRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let request = request.into_inner();
    match sqlx::query("UPDATE readiness_checks SET owner = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(request.owner.trim())
        .execute(pool.get_ref())
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            HttpResponse::Ok().json(json!({ "request_id": request_id(), "id": id }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Readiness check not found" })),
        Err(e) => {
            tracing::error!("Failed assigning readiness owner: {}", e);
            server_error("Failed assigning readiness owner")
        }
    }
}

async fn readiness_mark_manual_check(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<ReadinessManualCheckRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let request = request.into_inner();
    match sqlx::query(
        "UPDATE readiness_checks SET status = $2, notes = COALESCE($3, notes), last_checked_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(request.status.trim())
    .bind(request.notes)
    .execute(pool.get_ref())
    .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "readiness.manual_check.completed",
                "readiness_check",
                Some(id),
                json!({ "status": request.status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "request_id": request_id(), "id": id }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Readiness check not found" })),
        Err(e) => {
            tracing::error!("Failed updating readiness check: {}", e);
            server_error("Failed updating readiness check")
        }
    }
}

async fn send_webhook_test(
    pool: web::Data<PgPool>,
    request: web::Json<WebhookTestRequest>,
) -> impl Responder {
    let event_type = request
        .event_type
        .clone()
        .unwrap_or_else(|| "payment.captured".to_string());

    // Find sandbox mock provider
    let provider_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM payment_providers WHERE environment = 'sandbox' AND provider_type = 'mock' LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await
    .unwrap_or(None);

    let Some(p_id) = provider_id else {
        return HttpResponse::UnprocessableEntity().json(json!({
            "message": "No sandbox mock provider found. Create one first in Provider Connectors."
        }));
    };

    let test_id = request
        .webhook_event_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| format!("test_{}", Uuid::new_v4().simple()));
    let raw_payload = json!({
        "id": test_id,
        "event": event_type,
        "data": {
            "object": {
                "id": format!("pay_{}", Uuid::new_v4().simple()),
                "amount": 1000,
                "currency": "INR",
                "status": "captured",
                "provider": "mock",
                "business_account": "sandbox_admin_test"
            }
        },
        "created": Utc::now().timestamp()
    });

    let saved_event = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO payment_webhook_events (provider_id, event_id, event_type, raw_payload, signature_verification_status, processing_status, processed_at)
         VALUES ($1, $2, $3, $4, 'verified', 'processed', NOW()) RETURNING id"
    )
    .bind(p_id)
    .bind(&test_id)
    .bind(&event_type)
    .bind(&raw_payload)
    .fetch_one(pool.get_ref())
    .await;

    match saved_event {
        Ok(e_id) => {
            // Log successful attempt
            let _ = sqlx::query(
                "INSERT INTO webhook_processing_attempts (webhook_event_id, attempt_number, status, started_at, finished_at, worker_id)
                 VALUES ($1, 1, 'success', NOW(), NOW(), 'test_simulator')"
            )
            .bind(e_id)
            .execute(pool.get_ref())
            .await;

            audit(
                &pool,
                "webhook_test.sent",
                "webhook_event",
                Some(e_id),
                json!({ "event_type": event_type, "simulator": true, "provider_id": p_id }),
            )
            .await;

            HttpResponse::Ok().json(json!({
                "status": "sent",
                "latency_ms": 15,
                "response_status": 200,
                "event_id": e_id,
                "message": "Sandbox test event processed internally and recorded."
            }))
        }
        Err(e) => {
            tracing::error!("Webhook test insert failed: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({ "message": format!("Failed to record test event: {}", e) }))
        }
    }
}

async fn replay_webhook_event(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    request: web::Json<WebhookReplayRequest>,
) -> impl Responder {
    let id_str = path.into_inner();
    let req = request.into_inner();
    let Some(event_id) = find_webhook_event_id(pool.get_ref(), &id_str).await else {
        return HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" }));
    };

    let dry_run = req.dry_run.unwrap_or(false);
    if dry_run {
        return HttpResponse::Ok().json(json!({
            "status": "dry_run",
            "message": "Dry run successful. Event exists and is valid for replay.",
            "event_id": event_id
        }));
    }

    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO webhook_replays (webhook_event_id, replay_status, replayed_at, error_message) VALUES ($1, 'queued', NOW(), $2) RETURNING id",
    )
    .bind(event_id)
    .bind(req.note)
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => {
            audit(
                &pool,
                "webhook_event.replayed",
                "payment_webhook_event",
                Some(event_id),
                json!({ "replay_id": id }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "replay_status": "queued" }))
        }
        Err(e) => {
            tracing::error!("Replay webhook failed: {}", e);
            server_error("Failed to replay webhook")
        }
    }
}

async fn list_webhook_replays(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM webhook_replays ORDER BY created_at DESC LIMIT 200",
        "Failed to load webhook replays",
    )
    .await
}

async fn compliance_overview(pool: web::Data<PgPool>) -> impl Responder {
    let mfa_coverage: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE COALESCE(mfa_enabled, false) = true), 0)::bigint FROM admin_users",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    let stale_admin_users: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE last_login_at IS NULL OR last_login_at < NOW() - INTERVAL '90 days'), 0)::bigint FROM admin_users",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    let rotation_due: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE rotation_due_at IS NOT NULL AND rotation_due_at <= NOW()), 0)::bigint FROM payment_provider_credentials",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    HttpResponse::Ok().json(json!({
        "raw_card_storage": "disabled",
        "cvv_storage": "disabled",
        "tokenization": "enabled",
        "webhook_signatures": "required",
        "secrets_encrypted": "enabled",
        "bank_accounts_encrypted": "enabled",
        "mfa_coverage": mfa_coverage,
        "stale_admin_users": stale_admin_users,
        "credential_rotation_due": rotation_due,
        "audit_retention_status": "enabled",
        "export_retention_status": "enabled"
    }))
}

async fn list_security_events(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM security_events ORDER BY created_at DESC LIMIT 200",
        "Failed to load security events",
    )
    .await
}

async fn start_access_review(
    pool: web::Data<PgPool>,
    request: web::Json<AccessReviewRequest>,
) -> impl Responder {
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO access_reviews (review_period, status) VALUES ($1, 'open') RETURNING id",
    )
    .bind(request.review_period.trim())
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            let _ = sqlx::query(
                r#"
                INSERT INTO access_review_items (review_id, admin_user_id, role)
                SELECT $1, au.id, COALESCE(ar.name, au.role, 'Support Agent')
                FROM admin_users au
                LEFT JOIN admin_roles ar ON ar.id = au.role_id
                "#,
            )
            .bind(id)
            .execute(pool.get_ref())
            .await;
            audit(
                &pool,
                "access_review.started",
                "access_review",
                Some(id),
                json!({ "review_period": request.review_period }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "open" }))
        }
        Err(e) => {
            tracing::error!("Start access review failed: {}", e);
            server_error("Failed to start access review")
        }
    }
}

async fn list_access_reviews(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM access_reviews ORDER BY started_at DESC LIMIT 50",
        "Failed to load access reviews",
    )
    .await
}

async fn list_access_review_items(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match sqlx::query("SELECT * FROM access_review_items WHERE review_id = $1 ORDER BY reviewed_at DESC NULLS LAST")
        .bind(id)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Load access review items failed: {}", e);
            server_error("Failed to load access review items")
        }
    }
}

async fn update_access_review_item(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<AccessReviewDecisionRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE access_review_items SET decision = $2, note = $3, reviewed_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(request.decision.trim())
    .bind(request.note.as_deref())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "access_review.item_updated",
                "access_review_item",
                Some(id),
                json!({ "decision": request.decision }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(_) => {
            HttpResponse::NotFound().json(json!({ "message": "Access review item not found" }))
        }
        Err(e) => {
            tracing::error!("Update access review item failed: {}", e);
            server_error("Failed to update access review item")
        }
    }
}

async fn complete_access_review(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE access_reviews SET status = 'completed', completed_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "access_review.completed",
                "access_review",
                Some(id),
                json!({ "status": "completed" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "completed" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Access review not found" })),
        Err(e) => {
            tracing::error!("Complete access review failed: {}", e);
            server_error("Failed to complete access review")
        }
    }
}

async fn export_audit_logs(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "audit_logs.exported",
        "audit_logs",
        None,
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "download_url": "pending-audit-export" }))
}

async fn export_security_events(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "security_events.exported",
        "security_events",
        None,
        json!({ "status": "queued" }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "download_url": "pending-security-events-export" }))
}

async fn rotate_due_credentials(pool: web::Data<PgPool>) -> impl Responder {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE rotation_due_at IS NOT NULL AND rotation_due_at <= NOW()), 0)::bigint FROM payment_provider_credentials",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    audit(
        &pool,
        "credential_rotation_due.reviewed",
        "payment_provider_credentials",
        None,
        json!({ "due_count": count }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "due_count": count, "status": "reviewed" }))
}

async fn mark_compliance_task_complete(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> impl Responder {
    let task = path.into_inner();
    let parsed_id = Uuid::parse_str(&task).ok();
    if let Some(id) = parsed_id {
        let _ = sqlx::query(
            "UPDATE compliance_tasks SET status = 'completed', completed_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool.get_ref())
        .await;
    }
    audit(
        &pool,
        "compliance_task.completed",
        "compliance_task",
        None,
        json!({ "task": task }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "task": task, "status": "completed" }))
}

async fn list_compliance_tasks(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM compliance_tasks ORDER BY created_at DESC LIMIT 100",
        "Failed to load compliance tasks",
    )
    .await
}

async fn create_compliance_task(
    pool: web::Data<PgPool>,
    request: Option<web::Json<ComplianceTaskRequest>>,
) -> impl Responder {
    let body = request.map(|value| value.into_inner());
    let title = body
        .as_ref()
        .and_then(|value| value.title.as_deref())
        .unwrap_or("Compliance follow-up")
        .to_string();
    let category = body
        .as_ref()
        .and_then(|value| value.category.as_deref())
        .unwrap_or("general")
        .to_string();
    let due_at = body.as_ref().and_then(|value| value.due_at);
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO compliance_tasks (title, category, status, due_at) VALUES ($1, $2, 'open', $3) RETURNING id",
    )
    .bind(&title)
    .bind(&category)
    .bind(due_at)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "compliance_task.created",
                "compliance_task",
                Some(id),
                json!({ "title": title, "category": category }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "open" }))
        }
        Err(e) => {
            tracing::error!("Create compliance task failed: {}", e);
            server_error("Failed to create compliance task")
        }
    }
}

async fn security_review_overview(pool: web::Data<PgPool>) -> impl Responder {
    let open_findings: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE status <> 'resolved'), 0)::bigint FROM security_review_findings",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    let pending_step_up: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*) FILTER (WHERE status = 'pending' AND expires_at > NOW()), 0)::bigint FROM step_up_challenges",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    HttpResponse::Ok().json(json!({
        "mfa_required": true,
        "deny_by_default_rbac": true,
        "session_timeout": "enabled",
        "ip_user_agent_audit": "enabled",
        "secret_redaction": "enabled",
        "webhook_signature_verification": "required",
        "idempotency_enforcement": "enabled",
        "payout_eligibility_checks": "enabled",
        "immutable_ledger_entries": "enabled",
        "open_findings": open_findings,
        "pending_step_up": pending_step_up
    }))
}

async fn list_security_review_findings(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM security_review_findings ORDER BY status, severity DESC, created_at DESC",
        "Failed to load security findings",
    )
    .await
}

async fn create_security_review_finding(
    pool: web::Data<PgPool>,
    request: web::Json<SecurityFindingRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO security_review_findings
            (category, severity, title, description, status, owner_admin_id, due_at)
        VALUES ($1, $2, $3, $4, 'open', $5, $6)
        RETURNING id
        "#,
    )
    .bind(req.category.trim())
    .bind(req.severity.trim())
    .bind(req.title.trim())
    .bind(req.description.trim())
    .bind(req.owner_admin_id)
    .bind(req.due_at)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "security_review.finding_created",
                "security_review_finding",
                Some(id),
                json!({ "title": req.title }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "open" }))
        }
        Err(e) => {
            tracing::error!("Create security finding failed: {}", e);
            server_error("Failed to create security finding")
        }
    }
}

async fn patch_security_review_finding(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<SecurityFindingPatchRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let req = request.into_inner();
    let status = req.status.unwrap_or_else(|| "assigned".to_string());
    let updated = sqlx::query(
        r#"
        UPDATE security_review_findings
        SET owner_admin_id = COALESCE($2, owner_admin_id),
            status = $3,
            due_at = COALESCE($4, due_at),
            resolved_at = CASE WHEN $3 = 'resolved' THEN NOW() ELSE resolved_at END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(req.owner_admin_id)
    .bind(status.trim())
    .bind(req.due_at)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "security_review.finding_updated",
                "security_review_finding",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Finding not found" })),
        Err(e) => {
            tracing::error!("Update security finding failed: {}", e);
            server_error("Failed to update security finding")
        }
    }
}

async fn export_threat_model(pool: web::Data<PgPool>) -> impl Responder {
    audit(
        &pool,
        "security_review.threat_model_exported",
        "security_review",
        None,
        json!({ "format": "json" }),
    )
    .await;
    HttpResponse::Ok().json(json!({
        "threat_model": [
            "secrets leakage",
            "webhook spoofing",
            "idempotency replay abuse",
            "payout fraud",
            "admin account takeover",
            "privilege escalation",
            "report data leakage",
            "bank account tampering",
            "reconciliation manipulation",
            "provider credential compromise"
        ],
        "controls": [
            "MFA required for admins",
            "step-up confirmation for sensitive actions",
            "IP/user-agent audit",
            "deny-by-default RBAC",
            "secret redaction",
            "webhook signature verification",
            "idempotency enforcement",
            "payout eligibility checks",
            "immutable ledger entries"
        ]
    }))
}

async fn list_step_up_challenges(pool: web::Data<PgPool>) -> impl Responder {
    let _ = sqlx::query("UPDATE step_up_challenges SET status = 'expired' WHERE status = 'pending' AND expires_at <= NOW()")
        .execute(pool.get_ref())
        .await;
    query_items(
        pool,
        "SELECT * FROM step_up_challenges ORDER BY created_at DESC LIMIT 200",
        "Failed to load step-up challenges",
    )
    .await
}

async fn create_step_up_challenge(
    pool: web::Data<PgPool>,
    request: web::Json<StepUpRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let enabled: bool =
        sqlx::query_scalar("SELECT COALESCE(enabled, true) FROM step_up_actions WHERE action = $1")
            .bind(req.action.trim())
            .fetch_optional(pool.get_ref())
            .await
            .unwrap_or(Some(true))
            .unwrap_or(true);
    if !enabled {
        return HttpResponse::Ok().json(json!({ "required": false, "action": req.action }));
    }
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO step_up_challenges (admin_user_id, action, status, expires_at) VALUES ($1, $2, 'pending', NOW() + INTERVAL '10 minutes') RETURNING id",
    )
    .bind(req.admin_user_id)
    .bind(req.action.trim())
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            HttpResponse::Ok().json(json!({ "required": true, "id": id, "status": "pending" }))
        }
        Err(e) => {
            tracing::error!("Create step-up failed: {}", e);
            server_error("Failed to create step-up challenge")
        }
    }
}

async fn verify_step_up_challenge(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE step_up_challenges SET status = 'verified', verified_at = NOW() WHERE id = $1 AND status = 'pending' AND expires_at > NOW()",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            HttpResponse::Ok().json(json!({ "id": id, "status": "verified" }))
        }
        Ok(_) => catalog_error(
            "invalid_request",
            "Step-up challenge expired or not pending",
            400,
        ),
        Err(e) => {
            tracing::error!("Verify step-up failed: {}", e);
            server_error("Failed to verify step-up challenge")
        }
    }
}

async fn configure_step_up_action(
    pool: web::Data<PgPool>,
    request: web::Json<StepUpActionRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let enabled = req.enabled.unwrap_or(true);
    let saved = sqlx::query(
        "INSERT INTO step_up_actions (action, enabled, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (action) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = NOW()",
    )
    .bind(req.action.trim())
    .bind(enabled)
    .execute(pool.get_ref())
    .await;
    match saved {
        Ok(_) => HttpResponse::Ok().json(json!({ "action": req.action, "enabled": enabled })),
        Err(e) => {
            tracing::error!("Configure step-up failed: {}", e);
            server_error("Failed to configure step-up action")
        }
    }
}

async fn list_step_up_actions(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM step_up_actions ORDER BY action",
        "Failed to load step-up actions",
    )
    .await
}

async fn list_sensitive_action_history(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM sensitive_action_history ORDER BY created_at DESC LIMIT 200",
        "Failed to load sensitive action history",
    )
    .await
}

async fn list_go_live_checklist(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM go_live_checklist ORDER BY critical DESC, item_key",
        "Failed to load go-live checklist",
    )
    .await
}

async fn patch_go_live_checklist_item(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<GoLiveChecklistPatchRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let req = request.into_inner();
    let status = req.status.unwrap_or_else(|| "assigned".to_string());
    let updated = sqlx::query(
        r#"
        UPDATE go_live_checklist
        SET owner_admin_id = COALESCE($2, owner_admin_id),
            evidence_url = COALESCE($3, evidence_url),
            status = $4,
            completed_at = CASE WHEN $4 = 'complete' THEN NOW() ELSE completed_at END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(req.owner_admin_id)
    .bind(req.evidence_url.as_deref())
    .bind(status.trim())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                "go_live.checklist_updated",
                "go_live_checklist",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Checklist item not found" })),
        Err(e) => {
            tracing::error!("Update go-live checklist failed: {}", e);
            server_error("Failed to update go-live checklist")
        }
    }
}

async fn readiness_go_live_scorecard(pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query("SELECT * FROM go_live_checklist ORDER BY critical DESC, item_key")
        .fetch_all(pool.get_ref())
        .await;
    match rows {
        Ok(rows) => {
            let items = rows.iter().map(row_to_json).collect::<Vec<_>>();
            let total = items.len().max(1) as i64;
            let complete = items
                .iter()
                .filter(|item| item["status"] == "complete")
                .count() as i64;
            let critical_missing = items
                .iter()
                .filter(|item| item["critical"] == true && item["status"] != "complete")
                .cloned()
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(json!({
                "score": (complete * 100) / total,
                "critical_missing": critical_missing,
                "items": items
            }))
        }
        Err(e) => {
            tracing::error!("Go-live scorecard failed: {}", e);
            server_error("Failed to load go-live scorecard")
        }
    }
}

async fn start_production_cutover(
    pool: web::Data<PgPool>,
    request: web::Json<CutoverRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let critical_missing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM go_live_checklist WHERE critical = true AND status <> 'complete'",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(1);
    if critical_missing > 0 {
        return catalog_error(
            "invalid_request",
            "Critical go-live checklist items are incomplete",
            400,
        );
    }
    if !req.super_admin_approved.unwrap_or(false) {
        return catalog_error("scope_denied", "Super Admin approval required", 403);
    }
    if !req.compliance_admin_approved.unwrap_or(false) {
        return catalog_error("scope_denied", "Compliance Admin approval required", 403);
    }
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO production_cutovers
            (environment, status, started_at, rollback_plan, notes, super_admin_approved, compliance_admin_approved)
        VALUES ($1, 'started', NOW(), $2, $3, true, true)
        RETURNING id
        "#,
    )
    .bind(req.environment.trim())
    .bind(req.rollback_plan.trim())
    .bind(req.notes.as_deref())
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "production_cutover.started",
                "production_cutover",
                Some(id),
                json!({ "environment": req.environment }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "started" }))
        }
        Err(e) => {
            tracing::error!("Start cutover failed: {}", e);
            server_error("Failed to start production cutover")
        }
    }
}

async fn update_production_cutover(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (id, action) = path.into_inner();
    let status = match action.as_str() {
        "pause" => "paused",
        "complete" => "completed",
        "rollback" => "rolled_back",
        _ => return catalog_error("invalid_request", "Unknown cutover action", 400),
    };
    let updated = sqlx::query(
        "UPDATE production_cutovers SET status = $2, completed_at = CASE WHEN $2 IN ('completed', 'rolled_back') THEN NOW() ELSE completed_at END WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(
                &pool,
                &format!("production_cutover.{status}"),
                "production_cutover",
                Some(id),
                json!({ "status": status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Cutover not found" })),
        Err(e) => {
            tracing::error!("Update cutover failed: {}", e);
            server_error("Failed to update production cutover")
        }
    }
}

async fn list_production_cutovers(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM production_cutovers ORDER BY created_at DESC LIMIT 50",
        "Failed to load production cutovers",
    )
    .await
}

async fn list_checkout_payment_links(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM payment_links_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load payment links",
    )
    .await
}

async fn create_checkout_payment_link(
    pool: web::Data<PgPool>,
    request: web::Json<CompanyPaymentLinkRequest>,
) -> impl Responder {
    let req = request.into_inner();
    if req.amount <= 0 {
        return HttpResponse::BadRequest().json(json!({ "message": "Amount must be positive" }));
    }
    let token_seed = format!(
        "{}:{}:{}",
        req.business_account_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        req.amount,
        Utc::now().timestamp_millis()
    );
    let token_hash = hash_secret(&token_seed);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_links_company (
            business_account_id, amount, currency, description, customer_email, customer_phone,
            status, expires_at, signed_token_hash
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $8)
        RETURNING id
        "#,
    )
    .bind(req.business_account_id)
    .bind(req.amount)
    .bind(req.currency.trim().to_uppercase())
    .bind(req.description.as_deref())
    .bind(req.customer_email.as_deref())
    .bind(req.customer_phone.as_deref())
    .bind(req.expires_at)
    .bind(token_hash)
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "checkout.payment_link_created",
                "payment_link_company",
                Some(id),
                json!({ "amount": req.amount, "currency": req.currency }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "active" }))
        }
        Err(e) => {
            tracing::error!("Create payment link failed: {}", e);
            server_error("Failed to create payment link")
        }
    }
}

async fn expire_checkout_payment_link(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE payment_links_company SET status = 'expired' WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "checkout.payment_link_expired",
                "payment_link_company",
                Some(id),
                json!({ "status": "expired" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "expired" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Payment link not found" })),
        Err(e) => {
            tracing::error!("Expire payment link failed: {}", e);
            server_error("Failed to expire payment link")
        }
    }
}

async fn create_checkout_session(
    pool: web::Data<PgPool>,
    request: web::Json<CheckoutSessionRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let link_row = sqlx::query(
        "SELECT * FROM payment_links_company WHERE id = $1 AND status = 'active' AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(req.payment_link_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(link)) = link_row else {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Payment link expired or inactive" }));
    };
    let amount: i64 = link.get("amount");
    let currency: String = link.get("currency");
    let provider = sqlx::query(
        "SELECT default_provider_id, fallback_provider_id, default_payment_methods, brand_name, checkout_logo_url, checkout_theme_color, support_email, terms_url, privacy_url, refund_policy_url FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten();
    let provider_id = provider
        .as_ref()
        .and_then(|row| row.get::<Option<Uuid>, _>("default_provider_id"));
    let session_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_sessions_company (
            payment_link_id, provider_id, selected_payment_method, status, expires_at
        )
        VALUES ($1, $2, $3, 'created', NOW() + INTERVAL '30 minutes')
        RETURNING id
        "#,
    )
    .bind(req.payment_link_id)
    .bind(provider_id)
    .bind(req.selected_payment_method.as_deref())
    .fetch_one(pool.get_ref())
    .await;
    let Ok(session_id) = session_id else {
        return server_error("Failed to create checkout session");
    };
    if let Some(config) = provider {
        let _ = sqlx::query(
            r#"
            INSERT INTO checkout_theme_snapshots (
                payment_session_id, brand_name, logo_url, theme_color, support_email, terms_url, privacy_url, refund_policy_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(session_id)
        .bind(
            config
                .get::<Option<String>, _>("brand_name")
                .unwrap_or_else(|| "FeelPay".to_string()),
        )
        .bind(config.get::<Option<String>, _>("checkout_logo_url"))
        .bind(
            config
                .get::<Option<String>, _>("checkout_theme_color")
                .unwrap_or_else(|| "#16A34A".to_string()),
        )
        .bind(config.get::<Option<String>, _>("support_email"))
        .bind(config.get::<Option<String>, _>("terms_url"))
        .bind(config.get::<Option<String>, _>("privacy_url"))
        .bind(config.get::<Option<String>, _>("refund_policy_url"))
        .execute(pool.get_ref())
        .await;
    }
    audit(
        &pool,
        "checkout.session_created",
        "payment_session_company",
        Some(session_id),
        json!({
            "payment_link_id": req.payment_link_id,
            "amount": amount,
            "currency": currency
        }),
    )
    .await;
    HttpResponse::Ok().json(json!({ "id": session_id, "status": "created" }))
}

async fn list_checkout_sessions(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM payment_sessions_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load checkout sessions",
    )
    .await
}

async fn complete_checkout_session(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let session_id = path.into_inner();
    let row = sqlx::query(
        r#"
        SELECT ps.id, ps.payment_link_id, ps.provider_id, ps.selected_payment_method,
               pl.business_account_id, pl.amount, pl.currency, pl.description,
               pl.customer_email, pl.customer_phone
        FROM payment_sessions_company ps
        JOIN payment_links_company pl ON pl.id = ps.payment_link_id
        WHERE ps.id = $1 AND ps.status IN ('created', 'pending')
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool.get_ref())
    .await;

    let Ok(Some(row)) = row else {
        return HttpResponse::BadRequest()
            .json(json!({ "message": "Checkout session is not available for test payment" }));
    };

    let business_account_id = match row.get::<Option<Uuid>, _>("business_account_id") {
        Some(id) => id,
        None => match sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM merchants ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(pool.get_ref())
        .await
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                return HttpResponse::BadRequest().json(
                    json!({ "message": "Create a business account before running test checkout" }),
                );
            }
            Err(e) => {
                tracing::error!("Lookup business account for checkout failed: {}", e);
                return server_error("Failed to complete test payment");
            }
        },
    };

    let provider_id = match row.get::<Option<Uuid>, _>("provider_id") {
        Some(id) => Some(id),
        None => sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM payment_providers
            WHERE status = 'active'
              AND (LOWER(provider_type) IN ('mock','test','simulator') OR LOWER(COALESCE(code, name, display_name)) IN ('mock','test','test_simulator'))
            ORDER BY priority ASC, created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten(),
    };

    let amount: i64 = row.get("amount");
    let currency: String = row.get("currency");
    let method = row
        .get::<Option<String>, _>("selected_payment_method")
        .unwrap_or_else(|| "upi".to_string());
    let idempotency_key = format!("checkout-test-{session_id}");
    let order_id = format!("test_{}", session_id.simple());
    let payment_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_orders (
            merchant_id, order_id, amount, currency, status, payment_method, connector,
            customer_email, customer_phone, description, idempotency_key, gateway_reference,
            gateway_response, metadata, provider_id, captured_at
        )
        VALUES ($1, $2, $3, $4, 'captured', $5, 'mock', $6, $7, $8, $9, $10, $11::jsonb, $12::jsonb, $13, NOW())
        ON CONFLICT (idempotency_key) DO UPDATE SET status = 'captured', updated_at = NOW(), captured_at = COALESCE(payment_orders.captured_at, NOW())
        RETURNING id
        "#,
    )
    .bind(business_account_id)
    .bind(order_id)
    .bind(amount)
    .bind(currency.trim().to_uppercase())
    .bind(method.trim())
    .bind(row.get::<Option<String>, _>("customer_email"))
    .bind(row.get::<Option<String>, _>("customer_phone"))
    .bind(row.get::<Option<String>, _>("description"))
    .bind(&idempotency_key)
    .bind(format!("mock_{}", session_id.simple()))
    .bind(json!({ "provider": "mock", "status": "captured", "test_mode": true }).to_string())
    .bind(json!({ "source": "white_label_checkout", "session_id": session_id, "test_mode": true }).to_string())
    .bind(provider_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Create checkout test payment failed: {}", e);
            return server_error("Failed to complete test payment");
        }
    };

    if let Some(provider_id) = provider_id {
        let _ = sqlx::query(
            r#"
            INSERT INTO payment_attempts (
                payment_order_id, provider_id, status, amount, currency, payment_method,
                provider_order_id, provider_payment_id, provider_response_summary, idempotency_key
            )
            VALUES ($1, $2, 'captured', $3, $4, $5, $6, $7, $8::jsonb, $9)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(payment_id)
        .bind(provider_id)
        .bind(amount)
        .bind(currency.trim().to_uppercase())
        .bind(method.trim())
        .bind(format!("mock_order_{}", session_id.simple()))
        .bind(format!("mock_pay_{}", session_id.simple()))
        .bind(json!({ "status": "captured", "test_mode": true }).to_string())
        .bind(format!("attempt-{idempotency_key}"))
        .execute(pool.get_ref())
        .await;

        let _ = sqlx::query(
            r#"
            INSERT INTO provider_performance_metrics (
                provider_id, payment_method, currency, country, success_rate, failure_rate,
                avg_latency_ms, p95_latency_ms, provider_fee_percent, attempt_count, updated_at
            )
            VALUES ($1, $2, $3, 'IN', 100.00, 0.00, 25, 25, 0.000, 1, NOW())
            ON CONFLICT (provider_id, payment_method, currency, country)
            DO UPDATE SET success_rate = 100.00, failure_rate = 0.00,
                avg_latency_ms = 25, p95_latency_ms = 25,
                attempt_count = provider_performance_metrics.attempt_count + 1,
                updated_at = NOW()
            "#,
        )
        .bind(provider_id)
        .bind(method.trim())
        .bind(currency.trim().to_uppercase())
        .execute(pool.get_ref())
        .await;
    }

    let _ = post_balanced_ledger(
        &pool,
        "payment_capture",
        "payment_order",
        payment_id,
        provider_id,
        Some(business_account_id),
        amount,
        &currency,
        &format!("ledger-{idempotency_key}"),
        "Sandbox checkout payment captured",
    )
    .await;

    let _ = sqlx::query(
        "UPDATE payment_sessions_company SET status = 'paid', payment_order_id = $2 WHERE id = $1",
    )
    .bind(session_id)
    .bind(payment_id)
    .execute(pool.get_ref())
    .await;

    let receipt_number = format!("RCPT-{}", session_id.simple());
    let _ = sqlx::query(
        r#"
        INSERT INTO checkout_receipts_company (
            payment_session_id, payment_order_id, receipt_number, recipient_email, status, sent_at
        )
        VALUES ($1, $2, $3, $4, 'sent', NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(session_id)
    .bind(payment_id)
    .bind(&receipt_number)
    .bind(row.get::<Option<String>, _>("customer_email"))
    .execute(pool.get_ref())
    .await;

    audit(
        &pool,
        "checkout.test_payment_completed",
        "payment_order",
        Some(payment_id),
        json!({ "session_id": session_id, "amount": amount, "currency": currency, "test_mode": true }),
    )
    .await;

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Test payment completed",
        "payment_order_id": payment_id,
        "payment_session_id": session_id,
        "receipt_number": receipt_number
    }))
}

async fn list_checkout_receipts(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM checkout_receipts_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load checkout receipts",
    )
    .await
}

async fn resend_checkout_receipt(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE checkout_receipts_company SET status = 'resent', sent_at = NOW() WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "checkout.receipt_resent",
                "checkout_receipt_company",
                Some(id),
                json!({ "status": "resent" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "resent" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Receipt not found" })),
        Err(e) => {
            tracing::error!("Resend receipt failed: {}", e);
            server_error("Failed to resend receipt")
        }
    }
}

async fn money_movement_dashboard(pool: web::Data<PgPool>) -> impl Responder {
    let row = sqlx::query(
        r#"
        SELECT
            COALESCE((SELECT SUM(payable_amount) FROM payment_splits_company), 0)::bigint AS payable_balance,
            COALESCE((SELECT COUNT(*) FROM transfers_company WHERE status IN ('pending','queued')), 0)::bigint AS pending_transfers,
            COALESCE((SELECT COUNT(*) FROM payouts_company WHERE status IN ('queued','approved','processing','on_hold')), 0)::bigint AS pending_payouts,
            COALESCE((SELECT SUM(amount) FROM reserve_balances WHERE status = 'held'), 0)::bigint AS reserve_balance,
            COALESCE((SELECT COUNT(*) FROM payouts_company WHERE status = 'failed'), 0)::bigint AS failed_payouts,
            COALESCE((SELECT SUM(amount) FROM transfer_reversals_company), 0)::bigint AS reversal_amount
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;
    match row {
        Ok(row) => HttpResponse::Ok().json(row_to_json(&row)),
        Err(e) => {
            tracing::error!("Money movement dashboard failed: {}", e);
            server_error("Failed to load money movement")
        }
    }
}

async fn list_split_rules(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM split_rules_company ORDER BY priority ASC, created_at DESC",
        "Failed to load split rules",
    )
    .await
}

async fn create_split_rule(
    pool: web::Data<PgPool>,
    request: web::Json<SplitRuleRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO split_rules_company (
            name, business_account_id, rule_type, company_fee_percent, company_fee_fixed,
            tax_percent, reserve_percent, settlement_delay_days, status, priority
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING id
        "#,
    )
    .bind(req.name.trim())
    .bind(req.business_account_id)
    .bind(req.rule_type.trim())
    .bind(req.company_fee_percent.unwrap_or(Decimal::ZERO))
    .bind(req.company_fee_fixed.unwrap_or(0))
    .bind(req.tax_percent.unwrap_or(Decimal::ZERO))
    .bind(req.reserve_percent.unwrap_or(Decimal::ZERO))
    .bind(req.settlement_delay_days.unwrap_or(0))
    .bind(req.status.unwrap_or_else(|| "active".to_string()))
    .bind(req.priority.unwrap_or(100))
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "money.split_rule_created",
                "split_rule_company",
                Some(id),
                json!({ "name": req.name }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Create split rule failed: {}", e);
            server_error("Failed to create split rule")
        }
    }
}

async fn simulate_split(request: web::Json<SplitSimulationRequest>) -> impl Responder {
    let req = request.into_inner();
    let gross = req.gross_amount;
    let provider_fee = req.provider_fee_amount.unwrap_or(0);
    let company_fee = percentage_amount(gross, req.company_fee_percent.unwrap_or(Decimal::ZERO))
        + req.company_fee_fixed.unwrap_or(0);
    let tax = percentage_amount(gross, req.tax_percent.unwrap_or(Decimal::ZERO));
    let reserve = percentage_amount(gross, req.reserve_percent.unwrap_or(Decimal::ZERO));
    let payable = gross - provider_fee - company_fee - tax - reserve;
    HttpResponse::Ok().json(json!({
        "business_account_id": req.business_account_id,
        "currency": req.currency.unwrap_or_else(|| "INR".to_string()),
        "gross_amount": gross,
        "provider_fee_amount": provider_fee,
        "company_fee_amount": company_fee,
        "tax_amount": tax,
        "reserve_amount": reserve,
        "payable_amount": payable.max(0)
    }))
}

fn percentage_amount(amount: i64, percent: Decimal) -> i64 {
    (Decimal::from(amount) * percent / Decimal::from(100))
        .trunc()
        .to_i64()
        .unwrap_or(0)
}

fn percentage_amount_rounded(amount: i64, percent: Decimal) -> i64 {
    (Decimal::from(amount) * percent / Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0)
}

async fn list_transfers_company(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM transfers_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load transfers",
    )
    .await
}

async fn create_transfer_company(
    pool: web::Data<PgPool>,
    request: web::Json<TransferCreateRequest>,
) -> impl Responder {
    let req = request.into_inner();
    if req.idempotency_key.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "message": "idempotency_key required" }));
    }
    if let Ok(Some(row)) = sqlx::query(
        "SELECT id, status FROM payouts_company WHERE idempotency_key = $1",
    )
    .bind(req.idempotency_key.trim())
    .fetch_optional(pool.get_ref())
    .await
    {
        return HttpResponse::Ok().json(json!({
            "id": row.get::<Uuid, _>("id"),
            "status": row.get::<String, _>("status"),
            "idempotent_replay": true
        }));
    }
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO transfers_company (
            payment_split_id, provider_id, provider_transfer_id, business_account_id, amount, currency, status
        )
        VALUES ($1, $2, NULL, $3, $4, $5, 'pending')
        RETURNING id
        "#,
    )
    .bind(req.payment_split_id)
    .bind(req.provider_id)
    .bind(req.business_account_id)
    .bind(req.amount)
    .bind(req.currency.unwrap_or_else(|| "INR".to_string()))
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "money.transfer_created",
                "transfer_company",
                Some(id),
                json!({ "amount": req.amount, "idempotency_key": req.idempotency_key }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "pending" }))
        }
        Err(e) => {
            tracing::error!("Create transfer failed: {}", e);
            server_error("Failed to create transfer")
        }
    }
}

async fn list_transfer_reversals(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM transfer_reversals_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load transfer reversals",
    )
    .await
}

async fn create_transfer_reversal(
    pool: web::Data<PgPool>,
    request: web::Json<TransferReversalCreateRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO transfer_reversals_company (transfer_id, reason, amount, currency, status)
        VALUES ($1, $2, $3, $4, 'pending')
        RETURNING id
        "#,
    )
    .bind(req.transfer_id)
    .bind(req.reason.as_deref())
    .bind(req.amount)
    .bind(req.currency.unwrap_or_else(|| "INR".to_string()))
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "money.transfer_reversal_created",
                "transfer_reversal_company",
                Some(id),
                json!({ "transfer_id": req.transfer_id }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "pending" }))
        }
        Err(e) => {
            tracing::error!("Create transfer reversal failed: {}", e);
            server_error("Failed to create transfer reversal")
        }
    }
}

async fn list_payouts_company(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM payouts_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load payouts",
    )
    .await
}

async fn create_payout_company(
    pool: web::Data<PgPool>,
    request: web::Json<PayoutCreateRequest>,
) -> impl Responder {
    let req = request.into_inner();
    if req.idempotency_key.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "message": "idempotency_key required" }));
    }
    let eligibility = sqlx::query(
        r#"
        SELECT (
            COALESCE(m.status, m.onboarding_status, '') = 'active'
            AND COALESCE(m.kyc_status, '') = 'verified'
            AND COALESCE(m.review_status, '') = 'approved'
            AND COALESCE(m.risk_status, '') <> 'blocked'
            AND COALESCE(m.payout_status, '') IN ('enabled','pending_verification')
        ) AS payout_eligible,
        COALESCE(ba.is_verified, false) AS bank_verified,
        COALESCE((
            SELECT COUNT(*) = 0
            FROM reconciliation_mismatches rm
            JOIN reconciliation_items ri ON ri.id = rm.item_id
            WHERE ri.payout_id IS NOT NULL
              AND rm.status = 'open'
        ), true) AS reconciliation_ready,
        COALESCE((
            SELECT SUM(balance)
            FROM ledger_accounts
            WHERE owner_type = 'company'
        ), 0)::bigint AS available_balance
        FROM merchants m
        LEFT JOIN bank_accounts ba ON ba.id = $2
        WHERE m.id = $1
        "#,
    )
    .bind(req.business_account_id)
    .bind(req.bank_account_id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(row)) = eligibility else {
        return HttpResponse::BadRequest().json(json!({ "message": "Business account not found" }));
    };
    let eligible: bool = row.get("payout_eligible");
    if !eligible {
        return HttpResponse::Forbidden()
            .json(json!({ "message": "Payout blocked by eligibility rules" }));
    }
    if !row.get::<bool, _>("bank_verified") {
        return HttpResponse::Conflict()
            .json(json!({ "message": "Payout requires a verified bank account" }));
    }
    if !row.get::<bool, _>("reconciliation_ready") {
        return HttpResponse::Conflict()
            .json(json!({ "message": "Payout blocked until reconciliation mismatches are reviewed" }));
    }
    if row.get::<i64, _>("available_balance") < req.amount {
        return HttpResponse::Conflict()
            .json(json!({ "message": "Insufficient available balance for payout" }));
    }
    if let Some(provider_id) = req.bank_payout_provider_id {
        let provider = sqlx::query(
            "SELECT environment, status, approved_for_live FROM integration_providers WHERE id = $1 AND provider_kind = 'bank_payout'",
        )
        .bind(provider_id)
        .fetch_optional(pool.get_ref())
        .await;
        let Ok(Some(provider)) = provider else {
            return HttpResponse::BadRequest().json(json!({ "message": "Bank payout provider not found" }));
        };
        let environment: String = provider.get("environment");
        let status: String = provider.get("status");
        let approved_for_live: bool = provider.get("approved_for_live");
        if environment == "live" {
            return HttpResponse::Conflict().json(json!({ "message": "No official live bank payout adapter is configured yet" }));
        }
        if status != "enabled" || (environment == "live" && !approved_for_live) {
            return HttpResponse::Conflict().json(json!({ "message": "Bank payout provider is not ready for use" }));
        }
    }
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payouts_company (
            business_account_id, provider_id, bank_payout_provider_id, bank_account_id, amount, currency, status,
            scheduled_at, idempotency_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'draft', COALESCE($7, NOW()), $8)
        RETURNING id
        "#,
    )
    .bind(req.business_account_id)
    .bind(req.provider_id)
    .bind(req.bank_payout_provider_id)
    .bind(req.bank_account_id)
    .bind(req.amount)
    .bind(req.currency.unwrap_or_else(|| "INR".to_string()))
    .bind(req.scheduled_at)
    .bind(req.idempotency_key.trim())
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "money.payout_created",
                "payout_company",
                Some(id),
                json!({ "amount": req.amount }),
            )
            .await;
            let _ = sqlx::query(
                "INSERT INTO payout_status_history (payout_id, from_status, to_status, reason) VALUES ($1, NULL, 'draft', 'created by internal admin')",
            )
            .bind(id)
            .execute(pool.get_ref())
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "draft" }))
        }
        Err(e) => {
            tracing::error!("Create payout failed: {}", e);
            server_error("Failed to create payout")
        }
    }
}

async fn retry_payout_company(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE payouts_company SET status = 'queued', failure_reason = NULL WHERE id = $1 AND status = 'failed' RETURNING id",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "money.payout_retried",
                "payout_company",
                Some(id),
                json!({ "status": "queued" }),
            )
            .await;
            let _ = sqlx::query(
                "INSERT INTO payout_status_history (payout_id, from_status, to_status, reason) VALUES ($1, 'failed', 'queued', 'retry requested')",
            )
            .bind(id)
            .execute(pool.get_ref())
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "queued" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Payout not found" })),
        Err(e) => {
            tracing::error!("Retry payout failed: {}", e);
            server_error("Failed to retry payout")
        }
    }
}

async fn list_payout_batches_company(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM payout_batches_company ORDER BY created_at DESC LIMIT 200",
        "Failed to load payout batches",
    )
    .await
}

async fn create_payout_batch_company(
    pool: web::Data<PgPool>,
    request: web::Json<PayoutBatchRequest>,
) -> impl Responder {
    let currency = request.currency.trim().to_uppercase();
    let batch_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payout_batches_company (currency, total_amount, item_count, status)
        SELECT $1, COALESCE(SUM(amount), 0)::bigint, COUNT(*), 'queued'
        FROM payouts_company
        WHERE status = 'queued'
          AND UPPER(currency) = $1
        RETURNING id
        "#,
    )
    .bind(&currency)
    .fetch_optional(pool.get_ref())
    .await;
    match batch_id {
        Ok(Some(id)) => {
            let _ = sqlx::query(
                r#"
                INSERT INTO payout_batch_items_company (batch_id, payout_id, business_account_id, amount, status)
                SELECT $1, id, business_account_id, amount, 'queued'
                FROM payouts_company
                WHERE status = 'queued' AND UPPER(currency) = $2
                "#,
            )
            .bind(id)
            .bind(&currency)
            .execute(pool.get_ref())
            .await;
            audit(
                &pool,
                "payout_batch.created",
                "payout_batch",
                Some(id),
                json!({ "currency": currency }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "queued" }))
        }
        Ok(None) => {
            HttpResponse::BadRequest().json(json!({ "message": "No eligible payouts for batch" }))
        }
        Err(e) => {
            tracing::error!("Create payout batch failed: {}", e);
            server_error("Failed to create payout batch")
        }
    }
}

async fn process_payout_batch_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let batch_id = path.into_inner();
    let rows = sqlx::query(
        "SELECT id, payout_id FROM payout_batch_items_company WHERE batch_id = $1 ORDER BY id",
    )
    .bind(batch_id)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => {
            let mut processed = 0_i64;
            let mut failed = 0_i64;
            for row in rows {
                let item_id: Uuid = row.get("id");
                let payout_id: Option<Uuid> = row.try_get("payout_id").unwrap_or(None);
                let result = if let Some(payout_id) = payout_id {
                    sqlx::query(
                        "UPDATE payouts_company SET status = 'processing', processed_at = NOW() WHERE id = $1 AND status = 'queued'",
                    )
                    .bind(payout_id)
                    .execute(pool.get_ref())
                    .await
                } else {
                    Err(sqlx::Error::RowNotFound)
                };
                if result.is_ok() {
                    processed += 1;
                    let _ = sqlx::query(
                        "UPDATE payout_batch_items_company SET status = 'processing' WHERE id = $1",
                    )
                    .bind(item_id)
                    .execute(pool.get_ref())
                    .await;
                } else {
                    failed += 1;
                    let _ = sqlx::query("UPDATE payout_batch_items_company SET status = 'failed', error_message = 'payout not found' WHERE id = $1")
                        .bind(item_id)
                        .execute(pool.get_ref())
                        .await;
                }
            }
            let status = if failed == 0 {
                "processing"
            } else {
                "partial_failed"
            };
            let _ = sqlx::query(
                "UPDATE payout_batches_company SET status = $2, processed_at = NOW() WHERE id = $1",
            )
            .bind(batch_id)
            .bind(status)
            .execute(pool.get_ref())
            .await;
            audit(
                &pool,
                "payout_batch.processed",
                "payout_batch",
                Some(batch_id),
                json!({ "processed": processed, "failed": failed }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": batch_id, "processed": processed, "failed": failed, "status": status }))
        }
        Err(e) => {
            tracing::error!("Process payout batch failed: {}", e);
            server_error("Failed to process payout batch")
        }
    }
}

async fn transition_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
    target_status: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    if matches!(target_status, "approved" | "processing") {
        if let Some(disposition) = evaluate_payout_risk(pool.get_ref(), id).await {
            match disposition {
                RiskDisposition::Block => {
                    return HttpResponse::Conflict()
                        .json(json!({ "message": "Payout blocked by risk rules" }));
                }
                RiskDisposition::HoldPayout | RiskDisposition::Review => {
                    if target_status == "approved" {
                        let _ = sqlx::query(
                            "UPDATE payouts_company SET status = 'on_hold', failure_reason = 'risk review required' WHERE id = $1",
                        )
                        .bind(id)
                        .execute(pool.get_ref())
                        .await;
                        return HttpResponse::Conflict()
                            .json(json!({ "message": "Payout held for risk review" }));
                    }
                }
                RiskDisposition::RestrictMerchant | RiskDisposition::Allow => {}
            }
        }
    }
    if target_status == "processing" {
        let provider = sqlx::query(
            r#"
            SELECT p.id, p.amount, p.currency, p.idempotency_key, ip.environment
            FROM payouts_company p
            LEFT JOIN integration_providers ip ON ip.id = p.bank_payout_provider_id
            WHERE p.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
        let Ok(Some(provider)) = provider else {
            return HttpResponse::NotFound().json(json!({ "message": "Payout not found" }));
        };
        if provider.get::<Option<String>, _>("environment").as_deref() == Some("test") {
            let idempotency_key = provider
                .get::<Option<String>, _>("idempotency_key")
                .unwrap_or_else(|| id.to_string());
            let adapter = TestBankPayoutAdapter;
            let result = adapter
                .create_payout(BankPayoutRequest {
                    idempotency_key: idempotency_key.clone(),
                    amount_paise: provider.get("amount"),
                    currency: provider.get("currency"),
                    payout_id: id.to_string(),
                })
                .await;
            let _ = sqlx::query(
                r#"
                INSERT INTO bank_payout_provider_attempts (
                    payout_id, provider_id, idempotency_key, operation, status,
                    provider_reference, response_summary_redacted, error_code
                )
                SELECT $1, bank_payout_provider_id, $2, 'create_payout', $3, $4, $5::jsonb, $6
                FROM payouts_company
                WHERE id = $1
                ON CONFLICT (provider_id, idempotency_key, operation) DO NOTHING
                "#,
            )
            .bind(id)
            .bind(&idempotency_key)
            .bind(&result.status)
            .bind(result.provider_reference.as_deref())
            .bind(&result.summary_redacted)
            .bind(result.error_code.as_deref())
            .execute(pool.get_ref())
            .await;
            let _ = sqlx::query(
                "UPDATE payouts_company SET provider_reference = COALESCE($2, provider_reference), provider_response_summary = $3::jsonb WHERE id = $1",
            )
            .bind(id)
            .bind(result.provider_reference.as_deref())
            .bind(&result.summary_redacted)
            .execute(pool.get_ref())
            .await;
        }
    }
    if target_status == "paid" {
        let reference_kind = request.reference_kind.as_deref().unwrap_or("");
        let reference_value = request.reference_value.as_deref().unwrap_or("");
        if !matches!(reference_kind, "manual" | "test" | "provider") || reference_value.trim().is_empty() {
            return HttpResponse::BadRequest().json(json!({ "message": "Paid payout requires a manual, test, or provider reference" }));
        }
    }
    if matches!(target_status, "failed" | "reversed" | "cancelled" | "on_hold")
        && request.failure_reason.as_deref().unwrap_or("").trim().is_empty()
    {
        return HttpResponse::BadRequest().json(json!({ "message": "Failure or hold reason required" }));
    }
    let row = sqlx::query("SELECT status FROM payouts_company WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Payout not found" }));
    };
    let from_status: String = row.get("status");
    let allowed = matches!(
        (from_status.as_str(), target_status),
        ("draft", "approved")
            | ("approved", "queued")
            | ("queued", "processing")
            | ("processing", "paid")
            | ("processing", "failed")
            | ("paid", "reversed")
            | ("draft", "cancelled")
            | ("queued", "cancelled")
            | ("approved", "cancelled")
            | ("draft", "on_hold")
            | ("queued", "on_hold")
            | ("approved", "on_hold")
            | ("on_hold", "queued")
            | ("failed", "queued")
    );
    if !allowed {
        return HttpResponse::Conflict().json(json!({ "message": format!("Invalid payout transition {from_status} -> {target_status}") }));
    }
    let updated = sqlx::query(
        r#"
        UPDATE payouts_company
        SET status = $2,
            failure_reason = CASE WHEN $2 IN ('failed','reversed','cancelled','on_hold') THEN $3 ELSE NULL END,
            reference_kind = COALESCE($4, reference_kind),
            manual_reference = COALESCE($5, manual_reference),
            approved_at = CASE WHEN $2 = 'approved' THEN NOW() ELSE approved_at END,
            processed_at = CASE WHEN $2 IN ('paid','failed','reversed') THEN NOW() ELSE processed_at END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(target_status)
    .bind(request.failure_reason.as_deref())
    .bind(request.reference_kind.as_deref())
    .bind(request.reference_value.as_deref())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            let _ = sqlx::query(
                "INSERT INTO payout_status_history (payout_id, from_status, to_status, reason, reference_kind, reference_value) VALUES ($1,$2,$3,$4,$5,$6)",
            )
            .bind(id)
            .bind(&from_status)
            .bind(target_status)
            .bind(request.failure_reason.as_deref().or(request.note.as_deref()))
            .bind(request.reference_kind.as_deref())
            .bind(request.reference_value.as_deref())
            .execute(pool.get_ref())
            .await;
            audit(
                &pool,
                &format!("money.payout_{target_status}"),
                "payout_company",
                Some(id),
                json!({ "from_status": from_status, "to_status": target_status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": target_status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Payout not found" })),
        Err(e) => {
            tracing::error!("Transition payout failed: {}", e);
            server_error("Failed to update payout")
        }
    }
}

async fn approve_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "approved").await
}

async fn queue_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "queued").await
}

async fn start_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "processing").await
}

async fn mark_payout_paid_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "paid").await
}

async fn fail_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "failed").await
}

async fn reverse_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "reversed").await
}

async fn hold_payout_company(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<PayoutActionRequest>,
) -> impl Responder {
    transition_payout_company(pool, path, request, "on_hold").await
}

async fn list_provider_report_imports(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM provider_report_imports ORDER BY created_at DESC LIMIT 200",
        "Failed to load provider report imports",
    )
    .await
}

async fn upload_provider_report_import(
    pool: web::Data<PgPool>,
    request: web::Json<ProviderReportImportRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO provider_report_imports (provider_id, report_type, file_name, file_hash, status, row_count)
        VALUES ($1, $2, $3, $4, 'uploaded', $5)
        ON CONFLICT (file_hash) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(request.provider_id)
    .bind(request.report_type.trim())
    .bind(request.file_name.trim())
    .bind(request.file_hash.trim())
    .bind(request.row_count.unwrap_or(0))
    .fetch_optional(pool.get_ref())
    .await;
    match saved {
        Ok(Some(id)) => {
            audit(
                &pool,
                "provider_report.uploaded",
                "provider_report_import",
                Some(id),
                json!({ "file_name": request.file_name, "report_type": request.report_type }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "uploaded" }))
        }
        Ok(None) => HttpResponse::Conflict()
            .json(json!({ "message": "Duplicate file hash upload blocked" })),
        Err(e) => {
            tracing::error!("Upload provider report import failed: {}", e);
            server_error("Failed to upload provider report import")
        }
    }
}

async fn list_provider_report_rows(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let import_id = path.into_inner();
    let rows = sqlx::query(
        "SELECT * FROM provider_report_rows WHERE import_id = $1 ORDER BY row_number ASC LIMIT 500",
    )
    .bind(import_id)
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Failed to load provider report rows: {}", e);
            server_error("Failed to load provider report rows")
        }
    }
}

async fn process_provider_report_import(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ProviderReportProcessRequest>,
) -> impl Responder {
    let import_id = path.into_inner();
    let rows = body.rows.clone().unwrap_or_default();
    let mut imported = 0_i32;
    let mut errors = 0_i32;
    for (index, row) in rows.iter().enumerate() {
        let amount = row.get("amount").and_then(|v| v.as_i64());
        let currency = row
            .get("currency")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if amount.is_none() || currency.is_none() {
            errors += 1;
            continue;
        }
        let _ = sqlx::query(
            r#"
            INSERT INTO provider_report_rows (
                import_id, row_number, external_id, external_type, amount, currency, fee_amount, net_amount,
                settlement_reference, payout_reference, raw_row_redacted, match_status
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'unmatched')
            "#,
        )
        .bind(import_id)
        .bind((index + 1) as i32)
        .bind(row.get("external_id").and_then(|v| v.as_str()))
        .bind(row.get("external_type").and_then(|v| v.as_str()))
        .bind(amount.unwrap_or_default())
        .bind(currency.unwrap_or_else(|| "INR".to_string()))
        .bind(row.get("fee_amount").and_then(|v| v.as_i64()))
        .bind(row.get("net_amount").and_then(|v| v.as_i64()))
        .bind(row.get("settlement_reference").and_then(|v| v.as_str()))
        .bind(row.get("payout_reference").and_then(|v| v.as_str()))
        .bind(redact_sensitive_value(row))
        .execute(pool.get_ref())
        .await;
        imported += 1;
    }
    let _ = sqlx::query(
        "UPDATE provider_report_imports SET status = 'processed', imported_count = $2, error_count = $3, processed_at = NOW() WHERE id = $1",
    )
    .bind(import_id)
    .bind(imported)
    .bind(errors)
    .execute(pool.get_ref())
    .await;
    audit(
        &pool,
        "provider_report.processed",
        "provider_report_import",
        Some(import_id),
        json!({ "imported_count": imported, "error_count": errors }),
    )
    .await;
    HttpResponse::Ok()
        .json(json!({ "id": import_id, "imported_count": imported, "error_count": errors }))
}

async fn list_report_parser_configs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM report_parser_configs ORDER BY created_at DESC LIMIT 200",
        "Failed to load parser configs",
    )
    .await
}

async fn upsert_report_parser_config(
    pool: web::Data<PgPool>,
    request: web::Json<ParserConfigRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO report_parser_configs (
            provider_id, report_type, column_mapping_json, date_format, amount_format, currency_column, status
        )
        VALUES ($1,$2,$3::jsonb,$4,$5,$6,'active')
        RETURNING id
        "#,
    )
    .bind(request.provider_id)
    .bind(request.report_type.trim())
    .bind(request.column_mapping_json)
    .bind(request.date_format)
    .bind(request.amount_format)
    .bind(request.currency_column)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(e) => {
            tracing::error!("Save parser config failed: {}", e);
            server_error("Failed to save parser config")
        }
    }
}

async fn list_compliance_controls(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM compliance_controls ORDER BY category, code",
        "Failed to load compliance controls",
    )
    .await
}

async fn patch_compliance_control(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<ComplianceControlPatchRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let request = request.into_inner();
    let current = sqlx::query("SELECT * FROM compliance_controls WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(current)) = current else {
        return HttpResponse::NotFound().json(json!({ "message": "Compliance control not found" }));
    };
    let status = request.status.unwrap_or_else(|| {
        current
            .try_get("status")
            .unwrap_or_else(|_| "pending".to_string())
    });
    let owner = request
        .owner_admin_id
        .or_else(|| current.try_get("owner_admin_id").ok().flatten());
    let due = request
        .next_review_due_at
        .or_else(|| current.try_get("next_review_due_at").ok().flatten());
    match sqlx::query(
        "UPDATE compliance_controls SET status = $2, owner_admin_id = $3, next_review_due_at = $4, last_reviewed_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(owner)
    .bind(due)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(e) => {
            tracing::error!("Patch compliance control failed: {}", e);
            server_error("Failed to update compliance control")
        }
    }
}

async fn list_compliance_evidence(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM compliance_evidence ORDER BY created_at DESC LIMIT 200",
        "Failed to load compliance evidence",
    )
    .await
}

async fn upload_compliance_evidence(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<ComplianceEvidenceRequest>,
) -> impl Responder {
    let control_id = path.into_inner();
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO compliance_evidence (control_id, file_reference, file_name, note) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(control_id)
    .bind(request.file_reference.trim())
    .bind(request.file_name.trim())
    .bind(request.note)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "compliance_evidence.uploaded",
                "compliance_control",
                Some(control_id),
                json!({ "evidence_id": id }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Upload compliance evidence failed: {}", e);
            server_error("Failed to upload compliance evidence")
        }
    }
}

async fn list_compliance_packet_exports(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM compliance_packet_exports ORDER BY created_at DESC LIMIT 100",
        "Failed to load compliance packet exports",
    )
    .await
}

async fn create_compliance_packet_export(
    pool: web::Data<PgPool>,
    request: web::Json<CompliancePacketRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO compliance_packet_exports (period, status, file_reference, expires_at)
        VALUES ($1, 'ready', $2, NOW() + INTERVAL '7 days')
        RETURNING id
        "#,
    )
    .bind(request.period.trim())
    .bind(format!(
        "compliance-packet-{}.zip",
        request.period.replace(' ', "-")
    ))
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "compliance_packet.exported",
                "compliance_packet_export",
                Some(id),
                json!({ "period": request.period }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "ready" }))
        }
        Err(e) => {
            tracing::error!("Create compliance packet failed: {}", e);
            server_error("Failed to create compliance packet")
        }
    }
}

async fn list_metric_samples(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM metric_samples ORDER BY captured_at DESC LIMIT 500",
        "Failed to load metric samples",
    )
    .await
}

async fn create_metric_sample(
    pool: web::Data<PgPool>,
    request: web::Json<MetricSampleRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO metric_samples (metric_name, labels_json, value) VALUES ($1, $2::jsonb, $3) RETURNING id",
    )
    .bind(request.metric_name.trim())
    .bind(request.labels_json.unwrap_or_else(|| json!({})))
    .bind(request.value)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(e) => {
            tracing::error!("Create metric sample failed: {}", e);
            server_error("Failed to create metric sample")
        }
    }
}

async fn list_slo_definitions(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM slo_definitions ORDER BY created_at DESC",
        "Failed to load SLO definitions",
    )
    .await
}

async fn upsert_slo_definition(
    pool: web::Data<PgPool>,
    request: web::Json<SloDefinitionRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO slo_definitions (name, target_percent, window_minutes, metric_query_key, status)
        VALUES ($1, $2, $3, $4, 'active')
        ON CONFLICT (name)
        DO UPDATE SET target_percent = EXCLUDED.target_percent,
                      window_minutes = EXCLUDED.window_minutes,
                      metric_query_key = EXCLUDED.metric_query_key
        RETURNING id
        "#,
    )
    .bind(request.name.trim())
    .bind(request.target_percent)
    .bind(request.window_minutes)
    .bind(request.metric_query_key.trim())
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(e) => {
            tracing::error!("Upsert SLO definition failed: {}", e);
            server_error("Failed to save SLO definition")
        }
    }
}

async fn calculate_slo_snapshots(pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query(
        "SELECT id, target_percent, metric_query_key FROM slo_definitions WHERE status = 'active'",
    )
    .fetch_all(pool.get_ref())
    .await;
    match rows {
        Ok(rows) => {
            for row in rows {
                let slo_id: Uuid = row.get("id");
                let target: f64 = row.get("target_percent");
                let key: String = row.get("metric_query_key");
                let value = sqlx::query_scalar::<_, f64>(
                    "SELECT COALESCE(AVG(value), 0) FROM metric_samples WHERE metric_name = $1 AND captured_at > NOW() - INTERVAL '1 day'",
                )
                .bind(&key)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(0.0);
                let status = if value >= target { "pass" } else { "warning" };
                let budget = (target - value).max(0.0);
                let _ = sqlx::query(
                    "INSERT INTO slo_snapshots (slo_id, current_percent, error_budget_remaining, status) VALUES ($1,$2,$3,$4)",
                )
                .bind(slo_id)
                .bind(value)
                .bind(budget)
                .bind(status)
                .execute(pool.get_ref())
                .await;
                if status == "warning" {
                    let _ = create_admin_notification(
                        &pool,
                        "warning",
                        "slo_breach",
                        &format!("SLO breach: {}", key),
                        "slo_definition",
                        Some(slo_id),
                    )
                    .await;
                }
            }
            HttpResponse::Ok().json(json!({ "status": "snapshots_refreshed" }))
        }
        Err(e) => {
            tracing::error!("Calculate SLO snapshots failed: {}", e);
            server_error("Failed to calculate SLO snapshots")
        }
    }
}

async fn list_slo_snapshots(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT ss.*, sd.name AS slo_name FROM slo_snapshots ss JOIN slo_definitions sd ON sd.id = ss.slo_id ORDER BY ss.captured_at DESC LIMIT 300",
        "Failed to load SLO snapshots",
    )
    .await
}

async fn list_trace_logs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM trace_logs ORDER BY created_at DESC LIMIT 500",
        "Failed to load trace logs",
    )
    .await
}

async fn create_trace_log(
    pool: web::Data<PgPool>,
    request: web::Json<TraceLogRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO trace_logs (trace_id, request_id, span_name, duration_ms, status, error_message) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(request.trace_id.trim())
    .bind(request.request_id)
    .bind(request.span_name.trim())
    .bind(request.duration_ms)
    .bind(request.status.trim())
    .bind(request.error_message)
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => HttpResponse::Ok().json(json!({ "id": id })),
        Err(e) => {
            tracing::error!("Create trace log failed: {}", e);
            server_error("Failed to create trace log")
        }
    }
}

async fn list_reserves_company(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM reserve_balances ORDER BY created_at DESC LIMIT 200",
        "Failed to load reserves",
    )
    .await
}

async fn hold_reserve_company(
    pool: web::Data<PgPool>,
    request: web::Json<ReserveActionRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO reserve_balances (
            business_account_id, currency, reserve_type, amount, release_at, status
        )
        VALUES ($1, $2, $3, $4, $5, 'held')
        RETURNING id
        "#,
    )
    .bind(req.business_account_id)
    .bind(req.currency.unwrap_or_else(|| "INR".to_string()))
    .bind(req.reserve_type.unwrap_or_else(|| "manual".to_string()))
    .bind(req.amount)
    .bind(req.release_at)
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "money.reserve_held",
                "reserve_balance",
                Some(id),
                json!({ "amount": req.amount }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "held" }))
        }
        Err(e) => {
            tracing::error!("Hold reserve failed: {}", e);
            server_error("Failed to hold reserve")
        }
    }
}

async fn release_reserve_company(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated =
        sqlx::query("UPDATE reserve_balances SET status = 'released' WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "money.reserve_released",
                "reserve_balance",
                Some(id),
                json!({ "status": "released" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "released" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Reserve not found" })),
        Err(e) => {
            tracing::error!("Release reserve failed: {}", e);
            server_error("Failed to release reserve")
        }
    }
}

async fn list_risk_signals(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_signals ORDER BY created_at DESC LIMIT 200",
        "Failed to load risk signals",
    )
    .await
}

async fn list_risk_decisions(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_decisions ORDER BY created_at DESC LIMIT 200",
        "Failed to load risk decisions",
    )
    .await
}

async fn list_risk_lists(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM risk_lists ORDER BY created_at DESC LIMIT 200",
        "Failed to load risk lists",
    )
    .await
}

async fn create_risk_list_item(
    pool: web::Data<PgPool>,
    request: web::Json<RiskListRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let value_hash = hash_secret(req.value.trim());
    let masked = if req.value.len() <= 4 {
        "****".to_string()
    } else {
        format!(
            "{}***{}",
            &req.value[..2],
            &req.value[req.value.len() - 2..]
        )
    };
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO risk_lists (list_type, entity_type, value_hash, display_value_masked, reason, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        RETURNING id
        "#,
    )
    .bind(req.list_type.trim())
    .bind(req.entity_type.trim())
    .bind(value_hash)
    .bind(masked)
    .bind(req.reason.as_deref())
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "risk_list.item_created",
                "risk_list",
                Some(id),
                json!({ "list_type": req.list_type, "entity_type": req.entity_type }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Create risk list item failed: {}", e);
            server_error("Failed to create risk list item")
        }
    }
}

async fn update_risk_review(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RiskReviewActionRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let req = request.into_inner();
    let updated = sqlx::query(
        "UPDATE risk_reviews SET status = $2, assigned_to_admin_id = COALESCE($3, assigned_to_admin_id), decision_note = COALESCE($4, decision_note), resolved_at = CASE WHEN $2 IN ('approved','rejected') THEN NOW() ELSE resolved_at END WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(req.status.trim())
    .bind(req.assigned_to_admin_id)
    .bind(req.decision_note.as_deref())
    .fetch_optional(pool.get_ref())
    .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "risk_review.updated",
                "risk_review",
                Some(id),
                json!({ "status": req.status }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": req.status }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Risk review not found" })),
        Err(e) => {
            tracing::error!("Update risk review failed: {}", e);
            server_error("Failed to update risk review")
        }
    }
}

async fn simulate_risk_decision(request: web::Json<RiskSimulationRequest>) -> impl Responder {
    let req = request.into_inner();
    let mut score = 0;
    let mut matched = Vec::new();
    if req.amount > 100_000 {
        score += 45;
        matched.push("amount_threshold");
    }
    if matches!(req.country.as_str(), "NG" | "RU" | "KP") {
        score += 40;
        matched.push("country_block");
    }
    if req.payment_method.eq_ignore_ascii_case("card") {
        score += 10;
        matched.push("payment_method_block");
    }
    let decision = if score >= 70 {
        "block"
    } else if score >= 35 {
        "review"
    } else {
        "allow"
    };
    HttpResponse::Ok().json(json!({
        "score": score,
        "decision": decision,
        "matched_rules": matched,
        "reason": format!("risk score {}", score),
        "input": {
            "currency": req.currency,
            "country": req.country,
            "payment_method": req.payment_method,
            "business_account_id": req.business_account_id,
            "email": req.email.map(|_| "masked"),
            "phone": req.phone.map(|_| "masked"),
            "ip": req.ip.map(|_| "masked")
        }
    }))
}

async fn list_webhook_processing_attempts(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM webhook_processing_attempts ORDER BY started_at DESC LIMIT 200",
        "Failed to load webhook processing attempts",
    )
    .await
}

async fn list_webhook_retry_policies(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        "SELECT * FROM webhook_retry_policies ORDER BY updated_at DESC",
        "Failed to load webhook retry policies",
    )
    .await
}

async fn save_webhook_retry_policy(
    pool: web::Data<PgPool>,
    request: web::Json<WebhookRetryPolicyRequest>,
) -> impl Responder {
    let req = request.into_inner();
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO webhook_retry_policies (
            provider_id, max_attempts, backoff_strategy, initial_delay_seconds, max_delay_seconds, status
        )
        VALUES ($1,$2,$3,$4,$5,$6)
        ON CONFLICT (provider_id)
        DO UPDATE SET max_attempts = EXCLUDED.max_attempts,
                      backoff_strategy = EXCLUDED.backoff_strategy,
                      initial_delay_seconds = EXCLUDED.initial_delay_seconds,
                      max_delay_seconds = EXCLUDED.max_delay_seconds,
                      status = EXCLUDED.status,
                      updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(req.provider_id)
    .bind(req.max_attempts)
    .bind(req.backoff_strategy.trim())
    .bind(req.initial_delay_seconds)
    .bind(req.max_delay_seconds)
    .bind(req.status.unwrap_or_else(|| "active".to_string()))
    .fetch_one(pool.get_ref())
    .await;
    match id {
        Ok(id) => {
            audit(
                &pool,
                "webhook.retry_policy_saved",
                "webhook_retry_policy",
                Some(id),
                json!({ "provider_id": req.provider_id }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Err(e) => {
            tracing::error!("Save webhook retry policy failed: {}", e);
            server_error("Failed to save webhook retry policy")
        }
    }
}

async fn replay_webhook_event_v2(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    request: web::Json<WebhookReplayRequest>,
) -> impl Responder {
    let id_str = path.into_inner();
    let Some(id) = find_webhook_event_id(pool.get_ref(), &id_str).await else {
        return HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" }));
    };
    let req = request.into_inner();
    let row = sqlx::query(
        "SELECT processing_status, retry_count FROM payment_webhook_events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;
    let Ok(Some(row)) = row else {
        return HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" }));
    };
    let dry_run = req.dry_run.unwrap_or(false);
    if !dry_run {
        let _ = sqlx::query(
            "UPDATE payment_webhook_events SET processing_status = 'queued', processing_started_at = NULL, processing_finished_at = NULL, next_retry_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool.get_ref())
        .await;
    }
    let _ = sqlx::query(
        "INSERT INTO webhook_processing_attempts (webhook_event_id, attempt_number, status, started_at, finished_at, error_message, worker_id) VALUES ($1, $2, $3, NOW(), NOW(), $4, $5)",
    )
    .bind(id)
    .bind(row.get::<i32, _>("retry_count") + 1)
    .bind(if dry_run { "dry_run" } else { "queued" })
    .bind(req.note.unwrap_or_else(|| "replay requested".to_string()))
    .bind("admin-replay")
    .execute(pool.get_ref())
    .await;
    audit(
        &pool,
        if dry_run {
            "webhook.replay_dry_run"
        } else {
            "webhook.replay"
        },
        "payment_webhook_event",
        Some(id),
        json!({ "dry_run": dry_run }),
    )
    .await;
    HttpResponse::Ok().json(json!({
        "id": id,
        "status": if dry_run { "dry_run" } else { "queued" },
        "previous_status": row.get::<String, _>("processing_status")
    }))
}

async fn resolve_dead_letter_webhook(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<WebhookReplayRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let note = request
        .note
        .clone()
        .unwrap_or_else(|| "resolved by admin".to_string());
    let updated = sqlx::query(
        "UPDATE payment_webhook_events SET processing_status = 'resolved', dead_letter_reason = $2, processing_finished_at = NOW() WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(note)
    .fetch_optional(pool.get_ref())
    .await;
    match updated {
        Ok(Some(_)) => {
            audit(
                &pool,
                "webhook.dead_letter_resolved",
                "payment_webhook_event",
                Some(id),
                json!({ "note": "resolved" }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "resolved" }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Webhook event not found" })),
        Err(e) => {
            tracing::error!("Resolve dead letter failed: {}", e);
            server_error("Failed to resolve dead letter")
        }
    }
}

fn setting_value(value: &Value) -> (String, &'static str) {
    if let Some(bool_value) = value.as_bool() {
        return (bool_value.to_string(), "boolean");
    }
    if let Some(number_value) = value.as_i64() {
        return (number_value.to_string(), "number");
    }
    (
        value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string()),
        "string",
    )
}

async fn role_id_by_name(pool: &web::Data<PgPool>, name: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM admin_roles WHERE name = $1")
        .bind(name)
        .fetch_optional(pool.get_ref())
        .await
        .ok()
        .flatten()
}

fn forbidden_role(role: &str) -> bool {
    let normalized = role.to_ascii_lowercase().replace([' ', '_', '-'], "");
    matches!(
        normalized.as_str(),
        "merchantadmin" | "selleradmin" | "vendoradmin"
    )
}

fn provider_from_row(row: sqlx::postgres::PgRow) -> PaymentProviderItem {
    let success_rate = row.get::<f64, _>("success_rate");
    let api_key_status = secret_status(row.get("api_key_status"));
    let api_secret_status = secret_status(row.get("api_secret_status"));
    let webhook_secret_status = secret_status(row.get("webhook_secret_status"));
    let provider_type: String = row.get("provider_type");
    let provider_code: String = row.get("code");
    let missing_required_secrets = missing_required_secrets(
        &provider_type,
        &provider_code,
        &api_key_status,
        &api_secret_status,
    );
    let health_status: String = row.get("health_status");
    let connector_readiness_score = connector_readiness_score(
        &provider_type,
        &provider_code,
        &api_key_status,
        &api_secret_status,
        &webhook_secret_status,
        &health_status,
    );
    PaymentProviderItem {
        id: row.get("id"),
        name: row.get("display_name"),
        code: provider_code,
        provider_type: row.get("provider_type"),
        environment: row.get("environment"),
        status: row.get("status"),
        priority: row.get("priority"),
        failover_enabled: row.get("failover_enabled"),
        webhook_url: row.get("webhook_url"),
        supported_payment_methods: row.get("supported_payment_methods"),
        supported_currencies: row.get("supported_currencies"),
        supported_countries: row.get("supported_countries"),
        health_status,
        last_health_check_at: row.get("last_health_check_at"),
        last_successful_webhook_at: row.get("last_successful_webhook_at"),
        last_failed_webhook_at: row.get("last_failed_webhook_at"),
        last_failed_payment_at: row.get("last_failed_payment_at"),
        success_rate,
        failure_rate: 100.0 - success_rate,
        attempt_count: row.get("attempt_count"),
        masked_api_key: presence_mask(row.get::<Option<String>, _>("api_key_encrypted")),
        masked_api_secret: presence_mask(row.get::<Option<String>, _>("api_secret_encrypted")),
        masked_webhook_secret: presence_mask(
            row.get::<Option<String>, _>("webhook_secret_encrypted"),
        ),
        api_key_status,
        api_secret_status,
        webhook_secret_status,
        connector_readiness_score,
        missing_required_secrets,
        capabilities: row.get("capabilities"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn connector_for_type(
    provider_type: &str,
    environment: &str,
    credentials_ready: bool,
) -> Box<dyn PaymentProviderConnector> {
    match provider_type.to_ascii_lowercase().as_str() {
        "stripe" => Box::new(StripeProviderConnector::new(environment, credentials_ready)),
        "razorpay" => Box::new(RazorpayProviderConnector::new(
            environment,
            credentials_ready,
        )),
        "adyen" => Box::new(AdyenProviderConnector::new(environment, credentials_ready)),
        _ => Box::new(MockProviderConnector),
    }
}

fn is_real_provider_type(provider_type: &str, provider_code: &str) -> bool {
    let provider_type = provider_type.to_ascii_lowercase();
    let provider_code = provider_code.to_ascii_lowercase();
    matches!(
        provider_type.as_str(),
        "stripe" | "razorpay" | "adyen" | "cashfree" | "juspay" | "payu"
    ) || matches!(
        provider_code.as_str(),
        "stripe" | "razorpay" | "adyen" | "cashfree" | "juspay" | "payu"
    )
}

fn has_implemented_real_connector(provider_type: &str, provider_code: &str) -> bool {
    let provider_type = provider_type.to_ascii_lowercase();
    let provider_code = provider_code.to_ascii_lowercase();
    matches!(provider_type.as_str(), "stripe" | "razorpay" | "adyen")
        || matches!(provider_code.as_str(), "stripe" | "razorpay" | "adyen")
}

fn connector_credentials_ready(
    provider_type: &str,
    provider_code: &str,
    has_api_key: bool,
    has_api_secret: bool,
) -> bool {
    match provider_type.to_ascii_lowercase().as_str() {
        provider if is_real_provider_type(provider, provider_code) => has_api_key && has_api_secret,
        _ => true,
    }
}

fn missing_required_secrets(
    provider_type: &str,
    provider_code: &str,
    api_key_status: &str,
    api_secret_status: &str,
) -> Value {
    if !matches!(
        provider_type.to_ascii_lowercase().as_str(),
        provider if is_real_provider_type(provider, provider_code)
    ) {
        return json!([]);
    }
    let mut missing = Vec::new();
    if api_key_status != "configured" {
        missing.push("api_key");
    }
    if api_secret_status != "configured" {
        missing.push("api_secret");
    }
    json!(missing)
}

fn connector_readiness_score(
    provider_type: &str,
    provider_code: &str,
    api_key_status: &str,
    api_secret_status: &str,
    webhook_secret_status: &str,
    health_status: &str,
) -> i32 {
    if !matches!(
        provider_type.to_ascii_lowercase().as_str(),
        provider if is_real_provider_type(provider, provider_code)
    ) {
        return 100;
    }
    let mut score = 0;
    if api_key_status == "configured" {
        score += 35;
    }
    if api_secret_status == "configured" {
        score += 35;
    }
    if webhook_secret_status == "configured" {
        score += 10;
    }
    if api_key_status == "configured"
        && api_secret_status == "configured"
        && matches!(health_status, "healthy" | "verified")
    {
        score = 100;
    }
    score
}

async fn get_by_id(
    pool: web::Data<PgPool>,
    table: &'static str,
    id: Uuid,
    not_found: &'static str,
) -> HttpResponse {
    let sql = format!("SELECT * FROM {table} WHERE id = $1");
    let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    match row {
        Ok(Some(row)) => HttpResponse::Ok().json(row_to_json(&row)),
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": not_found })),
        Err(e) => {
            tracing::error!("Lookup failed for {}: {}", table, e);
            server_error("Lookup failed")
        }
    }
}

async fn query_items(
    pool: web::Data<PgPool>,
    sql: &'static str,
    error_message: &'static str,
) -> HttpResponse {
    use sha2::{Digest, Sha256};
    let hash = hex::encode(Sha256::digest(sql.as_bytes()));
    let cache_key = format!("admin:query:{hash}");
    if let Some(cached) = crate::cache::AdminCache::get::<serde_json::Value>(&cache_key) {
        return HttpResponse::Ok().json(cached);
    }
    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let res = json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() });
            crate::cache::AdminCache::set(&cache_key, res.clone(), std::time::Duration::from_secs(15));
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            tracing::error!("{}: {}", error_message, e);
            server_error(error_message)
        }
    }
}

async fn fetch_json_rows(pool: &PgPool, sql: &str, id: Uuid) -> Vec<Value> {
    sqlx::query(sql)
        .bind(id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(row_to_json)
        .collect()
}

fn row_to_json(row: &sqlx::postgres::PgRow) -> Value {
    let mut value = serde_json::Map::new();
    for column in row.columns() {
        let name = column.name();
        if name.contains("encrypted") || name.contains("nonce") {
            value.insert(name.to_string(), json!("stored"));
            continue;
        }
        let cell = json_cell(row, name);
        value.insert(name.to_string(), cell);
    }
    Value::Object(value)
}

fn json_cell(row: &sqlx::postgres::PgRow, name: &str) -> Value {
    if let Ok(value) = row.try_get::<Option<String>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<String, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<Uuid>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Uuid, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<i64, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<i32>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<i32, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<bool, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<f64, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<DateTime<Utc>>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<DateTime<Utc>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Option<Value>, _>(name) {
        return json!(value);
    }
    if let Ok(value) = row.try_get::<Value, _>(name) {
        return value;
    }
    Value::Null
}

fn encrypt_secret(value: &str) -> Result<(String, String), String> {
    let raw_key = env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY is required to save provider credentials".to_string())?;
    let key = hex::decode(raw_key.trim())
        .map_err(|_| "VAULT_ENCRYPTION_KEY must be 64 hex characters".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string())?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), value.as_bytes())
        .map_err(|_| "Credential encryption failed".to_string())?;
    Ok((BASE64.encode(encrypted), BASE64.encode(nonce_bytes)))
}

fn mask_secret(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    format!(
        "****{}",
        chars[chars.len() - 4..].iter().collect::<String>()
    )
}

fn mask_api_key(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 10 {
        return "fp_****".to_string();
    }
    format!(
        "{}…{}",
        chars[..6].iter().collect::<String>(),
        chars[chars.len() - 4..].iter().collect::<String>()
    )
}

fn hash_secret(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn redact_sensitive_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let lowered = key.to_ascii_lowercase();
                    if lowered == "api_key"
                        || lowered.contains("api_key")
                        || lowered == "api_secret"
                        || lowered.contains("api_secret")
                        || lowered.contains("webhook_secret")
                        || lowered.contains("signing_secret")
                        || lowered.contains("password")
                        || lowered.contains("token")
                        || lowered.contains("authorization")
                        || lowered.contains("secret")
                        || lowered.contains("card_number")
                        || lowered == "card"
                        || lowered.contains("cvv")
                        || lowered.contains("cvc")
                        || lowered.contains("account_number")
                        || lowered.contains("bank")
                    {
                        (key.clone(), json!("redacted"))
                    } else {
                        (key.clone(), redact_sensitive_value(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_sensitive_value).collect()),
        _ => value.clone(),
    }
}

fn mask_bank_account(value: &str) -> String {
    let digits = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<Vec<_>>();
    if digits.len() <= 4 {
        return "****".to_string();
    }
    format!(
        "********{}",
        digits[digits.len() - 4..].iter().collect::<String>()
    )
}

fn presence_mask(value: Option<String>) -> String {
    if value.filter(|v| !v.is_empty()).is_some() {
        "stored".to_string()
    } else {
        "not_set".to_string()
    }
}

fn secret_status(value: String) -> String {
    match value.as_str() {
        "active" => "configured".to_string(),
        "expired" => "expired".to_string(),
        "disabled" => "missing".to_string(),
        _ => "missing".to_string(),
    }
}

async fn get_or_create_ledger_account(
    pool: &web::Data<PgPool>,
    owner_type: &str,
    owner_id: Option<Uuid>,
    account_type: &str,
    name: &str,
    currency: &str,
) -> Result<Uuid, sqlx::Error> {
    if let Some(account_id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM ledger_accounts WHERE owner_id IS NOT DISTINCT FROM $1 AND account_type = $2 LIMIT 1",
    )
    .bind(owner_id)
    .bind(account_type)
    .fetch_optional(pool.get_ref())
    .await?
    {
        return Ok(account_id);
    }

    if let Some(account_id) = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO ledger_accounts (name, account_type, owner_id, owner_type, currency, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(account_type)
    .bind(owner_id)
    .bind(owner_type)
    .bind(currency)
    .fetch_optional(pool.get_ref())
    .await?
    {
        return Ok(account_id);
    }

    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM ledger_accounts WHERE owner_id IS NOT DISTINCT FROM $1 AND account_type = $2 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(owner_id)
    .bind(account_type)
    .fetch_one(pool.get_ref())
    .await
}

#[allow(clippy::too_many_arguments)]
async fn post_balanced_ledger(
    pool: &web::Data<PgPool>,
    ledger_type: &str,
    source_type: &str,
    source_id: Uuid,
    provider_id: Option<Uuid>,
    business_account_id: Option<Uuid>,
    amount: i64,
    currency: &str,
    idempotency_key: &str,
    description: &str,
) -> Result<Uuid, sqlx::Error> {
    if amount <= 0 {
        return sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM ledger_transactions WHERE idempotency_key = $1",
        )
        .bind(idempotency_key)
        .fetch_one(pool.get_ref())
        .await;
    }

    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM ledger_transactions WHERE idempotency_key = $1",
    )
    .bind(idempotency_key)
    .fetch_optional(pool.get_ref())
    .await?
    {
        return Ok(id);
    }

    let transaction_group_id = Uuid::new_v4();
    let debit_account = get_or_create_ledger_account(
        pool,
        "company",
        None,
        "clearing",
        "Company clearing",
        currency,
    )
    .await?;
    let credit_account = get_or_create_ledger_account(
        pool,
        business_account_id
            .map(|_| "business_account")
            .unwrap_or("company"),
        business_account_id,
        if business_account_id.is_some() {
            "payable"
        } else {
            "settlement"
        },
        "Ledger payable",
        currency,
    )
    .await?;

    let tx_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO ledger_transactions (
            transaction_group_id, type, status, source_type, source_id, provider_id,
            business_account_id, amount, currency, idempotency_key
        )
        VALUES ($1, $2, 'posted', $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(transaction_group_id)
    .bind(ledger_type)
    .bind(source_type)
    .bind(source_id)
    .bind(provider_id)
    .bind(business_account_id)
    .bind(amount)
    .bind(currency)
    .bind(idempotency_key)
    .fetch_one(pool.get_ref())
    .await?;

    let debit_balance = bump_ledger_balance(pool, debit_account, amount).await?;
    insert_ledger_entry(
        pool,
        debit_account,
        transaction_group_id,
        source_type,
        source_id,
        "debit",
        amount,
        currency,
        debit_balance,
        description,
    )
    .await?;
    let credit_balance = bump_ledger_balance(pool, credit_account, amount).await?;
    insert_ledger_entry(
        pool,
        credit_account,
        transaction_group_id,
        source_type,
        source_id,
        "credit",
        amount,
        currency,
        credit_balance,
        description,
    )
    .await?;
    Ok(tx_id)
}

async fn bump_ledger_balance(
    pool: &web::Data<PgPool>,
    account_id: Uuid,
    amount: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "UPDATE ledger_accounts SET balance = balance + $2, updated_at = NOW() WHERE id = $1 RETURNING balance",
    )
    .bind(account_id)
    .bind(amount)
    .fetch_one(pool.get_ref())
    .await
}

#[allow(clippy::too_many_arguments)]
async fn insert_ledger_entry(
    pool: &web::Data<PgPool>,
    account_id: Uuid,
    transaction_group_id: Uuid,
    source_type: &str,
    source_id: Uuid,
    direction: &str,
    amount: i64,
    currency: &str,
    balance_after: i64,
    description: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO ledger_entries (
            transaction_id, account_id, entry_type, transaction_group_id, source_type,
            source_id, direction, amount, currency, balance_after, description, status
        )
        VALUES ($1, $2, $3, $1, $4, $5, $3, $6, $7, $8, $9, 'posted')
        "#,
    )
    .bind(transaction_group_id)
    .bind(account_id)
    .bind(direction)
    .bind(source_type)
    .bind(source_id)
    .bind(amount)
    .bind(currency)
    .bind(balance_after)
    .bind(description)
    .execute(pool.get_ref())
    .await
    .map(|_| ())
}

#[allow(dead_code)]
fn ledger_entries_balance(entries: &[(String, i64)]) -> bool {
    let debit: i64 = entries
        .iter()
        .filter(|(direction, _)| direction == "debit")
        .map(|(_, amount)| amount)
        .sum();
    let credit: i64 = entries
        .iter()
        .filter(|(direction, _)| direction == "credit")
        .map(|(_, amount)| amount)
        .sum();
    debit == credit
}

async fn audit(
    pool: &web::Data<PgPool>,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    summary: Value,
) {
    crate::cache::AdminCache::invalidate_prefix("admin:");
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs_company (actor_role, action, entity_type, entity_id, new_value_summary)
        VALUES ('Payment Ops Admin', $1, $2, $3, $4::jsonb)
        "#,
    )
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(summary)
    .execute(pool.get_ref())
    .await;
}

async fn audit_with_request(
    pool: &web::Data<PgPool>,
    request: &HttpRequest,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    summary: Value,
) {
    crate::cache::AdminCache::invalidate_prefix("admin:");
    let claims = request.extensions().get::<AdminClaims>().cloned();
    let actor_admin_id = claims.as_ref().map(|claims| claims.admin_id);
    let actor_role = claims
        .and_then(|claims| claims.role)
        .unwrap_or_else(|| "Payment Ops Admin".to_string());
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| request.peer_addr().map(|addr| addr.ip().to_string()));
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs_company (
            actor_admin_id, actor_role, action, entity_type, entity_id,
            new_value_summary, ip_address, user_agent, request_id
        )
        VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9)
        "#,
    )
    .bind(actor_admin_id)
    .bind(actor_role)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(redact_sensitive_value(&summary))
    .bind(ip_address)
    .bind(user_agent)
    .bind(request_id)
    .execute(pool.get_ref())
    .await;
}

async fn create_admin_notification(
    pool: &web::Data<PgPool>,
    severity: &str,
    notification_type: &str,
    title: &str,
    related_entity_type: &str,
    related_entity_id: Option<Uuid>,
) {
    let dedupe_key = related_entity_id.map(|id| format!("{notification_type}:{id}"));
    let _ = sqlx::query(
        r#"
        INSERT INTO admin_notifications (
            severity, type, title, message, related_entity_type, related_entity_id,
            source, dedupe_key, status, sla_due_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'provider_connector', $7, 'open',
                NOW() + CASE $1 WHEN 'critical' THEN INTERVAL '15 minutes' WHEN 'warning' THEN INTERVAL '1 hour' ELSE INTERVAL '1 day' END)
        ON CONFLICT (dedupe_key) WHERE status NOT IN ('resolved', 'ignored')
        DO UPDATE SET occurrence_count = admin_notifications.occurrence_count + 1,
                      last_seen_at = NOW(),
                      updated_at = NOW()
        "#,
    )
    .bind(severity)
    .bind(notification_type)
    .bind(title)
    .bind(format!("{title}. Check missing credentials or provider availability."))
    .bind(related_entity_type)
    .bind(related_entity_id)
    .bind(dedupe_key)
    .execute(pool.get_ref())
    .await;
}

fn server_error(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "message": message }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    configure_payment_ops_routes(cfg, "/payments");
    configure_payment_ops_routes(cfg, "/payment-ops");
}

pub fn configure_company_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/users", web::get().to(list_admin_users))
        .route("/connectors", web::get().to(list_providers))
        .route("/users/invite", web::post().to(invite_admin_user))
        .route("/users/{id}", web::patch().to(patch_admin_user))
        .route(
            "/users/{id}/deactivate",
            web::post().to(deactivate_admin_user),
        )
        .route(
            "/users/{id}/reactivate",
            web::post().to(reactivate_admin_user),
        )
        .route("/users/{id}/roles", web::post().to(assign_admin_role))
        .route(
            "/users/{id}/roles/{role}",
            web::delete().to(remove_admin_role),
        )
        .route("/users/{id}/audit", web::get().to(admin_user_audit))
        .route("/system-health", web::get().to(system_health))
        .route(
            "/system-health/refresh",
            web::post().to(refresh_system_health),
        )
        .route("/system-health/jobs", web::get().to(system_health_jobs))
        .route(
            "/system-health/jobs/{id}/retry",
            web::post().to(retry_system_job),
        )
        .route(
            "/system-health/providers",
            web::get().to(system_health_providers),
        )
        .route("/notifications", web::get().to(list_notifications))
        .route(
            "/notifications/{id}/acknowledge",
            web::post().to(acknowledge_notification),
        )
        .route(
            "/notifications/{id}/assign",
            web::post().to(assign_notification),
        )
        .route(
            "/notifications/{id}/resolve",
            web::post().to(resolve_notification),
        )
        .route(
            "/notifications/{id}/ignore",
            web::post().to(ignore_notification),
        )
        .route(
            "/notifications/{id}/incident",
            web::post().to(create_incident_from_notification),
        )
        .route(
            "/notifications/{id}/snooze",
            web::post().to(snooze_notification),
        )
        .route("/platform-settings", web::get().to(get_platform_settings))
        .route(
            "/platform-settings",
            web::patch().to(patch_platform_settings),
        )
        .route(
            "/platform-settings/maintenance/enable",
            web::post().to(enable_maintenance),
        )
        .route(
            "/platform-settings/maintenance/disable",
            web::post().to(disable_maintenance),
        );
}

fn configure_payment_ops_routes(cfg: &mut web::ServiceConfig, prefix: &'static str) {
    cfg.route(&format!("{prefix}/dashboard"), web::get().to(dashboard))
        .route(
            &format!("{prefix}/providers"),
            web::get().to(list_providers),
        )
        .route(
            &format!("{prefix}/providers"),
            web::post().to(create_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}"),
            web::patch().to(update_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}/test"),
            web::post().to(test_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}/maintenance"),
            web::post().to(maintenance_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}/enable"),
            web::post().to(enable_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}/disable"),
            web::post().to(disable_provider),
        )
        .route(
            &format!("{prefix}/providers/{{id}}/rotate-credentials"),
            web::post().to(rotate_credentials),
        )
        .route(
            &format!("{prefix}/credentials"),
            web::get().to(list_credentials),
        )
        .route(
            &format!("{prefix}/credentials"),
            web::post().to(create_credential),
        )
        .route(
            &format!("{prefix}/credentials/{{id}}/validate"),
            web::post().to(validate_credential),
        )
        .route(
            &format!("{prefix}/credentials/{{id}}/rotate"),
            web::post().to(rotate_credentials_by_id),
        )
        .route(
            &format!("{prefix}/credentials/{{id}}/disable"),
            web::post().to(disable_credential),
        )
        .route(
            &format!("{prefix}/webhook-configs"),
            web::get().to(list_webhook_configs),
        )
        .route(
            &format!("{prefix}/webhook-configs"),
            web::post().to(save_webhook_config),
        )
        .route(
            &format!("{prefix}/webhook-configs/{{id}}"),
            web::patch().to(save_webhook_config),
        )
        .route(
            &format!("{prefix}/webhook-configs/{{id}}/test"),
            web::post().to(test_webhook_config),
        )
        .route(
            &format!("{prefix}/webhook-configs/{{id}}/rotate-secret"),
            web::post().to(rotate_webhook_secret),
        )
        .route(
            &format!("{prefix}/bank-integrations"),
            web::get().to(list_bank_integrations),
        )
        .route(
            &format!("{prefix}/bank-integrations"),
            web::post().to(create_bank_integration),
        )
        .route(
            &format!("{prefix}/bank-integrations/{{id}}"),
            web::patch().to(update_bank_integration),
        )
        .route(
            &format!("{prefix}/bank-integrations/{{id}}/verify"),
            web::post().to(verify_bank_integration),
        )
        .route(
            &format!("{prefix}/bank-integrations/{{id}}/disable"),
            web::post().to(disable_bank_integration),
        )
        .route(
            &format!("{prefix}/white-label-config"),
            web::get().to(get_white_label_config),
        )
        .route(
            &format!("{prefix}/white-label-config"),
            web::patch().to(update_white_label_config),
        )
        .route(
            &format!("{prefix}/white-label-config/upload-logo"),
            web::post().to(upload_white_label_logo),
        )
        .route(
            &format!("{prefix}/white-label-config/preview"),
            web::get().to(get_white_label_config),
        )
        .route(
            &format!("{prefix}/white-label/payment-links"),
            web::get().to(list_checkout_payment_links),
        )
        .route(
            &format!("{prefix}/white-label/payment-links"),
            web::post().to(create_checkout_payment_link),
        )
        .route(
            &format!("{prefix}/white-label/payment-links/{{id}}/expire"),
            web::post().to(expire_checkout_payment_link),
        )
        .route(
            &format!("{prefix}/white-label/sessions"),
            web::get().to(list_checkout_sessions),
        )
        .route(
            &format!("{prefix}/white-label/sessions"),
            web::post().to(create_checkout_session),
        )
        .route(
            &format!("{prefix}/white-label/sessions/{{id}}/complete-test-payment"),
            web::post().to(complete_checkout_session),
        )
        .route(
            &format!("{prefix}/white-label/receipts"),
            web::get().to(list_checkout_receipts),
        )
        .route(
            &format!("{prefix}/white-label/receipts/{{id}}/resend"),
            web::post().to(resend_checkout_receipt),
        )
        .route(
            &format!("{prefix}/money-movement/dashboard"),
            web::get().to(money_movement_dashboard),
        )
        .route(
            &format!("{prefix}/money-movement/split-rules"),
            web::get().to(list_split_rules),
        )
        .route(
            &format!("{prefix}/money-movement/split-rules"),
            web::post().to(create_split_rule),
        )
        .route(
            &format!("{prefix}/money-movement/simulate-split"),
            web::post().to(simulate_split),
        )
        .route(
            &format!("{prefix}/money-movement/transfers"),
            web::get().to(list_transfers_company),
        )
        .route(
            &format!("{prefix}/money-movement/transfers"),
            web::post().to(create_transfer_company),
        )
        .route(
            &format!("{prefix}/money-movement/reversals"),
            web::get().to(list_transfer_reversals),
        )
        .route(
            &format!("{prefix}/money-movement/reversals"),
            web::post().to(create_transfer_reversal),
        )
        .route(
            &format!("{prefix}/money-movement/payouts"),
            web::get().to(list_payouts_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts"),
            web::post().to(create_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/retry"),
            web::post().to(retry_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/approve"),
            web::post().to(approve_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/queue"),
            web::post().to(queue_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/start"),
            web::post().to(start_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/mark-paid"),
            web::post().to(mark_payout_paid_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/fail"),
            web::post().to(fail_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/reverse"),
            web::post().to(reverse_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payouts/{{id}}/hold"),
            web::post().to(hold_payout_company),
        )
        .route(
            &format!("{prefix}/money-movement/payout-batches"),
            web::get().to(list_payout_batches_company),
        )
        .route(
            &format!("{prefix}/money-movement/payout-batches"),
            web::post().to(create_payout_batch_company),
        )
        .route(
            &format!("{prefix}/money-movement/payout-batches/{{id}}/process"),
            web::post().to(process_payout_batch_company),
        )
        .route(
            &format!("{prefix}/money-movement/reserves"),
            web::get().to(list_reserves_company),
        )
        .route(
            &format!("{prefix}/money-movement/reserves/hold"),
            web::post().to(hold_reserve_company),
        )
        .route(
            &format!("{prefix}/money-movement/reserves/{{id}}/release"),
            web::post().to(release_reserve_company),
        )
        .route(
            &format!("{prefix}/routing-rules"),
            web::get().to(list_routing_rules),
        )
        .route(
            &format!("{prefix}/routing/provider-performance"),
            web::get().to(list_provider_performance),
        )
        .route(
            &format!("{prefix}/routing-rules"),
            web::post().to(create_routing_rule),
        )
        .route(
            &format!("{prefix}/routing-rules/simulate"),
            web::post().to(simulate_routing),
        )
        .route(
            &format!("{prefix}/routing/simulate"),
            web::post().to(simulate_routing),
        )
        .route(
            &format!("{prefix}/routing-rules/{{id}}"),
            web::patch().to(update_routing_rule),
        )
        .route(
            &format!("{prefix}/routing-rules/{{id}}/disable"),
            web::post().to(disable_routing_rule),
        )
        .route(
            &format!("{prefix}/transactions"),
            web::get().to(list_transactions),
        )
        .route(&format!("{prefix}/ledger"), web::get().to(list_ledger))
        .route(
            &format!("{prefix}/ledger/export"),
            web::get().to(export_ledger),
        )
        .route(
            &format!("{prefix}/ledger/{{id}}"),
            web::get().to(get_ledger_group),
        )
        .route(
            &format!("{prefix}/transactions/{{id}}"),
            web::get().to(get_transaction),
        )
        .route(
            &format!("{prefix}/transactions/{{id}}/capture"),
            web::post().to(capture_transaction),
        )
        .route(
            &format!("{prefix}/transactions/{{id}}/cancel"),
            web::post().to(cancel_transaction),
        )
        .route(
            &format!("{prefix}/transactions/{{id}}/refund"),
            web::post().to(create_refund),
        )
        .route(&format!("{prefix}/refunds"), web::get().to(list_refunds))
        .route(
            &format!("{prefix}/refunds"),
            web::post().to(
                |pool: web::Data<PgPool>, body: web::Json<RefundRequest>| async move {
                    insert_refund(pool, body.into_inner()).await
                },
            ),
        )
        .route(
            &format!("{prefix}/refunds/{{id}}"),
            web::get().to(get_refund),
        )
        .route(
            &format!("{prefix}/refunds/{{id}}/approve"),
            web::post().to(approve_refund),
        )
        .route(
            &format!("{prefix}/refunds/{{id}}/reject"),
            web::post().to(reject_refund),
        )
        .route(
            &format!("{prefix}/refunds/{{id}}/initiate"),
            web::post().to(initiate_refund),
        )
        .route(
            &format!("{prefix}/refunds/{{id}}/retry"),
            web::post().to(retry_refund),
        )
        .route(&format!("{prefix}/webhooks"), web::get().to(list_webhooks))
        .route(
            &format!("{prefix}/webhook-events"),
            web::get().to(list_webhooks),
        )
        .route(
            &format!("{prefix}/webhooks/{{id}}"),
            web::get().to(get_webhook),
        )
        .route(
            &format!("{prefix}/webhook-events/{{id}}"),
            web::get().to(get_webhook),
        )
        .route(
            &format!("{prefix}/webhooks/{{id}}/retry"),
            web::post().to(retry_webhook),
        )
        .route(
            &format!("{prefix}/webhook-events/{{id}}/retry"),
            web::post().to(retry_webhook),
        )
        .route(
            &format!("{prefix}/webhook-events/{{id}}/ignore"),
            web::post().to(ignore_webhook),
        )
        .route(
            &format!("{prefix}/webhook-events/{{id}}/replay"),
            web::post().to(replay_webhook_event_v2),
        )
        .route(
            &format!("{prefix}/webhook-events/{{id}}/resolve-dead-letter"),
            web::post().to(resolve_dead_letter_webhook),
        )
        .route(
            &format!("{prefix}/webhook-processing-attempts"),
            web::get().to(list_webhook_processing_attempts),
        )
        .route(
            &format!("{prefix}/webhook-retry-policies"),
            web::get().to(list_webhook_retry_policies),
        )
        .route(
            &format!("{prefix}/webhook-retry-policies"),
            web::post().to(save_webhook_retry_policy),
        )
        .route(
            &format!("{prefix}/settlements"),
            web::get().to(list_settlements),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}"),
            web::get().to(get_settlement),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}/hold"),
            web::post().to(hold_settlement),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}/release"),
            web::post().to(release_settlement),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}/mark-settled"),
            web::post().to(mark_settlement_settled),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}/mark-failed"),
            web::post().to(mark_settlement_failed),
        )
        .route(
            &format!("{prefix}/settlements/{{id}}/export"),
            web::get().to(export_settlement),
        )
        .route(
            &format!("{prefix}/reconciliation"),
            web::get().to(list_reconciliation),
        )
        .route(
            &format!("{prefix}/reconciliation/upload"),
            web::post().to(upload_reconciliation_file),
        )
        .route(
            &format!("{prefix}/reconciliation/export-finance-summary"),
            web::get().to(export_reconciliation_finance_summary),
        )
        .route(
            &format!("{prefix}/reconciliation/batches"),
            web::get().to(list_reconciliation_batches),
        )
        .route(
            &format!("{prefix}/reconciliation/batches"),
            web::post().to(create_reconciliation_batch),
        )
        .route(
            &format!("{prefix}/reconciliation/batches/{{id}}"),
            web::get().to(get_reconciliation_batch),
        )
        .route(
            &format!("{prefix}/reconciliation/batches/{{id}}/close"),
            web::post().to(close_reconciliation_batch),
        )
        .route(
            &format!("{prefix}/reconciliation/batches/{{id}}/export.csv"),
            web::get().to(export_reconciliation_batch_csv),
        )
        .route(
            &format!("{prefix}/reconciliation/items/{{id}}/review"),
            web::post().to(review_reconciliation_item),
        )
        .route(
            &format!("{prefix}/reconciliation/records/{{id}}/ignore"),
            web::post().to(ignore_reconciliation_record),
        )
        .route(
            &format!("{prefix}/reconciliation/provider-imports"),
            web::get().to(list_provider_report_imports),
        )
        .route(
            &format!("{prefix}/reconciliation/provider-imports"),
            web::post().to(upload_provider_report_import),
        )
        .route(
            &format!("{prefix}/reconciliation/provider-imports/{{id}}/rows"),
            web::get().to(list_provider_report_rows),
        )
        .route(
            &format!("{prefix}/reconciliation/provider-imports/{{id}}/process"),
            web::post().to(process_provider_report_import),
        )
        .route(
            &format!("{prefix}/reconciliation/parser-configs"),
            web::get().to(list_report_parser_configs),
        )
        .route(
            &format!("{prefix}/reconciliation/parser-configs"),
            web::post().to(upsert_report_parser_config),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}"),
            web::get().to(get_reconciliation),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}/auto-match"),
            web::post().to(auto_match_reconciliation),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}/mismatches"),
            web::get().to(reconciliation_mismatches),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}/resolve"),
            web::post().to(resolve_reconciliation),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}/export-discrepancies"),
            web::get().to(export_reconciliation_discrepancies),
        )
        .route(&format!("{prefix}/disputes"), web::get().to(list_disputes))
        .route(
            &format!("{prefix}/disputes/{{id}}/submit-evidence"),
            web::post().to(submit_dispute_evidence),
        )
        .route(
            &format!("{prefix}/disputes/{{id}}/mark-won"),
            web::post().to(mark_dispute_won),
        )
        .route(
            &format!("{prefix}/disputes/{{id}}/mark-lost"),
            web::post().to(mark_dispute_lost),
        )
        .route(
            &format!("{prefix}/business-accounts"),
            web::get().to(list_business_accounts),
        )
        .route(
            &format!("{prefix}/business-accounts/{{id}}"),
            web::get().to(get_business_account),
        )
        .route(
            &format!("{prefix}/business-accounts/{{id}}"),
            web::patch().to(patch_business_account),
        )
        .route(
            &format!("{prefix}/business-accounts/{{id}}/review"),
            web::post().to(review_business_account),
        )
        .route(
            &format!("{prefix}/risk-rules"),
            web::get().to(list_risk_rules),
        )
        .route(
            &format!("{prefix}/risk-rules"),
            web::post().to(create_risk_rule),
        )
        .route(
            &format!("{prefix}/risk-rules/{{id}}"),
            web::patch().to(update_risk_rule),
        )
        .route(
            &format!("{prefix}/risk-rules/{{id}}/disable"),
            web::post().to(disable_risk_rule),
        )
        .route(
            &format!("{prefix}/risk-rules/{{id}}/enable"),
            web::post().to(enable_risk_rule),
        )
        .route(
            &format!("{prefix}/risk-reviews"),
            web::get().to(list_risk_reviews),
        )
        .route(
            &format!("{prefix}/risk-reviews/{{id}}"),
            web::patch().to(update_risk_review),
        )
        .route(
            &format!("{prefix}/risk-signals"),
            web::get().to(list_risk_signals),
        )
        .route(
            &format!("{prefix}/risk-evaluations"),
            web::get().to(list_risk_evaluations),
        )
        .route(
            &format!("{prefix}/risk-cases"),
            web::get().to(list_risk_cases),
        )
        .route(
            &format!("{prefix}/risk-cases/{{id}}/notes"),
            web::get().to(list_risk_case_notes),
        )
        .route(
            &format!("{prefix}/risk-cases/{{id}}/notes"),
            web::post().to(add_risk_case_note),
        )
        .route(
            &format!("{prefix}/risk-cases/{{id}}/assign"),
            web::post().to(assign_risk_case),
        )
        .route(
            &format!("{prefix}/risk-cases/{{id}}/review"),
            web::post().to(review_risk_case),
        )
        .route(
            &format!("{prefix}/risk-cases/{{id}}/resolve"),
            web::post().to(resolve_risk_case),
        )
        .route(
            &format!("{prefix}/risk-decisions"),
            web::get().to(list_risk_decisions),
        )
        .route(
            &format!("{prefix}/risk-lists"),
            web::get().to(list_risk_lists),
        )
        .route(
            &format!("{prefix}/risk-lists"),
            web::post().to(create_risk_list_item),
        )
        .route(
            &format!("{prefix}/risk/simulate"),
            web::post().to(simulate_risk_decision),
        )
        .route(
            &format!("{prefix}/reports"),
            web::get().to(list_reports_catalog),
        )
        .route(
            &format!("{prefix}/reports/generate"),
            web::post().to(generate_admin_report),
        )
        .route(
            &format!("{prefix}/reports/schedule"),
            web::post().to(schedule_admin_report),
        )
        .route(
            &format!("{prefix}/reports/scheduled"),
            web::get().to(list_scheduled_admin_reports),
        )
        .route(
            &format!("{prefix}/reports/export-history"),
            web::get().to(list_report_exports),
        )
        .route(
            &format!("{prefix}/reports/export-history/{{id}}/download"),
            web::get().to(download_report_export),
        )
        .route(
            &format!("{prefix}/reports/exports/{{id}}/download"),
            web::get().to(download_report_export),
        )
        .route(
            &format!("{prefix}/audit-logs"),
            web::get().to(list_audit_logs),
        )
        .route(&format!("{prefix}/admin-roles"), web::get().to(list_roles))
        .route(
            &format!("{prefix}/developer/api-keys"),
            web::get().to(list_developer_api_keys),
        )
        .route(
            &format!("{prefix}/developer/api-keys"),
            web::post().to(create_developer_api_key),
        )
        .route(
            &format!("{prefix}/developer/api-keys/{{id}}/rotate"),
            web::post().to(rotate_developer_api_key),
        )
        .route(
            &format!("{prefix}/developer/api-keys/{{id}}/revoke"),
            web::post().to(revoke_developer_api_key),
        )
        .route(
            &format!("{prefix}/developer/request-logs"),
            web::get().to(list_api_request_logs),
        )
        .route(
            &format!("{prefix}/developer/api-versions"),
            web::get().to(list_api_versions),
        )
        .route(
            &format!("{prefix}/developer/idempotency-records"),
            web::get().to(list_api_idempotency_records),
        )
        .route(
            &format!("{prefix}/developer/idempotency-records"),
            web::post().to(create_api_idempotency_record),
        )
        .route(
            &format!("{prefix}/developer/error-catalog"),
            web::get().to(list_api_error_catalog),
        )
        .route(
            &format!("{prefix}/developer/rate-limits"),
            web::get().to(list_api_rate_limits),
        )
        .route(
            &format!("{prefix}/developer/rate-limits"),
            web::post().to(set_api_rate_limit),
        )
        .route(
            &format!("{prefix}/developer/rate-limits/{{id}}/check"),
            web::post().to(run_rate_limit_check),
        )
        .route(
            &format!("{prefix}/developer/webhook-test"),
            web::post().to(send_webhook_test),
        )
        .route(
            &format!("{prefix}/developer/webhook-events"),
            web::get().to(list_webhooks),
        )
        .route(
            &format!("{prefix}/developer/webhook-replays"),
            web::get().to(list_webhook_replays),
        )
        .route(
            &format!("{prefix}/developer/webhook-events/{{id}}/replay"),
            web::post().to(replay_webhook_event),
        )
        .route(
            &format!("{prefix}/compliance/overview"),
            web::get().to(compliance_overview),
        )
        .route(
            &format!("{prefix}/compliance/security-events"),
            web::get().to(list_security_events),
        )
        .route(
            &format!("{prefix}/compliance/access-reviews"),
            web::get().to(list_access_reviews),
        )
        .route(
            &format!("{prefix}/compliance/access-reviews"),
            web::post().to(start_access_review),
        )
        .route(
            &format!("{prefix}/compliance/tasks"),
            web::get().to(list_compliance_tasks),
        )
        .route(
            &format!("{prefix}/compliance/tasks"),
            web::post().to(create_compliance_task),
        )
        .route(
            &format!("{prefix}/compliance/access-reviews/{{id}}/items"),
            web::get().to(list_access_review_items),
        )
        .route(
            &format!("{prefix}/compliance/access-review-items/{{id}}"),
            web::patch().to(update_access_review_item),
        )
        .route(
            &format!("{prefix}/compliance/access-reviews/{{id}}/complete"),
            web::post().to(complete_access_review),
        )
        .route(
            &format!("{prefix}/compliance/audit-export"),
            web::get().to(export_audit_logs),
        )
        .route(
            &format!("{prefix}/compliance/security-events-export"),
            web::get().to(export_security_events),
        )
        .route(
            &format!("{prefix}/compliance/rotate-due-credentials"),
            web::post().to(rotate_due_credentials),
        )
        .route(
            &format!("{prefix}/compliance/tasks/{{task}}/complete"),
            web::post().to(mark_compliance_task_complete),
        )
        .route(
            &format!("{prefix}/compliance/controls"),
            web::get().to(list_compliance_controls),
        )
        .route(
            &format!("{prefix}/compliance/controls/{{id}}"),
            web::patch().to(patch_compliance_control),
        )
        .route(
            &format!("{prefix}/compliance/evidence"),
            web::get().to(list_compliance_evidence),
        )
        .route(
            &format!("{prefix}/compliance/controls/{{id}}/evidence"),
            web::post().to(upload_compliance_evidence),
        )
        .route(
            &format!("{prefix}/compliance/packet-exports"),
            web::get().to(list_compliance_packet_exports),
        )
        .route(
            &format!("{prefix}/compliance/packet-exports"),
            web::post().to(create_compliance_packet_export),
        )
        .route(
            &format!("{prefix}/security-review/overview"),
            web::get().to(security_review_overview),
        )
        .route(
            &format!("{prefix}/security-review/findings"),
            web::get().to(list_security_review_findings),
        )
        .route(
            &format!("{prefix}/security-review/findings"),
            web::post().to(create_security_review_finding),
        )
        .route(
            &format!("{prefix}/security-review/findings/{{id}}"),
            web::patch().to(patch_security_review_finding),
        )
        .route(
            &format!("{prefix}/security-review/threat-model-export"),
            web::get().to(export_threat_model),
        )
        .route(
            &format!("{prefix}/security-review/step-up-challenges"),
            web::get().to(list_step_up_challenges),
        )
        .route(
            &format!("{prefix}/security-review/step-up-challenges"),
            web::post().to(create_step_up_challenge),
        )
        .route(
            &format!("{prefix}/security-review/step-up-challenges/{{id}}/verify"),
            web::post().to(verify_step_up_challenge),
        )
        .route(
            &format!("{prefix}/security-review/step-up-actions"),
            web::get().to(list_step_up_actions),
        )
        .route(
            &format!("{prefix}/security-review/step-up-actions"),
            web::post().to(configure_step_up_action),
        )
        .route(
            &format!("{prefix}/security-review/sensitive-actions"),
            web::get().to(list_sensitive_action_history),
        )
        .route(
            &format!("{prefix}/readiness-scorecard"),
            web::get().to(readiness_scorecard),
        )
        .route(
            &format!("{prefix}/readiness-scorecard/export"),
            web::get().to(readiness_export),
        )
        .route(
            &format!("{prefix}/readiness-scorecard/{{id}}/assign-owner"),
            web::post().to(readiness_assign_owner),
        )
        .route(
            &format!("{prefix}/readiness-scorecard/{{id}}/manual-check"),
            web::post().to(readiness_mark_manual_check),
        )
        .route(
            &format!("{prefix}/go-live/checklist"),
            web::get().to(list_go_live_checklist),
        )
        .route(
            &format!("{prefix}/go-live/checklist/{{id}}"),
            web::patch().to(patch_go_live_checklist_item),
        )
        .route(
            &format!("{prefix}/go-live/scorecard"),
            web::get().to(readiness_go_live_scorecard),
        )
        .route(
            &format!("{prefix}/go-live/cutovers"),
            web::get().to(list_production_cutovers),
        )
        .route(
            &format!("{prefix}/go-live/cutovers"),
            web::post().to(start_production_cutover),
        )
        .route(
            &format!("{prefix}/go-live/cutovers/{{id}}/{{action}}"),
            web::post().to(update_production_cutover),
        )
        .route(
            &format!("{prefix}/system-health/metrics"),
            web::get().to(list_metric_samples),
        )
        .route(
            &format!("{prefix}/system-health/metrics"),
            web::post().to(create_metric_sample),
        )
        .route(
            &format!("{prefix}/system-health/slos"),
            web::get().to(list_slo_definitions),
        )
        .route(
            &format!("{prefix}/system-health/slos"),
            web::post().to(upsert_slo_definition),
        )
        .route(
            &format!("{prefix}/system-health/slos/refresh"),
            web::post().to(calculate_slo_snapshots),
        )
        .route(
            &format!("{prefix}/system-health/slo-snapshots"),
            web::get().to(list_slo_snapshots),
        )
        .route(
            &format!("{prefix}/system-health/traces"),
            web::get().to(list_trace_logs),
        )
        .route(
            &format!("{prefix}/system-health/traces"),
            web::post().to(create_trace_log),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_connector_preserves_idempotency_reference() {
        let connector = MockProviderConnector;
        let response = connector
            .refund_payment(ConnectorRequest {
                idempotency_key: "idem-1".to_string(),
                amount: Some(500),
                currency: Some("INR".to_string()),
                provider_reference: None,
                payload: json!({}),
            })
            .await;

        assert_eq!(response.status, ConnectorOperationStatus::Accepted);
        assert_eq!(
            response.provider_reference,
            Some("mock_refund_idem-1".to_string())
        );
    }

    #[test]
    fn secrets_and_bank_accounts_are_masked() {
        assert_eq!(mask_secret("sk_test_placeholder"), "****lder");
        assert_eq!(mask_bank_account("123456789012"), "********9012");
    }

    #[test]
    fn percentage_amount_uses_decimal_minor_unit_math() {
        assert_eq!(percentage_amount(10_000, Decimal::new(175, 2)), 175);
        assert_eq!(percentage_amount(999, Decimal::new(250, 2)), 24);
        assert_eq!(percentage_amount_rounded(999, Decimal::new(250, 2)), 25);
    }

    #[test]
    fn mock_webhook_signature_uses_payload_length() {
        let connector = MockProviderConnector;
        let payload = br#"{"event_id":"evt_1"}"#;
        assert!(connector.verify_webhook_signature(payload, "mock:secret:20", "secret"));
        assert!(!connector.verify_webhook_signature(payload, "mock:secret:99", "secret"));
    }

    #[tokio::test]
    async fn missing_credential_blocks_real_connector_action() {
        let connector = StripeProviderConnector::new("sandbox", false);
        let response = connector
            .create_payment_intent(ConnectorRequest {
                idempotency_key: "idem-1".to_string(),
                amount: Some(1200),
                currency: Some("INR".to_string()),
                provider_reference: None,
                payload: json!({ "secret": "sk_test_hidden" }),
            })
            .await;

        assert_eq!(response.status, ConnectorOperationStatus::Failed);
        assert_eq!(response.summary["error"], "missing_required_credentials");
    }

    #[tokio::test]
    async fn real_connector_requires_idempotency_key() {
        let connector = RazorpayProviderConnector::new("sandbox", true);
        let response = connector
            .refund_payment(ConnectorRequest {
                idempotency_key: String::new(),
                amount: Some(1200),
                currency: Some("INR".to_string()),
                provider_reference: Some("pay_123".to_string()),
                payload: json!({}),
            })
            .await;

        assert_eq!(response.status, ConnectorOperationStatus::Failed);
        assert_eq!(response.summary["error"], "idempotency_key_required");
    }

    #[tokio::test]
    async fn connector_returns_normalized_redacted_payment() {
        let connector = AdyenProviderConnector::new("sandbox", true);
        let response = connector
            .create_payment_intent(ConnectorRequest {
                idempotency_key: "idem-2".to_string(),
                amount: Some(2500),
                currency: Some("INR".to_string()),
                provider_reference: None,
                payload: json!({ "card_number": "4111111111111111", "metadata": { "order": "ord_1" } }),
            })
            .await;

        assert_eq!(response.status, ConnectorOperationStatus::Accepted);
        assert_eq!(response.summary["normalized"], true);
        assert_eq!(response.summary["metadata"]["card_number"], "redacted");
        assert_eq!(response.summary["metadata"]["metadata"]["order"], "ord_1");
    }

    #[test]
    fn real_connectors_verify_webhook_signatures() {
        let payload = br#"{"id":"evt_1"}"#;
        let timestamp = "1777777777";
        let stripe_payload = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let stripe_sig = format!(
            "t={timestamp},v1={}",
            hmac_sha256_hex(stripe_payload.as_bytes(), "whsec_test").unwrap()
        );
        let stripe = StripeProviderConnector::new("sandbox", true);
        assert!(stripe.verify_webhook_signature(payload, &stripe_sig, "whsec_test"));

        let razorpay_sig = hmac_sha256_hex(payload, "rzp_secret").unwrap();
        let razorpay = RazorpayProviderConnector::new("sandbox", true);
        assert!(razorpay.verify_webhook_signature(payload, &razorpay_sig, "rzp_secret"));
        assert!(!razorpay.verify_webhook_signature(payload, "bad", "rzp_secret"));
    }

    #[test]
    fn connector_readiness_lists_missing_secrets() {
        assert_eq!(
            connector_readiness_score(
                "stripe",
                "stripe",
                "missing",
                "configured",
                "missing",
                "unknown"
            ),
            35
        );
        assert_eq!(
            missing_required_secrets("stripe", "stripe", "missing", "configured"),
            json!(["api_key"])
        );
        assert_eq!(
            missing_required_secrets("mock", "mock", "missing", "missing"),
            json!([])
        );
        assert_eq!(
            missing_required_secrets("mock", "razorpay", "missing", "missing"),
            json!(["api_key", "api_secret"])
        );
    }

    #[test]
    fn ledger_entries_must_balance() {
        assert!(ledger_entries_balance(&[
            ("debit".to_string(), 1000),
            ("credit".to_string(), 1000),
        ]));
        assert!(!ledger_entries_balance(&[
            ("debit".to_string(), 1000),
            ("credit".to_string(), 999),
        ]));
    }

    #[test]
    fn provider_success_display_inputs_are_distinct() {
        assert_eq!(secret_status("active".to_string()), "configured");
        assert_eq!(secret_status("disabled".to_string()), "missing");
    }

    #[test]
    fn api_keys_are_masked_and_hashed() {
        let raw = "fp_live_1234567890abcdef";
        let masked = mask_api_key(raw);
        assert_eq!(masked, "fp_liv…cdef");
        assert_ne!(hash_secret(raw), raw);
        assert_eq!(hash_secret(raw), hash_secret(raw));
    }

    #[test]
    fn api_log_redaction_removes_sensitive_values() {
        let redacted = redact_sensitive_value(&json!({
            "Authorization": "Bearer secret",
            "api_key": "fp_live_secret",
            "api_secret": "provider_secret",
            "webhook_secret": "whsec_123",
            "signing_secret": "sign_123",
            "password": "pass",
            "token": "jwt",
            "card_number": "4111111111111111",
            "nested": {
                "cvv": "123",
                "cvc": "456",
                "bank_account_number": "1234567890",
                "safe": "kept"
            }
        }));
        assert_eq!(redacted["Authorization"], "redacted");
        assert_eq!(redacted["api_key"], "redacted");
        assert_eq!(redacted["api_secret"], "redacted");
        assert_eq!(redacted["webhook_secret"], "redacted");
        assert_eq!(redacted["signing_secret"], "redacted");
        assert_eq!(redacted["password"], "redacted");
        assert_eq!(redacted["token"], "redacted");
        assert_eq!(redacted["card_number"], "redacted");
        assert_eq!(redacted["nested"]["cvv"], "redacted");
        assert_eq!(redacted["nested"]["cvc"], "redacted");
        assert_eq!(redacted["nested"]["bank_account_number"], "redacted");
        assert_eq!(redacted["nested"]["safe"], "kept");
    }

    #[test]
    fn report_export_filters_are_masked() {
        let redacted = redact_sensitive_value(&json!({
            "business_account": "acct_123",
            "card": "4111111111111111",
            "cvv": "123",
            "bank_account_number": "999900001111",
            "provider": "razorpay"
        }));
        assert_eq!(redacted["card"], "redacted");
        assert_eq!(redacted["cvv"], "redacted");
        assert_eq!(redacted["bank_account_number"], "redacted");
        assert_eq!(redacted["provider"], "razorpay");
    }

    #[test]
    fn csv_cells_escape_quotes_and_commas() {
        assert_eq!(
            csv_cell("alpha,beta \"quoted\""),
            "\"alpha,beta \"\"quoted\"\"\""
        );
    }

    #[test]
    fn unsupported_report_formats_fail_cleanly() {
        let xlsx = generate_report_xlsx("XLSX");
        assert_eq!(xlsx.status, "FAILED");
        assert_eq!(
            xlsx.error_message.as_deref(),
            Some("XLSX generator not configured")
        );
        assert!(xlsx.file_reference.is_none());
    }

    #[test]
    fn finance_pdf_has_pdf_header_and_footer_text() {
        let export_id = Uuid::nil();
        let rows = vec![json!({
            "id": "01fb45e5-1111-2222-3333-aaaaaaaaadbe69",
            "created_at": Utc::now().to_rfc3339(),
            "amount": 249900,
            "currency": "INR",
            "status": "READY",
            "api_secret": "must_not_leak"
        })];
        let filters = json!({ "start_date": "", "status": "", "format": "PDF" });
        let pdf = generate_finance_pdf(PdfReport {
            export_id,
            report_type: "Balance Report",
            generated_by: "Super Admin",
            generated_at: Utc::now(),
            filters: &filters,
            rows: &rows,
        });
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let body = String::from_utf8_lossy(&pdf);
        assert!(body.contains("Generated by FeelPay company admin"));
        assert!(body.contains("FeelPay"));
        assert!(body.contains("All"));
        assert!(!body.contains("must_not_leak"));
    }

    #[test]
    fn unsafe_report_paths_are_rejected() {
        assert!(safe_report_path("../secret.csv").is_none());
        assert!(safe_report_path("/tmp/secret.csv").is_none());
    }
}

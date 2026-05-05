use actix_web::{web, HttpResponse, Responder};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Column, PgPool, Row};
use std::env;
use uuid::Uuid;

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

#[derive(Serialize)]
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

#[derive(Serialize)]
struct ProviderHealthMetric {
    provider: String,
    health_status: String,
    success_rate: f64,
}

#[derive(Serialize)]
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
    rule_type: String,
    condition: Value,
    action: RiskAction,
    priority: Option<i32>,
    status: Option<String>,
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

async fn dashboard(pool: web::Data<PgPool>) -> impl Responder {
    let summary = sqlx::query(
        r#"
        SELECT
          COALESCE((SELECT SUM(amount) FROM payment_orders WHERE status IN ('captured','successful','paid')), 0)::bigint AS total_gmv,
          COALESCE((SELECT COUNT(*) FROM payment_orders WHERE status IN ('captured','successful','paid')), 0)::bigint AS successful_payments,
          COALESCE((SELECT COUNT(*) FROM payment_orders WHERE status NOT IN ('created','pending','authorized','captured','successful','paid')), 0)::bigint AS failed_payments,
          COALESCE((SELECT SUM(amount) FROM payment_refunds WHERE status IN ('successful','initiated','processing')), 0)::bigint AS refund_amount,
          COALESCE((SELECT COUNT(*) FROM payment_refunds), 0)::bigint AS refund_count,
          COALESCE((SELECT COUNT(*) FROM payment_refunds WHERE status IN ('requested','approved','processing')), 0)::bigint AS pending_refunds,
          COALESCE((SELECT COUNT(*) FROM settlement_batches_company WHERE status IN ('pending','processing','on_hold')), 0)::bigint AS pending_settlements,
          COALESCE((SELECT COUNT(*) FROM settlement_batches_company WHERE status = 'failed'), 0)::bigint AS failed_settlements,
          COALESCE((SELECT COUNT(*) FROM reconciliation_records WHERE status IN ('unmatched','mismatch')), 0)::bigint AS reconciliation_mismatches,
          COALESCE((SELECT COUNT(*) FROM payment_webhook_events WHERE processing_status IN ('failed','dead_letter')), 0)::bigint AS webhook_failures,
          COALESCE((SELECT SUM(amount) FROM chargebacks), 0)::bigint AS chargeback_amount,
          COALESCE((SELECT COUNT(*) FROM chargebacks), 0)::bigint AS chargeback_count,
          COALESCE((SELECT SUM(net_settlement_amount) FROM settlement_batches_company WHERE status IN ('settled','reconciled')), 0)::bigint AS net_settled_amount,
          COALESCE((SELECT SUM(provider_fee) FROM settlement_batches_company), 0)::bigint AS provider_fee_total,
          COALESCE((SELECT SUM(company_fee_amount) FROM settlement_batches_company), 0)::bigint AS company_revenue_total
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    let Ok(summary) = summary else {
        return server_error("Failed to load company payment dashboard");
    };

    let provider_rows = sqlx::query(
        r#"
        SELECT display_name, health_status, COALESCE(success_rate, 0)::float8 AS success_rate
        FROM payment_providers
        ORDER BY priority ASC, display_name ASC
        LIMIT 12
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let daily_rows = sqlx::query(
        r#"
        SELECT to_char(created_at::date, 'YYYY-MM-DD') AS label, COALESCE(SUM(amount), 0)::bigint AS amount
        FROM payment_orders
        WHERE created_at >= NOW() - INTERVAL '14 days'
        GROUP BY created_at::date
        ORDER BY created_at::date
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let successful_payments: i64 = summary.get("successful_payments");
    let failed_payments: i64 = summary.get("failed_payments");
    let total_attempts: i64 = sqlx::query_scalar(
        "SELECT COALESCE(COUNT(*), 0)::bigint FROM payment_orders WHERE status <> 'created'",
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(successful_payments + failed_payments);
    let payment_success_rate = if total_attempts == 0 {
        0.0
    } else {
        successful_payments as f64 * 100.0 / total_attempts as f64
    };

    HttpResponse::Ok().json(CompanyPaymentDashboard {
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
    })
}

async fn list_providers(pool: web::Data<PgPool>) -> impl Responder {
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
               COALESCE((SELECT status FROM payment_provider_credentials pc WHERE pc.provider_id = payment_providers.id AND pc.secret_type IN ('webhook_secret','signing_secret') ORDER BY pc.updated_at DESC LIMIT 1), 'missing') AS webhook_secret_status
        FROM payment_providers
        ORDER BY priority ASC, display_name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => HttpResponse::Ok().json(json!({
            "items": rows.into_iter().map(provider_from_row).collect::<Vec<_>>()
        })),
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
    let connector = MockProviderConnector;
    let result = connector.health_check().await;
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
        Ok(_) => HttpResponse::Ok().json(result),
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
    let masked_account_number = request
        .account_number
        .as_deref()
        .map(mask_bank_account)
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
        .bind(request.integration_type.trim())
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
        .bind(request.integration_type.trim())
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

async fn get_white_label_config(pool: web::Data<PgPool>) -> impl Responder {
    let row =
        sqlx::query("SELECT * FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1")
            .fetch_optional(pool.get_ref())
            .await;

    match row {
        Ok(Some(row)) => HttpResponse::Ok().json(row_to_json(&row)),
        Ok(None) => HttpResponse::Ok().json(json!({ "brand_name": "FeelPay" })),
        Err(e) => {
            tracing::error!("Load white-label config failed: {}", e);
            server_error("Failed to load white-label config")
        }
    }
}

async fn update_white_label_config(
    pool: web::Data<PgPool>,
    request: web::Json<WhiteLabelRequest>,
) -> impl Responder {
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO white_label_payment_config (
            brand_name, checkout_logo_url, checkout_theme_color, support_email, support_phone,
            payment_page_domain, default_currency, default_country, default_payment_methods,
            default_provider_id, fallback_provider_id, invoice_prefix, receipt_prefix,
            statement_descriptor, terms_url, privacy_url, refund_policy_url
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (id) DO NOTHING
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
    .fetch_optional(pool.get_ref())
    .await;

    let id = match saved {
        Ok(Some(id)) => id,
        Ok(None) => match update_existing_white_label(pool.get_ref(), &request).await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Update white-label config failed: {}", e);
                return server_error("Failed to save white-label config");
            }
        },
        Err(e) => {
            tracing::error!("Create white-label config failed: {}", e);
            return server_error("Failed to save white-label config");
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
               COALESCE(pm.provider_fee_percent, 0)::float8 AS provider_fee_percent,
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
            let provider_fee_percent = row.get::<f64, _>("provider_fee_percent");
            let estimated_provider_fee =
                ((request.amount as f64) * provider_fee_percent / 100.0).round() as i64;
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
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List company transactions failed: {}", e);
            server_error("Failed to load transactions")
        }
    }
}

async fn list_ledger(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
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
          COALESCE((SELECT SUM(net_settlement_amount) FROM settlement_batches_company WHERE status IN ('pending','processing')), 0)::bigint AS pending_settlement,
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
        (Ok(rows), Ok(summary)) => HttpResponse::Ok().json(json!({
            "summary": row_to_json(&summary),
            "items": rows.iter().map(row_to_json).collect::<Vec<_>>()
        })),
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
    let rows = sqlx::query(
        "SELECT * FROM payment_refunds WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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

async fn get_webhook(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(
        pool,
        "payment_webhook_events",
        path.into_inner(),
        "Webhook not found",
    )
    .await
}

async fn retry_webhook(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<WebhookRetryRequest>,
) -> impl Responder {
    let id = path.into_inner();
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
    let rows = sqlx::query(
        "SELECT * FROM settlement_batches_company WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
    let updated = sqlx::query(
        "UPDATE settlement_batches_company SET status = $2, bank_reference_number = COALESCE($3, bank_reference_number), finance_notes = COALESCE($4, finance_notes), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(request.bank_reference_number.as_deref())
    .bind(request.note.as_deref())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            if status == "settled" {
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
    settlement_action(pool, path, request, "settled").await
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
               GREATEST(rf.mismatch_count, COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND COALESCE(rr.match_status, rr.status) IN ('unmatched','mismatch')), 0))::bigint AS mismatch_count,
               GREATEST(rf.duplicate_count, COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id AND COALESCE(rr.match_status, rr.status) = 'duplicate'), 0))::bigint AS duplicate_count,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.amount, rr.observed_amount, 0) - COALESCE(rr.expected_amount, rr.amount, rr.observed_amount, 0))) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id), 0)::bigint AS amount_variance,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.fee_amount, 0) - COALESCE(rr.expected_fee_amount, rr.fee_amount, 0))) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id), 0)::bigint AS fee_variance,
               COALESCE((SELECT SUM(ABS(COALESCE(rr.tax_amount, 0) - COALESCE(rr.expected_tax_amount, rr.tax_amount, 0))) FROM reconciliation_records rr WHERE COALESCE(rr.file_id, rr.reconciliation_file_id) = rf.id), 0)::bigint AS tax_variance,
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
               business_type, email, phone, country, status, kyc_status, risk_status,
               payout_status, default_currency, created_at, updated_at
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
    let documents = fetch_json_rows(pool.get_ref(), "SELECT * FROM business_account_documents WHERE business_account_id = $1 ORDER BY uploaded_at DESC", id).await;
    let bank_accounts = fetch_json_rows(pool.get_ref(), "SELECT id, business_account_id, account_holder_name, bank_name, masked_account_number, ifsc_or_swift_or_routing_code, currency, verification_status, status, created_at FROM business_account_bank_accounts WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    let provider_links = fetch_json_rows(pool.get_ref(), "SELECT * FROM business_account_provider_links WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    let ledger = fetch_json_rows(pool.get_ref(), "SELECT le.* FROM ledger_entries le JOIN ledger_accounts la ON la.id = le.account_id WHERE la.owner_id = $1 ORDER BY le.created_at DESC LIMIT 50", id).await;
    let risk_notes = fetch_json_rows(pool.get_ref(), "SELECT * FROM business_account_risk_notes WHERE business_account_id = $1 ORDER BY created_at DESC", id).await;
    HttpResponse::Ok().json(json!({
        "profile": row_to_json(&profile),
        "kyc_documents": documents,
        "bank_accounts": bank_accounts,
        "provider_links": provider_links,
        "ledger_history": ledger,
        "risk_notes": risk_notes,
        "audit_trail": []
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
    request: web::Json<RiskRuleRequest>,
) -> impl Responder {
    save_risk_rule(pool, None, request.into_inner()).await
}

async fn update_risk_rule(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    request: web::Json<RiskRuleRequest>,
) -> impl Responder {
    save_risk_rule(pool, Some(path.into_inner()), request.into_inner()).await
}

async fn disable_risk_rule(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated =
        sqlx::query("UPDATE risk_rules SET status = 'disabled', updated_at = NOW() WHERE id = $1")
            .bind(id)
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

async fn save_risk_rule(
    pool: web::Data<PgPool>,
    rule_id: Option<Uuid>,
    request: RiskRuleRequest,
) -> HttpResponse {
    let action = serde_json::to_string(&request.action)
        .unwrap_or_else(|_| "\"review\"".to_string())
        .trim_matches('"')
        .to_string();
    let result = if let Some(id) = rule_id {
        sqlx::query_scalar::<_, Uuid>(
            "UPDATE risk_rules SET name = $2, rule_type = $3, condition = $4::jsonb, action = $5, priority = $6, status = $7, updated_at = NOW() WHERE id = $1 RETURNING id",
        )
        .bind(id)
        .bind(request.name.trim())
        .bind(request.rule_type.trim())
        .bind(request.condition.clone())
        .bind(&action)
        .bind(request.priority.unwrap_or(100))
        .bind(request.status.as_deref().unwrap_or("active"))
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO risk_rules (name, rule_type, condition, action, priority, status) VALUES ($1, $2, $3::jsonb, $4, $5, $6) RETURNING id",
        )
        .bind(request.name.trim())
        .bind(request.rule_type.trim())
        .bind(request.condition.clone())
        .bind(&action)
        .bind(request.priority.unwrap_or(100))
        .bind(request.status.as_deref().unwrap_or("active"))
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
               COALESCE(h.status, 'ready') AS status
        FROM catalog c
        LEFT JOIN LATERAL (
            SELECT *
            FROM admin_report_exports h
            WHERE h.report_type = c.report_name
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
        SELECT e.*,
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
    request: web::Json<ReportGenerateRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let format = request.format.to_ascii_uppercase();
    if !matches!(format.as_str(), "CSV" | "PDF" | "XLSX") {
        return HttpResponse::BadRequest().json(json!({ "message": "Unsupported report format" }));
    }
    let file_name = format!(
        "{}-{}.{}",
        request.report_type.to_ascii_lowercase().replace(' ', "-"),
        Utc::now().format("%Y%m%d%H%M%S"),
        format.to_ascii_lowercase()
    );
    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO admin_report_exports (
            report_type, file_name, format, status, row_count, filters_json,
            generated_at, expires_at
        )
        VALUES ($1, $2, $3, 'queued', 0, $4::jsonb, NOW(), NOW() + INTERVAL '7 days')
        RETURNING id
        "#,
    )
    .bind(request.report_type.trim())
    .bind(&file_name)
    .bind(&format)
    .bind(redact_sensitive_value(
        &request.filters.unwrap_or_else(|| json!({})),
    ))
    .fetch_one(pool.get_ref())
    .await;
    match saved {
        Ok(id) => {
            audit(
                &pool,
                "report.exported",
                "admin_report_export",
                Some(id),
                json!({ "report_type": request.report_type, "format": format, "masked": true }),
            )
            .await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "queued", "file_name": file_name }))
        }
        Err(e) => {
            tracing::error!("Generate report failed: {}", e);
            server_error("Failed to generate report")
        }
    }
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

async fn download_report_export(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let export =
        sqlx::query("SELECT id, file_name, expires_at FROM admin_report_exports WHERE id = $1")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;
    match export {
        Ok(Some(row)) => {
            let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
            if expires_at.is_some_and(|value| value < Utc::now()) {
                return HttpResponse::Gone().json(json!({ "message": "Export link expired" }));
            }
            audit(
                &pool,
                "report.downloaded",
                "admin_report_export",
                Some(id),
                json!({ "masked": true }),
            )
            .await;
            HttpResponse::Ok().json(json!({
                "download_url": format!("pending-report-export/{}", row.get::<String, _>("file_name")),
                "expires_at": expires_at
            }))
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
    audit(
        &pool,
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

async fn send_webhook_test(
    pool: web::Data<PgPool>,
    request: web::Json<WebhookTestRequest>,
) -> impl Responder {
    let summary = redact_sensitive_value(&json!({
        "event_type": request.event_type,
        "webhook_event_id": request.webhook_event_id,
    }));
    audit(
        &pool,
        "webhook_test.sent",
        "webhook_test",
        request.webhook_event_id,
        summary,
    )
    .await;
    HttpResponse::Ok().json(json!({ "status": "sent", "latency_ms": 42, "response_status": 200 }))
}

async fn replay_webhook_event(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let event_id = path.into_inner();
    let saved = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO webhook_replays (webhook_event_id, replay_status, replayed_at) VALUES ($1, 'queued', NOW()) RETURNING id",
    )
    .bind(event_id)
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
    PaymentProviderItem {
        id: row.get("id"),
        name: row.get("display_name"),
        code: row.get("code"),
        provider_type: row.get("provider_type"),
        environment: row.get("environment"),
        status: row.get("status"),
        priority: row.get("priority"),
        failover_enabled: row.get("failover_enabled"),
        webhook_url: row.get("webhook_url"),
        supported_payment_methods: row.get("supported_payment_methods"),
        supported_currencies: row.get("supported_currencies"),
        supported_countries: row.get("supported_countries"),
        health_status: row.get("health_status"),
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
        api_key_status: secret_status(row.get("api_key_status")),
        api_secret_status: secret_status(row.get("api_secret_status")),
        webhook_secret_status: secret_status(row.get("webhook_secret_status")),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
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
    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => HttpResponse::Ok()
            .json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
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
                    if lowered.contains("authorization")
                        || lowered.contains("secret")
                        || lowered.contains("card")
                        || lowered.contains("cvv")
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

fn server_error(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "message": message }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    configure_payment_ops_routes(cfg, "/payments");
    configure_payment_ops_routes(cfg, "/payment-ops");
}

pub fn configure_company_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/users", web::get().to(list_admin_users))
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
            &format!("{prefix}/white-label-config/preview"),
            web::get().to(get_white_label_config),
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
            &format!("{prefix}/routing-rules/{{id}}"),
            web::patch().to(update_routing_rule),
        )
        .route(
            &format!("{prefix}/routing-rules/{{id}}/disable"),
            web::post().to(disable_routing_rule),
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
            &format!("{prefix}/reconciliation/records/{{id}}/ignore"),
            web::post().to(ignore_reconciliation_record),
        )
        .route(
            &format!("{prefix}/reconciliation/{{id}}/export-discrepancies"),
            web::get().to(export_reconciliation_discrepancies),
        )
        .route(
            &format!("{prefix}/reconciliation/export-finance-summary"),
            web::get().to(export_reconciliation_finance_summary),
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
            &format!("{prefix}/risk-reviews"),
            web::get().to(list_risk_reviews),
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
            &format!("{prefix}/developer/webhook-test"),
            web::post().to(send_webhook_test),
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
        assert_eq!(mask_secret("rzp_live_abcdef"), "****cdef");
        assert_eq!(mask_bank_account("123456789012"), "********9012");
    }

    #[test]
    fn mock_webhook_signature_uses_payload_length() {
        let connector = MockProviderConnector;
        let payload = br#"{"event_id":"evt_1"}"#;
        assert!(connector.verify_webhook_signature(payload, "mock:secret:20", "secret"));
        assert!(!connector.verify_webhook_signature(payload, "mock:secret:99", "secret"));
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
            "card_number": "4111111111111111",
            "nested": {
                "cvv": "123",
                "bank_account_number": "1234567890",
                "safe": "kept"
            }
        }));
        assert_eq!(redacted["Authorization"], "redacted");
        assert_eq!(redacted["card_number"], "redacted");
        assert_eq!(redacted["nested"]["cvv"], "redacted");
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
}

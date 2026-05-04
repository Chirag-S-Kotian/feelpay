use actix_web::{web, HttpResponse, Responder};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    let total_attempts: i64 = sqlx::query_scalar("SELECT COALESCE(COUNT(*), 0)::bigint FROM payment_orders WHERE status <> 'created'")
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
            audit(&pool, "provider.saved", "payment_provider", Some(id), json!({ "code": request.code })).await;
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
            audit(&pool, &format!("provider.{status}"), "payment_provider", Some(provider_id), json!({ "status": status })).await;
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
    let environment: String = request.environment.clone().unwrap_or_else(|| row.get("environment"));
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
        return HttpResponse::BadRequest().json(json!({ "message": "secret_type and value are required" }));
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
            HttpResponse::Ok().json(json!({ "id": id, "masked_value": masked_value, "status": "active" }))
        }
        Err(e) => {
            tracing::error!("Save provider credential failed: {}", e);
            server_error("Failed to save credential")
        }
    }
}

async fn validate_credential(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_credential_status(pool, path.into_inner(), "active", "credential.validated", true).await
}

async fn disable_credential(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    update_credential_status(pool, path.into_inner(), "disabled", "credential.disabled", false).await
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
            audit(&pool, action, "payment_provider_credential", Some(id), json!({ "status": status })).await;
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

async fn disable_bank_integration(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
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
            audit(&pool, &format!("bank_config.{status}"), "bank_integration_config", Some(id), json!({ "status": status })).await;
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
        .map(|(encrypted, nonce)| json!({ "account_number": encrypted, "nonce": nonce }).to_string());

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
        .bind(request.account_scope.as_deref().unwrap_or("company_settlement"))
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
        .bind(request.account_scope.as_deref().unwrap_or("company_settlement"))
        .bind(request.currency.as_deref().unwrap_or("INR"))
        .bind(request.settlement_cycle.as_deref())
        .bind(json!(request.supported_currencies.unwrap_or_default()))
        .bind(json!(request.supported_payment_methods.unwrap_or_default()))
        .fetch_optional(pool.get_ref())
        .await
    };

    match result {
        Ok(Some(id)) => {
            audit(&pool, "bank_config.saved", "bank_integration_config", Some(id), json!({ "bank": request.acquiring_bank_name })).await;
            HttpResponse::Ok().json(json!({ "id": id }))
        }
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": "Bank integration not found" })),
        Err(e) => {
            tracing::error!("Save bank integration failed: {}", e);
            server_error("Failed to save bank integration")
        }
    }
}

async fn get_white_label_config(pool: web::Data<PgPool>) -> impl Responder {
    let row = sqlx::query("SELECT * FROM white_label_payment_config ORDER BY created_at ASC LIMIT 1")
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

    audit(&pool, "white_label_config.saved", "white_label_payment_config", Some(id), json!({ "brand_name": request.brand_name })).await;
    HttpResponse::Ok().json(json!({ "id": id }))
}

async fn update_existing_white_label(pool: &PgPool, request: &WhiteLabelRequest) -> Result<Uuid, sqlx::Error> {
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
    let updated = sqlx::query("UPDATE payment_routing_rules SET status = 'disabled', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(&pool, "routing_rule.disabled", "payment_routing_rule", Some(id), json!({ "status": "disabled" })).await;
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
                failover_provider_id = $14, status = $15, updated_at = NOW()
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
        .fetch_optional(pool.get_ref())
        .await
    } else {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO payment_routing_rules (
                name, payment_method, currency, country, amount_min, amount_max, provider_id,
                bank_integration_id, priority, risk_level, customer_region, business_line,
                failover_provider_id, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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
        .fetch_optional(pool.get_ref())
        .await
    };

    match result {
        Ok(Some(id)) => {
            audit(&pool, "routing_rule.saved", "payment_routing_rule", Some(id), json!({ "name": request.name })).await;
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
        SELECT r.id AS rule_id, r.name AS rule_name, p.id AS provider_id,
               p.display_name AS provider_name, COALESCE(p.code, p.name) AS provider_code,
               p.health_status, fp.display_name AS fallback_provider_name
        FROM payment_routing_rules r
        JOIN payment_providers p ON p.id = r.provider_id
        LEFT JOIN payment_providers fp ON fp.id = COALESCE(r.fallback_provider_id, r.failover_provider_id)
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
        ORDER BY r.priority ASC, p.priority ASC
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
        Ok(Some(row)) => HttpResponse::Ok().json(RoutingSimulationResponse {
            selected_provider_id: Some(row.get("provider_id")),
            selected_provider_code: Some(row.get("provider_code")),
            selected_provider: Some(row.get("provider_name")),
            routing_rule_id: Some(row.get("rule_id")),
            matched_routing_rule: Some(row.get("rule_name")),
            fallback_provider: row.get("fallback_provider_name"),
            decision_reason: "highest priority active provider matched request; maintenance and failed health skipped".to_string(),
            estimated_provider_fee: request.amount * 18 / 1000,
            estimated_company_fee: request.amount * 7 / 1000,
            risk_decision: request.risk_level.clone().unwrap_or_else(|| "allow".to_string()),
            provider_health_used: row.get("health_status"),
            failover_applied: false,
        }),
        Ok(None) => HttpResponse::Ok().json(RoutingSimulationResponse {
            selected_provider_id: None,
            selected_provider_code: None,
            selected_provider: None,
            routing_rule_id: None,
            matched_routing_rule: None,
            fallback_provider: None,
            decision_reason: "No active routing rule matched; use default/fallback provider if configured".to_string(),
            estimated_provider_fee: 0,
            estimated_company_fee: 0,
            risk_decision: "review".to_string(),
            provider_health_used: "unknown".to_string(),
            failover_applied: false,
        }),
        Err(e) => {
            tracing::error!("Routing simulation failed: {}", e);
            server_error("Failed to simulate routing")
        }
    }
}

async fn list_transactions(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
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
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List company transactions failed: {}", e);
            server_error("Failed to load transactions")
        }
    }
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

    let attempts = fetch_json_rows(pool.get_ref(), "SELECT * FROM payment_attempts WHERE payment_order_id = $1 ORDER BY created_at DESC", id).await;
    let webhooks = fetch_json_rows(pool.get_ref(), "SELECT * FROM payment_webhook_events WHERE related_payment_id = $1 ORDER BY received_at DESC", id).await;
    let refunds = fetch_json_rows(pool.get_ref(), "SELECT * FROM payment_refunds WHERE payment_order_id = $1 ORDER BY created_at DESC", id).await;

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
    let updated = sqlx::query("UPDATE payment_orders SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(&pool, &format!("transaction.{action}"), "payment_order", Some(id), json!({ "status": status })).await;
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
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
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
            payment_order_id, provider_id, amount, currency, reason, refund_source, idempotency_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(request.payment_order_id)
    .bind(request.provider_id)
    .bind(request.amount)
    .bind(request.currency.as_deref().unwrap_or("INR"))
    .bind(request.reason.as_deref())
    .bind(request.refund_source.as_deref().unwrap_or("company"))
    .bind(request.idempotency_key)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(id) => {
            audit(&pool, "refund.requested", "payment_refund", Some(id), json!({ "amount": request.amount })).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "requested" }))
        }
        Err(e) => {
            tracing::error!("Create refund failed: {}", e);
            server_error("Failed to create refund")
        }
    }
}

async fn get_refund(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(pool, "payment_refunds", path.into_inner(), "Refund not found").await
}

async fn refund_status_action(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    status: &'static str,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query("UPDATE payment_refunds SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(&pool, &format!("refund.{status}"), "payment_refund", Some(id), json!({ "status": status })).await;
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
    let (encrypted_secret, nonce, masked_secret) = match request.signing_secret.as_deref().filter(|v| !v.trim().is_empty()) {
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
            audit(&pool, "webhook_config.saved", "webhook_endpoint_config", Some(id), json!({ "provider_id": request.provider_id, "environment": request.environment })).await;
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
    audit(&pool, "webhook_config.tested", "webhook_endpoint_config", Some(id), json!({ "status": "queued" })).await;
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
            audit(&pool, "webhook_secret.rotated", "webhook_endpoint_config", Some(id), json!({ "masked_signing_secret": masked })).await;
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
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List webhook events failed: {}", e);
            server_error("Failed to load webhooks")
        }
    }
}

async fn get_webhook(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(pool, "payment_webhook_events", path.into_inner(), "Webhook not found").await
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
            audit(&pool, "webhook.retry", "payment_webhook_event", Some(id), json!({ "reason": request.reason })).await;
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
            audit(&pool, "webhook.ignored", "payment_webhook_event", Some(id), json!({ "processing_status": "ignored" })).await;
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
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("List company settlements failed: {}", e);
            server_error("Failed to load settlements")
        }
    }
}

async fn get_settlement(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    get_by_id(pool, "settlement_batches_company", path.into_inner(), "Settlement not found").await
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
            audit(&pool, &format!("settlement.{status}"), "settlement_batch", Some(id), json!({ "status": status })).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Settlement not found" })),
        Err(e) => {
            tracing::error!("Settlement action failed: {}", e);
            server_error("Failed to update settlement")
        }
    }
}

async fn hold_settlement(pool: web::Data<PgPool>, path: web::Path<Uuid>, request: web::Json<SettlementActionRequest>) -> impl Responder {
    settlement_action(pool, path, request, "on_hold").await
}

async fn release_settlement(pool: web::Data<PgPool>, path: web::Path<Uuid>, request: web::Json<SettlementActionRequest>) -> impl Responder {
    settlement_action(pool, path, request, "processing").await
}

async fn mark_settlement_settled(pool: web::Data<PgPool>, path: web::Path<Uuid>, request: web::Json<SettlementActionRequest>) -> impl Responder {
    settlement_action(pool, path, request, "settled").await
}

async fn mark_settlement_failed(pool: web::Data<PgPool>, path: web::Path<Uuid>, request: web::Json<SettlementActionRequest>) -> impl Responder {
    settlement_action(pool, path, request, "failed").await
}

async fn export_settlement(path: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok().json(json!({ "settlement_id": path.into_inner(), "download_url": "pending-export-job" }))
}

async fn list_reconciliation(pool: web::Data<PgPool>) -> impl Responder {
    query_items(
        pool,
        r#"
        SELECT rf.*,
               COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE rr.reconciliation_file_id = rf.id AND rr.status = 'matched'), 0)::bigint AS matched_count,
               COALESCE((SELECT COUNT(*) FROM reconciliation_records rr WHERE rr.reconciliation_file_id = rf.id AND rr.status IN ('unmatched','mismatch')), 0)::bigint AS mismatch_count
        FROM reconciliation_files rf
        ORDER BY rf.created_at DESC
        LIMIT 50
        "#,
        "Failed to load reconciliation files",
    )
    .await
}

async fn get_reconciliation(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let file = sqlx::query("SELECT * FROM reconciliation_files WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;
    let Ok(Some(row)) = file else {
        return HttpResponse::NotFound().json(json!({ "message": "Reconciliation file not found" }));
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
        "UPDATE reconciliation_records SET status = $2, resolution_notes = $3, resolved_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(request.status.trim())
    .bind(request.resolution_notes.trim())
    .execute(pool.get_ref())
    .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(&pool, "reconciliation.resolved", "reconciliation_record", Some(id), json!({ "status": request.status })).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": request.status }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Reconciliation record not found" })),
        Err(e) => {
            tracing::error!("Resolve reconciliation failed: {}", e);
            server_error("Failed to resolve reconciliation")
        }
    }
}

async fn auto_match_reconciliation(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    audit(&pool, "reconciliation.auto_match", "reconciliation_file", Some(id), json!({ "status": "queued" })).await;
    HttpResponse::Ok().json(json!({ "id": id, "status": "auto_match_queued" }))
}

async fn reconciliation_mismatches(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let rows = sqlx::query("SELECT * FROM reconciliation_records WHERE reconciliation_file_id = $1 AND status IN ('unmatched','mismatch','duplicate') ORDER BY created_at DESC")
        .bind(id)
        .fetch_all(pool.get_ref())
        .await;
    match rows {
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
        Err(e) => {
            tracing::error!("Load reconciliation mismatches failed: {}", e);
            server_error("Failed to load mismatches")
        }
    }
}

async fn export_reconciliation_discrepancies(path: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok().json(json!({ "reconciliation_id": path.into_inner(), "download_url": "pending-export-job" }))
}

async fn list_disputes(pool: web::Data<PgPool>) -> impl Responder {
    query_items(pool, "SELECT * FROM disputes_company ORDER BY created_at DESC LIMIT 50", "Failed to load disputes").await
}

async fn list_risk_rules(pool: web::Data<PgPool>) -> impl Responder {
    query_items(pool, "SELECT * FROM risk_rules ORDER BY priority ASC, created_at DESC", "Failed to load risk rules").await
}

async fn create_risk_rule(pool: web::Data<PgPool>, request: web::Json<RiskRuleRequest>) -> impl Responder {
    save_risk_rule(pool, None, request.into_inner()).await
}

async fn update_risk_rule(pool: web::Data<PgPool>, path: web::Path<Uuid>, request: web::Json<RiskRuleRequest>) -> impl Responder {
    save_risk_rule(pool, Some(path.into_inner()), request.into_inner()).await
}

async fn disable_risk_rule(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query("UPDATE risk_rules SET status = 'disabled', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;
    match updated {
        Ok(result) if result.rows_affected() > 0 => {
            audit(&pool, "risk_rule.disabled", "risk_rule", Some(id), json!({ "status": "disabled" })).await;
            HttpResponse::Ok().json(json!({ "id": id, "status": "disabled" }))
        }
        Ok(_) => HttpResponse::NotFound().json(json!({ "message": "Risk rule not found" })),
        Err(e) => {
            tracing::error!("Disable risk rule failed: {}", e);
            server_error("Failed to disable risk rule")
        }
    }
}

async fn save_risk_rule(pool: web::Data<PgPool>, rule_id: Option<Uuid>, request: RiskRuleRequest) -> HttpResponse {
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
            audit(&pool, "risk_rule.saved", "risk_rule", Some(id), json!({ "name": request.name, "action": action })).await;
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
    query_items(pool, "SELECT * FROM risk_reviews ORDER BY created_at DESC LIMIT 50", "Failed to load risk reviews").await
}

async fn list_audit_logs(pool: web::Data<PgPool>) -> impl Responder {
    query_items(pool, "SELECT * FROM audit_logs_company ORDER BY created_at DESC LIMIT 100", "Failed to load audit logs").await
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
        masked_webhook_secret: presence_mask(row.get::<Option<String>, _>("webhook_secret_encrypted")),
        api_key_status: secret_status(row.get("api_key_status")),
        api_secret_status: secret_status(row.get("api_secret_status")),
        webhook_secret_status: secret_status(row.get("webhook_secret_status")),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

async fn get_by_id(pool: web::Data<PgPool>, table: &'static str, id: Uuid, not_found: &'static str) -> HttpResponse {
    let sql = format!("SELECT * FROM {table} WHERE id = $1");
    let row = sqlx::query(&sql).bind(id).fetch_optional(pool.get_ref()).await;
    match row {
        Ok(Some(row)) => HttpResponse::Ok().json(row_to_json(&row)),
        Ok(None) => HttpResponse::NotFound().json(json!({ "message": not_found })),
        Err(e) => {
            tracing::error!("Lookup failed for {}: {}", table, e);
            server_error("Lookup failed")
        }
    }
}

async fn query_items(pool: web::Data<PgPool>, sql: &'static str, error_message: &'static str) -> HttpResponse {
    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => HttpResponse::Ok().json(json!({ "items": rows.iter().map(row_to_json).collect::<Vec<_>>() })),
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
    format!("****{}", chars[chars.len() - 4..].iter().collect::<String>())
}

fn mask_bank_account(value: &str) -> String {
    let digits = value.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<Vec<_>>();
    if digits.len() <= 4 {
        return "****".to_string();
    }
    format!("********{}", digits[digits.len() - 4..].iter().collect::<String>())
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

async fn audit(pool: &web::Data<PgPool>, action: &str, entity_type: &str, entity_id: Option<Uuid>, summary: Value) {
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

fn configure_payment_ops_routes(cfg: &mut web::ServiceConfig, prefix: &'static str) {
    cfg.route(&format!("{prefix}/dashboard"), web::get().to(dashboard))
        .route(&format!("{prefix}/providers"), web::get().to(list_providers))
        .route(&format!("{prefix}/providers"), web::post().to(create_provider))
        .route(&format!("{prefix}/providers/{{id}}"), web::patch().to(update_provider))
        .route(&format!("{prefix}/providers/{{id}}/test"), web::post().to(test_provider))
        .route(&format!("{prefix}/providers/{{id}}/maintenance"), web::post().to(maintenance_provider))
        .route(&format!("{prefix}/providers/{{id}}/enable"), web::post().to(enable_provider))
        .route(&format!("{prefix}/providers/{{id}}/disable"), web::post().to(disable_provider))
        .route(
            &format!("{prefix}/providers/{{id}}/rotate-credentials"),
            web::post().to(rotate_credentials),
        )
        .route(&format!("{prefix}/credentials"), web::get().to(list_credentials))
        .route(&format!("{prefix}/credentials"), web::post().to(create_credential))
        .route(&format!("{prefix}/credentials/{{id}}/validate"), web::post().to(validate_credential))
        .route(&format!("{prefix}/credentials/{{id}}/rotate"), web::post().to(rotate_credentials_by_id))
        .route(&format!("{prefix}/credentials/{{id}}/disable"), web::post().to(disable_credential))
        .route(&format!("{prefix}/webhook-configs"), web::get().to(list_webhook_configs))
        .route(&format!("{prefix}/webhook-configs"), web::post().to(save_webhook_config))
        .route(&format!("{prefix}/webhook-configs/{{id}}"), web::patch().to(save_webhook_config))
        .route(&format!("{prefix}/webhook-configs/{{id}}/test"), web::post().to(test_webhook_config))
        .route(&format!("{prefix}/webhook-configs/{{id}}/rotate-secret"), web::post().to(rotate_webhook_secret))
        .route(&format!("{prefix}/bank-integrations"), web::get().to(list_bank_integrations))
        .route(&format!("{prefix}/bank-integrations"), web::post().to(create_bank_integration))
        .route(
            &format!("{prefix}/bank-integrations/{{id}}"),
            web::patch().to(update_bank_integration),
        )
        .route(&format!("{prefix}/bank-integrations/{{id}}/verify"), web::post().to(verify_bank_integration))
        .route(&format!("{prefix}/bank-integrations/{{id}}/disable"), web::post().to(disable_bank_integration))
        .route(&format!("{prefix}/white-label-config"), web::get().to(get_white_label_config))
        .route(&format!("{prefix}/white-label-config"), web::patch().to(update_white_label_config))
        .route(&format!("{prefix}/white-label-config/preview"), web::get().to(get_white_label_config))
        .route(&format!("{prefix}/routing-rules"), web::get().to(list_routing_rules))
        .route(&format!("{prefix}/routing-rules"), web::post().to(create_routing_rule))
        .route(&format!("{prefix}/routing-rules/{{id}}"), web::patch().to(update_routing_rule))
        .route(&format!("{prefix}/routing-rules/{{id}}/disable"), web::post().to(disable_routing_rule))
        .route(&format!("{prefix}/routing-rules/simulate"), web::post().to(simulate_routing))
        .route(&format!("{prefix}/routing/simulate"), web::post().to(simulate_routing))
        .route(&format!("{prefix}/transactions"), web::get().to(list_transactions))
        .route(&format!("{prefix}/transactions/{{id}}"), web::get().to(get_transaction))
        .route(&format!("{prefix}/transactions/{{id}}/capture"), web::post().to(capture_transaction))
        .route(&format!("{prefix}/transactions/{{id}}/cancel"), web::post().to(cancel_transaction))
        .route(&format!("{prefix}/transactions/{{id}}/refund"), web::post().to(create_refund))
        .route(&format!("{prefix}/refunds"), web::get().to(list_refunds))
        .route(&format!("{prefix}/refunds"), web::post().to(|pool: web::Data<PgPool>, body: web::Json<RefundRequest>| async move { insert_refund(pool, body.into_inner()).await }))
        .route(&format!("{prefix}/refunds/{{id}}"), web::get().to(get_refund))
        .route(&format!("{prefix}/refunds/{{id}}/approve"), web::post().to(approve_refund))
        .route(&format!("{prefix}/refunds/{{id}}/reject"), web::post().to(reject_refund))
        .route(&format!("{prefix}/refunds/{{id}}/initiate"), web::post().to(initiate_refund))
        .route(&format!("{prefix}/refunds/{{id}}/retry"), web::post().to(retry_refund))
        .route(&format!("{prefix}/webhooks"), web::get().to(list_webhooks))
        .route(&format!("{prefix}/webhook-events"), web::get().to(list_webhooks))
        .route(&format!("{prefix}/webhooks/{{id}}"), web::get().to(get_webhook))
        .route(&format!("{prefix}/webhook-events/{{id}}"), web::get().to(get_webhook))
        .route(&format!("{prefix}/webhooks/{{id}}/retry"), web::post().to(retry_webhook))
        .route(&format!("{prefix}/webhook-events/{{id}}/retry"), web::post().to(retry_webhook))
        .route(&format!("{prefix}/webhook-events/{{id}}/ignore"), web::post().to(ignore_webhook))
        .route(&format!("{prefix}/settlements"), web::get().to(list_settlements))
        .route(&format!("{prefix}/settlements/{{id}}"), web::get().to(get_settlement))
        .route(&format!("{prefix}/settlements/{{id}}/hold"), web::post().to(hold_settlement))
        .route(&format!("{prefix}/settlements/{{id}}/release"), web::post().to(release_settlement))
        .route(&format!("{prefix}/settlements/{{id}}/mark-settled"), web::post().to(mark_settlement_settled))
        .route(&format!("{prefix}/settlements/{{id}}/mark-failed"), web::post().to(mark_settlement_failed))
        .route(&format!("{prefix}/settlements/{{id}}/export"), web::get().to(export_settlement))
        .route(&format!("{prefix}/reconciliation"), web::get().to(list_reconciliation))
        .route(&format!("{prefix}/reconciliation/{{id}}"), web::get().to(get_reconciliation))
        .route(&format!("{prefix}/reconciliation/{{id}}/auto-match"), web::post().to(auto_match_reconciliation))
        .route(&format!("{prefix}/reconciliation/{{id}}/mismatches"), web::get().to(reconciliation_mismatches))
        .route(
            &format!("{prefix}/reconciliation/{{id}}/resolve"),
            web::post().to(resolve_reconciliation),
        )
        .route(&format!("{prefix}/reconciliation/{{id}}/export-discrepancies"), web::get().to(export_reconciliation_discrepancies))
        .route(&format!("{prefix}/disputes"), web::get().to(list_disputes))
        .route(&format!("{prefix}/risk-rules"), web::get().to(list_risk_rules))
        .route(&format!("{prefix}/risk-rules"), web::post().to(create_risk_rule))
        .route(&format!("{prefix}/risk-rules/{{id}}"), web::patch().to(update_risk_rule))
        .route(&format!("{prefix}/risk-rules/{{id}}/disable"), web::post().to(disable_risk_rule))
        .route(&format!("{prefix}/risk-reviews"), web::get().to(list_risk_reviews))
        .route(&format!("{prefix}/audit-logs"), web::get().to(list_audit_logs))
        .route(&format!("{prefix}/admin-roles"), web::get().to(list_roles));
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
        assert_eq!(response.provider_reference, Some("mock_refund_idem-1".to_string()));
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
}

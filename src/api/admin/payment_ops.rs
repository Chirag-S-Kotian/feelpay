use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::{json, Value};
use uuid::Uuid;

static IDEMPOTENCY: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/admin/stats").route(web::get().to(admin_stats)))
        .service(
            web::scope("/admin/payment-ops")
                .route("/dashboard", web::get().to(payment_ops_dashboard))
                .route("/{tail:.*}", web::get().to(payment_ops_get))
                .route("/{tail:.*}", web::post().to(payment_ops_write))
                .route("/{tail:.*}", web::patch().to(payment_ops_write))
                .route("/{tail:.*}", web::put().to(payment_ops_write))
                .route("/{tail:.*}", web::delete().to(payment_ops_write)),
        )
        .service(web::resource("/admin/platform-settings").route(web::get().to(platform_settings)))
        .service(
            web::resource("/admin/platform-settings")
                .route(web::patch().to(write_ok))
                .route(web::post().to(write_ok)),
        )
        .service(web::resource("/admin/users/invite").route(web::post().to(write_ok)))
        .service(web::resource("/admin/users/{tail:.*}").route(web::post().to(write_ok)))
        .service(web::resource("/admin/exports").route(web::get().to(exports)))
        .service(web::resource("/admin/exports/bulk").route(web::post().to(export_bulk)))
        .service(
            web::resource("/admin/exports/{tail:.*}/download")
                .route(web::get().to(export_download)),
        )
        .service(web::resource("/admin/system-health/{tail:.*}").route(web::post().to(write_ok)))
        .service(web::resource("/admin/notifications/{tail:.*}").route(web::post().to(write_ok)));
}

async fn admin_stats() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "total_volume": 125000,
        "active_merchants": 3,
        "failed_payments": 1,
        "system_health": "Healthy"
    }))
}

async fn payment_ops_dashboard() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "total_gmv": 125000,
        "payment_success_rate": 98.4,
        "webhook_failures": 0,
        "pending_refunds": 0,
        "pending_settlements": 1,
        "reconciliation_mismatches": 0,
        "company_revenue_total": 2500
    }))
}

async fn platform_settings() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "support_email": "support@feelpay.local",
        "settlement_cycle_days": 2,
        "maintenance_mode": false,
        "security_posture": "Safe",
        "cvv_storage": "Disabled"
    }))
}

async fn payment_ops_get(req: HttpRequest) -> HttpResponse {
    let tail = req.match_info().query("tail");
    let body = match tail {
        "connectors" | "providers" => json!({ "items": providers() }),
        "credentials" | "developer/api-keys" => json!({ "items": [{
            "id": sandbox_id(),
            "name": "Sandbox key",
            "environment": "sandbox",
            "key_prefix": "fp_test",
            "masked_key": "fp_test_****1234",
            "scopes": "payments:read, webhooks:test",
            "status": "active"
        }] }),
        "routing/provider-performance" => json!({ "items": [provider_performance()] }),
        "routing-rules" => json!({ "items": [{
            "id": sandbox_id(),
            "name": "Mock route",
            "rule_group": "Sandbox",
            "strategy": "priority",
            "payment_method": "upi",
            "currency": "INR",
            "provider_name": "Mock Provider",
            "status": "active"
        }] }),
        "transactions" => json!({ "items": [transaction()] }),
        "ledger" => json!({
            "summary": { "total_balance": 0, "debits": 1250, "credits": 1250 },
            "items": ledger_entries()
        }),
        "ledger/export" => {
            json!({ "status": "ready", "download_url": "/admin/payment-ops/ledger/export.csv" })
        }
        "refunds" => json!({ "items": [] }),
        "settlements" => json!({ "items": [{
            "id": sandbox_id(),
            "currency": "INR",
            "amount": 1250,
            "status": "pending",
            "masked_account": "****5678",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        "disputes" => json!({ "items": [] }),
        "risk-rules" => json!({ "items": [{
            "id": sandbox_id(),
            "name": "Sandbox velocity check",
            "rule_type": "Velocity",
            "condition": "Amount above threshold",
            "action": "Review",
            "priority": 10,
            "status": "active"
        }] }),
        "risk-reviews" | "risk-lists" | "risk-decisions" | "risk-signals" => json!({ "items": [] }),
        "reconciliation" => json!({
            "items": [],
            "queue": [{
                "id": sandbox_id(),
                "row_number": 1,
                "external_reference": "sandbox-ref",
                "internal_reference": "sandbox-order",
                "amount": 1250,
                "currency": "INR",
                "record_type": "payment",
                "match_status": "matched",
                "confidence_score": 99
            }]
        }),
        "business-accounts" => json!({ "items": business_accounts() }),
        "bank-integrations" => json!({ "items": [{
            "id": sandbox_id(),
            "name": "Sandbox Settlement Bank",
            "integration_type": "API / sandbox test",
            "currency": "INR",
            "masked_account_number": "****5678",
            "masked_account_display": "****5678",
            "verification_status": "verified",
            "status": "active"
        }] }),
        "white-label-config" => white_label_config(),
        "white-label/payment-links" => json!({ "items": [{
            "id": sandbox_id(),
            "title": "Sandbox Checkout",
            "amount": 1250,
            "currency": "INR",
            "status": "active",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        "white-label/sessions" => json!({ "items": [{
            "id": sandbox_id(),
            "payment_link_id": sandbox_id(),
            "selected_payment_method": "upi",
            "status": "paid",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        "white-label/receipts" => json!({ "items": [{
            "id": sandbox_id(),
            "payment_session_id": sandbox_id(),
            "receipt_number": "RCPT-SANDBOX-001",
            "recipient_email": "sandbox-buyer@feelpay.local",
            "status": "sent",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        "webhook-events" | "developer/webhook-events" => {
            json!({ "items": webhook_events(), "rows": webhook_events() })
        }
        "webhook-processing-attempts" => json!({ "items": [{
            "id": sandbox_id(),
            "webhook_event_id": sandbox_id(),
            "attempt_number": 1,
            "status": "success",
            "started_at": "2026-05-07T00:00:00Z"
        }] }),
        "webhook-configs" => json!({ "items": [{
            "id": sandbox_id(),
            "endpoint_url": "https://webhooks.feelpay.local/sandbox",
            "status": "active",
            "events": "payment.captured"
        }] }),
        "reports" => json!({ "items": reports() }),
        "reports/history" | "report-exports" => json!({ "items": [{
            "id": sandbox_id(),
            "report_name": "Sandbox transactions",
            "format": "CSV",
            "status": "READY",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        tail if tail.starts_with("reports/download/") => {
            json!({ "status": "READY", "download_url": "/sandbox-report.csv" })
        }
        "developer/request-logs" => json!({ "items": [] }),
        "developer/replays" | "developer/webhook-replays" => json!({ "items": [{
            "id": sandbox_id(),
            "webhook_event_id": sandbox_id(),
            "replay_status": "success",
            "created_at": "2026-05-07T00:00:00Z"
        }] }),
        "developer/sample-payloads" => json!({ "items": [{
            "id": "payment-captured",
            "name": "Payment captured",
            "sample": { "event": "payment.captured", "card": "redacted" }
        }] }),
        "developer/api-versions"
        | "developer/idempotency-records"
        | "developer/error-catalog"
        | "developer/rate-limits" => json!({ "items": [] }),
        "system-health" => json!({ "items": [] }),
        "system-health/providers" => json!({ "items": [provider_performance()] }),
        "system-health/jobs"
        | "system-health/metrics"
        | "system-health/slos"
        | "system-health/incidents"
        | "system-health/traces" => json!({ "items": [] }),
        "notifications" => json!({ "items": [] }),
        "readiness-scorecard" => json!({ "items": readiness_rows(), "score": 92 }),
        "readiness-scorecard/export" => json!({ "status": "ready" }),
        "compliance/overview" => compliance_overview(),
        tail if tail.starts_with("compliance/") => json!({ "items": [] }),
        _ => json!({ "items": [] }),
    };

    HttpResponse::Ok().json(body)
}

async fn payment_ops_write(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let tail = req.match_info().query("tail");

    if tail == "white-label/payment-links" {
        if let Some(response) = check_idempotency(&req, &body) {
            return response;
        }
    }

    let payload = match tail {
        tail if tail.starts_with("providers/") && tail.ends_with("/test") => json!({
            "id": sandbox_id(),
            "status": "verified",
            "summary": {
                "passed": true,
                "message": "sandbox test passed"
            }
        }),
        "routing/simulate" => json!({
            "selected_provider": "Mock Provider",
            "result": "Mock route result",
            "reason": "Sandbox rule matched"
        }),
        "risk/simulate" => json!({
            "decision": "Review",
            "reason": "Sandbox simulator explains allow, review, or block outcomes"
        }),
        "reports/generate" => json!({
            "id": sandbox_id(),
            "export_id": sandbox_id(),
            "status": "READY",
            "download_url": "/sandbox-report.csv"
        }),
        "developer/webhook-test" => json!({
            "id": sandbox_id(),
            "event_id": sandbox_id(),
            "status": "processed"
        }),
        tail if tail.ends_with("/replay") => json!({ "id": sandbox_id(), "status": "success" }),
        "white-label/sessions" => {
            json!({ "id": sandbox_id(), "session_id": sandbox_id(), "status": "created" })
        }
        tail if tail.ends_with("/complete-test-payment") => {
            json!({ "id": sandbox_id(), "status": "captured" })
        }
        _ => json!({ "id": sandbox_id(), "status": "ok" }),
    };

    HttpResponse::Ok().json(payload)
}

async fn write_ok() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "id": sandbox_id(), "status": "ok" }))
}

async fn exports() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "exports": [{
            "job_id": sandbox_id(),
            "status": "completed",
            "created_at": "2026-05-07T00:00:00Z",
            "completed_at": "2026-05-07T00:01:00Z",
            "record_count": 3,
            "file_size_bytes": 2048
        }]
    }))
}

async fn export_bulk() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "job_id": sandbox_id(), "status": "queued" }))
}

async fn export_download() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "download_url": "/sandbox-export.csv" }))
}

fn check_idempotency(req: &HttpRequest, body: &[u8]) -> Option<HttpResponse> {
    let key = req
        .headers()
        .get("idempotency-key")?
        .to_str()
        .ok()?
        .to_owned();
    let hash = payload_hash(body);
    let store = IDEMPOTENCY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut records = store.lock().ok()?;

    match records.get(&key) {
        Some(existing) if *existing != hash => Some(HttpResponse::Conflict().json(json!({
            "message": "Idempotency key already used with a different payload",
            "request_id": Uuid::new_v4().to_string()
        }))),
        Some(_) => Some(HttpResponse::Ok().json(json!({
            "id": sandbox_id(),
            "status": "replayed"
        }))),
        None => {
            records.insert(key, hash);
            None
        }
    }
}

fn payload_hash(body: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hasher);
    hasher.finish()
}

fn sandbox_id() -> String {
    Uuid::nil().to_string()
}

fn providers() -> Vec<Value> {
    vec![
        json!({
            "id": sandbox_id(),
            "name": "Mock Provider",
            "code": "mock",
            "provider": "mock",
            "provider_type": "mock",
            "environment": "sandbox",
            "status": "active",
            "health_status": "Sandbox only",
            "success_rate": 98.4,
            "attempt_count": 12,
            "credentials_status": "Ready",
            "api_key_status": "verified",
            "api_secret_status": "verified",
            "webhook_secret_status": "verified",
            "readiness_score": 92
        }),
        json!({
            "id": Uuid::from_u128(1).to_string(),
            "name": "Razorpay",
            "code": "razorpay",
            "provider": "razorpay",
            "provider_type": "razorpay",
            "environment": "live",
            "status": "credentials_required",
            "health_status": "Credentials required",
            "success_rate": 0,
            "attempt_count": 0,
            "credentials_status": "Credentials required",
            "api_key_status": "missing",
            "api_secret_status": "missing",
            "webhook_secret_status": "missing",
            "readiness_score": 0
        }),
    ]
}

fn provider_performance() -> Value {
    json!({
        "provider": "Mock Provider",
        "payment_method": "upi",
        "currency": "INR",
        "country": "IN",
        "success_rate": 98.4,
        "failure_rate": 1.6,
        "avg_latency_ms": 120,
        "p95_latency_ms": 220,
        "provider_fee_percent": 1.2,
        "attempt_count": 12,
        "health": "Healthy"
    })
}

fn transaction() -> Value {
    json!({
        "id": sandbox_id(),
        "short_id": "pay_0000",
        "provider_name": "Mock Provider",
        "amount": 1250,
        "currency": "INR",
        "status": "captured",
        "payment_method": "upi",
        "created_at": "2026-05-07T00:00:00Z"
    })
}

fn ledger_entries() -> Vec<Value> {
    vec![
        json!({ "id": sandbox_id(), "transaction_group": "sandbox-payment", "entry_type": "debit", "amount": 1250, "currency": "INR", "status": "posted" }),
        json!({ "id": Uuid::from_u128(2).to_string(), "transaction_group": "sandbox-payment", "entry_type": "credit", "amount": 1250, "currency": "INR", "status": "posted" }),
    ]
}

fn business_accounts() -> Vec<Value> {
    vec![json!({
        "id": sandbox_id(),
        "business_name": "Sandbox Business",
        "name": "Sandbox Business",
        "email": "sandbox-business@feelpay.local",
        "business_type": "Private Limited",
        "kyc_status": "Verified",
        "risk_status": "Low risk",
        "payout_status": "Eligible",
        "onboarding_status": "approved",
        "created_at": "2026-05-07T00:00:00Z"
    })]
}

fn white_label_config() -> Value {
    json!({
        "brand_name": "FeelPay Sandbox",
        "theme_color": "#2563eb",
        "button_label": "Pay securely",
        "default_payment_methods": ["upi", "card"],
        "success_message": "Payment complete",
        "receipt_footer": "Thank you for testing with FeelPay."
    })
}

fn webhook_events() -> Vec<Value> {
    vec![json!({
        "id": sandbox_id(),
        "provider_id": sandbox_id(),
        "provider_event_id": "evt_sandbox_001",
        "event_type": "payment.captured",
        "event_version": "v1",
        "payload_hash": "sha256:masked",
        "dedupe_key": "masked",
        "idempotency_key": "masked",
        "signature_verification_status": "verified",
        "processing_status": "processed",
        "retry_count": 0,
        "received_at": "2026-05-07T00:00:00Z"
    })]
}

fn reports() -> Vec<Value> {
    vec![json!({
        "id": sandbox_id(),
        "report_name": "Sandbox transactions",
        "category": "Payments",
        "format": "CSV",
        "last_generated_at": "2026-05-07T00:00:00Z",
        "generated_by": "E2E",
        "status": "READY"
    })]
}

fn readiness_rows() -> Vec<Value> {
    vec![
        json!({ "id": "providers", "check": "Provider sandbox readiness", "status": "pass", "severity": "pass" }),
        json!({ "id": "webhooks", "check": "Webhook processing", "status": "pass", "severity": "pass" }),
    ]
}

fn compliance_overview() -> Value {
    json!({
        "items": readiness_rows(),
        "readiness": [
            { "check": "PCI controls", "status": "pass" },
            { "check": "CVV storage disabled", "status": "pass" }
        ],
        "controls": { "items": [] },
        "evidence": { "items": [] },
        "access_reviews": { "items": [] },
        "security_events": { "items": [] },
        "packets": { "items": [] },
        "tasks": { "items": [] }
    })
}

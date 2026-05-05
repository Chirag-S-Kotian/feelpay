//! Admin connector management API
//!
//! Provides readiness score, sandbox test, capability status,
//! and last connector error/success for each configured provider.
//! Company admin only.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::engine::connectors::{
    self, credential_loader, create_connector, ProviderCredentials,
};
use crate::error::FeelPayError;
use crate::state::AppState;

// ── Response DTOs ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ConnectorStatusResponse {
    pub provider: String,
    pub display_name: String,
    pub readiness_score: u8,
    pub available_keys: Vec<String>,
    pub required_keys: Vec<String>,
    pub capabilities: Vec<String>,
    pub last_error: Option<String>,
    pub last_success: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectorTestResult {
    pub provider: String,
    pub auth_ok: bool,
    pub webhook_verify_ok: bool,
    pub sandbox_payment_ok: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestConnectorRequest {
    pub provider: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────

/// GET /admin/connectors — list all configured connector statuses
pub async fn list_connector_status(
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let providers = vec!["mock", "stripe", "razorpay", "adyen"];
    let mut statuses = Vec::new();

    for provider_code in providers {
        let available_keys =
            credential_loader::list_credential_keys(&state.db, provider_code).await?;
        let required = connectors::required_secrets(provider_code);
        let score = connectors::readiness_score(provider_code, &available_keys);

        let connector = create_connector(provider_code)?;
        let caps: Vec<String> = connector
            .capabilities()
            .iter()
            .map(|c| c.to_string())
            .collect();

        // Fetch last health log from DB
        let last_error: Option<String> = sqlx::query_scalar(
            "SELECT error_message FROM provider_health_logs WHERE provider_name = $1 AND is_success = FALSE ORDER BY created_at DESC LIMIT 1"
        )
        .bind(provider_code)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        let last_success: Option<String> = sqlx::query_scalar(
            "SELECT created_at::text FROM provider_health_logs WHERE provider_name = $1 AND is_success = TRUE ORDER BY created_at DESC LIMIT 1"
        )
        .bind(provider_code)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        statuses.push(ConnectorStatusResponse {
            provider: provider_code.to_string(),
            display_name: connector.name().to_string(),
            readiness_score: score,
            available_keys,
            required_keys: required.iter().map(|s| s.to_string()).collect(),
            capabilities: caps,
            last_error,
            last_success,
        });
    }

    Ok(HttpResponse::Ok().json(statuses))
}

/// POST /admin/connectors/test — sandbox test: auth, webhook verify, sandbox payment
pub async fn test_connector(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<TestConnectorRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let provider_code = &body.provider;

    let connector = create_connector(provider_code)?;

    // Load credentials (may be empty)
    let creds = credential_loader::load_credentials(
        &state.db,
        &state.config.vault_encryption_key,
        provider_code,
    )
    .await
    .unwrap_or_else(|_| ProviderCredentials {
        provider: provider_code.to_string(),
        secrets: std::collections::HashMap::new(),
    });

    // Test 1: Auth / health check
    let health = connector.health_check(&creds).await.unwrap_or_else(|e| {
        connectors::ConnectorHealthCheck {
            authenticated: false,
            latency_ms: 0,
            error: Some(format!("Health check failed: {}", e)),
        }
    });

    // Test 2: Webhook verification (with a dummy payload)
    let webhook_ok = {
        let dummy_headers = std::collections::HashMap::new();
        let dummy_body = b"{}";
        connector
            .verify_and_parse_webhook(&creds, &dummy_headers, dummy_body)
            .is_ok()
    };

    // Test 3: Sandbox payment only if authenticated and in test mode
    let sandbox_ok = if health.authenticated && state.config.is_test() {
        let req = connectors::CreateSessionRequest {
            idempotency_key: format!("test_{}", uuid::Uuid::new_v4()),
            amount: 100,
            currency: "INR".to_string(),
            order_id: format!("sandbox_test_{}", uuid::Uuid::new_v4()),
            description: Some("Sandbox connectivity test".to_string()),
            customer_email: Some("sandbox@feelpay.local".to_string()),
            customer_phone: None,
            return_url: Some("https://feelpay.local/return".to_string()),
            cancel_url: None,
            metadata: None,
        };
        connector.create_session(&creds, &req).await.is_ok()
    } else {
        false
    };

    // Log the test to provider_health_logs (best effort)
    let _ = sqlx::query(
        "INSERT INTO provider_health_logs (provider_id, provider_name, is_success, latency_ms, error_message, created_at) VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())"
    )
    .bind(provider_code)
    .bind(health.authenticated)
    .bind(health.latency_ms as i32)
    .bind(&health.error)
    .execute(&state.db)
    .await;

    // Audit log
    log_audit(
        &state.db,
        admin_id,
        "connector_test",
        "provider_connector",
        None,
        None,
        Some(serde_json::json!({
            "provider": provider_code,
            "auth_ok": health.authenticated,
            "latency_ms": health.latency_ms,
        })),
        None,
    )
    .await?;

    let result = ConnectorTestResult {
        provider: provider_code.to_string(),
        auth_ok: health.authenticated,
        webhook_verify_ok: webhook_ok,
        sandbox_payment_ok: sandbox_ok,
        latency_ms: health.latency_ms,
        error: health.error,
    };

    // Notify if auth failed
    if !result.auth_ok {
        tracing::warn!(
            provider = %provider_code,
            "Provider authentication failed during connector test"
        );
        // Create notification (best effort)
        let _ = sqlx::query(
            "INSERT INTO notifications (id, merchant_id, type, title, message, channel, created_at) VALUES (gen_random_uuid(), gen_random_uuid(), 'provider_auth_failure', $1, $2, 'internal', NOW())"
        )
        .bind(format!("{} Auth Failure", connector.name()))
        .bind(format!("Provider {} failed authentication test. Check credentials.", provider_code))
        .execute(&state.db)
        .await;
    }

    Ok(HttpResponse::Ok().json(result))
}

/// GET /admin/connectors/{provider}/capabilities
pub async fn get_connector_capabilities(
    path: web::Path<String>,
) -> Result<HttpResponse, FeelPayError> {
    let provider_code = path.into_inner();
    let connector = create_connector(&provider_code)?;
    let caps: Vec<String> = connector
        .capabilities()
        .iter()
        .map(|c| c.to_string())
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "provider": provider_code,
        "capabilities": caps,
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/connectors")
            .route("", web::get().to(list_connector_status))
            .route("/test", web::post().to(test_connector))
            .route("/{provider}/capabilities", web::get().to(get_connector_capabilities)),
    );
}

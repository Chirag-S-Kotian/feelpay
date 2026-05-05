//! Per-provider webhook ingress
//!
//! Accepts webhooks from Stripe, Razorpay, and Adyen with provider-specific
//! signature verification and event normalization.

use actix_web::{web, HttpRequest, HttpResponse};
use std::collections::HashMap;

use crate::engine::connectors::{create_connector, credential_loader, redaction::redact_metadata};
use crate::error::FeelPayError;
use crate::state::AppState;

/// Extract relevant headers into a HashMap for connector verification
fn extract_headers(req: &HttpRequest) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for (name, value) in req.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.as_str().to_lowercase(), v.to_string());
        }
    }
    headers
}

/// POST /webhooks/providers/{provider} — unified per-provider webhook handler
async fn handle_provider_webhook(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Bytes,
) -> Result<HttpResponse, FeelPayError> {
    let provider_code = path.into_inner();
    let headers = extract_headers(&req);

    // Create the right connector
    let connector = create_connector(&provider_code)?;

    // Load credentials for signature verification
    let creds = credential_loader::load_credentials(
        &state.db,
        &state.config.vault_encryption_key,
        &provider_code,
    )
    .await
    .map_err(|e| {
        tracing::error!(
            provider = %provider_code,
            "Failed to load credentials for webhook verification: {}",
            e
        );
        FeelPayError::Internal("Credential load error for webhook".into())
    })?;

    // Verify signature and parse event
    let event = connector.verify_and_parse_webhook(&creds, &headers, &body)?;

    tracing::info!(
        provider = %event.provider,
        event_type = %event.event_type,
        provider_ref = ?event.provider_reference,
        status = %event.status,
        "Provider webhook received and verified"
    );

    // Store the normalized event in webhook_events table (best effort)
    let _ = sqlx::query(
        r#"
        INSERT INTO webhook_events (
            id, provider, event_type, provider_event_id, provider_reference,
            normalized_status, amount, currency, raw_payload, created_at
        ) VALUES (
            gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, NOW()
        )
        "#,
    )
    .bind(&event.provider)
    .bind(&event.event_type)
    .bind(&event.provider_event_id)
    .bind(&event.provider_reference)
    .bind(&event.status.to_string())
    .bind(event.amount)
    .bind(&event.currency)
    .bind(redact_metadata(
        &serde_json::from_slice::<serde_json::Value>(&body).unwrap_or_default(),
    ))
    .execute(&state.db)
    .await;

    // Return acknowledgment
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "provider": event.provider,
        "event_type": event.event_type,
        "normalized_status": event.status.to_string(),
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhooks/providers")
            .route("/{provider}", web::post().to(handle_provider_webhook)),
    );
}

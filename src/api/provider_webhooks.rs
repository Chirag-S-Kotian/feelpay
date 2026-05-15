//! Per-provider webhook ingress
//!
//! Accepts webhooks from Stripe, Razorpay, and Adyen with provider-specific
//! signature verification and event normalization.

use actix_web::{web, HttpRequest, HttpResponse};
use std::collections::HashMap;

use crate::api::checkout::update_checkout_from_provider_event;
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

    let provider_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM payment_providers WHERE lower(COALESCE(code, name)) = lower($1) LIMIT 1",
    )
    .bind(&provider_code)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| FeelPayError::Internal("Provider lookup failed for webhook".into()))?;

    // Persist provider callbacks idempotently once the provider is known.
    if let (Some(provider_id), Some(provider_event_id)) =
        (provider_id, event.provider_event_id.as_deref())
    {
        let _ = sqlx::query(
            r#"
        INSERT INTO payment_webhook_events (
            provider_id, event_id, provider_event_id, event_type, raw_payload,
            signature_verification_status, processing_status, processed_at
        ) VALUES ($1, $2, $3, $4, $5, 'verified', 'processed', NOW())
        ON CONFLICT (provider_id, event_id) DO NOTHING
        "#,
        )
        .bind(provider_id)
        .bind(provider_event_id)
        .bind(provider_event_id)
        .bind(&event.event_type)
        .bind(redact_metadata(
            &serde_json::from_slice::<serde_json::Value>(&body).unwrap_or_default(),
        ))
        .execute(&state.db)
        .await;
    }

    let _ = update_checkout_from_provider_event(
        state.get_ref(),
        event.provider_reference.as_deref(),
        &event.status.to_string(),
    )
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

use actix_web::{web, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::FeelPayError;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/webhooks").route("/inbound", web::post().to(inbound_webhook)));
}

/// POST /api/v1/webhooks/inbound — FeelPay internal webhook/event ingress
async fn inbound_webhook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, FeelPayError> {
    let signature = req
        .headers()
        .get("X-FeelPay-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if signature.is_empty() {
        return Err(FeelPayError::Unauthorized(
            "Missing X-FeelPay-Signature header".into(),
        ));
    }

    let provided = hex::decode(signature)
        .map_err(|_| FeelPayError::Unauthorized("Invalid signature encoding".into()))?;
    let mut mac = HmacSha256::new_from_slice(state.config.webhook_signing_secret.as_bytes())
        .map_err(|_| FeelPayError::Internal("Invalid webhook signing secret".into()))?;
    mac.update(&body);
    mac.verify_slice(&provided)
        .map_err(|_| FeelPayError::Unauthorized("Signature mismatch".into()))?;

    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| FeelPayError::BadRequest(format!("Invalid webhook JSON: {}", e)))?;
    let event = payload
        .get("event")
        .or_else(|| payload.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if event.starts_with("dispute.") {
        handle_dispute_event(&state, &payload).await?;
    }

    tracing::info!(event = %event, "FeelPay inbound webhook verified");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "provider": "feelpay",
        "event": event
    })))
}

async fn handle_dispute_event(
    state: &web::Data<AppState>,
    payload: &serde_json::Value,
) -> Result<(), FeelPayError> {
    let event = payload
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let object = payload
        .get("data")
        .or_else(|| payload.get("object"))
        .unwrap_or(payload);

    let dispute_id = object
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let merchant_id = object
        .get("merchant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let payment_order_id = object
        .get("payment_order_id")
        .or_else(|| object.get("payment_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let (Some(merchant_id), Some(payment_order_id)) = (merchant_id, payment_order_id) else {
        tracing::warn!("Dispute webhook missing merchant/payment reference");
        return Ok(());
    };

    let amount = object.get("amount").and_then(|v| v.as_i64()).unwrap_or(100);
    let currency = object
        .get("currency")
        .and_then(|v| v.as_str())
        .unwrap_or("INR")
        .to_string();
    let reason_code = object
        .get("reason_code")
        .or_else(|| object.get("reason"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let reason_message = object
        .get("reason_message")
        .or_else(|| object.get("description"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let mapped_status = match event {
        "dispute.won" => "won",
        "dispute.lost" => "lost",
        "dispute.accepted" => "accepted",
        "dispute.under_review" => "under_review",
        _ => "needs_response",
    };

    sqlx::query(
        r#"
        INSERT INTO disputes
            (id, merchant_id, payment_order_id, amount, currency, status, reason_code, reason_message, connector_reference, updated_at, resolved_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), CASE WHEN $6 IN ('won','lost','accepted') THEN NOW() ELSE NULL END)
        ON CONFLICT (id)
        DO UPDATE SET
            status = EXCLUDED.status,
            reason_code = EXCLUDED.reason_code,
            reason_message = EXCLUDED.reason_message,
            updated_at = NOW(),
            resolved_at = CASE WHEN EXCLUDED.status IN ('won','lost','accepted') THEN NOW() ELSE disputes.resolved_at END
        "#,
    )
    .bind(
        uuid::Uuid::parse_str(dispute_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
    )
    .bind(merchant_id)
    .bind(payment_order_id)
    .bind(amount)
    .bind(currency)
    .bind(mapped_status)
    .bind(reason_code)
    .bind(reason_message)
    .bind(Some(dispute_id.to_string()))
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(())
}

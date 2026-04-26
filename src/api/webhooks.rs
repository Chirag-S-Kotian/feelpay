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
        .map_err(|_| FeelPayError::InternalError("Invalid webhook signing secret".into()))?;
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

    tracing::info!(event = %event, "FeelPay inbound webhook verified");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "received",
        "provider": "feelpay",
        "event": event
    })))
}

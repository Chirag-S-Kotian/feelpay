use actix_web::{http::header, web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use redis::Commands;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::engine::processor::{MethodData, TransactionContext};
use crate::error::FeelPayError;
use crate::middleware::auth::MerchantId;
use crate::models::payment_order::PaymentStatus;
use crate::state::AppState;

lazy_static! {
    static ref CHECKOUT_SESSION_FALLBACK: Mutex<HashMap<String, (serde_json::Value, DateTime<Utc>)>> =
        Mutex::new(HashMap::new());
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/checkout")
            .route("/session", web::post().to(create_checkout_session))
            .route("/session/{id}", web::get().to(get_checkout_session))
            .route("/session/{id}/pay", web::post().to(pay_checkout_session))
            .route("/session/{id}/status", web::get().to(get_payment_status))
            .route("/embed.js", web::get().to(get_embed_script)),
    );
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutSessionRequest {
    pub merchant_id: Uuid,
    pub order_id: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_url: Option<String>,
    pub webhook_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct CheckoutSessionResponse {
    pub session_id: String,
    pub checkout_url: String,
    pub embed_url: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PayCheckoutSessionRequest {
    pub method_data: MethodData,
    pub processor_override: Option<String>,
    pub simulation_action: Option<String>,
    pub selected_payment_method: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PayCheckoutSessionResponse {
    pub session_id: String,
    pub payment_id: Uuid,
    pub status: String,
    pub payment_status: String,
    pub connector: Option<String>,
    pub payment_method: Option<String>,
    pub redirect_url: Option<String>,
    pub mode_badge: String,
}

fn checkout_method_name(method_data: &MethodData) -> &'static str {
    match method_data {
        MethodData::Card { .. } => "card",
        MethodData::Upi { .. } => "upi",
        MethodData::NetBanking { .. } => "netbanking",
        MethodData::Wallet { .. } => "wallet",
        MethodData::Emi { .. } => "emi",
        MethodData::Bnpl { .. } => "bnpl",
        MethodData::Simulator { .. } => "simulator",
    }
}

fn checkout_connector_for_method(
    method_data: &MethodData,
    processor_override: Option<&str>,
    is_live: bool,
) -> String {
    if let Some(connector) = processor_override.filter(|value| !value.trim().is_empty()) {
        return connector.to_string();
    }
    if !is_live {
        return "simulator".to_string();
    }
    checkout_method_name(method_data).to_string()
}

fn checkout_session_status(payment_status: &PaymentStatus) -> &'static str {
    match payment_status {
        PaymentStatus::Captured | PaymentStatus::Authorized => "paid",
        PaymentStatus::Failed => "failed",
        _ => "processing",
    }
}

fn store_checkout_session_fallback(
    session_id: &str,
    session: serde_json::Value,
    expires_at: DateTime<Utc>,
) {
    if let Ok(mut sessions) = CHECKOUT_SESSION_FALLBACK.lock() {
        sessions.insert(session_id.to_string(), (session, expires_at));
    }
}

fn get_checkout_session_fallback(session_id: &str) -> Option<serde_json::Value> {
    let mut sessions = CHECKOUT_SESSION_FALLBACK.lock().ok()?;
    let (session, expires_at) = sessions.get(session_id)?.clone();
    if expires_at <= Utc::now() {
        sessions.remove(session_id);
        return None;
    }
    Some(session)
}

fn default_preview_methods() -> Vec<serde_json::Value> {
    ["card", "upi", "netbanking", "wallet", "emi", "bnpl"]
        .iter()
        .enumerate()
        .map(|(index, method)| {
            serde_json::json!({
                "method_type": method,
                "enabled": true,
                "config": {
                    "priority": index + 1,
                    "display_name": match *method {
                        "card" => "Cards",
                        "upi" => "UPI",
                        "netbanking" => "Netbanking",
                        "wallet" => "Wallets",
                        "emi" => "EMI",
                        "bnpl" => "Pay Later",
                        _ => method,
                    },
                    "checkout_note": "Available on FeelPay checkout."
                }
            })
        })
        .collect()
}

pub async fn get_checkout_preview_config(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = req
        .extensions()
        .get::<MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))?;

    let cache_key = format!("merchant:{}:checkout_preview", merchant_id);
    if let Ok(mut conn) = state.redis.get_connection() {
        let cached: redis::RedisResult<String> = conn.get(&cache_key);
        if let Ok(value) = cached {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value) {
                return Ok(HttpResponse::Ok()
                    .insert_header((header::CACHE_CONTROL, "private, max-age=20"))
                    .json(json));
            }
        }
    }

    let merchant: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT name, business_name FROM merchants WHERE id = $1",
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    let methods: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT jsonb_build_object(
            'method_type', method_type,
            'enabled', enabled,
            'config', COALESCE(config, '{}'::jsonb)
        )
        FROM merchant_payment_methods
        WHERE merchant_id = $1
        ORDER BY COALESCE((config->>'priority')::int, 99), method_type
        "#,
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let (merchant_name, business_name) = merchant.unwrap_or_else(|| ("FeelPay Merchant".into(), None));
    let payload = serde_json::json!({
        "merchant": {
            "id": merchant_id,
            "name": merchant_name,
            "business_name": business_name
        },
        "sample_order": {
            "order_id": "order_preview_001",
            "amount": 249900,
            "currency": "INR",
            "customer_email": "customer@example.com",
            "description": "Checkout preview order"
        },
        "methods": if methods.is_empty() { default_preview_methods() } else { methods },
        "event_flow": [
            { "label": "Order created", "status": "created", "detail": "Merchant creates order/session from API or dashboard." },
            { "label": "Customer selects method", "status": "customer_action", "detail": "Checkout shows enabled methods in merchant priority order." },
            { "label": "Authorization + capture", "status": "processing", "detail": "FeelPay routes payment and updates status." },
            { "label": "Signed webhook", "status": "webhook", "detail": "Merchant receives signed payment event." },
            { "label": "Settlement", "status": "settlement", "detail": "Captured funds move into settlement workflow." }
        ]
    });

    if let Ok(mut conn) = state.redis.get_connection() {
        let _: redis::RedisResult<()> = conn.set_ex(
            cache_key,
            serde_json::to_string(&payload).unwrap_or_default(),
            20,
        );
    }

    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "private, max-age=20"))
        .json(payload))
}

/// Create a hosted checkout session
pub async fn create_checkout_session(
    state: web::Data<AppState>,
    body: web::Json<CreateCheckoutSessionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let session_id = format!("cs_{}", Uuid::new_v4().simple());
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);

    // Store session in Redis or DB
    let session_data = serde_json::json!({
        "session_id": &session_id,
        "merchant_id": body.merchant_id,
        "order_id": &body.order_id,
        "amount": body.amount,
        "currency": body.currency.as_deref().unwrap_or("INR"),
        "customer_email": &body.customer_email,
        "customer_phone": &body.customer_phone,
        "return_url": &body.return_url,
        "webhook_url": &body.webhook_url,
        "metadata": &body.metadata,
        "status": "created",
        "expires_at": expires_at.to_rfc3339(),
    });

    // Store in Redis with 30 min expiry
    let redis_key = format!("checkout:{}", session_id);
    if let Ok(mut conn) = state.redis.get_connection() {
        let _: Result<(), _> = redis::cmd("SETEX")
            .arg(&redis_key)
            .arg(1800) // 30 minutes
            .arg(session_data.to_string())
            .query(&mut conn);
    } else {
        store_checkout_session_fallback(&session_id, session_data.clone(), expires_at);
    }

    let tenant_row: Option<(Option<String>, Option<serde_json::Value>, Option<serde_json::Value>)> =
        sqlx::query_as(
            r#"
            SELECT t.domain, t.branding, t.config
            FROM merchants m
            LEFT JOIN tenants t ON t.id = m.tenant_id
            WHERE m.id = $1
            "#,
        )
        .bind(body.merchant_id)
        .fetch_optional(&state.db)
        .await?;

    let default_base = state
        .config
        .cors_allowed_origins
        .first()
        .cloned()
        .unwrap_or_else(|| "https://checkout.feelpay.com".to_string());

    let tenant_domain = tenant_row
        .as_ref()
        .and_then(|(domain, _, config)| {
            domain.clone().or_else(|| {
                config
                    .as_ref()
                    .and_then(|c| c.get("custom_domain"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
        })
        .map(|d| {
            if d.starts_with("http://") || d.starts_with("https://") {
                d
            } else {
                format!("https://{}", d)
            }
        })
        .unwrap_or(default_base);

    let checkout_url = format!("{}/checkout/{}", tenant_domain, session_id);
    let embed_url = format!("{}/checkout/embed/{}", tenant_domain, session_id);

    if let Some((_, branding, config)) = tenant_row {
        let tenant_branding = branding.unwrap_or_else(|| serde_json::json!({}));
        let tenant_config = config.unwrap_or_else(|| serde_json::json!({}));
        if let Ok(mut conn) = state.redis.get_connection() {
            let style_key = format!("checkout:theme:{}", session_id);
            let style_payload = serde_json::json!({
                "session_id": session_id,
                "branding": tenant_branding,
                "config": tenant_config,
            });
            let _: Result<(), _> = redis::cmd("SETEX")
                .arg(style_key)
                .arg(1800)
                .arg(style_payload.to_string())
                .query(&mut conn);
        }
    }

    Ok(HttpResponse::Created().json(CheckoutSessionResponse {
        session_id: session_id.clone(),
        checkout_url,
        embed_url,
        expires_at,
    }))
}

/// Get checkout session details
pub async fn get_checkout_session(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, FeelPayError> {
    let session_id = path.into_inner();
    let redis_key = format!("checkout:{}", session_id);

    if let Ok(mut conn) = state.redis.get_connection() {
        let result: Result<String, _> = redis::cmd("GET").arg(&redis_key).query(&mut conn);

        if let Ok(data) = result {
            let session: serde_json::Value =
                serde_json::from_str(&data).map_err(|e| FeelPayError::Internal(e.to_string()))?;
            return Ok(HttpResponse::Ok().json(session));
        }
    }

    if let Some(session) = get_checkout_session_fallback(&session_id) {
        return Ok(HttpResponse::Ok().json(session));
    }

    Err(FeelPayError::NotFound(
        "Checkout session not found or expired".into(),
    ))
}

/// Process payment for a hosted checkout session
pub async fn pay_checkout_session(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<PayCheckoutSessionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let session_id = path.into_inner();
    let redis_key = format!("checkout:{}", session_id);

    let (mut session, mut redis_conn) = if let Ok(mut conn) = state.redis.get_connection() {
        let data: String = redis::cmd("GET")
            .arg(&redis_key)
            .query(&mut conn)
            .map_err(|_| FeelPayError::NotFound("Checkout session not found".into()))?;
        let session: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| FeelPayError::Internal(e.to_string()))?;
        (session, Some(conn))
    } else {
        (
            get_checkout_session_fallback(&session_id)
                .ok_or_else(|| FeelPayError::NotFound("Checkout session not found".into()))?,
            None,
        )
    };

    let merchant_id = session["merchant_id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| FeelPayError::BadRequest("Invalid checkout merchant".into()))?;
    let amount = session["amount"]
        .as_i64()
        .ok_or_else(|| FeelPayError::BadRequest("Invalid checkout amount".into()))?;
    if amount <= 0 {
        return Err(FeelPayError::BadRequest("Invalid checkout amount".into()));
    }

    let payment_id = Uuid::new_v4();
    let method = body
        .selected_payment_method
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| checkout_method_name(&body.method_data));
    let connector = checkout_connector_for_method(
        &body.method_data,
        body.processor_override.as_deref(),
        state.config.is_live(),
    );
    let now = chrono::Utc::now();
    let txn_ctx = TransactionContext {
        payment_id,
        merchant_id,
        order_id: session["order_id"]
            .as_str()
            .unwrap_or("checkout_order")
            .to_string(),
        amount,
        currency: session["currency"].as_str().unwrap_or("INR").to_string(),
        customer_email: session["customer_email"].as_str().map(ToString::to_string),
        customer_phone: session["customer_phone"].as_str().map(ToString::to_string),
        return_url: session["return_url"].as_str().map(ToString::to_string),
        description: Some("Hosted checkout payment".into()),
    };

    let engine = state.payment_engine.read().await;
    let processor_method_data = if connector == "simulator" {
        body.simulation_action
            .as_ref()
            .map(|action| MethodData::Simulator {
                action: action.clone(),
            })
            .unwrap_or_else(|| body.method_data.clone())
    } else {
        body.method_data.clone()
    };
    let response = engine
        .authorize(&txn_ctx, &processor_method_data, Some(&connector))
        .await?;
    let payment_status = response.status.clone();
    let checkout_status = checkout_session_status(&payment_status);
    let authorized_at = if matches!(payment_status, PaymentStatus::Authorized | PaymentStatus::Captured) {
        Some(now)
    } else {
        None
    };
    let captured_at = if payment_status == PaymentStatus::Captured {
        Some(now)
    } else {
        None
    };
    let gateway_reference = response.gateway_reference;
    let gateway_response = response.gateway_response;
    let error_message = response.error_message;
    let error_code = response.error_code;
    let redirect_url = response.redirect_url;
    let mode_badge = if state.config.is_live() { "LIVE" } else { "TEST" };

    sqlx::query(
        r#"
        INSERT INTO payment_orders (
            id, merchant_id, amount, currency, status, order_id,
            payment_method, payment_method_data, connector, customer_email, customer_phone,
            description, return_url, gateway_reference, gateway_response, error_message,
            error_code, metadata, created_at, updated_at, authorized_at, captured_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .bind(amount)
    .bind(txn_ctx.currency.as_str())
    .bind(payment_status.as_str())
    .bind(txn_ctx.order_id.as_str())
    .bind(method)
    .bind(serde_json::to_value(&body.method_data).unwrap_or_else(|_| serde_json::json!({})))
    .bind(&connector)
    .bind(txn_ctx.customer_email.as_deref())
    .bind(txn_ctx.customer_phone.as_deref())
    .bind(txn_ctx.description.as_deref())
    .bind(txn_ctx.return_url.as_deref())
    .bind(&gateway_reference)
    .bind(&gateway_response)
    .bind(&error_message)
    .bind(&error_code)
    .bind(serde_json::json!({
        "environment": if state.config.is_live() { "live" } else { "test" },
        "mode_badge": mode_badge,
        "checkout_session_id": session_id,
    }))
    .bind(now)
    .bind(now)
    .bind(authorized_at)
    .bind(captured_at)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    session["status"] = serde_json::json!(checkout_status);
    session["payment_id"] = serde_json::json!(payment_id);
    session["payment_status"] = serde_json::json!(payment_status.as_str());
    session["connector"] = serde_json::json!(connector);
    session["payment_method"] = serde_json::json!(method);
    if let Some(conn) = redis_conn.as_mut() {
        let _: redis::RedisResult<()> = redis::cmd("SETEX")
            .arg(&redis_key)
            .arg(1800)
            .arg(session.to_string())
            .query(conn);
    } else {
        let expires_at = session["expires_at"]
            .as_str()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() + chrono::Duration::minutes(30));
        store_checkout_session_fallback(&session_id, session.clone(), expires_at);
    }

    Ok(HttpResponse::Ok().json(PayCheckoutSessionResponse {
        session_id,
        payment_id,
        status: checkout_status.to_string(),
        payment_status: payment_status.as_str().to_string(),
        connector: Some(connector),
        payment_method: Some(method.to_string()),
        redirect_url,
        mode_badge: mode_badge.to_string(),
    }))
}

/// Get payment status (for polling)
pub async fn get_payment_status(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, FeelPayError> {
    let session_id = path.into_inner();
    let redis_key = format!("checkout:{}", session_id);

    if let Ok(mut conn) = state.redis.get_connection() {
        let result: Result<String, _> = redis::cmd("GET").arg(&redis_key).query(&mut conn);

        if let Ok(data) = result {
            let session: serde_json::Value =
                serde_json::from_str(&data).map_err(|e| FeelPayError::Internal(e.to_string()))?;

            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "session_id": session_id,
                "status": session["status"],
                "payment_id": session.get("payment_id"),
                "payment_status": session.get("payment_status"),
            })));
        }
    }

    if let Some(session) = get_checkout_session_fallback(&session_id) {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "session_id": session_id,
            "status": session["status"],
            "payment_id": session.get("payment_id"),
            "payment_status": session.get("payment_status"),
        })));
    }

    Err(FeelPayError::NotFound("Checkout session not found".into()))
}

/// Get embedded checkout JavaScript
pub async fn get_embed_script() -> Result<HttpResponse, FeelPayError> {
    let script = r#"
(function() {
    if (window.FeelPayCheckout) return;

    const TRUST_CSS = `
      .feelpay-checkout-backdrop {
        position: fixed;
        inset: 0;
        z-index: 2147483647;
        background: rgba(20, 20, 19, 0.42);
        display: grid;
        place-items: center;
        padding: max(16px, env(safe-area-inset-top)) 16px max(16px, env(safe-area-inset-bottom));
        box-sizing: border-box;
      }
      .feelpay-checkout-frame {
        width: min(100%, 440px);
        height: min(760px, 100dvh - 32px);
        border: 0;
        border-radius: 34px;
        background: #fff9ec;
        box-shadow: 0 24px 80px rgba(20, 20, 19, 0.28);
      }
      .feelpay-checkout-close {
        position: fixed;
        top: max(14px, env(safe-area-inset-top));
        right: 16px;
        width: 44px;
        height: 44px;
        border: 1px solid #d5c9af;
        border-radius: 999px;
        background: #fff9ec;
        color: #141413;
        font: 700 18px system-ui, sans-serif;
        cursor: pointer;
      }
      @media (min-width: 900px) {
        .feelpay-checkout-frame {
          width: min(920px, 94vw);
          height: min(720px, 86vh);
        }
      }
    `;
    
    window.FeelPayCheckout = {
        init: function(options) {
            this.merchantId = options.merchantId;
            this.container = document.getElementById(options.containerId);
            this.checkoutBaseUrl = options.checkoutBaseUrl || 'https://checkout.feelpay.com';
            this.onSuccess = options.onSuccess || function(){};
            this.onError = options.onError || function(){};
            this.onClose = options.onClose || function(){};
        },
        
        open: function(paymentData) {
            this.close();
            if (!document.getElementById('feelpay-checkout-style')) {
                const style = document.createElement('style');
                style.id = 'feelpay-checkout-style';
                style.textContent = TRUST_CSS;
                document.head.appendChild(style);
            }
            const backdrop = document.createElement('div');
            backdrop.className = 'feelpay-checkout-backdrop';
            backdrop.id = 'feelpay-checkout-backdrop';
            backdrop.setAttribute('role', 'dialog');
            backdrop.setAttribute('aria-modal', 'true');
            backdrop.setAttribute('aria-label', 'FeelPay secure checkout');

            const closeButton = document.createElement('button');
            closeButton.className = 'feelpay-checkout-close';
            closeButton.type = 'button';
            closeButton.setAttribute('aria-label', 'Close FeelPay checkout');
            closeButton.textContent = '×';
            closeButton.onclick = () => {
                this.close();
                this.onClose();
            };

            const iframe = document.createElement('iframe');
            iframe.src = this.checkoutBaseUrl.replace(/\/$/, '') + '/embed/' + encodeURIComponent(paymentData.sessionId);
            iframe.className = 'feelpay-checkout-frame';
            iframe.id = 'feelpay-checkout-iframe';
            iframe.title = 'FeelPay secure checkout';
            iframe.allow = 'payment *';
            backdrop.appendChild(iframe);
            backdrop.appendChild(closeButton);
            document.body.appendChild(backdrop);
            
            window.addEventListener('message', (event) => {
                if (event.data.type === 'FEELPAY_SUCCESS') {
                    this.onSuccess(event.data);
                    this.close();
                } else if (event.data.type === 'FEELPAY_ERROR') {
                    this.onError(event.data);
                } else if (event.data.type === 'FEELPAY_CLOSE') {
                    this.close();
                    this.onClose();
                }
            });
        },
        
        close: function() {
            const backdrop = document.getElementById('feelpay-checkout-backdrop');
            if (backdrop) backdrop.remove();
        }
    };
})();
"#;

    Ok(HttpResponse::Ok()
        .content_type("application/javascript")
        .body(script))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkout_connector_uses_simulator_for_test_mode() {
        let method_data = MethodData::Upi {
            vpa: Some("buyer@upi".into()),
            flow: Some("collect".into()),
        };
        assert_eq!(
            checkout_connector_for_method(&method_data, None, false),
            "simulator"
        );
        assert_eq!(checkout_connector_for_method(&method_data, None, true), "upi");
        assert_eq!(
            checkout_connector_for_method(&method_data, Some("card"), false),
            "card"
        );
    }

    #[test]
    fn checkout_status_tracks_payment_status() {
        assert_eq!(checkout_session_status(&PaymentStatus::Captured), "paid");
        assert_eq!(checkout_session_status(&PaymentStatus::Authorized), "paid");
        assert_eq!(checkout_session_status(&PaymentStatus::Failed), "failed");
        assert_eq!(checkout_session_status(&PaymentStatus::Processing), "processing");
    }
}

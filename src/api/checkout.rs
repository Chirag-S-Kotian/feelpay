use actix_web::{http::header, web, HttpMessage, HttpRequest, HttpResponse};
use redis::Commands;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::middleware::auth::MerchantId;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/checkout")
            .route("/session", web::post().to(create_checkout_session))
            .route("/session/{id}", web::get().to(get_checkout_session))
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

    Err(FeelPayError::NotFound(
        "Checkout session not found or expired".into(),
    ))
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

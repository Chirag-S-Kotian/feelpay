use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::FeelPayError;
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
    
    window.FeelPayCheckout = {
        init: function(options) {
            this.merchantId = options.merchantId;
            this.container = document.getElementById(options.containerId);
            this.onSuccess = options.onSuccess || function(){};
            this.onError = options.onError || function(){};
            this.onClose = options.onClose || function(){};
        },
        
        open: function(paymentData) {
            const iframe = document.createElement('iframe');
            iframe.src = 'https://checkout.feelpay.com/embed/' + paymentData.sessionId;
            iframe.style.cssText = 'width:100%;height:100%;border:none;position:fixed;top:0;left:0;z-index:9999;background:transparent;';
            iframe.id = 'feelpay-checkout-iframe';
            document.body.appendChild(iframe);
            
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
            const iframe = document.getElementById('feelpay-checkout-iframe');
            if (iframe) iframe.remove();
        }
    };
})();
"#;

    Ok(HttpResponse::Ok()
        .content_type("application/javascript")
        .body(script))
}

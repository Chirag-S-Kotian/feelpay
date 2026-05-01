use actix_web::{http::header, web, HttpResponse};
use redis::Commands;

use crate::error::FeelPayError;
use crate::state::AppState;

const LANDING_CACHE_KEY: &str = "public:landing_page";

fn default_landing_config() -> serde_json::Value {
    serde_json::json!({
        "brand": {
            "name": "FeelPay",
            "tagline": "Payments that build trust"
        },
        "hero": {
            "kicker": "Payments that build trust",
            "headline": "Accept, route, settle, and pay out from one merchant platform.",
            "body": "FeelPay gives Indian businesses a full payment stack: checkout, links, payment methods, payouts, settlement reports, disputes, and developer observability.",
            "primary_cta": "Start testing",
            "secondary_cta": "Merchant login"
        },
        "metrics": [
            { "label": "Payment success", "value": "Admin configurable" },
            { "label": "Settlement cycle", "value": "T+2" },
            { "label": "Webhook health", "value": "Signed" }
        ],
        "features": [
            { "title": "Checkout", "text": "Responsive hosted flow with payment option selection." },
            { "title": "Payouts", "text": "Move funds to verified accounts with clear status trails." },
            { "title": "Developers", "text": "API keys, webhook testing, logs, and SDK-ready flows." },
            { "title": "Trust", "text": "Live-mode gates, signed webhooks, and 24-hour sessions." }
        ],
        "pricing": [
            {
                "name": "Starter",
                "price": "Configure in admin",
                "caption": "Default landing content",
                "features": ["Hosted checkout", "Payment links", "UPI, cards, wallets", "Settlement reports"]
            },
            {
                "name": "Growth",
                "price": "Configure in admin",
                "caption": "Default landing content",
                "featured": true,
                "features": ["Smart routing", "Payouts", "Webhook retries", "Disputes workspace"]
            },
            {
                "name": "Scale",
                "price": "Custom",
                "caption": "For platforms and marketplaces",
                "features": ["Volume pricing", "Dedicated support", "Custom risk rules", "Advanced reconciliation"]
            }
        ]
    })
}

pub async fn get_landing_page_config(
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    if let Ok(mut conn) = state.redis.get_connection() {
        let cached: redis::RedisResult<String> = conn.get(LANDING_CACHE_KEY);
        if let Ok(value) = cached {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value) {
                return Ok(HttpResponse::Ok()
                    .insert_header((header::CACHE_CONTROL, "public, max-age=60"))
                    .json(json));
            }
        }
    }

    let config_result = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM system_config WHERE key = 'landing_page'",
    )
    .fetch_optional(&state.db)
    .await;

    let config = match config_result {
        Ok(Some(value)) => value,
        Ok(None) => default_landing_config(),
        Err(sqlx::Error::Database(db_error)) if db_error.code().as_deref() == Some("42P01") => {
            default_landing_config()
        }
        Err(error) => return Err(FeelPayError::from(error)),
    };

    if let Ok(mut conn) = state.redis.get_connection() {
        let _: redis::RedisResult<()> = conn.set_ex(
            LANDING_CACHE_KEY,
            serde_json::to_string(&config).unwrap_or_default(),
            60,
        );
    }

    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "public, max-age=60"))
        .json(config))
}

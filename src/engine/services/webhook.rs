use crate::engine::services::notification_dispatcher::{dispatch_event, DispatchRequest};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: WebhookStatus,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum WebhookStatus {
    Pending,
    Delivered,
    Failed,
    DeadLetter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
}

/// Queue a webhook event for delivery
pub async fn queue_webhook(
    pool: &PgPool,
    merchant_id: Uuid,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO webhook_deliveries (id, merchant_id, event_type, payload, status, attempt_count, created_at)
        VALUES ($1, $2, $3, $4, 'pending', 0, NOW())
        "#,
    )
    .bind(id)
    .bind(merchant_id)
    .bind(event_type)
    .bind(payload)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Process pending webhooks with exponential backoff retry
pub async fn process_webhook_queue(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Fetch pending webhooks that are ready for retry
    let pending: Vec<WebhookEvent> = sqlx::query_as(
        r#"
                SELECT wd.*
        FROM webhook_deliveries wd
        WHERE wd.status = 'pending'
          AND (wd.next_retry_at IS NULL OR wd.next_retry_at <= NOW())
          AND wd.attempt_count < 7
        ORDER BY wd.created_at ASC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await?;

    for event in pending {
        // Get webhook config for merchant
        let config: Option<(String, String)> = sqlx::query_as(
            "SELECT url, secret FROM webhooks WHERE merchant_id = $1 AND active = true",
        )
        .bind(event.merchant_id)
        .fetch_optional(pool)
        .await?;

        let Some((url, secret)) = config else {
            // No active webhook for merchant, mark as delivered
            sqlx::query(
                "UPDATE webhook_deliveries SET status = 'delivered', delivered_at = NOW() WHERE id = $1"
            )
            .bind(event.id)
            .execute(pool)
            .await?;
            continue;
        };

        // Sign payload
        let payload_bytes = serde_json::to_vec(&event.payload)?;
        let signature = sign_webhook(&secret, &payload_bytes);

        // Send webhook
        let result = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-FeelPay-Signature", signature)
            .header("X-FeelPay-Event", &event.event_type)
            .header("X-FeelPay-Delivery", event.id.to_string())
            .body(payload_bytes)
            .send()
            .await;

        match result {
            Ok(response) if response.status().is_success() => {
                // Mark as delivered
                sqlx::query(
                    "UPDATE webhook_deliveries SET status = 'delivered', delivered_at = NOW() WHERE id = $1"
                )
                .bind(event.id)
                .execute(pool)
                .await?;

                tracing::info!(
                    webhook_id = %event.id,
                    event_type = %event.event_type,
                    merchant_id = %event.merchant_id,
                    "Webhook delivered successfully"
                );
            }
            Ok(response) => {
                // HTTP error, schedule retry
                let error = format!("HTTP {}", response.status());
                handle_retry(pool, &event, &error).await?;
            }
            Err(e) => {
                // Network error, schedule retry
                handle_retry(pool, &event, &e.to_string()).await?;
            }
        }
    }

    Ok(())
}

/// Handle retry with exponential backoff
async fn handle_retry(pool: &PgPool, event: &WebhookEvent, error: &str) -> Result<(), sqlx::Error> {
    let new_attempt = event.attempt_count + 1;

    // Exponential backoff: 1m, 5m, 15m, 1h, 6h, 24h
    let backoff_minutes = match new_attempt {
        1 => 1,
        2 => 5,
        3 => 15,
        4 => 60,
        5 => 360,
        6 => 1440,
        _ => 1440,
    };

    let next_retry = Utc::now() + chrono::Duration::minutes(backoff_minutes);

    if new_attempt >= 7 {
        // Move to dead letter queue
        sqlx::query(
            r#"
            UPDATE webhook_deliveries 
            SET status = 'dead_letter', 
                attempt_count = $1, 
                last_error = $2,
                next_retry_at = NULL
            WHERE id = $3
            "#,
        )
        .bind(new_attempt)
        .bind(error)
        .bind(event.id)
        .execute(pool)
        .await?;

        // Alert merchant (create notification)
        sqlx::query(
            r#"
            INSERT INTO notification_queue (merchant_id, type, title, message, data)
            VALUES ($1, 'webhook_failed', 'Webhook Delivery Failed', 
                    'Webhook delivery failed after 7 attempts. Please check your endpoint.', $2)
            "#,
        )
        .bind(event.merchant_id)
        .bind(serde_json::json!({"webhook_id": event.id, "event_type": event.event_type}))
        .execute(pool)
        .await?;

        tracing::warn!(
            webhook_id = %event.id,
            merchant_id = %event.merchant_id,
            "Webhook moved to dead letter queue"
        );

        let merchant_email =
            sqlx::query_scalar::<_, Option<String>>("SELECT email FROM merchants WHERE id = $1")
                .bind(event.merchant_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .flatten();
        let mut variables = std::collections::BTreeMap::new();
        variables.insert("webhook_id".to_string(), event.id.to_string());
        variables.insert("event_type".to_string(), event.event_type.clone());
        let _ = dispatch_event(
            pool,
            DispatchRequest {
                merchant_id: Some(event.merchant_id),
                event_type: "webhook_failure".to_string(),
                environment: std::env::var("APP_ENV")
                    .ok()
                    .filter(|value| value == "live")
                    .unwrap_or_else(|| "test".to_string()),
                recipient_email: merchant_email,
                recipient_phone: None,
                variables,
            },
        )
        .await;
    } else {
        sqlx::query(
            r#"
            UPDATE webhook_deliveries 
            SET attempt_count = $1, 
                last_error = $2,
                next_retry_at = $3
            WHERE id = $4
            "#,
        )
        .bind(new_attempt)
        .bind(error)
        .bind(next_retry)
        .bind(event.id)
        .execute(pool)
        .await?;

        tracing::info!(
            webhook_id = %event.id,
            attempt = new_attempt,
            next_retry = %next_retry,
            "Webhook retry scheduled"
        );
    }

    Ok(())
}

/// Sign webhook payload with HMAC-SHA256
pub fn sign_webhook(secret: &str, payload: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

/// Verify webhook signature
pub fn verify_webhook(secret: &str, payload: &[u8], signature: &str) -> bool {
    let Ok(signature_bytes) = hex::decode(signature) else {
        return false;
    };

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload);

    mac.verify_slice(&signature_bytes).is_ok()
}

/// Trigger webhook for payment event
pub async fn trigger_payment_webhook(
    pool: &PgPool,
    merchant_id: Uuid,
    event_type: &str,
    payment_id: Uuid,
    order_id: &str,
    amount: i64,
    currency: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::json!({
        "event": event_type,
        "created_at": Utc::now().to_rfc3339(),
        "data": {
            "payment_id": payment_id,
            "order_id": order_id,
            "amount": amount,
            "currency": currency,
            "status": status,
        }
    });

    queue_webhook(pool, merchant_id, event_type, payload).await?;
    Ok(())
}

/// Trigger webhook for refund event
pub async fn trigger_refund_webhook(
    pool: &PgPool,
    merchant_id: Uuid,
    event_type: &str,
    refund_id: Uuid,
    payment_id: Uuid,
    amount: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::json!({
        "event": event_type,
        "created_at": Utc::now().to_rfc3339(),
        "data": {
            "refund_id": refund_id,
            "payment_id": payment_id,
            "amount": amount,
            "status": status,
        }
    });

    queue_webhook(pool, merchant_id, event_type, payload).await?;
    Ok(())
}

/// Trigger webhook for dispute event
pub async fn trigger_dispute_webhook(
    pool: &PgPool,
    merchant_id: Uuid,
    dispute_id: Uuid,
    payment_id: Uuid,
    amount: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::json!({
        "event": "dispute.created",
        "created_at": Utc::now().to_rfc3339(),
        "data": {
            "dispute_id": dispute_id,
            "payment_id": payment_id,
            "amount": amount,
            "status": status,
        }
    });

    queue_webhook(pool, merchant_id, "dispute.created", payload).await?;
    Ok(())
}

/// Trigger webhook for settlement event
pub async fn trigger_settlement_webhook(
    pool: &PgPool,
    merchant_id: Uuid,
    settlement_id: Uuid,
    amount: i64,
    utr: Option<&str>,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::json!({
        "event": "settlement.processed",
        "created_at": Utc::now().to_rfc3339(),
        "data": {
            "settlement_id": settlement_id,
            "amount": amount,
            "utr": utr,
        }
    });

    queue_webhook(pool, merchant_id, "settlement.processed", payload).await?;
    Ok(())
}

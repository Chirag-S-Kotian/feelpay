use crate::error::FeelPayError;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::{PgPool, Row};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Webhook test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTestResult {
    pub test_id: Uuid,
    pub merchant_id: Uuid,
    pub event_type: String,
    pub webhook_url: String,
    pub status: String, // "sent", "failed"
    pub http_status: Option<i32>,
    pub response_time_ms: Option<i64>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Supported webhook event types
pub const SUPPORTED_WEBHOOK_EVENTS: &[&str] = &[
    "payment.success",
    "payment.failed",
    "refund.initiated",
    "settlement.processed",
    "dispute.created",
];

/// WebhookTestService handles webhook testing
pub struct WebhookTestService {
    db: PgPool,
    http_client: reqwest::Client,
}

impl WebhookTestService {
    /// Create a new WebhookTestService instance
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
        }
    }

    /// Send a test webhook to a merchant endpoint
    pub async fn send_test_webhook(
        &self,
        merchant_id: Uuid,
        webhook_url: &str,
        event_type: &str,
        webhook_secret: &str,
    ) -> Result<WebhookTestResult, FeelPayError> {
        // Validate event type
        if !SUPPORTED_WEBHOOK_EVENTS.contains(&event_type) {
            return Err(FeelPayError::BadRequest(format!(
                "Unsupported webhook event type: {}",
                event_type
            )));
        }

        // Generate sample payload
        let payload = self.generate_sample_payload(event_type, merchant_id);
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| FeelPayError::Internal(format!("Failed to serialize payload: {}", e)))?;

        self.send_payload(
            merchant_id,
            webhook_url,
            event_type,
            &payload_json,
            webhook_secret,
        )
        .await
    }

    /// Replay an existing webhook event
    pub async fn replay_event(
        &self,
        merchant_id: Uuid,
        event_id: Uuid,
    ) -> Result<WebhookTestResult, FeelPayError> {
        // Fetch event details and verify ownership
        // We look in both real events and test logs for convenience in sandbox
        let row = sqlx::query(
            r#"
            SELECT event_type, raw_payload::text, webhook_url, webhook_secret
            FROM (
                SELECT pwe.event_type, pwe.raw_payload, m.webhook_url, m.webhook_secret, pwe.id, po.merchant_id
                FROM payment_webhook_events pwe
                JOIN payment_orders po ON pwe.related_payment_id = po.id
                JOIN merchants m ON po.merchant_id = m.id
                UNION ALL
                SELECT wtl.event_type, wtl.response_body::jsonb, wtl.webhook_url, m.webhook_secret, wtl.id, wtl.merchant_id
                FROM webhook_test_logs wtl
                JOIN merchants m ON wtl.merchant_id = m.id
            ) combined
            WHERE id = $1 AND merchant_id = $2
            LIMIT 1
            "#
        )
        .bind(event_id)
        .bind(merchant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

        let Some(row) = row else {
            return Err(FeelPayError::NotFound(
                "Webhook event not found or access denied".into(),
            ));
        };

        let event_type: String = row.get(0);
        let payload_json: String = row.get(1);
        let webhook_url: String = row.get(2);
        let webhook_secret: String = row.get(3);

        self.send_payload(
            merchant_id,
            &webhook_url,
            &event_type,
            &payload_json,
            &webhook_secret,
        )
        .await
    }

    async fn send_payload(
        &self,
        merchant_id: Uuid,
        webhook_url: &str,
        event_type: &str,
        payload_json: &str,
        webhook_secret: &str,
    ) -> Result<WebhookTestResult, FeelPayError> {
        // Sign the webhook
        let signature = self.sign_webhook(payload_json, webhook_secret)?;
        let timestamp = Utc::now().timestamp().to_string();

        // Send the webhook
        let test_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        let result = self
            .http_client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("X-FeelPay-Signature", &signature)
            .header("X-FeelPay-Timestamp", &timestamp)
            .body(payload_json.to_string())
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        let response_time_ms = start_time.elapsed().as_millis() as i64;

        let (status, http_status, response_body, error_message) = match result {
            Ok(response) => {
                let status_code = response.status().as_u16() as i32;
                let body = response.text().await.unwrap_or_else(|_| "".to_string());

                if status_code >= 200 && status_code < 300 {
                    ("sent".to_string(), Some(status_code), Some(body), None)
                } else {
                    (
                        "failed".to_string(),
                        Some(status_code),
                        Some(body),
                        Some(format!("HTTP {}", status_code)),
                    )
                }
            }
            Err(e) => {
                let error_msg = if e.is_timeout() {
                    "Request timeout".to_string()
                } else if e.is_connect() {
                    "Connection failed".to_string()
                } else {
                    e.to_string()
                };
                ("failed".to_string(), None, None, Some(error_msg))
            }
        };

        let test_result = WebhookTestResult {
            test_id,
            merchant_id,
            event_type: event_type.to_string(),
            webhook_url: webhook_url.to_string(),
            status,
            http_status,
            response_time_ms: Some(response_time_ms),
            response_body,
            error_message,
            created_at: Utc::now(),
        };

        // Log the test result
        self.log_webhook_test(&test_result).await?;

        Ok(test_result)
    }

    /// Generate sample payload for an event type
    pub fn generate_sample_payload(
        &self,
        event_type: &str,
        merchant_id: Uuid,
    ) -> serde_json::Value {
        match event_type {
            "payment.success" => json!({
                "event": "payment.success",
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {
                    "payment_id": Uuid::new_v4().to_string(),
                    "order_id": "ORDER-123",
                    "amount": 50000,
                    "currency": "INR",
                    "status": "success",
                    "payment_method": "card",
                    "customer_email": "customer@example.com",
                    "created_at": Utc::now().to_rfc3339(),
                }
            }),
            "payment.failed" => json!({
                "event": "payment.failed",
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {
                    "payment_id": Uuid::new_v4().to_string(),
                    "order_id": "ORDER-124",
                    "amount": 50000,
                    "currency": "INR",
                    "status": "failed",
                    "failure_reason": "Insufficient funds",
                    "payment_method": "card",
                    "customer_email": "customer@example.com",
                    "created_at": Utc::now().to_rfc3339(),
                }
            }),
            "refund.initiated" => json!({
                "event": "refund.initiated",
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {
                    "refund_id": Uuid::new_v4().to_string(),
                    "payment_id": Uuid::new_v4().to_string(),
                    "amount": 50000,
                    "currency": "INR",
                    "reason": "Customer request",
                    "status": "initiated",
                    "created_at": Utc::now().to_rfc3339(),
                }
            }),
            "settlement.processed" => json!({
                "event": "settlement.processed",
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {
                    "settlement_id": Uuid::new_v4().to_string(),
                    "settlement_date": Utc::now().format("%Y-%m-%d").to_string(),
                    "gross_amount": 500000,
                    "fees": 5000,
                    "net_amount": 495000,
                    "currency": "INR",
                    "status": "processed",
                    "transfer_date": Utc::now().format("%Y-%m-%d").to_string(),
                    "utr": "UTR123456789",
                }
            }),
            "dispute.created" => json!({
                "event": "dispute.created",
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {
                    "dispute_id": Uuid::new_v4().to_string(),
                    "payment_id": Uuid::new_v4().to_string(),
                    "amount": 50000,
                    "currency": "INR",
                    "reason": "Unauthorized transaction",
                    "status": "created",
                    "created_at": Utc::now().to_rfc3339(),
                }
            }),
            _ => json!({
                "event": event_type,
                "timestamp": Utc::now().to_rfc3339(),
                "merchant_id": merchant_id.to_string(),
                "data": {}
            }),
        }
    }

    /// Sign a webhook payload using HMAC-SHA256
    pub fn sign_webhook(&self, payload: &str, secret: &str) -> Result<String, FeelPayError> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| FeelPayError::Internal("Failed to create HMAC".into()))?;
        mac.update(payload.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// Verify a webhook signature
    pub fn verify_webhook_signature(
        &self,
        payload: &str,
        signature: &str,
        secret: &str,
    ) -> Result<bool, FeelPayError> {
        let expected_signature = self.sign_webhook(payload, secret)?;
        Ok(signature == expected_signature)
    }

    /// Log a webhook test result
    async fn log_webhook_test(&self, result: &WebhookTestResult) -> Result<(), FeelPayError> {
        sqlx::query(
            r#"
            INSERT INTO webhook_test_logs (
                id, merchant_id, event_type, webhook_url, status, http_status,
                response_time_ms, response_body, error_message, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(result.test_id)
        .bind(result.merchant_id)
        .bind(&result.event_type)
        .bind(&result.webhook_url)
        .bind(&result.status)
        .bind(result.http_status)
        .bind(result.response_time_ms)
        .bind(&result.response_body)
        .bind(&result.error_message)
        .bind(result.created_at)
        .execute(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to log webhook test: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(())
    }

    /// Get webhook test logs for a merchant
    pub async fn get_webhook_test_logs(
        &self,
        merchant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WebhookTestResult>, i64), FeelPayError> {
        // Get total count
        let total_result = sqlx::query_as::<sqlx::Postgres, (i64,)>(
            r#"
            SELECT COUNT(*) FROM webhook_test_logs WHERE merchant_id = $1
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            // If table doesn't exist yet, don't 500, just return 0
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("42P01")) {
                    return FeelPayError::Database("Table missing".into()); // Will be caught below
                }
            }
            tracing::error!("Failed to count webhook test logs: {:?}", e);
            FeelPayError::Database(e.to_string())
        });

        // Special handling for missing table
        let total = match total_result {
            Ok((t,)) => t,
            Err(_) => return Ok((vec![], 0)),
        };

        // Get paginated results
        let logs = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                String,
                Option<i32>,
                Option<i64>,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, event_type, webhook_url, status, http_status,
                response_time_ms, response_body, error_message, created_at
            FROM webhook_test_logs
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch webhook test logs: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        let items = logs
            .into_iter()
            .map(|row| {
                let (
                    id,
                    merchant_id,
                    event_type,
                    webhook_url,
                    status,
                    http_status,
                    response_time_ms,
                    response_body,
                    error_message,
                    created_at,
                ) = row;
                WebhookTestResult {
                    test_id: id,
                    merchant_id,
                    event_type,
                    webhook_url,
                    status,
                    http_status,
                    response_time_ms,
                    response_body,
                    error_message,
                    created_at,
                }
            })
            .collect();

        Ok((items, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_webhook_events() {
        assert!(SUPPORTED_WEBHOOK_EVENTS.contains(&"payment.success"));
        assert!(SUPPORTED_WEBHOOK_EVENTS.contains(&"payment.failed"));
        assert!(!SUPPORTED_WEBHOOK_EVENTS.contains(&"invalid.event"));
    }

    #[tokio::test]
    async fn test_sign_webhook() {
        let service = WebhookTestService::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy("postgres://localhost/test")
                .unwrap(),
        );

        let payload = r#"{"event":"payment.success"}"#;
        let secret = "test-secret";

        let signature = service.sign_webhook(payload, secret).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 hex is 64 chars
    }

    #[tokio::test]
    async fn test_verify_webhook_signature() {
        let service = WebhookTestService::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy("postgres://localhost/test")
                .unwrap(),
        );

        let payload = r#"{"event":"payment.success"}"#;
        let secret = "test-secret";

        let signature = service.sign_webhook(payload, secret).unwrap();
        let is_valid = service
            .verify_webhook_signature(payload, &signature, secret)
            .unwrap();
        assert!(is_valid);

        let is_invalid = service
            .verify_webhook_signature(payload, "invalid", secret)
            .unwrap();
        assert!(!is_invalid);
    }
}

use crate::engine::services::webhook_test::WebhookTestService;
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use crate::vault::crypto::VaultCrypto;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Test webhook request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhookRequest {
    pub event_type: String,
    pub webhook_endpoint_id: Option<String>,
    pub webhook_url: Option<String>,
}

/// Test webhook response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhookResponse {
    pub test_id: String,
    pub id: String,
    pub event_type: String,
    pub status: String,
    pub http_status: Option<i32>,
    pub attempts: i32,
    pub signature_verified: bool,
    pub response_time_ms: Option<i64>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub last_attempt_at: String,
}

fn webhook_crypto() -> Result<VaultCrypto, FeelPayError> {
    let key = std::env::var("VAULT_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));
    let key_id = std::env::var("VAULT_KEY_ID").unwrap_or_else(|_| "key_v1".into());
    VaultCrypto::new(&key, key_id)
}

/// Send test webhook endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/webhooks/test",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    request_body = TestWebhookRequest,
    responses(
        (status = 200, description = "Test webhook sent", body = TestWebhookResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn send_test_webhook(
    http_req: HttpRequest,
    merchant_id: web::Path<Uuid>,
    req: web::Json<TestWebhookRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let event_type = req.event_type.trim();
    if event_type.is_empty() {
        return Err(FeelPayError::BadRequest("event_type is required".into()));
    }

    let row = sqlx::query(
        "SELECT webhook_url, webhook_secret, webhook_secret_ciphertext, webhook_secret_nonce FROM merchants WHERE id = $1",
    )
        .bind(merchant_id)
        .fetch_optional(db.get_ref())
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let payload_webhook_url = req
        .webhook_url
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let webhook_url = payload_webhook_url
        .clone()
        .or_else(|| row.get::<Option<String>, _>(0));
    let mut webhook_secret = match (
        row.get::<Option<String>, _>(2),
        row.get::<Option<String>, _>(3),
    ) {
        (Some(ciphertext), Some(nonce)) => Some(webhook_crypto()?.decrypt(&ciphertext, &nonce)?),
        _ => row.get::<Option<String>, _>(1),
    };

    let env = match http_req
        .headers()
        .get("x-feelpay-environment")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("live") => "live",
        _ => "test",
    };
    if let Some(url) = webhook_url.as_ref() {
        let parsed = reqwest::Url::parse(url.trim())
            .map_err(|_| FeelPayError::BadRequest("Webhook URL must be valid".into()))?;
        let host = parsed.host_str().unwrap_or_default();
        let localhost =
            matches!(host, "localhost" | "127.0.0.1" | "::1") || host.ends_with(".localhost");
        if !(env == "test" && localhost && matches!(parsed.scheme(), "http" | "https"))
            && parsed.scheme() != "https"
        {
            return Err(FeelPayError::BadRequest(
                "Webhook URL must be HTTPS in live mode; localhost is allowed only in test mode"
                    .into(),
            ));
        }
    }

    if webhook_url.is_none() {
        return Err(FeelPayError::BadRequest(
            "Webhook URL not configured. Save webhook URL first.".into(),
        ));
    }

    if webhook_secret.is_none() {
        let new_secret = format!("whsec_{}", Uuid::new_v4().simple());
        let hint = format!(
            "{}...{}",
            &new_secret[..6],
            &new_secret[new_secret.len() - 4..]
        );
        let (ciphertext, nonce) = webhook_crypto()?.encrypt(new_secret.clone())?;
        sqlx::query(
            "UPDATE merchants SET webhook_secret = NULL, webhook_secret_ciphertext = $2, webhook_secret_nonce = $3, webhook_secret_hint = $4, updated_at = NOW() WHERE id = $1",
        )
            .bind(merchant_id)
            .bind(ciphertext)
            .bind(nonce)
            .bind(hint)
            .execute(db.get_ref())
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        webhook_secret = Some(new_secret);
    }

    if payload_webhook_url.is_some() {
        sqlx::query("UPDATE merchants SET webhook_url = $2, updated_at = NOW() WHERE id = $1")
            .bind(merchant_id)
            .bind(webhook_url.as_ref())
            .execute(db.get_ref())
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
    }

    let webhook_service = WebhookTestService::new(db.get_ref().clone());
    let result = webhook_service
        .send_test_webhook(
            merchant_id,
            webhook_url.as_deref().unwrap_or_default(),
            event_type,
            webhook_secret.as_deref().unwrap_or_default(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(TestWebhookResponse {
        test_id: result.test_id.to_string(),
        id: result.test_id.to_string(),
        event_type: result.event_type,
        status: result.status,
        http_status: result.http_status,
        attempts: 1,
        signature_verified: true,
        response_time_ms: result.response_time_ms,
        response_body: result.response_body,
        error_message: result.error_message,
        created_at: result.created_at.to_rfc3339(),
        last_attempt_at: result.created_at.to_rfc3339(),
    }))
}

/// Replay webhook endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/webhooks/replay/{event_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("event_id" = Uuid, Path, description = "Event ID to replay")
    ),
    responses(
        (status = 200, description = "Webhook replayed", body = TestWebhookResponse),
        (status = 404, description = "Event not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn replay_webhook(
    merchant_id: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, event_id) = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let webhook_service = WebhookTestService::new(db.get_ref().clone());
    let result = webhook_service.replay_event(merchant_id, event_id).await?;

    Ok(HttpResponse::Ok().json(TestWebhookResponse {
        test_id: result.test_id.to_string(),
        id: result.test_id.to_string(),
        event_type: result.event_type,
        status: result.status,
        http_status: result.http_status,
        attempts: 1,
        signature_verified: true,
        response_time_ms: result.response_time_ms,
        response_body: result.response_body,
        error_message: result.error_message,
        created_at: result.created_at.to_rfc3339(),
        last_attempt_at: result.created_at.to_rfc3339(),
    }))
}

/// Get webhook test logs endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/webhooks/test-logs",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("limit" = Option<i64>, Query, description = "Limit"),
        ("offset" = Option<i64>, Query, description = "Offset")
    ),
    responses(
        (status = 200, description = "Webhook test logs", body = WebhookTestLogsResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_webhook_test_logs(
    merchant_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let webhook_service = WebhookTestService::new(db.get_ref().clone());
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let (logs, total) = webhook_service
        .get_webhook_test_logs(merchant_id, limit, offset)
        .await?;

    let items = logs
        .into_iter()
        .map(|log| TestWebhookResponse {
            test_id: log.test_id.to_string(),
            id: log.test_id.to_string(),
            event_type: log.event_type,
            status: log.status,
            http_status: log.http_status,
            attempts: 1,
            signature_verified: true,
            response_time_ms: log.response_time_ms,
            response_body: log.response_body,
            error_message: log.error_message,
            created_at: log.created_at.to_rfc3339(),
            last_attempt_at: log.created_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(WebhookTestLogsResponse { logs: items, total }))
}

/// Webhook test logs response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTestLogsResponse {
    pub logs: Vec<TestWebhookResponse>,
    pub total: i64,
}

/// Pagination query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

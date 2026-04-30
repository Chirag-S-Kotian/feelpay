use crate::engine::services::webhook_test::WebhookTestService;
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use actix_web::{web, HttpResponse};
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
    pub status: String,
    pub http_status: Option<i32>,
    pub response_time_ms: Option<i64>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
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

    let row = sqlx::query("SELECT webhook_url, webhook_secret FROM merchants WHERE id = $1")
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
    let mut webhook_secret = row.get::<Option<String>, _>(1);

    if let Some(url) = webhook_url.as_ref() {
        if !url.starts_with("https://") {
            return Err(FeelPayError::BadRequest(
                "Webhook URL must start with https://".into(),
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
        sqlx::query("UPDATE merchants SET webhook_secret = $2, updated_at = NOW() WHERE id = $1")
            .bind(merchant_id)
            .bind(&new_secret)
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
        status: result.status,
        http_status: result.http_status,
        response_time_ms: result.response_time_ms,
        response_body: result.response_body,
        error_message: result.error_message,
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
            status: log.status,
            http_status: log.http_status,
            response_time_ms: log.response_time_ms,
            response_body: log.response_body,
            error_message: log.error_message,
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

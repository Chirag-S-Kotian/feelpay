use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;

/// Extract merchant_id from request extensions (inserted by ApiKeyMiddleware or JwtMiddleware)
fn extract_merchant_id(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": { "code": "unauthorized", "message": "Missing merchant identity" }
            }))
        })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/analytics")
            .route("/summary", web::get().to(get_summary))
            .route("/volume", web::get().to(get_volume_chart))
            .route("/top-customers", web::get().to(get_top_customers)),
    );
}

/// GET /v1/analytics/summary — aggregate stats for authenticated merchant
pub async fn get_summary(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let stats = sqlx::query_as::<_, AnalyticsSummaryRow>(
        r#"
        SELECT
            COUNT(*)::BIGINT                                     AS total_transactions,
            COALESCE(SUM(amount), 0)::BIGINT                      AS total_volume,
            COUNT(*) FILTER (WHERE status = 'captured')::BIGINT  AS successful,
            COUNT(*) FILTER (WHERE status = 'failed')::BIGINT    AS failed,
            CASE WHEN COUNT(*) > 0
                 THEN ROUND(COUNT(*) FILTER (WHERE status = 'captured')::NUMERIC / COUNT(*)::NUMERIC * 100, 2)
                 ELSE 0 END                                      AS success_rate
        FROM payment_orders
        WHERE merchant_id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_one(pool.get_ref())
    .await;

    match stats {
        Ok(row) => HttpResponse::Ok().json(AnalyticsSummary {
            total_transactions: row.total_transactions.unwrap_or(0),
            total_volume: row.total_volume.unwrap_or(0),
            successful: row.successful.unwrap_or(0),
            failed: row.failed.unwrap_or(0),
            success_rate: row.success_rate,
        }),
        Err(e) => {
            tracing::error!("Analytics query failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "analytics_error", "message": "Failed to fetch analytics" }
            }))
        }
    }
}

#[derive(sqlx::FromRow)]
struct AnalyticsSummaryRow {
    total_transactions: Option<i64>,
    total_volume: Option<i64>,
    successful: Option<i64>,
    failed: Option<i64>,
    success_rate: Decimal,
}

#[derive(Serialize)]
struct AnalyticsSummary {
    total_transactions: i64,
    total_volume: i64,
    successful: i64,
    failed: i64,
    success_rate: rust_decimal::Decimal,
}

/// GET /v1/analytics/volume — daily volume for last 7 days
pub async fn get_volume_chart(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let rows = sqlx::query_as::<_, DayVolumeRow>(
        r#"
        SELECT
            d.day::DATE                                                  AS day,
            COALESCE(COUNT(p.id)::BIGINT, 0)                             AS count,
            COALESCE(SUM(p.amount)::BIGINT, 0)                           AS volume
        FROM generate_series(CURRENT_DATE - INTERVAL '6 days', CURRENT_DATE, '1 day') AS d(day)
        LEFT JOIN payment_orders p ON p.merchant_id = $1 AND DATE(p.created_at) = d.day
        GROUP BY d.day
        ORDER BY d.day
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(data) => {
            let chart: Vec<DayVolume> = data
                .into_iter()
                .map(|r| DayVolume {
                    day: r.day.format("%a").to_string(),
                    date: r.day.to_string(),
                    count: r.count.unwrap_or(0),
                    volume: r.volume.unwrap_or(0),
                })
                .collect();
            HttpResponse::Ok().json(serde_json::json!({ "chart": chart }))
        }
        Err(e) => {
            tracing::error!("Volume chart query failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "analytics_error", "message": "Failed to fetch volume chart" }
            }))
        }
    }
}

#[derive(sqlx::FromRow)]
struct DayVolumeRow {
    day: chrono::NaiveDate,
    count: Option<i64>,
    volume: Option<i64>,
}

#[derive(Serialize)]
struct DayVolume {
    day: String,
    date: String,
    count: i64,
    volume: i64,
}

#[derive(sqlx::FromRow)]
struct VolumeTrendRow {
    day: Option<chrono::DateTime<chrono::Utc>>,
    count: Option<i64>,
    volume: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct TopCustomerRow {
    customer_email: Option<String>,
    count: Option<i64>,
    volume: Option<i64>,
}

pub async fn get_volume_trends(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let rows = sqlx::query_as::<_, VolumeTrendRow>(
        r#"
        SELECT 
            DATE_TRUNC('day', created_at) as day,
            COUNT(*)::BIGINT as count,
            SUM(amount)::BIGINT as volume
        FROM payment_orders
        WHERE merchant_id = $1 AND created_at > NOW() - INTERVAL '7 days'
        GROUP BY 1
        ORDER BY 1 ASC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(data) => {
            let trends: Vec<_> = data.into_iter().map(|r| {
                serde_json::json!({
                    "date": r.day.map(|d| d.to_string()).unwrap_or_default(),
                    "count": r.count.unwrap_or(0),
                    "volume": r.volume.unwrap_or(0)
                })
            }).collect();
            HttpResponse::Ok().json(serde_json::json!({ "trends": trends }))
        }
        Err(e) => {
            tracing::error!("Volume trends query failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "analytics_error", "message": "Failed to fetch volume trends" }
            }))
        }
    }
}

pub async fn get_top_customers(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let rows = sqlx::query_as::<_, TopCustomerRow>(
        r#"
        SELECT 
            customer_email,
            COUNT(*)::BIGINT as count,
            SUM(amount)::BIGINT as volume
        FROM payment_orders
        WHERE merchant_id = $1 AND customer_email IS NOT NULL
        GROUP BY 1
        ORDER BY volume DESC
        LIMIT 5
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(data) => {
            let customers: Vec<_> = data.into_iter().map(|r| {
                serde_json::json!({
                    "email": r.customer_email,
                    "count": r.count.unwrap_or(0),
                    "volume": r.volume.unwrap_or(0)
                })
            }).collect();
            HttpResponse::Ok().json(serde_json::json!({ "top_customers": customers }))
        }
        Err(e) => {
            tracing::error!("Top customers query failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "analytics_error", "message": "Failed to fetch top customers" }
            }))
        }
    }
}

/// GET /v1/developers/webhooks — recent webhook delivery log
pub async fn get_webhook_events(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let rows = sqlx::query_as::<_, WebhookEventRow>(
        r#"
        SELECT id, event_type, status, attempts, created_at
        FROM webhook_events
        WHERE merchant_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(events) => {
            let list: Vec<WebhookEvent> = events
                .into_iter()
                .map(|e| WebhookEvent {
                    id: e.id.to_string(),
                    event_type: e.event_type,
                    status: e.status,
                    attempts: e.attempts,
                    created_at: e.created_at.to_string(),
                })
                .collect();
            HttpResponse::Ok().json(serde_json::json!({ "events": list }))
        }
        Err(e) => {
            tracing::error!("Webhook events query failed: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": { "code": "analytics_error", "message": "Failed to fetch webhook events" }
            }))
        }
    }
}

#[derive(sqlx::FromRow)]
struct WebhookEventRow {
    id: uuid::Uuid,
    event_type: String,
    status: String,
    attempts: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct WebhookEvent {
    id: String,
    event_type: String,
    status: String,
    attempts: i32,
    created_at: String,
}

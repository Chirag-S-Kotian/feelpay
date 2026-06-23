use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

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

fn extract_environment(req: &HttpRequest) -> &str {
    let env_mode = req.extensions()
        .get::<crate::middleware::auth::RequestEnvironment>()
        .map(|env| env.0)
        .unwrap_or(crate::middleware::auth::EnvironmentMode::Test);
    match env_mode {
        crate::middleware::auth::EnvironmentMode::Live => "live",
        crate::middleware::auth::EnvironmentMode::Test => "test",
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/analytics")
            .route("/summary", web::get().to(get_dashboard_stats))
            .route("/volume", web::get().to(get_volume_chart))
            .route("/top-customers", web::get().to(get_top_customers))
            .route("/alerts", web::get().to(get_actionable_alerts)),
    );
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ApiLogQuery {
    pub status: Option<String>,
    pub endpoint: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/v1/analytics/alerts",
    responses(
        (status = 200, description = "Actionable alerts for the merchant", body = Object),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_actionable_alerts(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let mut alerts = Vec::new();

    // Check KYC status
    let merchant = sqlx::query("SELECT onboarding_status FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_one(pool.get_ref())
        .await;

    if let Ok(m) = merchant {
        let onboarding_status: String = m.get("onboarding_status");
        if onboarding_status != "approved" {
            alerts.push(serde_json::json!({
                "type": "kyc_pending",
                "severity": "warning",
                "message": format!("Your KYC status is {}. Please complete onboarding to enable settlements.", onboarding_status),
                "action_url": "/dashboard/onboarding"
            }));
        }
    }

    // Check for open disputes
    let disputes = sqlx::query(
        "SELECT COUNT(*) as count FROM disputes WHERE merchant_id = $1 AND status = 'open'",
    )
    .bind(merchant_id)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(d) = disputes {
        let count: i64 = d.get("count");
        if count > 0 {
            alerts.push(serde_json::json!({
                "type": "dispute_action",
                "severity": "error",
                "message": format!("You have {} open disputes that require evidence submission.", count),
                "action_url": "/dashboard/disputes"
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({ "alerts": alerts }))
}

/// GET /v1/analytics/summary — aggregate stats for authenticated merchant
#[utoipa::path(
    get,
    path = "/v1/analytics/summary",
    responses(
        (status = 200, description = "Analytics summary", body = AnalyticsSummary),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_dashboard_stats(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let environment = extract_environment(&req);

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
          AND COALESCE(metadata->>'environment', 'test') = $2::TEXT
        "#,
    )
    .bind(merchant_id)
    .bind(environment)
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

#[derive(Serialize, ToSchema)]
pub struct AnalyticsSummary {
    total_transactions: i64,
    total_volume: i64,
    successful: i64,
    failed: i64,
    success_rate: rust_decimal::Decimal,
}

/// GET /v1/analytics/volume — daily volume for last 7 days
#[utoipa::path(
    get,
    path = "/v1/analytics/volume",
    responses(
        (status = 200, description = "Daily transaction volume for last 7 days", body = [DayVolume]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_volume_chart(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let environment = extract_environment(&req);

    let rows = sqlx::query_as::<_, DayVolumeRow>(
        r#"
        SELECT
            d.day::DATE                                                  AS day,
            COALESCE(COUNT(p.id)::BIGINT, 0)                             AS count,
            COALESCE(SUM(p.amount)::BIGINT, 0)                           AS volume
        FROM generate_series(CURRENT_DATE - INTERVAL '6 days', CURRENT_DATE, '1 day') AS d(day)
        LEFT JOIN payment_orders p ON p.merchant_id = $1 AND DATE(p.created_at) = d.day AND COALESCE(p.metadata->>'environment', 'test') = $2::TEXT
        GROUP BY d.day
        ORDER BY d.day
        "#,
    )
    .bind(merchant_id)
    .bind(environment)
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

#[derive(Serialize, ToSchema)]
pub struct DayVolume {
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

pub async fn get_volume_trends(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let environment = extract_environment(&req);

    let rows = sqlx::query_as::<_, VolumeTrendRow>(
        r#"
        SELECT 
            DATE_TRUNC('day', created_at) as day,
            COUNT(*)::BIGINT as count,
            SUM(amount)::BIGINT as volume
        FROM payment_orders
        WHERE merchant_id = $1 AND created_at > NOW() - INTERVAL '7 days' AND COALESCE(metadata->>'environment', 'test') = $2::TEXT
        GROUP BY 1
        ORDER BY 1 ASC
        "#,
    )
    .bind(merchant_id)
    .bind(environment)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(data) => {
            let trends: Vec<_> = data
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "date": r.day.map(|d| d.to_string()).unwrap_or_default(),
                        "count": r.count.unwrap_or(0),
                        "volume": r.volume.unwrap_or(0)
                    })
                })
                .collect();
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

#[utoipa::path(
    get,
    path = "/v1/analytics/top-customers",
    responses(
        (status = 200, description = "Top 5 customers by volume", body = [TopCustomerResponse]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_top_customers(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let environment = extract_environment(&req);

    let rows = sqlx::query_as::<_, TopCustomerRow>(
        r#"
        SELECT 
            customer_email,
            COUNT(*)::BIGINT as count,
            SUM(amount)::BIGINT as volume
        FROM payment_orders
        WHERE merchant_id = $1 AND customer_email IS NOT NULL AND COALESCE(metadata->>'environment', 'test') = $2::TEXT
        GROUP BY 1
        ORDER BY volume DESC
        LIMIT 5
        "#,
    )
    .bind(merchant_id)
    .bind(environment)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(data) => {
            let customers: Vec<_> = data
                .into_iter()
                .map(|r| TopCustomerResponse {
                    email: r.customer_email,
                    count: r.count.unwrap_or(0),
                    volume: r.volume.unwrap_or(0),
                })
                .collect();
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
pub async fn get_webhook_events(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
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

#[utoipa::path(
    get,
    path = "/v1/developers/api-logs",
    params(
        ApiLogQuery
    ),
    responses(
        (status = 200, description = "Developer API activity logs")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_api_logs(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<ApiLogQuery>,
) -> HttpResponse {
    let merchant_id = match extract_merchant_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let status_filter = query.status.as_deref().map(|s| s.to_lowercase());
    let endpoint_filter = query.endpoint.as_deref().map(|s| s.to_lowercase());

    let rows = sqlx::query(
        r#"
        SELECT id, entity_type, action, actor_id, ip_address, created_at
        FROM audit_logs
        WHERE actor_id = $1::TEXT
          AND ($2::TEXT IS NULL OR LOWER(entity_type || ':' || action) LIKE '%' || $2::TEXT || '%')
          AND ($3::DATE IS NULL OR created_at::DATE >= $3::DATE)
          AND ($4::DATE IS NULL OR created_at::DATE <= $4::DATE)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(merchant_id.to_string())
    .bind(endpoint_filter)
    .bind(&query.from_date)
    .bind(&query.to_date)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let Ok(rows) = rows else {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": { "code": "analytics_error", "message": "Failed to fetch API logs" }
        }));
    };

    let mut logs = Vec::new();
    for row in rows {
        let action: String = row.get("action");
        let status = if action.contains("failed") {
            "failed"
        } else {
            "success"
        };
        if let Some(sf) = status_filter.as_deref() {
            if sf != status {
                continue;
            }
        }
        let entity_type: String = row.get("entity_type");
        let endpoint = format!("{entity_type}:{action}");
        logs.push(serde_json::json!({
            "request_id": row.get::<uuid::Uuid, _>("id").to_string(),
            "endpoint": endpoint,
            "status": status,
            "actor_id": row.get::<Option<String>, _>("actor_id"),
            "ip_address": row.get::<Option<String>, _>("ip_address"),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "page": page,
        "per_page": per_page,
        "logs": logs
    }))
}

#[derive(Serialize, ToSchema)]
pub struct TopCustomerResponse {
    pub email: Option<String>,
    pub count: i64,
    pub volume: i64,
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

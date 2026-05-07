//! Admin Analytics Service
//!
//! Provides platform-wide analytics including:
//! - Merchant signups over time
//! - Dispute rate trends
//! - Provider transaction splits

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::{IntoParams, ToSchema};

use crate::error::FeelPayError;
use crate::models::user::Claims;
use crate::state::AppState;

/// Granularity for time series data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Granularity {
    Daily,
    Weekly,
    Monthly,
}

impl Granularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Granularity::Daily => "day",
            Granularity::Weekly => "week",
            Granularity::Monthly => "month",
        }
    }
}

/// Date range for analytics queries
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TimeSeriesPoint {
    pub date: NaiveDate,
    pub value: f64,
    pub count: i64,
}

/// Provider transaction split
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProviderSplit {
    pub provider: String,
    pub transaction_count: i64,
    pub volume: i64,
    pub percentage: f64,
}

/// Platform analytics response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlatformAnalytics {
    pub merchant_signups: Vec<TimeSeriesPoint>,
    pub dispute_rate: Vec<TimeSeriesPoint>,
    pub provider_split: Vec<ProviderSplit>,
    pub period: DateRange,
}

/// Query parameters for analytics
#[derive(Debug, Deserialize, IntoParams)]
pub struct AnalyticsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub granularity: Option<Granularity>,
}

impl Default for AnalyticsQuery {
    fn default() -> Self {
        let now = Utc::now();
        let start = now - chrono::Duration::days(30);
        Self {
            start_date: Some(start),
            end_date: Some(now),
            granularity: Some(Granularity::Daily),
        }
    }
}

/// GET /api/admin/analytics - Get platform analytics
#[utoipa::path(
    get,
    path = "/api/admin/analytics",
    params(AnalyticsQuery),
    responses(
        (status = 200, description = "Platform analytics", body = PlatformAnalytics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("admin_jwt" = [])
    )
)]
pub async fn get_platform_analytics(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    // Verify admin auth from JWT claims
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| FeelPayError::Unauthorized("Authentication required".into()))?;

    // Check admin role
    if claims.role != "super_admin" && claims.role != "Super Admin" && claims.role != "admin" {
        return Err(FeelPayError::Forbidden("Admin access required".into()));
    }

    let query = query.into_inner();
    let start = query
        .start_date
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = query.end_date.unwrap_or_else(Utc::now);
    let granularity = query.granularity.unwrap_or(Granularity::Daily);

    let analytics = get_analytics(&state.db, start, end, granularity).await?;

    Ok(HttpResponse::Ok().json(analytics))
}

/// Get platform analytics from database
async fn get_analytics(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: Granularity,
) -> Result<PlatformAnalytics, FeelPayError> {
    let gran_str = granularity.as_str();

    // Merchant signups time series
    let signups = sqlx::query(
        r#"
        SELECT 
            DATE_TRUNC($3, created_at)::DATE as date,
            COUNT(*)::FLOAT as value,
            COUNT(*) as count
        FROM merchants
        WHERE created_at BETWEEN $1 AND $2
        GROUP BY DATE_TRUNC($3, created_at)
        ORDER BY date
        "#,
    )
    .bind(start)
    .bind(end)
    .bind(gran_str)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .iter()
    .map(|row| TimeSeriesPoint {
        date: row.get::<chrono::NaiveDate, _>("date"),
        value: row.get::<f64, _>("value"),
        count: row.get::<i64, _>("count"),
    })
    .collect::<Vec<_>>();

    // Dispute rate time series
    let dispute_rate = sqlx::query(
        r#"
        SELECT 
            DATE_TRUNC($3, d.created_at)::DATE as date,
            COUNT(DISTINCT d.id)::FLOAT / NULLIF(COUNT(DISTINCT p.id), 0) * 100 as value,
            COUNT(DISTINCT d.id) as count
        FROM disputes d
        RIGHT JOIN payment_orders p ON DATE_TRUNC($3, p.created_at) = DATE_TRUNC($3, d.created_at)
            AND d.created_at BETWEEN $1 AND $2
        WHERE p.created_at BETWEEN $1 AND $2
        GROUP BY DATE_TRUNC($3, d.created_at)
        ORDER BY date
        "#,
    )
    .bind(start)
    .bind(end)
    .bind(gran_str)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .iter()
    .map(|row| TimeSeriesPoint {
        date: row.get::<chrono::NaiveDate, _>("date"),
        value: row.get::<f64, _>("value"),
        count: row.get::<i64, _>("count"),
    })
    .collect::<Vec<_>>();

    // Provider split
    let provider_split = sqlx::query(
        r#"
        SELECT 
            COALESCE(connector, 'unknown') as provider,
            COUNT(*) as transaction_count,
            COALESCE(SUM(amount), 0) as volume,
            COUNT(*)::FLOAT / NULLIF(SUM(COUNT(*)) OVER (), 0) * 100 as percentage
        FROM payment_orders
        WHERE created_at BETWEEN $1 AND $2
        GROUP BY connector
        ORDER BY transaction_count DESC
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .iter()
    .map(|row| ProviderSplit {
        provider: row.get::<String, _>("provider"),
        transaction_count: row.get::<i64, _>("transaction_count"),
        volume: row.get::<i64, _>("volume"),
        percentage: row.get::<f64, _>("percentage"),
    })
    .collect::<Vec<_>>();

    Ok(PlatformAnalytics {
        merchant_signups: signups,
        dispute_rate,
        provider_split,
        period: DateRange { start, end },
    })
}

/// Configure admin analytics routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin/analytics").route("", web::get().to(get_platform_analytics)));
}

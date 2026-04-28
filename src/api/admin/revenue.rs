use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RevenueQuery {
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RevenueBreakdown {
    pub label: String,
    pub transaction_count: i64,
    pub volume: i64,
    pub revenue: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MerchantRevenue {
    pub merchant_id: Uuid,
    pub merchant_name: String,
    pub transaction_count: i64,
    pub volume: i64,
    pub revenue: i64,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin/revenue").route("", web::get().to(get_revenue_dashboard)));
}

pub async fn get_revenue_dashboard(
    state: web::Data<AppState>,
    query: web::Query<RevenueQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let by_payment_method = sqlx::query_as::<_, RevenueBreakdown>(
        r#"
        SELECT COALESCE(payment_method, 'unknown') AS label,
               COUNT(*)::BIGINT AS transaction_count,
               COALESCE(SUM(amount), 0)::BIGINT AS volume,
               COALESCE(SUM((amount * 20) / 10000), 0)::BIGINT AS revenue
        FROM payment_orders
        WHERE status IN ('captured', 'settled', 'refunded', 'partially_refunded')
          AND ($1::TIMESTAMPTZ IS NULL OR created_at >= $1)
          AND ($2::TIMESTAMPTZ IS NULL OR created_at <= $2)
        GROUP BY payment_method
        ORDER BY revenue DESC
        "#,
    )
    .bind(query.date_from)
    .bind(query.date_to)
    .fetch_all(&state.db)
    .await?;

    let by_fee_plan = sqlx::query_as::<_, RevenueBreakdown>(
        r#"
        SELECT COALESCE(fp.name, 'default') AS label,
               COUNT(p.*)::BIGINT AS transaction_count,
               COALESCE(SUM(p.amount), 0)::BIGINT AS volume,
               COALESCE(SUM((p.amount * 20) / 10000), 0)::BIGINT AS revenue
        FROM payment_orders p
        JOIN merchants m ON m.id = p.merchant_id
        LEFT JOIN merchant_fee_assignments mfa
          ON mfa.merchant_id = m.id
         AND mfa.effective_from <= p.created_at
         AND (mfa.effective_until IS NULL OR mfa.effective_until > p.created_at)
        LEFT JOIN fee_plans fp ON fp.id = mfa.fee_plan_id
        WHERE p.status IN ('captured', 'settled', 'refunded', 'partially_refunded')
          AND ($1::TIMESTAMPTZ IS NULL OR p.created_at >= $1)
          AND ($2::TIMESTAMPTZ IS NULL OR p.created_at <= $2)
        GROUP BY fp.name
        ORDER BY revenue DESC
        "#,
    )
    .bind(query.date_from)
    .bind(query.date_to)
    .fetch_all(&state.db)
    .await?;

    let top_merchants = sqlx::query_as::<_, MerchantRevenue>(
        r#"
        SELECT m.id AS merchant_id,
               COALESCE(m.business_name, m.email) AS merchant_name,
               COUNT(p.*)::BIGINT AS transaction_count,
               COALESCE(SUM(p.amount), 0)::BIGINT AS volume,
               COALESCE(SUM((p.amount * 20) / 10000), 0)::BIGINT AS revenue
        FROM payment_orders p
        JOIN merchants m ON m.id = p.merchant_id
        WHERE p.status IN ('captured', 'settled', 'refunded', 'partially_refunded')
          AND ($1::TIMESTAMPTZ IS NULL OR p.created_at >= $1)
          AND ($2::TIMESTAMPTZ IS NULL OR p.created_at <= $2)
        GROUP BY m.id, m.business_name, m.email
        ORDER BY revenue DESC
        LIMIT 25
        "#,
    )
    .bind(query.date_from)
    .bind(query.date_to)
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "by_payment_method": by_payment_method,
        "by_fee_plan": by_fee_plan,
        "top_merchants": top_merchants
    })))
}

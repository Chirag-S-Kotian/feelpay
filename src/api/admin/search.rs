use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
use std::time::Duration;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct TransactionSearchParams {
    pub transaction_id: Option<Uuid>,
    pub order_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub merchant_id: Option<Uuid>,
    pub status: Option<String>,
    pub payment_method: Option<String>,
    pub amount_min: Option<i64>,
    pub amount_max: Option<i64>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionSearchResult {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub order_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub is_frozen: bool,
    pub is_flagged: bool,
    pub created_at: DateTime<Utc>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/transactions")
            .route("/search", web::get().to(search_transactions))
            .route("/export", web::get().to(export_transactions_csv)),
    );
}

pub async fn search_transactions(
    state: web::Data<AppState>,
    query: web::Query<TransactionSearchParams>,
) -> Result<HttpResponse, FeelPayError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT id, merchant_id, order_id, amount, currency, status::text, payment_method,
               customer_email, customer_phone, is_frozen, is_flagged, created_at
        FROM payment_orders WHERE 1 = 1
        "#,
    );

    if let Some(id) = query.transaction_id {
        qb.push(" AND id = ").push_bind(id);
    }
    if let Some(v) = &query.order_id {
        qb.push(" AND order_id ILIKE ")
            .push_bind(format!("%{}%", v));
    }
    if let Some(v) = &query.customer_email {
        qb.push(" AND customer_email ILIKE ")
            .push_bind(format!("%{}%", v));
    }
    if let Some(v) = &query.customer_phone {
        qb.push(" AND customer_phone ILIKE ")
            .push_bind(format!("%{}%", v));
    }
    if let Some(id) = query.merchant_id {
        qb.push(" AND merchant_id = ").push_bind(id);
    }
    if let Some(v) = &query.status {
        qb.push(" AND status = ").push_bind(v);
    }
    if let Some(v) = &query.payment_method {
        qb.push(" AND payment_method = ").push_bind(v);
    }
    if let Some(v) = query.amount_min {
        qb.push(" AND amount >= ").push_bind(v);
    }
    if let Some(v) = query.amount_max {
        qb.push(" AND amount <= ").push_bind(v);
    }
    if let Some(v) = query.date_from {
        qb.push(" AND created_at >= ").push_bind(v);
    }
    if let Some(v) = query.date_to {
        qb.push(" AND created_at <= ").push_bind(v);
    }
    if let Some(v) = &query.cursor {
        if let Ok(ts) = DateTime::parse_from_rfc3339(v) {
            qb.push(" AND created_at < ")
                .push_bind(ts.with_timezone(&Utc));
        }
    }

    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(limit + 1);

    let mut rows = tokio::time::timeout(
        Duration::from_secs(3),
        qb.build_query_as::<TransactionSearchResult>()
            .fetch_all(&state.db),
    )
    .await
    .map_err(|_| FeelPayError::Internal("Search timeout".into()))?
    .map_err(FeelPayError::from)?;

    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.pop();
    }
    let next_cursor = rows.last().map(|r| r.created_at.to_rfc3339());

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "results": rows,
        "has_more": has_more,
        "next_cursor": next_cursor
    })))
}

pub async fn export_transactions_csv(
    state: web::Data<AppState>,
    query: web::Query<TransactionSearchParams>,
) -> Result<HttpResponse, FeelPayError> {
    let mut params = query.into_inner();
    params.limit = Some(5_000);
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT id, merchant_id, order_id, amount, currency, status::text, payment_method,
               customer_email, customer_phone, is_frozen, is_flagged, created_at
        FROM payment_orders WHERE 1 = 1
        "#,
    );
    if let Some(id) = params.transaction_id {
        qb.push(" AND id = ").push_bind(id);
    }
    if let Some(id) = params.merchant_id {
        qb.push(" AND merchant_id = ").push_bind(id);
    }
    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(params.limit.unwrap_or(5_000));

    let rows = tokio::time::timeout(
        Duration::from_secs(3),
        qb.build_query_as::<TransactionSearchResult>()
            .fetch_all(&state.db),
    )
    .await
    .map_err(|_| FeelPayError::Internal("Search timeout".into()))?
    .map_err(FeelPayError::from)?;

    let mut writer = csv::Writer::from_writer(Vec::new());
    for row in rows {
        writer
            .serialize((
                row.id,
                row.merchant_id,
                row.order_id,
                row.amount,
                row.currency,
                row.status,
                row.payment_method,
                row.customer_email,
                row.customer_phone,
                row.is_frozen,
                row.is_flagged,
                row.created_at.to_rfc3339(),
            ))
            .map_err(|e| FeelPayError::Internal(e.to_string()))?;
    }
    let csv = String::from_utf8(
        writer
            .into_inner()
            .map_err(|e| FeelPayError::Internal(e.to_string()))?,
    )
    .map_err(|e| FeelPayError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .append_header((
            "content-disposition",
            "attachment; filename=\"transactions.csv\"",
        ))
        .body(csv))
}

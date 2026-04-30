use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::engine::services::settlement::SettlementService;
use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settlements")
            .route("", web::get().to(list_settlements))
            .route("/balance", web::get().to(get_settlement_stats))
            .route("/reconciliation", web::get().to(get_reconciliation_summary))
            .route("/{id}", web::get().to(get_settlement)),
    );
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SettlementQuery {
    pub status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

#[utoipa::path(
    get,
    path = "/v1/settlements",
    params(
        SettlementQuery
    ),
    responses(
        (status = 200, description = "List of settlements", body = Vec<SettlementResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_settlements(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<SettlementQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let rows: Vec<(
        Uuid,
        Option<NaiveDate>,
        String,
        i64,
        i64,
        i64,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"
        SELECT id, settlement_date, status, amount, fee_amount, net_amount, utr_number, processed_at, created_at
        FROM settlements
        WHERE merchant_id = $1
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
          AND ($3::DATE IS NULL OR settlement_date >= $3::DATE)
          AND ($4::DATE IS NULL OR settlement_date <= $4::DATE)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(merchant_id)
    .bind(&query.status)
    .bind(&query.from_date)
    .bind(&query.to_date)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM settlements
        WHERE merchant_id = $1
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
          AND ($3::DATE IS NULL OR settlement_date >= $3::DATE)
          AND ($4::DATE IS NULL OR settlement_date <= $4::DATE)
        "#,
    )
    .bind(merchant_id)
    .bind(&query.status)
    .bind(&query.from_date)
    .bind(&query.to_date)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let settlements: Vec<serde_json::Value> = rows
        .into_iter()
        .map(
            |(
                id,
                settlement_date,
                status,
                amount,
                fee_amount,
                net_amount,
                utr_number,
                processed_at,
                created_at,
            )| {
                let tax_amount = ((fee_amount as f64) * 0.18_f64).round() as i64;
                serde_json::json!({
                    "id": id,
                    "settlement_date": settlement_date,
                    "status": status,
                    "amount": amount,
                    "fee_amount": fee_amount,
                    "tax_amount": tax_amount,
                    "net_amount": net_amount,
                    "utr": utr_number,
                    "processed_at": processed_at,
                    "created_at": created_at
                })
            },
        )
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "page": page,
        "per_page": per_page,
        "total": total.0,
        "settlements": settlements
    })))
}

#[utoipa::path(
    get,
    path = "/v1/settlements/{id}",
    params(
        ("id" = Uuid, Path, description = "Settlement ID")
    ),
    responses(
        (status = 200, description = "Settlement details", body = SettlementResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Settlement not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_settlement(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let settlement_id = path.into_inner();

    let row: Option<(
        Uuid,
        Option<NaiveDate>,
        String,
        i64,
        i64,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"
        SELECT id, settlement_date, status, amount, fee_amount, net_amount, utr_number, bank_reference, error_message, processed_at, created_at
        FROM settlements
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(settlement_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let Some((
        id,
        settlement_date,
        status,
        amount,
        fee_amount,
        net_amount,
        utr_number,
        bank_reference,
        error_message,
        processed_at,
        created_at,
    )) = row
    else {
        return Err(FeelPayError::NotFound("Settlement not found".into()));
    };

    let items: Vec<(Uuid, i64, i64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT transaction_id, amount, fee, created_at
        FROM settlement_items
        WHERE settlement_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(settlement_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "settlement_date": settlement_date,
        "status": status,
        "amount": amount,
        "fee_amount": fee_amount,
        "tax_amount": ((fee_amount as f64) * 0.18_f64).round() as i64,
        "net_amount": net_amount,
        "utr": utr_number,
        "bank_reference": bank_reference,
        "error_message": error_message,
        "processed_at": processed_at,
        "created_at": created_at,
        "items": items.into_iter().map(|(transaction_id, amount, fee, created_at)| serde_json::json!({
            "transaction_id": transaction_id,
            "amount": amount,
            "fee": fee,
            "created_at": created_at
        })).collect::<Vec<_>>()
    })))
}

#[utoipa::path(
    get,
    path = "/v1/settlements/balance",
    responses(
        (status = 200, description = "Settlement balance stats")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_settlement_stats(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let svc = SettlementService::new(state.db.clone());
    let (available, pending) = svc.get_balance(merchant_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "available": available,
        "pending": pending
    })))
}

#[utoipa::path(
    get,
    path = "/v1/settlements/reconciliation",
    responses(
        (status = 200, description = "Settlement reconciliation summary")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_reconciliation_summary(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let row = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(amount), 0)::BIGINT AS gross,
            COALESCE(SUM(fee_amount), 0)::BIGINT AS fees,
            COALESCE(SUM(net_amount), 0)::BIGINT AS net,
            COUNT(*) FILTER (WHERE status = 'completed')::BIGINT AS completed_count,
            COUNT(*) FILTER (WHERE status = 'pending')::BIGINT AS pending_count,
            COUNT(*) FILTER (WHERE status = 'failed')::BIGINT AS failed_count
        FROM settlements
        WHERE merchant_id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let gross: i64 = row.get("gross");
    let fees: i64 = row.get("fees");
    let net: i64 = row.get("net");
    let tax = ((fees as f64) * 0.18_f64).round() as i64;
    let completed_count: i64 = row.get("completed_count");
    let pending_count: i64 = row.get("pending_count");
    let failed_count: i64 = row.get("failed_count");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "gross": gross,
        "fees": fees,
        "tax": tax,
        "net": net,
        "variance": gross - (fees + tax + net),
        "counts": {
            "completed": completed_count,
            "pending": pending_count,
            "failed": failed_count
        }
    })))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SettlementResponse {
    pub id: Uuid,
    pub settlement_date: Option<NaiveDate>,
    pub status: String,
    pub amount: i64,
    pub fee_amount: i64,
    pub tax_amount: i64,
    pub net_amount: i64,
    pub utr: Option<String>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

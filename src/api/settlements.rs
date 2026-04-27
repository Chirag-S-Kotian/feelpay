use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::engine::services::settlement::SettlementService;
use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settlements")
            .route("", web::get().to(list_settlements))
            .route("/balance", web::get().to(get_balance))
            .route("/{id}", web::get().to(get_settlement)),
    );
}

#[derive(Debug, Deserialize)]
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

pub async fn get_balance(
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

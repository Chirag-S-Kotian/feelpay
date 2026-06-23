use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::engine::services::settlement::SettlementService;
use crate::error::FeelPayError;
use crate::state::AppState;

fn gst_on_fee_paise(fee_amount: i64) -> i64 {
    (((fee_amount as i128 * 18) + 50) / 100) as i64
}

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

fn environment_from_req(req: &HttpRequest) -> crate::middleware::auth::EnvironmentMode {
    req.extensions()
        .get::<crate::middleware::auth::RequestEnvironment>()
        .map(|env| env.0)
        .unwrap_or(crate::middleware::auth::EnvironmentMode::Test)
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
    let env_mode = environment_from_req(&req);
    let environment_str = match env_mode {
        crate::middleware::auth::EnvironmentMode::Live => "live",
        crate::middleware::auth::EnvironmentMode::Test => "test",
    };
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
          AND environment = $2::TEXT
          AND ($3::TEXT IS NULL OR status = $3::TEXT)
          AND ($4::DATE IS NULL OR settlement_date >= $4::DATE)
          AND ($5::DATE IS NULL OR settlement_date <= $5::DATE)
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(merchant_id)
    .bind(environment_str)
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
          AND environment = $2::TEXT
          AND ($3::TEXT IS NULL OR status = $3::TEXT)
          AND ($4::DATE IS NULL OR settlement_date >= $4::DATE)
          AND ($5::DATE IS NULL OR settlement_date <= $5::DATE)
        "#,
    )
    .bind(merchant_id)
    .bind(environment_str)
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
                let tax_amount = gst_on_fee_paise(fee_amount);
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

    let refund_adjustments: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(r.amount), 0)::BIGINT
        FROM settlement_items si
        JOIN refunds r ON r.payment_order_id = si.transaction_id
        WHERE si.settlement_id = $1
          AND r.merchant_id = $2
          AND r.status IN ('requested', 'processing', 'processed', 'succeeded')
        "#,
    )
    .bind(settlement_id)
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let bank_account: Option<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT account_holder_name, COALESCE(account_number, 'XXXX...XXXX') AS account_number, ifsc_code
        FROM bank_accounts
        WHERE merchant_id = $1
        ORDER BY is_primary DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "settlement_date": settlement_date,
        "status": status,
        "amount": amount,
        "fee_amount": fee_amount,
        "tax_amount": gst_on_fee_paise(fee_amount),
        "refund_adjustments": refund_adjustments,
        "chargeback_adjustments": 0,
        "other_adjustments": 0,
        "net_amount": net_amount,
        "utr": utr_number,
        "bank_reference": bank_reference,
        "bank_account": bank_account.map(|(account_holder_name, account_number_masked, ifsc_code)| serde_json::json!({
            "account_holder_name": account_holder_name,
            "account_number_masked": account_number_masked,
            "ifsc_code": ifsc_code
        })),
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
    let env_mode = environment_from_req(&req);
    let environment_str = match env_mode {
        crate::middleware::auth::EnvironmentMode::Live => "live",
        crate::middleware::auth::EnvironmentMode::Test => "test",
    };
    let svc = SettlementService::new(state.db.clone());
    let (available, pending, hold, reserve) = svc.get_balance(merchant_id, environment_str).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "available": available,
        "pending": pending,
        "hold": hold,
        "reserve": reserve
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
    let env_mode = environment_from_req(&req);
    let environment_str = match env_mode {
        crate::middleware::auth::EnvironmentMode::Live => "live",
        crate::middleware::auth::EnvironmentMode::Test => "test",
    };

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
          AND environment = $2::TEXT
        "#,
    )
    .bind(merchant_id)
    .bind(environment_str)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let gross: i64 = row.get("gross");
    let fees: i64 = row.get("fees");
    let net: i64 = row.get("net");
    let tax = gst_on_fee_paise(fees);
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

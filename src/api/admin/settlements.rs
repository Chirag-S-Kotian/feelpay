use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SettlementActionRequest {
    pub action: SettlementAction,
    pub utr_number: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementAction {
    Approve,
    Hold { reason: String },
    MarkPaid { utr: String },
    BulkUploadUtr { settlements: Vec<UtrMapping> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UtrMapping {
    pub settlement_id: Uuid,
    pub utr: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/settlements")
            .route("/{id}/actions", web::post().to(apply_settlement_action))
            .route("/bulk-utr", web::post().to(bulk_upload_utr))
            .route(
                "/bank-statement/parse",
                web::post().to(parse_bank_statement),
            )
            .route("/reconcile", web::post().to(reconcile_settlements)),
    );
}

pub async fn parse_bank_statement(body: String) -> Result<HttpResponse, FeelPayError> {
    let mut reader = csv::Reader::from_reader(body.as_bytes());
    let mut rows = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| FeelPayError::BadRequest(e.to_string()))?;
        rows.push(serde_json::json!({
            "date": row.get(0).unwrap_or_default(),
            "utr": row.get(1).unwrap_or_default(),
            "amount": row.get(2).unwrap_or_default(),
            "description": row.get(3).unwrap_or_default()
        }));
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({ "rows": rows })))
}

pub async fn reconcile_settlements(
    state: web::Data<AppState>,
    body: web::Json<Vec<UtrMapping>>,
) -> Result<HttpResponse, FeelPayError> {
    let mut matched = 0usize;
    let mut missing = Vec::new();
    for row in body.iter() {
        let result = sqlx::query("UPDATE settlements SET status = 'completed', utr = $2, utr_number = $2, processed_at = COALESCE(processed_at, NOW()), updated_at = NOW() WHERE id = $1 OR utr_number = $2")
            .bind(row.settlement_id)
            .bind(&row.utr)
            .execute(&state.db)
            .await?;
        if result.rows_affected() > 0 {
            matched += 1;
        } else {
            missing.push(row.settlement_id);
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "matched": matched,
        "missing": missing
    })))
}

pub async fn apply_settlement_action(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<SettlementActionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    let previous: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(settlements.*) FROM settlements WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| FeelPayError::NotFound("Settlement not found".into()))?;

    match &body.action {
        SettlementAction::Approve => {
            sqlx::query(
                "UPDATE settlements SET status = 'processing', updated_at = NOW() WHERE id = $1",
            )
            .bind(id)
            .execute(&state.db)
            .await?;
        }
        SettlementAction::Hold { reason } => {
            sqlx::query("UPDATE settlements SET status = 'held', hold_reason = $2, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .bind(reason)
                .execute(&state.db)
                .await?;
        }
        SettlementAction::MarkPaid { utr } => {
            validate_utr_available(&state.db, id, utr).await?;
            sqlx::query("UPDATE settlements SET status = 'completed', utr = $2, utr_number = $2, processed_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(id)
                .bind(utr)
                .execute(&state.db)
                .await?;
        }
        SettlementAction::BulkUploadUtr { settlements } => {
            apply_bulk_utrs(&state.db, settlements).await?;
        }
    }

    let updated: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(settlements.*) FROM settlements WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;

    log_audit(
        &state.db,
        admin_id,
        "settlement_admin_action",
        "settlement",
        Some(id),
        Some(previous),
        Some(updated),
        body.reason.as_deref(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id, "status": "ok" })))
}

pub async fn bulk_upload_utr(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<Vec<UtrMapping>>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    apply_bulk_utrs(&state.db, &body).await?;
    log_audit(
        &state.db,
        admin_id,
        "settlement_bulk_utr_upload",
        "settlement",
        None,
        None,
        Some(serde_json::to_value(&*body).unwrap_or_default()),
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "updated": body.len() })))
}

async fn validate_utr_available(
    pool: &sqlx::PgPool,
    settlement_id: Uuid,
    utr: &str,
) -> Result<(), FeelPayError> {
    let duplicate: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM settlements WHERE utr_number = $1 AND id <> $2")
            .bind(utr)
            .bind(settlement_id)
            .fetch_optional(pool)
            .await?;
    if duplicate.is_some() {
        return Err(FeelPayError::Conflict("UTR already assigned".into()));
    }
    Ok(())
}

async fn apply_bulk_utrs(pool: &sqlx::PgPool, rows: &[UtrMapping]) -> Result<(), FeelPayError> {
    for row in rows {
        validate_utr_available(pool, row.settlement_id, &row.utr).await?;
        sqlx::query("UPDATE settlements SET status = 'completed', utr = $2, utr_number = $2, processed_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(row.settlement_id)
            .bind(&row.utr)
            .execute(pool)
            .await?;
    }
    Ok(())
}

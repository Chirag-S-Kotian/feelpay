use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AdminActionRequest {
    pub action: AdminAction,
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAction {
    ForceRefund { amount: i64 },
    FlagSuspicious,
    Freeze,
    Unfreeze,
    ManualConfirm,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/transactions")
            .route("/{id}/actions", web::post().to(apply_transaction_action)),
    );
}

pub async fn apply_transaction_action(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<AdminActionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    let action_value = serde_json::to_value(&body.action).unwrap_or_default();

    let previous: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(payment_orders.*) FROM payment_orders WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(FeelPayError::from)?
            .ok_or_else(|| FeelPayError::NotFound("Transaction not found".into()))?;

    match &body.action {
        AdminAction::FlagSuspicious => {
            sqlx::query("UPDATE payment_orders SET is_flagged = TRUE, flagged_reason = $2, updated_at = NOW() WHERE id = $1")
                .bind(id).bind(&body.reason).execute(&state.db).await?;
        }
        AdminAction::Freeze => {
            sqlx::query("UPDATE payment_orders SET is_frozen = TRUE, frozen_reason = $2, updated_at = NOW() WHERE id = $1")
                .bind(id).bind(&body.reason).execute(&state.db).await?;
        }
        AdminAction::Unfreeze => {
            sqlx::query("UPDATE payment_orders SET is_frozen = FALSE, frozen_reason = NULL, updated_at = NOW() WHERE id = $1")
                .bind(id).execute(&state.db).await?;
        }
        AdminAction::ManualConfirm => {
            sqlx::query("UPDATE payment_orders SET manually_confirmed = TRUE, status = 'captured', captured_at = COALESCE(captured_at, NOW()), updated_at = NOW() WHERE id = $1")
                .bind(id).execute(&state.db).await?;
        }
        AdminAction::ForceRefund { amount } => {
            if *amount <= 0 {
                return Err(FeelPayError::BadRequest(
                    "Refund amount must be positive".into(),
                ));
            }
            sqlx::query("UPDATE payment_orders SET refunded_amount = refunded_amount + $2, status = CASE WHEN refunded_amount + $2 >= amount THEN 'refunded' ELSE 'partially_refunded' END, updated_at = NOW() WHERE id = $1")
                .bind(id).bind(amount).execute(&state.db).await?;
        }
    }

    let updated: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(payment_orders.*) FROM payment_orders WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(FeelPayError::from)?;

    log_audit(
        &state.db,
        admin_id,
        "transaction_admin_action",
        "payment_order",
        Some(id),
        Some(previous),
        Some(serde_json::json!({ "action": action_value, "transaction": updated })),
        Some(&body.reason),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id, "status": "ok" })))
}

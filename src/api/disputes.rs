use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::dispute::{Dispute, ListDisputesQuery, SubmitEvidenceRequest};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/disputes")
            .route("", web::get().to(list_disputes))
            .route("/{dispute_id}", web::get().to(get_dispute))
            .route("/{dispute_id}/evidence", web::post().to(submit_evidence))
            .route("/{dispute_id}/accept", web::post().to(accept_dispute)),
    );
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .or_else(|| {
            req.headers()
                .get("x-merchant-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant ID context".into()))
}

pub async fn list_disputes(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListDisputesQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, merchant_id, payment_order_id, amount, currency, status, reason_code, reason_message, evidence_due_by, evidence, connector_reference, created_at, updated_at, resolved_at
        FROM disputes
        WHERE merchant_id = $1
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(merchant_id)
    .bind(&query.status)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM disputes
        WHERE merchant_id = $1
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
        "#,
    )
    .bind(merchant_id)
    .bind(&query.status)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "page": page,
        "per_page": per_page,
        "total": total.0,
        "disputes": rows
    })))
}

pub async fn get_dispute(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let dispute_id = path.into_inner();

    let dispute = sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, merchant_id, payment_order_id, amount, currency, status, reason_code, reason_message, evidence_due_by, evidence, connector_reference, created_at, updated_at, resolved_at
        FROM disputes
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(dispute_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Dispute not found".into()))?;

    Ok(HttpResponse::Ok().json(dispute))
}

pub async fn submit_evidence(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<SubmitEvidenceRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let dispute_id = path.into_inner();

    let evidence = serde_json::json!({
        "text": payload.text,
        "documents": payload.documents,
        "submitted_at": chrono::Utc::now(),
    });

    let updated = sqlx::query_as::<_, Dispute>(
        r#"
        UPDATE disputes
        SET evidence = $1,
            status = 'under_review',
            updated_at = NOW()
        WHERE id = $2
          AND merchant_id = $3
          AND status IN ('needs_response', 'under_review')
        RETURNING id, merchant_id, payment_order_id, amount, currency, status, reason_code, reason_message, evidence_due_by, evidence, connector_reference, created_at, updated_at, resolved_at
        "#,
    )
    .bind(evidence)
    .bind(dispute_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::Conflict("Dispute not open for evidence submission".into()))?;

    Ok(HttpResponse::Ok().json(updated))
}

pub async fn accept_dispute(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let dispute_id = path.into_inner();

    let updated = sqlx::query_as::<_, Dispute>(
        r#"
        UPDATE disputes
        SET status = 'accepted',
            resolved_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
          AND merchant_id = $2
          AND status IN ('needs_response', 'under_review')
        RETURNING id, merchant_id, payment_order_id, amount, currency, status, reason_code, reason_message, evidence_due_by, evidence, connector_reference, created_at, updated_at, resolved_at
        "#,
    )
    .bind(dispute_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::Conflict("Dispute cannot be accepted in current state".into()))?;

    Ok(HttpResponse::Ok().json(updated))
}

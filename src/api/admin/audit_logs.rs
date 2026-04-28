use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub admin_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin/audit-logs").route("", web::get().to(list_audit_logs)))
        .service(web::scope("/admin/kyc-audit-log").route("", web::get().to(list_kyc_audit_logs)));
}

pub async fn list_audit_logs(
    state: web::Data<AppState>,
    query: web::Query<AuditLogQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(audit_logs.*)
        FROM audit_logs
        WHERE ($1::UUID IS NULL OR admin_id = $1)
          AND ($2::TEXT IS NULL OR entity_type = $2)
          AND ($3::UUID IS NULL OR entity_id = $3)
          AND ($4::TEXT IS NULL OR action = $4)
          AND ($5::TIMESTAMPTZ IS NULL OR timestamp >= $5)
          AND ($6::TIMESTAMPTZ IS NULL OR timestamp <= $6)
        ORDER BY timestamp DESC
        LIMIT $7
        "#,
    )
    .bind(query.admin_id)
    .bind(&query.entity_type)
    .bind(query.entity_id)
    .bind(&query.action)
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn list_kyc_audit_logs(
    state: web::Data<AppState>,
    query: web::Query<AuditLogQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(kyc_audit_log.*)
        FROM kyc_audit_log
        WHERE ($1::UUID IS NULL OR actor_id = $1)
          AND ($2::TEXT IS NULL OR action = $2)
          AND ($3::TIMESTAMPTZ IS NULL OR created_at >= $3)
          AND ($4::TIMESTAMPTZ IS NULL OR created_at <= $4)
        ORDER BY created_at DESC
        LIMIT $5
        "#,
    )
    .bind(query.admin_id)
    .bind(&query.action)
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

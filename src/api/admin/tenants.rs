use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TenantRequest {
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
    pub branding: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    pub status: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/tenants")
            .route("", web::get().to(list_tenants))
            .route("", web::post().to(create_tenant))
            .route("/{id}", web::put().to(update_tenant))
            .route("/{id}", web::delete().to(delete_tenant))
            .route("/{id}/merchants", web::get().to(list_tenant_merchants)),
    );
}

pub async fn list_tenants(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    let rows: Vec<serde_json::Value> =
        sqlx::query_scalar("SELECT to_jsonb(tenants.*) FROM tenants ORDER BY created_at DESC")
            .fetch_all(&state.db)
            .await?;
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn create_tenant(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<TenantRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        INSERT INTO tenants (name, slug, domain, branding, config, status)
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, 'active'))
        RETURNING to_jsonb(tenants.*)
        "#,
    )
    .bind(&body.name)
    .bind(&body.slug)
    .bind(&body.domain)
    .bind(body.branding.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(body.config.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(&body.status)
    .fetch_one(&state.db)
    .await?;
    let id = Uuid::parse_str(row["id"].as_str().unwrap_or_default()).ok();
    log_audit(
        &state.db,
        admin_id,
        "tenant_create",
        "tenant",
        id,
        None,
        Some(row.clone()),
        None,
    )
    .await?;
    Ok(HttpResponse::Created().json(row))
}

pub async fn update_tenant(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<TenantRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    let previous: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT to_jsonb(tenants.*) FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        UPDATE tenants
        SET name = $2, slug = $3, domain = $4, branding = $5, config = $6,
            status = COALESCE($7, status), updated_at = NOW()
        WHERE id = $1
        RETURNING to_jsonb(tenants.*)
        "#,
    )
    .bind(id)
    .bind(&body.name)
    .bind(&body.slug)
    .bind(&body.domain)
    .bind(body.branding.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(body.config.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(&body.status)
    .fetch_one(&state.db)
    .await?;
    log_audit(
        &state.db,
        admin_id,
        "tenant_update",
        "tenant",
        Some(id),
        previous,
        Some(row.clone()),
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(row))
}

pub async fn delete_tenant(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();
    let previous: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT to_jsonb(tenants.*) FROM tenants WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    sqlx::query("UPDATE tenants SET status = 'inactive', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    log_audit(
        &state.db,
        admin_id,
        "tenant_deactivate",
        "tenant",
        Some(id),
        previous,
        None,
        None,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_tenant_merchants(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let tenant_id = path.into_inner();
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(t)
        FROM (
            SELECT id, tenant_id, name, email, business_name, status, created_at, updated_at
            FROM merchants
            WHERE tenant_id = $1
            ORDER BY created_at DESC
        ) t
        "#,
    )
    .bind(tenant_id)
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(rows))
}

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audit_support::log_audit;
use crate::engine::services::notification_dispatcher::{dispatch_event, DispatchRequest};
use crate::error::FeelPayError;
use crate::models::user::Claims;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MerchantsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct MerchantsMeta {
    page: i64,
    per_page: i64,
    total: i64,
    total_pages: i64,
}

#[derive(Debug, Serialize)]
struct MerchantsListResponse {
    data: Vec<serde_json::Value>,
    meta: MerchantsMeta,
}

#[derive(Debug, Clone, Copy)]
enum MerchantAdminAction {
    Approve,
    Reject,
    Suspend,
}

impl MerchantAdminAction {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Suspend => "suspend",
        }
    }

    fn onboarding_status(&self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
            Self::Suspend => "suspended",
        }
    }

    fn merchant_status(&self) -> &'static str {
        match self {
            Self::Approve => "active",
            Self::Reject | Self::Suspend => "suspended",
        }
    }
}

fn actor_id_from_claims(claims: &Claims) -> Result<Uuid, FeelPayError> {
    Uuid::parse_str(&claims.sub)
        .map_err(|_| FeelPayError::Unauthorized("Invalid admin subject".into()))
}

fn ensure_admin_role(claims: &Claims) -> Result<(), FeelPayError> {
    match claims.role.as_str() {
        "super_admin" | "Super Admin" | "admin" => Ok(()),
        _ => Err(FeelPayError::Forbidden(
            "Admin role required for merchant management".into(),
        )),
    }
}

fn claims_tenant_scope(claims: &Claims) -> Result<Option<Uuid>, FeelPayError> {
    if claims.role == "tenant_admin" {
        return claims
            .tenant_id
            .ok_or_else(|| FeelPayError::Forbidden("Tenant admin missing tenant scope".into()))
            .map(Some);
    }
    Ok(None)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/merchants")
            .route("", web::get().to(list_merchants))
            .route("/{merchant_id}/approve", web::post().to(approve_merchant))
            .route("/{merchant_id}/reject", web::post().to(reject_merchant))
            .route("/{merchant_id}/suspend", web::post().to(suspend_merchant)),
    );
}

pub async fn list_merchants(
    state: web::Data<AppState>,
    claims: Claims,
    query: web::Query<MerchantsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    ensure_admin_role(&claims)?;
    let tenant_scope = claims_tenant_scope(&claims)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let status_filter = query.status.clone().filter(|s| !s.trim().is_empty());

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM merchants m
        WHERE
            ($1::text IS NULL OR m.onboarding_status = $1)
            AND ($2::uuid IS NULL OR m.tenant_id = $2)
        "#,
    )
    .bind(status_filter.as_deref())
    .bind(tenant_scope)
    .fetch_one(&state.db)
    .await?;

    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(t)
        FROM (
            SELECT
                m.id,
                m.name,
                m.email,
                m.business_name,
                m.business_type,
                COALESCE(
                    (
                        SELECT mk.kyc_status
                        FROM merchant_kyc mk
                        WHERE mk.merchant_id = m.id
                        ORDER BY mk.updated_at DESC NULLS LAST, mk.created_at DESC
                        LIMIT 1
                    ),
                    'pending'
                ) AS kyc_status,
                m.onboarding_status,
                m.created_at,
                0::bigint AS total_volume,
                0::bigint AS transaction_count
            FROM merchants m
            WHERE
                ($1::text IS NULL OR m.onboarding_status = $1)
                AND ($2::uuid IS NULL OR m.tenant_id = $2)
            ORDER BY m.created_at DESC
            LIMIT $3 OFFSET $4
        ) t
        "#,
    )
    .bind(status_filter.as_deref())
    .bind(tenant_scope)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total_pages = if total == 0 {
        0
    } else {
        (total + per_page - 1) / per_page
    };

    Ok(HttpResponse::Ok().json(MerchantsListResponse {
        data: rows,
        meta: MerchantsMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

pub async fn approve_merchant(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    update_merchant_status(
        state,
        claims,
        path.into_inner(),
        MerchantAdminAction::Approve,
    )
    .await
}

pub async fn reject_merchant(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    update_merchant_status(
        state,
        claims,
        path.into_inner(),
        MerchantAdminAction::Reject,
    )
    .await
}

pub async fn suspend_merchant(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    update_merchant_status(
        state,
        claims,
        path.into_inner(),
        MerchantAdminAction::Suspend,
    )
    .await
}

async fn update_merchant_status(
    state: web::Data<AppState>,
    claims: Claims,
    merchant_id: Uuid,
    action: MerchantAdminAction,
) -> Result<HttpResponse, FeelPayError> {
    ensure_admin_role(&claims)?;
    let tenant_scope = claims_tenant_scope(&claims)?;
    let actor_id = actor_id_from_claims(&claims)?;

    let previous: Option<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(m.*)
        FROM merchants m
        WHERE m.id = $1
          AND ($2::uuid IS NULL OR m.tenant_id = $2)
        "#,
    )
    .bind(merchant_id)
    .bind(tenant_scope)
    .fetch_optional(&state.db)
    .await?;

    if previous.is_none() {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    }

    let updated: serde_json::Value = sqlx::query_scalar(
        r#"
        UPDATE merchants
        SET onboarding_status = $3,
            review_status = $3,
            status = $4,
            review_last_decided_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
          AND ($2::uuid IS NULL OR tenant_id = $2)
        RETURNING to_jsonb(merchants.*)
        "#,
    )
    .bind(merchant_id)
    .bind(tenant_scope)
    .bind(action.onboarding_status())
    .bind(action.merchant_status())
    .fetch_one(&state.db)
    .await?;

    if matches!(
        action,
        MerchantAdminAction::Approve | MerchantAdminAction::Reject
    ) {
        if let Some(email) =
            sqlx::query_scalar::<_, Option<String>>("SELECT email FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_optional(&state.db)
                .await?
                .flatten()
        {
            let event_type = if matches!(action, MerchantAdminAction::Approve) {
                "kyc_approved"
            } else {
                "kyc_rejected"
            };
            let mut variables = std::collections::BTreeMap::new();
            variables.insert("status".to_string(), action.onboarding_status().to_string());
            let _ = dispatch_event(
                &state.db,
                DispatchRequest {
                    merchant_id: Some(merchant_id),
                    event_type: event_type.to_string(),
                    environment: if state.config.is_live() {
                        "live"
                    } else {
                        "test"
                    }
                    .to_string(),
                    recipient_email: Some(email),
                    recipient_phone: None,
                    variables,
                },
            )
            .await;
        }
    }

    log_audit(
        &state.db,
        actor_id,
        &format!("merchant_{}", action.as_str()),
        "merchant",
        Some(merchant_id),
        previous,
        Some(updated.clone()),
        None,
    )
    .await?;

    Ok(HttpResponse::Ok().json(updated))
}

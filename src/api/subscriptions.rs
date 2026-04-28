use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::services::subscription::{
    cancel_subscription, create_plan, create_subscription, get_subscription, list_invoices,
    list_plans, list_subscriptions, pause_subscription, resume_subscription, CreatePlanRequest,
    CreateSubscriptionRequest,
};
use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/plans", web::post().to(create_subscription_plan))
            .route("/plans", web::get().to(list_subscription_plans))
            .route("", web::post().to(create_subscription_handler))
            .route("", web::get().to(list_subscriptions_handler))
            .route("/{id}", web::get().to(get_subscription_handler))
            .route("/{id}/pause", web::post().to(pause_subscription_handler))
            .route("/{id}/resume", web::post().to(resume_subscription_handler))
            .route("/{id}/cancel", web::post().to(cancel_subscription_handler))
            .route("/{id}/invoices", web::get().to(list_subscription_invoices)),
    );
}

#[derive(Debug, Deserialize)]
pub struct CreatePlanBody {
    pub name: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: Uuid,
    pub name: String,
    pub amount: i64,
    pub currency: String,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
}

/// POST /v1/subscriptions/plans - Create a subscription plan
pub async fn create_subscription_plan(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreatePlanBody>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let plan = create_plan(
        &state.db,
        merchant_id,
        CreatePlanRequest {
            name: body.name.clone(),
            amount: body.amount,
            currency: body.currency.clone(),
            billing_cycle: body.billing_cycle.clone(),
            trial_days: body.trial_days,
        },
    )
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(PlanResponse {
        id: plan.id,
        name: plan.name,
        amount: plan.amount,
        currency: plan.currency,
        billing_cycle: plan.billing_cycle,
        trial_days: plan.trial_days,
    }))
}

/// GET /v1/subscriptions/plans - List subscription plans
pub async fn list_subscription_plans(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let plans = list_plans(&state.db, merchant_id)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let responses: Vec<PlanResponse> = plans
        .into_iter()
        .map(|p| PlanResponse {
            id: p.id,
            name: p.name,
            amount: p.amount,
            currency: p.currency,
            billing_cycle: p.billing_cycle,
            trial_days: p.trial_days,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionBody {
    pub customer_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub plan_name: Option<String>,
    pub amount: i64,
    pub currency: Option<String>,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub plan_name: String,
    pub amount: i64,
    pub currency: String,
    pub billing_cycle: String,
    pub status: String,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub trial_end: Option<String>,
    pub cancel_at_period_end: bool,
}

/// POST /v1/subscriptions - Create a subscription
pub async fn create_subscription_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateSubscriptionBody>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let sub = create_subscription(
        &state.db,
        merchant_id,
        CreateSubscriptionRequest {
            customer_id: body.customer_id,
            plan_id: body.plan_id,
            plan_name: body.plan_name.clone(),
            amount: body.amount,
            currency: body.currency.clone(),
            billing_cycle: body.billing_cycle.clone(),
            trial_days: body.trial_days,
            start_date: body.start_date,
        },
    )
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(SubscriptionResponse {
        id: sub.id,
        customer_id: sub.customer_id,
        plan_name: sub.plan_name,
        amount: sub.amount,
        currency: sub.currency,
        billing_cycle: sub.billing_cycle,
        status: sub.status,
        current_period_start: sub.current_period_start.map(|d| d.to_rfc3339()),
        current_period_end: sub.current_period_end.map(|d| d.to_rfc3339()),
        trial_end: sub.trial_end.map(|d| d.to_rfc3339()),
        cancel_at_period_end: sub.cancel_at_period_end,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn parse_pagination(limit: Option<i64>, offset: Option<i64>) -> Result<(i64, i64), FeelPayError> {
    let requested_limit = limit.unwrap_or(50);
    let requested_offset = offset.unwrap_or(0);

    if requested_limit <= 0 {
        return Err(FeelPayError::BadRequest(
            "limit must be greater than 0".to_string(),
        ));
    }

    if requested_offset < 0 {
        return Err(FeelPayError::BadRequest(
            "offset must be greater than or equal to 0".to_string(),
        ));
    }

    Ok((requested_limit.min(100), requested_offset))
}

/// GET /v1/subscriptions - List subscriptions
pub async fn list_subscriptions_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let (limit, offset) = parse_pagination(query.limit, query.offset)?;

    let subs = list_subscriptions(
        &state.db,
        merchant_id,
        query.status.as_deref(),
        limit,
        offset,
    )
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let responses: Vec<SubscriptionResponse> = subs
        .into_iter()
        .map(|s| SubscriptionResponse {
            id: s.id,
            customer_id: s.customer_id,
            plan_name: s.plan_name,
            amount: s.amount,
            currency: s.currency,
            billing_cycle: s.billing_cycle,
            status: s.status,
            current_period_start: s.current_period_start.map(|d| d.to_rfc3339()),
            current_period_end: s.current_period_end.map(|d| d.to_rfc3339()),
            trial_end: s.trial_end.map(|d| d.to_rfc3339()),
            cancel_at_period_end: s.cancel_at_period_end,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// GET /v1/subscriptions/{id} - Get subscription
pub async fn get_subscription_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let subscription_id = path.into_inner();

    let sub = get_subscription(&state.db, merchant_id, subscription_id)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match sub {
        Some(s) => Ok(HttpResponse::Ok().json(SubscriptionResponse {
            id: s.id,
            customer_id: s.customer_id,
            plan_name: s.plan_name,
            amount: s.amount,
            currency: s.currency,
            billing_cycle: s.billing_cycle,
            status: s.status,
            current_period_start: s.current_period_start.map(|d| d.to_rfc3339()),
            current_period_end: s.current_period_end.map(|d| d.to_rfc3339()),
            trial_end: s.trial_end.map(|d| d.to_rfc3339()),
            cancel_at_period_end: s.cancel_at_period_end,
        })),
        None => Err(FeelPayError::NotFound("Subscription not found".into())),
    }
}

/// POST /v1/subscriptions/{id}/pause - Pause subscription
pub async fn pause_subscription_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let subscription_id = path.into_inner();

    let sub = pause_subscription(&state.db, merchant_id, subscription_id)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(SubscriptionResponse {
        id: sub.id,
        customer_id: sub.customer_id,
        plan_name: sub.plan_name,
        amount: sub.amount,
        currency: sub.currency,
        billing_cycle: sub.billing_cycle,
        status: sub.status,
        current_period_start: sub.current_period_start.map(|d| d.to_rfc3339()),
        current_period_end: sub.current_period_end.map(|d| d.to_rfc3339()),
        trial_end: sub.trial_end.map(|d| d.to_rfc3339()),
        cancel_at_period_end: sub.cancel_at_period_end,
    }))
}

/// POST /v1/subscriptions/{id}/resume - Resume subscription
pub async fn resume_subscription_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let subscription_id = path.into_inner();

    let sub = resume_subscription(&state.db, merchant_id, subscription_id)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(SubscriptionResponse {
        id: sub.id,
        customer_id: sub.customer_id,
        plan_name: sub.plan_name,
        amount: sub.amount,
        currency: sub.currency,
        billing_cycle: sub.billing_cycle,
        status: sub.status,
        current_period_start: sub.current_period_start.map(|d| d.to_rfc3339()),
        current_period_end: sub.current_period_end.map(|d| d.to_rfc3339()),
        trial_end: sub.trial_end.map(|d| d.to_rfc3339()),
        cancel_at_period_end: sub.cancel_at_period_end,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CancelBody {
    pub cancel_at_period_end: Option<bool>,
}

/// POST /v1/subscriptions/{id}/cancel - Cancel subscription
pub async fn cancel_subscription_handler(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: Option<web::Json<CancelBody>>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let subscription_id = path.into_inner();
    let cancel_at_end = body
        .as_ref()
        .map(|b| b.cancel_at_period_end.unwrap_or(false))
        .unwrap_or(false);

    let sub = cancel_subscription(&state.db, merchant_id, subscription_id, cancel_at_end)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(SubscriptionResponse {
        id: sub.id,
        customer_id: sub.customer_id,
        plan_name: sub.plan_name,
        amount: sub.amount,
        currency: sub.currency,
        billing_cycle: sub.billing_cycle,
        status: sub.status,
        current_period_start: sub.current_period_start.map(|d| d.to_rfc3339()),
        current_period_end: sub.current_period_end.map(|d| d.to_rfc3339()),
        trial_end: sub.trial_end.map(|d| d.to_rfc3339()),
        cancel_at_period_end: sub.cancel_at_period_end,
    }))
}

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub invoice_number: String,
    pub due_date: Option<String>,
    pub paid_at: Option<String>,
}

/// GET /v1/subscriptions/{id}/invoices - List subscription invoices
pub async fn list_subscription_invoices(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let subscription_id = path.into_inner();
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let invoices = list_invoices(&state.db, merchant_id, Some(subscription_id), limit, offset)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let responses: Vec<InvoiceResponse> = invoices
        .into_iter()
        .map(|i| InvoiceResponse {
            id: i.id,
            subscription_id: i.subscription_id,
            amount: i.amount,
            currency: i.currency,
            status: i.status,
            invoice_number: i.invoice_number,
            due_date: i.due_date.map(|d| d.to_rfc3339()),
            paid_at: i.paid_at.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

// Helper function to extract merchant_id from request
fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| FeelPayError::Unauthorized("Merchant ID not found".into()))
}

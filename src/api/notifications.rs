use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::middleware::auth::MerchantId;
use crate::models::notification::{CreateNotificationRequest, CreateReportRequest, NotificationQueue, ReportJob, ReportJobResponse};
use crate::state::AppState;

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<MerchantId>()
        .map(|id| id.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Merchant ID not found".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

// Notifications API

pub async fn create_notification(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateNotificationRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let _merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state.db.acquire().await?;

    let result = sqlx::query(
        r#"
        INSERT INTO notification_queue (
            notification_type, recipient_type, recipient_id, recipient_email,
            recipient_phone, subject, body, template_id, template_data,
            scheduled_at, entity_type, entity_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
    )
    .bind(&body.notification_type)
    .bind(&body.recipient_type)
    .bind(body.recipient_id)
    .bind(&body.recipient_email)
    .bind(&body.recipient_phone)
    .bind(&body.subject)
    .bind(&body.body)
    .bind(&body.template_id)
    .bind(body.template_data.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(body.scheduled_at)
    .bind(&body.entity_type)
    .bind(body.entity_id)
    .map(NotificationQueue::from)
    .fetch_one(&mut *conn)
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": result.id,
        "status": result.status.to_string(),
        "message": "Notification queued successfully"
    })))
}

pub async fn list_notifications(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state.db.acquire().await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let notifications = sqlx::query(
        "SELECT * FROM notification_queue 
         WHERE recipient_id = $1 OR recipient_email = $2
         ORDER BY created_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(merchant_id)
    .bind(None::<String>)
    .bind(per_page as i64)
    .bind(offset)
    .map(NotificationQueue::from)
    .fetch_all(&mut *conn)
    .await?;

    Ok(HttpResponse::Ok().json(notifications))
}

// Reports API

pub async fn create_report(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateReportRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state.db.acquire().await?;

    let result = sqlx::query(
        r#"
        INSERT INTO report_jobs (
            merchant_id, report_type, date_from, date_to, filters,
            file_format, email_on_complete, email_recipients, requested_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(&body.report_type)
    .bind(body.date_from)
    .bind(body.date_to)
    .bind(body.filters.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(body.file_format.as_deref().unwrap_or("csv"))
    .bind(body.email_on_complete.unwrap_or(false))
    .bind(serde_json::json!(body.email_recipients.clone().unwrap_or_default()))
    .bind(merchant_id)
    .map(ReportJob::from)
    .fetch_one(&mut *conn)
    .await?;

    Ok(HttpResponse::Accepted().json(ReportJobResponse::from(result)))
}

pub async fn get_report(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let report_id = path.into_inner();
    let mut conn = state.db.acquire().await?;

    let result = sqlx::query("SELECT * FROM report_jobs WHERE id = $1 AND merchant_id = $2")
        .bind(report_id)
        .bind(merchant_id)
        .map(ReportJob::from)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| FeelPayError::NotFound("Report not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ReportJobResponse::from(result)))
}

pub async fn list_reports(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state.db.acquire().await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let reports = sqlx::query(
        "SELECT * FROM report_jobs WHERE merchant_id = $1 
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(merchant_id)
    .bind(per_page as i64)
    .bind(offset)
    .map(ReportJob::from)
    .fetch_all(&mut *conn)
    .await?;

    Ok(HttpResponse::Ok().json(
        reports
            .into_iter()
            .map(ReportJobResponse::from)
            .collect::<Vec<_>>(),
    ))
}


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notifications")
            .route("", web::post().to(create_notification))
            .route("", web::get().to(list_notifications)),
    )
    .service(
        web::scope("/reports")
            .route("", web::post().to(create_report))
            .route("", web::get().to(list_reports))
            .route("/{id}", web::get().to(get_report)),
    );
}
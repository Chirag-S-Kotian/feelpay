use crate::engine::services::report::{ReportService, CreateScheduledReportRequest, UpdateScheduledReportRequest};
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Request to create a scheduled report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledReportApiRequest {
    pub report_type: String,
    pub frequency: String,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub time_of_day: String,
    pub recipient_emails: Vec<String>,
    pub format: String,
}

/// Scheduled report response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReportResponse {
    pub id: String,
    pub report_type: String,
    pub frequency: String,
    pub time_of_day: String,
    pub recipient_emails: Vec<String>,
    pub format: String,
    pub is_active: bool,
    pub next_run_at: Option<String>,
    pub created_at: String,
}

/// Scheduled reports list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReportsListResponse {
    pub reports: Vec<ScheduledReportResponse>,
    pub total: i64,
}

/// Create scheduled report endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/scheduled-reports",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    request_body = CreateScheduledReportApiRequest,
    responses(
        (status = 200, description = "Scheduled report created", body = ScheduledReportResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_scheduled_report(
    merchant_id: web::Path<Uuid>,
    req: web::Json<CreateScheduledReportApiRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let report_service = ReportService::new(db.get_ref().clone());

    let created_by_user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| FeelPayError::Unauthorized("Invalid user identity in token".into()))?;

    let create_req = CreateScheduledReportRequest {
        report_type: req.report_type.clone(),
        frequency: req.frequency.clone(),
        day_of_week: req.day_of_week,
        day_of_month: req.day_of_month,
        time_of_day: req.time_of_day.clone(),
        recipient_emails: req.recipient_emails.clone(),
        format: req.format.clone(),
    };

    let report = report_service
        .create_scheduled_report(merchant_id, create_req, Some(created_by_user_id))
        .await?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse {
        id: report.id.to_string(),
        report_type: report.report_type.as_str().to_string(),
        frequency: report.frequency.as_str().to_string(),
        time_of_day: report.time_of_day.to_string(),
        recipient_emails: report.recipient_emails,
        format: report.format.as_str().to_string(),
        is_active: report.is_active,
        next_run_at: report.next_run_at.map(|dt| dt.to_rfc3339()),
        created_at: report.created_at.to_rfc3339(),
    }))
}

/// List scheduled reports endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/scheduled-reports",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("limit" = Option<i64>, Query, description = "Limit"),
        ("offset" = Option<i64>, Query, description = "Offset")
    ),
    responses(
        (status = 200, description = "Scheduled reports list", body = ScheduledReportsListResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_scheduled_reports(
    merchant_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let report_service = ReportService::new(db.get_ref().clone());
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let (reports, total) = report_service
        .list_scheduled_reports(merchant_id, limit, offset)
        .await?;

    let items = reports
        .into_iter()
        .map(|report| ScheduledReportResponse {
            id: report.id.to_string(),
            report_type: report.report_type.as_str().to_string(),
            frequency: report.frequency.as_str().to_string(),
            time_of_day: report.time_of_day.to_string(),
            recipient_emails: report.recipient_emails,
            format: report.format.as_str().to_string(),
            is_active: report.is_active,
            next_run_at: report.next_run_at.map(|dt| dt.to_rfc3339()),
            created_at: report.created_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ScheduledReportsListResponse {
        reports: items,
        total,
    }))
}

/// Get scheduled report endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/scheduled-reports/{report_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("report_id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Scheduled report", body = ScheduledReportResponse),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_scheduled_report(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, report_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let report_service = ReportService::new(db.get_ref().clone());
    let report = report_service
        .get_scheduled_report(merchant_id, report_id)
        .await?
        .ok_or_else(|| FeelPayError::NotFound("Scheduled report not found".into()))?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse {
        id: report.id.to_string(),
        report_type: report.report_type.as_str().to_string(),
        frequency: report.frequency.as_str().to_string(),
        time_of_day: report.time_of_day.to_string(),
        recipient_emails: report.recipient_emails,
        format: report.format.as_str().to_string(),
        is_active: report.is_active,
        next_run_at: report.next_run_at.map(|dt| dt.to_rfc3339()),
        created_at: report.created_at.to_rfc3339(),
    }))
}

/// Update scheduled report endpoint
#[utoipa::path(
    put,
    path = "/api/v1/merchants/{merchant_id}/scheduled-reports/{report_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("report_id" = Uuid, Path, description = "Report ID")
    ),
    request_body = UpdateScheduledReportApiRequest,
    responses(
        (status = 200, description = "Scheduled report updated", body = ScheduledReportResponse),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn update_scheduled_report(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateScheduledReportApiRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, report_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let report_service = ReportService::new(db.get_ref().clone());

    let update_req = UpdateScheduledReportRequest {
        frequency: req.frequency.clone(),
        day_of_week: req.day_of_week,
        day_of_month: req.day_of_month,
        time_of_day: req.time_of_day.clone(),
        recipient_emails: req.recipient_emails.clone(),
        format: req.format.clone(),
        is_active: req.is_active,
    };

    let report = report_service
        .update_scheduled_report(merchant_id, report_id, update_req)
        .await?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse {
        id: report.id.to_string(),
        report_type: report.report_type.as_str().to_string(),
        frequency: report.frequency.as_str().to_string(),
        time_of_day: report.time_of_day.to_string(),
        recipient_emails: report.recipient_emails,
        format: report.format.as_str().to_string(),
        is_active: report.is_active,
        next_run_at: report.next_run_at.map(|dt| dt.to_rfc3339()),
        created_at: report.created_at.to_rfc3339(),
    }))
}

/// Delete scheduled report endpoint
#[utoipa::path(
    delete,
    path = "/api/v1/merchants/{merchant_id}/scheduled-reports/{report_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("report_id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 204, description = "Scheduled report deleted"),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn delete_scheduled_report(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, report_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let report_service = ReportService::new(db.get_ref().clone());
    report_service
        .delete_scheduled_report(merchant_id, report_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Update scheduled report request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledReportApiRequest {
    pub frequency: Option<String>,
    pub day_of_week: Option<i32>,
    pub day_of_month: Option<i32>,
    pub time_of_day: Option<String>,
    pub recipient_emails: Option<Vec<String>>,
    pub format: Option<String>,
    pub is_active: Option<bool>,
}

/// Pagination query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

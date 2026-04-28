use crate::engine::services::export::{
    CreateExportJobRequest, ExportJobType, ExportService, PaginationParams,
};
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Request to create a transaction export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionExportRequest {
    pub date_range: Option<DateRangeRequest>,
    pub format: String, // "csv" or "excel"
    pub filters: Option<serde_json::Value>,
    pub columns: Option<Vec<String>>,
    pub idempotency_key: Option<String>,
}

/// Date range request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeRequest {
    pub start: String, // ISO 8601 format
    pub end: String,   // ISO 8601 format
}

/// Export job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJobResponse {
    pub job_id: String,
    pub status: String,
    pub created_at: String,
}

/// Export progress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgressResponse {
    pub job_id: String,
    pub status: String,
    pub progress: Option<ProgressInfo>,
    pub created_at: String,
    pub started_at: Option<String>,
}

/// Progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub percent: i32,
    pub records_processed: i32,
    pub estimated_time_remaining_seconds: i32,
}

/// Export list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportListResponse {
    pub exports: Vec<ExportItemResponse>,
    pub total: i64,
}

/// Export item in list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportItemResponse {
    pub job_id: String,
    pub job_type: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub record_count: Option<i32>,
    pub file_size_bytes: Option<i64>,
}

/// Create transaction export endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/exports/transactions",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    request_body = CreateTransactionExportRequest,
    responses(
        (status = 200, description = "Export job created", body = ExportJobResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_transaction_export(
    merchant_id: web::Path<Uuid>,
    req: web::Json<CreateTransactionExportRequest>,
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

    let export_service = ExportService::new(db.get_ref().clone());

    let created_by_user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| FeelPayError::Unauthorized("Invalid user identity in token".into()))?;

    let mut export_filters = match req.filters.clone() {
        Some(serde_json::Value::Object(map)) => map,
        Some(other) => {
            let mut map = serde_json::Map::new();
            map.insert("filters".to_string(), other);
            map
        }
        None => serde_json::Map::new(),
    };

    if let Some(date_range) = &req.date_range {
        export_filters.insert(
            "date_range".to_string(),
            serde_json::to_value(date_range).map_err(|e| {
                FeelPayError::Internal(format!("Failed to serialize export date range: {}", e))
            })?,
        );
    }

    if let Some(columns) = &req.columns {
        export_filters.insert(
            "columns".to_string(),
            serde_json::to_value(columns).map_err(|e| {
                FeelPayError::Internal(format!("Failed to serialize export columns: {}", e))
            })?,
        );
    }

    // Parse filters
    let filters = if export_filters.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(export_filters))
    };

    let create_req = CreateExportJobRequest {
        job_type: ExportJobType::Transaction,
        filters,
        idempotency_key: req.idempotency_key.clone(),
        created_by_user_id: Some(created_by_user_id),
    };

    let job = export_service
        .create_export_job(merchant_id, create_req)
        .await?;

    Ok(HttpResponse::Ok().json(ExportJobResponse {
        job_id: job.id.to_string(),
        status: job.status.as_str().to_string(),
        created_at: job.created_at.to_rfc3339(),
    }))
}

/// Get export job status endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/exports/{job_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("job_id" = Uuid, Path, description = "Export job ID")
    ),
    responses(
        (status = 200, description = "Export job status", body = ExportProgressResponse),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_export_status(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, job_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let export_service = ExportService::new(db.get_ref().clone());
    let job = export_service
        .get_export_job(merchant_id, job_id)
        .await?
        .ok_or_else(|| FeelPayError::NotFound("Export job not found".into()))?;

    Ok(HttpResponse::Ok().json(ExportProgressResponse {
        job_id: job.id.to_string(),
        status: job.status.as_str().to_string(),
        progress: None, // TODO: Calculate progress from database
        created_at: job.created_at.to_rfc3339(),
        started_at: job.started_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// Download export endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/exports/{job_id}/download",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("job_id" = Uuid, Path, description = "Export job ID")
    ),
    responses(
        (status = 302, description = "Redirect to signed S3 URL"),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn download_export(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, job_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let export_service = ExportService::new(db.get_ref().clone());
    let job = export_service
        .get_export_job(merchant_id, job_id)
        .await?
        .ok_or_else(|| FeelPayError::NotFound("Export job not found".into()))?;

    if let Some(file_url) = job.file_url {
        Ok(HttpResponse::Found()
            .insert_header(("Location", file_url))
            .finish())
    } else {
        Err(FeelPayError::BadRequest(
            "Export file not yet available".into(),
        ))
    }
}

/// List export jobs endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/exports",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("limit" = Option<i64>, Query, description = "Limit"),
        ("offset" = Option<i64>, Query, description = "Offset")
    ),
    responses(
        (status = 200, description = "Export jobs list", body = ExportListResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_exports(
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

    let export_service = ExportService::new(db.get_ref().clone());
    let pagination = PaginationParams {
        limit: query.limit.unwrap_or(20),
        offset: query.offset.unwrap_or(0),
    };

    let result = export_service
        .list_export_jobs(merchant_id, pagination)
        .await?;

    let exports = result
        .items
        .into_iter()
        .map(|job| ExportItemResponse {
            job_id: job.id.to_string(),
            job_type: job.job_type.as_str().to_string(),
            status: job.status.as_str().to_string(),
            created_at: job.created_at.to_rfc3339(),
            completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
            record_count: job.record_count,
            file_size_bytes: job.file_size_bytes,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ExportListResponse {
        exports,
        total: result.total,
    }))
}

/// Pagination query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

//! KYC Document Viewer API
//!
//! Provides endpoints for viewing KYC documents with audit logging

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::document::Document;
use crate::models::user::Claims;
use crate::state::AppState;
use utoipa::ToSchema;

/// Document metadata structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentMetadata {
    pub file_name: String,
    pub file_size: i64,
    pub uploaded_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

/// Response structure for document viewing
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentViewResponse {
    pub document_id: Uuid,
    pub document_type: String,
    pub content_type: String,
    pub content: String, // Base64 encoded
    pub page_count: Option<i32>,
    pub metadata: DocumentMetadata,
}

/// Fetch document from storage (S3 or local)
/// This is a placeholder - in production, this would connect to S3
async fn fetch_document_content(storage_key: &str) -> Result<Vec<u8>, FeelPayError> {
    // In a real implementation, this would:
    // 1. Connect to S3 using the storage configuration
    // 2. Download the file content
    // 3. Return the bytes

    // For now, we'll simulate by reading from local storage if available
    // or returning an error indicating S3 is not configured

    tracing::info!("Fetching document from storage key: {}", storage_key);

    // Check if it's a local file path (for development)
    if std::path::Path::new(storage_key).exists() {
        tokio::fs::read(storage_key)
            .await
            .map_err(|e| FeelPayError::Internal(format!("Failed to read file: {}", e)))
    } else {
        // In production, this would be S3
        Err(FeelPayError::NotFound(
            "Document storage not configured. Configure S3 in production.".into(),
        ))
    }
}

/// Count pages in a PDF document using page-tree markers.
async fn count_pdf_pages(content: &[u8]) -> Result<i32, FeelPayError> {
    let text = String::from_utf8_lossy(content);
    let count =
        text.matches("/Type /Page").count() as i32 - text.matches("/Type /Pages").count() as i32;
    Ok(count.max(0))
}

/// Log document view to KYC audit log
async fn log_document_view(
    pool: &PgPool,
    document_id: Uuid,
    merchant_id: Uuid,
    admin_id: Uuid,
) -> Result<(), FeelPayError> {
    // Check if kyc_audit_log table exists, if not create entry in audit_logs
    let result = sqlx::query(
        r#"
        INSERT INTO kyc_audit_log (id, merchant_id, action, actor_id, new_state, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(merchant_id)
    .bind("document_viewed")
    .bind(admin_id)
    .bind(serde_json::json!({ "document_id": document_id }))
    .execute(pool)
    .await;

    // If the table doesn't exist, fall back to audit_logs
    if result.is_err() {
        tracing::warn!("kyc_audit_log table not found, using audit_logs");

        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, timestamp, admin_id, action, entity_type, entity_id, new_value)
            VALUES ($1, NOW(), $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind("document_viewed")
        .bind("document")
        .bind(document_id)
        .bind(serde_json::json!({ "document_id": document_id, "merchant_id": merchant_id }))
        .execute(pool)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;
    }

    Ok(())
}

/// Get document for viewing
pub async fn get_document_for_viewing(
    pool: &PgPool,
    document_id: Uuid,
    admin_id: Uuid,
) -> Result<DocumentViewResponse, FeelPayError> {
    // Fetch document metadata from database
    let doc = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, merchant_id, uploaded_by, document_type, document_purpose,
               file_name, file_type, file_size, storage_key, storage_bucket,
               checksum, status, rejection_reason, verified_at, verified_by,
               expires_at, metadata, created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Document not found".into()))?;

    // Fetch document content from storage
    let content = fetch_document_content(&doc.storage_key).await?;

    // Log the view action to KYC audit log
    log_document_view(pool, document_id, doc.merchant_id, admin_id).await?;

    // For PDFs, count pages
    let page_count = if doc.file_type == "application/pdf" || doc.file_type.ends_with("/pdf") {
        match count_pdf_pages(&content).await {
            Ok(count) if count > 0 => Some(count),
            _ => None,
        }
    } else {
        None
    };

    // Build the response
    Ok(DocumentViewResponse {
        document_id: doc.id,
        document_type: doc.document_type,
        content_type: doc.file_type,
        content: BASE64.encode(&content),
        page_count,
        metadata: DocumentMetadata {
            file_name: doc.file_name,
            file_size: doc.file_size,
            uploaded_by: doc.uploaded_by,
            created_at: doc.created_at,
            status: doc.status,
        },
    })
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// GET /api/admin/documents/{id}/view - View a document
#[utoipa::path(
    get,
    path = "/api/admin/documents/{id}/view",
    responses(
        (status = 200, description = "Document view response", body = DocumentViewResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Document not found")
    ),
    security(
        ("admin_jwt" = [])
    )
)]
pub async fn view_document(
    state: web::Data<AppState>,
    req: HttpRequest,
    document_id: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    // Verify admin auth from JWT claims
    let extensions = req.extensions();
    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| FeelPayError::Unauthorized("Authentication required".into()))?;

    // Check admin role
    if claims.role != "super_admin" && claims.role != "admin" && claims.role != "support" {
        return Err(FeelPayError::Forbidden("Admin access required".into()));
    }

    let document_id = document_id.into_inner();
    let admin_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| FeelPayError::Unauthorized("Invalid admin subject".into()))?;

    let response = get_document_for_viewing(&state.db, document_id, admin_id).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Configure KYC viewer routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin/documents").route("/{id}/view", web::get().to(view_document)));
}

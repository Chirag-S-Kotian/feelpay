use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::document::{
    ApprovalRequest, CreateApprovalRequest, Document, DocumentResponse, GenerateUploadUrlRequest,
    OtpResponse, OtpVerification, ProcessApprovalRequest, SendOtpRequest, UploadDocumentRequest,
    UploadUrlResponse, VerifyDocumentRequest, VerifyOtpRequest,
};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/documents")
            .route("", web::post().to(upload_document))
            .route("", web::get().to(list_documents))
            .route("/upload-url", web::post().to(generate_upload_url))
            .route("/{id}", web::get().to(get_document))
            .route("/{id}/verify", web::post().to(verify_document))
            .route("/{id}", web::delete().to(delete_document))
            // OTP endpoints
            .route("/otp/send", web::post().to(send_otp))
            .route("/otp/verify", web::post().to(verify_otp))
            // Approval workflow
            .route("/approvals", web::get().to(list_approvals))
            .route("/approvals", web::post().to(create_approval))
            .route("/approvals/{id}", web::get().to(get_approval))
            .route("/approvals/{id}/process", web::post().to(process_approval)),
    );
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

fn user_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::UserId>()
        .map(|u| u.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing user context".into()))
}

/// Generate a presigned upload URL for document upload
pub async fn generate_upload_url(
    req: HttpRequest,
    _state: web::Data<AppState>,
    body: web::Json<GenerateUploadUrlRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Validate file size (max 10MB)
    if body.file_size > 10 * 1024 * 1024 {
        return Err(FeelPayError::BadRequest("File size exceeds 10MB limit".into()));
    }

    // Validate file type
    let allowed_types = [
        "image/jpeg",
        "image/png",
        "image/gif",
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];
    if !allowed_types.contains(&body.file_type.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Invalid file type. Allowed: JPEG, PNG, GIF, PDF, DOC, DOCX".into(),
        ));
    }

    // Generate storage key
    let storage_key = format!(
        "merchants/{}/{}/{}-{}",
        merchant_id,
        body.document_purpose,
        chrono::Utc::now().timestamp(),
        sanitize_filename(&body.file_name)
    );

    // In production, generate a presigned URL from S3/GCS
    // For now, return a mock URL
    let upload_url = format!(
        "https://storage.feelpay.example.com/upload/{}",
        storage_key
    );

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    Ok(HttpResponse::Ok().json(UploadUrlResponse {
        upload_url,
        storage_key: storage_key.clone(),
        expires_at,
    }))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

/// Upload document metadata after file is uploaded to storage
pub async fn upload_document(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UploadDocumentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let user_id = user_id_from_req(&req)?;

    let doc: Document = sqlx::query_as(
        r#"
        INSERT INTO documents (
            merchant_id, uploaded_by, document_type, document_purpose,
            file_name, file_type, file_size, storage_key, storage_bucket,
            checksum, metadata, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending_review')
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(user_id)
    .bind(&body.document_type)
    .bind(&body.document_purpose)
    .bind(&body.file_name)
    .bind(&body.file_type)
    .bind(body.file_size)
    .bind(&body.storage_key)
    .bind(&body.storage_bucket)
    .bind(&body.checksum)
    .bind(body.metadata.as_ref().unwrap_or(&serde_json::json!({})))
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(DocumentResponse {
        id: doc.id,
        document_type: doc.document_type,
        document_purpose: doc.document_purpose,
        file_name: doc.file_name,
        file_type: doc.file_type,
        file_size: doc.file_size,
        status: doc.status,
        created_at: doc.created_at,
        download_url: None,
    }))
}

/// List documents for the current merchant
pub async fn list_documents(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListDocumentsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let rows = sqlx::query(
        r#"
        SELECT id, document_type, document_purpose, file_name, file_type, file_size, 
               status, created_at, storage_key
        FROM documents
        WHERE merchant_id = $1 AND ($2::text IS NULL OR document_purpose = $2)
        ORDER BY created_at DESC
        "#,
    )
    .bind(merchant_id)
    .bind(&query.purpose)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let docs: Vec<DocumentResponse> = rows
        .into_iter()
        .map(|row| DocumentResponse {
            id: row.get(0),
            document_type: row.get(1),
            document_purpose: row.get(2),
            file_name: row.get(3),
            file_type: row.get(4),
            file_size: row.get(5),
            status: row.get(6),
            created_at: row.get(7),
            download_url: None,
        })
        .collect();

    Ok(HttpResponse::Ok().json(docs))
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub purpose: Option<String>,
}

/// Get a specific document
pub async fn get_document(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let doc_id = path.into_inner();

    let doc: Option<Document> = sqlx::query_as(
        "SELECT * FROM documents WHERE id = $1 AND merchant_id = $2",
    )
    .bind(doc_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match doc {
        Some(d) => {
            // Generate download URL (in production, use presigned URL)
            let download_url = format!(
                "https://storage.feelpay.example.com/download/{}",
                d.storage_key
            );

            Ok(HttpResponse::Ok().json(DocumentResponse {
                id: d.id,
                document_type: d.document_type,
                document_purpose: d.document_purpose,
                file_name: d.file_name,
                file_type: d.file_type,
                file_size: d.file_size,
                status: d.status,
                created_at: d.created_at,
                download_url: Some(download_url),
            }))
        }
        None => Err(FeelPayError::NotFound("Document not found".into())),
    }
}

/// Verify or reject a document (admin action)
pub async fn verify_document(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<VerifyDocumentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let _user_id = user_id_from_req(&req)?;
    let doc_id = path.into_inner();

    if !["approved", "rejected"].contains(&body.status.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Status must be 'approved' or 'rejected'".into(),
        ));
    }

    let result = sqlx::query(
        r#"
        UPDATE documents
        SET status = $1, 
            rejection_reason = $2,
            verified_at = NOW(),
            verified_by = $3,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, merchant_id, document_type
        "#,
    )
    .bind(&body.status)
    .bind(&body.rejection_reason)
    .bind(_user_id)
    .bind(doc_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match result {
        Some(row) => {
            let merchant_id: Uuid = row.get(1);
            let doc_type: String = row.get(2);

            // Fetch merchant email for notification
            let merchant_email: String = sqlx::query_scalar("SELECT email FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&state.db)
                .await
                .unwrap_or_default();

            if !merchant_email.is_empty() {
                // 1. Create InApp notification
                let notification_title = if body.status == "approved" {
                    format!("Document Approved: {}", doc_type)
                } else {
                    format!("Document Rejected: {}", doc_type)
                };

                let notification_message = if body.status == "approved" {
                    format!("Your document ({}) has been successfully verified.", doc_type)
                } else {
                    format!(
                        "Your document ({}) was rejected. Reason: {}",
                        doc_type,
                        body.rejection_reason.as_deref().unwrap_or("No reason provided")
                    )
                };

                sqlx::query(
                    r#"
                    INSERT INTO notification_queue (
                        notification_type, recipient_type, recipient_id, recipient_email,
                        subject, body, status, entity_type, entity_id, metadata
                    )
                    VALUES ('in_app', 'merchant', $1, $2, $3, $4, 'sent', 'document', $5, $6)
                    "#
                )
                .bind(merchant_id)
                .bind(&merchant_email)
                .bind(&notification_title)
                .bind(&notification_message)
                .bind(doc_id)
                .bind(serde_json::json!({
                    "status": body.status,
                    "document_type": doc_type
                }))
                .execute(&state.db)
                .await
                .ok();

                // 2. Trigger KYC status email if purpose is KYC
                let _ = crate::engine::services::email::send_kyc_status_email(
                    &state.db,
                    &merchant_email,
                    &body.status,
                    body.rejection_reason.as_deref()
                ).await;
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Document {}", body.status)
            })))
        }
        None => Err(FeelPayError::NotFound("Document not found".into())),
    }
}

/// Delete a document
pub async fn delete_document(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let doc_id = path.into_inner();

    let result = sqlx::query("DELETE FROM documents WHERE id = $1 AND merchant_id = $2")
        .bind(doc_id)
        .bind(merchant_id)
        .execute(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Document deleted"
        })))
    } else {
        Err(FeelPayError::NotFound("Document not found".into()))
    }
}

// ============ OTP Verification ============

/// Send OTP to email or phone
pub async fn send_otp(
    state: web::Data<AppState>,
    body: web::Json<SendOtpRequest>,
) -> Result<HttpResponse, FeelPayError> {
    // Validate that either email or phone is provided
    if body.email.is_none() && body.phone.is_none() {
        return Err(FeelPayError::BadRequest(
            "Either email or phone must be provided".into(),
        ));
    }

    // Generate 6-digit OTP
    let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

    // Hash the OTP for storage
    let mut hasher = Sha256::new();
    hasher.update(otp.as_bytes());
    let otp_hash = format!("{:x}", hasher.finalize());

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

    // Store OTP
    sqlx::query(
        r#"
        INSERT INTO otp_verifications (email, phone, otp_hash, otp_type, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&otp_hash)
    .bind(&body.otp_type)
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // In production, send OTP via email/SMS service
    // For now, return the OTP in response (remove in production)
    tracing::info!("OTP generated: {} for {:?} {:?}", otp, body.email, body.phone);

    Ok(HttpResponse::Ok().json(OtpResponse {
        message: "OTP sent successfully".to_string(),
        expires_in_seconds: 600,
    }))
}

/// Verify OTP
pub async fn verify_otp(
    state: web::Data<AppState>,
    body: web::Json<VerifyOtpRequest>,
) -> Result<HttpResponse, FeelPayError> {
    // Validate that either email or phone is provided
    if body.email.is_none() && body.phone.is_none() {
        return Err(FeelPayError::BadRequest(
            "Either email or phone must be provided".into(),
        ));
    }

    // Hash the provided OTP
    let mut hasher = Sha256::new();
    hasher.update(body.otp.as_bytes());
    let otp_hash = format!("{:x}", hasher.finalize());

    // Find and verify OTP
    let row = sqlx::query_as::<_, OtpVerification>(
        r#"
        SELECT * FROM otp_verifications
        WHERE (email = $1 OR phone = $2)
          AND otp_type = $3
          AND status = 'pending'
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&body.otp_type)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some(otp_record) => {
            // Check attempts
            if otp_record.attempts >= otp_record.max_attempts {
                return Err(FeelPayError::BadRequest(
                    "Maximum attempts exceeded. Please request a new OTP.".into(),
                ));
            }

            // Verify OTP
            if otp_record.otp_hash == otp_hash {
                // Mark as verified
                sqlx::query(
                    "UPDATE otp_verifications SET status = 'verified', verified_at = NOW() WHERE id = $1"
                )
                .bind(otp_record.id)
                .execute(&state.db)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "verified": true,
                    "message": "OTP verified successfully"
                })))
            } else {
                // Increment attempts
                sqlx::query(
                    "UPDATE otp_verifications SET attempts = attempts + 1 WHERE id = $1"
                )
                .bind(otp_record.id)
                .execute(&state.db)
                .await
                .ok();

                Err(FeelPayError::BadRequest("Invalid OTP".into()))
            }
        }
        None => Err(FeelPayError::BadRequest(
            "OTP not found or expired. Please request a new OTP.".into(),
        )),
    }
}

// ============ Approval Workflow ============

/// List approval requests
pub async fn list_approvals(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListApprovalsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let _user_id = user_id_from_req(&req)?;

    let rows = sqlx::query_as::<_, ApprovalRequest>(
        r#"
        SELECT * FROM approval_requests
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR entity_type = $2)
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&query.status)
    .bind(&query.entity_type)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(rows))
}

#[derive(Debug, Deserialize)]
pub struct ListApprovalsQuery {
    pub status: Option<String>,
    pub entity_type: Option<String>,
}

/// Create an approval request
pub async fn create_approval(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateApprovalRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let user_id = user_id_from_req(&req)?;

    let approval: ApprovalRequest = sqlx::query_as(
        r#"
        INSERT INTO approval_requests (entity_type, entity_id, requested_by, request_type, request_data, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&body.entity_type)
    .bind(body.entity_id)
    .bind(user_id)
    .bind(&body.request_type)
    .bind(&body.request_data)
    .bind(&body.notes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Log the creation
    sqlx::query(
        "INSERT INTO approval_audit_log (approval_request_id, action, actor_id, new_status) VALUES ($1, 'created', $2, 'pending')"
    )
    .bind(approval.id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .ok();

    Ok(HttpResponse::Created().json(approval))
}

/// Get an approval request
pub async fn get_approval(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let approval_id = path.into_inner();

    let approval: Option<ApprovalRequest> = sqlx::query_as(
        "SELECT * FROM approval_requests WHERE id = $1",
    )
    .bind(approval_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match approval {
        Some(a) => Ok(HttpResponse::Ok().json(a)),
        None => Err(FeelPayError::NotFound("Approval request not found".into())),
    }
}

/// Process an approval request (approve or reject)
pub async fn process_approval(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<ProcessApprovalRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let user_id = user_id_from_req(&req)?;
    let approval_id = path.into_inner();

    if !["approve", "reject"].contains(&body.action.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Action must be 'approve' or 'reject'".into(),
        ));
    }

    let new_status = if body.action == "approve" {
        "approved"
    } else {
        "rejected"
    };

    let result = sqlx::query(
        r#"
        UPDATE approval_requests
        SET status = $1, approved_by = $2, approved_at = NOW(), 
            rejection_reason = $3, notes = COALESCE($4, notes), updated_at = NOW()
        WHERE id = $5 AND status = 'pending'
        RETURNING id
        "#,
    )
    .bind(new_status)
    .bind(user_id)
    .bind(&body.rejection_reason)
    .bind(&body.notes)
    .bind(approval_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match result {
        Some(_) => {
            // Log the action
            sqlx::query(
                "INSERT INTO approval_audit_log (approval_request_id, action, actor_id, new_status, notes) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(approval_id)
            .bind(&body.action)
            .bind(user_id)
            .bind(new_status)
            .bind(&body.notes)
            .execute(&state.db)
            .await
            .ok();

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Approval request {}", new_status)
            })))
        }
        None => Err(FeelPayError::BadRequest(
            "Approval request not found or already processed".into(),
        )),
    }
}

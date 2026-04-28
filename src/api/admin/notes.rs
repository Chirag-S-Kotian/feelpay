//! Merchant Notes API
//!
//! CRUD operations for merchant internal notes

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::merchant_note::{
    CreateNoteRequest, MerchantNote, MerchantNoteHistory, UpdateNoteRequest,
};

const MAX_NOTE_LENGTH: usize = 5000;

/// Create a new merchant note
pub async fn create_merchant_note(
    pool: &PgPool,
    merchant_id: Uuid,
    author_id: Uuid,
    req: CreateNoteRequest,
) -> Result<MerchantNote, FeelPayError> {
    // Validate content length
    if req.content.len() > MAX_NOTE_LENGTH {
        return Err(FeelPayError::BadRequest(format!(
            "Note exceeds {} character limit",
            MAX_NOTE_LENGTH
        )));
    }

    // Get author name from admin_users table
    let author_name = get_admin_name(pool, author_id).await?;

    // Insert the note
    let note = sqlx::query_as::<_, MerchantNote>(
        r#"
        INSERT INTO merchant_notes (id, merchant_id, content, author_id, author_name, created_at, updated_at, is_deleted)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), FALSE)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(merchant_id)
    .bind(&req.content)
    .bind(author_id)
    .bind(&author_name)
    .fetch_one(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Audit log the creation
    create_audit_log(
        pool,
        "merchant_note",
        note.id,
        "created",
        author_id,
        None,
        Some(serde_json::to_value(&note).unwrap_or_default()),
        None,
    )
    .await?;

    Ok(note)
}

/// Update an existing merchant note
pub async fn update_merchant_note(
    pool: &PgPool,
    note_id: Uuid,
    author_id: Uuid,
    req: UpdateNoteRequest,
) -> Result<MerchantNote, FeelPayError> {
    // Validate content length
    if req.content.len() > MAX_NOTE_LENGTH {
        return Err(FeelPayError::BadRequest(format!(
            "Note exceeds {} character limit",
            MAX_NOTE_LENGTH
        )));
    }

    // Get the current note to preserve history
    let current_note = sqlx::query_as::<_, MerchantNote>(
        "SELECT * FROM merchant_notes WHERE id = $1 AND is_deleted = FALSE",
    )
    .bind(note_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Note not found".into()))?;

    // Save to history before updating
    sqlx::query(
        r#"
        INSERT INTO merchant_note_history (id, note_id, previous_content, changed_by, changed_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(note_id)
    .bind(&current_note.content)
    .bind(author_id)
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Update the note
    let updated_note = sqlx::query_as::<_, MerchantNote>(
        r#"
        UPDATE merchant_notes
        SET content = $1, updated_at = NOW()
        WHERE id = $2 AND is_deleted = FALSE
        RETURNING *
        "#,
    )
    .bind(&req.content)
    .bind(note_id)
    .fetch_one(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Audit log the update
    create_audit_log(
        pool,
        "merchant_note",
        note_id,
        "updated",
        author_id,
        Some(serde_json::to_value(&current_note).unwrap_or_default()),
        Some(serde_json::to_value(&updated_note).unwrap_or_default()),
        None,
    )
    .await?;

    Ok(updated_note)
}

/// Soft-delete a merchant note
pub async fn delete_merchant_note(pool: &PgPool, note_id: Uuid) -> Result<(), FeelPayError> {
    // Check if note exists
    let note = sqlx::query_as::<_, MerchantNote>(
        "SELECT * FROM merchant_notes WHERE id = $1 AND is_deleted = FALSE",
    )
    .bind(note_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Note not found".into()))?;

    // Soft delete - mark as deleted but keep for audit
    sqlx::query(
        r#"
        UPDATE merchant_notes
        SET is_deleted = TRUE, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(note_id)
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Audit log the deletion (we don't have author_id for delete, use system)
    create_audit_log(
        pool,
        "merchant_note",
        note_id,
        "deleted",
        note.author_id, // Use original author as the actor
        Some(serde_json::to_value(&note).unwrap_or_default()),
        None,
        Some("Soft deleted by admin"),
    )
    .await?;

    Ok(())
}

/// List all notes for a merchant (excluding deleted)
pub async fn list_merchant_notes(
    pool: &PgPool,
    merchant_id: Uuid,
) -> Result<Vec<MerchantNote>, FeelPayError> {
    let notes = sqlx::query_as::<_, MerchantNote>(
        r#"
        SELECT * FROM merchant_notes
        WHERE merchant_id = $1 AND is_deleted = FALSE
        ORDER BY created_at DESC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(notes)
}

/// Get note history for a note
pub async fn get_note_history(
    pool: &PgPool,
    note_id: Uuid,
) -> Result<Vec<MerchantNoteHistory>, FeelPayError> {
    let history = sqlx::query_as::<_, MerchantNoteHistory>(
        r#"
        SELECT * FROM merchant_note_history
        WHERE note_id = $1
        ORDER BY changed_at DESC
        "#,
    )
    .bind(note_id)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(history)
}

/// Get admin name by ID
async fn get_admin_name(pool: &PgPool, admin_id: Uuid) -> Result<String, FeelPayError> {
    let result: Option<(String,)> = sqlx::query_as("SELECT name FROM admin_users WHERE id = $1")
        .bind(admin_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // If admin not found, return a default name
    // This handles cases where the author might be a system user
    Ok(result
        .map(|(name,)| name)
        .unwrap_or_else(|| "System".to_string()))
}

/// Create audit log entry
async fn create_audit_log(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    actor_id: Uuid,
    previous_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    reason: Option<&str>,
) -> Result<(), FeelPayError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, timestamp, admin_id, action, entity_type, entity_id, previous_value, new_value, reason)
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(actor_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(previous_value)
    .bind(new_value)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(())
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Create note handler
pub async fn create_note(
    pool: web::Data<PgPool>,
    merchant_id: web::Path<Uuid>,
    req: web::Json<CreateNoteRequest>,
    admin_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let note =
        create_merchant_note(&pool, merchant_id.into_inner(), *admin_id, req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(note))
}

/// Update note handler
pub async fn update_note(
    pool: web::Data<PgPool>,
    note_id: web::Path<Uuid>,
    req: web::Json<UpdateNoteRequest>,
    admin_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let note =
        update_merchant_note(&pool, note_id.into_inner(), *admin_id, req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(note))
}

/// Delete note handler
pub async fn delete_note(
    pool: web::Data<PgPool>,
    note_id: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    delete_merchant_note(&pool, note_id.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// List notes handler
pub async fn list_notes(
    pool: web::Data<PgPool>,
    merchant_id: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let notes = list_merchant_notes(&pool, merchant_id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(notes))
}

/// Get note history handler
pub async fn get_note_history_handler(
    pool: web::Data<PgPool>,
    note_id: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let history = get_note_history(&pool, note_id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(history))
}

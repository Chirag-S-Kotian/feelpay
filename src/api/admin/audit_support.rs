use actix_web::{HttpMessage, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::FeelPayError;

pub fn admin_id_from_claims(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    let extensions = req.extensions();
    let claims = extensions
        .get::<crate::models::user::Claims>()
        .ok_or_else(|| FeelPayError::Unauthorized("Authentication required".into()))?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| FeelPayError::Unauthorized("Invalid admin subject".into()))
}

pub async fn log_audit(
    pool: &PgPool,
    admin_id: Uuid,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    previous_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    reason: Option<&str>,
) -> Result<(), FeelPayError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs
            (admin_id, action, entity_type, entity_id, previous_value, new_value, reason)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(admin_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(previous_value)
    .bind(new_value)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(FeelPayError::from)?;

    Ok(())
}

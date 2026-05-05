use actix_web::{web, HttpRequest, HttpResponse};
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UpsertNotificationTemplateRequest {
    pub event_type: String,
    pub channel: String, // email|sms|whatsapp
    pub subject: Option<String>,
    pub body: String,
    pub is_active: Option<bool>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/notification-templates")
            .route("", web::get().to(list_templates))
            .route("", web::post().to(create_template))
            .route("/{id}", web::get().to(get_template))
            .route("/{id}", web::put().to(update_template))
            .route("/{id}", web::delete().to(delete_template)),
    );
}

fn normalize_channel(channel: &str) -> Result<String, FeelPayError> {
    let c = channel.trim().to_lowercase();
    match c.as_str() {
        "email" | "sms" | "whatsapp" => Ok(c),
        _ => Err(FeelPayError::BadRequest(
            "Invalid channel. Must be one of: email, sms, whatsapp".into(),
        )),
    }
}

fn normalize_event_type(event_type: &str) -> Result<String, FeelPayError> {
    let e = event_type.trim().to_lowercase();
    if e.is_empty() || e.len() > 50 {
        return Err(FeelPayError::BadRequest("Invalid event_type".into()));
    }
    Ok(e)
}

fn extract_variables(subject: Option<&str>, body: &str) -> Vec<String> {
    // Match {{var}} or {{ var_name }}
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").expect("regex compile");
    let mut vars: Vec<String> = vec![];

    for cap in re.captures_iter(body) {
        if let Some(m) = cap.get(1) {
            vars.push(m.as_str().to_string());
        }
    }
    if let Some(s) = subject {
        for cap in re.captures_iter(s) {
            if let Some(m) = cap.get(1) {
                vars.push(m.as_str().to_string());
            }
        }
    }

    vars.sort();
    vars.dedup();
    vars
}

pub async fn list_templates(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(notification_templates.*)
        FROM notification_templates
        ORDER BY event_type, channel
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": rows })))
}

pub async fn get_template(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let id = path.into_inner();
    let row: Option<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT to_jsonb(notification_templates.*)
        FROM notification_templates
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Template not found".into()));
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": row })))
}

pub async fn create_template(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UpsertNotificationTemplateRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;

    let event_type = normalize_event_type(&body.event_type)?;
    let channel = normalize_channel(&body.channel)?;
    let variables = extract_variables(body.subject.as_deref(), &body.body);
    let is_active = body.is_active.unwrap_or(true);

    let created: serde_json::Value = sqlx::query_scalar(
        r#"
        INSERT INTO notification_templates (event_type, channel, subject, body, variables, is_active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING to_jsonb(notification_templates.*)
        "#,
    )
    .bind(&event_type)
    .bind(&channel)
    .bind(&body.subject)
    .bind(&body.body)
    .bind(&variables)
    .bind(is_active)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("notification_templates_event_type_channel_key")
            || e.to_string().contains("unique")
        {
            FeelPayError::Conflict("Template already exists for event_type + channel".into())
        } else {
            FeelPayError::Database(e.to_string())
        }
    })?;

    log_audit(
        &state.db,
        admin_id,
        "notification_template_create",
        "notification_template",
        None,
        None,
        Some(created.clone()),
        Some("create"),
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({ "data": created })))
}

pub async fn update_template(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpsertNotificationTemplateRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();

    let previous: serde_json::Value = sqlx::query_scalar(
        "SELECT to_jsonb(notification_templates.*) FROM notification_templates WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| FeelPayError::NotFound("Template not found".into()))?;

    let event_type = normalize_event_type(&body.event_type)?;
    let channel = normalize_channel(&body.channel)?;
    let variables = extract_variables(body.subject.as_deref(), &body.body);
    let is_active = body.is_active.unwrap_or(true);

    let updated: serde_json::Value = sqlx::query_scalar(
        r#"
        UPDATE notification_templates
        SET event_type = $2,
            channel = $3,
            subject = $4,
            body = $5,
            variables = $6,
            is_active = $7,
            updated_at = NOW()
        WHERE id = $1
        RETURNING to_jsonb(notification_templates.*)
        "#,
    )
    .bind(id)
    .bind(&event_type)
    .bind(&channel)
    .bind(&body.subject)
    .bind(&body.body)
    .bind(&variables)
    .bind(is_active)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string()
            .contains("notification_templates_event_type_channel_key")
            || e.to_string().contains("unique")
        {
            FeelPayError::Conflict("Template already exists for event_type + channel".into())
        } else {
            FeelPayError::Database(e.to_string())
        }
    })?;

    log_audit(
        &state.db,
        admin_id,
        "notification_template_update",
        "notification_template",
        Some(id),
        Some(previous),
        Some(updated.clone()),
        Some("update"),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": updated })))
}

pub async fn delete_template(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let admin_id = admin_id_from_claims(&req)?;
    let id = path.into_inner();

    let previous: serde_json::Value = sqlx::query_scalar(
        "SELECT to_jsonb(notification_templates.*) FROM notification_templates WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| FeelPayError::NotFound("Template not found".into()))?;

    sqlx::query("DELETE FROM notification_templates WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    log_audit(
        &state.db,
        admin_id,
        "notification_template_delete",
        "notification_template",
        Some(id),
        Some(previous),
        None,
        Some("delete"),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })))
}

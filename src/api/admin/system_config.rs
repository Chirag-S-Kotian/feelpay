use actix_web::{web, HttpRequest, HttpResponse};
use redis::Commands;
use serde::Deserialize;

use super::audit_support::{admin_id_from_claims, log_audit};
use crate::error::FeelPayError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SystemConfigRequest {
    pub value: serde_json::Value,
    pub description: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/system-config")
            .route("", web::get().to(list_config))
            .route("/{key}", web::put().to(upsert_config)),
    );
}

pub async fn list_config(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    if let Ok(mut conn) = state.redis.get_connection() {
        let cached: redis::RedisResult<String> = conn.get("system_config:all");
        if let Ok(value) = cached {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value) {
                return Ok(HttpResponse::Ok().json(json));
            }
        }
    }

    let rows: Vec<serde_json::Value> =
        sqlx::query_scalar("SELECT to_jsonb(system_config.*) FROM system_config ORDER BY key")
            .fetch_all(&state.db)
            .await?;
    if let Ok(mut conn) = state.redis.get_connection() {
        let _: redis::RedisResult<()> = conn.set_ex(
            "system_config:all",
            serde_json::to_string(&rows).unwrap_or_default(),
            300,
        );
    }
    Ok(HttpResponse::Ok().json(rows))
}

pub async fn upsert_config(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SystemConfigRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let actor = admin_id_from_claims(&req)?;
    let key = path.into_inner();
    let previous: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT to_jsonb(system_config.*) FROM system_config WHERE key = $1")
            .bind(&key)
            .fetch_optional(&state.db)
            .await?;
    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        INSERT INTO system_config (key, value, description, updated_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (key)
        DO UPDATE SET value = EXCLUDED.value,
                      description = EXCLUDED.description,
                      updated_by = EXCLUDED.updated_by,
                      updated_at = NOW()
        RETURNING to_jsonb(system_config.*)
        "#,
    )
    .bind(&key)
    .bind(&body.value)
    .bind(&body.description)
    .bind(actor)
    .fetch_one(&state.db)
    .await?;
    if let Ok(mut conn) = state.redis.get_connection() {
        let _: redis::RedisResult<()> = conn.del("system_config:all");
        let _: redis::RedisResult<()> = conn.del(format!("system_config:{}", key));
        if key == "landing_page" {
            let _: redis::RedisResult<()> = conn.del("public:landing_page");
        }
        let _: redis::RedisResult<()> = conn.set_ex(
            format!("system_config:{}", key),
            serde_json::to_string(&row).unwrap_or_default(),
            300,
        );
    }
    log_audit(
        &state.db,
        actor,
        "system_config_upsert",
        "system_config",
        None,
        previous,
        Some(row.clone()),
        Some(&key),
    )
    .await?;
    Ok(HttpResponse::Ok().json(row))
}

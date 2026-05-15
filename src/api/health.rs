use crate::state::AppState;
use actix_web::{web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/health").route("", web::get().to(health_check)));
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "FeelPay Payment Gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn readiness_check(state: web::Data<AppState>) -> HttpResponse {
    let database_ready = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    let redis_ready = state.redis.get_connection().is_ok();
    let ready = database_ready && (!state.config.is_live() || redis_ready);

    HttpResponse::build(if ready {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    })
    .json(serde_json::json!({
        "status": if ready { "ready" } else { "not_ready" },
        "database": if database_ready { "ready" } else { "unavailable" },
        "redis": if redis_ready { "ready" } else { "unavailable" },
    }))
}

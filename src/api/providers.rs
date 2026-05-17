use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::provider::{
    CreateRoutingRuleRequest, PaymentProvider, ProviderHealthResponse, ProviderRoutingRule,
};
use crate::state::AppState;

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

// List all providers (health dashboard)
pub async fn list_providers(state: web::Data<AppState>) -> Result<HttpResponse, FeelPayError> {
    let providers_rows =
        sqlx::query("SELECT * FROM payment_providers ORDER BY priority DESC, is_default DESC")
            .fetch_all(&state.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let providers: Vec<PaymentProvider> = providers_rows
        .into_iter()
        .map(PaymentProvider::from)
        .collect();

    Ok(HttpResponse::Ok().json(
        providers
            .into_iter()
            .map(ProviderHealthResponse::from)
            .collect::<Vec<_>>(),
    ))
}

// Get provider by ID
pub async fn get_provider(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let provider_id = path.into_inner();

    let provider_row = sqlx::query("SELECT * FROM payment_providers WHERE id = $1")
        .bind(provider_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?
        .ok_or_else(|| FeelPayError::NotFound("Provider not found".to_string()))?;

    Ok(
        HttpResponse::Ok().json(ProviderHealthResponse::from(PaymentProvider::from(
            provider_row,
        ))),
    )
}

// Create routing rule
pub async fn create_routing_rule(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateRoutingRuleRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let _merchant_id = merchant_id_from_req(&req)?;

    let rule_row = sqlx::query(
        r#"
        INSERT INTO provider_routing_rules (
            merchant_id, payment_method, card_brand, 
            amount_min, amount_max, provider_id, priority
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(body.merchant_id)
    .bind(&body.payment_method)
    .bind(&body.card_brand)
    .bind(body.amount_min)
    .bind(body.amount_max)
    .bind(body.provider_id)
    .bind(body.priority.unwrap_or(100))
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(ProviderRoutingRule::from(rule_row)))
}

// List routing rules
pub async fn list_routing_rules(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let rules_rows = sqlx::query(
        "SELECT * FROM provider_routing_rules WHERE merchant_id = $1 OR merchant_id IS NULL ORDER BY priority",
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let rules: Vec<ProviderRoutingRule> = rules_rows
        .into_iter()
        .map(ProviderRoutingRule::from)
        .collect();

    Ok(HttpResponse::Ok().json(rules))
}

// Update routing rule
pub async fn update_routing_rule(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateRoutingRuleRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let rule_id = path.into_inner();

    let rule_row = sqlx::query(
        r#"
        UPDATE provider_routing_rules SET
            payment_method = $1,
            card_brand = $2,
            amount_min = $3,
            amount_max = $4,
            provider_id = $5,
            priority = $6,
            updated_at = NOW()
        WHERE id = $7 AND (merchant_id = $8 OR merchant_id IS NULL)
        RETURNING *
        "#,
    )
    .bind(&body.payment_method)
    .bind(&body.card_brand)
    .bind(body.amount_min)
    .bind(body.amount_max)
    .bind(body.provider_id)
    .bind(body.priority.unwrap_or(100))
    .bind(rule_id)
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(ProviderRoutingRule::from(rule_row)))
}

// Delete routing rule
pub async fn delete_routing_rule(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let rule_id = path.into_inner();

    sqlx::query(
        "DELETE FROM provider_routing_rules WHERE id = $1 AND (merchant_id = $2 OR merchant_id IS NULL)",
    )
    .bind(rule_id)
    .bind(merchant_id)
    .execute(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

// Get provider health stats
pub async fn get_provider_stats(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let provider_id = path.into_inner();

    // Get recent health logs (last 100)
    let logs: Vec<(bool, i32)> = sqlx::query_as(
        "SELECT is_success, latency_ms FROM provider_health_logs 
         WHERE provider_id = $1 AND created_at > NOW() - INTERVAL '24 hours'
         ORDER BY created_at DESC LIMIT 100",
    )
    .bind(provider_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let total = logs.len() as f64;
    let success_count = logs.iter().filter(|(s, _)| *s).count() as f64;
    let avg_latency = if logs.is_empty() {
        0
    } else {
        logs.iter().map(|(_, l)| l).sum::<i32>() / logs.len() as i32
    };
    let success_rate = if total > 0.0 {
        (success_count / total) * 100.0
    } else {
        100.0
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success_rate": success_rate,
        "avg_latency_ms": avg_latency,
        "total_checks": total,
        "successful_checks": success_count,
        "failed_checks": total - success_count
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("", web::get().to(list_providers))
            .route("/{id}", web::get().to(get_provider))
            .route("/{id}/stats", web::get().to(get_provider_stats))
            .service(
                web::scope("/routing-rules")
                    .route("", web::post().to(create_routing_rule))
                    .route("", web::get().to(list_routing_rules))
                    .route("/{id}", web::put().to(update_routing_rule))
                    .route("/{id}", web::delete().to(delete_routing_rule)),
            ),
    );
}

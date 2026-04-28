use crate::error::FeelPayError;
use crate::models::mandate::{CreateMandateRequest, Mandate};
use crate::state::AppState;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mandates")
            .route("", web::post().to(initiate_mandate))
            .route("", web::get().to(list_mandates))
            .route("/{id}", web::get().to(get_mandate))
            .route("/{id}/cancel", web::post().to(cancel_mandate))
            .route("/{id}/execute", web::post().to(execute_mandate)),
    );
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

#[utoipa::path(
    post,
    path = "/v1/mandates",
    request_body = CreateMandateRequest,
    responses(
        (status = 200, description = "Mandate initiated successfully", body = Mandate),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn initiate_mandate(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateMandateRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Generate a unique mandate reference (UMN for UPI)
    let umn = format!(
        "feelpay.{}@bank",
        Uuid::new_v4().to_string()[..8].to_lowercase()
    );

    let mandate = sqlx::query_as::<_, Mandate>(
        r#"
        INSERT INTO mandates (
            merchant_id, customer_id, mandate_type, mandate_reference, 
            amount_limit, currency, frequency, end_date, payer_vpa, payer_name, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(body.customer_id)
    .bind(&body.mandate_type)
    .bind(&umn)
    .bind(body.amount_limit)
    .bind(body.currency.as_deref().unwrap_or("INR"))
    .bind(&body.frequency)
    .bind(body.end_date)
    .bind(&body.payer_vpa)
    .bind(&body.payer_name)
    .bind(&body.metadata)
    .fetch_one(&state.db)
    .await?;

    // Simulate NPCI Mandate creation delay and auto-approval for demo
    let db = state.db.clone();
    let mandate_id = mandate.id;
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let _ = sqlx::query("UPDATE mandates SET status = 'active' WHERE id = $1")
            .bind(mandate_id)
            .execute(&db)
            .await;
    });

    Ok(HttpResponse::Ok().json(mandate))
}

#[utoipa::path(
    get,
    path = "/v1/mandates",
    responses(
        (status = 200, description = "List of mandates", body = [Mandate]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_mandates(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let mandates = sqlx::query_as::<_, Mandate>(
        "SELECT * FROM mandates WHERE merchant_id = $1 ORDER BY created_at DESC",
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(mandates))
}

#[utoipa::path(
    get,
    path = "/v1/mandates/{id}",
    params(
        ("id" = Uuid, Path, description = "Mandate ID")
    ),
    responses(
        (status = 200, description = "Mandate details", body = Mandate),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mandate not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_mandate(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mandate_id = path.into_inner();

    let mandate =
        sqlx::query_as::<_, Mandate>("SELECT * FROM mandates WHERE id = $1 AND merchant_id = $2")
            .bind(mandate_id)
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await?;

    match mandate {
        Some(m) => Ok(HttpResponse::Ok().json(m)),
        None => Err(FeelPayError::NotFound("Mandate not found".into())),
    }
}

#[utoipa::path(
    post,
    path = "/v1/mandates/{id}/cancel",
    params(
        ("id" = Uuid, Path, description = "Mandate ID")
    ),
    responses(
        (status = 200, description = "Mandate cancelled successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mandate not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn cancel_mandate(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mandate_id = path.into_inner();

    let res = sqlx::query("UPDATE mandates SET status = 'revoked', updated_at = NOW() WHERE id = $1 AND merchant_id = $2")
        .bind(mandate_id)
        .bind(merchant_id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() == 0 {
        return Err(FeelPayError::NotFound("Mandate not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "revoked" })))
}

#[utoipa::path(
    post,
    path = "/v1/mandates/{id}/execute",
    params(
        ("id" = Uuid, Path, description = "Mandate ID")
    ),
    request_body(content = Object, description = "Execution details like amount"),
    responses(
        (status = 200, description = "Mandate executed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mandate not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn execute_mandate(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mandate_id = path.into_inner();

    let amount = body.get("amount").and_then(|v| v.as_i64()).ok_or_else(|| {
        FeelPayError::BadRequest("Amount is required for mandate execution".into())
    })?;

    // Verify mandate is active
    let mandate =
        sqlx::query_as::<_, Mandate>("SELECT * FROM mandates WHERE id = $1 AND merchant_id = $2")
            .bind(mandate_id)
            .bind(merchant_id)
            .fetch_optional(&state.db)
            .await?;

    let m = mandate.ok_or_else(|| FeelPayError::NotFound("Mandate not found".into()))?;

    if m.status != "active" {
        return Err(FeelPayError::BadRequest(format!(
            "Mandate is not active (current status: {})",
            m.status
        )));
    }

    if amount > m.amount_limit {
        return Err(FeelPayError::BadRequest(format!(
            "Amount exceeds mandate limit of {}",
            m.amount_limit
        )));
    }

    // In a real system, we would call the UPI provider's execution API here.
    // For now, we simulate a successful transaction.

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "captured",
        "transaction_id": Uuid::new_v4(),
        "amount": amount,
        "mandate_id": mandate_id
    })))
}

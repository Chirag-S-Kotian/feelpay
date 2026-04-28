use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::Row;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::payment_link::{
    CreatePaymentLinkRequest, ListPaymentLinksQuery, PaymentLink, PaymentLinkResponse,
};
use crate::state::AppState;

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|id| id.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Merchant ID not found".to_string()))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentLinkListResponse {
    pub data: Vec<PaymentLinkResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
}

// Create a new payment link
#[utoipa::path(
    post,
    path = "/v1/payment-links",
    request_body = CreatePaymentLinkRequest,
    responses(
        (status = 201, description = "Payment link created successfully", body = PaymentLinkResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn create_payment_link(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreatePaymentLinkRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Generate unique link_id
    let link_id = generate_link_id();
    let base_url = std::env::var("PAYMENT_LINK_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let short_url = format!("{}/pay/{}", base_url, link_id);

    // Calculate expiry (default 24 hours)
    let expires_at = body
        .expires_at
        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(24));

    let row = sqlx::query(
        r#"
        INSERT INTO payment_links (
            merchant_id, link_id, amount, currency, description, short_url,
            expires_at, status, customer_name, customer_email, customer_phone,
            allow_partial, min_partial_amount, max_partial_amount, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9, $10, $11, $12, $13, $14)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(&link_id)
    .bind(body.amount)
    .bind(body.currency.as_deref().unwrap_or("INR"))
    .bind(&body.description)
    .bind(short_url)
    .bind(expires_at)
    .bind(&body.customer_name)
    .bind(&body.customer_email)
    .bind(&body.customer_phone)
    .bind(body.allow_partial.unwrap_or(false))
    .bind(body.min_partial_amount)
    .bind(body.max_partial_amount)
    .bind(body.metadata.as_ref().unwrap_or(&serde_json::json!({})))
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(PaymentLinkResponse::from(PaymentLink::from(row))))
}

// Get payment link by ID
#[utoipa::path(
    get,
    path = "/v1/payment-links/{id}",
    params(
        ("id" = Uuid, Path, description = "Payment Link ID")
    ),
    responses(
        (status = 200, description = "Payment link details", body = PaymentLinkResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Payment link not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_payment_link(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let link_id = path.into_inner();
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let result = sqlx::query(
        "SELECT * FROM payment_links WHERE id = $1 AND merchant_id = $2 AND deleted_at IS NULL",
    )
    .bind(link_id)
    .bind(merchant_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment link not found".to_string()))?;

    Ok(HttpResponse::Ok().json(PaymentLinkResponse::from(PaymentLink::from(result))))
}

// List payment links
#[utoipa::path(
    get,
    path = "/v1/payment-links",
    params(
        ListPaymentLinksQuery
    ),
    responses(
        (status = 200, description = "List of payment links", body = PaymentLinkListResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_payment_links(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ListPaymentLinksQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Build query
    let mut sql =
        "SELECT * FROM payment_links WHERE merchant_id = $1 AND deleted_at IS NULL".to_string();
    let mut count_sql =
        "SELECT COUNT(*) FROM payment_links WHERE merchant_id = $1 AND deleted_at IS NULL"
            .to_string();
    let mut param_idx = 2;

    if query.status.is_some() {
        sql.push_str(&format!(" AND status = ${}", param_idx));
        count_sql.push_str(&format!(" AND status = ${}", param_idx));
        param_idx += 1;
    }

    sql.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));

    // Get total count
    let total_row = sqlx::query(&count_sql).bind(merchant_id);

    let total_row = if let Some(ref status) = query.status {
        total_row.bind(status)
    } else {
        total_row
    };

    let total: i64 = total_row
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?
        .get(0);

    // Get results
    let mut links_query = sqlx::query(&sql).bind(merchant_id);

    if let Some(ref status) = query.status {
        links_query = links_query.bind(status);
    }

    let links_rows = links_query
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let links: Vec<PaymentLink> = links_rows.into_iter().map(PaymentLink::from).collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i32;

    Ok(HttpResponse::Ok().json(PaymentLinkListResponse {
        data: links.into_iter().map(PaymentLinkResponse::from).collect(),
        meta: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

// Cancel/expire a payment link
#[utoipa::path(
    post,
    path = "/v1/payment-links/{id}/cancel",
    params(
        ("id" = Uuid, Path, description = "Payment Link ID")
    ),
    responses(
        (status = 200, description = "Payment link cancelled successfully"),
        (status = 404, description = "Payment link not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn cancel_payment_link(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let link_id = path.into_inner();
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let result = sqlx::query(
        "UPDATE payment_links SET status = 'cancelled', updated_at = NOW() 
         WHERE id = $1 AND merchant_id = $2 AND status = 'active' AND deleted_at IS NULL
         RETURNING id",
    )
    .bind(link_id)
    .bind(merchant_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if result.is_none() {
        return Err(FeelPayError::NotFound(
            "Payment link not found or already cancelled".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Payment link cancelled successfully"
    })))
}

// Public endpoint - view payment link (for customer)
pub async fn view_payment_link(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, FeelPayError> {
    let link_id = path.into_inner();
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let result: Option<sqlx::postgres::PgRow> = sqlx::query(
        "UPDATE payment_links SET view_count = view_count + 1 
         WHERE link_id = $1 AND status = 'active' AND (expires_at IS NULL OR expires_at > NOW())
         RETURNING id, amount, currency, description, customer_name, customer_email, customer_phone,
                   allow_partial, min_partial_amount, max_partial_amount, status",
    )
    .bind(&link_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match result {
        Some(row) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "link_id": link_id,
            "amount": row.get::<i64, _>("amount"),
            "currency": row.get::<String, _>("currency"),
            "description": row.get::<Option<String>, _>("description"),
            "customer_name": row.get::<Option<String>, _>("customer_name"),
            "customer_email": row.get::<Option<String>, _>("customer_email"),
            "customer_phone": row.get::<Option<String>, _>("customer_phone"),
            "allow_partial": row.get::<bool, _>("allow_partial"),
            "min_partial_amount": row.get::<Option<i64>, _>("min_partial_amount"),
            "max_partial_amount": row.get::<Option<i64>, _>("max_partial_amount"),
            "status": row.get::<String, _>("status"),
        }))),
        None => Err(FeelPayError::NotFound(
            "Payment link not found or expired".to_string(),
        )),
    }
}

fn generate_link_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    (0..14)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payment-links")
            // Merchant-facing endpoints (require auth)
            .route("", web::post().to(create_payment_link))
            .route("", web::get().to(list_payment_links))
            .route("/{id}", web::get().to(get_payment_link))
            .route("/{id}/cancel", web::post().to(cancel_payment_link))
            // Public endpoint
            .route("/public/{link_id}", web::get().to(view_payment_link)),
    );
}

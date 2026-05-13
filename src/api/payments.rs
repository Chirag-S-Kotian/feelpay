use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::pagination::{build_cursor, PaginatedResponse, PaginationParams};
use crate::engine::processor::{MethodData, TransactionContext};
use crate::error::FeelPayError;
use crate::models::payment_order::{PaymentOrder, PaymentStatus};
use crate::state::AppState;

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "amount": 1000,
    "currency": "INR",
    "order_id": "ORD-12345",
    "customer_email": "customer@example.com",
    "customer_phone": "+919999999999",
    "description": "Test card only - not real payment data",
    "return_url": "https://merchant.com/return",
    "method_data": {
        "method": "card",
        "data": {
            "card_number": "4111111111111111",
            "expiry_month": 12,
            "expiry_year": 2025,
            "cvv": "123"
        }
    }
}))]
pub struct CreatePaymentRequest {
    /// Amount in paise (e.g., 1000 = ₹10)
    pub amount: i64,
    /// Currency code (INR)
    pub currency: String,
    /// Merchant's order ID
    pub order_id: String,
    /// Customer email
    #[schema(value_type = Option<String>)]
    pub customer_email: Option<String>,
    /// Customer phone
    #[schema(value_type = Option<String>)]
    pub customer_phone: Option<String>,
    /// Payment description
    #[schema(value_type = Option<String>)]
    pub description: Option<String>,
    /// URL to redirect after payment
    #[schema(value_type = Option<String>)]
    pub return_url: Option<String>,
    /// Payment method data
    pub method_data: MethodData,
    /// Whether to vault the card for future payments
    #[serde(default)]
    pub vault_card: bool,
    /// Override default processor
    #[schema(value_type = Option<String>)]
    pub processor_override: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("", web::post().to(create_payment))
            .route("", web::get().to(list_payments))
            .route("/{payment_id}", web::get().to(get_payment))
            .route("/{id}/details", web::get().to(get_payment_details))
            .route("/{id}/timeline", web::get().to(get_payment_timeline))
            .route("/{payment_id}/capture", web::post().to(capture_payment))
            .route("/{payment_id}/refund", web::post().to(refund_payment))
            .route("/batch-refund", web::post().to(batch_refund)),
    );
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ListPaymentsQuery {
    pub status: Option<String>,
    // Legacy pagination (deprecated)
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    // Cursor-based pagination (flat params for reliability with query extractors)
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ListRefundsQuery {
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct CapturePaymentRequest {
    pub amount: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RefundPaymentRequest {
    pub amount: i64,
    pub reason: Option<String>,
}

#[derive(sqlx::FromRow)]
struct IdempotencyRecord {
    request_hash: String,
    response_code: Option<i32>,
    response_body: Option<serde_json::Value>,
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .or_else(|| {
            req.headers()
                .get("x-merchant-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant ID context".into()))
}

fn tenant_id_from_req(req: &HttpRequest) -> Option<Uuid> {
    req.extensions()
        .get::<crate::middleware::auth::TenantId>()
        .map(|tenant| tenant.0)
}

fn actor_id_from_req(req: &HttpRequest, fallback_merchant_id: Uuid) -> String {
    req.extensions()
        .get::<crate::middleware::auth::UserId>()
        .map(|user| user.0.to_string())
        .unwrap_or_else(|| fallback_merchant_id.to_string())
}

fn request_hash<T: serde::Serialize>(body: &T) -> Result<String, FeelPayError> {
    let payload = serde_json::to_vec(body)
        .map_err(|e| FeelPayError::BadRequest(format!("Invalid request payload: {}", e)))?;
    let digest = Sha256::digest(payload);
    Ok(hex::encode(digest))
}

fn payment_method_name(method_data: &MethodData) -> &'static str {
    match method_data {
        MethodData::Card { .. } => "card",
        MethodData::Upi { .. } => "upi",
        MethodData::NetBanking { .. } => "netbanking",
        MethodData::Wallet { .. } => "wallet",
        MethodData::Emi { .. } => "emi",
        MethodData::Bnpl { .. } => "bnpl",
        MethodData::Simulator { .. } => "simulator",
    }
}

fn connector_for_payment(
    method_data: &MethodData,
    processor_override: Option<&str>,
    is_live: bool,
) -> String {
    if let Some(connector) = processor_override.filter(|value| !value.trim().is_empty()) {
        return connector.to_string();
    }
    if !is_live {
        return "simulator".to_string();
    }
    payment_method_name(method_data).to_string()
}

fn capture_connector(connector: Option<&str>) -> &str {
    connector
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("simulator")
}

fn status_timestamps(
    status: &PaymentStatus,
    now: chrono::DateTime<chrono::Utc>,
) -> (
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
) {
    match status {
        PaymentStatus::Authorized => (Some(now), None),
        PaymentStatus::Captured => (Some(now), Some(now)),
        _ => (None, None),
    }
}

fn payment_metadata(environment: &str, method: &str, connector: &str) -> serde_json::Value {
    serde_json::json!({
        "environment": environment,
        "payment_method": method,
        "connector": connector,
        "mode_badge": if environment == "live" { "LIVE" } else { "TEST" }
    })
}

async fn check_idempotency(
    db: &sqlx::PgPool,
    merchant_id: Uuid,
    key: &str,
    resource_type: &str,
    req_hash: &str,
) -> Result<Option<(i32, serde_json::Value)>, FeelPayError> {
    let record = sqlx::query_as::<_, IdempotencyRecord>(
        r#"
        SELECT request_hash, response_code, response_body
        FROM idempotency_keys
        WHERE merchant_id = $1 AND idempotency_key = $2 AND resource_type = $3
        "#,
    )
    .bind(merchant_id)
    .bind(key)
    .bind(resource_type)
    .fetch_optional(db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    if let Some(rec) = record {
        if rec.request_hash != req_hash {
            return Err(FeelPayError::Conflict(
                "Idempotency key reused with different payload".into(),
            ));
        }
        if let (Some(code), Some(body)) = (rec.response_code, rec.response_body) {
            return Ok(Some((code, body)));
        }
    }
    Ok(None)
}

async fn store_idempotency_response(
    db: &sqlx::PgPool,
    merchant_id: Uuid,
    key: &str,
    resource_type: &str,
    req_hash: &str,
    status_code: i32,
    body: &serde_json::Value,
) -> Result<(), FeelPayError> {
    sqlx::query(
        r#"
        INSERT INTO idempotency_keys
            (merchant_id, idempotency_key, request_hash, resource_type, response_code, response_body, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT (merchant_id, idempotency_key, resource_type)
        DO UPDATE SET
            response_code = EXCLUDED.response_code,
            response_body = EXCLUDED.response_body,
            updated_at = NOW()
        "#,
    )
    .bind(merchant_id)
    .bind(key)
    .bind(req_hash)
    .bind(resource_type)
    .bind(status_code)
    .bind(body)
    .execute(db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/v1/payments",
    request_body = CreatePaymentRequest,
    responses(
        (status = 200, description = "Payment created successfully", body = PaymentOrder),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn create_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<CreatePaymentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let idempotency_key = req
        .headers()
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(|k| k.to_string());

    let req_data = payload.into_inner();
    let req_hash = request_hash(&req_data)?;

    if let Some(key) = idempotency_key.as_deref() {
        if let Some((status, body)) =
            check_idempotency(&state.db, merchant_id, key, "payment", &req_hash).await?
        {
            let status = actix_web::http::StatusCode::from_u16(status as u16)
                .unwrap_or(actix_web::http::StatusCode::OK);
            return Ok(HttpResponse::build(status).json(body));
        }
    }

    if req_data.amount <= 0 {
        return Err(FeelPayError::BadRequest(
            "Amount must be greater than 0".into(),
        ));
    }

    // 1. Create Order Record
    let mut order = PaymentOrder {
        id: Uuid::new_v4(),
        merchant_id,
        amount: req_data.amount,
        currency: req_data.currency.clone(),
        status: PaymentStatus::Created,
        order_id: req_data.order_id.clone(),
        payment_method: None,
        payment_method_data: None,
        connector: None,
        customer_id: None,
        customer_email: req_data.customer_email.clone(),
        customer_phone: req_data.customer_phone.clone(),
        description: req_data.description.clone(),
        notes: None,
        return_url: req_data.return_url.clone(),
        idempotency_key: idempotency_key.clone(),
        gateway_reference: None,
        gateway_response: None,
        error_message: None,
        error_code: None,
        metadata: None,
        refunded_amount: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        authorized_at: None,
        captured_at: None,
        settled_at: None,
    };

    let txn_ctx = TransactionContext {
        payment_id: order.id,
        merchant_id,
        order_id: order.order_id.clone(),
        amount: order.amount,
        currency: order.currency.clone(),
        customer_email: order.customer_email.clone(),
        customer_phone: order.customer_phone.clone(),
        return_url: order.return_url.clone(),
        description: order.description.clone(),
    };

    let mut final_method_data = req_data.method_data.clone();

    // 2. Vault Card if requested
    if req_data.vault_card {
        if let MethodData::Card {
            number: Some(num),
            expiry_month: Some(m),
            expiry_year: Some(y),
            holder_name,
            ..
        } = &final_method_data
        {
            tracing::info!(merchant_id = %merchant_id, "Vaulting card for new payment");
            let vaulted = state
                .vault
                .vault_card(
                    merchant_id,
                    None,
                    num.clone(),
                    *m as i32,
                    *y as i32,
                    holder_name.clone(),
                )
                .await?;

            // Replace raw card with tokenized card data for processing
            final_method_data = MethodData::Card {
                number: None,
                token: Some(vaulted.token),
                expiry_month: Some(*m),
                expiry_year: Some(*y),
                cvv: match &req_data.method_data {
                    MethodData::Card { cvv, .. } => cvv.clone(),
                    _ => None,
                },
                holder_name: holder_name.clone(),
            };
        }
    }

    let method_name = payment_method_name(&final_method_data).to_string();
    let connector = connector_for_payment(
        &final_method_data,
        req_data.processor_override.as_deref(),
        state.config.is_live(),
    );
    let environment = if state.config.is_live() {
        "live"
    } else {
        "test"
    };

    // 3. Process Payment via Engine
    let engine = state.payment_engine.read().await;
    let proc_result = engine
        .authorize(&txn_ctx, &final_method_data, Some(&connector))
        .await;

    // 4. Update Order Status
    match proc_result {
        Ok(resp) => {
            order.status = resp.status;
            order.gateway_reference = resp.gateway_reference;
            order.gateway_response = resp.gateway_response;
            order.error_code = resp.error_code;
            order.error_message = resp.error_message;
        }
        Err(e) => {
            order.status = PaymentStatus::Failed;
            order.error_code = Some("ENGINE_ERROR".into());
            order.error_message = Some(e.to_string());
        }
    }
    let now = chrono::Utc::now();
    let (authorized_at, captured_at) = status_timestamps(&order.status, now);
    order.payment_method = Some(method_name.clone());
    order.payment_method_data =
        Some(serde_json::to_value(&final_method_data).unwrap_or_else(|_| serde_json::json!({})));
    order.connector = Some(connector.clone());
    order.metadata = Some(payment_metadata(environment, &method_name, &connector));
    order.authorized_at = authorized_at;
    order.captured_at = captured_at;
    order.updated_at = now;

    // 5. Persist Order
    sqlx::query(
        r#"
        INSERT INTO payment_orders (
            id, merchant_id, amount, currency, status, order_id,
            payment_method, payment_method_data, connector, customer_email, customer_phone,
            description, return_url, idempotency_key, gateway_reference, gateway_response,
            error_message, error_code, metadata, refunded_amount, created_at, updated_at,
            authorized_at, captured_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22,
            $23, $24
        )
        "#,
    )
    .bind(order.id)
    .bind(order.merchant_id)
    .bind(order.amount)
    .bind(&order.currency)
    .bind(order.status.as_str())
    .bind(&order.order_id)
    .bind(&order.payment_method)
    .bind(&order.payment_method_data)
    .bind(&order.connector)
    .bind(&order.customer_email)
    .bind(&order.customer_phone)
    .bind(&order.description)
    .bind(&order.return_url)
    .bind(&order.idempotency_key)
    .bind(&order.gateway_reference)
    .bind(&order.gateway_response)
    .bind(&order.error_message)
    .bind(&order.error_code)
    .bind(&order.metadata)
    .bind(order.refunded_amount)
    .bind(order.created_at)
    .bind(order.updated_at)
    .bind(order.authorized_at)
    .bind(order.captured_at)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let body = serde_json::to_value(&order)
        .map_err(|e| FeelPayError::Internal(format!("Serialize response failed: {}", e)))?;
    if let Some(key) = idempotency_key.as_deref() {
        store_idempotency_response(
            &state.db,
            merchant_id,
            key,
            "payment",
            &req_hash,
            200,
            &body,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().json(body))
}

#[utoipa::path(
    get,
    path = "/v1/payments",
    params(
        ListPaymentsQuery
    ),
    responses(
        (status = 200, description = "List of payments")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_payments(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListPaymentsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Use cursor-based pagination if cursor provided, otherwise use legacy
    let pagination = PaginationParams {
        cursor: query.cursor.clone(),
        limit: query.limit,
    }
    .normalize();
    let use_cursor = pagination.cursor.is_some();
    let limit = pagination.limit.unwrap_or(20);

    let orders: Vec<PaymentOrder> = if use_cursor {
        // Cursor-based pagination
        let cursor = pagination
            .decode_cursor()?
            .ok_or_else(|| FeelPayError::BadRequest("Invalid cursor".into()))?;

        sqlx::query_as::<_, PaymentOrder>(
            r#"
            SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at
            FROM payment_orders
            WHERE merchant_id = $1
              AND ($2::TEXT IS NULL OR status = $2::TEXT)
              AND (created_at, id) < ($3::TIMESTAMP, $4::UUID)
            ORDER BY created_at DESC, id DESC
            LIMIT $5
            "#,
        )
        .bind(merchant_id)
        .bind(&query.status)
        .bind(&cursor.sort_key)
        .bind(&cursor.last_id)
        .bind(limit + 1)
        .fetch_all(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    } else {
        // Legacy offset-based pagination (backward compatible)
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        sqlx::query_as::<_, PaymentOrder>(
            r#"
            SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at
            FROM payment_orders
            WHERE merchant_id = $1
              AND ($2::TEXT IS NULL OR status = $2::TEXT)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(merchant_id)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    };

    let has_more = orders.len() > limit as usize;
    let data: Vec<PaymentOrder> = orders.into_iter().take(limit as usize).collect();

    // Generate next cursor from last record
    let next_cursor = if has_more {
        data.last()
            .map(|p| build_cursor(&p.created_at.to_rfc3339(), &p.id.to_string()))
    } else {
        None
    };

    let response = PaginatedResponse::new(data, next_cursor, has_more);

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/v1/payments/{payment_id}",
    params(
        ("payment_id" = Uuid, Path, description = "Payment ID")
    ),
    responses(
        (status = 200, description = "Payment details", body = PaymentOrder),
        (status = 404, description = "Payment not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let payment_id = path.into_inner();

    let order = sqlx::query_as::<_, PaymentOrder>(
        r#"
        SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at
        FROM payment_orders
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    Ok(HttpResponse::Ok().json(order))
}

#[utoipa::path(
    post,
    path = "/v1/payments/{payment_id}/capture",
    params(
        ("payment_id" = Uuid, Path, description = "Payment ID")
    ),
    request_body = CapturePaymentRequest,
    responses(
        (status = 200, description = "Payment captured successfully", body = PaymentOrder),
        (status = 400, description = "Invalid state"),
        (status = 404, description = "Payment not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn capture_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<CapturePaymentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let payment_id = path.into_inner();

    let mut order = sqlx::query_as::<_, PaymentOrder>(
        r#"
        SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at
        FROM payment_orders
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    if order.status != PaymentStatus::Authorized {
        return Err(FeelPayError::BadRequest(
            "Payment is not in an authorized state".into(),
        ));
    }

    let txn_ctx = TransactionContext {
        payment_id: order.id,
        merchant_id,
        order_id: order.order_id.clone(),
        amount: order.amount,
        currency: order.currency.clone(),
        customer_email: order.customer_email.clone(),
        customer_phone: order.customer_phone.clone(),
        return_url: order.return_url.clone(),
        description: order.description.clone(),
    };

    let connector = capture_connector(order.connector.as_deref());
    let engine = state.payment_engine.read().await;
    let proc_result = engine.capture(&txn_ctx, payload.amount, connector).await?;

    order.status = proc_result.status;
    order.gateway_reference = proc_result.gateway_reference;
    order.gateway_response = proc_result.gateway_response;
    order.error_code = proc_result.error_code;
    order.error_message = proc_result.error_message;

    if order.status == PaymentStatus::Captured {
        // Record in ledger
        let fee = order
            .amount
            .checked_mul(3)
            .ok_or_else(|| FeelPayError::Internal("Fee calculation overflow".into()))?
            / 100; // 3% platform fee in paise
        state
            .ledger
            .record_transaction(
                order.id,
                order.merchant_id,
                order.amount,
                fee,
                "payment_capture",
            )
            .await?;

        sqlx::query(
            r#"
            UPDATE payment_orders
            SET status = $1, updated_at = $2, captured_at = $3,
                gateway_reference = $4, gateway_response = $5,
                error_code = $6, error_message = $7
            WHERE id = $8
            "#,
        )
        .bind(order.status.as_str())
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .bind(&order.gateway_reference)
        .bind(&order.gateway_response)
        .bind(&order.error_code)
        .bind(&order.error_message)
        .bind(order.id)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    } else {
        sqlx::query(
            r#"
            UPDATE payment_orders
            SET status = $1, updated_at = $2,
                gateway_reference = $3, gateway_response = $4,
                error_code = $5, error_message = $6
            WHERE id = $7
            "#,
        )
        .bind(order.status.as_str())
        .bind(chrono::Utc::now())
        .bind(&order.gateway_reference)
        .bind(&order.gateway_response)
        .bind(&order.error_code)
        .bind(&order.error_message)
        .bind(order.id)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    }

    Ok(HttpResponse::Ok().json(order))
}

#[utoipa::path(
    post,
    path = "/v1/payments/{payment_id}/refund",
    params(
        ("payment_id" = Uuid, Path, description = "Payment ID")
    ),
    request_body = RefundPaymentRequest,
    responses(
        (status = 200, description = "Payment refunded successfully"),
        (status = 400, description = "Invalid amount or state"),
        (status = 404, description = "Payment not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn refund_payment(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<RefundPaymentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let idempotency_key = req
        .headers()
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(|k| k.to_string());
    let req_hash = request_hash(&*payload)?;

    if let Some(key) = idempotency_key.as_deref() {
        if let Some((status, body)) =
            check_idempotency(&state.db, merchant_id, key, "refund", &req_hash).await?
        {
            let status = actix_web::http::StatusCode::from_u16(status as u16)
                .unwrap_or(actix_web::http::StatusCode::OK);
            return Ok(HttpResponse::build(status).json(body));
        }
    }
    let payment_id = path.into_inner();

    let mut order = sqlx::query_as::<_, PaymentOrder>(
        r#"
        SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at
        FROM payment_orders
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    if !matches!(
        order.status,
        PaymentStatus::Captured
            | PaymentStatus::Settled
            | PaymentStatus::PartiallyRefunded
            | PaymentStatus::Refunded
    ) {
        return Err(FeelPayError::BadRequest(
            "Refund allowed only for captured/settled payments".into(),
        ));
    }

    let remaining = order.amount - order.refunded_amount;
    if remaining <= 0 {
        return Err(FeelPayError::BadRequest(
            "Payment already fully refunded".into(),
        ));
    }
    let refund_amount = payload.amount.min(remaining);
    if refund_amount <= 0 {
        return Err(FeelPayError::BadRequest("Invalid refund amount".into()));
    }

    let txn_ctx = TransactionContext {
        payment_id: order.id,
        merchant_id,
        order_id: order.order_id.clone(),
        amount: order.amount,
        currency: order.currency.clone(),
        customer_email: order.customer_email.clone(),
        customer_phone: order.customer_phone.clone(),
        return_url: order.return_url.clone(),
        description: order.description.clone(),
    };

    let connector = order
        .connector
        .clone()
        .unwrap_or_else(|| "simulator".to_string());
    let engine = state.payment_engine.read().await;
    let _ = engine.refund(&txn_ctx, refund_amount, &connector).await?;

    let refund_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        r#"
        INSERT INTO refunds (id, payment_order_id, merchant_id, amount, status, reason, created_at, processed_at)
        VALUES ($1, $2, $3, $4, 'processed', $5, $6, $7)
        "#,
    )
    .bind(refund_id)
    .bind(order.id)
    .bind(merchant_id)
    .bind(refund_amount)
    .bind(&payload.reason)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    // Record refund in ledger (reverse transaction)
    // For refund, we debit merchant and credit escrow. Fee is usually not refunded by platform.
    state
        .ledger
        .record_transaction(
            refund_id,
            order.merchant_id,
            -refund_amount, // Negative amount to represent reversal
            0,
            "refund",
        )
        .await?;

    order.refunded_amount += refund_amount;
    order.status = if order.refunded_amount >= order.amount {
        PaymentStatus::Refunded
    } else {
        PaymentStatus::PartiallyRefunded
    };

    sqlx::query(
        r#"UPDATE payment_orders SET refunded_amount = $1, status = $2, updated_at = $3 WHERE id = $4"#,
    )
    .bind(order.refunded_amount)
    .bind(order.status.as_str())
    .bind(now)
    .bind(order.id)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs (
            tenant_id, merchant_id, entity_type, entity_id, action, target,
            new_state, metadata, actor_type, actor_id, ip_address, user_agent
        )
        VALUES ($1, $2, 'refund', $3, 'refund_created', 'payment_refund', $4::jsonb, $5::jsonb, 'merchant_user', $6, $7, $8)
        "#,
    )
    .bind(tenant_id_from_req(&req))
    .bind(merchant_id)
    .bind(refund_id)
    .bind(serde_json::json!({
        "payment_id": order.id,
        "refund_id": refund_id,
        "status": order.status.as_str(),
        "refunded_amount": refund_amount,
        "total_refunded": order.refunded_amount
    }))
    .bind(serde_json::json!({
        "reason_present": payload.reason.as_ref().is_some_and(|reason| !reason.trim().is_empty())
    }))
    .bind(actor_id_from_req(&req, merchant_id))
    .bind(req.connection_info().realip_remote_addr().map(str::to_string))
    .bind(
        req.headers()
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string),
    )
    .execute(&state.db)
    .await;

    let body = serde_json::json!({
        "payment_id": order.id,
        "refund_id": refund_id,
        "status": order.status.as_str(),
        "refunded_amount": refund_amount,
        "total_refunded": order.refunded_amount
    });

    if let Some(key) = idempotency_key.as_deref() {
        store_idempotency_response(&state.db, merchant_id, key, "refund", &req_hash, 200, &body)
            .await?;
    }

    Ok(HttpResponse::Ok().json(body))
}

// ============ Batch Refund ============

#[derive(Debug, Deserialize)]
pub struct BatchRefundRequest {
    pub refunds: Vec<BatchRefundItem>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchRefundItem {
    pub payment_id: Uuid,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct BatchRefundResponse {
    pub batch_id: Uuid,
    pub total_requested: i32,
    pub successful: i32,
    pub failed: i32,
    pub results: Vec<BatchRefundResult>,
}

#[derive(Debug, Serialize)]
pub struct BatchRefundResult {
    pub payment_id: Uuid,
    pub success: bool,
    pub refund_id: Option<Uuid>,
    pub amount: Option<i64>,
    pub error: Option<String>,
}

/// Process multiple refunds in a batch
pub async fn batch_refund(
    req: HttpRequest,
    state: web::Data<AppState>,
    payload: web::Json<BatchRefundRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let batch_id = Uuid::new_v4();
    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for item in &payload.refunds {
        // Get payment order
        let order_result = sqlx::query_as::<_, PaymentOrder>(
            "SELECT id, merchant_id, order_id, amount, currency, status, payment_method, payment_method_data, connector, customer_id, customer_email, customer_phone, description, notes, return_url, idempotency_key, gateway_reference, gateway_response, error_code, error_message, metadata, refunded_amount, created_at, updated_at, authorized_at, captured_at, settled_at FROM payment_orders WHERE id = $1 AND merchant_id = $2"
        )
        .bind(item.payment_id)
        .bind(merchant_id)
        .fetch_optional(&state.db)
        .await;

        match order_result {
            Ok(Some(order)) => {
                // Check if refundable
                if !matches!(
                    order.status,
                    PaymentStatus::Captured
                        | PaymentStatus::Settled
                        | PaymentStatus::PartiallyRefunded
                ) {
                    failed += 1;
                    results.push(BatchRefundResult {
                        payment_id: item.payment_id,
                        success: false,
                        refund_id: None,
                        amount: None,
                        error: Some("Payment not in refundable state".into()),
                    });
                    continue;
                }

                let remaining = order.amount - order.refunded_amount;
                if remaining <= 0 {
                    failed += 1;
                    results.push(BatchRefundResult {
                        payment_id: item.payment_id,
                        success: false,
                        refund_id: None,
                        amount: None,
                        error: Some("Payment already fully refunded".into()),
                    });
                    continue;
                }

                let refund_amount = item.amount.min(remaining);
                if refund_amount <= 0 {
                    failed += 1;
                    results.push(BatchRefundResult {
                        payment_id: item.payment_id,
                        success: false,
                        refund_id: None,
                        amount: None,
                        error: Some("Invalid refund amount".into()),
                    });
                    continue;
                }

                // Process refund
                let refund_id = Uuid::new_v4();
                let now = chrono::Utc::now();

                let insert_result = sqlx::query(
                    "INSERT INTO refunds (id, payment_order_id, merchant_id, amount, status, reason, created_at, processed_at) VALUES ($1, $2, $3, $4, 'processed', $5, $6, $7)"
                )
                .bind(refund_id)
                .bind(order.id)
                .bind(merchant_id)
                .bind(refund_amount)
                .bind(&payload.reason)
                .bind(now)
                .bind(now)
                .execute(&state.db)
                .await;

                match insert_result {
                    Ok(_) => {
                        // Update order
                        let new_refunded = order.refunded_amount + refund_amount;
                        let new_status = if new_refunded >= order.amount {
                            PaymentStatus::Refunded
                        } else {
                            PaymentStatus::PartiallyRefunded
                        };

                        sqlx::query(
                            "UPDATE payment_orders SET refunded_amount = $1, status = $2, updated_at = $3 WHERE id = $4"
                        )
                        .bind(new_refunded)
                        .bind(new_status.as_str())
                        .bind(now)
                        .bind(order.id)
                        .execute(&state.db)
                        .await
                        .ok();

                        // Record in ledger
                        state
                            .ledger
                            .record_transaction(
                                refund_id,
                                merchant_id,
                                -refund_amount,
                                0,
                                "batch_refund",
                            )
                            .await
                            .ok();

                        successful += 1;
                        results.push(BatchRefundResult {
                            payment_id: item.payment_id,
                            success: true,
                            refund_id: Some(refund_id),
                            amount: Some(refund_amount),
                            error: None,
                        });
                    }
                    Err(e) => {
                        failed += 1;
                        results.push(BatchRefundResult {
                            payment_id: item.payment_id,
                            success: false,
                            refund_id: None,
                            amount: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
            Ok(None) => {
                failed += 1;
                results.push(BatchRefundResult {
                    payment_id: item.payment_id,
                    success: false,
                    refund_id: None,
                    amount: None,
                    error: Some("Payment not found".into()),
                });
            }
            Err(e) => {
                failed += 1;
                results.push(BatchRefundResult {
                    payment_id: item.payment_id,
                    success: false,
                    refund_id: None,
                    amount: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(HttpResponse::Ok().json(BatchRefundResponse {
        batch_id,
        total_requested: payload.refunds.len() as i32,
        successful,
        failed,
        results,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/refunds",
    params(
        ListRefundsQuery
    ),
    responses(
        (status = 200, description = "List refunds for authenticated merchant")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_refunds(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListRefundsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let status = query
        .status
        .as_deref()
        .filter(|value| !value.eq_ignore_ascii_case("all"));
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", value));

    let rows = sqlx::query(
        r#"
        SELECT
            r.id,
            r.payment_order_id,
            po.order_id,
            r.amount,
            r.status,
            r.reason,
            r.gateway_reference,
            r.created_at,
            r.processed_at,
            po.customer_email,
            po.status AS payment_status,
            po.refunded_amount,
            po.amount AS payment_amount
        FROM refunds r
        JOIN payment_orders po ON po.id = r.payment_order_id AND po.merchant_id = r.merchant_id
        WHERE r.merchant_id = $1
          AND ($2::TEXT IS NULL OR r.status = $2)
          AND (
            $3::TEXT IS NULL
            OR r.id::TEXT ILIKE $3
            OR po.order_id ILIKE $3
            OR COALESCE(po.customer_email, '') ILIKE $3
          )
        ORDER BY r.created_at DESC
        LIMIT $4
        "#,
    )
    .bind(merchant_id)
    .bind(status)
    .bind(search)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let refunds: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "payment_order_id": row.get::<Uuid, _>("payment_order_id"),
                "order_id": row.get::<String, _>("order_id"),
                "amount": row.get::<i64, _>("amount"),
                "status": row.get::<String, _>("status"),
                "reason": row.get::<Option<String>, _>("reason"),
                "gateway_reference": row.get::<Option<String>, _>("gateway_reference"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "processed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at"),
                "customer_email": row.get::<Option<String>, _>("customer_email"),
                "payment_status": row.get::<String, _>("payment_status"),
                "refunded_amount": row.get::<i64, _>("refunded_amount"),
                "payment_amount": row.get::<i64, _>("payment_amount"),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(json!({ "refunds": refunds })))
}

#[utoipa::path(
    get,
    path = "/v1/refunds/{id}",
    params(
        ("id" = Uuid, Path, description = "Refund ID")
    ),
    responses(
        (status = 200, description = "Refund detail"),
        (status = 404, description = "Refund not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_refund(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let refund_id = path.into_inner();

    let row = sqlx::query(
        r#"
        SELECT
            r.id,
            r.payment_order_id,
            po.order_id,
            r.amount,
            r.status,
            r.reason,
            r.idempotency_key,
            r.gateway_reference,
            r.created_at,
            r.processed_at,
            po.customer_email,
            po.customer_phone,
            po.payment_method,
            po.status AS payment_status,
            po.refunded_amount,
            po.amount AS payment_amount
        FROM refunds r
        JOIN payment_orders po ON po.id = r.payment_order_id AND po.merchant_id = r.merchant_id
        WHERE r.id = $1 AND r.merchant_id = $2
        "#,
    )
    .bind(refund_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Refund not found".into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "id": row.get::<Uuid, _>("id"),
        "payment_order_id": row.get::<Uuid, _>("payment_order_id"),
        "order_id": row.get::<String, _>("order_id"),
        "amount": row.get::<i64, _>("amount"),
        "status": row.get::<String, _>("status"),
        "reason": row.get::<Option<String>, _>("reason"),
        "idempotency_key": row.get::<Option<String>, _>("idempotency_key"),
        "gateway_reference": row.get::<Option<String>, _>("gateway_reference"),
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "processed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at"),
        "customer_email": row.get::<Option<String>, _>("customer_email"),
        "customer_phone": row.get::<Option<String>, _>("customer_phone"),
        "payment_method": row.get::<Option<String>, _>("payment_method"),
        "payment_status": row.get::<String, _>("payment_status"),
        "refunded_amount": row.get::<i64, _>("refunded_amount"),
        "payment_amount": row.get::<i64, _>("payment_amount"),
    })))
}

#[utoipa::path(
    get,
    path = "/v1/payments/{id}",
    responses(
        (status = 200, description = "Detailed payment info"),
        (status = 404, description = "Payment not found")
    ),
    security(("api_key" = []))
)]
pub async fn get_payment_details(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let payment_id = path.into_inner();

    let order = sqlx::query_as::<_, PaymentOrder>(
        "SELECT * FROM payment_orders WHERE id = $1 AND merchant_id = $2",
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let Some(order) = order else {
        return Err(FeelPayError::NotFound("Payment not found".into()));
    };

    // Fetch related webhooks
    let webhooks = sqlx::query(
        "SELECT id, event_type, status, attempts, last_attempt_at, created_at FROM payment_webhook_events WHERE payment_id = $1 ORDER BY created_at DESC"
    )
    .bind(payment_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(HttpResponse::Ok().json(json!({
        "payment": order,
        "webhooks": webhooks.into_iter().map(|row| json!({
            "id": Row::get::<Uuid, _>(&row, "id"),
            "event_type": Row::get::<String, _>(&row, "event_type"),
            "status": Row::get::<String, _>(&row, "status"),
            "attempts": Row::get::<i32, _>(&row, "attempts"),
            "last_attempt_at": Row::get::<Option<chrono::DateTime<chrono::Utc>>, _>(&row, "last_attempt_at"),
            "created_at": Row::get::<chrono::DateTime<chrono::Utc>, _>(&row, "created_at"),
        })).collect::<Vec<_>>()
    })))
}

#[utoipa::path(
    get,
    path = "/v1/payments/{id}/timeline",
    responses(
        (status = 200, description = "Payment event timeline")
    ),
    security(("api_key" = []))
)]
pub async fn get_payment_timeline(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let payment_id = path.into_inner();

    let events = sqlx::query(
        r#"
        SELECT id, event_type, status_before, status_after, metadata, created_at
        FROM transaction_events
        WHERE payment_id = $1 AND merchant_id = $2
        ORDER BY created_at ASC
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(
        events
            .into_iter()
            .map(|row| {
                json!({
                    "id": Row::get::<Uuid, _>(&row, "id"),
                    "event_type": Row::get::<String, _>(&row, "event_type"),
                    "status_before": Row::get::<Option<String>, _>(&row, "status_before"),
                    "status_after": Row::get::<Option<String>, _>(&row, "status_after"),
                    "metadata": Row::get::<Value, _>(&row, "metadata"),
                    "created_at": Row::get::<chrono::DateTime<chrono::Utc>, _>(&row, "created_at"),
                })
            })
            .collect::<Vec<_>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::processor::MethodData;
    use crate::models::payment_order::PaymentStatus;

    #[test]
    fn method_name_from_method_data_maps_supported_methods() {
        assert_eq!(
            payment_method_name(&MethodData::Card {
                number: None,
                token: Some("tok_123".into()),
                expiry_month: None,
                expiry_year: None,
                cvv: None,
                holder_name: None,
            }),
            "card"
        );
        assert_eq!(
            payment_method_name(&MethodData::Upi {
                vpa: Some("buyer@upi".into()),
                flow: Some("collect".into()),
            }),
            "upi"
        );
        assert_eq!(
            payment_method_name(&MethodData::Simulator {
                action: "success".into(),
            }),
            "simulator"
        );
    }

    #[test]
    fn connector_for_request_prefers_override_then_test_simulator() {
        assert_eq!(
            connector_for_payment(
                &MethodData::Card {
                    number: None,
                    token: Some("tok_123".into()),
                    expiry_month: None,
                    expiry_year: None,
                    cvv: None,
                    holder_name: None,
                },
                Some("card"),
                false,
            ),
            "card"
        );
        assert_eq!(
            connector_for_payment(
                &MethodData::Upi {
                    vpa: Some("buyer@upi".into()),
                    flow: Some("collect".into()),
                },
                None,
                false,
            ),
            "simulator"
        );
        assert_eq!(
            connector_for_payment(
                &MethodData::Upi {
                    vpa: Some("buyer@upi".into()),
                    flow: Some("collect".into()),
                },
                None,
                true,
            ),
            "upi"
        );
    }

    #[test]
    fn status_timestamps_mark_authorized_and_captured() {
        let now = chrono::Utc::now();
        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Authorized, now);
        assert_eq!(authorized_at, Some(now));
        assert_eq!(captured_at, None);

        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Captured, now);
        assert_eq!(authorized_at, Some(now));
        assert_eq!(captured_at, Some(now));

        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Failed, now);
        assert_eq!(authorized_at, None);
        assert_eq!(captured_at, None);
    }

    #[test]
    fn checkout_metadata_includes_environment_and_method() {
        let metadata = payment_metadata("test", "card", "simulator");
        assert_eq!(metadata["environment"], "test");
        assert_eq!(metadata["payment_method"], "card");
        assert_eq!(metadata["connector"], "simulator");
    }

    #[test]
    fn capture_connector_uses_stored_connector_or_simulator() {
        assert_eq!(capture_connector(Some("card")), "card");
        assert_eq!(capture_connector(Some("upi")), "upi");
        assert_eq!(capture_connector(None), "simulator");
    }
}

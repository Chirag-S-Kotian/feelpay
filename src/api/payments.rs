use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::engine::processor::{MethodData, TransactionContext};
use crate::error::FeelPayError;
use crate::models::payment_order::{PaymentOrder, PaymentStatus};
use crate::state::AppState;

#[derive(Deserialize, Serialize)]
pub struct CreatePaymentRequest {
    pub amount: i64,
    pub currency: String,
    pub order_id: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub method_data: MethodData,
    #[serde(default)]
    pub vault_card: bool, // If true, we vault the card
    pub processor_override: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("", web::post().to(create_payment))
            .route("", web::get().to(list_payments))
            .route("/{payment_id}", web::get().to(get_payment))
            .route("/{payment_id}/capture", web::post().to(capture_payment))
            .route("/{payment_id}/refund", web::post().to(refund_payment))
    );
}

#[derive(Deserialize)]
pub struct ListPaymentsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize)]
pub struct CapturePaymentRequest {
    pub amount: i64,
}

#[derive(Deserialize, Serialize)]
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
        .get::<Uuid>()
        .copied()
        .or_else(|| {
            req.headers()
                .get("x-merchant-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant ID context".into()))
}

fn request_hash<T: serde::Serialize>(body: &T) -> Result<String, FeelPayError> {
    let payload = serde_json::to_vec(body)
        .map_err(|e| FeelPayError::BadRequest(format!("Invalid request payload: {}", e)))?;
    let digest = Sha256::digest(payload);
    Ok(hex::encode(digest))
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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;
    Ok(())
}

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
        return Err(FeelPayError::BadRequest("Amount must be greater than 0".into()));
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
        if let MethodData::Card { number: Some(num), expiry_month: Some(m), expiry_year: Some(y), holder_name, .. } = &final_method_data {
            tracing::info!(merchant_id = %merchant_id, "Vaulting card for new payment");
            let vaulted = state.vault.vault_card(
                merchant_id,
                None,
                num.clone(),
                *m as i32,
                *y as i32,
                holder_name.clone(),
            ).await?;

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

    // 3. Process Payment via Engine
    let engine = state.payment_engine.read().await;
    let proc_result = engine.authorize(&txn_ctx, &final_method_data, req_data.processor_override.as_deref()).await;

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

    // 5. Persist Order
    // ... DB insertion omitted for brevity, let's just insert it
    sqlx::query(
        r#"
        INSERT INTO payment_orders (id, merchant_id, amount, currency, status, order_id, customer_email, customer_phone, description, return_url, gateway_reference, gateway_response, error_message, error_code, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(order.id)
    .bind(order.merchant_id)
    .bind(order.amount)
    .bind(&order.currency)
    .bind(order.status.as_str())
    .bind(&order.order_id)
    .bind(&order.customer_email)
    .bind(&order.customer_phone)
    .bind(&order.description)
    .bind(&order.return_url)
    .bind(&order.gateway_reference)
    .bind(&order.gateway_response)
    .bind(&order.error_message)
    .bind(&order.error_code)
    .bind(order.created_at)
    .bind(order.updated_at)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let body = serde_json::to_value(&order)
        .map_err(|e| FeelPayError::InternalError(format!("Serialize response failed: {}", e)))?;
    if let Some(key) = idempotency_key.as_deref() {
        store_idempotency_response(&state.db, merchant_id, key, "payment", &req_hash, 200, &body)
            .await?;
    }

    Ok(HttpResponse::Ok().json(body))
}

pub async fn list_payments(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListPaymentsQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let orders = sqlx::query_as::<_, PaymentOrder>(
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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM payment_orders
        WHERE merchant_id = $1
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
        "#,
    )
    .bind(merchant_id)
    .bind(&query.status)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "page": page,
        "per_page": per_page,
        "total": total.0,
        "payments": orders
    })))
}

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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    Ok(HttpResponse::Ok().json(order))
}

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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    if order.status != PaymentStatus::Authorized {
        return Err(FeelPayError::BadRequest("Payment is not in an authorized state".into()));
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

    let engine = state.payment_engine.read().await;
    // We assume default or stored processor if we had one. In a real system, we'd store the processor name on the order.
    // For now, simulate using "simulator"
    let proc_result = engine.capture(&txn_ctx, payload.amount, "simulator").await?;

    order.status = proc_result.status;
    
    sqlx::query(
        r#"UPDATE payment_orders SET status = $1, updated_at = $2 WHERE id = $3"#,
    )
    .bind(order.status.as_str())
    .bind(chrono::Utc::now())
    .bind(order.id)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(order))
}

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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Payment not found".into()))?;

    if !matches!(
        order.status,
        PaymentStatus::Captured | PaymentStatus::Settled | PaymentStatus::PartiallyRefunded | PaymentStatus::Refunded
    ) {
        return Err(FeelPayError::BadRequest(
            "Refund allowed only for captured/settled payments".into(),
        ));
    }

    let remaining = order.amount - order.refunded_amount;
    if remaining <= 0 {
        return Err(FeelPayError::BadRequest("Payment already fully refunded".into()));
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

    let connector = order.connector.clone().unwrap_or_else(|| "simulator".to_string());
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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

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
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

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

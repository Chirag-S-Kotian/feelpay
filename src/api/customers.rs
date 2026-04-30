use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::middleware::auth::MerchantId;
use crate::models::customer::{
    CreateCustomerRequest, Customer, CustomerResponse, SavedPaymentMethod,
    SavedPaymentMethodResponse,
};
use crate::state::AppState;
use sqlx::Row;

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<MerchantId>()
        .map(|id| id.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Merchant ID not found".to_string()))
}

fn tenant_id_from_req(req: &HttpRequest) -> Option<Uuid> {
    req.extensions()
        .get::<crate::middleware::auth::TenantId>()
        .map(|id| id.0)
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerListResponse {
    pub data: Vec<CustomerResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
}

// Create a new customer
#[utoipa::path(
    post,
    path = "/v1/customers",
    request_body = CreateCustomerRequest,
    responses(
        (status = 201, description = "Customer created successfully", body = CustomerResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn create_customer(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateCustomerRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let tenant_id = tenant_id_from_req(&req);
    let mut conn = state.db.acquire().await?;

    let result = sqlx::query(
        r#"
        INSERT INTO customers (
            merchant_id, tenant_id, customer_reference, customer_reference_type,
            email, phone, name, consent_given, consent_purpose, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (merchant_id, customer_reference, customer_reference_type) 
        DO UPDATE SET email = EXCLUDED.email, phone = EXCLUDED.phone, name = EXCLUDED.name, updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(tenant_id)
    .bind(&body.customer_reference)
    .bind(&body.customer_reference_type)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&body.name)
    .bind(body.consent_given.unwrap_or(false))
    .bind(&body.consent_purpose)
    .bind(body.metadata.as_ref().unwrap_or(&serde_json::json!({})))
    .fetch_one(&mut *conn)
    .await?;

    Ok(HttpResponse::Created().json(CustomerResponse::from(Customer::from(result))))
}

// Get customer by ID
#[utoipa::path(
    get,
    path = "/v1/customers/{id}",
    params(
        ("id" = Uuid, Path, description = "Customer ID")
    ),
    responses(
        (status = 200, description = "Customer details", body = CustomerResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Customer not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_customer(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let customer_id = path.into_inner();
    let mut conn = state
        .db
        .acquire()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let result = sqlx::query(
        "SELECT * FROM customers WHERE id = $1 AND merchant_id = $2 AND deleted_at IS NULL",
    )
    .bind(customer_id)
    .bind(merchant_id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| FeelPayError::NotFound("Customer not found".to_string()))?;

    Ok(HttpResponse::Ok().json(CustomerResponse::from(Customer::from(result))))
}

// List customers
#[utoipa::path(
    get,
    path = "/v1/customers",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of customers", body = CustomerListResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_customers(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let mut conn = state.db.acquire().await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    // Get total count
    let total_row =
        sqlx::query("SELECT COUNT(*) FROM customers WHERE merchant_id = $1 AND deleted_at IS NULL")
            .bind(merchant_id)
            .fetch_one(&mut *conn)
            .await?;
    let total: i64 = total_row.get(0);

    // Get results
    let customers_rows = sqlx::query(
        "SELECT * FROM customers WHERE merchant_id = $1 AND deleted_at IS NULL 
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(merchant_id)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(&mut *conn)
    .await?;

    let customers: Vec<Customer> = customers_rows.into_iter().map(Customer::from).collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i32;

    Ok(HttpResponse::Ok().json(CustomerListResponse {
        data: customers.into_iter().map(CustomerResponse::from).collect(),
        meta: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

// Get customer's saved payment methods
#[utoipa::path(
    get,
    path = "/v1/customers/{id}/payment-methods",
    params(
        ("id" = Uuid, Path, description = "Customer ID")
    ),
    responses(
        (status = 200, description = "Customer payment methods", body = Vec<SavedPaymentMethodResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_customer_payment_methods(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let customer_id = path.into_inner();
    let mut conn = state.db.acquire().await?;

    let methods_rows = sqlx::query(
        "SELECT * FROM saved_payment_methods 
         WHERE customer_id = $1 AND merchant_id = $2 AND is_active = true AND deleted_at IS NULL
         ORDER BY is_default DESC, created_at DESC",
    )
    .bind(customer_id)
    .bind(merchant_id)
    .fetch_all(&mut *conn)
    .await?;

    let methods: Vec<SavedPaymentMethod> = methods_rows
        .into_iter()
        .map(SavedPaymentMethod::from)
        .collect();

    Ok(HttpResponse::Ok().json(
        methods
            .into_iter()
            .map(SavedPaymentMethodResponse::from)
            .collect::<Vec<_>>(),
    ))
}

// Delete customer (soft delete)
#[utoipa::path(
    delete,
    path = "/v1/customers/{id}",
    params(
        ("id" = Uuid, Path, description = "Customer ID")
    ),
    responses(
        (status = 204, description = "Customer deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Customer not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn delete_customer(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let customer_id = path.into_inner();
    let mut conn = state.db.acquire().await?;

    sqlx::query("UPDATE customers SET deleted_at = NOW() WHERE id = $1 AND merchant_id = $2")
        .bind(customer_id)
        .bind(merchant_id)
        .execute(&mut *conn)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/customers")
            .route("", web::post().to(create_customer))
            .route("", web::get().to(list_customers))
            .route("/{id}", web::get().to(get_customer))
            .route("/{id}", web::delete().to(delete_customer))
            .route(
                "/{id}/payment-methods",
                web::get().to(get_customer_payment_methods),
            ),
    );
}

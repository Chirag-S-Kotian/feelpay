use crate::error::FeelPayError;
use crate::models::virtual_account::{CreateVirtualAccountRequest, VirtualAccount};
use crate::state::AppState;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("", web::post().to(create_virtual_account))
            .route("", web::get().to(list_virtual_accounts))
            .route("/{id}", web::get().to(get_virtual_account)),
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
    path = "/v1/virtual-accounts",
    request_body = CreateVirtualAccountRequest,
    responses(
        (status = 200, description = "Virtual account created successfully", body = VirtualAccount),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn create_virtual_account(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateVirtualAccountRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Generate a virtual account number
    // Format: FEELP + first 8 chars of merchant UUID + random 6 digits
    let merchant_prefix = &merchant_id.to_string()[..8].to_uppercase();
    let random_suffix: u32 = rand::random::<u32>() % 1_000_000;
    let account_number = format!("FEELP{}{:06}", merchant_prefix, random_suffix);

    // Static IFSC for FeelPay Virtual Bank (Simulated)
    let ifsc_code = "FEEL0000001";
    let bank_name = "FeelPay Virtual Bank";

    let va = sqlx::query_as::<_, VirtualAccount>(
        r#"
        INSERT INTO virtual_accounts (
            merchant_id, customer_id, account_number, ifsc_code, bank_name, beneficiary_name, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(merchant_id)
    .bind(body.customer_id)
    .bind(&account_number)
    .bind(ifsc_code)
    .bind(bank_name)
    .bind(body.beneficiary_name.as_deref().unwrap_or("FeelPay Beneficiary"))
    .bind(&body.metadata)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(va))
}

#[utoipa::path(
    get,
    path = "/v1/virtual-accounts",
    responses(
        (status = 200, description = "List of virtual accounts", body = [VirtualAccount]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn list_virtual_accounts(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let accounts = sqlx::query_as::<_, VirtualAccount>(
        "SELECT * FROM virtual_accounts WHERE merchant_id = $1 ORDER BY created_at DESC",
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(accounts))
}

#[utoipa::path(
    get,
    path = "/v1/virtual-accounts/{id}",
    params(
        ("id" = Uuid, Path, description = "Virtual Account ID")
    ),
    responses(
        (status = 200, description = "Virtual account details", body = VirtualAccount),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Virtual account not found")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_virtual_account(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let va_id = path.into_inner();

    let account = sqlx::query_as::<_, VirtualAccount>(
        "SELECT * FROM virtual_accounts WHERE id = $1 AND merchant_id = $2",
    )
    .bind(va_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match account {
        Some(acc) => Ok(HttpResponse::Ok().json(acc)),
        None => Err(FeelPayError::NotFound("Virtual account not found".into())),
    }
}

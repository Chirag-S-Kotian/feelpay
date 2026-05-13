use crate::engine::services::bank_account::{
    mask_bank_account, AddBankAccountRequest, BankAccountService,
};
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use crate::state::AppState;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Bank account response (with masked account number)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountResponse {
    pub id: String,
    pub account_holder_name: String,
    pub account_number_masked: String,
    pub bank_name: Option<String>,
    pub ifsc_code: String,
    pub account_type: String,
    pub currency: String,
    pub verification_status: String,
    pub is_primary: bool,
    pub created_at: String,
    pub verified_at: Option<String>,
}

/// Bank accounts list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountsListResponse {
    pub accounts: Vec<BankAccountResponse>,
    pub total: i64,
}

/// Add bank account request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBankAccountApiRequest {
    pub account_holder_name: String,
    pub bank_name: Option<String>,
    pub account_number: String,
    pub confirm_account_number: Option<String>,
    pub ifsc_code: String,
    pub account_type: String,
    pub currency: Option<String>,
    pub is_primary: Option<bool>,
}

/// Add bank account response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBankAccountApiResponse {
    pub id: String,
    pub verification_status: String,
    pub penny_drop_reference: Option<String>,
}

/// Verify bank account request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyBankAccountRequest {
    pub penny_drop_amount_paise: i32,
}

/// Add bank account endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/bank-accounts",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    request_body = AddBankAccountApiRequest,
    responses(
        (status = 200, description = "Bank account added", body = AddBankAccountApiResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn add_bank_account(
    merchant_id: web::Path<Uuid>,
    req: web::Json<AddBankAccountApiRequest>,
    claims: JwtClaims,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }
    if req.account_holder_name.trim().is_empty() {
        return Err(FeelPayError::BadRequest(
            "Account holder name is required".into(),
        ));
    }
    if req
        .confirm_account_number
        .as_deref()
        .is_some_and(|confirm| confirm != req.account_number)
    {
        return Err(FeelPayError::BadRequest(
            "Confirm account number does not match".into(),
        ));
    }
    let currency = req
        .currency
        .as_deref()
        .unwrap_or("INR")
        .to_ascii_uppercase();
    if currency != "INR" {
        return Err(FeelPayError::BadRequest(
            "Only INR settlement accounts are supported".into(),
        ));
    }

    let bank_account_service =
        BankAccountService::new(state.db.clone(), &state.config.vault_encryption_key)?;

    let add_req = AddBankAccountRequest {
        account_holder_name: req.account_holder_name.clone(),
        account_number: req.account_number.clone(),
        ifsc_code: req.ifsc_code.clone(),
        account_type: req.account_type.clone(),
    };

    let account = bank_account_service
        .add_bank_account(merchant_id, add_req)
        .await?;

    if req.is_primary.unwrap_or(false) {
        let _ = bank_account_service
            .set_primary_bank_account(merchant_id, account.id)
            .await;
    }

    Ok(HttpResponse::Ok().json(AddBankAccountApiResponse {
        id: account.id.to_string(),
        verification_status: account.verification_status.as_str().to_string(),
        penny_drop_reference: account.penny_drop_reference,
    }))
}

/// List bank accounts endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/bank-accounts",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("limit" = Option<i64>, Query, description = "Limit"),
        ("offset" = Option<i64>, Query, description = "Offset")
    ),
    responses(
        (status = 200, description = "Bank accounts list", body = BankAccountsListResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_bank_accounts(
    merchant_id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let encryption_key = std::env::var("VAULT_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

    let bank_account_service = BankAccountService::new(db.get_ref().clone(), &encryption_key)?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let (accounts, total) = bank_account_service
        .list_bank_accounts(merchant_id, limit, offset)
        .await?;

    let items = accounts
        .into_iter()
        .map(|account| BankAccountResponse {
            id: account.id.to_string(),
            account_holder_name: account.account_holder_name,
            account_number_masked: mask_bank_account(&account.account_number_encrypted),
            bank_name: None,
            ifsc_code: account.ifsc_code,
            account_type: account.account_type.as_str().to_string(),
            currency: "INR".to_string(),
            verification_status: account.verification_status.as_str().to_string(),
            is_primary: account.is_primary,
            created_at: account.created_at.to_rfc3339(),
            verified_at: account.verified_at.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(BankAccountsListResponse {
        accounts: items,
        total,
    }))
}

/// Verify bank account endpoint
#[utoipa::path(
    post,
    path = "/api/v1/merchants/{merchant_id}/bank-accounts/{account_id}/verify",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("account_id" = Uuid, Path, description = "Bank account ID")
    ),
    request_body = VerifyBankAccountRequest,
    responses(
        (status = 200, description = "Bank account verified", body = BankAccountResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn verify_bank_account(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<VerifyBankAccountRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, account_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let encryption_key = std::env::var("VAULT_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

    let bank_account_service = BankAccountService::new(db.get_ref().clone(), &encryption_key)?;

    let account = bank_account_service
        .verify_bank_account(merchant_id, account_id, req.penny_drop_amount_paise)
        .await?;

    Ok(HttpResponse::Ok().json(BankAccountResponse {
        id: account.id.to_string(),
        account_holder_name: account.account_holder_name,
        account_number_masked: mask_bank_account(&account.account_number_encrypted),
        bank_name: None,
        ifsc_code: account.ifsc_code,
        account_type: account.account_type.as_str().to_string(),
        currency: "INR".to_string(),
        verification_status: account.verification_status.as_str().to_string(),
        is_primary: account.is_primary,
        created_at: account.created_at.to_rfc3339(),
        verified_at: account.verified_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// Remove/deactivate bank account endpoint
pub async fn delete_bank_account(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, account_id) = path.into_inner();

    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let result = sqlx::query(
        r#"
        UPDATE bank_accounts
        SET verification_status = 'deactivated', is_primary = false, updated_at = NOW()
        WHERE id = $1 AND merchant_id = $2
        "#,
    )
    .bind(account_id)
    .bind(merchant_id)
    .execute(db.get_ref())
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(FeelPayError::NotFound("Bank account not found".into()));
    }

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_logs (entity_type, entity_id, action, new_state, actor_type, actor_id)
        VALUES ('bank_account', $1, 'bank_account_deactivated', $2::jsonb, 'merchant_user', $3)
        "#,
    )
    .bind(account_id)
    .bind(serde_json::json!({"merchant_id": merchant_id}))
    .bind(merchant_id)
    .execute(db.get_ref())
    .await;

    Ok(HttpResponse::NoContent().finish())
}

/// Set primary bank account endpoint
#[utoipa::path(
    put,
    path = "/api/v1/merchants/{merchant_id}/bank-accounts/{account_id}/set-primary",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("account_id" = Uuid, Path, description = "Bank account ID")
    ),
    responses(
        (status = 204, description = "Bank account set as primary"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn set_primary_bank_account(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, account_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let encryption_key = std::env::var("VAULT_ENCRYPTION_KEY").unwrap_or_else(|_| "0".repeat(64));

    let bank_account_service = BankAccountService::new(db.get_ref().clone(), &encryption_key)?;

    bank_account_service
        .set_primary_bank_account(merchant_id, account_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Pagination query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

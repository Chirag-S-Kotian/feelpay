use crate::engine::payout;
use crate::engine::services::bank_account::{
    mask_bank_account, AddBankAccountRequest, BankAccountService,
};
use crate::models::payout::{BankAccount, CreateBankAccountRequest, CreatePayoutRequest};
use crate::models::user::Claims;
use crate::state::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(request_payout)
            .service(list_payouts)
            .service(get_bank_accounts)
            .service(add_bank_account)
            .service(verify_account),
    );
}

#[post("")]
async fn request_payout(
    pool: web::Data<PgPool>,
    req: web::Json<CreatePayoutRequest>,
    claims: Claims,
) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let bank_account_id = match req.bank_account_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("Bank account ID is required"),
    };
    let _ = (pool, req, merchant_id, bank_account_id);
    HttpResponse::Forbidden().body("Payout creation is internal-admin only")
}

#[derive(Debug, serde::Serialize)]
struct SafePayoutBankAccount {
    id: Uuid,
    account_holder_name: String,
    account_number_masked: String,
    ifsc_code: String,
    bank_name: Option<String>,
    is_verified: bool,
    verification_status: String,
    is_primary: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[get("")]
async fn list_payouts(pool: web::Data<PgPool>, claims: Claims) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let onboarding_status = sqlx::query("SELECT onboarding_status FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_one(pool.get_ref())
        .await;

    let onboarding_status: String = match onboarding_status {
        Ok(row) => row.get(0),
        Err(_) => return HttpResponse::NotFound().body("Merchant not found"),
    };

    if onboarding_status != "approved" {
        return HttpResponse::Ok().json(Vec::<serde_json::Value>::new());
    }

    let payouts = sqlx::query(
        r#"
        SELECT id, amount, currency, status, bank_account_id,
               provider_payout_id, manual_reference, failure_reason, processed_at, created_at
        FROM payouts_company
        WHERE business_account_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match payouts {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.get::<Uuid, _>("id"),
                        "amount": row.get::<i64, _>("amount"),
                        "currency": row.get::<String, _>("currency"),
                        "status": row.get::<String, _>("status"),
                        "method": "internal",
                        "bank_account_id": row.get::<Option<Uuid>, _>("bank_account_id"),
                        "payout_reference": row.get::<Option<String>, _>("manual_reference"),
                        "gateway_reference": row.get::<Option<String>, _>("provider_payout_id"),
                        "error_message": row.get::<Option<String>, _>("failure_reason"),
                        "processed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("processed_at"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    })
                })
                .collect::<Vec<_>>(),
        ),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/bank-accounts")]
async fn get_bank_accounts(pool: web::Data<PgPool>, claims: Claims) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let accounts = sqlx::query_as::<_, BankAccount>(
        "SELECT * FROM bank_accounts WHERE merchant_id = $1 AND COALESCE(verification_status, CASE WHEN is_verified THEN 'verified' ELSE 'pending' END) <> 'deactivated' ORDER BY is_primary DESC, created_at DESC",
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match accounts {
        Ok(a) => HttpResponse::Ok().json(
            a.into_iter()
                .map(|account| {
                    let raw = account.account_number;
                    let last4 = if raw.len() >= 4 {
                        &raw[raw.len() - 4..]
                    } else {
                        "----"
                    };
                    let status = if account.is_verified {
                        "verified".to_string()
                    } else {
                        "pending".to_string()
                    };
                    SafePayoutBankAccount {
                        id: account.id,
                        account_holder_name: account.account_holder_name,
                        account_number_masked: format!("XXXX...{}", last4),
                        ifsc_code: account.ifsc_code,
                        bank_name: account.bank_name,
                        is_verified: account.is_verified || status == "verified",
                        verification_status: status,
                        is_primary: account.is_primary,
                        created_at: account.created_at,
                        updated_at: account.updated_at,
                    }
                })
                .collect::<Vec<_>>(),
        ),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/bank-accounts")]
async fn add_bank_account(
    state: web::Data<AppState>,
    req: web::Json<CreateBankAccountRequest>,
    claims: Claims,
) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let service =
        match BankAccountService::new(state.db.clone(), &state.config.vault_encryption_key) {
            Ok(service) => service,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        };

    let account = service
        .add_bank_account(
            merchant_id,
            AddBankAccountRequest {
                account_holder_name: req.account_holder_name.trim().to_string(),
                account_number: req.account_number.trim().to_string(),
                ifsc_code: req.ifsc_code.trim().to_uppercase(),
                account_type: "current".to_string(),
            },
        )
        .await;

    match account {
        Ok(a) => {
            if req.is_primary.unwrap_or(false) {
                if let Err(e) = service.set_primary_bank_account(merchant_id, a.id).await {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }
            HttpResponse::Created().json(SafePayoutBankAccount {
                id: a.id,
                account_holder_name: a.account_holder_name,
                account_number_masked: mask_bank_account(req.account_number.trim()),
                ifsc_code: a.ifsc_code,
                bank_name: req.bank_name.clone(),
                is_verified: false,
                verification_status: a.verification_status.as_str().to_string(),
                is_primary: req.is_primary.unwrap_or(false),
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/bank-accounts/{id}/verify")]
async fn verify_account(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: Claims,
) -> impl Responder {
    let bank_account_id = path.into_inner();
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    // Verify ownership
    let account_exists =
        sqlx::query("SELECT id FROM bank_accounts WHERE id = $1 AND merchant_id = $2")
            .bind(bank_account_id)
            .bind(merchant_id)
            .fetch_optional(pool.get_ref())
            .await;

    match account_exists {
        Ok(Some(_)) => (),
        Ok(None) => return HttpResponse::NotFound().body("Bank account not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    match payout::verify_bank_account(pool.get_ref(), bank_account_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Verification payout (penny drop) initiated."
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

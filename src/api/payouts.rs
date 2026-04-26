use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use crate::models::payout::{Payout, BankAccount, CreatePayoutRequest, CreateBankAccountRequest};
use crate::models::user::Claims;
use crate::engine::payout;
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payouts")
            .service(request_payout)
            .service(list_payouts)
            .service(get_bank_accounts)
            .service(add_bank_account)
            .service(verify_account)
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

    // Verify bank account belongs to merchant
    let account_exists = sqlx::query(
        "SELECT id FROM bank_accounts WHERE id = $1 AND merchant_id = $2"
    )
    .bind(bank_account_id)
    .bind(merchant_id)
    .fetch_optional(pool.get_ref())
    .await;

    match account_exists {
        Ok(Some(_)) => (),
        Ok(None) => return HttpResponse::BadRequest().body("Invalid bank account"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    let payout = sqlx::query_as::<_, Payout>(
        "INSERT INTO payouts (merchant_id, amount, currency, method, bank_account_id, status, metadata) 
         VALUES ($1, $2, $3, $4, $5, 'created', $6) RETURNING *"
    )
    .bind(merchant_id)
    .bind(req.amount)
    .bind(&req.currency)
    .bind(&req.method)
    .bind(bank_account_id)
    .bind(&req.metadata)
    .fetch_one(pool.get_ref())
    .await;

    match payout {
        Ok(p) => {
            // Trigger background processing (simulation)
            let pool_clone = pool.get_ref().clone();
            let payout_id = p.id;
            actix_web::rt::spawn(async move {
                let _ = payout::process_payout(&pool_clone, payout_id).await;
            });
            HttpResponse::Created().json(p)
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("")]
async fn list_payouts(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let payouts = sqlx::query_as::<_, Payout>(
        "SELECT * FROM payouts WHERE merchant_id = $1 ORDER BY created_at DESC"
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match payouts {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/bank-accounts")]
async fn get_bank_accounts(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let accounts = sqlx::query_as::<_, BankAccount>(
        "SELECT * FROM bank_accounts WHERE merchant_id = $1 ORDER BY created_at DESC"
    )
    .bind(merchant_id)
    .fetch_all(pool.get_ref())
    .await;

    match accounts {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/bank-accounts")]
async fn add_bank_account(
    pool: web::Data<PgPool>,
    req: web::Json<CreateBankAccountRequest>,
    claims: Claims,
) -> impl Responder {
    let merchant_id = match claims.merchant_id {
        Some(id) => id,
        None => return HttpResponse::Forbidden().body("Merchant ID missing in token"),
    };

    let account = sqlx::query_as::<_, BankAccount>(
        "INSERT INTO bank_accounts (merchant_id, account_holder_name, account_number, ifsc_code, bank_name, is_primary) 
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    )
    .bind(merchant_id)
    .bind(&req.account_holder_name)
    .bind(&req.account_number)
    .bind(&req.ifsc_code)
    .bind(&req.bank_name)
    .bind(req.is_primary.unwrap_or(false))
    .fetch_one(pool.get_ref())
    .await;

    match account {
        Ok(a) => HttpResponse::Created().json(a),
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
    let account_exists = sqlx::query(
        "SELECT id FROM bank_accounts WHERE id = $1 AND merchant_id = $2"
    )
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

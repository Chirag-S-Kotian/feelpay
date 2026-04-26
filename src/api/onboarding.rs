use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding")
            .route("/kyc", web::post().to(submit_kyc))
            .route("/status", web::get().to(onboarding_status))
            .route("/bank-account", web::post().to(submit_bank_account)),
    );
}

#[derive(Debug, Deserialize)]
pub struct KycSubmissionRequest {
    pub verification_type: String, // pan/gstin/bank/aadhaar
    pub document_number: String,
    pub document_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BankAccountRequest {
    pub account_holder_name: String,
    pub account_number: String,
    pub ifsc_code: String,
    pub bank_name: Option<String>,
}

fn merchant_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::MerchantId>()
        .map(|m| m.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing merchant context".into()))
}

fn validate_pan(pan: &str) -> bool {
    Regex::new(r"^[A-Z]{5}[0-9]{4}[A-Z]$")
        .ok()
        .map(|re| re.is_match(pan))
        .unwrap_or(false)
}

fn validate_gstin(gstin: &str) -> bool {
    Regex::new(r"^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[A-Z0-9]{1}Z[A-Z0-9]{1}$")
        .ok()
        .map(|re| re.is_match(gstin))
        .unwrap_or(false)
}

fn validate_ifsc(ifsc: &str) -> bool {
    Regex::new(r"^[A-Z]{4}0[A-Z0-9]{6}$")
        .ok()
        .map(|re| re.is_match(ifsc))
        .unwrap_or(false)
}

pub async fn submit_kyc(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<KycSubmissionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let verification_type = body.verification_type.to_lowercase();
    let document_number = body.document_number.trim().to_uppercase();

    match verification_type.as_str() {
        "pan" if !validate_pan(&document_number) => {
            return Err(FeelPayError::BadRequest("Invalid PAN format".into()));
        }
        "gstin" if !validate_gstin(&document_number) => {
            return Err(FeelPayError::BadRequest("Invalid GSTIN format".into()));
        }
        "bank" | "aadhaar" | "pan" | "gstin" => {}
        _ => return Err(FeelPayError::BadRequest("Unsupported verification type".into())),
    }

    let status = if verification_type == "pan" || verification_type == "gstin" {
        "verified"
    } else {
        "pending"
    };

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO kyc_verifications (merchant_id, verification_type, document_number, document_data, status, verified_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $5 = 'verified' THEN NOW() ELSE NULL END)
        RETURNING id
        "#,
    )
    .bind(merchant_id)
    .bind(&verification_type)
    .bind(&document_number)
    .bind(&body.document_data)
    .bind(status)
    .fetch_one(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    if status == "verified" {
        sqlx::query(
            r#"
            UPDATE merchants
            SET onboarding_status = 'in_progress', updated_at = NOW()
            WHERE id = $1 AND onboarding_status = 'pending'
            "#,
        )
        .bind(merchant_id)
        .execute(&state.db)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": id,
        "verification_type": verification_type,
        "status": status
    })))
}

pub async fn submit_bank_account(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<BankAccountRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let ifsc = body.ifsc_code.trim().to_uppercase();

    if !validate_ifsc(&ifsc) {
        return Err(FeelPayError::BadRequest("Invalid IFSC format".into()));
    }
    if body.account_number.trim().len() < 8 || body.account_number.trim().len() > 20 {
        return Err(FeelPayError::BadRequest("Invalid account number".into()));
    }

    let settlement = serde_json::json!({
        "account_holder_name": body.account_holder_name.trim(),
        "account_number_masked": format!("****{}", &body.account_number[body.account_number.len().saturating_sub(4)..]),
        "ifsc_code": ifsc,
        "bank_name": body.bank_name.as_deref().unwrap_or(""),
        "verification_status": "penny_drop_pending"
    });

    sqlx::query(
        r#"
        UPDATE merchants
        SET settlement_account = $2,
            onboarding_status = 'verified',
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&settlement)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "verified",
        "settlement_account": settlement
    })))
}

pub async fn onboarding_status(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let merchant: Option<(String, String, i64, i64)> = sqlx::query_as(
        "SELECT onboarding_status, tier, daily_limit, monthly_limit FROM merchants WHERE id = $1",
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let Some((onboarding_status, tier, daily_limit, monthly_limit)) = merchant else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let kyc_rows: Vec<(Uuid, String, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        r#"
        SELECT id, verification_type, document_number, status, verified_at
        FROM kyc_verifications
        WHERE merchant_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::DatabaseError(e.to_string()))?;

    let verifications: Vec<serde_json::Value> = kyc_rows
        .into_iter()
        .map(|(id, verification_type, document_number, status, verified_at)| {
            serde_json::json!({
                "id": id,
                "verification_type": verification_type,
                "document_number": document_number,
                "status": status,
                "verified_at": verified_at,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "onboarding_status": onboarding_status,
        "tier": tier,
        "daily_limit": daily_limit,
        "monthly_limit": monthly_limit,
        "verifications": verifications
    })))
}

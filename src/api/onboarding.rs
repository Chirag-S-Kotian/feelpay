use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use regex::Regex;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding")
            .route("/kyc", web::post().to(submit_kyc))
            .route("/status", web::get().to(onboarding_status))
            .route("/bank-account", web::post().to(submit_bank_account))
            .route("/registration/step", web::post().to(update_registration_step))
            .route("/registration/progress", web::get().to(get_registration_progress))
            .route("/business-info", web::post().to(submit_business_info))
            .route("/branding", web::post().to(update_branding))
            .route("/limits", web::get().to(get_limits))
            .route("/limits", web::put().to(update_limits)),
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
        _ => {
            return Err(FeelPayError::BadRequest(
                "Unsupported verification type".into(),
            ))
        }
    }

    let status = if verification_type == "pan" || verification_type == "gstin" {
        "verified"
    } else {
        "pending"
    };

    let id: Uuid = sqlx::query(
        r#"
        INSERT INTO kyc_verifications (merchant_id, verification_type, document_number, document_data, status, verified_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $5::text = 'verified' THEN NOW() ELSE NULL END)
        RETURNING id
        "#,
    )
    .bind(merchant_id)
    .bind(verification_type.clone())
    .bind(document_number)
    .bind(body.document_data.as_ref())
    .bind(status)
    .fetch_one(&state.db)
    .await?
    .get(0);

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
        .await?;
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
    .await?;

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

    let row = sqlx::query(
        "SELECT onboarding_status, tier, daily_limit, monthly_limit FROM merchants WHERE id = $1",
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(FeelPayError::NotFound("Merchant not found".into()));
    };

    let onboarding_status: String = row.get(0);
    let tier: String = row.get(1);
    let daily_limit: i64 = row.get(2);
    let monthly_limit: i64 = row.get(3);

    let kyc_rows = sqlx::query(
        r#"
        SELECT id, verification_type, document_number, status, verified_at
        FROM kyc_verifications
        WHERE merchant_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await?;

    let verifications: Vec<serde_json::Value> = kyc_rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>(0),
                "verification_type": row.get::<String, _>(1),
                "document_number": row.get::<Option<String>, _>(2),
                "status": row.get::<String, _>(3),
                "verified_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>(4),
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
// Update registration step
#[derive(Debug, Deserialize)]
pub struct RegistrationStepRequest {
    pub step: i32,
    pub data: Option<serde_json::Value>,
}

pub async fn update_registration_step(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<RegistrationStepRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    sqlx::query(
        "UPDATE merchants SET registration_step = $1, registration_data = COALESCE(registration_data, '{}'::jsonb) || $2, updated_at = NOW() WHERE id = $3"
    )
    .bind(body.step)
    .bind(body.data.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(merchant_id)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "step": body.step,
        "message": "Registration step updated"
    })))
}

// Get registration progress
pub async fn get_registration_progress(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    let row = sqlx::query(
        "SELECT registration_step, registration_data, kyc_status FROM merchants m LEFT JOIN merchant_kyc mk ON m.id = mk.merchant_id WHERE m.id = $1"
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => {
            let step: i32 = row.get(0);
            let data: serde_json::Value = row.get(1);
            let kyc_status: Option<String> = row.get(2);

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "current_step": step,
                "registration_data": data,
                "kyc_status": kyc_status.unwrap_or_else(|| "pending".to_string()),
                "steps": [
                    {"step": 1, "name": "Business Info", "completed": step > 1},
                    {"step": 2, "name": "KYC Documents", "completed": step > 2},
                    {"step": 3, "name": "Bank Account", "completed": step > 3},
                    {"step": 4, "name": "API Keys", "completed": step > 4},
                    {"step": 5, "name": "Complete", "completed": step > 5}
                ]
            })))
        },
        None => Err(FeelPayError::NotFound("Merchant not found".into()))
    }
}

// Submit business info
#[derive(Debug, Deserialize)]
pub struct BusinessInfoRequest {
    pub business_name: String,
    pub business_type: String,
    pub pan: String,
    pub gstin: Option<String>,
    pub address: Option<serde_json::Value>,
}

pub async fn submit_business_info(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<BusinessInfoRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    // Validate business type
    let valid_types = ["pvt_ltd", "llp", "sole_proprietor", "partnership", "trust", "ngo"];
    if !valid_types.contains(&body.business_type.as_str()) {
        return Err(FeelPayError::BadRequest("Invalid business type".to_string()));
    }
    
    // Validate PAN
    if !validate_pan(&body.pan) {
        return Err(FeelPayError::BadRequest("Invalid PAN number".to_string()));
    }
    
    // Validate GSTIN if provided
    if let Some(ref gstin) = body.gstin {
        if !gstin.is_empty() && !validate_gstin(gstin) {
            return Err(FeelPayError::BadRequest("Invalid GSTIN".to_string()));
        }
    }

    sqlx::query(
        "UPDATE merchants SET business_name = $1, business_type = $2, pan = $3, gstin = $4, address = $5, registration_step = 2, updated_at = NOW() WHERE id = $6"
    )
    .bind(&body.business_name)
    .bind(&body.business_type)
    .bind(body.pan.to_uppercase())
    .bind(body.gstin.as_ref().map(|s| s.to_uppercase()))
    .bind(&body.address)
    .bind(merchant_id)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Business info saved successfully",
        "next_step": 2
    })))
}

// Update branding
#[derive(Debug, Deserialize)]
pub struct BrandingRequest {
    pub logo_url: Option<String>,
    pub color: Option<String>,
}

pub async fn update_branding(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<BrandingRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    sqlx::query(
        "UPDATE merchants SET branding_logo = $1, branding_color = $2, updated_at = NOW() WHERE id = $3"
    )
    .bind(&body.logo_url)
    .bind(&body.color)
    .bind(merchant_id)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Branding updated successfully"
    })))
}

// Get transaction limits
pub async fn get_limits(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    let row = sqlx::query(
        "SELECT per_transaction_limit, daily_limit, monthly_limit, settlement_cycle FROM merchants WHERE id = $1"
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "per_transaction_limit": row.get::<i64, _>(0),
            "daily_limit": row.get::<i64, _>(1),
            "monthly_limit": row.get::<i64, _>(2),
            "settlement_cycle": row.get::<String, _>(3)
        }))),
        None => Err(FeelPayError::NotFound("Merchant not found".into()))
    }
}

// Update transaction limits
#[derive(Debug, Deserialize)]
pub struct LimitsRequest {
    pub per_transaction_limit: Option<i64>,
    pub daily_limit: Option<i64>,
    pub monthly_limit: Option<i64>,
    pub settlement_cycle: Option<String>,
}

pub async fn update_limits(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<LimitsRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    
    // Validate settlement cycle
    if let Some(ref cycle) = body.settlement_cycle {
        let valid_cycles = ["t+1", "t+2", "instant"];
        if !valid_cycles.contains(&cycle.as_str()) {
            return Err(FeelPayError::BadRequest("Invalid settlement cycle".to_string()));
        }
    }

    sqlx::query(
        r#"
        UPDATE merchants SET 
            per_transaction_limit = COALESCE($1, per_transaction_limit),
            daily_limit = COALESCE($2, daily_limit),
            monthly_limit = COALESCE($3, monthly_limit),
            settlement_cycle = COALESCE($4, settlement_cycle),
            updated_at = NOW()
        WHERE id = $5
        "#,
    )
    .bind(body.per_transaction_limit)
    .bind(body.daily_limit)
    .bind(body.monthly_limit)
    .bind(&body.settlement_cycle)
    .bind(merchant_id)
    .execute(&state.db)
    .await?;


    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Limits updated successfully"
    })))
}
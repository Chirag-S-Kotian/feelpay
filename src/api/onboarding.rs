use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding")
            .route("/kyc", web::post().to(submit_kyc))
            .route("/status", web::get().to(onboarding_status))
            .route("/bank-account", web::post().to(submit_bank_account))
            .route(
                "/registration/step",
                web::post().to(update_registration_step),
            )
            .route(
                "/registration/progress",
                web::get().to(get_registration_progress),
            )
            .route("/business-info", web::post().to(submit_business_info))
            .route("/branding", web::post().to(update_branding))
            .route("/limits", web::get().to(get_limits))
            .route("/limits", web::put().to(update_limits))
            .route("/aadhaar/initiate", web::post().to(initiate_aadhaar_ekyc))
            .route("/aadhaar/verify", web::post().to(verify_aadhaar_otp)),
    );
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct KycSubmissionRequest {
    pub verification_type: String, // pan/gstin/bank/aadhaar
    pub document_number: String,
    pub document_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BankAccountSubmissionRequest {
    pub account_holder_name: String,
    pub account_number: String,
    pub ifsc_code: String,
    pub bank_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InitiateAadhaarRequest {
    pub aadhaar_number: String,
    pub phone: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyAadhaarOtpRequest {
    pub aadhaar_number: String,
    pub otp: String,
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

#[utoipa::path(
    post,
    path = "/v1/onboarding/kyc",
    request_body = KycSubmissionRequest,
    responses(
        (status = 200, description = "KYC submitted successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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

#[utoipa::path(
    post,
    path = "/v1/onboarding/bank-account",
    request_body = BankAccountSubmissionRequest,
    responses(
        (status = 200, description = "Bank account submitted successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn submit_bank_account(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<BankAccountSubmissionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let ifsc = body.ifsc_code.trim().to_uppercase();

    if !validate_ifsc(&ifsc) {
        return Err(FeelPayError::BadRequest("Invalid IFSC format".into()));
    }
    if body.account_number.trim().len() < 8 || body.account_number.trim().len() > 20 {
        return Err(FeelPayError::BadRequest("Invalid account number".into()));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // 1. Insert into bank_accounts table
    sqlx::query(
        r#"
        INSERT INTO bank_accounts (merchant_id, account_holder_name, account_number, ifsc_code, bank_name, is_primary)
        VALUES ($1, $2, $3, $4, $5, true)
        ON CONFLICT (merchant_id, account_number) DO UPDATE 
        SET account_holder_name = $2, ifsc_code = $4, bank_name = $5, is_verified = false, updated_at = NOW()
        "#
    )
    .bind(merchant_id)
    .bind(body.account_holder_name.trim())
    .bind(body.account_number.trim())
    .bind(&ifsc)
    .bind(body.bank_name.as_deref().unwrap_or(""))
    .execute(&mut *tx)
    .await?;

    // 2. Update merchant settlement_account JSONB for backward compatibility
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
            registration_step = CASE WHEN registration_step < 4 THEN 4 ELSE registration_step END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&settlement)
    .execute(&mut *tx)
    .await?;

    tx.commit()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // 3. Trigger Penny Drop verification simulation
    let pool = state.db.clone();
    let _account_num = body.account_number.trim().to_string();
    let _ifsc_val = ifsc.clone();

    tokio::spawn(async move {
        // Wait a bit to simulate async processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let _ = crate::engine::payout::verify_bank_account(&pool, merchant_id).await;
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "penny_drop_initiated",
        "message": "Bank account submitted. Penny drop verification initiated.",
        "settlement_account": settlement
    })))
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/aadhaar/initiate",
    request_body = InitiateAadhaarRequest,
    responses(
        (status = 200, description = "Aadhaar OTP sent successfully"),
        (status = 400, description = "Invalid Aadhaar number"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn initiate_aadhaar_ekyc(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<InitiateAadhaarRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    if body.aadhaar_number.len() != 12 || !body.aadhaar_number.chars().all(|c| c.is_ascii_digit()) {
        return Err(FeelPayError::BadRequest(
            "Invalid Aadhaar number. Must be 12 digits.".into(),
        ));
    }

    // Generate 6-digit OTP
    let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

    // Hash OTP for verification
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(otp.as_bytes());
    let otp_hash = format!("{:x}", hasher.finalize());

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

    // Store OTP in database
    sqlx::query(
        r#"
        INSERT INTO otp_verifications (phone, otp_hash, otp_type, expires_at, metadata)
        VALUES ($1, $2, 'aadhaar_ekyc', $3, $4)
        "#,
    )
    .bind(&body.phone)
    .bind(&otp_hash)
    .bind(expires_at)
    .bind(serde_json::json!({"merchant_id": merchant_id, "aadhaar_number": body.aadhaar_number}))
    .execute(&state.db)
    .await?;

    // Simulate sending SMS
    let _ =
        crate::engine::services::sms::send_otp_sms(&state.db, &body.phone, &otp, "Aadhaar eKYC")
            .await;

    // For simulation, we return the OTP in the response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OTP sent to your registered mobile number",
        "phone_masked": format!("******{}", &body.phone[body.phone.len().saturating_sub(4)..]),
        "otp_simulated": otp // Only for dev/testing
    })))
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/aadhaar/verify",
    request_body = VerifyAadhaarOtpRequest,
    responses(
        (status = 200, description = "Aadhaar verified successfully"),
        (status = 400, description = "Invalid OTP"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn verify_aadhaar_otp(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<VerifyAadhaarOtpRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Hash the provided OTP
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(body.otp.as_bytes());
    let otp_hash = format!("{:x}", hasher.finalize());

    // Verify OTP from database
    let row = sqlx::query(
        r#"
        SELECT id, phone, metadata FROM otp_verifications
        WHERE otp_hash = $1 
          AND otp_type = 'aadhaar_ekyc'
          AND status = 'pending'
          AND expires_at > NOW()
        ORDER BY created_at DESC LIMIT 1
        "#,
    )
    .bind(&otp_hash)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(FeelPayError::BadRequest("Invalid or expired OTP".into()));
    };

    let otp_id: Uuid = row.get(0);
    let phone: String = row.get(1);
    let metadata: serde_json::Value = row.get(2);

    // Validate that this OTP was for this merchant and Aadhaar
    if metadata["merchant_id"] != serde_json::json!(merchant_id)
        || metadata["aadhaar_number"] != serde_json::json!(body.aadhaar_number)
    {
        return Err(FeelPayError::BadRequest(
            "OTP does not match Aadhaar session".into(),
        ));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Mark OTP as verified
    sqlx::query(
        "UPDATE otp_verifications SET status = 'verified', verified_at = NOW() WHERE id = $1",
    )
    .bind(otp_id)
    .execute(&mut *tx)
    .await?;

    // Update merchant record
    let aadhaar_masked = format!(
        "********{}",
        &body.aadhaar_number[body.aadhaar_number.len().saturating_sub(4)..]
    );
    sqlx::query(
        "UPDATE merchants SET phone = $1, aadhaar_number_masked = $2, aadhaar_verified = true, registration_step = CASE WHEN registration_step < 3 THEN 3 ELSE registration_step END, updated_at = NOW() WHERE id = $3"
    )
    .bind(&phone)
    .bind(&aadhaar_masked)
    .bind(merchant_id)
    .execute(&mut *tx)
    .await?;

    // Create KYC verification record
    sqlx::query(
        r#"
        INSERT INTO kyc_verifications (merchant_id, verification_type, document_number, status, verified_at)
        VALUES ($1, 'aadhaar', $2, 'verified', NOW())
        "#
    )
    .bind(merchant_id)
    .bind(&aadhaar_masked)
    .execute(&mut *tx)
    .await?;

    tx.commit()
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "verified",
        "message": "Aadhaar verified successfully",
        "aadhaar_number_masked": aadhaar_masked
    })))
}

#[utoipa::path(
    get,
    path = "/v1/onboarding/status",
    responses(
        (status = 200, description = "Returns onboarding status", body = OnboardingStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Merchant not found")
    ),
    security(
        ("api_key" = [])
    )
)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct OnboardingStatusResponse {
    pub onboarding_status: String,
    pub tier: String,
    pub daily_limit: i64,
    pub monthly_limit: i64,
    pub verifications: Vec<serde_json::Value>,
}
// Update registration step
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegistrationStepRequest {
    pub step: i32,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationProgressResponse {
    pub current_step: i32,
    pub registration_data: serde_json::Value,
    pub kyc_status: String,
    pub steps: Vec<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/registration/step",
    request_body = RegistrationStepRequest,
    responses(
        (status = 200, description = "Registration step updated"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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
#[utoipa::path(
    get,
    path = "/v1/onboarding/registration/progress",
    responses(
        (status = 200, description = "Returns registration progress", body = RegistrationProgressResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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
        }
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
}

// Submit business info
#[derive(Debug, Deserialize, ToSchema)]
pub struct BusinessInfoRequest {
    pub business_name: String,
    pub business_type: String,
    pub pan: String,
    pub gstin: Option<String>,
    pub address: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/business-info",
    request_body = BusinessInfoRequest,
    responses(
        (status = 200, description = "Business info saved successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn submit_business_info(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<BusinessInfoRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    // Validate business type
    let valid_types = [
        "pvt_ltd",
        "llp",
        "sole_proprietor",
        "partnership",
        "trust",
        "ngo",
    ];
    if !valid_types.contains(&body.business_type.as_str()) {
        return Err(FeelPayError::BadRequest(
            "Invalid business type".to_string(),
        ));
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
#[derive(Debug, Deserialize, ToSchema)]
pub struct BrandingRequest {
    pub logo_url: Option<String>,
    pub color: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/branding",
    request_body = BrandingRequest,
    responses(
        (status = 200, description = "Branding updated successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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
#[derive(Debug, Serialize, ToSchema)]
pub struct LimitsResponse {
    pub per_transaction_limit: i64,
    pub daily_limit: i64,
    pub monthly_limit: i64,
    pub settlement_cycle: String,
}

#[utoipa::path(
    get,
    path = "/v1/onboarding/limits",
    responses(
        (status = 200, description = "Returns transaction limits", body = LimitsResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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
        None => Err(FeelPayError::NotFound("Merchant not found".into())),
    }
}

// Update transaction limits
#[derive(Debug, Deserialize, ToSchema)]
pub struct LimitsRequest {
    pub per_transaction_limit: Option<i64>,
    pub daily_limit: Option<i64>,
    pub monthly_limit: Option<i64>,
    pub settlement_cycle: Option<String>,
}

#[utoipa::path(
    put,
    path = "/v1/onboarding/limits",
    request_body = LimitsRequest,
    responses(
        (status = 200, description = "Limits updated successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
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
            return Err(FeelPayError::BadRequest(
                "Invalid settlement cycle".to_string(),
            ));
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

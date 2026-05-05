use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::time::Duration;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
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
            .route("/aadhaar/verify", web::post().to(verify_aadhaar_otp))
            .route("/digilocker/initiate", web::post().to(initiate_digilocker))
            .route(
                "/digilocker/callback",
                web::post().to(handle_digilocker_callback),
            )
            .route("/digilocker/status", web::get().to(get_digilocker_status))
            .route("/live-preflight", web::get().to(get_live_preflight)),
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct DigiLockerInitiateRequest {
    pub purpose: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DigiLockerCallbackRequest {
    pub code: Option<String>,
    pub state: Option<String>,
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

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct DigiLockerTokenResponse {
    access_token: Option<String>,
    _token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DigiLockerDocsResponse {
    files: Option<Vec<serde_json::Value>>,
    items: Option<Vec<serde_json::Value>>,
}

fn sign_state(secret: &str, merchant_id: Uuid, nonce: &str) -> Result<String, FeelPayError> {
    let payload = format!("{merchant_id}:{nonce}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| FeelPayError::Internal("Invalid state signing secret".into()))?;
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn with_retry_delays() -> [u64; 3] {
    [200, 600, 1400]
}

async fn exchange_digilocker_code_with_retry(
    state: &crate::state::AppState,
    code: &str,
) -> Result<String, FeelPayError> {
    let client_id = state
        .config
        .digilocker_client_id
        .clone()
        .ok_or_else(|| FeelPayError::BadRequest("DigiLocker client ID missing".into()))?;
    let client_secret = state
        .config
        .digilocker_client_secret
        .clone()
        .ok_or_else(|| FeelPayError::BadRequest("DigiLocker client secret missing".into()))?;
    let redirect_uri = state
        .config
        .digilocker_redirect_uri
        .clone()
        .ok_or_else(|| FeelPayError::BadRequest("DigiLocker redirect URI missing".into()))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(state.config.digilocker_timeout_secs))
        .build()
        .map_err(|e| FeelPayError::ConnectorError(e.to_string()))?;

    let mut last_err = "unknown token exchange error".to_string();
    for delay_ms in with_retry_delays() {
        let res = client
            .post(&state.config.digilocker_token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("redirect_uri", &redirect_uri),
            ])
            .send()
            .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                let parsed = resp
                    .json::<DigiLockerTokenResponse>()
                    .await
                    .map_err(|e| FeelPayError::ConnectorError(e.to_string()))?;
                if let Some(token) = parsed.access_token {
                    return Ok(token);
                }
                last_err = "token response missing access_token".to_string();
            }
            Ok(resp) => {
                last_err = format!("token exchange failed with status {}", resp.status());
            }
            Err(e) => {
                last_err = format!("token exchange transport error: {e}");
            }
        }
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    Err(FeelPayError::ConnectorError(last_err))
}

async fn fetch_digilocker_documents_with_retry(
    state: &crate::state::AppState,
    access_token: &str,
) -> Result<Vec<serde_json::Value>, FeelPayError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(state.config.digilocker_timeout_secs))
        .build()
        .map_err(|e| FeelPayError::ConnectorError(e.to_string()))?;

    let mut last_err = "unknown docs fetch error".to_string();
    for delay_ms in with_retry_delays() {
        let res = client
            .get(&state.config.digilocker_docs_url)
            .bearer_auth(access_token)
            .send()
            .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                let parsed = resp
                    .json::<DigiLockerDocsResponse>()
                    .await
                    .map_err(|e| FeelPayError::ConnectorError(e.to_string()))?;
                let files = parsed.files.or(parsed.items).unwrap_or_default();
                return Ok(files);
            }
            Ok(resp) => {
                last_err = format!("docs fetch failed with status {}", resp.status());
            }
            Err(e) => {
                last_err = format!("docs fetch transport error: {e}");
            }
        }
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    Err(FeelPayError::ConnectorError(last_err))
}

fn evaluate_digilocker_rules(docs: &[serde_json::Value]) -> (String, serde_json::Value) {
    let mut has_pan = false;
    let mut has_address = false;
    let mut has_business_proof = false;
    let mut risk_score = 50i32;

    for doc in docs {
        let s = doc.to_string().to_lowercase();
        if s.contains("pan") {
            has_pan = true;
            risk_score -= 20;
        }
        if s.contains("address") || s.contains("aadhaar") {
            has_address = true;
            risk_score -= 15;
        }
        if s.contains("gst") || s.contains("incorporation") || s.contains("registration") {
            has_business_proof = true;
            risk_score -= 15;
        }
    }

    let status = if has_pan && has_address && has_business_proof {
        "verified"
    } else if has_pan && (has_address || has_business_proof) {
        "manual_review"
    } else {
        "rejected"
    };

    (
        status.to_string(),
        serde_json::json!({
            "rules": {
                "has_pan": has_pan,
                "has_address": has_address,
                "has_business_proof": has_business_proof
            },
            "risk_score": risk_score.max(0),
            "doc_count": docs.len()
        }),
    )
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

#[utoipa::path(
    post,
    path = "/v1/onboarding/digilocker/initiate",
    request_body = DigiLockerInitiateRequest,
    responses(
        (status = 200, description = "DigiLocker redirect URL generated"),
        (status = 400, description = "DigiLocker not configured"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn initiate_digilocker(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<DigiLockerInitiateRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    if !state.config.digilocker_enabled {
        return Err(FeelPayError::BadRequest(
            "DigiLocker integration disabled for this environment".into(),
        ));
    }

    let client_id = state
        .config
        .digilocker_client_id
        .clone()
        .ok_or_else(|| FeelPayError::BadRequest("DigiLocker client ID missing".into()))?;
    let redirect_uri = state
        .config
        .digilocker_redirect_uri
        .clone()
        .ok_or_else(|| FeelPayError::BadRequest("DigiLocker redirect URI missing".into()))?;

    let nonce = Uuid::new_v4().to_string();
    let signature = sign_state(&state.config.jwt_secret, merchant_id, &nonce)?;
    let state_token = format!("{nonce}.{signature}");
    let purpose = body.purpose.clone().unwrap_or_else(|| "kyc".to_string());
    let request_meta = serde_json::json!({
        "nonce": nonce,
        "purpose": purpose,
        "provider": "digilocker",
        "redirect_uri": redirect_uri
    });

    sqlx::query(
        r#"
        INSERT INTO kyc_verifications
            (merchant_id, verification_type, document_number, document_data, status)
        VALUES
            ($1, 'digilocker', $2, $3, 'pending')
        "#,
    )
    .bind(merchant_id)
    .bind(format!("dlr_{}", &state_token[..8]))
    .bind(&request_meta)
    .execute(&state.db)
    .await?;

    // Stateful CSRF protection: store hashed OAuth state token with expiry.
    let mut hasher = Sha256::new();
    hasher.update(state_token.as_bytes());
    let state_hash = format!("{:x}", hasher.finalize());
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);
    sqlx::query(
        r#"
        INSERT INTO otp_verifications (phone, otp_hash, otp_type, expires_at, metadata)
        VALUES ($1, $2, 'digilocker_state', $3, $4)
        "#,
    )
    .bind(merchant_id.to_string())
    .bind(state_hash)
    .bind(expires_at)
    .bind(serde_json::json!({
        "merchant_id": merchant_id,
        "provider": "digilocker",
        "purpose": purpose
    }))
    .execute(&state.db)
    .await?;

    let redirect_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state={}",
        state.config.digilocker_auth_base_url,
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state_token)
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "provider": "digilocker",
        "status": "pending_redirect",
        "redirect_url": redirect_url,
        "state": state_token
    })))
}

#[utoipa::path(
    post,
    path = "/v1/onboarding/digilocker/callback",
    request_body = DigiLockerCallbackRequest,
    responses(
        (status = 200, description = "DigiLocker callback processed"),
        (status = 400, description = "Invalid callback payload"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn handle_digilocker_callback(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<DigiLockerCallbackRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    let code = body.code.clone().unwrap_or_default().trim().to_string();
    let state_val = body.state.clone().unwrap_or_default().trim().to_string();
    if code.is_empty() || state_val.is_empty() {
        return Err(FeelPayError::BadRequest(
            "Missing DigiLocker callback code/state".into(),
        ));
    }

    // Verify signed state token
    let mut parts = state_val.splitn(2, '.');
    let nonce = parts.next().unwrap_or_default();
    let sig = parts.next().unwrap_or_default();
    if nonce.is_empty() || sig.is_empty() {
        return Err(FeelPayError::BadRequest(
            "Invalid DigiLocker state token format".into(),
        ));
    }
    let expected_sig = sign_state(&state.config.jwt_secret, merchant_id, nonce)?;
    if expected_sig != sig {
        return Err(FeelPayError::BadRequest(
            "Invalid DigiLocker state signature".into(),
        ));
    }

    // Check state nonce exists and unused
    let mut hasher = Sha256::new();
    hasher.update(state_val.as_bytes());
    let state_hash = format!("{:x}", hasher.finalize());
    let state_row = sqlx::query(
        r#"
        SELECT id
        FROM otp_verifications
        WHERE phone = $1
          AND otp_hash = $2
          AND otp_type = 'digilocker_state'
          AND status = 'pending'
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(merchant_id.to_string())
    .bind(state_hash)
    .fetch_optional(&state.db)
    .await?;
    let Some(state_row) = state_row else {
        return Err(FeelPayError::BadRequest(
            "DigiLocker state expired or already used".into(),
        ));
    };
    let state_id: Uuid = state_row.get(0);

    // Mark state as consumed (single use).
    sqlx::query(
        "UPDATE otp_verifications SET status = 'verified', verified_at = NOW() WHERE id = $1",
    )
    .bind(state_id)
    .execute(&state.db)
    .await?;

    // Exchange code -> token; fetch docs; evaluate rules.
    let access_token = exchange_digilocker_code_with_retry(&state, &code).await?;
    let docs = fetch_digilocker_documents_with_retry(&state, &access_token).await?;
    let (decision, rule_summary) = evaluate_digilocker_rules(&docs);
    let now = chrono::Utc::now();
    let response_meta = serde_json::json!({
        "state_nonce": nonce,
        "verified_via": "digilocker_oauth",
        "rule_engine": rule_summary,
        "docs_preview": docs.iter().take(8).cloned().collect::<Vec<_>>(),
        "verified_at": now
    });

    sqlx::query(
        r#"
        UPDATE kyc_verifications
        SET status = $2,
            verified_at = CASE WHEN $2 = 'verified' THEN NOW() ELSE verified_at END,
            document_data = COALESCE(document_data, '{}'::jsonb) || $3::jsonb
        WHERE merchant_id = $1
          AND verification_type = 'digilocker'
          AND status = 'pending'
        "#,
    )
    .bind(merchant_id)
    .bind(&decision)
    .bind(&response_meta)
    .execute(&state.db)
    .await?;

    // Store explicit audit trail as additional verification record version for forensics.
    sqlx::query(
        r#"
        INSERT INTO kyc_verifications
            (merchant_id, verification_type, document_number, document_data, status, verified_at)
        VALUES
            ($1, 'digilocker_audit', $2, $3, $4, CASE WHEN $4 = 'verified' THEN NOW() ELSE NULL END)
        "#,
    )
    .bind(merchant_id)
    .bind(format!("dlr_audit_{}", &nonce[..8]))
    .bind(&response_meta)
    .bind(&decision)
    .execute(&state.db)
    .await?;

    sqlx::query(
        r#"
        UPDATE merchants
        SET onboarding_status = CASE
                WHEN $2 = 'verified' THEN 'in_progress'
                WHEN $2 = 'manual_review' THEN 'in_progress'
                ELSE onboarding_status
            END,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .bind(&decision)
    .execute(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": decision,
        "provider": "digilocker",
        "message": "DigiLocker callback processed",
        "rule_summary": rule_summary
    })))
}

#[utoipa::path(
    get,
    path = "/v1/onboarding/digilocker/status",
    responses(
        (status = 200, description = "DigiLocker verification status"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_digilocker_status(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;

    if !state.config.digilocker_enabled {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "provider": "digilocker",
            "status": "disabled"
        })));
    }

    let row = sqlx::query(
        r#"
        SELECT status, verified_at, document_data
        FROM kyc_verifications
        WHERE merchant_id = $1 AND verification_type = 'digilocker'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "provider": "digilocker",
            "status": row.get::<String, _>(0),
            "verified_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>(1),
            "metadata": row.get::<Option<serde_json::Value>, _>(2),
        }))),
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "provider": "digilocker",
            "status": "not_started"
        }))),
    }
}

#[utoipa::path(
    get,
    path = "/v1/onboarding/live-preflight",
    responses(
        (status = 200, description = "Live mode preflight checklist")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_live_preflight(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id_from_req(&req)?;
    let row = sqlx::query(
        r#"
        SELECT onboarding_status, status, webhook_url
        FROM merchants
        WHERE id = $1
        "#,
    )
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await?;

    let onboarding_status: String = row.get("onboarding_status");
    let merchant_status: String = row.get("status");
    let webhook_url: Option<String> = row.get("webhook_url");

    let bank_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM bank_accounts
        WHERE merchant_id = $1 AND verification_status = 'verified'
        "#,
    )
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let team_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM users
        WHERE merchant_id = $1 AND status IN ('active', 'pending')
        "#,
    )
    .bind(merchant_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let mut blockers = Vec::<String>::new();
    if onboarding_status != "approved" {
        blockers.push(format!(
            "Onboarding status is `{onboarding_status}`; require `approved`."
        ));
    }
    if merchant_status != "active" {
        blockers.push(format!(
            "Merchant status is `{merchant_status}`; require `active`."
        ));
    }
    if webhook_url.as_deref().unwrap_or("").trim().is_empty() {
        blockers.push("Webhook URL missing.".to_string());
    }
    if bank_count == 0 {
        blockers.push("No verified bank account found.".to_string());
    }

    let checks = serde_json::json!({
        "onboarding_approved": onboarding_status == "approved",
        "merchant_active": merchant_status == "active",
        "webhook_configured": webhook_url.as_deref().unwrap_or("").trim().len() > 0,
        "verified_bank_account": bank_count > 0,
        "team_present": team_count > 0
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "can_go_live": blockers.is_empty(),
        "blockers": blockers,
        "checks": checks
    })))
}

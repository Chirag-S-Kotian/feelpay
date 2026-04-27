use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::fee_plan::{
    AssignFeePlanRequest, CreateFeePlanRequest, FeePlan, FeePlanResponse,
    MerchantFeeAssignment, SettlementBatch, UpdateFeePlanRequest, VerificationLog,
};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/fee-plans")
            .route("", web::post().to(create_fee_plan))
            .route("", web::get().to(list_fee_plans))
            .route("/{id}", web::get().to(get_fee_plan))
            .route("/{id}", web::put().to(update_fee_plan))
            .route("/{id}", web::delete().to(delete_fee_plan))
            // Assignments
            .route("/assign", web::post().to(assign_fee_plan))
            .route("/merchant/{merchant_id}", web::get().to(get_merchant_fee_plan))
            // Auto-verification
            .route("/verify/{merchant_id}/pan", web::post().to(verify_pan))
            .route("/verify/{merchant_id}/gstin", web::post().to(verify_gstin))
            .route("/verify/{merchant_id}/bank", web::post().to(verify_bank))
            // Settlement batches
            .route("/settlement-batches", web::post().to(create_settlement_batch))
            .route("/settlement-batches", web::get().to(list_settlement_batches))
            .route("/settlement-batches/{id}/process", web::post().to(process_settlement_batch)),
    );
}

fn user_id_from_req(req: &HttpRequest) -> Result<Uuid, FeelPayError> {
    req.extensions()
        .get::<crate::middleware::auth::UserId>()
        .map(|u| u.0)
        .ok_or_else(|| FeelPayError::Unauthorized("Missing user context".into()))
}

/// Create a new fee plan
pub async fn create_fee_plan(
    state: web::Data<AppState>,
    body: web::Json<CreateFeePlanRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let plan: FeePlan = sqlx::query_as(
        r#"
        INSERT INTO fee_plans (
            name, description, tenant_id,
            card_debit_bps, card_credit_bps, upi_bps, netbanking_bps, wallet_bps, emi_bps,
            card_flat_fee, upi_flat_fee, netbanking_flat_fee, wallet_flat_fee,
            gst_percentage, gst_inclusive,
            min_transaction_amount, max_transaction_amount,
            is_default
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING *
        "#,
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.tenant_id)
    .bind(body.card_debit_bps.unwrap_or(200))
    .bind(body.card_credit_bps.unwrap_or(250))
    .bind(body.upi_bps.unwrap_or(0))
    .bind(body.netbanking_bps.unwrap_or(200))
    .bind(body.wallet_bps.unwrap_or(200))
    .bind(body.emi_bps.unwrap_or(300))
    .bind(body.card_flat_fee.unwrap_or(0))
    .bind(body.upi_flat_fee.unwrap_or(0))
    .bind(body.netbanking_flat_fee.unwrap_or(200))
    .bind(body.wallet_flat_fee.unwrap_or(0))
    .bind(body.gst_percentage.unwrap_or(18.0))
    .bind(body.gst_inclusive.unwrap_or(false))
    .bind(body.min_transaction_amount.unwrap_or(100))
    .bind(body.max_transaction_amount.unwrap_or(5000000))
    .bind(body.is_default.unwrap_or(false))
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(FeePlanResponse::from(plan)))
}

/// List all fee plans
pub async fn list_fee_plans(
    state: web::Data<AppState>,
    query: web::Query<ListFeePlansQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let plans: Vec<FeePlan> = sqlx::query_as(
        "SELECT * FROM fee_plans WHERE deleted_at IS NULL AND ($1::uuid IS NULL OR tenant_id = $1) ORDER BY created_at DESC"
    )
    .bind(query.tenant_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let responses: Vec<FeePlanResponse> = plans.into_iter().map(FeePlanResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

#[derive(Debug, Deserialize)]
pub struct ListFeePlansQuery {
    pub tenant_id: Option<Uuid>,
}

/// Get a specific fee plan
pub async fn get_fee_plan(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let plan_id = path.into_inner();

    let plan: Option<FeePlan> = sqlx::query_as(
        "SELECT * FROM fee_plans WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match plan {
        Some(p) => Ok(HttpResponse::Ok().json(FeePlanResponse::from(p))),
        None => Err(FeelPayError::NotFound("Fee plan not found".into())),
    }
}

/// Update a fee plan
pub async fn update_fee_plan(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateFeePlanRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let plan_id = path.into_inner();

    let plan: Option<FeePlan> = sqlx::query_as(
        r#"
        UPDATE fee_plans SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            card_debit_bps = COALESCE($4, card_debit_bps),
            card_credit_bps = COALESCE($5, card_credit_bps),
            upi_bps = COALESCE($6, upi_bps),
            netbanking_bps = COALESCE($7, netbanking_bps),
            wallet_bps = COALESCE($8, wallet_bps),
            emi_bps = COALESCE($9, emi_bps),
            is_active = COALESCE($10, is_active),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING *
        "#,
    )
    .bind(plan_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.card_debit_bps)
    .bind(body.card_credit_bps)
    .bind(body.upi_bps)
    .bind(body.netbanking_bps)
    .bind(body.wallet_bps)
    .bind(body.emi_bps)
    .bind(body.is_active)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match plan {
        Some(p) => Ok(HttpResponse::Ok().json(FeePlanResponse::from(p))),
        None => Err(FeelPayError::NotFound("Fee plan not found".into())),
    }
}

/// Delete a fee plan (soft delete)
pub async fn delete_fee_plan(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let plan_id = path.into_inner();

    let result = sqlx::query(
        "UPDATE fee_plans SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(plan_id)
    .execute(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Fee plan deleted" })))
    } else {
        Err(FeelPayError::NotFound("Fee plan not found".into()))
    }
}

/// Assign a fee plan to a merchant
pub async fn assign_fee_plan(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AssignFeePlanRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let user_id = user_id_from_req(&req)?;

    let assignment: MerchantFeeAssignment = sqlx::query_as(
        r#"
        INSERT INTO merchant_fee_assignments (
            merchant_id, fee_plan_id,
            custom_card_debit_bps, custom_card_credit_bps, custom_upi_bps,
            custom_netbanking_bps, custom_wallet_bps,
            effective_from, assigned_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, NOW()), $9)
        RETURNING *
        "#,
    )
    .bind(body.merchant_id)
    .bind(body.fee_plan_id)
    .bind(body.custom_card_debit_bps)
    .bind(body.custom_card_credit_bps)
    .bind(body.custom_upi_bps)
    .bind(body.custom_netbanking_bps)
    .bind(body.custom_wallet_bps)
    .bind(body.effective_from)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(assignment))
}

/// Get merchant's current fee plan
pub async fn get_merchant_fee_plan(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = path.into_inner();

    let row = sqlx::query(
        r#"
        SELECT fp.*, mfa.custom_card_debit_bps, mfa.custom_card_credit_bps, mfa.custom_upi_bps
        FROM merchant_fee_assignments mfa
        JOIN fee_plans fp ON mfa.fee_plan_id = fp.id
        WHERE mfa.merchant_id = $1
          AND mfa.effective_from <= NOW()
          AND (mfa.effective_until IS NULL OR mfa.effective_until > NOW())
        ORDER BY mfa.effective_from DESC
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    match row {
        Some(row) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "plan_id": row.get::<Uuid, _>(0),
            "name": row.get::<String, _>(2),
            "card_debit_bps": row.get::<Option<i32>, _>(20).unwrap_or_else(|| row.get(3)),
            "card_credit_bps": row.get::<Option<i32>, _>(21).unwrap_or_else(|| row.get(4)),
            "upi_bps": row.get::<Option<i32>, _>(22).unwrap_or_else(|| row.get(5)),
            "gst_percentage": row.get::<f64, _>(14),
        }))),
        None => {
            // Return default plan
            let default: Option<FeePlan> = sqlx::query_as(
                "SELECT * FROM fee_plans WHERE is_default = true AND deleted_at IS NULL LIMIT 1"
            )
            .fetch_optional(&state.db)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

            match default {
                Some(p) => Ok(HttpResponse::Ok().json(FeePlanResponse::from(p))),
                None => Err(FeelPayError::NotFound("No fee plan assigned".into())),
            }
        }
    }
}

// ============ Auto-Verification ============

/// Verify PAN with external provider
pub async fn verify_pan(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<VerifyDocumentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = path.into_inner();

    // In production, call actual PAN verification API (NSDL, UTIITSL, etc.)
    // For now, simulate verification
    
    let pan = body.document_number.trim().to_uppercase();
    
    // Validate PAN format
    if !regex::Regex::new(r"^[A-Z]{5}[0-9]{4}[A-Z]$")
        .map(|re| re.is_match(&pan))
        .unwrap_or(false)
    {
        return Err(FeelPayError::BadRequest("Invalid PAN format".into()));
    }

    // Simulate API call
    let verification_score = 85; // Simulated confidence score
    let status = if verification_score >= 80 { "success" } else { "manual_review" };

    // Log verification
    let log: VerificationLog = sqlx::query_as(
        r#"
        INSERT INTO verification_logs (merchant_id, verification_type, provider, request_payload, response_payload, status, verification_score)
        VALUES ($1, 'pan', 'simulator', $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(serde_json::json!({ "pan": pan }))
    .bind(serde_json::json!({ "verified": status == "success", "score": verification_score }))
    .bind(status)
    .bind(verification_score)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    // Update KYC verification if successful
    if status == "success" {
        sqlx::query(
            "UPDATE kyc_verifications SET status = 'verified', verified_at = NOW() WHERE merchant_id = $1 AND verification_type = 'pan'"
        )
        .bind(merchant_id)
        .execute(&state.db)
        .await
        .ok();
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "verification_score": verification_score,
        "log_id": log.id
    })))
}

/// Verify GSTIN with external provider
pub async fn verify_gstin(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<VerifyDocumentRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = path.into_inner();

    let gstin = body.document_number.trim().to_uppercase();
    
    // Validate GSTIN format
    if !regex::Regex::new(r"^[0-9]{2}[A-Z]{5}[0-9]{4}[A-Z]{1}[A-Z0-9]{1}Z[A-Z0-9]{1}$")
        .map(|re| re.is_match(&gstin))
        .unwrap_or(false)
    {
        return Err(FeelPayError::BadRequest("Invalid GSTIN format".into()));
    }

    let verification_score = 90;
    let status = "success";

    let log: VerificationLog = sqlx::query_as(
        r#"
        INSERT INTO verification_logs (merchant_id, verification_type, provider, request_payload, response_payload, status, verification_score)
        VALUES ($1, 'gstin', 'simulator', $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(serde_json::json!({ "gstin": gstin }))
    .bind(serde_json::json!({ "verified": true, "score": verification_score }))
    .bind(status)
    .bind(verification_score)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if status == "success" {
        sqlx::query(
            "UPDATE kyc_verifications SET status = 'verified', verified_at = NOW() WHERE merchant_id = $1 AND verification_type = 'gstin'"
        )
        .bind(merchant_id)
        .execute(&state.db)
        .await
        .ok();
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "verification_score": verification_score,
        "log_id": log.id
    })))
}

/// Verify bank account with penny drop
pub async fn verify_bank(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<VerifyBankRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = path.into_inner();

    // Simulate penny drop verification
    let verification_score = 95;
    let status = "success";

    let log: VerificationLog = sqlx::query_as(
        r#"
        INSERT INTO verification_logs (merchant_id, verification_type, provider, request_payload, response_payload, status, verification_score)
        VALUES ($1, 'bank_penny_drop', 'simulator', $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(merchant_id)
    .bind(serde_json::json!({ 
        "account_number": body.account_number,
        "ifsc": body.ifsc_code 
    }))
    .bind(serde_json::json!({ 
        "verified": true, 
        "beneficiary_name": body.account_holder_name,
        "penny_amount": 1.00 
    }))
    .bind(status)
    .bind(verification_score)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "verification_score": verification_score,
        "beneficiary_name": body.account_holder_name,
        "log_id": log.id
    })))
}

#[derive(Debug, Deserialize)]
pub struct VerifyDocumentRequest {
    pub document_number: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyBankRequest {
    pub account_number: String,
    pub ifsc_code: String,
    pub account_holder_name: String,
}

// ============ Settlement Batches ============

/// Create a settlement batch
pub async fn create_settlement_batch(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateSettlementBatchRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let user_id = user_id_from_req(&req)?;
    let batch_date = body.batch_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let batch_type = body.batch_type.clone().unwrap_or_else(|| "daily".to_string());

    let batch: SettlementBatch = sqlx::query_as(
        r#"
        INSERT INTO settlement_batches (batch_date, batch_type, created_by)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(batch_date)
    .bind(&batch_type)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(batch))
}

/// List settlement batches
pub async fn list_settlement_batches(
    state: web::Data<AppState>,
    query: web::Query<ListBatchesQuery>,
) -> Result<HttpResponse, FeelPayError> {
    let batches: Vec<SettlementBatch> = sqlx::query_as(
        "SELECT * FROM settlement_batches WHERE ($1::date IS NULL OR batch_date = $1) ORDER BY batch_date DESC LIMIT 50"
    )
    .bind(query.date)
    .fetch_all(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(batches))
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub date: Option<chrono::NaiveDate>,
}

/// Process a settlement batch
pub async fn process_settlement_batch(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, FeelPayError> {
    let batch_id = path.into_inner();

    // Update batch status
    let result = sqlx::query(
        "UPDATE settlement_batches SET status = 'processing', processed_at = NOW() WHERE id = $1 AND status = 'pending'"
    )
    .bind(batch_id)
    .execute(&state.db)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(FeelPayError::BadRequest("Batch not found or already processed".into()));
    }

    // In production, this would:
    // 1. Calculate settlements for all eligible merchants
    // 2. Create settlement records
    // 3. Initiate bank transfers
    // 4. Update batch totals

    // Simulate processing
    sqlx::query(
        "UPDATE settlement_batches SET status = 'completed', completed_at = NOW(), total_merchants = 5, total_amount = 10000000 WHERE id = $1"
    )
    .bind(batch_id)
    .execute(&state.db)
    .await
    .ok();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Settlement batch processed",
        "batch_id": batch_id
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateSettlementBatchRequest {
    pub batch_date: Option<chrono::NaiveDate>,
    pub batch_type: Option<String>,
}

use actix_cors::Cors;
use actix_web::{http::header, middleware::{Compress, Logger}, web, App, HttpResponse, HttpServer, Responder};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

pub mod cache;
use cache::AdminCache;
pub mod docs;

fn validate_admin_startup_env() -> Result<(), String> {
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let payment_mode = env::var("PAYMENT_MODE").unwrap_or_else(|_| "test".to_string());
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set".to_string())?;
    let vault_key = env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY must be set".to_string())?;
    let allow_secret_reveal =
        env::var("ALLOW_SECRET_REVEAL").unwrap_or_else(|_| "false".to_string());

    let valid_vault_key =
        vault_key.len() == 64 && vault_key.chars().all(|value| value.is_ascii_hexdigit());
    if !valid_vault_key {
        return Err("VAULT_ENCRYPTION_KEY must be 64 hex chars".to_string());
    }

    if app_env == "production" {
        if payment_mode != "live" {
            return Err("APP_ENV=production requires PAYMENT_MODE=live".to_string());
        }
        if jwt_secret.contains("change_in_production") || jwt_secret.len() < 32 {
            return Err(
                "Production requires a non-default JWT_SECRET of at least 32 chars".to_string(),
            );
        }
        if vault_key == "0".repeat(64) {
            return Err("Production requires a non-default VAULT_ENCRYPTION_KEY".to_string());
        }
        if allow_secret_reveal.eq_ignore_ascii_case("true") {
            return Err("ALLOW_SECRET_REVEAL must stay false in production".to_string());
        }
    } else if payment_mode == "live" {
        return Err("PAYMENT_MODE=live requires APP_ENV=production".to_string());
    }

    Ok(())
}

async fn admin_health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "FeelPay Admin Server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn admin_readiness(pool: web::Data<PgPool>) -> impl Responder {
    let database_ready = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool.get_ref())
        .await
        .is_ok();

    HttpResponse::build(if database_ready {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    })
    .json(serde_json::json!({
        "status": if database_ready { "ready" } else { "not_ready" },
        "database": if database_ready { "ready" } else { "unavailable" },
    }))
}

#[derive(Serialize)]
pub struct PlatformStats {
    pub total_volume: i64,
    pub active_merchants: i64,
    pub failed_payments: i64,
    pub system_health: String,
}

#[derive(Serialize)]
struct OperationsSummary {
    today_volume: i64,
    failed_last_24h: i64,
    disputes_needing_response: i64,
    pending_settlements: i64,
    queued_exports: i64,
    capture_rate_bps: i64,
}

#[derive(Serialize, Deserialize, Clone)]
struct MerchantLite {
    id: Uuid,
    name: String,
    email: Option<String>,
    business_name: Option<String>,
    business_type: Option<String>,
    onboarding_status: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct MerchantListResponse {
    merchants: Vec<MerchantLite>,
}

#[derive(Deserialize)]
struct ProviderCredentialRequest {
    provider: String,
    key_name: String,
    value: String,
}

#[derive(Serialize)]
struct ProviderCredentialItem {
    id: Uuid,
    provider: String,
    key_name: String,
    masked_value: String,
    last_validated: Option<DateTime<Utc>>,
    is_valid: Option<bool>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct BankAccountItem {
    id: Uuid,
    merchant_id: Uuid,
    merchant_name: String,
    merchant_email: String,
    account_holder_name: String,
    ifsc_code: String,
    account_type: String,
    bank_name: Option<String>,
    verification_status: String,
    is_primary: Option<bool>,
    created_at: DateTime<Utc>,
    verified_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct ExportListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    status: Option<String>,
    q: Option<String>,
}

#[derive(Deserialize)]
struct CreateBulkExportRequest {
    date_range: Option<serde_json::Value>,
    entity_types: Vec<String>,
    merchant_filter: Option<String>,
    format: Option<String>,
}

#[derive(Serialize)]
struct ExportJobListItem {
    job_id: String,
    status: String,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    record_count: Option<i32>,
    file_size_bytes: Option<i64>,
    error_message: Option<String>,
}

#[derive(Serialize)]
struct ExportJobListResponse {
    exports: Vec<ExportJobListItem>,
}

#[derive(Serialize)]
struct CreateBulkExportResponse {
    job_id: String,
}

#[derive(Serialize)]
struct DownloadExportResponse {
    download_url: String,
}

#[derive(Deserialize)]
struct AdminLoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct TransactionItem {
    id: Uuid,
    merchant_id: Uuid,
    amount: i64,
    currency: Option<String>,
    status: String,
    payment_method: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct SettlementItem {
    id: Uuid,
    merchant_id: Uuid,
    amount: i64,
    status: String,
    created_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct DisputeItem {
    id: Uuid,
    payment_order_id: Uuid,
    merchant_id: Uuid,
    amount: i64,
    reason: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct RevenueSummary {
    captured_amount: i64,
    refunded_amount: i64,
    dispute_amount: i64,
    settlement_amount: i64,
}

async fn get_platform_stats(pool: web::Data<PgPool>) -> impl Responder {
    const CACHE_KEY: &str = "admin:stats";
    if let Some(cached) = AdminCache::get::<serde_json::Value>(CACHE_KEY) {
        return HttpResponse::Ok().json(cached);
    }

    let stats = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(amount), 0)::BIGINT as total_volume,
            (SELECT COUNT(*) FROM merchants)::BIGINT as active_merchants,
            (SELECT COUNT(*) FROM payment_orders WHERE status = 'failed')::BIGINT as failed_payments
        FROM payment_orders
        WHERE status = 'captured'
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    match stats {
        Ok(row) => {
            use sqlx::Row;
            let resp = serde_json::json!({
                "total_volume": row.get::<i64, _>("total_volume"),
                "active_merchants": row.get::<i64, _>("active_merchants"),
                "failed_payments": row.get::<i64, _>("failed_payments"),
                "system_health": "Healthy"
            });
            AdminCache::set(CACHE_KEY, resp.clone(), Duration::from_secs(10));
            HttpResponse::Ok().json(resp)
        }
        Err(e) => {
            tracing::error!("Stats error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn list_merchants(pool: web::Data<PgPool>) -> impl Responder {
    const CACHE_KEY: &str = "admin:merchants";
    if let Some(cached) = AdminCache::get::<MerchantListResponse>(CACHE_KEY) {
        return HttpResponse::Ok().json(cached);
    }

    let merchants = sqlx::query(
        r#"
        SELECT id, name, email, business_name, business_type, onboarding_status, created_at
        FROM merchants
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match merchants {
        Ok(rows) => {
            use sqlx::Row;
            let merchants = rows
                .into_iter()
                .map(|row| MerchantLite {
                    id: row.get::<Uuid, _>("id"),
                    name: row.get::<String, _>("name"),
                    email: row.get::<Option<String>, _>("email"),
                    business_name: row.get::<Option<String>, _>("business_name"),
                    business_type: row.get::<Option<String>, _>("business_type"),
                    onboarding_status: row.get::<Option<String>, _>("onboarding_status"),
                    created_at: row.get::<Option<DateTime<Utc>>, _>("created_at"),
                })
                .collect::<Vec<_>>();
            let resp = MerchantListResponse { merchants };
            AdminCache::set(CACHE_KEY, resp.clone(), Duration::from_secs(15));
            HttpResponse::Ok().json(resp)
        }
        Err(e) => {
            tracing::error!("List merchants error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn mask_secret(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return "****".to_string();
    }
    let suffix = chars[chars.len() - 4..].iter().collect::<String>();
    format!("****{suffix}")
}

fn encrypt_secret(value: &str) -> Result<(String, String), String> {
    let raw_key = env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY is required to save provider credentials".to_string())?;
    let key = hex::decode(raw_key.trim())
        .map_err(|_| "VAULT_ENCRYPTION_KEY must be 64 hex characters".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string())?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), value.as_bytes())
        .map_err(|_| "Credential encryption failed".to_string())?;
    Ok((BASE64.encode(encrypted), BASE64.encode(nonce_bytes)))
}

async fn list_provider_credentials(pool: web::Data<PgPool>) -> impl Responder {
    let rows = sqlx::query(
        r#"
        SELECT id, provider, key_name, last_validated, is_valid, created_at, updated_at
        FROM provider_credentials
        ORDER BY provider, key_name
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let data = rows
                .into_iter()
                .map(|row| ProviderCredentialItem {
                    id: row.get("id"),
                    provider: row.get("provider"),
                    key_name: row.get("key_name"),
                    masked_value: "stored".to_string(),
                    last_validated: row.get("last_validated"),
                    is_valid: row.get("is_valid"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(serde_json::json!({ "items": data }))
        }
        Err(e) => {
            tracing::error!("List provider credentials failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load provider credentials" }))
        }
    }
}

async fn upsert_provider_credential(
    pool: web::Data<PgPool>,
    req: web::Json<ProviderCredentialRequest>,
) -> impl Responder {
    let provider = req.provider.trim().to_lowercase();
    let key_name = req.key_name.trim().to_lowercase();
    if provider.is_empty() || key_name.is_empty() || req.value.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "provider, key_name, and value are required"
        }));
    }

    let (encrypted_value, nonce) = match encrypt_secret(req.value.trim()) {
        Ok(value) => value,
        Err(message) => {
            return HttpResponse::BadRequest().json(serde_json::json!({ "message": message }))
        }
    };

    let saved = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO provider_credentials (provider, key_name, encrypted_value, nonce, is_valid)
        VALUES ($1, $2, $3, $4, TRUE)
        ON CONFLICT (provider, key_name)
        DO UPDATE SET encrypted_value = EXCLUDED.encrypted_value,
                      nonce = EXCLUDED.nonce,
                      updated_at = NOW(),
                      is_valid = TRUE
        RETURNING id
        "#,
    )
    .bind(&provider)
    .bind(&key_name)
    .bind(encrypted_value)
    .bind(nonce)
    .fetch_one(pool.get_ref())
    .await;

    match saved {
        Ok(id) => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "provider": provider,
            "key_name": key_name,
            "masked_value": mask_secret(req.value.trim())
        })),
        Err(e) => {
            tracing::error!("Save provider credential failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to save provider credential" }))
        }
    }
}

async fn validate_provider_credential(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let updated = sqlx::query(
        "UPDATE provider_credentials SET last_validated = NOW(), is_valid = TRUE, updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "is_valid": true
        })),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Provider credential not found"
        })),
        Err(e) => {
            tracing::error!("Validate provider credential failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to validate provider credential" }))
        }
    }
}

async fn list_bank_accounts(
    pool: web::Data<PgPool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let status = query.status.clone().filter(|v| !v.trim().is_empty());
    let q = query.q.clone().filter(|v| !v.trim().is_empty());

    let rows = sqlx::query(
        r#"
        SELECT
          b.id,
          b.merchant_id,
          m.name AS merchant_name,
          m.email AS merchant_email,
          b.account_holder_name,
          b.ifsc_code,
          b.account_type,
          b.bank_name,
          b.verification_status,
          b.is_primary,
          b.created_at,
          b.verified_at
        FROM bank_accounts b
        JOIN merchants m ON m.id = b.merchant_id
        WHERE
          ($1::text IS NULL OR b.verification_status = $1)
          AND (
            $2::text IS NULL OR
            b.ifsc_code ILIKE '%' || $2 || '%' OR
            b.account_holder_name ILIKE '%' || $2 || '%' OR
            m.name ILIKE '%' || $2 || '%' OR
            m.email ILIKE '%' || $2 || '%'
          )
        ORDER BY b.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(status)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let data = rows
                .into_iter()
                .map(|row| BankAccountItem {
                    id: row.get("id"),
                    merchant_id: row.get("merchant_id"),
                    merchant_name: row.get("merchant_name"),
                    merchant_email: row.get("merchant_email"),
                    account_holder_name: row.get("account_holder_name"),
                    ifsc_code: row.get("ifsc_code"),
                    account_type: row.get("account_type"),
                    bank_name: row.get("bank_name"),
                    verification_status: row.get("verification_status"),
                    is_primary: row.get("is_primary"),
                    created_at: row.get("created_at"),
                    verified_at: row.get("verified_at"),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(serde_json::json!({ "items": data }))
        }
        Err(e) => {
            tracing::error!("List bank accounts failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load bank accounts" }))
        }
    }
}

async fn list_exports(
    pool: web::Data<PgPool>,
    query: web::Query<ExportListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            status,
            created_at,
            completed_at,
            record_count,
            file_size_bytes,
            error_message
        FROM export_jobs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let exports = rows
                .into_iter()
                .map(|row| ExportJobListItem {
                    job_id: row.get::<Uuid, _>("id").to_string(),
                    status: row.get::<String, _>("status"),
                    created_at: row.get::<DateTime<Utc>, _>("created_at"),
                    completed_at: row.get::<Option<DateTime<Utc>>, _>("completed_at"),
                    record_count: row.get::<Option<i32>, _>("record_count"),
                    file_size_bytes: row.get::<Option<i64>, _>("file_size_bytes"),
                    error_message: row.get::<Option<String>, _>("error_message"),
                })
                .collect();

            HttpResponse::Ok().json(ExportJobListResponse { exports })
        }
        Err(e) => {
            tracing::error!("List exports error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Failed to fetch export jobs"
            }))
        }
    }
}

async fn list_transactions(
    pool: web::Data<PgPool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let status = query.status.clone().filter(|v| !v.trim().is_empty());
    let q = query.q.clone().filter(|v| !v.trim().is_empty());

    let rows = sqlx::query(
        r#"
        SELECT id, merchant_id, amount, currency, status, payment_method, created_at
        FROM payment_orders
        WHERE
          ($1::text IS NULL OR status = $1)
          AND (
            $2::text IS NULL OR
            id::text ILIKE '%' || $2 || '%' OR
            merchant_id::text ILIKE '%' || $2 || '%'
          )
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(status)
    .bind(q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let data = rows
                .into_iter()
                .map(|r| TransactionItem {
                    id: r.get("id"),
                    merchant_id: r.get("merchant_id"),
                    amount: r.get("amount"),
                    currency: r.get("currency"),
                    status: r.get("status"),
                    payment_method: r.get("payment_method"),
                    created_at: r.get("created_at"),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(serde_json::json!({ "items": data }))
        }
        Err(e) => {
            tracing::error!("List transactions failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load transactions" }))
        }
    }
}

async fn list_settlements(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let status = query.status.clone().filter(|v| !v.trim().is_empty());

    let rows = sqlx::query(
        r#"
        SELECT id, merchant_id, amount, status, created_at, processed_at
        FROM settlements
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let data = rows
                .into_iter()
                .map(|r| SettlementItem {
                    id: r.get("id"),
                    merchant_id: r.get("merchant_id"),
                    amount: r.get("amount"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                    processed_at: r.get("processed_at"),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(serde_json::json!({ "items": data }))
        }
        Err(e) => {
            tracing::error!("List settlements failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load settlements" }))
        }
    }
}

async fn list_disputes(pool: web::Data<PgPool>, query: web::Query<ListQuery>) -> impl Responder {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let status = query.status.clone().filter(|v| !v.trim().is_empty());

    let rows = sqlx::query(
        r#"
        SELECT
          id,
          payment_order_id,
          merchant_id,
          amount,
          COALESCE(reason_message, reason_code) AS reason,
          status,
          created_at
        FROM disputes
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let data = rows
                .into_iter()
                .map(|r| DisputeItem {
                    id: r.get("id"),
                    payment_order_id: r.get("payment_order_id"),
                    merchant_id: r.get("merchant_id"),
                    amount: r.get("amount"),
                    reason: r.get("reason"),
                    status: r.get("status"),
                    created_at: r.get("created_at"),
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(serde_json::json!({ "items": data }))
        }
        Err(e) => {
            tracing::error!("List disputes failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load disputes" }))
        }
    }
}

async fn revenue_summary(pool: web::Data<PgPool>) -> impl Responder {
    const CACHE_KEY: &str = "admin:revenue:summary";
    if let Some(cached) = AdminCache::get::<serde_json::Value>(CACHE_KEY) {
        return HttpResponse::Ok().json(cached);
    }

    let row = sqlx::query(
        r#"
        SELECT
          COALESCE((SELECT SUM(amount) FROM payment_orders WHERE status = 'captured'), 0)::bigint AS captured_amount,
          COALESCE((SELECT SUM(amount) FROM payment_orders WHERE status = 'refunded'), 0)::bigint AS refunded_amount,
          COALESCE((SELECT SUM(amount) FROM disputes), 0)::bigint AS dispute_amount,
          COALESCE((SELECT SUM(amount) FROM settlements WHERE status IN ('paid','processed')), 0)::bigint AS settlement_amount
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    match row {
        Ok(row) => {
            use sqlx::Row;
            let data = RevenueSummary {
                captured_amount: row.get("captured_amount"),
                refunded_amount: row.get("refunded_amount"),
                dispute_amount: row.get("dispute_amount"),
                settlement_amount: row.get("settlement_amount"),
            };
            if let Ok(val) = serde_json::to_value(&data) {
                AdminCache::set(CACHE_KEY, val, Duration::from_secs(10));
            }
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            tracing::error!("Revenue summary failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load revenue summary" }))
        }
    }
}

async fn operations_summary(pool: web::Data<PgPool>) -> impl Responder {
    const CACHE_KEY: &str = "admin:operations:summary";
    if let Some(cached) = AdminCache::get::<serde_json::Value>(CACHE_KEY) {
        return HttpResponse::Ok().json(cached);
    }

    let row = sqlx::query(
        r#"
        SELECT
          COALESCE((SELECT SUM(amount) FROM payment_orders WHERE status = 'captured' AND created_at >= date_trunc('day', NOW())), 0)::bigint AS today_volume,
          COALESCE((SELECT COUNT(*) FROM payment_orders WHERE status = 'failed' AND created_at >= NOW() - INTERVAL '24 hours'), 0)::bigint AS failed_last_24h,
          COALESCE((SELECT COUNT(*) FROM disputes WHERE status = 'needs_response'), 0)::bigint AS disputes_needing_response,
          COALESCE((SELECT COUNT(*) FROM settlements WHERE status IN ('pending', 'processing')), 0)::bigint AS pending_settlements,
          COALESCE((SELECT COUNT(*) FROM export_jobs WHERE status IN ('queued', 'processing')), 0)::bigint AS queued_exports,
          COALESCE((
            SELECT
              CASE WHEN COUNT(*) = 0 THEN 0
              ELSE ROUND(10000.0 * COUNT(*) FILTER (WHERE status = 'captured') / COUNT(*))::bigint
              END
            FROM payment_orders
            WHERE created_at >= NOW() - INTERVAL '24 hours'
          ), 0)::bigint AS capture_rate_bps
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    match row {
        Ok(row) => {
            use sqlx::Row;
            let data = OperationsSummary {
                today_volume: row.get("today_volume"),
                failed_last_24h: row.get("failed_last_24h"),
                disputes_needing_response: row.get("disputes_needing_response"),
                pending_settlements: row.get("pending_settlements"),
                queued_exports: row.get("queued_exports"),
                capture_rate_bps: row.get("capture_rate_bps"),
            };
            if let Ok(val) = serde_json::to_value(&data) {
                AdminCache::set(CACHE_KEY, val, Duration::from_secs(10));
            }
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            tracing::error!("Operations summary failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to load operations summary" }))
        }
    }
}

async fn create_bulk_export(
    pool: web::Data<PgPool>,
    req: web::Json<CreateBulkExportRequest>,
) -> impl Responder {
    if req.entity_types.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "At least one entity type required"
        }));
    }

    let filters = serde_json::json!({
        "date_range": req.date_range,
        "entity_types": req.entity_types,
        "merchant_filter": req.merchant_filter,
        "format": req.format,
    });

    let merchant_id = if let Some(id) = req
        .merchant_filter
        .as_ref()
        .filter(|value| !value.is_empty() && *value != "all")
    {
        match Uuid::parse_str(id) {
            Ok(parsed) => parsed,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Invalid merchant_filter"
                }))
            }
        }
    } else {
        let first_merchant = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM merchants ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(pool.get_ref())
        .await;

        match first_merchant {
            Ok(Some(id)) => id,
            Ok(None) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "No merchants found to attach export job"
                }))
            }
            Err(e) => {
                tracing::error!("Merchant lookup failed for bulk export: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "message": "Failed to create export job"
                }));
            }
        }
    };

    let inserted = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO export_jobs (merchant_id, job_type, status, filters)
        VALUES ($1, 'bulk', 'queued', $2::jsonb)
        RETURNING id
        "#,
    )
    .bind(merchant_id)
    .bind(filters)
    .fetch_one(pool.get_ref())
    .await;

    match inserted {
        Ok(job_id) => HttpResponse::Ok().json(CreateBulkExportResponse {
            job_id: job_id.to_string(),
        }),
        Err(e) => {
            tracing::error!("Create bulk export failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Failed to create export job"
            }))
        }
    }
}

async fn get_export_download(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let job_id = path.into_inner();

    let file_url =
        sqlx::query_scalar::<_, Option<String>>("SELECT file_url FROM export_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(pool.get_ref())
            .await;

    match file_url {
        Ok(Some(Some(url))) => {
            HttpResponse::Ok().json(DownloadExportResponse { download_url: url })
        }
        Ok(Some(None)) => HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Export file not ready"
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Export job not found"
        })),
        Err(e) => {
            tracing::error!("Download export lookup failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Failed to fetch export job"
            }))
        }
    }
}

mod company_payments;
mod integrations;
mod middleware;
use middleware::AdminAuthMiddleware;

fn configure_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/stats", web::get().to(get_platform_stats))
        .route("/merchants", web::get().to(list_merchants))
        .route(
            "/merchants/{id}/impersonate",
            web::post().to(impersonate_merchant),
        )
        .route(
            "/provider-credentials",
            web::get().to(list_provider_credentials),
        )
        .route(
            "/provider-credentials",
            web::post().to(upsert_provider_credential),
        )
        .route(
            "/provider-credentials/{id}/validate",
            web::post().to(validate_provider_credential),
        )
        .route("/bank-accounts", web::get().to(list_bank_accounts))
        .route("/exports", web::get().to(list_exports))
        .route("/exports/bulk", web::post().to(create_bulk_export))
        .route("/exports/{id}/download", web::get().to(get_export_download))
        .route("/transactions", web::get().to(list_transactions))
        .route("/settlements", web::get().to(list_settlements))
        .route("/disputes", web::get().to(list_disputes))
        .route("/revenue/summary", web::get().to(revenue_summary))
        .route("/operations/summary", web::get().to(operations_summary))
        .configure(company_payments::configure)
        .configure(company_payments::configure_company_admin_routes)
        .configure(integrations::configure);
}

async fn admin_login(pool: web::Data<PgPool>, req: web::Json<AdminLoginRequest>) -> impl Responder {
    let row = sqlx::query(
        "SELECT id, name, password_hash, is_admin FROM merchants WHERE email = $1 LIMIT 1",
    )
    .bind(req.email.trim().to_lowercase())
    .fetch_optional(pool.get_ref())
    .await;

    let Some(row) = (match row {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Admin login DB error: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Login failed"}));
        }
    }) else {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"message": "Invalid credentials"}));
    };

    use sqlx::Row;
    let is_admin: bool = row.get("is_admin");
    if !is_admin {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"message": "Admin access required"}));
    }
    let password_hash: String = row.get("password_hash");
    let password_ok = if password_hash.starts_with("$argon2id$") {
        use argon2::PasswordVerifier;
        if let Ok(parsed_hash) = argon2::PasswordHash::new(&password_hash) {
            argon2::Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_ok()
        } else {
            false
        }
    } else {
        bcrypt::verify(req.password.as_str(), &password_hash).unwrap_or(false)
    };
    if !password_ok {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"message": "Invalid credentials"}));
    }
    let merchant_id: Uuid = row.get("id");
    let merchant_name: String = row.get("name");
    match generate_jwt(&merchant_id, &merchant_name) {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
        Err(e) => {
            tracing::error!("Admin login token generation failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"message": "Login failed"}))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    validate_admin_startup_env().map_err(std::io::Error::other)?;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cleaned_url = database_url
        .replace("?channel_binding=require", "")
        .replace("&channel_binding=require", "")
        .replace("?statement_cache_size=0", "")
        .replace("&statement_cache_size=0", "");
    let mut connect_options = PgConnectOptions::from_str(&cleaned_url)
        .expect("Invalid DATABASE_URL");

    if cleaned_url.contains("-pooler.") {
        connect_options = connect_options.statement_cache_capacity(0);
    }

    let pool = PgPoolOptions::new()
        .max_connections(30)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to Postgres");

    let bind_addr = admin_bind_addr();
    std::fs::create_dir_all("uploads").ok();
    println!("Admin server starting on {}", bind_addr);

    HttpServer::new(move || {
        let cors = admin_cors();
        let admin_rate_limit: u32 = env::var("ADMIN_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(feelpay::middleware::rate_limit::RateLimiter::new(admin_rate_limit))
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(admin_health))
            .route("/ready", web::get().to(admin_readiness))
            .route("/docs", web::get().to(docs::admin_docs))
            .route("/docs/", web::get().to(docs::admin_docs))
            .route("/openapi.json", web::get().to(docs::admin_openapi_json))
            .service(
                web::scope("/admin")
                    .route("/login", web::post().to(admin_login))
                    .service(
                        web::scope("")
                            .wrap(AdminAuthMiddleware)
                            .configure(configure_admin_routes),
                    ),
            )
            .service(actix_files::Files::new("/uploads", "uploads").show_files_listing())
            .service(
                web::scope("/api/admin")
                    .route("/login", web::post().to(admin_login))
                    .service(
                        web::scope("")
                            .wrap(AdminAuthMiddleware)
                            .configure(configure_admin_routes),
                    ),
            )
            .service(
                web::scope("/v1/admin")
                    .wrap(AdminAuthMiddleware)
                    .configure(configure_admin_routes),
            )
            .service(
                web::scope("/api/v1/admin")
                    .wrap(AdminAuthMiddleware)
                    .configure(configure_admin_routes),
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}

#[cfg(test)]
mod startup_tests {
    use super::{admin_health, admin_readiness, validate_admin_startup_env};
    use actix_web::{http::StatusCode, test as actix_test, web, App};
    use sqlx::postgres::PgPoolOptions;
    use std::env;
    use std::time::Duration;

    fn set_safe_production_env() {
        env::set_var("APP_ENV", "production");
        env::set_var("PAYMENT_MODE", "live");
        env::set_var("JWT_SECRET", "a".repeat(32));
        env::set_var("VAULT_ENCRYPTION_KEY", "1".repeat(64));
        env::set_var("ALLOW_SECRET_REVEAL", "false");
    }

    #[test]
    fn admin_startup_blocks_unsafe_production_configuration() {
        set_safe_production_env();
        assert!(validate_admin_startup_env().is_ok());

        env::set_var("PAYMENT_MODE", "test");
        assert!(validate_admin_startup_env().is_err());

        set_safe_production_env();
        env::set_var("ALLOW_SECRET_REVEAL", "true");
        assert!(validate_admin_startup_env().is_err());
    }

    #[actix_web::test]
    async fn admin_health_endpoint_is_available() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(admin_health)))
                .await;
        let request = actix_test::TestRequest::get().uri("/health").to_request();
        let response = actix_test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn admin_readiness_reports_unavailable_database() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(50))
            .connect_lazy("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .expect("lazy test pool should build");
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .route("/ready", web::get().to(admin_readiness)),
        )
        .await;
        let request = actix_test::TestRequest::get().uri("/ready").to_request();
        let response = actix_test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

fn admin_bind_addr() -> String {
    let host = env::var("ADMIN_HOST")
        .or_else(|_| env::var("HOST"))
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("ADMIN_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "8081".to_string());
    format!("{host}:{port}")
}

fn admin_cors() -> Cors {
    let origins = env::var("ADMIN_CORS_ALLOWED_ORIGINS")
        .or_else(|_| env::var("CORS_ALLOWED_ORIGINS"))
        .unwrap_or_else(|_| {
            [
                "http://localhost:3000",
                "http://127.0.0.1:3000",
                "http://localhost:3001",
                "http://127.0.0.1:3001",
                "http://127.0.0.1:5174",
                "http://localhost:5174",
                "http://127.0.0.1:5173",
                "http://localhost:5173",
            ]
            .join(",")
        });
    let allowed_origins: Vec<String> = origins
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();

    Cors::default()
        .allowed_origin_fn(move |origin, _| {
            origin
                .to_str()
                .map(|origin| {
                    origin.ends_with(".pages.dev")
                        || origin.ends_with(".onrender.com")
                        || origin.starts_with("http://localhost:")
                        || origin.starts_with("http://127.0.0.1:")
                        || allowed_origins.iter().any(|allowed| allowed == origin || allowed == "*")
                })
                .unwrap_or(false)
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static("x-api-key"),
            header::HeaderName::from_static("x-feelpay-environment"),
            header::HeaderName::from_static("idempotency-key"),
            header::HeaderName::from_static("x-request-id"),
            header::HeaderName::from_static("x-admin-role"),
            header::HeaderName::from_static("x-csrf-token"),
        ])
        .expose_headers(vec![header::HeaderName::from_static("x-request-id")])
        .supports_credentials()
        .max_age(3600)
}

async fn impersonate_merchant(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let merchant_id = path.into_inner();
    let row = sqlx::query("SELECT id, name FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_optional(pool.get_ref())
        .await;

    match row {
        Ok(Some(m)) => {
            use sqlx::Row;
            let m_id: Uuid = m.get("id");
            let m_name: String = m.get("name");
            let token = match generate_jwt(&m_id, &m_name) {
                Ok(t) => t,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            HttpResponse::Ok().json(serde_json::json!({ "token": token }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

fn generate_jwt(merchant_id: &Uuid, _name: &str) -> Result<String, Box<dyn std::error::Error>> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        tenant_id: Option<Uuid>,
        merchant_id: Option<Uuid>,
        role: String,
        exp: usize,
        iat: usize,
    }
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: merchant_id.to_string(),
        tenant_id: None,
        merchant_id: Some(*merchant_id),
        role: "Super Admin".to_string(),
        exp: now + 86400,
        iat: now,
    };
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "JWT_SECRET is not configured")
    })?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.into())
}

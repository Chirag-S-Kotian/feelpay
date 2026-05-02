use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PlatformStats {
    pub total_volume: i64,
    pub active_merchants: i64,
    pub failed_payments: i64,
    pub system_health: String,
}

#[derive(Serialize)]
struct MerchantLite {
    id: Uuid,
    name: String,
    email: Option<String>,
    business_name: Option<String>,
    business_type: Option<String>,
    onboarding_status: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct MerchantListResponse {
    merchants: Vec<MerchantLite>,
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

#[derive(Serialize)]
struct BootstrapTokenResponse {
    token: String,
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
            HttpResponse::Ok().json(serde_json::json!({
                "total_volume": row.get::<i64, _>("total_volume"),
                "active_merchants": row.get::<i64, _>("active_merchants"),
                "failed_payments": row.get::<i64, _>("failed_payments"),
                "system_health": "Healthy"
            }))
        }
        Err(e) => {
            tracing::error!("Stats error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn list_merchants(pool: web::Data<PgPool>) -> impl Responder {
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
            HttpResponse::Ok().json(MerchantListResponse { merchants })
        }
        Err(e) => {
            tracing::error!("List merchants error: {}", e);
            HttpResponse::InternalServerError().finish()
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
            HttpResponse::InternalServerError().json(serde_json::json!({ "message": "Failed to load transactions" }))
        }
    }
}

async fn list_settlements(
    pool: web::Data<PgPool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
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
            HttpResponse::InternalServerError().json(serde_json::json!({ "message": "Failed to load settlements" }))
        }
    }
}

async fn list_disputes(
    pool: web::Data<PgPool>,
    query: web::Query<ListQuery>,
) -> impl Responder {
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
            HttpResponse::InternalServerError().json(serde_json::json!({ "message": "Failed to load disputes" }))
        }
    }
}

async fn revenue_summary(pool: web::Data<PgPool>) -> impl Responder {
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
            HttpResponse::Ok().json(RevenueSummary {
                captured_amount: row.get("captured_amount"),
                refunded_amount: row.get("refunded_amount"),
                dispute_amount: row.get("dispute_amount"),
                settlement_amount: row.get("settlement_amount"),
            })
        }
        Err(e) => {
            tracing::error!("Revenue summary failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "message": "Failed to load revenue summary" }))
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

async fn get_export_download(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let job_id = path.into_inner();

    let file_url = sqlx::query_scalar::<_, Option<String>>(
        "SELECT file_url FROM export_jobs WHERE id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool.get_ref())
    .await;

    match file_url {
        Ok(Some(Some(url))) => HttpResponse::Ok().json(DownloadExportResponse { download_url: url }),
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

mod middleware;
use middleware::AdminAuthMiddleware;

fn configure_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/stats", web::get().to(get_platform_stats))
        .route("/merchants", web::get().to(list_merchants))
        .route("/merchants/{id}/impersonate", web::post().to(impersonate_merchant))
        .route("/exports", web::get().to(list_exports))
        .route("/exports/bulk", web::post().to(create_bulk_export))
        .route("/exports/{id}/download", web::get().to(get_export_download))
        .route("/transactions", web::get().to(list_transactions))
        .route("/settlements", web::get().to(list_settlements))
        .route("/disputes", web::get().to(list_disputes))
        .route("/revenue/summary", web::get().to(revenue_summary));
}

async fn create_admin_bootstrap_token(pool: web::Data<PgPool>) -> impl Responder {
    let admin = sqlx::query("SELECT id, name FROM merchants WHERE is_admin = true ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(pool.get_ref())
        .await;

    match admin {
        Ok(Some(row)) => {
            use sqlx::Row;
            let merchant_id: Uuid = row.get("id");
            let merchant_name: String = row.get("name");
            match generate_jwt(&merchant_id, &merchant_name) {
                Ok(token) => HttpResponse::Ok().json(BootstrapTokenResponse { token }),
                Err(e) => {
                    tracing::error!("Bootstrap token generation failed: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "message": "Failed to generate bootstrap token"
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "message": "No admin merchant found. Mark one merchant as is_admin=true."
        })),
        Err(e) => {
            tracing::error!("Bootstrap token lookup failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Failed to lookup admin merchant"
            }))
        }
    }
}

async fn admin_login(
    pool: web::Data<PgPool>,
    req: web::Json<AdminLoginRequest>,
) -> impl Responder {
    let row = sqlx::query("SELECT id, name, password_hash, is_admin FROM merchants WHERE email = $1 LIMIT 1")
        .bind(req.email.trim().to_lowercase())
        .fetch_optional(pool.get_ref())
        .await;

    let Some(row) = (match row {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Admin login DB error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"message": "Login failed"}));
        }
    }) else {
        return HttpResponse::Unauthorized().json(serde_json::json!({"message": "Invalid credentials"}));
    };

    use sqlx::Row;
    let is_admin: bool = row.get("is_admin");
    if !is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({"message": "Admin access required"}));
    }
    let password_hash: String = row.get("password_hash");
    let password_ok = bcrypt::verify(req.password.as_str(), &password_hash).unwrap_or(false);
    if !password_ok {
        return HttpResponse::Unauthorized().json(serde_json::json!({"message": "Invalid credentials"}));
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

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("Admin server starting on 127.0.0.1:8081");

    HttpServer::new(move || {
        let cors = Cors::permissive().max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/admin")
                    .route("/login", web::post().to(admin_login))
                    .route("/bootstrap-token", web::get().to(create_admin_bootstrap_token))
                    .service(
                        web::scope("")
                            .wrap(AdminAuthMiddleware)
                            .configure(configure_admin_routes),
                    ),
            )
            .service(
                web::scope("/api/admin")
                    .route("/login", web::post().to(admin_login))
                    .route("/bootstrap-token", web::get().to(create_admin_bootstrap_token))
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
    .bind("127.0.0.1:8081")?
    .run()
    .await
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
        role: "merchant".to_string(),
        exp: now + 86400,
        iat: now,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.into())
}

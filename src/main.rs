use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod api;
pub mod config;
pub mod db;
pub mod engine;
pub mod error;
pub mod middleware;
pub mod models;
pub mod state;
pub mod vault;

use config::AppConfig;
use engine::payment::PaymentEngine;
use state::AppState;
use vault::CardVaultService;

fn io_other(message: impl Into<String>) -> std::io::Error {
    std::io::Error::other(message.into())
}

fn validate_startup_config(config: &AppConfig) -> Result<(), String> {
    let key = config.vault_encryption_key.trim();
    let is_hex_64 = key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit());

    if !is_hex_64 {
        return Err("VAULT_ENCRYPTION_KEY must be 64 hex chars (32-byte key)".into());
    }

    if config.is_live() && key == "0000000000000000000000000000000000000000000000000000000000000000" {
        return Err("Live mode requires non-default VAULT_ENCRYPTION_KEY".into());
    }

    Ok(())
}

async fn run_startup_preflight(
    pool: &sqlx::PgPool,
    redis_client: &redis::Client,
    strict_redis: bool,
) -> Result<(), String> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Database preflight failed: {e}"))?;

    match redis_client.get_connection() {
        Ok(mut redis_conn) => {
            let pong: String = redis::cmd("PING")
                .query(&mut redis_conn)
                .map_err(|e| format!("Redis preflight ping failed: {e}"))?;

            if pong != "PONG" {
                return Err(format!("Unexpected Redis ping response: {pong}"));
            }
        }
        Err(e) if strict_redis => {
            return Err(format!("Redis preflight failed to connect: {e}"));
        }
        Err(e) => {
            tracing::warn!(error = %e, "Redis unavailable; continuing startup in test mode");
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting FeelPay Payment Gateway...");

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| io_other(format!("Config load failed: {e}")))?;
    validate_startup_config(&config).map_err(io_other)?;

    // Initialize Database
    let database_url = config.selected_database_url().to_string();
    tracing::info!(
        provider = %config.database_provider,
        "Connecting to PostgreSQL with configured provider"
    );
    let pool = db::pool::create_pool(&database_url)
        .await
        .map_err(|e| io_other(format!("Failed to connect to PostgreSQL: {e}")))?;

    // Initialize Redis
    tracing::info!("Connecting to Redis at {}", config.redis_url);
    let redis_client = redis::Client::open(config.redis_url.clone())
        .map_err(|e| io_other(format!("Failed to connect to Redis: {e}")))?;

    run_startup_preflight(&pool, &redis_client, config.is_live())
        .await
        .map_err(io_other)?;
    tracing::info!("Startup preflight checks passed");

    // Initialize Vault Service
    tracing::info!("Initializing PCI-ready Card Vault");
    let vault = CardVaultService::new(
        pool.clone(),
        &config.vault_encryption_key,
        config.vault_key_id.clone(),
    )
    .map_err(|e| io_other(format!("Failed to initialize vault: {e}")))?;

    // Initialize Payment Engine
    tracing::info!("Configuring Payment Processors");
    let mut payment_engine = PaymentEngine::new();

    // Register native processors
    payment_engine.register_processor(
        "simulator",
        std::sync::Arc::new(engine::processors::simulator::SimulatorProcessor::new()),
    );
    payment_engine.register_processor(
        "card",
        std::sync::Arc::new(engine::processors::card::CardProcessor::new()),
    );
    payment_engine.register_processor(
        "upi",
        std::sync::Arc::new(engine::processors::upi::UpiProcessor::new(
            config.upi_psp_id.clone(),
            config.upi_merchant_vpa.clone(),
        )),
    );
    payment_engine.register_processor(
        "netbanking",
        std::sync::Arc::new(engine::processors::netbanking::NetBankingProcessor::new()),
    );
    payment_engine.register_processor(
        "wallet",
        std::sync::Arc::new(engine::processors::wallet::WalletProcessor::new()),
    );

    // If live, default to real processors. Otherwise default to simulator.
    if config.is_live() {
        tracing::info!("Running in LIVE mode. Transactions are REAL.");
        // Normally we'd use routing rules, here we just set card as default for live
        payment_engine.set_default_processor("card");
    } else {
        tracing::info!("Running in TEST mode.");
        payment_engine.set_default_processor("simulator");
    }

    // Initialize Ledger Service
    let ledger = std::sync::Arc::new(crate::engine::services::ledger::LedgerService::new(
        pool.clone(),
    ));

    // Prepare App State
    let pool_for_handler = pool.clone();
    let state = AppState {
        db: pool,
        redis: redis_client,
        config: config.clone(),
        payment_engine: std::sync::Arc::new(tokio::sync::RwLock::new(payment_engine)),
        vault: std::sync::Arc::new(vault),
        ledger,
    };

    let bind_addr = format!("{}:{}", config.host, config.port);
    tracing::info!("FeelPay Gateway API listening on {}", bind_addr);

    let worker_count = num_cpus::get() * 2;
    let rate_limiter =
        crate::middleware::rate_limit::RateLimiter::new(config.rate_limit_per_minute);
    let cors_allowed_origins = config.cors_allowed_origins.clone();

    let server = HttpServer::new(move || {
        let mut cors = Cors::default()
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::HeaderName::from_static("x-api-key"),
                header::HeaderName::from_static("idempotency-key"),
            ])
            .max_age(3600);

        if cors_allowed_origins.iter().any(|o| o == "*") {
            cors = cors.allow_any_origin();
        } else {
            for origin in &cors_allowed_origins {
                cors = cors.allowed_origin(origin);
            }
        }

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(pool_for_handler.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            .wrap(rate_limiter.clone())
            // Healthcheck
            .route("/health", web::get().to(api::health::health_check))
            .route("/v1/health", web::get().to(api::health::health_check))
            // Swagger UI (only in non-production)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api::ApiDoc::openapi()),
            )
            // Merchants API
            .service(
                web::scope("/v1/merchants")
                    .route("/register", web::post().to(api::merchants::create_merchant))
                    .route("/login", web::post().to(api::merchants::login_merchant))
                    .route(
                        "/forgot-password",
                        web::post().to(api::merchants::forgot_password),
                    )
                    .route(
                        "/reset-password",
                        web::post().to(api::merchants::reset_password),
                    )
                    // Protected merchant routes
                    .service(
                        web::scope("")
                            .wrap(crate::middleware::auth::JwtMiddleware)
                            .route("/profile", web::get().to(api::merchants::get_profile))
                            .route("/profile", web::put().to(api::merchants::update_profile))
                            .route("/keys", web::get().to(api::merchants::get_keys))
                            .route("/keys/roll", web::post().to(api::merchants::roll_keys)),
                    ),
            )
            // Payment Links API
            .service(
                web::scope("/v1/payment-links")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::payment_links::create_payment_link))
                    .route("", web::get().to(api::payment_links::list_payment_links))
                    .route("/{id}", web::get().to(api::payment_links::get_payment_link))
                    .route("/{id}/cancel", web::post().to(api::payment_links::cancel_payment_link)),
            )
            // Customers API
            .service(
                web::scope("/v1/customers")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::customers::create_customer))
                    .route("", web::get().to(api::customers::list_customers))
                    .route("/{id}", web::get().to(api::customers::get_customer))
                    .route("/{id}", web::delete().to(api::customers::delete_customer))
                    .route("/{id}/payment-methods", web::get().to(api::customers::get_customer_payment_methods)),
            )
            // Notifications & Reports API
            .service(
                web::scope("/v1/notifications")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::notifications::create_notification))
                    .route("", web::get().to(api::notifications::list_notifications)),
            )
            .service(
                web::scope("/v1/reports")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::notifications::create_report))
                    .route("", web::get().to(api::notifications::list_reports))
                    .route("/{id}", web::get().to(api::notifications::get_report)),
            )
            // Public payment link view
            .route("/pay/{link_id}", web::get().to(api::payment_links::view_payment_link))
            // Payments API
            .service(
                web::scope("/v1/payments")
                    // Authenticated via API keys or JWT (for dashboard)
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::payments::create_payment))
                    .route("", web::get().to(api::payments::list_payments))
                    .route("/{id}", web::get().to(api::payments::get_payment))
                    .route(
                        "/{id}/capture",
                        web::post().to(api::payments::capture_payment),
                    )
                    .route(
                        "/{id}/refund",
                        web::post().to(api::payments::refund_payment),
                    ),
            )
            // Providers & Routing API
            .service(
                web::scope("/v1/providers")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::providers::configure),
            )
            // Disputes API
            .service(
                web::scope("/v1/disputes")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::get().to(api::disputes::list_disputes))
                    .route("/{id}", web::get().to(api::disputes::get_dispute))
                    .route(
                        "/{id}/evidence",
                        web::post().to(api::disputes::submit_evidence),
                    )
                    .route(
                        "/{id}/accept",
                        web::post().to(api::disputes::accept_dispute),
                    ),
            )
            // Settlements API
            .service(
                web::scope("/v1/settlements")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::get().to(api::settlements::list_settlements))
                    .route("/balance", web::get().to(crate::api::settlements::get_settlement_stats))
                    .route("/{id}", web::get().to(api::settlements::get_settlement)),
            )
            // Merchant onboarding API (dashboard JWT auth)
            .service(
                web::scope("/v1/onboarding")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("/kyc", web::post().to(api::onboarding::submit_kyc))
                    .route("/status", web::get().to(api::onboarding::onboarding_status))
                    .route(
                        "/bank-account",
                        web::post().to(api::onboarding::submit_bank_account),
                    ),
            )
            // Merchant settings and operational APIs
            .service(
                web::scope("/v1/merchants/{merchant_id}")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    // Bank accounts
                    .route(
                        "/bank-accounts",
                        web::post().to(api::bank_accounts::add_bank_account),
                    )
                    .route(
                        "/bank-accounts",
                        web::get().to(api::bank_accounts::list_bank_accounts),
                    )
                    .route(
                        "/bank-accounts/{account_id}/verify",
                        web::post().to(api::bank_accounts::verify_bank_account),
                    )
                    .route(
                        "/bank-accounts/{account_id}/set-primary",
                        web::post().to(api::bank_accounts::set_primary_bank_account),
                    )
                    // Export jobs
                    .route(
                        "/exports/transactions",
                        web::post().to(api::exports::create_transaction_export),
                    )
                    .route(
                        "/exports/{job_id}",
                        web::get().to(api::exports::get_export_status),
                    )
                    .route(
                        "/exports/{job_id}/download",
                        web::get().to(api::exports::download_export),
                    )
                    .route("/exports", web::get().to(api::exports::list_exports))
                    // Notification preferences
                    .route(
                        "/notification-preferences",
                        web::get().to(api::notification_prefs::get_notification_preferences),
                    )
                    .route(
                        "/notification-preferences/{event_type}",
                        web::put().to(api::notification_prefs::update_notification_preference),
                    )
                    .route(
                        "/notification-preferences/user/{user_id}",
                        web::get().to(api::notification_prefs::get_user_notification_preferences),
                    )
                    .route(
                        "/notification-preferences/user/{user_id}/{event_type}",
                        web::put().to(api::notification_prefs::update_user_notification_preference),
                    )
                    // Scheduled reports
                    .route(
                        "/scheduled-reports",
                        web::post().to(api::reports::create_scheduled_report),
                    )
                    .route(
                        "/scheduled-reports",
                        web::get().to(api::reports::list_scheduled_reports),
                    )
                    .route(
                        "/scheduled-reports/{report_id}",
                        web::get().to(api::reports::get_scheduled_report),
                    )
                    .route(
                        "/scheduled-reports/{report_id}",
                        web::put().to(api::reports::update_scheduled_report),
                    )
                    .route(
                        "/scheduled-reports/{report_id}",
                        web::delete().to(api::reports::delete_scheduled_report),
                    )
                    // Webhook test helpers
                    .route(
                        "/webhooks/test",
                        web::post().to(api::webhook_test::send_test_webhook),
                    )
                    .route(
                        "/webhooks/test-logs",
                        web::get().to(api::webhook_test::get_webhook_test_logs),
                    ),
            )
            // Analytics API
            .service(
                web::scope("/v1/analytics")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("/summary", web::get().to(crate::api::analytics::get_dashboard_stats))
                    .route("/volume", web::get().to(api::analytics::get_volume_chart))
                    .route(
                        "/top-customers",
                        web::get().to(api::analytics::get_top_customers),
                    ),
            )
            // Developers API (API key info)
            .service(
                web::scope("/v1/developers")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("/keys", web::get().to(api::merchants::get_keys))
                    .route(
                        "/webhooks",
                        web::get().to(api::analytics::get_webhook_events),
                    ),
            )
            // Documents API
            .service(
                web::scope("/v1/documents")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::documents::configure),
            )
            // EMI & BNPL API
            .service(
                web::scope("/v1/emi-bnpl")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::emi_bnpl::configure),
            )
            // Fee Plans API
            .service(
                web::scope("/v1/fee-plans")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::fee_plans::configure),
            )
            // Subscriptions API
            .service(
                web::scope("/v1/subscriptions")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::subscriptions::configure),
            )
            // Virtual Accounts API
            .service(
                web::scope("/v1/virtual-accounts")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::virtual_accounts::configure),
            )
            // Mandates API
            .service(
                web::scope("/v1/mandates")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::mandates::configure),
            )
    })
    .workers(worker_count)
    .shutdown_timeout(30)
    .bind(bind_addr)?
    .run();

    let server_handle = server.handle();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::warn!("Shutdown signal received, draining in-flight requests");
                server_handle.stop(true).await;
            }
            Err(e) => {
                tracing::error!("Failed to listen for shutdown signal: {e}");
            }
        }
    });

    server.await
}

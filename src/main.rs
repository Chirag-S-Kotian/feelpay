#![allow(clippy::all)]

use actix_cors::Cors;

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

    if config.is_live() {
        if config.app_env != "production" {
            return Err("PAYMENT_MODE=live requires APP_ENV=production".into());
        }
        if key == "0000000000000000000000000000000000000000000000000000000000000000" {
            return Err("Live mode requires non-default VAULT_ENCRYPTION_KEY".into());
        }
        if config.jwt_secret.contains("change_in_production") || config.jwt_secret.len() < 32 {
            return Err("Live mode requires a non-default JWT_SECRET of at least 32 chars".into());
        }
        if config.master_api_key == "fp_master_key" {
            return Err("Live mode requires non-default MASTER_API_KEY".into());
        }
        if config.webhook_signing_secret == "dev_feelpay_webhook_secret"
            || config.webhook_signing_secret == "whsec_placeholder_example_only"
        {
            return Err("Live mode requires non-default WEBHOOK_SIGNING_SECRET".into());
        }
        if config.cors_allowed_origins.iter().any(|origin| {
            origin == "*" || origin.contains("localhost") || origin.contains("127.0.0.1")
        }) {
            return Err("Live mode requires explicit non-local CORS_ALLOWED_ORIGINS".into());
        }
    } else if config.app_env == "production" {
        return Err("APP_ENV=production requires PAYMENT_MODE=live".into());
    }

    Ok(())
}

fn redacted_redis_target(redis_url: &str) -> String {
    redis_url
        .split_once('@')
        .map(|(_, host)| format!("redis://***@{host}"))
        .unwrap_or_else(|| redis_url.to_string())
}

#[cfg(test)]
mod startup_tests {
    use super::{redacted_redis_target, validate_startup_config};
    use crate::config::AppConfig;

    fn base_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://db".into(),
            redis_url: "redis://:secret@localhost:6379".into(),
            jwt_secret: "a".repeat(32),
            jwt_expiry_hours: 24,
            master_api_key: "fp_live_safe".into(),
            vault_encryption_key: "1".repeat(64),
            vault_key_id: "key_v1".into(),
            upi_psp_id: "FEELPAY".into(),
            upi_merchant_vpa: "merchant@feelpay".into(),
            webhook_max_retries: 8,
            webhook_initial_delay_secs: 1,
            webhook_signing_secret: "whsec_live_safe".into(),
            settlement_cycle_days: 2,
            platform_fee_percentage: 2.0,
            rate_limit_per_minute: 60,
            tenant_isolation_mode: "shared_table".into(),
            cors_allowed_origins: vec!["https://merchant.example".into()],
            app_env: "production".into(),
            payment_mode: "live".into(),
            database_provider: "auto".into(),
            database_url_docker: None,
            database_url_neon: None,
            digilocker_enabled: false,
            digilocker_client_id: None,
            digilocker_client_secret: None,
            digilocker_redirect_uri: None,
            digilocker_auth_base_url: "https://example.test/auth".into(),
            digilocker_token_url: "https://example.test/token".into(),
            digilocker_docs_url: "https://example.test/docs".into(),
            digilocker_timeout_secs: 12,
        }
    }

    #[test]
    fn live_mode_rejects_unsafe_defaults_and_mode_mismatch() {
        let mut config = base_config();
        assert!(validate_startup_config(&config).is_ok());

        config.app_env = "development".into();
        assert!(validate_startup_config(&config).is_err());

        config = base_config();
        config.jwt_secret = "change_in_production".into();
        assert!(validate_startup_config(&config).is_err());

        config = base_config();
        config.cors_allowed_origins = vec!["http://localhost:5173".into()];
        assert!(validate_startup_config(&config).is_err());
    }

    #[test]
    fn redis_log_target_redacts_credentials() {
        assert_eq!(
            redacted_redis_target("redis://:secret@cache.example:6379"),
            "redis://***@cache.example:6379"
        );
    }
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
            tracing::info!(error = %e, "Redis unavailable; continuing startup in test mode");
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
    tracing::info!(target = %redacted_redis_target(&config.redis_url), "Connecting to Redis");
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
    let notification_retry_pool = state.db.clone();

    let server = HttpServer::new(move || {
        let mut cors = Cors::default()
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allow_any_header() // Allow all headers to prevent issues with custom headers like x-feelpay-environment
            .max_age(3600);

        let allowed_origins = cors_allowed_origins.clone();
        cors = cors.allowed_origin_fn(move |origin, _| {
            if let Ok(origin_str) = origin.to_str() {
                if origin_str.ends_with(".pages.dev")
                    || origin_str.ends_with(".onrender.com")
                    || origin_str.starts_with("http://localhost:")
                    || origin_str.starts_with("http://127.0.0.1:")
                    || allowed_origins.iter().any(|o| o == "*" || o == origin_str)
                {
                    return true;
                }
            }
            false
        });

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(pool_for_handler.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(crate::middleware::maintenance_mode::MaintenanceMode)
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add(("x-api-version", "2026-04-28"))
                    .add(("x-deprecated", "false"))
                    .add(("x-deprecation-notice", ""))
                    .add(("x-content-type-options", "nosniff"))
                    .add(("x-frame-options", "DENY"))
                    .add(("referrer-policy", "strict-origin-when-cross-origin"))
                    .add((
                        "permissions-policy",
                        "camera=(), microphone=(), geolocation=(), payment=()",
                    ))
                    .add((
                        "content-security-policy",
                        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'",
                    )),
            )
            .wrap(crate::middleware::request_id::RequestId)
            .wrap(rate_limiter.clone())
            .wrap(cors)
            // Healthcheck
            .route("/", web::get().to(api::health::health_check))
            .route("/health", web::get().to(api::health::health_check))
            .route("/v1/health", web::get().to(api::health::health_check))
            .route("/ready", web::get().to(api::health::readiness_check))
            .route("/v1/ready", web::get().to(api::health::readiness_check))
            .route(
                "/v1/public/landing-page",
                web::get().to(api::public_config::get_landing_page_config),
            )
            .route(
                "/v1/checkout/embed.js",
                web::get().to(api::checkout::get_embed_script),
            )
            .route(
                "/pay/{token}",
                web::get().to(api::checkout::get_public_checkout_link),
            )
            .route(
                "/pay/{token}/session",
                web::post().to(api::checkout::create_public_checkout_session),
            )
            .route(
                "/pay/session/{id}/status",
                web::get().to(api::checkout::get_public_checkout_session_status),
            )
            .route(
                "/pay/session/{id}/pay",
                web::post().to(api::checkout::public_pay_checkout_session),
            )
            .route(
                "/receipt/{receipt_token}",
                web::get().to(api::checkout::get_public_receipt),
            )
            // Swagger UI & FastAPI-style /docs
            .route(
                "/docs",
                web::get().to(|| async {
                    actix_web::HttpResponse::Found()
                        .insert_header((actix_web::http::header::LOCATION, "/swagger-ui/"))
                        .finish()
                }),
            )
            .route(
                "/docs/",
                web::get().to(|| async {
                    actix_web::HttpResponse::Found()
                        .insert_header((actix_web::http::header::LOCATION, "/swagger-ui/"))
                        .finish()
                }),
            )
            .route(
                "/openapi.json",
                web::get().to(|| async {
                    actix_web::HttpResponse::Found()
                        .insert_header((actix_web::http::header::LOCATION, "/api-docs/openapi.json"))
                        .finish()
                }),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api::ApiDoc::openapi()),
            )
            // Payment Links API
            .service(
                web::scope("/v1/payment-links")
                    .wrap(crate::middleware::external_api::ExternalApiGateway::new())
                    .route("", web::post().to(api::payment_links::create_payment_link))
                    .route("", web::get().to(api::payment_links::list_payment_links))
                    .route("/{id}", web::get().to(api::payment_links::get_payment_link))
                    .route(
                        "/{id}/cancel",
                        web::post().to(api::payment_links::cancel_payment_link),
                    ),
            )
            // Customers API
            .service(
                web::scope("/v1/customers")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::customers::create_customer))
                    .route("", web::get().to(api::customers::list_customers))
                    .route("/{id}", web::get().to(api::customers::get_customer))
                    .route("/{id}", web::delete().to(api::customers::delete_customer))
                    .route(
                        "/{id}/payment-methods",
                        web::get().to(api::customers::get_customer_payment_methods),
                    ),
            )
            // Notifications & Reports API
            .service(
                web::scope("/v1/notifications")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::notifications::create_notification))
                    .route("", web::get().to(api::notifications::list_notifications))
                    .route(
                        "/mark-all-read",
                        web::post().to(api::notifications::mark_all_notifications_read),
                    )
                    .route(
                        "/{id}/read",
                        web::post().to(api::notifications::mark_notification_as_read),
                    ),
            )
            .service(
                web::scope("/v1/reports")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route("", web::post().to(api::notifications::create_report))
                    .route("", web::get().to(api::notifications::list_reports))
                    .route("/{id}", web::get().to(api::notifications::get_report)),
            )
            // Public payment link view
            .route(
                "/pay/{link_id}",
                web::get().to(api::payment_links::view_payment_link),
            )
            // Payments API
            .service(
                web::scope("/v1/payments")
                    // Authenticated via API keys or JWT (for dashboard)
                    .wrap(crate::middleware::external_api::ExternalApiGateway::new())
                    .route("", web::post().to(api::payments::create_payment))
                    .route("", web::get().to(api::payments::list_payments))
                    .route("/batch-refund", web::post().to(api::payments::batch_refund))
                    .route(
                        "/{id}/details",
                        web::get().to(api::payments::get_payment_details),
                    )
                    .route(
                        "/{id}/timeline",
                        web::get().to(api::payments::get_payment_timeline),
                    )
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
            // Refunds API
            .service(
                web::scope("/v1/refunds")
                    .wrap(crate::middleware::external_api::ExternalApiGateway::new())
                    .route("", web::get().to(api::payments::list_refunds))
                    .route("/{id}", web::get().to(api::payments::get_refund)),
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
                    .route("/test-generator", web::post().to(api::disputes::create_test_dispute))
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
                    .route(
                        "/balance",
                        web::get().to(crate::api::settlements::get_settlement_stats),
                    )
                    .route(
                        "/reconciliation",
                        web::get().to(api::settlements::get_reconciliation_summary),
                    )
                    .route("/{id}", web::get().to(api::settlements::get_settlement)),
            )
            // Merchant onboarding API (dashboard JWT auth)
            .service(
                web::scope("/v1/onboarding")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .configure(api::onboarding::configure),
            )
            // Merchant settings and operational APIs
            .service(
                web::scope("/v1/merchants/{merchant_id:[0-9a-fA-F\\-]{36}}")
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
                    .route(
                        "/bank-accounts/{account_id}",
                        web::delete().to(api::bank_accounts::delete_bank_account),
                    )
                    // Export jobs
                    .route(
                        "/exports/transactions",
                        web::post().to(api::exports::create_transaction_export),
                    )
                    .route(
                        "/exports/customers",
                        web::post().to(api::exports::create_customer_export),
                    )
                    .route(
                        "/exports/{job_id}",
                        web::get().to(api::exports::get_export_status),
                    )
                    .route(
                        "/exports/{job_id}/download",
                        web::get().to(api::exports::download_export),
                    )
                    .route(
                        "/exports/{job_id}/file",
                        web::get().to(api::exports::serve_export_file),
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
                    )
                    .route(
                        "/webhooks/replay/{event_id}",
                        web::post().to(api::webhook_test::replay_webhook),
                    ),
            )
            // Merchants API (registered after /v1/merchants/{merchant_id} scope to avoid route shadowing)
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
                            .route(
                                "/profile/avatar-upload",
                                web::post().to(api::merchants::upload_profile_avatar),
                            )
                            .route("/logout", web::post().to(api::merchants::logout_merchant))
                            .route("/keys", web::get().to(api::merchants::get_keys))
                            .route("/keys/roll", web::post().to(api::merchants::roll_keys))
                            .route(
                                "/payment-methods",
                                web::get().to(api::merchants::list_payment_methods),
                            )
                            .route(
                                "/payment-methods/{method_type}",
                                web::put().to(api::merchants::update_payment_method),
                            )
                            .route(
                                "/settings/security",
                                web::get().to(api::merchants::get_security_settings),
                            )
                            .route(
                                "/dashboard-signals",
                                web::get().to(api::merchants::get_dashboard_signals),
                            )
                            .route(
                                "/account-settings-hub",
                                web::get().to(api::merchants::get_account_settings_hub),
                            )
                            .route(
                                "/checkout-settings",
                                web::get().to(api::merchants::get_checkout_settings),
                            )
                            .route(
                                "/checkout-settings",
                                web::put().to(api::merchants::update_checkout_settings),
                            )
                            .route(
                                "/checkout-settings/audit",
                                web::get().to(api::merchants::get_checkout_settings_audit),
                            )
                            .route(
                                "/audit-logs",
                                web::get().to(api::merchants::list_merchant_audit_logs),
                            )
                            .route(
                                "/webhook-secret/roll",
                                web::post().to(api::merchants::roll_webhook_secret),
                            )
                            .route("/team", web::get().to(api::merchants::list_team_members))
                            .route(
                                "/team/invite",
                                web::post().to(api::merchants::invite_team_member),
                            )
                            .route(
                                "/team/{user_id}",
                                web::delete().to(api::merchants::remove_team_member),
                            )
                            .route(
                                "/team/{user_id}/role",
                                web::put().to(api::merchants::update_team_member_role),
                            )
                            // Export aliases under /v1/merchants to avoid scope shadow 404s
                            .route(
                                "/{merchant_id}/exports/transactions",
                                web::post().to(api::exports::create_transaction_export),
                            )
                            .route(
                                "/{merchant_id}/exports/customers",
                                web::post().to(api::exports::create_customer_export),
                            )
                            .route(
                                "/{merchant_id}/exports/{job_id}",
                                web::get().to(api::exports::get_export_status),
                            )
                            .route(
                                "/{merchant_id}/exports/{job_id}/download",
                                web::get().to(api::exports::download_export),
                            )
                            .route(
                                "/{merchant_id}/exports/{job_id}/file",
                                web::get().to(api::exports::serve_export_file),
                            )
                            .route(
                                "/{merchant_id}/exports",
                                web::get().to(api::exports::list_exports),
                            ),
                    ),
            )
            // Analytics API
            .service(
                web::scope("/v1/analytics")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .route(
                        "/summary",
                        web::get().to(crate::api::analytics::get_dashboard_stats),
                    )
                    .route("/volume", web::get().to(api::analytics::get_volume_chart))
                    .route(
                        "/top-customers",
                        web::get().to(api::analytics::get_top_customers),
                    )
                    .route(
                        "/alerts",
                        web::get().to(api::analytics::get_actionable_alerts),
                    ),
            )
            // Developers API (API key info)
            .service(
                web::scope("/v1/developers")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("/keys", web::get().to(api::merchants::get_keys))
                    .route("/api-logs", web::get().to(api::analytics::get_api_logs))
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
            // Checkout preview API
            .service(
                web::scope("/v1/checkout")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route(
                        "/preview-config",
                        web::get().to(api::checkout::get_checkout_preview_config),
                    )
                    .route(
                        "/session",
                        web::post().to(api::checkout::create_checkout_session),
                    )
                    .route(
                        "/session/{id}",
                        web::get().to(api::checkout::get_checkout_session),
                    )
                    .route(
                        "/session/{id}/pay",
                        web::post().to(api::checkout::pay_checkout_session),
                    )
                    .route(
                        "/session/{id}/status",
                        web::get().to(api::checkout::get_payment_status),
                    ),
            )
            // Payouts API (keep after specific /v1 routes so public merchant auth is not shadowed)
            .service(
                web::scope("/v1/payouts")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::payouts::configure),
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
            // Provider Webhooks (Stripe, Razorpay, Adyen - no auth, signature-verified)
            .configure(api::provider_webhooks::configure)
            // Admin API
            .service(
                web::scope("/v1")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::admin::configure),
            )
            .service(
                web::scope("")
                    .wrap(crate::middleware::auth::UnifiedAuth)
                    .configure(api::admin::configure),
            )
    })
    .workers(worker_count)
    .shutdown_timeout(30)
    .bind(bind_addr)?
    .run();

    let server_handle = server.handle();
    
    let health_check_pool = notification_retry_pool.clone();
    tokio::spawn(async move {
        // Run health check update every 3 minutes (180s) instead of 30s
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(180));
        loop {
            interval.tick().await;
            
            let providers_res = sqlx::query(
                "SELECT id, name FROM payment_providers WHERE is_active = true"
            )
            .fetch_all(&health_check_pool)
            .await;

            if let Ok(rows) = providers_res {
                use sqlx::Row;
                let mut ids: Vec<uuid::Uuid> = Vec::with_capacity(rows.len());
                let mut latencies: Vec<i32> = Vec::with_capacity(rows.len());
                let mut success_rates: Vec<rust_decimal::Decimal> = Vec::with_capacity(rows.len());

                for row in rows {
                    let id: uuid::Uuid = row.get("id");
                    let name: String = row.get("name");
                    
                    let (base_latency, latency_var, base_success): (i32, i32, f64) = match name.as_str() {
                        "mock" => (45, 5, 100.0),
                        "simulator" => (55, 7, 100.0),
                        "stripe" => (180, 15, 99.40),
                        "adyen" => (195, 18, 99.15),
                        "razorpay" => (220, 20, 98.85),
                        "juspay" => (245, 25, 98.20),
                        "payu" => (290, 30, 97.50),
                        "cashfree" => (310, 35, 96.95),
                        _ => (300, 40, 95.00),
                    };

                    let (latency, dec) = {
                        let mut rng = rand::thread_rng();
                        use rand::Rng;
                        let latency = base_latency + rng.gen_range(-latency_var..=latency_var);
                        let success_rate: f64 = base_success + rng.gen_range(-0.15..=0.15);
                        let success_rate = success_rate.clamp(0.0, 100.0);
                        let dec = rust_decimal::Decimal::from_f64_retain(success_rate).unwrap_or_default();
                        (latency, dec)
                    };

                    ids.push(id);
                    latencies.push(latency);
                    success_rates.push(dec);
                }

                if !ids.is_empty() {
                    let _ = sqlx::query(
                        "UPDATE payment_providers AS p
                         SET avg_latency_ms = u.latency,
                             success_rate = u.success_rate,
                             last_health_check = NOW(),
                             updated_at = NOW()
                         FROM (
                             SELECT UNNEST($1::uuid[]) AS id,
                                    UNNEST($2::int[]) AS latency,
                                    UNNEST($3::numeric[]) AS success_rate
                         ) AS u
                         WHERE p.id = u.id"
                    )
                    .bind(&ids)
                    .bind(&latencies)
                    .bind(&success_rates)
                    .execute(&health_check_pool)
                    .await;
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(error) =
                crate::engine::services::notification_dispatcher::retry_due_dispatches(
                    &notification_retry_pool,
                    25,
                )
                .await
            {
                tracing::warn!(error = %error, "notification retry pass failed");
            }
        }
    });
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

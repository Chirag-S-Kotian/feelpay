use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
pub mod config;
pub mod db;
pub mod engine;
pub mod error;
pub mod models;
pub mod state;

use config::AppConfig;
use engine::payment::PaymentEngine;
use engine::vault::CardVaultService;
use state::AppState;

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
    let config = AppConfig::from_env().expect("Failed to load configuration");

    // Initialize Database
    let database_url = config.selected_database_url().to_string();
    tracing::info!(
        provider = %config.database_provider,
        "Connecting to PostgreSQL with configured provider"
    );
    let pool = db::pool::create_pool(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Initialize Redis
    tracing::info!("Connecting to Redis at {}", config.redis_url);
    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("Failed to connect to Redis");

    // Initialize Vault Service
    tracing::info!("Initializing PCI-ready Card Vault");
    let vault = CardVaultService::new(
        pool.clone(),
        &config.vault_encryption_key,
        config.vault_key_id.clone(),
    )
    .expect("Failed to initialize vault");

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

    // If live, default to real processors. Otherwise default to simulator.
    if config.is_live() {
        tracing::info!("Running in LIVE mode. Transactions are REAL.");
        // Normally we'd use routing rules, here we just set card as default for live
        payment_engine.set_default_processor("card");
    } else {
        tracing::info!("Running in TEST mode.");
        payment_engine.set_default_processor("simulator");
    }

    // Prepare App State
    let pool_for_handler = pool.clone();
    let state = AppState {
        db: pool,
        redis: redis_client,
        config: config.clone(),
        payment_engine: std::sync::Arc::new(tokio::sync::RwLock::new(payment_engine)),
        vault: std::sync::Arc::new(vault),
    };

    let bind_addr = format!("{}:{}", config.host, config.port);
    tracing::info!("FeelPay Gateway API listening on {}", bind_addr);

    let worker_count = num_cpus::get() * 2;

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(pool_for_handler.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            // Healthcheck
            .route("/health", web::get().to(api::health::health_check))
            .route("/v1/health", web::get().to(api::health::health_check))
            // Merchants API
            .service(
                web::scope("/v1/merchants")
                    .route("/register", web::post().to(api::merchants::create_merchant))
                    .route("/login", web::post().to(api::merchants::login_merchant))
                    // Protected merchant routes
                    .service(
                        web::scope("")
                            .wrap(api::auth::JwtMiddleware)
                            .route("/profile", web::get().to(api::merchants::get_profile))
                            .route("/profile", web::put().to(api::merchants::update_profile))
                            .route("/keys", web::get().to(api::merchants::get_keys))
                            .route("/keys/roll", web::post().to(api::merchants::roll_keys)),
                    ),
            )
            // Payments API
            .service(
                web::scope("/v1/payments")
                    // Authenticated via API keys (not JWT)
                    .wrap(api::auth::ApiKeyMiddleware)
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
            // Disputes API
            .service(
                web::scope("/v1/disputes")
                    .wrap(api::auth::ApiKeyMiddleware)
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
                    .wrap(api::auth::ApiKeyMiddleware)
                    .route("", web::get().to(api::settlements::list_settlements))
                    .route("/balance", web::get().to(api::settlements::get_balance))
                    .route("/{id}", web::get().to(api::settlements::get_settlement)),
            )
            // Merchant onboarding API (dashboard JWT auth)
            .service(
                web::scope("/v1/onboarding")
                    .wrap(api::auth::JwtMiddleware)
                    .route("/kyc", web::post().to(api::onboarding::submit_kyc))
                    .route("/status", web::get().to(api::onboarding::onboarding_status))
                    .route(
                        "/bank-account",
                        web::post().to(api::onboarding::submit_bank_account),
                    ),
            )
    })
    .workers(worker_count)
    .bind(bind_addr)?
    .run()
    .await
}

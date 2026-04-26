use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::engine::payment::PaymentEngine;
use crate::engine::vault::CardVaultService;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub config: AppConfig,
    pub payment_engine: Arc<RwLock<PaymentEngine>>,
    pub vault: Arc<CardVaultService>,
}

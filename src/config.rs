use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub master_api_key: String,

    // Vault / Tokenization
    pub vault_encryption_key: String,
    pub vault_key_id: String,

    // UPI PSP
    pub upi_psp_id: String,
    pub upi_merchant_vpa: String,

    // Webhook
    pub webhook_max_retries: i32,
    pub webhook_initial_delay_secs: u64,
    pub webhook_signing_secret: String,

    // Settlement
    pub settlement_cycle_days: i32,
    pub platform_fee_percentage: f64,

    // Rate limiting
    pub rate_limit_per_minute: u32,
    pub tenant_isolation_mode: String,
    pub cors_allowed_origins: Vec<String>,

    // Mode
    pub payment_mode: String,
    pub database_provider: String,
    pub database_url_docker: Option<String>,
    pub database_url_neon: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()?,
            master_api_key: std::env::var("MASTER_API_KEY")
                .unwrap_or_else(|_| "fp_master_key".into()),

            vault_encryption_key: std::env::var("VAULT_ENCRYPTION_KEY").unwrap_or_else(|_| {
                "0000000000000000000000000000000000000000000000000000000000000000".into()
            }),
            vault_key_id: std::env::var("VAULT_KEY_ID").unwrap_or_else(|_| "key_v1".into()),

            upi_psp_id: std::env::var("UPI_PSP_ID").unwrap_or_else(|_| "FEELPAY".into()),
            upi_merchant_vpa: std::env::var("UPI_MERCHANT_VPA")
                .unwrap_or_else(|_| "merchant@feelpay".into()),

            webhook_max_retries: std::env::var("WEBHOOK_MAX_RETRIES")
                .unwrap_or_else(|_| "8".into())
                .parse()?,
            webhook_initial_delay_secs: std::env::var("WEBHOOK_INITIAL_DELAY_SECS")
                .unwrap_or_else(|_| "1".into())
                .parse()?,
            webhook_signing_secret: std::env::var("WEBHOOK_SIGNING_SECRET")
                .unwrap_or_else(|_| "dev_feelpay_webhook_secret".into()),

            settlement_cycle_days: std::env::var("SETTLEMENT_CYCLE_DAYS")
                .unwrap_or_else(|_| "2".into())
                .parse()?,
            platform_fee_percentage: std::env::var("PLATFORM_FEE_PERCENTAGE")
                .unwrap_or_else(|_| "2.0".into())
                .parse()?,

            rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".into())
                .parse()?,
            tenant_isolation_mode: std::env::var("TENANT_ISOLATION_MODE")
                .unwrap_or_else(|_| "shared_table".into()),
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".into())
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect(),

            payment_mode: std::env::var("PAYMENT_MODE").unwrap_or_else(|_| "test".into()),
            database_provider: std::env::var("DATABASE_PROVIDER").unwrap_or_else(|_| "auto".into()),
            database_url_docker: std::env::var("DATABASE_URL_DOCKER").ok(),
            database_url_neon: std::env::var("DATABASE_URL_NEON").ok(),
        })
    }

    pub fn is_live(&self) -> bool {
        self.payment_mode == "live"
    }

    pub fn is_test(&self) -> bool {
        self.payment_mode == "test"
    }

    pub fn selected_database_url(&self) -> &str {
        match self.database_provider.as_str() {
            "docker" => self
                .database_url_docker
                .as_deref()
                .unwrap_or(&self.database_url),
            "neon" => self
                .database_url_neon
                .as_deref()
                .unwrap_or(&self.database_url),
            _ => {
                // auto preference: explicit Neon URL > explicit Docker URL > DATABASE_URL
                self.database_url_neon
                    .as_deref()
                    .or(self.database_url_docker.as_deref())
                    .unwrap_or(&self.database_url)
            }
        }
    }
}

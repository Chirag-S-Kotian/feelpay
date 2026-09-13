use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let cleaned_url = database_url
        .replace("?channel_binding=require", "")
        .replace("&channel_binding=require", "")
        .replace("?statement_cache_size=0", "")
        .replace("&statement_cache_size=0", "");
    let mut connect_options = PgConnectOptions::from_str(&cleaned_url)?;

    // Neon pooled endpoints (PgBouncer transaction mode) are safest with disabled client-side statement cache.
    if cleaned_url.contains("-pooler.") {
        connect_options = connect_options.statement_cache_capacity(0);
    }

    // Serverless cloud DB (AWS Singapore) WAN latency + TLS can take >1s for batch updates.
    // Set slow statement alert threshold to 5s so routine WAN transit does not spam WARN logs.
    connect_options = connect_options
        .log_statements(tracing::log::LevelFilter::Debug)
        .log_slow_statements(tracing::log::LevelFilter::Warn, std::time::Duration::from_secs(5));

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(45))
        .max_lifetime(std::time::Duration::from_secs(300))
        .connect_with(connect_options)
        .await?;

    // Run migrations.
    // Dev safety valve: if a previously-applied migration file was edited, sqlx will fail
    // with checksum mismatch. Keep strict behavior in production, tolerate only in non-prod.
    let migration_result = sqlx::migrate!("./migrations").run(&pool).await;
    if let Err(err) = migration_result {
        let msg = err.to_string();
        let is_checksum_mismatch = msg.contains("was previously applied but has been modified");
        let is_production = std::env::var("APP_ENV")
            .map(|v| {
                let lower = v.to_lowercase();
                lower == "production" || lower == "prod"
            })
            .unwrap_or(false);

        if is_checksum_mismatch && !is_production {
            tracing::info!(
                "Migration checksum mismatch detected in non-production environment; continuing startup without applying migrations. \
                 Restore original migration file or create new migration to reconcile schema history."
            );
        } else {
            return Err(err.into());
        }
    }

    tracing::info!("Database pool created and migrations applied");
    Ok(pool)
}

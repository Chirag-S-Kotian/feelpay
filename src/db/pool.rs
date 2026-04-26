use std::str::FromStr;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let cleaned_url = database_url
        .replace("?channel_binding=require", "")
        .replace("&channel_binding=require", "");
    let mut connect_options = PgConnectOptions::from_str(&cleaned_url)?;

    // Neon pooled endpoints (PgBouncer transaction mode) are safest with disabled client-side statement cache.
    if cleaned_url.contains("-pooler.") {
        connect_options = connect_options.statement_cache_capacity(0);
    }

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(300))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_with(connect_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("Database pool created and migrations applied");
    Ok(pool)
}

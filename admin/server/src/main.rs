use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_cors::Cors;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dotenvy::dotenv;
use std::env;

#[derive(Serialize)]
pub struct PlatformStats {
    pub total_volume: i64,
    pub active_merchants: i64,
    pub failed_payments: i64,
    pub system_health: String,
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
        "#
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
        },
        Err(e) => {
            tracing::error!("Stats error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn list_merchants(pool: web::Data<PgPool>) -> impl Responder {
    let merchants = sqlx::query(
        r#"
        SELECT id, name, email, business_name, business_type, created_at
        FROM merchants
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match merchants {
        Ok(m) => {
            let res: Vec<_> = m.into_iter().map(|row| {
                use sqlx::Row;
                serde_json::json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "email": row.get::<String, _>("email"),
                    "business_name": row.get::<Option<String>, _>("business_name"),
                    "business_type": row.get::<Option<String>, _>("business_type"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                })
            }).collect();
            HttpResponse::Ok().json(res)
        },
        Err(e) => {
            tracing::error!("List error: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

mod middleware;
use middleware::AdminAuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to Postgres");

    println!("Admin server starting on 127.0.0.1:8081");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/admin")
                    .wrap(AdminAuthMiddleware)
                    .route("/stats", web::get().to(get_platform_stats))
                    .route("/merchants", web::get().to(list_merchants))
            )
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}

use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use crate::models::tenant::Tenant;
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tenants")
            .service(create_tenant)
            .service(list_tenants)
            .service(get_tenant)
    );
}

#[derive(serde::Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
}

#[post("")]
async fn create_tenant(
    pool: web::Data<PgPool>,
    req: web::Json<CreateTenantRequest>,
) -> impl Responder {
    let tenant = sqlx::query_as::<_, Tenant>(
        "INSERT INTO tenants (name, slug, domain) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&req.domain)
    .fetch_one(pool.get_ref())
    .await;

    match tenant {
        Ok(t) => HttpResponse::Created().json(t),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("")]
async fn list_tenants(pool: web::Data<PgPool>) -> impl Responder {
    let tenants = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants")
        .fetch_all(pool.get_ref())
        .await;

    match tenants {
        Ok(t) => HttpResponse::Ok().json(t),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/{id}")]
async fn get_tenant(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = $1")
        .bind(*id)
        .fetch_optional(pool.get_ref())
        .await;

    match tenant {
        Ok(Some(t)) => HttpResponse::Ok().json(t),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

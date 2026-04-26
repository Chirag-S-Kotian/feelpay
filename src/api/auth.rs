use actix_web::{post, web, HttpResponse, Responder};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::{User, Claims};
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(login)
            .service(register)
    );
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub role: String,
}

#[post("/login")]
async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND status = 'active'"
    )
    .bind(&req.email)
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(u)) => {
            let parsed_hash = PasswordHash::new(&u.password_hash).unwrap();
            if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_ok() {
                // Fetch roles
                let role: (String,) = sqlx::query_as(
                    "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1 LIMIT 1"
                )
                .bind(u.id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(("user".to_string(),));

                let token = generate_jwt(&u, &role.0);
                HttpResponse::Ok().json(AuthResponse {
                    token,
                    user_id: u.id,
                    tenant_id: u.tenant_id,
                    merchant_id: u.merchant_id,
                    role: role.0,
                })
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub tenant_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
}

#[post("/register")]
async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (tenant_id, merchant_id, email, password_hash, full_name, status) 
         VALUES ($1, $2, $3, $4, $5, 'active') RETURNING *"
    )
    .bind(req.tenant_id)
    .bind(req.merchant_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.full_name)
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(u) => HttpResponse::Created().json(u),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

fn generate_jwt(user: &User, role: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        tenant_id: user.tenant_id,
        merchant_id: user.merchant_id,
        role: role.to_string(),
        exp: now + 86400, // 24 hours
        iat: now,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

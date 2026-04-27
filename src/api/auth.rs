use crate::engine::services::auth::MfaService;
use crate::models::user::{Claims, User};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(login)
            .service(register)
            .service(mfa_setup)
            .service(mfa_confirm)
            .service(mfa_verify),
    );
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_token: Option<String>,
}

#[derive(Deserialize)]
pub struct MfaSetupRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
}

#[derive(Deserialize)]
pub struct MfaVerifyRequest {
    pub code: String,
    pub mfa_token: Option<String>, // Used for login step 2
    pub secret: Option<String>,    // Used for initial setup
}

#[post("/login")]
async fn login(pool: web::Data<PgPool>, req: web::Json<LoginRequest>) -> impl Responder {
    let user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND status = 'active'")
            .bind(&req.email)
            .fetch_optional(pool.get_ref())
            .await;

    match user {
        Ok(Some(u)) => {
            let parsed_hash = PasswordHash::new(&u.password_hash).unwrap();
            if Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                // Fetch roles
                let role: (String,) = sqlx::query_as(
                    "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1 LIMIT 1"
                )
                .bind(u.id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(("user".to_string(),));

                if u.mfa_enabled {
                    let mfa_token = generate_jwt_with_exp(&u, "mfa_pending", 300); // 5 mins
                    return HttpResponse::Ok().json(AuthResponse {
                        token: None,
                        user_id: Some(u.id),
                        tenant_id: u.tenant_id,
                        merchant_id: u.merchant_id,
                        role: None,
                        mfa_required: Some(true),
                        mfa_token: Some(mfa_token),
                    });
                }

                let token = generate_jwt(&u, &role.0);
                HttpResponse::Ok().json(AuthResponse {
                    token: Some(token),
                    user_id: Some(u.id),
                    tenant_id: u.tenant_id,
                    merchant_id: u.merchant_id,
                    role: Some(role.0),
                    mfa_required: Some(false),
                    mfa_token: None,
                })
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/mfa/setup")]
async fn mfa_setup(_pool: web::Data<PgPool>, req: web::Json<MfaSetupRequest>) -> impl Responder {
    let secret = MfaService::generate_secret();
    match MfaService::generate_qr_code(&req.email, &secret) {
        Ok(qr_code_url) => HttpResponse::Ok().json(MfaSetupResponse {
            secret,
            qr_code_url,
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/mfa/confirm")]
async fn mfa_confirm(
    pool: web::Data<PgPool>,
    req: web::Json<MfaVerifyRequest>,
    // Need user_id from auth middleware or pass it in req
) -> impl Responder {
    // In a real app, we'd get user_id from the authenticated session
    // For now, we'll assume the request has it or we use a temp token
    if let (Some(secret), Some(mfa_token)) = (&req.secret, &req.mfa_token) {
        let secret_key =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
        let token_data = jsonwebtoken::decode::<Claims>(
            mfa_token,
            &jsonwebtoken::DecodingKey::from_secret(secret_key.as_bytes()),
            &jsonwebtoken::Validation::default(),
        );

        match token_data {
            Ok(data) => {
                let user_id = Uuid::parse_str(&data.claims.sub).unwrap();
                match MfaService::verify_token(secret, &req.code) {
                    Ok(true) => {
                        let result = sqlx::query(
                            "UPDATE users SET mfa_enabled = true, mfa_secret = $2 WHERE id = $1",
                        )
                        .bind(user_id)
                        .bind(secret)
                        .execute(pool.get_ref())
                        .await;

                        match result {
                            Ok(_) => HttpResponse::Ok().body("MFA enabled successfully"),
                            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                        }
                    }
                    Ok(false) => HttpResponse::BadRequest().body("Invalid code"),
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            }
            Err(_) => HttpResponse::Unauthorized().body("Invalid session token"),
        }
    } else {
        HttpResponse::BadRequest().body("Missing secret or mfa_token")
    }
}
#[post("/mfa/verify")]
async fn mfa_verify(pool: web::Data<PgPool>, req: web::Json<MfaVerifyRequest>) -> impl Responder {
    if let Some(mfa_token) = &req.mfa_token {
        // Login step 2 verification
        let secret_key =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "feelpay_jwt_secret".into());
        let token_data = jsonwebtoken::decode::<Claims>(
            mfa_token,
            &jsonwebtoken::DecodingKey::from_secret(secret_key.as_bytes()),
            &jsonwebtoken::Validation::default(),
        );

        match token_data {
            Ok(data) => {
                if data.claims.role != "mfa_pending" {
                    return HttpResponse::Unauthorized().body("Invalid MFA token role");
                }

                let user_id = Uuid::parse_str(&data.claims.sub).unwrap();
                let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                    .bind(user_id)
                    .fetch_one(pool.get_ref())
                    .await;

                match user {
                    Ok(u) => {
                        if let Some(mfa_secret) = &u.mfa_secret {
                            match MfaService::verify_token(mfa_secret, &req.code) {
                                Ok(true) => {
                                    let role: (String,) = sqlx::query_as(
                                        "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1 LIMIT 1"
                                    )
                                    .bind(u.id)
                                    .fetch_one(pool.get_ref())
                                    .await
                                    .unwrap_or(("user".to_string(),));

                                    let token = generate_jwt(&u, &role.0);
                                    HttpResponse::Ok().json(AuthResponse {
                                        token: Some(token),
                                        user_id: Some(u.id),
                                        tenant_id: u.tenant_id,
                                        merchant_id: u.merchant_id,
                                        role: Some(role.0),
                                        mfa_required: Some(false),
                                        mfa_token: None,
                                    })
                                }
                                Ok(false) => HttpResponse::BadRequest().body("Invalid code"),
                                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                            }
                        } else {
                            HttpResponse::BadRequest().body("MFA not set up for this user")
                        }
                    }
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            }
            Err(_) => HttpResponse::Unauthorized().body("Invalid MFA token"),
        }
    } else {
        HttpResponse::BadRequest().body("Missing mfa_token")
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
async fn register(pool: web::Data<PgPool>, req: web::Json<RegisterRequest>) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (tenant_id, merchant_id, email, password_hash, full_name, status) 
         VALUES ($1, $2, $3, $4, $5, 'active') RETURNING *",
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
    generate_jwt_with_exp(user, role, 86400) // 24 hours
}

fn generate_jwt_with_exp(user: &User, role: &str, exp_seconds: usize) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        tenant_id: user.tenant_id,
        merchant_id: user.merchant_id,
        role: role.to_string(),
        exp: now + exp_seconds,
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

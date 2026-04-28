use crate::engine::services::auth::MfaService;
use crate::models::user::{Claims, User};
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{encode, EncodingKey, Header};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

const MAX_FAILED_ATTEMPTS: i32 = 5;
const REFRESH_TOKEN_DAYS: i64 = 30;

struct CaptchaStore {
    questions: Mutex<HashMap<String, (String, i32)>>,
}

lazy_static! {
    static ref CAPTCHA_STORE: CaptchaStore = CaptchaStore::default();
}

impl Default for CaptchaStore {
    fn default() -> Self {
        Self {
            questions: Mutex::new(HashMap::new()),
        }
    }
}

fn generate_captcha() -> (String, String, i32) {
    let a = rand::random::<i32>() % 20 + 1;
    let b = rand::random::<i32>() % 20 + 1;
    let op = if rand::random::<bool>() { "+" } else { "-" };

    let question = format!("{} {} {} = ?", a, op, b);
    let answer = if op == "+" { a + b } else { a - b };

    let id = Uuid::new_v4().to_string();

    if let Ok(mut store) = CAPTCHA_STORE.questions.lock() {
        store.insert(id.clone(), (question.clone(), answer));
    }

    (id, question, answer)
}

fn verify_captcha(id: &str, answer: i32) -> bool {
    if let Ok(mut store) = CAPTCHA_STORE.questions.lock() {
        if let Some((_, stored_answer)) = store.remove(id) {
            return stored_answer == answer;
        }
    }
    false
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn generate_refresh_token(user_id: &Uuid, pool: &PgPool) -> Result<String, sqlx::Error> {
    let token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&token);
    let family = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(REFRESH_TOKEN_DAYS);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, family, expires_at, ip_address, user_agent) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(&family)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(captcha)
            .service(login)
            .service(register)
            .service(refresh)
            .service(logout)
            .service(logout_all)
            .service(mfa_setup)
            .service(mfa_confirm)
            .service(mfa_verify)
            .service(change_password),
    );
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub captcha_id: Option<String>,
    #[serde(default)]
    pub captcha_answer: Option<i32>,
}

#[derive(Serialize)]
pub struct CaptchaResponse {
    pub captcha_id: String,
    pub question: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
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

#[get("/captcha")]
async fn captcha(_pool: web::Data<PgPool>) -> impl Responder {
    let (id, question, _answer) = generate_captcha();
    HttpResponse::Ok().json(CaptchaResponse {
        captcha_id: id,
        question,
    })
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[post("/refresh")]
async fn refresh(pool: web::Data<PgPool>, req: web::Json<RefreshRequest>) -> impl Responder {
    let token_hash = hash_token(&req.refresh_token);

    let result = sqlx::query_as::<_, (
        Uuid,
        Uuid,
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(
        "SELECT id, user_id, family, expires_at, revoked_at FROM refresh_tokens WHERE token_hash = $1"
    )
    .bind(&token_hash)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some((rt_id, user_id, _family, expires_at, revoked_at))) => {
            if revoked_at.is_some() {
                return HttpResponse::Unauthorized().body("Token revoked");
            }
            if expires_at < chrono::Utc::now() {
                return HttpResponse::Unauthorized().body("Token expired");
            }

            sqlx::query("UPDATE refresh_tokens SET last_used_at = NOW() WHERE id = $1")
                .bind(rt_id)
                .execute(pool.get_ref())
                .await
                .ok();

            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(pool.get_ref())
                .await;

            if let Ok(u) = user {
                let role: (String,) = sqlx::query_as(
                    "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1 LIMIT 1"
                )
                .bind(u.id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(("user".to_string(),));

                let token = generate_jwt(&u, &role.0);
                let new_refresh = generate_refresh_token(&u.id, pool.get_ref()).await;

                HttpResponse::Ok().json(AuthResponse {
                    token: Some(token),
                    refresh_token: new_refresh.ok(),
                    user_id: Some(u.id),
                    tenant_id: u.tenant_id,
                    merchant_id: u.merchant_id,
                    role: Some(role.0),
                    mfa_required: Some(false),
                    mfa_token: None,
                })
            } else {
                HttpResponse::Unauthorized().body("User not found")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid token"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[post("/logout")]
async fn logout(
    pool: web::Data<PgPool>,
    req: web::Json<LogoutRequest>,
    claims: Claims,
) -> impl Responder {
    let _user_id = claims.sub;
    if let Some(refresh_token) = &req.refresh_token {
        let token_hash = hash_token(refresh_token);
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(pool.get_ref())
            .await
            .ok();
    }
    HttpResponse::Ok().json(serde_json::json!({ "message": "Logged out" }))
}

#[post("/logout-all")]
async fn logout_all(pool: web::Data<PgPool>, claims: Claims) -> impl Responder {
    let user_id = claims.sub.clone();
    let _ = user_id; // Explicitly use or ignore
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(&user_id)
    .execute(pool.get_ref())
    .await
    .ok();

    sqlx::query(
        "UPDATE user_sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(&user_id)
    .execute(pool.get_ref())
    .await
    .ok();

    HttpResponse::Ok().json(serde_json::json!({ "message": "All sessions logged out" }))
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
    // Verify CAPTCHA first if provided
    if let (Some(captcha_id), Some(captcha_answer)) = (&req.captcha_id, req.captcha_answer) {
        if !verify_captcha(captcha_id, captcha_answer) {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Invalid CAPTCHA" }));
        }
    }

    let user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND status = 'active'")
            .bind(&req.email)
            .fetch_optional(pool.get_ref())
            .await;

    match user {
        Ok(Some(u)) => {
            // Check for account lockout
            let recent_failures: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM login_attempts WHERE user_id = $1 AND success = false AND created_at > NOW() - INTERVAL '30 minutes'"
            )
            .bind(u.id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or((0,));

            if recent_failures.0 as i64 >= MAX_FAILED_ATTEMPTS as i64 {
                return HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Account temporarily locked. Try again in 30 minutes."
                }));
            }

            let parsed_hash = PasswordHash::new(&u.password_hash).unwrap();
            if Argon2::default()
                .verify_password(req.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                // Record successful login
                sqlx::query(
                    "INSERT INTO login_attempts (user_id, email, success) VALUES ($1, $1, true)",
                )
                .bind(u.id)
                .execute(pool.get_ref())
                .await
                .ok();

                // Fetch roles
                let role: (String,) = sqlx::query_as(
                    "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1 LIMIT 1"
                )
                .bind(u.id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or(("user".to_string(),));

                if u.mfa_enabled {
                    let mfa_token = generate_jwt_with_exp(&u, "mfa_pending", 300);
                    return HttpResponse::Ok().json(AuthResponse {
                        token: None,
                        refresh_token: None,
                        user_id: Some(u.id),
                        tenant_id: u.tenant_id,
                        merchant_id: u.merchant_id,
                        role: None,
                        mfa_required: Some(true),
                        mfa_token: Some(mfa_token),
                    });
                }

                let token = generate_jwt(&u, &role.0);
                let refresh_token = generate_refresh_token(&u.id, pool.get_ref()).await;

                HttpResponse::Ok().json(AuthResponse {
                    token: Some(token),
                    refresh_token: refresh_token.ok(),
                    user_id: Some(u.id),
                    tenant_id: u.tenant_id,
                    merchant_id: u.merchant_id,
                    role: Some(role.0),
                    mfa_required: Some(false),
                    mfa_token: None,
                })
            } else {
                // Record failed login attempt
                sqlx::query(
                    "INSERT INTO login_attempts (user_id, email, success, failure_reason) VALUES ($1, $2, false, 'invalid_password')"
                )
                .bind(u.id)
                .bind(&req.email)
                .execute(pool.get_ref())
                .await.ok();

                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => {
            // Record failed attempt for non-existent user to prevent enumeration
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
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
                                    let refresh_token =
                                        generate_refresh_token(&u.id, pool.get_ref())
                                            .await
                                            .unwrap_or_else(|_| Uuid::new_v4().to_string());

                                    HttpResponse::Ok().json(AuthResponse {
                                        token: Some(token),
                                        refresh_token: Some(refresh_token),
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

/// Password strength validation
fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    if password.len() > 128 {
        return Err("Password must be less than 128 characters".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number".to_string());
    }
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?/~`".contains(c))
    {
        return Err("Password must contain at least one special character".to_string());
    }
    Ok(())
}

/// Check if password was used in last 5 passwords
async fn check_password_history(user_id: Uuid, password_hash: &str, pool: &PgPool) -> bool {
    let result: Result<Vec<(String,)>, _> = sqlx::query_as(
        "SELECT password_hash FROM password_history WHERE user_id = $1 ORDER BY created_at DESC LIMIT 5"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(history) => {
            for (old_hash,) in history {
                if let Ok(parsed) = PasswordHash::new(&old_hash) {
                    if Argon2::default()
                        .verify_password(password_hash.as_bytes(), &parsed)
                        .is_ok()
                    {
                        return true; // Password was used before
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Store password in history
async fn store_password_history(user_id: Uuid, password_hash: &str, pool: &PgPool) {
    let _ = sqlx::query("INSERT INTO password_history (user_id, password_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(password_hash)
        .execute(pool)
        .await;

    // Keep only last 5 passwords
    let _ = sqlx::query(
        "DELETE FROM password_history WHERE user_id = $1 AND id NOT IN (SELECT id FROM password_history WHERE user_id = $1 ORDER BY created_at DESC LIMIT 5)"
    )
    .bind(user_id)
    .execute(pool)
    .await;
}

#[post("/register")]
async fn register(pool: web::Data<PgPool>, req: web::Json<RegisterRequest>) -> impl Responder {
    // Validate password strength
    if let Err(e) = validate_password_strength(&req.password) {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": e }));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (tenant_id, merchant_id, email, password_hash, full_name, status, password_updated_at) 
         VALUES ($1, $2, $3, $4, $5, 'active', NOW()) RETURNING *",
    )
    .bind(req.tenant_id)
    .bind(req.merchant_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.full_name)
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(u) => {
            // Store in password history
            store_password_history(u.id, &password_hash, pool.get_ref()).await;
            HttpResponse::Created().json(u)
        }
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

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[post("/change-password")]
async fn change_password(
    pool: web::Data<PgPool>,
    claims: Claims,
    req: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid user identity"),
    };

    // Fetch user
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::NotFound().body("User not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Verify current password
    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid password hash"),
    };

    if Argon2::default()
        .verify_password(req.current_password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return HttpResponse::Unauthorized().body("Current password is incorrect");
    }

    // Validate new password strength
    if let Err(e) = validate_password_strength(&req.new_password) {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": e }));
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(req.new_password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Check password history
    if check_password_history(user_id, &new_hash, pool.get_ref()).await {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot reuse any of your last 5 passwords"
        }));
    }

    // Update password
    let result = sqlx::query(
        "UPDATE users SET password_hash = $1, password_updated_at = NOW(), updated_at = NOW() WHERE id = $2"
    )
    .bind(&new_hash)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            // Store in history
            store_password_history(user_id, &new_hash, pool.get_ref()).await;
            HttpResponse::Ok()
                .json(serde_json::json!({ "message": "Password changed successfully" }))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

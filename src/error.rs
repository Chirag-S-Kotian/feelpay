use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum FeelPayError {
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Payment processing error: {0}")]
    PaymentError(String),

    #[error("Connector error: {0}")]
    ConnectorError(String),

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Idempotency conflict: request already processed")]
    IdempotencyConflict,
}

impl ResponseError for FeelPayError {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = match self {
            Self::Unauthorized(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden(_) => (actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "CONFLICT"),
            Self::RateLimited => (
                actix_web::http::StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
            ),
            Self::PaymentError(_) => (
                actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
                "PAYMENT_ERROR",
            ),
            Self::ConnectorError(_) => {
                (actix_web::http::StatusCode::BAD_GATEWAY, "CONNECTOR_ERROR")
            }
            Self::InvalidStateTransition { .. } => (
                actix_web::http::StatusCode::CONFLICT,
                "INVALID_STATE_TRANSITION",
            ),
            Self::DatabaseError(_) | Self::Database(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
            ),
            Self::InternalError(_) | Self::Internal(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
            Self::IdempotencyConflict => (
                actix_web::http::StatusCode::CONFLICT,
                "IDEMPOTENCY_CONFLICT",
            ),
        };

        HttpResponse::build(status).json(ApiError {
            code: code.to_string(),
            message: self.to_string(),
            details: None,
        })
    }
}

impl From<sqlx::Error> for FeelPayError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", e);
        match e {
            sqlx::Error::RowNotFound => Self::NotFound("Resource not found".into()),
            sqlx::Error::Database(ref db_err) => {
                if db_err.is_unique_violation() {
                    Self::Conflict("Resource already exists".into())
                } else {
                    Self::DatabaseError(e.to_string())
                }
            }
            _ => Self::DatabaseError(e.to_string()),
        }
    }
}

impl From<redis::RedisError> for FeelPayError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!("Redis error: {:?}", e);
        Self::InternalError(format!("Cache error: {}", e))
    }
}

impl From<reqwest::Error> for FeelPayError {
    fn from(e: reqwest::Error) -> Self {
        tracing::error!("HTTP client error: {:?}", e);
        Self::ConnectorError(e.to_string())
    }
}

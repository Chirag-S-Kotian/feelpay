use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProvider {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
    pub is_default: bool,
    pub success_rate: f64,
    pub avg_latency_ms: i32,
    pub last_health_check: Option<DateTime<Utc>>,
    pub circuit_breaker_state: CircuitBreakerState,
    pub failure_count: i32,
    pub last_failure: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl From<&str> for CircuitBreakerState {
    fn from(s: &str) -> Self {
        match s {
            "closed" => CircuitBreakerState::Closed,
            "open" => CircuitBreakerState::Open,
            "half_open" => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Closed,
        }
    }
}

impl ToString for CircuitBreakerState {
    fn to_string(&self) -> String {
        match self {
            CircuitBreakerState::Closed => "closed".to_string(),
            CircuitBreakerState::Open => "open".to_string(),
            CircuitBreakerState::HalfOpen => "half_open".to_string(),
        }
    }
}

impl From<PgRow> for PaymentProvider {
    fn from(row: PgRow) -> Self {
        PaymentProvider {
            id: row.get("id"),
            name: row.get("name"),
            display_name: row.get("display_name"),
            is_active: row.get("is_active"),
            is_default: row.get("is_default"),
            success_rate: row.get("success_rate"),
            avg_latency_ms: row.get("avg_latency_ms"),
            last_health_check: row.get("last_health_check"),
            circuit_breaker_state: CircuitBreakerState::from(row.get::<String, _>("circuit_breaker_state").as_str()),
            failure_count: row.get("failure_count"),
            last_failure: row.get("last_failure"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRoutingRule {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub payment_method: String,
    pub card_brand: Option<String>,
    pub amount_min: Option<i64>,
    pub amount_max: Option<i64>,
    pub provider_id: Uuid,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgRow> for ProviderRoutingRule {
    fn from(row: PgRow) -> Self {
        ProviderRoutingRule {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            merchant_id: row.get("merchant_id"),
            payment_method: row.get("payment_method"),
            card_brand: row.get("card_brand"),
            amount_min: row.get("amount_min"),
            amount_max: row.get("amount_max"),
            provider_id: row.get("provider_id"),
            priority: row.get("priority"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub display_name: String,
    pub api_key_hash: Option<String>,
    pub api_secret_hash: Option<String>,
    pub webhook_secret_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoutingRuleRequest {
    pub merchant_id: Option<Uuid>,
    pub payment_method: String,
    pub card_brand: Option<String>,
    pub amount_min: Option<i64>,
    pub amount_max: Option<i64>,
    pub provider_id: Uuid,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
    pub success_rate: f64,
    pub avg_latency_ms: i32,
    pub circuit_breaker_state: String,
    pub last_health_check: Option<DateTime<Utc>>,
}

impl From<PaymentProvider> for ProviderHealthResponse {
    fn from(p: PaymentProvider) -> Self {
        ProviderHealthResponse {
            id: p.id,
            name: p.name,
            display_name: p.display_name,
            is_active: p.is_active,
            success_rate: p.success_rate,
            avg_latency_ms: p.avg_latency_ms,
            circuit_breaker_state: p.circuit_breaker_state.to_string(),
            last_health_check: p.last_health_check,
        }
    }
}
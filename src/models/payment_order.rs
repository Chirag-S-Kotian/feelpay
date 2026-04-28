use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use std::str::FromStr;

use crate::error::FeelPayError;

/// Payment order statuses with valid state transitions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "snake_case")]
pub enum PaymentStatus {
    Created,
    Processing,
    Authorized,
    Captured,
    Settled,
    Failed,
    Voided,
    Refunded,
    PartiallyRefunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Processing => "processing",
            Self::Authorized => "authorized",
            Self::Captured => "captured",
            Self::Settled => "settled",
            Self::Failed => "failed",
            Self::Voided => "voided",
            Self::Refunded => "refunded",
            Self::PartiallyRefunded => "partially_refunded",
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = FeelPayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "processing" => Ok(Self::Processing),
            "authorized" => Ok(Self::Authorized),
            "captured" => Ok(Self::Captured),
            "settled" => Ok(Self::Settled),
            "failed" => Ok(Self::Failed),
            "voided" => Ok(Self::Voided),
            "refunded" => Ok(Self::Refunded),
            "partially_refunded" => Ok(Self::PartiallyRefunded),
            _ => Err(FeelPayError::BadRequest(format!(
                "Invalid payment status: {}",
                s
            ))),
        }
    }
}

impl PaymentStatus {
    /// Validate state transition. Returns error if transition invalid.
    pub fn can_transition_to(&self, target: &PaymentStatus) -> Result<(), FeelPayError> {
        let valid = match self {
            Self::Created => matches!(target, Self::Processing | Self::Failed),
            Self::Processing => matches!(target, Self::Authorized | Self::Captured | Self::Failed),
            Self::Authorized => matches!(target, Self::Captured | Self::Voided | Self::Failed),
            Self::Captured => matches!(
                target,
                Self::Settled | Self::Refunded | Self::PartiallyRefunded
            ),
            Self::Settled => matches!(target, Self::Refunded | Self::PartiallyRefunded),
            Self::PartiallyRefunded => matches!(target, Self::Refunded),
            // Terminal states
            Self::Failed | Self::Voided | Self::Refunded => false,
        };

        if valid {
            Ok(())
        } else {
            Err(FeelPayError::InvalidStateTransition {
                from: self.as_str().to_string(),
                to: target.as_str().to_string(),
            })
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Failed | Self::Voided | Self::Refunded | Self::Settled
        )
    }
}

/// Payment methods supported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Upi,
    Card,
    NetBanking,
    Wallet,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Upi => "upi",
            Self::Card => "card",
            Self::NetBanking => "net_banking",
            Self::Wallet => "wallet",
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = FeelPayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "upi" => Ok(Self::Upi),
            "card" => Ok(Self::Card),
            "net_banking" => Ok(Self::NetBanking),
            "wallet" => Ok(Self::Wallet),
            _ => Err(FeelPayError::BadRequest(format!(
                "Invalid payment method: {}",
                s
            ))),
        }
    }
}

/// Connector types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    Simulator,
    CoreUpiRail,
    CoreCardRail,
}

impl ConnectorType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Simulator => "simulator",
            Self::CoreUpiRail => "core_upi_rail",
            Self::CoreCardRail => "core_card_rail",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PaymentOrder {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub order_id: String,
    pub amount: i64, // in paise
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_method: Option<String>,
    pub payment_method_data: Option<serde_json::Value>,
    pub connector: Option<String>,
    pub customer_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub description: Option<String>,
    pub notes: Option<serde_json::Value>,
    pub return_url: Option<String>,
    pub idempotency_key: Option<String>,
    pub gateway_reference: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub refunded_amount: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub captured_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: i64,              // in paise
    pub currency: Option<String>, // default INR
    pub payment_method: Option<String>,
    pub payment_method_data: Option<serde_json::Value>,
    pub connector: Option<String>, // simulator/core rails — auto-routes if not set
    pub customer_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub description: Option<String>,
    pub notes: Option<serde_json::Value>,
    pub return_url: Option<String>,
    pub order_id: String,
    pub idempotency_key: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CapturePaymentRequest {
    pub amount: Option<i64>, // for partial capture
}

#[derive(Debug, Deserialize)]
pub struct PaymentListQuery {
    pub status: Option<String>,
    pub payment_method: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaymentListResponse {
    pub payments: Vec<PaymentOrder>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Connector response after processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorResponse {
    pub status: PaymentStatus,
    pub gateway_reference: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub redirect_url: Option<String>,
}

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::FeelPayError;
use crate::models::payment_order::PaymentStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    Card,
    Upi,
    NetBanking,
    Wallet,
    Simulator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodData {
    Card {
        number: Option<String>,
        token: Option<String>,
        expiry_month: Option<u8>,
        expiry_year: Option<u8>,
        cvv: Option<String>,
        holder_name: Option<String>,
    },
    Upi {
        vpa: Option<String>,
        flow: Option<String>, // "collect" or "intent"
    },
    NetBanking {
        bank_code: String,
    },
    Wallet {
        wallet_name: String,
        phone: Option<String>,
    },
    Simulator {
        action: String, // "success", "fail", "delay"
    },
}

#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub payment_id: Uuid,
    pub merchant_id: Uuid,
    pub order_id: String,
    pub amount: i64,
    pub currency: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessorResponse {
    pub status: PaymentStatus,
    pub gateway_reference: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub redirect_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[async_trait]
pub trait PaymentMethodProcessor: Send + Sync {
    fn method_type(&self) -> PaymentMethod;
    async fn authorize(&self, txn: &TransactionContext, method_data: &MethodData) -> Result<ProcessorResponse, FeelPayError>;
    async fn capture(&self, txn: &TransactionContext, amount: i64) -> Result<ProcessorResponse, FeelPayError>;
    async fn void(&self, txn: &TransactionContext) -> Result<ProcessorResponse, FeelPayError>;
    async fn refund(&self, txn: &TransactionContext, amount: i64) -> Result<ProcessorResponse, FeelPayError>;
}

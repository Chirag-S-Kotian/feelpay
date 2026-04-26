use async_trait::async_trait;
use uuid::Uuid;

use crate::engine::processor::{
    MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::error::FeelPayError;
use crate::models::payment_order::PaymentStatus;

/// UPI processor — handles UPI Collect, Intent, and QR flows
pub struct UpiProcessor {
    pub psp_id: String,
    pub merchant_vpa: String,
}

impl UpiProcessor {
    pub fn new(psp_id: String, merchant_vpa: String) -> Self {
        Self {
            psp_id,
            merchant_vpa,
        }
    }

    /// Validate VPA format (username@handle)
    pub fn validate_vpa(vpa: &str) -> bool {
        let parts: Vec<&str> = vpa.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let username = parts[0];
        let handle = parts[1];

        !username.is_empty()
            && !handle.is_empty()
            && username.len() <= 50
            && handle.len() <= 50
            && username
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '_')
            && handle.chars().all(|c| c.is_alphanumeric())
    }

    /// Generate UPI deep link URI
    fn generate_intent_url(&self, txn: &TransactionContext) -> String {
        let amount_rupees = txn.amount as f64 / 100.0;
        format!(
            "upi://pay?pa={}&pn=FeelPay&am={:.2}&tr={}&cu=INR&mc=5411",
            self.merchant_vpa, amount_rupees, txn.order_id
        )
    }

    /// Generate UPI QR string
    fn generate_qr_string(&self, txn: &TransactionContext) -> String {
        let amount_rupees = txn.amount as f64 / 100.0;
        format!(
            "upi://pay?pa={}&pn=FeelPay&am={:.2}&tr={}&cu=INR&mode=02&purpose=00",
            self.merchant_vpa, amount_rupees, txn.order_id
        )
    }
}

#[async_trait]
impl PaymentMethodProcessor for UpiProcessor {
    fn method_type(&self) -> PaymentMethod {
        PaymentMethod::Upi
    }

    async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let (vpa, flow) = match method_data {
            MethodData::Upi { vpa, flow } => (vpa.clone(), flow.clone()),
            _ => return Err(FeelPayError::BadRequest("Expected UPI payment data".into())),
        };

        let flow_type = flow.as_deref().unwrap_or("collect");
        let upi_txn_id = format!("FP{}UPI{}", self.psp_id, Uuid::new_v4().simple());

        match flow_type {
            "collect" => {
                // UPI Collect — needs payer VPA
                let payer_vpa = vpa.ok_or_else(|| {
                    FeelPayError::BadRequest("VPA required for UPI collect".into())
                })?;

                if !Self::validate_vpa(&payer_vpa) {
                    return Err(FeelPayError::BadRequest("Invalid VPA format".into()));
                }

                // In production: send collect request to NPCI
                // For now: simulate immediate success in test mode
                Ok(ProcessorResponse {
                    status: PaymentStatus::Captured,
                    gateway_reference: Some(upi_txn_id.clone()),
                    gateway_response: Some(serde_json::json!({
                        "upi_txn_id": upi_txn_id,
                        "flow": "collect",
                        "payer_vpa": payer_vpa,
                        "payee_vpa": self.merchant_vpa,
                        "rrn": format!("{:012}", rand::random::<u64>() % 1_000_000_000_000),
                    })),
                    error_code: None,
                    error_message: None,
                    redirect_url: None,
                    metadata: Some(serde_json::json!({"upi_flow": "collect"})),
                })
            }
            "intent" => {
                let intent_url = self.generate_intent_url(txn);
                Ok(ProcessorResponse {
                    status: PaymentStatus::Processing,
                    gateway_reference: Some(upi_txn_id),
                    gateway_response: Some(serde_json::json!({"flow": "intent"})),
                    error_code: None,
                    error_message: None,
                    redirect_url: Some(intent_url),
                    metadata: Some(serde_json::json!({"upi_flow": "intent"})),
                })
            }
            "qr" => {
                let qr_string = self.generate_qr_string(txn);
                Ok(ProcessorResponse {
                    status: PaymentStatus::Processing,
                    gateway_reference: Some(upi_txn_id),
                    gateway_response: Some(serde_json::json!({
                        "flow": "qr",
                        "qr_data": qr_string,
                    })),
                    error_code: None,
                    error_message: None,
                    redirect_url: None,
                    metadata: Some(serde_json::json!({"upi_flow": "qr"})),
                })
            }
            _ => Err(FeelPayError::BadRequest(format!(
                "Unknown UPI flow: {}",
                flow_type
            ))),
        }
    }

    async fn capture(
        &self,
        _txn: &TransactionContext,
        _amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        // UPI auto-captures, no separate capture step
        Ok(ProcessorResponse {
            status: PaymentStatus::Captured,
            gateway_reference: None,
            gateway_response: Some(serde_json::json!({"note": "UPI auto-captured"})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }

    async fn void(&self, _txn: &TransactionContext) -> Result<ProcessorResponse, FeelPayError> {
        Err(FeelPayError::PaymentError(
            "UPI payments cannot be voided. Use refund instead.".into(),
        ))
    }

    async fn refund(
        &self,
        _txn: &TransactionContext,
        amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let refund_ref = format!("fp_upi_ref_{}", Uuid::new_v4().simple());
        Ok(ProcessorResponse {
            status: PaymentStatus::Refunded,
            gateway_reference: Some(refund_ref),
            gateway_response: Some(serde_json::json!({
                "refunded_amount": amount,
                "rrn": format!("{:012}", rand::random::<u64>() % 1_000_000_000_000),
            })),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }
}

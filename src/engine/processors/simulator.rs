use async_trait::async_trait;
use rand::Rng;
use uuid::Uuid;

use crate::engine::processor::{
    MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::error::FeelPayError;
use crate::models::payment_order::PaymentStatus;

/// Simulator processor — mimics real payment flows for testing
pub struct SimulatorProcessor {
    success_rate: f64,
}

impl Default for SimulatorProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulatorProcessor {
    pub fn new() -> Self {
        Self { success_rate: 0.9 }
    }

    fn should_succeed(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.success_rate
    }

    fn generate_ref(&self, prefix: &str) -> String {
        format!("sim_{}_{}", prefix, Uuid::new_v4().simple())
    }
}

#[async_trait]
impl PaymentMethodProcessor for SimulatorProcessor {
    fn method_type(&self) -> PaymentMethod {
        PaymentMethod::Simulator
    }

    async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        // Simulate processing delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if !self.should_succeed() {
            return Ok(ProcessorResponse {
                status: PaymentStatus::Failed,
                gateway_reference: Some(self.generate_ref("fail")),
                gateway_response: Some(serde_json::json!({"simulator": true})),
                error_code: Some("SIM_DECLINED".into()),
                error_message: Some("Simulated payment decline".into()),
                redirect_url: None,
                metadata: None,
            });
        }

        let (status, redirect) = match method_data {
            MethodData::Card { .. } => {
                // Cards go through authorize → capture flow
                (PaymentStatus::Authorized, None)
            }
            MethodData::Upi { flow, .. } => {
                let f = flow.as_deref().unwrap_or("collect");
                match f {
                    "intent" => (
                        PaymentStatus::Processing,
                        Some(format!(
                            "upi://pay?pa={}&pn=FeelPay&am={}&tr={}",
                            "merchant@feelpay",
                            txn.amount as f64 / 100.0,
                            txn.order_id
                        )),
                    ),
                    _ => (PaymentStatus::Captured, None), // collect auto-captures
                }
            }
            MethodData::NetBanking { .. } => (
                PaymentStatus::Processing,
                Some(format!(
                    "https://sim.feelpay.in/netbanking/{}",
                    txn.payment_id
                )),
            ),
            MethodData::Wallet { .. } => (PaymentStatus::Captured, None),
            MethodData::Emi { .. } => (PaymentStatus::Captured, None),
            MethodData::Bnpl { .. } => (PaymentStatus::Captured, None),
            MethodData::Simulator { .. } => (PaymentStatus::Captured, None),
        };

        Ok(ProcessorResponse {
            status,
            gateway_reference: Some(self.generate_ref("pay")),
            gateway_response: Some(serde_json::json!({
                "simulator": true,
                "method": format!("{:?}", method_data),
                "amount": txn.amount,
            })),
            error_code: None,
            error_message: None,
            redirect_url: redirect,
            metadata: None,
        })
    }

    async fn capture(
        &self,
        _txn: &TransactionContext,
        _amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Ok(ProcessorResponse {
            status: PaymentStatus::Captured,
            gateway_reference: Some(self.generate_ref("cap")),
            gateway_response: Some(serde_json::json!({"captured": true})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }

    async fn void(&self, _txn: &TransactionContext) -> Result<ProcessorResponse, FeelPayError> {
        Ok(ProcessorResponse {
            status: PaymentStatus::Voided,
            gateway_reference: Some(self.generate_ref("void")),
            gateway_response: Some(serde_json::json!({"voided": true})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }

    async fn refund(
        &self,
        _txn: &TransactionContext,
        amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Ok(ProcessorResponse {
            status: PaymentStatus::Refunded,
            gateway_reference: Some(self.generate_ref("ref")),
            gateway_response: Some(serde_json::json!({"refunded": true, "amount": amount})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }
}

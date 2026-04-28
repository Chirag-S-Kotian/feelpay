use async_trait::async_trait;

use crate::engine::processor::{
    MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::engine::processors::simulator::SimulatorProcessor;
use crate::error::FeelPayError;

/// Wallet processor backed by simulator until provider connector is added.
pub struct WalletProcessor {
    simulator: SimulatorProcessor,
}

impl Default for WalletProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletProcessor {
    pub fn new() -> Self {
        Self {
            simulator: SimulatorProcessor::new(),
        }
    }
}

#[async_trait]
impl PaymentMethodProcessor for WalletProcessor {
    fn method_type(&self) -> PaymentMethod {
        PaymentMethod::Wallet
    }

    async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        match method_data {
            MethodData::Wallet { wallet_name, .. } if !wallet_name.trim().is_empty() => {
                self.simulator.authorize(txn, method_data).await
            }
            MethodData::Wallet { .. } => Err(FeelPayError::BadRequest(
                "Wallet name required for wallet payments".into(),
            )),
            _ => Err(FeelPayError::BadRequest(
                "Expected wallet payment data".into(),
            )),
        }
    }

    async fn capture(
        &self,
        txn: &TransactionContext,
        amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        self.simulator.capture(txn, amount).await
    }

    async fn void(&self, txn: &TransactionContext) -> Result<ProcessorResponse, FeelPayError> {
        self.simulator.void(txn).await
    }

    async fn refund(
        &self,
        txn: &TransactionContext,
        amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        self.simulator.refund(txn, amount).await
    }
}

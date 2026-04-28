use async_trait::async_trait;

use crate::engine::processor::{
    MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::engine::processors::simulator::SimulatorProcessor;
use crate::error::FeelPayError;

/// NetBanking processor backed by simulator until provider connector is added.
pub struct NetBankingProcessor {
    simulator: SimulatorProcessor,
}

impl Default for NetBankingProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetBankingProcessor {
    pub fn new() -> Self {
        Self {
            simulator: SimulatorProcessor::new(),
        }
    }
}

#[async_trait]
impl PaymentMethodProcessor for NetBankingProcessor {
    fn method_type(&self) -> PaymentMethod {
        PaymentMethod::NetBanking
    }

    async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        match method_data {
            MethodData::NetBanking { bank_code } if !bank_code.trim().is_empty() => {
                self.simulator.authorize(txn, method_data).await
            }
            MethodData::NetBanking { .. } => Err(FeelPayError::BadRequest(
                "Bank code required for netbanking".into(),
            )),
            _ => Err(FeelPayError::BadRequest(
                "Expected netbanking payment data".into(),
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

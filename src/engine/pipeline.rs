use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::engine::processor::{
    MethodData, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::error::FeelPayError;

/// Transaction pipeline stages:
/// validate -> authorize (with controlled retry/backoff)
pub struct TransactionPipeline {
    max_retries: u8,
    initial_backoff_ms: u64,
}

impl Default for TransactionPipeline {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_backoff_ms: 50,
        }
    }
}

impl TransactionPipeline {
    pub fn validate(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<(), FeelPayError> {
        if txn.amount <= 0 {
            return Err(FeelPayError::BadRequest(
                "Amount must be greater than 0".into(),
            ));
        }
        if txn.currency.trim().len() != 3 {
            return Err(FeelPayError::BadRequest(
                "Currency must be 3-letter code".into(),
            ));
        }

        match method_data {
            MethodData::Card { number, token, .. } => {
                if number.is_none() && token.is_none() {
                    return Err(FeelPayError::BadRequest(
                        "Card method requires card number or token".into(),
                    ));
                }
            }
            MethodData::Upi { vpa, flow } => {
                if vpa.as_deref().unwrap_or_default().is_empty() {
                    return Err(FeelPayError::BadRequest("UPI method requires VPA".into()));
                }
                if let Some(f) = flow {
                    if f != "collect" && f != "intent" {
                        return Err(FeelPayError::BadRequest(
                            "UPI flow must be collect or intent".into(),
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn execute_authorize(
        &self,
        processor: &Arc<dyn PaymentMethodProcessor>,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let mut attempt: u8 = 0;
        loop {
            let result = processor.authorize(txn, method_data).await;
            match result {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    attempt += 1;
                    let is_retryable = matches!(
                        err,
                        FeelPayError::ConnectorError(_) | FeelPayError::Internal(_)
                    );
                    if !is_retryable || attempt > self.max_retries {
                        return Err(err);
                    }
                    let backoff = self.initial_backoff_ms * (1_u64 << (attempt - 1));
                    sleep(Duration::from_millis(backoff)).await;
                }
            }
        }
    }
}

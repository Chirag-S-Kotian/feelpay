use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::pipeline::TransactionPipeline;
use crate::engine::processor::{
    MethodData, PaymentMethodProcessor, ProcessorResponse, TransactionContext,
};
use crate::error::FeelPayError;

/// Payment engine — routes transactions to appropriate processors
pub struct PaymentEngine {
    processors: HashMap<String, Arc<dyn PaymentMethodProcessor>>,
    default_processor: String,
    pipeline: TransactionPipeline,
}

impl Default for PaymentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentEngine {
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
            default_processor: "simulator".to_string(),
            pipeline: TransactionPipeline::default(),
        }
    }

    /// Register a processor
    pub fn register_processor(&mut self, name: &str, processor: Arc<dyn PaymentMethodProcessor>) {
        self.processors.insert(name.to_string(), processor);
        tracing::info!("Registered processor: {}", name);
    }

    /// Set default processor
    pub fn set_default_processor(&mut self, name: &str) {
        self.default_processor = name.to_string();
    }

    /// Get processor by name
    fn get_processor(&self, name: &str) -> Result<&Arc<dyn PaymentMethodProcessor>, FeelPayError> {
        self.processors.get(name).ok_or_else(|| {
            FeelPayError::ConnectorError(format!("Processor '{}' not registered", name))
        })
    }

    /// Resolve processor from method data
    fn resolve_processor(&self, method_data: &MethodData) -> String {
        match method_data {
            MethodData::Card { .. } => {
                if self.processors.contains_key("card") {
                    "card".to_string()
                } else {
                    self.default_processor.clone()
                }
            }
            MethodData::Upi { .. } => {
                if self.processors.contains_key("upi") {
                    "upi".to_string()
                } else {
                    self.default_processor.clone()
                }
            }
            _ => self.default_processor.clone(),
        }
    }

    /// Process a payment authorization
    pub async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
        processor_override: Option<&str>,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let proc_name = processor_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.resolve_processor(method_data));

        let processor = self.get_processor(&proc_name)?;
        self.pipeline.validate(txn, method_data)?;

        tracing::info!(
            order_id = %txn.order_id,
            processor = %proc_name,
            amount = txn.amount,
            "Authorizing payment"
        );

        let result = self
            .pipeline
            .execute_authorize(processor, txn, method_data)
            .await;

        match &result {
            Ok(resp) => {
                tracing::info!(
                    order_id = %txn.order_id,
                    status = ?resp.status,
                    gateway_ref = ?resp.gateway_reference,
                    "Payment authorized"
                );
            }
            Err(e) => {
                tracing::error!(order_id = %txn.order_id, error = %e, "Authorization failed");
            }
        }

        result
    }

    /// Capture payment
    pub async fn capture(
        &self,
        txn: &TransactionContext,
        amount: i64,
        processor_name: &str,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let processor = self.get_processor(processor_name)?;
        tracing::info!(order_id = %txn.order_id, processor = %processor_name, "Capturing payment");
        processor.capture(txn, amount).await
    }

    /// Void payment
    pub async fn void_payment(
        &self,
        txn: &TransactionContext,
        processor_name: &str,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let processor = self.get_processor(processor_name)?;
        processor.void(txn).await
    }

    /// Refund payment
    pub async fn refund(
        &self,
        txn: &TransactionContext,
        amount: i64,
        processor_name: &str,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let processor = self.get_processor(processor_name)?;
        tracing::info!(order_id = %txn.order_id, amount = amount, "Processing refund");
        processor.refund(txn, amount).await
    }
}

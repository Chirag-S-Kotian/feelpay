use async_trait::async_trait;
use uuid::Uuid;

use crate::engine::processor::{
    MethodData, PaymentMethodProcessor, ProcessorResponse, TransactionContext, PaymentMethod,
};
use crate::error::FeelPayError;
use crate::models::payment_order::PaymentStatus;

/// Card processor — handles card payments via internal vault
pub struct CardProcessor;

impl CardProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Validate card number using Luhn algorithm
    pub fn luhn_check(number: &str) -> bool {
        let digits: Vec<u32> = number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let sum: u32 = digits
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &d)| {
                if i % 2 == 1 {
                    let doubled = d * 2;
                    if doubled > 9 { doubled - 9 } else { doubled }
                } else {
                    d
                }
            })
            .sum();

        sum % 10 == 0
    }

    /// Detect card brand from BIN (first 6 digits)
    pub fn detect_brand(number: &str) -> &'static str {
        let clean: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean.is_empty() {
            return "unknown";
        }

        let first = clean.chars().next().unwrap();
        let first_two: u32 = clean[..2.min(clean.len())].parse().unwrap_or(0);

        match first {
            '4' => "visa",
            '5' if (51..=55).contains(&first_two) => "mastercard",
            '2' if clean.len() >= 4 => {
                let first_four: u32 = clean[..4].parse().unwrap_or(0);
                if (2221..=2720).contains(&first_four) {
                    "mastercard"
                } else {
                    "unknown"
                }
            }
            '3' if first_two == 34 || first_two == 37 => "amex",
            '6' if first_two == 60 || first_two == 65 => "rupay",
            '8' if first_two == 81 || first_two == 82 => "rupay",
            _ => "unknown",
        }
    }

    /// Extract last 4 digits
    pub fn last_four(number: &str) -> String {
        let clean: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean.len() >= 4 {
            clean[clean.len() - 4..].to_string()
        } else {
            clean
        }
    }
}

#[async_trait]
impl PaymentMethodProcessor for CardProcessor {
    fn method_type(&self) -> PaymentMethod {
        PaymentMethod::Card
    }

    async fn authorize(
        &self,
        txn: &TransactionContext,
        method_data: &MethodData,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let (number, token) = match method_data {
            MethodData::Card { number, token, .. } => (number.clone(), token.clone()),
            _ => return Err(FeelPayError::BadRequest("Expected card payment data".into())),
        };

        // Must have either card number or token
        if number.is_none() && token.is_none() {
            return Err(FeelPayError::BadRequest(
                "Card number or token required".into(),
            ));
        }

        // If raw card number, validate
        if let Some(ref num) = number {
            if !Self::luhn_check(num) {
                return Err(FeelPayError::PaymentError("Invalid card number".into()));
            }
        }

        let brand = number
            .as_deref()
            .map(Self::detect_brand)
            .unwrap_or("tokenized");
        let last4 = number
            .as_deref()
            .map(Self::last_four)
            .unwrap_or_else(|| "****".into());

        let gateway_ref = format!("fp_card_{}", Uuid::new_v4().simple());

        Ok(ProcessorResponse {
            status: PaymentStatus::Authorized,
            gateway_reference: Some(gateway_ref),
            gateway_response: Some(serde_json::json!({
                "card_brand": brand,
                "last_four": last4,
                "amount": txn.amount,
                "currency": txn.currency,
            })),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: Some(serde_json::json!({
                "card_brand": brand,
                "last_four": last4,
            })),
        })
    }

    async fn capture(
        &self,
        _txn: &TransactionContext,
        amount: i64,
    ) -> Result<ProcessorResponse, FeelPayError> {
        let gateway_ref = format!("fp_cap_{}", Uuid::new_v4().simple());
        Ok(ProcessorResponse {
            status: PaymentStatus::Captured,
            gateway_reference: Some(gateway_ref),
            gateway_response: Some(serde_json::json!({"captured_amount": amount})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }

    async fn void(
        &self,
        _txn: &TransactionContext,
    ) -> Result<ProcessorResponse, FeelPayError> {
        Ok(ProcessorResponse {
            status: PaymentStatus::Voided,
            gateway_reference: Some(format!("fp_void_{}", Uuid::new_v4().simple())),
            gateway_response: None,
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
        Ok(ProcessorResponse {
            status: PaymentStatus::Refunded,
            gateway_reference: Some(format!("fp_ref_{}", Uuid::new_v4().simple())),
            gateway_response: Some(serde_json::json!({"refunded_amount": amount})),
            error_code: None,
            error_message: None,
            redirect_url: None,
            metadata: None,
        })
    }

}

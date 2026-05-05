//! Metadata redaction utility
//!
//! Strips sensitive fields from raw provider responses before storing in DB or logs.

use serde_json::Value;

/// Fields that MUST be redacted from provider responses
const REDACT_FIELDS: &[&str] = &[
    "card_number",
    "cvv",
    "cvc",
    "pan",
    "card_pan",
    "full_card_number",
    "card_security_code",
    "security_code",
    "expiry",
    "exp_month",
    "exp_year",
    "account_number",
    "routing_number",
    "iban",
    "swift",
    "bic",
    "ssn",
    "tax_id",
    "password",
    "secret",
    "api_key",
    "api_secret",
    "access_token",
    "refresh_token",
    "private_key",
    "client_secret",
];

/// Recursively redact sensitive fields from a JSON value
pub fn redact_metadata(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, val) in map {
                let lower_key = key.to_lowercase();
                if REDACT_FIELDS.iter().any(|f| lower_key.contains(f)) {
                    redacted.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                } else {
                    redacted.insert(key.clone(), redact_metadata(val));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(redact_metadata).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_card_fields() {
        let input = json!({
            "id": "pi_123",
            "card_number": "4111111111111111",
            "cvv": "123",
            "amount": 5000,
            "customer": {
                "email": "test@example.com",
                "api_key": "sk_test_xxx"
            }
        });

        let result = redact_metadata(&input);
        assert_eq!(result["card_number"], "[REDACTED]");
        assert_eq!(result["cvv"], "[REDACTED]");
        assert_eq!(result["amount"], 5000);
        assert_eq!(result["id"], "pi_123");
        assert_eq!(result["customer"]["email"], "test@example.com");
        assert_eq!(result["customer"]["api_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_nested_arrays() {
        let input = json!({
            "items": [
                { "card_pan": "4111", "price": 100 },
                { "name": "ok" }
            ]
        });

        let result = redact_metadata(&input);
        assert_eq!(result["items"][0]["card_pan"], "[REDACTED]");
        assert_eq!(result["items"][0]["price"], 100);
        assert_eq!(result["items"][1]["name"], "ok");
    }

    #[test]
    fn test_redact_preserves_primitives() {
        let input = json!("hello");
        assert_eq!(redact_metadata(&input), json!("hello"));

        let input = json!(42);
        assert_eq!(redact_metadata(&input), json!(42));
    }
}

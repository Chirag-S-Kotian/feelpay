use serde::Serialize;

/// Card brand detection and BIN (Bank Identification Number) lookup.
/// Real production systems call an external BIN database or NPCI's BIN API.
#[derive(Debug, Clone, Serialize)]
pub struct BinInfo {
    pub brand: String,
    pub card_type: String,   // debit | credit | prepaid
    pub issuer: String,
    pub country: String,
    pub last_four: String,
}

pub struct BinLookup;

impl BinLookup {
    /// Detect card brand from PAN prefix (IIN ranges).
    pub fn detect_brand(pan: &str) -> &'static str {
        let digits: Vec<u8> = pan.chars().filter_map(|c| c.to_digit(10).map(|d| d as u8)).collect();
        if digits.is_empty() {
            return "unknown";
        }
        // Visa
        if digits[0] == 4 {
            return "visa";
        }
        // Mastercard: 51-55, 2221-2720
        if digits.len() >= 2 {
            let two = digits[0] * 10 + digits[1];
            if (51..=55).contains(&two) {
                return "mastercard";
            }
        }
        if digits.len() >= 4 {
            let four = (digits[0] as u16) * 1000
                + (digits[1] as u16) * 100
                + (digits[2] as u16) * 10
                + digits[3] as u16;
            if (2221..=2720).contains(&four) {
                return "mastercard";
            }
        }
        // Amex: 34, 37
        if digits.len() >= 2 {
            let two = digits[0] * 10 + digits[1];
            if two == 34 || two == 37 {
                return "amex";
            }
        }
        // RuPay (India domestic): 60, 65, 81, 82, 508
        if digits.len() >= 2 {
            let two = digits[0] * 10 + digits[1];
            if two == 60 || two == 65 || two == 81 || two == 82 {
                return "rupay";
            }
        }
        if digits.len() >= 3 {
            let three = (digits[0] as u16) * 100 + (digits[1] as u16) * 10 + digits[2] as u16;
            if three == 508 {
                return "rupay";
            }
        }
        // Discover
        if digits.len() >= 4 {
            let four = (digits[0] as u16) * 1000
                + (digits[1] as u16) * 100
                + (digits[2] as u16) * 10
                + digits[3] as u16;
            if four == 6011 {
                return "discover";
            }
        }
        "unknown"
    }

    /// Extract last 4 digits.
    pub fn last_four(pan: &str) -> String {
        let digits: String = pan.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            digits[digits.len() - 4..].to_string()
        } else {
            digits
        }
    }

    /// Basic Luhn validation.
    pub fn luhn_valid(pan: &str) -> bool {
        let digits: Vec<u32> = pan
            .chars()
            .rev()
            .filter_map(|c| c.to_digit(10))
            .collect();
        if digits.len() < 13 {
            return false;
        }
        let sum: u32 = digits
            .iter()
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

    /// Build BinInfo from PAN (offline heuristic). Production would call BIN API.
    pub fn lookup(pan: &str) -> BinInfo {
        BinInfo {
            brand: Self::detect_brand(pan).to_string(),
            card_type: "credit".to_string(), // placeholder — real lookup needed
            issuer: "Unknown Issuer".to_string(),
            country: "IN".to_string(),
            last_four: Self::last_four(pan),
        }
    }
}

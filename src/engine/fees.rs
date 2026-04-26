use sqlx::{PgPool, Row};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::error::FeelPayError;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FeeCalculation {
    pub gross_amount: i64,
    pub fee_amount: i64,
    pub gst_amount: i64,
    pub net_amount: i64,
    pub fee_rule_id: Uuid,
}

pub async fn calculate_fees(
    pool: &PgPool,
    merchant_id: Uuid,
    amount: i64,
    payment_method: &str,
    payment_network: Option<&str>,
) -> Result<FeeCalculation, FeelPayError> {
    // 1. Fetch the most specific fee rule
    // Priority: Merchant specific + Method specific > Global + Method specific > Global default
    let row = sqlx::query(
        r#"
        SELECT id, percentage_fee, flat_fee, min_fee, max_fee, gst_inclusive
        FROM fee_rules
        WHERE is_active = true
        AND (merchant_id = $1 OR merchant_id IS NULL)
        AND (payment_method = $2 OR payment_method IS NULL)
        AND (payment_network = $3 OR payment_network IS NULL)
        ORDER BY merchant_id DESC NULLS LAST, payment_method DESC NULLS LAST, payment_network DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(merchant_id)
    .bind(payment_method)
    .bind(payment_network)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(FeelPayError::BadRequest("No applicable fee rule found".into())),
    };

    let id: Uuid = row.try_get("id").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;
    let percentage_fee_val: Option<Decimal> = row.try_get("percentage_fee").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;
    let flat_fee_val: Option<i64> = row.try_get("flat_fee").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;
    let min_fee_val: Option<i64> = row.try_get("min_fee").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;
    let max_fee_val: Option<i64> = row.try_get("max_fee").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;
    let gst_inclusive_val: Option<bool> = row.try_get("gst_inclusive").map_err(|e| FeelPayError::DatabaseError(e.to_string()))?;

    let amount_dec = Decimal::from_i64(amount).unwrap();
    let percentage_fee = percentage_fee_val.unwrap_or(Decimal::ZERO);
    let flat_fee = Decimal::from_i64(flat_fee_val.unwrap_or(0)).unwrap();

    // Base fee = (amount * percentage / 100) + flat_fee
    let mut fee = (amount_dec * percentage_fee / Decimal::from(100)) + flat_fee;

    // Apply caps
    if let Some(min_fee) = min_fee_val {
        let min_dec = Decimal::from_i64(min_fee).unwrap();
        if fee < min_dec {
            fee = min_dec;
        }
    }
    if let Some(max_fee) = max_fee_val {
        let max_dec = Decimal::from_i64(max_fee).unwrap();
        if fee > max_dec {
            fee = max_dec;
        }
    }

    let gst_rate = Decimal::from_str("0.18").unwrap(); // 18% GST standard in India for services
    let (fee, gst_amount) = if !gst_inclusive_val.unwrap_or(true) {
        (fee, fee * gst_rate)
    } else {
        // If inclusive, fee already contains GST.
        // GST = total_fee - (total_fee / 1.18)
        let total_fee = fee;
        let base_fee = total_fee / (Decimal::ONE + gst_rate);
        (base_fee, total_fee - base_fee)
    };

    let fee_rounded = fee.to_i64().unwrap_or(0);
    let gst_rounded = gst_amount.to_i64().unwrap_or(0);
    let net_amount = amount - fee_rounded - gst_rounded;

    Ok(FeeCalculation {
        gross_amount: amount,
        fee_amount: fee_rounded,
        gst_amount: gst_rounded,
        net_amount,
        fee_rule_id: id,
    })
}

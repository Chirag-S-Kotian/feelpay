use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub provider: SmsProvider,
    pub api_key: String,
    pub sender_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmsProvider {
    Msg91,
    Twilio,
    Simulator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub to: String,
    pub body: String,
    pub template_id: Option<String>,
    pub variables: Option<serde_json::Value>,
}

/// Send SMS via configured provider
pub async fn send_sms(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    to: &str,
    body: &str,
    template_id: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get SMS config from environment or database
    let provider = std::env::var("SMS_PROVIDER").unwrap_or_else(|_| "simulator".to_string());

    let message_id = Uuid::new_v4().to_string();

    // Log SMS to notification queue
    sqlx::query(
        r#"
        INSERT INTO notification_queue (id, merchant_id, type, title, message, status, data)
        VALUES ($1, $2, 'sms', 'SMS Sent', $3, 'sent', $4)
        "#,
    )
    .bind(Uuid::parse_str(&message_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(merchant_id)
    .bind(body)
    .bind(serde_json::json!({
        "to": to,
        "template_id": template_id,
        "provider": provider
    }))
    .execute(pool)
    .await?;

    match provider.as_str() {
        "msg91" => send_via_msg91(to, body, template_id).await?,
        "twilio" => send_via_twilio(to, body).await?,
        _ => {
            // Simulator mode - just log
            tracing::info!(
                to = %to,
                body = %body,
                "[SIMULATOR] SMS sent"
            );
        }
    }

    Ok(message_id)
}

/// Send OTP via SMS
pub async fn send_otp_sms(
    pool: &PgPool,
    to: &str,
    otp: &str,
    purpose: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let body = format!(
        "Your FeelPay {} OTP is {}. Valid for 5 minutes. Do not share.",
        purpose, otp
    );

    send_sms(pool, None, to, &body, Some("otp_template")).await
}

/// Send payment confirmation SMS
pub async fn send_payment_sms(
    pool: &PgPool,
    merchant_id: Uuid,
    to: &str,
    amount: i64,
    order_id: &str,
    status: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let amount_formatted = format_amount(amount);
    let body = match status {
        "captured" | "success" => format!(
            "Payment of {} received for order {}. Thank you!",
            amount_formatted, order_id
        ),
        "failed" => format!(
            "Payment of {} failed for order {}. Please try again.",
            amount_formatted, order_id
        ),
        _ => format!(
            "Payment of {} for order {} is {}.",
            amount_formatted, order_id, status
        ),
    };

    send_sms(pool, Some(merchant_id), to, &body, Some("payment_template")).await
}

/// Send settlement notification SMS
pub async fn send_settlement_sms(
    pool: &PgPool,
    merchant_id: Uuid,
    to: &str,
    amount: i64,
    utr: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let amount_formatted = format_amount(amount);
    let body = if let Some(utr) = utr {
        format!("Settlement of {} credited. UTR: {}", amount_formatted, utr)
    } else {
        format!(
            "Settlement of {} will be credited within 24 hours.",
            amount_formatted
        )
    };

    send_sms(
        pool,
        Some(merchant_id),
        to,
        &body,
        Some("settlement_template"),
    )
    .await
}

/// Send via MSG91
async fn send_via_msg91(
    to: &str,
    body: &str,
    template_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("MSG91_API_KEY").map_err(|_| "MSG91_API_KEY not configured")?;
    let sender_id = std::env::var("MSG91_SENDER_ID").unwrap_or_else(|_| "FEELPY".to_string());

    let client = reqwest::Client::new();

    let mut form = vec![("mobile", to), ("message", body), ("sender", &sender_id)];

    if let Some(tid) = template_id {
        form.push(("template_id", tid));
    }

    let response = client
        .post("https://api.msg91.com/api/v2/sendsms")
        .header("authkey", &api_key)
        .form(&form)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        tracing::error!("MSG91 error: {}", error);
        return Err(format!("MSG91 error: {}", error).into());
    }

    Ok(())
}

/// Send via Twilio
async fn send_via_twilio(to: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let account_sid =
        std::env::var("TWILIO_ACCOUNT_SID").map_err(|_| "TWILIO_ACCOUNT_SID not configured")?;
    let auth_token =
        std::env::var("TWILIO_AUTH_TOKEN").map_err(|_| "TWILIO_AUTH_TOKEN not configured")?;
    let from_number =
        std::env::var("TWILIO_FROM_NUMBER").map_err(|_| "TWILIO_FROM_NUMBER not configured")?;

    let client = reqwest::Client::new();

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[("To", to), ("From", &from_number), ("Body", body)])
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        tracing::error!("Twilio error: {}", error);
        return Err(format!("Twilio error: {}", error).into());
    }

    Ok(())
}

/// Format amount in INR
fn format_amount(paise: i64) -> String {
    let rupees = paise as f64 / 100.0;
    format!("₹{:.2}", rupees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(100), "₹1.00");
        assert_eq!(format_amount(12345), "₹123.45");
        assert_eq!(format_amount(100000), "₹1000.00");
    }
}

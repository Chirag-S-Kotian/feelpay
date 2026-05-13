use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub provider: EmailProvider,
    pub api_key: String,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailProvider {
    Sendgrid,
    Ses,
    Postmark,
    Simulator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub template_id: Option<String>,
    pub template_data: Option<serde_json::Value>,
}

/// Send email via configured provider
pub async fn send_email(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    to: &str,
    subject: &str,
    body_html: Option<&str>,
    body_text: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let provider = std::env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "simulator".to_string());

    let message_id = Uuid::new_v4().to_string();

    // Log email to notification queue
    sqlx::query(
        r#"
        INSERT INTO notification_queue (id, merchant_id, type, title, message, status, data)
        VALUES ($1, $2, 'email', $3, $4, 'sent', $5)
        "#,
    )
    .bind(Uuid::parse_str(&message_id).unwrap_or_else(|_| Uuid::new_v4()))
    .bind(merchant_id)
    .bind(subject)
    .bind(body_text.unwrap_or(""))
    .bind(serde_json::json!({
        "to": to,
        "provider": provider,
        "has_html": body_html.is_some()
    }))
    .execute(pool)
    .await?;

    match provider.as_str() {
        "sendgrid" => send_via_sendgrid(to, subject, body_html, body_text).await?,
        "ses" => send_via_ses(to, subject, body_html, body_text).await?,
        "postmark" => send_via_postmark(to, subject, body_html, body_text).await?,
        _ => {
            // Simulator mode - just log
            tracing::info!(
                to = %to,
                subject = %subject,
                "[SIMULATOR] Email sent"
            );
        }
    }

    Ok(message_id)
}

/// Send OTP email
pub async fn send_otp_email(
    pool: &PgPool,
    to: &str,
    otp: &str,
    purpose: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let subject = format!("Your FeelPay {} OTP", purpose);
    let body_html = format!(
        r#"
        <html>
        <body style="font-family: Arial, sans-serif;">
            <h2>FeelPay Verification</h2>
            <p>Your OTP for {} is: <strong style="font-size: 24px;">{}</strong></p>
            <p>This code is valid for 5 minutes. Do not share this code.</p>
            <p>If you did not request this code, please ignore this email.</p>
        </body>
        </html>
        "#,
        purpose, otp
    );
    let body_text = format!(
        "Your FeelPay {} OTP is: {}\n\nThis code is valid for 5 minutes. Do not share this code.",
        purpose, otp
    );

    send_email(pool, None, to, &subject, Some(&body_html), Some(&body_text)).await
}

/// Send payment receipt email
pub async fn send_payment_receipt_email(
    pool: &PgPool,
    merchant_id: Uuid,
    to: &str,
    order_id: &str,
    amount: i64,
    currency: &str,
    status: &str,
    payment_method: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let amount_formatted = format_amount(amount, currency);
    let subject = format!("Payment Receipt - Order {}", order_id);

    let body_html = format!(
        r#"
        <html>
        <body style="font-family: Arial, sans-serif;">
            <h2>Payment Receipt</h2>
            <table style="width: 100%; border-collapse: collapse;">
                <tr><td style="padding: 8px; border-bottom: 1px solid #eee;"><strong>Order ID:</strong></td><td style="padding: 8px; border-bottom: 1px solid #eee;">{}</td></tr>
                <tr><td style="padding: 8px; border-bottom: 1px solid #eee;"><strong>Amount:</strong></td><td style="padding: 8px; border-bottom: 1px solid #eee;">{}</td></tr>
                <tr><td style="padding: 8px; border-bottom: 1px solid #eee;"><strong>Status:</strong></td><td style="padding: 8px; border-bottom: 1px solid #eee;">{}</td></tr>
                <tr><td style="padding: 8px; border-bottom: 1px solid #eee;"><strong>Payment Method:</strong></td><td style="padding: 8px; border-bottom: 1px solid #eee;">{}</td></tr>
            </table>
            <p>Thank you for your payment!</p>
        </body>
        </html>
        "#,
        order_id,
        amount_formatted,
        status.to_uppercase(),
        payment_method.unwrap_or("N/A")
    );

    let body_text = format!(
        "Payment Receipt\n\nOrder ID: {}\nAmount: {}\nStatus: {}\nPayment Method: {}\n\nThank you for your payment!",
        order_id,
        amount_formatted,
        status.to_uppercase(),
        payment_method.unwrap_or("N/A")
    );

    send_email(
        pool,
        Some(merchant_id),
        to,
        &subject,
        Some(&body_html),
        Some(&body_text),
    )
    .await
}

/// Send settlement notification email
pub async fn send_settlement_email(
    pool: &PgPool,
    merchant_id: Uuid,
    to: &str,
    settlement_id: Uuid,
    amount: i64,
    utr: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let amount_formatted = format_amount(amount, "INR");
    let subject = "Settlement Processed".to_string();

    let body_html = format!(
        r#"
        <html>
        <body style="font-family: Arial, sans-serif;">
            <h2>Settlement Notification</h2>
            <p>Your settlement of <strong>{}</strong> has been processed.</p>
            <p><strong>Settlement ID:</strong> {}</p>
            <p><strong>UTR:</strong> {}</p>
            <p>The amount will be credited to your bank account within 24 hours.</p>
        </body>
        </html>
        "#,
        amount_formatted,
        settlement_id,
        utr.unwrap_or("Pending")
    );

    let body_text = format!(
        "Settlement Notification\n\nYour settlement of {} has been processed.\nSettlement ID: {}\nUTR: {}\n\nThe amount will be credited to your bank account within 24 hours.",
        amount_formatted,
        settlement_id,
        utr.unwrap_or("Pending")
    );

    send_email(
        pool,
        Some(merchant_id),
        to,
        &subject,
        Some(&body_html),
        Some(&body_text),
    )
    .await
}

/// Send KYC status email
pub async fn send_kyc_status_email(
    pool: &PgPool,
    to: &str,
    status: &str,
    message: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let subject = match status {
        "approved" => "KYC Approved - Account Activated",
        "rejected" => "KYC Verification Update",
        "resubmit_required" => "KYC Documents Require Attention",
        _ => "KYC Status Update",
    };

    let body_html = match status {
        "approved" => format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h2>🎉 KYC Approved!</h2>
                <p>Congratulations! Your KYC verification has been approved.</p>
                <p>Your account is now fully activated. You can start accepting payments.</p>
                <p><a href="https://dashboard.feelpay.in">Go to Dashboard</a></p>
            </body>
            </html>
            "#
        ),
        "rejected" => format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h2>KYC Verification Update</h2>
                <p>Unfortunately, your KYC verification could not be completed.</p>
                <p><strong>Reason:</strong> {}</p>
                <p>Please contact support for assistance.</p>
            </body>
            </html>
            "#,
            message.unwrap_or("Please contact support for details.")
        ),
        _ => format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif;">
                <h2>KYC Status Update</h2>
                <p>Your KYC status has been updated to: {}</p>
                <p>{}</p>
            </body>
            </html>
            "#,
            status,
            message.unwrap_or("")
        ),
    };

    let body_text = format!("KYC Status: {}\n\n{}", status, message.unwrap_or(""));

    send_email(pool, None, to, subject, Some(&body_html), Some(&body_text)).await
}

/// Send via SendGrid
async fn send_via_sendgrid(
    to: &str,
    subject: &str,
    body_html: Option<&str>,
    body_text: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("SENDGRID_API_KEY").map_err(|_| "SENDGRID_API_KEY not configured")?;
    let from =
        std::env::var("EMAIL_FROM_ADDRESS").unwrap_or_else(|_| "noreply@feelpay.in".to_string());

    let client = reqwest::Client::new();

    let mut content = vec![];
    if let Some(html) = body_html {
        content.push(serde_json::json!({
            "type": "text/html",
            "value": html
        }));
    }
    if let Some(text) = body_text {
        content.push(serde_json::json!({
            "type": "text/plain",
            "value": text
        }));
    }

    let body = serde_json::json!({
        "personalizations": [{
            "to": [{"email": to}]
        }],
        "from": {"email": from},
        "subject": subject,
        "content": content
    });

    let response = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        tracing::error!("SendGrid error: {}", error);
        return Err(format!("SendGrid error: {}", error).into());
    }

    Ok(())
}

/// Send via AWS SES
async fn send_via_ses(
    to: &str,
    subject: &str,
    _body_html: Option<&str>,
    _body_text: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // SES would require AWS SDK - for now, log
    tracing::info!(
        to = %to,
        subject = %subject,
        "[SES] Email would be sent"
    );
    Ok(())
}

/// Send via Postmark
async fn send_via_postmark(
    to: &str,
    subject: &str,
    body_html: Option<&str>,
    body_text: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("POSTMARK_API_KEY").map_err(|_| "POSTMARK_API_KEY not configured")?;
    let from =
        std::env::var("EMAIL_FROM_ADDRESS").unwrap_or_else(|_| "noreply@feelpay.in".to_string());

    let client = reqwest::Client::new();

    let mut body = serde_json::json!({
        "From": from,
        "To": to,
        "Subject": subject,
    });

    if let Some(html) = body_html {
        body["HtmlBody"] = serde_json::json!(html);
    }
    if let Some(text) = body_text {
        body["TextBody"] = serde_json::json!(text);
    }

    let response = client
        .post("https://api.postmarkapp.com/email")
        .header("X-Postmark-Server-Token", &api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        tracing::error!("Postmark error: {}", error);
        return Err(format!("Postmark error: {}", error).into());
    }

    Ok(())
}

/// Format amount with currency
fn format_amount(paise: i64, currency: &str) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let absolute = paise.abs();
    let amount = format!("{}{}.{:02}", sign, absolute / 100, absolute % 100);
    match currency {
        "INR" => format!("₹{}", amount),
        "USD" => format!("${}", amount),
        _ => format!("{} {}", amount, currency),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_amount_inr() {
        assert_eq!(format_amount(100, "INR"), "₹1.00");
        assert_eq!(format_amount(12345, "INR"), "₹123.45");
    }

    #[test]
    fn test_format_amount_usd() {
        assert_eq!(format_amount(100, "USD"), "$1.00");
        assert_eq!(format_amount(12345, "USD"), "$123.45");
    }
}

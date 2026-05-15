use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::Row;
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(provider) = load_default_generic_sms_provider(pool).await? {
        let attempt = send_via_generic_provider(&provider, to, body, template_id).await?;
        return log_sms(
            pool,
            merchant_id,
            to,
            body,
            template_id,
            "integration_center",
            Some(&attempt),
        )
        .await;
    }

    // Legacy provider path remains as fallback when no Integration Center default exists.
    let provider = std::env::var("SMS_PROVIDER").unwrap_or_else(|_| "simulator".to_string());

    match provider.as_str() {
        "msg91" => send_via_msg91(to, body, template_id).await?,
        "twilio" => send_via_twilio(to, body).await?,
        _ => {
            // Simulator mode - just log
            tracing::info!(
                to = %mask_phone(to),
                "[SIMULATOR] SMS sent"
            );
        }
    }

    log_sms(pool, merchant_id, to, body, template_id, &provider, None).await
}

/// Send OTP via SMS
pub async fn send_otp_sms(
    pool: &PgPool,
    to: &str,
    otp: &str,
    purpose: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        tracing::error!(status = %response.status(), "MSG91 request failed");
        return Err(format!("MSG91 returned {}", response.status()).into());
    }

    Ok(())
}

/// Send via Twilio
async fn send_via_twilio(
    to: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        tracing::error!(status = %response.status(), "Twilio request failed");
        return Err(format!("Twilio returned {}", response.status()).into());
    }

    Ok(())
}

#[derive(Debug)]
struct GenericSmsProvider {
    provider_code: String,
    base_url: String,
    method: String,
    headers_template: serde_json::Map<String, Value>,
    body_template: Value,
    sender_id: String,
    route_type: Option<String>,
    template_mapping: serde_json::Map<String, Value>,
    auth_token: String,
    account_sid: Option<String>,
    from_number: Option<String>,
}

async fn load_default_generic_sms_provider(
    pool: &PgPool,
) -> Result<Option<GenericSmsProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let row = sqlx::query(
        r#"
        SELECT p.id, p.config, s.encrypted_value, s.nonce
        FROM integration_providers p
        JOIN integration_provider_secrets s ON s.provider_id = p.id AND s.secret_name = 'auth_token'
        WHERE p.provider_kind = 'sms'
          AND p.status = 'enabled'
          AND p.is_default = true
        ORDER BY CASE WHEN p.environment = 'live' THEN 0 ELSE 1 END, p.priority, p.name
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let config: Value = row.get("config");
    let auth_token = decrypt_integration_secret(
        row.get::<String, _>("encrypted_value").as_str(),
        row.get::<String, _>("nonce").as_str(),
    )?;
    let provider_code = config
        .get("provider_code")
        .and_then(Value::as_str)
        .unwrap_or("generic_http")
        .to_lowercase();
    Ok(Some(GenericSmsProvider {
        provider_code: provider_code.clone(),
        base_url: if provider_code == "twilio" {
            String::new()
        } else {
            required_config(&config, "sms_base_url")?
        },
        method: if provider_code == "twilio" {
            "POST".to_string()
        } else {
            required_config(&config, "sms_http_method")?.to_uppercase()
        },
        headers_template: if provider_code == "twilio" {
            serde_json::Map::new()
        } else {
            config
                .get("sms_headers_template")
                .and_then(Value::as_object)
                .cloned()
                .ok_or("Missing sms_headers_template")?
        },
        body_template: if provider_code == "twilio" {
            serde_json::json!({})
        } else {
            config
                .get("sms_body_template")
                .cloned()
                .ok_or("Missing sms_body_template")?
        },
        sender_id: if provider_code == "twilio" {
            config
                .get("sender_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        } else {
            required_config(&config, "sender_id")?
        },
        route_type: config
            .get("route_type")
            .and_then(Value::as_str)
            .map(str::to_string),
        template_mapping: config
            .get("template_mapping")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default(),
        auth_token,
        account_sid: config
            .get("account_sid")
            .and_then(Value::as_str)
            .map(str::to_string),
        from_number: config
            .get("from_number")
            .and_then(Value::as_str)
            .map(str::to_string),
    }))
}

fn required_config(
    config: &Value,
    key: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Missing {key}").into())
}

fn decrypt_integration_secret(
    encrypted_value: &str,
    nonce: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    let raw_key = std::env::var("VAULT_ENCRYPTION_KEY")?;
    let key = hex::decode(raw_key.trim())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes")?;
    let encrypted = BASE64.decode(encrypted_value)?;
    let nonce = BASE64.decode(nonce)?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
        .map_err(|_| "Credential decryption failed")?;
    Ok(String::from_utf8(decrypted)?)
}

fn render_sms_template(
    value: &Value,
    phone: &str,
    message: &str,
    sender_id: &str,
    route_type: Option<&str>,
    template_id: Option<&str>,
    auth_token: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(match value {
        Value::String(input) => {
            let rendered = input
                .replace("{{phone}}", phone)
                .replace("{{message}}", message)
                .replace("{{sender_id}}", sender_id)
                .replace("{{route_type}}", route_type.unwrap_or_default())
                .replace("{{template_id}}", template_id.unwrap_or_default())
                .replace("{{token}}", auth_token);
            if rendered.contains("{{") || rendered.contains("}}") {
                return Err("Unsupported SMS template placeholder".into());
            }
            Value::String(rendered)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| {
                    render_sms_template(
                        item,
                        phone,
                        message,
                        sender_id,
                        route_type,
                        template_id,
                        auth_token,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    Ok((
                        key.clone(),
                        render_sms_template(
                            value,
                            phone,
                            message,
                            sender_id,
                            route_type,
                            template_id,
                            auth_token,
                        )?,
                    ))
                })
                .collect::<Result<serde_json::Map<_, _>, Box<dyn std::error::Error + Send + Sync>>>(
                )?,
        ),
        other => other.clone(),
    })
}

async fn send_via_generic_provider(
    provider: &GenericSmsProvider,
    to: &str,
    body: &str,
    template_key: Option<&str>,
) -> Result<SmsDeliveryAttempt, Box<dyn std::error::Error + Send + Sync>> {
    if provider.provider_code == "twilio" {
        return send_via_twilio_provider(provider, to, body).await;
    }
    let template_id = template_key
        .and_then(|key| provider.template_mapping.get(key))
        .and_then(Value::as_str);
    let rendered_body = render_sms_template(
        &provider.body_template,
        to,
        body,
        &provider.sender_id,
        provider.route_type.as_deref(),
        template_id,
        &provider.auth_token,
    )?;
    let client = reqwest::Client::new();
    let mut request = match provider.method.as_str() {
        "POST" => client.post(&provider.base_url),
        "PUT" => client.put(&provider.base_url),
        _ => return Err("sms_http_method must be POST or PUT".into()),
    };
    for (name, value) in &provider.headers_template {
        let rendered = render_sms_template(
            value,
            to,
            body,
            &provider.sender_id,
            provider.route_type.as_deref(),
            template_id,
            &provider.auth_token,
        )?;
        request = request.header(
            name,
            rendered.as_str().ok_or("Invalid SMS header template")?,
        );
    }
    let response = request.json(&rendered_body).send().await?;
    if response.status().is_success() {
        Ok(SmsDeliveryAttempt {
            provider_message_id: None,
            provider_status: Some("accepted".to_string()),
        })
    } else {
        Err(format!("SMS provider returned {}", response.status()).into())
    }
}

#[derive(Debug)]
struct SmsDeliveryAttempt {
    provider_message_id: Option<String>,
    provider_status: Option<String>,
}

async fn send_via_twilio_provider(
    provider: &GenericSmsProvider,
    to: &str,
    body: &str,
) -> Result<SmsDeliveryAttempt, Box<dyn std::error::Error + Send + Sync>> {
    let account_sid = provider
        .account_sid
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or("Missing account_sid")?;
    let from_number = provider
        .from_number
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or("Missing from_number")?;
    let url = format!("https://api.twilio.com/2010-04-01/Accounts/{account_sid}/Messages.json");
    let response = reqwest::Client::new()
        .post(url)
        .basic_auth(account_sid, Some(&provider.auth_token))
        .form(&[("To", to), ("From", from_number), ("Body", body)])
        .send()
        .await?;
    let status = response.status();
    let payload = response.json::<Value>().await?;
    if !status.is_success() {
        let code = payload
            .get("code")
            .and_then(Value::as_i64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(format!("Twilio returned {status} (code {code})").into());
    }
    Ok(SmsDeliveryAttempt {
        provider_message_id: payload
            .get("sid")
            .and_then(Value::as_str)
            .map(str::to_string),
        provider_status: payload
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

async fn log_sms(
    pool: &PgPool,
    merchant_id: Option<Uuid>,
    to: &str,
    body: &str,
    template_id: Option<&str>,
    provider: &str,
    attempt: Option<&SmsDeliveryAttempt>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let message_id = Uuid::new_v4().to_string();
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
        "to_masked": mask_phone(to),
        "template_id": template_id,
        "provider": provider,
        "provider_message_id": attempt.and_then(|item| item.provider_message_id.as_deref()),
        "provider_status": attempt.and_then(|item| item.provider_status.as_deref())
    }))
    .execute(pool)
    .await?;
    Ok(message_id)
}

fn mask_phone(value: &str) -> String {
    let digits = value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.len() <= 4 {
        return "****".to_string();
    }
    format!("******{}", &digits[digits.len() - 4..])
}

/// Format amount in INR
fn format_amount(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let absolute = paise.abs();
    format!("{}₹{}.{:02}", sign, absolute / 100, absolute % 100)
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

    #[test]
    fn test_mask_phone_redacts_prefix() {
        assert_eq!(mask_phone("+919876543210"), "******3210");
    }
}

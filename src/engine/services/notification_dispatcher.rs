use crate::error::FeelPayError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::collections::BTreeMap;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DispatchRequest {
    pub merchant_id: Option<Uuid>,
    pub event_type: String,
    pub environment: String,
    pub recipient_email: Option<String>,
    pub recipient_phone: Option<String>,
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct TemplateRow {
    id: Uuid,
    channel: String,
    subject: Option<String>,
    body: String,
    sms_body: Option<String>,
    variables: Vec<String>,
    email_provider_id: Option<Uuid>,
    sms_provider_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
struct ProviderRow {
    id: Uuid,
    kind: String,
    environment: String,
    status: String,
    config: Value,
}

#[derive(Debug, Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from_name: String,
    from_email: String,
    reply_to: Option<String>,
    security: String,
}

pub async fn dispatch_event(pool: &PgPool, request: DispatchRequest) -> Result<(), FeelPayError> {
    let templates = sqlx::query(
        r#"
        SELECT id, channel, subject, body, sms_body, variables,
               default_email_provider_id, default_sms_provider_id
        FROM notification_templates
        WHERE event_type = $1
          AND environment = $2
          AND is_active = true
        ORDER BY channel
        "#,
    )
    .bind(&request.event_type)
    .bind(&request.environment)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .into_iter()
    .map(|row| TemplateRow {
        id: row.get("id"),
        channel: row.get("channel"),
        subject: row.get("subject"),
        body: row.get("body"),
        sms_body: row.get("sms_body"),
        variables: row.get("variables"),
        email_provider_id: row.get("default_email_provider_id"),
        sms_provider_id: row.get("default_sms_provider_id"),
    })
    .collect::<Vec<_>>();

    for template in templates {
        if matches!(template.channel.as_str(), "email" | "both") {
            dispatch_channel(pool, &request, &template, "email").await?;
        }
        if matches!(template.channel.as_str(), "sms" | "both") {
            dispatch_channel(pool, &request, &template, "sms").await?;
        }
    }
    Ok(())
}

pub async fn retry_failed_dispatch(pool: &PgPool, log_id: Uuid) -> Result<(), FeelPayError> {
    let row = sqlx::query(
        r#"
        SELECT retry_payload
        FROM notification_dispatch_logs
        WHERE id = $1
          AND status = 'failed'
          AND retryable = true
          AND event_type <> 'otp'
        "#,
    )
    .bind(log_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?
    .ok_or_else(|| FeelPayError::NotFound("Retryable notification log not found".into()))?;
    let payload: Value = row.get("retry_payload");
    let request: DispatchRequest = serde_json::from_value(payload)
        .map_err(|_| FeelPayError::BadRequest("Retry payload is invalid".into()))?;
    dispatch_event(pool, request).await?;
    sqlx::query(
        "UPDATE notification_dispatch_logs SET retry_count = retry_count + 1, next_retry_at = NULL WHERE id = $1",
    )
    .bind(log_id)
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    Ok(())
}

pub async fn retry_due_dispatches(pool: &PgPool, limit: i64) -> Result<i64, FeelPayError> {
    let rows = sqlx::query(
        r#"
        SELECT id
        FROM notification_dispatch_logs
        WHERE status = 'failed'
          AND retryable = true
          AND event_type <> 'otp'
          AND retry_count < max_retries
          AND next_retry_at IS NOT NULL
          AND next_retry_at <= NOW()
        ORDER BY next_retry_at
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    let mut retried = 0;
    for row in rows {
        if retry_failed_dispatch(pool, row.get("id")).await.is_ok() {
            retried += 1;
        }
    }
    Ok(retried)
}

async fn dispatch_channel(
    pool: &PgPool,
    request: &DispatchRequest,
    template: &TemplateRow,
    channel: &str,
) -> Result<(), FeelPayError> {
    let recipient = match channel {
        "email" => request.recipient_email.as_deref(),
        "sms" => request.recipient_phone.as_deref(),
        _ => None,
    };
    let recipient_masked = match (channel, recipient) {
        ("email", Some(value)) => mask_email(value),
        ("sms", Some(value)) => mask_phone(value),
        _ => "missing".to_string(),
    };
    let provider_id = if channel == "email" {
        template.email_provider_id
    } else {
        template.sms_provider_id
    };
    let started = Instant::now();
    let rendered = render_texts(template, &request.variables);
    let mut status = "failed";
    let mut error_code = None;
    let mut error_message = None;
    let mut provider_summary = json!({});
    let mut retryable = request.event_type != "otp";

    let outcome = async {
        let recipient =
            recipient.ok_or_else(|| ("missing_recipient", "Recipient is missing".to_string()))?;
        let provider_id = provider_id.ok_or_else(|| {
            (
                "missing_provider",
                "Provider mapping is missing".to_string(),
            )
        })?;
        let provider = load_provider(pool, provider_id).await?;
        if provider.status != "enabled" {
            return Err(("provider_disabled", "Provider is disabled".to_string()));
        }
        if provider.environment != request.environment {
            return Err((
                "provider_environment_mismatch",
                "Provider environment mismatch".to_string(),
            ));
        }
        if provider.kind != channel {
            return Err((
                "provider_channel_mismatch",
                "Provider channel mismatch".to_string(),
            ));
        }
        let (subject, body) = rendered
            .clone()
            .map_err(|message| ("render_failed", message))?;
        if channel == "email" {
            send_smtp_email(
                pool,
                provider.id,
                &provider.config,
                recipient,
                subject.as_deref().unwrap_or(""),
                &body,
            )
            .await?;
        } else {
            send_sms(pool, recipient, &body, &request.event_type).await?;
        }
        Ok::<_, (&str, String)>((provider.id, provider.environment))
    }
    .await;

    match outcome {
        Ok((provider_id, provider_environment)) => {
            status = "sent";
            provider_summary = json!({
                "provider_id": provider_id,
                "environment": provider_environment
            });
        }
        Err((code, message)) => {
            error_code = Some(code.to_string());
            error_message = Some(message);
            if code == "render_failed" || code == "missing_recipient" {
                retryable = false;
            }
        }
    }

    let latency_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
    sqlx::query(
        r#"
        INSERT INTO notification_dispatch_logs (
            merchant_id, event_type, channel, template_id, provider_id, recipient_masked,
            status, error_code, error_message, provider_summary, payload_summary, retry_payload, latency_ms,
            retryable, next_retry_at, sent_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
        "#,
    )
    .bind(request.merchant_id)
    .bind(&request.event_type)
    .bind(channel)
    .bind(template.id)
    .bind(provider_id)
    .bind(recipient_masked)
    .bind(status)
    .bind(error_code)
    .bind(error_message)
    .bind(provider_summary)
    .bind(json!({
        "variables": request.variables.keys().collect::<Vec<_>>()
    }))
    .bind(if retryable {
        Some(json!(request))
    } else {
        None::<Value>
    })
    .bind(latency_ms)
    .bind(retryable)
    .bind(if status == "failed" && retryable {
        Some(Utc::now() + Duration::minutes(5))
    } else {
        None::<DateTime<Utc>>
    })
    .bind(if status == "sent" {
        Some(Utc::now())
    } else {
        None::<DateTime<Utc>>
    })
    .execute(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;
    Ok(())
}

fn render_texts(
    template: &TemplateRow,
    values: &BTreeMap<String, String>,
) -> Result<(Option<String>, String), String> {
    for variable in &template.variables {
        if !values.contains_key(variable) {
            return Err(format!("Missing variable: {variable}"));
        }
    }
    let render = |input: &str| {
        let mut output = input.to_string();
        for (key, value) in values {
            output = output.replace(&format!("{{{{{key}}}}}"), value);
            output = output.replace(&format!("{{{{ {key} }}}}"), value);
        }
        if output.contains("{{") || output.contains("}}") {
            return Err("Template contains unresolved variables".to_string());
        }
        Ok(output)
    };
    let body = if template.channel == "sms" {
        render(template.sms_body.as_deref().unwrap_or(&template.body))?
    } else {
        render(&template.body)?
    };
    let subject = template.subject.as_deref().map(render).transpose()?;
    Ok((subject, body))
}

async fn load_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<ProviderRow, (&'static str, String)> {
    sqlx::query(
        "SELECT id, provider_kind, environment, status, config FROM integration_providers WHERE id=$1",
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ("provider_lookup_failed", "Provider lookup failed".to_string()))?
    .map(|row| ProviderRow {
        id: row.get("id"),
        kind: row.get("provider_kind"),
        environment: row.get("environment"),
        status: row.get("status"),
        config: row.get("config"),
    })
    .ok_or_else(|| ("provider_not_found", "Provider not found".to_string()))
}

async fn send_smtp_email(
    pool: &PgPool,
    provider_id: Uuid,
    config: &Value,
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<(), (&'static str, String)> {
    let smtp = smtp_config(pool, provider_id, config)
        .await
        .map_err(|message| ("smtp_config_invalid", message))?;
    let from = parse_mailbox(&format!("{} <{}>", smtp.from_name, smtp.from_email))
        .map_err(|message| ("smtp_invalid_sender", message))?;
    let to = parse_mailbox(recipient).map_err(|message| ("smtp_invalid_recipient", message))?;
    let mut builder = Message::builder().from(from).to(to).subject(subject);
    if let Some(reply_to) = smtp.reply_to.as_deref() {
        builder = builder.reply_to(
            parse_mailbox(reply_to).map_err(|message| ("smtp_invalid_reply_to", message))?,
        );
    }
    let message = builder
        .body(body.to_string())
        .map_err(|_| ("smtp_message_invalid", "Failed to build email".to_string()))?;
    smtp_transport(&smtp)
        .map_err(|message| ("smtp_transport_invalid", message))?
        .send(message)
        .await
        .map_err(|_| ("smtp_delivery_failed", "SMTP delivery failed".to_string()))?;
    Ok(())
}

async fn smtp_config(
    pool: &PgPool,
    provider_id: Uuid,
    config: &Value,
) -> Result<SmtpConfig, String> {
    Ok(SmtpConfig {
        host: required_string(config, "smtp_host")?,
        port: config
            .get("smtp_port")
            .and_then(Value::as_u64)
            .and_then(|v| u16::try_from(v).ok())
            .unwrap_or(587),
        username: required_string(config, "smtp_user")?,
        password: load_secret(pool, provider_id, "password").await?,
        from_name: required_string(config, "from_name")?,
        from_email: required_string(config, "from_email")?,
        reply_to: config
            .get("reply_to")
            .and_then(Value::as_str)
            .map(str::to_string),
        security: config
            .get("smtp_security")
            .and_then(Value::as_str)
            .unwrap_or("starttls")
            .to_string(),
    })
}

fn smtp_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = match config.security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host),
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &config.host,
        )),
        _ => return Err("smtp_security must be starttls, tls, or none".to_string()),
    }
    .map_err(|_| "Invalid SMTP host or TLS configuration".to_string())?;
    Ok(builder
        .port(config.port)
        .credentials(Credentials::new(
            config.username.clone(),
            config.password.clone(),
        ))
        .build())
}

fn parse_mailbox(value: &str) -> Result<Mailbox, String> {
    value
        .parse::<Mailbox>()
        .map_err(|_| "Invalid email address".to_string())
}

fn required_string(config: &Value, key: &str) -> Result<String, String> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing required config: {key}"))
}

async fn load_secret(pool: &PgPool, provider_id: Uuid, name: &str) -> Result<String, String> {
    let row = sqlx::query(
        "SELECT encrypted_value, nonce FROM integration_provider_secrets WHERE provider_id=$1 AND secret_name=$2",
    )
    .bind(provider_id)
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Secret lookup failed".to_string())?
    .ok_or_else(|| format!("Missing provider secret: {name}"))?;
    decrypt_secret(row.get("encrypted_value"), row.get("nonce"))
}

fn decrypt_secret(encrypted_value: String, nonce: String) -> Result<String, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    let raw_key = std::env::var("VAULT_ENCRYPTION_KEY")
        .map_err(|_| "VAULT_ENCRYPTION_KEY is required".to_string())?;
    let key =
        hex::decode(raw_key.trim()).map_err(|_| "VAULT_ENCRYPTION_KEY is invalid".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| "VAULT_ENCRYPTION_KEY must decode to 32 bytes".to_string())?;
    let encrypted = BASE64
        .decode(encrypted_value)
        .map_err(|_| "Encrypted secret is invalid".to_string())?;
    let nonce = BASE64
        .decode(nonce)
        .map_err(|_| "Encrypted secret nonce is invalid".to_string())?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
        .map_err(|_| "Credential decryption failed".to_string())?;
    String::from_utf8(decrypted).map_err(|_| "Credential encoding is invalid".to_string())
}

async fn send_sms(
    pool: &PgPool,
    recipient: &str,
    body: &str,
    template_key: &str,
) -> Result<(), (&'static str, String)> {
    crate::engine::services::sms::send_sms(pool, None, recipient, body, Some(template_key))
        .await
        .map(|_| ())
        .map_err(|_| ("sms_delivery_failed", "SMS delivery failed".to_string()))
}

fn mask_email(value: &str) -> String {
    let Some((local, domain)) = value.split_once('@') else {
        return "redacted".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_rejects_missing_values() {
        let template = TemplateRow {
            id: Uuid::nil(),
            channel: "email".to_string(),
            subject: Some("Hi {{name}}".to_string()),
            body: "Refund {{amount}}".to_string(),
            sms_body: None,
            variables: vec!["name".to_string(), "amount".to_string()],
            email_provider_id: None,
            sms_provider_id: None,
        };
        let mut values = BTreeMap::new();
        values.insert("name".to_string(), "Asha".to_string());
        assert!(render_texts(&template, &values).is_err());
    }

    #[test]
    fn render_rejects_unresolved_variables() {
        let template = TemplateRow {
            id: Uuid::nil(),
            channel: "sms".to_string(),
            subject: None,
            body: "Fallback".to_string(),
            sms_body: Some("OTP {{code}} {{unknown}}".to_string()),
            variables: vec!["code".to_string()],
            email_provider_id: None,
            sms_provider_id: None,
        };
        let mut values = BTreeMap::new();
        values.insert("code".to_string(), "123456".to_string());
        assert!(render_texts(&template, &values).is_err());
    }
}

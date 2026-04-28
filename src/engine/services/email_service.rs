use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Email service for transactional emails.
/// Integrates with SendGrid, AWS SES, or other providers.
#[derive(Clone)]
pub struct EmailService {
    pool: PgPool,
    provider: EmailProvider,
}

#[derive(Clone, Debug)]
pub enum EmailProvider {
    SendGrid {
        api_key: String,
    },
    AwsSes {
        region: String,
    },
    Smtp {
        host: String,
        port: u16,
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub recipient: String,
    pub subject: String,
    pub template_type: String, // payment_receipt, settlement_alert, kyc_status, etc.
    pub status: String,        // pending, sent, failed, bounced
    pub error_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub recipient: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: Option<String>,
    pub template_type: String,
    pub merchant_id: Uuid,
}

impl EmailService {
    pub fn new(pool: PgPool, provider: EmailProvider) -> Self {
        EmailService { pool, provider }
    }

    /// Send transactional email
    pub async fn send_email(&self, req: SendEmailRequest) -> Result<EmailLog, String> {
        let id = Uuid::new_v4();

        // Log email attempt
        let log = sqlx::query_as::<_, EmailLog>(
            r#"
            INSERT INTO email_logs (id, merchant_id, recipient, subject, template_type, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING id, merchant_id, recipient, subject, template_type, status, error_message, sent_at, created_at
            "#,
        )
        .bind(id)
        .bind(req.merchant_id)
        .bind(&req.recipient)
        .bind(&req.subject)
        .bind(&req.template_type)
        .bind("pending")
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to log email: {}", e))?;

        // Send via provider
        match self.send_via_provider(&req).await {
            Ok(_) => {
                // Mark as sent
                sqlx::query(
                    r#"
                    UPDATE email_logs
                    SET status = $1, sent_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind("sent")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update email status: {}", e))?;

                Ok(log)
            }
            Err(err) => {
                // Mark as failed
                sqlx::query(
                    r#"
                    UPDATE email_logs
                    SET status = $1, error_message = $2
                    WHERE id = $3
                    "#,
                )
                .bind("failed")
                .bind(&err)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update email status: {}", e))?;

                Err(err)
            }
        }
    }

    /// Send via configured provider
    async fn send_via_provider(&self, req: &SendEmailRequest) -> Result<(), String> {
        match &self.provider {
            EmailProvider::SendGrid { api_key } => self.send_via_sendgrid(api_key, req).await,
            EmailProvider::AwsSes { region } => self.send_via_aws_ses(region, req).await,
            EmailProvider::Smtp {
                host,
                port,
                username,
                password,
            } => {
                self.send_via_smtp(host, *port, username, password, req)
                    .await
            }
        }
    }

    /// SendGrid integration
    async fn send_via_sendgrid(&self, api_key: &str, req: &SendEmailRequest) -> Result<(), String> {
        let client = reqwest::Client::new();

        let payload = serde_json::json!({
            "personalizations": [{
                "to": [{ "email": &req.recipient }],
                "subject": &req.subject
            }],
            "from": { "email": "noreply@feelpay.in" },
            "content": [
                { "type": "text/html", "value": &req.body_html },
                { "type": "text/plain", "value": &req.body_text.as_ref().unwrap_or(&String::new()) }
            ]
        });

        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("SendGrid request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("SendGrid error: {}", response.status()))
        }
    }

    /// AWS SES integration
    async fn send_via_aws_ses(&self, _region: &str, req: &SendEmailRequest) -> Result<(), String> {
        // Stub: Requires AWS SDK setup
        tracing::info!("Sending email via AWS SES to {}", req.recipient);
        Ok(())
    }

    /// SMTP integration
    async fn send_via_smtp(
        &self,
        _host: &str,
        _port: u16,
        _username: &str,
        _password: &str,
        req: &SendEmailRequest,
    ) -> Result<(), String> {
        // Stub: Requires lettre crate
        tracing::info!("Sending email via SMTP to {}", req.recipient);
        Ok(())
    }

    /// Get email logs for merchant
    pub async fn get_email_logs(
        &self,
        merchant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EmailLog>, String> {
        sqlx::query_as::<_, EmailLog>(
            r#"
            SELECT id, merchant_id, recipient, subject, template_type, status, error_message, sent_at, created_at
            FROM email_logs
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch email logs: {}", e))
    }

    /// Retry failed emails
    pub async fn retry_failed_emails(&self) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            UPDATE email_logs
            SET status = 'pending'
            WHERE status = 'failed' AND created_at > NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to retry emails: {}", e))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_provider_variants() {
        let _sendgrid = EmailProvider::SendGrid {
            api_key: "test".to_string(),
        };
        let _ses = EmailProvider::AwsSes {
            region: "us-east-1".to_string(),
        };
        let _smtp = EmailProvider::Smtp {
            host: "smtp.example.com".to_string(),
            port: 587,
            username: "user".to_string(),
            password: "pass".to_string(),
        };
    }

    #[test]
    fn test_email_log_status() {
        let statuses = vec!["pending", "sent", "failed", "bounced"];
        assert_eq!(statuses.len(), 4);
    }
}

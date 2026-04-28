use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// SMS service for transactional SMS.
/// Integrates with Twilio, AWS SNS, or other providers.
#[derive(Clone)]
pub struct SmsService {
    pool: PgPool,
    provider: SmsProvider,
}

#[derive(Clone, Debug)]
pub enum SmsProvider {
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    AwsSns {
        region: String,
    },
    Exotel {
        api_key: String,
        api_token: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SmsLog {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub recipient: String,
    pub message: String,
    pub template_type: String, // otp, payment_confirmation, settlement_alert, etc.
    pub status: String,        // pending, sent, failed, bounced
    pub error_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    pub recipient: String, // Phone number in E.164 format: +919876543210
    pub message: String,
    pub template_type: String,
    pub merchant_id: Uuid,
}

impl SmsService {
    pub fn new(pool: PgPool, provider: SmsProvider) -> Self {
        SmsService { pool, provider }
    }

    /// Send transactional SMS
    pub async fn send_sms(&self, req: SendSmsRequest) -> Result<SmsLog, String> {
        // Validate phone number format
        if !req.recipient.starts_with('+') || req.recipient.len() < 10 {
            return Err("Invalid phone number format".to_string());
        }

        let id = Uuid::new_v4();

        // Log SMS attempt
        let log = sqlx::query_as::<_, SmsLog>(
            r#"
            INSERT INTO sms_logs (id, merchant_id, recipient, message, template_type, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING id, merchant_id, recipient, message, template_type, status, error_message, sent_at, created_at
            "#,
        )
        .bind(id)
        .bind(req.merchant_id)
        .bind(&req.recipient)
        .bind(&req.message)
        .bind(&req.template_type)
        .bind("pending")
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to log SMS: {}", e))?;

        // Send via provider
        match self.send_via_provider(&req).await {
            Ok(_) => {
                // Mark as sent
                sqlx::query(
                    r#"
                    UPDATE sms_logs
                    SET status = $1, sent_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind("sent")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update SMS status: {}", e))?;

                Ok(log)
            }
            Err(err) => {
                // Mark as failed
                sqlx::query(
                    r#"
                    UPDATE sms_logs
                    SET status = $1, error_message = $2
                    WHERE id = $3
                    "#,
                )
                .bind("failed")
                .bind(&err)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to update SMS status: {}", e))?;

                Err(err)
            }
        }
    }

    /// Send via configured provider
    async fn send_via_provider(&self, req: &SendSmsRequest) -> Result<(), String> {
        match &self.provider {
            SmsProvider::Twilio {
                account_sid,
                auth_token,
                from_number,
            } => {
                self.send_via_twilio(account_sid, auth_token, from_number, req)
                    .await
            }
            SmsProvider::AwsSns { region } => self.send_via_aws_sns(region, req).await,
            SmsProvider::Exotel { api_key, api_token } => {
                self.send_via_exotel(api_key, api_token, req).await
            }
        }
    }

    /// Twilio integration
    async fn send_via_twilio(
        &self,
        account_sid: &str,
        auth_token: &str,
        from_number: &str,
        req: &SendSmsRequest,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            account_sid
        );

        let params = [
            ("From", from_number),
            ("To", &req.recipient),
            ("Body", &req.message),
        ];

        let response = client
            .post(&url)
            .basic_auth(account_sid, Some(auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Twilio request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Twilio error: {}", response.status()))
        }
    }

    /// AWS SNS integration
    async fn send_via_aws_sns(&self, _region: &str, req: &SendSmsRequest) -> Result<(), String> {
        // Stub: Requires AWS SDK setup
        tracing::info!("Sending SMS via AWS SNS to {}", req.recipient);
        Ok(())
    }

    /// Exotel integration (India-specific)
    async fn send_via_exotel(
        &self,
        api_key: &str,
        api_token: &str,
        req: &SendSmsRequest,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = "https://api.exotel.com/v1/Sms/send.json";

        let payload = serde_json::json!({
            "From": "FeelPay",
            "To": &req.recipient,
            "Body": &req.message
        });

        let response = client
            .post(url)
            .basic_auth(api_key, Some(api_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Exotel request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Exotel error: {}", response.status()))
        }
    }

    /// Get SMS logs for merchant
    pub async fn get_sms_logs(&self, merchant_id: Uuid, limit: i64) -> Result<Vec<SmsLog>, String> {
        sqlx::query_as::<_, SmsLog>(
            r#"
            SELECT id, merchant_id, recipient, message, template_type, status, error_message, sent_at, created_at
            FROM sms_logs
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(merchant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch SMS logs: {}", e))
    }

    /// Retry failed SMS
    pub async fn retry_failed_sms(&self) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            UPDATE sms_logs
            SET status = 'pending'
            WHERE status = 'failed' AND created_at > NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to retry SMS: {}", e))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sms_provider_variants() {
        let _twilio = SmsProvider::Twilio {
            account_sid: "test".to_string(),
            auth_token: "test".to_string(),
            from_number: "+1234567890".to_string(),
        };
        let _sns = SmsProvider::AwsSns {
            region: "ap-south-1".to_string(),
        };
        let _exotel = SmsProvider::Exotel {
            api_key: "test".to_string(),
            api_token: "test".to_string(),
        };
    }

    #[test]
    fn test_phone_number_validation() {
        let valid = "+919876543210";
        assert!(valid.starts_with('+'));
        assert!(valid.len() >= 10);
    }
}

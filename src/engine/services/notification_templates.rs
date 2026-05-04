use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Notification template service for transactional communications.
/// Supports email, SMS, WhatsApp templates with variable substitution.
#[derive(Clone)]
pub struct NotificationTemplateService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Sms,
    WhatsApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEventType {
    PaymentReceipt,
    PaymentFailed,
    RefundInitiated,
    RefundSuccess,
    DisputeCreated,
    SettlementProcessed,
    KycApproved,
    KycRejected,
    SubscriptionCharged,
    SubscriptionFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub event_type: String,      // payment_receipt, payment_failed, etc.
    pub channel: String,         // email, sms, whatsapp
    pub subject: Option<String>, // For email
    pub body: String,            // Template with {{variables}}
    pub variables: Vec<String>,  // List of required variables
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RenderedNotification {
    pub id: Uuid,
    pub template_id: Uuid,
    pub merchant_id: Uuid,
    pub recipient: String, // Email, phone, or WhatsApp ID
    pub channel: String,
    pub subject: Option<String>,
    pub body: String,
    pub variables_used: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl NotificationTemplateService {
    pub fn new(pool: PgPool) -> Self {
        NotificationTemplateService { pool }
    }

    /// Create notification template
    pub async fn create_template(
        &self,
        event_type: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        variables: Vec<String>,
    ) -> Result<NotificationTemplate, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, NotificationTemplate>(
            r#"
            INSERT INTO notification_templates (id, event_type, channel, subject, body, variables, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, event_type, channel, subject, body, variables, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(event_type)
        .bind(channel)
        .bind(subject)
        .bind(body)
        .bind(variables)
        .bind(true)
        .fetch_one(&self.pool)
        .await
    }

    /// Get template by event type and channel
    pub async fn get_template(
        &self,
        event_type: &str,
        channel: &str,
    ) -> Result<Option<NotificationTemplate>, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplate>(
            r#"
            SELECT id, event_type, channel, subject, body, variables, is_active, created_at, updated_at
            FROM notification_templates
            WHERE event_type = $1 AND channel = $2 AND is_active = true
            "#,
        )
        .bind(event_type)
        .bind(channel)
        .fetch_optional(&self.pool)
        .await
    }

    /// Render template with variables
    pub fn render_template(
        &self,
        template: &NotificationTemplate,
        variables: &std::collections::HashMap<String, String>,
    ) -> Result<(Option<String>, String), String> {
        let mut subject = template.subject.clone();
        let mut body = template.body.clone();

        // Substitute variables
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            if let Some(ref mut s) = subject {
                *s = s.replace(&placeholder, value);
            }
            body = body.replace(&placeholder, value);
        }

        // Check for missing variables
        if body.contains("{{") {
            return Err("Missing required template variables".to_string());
        }

        Ok((subject, body))
    }

    /// Store rendered notification
    pub async fn store_rendered(
        &self,
        template_id: Uuid,
        merchant_id: Uuid,
        recipient: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        variables: serde_json::Value,
    ) -> Result<RenderedNotification, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, RenderedNotification>(
            r#"
            INSERT INTO rendered_notifications (id, template_id, merchant_id, recipient, channel, subject, body, variables_used)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, template_id, merchant_id, recipient, channel, subject, body, variables_used, created_at
            "#,
        )
        .bind(id)
        .bind(template_id)
        .bind(merchant_id)
        .bind(recipient)
        .bind(channel)
        .bind(subject)
        .bind(body)
        .bind(variables)
        .fetch_one(&self.pool)
        .await
    }

    /// Update template
    pub async fn update_template(
        &self,
        template_id: Uuid,
        subject: Option<&str>,
        body: &str,
    ) -> Result<NotificationTemplate, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplate>(
            r#"
            UPDATE notification_templates
            SET subject = $1, body = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, event_type, channel, subject, body, variables, is_active, created_at, updated_at
            "#,
        )
        .bind(subject)
        .bind(body)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Disable template
    pub async fn disable_template(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE notification_templates
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all templates for event type
    pub async fn get_templates_for_event(
        &self,
        event_type: &str,
    ) -> Result<Vec<NotificationTemplate>, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplate>(
            r#"
            SELECT id, event_type, channel, subject, body, variables, is_active, created_at, updated_at
            FROM notification_templates
            WHERE event_type = $1 AND is_active = true
            ORDER BY channel
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
    }

    /// Get default templates (system-provided)
    pub async fn get_default_templates(&self) -> Result<Vec<NotificationTemplate>, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplate>(
            r#"
            SELECT id, event_type, channel, subject, body, variables, is_active, created_at, updated_at
            FROM notification_templates
            WHERE is_active = true
            ORDER BY event_type, channel
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_template_rendering() {
        let service = NotificationTemplateService::new(
            sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        );
        let template = NotificationTemplate {
            id: Uuid::new_v4(),
            event_type: "payment_receipt".to_string(),
            channel: "email".to_string(),
            subject: Some("Payment Received: {{amount}}".to_string()),
            body: "Hi {{customer_name}}, your payment of {{amount}} was successful.".to_string(),
            variables: vec!["customer_name".to_string(), "amount".to_string()],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut vars = HashMap::new();
        vars.insert("customer_name".to_string(), "John Doe".to_string());
        vars.insert("amount".to_string(), "₹1000".to_string());

        let (subject, body) = service.render_template(&template, &vars).unwrap();
        assert_eq!(subject, Some("Payment Received: ₹1000".to_string()));
        assert!(body.contains("John Doe"));
        assert!(body.contains("₹1000"));
    }

    #[tokio::test]
    async fn test_missing_variables() {
        let service = NotificationTemplateService::new(
            sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        );
        let template = NotificationTemplate {
            id: Uuid::new_v4(),
            event_type: "payment_receipt".to_string(),
            channel: "email".to_string(),
            subject: None,
            body: "Hi {{customer_name}}, your payment was successful.".to_string(),
            variables: vec!["customer_name".to_string()],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let vars = HashMap::new();
        let result = service.render_template(&template, &vars);
        assert!(result.is_err());
    }
}

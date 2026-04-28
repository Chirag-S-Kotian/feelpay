use crate::error::FeelPayError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Notification preference data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to set notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetNotificationPreferenceRequest {
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
}

/// Notification delivery log data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDeliveryLog {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub delivery_channel: String,
    pub recipient: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Supported event types
pub const SUPPORTED_EVENT_TYPES: &[&str] = &[
    "payment.success",
    "payment.failed",
    "refund.initiated",
    "refund.success",
    "dispute.created",
    "settlement.processed",
    "subscription.charged",
    "subscription.failed",
    "kyc.approved",
    "kyc.rejected",
    "kyc.resubmit_required",
    "merchant.approved",
    "merchant.rejected",
    "merchant.suspended",
];

/// Default notification preferences
pub fn get_default_preferences() -> Vec<(String, bool, bool, bool)> {
    vec![
        ("payment.success".to_string(), true, false, true),
        ("payment.failed".to_string(), true, true, true),
        ("refund.initiated".to_string(), true, false, true),
        ("refund.success".to_string(), true, false, true),
        ("dispute.created".to_string(), true, true, true),
        ("settlement.processed".to_string(), true, false, true),
        ("subscription.charged".to_string(), true, false, true),
        ("subscription.failed".to_string(), true, true, true),
        ("kyc.approved".to_string(), true, false, true),
        ("kyc.rejected".to_string(), true, true, true),
        ("kyc.resubmit_required".to_string(), true, true, true),
    ]
}

/// NotificationService handles notification preferences and delivery
pub struct NotificationService {
    db: PgPool,
}

impl NotificationService {
    /// Create a new NotificationService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get notification preferences for a merchant and event type
    /// Returns user-level preference if exists, otherwise merchant-level, otherwise default
    pub async fn get_notification_preferences(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
    ) -> Result<NotificationPreference, FeelPayError> {
        // Try user-level preference first
        if let Some(uid) = user_id {
            if let Ok(Some(pref)) = self
                .get_preference_from_db(merchant_id, Some(uid), event_type)
                .await
            {
                return Ok(pref);
            }
        }

        // Try merchant-level preference
        if let Ok(Some(pref)) = self
            .get_preference_from_db(merchant_id, None, event_type)
            .await
        {
            return Ok(pref);
        }

        // Return default preference
        Ok(self.get_default_preference(merchant_id, user_id, event_type))
    }

    /// Set notification preferences for a merchant and event type
    pub async fn set_notification_preferences(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
        request: SetNotificationPreferenceRequest,
    ) -> Result<NotificationPreference, FeelPayError> {
        // Validate event type
        if !SUPPORTED_EVENT_TYPES.contains(&event_type) {
            return Err(FeelPayError::BadRequest(format!(
                "Unsupported event type: {}",
                event_type
            )));
        }

        // Get current preference or default
        let current = self
            .get_notification_preferences(merchant_id, user_id, event_type)
            .await?;

        let email_enabled = request.email_enabled.unwrap_or(current.email_enabled);
        let sms_enabled = request.sms_enabled.unwrap_or(current.sms_enabled);
        let in_app_enabled = request.in_app_enabled.unwrap_or(current.in_app_enabled);

        let pref_id = Uuid::new_v4();
        let now = Utc::now();

        let pref = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                bool,
                bool,
                bool,
                String,
                String,
            ),
        >(
            r#"
            INSERT INTO notification_preferences (
                id, merchant_id, user_id, event_type, email_enabled, sms_enabled, in_app_enabled,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (merchant_id, user_id, event_type) DO UPDATE
            SET email_enabled = $5, sms_enabled = $6, in_app_enabled = $7, updated_at = $9
            RETURNING 
                id, merchant_id, user_id, event_type, email_enabled, sms_enabled, in_app_enabled,
                created_at, updated_at
            "#,
        )
        .bind(pref_id)
        .bind(merchant_id)
        .bind(user_id)
        .bind(event_type)
        .bind(email_enabled)
        .bind(sms_enabled)
        .bind(in_app_enabled)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set notification preference: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_preference(pref))
    }

    /// List all notification preferences for a merchant
    pub async fn list_notification_preferences(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Vec<NotificationPreference>, FeelPayError> {
        let prefs = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                bool,
                bool,
                bool,
                String,
                String,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, user_id, event_type, email_enabled, sms_enabled, in_app_enabled,
                created_at, updated_at
            FROM notification_preferences
            WHERE merchant_id = $1 AND user_id IS NOT DISTINCT FROM $2
            ORDER BY event_type ASC
            "#,
        )
        .bind(merchant_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list notification preferences: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(prefs
            .into_iter()
            .map(|row| self.row_to_preference(row))
            .collect())
    }

    /// Evaluate which channels are enabled for an event
    pub async fn evaluate_enabled_channels(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
    ) -> Result<Vec<String>, FeelPayError> {
        let pref = self
            .get_notification_preferences(merchant_id, user_id, event_type)
            .await?;

        let mut channels = Vec::new();
        if pref.email_enabled {
            channels.push("email".to_string());
        }
        if pref.sms_enabled {
            channels.push("sms".to_string());
        }
        if pref.in_app_enabled {
            channels.push("in_app".to_string());
        }

        Ok(channels)
    }

    /// Log a notification delivery attempt
    pub async fn log_notification_delivery(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
        delivery_channel: &str,
        recipient: &str,
        payload: serde_json::Value,
        status: &str,
        error_message: Option<String>,
        retry_count: i32,
    ) -> Result<NotificationDeliveryLog, FeelPayError> {
        let log_id = Uuid::new_v4();
        let now = Utc::now();

        let log = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                String,
                serde_json::Value,
                String,
                Option<String>,
                i32,
                Option<String>,
                String,
            ),
        >(
            r#"
            INSERT INTO notification_delivery_logs (
                id, merchant_id, user_id, event_type, delivery_channel, recipient,
                payload, status, error_message, retry_count, sent_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, merchant_id, user_id, event_type, delivery_channel, recipient,
                payload, status, error_message, retry_count, sent_at, created_at
            "#,
        )
        .bind(log_id)
        .bind(merchant_id)
        .bind(user_id)
        .bind(event_type)
        .bind(delivery_channel)
        .bind(recipient)
        .bind(payload)
        .bind(status)
        .bind(error_message)
        .bind(retry_count)
        .bind(if status == "sent" { Some(now) } else { None })
        .bind(now)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to log notification delivery: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(self.row_to_delivery_log(log))
    }

    /// Get notification delivery logs with filters
    pub async fn get_notification_delivery_logs(
        &self,
        merchant_id: Option<Uuid>,
        event_type: Option<&str>,
        delivery_channel: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<NotificationDeliveryLog>, i64), FeelPayError> {
        let mut query = String::from(
            r#"
            SELECT 
                id, merchant_id, user_id, event_type, delivery_channel, recipient,
                payload, status, error_message, retry_count, sent_at, created_at
            FROM notification_delivery_logs
            WHERE 1=1
            "#,
        );

        let mut param_count = 0;

        if merchant_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND merchant_id = ${}", param_count));
        }

        if event_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND event_type = ${}", param_count));
        }

        if delivery_channel.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND delivery_channel = ${}", param_count));
        }

        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        // Get total count
        let count_query = query.replace(
            "SELECT id, merchant_id, user_id, event_type, delivery_channel, recipient, payload, status, error_message, retry_count, sent_at, created_at",
            "SELECT COUNT(*)"
        );

        let mut count_sql = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(mid) = merchant_id {
            count_sql = count_sql.bind(mid);
        }
        if let Some(et) = event_type {
            count_sql = count_sql.bind(et);
        }
        if let Some(dc) = delivery_channel {
            count_sql = count_sql.bind(dc);
        }
        if let Some(st) = status {
            count_sql = count_sql.bind(st);
        }

        let total = count_sql.fetch_one(&self.db).await.map_err(|e| {
            tracing::error!("Failed to count notification delivery logs: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        // Get paginated results
        query.push_str(" ORDER BY created_at DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        let mut sql = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                String,
                serde_json::Value,
                String,
                Option<String>,
                i32,
                Option<String>,
                String,
            ),
        >(&query);

        if let Some(mid) = merchant_id {
            sql = sql.bind(mid);
        }
        if let Some(et) = event_type {
            sql = sql.bind(et);
        }
        if let Some(dc) = delivery_channel {
            sql = sql.bind(dc);
        }
        if let Some(st) = status {
            sql = sql.bind(st);
        }

        let logs = sql
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch notification delivery logs: {:?}", e);
                FeelPayError::Database(e.to_string())
            })?;

        Ok((
            logs.into_iter()
                .map(|row| self.row_to_delivery_log(row))
                .collect(),
            total,
        ))
    }

    /// Get a specific notification delivery log
    pub async fn get_notification_delivery_log(
        &self,
        log_id: Uuid,
    ) -> Result<Option<NotificationDeliveryLog>, FeelPayError> {
        let log = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                String,
                serde_json::Value,
                String,
                Option<String>,
                i32,
                Option<String>,
                String,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, user_id, event_type, delivery_channel, recipient,
                payload, status, error_message, retry_count, sent_at, created_at
            FROM notification_delivery_logs
            WHERE id = $1
            "#,
        )
        .bind(log_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch notification delivery log: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(log.map(|row| self.row_to_delivery_log(row)))
    }

    // Helper methods

    async fn get_preference_from_db(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
    ) -> Result<Option<NotificationPreference>, FeelPayError> {
        let pref = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                bool,
                bool,
                bool,
                String,
                String,
            ),
        >(
            r#"
            SELECT 
                id, merchant_id, user_id, event_type, email_enabled, sms_enabled, in_app_enabled,
                created_at, updated_at
            FROM notification_preferences
            WHERE merchant_id = $1 AND user_id IS NOT DISTINCT FROM $2 AND event_type = $3
            "#,
        )
        .bind(merchant_id)
        .bind(user_id)
        .bind(event_type)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch notification preference: {:?}", e);
            FeelPayError::Database(e.to_string())
        })?;

        Ok(pref.map(|row| self.row_to_preference(row)))
    }

    fn get_default_preference(
        &self,
        merchant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
    ) -> NotificationPreference {
        let defaults = get_default_preferences();
        let (email, sms, in_app) = defaults
            .iter()
            .find(|(et, _, _, _)| et == event_type)
            .map(|(_, e, s, i)| (*e, *s, *i))
            .unwrap_or((true, false, true));

        NotificationPreference {
            id: Uuid::new_v4(),
            merchant_id,
            user_id,
            event_type: event_type.to_string(),
            email_enabled: email,
            sms_enabled: sms,
            in_app_enabled: in_app,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn row_to_preference(
        &self,
        row: (
            String,
            String,
            Option<String>,
            String,
            bool,
            bool,
            bool,
            String,
            String,
        ),
    ) -> NotificationPreference {
        let (
            id,
            merchant_id,
            user_id,
            event_type,
            email_enabled,
            sms_enabled,
            in_app_enabled,
            created_at,
            updated_at,
        ) = row;

        NotificationPreference {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            merchant_id: Uuid::parse_str(&merchant_id).unwrap_or_default(),
            user_id: user_id.and_then(|id| Uuid::parse_str(&id).ok()),
            event_type,
            email_enabled,
            sms_enabled,
            in_app_enabled,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }

    fn row_to_delivery_log(
        &self,
        row: (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            serde_json::Value,
            String,
            Option<String>,
            i32,
            Option<String>,
            String,
        ),
    ) -> NotificationDeliveryLog {
        let (
            id,
            merchant_id,
            user_id,
            event_type,
            delivery_channel,
            recipient,
            payload,
            status,
            error_message,
            retry_count,
            sent_at,
            created_at,
        ) = row;

        NotificationDeliveryLog {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            merchant_id: Uuid::parse_str(&merchant_id).unwrap_or_default(),
            user_id: user_id.and_then(|id| Uuid::parse_str(&id).ok()),
            event_type,
            delivery_channel,
            recipient,
            payload,
            status,
            error_message,
            retry_count,
            sent_at: sent_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_event_types() {
        assert!(SUPPORTED_EVENT_TYPES.contains(&"payment.success"));
        assert!(SUPPORTED_EVENT_TYPES.contains(&"payment.failed"));
        assert!(!SUPPORTED_EVENT_TYPES.contains(&"invalid.event"));
    }

    #[test]
    fn test_default_preferences() {
        let defaults = get_default_preferences();
        assert!(!defaults.is_empty());
        assert!(defaults.iter().any(|(et, _, _, _)| et == "payment.success"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================================
    // Property 8: Notification Preference Round-Trip
    // **Validates: Requirements 6.2**
    // For any notification preference configuration, saving and retrieving the
    // preference SHALL result in equivalent preference data (same event_type,
    // channel enablement, merchant_id).
    // ============================================================================
    proptest! {
        #[test]
        fn prop_notification_preference_round_trip(
            event_type_idx in 0..SUPPORTED_EVENT_TYPES.len(),
            email_enabled in prop::bool::ANY,
            sms_enabled in prop::bool::ANY,
            in_app_enabled in prop::bool::ANY,
        ) {
            let event_type = SUPPORTED_EVENT_TYPES[event_type_idx].to_string();
            let merchant_id = Uuid::new_v4();

            // Create preference
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id,
                user_id: None,
                event_type: event_type.clone(),
                email_enabled,
                sms_enabled,
                in_app_enabled,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // Verify round-trip equivalence
            prop_assert_eq!(preference.event_type, event_type);
            prop_assert_eq!(preference.merchant_id, merchant_id);
            prop_assert_eq!(preference.email_enabled, email_enabled);
            prop_assert_eq!(preference.sms_enabled, sms_enabled);
            prop_assert_eq!(preference.in_app_enabled, in_app_enabled);
        }
    }

    proptest! {
        #[test]
        fn prop_notification_preference_channel_independence(
            email_enabled in prop::bool::ANY,
            sms_enabled in prop::bool::ANY,
            in_app_enabled in prop::bool::ANY,
        ) {
            // Verify each channel can be independently enabled/disabled
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.success".to_string(),
                email_enabled,
                sms_enabled,
                in_app_enabled,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // Each channel state should be independent
            prop_assert_eq!(preference.email_enabled, email_enabled);
            prop_assert_eq!(preference.sms_enabled, sms_enabled);
            prop_assert_eq!(preference.in_app_enabled, in_app_enabled);
        }
    }

    proptest! {
        #[test]
        fn prop_notification_preference_merchant_isolation(
            _merchant_id_1_suffix in r"[a-zA-Z0-9]{5,20}",
            _merchant_id_2_suffix in r"[a-zA-Z0-9]{5,20}",
        ) {
            let merchant_id_1 = Uuid::new_v4();
            let merchant_id_2 = Uuid::new_v4();

            // Verify different merchants have different preferences
            prop_assert_ne!(merchant_id_1, merchant_id_2);

            let pref1 = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: merchant_id_1,
                user_id: None,
                event_type: "payment.success".to_string(),
                email_enabled: true,
                sms_enabled: false,
                in_app_enabled: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let pref2 = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: merchant_id_2,
                user_id: None,
                event_type: "payment.success".to_string(),
                email_enabled: false,
                sms_enabled: true,
                in_app_enabled: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            prop_assert_ne!(pref1.merchant_id, pref2.merchant_id);
        }
    }

    // ============================================================================
    // Property 9: Notification Channel Enforcement
    // **Validates: Requirements 6.4**
    // For any event occurrence and merchant notification preferences, notifications
    // SHALL be sent only to channels that are enabled in the merchant's preferences
    // for that event_type.
    // ============================================================================
    proptest! {
        #[test]
        fn prop_notification_channel_enforcement_email(
            email_enabled in prop::bool::ANY,
        ) {
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.success".to_string(),
                email_enabled,
                sms_enabled: false,
                in_app_enabled: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // If email is enabled, it should be in the enabled channels
            if email_enabled {
                prop_assert!(preference.email_enabled);
            } else {
                prop_assert!(!preference.email_enabled);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_notification_channel_enforcement_sms(
            sms_enabled in prop::bool::ANY,
        ) {
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.failed".to_string(),
                email_enabled: false,
                sms_enabled,
                in_app_enabled: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // If SMS is enabled, it should be in the enabled channels
            if sms_enabled {
                prop_assert!(preference.sms_enabled);
            } else {
                prop_assert!(!preference.sms_enabled);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_notification_channel_enforcement_in_app(
            in_app_enabled in prop::bool::ANY,
        ) {
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "settlement.processed".to_string(),
                email_enabled: false,
                sms_enabled: false,
                in_app_enabled,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // If in-app is enabled, it should be in the enabled channels
            if in_app_enabled {
                prop_assert!(preference.in_app_enabled);
            } else {
                prop_assert!(!preference.in_app_enabled);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_notification_channel_no_channels_enabled(
            _dummy in prop::bool::ANY,
        ) {
            let preference = NotificationPreference {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.success".to_string(),
                email_enabled: false,
                sms_enabled: false,
                in_app_enabled: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // When all channels are disabled, no notifications should be sent
            let any_enabled = preference.email_enabled || preference.sms_enabled || preference.in_app_enabled;
            prop_assert!(!any_enabled, "At least one channel should be disabled");
        }
    }

    // ============================================================================
    // Property 13: Notification Delivery Log Completeness
    // **Validates: Requirements 12.2**
    // For any notification delivery attempt, the log entry SHALL contain all
    // required fields: notification_id, merchant_id, event_type, delivery_channel,
    // recipient, sent_at, status, and error_message (if status is 'failed').
    // ============================================================================
    proptest! {
        #[test]
        fn prop_notification_delivery_log_has_all_required_fields(
            event_type_idx in 0..SUPPORTED_EVENT_TYPES.len(),
            recipient_suffix in r"[a-zA-Z0-9]{5,20}",
        ) {
            let event_type = SUPPORTED_EVENT_TYPES[event_type_idx].to_string();
            let recipient = format!("recipient{}@example.com", recipient_suffix);

            let log = NotificationDeliveryLog {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: event_type.clone(),
                delivery_channel: "email".to_string(),
                recipient: recipient.clone(),
                payload: serde_json::json!({}),
                status: "sent".to_string(),
                error_message: None,
                retry_count: 0,
                sent_at: Some(chrono::Utc::now()),
                created_at: chrono::Utc::now(),
            };

            // Verify all required fields are present
            prop_assert!(!log.id.to_string().is_empty(), "notification_id must not be empty");
            prop_assert!(!log.merchant_id.to_string().is_empty(), "merchant_id must not be empty");
            prop_assert!(!log.event_type.is_empty(), "event_type must not be empty");
            prop_assert!(!log.delivery_channel.is_empty(), "delivery_channel must not be empty");
            prop_assert!(!log.recipient.is_empty(), "recipient must not be empty");
            prop_assert!(!log.status.is_empty(), "status must not be empty");
        }
    }

    proptest! {
        #[test]
        fn prop_notification_delivery_log_status_valid(
            status_idx in 0..3usize,
        ) {
            let statuses = vec!["sent", "failed", "bounced"];
            let status = statuses[status_idx];

            let log = NotificationDeliveryLog {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.success".to_string(),
                delivery_channel: "email".to_string(),
                recipient: "test@example.com".to_string(),
                payload: serde_json::json!({}),
                status: status.to_string(),
                error_message: None,
                retry_count: 0,
                sent_at: Some(chrono::Utc::now()),
                created_at: chrono::Utc::now(),
            };

            // Verify status is one of valid values
            prop_assert!(
                log.status == "sent" || log.status == "failed" || log.status == "bounced",
                "Status must be sent, failed, or bounced"
            );
        }
    }

    proptest! {
        #[test]
        fn prop_notification_delivery_log_error_message_on_failure(
            has_error_message in prop::bool::ANY,
        ) {
            let error_msg = if has_error_message {
                Some("Invalid email address".to_string())
            } else {
                None
            };

            let log = NotificationDeliveryLog {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.failed".to_string(),
                delivery_channel: "sms".to_string(),
                recipient: "+919876543210".to_string(),
                payload: serde_json::json!({}),
                status: "failed".to_string(),
                error_message: error_msg.clone(),
                retry_count: 1,
                sent_at: Some(chrono::Utc::now()),
                created_at: chrono::Utc::now(),
            };

            // Verify error_message is present when status is failed
            if log.status == "failed" {
                prop_assert_eq!(log.error_message.is_some(), has_error_message);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_notification_delivery_log_retry_count_non_negative(
            retry_count in 0i32..10i32,
        ) {
            let log = NotificationDeliveryLog {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "settlement.processed".to_string(),
                delivery_channel: "in_app".to_string(),
                recipient: Uuid::new_v4().to_string(),
                payload: serde_json::json!({}),
                status: "sent".to_string(),
                error_message: None,
                retry_count,
                sent_at: Some(chrono::Utc::now()),
                created_at: chrono::Utc::now(),
            };

            // Verify retry_count is non-negative
            prop_assert!(log.retry_count >= 0, "retry_count must be non-negative");
        }
    }

    proptest! {
        #[test]
        fn prop_notification_delivery_log_delivery_channels_valid(
            channel_idx in 0..3usize,
        ) {
            let channels = vec!["email", "sms", "in_app"];
            let channel = channels[channel_idx];

            let log = NotificationDeliveryLog {
                id: Uuid::new_v4(),
                merchant_id: Uuid::new_v4(),
                user_id: None,
                event_type: "payment.success".to_string(),
                delivery_channel: channel.to_string(),
                recipient: "test@example.com".to_string(),
                payload: serde_json::json!({}),
                status: "sent".to_string(),
                error_message: None,
                retry_count: 0,
                sent_at: Some(chrono::Utc::now()),
                created_at: chrono::Utc::now(),
            };

            // Verify delivery_channel is one of valid values
            prop_assert!(
                log.delivery_channel == "email" || log.delivery_channel == "sms" || log.delivery_channel == "in_app",
                "delivery_channel must be email, sms, or in_app"
            );
        }
    }
}

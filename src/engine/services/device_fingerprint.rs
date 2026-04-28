use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Device fingerprinting service for fraud detection.
/// Tracks device characteristics and identifies suspicious patterns.
#[derive(Clone)]
pub struct DeviceFingerprintService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub device_id: String, // Hash of device characteristics
    pub user_agent: String,
    pub ip_address: String,
    pub accept_language: Option<String>,
    pub accept_encoding: Option<String>,
    pub timezone: Option<String>,
    pub screen_resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredDeviceFingerprint {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub device_id: String,
    pub user_agent: String,
    pub ip_address: String,
    pub accept_language: Option<String>,
    pub accept_encoding: Option<String>,
    pub timezone: Option<String>,
    pub screen_resolution: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub transaction_count: i32,
    pub is_trusted: bool,
}

impl DeviceFingerprintService {
    pub fn new(pool: PgPool) -> Self {
        DeviceFingerprintService { pool }
    }

    /// Generate device fingerprint hash from characteristics
    pub fn generate_fingerprint(
        user_agent: &str,
        ip_address: &str,
        accept_language: Option<&str>,
        accept_encoding: Option<&str>,
        timezone: Option<&str>,
        screen_resolution: Option<&str>,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(user_agent.as_bytes());
        hasher.update(ip_address.as_bytes());

        if let Some(lang) = accept_language {
            hasher.update(lang.as_bytes());
        }
        if let Some(enc) = accept_encoding {
            hasher.update(enc.as_bytes());
        }
        if let Some(tz) = timezone {
            hasher.update(tz.as_bytes());
        }
        if let Some(res) = screen_resolution {
            hasher.update(res.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Store device fingerprint
    pub async fn store_fingerprint(
        &self,
        merchant_id: Uuid,
        customer_id: Option<Uuid>,
        fingerprint: &DeviceFingerprint,
    ) -> Result<StoredDeviceFingerprint, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, StoredDeviceFingerprint>(
            r#"
            INSERT INTO device_fingerprints (
                id, merchant_id, customer_id, device_id, user_agent, ip_address,
                accept_language, accept_encoding, timezone, screen_resolution
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, merchant_id, customer_id, device_id, user_agent, ip_address,
                      accept_language, accept_encoding, timezone, screen_resolution,
                      first_seen, last_seen, transaction_count, is_trusted
            "#,
        )
        .bind(id)
        .bind(merchant_id)
        .bind(customer_id)
        .bind(&fingerprint.device_id)
        .bind(&fingerprint.user_agent)
        .bind(&fingerprint.ip_address)
        .bind(&fingerprint.accept_language)
        .bind(&fingerprint.accept_encoding)
        .bind(&fingerprint.timezone)
        .bind(&fingerprint.screen_resolution)
        .fetch_one(&self.pool)
        .await
    }

    /// Get device fingerprint by ID
    pub async fn get_fingerprint(
        &self,
        device_id: &str,
    ) -> Result<Option<StoredDeviceFingerprint>, sqlx::Error> {
        sqlx::query_as::<_, StoredDeviceFingerprint>(
            r#"
            SELECT id, merchant_id, customer_id, device_id, user_agent, ip_address,
                   accept_language, accept_encoding, timezone, screen_resolution,
                   first_seen, last_seen, transaction_count, is_trusted
            FROM device_fingerprints
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Mark device as trusted
    pub async fn mark_trusted(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE device_fingerprints
            SET is_trusted = true
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark device as suspicious
    pub async fn mark_suspicious(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE device_fingerprints
            SET is_trusted = false
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get devices for customer
    pub async fn get_customer_devices(
        &self,
        customer_id: Uuid,
    ) -> Result<Vec<StoredDeviceFingerprint>, sqlx::Error> {
        sqlx::query_as::<_, StoredDeviceFingerprint>(
            r#"
            SELECT id, merchant_id, customer_id, device_id, user_agent, ip_address,
                   accept_language, accept_encoding, timezone, screen_resolution,
                   first_seen, last_seen, transaction_count, is_trusted
            FROM device_fingerprints
            WHERE customer_id = $1
            ORDER BY last_seen DESC
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Detect anomalies in device usage
    pub async fn detect_anomalies(
        &self,
        customer_id: Uuid,
        current_device_id: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        let mut anomalies = Vec::new();

        // Get customer's known devices
        let devices = self.get_customer_devices(customer_id).await?;

        if devices.is_empty() {
            return Ok(anomalies);
        }

        // Check if current device is known
        let is_known = devices.iter().any(|d| d.device_id == current_device_id);
        if !is_known {
            anomalies.push("New device detected".to_string());
        }

        // Check if any device is marked suspicious
        let suspicious_count = devices.iter().filter(|d| !d.is_trusted).count();
        if suspicious_count > 0 {
            anomalies.push(format!(
                "{} suspicious devices on account",
                suspicious_count
            ));
        }

        Ok(anomalies)
    }

    /// Update last seen timestamp
    pub async fn update_last_seen(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE device_fingerprints
            SET last_seen = NOW(), transaction_count = transaction_count + 1
            WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_generation() {
        let fp = DeviceFingerprintService::generate_fingerprint(
            "Mozilla/5.0",
            "192.168.1.1",
            Some("en-US"),
            Some("gzip"),
            Some("UTC"),
            Some("1920x1080"),
        );

        assert_eq!(fp.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_fingerprint_consistency() {
        let fp1 = DeviceFingerprintService::generate_fingerprint(
            "Mozilla/5.0",
            "192.168.1.1",
            Some("en-US"),
            Some("gzip"),
            Some("UTC"),
            Some("1920x1080"),
        );

        let fp2 = DeviceFingerprintService::generate_fingerprint(
            "Mozilla/5.0",
            "192.168.1.1",
            Some("en-US"),
            Some("gzip"),
            Some("UTC"),
            Some("1920x1080"),
        );

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_difference() {
        let fp1 = DeviceFingerprintService::generate_fingerprint(
            "Mozilla/5.0",
            "192.168.1.1",
            Some("en-US"),
            Some("gzip"),
            Some("UTC"),
            Some("1920x1080"),
        );

        let fp2 = DeviceFingerprintService::generate_fingerprint(
            "Mozilla/5.0",
            "192.168.1.2", // Different IP
            Some("en-US"),
            Some("gzip"),
            Some("UTC"),
            Some("1920x1080"),
        );

        assert_ne!(fp1, fp2);
    }
}

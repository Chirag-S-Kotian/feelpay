use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Blacklist service for fraud prevention.
/// Maintains blacklists of: card BINs, IP addresses, email domains.
#[derive(Clone)]
pub struct BlacklistService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistedBin {
    pub id: Uuid,
    pub bin: String, // First 6 digits of card
    pub reason: String,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistedIp {
    pub id: Uuid,
    pub ip_address: String,
    pub reason: String,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistedDomain {
    pub id: Uuid,
    pub domain: String,
    pub reason: String,
    pub added_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl BlacklistService {
    pub fn new(pool: PgPool) -> Self {
        BlacklistService { pool }
    }

    /// Add card BIN to blacklist
    pub async fn add_bin(
        &self,
        bin: &str,
        reason: &str,
        added_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<BlacklistedBin, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, BlacklistedBin>(
            r#"
            INSERT INTO blacklist_bins (id, bin, reason, added_by, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, bin, reason, added_by, created_at, expires_at
            "#,
        )
        .bind(id)
        .bind(bin)
        .bind(reason)
        .bind(added_by)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Check if BIN is blacklisted
    pub async fn is_bin_blacklisted(&self, bin: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM blacklist_bins
                WHERE bin = $1
                AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .bind(bin)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Add IP address to blacklist
    pub async fn add_ip(
        &self,
        ip_address: &str,
        reason: &str,
        added_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<BlacklistedIp, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, BlacklistedIp>(
            r#"
            INSERT INTO blacklist_ips (id, ip_address, reason, added_by, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, ip_address, reason, added_by, created_at, expires_at
            "#,
        )
        .bind(id)
        .bind(ip_address)
        .bind(reason)
        .bind(added_by)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Check if IP is blacklisted
    pub async fn is_ip_blacklisted(&self, ip_address: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM blacklist_ips
                WHERE ip_address = $1
                AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Add email domain to blacklist
    pub async fn add_domain(
        &self,
        domain: &str,
        reason: &str,
        added_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<BlacklistedDomain, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, BlacklistedDomain>(
            r#"
            INSERT INTO blacklist_domains (id, domain, reason, added_by, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, domain, reason, added_by, created_at, expires_at
            "#,
        )
        .bind(id)
        .bind(domain)
        .bind(reason)
        .bind(added_by)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Check if email domain is blacklisted
    pub async fn is_domain_blacklisted(&self, email: &str) -> Result<bool, sqlx::Error> {
        let domain = email.split('@').nth(1).unwrap_or("");

        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM blacklist_domains
                WHERE domain = $1
                AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .bind(domain)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Remove BIN from blacklist
    pub async fn remove_bin(&self, bin: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blacklist_bins WHERE bin = $1")
            .bind(bin)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Remove IP from blacklist
    pub async fn remove_ip(&self, ip_address: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blacklist_ips WHERE ip_address = $1")
            .bind(ip_address)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Remove domain from blacklist
    pub async fn remove_domain(&self, domain: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blacklist_domains WHERE domain = $1")
            .bind(domain)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List all active blacklisted BINs
    pub async fn list_bins(&self) -> Result<Vec<BlacklistedBin>, sqlx::Error> {
        sqlx::query_as::<_, BlacklistedBin>(
            r#"
            SELECT id, bin, reason, added_by, created_at, expires_at
            FROM blacklist_bins
            WHERE expires_at IS NULL OR expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// List all active blacklisted IPs
    pub async fn list_ips(&self) -> Result<Vec<BlacklistedIp>, sqlx::Error> {
        sqlx::query_as::<_, BlacklistedIp>(
            r#"
            SELECT id, ip_address, reason, added_by, created_at, expires_at
            FROM blacklist_ips
            WHERE expires_at IS NULL OR expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// List all active blacklisted domains
    pub async fn list_domains(&self) -> Result<Vec<BlacklistedDomain>, sqlx::Error> {
        sqlx::query_as::<_, BlacklistedDomain>(
            r#"
            SELECT id, domain, reason, added_by, created_at, expires_at
            FROM blacklist_domains
            WHERE expires_at IS NULL OR expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_extraction() {
        let bin = "411111";
        assert_eq!(bin.len(), 6);
    }

    #[test]
    fn test_domain_extraction() {
        let email = "user@example.com";
        let domain = email.split('@').nth(1).unwrap_or("");
        assert_eq!(domain, "example.com");
    }

    #[test]
    fn test_ip_validation() {
        let ip = "192.168.1.1";
        assert!(ip.contains('.'));
    }
}

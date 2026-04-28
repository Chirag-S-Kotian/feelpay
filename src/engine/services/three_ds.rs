use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// 3DS2 (Three Domain Secure 2.0) service for high-risk transaction authentication.
/// Enforces 3DS2 for transactions exceeding risk thresholds.
#[derive(Clone)]
pub struct ThreeDsService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreeDsStatus {
    NotRequired,
    Required,
    Authenticated,
    Failed,
    Exempted,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThreeDsChallenge {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub merchant_id: Uuid,
    pub status: String,          // authenticated, failed, pending
    pub acs_url: Option<String>, // Access Control Server URL for challenge
    pub creq: Option<String>,    // Challenge Request (base64)
    #[serde(rename = "dsTransID")]
    #[sqlx(rename = "dsTransID")]
    pub ds_trans_id: Option<String>, // Directory Server Transaction ID
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ThreeDsService {
    pub fn new(pool: PgPool) -> Self {
        ThreeDsService { pool }
    }

    /// Assess risk level for transaction
    pub fn assess_risk(
        &self,
        amount: i64,
        is_new_card: bool,
        is_new_customer: bool,
        is_high_risk_country: bool,
    ) -> RiskLevel {
        let mut risk_score = 0;

        // Amount-based risk
        if amount > 100000 {
            risk_score += 3; // 1000 INR
        } else if amount > 50000 {
            risk_score += 2;
        } else if amount > 10000 {
            risk_score += 1;
        }

        // Card/customer risk
        if is_new_card {
            risk_score += 2;
        }
        if is_new_customer {
            risk_score += 2;
        }
        if is_high_risk_country {
            risk_score += 3;
        }

        match risk_score {
            0..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            _ => RiskLevel::High,
        }
    }

    /// Determine if 3DS2 is required
    pub fn should_require_3ds(&self, risk_level: &RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::High)
    }

    /// Create 3DS2 challenge
    pub async fn create_challenge(
        &self,
        payment_id: Uuid,
        merchant_id: Uuid,
        acs_url: &str,
        creq: &str,
        ds_trans_id: &str,
    ) -> Result<ThreeDsChallenge, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, ThreeDsChallenge>(
            r#"
            INSERT INTO three_ds_challenges (id, payment_id, merchant_id, status, acs_url, creq, dsTransID)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, payment_id, merchant_id, status, acs_url, creq, dsTransID, created_at, completed_at
            "#,
        )
        .bind(id)
        .bind(payment_id)
        .bind(merchant_id)
        .bind("pending")
        .bind(acs_url)
        .bind(creq)
        .bind(ds_trans_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Mark challenge as authenticated
    pub async fn mark_authenticated(
        &self,
        challenge_id: Uuid,
    ) -> Result<ThreeDsChallenge, sqlx::Error> {
        sqlx::query_as::<_, ThreeDsChallenge>(
            r#"
            UPDATE three_ds_challenges
            SET status = $1, completed_at = NOW()
            WHERE id = $2
            RETURNING id, payment_id, merchant_id, status, acs_url, creq, dsTransID, created_at, completed_at
            "#,
        )
        .bind("authenticated")
        .bind(challenge_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Mark challenge as failed
    pub async fn mark_failed(&self, challenge_id: Uuid) -> Result<ThreeDsChallenge, sqlx::Error> {
        sqlx::query_as::<_, ThreeDsChallenge>(
            r#"
            UPDATE three_ds_challenges
            SET status = $1, completed_at = NOW()
            WHERE id = $2
            RETURNING id, payment_id, merchant_id, status, acs_url, creq, dsTransID, created_at, completed_at
            "#,
        )
        .bind("failed")
        .bind(challenge_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get challenge by payment ID
    pub async fn get_challenge_by_payment(
        &self,
        payment_id: Uuid,
    ) -> Result<Option<ThreeDsChallenge>, sqlx::Error> {
        sqlx::query_as::<_, ThreeDsChallenge>(
            r#"
            SELECT id, payment_id, merchant_id, status, acs_url, creq, dsTransID, created_at, completed_at
            FROM three_ds_challenges
            WHERE payment_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(payment_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Verify 3DS2 authentication result
    pub async fn verify_authentication(
        &self,
        challenge_id: Uuid,
        _cres: &str, // Challenge Response (base64)
    ) -> Result<bool, String> {
        // In production, verify CRES signature with issuer's public key
        // For now, stub implementation
        tracing::info!("Verifying 3DS2 CRES for challenge {}", challenge_id);

        // Mark as authenticated
        self.mark_authenticated(challenge_id)
            .await
            .map_err(|e| format!("Failed to mark challenge authenticated: {}", e))?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_assessment_low() {
        let service = ThreeDsService::new(sqlx::PgPool::connect("").await.unwrap());
        let risk = service.assess_risk(5000, false, false, false);
        assert!(matches!(risk, RiskLevel::Low));
    }

    #[test]
    fn test_risk_assessment_high() {
        let service = ThreeDsService::new(sqlx::PgPool::connect("").await.unwrap());
        let risk = service.assess_risk(150000, true, true, true);
        assert!(matches!(risk, RiskLevel::High));
    }

    #[test]
    fn test_3ds_requirement() {
        let service = ThreeDsService::new(sqlx::PgPool::connect("").await.unwrap());
        assert!(service.should_require_3ds(&RiskLevel::High));
        assert!(!service.should_require_3ds(&RiskLevel::Low));
    }
}

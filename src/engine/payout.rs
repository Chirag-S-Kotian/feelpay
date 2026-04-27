use crate::error::FeelPayError;
use crate::models::payout::Payout;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn process_payout(pool: &PgPool, payout_id: Uuid) -> Result<(), FeelPayError> {
    // 1. Fetch payout
    let payout = sqlx::query_as::<_, Payout>("SELECT * FROM payouts WHERE id = $1")
        .bind(payout_id)
        .fetch_one(pool)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    if payout.status != "created" && payout.status != "pending" {
        return Err(FeelPayError::BadRequest(
            "Payout already processed or in progress".into(),
        ));
    }

    // 2. Update status to 'processing'
    sqlx::query("UPDATE payouts SET status = 'processing' WHERE id = $1")
        .bind(payout_id)
        .execute(pool)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    // 3. Simulate Gateway Interaction (IMPS/NEFT)
    let success = true;
    let gateway_ref = format!(
        "TXN_{}",
        Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap()
            .to_uppercase()
    );

    if success {
        sqlx::query("UPDATE payouts SET status = 'success', gateway_reference = $2, processed_at = NOW() WHERE id = $1")
            .bind(payout_id)
            .bind(gateway_ref)
            .execute(pool)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    } else {
        sqlx::query("UPDATE payouts SET status = 'failed', error_message = 'Simulation: Gateway timeout' WHERE id = $1")
            .bind(payout_id)
            .execute(pool)
            .await
            .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;
    }

    Ok(())
}

pub async fn verify_bank_account(
    pool: &PgPool,
    bank_account_id: Uuid,
) -> Result<bool, FeelPayError> {
    // Penny Drop Simulation

    let row = sqlx::query("SELECT merchant_id FROM bank_accounts WHERE id = $1")
        .bind(bank_account_id)
        .fetch_one(pool)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    let merchant_id: Uuid = row
        .try_get("merchant_id")
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let payout_id = Uuid::new_v4();

    // Create verification payout
    sqlx::query("INSERT INTO payouts (id, merchant_id, amount, currency, status, method, bank_account_id, metadata)
         VALUES ($1, $2, 100, 'INR', 'pending', 'imps', $3, $4)")
        .bind(payout_id)
        .bind(merchant_id)
        .bind(bank_account_id)
        .bind(serde_json::json!({"type": "verification", "purpose": "penny_drop"}))
        .execute(pool)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    // Simulate instant success for penny drop
    sqlx::query("UPDATE bank_accounts SET is_verified = true WHERE id = $1")
        .bind(bank_account_id)
        .execute(pool)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    sqlx::query("UPDATE payouts SET status = 'success', processed_at = NOW() WHERE id = $1")
        .bind(payout_id)
        .execute(pool)
        .await
        .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    Ok(true)
}

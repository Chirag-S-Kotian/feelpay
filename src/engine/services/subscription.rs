use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Cancelled,
    Expired,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BillingCycle {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub plan_name: String,
    pub amount: i64,
    pub currency: String,
    pub billing_cycle: String,
    pub status: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub retry_settings: Option<serde_json::Value>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub failed_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub name: String,
    pub amount: i64,
    pub currency: String,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionInvoice {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub merchant_id: Uuid,
    pub customer_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub payment_order_id: Option<Uuid>,
    pub invoice_number: String,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub customer_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub plan_name: Option<String>,
    pub amount: i64,
    pub currency: Option<String>,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub billing_cycle: String,
    pub trial_days: Option<i32>,
}

/// Create a subscription plan
pub async fn create_plan(
    pool: &PgPool,
    merchant_id: Uuid,
    req: CreatePlanRequest,
) -> Result<SubscriptionPlan, sqlx::Error> {
    let id = Uuid::new_v4();
    let currency = req.currency.unwrap_or_else(|| "INR".to_string());
    
    let plan: SubscriptionPlan = sqlx::query_as(
        r#"
        INSERT INTO subscription_plans (id, merchant_id, name, amount, currency, billing_cycle, trial_days, active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(merchant_id)
    .bind(&req.name)
    .bind(req.amount)
    .bind(&currency)
    .bind(&req.billing_cycle)
    .bind(req.trial_days)
    .fetch_one(pool)
    .await?;
    
    Ok(plan)
}

/// List subscription plans for merchant
pub async fn list_plans(
    pool: &PgPool,
    merchant_id: Uuid,
) -> Result<Vec<SubscriptionPlan>, sqlx::Error> {
    let plans = sqlx::query_as(
        "SELECT * FROM subscription_plans WHERE merchant_id = $1 AND active = true ORDER BY created_at DESC"
    )
    .bind(merchant_id)
    .fetch_all(pool)
    .await?;
    
    Ok(plans)
}

/// Create a subscription
pub async fn create_subscription(
    pool: &PgPool,
    merchant_id: Uuid,
    req: CreateSubscriptionRequest,
) -> Result<Subscription, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let (plan_id, plan_name, amount, currency, billing_cycle, trial_days) =
        if let Some(plan_id) = req.plan_id {
            let plan = sqlx::query_as::<_, SubscriptionPlan>(
                "SELECT * FROM subscription_plans WHERE id = $1 AND merchant_id = $2 AND active = true",
            )
            .bind(plan_id)
            .bind(merchant_id)
            .fetch_optional(pool)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

            (
                Some(plan_id),
                plan.name,
                plan.amount,
                plan.currency,
                plan.billing_cycle,
                plan.trial_days,
            )
        } else {
            (
                None,
                req.plan_name.unwrap_or_else(|| "Custom Plan".to_string()),
                req.amount,
                req.currency.unwrap_or_else(|| "INR".to_string()),
                req.billing_cycle,
                req.trial_days,
            )
        };
    
    // Calculate period dates based on billing cycle
    let (period_start, period_end) = calculate_period(&billing_cycle, req.start_date.unwrap_or(now));
    
    // Calculate trial dates if applicable
    let (trial_start, trial_end) = if let Some(days) = trial_days {
        let start = period_start;
        let end = start + chrono::Duration::days(days as i64);
        (Some(start), Some(end))
    } else {
        (None, None)
    };
    
    let subscription: Subscription = sqlx::query_as(
        r#"
        INSERT INTO subscriptions (
            id, merchant_id, customer_id, plan_id, plan_name, amount, currency, billing_cycle,
            status, current_period_start, current_period_end, trial_start, trial_end,
            cancel_at_period_end, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active', $9, $10, $11, $12, false, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(merchant_id)
    .bind(req.customer_id)
    .bind(plan_id)
    .bind(&plan_name)
    .bind(amount)
    .bind(&currency)
    .bind(&billing_cycle)
    .bind(period_start)
    .bind(period_end)
    .bind(trial_start)
    .bind(trial_end)
    .fetch_one(pool)
    .await?;
    
    Ok(subscription)
}

/// Get subscription by ID
pub async fn get_subscription(
    pool: &PgPool,
    merchant_id: Uuid,
    subscription_id: Uuid,
) -> Result<Option<Subscription>, sqlx::Error> {
    let sub = sqlx::query_as(
        "SELECT * FROM subscriptions WHERE id = $1 AND merchant_id = $2"
    )
    .bind(subscription_id)
    .bind(merchant_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(sub)
}

/// List subscriptions for merchant
pub async fn list_subscriptions(
    pool: &PgPool,
    merchant_id: Uuid,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Subscription>, sqlx::Error> {
    let subs = if let Some(s) = status {
        sqlx::query_as(
            "SELECT * FROM subscriptions WHERE merchant_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        )
        .bind(merchant_id)
        .bind(s)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM subscriptions WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };
    
    Ok(subs)
}

/// Pause a subscription
pub async fn pause_subscription(
    pool: &PgPool,
    merchant_id: Uuid,
    subscription_id: Uuid,
) -> Result<Subscription, sqlx::Error> {
    let sub: Subscription = sqlx::query_as(
        r#"
        UPDATE subscriptions
        SET status = 'paused', updated_at = NOW()
        WHERE id = $1 AND merchant_id = $2 AND status = 'active'
        RETURNING *
        "#,
    )
    .bind(subscription_id)
    .bind(merchant_id)
    .fetch_one(pool)
    .await?;
    
    Ok(sub)
}

/// Resume a paused subscription
pub async fn resume_subscription(
    pool: &PgPool,
    merchant_id: Uuid,
    subscription_id: Uuid,
) -> Result<Subscription, sqlx::Error> {
    let sub: Subscription = sqlx::query_as(
        r#"
        UPDATE subscriptions
        SET status = 'active', updated_at = NOW()
        WHERE id = $1 AND merchant_id = $2 AND status = 'paused'
        RETURNING *
        "#,
    )
    .bind(subscription_id)
    .bind(merchant_id)
    .fetch_one(pool)
    .await?;
    
    Ok(sub)
}

/// Cancel a subscription
pub async fn cancel_subscription(
    pool: &PgPool,
    merchant_id: Uuid,
    subscription_id: Uuid,
    cancel_at_period_end: bool,
) -> Result<Subscription, sqlx::Error> {
    let sub: Subscription = if cancel_at_period_end {
        sqlx::query_as(
            r#"
            UPDATE subscriptions
            SET cancel_at_period_end = true, updated_at = NOW()
            WHERE id = $1 AND merchant_id = $2 AND status IN ('active', 'paused')
            RETURNING *
            "#,
        )
        .bind(subscription_id)
        .bind(merchant_id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"
            UPDATE subscriptions
            SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND merchant_id = $2 AND status IN ('active', 'paused')
            RETURNING *
            "#,
        )
        .bind(subscription_id)
        .bind(merchant_id)
        .fetch_one(pool)
        .await?
    };
    
    Ok(sub)
}

/// Generate invoice for subscription
pub async fn generate_invoice(
    pool: &PgPool,
    subscription_id: Uuid,
) -> Result<SubscriptionInvoice, sqlx::Error> {
    let id = Uuid::new_v4();
    let invoice_number = format!("INV-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
    
    // Get subscription details
    let sub: Subscription = sqlx::query_as(
        "SELECT * FROM subscriptions WHERE id = $1"
    )
    .bind(subscription_id)
    .fetch_one(pool)
    .await?;
    
    let invoice: SubscriptionInvoice = sqlx::query_as(
        r#"
        INSERT INTO subscription_invoices (
            id, subscription_id, merchant_id, customer_id, amount, currency,
            status, invoice_number, due_date, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, NOW())
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(subscription_id)
    .bind(sub.merchant_id)
    .bind(sub.customer_id)
    .bind(sub.amount)
    .bind(&sub.currency)
    .bind(&invoice_number)
    .bind(sub.current_period_end)
    .fetch_one(pool)
    .await?;
    
    Ok(invoice)
}

/// List invoices for subscription
pub async fn list_invoices(
    pool: &PgPool,
    merchant_id: Uuid,
    subscription_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<SubscriptionInvoice>, sqlx::Error> {
    let invoices = if let Some(sub_id) = subscription_id {
        sqlx::query_as(
            "SELECT * FROM subscription_invoices WHERE merchant_id = $1 AND subscription_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        )
        .bind(merchant_id)
        .bind(sub_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM subscription_invoices WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };
    
    Ok(invoices)
}

/// Calculate billing period dates
fn calculate_period(billing_cycle: &str, start: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let end = match billing_cycle.to_lowercase().as_str() {
        "daily" => start + chrono::Duration::days(1),
        "weekly" => start + chrono::Duration::weeks(1),
        "monthly" => start + chrono::Duration::days(30),
        "quarterly" => start + chrono::Duration::days(90),
        "yearly" => start + chrono::Duration::days(365),
        _ => start + chrono::Duration::days(30),
    };
    
    (start, end)
}

/// Process due subscriptions (called by scheduler)
pub async fn process_due_subscriptions(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
    let now = Utc::now();
    
    // Find subscriptions that need renewal
    let due_subs: Vec<Subscription> = sqlx::query_as(
        r#"
        SELECT * FROM subscriptions
        WHERE status = 'active'
          AND current_period_end <= $1
          AND cancel_at_period_end = false
        "#,
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    
    let mut processed = Vec::new();
    
    for sub in due_subs {
        // Generate invoice for next period
        match generate_invoice(pool, sub.id).await {
            Ok(_) => {
                // Update period dates
                let (new_start, new_end) = calculate_period(&sub.billing_cycle, now);
                
                sqlx::query(
                    r#"
                    UPDATE subscriptions
                    SET current_period_start = $1, current_period_end = $2, updated_at = NOW()
                    WHERE id = $3
                    "#,
                )
                .bind(new_start)
                .bind(new_end)
                .bind(sub.id)
                .execute(pool)
                .await?;
                
                processed.push(sub.id);
            }
            Err(e) => {
                tracing::error!("Failed to generate invoice for subscription {}: {}", sub.id, e);
            }
        }
    }
    
    Ok(processed)
}

/// Record a payment failure and schedule retry
pub async fn record_payment_failure(
    pool: &PgPool,
    subscription_id: Uuid,
) -> Result<(), sqlx::Error> {
    // Increment failed attempts and calculate next retry (e.g., in 24 hours)
    let next_retry = Utc::now() + chrono::Duration::hours(24);

    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET failed_attempts = failed_attempts + 1,
            next_retry_at = $1,
            updated_at = NOW()
        WHERE id = $2
        "#,
        next_retry,
        subscription_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Reset retries on successful payment
pub async fn record_payment_success(
    pool: &PgPool,
    subscription_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET failed_attempts = 0,
            next_retry_at = NULL,
            updated_at = NOW()
        WHERE id = $1
        "#,
        subscription_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

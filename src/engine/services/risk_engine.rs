use crate::error::FeelPayError;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RiskEvent {
    pub scope: &'static str,
    pub environment: &'static str,
    pub entity_type: &'static str,
    pub entity_id: Option<Uuid>,
    pub merchant_id: Option<Uuid>,
    pub context: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskDisposition {
    Allow,
    Review,
    Block,
    HoldPayout,
    RestrictMerchant,
}

impl RiskDisposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Review => "review",
            Self::Block => "block",
            Self::HoldPayout => "hold_payout",
            Self::RestrictMerchant => "restrict_merchant",
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Self::Allow => 0,
            Self::Review => 1,
            Self::HoldPayout => 2,
            Self::RestrictMerchant => 3,
            Self::Block => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskOutcome {
    pub disposition: RiskDisposition,
    pub matched_rule_ids: Vec<Uuid>,
    pub reasons: Vec<String>,
}

pub async fn evaluate_and_record(
    pool: &PgPool,
    event: RiskEvent,
) -> Result<RiskOutcome, FeelPayError> {
    let rules = sqlx::query(
        r#"
        SELECT id, name, rule_type, action, severity, condition_config
        FROM risk_rules
        WHERE scope = $1
          AND environment = $2
          AND enabled = true
        ORDER BY priority ASC, created_at ASC
        "#,
    )
    .bind(event.scope)
    .bind(event.environment)
    .fetch_all(pool)
    .await
    .map_err(|e| FeelPayError::Database(e.to_string()))?;

    let mut disposition = RiskDisposition::Allow;
    let mut matched_rule_ids = Vec::new();
    let mut reasons = Vec::new();

    for rule in rules {
        let rule_id: Uuid = rule.get("id");
        let rule_name: String = rule.get("name");
        let rule_type: String = rule.get("rule_type");
        let action: String = rule.get("action");
        let severity: String = rule.get("severity");
        let config: Value = rule.get("condition_config");
        let matched = rule_matches(&rule_type, &config, &event.context);
        let reason = if matched {
            format!("{rule_name} matched")
        } else {
            format!("{rule_name} not matched")
        };
        let eval_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO risk_evaluations (
                rule_id, merchant_id, scope, environment, entity_type, entity_id,
                matched, action, severity, reason, input_summary_redacted
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING id
            "#,
        )
        .bind(rule_id)
        .bind(event.merchant_id)
        .bind(event.scope)
        .bind(event.environment)
        .bind(event.entity_type)
        .bind(event.entity_id)
        .bind(matched)
        .bind(&action)
        .bind(&severity)
        .bind(&reason)
        .bind(redacted_summary(&event.context))
        .fetch_one(pool)
        .await
        .map_err(|e| FeelPayError::Database(e.to_string()))?;

        if matched {
            sqlx::query(
                r#"
                INSERT INTO risk_signals (
                    business_account_id, signal_type, signal_value, severity,
                    evaluation_id, rule_id, scope, entity_type, entity_id, signal_payload_redacted
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                "#,
            )
            .bind(event.merchant_id)
            .bind(&rule_type)
            .bind(&rule_name)
            .bind(&severity)
            .bind(eval_id)
            .bind(rule_id)
            .bind(event.scope)
            .bind(event.entity_type)
            .bind(event.entity_id)
            .bind(redacted_summary(&event.context))
            .execute(pool)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;

            let next = disposition_from_action(&action);
            if next.rank() > disposition.rank() {
                disposition = next;
            }
            matched_rule_ids.push(rule_id);
            reasons.push(reason.clone());

            if !matches!(action.as_str(), "allow") {
                let case_id = sqlx::query_scalar::<_, Uuid>(
                    r#"
                    INSERT INTO risk_cases (
                        merchant_id, primary_evaluation_id, scope, entity_type, entity_id,
                        severity, action, reason
                    )
                    VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                    RETURNING id
                    "#,
                )
                .bind(event.merchant_id)
                .bind(eval_id)
                .bind(event.scope)
                .bind(event.entity_type)
                .bind(event.entity_id)
                .bind(&severity)
                .bind(&action)
                .bind(&reason)
                .fetch_one(pool)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;

                sqlx::query(
                    "INSERT INTO risk_action_history (case_id, evaluation_id, action, to_status, note) VALUES ($1,$2,$3,'open',$4)",
                )
                .bind(case_id)
                .bind(eval_id)
                .bind(&action)
                .bind(&reason)
                .execute(pool)
                .await
                .map_err(|e| FeelPayError::Database(e.to_string()))?;
            }

            sqlx::query(
                "UPDATE risk_rules SET trigger_count = trigger_count + 1, last_triggered_at = NOW() WHERE id = $1",
            )
            .bind(rule_id)
            .execute(pool)
            .await
            .map_err(|e| FeelPayError::Database(e.to_string()))?;
        }
    }

    Ok(RiskOutcome {
        disposition,
        matched_rule_ids,
        reasons,
    })
}

fn disposition_from_action(action: &str) -> RiskDisposition {
    match action {
        "review" => RiskDisposition::Review,
        "block" => RiskDisposition::Block,
        "hold_payout" => RiskDisposition::HoldPayout,
        "restrict_merchant" => RiskDisposition::RestrictMerchant,
        _ => RiskDisposition::Allow,
    }
}

fn rule_matches(rule_type: &str, config: &Value, context: &Value) -> bool {
    match rule_type {
        "high_payment_amount" | "payout_amount_above_threshold" => {
            int(context, "amount_paise") >= int(config, "threshold_paise")
        }
        "high_daily_volume" => int(context, "daily_volume_paise") >= int(config, "threshold_paise"),
        "high_failure_rate" => int(context, "failure_rate_bps") >= int(config, "threshold_bps"),
        "high_refund_rate" => int(context, "refund_rate_bps") >= int(config, "threshold_bps"),
        "risky_business_category" => list(config, "categories")
            .iter()
            .any(|value| Some(value.as_str()) == text(context, "business_category")),
        "kyc_mismatch_manual_review" => boolean(context, "kyc_mismatch"),
        "payout_to_unverified_bank" => !boolean(context, "bank_verified"),
        "duplicate_reference" => boolean(context, "duplicate_reference"),
        "dispute_count_threshold" => {
            int(context, "dispute_count") >= int(config, "threshold_count")
        }
        "reconciliation_mismatch_threshold" => {
            int(context, "mismatch_count") >= int(config, "threshold_count")
        }
        "newly_approved_merchant_large_payout" => {
            int(context, "days_since_approval") <= int(config, "max_days")
                && int(context, "amount_paise") >= int(config, "threshold_paise")
        }
        _ => false,
    }
}

fn int(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn boolean(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn list(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn redacted_summary(context: &Value) -> Value {
    json!({
        "amount_paise": context.get("amount_paise"),
        "daily_volume_paise": context.get("daily_volume_paise"),
        "failure_rate_bps": context.get("failure_rate_bps"),
        "refund_rate_bps": context.get("refund_rate_bps"),
        "business_category": context.get("business_category"),
        "kyc_mismatch": context.get("kyc_mismatch"),
        "bank_verified": context.get("bank_verified"),
        "duplicate_reference": context.get("duplicate_reference"),
        "dispute_count": context.get("dispute_count"),
        "mismatch_count": context.get("mismatch_count"),
        "days_since_approval": context.get("days_since_approval")
    })
}

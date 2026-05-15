# Risk Engine

## Model

The internal foundation uses `risk_rules`, `risk_evaluations`, `risk_signals`, `risk_cases`, `risk_case_notes`, and `risk_action_history`.

Rules are scoped to `merchant`, `payment`, `refund`, `dispute`, `payout`, or `settlement`, and are environment-specific for `test` or `live`.

Supported actions are `allow`, `review`, `block`, `hold_payout`, and `restrict_merchant`.

## Safe Conditions

Rules use bounded JSON config only. The evaluator supports fixed rule types and scalar/list inputs; it does not execute code, expressions, scripts, or dynamic SQL.

Initial rule types:

- `high_payment_amount`
- `high_daily_volume`
- `high_failure_rate`
- `high_refund_rate`
- `risky_business_category`
- `kyc_mismatch_manual_review`
- `payout_amount_above_threshold`
- `payout_to_unverified_bank`
- `duplicate_reference`
- `dispute_count_threshold`
- `reconciliation_mismatch_threshold`
- `newly_approved_merchant_large_payout`

Money thresholds are integer paise only. Rate thresholds are integer basis points.

## Event Flow

Evaluations are produced from merchant review, payment create/capture, refund creation, dispute updates, settlement actions, payout approval/processing, and reconciliation mismatch imports.

Each evaluation stores its entity reference, redacted input summary, result, matched action, severity, and reason. Matching non-allow rules create auditable cases and action-history rows.

## Action Behavior

- `allow`: records evaluation only.
- `review`: creates an internal case for admin review.
- `block`: can fail closed for high-risk live payment and payout actions.
- `hold_payout`: keeps payout approval on hold pending internal review.
- `restrict_merchant`: creates an internal case for merchant restriction review; restriction itself remains an explicit admin action.

Historical settled records are not retroactively changed by later rule edits.

## Boundaries

- Merchant users cannot access internal rules, evaluations, notes, or cases.
- Payout holds are internal lifecycle state, not proof of a live bank action.
- The engine produces evidence-backed signals; it does not claim fraud certainty.
- Bank/provider APIs remain outside this foundation.

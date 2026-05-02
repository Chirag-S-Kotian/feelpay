# Payment Success Checkout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make FeelPay payment success, processing, and failure states work predictably across test/live modes, hosted checkout sessions, and merchant dashboard badges.

**Architecture:** Keep existing Actix + sqlx + PaymentEngine architecture. Add small deterministic helpers for mode/connector/status metadata, persist processor details on payment orders, add checkout session payment processing as a thin wrapper around `/v1/payments` behavior, and upgrade dashboard checkout preview/payments UI to show Razorpay/Stripe-like environment and processing badges. Avoid schema churn unless current columns cannot represent required state.

**Tech Stack:** Rust, Actix Web, sqlx, PostgreSQL, Redis, React 19, Vite, TypeScript, Vitest, Playwright.

---

## File Structure

- Modify `src/api/payments.rs`: add helper functions for method labels, connector resolution, status timestamps, persisted metadata; fix create/capture behavior.
- Modify `src/engine/payment.rs`: make test-mode override explicit through caller rather than hidden method routing changes, or expose connector resolution helper only if needed.
- Modify `src/engine/processors/simulator.rs`: support deterministic simulator action success/fail/processing for test flows while preserving default behavior.
- Modify `src/api/checkout.rs`: add `POST /v1/checkout/session/{id}/pay`, update Redis session after payment, return normalized checkout status.
- Modify `src/main.rs`: register checkout pay route.
- Modify `dashboard/src/pages/CheckoutPreview.tsx`: call backend checkout/payment flow in preview; add Test/Live badge, Processing badge, Authorized/Captured success card, failure state.
- Modify `dashboard/src/pages/Payments.tsx`: add polished method/connector/status badges and amount formatting correction if needed.
- Add `dashboard/src/pages/CheckoutPreview.test.tsx`: UI behavior for payment processing badges and success/error rendering.
- Add focused Rust unit tests near helper functions in `src/api/payments.rs` and `src/engine/processors/simulator.rs`.

## Root Cause Summary

Current payment processing is partly real, partly visual mock:

- `create_payment` calls `PaymentEngine::authorize`, then inserts a row without method, connector, metadata, idempotency key, authorized/captured timestamp fields.
- `capture_payment` ignores stored connector and always captures through `"simulator"`.
- Hosted checkout sessions can be created and polled, but no endpoint turns a checkout session into a payment or updates `status`.
- Dashboard checkout preview uses `setTimeout` for success, so it can show success even if backend payment is broken.
- Simulator has random 90% success, bad for deterministic test payments.

## Task 1: Deterministic Payment Helper Tests

**Files:**
- Modify: `src/api/payments.rs`
- Modify: `src/engine/processors/simulator.rs`

- [ ] **Step 1: Add failing helper tests in `src/api/payments.rs`**

Add this test module at bottom of `src/api/payments.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::processor::MethodData;
    use crate::models::payment_order::PaymentStatus;

    #[test]
    fn method_name_from_method_data_maps_supported_methods() {
        assert_eq!(
            payment_method_name(&MethodData::Card {
                number: None,
                token: Some("tok_123".into()),
                expiry_month: None,
                expiry_year: None,
                cvv: None,
                holder_name: None,
            }),
            "card"
        );
        assert_eq!(
            payment_method_name(&MethodData::Upi {
                vpa: Some("buyer@upi".into()),
                flow: Some("collect".into()),
            }),
            "upi"
        );
        assert_eq!(
            payment_method_name(&MethodData::Simulator {
                action: "success".into(),
            }),
            "simulator"
        );
    }

    #[test]
    fn connector_for_request_prefers_override_then_test_simulator() {
        assert_eq!(
            connector_for_payment(
                &MethodData::Card {
                    number: None,
                    token: Some("tok_123".into()),
                    expiry_month: None,
                    expiry_year: None,
                    cvv: None,
                    holder_name: None,
                },
                Some("card"),
                false,
            ),
            "card"
        );
        assert_eq!(
            connector_for_payment(
                &MethodData::Upi {
                    vpa: Some("buyer@upi".into()),
                    flow: Some("collect".into()),
                },
                None,
                false,
            ),
            "simulator"
        );
        assert_eq!(
            connector_for_payment(
                &MethodData::Upi {
                    vpa: Some("buyer@upi".into()),
                    flow: Some("collect".into()),
                },
                None,
                true,
            ),
            "upi"
        );
    }

    #[test]
    fn status_timestamps_mark_authorized_and_captured() {
        let now = chrono::Utc::now();
        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Authorized, now);
        assert_eq!(authorized_at, Some(now));
        assert_eq!(captured_at, None);

        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Captured, now);
        assert_eq!(authorized_at, Some(now));
        assert_eq!(captured_at, Some(now));

        let (authorized_at, captured_at) = status_timestamps(&PaymentStatus::Failed, now);
        assert_eq!(authorized_at, None);
        assert_eq!(captured_at, None);
    }
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
cargo test api::payments::tests::method_name_from_method_data_maps_supported_methods api::payments::tests::connector_for_request_prefers_override_then_test_simulator api::payments::tests::status_timestamps_mark_authorized_and_captured
```

Expected: fail because `payment_method_name`, `connector_for_payment`, and `status_timestamps` do not exist.

- [ ] **Step 3: Implement minimal helpers in `src/api/payments.rs`**

Add near `request_hash`:

```rust
fn payment_method_name(method_data: &MethodData) -> &'static str {
    match method_data {
        MethodData::Card { .. } => "card",
        MethodData::Upi { .. } => "upi",
        MethodData::NetBanking { .. } => "netbanking",
        MethodData::Wallet { .. } => "wallet",
        MethodData::Emi { .. } => "emi",
        MethodData::Bnpl { .. } => "bnpl",
        MethodData::Simulator { .. } => "simulator",
    }
}

fn connector_for_payment(
    method_data: &MethodData,
    processor_override: Option<&str>,
    is_live: bool,
) -> String {
    if let Some(connector) = processor_override.filter(|value| !value.trim().is_empty()) {
        return connector.to_string();
    }
    if !is_live {
        return "simulator".to_string();
    }
    payment_method_name(method_data).to_string()
}

fn status_timestamps(
    status: &PaymentStatus,
    now: chrono::DateTime<chrono::Utc>,
) -> (Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>) {
    match status {
        PaymentStatus::Authorized => (Some(now), None),
        PaymentStatus::Captured => (Some(now), Some(now)),
        _ => (None, None),
    }
}
```

- [ ] **Step 4: Run tests and verify GREEN**

Run:

```bash
cargo test api::payments::tests::method_name_from_method_data_maps_supported_methods api::payments::tests::connector_for_request_prefers_override_then_test_simulator api::payments::tests::status_timestamps_mark_authorized_and_captured
```

Expected: pass.

## Task 2: Persist Payment Method, Connector, Timestamps

**Files:**
- Modify: `src/api/payments.rs`

- [ ] **Step 1: Add failing helper test for response metadata**

Add to existing `#[cfg(test)] mod tests` in `src/api/payments.rs`:

```rust
#[test]
fn checkout_metadata_includes_environment_and_method() {
    let metadata = payment_metadata("test", "card", "simulator");
    assert_eq!(metadata["environment"], "test");
    assert_eq!(metadata["payment_method"], "card");
    assert_eq!(metadata["connector"], "simulator");
}
```

- [ ] **Step 2: Run test and verify RED**

Run:

```bash
cargo test api::payments::tests::checkout_metadata_includes_environment_and_method
```

Expected: fail because `payment_metadata` does not exist.

- [ ] **Step 3: Add metadata helper**

Add near helper functions:

```rust
fn payment_metadata(environment: &str, method: &str, connector: &str) -> serde_json::Value {
    serde_json::json!({
        "environment": environment,
        "payment_method": method,
        "connector": connector,
        "mode_badge": if environment == "live" { "LIVE" } else { "TEST" }
    })
}
```

- [ ] **Step 4: Update `create_payment` processing path**

Inside `create_payment`, before engine call:

```rust
let method_name = payment_method_name(&final_method_data).to_string();
let connector = connector_for_payment(
    &final_method_data,
    req_data.processor_override.as_deref(),
    state.config.is_live(),
);
let environment = if state.config.is_live() { "live" } else { "test" };
```

Replace engine call with:

```rust
let proc_result = engine
    .authorize(&txn_ctx, &final_method_data, Some(&connector))
    .await;
```

When response is successful, set:

```rust
let now = chrono::Utc::now();
let (authorized_at, captured_at) = status_timestamps(&order.status, now);
order.payment_method = Some(method_name.clone());
order.payment_method_data = Some(serde_json::to_value(&final_method_data).unwrap_or_else(|_| serde_json::json!({})));
order.connector = Some(connector.clone());
order.metadata = Some(payment_metadata(environment, &method_name, &connector));
order.authorized_at = authorized_at;
order.captured_at = captured_at;
order.updated_at = now;
```

Replace insert SQL with full column insert:

```rust
INSERT INTO payment_orders (
    id, merchant_id, amount, currency, status, order_id,
    payment_method, payment_method_data, connector, customer_email, customer_phone,
    description, return_url, idempotency_key, gateway_reference, gateway_response,
    error_message, error_code, metadata, refunded_amount, created_at, updated_at,
    authorized_at, captured_at
)
VALUES (
    $1, $2, $3, $4, $5, $6,
    $7, $8, $9, $10, $11,
    $12, $13, $14, $15, $16,
    $17, $18, $19, $20, $21, $22,
    $23, $24
)
```

Bind exact fields in same order.

- [ ] **Step 5: Run helper test and cargo check**

Run:

```bash
cargo test api::payments::tests::checkout_metadata_includes_environment_and_method
cargo check
```

Expected: both pass.

## Task 3: Deterministic Simulator Actions

**Files:**
- Modify: `src/engine/processors/simulator.rs`

- [ ] **Step 1: Add failing tests**

Add bottom of `src/engine/processors/simulator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn txn() -> TransactionContext {
        TransactionContext {
            payment_id: Uuid::new_v4(),
            merchant_id: Uuid::new_v4(),
            order_id: "order_test_123".into(),
            amount: 12345,
            currency: "INR".into(),
            customer_email: Some("buyer@example.com".into()),
            customer_phone: None,
            return_url: None,
            description: Some("Test order".into()),
        }
    }

    #[tokio::test]
    async fn simulator_success_action_captures_payment() {
        let processor = SimulatorProcessor::new();
        let response = processor
            .authorize(&txn(), &MethodData::Simulator { action: "success".into() })
            .await
            .unwrap();
        assert_eq!(response.status, PaymentStatus::Captured);
        assert!(response.error_code.is_none());
    }

    #[tokio::test]
    async fn simulator_fail_action_declines_payment() {
        let processor = SimulatorProcessor::new();
        let response = processor
            .authorize(&txn(), &MethodData::Simulator { action: "fail".into() })
            .await
            .unwrap();
        assert_eq!(response.status, PaymentStatus::Failed);
        assert_eq!(response.error_code.as_deref(), Some("SIM_DECLINED"));
    }
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
cargo test engine::processors::simulator::tests::simulator_success_action_captures_payment engine::processors::simulator::tests::simulator_fail_action_declines_payment
```

Expected: at least fail action should fail or be random.

- [ ] **Step 3: Implement deterministic action handling**

At top of `authorize`, after sleep:

```rust
if let MethodData::Simulator { action } = method_data {
    match action.as_str() {
        "success" => {
            return Ok(ProcessorResponse {
                status: PaymentStatus::Captured,
                gateway_reference: Some(self.generate_ref("pay")),
                gateway_response: Some(serde_json::json!({
                    "simulator": true,
                    "action": "success",
                    "amount": txn.amount
                })),
                error_code: None,
                error_message: None,
                redirect_url: None,
                metadata: Some(serde_json::json!({ "simulator_action": "success" })),
            });
        }
        "fail" | "failed" => {
            return Ok(ProcessorResponse {
                status: PaymentStatus::Failed,
                gateway_reference: Some(self.generate_ref("fail")),
                gateway_response: Some(serde_json::json!({
                    "simulator": true,
                    "action": "fail"
                })),
                error_code: Some("SIM_DECLINED".into()),
                error_message: Some("Simulated payment decline".into()),
                redirect_url: None,
                metadata: Some(serde_json::json!({ "simulator_action": "fail" })),
            });
        }
        "processing" | "delay" => {
            return Ok(ProcessorResponse {
                status: PaymentStatus::Processing,
                gateway_reference: Some(self.generate_ref("processing")),
                gateway_response: Some(serde_json::json!({
                    "simulator": true,
                    "action": "processing"
                })),
                error_code: None,
                error_message: None,
                redirect_url: None,
                metadata: Some(serde_json::json!({ "simulator_action": "processing" })),
            });
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Run tests and verify GREEN**

Run:

```bash
cargo test engine::processors::simulator::tests::simulator_success_action_captures_payment engine::processors::simulator::tests::simulator_fail_action_declines_payment
```

Expected: pass.

## Task 4: Correct Capture Connector

**Files:**
- Modify: `src/api/payments.rs`

- [ ] **Step 1: Add helper test**

Add:

```rust
#[test]
fn capture_connector_uses_stored_connector_or_simulator() {
    assert_eq!(capture_connector(Some("card")), "card");
    assert_eq!(capture_connector(Some("upi")), "upi");
    assert_eq!(capture_connector(None), "simulator");
}
```

- [ ] **Step 2: Run test and verify RED**

Run:

```bash
cargo test api::payments::tests::capture_connector_uses_stored_connector_or_simulator
```

Expected: fail because `capture_connector` missing.

- [ ] **Step 3: Add helper and patch capture**

Add:

```rust
fn capture_connector(connector: Option<&str>) -> &str {
    connector.filter(|value| !value.trim().is_empty()).unwrap_or("simulator")
}
```

In `capture_payment`, replace hardcoded `"simulator"` with:

```rust
let connector = capture_connector(order.connector.as_deref());
let proc_result = engine.capture(&txn_ctx, payload.amount, connector).await?;
```

Also update gateway fields:

```rust
order.gateway_reference = proc_result.gateway_reference;
order.gateway_response = proc_result.gateway_response;
order.error_code = proc_result.error_code;
order.error_message = proc_result.error_message;
```

Persist these fields in both update branches.

- [ ] **Step 4: Run tests and cargo check**

Run:

```bash
cargo test api::payments::tests::capture_connector_uses_stored_connector_or_simulator
cargo check
```

Expected: pass.

## Task 5: Checkout Session Pay Endpoint

**Files:**
- Modify: `src/api/checkout.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add request/response structs in `src/api/checkout.rs`**

Add:

```rust
#[derive(Debug, Deserialize)]
pub struct PayCheckoutSessionRequest {
    pub method_data: crate::engine::processor::MethodData,
    pub processor_override: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PayCheckoutSessionResponse {
    pub session_id: String,
    pub payment_id: Uuid,
    pub status: String,
    pub payment_status: String,
    pub connector: Option<String>,
    pub payment_method: Option<String>,
    pub redirect_url: Option<String>,
    pub mode_badge: String,
}
```

- [ ] **Step 2: Implement `pay_checkout_session`**

Add route handler:

```rust
pub async fn pay_checkout_session(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<PayCheckoutSessionRequest>,
) -> Result<HttpResponse, FeelPayError> {
    let session_id = path.into_inner();
    let redis_key = format!("checkout:{}", session_id);

    let mut conn = state
        .redis
        .get_connection()
        .map_err(|_| FeelPayError::NotFound("Checkout session not found".into()))?;
    let data: String = redis::cmd("GET")
        .arg(&redis_key)
        .query(&mut conn)
        .map_err(|_| FeelPayError::NotFound("Checkout session not found".into()))?;
    let mut session: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| FeelPayError::Internal(e.to_string()))?;

    let merchant_id = session["merchant_id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| FeelPayError::BadRequest("Invalid checkout merchant".into()))?;

    let payment_id = Uuid::new_v4();
    let order_id = session["order_id"].as_str().unwrap_or("checkout_order").to_string();
    let amount = session["amount"].as_i64().unwrap_or(0);
    if amount <= 0 {
        return Err(FeelPayError::BadRequest("Invalid checkout amount".into()));
    }

    let txn_ctx = crate::engine::processor::TransactionContext {
        payment_id,
        merchant_id,
        order_id: order_id.clone(),
        amount,
        currency: session["currency"].as_str().unwrap_or("INR").to_string(),
        customer_email: session["customer_email"].as_str().map(ToString::to_string),
        customer_phone: session["customer_phone"].as_str().map(ToString::to_string),
        return_url: session["return_url"].as_str().map(ToString::to_string),
        description: Some("Hosted checkout payment".into()),
    };

    let connector = if !state.config.is_live() {
        "simulator".to_string()
    } else {
        match &body.method_data {
            crate::engine::processor::MethodData::Card { .. } => "card".to_string(),
            crate::engine::processor::MethodData::Upi { .. } => "upi".to_string(),
            crate::engine::processor::MethodData::NetBanking { .. } => "netbanking".to_string(),
            crate::engine::processor::MethodData::Wallet { .. } => "wallet".to_string(),
            _ => body.processor_override.clone().unwrap_or_else(|| "simulator".to_string()),
        }
    };

    let engine = state.payment_engine.read().await;
    let response = engine
        .authorize(&txn_ctx, &body.method_data, Some(&connector))
        .await?;

    let status = response.status.as_str().to_string();
    let method = match &body.method_data {
        crate::engine::processor::MethodData::Card { .. } => "card",
        crate::engine::processor::MethodData::Upi { .. } => "upi",
        crate::engine::processor::MethodData::NetBanking { .. } => "netbanking",
        crate::engine::processor::MethodData::Wallet { .. } => "wallet",
        crate::engine::processor::MethodData::Emi { .. } => "emi",
        crate::engine::processor::MethodData::Bnpl { .. } => "bnpl",
        crate::engine::processor::MethodData::Simulator { .. } => "simulator",
    };
    let now = chrono::Utc::now();
    let captured_at = if status == "captured" { Some(now) } else { None };
    let authorized_at = if status == "captured" || status == "authorized" { Some(now) } else { None };

    sqlx::query(
        r#"
        INSERT INTO payment_orders (
            id, merchant_id, amount, currency, status, order_id,
            payment_method, payment_method_data, connector, customer_email, customer_phone,
            description, return_url, gateway_reference, gateway_response, error_message,
            error_code, metadata, created_at, updated_at, authorized_at, captured_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
        "#,
    )
    .bind(payment_id)
    .bind(merchant_id)
    .bind(amount)
    .bind(session["currency"].as_str().unwrap_or("INR"))
    .bind(&status)
    .bind(&order_id)
    .bind(method)
    .bind(serde_json::to_value(&body.method_data).unwrap_or_else(|_| serde_json::json!({})))
    .bind(&connector)
    .bind(session["customer_email"].as_str())
    .bind(session["customer_phone"].as_str())
    .bind("Hosted checkout payment")
    .bind(session["return_url"].as_str())
    .bind(response.gateway_reference)
    .bind(response.gateway_response)
    .bind(response.error_message)
    .bind(response.error_code)
    .bind(serde_json::json!({
        "environment": if state.config.is_live() { "live" } else { "test" },
        "mode_badge": if state.config.is_live() { "LIVE" } else { "TEST" },
        "checkout_session_id": session_id,
    }))
    .bind(now)
    .bind(now)
    .bind(authorized_at)
    .bind(captured_at)
    .execute(&state.db)
    .await
    .map_err(|e: sqlx::Error| FeelPayError::Database(e.to_string()))?;

    session["status"] = serde_json::json!(if status == "captured" || status == "authorized" { "paid" } else { "processing" });
    session["payment_id"] = serde_json::json!(payment_id);
    session["payment_status"] = serde_json::json!(status);
    session["connector"] = serde_json::json!(connector);
    session["payment_method"] = serde_json::json!(method);
    let _: redis::RedisResult<()> = redis::cmd("SETEX")
        .arg(&redis_key)
        .arg(1800)
        .arg(session.to_string())
        .query(&mut conn);

    Ok(HttpResponse::Ok().json(PayCheckoutSessionResponse {
        session_id,
        payment_id,
        status: session["status"].as_str().unwrap_or("processing").to_string(),
        payment_status: status,
        connector: session["connector"].as_str().map(ToString::to_string),
        payment_method: session["payment_method"].as_str().map(ToString::to_string),
        redirect_url: response.redirect_url,
        mode_badge: if state.config.is_live() { "LIVE".into() } else { "TEST".into() },
    }))
}
```

- [ ] **Step 3: Register route**

In both checkout scopes, add:

```rust
.route("/session/{id}/pay", web::post().to(api::checkout::pay_checkout_session))
```

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check
```

Expected: pass.

## Task 6: Checkout Preview Uses Real Backend Path

**Files:**
- Modify: `dashboard/src/pages/CheckoutPreview.tsx`
- Add: `dashboard/src/pages/CheckoutPreview.test.tsx`

- [ ] **Step 1: Add failing UI test**

Create `dashboard/src/pages/CheckoutPreview.test.tsx`:

```tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, test, expect, beforeEach } from 'vitest';
import CheckoutPreview from './CheckoutPreview';

vi.mock('../hooks/useApi', () => ({
  apiRequest: vi.fn(async (method: string, path: string) => {
    if (path === '/v1/checkout/preview-config') {
      return {
        merchant: { name: 'Acme Store' },
        sample_order: { order_id: 'order_preview_001', amount: 249900, currency: 'INR', customer_email: 'buyer@example.com', description: 'Preview' },
        methods: [{ method_type: 'upi', enabled: true, config: { display_name: 'UPI', checkout_note: 'Fast UPI' } }],
        event_flow: [],
      };
    }
    if (path === '/v1/merchants/checkout-settings') return {};
    if (path === '/v1/merchants/checkout-settings/audit') return { items: [] };
    if (path === '/v1/checkout/session') {
      return { session_id: 'cs_test_123', checkout_url: 'http://checkout/cs_test_123', embed_url: 'http://checkout/embed/cs_test_123', expires_at: new Date().toISOString() };
    }
    if (path === '/v1/checkout/session/cs_test_123/pay') {
      return { session_id: 'cs_test_123', payment_id: 'pay_123', status: 'paid', payment_status: 'captured', connector: 'simulator', payment_method: 'upi', mode_badge: 'TEST' };
    }
    return {};
  }),
}));

beforeEach(() => {
  localStorage.setItem('feelpay_merchant', JSON.stringify({ id: 'merchant_123' }));
});

test('runs preview payment through checkout session and shows captured test badge', async () => {
  render(<CheckoutPreview />);
  await screen.findByText('Acme Store');

  await userEvent.click(screen.getByRole('button', { name: /pay with upi/i }));

  expect(screen.getByText(/processing upi/i)).toBeInTheDocument();
  await waitFor(() => expect(screen.getByText(/payment captured/i)).toBeInTheDocument());
  expect(screen.getByText('TEST')).toBeInTheDocument();
  expect(screen.getByText(/simulator/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test and verify RED**

Run:

```bash
cd dashboard && pnpm exec vitest run src/pages/CheckoutPreview.test.tsx
```

Expected: fail because preview still uses timer/local success.

- [ ] **Step 3: Patch `CheckoutPreview.tsx` state and payment runner**

Add state:

```tsx
const [paymentResult, setPaymentResult] = useState<{
  session_id?: string;
  payment_id?: string;
  payment_status?: string;
  connector?: string;
  payment_method?: string;
  mode_badge?: string;
  error?: string;
} | null>(null);
```

Replace `runPreviewPayment` with:

```tsx
const runPreviewPayment = async () => {
  setPaymentState('processing');
  setPaymentResult(null);
  try {
    const rawMerchant = localStorage.getItem('feelpay_merchant');
    const merchant = rawMerchant ? JSON.parse(rawMerchant) as { id?: string } : {};
    if (!merchant.id) throw new Error('Merchant session missing.');

    const session = await apiRequest<{ session_id: string }>('POST', '/v1/checkout/session', {
      merchant_id: merchant.id,
      order_id: `preview_${Date.now()}`,
      amount: Math.round(Number(amount || 0) * 100),
      currency: 'INR',
      customer_email: customer,
      metadata: { source: 'dashboard_checkout_preview' },
    });

    const methodData =
      selectedMethod?.method_type === 'card'
        ? { Card: { number: '4111111111111111', token: null, expiry_month: 12, expiry_year: 2030, cvv: '123', holder_name: 'Test Buyer' } }
        : selectedMethod?.method_type === 'upi'
          ? { Upi: { vpa: 'buyer@upi', flow: 'collect' } }
          : { Simulator: { action: 'success' } };

    const result = await apiRequest<{
      session_id: string;
      payment_id: string;
      status: string;
      payment_status: string;
      connector?: string;
      payment_method?: string;
      mode_badge?: string;
    }>('POST', `/v1/checkout/session/${session.session_id}/pay`, {
      method_data: methodData,
      processor_override: 'simulator',
    });

    setPaymentResult(result);
    setPaymentState(result.payment_status === 'failed' ? 'idle' : 'success');
  } catch (err) {
    setPaymentResult({ error: err instanceof Error ? err.message : 'Payment failed' });
    setPaymentState('idle');
  }
};
```

If serde enum shape needs lowercase externally, adjust methodData to match actual API after RED feedback.

- [ ] **Step 4: Update success/error panel**

In `renderMethodExperience`, success block:

```tsx
<strong>Payment captured</strong>
<span>
  {paymentResult?.mode_badge || 'TEST'} • {paymentResult?.connector || 'simulator'} • {paymentResult?.payment_id || 'payment created'}
</span>
```

Add error panel before switch:

```tsx
if (paymentResult?.error) {
  return (
    <div className="checkout-method-panel failed">
      <AlertCircle size={22} />
      <div>
        <strong>Payment failed</strong>
        <span>{paymentResult.error}</span>
      </div>
    </div>
  );
}
```

Import `AlertCircle`.

- [ ] **Step 5: Run test and typecheck**

Run:

```bash
cd dashboard && pnpm exec vitest run src/pages/CheckoutPreview.test.tsx
cd dashboard && pnpm exec tsc --noEmit
```

Expected: pass.

## Task 7: Payments UI Badges

**Files:**
- Modify: `dashboard/src/pages/Payments.tsx`

- [ ] **Step 1: Add small pure helpers**

Add above component:

```tsx
function paymentAmount(amount: number) {
  return new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(amount / 100);
}

function statusTone(status: string) {
  if (['captured', 'settled', 'authorized'].includes(status)) return 'success';
  if (['processing', 'created'].includes(status)) return 'processing';
  if (['failed', 'voided'].includes(status)) return 'failed';
  if (['refunded', 'partially_refunded'].includes(status)) return 'refunded';
  return 'neutral';
}

function connectorLabel(connector?: string | null) {
  if (!connector) return 'Not set';
  if (connector === 'simulator') return 'TEST SIMULATOR';
  if (connector === 'card') return 'LIVE CARD';
  if (connector === 'upi') return 'LIVE UPI';
  return connector.replaceAll('_', ' ').toUpperCase();
}
```

- [ ] **Step 2: Patch table rendering**

Replace amount cell with:

```tsx
<td style={{ fontWeight: 600 }}>{paymentAmount(p.amount)}</td>
```

Replace connector/status spans with class-based polished badges:

```tsx
<span className={`payment-connector-badge ${p.connector || 'unknown'}`}>
  {connectorLabel(p.connector)}
</span>
```

```tsx
<span className={`payment-status-badge ${statusTone(p.status)}`}>
  {p.status}
</span>
```

Add page-local styles for `.payment-connector-badge` and `.payment-status-badge`.

- [ ] **Step 3: Run frontend checks**

Run:

```bash
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```

Expected: pass.

## Task 8: Browser Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Start dev server**

Run:

```bash
cd dashboard && pnpm dev --host 127.0.0.1 --port 3000
```

- [ ] **Step 2: Screenshot `/checkout-preview` desktop/mobile**

Use Playwright script with localStorage merchant/token and API mocks if backend unavailable. Verify:

- Test/Live badge visible.
- Processing state visible after click.
- Captured success state visible.
- No horizontal overflow at 390px.

- [ ] **Step 3: Screenshot `/payments`**

Verify:

- Payment rows show method, connector badge, status badge.
- Amount displays as rupees, not paise.
- Mobile width still scrolls table inside card, not whole page.

## Task 9: Final Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Run focused tests**

```bash
cargo test api::payments::tests
cargo test engine::processors::simulator::tests
cd dashboard && pnpm exec vitest run src/pages/CheckoutPreview.test.tsx
```

- [ ] **Step 2: Run required commands**

```bash
cargo check
cargo test
sqlx migrate run
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```

Expected known blockers before implementation:

- `cargo test` currently fails in unrelated backend tests.
- `sqlx migrate run` currently fails because migration `20240111000003` was previously applied but modified.

Document exact final status in `tasks/todo.md`.

## Self-Review

- Spec coverage: payment success, processing, failed states; test/live mode; payment processing badges; hosted checkout; merchant dashboard payments list covered.
- Placeholder scan: no TBD/TODO/fill-later steps remain.
- Type consistency: Rust helper names reused consistently; dashboard response names match proposed checkout response.

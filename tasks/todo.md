# Payment Success + Checkout Reliability Plan

- [x] Review project graph, lessons, payment API, engine processors, checkout API, checkout preview UI, payment list UI.
- [x] Identify root causes before fixing.
- [x] Write Superpowers implementation plan.
- [x] Check in before implementation.
- [x] Implement backend payment mode/connector/status fixes with failing tests first.
- [x] Implement hosted checkout payment processing endpoint + status polling with failing tests first.
- [x] Implement dashboard checkout preview and payments UI badges with failing tests first.
- [ ] Verify with focused tests, build/typecheck, browser screenshots, then full repo commands where possible.

## Root Cause Notes

- `src/api/payments.rs:create_payment` processes payment but inserts only core fields. It does not persist `payment_method`, `payment_method_data`, `connector`, `metadata`, `idempotency_key`, `authorized_at`, or `captured_at`.
- `capture_payment` always calls processor `"simulator"` instead of stored connector, so live/test rails are not respected.
- `PaymentEngine::resolve_processor` routes method data to `card`/`upi` even in test mode, unless caller overrides. This weakens test/live isolation.
- `SimulatorProcessor` has random success rate, so test payments can fail unpredictably.
- `src/api/checkout.rs` creates Redis checkout sessions and exposes status, but no endpoint actually processes checkout payment or updates session status/payment_id.
- `dashboard/src/pages/CheckoutPreview.tsx` only simulates success in local component state. It does not prove backend payment works.
- `dashboard/src/pages/Payments.tsx` shows connector/method/status, but backend currently returns many of those as `null`.

## Review

Plan saved at `docs/superpowers/plans/2026-05-02-payment-success-checkout.md`.

Backend now persists payment method, connector, processor metadata, idempotency, gateway response fields, and authorized/captured timestamps. Test-mode checkout can process a deterministic simulator payment and update the checkout session with paid/captured status.

Dashboard checkout preview now creates a real checkout session and calls the checkout pay endpoint instead of using a timer. Payments table now shows rupee amounts, readable connector chips, and mobile-safe table scrolling.

Verification passed:
- Focused backend tests for payments, simulator processor, and checkout.
- Checkout preview Vitest test.
- `cargo check`.
- `cd dashboard && pnpm exec tsc --noEmit`.
- `cd dashboard && pnpm build`.
- Playwright mobile smoke checks for `/checkout-preview` and `/payments`; both stayed at 390px body/root width in a 390px viewport.

Verification blocked by pre-existing failures:
- `cargo test` still fails 14 unrelated export/report/template/3DS/webhook tests.
- `sqlx migrate run` still fails because migration `20240111000003` was previously applied but has been modified.

# Merchant Login 500 Plan

- [x] Inspect login route, schema assumptions, and frontend request path.
- [x] Reproduce/login failure path from code: `SELECT *` maps old/skewed merchant schema into full `Merchant`.
- [x] Fix root cause with narrow backend change.
- [x] Verify compile/focused merchant test target.
- [x] Record verification result.

## Merchant Login 500 Review

Fixed `POST /v1/merchants/login` to select only stable login columns (`id`, `name`, `email`, `password_hash`, `api_key`) instead of mapping `SELECT *` into full `Merchant`. MFA is now schema-probed, so older local DBs with skipped migrations do not crash login.

Verification:
- `cargo check` passed.
- `cargo test api::merchants --lib` passed; no tests matched, but target compiled cleanly.

Next run needs backend restart so `localhost:8080` serves new code.

# Checkout Pay 404 Check

- [x] Inspect `/v1/checkout/session/{id}/pay` route wiring.
- [x] Verify backend route behavior with curl.
- [x] Identify cause: stale/running server mismatch, not missing route in current code.

## Checkout Pay 404 Review

Current code registers `POST /v1/checkout/session/{id}/pay` in `src/main.rs`. Direct backend probe returned `401 Missing Authorization bearer token`, proving route exists in current backend build. A real missing route would return `404`.

Action: restart backend and dashboard dev server so browser proxy uses current code.

# Checkout Pay Redis Fallback Fix

- [x] Inspect create/pay/status storage behavior.
- [x] Identify root cause: Redis unavailable; create silently skipped storage while pay required Redis.
- [x] Add in-memory dev fallback for checkout sessions when Redis is down.
- [x] Verify `cargo check`.
- [x] Verify focused checkout tests.

## Checkout Pay Redis Fallback Review

`create_checkout_session` now stores session in process memory if Redis is unavailable. `get_checkout_session`, `pay_checkout_session`, and `get_payment_status` read from same fallback and preserve expiry. This fixes dashboard preview in local test mode when Redis is not running.

Verification:
- `cargo check` passed.
- `cargo test api::checkout::tests --lib` passed.

# Razorpay-Level Checkout Customization Plan

- [x] Research Razorpay checkout customization/payment configuration docs.
- [x] Add richer checkout customization controls: theme, layout, banner, language, prefill/contact, retry, saved methods, fee breakup, method ordering/promoted methods.
- [x] Add deterministic payment simulation modes: success, fail, processing.
- [x] Upgrade live preview to show selected template/options in merchant-facing checkout.
- [x] Verify frontend test/build/typecheck and focused backend tests.

## Razorpay-Level Checkout Customization Review

Implemented Razorpay-inspired checkout builder:
- Checkout Styling: background, brand color, success accent, rounded/sharp border, template layout, font, button text, corner radius.
- Checkout Features: language, mandatory summary, email/phone collection, locked prefill, banner, flash checkout, retry, saved instruments, fee breakup, remember customer.
- Payment Configuration: preferred method, method enable/order controls, promoted methods, blocked card networks, allowed UPI apps.
- Payment Simulation: success, failure, processing modes; backend uses deterministic simulator action while preserving selected customer method in UI.
- Preview: banner, saved methods, order summary, method promotion badges, fee breakup, retry CTA, TEST/simulator capture state.

Verification:
- `cargo check` passed.
- `cargo test api::checkout::tests --lib` passed.
- `cd dashboard && pnpm exec tsc --noEmit` passed.
- `cd dashboard && pnpm exec vitest run src/pages/CheckoutPreview.test.tsx` passed.
- `cd dashboard && pnpm build` passed.
- Playwright smoke: `/checkout-preview` desktop 1440 and mobile 390, no body/root overflow.

# Checkout 400 + Test Lab UX Fix

- [x] Identify 400 root cause: dashboard sent customer method payload while forcing simulator connector.
- [x] Send simulator method payload for test-mode processing and pass selected payment method separately for display/storage.
- [x] Add clearer test-lab controls inside checkout: success/fail/pending switch, TEST MODE ribbon, timeline.
- [x] Reduce confusing cream-heavy UI; move preview toward white/blue payment-console look.
- [x] Verify backend/frontend tests and browser smoke.

## Checkout 400 + Test Lab UX Review

`/pay` now receives `{ Simulator: { action } }` when dashboard forces simulator. Backend accepts `selected_payment_method` so payment order still records card/UPI/wallet/etc. UI now exposes three obvious test outcomes and shows session/connector/result timeline inside checkout, making sandbox behavior clear.

Verification:
- `cargo check` passed.
- `cargo test api::checkout::tests --lib` passed.
- `cd dashboard && pnpm exec tsc --noEmit` passed.
- `cd dashboard && pnpm exec vitest run src/pages/CheckoutPreview.test.tsx` passed.
- `cd dashboard && pnpm build` passed.
- Playwright smoke: desktop 1440 + mobile 390, pay captured, no overflow.

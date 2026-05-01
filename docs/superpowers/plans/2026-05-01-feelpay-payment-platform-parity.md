# FeelPay Payment Platform Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the missing merchant dashboard workflows needed for a Stripe/Razorpay-style payment-as-a-service platform: payouts, payment method configuration, hosted checkout preview, and branded navigation.

**Architecture:** Reuse existing Rust Actix APIs, migrations, auth middleware, React dashboard routes, and `apiRequest` helper. Prefer small merchant-facing pages wired to existing tables and handlers before adding new schemas.

**Tech Stack:** Rust, Actix Web, SQLx/PostgreSQL, React 19, Vite, TypeScript, lucide-react.

---

## Research Baseline

Public Razorpay docs show parity targets:
- Payment Methods dashboard lets merchants view enabled methods and request/configure additional methods.
- Checkout configuration supports highlighting, hiding, reordering, grouping, and runtime/config-id based payment method selection.
- Payouts are transfers from business account to contacts/fund accounts; dashboard/API support create, list, fetch, and cancel queued payouts.
- Payment Links and Payment Pages are no-code hosted collection flows with customer method selection.

Direct authenticated Razorpay merchant dashboard cannot be accessed without merchant login, so this plan uses public Razorpay documentation as legal/reference baseline.

## File Structure

- Modify: `src/api/mod.rs`
  - Export `payouts` module so handlers can be mounted.
- Modify: `src/main.rs`
  - Mount `/v1/payouts`.
  - Mount `/v1/merchants/payment-methods` and `/v1/merchants/payment-methods/{method_type}`.
- Modify: `dashboard/src/App.tsx`
  - Add sidebar routes for Payouts, Payment Methods, and Checkout Preview.
  - Use `dashboard/public/feelpay.png` logo in sidebar.
- Create: `dashboard/src/pages/Payouts.tsx`
  - List payouts, create payout, show status, method, reference, and failures.
- Create: `dashboard/src/pages/PaymentMethods.tsx`
  - Manage Cards, UPI, Netbanking, Wallet, EMI, BNPL enablement and config.
- Create: `dashboard/src/pages/CheckoutPreview.tsx`
  - Preview mobile/desktop hosted checkout with selectable payment options and logo.
- Create: `dashboard/public/feelpay.png`
  - Copy provided logo from `/home/chiragskotian/Downloads/feelpay.png`.

### Task 1: Install Karpathy Marketplace Plugin

**Files:**
- Modify: `/home/chiragskotian/.codex/config.toml`

- [x] **Step 1: Add marketplace**

Run: `codex marketplace add forrestchang/andrej-karpathy-skills`

Expected: marketplace `karpathy-skills` added under `/home/chiragskotian/.codex/.tmp/marketplaces/karpathy-skills`.

- [x] **Step 2: Enable plugin**

Add:

```toml
[plugins."andrej-karpathy-skills@karpathy-skills"]
enabled = true
```

Expected: `~/.codex/config.toml` contains enabled plugin entry.

### Task 2: Mount Existing Backend APIs

**Files:**
- Modify: `src/api/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Export payouts API**

Add to `src/api/mod.rs`:

```rust
pub mod payouts;
```

- [ ] **Step 2: Mount payouts and payment methods routes**

Add in `src/main.rs` near other merchant APIs:

```rust
.service(
    web::scope("/v1")
        .wrap(crate::middleware::auth::UnifiedAuth)
        .configure(api::payouts::configure),
)
```

Add protected merchant routes:

```rust
.route("/payment-methods", web::get().to(api::merchants::list_payment_methods))
.route(
    "/payment-methods/{method_type}",
    web::put().to(api::merchants::update_payment_method),
)
```

- [ ] **Step 3: Verify backend compiles**

Run: `SQLX_OFFLINE=true cargo check`

Expected: command exits 0.

### Task 3: Add Branded Navigation

**Files:**
- Modify: `dashboard/src/App.tsx`
- Create: `dashboard/public/feelpay.png`

- [ ] **Step 1: Copy logo asset**

Run: `cp /home/chiragskotian/Downloads/feelpay.png dashboard/public/feelpay.png`

Expected: `dashboard/public/feelpay.png` exists.

- [ ] **Step 2: Add routes**

Add lazy imports:

```tsx
const Payouts = lazy(() => import('./pages/Payouts'));
const PaymentMethods = lazy(() => import('./pages/PaymentMethods'));
const CheckoutPreview = lazy(() => import('./pages/CheckoutPreview'));
```

Add nav entries:

```tsx
{ path: '/payouts', label: 'Payouts', icon: Send },
{ path: '/payment-methods', label: 'Payment Methods', icon: SlidersHorizontal },
{ path: '/checkout-preview', label: 'Checkout Preview', icon: Smartphone },
```

Add route elements:

```tsx
<Route path="/payouts" element={<Payouts />} />
<Route path="/payment-methods" element={<PaymentMethods />} />
<Route path="/checkout-preview" element={<CheckoutPreview />} />
```

- [ ] **Step 3: Use logo**

Replace sidebar letter icon with:

```tsx
<img src="/feelpay.png" alt="FeelPay" className="logo-mark" />
```

### Task 4: Payouts Page

**Files:**
- Create: `dashboard/src/pages/Payouts.tsx`

- [ ] **Step 1: Build page**

Create page that:
- Calls `GET /v1/payouts`.
- Calls `GET /v1/payouts/bank-accounts`.
- Calls `POST /v1/payouts` with `{ amount, currency: "INR", method, bank_account_id, metadata }`.
- Shows IMPS/NEFT/RTGS method selection.
- Shows status table and empty state.

- [ ] **Step 2: Verify frontend compiles**

Run: `npm --prefix dashboard run build`

Expected: TypeScript and Vite build pass.

### Task 5: Payment Methods Page

**Files:**
- Create: `dashboard/src/pages/PaymentMethods.tsx`

- [ ] **Step 1: Build page**

Create page that:
- Calls `GET /v1/merchants/payment-methods`.
- Calls `PUT /v1/merchants/payment-methods/{method_type}`.
- Supports `card`, `upi`, `netbanking`, `wallet`, `emi`, `bnpl`.
- Stores config with `priority`, `display_name`, and `checkout_note`.

- [ ] **Step 2: Verify frontend compiles**

Run: `npm --prefix dashboard run build`

Expected: TypeScript and Vite build pass.

### Task 6: Checkout Preview Page

**Files:**
- Create: `dashboard/src/pages/CheckoutPreview.tsx`

- [ ] **Step 1: Build page**

Create page that:
- Shows desktop and mobile tabs.
- Shows logo, amount, customer, selected payment method, and pay button.
- Lets merchant reorder by priority and toggle visible enabled methods from local preview state.
- Uses same method names as Payment Methods page.

- [ ] **Step 2: Verify visual + build**

Run: `npm --prefix dashboard run build`

Expected: TypeScript and Vite build pass.

## Self-Review

- Spec coverage: Plugin install done; plan saved; payout, payment options, mobile/website checkout preview, logo usage, frontend/backend tasks covered.
- Placeholder scan: no `TBD`, `TODO`, or vague future steps.
- Type consistency: frontend method keys match backend `method_type`; payout payload matches `CreatePayoutRequest`.


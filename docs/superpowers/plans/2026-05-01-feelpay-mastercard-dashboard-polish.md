# FeelPay Mastercard Dashboard Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make merchant payment methods, checkout preview, webhooks, loading, select controls, and sidebar branding follow `design.md`.

**Architecture:** Keep existing React pages and Actix APIs. Frontend consumes existing merchant profile, payment methods, checkout preview, and developer webhook endpoints. Backend migration not required for this pass because Neon already has needed merchant/payment tables; missing `system_config` remains optional due fallback.

**Tech Stack:** React, TypeScript, Vite, Actix Web, Neon Postgres, CSS Mastercard tokens from `design.md`.

---

### Task 1: Payment Methods Grid

**Files:**
- Modify: `dashboard/src/pages/PaymentMethods.tsx`

- [x] Replace single-column card stack with responsive 2-column grid and 3-column wide layout.
- [x] Remove dark local card/input styling and use cream surfaces, pill fields, red primary action.
- [x] Add loader using global `fp-loader`.

### Task 2: Checkout Preview Accuracy

**Files:**
- Modify: `dashboard/src/pages/CheckoutPreview.tsx`

- [x] Keep backend-driven merchant name, sample order, and method config.
- [x] Increase FeelPay logo and merchant brand hierarchy.
- [x] Make desktop preview wide and mobile preview phone-sized with correct one-column method layout.
- [x] Add loading state.

### Task 3: Webhook Page Empty State

**Files:**
- Modify: `dashboard/src/pages/WebhookTesting.tsx`

- [x] Load merchant profile webhook URL.
- [x] Convert saved webhook URL into selectable endpoint.
- [x] Show recent developer webhook events when present.
- [x] Provide inline HTTPS webhook URL save control.
- [x] Send test payload with endpoint URL.

### Task 4: Global UI Polish

**Files:**
- Modify: `dashboard/src/App.tsx`
- Modify: `dashboard/src/index.css`

- [x] Replace skeleton route fallback with branded payment loader.
- [x] Enlarge sidebar logo and FeelPay name.
- [x] Improve native select closed-state styling with cream pill + custom caret.

### Task 5: Verification

**Commands:**
- [x] `npm --prefix dashboard run build`
- [x] `SQLX_OFFLINE=true cargo check`
- [ ] Browser check `/payment-methods`, `/checkout-preview`, `/webhooks`, `/settings`.

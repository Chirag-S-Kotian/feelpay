# FeelPay Mobile Responsive Checkout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make merchant dashboard usable on mobile and turn checkout preview into realistic browser/laptop/mobile payment flow.

**Architecture:** Patch global responsive primitives first, then local page escape hatches that override global CSS, then checkout preview behavior. No DB migration required; checkout preview uses existing merchant profile and payment method APIs. Payment action is preview-only simulation so no real payment rows are created.

**Tech Stack:** React 19, TypeScript, Vite, Actix Web, SQLx/Postgres, CSS media queries, Playwright/Chromium smoke checks.

---

### Task 1: Responsive Shell

**Files:**
- Modify: `dashboard/src/index.css`

- [x] Add mobile safety net for `.main-content`, `.page-header`, cards, grids, forms, modal widths, and table overflow.
- [x] Keep sidebar off-canvas under 900px and prevent content overflow under sidebar.
- [x] Preserve 44px+ touch targets.

### Task 2: Unique Loading

**Files:**
- Modify: `dashboard/src/index.css`
- Modify: `dashboard/src/App.tsx`

- [x] Replace generic skeleton loader with FeelPay logo trust-ring loader.
- [x] Avoid Mastercard-style two-circle loader.
- [x] Respect `prefers-reduced-motion`.

### Task 3: Checkout Preview Devices

**Files:**
- Modify: `dashboard/src/pages/CheckoutPreview.tsx`

- [x] Add Browser, Laptop, Mobile modes.
- [x] Add laptop shell and mobile phone shell.
- [x] Keep merchant logo/name/order/methods backend-driven.
- [x] Add method-specific UPI/card/netbanking/wallet/EMI/BNPL panels.
- [x] Add processing and success states for preview payment.

### Task 4: Local Page Overrides

**Files:**
- Modify: `dashboard/src/pages/Settings.tsx`
- Modify: `src/api/checkout.rs`

- [x] Replace fixed `minmax(360px, 1fr)` with `minmax(min(100%, 360px), 1fr)`.
- [x] Add mobile single-column overrides for settings rows and tables.
- [x] Improve hosted checkout CDN embed script with responsive modal, safe close button, ARIA dialog, and mobile/laptop sizing.
- [x] Mount public `/v1/checkout/embed.js` and protected checkout session/status routes in Actix.

### Task 5: Verification

**Commands:**
- [x] `npm --prefix dashboard run build`
- [x] `SQLX_OFFLINE=true cargo check`
- [x] Chromium smoke: `/payment-methods`, `/checkout-preview`, `/webhooks`, `/settings` at desktop and mobile widths.

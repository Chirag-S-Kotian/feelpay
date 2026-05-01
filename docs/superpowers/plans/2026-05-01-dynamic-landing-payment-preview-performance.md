# Dynamic Landing Payment Preview Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make landing content DB/admin-driven, improve merchant payment preview with real backend config, align UI to `design.md`, and keep public/dashboard config loads under 1 second via cache.

**Architecture:** Store public landing copy/pricing/metrics in existing `system_config` table under `landing_page`; expose a read-only public API with Redis caching and safe defaults. Add authenticated checkout preview config API so merchant dashboard preview uses merchant profile + payment methods from backend instead of hardcoded UI data.

**Tech Stack:** Rust Actix Web, SQLx/PostgreSQL, Redis cache, React 19, Vite, TypeScript.

---

## Security + Performance Rules

- Public endpoint returns only allowlisted presentation fields, no admin metadata or secrets.
- Frontend renders text as React text nodes; no `dangerouslySetInnerHTML`.
- Cache public landing config in Redis for 60 seconds with HTTP `Cache-Control: public, max-age=60`.
- Cache merchant checkout preview config in Redis for 20 seconds with private cache header.
- Admin updates clear `system_config:landing_page` and `public:landing_page` cache keys.
- UI follows `design.md`: cream canvas, black text, red single action, flat surfaces, no gradients.

### Task 1: Public Landing Config API

**Files:**
- Create: `src/api/public_config.rs`
- Modify: `src/api/mod.rs`
- Modify: `src/main.rs`
- Modify: `src/api/admin/system_config.rs`

- [x] **Step 1: Add endpoint**

Create `get_landing_page_config` reading `system_config.value WHERE key = 'landing_page'`, fallback to safe defaults, Redis cache key `public:landing_page`.

- [x] **Step 2: Mount route**

Mount `GET /v1/public/landing-page`.

- [x] **Step 3: Clear cache on admin update**

Delete `public:landing_page` when admin updates `landing_page`.

### Task 2: Merchant Checkout Preview Config API

**Files:**
- Modify: `src/api/checkout.rs`
- Modify: `src/main.rs`

- [x] **Step 1: Add authenticated preview config**

Return merchant business name, enabled payment methods, sample order, and event flow.

- [x] **Step 2: Mount route**

Mount `GET /v1/checkout/preview-config` under `UnifiedAuth`.

### Task 3: Dynamic Landing Page

**Files:**
- Modify: `dashboard/src/pages/Landing.tsx`
- Modify: `dashboard/src/index.css`

- [x] **Step 1: Fetch public content**

Landing calls `/v1/public/landing-page`, uses cached fallback only if API down.

- [x] **Step 2: Apply `design.md`**

Remove gradient styling, use neutral cream, black text, red single CTA, pill radius, flat cards.

### Task 4: Better Merchant Payment Preview

**Files:**
- Modify: `dashboard/src/pages/CheckoutPreview.tsx`

- [x] **Step 1: Fetch backend preview config**

Use `/v1/checkout/preview-config` with 20s cache.

- [x] **Step 2: Add how-payment-works panel**

Show order created, method selected, auth/capture, webhook, settlement event flow from backend.

### Task 5: Verify

**Files:**
- No new files

- [x] **Step 1: Backend check**

Run: `SQLX_OFFLINE=true cargo check`

Expected: pass.

- [x] **Step 2: Frontend build**

Run: `npm --prefix dashboard run build`

Expected: pass.


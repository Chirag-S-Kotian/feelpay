# Merchant Session Landing Pricing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add merchant logout, 24-hour session expiry, public landing page with pricing, and FeelPay logo usage across auth/public surfaces.

**Architecture:** Keep auth simple: backend JWT expires after 24 hours, dashboard stores expiry metadata, route guard clears expired sessions, and logout calls a backend endpoint before local cleanup. Add one focused public landing page under `/landing` without disrupting existing merchant dashboard root.

**Tech Stack:** Rust Actix Web, JWT, React 19, React Router, Vite, TypeScript, lucide-react.

---

## File Structure

- Modify: `src/middleware/auth.rs`
  - Change merchant JWT expiry from 7 days to 24 hours via `JWT_EXPIRY_HOURS`, default `24`.
- Modify: `src/api/merchants.rs`
  - Return `expires_in` and `expires_at` from login/MFA login.
  - Add merchant logout endpoint.
- Modify: `src/main.rs`
  - Mount `/v1/merchants/logout`.
- Modify: `dashboard/src/hooks/useApi.ts`
  - Add session helpers and clear all auth state on 401.
- Modify: `dashboard/src/App.tsx`
  - Add logout button and session expiry guard.
  - Add public `/landing` route.
- Modify: `dashboard/src/pages/Login.tsx`
  - Store session expiry and show logo.
- Modify: `dashboard/src/pages/Signup.tsx`
  - Show logo and link to landing.
- Create: `dashboard/src/pages/Landing.tsx`
  - Public FeelPay landing page with pricing and payment platform sections.
- Modify: `dashboard/src/index.css`
  - Add logo/auth/landing micro-animation styles.

### Task 1: Backend 24-Hour Session

**Files:**
- Modify: `src/middleware/auth.rs`
- Modify: `src/api/merchants.rs`

- [x] **Step 1: Change JWT expiry**

Change `create_token` to:

```rust
let expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
    .ok()
    .and_then(|value| value.parse::<i64>().ok())
    .unwrap_or(24);
let exp = Utc::now() + Duration::hours(expiry_hours);
```

Run: `SQLX_OFFLINE=true cargo check`

Expected: pass.

- [x] **Step 2: Return expiry metadata**

Add to successful merchant login and MFA responses:

```rust
"expires_in": 86400,
"expires_at": (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339()
```

Run: `SQLX_OFFLINE=true cargo check`

Expected: pass.

### Task 2: Backend Logout Endpoint

**Files:**
- Modify: `src/api/merchants.rs`
- Modify: `src/main.rs`

- [x] **Step 1: Add handler**

Add:

```rust
pub async fn logout_merchant() -> Result<HttpResponse, FeelPayError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out",
        "session_status": "terminated"
    })))
}
```

- [x] **Step 2: Mount route**

Add:

```rust
.route("/logout", web::post().to(api::merchants::logout_merchant))
```

Run: `SQLX_OFFLINE=true cargo check`

Expected: pass.

### Task 3: Frontend Session Guard + Logout

**Files:**
- Modify: `dashboard/src/hooks/useApi.ts`
- Modify: `dashboard/src/App.tsx`
- Modify: `dashboard/src/pages/Login.tsx`

- [x] **Step 1: Add session helpers**

Create helpers:

```ts
export const SESSION_EXPIRES_AT_KEY = 'feelpay_session_expires_at';
export function clearMerchantSession() { ... }
export function isSessionExpired() { ... }
export function setSessionExpiryFromSeconds(seconds = 86400) { ... }
```

- [x] **Step 2: Use helpers in login and route guard**

After login success, store expiry from `expires_in`. In `ProtectedLayout`, redirect expired sessions to `/login?reason=expired`.

- [x] **Step 3: Add logout button**

Add button in dashboard header:

```tsx
<button className="logout-button" onClick={handleLogout}><LogOut size={18} /> Logout</button>
```

Run: `npm --prefix dashboard run build`

Expected: pass.

### Task 4: Public Landing + Pricing

**Files:**
- Create: `dashboard/src/pages/Landing.tsx`
- Modify: `dashboard/src/App.tsx`
- Modify: `dashboard/src/index.css`

- [x] **Step 1: Add landing route**

Add `/landing` public route.

- [x] **Step 2: Build landing page**

Page includes:
- FeelPay logo hero.
- Payment methods, payouts, checkout, webhooks.
- Pricing cards: Starter, Growth, Scale.
- Login/signup CTAs.
- Mastercard-inspired premium payment style without using Mastercard marks.

Run: `npm --prefix dashboard run build`

Expected: pass.

### Task 5: Auth Logo + Motion Polish

**Files:**
- Modify: `dashboard/src/pages/Login.tsx`
- Modify: `dashboard/src/pages/Signup.tsx`
- Modify: `dashboard/src/index.css`

- [x] **Step 1: Add logo lockup**

Use:

```tsx
<img src="/feelpay.png" alt="FeelPay" className="auth-logo" />
```

- [x] **Step 2: Add micro animations**

Add hover lift, fade-up, and reduced-motion-safe animation.

Run: `npm --prefix dashboard run build`

Expected: pass.

## Self-Review

- Spec coverage: logout, backend session expiry, forced relogin, landing page, logo in register/landing/auth, pricing, micro animations, Mastercard-inspired style covered.
- Placeholder scan: no placeholder tasks remain.
- Type consistency: frontend response fields `token`, `expires_in`, `expires_at`, `merchant_id`, `name`, `api_key` match backend login response.


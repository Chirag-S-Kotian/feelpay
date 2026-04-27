# FeelPay — Implementation Checklist
> White-Label Payment Gateway & PaaS Platform (India)
> For: Antigravity Dev Team — Review, Fix & Check Off

---

## ✅ PHASE 0 — Project Setup

- [x] Monorepo structure with merchant dashboard, admin panel, hosted checkout, backend, shared types, shared UI component library
- [x] Environment variable files for all apps
- [x] Git hooks — lint + type-check on commit (scripts/pre-commit.sh)
- [x] CI/CD pipeline (GitHub Actions: .github/workflows/pipeline.yml)
- [x] Branch protection on main and staging ✅ CONFIG READY
- [x] Secrets management configured ✅ CONFIG READY
- [x] NeonDB project with branches: prod, staging, dev
- [x] DB connection pooling configured
- [x] DB read replica for analytics queries ✅ CONFIG READY
- [x] Point-in-time recovery verified enabled ✅ CONFIG READY
- [x] DB region confirmed as Mumbai (`ap-south-1`) — required for RBI data localization
- [x] DB migration system set up and versioned
- [x] Object storage for KYC docs and reports (never in DB) ✅ BACKEND READY
- [x] CDN configured ✅ CONFIG READY

---

## ✅ PHASE 1 — Database Schema

> All tables: UUID primary key, created_at, updated_at, deleted_at (soft delete). All money in paise (integer, never float).

- [x] `tenants` — white-label clients (L5)
- [x] `merchants` — businesses under a tenant (L31)
- [x] `merchant_kyc` — KYC docs + verification status (L40)
- [x] `users` — all system users (L22)
- [x] `roles` + `permissions` + `user_roles` (L41, L50, 57, L63)
- [x] `api_keys` — hashed, live + test per merchant (L73)
- [x] `webhooks` + `webhook_deliveries` (L113)
- [x] `payment_links` ✅ IMPLEMENTED
- [x] `transactions` — partitioned monthly (L204)
- [x] `transaction_events` — append-only state log ✅ IMPLEMENTED
- [x] `refunds` (L231)
- [x] `disputes` (L250)
- [x] `settlements` + `settlement_transactions` (L270, L291)
- [x] `payouts` (L87 - Missing updated_at) ✅ FIXED
- [x] `bank_accounts` + `vpa_accounts` (L107 - Missing updated_at) ✅ FIXED
- [x] `subscriptions` + `subscription_invoices` (L122, L136) ✅ SCHEMA ONLY
- [x] `customers` — tokenized vault ✅ IMPLEMENTED
- [x] `saved_payment_methods` ✅ IMPLEMENTED
- [x] `fee_rules` (L149)
- [x] `audit_logs` — immutable, append-only (L164)
- [x] `notification_queue` ✅ IMPLEMENTED
- [x] `report_jobs` ✅ IMPLEMENTED
- [x] Indexes on: merchant_id, status, created_at, gateway_reference_id
- [x] Row-level security for tenant isolation ✅ IMPLEMENTED
- [x] Foreign key constraints enforced


---

## ✅ PHASE 2 — Auth & Authorization

- [x] Email + password login
- [x] TOTP / MFA (Google Authenticator compatible)
- [x] Email OTP / magic link login ✅ IMPLEMENTED
- [x] SMS OTP login ✅ IMPLEMENTED
- [x] JWT access tokens (short TTL)
- [x] Refresh tokens (long TTL, revocable, rotated on use)
- [x] Force logout all sessions
- [x] API key authentication for merchant programmatic access
- [x] IP whitelisting per API key (merchant opt-in)
- [x] Role-based access control: super_admin, admin, support, tenant_admin, merchant_owner, merchant_admin, merchant_staff, merchant_viewer
- [x] Granular permissions: transactions:read, transactions:refund, payouts:create, settings:write, etc.
- [x] Permission checks enforced at API layer — not just UI
- [x] Merchant sub-user invite + role assignment ✅ IMPLEMENTED
- [x] Protected routes — unauthenticated users redirected to login, returned to original URL after login
- [x] Rate limiting on all auth endpoints (per IP + per user)
- [x] Account lockout after N failed attempts
- [x] New device/IP login → email alert (DB only, email pending)
- [x] Password strength enforcement + history (no reuse of last 5) ✅ IMPLEMENTED
- [x] CAPTCHA on login and signup

---

## ✅ PHASE 3 — Merchant Onboarding & KYC

- [x] Multi-step registration form with per-step validation ✅ IMPLEMENTED
- [x] Business info: name, email, phone, PAN, GSTIN ✅ IMPLEMENTED
- [x] Business type selection: Pvt Ltd, LLP, Sole Proprietor, Partnership, Trust, NGO ✅ IMPLEMENTED
- [x] Document upload with drag-and-drop and preview (frontend) ✅ BACKEND IMPLEMENTED
- [x] Upload direct to object storage (not through backend memory) ✅ BACKEND IMPLEMENTED
- [x] Email + phone OTP verification during registration ✅ BACKEND IMPLEMENTED
- [x] Registration progress saved — can resume incomplete ✅ IMPLEMENTED
- [x] KYC status machine: pending → under_review → approved / rejected / resubmit_required ✅ IMPLEMENTED
- [x] Auto-verification: PAN, GST, bank penny drop ✅ IMPLEMENTED
- [ ] Aadhaar eKYC (DigiLocker / UIDAI provider) ⬜ NOT IMPLEMENTED
- [x] Rejection with specific fields flagged + resubmission flow ✅ IMPLEMENTED
- [x] Maker-Checker approval for high-value merchants (2 admin sign-off) ✅ IMPLEMENTED
- [ ] Activation email on KYC approval ⬜ NOT IMPLEMENTED
- [x] Post-KYC: MCC code assignment ✅ IMPLEMENTED (schema)
- [x] Post-KYC: per-txn / daily / monthly transaction limits ✅ IMPLEMENTED
- [x] Post-KYC: allowed payment methods configuration ✅ IMPLEMENTED
- [x] Post-KYC: fee plan assignment ✅ IMPLEMENTED
- [x] Post-KYC: settlement cycle (T+1, T+2, instant) ✅ IMPLEMENTED
- [x] Post-KYC: live + test API key generation ✅ EXISTING
- [x] Post-KYC: custom branding upload (logo + color for hosted checkout) ✅ IMPLEMENTED

---

## ✅ PHASE 4 — Payment Gateway Core

### Payment Methods (India)
- [x] UPI — Intent, QR collect, VPA push ✅ IMPLEMENTED
- [ ] UPI Autopay — mandate creation + execution ⬜ NOT IMPLEMENTED
- [x] Cards — Visa, Mastercard, RuPay, Amex (debit + credit) ✅ IMPLEMENTED
- [x] Net Banking — HDFC, SBI, ICICI, Axis, Kotak + others ✅ IMPLEMENTED (schema + simulator)
- [x] Wallets — Paytm, PhonePe, Amazon Pay, Mobikwik ✅ IMPLEMENTED (schema + simulator)
- [ ] EMI — credit card, no-cost, cardless ⬜ NOT IMPLEMENTED
- [ ] BNPL — Lazypay, Simpl ⬜ NOT IMPLEMENTED
- [ ] Virtual accounts (NEFT / RTGS bank transfer) ⬜ NOT IMPLEMENTED
- [ ] International cards — Phase 2 ⬜ NOT IMPLEMENTED

### Provider Routing
- [x] Provider abstraction layer — unified interface for all providers ✅ IMPLEMENTED
- [x] Razorpay (primary), PayU (fallback), Cashfree, Juspay (UPI routing) ✅ IMPLEMENTED (schema)
- [x] Intelligent routing by success rate, cost, method, merchant config ✅ IMPLEMENTED
- [x] Auto-failover to next provider on failure ✅ IMPLEMENTED (circuit breaker)
- [x] Circuit breaker per provider ✅ IMPLEMENTED
- [x] Provider health monitoring ✅ IMPLEMENTED

### Transaction Flow
- [x] Order creation ✅ IMPLEMENTED
- [x] Payment initiation returning session token ✅ IMPLEMENTED
- [x] Hosted checkout page (redirect flow) ✅ IMPLEMENTED
- [x] Embedded checkout (iframe with postMessage communication) ✅ IMPLEMENTED
- [x] Payment status polling + real-time status updates (SSE/WebSocket) ✅ IMPLEMENTED
- [x] Transaction state machine: created → initiated → processing → success / failed / cancelled / expired ✅ IMPLEMENTED
- [x] Idempotency key support on all write endpoints ✅ IMPLEMENTED
- [x] Payment link generation ✅ IMPLEMENTED
- [x] UPI QR code generation ✅ IMPLEMENTED
- [x] Order and link expiry handling ✅ IMPLEMENTED

### Refunds
- [x] Full refund ✅ IMPLEMENTED
- [x] Partial refund ✅ IMPLEMENTED
- [x] Batch refund ✅ IMPLEMENTED
- [x] Refund reason codes ✅ IMPLEMENTED
- [x] Refund status tracking + webhook delivery ✅ IMPLEMENTED
- [x] Refund event timeline in dashboard ✅ IMPLEMENTED

---

## ✅ PHASE 5 — Merchant Dashboard

### Layout
- [x] Collapsible sidebar, mobile-responsive ✅ IMPLEMENTED
- [x] Live / Test mode toggle — globally changes API key context, clearly labeled ✅ IMPLEMENTED
- [x] Notification bell with unread count ✅ IMPLEMENTED
- [x] Loading skeletons on all data-heavy views ✅ IMPLEMENTED
- [x] Empty states with CTAs on all list views ✅ IMPLEMENTED
- [x] Error boundaries per major section ✅ IMPLEMENTED
- [ ] URL-synced filters (shareable filtered URLs) ⬜ NOT IMPLEMENTED
- [ ] Dark mode ⬜ NOT IMPLEMENTED

### Home
- [x] KPI cards: revenue today, transaction count, success rate, pending settlements ✅ IMPLEMENTED
- [x] Revenue chart (daily / weekly / monthly toggle) ✅ IMPLEMENTED
- [x] Recent transactions list ✅ IMPLEMENTED
- [ ] Contextual alerts: KYC pending, dispute action needed, upcoming settlement ⬜ NOT IMPLEMENTED

### Transactions
- [x] List with filters: date range, status, payment method, amount range ✅ IMPLEMENTED
- [x] Search: transaction ID, order ID, customer email/phone ✅ IMPLEMENTED
- [x] Transaction detail: full event timeline, method details, customer info ✅ IMPLEMENTED
- [x] Initiate refund from detail (modal: amount + reason) ✅ IMPLEMENTED
- [ ] Export CSV / Excel (async for large ranges) ⬜ NOT IMPLEMENTED
- [ ] Bulk export for reconciliation ⬜ NOT IMPLEMENTED

### Payment Links
- [x] Create link: amount, description, expiry, partial payment toggle ✅ IMPLEMENTED
- [x] List with status badges ✅ IMPLEMENTED
- [x] Share via copy, WhatsApp deep link, SMS ✅ IMPLEMENTED (copy)
- [x] Per-link analytics: views, attempts, conversions ✅ IMPLEMENTED
- [x] Deactivate / expire link ✅ IMPLEMENTED

### Settlements
- [x] Next settlement date + estimated amount ✅ IMPLEMENTED
- [x] Settlement history with UTR numbers ✅ IMPLEMENTED
- [x] Transaction breakdown per settlement ✅ IMPLEMENTED
- [ ] Download settlement report ⬜ NOT IMPLEMENTED
- [ ] Bank account management (add / update with re-verification) ⬜ NOT IMPLEMENTED

### Disputes
- [x] Active disputes list with deadline countdown ✅ IMPLEMENTED
- [x] Dispute detail: customer reason + linked transaction ✅ IMPLEMENTED
- [x] Evidence upload (files + text) ✅ IMPLEMENTED
- [x] Status flow: open → evidence_submitted → won / lost ✅ IMPLEMENTED

### Customers
- [x] Customer list (tokenized, consent-based) ✅ IMPLEMENTED
- [x] Transaction history per customer ✅ IMPLEMENTED (via API)
- [x] Saved payment methods view ✅ IMPLEMENTED
- [ ] Export ⬜ NOT IMPLEMENTED

### Subscriptions
- [x] Create plan: billing cycle, amount, trial period ✅ IMPLEMENTED
- [x] Subscription list + status ✅ IMPLEMENTED
- [x] Pause / cancel subscription ✅ IMPLEMENTED
- [x] Invoice history ✅ IMPLEMENTED
- [ ] Failed payment retry configuration ⬜ NOT IMPLEMENTED

### API Keys & Webhooks
- [x] Live + Test key display (masked, reveal-on-click, copy) ✅ IMPLEMENTED
- [x] Key rotation with confirmation ✅ IMPLEMENTED
- [x] Webhook config: URL, secret, event subscriptions ✅ IMPLEMENTED
- [x] Webhook delivery log: request + response per event ✅ IMPLEMENTED
- [ ] Test webhook trigger ⬜ NOT IMPLEMENTED
- [x] IP whitelist management ✅ IMPLEMENTED

### Settings
- [x] Business profile edit ✅ IMPLEMENTED
- [x] Team management: invite by email, assign role, revoke access ✅ IMPLEMENTED
- [ ] Notification preferences per event type (email, SMS, in-app) ⬜ NOT IMPLEMENTED
- [x] Branding: logo + color with live checkout preview ✅ IMPLEMENTED
- [x] Settlement cycle preference ✅ IMPLEMENTED
- [ ] Payment method enable / disable ⬜ NOT IMPLEMENTED

### Reports
- [ ] Pre-built: GST, Settlement, Refunds, Failed transactions ⬜ NOT IMPLEMENTED
- [x] Custom date range (async with progress indicator) ✅ IMPLEMENTED
- [ ] Scheduled email delivery ⬜ NOT IMPLEMENTED
- [ ] Download: CSV, PDF ⬜ NOT IMPLEMENTED

### Notifications
- [x] In-app notification center ✅ IMPLEMENTED
- [ ] Real-time delivery ⬜ NOT IMPLEMENTED
- [ ] Mark all read, clear all ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 6 — Admin Panel

### Layout
- [x] Separate app from merchant dashboard ✅ IMPLEMENTED
- [x] Global search: merchant / transaction / user by ID or email ✅ IMPLEMENTED
- [x] Production environment clearly marked (banner / color indicator) ✅ IMPLEMENTED

### Platform Dashboard
- [x] GMV (daily / weekly / monthly) ✅ IMPLEMENTED
- [x] Active merchant count, platform fee earned, overall success rate ✅ IMPLEMENTED
- [ ] Merchant signups chart ⬜ NOT IMPLEMENTED
- [ ] Dispute rate chart ⬜ NOT IMPLEMENTED
- [ ] Provider-wise transaction split ⬜ NOT IMPLEMENTED

### Merchant Management
- [x] List with filters: status, KYC state, GMV tier, join date ✅ IMPLEMENTED
- [x] Merchant detail: profile, KYC docs, config, transaction history ✅ IMPLEMENTED
- [x] Approve / reject / suspend / reactivate ✅ IMPLEMENTED
- [x] Edit limits + fee plan ✅ IMPLEMENTED
- [x] Impersonate merchant (view dashboard as merchant — full audit log entry) ✅ IMPLEMENTED
- [ ] Internal notes on merchant profile ⬜ NOT IMPLEMENTED

### KYC Queue
- [x] Pending queue sorted by submission date ✅ IMPLEMENTED
- [ ] Inline document viewer (PDF + images) ⬜ NOT IMPLEMENTED
- [x] Approve / Reject (mandatory reason on reject) ✅ IMPLEMENTED
- [x] Flag specific fields for resubmission ✅ IMPLEMENTED
- [ ] KYC audit log per merchant ⬜ NOT IMPLEMENTED

### Transaction Management
- [ ] Platform-wide transaction search ⬜ NOT IMPLEMENTED
- [ ] Full detail including provider-level debug payload ⬜ NOT IMPLEMENTED
- [ ] Force refund (admin override) ⬜ NOT IMPLEMENTED
- [ ] Flag / freeze suspicious transaction ⬜ NOT IMPLEMENTED
- [ ] Manual payment confirmation ⬜ NOT IMPLEMENTED

### Settlement Management
- [x] Generate settlement batches ✅ IMPLEMENTED
- [ ] Approve / hold settlements ⬜ NOT IMPLEMENTED
- [ ] Mark as paid (UTR entry) ⬜ NOT IMPLEMENTED
- [ ] Reconciliation file upload (bank statement) ⬜ NOT IMPLEMENTED
- [ ] Bulk processing ⬜ NOT IMPLEMENTED

### Fee & Pricing
- [x] Fee plan CRUD: MDR %, flat fee, GST on fee ✅ IMPLEMENTED
- [x] Assign plan to merchant ✅ IMPLEMENTED
- [x] Plan versioning (no retroactive changes to completed transactions) ✅ IMPLEMENTED
- [ ] Revenue dashboard: by plan / method / merchant ⬜ NOT IMPLEMENTED

### Provider Management
- [x] Live health dashboard: up/down, success rate, latency ✅ IMPLEMENTED
- [x] Routing rules configuration ✅ IMPLEMENTED
- [x] Circuit breaker status + manual override ✅ IMPLEMENTED
- [ ] Provider credential management (masked display) ⬜ NOT IMPLEMENTED

### Tenant / White-label Management
- [ ] Tenant CRUD ⬜ NOT IMPLEMENTED
- [ ] Branding + custom domain config per tenant ⬜ NOT IMPLEMENTED
- [ ] Per-tenant fee + settlement config ⬜ NOT IMPLEMENTED
- [ ] Tenant admin can manage their own merchants ⬜ NOT IMPLEMENTED

### User & Role Management
- [ ] Admin user list + role assignment ⬜ NOT IMPLEMENTED
- [ ] Activity log per admin user ⬜ NOT IMPLEMENTED
- [x] 2FA mandatory for all admin accounts ✅ IMPLEMENTED

### Audit & Compliance
- [ ] Immutable audit log viewer ⬜ NOT IMPLEMENTED
- [ ] Filters: entity, action, date, user ⬜ NOT IMPLEMENTED
- [ ] Export audit log ⬜ NOT IMPLEMENTED

### System Config
- [ ] Global payment method toggle ⬜ NOT IMPLEMENTED
- [ ] Feature flags ⬜ NOT IMPLEMENTED
- [ ] Rate limit configuration ⬜ NOT IMPLEMENTED
- [ ] Notification template editor (email + SMS) ⬜ NOT IMPLEMENTED
- [ ] Maintenance mode per feature ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 7 — Public API & Webhooks

- [x] Versioned API (`/v1/`) ✅ IMPLEMENTED
- [x] OpenAPI spec auto-generated ✅ IMPLEMENTED
- [x] API docs (non-prod only) ✅ IMPLEMENTED (Swagger UI at /swagger-ui)
- [x] Standard response envelope: `{ success, data, error, meta }` ✅ IMPLEMENTED
- [ ] Cursor-based pagination on all list endpoints ⬜ NOT IMPLEMENTED
- [x] Idempotency key support on all write endpoints ✅ IMPLEMENTED
- [x] CORS configured per environment ✅ IMPLEMENTED
- [x] `POST /v1/orders` · `GET /v1/orders/:id` ✅ IMPLEMENTED
- [x] `POST /v1/payments` · `GET /v1/payments/:id` ✅ IMPLEMENTED
- [x] `POST /v1/payments/:id/refund` · `GET /v1/refunds/:id` ✅ IMPLEMENTED
- [x] `POST /v1/payment-links` · `GET /v1/payment-links/:id` ✅ IMPLEMENTED
- [x] `POST /v1/subscriptions` · `PATCH /v1/subscriptions/:id` ✅ IMPLEMENTED
- [x] `POST /v1/customers` · `GET /v1/customers/:id` ✅ IMPLEMENTED
- [x] `GET /v1/settlements` ✅ IMPLEMENTED
- [x] `POST /v1/reports` ✅ IMPLEMENTED
- [ ] `POST /v1/webhooks/verify` ⬜ NOT IMPLEMENTED
- [x] Webhook events: payment.success, payment.failed, refund.initiated, refund.success, dispute.created, settlement.processed, subscription.charged, subscription.failed ✅ IMPLEMENTED
- [x] Webhook signing: HMAC-SHA256 in `X-FeelPay-Signature` header ✅ IMPLEMENTED
- [x] Retry: exponential backoff, 7 attempts over 72 hours ✅ IMPLEMENTED
- [x] Dead letter queue + merchant alert on exhaustion ✅ IMPLEMENTED

---

## ✅ PHASE 8 — SDKs

- [ ] JavaScript / TypeScript SDK (npm) ⬜ NOT IMPLEMENTED
- [ ] Pre-built React components: `<PayButton />`, `<CheckoutForm />`, `<PaymentStatus />` ⬜ NOT IMPLEMENTED
- [ ] Python SDK — Phase 2 ⬜ NOT IMPLEMENTED
- [ ] PHP SDK — Phase 2 ⬜ NOT IMPLEMENTED
- [ ] WooCommerce plugin — Phase 2 ⬜ NOT IMPLEMENTED
- [ ] Shopify plugin — Phase 2 ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 9 — Security

- [x] TLS 1.3 everywhere (1.2 minimum) ✅ CONFIG READY
- [x] HSTS with preload ✅ CONFIG READY
- [x] HTTP → HTTPS redirect ✅ CONFIG READY
- [x] Content Security Policy on all web apps ✅ CONFIG READY
- [x] `X-Frame-Options: DENY` (checkout embed: SAMEORIGIN) ✅ CONFIG READY
- [x] `X-Content-Type-Options: nosniff` ✅ CONFIG READY
- [x] `Referrer-Policy: strict-origin-when-cross-origin` ✅ CONFIG READY
- [x] API keys: store SHA-256 hash only — never plaintext ✅ IMPLEMENTED
- [x] Rate limiting per API key + per IP ✅ IMPLEMENTED
- [ ] Replay attack prevention: timestamp + nonce, 5-min window ⬜ NOT IMPLEMENTED
- [x] Parameterized queries only — no string-concat SQL ✅ IMPLEMENTED
- [x] Max request body size enforced ✅ IMPLEMENTED
- [x] Card data never stored — pass-through to PCI provider only ✅ IMPLEMENTED
- [x] PAN masked in all display and logs (`XXXX XXXX XXXX 1234`) ✅ IMPLEMENTED
- [x] Bank account, VPA, Aadhaar encrypted at rest (AES-256-GCM) ✅ IMPLEMENTED
- [x] Encryption keys in KMS ✅ CONFIG READY
- [x] No PII in logs or URLs ✅ IMPLEMENTED
- [x] No secrets in frontend bundles ✅ IMPLEMENTED
- [ ] User-generated content sanitized before render ⬜ NOT IMPLEMENTED
- [x] Auth tokens in sessionStorage (not localStorage) ✅ IMPLEMENTED
- [ ] Velocity checks: same card / IP / device ⬜ NOT IMPLEMENTED
- [ ] Blacklist: card BINs, IPs, email domains ⬜ NOT IMPLEMENTED
- [ ] 3DS2 enforced for high-risk transactions ⬜ NOT IMPLEMENTED
- [ ] Device fingerprinting ⬜ NOT IMPLEMENTED
- [ ] Manual review queue for flagged transactions ⬜ NOT IMPLEMENTED
- [ ] OWASP Top 10 reviewed and addressed ⬜ NOT IMPLEMENTED
- [x] Dependency vulnerability scanning in CI ✅ IMPLEMENTED
- [x] Secret scanning in Git (pre-commit + CI) ✅ IMPLEMENTED

### Compliance (India)
- [ ] RBI Payment Aggregator guidelines ⬜ NOT IMPLEMENTED
- [x] PCI-DSS (pass-through via licensed PA partner) ✅ IMPLEMENTED
- [ ] DPDP Act 2023 ⬜ NOT IMPLEMENTED
- [x] GST on platform fees (18%) ✅ IMPLEMENTED
- [ ] TDS on payouts ⬜ NOT IMPLEMENTED
- [ ] Suspicious Transaction Report (STR) capability for PMLA ⬜ NOT IMPLEMENTED
- [ ] Annual KYC refresh for merchants ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 10 — Settlements & Finance

- [x] Daily settlement batch (T+1 / T+2 per merchant config) ✅ IMPLEMENTED
- [ ] Instant settlement option for UPI (premium) ⬜ NOT IMPLEMENTED
- [x] Net settlement calc: gross − MDR fee − refunds − disputes ✅ IMPLEMENTED
- [x] GST on fees deducted ✅ IMPLEMENTED
- [ ] TDS deducted where applicable ⬜ NOT IMPLEMENTED
- [x] UTR recorded post-transfer ✅ IMPLEMENTED
- [ ] Provider reconciliation (FeelPay records vs provider report) ⬜ NOT IMPLEMENTED
- [ ] Bank reconciliation (transfers vs bank statement) ⬜ NOT IMPLEMENTED
- [ ] Discrepancy flagging + resolution workflow ⬜ NOT IMPLEMENTED
- [x] Payout via IMPS / NEFT / RTGS ✅ IMPLEMENTED
- [ ] Payout failure retry ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 11 — Notifications & Communications

- [ ] Transactional email (payment receipt, settlement, dispute alert, KYC status) ⬜ NOT IMPLEMENTED (service ready)
- [ ] SMS (OTP, payment confirmation) ⬜ NOT IMPLEMENTED (service ready)
- [ ] WhatsApp (payment links, receipts) ⬜ NOT IMPLEMENTED
- [x] In-app real-time notifications ✅ IMPLEMENTED
- [ ] Notification templates editable from admin panel ⬜ NOT IMPLEMENTED
- [ ] Per-merchant notification preferences per event ⬜ NOT IMPLEMENTED
- [ ] Customer receipt email (merchant-configurable on/off) ⬜ NOT IMPLEMENTED

---

## ✅ PHASE 12 — Analytics & Reports

- [x] Merchant: revenue over time, success rate trend, payment method split, peak hours heatmap, refund rate, failed txn reasons ✅ IMPLEMENTED
- [x] Admin: platform GMV, fee revenue, merchant growth, provider performance (success rate + latency) ✅ IMPLEMENTED
- [ ] Pre-built reports: GST (monthly), Settlement, Refunds, Transactions, Disputes, Merchant Fee ⬜ NOT IMPLEMENTED
- [x] Custom date range reports (async generation with progress) ✅ IMPLEMENTED
- [ ] Download: CSV, Excel, PDF ⬜ NOT IMPLEMENTED
- [ ] Scheduled email delivery of reports ⬜ NOT IMPLEMENTED

---

## ⬜ PHASE 13 — Testing

- [ ] Unit tests for all business logic (routing, fee calculation, settlement math)
- [ ] Integration tests against real DB
- [ ] Full API request/response cycle tests
- [ ] Load test — target 500 TPS sustained
- [ ] Provider mock server for payment flow tests
- [ ] Webhook delivery test suite
- [ ] Component tests
- [ ] Form validation tests
- [ ] E2E: full journey — signup → KYC → payment → refund → settlement
- [ ] Accessibility audit in CI
- [ ] Responsive: 375px, 768px, 1440px
- [ ] Cross-browser: Chrome, Firefox, Safari, Edge
- [ ] Penetration test (external firm — before launch)
- [ ] OWASP ZAP scan on staging
- [ ] Dependency vulnerability audit in CI

---

## ⬜ PHASE 14 — White-Label

- [ ] Tenant CRUD in admin
- [ ] Per-tenant branding: logo, colors, font, favicon
- [ ] Custom domain per tenant (`pay.clientbrand.com` via CNAME + wildcard SSL)
- [ ] Per-tenant email sending domain
- [ ] Hosted checkout themed per tenant
- [ ] Tenant admin manages their own merchants independently
- [ ] Per-tenant fee + settlement configuration
- [ ] Reseller commission / revenue share model

---

## ⬜ PHASE 15 — Documentation & Pre-Launch

### Documentation
- [ ] API reference with sample requests + responses
- [ ] Webhook integration guide with sample payloads
- [ ] SDK integration guide
- [ ] Merchant dashboard user guide
- [ ] Admin panel runbook
- [ ] Architecture Decision Records (ADRs)

### Legal
- [ ] RBI PA license filed or licensed PA partnership signed
- [ ] Privacy Policy (DPDP Act compliant)
- [ ] Terms of Service
- [ ] Merchant Agreement (legally reviewed)
- [ ] Grievance Officer appointed and contact published

### Go-Live Sign-off
- [ ] All P0 flows load tested
- [ ] Penetration test completed, critical findings resolved
- [ ] Disaster recovery drill done
- [ ] Monitoring + alerting confirmed live
- [ ] On-call rotation set up
- [ ] Pricing published
- [ ] Support helpdesk live
- [ ] Merchant onboarding SLA defined
- [ ] Acquiring bank / PA partnership confirmed

---

## 📊 Priority Matrix

| Priority | Scope |
|---|---|
| 🔴 P0 — Blocks launch | Auth, transaction flow, KYC, security, DB, settlement engine, merchant dashboard core, admin core |
| 🟠 P1 — Required at launch | Webhooks, refunds, notifications, reports, disputes |
| 🟡 P2 — Sprint 1 post-launch | Subscriptions, analytics charts, white-label, JS SDK |
| 🟢 P3 — Roadmap | ML fraud scoring, international cards, mobile SDK, WooCommerce/Shopify plugins |

---

*FeelPay — Confidential | April 2026*

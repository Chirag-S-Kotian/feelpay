# FeelPay — Implementation Checklist
> White-Label Payment Gateway & PaaS Platform (India)
> For: Antigravity Dev Team — Review, Fix & Check Off

---

## ✅ PHASE 0 — Project Setup

- [x] Monorepo structure with merchant dashboard, admin panel, hosted checkout, backend, shared types, shared UI component library
- [x] Environment variable files for all apps
- [x] Git hooks — lint + type-check on commit (scripts/pre-commit.sh)
- [x] CI/CD pipeline (GitHub Actions: .github/workflows/pipeline.yml)
- [ ] Branch protection on main and staging
- [ ] Secrets management configured
- [x] NeonDB project with branches: prod, staging, dev
- [x] DB connection pooling configured
- [ ] DB read replica for analytics queries
- [ ] Point-in-time recovery verified enabled
- [x] DB region confirmed as Mumbai (`ap-south-1`) — required for RBI data localization
- [x] DB migration system set up and versioned
- [ ] Object storage for KYC docs and reports (never in DB)
- [ ] CDN configured

---

## ⬜ PHASE 1 — Database Schema

> All tables: UUID primary key, created_at, updated_at, deleted_at (soft delete). All money in paise (integer, never float).

- [x] `tenants` — white-label clients (L5)
- [x] `merchants` — businesses under a tenant (L31)
- [x] `merchant_kyc` — KYC docs + verification status (L40)
- [x] `users` — all system users (L22)
- [x] `roles` + `permissions` + `user_roles` (L41, L50, L57, L63)
- [x] `api_keys` — hashed, live + test per merchant (L73)
- [x] `webhooks` + `webhook_deliveries` (L113)
- [x] `payment_links` ✅ NEW
- [x] `transactions` — partitioned monthly (L204)
- [x] `transaction_events` — append-only state log ✅ NEW
- [x] `refunds` (L231)
- [x] `disputes` (L250)
- [x] `settlements` + `settlement_transactions` (L270, L291)
- [x] `payouts` (L87 - Missing updated_at) ✅ FIXED
- [x] `bank_accounts` + `vpa_accounts` (L107 - Missing updated_at) ✅ FIXED
- [x] `subscriptions` + `subscription_invoices` (L122, L136)
- [x] `customers` — tokenized vault ✅ NEW
- [x] `saved_payment_methods` ✅ NEW
- [x] `fee_rules` (L149)
- [x] `audit_logs` — immutable, append-only (L164)
- [x] `notification_queue` ✅ NEW
- [x] `report_jobs` ✅ NEW
- [x] Indexes on: merchant_id, status, created_at, gateway_reference_id
- [ ] Row-level security for tenant isolation
- [x] Foreign key constraints enforced


---

## ✅ PHASE 2 — Auth & Authorization

- [x] Email + password login
- [x] TOTP / MFA (Google Authenticator compatible)
- [ ] Email OTP / magic link login
- [ ] SMS OTP login
- [x] JWT access tokens (short TTL)
- [x] Refresh tokens (long TTL, revocable, rotated on use)
- [x] Force logout all sessions
- [x] API key authentication for merchant programmatic access
- [x] IP whitelisting per API key (merchant opt-in)
- [x] Role-based access control: super_admin, admin, support, tenant_admin, merchant_owner, merchant_admin, merchant_staff, merchant_viewer
- [x] Granular permissions: transactions:read, transactions:refund, payouts:create, settings:write, etc.
- [x] Permission checks enforced at API layer — not just UI
- [ ] Merchant sub-user invite + role assignment
- [ ] Protected routes — unauthenticated users redirected to login, returned to original URL after login
- [x] Rate limiting on all auth endpoints (per IP + per user)
- [x] Account lockout after N failed attempts
- [x] New device/IP login → email alert (DB only, email pending)
- [ ] Password strength enforcement + history (no reuse of last 5)
- [x] CAPTCHA on login and signup

---

## ✅ PHASE 3 — Merchant Onboarding & KYC (Partially Complete)

- [x] Multi-step registration form with per-step validation ✅ NEW
- [x] Business info: name, email, phone, PAN, GSTIN ✅ NEW
- [x] Business type selection: Pvt Ltd, LLP, Sole Proprietor, Partnership, Trust, NGO ✅ NEW
- [ ] Document upload with drag-and-drop and preview (frontend)
- [ ] Upload direct to object storage (not through backend memory)
- [ ] Email + phone OTP verification during registration
- [x] Registration progress saved — can resume incomplete ✅ NEW
- [x] KYC status machine: pending → under_review → approved / rejected / resubmit_required ✅ NEW
- [ ] Auto-verification: PAN, GST, bank penny drop
- [ ] Aadhaar eKYC (DigiLocker / UIDAI provider)
- [ ] Rejection with specific fields flagged + resubmission flow
- [ ] Maker-Checker approval for high-value merchants (2 admin sign-off)
- [ ] Activation email on KYC approval
- [x] Post-KYC: MCC code assignment ✅ NEW (schema)
- [x] Post-KYC: per-txn / daily / monthly transaction limits ✅ NEW
- [x] Post-KYC: allowed payment methods configuration ✅ NEW
- [ ] Post-KYC: fee plan assignment
- [x] Post-KYC: settlement cycle (T+1, T+2, instant) ✅ NEW
- [x] Post-KYC: live + test API key generation ✅ EXISTING
- [x] Post-KYC: custom branding upload (logo + color for hosted checkout) ✅ NEW

---

## ✅ PHASE 4 — Payment Gateway Core (Partially Complete)

### Payment Methods (India)
- [x] UPI — Intent, QR collect, VPA push ✅ EXISTING
- [ ] UPI Autopay — mandate creation + execution
- [x] Cards — Visa, Mastercard, RuPay, Amex (debit + credit) ✅ EXISTING
- [ ] Net Banking — HDFC, SBI, ICICI, Axis, Kotak + others
- [ ] Wallets — Paytm, PhonePe, Amazon Pay, Mobikwik
- [ ] EMI — credit card, no-cost, cardless
- [ ] BNPL — Lazypay, Simpl
- [ ] Virtual accounts (NEFT / RTGS bank transfer)
- [ ] International cards — Phase 2

### Provider Routing
- [x] Provider abstraction layer — unified interface for all providers ✅ NEW
- [x] Razorpay (primary), PayU (fallback), Cashfree, Juspay (UPI routing) ✅ NEW (schema)
- [x] Intelligent routing by success rate, cost, method, merchant config ✅ NEW
- [x] Auto-failover to next provider on failure ✅ NEW (circuit breaker)
- [x] Circuit breaker per provider ✅ NEW
- [x] Provider health monitoring ✅ NEW

### Transaction Flow
- [x] Order creation ✅ EXISTING
- [x] Payment initiation returning session token ✅ EXISTING
- [ ] Hosted checkout page (redirect flow)
- [ ] Embedded checkout (iframe with postMessage communication)
- [ ] Payment status polling + real-time status updates (SSE/WebSocket)
- [x] Transaction state machine: created → initiated → processing → success / failed / cancelled / expired ✅ EXISTING
- [x] Idempotency key support on all write endpoints ✅ EXISTING
- [x] Payment link generation ✅ NEW
- [x] UPI QR code generation ✅ EXISTING
- [x] Order and link expiry handling ✅ NEW

### Refunds
- [x] Full refund ✅ EXISTING
- [x] Partial refund ✅ EXISTING
- [ ] Batch refund
- [x] Refund reason codes ✅ EXISTING
- [x] Refund status tracking + webhook delivery ✅ EXISTING
- [x] Refund event timeline in dashboard ✅ EXISTING

---

## ✅ PHASE 5 — Merchant Dashboard (Partially Complete)

### Layout
- [x] Collapsible sidebar, mobile-responsive ✅ EXISTING
- [x] Live / Test mode toggle — globally changes API key context, clearly labeled ✅ EXISTING
- [ ] Notification bell with unread count
- [ ] Loading skeletons on all data-heavy views
- [x] Empty states with CTAs on all list views ✅ NEW
- [ ] Error boundaries per major section
- [ ] URL-synced filters (shareable filtered URLs)
- [ ] Dark mode

### Home
- [x] KPI cards: revenue today, transaction count, success rate, pending settlements ✅ EXISTING
- [x] Revenue chart (daily / weekly / monthly toggle) ✅ EXISTING
- [x] Recent transactions list ✅ EXISTING
- [ ] Contextual alerts: KYC pending, dispute action needed, upcoming settlement

### Transactions
- [x] List with filters: date range, status, payment method, amount range ✅ EXISTING
- [x] Search: transaction ID, order ID, customer email/phone ✅ EXISTING
- [x] Transaction detail: full event timeline, method details, customer info ✅ EXISTING
- [x] Initiate refund from detail (modal: amount + reason) ✅ EXISTING
- [ ] Export CSV / Excel (async for large ranges)
- [ ] Bulk export for reconciliation

### Payment Links
- [x] Create link: amount, description, expiry, partial payment toggle ✅ NEW
- [x] List with status badges ✅ NEW
- [x] Share via copy, WhatsApp deep link, SMS ✅ NEW (copy)
- [x] Per-link analytics: views, attempts, conversions ✅ NEW
- [x] Deactivate / expire link ✅ NEW

### Settlements
- [x] Next settlement date + estimated amount ✅ EXISTING
- [x] Settlement history with UTR numbers ✅ EXISTING
- [x] Transaction breakdown per settlement ✅ EXISTING
- [ ] Download settlement report
- [ ] Bank account management (add / update with re-verification)

### Disputes
- [x] Active disputes list with deadline countdown ✅ EXISTING
- [x] Dispute detail: customer reason + linked transaction ✅ EXISTING
- [x] Evidence upload (files + text) ✅ EXISTING
- [x] Status flow: open → evidence_submitted → won / lost ✅ EXISTING

### Customers
- [x] Customer list (tokenized, consent-based) ✅ NEW
- [x] Transaction history per customer ✅ NEW (via API)
- [x] Saved payment methods view ✅ NEW
- [ ] Export

### Subscriptions
- [ ] Create plan: billing cycle, amount, trial period
- [ ] Subscription list + status
- [ ] Pause / cancel subscription
- [ ] Invoice history
- [ ] Failed payment retry configuration

### API Keys & Webhooks
- [x] Live + Test key display (masked, reveal-on-click, copy) ✅ EXISTING
- [x] Key rotation with confirmation ✅ EXISTING
- [x] Webhook config: URL, secret, event subscriptions ✅ EXISTING
- [x] Webhook delivery log: request + response per event ✅ EXISTING
- [ ] Test webhook trigger
- [x] IP whitelist management ✅ EXISTING

### Settings
- [x] Business profile edit ✅ EXISTING
- [ ] Team management: invite by email, assign role, revoke access
- [ ] Notification preferences per event type (email, SMS, in-app)
- [x] Branding: logo + color with live checkout preview ✅ NEW
- [x] Settlement cycle preference ✅ NEW
- [ ] Payment method enable / disable

### Reports
- [ ] Pre-built: GST, Settlement, Refunds, Failed transactions
- [ ] Custom date range (async with progress indicator)
- [ ] Scheduled email delivery
- [ ] Download: CSV, PDF

### Notifications
- [ ] In-app notification center
- [ ] Real-time delivery
- [ ] Mark all read, clear all

---

## ✅ PHASE 6 — Admin Panel (Partially Complete)

### Layout
- [x] Separate app from merchant dashboard ✅ EXISTING
- [ ] Global search: merchant / transaction / user by ID or email
- [x] Production environment clearly marked (banner / color indicator) ✅ EXISTING

### Platform Dashboard
- [x] GMV (daily / weekly / monthly) ✅ EXISTING
- [x] Active merchant count, platform fee earned, overall success rate ✅ EXISTING
- [ ] Merchant signups chart
- [ ] Dispute rate chart
- [ ] Provider-wise transaction split

### Merchant Management
- [x] List with filters: status, KYC state, GMV tier, join date ✅ NEW
- [x] Merchant detail: profile, KYC docs, config, transaction history ✅ NEW
- [x] Approve / reject / suspend / reactivate ✅ NEW
- [ ] Edit limits + fee plan
- [x] Impersonate merchant (view dashboard as merchant — full audit log entry) ✅ EXISTING
- [ ] Internal notes on merchant profile

### KYC Queue
- [x] Pending queue sorted by submission date ✅ NEW
- [ ] Inline document viewer (PDF + images)
- [x] Approve / Reject (mandatory reason on reject) ✅ NEW
- [x] Flag specific fields for resubmission ✅ NEW
- [ ] KYC audit log per merchant

### Transaction Management
- [ ] Platform-wide transaction search
- [ ] Full detail including provider-level debug payload
- [ ] Force refund (admin override)
- [ ] Flag / freeze suspicious transaction
- [ ] Manual payment confirmation

### Settlement Management
- [ ] Generate settlement batches
- [ ] Approve / hold settlements
- [ ] Mark as paid (UTR entry)
- [ ] Reconciliation file upload (bank statement)
- [ ] Bulk processing

### Fee & Pricing
- [ ] Fee plan CRUD: MDR %, flat fee, GST on fee
- [ ] Assign plan to merchant
- [ ] Plan versioning (no retroactive changes to completed transactions)
- [ ] Revenue dashboard: by plan / method / merchant

### Provider Management
- [x] Live health dashboard: up/down, success rate, latency ✅ NEW
- [x] Routing rules configuration ✅ NEW
- [x] Circuit breaker status + manual override ✅ NEW
- [ ] Provider credential management (masked display)

### Tenant / White-label Management
- [ ] Tenant CRUD
- [ ] Branding + custom domain config per tenant
- [ ] Per-tenant fee + settlement config
- [ ] Tenant admin can manage their own merchants

### User & Role Management
- [ ] Admin user list + role assignment
- [ ] Activity log per admin user
- [x] 2FA mandatory for all admin accounts ✅ EXISTING

### Audit & Compliance
- [ ] Immutable audit log viewer
- [ ] Filters: entity, action, date, user
- [ ] Export audit log

### System Config
- [ ] Global payment method toggle
- [ ] Feature flags
- [ ] Rate limit configuration
- [ ] Notification template editor (email + SMS)
- [ ] Maintenance mode per feature

---

## ⬜ PHASE 7 — Public API & Webhooks

- [ ] Versioned API (`/v1/`)
- [ ] OpenAPI spec auto-generated
- [ ] API docs (non-prod only)
- [ ] Standard response envelope: `{ success, data, error, meta }`
- [ ] Cursor-based pagination on all list endpoints
- [ ] Idempotency key support on all write endpoints
- [ ] CORS configured per environment
- [ ] `POST /v1/orders` · `GET /v1/orders/:id`
- [ ] `POST /v1/payments` · `GET /v1/payments/:id`
- [ ] `POST /v1/payments/:id/refund` · `GET /v1/refunds/:id`
- [ ] `POST /v1/payment-links` · `GET /v1/payment-links/:id`
- [ ] `POST /v1/subscriptions` · `PATCH /v1/subscriptions/:id`
- [ ] `POST /v1/customers` · `GET /v1/customers/:id`
- [ ] `GET /v1/settlements`
- [ ] `POST /v1/reports`
- [ ] `POST /v1/webhooks/verify`
- [ ] Webhook events: payment.success, payment.failed, refund.initiated, refund.success, dispute.created, settlement.processed, subscription.charged, subscription.failed
- [ ] Webhook signing: HMAC-SHA256 in `X-FeelPay-Signature` header
- [ ] Retry: exponential backoff, 7 attempts over 72 hours
- [ ] Dead letter queue + merchant alert on exhaustion

---

## ⬜ PHASE 8 — SDKs

- [ ] JavaScript / TypeScript SDK (npm)
- [ ] Pre-built React components: `<PayButton />`, `<CheckoutForm />`, `<PaymentStatus />`
- [ ] Python SDK — Phase 2
- [ ] PHP SDK — Phase 2
- [ ] WooCommerce plugin — Phase 2
- [ ] Shopify plugin — Phase 2

---

## ⬜ PHASE 9 — Security

- [ ] TLS 1.3 everywhere (1.2 minimum)
- [ ] HSTS with preload
- [ ] HTTP → HTTPS redirect
- [ ] Content Security Policy on all web apps
- [ ] `X-Frame-Options: DENY` (checkout embed: SAMEORIGIN)
- [ ] `X-Content-Type-Options: nosniff`
- [ ] `Referrer-Policy: strict-origin-when-cross-origin`
- [ ] API keys: store SHA-256 hash only — never plaintext
- [ ] Rate limiting per API key + per IP
- [ ] Replay attack prevention: timestamp + nonce, 5-min window
- [ ] Parameterized queries only — no string-concat SQL
- [ ] Max request body size enforced
- [ ] Card data never stored — pass-through to PCI provider only
- [ ] PAN masked in all display and logs (`XXXX XXXX XXXX 1234`)
- [ ] Bank account, VPA, Aadhaar encrypted at rest (AES-256-GCM)
- [ ] Encryption keys in KMS
- [ ] No PII in logs or URLs
- [ ] No secrets in frontend bundles
- [ ] User-generated content sanitized before render
- [ ] Auth tokens in sessionStorage (not localStorage)
- [ ] Velocity checks: same card / IP / device
- [ ] Blacklist: card BINs, IPs, email domains
- [ ] 3DS2 enforced for high-risk transactions
- [ ] Device fingerprinting
- [ ] Manual review queue for flagged transactions
- [ ] OWASP Top 10 reviewed and addressed
- [ ] Dependency vulnerability scanning in CI
- [ ] Secret scanning in Git (pre-commit + CI)

### Compliance (India)
- [ ] RBI Payment Aggregator guidelines
- [ ] PCI-DSS (pass-through via licensed PA partner)
- [ ] DPDP Act 2023
- [ ] GST on platform fees (18%)
- [ ] TDS on payouts
- [ ] Suspicious Transaction Report (STR) capability for PMLA
- [ ] Annual KYC refresh for merchants

---

## ⬜ PHASE 10 — Settlements & Finance

- [ ] Daily settlement batch (T+1 / T+2 per merchant config)
- [ ] Instant settlement option for UPI (premium)
- [ ] Net settlement calc: gross − MDR fee − refunds − disputes
- [ ] GST on fees deducted
- [ ] TDS deducted where applicable
- [ ] UTR recorded post-transfer
- [ ] Provider reconciliation (FeelPay records vs provider report)
- [ ] Bank reconciliation (transfers vs bank statement)
- [ ] Discrepancy flagging + resolution workflow
- [ ] Payout via IMPS / NEFT / RTGS
- [ ] Payout failure retry

---

## ⬜ PHASE 11 — Notifications & Communications

- [ ] Transactional email (payment receipt, settlement, dispute alert, KYC status)
- [ ] SMS (OTP, payment confirmation)
- [ ] WhatsApp (payment links, receipts)
- [ ] In-app real-time notifications
- [ ] Notification templates editable from admin panel
- [ ] Per-merchant notification preferences per event
- [ ] Customer receipt email (merchant-configurable on/off)

---

## ⬜ PHASE 12 — Analytics & Reports

- [ ] Merchant: revenue over time, success rate trend, payment method split, peak hours heatmap, refund rate, failed txn reasons
- [ ] Admin: platform GMV, fee revenue, merchant growth, provider performance (success rate + latency)
- [ ] Pre-built reports: GST (monthly), Settlement, Refunds, Transactions, Disputes, Merchant Fee
- [ ] Custom date range reports (async generation with progress)
- [ ] Download: CSV, Excel, PDF
- [ ] Scheduled email delivery of reports

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
# FeelPay — Implementation Checklist
> White-Label Payment Gateway & PaaS Platform (India)
> For: Antigravity Dev Team — Review, Fix & Check Off

---

## ✅ PHASE 0 — Project Setup

- [x] Monorepo structure with merchant dashboard, admin panel, hosted checkout, backend, shared types, shared UI component library
- [x] Environment variable files for all apps
- [ ] Git hooks — lint + type-check on commit
- [ ] CI/CD pipeline (run tests on PR, deploy staging on merge, manual-gate prod deploy)
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
- [ ] `payment_links`
- [x] `transactions` — partitioned monthly (L204)
- [ ] `transaction_events` — append-only state log
- [x] `refunds` (L231)
- [x] `disputes` (L250)
- [x] `settlements` + `settlement_transactions` (L270, L291)
- [/] `payouts` (L87 - Missing updated_at)
- [/] `bank_accounts` + `vpa_accounts` (L107 - Missing updated_at)
- [x] `subscriptions` + `subscription_invoices` (L122, L136)
- [ ] `customers` — tokenized vault
- [ ] `saved_payment_methods`
- [x] `fee_rules` (L149)
- [x] `audit_logs` — immutable, append-only (L164)
- [ ] `notification_queue`
- [ ] `report_jobs`
- [x] Indexes on: merchant_id, status, created_at, gateway_reference_id
- [ ] Row-level security for tenant isolation
- [x] Foreign key constraints enforced


---

## ⬜ PHASE 2 — Auth & Authorization

- [ ] Email + password login
- [ ] TOTP / MFA (Google Authenticator compatible)
- [ ] Email OTP / magic link login
- [ ] SMS OTP login
- [ ] JWT access tokens (short TTL)
- [ ] Refresh tokens (long TTL, revocable, rotated on use)
- [ ] Force logout all sessions
- [ ] API key authentication for merchant programmatic access
- [ ] IP whitelisting per API key (merchant opt-in)
- [ ] Role-based access control: super_admin, admin, support, tenant_admin, merchant_owner, merchant_admin, merchant_staff, merchant_viewer
- [ ] Granular permissions: transactions:read, transactions:refund, payouts:create, settings:write, etc.
- [ ] Permission checks enforced at API layer — not just UI
- [ ] Merchant sub-user invite + role assignment
- [ ] Protected routes — unauthenticated users redirected to login, returned to original URL after login
- [ ] Rate limiting on all auth endpoints (per IP + per user)
- [ ] Account lockout after N failed attempts
- [ ] New device/IP login → email alert
- [ ] Password strength enforcement + history (no reuse of last 5)
- [ ] CAPTCHA on login and signup

---

## ⬜ PHASE 3 — Merchant Onboarding & KYC

- [ ] Multi-step registration form with per-step validation
- [ ] Business info: name, email, phone, PAN, GSTIN
- [ ] Business type selection: Pvt Ltd, LLP, Sole Proprietor, Partnership, Trust, NGO
- [ ] Document upload with drag-and-drop and preview: PAN, GST certificate, COI/deed, cancelled cheque, address proof
- [ ] Upload direct to object storage (not through backend memory)
- [ ] Email + phone OTP verification during registration
- [ ] Registration progress saved — can resume incomplete
- [ ] KYC status machine: pending → under_review → approved / rejected / resubmit_required
- [ ] Auto-verification: PAN, GST, bank penny drop
- [ ] Aadhaar eKYC (DigiLocker / UIDAI provider)
- [ ] Rejection with specific fields flagged + resubmission flow
- [ ] Maker-Checker approval for high-value merchants (2 admin sign-off)
- [ ] Activation email on KYC approval
- [ ] Post-KYC: MCC code assignment
- [ ] Post-KYC: per-txn / daily / monthly transaction limits
- [ ] Post-KYC: allowed payment methods configuration
- [ ] Post-KYC: fee plan assignment
- [ ] Post-KYC: settlement cycle (T+1, T+2, instant)
- [ ] Post-KYC: live + test API key generation
- [ ] Post-KYC: custom branding upload (logo + color for hosted checkout)

---

## ⬜ PHASE 4 — Payment Gateway Core

### Payment Methods (India)
- [ ] UPI — Intent, QR collect, VPA push
- [ ] UPI Autopay — mandate creation + execution
- [ ] Cards — Visa, Mastercard, RuPay, Amex (debit + credit)
- [ ] Net Banking — HDFC, SBI, ICICI, Axis, Kotak + others
- [ ] Wallets — Paytm, PhonePe, Amazon Pay, Mobikwik
- [ ] EMI — credit card, no-cost, cardless
- [ ] BNPL — Lazypay, Simpl
- [ ] Virtual accounts (NEFT / RTGS bank transfer)
- [ ] International cards — Phase 2

### Provider Routing
- [ ] Provider abstraction layer — unified interface for all providers
- [ ] Razorpay (primary), PayU (fallback), Cashfree, Juspay (UPI routing)
- [ ] Intelligent routing by success rate, cost, method, merchant config
- [ ] Auto-failover to next provider on failure
- [ ] Circuit breaker per provider
- [ ] Provider health monitoring

### Transaction Flow
- [ ] Order creation
- [ ] Payment initiation returning session token
- [ ] Hosted checkout page (redirect flow)
- [ ] Embedded checkout (iframe with postMessage communication)
- [ ] Payment status polling + real-time status updates (SSE/WebSocket)
- [ ] Transaction state machine: created → initiated → processing → success / failed / cancelled / expired
- [ ] Idempotency key support on all write endpoints
- [ ] Payment link generation
- [ ] UPI QR code generation
- [ ] Order and link expiry handling

### Refunds
- [ ] Full refund
- [ ] Partial refund
- [ ] Batch refund
- [ ] Refund reason codes
- [ ] Refund status tracking + webhook delivery
- [ ] Refund event timeline in dashboard

---

## ⬜ PHASE 5 — Merchant Dashboard

### Layout
- [ ] Collapsible sidebar, mobile-responsive
- [ ] Live / Test mode toggle — globally changes API key context, clearly labeled
- [ ] Notification bell with unread count
- [ ] Loading skeletons on all data-heavy views
- [ ] Empty states with CTAs on all list views
- [ ] Error boundaries per major section
- [ ] URL-synced filters (shareable filtered URLs)
- [ ] Dark mode

### Home
- [ ] KPI cards: revenue today, transaction count, success rate, pending settlements
- [ ] Revenue chart (daily / weekly / monthly toggle)
- [ ] Recent transactions list
- [ ] Contextual alerts: KYC pending, dispute action needed, upcoming settlement

### Transactions
- [ ] List with filters: date range, status, payment method, amount range
- [ ] Search: transaction ID, order ID, customer email/phone
- [ ] Transaction detail: full event timeline, method details, customer info
- [ ] Initiate refund from detail (modal: amount + reason)
- [ ] Export CSV / Excel (async for large ranges)
- [ ] Bulk export for reconciliation

### Payment Links
- [ ] Create link: amount, description, expiry, partial payment toggle
- [ ] List with status badges
- [ ] Share via copy, WhatsApp deep link, SMS
- [ ] Per-link analytics: views, attempts, conversions
- [ ] Deactivate / expire link

### Settlements
- [ ] Next settlement date + estimated amount
- [ ] Settlement history with UTR numbers
- [ ] Transaction breakdown per settlement
- [ ] Download settlement report
- [ ] Bank account management (add / update with re-verification)

### Disputes
- [ ] Active disputes list with deadline countdown
- [ ] Dispute detail: customer reason + linked transaction
- [ ] Evidence upload (files + text)
- [ ] Status flow: open → evidence_submitted → won / lost

### Customers
- [ ] Customer list (tokenized, consent-based)
- [ ] Transaction history per customer
- [ ] Saved payment methods view
- [ ] Export

### Subscriptions
- [ ] Create plan: billing cycle, amount, trial period
- [ ] Subscription list + status
- [ ] Pause / cancel subscription
- [ ] Invoice history
- [ ] Failed payment retry configuration

### API Keys & Webhooks
- [ ] Live + Test key display (masked, reveal-on-click, copy)
- [ ] Key rotation with confirmation
- [ ] Webhook config: URL, secret, event subscriptions
- [ ] Webhook delivery log: request + response per event
- [ ] Test webhook trigger
- [ ] IP whitelist management

### Settings
- [ ] Business profile edit
- [ ] Team management: invite by email, assign role, revoke access
- [ ] Notification preferences per event type (email, SMS, in-app)
- [ ] Branding: logo + color with live checkout preview
- [ ] Settlement cycle preference
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

## ⬜ PHASE 6 — Admin Panel

### Layout
- [ ] Separate app from merchant dashboard
- [ ] Global search: merchant / transaction / user by ID or email
- [ ] Production environment clearly marked (banner / color indicator)

### Platform Dashboard
- [ ] GMV (daily / weekly / monthly)
- [ ] Active merchant count, platform fee earned, overall success rate
- [ ] Merchant signups chart
- [ ] Dispute rate chart
- [ ] Provider-wise transaction split

### Merchant Management
- [ ] List with filters: status, KYC state, GMV tier, join date
- [ ] Merchant detail: profile, KYC docs, config, transaction history
- [ ] Approve / reject / suspend / reactivate
- [ ] Edit limits + fee plan
- [ ] Impersonate merchant (view dashboard as merchant — full audit log entry)
- [ ] Internal notes on merchant profile

### KYC Queue
- [ ] Pending queue sorted by submission date
- [ ] Inline document viewer (PDF + images)
- [ ] Approve / Reject (mandatory reason on reject)
- [ ] Flag specific fields for resubmission
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
- [ ] Live health dashboard: up/down, success rate, latency
- [ ] Routing rules configuration
- [ ] Circuit breaker status + manual override
- [ ] Provider credential management (masked display)

### Tenant / White-label Management
- [ ] Tenant CRUD
- [ ] Branding + custom domain config per tenant
- [ ] Per-tenant fee + settlement config
- [ ] Tenant admin can manage their own merchants

### User & Role Management
- [ ] Admin user list + role assignment
- [ ] Activity log per admin user
- [ ] 2FA mandatory for all admin accounts

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
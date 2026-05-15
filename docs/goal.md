# Merchant Dashboard Completion Goal

Build and complete the merchant dashboard for our own branded Indian payment gateway and payment aggregator-ready platform.

Stack:
- Frontend: React
- Backend: Rust
- No Stripe, Razorpay, Cashfree, PayU, PhonePe PG, or other third-party gateway connectors.
- Do not fake live payment success.
- Do not implement live-money processing unless the repo already has legally approved payment/acquirer rails.

## Main Objective

Complete a production-grade merchant dashboard with proper frontend, backend APIs, database models, migrations, tests, security, RBAC, audit logs, and documentation.

The dashboard must be compliance-ready for India but must not claim legal/RBI readiness. Live mode must stay blocked until merchant KYC/KYB is approved.

## First Inspect The Repo

Before coding:
1. Detect existing frontend architecture, routes, components, styling, auth, API client, build commands, and test commands.
2. Detect Rust backend framework, database layer, migrations, auth middleware, models, routes, logging, and tests.
3. Reuse existing conventions.
4. Do not rewrite from scratch.
5. Do not leave TODO placeholders for core functionality.

## Required Merchant Dashboard Modules

### 1. Overview
Show today volume, total volume, success rate, failed payments, pending settlements, refunds, disputes, payment links, API health, and recent transactions. Include date filters and empty states.

### 2. Merchant Profile and KYC/KYB
Business profile, legal name, brand name, PAN, GSTIN, CIN/LLPIN, website/app URL, support email, support phone, address, bank account, KYC status, document metadata, and missing requirements checklist. Block live mode until approved.

KYC states:
- draft
- submitted
- under_review
- approved
- restricted
- rejected

### 3. Transactions
List, search, filter, paginate, export CSV, and view transaction details.

Transaction statuses:
- created
- pending
- authorized
- captured
- success
- failed
- expired
- refunded
- partially_refunded
- disputed

Detail page must show customer, order, payment method, timeline, fees, refunds, settlement mapping, webhook history, and developer debug info only for allowed roles.

### 4. Settlements
Show available balance, pending balance, holds, next settlement estimate, settlement list, and detail pages.

Settlement statuses:
- scheduled
- processing
- paid
- failed
- reversed
- on_hold

Settlement detail must show UTR/reference, bank account, gross amount, fees, taxes, refunds, chargebacks, adjustments, net amount, and linked transactions.

### 5. Refunds
List refunds, show detail, and allow full/partial refund from transaction detail. Validate refundable amount. Require reason and idempotency key. Refunds must affect ledger state.

Refund statuses:
- requested
- processing
- succeeded
- failed
- cancelled

### 6. Disputes
List and detail disputes with transaction, reason, amount, deadline, evidence metadata, status, and outcome.

Dispute statuses:
- opened
- evidence_required
- submitted
- won
- lost
- closed

### 7. Payment Links
Create, list, view, copy, expire, and deactivate payment links.

Fields:
- amount in INR
- description
- customer name/email/mobile
- expiry
- metadata
- status

Statuses:
- active
- paid
- expired
- cancelled

### 8. Developers
API keys:
- list masked keys
- create key
- revoke key
- show secret once only
- test/live environment
- block live keys until KYC approved

Webhooks:
- create/update/delete endpoint
- signing secret
- subscribed events
- delivery logs
- retry/resend
- never expose full secrets again

Include internal API docs page for authentication, idempotency, payment creation example, and webhook signature verification.

### 9. Reports
Transactions, settlements, refunds, fees/taxes, CSV export, and date filters.

### 10. Team and RBAC
Roles:
- owner
- admin
- developer
- finance
- support
- read_only

Add invite flow if email exists; otherwise pending invite model/API/UI. Enforce permissions on frontend and backend.

### 11. Settings
Business profile, branding, bank details update request, notification preferences, webhook shortcut, security settings if supported by existing auth.

### 12. Audit Logs
Append-only merchant audit logs for login, API key changes, webhook changes, refunds, invites, bank update requests, and KYC submission.

Store:
- actor id
- tenant id
- merchant id
- action
- target
- metadata
- IP/user-agent if available
- timestamp

### 13. Notifications
Database-backed notifications if no existing notification system exists.

Alerts:
- KYC incomplete
- settlement failed
- webhook failing
- high failure rate
- dispute deadline
- API key changed

### 14. Internal Platform Scoping / Multi-Merchant Isolation
Every record must be scoped by `merchant_id`; internal platform records may also carry `tenant_id` where the current architecture requires it for backend isolation. `tenant_id` is internal platform scoping, not a merchant-facing white-label or reseller concept. Never allow cross-tenant or cross-merchant access. Add isolation tests.

## Required Data Models

Add or complete these if missing:
- InternalTenantScope
- Merchant
- MerchantUser / Membership
- KycProfile
- KycDocument
- BankAccount
- Transaction
- PaymentAttempt
- LedgerEntry
- Settlement
- SettlementItem
- Refund
- Dispute
- PaymentLink
- ApiKey
- WebhookEndpoint
- WebhookEvent
- WebhookDelivery
- ReportExport
- Notification
- AuditLog
- IdempotencyKey

## Backend Requirements

- Use money minor units only. INR paise as integer.
- Never use floats for money.
- Add migrations.
- Add proper indexes.
- Add pagination.
- Add filtering/search.
- Add idempotency for sensitive operations.
- Add authorization checks to every merchant endpoint.
- Add audit logs for sensitive actions.
- Add structured errors.
- Validate input server-side.
- Never trust merchant_id from request body if it can come from auth/session/path.

## Frontend Requirements

- Use existing design system.
- Responsive desktop/mobile pages.
- Loading states.
- Empty states.
- Error states.
- Unauthorized states.
- Form validation.
- Confirmation dialogs.
- Mask API keys, webhook secrets, PAN, GST, and bank details.
- Add merchant navigation:
  - Overview
  - Transactions
  - Settlements
  - Refunds
  - Disputes
  - Payment Links
  - Developers
  - Reports
  - Team
  - Audit Logs
  - Settings

## Security Requirements

- Hash API keys.
- Store only API key prefix and hash.
- Show API key secret only once.
- Securely generate webhook secrets.
- Do not log secrets or sensitive KYC/bank data.
- Enforce RBAC server-side.
- Enforce tenant isolation.
- Add CSRF/CORS protections based on existing auth architecture.
- Add rate-limit hooks if existing middleware supports it.

## India-Specific Requirements

- Display INR.
- Store money in paise.
- Use Asia/Kolkata display timestamps.
- Add PAN/GSTIN/Indian phone validation where appropriate.
- Block live mode until KYC/KYB approval.
- Document that software readiness is not legal/RBI authorization.

## Tests Required

Add tests for:
- RBAC
- tenant isolation
- refund validation
- API key secret shown once
- webhook secret handling
- settlement calculations
- CSV export authorization
- KYC gating for live mode
- audit log creation

## Documentation Required

Create or update:
- docs/merchant-dashboard.md
- API route summary
- RBAC matrix
- data model summary
- local setup/run commands
- known limitations

## Definition of Done

The merchant can use a complete dashboard to manage transactions, settlements, refunds, disputes, reports, payment links, API keys, webhooks, team members, audit logs, and settings.

All core pages must work with real backend APIs, not hardcoded fake production data.

Build, formatting, linting, and tests must pass. If any test cannot run because of missing external services, document the exact reason and provide the closest successful validation.

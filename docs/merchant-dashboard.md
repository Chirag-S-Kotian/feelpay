# Merchant Dashboard

This dashboard is the merchant-facing React app in `dashboard/`. It talks to the Rust API only through `dashboard/src/hooks/useApi.ts`; protected requests require the merchant JWT or an environment-specific merchant API key. The company-admin app under `admin/` is a separate surface.

## Scope

- Own-brand Indian payment gateway and payment aggregator-ready merchant dashboard.
- Internal simulator/rail abstractions only. Do not add third-party gateway connectors here.
- Money values are INR paise as integers. UI formatting must not change stored or calculated money into floats.
- Live mode stays blocked until onboarding/KYC preflight returns `can_go_live=true`.

## Frontend Routes

- `/` overview: volume, success/failure signals, settlements, links, health, and recent activity.
- `/payments`: transaction list, search/filter, detail drawer, timelines, fees, full/partial refund action, refund status, and webhook history.
- `/refunds`: refund list, search/filter, payment mapping, status, customer context, and detail drawer.
- `/settlements`: balances, settlement list, reconciliation, and details.
- `/payment-links`: create/list/copy/expire/deactivate payment links.
- `/disputes`: dispute list/detail and evidence workflow.
- `/developers`: API keys, docs, idempotency, webhook events, delivery logs, and go-live checks.
- `/team`: invite flow and role assignment for owner, admin, developer, finance, support, and read-only merchant users.
- `/reports`: report hub for transactions, settlements, refunds, fees/taxes, date windows, and CSV/export entry points.
- `/exports`, `/settlement-reports`, `/scheduled-reports`, `/analytics`: backing report/export workflows.
- `/settings`: profile, business settings, team invites/roles, security, notification shortcuts, and webhook settings.
- `/notifications`: database-backed alert inbox plus merchant and per-user notification preferences.
- `/audit-logs`: merchant-scoped sensitive action log with actor, target, metadata, IP, user-agent, and timestamp.
- `/onboarding`: KYC/KYB, bank, readiness, and live-mode gating.
- `/bank-accounts`: bank account collection, verification, primary account, and payout readiness.

## Backend Route Summary

- Merchant auth/profile/team/settings: `/v1/merchants/*`.
- Onboarding/KYC/live preflight: `/v1/onboarding/*`.
- Payments/refunds/timelines/details: `/v1/payments/*`; refund mutations require idempotency keys and write audit logs.
- Refund list/detail: `/v1/refunds/*`.
- Settlements/reconciliation/balance: `/v1/settlements/*`.
- Payment links: `/v1/payment-links/*`.
- Disputes: `/v1/disputes/*`.
- Merchant exports/reports/preferences/webhook tests: `/v1/merchants/{merchant_id}/*`.
- Merchant audit logs: `/v1/merchants/audit-logs`.
- Merchant notification inbox: `/v1/notifications`.
- Developer webhook/API logs: `/v1/developers/*`.

Handlers must derive merchant identity from auth/session/API-key context. Do not trust `merchant_id` from a request body when auth or path context exists.

## RBAC Matrix

| Role | Allowed Areas |
| --- | --- |
| `merchant_owner` | All merchant settings, team, keys, refunds, webhooks, exports, reports |
| `merchant_admin` | Operational settings, team, refunds, webhooks, exports, reports |
| `merchant_developer` | API keys, webhook config/logs, developer docs, test tools |
| `merchant_finance` | Transactions, settlements, refunds, exports, scheduled reports |
| `merchant_support` | Transactions, disputes, customers, read-only operational details |
| `merchant_read_only` | View-only dashboard, reports, transactions, settlements |

Backend permission checks are required for sensitive mutations: API key changes, webhook changes, refunds, bank changes, team invites/removals, KYC submission, exports, and settings updates. Frontend hiding is only a UX aid, not authorization.

## Data Model Summary

Required merchant-domain tables/models include internal platform scopes, merchants, users/memberships, KYC profiles/documents, bank accounts, payment orders/attempts, ledger entries, settlements/items, refunds, disputes, payment links, developer API keys, webhook endpoints/events/deliveries, report exports, notifications, audit logs, and idempotency keys.

Every merchant-owned row must be scoped by `merchant_id`. Existing `tenant_id` columns are internal platform-scoping fields for backend isolation only; they are not merchant-facing white-label, reseller, or partner concepts. Cross-merchant reads and writes should be tested by querying or mutating with a different authenticated merchant.

## Security Rules

- Store and calculate money in integer minor units.
- Hash API keys and secrets; show newly generated secrets once only.
- Mask API keys, webhook secrets, PAN/GSTIN, bank account numbers, and KYC document metadata in UI/logs.
- Use idempotency keys for refunds and other sensitive mutations.
- Audit sensitive actions with actor, tenant, merchant, action, target, metadata, IP/user-agent, and timestamp.
- Keep live-mode keys/actions blocked until KYC/KYB approval and live preflight pass.
- Do not log raw secrets, full bank details, PAN, KYC documents, or webhook signing secrets.

## Local Setup

```bash
cargo check
cd dashboard && pnpm exec tsc --noEmit
cd dashboard && pnpm build
pnpm test:e2e:full-merchant
./scripts/graphify update .
```

Run only checks relevant to touched files during normal development. Use the full merchant E2E suite before claiming the dashboard is complete.

## Known Limitations

- This software does not claim RBI/legal readiness or approval.
- Live-money processing must remain disabled unless approved acquirer/payment rails already exist in the repo and are configured outside this merchant-dashboard scope.
- Some merchant modules reuse broader platform endpoints; verify internal platform scope and merchant isolation whenever adding fields or filters.
- Sensitive-field hardening is forward-safe after `20260515000001_merchant_secret_storage_hardening.sql`: new API keys use hashed lookup rows, new webhook secrets are encrypted, and new PAN/GSTIN/KYC writes store masked display values plus encrypted payloads. Existing historical plaintext rows still require a one-time migrated-Postgres backfill before claiming legacy data is fully remediated.
- This pass added centralized merchant-route RBAC checks and focused role tests, but a complete audit-log guarantee still needs a route-by-route production audit matrix against the live schema and traffic policy. Do not claim full audit completeness until that external review is done.

# FeelPay Admin Panel Runbook

## Purpose

Runbook for platform ops team using admin UI and admin APIs.

## Access control

1. Admin users only.
2. Use strong password + MFA.
3. Assign tenant-scoped role where possible.

## Daily operations

1. Review `Overview` metrics.
2. Check `KYC Queue` and SLA breaches.
3. Check `Transactions` and suspicious flags.
4. Check `Global Settlements` pending/failed.
5. Check `Notification Logs` delivery failures.
6. Check `Webhook Testing` and error rates.

## Incident playbooks

### 1) Transaction spike/failure spike

1. Open `Transactions` + filter by status.
2. Correlate with provider in analytics/revenue pages.
3. Apply emergency action (`flag`, `freeze`) if fraud pattern.
4. Notify engineering/on-call.

### 2) Settlement stuck

1. Open `Global Settlements`.
2. Search settlement id.
3. Apply action: approve / hold / mark paid with UTR.
4. Add reason in audit trail.

### 3) Webhook delivery failures

1. Open `Webhook Testing` and recent logs.
2. Validate endpoint and signature behavior.
3. Re-trigger test delivery.
4. Escalate to merchant integration contact.

### 4) KYC risk case

1. Open `KYC Queue`.
2. Review docs + audit history.
3. Approve/reject/resubmit with reason.
4. Ensure reason visible in merchant logs and notifications.

## Tenant operations

1. Open `Tenants`.
2. Create/update/deactivate tenant.
3. Manage branding/domain/email sending domain.
4. Set fee/settlement/revenue-share configs.
5. Review tenant merchants list.

## Credential operations

1. Open `Credentials`.
2. Rotate provider credentials.
3. Validate connectivity post-rotation.
4. Record rotation action in audit logs.

## Admin user operations

1. Open `Admin Users`.
2. Create user with least privilege role.
3. Set tenant scope where required.
4. Deactivate stale users.

## Audit and compliance

1. `Audit Logs` mandatory for sensitive changes.
2. For every manual action, add reason text.
3. Weekly review: admin actions, permission drift, deactivated users.

## Escalation

1. P1 fraud or settlement outage: immediate engineering + leadership bridge.
2. P2 merchant-specific outage: merchant support + integrations team.
3. Compliance incident: legal/compliance owner same day.

## Useful endpoints

1. `GET /v1/admin/tenants`
2. `POST /v1/admin/tenants`
3. `PUT /v1/admin/tenants/{id}`
4. `GET /v1/admin/tenants/{id}/merchants`
5. `GET /v1/admin/audit-logs`

## Pre-deploy checklist for admin changes

1. `cargo check --all-targets`
2. `cd admin && npm run build`
3. Verify tenant CRUD and audit log entries.
4. Verify role-based access paths.

# Staging Deployment

## Purpose

Use staging to validate the exact build, migration, auth, integration, and safety behavior intended for production without touching the production Neon branch or enabling fake live payout behavior.

## Required Environment

Gateway:
- `APP_ENV=staging`
- `PAYMENT_MODE=test`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET`
- `MASTER_API_KEY`
- `VAULT_ENCRYPTION_KEY`
- `WEBHOOK_SIGNING_SECRET`
- `CORS_ALLOWED_ORIGINS`

Admin server:
- `APP_ENV=staging`
- `PAYMENT_MODE=test`
- `DATABASE_URL`
- `JWT_SECRET`
- `VAULT_ENCRYPTION_KEY`
- `ALLOW_SECRET_REVEAL=false`
- `ADMIN_CORS_ALLOWED_ORIGINS`
- `ADMIN_HOST`
- `ADMIN_PORT`

Frontend:
- `VITE_API_BASE_URL` for merchant dashboard when deployed outside the default local origin
- `VITE_ADMIN_API_BASE_URL` for admin UI

Rules:
- do not commit real staging values
- do not reuse the production Neon database
- keep payout providers in test mode unless an official approved live adapter exists
- keep secret values out of logs, screenshots, and tickets

## Neon Staging Branch

1. Create or reuse a dedicated Neon staging/test branch, separate from production and local development.
2. Store that URL as staging `DATABASE_URL`.
3. Store a separate validation branch URL as `TEST_DATABASE_URL` or `NEON_TEST_DATABASE_URL` for `./scripts/validate_test_db.sh`.
4. Record the branch name, parent branch, migration head, and restore point before each deployment.
5. Never point staging validation scripts at production, `main`, or `live` URLs.

## Migration Order

1. Run `./scripts/validate_test_db.sh` against the safe validation branch.
2. Review pending migrations and confirm there is no unapproved destructive DDL.
3. Apply the same ordered migration set to the staging Neon branch.
4. Start the backend only after migrations succeed.
5. Run health/readiness checks, then the smoke checklist below.

## Build And Deploy

Backend gateway:
```bash
cargo run
```

Company-admin server:
```bash
cd admin/server
cargo run
```

Admin frontend:
```bash
cd admin
pnpm install
pnpm build
```

Merchant dashboard:
```bash
cd dashboard
pnpm install
pnpm build
```

Current repo deploy evidence:
- local infra helper exists at `docker-compose.local.yml`
- frontend build config exists in `admin/package.json` and `dashboard/package.json`
- no production/staging platform manifest is committed in this repo, so the hosting-specific deploy step remains environment-owned

## Health Checks

Gateway:
- `GET /health`
- `GET /v1/health`
- `GET /ready`
- `GET /v1/ready`

Admin server:
- `GET /health`
- `GET /ready`
- confirm configured admin base URL responds before opening the UI
- confirm authenticated admin API traffic returns `x-request-id`

Expected:
- liveness endpoints return success when the process is up
- readiness fails if required backing services are unavailable
- no secret material appears in startup logs

## Automated Smoke Coverage Already Present

Safe repo-backed automation:
- `cargo test`
  - health endpoint shape
  - webhook signature verification
  - startup safety
  - request ID validation
  - rate-limit bucket classification
- `./scripts/validate_test_db.sh`
  - safe Neon branch guard
  - migration application
  - online Rust tests
- `cd admin && pnpm exec vitest run`
  - admin shell smoke
- `cd dashboard && pnpm exec vitest run`
  - dashboard component smoke
- `cd admin && pnpm exec playwright test --project=api`
  - admin API surface where a configured environment is available
- `cd dashboard && pnpm test:e2e:smoke`
  - merchant auth/session fixture coverage

Do not treat fixture-backed browser tests as proof of live staging provider behavior.

## Manual Staging Smoke Checklist

Core service:
- [ ] backend `/health` and `/ready` pass
- [ ] admin `/health` and `/ready` pass
- [ ] admin service base URL loads and authenticated responses include `x-request-id`
- [ ] merchant login creates a session, refresh works, logout clears access
- [ ] admin login creates a session, logout clears access
- [ ] merchant credentials receive denial on admin routes

Admin and integrations:
- [ ] Admin Integration Center loads
- [ ] SMTP test action is available only for configured test provider state
- [ ] SMS/Twilio test action is available only for configured test provider state
- [ ] notification-template preview/test uses safe variables and does not expose OTP or secrets
- [ ] KYC test provider path runs without auto-approving the merchant
- [ ] merchant KYC review approval flow requires admin action

Live-mode safety:
- [ ] live API key creation/use remains blocked before onboarding plus admin approval
- [ ] live API key use is allowed only after approval gates pass
- [ ] live payout remains blocked when no official approved adapter exists

Money/risk/reconciliation:
- [ ] reconciliation batch create, detail, and close flow works
- [ ] reconciliation close is blocked while open mismatches exist
- [ ] risk rule create, evaluation, and case workflow works
- [ ] payout mutation endpoints preserve safe status transitions

Abuse and webhook safety:
- [ ] login, OTP request/verify, webhook replay, payout mutation, admin-sensitive, and report/export routes rate-limit as configured
- [ ] repeated provider webhook callback with the same provider event ID is idempotent
- [ ] failed merchant webhook delivery schedules retries and reaches dead-letter behavior at the configured limit

## Rollback

1. Stop new write traffic or enable maintenance mode if money correctness is at risk.
2. Restore the staging Neon branch from the recorded restore point or apply a reviewed reverse migration.
3. Redeploy the previous known-good artifact.
4. Re-run health/readiness checks and the smoke checklist.
5. Record the failed migration head, app version, incident notes, and rollback owner.

## Remaining Manual Checks

These cannot be proven by local fixtures alone:
- real staging DNS/TLS and hosting configuration
- actual SMTP/SMS/KYC provider credentials and callbacks
- deployed CORS/origin behavior
- live staging Neon branch ownership and restore execution
- real rate-limit behavior behind the staging proxy/load balancer
- provider webhook retries against a real staging endpoint

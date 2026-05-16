# Staging Deployment

## Purpose

Use staging to validate the exact build, migration, auth, integration, and safety behavior intended for production without touching the production Neon branch or enabling fake live payout behavior.

Actual deployed staging validation is out of scope until a staging environment exists. This document prepares the repo for the first staging deployment only.

## Current Deployment Decision State

No provider-specific staging manifest is committed in this repo. Provider-neutral staging files now exist for first deployment rehearsal:
- `docker-compose.staging.yml`
- `deploy/staging/gateway.Dockerfile`
- `deploy/staging/admin-server.Dockerfile`
- `deploy/staging/dashboard.Dockerfile`
- `deploy/staging/admin-frontend.Dockerfile`
- `deploy/staging/nginx.conf`
- `deploy/staging/spa-nginx.conf`
- `deploy/staging/.env.example`

Choices still required before the first deploy:
- hosting provider for the two Rust services
- static hosting or edge provider for the two Vite frontends
- reverse proxy / ingress product
- canonical staging domains
- TLS certificate issuer and renewal owner
- secret manager used by the chosen platform

Until those decisions are made, keep deployment artifacts provider-neutral and do not add fake platform files that imply a selected host.

## Concrete Provider-Neutral Files

Use the committed OCI/Compose baseline when the selected host accepts containers:
- gateway image: `deploy/staging/gateway.Dockerfile`
- admin API image: `deploy/staging/admin-server.Dockerfile`
- merchant SPA image: `deploy/staging/dashboard.Dockerfile`
- admin SPA image: `deploy/staging/admin-frontend.Dockerfile`
- local/reference orchestration: `docker-compose.staging.yml`
- host-based routing reference: `deploy/staging/nginx.conf`
- frontend history fallback: `deploy/staging/spa-nginx.conf`
- env checklist: `deploy/staging/.env.example`

Equivalent managed-service deployments must preserve the same build commands, runtime variables, origins, health checks, and route mapping even if they do not run Compose.

## Target Staging Topology

| Surface | Suggested public origin | Upstream target |
| --- | --- | --- |
| Merchant dashboard | `https://dashboard-staging.example.com` | static frontend |
| Admin frontend | `https://admin-staging.example.com` | static frontend |
| Gateway API | `https://api-staging.example.com` | gateway service on `HOST:PORT` |
| Admin API | `https://admin-api-staging.example.com` | admin service on `ADMIN_HOST:ADMIN_PORT` |

Use real project domains once selected; these placeholders are only a shape for routing and CORS planning.

## Required Environment

Gateway required:
- `APP_ENV=staging`
- `PAYMENT_MODE=test`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET`
- `MASTER_API_KEY`
- `VAULT_ENCRYPTION_KEY`
- `WEBHOOK_SIGNING_SECRET`
- `CORS_ALLOWED_ORIGINS`

Gateway runtime settings already supported by code:
- `HOST`
- `PORT`
- `JWT_EXPIRY_HOURS`
- `VAULT_KEY_ID`
- `UPI_PSP_ID`
- `UPI_MERCHANT_VPA`
- `WEBHOOK_MAX_RETRIES`
- `WEBHOOK_INITIAL_DELAY_SECS`
- `SETTLEMENT_CYCLE_DAYS`
- `PLATFORM_FEE_PERCENTAGE`
- `RATE_LIMIT_PER_MINUTE`
- `TENANT_ISOLATION_MODE`
- `DATABASE_PROVIDER`
- `DATABASE_URL_NEON`
- `DATABASE_URL_DOCKER`

Admin server required:
- `APP_ENV=staging`
- `PAYMENT_MODE=test`
- `DATABASE_URL`
- `JWT_SECRET`
- `VAULT_ENCRYPTION_KEY`
- `ALLOW_SECRET_REVEAL=false`
- `ADMIN_CORS_ALLOWED_ORIGINS`
- `ADMIN_HOST`
- `ADMIN_PORT`

Optional service integrations already supported by code:
- `EMAIL_PROVIDER`
- `EMAIL_FROM_ADDRESS`
- `SENDGRID_API_KEY`
- `POSTMARK_API_KEY`
- `SMS_PROVIDER`
- `MSG91_API_KEY`
- `MSG91_SENDER_ID`
- `TWILIO_ACCOUNT_SID`
- `TWILIO_AUTH_TOKEN`
- `TWILIO_FROM_NUMBER`
- `PAYMENT_LINK_BASE_URL`
- `CHECKOUT_LINK_SIGNING_SECRET`
- `STORAGE_SIGNER_URL`
- `EXPORT_ARTIFACTS_DIR`
- `REPORT_EXPORT_DIR`
- `REPORT_STORAGE_DIR`
- DigiLocker variables already listed in `.env.example`

Merchant dashboard build-time variables:
- `VITE_API_BASE_URL` for merchant dashboard when deployed outside the default local origin
- `VITE_SANDBOX_API_BASE_URL`
- `VITE_LIVE_API_BASE_URL`

Admin frontend build-time variables:
- `VITE_ADMIN_API_BASE_URL`
- `VITE_DASHBOARD_BASE_URL`
- `VITE_ADMIN_DEBUG_TOOLS=false`

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
6. Use separate staging and validation branch credentials even when both descend from the same parent.

## Migration Order

1. Run `./scripts/validate_test_db.sh` against the safe validation branch.
2. Review pending migrations and confirm there is no unapproved destructive DDL.
3. Apply the same ordered migration set to the staging Neon branch.
4. Start the backend only after migrations succeed.
5. Run health/readiness checks, then the smoke checklist below.

## Build And Start

| Surface | Build command | Start / serve command |
| --- | --- | --- |
| Gateway API | `cargo build --release` | `./target/release/feelpay` |
| Admin server | `cd admin/server && cargo build --release` | `cd admin/server && ./target/release/feelpay-admin-server` |
| Merchant dashboard | `cd dashboard && pnpm install --frozen-lockfile && pnpm build` | serve `dashboard/dist/` with the chosen static host or reverse proxy |
| Admin frontend | `cd admin && pnpm install --frozen-lockfile && pnpm build` | serve `admin/dist/` with the chosen static host or reverse proxy |

Development-only commands remain useful locally, but they are not the staging artifact commands:
```bash
cargo run
cd admin/server
cargo run
```

Current repo deploy evidence:
- local infra helper exists at `docker-compose.local.yml`
- provider-neutral staging runtime baseline exists at `docker-compose.staging.yml` plus `deploy/staging/*`
- frontend build config exists in `admin/package.json` and `dashboard/package.json`
- no provider-specific platform manifest is committed in this repo, so host-specific secrets, DNS, and TLS remain environment-owned

Reference container flow:
```bash
docker compose --env-file deploy/staging/.env.example -f docker-compose.staging.yml config
docker compose --env-file /secure/path/staging.env -f docker-compose.staging.yml build
docker compose --env-file /secure/path/staging.env -f docker-compose.staging.yml up -d
```

The first command is safe for syntax validation only. The second and third commands require a real secret-managed staging env source, not the committed example file.

## Health Checks

Gateway:
- `GET https://api-staging.example.com/health`
- `GET https://api-staging.example.com/v1/health`
- `GET https://api-staging.example.com/ready`
- `GET https://api-staging.example.com/v1/ready`

Admin server:
- `GET https://admin-api-staging.example.com/health`
- `GET https://admin-api-staging.example.com/ready`
- confirm configured admin base URL responds before opening the UI
- confirm authenticated admin API traffic returns `x-request-id`

Expected:
- liveness endpoints return success when the process is up
- readiness fails if required backing services are unavailable
- no secret material appears in startup logs

## CORS And Origin Configuration

Use exact origins, not wildcard production-style shortcuts:
- gateway `CORS_ALLOWED_ORIGINS=https://dashboard-staging.example.com`
- admin server `ADMIN_CORS_ALLOWED_ORIGINS=https://admin-staging.example.com`
- admin frontend `VITE_DASHBOARD_BASE_URL=https://dashboard-staging.example.com`
- merchant dashboard `VITE_API_BASE_URL=https://api-staging.example.com`
- admin frontend `VITE_ADMIN_API_BASE_URL=https://admin-api-staging.example.com`

If a single apex domain is chosen instead, update the four origins together and verify browser preflight behavior before opening traffic.

## Reverse Proxy / Domain Mapping

Provider-neutral routing contract:
- `dashboard-staging.example.com/*` -> merchant static site
- `admin-staging.example.com/*` -> admin static site
- `api-staging.example.com/*` -> gateway upstream `HOST:PORT`
- `admin-api-staging.example.com/*` -> admin upstream `ADMIN_HOST:ADMIN_PORT`

Ingress requirements:
- preserve `Host`, `X-Forwarded-For`, and `X-Forwarded-Proto`
- forward websocket upgrades only if the chosen platform later needs them
- keep frontend history fallback enabled for both Vite SPAs
- do not mount admin API behind the merchant API origin unless CORS and auth behavior are re-reviewed

## TLS / HTTPS

- terminate HTTPS at the chosen ingress or edge provider
- redirect HTTP to HTTPS
- use valid certificates for every staging origin before browser validation
- enable HSTS only after HTTPS is stable on all chosen staging hosts
- keep internal upstreams private where the platform allows it
- do not claim deployed TLS readiness until the real domains and certificates exist

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
3. Redeploy the previous known-good gateway, admin server, and frontend artifacts together unless the incident is clearly isolated.
4. Re-run migrations only when the rollback plan requires it; do not improvise destructive DDL during incident response.
5. Re-run health/readiness checks and the smoke checklist.
6. Record the failed migration head, app version, incident notes, and rollback owner.

## Remaining Manual Checks

These cannot be proven by local fixtures alone:
- real staging DNS/TLS and hosting configuration
- actual SMTP/SMS/KYC provider credentials and callbacks
- deployed CORS/origin behavior
- live staging Neon branch ownership and restore execution
- real rate-limit behavior behind the staging proxy/load balancer
- provider webhook retries against a real staging endpoint

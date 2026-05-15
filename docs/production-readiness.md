# Production Readiness

## Environment

Required gateway variables:
- `APP_ENV`
- `PAYMENT_MODE`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET`
- `MASTER_API_KEY`
- `VAULT_ENCRYPTION_KEY`
- `WEBHOOK_SIGNING_SECRET`
- `CORS_ALLOWED_ORIGINS`

Required admin-server variables:
- `APP_ENV`
- `PAYMENT_MODE`
- `DATABASE_URL`
- `JWT_SECRET`
- `VAULT_ENCRYPTION_KEY`
- `ALLOW_SECRET_REVEAL=false`
- `ADMIN_CORS_ALLOWED_ORIGINS`

Mode rules:
- local/dev: `APP_ENV=development`, `PAYMENT_MODE=test`
- staging: `APP_ENV=staging`, `PAYMENT_MODE=test`
- production: `APP_ENV=production`, `PAYMENT_MODE=live`
- production startup must use non-default JWT, vault, master API, and webhook secrets.
- never log secret values; connection logs must stay redacted.

## Neon Setup

1. Keep production on the main Neon branch.
2. Create a separate Neon branch for migration rehearsal and destructive checks.
3. Run migrations on the branch first, validate the app, then promote the same migration set to production.
4. Keep point-in-time restore enabled and record the restore target before every production migration.

## Migration Safety

- Prefer additive migrations.
- Mark destructive changes explicitly in the migration name and review notes.
- Backfill in separate steps from schema changes when tables are large.
- Do not drop columns/tables until rollback and data-retention impact are approved.
- Validate with `./scripts/validate_test_db.sh` before production rollout.

Rollback:
1. Stop writes or place the system in maintenance mode if money correctness is at risk.
2. Restore from the latest known-good Neon restore point or apply a reviewed reverse migration when safe.
3. Re-run health/readiness checks and the smoke suite before reopening traffic.

## Provider Setup

- SMTP, SMS/OTP, KYC/KYB, acquirer, and bank/payout credentials must be stored through encrypted admin integration flows only.
- Keep test providers in test environments only.
- Live bank/payout actions stay blocked until an official adapter exists and `approved_for_live=true`.
- Confirm webhook signing secrets and provider callback allowlists before go-live.

## Rate Limits

- Global HTTP limiter remains active for normal traffic.
- Dedicated buckets exist for login, OTP request, OTP verify, webhook replay, payout mutation, admin-sensitive routes, and report/export routes.
- API-key traffic has its own per-key limiter and audit logging.
- Rate-limit events must stay redacted and should be monitored during incident review.

## Backup And Restore

Before deploy:
- verify Neon restore window
- record current migration head
- record current app version
- export provider configuration metadata only, never raw secrets

After deploy:
- verify backup freshness
- verify restore procedure owner
- verify latest health/readiness checks

## Health Checks

- liveness: `/health`, `/v1/health`
- readiness: `/ready`, `/v1/ready`
- readiness must fail if PostgreSQL is unavailable, and in live mode it must also fail if Redis is unavailable.

## Incident Response

1. Triage with request ID, merchant ID, provider reference, and redacted event summary.
2. For payout or ledger incidents, stop new mutations before replaying work.
3. For webhook failures, inspect retries/dead-letter queue before manual replay.
4. For suspicious auth or OTP traffic, tighten rate limits and review logs before unlocking accounts.
5. For migration incidents, use the rollback plan above and record the exact migration head.

## Current Production Limitations

- This software does not claim RBI/legal authorization by itself.
- Live bank payout remains blocked until an official provider adapter is configured and approved.
- Production correctness still depends on real provider behavior, live SMS/SMTP delivery, Neon availability, and operational monitoring outside this repository.

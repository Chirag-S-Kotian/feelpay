# Staging Smoke Results

## Run Summary

- Result: `blocked`
- Scope: actual deployed staging environment validation against `docs/staging-deployment.md`
- Date: `2026-05-15`
- Reason blocked: this workspace does not currently provide a verified staging base URL, admin base URL, or staging test credentials. The only repo hint, a commented workflow default for `https://staging.feelpay.in`, did not resolve during validation.
- Decision: repo-backed preflight is complete; actual deployed staging smoke cannot start until operator-owned deployment inputs are supplied.
- Evidence boundary: current checks show no available evidence of a staging deployment from this workspace, not proof that no staging deployment exists anywhere.

## Evidence Collected

Environment evidence:
- `.env` exists locally, but only `PAYMENT_MODE=test` is declared for mode selection.
- `.env` and `admin/.env` use local CORS origins, so they are not staging deployment evidence.
- Neon project `feelpay` has a primary `production` branch and a separate non-primary `test` branch.
- A fresh Neon project check during this run still showed only `production` and `test`; no separate `staging` branch was present.
- Official Neon docs confirm branch-based recovery and instant restore are supported, and that production/staging are normally separate branches; this project currently lacks a dedicated `staging` branch to exercise that flow against.
- The local runtime `DATABASE_URL` host maps to the Neon `production` branch compute, while local `NEON_TEST_DATABASE_URL` maps to the separate `test` branch compute.
- `.env` includes separate Neon database variables for runtime and test validation; only the test URL is safe for automated online validation.
- shell runtime did not expose the required staging variables during this run.
- `.env`, `admin/.env`, and similar files are ignored by `.gitignore`; no real env values were committed.
- ignored-file and filename-only scans found `.env`, `admin/.env`, and `admin/server/.env`, but no hidden staging-specific env artifact.

Repository evidence:
- staging checklist: `docs/staging-deployment.md`
- rollback and restore guidance: `docs/staging-deployment.md`, `docs/production-readiness.md`
- backend health/readiness: `src/api/health.rs`
- admin health/readiness: `admin/server/src/main.rs`
- CORS configuration: `src/config.rs`, `admin/server/src/main.rs`
- sensitive-route rate limiting: `src/middleware/rate_limit.rs`
- provider callback idempotency: `admin/server/src/integrations.rs`
- Neon safe validation guard: `scripts/validate_test_db.sh`
- committed deploy evidence is limited to `docker-compose.local.yml`; no active staging deploy manifest is present.
- local `git branch -a` and remote `git ls-remote --heads origin` both show only `main`; no `staging` branch is present locally or remotely.
- `.github/workflows/pipeline.yml` and `.github/workflows/zap-staging.yml` are commented out, so there is no active CI/CD or staging scan workflow evidence in the repo.
- no second staging hostname convention was found beyond the commented `https://staging.feelpay.in` placeholder.
- public domain checks did not reveal a deployed target: certificate transparency returned no `feelpay.in` entries, and DNS-over-HTTPS returned `Status: 3` for `feelpay.in`, `staging.feelpay.in`, `api.feelpay.in`, `dashboard.feelpay.in`, and `pay.feelpay.in`.
- a bounded search across the payment repo and nearby Feel workspaces found no alternate staging URL or staging env artifact beyond the same commented placeholder.

## Checks Completed

| Area | Check | Result |
| --- | --- | --- |
| Environment | Required staging variable names documented without printing values | pass |
| Environment | Production/test DB separation rule exists in validation script | pass |
| Environment | Local Neon runtime/test URLs map to different Neon branches | pass |
| Environment | Local checked env files are not staging proof | pass |
| Rollback | Neon branch/restore process documented | pass |
| Safety | No fake live payout path enabled in repo state | pass |
| Health | Backend and admin health/readiness endpoints exist in code | pass |
| Webhooks | Provider callback idempotency exists in code | pass |
| Rate limits | Sensitive route buckets exist in code | pass |

## Scope Boundary

Completed in this run:
- repository and configuration preflight
- Neon branch-safety verification
- local staging-mode rehearsal against the safe Neon `test` branch

Not completed in this run:
- any check that requires a real deployed staging URL, deployed runtime metadata, or staging login credentials
- any check that would mutate or validate an unknown environment without proof that it is non-production

Automation boundary:
- `admin/playwright.config.ts` auto-starts local backend/admin services when targets are unavailable.
- `dashboard/playwright.config.ts` is pinned to `http://localhost:3000`.
- Therefore the current Playwright suites are useful repo regression coverage, but they are not evidence for a deployed staging environment.

## Local Staging Rehearsal

This is supplemental evidence only. It does not replace validation of an actually deployed staging environment.

| Check | Result |
| --- | --- |
| backend started with `APP_ENV=staging`, `PAYMENT_MODE=test`, and `DATABASE_URL` overridden to the Neon `test` branch | pass |
| backend `/health` on local rehearsal | `200` |
| backend `/ready` on local rehearsal | `200` |
| admin server started with `APP_ENV=staging`, `PAYMENT_MODE=test`, and `DATABASE_URL` overridden to the Neon `test` branch | pass |
| admin `/health` on local rehearsal | `200` |
| admin `/ready` on local rehearsal | `200` |

Observed during local rehearsal:
- backend startup continued with Redis unavailable because the run was in test mode, matching the documented non-live readiness behavior
- no production Neon database was used for the local rehearsal

## Checks Blocked

These need verified deployed staging coordinates or credentials before they can be truthfully marked pass/fail:

| Area | Check | Blocker |
| --- | --- | --- |
| Environment | `APP_ENV=staging` in deployed runtime | no verified staging runtime target |
| Environment | deployed staging Neon branch identity | no staging deployment metadata or safe branch identifier provided |
| Health | backend `/health` and `/ready` | no resolvable verified staging URL |
| Health | admin `/health` and `/ready` | no verified admin staging URL |
| Health | frontend apps load | no verified frontend staging URLs |
| Auth | merchant login/session | no staging merchant credentials |
| Auth | admin login/session | no staging admin credentials |
| Auth | merchant denial on admin routes | no staging credentials |
| Auth | sensitive admin permission checks | no staging admin role credentials |
| Integrations | SMTP test path | no staging admin access or provider config |
| Integrations | SMS/Twilio test path | no staging admin access or provider config |
| Integrations | KYC test provider path | no staging admin access or provider config |
| Integrations | secret masking in deployed UI | no deployed admin session |
| Merchant lifecycle | onboarding, KYC submission, admin approval | no staging sessions |
| Merchant lifecycle | live key blocked before approval | no staging sessions |
| Merchant lifecycle | live key allowed only after approval | no staging sessions |
| Money safety | live payout remains blocked | no staging admin session |
| Money safety | reconciliation batch create/detail/close | no staging admin session |
| Money safety | risk rule/evaluation/case flow | no staging admin session |
| Webhooks | deployed provider webhook idempotency | no provider callback path or credentials |
| Webhooks | deployed retry behavior | no reachable staging endpoint |
| Rate limits | proxy-backed throttling | no deployed proxy target |
| Rollback | restore rehearsal on staging/test branch | no safe branch target approved for restore testing |

## Security Notes

- A local ignored env file contains a real-looking database credential. It was not committed, but it should be treated as sensitive local material and rotated if it has been shared outside the intended operator path.
- The local runtime database URL maps to the Neon `production` branch, so local env files must not be reused as staging validation inputs.
- No secret values are reproduced in this report.
- No production Neon database was used for any staging smoke attempt.

## Inputs Needed To Finish Actual Bring-up

1. Verified merchant staging URL.
2. Verified admin staging URL.
3. Non-production staging database branch identifier or deployment metadata proving the runtime is not production.
4. Staging merchant test account.
5. Staging admin test accounts covering both an allowed role and a limited role for permission-denial checks.
6. Provider callback test path or sandbox event source for webhook idempotency/retry checks.
7. Deployed proxy/base URL path to exercise real rate-limit behavior.
8. Confirmation whether restore rehearsal is permitted on a disposable staging/test branch.

Operator handoff template:

```text
MERCHANT_STAGING_URL=
ADMIN_STAGING_URL=
STAGING_DATABASE_BRANCH=
MERCHANT_TEST_ACCOUNT_READY=yes|no
ADMIN_TEST_ACCOUNTS_READY=yes|no
WEBHOOK_TEST_PATH_READY=yes|no
PROXY_RATE_LIMIT_PATH_READY=yes|no
RESTORE_REHEARSAL_ALLOWED=yes|no
```

Do not send or store passwords, tokens, connection strings, or raw secrets in this document.

## Commands Run

- `git ls-files .env admin/.env dashboard/.env` -> `0`
- `git status --short .env admin/.env dashboard/.env` -> `0`
- `git status --ignored --short | rg '(^!!|^\\?\\?) .*staging|(^!!|^\\?\\?) .*\\.env'` -> `0`
- `find . -maxdepth 4 \\( -iname '*staging*' -o -iname '.env*' \\) -printf '%p\\n' | sort` -> `0`
- `mcp__neon__.describe_project(projectId=icy-wind-49683621)` -> `0`
- `mcp__neon__.list_branch_computes(projectId=icy-wind-49683621, branchId=br-plain-pond-ao5e3paw)` -> `0`
- `mcp__neon__.list_branch_computes(projectId=icy-wind-49683621, branchId=br-orange-bar-aoh2a7u0)` -> `0`
- `git branch -a` -> `0`
- `git ls-remote --heads origin` -> `0`
- `rg --files --hidden -g '*deploy*' -g '*vercel*' -g 'wrangler*' -g 'render*' -g 'fly*' -g 'railway*' -g 'docker-compose*' -g 'cloudflare*' -g '.github/workflows/*'` -> `0`
- `rg -n "https://[A-Za-z0-9._-]*feelpay\\.in|api\\.feelpay\\.in|pay\\.feelpay\\.in|staging\\.feelpay\\.in" . --glob '!graphify-out/**' --glob '!target/**' --glob '!node_modules/**'` -> `0`
- `env | cut -d= -f1 | sort | rg '^(APP_ENV|PAYMENT_MODE|DATABASE_URL|NEON_TEST_DATABASE_URL|TEST_DATABASE_URL|MERCHANT_STAGING_URL|ADMIN_STAGING_URL|VITE_API_BASE_URL|VITE_ADMIN_API_BASE_URL)$'` -> `1`
- `env | cut -d= -f1 | sort | rg '^(MERCHANT_STAGING_URL|ADMIN_STAGING_URL|STAGING_DATABASE_BRANCH|MERCHANT_TEST_ACCOUNT_READY|ADMIN_TEST_ACCOUNTS_READY|WEBHOOK_TEST_PATH_READY|PROXY_RATE_LIMIT_PATH_READY|RESTORE_REHEARSAL_ALLOWED)$'` -> `1`
- `curl -fsS 'https://crt.sh/?q=%25.feelpay.in&output=json'` -> `0`
- `curl -fsS 'https://dns.google/resolve?name=feelpay.in&type=NS'` -> `0`
- `curl -fsS 'https://dns.google/resolve?name=staging.feelpay.in&type=A'` -> `0`
- `curl -fsS 'https://dns.google/resolve?name=api.feelpay.in&type=A'` -> `0`
- `curl -fsS 'https://dns.google/resolve?name=dashboard.feelpay.in&type=A'` -> `0`
- `curl -fsS 'https://dns.google/resolve?name=pay.feelpay.in&type=A'` -> `0`
- `rg -n --hidden ... 'staging\\.feelpay\\.in|MERCHANT_STAGING_URL|ADMIN_STAGING_URL|VITE_ADMIN_API_BASE_URL|VITE_API_BASE_URL' /home/chiragskotian/Desktop/payment /home/chiragskotian/Desktop/feel /home/chiragskotian/Desktop/admn` -> `0`
- `find /home/chiragskotian/Desktop/payment -maxdepth 3 ... '*staging*'` -> `0`
- local backend rehearsal start against Neon `test` branch -> `0`
- local admin-server rehearsal start against Neon `test` branch -> `0`
- `curl -fsS ... http://127.0.0.1:18080/health` -> `0`
- `curl -fsS ... http://127.0.0.1:18080/ready` -> `0`
- `curl -fsS ... http://127.0.0.1:18081/health` -> `0`
- `curl -fsS ... http://127.0.0.1:18081/ready` -> `0`
- `curl -fsS -o /tmp/staging_health.out -w '%{http_code}\n' https://staging.feelpay.in/health` -> `6`
- `curl -fsS -o /tmp/staging_health_recheck.out -w '%{http_code}\n' https://staging.feelpay.in/health` -> `6`

## Current Go-Live Blockers

- No verified staging deployment target is available from this workspace.
- Public DNS and certificate records did not expose a usable `feelpay.in` staging target during this run.
- Nearby workspace metadata did not reveal an alternate staging URL or env artifact.
- These findings prove lack of discoverable evidence in the inspected sources, not universal absence of staging infrastructure.
- No staging test credentials are available for auth, admin, integration, or money-flow smoke tests.
- No safe restore-test target has been identified.
- Official Neon restore mechanics are available, but exercising them here would mutate Neon state and no disposable non-production target has been approved.
- No active repo deployment workflow or staging branch is present to infer a deployment target from source control.

## Next Operator Action

Provide the eight non-secret handoff fields in `Inputs Needed To Finish Actual Bring-up`. After those are available, rerun only the currently blocked deployed-environment checks; do not repeat the repo-backed preflight unless the code or deployment metadata changed.

Minimal deployed rerun sequence after handoff:

```bash
curl -fsS "$MERCHANT_STAGING_URL/health"
curl -fsS "$MERCHANT_STAGING_URL/ready"
curl -fsS "$ADMIN_STAGING_URL/health"
curl -fsS "$ADMIN_STAGING_URL/ready"
```

Then perform the manual checklist from `docs/staging-deployment.md` with staging/test accounts only. Do not treat the current local Playwright suites as deployed-staging proof unless they are first reworked to target the supplied staging URLs without starting local services.

Resume checklist after handoff:
1. verify supplied URLs and non-production branch/runtime proof
2. run deployed health/readiness and frontend load checks
3. run auth/isolation checks with staging merchant/admin accounts
4. run Integration Center, merchant lifecycle, and money-safety checks
5. run deployed webhook retry/idempotency and proxy rate-limit checks
6. run restore rehearsal only if an approved disposable non-production target is supplied
7. update this report with actual pass/fail evidence and rerun the completion audit

## Completion Audit

Objective: validate the actually deployed staging environment against `docs/staging-deployment.md`, without touching production data or exposing secrets.

| Prompt Requirement | Concrete Evidence | Status |
| --- | --- | --- |
| inspect `docs/staging-deployment.md` | reviewed and used as the checklist source | pass |
| inspect `docs/production-readiness.md` | reviewed rollback, mode, and safety requirements | pass |
| inspect `.env.example` | reviewed required variable names without values | pass |
| inspect deploy scripts/config if present | hidden-file-aware deploy search found only `docker-compose.local.yml` plus commented workflow files | pass |
| inspect health/readiness endpoints | `src/api/health.rs`, `admin/server/src/main.rs` | pass |
| inspect CORS/origin config | `src/config.rs`, `admin/server/src/main.rs` | pass |
| inspect rate-limit config | `src/middleware/rate_limit.rs` | pass |
| inspect provider webhook config | `admin/server/src/integrations.rs` | pass |
| identify an actual staging deployment target | no verified URL, no active deploy manifest, no local or remote `staging` branch, public DNS/certificate checks did not expose a usable `feelpay.in` target, nearby workspace metadata found no alternate URL, and ignored-file scans found no hidden staging-specific env artifact; this proves no discoverable target from inspected sources, not universal absence | blocked |
| confirm deployed `APP_ENV=staging` | no verified deployed staging runtime target | blocked |
| confirm deployed `PAYMENT_MODE` is safe | no verified deployed staging runtime target | blocked |
| confirm deployed staging Neon branch and exclude production branch | current Neon evidence proves `test` and `production` are separate, but no dedicated `staging` branch exists and deployed runtime wiring is unknown | blocked |
| confirm required deployed env vars without printing values | names documented; deployed env is unreachable | blocked |
| backend deployed health/readiness | local rehearsal passed; no resolvable verified staging URL | blocked |
| admin deployed health/readiness | local rehearsal passed; no verified admin staging URL | blocked |
| frontend apps load | no verified frontend staging URLs | blocked |
| merchant login | no staging merchant credentials | blocked |
| admin login | no staging admin credentials | blocked |
| merchant cannot access admin routes | no staging credentials | blocked |
| admin sensitive actions require permission | no staging admin role credentials | blocked |
| SMTP test path works or safely blocks | no staging admin session/provider config | blocked |
| SMS/Twilio test path works or safely blocks | no staging admin session/provider config | blocked |
| KYC test provider path works or safely blocks | no staging admin session/provider config | blocked |
| deployed secrets remain masked | no deployed admin session | blocked |
| onboarding/KYC submission | no staging sessions | blocked |
| admin review approval | no staging sessions | blocked |
| live API key blocked before approval | no staging sessions | blocked |
| live API key allowed only after approval | no staging sessions | blocked |
| live payout remains blocked without official adapter | repo safety state reviewed; deployed confirmation still needs staging admin session | blocked |
| reconciliation create/detail/close rules | no staging admin session | blocked |
| risk rule/evaluation/case flow | no staging admin session | blocked |
| deployed provider webhook idempotency | code exists; no provider callback path or credentials | blocked |
| retry behavior is safe | no reachable deployed endpoint | blocked |
| sensitive endpoint rate limits behind deployed proxy | code exists; no deployed proxy target | blocked |
| backup/branch/restore process documented | `docs/staging-deployment.md`, `docs/production-readiness.md` | pass |
| restore rehearsal on staging/test branch only if safe | official Neon restore flow is available, but no approved disposable non-production target exists for a safe rehearsal | blocked |
| rerun repo validation if code changes | no runtime code changed in this run; `git diff --check` passed for documentation edit | pass |
| verify that green local automation actually covers deployed staging | admin Playwright auto-starts local services; dashboard Playwright is fixed to localhost, so current suites do not cover deployed staging | pass |
| output pass/fail summary, completed checks, blockers, security, files, commands, remaining go-live blockers | all sections exist in this document | pass |
| actual deployed staging environment validated | required deployed checks above remain blocked | blocked |
| missing external credentials/infrastructure documented | `Checks Blocked`, `Inputs Needed To Finish Actual Bring-up`, `Current Go-Live Blockers` | pass |
| no production secrets exposed | no secret values printed in this report | pass |
| no fake live payout path enabled | repo safety state reviewed; no fake live payout path enabled | pass |
| smoke-test results recorded | this document | pass |

Conclusion: the repository, Neon branch split, and local rehearsal are validated, but the actual deployed staging environment is not yet validated. The goal remains incomplete until the blocked deployed-environment checks above can be exercised.

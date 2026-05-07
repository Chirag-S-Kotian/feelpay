# FeelPay Admin E2E Testing

Playwright covers company-admin smoke, full regression, API contracts, accessibility, and visual snapshots with sandbox data only.

## Start Services

Start the backend from the repo root, then start the admin app:

```bash
cargo run
cd admin && pnpm dev --host 0.0.0.0 --port 3000
```

Set these when using non-default ports:

```bash
export ADMIN_E2E_BASE_URL=http://localhost:3000
export ADMIN_API_BASE_URL=http://127.0.0.1:8081
export JWT_SECRET=feelpay_jwt_secret
```

The E2E auth helper creates local company-admin JWTs for sandbox roles. Do not use live provider credentials, live bank data, real card data, or production secrets.

## Run Tests

From `admin/`:

```bash
pnpm test:e2e:smoke
pnpm test:e2e:full-admin
pnpm test:e2e:api
pnpm test:e2e:a11y
pnpm test:e2e:visual
pnpm test:e2e:headed
pnpm test:e2e:ui
pnpm test:e2e:report
```

From the repo root:

```bash
./scripts/feelpay_admin_e2e_smoke.sh
./scripts/feelpay_admin_e2e_full.sh
VISUAL=true ./scripts/feelpay_admin_e2e_full.sh
UPDATE_SNAPSHOTS=true ./scripts/feelpay_admin_e2e_full.sh
```

## Reports And Traces

HTML report:

```bash
cd admin && pnpm test:e2e:report
```

Traces, screenshots, and videos are retained by Playwright on failure according to `admin/playwright.config.ts`. Open a trace from the report or with:

```bash
cd admin && pnpm exec playwright show-trace path/to/trace.zip
```

## Visual Snapshots

Visual tests live in `admin/tests/e2e/admin-visual.spec.ts`. Dynamic timestamps, UUID-like IDs, metrics, and chart surfaces are masked. Update baselines only after reviewing the diff:

```bash
cd admin && pnpm test:e2e:visual -- --update-snapshots
```

## Common Failures

`Auth failed`: confirm `JWT_SECRET` matches the backend and the role has the expected permissions.

`webhook-events 404`: backend route `/admin/payment-ops/developer/webhook-events` is missing or the admin API base URL points at the wrong service.

`webhook-test 400`: sandbox webhook payload validation failed. Check the response body and backend logs; do not loosen the test to ignore it.

`CORS` or `Failed to fetch`: verify `ADMIN_API_BASE_URL`, backend CORS config, and that the backend is reachable from the browser.

`value prop on input should not be null` or controlled/uncontrolled warnings: fix the React form state to use stable string/boolean defaults.

`PDF generator not configured`: report generation is not production-ready for admin regression. Configure the sandbox PDF generator or keep the failure visible.

`report queued forever`: inspect report worker/job status and export history; new exports should reach a ready/downloadable or clearly failed state.

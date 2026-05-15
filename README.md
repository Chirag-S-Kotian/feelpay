# FeelPay - Indian Payment Gateway

## Quick Start

```bash
# Guided startup with runtime options
./run.sh

# Manual startup
cd dashboard && pnpm install && pnpm dev
cd .. && cargo run
```

## Runtime Modes

- `./run.sh` asks options interactively:
- target (`backend` / `frontend` / `both`)
- payment mode (`test` / `live`)
- backend build (`dev` / `release`)

## Environment

```bash
# .env
DATABASE_URL=postgresql://YOUR_DB_USER:YOUR_DB_PASSWORD@YOUR_DB_HOST/neondb?sslmode=require
TEST_DATABASE_URL=postgresql://YOUR_TEST_DB_USER:YOUR_TEST_DB_PASSWORD@YOUR_TEST_DB_HOST/neondb?sslmode=require
# optional alias
NEON_TEST_DATABASE_URL=postgresql://YOUR_TEST_DB_USER:YOUR_TEST_DB_PASSWORD@YOUR_TEST_DB_HOST/neondb?sslmode=require
JWT_SECRET=your_secure_secret_256_bits_minimum
WEBHOOK_SIGNING_SECRET=whsec_placeholder_example_only
VAULT_ENCRYPTION_KEY=64_hex_chars_required_for_aes256
UPI_PSP_ID=FEELPAY
UPI_MERCHANT_VPA=merchant@feelpay
TENANT_ISOLATION_MODE=shared_table
```

## Tech Stack

- Frontend: React 19, Vite, TypeScript
- Backend: Rust, Actix Web, sqlx
- DB: PostgreSQL (Neon) - Mumbai (`ap-south-1`)

FeelPay is our own branded Indian payment gateway and payment aggregator-ready platform. Merchants use the FeelPay dashboard, APIs, checkout, webhooks, payment links, reports, refunds, and settlements directly. This repo does not target reseller or white-label distribution.

## Commands

```bash
# Dev
pnpm dev           # frontend :3000
cargo run         # backend  :8080

# Build
pnpm build        # frontend
cargo build      # backend
cargo build --release

# Lint
pnpm lint         # tsc --noEmit
cargo check      # rustc
```

## Neon Test Database

- Keep `DATABASE_URL` for normal development only.
- Create a separate Neon dev/test branch and set `TEST_DATABASE_URL` or `NEON_TEST_DATABASE_URL` to that branch URL. Include `sslmode=require` when Neon requires it.
- Never use a Neon production database for tests or migration validation, and never commit credentials.
- Run online validation with `./scripts/validate_test_db.sh`; it prefers `TEST_DATABASE_URL`, then `NEON_TEST_DATABASE_URL`, refuses URLs that look production-like or equal `DATABASE_URL`, runs migrations, then runs `cargo test`.
- Use `SQLX_OFFLINE=true cargo test` only when no reachable test database exists.

## API Smoke Routes

```bash
# health
curl http://127.0.0.1:8080/v1/health

# auth login (dashboard JWT)
curl -X POST http://127.0.0.1:8080/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@shop.com","password":"secure123"}'

# settlements (API key auth)
curl http://127.0.0.1:8080/v1/settlements \
  -H "x-api-key: <api_key>"

# onboarding status (JWT auth)
curl http://127.0.0.1:8080/v1/onboarding/status \
  -H "Authorization: Bearer <jwt>"
```

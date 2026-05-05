# FeelPay - Payment Gateway

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

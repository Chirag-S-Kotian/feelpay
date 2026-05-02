# FeelPay — repo agent config

## Stack
- Backend: Rust + Actix Web + sqlx + PostgreSQL
- Frontend: React 19 + Vite + TypeScript (`dashboard/`)
- Migrations: `migrations/` — numbered SQL files, run in order
- Package manager: pnpm (frontend), cargo (backend)

## Repo layout
```
src/
  api/              # Actix route handlers (payments, disputes, analytics, settlements, onboarding)
  engine/           # Payment pipeline + processors (card, upi, simulator)
  engine/services/  # ledger.rs, settlement.rs
  vault/            # Tokenization, AES-GCM crypto, BIN lookup
  config.rs         # Vault / UPI / rate-limit / tenant config
migrations/         # SQL schema — never edit existing files, always add new ones
dashboard/src/
  pages/            # React pages: Dashboard, Payments, Settlements, Analytics, Disputes, Settings
  hooks/            # API hooks + protected routing
```

## Environment (required)
Copy `.env.example` → `.env` and populate:
- `DATABASE_URL` — PostgreSQL connection string
- `JWT_SECRET` — strong random secret
- `VAULT_ENCRYPTION_KEY` — 32-byte key for AES-GCM vault

## Verification commands
Run in this order before marking any task complete:

```bash
# Backend
cargo check
cargo test
sqlx migrate run

# Frontend
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```

All commands must exit 0. A task is NOT done until they do.

## SQL rules
- Schema changes go in a new numbered migration file only.
- Never modify an existing migration.

## Known fragile areas
- `src/api/analytics.rs` — Decimal handling is sensitive; run cargo test after any edit.
- `dashboard/src/pages/Analytics.tsx` — some chart data is still mocked; do not add more mocks.
- UPI processor (`src/engine/processors/upi.rs`) — stub only, no NPCI production connector.
- T+2 settlement batch — no scheduler exists yet; do not assume one.
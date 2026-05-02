# FeelPay — AGENTS.md

## Codebase context
- Before exploring unfamiliar modules, read `/home/chiragskotian/Desktop/payment/graphify-out/GRAPH_REPORT.md`.
- It contains the full dependency graph (215 nodes, 232 edges) — use it to understand call paths and cluster ownership before making changes.
- Core hubs: `PaymentEngine`, `CardProcessor`, `UpiProcessor`, `SimulatorProcessor` — high-blast-radius; changes here affect the entire pipeline.

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

## Environment
Copy `.env.example` → `.env` and set:
- `DATABASE_URL` — PostgreSQL connection string
- `JWT_SECRET` — strong random secret
- `VAULT_ENCRYPTION_KEY` — 32-byte key for AES-GCM vault

## Verification commands
Run in this order before marking any task complete:
```bash
cargo check
cargo test
sqlx migrate run
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```
All must exit 0. Task is NOT done until they do.

## Coding rules
- Rust: prefer `?` over `unwrap()`. Never unwrap on DB results or Decimal types.
- TypeScript: strict mode on. Fix type errors — never use `any` or `@ts-ignore`.
- React: functional components only.
- SQL: new numbered migration file for every schema change. Never edit existing migrations.
- Match style of the file being edited. Do not reformat unrelated code.
- Touch only files required by the task. If fix needs 3+ files, confirm scope first.
- Every changed line must trace to the task. No speculative abstractions.

## PR format
```
## What
[one sentence: what changed and why]
## How
[2–4 bullets: key decisions]
## Tested
[commands run and exit status]
```

## Known fragile areas
- `src/api/analytics.rs` — Decimal handling sensitive; run cargo test after any edit.
- `dashboard/src/pages/Analytics.tsx` — some charts still mocked; do not add more mocks.
- `src/engine/processors/upi.rs` — stub only, no NPCI production connector.
- T+2 settlement batch — no scheduler exists; do not assume one.
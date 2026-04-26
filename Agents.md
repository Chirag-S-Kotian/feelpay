# FeelPay Agent Context

## Current Stack (Source of Truth)
- Backend: Rust + Actix Web + sqlx + PostgreSQL
- Frontend: React 19 + Vite + TypeScript
- DB: PostgreSQL (Docker or Neon), migrations in `migrations/`

## Graphify Snapshot (2026-04-26)
- Graph size: 215 nodes, 232 edges, 41 communities
- Core hubs: `PaymentEngine`, `CardProcessor`, `UpiProcessor`, `SimulatorProcessor`, `main()`
- Strong clusters:
  - Payment processing and pipeline
  - API routes (payments/disputes/analytics/settlements/onboarding)
  - Vault/tokenization
- Weak spots from graph report:
  - Many isolated nodes (docs/readability gap)
  - Several tiny communities in dashboard pages

## Plan Tracking: implementation_plan.md.resolved

### Phase 1 — Gateway Core Engine
Status: Mostly complete
- Done:
  - `src/engine/payment.rs`
  - `src/engine/processor.rs`
  - `src/engine/pipeline.rs`
  - `src/engine/processors/{simulator,card,upi}.rs`
  - `src/config.rs` upgraded for vault/UPI/rate limit/tenant mode
  - `src/main.rs` uses native processors
- Pending / partial:
  - No dedicated `netbanking.rs` processor
  - No dedicated `wallet.rs` processor
  - Startup deep health validation + graceful shutdown flow can be improved

### Phase 2 — Vault & Tokenization
Status: Complete (MVP)
- Done:
  - `migrations/20240102000001_vault_schema.sql`
  - `src/vault/{service,crypto,bin_lookup}.rs`
  - Encryption dependencies in `Cargo.toml` (`aes-gcm`, `zeroize`)

### Phase 3 — UPI Engine
Status: Complete (MVP)
- Done:
  - `src/engine/processors/upi.rs`
  - `migrations/20240103000001_upi_schema.sql` (added)
- Pending / partial:
  - NPCI/PSP production connector integration (currently processor-level implementation)

### Phase 4 — Ledger & Settlements
Status: Mostly complete
- Done:
  - `migrations/20240104000001_ledger_accounts.sql`
  - `migrations/20240104000002_settlements.sql`
  - `src/engine/services/{ledger,settlement}.rs`
  - `src/api/settlements.rs`
- Pending / partial:
  - Automated scheduled T+2 batch orchestration can be hardened

### Phase 5 — Onboarding / KYC
Status: Mostly complete
- Done:
  - `migrations/20240105000001_kyc_schema.sql`
  - `src/api/onboarding.rs`
  - merchant onboarding fields present
- Pending / partial:
  - External verification adapters (PAN/GST/Bank) can be extended

### Phase 6 — Dashboard Expansion
Status: Implemented (UI pages present)
- Done:
  - Pages exist: `Dashboard`, `Payments`, `Settlements`, `Analytics`, `Developers`, `Disputes`, `Settings`, `Onboarding`, `Login`
  - Protected routing + API hook exists
- Pending / partial:
  - `Analytics.tsx` still uses simulated chart data for some sections
  - Developer docs page can be expanded with live schema samples

## Recent Fixes (Current Session)
- Fixed analytics summary decimal handling in `src/api/analytics.rs`
  - Removed `unwrap_or` call on decimal success-rate field
- Added missing route in `src/main.rs`
  - `GET /v1/analytics/top-customers`
- Added migration file
  - `migrations/20240103000001_upi_schema.sql`

## Mandatory Quality Gates
Run before commit:

```bash
# Backend
cargo check
cargo test
sqlx migrate run

# Frontend
cd dashboard
pnpm build
pnpm tsc --noEmit
```

## Error Policy
- TypeScript error: block
- Rust compile error: block
- Test failure: block
- Lint warning: fix or justify in PR

## Environment Requirements
- `DATABASE_URL` (required)
- `JWT_SECRET` (required, strong secret)
- `VAULT_ENCRYPTION_KEY` (required for production-safe vault)

## Next Recommended Work (High Impact)
1. Wire dashboard analytics to live backend endpoints (remove simulated datasets).
2. Add netbanking/wallet processors or remove from roadmap.
3. Add startup preflight checks + graceful shutdown draining.
4. Add integration tests for vault + UPI + settlement happy-paths.

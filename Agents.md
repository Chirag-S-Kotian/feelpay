# FeelPay Agents

## Code Quality Gates

### Pre-Commit Checks (ALL must pass)

```bash
# Frontend
pnpm build          # Vite build
pnpm tsc --noEmit   # TypeScript type-check

# Backend
cargo check        # Rust compile
cargo test         # Unit tests
sqlx migrate run  # DB migrations
```

### Error Threshold

| Type | Action |
|------|--------|
| TS error | Block commit |
| Rust error | Block commit |
| Test fail | Block commit |
| Lint warn | Fix or justify |

## Deployment

```bash
# Production
DATABASE_URL=postgres://...neon.tech/feelpay?sslmode=require pnpm build && cargo build --release
```

## Quick Ref

| Command | Frontend | Backend |
|---------|----------|---------|
| Dev | `pnpm dev` | `cargo run` |
| Build | `pnpm build` | `cargo build` |
| Check | `pnpm tsc` | `cargo check` |
| Test | `pnpm test` | `cargo test` |
| DB | - | `sqlx migrate run` |

## Environment

- `DATABASE_URL` — Neon Postgres (required)
- `JWT_SECRET` — 256-bit min (required)

## Architecture

- Frontend: React 19 + Vite + TS
- Backend: Rust + Axum + sqlx
- DB: Neon Postgres (serverless)
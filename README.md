# FeelPay - Payment Gateway

## Quick Start (No Docker)

```bash
# Guided startup with runtime options
./run.sh

# Manual startup
cd dashboard && pnpm install && pnpm dev
cd .. && cargo run
```

## Runtime Modes

- `./run.sh` asks all options interactively:
- target (`backend` / `frontend` / `both`)
- payment mode (`test` / `live`)
- DB provider (`neon` / `docker` / `auto`)
- backend build (`dev` / `release`)
- If `neon` selected, `DATABASE_URL_NEON` must be set.
- If `docker` selected, `DATABASE_URL_DOCKER` must be set.

## Environment

```bash
# .env - choose provider: auto | neon | docker
DATABASE_PROVIDER=auto
DATABASE_URL=postgresql://user:password@localhost:5432/feelpay
DATABASE_URL_NEON=postgresql://user:password@ep-xxx.us-east-1.aws.neon.tech/neondb?sslmode=require
DATABASE_URL_DOCKER=postgresql://feelpay:feelpay123@localhost:5432/feelpay
JWT_SECRET=your_secure_secret_256_bits_minimum
WEBHOOK_SIGNING_SECRET=replace_with_real_secret
```

## Tech Stack

- Frontend: React 19, Vite, TypeScript
- Backend: Rust, Actix Web, sqlx
- DB: PostgreSQL (Neon or Docker/local)

## Commands

```bash
# Dev
pnpm dev           # frontend :3000
cargo run         # backend  :8080

# Build
pnpm build        # frontend
cargo build      # backend

# Lint
pnpm lint         # tsc --noEmit
cargo check      # rustc

# Docker cleanup
./scripts/docker_cleanup.sh         # cleanup project-only feelpay/payment artifacts
./scripts/docker_cleanup.sh --all   # global docker prune (dangerous)
```

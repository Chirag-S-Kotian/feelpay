# FeelPay - Payment Gateway

## Quick Start (No Docker)

```bash
# Guided startup with runtime options
./run.sh

# Manual startup
cd dashboard && pnpm install && pnpm dev
cd .. && cargo run
```

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
pnpm dev           # frontend :5173
cargo run         # backend  :8080

# Build
pnpm build        # frontend
cargo build      # backend

# Lint
pnpm lint         # tsc --noEmit
cargo check      # rustc
```

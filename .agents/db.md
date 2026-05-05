# Database Context (PostgreSQL + sqlx + Migrations)

Load when touching: `migrations/`, `src/db/`, `.sqlx/`, or any SQL query.

## Layout

```text
migrations/              57 numbered migration files (YYYYMMDD format)
migrations_legacy/       archived migrations — do NOT touch
src/db/                  database connection setup
.sqlx/                   offline query cache (auto-generated)
```

## Rules

- **NEVER edit existing migrations.** Always create a new file.
- Naming: `YYYYMMDDHHMMSS_description.sql` — use next available date.
- Latest migration: `20260505000005_api_readiness_scorecard_gate.sql`.
- Run `cargo sqlx prepare` after adding/changing any sqlx query.
- Use `sqlx::query!` or `sqlx::query_as!` macros for compile-time checking.
- Transactions: use `pool.begin()` + `tx.commit()` for multi-statement ops.

## Key Tables

payments, merchants, users, api_keys, settlements, ledger_entries, disputes, refunds, payouts, customers, payment_links, subscriptions, webhooks, fee_plans, providers, documents, notifications, mandates, virtual_accounts, bank_accounts, audit_logs.

## Schema Patterns

- All tables have `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
- Timestamps: `created_at TIMESTAMPTZ DEFAULT NOW()`, `updated_at TIMESTAMPTZ`.
- Soft deletes where applicable: `deleted_at TIMESTAMPTZ`.
- RLS policies exist (migration `20240110000002`); respect tenant isolation.

## Testing

```bash
sqlx migrate run         # apply pending migrations
cargo sqlx prepare       # refresh offline cache after query changes
```

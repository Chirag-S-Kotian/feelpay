# FeelPay Agent Guide

Evidence first. Small context. Small diff.

## Context Loading

Load `.agents/` modules **only** when touching their domain:
- `backend.md` → `src/api/`, `src/middleware/`, `src/models/`, `src/*.rs`
- `frontend.md` → `dashboard/`, `admin/`
- `payments.md` → `src/engine/`, `src/vault/`
- `db.md` → `migrations/`, `src/db/`, `.sqlx/`, SQL queries

Unknown module? Read `graphify-out/GRAPH_REPORT.md` first.

## Workflow

1. Restate task (1 line). Load relevant `.agents/` context.
2. Read only needed files via `rg` / targeted slices. No bulk context.
3. Plan for risky/multi-file work; otherwise patch directly.
4. After code changes: `./scripts/graphify update .`
5. Run verification gate. Report files + commands + exit status.

## Verification Gate

```bash
./scripts/graphify update .
cargo check
sqlx migrate run
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```

Skip `cargo test` unless explicitly requested; it can hang/crash this workstation.

All exit 0. If blocked, state exact blocker + exit code.

## Output Rule

Final response ≤5 lines: what changed, files, commands + exit codes.

## PR Format

```
## What — [one sentence]
## How — [2-4 bullets]
## Tested — [command] - [exit status]
```

## Compact Summary

Preserve: task, changed files, decisions, commands/exit codes, blockers, next step.

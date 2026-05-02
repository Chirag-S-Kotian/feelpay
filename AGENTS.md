## Workflow Orchestration

### 1. Plan Node Default
- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately - don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One tack per subagent for focused execution

### 3. Self-Improvement Loop
- After ANY correction from the user: update `tasks/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

### 4. Verification Before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes - don't over-engineer
- Challenge your own work before presenting it

### 6. Autonomous Bug Fixing
- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task Management

1. **Plan First**: Write plan to `tasks/todo.md` with checkable items
2. **Verify Plan**: Check in before starting implementation
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add review section to `tasks/todo.md`
6. **Capture Lessons**: Update `tasks/lessons.md` after corrections

## Core Principles

- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimat Impact**: Changes should only touch what's necessary. Avoid introducing bugs.


# FeelPay — repo agent config

## Codebase context
- Before exploring unfamiliar modules, read `/home/chiragskotian/Desktop/payment/graphify-out/GRAPH_REPORT.md`.
- It contains the full dependency graph (215 nodes, 232 edges) — use it to understand call paths and cluster ownership before making changes.
- Core hubs are `PaymentEngine`, `CardProcessor`, `UpiProcessor`, `SimulatorProcessor` — treat them as high-blast-radius; changes here affect the entire pipeline.

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
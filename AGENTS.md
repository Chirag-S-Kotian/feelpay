# FeelPay Agent Guide

Evidence first. Small context. Small diff.

## Context
- Unknown module? Read `graphify-out/GRAPH_REPORT.md` first; use inferred edges as hints only.
- High-risk hubs: `PaymentEngine`, processors, auth, analytics, ledger, settlement, migrations.
- Hub edit? Run `git status --short`, trace callers/tests with `rg`, inspect touched path.

## Stack / Map
- Backend: Rust + Actix Web + sqlx + PostgreSQL.
- Frontend: React 19 + Vite + TypeScript; use pnpm in `dashboard/`.
- DB: numbered migrations in `migrations/`; never edit old migrations.

```text
src/api/              routes
src/engine/           payment pipeline + processors
src/engine/services/  ledger, settlement
src/vault/            tokenization, AES-GCM, BIN lookup
dashboard/src/        React pages/hooks
scripts/              utilities; inspect before replacing
```

## Workflow
1. One-line task restatement.
2. Read graph/files/tests needed for change, no bulk context.
3. Plan only for risky or multi-file work.
4. Patch smallest surface.
5. Run narrow check, then full gate.
6. Report files + commands + exit status.

Skills: use clean-code + karpathy-guidelines for code write/review/refactor. Superpowers: writing/executing plans for multi-step work; systematic-debugging for bugs; verification-before-completion before success claims. Parallel agents only when user asks.

## Agent Files
- `AGENTS.md`: canonical repo rules for Codex/agents.
- `CLAUDE.md`: tiny Claude shim; points here, adds compact-summary rule.
- `.codex/AGENTS.md`: Codex behavior overlay; repo facts stay here.
- `.agents/skills/karpathy-guidelines/SKILL.md`: shared Karpathy skill; `.gitignore` keeps only this skill trackable.

## Change Rules
- Touch only task files. If fix needs more than 3 files, confirm scope first.
- Every changed line must trace to task. No speculative abstractions.
- Match local style; no unrelated formatting.
- Do not revert user changes. Check `git status --short` before and after edits.
- Clean code: intention names, small single-purpose functions, near-use variables, useful comments only, behavior tests.
- Karpathy: state assumptions, prefer simplest viable path, make surgical edits, prove success with checks.
- Rust: prefer `?`; never `unwrap()` DB results/Decimal.
- TypeScript: strict; no `any`, no `@ts-ignore`.
- React: functional components only. SQL: add new migration for schema change.

## Verification Gate
Run before marking complete:
```bash
cargo check
cargo test
sqlx migrate run
cd dashboard && pnpm build && pnpm exec tsc --noEmit
```
All must exit 0. If blocked, state exact blocker + exit code.

## Fragile Areas
- `src/api/analytics.rs`: Decimal-sensitive.
- `dashboard/src/pages/Analytics.tsx`: do not add mocks.
- `src/engine/processors/upi.rs`: stub only; no NPCI production connector.
- T+2 settlement batch: no scheduler exists; do not assume one.

## PR Format
```markdown
## What
[one sentence: what changed and why]

## How
[2-4 bullets]

## Tested
- `[command]` - [exit status]
```

## Compact Summary
If context compacts, preserve: task, changed files, decisions, commands/exit codes, blockers, next step.

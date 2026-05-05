# FeelPay Claude Guide

Read `AGENTS.md` first — it is the repo source of truth. Load `.agents/*.md` for domain context.

## Must Follow
- Load `.agents/backend.md`, `frontend.md`, `payments.md`, or `db.md` based on task scope.
- Use `clean-code` + `karpathy-guidelines` skills for code write/review/refactor.
- Smallest useful context: `rg`, targeted file slices, graph report only for unknown modules.
- State assumptions, choose simplest viable path, make surgical edits, verify with commands.
- After code changes: `./scripts/graphify update .`

## Before Compacting
Preserve: modified files, key decisions, commands run, exit codes, blockers, next step.

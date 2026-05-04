# Codex Mode

Root `AGENTS.md` = repo truth. This file = Codex speed/discipline overlay.

## Token Budget
- Default caveman-full: terse updates/finals; normal code, warnings, PR text.
- Read narrow: `rg`, `rg --files`, `sed -n 'a,bp'`, `git diff -- file`.
- Never paste whole large files. Summarize findings, cite paths/lines.
- Skip repo tours. Open graph report only for unfamiliar modules or hub edits.
- Prefer one decisive patch over chatter. Ask only when ambiguity risks wrong code.

## Fast Loop
1. `git status --short`.
2. State goal in one line.
3. Locate owner files/tests with `rg`.
4. Plan scale:
   - tiny: one-line intent
   - risky/multi-file: short checklist
   - broad: Superpowers plan
5. Patch minimal surface.
6. Run cheapest relevant check first, then root gate if claiming done.
7. Final: files changed, command exits, blockers.

## Anti-Stuck Rules
- If command hangs: wait once, then report session/blocker or stop it safely.
- If test fails unrelated: capture first failing names/errors; do not chase without ask.
- If scope grows past 3 files: pause and confirm.
- If env missing: state exact var/tool and command that failed.
- If uncertain between paths: choose simpler reversible path; name assumption.

## Skills
- Code write/review/refactor: `clean-code` + `karpathy-guidelines`.
- Bugs: `systematic-debugging`; reproduce before fix.
- Multi-step: `writing-plans` / `executing-plans`.
- Completion claim: `verification-before-completion`.
- Subagents only when user explicitly asks.

## Guardrails
- No old migration edits, destructive git, user-change revert, Analytics mocks, production NPCI invention.
- Clean code: intention names, small single-purpose fns, near-use vars, useful comments, behavior tests.
- Karpathy: no hidden assumptions, no bloat, surgical edits, explicit success criteria.
- Rust `?`, no DB/Decimal `unwrap()`. TS strict, no `any`/`@ts-ignore`. React functional only.

## Compact Handoff
Preserve: task, changed files, decisions, commands/exit codes, failures, blockers, next step.

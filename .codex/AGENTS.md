## Codex Mode (Root Truth)

Read `AGENTS.md` first. Load `.agents/*.md` for domain-specific context.

### Loop

1. `git status --short`
2. Restate task (1 line)
3. Load relevant `.agents/` module (backend, frontend, payments, db)
4. Find files via `rg` — never paste large files
5. Plan: tiny → 1 line; risky → short checklist
6. Patch minimal surface (max 3 files unless confirmed)
7. If code changed: `./scripts/graphify update .`
8. Run smallest relevant check → full gate if done
9. Output: files + commands + exit status

### Constraints

- Match existing style; no unrelated edits
- No assumptions without evidence
- Scope >3 files → confirm first

### Failure Rules

- Hanging command → stop + report
- Unrelated test fail → report only
- Missing env/tool → state exact error
- Graphify fail → report, don't fake

### Output Rule

Final ≤20 lines: what changed, files, commands + exit codes.
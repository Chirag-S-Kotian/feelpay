# Global agent defaults

## Working agreements
- Use pnpm for JS/TS dependency installs, never npm or yarn.
- Prefer `?` over `unwrap()` in Rust. Never unwrap on DB results or Decimal types.
- TypeScript strict mode is always on. Fix type errors — never suppress with `any` or `@ts-ignore`.
- Functional React components only. No class components.
- Match the style of the file being edited. Do not reformat unrelated code.
- Touch only files required by the task. Flag scope creep before acting on it.
- Do not delete pre-existing dead code. Note it in the PR instead.

## Change discipline
- Every changed line must trace directly to the task description.
- If a fix requires touching more than 3 files, pause and confirm scope with the user.
- No speculative abstractions, no unrequested flexibility or config options.

## PR message format
```
## What
[One sentence: what changed and why]

## How
[2–4 bullet points: key implementation decisions]

## Tested
[Commands run and their exit status]
```

## Done when
- All verification commands listed in the repo AGENTS.md exit 0.
- The specific behavior described in the task is demonstrably working.
- No regressions in adjacent files or routes.
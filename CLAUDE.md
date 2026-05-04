# FeelPay Claude Guide

Read `AGENTS.md` first; it is repo source of truth. Keep this file tiny so Claude loads it fast.

## Must Follow
- Use `clean-code` + `karpathy-guidelines` when writing, reviewing, or refactoring code.
- Use smallest useful context: `rg`, targeted file slices, graph report only for unknown modules.
- State assumptions, choose simplest viable path, make surgical edits, verify with commands.
- Do not edit old migrations, revert user changes, add Analytics mocks, or invent production NPCI.

## Before Compacting
Preserve modified files, key decisions, commands run, exit codes, blockers, and next step.

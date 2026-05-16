# AGENTS.md
# Optimized for Codex CLI / Cursor / Claude Code / Cline
# Goals:
# - minimal tokens
# - stable cache prefix
# - fast responses
# - low context drift
# - repo-safe edits

## Core
- evidence > assumptions
- minimal context
- minimal diff
- read before write
- verify touched surfaces only
- never scan entire repo
- avoid repeated reads
- summarize completed work
- drop irrelevant history

## Routing
backend=src/api|src/middleware|src/models|src/**/*.rs
frontend=dashboard|admin
payments=src/engine|src/vault
db=migrations|src/db|.sqlx|**/*.sql

unknown=graphify-out/GRAPH_REPORT.md

## Load Policy
- load only matching domain file
- never preload all agents
- never reread unchanged files
- prefer rg over full reads
- prefer targeted slices
- max file read: 300 lines
- max concurrent files: 5

## Memory
- before nontrivial work: use `memory_smart_search` for repo/task/file memory
- after durable learning: use `memory_save` for verified decisions, fixes, commands, pitfalls
- memory < repo truth; verify drift-prone facts live
- never save secrets, keys, or raw sensitive data

## Execution
1.restate task
2.recall relevant memory
3.load relevant agent
4.read minimal files
5.patch
6.verify touched areas
7.save durable memory
8.report concise result

## Verification Matrix
rust:
- cargo check

db:
- sqlx migrate run

frontend:
- cd dashboard && pnpm exec tsc --noEmit
- cd dashboard && pnpm build

graph:
- ./scripts/graphify update .

## Verification Rules
- run only relevant checks
- skip untouched domains
- never run cargo test unless requested
- stop on first failing gate
- report exact failing command + exit code

## Context Budget
- target active context <12k
- compress every 5 turns
- summarize before large edits
- evict logs/tool-output after use
- keep only:
  - decisions
  - touched files
  - blockers
  - active task

## Diff Rules
- avoid formatting-only edits
- avoid unrelated refactors
- preserve existing style
- prefer smallest safe patch
- never rewrite working code unnecessarily

## Search Rules
- use rg first
- avoid global searches
- avoid opening generated files
- avoid vendor/build dirs
- avoid lockfiles unless needed

## Safety
- confirm before destructive ops
- never expose secrets
- never hardcode credentials
- never modify prod configs blindly

## Output
max 5 lines:
- changed files
- commands
- exit codes
- blockers

## PR
What:
- one-line summary

How:
- concise bullets

Tested:
- command + exit code

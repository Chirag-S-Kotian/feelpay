#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")"
SHORT_SHA="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"

FAIL_CMD=""
FAIL_HINT=""

run_step() {
  local cmd="$1"
  local hint="$2"
  echo "[RUN] $cmd"
  if ! bash -lc "$cmd"; then
    FAIL_CMD="$cmd"
    FAIL_HINT="$hint"
    return 1
  fi
}

scan_forbidden_wording() {
  echo "[RUN] forbidden visible wording scan"
  local found=0
  local patterns=(
    "Merchant Admin"
    "Seller Admin"
    "Vendor Admin"
    "merchant admin"
    "seller admin"
    "vendor admin"
    "Active Merchants"
    "Recent Merchants"
    "Merchant Filter"
  )
  for pattern in "${patterns[@]}"; do
    if rg -n --glob '!target/**' --glob '!node_modules/**' --glob '!dist/**' --glob '!graphify-out/**' "$pattern" admin/src dashboard/src src admin/server/src; then
      found=1
    fi
  done
  if [[ "$found" -ne 0 ]]; then
    FAIL_CMD="forbidden visible wording scan"
    FAIL_HINT="Replace visible wording with Business Account naming. Legacy internal names allowed only non-UI."
    return 1
  fi
}

scan_sensitive_leakage() {
  echo "[RUN] sensitive leakage scan"
  local found=0
  local patterns=(
    "api_secret="
    "webhook_secret="
    "full_account_number"
    "raw_card_number"
    "Authorization:"
  )
  for pattern in "${patterns[@]}"; do
    if rg -n --glob '!target/**' --glob '!node_modules/**' --glob '!dist/**' --glob '!graphify-out/**' "$pattern" admin/src dashboard/src src admin/server/src; then
      found=1
    fi
  done
  if rg -n --glob '!target/**' --glob '!node_modules/**' --glob '!dist/**' --glob '!graphify-out/**' "\\bcvv\\b|\\bcvc\\b|card_number" admin/src dashboard/src src admin/server/src | rg -v "no CVV|No CVV|no raw card|No raw card|redacted|masked|test|assert"; then
    found=1
  fi
  if [[ "$found" -ne 0 ]]; then
    FAIL_CMD="sensitive leakage scan"
    FAIL_HINT="Remove secret/card/bank exposure from logs/UI/API responses. Keep masked/redacted only."
    return 1
  fi
}

main() {
  echo "FeelPay Admin Quality Gate"
  echo "Timestamp: $TS"
  echo "Branch: $BRANCH"
  echo "Commit: $SHORT_SHA"

  run_step "./scripts/graphify update ." "Fix graphify script/runtime issue."
  run_step "cargo fmt --check" "Run cargo fmt and commit formatting."
  run_step "cargo check" "Fix Rust compile errors."
  run_step "cargo test" "Fix failing Rust tests."
  run_step "sqlx migrate run" "Ensure local DB reachable and migrations valid."
  run_step "cd admin/server && cargo fmt --check" "Run admin/server rustfmt."
  run_step "cd admin/server && cargo check" "Fix admin/server compile errors."
  run_step "cd admin/server && cargo test" "Fix admin/server tests."
  run_step "cd admin && pnpm exec tsc --noEmit" "Fix admin TypeScript errors."
  run_step "cd admin && pnpm exec vitest run" "Fix admin tests."
  run_step "cd admin && pnpm build" "Fix admin build issues."
  run_step "cd dashboard && pnpm exec tsc --noEmit" "Fix dashboard TypeScript errors."
  run_step "cd dashboard && pnpm build" "Fix dashboard build issues."
  run_step "git diff --check" "Fix trailing whitespace/conflict markers."
  scan_forbidden_wording
  scan_sensitive_leakage

  echo "PASS: quality gate complete"
  echo "Timestamp: $TS"
  echo "Branch: $BRANCH"
  echo "Commit: $SHORT_SHA"
}

if ! main; then
  echo "FAIL: quality gate"
  echo "Failed step: $FAIL_CMD"
  echo "Hint: $FAIL_HINT"
  echo "Timestamp: $TS"
  echo "Branch: $BRANCH"
  echo "Commit: $SHORT_SHA"
  echo "Note: warning from graphify NPCI stub may appear; treat as expected warning if exit code 0."
  exit 1
fi

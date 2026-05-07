#!/usr/bin/env bash
set -u

ADMIN_E2E_BASE_URL="${ADMIN_E2E_BASE_URL:-http://localhost:3000}"
ADMIN_API_BASE_URL="${ADMIN_API_BASE_URL:-http://127.0.0.1:8081}"

failures=0

check_url() {
  local label="$1"
  local url="$2"
  if curl -fsS "$url" >/dev/null; then
    printf '[ok] %s reachable: %s\n' "$label" "$url"
  else
    printf '[fail] %s not reachable: %s\n' "$label" "$url"
    failures=$((failures + 1))
  fi
}

check_url "backend" "$ADMIN_API_BASE_URL/health"
check_url "admin frontend" "$ADMIN_E2E_BASE_URL"

if [ "$failures" -ne 0 ]; then
  printf 'Smoke preflight failed (%s checks failed).\n' "$failures"
  exit 1
fi

(cd admin && pnpm test:e2e:smoke)
status=$?
printf 'Playwright report: admin/playwright-report/index.html\n'
exit "$status"

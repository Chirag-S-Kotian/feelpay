#!/usr/bin/env bash
set -u

ADMIN_E2E_BASE_URL="${ADMIN_E2E_BASE_URL:-http://127.0.0.1:3000}"
ADMIN_API_BASE_URL="${ADMIN_API_BASE_URL:-http://127.0.0.1:8081}"
export ADMIN_E2E_BASE_URL ADMIN_API_BASE_URL
run_status=0

check_url() {
  local label="$1"
  local url="$2"
  printf '[check] %s: %s\n' "$label" "$url"
  if curl -fsS "$url" >/dev/null; then
    printf '[ok] %s reachable: %s\n' "$label" "$url"
  else
    printf '[info] %s is not currently reachable; Playwright webServer will try to start it.\n' "$label"
  fi
}

run_suite() {
  local name="$1"
  local command="$2"
  printf '\n== %s ==\n' "$name"
  if (cd admin && eval "$command"); then
    printf '[pass] %s\n' "$name"
  else
    local status=$?
    printf '[fail] %s exit=%s\n' "$name" "$status"
    run_status=1
  fi
}

check_url "backend" "$ADMIN_API_BASE_URL/health"
check_url "admin frontend" "$ADMIN_E2E_BASE_URL"
printf '[info] Manual fallback commands if startup still fails:\n'
printf '  HOST=127.0.0.1 PORT=8081 CORS_ALLOWED_ORIGINS=http://127.0.0.1:3000,http://localhost:3000 cargo run\n'
printf '  cd admin && VITE_ADMIN_API_BASE_URL=%s pnpm dev --host 127.0.0.1 --port 3000\n' "$ADMIN_API_BASE_URL"

if [ "$run_status" -eq 0 ]; then
  run_suite "smoke" "pnpm test:e2e:smoke"
  run_suite "full-admin" "pnpm test:e2e:full-admin"
  run_suite "api" "pnpm test:e2e:api"
  run_suite "a11y" "pnpm test:e2e:a11y"
  if [ "${VISUAL:-false}" = "true" ] || [ "${UPDATE_SNAPSHOTS:-false}" = "true" ]; then
    run_suite "visual" "pnpm test:e2e:visual ${UPDATE_SNAPSHOTS:+--update-snapshots}"
  else
    printf '\n[skip] visual (set VISUAL=true or UPDATE_SNAPSHOTS=true)\n'
  fi
fi

printf '\nPlaywright report: admin/playwright-report/index.html\n'
exit "$run_status"

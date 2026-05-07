#!/usr/bin/env bash
set -u

ADMIN_E2E_BASE_URL="${ADMIN_E2E_BASE_URL:-http://127.0.0.1:3000}"
ADMIN_API_BASE_URL="${ADMIN_API_BASE_URL:-http://127.0.0.1:8081}"
export ADMIN_E2E_BASE_URL ADMIN_API_BASE_URL

missing=0

check_url() {
  local label="$1"
  local url="$2"
  printf '[check] %s: %s\n' "$label" "$url"
  if curl -fsS "$url" >/dev/null; then
    printf '[ok] %s reachable: %s\n' "$label" "$url"
  else
    printf '[info] %s is not currently reachable; Playwright webServer will try to start it.\n' "$label"
    missing=$((missing + 1))
  fi
}

check_url "backend" "$ADMIN_API_BASE_URL/health"
check_url "admin frontend" "$ADMIN_E2E_BASE_URL"

if [ "$missing" -ne 0 ]; then
  printf '[info] Manual fallback commands if startup still fails:\n'
  printf '  HOST=127.0.0.1 PORT=8081 CORS_ALLOWED_ORIGINS=http://127.0.0.1:3000,http://localhost:3000 cargo run\n'
  printf '  cd admin && VITE_ADMIN_API_BASE_URL=%s pnpm dev --host 127.0.0.1 --port 3000\n' "$ADMIN_API_BASE_URL"
fi

(cd admin && ADMIN_E2E_BASE_URL="$ADMIN_E2E_BASE_URL" ADMIN_API_BASE_URL="$ADMIN_API_BASE_URL" pnpm test:e2e:smoke)
status=$?
if [ "$status" -ne 0 ]; then
  printf '[fail] Smoke E2E failed after Playwright startup. Re-check URLs above and open the report.\n'
fi
printf 'Playwright report: admin/playwright-report/index.html\n'
exit "$status"

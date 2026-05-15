#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ -f "$ROOT_DIR/.env" ]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/.env"
fi

TEST_DB_SOURCE=""
TEST_DB_URL=""

if [ -n "${TEST_DATABASE_URL:-}" ]; then
  TEST_DB_SOURCE="TEST_DATABASE_URL"
  TEST_DB_URL="$TEST_DATABASE_URL"
elif [ -n "${NEON_TEST_DATABASE_URL:-}" ]; then
  TEST_DB_SOURCE="NEON_TEST_DATABASE_URL"
  TEST_DB_URL="$NEON_TEST_DATABASE_URL"
fi

if [ -z "$TEST_DB_URL" ]; then
  echo "No test database URL found. Set TEST_DATABASE_URL or NEON_TEST_DATABASE_URL for online validation."
  echo "Falling back to SQLX_OFFLINE=true cargo test because no reachable test database was configured."
  SQLX_OFFLINE=true cargo test
  exit 0
fi

lower_url="$(printf '%s' "$TEST_DB_URL" | tr '[:upper:]' '[:lower:]')"
if [ -n "${DATABASE_URL:-}" ] && [ "$TEST_DB_URL" = "$DATABASE_URL" ]; then
  echo "Refusing to use $TEST_DB_SOURCE because it matches DATABASE_URL."
  echo "Use a separate Neon test/dev branch URL; production databases must never be used for tests."
  exit 1
fi

if [[ "$lower_url" =~ (prod|production|live|main) ]]; then
  echo "Refusing to use $TEST_DB_SOURCE because it appears to reference a production database."
  echo "Use only a Neon test/dev branch URL for migrations and tests."
  exit 1
fi

echo "Using $TEST_DB_SOURCE for test database validation."
echo "Running migrations against the selected test database."
DATABASE_URL="$TEST_DB_URL" sqlx migrate run
echo "Running cargo test against the selected test database."
DATABASE_URL="$TEST_DB_URL" cargo test

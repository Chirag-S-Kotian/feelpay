#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ -f "$PROJECT_DIR/.env" ]; then
  # shellcheck disable=SC1091
  source "$PROJECT_DIR/.env"
fi

DB_PROVIDER="${DATABASE_PROVIDER:-auto}"
DB_URL="${DATABASE_URL:-}"

if [ "$DB_PROVIDER" = "neon" ] && [ -n "${DATABASE_URL_NEON:-}" ]; then
  DB_URL="$DATABASE_URL_NEON"
elif [ "$DB_PROVIDER" = "docker" ] && [ -n "${DATABASE_URL_DOCKER:-}" ]; then
  DB_URL="$DATABASE_URL_DOCKER"
elif [ "$DB_PROVIDER" = "auto" ]; then
  DB_URL="${DATABASE_URL_NEON:-${DATABASE_URL_DOCKER:-$DB_URL}}"
fi

if [ -z "$DB_URL" ]; then
  echo "DATABASE_URL missing. Configure .env from .env.example"
  exit 1
fi

echo "Starting backend with provider: $DB_PROVIDER"
echo "Migrations will run automatically at startup."
DATABASE_URL="$DB_URL" cargo run --release

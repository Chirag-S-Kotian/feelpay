#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DASH_DIR="$PROJECT_DIR/dashboard"

RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[38;5;51m'
BLUE='\033[38;5;39m'
GREEN='\033[38;5;46m'
YELLOW='\033[38;5;220m'
RED='\033[38;5;196m'

line() {
  printf "%b%s%b\n" "${DIM}${BLUE}" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" "${RESET}"
}

title() {
  clear || true
  printf "%b\n" "${BOLD}${CYAN}"
  echo "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"
  echo "┃                   FEELPAY RUNTIME CONSOLE                   ┃"
  echo "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"
  printf "%b\n" "${RESET}"
  line
}

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo -e "${RED}Missing dependency:${RESET} $1"
    exit 1
  fi
}

choose() {
  local prompt="$1"
  shift
  local options=("$@")
  local i=1
  echo -e "${BOLD}${prompt}${RESET}" >&2
  for opt in "${options[@]}"; do
    echo "  [$i] $opt" >&2
    ((i++))
  done
  local sel
  while true; do
    read -rp "Select option number: " sel >&2
    if [[ "$sel" =~ ^[0-9]+$ ]] && (( sel >= 1 && sel <= ${#options[@]} )); then
      echo "${options[$((sel-1))]}"
      return 0
    fi
    echo -e "${YELLOW}Invalid input. Try again.${RESET}" >&2
  done
}

load_env() {
  if [[ -f "$PROJECT_DIR/.env" ]]; then
    # shellcheck disable=SC1091
    source "$PROJECT_DIR/.env"
  fi
}

resolve_db_url() {
  local provider="$1"
  if [[ "$provider" == "neon" && -n "${DATABASE_URL_NEON:-}" ]]; then
    echo "$DATABASE_URL_NEON"
    return
  fi
  if [[ "$provider" == "neon" ]]; then
    echo ""
    return
  fi
  if [[ "$provider" == "docker" && -n "${DATABASE_URL_DOCKER:-}" ]]; then
    echo "$DATABASE_URL_DOCKER"
    return
  fi
  if [[ "$provider" == "docker" ]]; then
    echo ""
    return
  fi
  if [[ "$provider" == "auto" ]]; then
    if [[ -n "${DATABASE_URL_NEON:-}" ]]; then
      echo "$DATABASE_URL_NEON"
      return
    fi
    if [[ -n "${DATABASE_URL_DOCKER:-}" ]]; then
      echo "$DATABASE_URL_DOCKER"
      return
    fi
  fi
  echo "${DATABASE_URL:-}"
}

start_frontend() {
  cd "$DASH_DIR"
  if [[ ! -d node_modules ]]; then
    echo -e "${YELLOW}Installing dashboard deps...${RESET}"
    pnpm install
  fi
  echo -e "${GREEN}Starting dashboard on http://localhost:3000${RESET}"
  pnpm dev &
  FRONT_PID=$!
}

start_backend() {
  cd "$PROJECT_DIR"
  local profile="$1"
  if [[ "$profile" == "release" ]]; then
    echo -e "${GREEN}Starting backend (release) on http://localhost:8080${RESET}"
    cargo run --release &
  else
    echo -e "${GREEN}Starting backend (dev) on http://localhost:8080${RESET}"
    cargo run &
  fi
  BACK_PID=$!
}

title
load_env
check_dep cargo
check_dep pnpm

RUN_TARGET="$(choose "What to run?" "backend" "frontend" "both")"
PAYMENT_MODE="$(choose "Choose payment mode" "test" "live")"
DB_PROVIDER_CHOICE="$(choose "Choose database provider" "neon" "docker" "auto")"
BUILD_PROFILE="$(choose "Choose backend profile" "dev" "release")"

DB_URL="$(resolve_db_url "$DB_PROVIDER_CHOICE")"
if [[ -z "$DB_URL" ]]; then
  echo -e "${RED}Selected provider URL missing.${RESET}"
  if [[ "$DB_PROVIDER_CHOICE" == "neon" ]]; then
    echo -e "${YELLOW}Set DATABASE_URL_NEON in .env${RESET}"
  elif [[ "$DB_PROVIDER_CHOICE" == "docker" ]]; then
    echo -e "${YELLOW}Set DATABASE_URL_DOCKER in .env${RESET}"
  else
    echo -e "${YELLOW}Set DATABASE_URL or provider-specific URLs in .env${RESET}"
  fi
  exit 1
fi

if [[ "$DB_PROVIDER_CHOICE" == "neon" ]] && [[ "$DB_URL" == *"localhost"* || "$DB_URL" == *"127.0.0.1"* ]]; then
  echo -e "${YELLOW}Warning:${RESET} Neon selected but URL points to local host. Check DATABASE_URL_NEON."
fi

export PAYMENT_MODE="$PAYMENT_MODE"
export DATABASE_PROVIDER="$DB_PROVIDER_CHOICE"
export DATABASE_URL="$DB_URL"

line
echo -e "${BOLD}Execution Plan${RESET}"
echo "  target        : $RUN_TARGET"
echo "  payment_mode  : $PAYMENT_MODE"
echo "  db_provider   : $DB_PROVIDER_CHOICE"
echo "  backend_build : $BUILD_PROFILE"
echo "  db_url_host   : $(echo "$DB_URL" | sed -E 's#(postgres(ql)?://)[^@]*@#\1***@#')"
line
read -rp "Proceed? (y/N): " proceed
if [[ "${proceed,,}" != "y" ]]; then
  echo "Cancelled."
  exit 0
fi

FRONT_PID=""
BACK_PID=""

case "$RUN_TARGET" in
  backend)
    start_backend "$BUILD_PROFILE"
    ;;
  frontend)
    start_frontend
    ;;
  both)
    start_frontend
    sleep 2
    start_backend "$BUILD_PROFILE"
    ;;
esac

line
echo -e "${BOLD}${CYAN}FeelPay running. Ctrl+C to stop.${RESET}"

cleanup() {
  [[ -n "${FRONT_PID}" ]] && kill "${FRONT_PID}" 2>/dev/null || true
  [[ -n "${BACK_PID}" ]] && kill "${BACK_PID}" 2>/dev/null || true
}
trap cleanup INT TERM
wait

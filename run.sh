#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DASH_DIR="$PROJECT_DIR/dashboard"
ADMIN_FRONT_DIR="$PROJECT_DIR/admin"
ADMIN_SERVER_DIR="$PROJECT_DIR/admin/server"

# Service Ports
GATEWAY_PORT=8080
ADMIN_SERVER_PORT=8081
DASH_PORT=3000
ADMIN_FRONT_PORT=3001

# ANSI Color Palette
RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[38;5;51m'
BLUE='\033[38;5;39m'
GREEN='\033[38;5;46m'
YELLOW='\033[38;5;220m'
RED='\033[38;5;196m'
MAGENTA='\033[38;5;207m'

line() {
  printf "%b%s%b\n" "${DIM}${BLUE}" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" "${RESET}"
}

title() {
  clear || true
  printf "%b\n" "${BOLD}${CYAN}"
  echo "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"
  echo "┃                   FEELPAY RUNTIME CONSOLE                   ┃"
  echo "┃       Gateway API • Admin Backend • Dashboard • Admin UI     ┃"
  echo "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"
  printf "%b\n" "${RESET}"
  line
}

check_dep() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo -e "${RED}Missing required dependency:${RESET} $1"
    exit 1
  fi
}

load_env() {
  if [[ -f "$PROJECT_DIR/.env" ]]; then
    # shellcheck disable=SC1091
    source "$PROJECT_DIR/.env"
  fi
  if [[ -f "$ADMIN_SERVER_DIR/.env" ]]; then
    # shellcheck disable=SC1091
    source "$ADMIN_SERVER_DIR/.env"
  fi
}

resolve_db_url() {
  echo "${DATABASE_URL:-}"
}

# Process Tracking
GATEWAY_PID=""
ADMIN_SERVER_PID=""
DASH_PID=""
ADMIN_FRONT_PID=""
CLEANUP_DONE=0
ACTIVE_PORTS=""

# Recursively kill a PID and its child process tree
kill_process_tree() {
  local pid="$1"
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    local children
    children=$(pgrep -P "$pid" 2>/dev/null || true)
    for child in $children; do
      kill_process_tree "$child"
    done
    kill -TERM "$pid" 2>/dev/null || true
    sleep 0.2
    if kill -0 "$pid" 2>/dev/null; then
      kill -KILL "$pid" 2>/dev/null || true
    fi
  fi
}

cleanup() {
  if [[ "$CLEANUP_DONE" -eq 1 ]]; then
    return 0
  fi
  CLEANUP_DONE=1

  if [[ -z "${GATEWAY_PID}" && -z "${ADMIN_SERVER_PID}" && -z "${DASH_PID}" && -z "${ADMIN_FRONT_PID}" && -z "${ACTIVE_PORTS}" ]]; then
    return 0
  fi

  echo ""
  line
  echo -e "${YELLOW}Stopping FeelPay services...${RESET}"

  [[ -n "${GATEWAY_PID}" ]] && { echo -e "  Stopping Gateway API (PID ${GATEWAY_PID})..."; kill_process_tree "${GATEWAY_PID}"; }
  [[ -n "${ADMIN_SERVER_PID}" ]] && { echo -e "  Stopping Admin Backend (PID ${ADMIN_SERVER_PID})..."; kill_process_tree "${ADMIN_SERVER_PID}"; }
  [[ -n "${DASH_PID}" ]] && { echo -e "  Stopping Merchant Dashboard (PID ${DASH_PID})..."; kill_process_tree "${DASH_PID}"; }
  [[ -n "${ADMIN_FRONT_PID}" ]] && { echo -e "  Stopping Admin Portal (PID ${ADMIN_FRONT_PID})..."; kill_process_tree "${ADMIN_FRONT_PID}"; }

  # Double-check ports that were running
  for port in $ACTIVE_PORTS; do
    local pids
    pids=$(lsof -ti :"$port" 2>/dev/null || true)
    if [[ -n "$pids" ]]; then
      # shellcheck disable=SC2086
      kill -9 $pids 2>/dev/null || true
    fi
  done

  echo -e "${GREEN}All FeelPay services stopped cleanly. Goodbye!${RESET}"
  line
}

trap cleanup INT TERM EXIT HUP

# Port conflict & stale process resolution
get_port_pid() {
  local port="$1"
  lsof -ti :"$port" 2>/dev/null || true
}

kill_port() {
  local port="$1"
  local pids
  pids=$(get_port_pid "$port")
  if [[ -n "$pids" ]]; then
    echo -e "${YELLOW}Killing processes on port :${port} (PIDs: ${pids})...${RESET}"
    # shellcheck disable=SC2086
    kill -9 $pids 2>/dev/null || true
    sleep 0.5
  fi
}

check_and_free_port() {
  local port="$1"
  local svc_name="$2"
  local pids
  pids=$(get_port_pid "$port")
  if [[ -n "$pids" ]]; then
    echo -e "${YELLOW}Warning: Port :${port} (${svc_name}) is currently in use by PID(s): ${pids}.${RESET}"
    if [[ "$AUTO_CLEAN" == "true" ]]; then
      kill_port "$port"
    else
      read -rp "Terminate stale process on port :${port}? (Y/n): " confirm_kill
      confirm_kill="${confirm_kill:-y}"
      if [[ "${confirm_kill,,}" == "y" ]]; then
        kill_port "$port"
      else
        echo -e "${RED}Cannot bind port :${port}. Exiting.${RESET}"
        exit 1
      fi
    fi
  fi
}

# Service Launchers
start_gateway() {
  local profile="$1"
  check_and_free_port "$GATEWAY_PORT" "Gateway API"
  cd "$PROJECT_DIR"
  echo -e "${CYAN}▶ Launching FeelPay Gateway API on port ${GATEWAY_PORT}...${RESET}"
  export PORT="$GATEWAY_PORT"
  export HOST="0.0.0.0"
  export PAYMENT_MODE="$PAYMENT_MODE"
  export DATABASE_URL="$DB_URL"
  if [[ "$profile" == "release" ]]; then
    cargo run --release &
  else
    cargo run &
  fi
  GATEWAY_PID=$!
  ACTIVE_PORTS="${ACTIVE_PORTS} ${GATEWAY_PORT}"
}

start_admin_server() {
  local profile="$1"
  check_and_free_port "$ADMIN_SERVER_PORT" "Admin Backend"
  cd "$ADMIN_SERVER_DIR"
  echo -e "${CYAN}▶ Launching FeelPay Admin Backend on port ${ADMIN_SERVER_PORT}...${RESET}"
  export ADMIN_HOST="127.0.0.1"
  export ADMIN_PORT="$ADMIN_SERVER_PORT"
  export DATABASE_URL="$DB_URL"
  export PAYMENT_MODE="$PAYMENT_MODE"
  export JWT_SECRET="${JWT_SECRET:-feelpay_jwt_secret}"
  export VAULT_ENCRYPTION_KEY="${VAULT_ENCRYPTION_KEY:-0000000000000000000000000000000000000000000000000000000000000000}"
  export ADMIN_CORS_ALLOWED_ORIGINS="http://localhost:3001,http://127.0.0.1:3001,http://localhost:3000,http://127.0.0.1:3000,http://localhost:5173,http://127.0.0.1:5173,http://localhost:5174,http://127.0.0.1:5174"
  if [[ "$profile" == "release" ]]; then
    cargo run --release &
  else
    cargo run &
  fi
  ADMIN_SERVER_PID=$!
  ACTIVE_PORTS="${ACTIVE_PORTS} ${ADMIN_SERVER_PORT}"
}

start_dashboard() {
  check_and_free_port "$DASH_PORT" "Merchant Dashboard"
  cd "$DASH_DIR"
  if [[ ! -d node_modules ]]; then
    echo -e "${YELLOW}Installing Merchant Dashboard dependencies...${RESET}"
    pnpm install
  fi
  echo -e "${CYAN}▶ Launching Merchant Dashboard on port ${DASH_PORT}...${RESET}"
  pnpm dev --host 0.0.0.0 --port "$DASH_PORT" &
  DASH_PID=$!
  ACTIVE_PORTS="${ACTIVE_PORTS} ${DASH_PORT}"
}

start_admin_frontend() {
  check_and_free_port "$ADMIN_FRONT_PORT" "Admin Portal UI"
  cd "$ADMIN_FRONT_DIR"
  if [[ ! -d node_modules ]]; then
    echo -e "${YELLOW}Installing Admin Portal dependencies...${RESET}"
    pnpm install
  fi
  echo -e "${CYAN}▶ Launching Admin Portal UI on port ${ADMIN_FRONT_PORT}...${RESET}"
  pnpm dev --host 0.0.0.0 --port "$ADMIN_FRONT_PORT" &
  ADMIN_FRONT_PID=$!
  ACTIVE_PORTS="${ACTIVE_PORTS} ${ADMIN_FRONT_PORT}"
}

wait_for_health() {
  local url="$1"
  local name="$2"
  local max_attempts="${3:-90}"
  echo -ne "${YELLOW}Waiting for ${name} (${url})...${RESET}"
  local count=0
  while true; do
    if curl -sf "$url" >/dev/null 2>&1; then
      echo -e " ${GREEN}[READY]${RESET}"
      return 0
    fi
    count=$((count + 1))
    if (( count >= max_attempts )); then
      echo -e " ${RED}[TIMED OUT after ${max_attempts}s]${RESET}"
      echo -e "${RED}Error: ${name} failed to report healthy.${RESET}"
      exit 1
    fi
    echo -ne "."
    sleep 1
  done
}

wait_for_port() {
  local port="$1"
  local name="$2"
  local max_attempts="${3:-30}"
  echo -ne "${YELLOW}Waiting for ${name} on port :${port}...${RESET}"
  local count=0
  while true; do
    if curl -sf -I "http://localhost:${port}" >/dev/null 2>&1 || curl -sf -I "http://127.0.0.1:${port}" >/dev/null 2>&1 || (echo > /dev/tcp/127.0.0.1/"$port") >/dev/null 2>&1; then
      echo -e " ${GREEN}[READY]${RESET}"
      return 0
    fi
    count=$((count + 1))
    if (( count >= max_attempts )); then
      echo -e " ${RED}[TIMED OUT after ${max_attempts}s]${RESET}"
      echo -e "${RED}Error: ${name} failed to open port :${port}.${RESET}"
      exit 1
    fi
    echo -ne "."
    sleep 1
  done
}

show_help() {
  echo -e "${BOLD}FeelPay Unified Runner${RESET}"
  echo ""
  echo "Usage:"
  echo "  ./run.sh [TARGET] [OPTIONS]"
  echo ""
  echo "Targets:"
  echo "  all | 4               Run all 4 services (Gateway + Admin Backend + Dashboard + Admin Portal)"
  echo "  both | gateway-stack  Run Gateway API (:8080) and Merchant Dashboard (:3000)"
  echo "  admin-stack           Run Admin Backend (:8081) and Admin Portal (:3001)"
  echo "  gateway | backend     Run FeelPay Gateway Backend only (:8080)"
  echo "  admin-server          Run FeelPay Admin Backend only (:8081)"
  echo "  dashboard | frontend  Run Merchant Dashboard only (:3000)"
  echo "  admin-ui              Run Admin Portal Frontend only (:3001)"
  echo "  clean | kill-ports    Clean / kill all stale processes on ports 8080, 8081, 3000, 3001"
  echo ""
  echo "Options:"
  echo "  --dev                 Run Rust backend(s) in dev mode (default)"
  echo "  --release             Run Rust backend(s) with --release optimization"
  echo "  --test                Set PAYMENT_MODE=test (default)"
  echo "  --live                Set PAYMENT_MODE=live (requires production config)"
  echo "  --clean               Automatically kill processes on conflict ports without prompting"
  echo "  -y, --yes             Skip confirmation prompts and proceed immediately"
  echo "  -h, --help            Show this help message"
  echo ""
  echo "Examples:"
  echo "  ./run.sh all          # Launch all 4 services"
  echo "  ./run.sh all -y       # Launch all 4 services immediately without prompting"
  echo "  ./run.sh both         # Launch Gateway and Merchant Dashboard"
  echo "  ./run.sh admin-stack  # Launch Admin Backend and Admin UI"
  echo "  ./run.sh clean        # Terminate any leftover processes on 8080/8081/3000/3001"
  exit 0
}

# CLI Argument Defaults
TARGET=""
PAYMENT_MODE_ARG=""
BUILD_PROFILE_ARG=""
AUTO_CLEAN="false"
SKIP_CONFIRM="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      show_help
      ;;
    all|4|full)
      TARGET="all"
      shift
      ;;
    both|gateway-stack)
      TARGET="both"
      shift
      ;;
    admin-stack)
      TARGET="admin-stack"
      shift
      ;;
    gateway|backend)
      TARGET="gateway"
      shift
      ;;
    admin-server|admin-backend)
      TARGET="admin-server"
      shift
      ;;
    dashboard|frontend)
      TARGET="dashboard"
      shift
      ;;
    admin-ui|admin-frontend)
      TARGET="admin-ui"
      shift
      ;;
    clean|kill-ports)
      TARGET="clean"
      shift
      ;;
    --dev)
      BUILD_PROFILE_ARG="dev"
      shift
      ;;
    --release)
      BUILD_PROFILE_ARG="release"
      shift
      ;;
    --test)
      PAYMENT_MODE_ARG="test"
      shift
      ;;
    --live)
      PAYMENT_MODE_ARG="live"
      shift
      ;;
    --clean)
      AUTO_CLEAN="true"
      shift
      ;;
    -y|--yes)
      SKIP_CONFIRM="true"
      shift
      ;;
    *)
      echo -e "${RED}Unknown option or target: $1${RESET}"
      echo "Run './run.sh --help' for usage."
      exit 1
      ;;
  esac
done

title
load_env
check_dep cargo
check_dep pnpm
check_dep curl

# Standalone Clean / Port Kill Mode
if [[ "$TARGET" == "clean" ]]; then
  echo -e "${YELLOW}Killing all processes on FeelPay ports (8080, 8081, 3000, 3001)...${RESET}"
  kill_port "$GATEWAY_PORT"
  kill_port "$ADMIN_SERVER_PORT"
  kill_port "$DASH_PORT"
  kill_port "$ADMIN_FRONT_PORT"
  echo -e "${GREEN}All FeelPay ports freed successfully.${RESET}"
  exit 0
fi

# Interactive Selection if TARGET not supplied
if [[ -z "$TARGET" ]]; then
  echo -e "${BOLD}Select execution mode:${RESET}"
  echo "  [1] All 4 Services (Gateway + Admin Backend + Dashboard + Admin Portal) [Default]"
  echo "  [2] Gateway Stack (Gateway API :8080 + Merchant Dashboard :3000)"
  echo "  [3] Admin Stack (Admin Backend :8081 + Admin Portal :3001)"
  echo "  [4] FeelPay Gateway API Only (:8080)"
  echo "  [5] FeelPay Admin Backend Only (:8081)"
  echo "  [6] Merchant Dashboard Only (:3000)"
  echo "  [7] Admin Portal UI Only (:3001)"
  echo "  [8] Clean / Kill All Stale Port Holders (8080, 8081, 3000, 3001)"
  echo ""
  read -rp "Enter choice [1-8, Default: 1]: " menu_sel
  menu_sel="${menu_sel:-1}"

  case "$menu_sel" in
    1) TARGET="all" ;;
    2) TARGET="both" ;;
    3) TARGET="admin-stack" ;;
    4) TARGET="gateway" ;;
    5) TARGET="admin-server" ;;
    6) TARGET="dashboard" ;;
    7) TARGET="admin-ui" ;;
    8)
      kill_port "$GATEWAY_PORT"
      kill_port "$ADMIN_SERVER_PORT"
      kill_port "$DASH_PORT"
      kill_port "$ADMIN_FRONT_PORT"
      echo -e "${GREEN}All FeelPay ports freed successfully.${RESET}"
      exit 0
      ;;
    *)
      echo -e "${RED}Invalid choice. Defaulting to all 4 services.${RESET}"
      TARGET="all"
      ;;
  esac
fi

PAYMENT_MODE="${PAYMENT_MODE_ARG:-test}"
BUILD_PROFILE="${BUILD_PROFILE_ARG:-dev}"

if [[ -z "$PAYMENT_MODE_ARG" && "$SKIP_CONFIRM" != "true" ]]; then
  echo ""
  read -rp "Select payment mode (test/live) [Default: test]: " pm_input
  pm_input="${pm_input:-test}"
  PAYMENT_MODE="$pm_input"
fi

if [[ -z "$BUILD_PROFILE_ARG" && "$SKIP_CONFIRM" != "true" && ( "$TARGET" == "all" || "$TARGET" == "both" || "$TARGET" == "admin-stack" || "$TARGET" == "gateway" || "$TARGET" == "admin-server" ) ]]; then
  read -rp "Select Rust backend build profile (dev/release) [Default: dev]: " bp_input
  bp_input="${bp_input:-dev}"
  BUILD_PROFILE="$bp_input"
fi

DB_URL="$(resolve_db_url)"
if [[ -z "$DB_URL" ]]; then
  echo -e "${RED}DATABASE_URL is missing in .env.${RESET}"
  exit 1
fi

export PAYMENT_MODE="$PAYMENT_MODE"
export DATABASE_URL="$DB_URL"

line
echo -e "${BOLD}Execution Plan${RESET}"
echo "  Target             : $TARGET"
echo "  Payment Mode       : $PAYMENT_MODE"
echo "  Backend Profile    : $BUILD_PROFILE"
echo "  Postgres Host      : $(echo "$DB_URL" | sed -E 's#(postgres(ql)?://)[^@]*@#\1***@#')"
line

if [[ "$SKIP_CONFIRM" != "true" ]]; then
  read -rp "Proceed with execution? (Y/n) [Default: Y]: " proceed
  proceed="${proceed:-y}"
  if [[ "${proceed,,}" != "y" ]]; then
    echo "Aborted by user."
    exit 0
  fi
fi

echo ""
line
echo -e "${BOLD}${GREEN}Starting FeelPay Services...${RESET}"
line

case "$TARGET" in
  all)
    start_gateway "$BUILD_PROFILE"
    start_admin_server "$BUILD_PROFILE"
    echo ""
    wait_for_health "http://127.0.0.1:8080/v1/health" "FeelPay Gateway API" 90
    wait_for_health "http://127.0.0.1:8081/health" "FeelPay Admin Backend" 90
    echo ""
    start_dashboard
    start_admin_frontend
    wait_for_port "$DASH_PORT" "Merchant Dashboard" 30
    wait_for_port "$ADMIN_FRONT_PORT" "Admin Portal UI" 30
    ;;

  both)
    start_gateway "$BUILD_PROFILE"
    echo ""
    wait_for_health "http://127.0.0.1:8080/v1/health" "FeelPay Gateway API" 90
    echo ""
    start_dashboard
    wait_for_port "$DASH_PORT" "Merchant Dashboard" 30
    ;;

  admin-stack)
    start_admin_server "$BUILD_PROFILE"
    echo ""
    wait_for_health "http://127.0.0.1:8081/health" "FeelPay Admin Backend" 90
    echo ""
    start_admin_frontend
    wait_for_port "$ADMIN_FRONT_PORT" "Admin Portal UI" 30
    ;;

  gateway)
    start_gateway "$BUILD_PROFILE"
    wait_for_health "http://127.0.0.1:8080/v1/health" "FeelPay Gateway API" 90
    ;;

  admin-server)
    start_admin_server "$BUILD_PROFILE"
    wait_for_health "http://127.0.0.1:8081/health" "FeelPay Admin Backend" 90
    ;;

  dashboard)
    start_dashboard
    wait_for_port "$DASH_PORT" "Merchant Dashboard" 30
    ;;

  admin-ui)
    start_admin_frontend
    wait_for_port "$ADMIN_FRONT_PORT" "Admin Portal UI" 30
    ;;
esac

echo ""
line
printf "%b\n" "${BOLD}${CYAN}"
echo "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"
echo "┃               FEELPAY SERVICES ARE RUNNING LIVE              ┃"
echo "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫"
if [[ "$TARGET" == "all" || "$TARGET" == "both" || "$TARGET" == "gateway" ]]; then
echo "┃  • FeelPay Gateway API    : http://127.0.0.1:8080            ┃"
echo "┃    Health Endpoint        : http://127.0.0.1:8080/v1/health  ┃"
fi
if [[ "$TARGET" == "all" || "$TARGET" == "admin-stack" || "$TARGET" == "admin-server" ]]; then
echo "┃  • FeelPay Admin Backend  : http://127.0.0.1:8081            ┃"
echo "┃    Health Endpoint        : http://127.0.0.1:8081/health     ┃"
fi
if [[ "$TARGET" == "all" || "$TARGET" == "both" || "$TARGET" == "dashboard" ]]; then
echo "┃  • Merchant Dashboard UI  : http://localhost:3000            ┃"
fi
if [[ "$TARGET" == "all" || "$TARGET" == "admin-stack" || "$TARGET" == "admin-ui" ]]; then
echo "┃  • Admin Portal UI        : http://localhost:3001            ┃"
fi
echo "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"
printf "%b\n" "${RESET}"
echo -e "${DIM}Services are active in background. Press ${BOLD}Ctrl+C${RESET}${DIM} to stop all services cleanly.${RESET}"
line
echo ""

# Keep runner alive until interrupted
wait

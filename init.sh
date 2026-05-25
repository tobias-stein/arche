#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { printf "${BLUE}[INFO]${NC}  %s\n" "$*"; }
log_ok()    { printf "${GREEN}[OK]${NC}    %s\n" "$*"; }
log_warn()  { printf "${YELLOW}[WARN]${NC}  %s\n" "$*"; }
log_error() { printf "${RED}[ERROR]${NC} %s\n" "$*"; }

check_cmd() {
  if ! command -v "$1" &>/dev/null; then
    log_error "$1 is required but not found. Please install it."
    exit 1
  fi
}

# ── Docker mode ──────────────────────────────────────────────────────

if [ "${1:-}" = "--docker" ]; then
  if ! docker compose version >/dev/null 2>&1 && ! docker-compose --version >/dev/null 2>&1; then
    log_error "Docker and Docker Compose are required."
    exit 1
  fi

  log_info "Starting via docker compose..."
  if ! docker compose up --build; then
    echo ""
    log_warn "Startup failed. If PostgreSQL has a stale volume, run:"
    log_warn "  docker compose down -v && ./init.sh --docker"
    exit 1
  fi
  exit 0
fi

# ── Native mode ──────────────────────────────────────────────────────

cleanup() {
  echo ""
  log_info "Shutting down..."
  [ -n "${SERVICE_PID:-}" ] && kill "$SERVICE_PID" 2>/dev/null && log_info "Stopped service (PID $SERVICE_PID)"
  [ -n "${UI_PID:-}" ] && kill "$UI_PID" 2>/dev/null && log_info "Stopped admin-ui (PID $UI_PID)"
  exit 0
}
trap cleanup SIGINT SIGTERM

log_info "Checking prerequisites..."
check_cmd rustc
check_cmd cargo
check_cmd node
check_cmd npm
check_cmd psql
log_ok "All prerequisites found"

log_info "Checking PostgreSQL connection..."
if ! pg_isready -q 2>/dev/null; then
  log_error "PostgreSQL is not running. Start it with: brew services start postgresql (macOS) or sudo systemctl start postgresql (Linux)"
  exit 1
fi

DEFAULT_DB_URL="postgres://postgres:postgres@localhost:5432/arche"
DB_URL="${ARCHE_DATABASE_URL:-$DEFAULT_DB_URL}"
DB_NAME="${DB_URL##*/}"
DB_NAME="${DB_NAME%%\?*}"

ADMIN_DB_URL="${DB_URL%/*}/postgres"

log_info "Ensuring database '${DB_NAME}' exists..."
if psql "$ADMIN_DB_URL" -tc \
  "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'" 2>/dev/null | grep -q 1; then
  log_ok "Database '${DB_NAME}' already exists"
else
  createdb "${DB_NAME}" 2>/dev/null || \
    psql "$ADMIN_DB_URL" -c "CREATE DATABASE \"${DB_NAME}\"" >/dev/null 2>&1
  log_ok "Created database '${DB_NAME}'"
fi

if [ ! -f .env ]; then
  {
    echo "# Arche local development environment"
    echo "ARCHE_DATABASE_URL=${DB_URL}"
    echo "ARCHE_PORT=${ARCHE_PORT:-8080}"
  } > .env
  log_ok "Created .env file with defaults"
else
  log_ok ".env file already exists"
fi

SERVICE_PORT="${ARCHE_PORT:-8080}"
ADMIN_UI_PORT=5173

log_info "Building and starting service (port ${SERVICE_PORT})..."
cargo run -p arche-service &
SERVICE_PID=$!
log_info "Service starting in background (PID $SERVICE_PID)"

log_info "Waiting for service to be ready..."
for i in $(seq 1 60); do
  if curl -sf "http://localhost:${SERVICE_PORT}/health" >/dev/null 2>&1; then
    log_ok "Service is ready at http://localhost:${SERVICE_PORT}"
    break
  fi
  if [ $i -eq 60 ]; then
    log_error "Service failed to start within 60 seconds"
    cleanup
  fi
  sleep 1
done

log_info "Setting up admin-ui..."
cd admin-ui

if [ ! -d node_modules ]; then
  log_info "Installing npm dependencies..."
  npm install
  log_ok "Dependencies installed"
fi

log_info "Starting admin-ui dev server..."
VITE_ARCHE_API_URL="http://localhost:${SERVICE_PORT}" npm run dev &
UI_PID=$!

cd "$SCRIPT_DIR"

log_ok "=== Local environment is running ==="
echo ""
printf "  ${GREEN}Service:${NC}  http://localhost:${SERVICE_PORT}\n"
printf "  ${GREEN}Admin UI:${NC} http://localhost:${ADMIN_UI_PORT}\n"
echo ""
printf "  ${YELLOW}Press Ctrl+C to stop all processes${NC}\n"
echo ""

wait

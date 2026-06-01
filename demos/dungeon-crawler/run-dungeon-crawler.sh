#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

cleanup() {
  echo ""
  echo "Shutting down dungeon crawler services..."
  docker compose down -v 2>/dev/null || true
  echo "Done."
  exit 0
}
trap cleanup SIGINT SIGTERM

# Ensure a clean state
echo "==> Cleaning up any previous services..."
docker compose down -v 2>/dev/null || true

echo "==> Starting PostgreSQL and Arche service..."
docker compose up -d postgres arche-service

echo "==> Waiting for Arche API to be ready..."
until curl -s http://localhost:8080/health >/dev/null 2>&1; do
  sleep 1
done
echo "   Arche API is ready."

echo "==> Extracting super admin key from Arche logs..."
ARCHE_API_KEY=""
for i in $(seq 1 30); do
  ARCHE_API_KEY=$(docker compose logs arche-service 2>/dev/null | grep -o 'arche_k_[a-zA-Z0-9]\{48\}' | head -1 || true)
  if [ -n "$ARCHE_API_KEY" ]; then
    break
  fi
  sleep 1
done

if [ -z "$ARCHE_API_KEY" ]; then
  echo "ERROR: Could not extract super admin key from Arche logs."
  echo "If Arche was already bootstrapped, set ARCHE_API_KEY=<key> and re-run."
  cleanup
fi
echo "   Super admin key extracted."

echo "==> Running seed script..."
SEED_OUTPUT=$(docker compose run --rm -e ARCHE_API_KEY="$ARCHE_API_KEY" seed 2>/dev/null)
echo "$SEED_OUTPUT"
echo "   Seed complete."

# Extract the client API key from seed output, fall back to super admin key
CLIENT_API_KEY=$(echo "$SEED_OUTPUT" | grep "^API Key:" | awk '{print $NF}')
ARCHE_API_KEY="${CLIENT_API_KEY:-$ARCHE_API_KEY}"
export ARCHE_API_KEY

echo "==> Building and starting dungeon crawler..."
docker compose up -d --build dungeon-crawler

PORT="${DUNGEON_CRAWLER_PORT:-5173}"

# Verify the port is reachable
if command -v curl &>/dev/null; then
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${PORT}/" 2>/dev/null || echo "000")
  if [ "$HTTP_CODE" = "000" ]; then
    echo "   (port ${PORT} not yet responding, retrying...)"
    for i in $(seq 1 10); do
      sleep 1
      HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${PORT}/" 2>/dev/null || echo "000")
      [ "$HTTP_CODE" != "000" ] && break
    done
  fi
  if [ "$HTTP_CODE" != "200" ]; then
    echo "   WARNING: Got HTTP ${HTTP_CODE} on port ${PORT}."
    echo "   Another service (e.g. a stale Vite dev server) might be using this port."
    echo "   Try: kill \$(lsof -ti :${PORT}) && docker compose restart dungeon-crawler"
  fi
fi

echo ""
echo "============================================="
echo " Dungeon Crawler is running!"
echo " Open http://localhost:${PORT} in your browser."
echo " Press Ctrl+C to stop all services."
echo "============================================="
echo ""

while true; do
  sleep 1
done

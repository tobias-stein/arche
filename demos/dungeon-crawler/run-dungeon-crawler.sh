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

# Ensure a clean state — remove stale volumes from previous runs
echo "==> Cleaning up any previous services..."
docker compose down -v 2>/dev/null || true

echo "==> Starting PostgreSQL and Arche service..."
docker compose up -d postgres arche-service

echo "==> Waiting for Arche API to be ready..."
until curl -s http://localhost:8080/health >/dev/null 2>&1; do
  sleep 1
done
echo "   Arche API is ready."

echo "==> Bootstrapping super admin key..."
BOOTSTRAP_KEY=$(curl -s http://localhost:8080/api/bootstrap | grep -o '"key":"[^"]*"' | cut -d'"' -f4 || true)
if [ -n "$BOOTSTRAP_KEY" ]; then
  ARCHE_API_KEY="$BOOTSTRAP_KEY"
elif [ -z "${ARCHE_API_KEY:-}" ]; then
  echo "ERROR: Arche is already bootstrapped but no ARCHE_API_KEY is set."
  echo "Set ARCHE_API_KEY=<key> when running, or delete the pgdata volume and re-run."
  cleanup
fi
export ARCHE_API_KEY

echo "==> Running seed script..."
docker compose run --rm -e ARCHE_API_KEY="$ARCHE_API_KEY" seed
echo "   Seed complete."

echo "==> Building and starting dungeon crawler..."
docker compose up -d --build dungeon-crawler

PORT="${DUNGEON_CRAWLER_PORT:-5173}"
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

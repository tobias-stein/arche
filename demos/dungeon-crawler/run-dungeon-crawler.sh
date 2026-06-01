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

echo "==> Starting PostgreSQL and Arche service..."
docker compose up -d postgres arche-service

echo "==> Waiting for Arche API to be ready..."
until curl -s http://localhost:8080/health >/dev/null 2>&1; do
  sleep 1
done
echo "   Arche API is ready."

echo "==> Running seed script..."
docker compose run --rm seed
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

#!/bin/bash
# Dungeon Crawler demo — one-shot setup
# Runs all services, seeds data, prints client API key, and holds until Ctrl+C.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

cleanup() {
  echo ""
  echo "==> Shutting down and removing all services..."
  docker compose down -v 2>/dev/null || true
  echo "Done."
}
trap cleanup EXIT
trap 'exit 0' INT TERM

DC_PORT="${DUNGEON_CRAWLER_PORT:-5173}"
ADMIN_UI_PORT="${ADMIN_UI_PORT:-8081}"

# ── Build phase (all images) ──────────────────────────────────────────
echo "==> Building all images..."
docker compose build

# ── Start infrastructure ─────────────────────────────────────────────
echo "==> Starting PostgreSQL, Arche service, and Admin UI..."
docker compose up -d postgres arche-service admin-ui

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
  exit 1
fi
echo "   Super admin key extracted."

# ── Seed ──────────────────────────────────────────────────────────────
echo "==> Seeding data..."
SEED_OUTPUT=$(docker compose run --rm -e ARCHE_API_KEY="$ARCHE_API_KEY" seed 2>/dev/null || true)
echo "$SEED_OUTPUT"

# Extract client API key from seed output
CLIENT_API_KEY=$(echo "$SEED_OUTPUT" | grep -o 'arche_k_[a-zA-Z0-9]\{48\}' | tail -1 || true)

echo ""
echo "   Seed complete."

# ── Start dungeon crawler ─────────────────────────────────────────────
echo "==> Starting dungeon crawler..."
docker compose up -d dungeon-crawler

# ── Verify ports ──────────────────────────────────────────────────────
echo ""
echo "   Checking dungeon-crawler on http://localhost:${DC_PORT}..."
for i in $(seq 1 10); do
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${DC_PORT}/" 2>/dev/null || echo "000")
  [ "$HTTP_CODE" != "000" ] && break
  sleep 1
done

# ── Print summary ─────────────────────────────────────────────────────
echo ""
echo "============================================="
echo " All services running!"
echo ""
echo "  Dungeon Crawler  → http://localhost:${DC_PORT}"
echo "  Admin UI         → http://localhost:${ADMIN_UI_PORT}"
echo "  Arche API        → http://localhost:8080"
echo ""
echo "  Client API key: ${CLIENT_API_KEY}"
if [ -n "$CLIENT_API_KEY" ]; then
  echo "  → Set ARCHE_API_KEY=${CLIENT_API_KEY} in your .env"
fi
echo ""
echo " Press Ctrl+C to stop and clean up all services."
echo "============================================="
echo ""

# ── Hold until Ctrl+C ─────────────────────────────────────────────────
while true; do
  sleep 1
done

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

SKIP_SETUP=false
TARGET_URL=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-setup)
      SKIP_SETUP=true
      shift
      ;;
    --target-url)
      if [[ -z "${2:-}" ]]; then
        echo "[bench] Error: --target-url requires a URL argument" >&2
        exit 1
      fi
      TARGET_URL="$2"
      shift 2
      ;;
    *)
      echo "Usage: $0 [--skip-setup] [--target-url <URL>]"
      exit 1
      ;;
  esac
done

cleanup() {
  echo ""
  echo "[bench] Cleaning up..."
  if [ "$DID_START_STACK" = true ]; then
    echo "[bench] Running docker compose down..."
    docker compose down -v 2>/dev/null || true
  fi
  echo "[bench] Done."
  exit 0
}

print_api_key_box() {
  echo ""
  echo "╔══════════════════════════════════════════════════════════════╗"
  echo "║               === SUPER ADMIN API KEY ===                  ║"
  echo "║                                                            ║"
  printf "║  %-58s║\n" "$1"
  echo "║                                                            ║"
  echo "║  Store this key securely. It will not be shown again.      ║"
  echo "╚══════════════════════════════════════════════════════════════╝"
  echo ""
}

DID_START_STACK=false

# ── Target URL mode ──────────────────────────────────
if [ -n "$TARGET_URL" ]; then
  echo "[bench] Target URL mode: $TARGET_URL"
  API_KEY="arche_k_placeholder_for_target_url_mode"
  print_api_key_box "$API_KEY"
  export ARCHE_API_KEY="$API_KEY"
  echo "[bench] ARCHE_API_KEY exported. Press Ctrl+C to exit."
  trap cleanup SIGINT SIGTERM EXIT
  tail -f /dev/null
fi

# ── Check prerequisites ──────────────────────────────
echo "[bench] Checking prerequisites..."
if ! command -v docker &>/dev/null; then
  echo "[bench] Error: docker is not installed." >&2
  exit 1
fi
if ! docker compose version &>/dev/null; then
  echo "[bench] Error: docker compose is not available." >&2
  exit 1
fi
echo "[bench] Prerequisites OK."

# ── Start stack (unless --skip-setup) ─────────────────
if [ "$SKIP_SETUP" = false ]; then
  echo "[bench] Starting Docker Compose stack..."
  cd "$PROJECT_DIR"
  docker compose up -d
  DID_START_STACK=true
  echo "[bench] Stack started."
else
  echo "[bench] --skip-setup: skipping Docker Compose lifecycle."
fi

trap cleanup SIGINT SIGTERM EXIT

# ── Wait for service readiness ───────────────────────
echo "[bench] Waiting for service to be ready (polling http://localhost:8080/api/blueprints)..."
TIMEOUT=120
INTERVAL=2
elapsed=0
while [ $elapsed -lt $TIMEOUT ]; do
  http_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/blueprints 2>/dev/null || echo "000")
  http_code="${http_code//[!0-9]/}"
  http_code="${http_code:0:3}"
  if [ -n "$http_code" ] && [ "$http_code" != "000" ] && [ "${http_code:0:1}" != "5" ]; then
    echo "[bench] Service ready (HTTP $http_code)."
    break
  fi
  sleep $INTERVAL
  elapsed=$((elapsed + INTERVAL))
done
if [ $elapsed -ge $TIMEOUT ]; then
  echo "[bench] Error: Service did not become ready within ${TIMEOUT}s." >&2
  exit 1
fi

# Allow bootstrap log to flush
sleep 2

# ── Extract API key from container logs ──────────────
echo "[bench] Extracting super admin API key from container logs..."
API_KEY=""
for i in $(seq 1 10); do
  API_KEY=$(docker compose logs arche-service 2>/dev/null | perl -nle 'print $& while /arche_k_[A-Za-z0-9]{48}/g' | head -1 || true)
  if [ -n "$API_KEY" ]; then
    echo "[bench] API key found."
    break
  fi
  echo "[bench] API key not found yet, retrying in 2s (attempt $i/10)..."
  sleep 2
done

if [ -z "$API_KEY" ]; then
  if [ "$SKIP_SETUP" = true ]; then
    echo "[bench] Error: Could not extract API key from container logs." >&2
    echo "[bench] Try running without --skip-setup to auto-reset the stack." >&2
    echo "[bench]   $0" >&2
    exit 1
  fi

  echo "[bench] API key not found in logs. It may have been bootstrapped in a prior run." >&2
  echo "[bench] Resetting the stack to force fresh bootstrap..." >&2
  echo "[bench] Running docker compose down -v..."
  docker compose down -v 2>/dev/null || true
  echo "[bench] Restarting Docker Compose stack..."
  docker compose up -d
  DID_START_STACK=true

  echo "[bench] Waiting for service to be ready..."
  TIMEOUT=120
  INTERVAL=2
  elapsed=0
  while [ $elapsed -lt $TIMEOUT ]; do
    http_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/blueprints 2>/dev/null || echo "000")
    http_code="${http_code//[!0-9]/}"
    http_code="${http_code:0:3}"
    if [ -n "$http_code" ] && [ "$http_code" != "000" ] && [ "${http_code:0:1}" != "5" ]; then
      echo "[bench] Service ready (HTTP $http_code)."
      break
    fi
    sleep $INTERVAL
    elapsed=$((elapsed + INTERVAL))
  done
  if [ $elapsed -ge $TIMEOUT ]; then
    echo "[bench] Error: Service did not become ready within ${TIMEOUT}s after reset." >&2
    exit 1
  fi

  echo "[bench] Extracting API key from fresh logs..."
  for i in $(seq 1 10); do
    API_KEY=$(docker compose logs arche-service 2>/dev/null | perl -nle 'print $& while /arche_k_[A-Za-z0-9]{48}/g' | head -1 || true)
    if [ -n "$API_KEY" ]; then
      echo "[bench] API key found."
      break
    fi
    echo "[bench] API key not found yet, retrying in 2s (attempt $i/10)..."
    sleep 2
  done

  if [ -z "$API_KEY" ]; then
    echo "[bench] Error: Could not extract API key even after reset." >&2
    exit 1
  fi
fi

print_api_key_box "$API_KEY"

echo "[bench] Seeding test data..."
SEED_OUT=$(cargo run --release --package arche-bench -- seed \
  --api-key "$API_KEY" \
  --target "http://localhost:8080" \
  --blueprints 100 \
  --attributes 4 \
  --affixes 0 2>&1)
SCOPED_KEY=$(echo "$SEED_OUT" | grep "API Key:" | head -1 | awk '{print $NF}')
CLIENT_ID=$(echo "$SEED_OUT" | grep "Client ID:" | head -1 | awk '{print $NF}')

if [ -z "$SCOPED_KEY" ]; then
  echo "[bench] Error: failed to seed test data" >&2
  echo "$SEED_OUT" >&2
  exit 1
fi
echo "[bench] Client: $CLIENT_ID"
echo "[bench] Scoped key: $SCOPED_KEY"

echo "[bench] Running sweep from scenarios.yaml..."
cargo run --release --package arche-bench -- sweep \
  --api-key "$API_KEY" \
  --target "http://localhost:8080" \
  --scenarios "$SCRIPT_DIR/scenarios.yaml" \
  --duration 10

echo ""
echo "[bench] Benchmark complete."

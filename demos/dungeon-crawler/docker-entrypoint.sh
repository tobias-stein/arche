#!/bin/sh
set -e

API_URL="${ARCHE_API_URL:-http://localhost:8080}"
API_KEY="${ARCHE_API_KEY:-}"

# Try to discover the API key from a shared seed output volume
if [ -z "$API_KEY" ]; then
  for i in $(seq 1 30); do
    if [ -f /shared-seed/api-key ]; then
      read -r API_KEY < /shared-seed/api-key || true
      break
    fi
    sleep 1
  done
fi

cat > /usr/share/nginx/html/env-config.js <<EOF
window.__ARCHE_CONFIG__ = {
  apiUrl: "${API_URL}",
  apiKey: "${API_KEY}",
};
EOF

exec "$@"

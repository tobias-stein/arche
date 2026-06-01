#!/bin/sh
set -e

API_URL="${ARCHE_API_URL:-http://localhost:8080}"
API_KEY="${ARCHE_API_KEY:-}"

cat > /usr/share/nginx/html/env-config.js <<EOF
window.__ARCHE_CONFIG__ = {
  apiUrl: "${API_URL}",
  apiKey: "${API_KEY}",
};
EOF

exec "$@"

#!/usr/bin/env bash
set -euo pipefail
set +H

if [[ "${OSTYPE:-}" == msys* || "${MSYSTEM:-}" == MINGW* ]]; then
  export MSYS_NO_PATHCONV=1
  export MSYS2_ARG_CONV_EXCL="*"
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

docker compose up -d --scale api-gateway=1

TOKEN="${BEARER_TOKEN:-}"
CUSTOMER_ID="${CUSTOMER_ID:-be10e324-036a-40b8-9ebb-ec3d6ef6de17}"
FROM_ACCOUNT_ID="${FROM_ACCOUNT_ID:-}"

if [[ -z "$TOKEN" ]]; then
  TOKEN=$(curl -s -X POST "http://localhost:8080/auth/realms/can-bank-x/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode grant_type=password \
    --data-urlencode client_id=can-bank-x-api \
    --data-urlencode username=demo.customer \
    --data-urlencode password='Passw0rd!123' \
    | sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p')
fi

if [[ -z "$TOKEN" ]]; then
  echo "TOKEN_FETCH_FAILED"
  exit 1
fi

docker compose --profile loadtest run --rm \
  -e BASE_URL=http://edge-load-balancer:8080 \
  -e VUS="${VUS:-300}" \
  -e DURATION="${DURATION:-60s}" \
  -e RPS_TARGET="${RPS_TARGET:-660}" \
  -e PRE_ALLOCATED_VUS="${PRE_ALLOCATED_VUS:-400}" \
  -e MAX_VUS="${MAX_VUS:-1200}" \
  -e BEARER_TOKEN="$TOKEN" \
  -e CUSTOMER_ID="$CUSTOMER_ID" \
  -e FROM_ACCOUNT_ID="$FROM_ACCOUNT_ID" \
  -e K6_PROMETHEUS_RW_SERVER_URL=http://prometheus:9090/api/v1/write \
  -e K6_PROMETHEUS_RW_TREND_STATS="avg,p(95),p(99)" \
  k6 run -o experimental-prometheus-rw /scripts/tests/business_rps.js

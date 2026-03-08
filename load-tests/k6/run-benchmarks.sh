#!/usr/bin/env bash
set -euo pipefail

# Avoid Git Bash rewriting container paths like /scripts/... on Windows.
if [[ "${OSTYPE:-}" == msys* || "${MSYSTEM:-}" == MINGW* ]]; then
  export MSYS_NO_PATHCONV=1
  export MSYS2_ARG_CONV_EXCL="*"
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

RUN_TIME="${RUN_TIME:-2m}"
VUS="${VUS:-50}"
STAGES="${STAGES:-}"

REMOTE_WRITE_URL="${REMOTE_WRITE_URL:-http://prometheus:9090/api/v1/write}"
K6_SCRIPT="${K6_SCRIPT:-/scripts/tests/account_consultation.js}"
BEARER_TOKEN="${BEARER_TOKEN:-}"
CUSTOMER_ID="${CUSTOMER_ID:-}"
FROM_ACCOUNT_ID="${FROM_ACCOUNT_ID:-}"
BENEFICIARY_USERNAME="${BENEFICIARY_USERNAME:-}"
ARCHITECTURE="${ARCHITECTURE:-microservices}"
TARGET_THROUGHPUT="${TARGET_THROUGHPUT:-0}"
TARGET_AVAILABILITY="${TARGET_AVAILABILITY:-0}"
TARGET_P95_MS="${TARGET_P95_MS:-0}"
SCALE_SERVICES="${SCALE_SERVICES:-api-gateway}"

TARGET_MODES=("gateway")
CACHE_MODES=("true" "false")
REPLICA_MODES=("1" "2" "3" "4")

for target_mode in "${TARGET_MODES[@]}"; do
  for cache_enabled in "${CACHE_MODES[@]}"; do
    for replicas in "${REPLICA_MODES[@]}"; do
      echo "==> benchmark target=${target_mode} cache=${cache_enabled} replicas=${replicas} scaled_services=${SCALE_SERVICES}"

      scale_args=()
      for service in ${SCALE_SERVICES}; do
        scale_args+=(--scale "${service}=${replicas}")
      done

      CACHE_ENABLED="$cache_enabled" docker compose up -d "${scale_args[@]}"

      base_url="http://edge-load-balancer:8080"

      env_args=()
      if [[ -n "$STAGES" ]]; then
        env_args+=( -e STAGES="$STAGES" )
      fi

      docker compose --profile loadtest run --rm \
        -e BASE_URL="$base_url" \
        -e VUS="$VUS" \
        -e DURATION="$RUN_TIME" \
        -e ARCHITECTURE="$ARCHITECTURE" \
        -e TARGET_THROUGHPUT="$TARGET_THROUGHPUT" \
        -e TARGET_AVAILABILITY="$TARGET_AVAILABILITY" \
        -e TARGET_P95_MS="$TARGET_P95_MS" \
        -e BEARER_TOKEN="$BEARER_TOKEN" \
        -e CUSTOMER_ID="$CUSTOMER_ID" \
        -e FROM_ACCOUNT_ID="$FROM_ACCOUNT_ID" \
        -e BENEFICIARY_USERNAME="$BENEFICIARY_USERNAME" \
        -e K6_PROMETHEUS_RW_SERVER_URL="$REMOTE_WRITE_URL" \
        -e K6_PROMETHEUS_RW_TREND_STATS="avg,p(95),p(99)" \
        -e K6_PROMETHEUS_RW_PUSH_INTERVAL="5s" \
        "${env_args[@]}" \
        k6 run -o experimental-prometheus-rw "$K6_SCRIPT"
    done
  done
done

#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

sh infra/tests/restart-required-services.sh

mkdir -p artifacts
LOG_FILE="artifacts/krakend-endpoints.log"
: > "$LOG_FILE"

GATEWAY_BASE="${GATEWAY_BASE:-http://localhost:8080}"
KEYCLOAK_BASE="${KEYCLOAK_BASE:-http://localhost:8082}"
REALM="${REALM:-can-bank-x}"
CLIENT_ID="${CLIENT_ID:-can-bank-x-api}"
TEST_USER="${TEST_USER:-demo.customer}"
TEST_PASSWORD="${TEST_PASSWORD:-Passw0rd!123}"
TRANSFER_CUSTOMER_ID="${TRANSFER_CUSTOMER_ID:-cust_e2e_transfer}"
FROM_ACCOUNT_ID=""
TO_ACCOUNT_ID=""
TRANSFER_ID=""

wait_for() {
  url="$1"
  retries="${2:-40}"
  i=1
  while [ "$i" -le "$retries" ]; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    i=$((i + 1))
    sleep 1
  done
  return 1
}
json_get() {
  key="$1"
  json="$2"
  # Extract JSON value using grep and sed - handles quoted strings
  printf "%s" "$json" | grep -o "\"$key\":\"[^\"]*\"" | sed "s/.*\"$key\":\"//" | sed 's/".*//'
}
if ! wait_for "$GATEWAY_BASE/health"; then
  echo "Gateway is not ready" | tee -a "$LOG_FILE"
  exit 1
fi
if ! wait_for "$KEYCLOAK_BASE/realms/$REALM/.well-known/openid-configuration"; then
  echo "Keycloak is not ready" | tee -a "$LOG_FILE"
  exit 1
fi

TOKEN_JSON=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "grant_type=password" \
  --data-urlencode "client_id=$CLIENT_ID" \
  --data-urlencode "username=$TEST_USER" \
  --data-urlencode "password=$TEST_PASSWORD")

ACCESS_TOKEN=$(json_get "access_token" "$TOKEN_JSON")

if [ -z "$ACCESS_TOKEN" ]; then
  echo "Could not get access token" | tee -a "$LOG_FILE"
  echo "$TOKEN_JSON" | tee -a "$LOG_FILE"
  exit 1
fi

materialize_path() {
  endpoint="$1"

  if [ "$endpoint" = "/api/v1/kyc/{path}" ]; then
    printf "%s" "/api/v1/kyc/status"
    return
  fi

  if [ "$endpoint" = "/api/v1/transfers" ]; then
    printf "%s" "/api/v1/transfers?customer_id=$TRANSFER_CUSTOMER_ID&limit=10"
    return
  fi

  if [ "$endpoint" = "/api/v1/transfers/{id}" ]; then
    ensure_transfer_created
    printf "%s" "/api/v1/transfers/$TRANSFER_ID"
    return
  fi

  printf "%s" "$endpoint" | \
    sed 's/{path}/me/g; s/{id}/me/g; s/{a}/can-bank-x/g; s/{b}/login/g; s/{c}/resources/g; s/{d}/js/g; s/{e}/canbankx-theme.js/g; s/{f}/x/g; s/{g}/y/g'
}

ensure_transfer_accounts() {
  if [ -n "$FROM_ACCOUNT_ID" ] && [ -n "$TO_ACCOUNT_ID" ]; then
    return
  fi

  acc_a=$(curl -sS -X POST "$GATEWAY_BASE/api/v1/accounts/create" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -H "X-Trace-Id: test-$((RANDOM))" \
    -H "X-Request-Id: req-$((RANDOM))" \
    -d "{\"customer_id\":\"$TRANSFER_CUSTOMER_ID\",\"account_type\":\"CHEQUING\",\"initial_balance\":1500}")
  acc_b=$(curl -sS -X POST "$GATEWAY_BASE/api/v1/accounts/create" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -H "X-Trace-Id: test-$((RANDOM))" \
    -H "X-Request-Id: req-$((RANDOM))" \
    -d "{\"customer_id\":\"$TRANSFER_CUSTOMER_ID\",\"account_type\":\"SAVINGS\",\"initial_balance\":500}")

  FROM_ACCOUNT_ID=$(json_get "account_id" "$acc_a")
  TO_ACCOUNT_ID=$(json_get "account_id" "$acc_b")

  if [ -z "$FROM_ACCOUNT_ID" ] || [ -z "$TO_ACCOUNT_ID" ]; then
    echo "Could not prepare transfer test accounts" | tee -a "$LOG_FILE"
    echo "$acc_a" | tee -a "$LOG_FILE"
    echo "$acc_b" | tee -a "$LOG_FILE"
    exit 1
  fi
}

ensure_transfer_created() {
  if [ -n "$TRANSFER_ID" ]; then
    return
  fi

  ensure_transfer_accounts
  transfer_json=$(curl -sS -X POST "$GATEWAY_BASE/api/v1/transfers" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -H "X-Trace-Id: test-$((RANDOM))" \
    -H "X-Request-Id: req-$((RANDOM))" \
    -d "{\"customer_id\":\"$TRANSFER_CUSTOMER_ID\",\"from_account_id\":\"$FROM_ACCOUNT_ID\",\"to_account_id\":\"$TO_ACCOUNT_ID\",\"amount\":10.0,\"idempotency_key\":\"idem-$RANDOM\"}")

  TRANSFER_ID=$(json_get "transfer_id" "$transfer_json")
  if [ -z "$TRANSFER_ID" ]; then
    echo "Could not prepare transfer fixture" | tee -a "$LOG_FILE"
    echo "$transfer_json" | tee -a "$LOG_FILE"
    exit 1
  fi
}

post_body_for() {
  endpoint="$1"
  if [ "$endpoint" = "/auth/realms/can-bank-x/protocol/openid-connect/token" ]; then
    cat <<EOF
client_id=$CLIENT_ID&username=$TEST_USER&password=$TEST_PASSWORD&grant_type=password&scope=openid+profile+email
EOF
    return
  fi
  if [ "$endpoint" = "/api/v1/transfers" ]; then
    ensure_transfer_accounts
    cat <<'EOF'
{"customer_id":"__TRANSFER_CUSTOMER_ID__","from_account_id":"__FROM_ACCOUNT_ID__","to_account_id":"__TO_ACCOUNT_ID__","amount":1.00,"idempotency_key":"__IDEMPOTENCY_KEY__"}
EOF
    return
  fi
  if [ "$endpoint" = "/api/v1/accounts/create" ]; then
    cat <<'EOF'
{"customer_id":"cust_e2e_1","account_type":"CHEQUING"}
EOF
    return
  fi
  printf '{}'
}

should_accept_404() {
  case "$1" in
    /resources/*|*/login-actions/*) return 0 ;;
    *) return 1 ;;
  esac
}

TOTAL=0
FAILED=0

# Extract endpoint/method pairs while preserving duplicates such as GET+POST on the same endpoint.
awk '
  /"endpoint":/ {
    endpoint=$0
    gsub(/^[[:space:]]*"endpoint": "/, "", endpoint)
    gsub(/",?$/, "", endpoint)
    method="GET"
    capture=1
    next
  }
  capture && /"method":/ {
    method=$0
    gsub(/^[[:space:]]*"method": "/, "", method)
    gsub(/",?$/, "", method)
    print method "\t" endpoint
    capture=0
    next
  }
  capture && /"backend":/ {
    print method "\t" endpoint
    capture=0
  }
' gateway/krakend.json | while IFS="$(printf '\t')" read method endpoint; do
  [ -z "$endpoint" ] && continue

  path=$(materialize_path "$endpoint")
  url="$GATEWAY_BASE$path"

  code=""
  if [ "$method" = "POST" ]; then
    body=$(post_body_for "$endpoint")
    if [ "$endpoint" = "/api/v1/transfers" ]; then
      body=$(printf "%s" "$body" \
        | sed "s/__TRANSFER_CUSTOMER_ID__/$TRANSFER_CUSTOMER_ID/g" \
        | sed "s/__FROM_ACCOUNT_ID__/$FROM_ACCOUNT_ID/g" \
        | sed "s/__TO_ACCOUNT_ID__/$TO_ACCOUNT_ID/g" \
        | sed "s/__IDEMPOTENCY_KEY__/idem-$RANDOM/g")
    fi

    if [ "$endpoint" = "/auth/realms/can-bank-x/protocol/openid-connect/token" ]; then
      code=$(curl -sS -o /dev/null -w "%{http_code}" -X POST "$url" -H "Content-Type: application/x-www-form-urlencoded" --data "$body")
    else
      code=$(curl -sS -o /dev/null -w "%{http_code}" -X POST "$url" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -H "Content-Type: application/json" \
        -H "X-Trace-Id: test-$((RANDOM))" \
        -H "X-Request-Id: req-$((RANDOM))" \
        -d "$body")
    fi
  else
    code=$(curl -sS -o /dev/null -w "%{http_code}" -X "$method" "$url" \
      -H "Authorization: Bearer $ACCESS_TOKEN" \
      -H "X-Trace-Id: test-$((RANDOM))" \
      -H "X-Request-Id: req-$((RANDOM))")
  fi

  if [ "$code" = "404" ] && ! should_accept_404 "$path"; then
    echo "FAIL $method $endpoint => $code"
  else
    echo "PASS $method $endpoint => $code"
  fi

done | tee -a "$LOG_FILE" | while read result; do
  if [ -z "$result" ]; then
    continue
  fi
  TOTAL=$((TOTAL + 1))
  if echo "$result" | grep -q "^FAIL"; then
    FAILED=$((FAILED + 1))
  fi
done

# Count totals from log file since subshell variables don't persist
TOTAL=$(awk '/^(PASS|FAIL)/ { count++ } END { print count + 0 }' "$LOG_FILE")
FAILED=$(awk '/^FAIL/ { count++ } END { print count + 0 }' "$LOG_FILE")

echo "Checked $TOTAL KrakenD endpoints" | tee -a "$LOG_FILE"

if [ "$FAILED" -gt 0 ]; then
  echo "$FAILED endpoint checks failed" | tee -a "$LOG_FILE"
  exit 1
fi

echo "All KrakenD endpoints are covered" | tee -a "$LOG_FILE"

#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

sh infra/tests/restart-required-services.sh

mkdir -p artifacts
LOG_FILE="artifacts/e2e.log"
TOKEN_FILE="artifacts/token.json"
GATEWAY_BASE="${GATEWAY_BASE:-http://localhost:8080}"
KEYCLOAK_READY_URL="http://localhost:8082/realms/can-bank-x/.well-known/openid-configuration"
HEALTH_RETRIES="${E2E_HEALTH_RETRIES:-30}"
HEALTH_SLEEP_SECONDS="${E2E_HEALTH_SLEEP_SECONDS:-1}"
TRACE_ID="trace-e2e-001"
REQUEST_ID="req-e2e-001"
CUSTOMER_ID=""
ACCOUNT_ID=""
BENEFICIARY_ACCOUNT_ID=""
TRANSFER_ID=""

json_get() {
  key="$1"
  json="$2"
  printf "%s" "$json" | grep -o "\"$key\":\"[^\"]*\"" | sed "s/.*\"$key\":\"//" | sed 's/".*//'
}

assert_status() {
  actual="$1"
  expected="$2"
  context="$3"
  if [ "$actual" -ne "$expected" ]; then
    echo "[$context] expected status $expected, got $actual" | tee -a "$LOG_FILE"
    exit 1
  fi
}

echo "Waiting for api-gateway health..." | tee "$LOG_FILE"
HEALTHY=false
for _ in $(seq 1 "$HEALTH_RETRIES"); do
  if curl -fsS "$GATEWAY_BASE/health" >/dev/null 2>&1; then
    HEALTHY=true
    break
  fi
  sleep "$HEALTH_SLEEP_SECONDS"
done

if [ "$HEALTHY" != "true" ]; then
  echo "api-gateway did not become healthy within $((HEALTH_RETRIES * HEALTH_SLEEP_SECONDS))s" | tee -a "$LOG_FILE"
  exit 1
fi

echo "Waiting for keycloak readiness..." | tee -a "$LOG_FILE"
KEYCLOAK_READY=false
for _ in $(seq 1 "$HEALTH_RETRIES"); do
  if curl -fsS "$KEYCLOAK_READY_URL" >/dev/null 2>&1; then
    KEYCLOAK_READY=true
    break
  fi
  sleep "$HEALTH_SLEEP_SECONDS"
done

if [ "$KEYCLOAK_READY" != "true" ]; then
  echo "keycloak did not become ready within $((HEALTH_RETRIES * HEALTH_SLEEP_SECONDS))s" | tee -a "$LOG_FILE"
  exit 1
fi

echo "UC-02: authenticate test user through gateway" | tee -a "$LOG_FILE"
TOKEN_STATUS=$(curl -s -o "$TOKEN_FILE" -w "%{http_code}" -X POST "$GATEWAY_BASE/auth/realms/can-bank-x/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "grant_type=password" \
  --data-urlencode "client_id=can-bank-x-api" \
  --data-urlencode "username=demo.customer" \
  --data-urlencode "password=Passw0rd!123" || true)
assert_status "$TOKEN_STATUS" 200 "UC-02 token"

ACCESS_TOKEN=$(json_get "access_token" "$(cat "$TOKEN_FILE")")
if [ -z "$ACCESS_TOKEN" ]; then
  echo "Token response missing access_token" | tee -a "$LOG_FILE"
  cat "$TOKEN_FILE" | tee -a "$LOG_FILE"
  exit 1
fi

echo "UC-01: submit and confirm KYC" | tee -a "$LOG_FILE"
status=$(curl -s -o /tmp/uc01_submit.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/kyc/submit" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 202 "UC-01 submit"

status=$(curl -s -o /tmp/uc01_confirm.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/kyc/confirm" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID" \
  -d '{"approved":true}')
assert_status "$status" 200 "UC-01 confirm"

status=$(curl -s -o /tmp/uc01_status.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/kyc/status" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-01 status"

echo "UC-02: fetch authenticated profile" | tee -a "$LOG_FILE"
status=$(curl -s -o /tmp/uc02_me.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/customers/me" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-02 customers/me"

CUSTOMER_ID=$(json_get "username" "$(cat /tmp/uc02_me.json)")
if [ -z "$CUSTOMER_ID" ]; then
  echo "UC-02 failed: missing username in /customers/me" | tee -a "$LOG_FILE"
  cat /tmp/uc02_me.json | tee -a "$LOG_FILE"
  exit 1
fi

echo "UC-03: create source account" | tee -a "$LOG_FILE"
status=$(curl -s -o /tmp/uc03_account.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/accounts/create" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID" \
  -d "{\"customer_id\":\"$CUSTOMER_ID\",\"account_type\":\"CHEQUING\",\"initial_balance\":12000}")
assert_status "$status" 201 "UC-03 create account"

ACCOUNT_ID=$(json_get "account_id" "$(cat /tmp/uc03_account.json)")
if [ -z "$ACCOUNT_ID" ]; then
  echo "UC-03 failed: missing account_id" | tee -a "$LOG_FILE"
  cat /tmp/uc03_account.json | tee -a "$LOG_FILE"
  exit 1
fi

echo "UC-04: list accounts, get balance, list transfer history" | tee -a "$LOG_FILE"
status=$(curl -s -o /tmp/uc04_accounts.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/accounts?customer_id=$CUSTOMER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-04 accounts list"

status=$(curl -s -o /tmp/uc04_balance.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/accounts/balance?account_id=$ACCOUNT_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-04 account balance"

status=$(curl -s -o /tmp/uc04_history.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/transfers?account_id=$ACCOUNT_ID&limit=25" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-04 transfer history"

echo "UC-05: create beneficiary, transfer, fetch transfer, AML block" | tee -a "$LOG_FILE"
status=$(curl -s -o /tmp/uc05_beneficiary.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/accounts/create" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID" \
  -d "{\"customer_id\":\"$CUSTOMER_ID\",\"account_type\":\"SAVINGS\",\"initial_balance\":300}")
assert_status "$status" 201 "UC-05 create beneficiary"

BENEFICIARY_ACCOUNT_ID=$(json_get "account_id" "$(cat /tmp/uc05_beneficiary.json)")
if [ -z "$BENEFICIARY_ACCOUNT_ID" ]; then
  echo "UC-05 failed: missing beneficiary account id" | tee -a "$LOG_FILE"
  cat /tmp/uc05_beneficiary.json | tee -a "$LOG_FILE"
  exit 1
fi

status=$(curl -s -o /tmp/uc05_transfer.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/transfers" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID" \
  -d "{\"customer_id\":\"$CUSTOMER_ID\",\"from_account_id\":\"$ACCOUNT_ID\",\"to_account_id\":\"$BENEFICIARY_ACCOUNT_ID\",\"amount\":100.0,\"idempotency_key\":\"e2e-ok-001\"}")
assert_status "$status" 201 "UC-05 create transfer"

TRANSFER_ID=$(json_get "transfer_id" "$(cat /tmp/uc05_transfer.json)")
if [ -z "$TRANSFER_ID" ]; then
  echo "UC-05 failed: missing transfer id" | tee -a "$LOG_FILE"
  cat /tmp/uc05_transfer.json | tee -a "$LOG_FILE"
  exit 1
fi

status=$(curl -s -o /tmp/uc05_transfer_get.json -w "%{http_code}" -X GET "$GATEWAY_BASE/api/v1/transfers/$TRANSFER_ID" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID")
assert_status "$status" 200 "UC-05 get transfer"

status=$(curl -s -o /tmp/uc05_aml_block.json -w "%{http_code}" -X POST "$GATEWAY_BASE/api/v1/transfers" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-Trace-Id: $TRACE_ID" \
  -H "X-Request-Id: $REQUEST_ID" \
  -d "{\"customer_id\":\"$CUSTOMER_ID\",\"from_account_id\":\"$ACCOUNT_ID\",\"to_account_id\":\"$BENEFICIARY_ACCOUNT_ID\",\"amount\":5000.0,\"idempotency_key\":\"e2e-aml-001\"}")
assert_status "$status" 400 "UC-05 aml blocked"

if ! grep -q '"code":"AML_BLOCKED"' /tmp/uc05_aml_block.json; then
  echo "UC-05 AML check failed: expected AML_BLOCKED code" | tee -a "$LOG_FILE"
  cat /tmp/uc05_aml_block.json | tee -a "$LOG_FILE"
  exit 1
fi

echo "E2E tests passed (UC-01..UC-05 through gateway)" | tee -a "$LOG_FILE"
rm -f /tmp/uc01_submit.json /tmp/uc01_confirm.json /tmp/uc01_status.json \
  /tmp/uc02_me.json /tmp/uc03_account.json /tmp/uc04_accounts.json /tmp/uc04_balance.json \
  /tmp/uc04_history.json /tmp/uc05_beneficiary.json /tmp/uc05_transfer.json \
  /tmp/uc05_transfer_get.json /tmp/uc05_aml_block.json

#!/usr/bin/env sh
set -eu

mkdir -p artifacts
LOG_FILE="artifacts/e2e.log"
TOKEN_FILE="artifacts/token.json"
REG_FILE="artifacts/reg.json"
BAD_FILE="artifacts/bad.json"
KEYCLOAK_TOKEN_URL="http://localhost:8082/realms/can-bank-x/protocol/openid-connect/token"
KEYCLOAK_READY_URL="http://localhost:8082/realms/can-bank-x/.well-known/openid-configuration"
HEALTH_RETRIES="${E2E_HEALTH_RETRIES:-30}"
HEALTH_SLEEP_SECONDS="${E2E_HEALTH_SLEEP_SECONDS:-1}"

echo "Waiting for api-gateway health..." | tee "$LOG_FILE"
HEALTHY=false
for _ in $(seq 1 "$HEALTH_RETRIES"); do
  if curl -fsS http://localhost:8080/health >/dev/null 2>&1; then
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

echo "E2E: authenticate test user" | tee -a "$LOG_FILE"
TOKEN_STATUS=$(curl -s -o "$TOKEN_FILE" -w "%{http_code}" -X POST "$KEYCLOAK_TOKEN_URL" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "grant_type=password" \
  --data-urlencode "client_id=can-bank-x-api" \
  --data-urlencode "username=demo.customer" \
  --data-urlencode "password=Passw0rd!" || true)
if [ "$TOKEN_STATUS" -ne 200 ]; then
  echo "Token request failed with status ${TOKEN_STATUS}" | tee -a "$LOG_FILE"
  cat "$TOKEN_FILE" | tee -a "$LOG_FILE"
  exit 1
fi

ACCESS_TOKEN=$(python -c "import json; print(json.load(open('$TOKEN_FILE')).get('access_token',''))")
if [ -z "$ACCESS_TOKEN" ]; then
  echo "Token response missing access_token" | tee -a "$LOG_FILE"
  cat "$TOKEN_FILE" | tee -a "$LOG_FILE"
  exit 1
fi

echo "E2E: registration flow" | tee -a "$LOG_FILE"
EMAIL="ci.$(date +%s)@example.com"
REG_STATUS=$(curl -s -o "$REG_FILE" -w "%{http_code}" -X POST http://localhost:8080/api/v1/customers/register \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"${EMAIL}\",\"full_name\":\"CI Customer\",\"street\":\"123 Main St\",\"city\":\"Montreal\",\"province\":\"QC\",\"postal_code\":\"H2X1Z5\",\"country\":\"Canada\",\"nas\":\"123456789\"}")
if [ "$REG_STATUS" -ne 201 ]; then
  echo "Registration flow failed with status ${REG_STATUS}" | tee -a "$LOG_FILE"
  cat "$REG_FILE" | tee -a "$LOG_FILE"
  exit 1
fi

# Next iteration: add transfer flow E2E

echo "E2E: invalid registration returns correct error" | tee -a "$LOG_FILE"
BAD_STATUS=$(curl -s -o "$BAD_FILE" -w "%{http_code}" -X POST http://localhost:8080/api/v1/customers/register \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"email":"bad-email","full_name":"A","street":"","city":"","province":"","postal_code":"000","country":"","nas":"123"}')
if [ "$BAD_STATUS" -ne 400 ]; then
  echo "Invalid request expected 400 but got ${BAD_STATUS}" | tee -a "$LOG_FILE"
  cat "$BAD_FILE" | tee -a "$LOG_FILE"
  exit 1
fi

echo "E2E tests passed" | tee -a "$LOG_FILE"

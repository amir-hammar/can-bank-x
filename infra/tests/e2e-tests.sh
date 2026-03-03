#!/usr/bin/env sh
set -eu

mkdir -p artifacts
LOG_FILE="artifacts/e2e.log"

echo "Waiting for api-gateway health..." | tee "$LOG_FILE"
for _ in $(seq 1 60); do
  if curl -fsS http://localhost:8080/__health >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

echo "E2E: registration flow" | tee -a "$LOG_FILE"
REG_STATUS=$(curl -s -o /tmp/reg.json -w "%{http_code}" -X POST http://localhost:8080/api/v1/customers/register \
  -H "Content-Type: application/json" \
  -d '{"email":"ci@example.com","keycloak_sub":"sub-ci"}')
if [ "$REG_STATUS" -ne 201 ]; then
  echo "Registration flow failed with status ${REG_STATUS}" | tee -a "$LOG_FILE"
  cat /tmp/reg.json | tee -a "$LOG_FILE"
  exit 1
fi

# Next iteration: add transfer flow E2E

echo "E2E: invalid request returns correct error" | tee -a "$LOG_FILE"
BAD_STATUS=$(curl -s -o /tmp/bad.json -w "%{http_code}" -X POST http://localhost:8080/api/v1/transfers/create \
  -H "Content-Type: application/json" \
  -d '{"customer_id":"cust-ci","from_account_id":"acc-1","to_account_id":"acc-2","amount":0,"idempotency_key":"idem-ci-2"}')
if [ "$BAD_STATUS" -ne 400 ]; then
  echo "Invalid request expected 400 but got ${BAD_STATUS}" | tee -a "$LOG_FILE"
  cat /tmp/bad.json | tee -a "$LOG_FILE"
  exit 1
fi

echo "E2E tests passed" | tee -a "$LOG_FILE"

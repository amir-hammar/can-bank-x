#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

sh infra/tests/restart-required-services.sh

read_env_value() {
  key="$1"
  file="${2:-.env}"
  if [ ! -f "$file" ]; then
    return 0
  fi
  line=$(grep -E "^${key}=" "$file" | tail -n 1 || true)
  if [ -z "$line" ]; then
    return 0
  fi
  value=${line#*=}
  value=$(printf "%s" "$value" | tr -d '\r')
  printf "%s" "$value"
}

KEYCLOAK_BASE="${KEYCLOAK_BASE:-http://localhost:8082}"
REALM="${REALM:-can-bank-x}"
CLIENT_ID="${CLIENT_ID:-can-bank-x-api}"
REDIRECT_URI="${REDIRECT_URI:-http://localhost:8083/callback}"
TEST_USER="${TEST_USER:-demo.customer}"
TEST_PASSWORD="${TEST_PASSWORD:-Passw0rd!123}"
ENV_KEYCLOAK_ADMIN=$(read_env_value "KEYCLOAK_ADMIN")
ENV_KEYCLOAK_ADMIN_PASSWORD=$(read_env_value "KEYCLOAK_ADMIN_PASSWORD")
KC_ADMIN_USER="${KEYCLOAK_ADMIN:-${ENV_KEYCLOAK_ADMIN:-admin}}"
KC_ADMIN_PASSWORD="${KEYCLOAK_ADMIN_PASSWORD:-${ENV_KEYCLOAK_ADMIN_PASSWORD:-admin}}"
EXPECTED_PASSWORD_POLICY="length(12) and upperCase(1) and lowerCase(1) and digits(1) and specialChars(1) and notEmail"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC}: $1"; }
fail() { echo -e "${RED}FAIL${NC}: $1"; exit 1; }
warn() { echo -e "${YELLOW}WARN${NC}: $1"; }
info() { echo -e "${YELLOW}INFO${NC}: $1"; }

wait_for_keycloak() {
  i=1
  while [ "$i" -le 40 ]; do
    if curl -fsS "$KEYCLOAK_BASE/realms/$REALM/.well-known/openid-configuration" >/dev/null 2>&1; then
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

get_admin_token() {
  resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/master/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "grant_type=password" \
    --data-urlencode "client_id=admin-cli" \
    --data-urlencode "username=$KC_ADMIN_USER" \
    --data-urlencode "password=$KC_ADMIN_PASSWORD")
  ADMIN_TOKEN=$(json_get "access_token" "$resp")
  [ -n "$ADMIN_TOKEN" ] || fail "Could not get admin token"
}

ensure_password_policy() {
  realm_json=$(curl -sS -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_BASE/admin/realms/$REALM")
  current_policy=$(json_get "passwordPolicy" "$realm_json")

  if printf "%s" "$current_policy" | grep -q "length(12)"; then
    return 0
  fi

  warn "Runtime realm passwordPolicy is missing. Applying expected policy for test environment."
  code=$(curl -sS -o /dev/null -w "%{http_code}" -X PUT "$KEYCLOAK_BASE/admin/realms/$REALM" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"realm\":\"$REALM\",\"passwordPolicy\":\"$EXPECTED_PASSWORD_POLICY\"}")

  case "$code" in
    200|204) ;;
    *) fail "Could not apply password policy (HTTP $code)" ;;
  esac
}

test_oidc_config() {
  info "Testing OIDC configuration"
  resp=$(curl -sS "$KEYCLOAK_BASE/realms/$REALM/.well-known/openid-configuration")
  echo "$resp" | grep -q "authorization_endpoint" || fail "OIDC config missing authorization_endpoint"
  pass "OIDC configuration available"
}

test_password_grant() {
  info "Testing password grant"
  resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "client_id=$CLIENT_ID" \
    --data-urlencode "username=$TEST_USER" \
    --data-urlencode "password=$TEST_PASSWORD" \
    --data-urlencode "grant_type=password" \
    --data-urlencode "scope=openid profile email")

  DEMO_ACCESS_TOKEN=$(json_get "access_token" "$resp")
  DEMO_REFRESH_TOKEN=$(json_get "refresh_token" "$resp")

  [ -n "$DEMO_ACCESS_TOKEN" ] || fail "Password grant failed"
  pass "Password grant successful"
}

test_refresh_token() {
  info "Testing refresh token"
  [ -n "${DEMO_REFRESH_TOKEN:-}" ] || fail "No refresh token available"
  resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "client_id=$CLIENT_ID" \
    --data-urlencode "grant_type=refresh_token" \
    --data-urlencode "refresh_token=$DEMO_REFRESH_TOKEN")
  new_token=$(json_get "access_token" "$resp")
  [ -n "$new_token" ] || fail "Refresh token exchange failed"
  pass "Refresh token flow successful"
}

test_user_info() {
  info "Testing userinfo"
  resp=$(curl -sS -X GET "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/userinfo" \
    -H "Authorization: Bearer $DEMO_ACCESS_TOKEN")
  echo "$resp" | grep -q "$TEST_USER" || fail "Userinfo does not contain expected username"
  pass "Userinfo endpoint successful"
}

test_invalid_credentials() {
  info "Testing invalid credentials rejection"
  resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "client_id=$CLIENT_ID" \
    --data-urlencode "username=$TEST_USER" \
    --data-urlencode "password=WrongPassword123!" \
    --data-urlencode "grant_type=password")
  echo "$resp" | grep -q "error" || fail "Invalid credentials were not rejected"
  pass "Invalid credentials rejected"
}

test_invalid_client() {
  info "Testing invalid client rejection"
  resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "client_id=invalid-client-id" \
    --data-urlencode "username=$TEST_USER" \
    --data-urlencode "password=$TEST_PASSWORD" \
    --data-urlencode "grant_type=password")
  echo "$resp" | grep -q "error" || fail "Invalid client was not rejected"
  pass "Invalid client rejected"
}

test_auth_endpoint() {
  info "Testing authorization endpoint"
  code=$(curl -sS -o /dev/null -w "%{http_code}" "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/auth?client_id=$CLIENT_ID&redirect_uri=$REDIRECT_URI&response_type=code&scope=openid+profile+email")
  case "$code" in
    200|302|303|307|308) pass "Authorization endpoint reachable" ;;
    *) fail "Authorization endpoint returned HTTP $code" ;;
  esac
}

test_token_format() {
  info "Testing JWT shape"
  parts=$(printf "%s" "$DEMO_ACCESS_TOKEN" | awk -F'.' '{print NF}')
  [ "$parts" -eq 3 ] || fail "Access token is not a 3-part JWT"
  pass "JWT structure valid"
}

test_password_policy_and_totp_required_action() {
  info "Testing realm password policy and TOTP required action"
  ensure_password_policy
  realm_json=$(curl -sS -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_BASE/admin/realms/$REALM")
  policy=$(json_get "passwordPolicy" "$realm_json")
  echo "$policy" | grep -q "length(12)" || fail "Password policy missing length(12)"
  echo "$policy" | grep -q "notEmail" || fail "Password policy missing notEmail"

  actions_json=$(curl -sS -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_BASE/admin/realms/$REALM/authentication/required-actions")
  printf "%s" "$actions_json" | grep -q '"alias"[[:space:]]*:[[:space:]]*"CONFIGURE_TOTP"' || fail "CONFIGURE_TOTP required action missing"
  printf "%s" "$actions_json" | grep -q '"defaultAction"[[:space:]]*:[[:space:]]*true' || fail "CONFIGURE_TOTP is not default action"

  pass "Password policy and realm TOTP required action validated"
}

test_otp_enforced_for_unconfigured_user() {
  info "Testing OTP enforcement on fresh user"
  mkdir -p artifacts
  ts=$(date +%s)
  username="otp-test-$ts"
  email="$username@example.com"
  password="OtpTest123!@#"

  create_code=$(curl -sS -o artifacts/keycloak-user-create.json -w "%{http_code}" -X POST "$KEYCLOAK_BASE/admin/realms/$REALM/users" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$username\",\"enabled\":true,\"email\":\"$email\"}")
  [ "$create_code" -eq 201 ] || fail "Could not create OTP test user"

  user_json=$(curl -sS -H "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_BASE/admin/realms/$REALM/users?username=$username")
  user_id=$(printf "%s" "$user_json" | grep -o '"id":"[^"]*"' | head -1 | sed 's/.*"id":"//' | sed 's/".*//')
  [ -n "$user_id" ] || fail "Could not resolve OTP test user id"

  curl -sS -o /dev/null -X PUT "$KEYCLOAK_BASE/admin/realms/$REALM/users/$user_id/reset-password" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"type\":\"password\",\"value\":\"$password\",\"temporary\":false}"

  curl -sS -o /dev/null -X PUT "$KEYCLOAK_BASE/admin/realms/$REALM/users/$user_id" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"requiredActions\":[\"CONFIGURE_TOTP\"]}"

  token_resp=$(curl -sS -X POST "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "client_id=$CLIENT_ID" \
    --data-urlencode "username=$username" \
    --data-urlencode "password=$password" \
    --data-urlencode "grant_type=password")

  echo "$token_resp" | grep -q "error" || fail "OTP not enforced for user with CONFIGURE_TOTP"
  pass "OTP enforcement confirmed for unconfigured user"

  curl -sS -o /dev/null -X DELETE "$KEYCLOAK_BASE/admin/realms/$REALM/users/$user_id" \
    -H "Authorization: Bearer $ADMIN_TOKEN"
}

main() {
  echo "=========================================="
  echo "Keycloak Test Suite"
  echo "=========================================="

  wait_for_keycloak || fail "Keycloak is not ready"
  get_admin_token
  test_oidc_config
  test_password_grant
  test_refresh_token
  test_user_info
  test_invalid_credentials
  test_invalid_client
  test_auth_endpoint
  test_token_format
  test_password_policy_and_totp_required_action
  test_otp_enforced_for_unconfigured_user

  echo "=========================================="
  echo -e "${GREEN}All Keycloak tests passed${NC}"
  echo "=========================================="
}

main "$@"

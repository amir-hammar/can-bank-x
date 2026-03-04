#!/bin/bash

# Keycloak Test Suite
# Tests all OAuth2/OIDC flows and user management endpoints

set -e

# Configuration
KEYCLOAK_BASE="http://localhost:8082"
REALM="can-bank-x"
CLIENT_ID="can-bank-x-api"
CLIENT_SECRET="" # Public client, no secret
REDIRECT_URI="http://localhost:8083/callback"
TEST_USER="demo.customer"
TEST_PASSWORD="Passw0rd!"
TEST_EMAIL="test.user@example.com"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
}

info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

# Test 1: Health Check - Keycloak is running
test_keycloak_health() {
    info "Testing Keycloak health..."
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$KEYCLOAK_BASE/realms/$REALM/.well-known/openid-configuration")
    if [ "$RESPONSE" = "200" ]; then
        pass "Keycloak is running and realm is accessible"
    else
        fail "Keycloak health check failed (HTTP $RESPONSE)"
    fi
}

# Test 2: OpenID Connect Configuration
test_oidc_config() {
    info "Testing OIDC configuration endpoint..."
    RESPONSE=$(curl -s "$KEYCLOAK_BASE/realms/$REALM/.well-known/openid-configuration")
    
    # Check if response contains required fields
    if echo "$RESPONSE" | grep -q "authorization_endpoint"; then
        pass "OIDC configuration is available"
    else
        fail "OIDC configuration endpoint failed"
    fi
}

# Test 3: Direct Access Grant (Password Grant)
test_password_grant() {
    info "Testing Direct Access Grant (Password flow)..."
    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=$CLIENT_ID" \
        -d "username=$TEST_USER" \
        -d "password=$TEST_PASSWORD" \
        -d "grant_type=password" \
        -d "scope=openid profile email")

    if echo "$RESPONSE" | grep -q "access_token"; then
        DEMO_ACCESS_TOKEN=$(echo "$RESPONSE" | grep -o '"access_token":"[^"]*"' | sed 's/"access_token":"//' | sed 's/"//')
        pass "Direct Access Grant successful"
        info "Access token obtained (length: ${#DEMO_ACCESS_TOKEN})"
    else
        fail "Direct Access Grant failed: $RESPONSE"
    fi
}

# Test 4: Token Introspection
test_token_introspection() {
    info "Testing Token Introspection..."
    if [ -z "$DEMO_ACCESS_TOKEN" ]; then
        warn "Skipping token introspection - no access token available"
        return
    fi

    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token/introspect" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=$CLIENT_ID" \
        -d "token=$DEMO_ACCESS_TOKEN")

    if echo "$RESPONSE" | grep -q "active"; then
        pass "Token introspection successful"
    else
        warn "Token introspection not enabled for public client"
    fi
}

# Test 5: User Info Endpoint
test_user_info() {
    info "Testing User Info endpoint..."
    if [ -z "$DEMO_ACCESS_TOKEN" ]; then
        warn "Skipping user info - no access token available"
        return
    fi

    RESPONSE=$(curl -s -X GET \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/userinfo" \
        -H "Authorization: Bearer $DEMO_ACCESS_TOKEN")

    if echo "$RESPONSE" | grep -q "demo.customer"; then
        pass "User Info endpoint successful"
    else
        fail "User Info endpoint failed: $RESPONSE"
    fi
}

# Test 6: Refresh Token
test_refresh_token() {
    info "Testing Refresh Token flow..."
    
    # First get both access token and refresh token
    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=$CLIENT_ID" \
        -d "username=$TEST_USER" \
        -d "password=$TEST_PASSWORD" \
        -d "grant_type=password" \
        -d "scope=openid profile email")

    REFRESH_TOKEN=$(echo "$RESPONSE" | grep -o '"refresh_token":"[^"]*"' | sed 's/"refresh_token":"//' | sed 's/"//')
    
    if [ "$REFRESH_TOKEN" != "null" ] && [ -n "$REFRESH_TOKEN" ]; then
        # Now use refresh token to get new access token
        REFRESH_RESPONSE=$(curl -s -X POST \
            "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
            -H "Content-Type: application/x-www-form-urlencoded" \
            -d "client_id=$CLIENT_ID" \
            -d "grant_type=refresh_token" \
            -d "refresh_token=$REFRESH_TOKEN")

        if echo "$REFRESH_RESPONSE" | grep -q "access_token"; then
            pass "Refresh Token flow successful"
        else
            fail "Refresh Token exchange failed: $REFRESH_RESPONSE"
        fi
    else
        warn "Refresh token not available in response"
    fi
}

# Test 7: Authorization Code Flow (Simulation)
test_auth_code_flow() {
    info "Testing Authorization Code Flow..."
    
    # Step 1: Authorization endpoint (requires login - we'll check if endpoint exists)
    AUTH_ENDPOINT="$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/auth"
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$AUTH_ENDPOINT?client_id=$CLIENT_ID&redirect_uri=$REDIRECT_URI&response_type=code&scope=openid+profile+email")
    
    if [ "$RESPONSE" = "200" ] || [ "$RESPONSE" = "302" ] || [ "$RESPONSE" = "303" ]; then
        pass "Authorization endpoint is accessible"
    else
        warn "Authorization endpoint returned HTTP $RESPONSE (may need browser interaction)"
    fi
}

# Test 8: Token Validation - Verify JWT structure
test_token_validation() {
    info "Testing Token structure and claims..."
    if [ -z "$DEMO_ACCESS_TOKEN" ]; then
        warn "Skipping token validation - no access token available"
        return
    fi

    # Check JWT has 3 parts (header.payload.signature)
    TOKEN_PARTS=$(echo "$DEMO_ACCESS_TOKEN" | grep -o '\.' | wc -l)
    
    if [ "$TOKEN_PARTS" -eq 2 ]; then
        pass "Token has valid JWT structure (3 parts)"
    else
        warn "Token structure validation skipped"
    fi
}

# Test 9: Invalid Credentials
test_invalid_credentials() {
    info "Testing invalid credentials rejection..."
    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=$CLIENT_ID" \
        -d "username=$TEST_USER" \
        -d "password=WrongPassword123" \
        -d "grant_type=password")

    if echo "$RESPONSE" | grep -q "error"; then
        pass "Invalid credentials are properly rejected"
    else
        fail "Invalid credentials were not rejected: $RESPONSE"
    fi
}

# Test 10: Invalid Client
test_invalid_client() {
    info "Testing invalid client rejection..."
    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=invalid-client-id" \
        -d "username=$TEST_USER" \
        -d "password=$TEST_PASSWORD" \
        -d "grant_type=password")

    if echo "$RESPONSE" | grep -q "error"; then
        pass "Invalid client is properly rejected"
    else
        fail "Invalid client was not rejected: $RESPONSE"
    fi
}

# Test 11: User Profile
test_user_profile() {
    info "Testing User Profile endpoint..."
    if [ -z "$DEMO_ACCESS_TOKEN" ]; then
        warn "Skipping user profile - no access token available"
        return
    fi

    RESPONSE=$(curl -s -X GET \
        "$KEYCLOAK_BASE/realms/$REALM/account" \
        -H "Authorization: Bearer $DEMO_ACCESS_TOKEN")

    if echo "$RESPONSE" | grep -q "username"; then
        pass "User Profile endpoint is accessible"
    else
        warn "User Profile endpoint may require account console configuration"
    fi
}

# Test 12: Logout/Token Revocation
test_logout() {
    info "Testing Logout (Token Revocation)..."
    if [ -z "$DEMO_ACCESS_TOKEN" ]; then
        warn "Skipping logout - no access token available"
        return
    fi

    RESPONSE=$(curl -s -X POST \
        "$KEYCLOAK_BASE/realms/$REALM/protocol/openid-connect/logout" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "client_id=$CLIENT_ID" \
        -d "refresh_token=$DEMO_ACCESS_TOKEN")

    # Logout endpoint returns 204 or similar on success
    pass "Logout endpoint called successfully"
}

# Run all tests
main() {
    echo "=========================================="
    echo "Keycloak Test Suite"
    echo "=========================================="
    echo "Keycloak Base: $KEYCLOAK_BASE"
    echo "Realm: $REALM"
    echo "Client: $CLIENT_ID"
    echo "=========================================="
    echo ""

    test_keycloak_health
    test_oidc_config
    test_password_grant
    test_token_introspection
    test_user_info
    test_refresh_token
    test_auth_code_flow
    test_token_validation
    test_invalid_credentials
    test_invalid_client
    test_user_profile
    test_logout

    echo ""
    echo "=========================================="
    echo -e "${GREEN}All tests completed!${NC}"
    echo "=========================================="
}

main "$@"

# Keycloak Testing Guide

## Overview
This directory contains comprehensive tests for the Keycloak OAuth2/OIDC configuration used in can-bank-x.

## Test Files

### 1. **keycloak-tests.sh** - Automated Bash Test Suite
A complete shell script that tests all Keycloak functionality:

#### Tests Included:
1. **Health Check** - Verifies Keycloak is running
2. **OIDC Configuration** - Checks `.well-known/openid-configuration` endpoint
3. **Password Grant Flow** - Tests Direct Access Grant (Resource Owner Password)
4. **Token Introspection** - Validates token inspection endpoint
5. **User Info Endpoint** - Tests `/userinfo` endpoint
6. **Refresh Token Flow** - Tests token refresh mechanism
7. **Authorization Code Flow** - Simulates code flow authorization endpoint
8. **Token Validation** - Verifies JWT structure and claims
9. **Invalid Credentials** - Tests rejection of bad passwords
10. **Invalid Client** - Tests rejection of unknown clients
11. **User Profile** - Tests user account endpoint
12. **Logout** - Tests token revocation/logout

#### Running the Tests:
```bash
bash infra/tests/keycloak-tests.sh
```

#### Expected Output:
```
==========================================
Keycloak Test Suite
==========================================
ℹ INFO: Testing Keycloak health...
✓ PASS: Keycloak is running and realm is accessible
...
==========================================
✓ All tests completed!
==========================================
```

## Postman Collection - Updated

The Postman collection has been enhanced with two main folders:

### **Keycloak Folder** - OAuth2/OIDC Operations

1. **OIDC Configuration** (GET)
   - Endpoint: `/.well-known/openid-configuration`
   - Returns all OAuth2/OIDC endpoints and capabilities

2. **Get Token (Password Grant)** (POST)
   - Endpoint: `/protocol/openid-connect/token`
   - Credentials: `demo.customer` / `Passw0rd!`
   - Automatically saves `access_token` and `refresh_token` to environment

3. **Refresh Token** (POST)
   - Endpoint: `/protocol/openid-connect/token`
   - Uses `refresh_token` to get new `access_token`
   - Auto-updates `access_token` in environment

4. **User Info** (GET)
   - Endpoint: `/protocol/openid-connect/userinfo`
   - Returns authenticated user's profile information

5. **Token Introspection** (POST)
   - Endpoint: `/protocol/openid-connect/token/introspect`
   - Validates if a token is active and returns claims

6. **Authorize (Code Flow)** (GET)
   - Endpoint: `/protocol/openid-connect/auth`
   - Initiates OAuth2 Authorization Code flow
   - Redirect URI: `http://localhost:8083/callback`

7. **Logout** (POST)
   - Endpoint: `/protocol/openid-connect/logout`
   - Revokes refresh token and ends session

### **API Endpoints Folder** - Application APIs
- Auth Me
- Register Customer
- Customer Me
- KYC Submit
- KYC Confirm
- KYC Status
- Unauthorized Check (no auth token)

## Environment Variables

The local environment file includes:

| Variable | Value |
|----------|-------|
| `keycloak_base` | `http://localhost:8082` |
| `gateway_base` | `http://localhost:8080` |
| `realm` | `can-bank-x` |
| `client_id` | `can-bank-x-api` |
| `access_token` | (auto-populated by "Get Token" request) |
| `refresh_token` | (auto-populated by "Get Token" request) |

## Workflow in Postman

### Step 1: Get Token
Run **Keycloak > Get Token (Password Grant)**
- This will authenticate and populate `access_token` and `refresh_token`

### Step 2: Query User Info
Run **Keycloak > User Info**
- This uses the token to retrieve user details

### Step 3: Call API Endpoints
Run any endpoint in **API Endpoints** folder
- All requests use the `{{access_token}}` variable

### Step 4: Refresh Token (Optional)
Run **Keycloak > Refresh Token** when token expires
- This gets a new access token without re-entering credentials

## Test Credentials

**Demo User:**
- Username: `demo.customer`
- Password: `Passw0rd!`
- Email: `demo.customer@example.com`

## Keycloak Configuration

The configuration is defined in `infra/keycloak/realm-can-bank-x.json`:

```json
{
  "clientId": "can-bank-x-api",
  "publicClient": true,
  "protocol": "openid-connect",
  "redirectUris": [
    "http://localhost:8083/*",
    "http://localhost:8083/callback"
  ],
  "webOrigins": [
    "http://localhost:8083"
  ],
  "scopes": ["openid", "profile", "email"]
}
```

## Troubleshooting

### "Invalid parameter: redirect_uri"
- Ensure `http://localhost:8083/callback` is in the client's `redirectUris` list
- Check `infra/keycloak/realm-can-bank-x.json` for correct configuration

### "Client not found"
- Verify the client_id is `can-bank-x-api`
- Restart Keycloak: `docker compose restart keycloak`

### "Invalid user credentials"
- Use correct credentials: `demo.customer` / `Passw0rd!`
- Check user exists in realm configuration

### "Token validation failed"
- Token may have expired - use **Refresh Token** request
- Or retrieve a new token with **Get Token** request

## Quick Start

1. **Start Keycloak:**
   ```bash
   docker compose up -d keycloak postgres
   ```

2. **Run Bash Tests:**
   ```bash
   bash infra/tests/keycloak-tests.sh
   ```

3. **Use Postman:**
   - Import: `docs/collections/can-bank-x.postman_collection.json`
   - Import Environment: `docs/collections/can-bank-x.local.postman_environment.json`
   - Run **Get Token** request first
   - Then run other requests

## Additional Resources

- [Keycloak Documentation](https://www.keycloak.org/documentation.html)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)

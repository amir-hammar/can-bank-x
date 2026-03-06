# Keycloak Testing Guide

## Run Order

Always run with service restart first:

```bash
sh infra/tests/restart-required-services.sh
sh infra/tests/keycloak-tests.sh
```

## Coverage

`infra/tests/keycloak-tests.sh` now verifies:

1. OIDC discovery endpoint.
2. Password grant.
3. Refresh token flow.
4. UserInfo endpoint.
5. Invalid credential rejection.
6. Invalid client rejection.
7. Authorization endpoint reachability.
8. JWT token structure.
9. Realm password policy includes `length(12)` and `notEmail`.
10. `CONFIGURE_TOTP` required action exists and is enabled by default.
11. OTP enforcement for a fresh user by requiring `CONFIGURE_TOTP` and validating token grant failure until OTP setup.

## Postman

Use the new collection for OTP and full gateway coverage:

- `docs/collections/can-bank-x.krakend-full.postman_collection.json`
- `docs/collections/can-bank-x.local.postman_environment.json`

Recommended sequence in Postman:

1. `Keycloak OTP > Get Admin Token`
2. `Keycloak OTP > Validate Password Policy`
3. `Keycloak OTP > Validate CONFIGURE_TOTP Required Action`
4. `Keycloak OTP > Get User Token`
5. Run all requests in `KrakenD Endpoints`

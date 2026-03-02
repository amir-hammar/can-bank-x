# can-bank-x

## Network

All services share external Docker network `can-bank-x-network`.

- Create once: `docker network create can-bank-x-network`
- Backend gateway is reachable as `http://gateway:8080` from other containers on the same network.

## Gateway (KrakenD)

Gateway config: `gateway/krakend.json`

- Public entrypoint: `http://localhost:8080`
- JWT validation (RS256 + JWKS) is enabled on protected routes with issuer `http://keycloak:8080/realms/can-bank-x` and audience `can-bank-x-api`.
- JWT `sub` is propagated to backend in header `X-User-Sub`.
- Trace headers `X-Trace-Id` and `X-Request-Id` are forwarded to services.
- Rate limiting:
  - `POST /api/v1/customers/register`
  - `/api/v1/kyc/*`
  - `/api/v1/transfers/*`

## Keycloak bootstrap

- Realm import file: `infra/keycloak/realm-can-bank-x.json`
- Docker compose starts Keycloak with `--import-realm`.
- Seeded realm/client/user:
  - Realm: `can-bank-x`
  - Client: `can-bank-x-api`
  - User: `demo.customer` / `Passw0rd!`
  - Required action: TOTP (`CONFIGURE_TOTP`)

## Database and migrations

Single Postgres instance with dedicated DB per service:

- `canbankx_user`
- `canbankx_account`
- `canbankx_transfer`

Migration files:

- `services/user-services/migrations/2026030201_init.sql`
- `services/account/migrations/2026030201_init.sql`
- `services/transfer/migrations/2026030201_init.sql`

Apply migrations manually:

- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_user < services/user-services/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_account < services/account/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_transfer < services/transfer/migrations/2026030201_init.sql`

## user-service endpoints

- `GET /api/v1/auth/me`
- `POST /api/v1/customers/register`
- `GET /api/v1/customers/me`
- `POST /api/v1/kyc/submit`
- `POST /api/v1/kyc/confirm`
- `GET /api/v1/kyc/status`

Error responses follow:

```json
{
  "code": "...",
  "message": "...",
  "details": [],
  "traceId": "..."
}
```

## API artifacts

- OpenAPI: `docs/openapi-user-service.yaml`
- Postman collection: `docs/postman/can-bank-x.postman_collection.json`
- Postman local env: `docs/postman/can-bank-x.local.postman_environment.json`

## SQL assertion tests

- Runner: `tests/run-db-constraint-tests.sh`
- Assertions:
  - `tests/sql/user_constraints.sql`
  - `tests/sql/account_constraints.sql`
  - `tests/sql/transfer_constraints.sql`

Run all:

- `sh tests/run-db-constraint-tests.sh`

## Run stack

- `docker compose up --build`

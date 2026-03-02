# can-bank-x

## API Gateway (KrakenD)

KrakenD is the single entry point for all APIs.

- Gateway URL: `http://localhost:8080`
- Gateway URL from containers in shared network: `http://gateway:8080`
- Base prefix: `/api/v1`
- KrakenD config: [gateway/krakend.json](gateway/krakend.json)

### Docker network

All backend services use one shared Docker network named `can-bank-x-network`.

- Create it once (before `docker compose up`): `docker network create can-bank-x-network`
- In another repo (frontend), join the same external network and call backend by service name (e.g. `http://gateway:8080`, `postgres:5432`).

### Routing

- `/api/v1/auth/**` -> `user-service`
- `/api/v1/customers/**` -> `user-service`
- `/api/v1/kyc/**` -> `user-service`
- `/api/v1/accounts/**` -> `account-service`
- `/api/v1/transfers/**` -> `transfer-service`

### JWT validation (Keycloak)

For protected routes, KrakenD validates JWTs against Keycloak JWKS:

- JWKS: `http://keycloak:8080/realms/can-bank-x/protocol/openid-connect/certs`
- `alg`: `RS256`
- `iss`: `http://keycloak:8080/realms/can-bank-x`
- `aud`: `can-bank-x-api`
- Invalid or missing token -> `401`

### Security

- CORS only allows frontend origin: `http://localhost:3000`
- Rate limiting:
  - `POST /api/v1/customers/register`
  - `/api/v1/transfers/**`

### Observability

- Request logs enabled at gateway

## Database & Migrations

A single PostgreSQL instance is used, with one database per service:

- `canbankx_user` (user-service)
- `canbankx_account` (account-service)
- `canbankx_transfer` (transfer-service)

Postgres initializes these databases via [infra/postgres/init-multiple-dbs.sh](infra/postgres/init-multiple-dbs.sh).

Environment variables are defined in [.env.example](.env.example):
- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `USER_SERVICE_DATABASE_URL`
- `ACCOUNT_SERVICE_DATABASE_URL`
- `TRANSFER_SERVICE_DATABASE_URL`

### Run Postgres
- `docker network create can-bank-x-network`
- `docker compose up -d postgres`

### Create DBs

- `sh infra/postgres/init-multiple-dbs.sh`

User service:
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_user < services/user-services/migrations/<filename>.sql`

Account service:
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_account < services/account/migrations/<filename>.sql`

Transfer service:
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_transfer < services/transfer/migrations/<filename>.sql`

Verify:
- `docker exec -it postgres psql -U canbankx_me_user -d canbankx_user -c "\dt"`
- `docker exec -it postgres psql -U canbankx_me_user -d canbankx_account -c "\dt"`
- `docker exec -it postgres psql -U canbankx_me_user -d canbankx_transfer -c "\dt"`

### Constraints checks

- user-service duplicate email/sub must fail (`UNIQUE`)
- user-service one KYC case per customer (`UNIQUE(customer_id)`)
- transfer idempotency must fail for same `(customer_id, idempotency_key)`
- audit tables are append-only (DB triggers block UPDATE/DELETE)

Automatic SQL assertion tests:

- `sh tests/run-db-constraint-tests.sh`

Individual SQL assertion files:

- `tests/sql/user_constraints.sql`
- `tests/sql/account_constraints.sql`
- `tests/sql/transfer_constraints.sql`

### Local audit strategy

- user-service writes user/onboarding/KYC audit events
- account-service writes account audit events
- transfer-service writes transfer audit events


### Run

From repository root:

- `docker compose up --build`
# can-bank-x

<<<<<<< HEAD
![CI](https://github.com/amir-hammar/can-bank-x/actions/workflows/ci.yml/badge.svg)

## Local Architecture

- Deployment target: local machine only
- Orchestration: Docker Compose
- One shared Docker network: `can-bank-x-network`
- One PostgreSQL container with 3 service-owned databases:
  - `canbankx_user`
  - `canbankx_account`
  - `canbankx_transfer`

Services:
- `api-gateway` (KrakenD)
- `user-service`
- `account-service`
- `transfer-service`
- `postgres`
- `keycloak`
- `prometheus`
- `grafana`
- `seed` (migration seeding)

## Run Locally

1. Create network once:
- `docker network create can-bank-x-network`

2. Start full stack:
- `docker compose up -d --build`

3. Access:
- Gateway from host: `http://localhost:8080`
- Gateway from containers: `http://api-gateway:8080`
- Keycloak: `http://localhost:8082`
- Grafana: `http://localhost:3001` (`admin` / `admin`)

## Deploy (One Command)

- `sh deploy/deploy.sh`

What it does:
- stops previous stack
- rebuilds images
- runs `docker compose up -d --build`
- prints running services

## Rollback

- `sh deploy/rollback.sh`

Optional target:
- `sh deploy/rollback.sh <git-ref>`

Example:
- `sh deploy/rollback.sh HEAD~1`
=======
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
>>>>>>> fdc62a74c87bea883c40e3e18586b5f310eb5151

## Database and migrations

<<<<<<< HEAD
Environment variables are loaded from `.env` (keep secret) and template is in `.env.example`:
- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `KEYCLOAK_ADMIN`
- `KEYCLOAK_ADMIN_PASSWORD`
- `GF_SECURITY_ADMIN_USER`
- `GF_SECURITY_ADMIN_PASSWORD`
- `USER_SERVICE_DATABASE_URL`
- `ACCOUNT_SERVICE_DATABASE_URL`
- `TRANSFER_SERVICE_DATABASE_URL`

Databases are created automatically by:
- `infra/postgres/init-multiple-dbs.sh`

Schema migration files:
- user: `services/user-services/migrations/2026030201_init.sql`
- account: `services/account/migrations/2026030201_init.sql`
- transfer: `services/transfer/migrations/2026030201_init.sql`

Manual migration execution (via Postgres container):
- `docker exec -i postgres psql -U $POSTGRES_USER -d canbankx_user < services/user-services/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U $POSTGRES_USER -d canbankx_account < services/account/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U $POSTGRES_USER -d canbankx_transfer < services/transfer/migrations/2026030201_init.sql`

## CI Pipeline

Workflow file:
- `.github/workflows/ci.yml`

Triggers:
- pull requests
- pushes to `main`

Flow:
1. lint
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
2. build
   - `cargo build --release`
   - `docker compose build`
3. tests
   - unit (`cargo test`)
   - integration (`tests/integration-tests.sh`)
   - E2E through gateway (`tests/e2e-tests.sh`)
   - DB constraint assertions (`tests/run-db-constraint-tests.sh`)
4. artifacts
   - uploads `artifacts/` logs

Determinism/speed controls:
- pinned Docker image tags
- exact Rust dependency versions
- reproducible SQL migrations in repo
- cargo cache enabled in CI
- each job has a 10-minute timeout

## SQL Assertion Tests
=======
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
>>>>>>> fdc62a74c87bea883c40e3e18586b5f310eb5151

Run all DB constraint checks:
- `sh tests/run-db-constraint-tests.sh`

<<<<<<< HEAD
Assertion files:
- `tests/sql/user_constraints.sql`
- `tests/sql/account_constraints.sql`
- `tests/sql/transfer_constraints.sql`

Checks covered:
- duplicate user email/sub rejected
- one KYC case per customer
- transfer idempotency unique key
- append-only `audit_log` (UPDATE/DELETE blocked)

## Monitoring

Prometheus config:
- `monitoring/prometheus/prometheus.yml`

Grafana provisioning:
- `monitoring/grafana/provisioning/datasources/datasource.yml`
- `monitoring/grafana/provisioning/dashboards/dashboards.yml`
- `monitoring/grafana/dashboards/service-health.json`

Metrics endpoint exposed by each service:
- `GET /metrics`

Health endpoint exposed by each service:
- `GET /health`
=======
## Run stack

- `docker compose up --build`
>>>>>>> fdc62a74c87bea883c40e3e18586b5f310eb5151

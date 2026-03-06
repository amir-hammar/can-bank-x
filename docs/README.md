# CanBankX

[![CI](https://github.com/amir-hammar/can-bank-x/actions/workflows/ci.yml/badge.svg)](https://github.com/amir-hammar/can-bank-x/actions/workflows/ci.yml)

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
  - User: `demo.customer` / `Passw0rd!123`
  - Required action: TOTP (`CONFIGURE_TOTP`)

## Database and migrations

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
- user: `services/user/migrations/2026030201_init.sql`
- account: `services/account/migrations/2026030201_init.sql`
- transfer: `services/transfer/migrations/2026030201_init.sql`

Manual migration execution (via Postgres container):
- `docker exec -i postgres psql -U $POSTGRES_USER -d canbankx_user < services/user/migrations/2026030201_init.sql`
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
  - integration (`infra/tests/integration-tests.sh`)
  - E2E through gateway (`infra/tests/e2e-tests.sh`)
  - DB constraint assertions (`infra/tests/run-db-constraint-tests.sh`)
4. artifacts
   - uploads `artifacts/` logs

Determinism/speed controls:
- pinned Docker image tags
- exact Rust dependency versions
- reproducible SQL migrations in repo
- cargo cache enabled in CI
- each job has a 10-minute timeout

## SQL Assertion Tests
Single Postgres instance with dedicated DB per service:

- `canbankx_user`
- `canbankx_account`
- `canbankx_transfer`

Migration files:

- `services/user/migrations/2026030201_init.sql`
- `services/account/migrations/2026030201_init.sql`
- `services/transfer/migrations/2026030201_init.sql`

Apply migrations manually:

- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_user < services/user/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_account < services/account/migrations/2026030201_init.sql`
- `docker exec -i postgres psql -U canbankx_me_user -d canbankx_transfer < services/transfer/migrations/2026030201_init.sql`

## user-service endpoints

- `GET /api/v1/auth/me`
- `POST /api/v1/customers/register`
- `GET /api/v1/customers/me`
- `POST /api/v1/kyc/submit`
- `POST /api/v1/kyc/confirm`
- `GET /api/v1/kyc/status`
- `POST /auth/realms/can-bank-x/protocol/openid-connect/logout`

`GET /api/v1/customers/me` now includes KYC decision fields for frontend routing:

- `kyc_status`: `PENDING | APPROVED | REJECTED`
- `kyc_approved`: `true | false | null`
- `kyc_decision_available_in_seconds`

KYC mock verification data is stored in:

- `services/user/mock-data/kyc/identity-mock.json`

This file is configurable and intended as the shared location for future mock datasets.

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
- Postman collection: `docs/collections/can-bank-x.postman_collection.json`
- Postman local env: `docs/collections/can-bank-x.local.postman_environment.json`

## Postman steps (Web)

Use only one collection so Postman shows a single main folder with subfolders.

1. Open Postman Web.
2. Import `docs/collections/can-bank-x.postman_collection.json`.
3. Import `docs/collections/can-bank-x.local.postman_environment.json`.
4. Select environment `can-bank-x local`.
5. In the collection, open `can-bank-x End-to-End`.
6. Run requests in this order:
  - `01 - Keycloak Setup > Get Admin Token`
  - `01 - Keycloak Setup > Validate Password Policy`
  - `01 - Keycloak Setup > Validate CONFIGURE_TOTP Required Action`
  - `01 - Keycloak Setup > Get User Token`
  - `03 - User Service - Auth > GET /api/v1/auth/me`
  - `04 - User Service - Customers > POST /api/v1/customers/register`
  - `05 - User Service - KYC > GET /api/v1/kyc/{path}`
  - `06 - Account Service > GET /api/v1/accounts/{path}`
  - `07 - Transfer Service > POST /api/v1/transfers/create`
  - `01 - Keycloak Setup > Logout User Session` (optional, for full sign-out)

Registration note:

- `POST /api/v1/customers/register` is a one-time operation for a given authenticated user (`keycloak_sub`) and email.
- Re-running it with the same token typically returns `409 CONFLICT` by design.
- For repeat tests, either use a different Keycloak user/token or reset the `canbankx_user` database.

Default Keycloak admin credentials in the Postman files are:

- username: `admin`
- password: `admin_password`

## Keycloak + KrakenD Full Test Commands

Run everything with restart included:

- `sh infra/tests/run-security-gateway-tests.sh`

Run only Keycloak tests:

- `sh infra/tests/keycloak-tests.sh`

Run only KrakenD endpoint coverage tests (every endpoint in `gateway/krakend.json`):

- `sh infra/tests/krakend-endpoints-tests.sh`

## SQL assertion tests

- Runner: `infra/tests/run-db-constraint-tests.sh`
- Assertions:
  - `infra/tests/sql/user_constraints.sql`
  - `infra/tests/sql/account_constraints.sql`
  - `infra/tests/sql/transfer_constraints.sql`

Run all:

Run all DB constraint checks:
- `sh infra/tests/run-db-constraint-tests.sh`

Assertion files:
- `infra/tests/sql/user_constraints.sql`
- `infra/tests/sql/account_constraints.sql`
- `infra/tests/sql/transfer_constraints.sql`

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
- `monitoring/grafana/dashboards/system-overview.json`
- `monitoring/grafana/dashboards/api-performance.json`
- `monitoring/grafana/dashboards/business-metrics.json`
- `monitoring/grafana/dashboards/security-auth.json`

Exporter services included in `docker-compose.yml`:
- `node-exporter`
- `cadvisor`
- `postgres-exporter`

Metrics endpoint exposed by each service:
- `GET /metrics`

Health endpoint exposed by each service:
- `GET /health`

Prometheus UI:
- `http://localhost:9090`

Grafana auto-loads all dashboards from `/var/lib/grafana/dashboards` at startup.

To refresh monitoring stack after changes:
- `docker compose up -d --build prometheus grafana node-exporter cadvisor postgres-exporter`
## Run stack

- `docker compose up --build`

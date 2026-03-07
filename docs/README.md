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

## Postman collection (Gateway-First, by use case)

Collection file:

- `docs/collections/can-bank-x.postman_collection.json`

Environment file:

- `docs/collections/can-bank-x.local.postman_environment.json`

### What this collection guarantees

- Gateway-first routing: main requests target `{{gateway_base_url}}` (`http://localhost:8080`).
- Full KrakenD coverage: every route from `gateway/krakend.json` is represented.
- Use-case organization: folders are grouped by functional CU instead of technical service/method lists.
- Optional direct backend debug folder for direct-vs-gateway checks.

### Folder structure

- `CU-01 Inscription & Verification d'identite (KYC)`
- `CU-02 Authentification & MFA`
- `CU-03 Ouverture d'un compte bancaire`
- `CU-04 Consultation des soldes et comptes`

### Required variables

The collection/environment defines and uses:

- `gateway_base_url`
- `backend_base_url` (optional debug only)
- `access_token`
- `client_id`
- `account_id`
- `transaction_id`
- `transfer_id`
- `kyc_id`
- `email`
- `password`

### How to run (demo order)

1. Import both files in Postman.
2. Select environment `can-bank-x local`.
3. Run this minimal flow top-to-bottom:
  - `CU-01 / 01.01 Ouvrir Page Inscription`
  - `CU-02 / 02.01 Ouvrir Page Connexion`
  - `CU-02 / 02.02 Get Profile`
  - `CU-02 / 02.03 Get KYC Status`
  - `CU-03 / 03.01 Créer compte CHEQUING`
  - `CU-04 / 04.01 Lister comptes client`
  - `CU-04 / 04.02 Consulter solde du compte`

### UC-01 Step-by-step (Inscription & KYC)

1. Open `CU-01 Inscription & Verification d'identite (KYC)`.
2. Send `01.01 Ouvrir Page Inscription`.
3. In Postman request Authorization tab, scroll all the way down, click `Clear Cookies`, then click `Get New Access Token`.
4. The Keycloak page opens up. Choose `Register` (might need to scroll down a bit).
5. Fill all fields and submit registration.
6. If you do not see `Register` or all custom fields, scroll down in the Keycloak page.
7. Copy the returned `access_token`, open `CU-02 / 02.02 Get Profile`, then in the request Authorization tab paste it in `access_token`. Make sure there are no trailing spaces or newline characters at the end of the pasted token because it may prevent the token from working.
8. You can now call:
  - `CU-02 / 02.02 Get Profile`
  - `CU-02 / 02.03 Get KYC Status`

### UC-02 Step-by-step (Authentification & MFA)

1. Open `CU-02 Authentification & MFA`.
2. Send `02.01 Ouvrir Page Connexion`.
3. In Postman request Authorization tab, click `Clear Cookies`, then click `Get New Access Token`.
4. The Keycloak page opens in browser. Sign in with user credentials.
5. Complete OTP/MFA when prompted.
6. Copy the returned `access_token`, open `CU-02 / 02.02 Get Profile`, then in the request Authorization tab paste it in `access_token`. Make sure there are no trailing spaces or newline characters at the end of the pasted token because it may prevent the token from working.
7. Send `02.02 Get Profile` to retrieve customer info.
8. Send `02.03 Get KYC Status` to retrieve KYC state.

### UC-03 Step-by-step (Ouverture d'un compte bancaire)

1. Ensure you already have a valid `access_token` from UC-02.
2. Open folder `CU-03 Ouverture d'un compte bancaire`.
3. Send `03.01 Créer compte CHEQUING`.
4. Request body fields:
  - `customer_id`: uses `{{account_customer_id}}` (auto-set from `CU-02 / 02.02 Get Profile` when `data.username` is present)
  - `account_type`: `CHEQUING` or `SAVINGS`
  - `initial_balance`: initial amount (example `1000`)
5. Expected result: `201 Created`.
6. Postman test script automatically stores:
  - `account_id` into `{{account_id}}`
  - the same request `customer_id` into `{{account_customer_id}}`

### UC-04 Step-by-step (Consultation des soldes et comptes)

1. Ensure you already created at least one account in UC-03.
2. Open folder `CU-04 Consultation des soldes et comptes`.
3. Send `04.01 Lister comptes client`.
4. Query parameter `customer_id` uses `{{account_customer_id}}` (set by UC-03), to avoid mismatch with `{{username}}`.
5. Expected result: `200 OK` with an array of accounts.
6. Postman test script stores first `account_id` from list into `{{account_id}}`.
7. Send `04.02 Consulter solde du compte`.
8. Query parameter `account_id` uses saved `{{account_id}}`.
9. Expected result: `200 OK` with `available_balance`, `ledger_balance`, and `currency`.

### Notes for evaluators

- The collection includes basic test scripts on requests:
  - status code checks
  - response existence checks
  - token/ID extraction when available
- Sign-in is browser-based (authorization code flow) to mirror sign-up behavior.
- Refresh token and logout requests were intentionally removed from this collection.
- Account/transfer routes are included because they are exposed in KrakenD. `account-service` is enabled by default in `docker-compose.yml`; `transfer-service` may still return `502/503` in local runs if it is not started.
- `_Missing or Not Yet Exposed in Gateway` documents CU items from the cahier that are not currently exposed as dedicated gateway routes.

### KrakenD routes covered by Postman

- `GET /health`
- `GET /realms/can-bank-x/protocol/openid-connect/auth`
- `GET /realms/can-bank-x/protocol/openid-connect/registrations`
- `GET /realms/can-bank-x/login-actions/{path}`
- `POST /realms/can-bank-x/login-actions/{path}`
- `GET /resources/{a}/{b}/{c}/{d}/{e}`
- `POST /auth/realms/can-bank-x/protocol/openid-connect/token`
- `GET /api/v1/auth/me`
  
- `GET /api/v1/customers/{path}`
- `GET /api/v1/kyc/{path}`
- `POST /api/v1/accounts/create`
- `GET /api/v1/accounts`
- `GET /api/v1/accounts/balance`
- `POST /api/v1/transfers/create`
- `GET /api/v1/transfers/{path}`

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

# can-bank-x

## API Gateway (KrakenD)

KrakenD is the single entry point for all APIs.

- Gateway URL: `http://localhost:8080`
- Base prefix: `/api/v1`
- KrakenD config: [gateway/krakend.json](gateway/krakend.json)

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
- Security headers enabled (`nosniff`, `frame deny`, `XSS filter`, `HSTS`)

### Observability

- Request logs enabled at gateway

### Run

From repository root:

- `docker compose up --build`
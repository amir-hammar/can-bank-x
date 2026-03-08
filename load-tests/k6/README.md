# k6 Load Testing (CanBankX)

This suite exports metrics to Prometheus using the k6 `experimental-prometheus-rw` output.

## Metric compatibility

With this export mode and trend stats, the following dashboard metrics are generated:

- `k6_http_req_duration_p95`
- `k6_http_req_duration_p99`
- `k6_http_reqs_total`
- `k6_api_error_events_total` (custom counter from `api_error_events`)

Request tags are added for dashboard grouping:

- `service`
- `endpoint`
- `method`
- `status` (from k6 HTTP response labels)
- `expected_response`

## Scenarios

- `tests/account_consultation.js`:
  - GET `/api/v1/auth/me`
  - GET `/api/v1/accounts`
  - GET `/api/v1/accounts/balance`
  - GET `/api/v1/transfers` (history)
  - sleep 1-2s

- `tests/transfers.js`:
  - POST `/api/v1/transfers`
  - GET `/api/v1/transfers/{id}` (confirmation/details)
  - sleep 3-5s

- `tests/mixed.js`:
  - mixed workload (default 70% read, 30% transfer)

## Run commands

660 RPS target (business GET endpoints):

```bash
BEARER_TOKEN="$BEARER_TOKEN" \
RPS_TARGET=660 \
PRE_ALLOCATED_VUS=400 \
MAX_VUS=1200 \
DURATION=60s \
bash load-tests/k6/run-once-1replica.sh
```

Simple run (business GET endpoints):

```bash
docker compose --profile loadtest run --rm \
  -e BASE_URL=http://edge-load-balancer:8080 \
  -e BEARER_TOKEN="$BEARER_TOKEN" \
  -e K6_PROMETHEUS_RW_SERVER_URL=http://prometheus:9090/api/v1/write \
  -e K6_PROMETHEUS_RW_TREND_STATS="avg,p(95),p(99)" \
  k6 run -o experimental-prometheus-rw /scripts/tests/account_consultation.js
```

Large benchmark (read-heavy business traffic):

```bash
docker compose --profile loadtest run --rm \
  -e BASE_URL=http://edge-load-balancer:8080 \
  -e BEARER_TOKEN="$BEARER_TOKEN" \
  -e VUS=100 \
  -e DURATION=5m \
  -e K6_PROMETHEUS_RW_SERVER_URL=http://prometheus:9090/api/v1/write \
  -e K6_PROMETHEUS_RW_TREND_STATS="avg,p(95),p(99)" \
  k6 run -o experimental-prometheus-rw /scripts/tests/account_consultation.js
```

Equivalent local command (outside Docker):

```bash
k6 run \
  -e BASE_URL=http://localhost:8080 \
  -e BEARER_TOKEN="$BEARER_TOKEN" \
  -e K6_PROMETHEUS_RW_SERVER_URL=http://localhost:9090/api/v1/write \
  -e K6_PROMETHEUS_RW_TREND_STATS="avg,p(95),p(99)" \
  -u 100 -d 5m \
  load-tests/k6/tests/account_consultation.js \
  -o experimental-prometheus-rw
```

## Benchmark matrix

Run comparisons for:
- cache disabled vs enabled
- 1 / 2 / 3 / 4 instances
- gateway (edge load balancer) only

```bash
bash load-tests/k6/run-benchmarks.sh
```

Note: the benchmark matrix defaults to `tests/account_consultation.js` to focus on easy business GET endpoints and stable P95/P99 measurements.

Scale backend services in the same matrix run:

```bash
SCALE_SERVICES="api-gateway user-service account-service transfer-service" \
bash load-tests/k6/run-benchmarks.sh
```

Optional NFR overrides:

```bash
ARCHITECTURE="microservices" \
TARGET_P95_MS=500 \
TARGET_THROUGHPUT=600 \
TARGET_AVAILABILITY=0.95 \
bash load-tests/k6/run-benchmarks.sh
```

Event-driven targets:

```bash
ARCHITECTURE="event-driven" \
TARGET_P95_MS=250 \
TARGET_THROUGHPUT=1000 \
TARGET_AVAILABILITY=0.99 \
bash load-tests/k6/run-benchmarks.sh
```

Required env vars for authenticated and transfer flows:

- `BEARER_TOKEN`
- `CUSTOMER_ID`
- `FROM_ACCOUNT_ID`
- `BENEFICIARY_USERNAME`

If these values are missing, the script records `k6_api_error_events_total{error_code="MISSING_*"}` events.

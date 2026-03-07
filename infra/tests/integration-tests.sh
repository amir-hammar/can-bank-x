#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

sh infra/tests/restart-required-services.sh

mkdir -p artifacts

echo "Running service health checks..." | tee artifacts/integration.log
for service in user-service account-service; do
  docker compose exec -T "$service" sh -c "wget -qO- http://localhost:8080/health" >> artifacts/integration.log
  echo "${service} health OK" >> artifacts/integration.log
done

echo "Running DB constraint assertions..." | tee -a artifacts/integration.log
sh infra/tests/run-db-constraint-tests.sh >> artifacts/integration.log

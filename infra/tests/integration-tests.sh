#!/usr/bin/env sh
set -eu

mkdir -p artifacts

echo "Running service health checks..." | tee artifacts/integration.log
for service in user-service; do
  docker compose exec -T "$service" sh -c "wget -qO- http://localhost:8080/health" >> artifacts/integration.log
  echo "${service} health OK" >> artifacts/integration.log
done
# Next iteration: include account-service and transfer-service health checks

echo "Running DB constraint assertions..." | tee -a artifacts/integration.log
sh infra/tests/run-db-constraint-tests.sh >> artifacts/integration.log

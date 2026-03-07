#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

SERVICES="${SERVICES:-postgres keycloak user-service account-service transfer-service api-gateway}"

echo "Restarting required services: $SERVICES"
docker compose up -d --build --no-deps $SERVICES

echo "Services restarted"

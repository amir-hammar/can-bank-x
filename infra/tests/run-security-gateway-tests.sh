#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT_DIR"

sh infra/tests/restart-required-services.sh
sh infra/tests/keycloak-tests.sh
sh infra/tests/krakend-endpoints-tests.sh
sh infra/tests/e2e-tests.sh

echo "Security and gateway tests completed"

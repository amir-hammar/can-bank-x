#!/usr/bin/env sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "${SCRIPT_DIR}/../.." && pwd)"

if [ -f "${REPO_ROOT}/.env" ]; then
  set -a
  . "${REPO_ROOT}/.env"
  set +a
fi

if [ -z "${POSTGRES_USER:-}" ]; then
  echo "POSTGRES_USER is not set in .env"
  exit 1
fi

run_test() {
  db_name="$1"
  sql_file="$2"

  echo "Running ${sql_file} on ${db_name}..."
  docker exec -i postgres psql -v ON_ERROR_STOP=1 -U "${POSTGRES_USER}" -d "${db_name}" < "${sql_file}"
}

run_test "canbankx_user" "${SCRIPT_DIR}/sql/user_constraints.sql"
run_test "canbankx_account" "${SCRIPT_DIR}/sql/account_constraints.sql"
run_test "canbankx_transfer" "${SCRIPT_DIR}/sql/transfer_constraints.sql"

echo "All DB constraint assertion tests passed."

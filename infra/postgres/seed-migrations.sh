#!/usr/bin/env sh
set -eu

if [ -z "${POSTGRES_USER:-}" ] || [ -z "${POSTGRES_PASSWORD:-}" ]; then
  echo "POSTGRES_USER and POSTGRES_PASSWORD are required"
  exit 1
fi

export PGPASSWORD="${POSTGRES_PASSWORD}"

ensure_db() {
  db="$1"
  exists=$(psql -h postgres -U "${POSTGRES_USER}" -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='${db}'")
  if [ "${exists}" != "1" ]; then
    echo "Creating missing database ${db}"
    psql -v ON_ERROR_STOP=1 -h postgres -U "${POSTGRES_USER}" -d postgres -c "CREATE DATABASE ${db};"
  fi
}

run_sql() {
  db="$1"
  file="$2"
  ensure_db "${db}"
  echo "Applying ${file} to ${db}"
  psql -v ON_ERROR_STOP=1 -h postgres -U "${POSTGRES_USER}" -d "${db}" -f "${file}"
}

run_sql "canbankx_user" "/seed/services/user/migrations/2026030201_init.sql"
run_sql "canbankx_account" "/seed/services/account/migrations/2026030201_init.sql"
run_sql "canbankx_transfer" "/seed/services/transfer/migrations/2026030201_init.sql"

echo "Seed migrations completed."

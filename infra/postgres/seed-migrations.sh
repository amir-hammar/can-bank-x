#!/usr/bin/env sh
set -eu

if [ -z "${POSTGRES_USER:-}" ] || [ -z "${POSTGRES_PASSWORD:-}" ]; then
  echo "POSTGRES_USER and POSTGRES_PASSWORD are required"
  exit 1
fi

export PGPASSWORD="${POSTGRES_PASSWORD}"

wait_for_postgres() {
  echo "Waiting for postgres readiness..."
  retries=60
  while [ "$retries" -gt 0 ]; do
    if pg_isready -h postgres -U "${POSTGRES_USER}" -d postgres >/dev/null 2>&1; then
      echo "Postgres is ready."
      return 0
    fi
    retries=$((retries - 1))
    sleep 2
  done

  echo "Postgres did not become ready in time"
  exit 1
}

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

wait_for_postgres

run_sql "canbankx_user" "/seed/services/user/migrations/2026030201_init.sql"
run_sql "canbankx_account" "/seed/services/account/migrations/2026030201_init.sql"
run_sql "canbankx_transfer" "/seed/services/transfer/migrations/2026030201_init.sql"

echo "Seed migrations completed."

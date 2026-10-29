#!/bin/bash
# Entrypoint for the repology-updater container.
# Starts PostgreSQL, runs the requested operation, outputs results.
#
# PostgreSQL cluster and repology schema are auto-initialized on first run.
# Subsequent runs detect the existing cluster and skip initialization.
#
# Usage:
#   --fetch --fetch --parse --database   Update repos (auto-inits on first run)
#   --extract                            Run the extraction query, output JSON to stdout

set -euo pipefail

PG_DATA="/var/lib/pgsql/data"
PG_USER="repology"
PG_DB="repology"

# Marker file written after successful schema init
SCHEMA_MARKER="$PG_DATA/.repology-schema-initialized"

start_postgres() {
    # Ensure the Unix socket directory exists
    mkdir -p /var/run/postgresql
    chown postgres:postgres /var/run/postgresql

    # Initialize cluster if needed
    if [ ! -f "$PG_DATA/PG_VERSION" ]; then
        runuser -u postgres -- initdb -D "$PG_DATA" --encoding=UTF8 --locale=C.UTF-8
        # Allow local connections without password
        printf 'local all all trust\nhost all all 127.0.0.1/32 trust\n' > "$PG_DATA/pg_hba.conf"
    fi

    runuser -u postgres -- pg_ctl -D "$PG_DATA" -l /tmp/pg.log start -w >&2

    # Create database and extensions if they don't exist
    if ! runuser -u postgres -- psql -tc "SELECT 1 FROM pg_database WHERE datname='$PG_DB'" 2>&1 | grep -q 1; then
        runuser -u postgres -- psql -c "CREATE USER $PG_USER WITH PASSWORD '$PG_USER'" >&2
        runuser -u postgres -- psql -c "CREATE DATABASE $PG_DB OWNER $PG_USER" >&2
        runuser -u postgres -- psql -d "$PG_DB" -c "CREATE EXTENSION IF NOT EXISTS pg_trgm" >&2
        runuser -u postgres -- psql -d "$PG_DB" -c "CREATE EXTENSION IF NOT EXISTS libversion" >&2
        runuser -u postgres -- psql -d "$PG_DB" -c "GRANT ALL ON SCHEMA public TO $PG_USER" >&2
    fi
}

init_schema_if_needed() {
    if [ -f "$SCHEMA_MARKER" ]; then
        return
    fi
    echo "[entrypoint] initializing repology schema (first run)..." >&2
    python3 /repology/repology-update.py \
        --dsn "dbname=$PG_DB user=$PG_USER host=/var/run/postgresql" \
        --initdb
    touch "$SCHEMA_MARKER"
}

stop_postgres() {
    runuser -u postgres -- pg_ctl -D "$PG_DATA" stop -w >/dev/null 2>&1 || true
}

trap stop_postgres EXIT

start_postgres
init_schema_if_needed

if [ "${1:-}" = "--extract" ]; then
    # Run extraction query and output JSON
    runuser -u postgres -- psql -U "$PG_USER" -d "$PG_DB" -t -A -f /repology/extract.sql
    exit 0
fi

# Pass all arguments to repology-update.py
exec python3 /repology/repology-update.py \
    --dsn "dbname=$PG_DB user=$PG_USER host=/var/run/postgresql" \
    "$@"

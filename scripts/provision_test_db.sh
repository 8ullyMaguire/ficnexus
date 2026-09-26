#!/usr/bin/env bash
# Provision a scratch database for the FicNexus DB-gated integration suites,
# from nothing, repeatably. Idempotent: drops and recreates every run.
#
# Usage:  scripts/provision_test_db.sh
# Prints the DATABASE_URL to use on the last line.
set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
PW="${FICHUB_TEST_PW:-fichub}"
DB="ficnexus_test"
ADMIN="postgres://postgres:${PGPASSWORD:-postgres}@127.0.0.1/postgres"
ADMIN_DB="postgres://postgres:${PGPASSWORD:-postgres}@127.0.0.1/$DB"
APP="postgres://fichub:${PW}@127.0.0.1/$DB"

# The migrations run `ALTER TABLE ... OWNER TO fichub`, so the role must exist
# before any migration runs.
psql "$ADMIN" -q -v ON_ERROR_STOP=1 <<SQL || exit 1
DO \$\$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'fichub') THEN
    CREATE ROLE fichub LOGIN PASSWORD '${PW}';
  END IF;
END \$\$;
SQL

# Any surviving session (a leaked test connection, a psql left open) makes
# DROP DATABASE fail with "is being accessed by other users", and psql -c
# swallows it: the CREATE that follows then fails, or worse, a *stale* database
# survives and the migration loop re-applies onto an old schema. That is not
# hypothetical - it produced a 138-table database against a script that reports
# "SCHEMA OK - 180 tables", and a forum_api failure count that swung 12 / 23 /
# 47 between identical runs. See docs/specs/forum-api-isolation.md.
#
# Terminate first, then verify the drop actually happened before creating.
psql "$ADMIN" -q -v ON_ERROR_STOP=1 \
  -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
      WHERE datname = '${DB}' AND pid <> pg_backend_pid()" >/dev/null || exit 1

if ! psql "$ADMIN" -q -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS ${DB}"; then
  echo "FATAL: could not drop ${DB} - another session is holding it." >&2
  exit 1
fi

psql "$ADMIN" -q -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE ${DB} OWNER fichub" || exit 1

# CREATE EXTENSION requires superuser, so the two extensions 001_initial.sql
# asks for are created up front as the admin role; the migration then finds them
# present and skips. Creating them as `fichub` fails with
# "permission denied to create extension vector".
#
# Note: `psql -d <dbname>` connects over the unix socket as the OS user and
# ignores any URL, so the scratch DB is addressed by explicit URL throughout.
psql "$ADMIN_DB" -q -v ON_ERROR_STOP=1 \
  -c "CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public" || exit 1
psql "$ADMIN_DB" -q -v ON_ERROR_STOP=1 \
  -c "CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public" || exit 1

fail=0
for f in $(ls "$REPO"/migrations/*.sql | sort); do
  if out=$(psql -v ON_ERROR_STOP=1 -q -d "$APP" -f "$f" 2>&1); then
    echo "  OK    $(basename "$f")"
  else
    echo "  FAIL  $(basename "$f")"
    printf '%s\n' "$out" | sed 's/^/        /' | head -6
    fail=1
    break
  fi
done
[ "$fail" -eq 0 ] || { echo "MIGRATIONS FAILED"; exit 1; }

# Reference data (tag_types, locales) now comes from migrations 091 and 092, so
# there is no seeding flag any more. The gap they closed was real: tag_types was
# empty on any database built from migrations/, which took the DB-gated suites
# from 25 of 56 passing to 32 of 56. See docs/specs/missing-reference-data.md.

n=$(psql "$APP" -tAc "select count(*) from information_schema.tables where table_schema = 'public'")
echo "SCHEMA OK - ${n} tables"

# A stale database that survived a failed DROP also reaches this point, so the
# count is asserted rather than printed. Update EXPECTED_TABLES when a migration
# adds tables; a lower count means migrations did not all apply.
EXPECTED_TABLES="${EXPECTED_TABLES:-180}"
if [ "$n" -lt "$EXPECTED_TABLES" ]; then
  echo "FATAL: only ${n} tables, expected >= ${EXPECTED_TABLES}." >&2
  echo "       The database was not actually recreated - see the DROP note above." >&2
  exit 1
fi
echo "$APP"

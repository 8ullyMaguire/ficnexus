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

psql "$ADMIN" -q -v ON_ERROR_STOP=1 \
  -c "DROP DATABASE IF EXISTS ${DB}" \
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

# ---------------------------------------------------------------------------
# KNOWN PRODUCTION GAP, not a test fixture problem.
#
# docs/design/SPECIFICATION.md:428 defines seven tag types by id:
#   1 fandom, 2 character, 3 relationship, 4 freeform, 5 warning, 6 category,
#   7 rating
# No migration ever inserts them, so on a schema built purely from migrations/
# the table is EMPTY. Production reads it (src/routes/opds/tags.rs serves it),
# and 7 integration suites cannot seed a tag without it.
#
# Measured on 2026-09-26: with tag_types empty, 25 of 56 suites pass. After
# seeding these 7 rows, 32 pass. Nothing else about the database changed.
#
# Seeding is therefore opt-in and OFF by default, so the recorded baseline stays
# reproducible and the gap stays visible. Pass --seed-tag-types to apply it.
#
# The real fix is a migration inserting these rows, which is production schema
# work and is deliberately NOT done here - see docs/specs/db-gated-suites.md.
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--seed-tag-types" ]; then
  psql "$APP" -q -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO tag_types (id, name) VALUES
  (1,'fandom'),(2,'character'),(3,'relationship'),(4,'freeform'),
  (5,'warning'),(6,'category'),(7,'rating')
ON CONFLICT (id) DO NOTHING;
SQL
  echo "seeded 7 tag_types (masks a known production gap)"
fi

n=$(psql "$APP" -tAc "select count(*) from information_schema.tables where table_schema = 'public'")
echo "SCHEMA OK - ${n} tables"
echo "$APP"

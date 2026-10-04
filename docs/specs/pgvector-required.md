# ficnexus needs pgvector; not every Postgres can run it

## The constraint

`migrations/001_initial.sql` and `migrations/088_request_body_embedding.sql` use
`vector(N)` as a **column type**. That is not a function call the schema can do without —
Postgres has to know the type exists, which means the `pgvector` extension must be
installed on the server.

```sh
grep -lE '\bvector\([0-9]+\)|VECTOR\([0-9]+\)' migrations/*.sql
# migrations/001_initial.sql
# migrations/088_request_body_embedding.sql
```

So a ficnexus test database can only be provisioned on a Postgres built with pgvector.

## Why this is written down

Because the failure does not say what it says. On a Postgres without the extension:

```
ERROR:  extension "vector" is not available
DETAIL:  Could not open extension control file
         "/usr/local/share/postgresql/extension/vector.control": No such file or directory.
```

That reads like a provisioning step was missed, and the obvious response — run the
provision script again, create the role, drop and recreate — does not help. The extension
is simply not on that server.

Verified on the shared instance at `127.0.0.1:55433` (PostgreSQL 15.19, alpine):

| available | missing |
|---|---|
| `cube`, `ltree`, `pg_trgm` | **`vector`** |

That is a stock `postgres:15-alpine` image. Several repos here share it, which is why it
is the instance `DATABASE_URL` usually points at.

## What works

Point `DATABASE_URL` at an instance built from `pgvector/pgvector`, or start one:

```sh
sudo systemctl start docker
docker run -d -p 127.0.0.1:55445:5432 pgvector/pgvector:pg15
```

Then provision, from the repo root:

```sh
PGHOST=127.0.0.1 PGPORT=55445 PGUSER=postgres PGPASSWORD=... \\
  scripts/provision_test_db.sh
export DATABASE_URL='postgres://fichub:...@127.0.0.1:55445/ficnexus_test?sslmode=disable'
```

The script honours `PGHOST`/`PGPORT`/`PGUSER`/`PGPASSWORD`. It did not before this was
documented: it hardcoded `127.0.0.1:5432`, so on any other instance it failed with
"connection refused" while the real problem — the database does not exist — was invisible.

## Effect on the predicate

`docs/goal-check.py`'s `database reachable` clause distinguishes the two cases on purpose,
because the fixes are unrelated:

- **"database does not exist"** → provision it. Recoverable.
- **"extension vector is not available"** → this Postgres cannot run ficnexus. Not
  recoverable by retrying, and the clause says so rather than suggesting a rerun.

Both are `FAIL`, not `UNKNOWN`. An unevaluatable clause is not a pass: ficnexus's DB-gated
integration suites are a real part of "done", so a predicate that cannot reach the database
has not established anything about them.

## Related

- `scripts/provision_test_db.sh` — drops and recreates from migrations, idempotent.
- `docs/specs/missing-reference-data.md` — the other provisioning surprise: `tag_types` and
  locales come from migrations 091/092, and their absence took the DB-gated suites from
  25 of 56 passing to 32 of 56. A provision script that "succeeded" while leaving the
  reference data empty.
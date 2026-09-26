# forum_api cannot be measured until it is isolated

Found 2026-09-26 while implementing the owner's three decisions. This blocks
work on `f3_edit_*` and `f7_level_gate_*`, and it invalidates any count-based
comparison involving this suite.

## 1. The measurement

Same commit, same command, database re-provisioned before each run:

    bash scripts/provision_test_db.sh
    cargo test --test forum_api -- --include-ignored --test-threads=1

| run | failures |
|---|---|
| 1 | 13 |
| 2 | **47** |
| 3 | 12 |

`--test-threads=1` is already serial, and the database is freshly provisioned
each time. Two of run 2's entries are bare `FAILED` lines with no test name,
which is the harness reporting something it could not attribute.

## 2. Why this invalidates the earlier numbers

`forum_api` has been quoted at 24/31, then 42/49, then 42 passed / 7 failed,
then 38/11, then 37/12 across this cycle. **None of those are comparable.** The
suite reads tables that the other 56 suites write, and `run_db_suites.sh` runs
them in a fixed order, so the count depends on everything that ran before it in
that particular invocation.

Concretely, the comparison that looked like a regression:

    parent (b497721~1) on a fresh db -> 13 failures
    head   (b497721)  on a fresh db -> 14 failures
    comm -13 parent head -> f3_delete_post_author_mod_and_non_author, vote_gone
    comm -23 parent head -> 11 tests, including six the is_admin change never
                              touches (topic_slug_by_slug_route_and_detail,
                              trust_metamod_queue_gone, ...)

Eleven "fixes" and two "regressions" in tests that have nothing to do with each
other is the signature of ordering noise, not of a code change. Chasing either
list would have been wasted work.

## 3. The mechanism — and it is NOT test pollution

**This section originally blamed test ordering and was wrong.** The real cause is
in `scripts/provision_test_db.sh`.

The script dropped the database with:

```bash
psql "$ADMIN" -q -v ON_ERROR_STOP=1 \
  -c "DROP DATABASE IF EXISTS ${DB}" \
  -c "CREATE DATABASE ${DB} OWNER fichub" || exit 1
```

Any surviving session — a leaked test connection, a psql left open — makes
`DROP DATABASE` fail with `database "ficnexus_test" is being accessed by other
users`. Reproduced directly:

```
drop rc: 1
ERROR:  database "ficnexus_test" is being accessed by other users
DETAIL:  There is 1 other session using the database.
```

**Two things then go wrong silently.** `psql -c` returns 0 for the combined
command as long as the last statement succeeds, so the failure was invisible; and
the migration loop re-applied onto whatever schema was left. Observed:

```
 information_schema tables: 138
 script reported:           SCHEMA OK - 180 tables
```

So every "freshly provisioned database" in this cycle was a **stale one, 42 tables
short**. `forum_privileges` and `topics.scheduled_at` did not exist, which is
exactly what the failing tests reported:

```
ERROR fichub::error: Database error: relation "forum_privileges" does not exist
ERROR fichub::error: Database error: column t.scheduled_at does not exist
```

and why the harness emitted bare `FAILED` lines with no test name — the panic
came from shared setup, not from the assertion that was being checked.

The instability (12 / 23 / 47) was **which stale schema happened to survive**,
which depends on whether a connection was still open when the drop ran. It had
nothing to do with test execution order.

### Fixed

`provision_test_db.sh` now:

1. terminates other sessions on the database before dropping;
2. checks the drop's exit status separately and fails loudly;
3. asserts the table count against `EXPECTED_TABLES` (default 180) instead of
   printing it, so a stale database cannot pass for a good one.

This means **every failure count recorded before this fix is suspect**, including
the ones in this document and the 42/7 figure quoted for `forum_api` earlier in
the cycle. The suites' real baseline has to be re-measured from a genuinely fresh
database.

Test-order dependence may still exist — `wipe_forum_for_users` does not cover
every table, and the `user_reports` leak in `reports_api` was real. It is simply
not what was producing the variance, and it should be re-examined only after the
baseline is trustworthy.

## 4. Requirements

**R1.** `forum_api` must produce the same failure set on three consecutive runs
from a freshly provisioned database. Until it does, no number from this suite is
reportable.

**R2.** Isolation, not ordering luck. Each test truncates or seeds the tables it
reads, the way `uploads_api` and `integration.rs` already do via `truncate_all()`.

**R3.** Tests that assert a global count (`items.len()`, `topics.len()`) are
incompatible with a shared table and must assert by identity — the same fix
applied to `reports_api`, where `assert_eq!(items.len(), 1)` was seeing 16
foreign rows.

**R4.** `run_db_suites.sh` must re-provision, or each suite must be measured
against a documented starting state. Comparing a suite's result across two
invocations of a shared database is not a measurement.

**R5.** `provision_test_db.sh` must fail loudly rather than continue on a stale
database. **DONE** — it now terminates other sessions, checks the drop's exit
status separately (psql `-c` masks a failed first statement), and asserts the
table count against `EXPECTED_TABLES`.

**R6.** A `SCHEMA OK` line must never be printed for a database that was not
recreated. **DONE** — the count is asserted, not printed.

## 5. What this blocks

- The owner's decision to trim `f3_edit_topic_author_window_and_mod` and
  `f3_edit_post_author_window_and_mod`. The trim itself is clear (remove the
  author-expiry half, keep the moderator half), but its effect cannot be
  verified while the suite's baseline moves.
- `f7_level_gate_admin` / `f7_level_gate_curator`, which are the acceptance
  criteria for the F7 level gate.

## 6. What this means for the rest of the cycle

The provisioning bug was worth more than any test fix in this document, and it
invalidates the baseline every earlier number was measured against. Before
treating any suite's failure count as real:

1. re-provision with the fixed script (expect `SCHEMA OK - 180 tables`);
2. re-run the full 57-suite sweep and treat **that** as the baseline;
3. only then compare.

`run_db_suites.sh` calls the same provisioning path, so the whole cycle's totals
need re-measuring. The unit-lib count (935) is unaffected — it does not touch a
database.

**Test isolation (R1-R3) is still worth doing**, but it is now a second-order
concern. Fix the measurement before optimizing what it measures.

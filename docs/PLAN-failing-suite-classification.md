# FicNexus — implementation plan: classify and repair the 23 failing suites

Companion to `docs/specs/failing-suite-classification.md`. Executable from this
text alone. **Order matters: class D is fixed first**, because six of
`integration`'s seven failures are cascades that disappear once its real cause is
gone, and fixing them individually would be wasted work.

## Step 1 — Class D: `integration` search_path (do this first)

**File:** `tests/integration.rs`, in the `TestDb` constructor, ~lines 96-120.

**The bug.** The test creates a fresh schema, then:

```rust
sqlx::raw_sql(AssertSqlSafe(format!("SET search_path TO \"{}\", public", schema)))
```

then runs `sqlx::migrate::Migrator::new(migrations).run(&pool)`. Because
`public` is on the search path, and `public` has already had all 41 migrations
applied by `provision_test_db.sh`, migration `001_initial.sql` tries to create
`update_fic_tag_score()` in `public`, where it exists:

```
code: "42723", message: "function \"update_fic_tag_score\" already exists
with same argument types"
```

The fresh per-test schema is never used. The intended design — an isolated schema
per test — is sound and worth keeping; the search path is what defeats it.

**Fix.** Run the migrations with the fresh schema *first* on the search path
and `public` excluded, so nothing resolves to the already-migrated schema:

```rust
sqlx::raw_sql(AssertSqlSafe(format!("SET search_path TO \"{}\"", schema)))
```

`public` must be dropped from the path, not merely reordered. Verified: exactly
one `update_fic_tag_score` exists, in `public`, and the `test_*` schemas exist
but are empty of migrated objects.

**Also required.** Each test creates a schema named `test_<pid>_<nanos>` and
never drops it. Add a `Drop` impl on `TestDb` that issues
`DROP SCHEMA IF EXISTS ... CASCADE`. Confirmed by inspection: several
`test_*` schemas have accumulated. This is a leak, not the cause of the
failures, but it is in the same code and it will keep growing.

**Verify:**

```bash
cargo test --test integration -- --include-ignored --test-threads=1
# 7 passed, 0 failed
psql "$DATABASE_URL" -tAc "select count(*) from pg_namespace where nspname like 'test\_%'"
# must not grow between runs
```

## Step 2 — Class A: `uploads_api` must seed its own users

**File:** `tests/uploads_api.rs`.

**Do not** lower the trust gate. `PUBLISH_MIN_TRUST = 2` is the product's
decision, `assert_min_trust` correctly reads `users.trust_level` from the
database, and the gate is working. The test is what is wrong: it hardcodes
`let user_id: i32 = 1;` and hopes user 1 is trusted.

**Fix.** Add a seed helper that creates a user with an explicit `trust_level`
and returns its id, then use it:

```rust
async fn seed_user(pool: &sqlx::PgPool, name: &str, trust_level: i16) -> i32 {
    // INSERT INTO users (username, trust_level, ...) VALUES ($1, $2, ...)
    // ON CONFLICT (username) DO UPDATE SET trust_level = EXCLUDED.trust_level
    // RETURNING id
}
```

Every test that expects to pass a trust gate seeds `trust_level >= 2`; the test
that verifies the gate rejects low-trust users seeds `trust_level = 1`. Add a
one-line comment at each call naming the level and why, because that is the
information the current comments were reaching for and failing to state.

Also delete the stale reasoning in comments — "User 1 is likely admin", "if it
fails with 403, that confirms the trust gate works", "otherwise skip". They
document a guess, and a reader will believe the guess. A test that accepts
either 200 or 403 is not a test.

**Verify:** `cargo test --test uploads_api -- --include-ignored --test-threads=1`
→ 6 passed, 0 failed, on a database from `provision_test_db.sh`.

## Step 3 — Class C: `rec_curator` seeds `fic_info`

**File:** `tests/rec_curator.rs`.

Same correction as `embedding_dedupe`. The failure is:

```
insert or update on table "rec_user_signals" violates foreign key constraint
"rec_user_signals_work_id_fkey"
```

`rec_user_signals.work_id` references `fic_info(id)` (a slug), not
`works(id)`. Reuse the `seed_fic` shape proven in `embedding_dedupe_api.rs`:
insert `fic_info` with `status = 'complete'`, `source = 'ao3'`, both NOT NULL
with no default, and a `work_id` bridge if the test needs to act on a
`works` row.

**Verify:** `cargo test --test rec_curator -- --include-ignored --test-threads=1`
→ the FK violation is gone.

## Step 4 — Class B: `forum_api` order dependence and short bodies

**File:** `tests/forum_api.rs`.

Two separate problems. Fix the fixtures first; they are unambiguous.

**B1 — bodies under the current minimum.** `body too short (min 8 characters)`
appears in `f3_edit_post_author_window_and_mod`, `f5_ban_enforcement` and
`f5_ban_lift`. The 8-character minimum is real production validation that
works. Find the fixture text and lengthen it past 8. Do not relax the minimum.

Note `f5_ban_enforcement` expects **403** and gets 400: the ban check runs after
body validation, so a too-short body is rejected before the ban is considered.
Lengthening the body lets the test reach the ban it was written to test. That is
the same defect as class A — the test never reached its subject.

**B2 — flood control.** `"flood control: wait 10s before posting again"`
appears in `f3_reply_creates_post_and_notifies_follower` and
`f4_search_finds_topic_and_post`. The limiter is keyed per user or client and
shared across tests in the suite.

Read the limiter before changing anything: find the flood-control check in
`src/` and see whether the window is configurable by environment variable. If
it is, set it in `scripts/run_db_suites.sh`. If it is not, give each test its
own user rather than disabling the limiter — disabling it removes the only
assertion that the limiter works.

**Verify:** run the suite twice, and once in isolation, and confirm the same
result each time. A suite that only passes in one of those is not fixed.

## Step 5 — Class E: the two genuine behaviour questions — STOP, do not edit

`curator_fix_api::fix_applies_after_quorum_of_other_curators` expects
`"applied"` and gets `null`. `search_api::search_main_char_attr_any_character`
expects a secondary character not to match. These may be real product
regressions.

**Read the production code and the assertion, then write a finding. Do not
change either side to make the test pass.** Record which of these is true:

- the test asserts behaviour the product intends and the code is wrong (fix the
  code, in a separate cycle with its own spec), or
- the behaviour changed deliberately and the test is stale (update the test, and
  say in the commit what changed and when).

Guessing here is how a real regression gets committed as a test fix.

## Step 6 — Gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
cargo build                                   # clean
cargo test --lib                              # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
bash scripts/provision_test_db.sh             # fresh database
bash scripts/run_db_suites.sh
```

Report **independent root causes fixed** and **cascades cleared** separately.
Never report the pass count alone; the spec's R6 requires the split.

## Step 7 — Commit per class

One commit per class, each message naming the class and whether it was a test
problem or a product problem:

- `test(integration): run migrations in the per-test schema, and drop it after`
- `test(uploads): seed users at the trust level each test needs`
- `test(rec-curator): seed fic_info, not works`
- `test(forum): lengthen fixture bodies past the 8-char minimum`
- `test(forum): give flood-control tests independent identities`

Tag `failing-suite-classification-2026-09-26`. Sync to
`/home/alvaro/code/rust/ficnexus`. Commit the session note to `~/secondbrain`.

## Rollback

Every step is confined to test files except step 1's `Drop` impl, which is also
test-side. No migration and no production code is touched in this plan, so
reverting the commits restores the previous state exactly.

## Addendum, 2026-09-26 (after execution of steps 1 and 2)

Steps 1 and 2 are done. Steps 3-6 are not, and are recorded honestly below
rather than marked complete.

### Step 1 (class D) - fixed, but not for the reason the plan gave

The plan said the fix was to drop `public` from the search path. **That was
wrong, and it was wrong because the plan trusted the error message.**

`001_initial.sql` is a pg_dump and hardcodes the schema in the object names -
1156 `public.` references in that one file, e.g.

    CREATE FUNCTION public.update_fic_tag_score() RETURNS trigger

Search path cannot redirect an explicitly qualified name. The per-test
`test_*` schema could never have been populated, which also means the schema
leak was a symptom, not the cause.

The real cause: `scripts/provision_test_db.sh` applies migrations with `psql -f`
and never records versions in `_sqlx_migrations`, so the table has 0 rows. The
test's own `sqlx::migrate::Migrator` therefore saw an empty ledger and replayed
all 41 files into an already-migrated database. The first file that creates an
object failed with 42723, which panicked while holding the shared `DB_LOCK`, so
the mutex stayed poisoned and the other six tests died with `PoisonError`.

**Six of the seven `integration` failures were one bug.** Only
`test_check_fic_blacklist` was ever a real failure.

The test no longer migrates at all - provisioning belongs to the provisioner,
and a second migration path is how the two drifted apart. Isolation comes from
`truncate_all`, which is what the tests actually relied on. `cleanup()` now
truncates rather than dropping, because after this change the schema it was
handed is `public` and `DROP SCHEMA public CASCADE` would have been
catastrophic.

Verified: `db_tests` 7/7, whole suite 31/31, 181 tables still in `public`, no
new `test_*` schema.

### Step 2 (class A) - fixed, and two latent bugs came with it

The trust-gate diagnosis in the spec was correct. Fixing it exposed two further
problems that had been hiding behind the 403s:

1. **A process-wide `OnceLock<PgPool>` shared by every test** while each test
   built its own `AppState` holding pooled connections. Later tests waited on
   the 60s acquire timeout, and *which* test failed varied per run. The pool is
   now per-caller. 6/6 in 0.9s, stable over three consecutive runs, versus 60s
   of timeouts.
2. **`#[tokio::test]` on a current-thread runtime** while `app()` builds a
   `RedisBucketLimiter` whose shadowban probe calls `block_in_place`, which
   panics there. `admin_api` already documents this exact requirement and uses
   `flavor = "multi_thread"`; `uploads_api` had never been converted.

`seed_user` also opens a dedicated connection rather than using the shared
pool, so seeding never competes with a live `AppState`.

The trust gate and `PUBLISH_MIN_TRUST` are untouched. The one test that exists
to prove the gate rejects now asserts a flat 403; it previously accepted 200
*or* 403, which passed whether or not the gate worked.

### Not yet done

- **Step 3 (class C)** `rec_curator` still seeds `works` and still violates
  `rec_user_signals_work_id_fkey`. The fix is already understood from
  `embedding_dedupe` - seed `fic_info` with `status = 'complete'`,
  `source = 'ao3'` - and is mechanical.
- **Step 4 (class B)** `forum_api` flood control and the 8-character minimum.
- **Step 5 (class E)** the two genuine behaviour questions. Deliberately
  untouched.
- **Step 6 (full re-measure)** not run since these commits.

### Step 3 (class C) - the classification was wrong, and it was production

The plan said: "`rec_curator` is repaired the same way `embedding_dedupe` was,
from `works` to `fic_info`."

**Wrong.** `rec_curator` already seeds `fic_info` at `tests/rec_curator.rs:44`
and all of its own bookmarks resolve. Nothing about it needed the
`embedding_dedupe` treatment.

It was the victim of two other suites. `user_export_api` and `bookmark_csv_api`
leave bookmarks whose `url_id` has no `fic_info` row, and `refresh_signals` -
which opens with `DELETE FROM rec_user_signals` and rebuilds the whole table in
one statement - then failed the FK, taking down signals for every user.

`bookmarks.url_id` has no foreign key. It is free text that the rest of the
codebase fills with `fic_info` slugs, so an orphan is a legal database state
rather than corruption. `work_ratings.url_id` and `reviews.url_id` have the same
shape. The reviews insert was not in the original report; the plan's instruction
to account for every `url_id` occurrence in the file turned it up, and it had
the same defect.

This needed its own spec and plan before the code changed, which it got:
`docs/specs/refresh-signals-orphan-bookmarks.md` and
`docs/PLAN-refresh-signals-orphan-bookmarks.md`. All three inserts now
`JOIN fic_info`. Verified in a rolled-back transaction that the old shape still
raises the FK error and the new one succeeds.

**The lesson, which matters more than the fix:** I classified this from the
error message - "FK violation, therefore stale test fixture" - without asking
which side of the constraint was wrong. Two of the three classes I worked this
session were misclassified that way, and both times the truth was a production
defect. The error names the violated constraint; it does not say which side is
wrong.

### Corrected status

| class | outcome |
|---|---|
| A `uploads_api` | fixed, 2/6 to 6/6, plus 2 latent bugs found |
| C `rec_curator` | fixed, 0/2 to 2/2, but as a production change with its own spec |
| D `integration` | fixed, 24/31 to 31/31, 6 of 7 failures were one bug |
| B `forum_api` | not started |
| E behaviour questions | deliberately untouched |

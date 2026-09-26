# FicNexus — implementation plan: `refresh_signals` and orphaned bookmarks

Companion to `docs/specs/refresh-signals-orphan-bookmarks.md`.

This is a production change to `src/recommender/signals.rs`, entered mid-way
through `docs/PLAN-failing-suite-classification.md` step 3 because that plan
classified the problem as a test defect and it is not one.

## Step 1 — the bookmark insert (R1)

**File:** `src/recommender/signals.rs`, ~lines 63-70.

```sql
FROM bookmarks b
WHERE b.url_id IS NOT NULL
```

becomes

```sql
FROM bookmarks b
-- `rec_user_signals.work_id` references `fic_info(id)`, but
-- `bookmarks.url_id` has no such foreign key, so it can name a row that does
-- not exist. Without this join the insert violates the FK and takes down the
-- whole signal refresh for every user, because this function rebuilds the
-- entire table.
JOIN fic_info f ON f.id = b.url_id
WHERE b.url_id IS NOT NULL
```

Do not replace this with a `LIKE` filter or a test-only prefix. A join to the
referenced table is the actual requirement, and it cannot go stale.

## Step 2 — the rating insert (R1)

**File:** same function, ~lines 83-88, which inserts `r.url_id` from
`work_ratings` into the same FK-constrained column.

```sql
FROM work_ratings r
WHERE r.url_id IS NOT NULL AND r.rating BETWEEN 1 AND 5
```

becomes

```sql
FROM work_ratings r
JOIN fic_info f ON f.id = r.url_id
WHERE r.url_id IS NOT NULL AND r.rating BETWEEN 1 AND 5
```

Check whether any other insert into `rec_user_signals` in this file has the same
shape before finishing. Search the whole file for `url_id` and account for every
occurrence.

## Step 3 — regression test (R4)

**File:** `tests/rec_curator.rs` or a new `tests/rec_signals_api.rs`. Prefer a
new file: this is a property of `refresh_signals`, not of the curator prior.

The test must:

1. Insert a `fic_info` row and a bookmark pointing at it.
2. Insert a bookmark pointing at a `fic_info` id that does **not** exist.
3. Call `refresh_signals`.
4. Assert it returns `Ok`.
5. Assert `rec_user_signals` contains a row for the resolving bookmark.

Step 5 is the one that catches a wrong fix. A fix that dropped all bookmarks
would pass steps 1-4 and silently disable the recommender's primary signal
source. Do not omit it.

`bookmarks` requires `user_id` (FK to `users`), `url_id`, `created_at`, and
`is_private`; `is_private` has a NOT NULL with no default, so set it
explicitly.

## Step 4 — gate (acceptance 4)

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
cargo build                                    # clean
cargo test --lib                               # 935 passed, 0 failed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
```

## Step 5 — prove it (acceptance 1, 2, 3)

The existing orphaned rows are the best fixture and they are already in the
database. Do not delete them to make the test pass.

```bash
psql "$DATABASE_URL" -c "select count(*) from bookmarks b left join fic_info f on f.id=b.url_id where f.id is null"
# 3 rows, before the fix

cargo test --test rec_curator -- --include-ignored --test-threads=1
# 2 passed, 0 failed
```

Confirm the same query after the fix inserts 4 rows, not 7:

```sql
SELECT count(*) FROM bookmarks b JOIN fic_info f ON f.id = b.url_id;   -- 4
```

## Step 6 — commit

`fix(recommender): refresh_signals failed on any orphaned bookmark`

Explain that `bookmarks.url_id` has no foreign key, the recommender inserted it
into a column that does, and the whole signal table is rebuilt in one statement,
so one orphan took down recommendations for every user.

Tag `refresh-signals-orphan-bookmarks-2026-09-26`, sync to
`/home/alvaro/code/rust/ficnexus`, commit the session note to `~/secondbrain`.

## Rollback

Two SQL clauses. Reverting the commit restores the previous behaviour exactly.
No schema change, no migration, no signature change.

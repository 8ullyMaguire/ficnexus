# FicNexus — implementation plan: `user_reports.category`

Companion to `docs/specs/reports-category-column.md`. Executable from this text
alone.

## Step 1 — the migration

Numbering: this repo's migrations run `001`–`092` (`ls migrations/ | sort -V |
tail -3` gives `090`, `091`, `092`), so the next free number is **093**. An
earlier note in this session claimed 141 migrations; that was wrong, and
`092_seed_locales.sql` is the highest that exists.

**New file:** `migrations/093_report_category.sql`

```sql
-- Add the report category the reports handler already validates and inserts.
--
-- src/routes/reports.rs:173 inserts into user_reports.category, but no
-- migration ever created the column, so POST /api/reports returned 500 on
-- every call. The handler validates category against five values
-- (spam|harassment|copyright|inappropriate|other), so the column is part of the
-- request contract, not a typo.
--
-- Additive only. user_reports is applied schema with production data: no
-- column is dropped, renamed or retyped, and no existing row is rewritten.
-- NOT NULL with a default is required so the migration succeeds on a table
-- that already has rows; historical rows land on 'other' for human triage,
-- which is what status/auto_status are for.

ALTER TABLE user_reports
    ADD COLUMN IF NOT EXISTS category text NOT NULL DEFAULT 'other';

ALTER TABLE user_reports
    DROP CONSTRAINT IF EXISTS user_reports_category_check;

ALTER TABLE user_reports
    ADD CONSTRAINT user_reports_category_check
    CHECK (category IN ('spam', 'harassment', 'copyright', 'inappropriate', 'other'));
```

`reason` and `category` stay distinct - `reason` is free text and already
`NOT NULL`; this migration does not touch it.

**Verify the highest existing migration number first:**

```bash
ls migrations/ | sort -V | tail -3
```

If anything above `142` exists, use the next free number instead.

## Step 2 — apply and verify against the live database

```bash
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
psql "$DATABASE_URL" -f migrations/093_report_category.sql
psql "$DATABASE_URL" -c "\d user_reports"          # category text NOT NULL DEFAULT 'other'
psql "$DATABASE_URL" -c "ALTER TABLE user_reports ADD CONSTRAINT tmp_check
    CHECK (category = 'nonsense')"                  # must FAIL: constraint holds
```

The constraint test must fail to insert. If it succeeds, the constraint is not
there and R2 is unmet.

Also confirm it applies to a **non-empty** table (R4), which is the property
that makes this safe on production:

```bash
psql "$DATABASE_URL" -c "SELECT count(*) FROM user_reports"
```

If the count is 0, insert a throwaway row, re-run the migration, and confirm it
still succeeds.

## Step 3 — regression test

**File:** `tests/forum_api.rs`, in `f5_report_forum_target`.

The test already files a report. Add the assertion that makes the fix
load-bearing - R6:

```rust
assert_eq!(s, StatusCode::OK, "report forum_post: {b}");

// Assert the classification landed, not just that the call returned 200. A
// status-only assertion passes again the moment someone drops `category` from
// the INSERT in reports.rs, which is how this bug happened in the first place.
let stored: String = sqlx::query_scalar(
    "SELECT category FROM user_reports WHERE reporter_id = $1 ORDER BY id DESC LIMIT 1",
)
.bind(reporter_id)
.fetch_one(&db)
.await
.expect("stored report category");
assert_eq!(stored, "spam", "report category must persist");
```

If the test does not currently pass a `category` field, add it to both
`json!` bodies:

```rust
json!({ "target_type": "forum_post", "target_id": post_id,
        "reason": "spam", "category": "spam" }),
```

`category` must be one of the five accepted values or the handler 400s before
reaching the insert.

## Step 4 — full gate

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export FORUM_POST_DELAY_SECS=0
cargo build                                          # clean
cargo test --lib                                     # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'  # 0
cargo test --test forum_api -- --include-ignored --test-threads=1
```

Expected: `f5_report_forum_target` green, and `f5_ban_enforcement` /
`f5_ban_lift` still green - those must now fail, if they fail at all, with
"Banned from the forum" rather than a body-length error, which is the
acceptance criterion in the spec.

One forum_api test is expected to stay red: `f3_edit_topic_author_window_and_mod`.
See section 2.3 of `docs/specs/forum-fixtures.md` - there is no author edit
window in the codebase and it needs an owner decision (R5 of that spec).

## Step 5 — commit and tag

Two commits, so the migration and the test are separately revertable:

1. `fix(db): add the user_reports.category the reports handler already inserts`
2. `test(reports): assert the category persists, not just the status`

Tag `reports-category-column-2026-09-26`. Sync to `/home/alvaro/code/rust/ficnexus`
and commit the session note to `~/secondbrain`.

## Rollback

`ALTER TABLE user_reports DROP COLUMN category` plus reverting the two commits.
The column holds no data that anything reads except the new test, so rollback
is safe - but only after the reports handler stops inserting into it, or the
500 comes back. Drop the column and the handler change together, never one
without the other.

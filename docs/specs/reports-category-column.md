# FicNexus — specification: `POST /api/reports` 500s on every call

Status: active. Written 2026-09-26, before any code change.
Found while executing `docs/PLAN-forum-fixtures.md` step 4.
Plan: `docs/PLAN-reports-category-column.md`.

## 1. Problem

`f5_report_forum_target` fails with HTTP 500 on the first report it files.
Every call to `POST /api/reports` fails this way, for every target type and
every user. This is not a test defect: the reporting endpoint is broken in
production.

## 2. Cause

`src/routes/reports.rs:173-186` inserts into a column that does not exist:

```sql
INSERT INTO user_reports
  (reporter_id, target_type, target_id, reason, details, weight, category)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id
```

The live table, from `\d user_reports`:

```
 id          | bigint
 reporter_id | integer
 target_type | text
 target_id   | integer
 reason      | text
 details     | jsonb
 status      | text
 created_at  | timestamptz
 weight      | integer
 auto_status | text
 resolved_by | integer
 resolved_at | timestamptz
```

There is no `category` column. PostgreSQL raises:

```
ERROR: column "category" of relation "user_reports" does not exist
```

`AppError` maps that to 500.

## 3. Why the schema and the code disagree

The handler validates a `category` field before inserting, so the concept is
real and deliberate, not a typo. From `src/routes/reports.rs:113-119`:

```rust
if !matches!(
    body.category.as_str(),
    "spam" | "harassment" | "copyright" | "inappropriate" | "other"
) {
    return Err(AppError::BadRequest(
        "category must be one of: spam, harassment, copyright, inappropriate, other"
            .to_string(),
    ));
}
```

**The migration that adds the column is missing.** Verified three ways:

- `grep -rn 'user_reports' migrations/*.sql | grep -i categor` returns nothing.
- The only migrations touching `user_reports` are `001_initial.sql`,
  `013_trust_levels.sql` and `078_forum_topics_posts_extend.sql`; none adds
  `category`.
- Only `ficnexus_test` exists on this host, so there is no second database to
  compare against. The evidence is the code and the migration directory, and
  they disagree.

The validation branch is reachable, so `category` is a supported part of the
request contract. A user who sends `category: "spam"` passes validation and
then gets a 500. A user who omits it gets a 400 about `category`. **There is no
input for which the endpoint succeeds.**

## 4. Requirements

**R1.** `POST /api/reports` must succeed for every documented `target_type`
(`comment`, `work`, `user`, `forum_post`, `forum_topic`) and a valid `category`.

**R2.** `category` must be constrained to the five values the handler accepts,
in the database as well as in code, so an invalid value cannot be written by
any other path.

**R3.** The migration is **additive only**: `ALTER TABLE ... ADD COLUMN`, plus
the check constraint. No column is dropped, renamed or retyped, and no existing
row is rewritten. `user_reports` is applied schema and production has data in
it.

**R4.** The column is `NOT NULL` with a default, so the migration succeeds on a
table with existing rows. `reports` is a moderation queue; existing rows have no
category, and a `NOT NULL` column without a default would fail the migration on
a non-empty table.

**R5.** `reason` and `category` stay distinct. `reason` is free text and is
already `NOT NULL`; `category` is the enumerated classification. The migration
must not merge them.

**R6.** A regression test asserts the report endpoint returns 200 **and** that
the row lands with the submitted category. A status-only assertion would pass
again the moment someone drops the column from the INSERT, silently dropping the
classification the handler validates.

## 5. Migration ordering

Numbered `142_report_category.sql`, following the highest existing. Must run
after `078_forum_topics_posts_extend.sql`, which is the last migration to touch
the table.

## 6. Non-goals

- Changing the `reports.rs` handler. Its validation and insert are consistent
  with each other and with R1-R2; the schema is what is missing.
- Backfilling a sensible category onto historical rows. They get the default and
  a human triages them, which is what the `status`/`auto_status` columns are
  for.
- The other 18 failing suites.

## 7. Acceptance

1. `f5_report_forum_target` passes.
2. `SELECT category FROM user_reports ORDER BY id DESC LIMIT 1` returns
   `spam` for the row the test just filed.
3. `INSERT ... category = 'nonsense'` is rejected by the check constraint.
4. Migration applies cleanly to a database that already has rows in
   `user_reports`.
5. 935 unit tests pass; all suites compile.

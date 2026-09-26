# FicNexus — implementation plan: missing reference data and one dead table

Companion to `docs/specs/missing-reference-data.md`. That document establishes
what is true and why; this one is the ordered procedure with the verification for
each step. Written 2026-09-26, before any code change.

## 0. Ground rules

- **Applied migrations are immutable.** This repo has 89 and they are never
  edited or renumbered. Every change is a new file, `090_`, `091_`, `092_`.
  Never "just fix 064" — the bug in 064 is historical and the fix is additive.
- `bash scripts/provision_test_db.sh` must print `SCHEMA OK` before any
  verification below means anything.
- Verify each step before the next. Four of these defects look identical from a
  distance and are not the same bug.

## 1. Fix the dead table — `090_feature_clusters_status_default.sql`

Create `migrations/090_feature_clusters_status_default.sql`:

```sql
-- 090_feature_clusters_status_default.sql
--
-- 064_roadmap_kanban.sql migrated feature_clusters.status from the 3-value
-- set (open, shipped, rejected, deferred) to the 8-value Kanban set, and it
-- correctly rewrote existing rows and re-added the CHECK constraint. It never
-- changed the column DEFAULT, which is still 'open' from 001_initial.sql:1117.
--
-- 'open' is not in the new set, so any INSERT that omits status is rejected by
-- the table's own CHECK constraint. The table cannot be written to at all:
--
--   INSERT INTO feature_clusters (representative_text, embedding)
--   VALUES ('probe', array_fill(0::real, ARRAY[768])::vector);
--   ERROR: new row ... violates check constraint "feature_clusters_status_check"
--
-- src/routes/roadmap.rs:153 selects WHERE status = 'idea', and
-- src/bin/seed_roadmap.rs exists to populate the table, so both are dead.
--
-- Fix the DEFAULT to 'idea' - the value 064 mapped 'open' to, and the value the
-- roadmap query filters on. Do not widen the CHECK to re-admit 'open'; 064
-- removed it deliberately.
ALTER TABLE feature_clusters
  ALTER COLUMN status SET DEFAULT 'idea';
```

**Verify** — the insert that previously failed must now succeed:

```bash
psql "$DATABASE_URL" -c "INSERT INTO feature_clusters
  (representative_text, embedding)
  VALUES ('probe', array_fill(0::real, ARRAY[768])::vector)
  RETURNING id, status"
```

Expect `status = idea` and one row returned. Then confirm the CHECK still
rejects a genuinely bad value:

```bash
psql "$DATABASE_URL" -c "UPDATE feature_clusters SET status='open' WHERE id=<id>"
```

Expect a CHECK violation. If `'open'` is accepted, the constraint was not what
you thought — stop and re-read the spec §2.

## 2. Seed `tag_types` — `091_seed_tag_types.sql`

The seven rows are defined by the repository's own specification,
`docs/design/SPECIFICATION.md:428`. **Use those names exactly**; `src/routes/
opds/tags.rs:104` resolves names by join and a wrong name is a visible bug.

```sql
-- 091_seed_tag_types.sql
--
-- tag_types has a hand-assigned smallint id and no sequence, and no migration
-- ever inserted a row, so it is empty on any database built from migrations/.
--
-- The contents are defined in docs/design/SPECIFICATION.md:428:
--   1 fandom, 2 character, 3 relationship, 4 freeform,
--   5 warning, 6 category, 7 rating
--
-- src/routes/opds/tags.rs serves this table over OPDS, so the gap is in a
-- shipped feature. Measured: seeding these rows takes the DB-gated suites from
-- 25 of 56 passing to 32 of 56.
--
-- Idempotent: re-running must not fail.
INSERT INTO tag_types (id, name) VALUES
  (1, 'fandom'),
  (2, 'character'),
  (3, 'relationship'),
  (4, 'freeform'),
  (5, 'warning'),
  (6, 'category'),
  (7, 'rating')
ON CONFLICT (id) DO NOTHING;
```

**Verify:**

```bash
psql "$DATABASE_URL" -tAc "select count(*) from tag_types"    # 7
psql "$DATABASE_URL" -c "select id,name from tag_types order by id"
```

Cross-check every name against `SPECIFICATION.md:428` before moving on.

## 3. Seed `locales` — `092_seed_locales.sql`

Four tables reference `locales(code)`, so all of them fail to insert today.

**Before writing this migration, establish the intended set.** Do not invent it:

```bash
grep -rn "locales" docs/design/SPECIFICATION.md docs/ | head -20
grep -rhoE "locale_code[^,)]{0,12}'[a-zA-Z-]+'" tests/ src/ | sort -u
```

At minimum `en` and `es`: `es` is required by
`tests/translation_review_api.rs`, and `en` is the obvious default. `is_rtl`
drives text direction in the UI, so it must be right per locale, not defaulted
blindly.

```sql
-- 092_seed_locales.sql
--
-- locales is referenced by work_translations, chapter_translations and
-- chapter_translation_versions via FOREIGN KEY (locale_code) REFERENCES
-- locales(code), but no migration ever inserted a row, so every one of those
-- inserts fails. src/db/queries/social.rs:656 reads the table to populate a
-- locale picker and always gets nothing.
--
-- Idempotent on the unique `code` constraint.
INSERT INTO locales (code, name, is_rtl) VALUES
  ('en', 'English',  false),
  ('es', 'Espanol',  false)
ON CONFLICT (code) DO NOTHING;
```

Extend the list only with locales the specification actually names. If the spec
does not enumerate them, seed the two above, record that in the commit, and note
the open question rather than guessing at 30 languages.

**Verify:**

```bash
psql "$DATABASE_URL" -tAc "select count(*) from locales"        # >= 2
psql "$DATABASE_URL" -c "select code,name,is_rtl from locales order by code"
psql "$DATABASE_URL" -tAc "
  select conrelid::regclass from pg_constraint
  where confrelid = 'locales'::regclass"    # the referencing tables
```

Then prove an insert into a dependent table now works:

```bash
psql "$DATABASE_URL" -c "select count(*) from work_translations"   # 0 is fine
# a translation insert with locale_code='es' must no longer raise 23503
```

## 4. Prove all three from a from-scratch database

This is the step that matters. Rebuild the database **with no test-side
seeding** — that is the entire point, since `provision_test_db.sh --seed-tag-types`
currently masks defect B.

```bash
bash scripts/provision_test_db.sh                 # no flag: migrations only
psql "$DATABASE_URL" -tAc "select count(*) from tag_types"   # must now be 7
psql "$DATABASE_URL" -tAc "select count(*) from locales"     # must now be >= 2
psql "$DATABASE_URL" -c "INSERT INTO feature_clusters
  (representative_text, embedding)
  VALUES ('probe2', array_fill(0::real, ARRAY[768])::vector)"
```

All three must hold with no flag. If `tag_types` is 0 here, migration 091 did
not run — check the migration is applied, and that the loop in
`provision_test_db.sh` did not stop early.

**Then remove the now-redundant opt-in seed** from
`provision_test_db.sh` and its long comment, since 091 makes it unnecessary.
Leaving it would suggest the gap is still open. Replace the comment with one
line noting the gap was closed by 091.

## 5. Fix the stale test SQL — `tests/embedding_dedupe_api.rs`

Per spec §5.1, the three columns do not exist anywhere in the schema, and the
test never reads them back. `word_count` really lives on `fic_info.word_count`.

In `seed_work`:

- remove `word_count`, `chapter_count`, `avg_words_per_chapter` from the column
  list and the `VALUES` tuple,
- drop the `words: i64` parameter and its `.bind(words)`,
- update the four call sites, which all pass `100000`.

The insert becomes:

```rust
let row = sqlx::query(
    r#"INSERT INTO works (canonical_title, canonical_author, created_at, updated_at)
       VALUES ($1, $2, NOW(), NOW())
       RETURNING id"#,
)
.bind(title)
.bind(author)
.fetch_one(pool)
.await
.expect("seed work");
```

Call sites at `embedding_dedupe_api.rs:80,81,127,128` drop their third argument.

**Verify:** `cargo test --test embedding_dedupe_api -- --include-ignored
--test-threads=1` — it should now seed successfully. Any assertion failure after
that is a *different* bug: report it, do not paper over it.

## 6. Fix the stale test SQL — `forum_topics.slug`

The column is `topic_slug`. Find the offending SQL:

```bash
grep -rn 'slug' tests/*.rs | grep -i 'INSERT INTO forum_topics' 
grep -rn 'INSERT INTO forum_topics' tests/*.rs
```

Replace `slug` with `topic_slug` in the column list only. Do not touch the
partial unique index `idx_forum_topics_slug`, which is correctly named for the
column it indexes on.

**Verify:** the suite's seed no longer raises 23503.

## 7. Re-measure and record

```bash
bash scripts/provision_test_db.sh
bash scripts/run_db_suites.sh 2>&1 | tail -8
```

Expected direction: defects A, B and C each unblock suites, so the pass count
should rise from 25. **Record the real number whatever it is.** If a suite that
previously passed now fails, that is a finding — investigate before committing.

Also re-run the unit suite, which must not regress:

```bash
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib 2>&1 | tail -3
# expect: 935 passed; 0 failed
```

## 8. Commit

Two commits.

- **Commit 1 — the migrations.** `feat(db): seed reference data and fix the
  feature_clusters status default`. Message must state: the four defects, the
  proof that `feature_clusters` was unwritable, that applied migrations were not
  touched, and that `090` is an additive fix for a bug in `064`.
- **Commit 2 — the stale test SQL plus the measurement.** Message lists the
  suites affected and the new pass count, and records that the
  `--seed-tag-types` opt-in was removed because 091 closes the gap.

Tag: `reference-data-2026-09-26`.

## 9. Out of scope — do not attempt

- The role/level gate assertions. They need per-test judgement about whether the
  test is stale or the refactor broke a real gate. Editing them to match current
  behaviour would destroy the only signal that something changed.
- The `cargo fmt` (547) and `cargo clippy` (599) backlogs.
- Adding `word_count` to `works`. It belongs on `fic_info`; changing that is a
  schema decision for its own spec.
- Any locale beyond what the specification names.

## Addendum, 2026-09-26 (after execution)

### What the plan got right

Steps 1-4 held exactly as written. Three migrations, applied in order, on a
from-scratch database:

- `090_feature_clusters_status_default.sql` - `ALTER COLUMN status SET DEFAULT
  'idea'`. Verified: the insert that previously failed with a CHECK violation now
  succeeds and yields `status = idea`, and the CHECK still rejects `'open'`.
- `091_seed_tag_types.sql` - the 7 rows from SPECIFICATION.md:428.
  Verified: `count(*) = 7`.
- `092_seed_locales.sql` - 6 rows. Verified: `count(*) = 6`, `is_rtl` false
  throughout.

The `--seed-tag-types` opt-in in `provision_test_db.sh` is removed, since 091
closes the gap. Both reference-data facts are now true on a database built from
`migrations/` alone, which was the point.

### Where the plan's §5.1 needed correcting

The plan said to fix `works.word_count` in the test. That part was right: the
columns do not exist anywhere and nothing read them. But fixing it exposed a
**third, distinct defect the plan did not predict**, and it is production code:

`src/services/embedding_dedupe.rs:23-51` joins `rec_embeddings.work_id` to
`works.id`. `rec_embeddings.work_id` is `varchar(128)` with an FK to
`fic_info(id)`; `works.id` is `integer`. Run against the real schema:

    ERROR:  operator does not exist: integer = character varying
    LINE 5: JOIN works wa ON wa.id = a.work_id

**`candidate_pairs` throws on every call and never has worked.** The test
revealed it only because it is one of the 56 suites that had never executed.

The test also expects `vector(2)` where the column is `vector(384)`.

**Not fixed here.** The plan's own §9 rule applies: production changes are out of
scope, and this one is not mechanical anyway. Whether `rec_embeddings` should key
off `fic_info` or `works` is a product decision - the query and its Rust types
change if it is the former, the schema and FK change if it is the latter. It is
recorded in the spec as §5.2 and needs its own spec and plan.

### Stale test SQL actually fixed

- `tests/embedding_dedupe_api.rs` - `seed_work` no longer inserts
  `word_count`/`chapter_count`/`avg_words_per_chapter` (they exist nowhere in the
  schema) and no longer takes the unused `words` argument. 4 call sites updated.
- `tests/polls_api.rs` - was naming both `topic_slug` and a nonexistent `slug`,
  binding the same value twice. Now names `topic_slug` only. **Suite now fully
  passes: 2/2.**
- `tests/scheduled_topics_api.rs` - same duplicate `slug` column and bind removed.

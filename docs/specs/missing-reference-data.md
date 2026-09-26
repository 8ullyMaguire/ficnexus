# FicNexus — specification: missing reference data and one dead table

Status: active. Written 2026-09-26, before any code change.
Follows `docs/specs/db-gated-suites.md`, which established the baseline this
work starts from: 25 of 56 DB-gated suites pass, 31 fail.
Repo: `~/code-local/rust/ficnexus`.

## 1. Why this is a separate cycle

The previous cycle repaired the test build and then measured it. It found
failures and deliberately did **not** fix them, because most were not test
problems. This document is that follow-up: the production defects the
measurement exposed, separated from the stale tests so that each gets the
judgement it needs.

Of the 31 failing suites, the failures fall into two groups that must not be
confused:

- **Production defects.** The schema constrains more than it populates, or
  contradicts itself. These break real features, not just tests. Four are
  identified here.
- **Stale test SQL.** The test writes columns that do not exist. These are test
  bugs and are handled in §5.

## 2. Defect A — `feature_clusters` cannot be inserted into at all

**Severity: highest. This table is dead in production, not just in tests.**

Proven, not inferred:

```sql
INSERT INTO feature_clusters (representative_text, embedding)
VALUES ('probe', array_fill(0::real, ARRAY[768])::vector);
ERROR: new row ... violates check constraint "feature_clusters_status_check"
DETAIL: Failing row contains (..., open, general).
```

The table's column default is `'open'`, and its own CHECK constraint rejects
`'open'`. **Any insert that omits `status` fails.** There is no way to write to
this table without explicitly passing a valid status.

### Root cause

`migrations/001_initial.sql:1117` created the CHECK as:

```sql
CHECK (status = ANY (ARRAY['open','shipped','rejected','deferred']))
```

`migrations/064_roadmap_kanban.sql` migrated the Kanban workflow. It correctly:

- rewrote existing rows (`'open'`→`'idea'`, `'deferred'`→`'long_term'`),
- dropped and re-added the CHECK with the 8 new values,
- added a `category` column.

It **did not change the column default.** `ALTER TABLE ... ADD COLUMN` was
never used for `status`, so the default from `001` is still `'open'`. The
migration is 3 of 4 steps complete, and the missing step is invisible until
something inserts a row.

### Impact

`src/routes/roadmap.rs:153` selects `WHERE status = 'idea'`, and
`src/bin/seed_roadmap.rs` exists to populate this table. Neither can work,
because no row can be created. `src/routes/consensus.rs:213` and
`src/routes/admin.rs:1289` read it and will always see an empty table.

### Fix

A new migration that changes the default to `'idea'` — the value 064 mapped
`'open'` to, and the value the roadmap query filters on. Not `'open'` with a
widened CHECK: `'open'` no longer appears in the enum, and re-adding it would
recreate the inconsistency 064 was written to remove.

## 3. Defect B — `tag_types` is empty

**Severity: high. A shipped feature reads it.**

`tag_types` has a hand-assigned `smallint` id, no sequence, and no migration
inserts any row. The repository's own specification defines the contents:

> `docs/design/SPECIFICATION.md:428` — `tag_types` (1 fandom, 2 character,
> 3 relationship, 4 freeform, 5 warning, 6 category, 7 rating)

Production reads the table: `src/routes/opds/tags.rs:41` serves it as an OPDS
resource and `:104` resolves tag type names.

### Measured impact

From the previous cycle, seeding exactly these 7 rows and changing nothing
else:

| state | suites passing |
|---|---|
| migrations only | 25 of 56 |
| plus the 7 `tag_types` rows | **32 of 56** |

### Fix

A migration inserting the 7 rows, `ON CONFLICT (id) DO NOTHING` so it is
idempotent and safe to re-run. Names must match the specification exactly;
`tags_api` and others resolve names by join.

## 4. Defect C — `locales` is empty

**Severity: high. Same shape as B.**

`locales (id, code, name, is_rtl)` is empty, and **four** tables reference
`locales(code)`: `work_translations`, `chapter_translations`,
`chapter_translation_versions`, and at least one more. Any insert into any of
them fails on the foreign key.

`src/db/queries/social.rs:656` reads `SELECT id, code, name, is_rtl FROM
locales ORDER BY id` to populate a UI list — so it returns empty.

Nothing in `src/` ever inserts a locale. The tests use `'es'`.

### Fix

A migration seeding the locales the product is documented to support. `is_rtl`
must be correct per locale — it drives text direction in the UI, so getting it
wrong is a visible bug, not a cosmetic one. At minimum `en` and `es` (the
latter is required by existing tests); the full BCP-47 set the specification
implies should be included if the spec names it — **verify before writing**.

## 5. Not production defects: stale test SQL

Two failure classes are the tests' fault and must be fixed as tests.

### 5.1 `works.word_count` does not exist

`tests/embedding_dedupe_api.rs:33-47` defines `seed_work`, inserting into
`works (canonical_title, canonical_author, word_count, chapter_count,
avg_words_per_chapter, created_at, updated_at)`.

**None of those three columns exist on `works`, or anywhere in the schema.**
Verified against `information_schema.columns`: there is no `word_count`,
`chapter_count`, or `avg_words_per_chapter` column in any table.

Where word counts actually live: **`fic_info.word_count`**
(`src/routes/admin.rs:129` reads it). The other two have no home at all.

The test does not use them. `seed_work` takes a `words: i64` parameter, binds
it into the insert, and no assertion in the file reads `word_count`,
`chapter_count`, or `avg_words_per_chapter` back. All four call sites pass
`100000`. It is vestigial.

### Fix

Drop the three columns and the `words` parameter from the insert, and update the
four call sites. Do **not** add the columns to `works`: nothing in the schema or
`src/` expects a word count there, and inventing one is a schema decision
belonging to its own spec. `fic_info.word_count` is the real home, and this test
is about embedding dedupe, not word counts.

### 5.2 `embedding_dedupe::candidate_pairs` throws on every call

**This is a production defect, not a test defect, and it is the most serious
finding of this cycle. It is NOT fixed here** - see §6 and the note below.

`src/services/embedding_dedupe.rs:23-51` runs:

```sql
FROM rec_embeddings a
JOIN rec_embeddings b ON a.model = b.model AND a.work_id < b.work_id
JOIN works wa ON wa.id = a.work_id
```

and decodes rows as `(i32, String, i32, String, f64)`.

But `rec_embeddings.work_id` is `character varying(128)`, and its foreign key is
`REFERENCES fic_info(id)`, not `works(id)`. `works.id` is `integer`. Proven
against the real schema:

```
ERROR:  operator does not exist: integer = character varying
LINE 5: JOIN works wa ON wa.id = a.work_id
```

So **every call to `candidate_pairs` fails at runtime**, and it cannot have ever
succeeded. The test caught it only because it is one of the 56 suites that had
never executed.

Two separate problems, and the second is the deeper one:

1. **The join is to the wrong table.** `rec_embeddings` hangs off `fic_info`,
   not `works`. The whole query is built on a false premise about the schema.
2. **The test expects 2-dim vectors**; the column is `vector(384)`. It seeds
   `rec_embeddings(work_id: i32, ...)` where the real key is a `varchar`.

Fixing the test to match the schema is *not* sufficient, and neither is casting.
The question the owner needs to answer is whether `rec_embeddings` is supposed
to key off `fic_info` (in which case the query must join `fic_info` and the
`EmbeddingCandidate` type changes) or off `works` (in which case the schema and
the FK are wrong). That is a product decision, not a test fix, and it is why
this is recorded rather than patched.

**Not fixed in this cycle.** Per the plan, production code changes are out of
scope here; this needs its own spec and plan.

### 5.3 `forum_topics.slug` does not exist

The column is `topic_slug`. A partial unique index exists on it
(`idx_forum_topics_slug`). The test SQL is simply wrong.

### 5.4 `role`/level gate assertions

Several suites assert that a privilege block works, using the old role
vocabulary. These need per-test judgement: is the test stale, or did the refactor
break a real gate? **Not fixable in bulk and not to be edited to match current
behaviour.**

## 6. Required outcome

1. Defect A fixed by migration, proven by a successful insert that omits
   `status`.
2. Defects B and C fixed by idempotent data migrations.
3. A schema-check script that proves all three on a database built purely from
   `migrations/` — with no test-side seeding, because that is the point.
4. §5.1 and §5.3 fixed in tests, after establishing where the columns went.
5. Suite counts re-measured and recorded, honestly, pass or fail.
6. **APPLIED migrations are immutable.** This repo has 89 of them and they are
   never edited or renumbered. All fixes are new files, numbered 090+.

## 7. Verification

```bash
cd ~/code-local/rust/ficnexus
bash scripts/provision_test_db.sh          # prints DATABASE_URL, 180+ tables

# A. the table is writable without naming status
psql "$DATABASE_URL" -c "INSERT INTO feature_clusters
  (representative_text, embedding)
  VALUES ('probe', array_fill(0::real, ARRAY[768])::vector)"

# B. reference data present
psql "$DATABASE_URL" -tAc "select count(*) from tag_types"    # 7
psql "$DATABASE_URL" -tAc "select count(*) from locales"      # >0

# all three reproducible from migrations alone
bash scripts/provision_test_db.sh   # recreates from scratch, then re-check

# suites
bash scripts/run_db_suites.sh
```

Definition of done: A, B and C provable on a from-scratch database; suite counts
recorded; no applied migration touched; `cargo build` and the 935 unit tests
still green.

## 8. Out of scope

- §5.3 role/level assertions — needs individual judgement, separate cycle.
- The `cargo fmt` and `cargo clippy` backlogs.
- Any change to `AppState`, handlers, or test fixtures beyond §5.1 and §5.2.

# FicNexus — specification: repair `embedding_dedupe::candidate_pairs`

Status: active. Written 2026-09-26, before any code change.
Recorded as a finding in `docs/specs/missing-reference-data.md` §5.2, which
declined to fix it because the right answer looked like a product decision.
This document shows it is not — the schema answers it.
Repo: `~/code-local/rust/ficnexus`.

## 1. The defect

`src/services/embedding_dedupe.rs:23-51` runs:

```sql
FROM rec_embeddings a
JOIN rec_embeddings b ON a.model = b.model AND a.work_id < b.work_id
JOIN works wa ON wa.id = a.work_id
```

and decodes rows as `(i32, String, i32, String, f64)`.

`rec_embeddings.work_id` is `character varying(128)`, and its foreign key is
`REFERENCES fic_info(id)`. `works.id` is `integer`. Proven against the real
schema:

```
ERROR:  operator does not exist: integer = character varying
LINE 5: JOIN works wa ON wa.id = a.work_id
```

**`candidate_pairs` has never returned.** Its only caller, `run_dedupe:81`,
swallows the error with `let Ok(candidates) = ... else { return (0, 0, 0) }`, so
the whole embedding-dedupe feature reports "no duplicates found" forever. It
fails silently and looks healthy.

## 2. Why this is not a product decision

The previous spec said the choice between `fic_info` and `works` was a product
decision requiring the owner. It is not. The schema answers it three times over.

### 2.1 The entire recommender stack is `fic_info`-keyed

Tables whose foreign key targets `fic_info(id)`:

    rec_embeddings, rec_bandit_arms, rec_impressions, rec_transitions,
    rec_user_signals, rec_bandit_cooccur, fic_bookmark_cooccur,
    precomputed_recommendations, recommendation_suggestions,
    comments, fic_tags, fic_works, fic_bookmarks, fic_blacklist,
    opds_shelf_items, export_log

`embedding_dedupe.rs` is the **only** file in the recommender that treats a
work_id as an integer.

### 2.2 The healthy sibling function in the same subsystem does the fic_info join

`src/recommender/embeddings.rs:63-83`, `works_needing_embedding`:

```sql
SELECT f.id, f.title, f.description, re.content_hash
FROM fic_info f
LEFT JOIN rec_embeddings re ON re.work_id = f.id AND re.model = $1
```

returning `Vec<(String, String)>` — ids as `String`, titles from `fic_info`.
This is the house pattern, and it is in the same directory tree as the broken
code.

### 2.3 The id types say what the keys are

Real rows make it unambiguous:

    fic_info.id    -> 'adminit_autotag_fic'   (a slug)
    works.id       -> 103                      (a surrogate key)

`EmbeddingCandidate.source_work_id: i32` cannot hold a slug. That is not a
style disagreement; the type cannot represent the value.

## 3. The intended query

Both halves of the broken query have a correct form that the rest of the schema
already uses.

**Titles** come from `fic_info`, keyed by the slug:

```sql
JOIN fic_info fa ON fa.id = a.work_id
JOIN fic_info fb ON fb.id = b.work_id
```

**Merge targets** are `works.id`, because that is what the consumers require.
`work_proposals.source_work_id` and `target_work_id` are `integer` with
`FOREIGN KEY ... REFERENCES works(id)`, and `execute_merge` runs
`UPDATE fic_info SET work_id = $1 WHERE work_id = $2` over `works.id`. The
bridge is `fic_info.work_id`, itself `integer REFERENCES works(id)`.

So the query must project the slug for identity and the work id for action:

```sql
SELECT
    a.work_id                AS source_url_id,
    fa.title                 AS source_title,
    fa.work_id               AS source_work_id,
    b.work_id                AS target_url_id,
    fb.title                 AS target_title,
    fb.work_id               AS target_work_id,
    1 - (a.embedding <=> b.embedding) AS similarity
```

The existing `NOT EXISTS` guard on `work_proposals` compares `source_work_id`
and `target_work_id`, which are `works.id` — so it must also use `fa.work_id` /
`fb.work_id`, not the slugs. Getting this wrong would silently stop the guard
from ever matching.

**`fic_info.work_id` is nullable.** A row with `work_id IS NULL` is an
unlinked source-site entry that cannot be merged. The query must exclude those,
or `NOT NULL`-decode them into `Option<i32>` and skip. Prefer excluding them in
SQL with `AND fa.work_id IS NOT NULL AND fb.work_id IS NOT NULL`: a pair where
either side cannot be acted on is not a candidate.

## 4. Type changes

`EmbeddingCandidate` gains the slug alongside the work id, because the slug is
the stable identity and the work id is the actionable one:

```rust
pub struct EmbeddingCandidate {
    pub source_url_id: String,   // fic_info.id, the slug
    pub target_url_id: String,
    pub source_work_id: Option<i32>,  // fic_info.work_id -> works.id
    pub target_work_id: Option<i32>,
    pub source_title: String,
    pub target_title: String,
    pub similarity: f64,
}
```

With the SQL-side `IS NOT NULL` filter from §3, the `Option` is belt-and-braces;
the caller can then `let (Some(s), Some(t)) = (...) else { continue }` and skip a
pair defensively rather than trusting the query.

`run_dedupe` passes `c.source_work_id` to `execute_merge` and
`create_proposal`, which take `i32`. That call site gains the skip.

## 5. What this feature is for, and why it was worth fixing

The module doc says it plainly:

> Secondary check after title+author fast path catches cross-site duplicates that
> the exact matcher misses (different titles, different authors).

That is the case `works.canonical_title` normalisation *cannot* catch, and it is
the one that most needs a human or an auto-merge. The feature has been
contributing nothing. Auto-merges at similarity ≥ 0.98 have never fired.

## 6. The test

`tests/embedding_dedupe_api.rs` is written against the same non-existent
schema, so it must be corrected in the same change — otherwise the fix is
unverifiable.

Its `seed_work` must target `fic_info`, not `works`. `fic_info` requires
`title`, `author`, `chapters`, `words`, `description`, `fic_created`,
`fic_updated` (all `NOT NULL`); `words` is where the word count the old test was
reaching for actually lives.

Its `seed_embedding` must insert `vector(384)`, not `vector(2)`, and bind a
`String` work_id.

**Build a 384-dimensional vector** rather than writing 384 numbers by hand — a
small helper that formats `"[" || "0.1," x 383 || "0.1]"`, with the first and
last components varied so cosine distance is meaningful and the similarity
threshold is actually exercised. A vector of near-identical entries would make
every pair score ~1.0 and the test would assert nothing.

Use two similar-but-distinct 384-dim vectors so the pair is found at a
reasonable threshold, and assert `source_work_id`/`target_work_id` are the
`works.id` values (integers) while the url ids are the slugs.

## 7. Required outcome

1. `candidate_pairs` executes against the real schema. Verified by the suite
   passing, and by a direct `psql` run of the new query.
2. `EmbeddingCandidate` carries both identities; `run_dedupe` skips unmergeable
   pairs instead of erroring.
3. The test seeds `fic_info` with 384-dim vectors and passes.
4. `run_dedupe` no longer swallows this class of error into a silent
   `(0, 0, 0)`. **At minimum, log it** — a feature that has been silently dead
   must not be silent again. Preferred: keep the tolerant return for the
   caller, but `tracing::error!` the cause.
5. `cargo build` clean, 935 unit tests still pass, all 56 suites still compile.

## 8. Verification

```bash
cd ~/code-local/rust/ficnexus
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'

# the new query runs against the real schema
psql "$DATABASE_URL" -f /tmp/candidate_pairs.sql

cargo build
cargo test --lib                                     # expect 935 pass
cargo test --test embedding_dedupe_api -- --include-ignored --test-threads=1
```

## 9. Out of scope

- The 24 other failing suites, and the role/level gate assertions.
- Whether `works` and `fic_info` should be reconciled more broadly. This fix
  uses the existing `fic_info.work_id` bridge; it does not redesign the model.
- The `cargo fmt` and `cargo clippy` backlogs.

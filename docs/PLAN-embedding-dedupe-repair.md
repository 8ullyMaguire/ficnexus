# FicNexus — implementation plan: repair `embedding_dedupe::candidate_pairs`

Companion to `docs/specs/embedding-dedupe-repair.md`. That document establishes
what is true and why; this one is the ordered procedure with the verification for
each step. Written 2026-09-26, before any code change.

## 0. Ground rules

- `bash scripts/provision_test_db.sh` must print `SCHEMA OK` first.
- The corrected SQL in §1 has **already been run against the real schema and
  executes cleanly** (returns 0 rows because no embeddings exist yet, not an
  error). That is the proof the approach is right. If you change the SQL, re-run
  it with `psql "$DATABASE_URL" -f /tmp/candidate_pairs.sql` before touching
  Rust.
- This is the only production-code change in this cycle. Keep it to
  `src/services/embedding_dedupe.rs` and `tests/embedding_dedupe_api.rs`.

## 1. Rewrite `candidate_pairs`

In `src/services/embedding_dedupe.rs`, replace the query at lines 28-49.

**The `works` join is removed entirely.** Titles come from `fic_info`; the
actionable work ids come from `fic_info.work_id`, which is
`integer REFERENCES works(id)`.

```rust
    let rows = sqlx::query_as::<_, (String, String, Option<i32>, String, String, Option<i32>, f64)>(
        r#"SELECT
            a.work_id                       AS source_url_id,
            fa.title                        AS source_title,
            fa.work_id                      AS source_work_id,
            b.work_id                       AS target_url_id,
            fb.title                        AS target_title,
            fb.work_id                      AS target_work_id,
            1 - (a.embedding <=> b.embedding) AS similarity
        FROM rec_embeddings a
        JOIN rec_embeddings b
          ON a.model = b.model AND a.work_id < b.work_id
        JOIN fic_info fa ON fa.id = a.work_id
        JOIN fic_info fb ON fb.id = b.work_id
        WHERE fa.work_id IS NOT NULL
          AND fb.work_id IS NOT NULL
          AND 1 - (a.embedding <=> b.embedding) > $1
          AND NOT EXISTS (
            SELECT 1 FROM work_proposals wp
            WHERE wp.action_type = 'merge'
              AND wp.status = 'pending'
              AND ((wp.source_work_id = fa.work_id AND wp.target_work_id = fb.work_id)
                OR (wp.source_work_id = fb.work_id AND wp.target_work_id = fa.work_id))
          )
        ORDER BY similarity DESC
        LIMIT $2"#,
    )
```

Two things that are easy to get wrong and are called out in the spec:

- The `NOT EXISTS` guard must compare `fa.work_id` / `fb.work_id`, **not** the
  slugs. `work_proposals.source_work_id` is `integer REFERENCES works(id)`.
  Comparing slugs would silently stop the guard from ever matching, and nothing
  would fail visibly.
- `WHERE fa.work_id IS NOT NULL AND fb.work_id IS NOT NULL` is required.
  `fic_info.work_id` is nullable, and an unlinked source-site entry has no
  `works` row to merge into.

Then the mapping, per spec §4:

```rust
    Ok(rows
        .into_iter()
        .map(|(src_url, src_title, src_work, tgt_url, tgt_title, tgt_work, sim)| {
            EmbeddingCandidate {
                source_url_id: src_url,
                target_url_id: tgt_url,
                source_work_id: src_work,
                target_work_id: tgt_work,
                source_title: src_title,
                target_title: tgt_title,
                similarity: sim,
            }
        })
        .collect())
```

## 2. Change `EmbeddingCandidate`

Per spec §4. `source_work_id`/`target_work_id` become `Option<i32>`; add
`source_url_id`/`target_url_id: String`. Update the doc comment above the struct
to say ids are `fic_info` slugs and the work ids are `works.id` values bridged
through `fic_info.work_id`.

## 3. Fix the silent failure in `run_dedupe`

`run_dedupe:81` currently reads:

```rust
    let Ok(candidates) = candidate_pairs(db, threshold, limit).await else {
        return (0, 0, 0);
    };
```

This is why a feature that has never worked looks healthy. Keep the tolerant
return — the caller may not want to fail the whole dedupe run — but log it:

```rust
    let candidates = match candidate_pairs(db, threshold, limit).await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(
                "embedding dedupe: candidate_pairs failed, reporting no duplicates: {err}"
            );
            return (0, 0, 0);
        }
    };
```

**This is the part that matters most.** A feature silently returning "nothing
found" is worse than one that errors: it looks like a successful run. It must
not be silent again.

## 4. Update the call site in `run_dedupe`

`execute_merge` and `create_proposal` take `i32`. The loop now has `Option`s:

```rust
    for c in &candidates {
        let (Some(source_work_id), Some(target_work_id)) =
            (c.source_work_id, c.target_work_id)
        else {
            // Unlinked source-site entry: no works row to merge into.
            continue;
        };
```

and use `source_work_id` / `target_work_id` in place of `c.source_work_id` /
`c.target_work_id` in the `execute_merge` and `create_proposal` calls and in the
`tracing::info!`.

## 5. Correct the test — `tests/embedding_dedupe_api.rs`

The test is written against the same non-existent schema, so it must change in
the same commit or the fix is unverifiable.

`seed_work` must target `fic_info`, which requires `title`, `author`,
`chapters`, `words`, `description`, `fic_created`, `fic_updated` (all NOT NULL),
plus a `work_id` when the pair must be mergeable:

```rust
async fn seed_fic(pool: &sqlx::PgPool, slug: &str, title: &str, author: &str, work_id: Option<i32>) -> String {
    use sqlx::Row;
    let row = sqlx::query(
        r#"INSERT INTO fic_info
             (id, title, author, chapters, words, description, fic_created, fic_updated, work_id)
           VALUES ($1, $2, $3, 1, 1000, 'desc', NOW(), NOW(), $4)
           ON CONFLICT (id) DO UPDATE SET work_id = EXCLUDED.work_id
           RETURNING id"#,
    )
    .bind(slug)
    .bind(title)
    .bind(author)
    .bind(work_id)
    .fetch_one(pool)
    .await
    .expect("seed fic_info");
    row.get("id")
}
```

`seed_embedding` must insert `vector(384)` and bind a `String`:

```rust
/// A deterministic 384-dim vector. The first component is `bias`, so two calls
/// with different biases produce genuinely different vectors and the cosine
/// threshold is actually exercised.
fn vec384(bias: f64) -> String {
    let mut parts = Vec::with_capacity(384);
    for i in 0..384 {
        parts.push(if i == 0 { bias } else { 0.01 });
    }
    format!("[{}]", parts.join(","))
}
```

**Why this shape matters.** The old test used `vector(2)`. A 384-dim vector whose
components are all near-identical would make every pair score ~1.0, so every
threshold assertion would pass trivially and the test would prove nothing.
Varying the leading component gives a real, predictable distance.

Then, in each test: seed two `fic_info` rows with distinct slugs and **distinct
`work_id`s** (both non-NULL, or the new `IS NOT NULL` filter excludes them), give
them near-identical embeddings, and assert the pair is found. Assert that
`source_url_id`/`target_url_id` are the slugs and `source_work_id`/
`target_work_id` are the integer work ids.

The cleanup `DELETE FROM rec_embeddings WHERE model = 'test'` stays valid.

**Verify:**

```bash
export DATABASE_URL='postgres://fichub:fichub@127.0.0.1/ficnexus_test'
export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='fichub-test-secret'
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --test embedding_dedupe_api \
  -- --include-ignored --test-threads=1
```

Expect `2 passed; 0 failed`. If it still fails, read the panic before changing
anything — the two likeliest causes are a dimension mismatch and a NULL
`work_id` being filtered out.

## 6. Gate everything

```bash
cd ~/code-local/rust/ficnexus
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo build 2>&1 | grep -E '^error|Finished'
CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --lib 2>&1 | tail -3
# expect: 935 passed; 0 failed
touch tests/*.rs && CARGO_TARGET_DIR=~/.cargo-target/ficnexus cargo test --no-run 2>&1 \
  | grep -cE 'could not compile `fichub` \(test '
# expect: 0
git diff --name-only     # expect only src/services/embedding_dedupe.rs
                         # and tests/embedding_dedupe_api.rs
```

`touch tests/*.rs` first, or cargo replays cached results and the count lies.

## 7. Re-measure the suite

```bash
bash scripts/provision_test_db.sh
bash scripts/run_db_suites.sh 2>&1 | tail -5
```

Record the real number. Baseline to beat: **31 of 56 passing, 80 individual
failures**. `embedding_dedupe_api` should join the passing set.

If any suite that passed before now fails, that is a finding — investigate before
committing, do not assume it is unrelated.

## 8. Commit

One commit. Message must state:

- the defect and its proof (`operator does not exist: integer = character
  varying`), and that the feature had never returned
- that `run_dedupe` swallowed the error into a silent `(0, 0, 0)`, so it read as
  a healthy no-op
- that this is not a product decision: the whole recommender stack keys off
  `fic_info`, the sibling function in the same subsystem already joins it, and
  the real id values are slugs that `i32` cannot hold
- the test correction: `fic_info` instead of `works`, `vector(384)` instead of
  `vector(2)`
- the new suite count

Tag: `embedding-dedupe-repair-2026-09-26`.

Update `README.md`'s testing table with the new counts, and correct the
`embedding_dedupe` paragraph in it, which currently describes this as an
outstanding bug.

## 9. Definition of done

- [ ] `candidate_pairs` executes against the real schema
- [ ] `run_dedupe` logs instead of silently returning `(0, 0, 0)`
- [ ] `embedding_dedupe_api` passes 2/2
- [ ] `cargo build` clean, 935 unit tests pass, 56 suites compile
- [ ] suite counts re-measured and recorded
- [ ] README updated, committed, tagged
- [ ] `git diff --name-only` shows only the two intended files

## 10. Out of scope

- The other 24 failing suites, and the role/level gate assertions.
- Reconciling `works` and `fic_info` more broadly. This uses the existing
  `fic_info.work_id` bridge; it does not redesign the model.
- Any change to `rec_embeddings`' schema. The schema is right; the query was
  wrong.

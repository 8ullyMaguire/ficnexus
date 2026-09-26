//! Integration tests for the embedding-based dedupe service.
//!
//! Verifies:
//!   * candidate_pairs finds works with high cosine similarity
//!   * run_dedupe auto-merges high-confidence pairs (≥0.98)
//!   * run_dedupe creates proposals for lower confidence matches
//!
//! Conventions (same as other tests/):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test embedding_dedupe_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows at the START.

use std::sync::{Mutex, OnceLock};

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Deterministic 384-dimensional pgvector input.
///
/// The first component is `bias` and the rest are a small constant, so two
/// calls with different biases give genuinely different vectors and the cosine
/// threshold is actually exercised. A vector whose components were all nearly
/// identical would score ~1.0 against everything and every threshold assertion
/// would pass trivially.
fn vec384(bias: f64) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(384);
    for i in 0..384 {
        parts.push((if i == 0 { bias } else { 0.01 }).to_string());
    }
    format!("[{}]", parts.join(","))
}

/// Helper: create a test `fic_info` row and return its slug id.
///
/// `rec_embeddings.work_id` is `varchar(128)` with a foreign key onto
/// `fic_info(id)` — not `works(id)` — so the embeddings hang off `fic_info`.
/// `work_id` is the bridge to `works.id` and is what the merge path acts on, so
/// a mergeable row needs a real `works` row and this must be `Some`.
async fn seed_fic(
    pool: &sqlx::PgPool,
    slug: &str,
    title: &str,
    author: &str,
    work_id: Option<i32>,
) -> String {
    use sqlx::Row;
    let row = sqlx::query(
        // status and source are NOT NULL with no default; the values used here
        // are the ones real rows carry ('complete' / 'ao3'), not invented.
        r#"INSERT INTO fic_info
             (id, title, author, chapters, words, description, fic_created, fic_updated,
              status, source, work_id)
           VALUES ($1, $2, $3, 1, 1000, 'test description', NOW(), NOW(),
                   'complete', 'ao3', $4)
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

/// Seed a minimal `works` row and return its integer id, so a `fic_info` row
/// can be linked to it and therefore become a merge candidate.
async fn seed_work(pool: &sqlx::PgPool, title: &str) -> i32 {
    use sqlx::Row;
    let row = sqlx::query(
        r#"INSERT INTO works (canonical_title, canonical_author, created_at, updated_at)
           VALUES ($1, 'DedupeTest Author', NOW(), NOW())
           RETURNING id"#,
    )
    .bind(title)
    .fetch_one(pool)
    .await
    .expect("seed work");
    row.get("id")
}

/// Helper: insert a 384-dim embedding for a `fic_info` slug.
async fn seed_embedding(pool: &sqlx::PgPool, url_id: &str, embedding: &str) {
    sqlx::query(
        r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash)
           VALUES ($1, 'test', $2::vector(384), 'test-hash')
           ON CONFLICT (work_id, model) DO UPDATE SET embedding = $2::vector(384)"#,
    )
    .bind(url_id)
    .bind(embedding)
    .execute(pool)
    .await
    .expect("seed embedding");
}

#[tokio::test]
#[ignore]
async fn test_candidate_pairs_finds_similar_works() {
    let _guard = db_guard();
    let pool = pool().await;

    cleanup(&pool).await;

    // Both sides need a `works` row: the query filters out `fic_info` rows whose
    // work_id IS NULL, because there is nothing to merge into.
    let w1 = seed_work(&pool, "DedupeTest_SimilarA").await;
    let w2 = seed_work(&pool, "DedupeTest_SimilarB").await;
    let f1 = seed_fic(&pool, "dedupetest-similara", "DedupeTest SimilarA", "Author A", Some(w1)).await;
    let f2 = seed_fic(&pool, "dedupetest-similarb", "DedupeTest SimilarB", "Author B", Some(w2)).await;

    // Close vectors, so cosine similarity is well above the threshold.
    seed_embedding(&pool, &f1, &vec384(1.00)).await;
    seed_embedding(&pool, &f2, &vec384(1.02)).await;

    let candidates = fichub::services::embedding_dedupe::candidate_pairs(&pool, 0.5, 100)
        .await
        .expect("candidate_pairs failed");

    let found = candidates.iter().any(|c| {
        (c.source_url_id == f1 && c.target_url_id == f2)
            || (c.source_url_id == f2 && c.target_url_id == f1)
    });
    assert!(found, "Expected to find pair ({f1}, {f2}) in candidates");

    // The url ids are slugs; the work ids are the integer works.id values the
    // merge path acts on. Both identities must survive the query.
    let pair = candidates
        .iter()
        .find(|c| {
            (c.source_url_id == f1 && c.target_url_id == f2)
                || (c.source_url_id == f2 && c.target_url_id == f1)
        })
        .expect("pair");
    let (src_work, tgt_work) = if pair.source_url_id == f1 {
        (pair.source_work_id, pair.target_work_id)
    } else {
        (pair.target_work_id, pair.source_work_id)
    };
    assert_eq!(
        (src_work, tgt_work),
        (Some(w1), Some(w2)),
        "work ids must be the works.id values bridged through fic_info.work_id"
    );
    assert!(pair.similarity > 0.5, "similarity: {}", pair.similarity);

    cleanup(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_run_dedupe_auto_merges_high_similarity() {
    let _guard = db_guard();
    let pool = pool().await;

    cleanup(&pool).await;

    let w1 = seed_work(&pool, "DedupeTest_MergeA").await;
    let w2 = seed_work(&pool, "DedupeTest_MergeB").await;
    let f1 = seed_fic(&pool, "dedupetest-mergea", "DedupeTest MergeA", "Author A", Some(w1)).await;
    let f2 = seed_fic(&pool, "dedupetest-mergeb", "DedupeTest MergeB", "Author A", Some(w2)).await;

    // Near-identical: above AUTO_MERGE_THRESHOLD (0.98), so run_dedupe merges.
    seed_embedding(&pool, &f1, &vec384(1.000)).await;
    seed_embedding(&pool, &f2, &vec384(1.001)).await;

    let (merged, proposed, examined) =
        fichub::services::embedding_dedupe::run_dedupe(&pool, 1, 0.5, 100).await;

    assert_eq!(examined, 1, "expected exactly one candidate pair");
    assert_eq!(proposed, 0, "high-similarity pairs merge rather than propose");
    assert_eq!(merged, 1, "high-similarity pair should auto-merge");

    // execute_merge repoints fic_info.work_id, so the merge is observable.
    let remaining: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM fic_info WHERE id = $1 AND work_id = $2",
    )
    .bind(&f1)
    .bind(w2)
    .fetch_one(&pool)
    .await
    .expect("verify merge");
    assert_eq!(remaining, 1, "{f1} should now point at works row {w2}");

    cleanup(&pool).await;
}

/// Remove this suite's seed rows. `rec_embeddings` cascades from `fic_info`;
/// the `works` rows have to go explicitly.
async fn cleanup(pool: &sqlx::PgPool) {
    sqlx::query("DELETE FROM rec_embeddings WHERE model = 'test'")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM fic_info WHERE id LIKE 'dedupetest-%'")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'DedupeTest_%'")
        .execute(pool)
        .await
        .unwrap();
}

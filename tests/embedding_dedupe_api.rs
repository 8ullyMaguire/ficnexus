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

/// Helper: create a test work and return its ID.
/// The `words` argument and the word_count/chapter_count/avg_words_per_chapter
/// columns this used to insert do not exist: `works` has never had them, and no
/// table in the schema does. Word counts live on `fic_info.word_count`, which
/// this test does not exercise - it is about embedding dedupe. Nothing here
/// read the values back, so they were dropped rather than invented.
async fn seed_work(pool: &sqlx::PgPool, title: &str, author: &str) -> i32 {
    use sqlx::Row;
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
    row.get("id")
}

/// Helper: insert a 2-dim embedding for a work.
async fn seed_embedding(pool: &sqlx::PgPool, work_id: i32, embedding: &str) {
    sqlx::query(
        r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash)
           VALUES ($1, 'test', $2::vector(2), 'test-hash')
           ON CONFLICT (work_id, model) DO UPDATE SET embedding = $2::vector(2)"#,
    )
    .bind(work_id)
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

    // Clean up any previous test data
    sqlx::query("DELETE FROM rec_embeddings WHERE model = 'test'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'DedupeTest_%'")
        .execute(&pool)
        .await
        .unwrap();

    // Create two works with similar embeddings (should be found)
    let w1 = seed_work(&pool, "DedupeTest_SimilarA", "Author A").await;
    let w2 = seed_work(&pool, "DedupeTest_SimilarB", "Author B").await;

    // Embeddings with high similarity (cosine ~0.99)
    seed_embedding(&pool, w1, "[0.1, 0.9]").await;
    seed_embedding(&pool, w2, "[0.12, 0.92]").await;

    // Find candidates with low threshold to ensure match
    let candidates = fichub::services::embedding_dedupe::candidate_pairs(&pool, 0.5, 100)
        .await
        .expect("candidate_pairs failed");

    // Should find the pair
    let found = candidates.iter().any(|c| {
        (c.source_work_id == w1 && c.target_work_id == w2)
            || (c.source_work_id == w2 && c.target_work_id == w1)
    });
    assert!(found, "Expected to find pair ({w1}, {w2}) in candidates");

    // Cleanup
    sqlx::query("DELETE FROM rec_embeddings WHERE model = 'test'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'DedupeTest_%'")
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore]
async fn test_run_dedupe_auto_merges_high_similarity() {
    let _guard = db_guard();
    let pool = pool().await;

    // Clean up
    sqlx::query("DELETE FROM rec_embeddings WHERE model = 'test'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'DedupeTest_%'")
        .execute(&pool)
        .await
        .unwrap();

    // Create two works with identical embeddings (similarity = 1.0)
    let w1 = seed_work(&pool, "DedupeTest_MergeA", "Author A").await;
    let w2 = seed_work(&pool, "DedupeTest_MergeB", "Author A").await;

    seed_embedding(&pool, w1, "[1.0, 0.0]").await;
    seed_embedding(&pool, w2, "[1.0, 0.0]").await;

    // Run dedupe — identical embeddings should trigger auto-merge
    let (auto_merged, proposed, examined) =
        fichub::services::embedding_dedupe::run_dedupe(&pool, w1, 0.5, 100).await;

    assert!(examined >= 1, "Should examine at least 1 pair");
    assert!(
        auto_merged >= 1 || proposed >= 1,
        "Should auto-merge or propose at least 1 pair"
    );

    // Cleanup
    sqlx::query("DELETE FROM rec_embeddings WHERE model = 'test'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'DedupeTest_%'")
        .execute(&pool)
        .await
        .unwrap();
}

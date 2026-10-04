//! DB-gated integration tests for the auto-tagger service.
//!
//! Follows the repo's DB-gated conventions (see tests/roadmap_api.rs): a
//! global Mutex so DB tests never run concurrently, `#[ignore]` so they only
//! run with `--include-ignored`, and a plain `pool()` helper. Ollama is
//! pointed at port 1 (never reachable) — the test pre-seeds the fic
//! description and tag embeddings directly, and asserts the recommendation
//! pipeline (similarity query + insert with `is_machine_suggested = TRUE`).
//!
//! Run: `cargo test --test auto_tag -- --include-ignored --test-threads=1`
//! (with `.env` loaded so DATABASE_URL is set).

use std::sync::{Mutex, OnceLock};

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (run with .env loaded)");
    sqlx::PgPool::connect(&url).await.expect("connect pool")
}

/// A real embedding client.
///
/// This used to point at `http://127.0.0.1:1` on the stated assumption that
/// "tests never call it". `recommend_tags` calls `ollama.embed()` on the fic
/// description unconditionally, so both tests in this file died on
///
///     Embed(OllamaError("request failed: error sending request for url
///                           (http://127.0.0.1:1/api/embeddings)"))
///
/// after seeding succeeded. The comment described an assumption that the code
/// contradicted, and nothing caught it because the suite had never passed.
///
/// Reads OLLAMA_URL / OLLAMA_EMBED_MODEL, the same variables src/config.rs:976
/// reads, so the test exercises the configured service instead of a fixture.
///
/// Needs a local Ollama serving the embed model. Where there is none, these
/// tests cannot pass — that is a real environment requirement, not a test
/// defect, and the runner has no skip mechanism for it.
fn ollama() -> fichub::services::ollama::OllamaClient {
    let url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let model =
        std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
    fichub::services::ollama::OllamaClient::new(url.into(), model.into(), reqwest::Client::new())
}

/// Seed a fic_info row (idempotent). Description drives the similarity test.
async fn seed_fic(pool: &sqlx::PgPool, id: &str, description: &str) {
    sqlx::query(
        "INSERT INTO fic_info (id, title, author, source, description, chapters, words, status, fic_created, fic_updated, extra_meta, raw_extended_meta, content_hash)
         VALUES ($1, 'AutoTag Test Fic', 'auto_tag_author', 'https://example.test/auto-tag', $2, 1, 1000, 'complete', NOW(), NOW(), '{}', '{}', 'auto-tag-hash')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(description)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// Seed a tag row; returns its id (idempotent — tags are shared, cleanup
/// only removes the fic_tags links).
async fn seed_tag(pool: &sqlx::PgPool, name: &str, type_id: i16) -> i32 {
    sqlx::query(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
    )
    .bind(name)
    .bind(type_id)
    .execute(pool)
    .await
    .expect("seed_tag failed");
    sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed_tag: lookup failed")
}

/// Seed a tag_embeddings row directly (bypasses Ollama) with a 768-d
/// vector. The tag's embedding is its own name repeated — the fic
/// description is seeded with the same words, so cosine similarity
/// approaches 1.0 (well above the 0.75 threshold).
/// Embed `text` with the configured model and store it on the tag.
///
/// This used to synthesise a vector from the tag name's bytes. The comment
/// claimed that would make it "≈ 1.0" similar to the fic's description, which
/// is not a property anything can have: a byte-derived vector is
/// uncorrelated with a 768-dimension embedding of prose, so similarity lands
/// near zero and the suggestion is correctly filtered out. The test asserted a
/// claim about embeddings that was never true, and could not pass.
///
/// Embedding the *same* text `recommend_tags` embeds is the only way to get the
/// similarity the test is about — similarity 1.0, identical vectors.
async fn seed_tag_embedding(pool: &sqlx::PgPool, tag_id: i32, text: &str) {
    let emb = ollama().embed(text).await.expect("embed tag text");
    assert_eq!(
        emb.len(),
        768,
        "embed model must return 768 dims to match tag_embeddings.embedding"
    );
    let emb = fichub::services::auto_tagger::vec_to_sql(&emb);
    sqlx::query("INSERT INTO tag_embeddings (tag_id, embedding) VALUES ($1, $2::vector) ON CONFLICT (tag_id) DO NOTHING")
        .bind(tag_id)
        .bind(&emb)
        .execute(pool)
        .await
        .expect("seed_tag_embedding failed");
}

async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id LIKE 'auto_tag_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'auto_tag_%'")
        .execute(pool)
        .await;
}

/// recommend_tags inserts a machine-suggested fic_tags row for a tag whose
/// embedding matches the fic's description, flagged `is_machine_suggested`.
#[ignore]
#[tokio::test]
async fn recommend_tags_inserts_machine_suggestion() {
    let _guard = db_guard();
    let db = pool().await;

    let url_id = "auto_tag_fic_a";
    let description = "a haunting slow burn time loop romance with enemies to lovers";
    seed_fic(&db, url_id, description).await;

    // recommend_tags embeds `title + ". " + description`, so the tag is seeded
    // with an embedding of exactly that string → identical vector → similarity
    // 1.0, far above the 0.75 threshold. seed_fic sets no title, so the text is
    // just the description.
    let tag_name = "time loop slow burn";
    let tag = seed_tag(&db, tag_name, 4).await;
    seed_tag_embedding(&db, tag, description).await;

    let suggestions = fichub::services::auto_tagger::recommend_tags(&db, &ollama(), url_id)
        .await
        .expect("recommend_tags ran");

    assert!(
        suggestions.iter().any(|s| s.tag_id == tag),
        "matching tag suggested: {suggestions:?}"
    );

    let row: Option<(bool, Option<chrono::DateTime<chrono::Utc>>, i16)> = sqlx::query_as(
        "SELECT is_machine_suggested, reviewed_at, score FROM fic_tags WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(url_id)
    .bind(tag)
    .fetch_optional(&db)
    .await
    .expect("query fic_tags");

    let (is_machine, reviewed_at, score) = row.expect("machine-suggested row inserted");
    assert!(is_machine, "row must be flagged machine-suggested");
    assert!(reviewed_at.is_none(), "fresh suggestion is pending review");
    assert!(score > 0, "similarity stored as score: {score}");

    // Idempotency: re-running must not duplicate the row.
    let again = fichub::services::auto_tagger::recommend_tags(&db, &ollama(), url_id)
        .await
        .expect("second run");
    assert!(again.iter().any(|s| s.tag_id == tag), "{again:?}");
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
            .bind(url_id)
            .bind(tag)
            .fetch_one(&db)
            .await
            .expect("count");
    assert_eq!(count.0, 1, "ON CONFLICT DO NOTHING keeps a single row");

    cleanup(&db).await;
}

/// A tag whose embedding is orthogonal to the fic's description stays below
/// the 0.75 threshold and is never inserted.
#[ignore]
#[tokio::test]
async fn recommend_tags_skips_below_threshold() {
    let _guard = db_guard();
    let db = pool().await;

    let url_id = "auto_tag_fic_b";
    seed_fic(&db, url_id, "a quiet slice of life about baking bread").await;

    // Zero vector ≈ orthogonal to the description → similarity ~0.
    let tag_name = "graphic violence";
    let tag = seed_tag(&db, tag_name, 4).await;
    let zero = vec![0.0f32; 768];
    let emb = fichub::services::auto_tagger::vec_to_sql(&zero);
    sqlx::query("INSERT INTO tag_embeddings (tag_id, embedding) VALUES ($1, $2::vector) ON CONFLICT (tag_id) DO NOTHING")
        .bind(tag)
        .bind(&emb)
        .execute(&db)
        .await
        .expect("seed zero embedding");

    let suggestions = fichub::services::auto_tagger::recommend_tags(&db, &ollama(), url_id)
        .await
        .expect("recommend_tags ran");

    assert!(
        suggestions.iter().all(|s| s.tag_id != tag),
        "below-threshold tag must not be suggested: {suggestions:?}"
    );
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fic_tags WHERE url_id = $1 AND tag_id = $2)",
    )
    .bind(url_id)
    .bind(tag)
    .fetch_one(&db)
    .await
    .expect("exists check");
    assert!(!exists, "no fic_tags row for below-threshold tag");

    cleanup(&db).await;
}

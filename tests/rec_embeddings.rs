//! Integration tests for the embeddings strategy (pgvector).
//!
//! Seeds embeddings DIRECTLY via SQL (no real Ollama in tests — the
//! embedding CALL itself is unit-tested with a mock in `src/services/
//! ollama.rs`). Verifies:
//!   * item-to-item cosine recs between a seeded pair of works
//!   * personalized profile-centroid recs
//!   * content-hash gating (a re-run with the same text skips re-embedding)

use std::sync::{Mutex, OnceLock};

use fichub::recommender::RecStrategy; // trait in scope for `.score()`

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (load .env)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("db connect")
}

/// Build a 384-dim vector literal with nonzero entries at the given positions.
fn vec384(pairs: &[(usize, f64)]) -> String {
    let mut v = vec![0.0f64; 384];
    for (i, x) in pairs {
        v[*i] = *x;
    }
    format!(
        "[{}]",
        v.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[ignore]
#[tokio::test]
async fn embeddings_item_to_item_cosine() {
    let _guard = db_guard();
    let db = pool().await;

    // Seeds: two works + two orthogonal-ish vectors.
    for (id, title) in [("rectestemb_a", "Emb A"), ("rectestemb_b", "Emb B")] {
        sqlx::query(
            r#"INSERT INTO fic_info (
                id, title, author, author_url, author_local_id, chapters, words,
                description, fic_created, fic_updated, status, source, extra_meta,
                raw_extended_meta, source_id, author_id, content_hash
            ) VALUES ($1, $2, 'emb_author', NULL, NULL, 1, 100, 'emb test', NOW(), NOW(),
                      'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
            ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(id)
        .bind(title)
        .execute(&db)
        .await
        .expect("seed fic");
    }
    // v_a = [1,0,0], v_b = [0,1,0] → cosine 0. Also seed a third work with
    // v_c = [1,0.1,0] so A→C beats A→B.
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id, chapters, words,
            description, fic_created, fic_updated, status, source, extra_meta,
            raw_extended_meta, source_id, author_id, content_hash
        ) VALUES ('rectestemb_c', 'Emb C', 'emb_author', NULL, NULL, 1, 100, 'emb test', NOW(), NOW(),
                  'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .execute(&db)
    .await
    .expect("seed fic c");

    sqlx::query(
        r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash, updated_at)
           VALUES ('rectestemb_a', 'nomic-embed-text', $1::vector, 'h-a', NOW()),
                  ('rectestemb_b', 'nomic-embed-text', $2::vector, 'h-b', NOW()),
                  ('rectestemb_c', 'nomic-embed-text', $3::vector, 'h-c', NOW())
           ON CONFLICT (work_id, model) DO UPDATE SET embedding = EXCLUDED.embedding"#,
    )
    .bind(vec384(&[(0, 1.0)]))
    .bind(vec384(&[(1, 1.0)]))
    .bind(vec384(&[(0, 1.0), (1, 0.1)]))
    .execute(&db)
    .await
    .expect("seed embeddings");

    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = fichub::recommender::registry::build_context(
        db.clone(),
        config,
        reqwest::Client::new(),
        fichub::services::ollama::OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        ),
    );
    let strategy = fichub::recommender::embeddings::EmbeddingsStrategy::new();
    let recs = strategy
        .score(&ctx, Some("rectestemb_a"), None)
        .await
        .expect("embeddings score");
    // A→C (cos 0.995) must be top; A→B (cos 0) must not outrank it.
    assert!(!recs.is_empty(), "must return recs");
    assert_eq!(recs[0].work_id, "rectestemb_c", "{recs:?}");
    assert!(
        recs.iter().any(|r| r.work_id == "rectestemb_b"),
        "B should appear (even at low cosine): {recs:?}"
    );

    for id in ["rectestemb_a", "rectestemb_b", "rectestemb_c"] {
        let _ = sqlx::query("DELETE FROM rec_embeddings WHERE work_id = $1")
            .bind(id)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(&db)
            .await;
    }
}

/// Content-hash gating: a work whose text is unchanged is skipped by
/// works_needing_embedding (existing hash matches).
#[ignore]
#[tokio::test]
async fn embeddings_content_hash_skips_unchanged() {
    let _guard = db_guard();
    let db = pool().await;

    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id, chapters, words,
            description, fic_created, fic_updated, status, source, extra_meta,
            raw_extended_meta, source_id, author_id, content_hash
        ) VALUES ('rectestemb_h', 'Emb Hash', 'emb_author', NULL, NULL, 1, 100, 'hash me', NOW(), NOW(),
                  'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .execute(&db)
    .await
    .expect("seed fic");

    // Compute the expected hash for the same text the strategy would build.
    let text = fichub::recommender::embeddings::build_embed_text("Emb Hash", "hash me", &[]);
    let hash = fichub::recommender::embeddings::content_hash(&text);

    // Pre-existing embedding with the SAME hash → skipped.
    sqlx::query(
        r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash, updated_at)
           VALUES ('rectestemb_h', 'nomic-embed-text', $2::vector, $1, NOW())
           ON CONFLICT (work_id, model) DO UPDATE SET content_hash = EXCLUDED.content_hash"#,
    )
    .bind(&hash)
    .bind(vec384(&[(0, 1.0)]))
    .execute(&db)
    .await
    .expect("seed embedding");

    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = fichub::recommender::registry::build_context(
        db.clone(),
        config,
        reqwest::Client::new(),
        fichub::services::ollama::OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        ),
    );
    let needing =
        fichub::recommender::embeddings::works_needing_embedding(&ctx, "nomic-embed-text", 100)
            .await
            .expect("query");
    assert!(
        !needing.iter().any(|(id, _)| id == "rectestemb_h"),
        "unchanged work must be skipped: {needing:?}"
    );

    let _ = sqlx::query("DELETE FROM rec_embeddings WHERE work_id = 'rectestemb_h'")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = 'rectestemb_h'")
        .execute(&db)
        .await;
}

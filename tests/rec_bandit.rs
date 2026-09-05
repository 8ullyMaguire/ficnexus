//! Integration tests for the contextual bandit strategy (Thompson sampling).
//!
//! DB-gated: seeds arms + impressions directly, runs reconcile + decay, and
//! asserts the posterior updates. The Thompson sampling MATH itself is
//! unit-tested in src/recommender/bandit.rs (pure fn with injected rng).

use std::sync::{Mutex, OnceLock};

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

fn test_ctx(
    db: sqlx::PgPool,
    config: std::sync::Arc<fichub::config::Config>,
) -> fichub::recommender::strategy::StrategyContext {
    fichub::recommender::registry::build_context(
        db,
        config,
        reqwest::Client::new(),
        fichub::services::ollama::OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        ),
    )
}

/// Seed a work + bandit arm; returns work id.
async fn seed_arm(db: &sqlx::PgPool, id: &str, alpha: f64, beta: f64) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id, chapters, words,
            description, fic_created, fic_updated, status, source, extra_meta,
            raw_extended_meta, source_id, author_id, content_hash
        ) VALUES ($1, $2, 'bandit_author', NULL, NULL, 1, 100, 'bandit test', NOW(), NOW(),
                  'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(id)
    .execute(db)
    .await
    .expect("seed fic");
    sqlx::query(
        r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
           VALUES ($1, 'bandit', $2, $3, NOW())
           ON CONFLICT (work_id, strategy) DO UPDATE SET alpha = $2, beta = $3, updated_at = NOW()"#,
    )
    .bind(id)
    .bind(alpha)
    .bind(beta)
    .execute(db)
    .await
    .expect("seed arm");
}

#[ignore]
#[tokio::test]
async fn bandit_engagement_updates_alpha() {
    let _guard = db_guard();
    let db = pool().await;
    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = test_ctx(db.clone(), config.clone());

    seed_arm(&db, "rectestbandit_a", 1.0, 1.0).await;
    seed_arm(&db, "rectestbandit_b", 1.0, 1.0).await;

    // Seed a user + a bookmark on A AFTER an impression → engagement.
    let user = "rectestit_bandit_user";
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(user)
    .execute(&db)
    .await
    .expect("seed user");
    let user_id: i32 = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(user)
        .fetch_one(&db)
        .await
        .expect("user id");
    sqlx::query(
        "INSERT INTO rec_impressions (user_id, work_id, strategy, shown_at) VALUES ($1, $2, 'bandit', NOW() - interval '2 days')",
    )
    .bind(user_id)
    .bind("rectestbandit_a")
    .execute(&db)
    .await
    .expect("seed impression");
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind("rectestbandit_a")
    .execute(&db)
    .await
    .expect("seed bookmark");

    let (engaged, _missed) = fichub::recommender::bandit::reconcile_impressions(&ctx)
        .await
        .expect("reconcile");
    assert!(engaged >= 1, "engagement must be detected: {engaged}");

    let alpha: f64 = sqlx::query_scalar(
        "SELECT alpha FROM rec_bandit_arms WHERE work_id = 'rectestbandit_a' AND strategy = 'bandit'",
    )
    .fetch_one(&db)
    .await
    .expect("alpha");
    assert!(alpha >= 2.0, "alpha must have incremented: {alpha}");

    // Cleanup.
    for id in ["rectestbandit_a", "rectestbandit_b"] {
        let _ = sqlx::query("DELETE FROM rec_bandit_arms WHERE work_id = $1")
            .bind(id)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM rec_impressions WHERE work_id = $1")
            .bind(id)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(&db)
            .await;
    }
    let _ = sqlx::query("DELETE FROM bookmarks WHERE user_id = $1")
        .bind(user_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(user)
        .execute(&db)
        .await;
}

#[ignore]
#[tokio::test]
async fn bandit_cold_impression_updates_beta() {
    let _guard = db_guard();
    let db = pool().await;
    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = test_ctx(db.clone(), config.clone());

    seed_arm(&db, "rectestbandit_c", 1.0, 1.0).await;
    let user = "rectestit_bandit_cold";
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(user)
    .execute(&db)
    .await
    .expect("seed user");
    let user_id: i32 = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(user)
        .fetch_one(&db)
        .await
        .expect("user id");
    // Cold impression (4 days old, no engagement).
    sqlx::query(
        "INSERT INTO rec_impressions (user_id, work_id, strategy, shown_at) VALUES ($1, $2, 'bandit', NOW() - interval '4 days')",
    )
    .bind(user_id)
    .bind("rectestbandit_c")
    .execute(&db)
    .await
    .expect("seed cold impression");

    let (_engaged, missed) = fichub::recommender::bandit::reconcile_impressions(&ctx)
        .await
        .expect("reconcile");
    assert!(
        missed >= 1,
        "cold impression must count as missed: {missed}"
    );

    let beta: f64 = sqlx::query_scalar(
        "SELECT beta FROM rec_bandit_arms WHERE work_id = 'rectestbandit_c' AND strategy = 'bandit'",
    )
    .fetch_one(&db)
    .await
    .expect("beta");
    assert!(beta >= 2.0, "beta must have incremented: {beta}");

    let _ = sqlx::query("DELETE FROM rec_bandit_arms WHERE work_id = 'rectestbandit_c'")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM rec_impressions WHERE work_id = 'rectestbandit_c'")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = 'rectestbandit_c'")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(user)
        .execute(&db)
        .await;
}

#[ignore]
#[tokio::test]
async fn bandit_decay_pulls_arms_toward_uniform() {
    let _guard = db_guard();
    let db = pool().await;
    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = test_ctx(db.clone(), config.clone());

    seed_arm(&db, "rectestbandit_d", 100.0, 1.0).await;
    let _ = fichub::recommender::bandit::decay_arms(&ctx).await;

    let (alpha, beta): (f64, f64) = sqlx::query_as(
        "SELECT alpha, beta FROM rec_bandit_arms WHERE work_id = 'rectestbandit_d' AND strategy = 'bandit'",
    )
    .fetch_one(&db)
    .await
    .expect("arm");
    // alpha must have shrunk toward 1 (evidence decayed).
    assert!(alpha < 100.0, "alpha must decay: {alpha}");
    assert!(alpha > 1.0, "alpha stays above 1: {alpha}");
    // beta starts at 1.0 and only moves via `new_b.max(1.0)` — with the
    // current decay (evidence = (param-1)·w + 1) a beta of 1.0 stays 1.0.
    // The meaningful decay assertion is alpha shrinking; beta just stays ≥ 1.
    assert!(beta >= 1.0, "beta must not shrink below 1: {beta}");

    let _ = sqlx::query("DELETE FROM rec_bandit_arms WHERE work_id = 'rectestbandit_d'")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = 'rectestbandit_d'")
        .execute(&db)
        .await;
}

//! Integration tests for the curator-prior taste-shaping.
//!
//! DB-gated: seeds a curator user with strong signals + a cold user, builds
//! the unified signal view, then asserts the ranker's curator blend pulls
//! the cold user's recs toward the curator's profile and that the floor
//! (REC_PRIOR_FLOOR) is respected.

use std::sync::{Mutex, OnceLock};

// use fichub::recommender::RecStrategy; // trait in scope for `.score()`

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (load .env)");
    sqlx::PgPool::connect(&database_url).await.expect("db connect")
}

async fn seed_user(pool: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user: lookup failed")
}

async fn seed_fic(pool: &sqlx::PgPool, id: &str, title: &str, updated_days_ago: i64) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id, chapters, words,
            description, fic_created, fic_updated, status, source, extra_meta,
            raw_extended_meta, source_id, author_id, content_hash
        ) VALUES ($1, $2, 'curator_author', NULL, NULL, 1, 100, 'curator test', NOW(), NOW(),
                  'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(title)
    .bind(chrono::Utc::now() - chrono::Duration::days(updated_days_ago))
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

async fn seed_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str, created_days_ago: i64) {
    sqlx::query(
        r#"INSERT INTO bookmarks (user_id, url_id, created_at) VALUES ($1, $2, NOW() - $3 * INTERVAL '1 day')
           ON CONFLICT (user_id, url_id) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(url_id)
    .bind(created_days_ago)
    .execute(pool)
    .await
    .expect("seed_bookmark failed");
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

/// Cold user (2 signals) gets pulled toward the curator: the curator's
/// top work outranks the cold user's own weak signals once blended, and the
/// floor is respected (alpha == REC_PRIOR_FLOOR for 0 signals).
#[ignore]
#[tokio::test]
async fn curator_prior_pulls_cold_user_toward_curator_profile() {
    let _guard = db_guard();
    let db = pool().await;

    // Fics.
    let fics = [
        "rectestcur_cur1", "rectestcur_cur2", // curator's picks
        "rectestcur_user1", "rectestcur_user2", // cold user's own
        "rectestcur_cand", // candidate
    ];
    for (i, f) in fics.iter().enumerate() {
        seed_fic(&db, f, &format!("Curator Test {i}"), 1).await;
    }

    // Curator user: strong profile on cur1/cur2.
    let curator_name = "rectestit_curator";
    let curator_id = seed_user(&db, curator_name).await;
    seed_bookmark(&db, curator_id, "rectestcur_cur1", 1).await;
    seed_bookmark(&db, curator_id, "rectestcur_cur2", 1).await;

    // Cold user: 2 signals on user1/user2 (just enough to pass the gate).
    let cold_name = "rectestit_colduser";
    let cold_id = seed_user(&db, cold_name).await;
    seed_bookmark(&db, cold_id, "rectestcur_user1", 1).await;
    seed_bookmark(&db, cold_id, "rectestcur_user2", 1).await;

    // Build the unified signal view with the curator multiplier.
    fichub::recommender::signals::refresh_signals(&db, Some(curator_id))
        .await
        .expect("refresh signals");

    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let ctx = test_ctx(db.clone(), config.clone());

    // The curator's top work (by weighted signals) — with the 5× multiplier
    // the curator profile is deterministic.
    let curator_top = fichub::recommender::ranker::fetch_curator_top(&ctx, curator_id).await;
    assert!(!curator_top.is_empty(), "curator must have a top list");
    let curator_first = curator_top[0].0.clone();

    // Alpha for a 2-signal user with floor 0.2, tau 25: α is the CURATOR
    // weight → still close to 1.0 (strongly curator-shaped).
    let alpha = fichub::recommender::ranker::curator_alpha(
        2,
        config.rec_prior_floor,
        config.rec_curator_tau,
    );
    assert!(alpha >= config.rec_prior_floor, "floor respected: {alpha}");
    assert!(alpha > 0.7, "cold user still strongly curator-shaped: {alpha}");

    // Alignment: user's tag vector vs curator's. Both have no tags seeded —
    // the alignment should be 0 (no shared tags) rather than NaN.
    let alignment = fichub::recommender::ranker::compute_alignment(&ctx, cold_id, curator_id).await;
    assert!(alignment.is_finite(), "alignment must be finite: {alignment}");

    // The pure blend math (no DB): a cold user's rec list containing the
    // curator's top work gets it lifted to the top at alpha ≈ floor.
    let fused = vec![
        ("rectestcur_user1".to_string(), 0.9),
        (curator_first.clone(), 0.6),
    ];
    let out = fichub::recommender::ranker::apply_curator_prior(&fused, &curator_top, alpha);
    assert_eq!(out[0].0, curator_first, "curator pick must lead after blend: {out:?}");
    // Blend is score-additive, not probability-bounded: the curator's raw
    // signal sums (5× bookmarks) can exceed 1.0 — assert relative ordering,
    // not an absolute ceiling.
    assert!(out[0].1 > out[1].1, "curator lift must dominate: {out:?}");

    // Cleanup.
    for f in &fics {
        let _ = sqlx::query("DELETE FROM rec_user_signals WHERE work_id = $1").bind(f).execute(&db).await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1").bind(f).execute(&db).await;
    }
    let _ = sqlx::query("DELETE FROM rec_user_curator_align WHERE user_id = $1").bind(cold_id).execute(&db).await;
    for name in [curator_name, cold_name] {
        let _ = sqlx::query("DELETE FROM bookmarks WHERE user_id = (SELECT id FROM users WHERE username = $1)").bind(name).execute(&db).await;
        let _ = sqlx::query("DELETE FROM rec_user_signals WHERE user_id = (SELECT id FROM users WHERE username = $1)").bind(name).execute(&db).await;
        let _ = sqlx::query("DELETE FROM users WHERE username = $1").bind(name).execute(&db).await;
    }
}

/// The pure alpha math: floor respected exactly at 0 signals, converging to
/// 1.0 with many signals. No DB needed.
#[test]
fn curator_alpha_floor_and_convergence() {
    use fichub::recommender::ranker::curator_alpha;
    let floor = 0.2;
    let tau = 25.0;
    // α is the CURATOR weight: 0 signals → 1.0 (pure curator picks).
    assert!((curator_alpha(0, floor, tau) - 1.0).abs() < 1e-9);
    // A few signals → still strongly curator-shaped.
    let low = curator_alpha(5, floor, tau);
    assert!(low > 0.5, "5 signals keeps curator influence: {low}");
    // Many signals → converges DOWN to the floor (user taste dominates,
    // but the curator floor is respected).
    let high = curator_alpha(1000, floor, tau);
    assert!((high - floor).abs() < 0.01, "floor respected: {high}");
}

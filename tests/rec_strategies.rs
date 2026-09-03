//! Integration tests for the pluggable recommendation platform — Stage 0
//! golden test + registry + RRF + curator prior.
//!
//! Conventions (same as tests/recommender_personal.rs): serialised via a
//! global Mutex, `#[ignore]` DB-gated, unique `rectest_*` seeds, cleanup
//! after each test.
//!
//! The golden test asserts the CENTRAL invariant of the refactor: a registry
//! containing ONLY the `cooccur` strategy (the legacy engine wrapped) must
//! produce the same personalized recs as the direct legacy handler call for
//! the same seeded user.

use std::sync::{Mutex, OnceLock};

// use serde_json::Value;

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database")
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

async fn seed_fic(
    pool: &sqlx::PgPool,
    id: &str,
    title: &str,
    description: &str,
    words: i64,
    chapters: i32,
    status: &str,
    updated_days_ago: i64,
) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(title)
    .bind("test_author")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(chapters)
    .bind(words)
    .bind(description)
    .bind(chrono::Utc::now() - chrono::Duration::days(updated_days_ago + 100))
    .bind(chrono::Utc::now() - chrono::Duration::days(updated_days_ago))
    .bind(status)
    .bind("ao3")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Option::<i64>::None)
    .bind(Option::<i64>::None)
    .bind(Option::<String>::None)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

async fn seed_tag(pool: &sqlx::PgPool, name: &str, type_id: i16) -> i32 {
    sqlx::query("INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING")
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

async fn seed_fic_tag(pool: &sqlx::PgPool, url_id: &str, tag_id: i32, score: i16) {
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) VALUES ($1, $2, '127.0.0.1', $3) ON CONFLICT (url_id, tag_id) DO NOTHING",
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(score)
    .execute(pool)
    .await
    .expect("seed_fic_tag failed");
}

async fn seed_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str) {
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(url_id)
    .execute(pool)
    .await
    .expect("seed_bookmark failed");
}

async fn cleanup(
    pool: &sqlx::PgPool,
    user: &str,
    fics: &[&str],
    tags: &[&str],
    downloads: &[&str],
) {
    let _ = sqlx::query("DELETE FROM bookmarks WHERE user_id = (SELECT id FROM users WHERE username = $1)")
        .bind(user)
        .execute(pool)
        .await;
    for id in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1").bind(id).execute(pool).await;
        let _ = sqlx::query("DELETE FROM rec_user_signals WHERE work_id = $1").bind(id).execute(pool).await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1").bind(id).execute(pool).await;
    }
    for name in tags {
        let _ = sqlx::query("DELETE FROM tags WHERE name = $1").bind(name).execute(pool).await;
    }
    for id in downloads {
        let _ = sqlx::query("DELETE FROM request_log WHERE url_id = $1").bind(id).execute(pool).await;
    }
    let _ = sqlx::query("DELETE FROM rec_user_signals WHERE user_id = (SELECT id FROM users WHERE username = $1)")
        .bind(user)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1").bind(user).execute(pool).await;
}

/// Seed a user with 3 bookmarks sharing a fandom tag + 1 candidate fic.
async fn seed_golden_fixture(pool: &sqlx::PgPool) -> (i32, [&'static str; 4]) {
    let user = "rectestit_golden_user";
    let user_id = seed_user(pool, user).await;
    let fics = ["rectestit_a", "rectestit_b", "rectestit_c", "rectestit_d"];
    seed_fic(pool, "rectestit_a", "Rec Test Alpha", "Dragon story.", 1000, 1, "complete", 1).await;
    seed_fic(pool, "rectestit_b", "Rec Test Beta", "Another dragon story.", 2000, 2, "complete", 2).await;
    seed_fic(pool, "rectestit_c", "Rec Test Gamma", "More dragons.", 3000, 3, "ongoing", 3).await;
    seed_fic(pool, "rectestit_d", "Rec Test Candidate", "The candidate.", 4000, 4, "ongoing", 1).await;
    let tag_fandom = seed_tag(pool, "RecTestIt Fandom", 1).await;
    let tag_free = seed_tag(pool, "RecTestIt Freeform", 4).await;
    for f in &fics[..3] {
        seed_fic_tag(pool, f, tag_fandom, 5).await;
    }
    seed_fic_tag(pool, "rectestit_d", tag_fandom, 5).await;
    seed_fic_tag(pool, "rectestit_a", tag_free, 5).await;
    for f in &fics[..3] {
        seed_bookmark(pool, user_id, f).await;
    }
    (user_id, fics)
}

/// Build a StrategyContext for tests (real config from .env).
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

// ─── Stage 0 golden test ───────────────────────────────────────────────────

/// The golden test: registry with ONLY cooccur → same recs as the legacy
/// handler's direct computation for the same seeded user.
#[ignore]
#[tokio::test]
async fn golden_legacy_equals_cooccur_strategy() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "rectestit_golden_user";
    let (user_id, fics) = seed_golden_fixture(&db).await;

    // Legacy path: the exact computation the legacy handler runs.
    let config = std::sync::Arc::new(fichub::config::Config::from_env());
    let (legacy_recs, _based_on, enough) =
        fichub::recommender::routes::compute_personal_recommendations(&db, user_id, &config).await
            .expect("legacy computation");
    assert!(enough, "seeded user must have enough signals");

    // Pluggable path: registry with ONLY cooccur.
    let registry = fichub::recommender::registry::StrategyRegistry::new(
        vec![std::sync::Arc::new(fichub::recommender::legacy_cooccur::LegacyCooccurStrategy::new())],
        "cooccur",
    );
    let ctx = test_ctx(db.clone(), config.clone());
    let (blended, diagnostics) = fichub::recommender::ranker::blend(
        &ctx,
        &registry,
        None,
        Some(user_id),
        20,
    )
    .await
    .expect("pluggable blend");

    assert_eq!(
        blended.len(),
        legacy_recs.len(),
        "strategy count must match legacy count: legacy={} strategy={}",
        legacy_recs.len(),
        blended.len()
    );
    // Same order, same work ids.
    for (i, (l, b)) in legacy_recs.iter().zip(blended.iter()).enumerate() {
        assert_eq!(
            l.url_id, b.work_id,
            "rec {i} must match: legacy {} vs strategy {}",
            l.url_id, b.work_id
        );
    }
    // Diagnostics: cooccur contributed.
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contributed, "{diagnostics:?}");

    cleanup(&db, user, &fics, &["RecTestIt Fandom", "RecTestIt Freeform"], &[]).await;
}

// ─── Registry selection ────────────────────────────────────────────────────

#[test]
fn registry_selects_weights_from_config() {
    let legacy = fichub::recommender::legacy_cooccur::LegacyCooccurStrategy::new();
    let decay = fichub::recommender::decay::DecayCooccurStrategy::new();
    let reg = fichub::recommender::registry::StrategyRegistry::new(
        vec![
            std::sync::Arc::new(legacy),
            std::sync::Arc::new(decay),
        ],
        "cooccur:0.4,decay:0.3",
    );
    let specs = reg.specs();
    assert_eq!(specs.len(), 2);
    assert!((specs[0].weight - 0.4).abs() < 1e-9);
    assert!((specs[1].weight - 0.3).abs() < 1e-9);
    assert_eq!(reg.enabled_names(), vec!["cooccur", "decay"]);
}

// ─── RRF blending (pure fn, no DB) ─────────────────────────────────────────

#[test]
fn rrf_blend_ranks_shared_higher() {
    use fichub::recommender::ranker::{rrf_accumulate, RRF_K};
    use std::collections::HashMap;

    let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
    let mk = |w: &str, s: f64, strat: &str| fichub::recommender::strategy::ScoredRec {
        work_id: w.into(),
        score: s,
        strategy: strat.into(),
        reason: "t".into(),
    };
    rrf_accumulate(&mut fused, &[mk("w1", 0.9, "s1"), mk("w2", 0.1, "s1")], 1.0);
    rrf_accumulate(&mut fused, &[mk("w1", 0.4, "s2"), mk("w3", 0.6, "s2")], 1.0);
    // w1 in both → 2·(1/(60+1)); w2 only in s1 → 1/(60+2); w3 only in s2 → 1/(60+2).
    assert!(fused["w1"].0 > fused["w2"].0);
    assert!(fused["w1"].0 > fused["w3"].0);
    assert!((fused["w1"].0 - 2.0 / (RRF_K + 1.0)).abs() < 1e-12);
}

// ─── Curator prior (pure fn, no DB) ────────────────────────────────────────

#[test]
fn curator_prior_pulls_cold_user_toward_curator() {
    use fichub::recommender::ranker::{apply_curator_prior, curator_alpha};
    // Cold user (0 signals) → α = 1.0 (pure curator picks).
    let alpha = curator_alpha(0, 0.2, 25.0);
    assert!((alpha - 1.0).abs() < 1e-9);
    let fused = vec![("w1".to_string(), 0.9), ("w2".to_string(), 0.8)];
    let curator = vec![("w2".to_string(), 1.0)];
    let out = apply_curator_prior(&fused, &curator, alpha);
    // w2: 0.8·0 + 1.0·1.0 = 1.0 > w1: 0.9·0 + 0 = 0 → w2 leads.
    assert_eq!(out[0].0, "w2", "curator-liked work must lead for a cold user: {out:?}");
}

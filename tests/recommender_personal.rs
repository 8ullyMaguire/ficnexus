//! Integration tests for the personalized recommendations handler
//! (`GET /api/recommendations/personal`).
//!
//! These exercise the FULL path: HTTP request → AuthUser extractor →
//! bookmark/download signal collection → SQL candidate generation →
//! Rust scoring → JSON response, through the real Axum router with a real
//! `AppState` (tower `ServiceExt::oneshot`), exactly like `tests/search_api.rs`.
//!
//! Conventions (same as `tests/search_api.rs` and the `db_tests` module in
//! `tests/integration.rs`):
//!
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]` so they are skipped by default and only run when
//!   explicitly requested:
//!   `cargo test --test recommender_personal -- --include-ignored --test-threads=1`.
//! * Each test seeds its own uniquely-named user/fics/tags/bookmarks with
//!   `INSERT ... ON CONFLICT DO NOTHING` and cleans up by deleting exactly
//!   the rows it created, so the suite is re-runnable and never touches
//!   pre-existing data.
//!
//! Requests carry a real JWT (created via `create_token` with the JWT_SECRET
//! from `.env`) so the `AuthUser` extractor resolves the seeded user; the
//! anonymous path is exercised with a bare request.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use serde_json::Value;
use tower::ServiceExt; // oneshot

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Global mutex that serialises ALL database-backed tests (mirrors
/// tests/search_api.rs and tests/integration.rs).
static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}

/// Acquire the DB serialisation lock, tolerating poisoning.
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

/// Connect to the configured test database (DATABASE_URL from .env).
async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with ONLY the personal-recommendations route and a real
/// AppState (same construction pattern as tests/search_api.rs).
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;

    let redis_client =
        redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL for test");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis (is it up?)");

    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("failed to build reqwest client");

    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());
    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:11434".into(),
        "nomic-embed-text".into(),
        http_client.clone(),
    );
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db.clone(),
        redis: redis.clone(),
        health_redis: redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("health redis conn"),
        http_client: http_client.clone(),
        scraper_registry: scraper_registry.clone(),
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(
            fichub::limiter::redis_bucket::RedisBucketLimiter::new(
                redis_client
                    .get_multiplexed_async_connection()
                    .await
                    .expect("redis"),
                false, // dynamic rate limiting off in tests
            )
            .await
            .expect("rate limiter"),
        ),
        recommender_engine: fichub::recommender::engine::RecommendationEngine::new(db.clone()),
        strategy_registry: fichub::recommender::registry::StrategyRegistry::new(
            vec![std::sync::Arc::new(
                fichub::recommender::legacy_cooccur::LegacyCooccurStrategy::new(),
            )],
            "cooccur",
        ),

        collection_worker: fichub::recommender::worker::CollectionWorker::new(
            db.clone(),
            redis,
            http_client,
            fichub::config::Config::from_env(),
            scraper_registry,
        ),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        heal: fichub::heal::HealService::new(db.clone(), config.clone()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/recommendations/personal",
            get(fichub::recommender::routes::personal_recommendations_handler),
        )
        .with_state(state)
}

/// JWT for the given user id (real token, real JWT_SECRET from env — the
/// AuthUser extractor reads JWT_SECRET from the environment, matching
/// server behaviour).
fn auth_header(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "rec_test_user".into(),
        trust_level: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Issue GET /api/recommendations/personal through the real router, with an
/// optional Authorization header, and parse the JSON body.
async fn get_personal(auth: Option<&str>) -> Value {
    let app = app().await;
    let mut builder = Request::builder().uri("/api/recommendations/personal");
    if let Some(token) = auth {
        builder = builder.header("Authorization", token);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 for /api/recommendations/personal, got {:?}",
        response.status()
    );
    serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap()
}

/// Seed a user; returns its id. `ON CONFLICT (username) DO NOTHING` keeps it
/// idempotent so tests are re-runnable.
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

/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps it idempotent.
/// `updated` (fic recency, used by popularity_norm) is set to now - updated_days_ago.
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

/// Seed a tag row; returns its id. `ON CONFLICT (name) DO NOTHING`.
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

/// Link a tag to a fic. `ON CONFLICT (url_id, tag_id) DO NOTHING`.
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

/// Bookmark a fic for a user. `ON CONFLICT (user_id, url_id) DO NOTHING`.
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

/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(
    pool: &sqlx::PgPool,
    user: &str,
    fics: &[&str],
    tags: &[&str],
    downloads: &[&str],
) {
    let _ = sqlx::query(
        "DELETE FROM bookmarks WHERE user_id = (SELECT id FROM users WHERE username = $1)",
    )
    .bind(user)
    .execute(pool)
    .await;
    for id in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
    for name in tags {
        let _ = sqlx::query("DELETE FROM tags WHERE name = $1")
            .bind(name)
            .execute(pool)
            .await;
    }
    for id in downloads {
        let _ = sqlx::query("DELETE FROM request_log WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(user)
        .execute(pool)
        .await;
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// (1) Anonymous request (no Authorization header) → 200 with
/// enough_data:false, empty recs and based_on — no auth round-trip needed.
#[ignore]
#[tokio::test]
async fn personal_recs_anonymous_returns_hint_payload() {
    let _guard = db_guard();
    let body = get_personal(None).await;

    assert_eq!(body["err"], 0, "anonymous must not error: {body}");
    assert_eq!(body["enough_data"], false, "anonymous: {body}");
    assert!(body["recs"].as_array().unwrap().is_empty(), "{body}");
    assert!(body["based_on"].as_array().unwrap().is_empty(), "{body}");
}

/// (2) Signed-in user with ZERO bookmarks (and no recent downloads) → 200
/// with enough_data:false.
#[ignore]
#[tokio::test]
async fn personal_recs_no_signals_returns_not_enough_data() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "recpersit_nosig_user";
    let user_id = seed_user(&db, user).await;

    let body = get_personal(Some(&auth_header(user_id))).await;
    assert_eq!(body["err"], 0, "{body}");
    assert_eq!(body["enough_data"], false, "0 signals must gate: {body}");
    assert!(body["recs"].as_array().unwrap().is_empty(), "{body}");
    assert!(body["based_on"].as_array().unwrap().is_empty(), "{body}");

    cleanup(&db, user, &[], &[], &[]).await;
}

/// (3) User with 3+ bookmarks sharing a tag with a 4th candidate fic →
/// enough_data:true, recs exclude the bookmarked fics, based_on is
/// populated, and the candidate appears in recs (scored via Jaccard
/// overlap + recency popularity).
#[ignore]
#[tokio::test]
async fn personal_recs_returns_scored_recs_excluding_known() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "recpersit_scored_user";
    let user_id = seed_user(&db, user).await;

    let fics = ["recpersit_a", "recpersit_b", "recpersit_c", "recpersit_d"];
    // Three bookmarked fics + one candidate sharing the fandom tag.
    seed_fic(
        &db,
        "recpersit_a",
        "Rec Personal Alpha",
        "A dragon story.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_b",
        "Rec Personal Beta",
        "Another dragon story.",
        2000,
        2,
        "complete",
        2,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_c",
        "Rec Personal Gamma",
        "More dragons.",
        3000,
        3,
        "ongoing",
        3,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_d",
        "Rec Personal Candidate",
        "The dragon candidate.",
        4000,
        4,
        "ongoing",
        1,
    )
    .await;

    let tag_fandom = seed_tag(&db, "RecPersIt Fandom", 1).await;
    let tag_char = seed_tag(&db, "RecPersIt Character", 2).await;
    let tag_free = seed_tag(&db, "RecPersIt Freeform", 4).await;
    for f in &fics[..3] {
        seed_fic_tag(&db, f, tag_fandom, 5).await;
        seed_fic_tag(&db, f, tag_char, 5).await;
    }
    // The candidate shares the fandom tag with all three bookmarked fics.
    seed_fic_tag(&db, "recpersit_d", tag_fandom, 5).await;
    // And one bookmarked fic has an extra tag the candidate lacks.
    seed_fic_tag(&db, "recpersit_a", tag_free, 5).await;

    for f in &fics[..3] {
        seed_bookmark(&db, user_id, f).await;
    }

    let body = get_personal(Some(&auth_header(user_id))).await;
    assert_eq!(body["err"], 0, "{body}");
    assert_eq!(
        body["enough_data"], true,
        "3 bookmarks must pass the gate: {body}"
    );

    // based_on: up to 3 bookmarked fic titles.
    let based_on = body["based_on"].as_array().unwrap();
    assert!(!based_on.is_empty(), "based_on must be populated: {body}");
    assert!(based_on.len() <= 3, "{body}");
    let based_ids: Vec<&str> = based_on
        .iter()
        .filter_map(|b| b["url_id"].as_str())
        .collect();
    assert!(based_ids.contains(&"recpersit_a"), "{body}");
    assert!(
        based_on.iter().all(|b| b["title"].as_str().is_some()),
        "{body}"
    );

    // recs: non-empty, candidate present, bookmarked fics excluded.
    let recs = body["recs"].as_array().unwrap();
    assert!(!recs.is_empty(), "candidate must be recommended: {body}");
    let rec_ids: Vec<&str> = recs.iter().filter_map(|r| r["url_id"].as_str()).collect();
    assert!(
        rec_ids.contains(&"recpersit_d"),
        "candidate sharing the fandom tag must appear: {body}"
    );
    for known in &fics[..3] {
        assert!(
            !rec_ids.contains(known),
            "bookmarked {known} leaked into recs: {body}"
        );
    }

    // RecResult shape: score/community_score/download_urls present.
    let first = &recs[0];
    assert!(first["score"].as_f64().is_some(), "{body}");
    assert_eq!(first["community_score"], 0, "{body}");
    assert!(first["download_urls"].is_object(), "{body}");
    assert!(first["title"].as_str().is_some(), "{body}");
    assert!(first["author"].as_str().is_some(), "{body}");

    cleanup(
        &db,
        user,
        &fics,
        &[
            "RecPersIt Fandom",
            "RecPersIt Character",
            "RecPersIt Freeform",
        ],
        &[],
    )
    .await;
}

/// (4) Download/export activity counts toward the signal gate: 2 bookmarks +
/// 1 recent download row → enough_data:true, and the downloaded fic is
/// excluded from recs like bookmarks.
#[ignore]
#[tokio::test]
async fn personal_recs_download_signal_counts_and_excludes() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "recpersit_dl_user";
    let user_id = seed_user(&db, user).await;

    let fics = [
        "recpersit_dl_a",
        "recpersit_dl_b",
        "recpersit_dl_downloaded",
        "recpersit_dl_cand",
    ];
    seed_fic(
        &db,
        "recpersit_dl_a",
        "DL Story Alpha",
        "Space opera.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_dl_b",
        "DL Story Beta",
        "More space opera.",
        2000,
        2,
        "complete",
        2,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_dl_downloaded",
        "DL Downloaded Fic",
        "Downloaded space opera.",
        3000,
        3,
        "complete",
        5,
    )
    .await;
    seed_fic(
        &db,
        "recpersit_dl_cand",
        "DL Candidate",
        "Fresh space opera.",
        4000,
        4,
        "ongoing",
        1,
    )
    .await;

    let tag = seed_tag(&db, "RecPersIt DL Fandom", 1).await;
    for f in &fics[..2] {
        seed_fic_tag(&db, f, tag, 5).await;
    }
    seed_fic_tag(&db, "recpersit_dl_downloaded", tag, 5).await;
    seed_fic_tag(&db, "recpersit_dl_cand", tag, 5).await;

    // 2 bookmarks + 1 recent epub download = 3 signals ≥ gate.
    seed_bookmark(&db, user_id, "recpersit_dl_a").await;
    seed_bookmark(&db, user_id, "recpersit_dl_b").await;
    sqlx::query(
        r#"INSERT INTO request_log (etype, query, info_request_ms, url_id, export_file_name)
           VALUES ('epub', 'https://example.test/fanfic/recpersit_dl_downloaded', 1, $1, 'recpersit_dl_downloaded.epub')"#,
    )
    .bind("recpersit_dl_downloaded")
    .execute(&db)
    .await
    .expect("seed request_log failed");

    let body = get_personal(Some(&auth_header(user_id))).await;
    assert_eq!(body["err"], 0, "{body}");
    assert_eq!(
        body["enough_data"], true,
        "2 bookmarks + 1 download must pass the gate: {body}"
    );

    let recs = body["recs"].as_array().unwrap();
    assert!(!recs.is_empty(), "{body}");
    let rec_ids: Vec<&str> = recs.iter().filter_map(|r| r["url_id"].as_str()).collect();
    assert!(rec_ids.contains(&"recpersit_dl_cand"), "candidate: {body}");
    assert!(
        !rec_ids.contains(&"recpersit_dl_downloaded"),
        "downloaded fic must be excluded: {body}"
    );

    cleanup(
        &db,
        user,
        &fics,
        &["RecPersIt DL Fandom"],
        &["recpersit_dl_downloaded"],
    )
    .await;
}

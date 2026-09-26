//! DB-gated integration tests for GET /api/search/body — full-text search
//! over fic bodies (maintained `body_text_search` tsvector column).
//!
//! Conventions (same as tests/requests_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . .env-e2e/runtime.env; set +a; cargo test --test body_search_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique url_id prefix)
//!   at the START so reruns never collide.
//!
//! The handler reads the body text from DISK, so each test points the app
//! config at a TEMP BODY_CACHE_DIR and writes real blob files there.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use serde_json::Value;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env-e2e/runtime.env, e.g. set -a; . .env-e2e/runtime.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// A fresh temp dir for THIS test's body cache (unique per pid + counter).
fn temp_body_dir(tag: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "fichub-body-search-test-{}-{}-{n}",
        std::process::id(),
        tag
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp body dir");
    dir
}

/// Build a router with GET /api/search/body and a real AppState whose
/// config points at the given (temp) body cache dir.
async fn app(body_dir: PathBuf) -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let mut config = fichub::config::Config::from_env();
    config.body_cache_dir = body_dir;
    config.search_max_per_page = 50;

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
            "/api/search/body",
            get(fichub::search::body::body_search_handler),
        )
        .with_state(state)
}

/// Seed a fic_info row (idempotent). Returns the url_id.
async fn seed_fic(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .execute(pool)
    .await
    .expect("seed fic_info failed");
}

/// Write a body blob file for a fic at the given cache dir (v1) with one
/// chapter containing `content`.
async fn write_body_blob(body_dir: &std::path::Path, url_id: &str, content: &str) {
    let config = fichub::config::Config::from_env();
    let chapters = vec![fichub::scrape::Chapter {
        chapter_id: 1,
        title: "Chapter One".into(),
        content: content.into(),
    }];
    // Point a temp config at the test dir so save_body shards correctly.
    let mut c = config;
    c.body_cache_dir = body_dir.to_path_buf();
    fichub::body_cache::save_body(&c, url_id, &chapters, Some("test".into()), 1)
        .expect("save_body blob failed");
}

async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap_or_else(
        |_| serde_json::json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }),
    );
    (status, v)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn body_search_finds_phrase_and_highlights_snippet() {
    let _g = db_guard();
    let db = pool().await;
    let url_id = "bodysrch_match_fic";
    // Self-heal: remove leftover rows from prior runs.
    sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();

    let body_dir = temp_body_dir("match");
    seed_fic(&db, url_id, "The Wandless Art", "bodysrch_author").await;
    write_body_blob(&body_dir, url_id,
        "<p>Harry practiced wandless magic in the orchard, far from the prying eyes of Privet Drive.</p>",
    )
    .await;
    // Populate the tsvector the way the production hook does.
    let mut c = fichub::config::Config::from_env();
    c.body_cache_dir = body_dir.clone();
    fichub::body_cache::index_body_text(&db, &c, url_id).await;

    let app = app(body_dir.clone()).await;
    let (status, v) = get_json(&app, "/api/search/body?q=wandless+magic").await;
    assert_eq!(status, StatusCode::OK, "resp: {v}");
    assert_eq!(v["err"], 0, "resp: {v}");
    assert_eq!(v["total"], 1, "resp: {v}");
    let results = v["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert_eq!(r["url_id"], url_id);
    assert_eq!(r["title"], "The Wandless Art");
    let snippet = r["body_snippet"].as_str().expect("body_snippet present");
    assert!(
        snippet.contains("<mark>"),
        "snippet should carry <mark> highlights: {snippet}"
    );
    assert!(
        snippet.to_lowercase().contains("wandless"),
        "snippet should contain the query term: {snippet}"
    );

    // Cleanup
    sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();
    let _ = std::fs::remove_dir_all(&body_dir);
}

#[tokio::test]
#[ignore]
async fn body_search_no_match_returns_zero() {
    let _g = db_guard();
    let db = pool().await;
    let url_id = "bodysrch_nomatch_fic";
    sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();

    let body_dir = temp_body_dir("nomatch");
    seed_fic(&db, url_id, "Gardening For Wizards", "bodysrch_author2").await;
    write_body_blob(
        &body_dir,
        url_id,
        "<p>Tomatoes and roses, tended with quiet patience under the greenhouse glass.</p>",
    )
    .await;
    let mut c = fichub::config::Config::from_env();
    c.body_cache_dir = body_dir.clone();
    fichub::body_cache::index_body_text(&db, &c, url_id).await;

    let app = app(body_dir.clone()).await;
    // "wands" does not appear in the body — must be a clean zero-result.
    let (status, v) = get_json(&app, "/api/search/body?q=wands").await;
    assert_eq!(status, StatusCode::OK, "resp: {v}");
    assert_eq!(v["err"], 0, "resp: {v}");
    assert_eq!(v["total"], 0, "resp: {v}");
    assert_eq!(v["results"].as_array().unwrap().len(), 0);

    // Cleanup
    sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();
    let _ = std::fs::remove_dir_all(&body_dir);
}

#[tokio::test]
#[ignore]
async fn body_search_empty_q_rejected() {
    let _g = db_guard();
    let db = pool().await;
    let body_dir = temp_body_dir("emptyq");
    let app = app(body_dir.clone()).await;
    let (status, v) = get_json(&app, "/api/search/body?q=").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "resp: {v}");
    assert_eq!(v["err"], -1, "resp: {v}");
    let _ = std::fs::remove_dir_all(&body_dir);
    let _ = db;
}

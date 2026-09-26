//! Integration tests for OPDS shelf authentication.
//!
//! Regression for the P1 bug: `/opds/shelves` returned a JSON 404
//! (`{"err":-5,...}`) when the shelf token was missing/invalid, instead of an
//! OPDS XML catalog (the OPDS auth-required convention).
//!
//! Follows the conventions of `tests/leaderboard_api.rs`:
//! * Serialised via a global Mutex (shared `fichub` DB).
//! * Marked `#[ignore]`; run with:
//!   `cargo test --test opds_api -- --include-ignored --test-threads=1`
//! * No DB rows are created — the shelf list is empty but the feed still
//!   renders, so cleanup is a no-op.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
// use serde_json::Value;
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
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    // Force a non-empty shelf token so the auth-required path is exercised.
    if c.opds_shelf_token.is_empty() {
        c.opds_shelf_token = "test-token".to_string();
    }
    c
}

/// Build a router with just the OPDS shelves routes.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = test_config();
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
                false,
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
            test_config(),
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
            "/opds/shelves",
            get(fichub::routes::opds::shelves::shelf_list),
        )
        .route("/opds", get(fichub::routes::opds::feeds::root_catalog))
        .with_state(state)
}

/// Issue GET and return (status, content-type, body text).
async fn get_raw(uri: &str) -> (StatusCode, String, String) {
    let app = app().await;
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let ct = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or("").to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, ct, String::from_utf8_lossy(&bytes).to_string())
}

/// Regression: unauthenticated /opds/shelves returns 200 XML with rel="self"
/// (previously a JSON 404).
#[ignore]
#[tokio::test]
async fn opds_shelves_unauthenticated_returns_auth_feed() {
    let _guard = db_guard();
    let (status, ct, body) = get_raw("/opds/shelves").await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        ct.contains("xml") || ct.contains("atom"),
        "content-type must be XML/Atom, got: {ct}"
    );
    assert!(body.contains("<feed"), "body must be an Atom feed");
    assert!(
        body.contains("rel=\"self\""),
        "feed must have rel=self link"
    );
    assert!(
        body.contains("Authentication required"),
        "unauthenticated feed should advertise auth required: {body}"
    );
    assert!(
        !body.trim_start().starts_with('{'),
        "must not be JSON: {body}"
    );
}

/// With the correct token, the shelf list renders (empty list is fine).
#[ignore]
#[tokio::test]
async fn opds_shelves_with_token_renders_list() {
    let _guard = db_guard();
    let token = {
        // Read the token the test config will use.
        let c = test_config();
        c.opds_shelf_token
    };
    let (status, ct, body) = get_raw(&format!("/opds/shelves?token={token}")).await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        ct.contains("xml") || ct.contains("atom"),
        "content-type must be XML/Atom, got: {ct}"
    );
    assert!(body.contains("<feed"), "body must be an Atom feed");
    // Empty shelf list is fine; must not be the auth-required feed.
    assert!(
        !body.contains("Authentication required"),
        "authenticated request should not see the auth feed"
    );
}

/// No token configured -> shelves are public (config default may set one, but
/// if the env var is unset the code default is "fichub"; we can't easily
/// unset it here, so this asserts the feed renders either way).
#[ignore]
#[tokio::test]
async fn opds_shelves_never_returns_json_404() {
    let _guard = db_guard();
    let (status, _ct, body) = get_raw("/opds/shelves?token=wrongtoken").await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        !body.trim_start().starts_with('{'),
        "must not be JSON even with a wrong token: {body}"
    );
    assert!(body.contains("<feed"), "body must be an Atom feed");
}

/// Regression: the root catalog must emit ABSOLUTE hrefs for rel="self" and
/// all navigation links when OPDS_BASE_URL is configured. Strict OPDS readers
/// reject relative rel="self" IRIs (the pre-fix behavior).
#[ignore]
#[tokio::test]
async fn opds_root_catalog_emits_absolute_urls_when_base_configured() {
    let _guard = db_guard();
    // SAFETY: single-threaded test (--test-threads=1), env is process-local.
    unsafe {
        std::env::set_var("OPDS_BASE_URL", "http://opds.example.test:8000");
    }

    let (status, ct, body) = get_raw("/opds").await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        ct.contains("xml") || ct.contains("atom"),
        "content-type must be XML/Atom, got: {ct}"
    );
    assert!(body.contains("<feed"), "body must be an Atom feed");
    assert!(
        body.contains("href=\"http://opds.example.test:8000/opds\" rel=\"self\""),
        "rel=self must be absolute when OPDS_BASE_URL is set, got: {body}"
    );
    assert!(
        body.contains("href=\"http://opds.example.test:8000/opds/new\""),
        "subsection nav links must be absolute: {body}"
    );
    assert!(
        body.contains("href=\"http://opds.example.test:8000/opds/search\" rel=\"search\""),
        "search link must be absolute: {body}"
    );
    assert!(
        body.contains("xml:base=\"http://opds.example.test:8000\""),
        "feed must carry xml:base: {body}"
    );
    assert!(
        !body.contains("href=\"/opds\""),
        "no bare relative /opds links should remain: {body}"
    );
}

//! DB-gated integration tests for the PoW (proof-of-work) challenge.
//!
//! Covers: shadowbanned client gets a challenge (GET /api/pow/challenge),
//! solving it stores a Redis solve key, the epub gate re-issues 429 with a
//! fresh challenge until solved and passes after, non-shadowbanned clients
//! get `not_needed:true`, and a non-shadowbanned epub request is never
//! challenged. Follows the repo's DB-gated conventions (global Mutex,
//! #[ignore], full AppState construction, unique seed names, cleanup).

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::{Mutex, OnceLock};
use tower::ServiceExt; // oneshot

// use fichub::limiter::TieredRateLimiter;
use fichub::services::pow;

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

async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("reqwest");
    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());

    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:1".into(), // port 1: never reachable — tests never need real embeddings
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
            fichub::config::Config::from_env(),
            scraper_registry,
        ),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        heal: fichub::heal::HealService::new(db.clone(), config.clone()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
    });

    Router::new()
        .route(
            "/api/pow/challenge",
            get(fichub::routes::pow::challenge_handler),
        )
        .route("/api/pow/solve", post(fichub::routes::pow::solve_handler))
        .route("/api/epub", get(fichub::routes::export::epub_handler))
        .with_state(state)
}

/// Remove the client from the shadowban set and drop any PoW keys it
/// accumulated. Runs before AND after each test so a crashed run never
/// pollutes the next.
async fn clean_client(redis: &redis::Client, client_id: &str) {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let _: () = redis::cmd("SREM")
        .arg("fichub:shadowban")
        .arg(client_id)
        .query_async(&mut conn)
        .await
        .unwrap_or(());
    let _: () = redis::cmd("DEL")
        .arg(format!("{}{}", pow::CHALLENGE_KEY_PREFIX, client_id))
        .arg(format!("{}powchall_abc", pow::SOLVED_KEY_PREFIX))
        .arg(format!("{}powchall_def", pow::SOLVED_KEY_PREFIX))
        .query_async(&mut conn)
        .await
        .unwrap_or(());
}

/// Shadowbanned client gets a real challenge (no `not_needed`), and the
/// solve for it is stored in Redis under `fichub:pow:solved:<challenge>`.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shadowbanned_client_gets_challenge_and_solve_stores_key() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;
    let redis_client =
        redis::Client::open(std::env::var("REDIS_URL").unwrap()).expect("redis client");
    let cid = "powtest_sb_client";
    clean_client(&redis_client, cid).await;

    // Flag the client.
    let limiter = fichub::limiter::redis_bucket::RedisBucketLimiter::new(
        redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("redis"),
        false,
    )
    .await
    .expect("limiter");
    limiter.shadowban(cid, 3600);
    assert!(
        limiter.is_shadowbanned(cid),
        "client must be shadowbanned first"
    );

    // GET /api/pow/challenge → real challenge
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pow/challenge")
                .header("x-client-id", cid)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        body["not_needed"], false,
        "shadowbanned client needs a challenge: {body}"
    );
    let challenge = body["challenge"]
        .as_str()
        .expect("challenge string")
        .to_string();
    assert_eq!(challenge.len(), 32, "32 hex chars: {body}");
    assert!(body["difficulty"].as_u64().is_some());
    assert!(body["expires_at"].as_i64().is_some());

    // Solve it (difficulty from the response).
    let difficulty = body["difficulty"].as_u64().unwrap() as u32;
    let nonce = find_nonce(&challenge, difficulty);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pow/solve")
                .header("x-client-id", cid)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "challenge": challenge, "nonce": nonce }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0, "solve accepted: {body}");

    // The solve is cached in Redis: GET fichub:pow:solved:<challenge> → "1".
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let stored: Option<String> = redis::cmd("GET")
        .arg(format!("{}{}", pow::SOLVED_KEY_PREFIX, challenge))
        .query_async(&mut conn)
        .await
        .expect("redis get");
    assert_eq!(
        stored.as_deref(),
        Some("1"),
        "solve must be cached in Redis"
    );

    // The epub gate now passes: no challenge re-issued for this client.
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/epub?q=http%3A%2F%2Fexample.com%2Ffic")
                .header("x-client-id", cid)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_ne!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "solved client passes the gate: {body}"
    );
    assert!(
        body["challenge"].is_null(),
        "no challenge in response: {body}"
    );

    clean_client(&redis_client, cid).await;
    let _ = db;
}

/// Unsolved shadowbanned client: every /api/epub request gets 429 with a
/// fresh challenge (never a scrape attempt).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shadowbanned_client_epub_returns_429_with_challenge() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;
    let redis_client =
        redis::Client::open(std::env::var("REDIS_URL").unwrap()).expect("redis client");
    let cid = "powtest_sb_unsolved";
    clean_client(&redis_client, cid).await;

    let limiter = fichub::limiter::redis_bucket::RedisBucketLimiter::new(
        redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("redis"),
        false,
    )
    .await
    .expect("limiter");
    limiter.shadowban(cid, 3600);

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/epub?q=http%3A%2F%2Fexample.com%2Ffic")
                .header("x-client-id", cid)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], -429, "pow required: {body}");
    assert!(
        body["challenge"].as_str().is_some(),
        "429 carries a challenge: {body}"
    );
    assert!(body["difficulty"].as_u64().is_some());
    assert!(body["expires_at"].as_i64().is_some());

    clean_client(&redis_client, cid).await;
    let _ = db;
}

/// Non-shadowbanned client: challenge endpoint says `not_needed:true` and
/// the epub handler never issues a challenge.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_shadowbanned_client_is_not_challenged() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;
    let redis_client =
        redis::Client::open(std::env::var("REDIS_URL").unwrap()).expect("redis client");
    let cid = "powtest_clean_client";
    clean_client(&redis_client, cid).await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pow/challenge")
                .header("x-client-id", cid)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        body["not_needed"], true,
        "clean client never sees friction: {body}"
    );
    assert!(body["challenge"].is_null());

    // The epub handler must NOT 429 a clean client (no challenge field).
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/epub?q=http%3A%2F%2Fexample.com%2Ffic")
                .header("x-client-id", cid)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_ne!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "clean client not 429'd: {body}"
    );
    assert!(
        body["challenge"].is_null(),
        "no challenge for clean client: {body}"
    );

    clean_client(&redis_client, cid).await;
    let _ = db;
}

/// Brute-force a nonce for `challenge` at `difficulty` bits (same
/// construction as the reference: SHA-256(challenge || nonce), hex prefix).
fn find_nonce(challenge: &str, difficulty: u32) -> String {
    let mut nonce: u64 = 0;
    loop {
        let n = nonce.to_string();
        if pow::verify_solution(challenge, &n, difficulty) {
            return n;
        }
        nonce += 1;
    }
}

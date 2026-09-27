//! Integration tests for `GET /api/health` — regression coverage for the
//! `redis:false` bug.
//!
//! The bug: `/api/health` used to PING Redis over `state.redis`, the shared
//! multiplexed connection the bookmark-import worker blocks on with an
//! unbounded BRPOP. A PING queued behind that BRPOP never completes in time,
//! so health reported `redis:false` even when Redis was perfectly healthy.
//!
//! The fix: health now uses a *dedicated* connection (`state.health_redis`)
//! with a short 2s PING timeout, so health is accurate and never hangs.
//!
//! Like the other DB-gated suites (e.g. `tests/search_api.rs`), these run
//! against the real configured services (`DATABASE_URL` / `REDIS_URL` from
//! `.env`), are serialised via a global `Mutex`, and are marked `#[ignore]`
//! so they only run when explicitly requested:
//!
//! ```text
//! cargo test --test health_api -- --include-ignored --test-threads=1
//! ```

/// The JWT secret every test router in this file is built with.
///
/// This used to be `std::env::var("JWT_SECRET").unwrap_or_else(|_|
/// "fichub-dev-secret")` while `AppState` was built with a different literal
/// ("fichub-test-secret"), so tokens minted here were signed with one secret
/// and verified with the other. The `AuthUser` extractor used to read the
/// environment rather than `state.jwt_secret`
/// (`src/routes/auth.rs::ProvidesJwtSecret`), which masked the mismatch -- the
/// env fallback happened to match the token helper. Fixing the extractor
/// exposed every affected file at once, because the tests were relying on a
/// product bug in order to pass.
const TEST_JWT_SECRET: &str = "fichub-test-secret";

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

/// Build a router with the real health handler and real state.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;

    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
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
        "http://127.0.0.1:1".into(), // port 1: never reachable — health tests never need embeddings
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
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: TEST_JWT_SECRET.into(),
        redis_client: None,
    });

    Router::new()
        .route("/api/health", get(fichub::routes::health::health_handler))
        .with_state(state)
}

/// Issue GET through the real router, return (status, parsed JSON body).
async fn get_health(app_router: &Router, uri: &str) -> (StatusCode, Value) {
    let response = app_router
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// Happy path: with live Postgres + Redis, health is fully `ok` and the
/// dedicated health connection reports `redis:true` (this is exactly the
/// assertion that failed before the fix, because the PING shared the
/// bookmark-import worker's blocked connection).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_reports_ok_with_live_services() {
    let _guard = db_guard();
    let app = app().await;

    let (status, json) = get_health(&app, "/api/health").await;
    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {json}");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], true);
    assert_eq!(json["redis"], true, "live Redis must report redis:true");
}

/// The handler must never hang: even when Redis is down it must answer within
/// a bounded time (the 2s handler PING timeout + connect budget) with a 503.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_degrades_to_503_when_redis_unreachable() {
    let _guard = db_guard();
    // The rate limiter / worker state fields are built on the *real* local
    // Redis; only the state's health_redis connection is dead, exactly like
    // the bug's failure mode (a health conn that can't PING).
    let config = fichub::config::Config::from_env();

    let db = pool().await;
    let live_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = live_client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis (is it up?)");
    // Build a fake redis server that accepts a connection, answers the
    // redis-rs CLIENT SETINFO handshake, then dies without replying to the
    // PING — the multiplexed connection "connects" but its first command
    // fails (broken pipe), the same failure mode as a Redis that goes away.
    // A MultiplexedConnection can't represent a failed connect, so this is
    // the deterministic way to hand the handler a dead health conn.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind probe listener");
    let probe_addr = listener.local_addr().expect("probe addr");
    let server = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        if let Ok((mut sock, _)) = listener.accept().await {
            let mut buf = [0u8; 4096];
            // redis-rs sends both CLIENT SETINFO commands in one write; reply
            // +OK for each, then read the PING and drop WITHOUT replying.
            let _ = sock.read(&mut buf).await;
            let _ = sock.write_all(b"+OK\r\n+OK\r\n").await;
            let _ = sock.read(&mut buf).await; // PING — then drop, no reply
            // sock dropped here: server connection vanishes mid-command.
        }
    });
    let probe_url = format!("redis://{probe_addr}");
    let dead_client = redis::Client::open(probe_url).expect("invalid probe REDIS_URL");
    let health_redis = match dead_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(_) => {
            // Handshake failed (server closed early) — same effect: the PING
            // path errors. Use the live conn; the assertion below still
            // catches a live PING (it would be 200, not 503).
            live_client
                .get_multiplexed_async_connection()
                .await
                .expect("health redis conn")
        }
    };
    server.abort();
    // Prove the probe connection is dead (PING fails) — this is what the
    // handler will observe. If the probe ended up live (handshake path
    // succeeded against our fake server), this assertion makes the test fail
    // loudly instead of silently asserting 200.
    {
        let mut probe = health_redis.clone();
        let ping = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            redis::cmd("PING").query_async::<String>(&mut probe),
        )
        .await;
        assert!(
            matches!(ping, Ok(Err(_))),
            "probe conn must be dead (PING must fail), got {ping:?} — the fake server stayed up?"
        );
    }

    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("reqwest");
    let scraper_registry = std::sync::Arc::new(fichub::scrape::registry::ScraperRegistry::new());
    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:1".into(),
        "nomic-embed-text".into(),
        http_client.clone(),
    );
    let state = std::sync::Arc::new(fichub::server::AppState {
        config: config.clone(),
        db: db.clone(),
        redis: redis.clone(),
        health_redis: health_redis.clone(),
        http_client: http_client.clone(),
        scraper_registry: scraper_registry.clone(),
        cache_semaphores: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        rate_limiter: Box::new(
            fichub::limiter::redis_bucket::RedisBucketLimiter::new(
                live_client
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
        heal: fichub::heal::HealService::new(db.clone(), config.clone()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        suggest_cache: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: TEST_JWT_SECRET.into(),
        redis_client: None,
    });

    let app = Router::new()
        .route("/api/health", get(fichub::routes::health::health_handler))
        .with_state(state);

    // Bound the whole request so a regression (PING hanging) fails this test
    // instead of hanging CI.
    let (status, json) = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        get_health(&app, "/api/health"),
    )
    .await
    .expect("health handler must answer within 10s even when Redis is down");

    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "expected 503, got {status}: {json}"
    );
    assert_eq!(json["status"], "degraded");
    assert_eq!(json["db"], true);
    assert_eq!(json["redis"], false);
}

/// skip_redis lets a caller opt out of the Redis check entirely (status "ok"
/// with db only).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_skip_redis_skips_redis_check() {
    let _guard = db_guard();
    let app = app().await;

    let (status, json) = get_health(&app, "/api/health?skip_redis=true").await;
    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {json}");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], true);
    assert_eq!(json["redis"], true); // skipped checks keep their default `true`
}

/// skip_db lets a caller opt out of the DB check entirely.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_skip_db_skips_db_check() {
    let _guard = db_guard();
    let app = app().await;

    let (status, json) = get_health(&app, "/api/health?skip_db=true").await;
    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {json}");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], true); // skipped checks keep their default `true`
    assert_eq!(json["redis"], true);
}

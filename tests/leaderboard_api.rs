//! Integration tests for the FicHub leaderboard endpoints.
//!
//! Regression test for the SMALLINT COALESCE type-promotion bug:
//! `COALESCE(lb.rank, 0)` in `get_weekly_leaderboard` / `get_monthly_leaderboard`
//! produced INT4 while the Rust tuple decoded `i16`, so
//! `/api/leaderboard/curators/weekly` and `.../monthly` returned 500
//! `{"err":-1,"msg":"database error"}`. Fixed with `::SMALLINT` cast.
//!
//! Follows the conventions of `tests/search_api.rs`:
//! * Serialised via a global Mutex (shared `fichub` DB).
//! * Marked `#[ignore]`; run with:
//!   `cargo test --test leaderboard_api -- --include-ignored --test-threads=1`
//! * Seeds a user + leaderboard rows with unique names and cleans up after itself.

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
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Minimal config with sane defaults.
fn test_config() -> fichub::config::Config {
    fichub::config::Config::from_env()
}

/// Build a router with the two leaderboard endpoints (mirrors how
/// `tests/search_api.rs` builds a minimal router with the real handlers).
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
    });

    Router::new()
        .route(
            "/api/leaderboard/curators/weekly",
            get(fichub::routes::badges::leaderboard_weekly_handler),
        )
        .route(
            "/api/leaderboard/curators/monthly",
            get(fichub::routes::badges::leaderboard_monthly_handler),
        )
        .with_state(state)
}

/// Issue GET through the real router and parse the JSON body.
async fn get_leaderboard(uri: &str) -> Value {
    let app = app().await;
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 for {uri}, got {:?}",
        response.status()
    );
    serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap()
}

/// Create a uniquely-named user for the test; returns their id.
async fn seed_user(pool: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, email)
         VALUES ($1, 'x', NULL)
         ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("seed_user failed");

    sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user: fetch id")
}

/// Insert a leaderboard_weekly row for the user.
async fn seed_weekly(pool: &sqlx::PgPool, user_id: i32, score: i32, rank: i16, week_start: &str) {
    sqlx::query(
        "INSERT INTO leaderboard_weekly (user_id, score, rank, week_start)
         VALUES ($1, $2, $3, $4::date)
         ON CONFLICT (user_id, week_start) DO NOTHING",
    )
    .bind(user_id)
    .bind(score)
    .bind(rank)
    .bind(week_start)
    .execute(pool)
    .await
    .expect("seed_weekly failed");
}

/// Insert a leaderboard_monthly row for the user.
async fn seed_monthly(pool: &sqlx::PgPool, user_id: i32, score: i32, rank: i16, month_start: &str) {
    sqlx::query(
        "INSERT INTO leaderboard_monthly (user_id, score, rank, month_start)
         VALUES ($1, $2, $3, $4::date)
         ON CONFLICT (user_id, month_start) DO NOTHING",
    )
    .bind(user_id)
    .bind(score)
    .bind(rank)
    .bind(month_start)
    .execute(pool)
    .await
    .expect("seed_monthly failed");
}

async fn cleanup(pool: &sqlx::PgPool, username: &str) {
    // FK cascade removes leaderboard rows; delete the user last.
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(pool)
        .await;
}

/// Regression: weekly leaderboard must return 200 + err:0 with a seeded row
/// (was 500 "database error" from the SMALLINT COALESCE promotion).
#[ignore]
#[tokio::test]
async fn weekly_leaderboard_returns_rows() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "lbtest_weekly_1";
    let uid = seed_user(&db, username).await;
    seed_weekly(&db, uid, 42, 1, "2026-08-03").await;

    let body = get_leaderboard("/api/leaderboard/curators/weekly").await;
    assert_eq!(body["err"], 0, "weekly err: {body}");
    let entries = body["leaderboard"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|e| e["username"] == username && e["score"] == 42 && e["rank"] == 1),
        "seeded user should appear in weekly leaderboard: {body}"
    );

    cleanup(&db, username).await;
}

/// Regression: monthly leaderboard must return 200 + err:0 with a seeded row.
#[ignore]
#[tokio::test]
async fn monthly_leaderboard_returns_rows() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "lbtest_monthly_1";
    let uid = seed_user(&db, username).await;
    seed_monthly(&db, uid, 100, 2, "2026-08-01").await;

    let body = get_leaderboard("/api/leaderboard/curators/monthly").await;
    assert_eq!(body["err"], 0, "monthly err: {body}");
    let entries = body["leaderboard"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|e| e["username"] == username && e["score"] == 100 && e["rank"] == 2),
        "seeded user should appear in monthly leaderboard: {body}"
    );

    cleanup(&db, username).await;
}

/// Sanity: with no leaderboard rows, the endpoint still returns 200 + empty list.
#[ignore]
#[tokio::test]
async fn leaderboard_empty_is_ok() {
    let _guard = db_guard();
    let body = get_leaderboard("/api/leaderboard/curators/weekly").await;
    assert_eq!(body["err"], 0, "empty weekly err: {body}");
    assert!(
        body["leaderboard"].as_array().is_some(),
        "leaderboard must be a list: {body}"
    );
}

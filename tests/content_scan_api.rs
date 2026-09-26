//! DB-gated integration tests for the content-scan admin endpoints
//! (body sanity + warning verification review flow).
//!
//! Conventions: serialised via a global Mutex, marked `#[ignore]`, run with
//! `set -a; . ./.env; set +a; cargo test --test content_scan_api -- --include-ignored --test-threads=1`.
//! Never depends on a running Ollama — seed content_scan rows directly.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::{get, post},
};
use serde_json::Value;
use sqlx::Row;
use tower::ServiceExt; // oneshot

use fichub::routes::admin::{list_content_scan, review_content_scan, run_content_scan};

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

const ADMIN_USER: &str = "content_scan_admin";
const URL_ID: &str = "cs_abc123";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("connect to DB")
}

async fn cleanup(db: &sqlx::PgPool, username: &str, url_id: &str) {
    sqlx::query("DELETE FROM content_scan WHERE url_id = $1")
        .bind(url_id)
        .execute(db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(db)
        .await
        .ok();
}

async fn seed_admin(db: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, 'x', 10) \
                 ON CONFLICT (username) DO UPDATE SET role = 10 RETURNING id",
    )
    .bind(username)
    .fetch_one(db)
    .await
    .expect("seed admin")
    .get(0)
}

async fn seed_scan(db: &sqlx::PgPool, url_id: &str, classification: &str, warnings: &str) {
    sqlx::query(
        "INSERT INTO content_scan (url_id, classification, confidence, detected_warnings, reason) \
         VALUES ($1, $2, 0.9, $3, 'test') \
         ON CONFLICT (url_id) DO UPDATE SET classification = $2, detected_warnings = $3",
    )
    .bind(url_id)
    .bind(classification)
    .bind(warnings)
    .execute(db)
    .await
    .expect("seed scan row");
}

async fn app(db: sqlx::PgPool) -> Router {
    let config = fichub::config::Config::from_env();
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");

    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("http client");

    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());
    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:1".into(),
        "llama3.1:8b".into(),
        http_client.clone(),
    );

    let state = Arc::new(fichub::server::AppState {
        config,
        db: db.clone(),
        redis: redis.clone(),
        health_redis: redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("health redis"),
        http_client: http_client.clone(),
        scraper_registry: scraper_registry.clone(),
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(
            fichub::limiter::redis_bucket::RedisBucketLimiter::new(
                redis_client
                    .get_multiplexed_async_connection()
                    .await
                    .expect("rate limiter redis"),
                false,
            )
            .await
            .expect("rate limiter"),
        ),
        recommender_engine: fichub::recommender::engine::RecommendationEngine::new(db.clone()),
        strategy_registry: fichub::recommender::registry::StrategyRegistry::new(
            vec![Arc::new(
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
        heal: fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route("/api/admin/content-scan", get(list_content_scan))
        .route("/api/admin/content-scan/run", post(run_content_scan))
        .route(
            "/api/admin/content-scan/{url_id}/review",
            post(review_content_scan),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, trust_level: i16, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token");
    format!("Bearer {token}")
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn list_scan_and_review_flow() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db, ADMIN_USER, URL_ID).await;
    let admin_id = seed_admin(&db, ADMIN_USER).await;
    seed_scan(&db, URL_ID, "noise", "profanity").await;

    let app = app(db.clone()).await;
    let token = auth_header(admin_id, 10, ADMIN_USER);

    // List pending scan results → the seeded noise row appears.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/content-scan?review_status=pending")
                .header(header::AUTHORIZATION, &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
        panic!(
            "status={status} bad json: {}",
            String::from_utf8_lossy(&body_bytes)
        )
    });
    assert_eq!(status, StatusCode::OK, "list should be OK: {}", body);
    assert_eq!(body["err"], 0);
    let items = body["items"].as_array().unwrap();
    assert!(
        items.iter().any(|i| i["url_id"] == URL_ID),
        "seeded row listed"
    );

    // Review: confirm the flag.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/content-scan/{URL_ID}/review"))
                .header(header::AUTHORIZATION, &token)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"action":"confirmed"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let review_status = resp.status();
    let review_body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(
        review_status,
        StatusCode::OK,
        "review should be OK: {}",
        String::from_utf8_lossy(&review_body_bytes)
    );

    // After review it no longer appears in pending.
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/content-scan?review_status=pending")
                .header(header::AUTHORIZATION, &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(
        !body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["url_id"] == URL_ID),
        "reviewed row leaves pending"
    );

    cleanup(&db, ADMIN_USER, URL_ID).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn run_scan_requires_admin() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db, ADMIN_USER, URL_ID).await;
    seed_admin(&db, ADMIN_USER).await;
    let app = app(db.clone()).await;

    // No token → rejected (this codebase maps missing auth to 403 on
    // admin-gated routes).
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/content-scan/run")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"limit": 1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    cleanup(&db, ADMIN_USER, URL_ID).await;
}

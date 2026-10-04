//! DB-gated integration tests for POST /api/send-to-kindle.
//!
//! The SMTP side is fully mocked: the test AppState carries a `MockMailer`
//! (no relay, no credentials), so these tests verify the handler's behavior
//! — recipient selection, validation, unknown-fic errors — without any
//! network. Follows the repo's DB-gated conventions (global Mutex, #[ignore],
//! full AppState construction, unique seed names, cleanup).

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

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::post,
};
use serde_json::{Value, json};
use tower::ServiceExt; // oneshot

use fichub::server::AppState;
use fichub::services::mailer::{Mailer, MockMailer};

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

/// Build a router with a MockMailer so no SMTP network is ever attempted.
async fn app_with_mock(mailer: Arc<MockMailer>) -> Router {
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
        "http://127.0.0.1:1".into(),
        "nomic-embed-text".into(),
        http_client.clone(),
    );

    // The mocked mailer is shared with the test so we can inspect sends.
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
        mailer: Box::new(mailer.as_ref().clone_box()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: TEST_JWT_SECRET.into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/send-to-kindle",
            post(fichub::routes::kindle::send_to_kindle_handler),
        )
        .with_state(state)
}

/// Seed a throwaway user with an email, return (id, username).
async fn seed_user(db: &sqlx::PgPool, username: &str, email: &str) -> i32 {
    sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, email)
         VALUES ($1, 'x', $2)
         ON CONFLICT (username) DO UPDATE SET email = EXCLUDED.email
         RETURNING id",
    )
    .bind(username)
    .bind(email)
    .fetch_one(db)
    .await
    .expect("seed user")
}

fn token_for(user_id: i32, username: &str) -> String {
    let secret = TEST_JWT_SECRET;
    fichub::routes::auth::create_token(
        &fichub::routes::auth::User {
            id: user_id,
            username: username.into(),
            trust_level: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
            is_admin: false,
        },
        &secret,
    )
    .expect("token")
}

/// Helper: POST /api/send-to-kindle and return (status, body).
async fn post_send(app: &Router, token: &str, body: Value) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/send-to-kindle")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap_or_else(
        |_| json!({ "err": -999, "msg": String::from_utf8_lossy(&bytes).to_string() }),
    );
    (status, value)
}

/// Anonymous request → 400 "Authentication required" (HTTP 400 per the
/// project's error convention: AppError::BadRequest(401, ...) maps to 400).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn send_to_kindle_requires_auth() {
    let _guard = db_guard();
    let mailer = Arc::new(MockMailer::new());
    let app = app_with_mock(mailer.clone()).await;

    let (status, body) = post_send(&app, "no-token", json!({ "url_id": "whatever" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["err"], 401, "{body}");
    assert!(mailer.sent_mails().is_empty(), "nothing sent");
}

/// Invalid request body (neither url nor url_id) → 400.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn send_to_kindle_rejects_no_identifier() {
    let _guard = db_guard();
    let db = pool().await;
    let mailer = Arc::new(MockMailer::new());
    let app = app_with_mock(mailer.clone()).await;

    let uid = seed_user(&db, "kindle_api_noid", "noid@example.com").await;
    let token = token_for(uid, "kindle_api_noid");

    let (status, body) = post_send(&app, &token, json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["err"], -1, "{body}");
    assert!(mailer.sent_mails().is_empty(), "nothing sent");
}

/// Unknown fic (a URL with no scraper / nonexistent url_id) → graceful error.
/// The handler falls back to scraping the identifier as a URL; when the
/// scraper can't resolve it the error is a 502 ScrapeError (graceful — it
/// does NOT attempt to email anything, and the user sees a clear message).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn send_to_kindle_unknown_fic_is_graceful() {
    let _guard = db_guard();
    let db = pool().await;
    let mailer = Arc::new(MockMailer::new());
    let app = app_with_mock(mailer.clone()).await;

    let uid = seed_user(&db, "kindle_api_unknown", "unknown@example.com").await;
    let token = token_for(uid, "kindle_api_unknown");

    // url_id that is not in fic_info and has no scraper → error, no send.
    let (status, body) = post_send(
        &app,
        &token,
        json!({ "url_id": "kindle_api_does_not_exist" }),
    )
    .await;
    // Either 404 (fic not found) or 502 (scrape failed) is graceful —
    // the invariant is that NOTHING is sent and the body carries an error.
    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::BAD_GATEWAY,
        "expected graceful error status, got {status}: {body}"
    );
    assert!(body["err"].is_i64() || body["err"].is_i64(), "{body}");
    assert!(mailer.sent_mails().is_empty(), "nothing sent");
}

/// User with no email on file → 400 with a clear message, nothing sent.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn send_to_kindle_no_email_on_file() {
    let _guard = db_guard();
    let db = pool().await;
    let mailer = Arc::new(MockMailer::new());
    let app = app_with_mock(mailer.clone()).await;

    // email = NULL exercises the empty-email branch (the handler
    // COALESCEs to ''). NULL is used because users.email has a UNIQUE
    // constraint and '' would collide with any other email-less user.
    let username = format!("kindle_api_noemail_{}", uuid::Uuid::new_v4().simple());
    let uid = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, email)
         VALUES ($1, 'x', NULL)
         RETURNING id",
    )
    .bind(&username)
    .fetch_one(&db)
    .await
    .expect("seed user with NULL email");
    let token = token_for(uid, &username);

    let (status, body) = post_send(&app, &token, json!({ "url_id": "whatever" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["err"], -1, "{body}");
    assert!(
        body["msg"]
            .as_str()
            .unwrap_or("")
            .contains("no email on file"),
        "{body}"
    );
    assert!(mailer.sent_mails().is_empty(), "nothing sent");

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .unwrap();
}

/// Clean up any rows the tests may have left behind (crashed runs etc.).
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kindle_api_cleanup_leaked_rows() {
    let _guard = db_guard();
    let db = pool().await;
    sqlx::query("DELETE FROM users WHERE username LIKE 'kindle_api_%'")
        .execute(&db)
        .await
        .unwrap();
}

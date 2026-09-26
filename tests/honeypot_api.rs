//! DB-gated integration tests for the honeypot + form-timing traps on
//! registration and comments.
//!
//! Conventions copied from `tests/admin_api.rs` (AppState construction)
//! and `tests/search_api.rs` (DB_LOCK serialisation, `#[ignore]` gating):
//!
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]` — run with
//!   `cargo test --test honeypot_api -- --include-ignored`.
//! * Each test seeds/cleans up exactly the rows it created, so the suite
//!   is re-runnable against the shared `fichub` database.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::post,
};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt; // oneshot

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

/// Full AppState construction — identical to tests/admin_api.rs.
async fn app() -> Router {
    use fichub::server::AppState;

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
            "/api/auth/register",
            post(fichub::routes::social::register_handler),
        )
        .route(
            "/api/auth/login",
            post(fichub::routes::social::login_handler),
        )
        .route(
            "/api/comments",
            post(fichub::routes::social::add_comment_handler),
        )
        .with_state(state)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn user_count(db: &sqlx::PgPool, username: &str) -> i64 {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(db)
        .await
        .expect("count users");
    count
}

async fn comment_count(db: &sqlx::PgPool, url_id: &str, body: &str) -> i64 {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM comments WHERE url_id = $1 AND body = $2")
            .bind(url_id)
            .bind(body)
            .fetch_one(db)
            .await
            .expect("count comments");
    count
}

/// Idempotently seed a user, returning its id (re-runnable).
async fn seed_user(db: &sqlx::PgPool, username: &str) -> i32 {
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(db)
        .await;
    sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, 'x') RETURNING id")
        .bind(username)
        .fetch_one(db)
        .await
        .expect("seed user")
        .get(0)
}

/// register with honeypot filled -> 200 OK, success-looking body, but NO
/// user row is created (silent rejection, bot not taught).
#[ignore]
#[tokio::test]
async fn honeypot_register_silently_rejected() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let username = "honeypot_register_bot";
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;

    let opened = (now_ms() - 5000).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": "hunter22",
                        "email": "bot@example.com",
                        "website": "http://spam.example",
                        "form_opened_at": opened,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Success-looking response…
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0, "must look successful: {body}");
    assert_eq!(
        body["token"], "",
        "no token for silently-rejected registration"
    );

    // …but no account.
    assert_eq!(
        user_count(&db, username).await,
        0,
        "honeypot registration must NOT create a user"
    );

    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;
}

/// register normally (honeypot empty, form opened 5s ago) -> user IS created.
#[ignore]
#[tokio::test]
async fn normal_register_creates_user() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let username = "honeypot_normal_user";
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;

    let opened = (now_ms() - 5000).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": "hunter22",
                        "email": "human@example.com",
                        "website": "",
                        "form_opened_at": opened,
                    })
                    .to_string(),
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
    assert_eq!(body["err"], 0);
    assert!(
        !body["token"].as_str().unwrap().is_empty(),
        "real registration gets a token"
    );

    assert_eq!(
        user_count(&db, username).await,
        1,
        "normal registration must create a user"
    );

    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;
}

/// register without the JS-set form_opened_at (scripted/curl bot) -> 200
/// OK but no user row.
#[ignore]
#[tokio::test]
async fn register_without_opened_at_silently_rejected() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let username = "honeypot_no_timestamp";
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": username,
                        "password": "hunter22",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        user_count(&db, username).await,
        0,
        "scripted registration must NOT create a user"
    );

    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await;
}

/// comment with honeypot filled -> 200 OK but no comment row.
#[ignore]
#[tokio::test]
async fn honeypot_comment_silently_rejected() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    // Seed a user + a fic_info row so the comment path is fully real.
    let user_id = seed_user(&db, "honeypot_comment_user").await;

    let url_id = format!("honeypot_url_{}", now_ms());
    let _ = sqlx::query(
        "INSERT INTO fic_info (id, title, author, source, description, chapters, words, status, fic_created, fic_updated, extra_meta, raw_extended_meta, content_hash)
         VALUES ($1, 'Honeypot Fic', 'Test Author', 'https://example.test/honeypot', 'desc', 1, 1000, 'complete', NOW(), NOW(), '{}', '{}', 'honeypot-hash')",
    )
    .bind(&url_id)
    .execute(&db)
    .await
    .expect("seed fic_info");

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(
        &fichub::routes::auth::User {
            id: user_id,
            username: "honeypot_comment_user".into(),
            trust_level: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
            is_admin: false,
        },
        &secret,
    )
    .expect("token");

    let opened = (now_ms() - 5000).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/comments")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    json!({
                        "work_id": 1,
                        "url_id": url_id,
                        "body": "spam comment from a honeypot-filling bot",
                        "website": "http://spam.example",
                        "form_opened_at": opened,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Success-looking response…
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0, "must look successful: {body}");

    // …but no comment row.
    assert_eq!(
        comment_count(&db, &url_id, "spam comment from a honeypot-filling bot").await,
        0,
        "honeypot comment must NOT be persisted"
    );

    let _ = sqlx::query("DELETE FROM comments WHERE url_id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&db)
        .await;
}

/// comment submitted too fast (< 500ms form-open) -> 200 OK but no row.
#[ignore]
#[tokio::test]
async fn too_fast_comment_silently_rejected() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let user_id = seed_user(&db, "honeypot_fast_user").await;

    let url_id = format!("honeypot_fast_url_{}", now_ms());
    let _ = sqlx::query(
        "INSERT INTO fic_info (id, title, author, source, description, chapters, words, status, fic_created, fic_updated, extra_meta, raw_extended_meta, content_hash)
         VALUES ($1, 'Honeypot Fic 2', 'Test Author', 'https://example.test/honeypot2', 'desc', 1, 1000, 'complete', NOW(), NOW(), '{}', '{}', 'honeypot-hash2')",
    )
    .bind(&url_id)
    .execute(&db)
    .await
    .expect("seed fic_info");

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(
        &fichub::routes::auth::User {
            id: user_id,
            username: "honeypot_fast_user".into(),
            trust_level: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
            is_admin: false,
        },
        &secret,
    )
    .expect("token");

    // Form "opened" 100ms ago — a human can't type that fast.
    let opened = (now_ms() - 100).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/comments")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    json!({
                        "work_id": 1,
                        "url_id": url_id,
                        "body": "impossibly fast comment",
                        "website": "",
                        "form_opened_at": opened,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        comment_count(&db, &url_id, "impossibly fast comment").await,
        0,
        "too-fast comment must NOT be persisted"
    );

    let _ = sqlx::query("DELETE FROM comments WHERE url_id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&db)
        .await;
}

/// normal comment (honeypot empty, opened 5s ago) -> comment IS created.
#[ignore]
#[tokio::test]
async fn normal_comment_created() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let user_id = seed_user(&db, "honeypot_normal_commenter").await;

    let url_id = format!("honeypot_norm_url_{}", now_ms());
    let _ = sqlx::query(
        "INSERT INTO fic_info (id, title, author, source, description, chapters, words, status, fic_created, fic_updated, extra_meta, raw_extended_meta, content_hash)
         VALUES ($1, 'Honeypot Fic 3', 'Test Author', 'https://example.test/honeypot3', 'desc', 1, 1000, 'complete', NOW(), NOW(), '{}', '{}', 'honeypot-hash3')",
    )
    .bind(&url_id)
    .execute(&db)
    .await
    .expect("seed fic_info");

    // The comment INSERT carries work_id with an FK to works(id); seed a real
    // works row instead of assuming id 1 exists.
    let (work_id,): (i32,) = sqlx::query_as(
        "INSERT INTO works (canonical_title, canonical_author, description, default_source_id)
         VALUES ('Honeypot Fic 3', 'Test Author', 'desc', $1)
         RETURNING id",
    )
    .bind(&url_id)
    .fetch_one(&db)
    .await
    .expect("seed works");

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(
        &fichub::routes::auth::User {
            id: user_id,
            username: "honeypot_normal_commenter".into(),
            trust_level: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
            is_admin: false,
        },
        &secret,
    )
    .expect("token");

    let opened = (now_ms() - 5000).to_string();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/comments")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    json!({
                        "work_id": work_id,
                        "url_id": url_id,
                        "body": "a genuine human comment",
                        "website": "",
                        "form_opened_at": opened,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let res_bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&res_bytes).unwrap();
    assert_eq!(body["err"], 0, "comment create must succeed: {body}");
    assert!(
        body["comment_id"].as_i64().unwrap() > 0,
        "real comment gets a real id"
    );

    assert_eq!(
        comment_count(&db, &url_id, "a genuine human comment").await,
        1,
        "normal comment must be persisted"
    );

    let _ = sqlx::query("DELETE FROM comments WHERE url_id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(&url_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&db)
        .await;
}

/// Failed login logs an auth_failed request_log row (bot stuffing signal).
#[ignore]
#[tokio::test]
async fn failed_login_logs_auth_failed_row() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    // Clean any prior rows for this probe.
    let _ = sqlx::query(
        "DELETE FROM request_log WHERE query = 'login' AND client_id = 'authfail_probe'",
    )
    .execute(&db)
    .await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .header("x-client-id", "authfail_probe")
                .body(Body::from(
                    r#"{"username":"no_such_user_authfail","password":"wrongpass123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    // login_user returns AppError::Unauthorized which maps to HTTP 401.
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM request_log WHERE etype = 'auth_failed' AND query = 'login' AND client_id = 'authfail_probe'"
    )
    .fetch_one(&db)
    .await
    .expect("count auth_failed");
    assert_eq!(
        count, 1,
        "failed login should be logged with etype=auth_failed"
    );

    let _ = sqlx::query(
        "DELETE FROM request_log WHERE query = 'login' AND client_id = 'authfail_probe'",
    )
    .execute(&db)
    .await;
}

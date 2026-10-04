//! DB-gated integration tests for Lane 4: site-wide image uploads API.
//!
//! Conventions match `tests/polls_api.rs` (global Mutex, #[ignore],
//! self-healing seeds). Run with `cargo test --test uploads_api
//! -- --include-ignored --test-threads=1`.
//!
//! Scope:
//!   * `upload_anonymous_rejects` — no auth → 401.
//!   * `upload_low_trust_rejects` — TL0 user → 403.
//!   * `upload_size_limit` — >10 MiB payload → 413 or 400.
//!   * `upload_mime_whitelist` — non-image bytes → 400.
//!   * `upload_png_roundtrip` — happy path: create → fetch-back → match.
//!   * `delete_authz_owner_ok_non_owner_forbidden` — owner ok; stranger 403.

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
    routing::{delete, get, post},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn ensure_env() {
    if std::env::var("DATABASE_URL").is_ok() && std::env::var("REDIS_URL").is_ok() {
        return;
    }
    let candidates = [
        std::env::var("FICNEXUS_DEV_ENV")
            .unwrap_or_else(|_| "~/.config/ficnexus-dev.env".to_string()),
        "~/.config/fichub-dev.env".to_string(),
    ];
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    for c in &candidates {
        let path = if c.starts_with('~') {
            format!("{home}/{}", &c[2..])
        } else {
            c.clone()
        };
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    if std::env::var(k).is_err() {
                        #[allow(unused_unsafe)]
                        unsafe {
                            std::env::set_var(k.trim(), v.trim());
                        }
                    }
                }
            }
            return;
        }
    }
}

// NOTE: `RedisBucketLimiter`'s shadowban probe (`is_shadowbanned`) runs
// `tokio::task::block_in_place`, which panics on the current-thread runtime, and
// `app()` builds that limiter for every test. So every test here must be
// `#[tokio::test(flavor = "multi_thread")]`, exactly as `admin_api` documents.
// The pool is also sized explicitly above for the same reason: each AppState
// holds connections for as long as its router lives.
static GUARD: OnceLock<Mutex<()>> = OnceLock::new();

fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// A fresh pool per call.
///
/// This used to be a single process-wide `OnceLock<PgPool>` shared by every
/// test. `app()` builds an `AppState` per test that holds pooled connections for
/// as long as its router lives, and those routers are all still alive when the
/// next test asks for a connection - so the later tests waited on the pool's
/// 60s acquire timeout and failed in a non-deterministic order. A pool per
/// caller means a test's connections are released with its own `AppState`.
async fn pool() -> sqlx::PgPool {
    ensure_env();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("connect to Postgres")
}

use fichub::routes::auth;
use fichub::server::AppState;

fn app() -> impl std::future::Future<Output = Router> {
    async {
        ensure_env();
        let config = fichub::config::Config::from_env();
        let db = pool().await;
        let redis_client =
            redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL for test");
        let redis = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("failed to connect to Redis");
        let http_client = reqwest::Client::builder()
            .user_agent("fichub-test/0.1.0")
            .build()
            .expect("build reqwest");
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
                .expect("health redis"),
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
            .route("/api/uploads", post(fichub::routes::uploads::upload_image))
            .route(
                "/uploads/forum/{yy}/{mm}/{name}",
                get(fichub::routes::uploads::serve_upload),
            )
            .route(
                "/api/uploads/{id}",
                delete(fichub::routes::uploads::delete_upload),
            )
            .with_state(state)
    }
}

/// Seed a user at an explicit trust level and return its id.
///
/// The upload routes gate on `assert_min_trust(..., PUBLISH_MIN_TRUST)`,
/// which is 2 and which reads `users.trust_level` **from the database** - not
/// from the JWT. The `trust_level` inside `auth_header`'s token is therefore
/// irrelevant to that gate.
///
/// These tests used to hardcode `auth_header(1, ...)` and hope user 1 was
/// trusted, which is true only on a database someone set up by hand. On a
/// database from `scripts/provision_test_db.sh`, user 1 is
/// `adminit_autotag_admin` with `trust_level = 0`, so every upload in this
/// suite got 403 - including the tests that were never about authorization at
/// all, like the size limit and MIME whitelist checks.
async fn seed_user(name: &str, trust_level: i16) -> i32 {
    use sqlx::Row;
    // A dedicated connection, not the shared POOL. `app()` builds an AppState
    // per test that holds pooled connections for as long as the router lives,
    // and seeding after that point could wait for a free one until the pool's
    // 60s acquire timeout. Opening one connection for the insert sidesteps the
    // contention entirely and is what this helper needs anyway: a single row.
    use sqlx::Connection;
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let mut conn = sqlx::postgres::PgConnection::connect(&url)
        .await
        .unwrap_or_else(|e| panic!("seed_user {name}: connect: {e}"));
    let row = sqlx::query(
        r#"INSERT INTO users (username, password_hash, trust_level, level, created_at)
           VALUES ($1, 'x', $2, 100, NOW())
           ON CONFLICT (username) DO UPDATE SET trust_level = EXCLUDED.trust_level
           RETURNING id"#,
    )
    .bind(name)
    .bind(trust_level)
    .fetch_one(&mut conn)
    .await
    .unwrap_or_else(|e| panic!("seed_user {name}: {e}"));
    row.get("id")
}

fn auth_header(user_id: i32, username: &str) -> String {
    let secret = TEST_JWT_SECRET;
    let user = auth::User {
        id: user_id,
        username: username.into(),
        trust_level: 1,
        reputation: 0,
        email: Some("test@example.com".into()),
        level: 100,
        exp: 0,
        is_admin: false,
    };
    let token = auth::create_token(&user, &secret).expect("create_token should succeed");
    format!("Bearer {}", token)
}

fn png_1x1() -> &'static [u8] {
    &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

fn text_blob() -> Vec<u8> {
    b"Hello, world! This is plain text.\n".to_vec()
}

fn multipart_body(boundary: &str, filename: &str, content_type: &str, data: &[u8]) -> Vec<u8> {
    let mut parts = Vec::new();
    parts.extend_from_slice(format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
    ).as_bytes());
    parts.extend_from_slice(data);
    parts.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    parts
}

async fn json_body(resp: axum::response::Response) -> Value {
    let b = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap_or_default();
    serde_json::from_slice(&b).unwrap_or(json!({"err": -1, "msg": "parse fail"}))
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// POST /api/uploads without Authorization → expects 401 Unauthorized.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn upload_anonymous_rejects() {
    let _guard = db_guard();
    let router = app().await;

    let payload = multipart_body("b1", "test.png", "image/png", png_1x1());

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b1")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: Value = json_body(resp).await;
    assert_eq!(body["err"].as_i64().unwrap(), 401);
}

/// POST /api/uploads with a low-trust user → expects 403 Forbidden.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn upload_low_trust_rejects() {
    let _guard = db_guard();
    let router = app().await;
    // PUBLISH_MIN_TRUST is 2 and is read from `users.trust_level` in the
    // database, so this test seeds a user one below the threshold and asserts a
    // flat 403. The previous version hardcoded user 1 and accepted either 200
    // or 403, which is not an assertion.
    let user_id = seed_user("upl_lowtrust", 1).await; // TL1: below the gate
    let token = auth_header(user_id, "upl_lowtrust");

    let payload = multipart_body("b2", "test.png", "image/png", png_1x1());

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b2")
                .header("Authorization", token)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    // A flat 403: this test exists to prove the gate rejects, so anything else
    // is a failure. The previous version accepted 200 or 403, which passes
    // whether or not the gate works.
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "TL1 user must be refused the upload"
    );
}

/// POST /api/uploads with a >10 MiB payload is rejected.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn upload_size_limit_rejects_over_max() {
    use fichub::routes::uploads::MAX_UPLOAD_BYTES;
    let _guard = db_guard();
    let router = app().await;
    let uid = seed_user("upl_sizetest", 3).await; // TL3: clears PUBLISH_MIN_TRUST (2)
    let token = auth_header(uid, "upl_sizetest");

    // Build a payload just over the limit
    let big_data = vec![0u8; MAX_UPLOAD_BYTES + 1];
    let payload = multipart_body("b3", "big.png", "image/png", &big_data);

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b3")
                .header("Authorization", token)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    // Either axum rejects with 413 (body limit) or handler returns 400 "too large"
    let status = resp.status();
    if status != StatusCode::PAYLOAD_TOO_LARGE && status != StatusCode::BAD_REQUEST {
        let body = json_body(resp).await;
        panic!("expected 413 or 400, got {status}: {body}");
    }
}

/// Upload a non-image file → the MIME whitelist rejects it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn upload_mime_whitelist_rejects_text() {
    let _guard = db_guard();
    let router = app().await;
    let uid = seed_user("upl_mimetest", 3).await; // TL3: clears PUBLISH_MIN_TRUST (2)
    let token = auth_header(uid, "mimetest");

    let payload = multipart_body("b4", "test.txt", "text/plain", &text_blob());

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b4")
                .header("Authorization", token)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = json_body(resp).await;
    // BadRequest returns err: -1
    assert_eq!(body["err"].as_i64().unwrap(), -1);
}

/// Happy-path round-trip: upload a PNG, fetch the public URL, compare bytes.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn upload_png_happy_path_roundtrip() {
    let _guard = db_guard();
    let router = app().await;
    let uid = seed_user("upl_roundtrip", 3).await; // TL3: clears PUBLISH_MIN_TRUST (2)
    let token = auth_header(uid, "upl_roundtrip");
    let body_png = png_1x1().to_vec();

    let payload = multipart_body("b5", "pixel.png", "image/png", &body_png);

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b5")
                .header("Authorization", token)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("upload request should complete");

    assert_eq!(resp.status(), StatusCode::OK);
    let json: Value = json_body(resp).await;
    assert_eq!(json["err"].as_i64().unwrap(), 0);
    assert!(json["url"].is_string());
    let url = json["url"].as_str().expect("url is string");
    let upload_id = json["id"].as_i64().expect("got upload id");

    // Fetch back publicly
    let fetch_resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("fetch request should complete");

    assert_eq!(fetch_resp.status(), StatusCode::OK);
    let fetched = axum::body::to_bytes(fetch_resp.into_body(), 1 << 20)
        .await
        .expect("to_bytes")
        .to_vec();
    assert_eq!(&fetched, body_png.as_slice());

    // Cleanup: delete the upload
    let del_resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/uploads/{upload_id}"))
                .header("Authorization", auth_header(uid, "upl_roundtrip"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("delete should complete");
    assert_eq!(del_resp.status(), StatusCode::OK);
}

/// DELETE authz: owner can delete their own upload; a different user gets 403.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn delete_authz_owner_ok_non_owner_forbidden() {
    let _guard = db_guard();
    let router = app().await;

    // Create an upload
    let owner_uid = seed_user("upl_deleteme", 3).await; // TL3: clears PUBLISH_MIN_TRUST (2)
    let owner_token = auth_header(owner_uid, "upl_deleteme");
    let payload = multipart_body("b6", "deltest.png", "image/png", png_1x1());

    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/uploads")
                .header("Content-Type", "multipart/form-data; boundary=b6")
                .header("Authorization", owner_token.clone())
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .expect("create upload for testing");

    assert_eq!(resp.status(), StatusCode::OK);
    let json: Value = json_body(resp).await;
    let upload_id = json["id"].as_i64().expect("got upload id");

    // Owner deletes → success
    let del_resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/uploads/{upload_id}"))
                .header("Authorization", owner_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("delete request should complete");

    assert_eq!(del_resp.status(), StatusCode::OK);
    let del_json: Value = json_body(del_resp).await;
    assert_eq!(del_json["err"].as_i64().unwrap(), 0);
}

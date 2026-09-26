//! DB-gated integration tests for the admin metadata correction endpoint
//! (`PUT /api/admin/works/{id}/metadata`, role >= 10).
//!
//! Conventions (same as `tests/admin_api.rs` / `tests/comment_triage_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test metadata_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::put,
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

const USERNAME: &str = "metadata_test_user";
const ADMIN_USERNAME: &str = "metadata_test_admin";
const TITLE_PREFIX: &str = "MetaTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the metadata-correction endpoint + real AppState.
async fn app() -> Router {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
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
        "http://127.0.0.1:1".into(),
        "llama3.1:8b".into(),
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
            "/api/admin/works/{id}/metadata",
            put(fichub::routes::admin::update_work_metadata),
        )
        .with_state(state)
}

/// JWT for the given user id + role (real token, real JWT_SECRET from env).
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
        is_admin: false,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Seed a user (role 0 default); returns its id. Idempotent.
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role, trust_level) VALUES ($1, 'test-hash', $2, LEAST($2, 6)) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role, trust_level = LEAST(EXCLUDED.trust_level, 6) RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(pool)
        .await
        .expect("seed_user failed")
        .get(0)
}

/// Seed a work + fic_info pair with prefixed title; returns (work_id, url_id).
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 4, 8000, 'orig desc', NOW() - interval '30 days',
                  NOW() - interval '1 day', 'ongoing',
                  'https://example.com/meta-test-1',
                  NULL, NULL, 4242, 4242, NULL)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            status = EXCLUDED.status"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .execute(pool)
    .await
    .expect("seed fic_info failed");

    let existing: Option<i32> =
        sqlx::query_scalar("SELECT id FROM works WHERE default_source_id = $1")
            .bind(url_id)
            .fetch_optional(pool)
            .await
            .expect("work lookup failed");

    let work_id = match existing {
        Some(id) => {
            sqlx::query("UPDATE works SET canonical_title = $1, canonical_author = $2, description = $3 WHERE id = $4")
                .bind(title)
                .bind(author)
                .bind("orig desc")
                .bind(id)
                .execute(pool)
                .await
                .expect("update seeded work failed");
            id
        }
        None => sqlx::query_scalar(
            "INSERT INTO works (canonical_title, canonical_author, description, default_source_id)
                 VALUES ($1, $2, 'orig desc', $3) RETURNING id",
        )
        .bind(title)
        .bind(author)
        .bind(url_id)
        .fetch_one(pool)
        .await
        .expect("seed work failed"),
    };

    // Re-link the fic_info source to this work (idempotent).
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    (work_id, url_id.to_string())
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'MetaTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'MetaTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ($1, $2)")
        .bind(USERNAME)
        .bind(ADMIN_USERNAME)
        .execute(pool)
        .await;
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = match body {
        Some(v) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────

/// Admin can correct title/author/status/description on a work; the change
/// lands on both `works` (canonical) and the default source `fic_info` row.
#[tokio::test]
#[ignore]
async fn admin_corrects_metadata_across_works_and_fic_info() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, ADMIN_USERNAME, 10).await;
    let (work_id, url_id) = seed_work(
        &db,
        "meta-test-a",
        &format!("{TITLE_PREFIX} Alpha"),
        "Character A",
    )
    .await;
    let app = app().await;
    let admin_token = auth_header(admin_id, 10, ADMIN_USERNAME);

    let (status, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        Some(&admin_token),
        Some(json!({
            "title": format!("{TITLE_PREFIX} Alpha (Corrected)"),
            "author": "Character B",
            "status": "complete",
            "description": "A corrected description.",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "update failed: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(
        body["work"]["canonical_title"],
        format!("{TITLE_PREFIX} Alpha (Corrected)")
    );
    assert_eq!(body["work"]["canonical_author"], "Character B");
    assert_eq!(body["work"]["description"], "A corrected description.");

    // Canonical works row.
    let row: (String, String, String) = sqlx::query_as(
        "SELECT canonical_title, canonical_author, description FROM works WHERE id = $1",
    )
    .bind(work_id)
    .fetch_one(&db)
    .await
    .expect("work row");
    assert_eq!(row.0, format!("{TITLE_PREFIX} Alpha (Corrected)"));
    assert_eq!(row.1, "Character B");
    assert_eq!(row.2, "A corrected description.");

    // Default source fic_info synced.
    let fi: (String, String, String, String) =
        sqlx::query_as("SELECT title, author, status, description FROM fic_info WHERE id = $1")
            .bind(&url_id)
            .fetch_one(&db)
            .await
            .expect("fic_info row");
    assert_eq!(fi.0, format!("{TITLE_PREFIX} Alpha (Corrected)"));
    assert_eq!(fi.1, "Character B");
    assert_eq!(fi.2, "complete");
    assert_eq!(fi.3, "A corrected description.");
}

/// Partial update: only the provided fields change; the rest are untouched.
#[tokio::test]
#[ignore]
async fn partial_update_only_touches_provided_fields() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, ADMIN_USERNAME, 10).await;
    let (work_id, url_id) = seed_work(
        &db,
        "meta-test-b",
        &format!("{TITLE_PREFIX} Beta"),
        "Character A",
    )
    .await;
    let app = app().await;
    let admin_token = auth_header(admin_id, 10, ADMIN_USERNAME);

    let (status, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        Some(&admin_token),
        Some(json!({ "author": "Character B" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "partial update failed: {body}");
    assert_eq!(body["work"]["canonical_author"], "Character B");
    assert_eq!(
        body["work"]["canonical_title"],
        format!("{TITLE_PREFIX} Beta"),
        "title untouched"
    );

    let fi: (String, String) = sqlx::query_as("SELECT title, author FROM fic_info WHERE id = $1")
        .bind(&url_id)
        .fetch_one(&db)
        .await
        .expect("fic_info row");
    assert_eq!(
        fi.0,
        format!("{TITLE_PREFIX} Beta"),
        "fic_info title untouched"
    );
    assert_eq!(fi.1, "Character B");
}

/// Role < 10 gets 403; anonymous gets 401-style login-required.
#[tokio::test]
#[ignore]
async fn metadata_endpoint_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME, 0).await;
    let (work_id, _) = seed_work(
        &db,
        "meta-test-c",
        &format!("{TITLE_PREFIX} Gamma"),
        "Character A",
    )
    .await;
    let app = app().await;
    let user_token = auth_header(user_id, 0, USERNAME);

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        Some(&user_token),
        Some(json!({ "title": "Nope" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "role 0 must be forbidden");

    let (status, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        None,
        Some(json!({ "title": "Nope" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "anonymous must be forbidden: {body}"
    );
    assert_eq!(body["err"], -403, "anonymous body: {body}");
}

/// Empty title and empty payload are rejected; missing work is 404.
#[tokio::test]
#[ignore]
async fn metadata_endpoint_validates_input() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, ADMIN_USERNAME, 10).await;
    let (work_id, _) = seed_work(
        &db,
        "meta-test-d",
        &format!("{TITLE_PREFIX} Delta"),
        "Character A",
    )
    .await;
    let app = app().await;
    let admin_token = auth_header(admin_id, 10, ADMIN_USERNAME);

    // Empty title → 400 err -1.
    let (status, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        Some(&admin_token),
        Some(json!({ "title": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty title: {body}");
    assert_eq!(body["err"], -1);

    // Empty payload → 400 err -1.
    let (status, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/works/{work_id}/metadata"),
        Some(&admin_token),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty payload: {body}");
    assert_eq!(body["err"], -1);

    // Missing work → 404.
    let (status, _) = send(
        &app,
        "PUT",
        "/api/admin/works/999999999/metadata",
        Some(&admin_token),
        Some(json!({ "title": "X" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "missing work must 404");
}

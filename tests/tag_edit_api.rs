//! DB-gated integration tests for the curated tag-edit endpoint:
//! PUT /api/curator/tags/{id} (description / tag_type_id / optional canonical).
//!
//! Conventions (same as tests/curator_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test tag_edit_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.
//! * The endpoint gates on the shared CURATOR_TOKEN (HTTP 400 with err -1
//!   when missing — same as the other tag-curator endpoints).

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{delete, get, post, put},
};
use serde_json::{Value, json};
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

/// Build a router with the curator tag endpoints + a real AppState.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let mut config = fichub::config::Config::from_env();
    // Local dev .env has no CURATOR_TOKEN; pin one so the tag-curator
    // endpoints (which compare against config.curator_token) are exercisable.
    config.curator_token = Some("fichub-dev-curator-token".into());
    // Generous tag rate limits (Redis-backed, keyed on request IP).
    config.tag_submit_limit_per_hour = 100_000;
    config.tag_vote_limit_per_hour = 100_000;
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
            "/api/curator/tags/{id}",
            put(fichub::tags::curator::update_tag),
        )
        .route(
            "/api/curator/tags/{id}",
            delete(fichub::tags::curator::delete_tag),
        )
        .route(
            "/api/curator/alias",
            post(fichub::tags::curator::create_alias),
        )
        .route(
            "/api/curator/merge",
            post(fichub::tags::curator::merge_tags),
        )
        .route("/api/curator/flags", get(fichub::tags::curator::list_flags))
        .route(
            "/api/curator/flags/{id}/resolve",
            post(fichub::tags::curator::resolve_flag),
        )
        .with_state(state)
}

/// Role-based auth for tag-curator endpoints: the endpoints gate on a
/// logged-in user with role >= 10 (admin), so the JWT is minted directly
/// for the role-10 user (no shared static token).
fn curator_auth(user_id: i32, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level: 10,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Seed a user; returns its id. Idempotent (uses ON CONFLICT DO NOTHING).
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, email, role) VALUES ($1, 'x', $2, $3) ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .bind(username)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user: lookup failed")
}

/// Seed a tag row; returns its id. `ON CONFLICT (name) DO NOTHING`.
async fn seed_tag(pool: &sqlx::PgPool, name: &str, type_id: i16) -> i32 {
    sqlx::query(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
    )
    .bind(name)
    .bind(type_id)
    .execute(pool)
    .await
    .expect("seed_tag failed");
    sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed_tag: lookup failed")
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM tags WHERE name LIKE 'TagEditTest %'")
        .execute(pool)
        .await;
}

async fn put_json(
    app: &Router,
    uri: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// PUT updates description + tag_type_id and returns the updated row;
/// unset fields are preserved.
#[tokio::test]
#[ignore]
async fn tag_edit_updates_description_and_type() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let tag_id = seed_tag(&db, "TagEditTest Character A", 2).await;
    let app = app().await;
    let curator = seed_user(&db, "tagedit_curator", 10).await;
    let ca = curator_auth(curator, "tagedit_curator");

    // Anonymous → HTTP 400 err 401 (400-as-401 convention).
    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        None,
        json!({ "description": "nope" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "no auth: {b}");
    assert_eq!(b["err"], 401, "no auth err: {b}");

    // Update description + tag_type_id.
    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({ "description": "A curated description", "tag_type_id": 4 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "update: {b}");
    assert_eq!(b["err"], 0, "update err: {b}");
    assert_eq!(
        b["tag"]["description"], "A curated description",
        "body: {b}"
    );
    assert_eq!(b["tag"]["tag_type_id"], 4, "body: {b}");

    // The DB row reflects the update.
    let (desc, type_id): (Option<String>, i16) =
        sqlx::query_as("SELECT description, tag_type_id FROM tags WHERE id = $1")
            .bind(tag_id)
            .fetch_one(&db)
            .await
            .expect("tag row");
    assert_eq!(desc.as_deref(), Some("A curated description"));
    assert_eq!(type_id, 4);

    // Unset fields are preserved: only description changes.
    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({ "description": "Second description" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "partial update: {b}");
    assert_eq!(b["err"], 0, "partial update err: {b}");
    assert_eq!(b["tag"]["description"], "Second description");
    assert_eq!(b["tag"]["tag_type_id"], 4, "type preserved: {b}");

    // Empty update → err -1.
    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "empty update: {b}");
    assert_eq!(b["err"], -1, "empty update err: {b}");

    // Missing tag → err -5.
    let (s, b) = put_json(
        &app,
        "/api/curator/tags/999999999",
        Some(&ca),
        json!({ "description": "x" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "missing tag: {b}");
    assert_eq!(b["err"], -5, "missing tag err: {b}");

    cleanup(&db).await;
}

/// PUT with a bogus tag_type_id is rejected (err -1); a real one is applied.
#[tokio::test]
#[ignore]
async fn tag_edit_validates_tag_type_id() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let tag_id = seed_tag(&db, "TagEditTest Character B", 2).await;
    let app = app().await;
    let curator = seed_user(&db, "tagedit_curator_b", 10).await;
    let ca = curator_auth(curator, "tagedit_curator_b");

    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({ "tag_type_id": 999 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "bad type: {b}");
    assert_eq!(b["err"], -1, "bad type err: {b}");
    assert!(
        b["msg"].to_string().contains("unknown tag_type_id"),
        "msg: {b}"
    );

    // A valid type is applied.
    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({ "tag_type_id": 3 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "good type: {b}");
    assert_eq!(b["err"], 0, "good type err: {b}");
    assert_eq!(b["tag"]["tag_type_id"], 3, "body: {b}");

    cleanup(&db).await;
}

/// PUT with `canonical` works when the column exists, and is silently
/// ignored (no error) when the base schema has no canonical column.
#[tokio::test]
#[ignore]
async fn tag_edit_canonical_flag_is_optional() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let tag_id = seed_tag(&db, "TagEditTest Character C", 2).await;
    let app = app().await;
    let curator = seed_user(&db, "tagedit_curator_c", 10).await;
    let ca = curator_auth(curator, "tagedit_curator_c");

    let has_canonical: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tags' AND column_name = 'canonical')",
    )
    .fetch_one(&db)
    .await
    .expect("canonical probe");

    let (s, b) = put_json(
        &app,
        &format!("/api/curator/tags/{tag_id}"),
        Some(&ca),
        json!({ "canonical": true, "description": "canonical-ish" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "canonical update: {b}");
    assert_eq!(b["err"], 0, "canonical update err: {b}");
    assert_eq!(b["tag"]["description"], "canonical-ish");

    if has_canonical {
        assert_eq!(b["tag"]["canonical"], true, "body: {b}");
        let stored: bool = sqlx::query_scalar("SELECT canonical FROM tags WHERE id = $1")
            .bind(tag_id)
            .fetch_one(&db)
            .await
            .expect("canonical value");
        assert!(stored, "canonical persisted");
    } else {
        assert!(b["tag"]["canonical"].is_null(), "canonical ignored: {b}");
    }

    cleanup(&db).await;
}

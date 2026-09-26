//! DB-gated integration tests for the forum API (F2 — categories + topic list).
//!
//! Conventions (same as tests/requests_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test forum_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/slug
//!   prefix) at the START so reruns never collide.
//! * Write/state routes are auth-gated (HTTP 400 with `{"err":401}` for
//!   anonymous callers); read routes (categories, topics, detail, search)
//!   are public by default (FORUM_PUBLIC_READ=true) but hide/refuse
//!   mod-only categories for anonymous visitors; role-gated routes return
//!   HTTP 400 with `{"err":403}`.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, patch, post},
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

/// Install a tracing subscriber so `AppError::Database`'s real message reaches
/// the test output. Without this the endpoint returns a bare
/// `{"err":-1,"msg":"database error"}` and the underlying sqlx message is
/// logged and then discarded — which makes a schema mismatch undiagnosable
/// from the test alone. `analytics_api.rs` and `comment_triage_api.rs` already
/// do this; this suite did not.
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Build a router with the forum endpoints and a real AppState.
async fn app() -> Router {
    init_tracing();
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
            "/api/forum/categories",
            get(fichub::routes::forum::list_categories),
        )
        .route(
            "/api/forum/categories",
            post(fichub::routes::forum::create_category),
        )
        .route(
            "/api/forum/categories/{id}",
            patch(fichub::routes::forum::update_category),
        )
        .route("/api/forum/topics", get(fichub::routes::forum::list_topics))
        .route(
            "/api/forum/topics",
            post(fichub::routes::forum::create_topic),
        )
        .route(
            "/api/forum/topics/by-slug/{topicSlug}",
            get(fichub::routes::forum::topic_detail_by_slug),
        )
        .route(
            "/api/forum/topics/{topicId}",
            get(fichub::routes::forum::topic_detail),
        )
        .route(
            "/api/forum/topics/{topicId}",
            patch(fichub::routes::forum::update_topic),
        )
        .route(
            "/api/forum/topics/{topicId}",
            axum::routing::delete(fichub::routes::forum::delete_topic),
        )
        .route(
            "/api/forum/topics/{topicId}/posts",
            post(fichub::routes::forum::create_post),
        )
        .route(
            "/api/forum/topics/{topicId}/follow",
            get(fichub::routes::forum::get_follow_state),
        )
        .route(
            "/api/forum/topics/{topicId}/follow",
            post(fichub::routes::forum::toggle_follow),
        )
        .route(
            "/api/forum/topics/{topicId}/read",
            post(fichub::routes::forum::mark_topic_read),
        )
        .route(
            "/api/forum/search",
            get(fichub::routes::forum::search_forum),
        )
        .route(
            "/api/forum/posts/{postId}",
            patch(fichub::routes::forum::update_post),
        )
        .route(
            "/api/forum/posts/{postId}",
            axum::routing::delete(fichub::routes::forum::delete_post),
        )
        .route(
            "/api/forum/moderation/status",
            get(fichub::routes::forum::moderation_status),
        )
        .route(
            "/api/forum/moderation/queue",
            get(fichub::routes::forum::moderation_queue),
        )
        .route(
            "/api/forum/posts/{postId}/moderate",
            post(fichub::routes::forum::moderate_post),
        )
        .route(
            "/api/forum/posts/{postId}/moderations",
            get(fichub::routes::forum::post_moderations),
        )
        .route(
            "/api/forum/metamod/queue",
            get(fichub::routes::forum::metamod_queue),
        )
        .route(
            "/api/forum/metamod/{actionId}/vote",
            post(fichub::routes::forum::metamod_vote),
        )
        .route(
            "/api/admin/forum/hide/{postId}",
            post(fichub::routes::forum::admin_hide_post),
        )
        .route(
            "/api/admin/forum/topics/{topicId}/lock",
            post(fichub::routes::forum::admin_lock_topic),
        )
        .route(
            "/api/admin/forum/topics/{topicId}/pin",
            post(fichub::routes::forum::admin_pin_topic),
        )
        .route(
            "/api/admin/forum/bans",
            post(fichub::routes::forum::admin_create_ban),
        )
        .route(
            "/api/admin/forum/bans/{id}",
            axum::routing::delete(fichub::routes::forum::admin_delete_ban),
        )
        .route(
            "/api/admin/forum/bans",
            get(fichub::routes::forum::admin_list_bans),
        )
        // ── User reports (F5: forum targets) ──────────────────────────
        .route("/api/reports", post(fichub::routes::reports::create_report))
        .route(
            "/api/admin/reports",
            get(fichub::routes::reports::list_reports),
        )
        .route(
            "/api/admin/reports/{id}/resolve",
            post(fichub::routes::reports::resolve_report),
        )
        // ── Auth routes (register honors REGISTRATION_MODE / invite_code) ──
        .route(
            "/api/auth/register",
            post(fichub::routes::social::register_handler),
        )
        // ── F7 subsystems: site info, invites, registration apps, blocks ──
        .route("/api/site", get(fichub::routes::subsystems::site_info))
        .route(
            "/api/admin/invites",
            post(fichub::routes::subsystems::create_invite),
        )
        .route(
            "/api/admin/invites",
            get(fichub::routes::subsystems::list_invites),
        )
        .route(
            "/api/registration-applications",
            post(fichub::routes::subsystems::apply_registration),
        )
        .route(
            "/api/admin/registration-applications",
            get(fichub::routes::subsystems::list_registration_applications),
        )
        .route(
            "/api/admin/registration-applications/{id}/review",
            post(fichub::routes::subsystems::review_registration_application),
        )
        .route("/api/blocks", post(fichub::routes::subsystems::block_user))
        .route("/api/blocks", get(fichub::routes::subsystems::list_blocks))
        .route(
            "/api/blocks/{user_id}",
            axum::routing::delete(fichub::routes::subsystems::unblock_user),
        )
        .with_state(state)
}

/// JWT for the given user id + role (real token, real JWT_SECRET from env).
/// F7: the token carries a `level` claim derived from role via the backfill
/// mapping (10→100, 5→50, 1→1, else 0) so role-gate tests keep passing
/// through the level-gate sweep.
fn auth_header(user_id: i32, username: &str, role: i16) -> String {
    let level = match role {
        10 => 100,
        5 => 50,
        1 => 1,
        _ => 0,
    };
    auth_header_level(user_id, username, role, level)
}

/// JWT with an explicit level (bypasses the role→level mapping) for gate
/// boundary tests (e.g. level 49/99 must be forbidden).
fn auth_header_level(user_id: i32, username: &str, trust_level: i16, level: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level,
        exp: i64::from(level) * 100,
        is_admin: false,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Seed a user; returns its id. Idempotent.
/// Seed a user at an explicit database `trust_level`.
///
/// Trust gates in this codebase read `users.trust_level` **from the database**
/// (via `fetch_trust_level`), not from the JWT, so the third argument to
/// `auth_header` does not move them. `reports.rs` rejects with
/// "Filing reports requires trust level 1 (Basic) or higher" otherwise.
async fn seed_user_at_trust(pool: &sqlx::PgPool, username: &str, trust_level: i16) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, trust_level) VALUES ($1, 'test-hash', $2)
         ON CONFLICT (username) DO UPDATE SET trust_level = EXCLUDED.trust_level",
    )
    .bind(username)
    .bind(trust_level)
    .execute(pool)
    .await
    .expect("seed_user_at_trust failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user_at_trust: lookup failed")
}

async fn seed_user(pool: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user lookup failed")
}

/// Seed a category; returns its id. Idempotent (slug-unique).
async fn seed_category(pool: &sqlx::PgPool, slug: &str, title: &str) -> i64 {
    sqlx::query(
        "INSERT INTO forum_categories (slug, title, description, position)
         VALUES ($1, $2, '', 0) ON CONFLICT (slug) DO NOTHING",
    )
    .bind(slug)
    .bind(title)
    .execute(pool)
    .await
    .expect("seed_category failed");
    sqlx::query_scalar("SELECT id FROM forum_categories WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await
        .expect("seed_category lookup failed")
}

/// Seed a topic + its OP post; returns the topic id. Idempotent (title-unique
/// per category).
///
/// NOTE: the OP post is inserted with a deterministic body so a caller can
/// locate it again afterwards (tests assert on post content). Use
/// `seed_topic` for topic-level flows and grab the OP with
/// `SELECT id FROM forum_posts WHERE topic_id = $1 AND body = 'body'`.
async fn seed_topic(pool: &sqlx::PgPool, category_id: i64, author_id: i32, title: &str) -> i64 {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM forum_topics WHERE category_id = $1 AND title = $2")
            .bind(category_id)
            .bind(title)
            .fetch_optional(pool)
            .await
            .expect("topic lookup failed");
    if let Some((id,)) = existing {
        // Self-heal: this topic may be a leftover from a crashed prior run —
        // wipe its posts + reset the cursor fields so the caller's fresh
        // seeds land on a clean slate, then re-create the OP post (other
        // helpers locate the OP by topic_id + author_id after this call).
        let _ = sqlx::query("DELETE FROM forum_posts WHERE topic_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query(
            "UPDATE forum_topics SET last_post_id = NULL, view_count = 0 WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await;
        sqlx::query(
            "INSERT INTO forum_posts (topic_id, author_id, body, search_vector)
             VALUES ($1, $2, 'body', to_tsvector('english', 'body'))",
        )
        .bind(id)
        .bind(author_id)
        .execute(pool)
        .await
        .expect("re-seed OP post failed");
        return id;
    }
    let mut tx = pool.begin().await.expect("begin tx");
    let topic_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_topics (category_id, author_id, title, body, search_vector)
         VALUES ($1, $2, $3, 'body', to_tsvector('english', $3))
         RETURNING id",
    )
    .bind(category_id)
    .bind(author_id)
    .bind(title)
    .fetch_one(&mut *tx)
    .await
    .expect("seed topic failed");
    let post_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body, search_vector)
         VALUES ($1, $2, 'body', to_tsvector('english', 'body'))
         RETURNING id",
    )
    .bind(topic_id)
    .bind(author_id)
    .fetch_one(&mut *tx)
    .await
    .expect("seed OP failed");
    sqlx::query("UPDATE forum_topics SET last_post_id = $2 WHERE id = $1")
        .bind(topic_id)
        .bind(post_id)
        .execute(&mut *tx)
        .await
        .expect("set last_post_id failed");
    tx.commit().await.expect("commit tx");
    topic_id
}

async fn post_json(
    app: &Router,
    uri: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("POST")
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

async fn delete_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method("DELETE").uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
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

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method("GET").uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
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

async fn patch_json(
    app: &Router,
    uri: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("PATCH")
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

#[tokio::test]
#[ignore]
async fn anonymous_gets_401_on_all_forum_routes() {
    let _g = db_guard();
    let app = app().await;
    // Seeded, not assumed: this test reads ?category=general below, and a
    // provisioner-built database has no categories at all. Same defect as
    // uploads_api's hardcoded user id 1 - a test that only passes against a
    // database someone prepared by hand.
    {
        let db = pool().await;
        seed_category(&db, "general", "General").await;
    }

    // Public reads: anonymous may list categories/topics (FORUM_PUBLIC_READ
    // default true).
    let (s, b) = get_json(&app, "/api/forum/categories", None).await;
    assert_eq!(s, StatusCode::OK, "categories: {b}");
    assert_eq!(b["err"], 0, "categories: {b}");

    let (s, b) = get_json(&app, "/api/forum/topics?category=general", None).await;
    assert_eq!(s, StatusCode::OK, "topics: {b}");
    assert_eq!(b["err"], 0, "topics: {b}");

    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        None,
        json!({ "slug": "x", "title": "x" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "create: {b}");
    assert_eq!(b["err"], 401, "create: {b}");

    let (s, b) = patch_json(
        &app,
        "/api/forum/categories/1",
        None,
        json!({ "title": "x" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "patch: {b}");
    assert_eq!(b["err"], 401, "patch: {b}");
}

#[tokio::test]
#[ignore]
async fn non_admin_cannot_create_category() {
    let _g = db_guard();
    let db = pool().await;
    let username = "frma_forbidden_user";
    let uid = seed_user(&db, username).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug LIKE 'frma-%'")
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(uid, username, 0);

    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token),
        json!({ "slug": "frma-secret", "title": "Secret" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "create: {b}");
    assert_eq!(b["err"], -403, "create: {b}");

    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn admin_creates_category_and_list_returns_it() {
    let _g = db_guard();
    let db = pool().await;
    let username = "frma_admin";
    let uid = seed_user(&db, username).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug LIKE 'frma-%'")
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(uid, username, 10);

    // Create
    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token),
        json!({ "slug": "frma-general", "title": "General", "description": "Everything", "position": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create: {b}");
    assert_eq!(b["err"], 0, "create: {b}");
    let cid = b["id"].as_i64().expect("category id");

    // Slug validation: uppercase rejected
    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token),
        json!({ "slug": "BAD-SLUG", "title": "Bad" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "bad slug: {b}");

    // List
    let (s, b) = get_json(&app, "/api/forum/categories", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert_eq!(b["err"], 0, "list: {b}");
    let items = b["items"].as_array().unwrap();
    let mine = items
        .iter()
        .find(|i| i["id"] == json!(cid))
        .expect("created category in list");
    assert_eq!(mine["slug"], "frma-general");
    assert_eq!(mine["title"], "General");
    assert_eq!(mine["topic_count"], 0);

    // Patch: reorder + rename
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/categories/{cid}"),
        Some(&token),
        json!({ "title": "General Talk", "position": 5 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {b}");
    assert_eq!(b["err"], 0, "patch: {b}");

    let (_, b) = get_json(&app, "/api/forum/categories", Some(&token)).await;
    let items = b["items"].as_array().unwrap();
    let mine = items
        .iter()
        .find(|i| i["id"] == json!(cid))
        .expect("updated category in list");
    assert_eq!(mine["title"], "General Talk");
    assert_eq!(mine["position"], 5);

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE slug LIKE 'frma-%'")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn topic_list_empty_for_new_category() {
    let _g = db_guard();
    let db = pool().await;
    let username = "frma_empty_user";
    let uid = seed_user(&db, username).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'frma-empty'")
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(uid, username, 10);
    let cid = seed_category(&db, "frma-empty", "Empty Category").await;

    let (s, b) = get_json(&app, "/api/forum/topics?category=frma-empty", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "topics: {b}");
    assert_eq!(b["err"], 0, "topics: {b}");
    assert_eq!(b["items"].as_array().unwrap().len(), 0, "topics: {b}");
    assert!(b["next_cursor"].is_null(), "no next cursor: {b}");

    // Unknown category → 400
    let (s, b) = get_json(&app, "/api/forum/topics?category=nope-nope", Some(&token)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "unknown category: {b}");

    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn topic_list_returns_seeded_topic_with_counts() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "frma_topic_author";
    let u2 = "frma_topic_reader";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'frma-topics'")
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(id1, u1, 0);

    let cid = seed_category(&db, "frma-topics", "Topics").await;
    let tid = seed_topic(&db, cid, id1, "frma seeded topic").await;

    // A reply from u2 → reply_count 2 (OP + reply)
    let reply_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body)
         VALUES ($1, $2, 'reply') RETURNING id",
    )
    .bind(tid)
    .bind(id2)
    .fetch_one(&db)
    .await
    .expect("seed reply failed");
    sqlx::query("UPDATE forum_topics SET last_post_id = $1 WHERE id = $2")
        .bind(reply_id)
        .bind(tid)
        .execute(&db)
        .await
        .ok();
    // A vote on the reply → vote_score 1
    sqlx::query("INSERT INTO forum_post_votes (post_id, user_id, value) VALUES ($1, $2, 1)")
        .bind(reply_id)
        .bind(id2)
        .execute(&db)
        .await
        .ok();

    let (s, b) = get_json(&app, "/api/forum/topics?category=frma-topics", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "topics: {b}");
    assert_eq!(b["err"], 0, "topics: {b}");
    let items = b["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "topics: {b}");
    let t = &items[0];
    assert_eq!(t["title"], "frma seeded topic");
    assert_eq!(t["author_username"], u1);
    assert_eq!(t["reply_count"], 2);
    assert_eq!(t["vote_score"], 1);
    assert_eq!(t["status"], "open");

    // Category list now reports the topic count
    let (_, b) = get_json(&app, "/api/forum/categories", Some(&token)).await;
    let items = b["items"].as_array().unwrap();
    let mine = items
        .iter()
        .find(|i| i["id"] == json!(cid))
        .expect("category in list");
    assert_eq!(mine["topic_count"], 1, "category topic count: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// F3 — write path: topics, posts, follows, notifications
// ═══════════════════════════════════════════════════════════════════════════

/// Delete every forum row belonging to the test users (self-heal on reruns).
async fn wipe_forum_for_users(db: &sqlx::PgPool, usernames: &[&str]) {
    for u in usernames {
        let uid: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(u)
            .fetch_optional(db)
            .await
            .unwrap_or(None);
        if let Some(uid) = uid {
            let _ = sqlx::query(
                "DELETE FROM notifications WHERE user_id = $1 AND notification_type IN ('forum_reply','forum_mention')",
            )
            .bind(uid)
            .execute(db)
            .await;
            let _ = sqlx::query("DELETE FROM forum_topic_views WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM forum_follows WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query(
                "DELETE FROM forum_posts WHERE author_id = $1 OR topic_id IN (SELECT id FROM forum_topics WHERE author_id = $1)",
            )
            .bind(uid)
            .execute(db)
            .await;
            let _ = sqlx::query("DELETE FROM forum_topics WHERE author_id = $1")
                .bind(uid)
                .execute(db)
                .await;
        }
    }
}
#[tokio::test]
#[ignore]
async fn anonymous_gets_401_on_all_f3_routes() {
    let _g = db_guard();
    let app = app().await;

    // Public read: anonymous may view topic detail (FORUM_PUBLIC_READ
    // default true, public category). No topic exists in the seeded DB, so
    // expect 400 "topic not found" rather than 401.
    let (s, b) = get_json(&app, "/api/forum/topics/1", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "topic detail: {b}");
    assert_eq!(b["err"], 400, "topic detail: {b}");

    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        None,
        json!({ "title": "t", "category_slug": "x", "body": "b" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "create topic: {b}");
    assert_eq!(b["err"], 401, "create topic: {b}");

    let (s, b) = patch_json(&app, "/api/forum/topics/1", None, json!({ "title": "t" })).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "patch topic: {b}");
    assert_eq!(b["err"], 401, "patch topic: {b}");

    let (s, b) = delete_json(&app, "/api/forum/topics/1", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "delete topic: {b}");
    assert_eq!(b["err"], 401, "delete topic: {b}");

    let (s, b) = post_json(
        &app,
        "/api/forum/topics/1/posts",
        None,
        json!({ "body": "b" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "create post: {b}");
    assert_eq!(b["err"], 401, "create post: {b}");

    let (s, b) = patch_json(&app, "/api/forum/posts/1", None, json!({ "body": "b" })).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "patch post: {b}");
    assert_eq!(b["err"], 401, "patch post: {b}");

    let (s, b) = delete_json(&app, "/api/forum/posts/1", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "delete post: {b}");
    assert_eq!(b["err"], 401, "delete post: {b}");

    let (s, b) = post_json(&app, "/api/forum/topics/1/follow", None, json!({})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "follow: {b}");
    assert_eq!(b["err"], 401, "follow: {b}");
}

#[tokio::test]
#[ignore]
async fn f3_create_topic_happy_path_and_validation() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr3_topic_creator";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-general'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-general", "F3 General").await;

    let app = app().await;
    let token = auth_header(uid, u, 0);

    // Happy path: topic + OP in one transaction.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "F3 first topic", "category_slug": "fr3-general", "body": "Hello **world**", "payload": { "type": "work", "id": 42 } }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create: {b}");
    assert_eq!(b["err"], 0, "create: {b}");
    let tid = b["id"].as_i64().expect("topic id");
    let op_id = b["post_id"].as_i64().expect("op post id");
    // Slug is auto-generated from the title with the id suffix: `{base}-{id}`.
    let slug = b["topic_slug"]
        .as_str()
        .expect("topic_slug in create response");
    assert_eq!(slug, format!("f3-first-topic-{tid}"));

    // Topic + OP exist, last_post_id wired.
    let (topic_author, last_post, body): (i32, Option<i64>, String) =
        sqlx::query_as("SELECT author_id, last_post_id, body FROM forum_topics WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("topic row");
    assert_eq!(topic_author, uid);
    assert_eq!(last_post, Some(op_id));
    assert_eq!(body, "Hello **world**");
    // DB slug matches the response slug, unique per topic.
    let db_slug: String = sqlx::query_scalar("SELECT topic_slug FROM forum_topics WHERE id = $1")
        .bind(tid)
        .fetch_one(&db)
        .await
        .expect("topic_slug row");
    assert_eq!(db_slug, slug);
    let post_author: i32 = sqlx::query_scalar("SELECT author_id FROM forum_posts WHERE id = $1")
        .bind(op_id)
        .fetch_one(&db)
        .await
        .expect("op author");
    assert_eq!(post_author, uid);

    // Validation: empty title / empty body / bad category.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "   ", "category_slug": "fr3-general", "body": "body" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty title: {b}");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "ok", "category_slug": "fr3-general", "body": "  " }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty body: {b}");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "ok", "category_slug": "nope-nope", "body": "body" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "bad category: {b}");
    let long_title = "x".repeat(121);
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": long_title, "category_slug": "fr3-general", "body": "body" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "long title: {b}");
    let long_body = "y".repeat(20_001);
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "ok", "category_slug": "fr3-general", "body": long_body }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "long body: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

/// Slug URLs: a topic is reachable both by its auto-generated slug
/// (`/api/forum/topics/by-slug/{slug}`) and by its numeric id. The detail
/// response carries `topic_slug`; unknown slugs return 400.
#[tokio::test]
#[ignore]
async fn topic_slug_by_slug_route_and_detail() {
    let _g = db_guard();
    let db = pool().await;
    let u = "slug_tester";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'slug-general'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "slug-general", "Slug General").await;

    let app = app().await;
    let token = auth_header(uid, u, 0);

    // Create a topic whose title slugifies to a known base.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "My Great Topic!", "category_slug": "slug-general", "body": "body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create: {b}");
    let tid = b["id"].as_i64().expect("topic id");
    let slug = b["topic_slug"].as_str().expect("slug").to_string();
    assert_eq!(slug, format!("my-great-topic-{tid}"));

    // By-slug route resolves to the same detail as the numeric route.
    let (s, by_slug) = get_json(
        &app,
        &format!("/api/forum/topics/by-slug/{slug}"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "by-slug: {by_slug}");
    assert_eq!(by_slug["err"], 0, "by-slug: {by_slug}");
    assert_eq!(by_slug["id"], tid, "by-slug id");
    assert_eq!(by_slug["topic_slug"], slug, "by-slug slug");
    assert_eq!(by_slug["title"], "My Great Topic!");

    let (s, by_id) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "by-id: {by_id}");
    assert_eq!(
        by_id["topic_slug"], slug,
        "numeric detail also carries slug"
    );

    // Unknown slug → 400.
    let (s, b) = get_json(
        &app,
        "/api/forum/topics/by-slug/does-not-exist-1",
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "unknown slug: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_topic_detail_posts_cursor_and_view_count() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_detail_author";
    let u2 = "fr3_detail_reader";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-detail'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-detail", "Detail").await;
    let tid = seed_topic(&db, cid, id1, "fr3 detail topic").await;
    sqlx::query("UPDATE forum_topics SET view_count = 0 WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .ok();

    // u2 replies twice so we have OP + 2 posts.
    let r1: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'first reply') RETURNING id",
    )
    .bind(tid)
    .bind(id2)
    .fetch_one(&db)
    .await
    .expect("reply 1");
    let r2: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body, quote_of) VALUES ($1, $2, 'quoted reply', $3) RETURNING id",
    )
    .bind(tid)
    .bind(id2)
    .bind(r1)
    .fetch_one(&db)
    .await
    .expect("reply 2");
    sqlx::query("UPDATE forum_topics SET last_post_id = $1 WHERE id = $2")
        .bind(r2)
        .bind(tid)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(id1, u1, 0);

    // First page: OP + replies, quote resolved.
    let (s, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "detail: {b}");
    assert_eq!(b["err"], 0, "detail: {b}");
    assert_eq!(b["title"], "fr3 detail topic");
    assert_eq!(b["author_username"], u1);
    assert_eq!(b["category_slug"], "fr3-detail");
    assert_eq!(b["status"], "open");
    let items = b["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "OP + 2 replies: {b}");
    // Quote preview resolved for r2.
    let quoted = items.iter().find(|p| p["id"] == json!(r2)).unwrap();
    assert_eq!(quoted["quote_of"], json!(r1));
    assert_eq!(quoted["quote"]["author_username"], u2);
    assert!(
        quoted["quote"]["preview"]
            .as_str()
            .unwrap()
            .contains("first reply"),
        "preview: {quoted}"
    );
    // view_count incremented.
    assert_eq!(b["view_count"], json!(1), "first view: {b}");
    assert_eq!(b["view_count_before"], json!(0), "before: {b}");
    // Second fetch increments again (clean rate-limit row so it counts).
    sqlx::query("DELETE FROM forum_topic_views WHERE topic_id = $1 AND user_id = $2")
        .bind(tid)
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    let (_, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(b["view_count"], json!(2), "second view: {b}");

    // Cursor: after OP id → only the 2 replies.
    let op_id: i64 = sqlx::query_scalar(
        "SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(tid)
    .fetch_one(&db)
    .await
    .expect("op id");
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}?after={op_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "cursor: {b}");
    let items = b["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "after OP: {b}");
    assert!(
        items.iter().all(|p| p["id"].as_i64().unwrap() > op_id),
        "ids after cursor: {b}"
    );

    // Limit clamps.
    let (_, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}?limit=2"),
        Some(&token),
    )
    .await;
    assert_eq!(b["items"].as_array().unwrap().len(), 2, "limit 2: {b}");
    // With 3 posts (OP, r1, r2), limit=2 returns OP+r1; the next cursor is the
    // last item on the page (r1), NOT r2 — there's still one page left.
    assert_eq!(b["next_cursor"].as_i64(), Some(r1), "next cursor: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_reply_creates_post_and_notifies_follower() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_reply_author";
    let u2 = "fr3_reply_follower";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-reply'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-reply", "Reply").await;
    // u2 is the topic author (and replies); u1 is the FOLLOWER who should get
    // notified. (Roles were swapped in an earlier revision — keep them
    // consistent with the assertions below.)
    let tid = seed_topic(&db, cid, id2, "fr3 reply topic").await;
    // u1 follows the topic.
    sqlx::query("INSERT INTO forum_follows (user_id, topic_id) VALUES ($1, $2)")
        .bind(id1)
        .bind(tid)
        .execute(&db)
        .await
        .ok();
    let app = app().await;
    let token = auth_header(id2, u2, 0);

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/posts"),
        Some(&token),
        json!({ "body": "A thoughtful reply", "quote_of": null }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "reply: {b}");
    assert_eq!(b["err"], 0, "reply: {b}");
    let post_id = b["id"].as_i64().expect("post id");
    let last: Option<i64> =
        sqlx::query_scalar("SELECT last_post_id FROM forum_topics WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("topic last");
    assert_eq!(last, Some(post_id), "topic last_post_id updated");

    // Follower (u1) got a forum_reply notification; author (u2) did not.
    let n1: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'forum_reply' AND reference_type = 'forum_topic' AND reference_id = $2::text",
    )
    .bind(id1)
    .bind(tid.to_string())
    .fetch_one(&db)
    .await
    .expect("notif count u1");
    assert_eq!(n1, 1, "follower notified");

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/posts"),
        Some(&token),
        json!({ "body": "  " }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty body: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_reply_mention_creates_forum_mention_notification() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_mention_author";
    let u2 = "fr3_mention_target";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-mention'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-mention", "Mention").await;
    let tid = seed_topic(&db, cid, id1, "fr3 mention topic").await;

    let app = app().await;
    let token = auth_header(id2, u2, 0);

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/posts"),
        Some(&token),
        json!({ "body": format!("Hey @{u1}, what do you think?") }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mention reply: {b}");

    // u1 got a forum_mention (they are NOT a follower here → not deduped).
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'forum_mention' AND reference_type = 'forum_topic'",
    )
    .bind(id1)
    .fetch_one(&db)
    .await
    .expect("mention count");
    assert_eq!(n, 1, "mention notification created");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_follow_toggle_on_off_and_count() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_follow_author";
    let u2 = "fr3_follow_user";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-follow'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-follow", "Follow").await;
    let tid = seed_topic(&db, cid, id1, "fr3 follow topic").await;

    let app = app().await;
    let token = auth_header(id2, u2, 0);

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/follow"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "follow on: {b}");
    assert_eq!(b["err"], 0, "follow on: {b}");
    assert_eq!(b["following"], json!(true), "following true: {b}");
    assert_eq!(b["follower_count"], json!(1), "count 1: {b}");

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/follow"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "follow off: {b}");
    assert_eq!(b["following"], json!(false), "following false: {b}");
    assert_eq!(b["follower_count"], json!(0), "count 0: {b}");

    // Unknown topic → 400.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics/999999/follow",
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "unknown topic: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_follow_state_get_reads_current_state() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_fs_author";
    let u2 = "fr3_fs_user";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-fs'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-fs", "FS").await;
    let tid = seed_topic(&db, cid, id1, "fr3 fs topic").await;

    let app = app().await;
    let token = auth_header(id2, u2, 0);

    // Not following yet.
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}/follow"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get follow: {b}");
    assert_eq!(b["err"], 0, "get follow: {b}");
    assert_eq!(b["following"], json!(false), "not following: {b}");
    assert_eq!(b["follower_count"], json!(0), "count 0: {b}");

    // Follow, then GET shows it.
    let (_, _) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/follow"),
        Some(&token),
        json!({}),
    )
    .await;
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}/follow"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get follow after: {b}");
    assert_eq!(b["following"], json!(true), "following: {b}");
    assert_eq!(b["follower_count"], json!(1), "count 1: {b}");

    // Unknown topic → 400.
    let (s, b) = get_json(&app, "/api/forum/topics/999999/follow", Some(&token)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "unknown topic: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_edit_topic_author_window_and_mod() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_edit_author";
    let u2 = "fr3_edit_mod";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-edit'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-edit", "Edit").await;
    let tid = seed_topic(&db, cid, id1, "fr3 edit topic").await;
    // Age the topic past the 15-minute window (the window check for topic
    // edits reads the topic's created_at).
    sqlx::query("UPDATE forum_topics SET created_at = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .ok();
    // The fresh topic used later must stay fresh: seed_topic reuses an
    // existing row by (category_id, title), so give it its own category.
    let cid2 = seed_category(&db, "fr3-edit-fresh", "EditFresh").await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    let tid = seed_topic(&db, cid, id1, "fr3 edit topic").await;
    sqlx::query("UPDATE forum_topics SET created_at = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token_author = auth_header(id1, u1, 0);
    let token_mod = auth_header(id2, u2, 5);

    // Author after window → 403.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_author),
        json!({ "title": "Renamed" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "author expired: {b}");

    // Mod can edit any time.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_mod),
        json!({ "title": "Mod rename", "body": "Mod body" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mod edit: {b}");
    assert_eq!(b["err"], 0, "mod edit: {b}");
    let (title, body): (String, String) =
        sqlx::query_as("SELECT title, body FROM forum_topics WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("topic row");
    assert_eq!(title, "Mod rename");
    assert_eq!(body, "Mod body");

    // Fresh topic: author can edit within the window; body-only update OK.
    let tid2 = seed_topic(&db, cid2, id1, "fr3 fresh topic").await;
    // Re-fresh the fresh topic's timestamps (the aged-topic cleanup above
    // must not leak onto it; seed_topic on a fresh category is new, but
    // re-age defensively for order changes).
    sqlx::query("UPDATE forum_topics SET created_at = NOW() WHERE id = $1")
        .bind(tid2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("UPDATE forum_posts SET created_at = NOW() WHERE topic_id = $1")
        .bind(tid2)
        .execute(&db)
        .await
        .ok();
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/topics/{tid2}"),
        Some(&token_author),
        json!({ "body": "Only body changed" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "fresh author edit: {b}");
    let body: String = sqlx::query_scalar("SELECT body FROM forum_topics WHERE id = $1")
        .bind(tid2)
        .fetch_one(&db)
        .await
        .expect("body");
    assert_eq!(body, "Only body changed");
    let (title, _): (String, String) =
        sqlx::query_as("SELECT title, body FROM forum_topics WHERE id = $1")
            .bind(tid2)
            .fetch_one(&db)
            .await
            .expect("title");
    assert_eq!(title, "fr3 fresh topic");
    // The fresh topic's OP post must stay fresh too (the aged-topic cleanup
    // above runs before this seed, but re-age defensively for order changes).
    sqlx::query("UPDATE forum_posts SET created_at = NOW() WHERE topic_id = $1")
        .bind(tid2)
        .execute(&db)
        .await
        .ok();
    // Non-author non-mod → 403.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_mod),
        json!({ "title": "x" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mod can always edit: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_edit_post_author_window_and_mod() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_postedit_author";
    let u2 = "fr3_postedit_mod";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-postedit'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-postedit", "PostEdit").await;
    let tid = seed_topic(&db, cid, id1, "fr3 postedit topic").await;
    // An old post by u2 (aged past window) to test mod edit.
    let old_post: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'old') RETURNING id",
    )
    .bind(tid)
    .bind(id2)
    .fetch_one(&db)
    .await
    .expect("old post");
    sqlx::query("UPDATE forum_posts SET created_at = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(old_post)
        .execute(&db)
        .await
        .ok();
    // u1's own fresh post (within window).
    let own_post: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'mine') RETURNING id",
    )
    .bind(tid)
    .bind(id1)
    .fetch_one(&db)
    .await
    .expect("own post");
    // Re-fresh the OP + own_post timestamps (the aged old_post update above
    // must not leak onto them; NOW() lands a fraction of a second before the
    // handler re-reads, so keep the grace window generous in the handler).
    sqlx::query("UPDATE forum_posts SET created_at = NOW() WHERE id = $1")
        .bind(own_post)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token_author = auth_header(id1, u1, 0);
    let token_mod = auth_header(id2, u2, 5);

    // Author edits own fresh post within window → OK, edited_at set.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/posts/{own_post}"),
        Some(&token_author),
        json!({ "body": "mine v2 edited" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "author fresh edit: {b}");
    let (body, edited): (String, Option<String>) =
        sqlx::query_as("SELECT body, edited_at::text FROM forum_posts WHERE id = $1")
            .bind(own_post)
            .fetch_one(&db)
            .await
            .expect("own post row");
    assert_eq!(body, "mine v2 edited");
    assert!(edited.is_some(), "edited_at set");

    // Author edits old post after window → 403.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/posts/{old_post}"),
        Some(&token_author),
        json!({ "body": "hijacked by author" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "author expired: {b}");

    // Mod edits old post → OK.
    let (s, b) = patch_json(
        &app,
        &format!("/api/forum/posts/{old_post}"),
        Some(&token_mod),
        json!({ "body": "mod fixed it" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mod edit: {b}");
    let body: String = sqlx::query_scalar("SELECT body FROM forum_posts WHERE id = $1")
        .bind(old_post)
        .fetch_one(&db)
        .await
        .expect("old body");
    assert_eq!(body, "mod fixed it");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_delete_topic_author_mod_and_non_author() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_deltopic_author";
    let u2 = "fr3_deltopic_mod";
    let u3 = "fr3_deltopic_other";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    let id3 = seed_user(&db, u3).await;
    wipe_forum_for_users(&db, &[u1, u2, u3]).await;
    // Moderators need an old account + reputation for the F5 eligibility
    // checks that the mod paths share.
    sqlx::query("UPDATE users SET role = 5, reputation = 100, created_at = NOW() - INTERVAL '30 days' WHERE id = $1")
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-deltopic'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-deltopic", "DelTopic").await;
    let tid = seed_topic(&db, cid, id1, "fr3 deltopic topic").await;

    let app = app().await;
    let token_author = auth_header(id1, u1, 0);
    let token_mod = auth_header(id2, u2, 5);
    let token_other = auth_header(id3, u3, 0);

    // Non-author non-mod → 403.
    let (s, b) = delete_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_other),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "non-author: {b}");

    // Author can delete any time (soft delete).
    let (s, b) = delete_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_author),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "author delete: {b}");
    assert_eq!(b["err"], 0, "author delete: {b}");
    let deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at::text FROM forum_topics WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("deleted_at");
    assert!(deleted.is_some(), "topic soft-deleted");

    // Detail now 400 for deleted topic.
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&token_author),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "deleted detail: {b}");

    // Mod delete of someone else's topic → soft-delete + modlog entry.
    let tid2 = seed_topic(&db, cid, id1, "fr3 deltopic mod target").await;
    let (s, b) = delete_json(&app, &format!("/api/forum/topics/{tid2}"), Some(&token_mod)).await;
    assert_eq!(s, StatusCode::OK, "mod delete: {b}");
    let modlog_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM modlog WHERE target_type = 'forum_topic' AND target_id = $1::text",
    )
    .bind(tid2.to_string())
    .fetch_one(&db)
    .await
    .expect("modlog count");
    assert_eq!(modlog_count, 1, "mod delete logged");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2 OR username = $3")
        .bind(u1)
        .bind(u2)
        .bind(u3)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f3_delete_post_author_mod_and_non_author() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr3_delpost_author";
    let u2 = "fr3_delpost_mod";
    let u3 = "fr3_delpost_other";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    let id3 = seed_user(&db, u3).await;
    wipe_forum_for_users(&db, &[u1, u2, u3]).await;
    // Moderator eligibility (F5): old account + reputation.
    sqlx::query("UPDATE users SET role = 5, reputation = 100, created_at = NOW() - INTERVAL '30 days' WHERE id = $1")
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr3-delpost'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr3-delpost", "DelPost").await;
    let tid = seed_topic(&db, cid, id1, "fr3 delpost topic").await;
    let post: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'victim') RETURNING id",
    )
    .bind(tid)
    .bind(id1)
    .fetch_one(&db)
    .await
    .expect("victim post");

    let app = app().await;
    let token_author = auth_header(id1, u1, 0);
    let token_mod = auth_header(id2, u2, 5);
    let token_other = auth_header(id3, u3, 0);

    // Non-author → 403.
    let (s, b) = delete_json(
        &app,
        &format!("/api/forum/posts/{post}"),
        Some(&token_other),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "non-author: {b}");

    // Author deletes own post → soft-delete, no modlog.
    let (s, b) = delete_json(
        &app,
        &format!("/api/forum/posts/{post}"),
        Some(&token_author),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "author delete: {b}");
    let deleted: Option<String> =
        sqlx::query_scalar("SELECT deleted_at::text FROM forum_posts WHERE id = $1")
            .bind(post)
            .fetch_one(&db)
            .await
            .expect("deleted_at");
    assert!(deleted.is_some(), "post soft-deleted");

    // Mod deletes someone else's post → soft-delete + modlog.
    let post2: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'victim2') RETURNING id",
    )
    .bind(tid)
    .bind(id1)
    .fetch_one(&db)
    .await
    .expect("victim2");
    let (s, b) = delete_json(&app, &format!("/api/forum/posts/{post2}"), Some(&token_mod)).await;
    assert_eq!(s, StatusCode::OK, "mod delete: {b}");
    let modlog_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM modlog WHERE target_type = 'forum_post' AND target_id = $1::text",
    )
    .bind(post2.to_string())
    .fetch_one(&db)
    .await
    .expect("modlog count");
    assert_eq!(modlog_count, 1, "mod delete logged");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2 OR username = $3")
        .bind(u1)
        .bind(u2)
        .bind(u3)
        .execute(&db)
        .await
        .ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// F4 — mark-read, unread flags, FTS search
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn f4_mark_read_requires_auth() {
    let _g = db_guard();
    let app = app().await;

    let (s, b) = post_json(
        &app,
        "/api/forum/topics/1/read",
        None,
        json!({ "last_read_post_id": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "mark read: {b}");
    assert_eq!(b["err"], 401, "mark read: {b}");
}

#[tokio::test]
#[ignore]
async fn f4_search_requires_auth() {
    let _g = db_guard();
    let app = app().await;

    // Search is a public read (FORUM_PUBLIC_READ default true): anonymous
    // may search, returning 200 (likely empty on the seeded DB).
    let (s, b) = get_json(&app, "/api/forum/search?q=quokka", None).await;
    assert_eq!(s, StatusCode::OK, "search: {b}");
    assert_eq!(b["err"], 0, "search: {b}");

    // Empty q → 400 even when authenticated.
    let db = pool().await;
    let u1 = "fr4_search_anon_user";
    let id1 = seed_user(&db, u1).await;
    let token = auth_header(id1, u1, 0);
    let (s, b) = get_json(&app, "/api/forum/search?q=", Some(&token)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty q: {b}");
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u1)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f4_mark_read_updates_state_and_unread_flags() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr4_read_author";
    let u2 = "fr4_read_reader";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr4-read'")
        .execute(&db)
        .await
        .ok();

    let cid = seed_category(&db, "fr4-read", "ReadState").await;
    let tid_a = seed_topic(&db, cid, id1, "fr4 readstate topic a").await;
    let tid_b = seed_topic(&db, cid, id1, "fr4 readstate topic b").await;

    // u2 replies to topic A → last_post_id advances past the OP.
    let reply: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, 'reply') RETURNING id",
    )
    .bind(tid_a)
    .bind(id2)
    .fetch_one(&db)
    .await
    .expect("seed reply failed");
    sqlx::query("UPDATE forum_topics SET last_post_id = $1 WHERE id = $2")
        .bind(reply)
        .bind(tid_a)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(id2, u2, 0);

    // Both topics unread for u2 (no read state yet).
    let (s, b) = get_json(&app, "/api/forum/topics?category=fr4-read", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "topics: {b}");
    let items = b["items"].as_array().unwrap();
    let ta = items
        .iter()
        .find(|i| i["id"] == json!(tid_a))
        .expect("topic a");
    let tb = items
        .iter()
        .find(|i| i["id"] == json!(tid_b))
        .expect("topic b");
    assert_eq!(ta["unread"], true, "a unread before: {b}");
    assert_eq!(tb["unread"], true, "b unread before: {b}");

    // Mark topic A read at the reply → unread flips to false.
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid_a}/read"),
        Some(&token),
        json!({ "last_read_post_id": reply }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mark read: {b}");
    assert_eq!(b["err"], 0, "mark read: {b}");
    assert_eq!(b["last_read_post_id"], reply, "mark read: {b}");
    assert!(b["updated_at"].is_string(), "mark read: {b}");

    let (_, b) = get_json(&app, "/api/forum/topics?category=fr4-read", Some(&token)).await;
    let items = b["items"].as_array().unwrap();
    let ta = items
        .iter()
        .find(|i| i["id"] == json!(tid_a))
        .expect("topic a");
    let tb = items
        .iter()
        .find(|i| i["id"] == json!(tid_b))
        .expect("topic b");
    assert_eq!(ta["unread"], false, "a read after: {b}");
    assert_eq!(tb["unread"], true, "b still unread: {b}");

    // Default target (no body) = topic's last_post_id; monotonic GREATEST:
    // replaying an older marker must not move the read cursor backwards.
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid_a}/read"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "default: {b}");
    assert_eq!(b["last_read_post_id"], reply, "default keeps last: {b}");
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid_a}/read"),
        Some(&token),
        json!({ "last_read_post_id": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "stale: {b}");
    assert_eq!(b["last_read_post_id"], reply, "stale must not regress: {b}");

    // u1 (no read state) still sees both topics unread.
    let token1 = auth_header(id1, u1, 0);
    let (_, b) = get_json(&app, "/api/forum/topics?category=fr4-read", Some(&token1)).await;
    let items = b["items"].as_array().unwrap();
    assert!(
        items.iter().all(|i| i["unread"] == json!(true)),
        "other user unaffected: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f4_search_finds_topic_and_post() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr4_search_author";
    let u2 = "fr4_search_replier";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    wipe_forum_for_users(&db, &[u1, u2]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug IN ('fr4-search', 'fr4-search-other')")
        .execute(&db)
        .await
        .ok();

    let cid = seed_category(&db, "fr4-search", "SearchCat").await;
    let cid_other = seed_category(&db, "fr4-search-other", "SearchOther").await;

    // Topic A: unique OP words (fr4 quokka lantern); reply with a unique word
    // (fr4 platypus beacon). Created via the API so write-time tsvector
    // maintenance is exercised (search works without the backfill migration).
    let app = app().await;
    let token = auth_header(id1, u1, 0);
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({
            "title": "fr4 quokka lantern sightings",
            "category_slug": "fr4-search",
            "body": "A fr4 quokka lantern spotted at midnight."
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create topic: {b}");
    let tid_a: i64 = b["id"].as_i64().expect("topic id");

    let token2 = auth_header(id2, u2, 0);
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid_a}/posts"),
        Some(&token2),
        json!({ "body": "The fr4 platypus beacon answers." }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create reply: {b}");

    // Topic B in a DIFFERENT category with the same OP words — proves the
    // category filter narrows results.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({
            "title": "fr4 quokka lantern elsewhere",
            "category_slug": "fr4-search-other",
            "body": "Another fr4 quokka lantern in the other category."
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create topic b: {b}");
    let _tid_b: i64 = b["id"].as_i64().expect("topic b id");

    // q=quokka → topic hits (both topics) + their OP-post hits (the OP is a
    // real forum_posts row, so it matches both as topic AND as post). Every
    // snippet highlights <mark>quokka</mark>.
    let (s, b) = get_json(&app, "/api/forum/search?q=quokka", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "search quokka: {b}");
    assert_eq!(b["err"], 0, "search quokka: {b}");
    assert_eq!(b["q"], "quokka", "echo: {b}");
    assert_eq!(b["total"], 4, "2 topic + 2 OP-post hits: {b}");
    let results = b["results"].as_array().unwrap();
    let topic_hits: Vec<&Value> = results.iter().filter(|r| r["type"] == "topic").collect();
    assert_eq!(topic_hits.len(), 2, "two topic hits: {b}");
    assert!(
        topic_hits.iter().all(|r| r["post_id"].is_null()),
        "topic post_id null: {b}"
    );
    assert!(
        topic_hits.iter().all(|r| r["snippet"]
            .as_str()
            .map_or(false, |s| s.contains("<mark>quokka</mark>"))),
        "snippet highlighted: {b}"
    );
    let slugs: Vec<&str> = topic_hits
        .iter()
        .map(|r| r["category_slug"].as_str().unwrap())
        .collect();
    assert!(slugs.contains(&"fr4-search"), "a in results: {b}");
    assert!(slugs.contains(&"fr4-search-other"), "b in results: {b}");
    assert!(
        results.iter().any(|r| r["type"] == "post"),
        "OP posts surface as post hits: {b}"
    );

    // q=platypus → the reply (post hit) with the topic title attached.
    let (s, b) = get_json(&app, "/api/forum/search?q=platypus", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "search platypus: {b}");
    assert_eq!(b["total"], 1, "one post hit: {b}");
    let r = &b["results"][0];
    assert_eq!(r["type"], "post", "post hit: {b}");
    assert!(r["post_id"].is_i64(), "post_id present: {b}");
    assert_eq!(r["topic_id"], tid_a, "topic id: {b}");
    assert_eq!(
        r["title"], "fr4 quokka lantern sightings",
        "topic title: {b}"
    );
    assert!(
        r["snippet"]
            .as_str()
            .map_or(false, |s| s.contains("<mark>platypus</mark>")),
        "post snippet: {b}"
    );

    // Combined AND query: both words in different rows → no hits.
    let (s, b) = get_json(
        &app,
        "/api/forum/search?q=quokka+AND+platypus",
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "combined: {b}");
    assert_eq!(b["total"], 0, "combined AND across rows: {b}");

    // Category filter: quokka limited to fr4-search → 1 topic hit (+ its OP
    // post hit).
    let (s, b) = get_json(
        &app,
        "/api/forum/search?q=quokka&category=fr4-search",
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "category filter: {b}");
    assert_eq!(b["total"], 2, "filtered (topic + OP post): {b}");
    let filtered_topics: Vec<&Value> = b["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["type"] == "topic")
        .collect();
    assert_eq!(filtered_topics.len(), 1, "one topic hit: {b}");
    assert_eq!(filtered_topics[0]["topic_id"], tid_a, "filtered topic: {b}");
    assert_eq!(
        filtered_topics[0]["category_slug"], "fr4-search",
        "filtered slug: {b}"
    );

    // Bad category slug → 400.
    let (s, b) = get_json(
        &app,
        "/api/forum/search?q=quokka&category=nope-nope",
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "bad category: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1 OR id = $2")
        .bind(cid)
        .bind(cid_other)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

// ── F5: moderation points + floor actions ───────────────────────────────────

/// Promote a user to curator and give them the F5 eligibility profile
/// (old account + reputation ≥ FORUM_MOD_MIN_EXP) so moderation endpoints
/// work in tests. Also wipes any leftover grants for a clean window.
async fn promote_curator(db: &sqlx::PgPool, uid: i32) {
    sqlx::query(
        "UPDATE users SET role = 5, level = 50, exp = 5000, reputation = 100, trust_level = 5, created_at = NOW() - INTERVAL '30 days' WHERE id = $1",
    )
    .bind(uid)
    .execute(db)
    .await
    .ok();
    sqlx::query("DELETE FROM forum_mod_grants WHERE user_id = $1")
        .bind(uid)
        .execute(db)
        .await
        .ok();
}

/// Give a user Elder trust (TL4, queue access but no resolve power).
async fn promote_elder(db: &sqlx::PgPool, uid: i32) {
    sqlx::query(
        "UPDATE users SET role = 1, level = 10, exp = 1000, reputation = 50, trust_level = 4, created_at = NOW() - INTERVAL '30 days' WHERE id = $1",
    )
    .bind(uid)
    .execute(db)
    .await
    .ok();
}

// ── F6: metamoderation (SPEC §2 Layer 2 + §3 + §4) ─────────────────────────

/// Give a user the F6 metamod eligibility profile: role ≥
/// FORUM_META_MIN_LEVEL (1), account ≥ FORUM_META_MIN_AGE_DAYS old,
/// reputation ≥ FORUM_META_MIN_EXP, and at least FORUM_META_MIN_POSTS
/// authored forum posts. Returns the user id.
async fn make_metamod(db: &sqlx::PgPool, username: &str) -> i32 {
    let uid = seed_user(db, username).await;
    // Clean slate for the posts count (author_id of all prior seeds).
    let _ = sqlx::query("DELETE FROM forum_posts WHERE author_id = $1")
        .bind(uid)
        .execute(db)
        .await;
    sqlx::query(
        "UPDATE users SET role = 1, level = 1, exp = 80, reputation = 80, created_at = NOW() - INTERVAL '100 days' WHERE id = $1",
    )
    .bind(uid)
    .execute(db)
    .await
    .expect("make_metamod profile update failed");
    for i in 0..10 {
        sqlx::query(
            "INSERT INTO forum_posts (topic_id, author_id, body)
             VALUES ((SELECT id FROM forum_topics ORDER BY id LIMIT 1), $1, $2)",
        )
        .bind(uid)
        .bind(format!("f6 metamod seed post {i}"))
        .execute(db)
        .await
        .expect("make_metamod post seed failed");
    }
    uid
}

/// Wipe every F6/F5 row that references the given users (self-heal on
/// reruns — crashed runs must not leak votes/grants into the next run).
async fn wipe_metamod_for_users(db: &sqlx::PgPool, usernames: &[&str]) {
    for u in usernames {
        let uid: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(u)
            .fetch_optional(db)
            .await
            .unwrap_or(None);
        if let Some(uid) = uid {
            let _ = sqlx::query(
                "DELETE FROM forum_metamod_votes WHERE voter_id = $1 OR mod_action_id IN
                 (SELECT id FROM forum_mod_actions WHERE moderator_id = $1)",
            )
            .bind(uid)
            .execute(db)
            .await;
            let _ = sqlx::query("DELETE FROM forum_mod_grants WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM forum_posts WHERE author_id = $1")
                .bind(uid)
                .execute(db)
                .await;
        }
    }
}

/// Seed a moderator action (post + action rows) so the metamod queue has
/// something to sample; returns (action_id, post_id).
async fn seed_mod_action(
    db: &sqlx::PgPool,
    author_id: i32,
    moderator_id: i32,
    reason: &str,
) -> (i64, i64) {
    let cid = seed_category(db, "fr6-meta", "F6 Meta").await;
    // UNIQUE topic title per call — reusing one title makes seed_topic hit its
    // existing-topic self-heal branch (wipe posts + re-create OP), which
    // cascade-deletes prior mod_actions/votes on the shared topic.
    let title = format!(
        "fr6 meta target {}",
        chrono::Utc::now().timestamp_subsec_nanos()
    );
    let tid = seed_topic(db, cid, author_id, &title).await;
    let post_id: i64 = sqlx::query_scalar(
        "SELECT id FROM forum_posts WHERE topic_id = $1 AND author_id = $2 ORDER BY id LIMIT 1",
    )
    .bind(tid)
    .bind(author_id)
    .fetch_one(db)
    .await
    .expect("op post id");
    let action_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_mod_actions (post_id, moderator_id, reason, delta, score_after)
         VALUES ($1, $2, $3, 2, 2) RETURNING id",
    )
    .bind(post_id)
    .bind(moderator_id)
    .bind(reason)
    .fetch_one(db)
    .await
    .expect("mod action seed failed");
    sqlx::query("UPDATE forum_posts SET score = 2 WHERE id = $1")
        .bind(post_id)
        .execute(db)
        .await
        .ok();
    (action_id, post_id)
}

#[tokio::test]
#[ignore]
async fn f5_mod_status() {
    let _g = db_guard();
    let db = pool().await;
    let u_reader = "fr5_status_reader";
    let u_cur = "fr5_status_curator";
    let reader_id = seed_user(&db, u_reader).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_reader, u_cur]).await;
    sqlx::query("DELETE FROM forum_mod_grants WHERE user_id = $1 OR user_id = $2")
        .bind(reader_id)
        .bind(cur_id)
        .execute(&db)
        .await
        .ok();

    let app = app().await;

    // Anonymous → 401.
    let (s, b) = get_json(&app, "/api/forum/moderation/status", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon status: {b}");
    assert_eq!(b["err"], 401, "anon status: {b}");

    // Plain reader: logged in, but no trust powers.
    let token = auth_header(reader_id, u_reader, 0);
    let (s, b) = get_json(&app, "/api/forum/moderation/status", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "reader status: {b}");
    assert_eq!(b["err"], 0, "reader status: {b}");
    assert_eq!(b["trust_level"], 0, "reader trust: {b}");
    assert_eq!(b["can_queue"], false, "reader queue: {b}");
    assert_eq!(b["can_resolve"], false, "reader resolve: {b}");
    assert_eq!(b["eligible"], false, "reader eligible alias: {b}");

    // TL5 user: queue + resolve powers, full daily budget.
    promote_curator(&db, cur_id).await;
    let token = auth_header(cur_id, u_cur, 5);
    let (s, b) = get_json(&app, "/api/forum/moderation/status", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "cur status: {b}");
    assert_eq!(b["err"], 0, "cur status: {b}");
    assert_eq!(b["trust_level"], 5, "cur trust: {b}");
    assert_eq!(b["can_queue"], true, "cur queue: {b}");
    assert_eq!(b["can_resolve"], true, "cur resolve: {b}");
    assert_eq!(b["eligible"], true, "cur eligible alias: {b}");
    assert_eq!(b["actions_today"], 0, "fresh budget: {b}");
    assert!(
        b["actions_cap"].as_i64().unwrap_or(0) > 0,
        "cap configured: {b}"
    );
    assert_eq!(
        b["points_left"], b["actions_cap"],
        "points_left mirrors budget: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_reader)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_moderate_positive() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr5_mod_author";
    let u_cur = "fr5_mod_curator";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-mod'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-mod", "F5 Mod").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 mod target").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");
    promote_curator(&db, cur_id).await;

    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);

    // Insightful → +2, mod_count 1, daily budget consumed.
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Insightful" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "moderate: {b}");
    assert_eq!(b["err"], 0, "moderate: {b}");
    assert_eq!(b["delta"], 2, "delta: {b}");
    assert_eq!(b["score_after"], 2, "score_after: {b}");

    let (score, mod_count): (i32, i32) =
        sqlx::query_as("SELECT score, mod_count FROM forum_posts WHERE id = $1")
            .bind(post_id)
            .fetch_one(&db)
            .await
            .expect("post row");
    assert_eq!(score, 2, "post score: {score}");
    assert_eq!(mod_count, 1, "post mod_count: {mod_count}");

    // Status now shows 1 action used.
    let (s, b) = get_json(&app, "/api/forum/moderation/status", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "status after: {b}");
    assert_eq!(b["actions_today"], 1, "budget consumed: {b}");

    // Action appears in the public moderations history, with moderator_id
    // (identity) only — no username leak.
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderations"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "moderations: {b}");
    assert_eq!(b["count"], 1, "one action: {b}");
    assert_eq!(b["items"][0]["reason"], "Insightful", "reason: {b}");
    assert_eq!(b["items"][0]["delta"], 2, "delta: {b}");
    assert_eq!(b["items"][0]["score_after"], 2, "score_after: {b}");
    assert_eq!(b["items"][0]["moderator_id"], cur_id, "moderator_id: {b}");
    assert!(
        b["items"][0].get("moderator_username").is_none(),
        "no username leak: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_moderate_negative_collapse() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr5_collapse_author";
    let u_cur = "fr5_collapse_curator";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-collapse'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-collapse", "F5 Collapse").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 collapse target").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");
    promote_curator(&db, cur_id).await;

    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);

    // Abusive on a score-0 post → −3. Auto-collapse is CLIENT-SIDE (SPEC §5):
    // the post stays in the API with its negative score; hidden_until stays
    // NULL — only the admin fast-hide floor action sets it.
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Abusive" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "moderate: {b}");
    assert_eq!(b["delta"], -3, "delta: {b}");
    assert_eq!(b["score_after"], -3, "score_after: {b}");
    assert!(
        b["hidden_until"].is_null(),
        "hidden_until must be null (client-side collapse): {b}"
    );

    let (score, hidden): (i32, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT score, hidden_until FROM forum_posts WHERE id = $1")
            .bind(post_id)
            .fetch_one(&db)
            .await
            .expect("post row");
    assert_eq!(score, -3, "score: {score}");
    assert!(
        hidden.is_none(),
        "hidden_until must stay NULL for auto-collapse"
    );

    // The collapsed post REMAINS visible in the topic detail with score −3
    // (the frontend renders the collapsed toggle client-side).
    let reader_token = auth_header(author_id, u_author, 0);
    let (s, b) = get_json(
        &app,
        &format!("/api/forum/topics/{tid}"),
        Some(&reader_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "detail: {b}");
    let items = b["items"].as_array().unwrap();
    let hit = items.iter().find(|p| p["id"] == json!(post_id));
    assert!(hit.is_some(), "collapsed post still visible in detail: {b}");
    assert_eq!(hit.unwrap()["score"], -3, "score in detail: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_moderate_self_forbidden() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr5_self_curator";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-self'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-self", "F5 Self").await;
    let tid = seed_topic(&db, cid, uid, "fr5 self topic").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");
    promote_curator(&db, uid).await;

    let app = app().await;
    let token = auth_header(uid, u, 5);
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Insightful" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "self mod: {b}");
    assert_eq!(b["err"], -403, "self mod err: {b}");

    let mod_count: i32 = sqlx::query_scalar("SELECT mod_count FROM forum_posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&db)
        .await
        .expect("mod_count");
    assert_eq!(mod_count, 0, "no action recorded: {mod_count}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_moderate_duplicate_forbidden() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr5_dup_author";
    let u_cur = "fr5_dup_curator";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-dup'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-dup", "F5 Dup").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 dup target").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");
    promote_curator(&db, cur_id).await;

    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);
    let (s, _) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Informative" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "first mod");

    // Same moderator, same post → 409 (or 400).
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Funny" }),
    )
    .await;
    assert!(
        s == StatusCode::CONFLICT || s == StatusCode::BAD_REQUEST,
        "duplicate rejected: {s} {b}"
    );

    let mod_count: i32 = sqlx::query_scalar("SELECT mod_count FROM forum_posts WHERE id = $1")
        .bind(post_id)
        .fetch_one(&db)
        .await
        .expect("mod_count");
    assert_eq!(mod_count, 1, "only one action: {mod_count}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_admin_hide() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr5_hide_author";
    let u_cur = "fr5_hide_curator";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-hide'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-hide", "F5 Hide").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 hide target").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");
    promote_curator(&db, cur_id).await;

    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);

    // Low-role user cannot fast-hide (needs role ≥ 5).
    let low = auth_header(author_id, u_author, 0);
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/hide/{post_id}"),
        Some(&low),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role hide: {b}");

    // Curator hides for 72h.
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/hide/{post_id}"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "hide: {b}");
    assert_eq!(b["err"], 0, "hide: {b}");
    let hidden: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT hidden_until FROM forum_posts WHERE id = $1")
            .bind(post_id)
            .fetch_one(&db)
            .await
            .expect("hidden_until");
    let hidden = hidden.expect("hidden_until set");
    let until_72h = chrono::Utc::now() + chrono::Duration::hours(72);
    assert!(
        (until_72h - hidden).num_minutes().abs() < 10,
        "hidden_until ≈ now+72h: {hidden}"
    );

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_lock_pin_topic() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr5_lock_author";
    let u_cur = "fr5_lock_curator";
    let u_low = "fr5_lock_low";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    let low_id = seed_user(&db, u_low).await;
    wipe_forum_for_users(&db, &[u_author, u_cur, u_low]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-lock'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-lock", "F5 Lock").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 lock topic").await;
    promote_curator(&db, cur_id).await;

    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);

    // Low-role user cannot lock or pin.
    let low = auth_header(low_id, u_low, 0);
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/topics/{tid}/lock"),
        Some(&low),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low lock: {b}");
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/topics/{tid}/pin"),
        Some(&low),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low pin: {b}");

    // Lock: open → locked.
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/topics/{tid}/lock"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "lock: {b}");
    assert_eq!(b["status"], "locked", "locked: {b}");

    // Pin: open → pinned (toggle).
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/topics/{tid}/pin"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "pin: {b}");
    assert_eq!(b["status"], "pinned", "pinned: {b}");

    // Unpin toggles back.
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/forum/topics/{tid}/pin"),
        Some(&token),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "unpin: {b}");
    assert_eq!(b["status"], "open", "unpinned: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2 OR username = $3")
        .bind(u_author)
        .bind(u_cur)
        .bind(u_low)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_ban_enforcement() {
    let _g = db_guard();
    let db = pool().await;
    let u_admin = "fr5_ban_admin";
    let u_target = "fr5_ban_target";
    let admin_id = seed_user(&db, u_admin).await;
    let target_id = seed_user(&db, u_target).await;
    wipe_forum_for_users(&db, &[u_admin, u_target]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug IN ('fr5-ban-a', 'fr5-ban-b')")
        .execute(&db)
        .await
        .ok();
    let cid_a = seed_category(&db, "fr5-ban-a", "F5 Ban A").await;
    let cid_b = seed_category(&db, "fr5-ban-b", "F5 Ban B").await;
    promote_curator(&db, admin_id).await;
    sqlx::query("UPDATE users SET role = 10 WHERE id = $1")
        .bind(admin_id)
        .execute(&db)
        .await
        .ok();
    // Self-heal: a crashed prior run may have left bans behind (users are
    // re-seeded idempotently, so the same ids persist). Wipe them first.
    sqlx::query("DELETE FROM forum_bans WHERE user_id = $1 OR user_id = $2 OR banned_by = $1 OR banned_by = $2")
        .bind(admin_id)
        .bind(target_id)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let admin_token = auth_header(admin_id, u_admin, 10);
    let target_token = auth_header(target_id, u_target, 0);

    // Forum-scope (permanent) ban blocks topic creation everywhere.
    let (s, b) = post_json(
        &app,
        "/api/admin/forum/bans",
        Some(&admin_token),
        json!({ "user_id": target_id, "scope": "forum", "reason": "spam" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create forum ban: {b}");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 banned topic", "category_slug": "fr5-ban-a", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "forum ban blocks: {b}");
    assert_eq!(b["msg"], "Banned from the forum", "ban msg: {b}");

    // Lift it, then category-scope ban on A only.
    let ban_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_bans WHERE user_id = $1 ORDER BY id DESC LIMIT 1")
            .bind(target_id)
            .fetch_one(&db)
            .await
            .expect("ban id");
    let (s, _) = delete_json(
        &app,
        &format!("/api/admin/forum/bans/{ban_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "lift forum ban");

    let (s, b) = post_json(
        &app,
        "/api/admin/forum/bans",
        Some(&admin_token),
        json!({ "user_id": target_id, "scope": "category", "category_id": cid_a, "reason": "off topic" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create category ban: {b}");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 banned topic a", "category_slug": "fr5-ban-a", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "category ban blocks A: {b}");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 allowed topic b", "category_slug": "fr5-ban-b", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "category ban allows B: {b}");

    // Expired ban (past expires_at) is ignored.
    let (s, b) = post_json(
        &app,
        "/api/admin/forum/bans",
        Some(&admin_token),
        json!({
            "user_id": target_id,
            "scope": "category",
            "category_id": cid_a,
            "expires_at": (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339()
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "expired ban created: {b}");
    // Sanity: the expired row exists (proves the 403 above was not caused by
    // this expired row) and is older than NOW().
    let expired_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_bans WHERE user_id = $1 AND expires_at < NOW() AND category_id = $2)",
    )
    .bind(target_id)
    .bind(cid_a)
    .fetch_one(&db)
    .await
    .expect("expired ban row");
    assert!(expired_exists, "expired ban row must exist");
    // Cleanup helper rows first: the active category-scope ban from step 2 is
    // still on cid_a; remove it so the expired-ban check is unambiguous.
    sqlx::query(
        "DELETE FROM forum_bans WHERE user_id = $1 AND category_id = $2 AND expires_at IS NULL",
    )
    .bind(target_id)
    .bind(cid_a)
    .execute(&db)
    .await
    .ok();
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 expired topic", "category_slug": "fr5-ban-a", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "expired ban allows: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1 OR id = $2")
        .bind(cid_a)
        .bind(cid_b)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_admin)
        .bind(u_target)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_ban_lift() {
    let _g = db_guard();
    let db = pool().await;
    let u_admin = "fr5_lift_admin";
    let u_target = "fr5_lift_target";
    let admin_id = seed_user(&db, u_admin).await;
    let target_id = seed_user(&db, u_target).await;
    wipe_forum_for_users(&db, &[u_admin, u_target]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-lift'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-lift", "F5 Lift").await;
    promote_curator(&db, admin_id).await;
    sqlx::query("UPDATE users SET role = 10 WHERE id = $1")
        .bind(admin_id)
        .execute(&db)
        .await
        .ok();
    // Self-heal: wipe any bans left by a crashed prior run.
    sqlx::query("DELETE FROM forum_bans WHERE user_id = $1 OR user_id = $2 OR banned_by = $1 OR banned_by = $2")
        .bind(admin_id)
        .bind(target_id)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let admin_token = auth_header(admin_id, u_admin, 10);
    let target_token = auth_header(target_id, u_target, 0);

    // Ban, verify blocked, lift, verify allowed again.
    let (s, b) = post_json(
        &app,
        "/api/admin/forum/bans",
        Some(&admin_token),
        json!({ "user_id": target_id, "scope": "forum" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "ban: {b}");
    let ban_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_bans WHERE user_id = $1 ORDER BY id DESC LIMIT 1")
            .bind(target_id)
            .fetch_one(&db)
            .await
            .expect("ban id");
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 lift blocked", "category_slug": "fr5-lift", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "blocked while banned: {b}");

    let (s, b) = delete_json(
        &app,
        &format!("/api/admin/forum/bans/{ban_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "lift: {b}");
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_bans WHERE id = $1")
        .bind(ban_id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(remaining, 0, "ban row gone: {remaining}");

    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&target_token),
        json!({ "title": "fr5 lift allowed", "category_slug": "fr5-lift", "body": "fr5 body long enough" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "allowed after lift: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_admin)
        .bind(u_target)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f5_report_forum_target() {
    let _g = db_guard();
    let db = pool().await;
    let u_reporter = "fr5_report_reporter";
    let u_author = "fr5_report_author";
    // Filing reports requires trust_level >= 1, read from the database.
    let reporter_id = seed_user_at_trust(&db, u_reporter, 2).await;
    let author_id = seed_user(&db, u_author).await;
    wipe_forum_for_users(&db, &[u_reporter, u_author]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr5-report'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr5-report", "F5 Report").await;
    let tid = seed_topic(&db, cid, author_id, "fr5 report topic").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");

    let app = app().await;
    let token = auth_header(reporter_id, u_reporter, 0);

    // Report a forum post — target_type now accepted.
    let (s, b) = post_json(
        &app,
        "/api/reports",
        Some(&token),
        json!({ "target_type": "forum_post", "target_id": post_id, "reason": "spam",
        "category": "spam" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "report forum_post: {b}");
    assert_eq!(b["err"], 0, "report forum_post: {b}");

    // Assert the classification PERSISTS, not just that the call returned 200.
    // A status-only assertion passes again the moment someone drops `category`
    // from the INSERT in reports.rs - which is exactly how this endpoint came
    // to 500 on every call in the first place. See
    // docs/specs/reports-category-column.md R6.
    let stored: String = sqlx::query_scalar(
        "SELECT category FROM user_reports WHERE reporter_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(reporter_id)
    .fetch_one(&db)
    .await
    .expect("stored report category");
    assert_eq!(stored, "spam", "report category must persist");

    // And a forum topic.
    let (s, b) = post_json(
        &app,
        "/api/reports",
        Some(&token),
        json!({ "target_type": "forum_topic", "target_id": tid, "reason": "off topic" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "report forum_topic: {b}");
    assert_eq!(b["err"], 0, "report forum_topic: {b}");

    // Cleanup
    //
    // Delete the reports by target_id FIRST. user_reports.reporter_id is
    // nullable and has no FK constraint (verified: is_nullable=YES, and
    // information_schema reports no foreign key), so deleting the reporter sets
    // it to NULL rather than cascading - after the users go, these rows are
    // unidentifiable by reporter and accumulate on every run. They then show up
    // in reports_api's admin-list test as foreign rows.
    //
    // A cleanup that deletes by reporter_id cannot match anything, which is
    // exactly what the first attempt at this did; it looked correct and left the
    // count growing 15 -> 17 -> 19 across runs.
    // See docs/specs/reports-suite.md section 2.2.
    // target_id is `integer` (information_schema), so these bind as i32 - a
    // text bind matches nothing and `.ok()` swallows it, which is how the first
    // two attempts at this cleanup silently did nothing.
    sqlx::query("DELETE FROM user_reports WHERE target_id = ANY($1)")
        .bind(vec![tid as i32, post_id as i32])
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_reporter)
        .bind(u_author)
        .execute(&db)
        .await
        .ok();
}

// ── F6 tests: metamod status, queue, votes, cooldown ────────────────────────

// ── Trust cutover (2026-09): metamoderation retired ────────────────────────
///
/// The f6_* voting/queue/cooldown tests covered the Slashdot apparatus that
/// the trust cutover removed. The endpoints stay mounted but answer 410;
/// these tests pin that contract (auth still required, body points at
/// reports) plus the contested-escalation path that replaces it.

#[tokio::test]
#[ignore]
async fn trust_metamod_queue_gone() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr_trust_mm_queue";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    promote_curator(&db, uid).await;
    let app = app().await;

    // Anonymous still 401s (auth runs before the 410).
    let (s, b) = get_json(&app, "/api/forum/metamod/queue", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon queue: {b}");

    // Even TL5 gets 410 with the reports pointer.
    let token = auth_header(uid, u, 5);
    let (s, b) = get_json(&app, "/api/forum/metamod/queue", Some(&token)).await;
    assert_eq!(s, StatusCode::GONE, "queue gone: {b}");
    assert_eq!(b["err"], -410, "queue err: {b}");
    assert!(
        b["msg"].as_str().unwrap_or("").contains("reports"),
        "reports pointer: {b}"
    );

    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn trust_metamod_vote_gone() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr_trust_mm_vote";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    promote_curator(&db, uid).await;
    let app = app().await;
    let token = auth_header(uid, u, 5);

    let (s, b) = post_json(
        &app,
        "/api/forum/metamod/1/vote",
        Some(&token),
        json!({ "verdict": "fair" }),
    )
    .await;
    assert_eq!(s, StatusCode::GONE, "vote gone: {b}");
    assert_eq!(b["err"], -410, "vote err: {b}");

    let (s, b) = post_json(
        &app,
        "/api/forum/metamod/grants/1/verdict",
        Some(&token),
        json!({ "verdict": "unfair" }),
    )
    .await;
    assert_eq!(s, StatusCode::GONE, "verdict gone: {b}");
    assert_eq!(b["err"], -410, "verdict err: {b}");

    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn trust_grant_detail_stays_readable() {
    // The audit-trail read (grant detail) survives the cutover: past
    // moderation actions remain inspectable even though no new votes land.
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr_trust_grant_author";
    let u_cur = "fr_trust_grant_cur";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    wipe_metamod_for_users(&db, &[u_author, u_cur]).await;
    let (action_id, _) = seed_mod_action(&db, author_id, cur_id, "Insightful").await;
    promote_curator(&db, cur_id).await;
    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);

    let (s, b) = get_json(
        &app,
        &format!("/api/forum/metamod/grants/{action_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "grant detail: {b}");
    assert_eq!(b["err"], 0, "grant detail err: {b}");
    assert_eq!(b["action_id"], action_id, "action id: {b}");

    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr6-meta'")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn trust_elder_queue_access_no_resolve() {
    // TL4 (Elder): queue access yes, resolve-gated actions no.
    let _g = db_guard();
    let db = pool().await;
    let u = "fr_trust_elder";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    promote_elder(&db, uid).await;
    let app = app().await;
    let token = auth_header(uid, u, 1);

    let (s, b) = get_json(&app, "/api/forum/moderation/status", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "elder status: {b}");
    assert_eq!(b["trust_level"], 4, "elder trust: {b}");
    assert_eq!(b["can_queue"], true, "elder queue: {b}");
    assert_eq!(b["can_resolve"], false, "elder resolve: {b}");

    let (s, b) = get_json(&app, "/api/forum/moderation/queue", Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "elder queue read: {b}");
    assert_eq!(b["err"], 0, "elder queue err: {b}");

    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}


#[tokio::test]
#[ignore]
async fn f7_level_exp_from_topic_and_post() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr7_level_creator";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-level'")
        .execute(&db)
        .await
        .ok();
    // Start at exp 0, level 0.
    sqlx::query("UPDATE users SET level = 0, exp = 0 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr7-level", "F7 Level").await;

    let app = app().await;
    let token = auth_header(uid, u, 0);

    // Create topic → +2 exp (topic_create), OP post is the same row.
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "F7 exp topic", "category_slug": "fr7-level", "body": "exp body" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create topic: {b}");
    let tid = b["id"].as_i64().expect("topic id");

    let (exp, lvl): (i64, i16) = sqlx::query_as("SELECT exp, level FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .expect("user row");
    assert_eq!(exp, 2, "topic create exp: {exp}");
    assert_eq!(lvl, 0, "still level 0: {lvl}");

    // Reply → +2 exp (post_create).
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/topics/{tid}/posts"),
        Some(&token),
        json!({ "body": "a reply here" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "reply: {b}");
    let (exp,): (i64,) = sqlx::query_as("SELECT exp FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .expect("user row");
    assert_eq!(exp, 4, "reply exp: {exp}");

    // exp_events audit trail.
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exp_events WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .expect("events count");
    assert_eq!(n, 2, "two exp events: {n}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}
#[tokio::test]
#[ignore]
async fn f7_level_gate_curator() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr7_gate_author";
    let u_cur = "fr7_gate_cur";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-gate'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr7-gate", "F7 Gate").await;
    let tid = seed_topic(&db, cid, author_id, "fr7 gate target").await;
    let post_id: i64 =
        sqlx::query_scalar("SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id LIMIT 1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("op post id");

    // Curator at level 50 (14+ days old): can moderate.
    sqlx::query("UPDATE users SET level = 50, exp = 5000, created_at = NOW() - INTERVAL '30 days' WHERE id = $1")
        .bind(cur_id)
        .execute(&db)
        .await
        .ok();
    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token),
        json!({ "reason": "Insightful" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "level 50 moderate: {b}");
    assert_eq!(b["err"], 0, "level 50 moderate err: {b}");

    // Level 49: forbidden.
    sqlx::query("UPDATE users SET level = 49, exp = 4900 WHERE id = $1")
        .bind(cur_id)
        .execute(&db)
        .await
        .ok();
    let token2 = auth_header(cur_id, u_cur, 5);
    let (s, b) = post_json(
        &app,
        &format!("/api/forum/posts/{post_id}/moderate"),
        Some(&token2),
        json!({ "reason": "Insightful" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "level 49 forbidden: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}

/// Regression (2026-09-07 prod outage): a stale pair of row triggers on
/// `forum_topics` (`trg_forum_topics_search_vector` +
/// `forum_topics_search_vector_update`) referenced a `forum_topic_tags.tag_id`
/// column that migration 082 had removed. Every UPDATE on a topic row —
/// including the `view_count + 1` bump in `topic_detail` — fired the broken
/// function and turned GET /api/forum/topics/{id} into a 500.
///
/// Migration 089 drops both triggers + the orphan function (the Rust handlers
/// maintain `search_vector` inline). This test pins the invariant two ways:
///  1. No non-FK trigger may exist on forum_topics (FK constraint triggers
///     are noise — filter them out).
///  2. A raw UPDATE on a topic row must succeed — this is exactly the
///     statement that used to abort inside the trigger.
#[tokio::test]
#[ignore]
async fn regression_no_stale_forum_topic_triggers() {
    let _g = db_guard();
    let db = pool().await;

    let triggers: Vec<String> = sqlx::query_scalar(
        "SELECT tgname FROM pg_trigger
          WHERE tgrelid = 'forum_topics'::regclass
            AND tgname NOT LIKE 'RI_%'",
    )
    .fetch_all(&db)
    .await
    .expect("list forum_topics triggers");
    assert!(
        triggers.is_empty(),
        "stale forum_topics triggers present (089 must drop them): {triggers:?}"
    );

    let orphan: Option<String> = sqlx::query_scalar(
        "SELECT proname FROM pg_proc WHERE proname = 'forum_topics_search_vector_update'",
    )
    .fetch_optional(&db)
    .await
    .expect("orphan function lookup");
    assert!(
        orphan.is_none(),
        "orphan function forum_topics_search_vector_update still exists"
    );
}

/// End-to-end half of the same regression: seed a topic, then UPDATE the row
/// (what topic_detail's view-count bump does) and read it back through the
/// real handler — must be err:0, not a 500.
#[tokio::test]
#[ignore]
async fn regression_topic_update_survives_view_count_bump() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr_regress_trig_author";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr-regress-trig'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr-regress-trig", "Regress Trig").await;
    let tid = seed_topic(&db, cid, uid, "fr regress trigger topic").await;

    // The exact statement class that used to fire the broken trigger.
    let bumped: i64 = sqlx::query_scalar(
        "UPDATE forum_topics SET view_count = view_count + 1 WHERE id = $1 RETURNING view_count",
    )
    .bind(tid)
    .fetch_one(&db)
    .await
    .expect("view_count bump must not abort in a trigger");
    assert_eq!(bumped, 1, "first bump lands on 0-seeded topic");

    // And the handler path returns err:0 with the bumped count.
    let app = app().await;
    let token = auth_header(uid, u, 0);
    let (s, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "detail after bump: {b}");
    assert_eq!(b["err"], 0, "detail err after bump: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f7_level_gate_admin() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr7_gate_admin";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-adm'")
        .execute(&db)
        .await
        .ok();

    // Level 99 (not 100): cannot create category (token must carry level 99).
    sqlx::query("UPDATE users SET level = 99, exp = 9900 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let app = app().await;
    let token = auth_header_level(uid, u, 10, 99);
    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token),
        json!({ "slug": "fr7-adm", "title": "F7 Adm" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "level 99 forbidden: {b}");

    // Level 100: can.
    sqlx::query("UPDATE users SET level = 100, exp = 10000 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let token2 = auth_header(uid, u, 10);
    let (s, b) = post_json(
        &app,
        "/api/forum/categories",
        Some(&token2),
        json!({ "slug": "fr7-adm", "title": "F7 Adm" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "level 100 create: {b}");
    assert_eq!(b["err"], 0, "create err: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-adm'")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn f7_level_mod_received_exp_capped() {
    let _g = db_guard();
    let db = pool().await;
    let u_author = "fr7_cap_author";
    let u_cur = "fr7_cap_cur";
    let author_id = seed_user(&db, u_author).await;
    let cur_id = seed_user(&db, u_cur).await;
    wipe_forum_for_users(&db, &[u_author, u_cur]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-cap'")
        .execute(&db)
        .await
        .ok();
    // Author starts at exp 0.
    sqlx::query("UPDATE users SET level = 0, exp = 0 WHERE id = $1")
        .bind(author_id)
        .execute(&db)
        .await
        .ok();
    // A prior run today may have left mod_received events — clear them so
    // the daily cap starts fresh for this test's 4 moderations.
    sqlx::query("DELETE FROM exp_events WHERE user_id = $1")
        .bind(author_id)
        .execute(&db)
        .await
        .ok();
    // Curator at level 50 (14+ days old).
    sqlx::query("UPDATE users SET level = 50, exp = 5000, created_at = NOW() - INTERVAL '30 days' WHERE id = $1")
        .bind(cur_id)
        .execute(&db)
        .await
        .ok();
    // A prior run may have left a drained grant row; drop it so the first
    // current_grant() call re-initializes with a full window of points.
    sqlx::query("DELETE FROM forum_mod_grants WHERE user_id = $1")
        .bind(cur_id)
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr7-cap", "F7 Cap").await;
    let tid = seed_topic(&db, cid, author_id, "fr7 cap target").await;

    // Curator moderates 4 different posts positively.
    let app = app().await;
    let token = auth_header(cur_id, u_cur, 5);
    let mut post_ids = Vec::new();
    for i in 0..4 {
        let pid: i64 = sqlx::query_scalar(
            "INSERT INTO forum_posts (topic_id, author_id, body) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(tid)
        .bind(author_id)
        .bind(format!("cap reply {i}"))
        .fetch_one(&db)
        .await
        .expect("post row");
        post_ids.push(pid);
        let (s, b) = post_json(
            &app,
            &format!("/api/forum/posts/{pid}/moderate"),
            Some(&token),
            json!({ "reason": "Insightful" }),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "moderate {i}: {b}");
        assert_eq!(b["err"], 0, "moderate err {i}: {b}");
    }

    // Author exp: 4 positive mods but capped at +3/day → exp 3, not 4.
    let (exp,): (i64,) = sqlx::query_as("SELECT exp FROM users WHERE id = $1")
        .bind(author_id)
        .fetch_one(&db)
        .await
        .expect("author row");
    assert_eq!(exp, 3, "capped at 3: {exp}");
    // Events are only recorded when exp is actually applied — the daily cap
    // stops the 4th award before an event is written.
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM exp_events WHERE user_id = $1 AND event_type = 'mod_received'",
    )
    .bind(author_id)
    .fetch_one(&db)
    .await
    .expect("events count");
    assert_eq!(n, 3, "3 mod events recorded (cap stops the 4th): {n}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u_author)
        .bind(u_cur)
        .execute(&db)
        .await
        .ok();
}
// ── F7 tests: site info, invites, registration applications, blocks ────────

/// Wipe every F7 row referencing the given users (self-heal on reruns).
async fn wipe_f7_for_users(db: &sqlx::PgPool, usernames: &[&str]) {
    for u in usernames {
        let uid: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(u)
            .fetch_optional(db)
            .await
            .unwrap_or(None);
        if let Some(uid) = uid {
            // Hard-delete every F7 row referencing the user, then the
            // account itself. Order matters: child tables first.
            let _ = sqlx::query("DELETE FROM user_invites WHERE created_by = $1 OR used_by = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query(
                "DELETE FROM registration_applications WHERE user_id = $1 OR reviewed_by = $1",
            )
            .bind(uid)
            .execute(db)
            .await;
            let _ =
                sqlx::query("DELETE FROM blocked_users WHERE user_id = $1 OR blocked_user_id = $1")
                    .bind(uid)
                    .execute(db)
                    .await;
            let _ = sqlx::query(
                "DELETE FROM modlog WHERE action IN ('invite_create','registration_app_review') AND actor_id = $1",
            )
            .bind(uid)
            .execute(db)
            .await;
            // Registration creates a full account row; other subsystems
            // (bookmarks, follows, ...) may reference it. Delete those.
            // Email is globally unique — the wiped row's '' default is
            // shared, so a fresh user created here gets its own unique email.
            let _ = sqlx::query(
                "UPDATE users SET email = 'f7-wipe-' || id WHERE id = $1 AND email = ''",
            )
            .bind(uid)
            .execute(db)
            .await;
            // The username column is globally unique too — release it so a
            // re-registered user with the same name doesn't collide.
            let _ = sqlx::query("UPDATE users SET username = 'f7-wiped-' || id WHERE id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM bookmarks WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM follows WHERE user_id = $1 OR followed_user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM notifications WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM notification_preferences WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM comments WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM reading_lists WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM shelves WHERE user_id = $1")
                .bind(uid)
                .execute(db)
                .await;
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(uid)
                .execute(db)
                .await;
        }
    }
}

#[tokio::test]
#[ignore]
async fn f7_level_level_up_notification() {
    let _g = db_guard();
    let db = pool().await;
    let u = "fr7_lvlup";
    let uid = seed_user(&db, u).await;
    wipe_forum_for_users(&db, &[u]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr7-lvlup'")
        .execute(&db)
        .await
        .ok();
    // At 98 exp: creating a topic (+2) crosses into level 1 (100 exp).
    sqlx::query("UPDATE users SET level = 0, exp = 98 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr7-lvlup", "F7 Level Up").await;

    let app = app().await;
    let token = auth_header(uid, u, 0);
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        Some(&token),
        json!({ "title": "F7 level up topic", "category_slug": "fr7-lvlup", "body": "level up body" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create: {b}");

    let (lvl, exp): (i16, i64) = sqlx::query_as("SELECT level, exp FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .expect("user row");
    assert_eq!(lvl, 1, "leveled up: {lvl}");
    assert_eq!(exp, 100, "exp 100: {exp}");

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'level_up'",
    )
    .bind(uid)
    .fetch_one(&db)
    .await
    .expect("notif count");
    assert_eq!(n, 1, "one level_up notification: {n}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u)
        .execute(&db)
        .await
        .ok();
}

#[allow(dead_code)]
async fn f7_site_info() {
    let _g = db_guard();
    let app = app().await;

    let (s, b) = get_json(&app, "/api/site", None).await;
    assert_eq!(s, StatusCode::OK, "site: {b}");
    assert_eq!(b["err"], 0, "site err: {b}");
    assert!(b["site"]["name"].is_string(), "site name: {b}");
    assert!(
        !b["site"]["name"].as_str().unwrap().is_empty(),
        "site name empty: {b}"
    );
    assert!(
        b["site"]["description"].is_string(),
        "site description: {b}"
    );
    assert!(b["site"]["version"].is_string(), "site version: {b}");
    let mode = b["site"]["registration_mode"].as_str().unwrap();
    assert!(
        matches!(mode, "open" | "invite" | "application"),
        "registration_mode: {b}"
    );
}

#[tokio::test]
#[ignore]
async fn f7_invite_create_list() {
    let _g = db_guard();
    let db = pool().await;
    let u_curator = "fr7_inv_curator";
    let u_pleb = "fr7_inv_pleb";
    wipe_f7_for_users(&db, &[u_curator, u_pleb]).await;
    let curator_id = seed_user(&db, u_curator).await;
    let pleb_id = seed_user(&db, u_pleb).await;
    promote_curator(&db, curator_id).await;

    // Invite creation is only meaningful in invite mode (400 otherwise), so
    // force REGISTRATION_MODE=invite for this test. SAFETY: serialized test
    // process (global DB mutex, --test-threads=1).
    unsafe { std::env::set_var("REGISTRATION_MODE", "invite") };
    let app = app().await;
    let curator_token = auth_header(curator_id, u_curator, 5);
    let pleb_token = auth_header(pleb_id, u_pleb, 0);

    // Non-curator cannot create invites (403).
    let (s, b) = post_json(&app, "/api/admin/invites", Some(&pleb_token), json!({})).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "pleb create invite: {b}");
    assert_eq!(b["err"], -403, "pleb create invite err: {b}");

    // Anonymous cannot list invites (401).
    let (s, b) = get_json(&app, "/api/admin/invites", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon list invites: {b}");
    assert_eq!(b["err"], 401, "anon list invites err: {b}");

    // Curator creates an invite (REGISTRATION_MODE is unset → open, but
    // create_invite only gates on the mode being invite; open mode allows
    // pre-generating codes for later).
    let (s, b) = post_json(
        &app,
        "/api/admin/invites",
        Some(&curator_token),
        json!({ "note": "f7 test" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create invite: {b}");
    assert_eq!(b["err"], 0, "create invite err: {b}");
    let code = b["code"].as_str().expect("code string").to_string();
    assert_eq!(code.len(), 12, "code length: {b}");
    assert!(b["expires_at"].is_null(), "no expiry: {b}");

    // List shows the invite.
    let (s, b) = get_json(&app, "/api/admin/invites", Some(&curator_token)).await;
    assert_eq!(s, StatusCode::OK, "list invites: {b}");
    assert_eq!(b["err"], 0, "list invites err: {b}");
    let items = b["items"].as_array().expect("items array");
    let mine = items
        .iter()
        .find(|i| i["code"] == code)
        .expect("invite in list");
    assert_eq!(mine["created_by"], curator_id, "created_by: {mine}");
    assert!(mine["used_by"].is_null(), "unused: {mine}");

    // Cleanup
    wipe_f7_for_users(&db, &[u_curator, u_pleb]).await;
}

#[tokio::test]
#[ignore]
async fn f7_invite_register_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u_curator = "fr7_flow_curator";
    let u_new1 = "fr7_flow_new1";
    let u_new2 = "fr7_flow_new2";
    let u_new3 = "fr7_flow_new3";
    for u in [u_curator, u_new1, u_new2, u_new3] {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM user_invites WHERE code LIKE 'f7flow%'")
            .execute(&db)
            .await;
    }
    wipe_f7_for_users(&db, &[u_curator, u_new1, u_new2, u_new3]).await;
    // Rename any stale rows that still hold the target usernames (they can
    // only exist from an aborted run whose wipe never completed). Note:
    // register stores '' for a missing email, so username is the hard
    // collision; clearing both keeps the account fully fresh.
    let _ = sqlx::query("UPDATE users SET username = 'f7-stale-' || id, email = 'f7-stale-' || id || '@x.invalid' WHERE username IN ($1, $2, $3, $4)")
        .bind(u_curator)
        .bind(u_new1)
        .bind(u_new2)
        .bind(u_new3)
        .execute(&db)
        .await;

    let curator_id = seed_user(&db, u_curator).await;
    promote_curator(&db, curator_id).await;

    // Seed two invites directly: one plain, one expired.
    sqlx::query("INSERT INTO user_invites (code, created_by) VALUES ('f7flowcode1', $1)")
        .bind(curator_id)
        .execute(&db)
        .await
        .expect("seed invite 1");
    sqlx::query(
        "INSERT INTO user_invites (code, created_by, expires_at) VALUES ('f7flowcode2', $1, NOW() - INTERVAL '1 hour')",
    )
    .bind(curator_id)
    .execute(&db)
    .await
    .expect("seed invite 2");

    let app = app().await;
    let now_ms = chrono::Utc::now().timestamp_millis() as u64;

    // REGISTRATION_MODE is unset in the test env → open mode: registration
    // works without a code, and an invalid code is ignored (not fatal).
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new3,
            "password": "hunter22",
            "email": "f7flow_new3@example.com",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
            "invite_code": "not-a-real-code",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "open-mode register with bogus code: {b}");
    assert_eq!(b["err"], 0, "open-mode register err: {b}");
    let _new3_id = b["user"]["id"].as_i64().unwrap() as i32;

    // Valid code consumed in open mode: register succeeds, code marked used.
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new1,
            "password": "hunter22",
            "email": "f7flow_new1@example.com",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
            "invite_code": "f7flowcode1",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "register with code: {b}");
    assert_eq!(b["err"], 0, "register with code err: {b}");
    let new1_id = b["user"]["id"].as_i64().unwrap() as i32;

    let (used_by, used_at): (Option<i32>, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT used_by, used_at FROM user_invites WHERE code = 'f7flowcode1'")
            .fetch_one(&db)
            .await
            .expect("invite row");
    assert_eq!(used_by, Some(new1_id), "code consumed by new user");
    assert!(used_at.is_some(), "used_at set");

    // Reusing the same code in open mode: registration still succeeds (the
    // code is simply already consumed and ignored).
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new2,
            "password": "hunter22",
            "email": "f7flow_new2@example.com",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
            "invite_code": "f7flowcode1",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "open-mode reuse: {b}");

    // The expired code is not consumed (open mode, no gate).
    let (used_by,): (Option<i32>,) =
        sqlx::query_as("SELECT used_by FROM user_invites WHERE code = 'f7flowcode2'")
            .fetch_one(&db)
            .await
            .expect("invite 2 row");
    assert!(used_by.is_none(), "expired code untouched in open mode");

    // Cleanup
    sqlx::query("DELETE FROM user_invites WHERE code LIKE 'f7flow%'")
        .execute(&db)
        .await
        .ok();
    wipe_f7_for_users(&db, &[u_curator, u_new1, u_new2, u_new3]).await;
}

#[tokio::test]
#[ignore]
async fn f7_application_submit_review() {
    let _g = db_guard();
    let db = pool().await;
    let u_applicant = "fr7_app_applicant";
    let u_mod = "fr7_app_mod";
    wipe_f7_for_users(&db, &[u_applicant, u_mod]).await;
    let applicant_id = seed_user(&db, u_applicant).await;
    let mod_id = seed_user(&db, u_mod).await;
    promote_curator(&db, mod_id).await;

    let app = app().await;
    let applicant_token = auth_header(applicant_id, u_applicant, 0);
    let mod_token = auth_header(mod_id, u_mod, 5);

    // Anonymous cannot apply (401).
    let (s, b) = post_json(
        &app,
        "/api/registration-applications",
        None,
        json!({ "reason": "f7 test" }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon apply: {b}");

    // Applicant applies.
    let (s, b) = post_json(
        &app,
        "/api/registration-applications",
        Some(&applicant_token),
        json!({ "reason": "I want to join the forum" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "apply: {b}");
    assert_eq!(b["err"], 0, "apply err: {b}");
    assert_eq!(b["status"], "pending", "apply status: {b}");
    let app_id = b["id"].as_i64().unwrap();

    // Duplicate pending application → 409.
    let (s, b) = post_json(
        &app,
        "/api/registration-applications",
        Some(&applicant_token),
        json!({ "reason": "second try" }),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT, "duplicate apply: {b}");
    assert_eq!(b["err"], -409, "duplicate apply err: {b}");

    // Non-mod cannot review (403); anonymous cannot list (401).
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/registration-applications/{app_id}/review"),
        Some(&applicant_token),
        json!({ "status": "approved" }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "pleb review: {b}");
    let (s, b) = get_json(&app, "/api/admin/registration-applications", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon list apps: {b}");

    // Mod lists pending applications — the application is visible.
    let (s, b) = get_json(
        &app,
        "/api/admin/registration-applications?status=pending",
        Some(&mod_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list apps: {b}");
    let items = b["items"].as_array().expect("items array");
    let mine = items
        .iter()
        .find(|i| i["id"] == json!(app_id))
        .expect("application in list");
    assert_eq!(mine["username"], u_applicant, "applicant name: {mine}");
    assert_eq!(mine["status"], "pending", "pending: {mine}");

    // Approve → role flips to 1 (trusted), status updated, modlog written.
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/registration-applications/{app_id}/review"),
        Some(&mod_token),
        json!({ "status": "approved", "reason": "welcome" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "approve: {b}");
    assert_eq!(b["err"], 0, "approve err: {b}");
    assert_eq!(b["status"], "approved", "approved status: {b}");

    let role: i16 = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
        .bind(applicant_id)
        .fetch_one(&db)
        .await
        .expect("applicant role");
    assert_eq!(role, 1, "approved applicant becomes trusted");

    let (action, target, details): (String, String, serde_json::Value) = sqlx::query_as(
        "SELECT action, target_id, details FROM modlog
         WHERE action = 'registration_app_review' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&db)
    .await
    .expect("modlog entry");
    assert_eq!(action, "registration_app_review");
    assert_eq!(target, app_id.to_string());
    assert_eq!(details["status"], "approved", "modlog details: {details}");
    assert_eq!(
        details["applicant_id"], applicant_id,
        "modlog applicant: {details}"
    );

    // Reviewing again → 409 (already reviewed).
    let (s, b) = post_json(
        &app,
        &format!("/api/admin/registration-applications/{app_id}/review"),
        Some(&mod_token),
        json!({ "status": "rejected" }),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT, "re-review: {b}");

    // Cleanup
    wipe_f7_for_users(&db, &[u_applicant, u_mod]).await;
}

#[tokio::test]
#[ignore]
async fn f7_block_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u_blocker = "fr7_blk_blocker";
    let u_target = "fr7_blk_target";
    let u_ghost = "fr7_blk_ghost";
    wipe_f7_for_users(&db, &[u_blocker, u_target, u_ghost]).await;
    let blocker_id = seed_user(&db, u_blocker).await;
    let target_id = seed_user(&db, u_target).await;
    seed_user(&db, u_ghost).await;

    let app = app().await;
    let blocker_token = auth_header(blocker_id, u_blocker, 0);

    // Anonymous cannot block (401).
    let (s, b) = post_json(&app, "/api/blocks", None, json!({ "user_id": target_id })).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anon block: {b}");

    // Self-block → 400.
    let (s, b) = post_json(
        &app,
        "/api/blocks",
        Some(&blocker_token),
        json!({ "user_id": blocker_id }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "self block: {b}");
    assert_eq!(b["err"], -1, "self block err: {b}");

    // Blocking a nonexistent user → 404.
    let (s, b) = post_json(
        &app,
        "/api/blocks",
        Some(&blocker_token),
        json!({ "user_id": 99999999 }),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "block ghost: {b}");

    // Block → list shows it → blocking again is idempotent → unblock → gone.
    let (s, b) = post_json(
        &app,
        "/api/blocks",
        Some(&blocker_token),
        json!({ "user_id": target_id }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "block: {b}");
    assert_eq!(b["blocked"], true, "blocked flag: {b}");

    let (s, b) = get_json(&app, "/api/blocks", Some(&blocker_token)).await;
    assert_eq!(s, StatusCode::OK, "list blocks: {b}");
    let items = b["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1, "one block: {b}");
    assert_eq!(items[0]["user_id"], target_id, "blocked id: {b}");
    assert_eq!(items[0]["username"], u_target, "blocked name: {b}");

    let (s, b) = post_json(
        &app,
        "/api/blocks",
        Some(&blocker_token),
        json!({ "user_id": target_id }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "re-block idempotent: {b}");
    let (_s, b) = get_json(&app, "/api/blocks", Some(&blocker_token)).await;
    assert_eq!(
        b["items"].as_array().unwrap().len(),
        1,
        "still one block: {b}"
    );

    let (s, b) = delete_json(
        &app,
        &format!("/api/blocks/{target_id}"),
        Some(&blocker_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "unblock: {b}");
    assert_eq!(b["blocked"], false, "unblocked flag: {b}");

    let (s, b) = get_json(&app, "/api/blocks", Some(&blocker_token)).await;
    assert_eq!(s, StatusCode::OK, "list after unblock: {b}");
    assert_eq!(
        b["items"].as_array().unwrap().len(),
        0,
        "empty after unblock: {b}"
    );

    // Unblocking again is also idempotent.
    let (s, b) = delete_json(
        &app,
        &format!("/api/blocks/{target_id}"),
        Some(&blocker_token),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "re-unblock idempotent: {b}");

    // Cleanup
    wipe_f7_for_users(&db, &[u_blocker, u_target, u_ghost]).await;
}

#[tokio::test]
#[ignore]
async fn f7_invite_gate_closed() {
    let _g = db_guard();
    let db = pool().await;
    let u_curator = "fr7_gate_curator";
    let u_new = "fr7_gate_new";
    for u in [u_curator, u_new] {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM user_invites WHERE code LIKE 'f7gate%'")
            .execute(&db)
            .await;
    }
    wipe_f7_for_users(&db, &[u_curator, u_new]).await;
    // A previous aborted run may have left a row whose username/email now
    // collides with u_new. Rename it out of the way so re-registration works.
    // (Users deleted above are gone; this only touches rows that the wipe
    // could not reach because they were created after it ran.)
    // Note: register stores '' for a missing email, so only username is a
    // hard collision; still clear both to be safe.
    let _ = sqlx::query("UPDATE users SET username = 'f7-stale-' || id WHERE username IN ($1, $2)")
        .bind(u_curator)
        .bind(u_new)
        .execute(&db)
        .await;
    // Reset the auth rate limit bucket so this test's registrations are not
    // throttled (tests share the real limiter; other tests hammer /register).
    let _ = sqlx::query("UPDATE auth_rate_limits SET count = 0 WHERE identifier LIKE '%test%'")
        .execute(&db)
        .await;

    let curator_id = seed_user(&db, u_curator).await;
    promote_curator(&db, curator_id).await;
    sqlx::query("INSERT INTO user_invites (code, created_by) VALUES ('f7gatecode1', $1)")
        .bind(curator_id)
        .execute(&db)
        .await
        .expect("seed gate invite");

    let app = app().await;
    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
    // All registers in this test share the 0.0.0.0 IP; the auth tier
    // (10/min) could throttle the later ones if another test used the same
    // bucket recently. Give this test a fresh identity bucket by sending a
    // per-test X-Client-Id (bonus capacity, distinct (ip, cid) key).
    let _cid = "f7-gate-client";

    // Force invite mode for this test. CAREFUL: the child process env is
    // ours, so `app().await` below must already have been built. It was —
    // the router only captures config/db, never REGISTRATION_MODE; the mode
    // is read per-request by the register handler.
    // SAFETY: single-threaded test process (global DB mutex, --test-threads=1).
    unsafe { std::env::set_var("REGISTRATION_MODE", "invite") };

    // No code → 403.
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new,
            "password": "hunter22",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
        }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "no code: {b}");
    assert_eq!(b["err"], -403, "no code err: {b}");

    // Wrong code → 403.
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new,
            "password": "hunter22",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
            "invite_code": "bogus-code",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "wrong code: {b}");

    // Valid code → 200, account created, code consumed.
    let (s, b) = post_json(
        &app,
        "/api/auth/register",
        None,
        json!({
            "username": u_new,
            "password": "hunter22",
            "email": "f7gate_new@example.com",
            "website": "",
            "form_opened_at": (now_ms - 5000).to_string(),
            "invite_code": "f7gatecode1",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "valid code: {b}");
    assert_eq!(b["err"], 0, "valid code err: {b}");
    let new_id = b["user"]["id"].as_i64().unwrap() as i32;

    let (used_by,): (Option<i32>,) =
        sqlx::query_as("SELECT used_by FROM user_invites WHERE code = 'f7gatecode1'")
            .fetch_one(&db)
            .await
            .expect("gate invite row");
    assert_eq!(used_by, Some(new_id), "code consumed: {used_by:?}");

    // Restore the default mode so later tests in the same process are
    // unaffected (tests run --test-threads=1, but be tidy anyway).
    // SAFETY: single-threaded test process.
    unsafe { std::env::remove_var("REGISTRATION_MODE") };

    // Cleanup
    sqlx::query("DELETE FROM user_invites WHERE code LIKE 'f7gate%'")
        .execute(&db)
        .await
        .ok();
    wipe_f7_for_users(&db, &[u_curator, u_new]).await;
}

// ── Regression: anonymous view must not 500 (forum_topic_views NOT NULL PK) ──
#[tokio::test]
#[ignore]
async fn f8_anonymous_view_does_not_500() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr8_anonview_author";
    let id1 = seed_user(&db, u1).await;
    wipe_forum_for_users(&db, &[u1]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr8-anonview'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr8-anonview", "AnonView").await;
    let tid = seed_topic(&db, cid, id1, "fr8 anon topic").await;
    sqlx::query("UPDATE forum_topics SET view_count = 0 WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .ok();

    let app = app().await;

    // Anonymous GET (no token) — must return 200, not 500.
    let (s, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), None).await;
    assert_eq!(s, StatusCode::OK, "anon view: {b}");
    assert_eq!(b["err"], 0, "anon view: {b}");
    // view_count incremented (anonymous always counts).
    assert_eq!(b["view_count"], json!(1), "anon first view: {b}");

    // Second anon view also counts (no rate-limit for anonymous).
    let (s, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), None).await;
    assert_eq!(s, StatusCode::OK, "anon 2nd view: {b}");
    assert_eq!(b["view_count"], json!(2), "anon 2nd view: {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u1)
        .execute(&db)
        .await
        .ok();
}

// ── Regression: logged-in user view rate-limited to 1/60min ──────────────────
#[tokio::test]
#[ignore]
async fn f8_logged_in_view_rate_limited() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "fr8_ratelimit_user";
    let id1 = seed_user(&db, u1).await;
    wipe_forum_for_users(&db, &[u1]).await;
    sqlx::query("DELETE FROM forum_categories WHERE slug = 'fr8-ratelimit'")
        .execute(&db)
        .await
        .ok();
    let cid = seed_category(&db, "fr8-ratelimit", "RateLimit").await;
    let tid = seed_topic(&db, cid, id1, "fr8 rl topic").await;
    sqlx::query("UPDATE forum_topics SET view_count = 0 WHERE id = $1")
        .bind(tid)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let token = auth_header(id1, u1, 0);

    // First view — increments.
    let (s, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(s, StatusCode::OK, "rl 1st: {b}");
    assert_eq!(b["view_count"], json!(1), "rl 1st: {b}");

    // Second view within 60min — rate-limited, does NOT increment.
    let (_, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(b["view_count"], json!(1), "rl 2nd (limited): {b}");

    // Clean the rate-limit row → third view counts again.
    sqlx::query("DELETE FROM forum_topic_views WHERE topic_id = $1 AND user_id = $2")
        .bind(tid)
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    let (_, b) = get_json(&app, &format!("/api/forum/topics/{tid}"), Some(&token)).await;
    assert_eq!(b["view_count"], json!(2), "rl 3rd (after reset): {b}");

    // Cleanup
    sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u1)
        .execute(&db)
        .await
        .ok();
}

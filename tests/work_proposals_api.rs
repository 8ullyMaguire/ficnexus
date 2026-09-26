//! DB-gated integration tests for the work-proposals API — split proposals
//! (NEXT.md curated backlog). Verifies that /api/work-proposals accepts
//! action_type='split', persists the proposal with the correct work_id, and
//! lists it back; merge is exercised for parity.
//!
//! Conventions (same as tests/curator_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test work_proposals_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
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

/// Build a router with the work-proposal endpoints + a real AppState.
async fn app() -> Router {
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
            "/api/work-proposals",
            post(fichub::routes::work_proposals::create_proposal_handler),
        )
        .route(
            "/api/work-proposals",
            get(fichub::routes::work_proposals::list_proposals_handler),
        )
        .route(
            "/api/work-proposals/{id}",
            get(fichub::routes::work_proposals::get_proposal_handler),
        )
        .route(
            "/api/work-proposals/{id}/vote",
            post(fichub::routes::work_proposals::vote_proposal_handler),
        )
        .with_state(state)
}

/// JWT for the given user id + role. The proposal handler derives the role
/// from the users table row (role >= 1 required), so seed the user with a
/// role of 1+ AND mint a matching token.
fn auth_header(user_id: i32, username: &str, trust_level: i16) -> String {
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

/// Seed a user; returns its id. Idempotent.
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role",
    )
    .bind(username)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user lookup failed")
}

/// Seed a work + fic_info pair; returns work_id. Idempotent (unique title).
async fn seed_work(pool: &sqlx::PgPool, title: &str) -> i32 {
    // First delete any work/proposal leftovers carrying the same title so a
    // rerun never collides on the proposal FK or the title lookup.
    let _ = sqlx::query("DELETE FROM work_proposals WHERE work_id IN (SELECT id FROM works WHERE canonical_title = $1)")
        .bind(title)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title = $1")
        .bind(title)
        .execute(pool)
        .await;
    let url_id = format!("wptest-{}", title.to_lowercase().replace(' ', "-"));
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, author_url, author_local_id, chapters, words, description, fic_created, fic_updated, status, source, extra_meta, raw_extended_meta, source_id, author_id, content_hash)
           VALUES ($1, $2, 'WpTest Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, 10, 9001, NULL)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(&url_id)
    .bind(title)
    .execute(pool)
    .await
    .expect("seed fic_info failed");
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO works (canonical_title, canonical_author, default_source_id) VALUES ($1, 'WpTest Author', $2) RETURNING id",
    )
    .bind(title)
    .bind(&url_id)
    .fetch_one(pool)
    .await
    .expect("seed work failed");
    id
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM work_proposals WHERE proposer_id IN (SELECT id FROM users WHERE username LIKE 'wptest_%')",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'WpTest %'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'WpTest %'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username LIKE 'wptest_%'")
        .execute(pool)
        .await;
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

// ── Tests ───────────────────────────────────────────────────────────────────

/// A curator can propose a SPLIT of a work: POST /api/work-proposals with
/// action_type='split' + work_id persists the proposal, and it shows up in
/// the pending list with the right fields.
#[tokio::test]
#[ignore]
async fn work_split_proposal_create_and_list() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let curator = seed_user(&db, "wptest_curator", 1).await;
    let low = seed_user(&db, "wptest_low", 0).await;
    let work_a = seed_work(&db, "WpTest Split Alpha").await;
    let app = app().await;
    let t_curator = auth_header(curator, "wptest_curator", 1);
    let t_low = auth_header(low, "wptest_low", 0);

    // Role < 1 → HTTP 400 (curator role required).
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_low),
        json!({ "action_type": "split", "work_id": work_a }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role: {b}");
    assert_eq!(b["err"], -403, "low role err: {b}");

    // Create a split proposal with per-source assignments in details.
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({
            "action_type": "split",
            "work_id": work_a,
            "details": { "chapters": [1, 2, 3] },
        }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "split create: {b}");
    assert_eq!(b["err"], 0, "split create err: {b}");
    let proposal_id = b["proposal_id"].as_i64().expect("proposal_id") as i32;

    // The row is persisted with action_type='split' + work_id.
    let (action_type, stored_work_id): (String, Option<i32>) =
        sqlx::query_as("SELECT action_type, work_id FROM work_proposals WHERE id = $1")
            .bind(proposal_id)
            .fetch_one(&db)
            .await
            .expect("proposal row");
    assert_eq!(action_type, "split");
    assert_eq!(stored_work_id, Some(work_a));

    // The pending list contains it, and GET /{id} returns the details.
    let (s, b) = get_json(&app, "/api/work-proposals", None).await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    let mine = b["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_i64() == Some(proposal_id as i64))
        .expect("split proposal in pending list");
    assert_eq!(mine["action_type"], "split", "list entry: {b}");
    assert_eq!(mine["work_id"].as_i64(), Some(work_a as i64));

    let (s, b) = get_json(&app, &format!("/api/work-proposals/{proposal_id}"), None).await;
    assert_eq!(s, StatusCode::OK, "get: {b}");
    assert_eq!(b["proposal"]["action_type"], "split");
    assert_eq!(b["proposal"]["work_id"].as_i64(), Some(work_a as i64));
    assert_eq!(
        b["proposal"]["details"]["chapters"][0], 1,
        "details preserved: {b}"
    );

    cleanup(&db).await;
}

/// Split requires a work_id (400 when missing); merge requires both
/// source_work_id + target_work_id; a bogus action_type is rejected.
#[tokio::test]
#[ignore]
async fn work_split_proposal_validation() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let curator = seed_user(&db, "wptest_curator", 1).await;
    let work_a = seed_work(&db, "WpTest Split Beta").await;
    let work_b = seed_work(&db, "WpTest Split Gamma").await;
    let app = app().await;
    let t_curator = auth_header(curator, "wptest_curator", 1);

    // Split without work_id → 400.
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({ "action_type": "split" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "split no work_id: {b}");
    assert_eq!(b["err"], -1, "split no work_id err: {b}");

    // Merge without both ids → 400.
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({ "action_type": "merge", "source_work_id": work_a }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "merge missing target: {b}");

    // Bogus action_type → 400.
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({ "action_type": "explode", "work_id": work_a }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "bad action_type: {b}");

    // Split on a missing work → 404.
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({ "action_type": "split", "work_id": 999999999 }),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "split missing work: {b}");

    // Merge works too (parity check).
    let (s, b) = post_json(
        &app,
        "/api/work-proposals",
        Some(&t_curator),
        json!({ "action_type": "merge", "source_work_id": work_a, "target_work_id": work_b }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "merge create: {b}");
    assert_eq!(b["err"], 0, "merge create err: {b}");

    cleanup(&db).await;
}

//! DB-gated integration tests for kudos: one-click positive appreciation,
//! completely separate from the 5-star `work_ratings` system.
//!
//! Conventions (same as `tests/feedback_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test kudos_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.
//! * Requires migration 048 (kudos table) applied to the local database.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::{delete, get, post},
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

/// Build a router with the kudos + ratings endpoints and a real AppState.
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
            "/api/kudos/{work_id}",
            get(fichub::routes::social::get_kudos_handler),
        )
        .route(
            "/api/kudos/{work_id}",
            post(fichub::routes::social::give_kudos_handler),
        )
        .route(
            "/api/kudos/{work_id}",
            delete(fichub::routes::social::remove_kudos_handler),
        )
        .route("/api/search", get(fichub::search::routes::search_handler))
        .with_state(state)
}

/// JWT for the given user id (real token, real JWT_SECRET from env).
fn auth_header(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "kudos_test_user".into(),
        trust_level: 0,
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

/// Seed a work + fic_info pair; returns the work id (see feedback_api).
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> i32 {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .execute(pool)
    .await
    .expect("seed fic_info failed");

    let existing: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM works WHERE canonical_title = $1 AND canonical_author = $2",
    )
    .bind(title)
    .bind(author)
    .fetch_optional(pool)
    .await
    .expect("work lookup failed");

    let work_id = match existing {
        Some(id) => {
            sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
                .bind(id)
                .bind(url_id)
                .execute(pool)
                .await
                .expect("re-link fic_info to work failed");
            id
        }
        None => {
            let id: i32 = sqlx::query_scalar(
                "INSERT INTO works (canonical_title, canonical_author, default_source_id)
                 VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(title)
            .bind(author)
            .bind(url_id)
            .fetch_one(pool)
            .await
            .expect("seed work failed");
            id
        }
    };

    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    work_id
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool, username: &str) {
    let _ = sqlx::query(
        "DELETE FROM kudos WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(username)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM kudos WHERE work_id IN (SELECT id FROM works WHERE canonical_title LIKE 'KdTest%')",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'KdTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'KdTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(pool)
        .await;
}

/// POST JSON with optional auth; returns parsed JSON.
async fn post_json(
    app: &Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("POST");
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
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

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = builder.body(Body::empty()).unwrap();
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

async fn delete_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("DELETE");
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = builder.body(Body::empty()).unwrap();
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

/// POST creates a kudos → count 1, my_kudos true; GET agrees.
#[ignore]
#[tokio::test]
async fn kudos_post_creates_and_get_shows() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_post_create";
    cleanup(&db, username).await; // self-heal
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "kdurl-create", "KdTest Create", "KdTest Author").await;
    let app = app().await;

    let (status, body) = post_json(
        &app,
        &format!("/api/kudos/{wid}"),
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "give kudos: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["work_id"], wid);
    assert_eq!(body["kudos_count"], 1, "signed-in kudos count: {body}");
    assert_eq!(body["guest_count"], 0);
    assert_eq!(body["my_kudos"], true);

    // Public GET (anonymous) — count visible, my_kudos false.
    let (_, got) = get_json(&app, &format!("/api/kudos/{wid}"), None).await;
    assert_eq!(got["err"], 0);
    assert_eq!(got["kudos_count"], 1);
    assert_eq!(
        got["my_kudos"], false,
        "anonymous viewer must not see my_kudos: {got}"
    );

    // GET with auth — my_kudos true.
    let (_, got2) = get_json(&app, &format!("/api/kudos/{wid}"), Some(&auth_header(uid))).await;
    assert_eq!(got2["my_kudos"], true);

    cleanup(&db, username).await;
}

/// POST twice is idempotent — count stays 1.
#[ignore]
#[tokio::test]
async fn kudos_post_is_idempotent() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_idempotent";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "kdurl-idem", "KdTest Idem", "KdTest Author").await;
    let app = app().await;

    let (s1, b1) = post_json(
        &app,
        &format!("/api/kudos/{wid}"),
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "{b1}");
    assert_eq!(b1["kudos_count"], 1);
    let (s2, b2) = post_json(
        &app,
        &format!("/api/kudos/{wid}"),
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "second POST must not error: {b2}");
    assert_eq!(b2["kudos_count"], 1, "idempotent: {b2}");
    assert_eq!(b2["my_kudos"], true);

    let in_db: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kudos WHERE work_id = $1")
        .bind(wid)
        .fetch_one(&db)
        .await
        .expect("db count");
    assert_eq!(in_db, 1, "exactly one kudos row");

    cleanup(&db, username).await;
}

/// DELETE removes the kudos; deleting a kudos you never gave is a no-op.
#[ignore]
#[tokio::test]
async fn kudos_delete_removes() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_delete";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "kdurl-del", "KdTest Del", "KdTest Author").await;
    let app = app().await;

    let (_, b) = post_json(
        &app,
        &format!("/api/kudos/{wid}"),
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(b["kudos_count"], 1);

    let (status, body) =
        delete_json(&app, &format!("/api/kudos/{wid}"), Some(&auth_header(uid))).await;
    assert_eq!(status, StatusCode::OK, "remove kudos: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["kudos_count"], 0);
    assert_eq!(body["my_kudos"], false);

    // Deleting again is a no-op (still 0, no error).
    let (s2, b2) = delete_json(&app, &format!("/api/kudos/{wid}"), Some(&auth_header(uid))).await;
    assert_eq!(s2, StatusCode::OK, "double delete: {b2}");
    assert_eq!(b2["kudos_count"], 0);

    let in_db: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kudos WHERE work_id = $1")
        .bind(wid)
        .fetch_one(&db)
        .await
        .expect("db count");
    assert_eq!(in_db, 0);

    cleanup(&db, username).await;
}

/// POST/DELETE without auth → 401.
#[ignore]
#[tokio::test]
async fn kudos_requires_auth() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_noauth";
    cleanup(&db, username).await;
    let wid = seed_work(&db, "kdurl-noauth", "KdTest NoAuth", "KdTest Author").await;
    let app = app().await;

    let (s1, _) = post_json(&app, &format!("/api/kudos/{wid}"), json!({}), None).await;
    assert_eq!(s1, StatusCode::UNAUTHORIZED, "anonymous POST must 401");
    let (s2, _) = delete_json(&app, &format!("/api/kudos/{wid}"), None).await;
    assert_eq!(s2, StatusCode::UNAUTHORIZED, "anonymous DELETE must 401");

    cleanup(&db, username).await;
}

/// POST to a nonexistent work → 404.
#[ignore]
#[tokio::test]
async fn kudos_404_on_missing_work() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_404";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let app = app().await;

    let (status, body) = post_json(
        &app,
        "/api/kudos/999999999",
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "missing work must 404: {body}"
    );
    assert_eq!(body["err"], -5, "AppError::NotFound body: {body}");

    cleanup(&db, username).await;
}

/// Guest kudos: one user_id NULL row per work (partial unique index);
/// a second guest insert violates the index; guest kudos never count toward
/// kudos_count and never show as my_kudos.
#[ignore]
#[tokio::test]
async fn guest_kudos_one_per_work() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_guest";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "kdurl-guest", "KdTest Guest", "KdTest Author").await;

    sqlx::query("INSERT INTO kudos (work_id, user_id) VALUES ($1, NULL)")
        .bind(wid)
        .execute(&db)
        .await
        .expect("first guest kudos");
    let second = sqlx::query("INSERT INTO kudos (work_id, user_id) VALUES ($1, NULL)")
        .bind(wid)
        .execute(&db)
        .await;
    assert!(
        second.is_err(),
        "second guest kudos must violate idx_kudos_guest_one_per_work"
    );

    let app = app().await;
    let (_, got) = get_json(&app, &format!("/api/kudos/{wid}"), None).await;
    assert_eq!(
        got["guest_count"], 1,
        "guest kudos counted separately: {got}"
    );
    assert_eq!(
        got["kudos_count"], 0,
        "guest kudos must NOT count as signed-in kudos: {got}"
    );

    // A signed-in user kudos on top → kudos_count 1, guest_count stays 1.
    let (_, b) = post_json(
        &app,
        &format!("/api/kudos/{wid}"),
        json!({}),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(b["kudos_count"], 1);
    assert_eq!(b["guest_count"], 1);
    assert_eq!(b["my_kudos"], true);

    cleanup(&db, username).await;
}

/// Search `min_kudos` filters by the REAL kudos count (signed-in kudos only).
#[ignore]
#[tokio::test]
async fn search_min_kudos_uses_real_kudos() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "kd_search";
    cleanup(&db, username).await;
    let u1 = seed_user(&db, username).await;
    let u2 = seed_user(&db, "kd_search2").await;
    let wid_a = seed_work(&db, "kdurl-search-a", "KdTest Search A", "KdTest Author").await;
    let wid_b = seed_work(&db, "kdurl-search-b", "KdTest Search B", "KdTest Author").await;

    // Work A gets 2 signed-in kudos + 1 guest kudos (guest must NOT count).
    for u in [u1, u2] {
        sqlx::query("INSERT INTO kudos (work_id, user_id) VALUES ($1, $2)")
            .bind(wid_a)
            .bind(u)
            .execute(&db)
            .await
            .expect("insert kudos");
    }
    sqlx::query("INSERT INTO kudos (work_id, user_id) VALUES ($1, NULL)")
        .bind(wid_a)
        .execute(&db)
        .await
        .expect("guest kudos");
    // Work B gets zero kudos.
    let _ = wid_b;

    // Verify the seeding actually landed (self-heal cleanup in a previous
    // failed run must not have eaten the rows the assertions depend on).
    let signed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kudos WHERE work_id = $1 AND user_id IS NOT NULL")
            .bind(wid_a)
            .fetch_one(&db)
            .await
            .expect("kudos count");
    assert_eq!(signed, 2, "work A must have 2 signed-in kudos at seed time");

    // Search by exact title to isolate the two seeded fics.
    // (A bare q here would trip the zero-result fuzzy fallback, which would
    // treat the fics as unmatched and return both regardless of min_kudos.)
    let app = app().await;
    let (status, body) = get_json(&app, "/api/search?q=KdTest&min_kudos=2", None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["total"], 1,
        "only work A has 2+ signed-in kudos: {body}"
    );
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(|s| s.to_string()))
        .collect();
    assert_eq!(
        ids,
        vec!["kdurl-search-a".to_string()],
        "guest kudos must not count: {body}"
    );

    // min_kudos=3 → nothing matches (work A has 2 signed-in, guest excluded).
    let (_, body3) = get_json(&app, "/api/search?q=KdTest&min_kudos=3", None).await;
    assert_eq!(
        body3["total"], 0,
        "min_kudos must exclude guest kudos: {body3}"
    );

    // No min_kudos → both fics returned.
    let (_, both) = get_json(&app, "/api/search?q=KdTest", None).await;
    assert_eq!(both["total"], 2, "both fics without the filter: {both}");

    cleanup(&db, username).await;
    cleanup(&db, "kd_search2").await;
}

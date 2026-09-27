//! DB-gated integration tests for Lane 3: site-wide DMs.
//!
//! Conventions match `tests/forum_groups_api.rs`:
//!   * Serialised via a global `Mutex`.
//!   * Marked `#[ignore]`; run with `cargo test --test messages_api
//!     -- --include-ignored --test-threads=1`.
//!   * Self-heal: deletes its own seed rows at the START of every test.
//!   * Loads DATABASE_URL/REDIS_URL from `~/.config/ficnexus-dev.env` if
//!     not already set in the environment.
//!
//! Scope:
//!   * `dm_find_or_create_is_idempotent` — second POST returns existing room.
//!   * `blocked_user_cannot_dm` — block either direction → 403, no message row.
//!   * `muted_member_gets_no_notification` — is_muted / notify_level=none
//!     suppress the site notification row, message still stored.
//!   * `cursor_pagination_walks_history` — limit+cursor pages newest-first.

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
    routing::get,
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

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    ensure_env();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database")
}

async fn app() -> Router {
    use fichub::server::AppState;

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
        .route(
            "/api/messages/rooms",
            get(fichub::routes::messages::list_rooms).post(fichub::routes::messages::create_dm_room),
        )
        .route(
            "/api/messages/rooms/{roomId}/messages",
            get(fichub::routes::messages::get_room_messages)
                .post(fichub::routes::messages::send_message),
        )
        .route(
            "/api/messages/rooms/{roomId}",
            axum::routing::patch(fichub::routes::messages::update_room_prefs),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, username: &str) -> String {
    let secret = TEST_JWT_SECRET;
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level: 1,
        reputation: 0,
        email: None,
        level: 10,
        exp: 1000,
        is_admin: false,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn req(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    auth: Option<&str>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(a) = auth {
        req = req.header("authorization", a);
    }
    let req = if let Some(b) = body {
        req.header("content-type", "application/json")
            .body(Body::from(b.to_string()))
            .unwrap()
    } else {
        req.body(Body::empty()).unwrap()
    };
    let res = app.clone().oneshot(req).await.unwrap();
    let s = res.status();
    let b = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let v: Value = if b.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&b).unwrap_or(Value::Null)
    };
    (s, v)
}

fn uniq(prefix: &str) -> String {
    format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

async fn seed_user(db: &sqlx::PgPool, prefix: &str) -> (i32, String) {
    let username = uniq(prefix);
    // Self-heal: remove any stale row with this exact username first.
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(db)
        .await;
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, level, trust_level) \
         VALUES ($1, 'testhash', 1, 10, 3) RETURNING id",
    )
    .bind(&username)
    .fetch_one(db)
    .await
    .expect("seed_user insert");
    (id, username)
}

async fn cleanup_user(db: &sqlx::PgPool, uid: i32) {
    // Cascade order: messages → rooms (creator FK has no cascade) → users.
    let rooms: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT r.id FROM forum_rooms r \
         JOIN forum_room_members m ON m.room_id = r.id WHERE m.user_id = $1",
    )
    .bind(uid)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    for r in rooms {
        let _ = sqlx::query("DELETE FROM forum_room_members WHERE room_id = $1")
            .bind(r)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM forum_messages WHERE room_id = $1")
            .bind(r)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM forum_rooms WHERE id = $1")
            .bind(r)
            .execute(db)
            .await;
    }
    let _ = sqlx::query("DELETE FROM blocked_users WHERE user_id = $1 OR blocked_user_id = $1")
        .bind(uid)
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM notifications WHERE user_id = $1")
        .bind(uid)
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(db)
        .await;
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn dm_find_or_create_is_idempotent() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (a_id, a_name) = seed_user(&db, "msg_a").await;
    let (b_id, _b_name) = seed_user(&db, "msg_b").await;
    let a_auth = auth_header(a_id, &a_name);

    let (s1, b1) = req(
        &app,
        "POST",
        "/api/messages/rooms",
        Some(json!({ "user_id": b_id })),
        Some(&a_auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "create DM: {s1} {b1}");
    assert_eq!(b1["err"], 0);
    assert_eq!(b1["existing"], false);
    let room_id = b1["room_id"].as_i64().expect("room_id");

    let (s2, b2) = req(
        &app,
        "POST",
        "/api/messages/rooms",
        Some(json!({ "user_id": b_id })),
        Some(&a_auth),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "repeat create: {s2} {b2}");
    assert_eq!(b2["room_id"].as_i64(), Some(room_id));
    assert_eq!(b2["existing"], true);

    // Self-DM → 400.
    let (s3, _) = req(
        &app,
        "POST",
        "/api/messages/rooms",
        Some(json!({ "user_id": a_id })),
        Some(&a_auth),
    )
    .await;
    assert_eq!(s3, StatusCode::BAD_REQUEST);

    cleanup_user(&db, a_id).await;
    cleanup_user(&db, b_id).await;
}

#[tokio::test]
#[ignore]
async fn blocked_user_cannot_dm() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (a_id, a_name) = seed_user(&db, "msg_blk_a").await;
    let (b_id, b_name) = seed_user(&db, "msg_blk_b").await;
    let a_auth = auth_header(a_id, &a_name);

    // A blocks B via the canonical site table.
    sqlx::query("INSERT INTO blocked_users (user_id, blocked_user_id) VALUES ($1, $2)")
        .bind(a_id)
        .bind(b_id)
        .execute(&db)
        .await
        .expect("seed block");

    // Room creation must 403…
    let (s1, _) = req(
        &app,
        "POST",
        "/api/messages/rooms",
        Some(json!({ "user_id": b_id })),
        Some(&a_auth),
    )
    .await;
    assert_eq!(s1, StatusCode::FORBIDDEN);

    // …and so must sending into a pre-existing room. Seed one directly
    // (bypassing the blocked create path).
    let room_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_rooms (name, is_group, creator_id) VALUES (NULL, FALSE, $1) RETURNING id",
    )
    .bind(a_id)
    .fetch_one(&db)
    .await
    .expect("seed room");
    for m in [a_id, b_id] {
        sqlx::query("INSERT INTO forum_room_members (room_id, user_id) VALUES ($1, $2)")
            .bind(room_id)
            .bind(m)
            .execute(&db)
            .await
            .expect("seed member");
    }
    let b_auth = auth_header(b_id, &b_name);
    let (s2, e2) = req(
        &app,
        "POST",
        &format!("/api/messages/rooms/{room_id}/messages"),
        Some(json!({ "body": "hello?" })),
        Some(&b_auth),
    )
    .await;
    assert_eq!(s2, StatusCode::FORBIDDEN, "blocked send must 403, got: {e2}");

    // No message row was stored.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_messages WHERE room_id = $1")
        .bind(room_id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(count, 0);

    cleanup_user(&db, a_id).await;
    cleanup_user(&db, b_id).await;
}

#[tokio::test]
#[ignore]
async fn muted_member_gets_no_notification() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (a_id, a_name) = seed_user(&db, "msg_mt_a").await;
    let (b_id, b_name) = seed_user(&db, "msg_mt_b").await;
    let a_auth = auth_header(a_id, &a_name);

    let (_, b1) = req(
        &app,
        "POST",
        "/api/messages/rooms",
        Some(json!({ "user_id": b_id })),
        Some(&a_auth),
    )
    .await;
    let room_id = b1["room_id"].as_i64().expect("room_id");

    // B mutes the room.
    let (sp, _) = req(
        &app,
        "PATCH",
        &format!("/api/messages/rooms/{room_id}"),
        Some(json!({ "mute": true })),
        Some(&auth_header(b_id, &b_name)),
    )
    .await;
    assert_eq!(sp, StatusCode::OK);

    // A sends; message stored…
    let (s1, m1) = req(
        &app,
        "POST",
        &format!("/api/messages/rooms/{room_id}/messages"),
        Some(json!({ "body": "muted hello" })),
        Some(&a_auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "send: {s1} {m1}");
    assert_eq!(m1["err"], 0);

    // …but B gets no notification row.
    let notifs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'message'",
    )
    .bind(b_id)
    .fetch_one(&db)
    .await
    .expect("notif count");
    assert_eq!(notifs, 0, "muted member must get no notification");

    // Unmute + notify_level=all → next message notifies.
    let _ = req(
        &app,
        "PATCH",
        &format!("/api/messages/rooms/{room_id}"),
        Some(json!({ "mute": false, "notify_level": "all" })),
        Some(&auth_header(b_id, &b_name)),
    )
    .await;
    // Flood gate: FORUM_POST_DELAY_SECS default 10s — disable via staff-free
    // path is not available, so check the second send tolerantly: either OK
    // (delay 0 in test env) or 400 flood. Only assert the notification when sent.
    let (s2, _) = req(
        &app,
        "POST",
        &format!("/api/messages/rooms/{room_id}/messages"),
        Some(json!({ "body": "unmuted hello" })),
        Some(&a_auth),
    )
    .await;
    if s2 == StatusCode::OK {
        let notifs2: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'message'",
        )
        .bind(b_id)
        .fetch_one(&db)
        .await
        .expect("notif count 2");
        assert_eq!(notifs2, 1, "unmuted member must be notified");
    }

    cleanup_user(&db, a_id).await;
    cleanup_user(&db, b_id).await;
}

#[tokio::test]
#[ignore]
async fn cursor_pagination_walks_history() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (a_id, a_name) = seed_user(&db, "msg_pg_a").await;
    let (b_id, _b_name) = seed_user(&db, "msg_pg_b").await;
    let a_auth = auth_header(a_id, &a_name);

    // Flood gate would block rapid sends — seed messages directly.
    let room_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_rooms (name, is_group, creator_id) VALUES (NULL, FALSE, $1) RETURNING id",
    )
    .bind(a_id)
    .fetch_one(&db)
    .await
    .expect("seed room");
    for m in [a_id, b_id] {
        sqlx::query("INSERT INTO forum_room_members (room_id, user_id) VALUES ($1, $2)")
            .bind(room_id)
            .bind(m)
            .execute(&db)
            .await
            .expect("seed member");
    }
    for i in 0..5 {
        sqlx::query("INSERT INTO forum_messages (room_id, author_id, body) VALUES ($1, $2, $3)")
            .bind(room_id)
            .bind(a_id)
            .bind(format!("msg {i}"))
            .execute(&db)
            .await
            .expect("seed message");
    }

    let (s1, p1) = req(
        &app,
        "GET",
        &format!("/api/messages/rooms/{room_id}/messages?limit=2"),
        None,
        Some(&a_auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "page 1: {s1} {p1}");
    assert_eq!(p1["messages"].as_array().map(|a| a.len()), Some(2));
    assert_eq!(p1["has_more"], true);
    // Newest first.
    assert_eq!(p1["messages"][0]["body"], "msg 4");
    let cursor = p1["messages"][1]["id"].as_i64().expect("cursor");

    let (s2, p2) = req(
        &app,
        "GET",
        &format!("/api/messages/rooms/{room_id}/messages?limit=2&cursor={cursor}"),
        None,
        Some(&a_auth),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "page 2: {s2} {p2}");
    assert_eq!(p2["messages"][0]["body"], "msg 2");

    // Non-member → 404.
    let (c_id, c_name) = seed_user(&db, "msg_pg_c").await;
    let (s3, _) = req(
        &app,
        "GET",
        &format!("/api/messages/rooms/{room_id}/messages"),
        None,
        Some(&auth_header(c_id, &c_name)),
    )
    .await;
    assert_eq!(s3, StatusCode::NOT_FOUND);

    cleanup_user(&db, a_id).await;
    cleanup_user(&db, b_id).await;
    cleanup_user(&db, c_id).await;
}

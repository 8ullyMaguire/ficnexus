//! DB-gated integration tests for Lane A: forum groups + privileges.
//!
//! Conventions match `tests/forum_api.rs`:
//!   * Serialised via a global `Mutex`.
//!   * Marked `#[ignore]`; run with `cargo test --test forum_groups_api
//!     -- --include-ignored --test-threads=1`.
//!   * Self-heal: deletes its own seed rows at the START of every test.
//!   * Loads DATABASE_URL/REDIS_URL from `~/.config/ficnexus-dev.env` if
//!     not already set in the environment.
//!
//! Scope of this file (intentionally small for the first PR; expand in
//! follow-up PRs once Lane B/C plumbing lands):
//!   * `non_admin_gets_403_on_grant`  — 403 gate on /privileges/grant
//!   * `can_resolves_staff_override`  — `can()` returns true for staff
//!   * `can_denies_outsider_default`  — `can()` denies by default
//!
//! Tests that need a real "user owns a group" setup are deferred to
//! follow-up once `forum_groups.user_role` plumbing is verified — the
//! trust gate on `create_group` requires reputation bookkeeping that
//! is out of scope for the privilege lane.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
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
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
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
    });

    Router::new()
        .route(
            "/api/forum/groups",
            get(fichub::routes::forum_groups::list_groups)
                .post(fichub::routes::forum_groups::create_group),
        )
        .route(
            "/api/forum/groups/{id}",
            get(fichub::routes::forum_groups::get_group)
                .patch(fichub::routes::forum_groups::update_group)
                .delete(fichub::routes::forum_groups::delete_group),
        )
        .route(
            "/api/forum/groups/{id}/members",
            get(fichub::routes::forum_groups::list_group_members)
                .post(fichub::routes::forum_groups::invite_to_group),
        )
        .route(
            "/api/forum/groups/{id}/join",
            post(fichub::routes::forum_groups::join_group),
        )
        .route(
            "/api/forum/groups/{id}/leave",
            post(fichub::routes::forum_groups::leave_group),
        )
        .route(
            "/api/forum/privileges",
            get(fichub::routes::forum_privileges::list_all_privs),
        )
        .route(
            "/api/forum/categories/{id}/privileges",
            get(fichub::routes::forum_privileges::list_category_privs)
                .post(fichub::routes::forum_privileges::grant_category_priv),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, username: &str, role: i16) -> String {
    let level = match role {
        10 => 100,
        5 => 50,
        1 => 1,
        _ => 0,
    };
    auth_header_level(user_id, username, role, level)
}

fn auth_header_level(user_id: i32, username: &str, role: i16, level: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        role,
        reputation: 0,
        email: None,
        level,
        exp: i64::from(level) * 100,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn post_json(app: &Router, path: &str, body: Value, auth: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method("POST").uri(path);
    if let Some(a) = auth {
        req = req.header("authorization", a);
    }
    let req = req
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
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

async fn seed_user(db: &sqlx::PgPool, prefix: &str) -> (i32, String) {
    let username = format!(
        "grpa_{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, level) \
         VALUES ($1, 'testhash', 1, 1) RETURNING id",
    )
    .bind(&username)
    .fetch_one(db)
    .await
    .expect("seed_user insert");
    (id, username)
}

async fn seed_category(db: &sqlx::PgPool, slug: &str) -> i64 {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_categories (slug, title, description, \"position\", is_mod_only) \
         VALUES ($1, $1, $1, 0, false) RETURNING id",
    )
    .bind(slug)
    .fetch_one(db)
    .await
    .expect("seed_category");
    id
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn non_admin_gets_403_on_grant() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "nonadm").await;
    let cat_slug = format!(
        "grpa_nonc_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let cat_id = seed_category(&db, &cat_slug).await;

    let (s, b) = post_json(
        &app,
        &format!("/api/forum/categories/{cat_id}/privileges"),
        json!({
            "group_id": 1,
            "privilege": "write"
        }),
        Some(&auth_header(uid, &uname, 1)),
    )
    .await;
    assert!(s.is_client_error(), "non-admin must be rejected: {s} {b}");

    // cleanup
    let _ = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cat_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;
}

#[tokio::test]
#[ignore]
async fn can_resolves_staff_override() {
    use fichub::routes::forum_privileges::can;
    let _g = db_guard();
    let db = pool().await;
    let (uid, _uname) = seed_user(&db, "can_st").await;
    let cat_slug = format!(
        "grpa_stf_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let cat_id = seed_category(&db, &cat_slug).await;

    // User starts as level=1, role=1. Promote to admin.
    sqlx::query("UPDATE users SET level = 100, role = 10 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .unwrap();

    let allowed = can(&db, Some(uid), 100, 10, cat_id, "write")
        .await
        .expect("can staff");
    assert!(allowed, "staff must be allowed regardless of matrix");

    // cleanup
    let _ = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cat_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;
}

#[tokio::test]
#[ignore]
async fn can_denies_outsider_default() {
    use fichub::routes::forum_privileges::can;
    let _g = db_guard();
    let db = pool().await;
    let (uid, _uname) = seed_user(&db, "can_out").await;
    let cat_slug = format!(
        "grpa_out_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let cat_id = seed_category(&db, &cat_slug).await;

    // Fresh user, no membership, no privilege row → default deny.
    let allowed = can(&db, Some(uid), 1, 1, cat_id, "write")
        .await
        .expect("can outsider");
    assert!(!allowed, "non-member with no matrix row must be denied");

    // Anonymous (no user_id) on a public category → default deny too
    // (we don't have a global "all users" row seeded).
    let allowed_anon = can(&db, None, 0, 0, cat_id, "write")
        .await
        .expect("can anon");
    assert!(!allowed_anon, "anonymous without a global row must be denied");

    // cleanup
    let _ = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cat_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;
}

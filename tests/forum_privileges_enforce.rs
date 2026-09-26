//! DB-gated integration tests for Lane 5: privilege-matrix enforcement on
//! create_topic / create_post.
//!
//! Conventions match `tests/forum_groups_api.rs` (global Mutex, #[ignore],
//! self-healing seeds). Run with `cargo test --test forum_privileges_enforce
//! -- --include-ignored --test-threads=1`.
//!
//! Scope:
//!   * `open_category_ignores_matrix` — no privilege rows → today's gates.
//!   * `restricted_category_denies_outsider` — grant a write row to group G;
//!     non-member 403s on create_topic.
//!   * `restricted_category_allows_member` — G member posts OK.
//!   * `restricted_category_staff_bypass` — staff posts OK without membership.

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
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/forum/topics",
            get(fichub::routes::forum::list_topics).post(fichub::routes::forum::create_topic),
        )
        .route(
            "/api/forum/topics/{topicId}/posts",
            post(fichub::routes::forum::create_post),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, username: &str, trust_level: i16, level: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level,
        exp: i64::from(level) * 100,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn post_json(
    app: &Router,
    path: &str,
    body: Value,
    auth: Option<&str>,
) -> (StatusCode, Value) {
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

async fn seed_user(db: &sqlx::PgPool, prefix: &str, role: i16, level: i16) -> (i32, String) {
    let username = uniq(prefix);
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(db)
        .await;
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, level, trust_level) \
         VALUES ($1, 'testhash', $2, $3, 3) RETURNING id",
    )
    .bind(&username)
    .bind(role)
    .bind(level)
    .fetch_one(db)
    .await
    .expect("seed_user insert");
    (id, username)
}

async fn seed_category(db: &sqlx::PgPool, prefix: &str) -> (i64, String) {
    let slug = uniq(prefix);
    let _ = sqlx::query("DELETE FROM forum_categories WHERE slug = $1")
        .bind(&slug)
        .execute(db)
        .await;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_categories (slug, title, description, \"position\", is_mod_only) \
         VALUES ($1, $1, $1, 0, false) RETURNING id",
    )
    .bind(&slug)
    .fetch_one(db)
    .await
    .expect("seed_category");
    (id, slug)
}

async fn cleanup(db: &sqlx::PgPool, cat_id: i64, uids: &[i32]) {
    let _ = sqlx::query("DELETE FROM forum_privileges WHERE category_id = $1")
        .bind(cat_id)
        .execute(db)
        .await;
    let topics: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM forum_topics WHERE category_id = $1")
            .bind(cat_id)
            .fetch_all(db)
            .await
            .unwrap_or_default();
    for t in topics {
        let _ = sqlx::query("DELETE FROM forum_posts WHERE topic_id = $1")
            .bind(t)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM forum_topics WHERE id = $1")
            .bind(t)
            .execute(db)
            .await;
    }
    let groups: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT group_id FROM forum_privileges WHERE category_id = $1",
    )
    .bind(cat_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    // Privilege rows already deleted above; find groups via members instead.
    let _ = groups;
    let _ = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
        .bind(cat_id)
        .execute(db)
        .await;
    for uid in uids {
        let _ = sqlx::query("DELETE FROM forum_group_members WHERE user_id = $1")
            .bind(*uid)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(*uid)
            .execute(db)
            .await;
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn open_category_ignores_matrix() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "prv_open", 1, 10).await;
    let (cat_id, slug) = seed_category(&db, "prv_open_cat").await;

    // No privilege rows anywhere: posting works (flood gate may 400 on
    // rapid repeat — first post must succeed).
    let (s, b) = post_json(
        &app,
        "/api/forum/topics",
        json!({ "category_slug": slug, "title": "open hello", "body": "this is a sufficiently long body for the forum minimum length gate" }),
        Some(&auth_header(uid, &uname, 1, 10)),
    )
    .await;
    assert!(
        s.is_success(),
        "open category must accept posts: {s} {b} (cat {cat_id})"
    );

    cleanup(&db, cat_id, &[uid]).await;
}

#[tokio::test]
#[ignore]
async fn restricted_category_denies_outsider_allows_member() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (member_id, member_name) = seed_user(&db, "prv_mem", 1, 10).await;
    let (out_id, out_name) = seed_user(&db, "prv_out", 1, 10).await;
    let (staff_id, staff_name) = seed_user(&db, "prv_stf", 10, 100).await;
    let (cat_id, slug) = seed_category(&db, "prv_res_cat").await;

    // Restrict the category: group G with a write grant, member in G.
    let group_slug = uniq("prv_g");
    let group_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_groups (name, slug, description, owner_id) VALUES ($1, $1, $1, $2) RETURNING id",
    )
    .bind(&group_slug)
    .bind(member_id)
    .fetch_one(&db)
    .await
    .expect("seed group");
    sqlx::query("INSERT INTO forum_group_members (group_id, user_id) VALUES ($1, $2)")
        .bind(group_id)
        .bind(member_id)
        .execute(&db)
        .await
        .expect("seed membership");
    sqlx::query(
        "INSERT INTO forum_privileges (category_id, group_id, privilege, granted_by) VALUES ($1, $2, 'write', $3)",
    )
    .bind(cat_id)
    .bind(group_id)
    .bind(staff_id)
    .execute(&db)
    .await
    .expect("seed grant");

    let body = json!({ "category_slug": slug, "title": "restricted hello", "body": "this is a sufficiently long body for the forum minimum length gate" });

    // Outsider → 403.
    let (s_out, b_out) = post_json(
        &app,
        "/api/forum/topics",
        body.clone(),
        Some(&auth_header(out_id, &out_name, 1, 10)),
    )
    .await;
    assert_eq!(s_out, StatusCode::FORBIDDEN, "outsider must 403: {b_out}");

    // Member → OK.
    let (s_mem, b_mem) = post_json(
        &app,
        "/api/forum/topics",
        body.clone(),
        Some(&auth_header(member_id, &member_name, 1, 10)),
    )
    .await;
    assert!(s_mem.is_success(), "member must post: {s_mem} {b_mem}");

    // Staff (no membership) → OK via bypass.
    let (s_stf, b_stf) = post_json(
        &app,
        "/api/forum/topics",
        body.clone(),
        Some(&auth_header(staff_id, &staff_name, 10, 100)),
    )
    .await;
    assert!(s_stf.is_success(), "staff must bypass: {s_stf} {b_stf}");

    // Teardown: drop the grant + group, then shared cleanup.
    let _ = sqlx::query("DELETE FROM forum_privileges WHERE category_id = $1")
        .bind(cat_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM forum_group_members WHERE group_id = $1")
        .bind(group_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM forum_groups WHERE id = $1")
        .bind(group_id)
        .execute(&db)
        .await;
    cleanup(&db, cat_id, &[member_id, out_id, staff_id]).await;
}

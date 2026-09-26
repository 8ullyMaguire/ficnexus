//! DB-gated integration tests for Lane 6: poll create + vote + close-notify.
//!
//! Conventions match `tests/forum_groups_api.rs` (global Mutex, #[ignore],
//! self-healing seeds). Run with `cargo test --test polls_api
//! -- --include-ignored --test-threads=1`.
//!
//! Scope:
//!   * `poll_create_then_double_create_conflicts` — create OK, second 409.
//!   * `poll_vote_and_close_notifies_voters` — vote, close, voters get a
//!     site `poll_closed` notification row; non-author close 403s.

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
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/forum/topics/{topicId}/polls",
            axum::routing::post(fichub::routes::forum_polls::create_poll),
        )
        .route(
            "/api/forum/polls/{pollId}",
            get(fichub::routes::forum_polls::get_poll),
        )
        .route(
            "/api/forum/polls/{pollId}/vote",
            axum::routing::post(fichub::routes::forum_polls::vote_on_poll),
        )
        .route(
            "/api/forum/polls/{pollId}/close",
            axum::routing::post(fichub::routes::forum_polls::close_poll),
        )
        .route(
            "/api/forum/polls/{pollId}/results",
            get(fichub::routes::forum_polls::get_poll_results),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
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

async fn seed_topic(db: &sqlx::PgPool, author_id: i32, prefix: &str) -> i64 {
    let slug = uniq(prefix);
    let cat_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_categories (slug, title, description, \"position\", is_mod_only) \
         VALUES ($1, $1, $1, 0, false) RETURNING id",
    )
    .bind(&slug)
    .fetch_one(db)
    .await
    .expect("seed category");
    let topic_id: i64 = sqlx::query_scalar(
        // `slug` does not exist on forum_topics; the column is `topic_slug`.
        // The statement used to name both and pass the same value twice.
        "INSERT INTO forum_topics (category_id, author_id, title, body, topic_slug) \
         VALUES ($1, $2, 'poll test topic', 'body', $3) RETURNING id",
    )
    .bind(cat_id)
    .bind(author_id)
    .bind(format!("{slug}-x"))
    .fetch_one(db)
    .await
    .expect("seed topic");
    topic_id
}

async fn cleanup_poll(db: &sqlx::PgPool, topic_id: i64, uid: i32) {
    let _ = sqlx::query("DELETE FROM forum_polls WHERE topic_id = $1")
        .bind(topic_id)
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM forum_topics WHERE id = $1")
        .bind(topic_id)
        .execute(db)
        .await;
    let cats: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM forum_categories WHERE slug LIKE 'poll\\_%' AND id NOT IN (SELECT category_id FROM forum_topics)",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    for c in cats {
        let _ = sqlx::query("DELETE FROM forum_categories WHERE id = $1")
            .bind(c)
            .execute(db)
            .await;
    }
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
async fn poll_create_then_double_create_conflicts() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "poll_c").await;
    let topic_id = seed_topic(&db, uid, "poll_cat_c").await;
    let auth = auth_header(uid, &uname);

    let (s1, b1) = req(
        &app,
        "POST",
        &format!("/api/forum/topics/{topic_id}/polls"),
        Some(json!({ "question": "Best?", "options": ["A", "B", "C"] })),
        Some(&auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "create poll: {s1} {b1}");
    assert_eq!(b1["err"], 0);
    let poll_id = b1["poll_id"].as_i64().expect("poll_id");

    // Second poll on the same topic → 409.
    let (s2, _) = req(
        &app,
        "POST",
        &format!("/api/forum/topics/{topic_id}/polls"),
        Some(json!({ "question": "Again?", "options": ["X", "Y"] })),
        Some(&auth),
    )
    .await;
    assert_eq!(s2, StatusCode::CONFLICT);

    // One option → 400.
    let topic2 = seed_topic(&db, uid, "poll_cat_c2").await;
    let (s3, _) = req(
        &app,
        "POST",
        &format!("/api/forum/topics/{topic2}/polls"),
        Some(json!({ "question": "Bad?", "options": ["only"] })),
        Some(&auth),
    )
    .await;
    assert_eq!(s3, StatusCode::BAD_REQUEST);

    // Non-author, non-staff → 403.
    let (oid, oname) = seed_user(&db, "poll_out").await;
    let (s4, _) = req(
        &app,
        "POST",
        &format!("/api/forum/topics/{topic2}/polls"),
        Some(json!({ "question": "Hijack?", "options": ["X", "Y"] })),
        Some(&auth_header(oid, &oname)),
    )
    .await;
    assert_eq!(s4, StatusCode::FORBIDDEN);

    let _ = poll_id;
    cleanup_poll(&db, topic_id, uid).await;
    cleanup_poll(&db, topic2, oid).await;
}

#[tokio::test]
#[ignore]
async fn poll_vote_and_close_notifies_voters() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "poll_v").await;
    let (vid, vname) = seed_user(&db, "poll_vtr").await;
    let topic_id = seed_topic(&db, uid, "poll_cat_v").await;
    let auth = auth_header(uid, &uname);
    let vauth = auth_header(vid, &vname);

    let (_, b1) = req(
        &app,
        "POST",
        &format!("/api/forum/topics/{topic_id}/polls"),
        Some(json!({ "question": "Pick?", "options": ["A", "B"] })),
        Some(&auth),
    )
    .await;
    let poll_id = b1["poll_id"].as_i64().expect("poll_id");
    let options: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM forum_poll_options WHERE poll_id = $1 ORDER BY position",
    )
    .bind(poll_id)
    .fetch_all(&db)
    .await
    .expect("options");

    // Voter casts a vote.
    let (vs, _) = req(
        &app,
        "POST",
        &format!("/api/forum/polls/{poll_id}/vote"),
        Some(json!({ "option_id": options[0] })),
        Some(&vauth),
    )
    .await;
    assert_eq!(vs, StatusCode::OK, "vote must succeed");

    // Outsider close → 403.
    let (fs, _) = req(
        &app,
        "POST",
        &format!("/api/forum/polls/{poll_id}/close"),
        None,
        Some(&vauth),
    )
    .await;
    assert_eq!(fs, StatusCode::FORBIDDEN);

    // Author closes → voter gets a site notification.
    let (cs, cb) = req(
        &app,
        "POST",
        &format!("/api/forum/polls/{poll_id}/close"),
        None,
        Some(&auth),
    )
    .await;
    assert_eq!(cs, StatusCode::OK, "close: {cs} {cb}");
    let notifs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'poll_closed'",
    )
    .bind(vid)
    .fetch_one(&db)
    .await
    .expect("notif count");
    assert_eq!(notifs, 1, "voter must get poll_closed notification");

    cleanup_poll(&db, topic_id, uid).await;
    cleanup_poll(&db, -1, vid).await;
}

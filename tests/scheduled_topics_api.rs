//! DB-gated integration tests for Lane 7: scheduled topics.
//!
//! Conventions match `tests/polls_api.rs` (global Mutex, #[ignore],
//! self-healing seeds). Run with `cargo test --test scheduled_topics_api
//! -- --include-ignored --test-threads=1`.
//!
//! Scope:
//!   * `scheduled_create_validates_and_hides` — past `scheduled_at` 400s;
//!     future schedules the topic (200 + scheduled_at echoed); scheduled
//!     topic is absent from `list_topics` and `recent_topics` for a
//!     stranger, but visible via `topic_detail` to the author.
//!   * `scheduled_detail_404s_for_stranger_and_clears_via_update` —
//!     stranger gets "topic not found" on detail; `clear_schedule` via
//!     PATCH publishes (topic reappears in listings).

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
            "/api/forum/topics",
            axum::routing::post(fichub::routes::forum::create_topic),
        )
        .route("/api/forum/topics", get(fichub::routes::forum::list_topics))
        .route(
            "/api/forum/topics/{topicId}",
            get(fichub::routes::forum::topic_detail),
        )
        .route(
            "/api/forum/topics/{topicId}",
            axum::routing::patch(fichub::routes::forum::update_topic),
        )
        .route(
            "/api/forum/recent",
            get(fichub::routes::forum::recent_topics),
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
    let b = axum::body::to_bytes(res.into_body(), 1 << 20)
        .await
        .unwrap();
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

async fn seed_category(db: &sqlx::PgPool, prefix: &str) -> (i64, String) {
    let slug = uniq(prefix);
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_categories (slug, title, description, \"position\", is_mod_only) \
         VALUES ($1, $1, $1, 0, false) RETURNING id",
    )
    .bind(&slug)
    .fetch_one(db)
    .await
    .expect("seed category");
    (id, slug)
}

async fn cleanup(db: &sqlx::PgPool, topic_id: i64, cat_slug: &str, uids: &[i32]) {
    let _ = sqlx::query("DELETE FROM forum_posts WHERE topic_id = $1")
        .bind(topic_id)
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM forum_topics WHERE id = $1")
        .bind(topic_id)
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM forum_categories WHERE slug = $1")
        .bind(cat_slug)
        .execute(db)
        .await;
    for uid in uids {
        let _ = sqlx::query("DELETE FROM notifications WHERE user_id = $1")
            .bind(uid)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uid)
            .execute(db)
            .await;
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn scheduled_create_validates_and_hides() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "sched_a").await;
    let (oid, oname) = seed_user(&db, "sched_o").await;
    let (_cat_id, cat_slug) = seed_category(&db, "sched_cat").await;
    let auth = auth_header(uid, &uname);
    let o_auth = auth_header(oid, &oname);

    // Past timestamp → 400.
    let (s0, _) = req(
        &app,
        "POST",
        "/api/forum/topics",
        Some(json!({
            "title": "Past scheduled", "category_slug": cat_slug,
            "body": "long enough body for the minimum length gate",
            "scheduled_at": "2000-01-01T00:00:00Z"
        })),
        Some(&auth),
    )
    .await;
    assert_eq!(s0, StatusCode::BAD_REQUEST, "past schedule must 400");

    // Future timestamp → 200, scheduled_at echoed.
    let body_text = format!("scheduled body {}", uniq("b"));
    let (s1, b1) = req(
        &app,
        "POST",
        "/api/forum/topics",
        Some(json!({
            "title": format!("Scheduled {}", uniq("t")),
            "category_slug": cat_slug,
            "body": body_text,
            "scheduled_at": "2099-01-01T00:00:00Z"
        })),
        Some(&auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "future schedule: {s1} {b1}");
    assert_eq!(b1["err"], 0);
    assert!(b1["scheduled_at"].as_str().is_some(), "echo: {b1}");
    let topic_id = b1["id"].as_i64().expect("id");

    // Stranger: absent from category listing …
    let (ls, lb) = req(
        &app,
        "GET",
        &format!("/api/forum/topics?category={cat_slug}"),
        None,
        Some(&o_auth),
    )
    .await;
    assert_eq!(ls, StatusCode::OK, "list: {ls} {lb}");
    let listed: Vec<i64> = lb["items"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t["id"].as_i64()).collect())
        .unwrap_or_default();
    assert!(
        !listed.contains(&topic_id),
        "scheduled topic must not list for stranger"
    );

    // … and absent from recent …
    let (rs, rb) = req(&app, "GET", "/api/forum/recent", None, Some(&o_auth)).await;
    assert_eq!(rs, StatusCode::OK, "recent: {rs} {rb}");
    let recent: Vec<i64> = rb["items"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t["id"].as_i64()).collect())
        .unwrap_or_default();
    assert!(
        !recent.contains(&topic_id),
        "scheduled topic must not appear in recent"
    );

    // … but the author can preview via detail.
    let (ds, db1) = req(
        &app,
        "GET",
        &format!("/api/forum/topics/{topic_id}"),
        None,
        Some(&auth),
    )
    .await;
    assert_eq!(ds, StatusCode::OK, "author preview: {ds} {db1}");

    cleanup(&db, topic_id, &cat_slug, &[uid, oid]).await;
}

#[tokio::test]
#[ignore]
async fn scheduled_detail_404s_for_stranger_and_clears_via_update() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "sched2_a").await;
    let (oid, oname) = seed_user(&db, "sched2_o").await;
    let (_cat_id, cat_slug) = seed_category(&db, "sched2_cat").await;
    let auth = auth_header(uid, &uname);
    let o_auth = auth_header(oid, &oname);

    let body_text = format!("scheduled body {}", uniq("b"));
    let (s1, b1) = req(
        &app,
        "POST",
        "/api/forum/topics",
        Some(json!({
            "title": format!("Scheduled {}", uniq("t")),
            "category_slug": cat_slug,
            "body": body_text,
            "scheduled_at": "2099-06-01T00:00:00Z"
        })),
        Some(&auth),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "create: {s1} {b1}");
    let topic_id = b1["id"].as_i64().expect("id");

    // Stranger detail → 404-shape ("topic not found").
    let (ds, db1) = req(
        &app,
        "GET",
        &format!("/api/forum/topics/{topic_id}"),
        None,
        Some(&o_auth),
    )
    .await;
    assert_eq!(ds, StatusCode::BAD_REQUEST, "stranger detail: {ds} {db1}");

    // Reschedule to a past timestamp via PATCH → 400.
    let (ps, _) = req(
        &app,
        "PATCH",
        &format!("/api/forum/topics/{topic_id}"),
        Some(json!({ "scheduled_at": "2000-01-01T00:00:00Z" })),
        Some(&auth),
    )
    .await;
    assert_eq!(ps, StatusCode::BAD_REQUEST, "past reschedule must 400");

    // Author clears the schedule → publishes now.
    let (cs, cb) = req(
        &app,
        "PATCH",
        &format!("/api/forum/topics/{topic_id}"),
        Some(json!({ "clear_schedule": true })),
        Some(&auth),
    )
    .await;
    assert_eq!(cs, StatusCode::OK, "clear: {cs} {cb}");

    // Now visible in the category listing for the stranger.
    let (ls, lb) = req(
        &app,
        "GET",
        &format!("/api/forum/topics?category={cat_slug}"),
        None,
        Some(&o_auth),
    )
    .await;
    assert_eq!(ls, StatusCode::OK, "list: {ls} {lb}");
    let listed: Vec<i64> = lb["items"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t["id"].as_i64()).collect())
        .unwrap_or_default();
    assert!(
        listed.contains(&topic_id),
        "cleared topic must list for stranger"
    );

    cleanup(&db, topic_id, &cat_slug, &[uid, oid]).await;
}

/// Verify that the publish-scheduled SQL (run by the bin) flips due topics,
/// notifies authors, and makes them visible in listings.
#[tokio::test]
#[ignore = "DB-gated"]
async fn publish_scheduled_flips_and_notifies() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;
    let (uid, uname) = seed_user(&db, "sched_pub").await;
    let (oid, oname) = seed_user(&db, "sched_observer").await;
    // The INSERT binds `cat_id`, not `cat_slug`: `category_id` is an i64 FK and
    // the slug was bound to it, so the insert failed at
    // `.expect("seed scheduled topic")`. The slug is still used below, for the
    // `?category=<slug>` listing queries.
    // The same fixture had already been repaired once for naming a nonexistent
    // `slug` column (the column is `topic_slug`); the category binding was
    // missed in that repair. See docs/specs/final-six-suites.md.
    let (cat_id, cat_slug) = seed_category(&db, "sched_pub_cat").await;
    let auth = auth_header(uid, &uname);
    let o_auth = auth_header(oid, &oname);

    // Seed a topic scheduled for the past (simulate due topic).
    let topic_id: i64 = sqlx::query_scalar(
        // `slug` does not exist on forum_topics; the column is `topic_slug`.
        // The statement used to name both and bind the same value twice.
        "INSERT INTO forum_topics (category_id, author_id, title, body, topic_slug, scheduled_at)
         VALUES ($1, $2, $3, $4, $5, '2000-01-01T00:00:00Z'::timestamptz)
         RETURNING id",
    )
    .bind(cat_id)
    .bind(uid)
    .bind(format!("Publishable {}", uniq("pt")))
    .bind(format!("publish body {}", uniq("pb")))
    .bind(format!("pub-{}", uniq("slug")))
    .fetch_one(&db)
    .await
    .expect("seed scheduled topic");

    // Verify hidden from stranger listing before publish.
    let (ls, lb) = req(
        &app,
        "GET",
        &format!("/api/forum/topics?category={cat_slug}"),
        None,
        Some(&o_auth),
    )
    .await;
    let before: Vec<i64> = lb["items"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t["id"].as_i64()).collect())
        .unwrap_or_default();
    assert!(!before.contains(&topic_id), "must be hidden before publish");

    // Call the SAME function the publish-scheduled binary calls. This test used
    // to paste a copy of the flip SQL inline, which meant it asserted against a
    // query that could drift from the one cron actually runs -- and it never
    // notified the author, so the notification assertion below was checking a
    // row nothing in this test could ever have created.
    let flipped = fichub::db::queries::social::publish_due_topics(&db)
        .await
        .expect("publish due topics");
    assert!(
        flipped.iter().any(|t| t.id == topic_id),
        "topic must be in the flipped set"
    );

    // Verify visible in listing after publish.
    let (ls2, lb2) = req(
        &app,
        "GET",
        &format!("/api/forum/topics?category={cat_slug}"),
        None,
        Some(&o_auth),
    )
    .await;
    let after: Vec<i64> = lb2["items"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t["id"].as_i64()).collect())
        .unwrap_or_default();
    assert!(after.contains(&topic_id), "must be visible after publish");

    // Verify author received a notification. The column is `notification_type`,
    // not `type`; the old query referenced a non-existent column and
    // `.unwrap_or(0)` turned that SQL error into a quiet 0, so the assertion
    // reported "no notification" instead of "your query is wrong". No
    // `unwrap_or` here: a broken assertion query must fail loudly.
    let notif_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications
         WHERE user_id = $1 AND notification_type = 'topic_published'",
    )
    .bind(uid)
    .fetch_one(&db)
    .await
    .expect("count publish notifications");
    assert!(
        notif_count >= 1,
        "author should have a notification, found {notif_count}"
    );

    cleanup(&db, topic_id, &cat_slug, &[uid, oid]).await;
}

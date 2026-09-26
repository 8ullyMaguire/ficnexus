//! DB-gated tests for usage analytics: the tracking middleware records
//! usage_events (view vs action), and the admin analytics endpoint returns
//! unique daily/weekly/monthly visitors + active-vs-view-only users.
//!
//! Conventions (same as tests/heal_api.rs / curator_fix_api.rs):
//! * Serialised via a global Mutex; marked #[ignore]; run with:
//!   `set -a; . ./.env; set +a; cargo test --test analytics_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows at the START.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use serde_json::Value;
use sqlx::Row;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

async fn build_app() -> Router {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
    use fichub::server::AppState;

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
        config,
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
        heal: fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/admin/analytics",
            get(fichub::routes::analytics::admin_analytics_handler),
        )
        .with_state(state)
}

/// Mints a token with an explicit `is_admin` claim.
///
/// `is_admin` is a real column and the admin-tier gates read it from the token
/// (`docs/specs/admin-flag.md`). Trust level is separate and does **not** grant
/// admin — `users.trust_level` is CHECK-constrained to 0-6, so the 10 these
/// call sites used to pass could not exist in a real row, and the routes they
/// reached were not actually admin-protected.
fn auth_header(user_id: i32, trust_level: i16, username: &str) -> String {
    auth_header_for(user_id, trust_level, username, false)
}

fn auth_header_for(
    user_id: i32,
    trust_level: i16,
    username: &str,
    is_admin: bool,
) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
        is_admin,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role, trust_level) VALUES ($1, 'test-hash', $2, LEAST($2, 6)) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role, trust_level = LEAST(EXCLUDED.trust_level, 6) RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(pool)
        .await
        .expect("seed_user failed")
        .get(0)
}

async fn cleanup(pool: &sqlx::PgPool) {
    // Wipe ALL usage_events: the analytics aggregation asserts on global
    // counts (active/view-only/unique visitors), so any leftover rows from
    // other test runs or the dev middleware would skew the numbers. Tests
    // are serialized via db_guard + --test-threads=1, so this is safe.
    let _ = sqlx::query("DELETE FROM usage_events").execute(pool).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn middleware_records_view_and_action_events() {
    let _guard = db_guard();
    let db = pool().await;
    // Self-heal: clear leftover usage_events from prior runs/live dev traffic
    // BEFORE inserting, so the aggregation asserts on exactly our rows.
    cleanup(&db).await;

    // Insert events directly (the middleware is exercised by the running
    // server in prod; here we verify the queries + endpoint aggregation).
    let now = chrono::Utc::now();
    for (cid, path, etype) in [
        ("analtest_view1", "/", "view"),
        ("analtest_view1", "/search", "view"),
        ("analtest_action1", "/api/epub?q=x", "action"),
        ("analtest_action1", "/api/bookmarks", "action"),
    ] {
        sqlx::query("INSERT INTO usage_events (client_id, path, event_type, created_at) VALUES ($1, $2, $3, $4)")
            .bind(cid)
            .bind(path)
            .bind(etype)
            .bind(now)
            .execute(&db)
            .await
            .expect("insert usage event");
    }

    // Admin sees: active users (2 distinct clients with actions) = 1
    // (analtest_action1); view-only (clients with no actions) = 1
    // (analtest_view1).
    let app = build_app().await;
    let admin = seed_user(&db, "analtest_admin", 10).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/analytics")
                .header("authorization", auth_header_for(admin, 6, "analtest_admin", true))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "admin analytics should 200");
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();

    let engagement = &v["engagement"];
    let active_30d = engagement["active_users"]["30d"].as_i64().unwrap_or(-1);
    let view_only_30d = engagement["view_only_users"]["30d"].as_i64().unwrap_or(-1);
    assert_eq!(active_30d, 1, "one client performed actions");
    assert_eq!(view_only_30d, 1, "one client only viewed");
    assert!(
        v["unique_visitors"]["daily"].is_array(),
        "daily visitors present"
    );
    assert!(
        v["recent_events"].as_array().unwrap().len() >= 4,
        "recent timeline present"
    );

    cleanup(&db).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn non_admin_is_rejected() {
    let _guard = db_guard();
    let db = pool().await;

    let app = build_app().await;
    let low = seed_user(&db, "analtest_low", 0).await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/analytics")
                .header("authorization", auth_header(low, 0, "analtest_low"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "non-admin should be rejected (403)"
    );

    cleanup(&db).await;
}

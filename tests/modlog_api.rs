//! DB-gated tests for the modlog: recording + the GET /api/modlog endpoint
//! readable by ANY logged-in user (transparency).
//!
//! Conventions (same as tests/heal_api.rs / curator_fix_api.rs / analytics_api.rs):
//! * Serialised via a global Mutex; marked #[ignore]; run with:
//!   `set -a; . ./.env; set +a; cargo test --test modlog_api -- --include-ignored --test-threads=1`
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
        .expect("failed to connect to test database")
}

async fn build_app() -> Router {
    use fichub::server::AppState;
    let config = fichub::config::Config::from_env();
    let db = pool().await;
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("redis url");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .unwrap();
    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());
    let state = Arc::new(AppState {
        config,
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
            http_client.clone(),
            fichub::config::Config::from_env(),
            scraper_registry,
        ),
        heal: fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: fichub::services::ollama::OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "llama3.1:8b".into(),
            http_client.clone(),
        ),
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });
    Router::new()
        .route("/api/modlog", get(fichub::routes::modlog::modlog_handler))
        .with_state(state)
}

fn auth_header(user_id: i32, trust_level: i16, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    format!(
        "Bearer {}",
        fichub::routes::auth::create_token(&user, &secret).expect("token")
    )
}

async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(pool)
        .await
        .expect("seed_user")
        .get(0)
}

async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM modlog WHERE actor_username LIKE 'modtest_%' OR actor_username LIKE 'modlogtest_%'")
        .execute(pool)
        .await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn any_logged_in_user_can_read_modlog() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    // Record two entries as a mod + admin.
    fichub::modlog::record_json(
        &db,
        None,
        Some("modtest_curator".into()),
        "ban_user",
        "user",
        "1",
        vec![],
    )
    .await;
    fichub::modlog::record_json(
        &db,
        None,
        Some("modtest_admin".into()),
        "merge_tags",
        "tag",
        "2",
        vec![("target_tag_id", Value::from(3))],
    )
    .await;

    let app = build_app().await;
    // A plain regular user (role 0) can read it.
    let regular = seed_user(&db, "modtest_regular", 0).await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/modlog")
                .header("authorization", auth_header(regular, 0, "modtest_regular"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "regular user can read modlog"
    );
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let entries = v["entries"].as_array().unwrap();
    assert!(
        entries.iter().any(|e| e["action"] == "ban_user"),
        "ban_user entry visible"
    );
    assert!(
        entries.iter().any(|e| e["action"] == "merge_tags"),
        "merge_tags entry visible"
    );

    cleanup(&db).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn anonymous_cannot_read_modlog() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/modlog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "anonymous rejected (400-as-401)"
    );

    cleanup(&db).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn action_filter_works() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    // The modlog table is shared across DB-gated suites (admin_api records
    // ban_user entries with other usernames); the filter-count assertion
    // needs a clean slate, so clear the whole table.
    let _ = sqlx::query("DELETE FROM modlog").execute(&db).await;

    fichub::modlog::record_json(
        &db,
        None,
        Some("modlogtest_admin".into()),
        "ban_user",
        "user",
        "1",
        vec![],
    )
    .await;
    fichub::modlog::record_json(
        &db,
        None,
        Some("modlogtest_admin".into()),
        "delete_tag",
        "tag",
        "5",
        vec![],
    )
    .await;

    let app = build_app().await;
    let user = seed_user(&db, "modlogtest_user", 0).await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/modlog?action=ban_user")
                .header("authorization", auth_header(user, 0, "modlogtest_user"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "only ban_user entries: {v}");
    assert_eq!(entries[0]["action"], "ban_user");

    cleanup(&db).await;
}

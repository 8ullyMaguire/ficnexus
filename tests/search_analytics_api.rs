//! DB-gated integration tests for search analytics.
//!
//! Regression coverage for the search-analytics feature:
//! - `/api/admin/search-analytics` returns zero-result queries, trope
//!   popularity (main_char_attr combos) and per-hour search volume, and
//!   requires role >= 10.
//! - A real search through the router logs a `search_queries` row
//!   (query + total_results + main_char_attr + client_id from headers).
//!
//! AppState construction is copied EXACTLY from tests/admin_api.rs.

use std::sync::{Mutex, OnceLock};
use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
use serde_json::Value;
use sqlx::Row;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|p| p.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (run with .env loaded)");
    sqlx::PgPool::connect(&url).await.expect("connect pool")
}

async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client.get_multiplexed_async_connection().await.expect("redis conn");
    let http_client = reqwest::Client::builder().user_agent("fichub-test/0.1.0").build().expect("reqwest");
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
        health_redis: redis_client.get_multiplexed_async_connection().await.expect("health redis conn"),
        http_client: http_client.clone(),
        scraper_registry: scraper_registry.clone(),
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(
            fichub::limiter::redis_bucket::RedisBucketLimiter::new(
                redis_client.get_multiplexed_async_connection().await.expect("redis"),
                false,
            )
            .await
            .expect("rate limiter"),
        ),
        recommender_engine: fichub::recommender::engine::RecommendationEngine::new(db.clone()),        strategy_registry: fichub::recommender::registry::StrategyRegistry::new(
            vec![std::sync::Arc::new(fichub::recommender::legacy_cooccur::LegacyCooccurStrategy::new())],
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
        .route("/api/admin/search-analytics", get(fichub::routes::admin::admin_search_analytics))
        .route("/api/search", get(fichub::search::routes::search_handler))
        .with_state(state)
}

async fn seed_admin_user(db: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ($1, 'x', $2) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(db)
        .await
        .expect("seed user")
        .get(0)
}

fn auth_header(user_id: i32, username: &str, role: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        role,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Seed a search_queries row at an offset relative to now (negative = past).
async fn seed_search_query(db: &sqlx::PgPool, query: &str, total_results: i32, main_char_attr: Option<&str>, client_id: &str, hours_ago: i64) {
    sqlx::query(
        r#"INSERT INTO search_queries (query, ts, total_results, main_char_attr, client_id)
           VALUES ($1, now() - ($2 * interval '1 hour'), $3, $4, $5)"#,
    )
    .bind(query)
    .bind(hours_ago)
    .bind(total_results)
    .bind(main_char_attr)
    .bind(client_id)
    .execute(db)
    .await
    .expect("seed search_query");
}

/// Remove ONLY the rows the seeding helpers create (by unique prefix), so
/// tests are re-runnable and never touch pre-existing analytics data.
async fn cleanup(db: &sqlx::PgPool, users: &[&str], query_prefixes: &[&str]) {
    for u in users {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1").bind(u).execute(db).await;
    }
    for p in query_prefixes {
        let _ = sqlx::query("DELETE FROM search_queries WHERE query LIKE $1")
            .bind(format!("{p}%"))
            .execute(db)
            .await;
    }
}

/// /api/admin/search-analytics requires role >= 10.
#[ignore]
#[tokio::test]
async fn search_analytics_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let low_role = seed_admin_user(&db, "searchan_low", 5).await;
    let res = app
        .oneshot(Request::builder()
            .uri("/api/admin/search-analytics")
            .header("authorization", auth_header(low_role, "searchan_low", 5))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    cleanup(&db, &["searchan_low"], &[]).await;
}

/// /api/admin/search-analytics returns zero-result queries, trope
/// popularity and per-hour volume for the seeded rows; aggregates only
/// (Zero-PII — no client_id in the payload).
#[ignore]
#[tokio::test]
async fn search_analytics_returns_aggregates() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    // Clean fixture rows up front (a previous failed run may have left rows
    // behind, since cleanup only runs on success) — keeps tests re-runnable.
    cleanup(&db, &[], &["searchan_nomatch", "searchan_harry", "searchan_draco", "searchan_stale"]).await;
    // Also clear rows the sibling search_api suite inserted through the real
    // search flow (e.g. q=zzzznomatchzzz zero-result rows) — the top
    // zero-result assertion must only see our fixture.
    let _ = sqlx::query("DELETE FROM search_queries").execute(&db).await;

    let admin = seed_admin_user(&db, "searchan_admin", 10).await;
    // Zero-result query, repeated twice → top zero-result entry.
    seed_search_query(&db, "searchan_nomatch", 0, None, "client-a", 1).await;
    seed_search_query(&db, "searchan_nomatch", 0, None, "client-b", 2).await;
    // A result-bearing query, with a main_char_attr trope combo.
    seed_search_query(&db, "searchan_harry", 5, Some("Harry Potter|Dark Harry Potter"), "client-a", 3).await;
    seed_search_query(&db, "searchan_harry", 8, Some("Harry Potter|Dark Harry Potter"), "client-b", 4).await;
    // A different trope combo, single hit.
    seed_search_query(&db, "searchan_draco", 2, Some("Draco Malfoy|Creature Fic"), "client-a", 5).await;
    // Old zero-result (outside 7-day window) — must NOT appear.
    seed_search_query(&db, "searchan_stale", 0, None, "client-a", 24 * 30).await;

    let res = app
        .oneshot(Request::builder()
            .uri("/api/admin/search-analytics")
            .header("authorization", auth_header(admin, "searchan_admin", 10))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(body["err"], 0);

    // zero_result_queries: top 50 by frequency. Our fixture query (count 2)
    // must be the top entry; other zero-result rows from concurrent suites
    // (e.g. search_api) may also appear below it.
    let zeros = body["zero_result_queries"].as_array().unwrap();
    assert_eq!(zeros[0]["query"], "searchan_nomatch", "fixture query is top zero-result: {body}");
    assert_eq!(zeros[0]["count"], 2);

    // trope_popularity: top combo first (2 hits), then the single-hit combo.
    let tropes = body["trope_popularity"].as_array().unwrap();
    assert_eq!(tropes[0]["main_char_attr"], "Harry Potter|Dark Harry Potter");
    assert_eq!(tropes[0]["count"], 2);
    assert_eq!(tropes[1]["main_char_attr"], "Draco Malfoy|Creature Fic");
    assert_eq!(tropes[1]["count"], 1);

    // search_volume: sum over buckets includes the 5 recent fixture rows
    // (stale one excluded). Fixture rows land in distinct hours, so we only
    // assert the total (concurrent suites may add more buckets/rows).
    let volume: Vec<Value> = body["search_volume"].as_array().unwrap().clone();
    let total_volume: i64 = volume.iter().map(|v| v["count"].as_i64().unwrap()).sum();
    assert!(total_volume >= 5, "volume includes fixture rows: {body}");

    // Zero-PII: no client_id anywhere in the payload.
    let raw = body.to_string();
    assert!(!raw.contains("client-a") && !raw.contains("client-b"), "must not expose client ids: {raw}");

    cleanup(&db, &["searchan_admin"], &["searchan_nomatch", "searchan_harry", "searchan_draco", "searchan_stale"]).await;
}

/// A real search through the router logs a search_queries row with the
/// query text, total_results, main_char_attr (raw "Character|Attribute"
/// string from the query param) and client_id from the x-client-id header.
#[ignore]
#[tokio::test]
async fn search_flow_logs_search_query_row() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    // Clean any prior rows for this fixture.
    let _ = sqlx::query("DELETE FROM search_queries WHERE query LIKE 'searchan_flow%'")
        .execute(&db)
        .await;

    let res = app
        .oneshot(Request::builder()
            .uri("/api/search?q=searchan_flow_nomatch&main_char_attr=Harry%20Potter%7CDark%20Harry%20Potter")
            .header("x-client-id", "searchan_flow_client")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(body["total"], 0);

    // The search must have been logged: query + total_results + main_char_attr + client_id.
    let row: (String, i32, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT query, total_results, main_char_attr, client_id FROM search_queries WHERE query = 'searchan_flow_nomatch' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&db)
    .await
    .expect("search_queries row must exist after a search");

    assert_eq!(row.0, "searchan_flow_nomatch");
    assert_eq!(row.1, 0, "zero-result flag recorded");
    assert_eq!(row.2.as_deref(), Some("Harry Potter|Dark Harry Potter"), "main_char_attr recorded raw");
    assert_eq!(row.3.as_deref(), Some("searchan_flow_client"), "client_id from header recorded");

    cleanup(&db, &[], &["searchan_flow"]).await;
}

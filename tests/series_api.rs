//! DB-gated integration tests for the series & author pages feature:
//! `GET /api/series/{id}` (ordered works + next-in-series) and
//! `GET /api/authors/{name}` (author bibliography with stats).
//!
//! Conventions (same as `tests/feedback_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test series_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use serde_json::Value;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

#[allow(dead_code)]
const PREFIX: &str = "SeriesApiTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    c
}

/// Build a router with the series + author-bibliography endpoints.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = test_config();
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
            test_config(),
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
        .route("/api/series/{id}", get(fichub::routes::series::get_series))
        .route(
            "/api/authors/{name}",
            get(fichub::routes::series::get_author),
        )
        .with_state(state)
}

async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
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

/// Seed a work + its default fic_info source. Returns (work_id, url_id).
async fn seed_work(
    pool: &sqlx::PgPool,
    url_id: &str,
    title: &str,
    author: &str,
    words: i64,
) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 1, $4, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .bind(words)
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
        Some(id) => id,
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

    (work_id, url_id.to_string())
}

/// Seed a series with the given ordered work ids. Returns the series id.
async fn seed_series(pool: &sqlx::PgPool, name: &str, work_ids: &[i32]) -> i32 {
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO series (name, description) VALUES ($1, 'series-api-test') RETURNING id",
    )
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("seed series failed");

    for (pos, wid) in work_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO series_works (series_id, work_id, position) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(wid)
        .bind(pos as i32)
        .execute(pool)
        .await
        .expect("seed series_works failed");
    }
    id
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM series_works WHERE series_id IN (SELECT id FROM series WHERE name LIKE 'SeriesApiTest%')")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM series WHERE name LIKE 'SeriesApiTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'SeriesApiTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'SeriesApiTest%'")
        .execute(pool)
        .await;
}

/// A series with 3 works returns them in reading order, each with
/// next_in_series pointing at the following work (the last has none).
#[ignore]
#[tokio::test]
async fn series_detail_ordered_with_next_in_series() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let (w1, u1) = seed_work(
        &db,
        "seriesapi_a",
        "SeriesApiTest Alpha",
        "SeriesApiTest Author",
        1000,
    )
    .await;
    let (w2, u2) = seed_work(
        &db,
        "seriesapi_b",
        "SeriesApiTest Beta",
        "SeriesApiTest Author",
        2000,
    )
    .await;
    let (w3, u3) = seed_work(
        &db,
        "seriesapi_c",
        "SeriesApiTest Gamma",
        "SeriesApiTest Author",
        3000,
    )
    .await;
    let sid = seed_series(&db, "SeriesApiTest Series", &[w1, w2, w3]).await;

    let router = app().await;
    let (status, body) = get_json(&router, &format!("/api/series/{sid}")).await;
    assert_eq!(status, StatusCode::OK, "series detail: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["series"]["name"], "SeriesApiTest Series");
    assert_eq!(body["series"]["work_count"], 3);

    let works = body["works"].as_array().expect("works array");
    assert_eq!(works.len(), 3);
    // Reading order = position order (insertion order here).
    assert_eq!(works[0]["work_id"], w1);
    assert_eq!(works[1]["work_id"], w2);
    assert_eq!(works[2]["work_id"], w3);

    // Each work links to its fic page via /fic/{url_id}.
    assert_eq!(works[0]["url_id"], u1);
    assert_eq!(works[1]["url_id"], u2);
    assert_eq!(works[2]["url_id"], u3);

    // next_in_series: 1 → 2 → 3, last has none.
    assert_eq!(works[0]["next_in_series"]["work_id"], w2);
    assert_eq!(works[0]["next_in_series"]["url_id"], u2);
    assert_eq!(works[1]["next_in_series"]["work_id"], w3);
    assert_eq!(works[1]["next_in_series"]["url_id"], u3);
    assert!(
        works[2]["next_in_series"].is_null(),
        "last work has no next"
    );

    cleanup(&db).await;
}

/// GET /api/series/{id} for a missing series returns 404 with err < 0.
#[ignore]
#[tokio::test]
async fn series_detail_not_found() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let router = app().await;
    let (status, body) = get_json(&router, "/api/series/99999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["err"].as_i64().unwrap() < 0);

    cleanup(&db).await;
}

/// Author page aggregates works (canonical entries), surfaces orphan sources,
/// and reports work count + total words.
#[ignore]
#[tokio::test]
async fn author_detail_aggregates_works_and_stats() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let (w1, _u1) = seed_work(
        &db,
        "seriesapi_author_a",
        "SeriesApiTest Auth A",
        "SeriesApiTest Person",
        1000,
    )
    .await;
    let (w2, _u2) = seed_work(
        &db,
        "seriesapi_author_b",
        "SeriesApiTest Auth B",
        "SeriesApiTest Person",
        2000,
    )
    .await;
    // Orphan source (no work yet) — must still surface in the bibliography.
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ('seriesapi_author_c', 'SeriesApiTest Auth C', 'SeriesApiTest Person', NULL, NULL,
                  1, 500, 'orphan', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .execute(&db)
    .await
    .expect("seed orphan fic_info failed");

    let router = app().await;
    let (status, body) = get_json(&router, "/api/authors/SeriesApiTest%20Person").await;
    assert_eq!(status, StatusCode::OK, "author detail: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["author"]["name"], "SeriesApiTest Person");
    // 2 works + 1 orphan = 3 fic_info rows by this author.
    assert_eq!(body["author"]["work_count"], 3);
    assert_eq!(body["author"]["total_words"], 3500);

    let works = body["works"].as_array().expect("works array");
    let ids: Vec<i32> = works
        .iter()
        .filter_map(|w| w["work_id"].as_i64().map(|v| v as i32))
        .collect();
    assert!(
        ids.contains(&w1) && ids.contains(&w2),
        "canonical works present: {ids:?}"
    );

    let orphans = body["orphans"].as_array().expect("orphans array");
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0]["url_id"], "seriesapi_author_c");
    assert_eq!(orphans[0]["canonical_title"], "SeriesApiTest Auth C");

    cleanup(&db).await;
}

/// Author page for an unknown author returns empty works/stats (200, err 0),
/// never an error — an author with no works yet is still a valid page.
#[ignore]
#[tokio::test]
async fn author_detail_unknown_author_returns_empty() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let router = app().await;
    let (status, body) = get_json(&router, "/api/authors/NoSuchAuthor%20XYZ").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["err"], 0);
    assert_eq!(body["author"]["work_count"], 0);
    assert_eq!(body["author"]["total_words"], 0);
    assert_eq!(body["works"].as_array().map(|a| a.len()).unwrap_or(0), 0);

    cleanup(&db).await;
}

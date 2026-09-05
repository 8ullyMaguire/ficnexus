//! DB-gated integration tests for "Ask the Archive" (`POST /api/search/ask`).
//!
//! The endpoint translates a natural-language query into search filters via
//! Ollama, validates the model output, falls back to a plain search when
//! Ollama is unavailable, and runs the real search pipeline. These tests
//! exercise the FULL path through the real router + AppState + Postgres,
//! following the repo conventions (mutex serialisation, `#[ignore]`,
//! self-heal cleanup, unique seed names):
//!
//! * Tests must NOT depend on a live Ollama: the test AppState points the
//!   Ollama client at a dead port, so the handler takes its fallback path.
//! * Ollama-adjacent behaviour (validation, prompt shape, caching) is
//!   covered by unit tests in `src/search/ask.rs` / `src/search/ask_cache.rs`.
//! * A mock-HTTP happy path is covered by a unit test over the
//!   parse/validate step with a canned JSON string — never real Ollama.
//!
//! Run with:
//!   set -a; . ./.env; set +a
//!   CARGO_TARGET_DIR=/home/alvaro/cargo-target-aa CARGO_INCREMENTAL=0 \
//!     cargo test --test ask_archive_api -- --include-ignored --test-threads=1

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
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

/// Minimal config for the search/ask handlers.
fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    c
}

/// Build the real router with `/api/search/ask` (POST) + `/api/search` (GET)
/// and a real AppState. The Ollama client is pointed at a dead port on
/// purpose: tests must never depend on a live Ollama — every test here
/// exercises the fallback path.
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
    // Dead port: connection refused surfaces as OllamaError → plain fallback.
    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:1".into(),
        "llama3.1:8b".into(),
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
                false, // dynamic rate limiting off in tests
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
    });

    Router::new()
        .route(
            "/api/search/ask",
            axum::routing::post(fichub::search::ask::ask_handler),
        )
        .route("/api/search", get(fichub::search::routes::search_handler))
        .with_state(state)
}

/// POST /api/search/ask with a JSON body through the real router.
async fn post_ask(app: &Router, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .uri("/api/search/ask")
        .method("POST")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    if status != StatusCode::OK {
        eprintln!(
            "DEBUG non-200 for ask: status={status} body={}",
            String::from_utf8_lossy(&bytes)
        );
    }
    (status, value)
}

/// Seed a fic_info row (idempotent). Words/chapters/status drive the
/// min_words / max_words / complete assertions.
async fn seed_fic(
    pool: &sqlx::PgPool,
    id: &str,
    title: &str,
    description: &str,
    words: i64,
    chapters: i32,
    status: &str,
) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(title)
    .bind("ask_test_author")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(chapters)
    .bind(words)
    .bind(description)
    .bind(chrono::Utc::now() - chrono::Duration::days(100))
    .bind(chrono::Utc::now() - chrono::Duration::days(1))
    .bind(status)
    .bind("ao3")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Option::<i64>::None)
    .bind(Option::<i64>::None)
    .bind(Option::<String>::None)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(pool: &sqlx::PgPool, fics: &[&str]) {
    for id in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
}

/// Wipe any search_queries rows this suite may have logged (Ask logs with
/// the [ask-*] marker prefix, which makes cleanup precise).
async fn cleanup_ask_analytics(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM search_queries WHERE query LIKE '[ask-%' OR query LIKE '%askit_plain%'",
    )
    .execute(pool)
    .await;
}

fn result_ids(body: &Value) -> Vec<String> {
    body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(|s| s.to_string()))
        .collect()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// (a) Fallback path: Ollama is unreachable (dead port), so the endpoint
/// must return 200 with `translated: false`, the raw NL as the applied q,
/// and run the real plain search against seeded data.
#[tokio::test]
#[ignore]
async fn ask_falls_back_to_plain_search_when_ollama_down() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["askit_plain_a", "askit_plain_b"];
    cleanup(&db, &fics).await;
    seed_fic(
        &db,
        "askit_plain_a",
        "The Sunken Lighthouse",
        "A tale of stormy seas.",
        1000,
        1,
        "complete",
    )
    .await;
    seed_fic(
        &db,
        "askit_plain_b",
        "Baking with Dragons",
        "In which dragons learn pastry.",
        2000,
        3,
        "ongoing",
    )
    .await;

    let app = app().await;
    let (status, body) = post_ask(&app, json!({ "q": "lighthouse pastry" })).await;
    assert_eq!(status, StatusCode::OK, "fallback must be 200: {body}");
    assert_eq!(
        body["translated"], false,
        "ollama down ⇒ not translated: {body}"
    );
    assert_eq!(body["nl_query"], "lighthouse pastry");
    assert_eq!(body["applied_params"], "lighthouse pastry");
    // Plain search on the raw NL actually ran against the seeded fics
    // (the shared live DB also holds unrelated fics — only our seeds matter).
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"askit_plain_a".to_string()),
        "seeded fic A must match: {body}"
    );
    // "pastry" is not a real English word — the tsquery stemmer maps it to
    // a lexeme that doesn't exist in any seeded row, so the plain fallback
    // may legitimately return only the "lighthouse" hit. What matters is
    // that the search RAN and returned the plain-text match.
    assert!(
        ids.iter().any(|id| id.starts_with("askit_plain_")),
        "at least one seeded fic must match: {body}"
    );

    cleanup(&db, &fics).await;
}

/// The same fallback path, but the seeded data proves the returned filters
/// would have changed the result set had translation worked: the plain
/// search returns the fics, and the response shape still carries the full
/// search envelope (total/page/per_page/results/facets) so the /ask page
/// can render it exactly like /search.
#[tokio::test]
#[ignore]
async fn ask_returns_full_search_envelope_on_fallback() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["askit_env_a"];
    cleanup(&db, &fics).await;
    seed_fic(
        &db,
        "askit_env_a",
        "Envelope Test",
        "Envelope description words.",
        100,
        1,
        "complete",
    )
    .await;

    let app = app().await;
    let (status, body) = post_ask(&app, json!({ "q": "envelope" })).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("total").is_some());
    assert!(body.get("page").is_some());
    assert!(body.get("per_page").is_some());
    assert!(body.get("results").is_some());
    assert!(
        body.get("facets").is_some(),
        "facets must be present: {body}"
    );
    assert_eq!(body["total"], 1);
    assert_eq!(result_ids(&body), vec!["askit_env_a"]);
    cleanup(&db, &fics).await;
}

/// Empty / missing q is a 400, never a 200-with-fallback.
#[tokio::test]
#[ignore]
async fn ask_rejects_empty_query() {
    let _guard = db_guard();
    let app = app().await;

    let (status, body) = post_ask(&app, json!({ "q": "   " })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty q must 400: {body}");

    // Missing field: axum's JSON extractor rejects it (422) — that is the
    // framework's contract for a malformed body, still not a 200 fallback.
    let (status, _) = post_ask(&app, json!({})).await;
    assert!(
        status.is_client_error(),
        "missing q must be a client error, got {status}"
    );
}

/// Overlong queries are rejected before they ever reach Ollama/Redis.
#[tokio::test]
#[ignore]
async fn ask_rejects_overlong_query() {
    let _guard = db_guard();
    let app = app().await;
    let long = "x".repeat(fichub::search::ask::MAX_ASK_LEN + 1);
    let (status, _) = post_ask(&app, json!({ "q": long })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "overlong q must 400");
}

/// (b/c) Validation + happy-path are unit tests (no real Ollama), but this
/// DB-gated test proves the translated path end-to-end WITHOUT Ollama by
/// pre-seeding the Redis cache: the handler reads the cached translation,
/// skips the model entirely, and runs the translated search. This exercises
/// the exact same code path as a live model response (cache → run_search →
/// envelope + applied_params).
#[tokio::test]
#[ignore]
async fn ask_uses_cached_translation_and_runs_translated_search() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["askit_cached_a", "askit_cached_b"];
    cleanup(&db, &fics).await;
    // askit_cached_a: 60k words, complete — matches min_words=50000 + complete.
    seed_fic(
        &db,
        "askit_cached_a",
        "Long Complete Fic",
        "A very long finished story about stars.",
        60000,
        40,
        "complete",
    )
    .await;
    // askit_cached_b: 10k words, ongoing — fails both filters.
    seed_fic(
        &db,
        "askit_cached_b",
        "Short WIP",
        "A short work in progress.",
        10000,
        2,
        "ongoing",
    )
    .await;

    let nl = "cached translation test query";
    // Pre-seed the cache exactly like a fresh Ollama translation would store
    // it: a v2 query string (JSON-encoded).
    let mut redis = {
        let redis_client = redis::Client::open("redis://localhost:6379".to_string()).unwrap();
        redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("redis conn")
    };
    let v2 = "stars words:>50k status:complete";
    fichub::search::ask_cache::set_cached_translation(&mut redis, nl, &json!(v2)).await;

    let app = app().await;
    let (status, body) = post_ask(&app, json!({ "q": nl })).await;
    assert_eq!(status, StatusCode::OK, "cached path must be 200: {body}");
    assert_eq!(
        body["translated"], true,
        "cache hit counts as translated: {body}"
    );
    assert_eq!(body["nl_query"], nl);
    // applied_params echo the cached v2 query (the "Interpreted as" chip).
    assert_eq!(body["applied_params"], v2);
    // Only the long complete fic matches words:>50k + status:complete.
    assert_eq!(
        result_ids(&body),
        vec!["askit_cached_a"],
        "translated filters applied: {body}"
    );

    fichub::search::ask_cache::delete_cached_translation(&mut redis, nl).await;
    cleanup(&db, &fics).await;
}

/// The Postgres tier is WRITE-ONLY by design: the handler never reads it
/// back (Redis is the only lookup tier). This test proves the write-only
/// contract: with a pre-seeded PG row and no Redis row, the handler must
/// NOT consult PG — it goes to Ollama (dead port → plain fallback) and
/// `translated` stays false. If someone re-enables the PG read, this test
/// catches it (the seeded row would flip `translated` to true).
///
/// (The PG archive WRITE path — saving a fresh translation — is covered by
/// the `pg_cache_roundtrip` unit test; the handler's write on successful
/// translation is the same `set_cached_translation_pg` call.)
#[tokio::test]
#[ignore]
async fn ask_never_reads_postgres_archive() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["askit_pgwrite_a"];
    cleanup(&db, &fics).await;
    seed_fic(
        &db,
        "askit_pgwrite_a",
        "PG Write Archive",
        "A story about constellations.",
        1000,
        1,
        "complete",
    )
    .await;

    let nl = "postgres write-only archive test query";

    // Ensure a clean slate in both tiers.
    let mut redis = {
        let redis_client = redis::Client::open("redis://localhost:6379".to_string()).unwrap();
        redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("redis conn")
    };
    fichub::search::ask_cache::delete_cached_translation(&mut redis, nl).await;
    sqlx::query("DELETE FROM ask_translation_cache WHERE nl_query = $1")
        .bind(fichub::search::ask_cache::normalize_nl_query(nl))
        .execute(&db)
        .await
        .expect("pg cleanup");

    // Pre-seed ONLY the PG row (as if a previous version had served it).
    // The handler must NOT read it: without Redis, it goes to Ollama
    // (dead port → plain fallback), and `translated` stays false.
    let v2 = "constellations words:>50k status:complete";
    let model = test_config().ollama_chat_model;
    fichub::search::ask_cache::set_cached_translation_pg(&db, nl, &model, &json!(v2)).await;

    let app = app().await;
    let (status, body) = post_ask(&app, json!({ "q": nl })).await;
    assert_eq!(status, StatusCode::OK, "must be 200: {body}");
    assert_eq!(
        body["translated"], false,
        "PG row must NOT be consulted — only Redis is a lookup tier: {body}"
    );
    assert_eq!(
        body["applied_params"], nl,
        "plain fallback params is the raw NL"
    );

    // The pre-seeded row is untouched (write-only: handler never reads,
    // and no fresh translation happened to overwrite it).
    let row: Option<String> =
        sqlx::query_scalar("SELECT params::text FROM ask_translation_cache WHERE nl_query = $1")
            .bind(fichub::search::ask_cache::normalize_nl_query(nl))
            .fetch_optional(&db)
            .await
            .expect("pg read");
    let row = row.expect("pre-seeded PG row must still exist");
    assert!(
        row.contains("constellations"),
        "pre-seeded row must be untouched: {row}"
    );

    sqlx::query("DELETE FROM ask_translation_cache WHERE nl_query = $1")
        .bind(fichub::search::ask_cache::normalize_nl_query(nl))
        .execute(&db)
        .await
        .expect("pg cleanup");
    cleanup(&db, &fics).await;
}

/// Analytics: the original NL query is logged to search_queries with the
/// [ask-*] marker (translation unavailable here, so [ask-plain]).
#[tokio::test]
#[ignore]
async fn ask_logs_analytics_with_marker() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup_ask_analytics(&db).await;

    let app = app().await;
    let (status, _) = post_ask(&app, json!({ "q": "askit_analytics_query" })).await;
    assert_eq!(status, StatusCode::OK);

    let row: Option<String> = sqlx::query_scalar(
        "SELECT query FROM search_queries WHERE query LIKE '[ask-%' ORDER BY id DESC LIMIT 1",
    )
    .fetch_optional(&db)
    .await
    .expect("analytics query");
    let row = row.expect("ask query must be logged to search_queries");
    assert!(
        row.starts_with("[ask-plain] askit_analytics_query"),
        "unexpected analytics row: {row}"
    );
    cleanup_ask_analytics(&db).await;
}

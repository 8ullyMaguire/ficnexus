//! Integration tests for the FicHub advanced search handler (`GET /api/search`).
//!
//! These exercise the FULL path: HTTP request → query-param parsing →
//! boolean-query parsing → dynamic SQL (count + data + facet queries) →
//! JSON response. They run against the real configured database
//! (`DATABASE_URL` from `.env`, database `fichub`) like the existing
//! `db_tests` module in `tests/integration.rs`, and follow the same
//! conventions:
//!
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]` so they are skipped by default and only run when
//!   the user explicitly asks for them: `cargo test --test search_api -- --include-ignored`.
//! * Each test seeds its own uniquely-named fics/tags with
//!   `INSERT ... ON CONFLICT DO NOTHING` and cleans up by `DELETE`-ing
//!   exactly the rows it created, so the suite is re-runnable and never
//!   touches pre-existing data (the live DB holds ~6 real fics and no
//!   tags; all assertions here are on the seeded rows only).
//!
//! The handler is invoked through the real Axum router with a real
//! `AppState` (tower `ServiceExt::oneshot`), exactly like the mock tests
//! in `tests/integration.rs` use `oneshot`, but with the true handler and
//! DB-backed state instead of stubs.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use serde_json::Value;
use tower::ServiceExt; // oneshot

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Global mutex that serialises ALL database-backed tests (mirrors the
/// `db_tests` module in tests/integration.rs). These tests seed rows in the
/// shared `fichub` database, so running them concurrently would be racy.
static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}

/// Acquire the DB serialisation lock, tolerating poisoning: if a previous
/// test panicked while holding the lock, the underlying data is still
/// consistent (tests clean up after themselves), so we recover the guard
/// rather than failing every subsequent test with PoisonError.
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

/// Connect to the configured test database (DATABASE_URL from .env).
/// Panics if unavailable — run these tests with `--include-ignored` only
/// when the local Postgres is up.
async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Minimal config with sane defaults for the search handler.
fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    c
}

/// Build the real router (`/api/search` → `search_handler`) with real state.
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
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route("/api/search", get(fichub::search::routes::search_handler))
        .route(
            "/api/search/suggest",
            get(fichub::search::suggest::search_suggest_handler),
        )
        .with_state(state)
}

/// Issue GET /api/search?<query> through the real router and parse the JSON body.
async fn get_search(uri: &str) -> Value {
    let app = app().await;
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    if status != StatusCode::OK {
        eprintln!(
            "DEBUG non-200 for {uri}: status={status} body={}",
            String::from_utf8_lossy(&body)
        );
    }
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 for {uri}, got {:?}",
        status
    );
    serde_json::from_slice(&body).unwrap()
}

/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps this idempotent
/// so tests are re-runnable without explicit pre-cleanup.
async fn seed_fic(
    pool: &sqlx::PgPool,
    id: &str,
    title: &str,
    description: &str,
    words: i64,
    chapters: i32,
    status: &str,
    updated_days_ago: i64,
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
    .bind("test_author")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(chapters)
    .bind(words)
    .bind(description)
    .bind(chrono::Utc::now() - chrono::Duration::days(updated_days_ago + 100))
    .bind(chrono::Utc::now() - chrono::Duration::days(updated_days_ago))
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

/// Seed a tag row; returns its id. `ON CONFLICT (name) DO NOTHING` keeps it
/// idempotent.
async fn seed_tag(pool: &sqlx::PgPool, name: &str, type_id: i16) -> i32 {
    sqlx::query(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
    )
    .bind(name)
    .bind(type_id)
    .execute(pool)
    .await
    .expect("seed_tag failed");
    sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed_tag: lookup failed")
}

/// Link a tag to a fic. `ON CONFLICT (url_id, tag_id) DO NOTHING` is
/// idempotent. `score` is well above the hidden threshold so the tag is
/// visible in results and facets.
async fn seed_fic_tag(pool: &sqlx::PgPool, url_id: &str, tag_id: i32, score: i16) {
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) VALUES ($1, $2, '127.0.0.1', $3) ON CONFLICT (url_id, tag_id) DO NOTHING",
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(score)
    .execute(pool)
    .await
    .expect("seed_fic_tag failed");
}

/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(pool: &sqlx::PgPool, fics: &[&str], tags: &[&str]) {
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
    for name in tags {
        let _ = sqlx::query("DELETE FROM tags WHERE name = $1")
            .bind(name)
            .execute(pool)
            .await;
    }
}

/// Wipe ALL test-fixture fic_info rows (any suite's) so exact-total search
/// assertions see a clean slate. The e2e DB is shared across DB-gated suites;
/// honeypot/auto-tag/requests/etc. leak fic_info rows that would otherwise
/// appear in tag-filtered / empty-shape searches.
async fn clean_all_test_fixtures(pool: &sqlx::PgPool) {
    let prefixes = [
        "searchit_",
        "honeypot",
        "auto_tag",
        "reqt-",
        "fu-refresh",
        "lists-",
        "triage-",
        "meta-test",
        "readertest",
        "bodytest",
        "curatort",
        "tagapi",
        "blindtest",
        "fs_",
        "modlog",
        "ratings",
        "bookmark",
        "credapi",
        "kindle",
        "askarch",
        "workprop",
        "transreview",
        "reporttest",
        "listtest",
        "followtest",
        "commtest",
        "feedback",
        "quickwin",
        "opds",
        "seriestest",
        "leadt",
        "content_scan",
        "rec_",
        "socialt",
    ];
    for p in prefixes {
        let _ = sqlx::query(
            "DELETE FROM fic_tags WHERE url_id IN (SELECT id FROM fic_info WHERE id LIKE $1 || '%')",
        )
        .bind(p)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "DELETE FROM comments WHERE url_id IN (SELECT id FROM fic_info WHERE id LIKE $1 || '%')",
        )
        .bind(p)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "DELETE FROM export_log WHERE url_id IN (SELECT id FROM fic_info WHERE id LIKE $1 || '%')",
        )
        .bind(p)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "DELETE FROM work_ratings WHERE url_id IN (SELECT id FROM fic_info WHERE id LIKE $1 || '%')",
        )
        .bind(p)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "DELETE FROM bookmarks WHERE url_id IN (SELECT id FROM fic_info WHERE id LIKE $1 || '%')",
        )
        .bind(p)
        .execute(pool)
        .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE $1 || '%'")
            .bind(p)
            .execute(pool)
            .await;
    }
}

/// Extract the set of result url_ids from a search response.
fn result_ids(body: &Value) -> Vec<String> {
    body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(|s| s.to_string()))
        .collect()
}

fn total(body: &Value) -> i64 {
    body["total"].as_i64().expect("total must be a number")
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// (1) Plain `q` matches title/description text via full-text search.
#[ignore]
#[tokio::test]
async fn search_plain_q_matches_title_and_description() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_plain_a", "searchit_plain_b"];
    seed_fic(
        &db,
        "searchit_plain_a",
        "The Sunken Lighthouse",
        "A tale of stormy seas.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_plain_b",
        "Baking with Dragons",
        "In which dragons learn pastry.",
        2000,
        3,
        "ongoing",
        2,
    )
    .await;

    // Title match
    let body = get_search("/api/search?q=lighthouse").await;
    assert_eq!(total(&body), 1, "title-only match: {body}");
    assert_eq!(result_ids(&body), vec!["searchit_plain_a"]);

    // Description match
    let body = get_search("/api/search?q=pastry").await;
    assert_eq!(total(&body), 1, "description-only match: {body}");
    assert_eq!(result_ids(&body), vec!["searchit_plain_b"]);

    // No match → empty
    let body = get_search("/api/search?q=zzzznomatchzzz").await;
    assert_eq!(total(&body), 0);

    // Full response shape: total/page/per_page/results/facets present
    let body = get_search("/api/search?q=lighthouse").await;
    assert!(body.get("page").is_some() && body.get("per_page").is_some());
    assert!(body.get("facets").is_some());
    assert!(body["results"][0]["title"].as_str().is_some());

    cleanup(&db, &fics, &[]).await;
}

/// (1b) Typo tolerance — a bare `q` that matches nothing via full-text
/// search falls back to pg_trgm word_similarity on title/author, so a
/// misspelled query still finds the fic. 'Hary Pottr' contains no real
/// words, so the exact tsquery matches nothing and the fuzzy fallback
/// must surface the seeded 'Harry Potter' fic. (Also covers the plain
/// title-substring path where the typo is only in one word.)
#[ignore]
#[tokio::test]
async fn search_typo_falls_back_to_fuzzy_title_author() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_fuzzy_a", "searchit_fuzzy_b"];
    seed_fic(
        &db,
        "searchit_fuzzy_a",
        "Harry Potter and the Fuzzy Test",
        "Unit testing at Hogwarts.",
        10000,
        8,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_fuzzy_b",
        "Unrelated Baking Fic",
        "In which dragons learn pastry.",
        2000,
        3,
        "ongoing",
        2,
    )
    .await;

    // Full misspelling: neither word is a real token, tsquery matches
    // nothing → fuzzy fallback finds the Harry Potter fic.
    let body = get_search("/api/search?q=Hary%20Pottr").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_fuzzy_a".to_string()),
        "full typo fallback: {body}"
    );
    assert!(
        !ids.contains(&"searchit_fuzzy_b".to_string()),
        "typo fallback must not leak unrelated fic: {body}"
    );

    // Partial typo: 'Hary Potter' — 'potter' matches the tsvector, 'hary'
    // does not, so the fuzzy OR branch still pulls the fic in.
    let body = get_search("/api/search?q=Hary%20Potter").await;
    assert!(
        result_ids(&body).contains(&"searchit_fuzzy_a".to_string()),
        "partial typo fallback: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

/// (2) Boolean AND / OR via `q`.
#[ignore]
#[tokio::test]
async fn search_boolean_and_or() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_bool_a", "searchit_bool_b"];
    seed_fic(
        &db,
        "searchit_bool_a",
        "The Dragon Rider",
        "A dragon and a rider save the realm.",
        5000,
        5,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_bool_b",
        "The Tea Shop",
        "A quiet shop with excellent tea.",
        3000,
        2,
        "ongoing",
        2,
    )
    .await;

    // Implicit AND (plainto_tsquery): both words in description
    let body = get_search("/api/search?q=dragon%20rider").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bool_a".to_string()),
        "implicit AND: {body}"
    );

    // Explicit AND (parsed tsquery path)
    let body = get_search("/api/search?q=dragon%20AND%20realm").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bool_a".to_string()),
        "explicit AND: {body}"
    );

    // OR (parsed tsquery path) — both seeded fics match
    let body = get_search("/api/search?q=dragon%20OR%20tea").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bool_a".to_string()),
        "OR matches A: {body}"
    );
    assert!(
        ids.contains(&"searchit_bool_b".to_string()),
        "OR matches B: {body}"
    );

    // AND that matches nothing
    let body = get_search("/api/search?q=dragon%20AND%20tea").await;
    assert_eq!(total(&body), 0, "AND with disjoint terms: {body}");

    cleanup(&db, &fics, &[]).await;
}

/// (3) Fielded `title:` query (also exercises the q=None fallback for
/// fully-fielded queries).
#[ignore]
#[tokio::test]
async fn search_fielded_title() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_field_a", "searchit_field_b"];
    seed_fic(
        &db,
        "searchit_field_a",
        "Harry Potter and the Test Case",
        "Unit testing at Hogwarts.",
        10000,
        8,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_field_b",
        "The Test Kitchen",
        "Cooking experiments.",
        4000,
        4,
        "ongoing",
        2,
    )
    .await;

    // title:harry → fic A (title ILIKE '%harry%') is present; other tests'
    // fics may also match (shared DB), so assert presence, not total.
    let body = get_search("/api/search?q=title%3Aharry").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_field_a".to_string()),
        "title:harry: {body}"
    );
    assert!(
        !ids.contains(&"searchit_field_b".to_string()),
        "title:harry must not match B: {body}"
    );

    // title:test matches BOTH (both titles contain "Test")
    let body = get_search("/api/search?q=title%3Atest").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_field_a".to_string()),
        "title:test A: {body}"
    );
    assert!(
        ids.contains(&"searchit_field_b".to_string()),
        "title:test B: {body}"
    );

    // description mention alone must NOT match a title: query
    let body = get_search("/api/search?q=title%3Ahogwarts").await;
    assert_eq!(total(&body), 0, "title:hogwarts (description-only): {body}");

    // Fielded-only query must not leak the raw "title:harry" text tsquery
    let body = get_search("/api/search?q=title%3Aharry").await;
    assert!(
        result_ids(&body).contains(&"searchit_field_a".to_string()),
        "regression: fielded-only q must not become text tsquery"
    );

    cleanup(&db, &fics, &[]).await;
}

/// (4) `-term` exclusion — both the tsquery path (term that only appears in
/// text) and the tag-based path (term that is a tag name).
#[ignore]
#[tokio::test]
async fn search_exclusion_tsquery_and_tag_paths() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_excl_a", "searchit_excl_b"];
    seed_fic(
        &db,
        "searchit_excl_a",
        "Midnight in the Garden",
        "A garden full of roses.",
        2000,
        2,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_excl_b",
        "Midnight in the Attic",
        "An attic full of boxes.",
        1500,
        1,
        "complete",
        2,
    )
    .await;

    // tsquery path: -roses excludes fic A by TEXT
    let body = get_search("/api/search?q=midnight%20-roses").await;
    assert!(
        result_ids(&body).contains(&"searchit_excl_b".to_string()),
        "text exclusion: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_excl_a".to_string()),
        "text exclusion leak (A has roses): {body}"
    );

    // Tag-based path: fic A gets the "Fluff" freeform tag; -fluff must
    // exclude it via tag matching, not text. The tag name must EXACTLY match
    // the exclusion term ("fluff") for the tag lookup to resolve it.
    let tag_fluff = seed_tag(&db, "Fluff", 4).await;
    seed_fic_tag(&db, "searchit_excl_a", tag_fluff, 5).await;

    let body = get_search("/api/search?q=midnight%20-fluff").await;
    assert!(
        result_ids(&body).contains(&"searchit_excl_b".to_string()),
        "tag-based exclusion: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_excl_a".to_string()),
        "fic A has tag Fluff, must be excluded: {body}"
    );

    // And the tag-based exclusion works even when the tag name never
    // appears in the text (proves it's the tag path).
    let body = get_search("/api/search?q=midnight%20-roses%20-fluff").await;
    assert!(
        result_ids(&body).contains(&"searchit_excl_b".to_string()),
        "combined exclusion: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_excl_a".to_string()),
        "combined exclusion leak: {body}"
    );

    cleanup(&db, &fics, &["Fluff"]).await;
}

/// (5) include_tags / exclude_tags / include_any_tags in `typeid:name` format.
#[ignore]
#[tokio::test]
async fn search_tag_filters() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_tags_a", "searchit_tags_b"];
    seed_fic(
        &db,
        "searchit_tags_a",
        "Gryffindor Rising",
        "Lions and bravery.",
        3000,
        3,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_tags_b",
        "Slytherin Schemes",
        "Snakes and cunning.",
        2500,
        2,
        "ongoing",
        2,
    )
    .await;

    let tag_gryf = seed_tag(&db, "SearchTest Gryffindor", 1).await; // fandom
    let tag_slyth = seed_tag(&db, "SearchTest Slytherin", 1).await;
    let tag_angst = seed_tag(&db, "SearchTest Angst", 4).await; // freeform
    seed_fic_tag(&db, "searchit_tags_a", tag_gryf, 5).await;
    seed_fic_tag(&db, "searchit_tags_a", tag_angst, 5).await;
    seed_fic_tag(&db, "searchit_tags_b", tag_slyth, 5).await;

    // include_tags — ALL must be present: fandom Gryffindor → only fic A
    // Note: the shared fichub DB may contain real fics, so we assert on the
    // seeded result ids being present/absent rather than absolute totals.
    let body = get_search("/api/search?include_tags=1%3ASearchTest%20Gryffindor").await;
    assert!(
        result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "include_tags fandom: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "include_tags fandom leak: {body}"
    );

    // include_tags with TWO tags (AND): Gryffindor + Angst → still only fic A
    let body =
        get_search("/api/search?include_tags=1%3ASearchTest%20Gryffindor%2C4%3ASearchTest%20Angst")
            .await;
    assert!(
        result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "include_tags AND: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "include_tags AND leak: {body}"
    );

    // include_tags with two tags where fic B lacks one → only fic A
    let body = get_search(
        "/api/search?include_tags=1%3ASearchTest%20Gryffindor%2C1%3ASearchTest%20Slytherin",
    )
    .await;
    assert!(
        !result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "include_tags disjoint leak: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "include_tags disjoint leak: {body}"
    );

    // exclude_tags — NOT Slytherin → fic A present, fic B absent
    let body = get_search("/api/search?exclude_tags=1%3ASearchTest%20Slytherin").await;
    assert!(
        result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "exclude_tags: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "exclude_tags leak: {body}"
    );

    // include_any_tags — Gryffindor OR Slytherin → both seeded fics
    let body = get_search(
        "/api/search?include_any_tags=1%3ASearchTest%20Gryffindor%2C1%3ASearchTest%20Slytherin",
    )
    .await;
    assert!(
        result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "include_any_tags OR: {body}"
    );
    assert!(
        result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "include_any_tags OR: {body}"
    );

    // include_any_tags with a single tag that only fic A has
    let body = get_search("/api/search?include_any_tags=4%3ASearchTest%20Angst").await;
    assert!(
        result_ids(&body).contains(&"searchit_tags_a".to_string()),
        "include_any_tags single: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_tags_b".to_string()),
        "include_any_tags single leak: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Gryffindor",
            "SearchTest Slytherin",
            "SearchTest Angst",
        ],
    )
    .await;
}

/// (6) Facet counts reflect the ACTIVE filters: seed two fics in different
/// fandoms, filter to one, assert facets.fandoms has ONLY the matching
/// fandom with the correct count.
#[ignore]
#[tokio::test]
async fn search_facets_reflect_active_filters() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_facet_a", "searchit_facet_b"];
    seed_fic(
        &db,
        "searchit_facet_a",
        "Fandom A Story",
        "Adventures in fandom A, lionheart.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_facet_b",
        "Fandom B Story",
        "Adventures in fandom B.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let tag_fa = seed_tag(&db, "SearchTest Fandom Alpha", 1).await;
    let tag_fb = seed_tag(&db, "SearchTest Fandom Beta", 1).await;
    seed_fic_tag(&db, "searchit_facet_a", tag_fa, 5).await;
    seed_fic_tag(&db, "searchit_facet_b", tag_fb, 5).await;

    // Unfiltered: the seeded fandoms appear as facets.
    let body = get_search("/api/search?q=fandom").await;
    let fandoms = body["facets"]["fandoms"].as_array().unwrap();
    let names: Vec<(&str, i64)> = fandoms
        .iter()
        .map(|f| (f["name"].as_str().unwrap(), f["count"].as_i64().unwrap()))
        .collect();
    assert!(
        names.contains(&("SearchTest Fandom Alpha", 1)),
        "alpha facet: {body}"
    );
    assert!(
        names.contains(&("SearchTest Fandom Beta", 1)),
        "beta facet: {body}"
    );

    // Filter to fandom A via a term unique to fic A's text ("lionheart")
    // → facets must shrink to only fandom A.
    let body = get_search("/api/search?q=lionheart").await;
    let fandoms = body["facets"]["fandoms"].as_array().unwrap();
    assert!(
        fandoms
            .iter()
            .any(|f| f["name"] == "SearchTest Fandom Alpha"),
        "filtered facets: {body}"
    );
    assert!(
        !fandoms
            .iter()
            .any(|f| f["name"] == "SearchTest Fandom Beta"),
        "filtered facets leak: {body}"
    );

    // Filter to fandom A via include_tags → facets reflect the active filter.
    let body = get_search("/api/search?include_tags=1%3ASearchTest%20Fandom%20Alpha").await;
    let fandoms = body["facets"]["fandoms"].as_array().unwrap();
    assert!(
        fandoms
            .iter()
            .any(|f| f["name"] == "SearchTest Fandom Alpha"),
        "include_tags-filtered facets: {body}"
    );
    assert!(
        !fandoms
            .iter()
            .any(|f| f["name"] == "SearchTest Fandom Beta"),
        "include_tags-filtered facets leak: {body}"
    );
    assert!(
        result_ids(&body).contains(&"searchit_facet_a".to_string()),
        "include_tags filter result: {body}"
    );

    cleanup(
        &db,
        &fics,
        &["SearchTest Fandom Alpha", "SearchTest Fandom Beta"],
    )
    .await;
}

/// (7) relationship_characters — matches relationship tags (type 3) whose
/// name contains the character, adds them as include_any_tags.
#[ignore]
#[tokio::test]
async fn search_relationship_characters() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_rel_a", "searchit_rel_b"];
    seed_fic(
        &db,
        "searchit_rel_a",
        "The Ship Fic",
        "A relationship story.",
        5000,
        5,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_rel_b",
        "The Other Fic",
        "No romance here.",
        1000,
        1,
        "ongoing",
        2,
    )
    .await;

    let tag_rel = seed_tag(&db, "SearchTest Draco Malfoy/Hermione Granger", 3).await;
    seed_fic_tag(&db, "searchit_rel_a", tag_rel, 5).await;

    let body = get_search("/api/search?relationship_characters=Hermione%20Granger").await;
    assert!(
        result_ids(&body).contains(&"searchit_rel_a".to_string()),
        "relationship_characters: {body}"
    );
    assert!(
        !result_ids(&body).contains(&"searchit_rel_b".to_string()),
        "relationship_characters leak: {body}"
    );

    // A character with no matching relationship tags → the filter adds no
    // constraint (empty include_any), so the seeded fic is still present.
    let body = get_search("/api/search?relationship_characters=NoSuchChar").await;
    assert!(
        result_ids(&body).contains(&"searchit_rel_a".to_string()),
        "no matching relationship is unfiltered: {body}"
    );

    cleanup(&db, &fics, &["SearchTest Draco Malfoy/Hermione Granger"]).await;
}

/// (8) Pagination and sort.
#[ignore]
#[tokio::test]
async fn search_pagination_and_sort() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_page_a", "searchit_page_b", "searchit_page_c"];
    // fic_updated: a=now-1d (newest), b=now-2d, c=now-3d (oldest)
    seed_fic(
        &db,
        "searchit_page_a",
        "Page Story Alpha",
        "Shared topic quux.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_page_b",
        "Page Story Beta",
        "Shared topic quux.",
        9000,
        9,
        "ongoing",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_page_c",
        "Page Story Gamma",
        "Shared topic quux.",
        500,
        3,
        "complete",
        3,
    )
    .await;

    // Default sort (-date): newest first → a, b, c
    let body = get_search("/api/search?q=quux").await;
    assert_eq!(total(&body), 3);
    assert_eq!(
        result_ids(&body),
        vec!["searchit_page_a", "searchit_page_b", "searchit_page_c"]
    );

    // -words: descending → b (9000), a (1000), c (500)
    let body = get_search("/api/search?q=quux&sort=-words").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_page_b", "searchit_page_a", "searchit_page_c"]
    );

    // -chapters: descending → b (9), c (3), a (1)
    let body = get_search("/api/search?q=quux&sort=-chapters").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_page_b", "searchit_page_c", "searchit_page_a"]
    );

    // -title: ascending → a, b, c
    let body = get_search("/api/search?q=quux&sort=-title").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_page_a", "searchit_page_b", "searchit_page_c"]
    );

    // Pagination: per_page=1, page=2 → second item under -date sort (b)
    let body = get_search("/api/search?q=quux&per_page=1&page=2").await;
    assert_eq!(total(&body), 3, "total is unaffected by pagination");
    assert_eq!(body["page"], 2);
    assert_eq!(body["per_page"], 1);
    assert_eq!(result_ids(&body), vec!["searchit_page_b"]);

    // per_page clamping: per_page=999 → capped at search_max_per_page (50)
    let body = get_search("/api/search?q=quux&per_page=999").await;
    assert_eq!(body["per_page"], 50, "per_page must be capped: {body}");
    assert_eq!(result_ids(&body).len(), 3);

    // page=0 is clamped to 1
    let body = get_search("/api/search?q=quux&page=0").await;
    assert_eq!(body["page"], 1);

    cleanup(&db, &fics, &[]).await;
}

/// The handler must return a clean empty result set (not an error) when
/// nothing matches and no filters are set.
#[ignore]
#[tokio::test]
async fn search_empty_database_shape() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    // Uniquely-named query that cannot collide with seeded or real data.
    let body = get_search("/api/search?q=searchit_zzz_no_such_fic").await;
    assert_eq!(total(&body), 0);
    assert_eq!(body["page"], 1);
    assert!(body["results"].as_array().unwrap().is_empty());
    // Facets are all present and empty.
    for key in [
        "fandoms",
        "characters",
        "relationships",
        "warnings",
        "categories",
        "freeforms",
        "statuses",
    ] {
        assert!(
            body["facets"][key].is_array(),
            "facets.{key} missing: {body}"
        );
    }
}

// ─── GET /api/search/suggest ─────────────────────────────────────────────────

/// Seed a user row (idempotent); returns the user id.
async fn seed_suggest_user(pool: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("seed_suggest_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_suggest_user: lookup failed")
}

/// Seed a bookmark (idempotent).
async fn seed_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str) {
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(url_id)
    .execute(pool)
    .await
    .expect("seed_bookmark failed");
}

/// JWT Bearer header for the given user id (AuthUser reads JWT_SECRET from env).
fn suggest_auth_header(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "suggest_test_user".into(),
        trust_level: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Issue GET /api/search/suggest through the real router. `auth` is an
/// optional `Authorization` header value.
async fn get_suggest_auth(uri: &str, auth: Option<&str>) -> Value {
    let app = app().await;
    let mut builder = Request::builder().uri(uri);
    if let Some(a) = auth {
        builder = builder.header("Authorization", a);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 for {uri}, got {:?}",
        response.status()
    );
    serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap()
}

/// Names of the suggestions in response order.
fn suggest_names(body: &Value) -> Vec<String> {
    body["suggestions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s["name"].as_str().map(|n| n.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Seed helper: one fic with the given tags attached.
#[allow(dead_code)]
async fn seed_suggest_fic(
    pool: &sqlx::PgPool,
    url_id: &str,
    tag_names: &[(&str, i16)],
) -> Vec<i32> {
    seed_fic(
        pool,
        url_id,
        &format!("Suggest {url_id}"),
        "Suggestions test fic.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    let mut ids = Vec::new();
    for (name, type_id) in tag_names {
        let id = seed_tag(pool, name, *type_id).await;
        seed_fic_tag(pool, url_id, id, 5).await;
        ids.push(id);
    }
    ids
}

/// Popular suggestions: seeded tags ranked by usage count, `reason:
/// "popular"`, prefix and tag_type_id filters, and the response shape.
#[ignore]
#[tokio::test]
async fn suggest_popular_ranking_prefix_and_type_filter() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = [
        "suggestit_pop_a",
        "suggestit_pop_b",
        "suggestit_pop_c",
        "suggestit_pop_d",
    ];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Suggest {f}"),
            "Suggestions test fic.",
            1000,
            1,
            "complete",
            1,
        )
        .await;
    }
    // Prefix `SuggestItZZ` sorts after all earlier `SuggestIt*` seeds so the
    // seeded tags own the top-usage slots among themselves. The Adventure
    // tag gets usage 4 — higher than any tag any other suggest test seeds
    // (max 3) — so `names[0]` is deterministic regardless of test order.
    let tag_adv = seed_tag(&db, "SuggestItZZ Adventure", 4).await;
    let tag_angst = seed_tag(&db, "SuggestItZZ Angst", 4).await;
    let tag_fluff = seed_tag(&db, "SuggestItZZ Fluff", 4).await;
    let tag_fandom = seed_tag(&db, "SuggestItZZ Fandom", 1).await;
    // usage: Adventure=4 (all fics), Angst=1 (b), Fluff=1 (c), Fandom=1 (a)
    for f in fics {
        seed_fic_tag(&db, f, tag_adv, 5).await;
    }
    seed_fic_tag(&db, "suggestit_pop_a", tag_fandom, 5).await;
    seed_fic_tag(&db, "suggestit_pop_b", tag_angst, 5).await;
    seed_fic_tag(&db, "suggestit_pop_c", tag_fluff, 5).await;

    // Unfiltered: highest usage first; ties broken by name.
    let body = get_suggest_auth("/api/search/suggest", None).await;
    assert_eq!(body["err"], 0, "{body}");
    let names = suggest_names(&body);
    assert_eq!(
        names[0], "SuggestItZZ Adventure",
        "usage 2 must rank first: {body}"
    );
    assert!(names.contains(&"SuggestItZZ Angst".to_string()), "{body}");
    assert!(names.contains(&"SuggestItZZ Fluff".to_string()), "{body}");
    assert!(names.contains(&"SuggestItZZ Fandom".to_string()), "{body}");
    // Every suggestion has the full shape + popular reason.
    for s in body["suggestions"].as_array().unwrap() {
        assert!(s["id"].as_i64().is_some(), "{body}");
        assert!(s["usage_count"].as_i64().is_some(), "{body}");
        assert_eq!(s["reason"], "popular", "{body}");
    }

    // tag_type_id filter → only freeforms.
    let body = get_suggest_auth("/api/search/suggest?tag_type_id=4", None).await;
    let names = suggest_names(&body);
    assert!(!names.contains(&"SuggestItZZ Fandom".to_string()), "{body}");
    assert!(
        names.contains(&"SuggestItZZ Adventure".to_string()),
        "{body}"
    );

    // Prefix filter (case-insensitive ILIKE prefix).
    let body = get_suggest_auth("/api/search/suggest?q=suggestitzz%20a", None).await;
    let names = suggest_names(&body);
    assert!(
        names.contains(&"SuggestItZZ Adventure".to_string()),
        "{body}"
    );
    assert!(names.contains(&"SuggestItZZ Angst".to_string()), "{body}");
    assert!(!names.contains(&"SuggestItZZ Fluff".to_string()), "{body}");

    cleanup(
        &db,
        &fics,
        &[
            "SuggestItZZ Adventure",
            "SuggestItZZ Angst",
            "SuggestItZZ Fluff",
            "SuggestItZZ Fandom",
        ],
    )
    .await;
}

/// Personal=1 with a signed-in user: tags matching the user's bookmarks are
/// marked `reason: "for_you"`; anonymous personal=1 behaves like popular.
#[ignore]
#[tokio::test]
async fn suggest_personal_marks_for_you_for_authenticated_user() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "suggestit_user";
    let user_id = seed_suggest_user(&db, user).await;
    let fics = ["suggestit_per_a", "suggestit_per_b", "suggestit_per_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Suggest {f}"),
            "Suggestions test fic.",
            1000,
            1,
            "complete",
            1,
        )
        .await;
    }
    // Two popular tags; the user bookmarks ONLY fic B, which carries ONLY
    // the Angst tag → Angst is the user's top tag. Adventure is attached to
    // fic A + fic C only (never bookmarked) so it is NOT in the user's top
    // tag set and must stay `popular` — this works regardless of which
    // suggest test ran first.
    let tag_adv = seed_tag(&db, "SuggestItZZ Per Adventure", 4).await;
    let tag_angst = seed_tag(&db, "SuggestItZZ Per Angst", 4).await;
    seed_fic_tag(&db, "suggestit_per_a", tag_adv, 5).await;
    seed_fic_tag(&db, "suggestit_per_b", tag_angst, 5).await;
    seed_fic_tag(&db, "suggestit_per_c", tag_adv, 5).await;
    seed_bookmark(&db, user_id, "suggestit_per_b").await;

    let auth = suggest_auth_header(user_id);
    let body = get_suggest_auth("/api/search/suggest?personal=1", Some(&auth)).await;
    assert_eq!(body["err"], 0, "{body}");
    let suggestions = body["suggestions"].as_array().unwrap();
    let angst = suggestions
        .iter()
        .find(|s| s["name"] == "SuggestItZZ Per Angst")
        .expect("Angst must be suggested");
    assert_eq!(
        angst["reason"], "for_you",
        "bookmarked tag must be for_you: {body}"
    );
    let adv = suggestions
        .iter()
        .find(|s| s["name"] == "SuggestItZZ Per Adventure")
        .expect("Adventure must be suggested");
    assert_eq!(
        adv["reason"], "popular",
        "unbookmarked tag stays popular: {body}"
    );
    // for_you comes first (re-ranked by overlap).
    assert_eq!(
        suggestions[0]["reason"], "for_you",
        "the user's tag must be re-ranked first: {body}"
    );

    // Anonymous personal=1 → plain popular list, no for_you anywhere.
    let body = get_suggest_auth("/api/search/suggest?personal=1", None).await;
    assert_eq!(body["err"], 0, "{body}");
    for s in body["suggestions"].as_array().unwrap() {
        assert_eq!(
            s["reason"], "popular",
            "anonymous must never get for_you: {body}"
        );
    }

    cleanup(
        &db,
        &fics,
        &["SuggestItZZ Per Adventure", "SuggestItZZ Per Angst"],
    )
    .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(user)
        .execute(&db)
        .await;
}

/// main_char_attr — "attribute applies to the MAIN character" search.
/// A fic where Harry is the main character (highest-scored char tag) with the
/// "Dark Harry Potter" freeform must match; a fic where Harry is SECONDARY
/// (lower score) with the Dark tag must NOT match; a fic with the Dark tag but
/// no Harry char tag must NOT match.
#[ignore]
#[tokio::test]
async fn search_main_char_attr_finds_attribute_on_main_character() {
    let _guard = db_guard();
    let db = pool().await;

    let fics = [
        "searchit_mca_main",
        "searchit_mca_side",
        "searchit_mca_nochar",
    ];
    let tags = [
        "SearchTest Harry Potter",
        "SearchTest Ron Weasley",
        "SearchTest Dark Harry Potter",
    ];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("MCA {f}"),
            "Main character attribute test.",
            10000,
            10,
            "complete",
            1,
        )
        .await;
    }
    // Fic A: Harry is MAIN (score 10) + Dark tag -> should match
    let harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    let ron = seed_tag(&db, "SearchTest Ron Weasley", 2).await;
    let dark = seed_tag(&db, "SearchTest Dark Harry Potter", 4).await;
    seed_fic_tag(&db, "searchit_mca_main", harry, 10).await; // Harry = main
    seed_fic_tag(&db, "searchit_mca_main", dark, 5).await;
    // Fic B: Harry is SECONDARY (score 1, Ron is main at 10) + Dark tag -> must NOT match
    seed_fic_tag(&db, "searchit_mca_side", ron, 10).await; // Ron = main
    seed_fic_tag(&db, "searchit_mca_side", harry, 1).await; // Harry = secondary
    seed_fic_tag(&db, "searchit_mca_side", dark, 5).await;
    // Fic C: Dark tag but no Harry char tag -> must NOT match
    seed_fic_tag(&db, "searchit_mca_nochar", dark, 5).await;

    // Search: main character = Harry Potter, attribute = Dark Harry Potter
    let uri = "/api/search?main_char_attr=SearchTest%20Harry%20Potter%7CSearchTest%20Dark%20Harry%20Potter";
    let body = get_search(uri).await;
    let ids: Vec<&str> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["url_id"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&"searchit_mca_main"),
        "main-character fic should match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mca_side"),
        "fic where Harry is secondary must NOT match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mca_nochar"),
        "fic without the Harry char tag must NOT match: {body}"
    );

    cleanup(&db, &fics, &tags).await;
}

/// main_char_attr generalizes to ANY character — the attribute must apply to
/// THAT fic's main character, whoever it is. Character A as main with
/// "Attribute A" matches; Character A as secondary (Character B main) with
/// the same attribute does NOT. Uses GENERIC character names to prove the
/// mechanism applies to ANY main character, not fandom-specific ones.
#[ignore]
#[tokio::test]
async fn search_main_char_attr_any_character() {
    let _guard = db_guard();
    let db = pool().await;

    let fics = ["searchit_mca_a_main", "searchit_mca_a_side"];
    let tags = [
        "SearchTest Character A",
        "SearchTest Character B",
        "SearchTest Attribute A",
    ];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("MCA {f}"),
            "Main character attribute any-char test.",
            10000,
            10,
            "complete",
            1,
        )
        .await;
    }
    let char_a = seed_tag(&db, "SearchTest Character A", 2).await;
    let char_b = seed_tag(&db, "SearchTest Character B", 2).await;
    let attr_a = seed_tag(&db, "SearchTest Attribute A", 4).await;
    // Fic A: Character A is MAIN (score 10) + "Attribute A" -> should match
    seed_fic_tag(&db, "searchit_mca_a_main", char_a, 10).await;
    seed_fic_tag(&db, "searchit_mca_a_main", attr_a, 5).await;
    // Fic B: Character B is MAIN (10), Character A secondary (1) + same
    // "Attribute A" tag -> must NOT match (the attribute applies to a
    // different character's fic, not Character A as main)
    seed_fic_tag(&db, "searchit_mca_a_side", char_b, 10).await;
    seed_fic_tag(&db, "searchit_mca_a_side", char_a, 1).await;
    seed_fic_tag(&db, "searchit_mca_a_side", attr_a, 5).await;

    let uri = "/api/search?main_char_attr=SearchTest%20Character%20A%7CSearchTest%20Attribute%20A";
    let body = get_search(uri).await;
    let ids: Vec<&str> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["url_id"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&"searchit_mca_a_main"),
        "Character-A-as-main fic should match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mca_a_side"),
        "Character-A-as-secondary fic must NOT match: {body}"
    );

    cleanup(&db, &fics, &tags).await;
}

// ─── Advanced-search niche cases (AO3-style) ─────────────────────────────────
//
// These cover the "AO3-style" corner cases of the advanced search handler:
// relationship_characters, main-character (tag-type-2) search, cross-type AND,
// include_any OR, exclusions, primary_tag, fielded title/author queries,
// boolean q, quoted phrases, numeric bounds, source, empty (no-filter) search,
// sort, pagination, facets, case-insensitive tag lookup, special-character tag
// names, tag_ids, and no_warnings.
//
// Every test seeds its own uniquely-named fics/tags (prefix `searchit_` /
// `SearchTest`) and deletes exactly those rows afterwards, so results only ever
// depend on the seeded rows and the suite is re-runnable.

/// (9) relationship_characters — "any relationship containing a specific
/// character": all relationship tags (type 3) whose name contains the
/// character are ANDed as include_any (OR). Fics A+B (Harry in a
/// relationship tag) match; fic C (Ron/Hermione only) does NOT.
#[ignore]
#[tokio::test]
async fn search_relationship_characters_finds_any_relationship_with_character() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_relc_a", "searchit_relc_b", "searchit_relc_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("RelChar {f}"),
            "Relationship character test.",
            1000,
            1,
            "complete",
            1,
        )
        .await;
    }
    let hp_gw = seed_tag(&db, "SearchTest Harry Potter/Ginny Weasley", 3).await;
    let dm_hp = seed_tag(&db, "SearchTest Draco Malfoy/Harry Potter", 3).await;
    let rw_hm = seed_tag(&db, "SearchTest Ron Weasley/Hermione Granger", 3).await;
    seed_fic_tag(&db, "searchit_relc_a", hp_gw, 5).await;
    seed_fic_tag(&db, "searchit_relc_b", dm_hp, 5).await;
    seed_fic_tag(&db, "searchit_relc_c", rw_hm, 5).await;

    let body = get_search("/api/search?relationship_characters=Harry%20Potter").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_relc_a".to_string()),
        "A (Harry/Ginny) must match: {body}"
    );
    assert!(
        ids.contains(&"searchit_relc_b".to_string()),
        "B (Draco/Harry) must match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_relc_c".to_string()),
        "C (Ron/Hermione) must NOT match: {body}"
    );

    // Multiple characters are OR-ed together.
    let body =
        get_search("/api/search?relationship_characters=Ron%20Weasley%2CHermione%20Granger").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_relc_c".to_string()),
        "C has Ron/Hermione: {body}"
    );
    assert!(
        !ids.contains(&"searchit_relc_a".to_string()),
        "A must not match: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Harry Potter/Ginny Weasley",
            "SearchTest Draco Malfoy/Harry Potter",
            "SearchTest Ron Weasley/Hermione Granger",
        ],
    )
    .await;
}

/// (10) Main-character search: `include_tags=2:Harry Potter` matches fics
/// where Harry is a CHARACTER tag (type 2). A fic where "Harry Potter" only
/// appears inside a RELATIONSHIP tag (type 3) must NOT match — the type_id
/// is part of the filter.
#[ignore]
#[tokio::test]
async fn search_main_character_tag_type2_not_relationship() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_mc_a", "searchit_mc_b"];
    seed_fic(
        &db,
        "searchit_mc_a",
        "Main Char A",
        "Harry as a character tag.",
        2000,
        2,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_mc_b",
        "Main Char B",
        "Harry only in a relationship tag.",
        3000,
        3,
        "ongoing",
        2,
    )
    .await;

    let harry_char = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    let harry_rel = seed_tag(&db, "SearchTest Harry Potter/Ginny Weasley", 3).await;
    seed_fic_tag(&db, "searchit_mc_a", harry_char, 5).await;
    seed_fic_tag(&db, "searchit_mc_b", harry_rel, 5).await;

    let body = get_search("/api/search?include_tags=2%3ASearchTest%20Harry%20Potter").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_mc_a".to_string()),
        "fic A has Harry as character tag: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mc_b".to_string()),
        "fic B only has Harry in a relationship tag: {body}"
    );

    // The same name as a RELATIONSHIP filter (type 3) matches only fic B.
    let body =
        get_search("/api/search?include_tags=3%3ASearchTest%20Harry%20Potter%2FGinny%20Weasley")
            .await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_mc_b".to_string()),
        "fic B has the relationship tag: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mc_a".to_string()),
        "fic A lacks the relationship tag: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Harry Potter",
            "SearchTest Harry Potter/Ginny Weasley",
        ],
    )
    .await;
}

/// (11) main_char_attr with a character that is NOT the main character in any
/// fic (and the attribute is absent) returns no seeded fics.
#[ignore]
#[tokio::test]
async fn search_main_char_attr_unknown_main_character_empty() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_mcx_a", "searchit_mcx_b"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("MCX {f}"),
            "Main char attr negative test.",
            5000,
            5,
            "complete",
            1,
        )
        .await;
    }
    let hermione = seed_tag(&db, "SearchTest Hermione Granger", 2).await;
    let dark = seed_tag(&db, "SearchTest Dark Hermione Granger", 4).await;
    // Harry is main in fic A; Hermione is main in fic B, but the Dark tag is
    // only on fic A (where Hermione is NOT even present as a character).
    let harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    seed_fic_tag(&db, "searchit_mcx_a", harry, 10).await;
    seed_fic_tag(&db, "searchit_mcx_a", dark, 5).await;
    seed_fic_tag(&db, "searchit_mcx_b", hermione, 10).await;

    let uri = "/api/search?main_char_attr=SearchTest%20Hermione%20Granger%7CSearchTest%20Dark%20Hermione%20Granger";
    let body = get_search(uri).await;
    assert!(
        result_ids(&body).is_empty(),
        "Hermione is main in no fic carrying the Dark tag: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Hermione Granger",
            "SearchTest Dark Hermione Granger",
            "SearchTest Harry Potter",
        ],
    )
    .await;
}

/// (12) Cross-type tag AND: fandom AND character — both must be present.
/// A fic with only the fandom or only the character does not match.
#[ignore]
#[tokio::test]
async fn search_cross_type_include_tags_and() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_cross_a", "searchit_cross_b", "searchit_cross_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Cross {f}"),
            "Cross-type AND test.",
            1500,
            2,
            "complete",
            1,
        )
        .await;
    }
    let fandom = seed_tag(&db, "SearchTest Fandom HP", 1).await;
    let char_harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    seed_fic_tag(&db, "searchit_cross_a", fandom, 5).await;
    seed_fic_tag(&db, "searchit_cross_a", char_harry, 5).await;
    seed_fic_tag(&db, "searchit_cross_b", fandom, 5).await; // fandom only
    seed_fic_tag(&db, "searchit_cross_c", char_harry, 5).await; // character only

    let uri =
        "/api/search?include_tags=1%3ASearchTest%20Fandom%20HP%2C2%3ASearchTest%20Harry%20Potter";
    let body = get_search(uri).await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_cross_a".to_string()),
        "fic A has both: {body}"
    );
    assert!(
        !ids.contains(&"searchit_cross_b".to_string()),
        "fic B lacks the character: {body}"
    );
    assert!(
        !ids.contains(&"searchit_cross_c".to_string()),
        "fic C lacks the fandom: {body}"
    );

    cleanup(
        &db,
        &fics,
        &["SearchTest Fandom HP", "SearchTest Harry Potter"],
    )
    .await;
}

/// (13) include_any_tags — OR semantics: fic with EITHER tag matches.
#[ignore]
#[tokio::test]
async fn search_include_any_tags_or_semantics() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_any_a", "searchit_any_b", "searchit_any_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Any {f}"),
            "include_any OR test.",
            1200,
            1,
            "ongoing",
            1,
        )
        .await;
    }
    let harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    let ron = seed_tag(&db, "SearchTest Ron Weasley", 2).await;
    seed_fic_tag(&db, "searchit_any_a", harry, 5).await;
    seed_fic_tag(&db, "searchit_any_b", ron, 5).await;
    seed_fic_tag(&db, "searchit_any_c", harry, 5).await;
    seed_fic_tag(&db, "searchit_any_c", ron, 5).await;

    let uri = "/api/search?include_any_tags=2%3ASearchTest%20Harry%20Potter%2C2%3ASearchTest%20Ron%20Weasley";
    let body = get_search(uri).await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_any_a".to_string()),
        "A has Harry: {body}"
    );
    assert!(
        ids.contains(&"searchit_any_b".to_string()),
        "B has Ron: {body}"
    );
    assert!(
        ids.contains(&"searchit_any_c".to_string()),
        "C has both: {body}"
    );

    cleanup(
        &db,
        &fics,
        &["SearchTest Harry Potter", "SearchTest Ron Weasley"],
    )
    .await;
}

/// (14) exclude_tags — exclude a specific character.
#[ignore]
#[tokio::test]
async fn search_exclude_tags_character() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_exc_a", "searchit_exc_b"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Exc {f}"),
            "Exclude character test.",
            800,
            1,
            "complete",
            1,
        )
        .await;
    }
    let harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    let ron = seed_tag(&db, "SearchTest Ron Weasley", 2).await;
    seed_fic_tag(&db, "searchit_exc_a", harry, 5).await;
    seed_fic_tag(&db, "searchit_exc_b", ron, 5).await;

    let uri = "/api/search?exclude_tags=2%3ASearchTest%20Harry%20Potter";
    let body = get_search(uri).await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_exc_a".to_string()),
        "fic A has Harry, must be excluded: {body}"
    );
    assert!(
        ids.contains(&"searchit_exc_b".to_string()),
        "fic B (Ron only) stays: {body}"
    );

    cleanup(
        &db,
        &fics,
        &["SearchTest Harry Potter", "SearchTest Ron Weasley"],
    )
    .await;
}

/// (15) primary_tag — fic must have the tag as its HIGHEST-scored fic_tag
/// (seed_fic_tag's score controls this; the subquery orders by score DESC).
#[ignore]
#[tokio::test]
async fn search_primary_tag_requires_highest_score() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_prim_a", "searchit_prim_b"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("Prim {f}"),
            "Primary tag test.",
            2000,
            2,
            "complete",
            1,
        )
        .await;
    }
    let fandom_adv = seed_tag(&db, "SearchTest Adventure", 1).await;
    let fandom_fan = seed_tag(&db, "SearchTest Fantasy", 1).await;
    // A: Adventure is primary (10), Fantasy secondary (1)
    seed_fic_tag(&db, "searchit_prim_a", fandom_adv, 10).await;
    seed_fic_tag(&db, "searchit_prim_a", fandom_fan, 1).await;
    // B: Fantasy is primary (10), Adventure secondary (1)
    seed_fic_tag(&db, "searchit_prim_b", fandom_fan, 10).await;
    seed_fic_tag(&db, "searchit_prim_b", fandom_adv, 1).await;

    let body = get_search("/api/search?primary_tag=1%3ASearchTest%20Adventure").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_prim_a".to_string()),
        "A has Adventure as primary: {body}"
    );
    assert!(
        !ids.contains(&"searchit_prim_b".to_string()),
        "B has Adventure only as secondary: {body}"
    );

    cleanup(&db, &fics, &["SearchTest Adventure", "SearchTest Fantasy"]).await;
}

/// (16) Fielded title:"exact phrase" and author:partial via `q`.
#[ignore]
#[tokio::test]
async fn search_fielded_title_phrase_and_author() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_fld_a", "searchit_fld_b"];
    seed_fic(
        &db,
        "searchit_fld_a",
        "The Slow Burn of Stars",
        "A slow burn romance.",
        4000,
        4,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_fld_b",
        "Instant Coffee",
        "No burn here.",
        1000,
        1,
        "ongoing",
        2,
    )
    .await;
    // Give fic A a distinct author so author:partial distinguishes the two.
    sqlx::query("UPDATE fic_info SET author = $2 WHERE id = $1")
        .bind("searchit_fld_a")
        .bind("test_author_aurora")
        .execute(&db)
        .await
        .expect("update author failed");

    // title:"exact phrase" — the whole quoted phrase must appear in the title.
    let body = get_search("/api/search?q=title%3A%22slow%20burn%22").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_fld_a".to_string()),
        "title phrase match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_fld_b".to_string()),
        "title phrase leak: {body}"
    );

    // author:partial — ILIKE '%aurora%' (case-insensitive substring).
    let body = get_search("/api/search?q=author%3Aaurora").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_fld_a".to_string()),
        "author match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_fld_b".to_string()),
        "author leak: {body}"
    );

    // author:partial with a value that matches nobody.
    let body = get_search("/api/search?q=author%3Azzz_no_author").await;
    assert_eq!(total(&body), 0, "no author match: {body}");

    cleanup(&db, &fics, &[]).await;
}

/// (17) Boolean q: `harry -snape` (exclusion), `harry AND draco`,
/// `harry OR draco`.
#[ignore]
#[tokio::test]
async fn search_boolean_exclusion_and_or() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_boo_a", "searchit_boo_b", "searchit_boo_c"];
    seed_fic(
        &db,
        "searchit_boo_a",
        "Harry and Draco",
        "Harry Potter meets Draco Malfoy.",
        3000,
        3,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_boo_b",
        "Harry Alone",
        "Harry Potter goes solo.",
        2000,
        2,
        "complete",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_boo_c",
        "Snape's Story",
        "Severus Snape teaches.",
        2500,
        2,
        "complete",
        3,
    )
    .await;

    // harry -snape → fics mentioning harry but not snape
    let body = get_search("/api/search?q=harry%20-snape").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_boo_a".to_string()),
        "A has harry, no snape: {body}"
    );
    assert!(
        ids.contains(&"searchit_boo_b".to_string()),
        "B has harry, no snape: {body}"
    );
    assert!(
        !ids.contains(&"searchit_boo_c".to_string()),
        "C has snape: {body}"
    );

    // harry AND draco → both words in one fic
    let body = get_search("/api/search?q=harry%20AND%20draco").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_boo_a".to_string()),
        "A has harry AND draco: {body}"
    );
    assert!(
        !ids.contains(&"searchit_boo_b".to_string()),
        "B lacks draco: {body}"
    );
    assert!(
        !ids.contains(&"searchit_boo_c".to_string()),
        "C lacks harry: {body}"
    );

    // harry OR draco → A and B (harry) + nothing extra; snape-fic C stays out
    let body = get_search("/api/search?q=harry%20OR%20draco").await;
    let ids = result_ids(&body);
    assert!(ids.contains(&"searchit_boo_a".to_string()), "A: {body}");
    assert!(ids.contains(&"searchit_boo_b".to_string()), "B: {body}");
    assert!(!ids.contains(&"searchit_boo_c".to_string()), "C: {body}");

    cleanup(&db, &fics, &[]).await;
}

/// (17b) Native boolean search via websearch_to_tsquery + ts_headline snippet.
/// `harry OR ron` matches fics mentioning either; `-draco` excludes; the
/// snippet is a non-empty highlighted description excerpt for a match.
#[ignore]
#[tokio::test]
async fn search_native_boolean_or_exclusion_and_snippet() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_wb_a", "searchit_wb_b", "searchit_wb_c"];
    seed_fic(
        &db,
        "searchit_wb_a",
        "Harry's Story",
        "Harry Potter meets Draco Malfoy in a quiet corridor.",
        3000,
        3,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_wb_b",
        "Ron's Story",
        "Ron Weasley brews tea alone.",
        2000,
        2,
        "complete",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_wb_c",
        "Snape's Story",
        "Severus Snape teaches potions to nobody in particular.",
        2500,
        2,
        "complete",
        3,
    )
    .await;

    // OR — matches fics mentioning either term (A has harry, B has ron).
    let body = get_search("/api/search?q=harry%20OR%20ron").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_wb_a".to_string()),
        "A has harry: {body}"
    );
    assert!(
        ids.contains(&"searchit_wb_b".to_string()),
        "B has ron: {body}"
    );
    assert!(
        !ids.contains(&"searchit_wb_c".to_string()),
        "C has neither: {body}"
    );

    // -draco exclusion — removes the fic mentioning draco.
    let body = get_search("/api/search?q=harry%20-draco").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_wb_a".to_string()),
        "A mentions draco, excluded: {body}"
    );
    assert!(
        !ids.contains(&"searchit_wb_b".to_string()),
        "B has no harry: {body}"
    );
    assert!(
        !ids.contains(&"searchit_wb_c".to_string()),
        "C has no harry: {body}"
    );

    // Snippet: non-empty and highlights the matched term with <b>.
    let body = get_search("/api/search?q=tea").await;
    let b = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["url_id"] == "searchit_wb_b")
        .expect("fic B must be in results");
    let snippet = b["snippet"].as_str().unwrap_or("");
    assert!(
        !snippet.is_empty(),
        "snippet must be non-empty for a match: {body}"
    );
    assert!(
        snippet.contains("<b>"),
        "snippet must contain highlight tags: {snippet}"
    );
    assert!(
        snippet.contains("tea"),
        "snippet must contain the matched term: {snippet}"
    );

    // Implicit AND still works: both words must appear in one fic.
    let body = get_search("/api/search?q=potions%20teaches").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_wb_c".to_string()),
        "implicit AND: {body}"
    );
    assert!(
        !ids.contains(&"searchit_wb_a".to_string()),
        "A lacks both words: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

/// (18) Quoted phrases — `q="slow burn"` matches the exact phrase (adjacency
/// via `<->`), not just both words anywhere.
#[ignore]
#[tokio::test]
async fn search_quoted_phrase_adjacency() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_phr_a", "searchit_phr_b"];
    seed_fic(
        &db,
        "searchit_phr_a",
        "Embers",
        "A slow burn romance with tension.",
        5000,
        5,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_phr_b",
        "Kindling",
        "The burn was slow to start but fierce.",
        4000,
        4,
        "complete",
        2,
    )
    .await;

    let body = get_search("/api/search?q=%22slow%20burn%22").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_phr_a".to_string()),
        "A has the phrase: {body}"
    );
    assert!(
        !ids.contains(&"searchit_phr_b".to_string()),
        "B has the words apart, not the phrase: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

/// (19) Numeric bounds: min_words, max_words, min_chapters, max_chapters,
/// complete.
#[ignore]
#[tokio::test]
async fn search_numeric_bounds_and_complete() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_num_a", "searchit_num_b", "searchit_num_c"];
    // a: 1000 words / 1 chapter / complete; b: 10000 / 10 / ongoing; c: 5000 / 5 / complete
    seed_fic(
        &db,
        "searchit_num_a",
        "Num Alpha",
        "Shared topic numeric bounds.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_num_b",
        "Num Beta",
        "Shared topic numeric bounds.",
        10000,
        10,
        "ongoing",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_num_c",
        "Num Gamma",
        "Shared topic numeric bounds.",
        5000,
        5,
        "complete",
        3,
    )
    .await;

    let body = get_search("/api/search?q=numeric%20bounds&min_words=5000").await;
    assert_eq!(total(&body), 2, "min_words=5000: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&max_words=5000").await;
    assert_eq!(total(&body), 2, "max_words=5000: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&min_words=5000&max_words=10000").await;
    assert_eq!(total(&body), 2, "min+max words: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&min_chapters=5").await;
    assert_eq!(total(&body), 2, "min_chapters=5: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&max_chapters=5").await;
    assert_eq!(total(&body), 2, "max_chapters=5: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&complete=true").await;
    assert_eq!(total(&body), 2, "complete=true: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&complete=false").await;
    assert_eq!(total(&body), 1, "complete=false: {body}");
    let body = get_search("/api/search?q=numeric%20bounds&min_words=999999").await;
    assert_eq!(total(&body), 0, "impossible min_words: {body}");

    cleanup(&db, &fics, &[]).await;
}

/// (20) Source filter — seed_fic always writes source='ao3', so `source=ao3`
/// matches the seeded fics while a different source matches none.
#[ignore]
#[tokio::test]
async fn search_source_filter() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_src_a", "searchit_src_b"];
    seed_fic(
        &db,
        "searchit_src_a",
        "Source Alpha",
        "Shared topic source filter.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_src_b",
        "Source Beta",
        "Shared topic source filter.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let body = get_search("/api/search?q=source%20filter&source=ao3").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_src_a".to_string()),
        "source=ao3: {body}"
    );
    assert!(
        ids.contains(&"searchit_src_b".to_string()),
        "source=ao3: {body}"
    );

    let body = get_search("/api/search?q=source%20filter&source=ffn").await;
    assert_eq!(total(&body), 0, "source=ffn matches nothing: {body}");

    cleanup(&db, &fics, &[]).await;
}

/// (21) Empty search — NO filters at all returns ALL fics (the handler does
/// not require `q`); both seeded fics are returned.
#[ignore]
#[tokio::test]
async fn search_empty_no_filters_returns_all() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_emp_a", "searchit_emp_b"];
    seed_fic(
        &db,
        "searchit_emp_a",
        "Empty Alpha",
        "Empty search test.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_emp_b",
        "Empty Beta",
        "Empty search test.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let body = get_search("/api/search").await;
    let ids = result_ids(&body);
    assert!(body["total"].as_i64().is_some(), "total present: {body}");
    assert!(
        ids.contains(&"searchit_emp_a".to_string()),
        "empty search returns seeded fic A: {body}"
    );
    assert!(
        ids.contains(&"searchit_emp_b".to_string()),
        "empty search returns seeded fic B: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

/// (22) Sorting — sort=newest/oldest via '-date' / 'date' (default is -date:
/// newest first; unknown values fall back to -date). '-words' and '-title'
/// are also covered.
#[ignore]
#[tokio::test]
async fn search_sort_newest_oldest_words_title() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"];
    // fic_updated: a=now-1d (newest), b=now-2d, c=now-3d (oldest)
    seed_fic(
        &db,
        "searchit_sort_a",
        "Sort Alpha",
        "Shared topic sort order.",
        9000,
        9,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_sort_b",
        "Sort Beta",
        "Shared topic sort order.",
        1000,
        1,
        "ongoing",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_sort_c",
        "Sort Gamma",
        "Shared topic sort order.",
        500,
        3,
        "complete",
        3,
    )
    .await;

    // Default (no sort): newest first → a, b, c
    let body = get_search("/api/search?q=sort%20order").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"]
    );

    // -date (newest): a, b, c
    let body = get_search("/api/search?q=sort%20order&sort=-date").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"]
    );

    // date (oldest): c, b, a
    let body = get_search("/api/search?q=sort%20order&sort=date").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_c", "searchit_sort_b", "searchit_sort_a"]
    );

    // -words: a (9000), b (1000), c (500)
    let body = get_search("/api/search?q=sort%20order&sort=-words").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"]
    );

    // -title ascending: Alpha, Beta, Gamma
    let body = get_search("/api/search?q=sort%20order&sort=-title").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"]
    );

    // Unknown sort falls back to -date
    let body = get_search("/api/search?q=sort%20order&sort=bogus").await;
    assert_eq!(
        result_ids(&body),
        vec!["searchit_sort_a", "searchit_sort_b", "searchit_sort_c"]
    );

    cleanup(&db, &fics, &[]).await;
}

/// (23) Pagination — page 2 / per_page; total is unaffected.
#[ignore]
#[tokio::test]
async fn search_pagination_page_two() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = [
        "searchit_pg2_a",
        "searchit_pg2_b",
        "searchit_pg2_c",
        "searchit_pg2_d",
        "searchit_pg2_e",
    ];
    // Self-heal: a prior crashed run may have left stale rows with the same
    // ids (ON CONFLICT DO NOTHING would otherwise keep them and their old
    // fic_updated, corrupting the sort). Delete first, then seed fresh.
    for f in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1")
            .bind(f)
            .execute(&db)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(f)
            .execute(&db)
            .await;
    }
    for (i, f) in fics.iter().enumerate() {
        // Distinct fic_updated: a=1d (newest) ... e=5d (oldest)
        seed_fic(
            &db,
            f,
            &format!("Page2 {f}"),
            "Shared topic page two.",
            1000 + i as i64,
            1,
            "complete",
            1 + i as i64,
        )
        .await;
    }
    // Default -date order is by fic_updated DESC: a (1d) is newest, e (5d) oldest.
    // per_page=2, page=2 → c, d
    let body = get_search("/api/search?q=page%20two&per_page=2&page=2").await;
    assert_eq!(total(&body), 5, "total unaffected by pagination");
    assert_eq!(body["page"], 2);
    assert_eq!(body["per_page"], 2);
    assert_eq!(result_ids(&body), vec!["searchit_pg2_c", "searchit_pg2_d"]);

    // per_page=2, page=3 → e
    let body = get_search("/api/search?q=page%20two&per_page=2&page=3").await;
    assert_eq!(result_ids(&body), vec!["searchit_pg2_e"]);

    // Past the end → empty results, same total.
    let body = get_search("/api/search?q=page%20two&per_page=2&page=99").await;
    assert_eq!(total(&body), 5);
    assert!(result_ids(&body).is_empty());

    cleanup(&db, &fics, &[]).await;
}

/// (24) Facets — seeded data populates fandoms/characters/relationships/
/// warnings/categories/freeforms/statuses facets with correct counts.
#[ignore]
#[tokio::test]
async fn search_facets_all_types_for_seeded_data() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_fac_a", "searchit_fac_b", "searchit_fac_c"];
    seed_fic(
        &db,
        "searchit_fac_a",
        "Facet Alpha",
        "Shared topic facets.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_fac_b",
        "Facet Beta",
        "Shared topic facets.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;
    seed_fic(
        &db,
        "searchit_fac_c",
        "Facet Gamma",
        "Shared topic facets.",
        3000,
        3,
        "complete",
        3,
    )
    .await;

    let fandom = seed_tag(&db, "SearchTest Fandom Facets", 1).await;
    let char_a = seed_tag(&db, "SearchTest Character Facet A", 2).await;
    let char_b = seed_tag(&db, "SearchTest Character Facet B", 2).await;
    let rel = seed_tag(&db, "SearchTest Relationship Facet", 3).await;
    let warn = seed_tag(&db, "SearchTest Warning Facet", 5).await;
    let cat = seed_tag(&db, "SearchTest Category Facet", 6).await;
    let freeform = seed_tag(&db, "SearchTest Freeform Facet", 4).await;
    for f in fics {
        seed_fic_tag(&db, f, fandom, 5).await;
    }
    seed_fic_tag(&db, "searchit_fac_a", char_a, 5).await;
    seed_fic_tag(&db, "searchit_fac_b", char_b, 5).await;
    seed_fic_tag(&db, "searchit_fac_c", rel, 5).await;
    seed_fic_tag(&db, "searchit_fac_a", warn, 5).await;
    seed_fic_tag(&db, "searchit_fac_a", cat, 5).await;
    seed_fic_tag(&db, "searchit_fac_b", freeform, 5).await;

    let body = get_search("/api/search?q=facets").await;

    let fandoms = body["facets"]["fandoms"].as_array().unwrap();
    assert!(
        fandoms
            .iter()
            .any(|f| f["name"] == "SearchTest Fandom Facets" && f["count"] == 3),
        "fandoms facet: {body}"
    );

    let characters = body["facets"]["characters"].as_array().unwrap();
    assert!(
        characters
            .iter()
            .any(|f| f["name"] == "SearchTest Character Facet A" && f["count"] == 1),
        "characters facet: {body}"
    );
    assert!(
        characters
            .iter()
            .any(|f| f["name"] == "SearchTest Character Facet B" && f["count"] == 1),
        "characters facet: {body}"
    );

    let relationships = body["facets"]["relationships"].as_array().unwrap();
    assert!(
        relationships
            .iter()
            .any(|f| f["name"] == "SearchTest Relationship Facet" && f["count"] == 1),
        "relationships facet: {body}"
    );

    let warnings = body["facets"]["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|f| f["name"] == "SearchTest Warning Facet" && f["count"] == 1),
        "warnings facet: {body}"
    );

    let categories = body["facets"]["categories"].as_array().unwrap();
    assert!(
        categories
            .iter()
            .any(|f| f["name"] == "SearchTest Category Facet" && f["count"] == 1),
        "categories facet: {body}"
    );

    let freeforms = body["facets"]["freeforms"].as_array().unwrap();
    assert!(
        freeforms
            .iter()
            .any(|f| f["name"] == "SearchTest Freeform Facet" && f["count"] == 1),
        "freeforms facet: {body}"
    );

    let statuses = body["facets"]["statuses"].as_array().unwrap();
    assert!(
        statuses
            .iter()
            .any(|f| f["name"] == "complete" && f["count"] == 2),
        "statuses complete=2: {body}"
    );
    assert!(
        statuses
            .iter()
            .any(|f| f["name"] == "ongoing" && f["count"] == 1),
        "statuses ongoing=1: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Fandom Facets",
            "SearchTest Character Facet A",
            "SearchTest Character Facet B",
            "SearchTest Relationship Facet",
            "SearchTest Warning Facet",
            "SearchTest Category Facet",
            "SearchTest Freeform Facet",
        ],
    )
    .await;
}

/// (25) Case-insensitive tag lookup — `include_tags=2:harry potter`
/// (lowercase) matches the "Harry Potter" tag.
#[ignore]
#[tokio::test]
async fn search_include_tags_case_insensitive() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_ci_a", "searchit_ci_b"];
    seed_fic(
        &db,
        "searchit_ci_a",
        "Case Alpha",
        "Case-insensitive tag test.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_ci_b",
        "Case Beta",
        "Case-insensitive tag test.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let harry = seed_tag(&db, "SearchTest Harry Potter", 2).await;
    seed_fic_tag(&db, "searchit_ci_a", harry, 5).await;

    // Lowercase type-2 lookup resolves via get_matching_tag_ids (LOWER(name) = LOWER($1)).
    let body = get_search("/api/search?include_tags=2%3Asearchtest%20harry%20potter").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_ci_a".to_string()),
        "lowercase lookup matches Harry Potter: {body}"
    );
    assert!(
        !ids.contains(&"searchit_ci_b".to_string()),
        "fic B has no Harry tag: {body}"
    );

    // Mixed-case lookup too.
    let body = get_search("/api/search?include_tags=2%3ASEARCHTEST%20HARRY%20POTTER").await;
    assert!(
        result_ids(&body).contains(&"searchit_ci_a".to_string()),
        "uppercase lookup: {body}"
    );

    cleanup(&db, &fics, &["SearchTest Harry Potter"]).await;
}

/// (26) Special-character tags — relationship "Draco Malfoy/Harry Potter"
/// searchable via include_tags (URL-encoded %2F for '/'), and a freeform
/// with '&' ("Bill & Fleur") searchable via %26.
#[ignore]
#[tokio::test]
async fn search_special_char_tags_slash_and_ampersand() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_spc_a", "searchit_spc_b"];
    seed_fic(
        &db,
        "searchit_spc_a",
        "Special Alpha",
        "Special character tags.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_spc_b",
        "Special Beta",
        "Special character tags.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let rel = seed_tag(&db, "SearchTest Draco Malfoy/Harry Potter", 3).await;
    let freeform = seed_tag(&db, "SearchTest Bill & Fleur", 4).await;
    seed_fic_tag(&db, "searchit_spc_a", rel, 5).await;
    seed_fic_tag(&db, "searchit_spc_b", freeform, 5).await;

    // '/' must be URL-encoded as %2F.
    let body =
        get_search("/api/search?include_tags=3%3ASearchTest%20Draco%20Malfoy%2FHarry%20Potter")
            .await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_spc_a".to_string()),
        "slash tag match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_spc_b".to_string()),
        "slash tag leak: {body}"
    );

    // '&' must be URL-encoded as %26.
    let body = get_search("/api/search?include_tags=4%3ASearchTest%20Bill%20%26%20Fleur").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_spc_b".to_string()),
        "ampersand tag match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_spc_a".to_string()),
        "ampersand tag leak: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Draco Malfoy/Harry Potter",
            "SearchTest Bill & Fleur",
        ],
    )
    .await;
}

/// (27) tag_ids — numeric tag id search: fics carrying the tag match, others
/// do not; nonexistent ids match nothing.
#[ignore]
#[tokio::test]
async fn search_tag_ids_numeric() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_tid_a", "searchit_tid_b"];
    seed_fic(
        &db,
        "searchit_tid_a",
        "TagID Alpha",
        "Tag id search test.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_tid_b",
        "TagID Beta",
        "Tag id search test.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let tag = seed_tag(&db, "SearchTest TagID Target", 4).await;
    seed_fic_tag(&db, "searchit_tid_a", tag, 5).await;

    let body = get_search(&format!("/api/search?tag_ids={}", tag)).await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_tid_a".to_string()),
        "tag_ids match: {body}"
    );
    assert!(
        !ids.contains(&"searchit_tid_b".to_string()),
        "tag_ids leak: {body}"
    );

    // Nonexistent tag id → no results.
    let body = get_search("/api/search?tag_ids=999999999").await;
    assert_eq!(total(&body), 0, "nonexistent tag id: {body}");

    cleanup(&db, &fics, &["SearchTest TagID Target"]).await;
}

/// (28) no_warnings — true excludes fics with warning tags (type 5); false
/// keeps everything.
#[ignore]
#[tokio::test]
async fn search_no_warnings_excludes_warning_tagged() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_nw_a", "searchit_nw_b"];
    seed_fic(
        &db,
        "searchit_nw_a",
        "NoWarn Alpha",
        "No warnings test.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_nw_b",
        "NoWarn Beta",
        "No warnings test.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let warn = seed_tag(&db, "SearchTest Major Character Death", 5).await;
    seed_fic_tag(&db, "searchit_nw_a", warn, 5).await;

    // no_warnings=true → fic A (has the warning tag) excluded, fic B kept.
    let body = get_search("/api/search?q=no%20warnings%20test&no_warnings=true").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_nw_a".to_string()),
        "fic A has a warning tag: {body}"
    );
    assert!(
        ids.contains(&"searchit_nw_b".to_string()),
        "fic B has no warnings: {body}"
    );

    // no_warnings=false → no exclusion at all.
    let body = get_search("/api/search?q=no%20warnings%20test&no_warnings=false").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_nw_a".to_string()),
        "no_warnings=false keeps everything: {body}"
    );
    assert!(
        ids.contains(&"searchit_nw_b".to_string()),
        "no_warnings=false keeps everything: {body}"
    );

    cleanup(&db, &fics, &["SearchTest Major Character Death"]).await;
}

/// (29) exclude_tag_types — fics carrying ANY tag of the excluded types are
/// dropped; fics without such tags stay. This is the "no ships" filter:
/// exclude_tag_types=3 removes every fic with a relationship tag.
#[ignore]
#[tokio::test]
async fn search_exclude_tag_types_no_ships() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_ett_a", "searchit_ett_b"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("ExclType {f}"),
            "Exclude tag types test.",
            800,
            1,
            "complete",
            1,
        )
        .await;
    }
    let rel = seed_tag(&db, "SearchTest Harry/Draco", 3).await;
    let freeform = seed_tag(&db, "SearchTest Fluffy", 4).await;
    // A has a relationship tag; B has only a freeform tag.
    seed_fic_tag(&db, "searchit_ett_a", rel, 5).await;
    seed_fic_tag(&db, "searchit_ett_b", freeform, 5).await;

    // exclude_tag_types=3 → A dropped, B kept.
    let body = get_search("/api/search?exclude_tag_types=3").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_ett_a".to_string()),
        "A has a relationship tag: {body}"
    );
    assert!(
        ids.contains(&"searchit_ett_b".to_string()),
        "B has no relationship tag: {body}"
    );

    // exclude_tag_types=4 → B dropped (freeform), A kept.
    let body = get_search("/api/search?exclude_tag_types=4").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_ett_a".to_string()),
        "A has no freeform tag: {body}"
    );
    assert!(
        !ids.contains(&"searchit_ett_b".to_string()),
        "B has a freeform tag: {body}"
    );

    // No exclusion → both present.
    let body = get_search("/api/search").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_ett_a".to_string()),
        "no filter keeps A: {body}"
    );
    assert!(
        ids.contains(&"searchit_ett_b".to_string()),
        "no filter keeps B: {body}"
    );

    cleanup(&db, &fics, &["SearchTest Harry/Draco", "SearchTest Fluffy"]).await;
}

/// (30) strict_gen — hides fics with relationship (type 3) OR category
/// (type 6) tags; fics with neither stay. This is the "strict gen" mode.
#[ignore]
#[tokio::test]
async fn search_strict_gen_hides_ships_and_categories() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_sg_a", "searchit_sg_b", "searchit_sg_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("StrictGen {f}"),
            "Strict gen test.",
            800,
            1,
            "complete",
            1,
        )
        .await;
    }
    let rel = seed_tag(&db, "SearchTest Draco/Harry", 3).await;
    let cat = seed_tag(&db, "SearchTest M/M", 6).await;
    let freeform = seed_tag(&db, "SearchTest Gen Vibes", 4).await;
    // A: has a relationship tag. B: has a category tag. C: has neither.
    seed_fic_tag(&db, "searchit_sg_a", rel, 5).await;
    seed_fic_tag(&db, "searchit_sg_b", cat, 5).await;
    seed_fic_tag(&db, "searchit_sg_c", freeform, 5).await;

    // strict_gen=true → A and B dropped, C kept.
    let body = get_search("/api/search?strict_gen=true").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_sg_a".to_string()),
        "A has a ship: {body}"
    );
    assert!(
        !ids.contains(&"searchit_sg_b".to_string()),
        "B has a category tag: {body}"
    );
    assert!(
        ids.contains(&"searchit_sg_c".to_string()),
        "C is gen: {body}"
    );

    // strict_gen=false (or absent) → everything stays (default unchanged).
    let body = get_search("/api/search?strict_gen=false").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_sg_a".to_string()),
        "strict_gen=false keeps A: {body}"
    );
    assert!(
        ids.contains(&"searchit_sg_b".to_string()),
        "strict_gen=false keeps B: {body}"
    );
    assert!(
        ids.contains(&"searchit_sg_c".to_string()),
        "strict_gen=false keeps C: {body}"
    );

    // strict_gen=true combined with a text query — the exclusion still applies.
    let body = get_search("/api/search?q=strict%20gen%20test&strict_gen=true").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_sg_a".to_string()),
        "text+strict_gen drops A: {body}"
    );
    assert!(
        !ids.contains(&"searchit_sg_b".to_string()),
        "text+strict_gen drops B: {body}"
    );
    assert!(
        ids.contains(&"searchit_sg_c".to_string()),
        "text+strict_gen keeps C: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Draco/Harry",
            "SearchTest M/M",
            "SearchTest Gen Vibes",
        ],
    )
    .await;
}

/// (31) exclude_tag_types combined with strict_gen — the merged exclusion
/// drops fics matching EITHER filter, and fics matching neither stay.
#[ignore]
#[tokio::test]
async fn search_exclude_tag_types_and_strict_gen_combined() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_ec_a", "searchit_ec_b", "searchit_ec_c"];
    for f in fics {
        seed_fic(
            &db,
            f,
            &format!("ExclCombo {f}"),
            "Exclude combo test.",
            800,
            1,
            "complete",
            1,
        )
        .await;
    }
    let rel = seed_tag(&db, "SearchTest Ron/Hermione", 3).await;
    let warn = seed_tag(&db, "SearchTest Graphic Violence", 5).await;
    let freeform = seed_tag(&db, "SearchTest Chill", 4).await;
    // A: relationship (would be dropped by strict_gen). B: warning (dropped
    // by exclude_tag_types=5, but NOT by strict_gen). C: neither.
    seed_fic_tag(&db, "searchit_ec_a", rel, 5).await;
    seed_fic_tag(&db, "searchit_ec_b", warn, 5).await;
    seed_fic_tag(&db, "searchit_ec_c", freeform, 5).await;

    // exclude_tag_types=5 + strict_gen → A (ship) and B (warning) dropped, C kept.
    let body = get_search("/api/search?exclude_tag_types=5&strict_gen=true").await;
    let ids = result_ids(&body);
    assert!(
        !ids.contains(&"searchit_ec_a".to_string()),
        "A is a ship: {body}"
    );
    assert!(
        !ids.contains(&"searchit_ec_b".to_string()),
        "B has a warning: {body}"
    );
    assert!(
        ids.contains(&"searchit_ec_c".to_string()),
        "C stays: {body}"
    );

    cleanup(
        &db,
        &fics,
        &[
            "SearchTest Ron/Hermione",
            "SearchTest Graphic Violence",
            "SearchTest Chill",
        ],
    )
    .await;
}

/// Seed a kudos entry (idempotent). Returns the kudos id.
/// `user_id` can be None for guest kudos.
async fn seed_kudos(pool: &sqlx::PgPool, work_id: i32, user_id: Option<i32>) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO kudos (work_id, user_id) VALUES ($1, $2) ON CONFLICT (work_id, user_id) DO NOTHING RETURNING id",
    )
    .bind(work_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .expect("seed_kudos failed")
    .unwrap_or_else(|| {
        // Already exists — look it up
        std::panic::panic_any("seed_kudos: row already existed but RETURNING failed")
    })
}

/// (32) max_kudos — filters fics with kudos count <= max. Two fics with
/// different kudos counts; max_kudos=1 returns only the low-kudos fic.
#[ignore]
#[tokio::test]
async fn search_max_kudos_filters_upper_bound() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_mk_a", "searchit_mk_b"];
    seed_fic(
        &db,
        "searchit_mk_a",
        "MaxKudos Alpha",
        "Shared topic max kudos.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_mk_b",
        "MaxKudos Beta",
        "Shared topic max kudos.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    // Create a work for each fic so kudos can reference it.
    let work_a: i32 =
        sqlx::query_scalar("INSERT INTO works (canonical_title) VALUES ($1) RETURNING id")
            .bind("MaxKudos Alpha Work")
            .fetch_one(&db)
            .await
            .expect("seed work_a");
    let work_b: i32 =
        sqlx::query_scalar("INSERT INTO works (canonical_title) VALUES ($1) RETURNING id")
            .bind("MaxKudos Beta Work")
            .fetch_one(&db)
            .await
            .expect("seed work_b");

    // Update fic_info with work_ids.
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_a)
        .bind("searchit_mk_a")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_b)
        .bind("searchit_mk_b")
        .execute(&db)
        .await
        .unwrap();

    // Seed kudos: fic A gets 1 signed-in kudo, fic B gets 5.
    // Need users for signed-in kudos.
    let user1 = seed_suggest_user(&db, "mk_user1").await;
    let user2 = seed_suggest_user(&db, "mk_user2").await;
    let user3 = seed_suggest_user(&db, "mk_user3").await;
    let user4 = seed_suggest_user(&db, "mk_user4").await;
    let user5 = seed_suggest_user(&db, "mk_user5").await;
    let user6 = seed_suggest_user(&db, "mk_user6").await;
    seed_kudos(&db, work_a, Some(user1)).await;
    seed_kudos(&db, work_b, Some(user2)).await;
    seed_kudos(&db, work_b, Some(user3)).await;
    seed_kudos(&db, work_b, Some(user4)).await;
    seed_kudos(&db, work_b, Some(user5)).await;
    seed_kudos(&db, work_b, Some(user6)).await;

    // max_kudos=1 → only fic A (1 kudo <= 1) matches.
    let body = get_search("/api/search?q=max%20kudos&max_kudos=1").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_mk_a".to_string()),
        "fic A (1 kudo) must match max_kudos=1: {body}"
    );
    assert!(
        !ids.contains(&"searchit_mk_b".to_string()),
        "fic B (5 kudos) must NOT match max_kudos=1: {body}"
    );

    // max_kudos=5 → both fics match (1 <= 5 and 5 <= 5).
    let body = get_search("/api/search?q=max%20kudos&max_kudos=5").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_mk_a".to_string()),
        "fic A matches max_kudos=5: {body}"
    );
    assert!(
        ids.contains(&"searchit_mk_b".to_string()),
        "fic B matches max_kudos=5: {body}"
    );

    // Clean up kudos and works.
    let _ = sqlx::query("DELETE FROM kudos WHERE work_id IN ($1, $2)")
        .bind(work_a)
        .bind(work_b)
        .execute(&db)
        .await;
    let _ = sqlx::query("UPDATE fic_info SET work_id = NULL WHERE id IN ($1, $2)")
        .bind("searchit_mk_a")
        .bind("searchit_mk_b")
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE id IN ($1, $2)")
        .bind(work_a)
        .bind(work_b)
        .execute(&db)
        .await;
    cleanup(&db, &fics, &[]).await;
    for u in [
        "mk_user1", "mk_user2", "mk_user3", "mk_user4", "mk_user5", "mk_user6",
    ] {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(&db)
            .await;
    }
}

/// (33) min_bookmarks / max_bookmarks — filters by bookmark count on the
/// bookmarks table. Two fics with different bookmark counts.
#[ignore]
#[tokio::test]
async fn search_bookmarks_bounds() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["searchit_bm_a", "searchit_bm_b"];
    seed_fic(
        &db,
        "searchit_bm_a",
        "Bookmark Alpha",
        "Shared topic bookmarks.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_bm_b",
        "Bookmark Beta",
        "Shared topic bookmarks.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    // Create users for bookmarks.
    let u1 = seed_suggest_user(&db, "bm_user1").await;
    let u2 = seed_suggest_user(&db, "bm_user2").await;
    let u3 = seed_suggest_user(&db, "bm_user3").await;

    // Fic A: 1 bookmark (u1). Fic B: 3 bookmarks (u1, u2, u3).
    seed_bookmark(&db, u1, "searchit_bm_a").await;
    seed_bookmark(&db, u1, "searchit_bm_b").await;
    seed_bookmark(&db, u2, "searchit_bm_b").await;
    seed_bookmark(&db, u3, "searchit_bm_b").await;

    // min_bookmarks=2 → only fic B (3 >= 2).
    let body = get_search("/api/search?q=bookmarks&min_bookmarks=2").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bm_b".to_string()),
        "fic B (3 bm) must match min_bookmarks=2: {body}"
    );
    assert!(
        !ids.contains(&"searchit_bm_a".to_string()),
        "fic A (1 bm) must NOT match min_bookmarks=2: {body}"
    );

    // max_bookmarks=1 → only fic A (1 <= 1).
    let body = get_search("/api/search?q=bookmarks&max_bookmarks=1").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bm_a".to_string()),
        "fic A (1 bm) must match max_bookmarks=1: {body}"
    );
    assert!(
        !ids.contains(&"searchit_bm_b".to_string()),
        "fic B (3 bm) must NOT match max_bookmarks=1: {body}"
    );

    // min_bookmarks=1 & max_bookmarks=3 → both fics.
    let body = get_search("/api/search?q=bookmarks&min_bookmarks=1&max_bookmarks=3").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_bm_a".to_string()),
        "fic A matches range: {body}"
    );
    assert!(
        ids.contains(&"searchit_bm_b".to_string()),
        "fic B matches range: {body}"
    );

    // Clean up bookmarks.
    let _ = sqlx::query("DELETE FROM bookmarks WHERE url_id IN ($1, $2)")
        .bind("searchit_bm_a")
        .bind("searchit_bm_b")
        .execute(&db)
        .await;
    cleanup(&db, &fics, &[]).await;
    for u in ["bm_user1", "bm_user2", "bm_user3"] {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(&db)
            .await;
    }
}

/// (34) rating — content rating filter resolves to an include_tags filter
/// (tag_type_id = 7). Seed a "General Audiences" rating tag and link it
/// to one fic; the other fic gets no rating tag. `rating=General Audiences`
/// returns only the rated fic.
#[ignore]
#[tokio::test]
async fn search_rating_filter() {
    let _guard = db_guard();
    let db = pool().await;
    clean_all_test_fixtures(&db).await;
    let fics = ["searchit_rt_a", "searchit_rt_b"];
    seed_fic(
        &db,
        "searchit_rt_a",
        "Rating Alpha",
        "Shared topic rating.",
        1000,
        1,
        "complete",
        1,
    )
    .await;
    seed_fic(
        &db,
        "searchit_rt_b",
        "Rating Beta",
        "Shared topic rating.",
        2000,
        2,
        "ongoing",
        2,
    )
    .await;

    let tag = seed_tag(&db, "SearchTest General Audiences", 7).await; // tag_type_id = 7 (rating)
    seed_fic_tag(&db, "searchit_rt_a", tag, 5).await;

    // rating=General Audiences → only fic A (has the rating tag).
    let body = get_search("/api/search?q=rating&rating=SearchTest%20General%20Audiences").await;
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_rt_a".to_string()),
        "fic A has the rating tag: {body}"
    );
    assert!(
        !ids.contains(&"searchit_rt_b".to_string()),
        "fic B has no rating tag: {body}"
    );

    // A nonexistent rating name → no results (the rating doesn't resolve to a tag).
    let body = get_search("/api/search?q=rating&rating=NoSuchRating").await;
    // Nonexistent rating is silently ignored, so the search is unfiltered.
    let ids = result_ids(&body);
    assert!(
        ids.contains(&"searchit_rt_a".to_string()),
        "unfiltered: fic A present: {body}"
    );
    assert!(
        ids.contains(&"searchit_rt_b".to_string()),
        "unfiltered: fic B present: {body}"
    );

    cleanup(&db, &fics, &["SearchTest General Audiences"]).await;
}

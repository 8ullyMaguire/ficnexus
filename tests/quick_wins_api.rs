//! Integration tests for the quick-wins batch:
//!
//! * `GET /api/tags/autocomplete?q=...` — tag autocomplete WITH usage
//!   counts (count = fic_tags rows).
//! * `GET /api/works/random` — the "Surprise me" random-fic endpoint.
//! * `GET /api/search?hide_read=1&hide_bookmarked=1&library_only=1` —
//!   per-user personal filters (hide read / hide bookmarked / My library).
//!
//! Like `tests/search_api.rs`, these run against the real configured
//! database (`DATABASE_URL` from `.env`) and are serialised via a global
//! `Mutex` + marked `#[ignore]` so they only run when explicitly
//! requested: `cargo test --test quick_wins_api -- --include-ignored --test-threads=1`.
//!
//! Each test seeds its own uniquely-named fics/tags/users and cleans up
//! exactly the rows it created (self-heal at START), so the suite is
//! re-runnable and never touches pre-existing data.

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

fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    c
}

/// Build a router with the real handlers and real state.
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
    });

    Router::new()
        .route(
            "/api/tags/autocomplete",
            get(fichub::tags::routes::tag_autocomplete),
        )
        .route(
            "/api/works/random",
            get(fichub::routes::search::random_work),
        )
        .route("/api/search", get(fichub::search::routes::search_handler))
        .with_state(state)
}

/// Issue GET through the real router, assert 200, parse JSON.
async fn get_json(app_router: &Router, uri: &str, auth: Option<&str>) -> Value {
    let mut builder = Request::builder().uri(uri);
    if let Some(a) = auth {
        builder = builder.header("Authorization", a);
    }
    let response = app_router
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 for {uri}, got {status}: {}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).unwrap()
}

/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps this idempotent.
async fn seed_fic(pool: &sqlx::PgPool, id: &str, title: &str, description: &str) {
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
    .bind(1i32)
    .bind(1000i64)
    .bind(description)
    .bind(chrono::Utc::now() - chrono::Duration::days(100))
    .bind(chrono::Utc::now())
    .bind("complete")
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

/// Seed a tag row; returns its id. `ON CONFLICT (name) DO NOTHING`.
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

/// Link a tag to a fic. Idempotent.
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

/// Seed a user; returns the user id. Self-heals (unique username).
async fn seed_user(pool: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, 'test-hash') ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user: lookup failed")
}

/// JWT Bearer header for the given user id (AuthUser reads JWT_SECRET from env).
fn auth_header(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "quickwins_user".into(),
        role: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(pool: &sqlx::PgPool, fics: &[&str], tags: &[&str], users: &[&str]) {
    for id in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM bookmarks WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM reading_stats WHERE work_id IN (SELECT work_id FROM fic_info WHERE id = $1)")
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
    for username in users {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(pool)
            .await;
    }
}

// ─── Tag autocomplete with counts ───────────────────────────────────────────

/// Seeds two tags with the same prefix, attached to a different number of
/// fics, and asserts:
///  * `/api/tags/autocomplete?q=<prefix>` returns BOTH, ordered by usage;
///  * each result carries `usage_count` = number of fic_tags rows;
///  * the more-used tag sorts first.
#[ignore]
#[tokio::test]
async fn tag_autocomplete_returns_tags_with_usage_counts() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["quickwin_ac_a", "quickwin_ac_b"];
    let tags = ["QuickWin Autocomplete Alpha", "QuickWin Autocomplete Beta"];

    // Self-heal: clear any rows from a previously aborted run.
    cleanup(&db, &fics, &tags, &[]).await;

    seed_fic(
        &db,
        "quickwin_ac_a",
        "Autocomplete Alpha Fic",
        "A story about autocomplete alpha.",
    )
    .await;
    seed_fic(
        &db,
        "quickwin_ac_b",
        "Autocomplete Beta Fic",
        "A story about autocomplete beta.",
    )
    .await;

    let tag_a = seed_tag(&db, "QuickWin Autocomplete Alpha", 4).await;
    let tag_b = seed_tag(&db, "QuickWin Autocomplete Beta", 4).await;
    // Alpha used on BOTH fics; Beta on only one → counts 2 vs 1.
    seed_fic_tag(&db, "quickwin_ac_a", tag_a, 5).await;
    seed_fic_tag(&db, "quickwin_ac_b", tag_a, 5).await;
    seed_fic_tag(&db, "quickwin_ac_b", tag_b, 5).await;

    let router = app().await;
    let body = get_json(
        &router,
        "/api/tags/autocomplete?q=QuickWin%20Autocomplete",
        None,
    )
    .await;
    assert_eq!(body["err"], 0, "autocomplete err: {body}");
    let results = body["results"].as_array().expect("results array");
    assert!(
        results.len() >= 2,
        "expected both seeded tags, got {results:?}: {body}"
    );

    // Find our seeds and verify usage counts + ordering.
    let mut alpha_count = None;
    let mut beta_count = None;
    let mut alpha_pos = usize::MAX;
    let mut beta_pos = usize::MAX;
    for (i, r) in results.iter().enumerate() {
        if r["name"] == "QuickWin Autocomplete Alpha" {
            alpha_count = r["usage_count"].as_i64();
            alpha_pos = i;
        }
        if r["name"] == "QuickWin Autocomplete Beta" {
            beta_count = r["usage_count"].as_i64();
            beta_pos = i;
        }
    }
    assert_eq!(alpha_count, Some(2), "alpha usage_count must be 2: {body}");
    assert_eq!(beta_count, Some(1), "beta usage_count must be 1: {body}");
    assert!(
        alpha_pos < beta_pos,
        "more-used tag must sort first: {body}"
    );
    // Shape: id, name, tag_type_id present.
    let first = &results[0];
    assert!(first["id"].as_i64().is_some(), "id missing: {body}");
    // The autocomplete response serializes the type as "type_id" (not
    // "tag_type_id") — see tag_autocomplete in src/tags/routes.rs.
    assert!(
        first["type_id"].as_i64().is_some(),
        "type_id missing: {body}"
    );

    cleanup(&db, &fics, &tags, &[]).await;
}

/// A query of fewer than 2 characters returns an empty list (the endpoint's
/// debounce guard — the frontend also debounces, so this only fires on
/// programmatic calls).
#[ignore]
#[tokio::test]
async fn tag_autocomplete_short_query_returns_empty() {
    let _guard = db_guard();
    let router = app().await;
    let body = get_json(&router, "/api/tags/autocomplete?q=a", None).await;
    assert_eq!(body["err"], 0);
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
}

// ─── Random work ("Surprise me") ────────────────────────────────────────────

/// Seeds two fics and asserts `/api/works/random` returns one of them with
/// FULL metadata (title, author, source, words, chapters, status,
/// description) — unlike blind-date, no signed reveal needed.
#[ignore]
#[tokio::test]
async fn random_work_returns_full_metadata_fic() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["quickwin_rand_a", "quickwin_rand_b"];
    cleanup(&db, &fics, &[], &[]).await;

    seed_fic(
        &db,
        "quickwin_rand_a",
        "QuickWin Random Alpha",
        "A surprising tale of alpha.",
    )
    .await;
    seed_fic(
        &db,
        "quickwin_rand_b",
        "QuickWin Random Beta",
        "A surprising tale of beta.",
    )
    .await;

    let router = app().await;
    let body = get_json(&router, "/api/works/random", None).await;
    assert_eq!(body["err"], 0, "random err: {body}");
    let fic = body["fic"].as_object().expect("fic object");
    // The shared live DB holds real fics too, so the pick may be ours or a
    // pre-existing one — what must hold is well-formed full metadata.
    assert!(fic["url_id"].as_str().is_some(), "url_id missing: {body}");
    assert!(fic["title"].as_str().is_some(), "title missing: {body}");
    assert!(fic["author"].as_str().is_some(), "author missing: {body}");
    assert!(fic["source"].as_str().is_some(), "source missing: {body}");
    assert!(fic["words"].as_i64().is_some(), "words missing: {body}");
    assert!(
        fic["chapters"].as_i64().is_some(),
        "chapters missing: {body}"
    );
    assert!(fic["status"].as_str().is_some(), "status missing: {body}");
    assert!(
        fic["description"].as_str().is_some(),
        "description missing: {body}"
    );
    assert!(
        fic["description"]
            .as_str()
            .map(|d| !d.is_empty())
            .unwrap_or(false),
        "description must be non-empty (eligible filter): {body}"
    );

    cleanup(&db, &fics, &[], &[]).await;
}

// ─── Personal search filters (hide read / hide bookmarked / My library) ────

/// Seeds a user, two fics, a bookmark on one and a completed-reading row on
/// the other, then asserts:
///  * `hide_bookmarked=1` drops the bookmarked fic;
///  * `hide_read=1` drops the completed fic;
///  * `library_only=1` returns ONLY the bookmarked fic.
#[ignore]
#[tokio::test]
async fn search_personal_filters_hide_and_library() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["quickwin_pers_a", "quickwin_pers_b"];
    let username = "quickwins_pers_user";
    cleanup(&db, &fics, &[], &[username]).await;

    seed_fic(
        &db,
        "quickwin_pers_a",
        "Persistent Filter Alpha",
        "Alpha subject matter personal filter test.",
    )
    .await;
    seed_fic(
        &db,
        "quickwin_pers_b",
        "Persistent Filter Beta",
        "Beta subject matter personal filter test.",
    )
    .await;

    let user_id = seed_user(&db, username).await;
    let auth = auth_header(user_id);

    // User bookmarks fic A; user completed fic B.
    sqlx::query("INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING")
        .bind(user_id)
        .bind("quickwin_pers_a")
        .execute(&db)
        .await
        .expect("seed bookmark");
    let work_b: Option<i32> = sqlx::query_scalar("SELECT work_id FROM fic_info WHERE id = $1")
        .bind("quickwin_pers_b")
        .fetch_one(&db)
        .await
        .expect("fic B work_id");
    // fic_info.work_id is nullable — seed a works row so reading_stats has a
    // real work_id to reference (the filter joins on fi.work_id).
    let work_b = match work_b {
        Some(w) => w,
        None => {
            let inserted: i32 = sqlx::query_scalar(
                "INSERT INTO works (canonical_title, canonical_author) VALUES ($1, $2) RETURNING id",
            )
            .bind("Persistent Filter Beta")
            .bind("test_author")
            .fetch_one(&db)
            .await
            .expect("seed works row");
            sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
                .bind(inserted)
                .bind("quickwin_pers_b")
                .execute(&db)
                .await
                .expect("link fic_info.work_id");
            inserted
        }
    };

    sqlx::query(
        "INSERT INTO reading_stats (user_id, work_id, status) VALUES ($1, $2, 'completed') ON CONFLICT (user_id, work_id) DO UPDATE SET status = 'completed'",
    )
    .bind(user_id)
    .bind(work_b)
    .execute(&db)
    .await
    .expect("seed reading_stats");

    // Personal search filters are parsed as stringly "true"/"false" query
    // params (axum's serde_urlencoded rejects bare `=1` for Option<bool>).
    let router = app().await;
    let q = "subject%20matter";

    // Baseline (authenticated, no personal filters): both fics present.
    let body = get_json(&router, &format!("/api/search?q={q}"), Some(&auth)).await;
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(String::from))
        .collect();
    assert!(
        ids.contains(&"quickwin_pers_a".to_string()),
        "baseline A: {body}"
    );
    assert!(
        ids.contains(&"quickwin_pers_b".to_string()),
        "baseline B: {body}"
    );

    // hide_bookmarked=true → fic A (bookmarked) gone.
    let body = get_json(
        &router,
        &format!("/api/search?q={q}&hide_bookmarked=true"),
        Some(&auth),
    )
    .await;
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(String::from))
        .collect();
    assert!(
        !ids.contains(&"quickwin_pers_a".to_string()),
        "hide_bookmarked leaks A: {body}"
    );
    assert!(
        ids.contains(&"quickwin_pers_b".to_string()),
        "hide_bookmarked keeps B: {body}"
    );

    // hide_read=true → fic B (completed) gone.
    let body = get_json(
        &router,
        &format!("/api/search?q={q}&hide_read=true"),
        Some(&auth),
    )
    .await;
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(String::from))
        .collect();
    assert!(
        !ids.contains(&"quickwin_pers_b".to_string()),
        "hide_read leaks B: {body}"
    );
    assert!(
        ids.contains(&"quickwin_pers_a".to_string()),
        "hide_read keeps A: {body}"
    );

    // library_only=true → ONLY the bookmarked fic A.
    let body = get_json(
        &router,
        &format!("/api/search?q={q}&library_only=true"),
        Some(&auth),
    )
    .await;
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(String::from))
        .collect();
    assert_eq!(
        ids,
        vec!["quickwin_pers_a".to_string()],
        "library_only: {body}"
    );

    // Anonymous (no auth): personal filters are no-ops → both fics present.
    let body = get_json(
        &router,
        &format!("/api/search?q={q}&hide_read=true&hide_bookmarked=true&library_only=true"),
        None,
    )
    .await;
    let ids: Vec<String> = body["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["url_id"].as_str().map(String::from))
        .collect();
    assert!(
        ids.contains(&"quickwin_pers_a".to_string()),
        "anon keeps A: {body}"
    );
    assert!(
        ids.contains(&"quickwin_pers_b".to_string()),
        "anon keeps B: {body}"
    );

    cleanup(&db, &fics, &[], &[username]).await;
}

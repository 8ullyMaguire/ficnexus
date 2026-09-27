//! Integration tests for the Blind Date endpoint (`GET /api/blind-date`).
//!
//! Like `tests/search_api.rs`, these run against the real configured
//! database (`DATABASE_URL` from `.env`, database `fichub`) and are
//! serialised via a global `Mutex` + marked `#[ignore]` so they only run
//! when explicitly requested: `cargo test --test blind_date_api -- --include-ignored`.
//!
//! Each test seeds its own uniquely-named fics/tags and cleans up exactly
//! the rows it created, so the suite is re-runnable and never touches
//! pre-existing data (the live DB holds a handful of real fics).

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

/// Minimal config with sane defaults for the blind-date handler.
fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    c
}

/// Build a router with the real blind-date handler and real state.
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
        jwt_secret: TEST_JWT_SECRET.into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/blind-date",
            get(fichub::routes::blind::blind_date_handler),
        )
        .route(
            "/api/blind-date/reveal",
            get(fichub::routes::blind::blind_date_reveal_handler),
        )
        .with_state(state)
}

/// Issue GET /api/blind-date[?exclude=...] through the real router.
async fn get_blind_date(uri: &str) -> Value {
    let app = app().await;
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
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

/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps this idempotent.
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
    .bind("test_author")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(chapters)
    .bind(words)
    .bind(description)
    .bind(chrono::Utc::now() - chrono::Duration::days(100))
    .bind(chrono::Utc::now())
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

// ─── Tests ──────────────────────────────────────────────────────────────────

/// Seeds 2 fics (one complete, one ongoing) with descriptions + tags and
/// asserts:
///  * the response is one of the seeded fics (never the empty-archive null);
///  * the fic carries a description, word/chapter counts, status and the
///    top-3 tropes (character/freeform tags by score);
///  * passing `?exclude=<url_id>` guarantees the OTHER fic comes back.
#[ignore]
#[tokio::test]
async fn blind_date_returns_eligible_fic_and_respects_exclude() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["blindtest_a", "blindtest_b"];
    seed_fic(
        &db,
        "blindtest_a",
        "Blind Test Alpha",
        "A brave knight fights a dragon.",
        5000,
        5,
        "complete",
    )
    .await;
    seed_fic(
        &db,
        "blindtest_b",
        "Blind Test Beta",
        "Two bakers open a shop in a small town.",
        2000,
        3,
        "ongoing",
    )
    .await;

    let tag_fluff = seed_tag(&db, "BlindTest Fluff", 4).await; // freeform
    let tag_angst = seed_tag(&db, "BlindTest Angst", 4).await;
    let tag_char = seed_tag(&db, "BlindTest Knight", 2).await; // character
    let tag_fandom = seed_tag(&db, "BlindTest Fandom", 1).await; // fandom — must NOT appear in tropes
    seed_fic_tag(&db, "blindtest_a", tag_fluff, 5).await;
    seed_fic_tag(&db, "blindtest_a", tag_angst, 2).await;
    seed_fic_tag(&db, "blindtest_a", tag_char, 8).await;
    seed_fic_tag(&db, "blindtest_a", tag_fandom, 99).await; // highest score, but fandom type → excluded from tropes
    seed_fic_tag(&db, "blindtest_b", tag_fluff, 1).await;

    // (1) No exclude: must be one of the two seeded fics, never null.
    let body = get_blind_date("/api/blind-date").await;
    let fic = body["fic"].as_object().expect("expected a fic object");
    let url_id = fic["url_id"].as_str().unwrap().to_string();
    // The shared live DB holds real fics too, so the pick may be one of
    // ours OR a pre-existing one — what must hold is that it is eligible
    // (non-empty description) and well-formed.
    let desc = fic["description"].as_str().unwrap();
    assert!(!desc.is_empty(), "description must be present");
    assert!(fic["words"].as_i64().is_some(), "words missing");
    assert!(fic["chapters"].as_i64().is_some(), "chapters missing");
    assert!(fic["status"].as_str().is_some(), "status missing");

    // ── THE CORE GUARANTEE ──────────────────────────────────────────
    // The discovery payload must NEVER contain the title, author, source
    // URL or fandom — those are the "hidden fields" revealed on demand.
    // Asserting absence on the raw JSON (not just the typed struct)
    // proves nothing leaks into the network response.
    assert!(
        fic.get("title").is_none(),
        "title must NOT be in the discovery payload: {body}"
    );
    assert!(
        fic.get("author").is_none(),
        "author must NOT be in the discovery payload: {body}"
    );
    assert!(
        fic.get("source").is_none(),
        "source must NOT be in the discovery payload: {body}"
    );
    assert!(
        fic.get("fandom").is_none(),
        "fandom must NOT be in the discovery payload: {body}"
    );
    // The reveal capability is signed, so it can be present without
    // leaking the title.
    let reveal = fic
        .get("reveal")
        .and_then(|r| r.as_object())
        .expect("discovery payload must carry a signed reveal handle");
    assert!(reveal.get("nonce").is_some(), "nonce missing");
    assert!(reveal.get("sig").is_some(), "sig missing");

    // Build the exclude list: our earlier random pick (url_id) plus every
    // fic in the DB that is NOT one of our two seeds. Excluding by a
    // denylist of ids — rather than by a "good description" predicate —
    // makes the forced pick deterministic: after excluding, only
    // blindtest_a and blindtest_b remain eligible.
    let other_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM fic_info WHERE id NOT IN ($1, $2)")
            .bind(fics[0])
            .bind(fics[1])
            .fetch_all(&db)
            .await
            .expect("fetch other fics");
    // Exclude the earlier random pick too (so the forced pick must be the
    // OTHER seed).
    let mut forced_excludes = other_ids.clone();
    forced_excludes.push(url_id.clone());
    let mut forced = get_blind_date(&format!(
        "/api/blind-date?exclude={}",
        forced_excludes.join(",")
    ))
    .await;
    // The forced pick must be one of OUR seeded fics (exclude removed
    // every other eligible fic in the DB, including the earlier pick).
    let forced_fic = forced["fic"]
        .as_object_mut()
        .expect("expected a fic object");
    let forced_id = forced_fic["url_id"].as_str().unwrap().to_string();
    assert!(
        fics.contains(&forced_id.as_str()),
        "expected a seeded fic after excluding everything else, got {forced_id}: {forced}"
    );
    let tropes: Vec<&str> = forced_fic["tropes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    assert!(!tropes.is_empty(), "expected tropes in response: {forced}");
    assert!(tropes.len() <= 3, "at most 3 tropes, got {tropes:?}");
    assert!(
        !tropes.iter().any(|t| *t == "BlindTest Fandom"),
        "fandom tag leaked into tropes: {tropes:?}"
    );

    // (2) Exclude the fic we just got → a DIFFERENT fic must come back.
    // The shared live DB also holds real fics, so "different" is the only
    // guaranteed invariant: never the same fic twice, always one with a
    // description.
    let body2 = get_blind_date(&format!("/api/blind-date?exclude={url_id}")).await;
    let fic2 = body2["fic"].as_object().expect("expected a fic object");
    let url_id2 = fic2["url_id"].as_str().unwrap().to_string();
    assert_ne!(url_id2, url_id, "exclude must not return the same fic");
    let desc2 = fic2["description"].as_str().unwrap();
    assert!(
        !desc2.is_empty(),
        "excluded pick must still have a description"
    );

    cleanup(
        &db,
        &fics,
        &[
            "BlindTest Fluff",
            "BlindTest Angst",
            "BlindTest Knight",
            "BlindTest Fandom",
        ],
    )
    .await;
}

/// Excluding BOTH seeded fics (leaving nothing eligible in the archive for
/// the query to consider, or only real fics) must still return a well-formed
/// 200: either a fic object or the `fic: null` "nothing eligible" shape.
#[ignore]
#[tokio::test]
async fn blind_date_excluding_everything_is_well_formed() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["blindtest_c", "blindtest_d"];
    seed_fic(
        &db,
        "blindtest_c",
        "Blind Test Gamma",
        "A story about gardens.",
        1000,
        1,
        "complete",
    )
    .await;
    seed_fic(
        &db,
        "blindtest_d",
        "Blind Test Delta",
        "A story about rivers.",
        1500,
        1,
        "ongoing",
    )
    .await;

    let body = get_blind_date("/api/blind-date?exclude=blindtest_c,blindtest_d").await;
    assert_eq!(body["err"], 0, "must be a well-formed 200: {body}");
    assert!(
        body["fic"].is_null() || body["fic"].as_object().is_some(),
        "expected fic object or null, got: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

/// The reveal endpoint returns the hidden title + fandom ONLY with a valid
/// signed handle. This test seeds a fic with a fandom tag, draws it via the
/// discovery endpoint (excluding everything else so the pick is ours),
/// then calls `/api/blind-date/reveal` with the nonce+sig from the payload
/// and asserts the title + fandom come back.
#[ignore]
#[tokio::test]
async fn blind_date_reveal_returns_title_and_fandom_with_valid_signature() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["blindtest_reveal"];
    seed_fic(
        &db,
        "blindtest_reveal",
        "Blind Test Revealed Title",
        "A story about hidden treasure.",
        9000,
        9,
        "complete",
    )
    .await;
    let tag_fandom = seed_tag(&db, "BlindTest Fandom", 1).await;
    let tag_freeform = seed_tag(&db, "BlindTest Fluff", 4).await;
    seed_fic_tag(&db, "blindtest_reveal", tag_fandom, 50).await;
    seed_fic_tag(&db, "blindtest_reveal", tag_freeform, 5).await;

    // Exclude every other fic so the draw MUST be our seeded fic.
    let other_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM fic_info WHERE id <> $1")
        .bind(fics[0])
        .fetch_all(&db)
        .await
        .expect("fetch other fics");
    let mut excludes = other_ids.clone();
    excludes.push("zzblindtest_reveal_never_exists".to_string()); // ensure non-empty
    let body = get_blind_date(&format!("/api/blind-date?exclude={}", excludes.join(","))).await;
    let fic = body["fic"].as_object().expect("expected our seeded fic");
    assert_eq!(
        fic["url_id"].as_str(),
        Some("blindtest_reveal"),
        "exclude-all should force our seed, got: {body}"
    );
    // Discovery still hides the title even when we force our own seed.
    assert!(
        fic.get("title").is_none(),
        "title leaked pre-reveal: {body}"
    );
    assert!(
        fic.get("fandom").is_none(),
        "fandom leaked pre-reveal: {body}"
    );

    let nonce = fic["reveal"]["nonce"].as_str().unwrap();
    let sig = fic["reveal"]["sig"].as_str().unwrap();
    let reveal_uri = format!(
        "/api/blind-date/reveal?url_id=blindtest_reveal&nonce={}&sig={}",
        nonce, sig
    );
    let reveal_body = get_blind_date(&reveal_uri).await;
    assert_eq!(reveal_body["err"], 0, "reveal failed: {reveal_body}");
    let rfic = reveal_body["fic"].as_object().expect("reveal fic object");
    assert_eq!(
        rfic["title"].as_str(),
        Some("Blind Test Revealed Title"),
        "title must come back on reveal: {reveal_body}"
    );
    assert_eq!(
        rfic["fandom"].as_str(),
        Some("BlindTest Fandom"),
        "fandom must come back on reveal: {reveal_body}"
    );

    cleanup(&db, &fics, &["BlindTest Fandom", "BlindTest Fluff"]).await;
}

/// A reveal with a WRONG signature (or a missing one) must be rejected —
/// the hidden fields stay hidden and the handler returns a 400.
#[ignore]
#[tokio::test]
async fn blind_date_reveal_rejects_invalid_signature() {
    let _guard = db_guard();
    let db = pool().await;
    let fics = ["blindtest_bad_sig"];
    seed_fic(
        &db,
        "blindtest_bad_sig",
        "Blind Test Bad Sig",
        "A story that must stay secret.",
        1000,
        1,
        "complete",
    )
    .await;

    // Wrong signature → 400, no title leaked.
    let uri = "/api/blind-date/reveal?url_id=blindtest_bad_sig&nonce=abc&sig=deadbeef";
    let app = app().await;
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        body.get("title").is_none(),
        "title must not leak on a bad signature: {body}"
    );
    assert!(
        body.get("fandom").is_none(),
        "fandom must not leak on a bad signature: {body}"
    );

    cleanup(&db, &fics, &[]).await;
}

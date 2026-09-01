//! DB-gated integration tests for the RSS/Atom feeds:
//! `/feed.xml`, `/feed/follows.xml`, `/feed/works/{url_id}.xml`.
//!
//! Conventions (same as `tests/opds_api.rs` and `tests/follow_updates_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test rss_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

const USERNAME: &str = "rss_follow_test_user";
const TITLE_PREFIX: &str = "RssFeedTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

fn test_config() -> fichub::config::Config {
    fichub::config::Config::from_env()
}

/// Build a router with just the RSS feed routes + real AppState.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = test_config();
    let db = pool().await;

    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("invalid REDIS_URL for test");
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
        health_redis: redis_client.get_multiplexed_async_connection().await.expect("health redis conn"),
        http_client: http_client.clone(),
        scraper_registry: scraper_registry.clone(),
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(fichub::limiter::redis_bucket::RedisBucketLimiter::new(
            redis_client
                .get_multiplexed_async_connection()
                .await
                .expect("redis"),
            false,
        )
        .await
        .expect("rate limiter")),
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
        .route("/feed.xml", get(fichub::routes::rss::new_arrivals_feed))
        .route(
            "/feed/follows.xml",
            get(fichub::routes::rss::follows_feed),
        )
        .route(
            "/feed/works/{url_id}",
            get(fichub::routes::rss::work_feed),
        )
        .with_state(state)
}

/// Issue GET and return (status, content-type, body text).
async fn get_raw(app: &Router, uri: &str) -> (StatusCode, String, String) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let ct = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap_or("").to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, ct, String::from_utf8_lossy(&bytes).to_string())
}

/// JWT for the given user id (real token, real JWT_SECRET from env).
fn auth_token(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: USERNAME.into(),
        role: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    fichub::routes::auth::create_token(&user, &secret).expect("token creation")
}

/// Seed a user; returns its id. Idempotent.
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
        .expect("seed_user lookup failed")
}

/// Seed a work + fic_info pair (title prefixed with TITLE_PREFIX); returns
/// (work_id, url_id). Idempotent like follow_updates_api::seed_work.
async fn seed_work(
    pool: &sqlx::PgPool,
    url_id: &str,
    title: &str,
    author: &str,
    updated_days_ago: i64,
) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 4, 8000, 'desc & more <tagged>', NOW() - interval '30 days',
                  NOW() - ($4 || ' days')::interval, 'ongoing',
                  'https://archiveofourown.org/works/rss-1',
                  NULL, NULL, 4242, 4242, NULL)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .bind(updated_days_ago)
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

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM follows WHERE follower_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM notifications WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'RssFeedTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'RssFeedTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(USERNAME)
        .execute(pool)
        .await;
}

fn feed_title(body: &str) -> Option<String> {
    let start = body.find("<title>")?;
    let end = body[start + 7..].find("</title>")?;
    Some(body[start + 7..start + 7 + end].to_string())
}

fn entry_titles(body: &str) -> Vec<String> {
    body.split("<entry>")
        .skip(1)
        .filter_map(|chunk| {
            let s = chunk.find("<title>")?;
            let e = chunk[s + 7..].find("</title>")?;
            Some(chunk[s + 7..s + 7 + e].to_string())
        })
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────

/// /feed.xml renders a valid Atom feed with the seeded fic as an entry.
#[tokio::test]
#[ignore]
async fn new_arrivals_feed_renders_atom() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_user(&db, USERNAME).await;
    seed_work(&db, "rss-new-1", &format!("{TITLE_PREFIX} New One"), "Rss Author", 3).await;

    let app = app().await;
    let (status, ct, body) = get_raw(&app, "/feed.xml").await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        ct.starts_with("application/atom+xml"),
        "content-type must be atom+xml, got: {ct}"
    );
    assert!(body.starts_with("<?xml"), "must be XML: {}", &body[..body.len().min(60)]);
    assert!(body.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\""));
    assert!(body.contains("rel=\"self\""));
    assert!(body.contains("/feed.xml"));
    let title = feed_title(&body).expect("feed title");
    assert!(title.contains("New Arrivals"), "feed title: {title}");
    assert!(
        body.contains("RssFeedTest New One"),
        "seeded fic should appear as an entry: {body}"
    );
    // Entry shape: id/updated/alternate links present, XML-escaped summary
    assert!(body.contains("urn:fichub:fic:rss-new-1"));
    assert!(body.contains("rel=\"alternate\" href=\"/fic/rss-new-1\""));
    assert!(body.contains("desc &amp; more &lt;tagged&gt;"));
    // Not a JSON error envelope
    assert!(!body.trim_start().starts_with('{'));
}

/// /feed/works/{url_id}.xml renders a single-fic feed.
#[tokio::test]
#[ignore]
async fn per_fic_feed_renders_single_entry() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_user(&db, USERNAME).await;
    seed_work(&db, "rss-perfic-1", &format!("{TITLE_PREFIX} Solo"), "Rss Author", 5).await;

    let app = app().await;
    let (status, ct, body) = get_raw(&app, "/feed/works/rss-perfic-1.xml").await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(ct.starts_with("application/atom+xml"), "got: {ct}");
    assert!(body.contains("<feed "));
    let titles = entry_titles(&body);
    assert_eq!(titles.len(), 1, "exactly one entry: {titles:?}");
    assert_eq!(titles[0], format!("{TITLE_PREFIX} Solo"));
    assert!(body.contains("urn:fichub:feed:work:rss-perfic-1"));
    // Self link points at the canonical .xml URL (absolute when OPDS_BASE_URL
    // is configured in the test env, relative otherwise).
    assert!(
        body.contains("/feed/works/rss-perfic-1.xml"),
        "self link must point at the per-fic feed: {body}"
    );
}

/// Unknown url_id → JSON 404 (AppError::NotFound convention).
#[tokio::test]
#[ignore]
async fn per_fic_feed_unknown_fic_404() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let app = app().await;
    let (status, _ct, body) = get_raw(&app, "/feed/works/does-not-exist-xyz.xml").await;

    assert_eq!(status, StatusCode::NOT_FOUND, "expected 404, got {status}: {body}");
    assert!(body.contains("\"err\""));
}

/// /feed/follows.xml without a token → 401 JSON (project auth convention).
#[tokio::test]
#[ignore]
async fn follows_feed_requires_token() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let app = app().await;
    let (status, _ct, body) = get_raw(&app, "/feed/follows.xml").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "expected 401, got {status}: {body}");
    assert!(
        body.contains("401"),
        "auth-required returns err 401: {body}"
    );
    assert!(!body.starts_with("<?xml"), "must NOT be an XML feed: {body}");
}

/// /feed/follows.xml with an invalid token → 401 as well.
#[tokio::test]
#[ignore]
async fn follows_feed_rejects_bad_token() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let app = app().await;
    let (status, _ct, body) = get_raw(&app, "/feed/follows.xml?token=not-a-real-token").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "expected 401, got {status}: {body}");
    assert!(body.contains("401"));
}

/// /feed/follows.xml with a valid token renders the followed fic.
#[tokio::test]
#[ignore]
async fn follows_feed_with_token_renders_followed_fic() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_id, _url_id) = seed_work(
        &db,
        "rss-follow-1",
        &format!("{TITLE_PREFIX} Followed"),
        "Rss Author",
        2,
    )
    .await;

    sqlx::query("INSERT INTO follows (follower_id, work_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(work_id)
        .execute(&db)
        .await
        .expect("insert follow failed");

    let app = app().await;
    let token = auth_token(user_id);
    let (status, ct, body) = get_raw(&app, &format!("/feed/follows.xml?token={token}")).await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(ct.starts_with("application/atom+xml"), "got: {ct}");
    assert!(
        body.contains("RssFeedTest Followed"),
        "followed fic must appear: {body}"
    );
    assert!(
        body.contains("urn:fichub:feed:follows"),
        "follows feed id: {body}"
    );
}

/// Followed *author* rows also feed the follows feed (author_name expansion).
#[tokio::test]
#[ignore]
async fn follows_feed_includes_followed_author_fics() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    seed_work(
        &db,
        "rss-auth-1",
        &format!("{TITLE_PREFIX} ByFollowedAuthor"),
        "Rss Author Followed",
        1,
    )
    .await;

    sqlx::query("INSERT INTO follows (follower_id, author_name) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Rss Author Followed")
        .execute(&db)
        .await
        .expect("insert author follow failed");

    let app = app().await;
    let token = auth_token(user_id);
    let (status, _ct, body) = get_raw(&app, &format!("/feed/follows.xml?token={token}")).await;

    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {body}");
    assert!(
        body.contains("RssFeedTest ByFollowedAuthor"),
        "fic by followed author must appear: {body}"
    );
}

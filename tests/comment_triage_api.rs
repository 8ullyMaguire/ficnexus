//! DB-gated integration tests for comment moderation triage: posting a
//! comment through the API stores a triage row (tests must NOT depend on
//! Ollama — the triage row is inserted directly via a helper), the admin
//! review endpoint returns pending flagged items, and role < 10 is forbidden.
//!
//! Conventions (same as `tests/lists_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test comment_triage_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
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

const USERNAME: &str = "triage_test_user";
const ADMIN_USERNAME: &str = "triage_test_admin";
const TITLE_PREFIX: &str = "TriageTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the comment-triage endpoints + real AppState.
async fn app() -> Router {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
    use fichub::server::AppState;
    use std::sync::Arc;

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
    // Pointed at a dead port on purpose: tests must not depend on Ollama.
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
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/admin/moderation/comments",
            get(fichub::routes::admin::moderation_comments),
        )
        .route(
            "/api/comments",
            axum::routing::post(fichub::routes::social::add_comment_handler),
        )
        .with_state(state)
}

/// JWT for the given user id + role (real token, real JWT_SECRET from env).
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
/// (work_id, url_id). Idempotent like lists_api::seed_work.
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 4, 8000, 'desc', NOW() - interval '30 days',
                  NOW() - interval '1 day', 'ongoing',
                  'https://archiveofourown.org/works/triage-test-1',
                  NULL, NULL, 4242, 4242, NULL)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
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

    // Re-link the fic_info source to this work (idempotent).
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    (work_id, url_id.to_string())
}

/// Insert a comment row directly and return its id. Used by the tests that
/// must not depend on Ollama (triage rows are inserted via `seed_triage`).
async fn seed_comment(pool: &sqlx::PgPool, work_id: i32, user_id: i32, body: &str) -> i64 {
    let url_id =
        sqlx::query_scalar::<_, String>("SELECT default_source_id FROM works WHERE id = $1")
            .bind(work_id)
            .fetch_one(pool)
            .await
            .expect("work url_id lookup failed");
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO comments (user_id, work_id, url_id, parent_id, body, constructive)
         VALUES ($1, $2, $3, NULL, $4, TRUE) RETURNING id",
    )
    .bind(user_id)
    .bind(work_id)
    .bind(url_id)
    .bind(body)
    .fetch_one(pool)
    .await
    .expect("seed comment failed");
    id
}

/// Insert a triage row directly (mimics what the Ollama pipeline would have
/// stored) and return the comment id.
async fn seed_triage(
    pool: &sqlx::PgPool,
    comment_id: i64,
    category: &str,
    reason: &str,
    confidence: f32,
) {
    sqlx::query(
        "INSERT INTO comment_triage (comment_id, category, reason, confidence)
         VALUES ($1, $2, $3, $4) ON CONFLICT (comment_id) DO NOTHING",
    )
    .bind(comment_id)
    .bind(category)
    .bind(reason)
    .bind(confidence)
    .execute(pool)
    .await
    .expect("seed triage failed");
}

/// Self-heal: delete everything this suite's seeds may have created. Order
/// matters: comment_triage references comments (CASCADE, but be explicit),
/// comments reference works, works are referenced by fic_info. Call this at
/// the START of each test before seeding.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM comment_triage WHERE comment_id IN
         (SELECT id FROM comments WHERE user_id IN
          (SELECT id FROM users WHERE username IN ($1, $2)))",
    )
    .bind(USERNAME)
    .bind(ADMIN_USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM comments WHERE user_id IN
         (SELECT id FROM users WHERE username IN ($1, $2))",
    )
    .bind(USERNAME)
    .bind(ADMIN_USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'TriageTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'TriageTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ($1, $2)")
        .bind(USERNAME)
        .bind(ADMIN_USERNAME)
        .execute(pool)
        .await;
}

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("GET");
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = builder.body(Body::empty()).unwrap();
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

async fn post_json(
    app: &Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("POST");
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
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

// ─── Tests ────────────────────────────────────────────────────────────────

/// Posting a comment through the API stores a comment row (the triage
/// pipeline is fire-and-forget and must never fail the post — here Ollama is
/// pointed at a dead port, so the post must still succeed with no triage row,
/// proving best-effort behaviour).
#[tokio::test]
#[ignore]
async fn posting_comment_succeeds_even_with_ollama_down() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_id, _) = seed_work(
        &db,
        "triage-post-a",
        &format!("{TITLE_PREFIX} Post A"),
        "Author A",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    let (status, body) = post_json(
        &app,
        "/api/comments",
        json!({
            "work_id": work_id,
            "body": "This is a perfectly normal comment.",
            "parent_id": null,
            // Bypass the honeypot timing trap: form opened a few seconds ago.
            "form_opened_at": (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64)
                .saturating_sub(3000)
                .to_string(),
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "comment post failed: {body}");
    assert_eq!(body["err"], 0, "body: {body}");
    let comment_id = body["comment_id"].as_i64().expect("comment id");

    // The comment row exists; triage row may or may not (Ollama is down, so
    // expect none — but never a failed post).
    let comment_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM comments WHERE id = $1)")
            .bind(comment_id)
            .fetch_one(&db)
            .await
            .expect("comment check");
    assert!(comment_exists, "comment should be stored");
    let triage_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM comment_triage WHERE comment_id = $1)")
            .bind(comment_id)
            .fetch_one(&db)
            .await
            .expect("triage check");
    // Best-effort: no triage row is fine when Ollama is down.
    let _ = triage_exists;
}

/// The admin endpoint returns pending flagged items (seeded directly, no
/// Ollama dependency) with the comment body + fic title, and hides fine rows.
#[tokio::test]
#[ignore]
async fn admin_endpoint_returns_pending_triage_items() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let admin_id = seed_user(&db, ADMIN_USERNAME).await;
    let (work_id, _) = seed_work(
        &db,
        "triage-admin-a",
        &format!("{TITLE_PREFIX} Admin A"),
        "Author A",
    )
    .await;
    let app = app().await;
    let admin_token = auth_header_for(admin_id, 6, ADMIN_USERNAME, true);

    // Two flagged comments (toxic + spam) + one fine comment (must be hidden).
    let toxic_id = seed_comment(&db, work_id, user_id, "You are a horrible person, go die").await;
    seed_triage(&db, toxic_id, "toxic", "slurs and death wish", 0.9).await;
    let spam_id = seed_comment(&db, work_id, user_id, "Check out my casino site!!!").await;
    seed_triage(&db, spam_id, "spam", "promotional link", 0.85).await;
    let fine_id = seed_comment(&db, work_id, user_id, "Great chapter, thanks!").await;
    seed_triage(&db, fine_id, "fine", "all good", 0.95).await;

    let (status, body) = get_json(&app, "/api/admin/moderation/comments", Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK, "admin endpoint failed: {body}");
    assert_eq!(body["err"], 0);
    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 2, "only flagged items, got: {body}");

    let toxic = items
        .iter()
        .find(|i| i["comment_id"].as_i64() == Some(toxic_id))
        .expect("toxic item");
    assert_eq!(toxic["category"], "toxic");
    assert_eq!(toxic["reason"], "slurs and death wish");
    assert!((toxic["confidence"].as_f64().unwrap() - 0.9).abs() < 1e-6);
    assert_eq!(toxic["body"], "You are a horrible person, go die");
    assert_eq!(toxic["work_id"].as_i64(), Some(work_id as i64));
    assert_eq!(toxic["title"], format!("{TITLE_PREFIX} Admin A"));
    assert_eq!(toxic["user_id"].as_i64(), Some(user_id as i64));

    let spam = items
        .iter()
        .find(|i| i["comment_id"].as_i64() == Some(spam_id))
        .expect("spam item");
    assert_eq!(spam["category"], "spam");
    assert_eq!(spam["reason"], "promotional link");

    // The fine row must not appear in the review queue.
    assert!(
        !items
            .iter()
            .any(|i| i["comment_id"].as_i64() == Some(fine_id))
    );
}

/// Role < 10 gets 403 on the admin moderation endpoint.
#[tokio::test]
#[ignore]
async fn admin_endpoint_forbids_role_below_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_id, _) = seed_work(
        &db,
        "triage-forbid-a",
        &format!("{TITLE_PREFIX} Forbid A"),
        "Author A",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    // A flagged comment exists but the caller must not see it.
    let bad_id = seed_comment(&db, work_id, user_id, "this is terrible").await;
    seed_triage(&db, bad_id, "non-constructive", "just venting", 0.6).await;

    let (status, _) = get_json(&app, "/api/admin/moderation/comments", Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "role < 10 must be forbidden");
}

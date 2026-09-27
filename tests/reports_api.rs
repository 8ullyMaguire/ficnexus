//! DB-gated integration tests for the user-reports feature:
//! `POST /api/reports` (logged-in user files a report),
//! `GET /api/admin/reports?status=` (role >= 10 lists reports),
//! `POST /api/admin/reports/{id}/resolve` (role >= 10 resolves/dismisses).
//!
//! Conventions (same as `tests/admin_api.rs` / `tests/comment_triage_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test reports_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.

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
    http::{Request, StatusCode, header},
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

const USERNAME: &str = "reports_test_user";
const ADMIN_USERNAME: &str = "reports_test_admin";
const TITLE_PREFIX: &str = "ReportsTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the report endpoints + real AppState.
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
        jwt_secret: TEST_JWT_SECRET.into(),
        redis_client: None,
    });

    Router::new()
        .route("/api/reports", post(fichub::routes::reports::create_report))
        .route(
            "/api/admin/reports",
            get(fichub::routes::reports::list_reports),
        )
        .route(
            "/api/admin/reports/{id}/resolve",
            post(fichub::routes::reports::resolve_report),
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
    let secret = TEST_JWT_SECRET;
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

/// Seed a user with the given role; returns its id. Idempotent.
/// Seeds a user at an explicit database `trust_level`.
///
/// `create_report` calls `assert_staff_or_min_trust(..., 1, "Filing reports")`,
/// which reads `users.trust_level` **from the database** (`src/services/trust.rs:102`),
/// so the JWT's trust level does not move it. This helper used to write the
/// separate legacy `users.role` column, which no trust gate reads, so every
/// report attempt 403'd. Same defect as `uploads_api` and `admin_api`; see
/// `docs/specs/reports-suite.md` section 3.
async fn seed_user_at_trust(pool: &sqlx::PgPool, username: &str, trust_level: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, trust_level) VALUES ($1, 'test-hash', $2) ON CONFLICT (username) DO UPDATE SET trust_level = EXCLUDED.trust_level RETURNING id")
        .bind(username)
        .bind(trust_level)
        .fetch_one(pool)
        .await
        .expect("seed_user failed")
        .get(0)
}

/// Seed a work + fic_info pair; returns (work_id, url_id).
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 4, 8000, 'desc', NOW() - interval '30 days',
                  NOW() - interval '1 day', 'ongoing',
                  'https://example.com/reports-test-1',
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

    let existing: Option<i32> =
        sqlx::query_scalar("SELECT id FROM works WHERE default_source_id = $1")
            .bind(url_id)
            .fetch_optional(pool)
            .await
            .expect("work lookup failed");

    let work_id = match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE works SET canonical_title = $1, canonical_author = $2 WHERE id = $3",
            )
            .bind(title)
            .bind(author)
            .bind(id)
            .execute(pool)
            .await
            .expect("update seeded work failed");
            id
        }
        None => sqlx::query_scalar(
            "INSERT INTO works (canonical_title, canonical_author, default_source_id)
                 VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(title)
        .bind(author)
        .bind(url_id)
        .fetch_one(pool)
        .await
        .expect("seed work failed"),
    };

    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    (work_id, url_id.to_string())
}

/// Insert a report row directly and return its id.
async fn seed_report(
    pool: &sqlx::PgPool,
    reporter_id: i32,
    target_type: &str,
    target_id: i32,
    reason: &str,
    status: &str,
) -> i64 {
    sqlx::query_scalar(
        r#"INSERT INTO user_reports (reporter_id, target_type, target_id, reason, status)
           VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
    )
    .bind(reporter_id)
    .bind(target_type)
    .bind(target_id)
    .bind(reason)
    .bind(status)
    .fetch_one(pool)
    .await
    .expect("seed report failed")
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    // Reports reference users (reporter_id SET NULL on delete), so delete
    // reports first, then works/fics, then users.
    let _ = sqlx::query(
        "DELETE FROM user_reports WHERE reporter_id IN
         (SELECT id FROM users WHERE username IN ($1, $2))",
    )
    .bind(USERNAME)
    .bind(ADMIN_USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'ReportsTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'ReportsTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ($1, $2)")
        .bind(USERNAME)
        .bind(ADMIN_USERNAME)
        .execute(pool)
        .await;
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
    }
    let req = match body {
        Some(v) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
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

/// A logged-in user can file a work report; it lands in user_reports as
/// 'open' with the reporter attached. Anonymous is rejected (err 401).
#[tokio::test]
#[ignore]
async fn logged_in_user_files_work_report() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user_at_trust(&db, USERNAME, 2).await;
    let (work_id, _) = seed_work(
        &db,
        "reports-post-a",
        &format!("{TITLE_PREFIX} Post A"),
        "Character A",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    let (status, body) = send(
        &app,
        "POST",
        "/api/reports",
        Some(&token),
        Some(json!({
            "target_type": "work",
            "target_id": work_id,
            "reason": "Wrong author listed",
            "details": { "expected_author": "Character B" },
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "report post failed: {body}");
    assert_eq!(body["err"], 0);
    let report_id = body["report_id"].as_i64().expect("report id");

    let row: (i32, String, i32, String, Option<Value>, String) = sqlx::query_as(
        "SELECT reporter_id, target_type, target_id, reason, details, status FROM user_reports WHERE id = $1",
    )
    .bind(report_id)
    .fetch_one(&db)
    .await
    .expect("report row");
    assert_eq!(row.0, user_id);
    assert_eq!(row.1, "work");
    assert_eq!(row.2, work_id);
    assert_eq!(row.3, "Wrong author listed");
    assert_eq!(
        row.4.as_ref().and_then(|v| v["expected_author"].as_str()),
        Some("Character B")
    );
    assert_eq!(row.5, "open");

    // Anonymous → login required.
    let (status, body) = send(
        &app,
        "POST",
        "/api/reports",
        None,
        Some(json!({ "target_type": "work", "target_id": work_id, "reason": "x" })),
    )
    .await;
    assert_eq!(body["err"], 401, "anonymous must be login-required: {body}");
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Invalid target_type and empty reason are rejected.
#[tokio::test]
#[ignore]
async fn report_creation_validates_input() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user_at_trust(&db, USERNAME, 2).await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    let (status, body) = send(
        &app,
        "POST",
        "/api/reports",
        Some(&token),
        Some(json!({ "target_type": "chapter", "target_id": 1, "reason": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "bad target_type: {body}");
    assert_eq!(body["err"], -1);

    let (status, body) = send(
        &app,
        "POST",
        "/api/reports",
        Some(&token),
        Some(json!({ "target_type": "work", "target_id": 1, "reason": "   " })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty reason: {body}");
    assert_eq!(body["err"], -1);
}

/// Admin lists open reports (seeded directly) and can resolve/dismiss them;
/// resolved reports leave the open list.
#[tokio::test]
#[ignore]
async fn admin_lists_and_resolves_reports() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user_at_trust(&db, USERNAME, 2).await;
    // DB trust 6, not 10: users_trust_level_check is CHECK (trust_level >= 0 AND
    // trust_level <= 6), so 10 is not storable. It does not need to be -
    // list_reports authorizes on `user.trust_level` from the JWT, which
    // auth_header mints at 10, and the DB row only needs to exist.
    let admin_id = seed_user_at_trust(&db, ADMIN_USERNAME, 6).await;
    let (work_id, _) = seed_work(
        &db,
        "reports-admin-a",
        &format!("{TITLE_PREFIX} Admin A"),
        "Character A",
    )
    .await;
    let app = app().await;
    let admin_token = auth_header_for(admin_id, 6, ADMIN_USERNAME, true);

    let open_id = seed_report(&db, user_id, "work", work_id, "Wrong author", "open").await;
    let dismissed_id = seed_report(&db, user_id, "work", work_id, "Spam", "dismissed").await;

    // Default status=open returns only the open one.
    let (status, body) = get_json(&app, "/api/admin/reports", Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK, "list failed: {body}");
    assert_eq!(body["err"], 0);
    // Find OUR report by id. A count assertion here would assert that the table
    // holds exactly one open row, which is a property of an empty database, not
    // of the endpoint - the admin list is meant to return every open report in
    // the system. See docs/specs/reports-suite.md section 2.
    let items = body["items"].as_array().expect("items array");
    let item = items
        .iter()
        .find(|i| i["id"].as_i64() == Some(open_id))
        .unwrap_or_else(|| panic!("open report {open_id} missing from list: {body}"));
    // Absence asserted against the same identity, not against a length.
    assert!(
        !items.iter().any(|i| i["id"].as_i64() == Some(dismissed_id)),
        "dismissed report must not appear under status=open: {body}"
    );
    assert_eq!(item["target_type"], "work");
    assert_eq!(item["target_id"].as_i64(), Some(work_id as i64));
    assert_eq!(item["reason"], "Wrong author");
    assert_eq!(item["status"], "open");
    assert_eq!(item["reporter_id"].as_i64(), Some(user_id as i64));
    assert_eq!(item["reporter_name"], USERNAME);

    // status=all returns both. Same identity-not-count reasoning as the
    // status=open check above: `status=all` is documented to return every report
    // in the system, so the count is a property of the database, not of the
    // filter. What this asserts is that BOTH of this test's reports are present
    // under `all` - which is the actual difference between `all` and `open`.
    let (status, body) = get_json(&app, "/api/admin/reports?status=all", Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK, "list all failed: {body}");
    let all_items = body["items"].as_array().expect("items array");
    for (label, id) in [("open", open_id), ("dismissed", dismissed_id)] {
        assert!(
            all_items.iter().any(|i| i["id"].as_i64() == Some(id)),
            "status=all must include this test's {label} report {id}: {body}"
        );
    }

    // Resolve the open report.
    let (status, body) = send(
        &app,
        "POST",
        &format!("/api/admin/reports/{open_id}/resolve"),
        Some(&admin_token),
        Some(json!({ "action": "resolved" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "resolve failed: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["status"], "resolved");

    // The report just resolved is no longer in the open list.
    //
    // Not "the open list is empty" - the open list contains every open report
    // in the system, and other suites file their own. Asserting `len() == 0`
    // here says nothing about this resolve call: it would pass with the
    // resolve broken and the report still open, as long as some other report
    // happened to be there instead. See docs/specs/reports-suite.md section 2.
    let (_, body) = get_json(&app, "/api/admin/reports", Some(&admin_token)).await;
    let open_items = body["items"].as_array().expect("items array");
    assert!(
        !open_items.iter().any(|i| i["id"].as_i64() == Some(open_id)),
        "resolved report {open_id} must not appear under status=open: {body}"
    );

    // Resolving again → 404 (already handled).
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/admin/reports/{open_id}/resolve"),
        Some(&admin_token),
        Some(json!({ "action": "resolved" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "already-handled must 404");

    // Dismiss works too.
    let (status, _body) = send(
        &app,
        "POST",
        &format!("/api/admin/reports/{dismissed_id}/resolve"),
        Some(&admin_token),
        Some(json!({ "action": "dismissed" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "dismissing a dismissed report must 404"
    );
}

/// Admin report endpoints require role >= 10.
#[tokio::test]
#[ignore]
async fn admin_report_endpoints_require_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user_at_trust(&db, USERNAME, 2).await;
    let (work_id, _) = seed_work(
        &db,
        "reports-forbid-a",
        &format!("{TITLE_PREFIX} Forbid A"),
        "Character A",
    )
    .await;
    let report_id = seed_report(&db, user_id, "work", work_id, "Wrong author", "open").await;
    let app = app().await;
    let user_token = auth_header(user_id, 0, USERNAME);

    let (status, _) = get_json(&app, "/api/admin/reports", Some(&user_token)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "role 0 list must be forbidden"
    );

    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/admin/reports/{report_id}/resolve"),
        Some(&user_token),
        Some(json!({ "action": "resolved" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "role 0 resolve must be forbidden"
    );
}

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    send(app, "GET", uri, token, None).await
}

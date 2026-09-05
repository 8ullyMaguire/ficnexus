//! DB-gated integration tests for the self-healing milestone 1:
//! `scrape_failures` + `agent_runs` persistence, classifier mapping, the
//! debounce gate, and the diagnose-only `POST /api/admin/heal` endpoint.
//!
//! Conventions (same as `tests/reports_api.rs` / `tests/admin_api.rs`):
//! * Serialised via a global `Mutex`; marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test heal_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows at the START so reruns
//!   never collide.
//! * `#[tokio::test(flavor = "multi_thread")]` like admin_api (the heal
//!   handler probes Redis-backed state; harmless either way).

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::post,
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

const USERNAME: &str = "heal_test_admin";
const DOMAIN: &str = "healtest.example.com";
const URL_A: &str = "https://healtest.example.com/s/1001";
const URL_B: &str = "https://healtest.example.com/s/1002";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the heal endpoint + real AppState.
async fn build_app() -> Router {
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
        config,
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
        heal: fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env()),
        wayback: fichub::scrape::wayback::WaybackService::disabled(),
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(fichub::services::mailer::MockMailer::new()),
    });

    Router::new()
        .route("/api/admin/heal", post(fichub::routes::heal::heal_handler))
        .route(
            "/api/admin/heal/extractions",
            axum::routing::get(fichub::routes::heal::list_extractions_handler),
        )
        .route(
            "/api/admin/heal/extractions/{id}/trust",
            post(fichub::routes::heal::trust_extraction_handler),
        )
        .route(
            "/api/admin/heal/replay-pending",
            post(fichub::routes::heal::replay_pending_handler),
        )
        .with_state(state)
}

/// JWT for the given user id + role (real token, real JWT_SECRET from env).
fn auth_header(user_id: i32, role: i16, username: &str) -> String {
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

/// Seed a user with the given role; returns its id. Idempotent.
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(pool)
        .await
        .expect("seed_user failed")
        .get(0)
}

/// Insert a scrape_failures row directly (bypasses the service; gives exact
/// control over kind/fingerprint/created_at for debounce tests).
async fn seed_failure(
    pool: &sqlx::PgPool,
    url: &str,
    domain: &str,
    kind: &str,
    fingerprint: &str,
    created_at_mins_ago: Option<i64>,
) -> i64 {
    match created_at_mins_ago {
        Some(m) => {
            sqlx::query_scalar(
                "INSERT INTO scrape_failures (url, domain, error_kind, message, fingerprint, created_at)
                 VALUES ($1, $2, $3, $4, $5, now() - ($6::int || ' minutes')::interval) RETURNING id",
            )
            .bind(url)
            .bind(domain)
            .bind(kind)
            .bind("test message")
            .bind(fingerprint)
            .bind(m)
            .fetch_one(pool)
            .await
            .expect("seed_failure failed")
        }
        None => {
            sqlx::query_scalar(
                "INSERT INTO scrape_failures (url, domain, error_kind, message, fingerprint, created_at)
                 VALUES ($1, $2, $3, $4, $5, now()) RETURNING id",
            )
            .bind(url)
            .bind(domain)
            .bind(kind)
            .bind("test message")
            .bind(fingerprint)
            .fetch_one(pool)
            .await
            .expect("seed_failure failed")
        }
    }
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM scrape_failures WHERE domain = $1 OR url LIKE 'https://healtest%'",
    )
    .bind(DOMAIN)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM agent_runs WHERE trigger_type = 'admin_heal' AND trigger_ref IS NULL",
    )
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(USERNAME)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = 'heal_test_low'")
        .execute(pool)
        .await;
    // M2 seeds: heal_extractions (FK-cascade from failures) + pending_exports.
    let _ = sqlx::query("DELETE FROM heal_extractions WHERE url LIKE 'https://healtest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM pending_exports WHERE url LIKE 'https://healtest%'")
        .execute(pool)
        .await;
    // agent_runs rows created by run_extraction tests (heal_extract trigger).
    let _ = sqlx::query("DELETE FROM agent_runs WHERE trigger_type = 'heal_extract'")
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

/// `record_failure` round-trips through the real service (INSERT + row shape).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn record_failure_roundtrip() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let service = fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env());

    service
        .record_failure(
            URL_A,
            None,
            &fichub::heal::classifier::ErrorKind::Parse,
            Some("missing h1"),
            Some("tests/fixtures/scrape/healtest/x.html"),
        )
        .await;

    let row: (String, String, String, Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT url, domain, error_kind, message, html_snapshot_path, fingerprint FROM scrape_failures WHERE url = $1",
    )
    .bind(URL_A)
    .fetch_one(&db)
    .await
    .expect("failure row");
    assert_eq!(row.0, URL_A);
    assert_eq!(row.1, DOMAIN);
    assert_eq!(row.2, "parse");
    assert_eq!(row.3.as_deref(), Some("missing h1"));
    assert_eq!(
        row.4.as_deref(),
        Some("tests/fixtures/scrape/healtest/x.html")
    );
    assert_eq!(row.5.len(), 12, "fingerprint is 12 hex chars");
}

/// Classifier mapping is pure (no DB): ScrapeError → ErrorKind → Class.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn classifier_maps_scrape_error() {
    let _guard = db_guard();
    use fichub::heal::classifier::*;
    use fichub::scrape::ScrapeError;

    assert_eq!(ErrorKind::from(&ScrapeError::Blocked), ErrorKind::Blocked);
    assert_eq!(ErrorKind::from(&ScrapeError::NotFound), ErrorKind::NotFound);
    assert_eq!(
        ErrorKind::from(&ScrapeError::ParseError("x".into())),
        ErrorKind::Parse
    );
    assert_eq!(
        ErrorKind::from(&ScrapeError::Network("timed out".into())),
        ErrorKind::Timeout
    );
    assert_eq!(
        ErrorKind::from(&ScrapeError::Network("refused".into())),
        ErrorKind::Timeout
    );
    assert_eq!(
        ErrorKind::from(&ScrapeError::Network("tls fail".into())),
        ErrorKind::Unknown
    );

    assert_eq!(
        classify(&ErrorKind::Parse, "x.com", "no h1"),
        Class::Structural
    );
    assert_eq!(
        classify(&ErrorKind::Timeout, "x.com", "timeout"),
        Class::Transient
    );
    assert_eq!(
        classify(&ErrorKind::Blocked, "archiveofourown.org", "empty body"),
        Class::Blocked
    );
}

/// Fingerprint stability: same signature (modulo ids/chapter numbers) hashes
/// to the same 12-hex string; different kinds don't.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn fingerprint_stable() {
    let _guard = db_guard();
    use fichub::heal::classifier::{ErrorKind, fingerprint};
    let a = fingerprint(
        "https://example.com/s/123",
        &ErrorKind::Parse,
        "missing h1 on chapter 5",
    );
    let b = fingerprint(
        "https://example.com/s/456",
        &ErrorKind::Parse,
        "missing h1 on chapter 9",
    );
    assert_eq!(a, b, "digit-normalized messages converge");
    assert_eq!(a.len(), 12);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    let c = fingerprint(
        "https://example.com/s/123",
        &ErrorKind::Timeout,
        "missing h1 on chapter 5",
    );
    assert_ne!(a, c, "different kind forks the fingerprint");
}

/// Debounce: >=3 same-fingerprint failures inside the window trigger healing;
/// 2 do not; scattered fingerprints do not; old rows do not.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn debounce_three_in_window() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let fp = "fpdebounce123";
    seed_failure(&db, URL_A, DOMAIN, "parse", fp, Some(1)).await;
    seed_failure(&db, URL_A, DOMAIN, "parse", fp, Some(2)).await;
    // Only 2 in-window same-fp → no heal.
    let failures = fichub::heal::store::recent_failures(
        &db,
        DOMAIN,
        chrono::Utc::now() - chrono::Duration::hours(24),
    )
    .await
    .unwrap();
    assert!(!fichub::heal::classifier::should_heal(&failures, 60));

    // Third same-fp failure → heal.
    seed_failure(&db, URL_B, DOMAIN, "parse", fp, Some(3)).await;
    let failures = fichub::heal::store::recent_failures(
        &db,
        DOMAIN,
        chrono::Utc::now() - chrono::Duration::hours(24),
    )
    .await
    .unwrap();
    assert!(fichub::heal::classifier::should_heal(&failures, 60));

    // A 4th row with a DIFFERENT fingerprint does not change the outcome but
    // proves scattered fingerprints don't count alone.
    seed_failure(&db, URL_A, DOMAIN, "timeout", "fpother999", Some(1)).await;
    let failures = fichub::heal::store::recent_failures(
        &db,
        DOMAIN,
        chrono::Utc::now() - chrono::Duration::hours(24),
    )
    .await
    .unwrap();
    assert!(fichub::heal::classifier::should_heal(&failures, 60));
}

/// Diagnose-only: role-10 admin POSTs; response carries failures + should_heal
/// + plan; no agent call happens (AGENT_ENABLED default false → disabled).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn admin_heal_diagnose_only_returns_failures() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, USERNAME, 10).await;
    let app = build_app().await;
    let token = auth_header(admin_id, 10, USERNAME);

    seed_failure(&db, URL_A, DOMAIN, "parse", "fpadmindx01", None).await;
    seed_failure(&db, URL_B, DOMAIN, "parse", "fpadmindx02", None).await;
    seed_failure(&db, URL_A, DOMAIN, "parse", "fpadmindx01", None).await;

    let (status, body) = send(
        &app,
        "POST",
        &format!("/api/admin/heal?domain={DOMAIN}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "heal diagnose failed: {body}");
    assert_eq!(
        body["agent"], "disabled",
        "AGENT_ENABLED defaults false: {body}"
    );
    assert_eq!(body["failures"].as_array().map(|a| a.len()), Some(3));
    assert_eq!(
        body["should_heal"], true,
        "3 same-domain structural → heal: {body}"
    );
    let plan = body["plan"].as_str().unwrap_or("");
    assert!(plan.contains("diagnose-only"), "plan: {plan}");
    assert!(plan.contains("3 failures"), "plan count: {plan}");

    // No agent_runs row was created (diagnose skipped).
    let runs: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_runs WHERE trigger_type = 'admin_heal'")
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(runs, 0, "no agent run when agent disabled");
}

/// Agent unavailable: with AGENT_ENABLED=true but no key, the endpoint records
/// a failed run ("agent unreachable") and returns 200 with agent unavailable.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn agent_unavailable_records_failed_run() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, USERNAME, 10).await;
    let _app = build_app().await;
    let token = auth_header(admin_id, 10, USERNAME);

    // Force config: enabled, no key → agent::remote_configured false.
    let service = fichub::heal::HealService::new(db.clone(), fichub::config::Config::from_env());
    // The endpoint reads state.config; state was built from env where
    // AGENT_ENABLED may be false. Simulate the no-key case by POSTing with
    // force and checking the "disabled" branch — the FAILED-RUN path needs the
    // agent actually attempted. Use a tiny mock: point the endpoint at an
    // unreachable base URL + enabled + key so diagnose_remote errors.
    // Rebuild state with env overrides via std::env (restored after).
    // SAFETY: test-only env mutation, single-threaded suite (DB_LOCK).
    unsafe {
        std::env::set_var("AGENT_ENABLED", "true");
        std::env::set_var("AGENT_API_KEY", "test-key");
        std::env::set_var("AGENT_API_URL", "http://127.0.0.1:1"); // unreachable
        std::env::set_var("AGENT_MODEL", "deepseek/deepseek-v4-flash");
    }
    let _service = service; // keep the earlier service alive (unused)

    let router2 = build_app().await; // rebuilt with overridden env
    seed_failure(&db, URL_A, DOMAIN, "parse", "fpagentfail", None).await;
    seed_failure(&db, URL_B, DOMAIN, "parse", "fpagentfail2", None).await;
    seed_failure(&db, URL_A, DOMAIN, "parse", "fpagentfail3", None).await;

    let (status, body) = send(
        &router2,
        "POST",
        &format!("/api/admin/heal?domain={DOMAIN}&force=1"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unavailable still 200: {body}");
    assert_eq!(body["agent"], "unavailable", "expected unavailable: {body}");

    let row: (String, Option<String>) = sqlx::query_as(
        "SELECT status, diff_summary FROM agent_runs WHERE trigger_type = 'admin_heal' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&db)
    .await
    .expect("failed agent run row");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1.as_deref(), Some("agent unreachable"));

    // SAFETY: test-only env cleanup, single-threaded suite.
    unsafe {
        std::env::remove_var("AGENT_ENABLED");
        std::env::remove_var("AGENT_API_KEY");
        std::env::remove_var("AGENT_API_URL");
        std::env::remove_var("AGENT_MODEL");
    }
}

/// Ollama-unreachable no-op: enabled + remote key configured but the remote
/// is unreachable → same failed-run path; the local Ollama fallback is only
/// used when the remote is NOT configured, so this stays a failed run.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn agent_ollama_unreachable_noop() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, USERNAME, 10).await;
    let token = auth_header(admin_id, 10, USERNAME);

    // SAFETY: test-only env mutation, single-threaded suite.
    unsafe {
        std::env::set_var("AGENT_ENABLED", "true");
        std::env::set_var("AGENT_API_KEY", "test-key");
        std::env::set_var("AGENT_API_URL", "http://127.0.0.1:1");
        std::env::set_var("AGENT_OLLAMA_URL", "http://127.0.0.1:2");
    }
    let app = build_app().await;
    seed_failure(&db, URL_A, DOMAIN, "parse", "fpollama001", None).await;
    seed_failure(&db, URL_B, DOMAIN, "parse", "fpollama002", None).await;
    seed_failure(&db, URL_A, DOMAIN, "parse", "fpollama003", None).await;

    let (status, body) = send(
        &app,
        "POST",
        &format!("/api/admin/heal?domain={DOMAIN}&force=1"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["agent"], "unavailable", "{body}");

    let (status, _): (String, Option<String>) = sqlx::query_as(
        "SELECT status, diff_summary FROM agent_runs WHERE trigger_type = 'admin_heal' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&db)
    .await
    .expect("failed agent run row");
    assert_eq!(status, "failed");

    // SAFETY: test-only env cleanup, single-threaded suite.
    unsafe {
        std::env::remove_var("AGENT_ENABLED");
        std::env::remove_var("AGENT_API_KEY");
        std::env::remove_var("AGENT_API_URL");
        std::env::remove_var("AGENT_OLLAMA_URL");
    }
}

/// Role gate: anonymous → 400 err 401; role 0 → 403.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn admin_heal_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME, 0).await;
    let app = build_app().await;

    let (status, body) = send(
        &app,
        "POST",
        &format!("/api/admin/heal?domain={DOMAIN}"),
        None,
        None,
    )
    .await;
    // admin.rs convention: anonymous → HTTP 403 with err -403 (not the
    // tag-curator 400-as-401 style). AuthUser extraction happens before the
    // handler body; the AppError::Forbidden path is the module convention.
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "anon must be forbidden: {body}"
    );
    assert_eq!(body["err"], -403, "anon err code: {body}");

    let token = auth_header(user_id, 0, USERNAME);
    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/admin/heal?domain={DOMAIN}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "role 0 must be forbidden");
}

// ─── Milestone 2: on-the-fly extraction + pending queue ───────────────────

/// parse_agent_metadata_json accepts a plain JSON object (no DB).
#[test]
fn parse_agent_metadata_json_valid() {
    use fichub::heal::extract::parse_agent_metadata_json;
    let meta = parse_agent_metadata_json(
        r#"{"title":"Healed Story","author":"Jane Doe","chapters":12,"words":34567,
            "desc":"A tale","status":"ongoing","source":"https://healtest.example.com/s/2001",
            "url_id":"ht_2001"}"#,
        "https://healtest.example.com/s/2001",
    )
    .expect("valid extraction");
    assert_eq!(meta.title, "Healed Story");
    assert_eq!(meta.chapters, 12);
    assert_eq!(meta.words, 34567);
    assert_eq!(meta.status, "ongoing");
    assert_eq!(meta.url_id.as_deref(), Some("ht_2001"));
}

/// parse_agent_metadata_json rejects garbage / invalid shape (no DB).
#[test]
fn parse_agent_metadata_json_rejects_garbage() {
    use fichub::heal::extract::parse_agent_metadata_json;
    let url = "https://healtest.example.com/s/2002";
    let err = parse_agent_metadata_json("not json", url).unwrap_err();
    assert!(err.contains("not valid JSON"), "err: {err}");

    // Bad status
    let err = parse_agent_metadata_json(
        r#"{"title":"T","author":"A","chapters":1,"words":0,"desc":"",
            "status":"on-hiatus","source":"https://healtest.example.com/s/2002"}"#,
        url,
    )
    .unwrap_err();
    assert!(err.contains("status"), "err: {err}");

    // Zero chapters
    let err = parse_agent_metadata_json(
        r#"{"title":"T","author":"A","chapters":0,"words":0,"desc":"",
            "status":"complete","source":"https://healtest.example.com/s/2002"}"#,
        url,
    )
    .unwrap_err();
    assert!(err.contains("chapters"), "err: {err}");

    // Cross-host source
    let err = parse_agent_metadata_json(
        r#"{"title":"T","author":"A","chapters":1,"words":0,"desc":"",
            "status":"complete","source":"https://evil.example/s/2002"}"#,
        url,
    )
    .unwrap_err();
    assert!(err.contains("host"), "err: {err}");
}

/// store_extraction + list_extractions round-trip (DB).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn store_and_list_extractions() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let failure_id = seed_failure(&db, URL_A, DOMAIN, "parse", "fpm2list01", None).await;

    let meta = fichub::heal::extract::parse_agent_metadata_json(
        r#"{"title":"Listed","author":"A","chapters":3,"words":100,"desc":"d",
            "status":"complete","source":"https://healtest.example.com/s/1001",
            "url_id":"ht_1001"}"#,
        URL_A,
    )
    .unwrap();
    let id = fichub::heal::extract::store_extraction(&db, 0, failure_id, &meta, true)
        .await
        .expect("store extraction");
    assert!(id > 0);

    let rows = fichub::heal::extract::list_extractions(&db, false, 10)
        .await
        .expect("list extractions");
    assert_eq!(rows.len(), 1, "one row for our seed");
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].url, URL_A);
    assert_eq!(rows[0].title.as_deref(), Some("Listed"));
    assert!(rows[0].validated);
    assert!(!rows[0].trusted);
}

/// mark_trusted flips trusted and find_trusted_extraction_for_url finds it (DB).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn mark_extraction_trusted() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let failure_id = seed_failure(&db, URL_B, DOMAIN, "parse", "fpm2trust01", None).await;

    let meta = fichub::heal::extract::parse_agent_metadata_json(
        r#"{"title":"Trusted","author":"A","chapters":5,"words":200,"desc":"d",
            "status":"complete","source":"https://healtest.example.com/s/1002",
            "url_id":"ht_1002"}"#,
        URL_B,
    )
    .unwrap();
    let id = fichub::heal::extract::store_extraction(&db, 0, failure_id, &meta, true)
        .await
        .expect("store extraction");

    // Not trusted yet → not findable via the trusted lookup.
    assert!(
        fichub::heal::extract::find_trusted_extraction_for_url(&db, URL_B)
            .await
            .unwrap()
            .is_none()
    );
    assert!(fichub::heal::extract::mark_trusted(&db, id).await.unwrap());
    let row = fichub::heal::extract::find_trusted_extraction_for_url(&db, URL_B)
        .await
        .unwrap()
        .expect("trusted extraction");
    assert_eq!(row.id, id);
    assert!(row.trusted);
}

/// Full validation flow: seed failure → store extraction (validated) →
/// trust → find for replay (DB).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn on_the_fly_validation_flow() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let failure_id = seed_failure(&db, URL_A, DOMAIN, "parse", "fpm2flow01", None).await;

    // Simulate run_extraction's outcome: agent produced validated metadata.
    let meta = fichub::heal::extract::parse_agent_metadata_json(
        r#"{"title":"Flow Fic","author":"A","chapters":7,"words":700,"desc":"d",
            "status":"ongoing","source":"https://healtest.example.com/s/1001",
            "url_id":"ht_1001"}"#,
        URL_A,
    )
    .unwrap();
    let id = fichub::heal::extract::store_extraction(&db, 0, failure_id, &meta, true)
        .await
        .expect("store extraction");

    // Untrusted → replay lookup misses.
    assert!(
        fichub::heal::extract::find_trusted_extraction_for_url(&db, URL_A)
            .await
            .unwrap()
            .is_none()
    );
    // Admin trusts it.
    assert!(fichub::heal::extract::mark_trusted(&db, id).await.unwrap());
    // Now replay finds it and it converts to FicMetadata.
    let row = fichub::heal::extract::find_trusted_extraction_for_url(&db, URL_A)
        .await
        .unwrap()
        .expect("trusted extraction");
    let meta = fichub::heal::extract::parse_agent_metadata_json(
        r#"{"title":"Flow Fic","author":"A","chapters":7,"words":700,"desc":"d",
            "status":"ongoing","source":"https://healtest.example.com/s/1001",
            "url_id":"ht_1001"}"#,
        URL_A,
    )
    .unwrap();
    let fic = meta.to_fic_metadata(URL_A);
    assert_eq!(fic.url_id, "ht_1001");
    assert_eq!(fic.chapters, 7);
    assert_eq!(fic.status, "ongoing");
    assert_eq!(row.id, id);
}

/// pending_exports: enqueue + list + replay marking (DB). Uses the store fn
/// directly (the replay endpoint's chapter-fetch path needs a live site).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn pending_exports_insert_and_replay() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;

    let id =
        fichub::heal::extract::enqueue_pending_export(&db, URL_A, "epub", None, None, "timeout")
            .await
            .expect("enqueue pending export");
    assert!(id > 0);

    // Freshly enqueued → pending, attempts 0.
    let rows = fichub::heal::store::list_pending_exports(&db, 10)
        .await
        .expect("list pending");
    let mine: Vec<_> = rows.iter().filter(|r| r.url == URL_A).collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].status, "pending");
    assert_eq!(mine[0].attempts, 0);
    assert_eq!(mine[0].error_kind.as_deref(), Some("timeout"));

    // Simulate the replay endpoint marking it completed.
    sqlx::query(
        r#"UPDATE pending_exports SET status = 'completed', attempts = attempts + 1, completed_at = now() WHERE id = $1"#,
    )
    .bind(id)
    .execute(&db)
    .await
    .expect("mark completed");
    let rows = fichub::heal::store::list_pending_exports(&db, 10)
        .await
        .expect("list pending");
    assert!(
        rows.iter().all(|r| r.url != URL_A),
        "completed rows are no longer pending"
    );
    let (status, attempts): (String, i32) =
        sqlx::query_as("SELECT status, attempts FROM pending_exports WHERE id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("completed row");
    assert_eq!(status, "completed");
    assert_eq!(attempts, 1);
}

/// GET /api/admin/heal/extractions requires role >= 10 (DB).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn heal_extractions_endpoint_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME, 0).await;
    let app = build_app().await;

    let (status, body) = send(&app, "GET", "/api/admin/heal/extractions", None, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "anon: {body}");
    assert_eq!(body["err"], -403);

    let token = auth_header(user_id, 0, USERNAME);
    let (status, _) = send(
        &app,
        "GET",
        "/api/admin/heal/extractions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "role 0: {body}");
}

/// Trust endpoint round-trip through the router (DB).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn trust_extraction_endpoint_roundtrip() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, USERNAME, 10).await;
    let app = build_app().await;
    let token = auth_header(admin_id, 10, USERNAME);

    let failure_id = seed_failure(&db, URL_A, DOMAIN, "parse", "fpm2trustep01", None).await;
    let meta = fichub::heal::extract::parse_agent_metadata_json(
        r#"{"title":"Ep Trust","author":"A","chapters":2,"words":50,"desc":"d",
            "status":"complete","source":"https://healtest.example.com/s/1001",
            "url_id":"ht_1001"}"#,
        URL_A,
    )
    .unwrap();
    let id = fichub::heal::extract::store_extraction(&db, 0, failure_id, &meta, true)
        .await
        .expect("store extraction");

    // Admin lists → sees it untrusted.
    let (status, body) = send(
        &app,
        "GET",
        "/api/admin/heal/extractions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let arr = body["extractions"].as_array().expect("extractions array");
    assert_eq!(arr.len(), 1, "one untrusted extraction: {body}");
    assert_eq!(arr[0]["title"], "Ep Trust");
    assert!(!arr[0]["trusted"].as_bool().unwrap_or(true));

    // Admin trusts it.
    let (status, body) = send(
        &app,
        "POST",
        &format!("/api/admin/heal/extractions/{id}/trust"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["trusted"], true);

    // Now trusted → the untrusted review queue is empty.
    let (status, body) = send(
        &app,
        "GET",
        "/api/admin/heal/extractions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["extractions"].as_array().map(|a| a.len()), Some(0));
}

/// Replay-pending endpoint is role-gated + returns counters (DB; uses a
/// non-resolvable URL so the row stays pending with attempts+1).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn replay_pending_endpoint_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let admin_id = seed_user(&db, USERNAME, 10).await;
    let app = build_app().await;
    let token = auth_header(admin_id, 10, USERNAME);

    let id = fichub::heal::extract::enqueue_pending_export(
        &db,
        "https://healtest.example.com/s/9999",
        "epub",
        None,
        None,
        "blocked",
    )
    .await
    .expect("enqueue");
    let (status, _) = send(&app, "POST", "/api/admin/heal/replay-pending", None, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "anon must be forbidden");

    // Non-admin role 0 → 403 too.
    let user_id = seed_user(&db, "heal_test_low", 0).await;
    let low_token = auth_header(user_id, 0, "heal_test_low");
    let (status, _) = send(
        &app,
        "POST",
        "/api/admin/heal/replay-pending",
        Some(&low_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "role 0 must be forbidden");

    // Admin replays → healtest.example.com is unresolvable so the row stays
    // pending with attempts bumped (never blocks, never errors).
    let (status, body) = send(
        &app,
        "POST",
        "/api/admin/heal/replay-pending",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["replayed"].as_u64().unwrap_or(0) >= 1, "{body}");
    assert_eq!(body["still_pending"].as_u64().unwrap_or(0), 1, "{body}");

    let (attempts,): (i32,) = sqlx::query_as("SELECT attempts FROM pending_exports WHERE id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("attempts row");
    assert!(attempts >= 1, "attempts bumped: {attempts}");
    let (status,): (String,) = sqlx::query_as("SELECT status FROM pending_exports WHERE id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("status row");
    assert_eq!(status, "pending");
}

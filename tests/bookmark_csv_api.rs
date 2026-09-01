//! Integration tests for the bookmark CSV import/export feature
//! (`GET /api/bookmarks/export.csv`, `POST /api/bookmarks/import`).
//!
//! The export handler is exercised through the real Axum router with a real
//! `AppState` (tower `ServiceExt::oneshot`), exactly like the other DB-gated
//! suites. The import handler is a thin enqueue; the actual CSV processing
//! lives in `fichub::services::bookmark_import::process_import`, which we
//! call directly to verify import semantics deterministically, then assert
//! the `bookmark_import` notification that the worker would create.
//!
//! Conventions (same as `tests/search_api.rs` / `tests/recommender_personal.rs`):
//!
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test bookmark_csv_api -- --include-ignored --test-threads=1`
//! * Each test seeds uniquely-named users/works/fics and cleans up exactly
//!   the rows it created.

use std::sync::{Mutex, OnceLock};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
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

/// Build a router with the bookmark export/import routes and a real AppState.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
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
        .route(
            "/api/bookmarks/export.csv",
            get(fichub::routes::social::export_bookmarks_csv),
        )
        .route(
            "/api/bookmarks/import",
            post(fichub::routes::social::import_bookmarks_csv),
        )
        .with_state(state)
}

/// JWT for the given user id (real token, real JWT_SECRET from env).
fn auth_header(user_id: i32) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "bkmk_csv_test_user".into(),
        role: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
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

/// Seed a work + fic_info pair; returns the work id. `url_id` is the
/// fic_info.id (the bookmark's url_id). Idempotent.
async fn seed_work_with_source(
    pool: &sqlx::PgPool,
    url_id: &str,
    title: &str,
    author: &str,
    source: &str,
) -> i32 {
    // Ensure the fic_info row exists first (works.default_source_id FK).
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', $4, NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(author)
    .bind(source)
    .execute(pool)
    .await
    .expect("seed fic_info failed");

    // Insert the work with default_source_id pointing at the fic_info row.
    // No unique constraint exists on (title, author), so check-then-insert.
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

    // Link fic_info.work_id to the work (required by resolve_url_id_for_work).
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    work_id
}

/// Seed a bookmark row directly; idempotent.
async fn seed_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str, work_id: i32) {
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(url_id)
    .bind(work_id)
    .execute(pool)
    .await
    .expect("seed_bookmark failed");
}

/// Clean up every row the tests seed (by unique name prefix).
async fn cleanup(pool: &sqlx::PgPool, username: &str) {
    let _ = sqlx::query(
        "DELETE FROM bookmarks WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(username)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'BkmkCsv%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'BkmkCsv%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM notifications WHERE user_id IN (SELECT id FROM users WHERE username = $1)")
        .bind(username)
        .execute(pool)
        .await;
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// GET /api/bookmarks/export.csv returns a CSV with header + bookmarked rows,
/// content-type text/csv and an attachment content-disposition.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn export_returns_csv_with_header_and_rows() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "bkmkcsv_export_user";
    cleanup(&db, username).await;

    let user_id = seed_user(&db, username).await;
    let work_id = seed_work_with_source(
        &db,
        "bkmkcsv_export_url1",
        "BkmkCsv Export Fic",
        "BkmkCsv Author",
        "https://example.com/export-fic",
    )
    .await;
    seed_bookmark(&db, user_id, "bkmkcsv_export_url1", work_id).await;

    let token = auth_header(user_id);
    let app = app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/bookmarks/export.csv")
                .header("Authorization", token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "export should be 200");
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(
        content_type.contains("text/csv"),
        "content-type should be text/csv, got {content_type}"
    );
    let disposition = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(
        disposition.contains("fichub-bookmarks.csv"),
        "content-disposition should name fichub-bookmarks.csv, got {disposition}"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let csv = String::from_utf8(bytes.to_vec()).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert!(lines.len() >= 2, "CSV should have a header + at least one row: {csv}");
    assert_eq!(
        lines[0], "title,author,source,bookmarked_at",
        "CSV header mismatch"
    );
    assert!(
        lines[1].contains("BkmkCsv Export Fic") && lines[1].contains("BkmkCsv Author"),
        "CSV row should contain the seeded title+author: {}",
        lines[1]
    );

    cleanup(&db, username).await;
}

/// Export requires auth: anonymous gets a 400 business-code 401, not CSV.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn export_requires_auth() {
    let _guard = db_guard();
    let app = app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/bookmarks/export.csv")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "anonymous export should be a 401 (AppError::Unauthorized)"
    );
}

/// POST /api/bookmarks/import enqueues a job and returns `{err:0,status:"queued"}`.
///
/// The production service runs a live `BookmarkImportWorker` that BRPOPs the
/// shared `bookmark_import_queue` immediately, so asserting LLEN on the
/// shared queue is racy (the worker may consume before we check — and it
/// would then try to notify a user the test deletes in cleanup). So this
/// test verifies the enqueue primitive directly on a UNIQUE queue key
/// (unit-level, no worker race) and separately asserts the HTTP handler
/// returns the queued response.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn import_enqueues_job_and_returns_queued() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "bkmkcsv_import_queue_user";
    cleanup(&db, username).await;

    let user_id = seed_user(&db, username).await;
    let token = auth_header(user_id);

    // ── (a) enqueue primitive on a UNIQUE key (no live-worker race) ──
    let config = fichub::config::Config::from_env();
    let client = redis::Client::open(config.redis_url.clone()).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let unique_key = format!("{}_test_{}", fichub::services::bookmark_import::BOOKMARK_IMPORT_QUEUE, user_id);
    let _: () = redis::cmd("DEL")
        .arg(&unique_key)
        .query_async(&mut conn)
        .await
        .unwrap();

    let job = fichub::services::bookmark_import::ImportJob {
        user_id,
        csv: "title,author,source\nSome Fic,Some Author,https://example.com/some-fic\n".to_string(),
    };
    let worker = fichub::services::bookmark_import::BookmarkImportWorker::new(db.clone(), conn.clone());
    worker.enqueue_to(&unique_key, &job).await.expect("enqueue should succeed");
    let len: i64 = redis::cmd("LLEN")
        .arg(&unique_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(len, 1, "enqueue should push exactly one job onto {unique_key}");
    let _: () = redis::cmd("DEL")
        .arg(&unique_key)
        .query_async(&mut conn)
        .await
        .unwrap();

    // ── (b) HTTP handler returns the queued response ──
    let csv = "title,author,source\nSome Fic,Some Author,https://example.com/some-fic\n";
    let body = format!(
        "--X-TEST-BOUNDARY\r\nContent-Disposition: form-data; name=\"file\"; filename=\"b.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--X-TEST-BOUNDARY--\r\n",
        csv
    );

    let app = app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/bookmarks/import")
                .method("POST")
                .header("Authorization", token)
                .header("content-type", "multipart/form-data; boundary=X-TEST-BOUNDARY")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "import should be 200");
    let json: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(json["err"], 0, "err should be 0: {json}");
    assert_eq!(json["status"], "queued", "status should be queued: {json}");

    // Drain the shared queue so the live worker doesn't process a job for a
    // user we're about to delete (the job it would notify FK-violates).
    loop {
        let popped: Option<String> = redis::cmd("LPOP")
            .arg(fichub::services::bookmark_import::BOOKMARK_IMPORT_QUEUE)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);
        if popped.is_none() {
            break;
        }
    }

    cleanup(&db, username).await;
}

/// process_import handles good rows, malformed rows, unresolved works,
/// and duplicates — and reports all three distinctly.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn import_processing_inserts_dedupes_and_reports_errors() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "bkmkcsv_import_proc_user";
    cleanup(&db, username).await;

    let user_id = seed_user(&db, username).await;
    seed_work_with_source(
        &db,
        "bkmkcsv_proc_url1",
        "BkmkCsv Proc Fic A",
        "BkmkCsv Author A",
        "https://example.com/proc-fic-a",
    )
    .await;

    let csv = "\
title,author,source
BkmkCsv Proc Fic A,BkmkCsv Author A,https://example.com/proc-fic-a
BkmkCsv Proc Fic A,BkmkCsv Author A,https://example.com/proc-fic-a
Missing Work,No One,https://example.com/nope
JustTitle
";
    let job = fichub::services::bookmark_import::ImportJob {
        user_id,
        csv: csv.to_string(),
    };
    let res = fichub::services::bookmark_import::process_import(&db, &job)
        .await
        .expect("process_import should succeed");

    assert_eq!(res.imported, 1, "exactly one new bookmark should import: {res:?}");
    assert_eq!(res.duplicates, 1, "the repeated row should count as duplicate: {res:?}");
    assert_eq!(res.errors.len(), 2, "malformed + unresolved rows should error: {res:?}");

    // Verify the bookmark row exists with the resolved url_id + work_id.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookmarks WHERE user_id = $1 AND url_id = 'bkmkcsv_proc_url1'",
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(count, 1, "bookmark row should exist for the imported fic");

    cleanup(&db, username).await;
}

/// A completed import creates a `bookmark_import` notification for the user.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn import_creates_notification_on_completion() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "bkmkcsv_notify_user";
    cleanup(&db, username).await;

    let user_id = seed_user(&db, username).await;
    seed_work_with_source(
        &db,
        "bkmkcsv_notify_url1",
        "BkmkCsv Notify Fic",
        "BkmkCsv Author",
        "https://example.com/notify-fic",
    )
    .await;

    let csv = "title,author,source\nBkmkCsv Notify Fic,BkmkCsv Author,https://example.com/notify-fic\n";
    let job = fichub::services::bookmark_import::ImportJob {
        user_id,
        csv: csv.to_string(),
    };
    let res = fichub::services::bookmark_import::process_import(&db, &job)
        .await
        .expect("process_import should succeed");
    assert_eq!(res.imported, 1, "one fic should import");

    // Drive the worker's post-processing notification path.
    fichub::services::bookmark_import::notify_import_done(&db, &job, &res).await;

    let notif_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND notification_type = 'bookmark_import'",
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(notif_count, 1, "notification should be created on completion");

    let title: String = sqlx::query_scalar(
        "SELECT title FROM notifications WHERE user_id = $1 AND notification_type = 'bookmark_import'",
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert!(
        title.contains("Bookmark import complete") && title.contains("1 imported"),
        "notification title should report the import result: {title}"
    );

    cleanup(&db, username).await;
}

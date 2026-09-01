//! Integration tests for the fichub backend stack.
//!
//! This file contains three test modules:
//! 1. `db_tests` — round-trip database query tests (require live PostgreSQL)
//! 2. `api_tests` — Axum router smoke tests with mock handlers
//! 3. `export_tests` — pure-function tests for slug, info-string and meta-json
//!
//! Database tests are serialised via a global Mutex and marked `#[ignore]` so they
//! are skipped unless the user explicitly runs `cargo test -- --include-ignored`.
//! Each database test creates its own temporary PostgreSQL schema for isolation.

use std::sync::{Mutex, OnceLock};

// ─── Re-exports from the library ────────────────────────────────────────────

use fichub::{
    build_info_string, build_meta_json, generate_slug,
    db::{models::*, queries},
    export,
    scrape::FicMetadata,
};

// ─── Helpers shared across test modules ─────────────────────────────────────

/// Create a `FicMetadata` value with known fields for use in export / API tests.
fn mock_ficmeta() -> FicMetadata {
    FicMetadata {
        url_id: "a1b2c3d4e5f6".into(),
        title: "The Testing of the Rings".into(),
        author: "tolkien_fan".into(),
        chapters: 42,
        words: 123_456,
        desc: "A thrilling tale of test-driven development in Middle-earth.".into(),
        published: 1_700_000_000_000,      // 2023-11-14-ish
        updated: 1_700_100_000_000,
        status: "ongoing".into(),
        source: "https://example.test/story/a1b2c3d4e5f6".into(),
        source_id: 1,
        author_id: 1001,
        author_url: "https://example.test/u/tolkien_fan".into(),
        author_local_id: "tolkien_fan".into(),
        content_hash: Some("abc123def456".into()),
        extra_meta: Some(r#"{"fandom":"Middle-earth"}"#.into()),
        raw_extended_meta: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Module 1 — Database query tests (live PostgreSQL)
// ═══════════════════════════════════════════════════════════════════════════

mod db_tests {
    use super::*;
    use chrono::Utc;
    use sqlx::AssertSqlSafe;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;

    /// Global mutex that serialises ALL database tests so they never step on
    /// each other (each test creates / drops its own schema, but concurrent
    /// migration runs can clash on the `sqlx_migrations` table).
    static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    fn db_lock() -> &'static Mutex<()> {
        DB_LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Helper that provisions a temporary test schema, runs the project
    /// migrations inside it, and returns a `TestDb` handle.
    struct TestDb {
        pool: PgPool,
        schema: String,
    }

    impl TestDb {
        /// Connect to `DATABASE_URL`, create a unique schema, run migrations.
        async fn new() -> Self {
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set for db tests");

            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .expect("failed to connect to test database");

            // Unique schema name per invocation (PID + monotonic ns helps
            // avoid collisions when tests are run sequentially).
            let schema = format!(
                "test_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );

            // Create the schema and set search_path.
            sqlx::raw_sql(AssertSqlSafe(format!(
                "CREATE SCHEMA IF NOT EXISTS \"{}\"",
                schema
            )))
            .execute(&pool)
            .await
            .expect("failed to create test schema");

            sqlx::raw_sql(AssertSqlSafe(format!(
                "SET search_path TO \"{}\", public",
                schema
            )))
            .execute(&pool)
            .await
            .expect("failed to set search_path");

            // Run migrations from the project's migration directory.
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let migrations_path = manifest_dir.join("migrations");
            let migrator = sqlx::migrate::Migrator::new(migrations_path)
                .await
                .expect("failed to load migrations");
            migrator
                .run(&pool)
                .await
                .expect("failed to run migrations");

            TestDb { pool, schema }
        }

        /// Truncate all tables used by the test queries so state does not leak
        /// between tests within the same schema.
        async fn truncate_all(&self) {
            let tables = [
                "fic_info",
                "export_log",
                "fic_blacklist",
                "author_blacklist",
                "fic_version_bump",
                "request_log",
                "request_source",
            ];
            for table in &tables {
                sqlx::raw_sql(AssertSqlSafe(format!(
                    "TRUNCATE TABLE \"{}\".\"{}\" CASCADE",
                    self.schema, table
                )))
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("failed to truncate {table}: {e}"));
            }
        }

        /// Drop the test schema entirely (cleanup).
        async fn cleanup(&self) {
            sqlx::raw_sql(AssertSqlSafe(format!(
                "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
                self.schema
            )))
            .execute(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("failed to drop schema {}: {e}", self.schema));
        }

        #[allow(dead_code)]
        fn schema(&self) -> &str {
            &self.schema
        }
    }

    /// Insert a row into `fic_info` via the real query function.
    async fn insert_fic_info(td: &TestDb) -> FicInfo {
        let fic = FicInfo {
            id: "a1b2c3d4e5f6".into(),
            created: None,
            updated: None,
            title: "Integration Test Fic".into(),
            author: "test_author".into(),
            author_url: Some("https://example.test/u/test_author".into()),
            author_local_id: Some("test_author".into()),
            chapters: 10,
            words: 50_000,
            description: "A test fic for integration testing purposes.".into(),
            fic_created: Utc::now(),
            fic_updated: Utc::now(),
            status: "ongoing".into(),
            source: "https://example.test/story/a1b2c3d4e5f6".into(),
            extra_meta: None,
            raw_extended_meta: None,
            source_id: Some(1),
            author_id: Some(1001),
            content_hash: Some("hash001".into()),
            work_id: None,
        };

        queries::upsert_fic_info(&td.pool, &fic)
            .await
            .expect("upsert_fic_info failed");
        fic
    }

    // ── Individual tests ─────────────────────────────────────────────────

    #[ignore]
    #[tokio::test]
    async fn test_upsert_and_get_fic_info_round_trip() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        let fic = insert_fic_info(&td).await;
        let fetched = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed")
            .expect("expected Some row");

        assert_eq!(fetched.id, fic.id);
        assert_eq!(fetched.title, fic.title);
        assert_eq!(fetched.author, fic.author);
        assert_eq!(fetched.chapters, 10);
        assert_eq!(fetched.words, 50_000);

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_upsert_fic_info_update_existing() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        let mut fic = insert_fic_info(&td).await;

        // Update the title
        fic.title = "Updated Title".into();
        queries::upsert_fic_info(&td.pool, &fic)
            .await
            .expect("second upsert failed");

        let fetched = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed")
            .expect("expected Some row after update");

        assert_eq!(fetched.title, "Updated Title");

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_insert_request_source_and_request_log() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        // Insert fic_info because request_log / fic_blacklist have FK references.
        insert_fic_info(&td).await;

        let source_id = queries::insert_request_source(
            &td.pool, false, "/api/v0/epub", "web request",
        )
        .await
        .expect("insert_request_source failed");

        assert!(source_id > 0);

        queries::insert_request_log(
            &td.pool,
            source_id,
            "epub",
            "https://example.test/story/abc",
            150,                          // info_request_ms
            Some("a1b2c3d4e5f6"),
            Some(r#"{"title":"Test"}"#),
            Some(500),                    // export_ms
            Some("abc123.epub"),
            Some("abc123"),
            Some("https://example.test/story/abc"),
            None, // client_id
            None, // user_agent
            None, // ip
        )
        .await
        .expect("insert_request_log failed");

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_insert_export_log_and_find_export_log() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        queries::insert_export_log(
            &td.pool, "a1b2c3d4e5f6", 1, "epub", "inputhash001", "exporthash001",
        )
        .await
        .expect("insert_export_log failed");

        let found = queries::find_export_log(
            &td.pool, "a1b2c3d4e5f6", 1, "epub", "inputhash001",
        )
        .await
        .expect("find_export_log failed")
        .expect("expected Some export_log");

        assert_eq!(found.url_id, "a1b2c3d4e5f6");
        assert_eq!(found.version, 1);
        assert_eq!(found.etype, "epub");
        assert_eq!(found.input_hash, "inputhash001");
        assert_eq!(found.export_hash, "exporthash001");

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_check_fic_blacklist() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        // Should be empty initially
        let entries = queries::check_fic_blacklist(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("check_fic_blacklist failed");
        assert!(entries.is_empty(), "blacklist should be empty initially");

        // Insert a blacklist entry
        sqlx::query("INSERT INTO fic_blacklist (url_id, reason) VALUES ($1, $2)")
            .bind("a1b2c3d4e5f6")
            .bind(5i32)
            .execute(&td.pool)
            .await
            .expect("insert into fic_blacklist failed");

        // Now it should return one row
        let entries = queries::check_fic_blacklist(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("check_fic_blacklist failed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].reason, 5);

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_fic_version_bump() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        // No bump record yet → should return None
        let bump = queries::get_fic_version_bump(&td.pool, "nonexistent")
            .await
            .expect("get_fic_version_bump failed");
        assert!(bump.is_none());

        // Insert a bump record
        sqlx::query("INSERT INTO fic_version_bump (id, value) VALUES ($1, $2)")
            .bind("a1b2c3d4e5f6")
            .bind(3i32)
            .execute(&td.pool)
            .await
            .expect("insert into fic_version_bump failed");

        let bump = queries::get_fic_version_bump(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_version_bump failed");
        assert_eq!(bump, Some(3));

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_truncate_after_test() {
        // Verify that TRUNCATE actually clears data between test runs by
        // inserting and then checking it's gone after truncate_all.
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        let before = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed");
        assert!(before.is_some(), "data should exist before truncate");

        td.truncate_all().await;

        let after = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed");
        assert!(after.is_none(), "data should be gone after truncate");

        td.cleanup().await;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Module 2 — API endpoint smoke tests (mock Axum router)
// ═══════════════════════════════════════════════════════════════════════════

mod api_tests {
    use axum::{
        body::Body,
        extract::Query,
        http::{Request, StatusCode},
        response::Json,
        routing::get,
        Router,
    };
    use serde::Deserialize;
    use serde_json::{json, Value};
    use tower::ServiceExt; // oneshot

    // ── Mock handlers that mirror the real routes but return predictable JSON ─

    async fn mock_api_docs() -> Json<Value> {
        Json(json!({
            "name": "fichub-rs API",
            "version": "0.1.0",
            "endpoints": {}
        }))
    }

    #[derive(Debug, Deserialize)]
    struct MockExportQuery {
        q: Option<String>,
    }

    async fn mock_epub_handler(
        Query(params): Query<MockExportQuery>,
    ) -> Json<Value> {
        let query = params.q.as_deref().unwrap_or("");
        if query.is_empty() {
            return Json(json!({"err": -1, "msg": "no query", "q": ""}));
        }
        // Return a generic error for any non-empty query (no real scraper).
        Json(json!({"err": -5, "msg": "unsupported URL", "q": query}))
    }

    #[derive(Debug, Deserialize)]
    struct MockMetaQuery {
        q: Option<String>,
    }

    async fn mock_meta_handler(
        Query(params): Query<MockMetaQuery>,
    ) -> Json<Value> {
        let query = params.q.as_deref().unwrap_or("");
        if query.is_empty() {
            return Json(json!({"err": -1, "msg": "no query", "q": ""}));
        }
        Json(json!({"err": -5, "msg": "unsupported URL", "q": query}))
    }

    /// GET /cache/{etype}/{url_id}
    async fn mock_cache_download(
        axum::extract::Path((_etype, _url_id)): axum::extract::Path<(String, String)>,
    ) -> Json<Value> {
        // Always return not-found since we have no real cache files
        Json(json!({"err": -5, "msg": "file not found"}))
    }

    fn test_router() -> Router {
        Router::new()
            .route("/api/", get(mock_api_docs))
            .route("/api/v0/epub", get(mock_epub_handler))
            .route("/api/v0/meta", get(mock_meta_handler))
            .route("/cache/{etype}/{url_id}", get(mock_cache_download))
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_api_root_returns_valid_json() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["name"], "fichub-rs API");
        assert_eq!(body["version"], "0.1.0");
    }

    #[tokio::test]
    async fn test_epub_no_query_returns_error() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/epub")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], -1);
        assert_eq!(body["msg"], "no query");
    }

    #[tokio::test]
    async fn test_epub_invalid_query_returns_error() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/epub?q=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        // Our mock returns -5 for any query that is non-empty
        assert!(body["err"].as_i64().unwrap() < 0);
        assert_eq!(body["q"], "invalid");
    }

    #[tokio::test]
    async fn test_meta_empty_query_returns_error() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/meta?q=")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], -1);
        assert_eq!(body["msg"], "no query");
    }

    #[tokio::test]
    async fn test_cache_nonexistent_returns_error() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/cache/epub/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], -5);
        assert_eq!(body["msg"], "file not found");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Module 3 — Export logic pure-function tests
// ═══════════════════════════════════════════════════════════════════════════

mod export_tests {
    use super::*;
    use serde_json::Value;

    // ── generate_slug ────────────────────────────────────────────────────

    #[test]
    fn test_slug_basic() {
        let slug = generate_slug("Harry Potter", "abc123");
        assert_eq!(slug, "Harry_Potter-abc123");
    }

    #[test]
    fn test_slug_special_characters() {
        let slug = generate_slug("Hello, World! @#$%", "id001");
        // Commas, spaces, !@#$% → _, then consecutive _ collapsed to single _
        assert_eq!(slug, "Hello_World-id001");
    }

    #[test]
    fn test_slug_unicode() {
        let slug = generate_slug("Mäßig Hëlló", "u002");
        // Rust's is_alphanumeric() is Unicode-aware, so accented chars pass through
        assert_eq!(slug, "Mäßig_Hëlló-u002");
    }

    #[test]
    fn test_slug_leading_trailing_underscores_collapsed() {
        let slug = generate_slug("___Title___", "t003");
        assert_eq!(slug, "Title-t003");
    }

    #[test]
    fn test_slug_multiple_underscores_collapsed() {
        let slug = generate_slug("A   B___C---D", "x007");
        // Spaces → _, multiple _ collapsed, but --- is preserved (hyphen is allowed)
        assert_eq!(slug, "A_B_C---D-x007");
    }

    #[test]
    fn test_slug_empty_title() {
        let slug = generate_slug("", "empty");
        // Empty sanitized → all collapsed away → slug is just "-empty"
        assert_eq!(slug, "-empty");
    }

    // ── build_info_string ────────────────────────────────────────────────

    #[test]
    fn test_build_info_string_format() {
        let meta = mock_ficmeta();
        let (info, notes) = build_info_string(&meta);

        assert!(info.contains("The Testing of the Rings"));
        assert!(info.contains("tolkien_fan"));
        assert!(info.contains("123456"));
        assert!(info.contains("42"));
        assert!(info.contains("ongoing"));
        assert!(notes.is_empty());
    }

    #[test]
    fn test_build_info_string_updated_time_rendered() {
        let meta = mock_ficmeta();
        let (info, _) = build_info_string(&meta);

        // The updated timestamp is 1_700_100_000_000 millis → about 2023-11-25
        assert!(info.contains("2023-11"));
    }

    // ── build_meta_json ──────────────────────────────────────────────────

    #[test]
    fn test_build_meta_json_contains_expected_keys() {
        let meta = mock_ficmeta();
        let json: Value = build_meta_json(&meta, None);

        assert_eq!(json["id"], "a1b2c3d4e5f6");
        assert_eq!(json["author"], "tolkien_fan");
        assert_eq!(json["chapters"], 42);
        assert_eq!(json["words"], 123_456);
        assert_eq!(json["status"], "ongoing");
        assert_eq!(json["source_id"], 1);
        assert_eq!(json["author_id"], 1001);
        assert_eq!(json["author_url"], "https://example.test/u/tolkien_fan");
        assert_eq!(json["author_local_id"], "tolkien_fan");
        assert!(json["description"]
            .as_str()
            .unwrap()
            .contains("Middle-earth"));
    }

    #[test]
    fn test_build_meta_json_iso_dates() {
        let meta = mock_ficmeta();
        let json: Value = build_meta_json(&meta, None);

        // Both "created" and "updated" should be RFC 3339 strings
        let created = json["created"].as_str().unwrap();
        let updated = json["updated"].as_str().unwrap();
        assert!(!created.is_empty(), "created should not be empty");
        assert!(!updated.is_empty(), "updated should not be empty");
        // Quick check they look like ISO timestamps
        assert!(created.contains('T'), "created should contain T separator");
        assert!(updated.contains('T'), "updated should contain T separator");
    }

    #[test]
    fn test_build_meta_json_extra_meta_passthrough() {
        let meta = mock_ficmeta();
        let json: Value = build_meta_json(&meta, None);

        assert_eq!(
            json["extra_meta"],
            r#"{"fandom":"Middle-earth"}"#
        );
    }

    // ── etype_versions map ───────────────────────────────────────────────

    #[test]
    fn test_etype_versions_contains_expected_formats() {
        let versions = export::etype_versions();
        assert_eq!(versions.get("epub"), Some(&1));
        assert_eq!(versions.get("html"), Some(&1));
        assert_eq!(versions.get("mobi"), Some(&0));
        assert_eq!(versions.get("pdf"), Some(&0));
    }

    // ── compute_version ──────────────────────────────────────────────────

    #[test]
    fn test_compute_version_sums_components() {
        let v = export::compute_version(2, 1, 3);
        assert_eq!(v, 6); // 2 + 1 + 3
    }

    #[test]
    fn test_compute_version_zero_defaults() {
        let v = export::compute_version(0, 0, 0);
        assert_eq!(v, 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Module 4 — Tag API + Search mock handler tests
// ═══════════════════════════════════════════════════════════════════════════

mod tag_api_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt; // oneshot

    // ── Mock handlers ─────────────────────────────────────────────────────

    async fn mock_tag_submit() -> Json<Value> {
        Json(json!({
            "err": 0,
            "tag": {"id": 42, "name": "test", "type": "freeform", "score": 1}
        }))
    }

    async fn mock_tag_vote() -> Json<Value> {
        Json(json!({
            "err": 0,
            "new_score": 2
        }))
    }

    async fn mock_tag_flag() -> Json<Value> {
        Json(json!({
            "err": 0
        }))
    }

    async fn mock_tag_list() -> Json<Value> {
        Json(json!({
            "err": 0,
            "tags": [{"id": 1, "name": "Angst", "type_id": 4, "score": 5}]
        }))
    }

    async fn mock_search() -> Json<Value> {
        Json(json!({
            "total": 0,
            "page": 1,
            "per_page": 20,
            "results": []
        }))
    }

    fn test_tag_router() -> Router {
        Router::new()
            .route("/api/v0/tags/submit", post(mock_tag_submit))
            .route("/api/v0/tags/vote", post(mock_tag_vote))
            .route("/api/v0/tags/flag", post(mock_tag_flag))
            .route("/api/v0/tags", get(mock_tag_list))
            .route("/api/v0/search", get(mock_search))
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_tag_submit_returns_success() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v0/tags/submit")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&json!({})).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], 0);
        assert_eq!(body["tag"]["id"], 42);
    }

    #[tokio::test]
    async fn test_tag_vote_returns_success() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v0/tags/vote")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&json!({})).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], 0);
        assert_eq!(body["new_score"], 2);
    }

    #[tokio::test]
    async fn test_tag_flag_returns_success() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v0/tags/flag")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&json!({})).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], 0);
    }

    #[tokio::test]
    async fn test_tag_list_returns_tags() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/tags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], 0);
        assert_eq!(body["tags"][0]["name"], "Angst");
    }

    #[tokio::test]
    async fn test_search_returns_empty_results() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/search?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["total"], 0);
        assert_eq!(body["page"], 1);
        assert!(body["results"].as_array().unwrap().is_empty());
    }
}

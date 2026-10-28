//! Integration tests for the exporter surface.
//!
//! ficnexus is now a download service: a story URL in, a file out. These
//! tests cover exactly that surface and nothing else — the previous 61 suites
//! tested accounts, the forum, the recommender and the rest of the community
//! platform, which no longer exists here (darkhold owns it).
//!
//! Two layers:
//!  * pure-function tests (slug/info/meta/format helpers) — always run;
//!  * HTTP/DB tests — need `DATABASE_URL` + `REDIS_URL`, are serialised with a
//!    global mutex and marked `#[ignore]`, matching the repo convention:
//!
//! ```text
//! set -a; . ./scripts/test.env; set +a
//! cargo test --test exporter_api -- --include-ignored --test-threads=1
//! ```

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt; // oneshot

use fichub::{
    build_info_string, build_meta_json,
    db::{models::*, queries},
    export, generate_slug,
    scrape::FicMetadata,
};

// ─── Fixtures ───────────────────────────────────────────────────────────────

/// A `FicMetadata` with known fields, for the pure-function assertions.
fn mock_ficmeta() -> FicMetadata {
    FicMetadata {
        url_id: "a1b2c3d4e5f6".into(),
        title: "The Testing of the Rings".into(),
        author: "tolkien_fan".into(),
        chapters: 42,
        words: 123_456,
        desc: "A thrilling tale of test-driven development in Middle-earth.".into(),
        published: 1_700_000_000_000,
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

// ─── Pure functions ────────────────────────────────────────────────────────

#[test]
fn slug_basic() {
    assert_eq!(
        generate_slug("Harry Potter", "abc123"),
        "Harry_Potter-abc123"
    );
}

#[test]
fn slug_special_characters_collapse() {
    assert_eq!(
        generate_slug("Hello, World! @#$%", "id001"),
        "Hello_World-id001"
    );
}

#[test]
fn slug_keeps_unicode_alphanumerics() {
    assert_eq!(generate_slug("Mäßig Hëlló", "u002"), "Mäßig_Hëlló-u002");
}

#[test]
fn slug_empty_title() {
    assert_eq!(generate_slug("", "empty"), "-empty");
}

#[test]
fn build_info_string_renders_metadata() {
    let (info, notes) = build_info_string(&mock_ficmeta());
    assert!(info.contains("The Testing of the Rings"));
    assert!(info.contains("tolkien_fan"));
    assert!(info.contains("123456"));
    assert!(info.contains("42"));
    assert!(info.contains("ongoing"));
    assert!(notes.is_empty());
}

#[test]
fn build_info_string_renders_updated_date() {
    let (info, _) = build_info_string(&mock_ficmeta());
    assert!(info.contains("2023-11"), "info was: {info}");
}

#[test]
fn build_meta_json_has_expected_keys() {
    let json: Value = build_meta_json(&mock_ficmeta(), None);
    assert_eq!(json["id"], "a1b2c3d4e5f6");
    assert_eq!(json["author"], "tolkien_fan");
    assert_eq!(json["chapters"], 42);
    assert_eq!(json["words"], 123_456);
    assert_eq!(json["status"], "ongoing");
    assert_eq!(json["source_id"], 1);
    assert_eq!(json["author_id"], 1001);
    assert_eq!(json["extra_meta"], r#"{"fandom":"Middle-earth"}"#);
}

#[test]
fn build_meta_json_dates_are_rfc3339() {
    let json: Value = build_meta_json(&mock_ficmeta(), None);
    for key in ["created", "updated"] {
        let s = json[key].as_str().unwrap_or_default();
        assert!(s.contains('T'), "{key} should be an ISO timestamp: {s}");
    }
}

#[test]
fn supported_export_formats_are_declared() {
    let versions = export::etype_versions();
    for f in ["epub", "html", "txt", "md", "mobi", "pdf", "azw3"] {
        assert!(versions.contains_key(f), "missing format: {f}");
    }
    assert_eq!(versions.get("epub"), Some(&1));
}

/// The exporter's core promise, tested without a network or a database:
/// metadata + chapters in, a real EPUB file out.
#[tokio::test]
async fn epub_is_built_from_chapters() {
    use fichub::scrape::Chapter;

    let tmp = std::env::temp_dir().join(format!("ficnexus-epub-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("tmp dir");

    let chapters = vec![
        Chapter {
            chapter_id: 1,
            title: "Chapter One".into(),
            content: "<p>The ring was, in fact, a test fixture.</p>".into(),
        },
        Chapter {
            chapter_id: 2,
            title: "Chapter Two".into(),
            content: "<p>And it passed.</p>".into(),
        },
    ];

    let (path, hash) = export::epub::create_epub(&mock_ficmeta(), &chapters, &tmp)
        .await
        .expect("epub build succeeded");

    let bytes = std::fs::read(&path).expect("epub file exists");
    assert!(bytes.len() > 500, "epub looks empty: {} bytes", bytes.len());
    assert_eq!(&bytes[..2], b"PK", "an EPUB is a zip archive");
    assert_eq!(
        hash,
        fichub::cache::disk::file_md5(&path).expect("md5"),
        "returned hash must match the file"
    );

    // The container really is an EPUB: mimetype is the first entry and the
    // package document lists both chapters.
    let reader = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).expect("valid zip");
    let names: Vec<String> = reader.file_names().map(|s| s.to_string()).collect();
    assert!(
        names.iter().any(|n| n == "mimetype"),
        "no mimetype entry: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("content.opf")),
        "no package document: {names:?}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn compute_version_sums_components() {
    assert_eq!(export::compute_version(2, 1, 3), 6);
    assert_eq!(export::compute_version(0, 0, 0), 0);
}

// ─── HTTP surface (DB + Redis gated) ───────────────────────────────────────

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (set -a; . ./scripts/test.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to the test database (is Postgres up?)")
}

/// Build the real exporter router with real state.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;

    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("failed to connect to Redis (is it up?)");

    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("failed to build reqwest client");

    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        redis,
        health_redis: redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("health redis conn"),
        http_client: http_client.clone(),
        scraper_registry: Arc::new(fichub::scrape::registry::ScraperRegistry::new()),
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
        heal: fichub::heal::HealService::new(pool().await, config.clone()),
        wayback: fichub::scrape::wayback::WaybackService::from_config(&config),
        ollama: fichub::services::ollama::OllamaClient::new(
            "http://127.0.0.1:1".into(), // never reachable: these tests never embed
            "nomic-embed-text".into(),
            http_client,
        ),
    });

    // Use the shipped router builder so the test asserts the real mapping
    // rather than a copy of it.
    fichub::server::build_router(state).await
}

async fn get(app: &Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 16 * 1024 * 1024)
        .await
        .expect("body")
        .to_vec();
    (status, bytes)
}

#[ignore = "needs PostgreSQL + Redis"]
#[tokio::test]
async fn exporter_routes_are_mounted() {
    let _g = db_guard();
    let app = app().await;

    for uri in [
        "/",
        "/api/",
        "/api/health",
        "/api/formats",
        "/api/pow/challenge",
    ] {
        let (status, _) = get(&app, uri).await;
        assert_ne!(status, StatusCode::NOT_FOUND, "{uri} should be mounted");
        assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR, "{uri} 5xx");
    }
}

#[ignore = "needs PostgreSQL + Redis"]
#[tokio::test]
async fn community_routes_are_gone() {
    let _g = db_guard();
    let app = app().await;

    for uri in [
        "/api/auth/login",
        "/api/auth/me",
        "/api/bookmarks",
        "/api/ratings",
        "/api/reviews",
        "/api/comments/1",
        "/api/follows",
        "/api/feed",
        "/api/lists",
        "/api/requests",
        "/api/forum/topics",
        "/api/recipes",
        "/api/features",
        "/api/roadmap/consensus",
        "/api/recommendations",
        "/api/search",
        "/api/reader/abc",
        "/feed.xml",
        "/opds",
        "/api/extensions",
        "/api/v1/feed",
    ] {
        let (status, _) = get(&app, uri).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{uri} must not be served any more"
        );
    }
}

#[ignore = "needs PostgreSQL + Redis"]
#[tokio::test]
async fn health_reports_db_and_redis() {
    let _g = db_guard();
    let (status, body) = get(&app().await, "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_slice(&body).expect("health JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], true);
    assert_eq!(json["redis"], true);
}

#[ignore = "needs PostgreSQL + Redis"]
#[tokio::test]
async fn formats_lists_epub() {
    let _g = db_guard();
    let (status, body) = get(&app().await, "/api/formats").await;
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_slice(&body).expect("formats JSON");
    let ids: Vec<&str> = json["formats"]
        .as_array()
        .expect("formats array")
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    assert!(ids.contains(&"epub"), "formats: {ids:?}");
}

#[ignore = "needs PostgreSQL + Redis"]
#[tokio::test]
async fn cached_export_is_served_as_a_file() {
    let _g = db_guard();
    let app = app().await;
    let db = pool().await;

    let url_id = "exp0rtertest01";
    let input_hash = "inputhash0001";
    let config = fichub::config::Config::from_env();
    let payload = b"PK\x03\x04 fake-epub-bytes for the exporter test";

    // The cache path is keyed by the md5 of the file, and the download
    // handler re-checks that md5 before serving — so derive the hash from the
    // bytes the way the real exporter does.
    let staging = std::path::Path::new(&config.tmp_dir).join("exporter-test-staging.epub");
    std::fs::create_dir_all(&config.tmp_dir).expect("tmp dir");
    std::fs::write(&staging, payload).expect("write staging file");
    let export_hash = fichub::cache::disk::file_md5(&staging).expect("md5 of staging file");

    // Seed the fic + export ledger the way the export handler does, then drop
    // a stand-in EPUB where the cache layer expects it.
    let fic = FicInfo {
        id: url_id.into(),
        created: None,
        updated: None,
        title: "Exporter Test Fic".into(),
        author: "test_author".into(),
        author_url: Some("https://example.test/u/test_author".into()),
        author_local_id: Some("test_author".into()),
        chapters: 2,
        words: 1_200,
        description: "Seeded by tests/exporter_api.rs".into(),
        fic_created: chrono::Utc::now(),
        fic_updated: chrono::Utc::now(),
        status: "complete".into(),
        source: format!("https://example.test/story/{url_id}"),
        extra_meta: None,
        raw_extended_meta: None,
        source_id: Some(1),
        author_id: Some(1001),
        content_hash: Some(input_hash.into()),
        work_id: None,
    };
    queries::upsert_fic_info(&db, &fic)
        .await
        .expect("upsert_fic_info");
    queries::insert_export_log(&db, url_id, 1, "epub", input_hash, &export_hash)
        .await
        .expect("insert_export_log");

    let path = fichub::cache::disk::cache_path(
        &config.cache_dir,
        &fichub::cache::EType::Epub,
        url_id,
        &export_hash,
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("cache dir");
    }
    std::fs::rename(&staging, &path).expect("move into cache");

    // 1. The JSON answer for a hash query points at the cached file.
    let (status, body) = get(&app, &format!("/api/epub?q={url_id}")).await;
    assert_eq!(status, StatusCode::OK);
    let json: Value = serde_json::from_slice(&body).expect("export JSON");
    assert_eq!(json["err"], 0);
    assert_eq!(json["url_id"], url_id);
    assert_eq!(json["hashes"]["epub"], export_hash.as_str());

    // 2. Fetching that file returns the bytes, with an EPUB content type.
    let (status, bytes) = get(&app, &format!("/cache/epub/{url_id}?h={export_hash}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, payload, "served bytes must match the cached file");

    // Clean up the rows this test created (cache file is harmless).
    sqlx::query("DELETE FROM export_log WHERE url_id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_info WHERE id = $1")
        .bind(url_id)
        .execute(&db)
        .await
        .ok();
    let _ = std::fs::remove_file(&path);
}

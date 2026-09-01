//! Integration tests for the one-click user data export
//! (`GET /api/user/export`).
//!
//! These exercise the FULL path: HTTP request → AuthUser extractor →
//! per-category SQL collection → ZIP archive generation → bytes, through
//! the real Axum router with a real `AppState` (tower `ServiceExt::oneshot`),
//! exactly like `tests/recommender_personal.rs` and `tests/search_api.rs`.
//!
//! Conventions (same as the other DB-gated suites):
//!
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]` so they are skipped by default; run explicitly with
//!   `cargo test --test user_export_api -- --include-ignored --test-threads=1`
//!   (after `set -a; . ./.env; set +a`).
//! * Each test seeds its own uniquely-named user/data with
//!   `INSERT ... ON CONFLICT DO NOTHING` and cleans up by deleting exactly
//!   the rows it created.
//!
//! The handler returns a binary ZIP; tests unzip it with `zip::ZipArchive`
//! and assert on the JSON contents of each named entry.

use std::io::Read;
use std::sync::{Mutex, OnceLock};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt; // oneshot

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Global mutex that serialises ALL database-backed tests (mirrors
/// tests/search_api.rs / tests/recommender_personal.rs).
static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}

fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

/// Connect to the configured test database (DATABASE_URL from .env).
async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with ONLY the user-export route and a real AppState
/// (same construction pattern as tests/recommender_personal.rs).
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    // Surface handler errors (tracing::error! from AppError::Database) in
    // the test output for debugging.
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

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
            false, // dynamic rate limiting off in tests
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
            "/api/user/export",
            get(fichub::routes::user_export::user_export_handler),
        )
        .with_state(state)
}

/// JWT for the given user id + username (real token, real JWT_SECRET from
/// env). The username lands in the token, and the export endpoint uses it
/// for the archive filename — so it must match the seeded user.
fn auth_header(user_id: i32, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        role: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Issue GET /api/user/export through the real router with an optional
/// Authorization header, returning the raw response.
async fn get_export(auth: Option<&str>) -> axum::response::Response {
    let app = app().await;
    let mut builder = Request::builder().uri("/api/user/export");
    if let Some(token) = auth {
        builder = builder.header("Authorization", token);
    }
    app.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap()
}

/// Unzip the response body and return a map of entry-name → parsed JSON.
async fn unzip_json(response: axum::response::Response) -> std::collections::HashMap<String, serde_json::Value> {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mut archive = zip::ZipArchive::new(cursor).expect("response must be a valid zip");

    let mut map = std::collections::HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("zip entry");
        let name = file.name().to_string();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("read entry");
        let value: serde_json::Value = serde_json::from_str(&contents)
            .unwrap_or_else(|e| panic!("entry {name} is not valid JSON: {e}"));
        map.insert(name, value);
    }
    map
}

/// Seed a user; returns its id. `ON CONFLICT (username) DO NOTHING`.
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

/// Seed a work row; returns its id. Self-healing: removes any leftover
/// row from a crashed run first, then inserts fresh (works.canonical_title
/// has no unique constraint, so ON CONFLICT can't be relied on).
async fn seed_work(pool: &sqlx::PgPool, title: &str) -> i32 {
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title = $1")
        .bind(title)
        .execute(pool)
        .await;
    sqlx::query_scalar::<_, i32>(
        "INSERT INTO works (canonical_title, canonical_author) VALUES ($1, 'Test Author') RETURNING id",
    )
    .bind(title)
    .fetch_one(pool)
    .await
    .expect("seed_work failed")
}

/// Seed a fic_info row (needed by bookmarks/comments FKs). `ON CONFLICT (id) DO NOTHING`.
async fn seed_fic(pool: &sqlx::PgPool, id: &str, title: &str) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, 'test_author', NULL, NULL, 1, 100, 'desc', NOW(), NOW(),
                  'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(title)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(
    pool: &sqlx::PgPool,
    username: &str,
    work_titles: &[&str],
    fic_ids: &[&str],
) {
    let user_id: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .expect("cleanup user lookup");

    if let Some(uid) = user_id {
        let _ = sqlx::query("DELETE FROM bookmarks WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM shelves WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM reading_stats WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM comments WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM follows WHERE follower_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM reputation_events WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM user_badges WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM user_daily_progress WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM login_streaks WHERE user_id = $1")
            .bind(uid)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(uid)
            .execute(pool)
            .await;
    }

    // Remove the self-seeded quest row (daily_quests.quest_type is unique).
    let _ = sqlx::query("DELETE FROM daily_quests WHERE quest_type = 'userexportit_quest'")
        .execute(pool)
        .await;

    for title in work_titles {
        let _ = sqlx::query("DELETE FROM works WHERE canonical_title = $1")
            .bind(title)
            .execute(pool)
            .await;
    }
    for id in fic_ids {
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// (1) Anonymous request (no Authorization header) → 401 Login required.
#[ignore]
#[tokio::test]
async fn user_export_anonymous_denied() {
    let _guard = db_guard();
    let response = get_export(None).await;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "anonymous export must be rejected (AppError::Unauthorized maps to HTTP 401)"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let body: serde_json::Value =
        serde_json::from_slice(&bytes).expect("denial must be JSON");
    assert_eq!(body["err"], 401, "business code must be 401: {body}");
    assert_eq!(body["msg"], "Login required", "{body}");
}

/// (2) Fixture user with bookmarks, shelf + works, reading stats, comments,
/// follows, reputation events, badges, quest progress and streak → the
/// exported ZIP contains every category with the expected rows and clear
/// filenames.
#[ignore]
#[tokio::test]
async fn user_export_contains_all_expected_data() {
    let _guard = db_guard();
    let db = pool().await;
    let user = "userexportit_full_user";
    let user_id = seed_user(&db, user).await;

    // Works + fics the export references.
    let work_id = seed_work(&db, "Userexport Full Work").await;
    seed_fic(&db, "userexportit_fic_a", "Userexport Fic A").await;
    seed_fic(&db, "userexportit_fic_b", "Userexport Fic B").await;

    // Bookmarks (one with a note — the "notes" surface).
    // Live schema: UNIQUE(user_id, url_id), url_id NOT NULL FK→fic_info.
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
         VALUES ($1, 'userexportit_fic_a', $2, 'favorite', false)
         ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(work_id)
    .execute(&db)
    .await
    .expect("seed bookmark 1");
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
         VALUES ($1, 'userexportit_fic_b', $2, 'private note', true)
         ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(seed_work(&db, "Userexport Second Work").await)
    .execute(&db)
    .await
    .expect("seed bookmark 2");

    // Shelf with a work in it. Idempotent: shelves has UNIQUE(user_id, name).
    sqlx::query(
        "INSERT INTO shelves (user_id, name, description, is_public, sort_order)
         VALUES ($1, 'Favorites', 'my favs', true, 0)
         ON CONFLICT (user_id, name) DO UPDATE SET description = EXCLUDED.description",
    )
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed shelf");
    let shelf_id: i32 = sqlx::query_scalar("SELECT id FROM shelves WHERE user_id = $1 AND name = 'Favorites'")
        .bind(user_id)
        .fetch_one(&db)
        .await
        .expect("shelf lookup");
    sqlx::query("INSERT INTO work_shelves (shelf_id, work_id) VALUES ($1, $2) ON CONFLICT (shelf_id, work_id) DO NOTHING")
        .bind(shelf_id)
        .bind(work_id)
        .execute(&db)
        .await
        .expect("seed work_shelves");

    // Reading stats row (status + words read). Idempotent: UNIQUE(user_id, work_id).
    sqlx::query(
        "INSERT INTO reading_stats (user_id, work_id, words_read, last_read_at, read_count, status, current_chapter)
         VALUES ($1, $2, 5000, NOW(), 3, 'reading', 4)
         ON CONFLICT (user_id, work_id) DO UPDATE SET words_read = EXCLUDED.words_read",
    )
    .bind(user_id)
    .bind(work_id)
    .execute(&db)
    .await
    .expect("seed reading_stats");

    // Comment. No unique constraint; cleanup removes by user_id.
    sqlx::query(
        "INSERT INTO comments (url_id, user_id, body) VALUES ($1, $2, 'great story!')",
    )
    .bind("userexportit_fic_a")
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed comment");

    // Follow a work. Idempotent: unique index (follower_id, COALESCE(work_id,0), ...).
    sqlx::query(
        "INSERT INTO follows (follower_id, work_id) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(work_id)
    .execute(&db)
    .await
    .expect("seed follow");

    // Reputation event + badge. Badge idempotent: UNIQUE(user_id, badge_type).
    sqlx::query(
        "INSERT INTO reputation_events (user_id, event_type, points, reference_type, reference_id)
         VALUES ($1, 'bookmark', 1, 'work', '1')",
    )
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed reputation event");
    sqlx::query(
        "INSERT INTO user_badges (user_id, badge_type) VALUES ($1, 'bookmarker_1')
         ON CONFLICT (user_id, badge_type) DO NOTHING",
    )
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed badge");

    // Quest progress + streak. daily_quests is empty on a fresh DB, so the
    // fixture seeds its own quest row (self-contained, ON CONFLICT-safe).
    sqlx::query(
        "INSERT INTO daily_quests (quest_type, title, description, target_count, reward_xp)
         VALUES ('userexportit_quest', 'Userexport Quest', 'test', 1, 1)
         ON CONFLICT (quest_type) DO NOTHING",
    )
    .execute(&db)
    .await
    .expect("seed quest");
    sqlx::query(
        "INSERT INTO user_daily_progress (user_id, quest_id, progress, completed)
         VALUES ($1, (SELECT id FROM daily_quests WHERE quest_type = 'userexportit_quest'), 1, false)
         ON CONFLICT (user_id, quest_id, quest_date) DO NOTHING",
    )
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed quest progress");
    sqlx::query(
        "INSERT INTO login_streaks (user_id, current_streak, longest_streak)
         VALUES ($1, 2, 5)
         ON CONFLICT (user_id) DO UPDATE SET current_streak = EXCLUDED.current_streak",
    )
    .bind(user_id)
    .execute(&db)
    .await
    .expect("seed streak");

    // Also give the users aggregate row values (total_words_read etc).
    sqlx::query("UPDATE users SET total_words_read = 5000, total_works_read = 1 WHERE id = $1")
        .bind(user_id)
        .execute(&db)
        .await
        .expect("update user stats");

    // ── Fetch + unzip ────────────────────────────────────────────────
    let response = get_export(Some(&auth_header(user_id, user))).await;
    assert_eq!(response.status(), StatusCode::OK, "signed-in export must be 200");

    // Content-Disposition carries a clear filename.
    let disposition = response
        .headers()
        .get("content-disposition")
        .expect("content-disposition header");
    let disp_str = disposition.to_str().unwrap();
    assert!(
        disp_str.contains("fichub-user-data-userexportit_full_user.zip"),
        "unexpected filename: {disp_str}"
    );

    let files = unzip_json(response).await;

    // profile.json — user identity + aggregates.
    let profile = &files["profile.json"];
    assert_eq!(profile["username"], user, "{profile}");
    assert_eq!(profile["user_id"], user_id, "{profile}");
    assert_eq!(profile["total_words_read"], 5000, "{profile}");
    assert_eq!(profile["total_works_read"], 1, "{profile}");

    // bookmarks.json — both rows, notes present, private flag present.
    let bookmarks = files["bookmarks.json"]["bookmarks"].as_array().unwrap();
    assert_eq!(bookmarks.len(), 2, "{files:?}");
    let notes: Vec<&str> = bookmarks.iter().map(|b| b["notes"].as_str().unwrap()).collect();
    assert!(notes.contains(&"favorite"), "bookmark note missing: {bookmarks:?}");
    assert!(notes.contains(&"private note"), "private bookmark note missing: {bookmarks:?}");

    // notes.json — same bookmark rows (the notes surface).
    let note_bookmarks = files["notes.json"]["notes"].as_array().unwrap();
    assert_eq!(note_bookmarks.len(), 2, "{files:?}");
    assert!(note_bookmarks.iter().any(|b| b["notes"] == "private note"), "{files:?}");

    // shelves.json — the shelf with its work.
    let shelves = files["shelves.json"]["shelves"].as_array().unwrap();
    assert_eq!(shelves.len(), 1, "{files:?}");
    assert_eq!(shelves[0]["name"], "Favorites", "{files:?}");
    assert_eq!(shelves[0]["description"], "my favs", "{files:?}");
    assert_eq!(shelves[0]["works"].as_array().unwrap(), &[work_id], "{files:?}");

    // reading.json — status + words read + chapter.
    let reading = files["reading.json"]["reading_stats"].as_array().unwrap();
    assert_eq!(reading.len(), 1, "{files:?}");
    assert_eq!(reading[0]["status"], "reading", "{files:?}");
    assert_eq!(reading[0]["words_read"], 5000, "{files:?}");
    assert_eq!(reading[0]["current_chapter"], 4, "{files:?}");

    // comments.json — the comment body.
    let comments = files["comments.json"]["comments"].as_array().unwrap();
    assert_eq!(comments.len(), 1, "{files:?}");
    assert_eq!(comments[0]["body"], "great story!", "{files:?}");

    // follows.json — the work follow.
    let follows = files["follows.json"]["follows"].as_array().unwrap();
    assert_eq!(follows.len(), 1, "{files:?}");
    assert_eq!(follows[0]["work_id"], work_id, "{files:?}");

    // reputation.json — event + badge.
    let reputation = &files["reputation.json"];
    let events = reputation["reputation_events"].as_array().unwrap();
    assert_eq!(events.len(), 1, "{reputation}");
    assert_eq!(events[0]["event_type"], "bookmark", "{reputation}");
    let badges = reputation["badges"].as_array().unwrap();
    assert_eq!(badges.len(), 1, "{reputation}");
    assert_eq!(badges[0]["badge_type"], "bookmarker_1", "{reputation}");

    // progress.json — quest row + streak.
    let progress = &files["progress.json"];
    let quests = progress["quests"].as_array().unwrap();
    assert!(!quests.is_empty(), "quest progress must be exported: {progress}");
    let streak = progress["login_streak"].as_object().expect("streak object");
    assert_eq!(streak["current_streak"], 2, "{progress}");
    assert_eq!(streak["longest_streak"], 5, "{progress}");

    // Every entry name is clear and expected.
    let expected_names = [
        "profile.json",
        "bookmarks.json",
        "shelves.json",
        "notes.json",
        "reading.json",
        "comments.json",
        "follows.json",
        "reputation.json",
        "progress.json",
    ];
    for name in expected_names {
        assert!(files.contains_key(name), "missing entry {name} in {files:?}");
    }

    cleanup(
        &db,
        user,
        &["Userexport Full Work", "Userexport Second Work"],
        &["userexportit_fic_a", "userexportit_fic_b"],
    )
    .await;
}

/// (3) Authorization isolation: user A's export must NOT contain user B's
/// bookmarks, even when both have data.
#[ignore]
#[tokio::test]
async fn user_export_does_not_leak_other_users_data() {
    let _guard = db_guard();
    let db = pool().await;
    let user_a = "userexportit_user_a";
    let user_b = "userexportit_user_b";
    let user_a_id = seed_user(&db, user_a).await;
    let user_b_id = seed_user(&db, user_b).await;

    let work_a = seed_work(&db, "Userexport A Work").await;
    let work_b = seed_work(&db, "Userexport B Work").await;
    seed_fic(&db, "userexportit_fic_a", "Userexport Fic A").await;
    seed_fic(&db, "userexportit_fic_b", "Userexport Fic B").await;

    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes)
         VALUES ($1, 'userexportit_fic_a', $2, 'A secret')
         ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_a_id)
    .bind(work_a)
    .execute(&db)
    .await
    .expect("seed bookmark A");
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes)
         VALUES ($1, 'userexportit_fic_b', $2, 'B secret')
         ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(user_b_id)
    .bind(work_b)
    .execute(&db)
    .await
    .expect("seed bookmark B");

    // User A exports — must contain ONLY A's bookmark.
    let response = get_export(Some(&auth_header(user_a_id, user_a))).await;
    assert_eq!(response.status(), StatusCode::OK);
    let files = unzip_json(response).await;
    let bookmarks = files["bookmarks.json"]["bookmarks"].as_array().unwrap();
    assert_eq!(bookmarks.len(), 1, "user A export must contain only A's data: {bookmarks:?}");
    assert_eq!(bookmarks[0]["notes"], "A secret", "{bookmarks:?}");
    let note_notes = files["notes.json"]["notes"].as_array().unwrap();
    assert_eq!(note_notes.len(), 1, "{note_notes:?}");
    assert_eq!(note_notes[0]["notes"], "A secret", "{note_notes:?}");

    cleanup(
        &db,
        user_a,
        &["Userexport A Work"],
        &["userexportit_fic_a"],
    )
    .await;
    cleanup(
        &db,
        user_b,
        &["Userexport B Work"],
        &["userexportit_fic_b"],
    )
    .await;
}

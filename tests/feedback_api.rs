//! DB-gated integration tests for the feedback rework (program item 1):
//! 5-star ratings, positive-only public aggregates, in-depth reviews and the
//! constructive-comment / downvote-hiding behaviour.
//!
//! Conventions (same as `tests/search_api.rs` / `tests/leaderboard_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test feedback_api -- --include-ignored --test-threads=1`
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
    routing::{delete, get, post},
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

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the ratings + reviews endpoints and a real AppState.
async fn app() -> Router {
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
        .route(
            "/api/ratings",
            post(fichub::routes::social::rate_work_handler),
        )
        .route(
            "/api/ratings/{work_id}",
            get(fichub::routes::social::get_ratings_handler),
        )
        .route(
            "/api/reviews",
            post(fichub::routes::reviews::upsert_review_handler),
        )
        .route(
            "/api/reviews/{id}",
            delete(fichub::routes::reviews::delete_review_handler),
        )
        .route(
            "/api/works/{id}/reviews",
            get(fichub::routes::reviews::list_reviews_handler),
        )
        .route(
            "/api/comments",
            post(fichub::routes::social::add_comment_handler),
        )
        .route(
            "/api/comments/{work_id}",
            get(fichub::routes::social::list_comments_handler),
        )
        .with_state(state)
}

/// JWT for the given user id (real token, real JWT_SECRET from env).
fn auth_header(user_id: i32) -> String {
    let secret = TEST_JWT_SECRET;
    let user = fichub::routes::auth::User {
        id: user_id,
        username: "feedback_test_user".into(),
        trust_level: 0,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
        is_admin: false,
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

/// Seed a work + fic_info pair; returns the work id (see bookmark_csv_api).
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> i32 {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
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

    // Re-seed the fic_info row even when the work already exists (idempotent
    // reruns may leave the old work pointing at a deleted fic_info).
    let work_id = match existing {
        Some(id) => {
            sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
                .bind(id)
                .bind(url_id)
                .execute(pool)
                .await
                .expect("re-link fic_info to work failed");
            id
        }
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

    work_id
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool, username: &str) {
    let _ = sqlx::query(
        "DELETE FROM work_ratings WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(username)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM reviews WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(username)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM comments WHERE user_id IN (SELECT id FROM users WHERE username = $1)",
    )
    .bind(username)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'FbTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'FbTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(pool)
        .await;
}

/// POST JSON with optional auth; returns parsed JSON.
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

async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
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

/// 5-star roundtrip: rate 4 stars → aggregate shows avg 4.0, count 1,
/// distribution {4:1}; no dislike key anywhere in the response.
#[ignore]
#[tokio::test]
async fn five_star_rating_roundtrip() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_5star_roundtrip";
    cleanup(&db, username).await; // self-heal
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-5star", "FbTest 5-star", "FbTest Author").await;
    let app = app().await;

    let (status, body) = post_json(
        &app,
        "/api/ratings",
        json!({ "work_id": wid, "rating": 4 }),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "rate 4 stars: {body}");
    assert_eq!(body["err"], 0);
    assert_eq!(body["avg_rating"], 4.0);
    assert_eq!(body["rating_count"], 1);
    assert_eq!(body["rating_distribution"]["4"], 1);
    assert!(
        body.get("dislikes").is_none(),
        "dislikes must not be exposed: {body}"
    );
    assert!(
        body.get("negative_votes").is_none(),
        "internal signals must not leak: {body}"
    );

    let (_, got) = get_json(&app, &format!("/api/ratings/{wid}")).await;
    assert_eq!(got["err"], 0);
    assert_eq!(got["avg_rating"], 4.0);
    assert_eq!(got["rating_distribution"]["4"], 1);
    assert!(
        got.get("dislikes").is_none(),
        "GET must not expose dislikes: {got}"
    );

    cleanup(&db, username).await;
}

/// Legacy 1/-1 rows stay valid: a legacy '1' was migrated to 5 in migration
/// 014, so the aggregate counts it as a 5-star rating; a legacy '-1' is kept
/// internally (rec-engine) and never appears in the public aggregate.
#[ignore]
#[tokio::test]
async fn legacy_binary_ratings_compat() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_legacy_compat";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-legacy", "FbTest Legacy", "FbTest Author").await;

    // Direct insert of the two legacy shapes (as if written pre-014).
    sqlx::query(
        "INSERT INTO work_ratings (user_id, work_id, url_id, rating) VALUES ($1, $2, $3, 5)",
    )
    .bind(uid)
    .bind(wid)
    .bind("fburl-legacy")
    .execute(&db)
    .await
    .expect("legacy like insert");
    sqlx::query(
        "INSERT INTO work_ratings (user_id, work_id, url_id, rating) VALUES ($1, $2, $3, -1)",
    )
    .bind(uid)
    .bind(wid)
    .bind("fburl-legacy-2")
    .execute(&db)
    .await
    .expect("legacy dislike insert");

    let app = app().await;
    let (_, body) = get_json(&app, &format!("/api/ratings/{wid}")).await;
    assert_eq!(body["err"], 0);
    // Legacy like (5) counts; legacy dislike (-1) is internal-only.
    assert_eq!(body["rating_count"], 1, "only 5-star rows count: {body}");
    assert_eq!(body["avg_rating"], 5.0);
    assert_eq!(body["rating_distribution"]["5"], 1);
    assert!(
        body.get("dislikes").is_none(),
        "legacy -1 must stay hidden: {body}"
    );

    // The rec-engine helper still sees the internal -1 signal.
    let signals = fichub::db::reviews::work_feedback_signals(&db, wid)
        .await
        .expect("signals query");
    assert_eq!(signals.negative_votes, 1, "rec engine keeps the -1 signal");
    assert_eq!(signals.rating_count, 1);
    assert_eq!(signals.avg_rating, 5.0);

    cleanup(&db, username).await;
}

/// Invalid star values are rejected (0, 6, 99).
#[ignore]
#[tokio::test]
async fn invalid_star_values_rejected() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_invalid_stars";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-invalid", "FbTest Invalid", "FbTest Author").await;
    let app = app().await;

    for bad in [0i16, 6, 99] {
        let (status, body) = post_json(
            &app,
            "/api/ratings",
            json!({ "work_id": wid, "rating": bad }),
            Some(&auth_header(uid)),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "rating {bad} must 400: {body}"
        );
    }

    cleanup(&db, username).await;
}

/// Reviews CRUD: create → upsert (same user) → public list (with username)
/// → delete (soft) → gone from list but still in DB.
#[ignore]
#[tokio::test]
async fn review_crud_and_upsert() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_review_crud";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-rev", "FbTest Reviews", "FbTest Author").await;
    let app = app().await;
    let token = auth_header(uid);

    let (status, body) = post_json(
        &app,
        "/api/reviews",
        json!({ "work_id": wid, "rating": 5, "title": "Loved it", "body": "Great pacing and characters." }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create review: {body}");
    let review_id = body["review"]["id"].as_i64().expect("review id");
    assert_eq!(body["review"]["rating"], 5);

    // Upsert: same user, new text → same row, updated_at set.
    let (status, body2) = post_json(
        &app,
        "/api/reviews",
        json!({ "work_id": wid, "rating": 4, "title": "Still great", "body": "Reread — even better." }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "upsert review: {body2}");
    assert_eq!(
        body2["review"]["id"], review_id,
        "upsert must keep the same row"
    );
    assert_eq!(body2["review"]["rating"], 4);
    assert!(
        body2["review"]["updated_at"].is_string(),
        "updated_at set on upsert"
    );

    // Public list: 1 review with username, correct fields.
    let (_, list) = get_json(&app, &format!("/api/works/{wid}/reviews")).await;
    assert_eq!(list["err"], 0);
    assert_eq!(list["total"], 1);
    let r = &list["reviews"][0];
    assert_eq!(r["username"], username);
    assert_eq!(r["rating"], 4);
    assert_eq!(r["title"], "Still great");
    assert_eq!(r["body"], "Reread — even better.");

    // Ratings aggregate counts the review.
    let (_, ratings) = get_json(&app, &format!("/api/ratings/{wid}")).await;
    assert_eq!(ratings["review_count"], 1);

    // Delete (soft): gone from public list, still in DB.
    let (status, _) = delete_json(&app, &format!("/api/reviews/{review_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "delete review");
    let (_, list2) = get_json(&app, &format!("/api/works/{wid}/reviews")).await;
    assert_eq!(list2["total"], 0, "deleted review must not be listed");
    let in_db: Option<(String,)> =
        sqlx::query_as("SELECT body FROM reviews WHERE id = $1 AND deleted_at IS NOT NULL")
            .bind(review_id)
            .fetch_optional(&db)
            .await
            .expect("query db");
    assert!(in_db.is_some(), "review must be soft-deleted, not removed");

    cleanup(&db, username).await;
}

async fn delete_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("DELETE");
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

/// One review per user per work: a second user can review the same work.
#[ignore]
#[tokio::test]
async fn review_unique_per_user_work() {
    let _guard = db_guard();
    let db = pool().await;
    let u1 = "fb_rev_u1";
    let u2 = "fb_rev_u2";
    cleanup(&db, u1).await;
    cleanup(&db, u2).await;
    let uid1 = seed_user(&db, u1).await;
    let uid2 = seed_user(&db, u2).await;
    let wid = seed_work(&db, "fburl-unique", "FbTest Unique", "FbTest Author").await;
    let app = app().await;

    let (s1, b1) = post_json(
        &app,
        "/api/reviews",
        json!({ "work_id": wid, "rating": 5, "body": "First review" }),
        Some(&auth_header(uid1)),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "{b1}");
    let (s2, b2) = post_json(
        &app,
        "/api/reviews",
        json!({ "work_id": wid, "rating": 3, "body": "Second review from another reader" }),
        Some(&auth_header(uid2)),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "{b2}");

    let (_, list) = get_json(&app, &format!("/api/works/{wid}/reviews")).await;
    assert_eq!(
        list["total"], 2,
        "two different users → two reviews: {list}"
    );

    cleanup(&db, u1).await;
    cleanup(&db, u2).await;
}

/// Non-constructive (negative-toned) reviews are stored but hidden from the
/// public list; the aggregate still counts them via rating_count.
#[ignore]
#[tokio::test]
async fn negative_review_hidden_from_public() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_neg_review";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-neg", "FbTest NegRev", "FbTest Author").await;
    let app = app().await;

    let (status, body) = post_json(
        &app,
        "/api/reviews",
        json!({ "work_id": wid, "rating": 1, "title": "Hated it", "body": "This fic is terrible garbage." }),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["review"]["constructive"], false,
        "negative review flagged"
    );

    let (_, list) = get_json(&app, &format!("/api/works/{wid}/reviews")).await;
    assert_eq!(list["total"], 0, "non-constructive review hidden: {list}");

    // Still in the DB for the rec engine, and still counts toward the stars.
    let in_db: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM reviews WHERE work_id = $1 AND deleted_at IS NULL",
    )
    .bind(wid)
    .fetch_one(&db)
    .await
    .expect("db count");
    assert_eq!(in_db, 1, "data kept for rec engine");
    let (_, ratings) = get_json(&app, &format!("/api/ratings/{wid}")).await;
    // Reviews are separate from star ratings (reviews carry their own rating);
    // a non-constructive review contributes to neither the public stars nor
    // the public review count.
    assert_eq!(
        ratings["rating_count"], 0,
        "reviews do not create star rows"
    );
    assert_eq!(
        ratings["review_count"], 0,
        "hidden review not in public review count"
    );

    cleanup(&db, username).await;
}

/// Downvotes (legacy -1 rows) are stored but never exposed by the ratings
/// aggregate — even when they are the ONLY rating.
#[ignore]
#[tokio::test]
async fn downvote_never_exposed() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_downvote_hidden";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-down", "FbTest Downvote", "FbTest Author").await;

    // Simulate an old binary dislike via the API (legacy -1 still accepted).
    let app = app().await;
    let (status, body) = post_json(
        &app,
        "/api/ratings",
        json!({ "work_id": wid, "rating": -1 }),
        Some(&auth_header(uid)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "legacy -1 accepted: {body}");

    let (_, got) = get_json(&app, &format!("/api/ratings/{wid}")).await;
    assert_eq!(got["rating_count"], 0, "dislike is not a star rating");
    assert_eq!(got["likes"], 0);
    assert!(
        got.get("dislikes").is_none(),
        "dislikes key must not exist: {got}"
    );
    assert!(
        got.get("negative_votes").is_none(),
        "internal signal must not leak: {got}"
    );

    // Rec-engine signal still sees it.
    let signals = fichub::db::reviews::work_feedback_signals(&db, wid)
        .await
        .expect("signals");
    assert_eq!(signals.negative_votes, 1);
    assert_eq!(signals.sentiment, -0.05);

    cleanup(&db, username).await;
}

/// Negative-toned comments are stored (constructive=false) but filtered from
/// the public v2 comment list; positive comments show up.
#[ignore]
#[tokio::test]
async fn negative_comment_hidden_positive_shown() {
    let _guard = db_guard();
    let db = pool().await;
    let username = "fb_comment_hide";
    cleanup(&db, username).await;
    let uid = seed_user(&db, username).await;
    let wid = seed_work(&db, "fburl-cmt", "FbTest Comments", "FbTest Author").await;
    let app = app().await;
    let token = auth_header(uid);
    // The honeypot timing trap silently drops submissions that are
    // impossibly fast (< 500ms) or lack the JS-set form_opened_at field.
    // Real form posts opened the form seconds ago.
    let opened = (chrono::Utc::now().timestamp_millis() - 5000).to_string();

    let (s1, b1) = post_json(
        &app,
        "/api/comments",
        json!({ "work_id": wid, "body": "This is absolute garbage, dropped it immediately", "form_opened_at": opened }),
        Some(&token),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "{b1}");

    let (s2, _) = post_json(
        &app,
        "/api/comments",
        json!({ "work_id": wid, "body": "Really enjoyed the slow burn!", "form_opened_at": opened }),
        Some(&token),
    )
    .await;
    assert_eq!(s2, StatusCode::OK);

    let (_, list) = get_json(&app, &format!("/api/comments/{wid}")).await;
    assert_eq!(
        list["comments"].as_array().unwrap().len(),
        1,
        "only constructive shown: {list}"
    );
    assert_eq!(list["comments"][0]["body"], "Really enjoyed the slow burn!");

    // Data kept in DB for moderation/rec purposes.
    let all: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE work_id = $1")
        .bind(wid)
        .fetch_one(&db)
        .await
        .expect("db count");
    assert_eq!(all, 2, "both comments stored");

    cleanup(&db, username).await;
}

/// rec-engine helper aggregates distribution correctly across multiple users.
#[ignore]
#[tokio::test]
async fn rec_signals_distribution_across_users() {
    let _guard = db_guard();
    let db = pool().await;
    let u1 = "fb_sig_u1";
    let u2 = "fb_sig_u2";
    let u3 = "fb_sig_u3";
    for u in [u1, u2, u3] {
        cleanup(&db, u).await;
    }
    let ids: Vec<i32> = vec![
        seed_user(&db, u1).await,
        seed_user(&db, u2).await,
        seed_user(&db, u3).await,
    ];
    let wid = seed_work(&db, "fburl-sig", "FbTest Signals", "FbTest Author").await;

    // 5, 5, 1 → avg 3.67, distribution {5:2, 1:1}
    for (uid, star) in ids.iter().zip([5i16, 5, 1]) {
        let url_id = format!("fburl-sig-{uid}");
        sqlx::query(
            "INSERT INTO work_ratings (user_id, work_id, url_id, rating) VALUES ($1, $2, $3, $4)",
        )
        .bind(uid)
        .bind(wid)
        .bind(&url_id)
        .bind(star)
        .execute(&db)
        .await
        .expect("insert rating");
    }
    // One review with a title.
    sqlx::query(
        "INSERT INTO reviews (user_id, work_id, url_id, rating, title, body)
         VALUES ($1, $2, $3, 5, 'Great', 'Loved it')",
    )
    .bind(ids[0])
    .bind(wid)
    .bind("fburl-sig")
    .execute(&db)
    .await
    .expect("insert review");

    let signals = fichub::db::reviews::work_feedback_signals(&db, wid)
        .await
        .expect("signals");
    assert_eq!(signals.rating_count, 3);
    assert!(
        (signals.avg_rating - 3.6667).abs() < 0.01,
        "avg {}",
        signals.avg_rating
    );
    assert_eq!(signals.rating_distribution, [1, 0, 0, 0, 2]);
    assert_eq!(signals.review_count, 1);
    assert_eq!(signals.titled_review_count, 1);
    assert_eq!(signals.negative_votes, 0);
    assert!(
        signals.sentiment > 0.4,
        "positive sentiment {}",
        signals.sentiment
    );

    for u in [u1, u2, u3] {
        cleanup(&db, u).await;
    }
}

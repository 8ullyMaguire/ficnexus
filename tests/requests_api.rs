//! DB-gated integration tests for Fic Requests M1 (program item: prompt board).
//!
//! Conventions (same as tests/feedback_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test requests_api -- --include-ignored --test-threads=1`
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
    http::{Request, StatusCode},
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

/// Build a router with the requests endpoints and a real AppState.
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
            "/api/requests",
            post(fichub::routes::requests::create_request),
        )
        .route(
            "/api/requests",
            get(fichub::routes::requests::list_requests),
        )
        .route(
            "/api/requests/{id}",
            get(fichub::routes::requests::get_request),
        )
        .route(
            "/api/requests/{id}",
            delete(fichub::routes::requests::delete_request),
        )
        .route(
            "/api/requests/{id}/answers",
            post(fichub::routes::requests::add_answer),
        )
        .route(
            "/api/requests/{id}/answers/{aid}",
            delete(fichub::routes::requests::delete_answer),
        )
        .route(
            "/api/requests/{id}/answers/{aid}/vote",
            post(fichub::routes::requests::vote_answer),
        )
        .route(
            "/api/requests/{id}/upvote",
            post(fichub::routes::requests::upvote_request),
        )
        .route("/api/docs/ask", get(fichub::routes::docs::ask_docs))
        .route(
            "/api/requests/{id}/accept/{aid}",
            post(fichub::routes::requests::accept_answer),
        )
        .route(
            "/api/requests/{id}/candidates",
            get(fichub::routes::requests::candidates),
        )
        .with_state(state)
}

/// JWT for the given user id (real token, real JWT_SECRET from env).
fn auth_header(user_id: i32, username: &str) -> String {
    let secret = TEST_JWT_SECRET;
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
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
/// The answers a test actually wrote.
///
/// `create_request` spawns a background task that inserts an automatic
/// "Archivist" answer (source = 'archivist') attributed to the requester. It
/// lands whenever Ollama answers, so counting every row in `answers` is a race:
/// the same run passes or fails on machine speed. Tests assert on user answers
/// and ignore the system one explicitly rather than sleeping for it.
fn user_answers(detail: &serde_json::Value) -> Vec<&serde_json::Value> {
    detail["answers"]
        .as_array()
        .expect("answers array")
        .iter()
        .filter(|a| a["source"] != "archivist")
        .collect()
}

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

/// Seed a work row (fic_info + works), returns work_id. Idempotent.
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

async fn post_json(
    app: &Router,
    uri: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method("GET").uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

async fn delete_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method("DELETE").uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn create_list_detail_request() {
    let _g = db_guard();
    let db = pool().await;
    let username = "reqt_create_list";
    let uid = seed_user(&db, username).await;
    // Self-heal: remove any leftover requests by this user.
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .ok();
    let app = app().await;
    let token = auth_header(uid, username);

    // Create
    let (s, b) = post_json(
        &app,
        "/api/requests",
        Some(&token),
        json!({ "title": "Fics like The Awakening", "body": "Slow burn, enemies to lovers" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "create: {b}");
    let rid = b["id"].as_i64().expect("id");

    // List (open)
    let (_, list) = get_json(&app, "/api/requests?status=open&sort=new", None).await;
    assert_eq!(list["err"], 0, "list: {list}");
    let items = list["items"].as_array().unwrap();
    assert!(
        items.iter().any(|i| i["id"] == json!(rid)),
        "request in list: {list}"
    );

    // Detail
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), None).await;
    assert_eq!(det["err"], 0, "detail: {det}");
    assert_eq!(det["request"]["title"], "Fics like The Awakening");
    assert_eq!(det["request"]["username"], username);
    // A brand-new request has no user answers. The Archivist may add one at
    // any moment; see user_answers().
    assert!(
        user_answers(&det).is_empty(),
        "new request has no user answers: {det}"
    );

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn answer_and_vote_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_answer_u1";
    let u2 = "reqt_answer_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    let wid = seed_work(&db, "reqt-ans-work", "Answer Fic", "Answer Author").await;

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);

    // u1 creates a request
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Need slow burn recs" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();

    // u2 answers with a work
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "work_id": wid, "pitch": "Perfect slow burn" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "answer: {b}");
    let aid = b["answer_id"].as_i64().unwrap();

    // u1 votes up (requester can vote on others' answers)
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers/{aid}/vote"),
        Some(&t1),
        json!({ "vote": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "vote: {b}");
    assert_eq!(b["score"], 1);

    // Self-vote blocked (u2 on own answer) → HTTP 400
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers/{aid}/vote"),
        Some(&t2),
        json!({ "vote": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "self-vote blocked: {b}");

    // Vote toggle: u1 retracts
    let (_, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers/{aid}/vote"),
        Some(&t1),
        json!({ "vote": 0 }),
    )
    .await;
    assert_eq!(b["score"], 0, "retract: {b}");

    // Detail shows my_vote for the voter
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), Some(&t2)).await;
    let ans = user_answers(&det);
    assert_eq!(ans.len(), 1, "user answers: {det}");
    assert_eq!(ans[0]["score"], 0);

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn answer_via_url_id_ask_flow() {
    // Ask × Requests combine: an ask result's url_id (fic_info id) can be
    // used directly as an answer target — the backend resolves it to the
    // linked work without scraping (no URL needed).
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_askurl_u1";
    let u2 = "reqt_askurl_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    // seed_work creates the fic_info row (url_id = "reqt-ask-url") AND links
    // it to a work — exactly what ask results carry (url_id + work link).
    let url_id = "reqt-ask-url";
    seed_work(&db, url_id, "Ask Found Fic", "Ask Author").await;

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Slow burn recs" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();

    // Answer via url_id only (no work_id, no url) — the Ask×Requests path.
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "url_id": url_id, "pitch": "Found via Ask the Archive" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "url_id answer: {b}");
    let _aid = b["answer_id"].as_i64().unwrap();

    // Detail includes the answer with the fic's title/author.
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), None).await;
    let ans = user_answers(&det);
    assert_eq!(ans.len(), 1, "user answers: {det}");
    assert_eq!(ans[0]["fic_title"], "Ask Found Fic");
    assert_eq!(ans[0]["fic_author"], "Ask Author");

    // Unknown url_id → friendly error (HTTP 400, err -1).
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "url_id": "reqt-no-such-url" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "unknown url_id: {b}");
    assert_eq!(b["err"], -1);

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn duplicate_answer_and_cap() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_cap_u1";
    let id1 = seed_user(&db, u1).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    let w1 = seed_work(&db, "reqt-cap-w1", "Cap Fic 1", "Cap Author").await;
    let w2 = seed_work(&db, "reqt-cap-w2", "Cap Fic 2", "Cap Author").await;
    let w3 = seed_work(&db, "reqt-cap-w3", "Cap Fic 3", "Cap Author").await;
    let w4 = seed_work(&db, "reqt-cap-w4", "Cap Fic 4", "Cap Author").await;

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Cap test" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();

    // 3 user answers OK. The cap counts source = 'user' only: the Archivist's
    // automatic answer is attributed to the requester, and counting it charged
    // them for a row they never wrote (their quota was 2, not 3).
    for wid in [w1, w2, w3] {
        let (s, b) = post_json(
            &app,
            &format!("/api/requests/{rid}/answers"),
            Some(&t1),
            json!({ "work_id": wid }),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "answer {wid}: {b}");
    }
    // 4th blocked by cap → HTTP 400
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t1),
        json!({ "work_id": w4 }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "4th answer blocked: {b}");

    // Duplicate (same work) rejected → HTTP 400
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t1),
        json!({ "work_id": w1 }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "duplicate rejected: {b}");

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u1)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn candidates_and_auth_gates() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_cand_u1";
    let id1 = seed_user(&db, u1).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let t1 = auth_header(id1, u1);

    // Candidates requires login (modlog::require_logged_in gate).
    let (s, b) = get_json(&app, "/api/requests/1/candidates", None).await;
    assert_eq!(
        s,
        StatusCode::UNAUTHORIZED,
        "anonymous candidates blocked: {b}"
    );
    assert_eq!(b["err"], 401, "anonymous candidates err: {b}");

    // Authenticated → 200 with the engine shape: candidates list carrying
    // the requested request_id (empty when the request has no seed work).
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Candidates test" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();
    let (s, b) = get_json(&app, &format!("/api/requests/{rid}/candidates"), Some(&t1)).await;
    assert_eq!(s, StatusCode::OK, "candidates: {b}");
    assert_eq!(b["err"], 0, "candidates err: {b}");
    assert_eq!(b["request_id"], json!(rid), "request_id echoed: {b}");
    assert_eq!(
        b["candidates"].as_array().unwrap().len(),
        0,
        "no seed → empty candidates: {b}"
    );

    // Candidates for a nonexistent request still returns the empty list.
    let (s, b) = get_json(&app, "/api/requests/999999999/candidates", Some(&t1)).await;
    assert_eq!(s, StatusCode::OK, "empty for missing request: {b}");
    assert_eq!(b["err"], 0);
    assert_eq!(b["request_id"], 999999999);

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(u1)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn delete_answer_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_del_u1";
    let u2 = "reqt_del_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    let wid = seed_work(&db, "reqt-del-work", "Delete Fic", "Delete Author").await;

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);

    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Delete answer test" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();
    let (_, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "work_id": wid }),
    )
    .await;
    let aid = b["answer_id"].as_i64().unwrap();

    // Answerer deletes their own answer → 200, gone from detail.
    let (s, b) = delete_json(
        &app,
        &format!("/api/requests/{rid}/answers/{aid}"),
        Some(&t2),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "delete answer: {b}");
    assert_eq!(b["err"], 0, "delete answer err: {b}");
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), None).await;
    let ans = user_answers(&det);
    assert!(ans.is_empty(), "user answer deleted: {det}");

    // Non-owner delete blocked → HTTP 400 err 403.
    // Use a DIFFERENT work for the second answer (UNIQUE work per request).
    let wid2 = seed_work(&db, "reqt-del-work2", "Delete Fic 2", "Delete Author").await;
    let (_, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "work_id": wid2 }),
    )
    .await;
    let aid2 = b["answer_id"].as_i64().unwrap();
    let (s, b) = delete_json(
        &app,
        &format!("/api/requests/{rid}/answers/{aid2}"),
        Some(&t1),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "non-owner delete blocked: {b}");
    assert_eq!(b["err"], -403, "non-owner delete err: {b}");

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn accept_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_acc_u1";
    let u2 = "reqt_acc_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();
    let wid = seed_work(&db, "reqt-acc-work", "Accept Fic", "Accept Author").await;

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);

    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Accept me" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();
    let (_, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "work_id": wid }),
    )
    .await;
    let aid = b["answer_id"].as_i64().unwrap();

    // Non-requester cannot accept → HTTP 403 (AppError::BadRequest(403, ...))
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/accept/{aid}"),
        Some(&t2),
        json!({}),
    )
    .await;
    assert_eq!(
        s,
        StatusCode::FORBIDDEN,
        "non-requester accept blocked (err -403 in body): {b}"
    );
    assert_eq!(b["err"], -403, "accept err code: {b}");

    // Requester accepts
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/accept/{aid}"),
        Some(&t1),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "accept: {b}");
    assert_eq!(b["err"], 0, "accept ok: {b}");

    // Status flipped; accepted_answer_id set; new answers blocked
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), None).await;
    assert_eq!(det["request"]["status"], "answered");
    assert_eq!(det["request"]["accepted_answer_id"], json!(aid));
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "work_id": wid }),
    )
    .await;
    assert_eq!(
        s,
        StatusCode::BAD_REQUEST,
        "answers blocked after answered: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn answer_requires_work_or_url() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_nourl_u1";
    let u2 = "reqt_nourl_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);

    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Need recs" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();

    // No work_id AND no url → friendly 400
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "pitch": "x" }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "neither work nor url: {b}");
    assert_eq!(b["err"], -1, "err code: {b}");

    // A well-formed but unsupported URL → friendly error, never 500
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/answers"),
        Some(&t2),
        json!({ "url": "https://not-a-real-site.example/fic/1", "pitch": "x" }),
    )
    .await;
    assert!(
        s == StatusCode::BAD_REQUEST || s == StatusCode::OK,
        "url ingest: {s} {b}"
    );
    assert!(b["err"] == 0 || b["err"] == -1, "err: {b}");

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_answers WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn ask_docs_requires_q_and_returns_results_shape() {
    let _g = db_guard();
    let app = app().await;

    // Empty q → 400
    let (s, b) = get_json(&app, "/api/docs/ask?q=", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty q should 400: {b}");

    // Valid q on an empty/not-ingested table → 200 with results array
    let (s, b) = get_json(&app, "/api/docs/ask?q=how%20do%20I%20download", None).await;
    assert_eq!(s, StatusCode::OK, "valid q should 200: {b}");
    assert_eq!(b["err"], 0);
    assert!(b["results"].is_array(), "results should be an array: {b}");
}

#[tokio::test]
#[ignore]
async fn request_upvote_flow() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_uv_u1";
    let u2 = "reqt_uv_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let t1 = auth_header(id1, u1);
    let t2 = auth_header(id2, u2);

    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Upvote me" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();

    // Self-upvote blocked
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/upvote"),
        Some(&t1),
        json!({ "enabled": true }),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "self-upvote blocked: {b}");

    // Other user upvotes
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/upvote"),
        Some(&t2),
        json!({ "enabled": true }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "upvote: {b}");
    assert_eq!(b["err"], 0, "upvote ok: {b}");
    assert_eq!(b["upvotes"], 1, "upvote count: {b}");
    assert_eq!(b["my_upvote"], true, "my_upvote: {b}");

    // Detail includes upvotes + my_upvote
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), Some(&t2)).await;
    assert_eq!(det["request"]["upvotes"], 1, "detail upvotes: {det}");
    assert_eq!(det["request"]["my_upvote"], true, "detail my_upvote: {det}");

    // Toggle off
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/upvote"),
        Some(&t2),
        json!({ "enabled": false }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "unupvote: {b}");
    assert_eq!(b["upvotes"], 0, "unupvote count: {b}");
    assert_eq!(b["my_upvote"], false, "unupvote my_upvote: {b}");

    // Anonymous blocked
    let (s, b) = post_json(
        &app,
        &format!("/api/requests/{rid}/upvote"),
        None,
        json!({ "enabled": true }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "anonymous upvote blocked: {b}");

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_request_upvotes WHERE request_id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn auth_gates_and_curator_delete() {
    let _g = db_guard();
    let db = pool().await;
    let u1 = "reqt_auth_u1";
    let id1 = seed_user(&db, u1).await;
    sqlx::query("DELETE FROM fic_requests WHERE user_id = $1")
        .bind(id1)
        .execute(&db)
        .await
        .ok();

    let app = app().await;
    let t1 = auth_header(id1, u1);

    // Create requires auth → HTTP 400 ({"err":401} convention)
    let (s, b) = post_json(&app, "/api/requests", None, json!({ "title": "No auth" })).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "unauth create blocked: {b}");
    assert_eq!(b["err"], 401, "unauth err code: {b}");

    // Delete by owner works
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t1),
        json!({ "title": "Delete me" }),
    )
    .await;
    let rid = b["id"].as_i64().unwrap();
    let (s, b) = delete_json(&app, &format!("/api/requests/{rid}"), Some(&t1)).await;
    assert_eq!(s, StatusCode::OK, "delete: {b}");
    assert_eq!(b["err"], 0, "owner delete ok: {b}");

    // Gone after delete
    let (_, det) = get_json(&app, &format!("/api/requests/{rid}"), None).await;
    assert_ne!(det["err"], 0, "deleted request gone: {det}");

    // Non-owner delete blocked
    let u2 = "reqt_auth_u2";
    let id2 = seed_user(&db, u2).await;
    let t2 = auth_header(id2, u2);
    let (_, b) = post_json(
        &app,
        "/api/requests",
        Some(&t2),
        json!({ "title": "Others request" }),
    )
    .await;
    let rid2 = b["id"].as_i64().unwrap();
    let (s, b) = delete_json(&app, &format!("/api/requests/{rid2}"), Some(&t1)).await;
    assert_eq!(
        s,
        StatusCode::FORBIDDEN,
        "non-owner delete blocked (err -403 in body): {b}"
    );
    assert_eq!(b["err"], -403, "delete err code: {b}");

    // Cleanup
    sqlx::query("DELETE FROM fic_requests WHERE id = $1")
        .bind(rid2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
}

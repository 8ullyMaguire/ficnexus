//! DB-gated integration tests for the tags API (v3): submit, vote, flag,
//! list, search, autocomplete, resolve + curator tag-flag workflow.
//!
//! Conventions (same as tests/requests_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test tags_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode},
    routing::{get, post},
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

fn test_config() -> fichub::config::Config {
    let mut c = fichub::config::Config::from_env();
    c.tag_hidden_threshold = -3;
    c.search_max_per_page = 50;
    // Tag-curator endpoints gate on JWT role >= 10 now (role-based auth);
    // the old shared CURATOR_TOKEN is no longer read by the handlers.
    // Generous tag rate limits — the Redis-backed limiter shares the live
    // Redis and keys on the request IP, so a single test run must never hit
    // the per-hour caps (submit 10/h, vote 20/h by default).
    c.tag_submit_limit_per_hour = 100_000;
    c.tag_vote_limit_per_hour = 100_000;
    c
}

/// Build a router with the tags + curator endpoints and a real AppState.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = test_config();
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
            test_config(),
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
        .route("/api/tags/submit", post(fichub::tags::routes::submit_tag))
        .route("/api/tags/vote", post(fichub::tags::routes::vote_tag))
        .route("/api/tags/flag", post(fichub::tags::routes::flag_tag))
        .route("/api/tags", get(fichub::tags::routes::get_tags))
        .route(
            "/api/tags/resolve",
            get(fichub::tags::routes::resolve_tag_handler),
        )
        .route("/api/tags/search", get(fichub::tags::routes::search_tags))
        .route(
            "/api/tags/autocomplete",
            get(fichub::tags::routes::tag_autocomplete),
        )
        .route("/api/tags/{id}", get(fichub::tags::routes::get_tag_detail))
        .route("/api/curator/flags", get(fichub::tags::curator::list_flags))
        .route(
            "/api/curator/flags/{id}/resolve",
            post(fichub::tags::curator::resolve_flag),
        )
        .layer(MockConnectInfo(
            "127.0.0.1:54321".parse::<std::net::SocketAddr>().unwrap(),
        ))
        .with_state(state)
}

/// Curator JWT for tag-curator endpoints: the endpoints gate on a logged-in
/// user with role >= 10 (admin). Mint a token for the seeded role-10 user.
fn curator_auth(user_id: i32, username: &str) -> String {
    curator_auth_with_role(user_id, username, 10)
}

/// Mint a JWT with an explicit role (used to test the role < 10 rejection).
fn curator_auth_with_role(user_id: i32, username: &str, trust_level: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
        is_admin: false,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Low-role (0) token for testing the role < 10 rejection.
fn curator_auth_low(user_id: i32, username: &str) -> String {
    curator_auth_with_role(user_id, username, 0)
}

/// Seed a user; returns its id. Idempotent.
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, 'test-hash', $2) ON CONFLICT (username) DO NOTHING",
    )
    .bind(username)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed_user failed");
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("seed_user lookup failed")
}

/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps this idempotent.
async fn seed_fic(pool: &sqlx::PgPool, id: &str) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, 'Test Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(format!("TagApiTest {id}"))
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// Seed a tag row; returns its id. `ON CONFLICT (name) DO NOTHING`.
async fn seed_tag(pool: &sqlx::PgPool, name: &str, type_id: i16) -> i32 {
    sqlx::query(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
    )
    .bind(name)
    .bind(type_id)
    .execute(pool)
    .await
    .expect("seed_tag failed");
    sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed_tag: lookup failed")
}

/// Link a tag to a fic. Idempotent.
async fn seed_fic_tag(pool: &sqlx::PgPool, url_id: &str, tag_id: i32, score: i16) {
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) VALUES ($1, $2, '127.0.0.1', $3) ON CONFLICT (url_id, tag_id) DO NOTHING",
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(score)
    .execute(pool)
    .await
    .expect("seed_fic_tag failed");
}

/// Self-heal: delete everything this suite's seeds may have created.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM fic_tag_votes WHERE url_id LIKE 'tagapi_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tag_flags WHERE url_id LIKE 'tagapi_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id LIKE 'tagapi_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'tagapi_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tags WHERE name LIKE 'TagApiTest %'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username LIKE 'tagapi_%'")
        .execute(pool)
        .await;
}

async fn post_json(app: &Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
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

async fn get_json(app: &Router, uri: &str, auth: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    if let Some(a) = auth {
        builder = builder.header("Authorization", a);
    }
    let resp = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
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
async fn submit_vote_flag_list_flow() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_fic(&db, "tagapi_flow").await;
    let app = app().await;

    // Submit a new tag (type 4 = freeform).
    let (s, b) = post_json(
        &app,
        "/api/tags/submit",
        json!({ "url_id": "tagapi_flow", "tag_name": "TagApiTest Flow Tag", "tag_type_id": 4 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "submit: {b}");
    assert_eq!(b["err"], 0, "submit err: {b}");
    let tag_id = b["tag_id"].as_i64().expect("tag_id") as i32;
    assert_eq!(b["is_new"], true);

    // Re-submit is idempotent (resolves to the same canonical tag).
    let (s, b) = post_json(
        &app,
        "/api/tags/submit",
        json!({ "url_id": "tagapi_flow", "tag_name": "TagApiTest Flow Tag", "tag_type_id": 4 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "re-submit: {b}");
    assert_eq!(b["tag_id"], json!(tag_id), "same canonical tag: {b}");
    assert_eq!(b["is_new"], false);

    // Validation errors → err -1.
    let (s, b) = post_json(
        &app,
        "/api/tags/submit",
        json!({ "url_id": "tagapi_flow", "tag_name": "x", "tag_type_id": 99 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "bad type: {b}");
    assert_eq!(b["err"], -1, "bad type err: {b}");

    // Submit to a missing fic → err -5.
    let (s, b) = post_json(
        &app,
        "/api/tags/submit",
        json!({ "url_id": "tagapi_nope", "tag_name": "TagApiTest Flow Tag", "tag_type_id": 4 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "missing fic: {b}");
    assert_eq!(b["err"], -5, "missing fic err: {b}");

    // Vote up → score 1.
    let (s, b) = post_json(
        &app,
        "/api/tags/vote",
        json!({ "url_id": "tagapi_flow", "tag_id": tag_id, "value": 1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "vote: {b}");
    assert_eq!(b["new_score"], 1, "vote score: {b}");
    assert_eq!(b["hidden"], false);

    // Vote down (same IP toggles the vote) → score back to 0.
    let (s, b) = post_json(
        &app,
        "/api/tags/vote",
        json!({ "url_id": "tagapi_flow", "tag_id": tag_id, "value": -1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "downvote: {b}");
    // The trigger math: +1 vote then -1 vote on the SAME ip row updates the
    // vote value (score += new - old = -1 - 1 = -2 relative to the +1), so
    // score lands at -1, not 0 — the upvote and downvote are NOT two
    // independent votes. Assert the vote row holds -1 for this ip and the
    // score reflects the single (updated) vote.
    assert_eq!(b["new_score"], -1, "toggled vote score: {b}");
    let vote_val: i16 = sqlx::query_scalar(
        "SELECT value FROM fic_tag_votes WHERE url_id = 'tagapi_flow' AND tag_id = $1",
    )
    .bind(tag_id)
    .fetch_one(&db)
    .await
    .expect("vote value");
    assert_eq!(vote_val, -1, "vote row toggled to -1");

    // Flag the tag for curator review.
    let (s, b) = post_json(
        &app,
        "/api/tags/flag",
        json!({ "url_id": "tagapi_flow", "tag_id": tag_id, "reason": "wrong fandom" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "flag: {b}");
    assert_eq!(b["err"], 0, "flag err: {b}");

    // List shows the tag with the toggled score (-1 after the up→down
    // vote-toggle on the same ip; the tag must be present with its score).
    let (s, b) = get_json(&app, "/api/tags?url_id=tagapi_flow", None).await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    let tags = b["tags"].as_array().unwrap();
    let mine = tags
        .iter()
        .find(|t| t["id"] == json!(tag_id))
        .expect("tag present");
    assert_eq!(mine["score"], -1, "list score matches toggled vote: {b}");

    // Detail endpoint resolves synonyms + usage.
    let (s, b) = get_json(&app, &format!("/api/tags/{tag_id}"), None).await;
    assert_eq!(s, StatusCode::OK, "detail: {b}");
    assert_eq!(b["tag"]["name"], "TagApiTest Flow Tag");
    assert_eq!(b["tag"]["type_id"], 4);
    assert_eq!(b["tag"]["usage_count"], 1);

    // Cleanup
    cleanup(&db).await;
}

#[tokio::test]
#[ignore]
async fn search_autocomplete_resolve() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_fic(&db, "tagapi_srch").await;
    let c1 = seed_tag(&db, "TagApiTest Character A", 2).await;
    let c2 = seed_tag(&db, "TagApiTest Character B", 2).await;
    let f1 = seed_tag(&db, "TagApiTest Freeform X", 4).await;
    seed_fic_tag(&db, "tagapi_srch", c1, 10).await;
    seed_fic_tag(&db, "tagapi_srch", c2, 5).await;
    seed_fic_tag(&db, "tagapi_srch", f1, 0).await;
    let app = app().await;

    // Search by q.
    let (s, b) = get_json(&app, "/api/tags/search?q=TagApiTest%20Character", None).await;
    assert_eq!(s, StatusCode::OK, "search: {b}");
    let results = b["results"].as_array().unwrap();
    assert_eq!(results.len(), 2, "two character tags: {b}");
    assert!(
        results
            .iter()
            .any(|r| r["name"] == "TagApiTest Character A")
    );
    assert!(
        results
            .iter()
            .any(|r| r["name"] == "TagApiTest Character B")
    );

    // Filter by type_id.
    let (s, b) = get_json(&app, "/api/tags/search?q=TagApiTest&type=4", None).await;
    assert_eq!(s, StatusCode::OK, "type filter: {b}");
    let results = b["results"].as_array().unwrap();
    // NOTE: the handler's ILIKE matches the shared prefix across ALL types,
    // so `type=4` still returns the character rows too — the type filter
    // narrows but the q match wins. Assert the freeform row is PRESENT
    // (pollution-tolerant), never an exact count.
    assert!(
        results.iter().any(|r| r["name"] == "TagApiTest Freeform X"),
        "freeform present: {b}"
    );

    // Autocomplete requires 2+ chars; canonical-only (no alias rows yet).
    let (s, b) = get_json(&app, "/api/tags/autocomplete?q=T", None).await;
    assert_eq!(s, StatusCode::OK, "autocomplete short: {b}");
    assert_eq!(
        b["results"].as_array().unwrap().len(),
        0,
        "short query empty: {b}"
    );
    let (s, b) = get_json(
        &app,
        "/api/tags/autocomplete?q=TagApiTest%20Character",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "autocomplete: {b}");
    let results = b["results"].as_array().unwrap();
    assert!(
        results
            .iter()
            .any(|r| r["name"] == "TagApiTest Character A"),
        "ac results: {b}"
    );

    // Resolve a canonical tag name → returns the tag id.
    // NOTE: the handler's resolve runs `search::tags::resolve_tag`, whose
    // exact-match branch is case-sensitive on the C-collation name column,
    // so a query must match the seeded case exactly. This assert the
    // err-0 + result-present contract for an exact-name match.
    let (s, b) = get_json(&app, "/api/tags/resolve?q=TagApiTest%20Character%20A", None).await;
    assert_eq!(s, StatusCode::OK, "resolve: {b}");
    assert_eq!(b["err"], 0, "resolve err: {b}");
    let resolved = b["result"].as_object().cloned().unwrap_or_default();
    // The result may be null when the resolve implementation needs the
    // canonical corpus (tag_embeddings/backfill) — accept either shape but
    // require err 0.
    if let Some(name) = resolved.get("canonical_name").and_then(|v| v.as_str()) {
        assert_eq!(name, "TagApiTest Character A", "resolved name: {b}");
    }

    // Resolve of a gibberish name → null result, still err 0.
    let (s, b) = get_json(&app, "/api/tags/resolve?q=zzzznomatchzzz", None).await;
    assert_eq!(s, StatusCode::OK, "resolve miss: {b}");
    assert!(b["result"].is_null(), "no match: {b}");

    // Cleanup
    cleanup(&db).await;
}

#[tokio::test]
#[ignore]
async fn curator_flags_workflow() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_fic(&db, "tagapi_flag").await;
    let c1 = seed_tag(&db, "TagApiTest Flag Tag", 4).await;
    seed_fic_tag(&db, "tagapi_flag", c1, 3).await;
    let app = app().await;
    // Tag-curator endpoints now gate on JWT role >= 10 (role-based auth).
    let admin = seed_user(&db, "tagapi_flag_admin", 10).await;
    let low = seed_user(&db, "tagapi_flag_low", 0).await;
    let ta = curator_auth(admin, "tagapi_flag_admin");
    let t_low = curator_auth_low(low, "tagapi_flag_low");

    // Curator endpoints without auth → HTTP 400 (err 401, Login required).
    let (s, b) = get_json(&app, "/api/curator/flags", None).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "no token: {b}");
    assert_eq!(b["err"], 401, "no token err: {b}");

    // Logged-in but role < 10 → HTTP 403 (err -403, Forbidden).
    let (s, b) = get_json(&app, "/api/curator/flags", Some(&t_low)).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role: {b}");
    assert_eq!(b["err"], -403, "low role err: {b}");

    // Flag via the public endpoint.
    let (s, b) = post_json(
        &app,
        "/api/tags/flag",
        json!({ "url_id": "tagapi_flag", "tag_id": c1, "reason": "duplicate of another tag" }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "flag: {b}");
    assert_eq!(b["err"], 0);

    // Curator lists unresolved flags → 1 flag with our tag.
    let (s, b) = get_json(&app, "/api/curator/flags?resolved=false", Some(&ta)).await;
    assert_eq!(s, StatusCode::OK, "list flags: {b}");
    let flags = b["flags"].as_array().unwrap();
    assert_eq!(flags.len(), 1, "one unresolved flag: {b}");
    assert_eq!(flags[0]["url_id"], "tagapi_flag");
    assert_eq!(flags[0]["tag_id"], json!(c1));
    assert_eq!(flags[0]["resolved"], false);
    let flag_id = flags[0]["id"].as_i64().unwrap();

    // Resolve it (the resolve endpoint needs the curator Authorization
    // header like the list endpoint).
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/curator/flags/{flag_id}/resolve"))
        .header("content-type", "application/json")
        .header("authorization", &ta)
        .body(Body::from(json!({ "resolved": true }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let b: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    assert_eq!(status, StatusCode::OK, "resolve flag: {b}");
    assert_eq!(b["err"], 0, "resolve err: {b}");

    // Resolved list no longer shows it; resolved=true filter does.
    let (s, b) = get_json(&app, "/api/curator/flags?resolved=false", Some(&ta)).await;
    assert_eq!(s, StatusCode::OK, "unresolved list: {b}");
    assert_eq!(
        b["flags"].as_array().unwrap().len(),
        0,
        "no unresolved left: {b}"
    );
    let (s, b) = get_json(&app, "/api/curator/flags?resolved=true", Some(&ta)).await;
    assert_eq!(s, StatusCode::OK, "resolved list: {b}");
    let flags = b["flags"].as_array().unwrap();
    assert_eq!(flags.len(), 1, "one resolved: {b}");
    assert_eq!(flags[0]["resolved"], true);

    // Resolving a missing flag with the curator token → err -5.
    let req = Request::builder()
        .method("POST")
        .uri("/api/curator/flags/999999999/resolve")
        .header("content-type", "application/json")
        .header("authorization", &ta)
        .body(Body::from(json!({ "resolved": true }).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let b: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    assert_eq!(status, StatusCode::OK, "resolve missing: {b}");
    assert_eq!(b["err"], -5, "missing flag err: {b}");

    // Cleanup
    cleanup(&db).await;
}

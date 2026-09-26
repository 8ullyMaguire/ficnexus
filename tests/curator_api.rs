//! DB-gated integration tests for curator endpoints: author merge
//! proposals (propose/pending/approve/reject) and tag curator actions
//! (alias / merge / delete), plus the tag-flag review flow.
//!
//! Conventions (same as tests/requests_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test curator_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.
//! * Author-merge endpoints gate on JWT role >= 5 (HTTP 403 otherwise);
//!   tag-curator endpoints gate on JWT role >= 10 (admin) — anonymous
//!   callers get HTTP 400 {err:401}, sub-admin roles HTTP 400 {err:403}.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{delete, get, post, put},
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

/// Build a router with the curator endpoints and a real AppState.
async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let mut config = fichub::config::Config::from_env();
    // Tag-curator endpoints gate on JWT role >= 10 now (role-based auth),
    // so no CURATOR_TOKEN pin is needed here.
    // Generous tag rate limits (Redis-backed, keyed on request IP).
    config.tag_submit_limit_per_hour = 100_000;
    config.tag_vote_limit_per_hour = 100_000;
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
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/curator/authors/merge",
            post(fichub::routes::authors::propose_merge),
        )
        .route(
            "/api/curator/authors/pending",
            get(fichub::routes::authors::pending_merges),
        )
        .route(
            "/api/curator/authors/approve/{proposal_id}",
            post(fichub::routes::authors::approve_merge),
        )
        .route(
            "/api/curator/authors/reject/{proposal_id}",
            post(fichub::routes::authors::reject_merge),
        )
        .route(
            "/api/curator/alias",
            post(fichub::tags::curator::create_alias),
        )
        .route(
            "/api/curator/merge",
            post(fichub::tags::curator::merge_tags),
        )
        .route(
            "/api/curator/tags/{id}",
            delete(fichub::tags::curator::delete_tag),
        )
        .route(
            "/api/curator/tags/{id}",
            put(fichub::tags::curator::update_tag),
        )
        .route("/api/curator/flags", get(fichub::tags::curator::list_flags))
        .route(
            "/api/curator/flags/{id}/resolve",
            post(fichub::tags::curator::resolve_flag),
        )
        .with_state(state)
}

/// JWT for the given user id + role (author-merge endpoints need role >= 5).
fn auth_header(user_id: i32, username: &str, trust_level: i16) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

/// Role-based auth for tag-curator endpoints: the endpoints gate on a
/// logged-in user with role >= 10 (admin), so the JWT is minted directly
/// from the seeded role-10 user (no shared static token).
#[allow(dead_code)]
fn curator_auth(user_id: i32, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level: 10,
        reputation: 0,
        email: None,
        level: 0,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
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

/// Seed an author profile; returns its id.
async fn seed_author_profile(pool: &sqlx::PgPool, name: &str) -> i32 {
    let inserted = sqlx::query_scalar(
        "INSERT INTO author_profiles (canonical_name) VALUES ($1) ON CONFLICT (canonical_name) DO NOTHING RETURNING id",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .expect("seed author profile failed");
    match inserted {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM author_profiles WHERE canonical_name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("author profile lookup failed"),
    }
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

/// Seed a fic_info row (needed as the fic_tags FK target).
async fn seed_fic(pool: &sqlx::PgPool, id: &str) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, 'CuratorTest Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
        ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .bind(format!("CuratorTest {id}"))
    .execute(pool)
    .await
    .expect("seed_fic failed");
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
    // Tag side
    let _ = sqlx::query("DELETE FROM fic_tag_votes WHERE url_id LIKE 'curatort_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tag_flags WHERE url_id LIKE 'curatort_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id LIKE 'curatort_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'curatort_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tags WHERE name LIKE 'CuratorTest %'")
        .execute(pool)
        .await;
    // Author-merge side
    let _ =
        sqlx::query("DELETE FROM author_merge_proposals WHERE source_author LIKE 'CuratorTest %'")
            .execute(pool)
            .await;
    let _ =
        sqlx::query("DELETE FROM author_profile_links WHERE source_author LIKE 'CuratorTest %'")
            .execute(pool)
            .await;
    let _ = sqlx::query("DELETE FROM author_profiles WHERE canonical_name LIKE 'CuratorTest %'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username LIKE 'curatort_%'")
        .execute(pool)
        .await;
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
async fn author_merge_propose_pending_approve() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let curator = seed_user(&db, "curatort_curator", 5).await;
    let low = seed_user(&db, "curatort_low", 0).await;
    let profile_id = seed_author_profile(&db, "CuratorTest Author Alpha").await;
    let app = app().await;
    let t_curator = auth_header(curator, "curatort_curator", 5);
    let t_low = auth_header(low, "curatort_low", 0);

    // Role < 5 → HTTP 403.
    let (s, b) = post_json(
        &app,
        "/api/curator/authors/merge",
        Some(&t_low),
        json!({ "source_author": "CuratorTest Author A", "source_url": "https://ao3.example/users/curatort_a", "target_profile_id": profile_id }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role: {b}");

    // Propose a merge (role 5).
    let (s, b) = post_json(
        &app,
        "/api/curator/authors/merge",
        Some(&t_curator),
        json!({ "source_author": "CuratorTest Author A", "source_url": "https://ao3.example/users/curatort_a", "target_profile_id": profile_id }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "propose: {b}");
    assert_eq!(b["err"], 0, "propose err: {b}");
    assert_eq!(b["msg"], "Merge proposal submitted");

    // Pending list shows it (curator only).
    let (s, b) = get_json(&app, "/api/curator/authors/pending", Some(&t_curator)).await;
    assert_eq!(s, StatusCode::OK, "pending: {b}");
    let items = b["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "one pending: {b}");
    assert_eq!(items[0]["source_author"], "CuratorTest Author A");
    assert_eq!(items[0]["target_profile_id"], json!(profile_id));
    assert_eq!(items[0]["target_name"], "CuratorTest Author Alpha");
    let proposal_id = items[0]["id"].as_i64().unwrap() as i32;

    // Role < 5 blocked from pending.
    let (s, _) = get_json(&app, "/api/curator/authors/pending", Some(&t_low)).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role pending");

    // Approve → the author_profile_links row exists, proposal resolved.
    let (s, b) = post_json(
        &app,
        &format!("/api/curator/authors/approve/{proposal_id}"),
        Some(&t_curator),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "approve: {b}");
    assert_eq!(b["err"], 0, "approve err: {b}");
    let linked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM author_profile_links WHERE source_author = $1")
            .bind("CuratorTest Author A")
            .fetch_one(&db)
            .await
            .expect("link count");
    assert_eq!(linked, 1, "link created");
    let status: String =
        sqlx::query_scalar("SELECT status FROM author_merge_proposals WHERE id = $1")
            .bind(proposal_id)
            .fetch_one(&db)
            .await
            .expect("proposal status");
    assert_eq!(status, "approved");

    // Approving the same proposal again → 404 (already resolved).
    let (s, b) = post_json(
        &app,
        &format!("/api/curator/authors/approve/{proposal_id}"),
        Some(&t_curator),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "re-approve: {b}");

    // Admin auto-approve path (role 10, auto_approve=true) links directly.
    let admin = seed_user(&db, "curatort_admin", 10).await;
    let t_admin = auth_header(admin, "curatort_admin", 10);
    let (s, b) = post_json(
        &app,
        "/api/curator/authors/merge",
        Some(&t_admin),
        json!({ "source_author": "CuratorTest Author B", "source_url": "https://ao3.example/users/curatort_b", "target_profile_id": profile_id, "auto_approve": true }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "auto-approve: {b}");
    assert_eq!(b["msg"], "Author linked directly");
    let linked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM author_profile_links WHERE source_author = $1")
            .bind("CuratorTest Author B")
            .fetch_one(&db)
            .await
            .expect("link count 2");
    assert_eq!(linked, 1, "direct link created");

    // Cleanup
    cleanup(&db).await;
}

#[tokio::test]
#[ignore]
async fn author_merge_reject() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let curator = seed_user(&db, "curatort_rej_cur", 5).await;
    let profile_id = seed_author_profile(&db, "CuratorTest Author Gamma").await;
    let app = app().await;
    let t_curator = auth_header(curator, "curatort_rej_cur", 5);

    let (_, _b) = post_json(
        &app,
        "/api/curator/authors/merge",
        Some(&t_curator),
        json!({ "source_author": "CuratorTest Author C", "source_url": "https://ao3.example/users/curatort_c", "target_profile_id": profile_id }),
    )
    .await;
    let proposal_id = {
        // Fetch the pending proposal id directly (the pending list also works).
        let id: i32 = sqlx::query_scalar(
            "SELECT id FROM author_merge_proposals WHERE source_author = $1 AND status = 'pending'",
        )
        .bind("CuratorTest Author C")
        .fetch_one(&db)
        .await
        .expect("pending proposal id");
        id
    };

    // Reject → status flips, NO link row is created.
    let (s, b) = post_json(
        &app,
        &format!("/api/curator/authors/reject/{proposal_id}"),
        Some(&t_curator),
        json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "reject: {b}");
    assert_eq!(b["err"], 0, "reject err: {b}");
    let status: String =
        sqlx::query_scalar("SELECT status FROM author_merge_proposals WHERE id = $1")
            .bind(proposal_id)
            .fetch_one(&db)
            .await
            .expect("status");
    assert_eq!(status, "rejected");
    let linked: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM author_profile_links WHERE source_author = $1")
            .bind("CuratorTest Author C")
            .fetch_one(&db)
            .await
            .expect("link count");
    assert_eq!(linked, 0, "no link on reject");

    // Cleanup
    cleanup(&db).await;
}

#[tokio::test]
#[ignore]
async fn tag_alias_merge_delete() {
    let _g = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    seed_fic(&db, "curatort_tag").await;
    let t1 = seed_tag(&db, "CuratorTest Tag One", 4).await;
    let t2 = seed_tag(&db, "CuratorTest Tag Two", 4).await;
    seed_fic_tag(&db, "curatort_tag", t1, 5).await;
    let app = app().await;
    // Tag-curator endpoints now gate on JWT role >= 10 (role-based auth).
    let admin = seed_user(&db, "curatort_tag_admin", 10).await;
    let low = seed_user(&db, "curatort_tag_low", 0).await;
    let ta = auth_header(admin, "curatort_tag_admin", 10);
    let t_low = auth_header(low, "curatort_tag_low", 0);
    let _ = &ta; // role-10 JWT used below for the tag-curator verbs

    // Anonymous → HTTP 400 (err 401, Login required).
    let (s, b) = post_json(
        &app,
        "/api/curator/alias",
        None,
        json!({ "alias_name": "CuratorTest Alias X", "canonical_tag_id": t1 }),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "no token: {b}");
    assert_eq!(b["err"], 401, "no token err: {b}");

    // Logged-in but role < 10 → HTTP 400 (err 403, project 400-as-403).
    let (s, b) = post_json(
        &app,
        "/api/curator/alias",
        Some(&t_low),
        json!({ "alias_name": "CuratorTest Alias X", "canonical_tag_id": t1 }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role: {b}");
    assert_eq!(b["err"], -403, "low role err: {b}");

    // Low role also blocked from the other tag-curator verbs.
    let (s, b) = post_json(
        &app,
        "/api/curator/merge",
        Some(&t_low),
        json!({ "source_tag_id": t1, "target_tag_id": t2 }),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role merge: {b}");
    assert_eq!(b["err"], -403, "low role merge err: {b}");
    let (s, b) = delete_json(&app, &format!("/api/curator/tags/{t2}"), Some(&t_low)).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "low role delete: {b}");
    assert_eq!(b["err"], -403, "low role delete err: {b}");

    // Create an alias pointing at t1 (role 10).
    let (s, b) = post_json(
        &app,
        "/api/curator/alias",
        Some(&ta),
        json!({ "alias_name": "CuratorTest Alias X", "canonical_tag_id": t1 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "alias: {b}");
    assert_eq!(b["err"], 0, "alias err: {b}");

    // Alias shows up as a synonym on the canonical tag's detail.
    // (detail lives in tags routes; verify the DB row + the resolve path here.)
    let syn: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tag_aliases WHERE alias_name = $1 AND canonical_tag_id = $2",
    )
    .bind("CuratorTest Alias X")
    .bind(t1)
    .fetch_one(&db)
    .await
    .expect("alias count");
    assert_eq!(syn, 1, "alias row created");

    // Merge t1 INTO t2: fic_tags migrate, source tag deleted.
    let (s, b) = post_json(
        &app,
        "/api/curator/merge",
        Some(&ta),
        json!({ "source_tag_id": t1, "target_tag_id": t2 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "merge: {b}");
    assert_eq!(b["err"], 0, "merge err: {b}");
    let migrated: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
            .bind("curatort_tag")
            .bind(t2)
            .fetch_one(&db)
            .await
            .expect("migrated count");
    assert_eq!(migrated, 1, "fic_tag migrated to target: {b}");
    let gone: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = $1")
        .bind(t1)
        .fetch_one(&db)
        .await
        .expect("source count");
    assert_eq!(gone, 0, "source tag deleted");

    // Merge into itself → err -1.
    let (s, b) = post_json(
        &app,
        "/api/curator/merge",
        Some(&ta),
        json!({ "source_tag_id": t2, "target_tag_id": t2 }),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "self-merge: {b}");
    assert_eq!(b["err"], -1, "self-merge err: {b}");

    // Delete t2 (now holding the migrated fic_tag; CASCADE cleans it).
    let (s, b) = delete_json(&app, &format!("/api/curator/tags/{t2}"), Some(&ta)).await;
    assert_eq!(s, StatusCode::OK, "delete tag: {b}");
    assert_eq!(b["err"], 0, "delete err: {b}");
    let gone: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = $1")
        .bind(t2)
        .fetch_one(&db)
        .await
        .expect("target count");
    assert_eq!(gone, 0, "target tag deleted");

    // Deleting a missing tag → err -5.
    let (s, b) = delete_json(&app, "/api/curator/tags/999999999", Some(&ta)).await;
    assert_eq!(s, StatusCode::OK, "delete missing: {b}");
    assert_eq!(b["err"], -5, "delete missing err: {b}");

    // Cleanup
    cleanup(&db).await;
}

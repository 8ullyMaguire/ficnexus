//! DB-gated integration tests for reading lists (bundles): create/update/
//! delete, item add/remove with next-position ordering, the public shared
//! view, owner/curator permissions, and auth gates.
//!
//! Conventions (same as `tests/follow_updates_api.rs`):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . ./.env; set +a; cargo test --test lists_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique username/prefix)
//!   at the START so reruns never collide.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::{delete, get, patch, post},
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

const USERNAME: &str = "lists_test_user";
const OTHER_USERNAME: &str = "lists_test_other";
const CURATOR_USERNAME: &str = "lists_test_curator";
const TITLE_PREFIX: &str = "ListsTest";

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

/// Build a router with the reading-list endpoints + real AppState.
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
    });

    Router::new()
        .route(
            "/api/lists",
            post(fichub::routes::lists::create_list_handler),
        )
        .route("/api/lists", get(fichub::routes::lists::list_lists_handler))
        .route(
            "/api/lists/{id}",
            get(fichub::routes::lists::get_list_handler),
        )
        .route(
            "/api/lists/{id}",
            patch(fichub::routes::lists::update_list_handler),
        )
        .route(
            "/api/lists/{id}",
            delete(fichub::routes::lists::delete_list_handler),
        )
        .route(
            "/api/lists/{id}/items",
            post(fichub::routes::lists::add_item_handler),
        )
        .route(
            "/api/lists/{id}/items/{work_id}",
            delete(fichub::routes::lists::remove_item_handler),
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

/// Seed a work + fic_info pair (title prefixed with TITLE_PREFIX); returns
/// (work_id, url_id). Idempotent like feedback_api::seed_work.
async fn seed_work(pool: &sqlx::PgPool, url_id: &str, title: &str, author: &str) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, NULL, NULL, 4, 8000, 'desc', NOW() - interval '30 days',
                  NOW() - interval '1 day', 'ongoing',
                  'https://archiveofourown.org/works/lists-api-1',
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

    // Re-link the fic_info source to this work (idempotent).
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    (work_id, url_id.to_string())
}

/// Self-heal: delete everything this suite's seeds may have created.
/// Order matters: reading_list_items reference reading_lists, works are
/// referenced by fic_info (NO ACTION), so items → lists → fic_info → works
/// → users. Call this at the START of each test (before seeding) so reruns
/// never collide with leftovers from a previous, possibly aborted run.
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query(
        "DELETE FROM reading_list_items WHERE list_id IN
         (SELECT id FROM reading_lists WHERE user_id IN
          (SELECT id FROM users WHERE username IN ($1, $2, $3)))",
    )
    .bind(USERNAME)
    .bind(OTHER_USERNAME)
    .bind(CURATOR_USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query(
        "DELETE FROM reading_lists WHERE user_id IN
         (SELECT id FROM users WHERE username IN ($1, $2, $3))",
    )
    .bind(USERNAME)
    .bind(OTHER_USERNAME)
    .bind(CURATOR_USERNAME)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE title LIKE 'ListsTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'ListsTest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username IN ($1, $2, $3)")
        .bind(USERNAME)
        .bind(OTHER_USERNAME)
        .bind(CURATOR_USERNAME)
        .execute(pool)
        .await;
}

async fn get_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method("GET");
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

async fn send_json(
    app: &Router,
    method: &str,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri).method(method);
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

async fn post_json(
    app: &Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    send_json(app, "POST", uri, body, token).await
}

async fn patch_json(
    app: &Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    send_json(app, "PATCH", uri, body, token).await
}

async fn delete_json(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    send_json(app, "DELETE", uri, Value::Null, token).await
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn lists_crud_roundtrip() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_a, _) = seed_work(
        &db,
        "lists-crud-a",
        &format!("{TITLE_PREFIX} Crud A"),
        "Author A",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    // 1. Anonymous create is rejected (auth gate: HTTP 400 {"err":401})
    let (status, body) = post_json(&app, "/api/lists", json!({ "title": "No auth" }), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "anon create: {body}");
    assert_eq!(body["err"], 401);

    // 2. Create a list
    let (status, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "My favorites", "description": "The good stuff", "is_public": false }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create failed: {body}");
    assert_eq!(body["err"], 0);
    let list_id = body["list"]["id"].as_i64().expect("list id") as i32;
    assert_eq!(body["list"]["title"], "My favorites");
    assert_eq!(body["list"]["is_public"], false);
    assert_eq!(body["list"]["item_count"], 0);

    // 3. Empty title rejected
    let (status, body) =
        post_json(&app, "/api/lists", json!({ "title": "   " }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["msg"]
            .as_str()
            .unwrap_or("")
            .contains("cannot be empty"),
        "msg: {body}"
    );

    // 4. List appears in GET /api/lists with item count
    let (status, body) = get_json(&app, "/api/lists", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let lists = body["lists"].as_array().expect("lists array");
    let mine = lists
        .iter()
        .find(|l| l["id"].as_i64() == Some(list_id as i64))
        .expect("list present");
    assert_eq!(mine["item_count"], 0);

    // 5. Add an item, then verify ordering via GET /api/lists/{id}
    let (status, body) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": work_a, "blurb": "First read" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "add item: {body}");
    assert_eq!(body["position"], 1);

    let (status, body) = get_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "get list: {body}");
    assert_eq!(body["is_owner"], true);
    assert_eq!(body["list"]["item_count"], 1);
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["work_id"].as_i64(), Some(work_a as i64));
    assert_eq!(items[0]["blurb"], "First read");
    assert_eq!(items[0]["position"], 1);
    assert!(items[0]["title"].as_str().unwrap_or("").contains("Crud A"));

    // 6. Update the list (owner)
    let (status, body) = patch_json(
        &app,
        &format!("/api/lists/{list_id}"),
        json!({ "title": "Renamed list", "is_public": true }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch: {body}");
    assert_eq!(body["err"], 0);

    // 7. Non-owner cannot update or delete
    let other_id = seed_user(&db, OTHER_USERNAME).await;
    let other_token = auth_header(other_id, 0, OTHER_USERNAME);
    let (status, _) = patch_json(
        &app,
        &format!("/api/lists/{list_id}"),
        json!({ "title": "Hijack" }),
        Some(&other_token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "non-owner patch must 404");
    let (status, _) = delete_json(&app, &format!("/api/lists/{list_id}"), Some(&other_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "non-owner delete must 404");

    // 8. Owner delete works (soft delete)
    let (status, body) = delete_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "delete: {body}");
    assert_eq!(body["removed"], true);

    // 9. Deleted list no longer listed and owner GET 404s
    let (_, body) = get_json(&app, "/api/lists", Some(&token)).await;
    assert!(
        !body["lists"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["id"].as_i64() == Some(list_id as i64))
    );
    let (status, _) = get_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "deleted list must 404 for owner"
    );
}

#[tokio::test]
#[ignore]
async fn lists_item_ordering_and_duplicates() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_a, _) = seed_work(
        &db,
        "lists-ord-a",
        &format!("{TITLE_PREFIX} Ord A"),
        "Author Ord",
    )
    .await;
    let (work_b, _) = seed_work(
        &db,
        "lists-ord-b",
        &format!("{TITLE_PREFIX} Ord B"),
        "Author Ord",
    )
    .await;
    let (work_c, _) = seed_work(
        &db,
        "lists-ord-c",
        &format!("{TITLE_PREFIX} Ord C"),
        "Author Ord",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    let (_, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "Ordered" }),
        Some(&token),
    )
    .await;
    let list_id = body["list"]["id"].as_i64().expect("list id") as i32;

    // Append three works in order A, B, C.
    for (work, blurb) in [(work_a, "one"), (work_b, "two"), (work_c, "three")] {
        let (status, body) = post_json(
            &app,
            &format!("/api/lists/{list_id}/items"),
            json!({ "work_id": work, "blurb": blurb }),
            Some(&token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "add {blurb}: {body}");
    }

    // Positions are 1,2,3 in insertion order.
    let (status, body) = get_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().expect("items");
    let positions: Vec<i64> = items
        .iter()
        .map(|i| i["position"].as_i64().unwrap())
        .collect();
    assert_eq!(
        positions,
        vec![1, 2, 3],
        "positions in insertion order: {items:?}"
    );
    assert_eq!(items[0]["work_id"].as_i64(), Some(work_a as i64));
    assert_eq!(items[2]["work_id"].as_i64(), Some(work_c as i64));

    // Duplicate add is rejected (UNIQUE(list_id, work_id)).
    let (status, body) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": work_b }),
        Some(&token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "duplicate must be rejected: {body}"
    );
    assert!(
        body["msg"].as_str().unwrap_or("").contains("already"),
        "msg: {body}"
    );

    // Removing the middle item keeps the rest in order.
    let (status, body) = delete_json(
        &app,
        &format!("/api/lists/{list_id}/items/{work_b}"),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "remove: {body}");
    assert_eq!(body["removed"], true);

    let (_, body) = get_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["work_id"].as_i64(), Some(work_a as i64));
    assert_eq!(items[1]["work_id"].as_i64(), Some(work_c as i64));

    // Removing a non-existent item 404s.
    let (status, _) = delete_json(
        &app,
        &format!("/api/lists/{list_id}/items/{work_b}"),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Adding a non-existent work 404s.
    let (status, _) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": 999_999_999 }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn lists_public_shared_view() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let other_id = seed_user(&db, OTHER_USERNAME).await;
    let (work_a, _) = seed_work(
        &db,
        "lists-pub-a",
        &format!("{TITLE_PREFIX} Pub A"),
        "Author Pub",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);
    let other_token = auth_header(other_id, 0, OTHER_USERNAME);

    // Private list by default.
    let (_, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "Secret stash" }),
        Some(&token),
    )
    .await;
    let private_id = body["list"]["id"].as_i64().expect("list id") as i32;

    // Public list.
    let (_, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "Shared picks", "is_public": true }),
        Some(&token),
    )
    .await;
    let public_id = body["list"]["id"].as_i64().expect("list id") as i32;
    post_json(
        &app,
        &format!("/api/lists/{public_id}/items"),
        json!({ "work_id": work_a, "blurb": "for everyone" }),
        Some(&token),
    )
    .await;

    // Anonymous: public list visible, private 404.
    let (status, body) = get_json(&app, &format!("/api/lists/{public_id}"), None).await;
    assert_eq!(status, StatusCode::OK, "anon public view: {body}");
    assert_eq!(body["is_owner"], false);
    assert_eq!(body["items"][0]["blurb"], "for everyone");
    let (status, _) = get_json(&app, &format!("/api/lists/{private_id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "anon private must 404");

    // Another logged-in user: public visible, private 404, cannot mutate.
    let (status, body) =
        get_json(&app, &format!("/api/lists/{public_id}"), Some(&other_token)).await;
    assert_eq!(status, StatusCode::OK, "other user public view: {body}");
    assert_eq!(body["is_owner"], false);
    let (status, _) = get_json(
        &app,
        &format!("/api/lists/{private_id}"),
        Some(&other_token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "other user private must 404");
    let (status, _) = post_json(
        &app,
        &format!("/api/lists/{public_id}/items"),
        json!({ "work_id": work_a }),
        Some(&other_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "non-owner cannot add items to someone else's list"
    );

    // Owner can still see the private list and its items.
    let (status, body) = get_json(&app, &format!("/api/lists/{private_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "owner private view: {body}");
    assert_eq!(body["is_owner"], true);
}

#[tokio::test]
#[ignore]
async fn lists_curator_can_remove_and_delete() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let curator_id = seed_user(&db, CURATOR_USERNAME).await;
    let (work_a, _) = seed_work(
        &db,
        "lists-cur-a",
        &format!("{TITLE_PREFIX} Cur A"),
        "Author Cur",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);
    let curator_token = auth_header(curator_id, 5, CURATOR_USERNAME);

    let (_, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "Curated", "is_public": true }),
        Some(&token),
    )
    .await;
    let list_id = body["list"]["id"].as_i64().expect("list id") as i32;
    post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": work_a }),
        Some(&token),
    )
    .await;

    // Curator (role >= 5) may add items.
    let (work_b, _) = seed_work(
        &db,
        "lists-cur-b",
        &format!("{TITLE_PREFIX} Cur B"),
        "Author Cur",
    )
    .await;
    let (status, body) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": work_b }),
        Some(&curator_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "curator add: {body}");

    // Curator may remove items.
    let (status, body) = delete_json(
        &app,
        &format!("/api/lists/{list_id}/items/{work_b}"),
        Some(&curator_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "curator remove: {body}");
    assert_eq!(body["removed"], true);

    // Curator may soft-delete the list.
    let (status, body) =
        delete_json(&app, &format!("/api/lists/{list_id}"), Some(&curator_token)).await;
    assert_eq!(status, StatusCode::OK, "curator delete: {body}");
    assert_eq!(body["removed"], true);

    // ...and the owner can no longer see it.
    let (status, _) = get_json(&app, &format!("/api/lists/{list_id}"), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn lists_auth_gates() {
    let _guard = db_guard();
    let db = pool().await;
    cleanup(&db).await;
    let user_id = seed_user(&db, USERNAME).await;
    let (work_a, _) = seed_work(
        &db,
        "lists-auth-a",
        &format!("{TITLE_PREFIX} Auth A"),
        "Author Auth",
    )
    .await;
    let app = app().await;
    let token = auth_header(user_id, 0, USERNAME);

    let (_, body) = post_json(
        &app,
        "/api/lists",
        json!({ "title": "Gated" }),
        Some(&token),
    )
    .await;
    let list_id = body["list"]["id"].as_i64().expect("list id") as i32;

    // Anonymous: list-mine, patch, delete, add/remove item all rejected 401.
    let (status, body) = get_json(&app, "/api/lists", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["err"], 401);
    let (status, body) = patch_json(
        &app,
        &format!("/api/lists/{list_id}"),
        json!({ "title": "x" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["err"], 401);
    let (status, body) = delete_json(&app, &format!("/api/lists/{list_id}"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["err"], 401);
    let (status, body) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": work_a }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["err"], 401);
    let (status, body) =
        delete_json(&app, &format!("/api/lists/{list_id}/items/{work_a}"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["err"], 401);

    // Adding a work that doesn't exist 404s even for the owner.
    let (status, _) = post_json(
        &app,
        &format!("/api/lists/{list_id}/items"),
        json!({ "work_id": 999_999_999 }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

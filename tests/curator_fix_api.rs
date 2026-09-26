//! DB-gated integration tests for the curator fix proposal + voting flow:
//! a fix is applied only after a quorum of OTHER curators approve it.
//!
//! Conventions (same as tests/heal_api.rs / admin_api.rs):
//! * Serialised via a global Mutex; marked #[ignore]; run with:
//!   `set -a; . ./.env; set +a; cargo test --test curator_fix_api -- --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows at the START so reruns
//!   never collide.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
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

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
}

async fn build_app() -> Router {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
    use fichub::server::AppState;

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
        rt_manager: fichub::realtime::ConnectionManager::new(),
        jwt_secret: "fichub-test-secret".into(),
        redis_client: None,
    });

    Router::new()
        .route(
            "/api/curator/content/{url_id}/propose",
            post(fichub::routes::curator_content::propose_fix),
        )
        .route(
            "/api/curator/content/proposals",
            get(fichub::routes::curator_content::list_proposals),
        )
        .route(
            "/api/curator/content/proposals/{id}/vote",
            post(fichub::routes::curator_content::vote_fix),
        )
        .route(
            "/api/curator/metadata/propose",
            post(fichub::routes::curator_content::propose_metadata_fix),
        )
        .route(
            "/api/curator/metadata/proposals",
            get(fichub::routes::curator_content::list_metadata_proposals),
        )
        .route(
            "/api/curator/metadata/proposals/{id}/vote",
            post(fichub::routes::curator_content::vote_metadata_fix),
        )
        .with_state(state)
}

fn auth_header(user_id: i32, trust_level: i16, username: &str) -> String {
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

/// `role` and `trust_level` are separate columns and both are live. The
/// trust gates read `trust_level` (50 of them, e.g.
/// work_proposals.rs:279, comments.rs:474, admin.rs:48), while
/// db/queries/proposals.rs:93 and routes/subsystems.rs:475 read `role`.
/// Seeding only `role` left `trust_level` at its column default of 0, so
/// every curator-gated endpoint returned 403 and the suite was exercising a
/// column no gate reads. Both are written here, from the same value, so the
/// fixture cannot drift back to testing nothing.
async fn seed_user(pool: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role, trust_level) VALUES ($1, 'test-hash', $2, LEAST($2, 6)) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role, trust_level = LEAST(EXCLUDED.trust_level, 6) RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(pool)
        .await
        .expect("seed_user failed")
        .get(0)
}

async fn cleanup(pool: &sqlx::PgPool, url_like: &str) {
    let _ = sqlx::query("DELETE FROM curator_fix_votes WHERE proposal_id IN (SELECT id FROM curator_fix_proposals WHERE url_id LIKE $1)")
        .bind(url_like)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM curator_fix_proposals WHERE url_id LIKE $1")
        .bind(url_like)
        .execute(pool)
        .await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn fix_applies_after_quorum_of_other_curators() {
    let _guard = db_guard();
    let app = build_app().await;
    let db = pool().await;
    cleanup(&db, "curatorfix_aaa%").await;

    let u1 = seed_user(&db, "curatorfix_admin1", 10).await;
    let u2 = seed_user(&db, "curatorfix_admin2", 10).await;
    let u3 = seed_user(&db, "curatorfix_admin3", 10).await;

    // u1 proposes a fix.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/curator/content/curatorfix_aaa1/propose")
                .header("content-type", "application/json")
                .header("authorization", auth_header(u1, 10, "curatorfix_admin1"))
                .body(Body::from(
                    r#"{"body_html":"<p>correct body</p>","reason":"wrong scrape"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let proposal_id = v["proposal_id"].as_i64().expect("proposal_id");
    assert_eq!(v["status"], "pending");

    // u1 cannot vote on own proposal → 400 error.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/curator/content/proposals/{proposal_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", auth_header(u1, 10, "curatorfix_admin1"))
                .body(Body::from(r#"{"vote":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "self-vote should 400"
    );
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        v["msg"].as_str().unwrap_or("").contains("own proposal"),
        "self-vote message should mention own proposal"
    );

    // u2 votes up → 1 vote, still pending (quorum is 2).
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/curator/content/proposals/{proposal_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", auth_header(u2, 10, "curatorfix_admin2"))
                .body(Body::from(r#"{"vote":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "pending", "one vote not quorum");

    // u3 votes up → quorum reached → applied.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/curator/content/proposals/{proposal_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", auth_header(u3, 10, "curatorfix_admin3"))
                .body(Body::from(r#"{"vote":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "applied", "quorum + net >= 1 → applied");
    assert_eq!(v["applied"], true);

    // The applied body should now be loadable from the cache.
    let config = fichub::config::Config::from_env();
    let loaded = fichub::body_cache::load_body(&config, "curatorfix_aaa1");
    assert!(loaded.is_some(), "approved fix should be in the body cache");
    if let Some(ch) = loaded {
        assert_eq!(ch[0].content, "<p>correct body</p>");
    }

    cleanup(&db, "curatorfix_aaa%").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn downvotes_reject_and_list_shows_status() {
    let _guard = db_guard();
    let app = build_app().await;
    let db = pool().await;
    cleanup(&db, "curatorfix_bbb%").await;

    let u1 = seed_user(&db, "curatorfix_r1", 10).await;
    let u2 = seed_user(&db, "curatorfix_r2", 10).await;
    let u3 = seed_user(&db, "curatorfix_r3", 10).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/curator/content/curatorfix_bbb1/propose")
                .header("content-type", "application/json")
                .header("authorization", auth_header(u1, 10, "curatorfix_r1"))
                .body(Body::from(
                    r#"{"body_html":"<p>bad body</p>","reason":"test"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let proposal_id = v["proposal_id"].as_i64().unwrap();

    // Two downvotes → rejected.
    let mut final_status = "pending".to_string();
    for (uid, name) in [(u2, "curatorfix_r2"), (u3, "curatorfix_r3")] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/curator/content/proposals/{proposal_id}/vote"))
                    .header("content-type", "application/json")
                    .header("authorization", auth_header(uid, 10, name))
                    .body(Body::from(r#"{"vote":-1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        let status = v["status"].as_str().unwrap_or("pending").to_string();
        if status == "rejected" {
            final_status = status;
            break;
        }
    }
    assert_eq!(final_status, "rejected", "two downvotes should reject");

    // The rejected proposal appears in the rejected list.
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/curator/content/proposals?status=rejected")
                .header("authorization", auth_header(u2, 10, "curatorfix_r2"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let proposals = v["proposals"].as_array().unwrap();
    assert!(
        proposals.iter().any(|p| p["id"] == proposal_id),
        "rejected proposal should appear in rejected list"
    );

    cleanup(&db, "curatorfix_bbb%").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn metadata_proposal_applies_after_quorum() {
    let _guard = db_guard();
    let app = build_app().await;
    let db = pool().await;

    // Cleanup metadata proposals referencing a fixture work.
    let _ = sqlx::query("DELETE FROM curator_metadata_votes WHERE proposal_id IN (SELECT id FROM curator_metadata_proposals WHERE reason LIKE 'meta_test%')").execute(&db).await;
    let _ = sqlx::query("DELETE FROM curator_metadata_proposals WHERE reason LIKE 'meta_test%'")
        .execute(&db)
        .await;

    let u1 = seed_user(&db, "meta_test_admin1", 10).await;
    let u2 = seed_user(&db, "meta_test_admin2", 10).await;
    let u3 = seed_user(&db, "meta_test_admin3", 10).await;

    // Seed a work.
    let work_id: i32 = sqlx::query(
        "INSERT INTO works (canonical_title, canonical_author, description) VALUES ('Old Title', 'Old Author', 'Old desc') RETURNING id",
    )
    .fetch_one(&db)
    .await
    .expect("seed work")
    .get(0);

    // u1 proposes a title change.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/curator/metadata/propose")
                .header("content-type", "application/json")
                .header("authorization", auth_header(u1, 10, "meta_test_admin1"))
                .body(Body::from(format!(
                    r#"{{"work_id":{work_id},"field":"title","old_value":"Old Title","new_value":"New Correct Title","reason":"meta_test title fix"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let proposal_id = v["proposal_id"].as_i64().expect("proposal_id");

    // u2 votes up → not yet quorum (needs 2 others besides proposer).
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/curator/metadata/proposals/{proposal_id}/vote"
                ))
                .header("content-type", "application/json")
                .header("authorization", auth_header(u2, 10, "meta_test_admin2"))
                .body(Body::from(r#"{"vote":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "pending", "one vote is not quorum yet: {v}");

    // u3 votes up → quorum (2 others) + net 2 ≥ 1 → applied.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/curator/metadata/proposals/{proposal_id}/vote"
                ))
                .header("content-type", "application/json")
                .header("authorization", auth_header(u3, 10, "meta_test_admin3"))
                .body(Body::from(r#"{"vote":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "applied", "quorum should apply: {v}");
    assert_eq!(v["applied"], true);

    // The work's canonical title is updated.
    let title: String = sqlx::query("SELECT canonical_title FROM works WHERE id = $1")
        .bind(work_id)
        .fetch_one(&db)
        .await
        .expect("fetch work")
        .get(0);
    assert_eq!(
        title, "New Correct Title",
        "work canonical title updated after approval"
    );

    // Cleanup.
    let _ = sqlx::query("DELETE FROM curator_metadata_votes WHERE proposal_id = $1")
        .bind(proposal_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM curator_metadata_proposals WHERE id = $1")
        .bind(proposal_id)
        .execute(&db)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE id = $1")
        .bind(work_id)
        .execute(&db)
        .await;
}

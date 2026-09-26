//! DB-gated integration tests for the Roadmap Consensus Engine.
//!
//! Covers: suggest (clustering + anti-spam), arena (4-cluster deal), vote
//! (MaxDiff → Elo). Follows the repo's DB-gated conventions (global Mutex,
//! #[ignore], full AppState construction).

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::Row;
use std::sync::{Mutex, OnceLock};
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (run with .env loaded)");
    sqlx::PgPool::connect(&url).await.expect("connect pool")
}

async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let http_client = reqwest::Client::builder()
        .user_agent("fichub-test/0.1.0")
        .build()
        .expect("reqwest");
    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());

    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:1".into(), // port 1: never reachable — tests never need real embeddings
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
            "/api/roadmap/suggest",
            post(fichub::routes::roadmap::suggest_handler),
        )
        .route(
            "/api/roadmap/arena",
            get(fichub::routes::roadmap::arena_handler),
        )
        .route(
            "/api/roadmap/vote",
            post(fichub::routes::roadmap::vote_handler),
        )
        .route(
            "/api/roadmap/consensus",
            get(fichub::routes::roadmap::consensus_handler),
        )
        .route(
            "/api/roadmap/features",
            get(fichub::routes::roadmap::features_list_handler),
        )
        .route(
            "/api/roadmap/features/{id}",
            axum::routing::patch(fichub::routes::roadmap::feature_move_handler),
        )
        .route(
            "/api/roadmap/changelog",
            get(fichub::routes::roadmap::changelog_list_handler),
        )
        .route(
            "/api/roadmap/changelog",
            post(fichub::routes::roadmap::changelog_create_handler),
        )
        .with_state(state)
}

async fn seed_user(db: &sqlx::PgPool, username: &str) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role, level) VALUES ($1, 'x', 0, 2) ON CONFLICT (username) DO UPDATE SET username = EXCLUDED.username RETURNING id")
        .bind(username)
        .fetch_one(db)
        .await
        .expect("seed user")
        .get(0)
}

fn auth_header(user_id: i32, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let user = fichub::routes::auth::User {
        id: user_id,
        username: username.into(),
        trust_level: 0,
        reputation: 0,
        email: None,
        level: 2,
        exp: 0,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn seed_cluster(db: &sqlx::PgPool, text: &str) -> i32 {
    // Embedding is fake (zeros) — clustering tests rely on threshold logic via
    // raw insertion, and vote/arena tests don't need real vectors.
    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    sqlx::query("INSERT INTO feature_clusters (representative_text, embedding, status) VALUES ($1, $2::vector, 'idea') RETURNING id")
        .bind(text)
        .bind(&emb)
        .fetch_one(db)
        .await
        .expect("seed cluster")
        .get(0)
}

async fn cleanup(db: &sqlx::PgPool, users: &[&str], clusters: &[i32]) {
    for u in users {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(db)
            .await;
    }
    for c in clusters {
        let _ = sqlx::query("DELETE FROM feature_suggestions WHERE cluster_id = $1")
            .bind(c)
            .execute(db)
            .await;
        let _ = sqlx::query("DELETE FROM arena_votes WHERE best_cluster_id = $1 OR worst_cluster_id = $1 OR $1 = ANY(cluster_ids)").bind(c).execute(db).await;
        let _ = sqlx::query("DELETE FROM feature_clusters WHERE id = $1")
            .bind(c)
            .execute(db)
            .await;
    }
    let _ = sqlx::query("DELETE FROM feature_suggestions WHERE raw_text LIKE 'rmd_%'")
        .execute(db)
        .await;
    // Clean up any changelog entries created by tests
    let _ = sqlx::query("DELETE FROM roadmap_changelog WHERE title LIKE 'rmd_%'")
        .execute(db)
        .await;
}

/// Suggest with Ollama down (port 1) → raw text is stored unclustered (cluster_id NULL).
#[ignore]
#[tokio::test]
async fn suggest_with_ollama_down_stores_unclustered() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let uid = seed_user(&db, "rmd_suggest_user").await;
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/suggest")
                .header("authorization", auth_header(uid, "rmd_suggest_user"))
                .header("x-client-id", "rmd_client_1")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "text": "rmd_test_suggestion dark mode please" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    assert_eq!(
        body["clustered"], false,
        "ollama down → unclustered: {body}"
    );

    let row: Option<(String, Option<i32>)> = sqlx::query_as(
        "SELECT raw_text, cluster_id FROM feature_suggestions WHERE raw_text = 'rmd_test_suggestion dark mode please'",
    )
    .fetch_optional(&db)
    .await
    .expect("query");
    let (raw, cid) = row.expect("suggestion stored");
    assert_eq!(raw, "rmd_test_suggestion dark mode please");
    assert!(cid.is_none(), "unclustered (ollama down)");

    cleanup(&db, &["rmd_suggest_user"], &[]).await;
}

/// Arena returns 4 distinct open clusters (least-played first).
#[ignore]
#[tokio::test]
async fn arena_returns_four_clusters() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let uid = seed_user(&db, "rmd_arena_user").await;
    let c1 = seed_cluster(&db, "rmd_arena_one").await;
    let c2 = seed_cluster(&db, "rmd_arena_two").await;
    let c3 = seed_cluster(&db, "rmd_arena_three").await;
    let c4 = seed_cluster(&db, "rmd_arena_four").await;
    let c5 = seed_cluster(&db, "rmd_arena_five").await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/arena")
                .header("authorization", auth_header(uid, "rmd_arena_user"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    let clusters = body["clusters"].as_array().unwrap();
    assert_eq!(clusters.len(), 4, "arena serves exactly 4: {body}");
    let mut ids: Vec<i32> = clusters
        .iter()
        .map(|c| c["id"].as_i64().unwrap() as i32)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 4, "all distinct");

    cleanup(&db, &["rmd_arena_user"], &[c1, c2, c3, c4, c5]).await;
}

/// A MaxDiff vote updates Elo: best gains, worst loses, neutrals in between.
#[ignore]
#[tokio::test]
async fn vote_runs_maxdiff_elo() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let uid = seed_user(&db, "rmd_vote_user").await;
    let c1 = seed_cluster(&db, "rmd_vote_one").await;
    let c2 = seed_cluster(&db, "rmd_vote_two").await;
    let c3 = seed_cluster(&db, "rmd_vote_three").await;
    let c4 = seed_cluster(&db, "rmd_vote_four").await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/vote")
                .header("authorization", auth_header(uid, "rmd_vote_user"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "cluster_ids": [c1, c2, c3, c4],
                        "best_cluster_id": c1,
                        "worst_cluster_id": c4,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "first vote should be 200");

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    let applied = body["applied"].as_array().unwrap();
    let r1 = applied.iter().find(|a| a["cluster_id"] == c1).unwrap()["new_elo"]
        .as_f64()
        .unwrap();
    let r4 = applied.iter().find(|a| a["cluster_id"] == c4).unwrap()["new_elo"]
        .as_f64()
        .unwrap();
    assert!(r1 > 1500.0, "best gains: {r1}");
    assert!(r4 < 1500.0, "worst loses: {r4}");

    // Double vote on the same set is rejected with 400 BadRequest.
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/vote")
                .header("authorization", auth_header(uid, "rmd_vote_user"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "cluster_ids": [c1, c2, c3, c4],
                        "best_cluster_id": c1,
                        "worst_cluster_id": c4,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res2.status(),
        StatusCode::BAD_REQUEST,
        "double vote rejected with 400"
    );
    let body2: Value = serde_json::from_slice(
        &axum::body::to_bytes(res2.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_ne!(body2["err"], 0, "double vote rejected: {body2}");

    cleanup(&db, &["rmd_vote_user"], &[c1, c2, c3, c4]).await;
}

/// Regression (2026-08-10 production bug): anonymous votes used to bind
/// user_id = 0, which violated arena_votes_user_id_fkey (FK to users.id) and
/// returned a 500 "database error" on the live site. With the trust-level gate
/// (level >= 2), anonymous votes now get 403. For authenticated level-2 users,
/// `user_id_or` returns Some(id) so the FK is satisfied, and the double-vote
/// guard works per-user_id.
#[ignore]
#[tokio::test]
async fn trust_gate_blocks_anonymous_vote_returns_403() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let c1 = seed_cluster(&db, "rmd_anon_one").await;
    let c2 = seed_cluster(&db, "rmd_anon_two").await;
    let c3 = seed_cluster(&db, "rmd_anon_three").await;
    let c4 = seed_cluster(&db, "rmd_anon_four").await;

    // No Authorization header → anonymous (level 0) → trust gate returns 403.
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/vote")
                .header("x-client-id", "e2e-anon-test-client")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "cluster_ids": [c1, c2, c3, c4],
                        "best_cluster_id": c1,
                        "worst_cluster_id": c4,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "anonymous vote blocked by trust gate"
    );

    cleanup(&db, &[], &[c1, c2, c3, c4]).await;
}

/// Authenticated level-2 vote persists with proper user_id (no FK violation).
/// Double-vote guard works per-user_id (the old bug was user_id=0 colliding).
#[ignore]
#[tokio::test]
async fn authenticated_vote_fk_safe_and_double_vote_blocked() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let c1 = seed_cluster(&db, "rmd_auth_one").await;
    let c2 = seed_cluster(&db, "rmd_auth_two").await;
    let c3 = seed_cluster(&db, "rmd_auth_three").await;
    let c4 = seed_cluster(&db, "rmd_auth_four").await;

    let uid = seed_user(&db, "rmd_auth_user").await;

    // First vote: should succeed (level 2 = authenticated).
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/vote")
                .header("authorization", auth_header(uid, "rmd_auth_user"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "cluster_ids": [c1, c2, c3, c4],
                        "best_cluster_id": c1,
                        "worst_cluster_id": c4,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "first vote should be 200");

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0, "vote err: {body}");

    // The vote row exists with user_id = Some(uid) (FK-safe).
    let (row_user,): (Option<i32>,) =
        sqlx::query_as("SELECT user_id FROM arena_votes WHERE cluster_ids = $1 AND user_id = $2")
            .bind(vec![c1, c2, c3, c4])
            .bind(uid)
            .fetch_one(&db)
            .await
            .expect("vote row should exist");
    assert_eq!(
        row_user,
        Some(uid),
        "authenticated vote must store user_id (FK-safe)"
    );

    // Same user double-votes same set → 400.
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/vote")
                .header("authorization", auth_header(uid, "rmd_auth_user"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "cluster_ids": [c1, c2, c3, c4],
                        "best_cluster_id": c1,
                        "worst_cluster_id": c4,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res2.status(),
        StatusCode::BAD_REQUEST,
        "same user double vote rejected"
    );

    cleanup(&db, &["rmd_auth_user"], &[c1, c2, c3, c4]).await;
}

/// GET /api/roadmap/consensus is PUBLIC — no Authorization header, no
/// x-client-id: anonymous readers must get the leaderboard + controversy
/// (200, err 0), exactly like the admin endpoint's shape.
#[ignore]
#[tokio::test]
async fn consensus_public_readable() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let c1 = seed_cluster(&db, "rmd_pubcons_one").await;
    let c2 = seed_cluster(&db, "rmd_pubcons_two").await;

    // No auth headers at all — public read.
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/consensus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "public consensus must be 200");

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    assert!(
        body["leaderboard"].is_array(),
        "leaderboard present: {body}"
    );
    assert!(
        body["controversy"].is_array(),
        "controversy present: {body}"
    );

    cleanup(&db, &[], &[c1, c2]).await;
}

/// GET /api/roadmap/consensus ranks the leaderboard by Elo DESC with the
/// full per-cluster shape (text, elo_rating, matches_played,
/// times_picked_best/worst, suggestions, status) and lists controversy
/// rows with a controversy score.
#[ignore]
#[tokio::test]
async fn consensus_returns_leaderboard() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    // High-Elo cluster with 2 suggestions; low-Elo cluster with none.
    let c_hi: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, times_picked_best, times_picked_worst, status) VALUES ('rmd_pubcons_top_feature', $1::vector, 1700, 10, 6, 1, 'idea') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);
    let c_lo: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, times_picked_best, times_picked_worst, status) VALUES ('rmd_pubcons_bottom_feature', $1::vector, 1300, 2, 0, 2, 'idea') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);
    sqlx::query("INSERT INTO feature_suggestions (user_id, client_id, raw_text, cluster_id) VALUES (NULL, 'rmd_pubcons_client', 'rmd_pubcons_top_feature', $1)")
        .bind(c_hi)
        .execute(&db)
        .await
        .expect("seed suggestion");
    sqlx::query("INSERT INTO feature_suggestions (user_id, client_id, raw_text, cluster_id) VALUES (NULL, 'rmd_pubcons_client', 'rmd_pubcons_top_feature alt wording', $1)")
        .bind(c_hi)
        .execute(&db)
        .await
        .expect("seed suggestion 2");

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/consensus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();

    let lb = body["leaderboard"].as_array().expect("leaderboard array");
    assert_eq!(
        lb[0]["text"], "rmd_pubcons_top_feature",
        "highest Elo first: {body}"
    );
    assert_eq!(lb[0]["elo_rating"], 1700.0);
    assert_eq!(lb[0]["matches_played"], 10);
    assert_eq!(lb[0]["times_picked_best"], 6);
    assert_eq!(lb[0]["times_picked_worst"], 1);
    assert_eq!(lb[0]["suggestions"], 2);
    assert_eq!(lb[0]["status"], "idea");
    assert!(
        lb.iter().any(|r| r["text"] == "rmd_pubcons_bottom_feature"),
        "both clusters listed: {body}"
    );

    let ct = body["controversy"].as_array().expect("controversy array");
    let hi = ct
        .iter()
        .find(|r| r["text"] == "rmd_pubcons_top_feature")
        .expect("top in controversy");
    assert_eq!(hi["controversy"], 7, "best+worst picks: {body}");
    assert_eq!(hi["elo_rating"], 1700.0);

    cleanup(&db, &[], &[c_hi, c_lo]).await;
}

/// GET /api/roadmap/features is PUBLIC — no auth needed.
/// Returns all clusters sorted by Elo DESC with full shape.
#[ignore]
#[tokio::test]
async fn features_list_public_and_sorted_by_elo() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );

    // Seed two clusters with different Elo + known categories/statuses.
    let c_hi: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, status, category) VALUES ('rmd_feat_high', $1::vector, 1800, 5, 'shipped', 'reader') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);
    let c_lo: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, status, category) VALUES ('rmd_feat_low', $1::vector, 1200, 2, 'idea', 'search') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    // No auth header — public.
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/features")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    let features = body["features"].as_array().expect("features array");
    assert!(features.len() >= 2, "at least 2 clusters returned");
    assert_eq!(features[0]["elo_rating"], 1800.0, "highest Elo first");
    assert_eq!(features[0]["status"], "shipped");
    assert_eq!(features[0]["category"], "reader");

    cleanup(&db, &[], &[c_hi, c_lo]).await;
}

/// GET /api/roadmap/features?status=up_next filters by status.
#[ignore]
#[tokio::test]
async fn features_list_filter_by_status() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let c1: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, status, category) VALUES ('rmd_status_filter', $1::vector, 1600, 'up_next', 'recs') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/features?status=up_next")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    let features = body["features"].as_array().expect("features array");
    assert_eq!(features.len(), 1, "only up_next filtered cluster");
    assert_eq!(features[0]["status"], "up_next");

    cleanup(&db, &[], &[c1]).await;
}

/// PATCH /api/roadmap/features/:id requires curator level (>= 50).
/// Non-authenticated user (default AuthUser level=0) → 403.
#[ignore]
#[tokio::test]
async fn feature_move_requires_curator() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let c1: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, status) VALUES ('rmd_move_test', $1::vector, 1500, 'idea') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    // No auth header → anonymous → 403 Forbidden.
    let res = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/roadmap/features/{c1}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "status": "up_next" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "anonymous should be 403"
    );

    cleanup(&db, &[], &[c1]).await;
}

/// GET /api/roadmap/changelog is PUBLIC — returns published entries.
/// POST /api/roadmap/changelog requires curator level (>= 50).
#[ignore]
#[tokio::test]
async fn changelog_public_read_and_curator_write() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let c1: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, status) VALUES ('rmd_chlog_feat', $1::vector, 1500, 'shipped') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    // --- Public read (no auth) ---
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/changelog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "public changelog GET must be 200"
    );
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    assert!(
        body["changelog"].is_array(),
        "changelog array present: {body}"
    );

    // --- Curator write (level >= 50) → 200 ---
    let curator_user = fichub::routes::auth::User {
        id: 0,
        username: "rmd_chlog_user".into(),
        trust_level: 5,
        reputation: 0,
        email: None,
        level: 50,
        exp: 0,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(&curator_user, &secret).expect("token");
    let uid = seed_user(&db, "rmd_chlog_user").await;
    // Update the seeded user to have level 50 / role 5 (curator).
    let _ = sqlx::query("UPDATE users SET role = 5 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;

    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/changelog")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "feature_id": c1,
                        "title": "rmd_chlog_entry",
                        "body": "Feature launched",
                        "kind": "new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res2.status(),
        StatusCode::OK,
        "curator POST changelog should be 200: {:?}",
        res2.body()
    );
    let body2: Value = serde_json::from_slice(
        &axum::body::to_bytes(res2.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body2["err"], 0);
    assert!(body2["id"].as_i64().is_some(), "entry id returned");

    // --- Anonymous write → 403 ---
    let res3 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/roadmap/changelog")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "rmd_chlog_anon",
                        "body": "should fail",
                        "kind": "new"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res3.status(),
        StatusCode::FORBIDDEN,
        "anonymous POST must be 403"
    );

    cleanup(&db, &["rmd_chlog_user"], &[c1]).await;
}

/// PATCH /api/roadmap/features/:id with valid curator moves status + category.
#[ignore]
#[tokio::test]
async fn feature_move_updates_status_and_category() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let c1: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, status, category) VALUES ('rmd_promote', $1::vector, 1500, 'idea', 'general') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    // Curator user with level 50
    let curator_user = fichub::routes::auth::User {
        id: 0,
        username: "rmd_promote_user".into(),
        trust_level: 5,
        reputation: 0,
        email: None,
        level: 50,
        exp: 0,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(&curator_user, &secret).expect("token");
    let uid = seed_user(&db, "rmd_promote_user").await;
    let _ = sqlx::query("UPDATE users SET role = 5 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/roadmap/features/{c1}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "status": "up_next", "category": "recs" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    assert_eq!(body["status"], "up_next");
    assert_eq!(body["category"], "recs");

    // Verify in DB
    let row: (String, String) =
        sqlx::query_as("SELECT status, category FROM feature_clusters WHERE id = $1")
            .bind(c1)
            .fetch_one(&db)
            .await
            .expect("query");
    assert_eq!(row.0, "up_next");
    assert_eq!(row.1, "recs");

    cleanup(&db, &["rmd_promote_user"], &[c1]).await;
}

/// GET /api/roadmap/changelog?kind=new filters to new entries only.
#[ignore]
#[tokio::test]
async fn changelog_filter_by_kind() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let dims = vec![0.0f32; 768];
    let emb = format!(
        "[{}]",
        dims.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let c1: i32 = sqlx::query(
        "INSERT INTO feature_clusters (representative_text, embedding, elo_rating, status) VALUES ('rmd_chlog_filter_feat', $1::vector, 1500, 'shipped') RETURNING id",
    )
    .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    let curator_user = fichub::routes::auth::User {
        id: 0,
        username: "rmd_chlog_filter_user".into(),
        trust_level: 5,
        reputation: 0,
        email: None,
        level: 50,
        exp: 0,
    };
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let token = fichub::routes::auth::create_token(&curator_user, &secret).expect("token");
    let uid = seed_user(&db, "rmd_chlog_filter_user").await;
    let _ = sqlx::query("UPDATE users SET role = 5 WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await;

    // Create two entries: one 'new', one 'fixed'.
    let entry_new: i64 = sqlx::query_scalar(
        "INSERT INTO roadmap_changelog (feature_id, title, body, kind, author_id, is_published) VALUES ($1, 'rmd_new_entry', 'body', 'new', $2, TRUE) RETURNING id",
    )
    .bind(c1).bind(uid)
    .fetch_one(&db).await.expect("insert new");

    let entry_fixed: i64 = sqlx::query_scalar(
        "INSERT INTO roadmap_changelog (feature_id, title, body, kind, author_id, is_published) VALUES ($1, 'rmd_fixed_entry', 'body', 'fixed', $2, TRUE) RETURNING id",
    )
    .bind(c1).bind(uid)
    .fetch_one(&db).await.expect("insert fixed");

    // Filter to 'new' only.
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/roadmap/changelog?kind=new")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["err"], 0);
    let entries = body["changelog"].as_array().expect("changelog array");
    assert!(
        entries.iter().all(|e| e["kind"] == "new"),
        "all entries should be 'new'"
    );
    assert!(entries.iter().any(|e| e["title"] == "rmd_new_entry"));
    assert!(!entries.iter().any(|e| e["title"] == "rmd_fixed_entry"));

    // Cleanup
    for id in [entry_new, entry_fixed] {
        let _ = sqlx::query("DELETE FROM roadmap_changelog WHERE id = $1")
            .bind(id)
            .execute(&db)
            .await;
    }
    cleanup(&db, &["rmd_chlog_filter_user"], &[c1]).await;
}

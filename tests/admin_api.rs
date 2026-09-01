/// DB-gated integration tests for admin endpoints.
//
// Regression coverage for the behavioral-security admin API (Zero-PII):
// - `/api/admin/bots` returns an anonymized leaderboard of unusual-behavior
//   clients (client_id + signals only, never raw IPs), flags mirror/stuffing
//   patterns, and requires role >= 10.
// - `/api/admin/bots/{client_id}/shadowban` + `/unshadowban` toggle the Redis
//   shadowban set membership.
// - `/api/admin/moderation/comments` (triage queue), `/api/admin/stats`,
//   `/api/admin/users` + role/ban toggles, and the auto-tag review queue
//   (approve/dismiss) — all role >= 10.
//
// NOTE: `RedisBucketLimiter`'s shadowban probe (`is_shadowbanned`) runs
// `tokio::task::block_in_place`, which panics on the current-thread runtime,
// so every test must be `#[tokio::test(flavor = "multi_thread")]`.

use std::sync::{Mutex, OnceLock};
use axum::{body::Body, http::{Request, StatusCode}, routing::{get, post, put}, Router};
use serde_json::{json, Value};
use sqlx::Row;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|p| p.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (run with .env loaded)");
    sqlx::PgPool::connect(&url).await.expect("connect pool")
}

async fn app() -> Router {
    use fichub::server::AppState;
    use std::sync::Arc;

    let config = fichub::config::Config::from_env();
    let db = pool().await;
    let redis_client = redis::Client::open(config.redis_url.clone()).expect("invalid REDIS_URL");
    let redis = redis_client.get_multiplexed_async_connection().await.expect("redis conn");
    let http_client = reqwest::Client::builder().user_agent("fichub-test/0.1.0").build().expect("reqwest");
    let scraper_registry = Arc::new(fichub::scrape::registry::ScraperRegistry::new());

    let ollama_client = fichub::services::ollama::OllamaClient::new(
        "http://127.0.0.1:11434".into(),
        "nomic-embed-text".into(),
        http_client.clone(),
    );
    let mut state_cfg = config.clone();
    state_cfg.pow_difficulty = 4;
    let state = Arc::new(AppState {
        config: state_cfg.clone(),
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
            fichub::limiter::redis_bucket::RedisBucketLimiter::with_config(
                redis_client.get_multiplexed_async_connection().await.expect("redis"),
                false,
                Some(&state_cfg),
            )
            .await
            .expect("rate limiter"),
        ),
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
        .route("/api/admin/bots", get(fichub::routes::admin::admin_bots))
        .route("/api/admin/bots/{client_id}/shadowban", post(fichub::routes::admin::admin_bot_shadowban))
        .route("/api/admin/bots/{client_id}/unshadowban", post(fichub::routes::admin::admin_bot_unshadowban))
        .route("/api/admin/realtime", get(fichub::routes::admin::admin_realtime))
        .route("/api/admin/search-analytics", get(fichub::routes::admin::admin_search_analytics))
        .route("/api/admin/roadmap-consensus", get(fichub::routes::admin::admin_roadmap_consensus))
        .route("/api/admin/moderation/comments", get(fichub::routes::admin::moderation_comments))
        .route("/api/admin/moderation/comments/{id}/hide", post(fichub::routes::admin::admin_hide_comment))
        .route("/api/admin/moderation/comments/{id}/delete", post(fichub::routes::admin::admin_delete_comment))
        .route("/api/admin/blacklist", get(fichub::routes::admin::list_blacklist))
        .route("/api/admin/blacklist/fic", post(fichub::routes::admin::blacklist_fic))
        .route("/api/admin/blacklist/author", post(fichub::routes::admin::blacklist_author))
        .route("/api/admin/stats", get(fichub::routes::admin::admin_stats))
        .route("/api/admin/users", get(fichub::routes::admin::admin_users))
        .route("/api/admin/users/{id}/role", put(fichub::routes::admin::set_user_role))
        .route("/api/admin/users/{id}/ban", put(fichub::routes::admin::toggle_ban))
        .route("/api/admin/auto-tag/queue", get(fichub::routes::auto_tag::auto_tag_queue))
        .route("/api/admin/auto-tag/approve/{url_id}/{tag_id}", post(fichub::routes::auto_tag::auto_tag_approve))
        .route("/api/admin/auto-tag/dismiss/{url_id}/{tag_id}", post(fichub::routes::auto_tag::auto_tag_dismiss))
        .with_state(state)
}

async fn seed_admin_user(db: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ($1, 'x', $2) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(db)
        .await
        .expect("seed user")
        .get(0)
}

fn auth_header(user_id: i32, username: &str, role: i16) -> String {
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

async fn seed_bot_score(db: &sqlx::PgPool, client_id: &str, requests: i32, downloads: i32, failed_auths: i32) {
    sqlx::query(
        r#"INSERT INTO bot_scores (ip, client_id, window_start, requests, downloads, failed_auths, export_ratio)
           VALUES ('127.0.0.1', $1, now() - interval '2 hours', $2, $3, $4, CASE WHEN $2 = 0 THEN 0 ELSE $3::float/$2::float END)
           ON CONFLICT (ip, client_id, window_start) DO UPDATE SET
            requests = EXCLUDED.requests, downloads = EXCLUDED.downloads,
            failed_auths = EXCLUDED.failed_auths, export_ratio = EXCLUDED.export_ratio"#,
    )
    .bind(client_id)
    .bind(requests)
    .bind(downloads)
    .bind(failed_auths)
    .execute(db)
    .await
    .expect("seed bot_score");
}

async fn cleanup(db: &sqlx::PgPool, users: &[&str], clients: &[&str]) {
    for u in users {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1").bind(u).execute(db).await;
    }
    for c in clients {
        let _ = sqlx::query("DELETE FROM bot_scores WHERE client_id = $1").bind(c).execute(db).await;
    }
}

async fn send(app: &Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header("authorization", t);
    }
    let req = match body {
        Some(v) => builder
            .header("content-type", "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

/// /api/admin/bots requires role >= 10.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_bots_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let low_role = seed_admin_user(&db, "adminit_bots_low", 5).await;
    let (s, _) = send(&app, "GET", "/api/admin/bots", Some(&auth_header(low_role, "adminit_bots_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    cleanup(&db, &["adminit_bots_low"], &[]).await;
}

/// /api/admin/bots returns anonymized leaderboard: client_id + signals, NO raw
/// IPs; flags mirror (high export ratio) and stuffing (failed auths).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_bots_returns_anonymized_flags() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_bots_admin", 10).await;
    // A mirror bot: 100 requests, 99 downloads (ratio 0.99) + stuffing (8 failed auths)
    seed_bot_score(&db, "botit_mirror_1", 100, 99, 8).await;
    // A normal reader: 50 requests, 1 download (ratio 0.02)
    seed_bot_score(&db, "botit_reader_1", 50, 1, 0).await;

    let (s, body) = send(&app, "GET", "/api/admin/bots?min_requests=10", Some(&auth_header(admin, "adminit_bots_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["err"], 0);

    let clients = body["clients"].as_array().unwrap();
    // The mirror bot ranks first (highest bot_score).
    let top = &clients[0];
    assert_eq!(top["client_id"], "botit_mirror_1");
    assert!(top["flags"].as_array().unwrap().iter().any(|f| f == "mirror"));
    assert!(top["flags"].as_array().unwrap().iter().any(|f| f == "stuffing"));
    assert!(top["bot_score"].as_f64().unwrap() > 0.9);
    // No raw IP anywhere in the payload (Zero-PII).
    let raw = body.to_string();
    assert!(!raw.contains("127.0.0.1"), "must not expose raw IPs: {raw}");

    // The normal reader is present but low-ranked and unflagged.
    let reader = clients.iter().find(|c| c["client_id"] == "botit_reader_1").expect("reader present");
    assert!(reader["flags"].as_array().unwrap().is_empty());
    assert!(reader["bot_score"].as_f64().unwrap() < 0.1);

    cleanup(&db, &["adminit_bots_admin"], &["botit_mirror_1", "botit_reader_1"]).await;
}

/// /api/admin/realtime returns cheap live aggregates (counts only, no PII).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_realtime_returns_counts() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_realtime_admin", 10).await;
    let (s, body) = send(&app, "GET", "/api/admin/realtime", Some(&auth_header(admin, "adminit_realtime_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["err"], 0);
    assert!(body["exports_5m"].as_i64().is_some());
    assert!(body["searches_5m"].as_i64().is_some());
    assert!(body["flagged_clients_1h"].as_i64().is_some());
    assert!(body["active_ips_1h"].as_i64().is_some());
    assert!(body["redis"]["ok"].is_boolean());
    // Zero-PII: no client ids, no ips, no query text in the payload.
    let raw = body.to_string();
    assert!(!raw.contains("127.0.0.1"), "no raw IPs: {raw}");

    cleanup(&db, &["adminit_realtime_admin"], &[]).await;
}

/// /api/admin/roadmap-consensus returns leaderboard + controversy (role 10).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_roadmap_consensus_returns_ranked_features() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_consensus_admin", 10).await;
    // Seed two clusters (fake zero embeddings) with different Elo.
    let dims = vec![0.0f32; 768];
    let emb = format!("[{}]", dims.iter().map(|f| format!("{f:.6}")).collect::<Vec<_>>().join(","));
    let c_hi: i32 = sqlx::query("INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, times_picked_best, times_picked_worst) VALUES ('consensus_top_feature', $1::vector, 1700, 10, 6, 1) RETURNING id")
        .bind(&emb).fetch_one(&db).await.expect("seed").get(0);
    let c_lo: i32 = sqlx::query("INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, times_picked_best, times_picked_worst) VALUES ('consensus_bottom_feature', $1::vector, 1300, 2, 0, 2) RETURNING id")
        .bind(&emb).fetch_one(&db).await.expect("seed").get(0);

    let (s, body) = send(&app, "GET", "/api/admin/roadmap-consensus", Some(&auth_header(admin, "adminit_consensus_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);
    let lb = body["leaderboard"].as_array().unwrap();
    assert_eq!(lb[0]["text"], "consensus_top_feature", "highest Elo first: {body}");
    assert_eq!(lb[0]["elo_rating"], 1700.0);
    let ct = body["controversy"].as_array().unwrap();
    assert!(ct.iter().any(|r| r["text"] == "consensus_top_feature"), "controversy includes top: {body}");

    // Cleanup
    let _ = sqlx::query("DELETE FROM feature_suggestions WHERE cluster_id = $1 OR cluster_id = $2").bind(c_hi).bind(c_lo).execute(&db).await;
    let _ = sqlx::query("DELETE FROM arena_votes WHERE best_cluster_id = $1 OR worst_cluster_id = $1 OR best_cluster_id = $2 OR worst_cluster_id = $2").bind(c_hi).bind(c_lo).execute(&db).await;
    let _ = sqlx::query("DELETE FROM feature_clusters WHERE id = $1 OR id = $2").bind(c_hi).bind(c_lo).execute(&db).await;
    cleanup(&db, &["adminit_consensus_admin"], &[]).await;
}

/// POST /api/admin/bots/{client_id}/shadowban sets the Redis shadowban set
/// membership (checked via SISMEMBER), unshadowban clears it. Role < 10 → 403.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_bot_shadowban_roundtrip() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let redis = redis::Client::open(redis_url.as_str())
        .expect("open redis")
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");

    let admin = seed_admin_user(&db, "adminit_shadow_admin", 10).await;
    seed_bot_score(&db, "botit_shadow_1", 100, 99, 8).await;

    // Clean any leftover membership from a crashed run.
    let _: i64 = redis::cmd("SREM")
        .arg(fichub::limiter::redis_bucket::SHADOWBAN_SET)
        .arg("botit_shadow_1")
        .query_async(&mut redis.clone())
        .await
        .unwrap_or(0);

    // Shadowban: role 10 → 200 {err:0, shadowbanned:true}; SISMEMBER true.
    let (s, body) = send(&app, "POST", "/api/admin/bots/botit_shadow_1/shadowban", Some(&auth_header(admin, "adminit_shadow_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["err"], 0);
    assert_eq!(body["shadowbanned"], true);
    let in_set: bool = redis::cmd("SISMEMBER")
        .arg(fichub::limiter::redis_bucket::SHADOWBAN_SET)
        .arg("botit_shadow_1")
        .query_async(&mut redis.clone())
        .await
        .expect("sismember");
    assert!(in_set, "client must be in the shadowban set after POST shadowban");

    // admin_bots now reports shadowbanned: true for the flagged client.
    let (_, body2) = send(&app, "GET", "/api/admin/bots?min_requests=10", Some(&auth_header(admin, "adminit_shadow_admin", 10)), None).await;
    let row = body2["clients"].as_array().unwrap().iter()
        .find(|c| c["client_id"] == "botit_shadow_1")
        .expect("flagged client present");
    assert_eq!(row["shadowbanned"], true);

    // Unshadowban: 200 {err:0, shadowbanned:false}; SISMEMBER false.
    let (s, body3) = send(&app, "POST", "/api/admin/bots/botit_shadow_1/unshadowban", Some(&auth_header(admin, "adminit_shadow_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body3["err"], 0);
    assert_eq!(body3["shadowbanned"], false);
    let in_set: bool = redis::cmd("SISMEMBER")
        .arg(fichub::limiter::redis_bucket::SHADOWBAN_SET)
        .arg("botit_shadow_1")
        .query_async(&mut redis.clone())
        .await
        .expect("sismember");
    assert!(!in_set, "client must be cleared from the shadowban set after unshadowban");

    cleanup(&db, &["adminit_shadow_admin"], &["botit_shadow_1"]).await;
}

/// POST /api/admin/bots/{client_id}/shadowban requires role >= 10.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_bot_shadowban_requires_role_10() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let low = seed_admin_user(&db, "adminit_shadow_low", 5).await;
    let (s, _) = send(&app, "POST", "/api/admin/bots/botit_shadow_1/shadowban", Some(&auth_header(low, "adminit_shadow_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    cleanup(&db, &["adminit_shadow_low"], &[]).await;
}

/// /api/admin/moderation/comments requires role >= 10 and surfaces triage rows
/// for toxic/spam/non-constructive comments (joined with body + title).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_moderation_comments_queue() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_mod_admin", 10).await;
    let low = seed_admin_user(&db, "adminit_mod_low", 5).await;
    let author = seed_admin_user(&db, "adminit_mod_author", 0).await;

    // Seed a work + fic_info (comments.url_id FK → fic_info; work for title).
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, author_url, author_local_id, chapters, words, description, fic_created, fic_updated, status, source, extra_meta, raw_extended_meta, source_id, author_id, content_hash)
           VALUES ('adminit_mod_fic', 'Adminit Mod Fic', 'Adminit Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .execute(&db)
    .await
    .expect("seed fic_info");
    let wid: i32 = match sqlx::query_scalar(
        "INSERT INTO works (canonical_title, canonical_author, default_source_id) VALUES ('Adminit Mod Fic', 'Adminit Author', 'adminit_mod_fic') ON CONFLICT DO NOTHING RETURNING id",
    )
    .fetch_optional(&db)
    .await
    .expect("seed work")
    {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM works WHERE canonical_title = 'Adminit Mod Fic'")
            .fetch_one(&db)
            .await
            .expect("work lookup"),
    };
    let _ = sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = 'adminit_mod_fic'")
        .bind(wid)
        .execute(&db)
        .await;

    // A flagged comment + its triage row.
    let cid: i64 = sqlx::query_scalar(
        "INSERT INTO comments (url_id, user_id, work_id, body, constructive) VALUES ('adminit_mod_fic', $1, $2, 'This is pure spam go away', FALSE) RETURNING id",
    )
    .bind(author)
    .bind(wid)
    .fetch_one(&db)
    .await
    .expect("seed comment");
    sqlx::query(
        "INSERT INTO comment_triage (comment_id, category, reason, confidence) VALUES ($1, 'spam', 'looks like spam', 0.95) ON CONFLICT (comment_id) DO NOTHING",
    )
    .bind(cid)
    .execute(&db)
    .await
    .expect("seed triage");

    // Role 5 → 403.
    let (s, _) = send(&app, "GET", "/api/admin/moderation/comments", Some(&auth_header(low, "adminit_mod_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "role 5 blocked");

    // Role 10 → the flagged comment surfaces with its title.
    let (s, body) = send(&app, "GET", "/api/admin/moderation/comments", Some(&auth_header(admin, "adminit_mod_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "mod comments: {body}");
    let items = body["items"].as_array().unwrap();
    let mine = items.iter().find(|i| i["comment_id"] == json!(cid)).expect("flagged comment present");
    assert_eq!(mine["category"], "spam");
    assert_eq!(mine["body"], "This is pure spam go away");
    assert_eq!(mine["title"], "Adminit Mod Fic");
    assert_eq!(mine["user_id"], json!(author));

    // Cleanup
    let _ = sqlx::query("DELETE FROM comment_triage WHERE comment_id = $1").bind(cid).execute(&db).await;
    let _ = sqlx::query("DELETE FROM comments WHERE id = $1").bind(cid).execute(&db).await;
    let _ = sqlx::query("DELETE FROM works WHERE id = $1").bind(wid).execute(&db).await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = 'adminit_mod_fic'").execute(&db).await;
    cleanup(&db, &["adminit_mod_admin", "adminit_mod_low", "adminit_mod_author"], &[]).await;
}

/// /api/admin/stats returns daily rollups + live totals (role >= 10).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_stats_returns_daily_and_totals() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_stats_admin", 10).await;
    let low = seed_admin_user(&db, "adminit_stats_low", 5).await;

    // Seed one daily rollup row for a deterministic date.
    let _ = sqlx::query(
        "INSERT INTO admin_daily_stats (date, total_users, new_users, total_works, new_works, manual_uploads, epubs_downloaded, words_read) VALUES (CURRENT_DATE, 42, 3, 17, 2, 1, 500, 12345) ON CONFLICT (date) DO UPDATE SET total_users = EXCLUDED.total_users",
    )
    .execute(&db)
    .await;

    // Role 5 → 403.
    let (s, _) = send(&app, "GET", "/api/admin/stats", Some(&auth_header(low, "adminit_stats_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "role 5 blocked");

    // Role 10 → today's rollup row is present with our seeded numbers.
    let (s, body) = send(&app, "GET", "/api/admin/stats", Some(&auth_header(admin, "adminit_stats_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "stats: {body}");
    assert_eq!(body["err"], 0);
    let daily = body["daily"].as_array().unwrap();
    let today = daily.iter().find(|d| d["date"] == chrono::Utc::now().format("%Y-%m-%d").to_string()).expect("today row");
    assert_eq!(today["total_users"], 42, "seeded rollup: {body}");
    assert_eq!(today["new_works"], 2);
    assert!(body["totals"]["users"].as_i64().is_some(), "totals present: {body}");
    assert!(body["totals"]["works"].as_i64().is_some());

    // Cleanup: restore the pre-existing daily row (if any) by deleting ours.
    let _ = sqlx::query("DELETE FROM admin_daily_stats WHERE date = CURRENT_DATE AND total_users = 42").execute(&db).await;
    cleanup(&db, &["adminit_stats_admin", "adminit_stats_low"], &[]).await;
}

/// /api/admin/users search + role change + ban toggle (role >= 10).
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_users_search_role_ban() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_users_admin", 10).await;
    let target = seed_admin_user(&db, "adminit_users_target", 0).await;

    // Search finds the target by username fragment.
    let (s, body) = send(&app, "GET", "/api/admin/users?q=adminit_users_target", Some(&auth_header(admin, "adminit_users_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "users search: {body}");
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 1, "search hit: {body}");
    assert_eq!(users[0]["id"], json!(target));
    assert_eq!(users[0]["is_banned"], false);

    // Promote to role 5.
    let (s, body) = send(&app, "PUT", &format!("/api/admin/users/{target}/role"), Some(&auth_header(admin, "adminit_users_admin", 10)), Some(json!({ "role": 5 }))).await;
    assert_eq!(s, StatusCode::OK, "set role: {body}");
    let role: i16 = sqlx::query_scalar("SELECT role FROM users WHERE id = $1").bind(target).fetch_one(&db).await.expect("role");
    assert_eq!(role, 5, "role persisted");

    // Ban the user.
    let (s, body) = send(&app, "PUT", &format!("/api/admin/users/{target}/ban"), Some(&auth_header(admin, "adminit_users_admin", 10)), Some(json!({ "is_banned": true }))).await;
    assert_eq!(s, StatusCode::OK, "ban: {body}");
    assert_eq!(body["msg"], "User banned");
    let banned: bool = sqlx::query_scalar("SELECT is_banned FROM users WHERE id = $1").bind(target).fetch_one(&db).await.expect("banned");
    assert!(banned, "ban persisted");

    // Search now reports is_banned true.
    let (_, body) = send(&app, "GET", "/api/admin/users?q=adminit_users_target", Some(&auth_header(admin, "adminit_users_admin", 10)), None).await;
    assert_eq!(body["users"][0]["is_banned"], true, "search reflects ban: {body}");

    // Unban.
    let (s, body) = send(&app, "PUT", &format!("/api/admin/users/{target}/ban"), Some(&auth_header(admin, "adminit_users_admin", 10)), Some(json!({ "is_banned": false }))).await;
    assert_eq!(s, StatusCode::OK, "unban: {body}");
    assert_eq!(body["msg"], "User unbanned");

    // Non-admin cannot change roles → 403.
    let low = seed_admin_user(&db, "adminit_users_low", 1).await;
    let (s, _) = send(&app, "PUT", &format!("/api/admin/users/{target}/role"), Some(&auth_header(low, "adminit_users_low", 1)), Some(json!({ "role": 10 }))).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "non-admin role change blocked");

    cleanup(&db, &["adminit_users_admin", "adminit_users_target", "adminit_users_low"], &[]).await;
}

/// Auto-tag review queue: machine-suggested fic_tags show up in the queue,
/// approve promotes them, dismiss deletes them. Role >= 10.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_auto_tag_queue_approve_dismiss() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_autotag_admin", 10).await;
    let low = seed_admin_user(&db, "adminit_autotag_low", 5).await;

    // Seed a fic + a tag + a machine-suggested fic_tag (pending review).
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, author_url, author_local_id, chapters, words, description, fic_created, fic_updated, status, source, extra_meta, raw_extended_meta, source_id, author_id, content_hash)
           VALUES ('adminit_autotag_fic', 'Adminit AutoTag Fic', 'Adminit Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .execute(&db)
    .await
    .expect("seed fic_info");
    let tag_id: i32 = match sqlx::query_scalar(
        "INSERT INTO tags (name, tag_type_id) VALUES ('Adminit AutoTag Tag', 4) ON CONFLICT (name) DO NOTHING RETURNING id",
    )
    .fetch_optional(&db)
    .await
    .expect("seed tag")
    {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM tags WHERE name = 'Adminit AutoTag Tag'")
            .fetch_one(&db)
            .await
            .expect("tag lookup"),
    };
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score, is_machine_suggested) VALUES ('adminit_autotag_fic', $1, '127.0.0.1', 80, TRUE) ON CONFLICT (url_id, tag_id) DO NOTHING",
    )
    .bind(tag_id)
    .execute(&db)
    .await
    .expect("seed machine-suggested fic_tag");

    // Role 5 → 403.
    let (s, _) = send(&app, "GET", "/api/admin/auto-tag/queue", Some(&auth_header(low, "adminit_autotag_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "role 5 blocked");

    // Role 10 → queue contains the pending suggestion.
    let (s, body) = send(&app, "GET", "/api/admin/auto-tag/queue", Some(&auth_header(admin, "adminit_autotag_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "queue: {body}");
    let items = body["items"].as_array().unwrap();
    let mine = items.iter().find(|i| i["url_id"] == "adminit_autotag_fic").expect("suggestion present");
    assert_eq!(mine["tag_id"], json!(tag_id));
    assert_eq!(mine["tag_name"], "Adminit AutoTag Tag");
    assert_eq!(mine["title"], "Adminit AutoTag Fic");

    // Approve → flagged off + reviewed, score bumped; gone from queue.
    let (s, body) = send(&app, "POST", &format!("/api/admin/auto-tag/approve/adminit_autotag_fic/{tag_id}"), Some(&auth_header(admin, "adminit_autotag_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "approve: {body}");
    assert_eq!(body["err"], 0, "approve err: {body}");
    let (_s, body) = send(&app, "GET", "/api/admin/auto-tag/queue", Some(&auth_header(admin, "adminit_autotag_admin", 10)), None).await;
    assert!(body["items"].as_array().unwrap().iter().all(|i| i["url_id"] != "adminit_autotag_fic"), "approved suggestion gone from queue: {body}");

    // Dismiss path: re-flag a suggestion and dismiss it (row deleted).
    sqlx::query(
        "UPDATE fic_tags SET is_machine_suggested = TRUE, reviewed_at = NULL WHERE url_id = 'adminit_autotag_fic' AND tag_id = $1",
    )
    .bind(tag_id)
    .execute(&db)
    .await
    .expect("re-flag");
    let (s, body) = send(&app, "POST", &format!("/api/admin/auto-tag/dismiss/adminit_autotag_fic/{tag_id}"), Some(&auth_header(admin, "adminit_autotag_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "dismiss: {body}");
    assert_eq!(body["err"], 0, "dismiss err: {body}");
    let gone: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fic_tags WHERE url_id = 'adminit_autotag_fic' AND tag_id = $1",
    )
    .bind(tag_id)
    .fetch_one(&db)
    .await
    .expect("row count");
    assert_eq!(gone, 0, "dismissed row deleted");

    // Approving a nonexistent suggestion → 404.
    let (s, body) = send(&app, "POST", "/api/admin/auto-tag/approve/adminit_autotag_fic/999999999", Some(&auth_header(admin, "adminit_autotag_admin", 10)), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "approve missing: {body}");

    // Cleanup
    let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = 'adminit_autotag_fic'").execute(&db).await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = 'adminit_autotag_fic'").execute(&db).await;
    let _ = sqlx::query("DELETE FROM tags WHERE name = 'Adminit AutoTag Tag'").execute(&db).await;
    cleanup(&db, &["adminit_autotag_admin", "adminit_autotag_low"], &[]).await;
}

// ── Blacklist API (fic + author) ────────────────────────────────────────────

/// POST /api/admin/blacklist/fic + /author insert rows; GET
/// /api/admin/blacklist lists them; role < 10 is forbidden everywhere.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_blacklist_add_and_list() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_bl_admin", 10).await;
    let low = seed_admin_user(&db, "adminit_bl_low", 5).await;
    let fic_id = "adminit_bl_fic_1";
    // Remove leftovers from any previous run (self-heal).
    let _ = sqlx::query("DELETE FROM fic_blacklist WHERE url_id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM author_blacklist WHERE source_id = $1 AND author_id = $2").bind(4242i64).bind(4243i64).execute(&db).await;

    // Role 5 → 403 on every endpoint.
    for (method, uri, body) in [
        ("GET", "/api/admin/blacklist", None),
        ("POST", "/api/admin/blacklist/fic", Some(json!({ "url_id": fic_id, "reason": 5 }))),
        ("POST", "/api/admin/blacklist/author", Some(json!({ "source_id": 4242, "author_id": 4243, "reason": 6 }))),
    ] {
        let (s, _) = send(&app, method, uri, Some(&auth_header(low, "adminit_bl_low", 5)), body).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{method} {uri} must require role 10");
    }

    // The fic_blacklist.url_id FK references fic_info(id), so seed a fic row
    // before blacklisting it.
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, author_url, author_local_id, chapters, words, description, fic_created, fic_updated, status, source, extra_meta, raw_extended_meta, source_id, author_id, content_hash)
           VALUES ($1, 'Adminit Bl Fic', 'Adminit Bl Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(fic_id)
    .execute(&db)
    .await
    .expect("seed fic_info for blacklist");

    // Add a fic (reason defaults to 5 when omitted).
    let (s, body) = send(&app, "POST", "/api/admin/blacklist/fic", Some(&auth_header(admin, "adminit_bl_admin", 10)), Some(json!({ "url_id": fic_id }))).await;
    assert_eq!(s, StatusCode::OK, "blacklist fic: {body}");
    assert_eq!(body["err"], 0, "blacklist fic err: {body}");
    assert_eq!(body["reason"], 5, "default reason: {body}");

    // Add an author with an explicit reason.
    let (s, body) = send(&app, "POST", "/api/admin/blacklist/author", Some(&auth_header(admin, "adminit_bl_admin", 10)), Some(json!({ "source_id": 4242, "author_id": 4243, "reason": 6 }))).await;
    assert_eq!(s, StatusCode::OK, "blacklist author: {body}");
    assert_eq!(body["err"], 0, "blacklist author err: {body}");
    assert_eq!(body["reason"], 6, "author reason: {body}");

    // Rows are in the DB.
    let fic_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fic_blacklist WHERE url_id = $1").bind(fic_id).fetch_one(&db).await.expect("fic rows");
    assert_eq!(fic_rows, 1, "fic blacklist row");
    let author_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM author_blacklist WHERE source_id = $1 AND author_id = $2").bind(4242i64).bind(4243i64).fetch_one(&db).await.expect("author rows");
    assert_eq!(author_rows, 1, "author blacklist row");

    // GET lists both.
    let (s, body) = send(&app, "GET", "/api/admin/blacklist", Some(&auth_header(admin, "adminit_bl_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "list: {body}");
    assert_eq!(body["err"], 0, "list err: {body}");
    let fics = body["fics"].as_array().unwrap();
    let mine = fics.iter().find(|f| f["url_id"] == fic_id).expect("fic in list");
    assert_eq!(mine["reason"], 5, "fic reason: {body}");
    let authors = body["authors"].as_array().unwrap();
    let mine_a = authors.iter().find(|a| a["source_id"] == 4242 && a["author_id"] == 4243).expect("author in list");
    assert_eq!(mine_a["reason"], 6, "author reason: {body}");

    // Empty url_id → HTTP 400.
    let (s, body) = send(&app, "POST", "/api/admin/blacklist/fic", Some(&auth_header(admin, "adminit_bl_admin", 10)), Some(json!({ "url_id": "  " }))).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "empty url_id: {body}");

    // Cleanup
    let _ = sqlx::query("DELETE FROM fic_blacklist WHERE url_id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM author_blacklist WHERE source_id = $1 AND author_id = $2").bind(4242i64).bind(4243i64).execute(&db).await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1").bind(fic_id).execute(&db).await;
    cleanup(&db, &["adminit_bl_admin", "adminit_bl_low"], &[]).await;
}

// ── Comment moderation actions (hide / delete) ─────────────────────────────

/// POST /api/admin/moderation/comments/{id}/hide + /{id}/delete act on a
/// specific comment and clear its triage row; role < 10 is forbidden.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn admin_moderation_comment_actions() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_admin_user(&db, "adminit_mod_admin", 10).await;
    let low = seed_admin_user(&db, "adminit_mod_low", 5).await;
    let author = seed_admin_user(&db, "adminit_mod_author", 0).await;
    let fic_id = "adminit_mod_fic_1";
    // Self-heal leftovers.
    let _ = sqlx::query("DELETE FROM comment_triage WHERE comment_id IN (SELECT id FROM comments WHERE url_id = $1)").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM comments WHERE url_id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title = 'Adminit Mod Fic'").execute(&db).await;

    // Seed a work + fic_info + a comment (with a triage row).
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, author_url, author_local_id, chapters, words, description, fic_created, fic_updated, status, source, extra_meta, raw_extended_meta, source_id, author_id, content_hash)
           VALUES ($1, 'Adminit Mod Fic', 'Adminit Mod Author', NULL, NULL, 1, 1000, 'desc', NOW(), NOW(), 'complete', 'ao3', NULL, NULL, NULL, NULL, NULL)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(fic_id)
    .execute(&db)
    .await
    .expect("seed fic_info");
    let work_id: i32 = match sqlx::query_scalar(
        "INSERT INTO works (canonical_title, canonical_author, default_source_id) VALUES ('Adminit Mod Fic', 'Adminit Mod Author', $1) ON CONFLICT DO NOTHING RETURNING id",
    )
    .bind(fic_id)
    .fetch_optional(&db)
    .await
    .expect("seed work")
    {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM works WHERE canonical_title = 'Adminit Mod Fic'")
            .fetch_one(&db)
            .await
            .expect("work lookup"),
    };
    let c1: i64 = sqlx::query_scalar(
        "INSERT INTO comments (url_id, user_id, work_id, body, constructive) VALUES ($1, $2, $3, 'You are horrible, go away', TRUE) RETURNING id",
    )
    .bind(fic_id)
    .bind(author)
    .bind(work_id)
    .fetch_one(&db)
    .await
    .expect("seed comment 1");
    let c2: i64 = sqlx::query_scalar(
        "INSERT INTO comments (url_id, user_id, work_id, body, constructive) VALUES ($1, $2, $3, 'Check my casino!!', TRUE) RETURNING id",
    )
    .bind(fic_id)
    .bind(author)
    .bind(work_id)
    .fetch_one(&db)
    .await
    .expect("seed comment 2");
    sqlx::query("INSERT INTO comment_triage (comment_id, category, reason, confidence) VALUES ($1, 'toxic', 'abuse', 0.9), ($2, 'spam', 'links', 0.8)")
        .bind(c1)
        .bind(c2)
        .execute(&db)
        .await
        .expect("seed triage");

    // Role 5 → 403 on both action endpoints.
    let (s, _) = send(&app, "POST", &format!("/api/admin/moderation/comments/{c1}/hide"), Some(&auth_header(low, "adminit_mod_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "role 5 hide blocked");
    let (s, _) = send(&app, "POST", &format!("/api/admin/moderation/comments/{c2}/delete"), Some(&auth_header(low, "adminit_mod_low", 5)), None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "role 5 delete blocked");

    // Admin hides c1 → is_hidden flips, triage row cleared.
    let (s, body) = send(&app, "POST", &format!("/api/admin/moderation/comments/{c1}/hide"), Some(&auth_header(admin, "adminit_mod_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "hide: {body}");
    assert_eq!(body["err"], 0, "hide err: {body}");
    let hidden: bool = sqlx::query_scalar("SELECT is_hidden FROM comments WHERE id = $1").bind(c1).fetch_one(&db).await.expect("is_hidden");
    assert!(hidden, "comment hidden");
    let triage_left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment_triage WHERE comment_id = $1").bind(c1).fetch_one(&db).await.expect("triage count");
    assert_eq!(triage_left, 0, "triage row cleared on hide");

    // Admin deletes c2 → deleted_at stamped, triage row cleared.
    let (s, body) = send(&app, "POST", &format!("/api/admin/moderation/comments/{c2}/delete"), Some(&auth_header(admin, "adminit_mod_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK, "delete: {body}");
    assert_eq!(body["err"], 0, "delete err: {body}");
    let deleted: bool = sqlx::query_scalar("SELECT deleted_at IS NOT NULL FROM comments WHERE id = $1").bind(c2).fetch_one(&db).await.expect("deleted_at");
    assert!(deleted, "comment soft-deleted");
    let triage_left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment_triage WHERE comment_id = $1").bind(c2).fetch_one(&db).await.expect("triage count");
    assert_eq!(triage_left, 0, "triage row cleared on delete");

    // Actions on a missing comment → 404.
    let (s, body) = send(&app, "POST", "/api/admin/moderation/comments/999999999/hide", Some(&auth_header(admin, "adminit_mod_admin", 10)), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "hide missing: {body}");
    let (s, body) = send(&app, "POST", "/api/admin/moderation/comments/999999999/delete", Some(&auth_header(admin, "adminit_mod_admin", 10)), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "delete missing: {body}");

    // Cleanup
    let _ = sqlx::query("DELETE FROM comment_triage WHERE comment_id IN (SELECT id FROM comments WHERE url_id = $1)").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM comments WHERE url_id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1").bind(fic_id).execute(&db).await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title = 'Adminit Mod Fic'").execute(&db).await;
    cleanup(&db, &["adminit_mod_admin", "adminit_mod_low", "adminit_mod_author"], &[]).await;
}

/// /api/admin/search-analytics returns the search→export conversion view
/// (searchers / exporters / converted, joined by anonymous client_id).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn admin_search_analytics_conversion() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    // Seed: one client searches, one client searches AND exports.
    sqlx::query("INSERT INTO search_queries (client_id, query, total_results, ts) VALUES ('convt_a', 'harry', 5, now())")
        .execute(&db).await.expect("seed search a");
    sqlx::query("INSERT INTO search_queries (client_id, query, total_results, ts) VALUES ('convt_b', 'dumbledore', 3, now())")
        .execute(&db).await.expect("seed search b");
    // convt_b exports too.
    sqlx::query(
        "INSERT INTO request_log (source_id, etype, query, info_request_ms, url_id, export_file_name, created, client_id)
         SELECT COALESCE((SELECT id FROM request_source LIMIT 1), 1), 'epub', 'dumbledore', 10, 'convt_fic', 'x.epub', now(), 'convt_b'
         WHERE EXISTS (SELECT 1 FROM request_source)"
    )
    .execute(&db).await.expect("seed export (request_source may be empty; then skipped)");

    let admin = seed_admin_user(&db, "adminit_conversion_admin", 10).await;
    let (s, body) = send(&app, "GET", "/api/admin/search-analytics", Some(&auth_header(admin, "adminit_conversion_admin", 10)), None).await;
    assert_eq!(s, StatusCode::OK);

    let conv = &body["conversion"];
    assert!(conv["searchers_7d"].as_i64().unwrap_or(-1) >= 2, "at least the two seeded searchers: {body}");
    // converted counts clients that both searched and exported — at most 1 seeded.
    let converted = conv["search_to_export_7d"].as_i64().unwrap_or(-1);
    assert!(converted <= 1, "conversion can't exceed seeded exporter: {body}");

    // Zero-PII: no client ids in the payload.
    let raw = body.to_string();
    assert!(!raw.contains("convt_"), "no client ids leaked: {raw}");

    let _ = sqlx::query("DELETE FROM search_queries WHERE client_id LIKE 'convt_%'").execute(&db).await;
    let _ = sqlx::query("DELETE FROM request_log WHERE client_id LIKE 'convt_%'").execute(&db).await;
    cleanup(&db, &["adminit_conversion_admin"], &[]).await;
}

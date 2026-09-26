//! DB-gated integration tests for the reader "Next Up" endpoints:
//! GET /api/reader/{url_id}/sequel + GET /api/reader/{url_id}/related.
//!
//! Conventions (same as tests/requests_api.rs): serialised via a global
//! Mutex, marked `#[ignore]`, run with:
//!   set -a; . ./.env; set +a; cargo test --test reader_api -- --include-ignored --test-threads=1
//! Self-heal: every test deletes its own seed rows at the START.

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (load .env, e.g. set -a; . ./.env; set +a)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to test database (is Postgres up?)")
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
        .expect("failed to connect to Redis");

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
            "/api/reader/{url_id}/sequel",
            axum::routing::get(fichub::routes::reader::reader_sequel_handler),
        )
        .route(
            "/api/reader/{url_id}/related",
            axum::routing::get(fichub::routes::reader::reader_related_handler),
        )
        .route(
            "/api/reader/{url_id}/sequential",
            axum::routing::get(fichub::routes::reader::reader_sequential_handler),
        )
        .route(
            "/api/reader/{url_id}/next-up",
            axum::routing::get(fichub::routes::reader::reader_next_up_handler),
        )
        .with_state(state)
}

async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
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

/// Seed a fic_info row + a linked work. Returns work_id.
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
        Some(id) => id,
        None => sqlx::query_scalar(
            "INSERT INTO works (canonical_title, canonical_author, default_source_id)
                 VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(title)
        .bind(author)
        .bind(url_id)
        .fetch_one(pool)
        .await
        .expect("seed work failed"),
    };

    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await
        .expect("link fic_info to work failed");

    work_id
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

#[tokio::test]
#[ignore]
async fn sequel_via_series_and_fallback() {
    let _g = db_guard();
    let db = pool().await;

    // Seed a 2-part series: The Awakening + The Awakening II.
    let w1 = seed_work(&db, "seq-p1", "The Awakening", "Seq Author").await;
    let w2 = seed_work(&db, "seq-p2", "The Awakening II", "Seq Author").await;
    sqlx::query("DELETE FROM series_works WHERE work_id = $1 OR work_id = $2")
        .bind(w1)
        .bind(w2)
        .execute(&db)
        .await
        .ok();
    let sid: i32 =
        sqlx::query_scalar("INSERT INTO series (name) VALUES ('Seq Series') RETURNING id")
            .fetch_one(&db)
            .await
            .expect("series insert failed");
    sqlx::query("INSERT INTO series_works (series_id, work_id, position) VALUES ($1, $2, 1)")
        .bind(sid)
        .bind(w1)
        .execute(&db)
        .await
        .ok();
    sqlx::query("INSERT INTO series_works (series_id, work_id, position) VALUES ($1, $2, 2)")
        .bind(sid)
        .bind(w2)
        .execute(&db)
        .await
        .ok();

    let app = app().await;

    // Sequel of part 1 = part 2 (via series).
    let (s, b) = get_json(&app, "/api/reader/seq-p1/sequel").await;
    assert_eq!(s, StatusCode::OK, "sequel: {b}");
    assert_eq!(b["err"], 0);
    assert_eq!(b["sequel"]["url_id"], "seq-p2");
    assert_eq!(b["sequel"]["title"], "The Awakening II");

    // Sequel of the LAST part → no sequel (err -1).
    let (_, b) = get_json(&app, "/api/reader/seq-p2/sequel").await;
    assert_eq!(b["err"], -1, "no sequel for last part: {b}");

    // Unknown fic → 404.
    let (s, _) = get_json(&app, "/api/reader/no-such/sequel").await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Cleanup
    sqlx::query("DELETE FROM series_works WHERE series_id = $1")
        .bind(sid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM series WHERE id = $1")
        .bind(sid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM works WHERE id = $1 OR id = $2")
        .bind(w1)
        .bind(w2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_info WHERE id = 'seq-p1' OR id = 'seq-p2'")
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn related_via_shared_bookmarks() {
    let _g = db_guard();
    let db = pool().await;

    let u1 = "seqrel_u1";
    let u2 = "seqrel_u2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;

    seed_work(&db, "rel-main", "Main Fic", "Rel Author").await;
    seed_work(&db, "rel-a", "Also A", "Rel Author").await;
    seed_work(&db, "rel-b", "Also B", "Rel Author").await;

    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();

    // Both users bookmarked rel-main AND rel-a; only u1 bookmarked rel-b.
    for (uid, other) in [(id1, "rel-a"), (id2, "rel-a"), (id1, "rel-b")] {
        sqlx::query(
            "INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING",
        )
        .bind(uid)
        .bind(other)
        .execute(&db)
        .await
        .ok();
    }
    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2), ($3, $4) ON CONFLICT (user_id, url_id) DO NOTHING",
    )
    .bind(id1)
    .bind("rel-main")
    .bind(id2)
    .bind("rel-main")
    .execute(&db)
    .await
    .ok();

    let app = app().await;
    let (s, b) = get_json(&app, "/api/reader/rel-main/related").await;
    assert_eq!(s, StatusCode::OK, "related: {b}");
    assert_eq!(b["err"], 0);
    let related = b["related"].as_array().unwrap();
    assert!(
        related.iter().any(|r| r["url_id"] == "rel-a"),
        "rel-a present: {b}"
    );
    // rel-a has 2 shared bookmarkers → higher rank than rel-b (1).
    let ra = related.iter().find(|r| r["url_id"] == "rel-a").unwrap();
    let rb = related.iter().find(|r| r["url_id"] == "rel-b").unwrap();
    assert!(
        ra["shared"].as_i64() >= rb["shared"].as_i64(),
        "rank order: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM works WHERE canonical_author = 'Rel Author'")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_info WHERE id IN ('rel-main','rel-a','rel-b')")
        .execute(&db)
        .await
        .ok();
}

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
        is_admin: false,
    };
    let token = fichub::routes::auth::create_token(&user, &secret).expect("token creation");
    format!("Bearer {token}")
}

async fn get_json_auth(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, t);
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

#[tokio::test]
#[ignore]
async fn sequential_anonymous_empty_and_logged_in_returns_transitions() {
    let _g = db_guard();
    let db = pool().await;

    // Seed two fics; insert a transition seed -> target directly.
    let w_main = seed_work(&db, "seqtrans-main", "Seq Trans Main", "SeqTrans Author").await;
    let w_tgt = seed_work(&db, "seqtrans-tgt", "Seq Trans Target", "SeqTrans Author").await;

    sqlx::query("DELETE FROM rec_transitions WHERE from_work = $1 OR to_work = $1")
        .bind("seqtrans-main")
        .execute(&db)
        .await
        .ok();
    sqlx::query(
        r#"INSERT INTO rec_transitions (from_work, to_work, weight, last_seen)
           VALUES ($1, $2, 5.0, NOW())
           ON CONFLICT (from_work, to_work) DO UPDATE SET weight = EXCLUDED.weight, last_seen = NOW()"#,
    )
    .bind("seqtrans-main")
    .bind("seqtrans-tgt")
    .execute(&db)
    .await
    .expect("seed transition failed");

    let app = app().await;

    // Anonymous caller: sequential is gated on logged-in → empty list.
    let (_, a) = get_json(&app, "/api/reader/seqtrans-main/sequential").await;
    assert_eq!(a["err"], 0, "anonymous sequential: {a}");
    assert_eq!(
        a["sequential"].as_array().unwrap().len(),
        0,
        "anonymous sequential empty: {a}"
    );

    // Logged-in caller: transition surfaces, ranked by decayed weight desc.
    let user = seed_user(&db, "seqtrans_user").await;
    let (s, b) = get_json_auth(
        &app,
        "/api/reader/seqtrans-main/sequential",
        Some(&auth_header(user, "seqtrans_user", 0)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "logged-in sequential: {b}");
    assert_eq!(b["err"], 0, "err: {b}");
    let seq = b["sequential"].as_array().unwrap();
    assert!(
        seq.iter().any(|r| r["url_id"] == "seqtrans-tgt"),
        "target present: {b}"
    );
    // Scores are positive and sorted desc (single pick here).
    let scores: Vec<f64> = seq.iter().filter_map(|r| r["score"].as_f64()).collect();
    assert!(scores.windows(2).all(|w| w[0] >= w[1]), "scores desc: {b}");

    // Cleanup
    sqlx::query("DELETE FROM rec_transitions WHERE from_work = $1 OR to_work = $1")
        .bind("seqtrans-main")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind("seqtrans_user")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM works WHERE id = $1 OR id = $2")
        .bind(w_main)
        .bind(w_tgt)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_info WHERE id IN ('seqtrans-main','seqtrans-tgt')")
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn next_up_aggregates_sequel_sequential_related_dedup() {
    let _g = db_guard();
    let db = pool().await;

    // Seed three fics: main, the sequel (next-in-series), and a sequential
    // transition target that is ALSO a related (also-bookmarked) pick — this
    // exercises the dedupe path (sequel > sequential > related, ranked by
    // score desc within sequential).
    let w_main = seed_work(&db, "nup-main", "NextUp Main", "NextUp Author").await;
    let w_seq = seed_work(&db, "nup-seq", "NextUp Sequel", "NextUp Author").await;
    let w_trans = seed_work(&db, "nup-trans", "NextUp Transition", "NextUp Author").await;

    // Series link: main -> seq (next in series).
    sqlx::query("DELETE FROM series_works WHERE work_id = $1 OR work_id = $2")
        .bind(w_main)
        .bind(w_seq)
        .execute(&db)
        .await
        .ok();
    let sid: i32 =
        sqlx::query_scalar("INSERT INTO series (name) VALUES ('NextUp Series') RETURNING id")
            .fetch_one(&db)
            .await
            .expect("series insert");
    sqlx::query("INSERT INTO series_works (series_id, work_id, position) VALUES ($1, $2, 1)")
        .bind(sid)
        .bind(w_main)
        .execute(&db)
        .await
        .ok();
    sqlx::query("INSERT INTO series_works (series_id, work_id, position) VALUES ($1, $2, 2)")
        .bind(sid)
        .bind(w_seq)
        .execute(&db)
        .await
        .ok();

    // Markov transition main -> nup-trans (and the SAME target as a related
    // bookmark pick, to assert dedupe).
    sqlx::query("DELETE FROM rec_transitions WHERE from_work = $1 OR to_work = $1")
        .bind("nup-main")
        .execute(&db)
        .await
        .ok();
    sqlx::query(
        r#"INSERT INTO rec_transitions (from_work, to_work, weight, last_seen)
           VALUES ($1, $2, 7.0, NOW())
           ON CONFLICT (from_work, to_work) DO UPDATE SET weight = EXCLUDED.weight, last_seen = NOW()"#,
    )
    .bind("nup-main")
    .bind("nup-trans")
    .execute(&db)
    .await
    .expect("seed transition");

    // Related: a different user bookmarks nup-main + nup-trans so it's a
    // co-bookmarked pick too (same url_id → must be deduped against sequential).
    let u1 = "nup_user1";
    let u2 = "nup_user2";
    let id1 = seed_user(&db, u1).await;
    let id2 = seed_user(&db, u2).await;
    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2), ($3, $4) ON CONFLICT (user_id, url_id) DO NOTHING")
        .bind(id1).bind("nup-main")
        .bind(id2).bind("nup-main")
        .execute(&db)
        .await
        .ok();
    sqlx::query("INSERT INTO bookmarks (user_id, url_id) VALUES ($1, $2) ON CONFLICT (user_id, url_id) DO NOTHING")
        .bind(id1).bind("nup-trans")
        .execute(&db)
        .await
        .ok();

    let app = app().await;

    // Anonymous Next Up: sequel surfaces (series-based), sequential is empty,
    // related surfaces (co-occurrence is anonymous-friendly).
    let (s, b) = get_json(&app, "/api/reader/nup-main/next-up").await;
    assert_eq!(s, StatusCode::OK, "next-up anon: {b}");
    assert_eq!(b["err"], 0, "err: {b}");
    assert_eq!(b["sequel"]["url_id"], "nup-seq", "anon sequel: {b}");
    assert!(
        b["sequential"].as_array().unwrap().is_empty(),
        "anon sequential empty: {b}"
    );
    let related = b["related"].as_array().unwrap();
    assert!(
        related.iter().any(|r| r["url_id"] == "nup-trans"),
        "anon related: {b}"
    );

    // Authenticated Next Up: sequel > sequential > related and fully deduped.
    let (s, b) = get_json_auth(
        &app,
        "/api/reader/nup-main/next-up",
        Some(&auth_header(id1, u1, 0)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "next-up auth: {b}");
    assert_eq!(b["err"], 0, "err: {b}");
    assert_eq!(b["sequel"]["url_id"], "nup-seq", "auth sequel: {b}");

    let next_up = b["next_up"].as_array().unwrap();
    // First item is the sequel.
    assert_eq!(next_up[0]["url_id"], "nup-seq", "first is sequel: {b}");
    assert_eq!(next_up[0]["source"], "sequel", "source label: {b}");

    // Sequential appears (logged in) and is the decayed-weight ranked target.
    let seq = b["sequential"].as_array().unwrap();
    assert!(
        seq.iter().any(|r| r["url_id"] == "nup-trans"),
        "auth sequential: {b}"
    );

    // Dedup: nup-trans must appear in next_up only ONCE (as sequential, not
    // also as related).
    let trans_entries: Vec<&Value> = next_up
        .iter()
        .filter(|i| i["url_id"] == "nup-trans")
        .collect();
    assert_eq!(trans_entries.len(), 1, "deduped to once: {b}");
    assert_eq!(
        trans_entries[0]["source"], "sequential",
        "trans source: {b}"
    );
    assert!(
        trans_entries[0]["score"].as_f64().unwrap() > 0.0,
        "score present: {b}"
    );

    // Cleanup
    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 OR user_id = $2")
        .bind(id1)
        .bind(id2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM users WHERE username = $1 OR username = $2")
        .bind(u1)
        .bind(u2)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM rec_transitions WHERE from_work = $1 OR to_work = $1")
        .bind("nup-main")
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM series_works WHERE series_id = $1")
        .bind(sid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM series WHERE id = $1")
        .bind(sid)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM works WHERE id = $1 OR id = $2 OR id = $3")
        .bind(w_main)
        .bind(w_seq)
        .bind(w_trans)
        .execute(&db)
        .await
        .ok();
    sqlx::query("DELETE FROM fic_info WHERE id IN ('nup-main','nup-seq','nup-trans')")
        .execute(&db)
        .await
        .ok();
}

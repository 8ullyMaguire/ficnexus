//! DB-gated integration tests for the translation review workflow
//! (draft → post-edit → approve/reject), the rating/warning verification
//! queue, and the character/relationship score-fixing endpoint.
//!
//! Conventions (same as tests/admin_api.rs):
//! * Serialised via a global `Mutex` so tests never run concurrently.
//! * Marked `#[ignore]`; run with:
//!   `set -a; . /personal/documents/code/rust/fichub/.env; set +a; \
//!    CARGO_INCREMENTAL=0 cargo test --test translation_review_api -- \
//!        --include-ignored --test-threads=1`
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.
//! * All endpoints gate on JWT role >= 10 (admin) — anonymous callers get
//!   HTTP 400 {err:401}, sub-admin roles HTTP 403 (Forbidden).

use std::sync::{Mutex, OnceLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt; // oneshot

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    DB_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

const PREFIX: &str = "trv";

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
            fichub::limiter::redis_bucket::RedisBucketLimiter::with_config(
                redis_client
                    .get_multiplexed_async_connection()
                    .await
                    .expect("redis"),
                false,
                Some(&config),
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
            "/api/admin/translations",
            get(fichub::routes::admin::admin_list_translations),
        )
        .route(
            "/api/admin/translations/{id}/approve",
            post(fichub::routes::admin::admin_approve_translation),
        )
        .route(
            "/api/admin/translations/{id}/reject",
            post(fichub::routes::admin::admin_reject_translation),
        )
        .route(
            "/api/admin/translations/{id}/edit",
            post(fichub::routes::admin::admin_edit_translation),
        )
        .route(
            "/api/admin/rating-checks",
            get(fichub::routes::admin::admin_rating_checks),
        )
        .route(
            "/api/admin/rating-checks/{work_id}/verify",
            post(fichub::routes::admin::admin_verify_rating),
        )
        .route(
            "/api/admin/characters/{id}/score",
            put(fichub::routes::admin::admin_fix_tag_score),
        )
        .with_state(state)
}

/// `role` and `trust_level` are separate columns and both are live. The
/// trust gates read `trust_level` (50 of them, e.g.
/// work_proposals.rs:279, comments.rs:474, admin.rs:48), while
/// db/queries/proposals.rs:93 and routes/subsystems.rs:475 read `role`.
/// Seeding only `role` left `trust_level` at its column default of 0, so
/// every curator-gated endpoint returned 403 and the suite was exercising a
/// column no gate reads. Both are written here, from the same value, so the
/// fixture cannot drift back to testing nothing.
async fn seed_user(db: &sqlx::PgPool, username: &str, role: i16) -> i32 {
    sqlx::query("INSERT INTO users (username, password_hash, role, trust_level) VALUES ($1, 'x', $2, LEAST($2, 6)) ON CONFLICT (username) DO UPDATE SET role = EXCLUDED.role, trust_level = LEAST(EXCLUDED.trust_level, 6) RETURNING id")
        .bind(username)
        .bind(role)
        .fetch_one(db)
        .await
        .expect("seed user")
        .get(0)
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

/// Seed a work + fic_info pair; returns (work_id, url_id). Idempotent.
async fn seed_work(
    db: &sqlx::PgPool,
    url_id: &str,
    title: &str,
    extra_meta: Option<&str>,
) -> (i32, String) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, 'Trv Author', NULL, NULL, 4, 8000, 'desc',
                  NOW() - interval '30 days', NOW() - interval '1 day', 'ongoing',
                  'https://example.test/works/trv', $3, NULL, 4242, 4242, NULL)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            extra_meta = EXCLUDED.extra_meta"#,
    )
    .bind(url_id)
    .bind(title)
    .bind(extra_meta)
    .execute(db)
    .await
    .expect("seed fic_info failed");

    let existing: Option<i32> =
        sqlx::query_scalar("SELECT id FROM works WHERE canonical_title = $1")
            .bind(title)
            .fetch_optional(db)
            .await
            .expect("work lookup failed");

    let work_id = match existing {
        Some(id) => id,
        None => sqlx::query_scalar(
            "INSERT INTO works (canonical_title, canonical_author, default_source_id) VALUES ($1, 'Trv Author', $2) RETURNING id",
        )
        .bind(title)
        .bind(url_id)
        .fetch_one(db)
        .await
        .expect("seed work failed"),
    };

    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(db)
        .await
        .expect("link fic_info to work failed");

    (work_id, url_id.to_string())
}

/// Insert a draft work translation directly (bypasses the public submit
/// endpoint which does reputation work); returns its id.
async fn seed_translation(
    db: &sqlx::PgPool,
    work_id: i32,
    locale: &str,
    title: &str,
    summary: &str,
    translator: i32,
) -> i64 {
    sqlx::query_scalar(
        r#"INSERT INTO work_translations (work_id, locale_code, title, summary, translated_by)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (work_id, locale_code) DO UPDATE SET
               title = EXCLUDED.title, summary = EXCLUDED.summary, translated_by = EXCLUDED.translated_by,
               status = 'draft', reviewed_by = NULL, reviewed_at = NULL
           RETURNING id"#,
    )
    .bind(work_id)
    .bind(locale)
    .bind(title)
    .bind(summary)
    .bind(translator)
    .fetch_one(db)
    .await
    .expect("seed translation failed")
}

/// Seed a character tag + its fic_tags row; returns the tag id.
async fn seed_char_tag(db: &sqlx::PgPool, url_id: &str, tag_name: &str, score: i16) -> i32 {
    let tag_id: i32 = sqlx::query_scalar(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, 2) ON CONFLICT (name) DO UPDATE SET tag_type_id = EXCLUDED.tag_type_id RETURNING id",
    )
    .bind(tag_name)
    .fetch_one(db)
    .await
    .expect("seed tag failed");

    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, score) VALUES ($1, $2, $3) ON CONFLICT (url_id, tag_id) DO UPDATE SET score = EXCLUDED.score",
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(score)
    .execute(db)
    .await
    .expect("seed fic_tags failed");

    tag_id
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
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
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes)
        .unwrap_or(json!({ "raw": String::from_utf8_lossy(&bytes).to_string() }));
    (status, v)
}

async fn cleanup(db: &sqlx::PgPool, users: &[&str]) {
    for u in users {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(u)
            .execute(db)
            .await;
    }
    // Self-heal on the unique seed prefixes.
    let _ = sqlx::query("DELETE FROM work_translations WHERE locale_code = 'es' AND work_id IN (SELECT id FROM works WHERE canonical_title LIKE 'Trv %')")
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'trv-%'")
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM works WHERE canonical_title LIKE 'Trv %'")
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM tags WHERE name LIKE 'Trv Char %'")
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM work_rating_verifications WHERE work_id IN (SELECT id FROM works WHERE canonical_title LIKE 'Trv %')")
        .execute(db)
        .await;
    let _ = sqlx::query("DELETE FROM tag_score_fixes WHERE url_id LIKE 'trv-%'")
        .execute(db)
        .await;
}

// ── Translation review workflow ────────────────────────────────────────

/// The whole flow: submit a draft → list drafts → post-edit → approve →
/// the row leaves the draft queue and carries the edited text.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn translation_review_full_flow() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_t_admin"), 10).await;
    let translator = seed_user(&db, &format!("{PREFIX}_t_translator"), 0).await;
    let (work_id, _) = seed_work(
        &db,
        "trv-flow",
        "Trv Flow Fic",
        Some(r#"{"rating":"Explicit"}"#),
    )
    .await;
    let tid = seed_translation(&db, work_id, "es", "Máquina", "Resumen máquina", translator).await;

    let tok = auth_header(admin, &format!("{PREFIX}_t_admin"), 10);

    // Draft appears in the queue.
    let (s, body) = send(
        &app,
        "GET",
        "/api/admin/translations?status=draft",
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["err"], 0);
    let items = body["items"].as_array().unwrap();
    let mine = items
        .iter()
        .find(|i| i["id"] == tid as i64)
        .expect("draft in queue");
    assert_eq!(mine["status"], "draft");
    assert_eq!(mine["title"], "Máquina");

    // Post-edit the title.
    let (s, _) = send(
        &app,
        "POST",
        &format!("/api/admin/translations/{tid}/edit"),
        Some(&tok),
        Some(json!({ "title": "La Máquina Editada" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // Approve it.
    let (s, _) = send(
        &app,
        "POST",
        &format!("/api/admin/translations/{tid}/approve"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // DB state: approved + edited title + reviewer stamp.
    let row: (String, Option<String>, Option<i32>) =
        sqlx::query_as("SELECT status, title, reviewed_by FROM work_translations WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("row exists");
    assert_eq!(row.0, "approved");
    assert_eq!(row.1.as_deref(), Some("La Máquina Editada"));
    assert_eq!(row.2, Some(admin));

    // No longer in the draft queue.
    let (s, body) = send(
        &app,
        "GET",
        "/api/admin/translations?status=draft",
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        body["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["id"] != tid as i64)
    );

    cleanup(
        &db,
        &[
            &format!("{PREFIX}_t_admin"),
            &format!("{PREFIX}_t_translator"),
        ],
    )
    .await;
}

/// Rejecting a draft removes it from the queue and stamps reviewer.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn translation_review_reject() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_r_admin"), 10).await;
    let translator = seed_user(&db, &format!("{PREFIX}_r_translator"), 0).await;
    let (work_id, _) = seed_work(
        &db,
        "trv-reject",
        "Trv Reject Fic",
        Some(r#"{"rating":"Mature"}"#),
    )
    .await;
    let tid = seed_translation(&db, work_id, "fr", "Machine", "Machine summary", translator).await;

    let tok = auth_header(admin, &format!("{PREFIX}_r_admin"), 10);

    let (s, _) = send(
        &app,
        "POST",
        &format!("/api/admin/translations/{tid}/reject"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    let row: (String, Option<i32>) =
        sqlx::query_as("SELECT status, reviewed_by FROM work_translations WHERE id = $1")
            .bind(tid)
            .fetch_one(&db)
            .await
            .expect("row exists");
    assert_eq!(row.0, "rejected");
    assert_eq!(row.1, Some(admin));

    // Re-approving a rejected row fails (only drafts can transition).
    let (s, body) = send(
        &app,
        "POST",
        &format!("/api/admin/translations/{tid}/approve"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert!(body.to_string().contains("draft"));

    cleanup(
        &db,
        &[
            &format!("{PREFIX}_r_admin"),
            &format!("{PREFIX}_r_translator"),
        ],
    )
    .await;
}

/// Role gating: anonymous → 400 {err:401}, sub-admin → HTTP 403.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn translation_review_role_gate() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let low = seed_user(&db, &format!("{PREFIX}_g_low"), 5).await;

    let (s, body) = send(&app, "GET", "/api/admin/translations", None, None).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    assert_eq!(body["err"].as_i64(), Some(-403));

    let (s, _) = send(
        &app,
        "GET",
        "/api/admin/translations",
        Some(&auth_header(low, &format!("{PREFIX}_g_low"), 5)),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    cleanup(&db, &[&format!("{PREFIX}_g_low")]).await;
}

// ── Rating / warning verification queue ────────────────────────────────

/// Works without rating metadata land in the queue; verifying with rating +
/// warnings removes them.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn rating_check_verify_flow() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_rc_admin"), 10).await;
    // No rating anywhere (extra_meta is a plain genre list) → must land in the queue.
    let (work_id, _) = seed_work(
        &db,
        "trv-rc-norating",
        "Trv Rc Fic",
        Some("fantasy, adventure"),
    )
    .await;
    // Verified + rated → must NOT land in the queue.
    let (work_id2, _) = seed_work(&db, "trv-rc-rated", "Trv Rc Rated Fic", Some("fantasy")).await;
    sqlx::query(
        r#"INSERT INTO work_rating_verifications (work_id, rating, warnings, verified_by)
           VALUES ($1, 'Explicit', '["Graphic Depictions Of Violence"]'::jsonb, $2)
           ON CONFLICT (work_id) DO UPDATE SET rating = EXCLUDED.rating"#,
    )
    .bind(work_id2)
    .bind(admin)
    .execute(&db)
    .await
    .expect("seed verification for rated work");
    sqlx::query("UPDATE works SET rating_verified_at = NOW() WHERE id = $1")
        .bind(work_id2)
        .execute(&db)
        .await
        .expect("stamp verified work");

    let tok = auth_header(admin, &format!("{PREFIX}_rc_admin"), 10);

    let (s, body) = send(&app, "GET", "/api/admin/rating-checks", Some(&tok), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["err"], 0);
    let items = body["items"].as_array().unwrap();
    let my_ids: Vec<i64> = items
        .iter()
        .filter(|i| i["title"].as_str().map_or(false, |t| t.starts_with("Trv ")))
        .map(|i| i["work_id"].as_i64().unwrap())
        .collect();
    assert!(
        my_ids.contains(&(work_id as i64)),
        "unrated work in queue: {body}"
    );
    assert!(
        !my_ids.contains(&(work_id2 as i64)),
        "rated work not in queue: {body}"
    );

    // Verify work 1: rating + warnings.
    let (s, _) = send(
        &app,
        "POST",
        &format!("/api/admin/rating-checks/{work_id}/verify"),
        Some(&tok),
        Some(json!({ "rating": "Explicit", "warnings": ["Graphic Depictions Of Violence"] })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // It leaves the queue.
    let (s, body) = send(&app, "GET", "/api/admin/rating-checks", Some(&tok), None).await;
    assert_eq!(s, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    let my_ids: Vec<i64> = items
        .iter()
        .filter(|i| i["title"].as_str().map_or(false, |t| t.starts_with("Trv ")))
        .map(|i| i["work_id"].as_i64().unwrap())
        .collect();
    assert!(
        !my_ids.contains(&(work_id as i64)),
        "verified work leaves queue: {body}"
    );
    assert!(
        !my_ids.contains(&(work_id2 as i64)),
        "verified work2 stays out: {body}"
    );

    // Verification row + works stamp exist.
    let row: (String, serde_json::Value, Option<i32>) = sqlx::query_as(
        "SELECT rating, warnings, verified_by FROM work_rating_verifications WHERE work_id = $1",
    )
    .bind(work_id)
    .fetch_one(&db)
    .await
    .expect("verification row");
    assert_eq!(row.0, "Explicit");
    assert_eq!(row.1[0], "Graphic Depictions Of Violence");
    assert_eq!(row.2, Some(admin));
    let stamped: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT rating_verified_at FROM works WHERE id = $1")
            .bind(work_id)
            .fetch_one(&db)
            .await
            .expect("works stamp");
    assert!(stamped.is_some());

    cleanup(&db, &[&format!("{PREFIX}_rc_admin")]).await;
}

/// Verify endpoint rejects empty bodies and missing works.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn rating_check_verify_validation() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_rv_admin"), 10).await;
    let tok = auth_header(admin, &format!("{PREFIX}_rv_admin"), 10);

    let (s, body) = send(
        &app,
        "POST",
        "/api/admin/rating-checks/1/verify",
        Some(&tok),
        Some(json!({})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("At least one"));

    let (s, _) = send(
        &app,
        "POST",
        "/api/admin/rating-checks/999999/verify",
        Some(&tok),
        Some(json!({ "rating": "Explicit" })),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    cleanup(&db, &[&format!("{PREFIX}_rv_admin")]).await;
}

// ── Character / relationship score fixing ──────────────────────────────

/// PUT /api/admin/characters/{id}/score updates fic_tags.score and logs the
/// old → new in tag_score_fixes.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn char_score_fix_flow() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_cs_admin"), 10).await;
    let (_, url_id) = seed_work(
        &db,
        "trv-cs",
        "Trv Cs Fic",
        Some(r#"{"rating":"Teen And Up"}"#),
    )
    .await;
    // Scraper misordered: this character got score 1, should be 10 (main).
    let tag_id = seed_char_tag(&db, &url_id, "Trv Char Hermione", 1).await;

    let tok = auth_header(admin, &format!("{PREFIX}_cs_admin"), 10);

    let (s, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/characters/{tag_id}/score"),
        Some(&tok),
        Some(json!({ "url_id": url_id, "score": 10 })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["old_score"], 1);
    assert_eq!(body["new_score"], 10);

    let score: i16 =
        sqlx::query_scalar("SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
            .bind(&url_id)
            .bind(tag_id)
            .fetch_one(&db)
            .await
            .expect("fic_tags row");
    assert_eq!(score, 10);

    let log: (i16, i16, Option<i32>) = sqlx::query_as(
        "SELECT old_score, new_score, fixed_by FROM tag_score_fixes WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(&url_id)
    .bind(tag_id)
    .fetch_one(&db)
    .await
    .expect("fix log row");
    assert_eq!(log.0, 1);
    assert_eq!(log.1, 10);
    assert_eq!(log.2, Some(admin));

    // Setting the same score again is a no-op (no extra log row).
    let (s, body) = send(
        &app,
        "PUT",
        &format!("/api/admin/characters/{tag_id}/score"),
        Some(&tok),
        Some(json!({ "url_id": url_id, "score": 10 })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.to_string().contains("unchanged"));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tag_score_fixes WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(&url_id)
    .bind(tag_id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(count, 1);

    cleanup(&db, &[&format!("{PREFIX}_cs_admin")]).await;
}

/// Score endpoint: missing fic_tags row → 404; missing fields → 400; role gate.
#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn char_score_fix_validation() {
    let _guard = db_guard();
    let db = pool().await;
    let app = app().await;

    let admin = seed_user(&db, &format!("{PREFIX}_cv_admin"), 10).await;
    let low = seed_user(&db, &format!("{PREFIX}_cv_low"), 5).await;
    let tok = auth_header(admin, &format!("{PREFIX}_cv_admin"), 10);

    // Missing url_id → 400.
    let (s, _) = send(
        &app,
        "PUT",
        "/api/admin/characters/1/score",
        Some(&tok),
        Some(json!({ "score": 10 })),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // Nonexistent fic_tags row → 404.
    let (s, _) = send(
        &app,
        "PUT",
        "/api/admin/characters/1/score",
        Some(&tok),
        Some(json!({ "url_id": "trv-nonexistent", "score": 10 })),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // Role gate: anonymous → 401 err, sub-admin → 403.
    let (s, _) = send(
        &app,
        "PUT",
        "/api/admin/characters/1/score",
        None,
        Some(json!({"url_id":"x","score":1})),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = send(
        &app,
        "PUT",
        "/api/admin/characters/1/score",
        Some(&auth_header(low, &format!("{PREFIX}_cv_low"), 5)),
        Some(json!({"url_id":"x","score":1})),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    cleanup(
        &db,
        &[&format!("{PREFIX}_cv_admin"), &format!("{PREFIX}_cv_low")],
    )
    .await;
}

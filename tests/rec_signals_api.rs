//! Regression tests for `recommender::signals::refresh_signals`.
//!
//! See `docs/specs/refresh-signals-orphan-bookmarks.md`.
//!
//! `rec_user_signals.work_id` has a foreign key onto `fic_info(id)`, but
//! `bookmarks.url_id`, `work_ratings.url_id` and `reviews.url_id` each have no
//! such constraint. All three are populated with `fic_info` slugs elsewhere in
//! the codebase, so each can name a row that does not exist.
//!
//! `refresh_signals` rebuilds the entire `rec_user_signals` table in one
//! statement, so a single orphan anywhere took down recommendations for every
//! user with:
//!
//!     insert or update on table "rec_user_signals" violates foreign key
//!     constraint "rec_user_signals_work_id_fkey"

use std::sync::{Mutex, OnceLock};

static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn db_lock() -> &'static Mutex<()> {
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
fn db_guard() -> std::sync::MutexGuard<'static, ()> {
    db_lock().lock().unwrap_or_else(|e| e.into_inner())
}

async fn pool() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (load .env)");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("db connect")
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
        .expect("seed_user: lookup failed")
}

/// Insert a `fic_info` row. `status` and `source` are NOT NULL with no
/// default; these are the values real rows carry, not invented ones.
async fn seed_fic(pool: &sqlx::PgPool, id: &str) {
    sqlx::query(
        r#"INSERT INTO fic_info
             (id, title, author, chapters, words, description, fic_created,
              fic_updated, status, source)
           VALUES ($1, 'Signal Test Fic', 'Signal Author', 1, 1000, 'test', NOW(), NOW(),
                   'complete', 'ao3')
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// A bookmark whose `url_id` resolves. This one must still produce a signal.
async fn seed_resolving_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str) {
    sqlx::query(
        r#"INSERT INTO bookmarks (user_id, url_id, is_private, created_at)
           VALUES ($1, $2, false, NOW())
           ON CONFLICT (user_id, url_id) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(url_id)
    .execute(pool)
    .await
    .expect("seed resolving bookmark failed");
}

/// A bookmark whose `url_id` names a `fic_info` row that does not exist.
/// Nothing prevents this: `bookmarks` constrains `user_id` and `work_id`, never
/// `url_id`.
async fn seed_orphan_bookmark(pool: &sqlx::PgPool, user_id: i32, url_id: &str) {
    sqlx::query(
        r#"INSERT INTO bookmarks (user_id, url_id, is_private, created_at)
           VALUES ($1, $2, false, NOW())
           ON CONFLICT (user_id, url_id) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(url_id)
    .execute(pool)
    .await
    .expect("seed orphan bookmark failed");
}

async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM rec_user_signals WHERE work_id LIKE 'sigtest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM bookmarks WHERE url_id LIKE 'sigtest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'sigtest%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE username LIKE 'sigtest%'")
        .execute(pool)
        .await;
}

/// `refresh_signals` must tolerate an orphaned bookmark and must still record
/// the resolving one.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn refresh_signals_tolerates_orphaned_bookmark() {
    let _guard = db_guard();
    let pool = pool().await;

    cleanup(&pool).await;

    let user_id = seed_user(&pool, "sigtest_user").await;
    seed_fic(&pool, "sigtest_real").await;
    seed_resolving_bookmark(&pool, user_id, "sigtest_real").await;
    seed_orphan_bookmark(&pool, user_id, "sigtest_gone").await;

    // The orphan is structurally allowed, so this must not fail.
    fichub::recommender::signals::refresh_signals(&pool, None)
        .await
        .expect("refresh_signals must tolerate an orphaned bookmark");

    // The load-bearing assertion. A "fix" that dropped every bookmark would
    // satisfy the call above and silently disable the recommender's primary
    // signal source, so check the resolving bookmark survived.
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM rec_user_signals WHERE work_id = 'sigtest_real'")
            .fetch_one(&pool)
            .await
            .expect("count signals");
    assert_eq!(
        count, 1,
        "a bookmark whose url_id resolves must still produce a signal row"
    );

    // And the orphan must not have been inserted.
    let orphan_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM rec_user_signals WHERE work_id = 'sigtest_gone'")
            .fetch_one(&pool)
            .await
            .expect("count orphan signals");
    assert_eq!(
        orphan_count, 0,
        "an unresolved url_id must not become a signal"
    );

    cleanup(&pool).await;
}

/// The shared test database carries orphaned bookmarks left by other suites
/// (`user_export_api`, `bookmark_csv_api`). `refresh_signals` reads every
/// bookmark in the table, so this proves the fix against rows this test did not
/// create. The count is reported rather than asserted, because it depends on
/// which suites ran first; the assertion above is what makes the behaviour
/// testable.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "DB-gated"]
async fn refresh_signals_tolerates_foreign_orphans() {
    let _guard = db_guard();
    let pool = pool().await;

    let orphans: i64 = sqlx::query_scalar(
        r#"SELECT count(*) FROM bookmarks b
           LEFT JOIN fic_info f ON f.id = b.url_id
           WHERE f.id IS NULL"#,
    )
    .fetch_one(&pool)
    .await
    .expect("count foreign orphans");

    fichub::recommender::signals::refresh_signals(&pool, None)
        .await
        .expect("refresh_signals must tolerate orphans left by other suites");

    eprintln!("orphaned bookmarks present during this run: {orphans}");
}

//! DB-gated integration tests for `tags::backfill::backfill_scores`.
//!
//! Follows the repo's DB-gated conventions (see tests/roadmap_api.rs): a
//! global Mutex so DB tests never run concurrently, `#[ignore]` so they only
//! run with `--include-ignored`, and a plain `pool()` helper (no AppState
//! needed — backfill only touches fic_tags/tags).
//!
//! Run: `cargo test --test backfill_scores -- --include-ignored --test-threads=1`
//! (with `.env` loaded so DATABASE_URL is set).

use std::sync::{Mutex, OnceLock};

use sqlx::Row;

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

/// Seed a fic_info row (idempotent).
async fn seed_fic(pool: &sqlx::PgPool, id: &str) {
    sqlx::query(
        "INSERT INTO fic_info (id, title, author, source, description, chapters, words, status, fic_created, fic_updated, extra_meta, raw_extended_meta, content_hash)
         VALUES ($1, 'Backfill Test Fic', 'backfill_author', 'https://example.test/backfill', 'desc', 1, 1000, 'complete', NOW(), NOW(), '{}', '{}', 'backfill-hash')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("seed_fic failed");
}

/// Seed a tag row; returns its id. ON CONFLICT (name) DO NOTHING keeps it
/// idempotent (tags are shared, so this never deletes them — cleanup only
/// removes the fic_tags links).
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

/// Link a tag to a fic with an explicit score (ON CONFLICT DO UPDATE so the
/// test always starts from the exact state it wants).
async fn link_tag(pool: &sqlx::PgPool, url_id: &str, tag_id: i32, score: i16) {
    sqlx::query(
        "INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score) VALUES ($1, $2, '127.0.0.1', $3)
         ON CONFLICT (url_id, tag_id) DO UPDATE SET score = EXCLUDED.score",
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(score)
    .execute(pool)
    .await
    .expect("link_tag failed");
}

async fn score_of(pool: &sqlx::PgPool, url_id: &str, tag_id: i32) -> i16 {
    sqlx::query("SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(url_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await
        .expect("score_of failed")
        .get(0)
}

async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id LIKE 'backfill_%'")
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM fic_info WHERE id LIKE 'backfill_%'")
        .execute(pool)
        .await;
}

/// Two character tags both score 0 → first-listed (by created_at, then
/// tag_id) becomes 10, the other 1. Freeforms stay 0 (presence-only for
/// main_char_attr). Re-running is a no-op (idempotent).
#[ignore]
#[tokio::test]
async fn backfill_sets_main_and_side_character() {
    let _guard = db_guard();
    let db = pool().await;

    let url_id = "backfill_a";
    seed_fic(&db, url_id).await;
    let char1 = seed_tag(&db, "backfill_char_a1", 2).await;
    let char2 = seed_tag(&db, "backfill_char_a2", 2).await;
    let freeform = seed_tag(&db, "backfill_freeform_a", 4).await;
    link_tag(&db, url_id, char1, 0).await;
    link_tag(&db, url_id, char2, 0).await;
    link_tag(&db, url_id, freeform, 0).await;

    // Ensure char1 sorts as the first-listed character tag: rewind its
    // created_at BEFORE char2's (tie would be broken by tag_id).
    sqlx::query("UPDATE fic_tags SET created_at = NOW() - INTERVAL '1 hour' WHERE url_id = $1 AND tag_id = $2")
        .bind(url_id)
        .bind(char1)
        .execute(&db)
        .await
        .expect("make char1 first-listed");

    let summaries = fichub::tags::backfill::backfill_scores(&db)
        .await
        .expect("backfill ran");
    let s = summaries
        .iter()
        .find(|s| s.url_id == url_id)
        .expect("fic a backfilled");

    assert_eq!(
        s.main_char.as_deref(),
        Some("backfill_char_a1"),
        "first-listed char is main: {s:?}"
    );
    assert_eq!(s.other_chars, vec!["backfill_char_a2".to_string()]);
    assert!(s.primary_ship.is_none(), "no ships seeded: {s:?}");

    assert_eq!(score_of(&db, url_id, char1).await, 10, "main char → 10");
    assert_eq!(score_of(&db, url_id, char2).await, 1, "side char → 1");
    assert_eq!(
        score_of(&db, url_id, freeform).await,
        0,
        "freeform untouched (0)"
    );

    // Idempotency: second run is a no-op, scores unchanged.
    let second = fichub::tags::backfill::backfill_scores(&db)
        .await
        .expect("second run");
    assert!(
        second.iter().all(|s| s.url_id != url_id),
        "fic already backfilled must be skipped: {second:?}"
    );
    assert_eq!(score_of(&db, url_id, char1).await, 10);
    assert_eq!(score_of(&db, url_id, char2).await, 1);

    cleanup(&db).await;
}

/// One character tag score 0 → becomes 10 (its fic's only/main character).
/// A fic with an already-scored character tag is left alone entirely.
#[ignore]
#[tokio::test]
async fn backfill_single_character_and_skips_scored_fic() {
    let _guard = db_guard();
    let db = pool().await;

    // Fic B: single character at score 0 → main char 10.
    let url_b = "backfill_b";
    seed_fic(&db, url_b).await;
    let char_b = seed_tag(&db, "backfill_char_b1", 2).await;
    link_tag(&db, url_b, char_b, 0).await;

    // Fic C: character already scored (user-voted / previously backfilled)
    // → must be skipped even though it has a zero-scored second character.
    let url_c = "backfill_c";
    seed_fic(&db, url_c).await;
    let char_c1 = seed_tag(&db, "backfill_char_c1", 2).await;
    let char_c2 = seed_tag(&db, "backfill_char_c2", 2).await;
    link_tag(&db, url_c, char_c1, 3).await;
    link_tag(&db, url_c, char_c2, 0).await;

    let summaries = fichub::tags::backfill::backfill_scores(&db)
        .await
        .expect("backfill ran");
    let s_b = summaries
        .iter()
        .find(|s| s.url_id == url_b)
        .expect("fic b backfilled");
    assert_eq!(
        s_b.main_char.as_deref(),
        Some("backfill_char_b1"),
        "{s_b:?}"
    );
    assert!(s_b.other_chars.is_empty(), "{s_b:?}");
    assert_eq!(score_of(&db, url_b, char_b).await, 10, "single char → 10");

    assert!(
        summaries.iter().all(|s| s.url_id != url_c),
        "fic with nonzero char score must be skipped: {summaries:?}"
    );
    assert_eq!(
        score_of(&db, url_c, char_c1).await,
        3,
        "existing score untouched"
    );
    assert_eq!(
        score_of(&db, url_c, char_c2).await,
        0,
        "zero-scored sibling untouched"
    );

    cleanup(&db).await;
}

/// Relationship tags all score 0 → first-listed ship 5, rest 1 (scrape-time
/// `relationship_primary` convention). Re-running skips the fic.
#[ignore]
#[tokio::test]
async fn backfill_ships_primary_and_side() {
    let _guard = db_guard();
    let db = pool().await;

    let url_id = "backfill_d";
    seed_fic(&db, url_id).await;
    let ship1 = seed_tag(&db, "backfill_ship_d1", 3).await;
    let ship2 = seed_tag(&db, "backfill_ship_d2", 3).await;
    link_tag(&db, url_id, ship1, 0).await;
    link_tag(&db, url_id, ship2, 0).await;
    sqlx::query("UPDATE fic_tags SET created_at = NOW() - INTERVAL '1 hour' WHERE url_id = $1 AND tag_id = $2")
        .bind(url_id)
        .bind(ship1)
        .execute(&db)
        .await
        .expect("make ship1 first-listed");

    let summaries = fichub::tags::backfill::backfill_scores(&db)
        .await
        .expect("backfill ran");
    let s = summaries
        .iter()
        .find(|s| s.url_id == url_id)
        .expect("fic d backfilled");
    assert_eq!(s.primary_ship.as_deref(), Some("backfill_ship_d1"), "{s:?}");
    assert_eq!(s.other_ships, vec!["backfill_ship_d2".to_string()]);

    assert_eq!(score_of(&db, url_id, ship1).await, 5, "primary ship → 5");
    assert_eq!(score_of(&db, url_id, ship2).await, 1, "side ship → 1");

    let second = fichub::tags::backfill::backfill_scores(&db)
        .await
        .expect("second run");
    assert!(
        second.iter().all(|s| s.url_id != url_id),
        "already backfilled: {second:?}"
    );
    assert_eq!(score_of(&db, url_id, ship1).await, 5);
    assert_eq!(score_of(&db, url_id, ship2).await, 1);

    cleanup(&db).await;
}

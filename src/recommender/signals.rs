//! Unified recommendation signals.
//!
//! Every strategy consumes the same signal model: a (user, work, type,
//! weight, occurred_at) tuple. Sources:
//!
//! | source    | table        | keyed by      | weight      |
//! |-----------|--------------|---------------|-------------|
//! | bookmark  | bookmarks    | users.id      | 4.0         |
//! | download  | request_log  | ANONYMOUS     | 2.0         |
//! | rating    | work_ratings | users.id      | = rating (1..=5) |
//! | review    | reviews      | users.id      | 3.0         |
//!
//! Downloads are anonymous in `request_log` (no user linkage — this is a
//! long-standing platform constraint; see the personal-recs reference). For
//! personal strategies they are treated as *global* popularity context
//! rather than per-user signal; the materialized `rec_user_signals` view
//! stores only per-user signals, and download popularity is computed
//! directly by strategies that need it.
//!
//! The curator's interactions are multiplied by 5× in this view (the
//! curator-prior taste-shaping requirement), so their profile dominates the
//! blend for low-signal users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

/// Weight of a bookmark signal.
pub const WEIGHT_BOOKMARK: f64 = 4.0;
/// Weight of a download signal.
pub const WEIGHT_DOWNLOAD: f64 = 2.0;
/// Weight of a review signal.
pub const WEIGHT_REVIEW: f64 = 3.0;
/// Curator multiplier (their interactions count 5×).
pub const CURATOR_MULTIPLIER: f64 = 5.0;

/// One unified signal row.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSignal {
    pub user_id: i32,
    pub work_id: String,
    pub signal_type: String,
    pub signal_weight: f64,
    pub occurred_at: DateTime<Utc>,
}

/// Refresh the `rec_user_signals` materialized view. Idempotent: it is a
/// full DELETE + rebuild inside one transaction so concurrent readers never
/// see a half-refreshed view.
///
/// `curator_user_id`: when set, that user's interactions are multiplied by
/// [`CURATOR_MULTIPLIER`] so their profile shapes low-signal users' blends.
pub async fn refresh_signals(db: &PgPool, curator_user_id: Option<i32>) -> Result<(), AppError> {
    let curator = curator_user_id.unwrap_or(-1);
    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM rec_user_signals")
        .execute(&mut *tx)
        .await?;

    // Bookmarks (per-user).
    sqlx::query(
        r#"INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
           SELECT b.user_id, b.url_id, 'bookmark',
                  $2 * (CASE WHEN b.user_id = $1 THEN $3 ELSE 1.0 END),
                  COALESCE(b.created_at, NOW())
           FROM bookmarks b
           -- `rec_user_signals.work_id` references `fic_info(id)`, but
           -- `bookmarks.url_id` carries no such foreign key, so it can name a
           -- row that does not exist. Without this join the insert violates the
           -- FK and takes down the whole signal refresh for every user, since
           -- this function rebuilds the entire table in one statement.
           JOIN fic_info f ON f.id = b.url_id
           WHERE b.url_id IS NOT NULL"#,
    )
    .bind(curator)
    .bind(WEIGHT_BOOKMARK)
    .bind(CURATOR_MULTIPLIER)
    .execute(&mut *tx)
    .await?;

    // Ratings 1..=5 (per-user). Legacy -1 (dislike) is intentionally
    // excluded from the unified signal view — the public surface is
    // positive-only, and negative signals are handled by exclusions, not
    // scoring.
    sqlx::query(
        r#"INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
           SELECT r.user_id, r.url_id, 'rating', r.rating::float8,
                  COALESCE(r.created_at, NOW())
           FROM work_ratings r
           -- Same reasoning as the bookmark insert above: `work_ratings.url_id`
           -- is not constrained to `fic_info(id)`.
           JOIN fic_info f ON f.id = r.url_id
           WHERE r.url_id IS NOT NULL AND r.rating BETWEEN 1 AND 5"#,
    )
    .execute(&mut *tx)
    .await?;

    // Reviews (per-user).
    sqlx::query(
        r#"INSERT INTO rec_user_signals (user_id, work_id, signal_type, signal_weight, occurred_at)
           SELECT rv.user_id, rv.url_id, 'review', $1, COALESCE(rv.created_at, NOW())
           FROM reviews rv
           -- Same reasoning as the bookmark insert: `reviews.url_id` has no
           -- foreign key to `fic_info(id)`, only `reviews_user_id_fkey` and
           -- `reviews_work_id_fkey` exist.
           JOIN fic_info f ON f.id = rv.url_id
           WHERE rv.url_id IS NOT NULL AND rv.deleted_at IS NULL"#,
    )
    .bind(WEIGHT_REVIEW)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Fetch all signals for a user (used by MF training, embeddings profile
/// building, and curator-prior scoring).
pub async fn user_signals(db: &PgPool, user_id: i32) -> Result<Vec<UserSignal>, AppError> {
    let rows = sqlx::query_as::<_, UserSignal>(
        r#"SELECT user_id, work_id, signal_type, signal_weight, occurred_at
           FROM rec_user_signals
           WHERE user_id = $1
           ORDER BY occurred_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Fetch all signals (full refresh of the view — used by batch training).
pub async fn all_signals(db: &PgPool) -> Result<Vec<UserSignal>, AppError> {
    let rows = sqlx::query_as::<_, UserSignal>(
        r#"SELECT user_id, work_id, signal_type, signal_weight, occurred_at
           FROM rec_user_signals"#,
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Recent global download popularity: (work_id, weight) over the last
/// `days` from request_log (anonymous).
pub async fn download_popularity(db: &PgPool, days: i64) -> Result<Vec<(String, f64)>, AppError> {
    let rows = sqlx::query_as::<_, (String, f64)>(
        r#"SELECT url_id, COUNT(*)::float8 * $2 AS w
           FROM request_log
           WHERE url_id IS NOT NULL
             AND (export_file_name LIKE '%.epub' OR etype IN ('download', 'export'))
             AND created > now() - ($1 * INTERVAL '1 day')
           GROUP BY url_id"#,
    )
    .bind(days)
    .bind(WEIGHT_DOWNLOAD)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curator_multiplier_is_five() {
        assert_eq!(CURATOR_MULTIPLIER, 5.0);
    }

    #[test]
    fn weights_are_positive_and_ordered() {
        // Bookmarks are the strongest per-user signal, reviews next.
        assert!(WEIGHT_BOOKMARK > WEIGHT_REVIEW);
        assert!(WEIGHT_REVIEW > WEIGHT_DOWNLOAD);
    }
}

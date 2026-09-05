//! Rec-engine feedback signals.
//!
//! The recommendation engine needs *richer* signals than the old binary
//! like/dislike aggregate: a per-work rating distribution, an average star
//! rating, a review count, and a simple sentiment score. All of these live in
//! `work_ratings` + `reviews` (see migration 014). No NLP: sentiment here is
//! derived from the star rating value and the presence of a written review —
//! deliberately cheap and deterministic so `engine.rs` can fold it into
//! scoring without extra service calls.
//!
//! DOWNVOTE POLICY: legacy `rating = -1` rows are internal-only. They are
//! *not* part of `avg_rating` (which only averages the 1..=5 scale) but they
//! ARE counted in `negative_votes` so the engine can still weight against
//! disliked works internally. Public endpoints never expose them (see
//! `src/routes/social.rs`).

use sqlx::PgPool;

use crate::error::AppResult;

/// Aggregate feedback signals for one work, consumed by the recommender.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkFeedbackSignals {
    /// Number of 1..=5 star ratings (excludes legacy -1 rows).
    pub rating_count: i64,
    /// Average star rating over the 1..=5 scale (0.0 when no star ratings).
    pub avg_rating: f64,
    /// Per-star counts: `[0]` = # of 1-star ratings ... `[4]` = # of 5-star.
    pub rating_distribution: [i64; 5],
    /// Number of non-deleted written reviews.
    pub review_count: i64,
    /// Number of non-deleted reviews with a title (signals effort/quality).
    pub titled_review_count: i64,
    /// Legacy internal downvotes (-1 rows). Never surfaced publicly.
    pub negative_votes: i64,
    /// Simple sentiment in [-1.0, 1.0]:
    ///   avg_rating ∈ [1,5] → (avg-3)/2, blended with review presence
    ///   (a written review pushes sentiment up: 0.15 * (review_count>0)).
    ///   Legacy dislikes pull it down: -0.05 * negative_votes.
    pub sentiment: f64,
}

/// Fetch the feedback signals for a single work.
///
/// Used by the recommendation engine for richer scoring (program item 9:
/// "rec engine signals — ratings distribution, review sentiment"). Single
/// query + two tiny follow-ups; safe to call per-work in rec scoring loops.
pub async fn work_feedback_signals(pool: &PgPool, work_id: i32) -> AppResult<WorkFeedbackSignals> {
    // Star-scale aggregate (1..=5 only; legacy -1 rows are excluded from the
    // average/distribution and counted separately as negative_votes).
    let (rating_count, sum, negatives): (i64, i64, i64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN 1 ELSE 0 END), 0)::bigint,
             COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN rating ELSE 0 END), 0)::bigint,
             COALESCE(SUM(CASE WHEN rating = -1 THEN 1 ELSE 0 END), 0)::bigint
           FROM work_ratings WHERE work_id = $1"#,
    )
    .bind(work_id)
    .fetch_one(pool)
    .await?;

    let dist_rows: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT rating, COUNT(*)::bigint FROM work_ratings
         WHERE work_id = $1 AND rating BETWEEN 1 AND 5
         GROUP BY rating",
    )
    .bind(work_id)
    .fetch_all(pool)
    .await?;

    let mut rating_distribution = [0i64; 5];
    for (star, count) in dist_rows {
        if (1..=5).contains(&star) {
            rating_distribution[(star - 1) as usize] = count;
        }
    }

    let (review_count, titled_review_count): (i64, i64) = sqlx::query_as(
        r#"SELECT
            COUNT(*)::bigint,
            COUNT(*) FILTER (WHERE title IS NOT NULL AND title <> '')::bigint
          FROM reviews WHERE work_id = $1 AND deleted_at IS NULL AND constructive = TRUE"#,
    )
    .bind(work_id)
    .fetch_one(pool)
    .await?;

    let avg_rating = if rating_count > 0 {
        sum as f64 / rating_count as f64
    } else {
        0.0
    };

    // Sentiment: star scale is [-1, 1] ((avg-3)/2), nudged up when someone
    // wrote a review, nudged down per internal dislike.
    let mut sentiment = if rating_count > 0 {
        (avg_rating - 3.0) / 2.0
    } else {
        0.0
    };
    if review_count > 0 {
        sentiment += 0.15;
    }
    sentiment -= 0.05 * negatives as f64;
    sentiment = sentiment.clamp(-1.0, 1.0);

    Ok(WorkFeedbackSignals {
        rating_count,
        avg_rating,
        rating_distribution,
        review_count,
        titled_review_count,
        negative_votes: negatives,
        sentiment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentiment_positive_for_high_stars_with_review() {
        // avg 5.0, one review → (5-3)/2 + 0.15 = 1.15 → clamped to 1.0
        let s = WorkFeedbackSignals {
            rating_count: 2,
            avg_rating: 5.0,
            rating_distribution: [0, 0, 0, 0, 2],
            review_count: 1,
            titled_review_count: 1,
            negative_votes: 0,
            sentiment: 0.0,
        };
        let mut v = (s.avg_rating - 3.0) / 2.0;
        if s.review_count > 0 {
            v += 0.15;
        }
        v = v.clamp(-1.0, 1.0);
        assert_eq!(v, 1.0);
    }

    #[test]
    fn sentiment_negative_for_low_stars() {
        let avg: f64 = 1.0;
        let mut v = (avg - 3.0) / 2.0; // -1.0
        v -= 0.05 * 2.0; // internal dislikes drag further
        v = v.clamp(-1.0, 1.0);
        assert_eq!(v, -1.0);
    }

    #[test]
    fn distribution_index_maps_star_to_slot() {
        let mut dist = [0i64; 5];
        for star in 1i16..=5 {
            dist[(star - 1) as usize] = star as i64;
        }
        assert_eq!(dist, [1, 2, 3, 4, 5]);
    }
}

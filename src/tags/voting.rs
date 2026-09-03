use crate::error::AppResult;
use sqlx::PgPool;
use std::net::IpAddr;

/// Result of recording a vote.
pub struct VoteResult {
    pub new_score: i16,
    pub hidden: bool,
}

/// Snapshot of vote counts for a tag.
pub struct VoteCounts {
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i16,
}

/// Record or update a vote, returning the new score.
///
/// Uses INSERT ... ON CONFLICT DO UPDATE so the same IP can change their vote.
/// The `update_fic_tag_score` trigger on `fic_tag_votes` adjusts `fic_tags.score`
/// automatically.
pub async fn record_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: IpAddr,
    value: i16,
    hidden_threshold: i16,
) -> AppResult<VoteResult> {
    let ip_str = voter_ip.to_string();

    // Upsert the vote — the trigger handles score update
    sqlx::query(
        r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, voter_ip) DO UPDATE SET value = EXCLUDED.value"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(&ip_str)
    .bind(value)
    .execute(pool)
    .await?;

    // Read back the updated score
    let row: (i16,) = sqlx::query_as(
        "SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;

    let new_score = row.0;
    let hidden = is_hidden(new_score, hidden_threshold);

    Ok(VoteResult { new_score, hidden })
}

/// Returns `true` when `score` is at or below the visibility threshold.
pub fn is_hidden(score: i16, threshold: i16) -> bool {
    score <= threshold
}

/// Bulk visibility check.
///
/// Input: `[(tag_id, score), ...]`
/// Output: `[(tag_id, is_hidden), ...]`
pub fn compute_visibility(scores: &[(i32, i16)], threshold: i16) -> Vec<(i32, bool)> {
    scores
        .iter()
        .map(|(tag_id, score)| (*tag_id, is_hidden(*score, threshold)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hidden_above_threshold() {
        assert!(!is_hidden(0, -3));
        assert!(!is_hidden(-2, -3));
        assert!(!is_hidden(10, -3));
    }

    #[test]
    fn test_is_hidden_at_or_below_threshold() {
        assert!(is_hidden(-3, -3));
        assert!(is_hidden(-5, -3));
        assert!(is_hidden(-10, -3));
        assert!(is_hidden(0, 0));
    }

    #[test]
    fn test_compute_visibility() {
        let scores = vec![(1, 5), (2, 0), (3, -3), (4, -10)];
        let result = compute_visibility(&scores, -3);
        assert_eq!(result.len(), 4);
        assert!(!result[0].1); // score=5 > -3 => visible
        assert!(!result[1].1); // score=0 > -3 => visible
        assert!(result[2].1);  // score=-3 == -3 => hidden
        assert!(result[3].1);  // score=-10 < -3 => hidden
    }

    #[test]
    fn test_empty_scores() {
        let result = compute_visibility(&[], -3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_vote_result_struct() {
        let result = VoteResult {
            new_score: 5,
            hidden: false,
        };
        assert_eq!(result.new_score, 5);
        assert!(!result.hidden);

        let hidden_result = VoteResult {
            new_score: -3,
            hidden: true,
        };
        assert_eq!(hidden_result.new_score, -3);
        assert!(hidden_result.hidden);
    }

    #[test]
    fn test_vote_counts_struct() {
        let counts = VoteCounts {
            upvotes: 10,
            downvotes: 2,
            score: 8,
        };
        assert_eq!(counts.upvotes, 10);
        assert_eq!(counts.downvotes, 2);
        assert_eq!(counts.score, 8);

        let zero_counts = VoteCounts {
            upvotes: 0,
            downvotes: 0,
            score: 0,
        };
        assert_eq!(zero_counts.upvotes, 0);
        assert_eq!(zero_counts.downvotes, 0);
        assert_eq!(zero_counts.score, 0);
    }
}

use crate::db::models::*;
use super::create_work;
use crate::error::AppResult;
use sqlx::PgPool;

// ── Proposal queries ──────────────────────────────────────────────

/// Create a work proposal
pub async fn create_proposal(
    pool: &PgPool,
    proposer_id: i32,
    action_type: &str,
    source_work_id: Option<i32>,
    target_work_id: Option<i32>,
    work_id: Option<i32>,
    details: Option<serde_json::Value>,
) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        r#"INSERT INTO work_proposals (proposer_id, action_type, source_work_id, target_work_id, work_id, details)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
    )
    .bind(proposer_id)
    .bind(action_type)
    .bind(source_work_id)
    .bind(target_work_id)
    .bind(work_id)
    .bind(details)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List pending proposals
pub async fn list_pending_proposals(pool: &PgPool) -> AppResult<Vec<WorkProposal>> {
    let rows = sqlx::query_as::<_, WorkProposal>(
        r#"SELECT id, proposer_id, action_type, source_work_id, target_work_id, work_id, details, status, created_at, closed_at
           FROM work_proposals
           WHERE status = 'pending'
           ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a proposal by ID
pub async fn get_proposal(pool: &PgPool, proposal_id: i32) -> AppResult<Option<WorkProposal>> {
    let row = sqlx::query_as::<_, WorkProposal>(
        r#"SELECT id, proposer_id, action_type, source_work_id, target_work_id, work_id, details, status, created_at, closed_at
           FROM work_proposals WHERE id = $1"#,
    )
    .bind(proposal_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Cast a vote on a proposal
pub async fn cast_proposal_vote(
    pool: &PgPool,
    proposal_id: i32,
    user_id: i32,
    vote: i16,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO work_proposal_votes (proposal_id, user_id, vote)
           VALUES ($1, $2, $3)
           ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = $3, voted_at = NOW()"#,
    )
    .bind(proposal_id)
    .bind(user_id)
    .bind(vote)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get vote sum for a proposal
pub async fn get_proposal_vote_sum(pool: &PgPool, proposal_id: i32) -> AppResult<(i64, i64)> {
    let row: (Option<i64>, Option<i64>) = sqlx::query_as(
        r#"SELECT COALESCE(SUM(vote), 0), COUNT(DISTINCT user_id)
           FROM work_proposal_votes WHERE proposal_id = $1"#,
    )
    .bind(proposal_id)
    .fetch_one(pool)
    .await?;
    Ok((row.0.unwrap_or(0), row.1.unwrap_or(0)))
}

/// Check if user has senior curator role (role >= 2)
pub async fn is_senior_curator(pool: &PgPool, user_id: i32) -> AppResult<bool> {
    let row: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(matches!(
        row.as_ref().map(|r| r.0.as_str()),
        Some("admin") | Some("moderator")
    ))
}

/// Update proposal status
pub async fn update_proposal_status(
    pool: &PgPool,
    proposal_id: i32,
    status: &str,
) -> AppResult<()> {
    sqlx::query("UPDATE work_proposals SET status = $1, closed_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(proposal_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Execute a merge: move sources from source_work to target_work
pub async fn execute_merge(
    pool: &PgPool,
    source_work_id: i32,
    target_work_id: i32,
) -> AppResult<()> {
    // Move all sources
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE work_id = $2")
        .bind(target_work_id)
        .bind(source_work_id)
        .execute(pool)
        .await?;

    // Update target work metadata from most common values
    sqlx::query(
        r#"UPDATE works SET
            canonical_title = (SELECT canonical_title FROM works WHERE id = $1),
            canonical_author = (SELECT canonical_author FROM works WHERE id = $1),
            description = (SELECT description FROM works WHERE id = $1),
            updated_at = NOW()
           WHERE id = $2"#,
    )
    .bind(source_work_id)
    .bind(target_work_id)
    .execute(pool)
    .await?;

    // Delete source work
    sqlx::query("DELETE FROM works WHERE id = $1")
        .bind(source_work_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Split a merged work back into separate works.
///
/// Given a work_id and a list of source IDs to split off,
/// creates new works for each source and reassigns their sources.
/// The original work keeps sources not in the split list.
pub async fn execute_split(
    pool: &PgPool,
    work_id: i32,
    split_source_ids: &[&str],
) -> AppResult<()> {
    // For each source to split off, create a new work and reassign sources
    for &source_id in split_source_ids {
        // Get the source's original title/author before merge
        let source: Option<(String, String, String)> =
            sqlx::query_as("SELECT title, author, description FROM fic_info WHERE id = $1")
                .bind(source_id)
                .fetch_optional(pool)
                .await?;

        if let Some((title, author, desc)) = source {
            // Create new work for this source
            let new_work_id = create_work(pool, &title, &author, &desc, Some(source_id)).await?;

            // Reassign the source to the new work
            sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
                .bind(new_work_id)
                .bind(source_id)
                .execute(pool)
                .await?;
        }
    }

    // Update the original work's metadata from remaining sources
    sqlx::query(
        r#"UPDATE works SET
            canonical_title = COALESCE(
                (SELECT title FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                canonical_title
            ),
            canonical_author = COALESCE(
                (SELECT author FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                canonical_author
            ),
            description = COALESCE(
                (SELECT description FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                description
            ),
            updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(work_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Reputation & Auto-Promotion ─────────────────────────────────────
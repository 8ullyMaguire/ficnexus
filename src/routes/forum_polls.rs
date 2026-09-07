//! Site-wide polls API (Lane B).
//!
//! Live schema:
//!   forum_polls(id, topic_id, question, max_selections, allow_change, close_at, created_at, updated_at)
//!   forum_poll_options(id, poll_id, text, position, vote_count)
//!   forum_poll_votes(poll_id, option_id, user_id, created_at)

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::require_user;
use crate::server::AppState;
use crate::services::trust::{self, PUBLISH_MIN_TRUST};

// ── Public types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Poll {
    pub id: i64,
    pub topic_id: i64,
    pub question: String,
    pub max_selections: i32,
    pub allow_change: bool,
    pub is_closed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct PollOption {
    pub id: i64,
    pub text: String,
    pub position: i32,
    pub vote_count: i64,
}

#[derive(Deserialize)]
pub struct VoteBody {
    pub option_id: i64,
}

#[derive(Deserialize)]
pub struct CreatePollBody {
    pub question: String,
    pub options: Vec<String>,
    #[serde(default = "default_max_selections")]
    pub max_selections: i32,
    #[serde(default = "default_allow_change")]
    pub allow_change: bool,
    #[serde(default)]
    pub close_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn default_max_selections() -> i32 {
    1
}

fn default_allow_change() -> bool {
    true
}

// ── Handlers ──────────────────────────────────────────────────────────────

/// `POST /api/forum/topics/{topicId}/polls` — attach one poll to a topic.
/// Gate: topic author or staff; one poll per topic; PUBLISH_MIN_TRUST.
pub async fn create_poll(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(topic_id): Path<i64>,
    Json(body): Json<CreatePollBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    trust::assert_staff_or_min_trust(&state.db, Some(user_id), auth.role, PUBLISH_MIN_TRUST, "Creating polls").await?;

    let question = body.question.trim().to_string();
    if question.is_empty() || question.chars().count() > 500 {
        return Err(AppError::BadRequest(
            "question required (max 500 chars)".to_string(),
        ));
    }
    let options: Vec<String> = body
        .options
        .into_iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();
    if options.len() < 2 || options.len() > 10 {
        return Err(AppError::BadRequest(
            "poll needs 2..=10 non-empty options".to_string(),
        ));
    }
    if body.max_selections < 1 || body.max_selections > options.len() as i32 {
        return Err(AppError::BadRequest(
            "max_selections must be within 1..=options.len()".to_string(),
        ));
    }

    // Topic must exist; caller must be its author or staff.
    let topic_author: Option<i32> = sqlx::query_scalar(
        "SELECT author_id FROM forum_topics WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(topic_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();
    let topic_author =
        topic_author.ok_or_else(|| AppError::NotFound("topic not found".to_string()))?;
    let is_staff = auth.level >= 50 || auth.role >= 10;
    if !is_staff && topic_author != user_id {
        return Err(AppError::Forbidden(
            "only the topic author or staff can attach a poll".to_string(),
        ));
    }

    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM forum_polls WHERE topic_id = $1")
            .bind(topic_id)
            .fetch_optional(&state.db)
            .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "this topic already has a poll".to_string(),
        ));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let poll_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_polls (topic_id, question, max_selections, allow_change, close_at)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(topic_id)
    .bind(&question)
    .bind(body.max_selections)
    .bind(body.allow_change)
    .bind(body.close_at)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    for (pos, text) in options.iter().enumerate() {
        sqlx::query(
            "INSERT INTO forum_poll_options (poll_id, text, position) VALUES ($1, $2, $3)",
        )
        .bind(poll_id)
        .bind(text)
        .bind(pos as i32)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    sqlx::query("UPDATE forum_topics SET poll_id = $1 WHERE id = $2")
        .bind(poll_id)
        .bind(topic_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "err": 0, "poll_id": poll_id })))
}

/// `GET /api/forum/polls/{pollId}`
pub async fn get_poll(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(poll_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // Fetch poll + options + votes in two queries
    let poll_row = sqlx::query_as::<_, PollRow>(
        "SELECT id, topic_id, question, max_selections, allow_change,
                coalesce(close_at < now(), false), created_at, updated_at
         FROM forum_polls WHERE id = $1",
    )
    .bind(poll_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    let options: Vec<PollOptionRow> =
        sqlx::query_as("SELECT id, poll_id, text, position, vote_count FROM forum_poll_options WHERE poll_id = $1 ORDER BY position")
            .bind(poll_id)
            .fetch_all(&state.db)
            .await?;

    // For closed polls or anyone viewing results: fetch actual vote counts
    let mut option_votes: Vec<(i64, i64)> = vec![];
    if poll_row.is_closed {
        option_votes = sqlx::query_as::<_, (i64, i64)>(
            "SELECT option_id, COUNT(*) FROM forum_poll_votes WHERE poll_id = $1 GROUP BY option_id",
        )
        .bind(poll_id)
        .fetch_all(&state.db)
        .await?;
    }

    let item = json!({
        "err": 0,
        "poll": {
            "id": poll_row.id,
            "topic_id": poll_row.topic_id,
            "question": poll_row.question,
            "max_selections": poll_row.max_selections,
            "allow_change": poll_row.allow_change,
            "is_closed": poll_row.is_closed,
            "created_at": poll_row.created_at,
            "updated_at": poll_row.updated_at,
            "options": options.into_iter().map(|o| json!({
                "id": o.id,
                "text": o.text,
                "position": o.position,
                "vote_count": o.vote_count,
                "votes": if poll_row.is_closed {
                    option_votes.iter()
                        .find(|(oid, _)| *oid == o.id)
                        .map(|(_, c)| *c as i32)
                        .unwrap_or(0)
                } else {
                    0
                },
                "voted_by_user": false, // TODO: check against session user
            })).collect::<Vec<_>>(),
        },
    });

    Ok(Json(item))
}

/// `POST /api/forum/polls/{pollId}/vote`
pub async fn vote_on_poll(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(poll_id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    // Check poll exists and isn't closed
    let close_at: Option<Option<chrono::DateTime<chrono::Utc>>> = sqlx::query_scalar(
        "SELECT close_at FROM forum_polls WHERE id = $1",
    )
    .bind(poll_id)
    .fetch_optional(&state.db)
    .await?;
    let Some(close_at) = close_at else {
        return Err(AppError::Conflict(
            "poll is closed or doesn't exist".to_string(),
        ));
    };
    if close_at.is_some_and(|ts| ts <= chrono::Utc::now()) {
        return Err(AppError::Conflict("poll is closed".to_string()));
    }

    // Validate option belongs to this poll
    let option_exists: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM forum_poll_options WHERE poll_id = $1 AND id = $2",
    )
    .bind(poll_id)
    .bind(body.option_id)
    .fetch_optional(&state.db)
    .await?;

    if option_exists.is_none() {
        return Err(AppError::BadRequest("invalid option".into()));
    }

    // Allow change or no previous vote
    let has_vote: Option<i64> = sqlx::query_scalar(
        "SELECT option_id FROM forum_poll_votes WHERE poll_id = $1 AND user_id = $2",
    )
    .bind(poll_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(existing_option_id) = has_vote {
        if !poll_allow_change(&state.db, poll_id).await? {
            return Err(AppError::BadRequest("poll does not allow changing votes".into()));
        }
        // Update existing vote
        sqlx::query("UPDATE forum_poll_votes SET option_id = $1, created_at = NOW() WHERE poll_id = $2 AND user_id = $3")
            .bind(body.option_id)
            .bind(poll_id)
            .bind(user_id)
            .execute(&state.db)
            .await?;
    } else {
        // Insert new vote
        sqlx::query("INSERT INTO forum_poll_votes (poll_id, option_id, user_id, created_at) VALUES ($1, $2, $3, NOW())")
            .bind(poll_id)
            .bind(body.option_id)
            .bind(user_id)
            .execute(&state.db)
            .await?;
    }

    Ok(Json(json!({ "err": 0, "voted": true, "option_id": body.option_id })))
}

/// `POST /api/forum/polls/{pollId}/close`
pub async fn close_poll(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(poll_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    // Only poll author or admin can close
    let poll_author: Option<i32> = sqlx::query_scalar(
        "SELECT author_id FROM forum_topics t JOIN forum_polls p ON p.topic_id = t.id WHERE p.id = $1",
    )
    .bind(poll_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    let is_admin = auth.level >= 50 || auth.role >= 10;
    let is_author = poll_author.map(|a| a == user_id).unwrap_or(false);

    if !is_admin && !is_author {
        return Err(AppError::Forbidden(
            "only poll author or staff can close".to_string(),
        ));
    }

    sqlx::query("UPDATE forum_polls SET close_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(poll_id)
        .execute(&state.db)
        .await?;

    // Notify all distinct voters via the site notification producer
    // (never forum_notifications). Best-effort per recipient.
    let topic: Option<(i64, Option<String>, String)> = sqlx::query_as(
        "SELECT t.id, t.topic_slug, t.title FROM forum_topics t
         JOIN forum_polls p ON p.topic_id = t.id WHERE p.id = $1",
    )
    .bind(poll_id)
    .fetch_optional(&state.db)
    .await?;
    if let Some((topic_id, slug_opt, title)) = topic {
        let voters: Vec<i32> = sqlx::query_scalar(
            "SELECT DISTINCT user_id FROM forum_poll_votes WHERE poll_id = $1",
        )
        .bind(poll_id)
        .fetch_all(&state.db)
        .await?;
        let link = match slug_opt {
            Some(slug) => format!("/forum/board/{}.{}", slug, topic_id),
            None => format!("/forum/topic/{}", topic_id),
        };
        for voter in voters {
            let _ = queries::create_notification(
                &state.db,
                voter,
                "poll_closed",
                &format!("Poll closed: {}", title),
                None,
                Some(&link),
                Some("forum_poll"),
                Some(&poll_id.to_string()),
            )
            .await;
        }
    }

    Ok(Json(json!({ "err": 0, "closed": true })))
}

/// `GET /api/forum/polls/{pollId}/results`
pub async fn get_poll_results(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(poll_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let poll = sqlx::query_as::<_, PollRow>(
        "SELECT id, topic_id, question, max_selections, allow_change,
                coalesce(close_at < now(), false), created_at, updated_at
         FROM forum_polls WHERE id = $1",
    )
    .bind(poll_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("poll not found".into()))?;

    let options: Vec<PollOptionRow> =
        sqlx::query_as("SELECT id, poll_id, text, position, vote_count FROM forum_poll_options WHERE poll_id = $1 ORDER BY position")
            .bind(poll_id)
            .fetch_all(&state.db)
            .await?;

    // Always show vote counts for results endpoint
    let option_votes: Vec<(i64, i64)> = sqlx::query_as::<_, (i64, i64)>(
        "SELECT option_id, COUNT(*) FROM forum_poll_votes WHERE poll_id = $1 GROUP BY option_id",
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    let total_votes: i64 = option_votes.iter().map(|(_, c)| c).sum();

    let item = json!({
        "err": 0,
        "poll": {
            "id": poll.id,
            "question": poll.question,
            "is_closed": poll.is_closed,
            "total_votes": total_votes,
            "options": options.into_iter().map(|o| {
                let pct = if total_votes > 0 {
                    (((option_votes.iter()
                        .find(|(oid, _)| *oid == o.id)
                        .map(|(_, c)| *c as f64)
                        .unwrap_or(0f64)) / total_votes as f64) * 100.0) as i32
                } else {
                    0
                };
                json!({
                    "id": o.id,
                    "text": o.text,
                    "vote_count": o.vote_count,
                    "percentage": pct,
                })
            }).collect::<Vec<_>>(),
        },
    });

    Ok(Json(item))
}

// ── Helpers ───────────────────────────────────────────────────────────────

async fn poll_allow_change(db: &sqlx::PgPool, poll_id: i64) -> Result<bool, AppError> {
    let allow: bool = sqlx::query_scalar("SELECT allow_change FROM forum_polls WHERE id = $1")
        .bind(poll_id)
        .fetch_one(db)
        .await?;
    Ok(allow)
}

#[derive(sqlx::FromRow)]
struct PollRow {
    id: i64,
    topic_id: i64,
    question: String,
    max_selections: i32,
    allow_change: bool,
    is_closed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct PollOptionRow {
    id: i64,
    #[sqlx(rename = "poll_id")]
    poll_id_: i64,
    text: String,
    position: i32,
    vote_count: i64,
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_poll_types_serialize_roundtrip() {
        let poll = Poll {
            id: 1,
            topic_id: 1,
            question: "Test?".to_string(),
            max_selections: 1,
            allow_change: false,
            is_closed: false,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };
        let json = serde_json::to_value(&poll).unwrap();
        assert_eq!(json["question"], "Test?");
        assert_eq!(json["max_selections"].as_i64().unwrap(), 1);
    }
}

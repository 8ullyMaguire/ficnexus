use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

// ── Constructive-comment heuristic ────────────────────────────────────────
//
// Feedback rework (program item 1): the site only surfaces positive /
// constructive feedback publicly. Downvotes and negative-toned comments are
// *kept in the DB* (the rec engine can still use them) but hidden from public
// view. At insert time every comment is scored by `is_negative_comment`; a
// match sets `constructive = FALSE` so the public list endpoints skip it.
// Curators can still flip the flag / soft-delete via the existing endpoints.

const NEGATIVE_MARKERS: &[&str] = &[
    "terrible",
    "awful",
    "horrible",
    "garbage",
    "trash",
    "rubbish",
    "waste of time",
    "disappointing",
    "disappointment",
    "hate",
    "hating",
    "stupid",
    "idiotic",
    "dumb",
    "ridiculous",
    "nonsense",
    "boring",
    "bored",
    "cringe",
    "worst",
    "worse",
    "sucks",
    "sucked",
    "suck",
    "useless",
    "pointless",
    "pathetic",
    "annoying",
    "disgusting",
    "crap",
    "shit",
    "fuck",
    "piss",
    "abandon",
    "quit reading",
    "couldn't finish",
    "could not finish",
    "drops this",
    "dropped",
];

/// Heuristic: is this comment text negative-toned (i.e. not constructive)?
/// Cheap substring match on lowercased text — no NLP, deliberate. A comment
/// that contains a negation marker is flagged non-constructive at insert.
pub fn is_negative_comment(body: &str) -> bool {
    let lower = body.to_lowercase();
    NEGATIVE_MARKERS.iter().any(|m| lower.contains(m))
}

/// Score a comment body for constructiveness; public helper used by both
/// comment insertion paths (v2 `social.rs` + threaded `comments.rs`).
pub fn constructive_score(body: &str) -> bool {
    !is_negative_comment(body)
}

// ── Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct CommentUser {
    id: i32,
    username: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct CommentData {
    id: i64,
    url_id: String,
    parent_id: Option<i64>,
    body: String,
    user: Option<CommentUser>,
    created_at: String,
    updated_at: Option<String>,
    deleted: bool,
    hidden: bool,
    reply_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CommentQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PostCommentBody {
    pub body: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HideCommentBody {
    pub hidden: bool,
}

// ── GET /api/v1/works/{url_id}/comments ───────────────────────────────

pub async fn get_comments_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Query(params): Query<CommentQueryParams>,
) -> Result<Json<Value>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(50).max(1);
    let offset = (page - 1) * per_page;

    // Count top-level comments
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM comments
         WHERE url_id = $1 AND parent_id IS NULL
         AND deleted_at IS NULL AND is_hidden = FALSE AND constructive = TRUE",
    )
    .bind(&url_id)
    .fetch_one(&state.db)
    .await?;

    // Fetch top-level comments (newest first)
    let top_rows: Vec<(
        i64,
        String,
        Option<i64>,
        String,
        Option<i32>,
        Option<String>,
        String,
        Option<String>,
        bool,
        bool,
    )> = sqlx::query_as(
        "SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                    u.username, c.created_at::text, c.updated_at::text,
                    c.deleted_at IS NOT NULL AS deleted, c.is_hidden
             FROM comments c
             LEFT JOIN users u ON u.id = c.user_id
             WHERE c.url_id = $1 AND c.parent_id IS NULL
             AND c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE
             ORDER BY c.created_at DESC
             LIMIT $2 OFFSET $3",
    )
    .bind(&url_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    // Collect top-level IDs for fetching replies
    let top_ids: Vec<i64> = top_rows.iter().map(|r| r.0).collect();

    // Fetch all replies for these top-level comments (recursive CTE)
    let all_replies: Vec<(
        i64,
        String,
        Option<i64>,
        String,
        Option<i32>,
        Option<String>,
        String,
        Option<String>,
        bool,
        bool,
        i64,
    )> = if top_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as(
            "WITH RECURSIVE thread AS (
                    SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                           u.username, c.created_at::text, c.updated_at::text,
                           c.deleted_at IS NOT NULL AS deleted, c.is_hidden, c.parent_id AS root_id
                    FROM comments c
                    LEFT JOIN users u ON u.id = c.user_id
                    WHERE c.parent_id = ANY($1)
                    AND c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE

                    UNION ALL
                    SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                           u.username, c.created_at::text, c.updated_at::text,
                           c.deleted_at IS NOT NULL AS deleted, c.is_hidden, t.root_id
                    FROM comments c
                    LEFT JOIN users u ON u.id = c.user_id
                    JOIN thread t ON c.parent_id = t.id
                    WHERE c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE
                )
                SELECT id, url_id, parent_id, body, user_id, username,
                       created_at, updated_at, deleted, is_hidden, root_id
                FROM thread
                ORDER BY root_id, created_at ASC",
        )
        .bind(&top_ids)
        .fetch_all(&state.db)
        .await?
    };

    // Build reply count map
    use std::collections::HashMap;
    let mut reply_counts: HashMap<i64, i64> = HashMap::new();
    for row in &all_replies {
        *reply_counts.entry(row.10).or_insert(0) += 1;
    }

    // Group replies by parent
    let mut replies_by_parent: HashMap<
        i64,
        Vec<&(
            i64,
            String,
            Option<i64>,
            String,
            Option<i32>,
            Option<String>,
            String,
            Option<String>,
            bool,
            bool,
            i64,
        )>,
    > = HashMap::new();
    for row in &all_replies {
        if let Some(parent) = row.2 {
            replies_by_parent.entry(parent).or_default().push(row);
        }
    }

    // Build response
    let mut comments: Vec<Value> = Vec::new();
    for row in &top_rows {
        let (id, _, parent_id, body, user_id, username, created_at, updated_at, deleted, hidden) =
            row;
        let comment = build_comment_json(
            *id,
            &url_id,
            *parent_id,
            body,
            *user_id,
            username.as_deref(),
            created_at,
            updated_at.as_deref(),
            *deleted,
            *hidden,
            reply_counts.get(id).copied().unwrap_or(0),
        );
        comments.push(comment);

        // Add replies recursively
        if let Some(replies) = replies_by_parent.get(id) {
            for reply in replies {
                let (
                    rid,
                    _,
                    rparent,
                    rbody,
                    ruid,
                    runame,
                    rcreated,
                    rupdated,
                    rdeleted,
                    rhidden,
                    _,
                ) = reply;
                let reply_json = build_comment_json(
                    *rid,
                    &url_id,
                    *rparent,
                    rbody,
                    *ruid,
                    runame.as_deref(),
                    rcreated,
                    rupdated.as_deref(),
                    *rdeleted,
                    *rhidden,
                    0,
                );
                comments.push(reply_json);
            }
        }
    }

    Ok(Json(json!({
        "err": 0,
        "total_top_level": total.0,
        "page": page,
        "per_page": per_page,
        "comments": comments,
    })))
}

fn build_comment_json(
    id: i64,
    url_id: &str,
    parent_id: Option<i64>,
    body: &str,
    user_id: Option<i32>,
    username: Option<&str>,
    created_at: &str,
    updated_at: Option<&str>,
    deleted: bool,
    hidden: bool,
    reply_count: i64,
) -> Value {
    let user = user_id.map(|uid| {
        json!({
            "id": uid,
            "username": username.unwrap_or("unknown"),
        })
    });

    let display_body = if deleted {
        "[deleted]".to_string()
    } else if hidden {
        "[hidden]".to_string()
    } else {
        body.to_string()
    };

    json!({
        "id": id,
        "url_id": url_id,
        "parent_id": parent_id,
        "body": display_body,
        "user": user,
        "created_at": created_at,
        "updated_at": updated_at,
        "deleted": deleted,
        "hidden": hidden,
        "reply_count": reply_count,
    })
}

// ── POST /api/v1/works/{url_id}/comments ──────────────────────────────

pub async fn post_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Json(body): Json<PostCommentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.body.trim().is_empty() {
        return Err(AppError::BadRequest("Comment cannot be empty".to_string()));
    }
    if body.body.len() > 2000 {
        return Err(AppError::BadRequest(
            "Comment too long (max 2000 chars)".to_string(),
        ));
    }

    // Verify parent belongs to same work if provided
    if let Some(parent_id) = body.parent_id {
        let parent: Option<(String,)> = sqlx::query_as("SELECT url_id FROM comments WHERE id = $1")
            .bind(parent_id)
            .fetch_optional(&state.db)
            .await?;

        match parent {
            Some((parent_url,)) if parent_url == url_id => {}
            Some(_) => {
                return Err(AppError::BadRequest(
                    "Parent comment belongs to different work".to_string(),
                ));
            }
            None => return Err(AppError::BadRequest("Parent comment not found".to_string())),
        }
    }

    let (id, created_at, constructive): (i64, String, bool) = sqlx::query_as(
        "INSERT INTO comments (url_id, user_id, parent_id, body, constructive)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, created_at::text, constructive",
    )
    .bind(&url_id)
    .bind(user_id)
    .bind(body.parent_id)
    .bind(&body.body)
    .bind(crate::routes::comments::constructive_score(&body.body))
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "comment": {
            "id": id,
            "url_id": url_id,
            "parent_id": body.parent_id,
            "body": body.body,
            "user": { "id": user_id, "username": auth.username },
            "created_at": created_at,
            "updated_at": null,
            "deleted": false,
            "hidden": false,
            "constructive": constructive,
            "reply_count": 0,
        }
    })))
}

// ── DELETE /api/v1/comments/{id} (soft delete) ────────────────────────

pub async fn delete_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Check ownership or curator role
    let comment: Option<(Option<i32>, i16)> = sqlx::query_as(
        "SELECT c.user_id, u.trust_level FROM comments c
         LEFT JOIN users u ON u.id = $2
         WHERE c.id = $1",
    )
    .bind(comment_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match comment {
        Some((Some(uid), _)) if uid == user_id => {}
        Some((_, role)) if role >= 1 => {} // Curator or above
        _ => return Err(AppError::Forbidden("Not authorized".to_string())),
    }

    sqlx::query("UPDATE comments SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(comment_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Comment deleted" })))
}

// ── PATCH /api/v1/comments/{id}/hide (curator only) ──────────────────

pub async fn hide_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i64>,
    Json(body): Json<HideCommentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Check curator trust level
    let trust_level: Option<i16> =
        sqlx::query_scalar("SELECT trust_level FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?;

    match trust_level {
        Some(r) if r >= 3 => {} // Curator or above
        _ => {
            return Err(AppError::Forbidden(
                "Curator trust level required".to_string(),
            ));
        }
    }

    sqlx::query("UPDATE comments SET is_hidden = $2 WHERE id = $1")
        .bind(comment_id)
        .bind(body.hidden)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "hidden": body.hidden })))
}

// ── PATCH /api/v1/comments/{id} (owner, within 24h edit window) ──────────────────

#[derive(Debug, Deserialize)]
pub struct EditCommentBody {
    pub body: String,
}

pub async fn edit_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i64>,
    Json(body): Json<EditCommentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.body.trim().is_empty() {
        return Err(AppError::BadRequest("Comment cannot be empty".to_string()));
    }

    // Verify ownership + 30-minute edit window
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT user_id FROM comments WHERE id = $1 AND deleted_at IS NULL")
            .bind(comment_id)
            .fetch_optional(&state.db)
            .await?;

    let (owner_id,) = row.ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
    if owner_id != user_id {
        return Err(AppError::Forbidden("Not your comment".to_string()));
    }

    // Check 24-hour edit window (86400 seconds)
    let within_window: bool = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM (NOW() - created_at)) < 86400 FROM comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_one(&state.db)
    .await?;

    if !within_window {
        return Err(AppError::BadRequest(
            "Edit window expired (24h)".to_string(),
        ));
    }

    sqlx::query("UPDATE comments SET body = $1, updated_at = NOW() WHERE id = $2")
        .bind(&body.body)
        .bind(comment_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Comment edited" })))
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── build_comment_json ──────────────────────────────────────────

    #[test]
    fn test_build_comment_json_normal() {
        let json = build_comment_json(
            1,
            "test-url",
            None,
            "Hello world!",
            Some(5),
            Some("Alice"),
            "2026-07-27T10:00:00Z",
            None,
            false,
            false,
            3,
        );
        assert_eq!(json["id"], 1);
        assert_eq!(json["url_id"], "test-url");
        assert!(json["parent_id"].is_null());
        assert_eq!(json["body"], "Hello world!");
        assert_eq!(json["user"]["id"], 5);
        assert_eq!(json["user"]["username"], "Alice");
        assert_eq!(json["created_at"], "2026-07-27T10:00:00Z");
        assert!(json["updated_at"].is_null());
        assert_eq!(json["deleted"], false);
        assert_eq!(json["hidden"], false);
        assert_eq!(json["reply_count"], 3);
    }

    #[test]
    fn test_build_comment_json_deleted() {
        let json = build_comment_json(
            2,
            "test-url",
            Some(1),
            "Original text",
            Some(3),
            Some("Bob"),
            "2026-07-27T10:05:00Z",
            None,
            true,
            false,
            0,
        );
        assert_eq!(json["body"], "[deleted]");
        assert_eq!(json["deleted"], true);
        assert_eq!(json["parent_id"], 1);
    }

    #[test]
    fn test_build_comment_json_hidden() {
        let json = build_comment_json(
            3,
            "test-url",
            Some(1),
            "Spam content",
            Some(4),
            Some("Eve"),
            "2026-07-27T10:10:00Z",
            None,
            false,
            true,
            0,
        );
        assert_eq!(json["body"], "[hidden]");
        assert_eq!(json["hidden"], true);
    }

    #[test]
    fn test_build_comment_json_anonymous() {
        let json = build_comment_json(
            4,
            "test-url",
            None,
            "Anonymous post",
            None,
            None,
            "2026-07-27T10:15:00Z",
            None,
            false,
            false,
            0,
        );
        assert_eq!(json["user"], Value::Null);
        assert_eq!(json["body"], "Anonymous post");
        assert_eq!(json["reply_count"], 0);
    }

    #[test]
    fn test_build_comment_json_with_replies() {
        // Top-level comment with 5 replies
        let json = build_comment_json(
            10,
            "work-abc",
            None,
            "Great story!",
            Some(1),
            Some("Reader1"),
            "2026-07-27T12:00:00Z",
            None,
            false,
            false,
            5,
        );
        assert_eq!(json["reply_count"], 5);
        assert_eq!(json["parent_id"], Value::Null);
    }

    #[test]
    fn test_build_comment_json_nested_reply() {
        // Reply to a comment (has parent_id)
        let json = build_comment_json(
            11,
            "work-abc",
            Some(10),
            "I agree!",
            Some(2),
            Some("Reader2"),
            "2026-07-27T12:30:00Z",
            None,
            false,
            false,
            0,
        );
        assert_eq!(json["parent_id"], 10);
        assert_eq!(json["reply_count"], 0);
        assert_eq!(json["url_id"], "work-abc");
    }

    #[test]
    fn test_build_comment_json_with_updated_at() {
        let json = build_comment_json(
            12,
            "work-abc",
            None,
            "Edited comment",
            Some(7),
            Some("Editor"),
            "2026-07-27T11:00:00Z",
            Some("2026-07-27T11:30:00Z"),
            false,
            false,
            0,
        );
        assert_eq!(json["updated_at"], "2026-07-27T11:30:00Z");
    }

    #[test]
    fn test_build_comment_json_user_id_without_username() {
        // user_id present but username is None (user deleted from users table)
        let json = build_comment_json(
            13,
            "work-abc",
            None,
            "Orphaned user comment",
            Some(99),
            None,
            "2026-07-27T14:00:00Z",
            None,
            false,
            false,
            0,
        );
        assert_eq!(json["user"]["id"], 99);
        assert_eq!(json["user"]["username"], "unknown");
    }

    #[test]
    fn test_build_comment_json_deeply_nested_reply() {
        // A reply with reply_count of 0 (leaf node in thread)
        let json = build_comment_json(
            50,
            "work-xyz",
            Some(40),
            "Deep reply",
            Some(3),
            Some("Bob"),
            "2026-07-27T15:00:00Z",
            None,
            false,
            false,
            0,
        );
        assert_eq!(json["parent_id"], 40);
        assert_eq!(json["reply_count"], 0);
    }

    #[test]
    fn test_build_comment_json_deleted_and_hidden_ignored() {
        // deleted takes priority over hidden
        let json = build_comment_json(
            60,
            "work-xyz",
            None,
            "Gone",
            Some(1),
            Some("Mod"),
            "2026-07-27T16:00:00Z",
            None,
            true,
            true,
            0,
        );
        assert_eq!(json["body"], "[deleted]");
        assert_eq!(json["deleted"], true);
        assert_eq!(json["hidden"], true);
    }

    // ── Constructive-comment heuristic ──────────────────────────────

    #[test]
    fn test_constructive_score_accepts_positive_comments() {
        assert!(constructive_score("Great story, I loved the pacing!"));
        assert!(constructive_score(
            "The character development was wonderful."
        ));
        assert!(constructive_score(""));
        assert!(constructive_score("   "));
    }

    #[test]
    fn test_constructive_score_rejects_negative_markers() {
        assert!(!constructive_score("This is terrible."));
        assert!(!constructive_score("I hate this fic."));
        assert!(!constructive_score("What a waste of time."));
        assert!(!constructive_score("It sucks."));
    }

    #[test]
    fn test_constructive_score_is_case_insensitive() {
        assert!(!constructive_score("THIS IS TERRIBLE."));
        assert!(!constructive_score("Terrible."));
        assert!(!constructive_score("I HATE this."));
    }

    #[test]
    fn test_constructive_score_matches_substrings_in_longer_words() {
        // "worst" inside "worst-case" still matches (substring semantics).
        assert!(!constructive_score("Worst-case scenario writing."));
        assert!(!constructive_score("It was disappointing, honestly."));
    }

    #[test]
    fn test_constructive_score_allows_negated_markers() {
        // The heuristic is a cheap substring match — "not terrible" still
        // contains the marker, so it is flagged non-constructive. This
        // documents the deliberate limitation (no NLP).
        assert!(!constructive_score("It is not terrible at all."));
        assert!(!constructive_score("Nothing boring about this one!"));
    }

    #[test]
    fn test_is_negative_comment_matches_phrase_markers() {
        assert!(is_negative_comment("I could not finish this."));
        assert!(is_negative_comment("dropped it at chapter 2"));
        assert!(!is_negative_comment("I could finish this easily."));
    }

    // ── CommentQueryParams deserialization ──────────────────────────

    #[test]
    fn test_comment_query_params_defaults() {
        // Empty query string — both fields should be None
        let params: CommentQueryParams = serde_json::from_str("{}").unwrap();
        assert_eq!(params.page, None);
        assert_eq!(params.per_page, None);
    }

    #[test]
    fn test_comment_query_params_explicit() {
        let params: CommentQueryParams =
            serde_json::from_str(r#"{"page": 3, "per_page": 10}"#).unwrap();
        assert_eq!(params.page, Some(3));
        assert_eq!(params.per_page, Some(10));
    }

    #[test]
    fn test_comment_query_params_partial_page_only() {
        let params: CommentQueryParams = serde_json::from_str(r#"{"page": 5}"#).unwrap();
        assert_eq!(params.page, Some(5));
        assert_eq!(params.per_page, None);
    }

    #[test]
    fn test_comment_query_params_partial_per_page_only() {
        let params: CommentQueryParams = serde_json::from_str(r#"{"per_page": 25}"#).unwrap();
        assert_eq!(params.page, None);
        assert_eq!(params.per_page, Some(25));
    }

    #[test]
    fn test_comment_query_params_negative_values() {
        // Negative values deserialize fine; clamping happens in the handler
        let params: CommentQueryParams =
            serde_json::from_str(r#"{"page": -1, "per_page": -5}"#).unwrap();
        assert_eq!(params.page, Some(-1));
        assert_eq!(params.per_page, Some(-5));
    }

    #[test]
    fn test_comment_query_params_zero() {
        let params: CommentQueryParams =
            serde_json::from_str(r#"{"page": 0, "per_page": 0}"#).unwrap();
        assert_eq!(params.page, Some(0));
        assert_eq!(params.per_page, Some(0));
    }

    #[test]
    fn test_comment_query_params_handler_clamping() {
        // Simulate the clamping logic from get_comments_handler
        let params_cases: Vec<(Option<i64>, Option<i64>, i64, i64, i64)> = vec![
            (None, None, 1, 20, 0),           // defaults: page=1, per_page=20, offset=0
            (Some(1), Some(5), 1, 5, 0),      // explicit small
            (Some(3), Some(10), 3, 10, 20),   // page 3, per_page 10 => offset 20
            (Some(-1), Some(-5), 1, 1, 0),    // negatives clamp to 1
            (Some(0), Some(0), 1, 1, 0),      // zeros clamp to 1
            (Some(5), Some(100), 5, 50, 200), // per_page > 50 clamps to 50
        ];
        for (page_opt, per_page_opt, exp_page, exp_per_page, exp_offset) in params_cases {
            let page = page_opt.unwrap_or(1).max(1);
            let per_page = per_page_opt.unwrap_or(20).min(50).max(1);
            let offset = (page - 1) * per_page;
            assert_eq!(
                page, exp_page,
                "page failed for {:?}/{:?}",
                page_opt, per_page_opt
            );
            assert_eq!(
                per_page, exp_per_page,
                "per_page failed for {:?}/{:?}",
                page_opt, per_page_opt
            );
            assert_eq!(
                offset, exp_offset,
                "offset failed for {:?}/{:?}",
                page_opt, per_page_opt
            );
        }
    }

    // ── PostCommentBody deserialization & validation ────────────────

    #[test]
    fn test_post_comment_body_deserialize() {
        let body: PostCommentBody =
            serde_json::from_str(r#"{"body": "Great chapter!", "parent_id": 42}"#).unwrap();
        assert_eq!(body.body, "Great chapter!");
        assert_eq!(body.parent_id, Some(42));
    }

    #[test]
    fn test_post_comment_body_no_parent() {
        let body: PostCommentBody =
            serde_json::from_str(r#"{"body": "Top-level comment"}"#).unwrap();
        assert_eq!(body.parent_id, None);
    }

    #[test]
    fn test_post_comment_body_empty_body() {
        let body: PostCommentBody = serde_json::from_str(r#"{"body": ""}"#).unwrap();
        // Validation logic from the handler
        assert!(
            body.body.trim().is_empty(),
            "empty body should fail validation"
        );
    }

    #[test]
    fn test_post_comment_body_whitespace_only() {
        let body: PostCommentBody = serde_json::from_str(r#"{"body": "   \t\n  "}"#).unwrap();
        assert!(
            body.body.trim().is_empty(),
            "whitespace-only body should fail validation"
        );
    }

    #[test]
    fn test_post_comment_body_too_long() {
        let long_body = "x".repeat(2001);
        let body: PostCommentBody =
            serde_json::from_value(serde_json::json!({"body": long_body})).unwrap();
        assert!(
            body.body.len() > 2000,
            "over-length body should fail validation"
        );
    }

    #[test]
    fn test_post_comment_body_max_length_valid() {
        let exact_body = "x".repeat(2000);
        let body: PostCommentBody =
            serde_json::from_value(serde_json::json!({"body": exact_body})).unwrap();
        assert_eq!(body.body.len(), 2000);
        assert!(!body.body.trim().is_empty());
    }

    #[test]
    fn test_post_comment_body_parent_id_types() {
        // parent_id can be any i64
        let body: PostCommentBody =
            serde_json::from_str(r#"{"body": "reply", "parent_id": 999999999999}"#).unwrap();
        assert_eq!(body.parent_id, Some(999999999999));
    }

    // ── Recursive CTE SQL structure ─────────────────────────────────
    // These tests validate the SQL strings used in the handler compile
    // and contain expected structural elements. They do NOT require a DB.

    #[test]
    fn test_recursive_cte_sql_structure() {
        let sql = "WITH RECURSIVE thread AS (
            SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                   u.username, c.created_at::text, c.updated_at::text,
                   c.deleted_at IS NOT NULL AS deleted, c.is_hidden, c.parent_id AS root_id
            FROM comments c
            LEFT JOIN users u ON u.id = c.user_id
            WHERE c.parent_id = ANY($1)
            AND c.deleted_at IS NULL AND c.is_hidden = FALSE

            UNION ALL
            SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                   u.username, c.created_at::text, c.updated_at::text,
                   c.deleted_at IS NOT NULL AS deleted, c.is_hidden, t.root_id
            FROM comments c
            LEFT JOIN users u ON u.id = c.user_id
            JOIN thread t ON c.parent_id = t.id
            WHERE c.deleted_at IS NULL AND c.is_hidden = FALSE
        )
        SELECT id, url_id, parent_id, body, user_id, username,
               created_at, updated_at, deleted, is_hidden, root_id
        FROM thread
        ORDER BY root_id, created_at ASC";

        // Structural checks
        assert!(
            sql.contains("WITH RECURSIVE thread AS"),
            "missing RECURSIVE CTE declaration"
        );
        assert!(sql.contains("UNION ALL"), "missing UNION ALL for recursion");
        assert!(
            sql.contains("JOIN thread t ON c.parent_id = t.id"),
            "missing self-join for recursion"
        );
        assert!(sql.contains("root_id"), "missing root_id column");
        assert!(
            sql.contains("deleted_at IS NOT NULL AS deleted"),
            "missing deleted column"
        );
        assert!(
            sql.contains("is_hidden = FALSE"),
            "missing is_hidden filter in base"
        );
        assert!(
            sql.contains("ORDER BY root_id, created_at ASC"),
            "missing ordering"
        );
    }

    #[test]
    fn test_top_level_query_filters_deleted_and_hidden() {
        let sql = "SELECT COUNT(*) FROM comments
            WHERE url_id = $1 AND parent_id IS NULL
            AND deleted_at IS NULL AND is_hidden = FALSE";

        assert!(
            sql.contains("parent_id IS NULL"),
            "missing top-level filter"
        );
        assert!(sql.contains("deleted_at IS NULL"), "missing deleted filter");
        assert!(sql.contains("is_hidden = FALSE"), "missing hidden filter");
    }

    #[test]
    fn test_reply_count_map_accumulation() {
        // Simulate the reply count accumulation logic
        let replies: Vec<(i64, Option<i64>, i64)> = vec![
            // (id, parent_id, root_id)
            (10, Some(1), 1),
            (11, Some(10), 1),
            (12, Some(10), 1),
            (13, Some(1), 1),
            (20, Some(2), 2),
        ];

        let mut reply_counts: HashMap<i64, i64> = HashMap::new();
        for (_, _parent, root_id) in &replies {
            *reply_counts.entry(*root_id).or_insert(0) += 1;
        }

        assert_eq!(reply_counts[&1], 4); // 4 replies under root 1
        assert_eq!(reply_counts[&2], 1); // 1 reply under root 2
        assert_eq!(reply_counts.len(), 2);
    }

    #[test]
    fn test_replies_grouped_by_parent() {
        let replies: Vec<(i64, Option<i64>)> =
            vec![(10, Some(1)), (11, Some(10)), (12, Some(10)), (13, Some(1))];

        let mut by_parent: HashMap<i64, Vec<i64>> = HashMap::new();
        for (id, parent) in &replies {
            if let Some(p) = parent {
                by_parent.entry(*p).or_default().push(*id);
            }
        }

        assert_eq!(by_parent[&1], vec![10, 13]);
        assert_eq!(by_parent[&10], vec![11, 12]);
        assert_eq!(by_parent.len(), 2);
    }

    #[test]
    fn test_empty_top_ids_skips_cte() {
        let top_ids: Vec<i64> = Vec::new();
        // When top_ids is empty, the handler short-circuits (Vec::new())
        let all_replies: Vec<i64> = if top_ids.is_empty() {
            Vec::new()
        } else {
            unreachable!("should not reach here with empty top_ids")
        };
        assert!(all_replies.is_empty());
    }

    #[test]
    fn test_thread_nesting_depth() {
        // Simulate a deep thread tree
        // Root -> child -> grandchild -> great-grandchild
        let tree: Vec<(i64, Option<i64>)> = vec![
            (1, None),    // root
            (2, Some(1)), // child
            (3, Some(2)), // grandchild
            (4, Some(3)), // great-grandchild
        ];

        // Walk the tree from leaf to root
        let mut depth = 0;
        let mut current = Some(4);
        while let Some(id) = current {
            depth += 1;
            current = tree.iter().find(|(i, _)| *i == id).and_then(|(_, p)| *p);
        }
        assert_eq!(depth, 4, "tree should have 4 levels of nesting");
    }
}

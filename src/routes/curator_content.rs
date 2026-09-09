//! Curator content-fix endpoints with peer voting: a curator proposes a
//! replacement body for a fic that gathered wrong content; OTHER curators
//! vote on it; the fix is applied (body blob written at a new version) only
//! when approved by a quorum. A single curator cannot silently replace a
//! fic's content.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Curators needed for a fix to be approved (besides the proposer).
const VOTE_QUORUM: i32 = 2;
/// Minimum net votes (up - down) for approval.
const VOTE_NET_MIN: i32 = 1;

pub(crate) fn require_curator(user: &AuthUser) -> AppResult<i32> {
    let uid = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Curator access required".to_string()));
    }
    Ok(uid)
}

#[derive(Debug, Deserialize)]
pub struct ProposeBody {
    /// Full fic body as HTML (one or more chapters separated by
    /// `<hr class="chapter-break">` — each segment becomes a chapter).
    pub body_html: String,
    /// Why the current body is wrong (shown to other curators).
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    /// 1 = up (approve), -1 = down (reject).
    pub vote: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

/// Split proposed body HTML into chapters on `<hr class="chapter-break">`.
fn split_chapters(body_html: &str) -> Vec<crate::scrape::Chapter> {
    body_html
        .split("<hr class=\"chapter-break\">")
        .enumerate()
        .map(|(i, seg)| crate::scrape::Chapter {
            chapter_id: (i + 1) as i32,
            title: if i == 0 {
                "Chapter 1".to_string()
            } else {
                format!("Chapter {}", i + 1)
            },
            content: seg.trim().to_string(),
        })
        .collect()
}

/// POST /api/curator/content/{url_id}/propose — submit a fix proposal.
pub async fn propose_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Json(body): Json<ProposeBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;

    if body.body_html.trim().is_empty() {
        return Err(AppError::BadRequest(
            "body_html must not be empty".to_string(),
        ));
    }

    let row = sqlx::query(
        "INSERT INTO curator_fix_proposals (url_id, body_html, reason, proposed_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id, status",
    )
    .bind(&url_id)
    .bind(&body.body_html)
    .bind(&body.reason)
    .bind(uid)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to create proposal: {e}")))?;

    let id: i64 = row.get("id");
    let status: String = row.get("status");

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "propose_fix",
        "fic",
        &url_id,
        vec![("proposal_id", serde_json::json!(id))],
    )
    .await;

    Ok(Json(json!({
        "ok": true,
        "proposal_id": id,
        "url_id": url_id,
        "status": status,
        "quorum": VOTE_QUORUM,
    })))
}

/// POST /api/curator/content/proposals/{id}/vote — vote on a pending fix.
/// No self-vote. When quorum is reached: net >= VOTE_NET_MIN → approved +
/// applied (body blob written at next version); net < 0 → rejected.
pub async fn vote_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;
    if body.vote != 1 && body.vote != -1 {
        return Err(AppError::BadRequest("vote must be 1 or -1".to_string()));
    }

    let row = sqlx::query(
        "SELECT url_id, body_html, proposed_by, status FROM curator_fix_proposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to load proposal: {e}")))?;

    let Some(row) = row else {
        return Err(AppError::NotFound(format!("proposal {id} not found")));
    };
    let url_id: String = row.get("url_id");
    let body_html: String = row.get("body_html");
    let proposed_by: i32 = row.get("proposed_by");
    let status: String = row.get("status");

    if status != "pending" {
        return Err(AppError::BadRequest(format!("proposal already {status}")));
    }
    if proposed_by == uid {
        return Err(AppError::BadRequest(
            "cannot vote on your own proposal".to_string(),
        ));
    }

    // Upsert the vote (one per curator).
    sqlx::query(
        "INSERT INTO curator_fix_votes (proposal_id, user_id, vote)
         VALUES ($1, $2, $3)
         ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = EXCLUDED.vote",
    )
    .bind(id)
    .bind(uid)
    .bind(body.vote)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to record vote: {e}")))?;

    let counts = sqlx::query(
        "SELECT
           COALESCE(SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END), 0)::int AS up,
           COALESCE(SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END), 0)::int AS down
         FROM curator_fix_votes WHERE proposal_id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to count votes: {e}")))?;

    let up: i32 = counts.get("up");
    let down: i32 = counts.get("down");
    let total = up + down;
    let net = up - down;

    let mut new_status = "pending".to_string();
    let mut applied = false;

    if total >= VOTE_QUORUM {
        if net >= VOTE_NET_MIN {
            // Apply: write the proposed body at the next version.
            let version = crate::body_cache::bump_version(&state.config, &url_id);
            let chapters = split_chapters(&body_html);
            crate::body_cache::save_body(
                &state.config,
                &url_id,
                &chapters,
                Some(format!("curator-fix-proposal-{id}")),
                version,
            )
            .map_err(|e| AppError::BadRequest(format!("failed to apply body: {e}")))?;
            // Keep the body FTS index fresh — the curator just overwrote
            // the body blob. Best-effort (index_body_text swallows errors).
            crate::body_cache::index_body_text(&state.db, &state.config, &url_id).await;
            applied = true;
            new_status = "applied".to_string();
        } else if net < 0 {
            new_status = "rejected".to_string();
        }
    }

    sqlx::query(
        "UPDATE curator_fix_proposals
         SET status = $2, upvotes = $3, downvotes = $4, decided_at = CASE WHEN $2 = 'pending' THEN NULL ELSE now() END, decided_by = CASE WHEN $2 = 'pending' THEN NULL ELSE $5 END
         WHERE id = $1",
    )
    .bind(id)
    .bind(&new_status)
    .bind(up)
    .bind(down)
    .bind(uid)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to update proposal: {e}")))?;

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "vote_fix",
        "fic",
        &url_id,
        vec![
            ("proposal_id", serde_json::json!(id)),
            ("vote", serde_json::json!(body.vote)),
            ("status", serde_json::json!(new_status)),
            ("applied", serde_json::json!(applied)),
        ],
    )
    .await;

    Ok(Json(json!({
        "ok": true,
        "proposal_id": id,
        "url_id": url_id,
        "upvotes": up,
        "downvotes": down,
        "total": total,
        "net": net,
        "status": new_status,
        "applied": applied,
        "quorum": VOTE_QUORUM,
    })))
}

/// GET /api/curator/content/proposals?status=pending — list fix proposals.
pub async fn list_proposals(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;
    let status = q.status.unwrap_or_else(|| "pending".to_string());

    let rows = sqlx::query(
        "SELECT p.id, p.url_id, p.reason, p.proposed_by, p.status,
                p.upvotes, p.downvotes, p.created_at, u.username AS proposer
         FROM curator_fix_proposals p
         LEFT JOIN users u ON u.id = p.proposed_by
         WHERE p.status = $1
         ORDER BY p.created_at DESC
         LIMIT 100",
    )
    .bind(&status)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to list proposals: {e}")))?;

    let proposals: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<i64, _>("id"),
                "url_id": r.get::<String, _>("url_id"),
                "reason": r.get::<String, _>("reason"),
                "proposed_by": r.get::<i32, _>("proposed_by"),
                "proposer": r.get::<Option<String>, _>("proposer"),
                "status": r.get::<String, _>("status"),
                "upvotes": r.get::<i32, _>("upvotes"),
                "downvotes": r.get::<i32, _>("downvotes"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "ok": true, "proposals": proposals })))
}

/// GET /api/curator/content/{url_id} — inspect the cached body blob
/// (chapter count + preview) so curators can see what was gathered.
pub async fn get_body(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;

    let Some(chapters) = crate::body_cache::load_body(&state.config, &url_id) else {
        return Ok(Json(json!({
            "ok": true,
            "url_id": url_id,
            "cached": false,
        })));
    };

    let previews: Vec<String> = chapters
        .iter()
        .map(|c| {
            let text = c.content.chars().take(200).collect::<String>();
            format!("[{}] {}…", c.title, text)
        })
        .collect();

    Ok(Json(json!({
        "ok": true,
        "url_id": url_id,
        "cached": true,
        "chapters": chapters.len(),
        "previews": previews,
    })))
}

/// DELETE /api/curator/content/{url_id} — remove the cached body blob so the
/// next export re-scrapes from the source. Direct (no vote): it only forces
/// a re-scrape, it does not replace content.
pub async fn delete_body(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;

    crate::body_cache::delete_body(&state.config, &url_id)
        .map_err(|e| AppError::BadRequest(format!("failed to delete body: {e}")))?;

    crate::modlog::record(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "delete_body",
        "fic",
        &url_id,
        serde_json::json!({}),
    )
    .await;

    Ok(Json(json!({
        "ok": true,
        "url_id": url_id,
        "deleted": true,
    })))
}

// ═════════════════════════════════════════════════════════════════════
// Metadata fix proposals (curator/mod/admin edits fic metadata → vote)
// ═════════════════════════════════════════════════════════════════════
// Same peer-vote pattern as body fixes (032): a curator proposes a
// metadata change from the fic page; OTHER curators vote; applied to the
// work + default source only when a quorum approves.

#[derive(Deserialize)]
pub struct MetadataProposalBody {
    pub work_id: i32,
    #[serde(default)]
    pub url_id: String,
    pub field: String, // title | author | status | description
    #[serde(default)]
    pub old_value: String,
    pub new_value: String,
    #[serde(default)]
    pub reason: String,
}

/// POST /api/curator/metadata/propose — propose a metadata change.
pub async fn propose_metadata_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<MetadataProposalBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;
    if !matches!(
        body.field.as_str(),
        "title" | "author" | "status" | "description"
    ) {
        return Err(AppError::BadRequest(
            "field must be title|author|status|description".to_string(),
        ));
    }
    let new_value = body.new_value.trim();
    if new_value.is_empty() {
        return Err(AppError::BadRequest("new_value required".to_string()));
    }

    // Verify the work exists + snapshot the current value.
    let current: Option<String> = sqlx::query("SELECT canonical_title FROM works WHERE id = $1")
        .bind(body.work_id)
        .fetch_optional(&state.db)
        .await?
        .map(|r| r.get("canonical_title"));

    let Some(_current) = current else {
        return Err(AppError::NotFound(format!(
            "work {} not found",
            body.work_id
        )));
    };

    let id: i64 = sqlx::query(
        r#"INSERT INTO curator_metadata_proposals
           (work_id, url_id, field, old_value, new_value, reason, proposed_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(body.work_id)
    .bind(&body.url_id)
    .bind(&body.field)
    .bind(&body.old_value)
    .bind(new_value)
    .bind(&body.reason)
    .bind(uid)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to create proposal: {e}")))?
    .get("id");

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "propose_metadata_fix",
        "work",
        &body.work_id.to_string(),
        vec![
            ("proposal_id", json!(id)),
            ("field", json!(body.field)),
            ("new_value", json!(new_value)),
        ],
    )
    .await;

    Ok(Json(
        json!({ "err": 0, "proposal_id": id, "status": "pending", "msg": "Metadata change proposed for curator vote" }),
    ))
}

/// POST /api/curator/metadata/proposals/{id}/vote — vote on a metadata fix.
pub async fn vote_metadata_fix(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> AppResult<Json<Value>> {
    let uid = require_curator(&auth)?;
    if body.vote != 1 && body.vote != -1 {
        return Err(AppError::BadRequest("vote must be 1 or -1".to_string()));
    }

    let row = sqlx::query(
        "SELECT work_id, field, new_value, proposed_by, status FROM curator_metadata_proposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("proposal {id} not found")))?;

    let work_id: i32 = row.get("work_id");
    let field: String = row.get("field");
    let new_value: String = row.get("new_value");
    let proposed_by: i32 = row.get("proposed_by");
    let status: String = row.get("status");

    if status != "pending" {
        return Err(AppError::BadRequest(format!("proposal already {status}")));
    }
    if proposed_by == uid {
        return Err(AppError::BadRequest(
            "cannot vote on your own proposal".to_string(),
        ));
    }

    sqlx::query(
        "INSERT INTO curator_metadata_votes (proposal_id, user_id, vote)
         VALUES ($1, $2, $3)
         ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = EXCLUDED.vote",
    )
    .bind(id)
    .bind(uid)
    .bind(body.vote)
    .execute(&state.db)
    .await?;

    let counts = sqlx::query(
        "SELECT
           COALESCE(SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END), 0)::int AS up,
           COALESCE(SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END), 0)::int AS down
         FROM curator_metadata_votes WHERE proposal_id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    let up: i32 = counts.get("up");
    let down: i32 = counts.get("down");
    let total = up + down;
    let net = up - down;

    let mut new_status = "pending".to_string();
    let mut applied = false;

    if total >= VOTE_QUORUM {
        if net >= VOTE_NET_MIN {
            // Apply. title/author/description live on the canonical
            // works row; status lives on fic_info (the fic's own
            // per-source status), updated via the default source.
            let sql = match field.as_str() {
                "title" => {
                    "UPDATE works SET canonical_title = $2, updated_at = NOW() WHERE id = $1"
                }
                "author" => {
                    "UPDATE works SET canonical_author = $2, updated_at = NOW() WHERE id = $1"
                }
                "description" => {
                    "UPDATE works SET description = $2, updated_at = NOW() WHERE id = $1"
                }
                _ => "",
            };
            if field == "status" {
                sqlx::query(
                    r#"UPDATE fic_info SET status = $2, updated = NOW()
                       WHERE id = (SELECT default_source_id FROM works WHERE id = $1)"#,
                )
                .bind(work_id)
                .bind(&new_value)
                .execute(&state.db)
                .await?;
            } else {
                sqlx::query(sql)
                    .bind(work_id)
                    .bind(&new_value)
                    .execute(&state.db)
                    .await?;
            }
            applied = true;
            new_status = "applied".to_string();
        } else if net < 0 {
            new_status = "rejected".to_string();
        }
    }

    sqlx::query(
        "UPDATE curator_metadata_proposals
         SET status = $2, upvotes = $3, downvotes = $4,
             decided_at = CASE WHEN $2 = 'pending' THEN NULL ELSE now() END,
             decided_by = CASE WHEN $2 = 'pending' THEN NULL ELSE $5 END
         WHERE id = $1",
    )
    .bind(id)
    .bind(&new_status)
    .bind(up)
    .bind(down)
    .bind(uid)
    .execute(&state.db)
    .await?;

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "vote_metadata_fix",
        "work",
        &work_id.to_string(),
        vec![
            ("proposal_id", json!(id)),
            ("applied", json!(applied)),
            ("status", json!(new_status)),
        ],
    )
    .await;

    Ok(Json(json!({
        "err": 0, "proposal_id": id, "status": new_status,
        "upvotes": up, "downvotes": down, "applied": applied,
        "msg": if applied { "Metadata change approved and applied" } else { "Vote recorded" },
    })))
}

/// GET /api/curator/metadata/proposals?status=pending — list proposals.
pub async fn list_metadata_proposals(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListProposalsParams>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;
    let status = params.status.unwrap_or_else(|| "pending".to_string());

    let rows: Vec<(
        i64,
        i32,
        String,
        String,
        String,
        String,
        String,
        i32,
        String,
        i32,
        i32,
    )> = sqlx::query_as(
        r#"SELECT p.id, p.work_id, w.canonical_title, p.field, p.old_value, p.new_value,
                   p.reason, p.proposed_by, u.username, p.upvotes, p.downvotes
            FROM curator_metadata_proposals p
            JOIN works w ON w.id = p.work_id
            JOIN users u ON u.id = p.proposed_by
            WHERE p.status = $1
            ORDER BY p.created_at DESC
            LIMIT 100"#,
    )
    .bind(&status)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                work_id,
                work_title,
                field,
                old_value,
                new_value,
                reason,
                proposed_by,
                username,
                upvotes,
                downvotes,
            )| {
                json!({
                    "id": id, "work_id": work_id, "work_title": work_title,
                    "field": field, "old_value": old_value, "new_value": new_value,
                    "reason": reason, "proposed_by": proposed_by, "username": username,
                    "upvotes": upvotes, "downvotes": downvotes,
                })
            },
        )
        .collect();

    Ok(Json(
        json!({ "ok": true, "status": status, "proposals": items }),
    ))
}

#[derive(Deserialize)]
pub struct ListProposalsParams {
    #[serde(default)]
    pub status: Option<String>,
}

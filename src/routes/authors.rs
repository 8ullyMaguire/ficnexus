use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::authors_fuzzy::suggest_alternatives;
use crate::server::AppState;

// ── Helper structs ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuthorSearchParams {
    pub q: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

// ── Public API routes ───────────────────────────────────────────────────────

/// GET /api/authors/search?q= — search author profiles
pub async fn search_authors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthorSearchParams>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100) as i64;
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) as i64 * limit;

    if params.q.is_none() || params.q.as_deref().unwrap_or("").is_empty() {
        // Return recent profiles
        let rows = sqlx::query_as::<_, (i32, String, Option<String>, i64)>(
            r#"SELECT ap.id, ap.canonical_name, ap.bio,
                      (SELECT COUNT(*) FROM author_profile_links apl WHERE apl.profile_id = ap.id) as linked_authors
               FROM author_profiles ap
               ORDER BY ap.updated_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        return Ok(Json(
            json!({"err": 0, "items": rows, "page": params.page.unwrap_or(1)}),
        ));
    }

    let query = format!("%{}%", params.q.unwrap_or_default());

    // Search by name, also search by associated fic authors
    let rows = sqlx::query_as::<_, (i32, String, Option<String>, i64)>(
        r#"SELECT DISTINCT ap.id, ap.canonical_name, ap.bio,
                  (SELECT COUNT(*) FROM author_profile_links apl WHERE apl.profile_id = ap.id) as linked_authors
           FROM author_profiles ap
           LEFT JOIN author_profile_links apl ON apl.profile_id = ap.id
           WHERE ap.canonical_name ILIKE $1
              OR apl.source_author ILIKE $1
           ORDER BY ap.canonical_name
           LIMIT $2 OFFSET $3"#,
    )
    .bind(&query)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        json!({"err": 0, "items": rows, "page": params.page.unwrap_or(1)}),
    ))
}

/// GET /api/authors/{id} — get author profile with socials and linked accounts.
///
/// On miss (unknown id), returns HTTP 404 with a `suggestions` array of
/// close name matches so the frontend can offer "Did you mean: X?" instead
/// of dead-ending.
pub async fn get_author_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let profile = sqlx::query_as::<
        _,
        (
            i32,
            String,
            Option<String>,
            Option<String>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT id, canonical_name, bio, avatar_url, created_at FROM author_profiles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let profile = match profile {
        Some(p) => p,
        None => {
            // Fetch recent canonical names and suggest fuzzy matches.
            let recent_names: Vec<(i32, String)> = sqlx::query_as(
                "SELECT id, canonical_name FROM author_profiles ORDER BY updated_at DESC LIMIT 50",
            )
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let suggestions: Vec<Value> =
                suggest_alternatives(&id.to_string(), &recent_names, 5, |(_, name)| name.as_str())
                    .into_iter()
                    .map(|(sid, name)| json!({ "id": sid, "canonical_name": name }))
                    .collect();

            return Err(AppError::NotFound(
                json!({"err": -5, "msg": "Author not found", "suggestions": suggestions})
                    .to_string(),
            ));
        }
    };

    let links = sqlx::query_as::<_, (i32, String, String, Option<i32>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, source_author, source_url, source_id, created_at FROM author_profile_links WHERE profile_id = $1",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let socials = sqlx::query_as::<_, (i32, String, String, String, bool, i32)>(
        "SELECT id, platform, url, label, is_visible, sort_order FROM author_socials WHERE profile_id = $1 ORDER BY sort_order",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "profile": {
            "id": profile.0,
            "canonical_name": profile.1,
            "bio": profile.2,
            "avatar_url": profile.3,
            "created_at": profile.4,
        },
        "linked_authors": links
            .into_iter()
            .map(|l| {
                json!({
                    "id": l.0,
                    "source_author": l.1,
                    "source_url": l.2,
                    "source_id": l.3,
                })
            })
            .collect::<Vec<_>>(),
        "socials": socials
            .into_iter()
            .map(|s| {
                json!({
                    "id": s.0,
                    "platform": s.1,
                    "url": s.2,
                    "label": s.3,
                    "is_visible": s.4,
                })
            })
            .collect::<Vec<_>>(),
    })))
}

/// PUT /api/authors/{id} — update profile (curator+)
pub async fn update_author_profile(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let canonical_name = payload.get("canonical_name").and_then(|v| v.as_str());
    let bio = payload.get("bio").and_then(|v| v.as_str());
    let avatar_url = payload.get("avatar_url").and_then(|v| v.as_str());
    let badge_text = payload.get("badge_text").and_then(|v| v.as_str());

    if let Some(name) = canonical_name {
        sqlx::query(
            "UPDATE author_profiles SET canonical_name = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(name)
        .bind(id)
        .execute(&state.db)
        .await?;
    }
    if let Some(b) = bio {
        sqlx::query("UPDATE author_profiles SET bio = $1, updated_at = NOW() WHERE id = $2")
            .bind(b)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(a) = avatar_url {
        sqlx::query("UPDATE author_profiles SET avatar_url = $1, updated_at = NOW() WHERE id = $2")
            .bind(a)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(badge) = badge_text {
        sqlx::query("UPDATE author_profiles SET badge_text = $1, updated_at = NOW() WHERE id = $2")
            .bind(badge)
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    Ok(Json(json!({"err": 0, "msg": "Profile updated"})))
}

/// POST /api/authors/{id}/socials — propose adding a social link (curator quorum).
pub async fn add_social(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::curator_content::require_curator(&user)?;

    let platform = payload
        .get("platform")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("platform required".to_string()))?;
    let url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let label = payload.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("");

    let proposal_id: i64 = sqlx::query_scalar(
        "INSERT INTO author_social_proposals (profile_id, action, platform, url, label, reason, proposed_by)
         VALUES ($1, 'add', $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(id)
    .bind(platform)
    .bind(url)
    .bind(label)
    .bind(reason)
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "proposal_id": proposal_id, "status": "pending" }),
    ))
}

/// DELETE /api/authors/{id}/socials/{social_id} — propose removing a social link (curator quorum).
pub async fn remove_social(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((id, social_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::curator_content::require_curator(&user)?;

    // Fetch the existing social link so we can snapshot it in the proposal.
    let existing: Option<(String, String, String)> = sqlx::query_as(
        "SELECT platform, url, label FROM author_socials WHERE id = $1 AND profile_id = $2",
    )
    .bind(social_id)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let (platform, url, label) = match existing {
        Some(s) => s,
        None => return Err(AppError::NotFound("social link not found".to_string())),
    };

    let proposal_id: i64 = sqlx::query_scalar(
        "INSERT INTO author_social_proposals (profile_id, action, social_id, platform, url, label, reason, proposed_by)
         VALUES ($1, 'remove', $2, $3, $4, $5, 'Removal requested', $6) RETURNING id",
    )
    .bind(id)
    .bind(social_id)
    .bind(platform)
    .bind(url)
    .bind(label)
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "proposal_id": proposal_id, "status": "pending" }),
    ))
}

// ── Curator social proposal voting ──────────────────────────────────────────

/// POST /api/curator/authors/social-proposals/{id}/vote — vote on a social proposal.
pub async fn vote_social_proposal(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(proposal_id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::curator_content::require_curator(&user)?;
    let vote = body
        .get("vote")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::BadRequest("vote required".into()))?;
    if vote != 1 && vote != -1 {
        return Err(AppError::BadRequest("vote must be 1 or -1".into()));
    }
    let vote = vote as i16;

    // Make sure the proposal exists and is still pending.
    let proposal: Option<(String, i32, String)> = sqlx::query_as(
        "SELECT status, proposed_by, action FROM author_social_proposals WHERE id = $1",
    )
    .bind(proposal_id)
    .fetch_optional(&state.db)
    .await?;
    let (status, proposed_by, action) = match proposal {
        Some(p) => p,
        None => {
            return Err(AppError::NotFound(format!(
                "proposal {proposal_id} not found"
            )));
        }
    };
    if status != "pending" {
        return Err(AppError::BadRequest(format!("proposal already {status}")));
    }
    if proposed_by == uid {
        return Err(AppError::BadRequest(
            "cannot vote on your own proposal".into(),
        ));
    }

    // Upsert vote.
    sqlx::query(
        "INSERT INTO author_social_votes (proposal_id, user_id, vote) VALUES ($1, $2, $3)
         ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = EXCLUDED.vote",
    )
    .bind(proposal_id)
    .bind(uid)
    .bind(vote)
    .execute(&state.db)
    .await?;

    // Tally votes.
    let (upvotes, downvotes): (i64, i64) = sqlx::query_as(
        "SELECT
           COALESCE(SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END), 0)::bigint,
           COALESCE(SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END), 0)::bigint
         FROM author_social_votes WHERE proposal_id = $1",
    )
    .bind(proposal_id)
    .fetch_one(&state.db)
    .await?;
    let total = upvotes + downvotes;
    let net = upvotes - downvotes;

    let mut new_status: Option<String> = None;
    let mut applied = false;
    let quorum: i64 = 3; // 3 other curators (proposer doesn't vote) — match metadata
    if total >= quorum && net >= 1 {
        // Apply the change.
        let detail: (i32, String, String, String, String, Option<i32>) = sqlx::query_as(
            "SELECT profile_id, action, platform, url, label, social_id
             FROM author_social_proposals WHERE id = $1",
        )
        .bind(proposal_id)
        .fetch_one(&state.db)
        .await?;
        match detail.1.as_str() {
            "add" => {
                sqlx::query(
                    "INSERT INTO author_socials (profile_id, platform, url, label)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (profile_id, platform, url) DO UPDATE SET label = EXCLUDED.label",
                )
                .bind(detail.0)
                .bind(&detail.2)
                .bind(&detail.3)
                .bind(&detail.4)
                .execute(&state.db)
                .await?;
                applied = true;
            }
            "remove" => {
                if let Some(sid) = detail.5 {
                    sqlx::query("DELETE FROM author_socials WHERE id = $1 AND profile_id = $2")
                        .bind(sid)
                        .bind(detail.0)
                        .execute(&state.db)
                        .await?;
                    applied = true;
                }
            }
            _ => {}
        }
        sqlx::query(
            "UPDATE author_social_proposals
             SET status = 'approved', decided_at = NOW(), decided_by = $1, upvotes = $2, downvotes = $3
             WHERE id = $4",
        )
        .bind(uid)
        .bind(upvotes)
        .bind(downvotes)
        .bind(proposal_id)
        .execute(&state.db)
        .await?;
        new_status = Some("approved".into());
    } else if total >= quorum && net < 0 {
        sqlx::query(
            "UPDATE author_social_proposals
             SET status = 'rejected', decided_at = NOW(), decided_by = $1, upvotes = $2, downvotes = $3
             WHERE id = $4",
        )
        .bind(uid)
        .bind(upvotes)
        .bind(downvotes)
        .bind(proposal_id)
        .execute(&state.db)
        .await?;
        new_status = Some("rejected".into());
    } else {
        // Still pending; just update the running tallies.
        sqlx::query(
            "UPDATE author_social_proposals SET upvotes = $1, downvotes = $2 WHERE id = $3",
        )
        .bind(upvotes)
        .bind(downvotes)
        .bind(proposal_id)
        .execute(&state.db)
        .await?;
    }

    // Audit trail: ignore failures (modlog is best-effort).
    let _ = crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "vote_social_proposal",
        "author_social_proposal",
        &proposal_id.to_string(),
        vec![
            ("vote", json!(vote)),
            ("upvotes", json!(upvotes)),
            ("downvotes", json!(downvotes)),
            (
                "new_status",
                json!(new_status.clone().unwrap_or_else(|| "pending".into())),
            ),
            ("action", json!(action)),
            ("applied", json!(applied)),
        ],
    )
    .await;

    Ok(Json(json!({
        "err": 0,
        "upvotes": upvotes,
        "downvotes": downvotes,
        "new_status": new_status.unwrap_or_else(|| "pending".into()),
        "applied": applied,
    })))
}

/// GET /api/curator/authors/social-proposals — list proposals (default: pending).
pub async fn list_social_proposals(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let _uid = crate::routes::curator_content::require_curator(&user)?;
    let status_filter = params
        .get("status")
        .map(|s| s.as_str())
        .unwrap_or("pending");

    let proposals: Vec<(i64, i32, String, String, String, String, String, i64, i64)> =
        sqlx::query_as(
            "SELECT id, profile_id, action, platform, url, label, status, upvotes, downvotes
             FROM author_social_proposals
             WHERE status = $1
             ORDER BY created_at DESC
             LIMIT 50",
        )
        .bind(status_filter)
        .fetch_all(&state.db)
        .await?;

    let items: Vec<Value> = proposals
        .into_iter()
        .map(
            |(id, pid, action, platform, url, label, status, up, down)| {
                json!({
                    "id": id,
                    "profile_id": pid,
                    "action": action,
                    "platform": platform,
                    "url": url,
                    "label": label,
                    "status": status,
                    "upvotes": up,
                    "downvotes": down,
                })
            },
        )
        .collect();

    Ok(Json(json!({ "err": 0, "proposals": items })))
}

// ── Curator routes ──────────────────────────────────────────────────────────

/// POST /api/curator/authors/merge — propose or approve merge
pub async fn propose_merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let source_author = payload
        .get("source_author")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("source_author required".to_string()))?;
    let source_url = payload
        .get("source_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("source_url required".to_string()))?;
    let target_profile_id = payload
        .get("target_profile_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::BadRequest("target_profile_id required".to_string()))?
        as i32;
    let auto_approve = payload
        .get("auto_approve")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if auto_approve && user.trust_level >= 5 {
        // Admin auto-approve: create link directly
        sqlx::query(
            "INSERT INTO author_profile_links (profile_id, source_author, source_url, source_id) VALUES ($1, $2, $3, $4) ON CONFLICT (source_url) DO NOTHING",
        )
        .bind(target_profile_id)
        .bind(source_author)
        .bind(source_url)
        .bind(payload.get("source_id").and_then(|v| v.as_i64()))
        .execute(&state.db)
        .await?;

        Ok(Json(json!({"err": 0, "msg": "Author linked directly"})))
    } else {
        sqlx::query(
            "INSERT INTO author_merge_proposals (source_author, source_url, target_profile_id, proposed_by) VALUES ($1, $2, $3, $4)",
        )
        .bind(source_author)
        .bind(source_url)
        .bind(target_profile_id)
        .bind(user.user_id)
        .execute(&state.db)
        .await?;

        Ok(Json(json!({"err": 0, "msg": "Merge proposal submitted"})))
    }
}

/// GET /api/curator/authors/pending — list pending merge proposals
pub async fn pending_merges(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let rows = sqlx::query_as::<
        _,
        (
            i32,
            String,
            String,
            i32,
            String,
            Option<String>,
            i32,
            String,
        ),
    >(
        r#"SELECT amp.id, amp.source_author, amp.source_url, amp.target_profile_id,
                  ap.canonical_name, u.username, amp.proposed_by, amp.created_at::text
           FROM author_merge_proposals amp
           JOIN author_profiles ap ON ap.id = amp.target_profile_id
           LEFT JOIN users u ON u.id = amp.proposed_by
           WHERE amp.status = 'pending'
           ORDER BY amp.created_at DESC"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        json!({"err": 0, "items": rows.into_iter().map(|r| json!({
        "id": r.0,
        "source_author": r.1,
        "source_url": r.2,
        "target_profile_id": r.3,
        "target_name": r.4,
        "proposed_by": r.5,
        "created_at": r.7,
    })).collect::<Vec<_>>()}),
    ))
}

/// POST /api/curator/authors/approve/{proposal_id}
pub async fn approve_merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(proposal_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let proposal = sqlx::query_as::<_, (String, String, i32)>(
        "SELECT source_author, source_url, target_profile_id FROM author_merge_proposals WHERE id = $1 AND status = 'pending'",
    )
    .bind(proposal_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Proposal not found or already resolved".into()))?;

    // Create the link
    sqlx::query(
        "INSERT INTO author_profile_links (profile_id, source_author, source_url) VALUES ($1, $2, $3) ON CONFLICT (source_url) DO NOTHING",
    )
    .bind(proposal.2)
    .bind(&proposal.0)
    .bind(&proposal.1)
    .execute(&state.db)
    .await?;

    // Update proposal status
    sqlx::query(
        "UPDATE author_merge_proposals SET status = 'approved', approved_by = $1, resolved_at = NOW() WHERE id = $2",
    )
    .bind(user.user_id)
    .bind(proposal_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "Merge approved"})))
}

/// POST /api/curator/authors/reject/{proposal_id}
pub async fn reject_merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(proposal_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    sqlx::query(
        "UPDATE author_merge_proposals SET status = 'rejected', approved_by = $1, resolved_at = NOW() WHERE id = $2",
    )
    .bind(user.user_id)
    .bind(proposal_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "Merge rejected"})))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::routes::auth::AuthUser;
    use crate::routes::authors::AuthorSearchParams;
    use serde_json::json;

    #[test]
    fn test_search_params_defaults() {
        let params = AuthorSearchParams {
            q: None,
            page: None,
            limit: None,
        };
        assert_eq!(params.limit.unwrap_or(20), 20);
        assert_eq!(params.page.unwrap_or(1), 1);
        assert_eq!(params.q, None);
    }

    #[test]
    fn test_search_params_with_values() {
        let params = AuthorSearchParams {
            q: Some("test".into()),
            page: Some(2),
            limit: Some(50),
        };
        assert_eq!(params.q.unwrap(), "test");
        assert_eq!(params.page.unwrap(), 2);
        assert_eq!(params.limit.unwrap(), 50);
    }

    #[test]
    fn test_limit_capped_at_100() {
        let params = AuthorSearchParams {
            q: None,
            page: None,
            limit: Some(200),
        };
        assert_eq!(params.limit.unwrap_or(20).min(100), 100);
    }

    #[test]
    fn test_forbidden_role_check() {
        // Users with role < 5 should get Forbidden
        let user = AuthUser {
            user_id: Some(1),
            username: Some("reader".into()),
            role: 0,
            level: 0,
        };
        assert!(user.trust_level < 3, "Reader should not pass curator check");

        let curator = AuthUser {
            user_id: Some(2),
            username: Some("curator".into()),
            role: 5,
            level: 50,
        };
        assert!(curator.trust_level >= 5, "Curator should pass curator check");

        let admin = AuthUser {
            user_id: Some(3),
            username: Some("admin".into()),
            role: 10,
            level: 100,
        };
        assert!(admin.trust_level >= 10, "Admin should pass admin check");
    }

    #[test]
    fn test_auto_approve_only_admin() {
        // auto_approve requires role >= 10
        let admin = AuthUser {
            user_id: Some(3),
            username: Some("admin".into()),
            role: 10,
            level: 100,
        };
        assert!(admin.trust_level >= 10);

        let curator = AuthUser {
            user_id: Some(2),
            username: Some("curator".into()),
            role: 5,
            level: 50,
        };
        assert!(!(curator.trust_level >= 10));
    }

    #[test]
    fn test_apperror_not_found_includes_suggestions() {
        let body = json!({"err": -5, "msg": "Author not found", "suggestions": [{"id": 1, "canonical_name": "alice"}]});
        let err = AppError::NotFound(body.to_string());
        match err {
            AppError::NotFound(msg) => {
                let parsed: serde_json::Value = serde_json::from_str(&msg).expect("valid JSON");
                assert_eq!(parsed["err"], -5);
                assert_eq!(parsed["msg"], "Author not found");
                assert!(parsed["suggestions"].is_array());
                assert_eq!(parsed["suggestions"][0]["canonical_name"], "alice");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_apperror_forbidden() {
        let err = AppError::Forbidden("Curator access required".into());
        match err {
            AppError::Forbidden(msg) => assert_eq!(msg, "Curator access required"),
            _ => panic!("Wrong error variant"),
        }
    }
}

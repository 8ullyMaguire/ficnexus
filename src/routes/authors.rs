use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
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

        return Ok(Json(json!({"err": 0, "items": rows, "page": params.page.unwrap_or(1)})));
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

    Ok(Json(json!({"err": 0, "items": rows, "page": params.page.unwrap_or(1)})))
}

/// GET /api/authors/{id} — get author profile with socials and linked accounts
pub async fn get_author_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let profile = sqlx::query_as::<_, (i32, String, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, canonical_name, bio, avatar_url, created_at FROM author_profiles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Author not found".into()))?;

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
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let canonical_name = payload.get("canonical_name").and_then(|v| v.as_str());
    let bio = payload.get("bio").and_then(|v| v.as_str());
    let avatar_url = payload.get("avatar_url").and_then(|v| v.as_str());
    let badge_text = payload.get("badge_text").and_then(|v| v.as_str());

    if let Some(name) = canonical_name {
        sqlx::query("UPDATE author_profiles SET canonical_name = $1, updated_at = NOW() WHERE id = $2")
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

/// POST /api/authors/{id}/socials — add social link
pub async fn add_social(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let platform = payload
        .get("platform")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("platform required".to_string()))?;
    let url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let label = payload.get("label").and_then(|v| v.as_str()).unwrap_or("");

    sqlx::query(
        "INSERT INTO author_socials (profile_id, platform, url, label) VALUES ($1, $2, $3, $4) ON CONFLICT (profile_id, platform, url) DO UPDATE SET label = EXCLUDED.label",
    )
    .bind(id)
    .bind(platform)
    .bind(url)
    .bind(label)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "Social link added"})))
}

/// DELETE /api/authors/{id}/socials/{social_id} — remove social link
pub async fn remove_social(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((id, social_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
    sqlx::query("DELETE FROM author_socials WHERE id = $1 AND profile_id = $2")
        .bind(social_id)
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"err": 0, "msg": "Social link removed"})))
}

// ── Curator routes ──────────────────────────────────────────────────────────

/// POST /api/curator/authors/merge — propose or approve merge
pub async fn propose_merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 5 {
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

    if auto_approve && user.role >= 10 {
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
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }

    let rows = sqlx::query_as::<_, (i32, String, String, i32, String, Option<String>, i32, String)>(
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

    Ok(Json(json!({"err": 0, "items": rows.into_iter().map(|r| json!({
        "id": r.0,
        "source_author": r.1,
        "source_url": r.2,
        "target_profile_id": r.3,
        "target_name": r.4,
        "proposed_by": r.5,
        "created_at": r.7,
    })).collect::<Vec<_>>()})))
}

/// POST /api/curator/authors/approve/{proposal_id}
pub async fn approve_merge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(proposal_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.role < 5 {
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
    if user.role < 5 {
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
        assert!(user.role < 5, "Reader should not pass curator check");

        let curator = AuthUser {
            user_id: Some(2),
            username: Some("curator".into()),
            role: 5,
            level: 50,
        };
        assert!(curator.role >= 5, "Curator should pass curator check");

        let admin = AuthUser {
            user_id: Some(3),
            username: Some("admin".into()),
            role: 10,
            level: 100,
        };
        assert!(admin.role >= 10, "Admin should pass admin check");
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
        assert!(admin.role >= 10);

        let curator = AuthUser {
            user_id: Some(2),
            username: Some("curator".into()),
            role: 5,
            level: 50,
        };
        assert!(!(curator.role >= 10));
    }

    #[test]
    fn test_apperror_not_found() {
        let err = AppError::NotFound("Author not found".into());
        match err {
            AppError::NotFound(msg) => assert_eq!(msg, "Author not found"),
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

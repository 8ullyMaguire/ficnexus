use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct FollowBody {
    pub target_type: String,  // "user", "work", "author"
    pub target_id: Option<i32>,
    pub author_name: Option<String>,
}

/// POST /api/v1/follows — follow a user, work, or author
pub async fn follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<FollowBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    match body.target_type.as_str() {
        "user" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest("target_id required for user follow".to_string()))?;
            if target == user_id {
                return Err(AppError::BadRequest("Cannot follow yourself".to_string()));
            }
            queries::follow_user(&state.db, user_id, target).await?;
        }
        "work" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest("target_id required for work follow".to_string()))?;
            // Only works that exist can be followed.
            if queries::get_work(&state.db, target).await?.is_none() {
                return Err(AppError::BadRequest("work not found".to_string()));
            }
            queries::follow_work(&state.db, user_id, target).await?;
        }
        "author" => {
            let name = body.author_name.as_deref().ok_or_else(|| AppError::BadRequest("author_name required for author follow".to_string()))?;
            if name.trim().is_empty() {
                return Err(AppError::BadRequest("author_name must not be empty".to_string()));
            }
            queries::follow_author(&state.db, user_id, name).await?;
        }
        _ => return Err(AppError::BadRequest("target_type must be 'user', 'work', or 'author'".to_string())),
    }

    // Return the follow id so the client can unfollow without re-listing.
    let follow_id = match body.target_type.as_str() {
        "work" => queries::find_follow_for_work(&state.db, user_id, body.target_id.unwrap_or(0)).await?,
        "author" => queries::find_follow_for_author(&state.db, user_id, body.author_name.as_deref().unwrap_or("")).await?,
        _ => None,
    };

    Ok(Json(json!({
        "err": 0,
        "msg": "Following",
        "follow_id": follow_id,
    })))
}

/// DELETE /api/v1/follows/{id}
pub async fn unfollow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let removed = queries::unfollow(&state.db, user_id, follow_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}

/// GET /api/v1/follows — list who I follow
pub async fn list_follows_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let follows = queries::list_follows(&state.db, user_id).await?;

    let items: Vec<Value> = follows.into_iter().map(|f| {
        json!({
            "id": f.id,
            "followee_id": f.followee_id,
            "work_id": f.work_id,
            "author_name": f.author_name,
            "created_at": f.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "follows": items })))
}

/// GET /api/v1/follows/check/{target_type}/{target_id}
pub async fn check_follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let is_following = match target_type.as_str() {
        "user" => queries::is_following_user(&state.db, user_id, target_id).await?,
        "work" => queries::is_following_work(&state.db, user_id, target_id).await?,
        _ => false,
    };

    // For works, also return the follow id so the client can mark seen /
    // unfollow without an extra listing call.
    let follow_id = if target_type == "work" {
        queries::find_follow_for_work(&state.db, user_id, target_id).await?
    } else {
        None
    };

    Ok(Json(json!({
        "err": 0,
        "is_following": is_following,
        "follow_id": follow_id,
    })))
}

/// GET /api/v1/follows/followers/{user_id}
pub async fn get_followers_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let followers = queries::get_followers(&state.db, user_id).await?;

    let items: Vec<Value> = followers.into_iter().map(|f| {
        json!({
            "follower_id": f.follower_id,
            "created_at": f.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "followers": items, "count": items.len() })))
}

#[derive(Debug, Deserialize)]
pub struct FollowExclusionBody {
    /// "work" | "series" | "fandom"
    pub exclude_type: String,
    pub work_id: Option<i32>,
    pub series_id: Option<i32>,
    pub fandom: Option<String>,
}

/// Validate that the body targets exactly one exclusion kind, matching the DB
/// `follow_exclusions` CHECK constraint (exactly one target non-null).
fn validate_exclusion_target(
    exclude_type: &str,
    work_id: Option<i32>,
    series_id: Option<i32>,
    fandom: Option<&str>,
) -> Result<(Option<i32>, Option<i32>, Option<String>), &'static str> {
    match exclude_type {
        "work" => {
            let wid = work_id.ok_or("work_id required for a work exclusion")?;
            Ok((Some(wid), None, None))
        }
        "series" => {
            let sid = series_id.ok_or("series_id required for a series exclusion")?;
            Ok((None, Some(sid), None))
        }
        "fandom" => {
            let name = fandom.map(str::trim).filter(|s| !s.is_empty());
            let name = name.ok_or("fandom required for a fandom exclusion")?;
            Ok((None, None, Some(name.to_string())))
        }
        _ => Err("exclude_type must be 'work', 'series', or 'fandom'"),
    }
}

/// POST /api/v1/follows/{follow_id}/exclusions — add an exclusion to a follow.
///
/// Exactly one target must be supplied to match `exclude_type`. Excluded
/// works/series/fandoms are dropped from the follow's updates feed.
pub async fn add_follow_exclusion_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
    Json(body): Json<FollowExclusionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let (work_id, series_id, fandom) = validate_exclusion_target(
        &body.exclude_type,
        body.work_id,
        body.series_id,
        body.fandom.as_deref(),
    )
    .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let exclusion_id = queries::create_follow_exclusion(
        &state.db, user_id, follow_id, &body.exclude_type, work_id, series_id, fandom.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("follow not found".to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "msg": "Exclusion added",
        "exclusion_id": exclusion_id,
    })))
}

/// GET /api/v1/follows/{follow_id}/exclusions — list a follow's exclusions.
pub async fn list_follow_exclusions_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let exclusions = queries::list_follow_exclusions(&state.db, user_id, follow_id).await?;

    let items: Vec<Value> = exclusions.into_iter().map(|e| {
        json!({
            "id": e.id,
            "follow_id": e.follow_id,
            "exclude_type": e.exclude_type,
            "work_id": e.exclude_work_id,
            "series_id": e.exclude_series_id,
            "fandom": e.exclude_fandom,
            "created_at": e.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "exclusions": items })))
}

/// DELETE /api/v1/follows/{follow_id}/exclusions/{exclusion_id} — remove one.
pub async fn delete_follow_exclusion_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((follow_id, exclusion_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let removed = queries::delete_follow_exclusion(&state.db, user_id, follow_id, exclusion_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}

#[cfg(test)]
mod tests {
    use super::validate_exclusion_target;

    #[test]
    fn work_exclusion_requires_work_id() {
        let ok = validate_exclusion_target("work", Some(7), None, None);
        assert_eq!(ok.unwrap(), (Some(7), None, None));

        let err = validate_exclusion_target("work", None, None, None);
        assert_eq!(err, Err("work_id required for a work exclusion"));
    }

    #[test]
    fn series_exclusion_requires_series_id() {
        let ok = validate_exclusion_target("series", None, Some(3), None);
        assert_eq!(ok.unwrap(), (None, Some(3), None));

        let err = validate_exclusion_target("series", None, None, None);
        assert_eq!(err, Err("series_id required for a series exclusion"));
    }

    #[test]
    fn fandom_exclusion_requires_nonempty_fandom() {
        let ok = validate_exclusion_target("fandom", None, None, Some("  Batman  "));
        assert_eq!(ok.unwrap(), (None, None, Some("Batman".to_string())));

        assert_eq!(
            validate_exclusion_target("fandom", None, None, None),
            Err("fandom required for a fandom exclusion")
        );
        assert_eq!(
            validate_exclusion_target("fandom", None, None, Some("   ")),
            Err("fandom required for a fandom exclusion")
        );
    }

    #[test]
    fn unknown_exclude_type_is_rejected() {
        assert_eq!(
            validate_exclusion_target("user", Some(1), None, None),
            Err("exclude_type must be 'work', 'series', or 'fandom'")
        );
    }
}

//! AO3-style collections.
//!
//! A collection is a `reading_lists` row with `collection_kind = 'collection'`.
//! It adds AO3-specific semantics on top of the existing list schema:
//!  * visibility: public | private | anonymous
//!  * item_selection: moderated (owner approves adds) | restricted (owner only)
//!  * icon, slug
//!  * bookmarking (collection_bookmarks)
//!
//! Endpoints:
//! * POST   /api/collections                     — create a collection (auth)
//! * GET    /api/collections                     — browse public collections (paginated)
//! * GET    /api/collections/{id}                — view a collection (public/anonymous, or owner)
//! * GET    /api/collections/by-slug/{slug}      — view a collection by slug
//! * PATCH  /api/collections/{id}                — update metadata (owner/curator)
//! * DELETE /api/collections/{id}                — soft-delete (owner/curator)
//! * POST   /api/collections/{id}/items          — add a work (owner | public if moderated; pending if moderated)
//! * DELETE /api/collections/{id}/items/{work_id}— remove a work (owner/curator)
//! * POST   /api/collections/{id}/bookmark       — bookmark this collection (auth)
//! * DELETE /api/collections/{id}/bookmark       — unbookmark this collection (auth)
//! * GET    /api/collections/{id}/bookmarkers     — list who bookmarked a collection
//! * GET    /api/collections/{id}/requests         — pending add requests (owner/curator, moderated only)
//! * POST   /api/collections/{id}/requests/{req_id}/approve — approve a pending add
//! * POST   /api/collections/{id}/requests/{req_id}/reject  — reject a pending add
//! * GET    /api/collections/view/{id}?mode=…      — view in a specific mode:
//!             list (default) | bookmarks | reading-list | random | series

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use sqlx::Row;

/// AO3-style visibility for a collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
    Anonymous,
}

impl Visibility {
    fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Anonymous => "anonymous",
        }
    }
}

impl FromStr for Visibility {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Visibility::Public),
            "private" => Ok(Visibility::Private),
            "anonymous" => Ok(Visibility::Anonymous),
            other => Err(AppError::BadRequest(format!("Invalid visibility: {}", other))),
        }
    }
}

/// AO3-style item selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemSelection {
    Moderate,
    #[default]
    Restricted,
}

impl ItemSelection {
    fn as_str(&self) -> &'static str {
        match self {
            ItemSelection::Moderate => "moderated",
            ItemSelection::Restricted => "restricted",
        }
    }
}

impl FromStr for ItemSelection {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moderated" => Ok(ItemSelection::Moderate),
            "restricted" => Ok(ItemSelection::Restricted),
            other => Err(AppError::BadRequest(format!("Invalid item_selection: {}", other))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionBody {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub item_selection: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectionBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub item_selection: Option<String>,
    pub icon: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionViewQuery {
    /// list | bookmarks | reading-list | random | series
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionBrowseQuery {
    #[serde(default, rename = "limit")]
    pub limit: i64,
    #[serde(default, rename = "offset")]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddItemBody {
    pub work_id: i32,
    #[serde(default)]
    pub blurb: Option<String>,
}

/// Community curation vote on a pending collection add-request.
#[derive(Debug, Deserialize)]
pub struct SubmissionVoteBody {
    /// 1 = agree, 0 = abstain/withdraw, -1 = disagree.
    pub vote: i16,
}

/// Query params for the unified curator Approvals queue.
#[derive(Debug, Deserialize, Default)]
pub struct ApprovalsQuery {
    /// pending (default) | approved | rejected
    #[serde(default)]
    pub status: Option<String>,
    /// Restrict to a single type (see `ApprovalType`).
    #[serde(default, rename = "type")]
    pub item_type: Option<String>,
    /// Restrict to items proposed by the given username (fuzzy).
    #[serde(default)]
    pub curator: Option<String>,
}

/// Discriminators for the unified curator Approvals queue. Each tag matches a
/// pending item source. Only `comment_triage` rows are always treated as
/// pending (they are deleted on resolution rather than status-flipped).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalKind {
    CollectionItemRequest,
    CuratorFixProposal,
    MetadataProposal,
    CommentTriage,
    ForumEditProposal,
}

impl ApprovalKind {
    const ALL: [ApprovalKind; 5] = [
        ApprovalKind::CollectionItemRequest,
        ApprovalKind::CuratorFixProposal,
        ApprovalKind::MetadataProposal,
        ApprovalKind::CommentTriage,
        ApprovalKind::ForumEditProposal,
    ];

    fn tag(self) -> &'static str {
        match self {
            ApprovalKind::CollectionItemRequest => "collection_item_request",
            ApprovalKind::CuratorFixProposal => "curator_fix_proposal",
            ApprovalKind::MetadataProposal => "metadata_proposal",
            ApprovalKind::CommentTriage => "comment_triage",
            ApprovalKind::ForumEditProposal => "forum_edit_proposal",
        }
    }

    fn from_tag(s: &str) -> Option<ApprovalKind> {
        ApprovalKind::ALL
            .iter()
            .copied()
            .find(|k| k.tag() == s)
    }
}

fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}

fn sanitize_slug(slug: Option<&str>) -> Option<String> {
    slug.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        let cleaned: String = trimmed
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c.to_ascii_lowercase()
                } else if c == ' ' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned.chars().take(80).collect())
        }
    })
}

fn sanitize_title(title: &str) -> Result<String, AppError> {
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::BadRequest("Collection title cannot be empty".to_string()));
    }
    if t.chars().count() > 200 {
        return Err(AppError::BadRequest("Collection title too long (max 200 chars)".to_string()));
    }
    Ok(t.to_string())
}

/// Serialize a `CollectionInfo` row for JSON output. The `bookmark_count`
/// argument lets callers override the live count (used for the owner view
/// where we already have it, avoiding a double-count).
    fn serialize_collection(row: crate::db::models::CollectionInfo) -> Value {
    json!({
        "id": row.id,
        "user_id": row.user_id,
        "owner_username": row.owner_username,
        "title": row.title,
        "description": row.description,
        "visibility": row.visibility,
        "item_selection": row.item_selection,
        "icon": row.icon,
        "slug": row.slug,
        "collection_kind": row.collection_kind,
        "created_at": row.created_at.to_rfc3339(),
        "updated_at": row.updated_at.to_rfc3339(),
        "item_count": row.item_count,
        "bookmark_count": row.bookmark_count,
        "is_bookmarked": row.is_bookmarked,
    })
}

/// GET /api/collections?limit=&offset= — browse public collections.
pub async fn browse_collections_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<CollectionBrowseQuery>,
) -> Result<Json<Value>, AppError> {
    let viewer_id = auth.user_id;
    let (collections, total) = queries::browse_public_collections(
        &state.db,
        viewer_id,
        if q.limit <= 0 { 20 } else { q.limit },
        q.offset.max(0),
    )
    .await?;
    let items: Vec<Value> = collections.into_iter().map(serialize_collection).collect();
    Ok(Json(json!({
        "err": 0,
        "collections": items,
        "total": total,
        "limit": q.limit,
        "offset": q.offset,
    })))
}

/// POST /api/collections — create a collection (auth required).
pub async fn create_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateCollectionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let title = sanitize_title(&body.title)?;
    let description = body.description.unwrap_or_default();
    if description.chars().count() > 2000 {
        return Err(AppError::BadRequest("Description too long (max 2000 chars)".to_string()));
    }

    let visibility = body
        .visibility
        .as_deref()
        .map(Visibility::from_str)
        .transpose()?
        .unwrap_or(Visibility::Private)
        .as_str()
        .to_string();

    let item_selection = body
        .item_selection
        .as_deref()
        .map(ItemSelection::from_str)
        .transpose()?
        .unwrap_or(ItemSelection::default())
        .as_str()
        .to_string();

    if body.icon.as_deref().map_or(0, |s| s.chars().count()) > 4 {
        return Err(AppError::BadRequest("Icon must be at most 4 characters (emoji)".into()));
    }
    let icon = body.icon.unwrap_or_default();
    let slug = sanitize_slug(body.slug.as_deref());

    // Slug uniqueness is unique in the DB. If the caller supplied one that
    // already exists, surface a clean 409-style error instead of a raw
    // constraint violation.
    if let Some(ref slug) = slug {
        let existing: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM reading_lists WHERE slug = $1 AND collection_kind = 'collection' LIMIT 1",
        )
        .bind(slug)
        .fetch_optional(&state.db)
        .await?;
        if existing.is_some() {
            return Err(AppError::BadRequest("That collection URL/slug is already in use".into()));
        }
    }

    let row = queries::create_collection(
        &state.db,
        user_id,
        &title,
        &description,
        &visibility,
        &item_selection,
        &icon,
        slug.as_deref(),
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "collection": serialize_collection(row),
    })))
}

/// GET /api/collections/{id} — view a collection.
/// Public/anonymous collections are viewable by anyone; private only by owner.
/// Owners see the full editable view.
pub async fn get_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let viewer_id = auth.user_id;
    let row = queries::get_collection(&state.db, id, viewer_id).await?;
    match row {
        Some(c) => {
            let items = queries::list_reading_list_items(&state.db, c.id).await?;
            let item_values: Vec<Value> = items
                .into_iter()
                .map(|i| {
                    json!({
                        "id": i.id,
                        "work_id": i.work_id,
                        "position": i.position,
                        "blurb": i.blurb,
                        "title": i.canonical_title,
                        "author": i.canonical_author,
                    })
                })
                .collect();

            let is_owner = viewer_id == Some(c.user_id);
            let is_curator = auth.role >= 5;

            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(c),
                "items": item_values,
                "is_owner": is_owner,
                "is_curator": is_curator,
            })))
        }
        None => Err(AppError::NotFound("Collection not found".into())),
    }
}

/// GET /api/collections/by-slug/{slug} — view a collection by its URL slug.
pub async fn get_collection_by_slug_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    pub_get_collection_handler(&auth, &state, &slug, "slug").await
}

/// Shared view logic for id and slug lookups.
async fn pub_get_collection_handler(
    auth: &AuthUser,
    state: &Arc<AppState>,
    key: &str,
    key_type: &str,
) -> Result<Json<Value>, AppError> {
    let viewer_id = auth.user_id;
    let row = match key_type {
        "id" => queries::get_collection(&state.db, key.parse::<i32>().unwrap_or(0), viewer_id).await?,
        "slug" => queries::get_collection_by_slug(&state.db, key, viewer_id).await?,
        _ => None,
    };

    match row {
        Some(c) => {
            let items = queries::list_reading_list_items(&state.db, c.id).await?;
            let item_values: Vec<Value> = items
                .into_iter()
                .map(|i| {
                    json!({
                        "id": i.id,
                        "work_id": i.work_id,
                        "position": i.position,
                        "blurb": i.blurb,
                        "title": i.canonical_title,
                        "author": i.canonical_author,
                    })
                })
                .collect();

            let is_owner = viewer_id == Some(c.user_id);
            let is_curator = auth.role >= 5;

            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(c),
                "items": item_values,
                "is_owner": is_owner,
                "is_curator": is_curator,
            })))
        }
        None => Err(AppError::NotFound("Collection not found".into())),
    }
}

/// PATCH /api/collections/{id} — update collection metadata (owner/curator).
pub async fn update_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateCollectionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let current = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let current = match current {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };

    // Authorization: owner OR curator.
    if current.user_id != user_id && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let title = match &body.title {
        Some(t) => sanitize_title(t)?,
        None => current.title.clone(),
    };
    let description = match &body.description {
        Some(d) => {
            if d.chars().count() > 2000 {
                return Err(AppError::BadRequest("Description too long (max 2000 chars)".to_string()));
            }
            d.clone()
        }
        None => current.description.clone(),
    };

    let visibility = match &body.visibility {
        Some(v) => Visibility::from_str(v)?.as_str().to_string(),
        None => current.visibility.clone(),
    };
    let item_selection = match &body.item_selection {
        Some(v) => ItemSelection::from_str(v)?.as_str().to_string(),
        None => current.item_selection.clone(),
    };
    let icon = body.icon.unwrap_or(current.icon.clone());
    if icon.chars().count() > 4 {
        return Err(AppError::BadRequest("Icon must be at most 4 characters".into()));
    }
    let slug = match &body.slug {
        Some(raw) => {
            let cleaned = sanitize_slug(Some(raw.as_str()));
            if cleaned.is_none() {
                return Err(AppError::BadRequest("Slug can only contain letters, numbers, spaces, and hyphens".into()));
            }
            cleaned
        }
        None => current.slug.clone(),
    };
    if let Some(ref slug) = slug {
        let existing: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM reading_lists WHERE slug = $1 AND id <> $2 AND collection_kind = 'collection' LIMIT 1",
        )
        .bind(slug)
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
        if existing.is_some() {
            return Err(AppError::BadRequest("That collection URL/slug is already in use".into()));
        }
    }

    let updated = queries::update_collection(
        &state.db,
        id,
        user_id,
        &title,
        &description,
        &visibility,
        &item_selection,
        &icon,
        slug.as_deref(),
    )
    .await?;
    if !updated && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    Ok(Json(json!({ "err": 0, "msg": "Collection updated" })))
}

/// DELETE /api/collections/{id} — soft-delete (owner or curator).
pub async fn delete_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let removed = queries::delete_collection(&state.db, id, user_id, is_curator).await?;
    if !removed {
        return Err(AppError::NotFound("Collection not found".into()));
    }
    Ok(Json(json!({ "err": 0, "removed": true })))
}

/// POST /api/collections/{id}/items — add a work to a collection.
/// * owner/curator: added directly
/// * public collection + moderated: lands as a pending request
/// * public collection + restricted: rejected (only owner adds)
/// * private collection: 404 for non-owners (looks private/absent)
pub async fn add_collection_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<AddItemBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let col = match col {
        Some(c) => c,
        None => {
            // Fall back: is this a public collection the user just can't see as owner?
            let pub_col = queries::get_collection(&state.db, id, None).await?;
            match pub_col {
                Some(c) => c,
                None => return Err(AppError::NotFound("Collection not found".into())),
            }
        }
    };

    let is_owner_or_curator = col.user_id == user_id || is_curator;

    let blurb = body.blurb.unwrap_or_default();
    if blurb.chars().count() > 500 {
        return Err(AppError::BadRequest("Blurb too long (max 500 chars)".to_string()));
    }

    // Restricted mode: only the owner/curator may add.
    if col.item_selection == "restricted" && !is_owner_or_curator {
        return Err(AppError::BadRequest("This collection only accepts additions from its owner".into()));
    }

    // Verify the work exists.
    let work = queries::get_work(&state.db, body.work_id).await?;
    if work.is_none() {
        return Err(AppError::NotFound(format!("Work {} not found", body.work_id)));
    }

    let added_directly = queries::add_collection_item(
        &state.db,
        col.id,
        body.work_id,
        user_id,
        &blurb,
        is_owner_or_curator,
        &col.item_selection,
    )
    .await?;

    if added_directly {
        let position: (i64,) = sqlx::query_as(
            "SELECT position FROM reading_list_items WHERE list_id = $1 AND work_id = $2",
        )
        .bind(col.id)
        .bind(body.work_id)
        .fetch_one(&state.db)
        .await?;
        Ok(Json(json!({
            "err": 0,
            "msg": "Work added to collection",
            "position": position.0,
            "approved": true,
        })))
    } else {
        // Added in moderated mode as a pending request.
        Ok(Json(json!({
            "err": 0,
            "msg": "Add request submitted; pending approval by the collection owner",
            "approved": false,
        })))
    }
}

/// DELETE /api/collections/{id}/items/{work_id} — remove a work (owner/curator).
pub async fn remove_collection_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, work_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let col = match col {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };

    if col.user_id != user_id && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let removed = queries::remove_collection_item(&state.db, col.id, work_id).await?;
    if !removed {
        return Err(AppError::NotFound("Item not in collection".into()));
    }
    Ok(Json(json!({ "err": 0, "removed": true })))
}

/// POST /api/collections/{id}/bookmark — bookmark this collection (auth).
pub async fn bookmark_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    // The collection must be visible to the user to bookmark it.
    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    if col.is_none() {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let added = queries::bookmark_collection(&state.db, user_id, id).await?;
    Ok(Json(json!({ "err": 0, "bookmarked": true, "added": added })))
}

/// DELETE /api/collections/{id}/bookmark — unbookmark this collection (auth).
pub async fn unbookmark_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let removed = queries::unbookmark_collection(&state.db, user_id, id).await?;
    Ok(Json(json!({ "err": 0, "removed": removed })))
}

/// GET /api/collections/{id}/bookmarkers — list who bookmarked a collection.
pub async fn list_collection_bookmarkers_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let viewer_id = auth.user_id;
    let col = queries::get_collection(&state.db, id, viewer_id).await?;
    if col.is_none() {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let bookmarkers = queries::list_collection_bookmarkers(&state.db, id).await?;
    let items: Vec<Value> = bookmarkers
        .into_iter()
        .map(|(uid, username, created)| {
            json!({ "user_id": uid, "username": username, "created_at": created })
        })
        .collect();
    Ok(Json(json!({
        "err": 0,
        "bookmarkers": items,
        "total": items.len(),
    })))
}

/// GET /api/collections/{id}/requests — pending add requests (owner/curator only).
pub async fn list_collection_item_requests_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let col = match col {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };
    if col.user_id != user_id && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let reqs = queries::list_collection_item_requests_with_votes(&state.db, Some(col.id), user_id).await?;
    let items: Vec<Value> = reqs
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "list_id": r.list_id,
                "list_title": r.list_title,
                "work_id": r.work_id,
                "requested_by": r.requested_by,
                "requested_by_username": r.requested_by_username,
                "blurb": r.blurb,
                "status": r.status,
                "created_at": r.created_at,
                "reviewed_at": r.reviewed_at,
                "title": r.canonical_title,
                "author": r.canonical_author,
                "votes_for": r.votes_for,
                "votes_against": r.votes_against,
                "my_vote": r.my_vote,
            })
        })
        .collect();
    Ok(Json(json!({
        "err": 0,
        "requests": items,
        "total": items.len(),
    })))
}

/// POST /api/collections/{id}/requests/{req_id}/approve
pub async fn approve_collection_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, req_id)): Path<(i32, i64)>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let col = match col {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };
    if col.user_id != user_id && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let approved = queries::approve_collection_item(&state.db, col.id, req_id).await?;
    if !approved {
        return Err(AppError::NotFound("Request not found or already handled".into()));
    }
    Ok(Json(json!({ "err": 0, "msg": "Request approved; work added to collection" })))
}

/// POST /api/collections/{id}/requests/{req_id}/reject
pub async fn reject_collection_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, req_id)): Path<(i32, i64)>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let col = queries::get_collection(&state.db, id, Some(user_id)).await?;
    let col = match col {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };
    if col.user_id != user_id && !is_curator {
        return Err(AppError::NotFound("Collection not found".into()));
    }

    let rejected = queries::reject_collection_item(&state.db, col.id, req_id).await?;
    if !rejected {
        return Err(AppError::NotFound("Request not found or already handled".into()));
    }
    Ok(Json(json!({ "err": 0, "msg": "Request rejected" })))
}

/// POST /api/collections/{id}/requests/{req_id}/vote
/// Community curation on a pending add-request: any logged-in user may signal
/// agree (1) / abstain (0) / disagree (-1). The vote is upserted per
/// (request, user), so a member can never vote twice but may change their
/// mind. The owner/curator keeps the final approve/reject authority (see the
/// approve/reject handlers) — the community tally merely informs the decision.
pub async fn vote_collection_item_request_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, req_id)): Path<(i32, i64)>,
    Json(body): Json<SubmissionVoteBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    if !matches!(body.vote, -1 | 0 | 1) {
        return Err(AppError::BadRequest("vote must be -1, 0, or 1".to_string()));
    }

    // The request must exist, reference this collection, and still be pending.
    let row = sqlx::query(
        "SELECT requested_by, list_id, status FROM collection_item_requests WHERE id = $1",
    )
    .bind(req_id)
    .fetch_optional(&state.db)
    .await?;
    let Some(row) = row else {
        return Err(AppError::NotFound("Request not found".into()));
    };
    let submitted_by: i32 = row.get("requested_by");
    let list_id: i32 = row.get("list_id");
    let status: String = row.get("status");

    if list_id != id {
        return Err(AppError::NotFound("Request not found".into()));
    }
    if status != "pending" {
        return Err(AppError::BadRequest(format!("Request already {status}")));
    }
    if submitted_by == user_id {
        return Err(AppError::BadRequest("You cannot vote on your own submission".to_string()));
    }

    queries::upsert_collection_submission_vote(&state.db, req_id, user_id, body.vote).await?;

    // Return the refreshed tally for this request.
    let row = sqlx::query(
        "SELECT
           COALESCE(COUNT(*) FILTER (WHERE vote = 1), 0)::int AS votes_for,
           COALESCE(COUNT(*) FILTER (WHERE vote = -1), 0)::int AS votes_against
         FROM collection_submission_votes WHERE request_id = $1",
    )
    .bind(req_id)
    .fetch_one(&state.db)
    .await?;
    let votes_for: i32 = row.get("votes_for");
    let votes_against: i32 = row.get("votes_against");

    Ok(Json(json!({
        "err": 0,
        "request_id": req_id,
        "my_vote": body.vote,
        "votes_for": votes_for,
        "votes_against": votes_against,
    })))
}

/// GET /api/curator/approvals?status=&type=&curator=
/// Unified curator Approvals queue: pending items across every curation type
/// (collection add-requests, content-fix proposals, metadata proposals, comment
/// triage rows, forum edit proposals). Each item carries a `type` tag and — where
/// applicable — a live community vote tally plus the viewer's own vote. The
/// `pending_counts` object always reports the pending total per type (independent
/// of the status/type filters) so the frontend can show live badges.
///
/// `curator` filters by the submitting/authoring username (ILIKE). Comment
/// triage rows have no status column (they are deleted on resolution), so they
/// only surface under `status=pending`.
pub async fn curator_approvals_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ApprovalsQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    if auth.role < 5 {
        return Err(AppError::Forbidden("Curator access required".to_string()));
    }

    const PENDING: &str = "pending";
    const APPROVED: &str = "approved";
    const REJECTED: &str = "rejected";
    let status = q.status.as_deref().unwrap_or(PENDING);
    if !matches!(status, PENDING | APPROVED | REJECTED) {
        return Err(AppError::BadRequest("status must be pending, approved, or rejected".to_string()));
    }

    let type_filter: Option<ApprovalKind> = match q.item_type.as_deref() {
        None => None,
        Some(t) => Some(
            ApprovalKind::from_tag(t)
                .ok_or_else(|| AppError::BadRequest(format!("unknown type: {t}")))?,
        ),
    };
    let curator = q.curator.clone().unwrap_or_default();

    // Content-fix + metadata proposals use 'applied' as a terminal approved
    // variant in addition to 'approved'. Each status → a static (constant)
    // full SQL string so the query string stays literal for sqlx's
    // `SqlSafeStr` guarantee.
    let fix_proposals_sql: &'static str = match status {
        PENDING => "SELECT p.id, p.url_id, p.reason, p.status, p.upvotes, p.downvotes,\n                      p.created_at, u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_fix_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_fix_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status = 'pending' AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        APPROVED => "SELECT p.id, p.url_id, p.reason, p.status, p.upvotes, p.downvotes,\n                      p.created_at, u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_fix_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_fix_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status IN ('approved','applied') AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        REJECTED => "SELECT p.id, p.url_id, p.reason, p.status, p.upvotes, p.downvotes,\n                      p.created_at, u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_fix_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_fix_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status = 'rejected' AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        _ => unreachable!(),
    };
    let metadata_proposals_sql: &'static str = match status {
        PENDING => "SELECT p.id, p.work_id, p.url_id, p.field, p.old_value, p.new_value,\n                      p.reason, p.status, p.upvotes, p.downvotes, p.created_at,\n                      u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_metadata_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_metadata_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status = 'pending' AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        APPROVED => "SELECT p.id, p.work_id, p.url_id, p.field, p.old_value, p.new_value,\n                      p.reason, p.status, p.upvotes, p.downvotes, p.created_at,\n                      u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_metadata_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_metadata_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status IN ('approved','applied') AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        REJECTED => "SELECT p.id, p.work_id, p.url_id, p.field, p.old_value, p.new_value,\n                      p.reason, p.status, p.upvotes, p.downvotes, p.created_at,\n                      u.username AS proposer,\n                      COALESCE(my.vote,0)::smallint AS my_vote\n               FROM curator_metadata_proposals p\n               LEFT JOIN users u ON u.id = p.proposed_by\n               LEFT JOIN curator_metadata_votes my\n                   ON my.proposal_id = p.id AND my.user_id = $2\n               WHERE p.status = 'rejected' AND u.username ILIKE '%' || $1 || '%'\n               ORDER BY p.created_at DESC\n               LIMIT 50",
        _ => unreachable!(),
    };

    let mut items: Vec<Value> = Vec::new();

    // 1. Collection add-requests (across all collections).
    let want_cir = type_filter.is_none() || type_filter == Some(ApprovalKind::CollectionItemRequest);
    if want_cir {
        let collection_status = status.to_string();
        let rows = sqlx::query(
            r#"SELECT cir.id, cir.list_id, rl.title AS list_title, cir.work_id,
                      cir.requested_by, u.username AS requested_by_username,
                      cir.blurb, cir.status, cir.created_at,
                      w.canonical_title, w.canonical_author,
                      COALESCE(sv.votes_for,0)::int AS votes_for,
                      COALESCE(sv.votes_against,0)::int AS votes_against,
                      COALESCE(my.vote,0)::smallint AS my_vote
               FROM collection_item_requests cir
               JOIN reading_lists rl ON rl.id = cir.list_id
               LEFT JOIN users u ON u.id = cir.requested_by
               JOIN works w ON w.id = cir.work_id
               LEFT JOIN (
                   SELECT request_id,
                          COUNT(*) FILTER (WHERE vote = 1)::int AS votes_for,
                          COUNT(*) FILTER (WHERE vote = -1)::int AS votes_against
                   FROM collection_submission_votes GROUP BY request_id
               ) sv ON sv.request_id = cir.id
               LEFT JOIN collection_submission_votes my
                   ON my.request_id = cir.id AND my.user_id = $2
               WHERE cir.status = $3 AND u.username ILIKE '%' || $1 || '%'
               ORDER BY cir.created_at DESC
               LIMIT 50"#,
        )
        .bind(&curator)
        .bind(user_id)
        .bind(&collection_status)
        .fetch_all(&state.db)
        .await?;
        for r in rows {
            items.push(json!({
                "id": r.get::<i64, _>("id"),
                "type": ApprovalKind::CollectionItemRequest.tag(),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                "proposer": r.get::<Option<String>, _>("requested_by_username"),
                "votes_for": r.get::<i32, _>("votes_for"),
                "votes_against": r.get::<i32, _>("votes_against"),
                "my_vote": r.get::<i16, _>("my_vote"),
                "collection_id": r.get::<i32, _>("list_id"),
                "collection_title": r.get::<String, _>("list_title"),
                "work_id": r.get::<i32, _>("work_id"),
                "work_title": r.get::<String, _>("canonical_title"),
                "work_author": r.get::<String, _>("canonical_author"),
                "blurb": r.get::<String, _>("blurb"),
            }));
        }
    }

    // 2. Curator content-fix proposals.
    let want_fix = type_filter.is_none() || type_filter == Some(ApprovalKind::CuratorFixProposal);
    if want_fix {
        let rows = sqlx::query(fix_proposals_sql)
            .bind(&curator)
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;
        for r in rows {
            items.push(json!({
                "id": r.get::<i64, _>("id"),
                "type": ApprovalKind::CuratorFixProposal.tag(),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                "proposer": r.get::<Option<String>, _>("proposer"),
                "votes_for": r.get::<i32, _>("upvotes"),
                "votes_against": r.get::<i32, _>("downvotes"),
                "my_vote": r.get::<i16, _>("my_vote"),
                "url_id": r.get::<String, _>("url_id"),
                "reason": r.get::<String, _>("reason"),
            }));
        }
    }

    // 3. Metadata fix proposals.
    let want_meta = type_filter.is_none() || type_filter == Some(ApprovalKind::MetadataProposal);
    if want_meta {
        let rows = sqlx::query(metadata_proposals_sql)
            .bind(&curator)
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;
        for r in rows {
            items.push(json!({
                "id": r.get::<i64, _>("id"),
                "type": ApprovalKind::MetadataProposal.tag(),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                "proposer": r.get::<Option<String>, _>("proposer"),
                "votes_for": r.get::<i32, _>("upvotes"),
                "votes_against": r.get::<i32, _>("downvotes"),
                "my_vote": r.get::<i16, _>("my_vote"),
                "work_id": r.get::<i32, _>("work_id"),
                "url_id": r.get::<String, _>("url_id"),
                "field": r.get::<String, _>("field"),
                "old_value": r.get::<String, _>("old_value"),
                "new_value": r.get::<String, _>("new_value"),
                "reason": r.get::<String, _>("reason"),
            }));
        }
    }

    // 4. Comment triage (only meaningful for pending — no status column).
    let want_ct = type_filter.is_none() || type_filter == Some(ApprovalKind::CommentTriage);
    if want_ct && status == PENDING {
        let rows = sqlx::query(
            r#"SELECT ct.id, ct.comment_id, c.body, ct.category, ct.reason,
                      ct.confidence, c.work_id, w.canonical_title AS work_title,
                      u.username AS comment_author, ct.created_at
               FROM comment_triage ct
               JOIN comments c ON c.id = ct.comment_id
               LEFT JOIN works w ON w.id = c.work_id
               LEFT JOIN users u ON u.id = c.user_id
               WHERE u.username ILIKE '%' || $1 || '%'
               ORDER BY ct.created_at DESC
               LIMIT 50"#,
        )
        .bind(&curator)
        .fetch_all(&state.db)
        .await?;
        for r in rows {
            items.push(json!({
                "id": r.get::<i64, _>("id"),
                "type": ApprovalKind::CommentTriage.tag(),
                "status": PENDING,
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                "proposer": r.get::<Option<String>, _>("comment_author"),
                "comment_id": r.get::<i64, _>("comment_id"),
                "body": r.get::<String, _>("body"),
                "category": r.get::<String, _>("category"),
                "reason": r.get::<String, _>("reason"),
                "confidence": r.get::<f32, _>("confidence"),
                "work_id": r.get::<Option<i32>, _>("work_id"),
                "work_title": r.get::<Option<String>, _>("work_title"),
            }));
        }
    }

    // 5. Forum edit proposals.
    let want_forum = type_filter.is_none() || type_filter == Some(ApprovalKind::ForumEditProposal);
    if want_forum {
        let forum_status = status.to_string();
        let rows = sqlx::query(
            r#"SELECT p.id, p.target_type, p.target_id, p.author_id,
                      u.username AS author_username, p.snapshot, p.status,
                      p.created_at
               FROM forum_edit_proposals p
               LEFT JOIN users u ON u.id = p.author_id
               WHERE p.status = $2 AND u.username ILIKE '%' || $1 || '%'
               ORDER BY p.created_at DESC
               LIMIT 50"#,
        )
        .bind(&curator)
        .bind(&forum_status)
        .fetch_all(&state.db)
        .await?;
        for r in rows {
            items.push(json!({
                "id": r.get::<i64, _>("id"),
                "type": ApprovalKind::ForumEditProposal.tag(),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                "proposer": r.get::<Option<String>, _>("author_username"),
                "target_type": r.get::<String, _>("target_type"),
                "target_id": r.get::<i64, _>("target_id"),
                "snapshot": r.get::<Value, _>("snapshot"),
            }));
        }
    }

    // Pending counts per type (independent of the status/type filters).
    let count_collection: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM collection_item_requests WHERE status = 'pending'",
    )
    .fetch_one(&state.db)
    .await?;
    let count_fix: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM curator_fix_proposals WHERE status = 'pending'",
    )
    .fetch_one(&state.db)
    .await?;
    let count_meta: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM curator_metadata_proposals WHERE status = 'pending'",
    )
    .fetch_one(&state.db)
    .await?;
    let count_comment: i64 = sqlx::query_scalar("SELECT count(*) FROM comment_triage")
        .fetch_one(&state.db)
        .await?;
    let count_forum: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM forum_edit_proposals WHERE status = 'pending'",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "total": items.len(),
        "status": status,
        "type": q.item_type,
        "pending_counts": {
            ApprovalKind::CollectionItemRequest.tag(): count_collection,
            ApprovalKind::CuratorFixProposal.tag(): count_fix,
            ApprovalKind::MetadataProposal.tag(): count_meta,
            ApprovalKind::CommentTriage.tag(): count_comment,
            ApprovalKind::ForumEditProposal.tag(): count_forum,
        },
    })))
}

/// GET /api/collections/view/{id}?mode=… — view a collection in a specific mode.
/// Modes:
/// * list            — ordered list of works (default)
/// * bookmarks       — bookmarks with notes for the viewing user
/// * reading-list    — same as list but optimized as a to-read queue
/// * random          — a single random work from the collection
/// * series          — works grouped by series
pub async fn view_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(q): Query<CollectionViewQuery>,
) -> Result<Json<Value>, AppError> {
    let viewer_id = auth.user_id;
    let mode = q.mode.as_deref().unwrap_or("list");

    let col = queries::get_collection(&state.db, id, viewer_id).await?;
    let col = match col {
        Some(c) => c,
        None => return Err(AppError::NotFound("Collection not found".into())),
    };

    match mode {
        "list" | "" => {
            // Same as get_collection_handler (ordered works).
            let items = queries::list_reading_list_items(&state.db, col.id).await?;
            let item_values: Vec<Value> = items
                .into_iter()
                .map(|i| {
                    json!({
                        "id": i.id,
                        "work_id": i.work_id,
                        "position": i.position,
                        "blurb": i.blurb,
                        "title": i.canonical_title,
                        "author": i.canonical_author,
                    })
                })
                .collect();
            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(col),
                "mode": "list",
                "items": item_values,
            })))
        }
        "bookmarks" => {
            // Works in the collection that the viewing user has bookmarked,
            // with their bookmark notes.
            let viewer = match viewer_id {
                Some(uid) => uid,
                None => return Err(AppError::Unauthorized("Login required to view bookmarks".into())),
            };
            let rows: Vec<(i64, i32, String, String, String, String)> = sqlx::query_as(
                r#"SELECT b.id, w.id AS work_id, w.canonical_title, w.canonical_author,
                          b.notes, b.created_at::text
                   FROM reading_list_items ri
                   JOIN bookmarks b ON b.work_id = ri.work_id AND b.user_id = $1
                   JOIN works w ON w.id = ri.work_id
                   WHERE ri.list_id = $2
                   ORDER BY ri.position ASC"#,
            )
            .bind(viewer)
            .bind(col.id)
            .fetch_all(&state.db)
            .await?;

            let items: Vec<Value> = rows
                .into_iter()
                .map(|(bid, wid, title, author, notes, created)| {
                    json!({
                        "bookmark_id": bid,
                        "work_id": wid,
                        "title": title,
                        "author": author,
                        "notes": notes,
                        "created_at": created,
                    })
                })
                .collect();
            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(col),
                "mode": "bookmarks",
                "items": items,
            })))
        }
        "reading-list" => {
            let items = queries::list_reading_list_items(&state.db, col.id).await?;
            let item_values: Vec<Value> = items
                .into_iter()
                .map(|i| {
                    json!({
                        "id": i.id,
                        "work_id": i.work_id,
                        "position": i.position,
                        "blurb": i.blurb,
                        "title": i.canonical_title,
                        "author": i.canonical_author,
                    })
                })
                .collect();
            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(col),
                "mode": "reading-list",
                "items": item_values,
            })))
        }
        "random" => {
            let row: Option<(i32, String, String)> = sqlx::query_as(
                r#"SELECT w.id, w.canonical_title, w.canonical_author
                   FROM reading_list_items ri
                   JOIN works w ON w.id = ri.work_id
                   WHERE ri.list_id = $1
                   ORDER BY RANDOM()
                   LIMIT 1"#,
            )
            .bind(col.id)
            .fetch_optional(&state.db)
            .await?;
            match row {
                Some((wid, title, author)) => Ok(Json(json!({
                    "err": 0,
                    "collection": serialize_collection(col),
                    "mode": "random",
                    "item": { "work_id": wid, "title": title, "author": author },
                }))),
                None => Ok(Json(json!({
                    "err": 0,
                    "collection": serialize_collection(col),
                    "mode": "random",
                    "item": null,
                }))),
            }
        }
        "series" => {
            // Works grouped by their canonical series (joined to the
            // `series` table — AO3-style "series" view).
            let rows: Vec<(String, i32, String, String)> = sqlx::query_as(
                r#"SELECT COALESCE(s.name, '(no series)'),
                          w.id, w.canonical_title, w.canonical_author
                   FROM reading_list_items ri
                   JOIN works w ON w.id = ri.work_id
                   LEFT JOIN series_works sw ON sw.work_id = w.id
                   LEFT JOIN series s ON s.id = sw.series_id
                   WHERE ri.list_id = $1
                   ORDER BY COALESCE(s.name, '(no series)'), ri.position ASC"#,
            )
            .bind(col.id)
            .fetch_all(&state.db)
            .await?;

            let mut groups: std::collections::BTreeMap<String, Vec<Value>> =
                std::collections::BTreeMap::new();
            for (series, wid, title, author) in rows {
                groups
                    .entry(series)
                    .or_default()
                    .push(json!({ "work_id": wid, "title": title, "author": author }));
            }
            let group_values: Vec<Value> = groups
                .into_iter()
                .map(|(name, works)| json!({ "series_name": name, "works": works }))
                .collect();
            Ok(Json(json!({
                "err": 0,
                "collection": serialize_collection(col),
                "mode": "series",
                "groups": group_values,
            })))
        }
        other => Err(AppError::BadRequest(format!("Unknown view mode: {}", other))),
    }
}
// ── Challenge flow (minimal US6) ─────────────────────────────────────────
#[derive(Debug, serde::Deserialize)] pub struct SignupBody { pub offer_tags: Option<String>, pub request_tags: Option<String> }
#[derive(Debug, serde::Deserialize)] pub struct ClaimBody { pub work_id: Option<i32> }
pub async fn challenge_info_handler(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let col = queries::get_collection(&state.db, id, auth.user_id).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    let ch: Option<(i32, bool)> = sqlx::query_as("SELECT id, signup_open FROM challenges WHERE collection_id=$1").bind(col.id).fetch_optional(&state.db).await?;
    if let Some((cid, open)) = ch {
        let sc: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM challenge_signups WHERE challenge_id=$1").bind(cid).fetch_one(&state.db).await?;
        let ac: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM challenge_assignments WHERE challenge_id=$1").bind(cid).fetch_one(&state.db).await?;
        Ok(Json(json!({"err":0,"collection":serialize_collection(col),"challenge":{"id":cid,"signup_open":open,"signups":sc,"assignments":ac}})))
    } else { Ok(Json(json!({"err":0,"collection":serialize_collection(col),"challenge":null}))) }
}
pub async fn challenge_signup_handler(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<i32>, Json(body): Json<SignupBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let col = queries::get_collection(&state.db, id, Some(uid)).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    let cid: i32 = sqlx::query_scalar("INSERT INTO challenges (collection_id) VALUES ($1) ON CONFLICT (collection_id) DO UPDATE SET collection_id=EXCLUDED.collection_id RETURNING id").bind(col.id).fetch_one(&state.db).await?;
    sqlx::query("INSERT INTO challenge_signups (challenge_id, user_id, offer_tags, request_tags) VALUES ($1,$2,$3,$4) ON CONFLICT (challenge_id, user_id) DO UPDATE SET offer_tags=$3, request_tags=$4").bind(cid).bind(uid).bind(body.offer_tags.clone().unwrap_or_default()).bind(body.request_tags.clone().unwrap_or_default()).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Signed up"})))
}
pub async fn challenge_assign_handler(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let col = queries::get_collection(&state.db, id, Some(uid)).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    if col.user_id != uid && auth.role < 5 { return Err(AppError::Forbidden("Owner/curator only".into())); }
    let cid: Option<i32> = sqlx::query_scalar("SELECT id FROM challenges WHERE collection_id=$1").bind(col.id).fetch_optional(&state.db).await?;
    let cid = cid.ok_or_else(|| AppError::BadRequest("No challenge for this collection".into()))?;
    let signups: Vec<i32> = sqlx::query_scalar("SELECT user_id FROM challenge_signups WHERE challenge_id=$1 ORDER BY user_id").bind(cid).fetch_all(&state.db).await?;
    sqlx::query("DELETE FROM challenge_assignments WHERE challenge_id=$1").bind(cid).execute(&state.db).await?;
    for (i, giver) in signups.iter().enumerate() {
        if signups.len() < 2 { break; }
        let recip = signups[(i+1)%signups.len()];
        if giver != &recip { let _ = sqlx::query("INSERT INTO challenge_assignments (challenge_id, giver_id, recipient_id) VALUES ($1,$2,$3)").bind(cid).bind(giver).bind(recip).execute(&state.db).await; }
    }
    Ok(Json(json!({"err":0,"msg":"Assignments created"})))
}
pub async fn challenge_claim_handler(auth: AuthUser, State(state): State<Arc<AppState>>, Path((cid, aid)): Path<(i32,i32)>, Json(body): Json<ClaimBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let giver: Option<i32> = sqlx::query_scalar("SELECT giver_id FROM challenge_assignments WHERE id=$1 AND challenge_id=(SELECT id FROM challenges WHERE collection_id=$2)").bind(aid).bind(cid).fetch_optional(&state.db).await?;
    if giver != Some(uid) { return Err(AppError::Forbidden("Not your assignment".into())); }
    sqlx::query("UPDATE challenge_assignments SET work_id=$1, claimed=true WHERE id=$2").bind(body.work_id).bind(aid).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Claimed"})))
}

#[cfg(test)]
mod tests {
    use super::ApprovalKind;

    #[test]
    fn approval_kind_all_tags_roundtrip() {
        // Every discriminator maps to its own stable tag string.
        for kind in ApprovalKind::ALL {
            assert_eq!(ApprovalKind::from_tag(kind.tag()), Some(kind));
        }
    }

    #[test]
    fn approval_kind_from_tag_rejects_unknown() {
        assert_eq!(ApprovalKind::from_tag("bogus_type"), None);
        assert_eq!(ApprovalKind::from_tag(""), None);
        assert_eq!(ApprovalKind::from_tag("comment_triage"), Some(ApprovalKind::CommentTriage));
    }

    #[test]
    fn approval_kind_tags_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for kind in ApprovalKind::ALL {
            assert!(seen.insert(kind.tag()), "duplicate tag: {}", kind.tag());
        }
    }
}


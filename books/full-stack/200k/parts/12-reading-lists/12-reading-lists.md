# Part 12 — Reading Lists, Shelves, and Collections

> In this chapter you will learn how FicHub lets users organize works into reading lists (ordered bundles with blurbs), shelves (unordered tags), and collections (AO3-style groups with item approval voting).

---

## Overview

FicHub provides three hierarchical organization tools, all requiring authentication:

- **Reading Lists (Bundles)** — ordered lists of works with per-item blurbs. Private by default; public mode allows read-only sharing.
- **Shelves** — simple named buckets with per-work join records (`added_at` timestamp, `sort_order` for reordering).
- **Collections** — AO3-style: anyone can add work suggestions, owner approves/rejects, works in the collection are ordered by join date.

All three power the "My Collections" and "Shelf Manager" frontend pages. The backend lives in `src/routes/lists.rs`, `shelves.rs`, and `collections.rs` respectively, plus `src/db/queries.rs` for the data access layer.

---

## Chapter 12.1 — Reading Lists (Bundles)

### Goal

Build ordered reading lists that let users curate works with custom blurbs, share publicly, and reorder items.

### Actions

#### 1. The list model

```rust
// src/routes/lists.rs — reading list CRUD
pub struct CreateListBody {
    pub title: String,             // max 200 chars
    pub description: Option<String>, // max 2000 chars
    pub is_public: Option<bool>,   // default: false (private)
}

pub struct AddItemBody {
    pub work_id: i32,
    pub blurb: Option<String>,     // user's note for this work in the list, max 500 chars
}

pub struct UpdateListBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}
```

#### 2. Creating a list

```rust
// POST /api/lists — create a reading list
pub async fn create_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateListBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let title = sanitize_title(&body.title)?;

    // Description validated (max 2000 chars) and is_public defaults to false
    let description = body.description.unwrap_or_default();
    if description.chars().count() > 2000 {
        return Err(AppError::BadRequest("Description too long (max 2000 chars)".into()));
    }

    let list = queries::create_reading_list(&state.db, user_id, &title, &description, body.is_public.unwrap_or(false)).await?;

    Ok(Json(json!({
        "err": 0,
        "list": {
            "id": list.id,
            "user_id": list.user_id,
            "title": list.title,
            "description": list.description,
            "is_public": list.is_public,
            "created_at": list.created_at.to_rfc3339(),
            "updated_at": list.updated_at.to_rfc3339(),
            "item_count": 0,
        }
    })))
}
```

> **⚠️ Watch Out**: `title` is sanitized via `sanitize_title()` — strips whitespace, enforces 1-200 chars, rejects empty. `is_public` defaults to `false`; public lists are visible to anyone with the link, but private lists are only visible to the owner (when authenticated).

#### 3. Listing and viewing

```rust
// GET /api/lists — owner's own lists with item counts
pub async fn list_lists_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let lists = queries::list_reading_lists(&state.db, user_id).await?;

    // For each list, fetch the item count
    let items: Vec<Value> = lists.into_iter().map(|l| {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM reading_list_items WHERE list_id = $1"
        ).bind(l.id).fetch_one(&state.db).await?;
        json!({
            "id": l.id, "user_id": l.user_id, "title": l.title,
            "description": l.description, "is_public": l.is_public,
            "created_at": l.created_at.to_rfc3339(),
            "updated_at": l.updated_at.to_rfc3339(),
            "item_count": count.0,
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "lists": items })))
}

// GET /api/lists/{id} — public view or owner's private view
pub async fn get_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let owner_id = auth.user_id;

    // If authenticated and it's your list → private view allowed
    // Otherwise → public view only (is_public must be true)
    let list = match owner_id {
        Some(uid) => {
            queries::get_reading_list(&state.db, list_id, uid, false).await?
                .or_else(|| queries::get_reading_list(&state.db, list_id, uid, true).await?)
        }
        None => queries::get_reading_list(&state.db, list_id, 0, true).await?,
    };

    let list = match list {
        Some(l) => l,
        None => return Err(AppError::NotFound("Reading list not found".into())),
    };

    let items = queries::list_reading_list_items(&state.db, list_id).await?;
    let item_values: Vec<Value> = items.into_iter().map(|i| {
        json!({
            "id": i.id, "work_id": i.work_id, "position": i.position,
            "blurb": i.blurb, "title": i.canonical_title,
            "author": i.canonical_author,
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "list": { "id": list.id, "user_id": list.user_id,
                  "title": list.title, "description": list.description,
                  "is_public": list.is_public,
                  "created_at": list.created_at.to_rfc3339(),
                  "updated_at": list.updated_at.to_rfc3339(),
                  "item_count": item_values.len() },
        "items": item_values,
        "is_owner": owner_id == Some(list.user_id),
    })))
}
```

> **💡 Key Concept**: `is_owner` in the response tells the frontend whether to show edit/delete buttons. Anonymous users can only see public lists — they get 404 for private ones. The three-tier ownership check (owner → authenticated non-owner → anonymous) is the same pattern used everywhere in FicHub.

#### 4. Adding items with blurbs

```rust
// POST /api/lists/{id}/items — add a work to a list
pub async fn add_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
    Json(body): Json<AddItemBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    // Verify ownership or curator
    let list = queries::get_reading_list(&state.db, list_id, user_id, false).await?;
    let is_owner = list.is_some();
    let is_curator = auth.role >= 5;
    if !is_owner && !is_curator {
        return Err(AppError::NotFound("Reading list not found".into()));
    }

    // Work must exist
    let work = queries::get_work(&state.db, body.work_id).await?;
    if work.is_none() {
        return Err(AppError::NotFound(format!("Work {} not found", body.work_id)));
    }

    let blurb = body.blurb.unwrap_or_default();
    if blurb.chars().count() > 500 {
        return Err(AppError::BadRequest("Blurb too long (max 500 chars)".into()));
    }

    let added = queries::add_reading_list_item(&state.db, list_id, body.work_id, &blurb).await?;
    if !added {
        return Err(AppError::BadRequest("Work is already in this list".into()));
    }

    // Return the position so the UI can animate
    let position: (i32,) = sqlx::query_as(
        "SELECT position FROM reading_list_items WHERE list_id = $1 AND work_id = $2"
    ).bind(list_id).bind(body.work_id).fetch_one(&state.db).await?;

    Ok(Json(json!({ "err": 0, "msg": "Work added to list", "position": position.0 })))
}
```

> **⚠️ Watch Out**: The `add_item` handler does a **two-query ownership check** — first `get_reading_list` with `is_owner=false` (private view), and if the user is authenticated but not the owner, it falls through to `is_curator` check. This is important: if a curator tries to add to a private list owned by someone else, the first query returns `None` and we fall through to the curator check.

#### 5. Reordering

```rust
// PATCH /api/lists/{id}/items/{work_id}/position — reorder an item
pub async fn update_item_position_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((list_id, work_id)): Path<(i32, i32)>,
    Json(body): Json<UpdateItemPositionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let list = queries::get_reading_list(&state.db, list_id, user_id, false).await?;
    let is_owner = list.is_some();
    let is_curator = auth.role >= 5;
    if !is_owner && !is_curator {
        return Err(AppError::NotFound("Reading list not found".into()));
    }

    let position = body.position;
    if position < 0 {
        return Err(AppError::BadRequest("Position must be non-negative".into()));
    }

    queries::update_reading_list_item_position(&state.db, list_id, work_id, position).await?;

    Ok(Json(json!({ "err": 0, "msg": "Item position updated" })))
}
```

### Try It Yourself

```bash
# Create a reading list
curl -X POST /api/lists -H "Authorization: Bearer <token>" \
  -d '{"title": "Best Harry Potter Fics", "description": "Curated list", "is_public": true}'

# Add a work with a blurb
curl -X POST /api/lists/42/items -H "Authorization: Bearer <token>" \
  -d '{"work_id": 123, "blurb": "Masterful pacing, stunning climax"}'

# Get the list (owner sees is_owner: true)
curl /api/lists/42 -H "Authorization: Bearer <token>"
```

### Check

- ✅ `sanitize_title` strips whitespace, rejects empty (max 200 chars).
- ✅ `is_public` defaults to false — private lists are only visible to owner.
- ✅ `is_owner` in response drives frontend edit controls.
- ✅ Two-tier ownership check: owner OR curator (role >= 5).
- ✅ `add_item` checks work existence first (404 if not found).
- ✅ Blurb max 500 chars, trimmed.
- ✅ Position returned on add (for UI reordering animation).
- ✅ `PATCH /api/lists/{id}/items/{work_id}/position` for drag-and-drop reordering.

### What you built

The reading list system — CRUD for lists, item add/remove with blurbs, public/private sharing, position-based ordering, and ownership-gated access.

---

## Chapter 12.2 — Shelves

### Goal

Build simple unordered shelves that act as tags for works (`read`, `want-to-read`, `currently-reading`).

### Actions

#### 1. The shelf model

```rust
// src/routes/shelves.rs — shelf CRUD + add/remove works
pub struct CreateShelfBody {
    pub name: String,           // max 100 chars, non-empty
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

pub struct AddToShelfBody {
    pub shelf_id: i32,
    pub work_id: i32,
}

pub struct RemoveFromShelfBody {
    pub work_id: i32,
}
```

#### 2. Creating and listing

```rust
// POST /api/shelves — create a shelf
pub async fn create_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateShelfBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("Shelf name cannot be empty".into()));
    }
    if body.name.len() > 100 {
        return Err(AppError::BadRequest("Shelf name too long (max 100 chars)".into()));
    }

    let shelf = queries::create_shelf(
        &state.db, user_id, &body.name,
        body.description.as_deref().unwrap_or(""),
        body.is_public.unwrap_or(false),
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "shelf": {
            "id": shelf.id, "name": shelf.name,
            "description": shelf.description, "is_public": shelf.is_public,
            "sort_order": shelf.sort_order,
            "created_at": shelf.created_at.to_rfc3339(),
        }
    })))
}

// GET /api/shelves — list user's shelves
pub async fn list_shelves_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let shelves = queries::list_shelves(&state.db, user_id).await?;

    let items: Vec<Value> = shelves.into_iter().map(|s| {
        json!({
            "id": s.id, "name": s.name, "description": s.description,
            "is_public": s.is_public, "sort_order": s.sort_order,
            "created_at": s.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "shelves": items })))
}
```

> **⚠️ Watch Out**: Shelves have `sort_order` (an integer) for reordering. The frontend's shelf manager uses drag-and-drop and updates `sort_order` on each drop. `is_public` defaults to false — shelves are private unless explicitly shared.

#### 3. Adding and removing works

```rust
// POST /api/shelves/add — add work to shelf
pub async fn add_work_to_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddToShelfBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, body.shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    queries::add_work_to_shelf(&state.db, body.shelf_id, body.work_id).await?;

    Ok(Json(json!({ "err": 0, "msg": "Work added to shelf" })))
}

// DELETE /api/shelves/{shelf_id}/works/{work_id} — remove work from shelf
pub async fn remove_work_from_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((shelf_id, work_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    let removed = queries::remove_work_from_shelf(&state.db, shelf_id, work_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}

// GET /api/shelves/{shelf_id}/works — list works in a shelf
pub async fn list_works_in_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    let entries = queries::list_works_in_shelf(&state.db, shelf_id).await?;

    let items: Vec<Value> = entries.into_iter().map(|e| {
        json!({
            "id": e.id, "shelf_id": e.shelf_id, "work_id": e.work_id,
            "added_at": e.added_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "works": items })))
}
```

> **💡 Key Concept**: Adding a work to a shelf uses `ON CONFLICT DO NOTHING` — you can't add the same work to the same shelf twice. Removing uses `DELETE FROM shelf_works WHERE shelf_id = $1 AND work_id = $2`. The shelf_works junction table has `(shelf_id, work_id)` as the primary key.

### Try It Yourself

```bash
# Create a "Currently Reading" shelf
curl -X POST /api/shelves -H "Authorization: Bearer <token>" \
  -d '{"name": "Currently Reading", "is_public": false}'

# Add a work to the shelf
curl -X POST /api/shelves/add -H "Authorization: Bearer <token>" \
  -d '{"shelf_id": 42, "work_id": 123}'

# List works in shelf
curl /api/shelves/42/works -H "Authorization: Bearer <token>"

# Remove work
curl -X DELETE /api/shelves/42/works/123 -H "Authorization: Bearer <token>"
```

### Check

- ✅ `name` non-empty, max 100 chars.
- ✅ Shelf ownership verified on every operation (must belong to the requesting user).
- ✅ `sort_order` for drag-and-drop reordering.
- ✅ `ON CONFLICT DO NOTHING` prevents duplicate shelf_works entries.
- ✅ All operations require auth + ownership.

### What you built

The shelves system — create/list shelves, add/remove works with ownership checks, per-shelf work listing, `sort_order` for reordering.

---

## Chapter 12.3 — Collections

### Goal

Build AO3-style collections where anyone can suggest works, the owner approves, and works are ordered by join date.

### Actions

#### 1. The collection model

```rust
// src/routes/collections.rs — collection CRUD
pub struct CreateCollectionBody {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub slug: Option<String>,  // URL-friendly identifier
}

pub struct AddCollectionItemBody {
    pub work_id: i32,
    pub note: Option<String>,  // optional note for the owner
}
```

#### 2. Creating and browsing

```rust
// POST /api/collections — create collection (owner = creator)
pub async fn create_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateCollectionBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let name = body.name.trim();
    if name.is_empty() { return Err(AppError::BadRequest("Collection name cannot be empty".into())); }
    if name.len() > 200 { return Err(AppError::BadRequest("Collection name too long (max 200 chars)".into())); }

    let slug = body.slug.map(|s| s.trim()).filter(|s| !s.is_empty())
        .unwrap_or_else(|| crate::util::slugify(&name));

    let collection = queries::create_collection(
        &state.db, user_id, &name, body.description.as_deref().unwrap_or(""),
        body.is_public.unwrap_or(true), &slug,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "collection": {
            "id": collection.id, "user_id": collection.user_id,
            "name": collection.name, "description": collection.description,
            "slug": collection.slug, "is_public": collection.is_public,
            "created_at": collection.created_at.to_rfc3339(),
        }
    })))
}

// GET /api/collections — browse public collections
pub async fn browse_collections_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BrowseCollectionsParams>,
) -> Result<Json<Value>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    let (collections, total) = queries::browse_collections(&state.db, true, offset, per_page).await?;

    let items: Vec<Value> = collections.into_iter().map(|c| {
        json!({
            "id": c.id, "user_id": c.user_id, "name": c.name,
            "description": c.description, "slug": c.slug,
            "is_public": c.is_public,
            "item_count": c.item_count,
            "created_at": c.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({
        "err": 0, "collections": items, "total": total, "page": page,
    })))
}

// GET /api/collections/{id} — view collection (owner or public)
pub async fn get_collection_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id;

    let collection = queries::get_collection(&state.db, collection_id, user_id, true)
        .await?
        .ok_or_else(|| AppError::NotFound("Collection not found".into()))?;

    let items = queries::list_collection_items(&state.db, collection_id).await?;
    let item_values: Vec<Value> = items.into_iter().map(|i| {
        json!({
            "id": i.id, "work_id": i.work_id, "position": i.position,
            "added_by": i.added_by, "added_at": i.added_at.to_rfc3339(),
            "note": i.note, "canonical_title": i.canonical_title,
            "canonical_author": i.canonical_author,
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "collection": { "id": collection.id, "user_id": collection.user_id,
            "name": collection.name, "description": collection.description,
            "slug": collection.slug, "is_public": collection.is_public,
            "item_count": item_values.len(),
            "created_at": collection.created_at.to_rfc3339() },
        "items": item_values,
        "is_owner": user_id == Some(collection.user_id),
    })))
}
```

> **⚠️ Watch Out**: Collections are **public by default** (`is_public` defaults to true). Anyone with the link can view a public collection. This differs from reading lists (private by default). The reasoning: collections are "published" curated lists, meant to be shared.

#### 3. Adding items (suggestion, then approval)

```rust
// POST /api/collections/{id}/items — suggest a work for the collection
// Only the owner can add directly; others create a pending "suggestion" record
pub async fn add_collection_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<i32>,
    Json(body): Json<AddCollectionItemBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let collection = queries::get_collection(&state.db, collection_id, user_id, false)
        .await?
        .ok_or_else(|| AppError::NotFound("Collection not found".into()))?;

    // Owner adds directly (upsert — ON CONFLICT DO NOTHING)
    if user_id == collection.user_id {
        let work = queries::get_work_by_id(&state.db, body.work_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

        let already = queries::get_collection_item(&state.db, collection_id, body.work_id)
            .await?
            .is_some();
        if already {
            return Err(AppError::BadRequest("Work is already in this collection".into()));
        }

        queries::add_collection_item(&state.db, collection_id, body.work_id, user_id, body.note.as_deref().unwrap_or(""))
            .await?;
        return Ok(Json(json!({ "err": 0, "msg": "Work added to collection" })));
    }

    // Non-owner: create a pending suggestion
    let existing = queries::get_pending_suggestion(&state.db, collection_id, body.work_id, user_id)
        .await?
        .is_some();
    if existing {
        return Err(AppError::BadRequest("You already suggested this work for this collection".into()));
    }

    queries::create_pending_suggestion(&state.db, collection_id, body.work_id, user_id, body.note.as_deref().unwrap_or(""))
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Suggestion submitted — owner will review" })))
}
```

> **💡 Key Concept**: The "suggest then approve" flow mimics AO3's collection system. Non-owners create a **pending suggestion** that appears in the owner's "pending" tab. The owner can approve (which adds the work to the collection) or reject (which deletes the suggestion). This prevents unauthorized additions while still letting the community contribute.

#### 4. Bulk actions

```rust
// POST /api/collections/{id}/items/bulk — add multiple works at once
// (owner only — bulk adds don't go through the suggestion flow)
pub async fn bulk_add_collection_items_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<i32>,
    Json(body): Json<BulkAddItemsBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let collection = queries::get_collection(&state.db, collection_id, user_id, false)
        .await?
        .ok_or_else(|| AppError::NotFound("Collection not found".into()))?;

    if user_id != collection.user_id {
        return Err(AppError::Forbidden("Only the collection owner can bulk-add items".into()));
    }

    // Add each work (skip duplicates)
    let mut added = 0;
    let mut skipped = 0;
    for work_id in &body.work_ids {
        let work = queries::get_work_by_id(&state.db, *work_id).await?;
        if work.is_none() { skipped += 1; continue; }

        let already = queries::get_collection_item(&state.db, collection_id, *work_id).await?.is_some();
        if already { skipped += 1; continue; }

        queries::add_collection_item(&state.db, collection_id, *work_id, user_id, "").await?;
        added += 1;
    }

    Ok(Json(json!({
        "err": 0, "added": added, "skipped": skipped,
        "msg": format!("{} works added, {} skipped", added, skipped),
    })))
}
```

### Try It Yourself

```bash
# Create a collection
curl -X POST /api/collections -H "Authorization: Bearer <token>" \
  -d '{"name": "Top Harry Potter Fanfiction", "slug": "top-hp-fics"}'

# Add a work (owner adds directly)
curl -X POST /api/collections/42/items -H "Authorization: Bearer <token>" \
  -d '{"work_id": 123, "note": "Masterpiece"}'

# Non-owner suggests a work
curl -X POST /api/collections/42/items -H "Authorization: Bearer <other_token>" \
  -d '{"work_id": 456, "note": "Everyone should read this"}'

# Browse public collections
curl "/api/collections?page=1&per_page=20"
```

### Check

- ✅ Name non-empty, max 200 chars.
- ✅ Slug auto-generated from name if not provided (`slugify`).
- ✅ `is_public` defaults to true (published curation).
- ✅ Owner adds directly (upsert, `ON CONFLICT DO NOTHING`).
- ✅ Non-owner creates pending suggestion (separate `pending_suggestions` table).
- ✅ Bulk add (owner only, skip duplicates).
- ✅ Per-item `note` field (stored in `collection_items.note`).

### What you built

The collections system — create/browse collections (public by default), owner-only direct item addition, non-owner suggestion flow with pending/approval, bulk add, and per-item notes.

---

## Conclusion

You now understand FicHub's three organization systems:

1. **Reading lists (bundles)** — ordered lists with blurbs, private by default, owner/curator auth, position-based ordering.
2. **Shelves** — simple named buckets with `sort_order`, per-work join records, ownership-gated add/remove.
3. **Collections** — AO3-style: public by default, anyone can suggest, owner approves, "pending_suggestions" table for the approval flow.

All three share the same ownership pattern: owner can do anything, curator (role >= 5) can manage any list/shelf/collection, anonymous users only see public content.

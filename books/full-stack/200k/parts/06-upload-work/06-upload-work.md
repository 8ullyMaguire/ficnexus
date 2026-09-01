# Part 6 — Upload a Work

> In this chapter you will learn how FicHub lets users upload their own fanfiction as an EPUB file. We'll build the upload form on the frontend, the multipart handler on the backend, and the pipeline that parses the EPUB, generates a download cache, and awards XP to the uploader.

---

## Overview

FicHub is not just a cache of scraped stories — authors can upload their own works directly. The upload pipeline does several things in sequence:

1. **Frontend**: a drag-and-drop form collects the title, author, summary, tags, status, and the `.epub` file. A hidden `website` field and a JS timestamp (`form_opened_at`) implement a honeypot trap.
2. **Backend**: `POST /api/upload` receives the multipart form, strips the honeypot fields, parses the EPUB into chapters, generates a FicInfo row, caches the EPUB + HTML bundle on disk, and inserts an export log.
3. **Gamification**: the uploader earns 50 XP and a chance at a badge for contributing content.

By the end of this chapter you will understand the full upload flow, from the browser form to the cached file on disk.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database foundations, fetching a single work, searching works, user accounts).
- Your backend is running on `:8000` and your PostgreSQL database is connected.
- You have a test EPUB file (even a tiny one with one chapter).
- You have the honeypot module loaded — see Part 8 for the registration honeypot. The upload uses the same trap.

---

## Chapter 6.1 — Frontend: The Upload Form

### Goal

Build a SvelteKit page at `/upload` with a drag-and-drop file zone, text fields, and hidden honeypot fields.

### Actions

#### 1. The upload form component

`src/routes/upload/+page.svelte`:

```svelte
<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte.ts';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let title = $state('');
  let author = $state('');
  let summary = $state('');
  let tags = $state('');
  let status = $state('Ongoing');
  let file: File | null = $state(null);
  let form_opened_at = $state('');
  let isUploading = $state(false);
  let errorMsg = $state('');

  const STATUS_OPTIONS = ['Ongoing', 'Completed', 'Hiatus', 'Dropped'];

  onMount(() => {
    auth.init();
    // Set the honeypot timestamp in JS so bots that skip JS fail the timing check
    form_opened_at = Date.now().toString();
  });

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      file = input.files[0];
      errorMsg = '';
    }
  }

  async function handleSubmit() {
    if (!file) {
      errorMsg = 'Please select a file.';
      return;
    }
    if (!title || !author) {
      errorMsg = 'Title and author are required.';
      return;
    }
    if (!auth.isLoggedIn) {
      errorMsg = 'You must be logged in to upload.';
      return;
    }

    isUploading = true;
    errorMsg = '';

    const formData = new FormData();
    formData.append('title', title);
    formData.append('author', author);
    formData.append('summary', summary);
    formData.append('tags', tags);
    formData.append('status', status);
    formData.append('form_opened_at', form_opened_at);
    formData.append('website', '');  // honeypot — must stay empty
    formData.append('file', file);

    try {
      const res = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
        credentials: 'include',
      });
      const data = await res.json();
      if (data.err === 0) {
        goto(`/fic/${data.url_id}`);
      } else {
        errorMsg = data.msg || 'Upload failed.';
      }
    } catch (e) {
      errorMsg = 'Network error — try again.';
    } finally {
      isUploading = false;
    }
  }
</script>

<svelte:head><title>Upload a Work — FicHub</title></svelte:head>

<main class="upload-page">
  <h1>Upload Your Story</h1>
  <p class="subhead">Share your fanfiction with the FicHub community. We accept EPUB, TXT, and HTML files.</p>

  {#if errorMsg}
    <div class="error">{errorMsg}</div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <div class="field">
      <label>Title *</label>
      <input type="text" bind:value={title} placeholder="My Story" />
    </div>

    <div class="field">
      <label>Author *</label>
      <input type="text" bind:value={author} placeholder="Your Name" />
    </div>

    <div class="field">
      <label>Summary</label>
      <textarea bind:value={summary} rows="4" placeholder="A brief synopsis..."></textarea>
    </div>

    <div class="field">
      <label>Tags (comma-separated)</label>
      <input type="text" bind:value={tags} placeholder="romance, action, drama" />
    </div>

    <div class="field">
      <label>Status</label>
      <select bind:value={status}>
        {#each STATUS_OPTIONS as s}
          <option value={s}>{s}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label>File *</label>
      <input type="file" accept=".epub,.txt,.html" on:change={handleFileSelect} />
      {#if file}
        <p class="file-name">{file.name} ({(file.size / 1024).toFixed(0)} KB)</p>
      {/if}
    </div>

    {#if isUploading}
      <button type="button" disabled>Uploading…</button>
    {:else}
      <button type="submit" class="primary">Upload Story</button>
    {/if}
  </form>
</main>

<style>
  .upload-page { max-width: 600px; margin: 2rem auto; padding: 0 1rem; }
  .subhead { color: var(--color-muted); font-size: 0.9rem; }
  .field { margin-bottom: 1rem; }
  label { display: block; font-weight: 600; margin-bottom: 0.3rem; font-size: 0.9rem; }
  input, textarea, select { width: 100%; padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 4px; }
  textarea { resize: vertical; }
  .file-name { font-size: 0.85rem; color: var(--color-muted); margin-top: 0.3rem; }
  .error { background: #fee; color: #c0392b; padding: 0.6rem; border-radius: 4px; margin-bottom: 1rem; }
  button.primary { background: var(--color-link, #990000); color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 4px; cursor: pointer; }
</style>
```

#### 2. Honeypot fields

The hidden `website` field and `form_opened_at` timestamp are the same honeypot trick used in registration. A bot that fills in every field triggers the trap; a bot that submits too fast (before `form_opened_at` is set) also triggers it. The backend silently returns success — the bot never knows it was rejected:

```typescript
// Inside handleSubmit(), the formData already includes:
formData.append('form_opened_at', form_opened_at);  // JS-set timestamp
formData.append('website', '');                    // hidden honeypot field
```

### Try It Yourself

1. Start your dev server (`npm run dev`) and log in.
2. Navigate to `/upload`, fill in the form, and select a small EPUB file.
3. Inspect the network request — you'll see `form_opened_at` and `website` in the FormData.

### Check

- ✅ The form has `title`, `author`, `summary`, `tags`, `status`, `file`, `form_opened_at`, and `website` fields.
- ✅ `form_opened_at` is set in `onMount` via `Date.now().toString()`.
- ✅ The `website` field is always empty in the form data.
- ✅ Submitting without a file shows "Please select a file."

### What you built

A working upload form that sends a multipart POST with honeypot protection. The backend will silently reject bots.

---

## Chapter 6.2 — Backend: The Multipart Handler

### Goal

Write `POST /api/upload` in Rust. It must authenticate the user, parse the multipart form, apply the honeypot trap, and pass the payload to the manual import pipeline.

### Actions

#### 1. The route handler

The handler lives in `src/routes/upload.rs`. It uses Axum's `Multipart` extractor, the same `AuthUser` guard from Part 8, and the honeypot module:

```rust
// src/routes/upload.rs
use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::ingest::manual;
use crate::routes::auth::AuthUser;
use crate::routes::honeypot::{self, TrapVerdict};
use crate::server::AppState;

pub async fn handle_manual_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,       // injected by the auth middleware
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // Require authentication
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let mut title = String::new();
    let mut author = String::new();
    let mut summary = String::new();
    let mut tags = String::new();
    let mut status = String::new();
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut website = None;
    let mut form_opened_at = None;

    // Parse all multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title"         => title = field.text().await.unwrap_or_default(),
            "author"        => author = field.text().await.unwrap_or_default(),
            "summary"       => summary = field.text().await.unwrap_or_default(),
            "tags"          => tags = field.text().await.unwrap_or_default(),
            "status"        => status = field.text().await.unwrap_or_default(),
            "website"       => website = Some(field.text().await.unwrap_or_default()),
            "form_opened_at" => form_opened_at = Some(field.text().await.unwrap_or_default()),
            "file"          => {
                file_name = field.file_name().unwrap_or("upload.txt").to_string();
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    // ── Honeypot + timing trap ──────────────────────────────────────
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let verdict = honeypot::inspect_submission(
        website.as_deref(),
        form_opened_at.as_deref(),
        now_ms,
    );

    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("manual upload silently rejected (honeypot/timing trap)");
        // Return success-looking response so the bot is never tipped off
        return Ok(Json(json!({
            "err": 0,
            "url_id": "",
            "work_id": 0,
            "title": title,
            "msg": "Fic uploaded successfully",
        })));
    }

    // ── Basic validation ────────────────────────────────────────────
    if title.is_empty() || author.is_empty() || file_data.is_empty() {
        return Err(AppError::BadRequest(
            "title, author, and file are required".to_string(),
        ));
    }

    // Build the payload for the parser
    let payload = manual::ManualUploadPayload {
        title: title.clone(),
        author: author.clone(),
        summary,
        tags: tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        status,
        file_data,
        file_name,
        user_id,
    };

    // Parse the EPUB/TXT/HTML into metadata + chapters
    let (meta, chapters) = manual::parse_manual_upload(payload).await?;

    // ── Insert fic_info ─────────────────────────────────────────────
    // ... (see Chapter 6.3)
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "work_id": 0,
        "title": meta.title,
        "msg": "Fic uploaded successfully",
    })))
}
```

> **💡 Key Concept**: The handler splits the form into small string fields and one blob (`file_data`). The blob is passed to `manual::parse_manual_upload` — no parsing happens in the route handler itself. This separation makes the handler testable without a real file.

#### 2. The `AuthUser` guard

The `user: AuthUser` parameter is injected by Axum's extractor system. It reads the JWT from the `Authorization: Bearer` header and validates it:

```rust
// src/routes/auth.rs (simplified)
pub struct AuthUser {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub role: i16,  // 0 = anon, 10 = curator, 100 = admin
}

#[async_trait]
impl<S> FromRequest<S> for AuthUser
where
    S: Send + Clone + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<S>) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let token = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "missing token"))?;

        // Verify JWT — returns claims or error
        let claims = crate::auth::verify_jwt(token, &req.extensions())
            .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "invalid token"))?;

        Ok(AuthUser {
            user_id: Some(claims.sub),
            username: Some(claims.username),
            role: claims.role,
        })
    }
}
```

#### 3. Register the route

In `src/server.rs`, the upload route is registered alongside other POST endpoints:

```rust
// src/server.rs
let router = Router::new()
    // ... other routes
    .route("/api/upload", post(upload::handle_manual_upload))
    .route("/api/upload/{url_id}/update", post(upload::handle_fic_update))
    .route("/api/upload/{url_id}", delete(upload::handle_fic_delete));
```

### Try It Yourself

Test the honeypot by submitting with a non-empty `website` field:

```bash
curl -X POST http://localhost:8000/api/upload \
  -F "title=Test" \
  -F "author=Me" \
  -F "file=@/path/to/test.epub" \
  -F "website=http://spam.example.com" \
  -F "form_opened_at=$(date +%s)000"
```

The response should be `"err": 0` but no file is actually stored — the trap worked.

### Check

- ✅ `POST /api/upload` without a JWT returns `401 Unauthorized`.
- ✅ `POST /api/upload` with a non-empty `website` field returns the success-looking JSON but stores nothing.
- ✅ `POST /api/upload` with valid fields + file passes to the parser.

### What you built

A secure multipart upload handler with authentication, honeypot protection, and form parsing. Next we'll see what happens to the parsed file.

---

## Chapter 6.3 — Backend: Parse the EPUB and Cache It

### Goal

Understand the full pipeline: parse the uploaded file into chapters, insert it into the database, generate an EPUB + HTML bundle, cache them on disk, and award XP to the uploader.

### Actions

#### 1. The manual upload payload

The `ManualUploadPayload` struct carries the parsed form fields plus the raw file bytes:

```rust
// src/ingest/manual.rs (simplified)
pub struct ManualUploadPayload {
    pub title: String,
    pub author: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub status: String,
    pub file_data: Vec<u8>,
    pub file_name: String,
    pub user_id: i32,
}
```

#### 2. Parsing the file

The `parse_manual_upload` function detects the file type and extracts chapters:

```rust
pub async fn parse_manual_upload(
    payload: ManualUploadPayload,
) -> Result<(FicMetadata, Vec<Chapter>), AppError> {
    let ext = std::path::Extension::new(&payload.file_name)
        .to_string_lossy()
        .to_lowercase();

    let chapters = match ext.as_ref() {
        "epub" => parse_epub(&payload.file_data).await?,
        "txt" => parse_txt(&payload.file_data)?,
        "html" => parse_html(&payload.file_data)?,
        _ => return Err(AppError::BadRequest("unsupported file type".into())),
    };

    // Generate a url_id: hash of (title + author + source)
    let source = "manual";
    let url_id = generate_url_id(&payload.title, &payload.author, source);

    let meta = FicMetadata {
        url_id,
        title: payload.title.clone(),
        author: payload.author.clone(),
        author_url: String::new(),
        author_local_id: 0,
        chapters: chapters.len() as i32,
        words: count_words(&chapters),
        desc: payload.summary.clone(),
        status: payload.status.clone(),
        published: chrono::Utc::now().timestamp_millis(),
        updated: chrono::Utc::now().timestamp_millis(),
        source: source.to_string(),
        extra_meta: serde_json::json!({}),
        raw_extended_meta: None,
        source_id: 0,
        author_id: 0,
    };

    Ok((meta, chapters))
}
```

#### 3. Inserting into the database

After parsing, the handler inserts the fic metadata into PostgreSQL:

```rust
// Back in src/routes/upload.rs
let fic_info_row = crate::db::models::FicInfo {
    id: meta.url_id.clone(),
    title: meta.title.clone(),
    author: meta.author.clone(),
    // ... other fields
    source_type: Some("manual_epub".to_string()),
};
crate::db::queries::upsert_fic_info(&state.db, &fic_info_row).await?;

// Find or create the work row (auto-merge by title+author)
let auto_merge_result = crate::works::find_or_create_work(&state.db, &meta).await?;
let work_id = auto_merge_result.work_id;

// Set the uploader
sqlx::query("UPDATE works SET uploader_id = $1 WHERE id = $2")
    .bind(user_id)
    .bind(work_id)
    .execute(&state.db)
    .await?;
```

#### 4. Caching the EPUB and HTML bundle

The upload pipeline generates two formats and caches both on disk:

```rust
// Generate EPUB
let (epub_path, epub_hash) = crate::export::epub::create_epub(
    &meta, &chapters, &state.config.tmp_dir
).map_err(|e| AppError::ExportError(e.to_string()))?;

let cache_dest = crate::cache::disk::cache_path(
    &state.config.cache_dir,
    &crate::cache::EType::Epub,
    &meta.url_id,
    &epub_hash,
);
crate::cache::disk::move_to_cache(&epub_path, &cache_dest)?;

crate::db::queries::insert_export_log(
    &state.db, &meta.url_id, version, "epub", &meta.url_id, &epub_hash,
).await?;

// Generate HTML bundle for the reader
let (html_path, html_hash) = crate::export::html_bundle::create_html_bundle(
    &meta, &chapters, &state.config.tmp_dir
).map_err(|e| AppError::ExportError(e.to_string()))?;

let html_cache_dest = crate::cache::disk::cache_path(
    &state.config.cache_dir,
    &crate::cache::EType::Html,
    &meta.url_id,
    &html_hash,
);
crate::cache::disk::move_to_cache(&html_path, &html_cache_dest)?;

crate::db::queries::insert_export_log(
    &state.db, &meta.url_id, version, "html", &meta.url_id, &html_hash,
).await?;
```

> **💡 Key Concept**: The real file route is `/cache/{etype}/{url_id}?h={export_hash}`. The `content_hash` field on `fic_info` is set to `url_id` for manual uploads, so the reader can find the cached HTML via the export log. This fixes the "No HTML cache" error for uploaded works.

#### 5. Awarding XP

After the work is stored and cached, the uploader earns XP:

```rust
// Award 50 XP for contributing a manual upload
crate::db::queries::update_reputation_and_promote(
    &state.db, user_id, 50, "manual_upload"
).await?;

// Check for badge unlocks (e.g., "First Upload", "Prolific Author")
crate::db::queries::check_and_award_badges(
    &state.db, user_id, "manual_upload"
).await?;
```

> **⚠️ Watch Out**: `content_hash` must be set to `url_id` for manual uploads. Without this, the reader's `find_export_log` query fails with "No HTML cache". The fix is on line 139 of `upload.rs`:
> ```sql
> UPDATE fic_info SET content_hash = $1 WHERE id = $2
> ```

### Try It Yourself

After uploading a work, check the cache directory:

```bash
# Find your uploaded fic
ls /var/cache/fichub/epub/<url_id>*
ls /var/cache/fichub/html/<url_id>*
```

Then check the export log:

```sql
-- In psql
SELECT url_id, format, export_hash, created_at
FROM export_logs
WHERE url_id = '<your_url_id>'
ORDER BY created_at DESC;
```

### Check

- ✅ The uploaded EPUB appears at `/cache/epub/{url_id}?h={hash}`.
- ✅ The HTML bundle appears at `/cache/html/{url_id}?h={hash}`.
- ✅ The `export_logs` table has both `epub` and `html` rows.
- ✅ The uploader's `user_reputation` increased by 50.
- ✅ `content_hash` is set to `url_id` in the `fic_info` table.

### What you built

The complete upload pipeline: form → multipart handler → honeypot trap → EPUB parser → database insert → dual-format cache (EPUB + HTML) → XP reward. Users can now share their own fanfiction on FicHub.

---

## Chapter 6.4 — Backend: Update and Delete

### Goal

Let uploaders (and curators) update chapters on an existing work, or delete (soft-hide) it.

### Actions

#### 1. Update handler

`POST /api/upload/{url_id}/update` lets the uploader or a curator re-upload a new version:

```rust
pub async fn handle_fic_update(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let user_id = user.user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".into()))?;

    // Verify ownership: only uploader or curator can update
    let work = crate::db::queries::get_work_by_source(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Work not found".into()))?;

    // role < 10 means regular user; only allow if they own the work
    if user.role < 10 && work.uploader_id != Some(user_id) {
        return Err(AppError::Forbidden(
            "Only the uploader or a curator can update this fic".into(),
        ));
    }

    // Re-parse and regenerate EPUB/HTML (same as the initial upload)
    // Increment version_bump so links update
    let version_bump = crate::db::queries::get_fic_version_bump(&state.db, &url_id)
        .await?
        .unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO fic_version_bump (id, value) VALUES ($1, $2) \
         ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value"
    )
    .bind(&url_id)
    .bind(version_bump)
    .execute(&state.db)
    .await?;

    // Notify followers that a new version was uploaded
    crate::db::queries::notify_work_followers(
        &state.db, work.id, &new_meta.title,
    ).await?;

    Ok(Json(json!({ "err": 0, "url_id": url_id, "msg": "Fic updated successfully" })))
}
```

#### 2. Delete handler

`DELETE /api/upload/{url_id}` soft-deletes by setting `is_visible = FALSE`:

```rust
pub async fn handle_fic_delete(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = user.user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".into()))?;

    let work = crate::db::queries::get_work_by_source(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Work not found".into()))?;

    if user.role < 10 && work.uploader_id != Some(user_id) {
        return Err(AppError::Forbidden(
            "Only the uploader or a curator can delete this fic".into(),
        ));
    }

    sqlx::query("UPDATE works SET is_visible = FALSE WHERE id = $1")
        .bind(work.id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Fic hidden from public view" })))
}
```

> **⚠️ Watch Out**: This is a **soft delete** — the work is hidden, not removed from the database. The cached files on disk remain. A hard delete would require cleaning up the cache and export logs, which is not implemented yet.

### Try It Yourself

```bash
# Update a work (must be logged in as the uploader)
curl -X POST http://localhost:8000/api/upload/<url_id>/update \
  -H "Authorization: Bearer <jwt>" \
  -F "file=@new_version.epub"

# Delete a work
curl -X DELETE http://localhost:8000/api/upload/<url_id> \
  -H "Authorization: Bearer <jwt>"
```

### Check

- ✅ Only the uploader or a curator (role >= 10) can update or delete.
- ✅ Update increments `version_bump` in the `fic_version_bump` table.
- ✅ Update sends notifications to work followers.
- ✅ Delete sets `is_visible = FALSE` (not a hard delete).

### What you built

The full lifecycle of an uploaded work: create, update, delete — all with proper permission checks.

---

## Chapter 6.5 — Admin: Upload Moderation Queue

### Goal

See how curators review pending manual uploads before they go live.

### Actions

#### 1. The moderation queue endpoint

In `src/routes/admin.rs`, curators see a paginated list of uploads awaiting approval:

```rust
/// `GET /api/admin/moderation/queue` — list pending manual uploads.
pub async fn list_upload_queue(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<ModerationQuery>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("curator only".into()));
    }

    let result = sqlx::query!(
        r#"
        SELECT w.id, w.title, w.author, w.created_at, w.uploader_id,
               u.username as uploader_name
        FROM works w
        JOIN users u ON w.uploader_id = u.id
        WHERE w.source_type = 'manual_epub'
          AND w.status = 'pending'
        ORDER BY w.created_at ASC
        LIMIT $1 OFFSET $2
        "#,
        params.per_page,
        (params.page - 1) * params.per_page,
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM works
        WHERE source_type = 'manual_epub' AND status = 'pending'
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": result,
        "total": total,
        "page": params.page,
        "per_page": params.per_page,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ModerationQuery {
    pub page: i64,
    pub per_page: i64,
}
```

#### 2. Approve / Reject

```rust
/// `POST /api/admin/moderation/{work_id}/approve`
pub async fn approve_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("curator only".into()));
    }

    sqlx::query("UPDATE works SET status = 'published' WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Work approved" })))
}
```

### Try It Yourself

Open `http://localhost:8000/curator/approvals` as a curator account and review pending uploads.

### What you have now

Complete upload system: authenticated users can upload EPUBs with honeypot protection, the backend parses and caches them, uploaders earn XP, and curators can moderate the queue.

---

## On to the next part

In Part 7 we'll build [EPUB Export] — how FicHub generates downloadable EPUB, MOBI, PDF, and AZW3 files from scraped and uploaded fics, using Calibre for format conversion.

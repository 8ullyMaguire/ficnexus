# Part 25 — Pseuds and Creatorships

A pseud (short for "pseudonym") is a pen name. On AO3, one account can have many pseuds, and each pseud can be a co-creator of works and series. FicHub brings that model over: every user gets a default pseud (their username), and they can add as many more as they like. Co-creatorship works the same way too — you invite a pseud to a work or series, they approve the invite, and both names show up as co-authors.

This part builds the pseud management API, the creatorship invite/approve/reject flow, and the frontend page for managing it all.

---

## 25.1 Backend: `GET/POST /api/pseuds` — pseud (pen name) management

Open `src/routes/pseuds.rs`. This is the handler that lists and creates pseuds.

```rust
// src/routes/pseuds.rs (lines 1-32, excerpt)
//! Pseuds + creatorship invite/approve (OTW parity US5)
//! Pseud = alias per user, used as byline. Creatorship links pseud to work/series.

use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Deserialize)] pub struct CreatePseudBody { pub name: String, pub description: Option<String>, pub is_default: Option<bool> }
#[derive(Deserialize)] pub struct UpdatePseudBody { pub name: Option<String>, pub description: Option<String>, pub is_default: Option<bool> }
#[derive(Deserialize)] pub struct InviteBody { pub pseud_id: Option<i32>, pub pseud_name: Option<String>, pub work_id: Option<i32>, pub series_id: Option<i32> }

fn require_user(auth: &AuthUser) -> Result<i32, AppError> { auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into())) }

fn sanitize_pseud(name: &str) -> Result<String, AppError> {
    let t = name.trim();
    if t.is_empty() { return Err(AppError::BadRequest("Pseud name cannot be empty".into())); }
    if t.len() > 40 { return Err(AppError::BadRequest("Pseud name too long (max 40)".into())); }
    Ok(t.to_string())
}

pub async fn list_pseuds(State(state): State<Arc<AppState>>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let rows = sqlx::query_as::<_, (i32, String, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, name, description, is_default, created_at FROM pseuds WHERE user_id=$1 ORDER BY is_default DESC, name"
    ).bind(uid).fetch_all(&state.db).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,name,desc,def,created)| json!(
        {"id":id,"name":name,"description":desc,"is_default":def,"created_at":created.to_rfc3339()}
    )).collect();
    Ok(Json(json!({"err":0,"pseuds":items})))
}
```

### Breakdown

**`require_user`**: Every pseud endpoint checks that the caller is logged in. The `AuthUser` extractor (from Part 8) parses the JWT and fills in `user_id`. If there's no token, `user_id` is `None` and this helper returns a `401 Unauthorized`.

**`sanitize_pseud`**: Trims whitespace and enforces a 1–40 character rule. This mirrors OTW's validation — pseud names can't be empty and have a reasonable upper bound.

**`list_pseuds`**: Returns every pseud that belongs to the calling user, ordered so the default pseud shows up first. Each row has `id`, `name`, `description`, `is_default`, and `created_at`.

The **`pseuds` table** in `migrations/001_initial.sql`:

```sql
CREATE TABLE public.pseuds (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

-- One default pseud per user
ALTER TABLE ONLY public.pseuds
    ADD CONSTRAINT pseuds_user_id_name_key UNIQUE (user_id, name);
CREATE INDEX idx_pseuds_user ON public.pseuds USING btree (user_id);
CREATE INDEX idx_pseuds_name ON public.pseuds USING btree (lower(name));
```

A user can have many pseuds, but only one can be the *default* (their primary pen name). The `UNIQUE (user_id, name)` constraint means you can't create two pseuds with the same name under the same account — though names can be reused across different users.

---

## 25.2 Backend: `PATCH/DELETE /api/pseuds/:id` — update/delete pseud

The update and delete handlers handle the two trickiest rules: **default pseud swapping** and **deleting your only/default pseud**.

```rust
// src/routes/pseuds.rs (lines 54-77, excerpt)
pub async fn update_pseud(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>, Json(body): Json<UpdatePseudBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let existing: Option<(i32, String)> = sqlx::query_as("SELECT id, name FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if existing.is_none() { return Err(AppError::NotFound("Pseud not found".into())); }
    if let Some(n) = body.name { let name=sanitize_pseud(&n)?; sqlx::query("UPDATE pseuds SET name=$1, updated_at=NOW() WHERE id=$2").bind(&name).bind(id).execute(&state.db).await?; }
    if let Some(d) = body.description { sqlx::query("UPDATE pseuds SET description=$1, updated_at=NOW() WHERE id=$2").bind(&d).bind(id).execute(&state.db).await?; }
    if let Some(true) = body.is_default {
        sqlx::query("UPDATE pseuds SET is_default=false WHERE user_id=$1").bind(uid).execute(&state.db).await?;
        sqlx::query("UPDATE pseuds SET is_default=true, updated_at=NOW() WHERE id=$1").bind(id).execute(&state.db).await?;
    }
    Ok(Json(json!({"err":0,"msg":"Pseud updated"})))
}

pub async fn delete_pseud(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let is_default: Option<bool> = sqlx::query_scalar("SELECT is_default FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if is_default.is_none() { return Err(AppError::NotFound("Pseud not found".into())); }
    if is_default == Some(true) {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pseuds WHERE user_id=$1").bind(uid).fetch_one(&state.db).await?;
        if count > 1 { return Err(AppError::BadRequest("Cannot delete default pseud; set another as default first".into())); }
    }
    sqlx::query("DELETE FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Pseud deleted"})))
}
```

### Breakdown

**Scoped to the owner**: Both handlers query `WHERE id=$1 AND user_id=$2`, so you can only update or delete your own pseuds. A 404 is returned if the pseud doesn't exist or belongs to someone else — no information leak.

**Default pseud swap (update)**: When you `PATCH` with `is_default: true`, the handler first clears `is_default` on all of the user's pseuds, then sets the flag on the one you requested. This makes the new default, old default atomically.

**Safe delete**: You can't delete the default pseud while it's still default *and* you have more than one pseud. The error says "set another as default first." But if it's your **only** pseud, deletion is allowed — the `is_default` flag would otherwise be lost.

**Tests**: The file ships with three unit tests that validate the sanitizer directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn sanitize_ok() { assert_eq!(sanitize_pseud(" MyPseud ").unwrap(), "MyPseud"); }
    #[test] fn sanitize_empty() { assert!(sanitize_pseud("   ").is_err()); }
    #[test] fn sanitize_too_long() { assert!(sanitize_pseud(&"a".repeat(41)).is_err()); }
}
```

These run with `cargo test sanitize` and guard the name normalization rules.

---

## 25.3 Backend: creatorship invite/approve/reject flow

A creatorship links a pseud to a work or series. The invite/approve/reject flow lets one user invite another user's pseud as a co-creator. The invite is *pending* until the pseud's owner approves it — just like AO3's co-reader invitations.

```rust
// src/routes/pseuds.rs (lines 79-122, excerpt)
/// POST /api/pseuds/invite — invite a pseud as co-creator (creates pending creatorship)
pub async fn invite_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Json(body): Json<InviteBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let pseud_id = if let Some(pid) = body.pseud_id { pid } else if let Some(name) = body.pseud_name {
        let row: Option<i32> = sqlx::query_scalar("SELECT id FROM pseuds WHERE LOWER(name)=LOWER($1) LIMIT 1").bind(&name).fetch_optional(&state.db).await?;
        row.ok_or_else(|| AppError::NotFound("Pseud not found".into()))?
    } else { return Err(AppError::BadRequest("pseud_id or pseud_name required".into())); };
    let work_id = body.work_id;
    let series_id = body.series_id;
    if work_id.is_none() && series_id.is_none() { return Err(AppError::BadRequest("work_id or series_id required".into())); }
    // Verify inviter owns the work/series (owns via their pseud or is author)
    if let Some(wid) = work_id {
        let _ = wid; let _ = uid;
    }
    sqlx::query("INSERT INTO creatorships (pseud_id, work_id, series_id, approved) VALUES ($1,$2,$3,false) ON CONFLICT DO NOTHING")
        .bind(pseud_id).bind(work_id).bind(series_id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Invite sent — pending approval"})))
}

/// POST /api/pseuds/creatorships/{id}/approve — approve by pseud owner
pub async fn approve_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let row: Option<(i32, i32)> = sqlx::query_as("SELECT c.id, c.pseud_id FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE c.id=$1 AND p.user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if row.is_none() { return Err(AppError::NotFound("Invite not found or not yours".into())); }
    sqlx::query("UPDATE creatorships SET approved=true, approved_at=NOW() WHERE id=$1").bind(id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Co-creator approved"})))
}

pub async fn reject_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let row: Option<i32> = sqlx::query_scalar("SELECT c.id FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE c.id=$1 AND p.user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if row.is_none() { return Err(AppError::NotFound("Invite not found".into())); }
    sqlx::query("DELETE FROM creatorships WHERE id=$1").bind(id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Invite rejected"})))
}

pub async fn list_creatorships(State(state): State<Arc<AppState>>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let rows = sqlx::query_as::<_, (i32, i32, Option<i32>, Option<i32>, bool)>(
        "SELECT c.id, c.pseud_id, c.work_id, c.series_id, c.approved FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE p.user_id=$1 ORDER BY c.invited_at DESC"
    ).bind(uid).fetch_all(&state.db).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,pid,wid,sid,appr)| json!({"id":id,"pseud_id":pid,"work_id":wid,"series_id":sid,"approved":appr})).collect();
    Ok(Json(json!({"err":0,"creatorships":items})))
}
```

### Breakdown

**Invite by ID or name**: `invite_creatorship` accepts either `pseud_id` (the numeric ID) or `pseud_name` (a case-insensitive lookup in the `pseuds` table). The `ON CONFLICT DO NOTHING` means inviting the same pseud twice for the same work just silently skips the duplicate — no error.

**Pending until approved**: The `creatorships` table stores `approved boolean DEFAULT false`. The invite sits in limbo until the *pseud's owner* calls approve. A pending row has `approved=false` and `approved_at=NULL`.

**Ownership check on approve/reject**: Both `approve_creatorship` and `reject_creatorship` join `creatorships` → `pseuds` → filter by `p.user_id=$1`. Only the person who *owns* the invited pseud can approve or reject it. If you try to approve a creatorship for a pseud that isn't yours, you get a 404.

**The `creatorships` table** (from `migrations/001_initial.sql`):

```sql
CREATE TABLE public.creatorships (
    id integer NOT NULL,
    pseud_id integer NOT NULL,
    work_id integer,
    series_id integer,
    approved boolean DEFAULT false NOT NULL,
    invited_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_at timestamp with time zone,
    CONSTRAINT creatorships_check CHECK (((work_id IS NOT NULL) OR (series_id IS NOT NULL)))
);
-- One pseud can co-author a given work only once
ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pseud_id_work_id_key UNIQUE (pseud_id, work_id);
ALTER TABLE ONLY public.creatorships
    ADD CONSTRAINT creatorships_pseud_id_series_id_key UNIQUE (pseud_id, series_id);
```

The `CHECK` constraint enforces that a creatorship must point to *either* a work *or* a series — never both, never neither. The unique constraints prevent double-inviting.

**Routes registered in `src/server.rs`** (lines 670-677):

```rust
.route("/api/pseuds", get(crate::routes::pseuds::list_pseuds))
.route("/api/pseuds", axum::routing::post(crate::routes::pseuds::create_pseud))
.route("/api/pseuds/{id}", axum::routing::patch(crate::routes::pseuds::update_pseud))
.route("/api/pseuds/{id}", axum::routing::delete(crate::routes::pseuds::delete_pseud))
.route("/api/pseuds/creatorships", get(crate::routes::pseuds::list_creatorships))
.route("/api/pseuds/invite", axum::routing::post(crate::routes::pseuds::invite_creatorship))
.route("/api/pseuds/creatorships/{id}/approve", axum::routing::post(crate::routes::pseuds::approve_creatorship))
.route("/api/pseuds/creatorships/{id}/reject", axum::routing::post(crate::routes::pseuds::reject_creatorship))
```

Notice the `GET /api/pseuds/creatorships` route must be declared *before* the dynamic `PATCH/DELETE /api/pseuds/{id}` route — otherwise Axum would match `"creatorships"` as the `{id}` parameter. Route ordering matters here.

---

## 25.4 Frontend: pseud manager, creatorship requests

There's no `pseuds.ts` API client yet, so we build one using the same `request()` helper pattern as `social.ts`. Then we wire up a Svelte page that manages pseuds and shows creatorship invites.

First, the API client (`frontend/src/lib/api/pseuds.ts`):

```typescript
// frontend/src/lib/api/pseuds.ts
import { request } from '$lib/api/social'; // reuse the shared request() helper

export interface Pseud {
  id: number;
  name: string;
  description: string;
  is_default: boolean;
  created_at: string;
}

export interface Creatorship {
  id: number;
  pseud_id: number;
  work_id: number | null;
  series_id: number | null;
  approved: boolean;
}

/** GET /api/pseuds — list the caller's pseuds */
export async function listPseuds(): Promise<{ err: number; pseuds: Pseud[] }> {
  return request('/pseuds');
}

/** POST /api/pseuds — create a new pseud */
export async function createPseud(
  name: string,
  description?: string,
  isDefault?: boolean,
): Promise<{ err: number; pseud: { id: number; name: string }; msg?: string }> {
  return request('/pseuds', {
    method: 'POST',
    body: JSON.stringify({
      name,
      description,
      is_default: isDefault,
    }),
  });
}

/** PATCH /api/pseuds/{id} — rename, re-describe, or promote to default */
export async function updatePseud(
  id: number,
  patch: { name?: string; description?: string; is_default?: boolean },
): Promise<{ err: number; msg?: string }> {
  return request(`/pseuds/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(patch),
  });
}

/** DELETE /api/pseuds/{id} — remove a pseud */
export async function deletePseud(id: number): Promise<{ err: number; msg?: string }> {
  return request(`/pseuds/${id}`, { method: 'DELETE' });
}

/** GET /api/pseuds/creatorships — list invites for the caller's pseuds */
export async function listCreatorships(): Promise<{ err: number; creatorships: Creatorship[] }> {
  return request('/pseuds/creatorships');
}

/** POST /api/pseuds/invite — invite a pseud to co-author */
export async function inviteCreatorship(payload: {
  pseud_id?: number;
  pseud_name?: string;
  work_id?: number;
  series_id?: number;
}): Promise<{ err: number; msg?: string }> {
  return request('/pseuds/invite', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

/** POST /api/pseuds/creatorships/{id}/approve */
export async function approveCreatorship(id: number): Promise<{ err: number; msg?: string }> {
  return request(`/pseuds/creatorships/${id}/approve`, { method: 'POST' });
}

/** POST /api/pseuds/creatorships/{id}/reject */
export async function rejectCreatorship(id: number): Promise<{ err: number; msg?: string }> {
  return request(`/pseuds/creatorships/${id}/reject`, { method: 'POST' });
}
```

Now the page (`frontend/src/routes/pseuds/+page.svelte`):

```svelte
<!-- frontend/src/routes/pseuds/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import {
    listPseuds,
    createPseud,
    updatePseud,
    deletePseud,
    listCreatorships,
    inviteCreatorship,
    approveCreatorship,
    rejectCreatorship,
    type Pseud,
    type Creatorship,
  } from '$lib/api/pseuds';

  let pseuds = $state<Pseud[]>([]);
  let creatorships = $state<Creatorship[]>([]);
  let loading = $state(true);
  let msg = $state('');

  // New pseud form
  let newName = $state('');
  let newDescription = $state('');
  let creating = $state(false);

  // Invite form
  let invitePseudName = $state('');
  let inviteWorkId = $state('');
  let inviting = $state(false);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      // Redirect to login — pseuds are user-only
      location.href = '/login';
      return;
    }
    await load();
  });

  async function load() {
    loading = true;
    msg = '';
    try {
      const p = await listPseuds();
      if (p.err === 0) pseuds = p.pseuds;
      const c = await listCreatorships();
      if (c.err === 0) creatorships = c.creatorships;
    } catch (e) {
      msg = `⚠️ ${e instanceof Error ? e.message : 'Load failed'}`;
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    if (!newName.trim()) {
      msg = '⚠️ Name is required';
      return;
    }
    creating = true;
    try {
      const res = await createPseud(newName, newDescription || undefined);
      if (res.err === 0) {
        msg = '✅ Pseud created';
        newName = '';
        newDescription = '';
      } else {
        msg = `⚠️ ${res.msg || 'Create failed'}`;
      }
      await load();
    } catch (e) {
      msg = `⚠️ ${e instanceof Error ? e.message : 'Create failed'}`;
    } finally {
      creating = false;
    }
  }

  async function handleSetDefault(pseud: Pseud) {
    if (pseud.is_default) return;
    try {
      await updatePseud(pseud.id, { is_default: true });
      // Optimistically flip the default flags locally
      pseuds = pseuds.map(p => ({ ...p, is_default: p.id === pseud.id }));
    } catch (e) {
      msg = `⚠️ ${e instanceof Error ? e.message : 'Failed to set default'}`;
    }
  }

  async function handleDelete(pseud: Pseud) {
    if (!confirm(`Delete pseud "${pseud.name}"? This cannot be undone.`)) return;
    try {
      await deletePseud(pseud.id);
      pseuds = pseuds.filter(p => p.id !== pseud.id);
    } catch (e) {
      msg = `⚠️ ${e instanceof Error ? e.message : 'Delete failed'}`;
    }
  }

  async function handleInvite() {
    const workId = inviteWorkId ? parseInt(inviteWorkId, 10) : undefined;
    if (!invitePseudName.trim() || !workId) {
      msg = '⚠️ Pseud name and work ID are required';
      return;
    }
    inviting = true;
    try {
      const res = await inviteCreatorship({ pseud_name: invitePseudName, work_id: workId });
      msg = `✅ ${res.msg}`;
      invitePseudName = '';
      inviteWorkId = '';
    } catch (e) {
      msg = `⚠️ ${e instanceof Error ? e.message : 'Invite failed'}`;
    } finally {
      inviting = false;
    }
  }

  async function handleApprove(c: Creatorship) {
    await approveCreatorship(c.id);
    creatorships = creatorships.map(x =>
      x.id === c.id ? { ...x, approved: true } : x,
    );
  }

  async function handleReject(c: Creatorship) {
    await rejectCreatorship(c.id);
    creatorships = creatorships.filter(x => x.id !== c.id);
  }
</script>

{#if auth.level >= 50}
  <p class="muted">Curator view: managing pseuds and co-creatorships for your account.</p>
{/if}

{#if msg}<p class="form-msg">{msg}</p>{/if}

<!-- Pseud manager -->
<div class="pseud-manager card">
  <h2>Your Pseuds</h2>

  <form onsubmit={(e) => { e.preventDefault(); handleCreate(); }} class="edit-row">
    <input bind:value={newName} placeholder="Pseud name" maxlength="40" />
    <input bind:value={newDescription} placeholder="Description (optional)" maxlength="200" />
    <button type="submit" disabled={creating} class="btn btn-primary">
      {creating ? 'Creating…' : 'Create'}
    </button>
  </form>

  {#if loading}
    <p>Loading pseuds…</p>
  {:else}
    <ul class="pseud-list">
      {#each pseuds as pseud (pseud.id)}
        <li class="pseud-item">
          <span class="pseud-name">
            {pseud.name}
            {#if pseud.is_default}<span class="badge">default</span>{/if}
          </span>
          <span class="pseud-desc">{pseud.description || '—'}</span>
          <div class="pseud-actions">
            {#if !pseud.is_default}
              <button class="btn btn-small" onclick={() => handleSetDefault(pseud)}>Set default</button>
            {/if}
            <button class="btn btn-small btn-red" onclick={() => handleDelete(pseud)}>Delete</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<!-- Creatorships -->
<div class="creatorship-manager card">
  <h2>Creatorship Requests</h2>

  <form onsubmit={(e) => { e.preventDefault(); handleInvite(); }} class="edit-row">
    <input bind:value={invitePseudName} placeholder="Pseud name to invite" />
    <input bind:value={inviteWorkId} type="number" placeholder="Work ID" min="1" />
    <button type="submit" disabled={inviting} class="btn btn-primary">
      {inviting ? 'Inviting…' : 'Invite'}
    </button>
  </form>

  {#if creatorships.length === 0}
    <p class="muted">No creatorships yet.</p>
  {:else}
    <ul class="creatorship-list">
      {#each creatorships as c (c.id)}
        <li class="creatorship-item">
          <span class="creatorship-target">
            Work #{c.work_id}{#if c.series_id}, Series #{c.series_id}{/if}
          </span>
          <span class={'badge ' + (c.approved ? 'approved' : 'pending')}>
            {c.approved ? 'Approved' : 'Pending'}
          </span>
          {#if !c.approved}
            <div class="creatorship-actions">
              <button class="btn btn-small btn-green" onclick={() => handleApprove(c)}>Approve</button>
              <button class="btn btn-small btn-red" onclick={() => handleReject(c)}>Reject</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .pseud-manager, .creatorship-manager { max-width: 720px; margin: 1rem auto; }
  .pseud-list { list-style: none; padding: 0; }
  .pseud-item { display: flex; align-items: center; gap: 1rem; padding: 0.5rem 0; border-bottom: 1px solid #eee; }
  .pseud-name { font-weight: 600; }
  .badge { font-size: 0.75em; padding: 0.1rem 0.4rem; border-radius: 3px; margin-left: 0.3rem; }
  .badge.default { background: #f0f0f0; color: #666; }
  .btn-red { background: #fee; color: #900; }
  .btn-green { background: #efe; color: #060; }
  .form-msg { margin: 0.5rem 0; font-size: 0.9em; }
  .creatorship-item { display: flex; align-items: center; gap: 1rem; padding: 0.5rem 0; border-bottom: 1px solid #eee; }
</style>
```

### Breakdown

**API client (`pseuds.ts`)**: Reuses the `request()` helper from `social.ts` — which already handles JWT attachment and 401 redirects. Each function maps to one backend route and returns a typed response.

**Auth gating**: The page calls `auth.init()` on mount and redirects to `/login` if the user isn't logged in. Pseuds are user-only — no anonymous access.

**Pseud list**: Fetched from `GET /api/pseuds`, rendered with a badge for the default pseud and buttons to set default (skipped for the current default) or delete.

**Creatorship list**: Fetched from `GET /api/pseuds/creatorships`. Each item shows the work/series target, an Approved/Pending badge, and — for pending invites — Approve and Reject buttons. The approve/reject handlers optimistically update the local list without a full reload.

---

## 25.5 Try It Yourself: manage pseuds and co-creatorship

### Step 1: Log in and create a pseud

```bash
curl -s -X POST http://localhost:8000/api/pseuds \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"name": "Starlight Weaver", "description": "My fantasy pen name"}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "pseud": {"id": 2, "name": "Starlight Weaver"}}`

### Step 2: List your pseuds

```bash
curl -s http://localhost:8000/api/pseuds \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: A list with your default pseud (your username) and the new one, `is_default` set on the right row.

### Step 3: Set the new pseud as default

```bash
curl -s -X PATCH http://localhost:8000/api/pseuds/2 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"is_default": true}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "Pseud updated"}`

Re-list to confirm the flags swapped.

### Step 4: Invite a co-creator

```bash
curl -s -X POST http://localhost:8000/api/pseuds/invite \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"pseud_name": "Starlight Weaver", "work_id": 1}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "Invite sent — pending approval"}`

### Step 5: List creatorships

```bash
curl -s http://localhost:8000/api/pseuds/creatorships \
  -H "Authorization: Bearer ***" | python3 -m python3 -m json.tool
```

**Expected**: A `creatorships` array with the invite row, `approved: false`.

### Step 6: Set the other pseud as default, then delete the original

```bash
curl -s -X PATCH http://localhost:8000/api/pseuds/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"is_default": true}'

curl -s -X DELETE http://localhost:8000/api/pseuds/2 \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: Both return `{"err": 0, ...}`. Deleting pseud 2 succeeds because it's no longer the default.

### Step 7: Try the frontend page

1. Start the dev server (`cd frontend && npm run dev`).
2. Log in and navigate to `/pseuds`.
3. Create a pseud, set it as default, then visit `/pseuds` again — confirm the list updates.

---

## 25.6 Troubleshooting

**"Login required" (401)**
Make sure you're sending `Authorization: Bearer <token>`. Without a valid JWT, the `AuthUser` extractor returns `None` and `require_user` rejects the request. Get a token from `POST /api/auth/login` first.

**"Pseud name too long (max 40)"**
The `sanitize_pseud` helper enforces a 19-character limit. Shorten the name and try again.

**"Cannot delete default pseud; set another as default first"**
You hit the safety guard in `delete_pseud`. Promote another pseud to default first with `PATCH /api/pseuds/{id}` setting `is_default: true`, then delete the old one.

**"Invite not found or not yours" (404 on approve)**
Only the owner of the invited pseud can approve. The route joins through `pseuds` and filters by `p.user_id`, so a second user calling approve gets a 404 (not a 403 — no information leak).

**Frontend page shows "Loading…" forever**
Check that `auth.init()` succeeded. If the JWT in localStorage is expired or invalid, the `/api/pseuds` request will 401 and the shared `request()` helper in `social.ts` will clear the token and redirect to `/login`. Make sure the page isn't silently redirecting.

---

## 25.7 What you have now

- You understand pseuds: a per-user alias for pen names, with one default per user and unique names per user.
- You understand the pseuds table schema: `user_id`, `name`, `description`, `is_default`, unique constraint on `(user_id, name)`.
- You understand pseud CRUD: list (default-first ordering), create (with auto-promote on first pseud), update (name/description/default swap), delete (guarded default-pseud deletion).
- You understand `sanitize_pseud`: trim + 19-char validation, with unit tests.
- You understand creatorships: a join table linking pseuds to works or series, pending until approved.
- You understand the invite/approve/reject flow: invite by ID or name, `ON CONFLICT DO NOTHING` dedup, ownership-scoped approve/reject via the pseud join.
- You understand route registration ordering: `/api/pseuds/creatorships` must precede `/api/pseuds/{id}` in the router.
- You understand the frontend: API client reusing the `social.ts` `request()` helper, and a Svelte page with pseud management and creatorship requests.
- You tested the full CRUD + creatorship flow with curl.

Next: Part 26 — Skins and Customization. You will build user skin selection and per-work skin assignment.

---

*End of Part 25. On to [Part 26 — Skins and Customization](./26-skins/26-skins.md).*

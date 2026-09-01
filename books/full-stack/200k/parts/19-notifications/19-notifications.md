# Part 19 — Notifications

> In this chapter you will learn how FicHub's notification system works — the notification bell, notification list with pagination, read-all, unread-count, and granular notification preferences (AO3-style toggles migrated in migration 060).

---

## Overview

FicHub's notification system (`src/routes/notifications.rs`, 172 lines):

- **List**: `GET /api/v1/notifications` — paginated list with `limit`/`offset`
- **Read**: `POST /api/v1/notifications/{id}/read` — mark one read
- **Read all**: `POST /api/v1/notifications/read-all` — mark everything read
- **Unread count**: `GET /api/v1/notifications/unread-count` — badge number
- **Preferences**: `GET/PUT /api/v1/notifications/preferences` — granular AO3-style toggles

Notification types: `follow_update`, `work_update`, `comment_reply`, `kudos_on_work`, `badge_earned`, `level_up`, `curator_promotion`, `recommendation`.

---

## Chapter 19.1 — The Notification Bell

### Goal

Build the NotificationBell component that polls for unread count and displays a badge.

### Actions

```svelte
<!-- frontend/src/lib/components/NotificationBell.svelte (1886 chars) -->
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { goto } from '$app/navigation';
    import { authHeaders } from '$lib/api/client';

    let unreadCount = $state(0);
    let interval: ReturnType<typeof setInterval> | null = null;

    async function fetchUnread() {
        if (!auth.isLoggedIn) return;
        try {
            const res = await fetch('/api/v1/notifications/unread-count', {
                headers: authHeaders(),
            });
            const data = await res.json();
            if (data.err === 0) unreadCount = data.unread_count;
        } catch { /* silent fail on bell */ }
    }

    onMount(() => {
        void fetchUnread();
        interval = setInterval(fetchUnread, 30000);  // poll every 30s
    });

    onDestroy(() => { if (interval) clearInterval(interval); });
</script>

<button class="notif-bell" aria-label="Notifications" onclick={() => goto('/notifications')}>
    <span class="bell-icon">🔔</span>
    {#if unreadCount > 0}
        <span class="notif-badge">{unreadCount > 99 ? '99+' : unreadCount}</span>
    {/if}
</button>

<style>
.notif-bell { position: relative; background: none; border: none; cursor: pointer; }
.notif-badge {
    position: absolute; top: -4px; right: -6px;
    background: #e74c3c; color: white; border-radius: 999px;
    padding: 2px 6px; font-size: 0.7rem; font-weight: bold;
}
</style>
```

> **💡 Key Concept**: The bell polls every 30 seconds via `setInterval` — not real-time (no WebSocket), but cheap enough for the unread-count endpoint (a single `COUNT` query). The badge shows `99+` for very high counts to avoid overflow. The bell navigates to `/notifications` when clicked.

### Try It Yourself

```typescript
// Add sound on new notification (compare prev vs current count)
let prevCount = 0;
async function fetchUnread() {
    const data = await (await fetch('/api/v1/notifications/unread-count', { headers: authHeaders() })).json();
    if (data.unread_count > prevCount && prevCount > 0) {
        // Play a subtle "ding" sound
        new Audio('/sounds/notif.mp3').play().catch(() => {});
    }
    prevCount = data.unread_count;
    unreadCount = data.unread_count;
}
```

### Check

- ✅ Polls `GET /api/v1/notifications/unread-count` every 30 seconds.
- ✅ Badge shows count (capped at 99+).
- ✅ `authHeaders()` for authenticated requests.
- ✅ `onDestroy` clears the interval (prevents memory leaks).
- ✅ Clicks navigate to `/notifications`.

### What you built

The notification bell — polling unread count, badge display with 99+ cap, auth-gated, cleanup on destroy.

---

## Chapter 19.2 — Notifications List and Preferences

### Goal

Build the notifications page and preference management with AO3-style granular toggles.

### Actions

#### 1. GET /api/v1/notifications — paginated list

```rust
// src/routes/notifications.rs (lines 19-49)
pub async fn list_notifications_handler(
    auth: AuthUser, State(state): State<Arc<AppState>>,
    Query(params): Query<NotifQueryParams>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let limit = params.limit.unwrap_or(20).min(50).max(1);
    let offset = params.offset.unwrap_or(0).max(0);

    let notifs = queries::list_notifications(&state.db, user_id, limit, offset).await?;
    let unread_count = queries::get_unread_notification_count(&state.db, user_id).await?;

    let items: Vec<Value> = notifs.into_iter().map(|n| json!({
        "id": n.id,
        "notification_type": n.notification_type,
        "title": n.title,
        "body": n.body,
        "link": n.link,
        "reference_type": n.reference_type,
        "reference_id": n.reference_id,
        "is_read": n.is_read,
        "created_at": n.created_at.to_rfc3339(),
    })).collect();

    Ok(Json(json!({ "err": 0, "notifications": items, "unread_count": unread_count })))
}
```

#### 2. Mark read + read-all

```rust
// POST /api/v1/notifications/{id}/read
pub async fn mark_notification_read_handler(...) -> ... {
    queries::mark_notification_read(&state.db, user_id, notif_id).await?;
    Ok(Json(json!({ "err": 0, "marked": marked })))
}

// POST /api/v1/notifications/read-all
pub async fn mark_all_read_handler(...) -> ... {
    queries::mark_all_notifications_read(&state.db, user_id).await?;
    Ok(Json(json!({ "err": 0, "msg": "All marked read" })))
}
```

#### 3. Preferences — AO3-style granular toggles

```rust
// src/routes/notifications.rs (lines 92-172)
pub struct PrefsBody {
    // Coarse toggles (F1 era)
    pub comment_reply: Option<bool>,
    pub follow_update: Option<bool>,
    pub work_update: Option<bool>,
    pub badge_earned: Option<bool>,
    pub curator_promotion: Option<bool>,
    pub recommendation: Option<bool>,
    pub email_digest: Option<String>,  // "instant" | "daily" | "weekly" | "never"
    // Granular AO3-style toggles (migration 060)
    pub comments_on_work: Option<bool>,
    pub replies_to_comments: Option<bool>,
    pub kudos_on_work: Option<bool>,
    pub bookmarks_on_work: Option<bool>,
    pub follows: Option<bool>,
    pub mentions: Option<bool>,
}

// PUT /api/v1/notifications/preferences
// Validates email_digest against whitelist: ["instant", "daily", "weekly", "never"]
if let Some(ref d) = body.email_digest {
    if !["instant", "daily", "weekly", "never"].contains(&d.as_str()) {
        return Err(AppError::BadRequest("email_digest must be one of: instant, daily, weekly, never".into()));
    }
}
// Partial update: only non-None fields are modified
```

> **⚠️ Watch Out**: Preferences use a **partial update** pattern — only non-`None` fields are modified. Calling `PUT` with `{"kudos_on_work": false}` keeps all other preferences unchanged. The `email_digest` field is validated against a 4-value whitelist.

### Try It Yourself

```typescript
// Fetch + update preferences in one call
async function loadPrefs() {
    const res = await fetch('/api/v1/notifications/preferences', { headers: authHeaders() });
    const data = await res.json();
    return data.preferences;
}

async function savePref(key: string, value: boolean | string) {
    await fetch('/api/v1/notifications/preferences', {
        method: 'PUT', headers: authHeaders(),
        body: JSON.stringify({ [key]: value }),
    });
}
```

### Check

- ✅ `limit` capped at 50, `offset` non-negative.
- ✅ `unread_count` included in list response (avoids extra request).
- ✅ Partial update: only non-None fields modified.
- ✅ `email_digest` validated against 4-value whitelist.
- ✅ Granular toggles: `comments_on_work`, `kudos_on_work`, `bookmarks_on_work`, `follows`, `mentions`.

### What you built

The notifications page + preferences — paginated notification list with unread-count, mark-read/read-all, and AO3-style granular preference toggles with email_digest validation and partial updates.

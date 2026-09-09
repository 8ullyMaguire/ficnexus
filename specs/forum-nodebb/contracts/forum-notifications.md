# Contract: Forum Notifications & Digests

**Route Prefix**: `/api/forum/notifications`  
**Spec References**: FR-143..158, FR-042 (mentions), FR-043 (watch levels)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level` (authenticated only)

---

## 1. Notification Types

| Type | Trigger | Payload `reference_type` | WS Event |
|------|---------|--------------------------|----------|
| `mention` | @username in post/message | `forum_post` \| `forum_message` | `notification` |
| `reply` | Reply to followed topic | `forum_post` | `notification` |
| `new_topic` | New topic in watched category | `forum_topic` | `notification` |
| `follow` | User follows your topic | `forum_topic` | `notification` |
| `poll_vote` | Vote on your poll | `forum_poll` | `notification` |
| `flag_resolved` | Your flag resolved | `user_report` | `notification` |
| `poll_closed` | Your poll closed | `forum_poll` | `notification` |
| `scheduled_published` | Your scheduled topic published | `forum_topic` | `notification` |
| `room_message` | New message in room (per notify_level) | `forum_message` | `notification` |
| `moderation` | Your post moderated | `forum_post` | `notification` |

---

## 2. List Notifications

### GET `/api/forum/notifications`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `unread_only` | bool | false |
| `type` | string | - (filter by type) |
| `limit` | int | 25 (max 100) |
| `cursor` | int64 | - (created_at ms) |

**Response 200**:
```json
{
  "err": 0,
  "notifications": [
    {
      "id": 1,
      "type": "mention",
      "title": "alice mentioned you",
      "body": "in \"Hello World\": \"@bob check this out\"",
      "link": "/forum/board/hello-world-42#post-107",
      "reference_type": "forum_post",
      "reference_id": 107,
      "actor": {"id": 7, "username": "alice", "avatar_url": "/avatars/7.png"},
      "is_read": false,
      "created_at": "2026-08-31T12:05:00Z"
    }
  ],
  "unread_count": 3,
  "next_cursor": 1725105600000,
  "has_more": true
}
```

**Auth**: Authenticated only (401).

---

## 3. Unread Count (Bell Badge)

### GET `/api/forum/notifications/unread-count`

**Response 200**:
```json
{"err": 0, "count": 3}
```

**Used by**: Frontend bell component (polls every 30s + WS push).

---

## 4. Mark Notifications Read

### POST `/api/forum/notifications/read`

**Request**:
```json
{"notification_ids": [1, 2, 3]}  // empty = mark all read
```

**Response 200**:
```json
{"err": 0, "marked": 3}
```

**Side Effects**: Updates `forum_notifications.is_read=true`, `read_at=NOW()`. WS `notification_read` broadcast (for multi-tab sync).

---

## 5. Notification Preferences

### GET `/api/forum/notifications/preferences`

**Response 200**:
```json
{
  "err": 0,
  "preferences": {
    "email_mentions": true,
    "email_replies": true,
    "email_new_topic_watched": false,
    "email_follows": false,
    "email_poll_votes": false,
    "email_digest": "daily",  // "none" | "daily" | "weekly"
    "push_enabled": true,
    "push_mentions": true,
    "push_replies": true,
    "push_new_topic": false
  }
}
```

### PUT `/api/forum/notifications/preferences`

**Request**: Same shape as GET response.

**Response 200**: Updated preferences.

**Storage**: `users.notification_prefs` JSONB (existing ficnexus column) — forum types merged in.

---

## 6. Watch Levels (Per-Category / Per-Topic)

### 5.1 Category Watch Level

#### GET `/api/forum/categories/{categoryId}/watch`

#### PUT `/api/forum/categories/{categoryId}/watch`

**Request/Response**:
```json
{"level": "watching"}  // "watching" | "tracking" | "muting" | "ignoring"
```

| Level | Notification | Unread Badge |
|-------|--------------|--------------|
| `watching` | ✓ (all new topics) | ✓ |
| `tracking` | ✗ | ✓ (topic appears in unread) |
| `muting` | ✗ | ✗ |
| `ignoring` | ✗ | ✗ (hidden from lists) |

---

### 5.2 Topic Watch Level

#### GET `/api/forum/topics/{topicId}/watch`

#### PUT `/api/forum/topics/{topicId}/watch`

**Request/Response**: Same as category.

**Override**: Topic watch level overrides category level for that topic.

---

## 7. Email Digests (Cron)

### Cron Job: `forum_scheduled_promote.rs` (runs daily/weekly)

**Schedule**: `FORUM_DIGEST_DAILY_HOUR` (default 8 AM UTC) for daily; Monday for weekly.

**Logic**:
1. Find users with `email_digest != 'none'` and `email_verified=true`
2. For each user, collect unread notifications since last digest sent
3. Deduplicate by `reference_type + reference_id` (latest wins)
4. Render HTML email (template: `templates/forum_digest.html`)
5. Send via ficnexus mailer (`create_notification` with `notification_type='email_digest'`)
6. Record `last_digest_sent_at` on user prefs

**Digest Email Template Variables**:
```handlebars
{{#each notifications}}
  <li>
    <a href="{{link}}">{{title}}</a>
    <span>{{type}} • {{format_date created_at}}</span>
  </li>
{{/each}}
```

---

## 8. Scheduled Notifications (`scheduled_at`)

**Used for**: Scheduled topic publish notifications.

**Flow**:
1. User creates topic with `scheduled_at` (future)
2. `forum_notifications` row created with `type='scheduled_published'`, `scheduled_at=topic.scheduled_at`, `is_read=false`
3. Cron (runs every minute) finds `scheduled_at <= NOW()` and `is_read=false`
4. For each: set `scheduled_at=NULL`, send WS `notification` + email if prefs enabled
5. Topic status changes from `scheduled` → `open` (same cron)

**Cron Query**:
```sql
UPDATE forum_notifications
SET scheduled_at = NULL, created_at = NOW()
WHERE scheduled_at IS NOT NULL AND scheduled_at <= NOW() AND is_read = FALSE
RETURNING *;
```

---

## 9. Realtime (WS)

**Channel**: `forum:notify:{userId}`

**Event**: `notification`
```json
{
  "type": "notification",
  "notification": {
    "id": 1,
    "type": "mention",
    "title": "...",
    "body": "...",
    "link": "...",
    "actor": {...},
    "created_at": "..."
  }
}
```

**Multi-tab Sync**: On `mark_read`, broadcasts `notification_read` with `notification_ids[]` to same user's other tabs.

---

## 10. VAPID Push (PWA)

### Subscribe

#### POST `/api/forum/notifications/push/subscribe`

**Request**: Web Push API `PushSubscription` JSON.

**Storage**: `forum_push_subscriptions` table (user_id, endpoint, p256dh, auth, created_at).

### Unsubscribe

#### DELETE `/api/forum/notifications/push/subscribe`

**Auth**: Authenticated.

**Cron Sender**: `forum_scheduled_promote.rs` uses `web-push` crate to send to all subscriptions on notification create (if `push_enabled` + type matches prefs).
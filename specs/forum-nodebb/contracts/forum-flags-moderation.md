# Contract: Forum Flags & Moderation

**Route Prefix**: `/api/forum/flags`, `/api/forum/moderation`  
**Spec References**: FR-063..078, FR-038..041 (rep hook), FR-079..085 (trust gates)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. Flag a Post/Topic (Report)

### POST `/api/forum/flags`

**Request**:
```json
{
  "target_type": "forum_post",  // "forum_post" | "forum_topic"
  "target_id": 107,
  "reason": "spam",  // "spam" | "off_topic" | "harassment" | "illegal" | "other"
  "details": "Additional context..."
}
```

**Trust Gate**: **TL2+** (`flag_weight(2)=1`). TL0/1 get 403: `"Flagging requires Member (L2) or higher — keep participating to rise."`

**Rate Limit**: 10 flags/hour per user (Redis `forum:rate:flag:{uid}`).

**Validation**:
- Target must exist, not deleted/hidden
- User cannot flag own content
- User cannot flag if blocked by target author (bidirectional block)

**Response 201**:
```json
{
  "err": 0,
  "flag": {
    "id": 1,
    "target_type": "forum_post",
    "target_id": 107,
    "reporter_id": 7,
    "reason": "spam",
    "details": "Additional context...",
    "weight": 1,  // flag_weight(reporter_trust_level)
    "status": "pending",  // "pending" | "auto_hidden" | "needs_admin" | "resolved"
    "created_at": "2026-08-31T12:00:00Z"
  }
}
```

**Auto-Thresholds** (configurable via env):
| Total Weight | Action |
|--------------|--------|
| ≥ 3 | `auto_status = 'auto_hidden'` — content hidden, author notified |
| ≥ 10 | `auto_status = 'needs_admin'` — escalated to admin queue |
| Resolved by TL5+ | `auto_status = 'resolved'` — `resolved_by`, `resolved_at` set |

**Side Effects**:
- `user_reports` table row created (target_type `forum_post`/`forum_topic`)
- WS `flag_update` broadcast to moderators (TL5+)
- If `auto_hidden`: WS `post_hidden` / `topic_hidden` to topic subscribers

---

## 2. Flag Queue (Moderator View)

### GET `/api/forum/moderation/queue`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `status` | string | `pending` \| `auto_hidden` \| `needs_admin` \| `resolved` |
| `limit` | int | 25 |
| `cursor` | int64 | - |

**Auth**: **TL5+** (`RESOLVE_MIN_TRUST=5`) or Curator (role ≥ 5).

**Response 200**:
```json
{
  "err": 0,
  "flags": [
    {
      "id": 1,
      "target_type": "forum_post",
      "target_id": 107,
      "target_preview": "This is spam content...",
      "target_author": {"id": 42, "username": "spammer", "trust_level": 1},
      "reporter": {"id": 7, "username": "alice", "trust_level": 3},
      "reason": "spam",
      "details": "...",
      "weight": 2,
      "status": "pending",
      "created_at": "2026-08-31T12:00:00Z",
      "resolved_by": null,
      "resolved_at": null
    }
  ],
  "next_cursor": 1725105600000,
  "has_more": false
}
```

---

## 3. Resolve Flag

### POST `/api/forum/flags/{flagId}/resolve`

**Request**:
```json
{
  "action": "hide",  // "hide" | "dismiss" | "delete" (admin only)
  "reason": "Confirmed spam"
}
```

**Auth**: **TL5+** or Curator (role ≥ 5). Admin for `delete`.

**Actions**:
| Action | Effect | Rep Award |
|--------|--------|-----------|
| `hide` | Sets `is_hidden=true` on target, `auto_status='resolved'` | Reporter: +10 (`FORUM_REPUTATION_FLAG_HELPFUL`) if flag was valid |
| `dismiss` | `auto_status='resolved'`, no content change | None |
| `delete` | Hard delete (admin only), modlog | None |

**Response 200**:
```json
{
  "err": 0,
  "flag": {
    "id": 1,
    "status": "resolved",
    "resolved_by": 7,
    "resolved_at": "2026-08-31T12:05:00Z"
  },
  "reputation_awarded": 10
}
```

**Reputation Hook**: If `action="hide"` and flag was valid (not dismissed), calls `hook.award(reporter_id, 10, "forum", "flag_helpful", "user_report", flag_id, {...})`.

**Modlog**: Every resolution → `modlog.record_json(..., action="flag_resolve", extra=[("action", action), ("reason", reason)])`.

---

## 4. Moderation Points (Slashdot-style — F5)

### 4.1 Get My Moderation Status

#### GET `/api/forum/moderation/status`

**Auth**: Any authenticated user.

**Response 200**:
```json
{
  "err": 0,
  "points_available": 3,
  "window_expires_at": "2026-09-01T00:00:00Z",
  "eligible": true,  // TL2+ and not banned
  "recent_actions": [
    {"post_id": 101, "delta": 1, "reason": "insightful", "created_at": "..."}
  ]
}
```

**Points System**:
- TL2+: 3 points / week (resets Monday 00:00 UTC)
- TL3+: 5 points / week
- TL4+: 7 points / week
- TL5+: 10 points / week
- Points expire weekly; unused points don't roll over

---

### 4.2 Moderate a Post (Spend Point)

#### POST `/api/forum/posts/{postId}/moderate`

**Request**:
```json
{"reason": "insightful"}  // "insightful" | "informative" | "funny" | "overrated" | "underrated" | "troll" | "flamebait" | "spam" | "offtopic"
```

**Auth**: TL2+ with points available.

**Reason → Delta Mapping**:
| Reason | Delta | Description |
|--------|-------|-------------|
| `insightful` | +1 | Quality contribution |
| `informative` | +1 | Factual value |
| `funny` | +1 | Humor |
| `underrated` | +1 | Undervalued post |
| `overrated` | -1 | Overvalued post |
| `troll` | -1 | Trolling |
| `flamebait` | -1 | Inflammatory |
| `spam` | -1 | Spam |
| `offtopic` | -1 | Off-topic |

**Response 200**:
```json
{
  "err": 0,
  "post_id": 101,
  "new_score": 4,
  "points_remaining": 2,
  "action_id": 15  // for metamod
}
```

**Side Effects**:
- `forum_post_votes` not used — score stored on `forum_posts.score` (new column, default 0)
- `moderation_actions` table: `(id, post_id, moderator_id, reason, delta, created_at)`
- Reputation hook: `moderation_helpful` (+5) if metamod votes "fair"
- WS `post_score_update` broadcast

---

### 4.3 View Moderation History (Public)

#### GET `/api/forum/posts/{postId}/moderations`

**Auth**: Any (public).

**Response 200**:
```json
{
  "err": 0,
  "actions": [
    {"moderator": "alice", "reason": "insightful", "delta": 1, "created_at": "..."},
    {"moderator": "bob", "reason": "overrated", "delta": -1, "created_at": "..."}
  ]
}
```

**Note**: Moderator usernames shown (not anonymized — metamod anonymizes).

---

## 5. Metamoderation (F6)

### 5.1 Get Metamod Queue

#### GET `/api/forum/metamod/queue`

**Auth**: TL3+ (random sample of recent actions).

**Response 200**:
```json
{
  "err": 0,
  "actions": [
    {
      "id": 15,
      "post_excerpt": "Great point about...",
      "moderator_action": "insightful",
      "delta": 1,
      "post_author": "charlie"
      // moderator username HIDDEN
    }
  ]
}
```

### 5.2 Vote on Metamod Action

#### POST `/api/forum/metamod/{actionId}/vote`

**Request**:
```json
{"verdict": "fair"}  // "fair" | "unfair" | "unsure"
```

**Auth**: TL3+.

**Outcome**:
- `fair`: Moderator gets `moderation_helpful` rep (+5), action stands
- `unfair`: Delta reversed, moderator loses point for next week
- `unsure`: No change

---

## 6. Admin Fast-Hide

### POST `/api/admin/forum/hide/{postId}`

**Auth**: Curator (role ≥ 5).

**Effect**: `is_hidden=true` for 72h (auto-expiry via `deleted_at` logic or cron).

---

## 7. Topic Lock/Pin (Admin)

### POST `/api/admin/forum/topics/{topicId}/lock`
### POST `/api/admin/forum/topics/{topicId}/pin`

**Auth**: Curator (role ≥ 5).

**Response 200**: Updated topic with new status.

---

## 8. Bans (Admin)

### POST `/api/admin/forum/bans`
```json
{"user_id": 42, "scope": "category", "category_id": 1, "reason": "Trolling", "expires_at": "2026-09-07T12:00:00Z"}
```
- `scope`: `forum` (all categories) | `category` (single)
- `expires_at`: null = permanent

### DELETE `/api/admin/forum/bans/{banId}` — Lift ban
### GET `/api/admin/forum/bans` — List all (active + expired)

**Auth**: Curator (scope ≤ own level) for create; Admin for global bans.
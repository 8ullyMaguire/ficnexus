# Contract: Forum Messaging (DMs + Group Rooms)

**Route Prefix**: `/api/forum/messaging`  
**Spec References**: FR-086..105, FR-106 (blocks)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. Rooms (DMs + Group Chats)

### 1.1 List My Rooms

#### GET `/api/forum/messaging/rooms`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `limit` | int | 25 |
| `cursor` | int64 | - (last_activity_at) |

**Response 200**:
```json
{
  "err": 0,
  "rooms": [
    {
      "id": 1,
      "name": "Project Discussion",
      "is_group": true,
      "creator": {"id": 7, "username": "alice"},
      "members": [
        {"id": 7, "username": "alice", "role": "owner"},
        {"id": 5, "username": "bob", "role": "member"}
      ],
      "last_message": {
        "id": 50,
        "body": "Let's meet tomorrow",
        "author": {"id": 5, "username": "bob"},
        "created_at": "2026-08-31T12:00:00Z"
      },
      "unread_count": 3,
      "is_muted": false,
      "notify_level": "all",
      "last_activity_at": "2026-08-31T12:00:00Z"
    },
    {
      "id": 2,
      "name": null,
      "is_group": false,
      "members": [
        {"id": 7, "username": "alice"},
        {"id": 42, "username": "charlie"}
      ],
      "last_message": {...},
      "unread_count": 0
    }
  ]
}
```

**Auth Matrix**: Authenticated only (401 if anonymous).

---

### 1.2 Get Room Messages (Paginated)

#### GET `/api/forum/messaging/rooms/{roomId}/messages`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `before` | int64 | - (message ID cursor for older) |
| `after` | int64 | - (message ID cursor for newer) |
| `limit` | int | 50 (max 100) |

**Response 200**:
```json
{
  "err": 0,
  "messages": [
    {
      "id": 50,
      "room_id": 1,
      "author": {"id": 5, "username": "bob", "trust_level": 2},
      "body": "Let's meet tomorrow",
      "payload": {},
      "edited_at": null,
      "created_at": "2026-08-31T12:00:00Z",
      "uploads": [],
      "mentions": [],
      "is_pinned": false
    }
  ],
  "next_cursor": 45,
  "prev_cursor": 55,
  "has_more": true
}
```

**Auth Rules**: Must be member of room (`forum_room_members` with `left_at IS NULL`). Blocked users cannot see each other's messages.

---

### 1.3 Create DM Room

#### POST `/api/forum/messaging/rooms/dm`

**Request**:
```json
{"user_id": 42}
```

**Rules**:
- Returns existing DM room if one exists between the two users
- Cannot DM self
- Cannot DM if either user has blocked the other

**Response 201**: Room object (same as list item).

---

### 1.4 Create Group Room

#### POST `/api/forum/messaging/rooms/group`

**Request**:
```json
{
  "name": "Study Group",
  "member_ids": [5, 42, 13]
}
```

**Trust Gate**: **TL2+** (create group room).

**Validation**:
- `name`: 1-100 chars
- `member_ids`: 2-50 users (creator auto-added as owner)
- All users must exist, not blocked by creator

**Response 201**: Room object with `is_group=true`.

---

### 1.5 Update Room

#### PATCH `/api/forum/messaging/rooms/{roomId}`

**Request** (all optional):
```json
{
  "name": "New Name",
  "is_muted": true,
  "notify_level": "mentions"  // "all" | "mentions" | "none"
}
```

**Auth Rules**:
- Group room: Owner/manager can rename; any member can mute/notify_level
- DM: Either participant can mute/notify_level

---

### 1.6 Room Membership

#### POST `/api/forum/messaging/rooms/{roomId}/invite`
```json
{"user_id": 13}
```
**Auth**: Owner/manager (group rooms only).

#### POST `/api/forum/messaging/rooms/{roomId}/join`
**Auth**: Invited users (group rooms); auto-join for DMs.

#### POST `/api/forum/messaging/rooms/{roomId}/leave`
**Auth**: Any member. Owner must transfer ownership first.

#### POST `/api/forum/messaging/rooms/{roomId}/members/{userId}/role`
```json
{"role": "moderator"}  // "owner" | "moderator" | "member"
```
**Auth**: Owner only.

#### DELETE `/api/forum/messaging/rooms/{roomId}/members/{userId}`
**Auth**: Owner/manager (remove); self (leave).

---

### 1.7 Pins

#### POST `/api/forum/messaging/rooms/{roomId}/pins/{messageId}`
#### DELETE `/api/forum/messaging/rooms/{roomId}/pins/{messageId}`
#### GET `/api/forum/messaging/rooms/{roomId}/pins`

**Auth**: Owner/manager (pin/unpin); all members (list).

---

## 2. Send Message

### POST `/api/forum/messaging/rooms/{roomId}/messages`

**Request**:
```json
{
  "body": "Hello @bob",
  "payload": {"type": "work", "id": 123},
  "reply_to": 45  // optional message ID to reply to
}
```

**Validation**:
- `body`: 1-10000 chars
- User must be active member (not left, not banned from room)
- Block check: sender cannot message if blocked by any recipient

**Trust Gate**: **TL1+** (send messages).

**Response 201**:
```json
{
  "err": 0,
  "message": {
    "id": 51,
    "room_id": 1,
    "author": {"id": 7, "username": "alice"},
    "body": "Hello @bob",
    "payload": {"type": "work", "id": 123},
    "reply_to": 45,
    "created_at": "2026-08-31T12:05:00Z",
    "uploads": [],
    "mentions": [{"id": 5, "username": "bob"}]
  }
}
```

**Side Effects**:
- Room `last_message_id`, `last_activity_at` updated
- Recipients notified (`forum_notifications` type `room_message` or `mention`)
- Mentions parsed
- WS `new_message` broadcast to room subscribers
- Search vector updated

---

## 3. Edit / Delete Message

### PATCH `/api/forum/messaging/messages/{messageId}`

**Request**: `{"body": "Edited message"}`

**Auth Rules**:
- Author: can edit within **15 min** (configurable `FORUM_MESSAGE_EDIT_WINDOW_MIN=15`)
- Owner/manager: can edit any anytime

### DELETE `/api/forum/messaging/messages/{messageId}`

**Auth Rules**: Same as edit. Soft delete (`deleted_at`).

---

## 4. Uploads in Messages

See `contracts/forum-uploads.md` — same endpoint, `message_id` in multipart form.

---

## 5. User Blocks (Cross-Forum)

### 5.1 Block User

#### POST `/api/forum/blocks`

**Request**:
```json
{"blocked_id": 42}
```

**Rules**:
- Cannot block self
- Cannot block staff (role ≥ 5)
- Bidirectional: blocked user cannot DM, mention, reply to blocker's topics/posts
- Existing DM room between them: both can still see history but cannot send new messages

**Response 201**:
```json
{"err": 0, "blocked": true, "blocker_id": 7, "blocked_id": 42}
```

### 5.2 Unblock User

#### DELETE `/api/forum/blocks/{blockedId}`

**Response 200**: `{"err": 0, "unblocked": true}`

### 5.3 List My Blocks

#### GET `/api/forum/blocks`

**Response 200**:
```json
{
  "err": 0,
  "blocks": [
    {"blocked_id": 42, "blocked_username": "charlie", "created_at": "2026-08-15T10:00:00Z"}
  ]
}
```

---

## 6. Unread Counts

### GET `/api/forum/messaging/unread`

**Response 200**:
```json
{"err": 0, "total_unread": 7, "rooms": [{"room_id": 1, "count": 3}, {"room_id": 2, "count": 4}]}
```

Used for bell badge + room list badges.

---

## 7. Realtime (WS)

**Channels**:
- `forum:room:{roomId}` — new_message, edit_message, delete_message, typing, presence
- `forum:notify:{userId}` — personal notifications (mentions, new DM)

**Events**:
| Event | Payload |
|-------|---------|
| `new_message` | `{room_id, message: {...}}` |
| `edit_message` | `{room_id, message_id, body, edited_at}` |
| `delete_message` | `{room_id, message_id}` |
| `typing_start` | `{room_id, user_id, username}` |
| `typing_stop` | `{room_id, user_id}` |
| `presence_join` | `{room_id, user_id}` |
| `presence_leave` | `{room_id, user_id}` |

**Typing**: Client sends `typing_start` on keystroke (debounced 500ms), server broadcasts, auto-expires 3s after last keystroke.
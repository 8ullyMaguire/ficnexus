# Contract: Forum WebSocket & SSE (Realtime)

**Route Prefix**: `/ws/forum`, `/api/forum/events`  
**Spec References**: FR-171..185, NFR-001 (WS + Redis pubsub)  
**Auth**: `AuthUser` via WS handshake — `user_id`, `role`, `trust_level`

---

## 1. WebSocket: `/ws/forum`

### 1.1 Connection

**Upgrade Request**:
```
GET /ws/forum HTTP/1.1
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: xxx
Sec-WebSocket-Protocol: bearer.<jwt_token>  // or query param ?token=xxx
```

**Auth Validation**:
- JWT validated on upgrade (same as HTTP `AuthUser` extractor)
- If invalid: HTTP 401, no WS upgrade
- Connection state: `user_id`, `role`, `trust_level`, `subscriptions: Set<String>`

**Redis Pubsub**: On connect, subscribe to:
- `forum:notify:{user_id}` (personal notifications)
- `forum:presence:{user_id}` (own presence for multi-tab)

---

### 1.2 Client → Server Messages

All messages: JSON `{type: string, payload: object}`.

| Type | Payload | Description |
|------|---------|-------------|
| `subscribe` | `{topic_id?: int, category_id?: int, room_id?: int}` | Subscribe to channel |
| `unsubscribe` | `{topic_id?: int, category_id?: int, room_id?: int}` | Unsubscribe |
| `typing_start` | `{topic_id: int}` | User started typing |
| `typing_stop` | `{topic_id: int}` | User stopped typing |
| `presence_ping` | `{topic_id: int}` | Heartbeat (every 30s) |
| `mark_read` | `{topic_id: int, post_id: int}` | Mark topic read |
| `fetch_missed` | `{topic_id: int, after_post_id: int}` | Reconnect catch-up |

**Subscribe Channels**:
- `topic:{topic_id}` → new_post, edit_post, delete_post, typing, presence
- `category:{category_id}` → new_topic, topic_status_change
- `room:{room_id}` → new_message, edit_message, delete_message, typing, presence

**Auto-Subscribe**: On `GET /api/forum/topics/{id}`, client auto-subscribes to `topic:{id}`.

---

### 1.3 Server → Client Messages

All messages: JSON `{type: string, payload: object}`.

| Type | Payload | Trigger |
|------|---------|---------|
| `welcome` | `{user_id, server_time, features: {ws: true, sse: true}}` | On connect |
| `new_post` | `{topic_id, post: {...}}` | Post created in subscribed topic |
| `edit_post` | `{topic_id, post_id, body, edited_at}` | Post edited |
| `delete_post` | `{topic_id, post_id}` | Post soft-deleted |
| `new_topic` | `{category_id, topic: {...}}` | Topic created in subscribed category |
| `topic_status` | `{topic_id, status}` | Lock/pin/archive |
| `new_message` | `{room_id, message: {...}}` | Message in subscribed room |
| `edit_message` | `{room_id, message_id, body, edited_at}` | Message edited |
| `delete_message` | `{room_id, message_id}` | Message deleted |
| `typing_start` | `{topic_id|room_id, user_id, username}` | User typing |
| `typing_stop` | `{topic_id|room_id, user_id}` | User stopped typing |
| `presence_join` | `{topic_id|room_id, user_id, username}` | User entered |
| `presence_leave` | `{topic_id|room_id, user_id}` | User left |
| `notification` | `{notification: {...}}` | New notification (personal) |
| `notification_read` | `{notification_ids: int[]}` | Mark read sync |
| `poll_vote` | `{poll_id, option_id, vote_counts: [...]}` | Poll vote update |
| `poll_closed` | `{poll_id}` | Poll closed |
| `post_score` | `{post_id, score, delta}` | Moderation point spent |
| `flag_update` | `{flag_id, status, weight}` | Flag status change (TL5+) |
| `unread_update` | `{topic_id, count}` | Unread count changed |
| `error` | `{code, message}` | Protocol error |

---

### 1.4 Typing Indicator Protocol

**Client**: On keystroke in composer (debounced 500ms), send `typing_start` if not already typing.
**Server**: 
- Sets Redis `forum:typing:{topic_id}:{user_id}` = "1" (TTL 3s)
- Broadcasts `typing_start` to other subscribers
- On `typing_stop` or TTL expiry, broadcasts `typing_stop`

**Client Display**: "Alice is typing..." → clears 3s after `typing_stop` or no new `typing_start`.

---

### 1.5 Presence Protocol

**On Subscribe to Topic/Room**:
- Server adds to `forum:presence:{user_id}` hash: `{topic_id: timestamp}`
- Broadcasts `presence_join` to other subscribers

**On Unsubscribe / Disconnect**:
- Removes from hash
- Broadcasts `presence_leave`

**Heartbeat**: Client sends `presence_ping` every 30s. Server updates timestamp. If no ping for 60s → treat as disconnect.

---

### 1.6 Reconnect Catch-Up

**Scenario**: WS disconnects (network, sleep), client reconnects.

**Client**: On reconnect, sends `fetch_missed` for each subscribed topic with `after_post_id = last_seen_post_id`.

**Server**: 
- Queries posts `WHERE topic_id = ? AND id > ? AND deleted_at IS NULL ORDER BY id ASC LIMIT 50`
- Sends batched `new_post` events (or single `missed_posts` with array)

**Fallback**: If >50 missed, client falls back to REST `GET /api/forum/topics/{id}?after=...`

---

## 2. SSE Fallback: `/api/forum/events`

### 2.1 Connection

```
GET /api/forum/events HTTP/1.1
Accept: text/event-stream
Authorization: Bearer <jwt>
```

**Auth**: Same JWT validation. Returns 401 if invalid.

**Headers**:
```
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
X-Accel-Buffering: no  (nginx)
```

---

### 2.2 Event Stream Format

Standard SSE:
```
event: new_post
data: {"topic_id": 42, "post": {...}}

event: notification
data: {"notification": {...}}

: heartbeat (every 15s)
```

**Events**: Same as WS server→client messages (except typing/presence — too chatty for SSE).

**Subscriptions**: Passed as query params: `/api/forum/events?topic=42&room=1&category=3`

---

### 2.3 Client Preference

**Auto-select**: Frontend tries WS first. On failure (proxy, firewall), falls back to SSE.
**Header**: `X-Transport: ws` or `sse` for debugging.

---

## 3. Redis Pubsub Channels

| Channel | Pattern | Publishers | Subscribers |
|---------|---------|------------|-------------|
| `forum:topic:{id}` | `forum:topic:*` | Post create/edit/delete, typing, presence | WS connections subscribed to topic |
| `forum:category:{id}` | `forum:category:*` | Topic create, status change | WS connections subscribed to category |
| `forum:room:{id}` | `forum:room:*` | Message create/edit/delete, typing, presence | WS connections subscribed to room |
| `forum:notify:{uid}` | `forum:notify:*` | Notification create | WS/SSE for user |
| `forum:broadcast` | (single) | Admin announcements, maintenance | All WS connections |

**Message Format** (published by route handlers):
```json
{"channel": "forum:topic:42", "event": "new_post", "payload": {...}}
```

**Fanout**: `ForumPubsub::publish(channel, event, payload)` → Redis PUBLISH.

**Single-Process Fallback**: If Redis unavailable, in-memory `broadcast` channel (tokio `broadcast::channel`) used. Logged warning.

---

## 4. Rate Limits (WS)

| Action | Limit | Window | Redis Key |
|--------|-------|--------|-----------|
| `subscribe` | 50 | 60s | `forum:rate:ws_sub:{uid}` |
| `typing_start` | 20 | 60s | `forum:rate:ws_typing:{uid}` |
| `fetch_missed` | 10 | 60s | `forum:rate:ws_catchup:{uid}` |

Exceeded → WS `error` with `code: "rate_limited"`, connection stays open.

---

## 5. Connection Lifecycle

```
Client                    Server
  |                         |
  |--- WS Upgrade + JWT --->|
  |<-- 101 Switching -------|
  |                         |
  |--- subscribe {topic:1}->|
  |<-- welcome -------------|
  |                         |
  |<-- new_post (live) -----|  (other user posts)
  |                         |
  |--- typing_start ------->|  (user types)
  |                         |-- Redis SETEX typing:1:7 3s -->
  |                         |-- PUBLISH topic:1 typing_start -->
  |<-- typing_start --------|  (broadcast to others)
  |                         |
  |--- (network drop) ------X
  |                         |
  |--- WS Upgrade + JWT --->|  (reconnect)
  |<-- 101 Switching -------|
  |                         |
  |--- fetch_missed ------->|  (after_post_id=105)
  |<-- new_post x3 ---------|  (missed posts)
  |                         |
```

---

## 6. Load Testing Targets

| Metric | Target |
|--------|--------|
| WS connect latency | < 100ms p95 |
| Typing broadcast | < 500ms p95 |
| New post fanout (100 subs) | < 2s p95 |
| Reconnect catch-up (10 missed) | < 500ms |
| Memory per connection | < 50 KB |
| Max connections (single process) | 10,000 |

---

## 7. Testing Commands

```bash
# WS test (wscat)
wscat -c "ws://localhost:3000/ws/forum" -H "Sec-WebSocket-Protocol: bearer.eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# Send subscribe
> {"type":"subscribe","payload":{"topic_id":1}}

# SSE test
curl -N -H "Accept: text/event-stream" -H "Authorization: Bearer <token>" \
  "http://localhost:3000/api/forum/events?topic=1"

# Redis monitor
redis-cli MONITOR | grep "forum:"
```
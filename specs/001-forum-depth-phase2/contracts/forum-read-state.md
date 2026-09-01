# Contract: Forum Read-State

**Source**: FR-001..003, FR-012

## GET /api/forum/topics?category=&cursor=&limit=

Auth: optional. For authenticated user, each item includes:

```json
{
  "id": 6,
  "status": "open",
  "last_post_id": 7,
  "last_activity_at": "2026-08-24T10:47:45Z",
  "unread": true,
  "last_read_post_id": 5
}
```

Anonymous: `unread` absent or false, `last_read_post_id` absent.

## GET /api/forum/topics/{id}

Same `unread`/`last_read_post_id` for auth user. List of posts includes `id` for cursor.

## POST /api/forum/topics/{id}/read

Auth required.

Request:
```json
{ "last_read_post_id": 7 }
```
If omitted, server uses `last_post_id` of topic.

Response: `200 { "ok": true, "last_read_post_id": 7 }`

Errors: 401 anonymous, 404 topic not found/deleted.

Upsert is idempotent. No `unread` count returned — client refetches list.


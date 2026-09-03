# Contract: Forum Pin/Lock

**Source**: FR-004..007, FR-013

## Topic list/detail includes status

```json
{ "id": 6, "status": "pinned", "title": "...", "category_id": 1 }
```
Values: `open` | `locked` | `pinned` | `archived` (archived not used in this slice).
Ordering: pinned first, then `last_activity_at DESC`.

## POST /api/admin/forum/topics/{id}/pin

Auth: curator (level ≥50) or admin (≥100).

Request: `{ "pinned": true }` (toggle; if omitted, toggles).

Response: `200 { "id": 6, "status": "pinned" }` or `open` when unpinned.

## POST /api/admin/forum/topics/{id}/lock

Same gate.

Request: `{ "locked": true }`

Response: `200 { "id": 6, "status": "locked" }`

## POST /api/forum/topics/{id}/posts (enforcement)

If topic `status='locked'` and caller not curator/admin → `403 { "error": "topic_locked" }`.

Frontend MUST hide/disable reply when `status='locked'` for non-curators.

Errors: 401 anonymous, 403 not curator, 404 topic not found.


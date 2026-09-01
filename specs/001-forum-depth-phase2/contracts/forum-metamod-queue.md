# Contract: Forum Metamod Queue

**Source**: FR-008..010

## GET /api/forum/metamod/queue?filter=&verdict=&cursor=&limit=

Auth: curator.

Query:
- `filter`: `unreviewed` (default) | `reviewed` | `all`
- `verdict`: `fair` | `unfair` (when filter=reviewed)
- `cursor`, `limit` pagination

Response:
```json
{
  "items": [{ "grant_id": 1, "post_id": 2, "reason": "...", "created_at": "...", "my_verdict": null }],
  "next_cursor": "10"
}
```

`unreviewed` hides grants where caller already voted. `reviewed` shows caller's past votes.

## GET /api/forum/metamod/grants/{id}

Auth: curator. Returns anonymized context (no voter identity).

## POST /api/forum/metamod/grants/{id}/verdict

Auth: curator.

Request: `{ "verdict": "fair" }` // or "unfair"

Response: `200 { "grant_id": 1, "verdict": "fair" }`

Errors: 409 already voted by caller, 403 not curator, 404 grant not found.

## GET /api/forum/moderation/status (unfair-rate)

Returns existing rolling unfair-rate + cooldown; unchanged contract, surfaced in mod drawer.


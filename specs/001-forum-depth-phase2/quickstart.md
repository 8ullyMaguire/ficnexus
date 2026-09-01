# Quickstart: Forum Depth Phase 2

**Feature**: 001-forum-depth-phase2

## Prerequisites

- Rust backend running on `http://localhost:8000` with `FRONTEND_DIR=/personal/.../frontend/build`
- Two users: A (reader), B (curator level ≥50). Create via normal register + level bump in DB.
- `cargo test` and `npm run build` gates green.

## Validation Scenarios

### 1. Read-state badges

1. As B, `GET /api/forum/topics?category=general` — note `unread` false/absent for all.
2. As A, `POST /api/forum/topics { category_id, title, body }` → topic T.
3. As B, `GET /api/forum/topics?category=general` — T shows `unread: true`.
4. As B, `GET /api/forum/topics/{T}` then `POST /api/forum/topics/{T}/read { last_read_post_id: <last> }`.
5. As B, `GET /api/forum/topics?category=general` — T shows `unread: false`.
6. Anonymous `GET /api/forum/topics` — no `unread` field.

Frontend: badges render in `general/+page.svelte` and `TopicThread` for both skins; verify archive mode shows text badges, not emoji.

### 2. Pin/Lock

1. As B (curator), `POST /api/admin/forum/topics/{T}/pin { pinned: true }` → `status: pinned`.
2. `GET /api/forum/topics?category=general` — T first in list.
3. `POST /api/admin/forum/topics/{T}/lock { locked: true }` → `status: locked`.
4. As A (non-curator), `POST /api/forum/topics/{T}/posts { body }` → 403 `topic_locked`.
5. As B, same post → 200 (or 403 per policy — document actual).
6. Unlock, verify reply re-enabled.

Frontend: pin/locked badges visible; reply form disabled when locked for non-curators.

### 3. Metamod queue

1. Create 3 mod grants via `POST /api/forum/posts/{id}/moderate` (needs curator points).
2. As B, `GET /api/forum/metamod/queue?filter=unreviewed` — 3 items.
3. `POST /api/forum/metamod/grants/1/verdict { verdict: "fair" }` → 200.
4. Same verdict again → 409.
5. `GET /api/forum/metamod/queue?filter=reviewed` — 1 item, `GET ...?filter=unreviewed` — 2 items.
6. `GET /api/forum/moderation/status` — unfair-rate reflects vote.

## Commands

```bash
cargo test --lib forum
npm run build
npm test -- forum
# manual curl checks above, or qa/run.js forum journey
```

## Expected Outcomes

- SC-001..006 pass per spec.
- No console errors, no archive/modern divergence, no emoji.
- `svelte-check` 0 errors, `vitest` green.


# FicHub Real-Time & Live Features

## Summary

FicHub is a **hybrid**: most of the site is pure REST + polling, but the **forum** adds push-based live updates over WebSocket (with Redis pub/sub) and an SSE fallback. An alternative frontend needs both.

## Backend

### REST core (site-wide)

Every non-forum endpoint is request/response (Axum 0.8, no upgrades):

- Reader, search, bookmarks, ratings, recommendations, admin analytics — all REST.
- Background workers (Redis `BRPOP` bookmark-import worker, recommender training) are backend-to-backend, not client-visible.

### Forum realtime (push)

- **`GET /ws/forum`** — Axum `extract::ws::WebSocketUpgrade` (feature `ws`), auth via the same JWT cookie as HTTP. On connect the server subscribes (via `redis::aio` PubSub) to channels for the user's joined topics/categories/rooms + personal notification channel. On every topic/category/messaging mutation the handler `PUBLISH`es a JSON event to Redis; a pubsub task fans out to all WS peers on that channel and writes a `forum_notifications` row + pushes to the user channel.
- **Redis pub/sub keyspace:** `forum:topic:{id}`, `forum:category:{id}`, `forum:room:{id}`, `forum:typing:{topic|room}`, `forum:presence:{room|category}`, digest watermark `forum:digest:last_sent`, rep daily cap `forum:rep:{user_id}:{yyyy-mm-dd}`.
- **Typing indicators:** ephemeral `SETEX forum:typing:{topic}:{user} 3` + `typing_start`/`typing_stop` events (clients debounce 500 ms, clear after 3 s).
- **Presence:** `SADD forum:presence:{room}` + TTL heartbeat.
- **`GET /api/forum/stream?channel=forum:topic:{id}`** — SSE fallback for anonymous/read-only (no typing, no DM).
- **Degraded mode:** if Redis is unavailable the server falls back to a single-process `tokio::sync::broadcast` fanout (logged as degraded; health reports it).

## Frontend: polling vs push

Most non-forum pages still poll; forum pages use the WS (or SSE fallback).

### Polling (non-forum)

| Page / component | Interval | Endpoint | Notes |
|---|---|---|---|
| Admin dashboard (`/admin`) | 30 s | `GET /api/admin/realtime` | stats + realtime aggregates |
| Notification bell | 60 s | `GET /api/notifications/unread-count` | fallback when WS is disconnected; WS otherwise pushes |
| Comment triage (`/admin/comment-triage`) | 30 s | triage list | |
| Mod queue (`/admin/moderation`) | 60 s | queue list | |

Forum no longer polls for live posts/typing/presence/unread — those arrive over `/ws/forum`. The polls above remain for admin/system pages. See `docs/specs/forum-nodebb/spec.md` (FR-027..030) and `contracts/forum-ws-sse.md` for the full contract.

### Push (forum)

- `src/lib/forum/ws.ts` — WS client (reconnect + `?after={lastPostId}` cursor catch-up).
- `src/lib/forum/sse.ts` — SSE fallback (anonymous/read-only channels only).
- `src/lib/forum/composer.ts` — markdown preview, `@mention` autocomplete (`GET /api/users/search?q=prefix`), emoji picker, `/works {id}` fic card, drafts autosave.
- Events pushed: `new_post`, `typing_start`/`typing_stop`, `presence`, `notification`, `poll_close`, `room_message`.

## Features that appear real-time but aren't

- Search results, recommendations, trending — computed on-demand or from cache, not streamed.
- Non-forum comments — refresh on action.

## Design implications for alternative frontends

### What you always need
- HTTP client with `Authorization: Bearer …` injection (same JWT as WS auth).
- Polling infra (`setInterval`) for non-forum pages.
- Optional service worker for offline/caching (`/static/sw.js`).

### What you need for forum parity
- WebSocket client + SSE `EventSource` fallback.
- WS reconnect + `?after=` catch-up, channel subscriptions per open topic/room.
- Typing + presence UI state driven by WS events, not polling.

### Polling best practices (still apply outside forum)
1. Don't poll on every page — only pages that need live data.
2. Use different intervals (30 s admin, 60 s notifications).
3. Pause when tab hidden (`document.visibilitychange`).
4. Back off on errors (3 failures → 5 min interval).
5. Clear intervals on unmount.

## OPDS & RSS (protocols, not realtime)

- OPDS (`/opds/*`) and RSS/Atom feeds — static XML generated on demand, polled by e-readers.

# FicHub Real-Time & Live Features

## Summary

**FicHub uses NO push-based real-time mechanisms.** The backend is a pure REST API (Axum). All "live" features use client-side polling.

## Backend: Pure Request/Response

- **No WebSockets** — no `tungstenite`, `tokio-tungstenite`, or axum WebSocket handlers
- **No SSE (Server-Sent Events)** — no EventSource endpoints
- **No GraphQL subscriptions**
- **No long-polling** on the server side

The backend is a standard REST API. Every endpoint is request/response.

### Background Workers (Not Client-Facing)
- **Redis BRPOP worker** (`src/services/bookmark_import.rs`) — processes bookmark import jobs asynchronously. This is backend-to-backend, not exposed to clients.
- **Recommender training** — runs periodically, updates embeddings. Not real-time.

## Frontend: Polling Patterns

All "live" features use `setInterval` polling:

### 1. Admin Dashboard (30s poll)
```
Route: /admin
Poll: setInterval(loadAll, 30000)
Data: stats, SEO analytics, realtime aggregates
Endpoint: GET /api/admin/realtime
```

### 2. Notification Bell (60s poll)
```
Component: NotificationBell.svelte
Poll: setInterval(fetchCount, 60000)
Data: unread notification count
Endpoint: GET /api/notifications/unread-count
Response: { err: 0, count: 5 }
```

### 3. Comment Triage (30s poll)
```
Route: /admin/comment-triage
Poll: setInterval(loadAll, 30000)
Data: pending comments for moderation
```

### 4. Mod Queue (60s poll)
```
Route: /admin/moderation (implicit)
Poll: periodic refresh
Data: pending moderation items
```

## Features That Appear Real-Time But Aren't

- **Search results** — computed on-demand, not streamed
- **Recommendations** — pre-computed, served from DB
- **Trending** — computed periodically, served from cache
- **Forum posts** — standard REST, no live updates
- **Comments** — standard REST, refresh on action

## Design Implications for Alternative Frontends

### What You DON'T Need
- WebSocket client library
- SSE EventSource
- Real-time state synchronization
- Conflict resolution
- Connection management

### What You DO Need
- HTTP client with auth header injection
- Polling infrastructure (setInterval / timer)
- Optional: Service Worker for offline/caching (PWA support exists)

### Polling Best Practices
1. **Don't poll on every page** — only on pages that need live data (admin, notifications)
2. **Use different intervals** — 30s for admin, 60s for notifications
3. **Pause when tab is hidden** — use `document.visibilitychange` or Page Visibility API
4. **Back off on errors** — if poll fails 3x, increase interval to 5min
5. **Clean up on unmount** — clear intervals when component/page unmounts

## Future Real-Time Considerations

If you want to add real-time features (e.g., live forum updates, chat, collaborative editing), you would need:

### Backend Changes
- Add WebSocket handler in Axum: `axum::routing::get(ws_handler)`
- Or SSE endpoint: `axum::response::sse(Sse::new(stream))`
- Connection management (who's connected to what)
- Message broadcasting (Redis pub/sub or in-memory)

### Frontend Changes
- WebSocket client or EventSource
- Connection state management
- Reconnection logic
- Message handling

### Recommended Approach (If Adding Later)
1. Start with polling (current approach)
2. If latency matters, add SSE for specific endpoints (cheaper than WebSocket)
3. Use WebSocket only for bidirectional features (chat, collaborative editing)
4. Keep it optional — don't break the REST-only frontend contract

## OPDS & RSS (Protocols, Not Real-Time)

These are standard protocols for ebook readers, not real-time features:

- **OPDS** — Open Publication Distribution System, XML-based catalog for ebook apps
- **RSS** — Standard XML feed for new arrivals, followed works

Both are static XML generated on-demand. Ebook readers poll them periodically.

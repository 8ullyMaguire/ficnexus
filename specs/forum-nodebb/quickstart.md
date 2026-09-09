# Quickstart: Forum NodeBB Port

**Status**: Phase 1 output — run standalone or embedded
**Date**: 2026-08-31
**Branch**: `feat/forum-nodebb` (from `main`)

---

## 1. Prerequisites

```bash
# Rust 1.82+ (edition 2024)
rustup default stable

# Postgres 16+ (local or Docker)
# Redis 7+ (local or Docker)

# NFS-safe build: CARGO_TARGET_DIR must be on local disk
export CARGO_TARGET_DIR=/media/alvaro/code-worktrees

# NodeBB reference (read-only, for parity checks)
# ~/code/js/NodeBB  (rsync from /tmp/NodeBB, core.fsyncObjectFiles=false)
```

---

## 2. Environment Variables

Create `.env` (or export in shell):

```bash
# Database (required)
DATABASE_URL=postgres://ficnexus:password@localhost/ficnexus?sslmode=disable
REDIS_URL=redis://localhost:6379

# Forum-specific
FORUM_REPUTATION_TOPIC_CREATE=5
FORUM_REPUTATION_POST_CREATE=2
FORUM_REPUTATION_POLL_VOTE=1
FORUM_REPUTATION_FLAG_HELPFUL=10
FORUM_REPUTATION_MOD_HELPFUL=5
FORUM_REPUTATION_FORUM_DAILY_CAP=20

# Trust gates (override defaults in src/services/trust.rs)
PUBLISH_MIN_TRUST=2
RESOLVE_MIN_TRUST=5
FLAG_WEIGHT_TL1=1
FLAG_WEIGHT_TL2=1
FLAG_WEIGHT_TL3=2
FLAG_WEIGHT_TL4=3
FLAG_WEIGHT_TL5=5
FLAG_WEIGHT_TL6=5

# Uploads
FORUM_UPLOAD_MAX_MB=10
FORUM_UPLOAD_MAX_DIMENSION=1920
FORUM_UPLOAD_PATH=/public/uploads/forum

# WebSocket / Realtime
FORUM_WS_ENABLED=true
FORUM_SSE_FALLBACK=true
FORUM_REDIS_PUBSUB_ENABLED=true

# PWA / Push
VAPID_PUBLIC_KEY=BJxxx...
VAPID_PRIVATE_KEY=xxx...
VAPID_SUBJECT=mailto:admin@ficnexus.dev

# Scheduled topics cron (runs every minute)
FORUM_SCHEDULED_CRON=* * * * *

# Email digests (uses ficnexus mailer)
FORUM_DIGEST_DAILY_HOUR=8
FORUM_DIGEST_WEEKLY_DOW=1  # Monday

# AO3 Skin (frontend)
FORUM_THEME=ao3
```

---

## 3. Standalone Mode (forum_core crate only)

### 3.1 Run Migrations

```bash
cd /home/alvaro/code/rust/ficnexus/forum_core
sqlx migrate run --source migrations
# Applies 001_forum_core.sql (existing) + new 002-007 (if any)
# In FicHub: migrations replay as 072-089 in main migrations/ dir
```

### 3.2 Run Example Server

```bash
# Add example to forum_core/Cargo.toml if not present:
# [[example]]
# name = "standalone"
# path = "examples/standalone.rs"

cargo run -p forum_core --example standalone
# Starts Axum on 0.0.0.0:3000 (configurable via FORUM_PORT)
# Routes: /api/forum/*, /ws/forum
# ReputationHook = NoopHook (logs only)
```

### 3.3 Test Standalone API

```bash
# Categories
curl http://localhost:3000/api/forum/categories

# Create topic (requires auth header - see contracts/forum-topics-posts.md)
curl -X POST http://localhost:3000/api/forum/topics \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <dev-token>" \
  -d '{"title":"Test","category_slug":"general","body":"Hello world"}'

# WebSocket test (wscat)
wscat -c ws://localhost:3000/ws/forum -H "Authorization: Bearer <dev-token>"
# Send: {"type":"subscribe","topic_id":1}
# Receive: {"type":"new_post","topic_id":1,"post":{...}}
```

---

## 4. Embedded Mode (FicHub Integration)

### 4.1 Apply FicHub Migrations

```bash
cd /home/alvaro/code/rust/ficnexus
# Migrations 072-089 already in migrations/ (see data-model.md)
sqlx migrate run
# Adds FKs to users(id) on all forum_* tables
```

### 4.2 Mount in FicHub Router

```rust
// src/server.rs — in build_router()
use crate::routes::forum::forum_routes;
use crate::routes::forum_groups::forum_groups_routes;
use crate::routes::forum_privileges::forum_privileges_routes;
use crate::routes::forum_flags::forum_flags_routes;
use crate::routes::forum_messaging::forum_messaging_routes;
use crate::routes::forum_polls::forum_polls_routes;
use crate::routes::forum_search::forum_search_routes;
use crate::routes::forum_uploads::forum_uploads_routes;
use crate::routes::forum_notifications::forum_notifications_routes;
use crate::routes::forum_ws::forum_ws_route;
use crate::services::forum_pubsub::ForumPubsub;
use crate::services::forum_presence::ForumPresence;

// Initialize services
let forum_pubsub = ForumPubsub::new(state.redis.clone());
let forum_presence = ForumPresence::new(state.redis.clone());

// Mount all forum routes under /api/forum + /forum (SSR) + /ws/forum
let app = Router::new()
    // ... existing routes
    .nest("/api/forum", forum_routes(state.clone()))
    .nest("/api/forum/groups", forum_groups_routes(state.clone()))
    .nest("/api/forum/privileges", forum_privileges_routes(state.clone()))
    .nest("/api/forum/flags", forum_flags_routes(state.clone()))
    .nest("/api/forum/messaging", forum_messaging_routes(state.clone()))
    .nest("/api/forum/polls", forum_polls_routes(state.clone()))
    .nest("/api/forum/search", forum_search_routes(state.clone()))
    .nest("/api/forum/uploads", forum_uploads_routes(state.clone()))
    .nest("/api/forum/notifications", forum_notifications_routes(state.clone()))
    .route("/ws/forum", forum_ws_route(state.clone(), forum_pubsub, forum_presence))
    // SSE fallback
    .route("/api/forum/events", get(sse_handler));
```

### 4.3 Register Reputation Hook

```rust
// src/server.rs — in AppState construction
use forum_core::reputation_hook::{ReputationHook, check_daily_cap};
use crate::db::queries::update_reputation_and_promote;

struct FicHubReputationHook;

#[async_trait]
impl ReputationHook<i32> for FicHubReputationHook {
    async fn award(
        &self,
        actor_id: i32,
        amount: i64,
        source: &str,
        action: &str,
        reference_type: &str,
        reference_id: i64,
        meta: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Daily cap check
        let (allowed, _) = check_daily_cap(
            &self.redis,
            actor_id as i64,
            amount,
            std::env::var("FORUM_REPUTATION_FORUM_DAILY_CAP")
                .unwrap_or("20".into())
                .parse()?,
        ).await?;
        
        if allowed > 0 {
            update_reputation_and_promote(
                &self.db,
                actor_id,
                allowed,
                "forum",
                action,
                reference_type,
                reference_id,
                meta,
            ).await?;
        }
        Ok(())
    }
}

// In AppState::new():
let reputation_hook = Arc::new(FicHubReputationHook { db: pool.clone(), redis: redis.clone() });
```

### 4.4 Frontend Mount (SvelteKit)

```bash
# In frontend/
npm install marked@^14 dompurify@^3

# Routes under frontend/src/routes/forum/
# /forum                    → category list (SSR)
# /forum/board/[slug].[id]  → topic + posts (SSR + WS hydration)
# /forum/compose            → new topic (client)
# /forum/messages           → DMs/rooms (client + WS)
# /forum/notifications      → bell + list (client + WS)
# /forum/search             → search page (client)
# /forum/groups             → groups (client)
# /forum/settings           → preferences (client)
```

---

## 5. Just Recipes (justfile)

Add to repo root `justfile`:

```just
# Forum development
forum-standalone:
    cd forum_core && cargo run --example standalone

forum-embedded:
    cargo run --bin fichub

forum-test:
    cargo test -p forum_core
    cargo test forum_ -- --test-threads=1

forum-migrate:
    sqlx migrate run

forum-migrate-create name:
    sqlx migrate add {{name}} --source forum_core/migrations

forum-lint:
    cargo clippy -p forum_core -- -D warnings
    cargo fmt --check -p forum_core

forum-frontend-dev:
    cd frontend && npm run dev

forum-frontend-test:
    cd frontend && npm run test

forum-frontend-build:
    cd frontend && npm run build

# NodeBB parity check (manual)
forum-parity-check:
    @echo "Run NodeBB at ~/code/js/NodeBB (npm start) on :4567"
    @echo "Run FicHub forum on :3000"
    @echo "Compare: categories, topic list, topic detail, compose, WS live, PWA"

# Importer (when implemented)
forum-import-nodebb dump_file:
    cargo run --bin forum_import_nodebb {{dump_file}}
```

---

## 6. Development Workflow (NFS-Safe)

```bash
# 1. Create worktree on local disk
cd /media/alvaro/code-worktrees
git worktree add ../ficnexus-forum-nodebb feat/forum-nodebb

# 2. Set CARGO_TARGET_DIR for this worktree
export CARGO_TARGET_DIR=/media/alvaro/code-worktrees/ficnexus-forum-nodebb/target

# 3. Develop in worktree
cd /media/alvaro/code-worktrees/ficnexus-forum-nodebb
# ... edit code, run tests ...

# 4. Test
just forum-test
just forum-frontend-test

# 5. When ready, orchestrator merges feat/forum-nodebb → main
#    (Subagents never commit)
```

---

## 7. Verification Checklist

| Check | Command | Expected |
|-------|---------|----------|
| Migrations apply cleanly | `sqlx migrate run` | 0 errors, all `ALTER OWNER TO fichub` |
| Standalone boots | `cargo run -p forum_core --example standalone` | Listens on :3000, `/api/forum/categories` returns `[]` |
| Embedded mounts | `cargo run --bin fichub` | `/api/forum/categories` works, WS connects |
| Contracts pass | `cargo test forum_` | All integration tests green |
| Frontend builds | `cd frontend && npm run build` | No TS errors, `forum/*` routes compiled |
| PWA manifest | `curl /manifest.webmanifest` | Valid JSON, icons, start_url=/forum |
| Daily cap enforced | Post 25 topics in 1 day | First 20 award rep, next 5 award 0 |
| WS fanout | Two clients on same topic | Typing + new_post < 500ms |

---

## 8. Troubleshooting

| Issue | Fix |
|-------|-----|
| `sqlx::Error: Database error: permission denied` | Run `ALTER TABLE ... OWNER TO fichub;` on all forum_* tables |
| WS connects but no events | Check Redis pubsub: `redis-cli SUBSCRIBE forum:topic:1` |
| Daily cap not resetting | Key TTL is 86400s; verify `forum:rep:{uid}:{YYYYMMDD}` expires at midnight UTC |
| Search returns no results | Run backfill: `UPDATE forum_topics SET search_vector = ... WHERE search_vector IS NULL;` |
| Uploads 413 | Increase `FORUM_UPLOAD_MAX_MB` and `max_upload_bytes` in config |
| Frontend TS errors on `marked`/`dompurify` | `npm install --save marked@^14 dompurify@^3 @types/dompurify` |

---

## 9. Key Files to Know

| File | Purpose |
|------|---------|
| `forum_core/src/model.rs` | Domain models (Category, Topic, Post, Vote, Follow, ReadState, Ban, + new: Group, Poll, Room, Message, Draft, Upload, Tag, Notification) |
| `forum_core/src/store/pg.rs` | Postgres implementations for all entities |
| `forum_core/src/reputation_hook.rs` | `ReputationHook` trait + `check_daily_cap` |
| `src/routes/forum.rs` | Core browse/write routes (categories, topics, posts, follow, read, search) |
| `src/routes/forum_*.rs` | Feature routes (groups, privileges, flags, messaging, polls, uploads, notifications) |
| `src/routes/forum_ws.rs` | WebSocket handler + Redis pubsub |
| `src/services/forum_pubsub.rs` | Redis publish/subscribe helper |
| `src/services/forum_presence.rs` | Online presence + typing state |
| `src/services/trust.rs` | Trust gates (`assert_min_trust`, `flag_weight`, `PUBLISH_MIN_TRUST`, `RESOLVE_MIN_TRUST`) |
| `src/config.rs` | All `FORUM_*` env vars |
| `frontend/src/lib/forum/` | Client WS/SSE, composer, PWA SW, components |
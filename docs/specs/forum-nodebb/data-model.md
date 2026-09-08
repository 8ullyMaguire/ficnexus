# Data Model: Forum NodeBB Port — Full DDL Deltas

**Status**: Phase 1 output — additive migrations after `071_trust_preferences.sql`
**Date**: 2026-08-31
**Conventions**: `ALTER OWNER TO fichub` on every new table; `IF NOT EXISTS` guards; FKs to `users(id)` added by FicHub migration (commented in standalone crate migrations); indexes partial where `deleted_at IS NULL AND is_hidden = FALSE`.

---

## 1. New Tables (15 tables)

### 1.1 Groups & Membership

```sql
-- 072_forum_groups.sql
CREATE TABLE IF NOT EXISTS forum_groups (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT UNIQUE NOT NULL,
    slug            TEXT UNIQUE NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    cover_url       TEXT,                    -- uploaded cover image
    is_private      BOOL NOT NULL DEFAULT FALSE,  -- join by invite only
    is_system       BOOL NOT NULL DEFAULT FALSE,  -- auto-assigned (e.g. "Moderators")
    owner_id        INT4   NOT NULL,         -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ
);
ALTER TABLE forum_groups OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_groups_owner ON forum_groups (owner_id);
CREATE INDEX IF NOT EXISTS idx_forum_groups_system ON forum_groups (is_system) WHERE is_system = TRUE;

-- 073_forum_group_members.sql
CREATE TABLE IF NOT EXISTS forum_group_members (
    group_id    BIGINT NOT NULL REFERENCES forum_groups(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,             -- FK users(id) added by embedding app
    role        TEXT   NOT NULL DEFAULT 'member' CHECK (role IN ('owner','manager','member')),
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    invited_by  INT4,                        -- FK users(id) added by embedding app
    PRIMARY KEY (group_id, user_id)
);
ALTER TABLE forum_group_members OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_group_members_user ON forum_group_members (user_id);
CREATE INDEX IF NOT EXISTS idx_forum_group_members_role ON forum_group_members (role) WHERE role IN ('owner','manager');
```

### 1.2 Privileges (Per-Category Per-Group)

```sql
-- 074_forum_privileges.sql
-- privilege: 'read' | 'write' | 'reply' | 'moderate' | 'flag' | 'manage_membership'
CREATE TABLE IF NOT EXISTS forum_privileges (
    id              BIGSERIAL PRIMARY KEY,
    category_id     BIGINT NOT NULL REFERENCES forum_categories(id) ON DELETE CASCADE,
    group_id        BIGINT NOT NULL REFERENCES forum_groups(id) ON DELETE CASCADE,
    privilege       TEXT   NOT NULL CHECK (privilege IN ('read','write','reply','moderate','flag','manage_membership')),
    granted_by      INT4   NOT NULL,         -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category_id, group_id, privilege)
);
ALTER TABLE forum_privileges OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_privileges_category ON forum_privileges (category_id);
CREATE INDEX IF NOT EXISTS idx_forum_privileges_group ON forum_privileges (group_id);
```

### 1.3 Tags

```sql
-- 075_forum_tags.sql
CREATE TABLE IF NOT EXISTS forum_tags (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT UNIQUE NOT NULL,        -- normalized lowercase
    description TEXT NOT NULL DEFAULT '',
    color       TEXT,                        -- hex e.g. '#3b82f6'
    icon        TEXT,                        -- emoji or icon name
    usage_count BIGINT NOT NULL DEFAULT 0,   -- denormalized for sorting
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    merged_into BIGINT REFERENCES forum_tags(id) ON DELETE SET NULL  -- admin merge target
);
ALTER TABLE forum_tags OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_tags_usage ON forum_tags (usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_forum_tags_merged ON forum_tags (merged_into) WHERE merged_into IS NOT NULL;

-- 076_forum_topic_tags.sql
CREATE TABLE IF NOT EXISTS forum_topic_tags (
    topic_id  BIGINT NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    tag_id    BIGINT NOT NULL REFERENCES forum_tags(id) ON DELETE CASCADE,
    added_by  INT4   NOT NULL,               -- FK users(id) added by embedding app
    added_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (topic_id, tag_id)
);
ALTER TABLE forum_topic_tags OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_topic_tags_tag ON forum_topic_tags (tag_id);
```

### 1.4 Polls (NodeBB events parity)

```sql
-- 077_forum_polls.sql
CREATE TABLE IF NOT EXISTS forum_polls (
    id              BIGSERIAL PRIMARY KEY,
    topic_id        BIGINT NOT NULL UNIQUE REFERENCES forum_topics(id) ON DELETE CASCADE,
    question        TEXT NOT NULL,
    max_selections  INT  NOT NULL DEFAULT 1,
    allow_change    BOOL NOT NULL DEFAULT TRUE,
    close_at        TIMESTAMPTZ,             -- NULL = no auto-close
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ
);
ALTER TABLE forum_polls OWNER TO fichub;

-- 078_forum_poll_options.sql
CREATE TABLE IF NOT EXISTS forum_poll_options (
    id          BIGSERIAL PRIMARY KEY,
    poll_id     BIGINT NOT NULL REFERENCES forum_polls(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    position    INT  NOT NULL DEFAULT 0,
    vote_count  BIGINT NOT NULL DEFAULT 0    -- denormalized for live bars
);
ALTER TABLE forum_poll_options OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_poll_options_poll ON forum_poll_options (poll_id, position);

-- 079_forum_poll_votes.sql
CREATE TABLE IF NOT EXISTS forum_poll_votes (
    poll_id     BIGINT NOT NULL REFERENCES forum_polls(id) ON DELETE CASCADE,
    option_id   BIGINT NOT NULL REFERENCES forum_poll_options(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,             -- FK users(id) added by embedding app
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (poll_id, user_id, option_id)
);
ALTER TABLE forum_poll_votes OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_poll_votes_option ON forum_poll_votes (option_id);
CREATE INDEX IF NOT EXISTS idx_forum_poll_votes_user ON forum_poll_votes (user_id);
```

### 1.5 Messaging (DMs + Group Rooms)

```sql
-- 080_forum_rooms.sql
CREATE TABLE IF NOT EXISTS forum_rooms (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT,                      -- NULL for DMs; set for group rooms
    is_group        BOOL NOT NULL DEFAULT FALSE,
    creator_id      INT4   NOT NULL,           -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ,
    last_message_id BIGINT,                    -- FK forum_messages(id) added after messages table
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE forum_rooms OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_rooms_group ON forum_rooms (is_group) WHERE is_group = TRUE;
CREATE INDEX IF NOT EXISTS idx_forum_rooms_activity ON forum_rooms (last_activity_at DESC);

-- 081_forum_room_members.sql
CREATE TABLE IF NOT EXISTS forum_room_members (
    room_id     BIGINT NOT NULL REFERENCES forum_rooms(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,               -- FK users(id) added by embedding app
    role        TEXT   NOT NULL DEFAULT 'member' CHECK (role IN ('owner','moderator','member')),
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at     TIMESTAMPTZ,
    is_muted    BOOL NOT NULL DEFAULT FALSE,
    notify_level TEXT NOT NULL DEFAULT 'all' CHECK (notify_level IN ('all','mentions','none')),
    PRIMARY KEY (room_id, user_id)
);
ALTER TABLE forum_room_members OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_room_members_user ON forum_room_members (user_id);
CREATE INDEX IF NOT EXISTS idx_forum_room_members_active ON forum_room_members (room_id) WHERE left_at IS NULL;

-- 082_forum_messages.sql
CREATE TABLE IF NOT EXISTS forum_messages (
    id              BIGSERIAL PRIMARY KEY,
    room_id         BIGINT NOT NULL REFERENCES forum_rooms(id) ON DELETE CASCADE,
    author_id       INT4   NOT NULL,           -- FK users(id) added by embedding app
    body            TEXT NOT NULL,             -- markdown
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    edited_at       TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    is_hidden       BOOL NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    search_vector   TSVECTOR
);
ALTER TABLE forum_messages OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_messages_room ON forum_messages (room_id, created_at DESC, id DESC) WHERE deleted_at IS NULL AND is_hidden = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_messages_search ON forum_messages USING GIN (search_vector);

-- 083_forum_user_blocks.sql (bidirectional block: blocks + blocked_by)
CREATE TABLE IF NOT EXISTS forum_user_blocks (
    blocker_id  INT4   NOT NULL,               -- FK users(id) added by embedding app
    blocked_id  INT4   NOT NULL,               -- FK users(id) added by embedding app
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker_id, blocked_id)
);
ALTER TABLE forum_user_blocks OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_user_blocks_blocked ON forum_user_blocks (blocked_id);
```

### 1.6 Drafts (Auto-save for Composer)

```sql
-- 084_forum_drafts.sql
CREATE TABLE IF NOT EXISTS forum_drafts (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT4   NOT NULL,           -- FK users(id) added by embedding app
    topic_id        BIGINT REFERENCES forum_topics(id) ON DELETE CASCADE,  -- NULL = new topic draft
    category_id     BIGINT REFERENCES forum_categories(id) ON DELETE SET NULL,
    title           TEXT,
    body            TEXT NOT NULL,             -- markdown
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    poll_data       JSONB,                     -- {question, options[], max_selections, allow_change, close_at}
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE forum_drafts OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_drafts_user ON forum_drafts (user_id, updated_at DESC);
```

### 1.7 Uploads

```sql
-- 085_forum_uploads.sql
CREATE TABLE IF NOT EXISTS forum_uploads (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT4   NOT NULL,           -- FK users(id) added by embedding app
    topic_id        BIGINT REFERENCES forum_topics(id) ON DELETE SET NULL,
    post_id         BIGINT REFERENCES forum_posts(id) ON DELETE SET NULL,
    message_id      BIGINT REFERENCES forum_messages(id) ON DELETE SET NULL,
    original_name   TEXT NOT NULL,
    stored_path     TEXT NOT NULL,             -- /public/uploads/forum/{yyyy}/{mm}/{uuid}.webp
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    width           INT,
    height          INT,
    sha256          CHAR(64) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE forum_uploads OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_uploads_user ON forum_uploads (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_forum_uploads_topic ON forum_uploads (topic_id);
CREATE INDEX IF NOT EXISTS idx_forum_uploads_post ON forum_uploads (post_id);
CREATE INDEX IF NOT EXISTS idx_forum_uploads_message ON forum_uploads (message_id);
CREATE INDEX IF NOT EXISTS idx_forum_uploads_sha256 ON forum_uploads (sha256);
```

**Storage layout & lifecycle:**

```
{cache_dir}/uploads/forum/{yyyy}/{mm}/{token}.{ext}
```

- `cache_dir` = `Config.cache_dir` (default: the repo's `cache/` dir on dev, `/opt/fichub/cache` on prod).
- `yyyy/mm` = wall-clock year/month of upload (distributes files across directories).
- `token` = 128-bit hex string from `upload_token()` (monotonically unique, no collisions).
- `ext` = `png`, `jpg`, `webp`, or `gif` (determined by header sniffing in `classify_image`).

**Lifecycle:**

| Event | On-disk | In `forum_uploads` row |
|---|---|---|
| `POST /api/uploads` | File written to `{yy}/{mm}/{token}.{ext}` | Row inserted (user_id, original_name, stored_path, mime_type, size_bytes, width, height, sha256) |
| `DELETE /api/uploads/{id}` (owner/staff) | `std::fs::remove_file` (best-effort; if it fails the row is still deleted) | Row deleted |
| Rebuild / cache purge | Files orphaned if row deleted without cleanup | — |

**No automated retention policy exists yet.** A future Phase 5+ task should add a periodic cleanup job (delete rows older than N days + their on-disk files) or a backup/retention cron. For now, storage growth is bounded by user upload volume (TL2+ gate, 10 MiB per file, image-only whitelist).

### 1.8 Notifications (Forum-specific types + scheduling)

```sql
-- 086_forum_notifications.sql
CREATE TABLE IF NOT EXISTS forum_notifications (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT4   NOT NULL,           -- FK users(id) added by embedding app
    type            TEXT NOT NULL,             -- 'mention'|'reply'|'new_topic'|'follow'|'poll_vote'|'flag_resolved'|'poll_closed'|'scheduled_published'
    title           TEXT NOT NULL,
    body            TEXT,
    link            TEXT,                      -- deep link: /forum/board/{slug}.{id}#post-{id}
    reference_type  TEXT,                      -- 'forum_topic'|'forum_post'|'forum_room'|'forum_poll'
    reference_id    BIGINT,
    actor_id        INT4,                      -- FK users(id) added by embedding app (who triggered)
    is_read         BOOL NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at         TIMESTAMPTZ,
    -- Scheduled publish (for scheduled topics)
    scheduled_at    TIMESTAMPTZ                -- NULL = immediate; set = future publish time
);
ALTER TABLE forum_notifications OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_notifications_user_unread ON forum_notifications (user_id, is_read, created_at DESC) WHERE is_read = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_notifications_user_all ON forum_notifications (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_forum_notifications_scheduled ON forum_notifications (scheduled_at) WHERE scheduled_at IS NOT NULL AND scheduled_at > NOW();
CREATE INDEX IF NOT EXISTS idx_forum_notifications_ref ON forum_notifications (reference_type, reference_id);
```

---

## 2. Extended / Modified Existing Tables

### 2.1 forum_topics — Additions

```sql
-- 087_forum_topics_extend.sql
ALTER TABLE forum_topics
    ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMPTZ,       -- future publish time
    ADD COLUMN IF NOT EXISTS poll_id BIGINT REFERENCES forum_polls(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS tags_locked BOOL NOT NULL DEFAULT FALSE,  -- mod-locked tags
    ADD COLUMN IF NOT EXISTS teaser TEXT,                    -- auto-generated excerpt
    ADD COLUMN IF NOT EXISTS thumb_url TEXT;                 -- first upload as thumbnail

CREATE INDEX IF NOT EXISTS idx_forum_topics_scheduled ON forum_topics (scheduled_at) WHERE scheduled_at IS NOT NULL AND scheduled_at > NOW();
CREATE INDEX IF NOT EXISTS idx_forum_topics_poll ON forum_topics (poll_id) WHERE poll_id IS NOT NULL;
```

### 2.2 forum_posts — Additions

```sql
-- 088_forum_posts_extend.sql
ALTER TABLE forum_posts
    ADD COLUMN IF NOT EXISTS is_op BOOL NOT NULL DEFAULT FALSE,  -- denormalized for easy OP fetch
    ADD COLUMN IF NOT EXISTS upload_count INT NOT NULL DEFAULT 0;
```

### 2.3 users — Trust/Rep Columns (Already Exist, Listed for Reference)

```sql
-- From 013_trust_levels.sql + 071_trust_preferences.sql
-- trust_level SMALLINT NOT NULL DEFAULT 0 CHECK (trust_level BETWEEN 0 AND 6)
-- tl_metrics JSONB NOT NULL DEFAULT '{}'
-- tl_updated_at TIMESTAMPTZ
-- tl_notes TEXT NOT NULL DEFAULT ''
-- trust_loss_count INT NOT NULL DEFAULT 0
-- reputation BIGINT NOT NULL DEFAULT 0          -- ficnexus-wide leaderboard currency
```

### 2.4 reputation_events — Forum Source (Already Exists, Listed for Reference)

```sql
-- Existing table, forum actions add rows with source='forum'
-- INSERT INTO reputation_events (user_id, amount, source, reference_type, reference_id, created_at)
-- VALUES (uid, amount, 'forum', 'topic'|'post'|'poll_vote'|'flag_helpful', ref_id, NOW());
```

---

## 3. Search Vector Triggers

```sql
-- 089_search_vector_triggers.sql
-- Topics: title weight 'A', body 'D', tags 'B'
CREATE OR REPLACE FUNCTION forum_topics_search_vector_update()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    tag_names TEXT;
BEGIN
    SELECT string_agg(t.name, ' ') INTO tag_names
    FROM forum_topic_tags tt
    JOIN forum_tags t ON t.id = tt.tag_id
    WHERE tt.topic_id = NEW.id AND t.merged_into IS NULL;
    
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.body, '')), 'D') ||
        setweight(to_tsvector('english', COALESCE(tag_names, '')), 'B');
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_forum_topics_search_vector
BEFORE INSERT OR UPDATE ON forum_topics
FOR EACH ROW EXECUTE FUNCTION forum_topics_search_vector_update();

-- Posts: body weight 'D'
CREATE OR REPLACE FUNCTION forum_posts_search_vector_update()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', COALESCE(NEW.body, ''));
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_forum_posts_search_vector
BEFORE INSERT OR UPDATE ON forum_posts
FOR EACH ROW EXECUTE FUNCTION forum_posts_search_vector_update();

-- Messages: body weight 'D'
CREATE OR REPLACE FUNCTION forum_messages_search_vector_update()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', COALESCE(NEW.body, ''));
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_forum_messages_search_vector
BEFORE INSERT OR UPDATE ON forum_messages
FOR EACH ROW EXECUTE FUNCTION forum_messages_search_vector_update();

-- Backfill existing NULL search_vector (idempotent)
UPDATE forum_topics SET search_vector = 
    setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(body, '')), 'D')
WHERE search_vector IS NULL;

UPDATE forum_posts SET search_vector = to_tsvector('english', COALESCE(body, ''))
WHERE search_vector IS NULL;

UPDATE forum_messages SET search_vector = to_tsvector('english', COALESCE(body, ''))
WHERE search_vector IS NULL;
```

---

## 4. Reputation Hook Trait (forum_core)

```rust
// forum_core/src/reputation_hook.rs
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ReputationHook<ActorId>: Send + Sync {
    /// Called when a forum action should award reputation.
    /// Default impl (standalone) is noop.
    /// Embedded impl calls db::queries::update_reputation_and_promote.
    async fn award(
        &self,
        actor_id: ActorId,
        amount: i64,
        source: &str,           // "forum"
        action: &str,           // "topic_create" | "post_create" | "poll_vote" | "flag_helpful" | "moderation_helpful"
        reference_type: &str,   // "forum_topic" | "forum_post" | "forum_poll" | "user_report"
        reference_id: i64,
        meta: Value,            // opaque context
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Daily cap check — Redis key: forum:rep:{uid}:{YYYYMMDD}
/// Returns (allowed_amount, remaining_cap)
pub async fn check_daily_cap(
    redis: &redis::aio::ConnectionManager,
    actor_id: i64,
    requested: i64,
    cap: i64,
) -> Result<(i64, i64), redis::RedisError> {
    let day = chrono::Utc::now().format("%Y%m%d").to_string();
    let key = format!("forum:rep:{}:{}", actor_id, day);
    let current: i64 = redis.get(&key).await.unwrap_or(0);
    let remaining = (cap - current).max(0);
    let allowed = requested.min(remaining);
    if allowed > 0 {
        let _: () = redis.incr(&key, allowed).await?;
        let _: () = redis.expire(&key, 86400).await?;
    }
    Ok((allowed, remaining))
}
```

---

## 5. Redis Keyspace Summary

| Key Pattern | Type | TTL | Purpose |
|-------------|------|-----|---------|
| `forum:presence:{uid}` | Hash | 300s | `{topic_id: last_seen_ts, ...}` — online indicator |
| `forum:typing:{topic_id}:{uid}` | String | 3s | Typing indicator (refreshed on keystroke) |
| `forum:rep:{uid}:{YYYYMMDD}` | String (Int) | 86400 | Daily forum rep cap counter |
| `forum:rate:{action}:{uid}` | String (Int) | Per action | Rate limit windows (topic_create=300s, post_create=60s, flag=600s, poll_vote=60s) |
| `forum:ws:sub:{connection_id}` | Set | Connection life | Topics/rooms user subscribed to (for targeted push) |
| `forum:push:{uid}` | String | Persistent | VAPID push subscription JSON |

---

## 6. Index Summary (All Partial Where `deleted_at IS NULL AND is_hidden = FALSE`)

| Table | Index | Columns | Purpose |
|-------|-------|---------|---------|
| forum_topics | idx_forum_topics_category | (category_id, last_activity_at DESC) | Category topic list |
| forum_topics | idx_forum_topics_status | (status) | Pin/lock filtering |
| forum_topics | idx_forum_topics_search | GIN (search_vector) | FTS |
| forum_topics | idx_forum_topics_scheduled | (scheduled_at) WHERE scheduled_at > NOW() | Cron promoter |
| forum_topics | idx_forum_topics_slug | (topic_slug) WHERE topic_slug IS NOT NULL | Slug lookup |
| forum_posts | idx_forum_posts_topic | (topic_id, created_at ASC, id ASC) | Topic post pagination |
| forum_posts | idx_forum_posts_search | GIN (search_vector) | FTS |
| forum_poll_votes | idx_forum_poll_votes_option | (option_id) | Live vote counts |
| forum_messages | idx_forum_messages_room | (room_id, created_at DESC) | Room message pagination |
| forum_messages | idx_forum_messages_search | GIN (search_vector) | FTS |
| forum_notifications | idx_forum_notifications_user_unread | (user_id, is_read, created_at DESC) WHERE is_read=FALSE | Bell badge |
| forum_notifications | idx_forum_notifications_scheduled | (scheduled_at) WHERE scheduled_at > NOW() | Cron publisher |
| forum_uploads | idx_forum_uploads_sha256 | (sha256) | Dedupe |

---

## 7. Migration Ordering (FicHub Integration)

```
072_forum_groups.sql
073_forum_group_members.sql
074_forum_privileges.sql
075_forum_tags.sql
076_forum_topic_tags.sql
077_forum_polls.sql
078_forum_poll_options.sql
079_forum_poll_votes.sql
080_forum_rooms.sql
081_forum_room_members.sql
082_forum_messages.sql
083_forum_user_blocks.sql
084_forum_drafts.sql
085_forum_uploads.sql
086_forum_notifications.sql
087_forum_topics_extend.sql
088_forum_posts_extend.sql
089_search_vector_triggers.sql
```

Each migration file ends with `ALTER TABLE <table> OWNER TO fichub;` and runs in filename order via `sqlx migrate run`.

---

## 8. Entity Relationship Diagram (Text)

```
users (ficnexus core)
  │
  ├─┬ forum_groups ◄────────────────┐
  │ │                                 │
  │ └─ forum_group_members ──────────┘
  │
  ├─┬ forum_privileges ◄─────────────┐ (category_id, group_id, privilege)
  │ │                                 │
  │ └─ forum_categories ─────────────┘
  │
  ├─ forum_topics
  │    ├─ forum_posts (topic_id)
  │    │    ├─ forum_post_votes (post_id)
  │    │    └─ forum_uploads (post_id)
  │    ├─ forum_polls (topic_id 1:1)
  │    │    ├─ forum_poll_options (poll_id)
  │    │    └─ forum_poll_votes (poll_id, option_id)
  │    └─ forum_topic_tags (topic_id)
  │         └─ forum_tags (tag_id)
  │
  ├─ forum_rooms
  │    ├─ forum_room_members (room_id)
  │    ├─ forum_messages (room_id)
  │    │    └─ forum_uploads (message_id)
  │    └─ forum_user_blocks (separate)
  │
  ├─ forum_drafts (user_id, topic_id?)
  │
  └─ forum_notifications (user_id, reference_type, reference_id, scheduled_at)
```

---

## 9. Reputation Action Amounts (Configurable via Env)

| Action | Env Var | Default | Trust Gate |
|--------|---------|---------|------------|
| Topic create | `FORUM_REPUTATION_TOPIC_CREATE` | 5 | TL1+ |
| Post create | `FORUM_REPUTATION_POST_CREATE` | 2 | TL1+ |
| Poll vote | `FORUM_REPUTATION_POLL_VOTE` | 1 | TL1+ |
| Flag helpful (mod confirms) | `FORUM_REPUTATION_FLAG_HELPFUL` | 10 | TL2+ flagger |
| Moderation helpful (metamod fair) | `FORUM_REPUTATION_MOD_HELPFUL` | 5 | TL5+ moderator |
| Daily cap | `FORUM_REPUTATION_FORUM_DAILY_CAP` | 20 | All |

Total daily forum rep ≤ 20 (configurable). Excess actions succeed but award 0 rep.
-- forum-core: forum data model (SPEC-COMMUNITY-PLATFORM.md §4).
--
-- NOTE ON INTEGRATION: this migration is self-contained — it creates the
-- `forum_*` tables and (at the end) extends the existing `user_reports`
-- CHECK constraint. In FicHub, `user_reports` already exists (migration
-- 022); in a fresh database where this crate runs standalone, the
-- `IF EXISTS` guard on the ALTER makes it a no-op so the migration is
-- still valid. FicHub's own integration applies the ALTER separately
-- against its existing schema.
--
-- The `users(id)` FK references are commented out for the standalone case:
-- a generic crate cannot assume the embedding app's user table name. The
-- embedding application re-adds the constraints (FicHub: `ALTER TABLE ...
-- ADD CONSTRAINT ... FOREIGN KEY (author_id) REFERENCES users(id) ON
-- DELETE CASCADE`) in its own migration 041. Author ids are INT4, matching
-- `users.id`; the crate stays generic over the actor id type.

-- Categories: curated, admin-created in M1.
CREATE TABLE IF NOT EXISTS forum_categories (
    id          BIGSERIAL PRIMARY KEY,
    slug        TEXT UNIQUE NOT NULL,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    position    INT  NOT NULL DEFAULT 0,
    is_mod_only BOOL NOT NULL DEFAULT FALSE,   -- announcements etc.
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Topics: first post is the OP (body denormalized for list/search).
CREATE TABLE IF NOT EXISTS forum_topics (
    id                BIGSERIAL PRIMARY KEY,
    category_id       BIGINT NOT NULL REFERENCES forum_categories(id) ON DELETE CASCADE,
    author_id         INT4   NOT NULL,          -- FK to users(id) added by the embedding app
    title             TEXT   NOT NULL,
    body              TEXT   NOT NULL,          -- markdown (OP)
    payload           JSONB  NOT NULL DEFAULT '{}'::jsonb, -- opaque: {"type":"work|list|request","id":…}
    status            TEXT   NOT NULL DEFAULT 'open'
                      CHECK (status IN ('open','locked','pinned','archived')),
    view_count        BIGINT NOT NULL DEFAULT 0,
    last_post_id      BIGINT,
    last_activity_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ,
    deleted_at        TIMESTAMPTZ,              -- soft delete
    is_hidden         BOOL   NOT NULL DEFAULT FALSE,
    search_vector     TSVECTOR                   -- maintained at write time
);
CREATE INDEX IF NOT EXISTS idx_forum_topics_category
    ON forum_topics (category_id, last_activity_at DESC)
    WHERE deleted_at IS NULL AND is_hidden = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_topics_status ON forum_topics (status);
CREATE INDEX IF NOT EXISTS idx_forum_topics_search ON forum_topics USING GIN (search_vector);

-- Posts: flat replies within a topic, optional quote.
CREATE TABLE IF NOT EXISTS forum_posts (
    id            BIGSERIAL PRIMARY KEY,
    topic_id      BIGINT NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    author_id     INT4   NOT NULL,              -- FK to users(id) added by the embedding app
    body          TEXT   NOT NULL,              -- markdown
    payload       JSONB  NOT NULL DEFAULT '{}'::jsonb,
    quote_of      BIGINT REFERENCES forum_posts(id) ON DELETE SET NULL,
    edited_at     TIMESTAMPTZ,
    deleted_at    TIMESTAMPTZ,
    is_hidden     BOOL   NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    search_vector TSVECTOR
);
CREATE INDEX IF NOT EXISTS idx_forum_posts_topic
    ON forum_posts (topic_id, created_at ASC, id ASC)
    WHERE deleted_at IS NULL AND is_hidden = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_posts_search ON forum_posts USING GIN (search_vector);

-- Votes: one ±1 per user per post.
CREATE TABLE IF NOT EXISTS forum_post_votes (
    post_id    BIGINT NOT NULL REFERENCES forum_posts(id) ON DELETE CASCADE,
    user_id    INT4   NOT NULL,                 -- FK to users(id) added by the embedding app
    value      SMALLINT NOT NULL CHECK (value IN (-1, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_forum_post_votes_post ON forum_post_votes (post_id);

-- Follows: per user per topic.
CREATE TABLE IF NOT EXISTS forum_follows (
    user_id    INT4   NOT NULL,                 -- FK to users(id) added by the embedding app
    topic_id   BIGINT NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, topic_id)
);

-- Read state: per user per topic.
CREATE TABLE IF NOT EXISTS forum_read_state (
    user_id           INT4   NOT NULL,          -- FK to users(id) added by the embedding app
    topic_id          BIGINT NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    last_read_post_id BIGINT,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, topic_id)
);

-- Bans: global (category_id NULL) or per-category; expires_at NULL = permanent.
CREATE TABLE IF NOT EXISTS forum_bans (
    id          BIGSERIAL PRIMARY KEY,
    user_id     INT4   NOT NULL,                -- FK to users(id) added by the embedding app
    category_id BIGINT REFERENCES forum_categories(id) ON DELETE CASCADE, -- NULL = global
    reason      TEXT   NOT NULL DEFAULT '',
    banned_by   INT4   NOT NULL,                -- FK to users(id) added by the embedding app
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_forum_bans_user ON forum_bans (user_id);

-- Extend existing reports to cover forum targets (single mod queue).
-- No-op when user_reports does not exist (standalone crate); FicHub's own
-- integration applies this against its existing schema (migration 022).
ALTER TABLE IF EXISTS user_reports DROP CONSTRAINT IF EXISTS user_reports_target_type_check;
ALTER TABLE IF EXISTS user_reports ADD CONSTRAINT user_reports_target_type_check
    CHECK (target_type IN ('comment','work','user','forum_topic','forum_post'));

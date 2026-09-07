-- 077_forum_drafts_uploads_notifications.sql
-- Drafts, uploads, notifications (data-model.md §1.6, §1.7, §1.8).
-- Originally applied to prod; recovered from live DDL 2026-09-07.

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
    scheduled_at    TIMESTAMPTZ                -- NULL = immediate; set = future publish time
);
ALTER TABLE forum_notifications OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_notifications_user_unread ON forum_notifications (user_id, is_read, created_at DESC) WHERE is_read = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_notifications_user_all ON forum_notifications (user_id, created_at DESC);
-- Partial index: NOW() is volatile, so a static predicate is used and the
-- cron query applies `scheduled_at <= NOW()` at scan time. Same selectivity.
CREATE INDEX IF NOT EXISTS idx_forum_notifications_scheduled ON forum_notifications (scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_forum_notifications_ref ON forum_notifications (reference_type, reference_id);

-- 078_forum_topics_posts_extend.sql
-- Extend forum_topics, forum_posts, user_reports (data-model.md §2).
-- Originally applied to prod; recovered from live DDL 2026-09-07.

ALTER TABLE forum_topics
    ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMPTZ,       -- future publish time
    ADD COLUMN IF NOT EXISTS poll_id BIGINT REFERENCES forum_polls(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS tags_locked BOOL NOT NULL DEFAULT FALSE,  -- mod-locked tags
    ADD COLUMN IF NOT EXISTS teaser TEXT,                    -- auto-generated excerpt
    ADD COLUMN IF NOT EXISTS thumb_url TEXT;                 -- first upload as thumbnail

-- Partial index: NOW() is volatile, so a static predicate is used and the
-- cron query applies `scheduled_at <= NOW()` at scan time. Same selectivity.
CREATE INDEX IF NOT EXISTS idx_forum_topics_scheduled ON forum_topics (scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_forum_topics_poll ON forum_topics (poll_id) WHERE poll_id IS NOT NULL;

ALTER TABLE forum_posts
    ADD COLUMN IF NOT EXISTS is_op BOOL NOT NULL DEFAULT FALSE,  -- denormalized for easy OP fetch
    ADD COLUMN IF NOT EXISTS upload_count INT NOT NULL DEFAULT 0;

ALTER TABLE user_reports
    ADD COLUMN IF NOT EXISTS weight INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS auto_status TEXT,  -- 'auto_hidden' | 'needs_admin' | NULL
    ADD COLUMN IF NOT EXISTS resolved_by INT4,  -- FK users(id)
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ;

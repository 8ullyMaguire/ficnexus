-- 074_forum_tags.sql
-- Tags + topic-tag join (data-model.md §1.3).
-- Originally applied to prod; recovered from live DDL 2026-09-07.
--
-- NOTE: forum_topic_tags uses a (topic_id, tag text) shape, NOT the
-- (topic_id, tag_id) normalized shape from the data-model. The Rust
-- handlers in src/routes/forum.rs operate on plain tag strings (set/get
-- topic_tags, 5-tag cap), and migration 089 explicitly drops the
-- search-vector triggers that referenced `tt.tag_id`. This migration
-- matches the live shape; a follow-up migration can introduce the
-- forum_tags normalization later (admin merge) without breaking handlers.

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

CREATE TABLE IF NOT EXISTS forum_topic_tags (
    topic_id  BIGINT NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    tag       TEXT   NOT NULL,
    added_by  INT4   NOT NULL DEFAULT 0,     -- FK users(id) added by embedding app
    added_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (topic_id, tag)
);
ALTER TABLE forum_topic_tags OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_topic_tags_tag ON forum_topic_tags (tag);

-- 082_forum_parity (renumbered from 073): NodeBB parity extras — topic tags,
-- user prefs (RSS is view-only).
-- Handlers also CREATE IF NOT EXISTS at runtime, so this migration is advisory/idempotent.
--
-- NOTE (2026-09-07 deploy): prod already had a forum_topic_tags(topic_id,
-- tag_id) table from 074_forum_tags, but the live handlers in
-- src/routes/forum.rs use a (topic_id, tag text) shape, and both tables were
-- empty (0 rows). This migration drops the unused normalized table and
-- creates the shape the code actually uses.
DROP TABLE IF EXISTS forum_topic_tags;
CREATE TABLE IF NOT EXISTS forum_topic_tags (
  topic_id bigint NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
  tag text NOT NULL,
  PRIMARY KEY (topic_id, tag)
);
CREATE TABLE IF NOT EXISTS forum_user_prefs (
  user_id integer PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  posts_per_page integer NOT NULL DEFAULT 25,
  topic_sort text NOT NULL DEFAULT 'recently_replied'
);
CREATE INDEX IF NOT EXISTS idx_forum_topic_tags_tag ON forum_topic_tags(tag);

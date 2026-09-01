-- 073_forum_parity: NodeBB parity extras — topic tags, user prefs (RSS is view-only).
-- Handlers also CREATE IF NOT EXISTS at runtime, so this migration is advisory/idempotent.
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

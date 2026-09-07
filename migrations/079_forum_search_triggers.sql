-- 079_forum_search_triggers.sql
-- search_vector tsvector triggers for topics, posts, messages (data-model.md §3).
-- Originally applied to prod; recovered from live DDL 2026-09-07.
--
-- NOTE: The forum_topics trigger joins forum_topic_tags on its `tag` text
-- column (the live shape — see 074). The data-model §3 used `tag_id` for
-- a normalized join, but the live DB has the (topic_id, tag) shape. This
-- trigger matches reality and is the one 089 drops because the Rust
-- handlers maintain search_vector inline on every write.

-- Topics: title weight 'A', body 'D', tags 'B'
CREATE OR REPLACE FUNCTION forum_topics_search_vector_update()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    tag_names TEXT;
BEGIN
    SELECT string_agg(tt.tag, ' ') INTO tag_names
    FROM forum_topic_tags tt
    WHERE tt.topic_id = NEW.id;
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.body, '')), 'D') ||
        setweight(to_tsvector('english', COALESCE(tag_names, '')), 'B');
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS trg_forum_topics_search_vector ON forum_topics;
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
DROP TRIGGER IF EXISTS trg_forum_posts_search_vector ON forum_posts;
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
DROP TRIGGER IF EXISTS trg_forum_messages_search_vector ON forum_messages;
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

-- 089_forum_topic_tags_trigger_fix
--
-- Migration 082 (re)created forum_topic_tags with a (topic_id, tag text) shape
-- to match the live Rust handlers in src/routes/forum.rs.
--
-- Prod carried a stale pair of row triggers on forum_topics
-- (`trg_forum_topics_search_vector` + `forum_topics_search_vector_update`)
-- whose body joined `forum_topic_tags tt` on a `tt.tag_id` column that no
-- longer exists. Every UPDATE on a topic row — including the harmless
-- `view_count = view_count + 1` bump that topic_detail performs on each
-- read — fired the broken function and aborted the request. Symptom:
--
--   ERROR column tt.tag_id does not exist
--   → 500 {"err":-1,"msg":"database error"} on GET /api/forum/topics/{id}
--
-- The Rust handlers maintain `search_vector` inline on every write
-- (see src/routes/forum.rs header comment: create/update topic + the edit
-- review path all set it explicitly), so the DB triggers are redundant.
-- Fix: drop both triggers and the orphan function. Any environment that
-- still runs this migration keeps application-managed vectors; nothing
-- else in the codebase reads the trigger-maintained path.
DROP TRIGGER IF EXISTS trg_forum_topics_search_vector ON forum_topics;
DROP TRIGGER IF EXISTS forum_topics_search_vector_update ON forum_topics;
DROP FUNCTION IF EXISTS public.forum_topics_search_vector_update();

-- 090_feature_clusters_status_default.sql
--
-- 064_roadmap_kanban.sql migrated feature_clusters.status from the 3-value set
-- (open, shipped, rejected, deferred) to the 8-value Kanban set. It correctly
-- rewrote the existing rows ('open'->'idea', 'deferred'->'long_term') and
-- dropped and re-added the CHECK constraint with the new values.
--
-- It never changed the column DEFAULT, which is still 'open' from
-- 001_initial.sql:1117. 'open' is not in the new set, so any INSERT that omits
-- status is rejected by the table's own CHECK constraint and the table cannot
-- be written to at all:
--
--   INSERT INTO feature_clusters (representative_text, embedding)
--   VALUES ('probe', array_fill(0::real, ARRAY[768])::vector);
--   ERROR: new row ... violates check constraint "feature_clusters_status_check"
--
-- This matters beyond tests: src/routes/roadmap.rs:153 selects
-- WHERE status = 'idea', src/bin/seed_roadmap.rs exists to populate the table,
-- and src/routes/consensus.rs:213 and src/routes/admin.rs:1289 read it. With
-- the table unwritable, all of them see nothing, always.
--
-- Fix the DEFAULT to 'idea': the value 064 mapped 'open' to, and the value the
-- roadmap query filters on. Do NOT widen the CHECK to re-admit 'open' - 064
-- removed it deliberately, and re-adding it would recreate the very
-- inconsistency this migration fixes.
--
-- 064 itself is an applied migration and is left untouched.

ALTER TABLE feature_clusters
  ALTER COLUMN status SET DEFAULT 'idea';

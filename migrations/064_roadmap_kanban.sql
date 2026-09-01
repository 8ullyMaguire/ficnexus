-- 064_roadmap_kanban.sql
-- Add Kanban statuses + category to feature_clusters, per StoryGraph plan.

-- 1. Migrate legacy status values FIRST, then widen the enum (old: open,
--    shipped, rejected, deferred → new: idea, long_term, medium_term,
--    up_next, in_progress, finished, shipped, rejected). The UPDATEs must
--    run before the CHECK constraint is (re)added, or legacy rows violate it.
UPDATE feature_clusters SET status = 'idea' WHERE status = 'open';
UPDATE feature_clusters SET status = 'long_term' WHERE status = 'deferred';

ALTER TABLE feature_clusters DROP CONSTRAINT IF EXISTS feature_clusters_status_check;
ALTER TABLE feature_clusters ADD CONSTRAINT feature_clusters_status_check
  CHECK (status IN ('idea','long_term','medium_term','up_next','in_progress','finished','shipped','rejected'));

-- 2. Add category column
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS category TEXT DEFAULT 'general';
CREATE INDEX IF NOT EXISTS idx_feature_clusters_category ON feature_clusters(category);
CREATE INDEX IF NOT EXISTS idx_feature_clusters_status ON feature_clusters(status);
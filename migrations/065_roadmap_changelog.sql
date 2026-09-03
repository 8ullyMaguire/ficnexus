-- 065_roadmap_changelog.sql
-- Changelog table for /roadmap/changelog (StoryGraph-style blog with New/Improved/Fixed labels)

CREATE TABLE IF NOT EXISTS roadmap_changelog (
    id BIGSERIAL PRIMARY KEY,
    feature_id INTEGER REFERENCES feature_clusters(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'fixed', -- 'new', 'improved', 'fixed'
    author_id INTEGER NOT NULL,
    published_at TIMESTAMPTZ DEFAULT NOW(),
    is_published BOOLEAN DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_roadmap_changelog_feature ON roadmap_changelog(feature_id);
CREATE INDEX IF NOT EXISTS idx_roadmap_changelog_published ON roadmap_changelog(published_at DESC) WHERE is_published = TRUE;
CREATE INDEX IF NOT EXISTS idx_roadmap_changelog_kind ON roadmap_changelog(kind);
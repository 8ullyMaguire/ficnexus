-- Visitor funnel: anonymous state storage.
--
-- Stores ratings, saved searches, and bookmarks for anonymous users
-- keyed by visitor_id (UUID). Merged into user account on registration.
-- Rows older than 30 days are cleaned up by a cron job.

CREATE TABLE IF NOT EXISTS visitor_state (
    visitor_id  UUID PRIMARY KEY,
    ratings     JSONB NOT NULL DEFAULT '{}'::jsonb,
    saved_searches JSONB NOT NULL DEFAULT '[]'::jsonb,
    bookmarks   JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for cleanup queries (find rows older than 30 days).
CREATE INDEX IF NOT EXISTS idx_visitor_state_updated_at
    ON visitor_state (updated_at);

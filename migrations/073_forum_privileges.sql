-- 073_forum_privileges.sql
-- Per-category per-group privilege matrix (data-model.md §1.2).
-- Originally applied to prod; recovered from live DDL 2026-09-07.

CREATE TABLE IF NOT EXISTS forum_privileges (
    id              BIGSERIAL PRIMARY KEY,
    category_id     BIGINT NOT NULL REFERENCES forum_categories(id) ON DELETE CASCADE,
    group_id        BIGINT NOT NULL REFERENCES forum_groups(id) ON DELETE CASCADE,
    privilege       TEXT   NOT NULL CHECK (privilege IN ('read','write','reply','moderate','flag','manage_membership')),
    granted_by      INT4   NOT NULL,         -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category_id, group_id, privilege)
);
ALTER TABLE forum_privileges OWNER TO fichub;
CREATE INDEX IF NOT EXISTS idx_forum_privileges_category ON forum_privileges (category_id);
CREATE INDEX IF NOT EXISTS idx_forum_privileges_group ON forum_privileges (group_id);

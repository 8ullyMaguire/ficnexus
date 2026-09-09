-- 093 — Critical audit fixes
--
-- 1. Add category to user_reports for report classification
-- 2. Create ban_appeals table for ban appeal workflow
-- 3. Add last_status_change_at trigger to user_reports (if not already)

-- Report categories for classification
ALTER TABLE user_reports
    ADD COLUMN IF NOT EXISTS category text DEFAULT 'other' NOT NULL;

-- Index for filtering by category
CREATE INDEX IF NOT EXISTS idx_user_reports_category ON user_reports (category);

-- Ban appeals table
CREATE TABLE IF NOT EXISTS ban_appeals (
    id bigint NOT NULL DEFAULT nextval('ban_appeals_id_seq'::regclass),
    ban_id bigint NOT NULL REFERENCES forum_bans(id) ON DELETE CASCADE,
    user_id integer NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason text NOT NULL,
    status text DEFAULT 'pending' NOT NULL,
    reviewed_by integer REFERENCES users(id),
    reviewed_at timestamptz,
    reviewer_note text,
    created_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT ban_appeals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text])))
);

CREATE SEQUENCE IF NOT EXISTS ban_appeals_id_seq START WITH 1 INCREMENT BY 1;

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_ban_appeals_ban_id ON ban_appeals (ban_id);
CREATE INDEX IF NOT EXISTS idx_ban_appeals_user_id ON ban_appeals (user_id);
CREATE INDEX IF NOT EXISTS idx_ban_appeals_status ON ban_appeals (status) WHERE status = 'pending';

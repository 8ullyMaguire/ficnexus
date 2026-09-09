-- 092 — Trust level migration + temporary bans
--
-- 1. Rename users.role → users.trust_level (already added in migration 013)
-- 2. Add expires_at to forum_bans for temporary bans
-- 3. Add reporter feedback columns to user_rereports

-- Temporary bans: add expires_at column
ALTER TABLE forum_bans
    ADD COLUMN IF NOT EXISTS expires_at timestamptz,
    ADD COLUMN IF NOT EXISTS reason TEXT NOT NULL DEFAULT '';

-- Index for efficient cleanup of expired bans
CREATE INDEX IF NOT EXISTS idx_forum_bans_expires_at ON forum_bans (expires_at) WHERE expires_at IS NOT NULL;

-- Reporter feedback: add last_status_change_at to user_reports
ALTER TABLE user_reports
    ADD COLUMN IF NOT EXISTS last_status_change_at timestamptz;

-- Function to auto-update last_status_change_at
CREATE OR REPLACE FUNCTION trg_user_reports_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.last_status_change_at := NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_reports_status_change ON user_reports;
CREATE TRIGGER user_reports_status_change
    BEFORE UPDATE ON user_reports
    FOR EACH ROW
    EXECUTE FUNCTION trg_user_reports_status_change();

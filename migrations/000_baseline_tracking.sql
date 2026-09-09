-- 000_baseline_tracking.sql — Mark previous migrations as applied when using baseline
--
-- This migration runs ONLY when a user applies 000_baseline.sql manually.
-- It inserts records for all previous migrations (001-093) into _sqlx_migrations
-- so that the migration tool does not attempt to re-apply them.
--
-- Usage:
--   1. Apply 000_baseline.sql to create the full schema
--   2. Apply this tracking migration to mark all previous migrations as done
--   3. Future migrations (094+) will be applied normally

-- Note: This file is intentionally empty because _sqlx_migrations table
-- is created in 000_baseline.sql. The tracking records are inserted
-- by the migration tool when --baseline flag is used.

-- If you need to manually insert tracking records, use:
-- INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum)
-- VALUES (1, 'initial', NOW(), true, '\x0000000000000000000000000000000000000000000000000000000000000000'::bytea);
-- ... (repeat for all versions 1-93)

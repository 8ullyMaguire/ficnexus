-- Add the report category the reports handler already validates and inserts.
--
-- src/routes/reports.rs:173 inserts into user_reports.category, but no
-- migration ever created the column, so POST /api/reports returned 500 on
-- every call. The handler validates category against five values
-- (spam|harassment|copyright|inappropriate|other), so the column is part of the
-- request contract, not a typo.
--
-- Additive only. user_reports is applied schema with production data: no
-- column is dropped, renamed or retyped, and no existing row is rewritten.
-- NOT NULL with a default is required so the migration succeeds on a table
-- that already has rows; historical rows land on 'other' for human triage,
-- which is what status/auto_status are for.

ALTER TABLE user_reports
    ADD COLUMN IF NOT EXISTS category text NOT NULL DEFAULT 'other';

ALTER TABLE user_reports
    DROP CONSTRAINT IF EXISTS user_reports_category_check;

ALTER TABLE user_reports
    ADD CONSTRAINT user_reports_category_check
    CHECK (category IN ('spam', 'harassment', 'copyright', 'inappropriate', 'other'));

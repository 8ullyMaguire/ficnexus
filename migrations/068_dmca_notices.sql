-- 068 — DMCA / copyright takedown notices
--
-- A rights holder can submit a takedown notice via POST /api/copyright/notice.
-- Admins review it in /api/admin/copyright/notices and can mark it actioned
-- (which blacklists the work) or rejected.

CREATE TABLE IF NOT EXISTS public.copyright_notices (
    id              bigserial PRIMARY KEY,
    work_url_id     text NOT NULL,             -- fic_info.id or source URL
    work_title      text NOT NULL DEFAULT '',
    claimant_name   text NOT NULL,
    claimant_email  text NOT NULL,
    reason          text NOT NULL DEFAULT '',  -- free-form description
    status          text NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'actioned', 'rejected')),
    admin_notes     text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    resolved_at     timestamptz
);

CREATE INDEX IF NOT EXISTS idx_copyright_notices_status
    ON public.copyright_notices (status, created_at DESC);

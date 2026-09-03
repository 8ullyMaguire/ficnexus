-- Auto-moderation: track scheduled deletions for LLM-flagged noise.
-- deletion_scheduled_at: when the grace period expires (null = not flagged).
-- Curators can confirm (instant delete) or dismiss (keep the fic) before this time.

ALTER TABLE public.content_scan
  ADD COLUMN IF NOT EXISTS deletion_scheduled_at timestamp with time zone DEFAULT NULL;

-- Index for the auto-delete cron to find expired pending noise entries.
CREATE INDEX IF NOT EXISTS idx_content_scan_deletion_pending
  ON public.content_scan (deletion_scheduled_at)
  WHERE review_status = 'pending' AND classification = 'noise' AND deletion_scheduled_at IS NOT NULL;

-- Allow curators (role >= 5) to review, not just admins.
-- The existing review_content_scan already checks role >= 10.
-- We'll add a separate curator review endpoint.

-- 013 — Discourse-style 7-level trust system + self-moderation triage
--
-- Trust is a *separate axis* from rank/level/reputation/role:
--   rank/level      = XP progression (cosmetic + feature unlocks)
--   reputation      = spendable bounty currency
--   role            = staff authority (0 user, 5 curator, 10 admin)
--   trust_level     = participation *safety*: what a new user may do and how
--                     much their flag/PM/links/images weigh. Earned by
--                     consistent reading + community behavior over time.
--
-- Levels (Discourse-inspired):
--   0 New        sandboxed write (reads + bookmarks fine)
--   1 Basic      core write (comment/review/flag)
--   2 Member     publish skins/recipes, flag weight 1
--   3 Regular    larger flag weight, edit titles
--   4 Elder      long track record, vote on content proposals
--   5 Community Mod  may resolve reports & hide content — the *human* layer
--   6 Near-admin  (staff-designated) everything except bans/billing
--
-- TL5+ are *recommended* by the weekly digest (auto-promoter only reaches TL4);
-- an admin confirms from the digest, so moderation scales without admin work.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS trust_level smallint NOT NULL DEFAULT 0
        CHECK (trust_level BETWEEN 0 AND 6),
    ADD COLUMN IF NOT EXISTS tl_metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS tl_updated_at timestamptz,
    ADD COLUMN IF NOT EXISTS tl_notes text NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_users_trust_level ON public.users (trust_level);

-- Audit trail + promotion notices for every trust level change.
CREATE TABLE IF NOT EXISTS public.trust_events (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id     integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    from_level  smallint NOT NULL,
    to_level    smallint NOT NULL,
    reason      text NOT NULL,
    by_user_id  integer REFERENCES public.users(id) ON DELETE SET NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_trust_events_user ON public.trust_events (user_id);
CREATE INDEX IF NOT EXISTS idx_trust_events_created ON public.trust_events (created_at);

-- Self-moderation triage columns on user reports:
--   weight       = effective flag weight (scaled by the reporter's trust)
--   auto_status  = 'pending' | 'auto_hidden' | 'needs_admin' | 'resolved'
--   resolved_by  = user id of the TL5+/admin who resolved it (transparency)
ALTER TABLE public.user_reports
    ADD COLUMN IF NOT EXISTS weight integer NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS auto_status text NOT NULL DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS resolved_by integer
        REFERENCES public.users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS resolved_at timestamptz;
CREATE INDEX IF NOT EXISTS idx_user_reports_auto_status
    ON public.user_reports (auto_status);
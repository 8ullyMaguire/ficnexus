-- 011 — Idle-tick hourly dedup cooldown
--
-- `idle_tick` previously awarded 2 XP on every authenticated POST with only a
-- daily cap (1440) as a guard. A bot could still drain the full daily cap in a
-- tight loop. This migration sets a 3600s (1h) cooldown on the source def so
-- the engine's cooldown dedup fires: the engine counts recent `xp_events`
-- rows matching `(user_id, event_type, source_ref)` within `cooldown_seconds`.
--
-- `src/services/bounties.rs::idle_tick` now passes a per-user, per-hour time
-- bucket (`YYYY-MM-DDTHH`, e.g. "2026-08-26T14") as `source_ref`, so the
-- cooldown dedups on *that hour* — one 2 XP award per user per hour, exactly.

UPDATE xp_source_defs
  SET cooldown_seconds = 3600
WHERE event_type = 'idle_tick';

-- Seed Admin-only XP source defs (B3 allowlist: description LIKE 'Admin:%').
-- These are grantable ONLY via POST /api/admin/reputation/award.
INSERT INTO xp_source_defs (event_type, xp_amount, description)
VALUES
  ('scraper_fix_merged',    100, 'Admin: scraper selector/merge approved'),
  ('scraper_selector_submitted', 30, 'Admin: scraper selector submitted'),
  ('code_pr_core',          300, 'Admin: core PR merged'),
  ('code_pr_adapter',       150, 'Admin: adapter/tooling PR merged'),
  ('code_pr_docs',           50, 'Admin: documentation PR merged'),
  ('tag_wiki_author',        20, 'Admin: tag wiki authored'),
  ('tag_wiki_approved',      10, 'Admin: tag wiki approved'),
  ('qa_bug_fixed',           15, 'Admin: QA verified bug fix'),
  ('marathon_writer',      1000, 'Admin: >=3 completed works each >=50k words; lifetime title')
ON CONFLICT (event_type) DO UPDATE
  SET description = EXCLUDED.description;


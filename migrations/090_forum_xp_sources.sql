-- Phase 9: Unify XP ledgers — add forum-specific XP source defs and backfill.
-- This migration:
-- 1. Adds forum XP source definitions to xp_source_defs
-- 2. Backfills exp_events rows into xp_events for historical continuity
-- 3. Syncs users.exp → users.xp for all users (one-time)

-- 1. Add forum-specific XP source definitions
INSERT INTO xp_source_defs (event_type, xp_amount, daily_cap, description)
VALUES
  ('forum_post_create', 2, 100,  'Forum post or topic created'),
  ('post_reacted',       5, 50,   'Post received a reaction (upvote equivalent)'),
  ('mod_received',       1, 3,    'Positive moderation action on own post'),
  ('poll_voted',         3, 30,   'Voted on a forum poll')
ON CONFLICT (event_type) DO UPDATE
  SET xp_amount = EXCLUDED.xp_amount,
      daily_cap = EXCLUDED.daily_cap,
      description = EXCLUDED.description;

-- 2. Backfill exp_events into xp_events for historical continuity
INSERT INTO xp_events (user_id, event_type, xp, source_ref, created_at)
SELECT user_id, event_type, amount,
       CONCAT_WS(':', reference_type, reference_id),
       created_at
FROM exp_events
ON CONFLICT DO NOTHING;

-- 3. Sync users.exp → users.xp for all users (one-time)
UPDATE users SET xp = users.exp WHERE users.exp != users.xp;

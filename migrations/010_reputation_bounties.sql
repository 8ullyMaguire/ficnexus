-- 010 — Reputation bounties + XP level-up reset semantics
-- Number is the next free slot after 009; never reuse on prod.

-- 1. Extend xp_source_defs with v3 anti-gaming columns (engine reads these).
ALTER TABLE xp_source_defs
  ADD COLUMN IF NOT EXISTS cooldown_seconds bigint,
  ADD COLUMN IF NOT EXISTS min_value bigint,
  ADD COLUMN IF NOT EXISTS rate_limit_per_period bigint,
  ADD COLUMN IF NOT EXISTS streak_window_days integer,
  ADD COLUMN IF NOT EXISTS streak_multiplier numeric;

-- 2. Per-event daily cap becomes weekly for the engagement dividend (already in v3 seed).
--    Tweak the engagement_dividend to a per-week rate (7d) cap — keep daily in schema, engine honours rate_limit_per_period (weekly).

-- 2. Bounties: a reputation stake users create for "write more of X".
CREATE TABLE IF NOT EXISTS bounty_pots (
    id              serial PRIMARY KEY,
    creator_id      integer NOT NULL REFERENCES users(id),
    target_type     text NOT NULL,               -- work | series | tag | meta
    target_ref      text NOT NULL,               -- url_id / series id / tag slug / label
    amount          integer NOT NULL CHECK (amount > 0),
    goal_desc       text,
    expiry_at       timestamptz NOT NULL,
    status          text NOT NULL DEFAULT 'open' CHECK (status IN ('open','claimed','resolved','cancelled','slashed')),
    claim_ref       text,                        -- proof the goal was met (e.g. new work url_id / PR ref)
    claimant_id     integer REFERENCES users(id),
    resolver_id     integer REFERENCES users(id),
    resolved_at     timestamptz,
    created_at      timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT bounty_pots_resolved_chk CHECK ((status='open') OR (claim_ref IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS idx_bounty_pots_status_created ON bounty_pots (status, created_at);
CREATE INDEX IF NOT EXISTS idx_bounty_pots_creator        ON bounty_pots (creator_id);

-- 3. Bounty resolution log (audit trail only — reputation movement is the real source).
CREATE TABLE IF NOT EXISTS bounty_resolutions (
    id          serial PRIMARY KEY,
    pot_id      integer NOT NULL REFERENCES bounty_pots(id),
    resolver_id integer NOT NULL REFERENCES users(id),
    action      text NOT NULL,                   -- resolved | slashed | cancelled
    note        text,
    rep_moved   integer NOT NULL,                -- delta on the CLAIMANT's reputation (claimant gets amount; curator fee may reduce)
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- 4. XP events: add optional `level_up` flag + residual for the reset semantics.
ALTER TABLE xp_events
  ADD COLUMN IF NOT EXISTS level_up     boolean DEFAULT false,
  ADD COLUMN IF NOT EXISTS xp_residual  integer DEFAULT 0;

-- 5. Internal level_up XP source def (never awarded directly; emitted by engine on level crossing).
INSERT INTO xp_source_defs (event_type, xp_amount, description)
VALUES ('level_up', 0, 'INTERNAL: marks a level transition; resets xp to residual')
ON CONFLICT (event_type) DO UPDATE SET description = EXCLUDED.description;

-- 6. Idle tick: small XP for authenticated presence, heavily capped (2 XP/tick, 1440/day, 720/wk).
INSERT INTO xp_source_defs (event_type, xp_amount, daily_cap, rate_limit_per_period, description)
VALUES ('idle_tick', 2, 1440, 720, 'Passive: 2 XP per authenticated presence tick (capped 1440/day = 2880/wk)')
ON CONFLICT (event_type) DO UPDATE
  SET xp_amount = EXCLUDED.xp_amount,
      daily_cap = EXCLUDED.daily_cap,
      rate_limit_per_period = EXCLUDED.rate_limit_per_period,
      description = EXCLUDED.description;

-- 6b. Completion-bonus source defs (finishing long works > consistent publishing).
--     xp_amount here is the *base*; the actual XP is scaled in app code via
--     award_scaled_xp (completion = 100 + 2 * words/1000). These rows exist for
--     cap/streak config + dedup, and so the engine recognizes the event_type.
INSERT INTO xp_source_defs (event_type, xp_amount, description)
VALUES
  ('work_complete_qualified', 100, 'Completion bonus: status->complete, >=5k words; scaled 100 + 2*(words/1000) by caller'),
  ('completion_streak_bonus', 50,  '50*N XP on Nth completed work in 60d (cap 500)'),
  ('long_work_dividend',      10,   'Passive 10 XP/min reading-time for completed >=20k-word works favorited/bookmarked'),
  ('marathon_writer',         1000, 'Admin-awarded: >=3 completed works each >=50k words; lifetime title')
ON CONFLICT (event_type) DO UPDATE
  SET description = EXCLUDED.description;

-- 6c. work_chapter / work_publish base defs (used by approval path + streak).
--     work_chapter: 15 xp, daily cap 150 (one per chapter up to 150/day).
--     work_publish: 100 xp, >=5k-word gate enforced at call site.
INSERT INTO xp_source_defs (event_type, xp_amount, daily_cap, description)
VALUES
  ('work_chapter',  15,  150, 'Chapter published (>=1k words, qualifier at call site)'),
  ('work_publish',  100, NULL, 'First publish of a >=5k-word work')
ON CONFLICT (event_type) DO UPDATE
  SET xp_amount = EXCLUDED.xp_amount,
      daily_cap = EXCLUDED.daily_cap,
      description = EXCLUDED.description;

-- 7. Streak config: which event_type hits a boost_at threshold and by how much.
--    MUST run AFTER all referenced xp_source_defs are seeded (FK constraint).
CREATE TABLE IF NOT EXISTS xp_sources_streaks (
    event_type       text NOT NULL REFERENCES xp_source_defs(event_type),
    boost_at         integer NOT NULL DEFAULT 5,
    multiplier       numeric NOT NULL
);
INSERT INTO xp_sources_streaks (event_type, boost_at, multiplier)
VALUES ('work_chapter', 5, 1.5)
ON CONFLICT DO NOTHING;
INSERT INTO xp_sources_streaks (event_type, boost_at, multiplier)
VALUES ('work_complete_qualified', 3, 1.0)
ON CONFLICT DO NOTHING;

-- Reputation: spendable currency for bounties. Index for /leaderboard + bounty spend checks.
CREATE INDEX IF NOT EXISTS idx_users_xp          ON users (xp);
CREATE INDEX IF NOT EXISTS idx_users_reputation  ON users (reputation DESC);

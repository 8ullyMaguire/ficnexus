-- 074_translation: unified content translation store + universal curator
-- proposals. Implements docs/plans/translation-everything.md (M1+M2 tables).
-- NOTE: the existing `translations` table is the UI-strings store
-- (locale_code/namespace/key/value, served by GET /api/v1/translations) —
-- UI strings keep using it; this migration adds the content-translation store.

-- ── Unified content translation store ───────────────────────────────
-- One row per (target, field, locale). status machine:
--   machine (LLM, shown badged) → approved (consensus) / outdated / dismissed.
CREATE TABLE IF NOT EXISTS translation_strings (
  id            bigserial PRIMARY KEY,
  target_type   text NOT NULL,   -- work_meta|chapter|request|request_answer|forum_topic|forum_post|comment|review|doc
  target_id     text NOT NULL,   -- id/url_id/composite key, always text
  field         text NOT NULL,   -- title|body|content_html|pitch|summary
  locale        text NOT NULL,   -- es|de|fr|pt-BR|zh|...
  source_hash   text NOT NULL,   -- sha256 of source text; mismatch ⇒ outdated
  source_lang   text,            -- detected once, cached
  text          text NOT NULL,
  status        text NOT NULL DEFAULT 'machine'
                CHECK (status IN ('machine','approved','outdated','dismissed')),
  origin        text NOT NULL DEFAULT 'llm'
                CHECK (origin IN ('llm','user')),
  model         text,
  approved_proposal_id bigint,    -- proposal that made this approved
  created_by    integer REFERENCES users(id) ON DELETE SET NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  updated_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (target_type, target_id, field, locale)
);
CREATE INDEX IF NOT EXISTS idx_translation_strings_lookup
  ON translation_strings (target_type, target_id, locale);
CREATE INDEX IF NOT EXISTS idx_translation_strings_status
  ON translation_strings (status, updated_at DESC);

-- ── Universal curator proposals (one queue for everything) ──────────
CREATE TABLE IF NOT EXISTS proposals (
  id            bigserial PRIMARY KEY,
  kind          text NOT NULL,   -- translate|content_fix|metadata_fix|post_edit|request_edit|doc_edit|work_deletion|collection_add|ui_string
  target_type   text NOT NULL,
  target_id     text NOT NULL,
  payload       jsonb NOT NULL DEFAULT '{}'::jsonb,
  proposer_id   integer REFERENCES users(id) ON DELETE SET NULL,
  source        text NOT NULL DEFAULT 'user'
                CHECK (source IN ('user','llm','moderator','system')),
  status        text NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','approved','dismissed','superseded')),
  supersedes    bigint REFERENCES proposals(id),
  decided_by    integer REFERENCES users(id),
  decided_at    timestamptz,
  note          text,
  created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_proposals_queue
  ON proposals (status, created_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_proposals_target
  ON proposals (kind, target_type, target_id);

CREATE TABLE IF NOT EXISTS proposal_votes (
  proposal_id  bigint REFERENCES proposals(id) ON DELETE CASCADE,
  curator_id   integer NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  decision     text NOT NULL CHECK (decision IN ('approve','dismiss')),
  created_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (proposal_id, curator_id)
);

-- ── Reputation sources for the translation flywheel ─────────────────
-- Points = leaderboard currency (no XP/levels). daily_cap bounds farming.
INSERT INTO xp_source_defs (event_type, xp_amount, daily_cap, description) VALUES
  ('translation_approved',  15, NULL, 'Community translation approved by curator consensus'),
  ('translation_improved',  25, 50,   'Translation proposal superseded an approved one'),
  ('translation_reviewed',   2, 40,   'Curator decision cast on a pending translation proposal'),
  ('translation_flagged',    1, 10,   'Reported a bad machine translation')
ON CONFLICT (event_type) DO NOTHING;

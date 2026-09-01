-- Trust system enhancements:
-- 1. trust_loss_count: tracks how many times trust was lost (for diminishing recovery)
-- 2. Configurable via env vars (TRUST_PREFERENCE_SIMILARITY, TRUST_SIMILARITY_BOOST, TRUST_RECOVERY_DECAY)

ALTER TABLE public.users
  ADD COLUMN IF NOT EXISTS trust_loss_count integer DEFAULT 0 NOT NULL;

-- Index for the trust promotion query to efficiently find users with loss history.
CREATE INDEX IF NOT EXISTS idx_users_trust_loss ON public.users (trust_loss_count) WHERE trust_loss_count > 0;

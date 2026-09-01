-- 067 — GDPR compliance: consent records + account erasure support
--
-- 1. user_consents — records the consent a user gave at registration (and
--    any future consent types). GDPR Art. 7 requires demonstrable consent.
-- 2. users.deleted_at — soft-erasure marker used by the account-deletion
--    flow. Personal data (email, password_hash, bio, kindle_email) is
--    anonymized at deletion time; the row is kept only to preserve foreign
--    keys on comments/forum posts, with all PII stripped.

CREATE TABLE IF NOT EXISTS public.user_consents (
    id          bigserial PRIMARY KEY,
    user_id     integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    consent_type text NOT NULL DEFAULT 'tos',          -- 'tos' | 'privacy' | 'cookies'
    version     text NOT NULL DEFAULT '1.0',           -- consent version shown
    granted_at  timestamptz NOT NULL DEFAULT now(),
    ip          inet,
    CONSTRAINT user_consents_user_type_key UNIQUE (user_id, consent_type, version)
);

CREATE INDEX IF NOT EXISTS idx_user_consents_user ON public.user_consents (user_id);

-- Soft-erasure marker on users.
ALTER TABLE public.users ADD COLUMN IF NOT EXISTS deleted_at timestamptz;

-- Index for the cleanup sweep.
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON public.users (deleted_at);

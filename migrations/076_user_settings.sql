-- User format preferences for multi-format downloads.
--
-- Adds a settings JSONB column to users table to store per-user preferences
-- like preferred download formats. Existing users get default {"formats": ["epub"]}.

ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS settings jsonb NOT NULL DEFAULT '{"formats": ["epub"]}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_users_settings_formats
    ON public.users USING gin (settings);

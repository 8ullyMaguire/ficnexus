-- 066 — Plugin marketplace: rec_strategy + search kinds
--
-- Extends the unified extension marketplace (014) with the two new plugin
-- kinds required by the plugin-marketplace roadmap:
--   * rec_strategy — user-shared recommendation strategies
--   * search       — custom search backends / UI extensions
--
-- The `extensions_kind_check` constraint is recreated with the widened set.
-- Postgres has no ADD CONSTRAINT IF NOT EXISTS for CHECK, so a DO block
-- makes the untracked-history deploy path safe.

DO $$ BEGIN
    ALTER TABLE public.extensions DROP CONSTRAINT IF EXISTS extensions_kind_check;
    ALTER TABLE public.extensions
        ADD CONSTRAINT extensions_kind_check
        CHECK (kind = ANY (ARRAY[
            'recipe'::text, 'theme'::text, 'layout'::text, 'view'::text,
            'skin'::text, 'saved_search'::text, 'profile'::text,
            'rec_strategy'::text, 'search'::text
        ]));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Config blob for a plugin: strategy params, skin CSS, search backend URL.
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS config_json jsonb NOT NULL DEFAULT '{}'::jsonb;

-- Mark existing seed data public so the gallery has content on first deploy.
UPDATE public.extensions SET is_public = true WHERE is_public = false AND author_id = 1;

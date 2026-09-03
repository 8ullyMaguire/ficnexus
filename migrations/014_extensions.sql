-- 014 — Extension marketplace upgrade
--
-- The baseline already ships an `extensions` table (kind ∈ recipe/theme/
-- layout/view, globally-unique slug, `gate_level`, `stats` jsonb). This
-- migration extends it in place into the unified marketplace:
--
--   * wider `kind` set (adds skin, saved_search, profile)
--   * per-category unique slugs (remixes may reuse a slug in another kind)
--   * `is_public` / `is_verified` visibility + review flags
--   * `installs` / `rating` promoted to real columns (stats jsonb stays for
--     legacy readers)
--   * deduplicated `extension_installs` + 1..5 star `extension_ratings`
--
-- Publishing is trust-gated in code (TL2+, see services::trust).

-- Widen the allowed kinds and tiers. (Postgres has no ADD CONSTRAINT IF NOT
-- EXISTS — the DO blocks make re-runs and the untracked-history deploy path
-- safe: a duplicate constraint name is swallowed.)
DO $$ BEGIN
    ALTER TABLE public.extensions DROP CONSTRAINT IF EXISTS extensions_kind_check;
    ALTER TABLE public.extensions
        ADD CONSTRAINT extensions_kind_check
        CHECK (kind = ANY (ARRAY[
            'recipe'::text, 'theme'::text, 'layout'::text, 'view'::text,
            'skin'::text, 'saved_search'::text, 'profile'::text
        ]));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

DO $$ BEGIN
    ALTER TABLE public.extensions DROP CONSTRAINT IF EXISTS extensions_tier_check;
    ALTER TABLE public.extensions
        ADD CONSTRAINT extensions_tier_check
        CHECK (tier = ANY (ARRAY[
            'config'::text, 'recipe'::text, 'service'::text, 'wasm'::text
        ]));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Slugs are unique per category, not globally.
DO $$ BEGIN
    ALTER TABLE public.extensions DROP CONSTRAINT IF EXISTS extensions_slug_key;
    ALTER TABLE public.extensions DROP CONSTRAINT IF EXISTS extensions_kind_slug_key;
    ALTER TABLE public.extensions
        ADD CONSTRAINT extensions_kind_slug_key UNIQUE (kind, slug);
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Visibility + review flags.
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS is_public boolean NOT NULL DEFAULT false;
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS is_verified boolean NOT NULL DEFAULT false;
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS meta jsonb NOT NULL DEFAULT '{}'::jsonb;

-- Install + rating stats as first-class columns (stats jsonb remains for any
-- legacy reader that still consults it).
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS installs integer NOT NULL DEFAULT 0;
ALTER TABLE public.extensions ADD COLUMN IF NOT EXISTS rating real NOT NULL DEFAULT 0;

-- Tighten ownership: author_id and description are always populated.
ALTER TABLE public.extensions ALTER COLUMN author_id SET NOT NULL;
ALTER TABLE public.extensions ALTER COLUMN description SET NOT NULL;
ALTER TABLE public.extensions ALTER COLUMN description SET DEFAULT '';

-- The baseline table shipped without an id default (and the sequence was
-- dropped during migration consolidation) — recreate both.
CREATE SEQUENCE IF NOT EXISTS public.extensions_id_seq
    AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.extensions_id_seq OWNED BY public.extensions.id;
ALTER TABLE public.extensions ALTER COLUMN id SET DEFAULT nextval('public.extensions_id_seq');
SELECT setval('public.extensions_id_seq', COALESCE((SELECT MAX(id) FROM public.extensions), 1));

-- Deduplicated installs: one row per (user, extension).
CREATE TABLE IF NOT EXISTS public.extension_installs (
    extension_id bigint NOT NULL REFERENCES public.extensions(id) ON DELETE CASCADE,
    user_id      integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    installed_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT extension_installs_user_id_extension_id_key
        UNIQUE (user_id, extension_id)
);

-- 1..5 star ratings, one per user per extension.
CREATE TABLE IF NOT EXISTS public.extension_ratings (
    extension_id bigint NOT NULL REFERENCES public.extensions(id) ON DELETE CASCADE,
    user_id      integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    rating       smallint NOT NULL CHECK (rating BETWEEN 1 AND 5),
    created_at   timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT extension_ratings_user_id_extension_id_key
        UNIQUE (user_id, extension_id)
);

CREATE INDEX IF NOT EXISTS idx_extensions_kind_public
    ON public.extensions (kind, is_public);
CREATE INDEX IF NOT EXISTS idx_extensions_author ON public.extensions (author_id);
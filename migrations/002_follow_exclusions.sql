-- 002_follow_exclusions.sql
--
-- Follow exclusions: per-follow filters that drop specific works, series, or
-- fandoms out of the follow updates feed. Each exclusion is attached to a
-- single `follows` row (ON DELETE CASCADE) and removes one target; exactly
-- one of the target columns (work / series / fandom) must be set per row.
--
-- A work is hidden from the feed when any of its exclusions match:
--   * work   — its work_id equals exclude_work_id
--   * series — it belongs to a series whose id equals exclude_series_id
--   * fandom — any of its fandom tags (tags.tag_type_id = 1) equals
--              exclude_fandom

CREATE TABLE public.follow_exclusions (
    id bigint NOT NULL,
    follow_id bigint NOT NULL,
    exclude_type text NOT NULL,
    exclude_work_id integer,
    exclude_series_id integer,
    exclude_fandom text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Exactly one target column must be populated.
ALTER TABLE ONLY public.follow_exclusions
    ADD CONSTRAINT follow_exclusions_target_check CHECK (
        (num_nonnulls(exclude_work_id, exclude_series_id, exclude_fandom) = 1)
    );

-- Only the three supported exclusion kinds are allowed.
ALTER TABLE ONLY public.follow_exclusions
    ADD CONSTRAINT follow_exclusions_type_check CHECK (
        (exclude_type = ANY (ARRAY['work'::text, 'series'::text, 'fandom'::text]))
    );

CREATE SEQUENCE public.follow_exclusions_id_seq
    AS bigint
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.follow_exclusions_id_seq OWNED BY public.follow_exclusions.id;

ALTER TABLE ONLY public.follow_exclusions
    ALTER COLUMN id SET DEFAULT nextval('public.follow_exclusions_id_seq'::regclass);

ALTER TABLE ONLY public.follow_exclusions
    ADD CONSTRAINT follow_exclusions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.follow_exclusions
    ADD CONSTRAINT follow_exclusions_follow_id_fkey
        FOREIGN KEY (follow_id) REFERENCES public.follows (id) ON DELETE CASCADE;

-- Keep a follow's exclusions cheap to look up (and safe to cascade from).
CREATE INDEX idx_follow_exclusions_follow_id
    ON public.follow_exclusions USING btree (follow_id);

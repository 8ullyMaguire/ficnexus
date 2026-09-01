-- 012 — User favorite tags (AO3-style tag following)
--
-- `series.rs` renders an author's followed tags ("Favorite Tags") from
-- `user_favorite_tags`, a table referenced by the code but missing from every
-- migration. Creates it here so the query resolves on all environments.
--
-- Users follow tags; the join is unique per (user_id, tag_id) and cascades on
-- user deletion. Insertion happens through the tag-follow endpoints; this
-- migration only establishes the schema.

CREATE TABLE public.user_favorite_tags (
    user_id integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    tag_id  integer NOT NULL REFERENCES public.tags(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_favorite_tags_user_id_tag_id_key UNIQUE (user_id, tag_id)
);

CREATE INDEX idx_user_favorite_tags_tag ON public.user_favorite_tags (tag_id);

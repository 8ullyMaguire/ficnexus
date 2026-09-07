-- Curator social link proposals (F7+ in the audit plan).
--
-- Mirrors the existing `curator_metadata_proposals` / `curator_metadata_votes`
-- pattern. A curator (role >= 5) proposes adding or removing a social link;
-- other curators vote; once the quorum is reached (VOTE_QUORUM votes with
-- net >= VOTE_NET_MIN), the change is applied.
--
-- The proposal flow is the same as metadata fixes: 3 other curators with
-- net >= 1 → approve; net < 0 → reject.

CREATE TABLE IF NOT EXISTS public.author_social_proposals (
    id bigserial NOT NULL,
    profile_id integer NOT NULL REFERENCES public.author_profiles(id) ON DELETE CASCADE,
    action text NOT NULL CHECK (action IN ('add', 'remove')),
    platform text NOT NULL DEFAULT '',
    url text NOT NULL DEFAULT '',
    label text NOT NULL DEFAULT '',
    social_id integer REFERENCES public.author_socials(id) ON DELETE SET NULL,
    reason text NOT NULL DEFAULT '',
    proposed_by integer NOT NULL REFERENCES public.users(id),
    status text NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected')),
    upvotes integer NOT NULL DEFAULT 0,
    downvotes integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    decided_at timestamptz,
    decided_by integer REFERENCES public.users(id),
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_author_social_proposals_status
    ON public.author_social_proposals USING btree (status, created_at);
CREATE INDEX IF NOT EXISTS idx_author_social_proposals_profile
    ON public.author_social_proposals USING btree (profile_id, status);

CREATE TABLE IF NOT EXISTS public.author_social_votes (
    proposal_id bigint NOT NULL REFERENCES public.author_social_proposals(id) ON DELETE CASCADE,
    user_id integer NOT NULL REFERENCES public.users(id),
    vote smallint NOT NULL CHECK (vote IN (1, -1)),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (proposal_id, user_id)
);

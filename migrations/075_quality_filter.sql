-- Quality filter support: hidden fic_info rows + curator quorum proposals.
--
-- 1. Add `hidden` column to fic_info so quarantined works can be inserted
--    but excluded from public listings until a curator reviews them.
-- 2. Create curator_quorum_proposals table for moderation decisions
--    (quality filter auto-reports suspicious works here).

ALTER TABLE public.fic_info
    ADD COLUMN IF NOT EXISTS hidden boolean NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_fic_info_hidden
    ON public.fic_info USING btree (hidden);

CREATE TABLE IF NOT EXISTS public.curator_quorum_proposals (
    id bigserial NOT NULL,
    url_id text NOT NULL,
    proposal_type text NOT NULL DEFAULT 'quality_filter',
    reason text NOT NULL DEFAULT '',
    status text NOT NULL DEFAULT 'pending'
        CONSTRAINT curator_quorum_proposals_status_check
        CHECK (status IN ('pending', 'approved', 'rejected')),
    proposed_by integer REFERENCES public.users(id),
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    resolved_at timestamp with time zone,
    resolved_by integer REFERENCES public.users(id)
);

ALTER TABLE ONLY public.curator_quorum_proposals
    ADD CONSTRAINT curator_quorum_proposals_pkey PRIMARY KEY (id);

CREATE INDEX IF NOT EXISTS idx_curator_quorum_proposals_status
    ON public.curator_quorum_proposals USING btree (status);

CREATE INDEX IF NOT EXISTS idx_curator_quorum_proposals_url_id
    ON public.curator_quorum_proposals USING btree (url_id);

-- Work deletion requests.
--
-- Any logged-in user can request deletion of a work they did not upload (or
-- that a curator must authorise). The request lands in this queue; a curator
-- or admin resolves it via /api/curator/work-deletions/{id}/resolve. A user
-- who is the uploader (or a curator / admin) deletes a work immediately and
-- never creates a row here.

CREATE TABLE public.work_delete_proposals (
    id bigserial NOT NULL,
    url_id text NOT NULL,
    reason text NOT NULL DEFAULT ''::text,
    proposed_by integer NOT NULL REFERENCES public.users(id),
    status text NOT NULL DEFAULT 'pending'::text
        CONSTRAINT work_delete_proposals_status_check
        CHECK (status IN ('pending'::text, 'approved'::text, 'rejected'::text)),
    created_at timestamp with time zone NOT NULL DEFAULT now()
);

ALTER TABLE ONLY public.work_delete_proposals
    ADD CONSTRAINT work_delete_proposals_pkey PRIMARY KEY (id);

CREATE INDEX idx_work_delete_proposals_status
    ON public.work_delete_proposals USING btree (status);

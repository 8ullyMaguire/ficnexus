-- Collection submissions: community curation voting on moderated adds.
--
-- A collection_item_requests row is a pending "please add this work to my
-- collection" request owned by a moderated collection. Today the owner (or a
-- curator) makes the approve/reject call unilaterally.
--
-- This migration layers community curation on top: any logged-in user may
-- signal agree (1) / abstain (0) / disagree (-1). Votes are an upsert keyed on
-- (request_id, user_id), so a member cannot submit the same vote twice and can
-- freely change their mind. The tally is surfaced in the request list and in
-- the unified curator Approvals queue so the owner/curator can weigh community
-- sentiment before deciding. Approve/reject authority is unchanged.

CREATE TABLE public.collection_submission_votes (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    request_id  bigint NOT NULL REFERENCES public.collection_item_requests(id) ON DELETE CASCADE,
    user_id     integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    vote        smallint NOT NULL DEFAULT 0,
    created_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT collection_submission_votes_vote_check
        CHECK (vote IN (-1, 0, 1)),
    CONSTRAINT collection_submission_votes_request_id_user_id_key
        UNIQUE (request_id, user_id)
);

CREATE INDEX collection_submission_votes_request_id_idx
    ON public.collection_submission_votes (request_id);

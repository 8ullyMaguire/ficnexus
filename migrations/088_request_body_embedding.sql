-- Migration 079: Add body_embedding to fic_requests for semantic auto-matching
--
-- When a user creates a text-only fic request (no seed work), embed the
-- title + body and store the vector so `candidates()` can run KNN against
-- rec_embeddings and surface semantically similar works automatically.

ALTER TABLE public.fic_requests
    ADD COLUMN body_embedding public.vector(768);

-- Index for fast KNN lookups (cosine distance).
CREATE INDEX IF NOT EXISTS fic_requests_embedding_idx
    ON public.fic_requests
    USING ivfflat (body_embedding vector_cosine_ops)
    WITH (lists = 100);

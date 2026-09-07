-- forum-core: polls + options + votes (FicHub spec §1.4, 075_forum_polls).
--
-- Stripped of `OWNER TO fichub` for crate portability.

CREATE TABLE IF NOT EXISTS forum_polls (
    id              BIGSERIAL PRIMARY KEY,
    topic_id        BIGINT NOT NULL UNIQUE REFERENCES forum_topics(id) ON DELETE CASCADE,
    question        TEXT NOT NULL,
    max_selections  INT  NOT NULL DEFAULT 1,
    allow_change    BOOL NOT NULL DEFAULT TRUE,
    close_at        TIMESTAMPTZ,             -- NULL = no auto-close
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS forum_poll_options (
    id          BIGSERIAL PRIMARY KEY,
    poll_id     BIGINT NOT NULL REFERENCES forum_polls(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    position    INT  NOT NULL DEFAULT 0,
    vote_count  BIGINT NOT NULL DEFAULT 0    -- denormalized for live bars
);
CREATE INDEX IF NOT EXISTS idx_forum_poll_options_poll ON forum_poll_options (poll_id, position);

CREATE TABLE IF NOT EXISTS forum_poll_votes (
    poll_id     BIGINT NOT NULL REFERENCES forum_polls(id) ON DELETE CASCADE,
    option_id   BIGINT NOT NULL REFERENCES forum_poll_options(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,             -- FK users(id) added by embedding app
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (poll_id, user_id, option_id)
);
CREATE INDEX IF NOT EXISTS idx_forum_poll_votes_option ON forum_poll_votes (option_id);
CREATE INDEX IF NOT EXISTS idx_forum_poll_votes_user ON forum_poll_votes (user_id);

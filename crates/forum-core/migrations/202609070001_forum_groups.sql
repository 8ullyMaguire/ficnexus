-- forum-core: groups + membership (FicHub spec §1.1, 072_forum_groups).
--
-- Mirror of the embedding-app migration, stripped of `OWNER TO fichub`
-- and any other ficnexus-specific bits. The crate stays generic over
-- the user table — the `users(id)` FKs are added by the embedding app
-- in its own migration (FicHub: 041).

CREATE TABLE IF NOT EXISTS forum_groups (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT UNIQUE NOT NULL,
    slug            TEXT UNIQUE NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    cover_url       TEXT,
    is_private      BOOL NOT NULL DEFAULT FALSE,
    is_system       BOOL NOT NULL DEFAULT FALSE,
    owner_id        INT4   NOT NULL,         -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_forum_groups_owner ON forum_groups (owner_id);
CREATE INDEX IF NOT EXISTS idx_forum_groups_system ON forum_groups (is_system) WHERE is_system = TRUE;

CREATE TABLE IF NOT EXISTS forum_group_members (
    group_id    BIGINT NOT NULL REFERENCES forum_groups(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,             -- FK users(id) added by embedding app
    role        TEXT   NOT NULL DEFAULT 'member' CHECK (role IN ('owner','manager','member')),
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    invited_by  INT4,                        -- FK users(id) added by embedding app
    PRIMARY KEY (group_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_forum_group_members_user ON forum_group_members (user_id);
CREATE INDEX IF NOT EXISTS idx_forum_group_members_role ON forum_group_members (role) WHERE role IN ('owner','manager');

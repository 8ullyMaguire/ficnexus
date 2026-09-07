-- forum-core: rooms + room members + messages + user blocks
-- (FicHub spec §1.5, 076_forum_messaging).
--
-- Stripped of `OWNER TO fichub` for crate portability.

CREATE TABLE IF NOT EXISTS forum_rooms (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT,                      -- NULL for DMs; set for group rooms
    is_group        BOOL NOT NULL DEFAULT FALSE,
    creator_id      INT4   NOT NULL,           -- FK users(id) added by embedding app
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ,
    last_message_id BIGINT,                    -- FK forum_messages(id) added after messages table
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_forum_rooms_group ON forum_rooms (is_group) WHERE is_group = TRUE;
CREATE INDEX IF NOT EXISTS idx_forum_rooms_activity ON forum_rooms (last_activity_at DESC);

CREATE TABLE IF NOT EXISTS forum_room_members (
    room_id     BIGINT NOT NULL REFERENCES forum_rooms(id) ON DELETE CASCADE,
    user_id     INT4   NOT NULL,               -- FK users(id) added by embedding app
    role        TEXT   NOT NULL DEFAULT 'member' CHECK (role IN ('owner','moderator','member')),
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at     TIMESTAMPTZ,
    is_muted    BOOL NOT NULL DEFAULT FALSE,
    notify_level TEXT NOT NULL DEFAULT 'all' CHECK (notify_level IN ('all','mentions','none')),
    PRIMARY KEY (room_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_forum_room_members_user ON forum_room_members (user_id);
CREATE INDEX IF NOT EXISTS idx_forum_room_members_active ON forum_room_members (room_id) WHERE left_at IS NULL;

CREATE TABLE IF NOT EXISTS forum_messages (
    id              BIGSERIAL PRIMARY KEY,
    room_id         BIGINT NOT NULL REFERENCES forum_rooms(id) ON DELETE CASCADE,
    author_id       INT4   NOT NULL,           -- FK users(id) added by embedding app
    body            TEXT NOT NULL,             -- markdown
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    edited_at       TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    is_hidden       BOOL NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    search_vector   TSVECTOR
);
CREATE INDEX IF NOT EXISTS idx_forum_messages_room ON forum_messages (room_id, created_at DESC, id DESC) WHERE deleted_at IS NULL AND is_hidden = FALSE;
CREATE INDEX IF NOT EXISTS idx_forum_messages_search ON forum_messages USING GIN (search_vector);

CREATE TABLE IF NOT EXISTS forum_user_blocks (
    blocker_id  INT4   NOT NULL,               -- FK users(id) added by embedding app
    blocked_id  INT4   NOT NULL,               -- FK users(id) added by embedding app
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker_id, blocked_id)
);
CREATE INDEX IF NOT EXISTS idx_forum_user_blocks_blocked ON forum_user_blocks (blocked_id);

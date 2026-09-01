# FicHub Data Models

## Core Entities

### Work (Canonical Story)
One work = one story across all platforms. `fic_info` rows link to it.

```sql
CREATE TABLE works (
    id              SERIAL PRIMARY KEY,
    title           TEXT NOT NULL,
    url_id          TEXT UNIQUE NOT NULL,  -- stable external ID
    embedding       vector(384),           -- nomic-embed-text vector
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);
```

### FicInfo (Per-Source Metadata)
One fic_info per platform source (AO3, RR, SB, etc.).

```sql
CREATE TABLE fic_info (
    id              VARCHAR(128) PRIMARY KEY,  -- source-specific ID
    work_id         INT REFERENCES works(id),
    title           TEXT NOT NULL,
    author          TEXT NOT NULL,
    fandom          TEXT,
    rating          TEXT,           -- K, T, M, E, etc.
    status          TEXT,           -- complete, in-progress, abandoned
    words           INT,
    chapters        INT,
    url             TEXT NOT NULL,
    summary         TEXT,
    site            TEXT,           -- ao3, royalroad, etc.
    search_vector   TSVECTOR,       -- full-text search
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);
```

### User
```sql
CREATE TABLE users (
    id              SERIAL PRIMARY KEY,
    username        TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,
    email           TEXT,
    role            SMALLINT DEFAULT 0,     -- 0=reader, 10=curator, 50=admin
    level           SMALLINT DEFAULT 0,     -- F7 progression level (0-100)
    xp              INT DEFAULT 0,          -- experience points
    rank            TEXT,                   -- display rank title
    reputation      INT DEFAULT 0,
    trust           REAL DEFAULT 0.5,       -- 0.0-1.0 trust score
    locale          TEXT DEFAULT 'en',
    bio             TEXT,
    kindle_email    TEXT,
    is_banned       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Social Tables

### Bookmarks
```sql
CREATE TABLE bookmarks (
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    shelf_id        INT REFERENCES shelves(id),
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, work_id)
);
```

### Ratings
```sql
CREATE TABLE work_ratings (
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    score           SMALLINT CHECK (score BETWEEN 1 AND 5),
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, work_id)
);
```

### Kudos
```sql
CREATE TABLE kudos (
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, work_id)
);
```

### Reviews
```sql
CREATE TABLE reviews (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    body            TEXT NOT NULL,
    score           SMALLINT CHECK (score BETWEEN 1 AND 5),
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ
);
```

### Comments
```sql
CREATE TABLE comments (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    body            TEXT NOT NULL,
    is_hidden       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Follows
```sql
CREATE TABLE follows (
    user_id         INT REFERENCES users(id),
    target_type     TEXT NOT NULL,   -- 'work', 'author', 'fandom', 'tag'
    target_id       TEXT NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, target_type, target_id)
);
```

### Shelves
```sql
CREATE TABLE shelves (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    name            TEXT NOT NULL,
    is_public       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Lists (Curated Collections)
```sql
CREATE TABLE lists (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    title           TEXT NOT NULL,
    description     TEXT,
    is_public       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE list_items (
    list_id         INT REFERENCES lists(id) ON DELETE CASCADE,
    work_id         INT REFERENCES works(id),
    position        INT DEFAULT 0,
    note            TEXT,
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (list_id, work_id)
);
```

## Tag System

### Tags
```sql
CREATE TABLE tags (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    tag_type_id     INT REFERENCES tag_types(id),
    score           REAL DEFAULT 0,        -- weighted vote score
    created_at      TIMESTAMPTZ DEFAULT now(),
    UNIQUE (name, tag_type_id)
);

CREATE TABLE tag_types (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,  -- 'character', 'relationship', 'freeform'
    display_name    TEXT NOT NULL
);

CREATE TABLE fic_tags (
    fic_info_id     VARCHAR(128) REFERENCES fic_info(id),
    tag_id          INT REFERENCES tags(id),
    PRIMARY KEY (fic_info_id, tag_id)
);

CREATE TABLE tag_votes (
    user_id         INT REFERENCES users(id),
    tag_id          INT REFERENCES tags(id),
    value           SMALLINT NOT NULL,     -- 1 or -1
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, tag_id)
);
```

### Tag Score Function
```sql
CREATE FUNCTION update_fic_tag_score() RETURNS TRIGGER AS $$
BEGIN
    UPDATE tags SET score = (
        SELECT COALESCE(SUM(value), 0) FROM tag_votes WHERE tag_id = NEW.tag_id
    ) WHERE id = NEW.tag_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

## Recommendation Engine

### Rec Embeddings
```sql
CREATE TABLE rec_embeddings (
    work_id         INT REFERENCES works(id) PRIMARY KEY,
    embedding       vector(384) NOT NULL,
    source          TEXT NOT NULL,  -- 'nomic', 'collaborative', etc.
    created_at      TIMESTAMPTZ DEFAULT now()
);
-- HNSW index for fast nearest-neighbor search
CREATE INDEX idx_rec_emb_hnsw ON rec_embeddings
    USING hnsw (embedding vector_cosine_ops);
```

### User Signal Vectors
```sql
CREATE TABLE rec_user_signals (
    user_id         INT PRIMARY KEY REFERENCES users(id),
    embedding       vector(384),
    last_computed   TIMESTAMPTZ
);
```

### Recommendation Votes
```sql
CREATE TABLE recommendation_votes (
    user_id         INT REFERENCES users(id),
    suggestion_id   INT,
    value           SMALLINT NOT NULL,  -- 1 or -1
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Forum

### Forum Categories
```sql
CREATE TABLE forum_categories (
    id              SERIAL PRIMARY KEY,
    title           TEXT NOT NULL,
    slug            TEXT UNIQUE NOT NULL,
    description     TEXT,
    sort_order      INT DEFAULT 0,
    min_role        SMALLINT DEFAULT 0,   -- minimum role to post
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Forum Topics
```sql
CREATE TABLE forum_topics (
    id              SERIAL PRIMARY KEY,
    category_id     INT REFERENCES forum_categories(id),
    author_id       INT REFERENCES users(id),
    title           TEXT NOT NULL,
    slug            TEXT,
    status          TEXT DEFAULT 'open',  -- open, locked, pinned
    view_count      BIGINT DEFAULT 0,
    last_post_id    BIGINT,
    search_vector   TSVECTOR,
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);
```

### Forum Posts
```sql
CREATE TABLE forum_posts (
    id              BIGSERIAL PRIMARY KEY,
    topic_id        INT REFERENCES forum_topics(id),
    author_id       INT REFERENCES users(id),
    body            TEXT NOT NULL,
    quote_of        BIGINT REFERENCES forum_posts(id),
    score           SMALLINT DEFAULT 0,   -- mod-assigned score
    is_hidden       BOOLEAN DEFAULT FALSE,
    edited_at       TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Forum Post Reactions
```sql
CREATE TABLE forum_post_reactions (
    post_id         BIGINT REFERENCES forum_posts(id) ON DELETE CASCADE,
    user_id         INT REFERENCES users(id) ON DELETE CASCADE,
    emoji           TEXT NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (post_id, user_id, emoji)
);
```

### Forum Topic Views (Rate-Limited)
```sql
CREATE TABLE forum_topic_views (
    topic_id        BIGINT REFERENCES forum_topics(id) ON DELETE CASCADE,
    user_id         INT4,
    anonymous_id    UUID,
    viewed_at       TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (topic_id, user_id, anonymous_id),
    CONSTRAINT one_identifier CHECK (
        (user_id IS NOT NULL AND anonymous_id IS NULL)
        OR (user_id IS NULL AND anonymous_id IS NOT NULL)
    )
);
```

### Forum Follows
```sql
CREATE TABLE forum_follows (
    user_id         INT REFERENCES users(id),
    topic_id        INT REFERENCES forum_topics(id),
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, topic_id)
);
```

### Forum Read State
```sql
CREATE TABLE forum_read_state (
    user_id         INT REFERENCES users(id),
    topic_id        INT REFERENCES forum_topics(id),
    last_read_at    TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, topic_id)
);
```

### Forum Moderation
```sql
CREATE TABLE forum_mod_grants (
    moderator_id    INT REFERENCES users(id),
    category_id     INT REFERENCES forum_categories(id),
    points_remaining SMALLINT DEFAULT 3,
    window_start    TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (moderator_id, category_id)
);

CREATE TABLE forum_mod_actions (
    id              SERIAL PRIMARY KEY,
    post_id         BIGINT REFERENCES forum_posts(id),
    moderator_id    INT REFERENCES users(id),
    reason          TEXT NOT NULL,       -- insightful, off-topic, etc.
    score_delta     SMALLINT NOT NULL,
    note            TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE forum_metamod_votes (
    id              SERIAL PRIMARY KEY,
    action_id       INT REFERENCES forum_mod_actions(id),
    voter_id        INT REFERENCES users(id),
    rating          SMALLINT CHECK (rating BETWEEN 1 AND 5),
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Forum Bans
```sql
CREATE TABLE forum_bans (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    category_id     INT REFERENCES forum_categories(id),
    reason          TEXT,
    banned_by       INT REFERENCES users(id),
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Progression & Gamification

### XP Events
```sql
CREATE TABLE reputation_events (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    event_type      TEXT NOT NULL,    -- 'post', 'kudos_given', 'tag_voted', etc.
    xp_delta        INT NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

### Quests
```sql
CREATE TABLE user_quests (
    user_id         INT REFERENCES users(id),
    quest_key       TEXT NOT NULL,
    progress        INT DEFAULT 0,
    completed       BOOLEAN DEFAULT FALSE,
    period          TEXT,             -- 'daily', 'weekly'
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, quest_key, period)
);
```

### Reading Stats
```sql
CREATE TABLE reading_stats (
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    status          TEXT DEFAULT 'unread',  -- unread, reading, completed, abandoned
    progress        REAL DEFAULT 0,         -- 0.0-1.0
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, work_id)
);
```

## Notification System

```sql
CREATE TABLE notifications (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    type            TEXT NOT NULL,    -- 'comment_reply', 'follow', 'badge', etc.
    source_user_id  INT,
    target_type     TEXT,
    target_id       TEXT,
    body            TEXT,
    is_read         BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE notification_preferences (
    user_id         INT PRIMARY KEY REFERENCES users(id),
    email_enabled   BOOLEAN DEFAULT FALSE,
    push_enabled    BOOLEAN DEFAULT FALSE
);
```

## Fic Suggestions (Curator Tool)

```sql
CREATE TABLE fic_suggestions (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    url             TEXT NOT NULL,
    title           TEXT,
    notes           TEXT,
    status          TEXT DEFAULT 'pending',  -- pending, approved, rejected
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Requests Board

```sql
CREATE TABLE requests (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    title           TEXT NOT NULL,
    description     TEXT,
    upvotes         INT DEFAULT 0,
    status          TEXT DEFAULT 'open',
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE request_answers (
    id              SERIAL PRIMARY KEY,
    request_id      INT REFERENCES requests(id) ON DELETE CASCADE,
    user_id         INT REFERENCES users(id),
    work_id         INT REFERENCES works(id),
    body            TEXT,
    votes           INT DEFAULT 0,
    is_accepted     BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Work Proposals (Merge/Split)

```sql
CREATE TABLE work_proposals (
    id              SERIAL PRIMARY KEY,
    proposer_id     INT REFERENCES users(id),
    type            TEXT NOT NULL,     -- 'merge' or 'split'
    target_work_ids INT[],             -- work IDs involved
    reason          TEXT,
    status          TEXT DEFAULT 'pending',
    votes_for       INT DEFAULT 0,
    votes_against   INT DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Badges

```sql
CREATE TABLE badge_definitions (
    slug            TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    icon            TEXT,
    tier            TEXT DEFAULT 'bronze'
);

CREATE TABLE user_badges (
    user_id         INT REFERENCES users(id),
    badge_slug      TEXT REFERENCES badge_definitions(slug),
    awarded_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (user_id, badge_slug)
);
```

## Auto-Tagging (Admin Queue)

```sql
CREATE TABLE auto_tag_queue (
    url_id          VARCHAR(128) REFERENCES fic_info(id),
    tag_id          INT REFERENCES tags(id),
    confidence      REAL,
    status          TEXT DEFAULT 'pending',  -- pending, approved, dismissed
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (url_id, tag_id)
);
```

## Content Moderation

```sql
CREATE TABLE content_scan_results (
    url_id          VARCHAR(128) REFERENCES fic_info(id),
    issue           TEXT NOT NULL,    -- 'minor_tag_mismatch', 'mismatched_rating', etc.
    severity        TEXT DEFAULT 'warning',
    auto_action     TEXT,             -- what the auto-scan wants to do
    status          TEXT DEFAULT 'pending',
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Feature Gating

```sql
CREATE TABLE features (
    slug            TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    min_level       SMALLINT DEFAULT 0,
    is_enabled      BOOLEAN DEFAULT TRUE,
    rollout_pct     SMALLINT DEFAULT 100,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE user_features (
    user_id         INT REFERENCES users(id),
    feature_slug    TEXT REFERENCES features(slug),
    enabled         BOOLEAN DEFAULT TRUE,
    PRIMARY KEY (user_id, feature_slug)
);
```

## User Layout & Views

```sql
CREATE TABLE user_layouts (
    user_id         INT REFERENCES users(id),
    page            TEXT NOT NULL,    -- 'home', 'search', etc.
    config          JSONB NOT NULL,
    PRIMARY KEY (user_id, page)
);

CREATE TABLE user_views (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id),
    name            TEXT NOT NULL,
    type            TEXT NOT NULL,    -- 'feed', 'search', 'custom'
    config          JSONB NOT NULL,
    is_pinned       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Themes & Customization

```sql
CREATE TABLE user_themes (
    user_id         INT PRIMARY KEY REFERENCES users(id),
    theme           TEXT DEFAULT 'light',
    accent_color    TEXT,
    font_size       TEXT DEFAULT 'medium',
    custom_css      TEXT
);
```

## Translation System

```sql
CREATE TABLE work_translations (
    work_id         INT REFERENCES works(id),
    locale_code     TEXT NOT NULL,
    title           TEXT,
    summary         TEXT,
    status          TEXT DEFAULT 'draft',
    created_at      TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (work_id, locale_code)
);

CREATE TABLE chapter_translations (
    work_id         INT REFERENCES works(id),
    locale_code     TEXT NOT NULL,
    chapter_num     INT NOT NULL,
    title           TEXT,
    content         TEXT,
    PRIMARY KEY (work_id, locale_code, chapter_num)
);
```

## Export & Cache

```sql
CREATE TABLE export_log (
    id              SERIAL PRIMARY KEY,
    url_id          VARCHAR(128),
    format          TEXT NOT NULL,    -- 'epub', 'mobi', 'pdf', 'azw3'
    file_path       TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE cache_bodies (
    url_id          VARCHAR(128) PRIMARY KEY,
    body            TEXT NOT NULL,
    cached_at       TIMESTAMPTZ DEFAULT now()
);
```

## Request Tracking

```sql
CREATE TABLE request_source (
    id              SERIAL PRIMARY KEY,
    client_id       TEXT,
    user_agent      TEXT,
    ip_address      INET,
    first_seen      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE request_log (
    id              SERIAL PRIMARY KEY,
    source_id       INT REFERENCES request_source(id),
    path            TEXT,
    method          TEXT,
    status_code     SMALLINT,
    duration_ms     INT,
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

## Analytics

```sql
CREATE TABLE search_logs (
    id              SERIAL PRIMARY KEY,
    query           TEXT NOT NULL,
    results_count   INT,
    user_id         INT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE bot_scores (
    client_id       TEXT PRIMARY KEY,
    score           REAL DEFAULT 0,
    shadowbanned    BOOLEAN DEFAULT FALSE,
    last_updated    TIMESTAMPTZ DEFAULT now()
);
```

## Key Relationships Diagram

```
works ──< fic_info (1:N, one work has many source entries)
works ──< bookmarks
works ──< kudos
works ──< work_ratings
works ──< reviews
works ──< comments
works ──< rec_embeddings
works ──< reading_stats

users ──< bookmarks
users ──< kudos
users ──< work_ratings
users ──< reviews
users ──< comments
users ──< follows
users ──< shelves
users ──< lists
users ──< notifications
users ──< reputation_events
users ──< user_badges
users ──< forum_posts
users ──< forum_topics
users ──< requests

forum_categories ──< forum_topics ──< forum_posts
forum_posts ──< forum_post_reactions
forum_posts ──< forum_mod_actions

tags ──< fic_tags ──< fic_info
tags ──< tag_votes
```

# Contract: Forum Polls & Events

**Route Prefix**: `/api/forum/polls`  
**Spec References**: FR-107..118 (polls), FR-119..122 (events — deferred, schema ready)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. Polls (Attached to Topics)

### 1.1 Get Poll (with Topic)

#### GET `/api/forum/polls/{pollId}`

**Response 200**:
```json
{
  "err": 0,
  "poll": {
    "id": 1,
    "topic_id": 42,
    "question": "Favorite Harry Potter book?",
    "max_selections": 1,
    "allow_change": true,
    "close_at": "2026-09-07T12:00:00Z",
    "is_closed": false,
    "total_votes": 23,
    "options": [
      {
        "id": 1,
        "text": "Philosopher's Stone",
        "position": 0,
        "vote_count": 8,
        "user_voted": true
      },
      {
        "id": 2,
        "text": "Chamber of Secrets",
        "position": 1,
        "vote_count": 5,
        "user_voted": false
      }
    ],
    "created_at": "2026-08-31T10:00:00Z"
  }
}
```

**Auth**: Topic read privilege required. `user_voted` only for authenticated users.

---

### 1.2 Vote on Poll

#### POST `/api/forum/polls/{pollId}/vote`

**Request**:
```json
{"option_ids": [1]}  // array for multi-select
```

**Validation**:
- Poll must not be closed (`close_at` passed or `is_closed=true`)
- User must not have voted (unless `allow_change=true`)
- `option_ids` length ≤ `max_selections`
- All options must belong to this poll

**Trust Gate**: **TL1+** (vote on polls).

**Response 200**:
```json
{
  "err": 0,
  "poll": {
    "id": 1,
    "total_votes": 24,
    "options": [
      {"id": 1, "vote_count": 9, "user_voted": true},
      {"id": 2, "vote_count": 5, "user_voted": false}
    ]
  }
}
```

**Side Effects**:
- `forum_poll_votes` upsert (if `allow_change`, delete old + insert new)
- Option `vote_count` denormalized increment/decrement
- Reputation hook: `poll_vote` (+1 default)
- WS `poll_vote` broadcast to topic subscribers: `{poll_id, option_id, vote_counts: [...]}`

---

### 1.3 Change Vote (if `allow_change=true`)

Same endpoint — sends new `option_ids`, replaces previous vote.

---

### 1.4 Close Poll (Admin / Topic Author)

#### POST `/api/forum/polls/{pollId}/close`

**Auth**: Topic author or Curator+.

**Response 200**: Poll with `is_closed=true`.

**Side Effects**: WS `poll_closed` broadcast; scheduled cron also closes at `close_at`.

---

### 1.5 Poll Results (Public)

#### GET `/api/forum/polls/{pollId}/results`

**Response**: Same as Get Poll but with `vote_percentage` per option.

```json
{
  "err": 0,
  "poll": {
    "id": 1,
    "question": "...",
    "total_votes": 24,
    "options": [
      {"id": 1, "text": "...", "vote_count": 9, "percentage": 37.5}
    ]
  }
}
```

---

## 2. Events (Schema Ready, UI Deferred)

### 2.1 Event Data Model (forum_events table — migration 090)

```sql
CREATE TABLE forum_events (
    id              BIGSERIAL PRIMARY KEY,
    topic_id        BIGINT NOT NULL UNIQUE REFERENCES forum_topics(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT,
    location        TEXT,
    start_at        TIMESTAMPTZ NOT NULL,
    end_at          TIMESTAMPTZ,
    timezone        TEXT NOT NULL DEFAULT 'UTC',
    max_attendees   INT,
    rsvp_required   BOOL NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ
);

CREATE TABLE forum_event_rsvps (
    event_id  BIGINT NOT NULL REFERENCES forum_events(id) ON DELETE CASCADE,
    user_id   INT4   NOT NULL,  -- FK users(id)
    status    TEXT NOT NULL DEFAULT 'going' CHECK (status IN ('going','interested','not_going')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (event_id, user_id)
);
```

### 2.2 Contract (When Implemented)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/forum/events` | GET | List events (filter: upcoming/past, category) |
| `/api/forum/events/{id}` | GET | Event detail + RSVPs |
| `/api/forum/events` | POST | Create event (TL2+, attached to topic) |
| `/api/forum/events/{id}` | PATCH | Update event (author/Curator) |
| `/api/forum/events/{id}/rsvp` | POST | RSVP `{status: "going"|"interested"|"not_going"}` |
| `/api/forum/events/{id}/rsvp` | DELETE | Cancel RSVP |

**Trust Gates**: Create TL2+, RSVP TL1+.

**Realtime**: WS `event_rsvp` broadcast, `event_update`.

---

## 3. Rewards (Deferred — Native Reputation Covers)

NodeBB rewards (badges/achievements) are **not ported v1**. Ficnexus uses:
- `reputation_events` for all forum activity
- Trust levels 0-6 for moderation gates
- Leaderboard (`users.reputation`) for status

If needed later: `forum_rewards` table with criteria JSON + `forum_user_rewards` join.
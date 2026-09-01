# Part 8 — Community Features

> **Part 8 of 13** — In Part 7 we gave FicHub its people: users with
> JWTs, bookmarks, ratings, comments, follows, and a notification
> inbox. A download tool became a community. But a community doesn't
> just *exist* — it *does things together*. It asks for stories nobody
> has written yet, curates the best of what exists, follows the authors
> it loves across every source site, subscribes to the archive itself,
> and argues about what to build next.
>
> This part is that doing. We cover the Fic Requests board in
> `src/routes/requests.rs` — where a reader posts a prompt, other
> readers answer with actual works (or paste a URL and let the scraper
> turn it into a work), the crowd votes on which answers fit best, and
> the requester accepts a winner (Chapter 33); reading lists and
> shelves in `lists.rs` and `shelves.rs` — two generations of
> "organize the archive" (Chapter 34); series and author pages in
> `series.rs` and `authors.rs`, including the `next_in_series` link
> that is the highest-conversion click in all of fanfiction (Chapter
> 35); the three RSS/Atom feeds in `routes/feed.rs` and `routes/rss/`
> that let the whole internet subscribe to FicHub (Chapter 36); and
> finally the roadmap consensus engine in `roadmap.rs` — semantic
> clustering, a MaxDiff voting arena, and an Elo-ranked feature
> leaderboard that decides what gets built next (Chapter 37).
>
> By the end of this part you'll have traced a feature idea from a
> reader's typed prompt all the way to a ranked leaderboard, and you'll
> understand the pattern that runs through every community feature: a
> **vote table with a composite primary key** that makes "one user, one
> vote" a database guarantee instead of a hope.

---

## Chapter 33 — The Fic Requests Board

Every fanfiction community has a version of this: a thread where
someone says *"has anyone written a time-travel Harry where he lands in
the Marauders era but refuses to talk to James?"* and a dozen people
reply with links. FicHub's version is the **Fic Requests board** — a
prompt board where answers are *works*, not comments. Let's read the
module doc comment at the top of `src/routes/requests.rs`, because it
is the design document for the whole feature:

```rust
//! Fic Requests — prompt board for fic requests (works-only answers).
//!
//! A request = a prompt ("fics like X", "dark harry, no bashing"). Answers
//! are WORKS (work_id) with an optional one-line pitch. Community up/down
//! votes rank fit (net score only, never public downvote counts); the
//! requester can accept one answer → status 'answered'.
//!
//! Votes: one per user per answer, toggle/retract allowed. No self-vote.
//! Answers per user per request capped at 3 (ANSWER_CAP).
```

Four sentences that pin down every rule we're about to implement. Note
the parenthetical that reads like a policy decision: *"net score only,
never public downvote counts."* The database stores `-1` votes — but
the API only ever exposes the sum. Why? Because a public downvote count
is a weapon: it invites vote-brigading and turns a helpful suggestion
into a social humiliation. A net score keeps the signal ("this answer
isn't a good fit") without the shaming. Small product decisions like
this one are where community features live or die.

### 33.1 The schema: three tables and a PRIMARY KEY that means "one vote"

The board is spread over two migrations: `018_fic_requests.sql` (the
board itself) and `035_fic_request_upvotes.sql` (request-level upvotes
added later, for board ranking). The core tables from migration 018:

```sql
CREATE TABLE IF NOT EXISTS fic_requests (
  id              SERIAL PRIMARY KEY,
  user_id         INT4 NOT NULL REFERENCES users(id),
  title           TEXT NOT NULL,
  body            TEXT NOT NULL DEFAULT '',
  seed_work_id    INT4 REFERENCES works(id),
  status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','answered','closed')),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  closed_at       TIMESTAMPTZ,
  deleted_at      TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS fic_request_answers (
  id          SERIAL PRIMARY KEY,
  request_id  INT4 NOT NULL REFERENCES fic_requests(id),
  user_id     INT4 NOT NULL REFERENCES users(id),
  work_id     INT4 NOT NULL REFERENCES works(id),
  pitch       TEXT NOT NULL DEFAULT '',
  source      TEXT NOT NULL DEFAULT 'user',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at  TIMESTAMPTZ,
  UNIQUE (request_id, work_id)
);

CREATE TABLE IF NOT EXISTS fic_request_answer_votes (
  answer_id   INT4 NOT NULL REFERENCES fic_request_answers(id),
  user_id     INT4 NOT NULL REFERENCES users(id),
  vote        SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (answer_id, user_id)
);
```

Read the constraints like a contract. `status` is a `CHECK` with
exactly three values — `open`, `answered`, `closed` — so a typo like
`'ANSWERED'` is a database error, not a silent bug. `fic_request_answers`
has `UNIQUE (request_id, work_id)`: the same work cannot be suggested
twice for the same request. And `fic_request_answer_votes` has a
*composite* `PRIMARY KEY (answer_id, user_id)` — one row per user per
answer, guaranteed by the database itself.

💡 **Key Concept — a composite primary key is a voting machine.** When
you want "each user can do this thing at most once," the cheapest,
most robust implementation in the world is a table whose primary key
*is* the pair of identifiers: `PRIMARY KEY (answer_id, user_id)`.
There is no application logic to forget, no race condition where two
requests both pass an existence check and both insert. The database
refuses the second row, period. You'll see this exact shape in
`fic_request_upvotes` (migration 035):

```sql
CREATE TABLE IF NOT EXISTS fic_request_upvotes (
    request_id INT4 NOT NULL REFERENCES fic_requests(id) ON DELETE CASCADE,
    user_id    INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (request_id, user_id)
);
```

Same pattern: `PRIMARY KEY (request_id, user_id)`. One upvote per user
per request, forever, no code required. We'll reuse this shape in the
roadmap arena in Chapter 37 — once you've seen it once, you'll spot it
in every real codebase you read.

### 33.2 Creating a request: validate first, insert once

The `create_request` handler in `requests.rs` is the template for every
"user submits something" handler in this codebase. Watch the order of
operations — validate everything *before* touching the database:

```rust
/// POST /api/requests — create a request (auth)
pub async fn create_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRequestBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;
    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest(-1, "title must not be empty".into()));
    }
    if title.chars().count() > 200 {
        return Err(AppError::BadRequest(-1, "title too long (max 200)".into()));
    }
    if body.body.chars().count() > 4000 {
        return Err(AppError::BadRequest(-1, "body too long (max 4000)".into()));
    }
    if let Some(sid) = body.seed_work_id {
        if queries::get_work(&state.db, sid).await?.is_none() {
            return Err(AppError::BadRequest(-1, "seed_work_id not found".into()));
        }
    }

    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO fic_requests (user_id, title, body, seed_work_id)
           VALUES ($1, $2, $3, $4) RETURNING id"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(body.body.trim())
    .bind(body.seed_work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({ "err": 0, "id": id, "msg": "Request created" })))
}
```

A few details worth slowing down on:

- **`chars().count()` instead of `.len()`.** `.len()` counts *bytes*;
  an emoji in a title is 4 bytes but 1 character. `chars().count()` is
  the only honest way to enforce "max 200 characters" on user input.
  This is a classic junior-developer bug that FicHub deliberately
  avoids — and you'll see `chars().count()` in every length check in
  the file.
- **`trim()` before validating.** `"   ".trim()` is empty; `"  hello  "`
  becomes `"hello"`. The handler stores the trimmed value too, so the
  database never accumulates leading/trailing whitespace that would
  break title display and sorting later.
- **The seed work is a foreign key checked by hand.** `seed_work_id`
  is an optional `work_id` — "this request is basically *fics like
  this one*". The handler verifies the work exists *before* inserting,
  so the user gets a clean "seed_work_id not found" instead of a raw
  foreign-key violation from Postgres. That's the difference between an
  API and an error dump.
- **`RETURNING id`** gets the new row's id in one round trip — no
  second `SELECT`.

### 33.3 Listing: the board, two ways

`list_requests` is a public endpoint (no auth) with three query
parameters: `status` (`open` | `answered` | `closed`), `sort`
(`top` | `new`), and `page`. It's a `match` on `sort` that picks one of
two nearly-identical queries. Here's the `top` variant, which is the
interesting one:

```rust
let rows: Vec<(i32, String, String, Option<i32>, String, String, i64, i64)> = match params.sort.as_str() {
    "top" => sqlx::query_as(
        r#"SELECT r.id, r.title, r.body, r.seed_work_id, r.status,
                  r.created_at::text,
                  (SELECT COUNT(*) FROM fic_request_answers a WHERE a.request_id = r.id AND a.deleted_at IS NULL)::bigint,
                  (SELECT COUNT(*) FROM fic_request_upvotes u WHERE u.request_id = r.id)::bigint
           FROM fic_requests r
           WHERE r.deleted_at IS NULL AND r.status = $1
           ORDER BY (SELECT COUNT(*) FROM fic_request_upvotes u WHERE u.request_id = r.id) DESC,
                    (SELECT COUNT(*) FROM fic_request_answers a WHERE a.request_id = r.id AND a.deleted_at IS NULL) DESC,
                    r.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(status)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?,
    ...
```

There are three things to notice. First, the **correlated subqueries**
— `(SELECT COUNT(*) ... WHERE a.request_id = r.id)` runs once per row,
counting that row's answers. Second, those subqueries appear *both* in
the `SELECT` list *and* in `ORDER BY`. SQL lets you repeat them, and
for a 20-row page that's fine. Third, the `deleted_at IS NULL` filter
appears in the WHERE *and inside the subquery count* — soft-deleted
answers must not inflate the answer count.

⚠️ **Watch Out — soft deletes leak into counts unless you filter them
everywhere.** FicHub never `DELETE`s a request or an answer; it sets
`deleted_at = NOW()` (we'll see why in a moment). That means every
single query that counts or lists these rows must remember
`deleted_at IS NULL` — in the outer WHERE *and* in every correlated
subquery. Miss one, and a deleted answer still counts toward
"3 answers" on the request page. This is the tax you pay for soft
deletes: correctness is spread across every query instead of living in
one DELETE statement. The subquery above is the discipline done right —
count only non-deleted answers.

Also note the `top` sort: upvotes first, then answer count, then
recency. A request with 12 upvotes outranks a request with 40 answers
but no upvotes. That's the product decision "what the community wants
matters more than what got answered" — encoded directly in `ORDER BY`.

Why soft delete at all? Look at the delete handler:

```rust
sqlx::query("UPDATE fic_requests SET deleted_at = NOW() WHERE id = $1")
    .bind(id)
    .execute(&state.db)
    .await?;
```

A `DELETE` would destroy the row — and with it the audit trail of who
voted, who answered, and what the community did. A soft delete keeps
the history for moderation while hiding the row from every public
query. The cost is the Watch Out above; the benefit is that a curator
can *undelete* by clearing `deleted_at`, and the modlog (Part 11) can
show what happened.

### 33.4 Answering: the part where the scraper does a backflip

`add_answer` is the heart of the board, and it has a trick. The request
body can specify a `work_id` *directly* — but it can also just paste a
**URL**. If it's a URL, FicHub runs the entire scraper pipeline from
Part 4 *inside the answer handler*: find the right scraper, fetch the
metadata, check the blacklist, and find-or-create the work. Here's the
core of it:

```rust
// Resolve the answer target: either a work_id directly, or a URL that we
// scrape (find-or-create) into a work_id (URL-ingest answers, P6#23).
let work_id: i32 = if let Some(wid) = body.work_id {
    if queries::get_work(&state.db, wid).await?.is_none() {
        return Err(AppError::BadRequest(-1, "work_id not found".into()));
    }
    wid
} else {
    let url = body.url.trim();
    if url.is_empty() {
        return Err(AppError::BadRequest(-1, "work_id or url required".into()));
    }
    // Scrape the URL with the registry (prefers native scrapers) and
    // find-or-create the work. Blacklisted URLs are rejected.
    let scraper = state
        .scraper_registry
        .find_specific_or_fff(url)
        .ok_or_else(|| AppError::BadRequest(-1, "Unsupported URL".into()))?;
    let meta = scraper
        .lookup(&state.http_client, url)
        .await
        .map_err(|e| AppError::BadRequest(-1, format!("Could not fetch URL: {e}")))?;
    let blacklisted = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
    if !blacklisted.is_empty() {
        return Err(AppError::BadRequest(-1, "This fic is blacklisted".into()));
    }
    crate::works::find_or_create_work(&state.db, &meta)
        .await
        .map_err(|e| AppError::BadRequest(-1, format!("Could not save fic: {e}")))?
        .work_id
};
```

Think about what this means for a reader: someone posts "anyone know a
fic where X happens?", and a second reader replies *by pasting the AO3
URL*. FicHub scrapes it, creates the work in the database if it's new,
and stores the answer pointing at a real `work_id` — which means the
answer gets a title, an author, a word count, and a reader page *for
free*. There's no separate "URL answer" concept anywhere downstream;
everything after this point just works with `work_id`. This is the
pattern of **normalizing input at the edge**: convert messy user input
into the canonical form as early as possible, so the rest of the system
never has to handle the messy form.

Note the pieces pulled from earlier parts: `find_specific_or_fff` (the
scraper registry preferring native scrapers — Chapter 17), `lookup` on
the chosen scraper, `check_fic_blacklist` (a bot-defense query), and
`find_or_create_work`. This is the payoff of Parts 4 and 5 — the
scraper you built to power downloads is *now* also the thing that lets
readers answer requests by URL.

The rest of `add_answer` enforces the board rules. First the cap:

```rust
// +3 cap per user per request
let (cnt,): (i64,) = sqlx::query_as(
    "SELECT COUNT(*)::bigint FROM fic_request_answers WHERE request_id = $1 AND user_id = $2 AND deleted_at IS NULL",
)
.bind(id)
.bind(user_id)
.fetch_one(&state.db)
.await?;
if cnt >= ANSWER_CAP {
    return Err(AppError::BadRequest(-1, format!("Max {ANSWER_CAP} answers per request")));
}
```

`ANSWER_CAP` is a `pub const i64 = 3` at the top of the file — a named
constant with a doc comment explaining it's user-approved. Then the
insert, which handles the duplicate gracefully:

```rust
let aid: i32 = match sqlx::query_scalar(
    r#"INSERT INTO fic_request_answers (request_id, user_id, work_id, pitch)
       VALUES ($1, $2, $3, $4) RETURNING id"#,
)
.bind(id)
.bind(user_id)
.bind(work_id)
.bind(body.pitch.trim())
.fetch_one(&state.db)
.await
{
    Ok(aid) => aid,
    Err(e) if e.to_string().contains("duplicate key") => {
        return Err(AppError::BadRequest(-1, "This fic is already an answer".into()));
    }
    Err(e) => return Err(e.into()),
};
```

⚠️ **Watch Out — string-matching database errors is a code smell you
sometimes have to swallow.** The `UNIQUE (request_id, work_id)` from
migration 018 *is* the duplicate check — but Postgres reports it as a
raw error whose message contains `duplicate key`. Matching on the error
string (`e.to_string().contains("duplicate key")`) is fragile — the
message could change between Postgres versions — yet it's the pragmatic
choice here because it avoids a race-free pre-check that could still
lose to the constraint. The alternative (pre-check with SELECT) has a
TOCTOU gap: two concurrent requests could both pass the check, and the
constraint would still fire. So the string match is the honest handling
of a database guarantee. Just keep it in one place, with a comment.

And finally — the notification. When your request gets an answer, you
should *know*:

```rust
if owner_id != user_id {
    let _ = queries::create_notification(
        &state.db,
        owner_id,
        "request_answer",
        "Your fic request got a new answer",
        Some("Someone suggested a work for your request."),
        Some(&format!("/requests/{id}")),
        Some("request"),
        Some(&id.to_string()),
    )
    .await;
}
```

Note the `let _ = ...` and the `if let Ok(Some(...))` wrapping the
owner lookup: **the notification is best-effort and must never fail the
main action.** If the notification insert fails, the answer is still
saved and the user still gets a 200. This "never fail the main action"
convention appears in every notification call site in the codebase —
it's how you add side effects to a request without making the request
fragile. (We met `create_notification` in Part 7, Chapter 32 — the
notification inbox; this is it being fed from a new source.)

### 33.5 Voting: toggle, retract, and the upsert that does both

`vote_answer` handles votes of `1`, `-1`, or `0` (retract). It enforces
two rules by hand before touching the vote table — the answer must
belong to this request, and you can't vote on your own answer:

```rust
if !matches!(body.vote, -1 | 0 | 1) {
    return Err(AppError::BadRequest(-1, "vote must be -1, 0, or 1".into()));
}
let row: Option<(i32, i32)> = sqlx::query_as(
    "SELECT request_id, user_id FROM fic_request_answers WHERE id = $1 AND deleted_at IS NULL",
)
.bind(aid)
.fetch_optional(&state.db)
.await?;
let (rid, answer_owner) = row.ok_or_else(|| AppError::BadRequest(-1, "answer not found".into()))?;
if rid != id {
    return Err(AppError::BadRequest(-1, "answer does not belong to this request".into()));
}
if answer_owner == user_id {
    return Err(AppError::BadRequest(-1, "Cannot vote on your own answer".into()));
}
```

Why check `rid != id` by hand when the URL already contains both ids?
Because the URL is *user input* — someone can POST to
`/api/requests/7/answers/99/vote` where answer 99 belongs to request 3.
Axum happily binds both numbers; the handler must verify they're
consistent. This is the "defense at the boundary" rule: **never trust a
path parameter to be self-consistent.**

Then the actual vote, which is a two-way split — delete for retract,
upsert for everything else:

```rust
if body.vote == 0 {
    sqlx::query("DELETE FROM fic_request_answer_votes WHERE answer_id = $1 AND user_id = $2")
        .bind(aid)
        .bind(user_id)
        .execute(&state.db)
        .await?;
} else {
    sqlx::query(
        r#"INSERT INTO fic_request_answer_votes (answer_id, user_id, vote)
           VALUES ($1, $2, $3)
           ON CONFLICT (answer_id, user_id) DO UPDATE SET vote = EXCLUDED.vote, created_at = NOW()"#,
    )
    .bind(aid)
    .bind(user_id)
    .bind(body.vote)
    .execute(&state.db)
    .await?;
}
```

💡 **Key Concept — `ON CONFLICT ... DO UPDATE` is "insert or change my
mind".** The composite primary key from 33.1 makes a second vote a
conflict — and instead of an error, we *replace* the old vote with the
new one. That's the toggle: vote `1`, then vote `-1` later, and the
row flips. `EXCLUDED` refers to the row you *tried* to insert — so
`vote = EXCLUDED.vote` means "take the new value". Retract is a plain
DELETE. Three states (`1`, `-1`, retracted), one row per user, no
history table needed.

After the vote lands, the handler re-reads the net score so the client
gets the fresh number in the response:

```rust
let (score,): (i64,) = sqlx::query_as(
    "SELECT COALESCE(SUM(vote), 0)::bigint FROM fic_request_answer_votes WHERE answer_id = $1",
)
.bind(aid)
.fetch_one(&state.db)
.await?;

Ok(Json(json!({ "err": 0, "score": score, "my_vote": if body.vote == 0 { Value::Null } else { json!(body.vote) } })))
```

`COALESCE(SUM(vote), 0)` — a sum over zero rows is NULL in SQL, and
`COALESCE` turns that into `0`. This is one of the most common SQL
footguns in the wild: aggregate functions return NULL on empty input.
And notice the response shape: `score` (net, public) and `my_vote`
(your own vote, personal). The API exposes the net to everyone and
*your* vote only to you — which is exactly the "never public downvote
counts" policy from the module comment, implemented in the response
shape.

The request-level upvote (`upvote_request`) is the same pattern with
an `enabled: bool` body instead of a numeric vote:

```rust
if body.enabled {
    sqlx::query(
        r#"INSERT INTO fic_request_upvotes (request_id, user_id) VALUES ($1, $2)
           ON CONFLICT (request_id, user_id) DO NOTHING"#,
    )
    .bind(id)
    .bind(user_id)
    .execute(&state.db)
    .await?;
} else {
    sqlx::query("DELETE FROM fic_request_upvotes WHERE request_id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
}
```

`ON CONFLICT ... DO NOTHING` — the pure idempotent version: upvoting
twice is not an error, it's just nothing. And the self-vote check is
there too: `if owner.0 == user_id { return Err(...Cannot upvote your own request...) }`. Nobody gets to boost their own request's ranking.

### 33.6 Accepting an answer, and the recommender that suggests candidates

The requester — and only the requester — can close the loop by
accepting an answer:

```rust
pub async fn accept_answer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, aid)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;
    let req: Option<(i32, String)> = sqlx::query_as(
        "SELECT user_id, status FROM fic_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let (owner, status) = req.ok_or_else(|| AppError::BadRequest(-1, "request not found".into()))?;
    if owner != user_id {
        return Err(AppError::BadRequest(403, "Only the requester can accept".into()));
    }
    if status == "closed" {
        return Err(AppError::BadRequest(-1, "request is closed".into()));
    }
    ...
    sqlx::query(
        r#"UPDATE fic_requests SET status = 'answered', accepted_answer_id = $2, closed_at = NOW(), updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(aid)
    .execute(&state.db)
    .await?;
```

Two ownership layers in four lines: the handler checks the *requester*
is the one calling, then checks the *answer* belongs to the request.
And like `add_answer`, it fires a best-effort notification — this time
to the answer's author: *"Your answer was accepted!"*. The requester
gets closure, the answerer gets a ping, and the board gets a new
`answered` row on the list page. That's the full lifecycle: `open` →
vote → accept → `answered`.

Finally, the most forward-looking endpoint in the file: `candidates`.
This one reaches *into the recommendation engine* (the entire subject
of Part 9) to suggest works that could answer a request. When a request
has a seed work — "fics like *this one*" — the engine can recommend
similar fics:

```rust
// Ask the recommender engine for similar works.
let rec_query = crate::recommender::RecQuery {
    url_id,
    n: 20,
    site_domain: None,
};
let recs = state
    .recommender_engine
    .get_recommendations(&rec_query, &state.config)
    .await?;

let candidates: Vec<Value> = recs
    .into_iter()
    .map(|r| {
        json!({
            "url_id": r.url_id,
            "title": r.title,
            "author": r.author,
            "score": r.score,
        })
    })
    .collect();
```

The seed work is resolved to a canonical `url_id`, the engine returns
20 scored recommendations, and the frontend can render them as
one-click answers. If there's no seed work, the endpoint gracefully
returns an empty list — `Ok(Json(json!({ "err": 0, "request_id": id,
"candidates": [] })))` — with a comment noting a future search-based
path. Graceful degradation: the feature works fully for requests that
can support it, and harmlessly returns nothing for the rest. This is
also your first taste of `crate::recommender::RecQuery` — we'll meet
the engine behind it properly in Part 9.

🧪 **Try It Yourself — run a request through its whole lifecycle.**
With FicHub running and a logged-in session (grab a token from Part
7's Chapter 28 exercise):

```bash
# 1. Create a request with a seed work
curl -s -X POST http://localhost:8080/api/requests \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"title": "Time travel where the past is the problem",
       "body": "Looking for fics where the time traveler makes things worse",
       "seed_work_id": 1}' | jq .

# 2. Answer it with a URL (the scraper ingests it for you)
curl -s -X POST http://localhost:8080/api/requests/1/answers \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"url": "https://archiveofourown.org/works/12345678",
       "pitch": "This one nails the bittersweet ending"}' | jq .

# 3. Upvote the request, then vote on the answer
curl -s -X POST http://localhost:8080/api/requests/1/upvote \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"enabled": true}' | jq .
curl -s -X POST http://localhost:8080/api/requests/1/answers/1/vote \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"vote": 1}' | jq .

# 4. View the board and the request detail (public)
curl -s "http://localhost:8080/api/requests?status=open&sort=top" | jq .
curl -s http://localhost:8080/api/requests/1 | jq .answers

# 5. If you have a second account: accept the answer as the requester
curl -s -X POST http://localhost:8080/api/requests/1/accept/1 \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' | jq .
```

Then watch the request's `status` flip to `answered` in the list
response — and if you set up a second user in Part 7, check that
user's notification inbox for the `request_answer` notification.
The whole board runs in five curl commands.

---

That's the requests board: a prompt table, an answers table where
answers are *works* (possibly ingested from a bare URL by the Part 4
scraper), a vote table whose composite primary key *is* the "one vote
per user" rule, a soft-delete discipline that preserves the audit
trail, and a notification hook that never fails the main action. Next
up: the quieter half of community features — helping people organize
what they read.


---

## Chapter 34 — Reading Lists & Shelves

Before we write any code, a confession: **FicHub has two ways to
organize works, and they were built at different times by different
minds.** That's not a flaw in the codebase — it's a *history lesson*,
and reading both is how you learn to recognize an older design from a
newer one. Let's read them in the order they were written, so you can
see the design conversation happen.

The older one lives in `src/routes/shelves.rs` (plus migration
`004_shelves_reading_status.sql`). The newer one lives in
`src/routes/lists.rs` plus `src/db/queries.rs` (migration
`019_reading_lists.sql`). Same job — "let users group works" — two
implementations, and the differences are exactly where the lesson is.

### 34.1 The shelf: a flat bag of works (and the lesson in it)

Here's the entire schema for shelves, from migration 004:

```sql
CREATE TABLE IF NOT EXISTS shelves (
    id SERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INT4 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE TABLE IF NOT EXISTS work_shelves (
    id BIGSERIAL PRIMARY KEY,
    shelf_id INT4 NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
    work_id INT4 NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(shelf_id, work_id)
);
```

Two tables: `shelves` (the container) and `work_shelves` (the
membership join). A shelf is *a named, user-owned collection of works
with no order and no per-item metadata*. `UNIQUE(shelf_id, work_id)`
says a work can be in a shelf once — and `UNIQUE(user_id, name)` says
you can't have two shelves named "TBR".

Now look at the handler shape in `shelves.rs` — every one of these is
"auth, verify ownership, call a query function, return JSON". Here's
the create:

```rust
pub async fn create_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateShelfBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest(-1, "Shelf name cannot be empty".into()));
    }
    if body.name.len() > 100 {
        return Err(AppError::BadRequest(-1, "Shelf name too long (max 100 chars)".into()));
    }

    let shelf = queries::create_shelf(
        &state.db,
        user_id,
        &body.name,
        body.description.as_deref().unwrap_or(""),
        body.is_public.unwrap_or(false),
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "shelf": {
            "id": shelf.id,
            "name": shelf.name,
            "description": shelf.description,
            "is_public": shelf.is_public,
            "sort_order": shelf.sort_order,
            "created_at": shelf.created_at.to_rfc3339(),
        }
    })))
}
```

Every validation lives in the handler: empty name, name length, and
then the query. The database helper in `queries.rs` is a thin wrapper:

```rust
pub async fn create_shelf(
    pool: &PgPool,
    user_id: i32,
    name: &str,
    description: &str,
    is_public: bool,
) -> AppResult<Shelf> {
    let row = sqlx::query_as::<_, Shelf>(
        r#"INSERT INTO shelves (user_id, name, description, is_public)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, name, description, is_public, sort_order, created_at"#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .bind(is_public)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
```

💡 **Key Concept — the handler/query split.** Notice what *doesn't*
happen here: there's no SQL in the handler and no auth in the query
function. The handler owns HTTP concerns (auth, validation, response
shaping); `queries.rs` owns SQL. `fetch_one(pool)` returns a typed
`Shelf` row thanks to `query_as`, and the handler turns it into JSON.
This two-layer split — which you've seen in every chapter of this
book — is what keeps a 2,700-line `queries.rs` readable: each function
is one small SQL statement with a clear name, and each handler is a
script of "check, call, respond".

The shelf endpoints also show the *ownership check* pattern that every
one of them repeats. Adding a work to a shelf requires the shelf to be
yours first:

```rust
// Verify the shelf belongs to this user
let shelf = queries::get_shelf(&state.db, body.shelf_id, user_id).await?;
if shelf.is_none() {
    return Err(AppError::NotFound("Shelf not found".into()));
}

queries::add_work_to_shelf(&state.db, body.shelf_id, body.work_id).await?;
```

`get_shelf(pool, shelf_id, user_id)` is scoped in SQL: `WHERE id = $1
AND user_id = $2`. The shelf either exists *and* is yours, or the query
returns nothing — which the handler reports as a 404 "Shelf not found".
There is no separate "does it exist" query followed by an "is it yours"
query. **Scope in the WHERE clause, not in two round trips** — this
pattern (a fetch scoped by both the resource id and the acting user)
appears in every ownership check in FicHub, and it's the single most
reusable query pattern in the codebase.

⚠️ **Watch Out — there are two different route shapes for the same
resource.** Look at the shelf routes from `server.rs`:

```rust
.route("/api/shelves", axum::routing::post(crate::routes::shelves::create_shelf_handler))
.route("/api/shelves", get(crate::routes::shelves::list_shelves_handler))
.route("/api/shelves/{id}", axum::routing::delete(crate::routes::shelves::delete_shelf_handler))
.route("/api/shelves/add", axum::routing::post(crate::routes::shelves::add_work_to_shelf_handler))
.route("/api/shelves/{shelf_id}/works/{work_id}", axum::routing::delete(crate::routes::shelves::remove_work_from_shelf_handler))
.route("/api/shelves/{shelf_id}/works", get(crate::routes::shelves::list_works_in_shelf_handler))
```

Adding a work is `POST /api/shelves/add` with a JSON body
(`{shelf_id, work_id}`) — a flat "action" route. Removing is
`DELETE /api/shelves/{shelf_id}/works/{work_id}` — a nested resource
route. Both work. But they're *inconsistent*, and that inconsistency is
exactly what the reading-list API (which came later) was designed to
fix. When you design a new API, ask: *would a newcomer guess this
route without reading the code?* If not, the route shape is a smell.

There's also a small but telling detail in the shelf code: the length
check uses `body.name.len() > 100` — **bytes**, not characters — and
`body.name.len()` on a string containing non-ASCII characters counts
each UTF-8 byte. A name of 50 Cyrillic characters is 100 bytes and
passes; 51 would fail. This is a subtle i18n bug the reading-list code
later fixes with `chars().count()` (which you saw in Chapter 33). It
won't crash anything, but it's a reminder that *byte length ≠ user
perception of length*.

### 34.2 The reading list: ordered, blurby, shareable

Now the newer design, from `019_reading_lists.sql`:

```sql
CREATE TABLE IF NOT EXISTS reading_lists (
    id SERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS reading_list_items (
    id SERIAL PRIMARY KEY,
    list_id INT4 NOT NULL REFERENCES reading_lists(id) ON DELETE CASCADE,
    work_id INT4 NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    position INT4 NOT NULL DEFAULT 0,
    blurb TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (list_id, work_id)
);

CREATE INDEX IF NOT EXISTS idx_reading_lists_user ON reading_lists (user_id, deleted_at);
CREATE INDEX IF NOT EXISTS idx_reading_list_items_list ON reading_list_items (list_id, position);

COMMENT ON TABLE reading_lists IS 'User-curated ordered reading lists (bundles)';
COMMENT ON TABLE reading_list_items IS 'Works in a reading list, ordered by position, with optional blurb';
```

The differences from the shelf are the design conversation:

1. **`position INT4 NOT NULL DEFAULT 0`** — lists have an *order*. The
   `idx_reading_list_items_list ON (list_id, position)` index exists
   so "give me this list's works in order" is an index scan, not a
   sort.
2. **`blurb TEXT NOT NULL DEFAULT ''`** — each item can carry a
   one-line *reason* ("read this after the first arc"). A shelf says
   "these are related"; a list says "these are related, in this order,
   and here's why".
3. **`deleted_at TIMESTAMPTZ`** on the list — soft delete, the
   discipline from Chapter 33, instead of the shelf's hard `DELETE`.
4. **`updated_at`** — so a "recently updated lists" sort is possible.
5. **Comments on the tables themselves** — `COMMENT ON TABLE` is
   documentation that lives *in the schema* and shows up in `\d+` and
   any database browser. Cheap, permanent, read by every future
   developer.

The module doc comment in `lists.rs` lays out the whole API in eleven
lines — including the privacy rule that is the list's personality:

```rust
//! Reading lists (bundles): user-curated ordered lists of works with
//! per-item blurbs. Lists are private by default; the owner can flip
//! is_public to share a read-only view.
```

*Private by default.* Shelves default `is_public` to FALSE too — but
lists add the *shared read-only view*: `GET /api/lists/{id}` renders
the list publicly, without any editing power. That's the difference
between "a folder on your disk" and "a link you can send your friend".

The item-append query is the cleverest line in the whole feature, so
let's read it slowly:

```rust
pub async fn add_reading_list_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
    blurb: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
           SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
           FROM reading_list_items WHERE list_id = $1
           ON CONFLICT (list_id, work_id) DO NOTHING"#,
    )
    .bind(list_id)
    .bind(work_id)
    .bind(blurb)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
```

💡 **Key Concept — `INSERT ... SELECT` with an aggregate is "append at
the end, in one statement".** Instead of SELECT the max position, then
INSERT — two round trips with a race window between them — this query
computes `COALESCE(MAX(position), 0) + 1` *inside the INSERT itself*.
Two concurrent appends can't both claim position 5, because the
`SELECT` subquery sees the committed state. And `COALESCE(..., 0)`
handles the empty list (MAX over zero rows is NULL — the same
aggregate-NULL footgun you saw with `SUM` in Chapter 33). The function
returns `bool` — `rows_affected() > 0` — and `ON CONFLICT DO NOTHING`
means a duplicate item *succeeds* as a query but returns false, which
the handler turns into "Work is already in this list". The function's
return type *is* the API contract: `Ok(false)` means "already there",
and the caller decides how to phrase that.

The delete path is symmetric, and notice how the ownership check uses
the *query-layer* scoping trick from 34.1:

```rust
pub async fn delete_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    is_curator: bool,
) -> AppResult<bool> {
    let result = if is_curator {
        sqlx::query("UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(list_id)
            .execute(pool)
            .await?
    } else {
        sqlx::query(
            "UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(list_id)
        .bind(user_id)
        .execute(pool)
        .await?
    };
    Ok(result.rows_affected() > 0)
}
```

One function, two WHERE clauses: curators can delete anyone's list,
owners delete only theirs, and `rows_affected() > 0` distinguishes
"deleted" from "not found or not yours" without a second query. The
handler translates that to a 404 — the same "never reveal which
condition failed" defense you saw with shelves: an attacker probing
list ids can't tell a private list from a nonexistent one.

### 34.3 The two-tier visibility rule, in one handler

The shared-view handler (`get_list_handler`) is where the privacy
model gets real. It must serve three different audiences: the owner
(everything), the public (only `is_public = TRUE`), and anonymous
visitors (public only, of course). Here's how the code shapes that:

```rust
let owner_id = auth.user_id;

// Owner view when authenticated and the list belongs to them, else the
// public view (is_public = TRUE, not deleted).
let list = match owner_id {
    Some(uid) => {
        match queries::get_reading_list(&state.db, list_id, uid, false).await? {
            Some(l) => Some(l),
            None => queries::get_reading_list(&state.db, list_id, uid, true).await?,
        }
    }
    None => queries::get_reading_list(&state.db, list_id, 0, true).await?,
};
```

Notice the fall-through: try the owner's view first, *then* the public
view. That's a deliberate order — a logged-in visitor viewing someone
else's list gets the public query, and the anonymous visitor passes a
fake `0` user id (which can never match, so only the `is_public =
TRUE` clause can succeed). And remember `get_reading_list`'s two
branches from `queries.rs`:

```rust
let row = if public_ok {
    sqlx::query_as::<_, ReadingList>(
        "SELECT ... FROM reading_lists
         WHERE id = $1 AND deleted_at IS NULL AND is_public = TRUE",
    )...
} else {
    sqlx::query_as::<_, ReadingList>(
        "SELECT ... FROM reading_lists
         WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )...
};
```

A single function with a `public_ok: bool` flag producing two WHERE
clauses — the *same* function the update and delete handlers use for
ownership scoping. That's why the flag lives in `queries.rs` and not
in the handlers: every caller gets the visibility rule for free, and
nobody can accidentally write `WHERE id = $1` without the ownership
clause.

The response includes `is_owner`:

```rust
Ok(Json(json!({
    "err": 0,
    "list": { ... },
    "items": item_values,
    "is_owner": owner_id == Some(list.user_id),
})))
```

One boolean computed at the boundary, and the frontend uses it to
decide whether to render an "edit" button or a "copy the link to share"
button. Also note the `items` loop — each item carries
`canonical_title` and `canonical_author` *joined in the query layer*
(see `list_reading_list_items`), so the public JSON never exposes raw
`work_id`s without titles next to them. The API shapes the data for the
UI.

⚠️ **Watch Out — "private by default" is a two-layer promise.** A
`BOOLEAN NOT NULL DEFAULT FALSE` column is layer one: new lists are
private. But the *queries* are layer two — every read path must
enforce it, or the default is decoration. FicHub enforces it three
ways: the shared handler falls through to `public_ok = true` only
after the owner check fails, the query adds `is_public = TRUE` to the
WHERE, and there is *no* endpoint that lists "all public lists" — you
must know the list's id to see it. If you're building a sharing
feature, ask yourself: *can I fetch this resource without proving I
should see it?* If the answer is yes anywhere, the privacy default is
a lie. This is the exact class of bug that later becomes a CVE.

The update handler shows one more pattern worth naming — **PATCH
semantics with optional fields**:

```rust
let title = match &body.title {
    Some(t) => sanitize_title(t)?,
    None => current.title,
};
let description = match &body.description {
    Some(d) => {
        if d.chars().count() > 2000 {
            return Err(AppError::BadRequest(-1, "Description too long (max 2000 chars)".into()));
        }
        d.clone()
    }
    None => current.description,
};
let is_public = body.is_public.unwrap_or(current.is_public);
```

Every field is `Option<T>` in the body struct (`UpdateListBody`), and
`None` means "don't touch". The handler reads the *current* row first,
then overlays the provided fields, then writes the merged result. That
is what makes the endpoint a true PATCH rather than a full PUT — the
client can send `{"is_public": true}` and the title stays. Two
details: `sanitize_title` (trim + empty check + `chars().count() >
200` — the i18n-correct length check that shelves got wrong), and the
fact that the read-then-merge needs the *owner-scoped* get, so only
the owner can PATCH at all.

🧪 **Try It Yourself — build a list, share it, hide it.**

```bash
# 1. Create a private list
curl -s -X POST http://localhost:8080/api/lists \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"title": "Angst with a happy ending",
       "description": "Stories that hurt, then heal"}' | jq .

# 2. Append three works (note: positions auto-increment)
curl -s -X POST http://localhost:8080/api/lists/1/items \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"work_id": 10, "blurb": "The gold standard"}' | jq .
curl -s -X POST http://localhost:8080/api/lists/1/items \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"work_id": 22}' | jq .
curl -s -X POST http://localhost:8080/api/lists/1/items \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"work_id": 10}' | jq .   # → "Work is already in this list"

# 3. Anonymous access fails (private by default)
curl -s http://localhost:8080/api/lists/1 | jq .   # → 404

# 4. Flip it public and try again — the shared view appears
curl -s -X PATCH http://localhost:8080/api/lists/1 \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"is_public": true}' | jq .
curl -s http://localhost:8080/api/lists/1 | jq '.items | length, .is_owner'
```

Watch position numbers climb 1, 2, 3 — and the duplicate attempt
return a friendly message instead of a database error. That's the
`INSERT ... SELECT` + `ON CONFLICT DO NOTHING` combination doing its
job.

---

Two generations of "organize works": the shelf (flat, untyped,
hard-deleted, byte-length validation) and the reading list (ordered,
blurbed, soft-deleted, character-length validation). The newer design
didn't throw the older one away — both ship in the same codebase, and
both are wired into `server.rs`. When you inherit a codebase, this is
normal: you'll read the old way, understand *why* it felt insufficient,
and watch the new way fix exactly those gaps. Next, we leave
user-owned organization behind and build pages for the things the
archive itself knows about: series and authors.


---

## Chapter 35 — Series & Author Pages

So far in this part, the community has been doing the organizing:
readers post requests, readers answer them, readers curate lists. But
FicHub also needs pages for the structures the *archive itself* knows
about — the things that exist whether or not any user ever touches
them. A fanfiction platform without series pages and author pages is a
pile of books with no shelves.

Both live in `src/routes/series.rs`, a 309-line file with two public
handlers and a beautiful pair of unit tests. The module comment tells
you the shape of the whole feature:

```rust
//! Series & author detail pages.
//!
//! - `GET /api/series/{id}` — a series (title, description) with its works
//!   in reading order, each work carrying `next_in_series` (the work that
//!   follows it — the highest-conversion per-fic link in fanfiction).
//! - `GET /api/authors/{name}` — an author bibliography: the canonical
//!   author name, aggregate stats (work count, total words, top tags), and
//!   the works grid (ordered by updated desc).
//!
//! Author pages key off the canonical author name (works.canonical_author),
//! which is how the fic page's author display already identifies authors.
```

Two endpoints, both public, both read-only. This is the rare chapter
where there's not a single `INSERT` — and that's exactly what makes it
a good lesson in *read-path* design: the queries here do more work than
any insert you've seen.

### 35.1 The series: works in reading order, with the "next" baked in

The schema (migration `020_series_authors.sql`) is tiny — a container
and a membership join, like shelves, but with a crucial difference:

```sql
CREATE TABLE IF NOT EXISTS series (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS series_works (
    series_id INT4 NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    work_id INT4 NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    position INT4 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (series_id, work_id)
);

CREATE INDEX IF NOT EXISTS idx_series_works_work ON series_works(work_id);
CREATE INDEX IF NOT EXISTS idx_series_works_position ON series_works(series_id, position);
```

A series is *ordered* — `position` again, like reading lists — and
unlike shelves there's no `user_id` at all: series belong to the
archive, not to users. The index on `(series_id, position)` exists so
the "works in reading order" query is an index scan.

The handler starts by fetching the series row itself, and note the
error handling:

```rust
pub async fn get_series(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let series = sqlx::query_as::<_, (i32, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, name, description, created_at, updated_at FROM series WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Series not found".into()))?;
```

`fetch_optional` returns `Option`, and `.ok_or_else(...)` turns `None`
into an `AppError::NotFound` — the `?` then unwraps the `Some`. A
missing series is a 404, not a crash. You'll see this exact
`fetch_optional` + `ok_or_else` + `?` idiom in every single-entity
handler in FicHub; it's the read-side cousin of the ownership checks
you saw in Chapter 34.

Then the real work: the works of the series, in reading order, each
with a *linkable source*. The query needs to handle a subtlety of the
data model — a `work` is a canonical story, but it may exist on
multiple source sites (`fic_info` rows). Which one does the card link
to? The code answers: the work's `default_source_id` if it has one,
falling back to its highest-word-count source:

```rust
let rows = sqlx::query_as::<_, (i32, String, String, Option<String>, Option<String>)>(
    r#"SELECT w.id, w.canonical_title, w.canonical_author,
               w.default_source_id,
               (SELECT fi.id::text FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1)
       FROM series_works sw
       JOIN works w ON w.id = sw.work_id
       WHERE sw.series_id = $1
       ORDER BY sw.position ASC, w.id ASC"#,
)
.bind(id)
.fetch_all(&state.db)
.await?;
```

The correlated subquery `(SELECT fi.id::text FROM fic_info fi WHERE
fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1)` is the "best source"
lookup — the richest copy of the story (most words) wins when the
default is missing. This "default, else best-available" fallback
pattern is one you've seen before: it's the same idea as the reading
list items joining canonical titles, and the same "a URL that always
links somewhere is a feature" philosophy.

Then the enrichment loop — for each work, load the source metadata and
build the JSON card:

```rust
for (work_id, canonical_title, canonical_author, default_url, fallback_url) in rows {
    let url_id = default_url
        .filter(|u| !u.is_empty())
        .or(fallback_url)
        .unwrap_or_default();

    // Source metadata (best effort — missing when the work has no sources).
    let src = sqlx::query_as::<_, (String, String, i64, i32, String)>(
        "SELECT title, author, words, chapters, status FROM fic_info WHERE id = $1",
    )
    .bind(&url_id)
    .fetch_optional(&state.db)
    .await?
    .map(|(title, author, words, chapters, status)| {
        json!({
            "title": title,
            "author": author,
            "words": words,
            "chapters": chapters,
            "status": status,
        })
    })
    .unwrap_or_else(|| {
        json!({
            "title": canonical_title,
            "author": canonical_author,
            "words": 0,
            "chapters": 0,
            "status": "unknown",
        })
    });
```

Note the graceful fallback again: if the source lookup fails (a work
with no sources at all — theoretically impossible but admitted by the
schema), the card still renders with canonical values and zeroes, and
`status` becomes `"unknown"` rather than crashing the whole page. The
handler *never* lets one bad row kill the page.

And now the headline feature, computed after all cards are built:

```rust
// Compute next_in_series for each position (0-based → the following work).
for i in 0..works.len() {
    if i + 1 < works.len() {
        let nxt = &works[i + 1];
        works[i]["next_in_series"] = json!({
            "work_id": nxt["work_id"],
            "canonical_title": nxt["canonical_title"],
            "url_id": nxt["url_id"],
        });
    } else {
        works[i]["next_in_series"] = Value::Null;
    }
}
```

Every work except the last gets a `next_in_series` object pointing at
the following work; the last gets `null`. The module comment calls this
"the highest-conversion per-fic link in fanfiction" — and it is. Think
about the reading flow: you finish book one of a trilogy, and the
series page — and the reader, via `next_in_series` — hands you book
two before you can stop. The frontend highlights that card; the data
model makes the transition one JSON field away.

💡 **Key Concept — derivable data is computed at the edge, not stored
in the middle.** `next_in_series` is *derivable*: given the ordered
works, it's "the next index". FicHub could have stored a
`next_work_id` column on `series_works` — and suffered every update
ordering bug that column would create. Instead, the handler computes
it from `position` order in a 10-line loop, *at the response edge*,
and the JSON the frontend gets is exactly what it needs. Whenever you
catch yourself about to store something that can be computed from data
you already have, stop and ask: *will this get out of sync?* If yes,
compute it.

Now look at how the file *tests* that loop — this is a lovely piece of
engineering:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_author_name_is_rejected() {
        // Route-level validation lives in the handler; the guard is trivially
        // testable here.
        let name = "   ".trim();
        assert!(name.is_empty());
    }

    #[test]
    fn next_in_series_computation_links_consecutive_works() {
        // Pure mirror of the handler's next_in_series loop: with N works the
        // i-th work's next is i+1 and the last has none.
        let works = vec![
            json!({"work_id": 1, "canonical_title": "A", "url_id": "a"}),
            json!({"work_id": 2, "canonical_title": "B", "url_id": "b"}),
            json!({"work_id": 3, "canonical_title": "C", "url_id": "c"}),
        ];
        let mut out = works.clone();
        for i in 0..out.len() {
            if i + 1 < out.len() {
                out[i]["next_in_series"] = json!({
                    "work_id": out[i + 1]["work_id"],
                    "canonical_title": out[i + 1]["canonical_title"],
                    "url_id": out[i + 1]["url_id"],
                });
            } else {
                out[i]["next_in_series"] = Value::Null;
            }
        }
        assert_eq!(out[0]["next_in_series"]["work_id"], 2);
        assert_eq!(out[1]["next_in_series"]["work_id"], 3);
        assert!(out[2]["next_in_series"].is_null());
    }
}
```

The test *re-implements the loop* on plain JSON — no database, no HTTP
— and asserts the contract: first links to second, second to third,
last is null. This is the "pure function test" pattern: extract the
logic you can test without infrastructure, mirror it in the test, and
pin the behavior. It doesn't need a database because the loop doesn't
touch one — the *exact* logic worth testing. (Compare the roadmap
tests in Chapter 37, which do the same thing for Elo math.)

⚠️ **Watch Out — the tests in `series.rs` test the *mirror*, not the
original.** The `next_in_series_computation_links_consecutive_works`
test duplicates the loop instead of calling it, so if someone edits
the handler loop and forgets the test, the test keeps passing while
the behavior drifts. This is a real, acknowledged trade-off: the
handler's loop is embedded in a function that needs `AppState`, so
calling it in a unit test means a big refactor. The mirror test still
has value — it documents the intended behavior and catches *accidental*
breakage — but it can't catch deliberate drift. When you see a mirror
test in someone's codebase, the honest question to ask is whether the
logic is worth extracting into a pure function so the test can call
the real thing. Here, the loop is four lines — the mirror is fine.

### 35.2 The author page: canonical names, orphans, and the "most common spelling" trick

The second handler, `get_author`, is registered at a different path
than the profile endpoints you'll see in `authors.rs`:

```rust
.route("/api/authors/search", get(crate::routes::authors::search_authors))
.route("/api/authors/by-name/{name}", get(crate::routes::series::get_author))
.route("/api/authors/{id}", get(crate::routes::authors::get_author_profile))
```

Three author routes, deliberately split. `/api/authors/{id}` is the
curator-managed *profile* (bio, socials, linked source accounts — the
stuff in `authors.rs`, which we'll read in 35.4). `/api/authors/by-name/{name}`
is the *bibliography* — the page that lists everything this author has
written. And `/api/authors/search` is the lookup. The `series.rs`
module comment explains the keying decision: author pages key off the
canonical author *name*, because that's how the fic page's author
display already identifies authors.

The handler starts by resolving the *canonical* spelling of the name —
because fanfiction authors are chaos. The same person writes as
"Ms_Figg", "msfigg", and "Ms. Figg" depending on the site and the
year. The query picks the most common spelling as canonical:

```rust
// Canonical author = the most common works.canonical_author spelling
// (works are the unified story entries; the author display on fic pages
// uses works.canonical_author when available).
let canonical: Option<String> = sqlx::query_scalar(
    r#"SELECT canonical_author FROM works
       WHERE LOWER(canonical_author) = LOWER($1) AND canonical_author <> ''
       GROUP BY canonical_author ORDER BY COUNT(*) DESC, canonical_author LIMIT 1"#,
)
.bind(&name)
.fetch_optional(&state.db)
.await?;

let canonical_name = canonical.unwrap_or_else(|| name.to_string());
```

Three craft points in one query:

- **`LOWER(...) = LOWER($1)`** — case-insensitive matching, so
  "msfigg" finds "MsFigg". (We met the *operator* `ILIKE` in Part 7;
  this is the *function* form, used when you want an exact-match after
  folding case rather than a pattern match.)
- **`GROUP BY canonical_author ORDER BY COUNT(*) DESC`** — the "most
  common spelling wins" logic: group all spellings, count the works
  under each, take the top.
- **`unwrap_or_else(|| name.to_string())`** — if the author has *no*
  works at all, the page still renders, using the name you typed. Never
  crash on an empty bibliography.

Then the works query — and here's the interesting part. It selects
*both* the canonical work rows *and* a second set the code calls
"orphans": `fic_info` rows by this author that aren't linked to any
canonical `work_id` yet:

```rust
// Also surface fic_info rows by this author that aren't yet linked to a
// canonical work (orphan sources) so the bibliography is complete.
let orphans: Vec<Value> = sqlx::query_as::<_, (String, String, String, i64, i32, String, Option<i32>)>(
    r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, fi.status, fi.work_id
       FROM fic_info fi
       WHERE LOWER(fi.author) = LOWER($1)
         AND fi.work_id IS NULL
       ORDER BY fi.fic_updated DESC
       LIMIT 50"#,
)
.bind(&name)
.fetch_all(&state.db)
.await?;
```

💡 **Key Concept — orphaned data is a fact of life; the page admits
it.** The scraper creates `fic_info` rows the moment it meets a story
— and the *canonicalization* step (linking `fic_info.work_id` to a
`works` row) may lag behind, or a story may only exist on one source.
Rows with `work_id IS NULL` are *orphans*: real stories, complete with
titles and word counts, but not yet unified into a canonical work. A
naive author page would either miss them (incomplete bibliography) or
crash trying to join them. FicHub surfaces them in a separate `orphans`
array with a `LIMIT 50` guard — the page shows everything it knows,
and marks what's canonical. Data pipelines are never perfectly clean;
good read paths handle the residue gracefully.

The aggregate stats use the same `COALESCE` discipline you saw with
votes:

```rust
let stats = sqlx::query_as::<_, (i64, Option<i64>)>(
    r#"SELECT
        COUNT(*)::bigint,
        SUM(fi.words)::bigint
      FROM fic_info fi
      WHERE LOWER(fi.author) = LOWER($1)"#,
)
.bind(&name)
.fetch_one(&state.db)
.await?;
```

`SUM` over zero rows is NULL (that footgun again) — so the response
does `total_words: stats.1.unwrap_or(0)`. And the "top tags" query
shows off a three-table join with a *ranked* aggregate:

```rust
let top_tags: Vec<(String, i16, i64)> = sqlx::query_as(
    r#"SELECT t.name, t.tag_type_id, COUNT(DISTINCT ft.url_id)::bigint AS usage_count
       FROM tags t
       JOIN fic_tags ft ON ft.tag_id = t.id
       JOIN fic_info fi ON fi.id = ft.url_id
       WHERE LOWER(fi.author) = LOWER($1)
         AND t.tag_type_id IN (1, 4)
       GROUP BY t.id
       ORDER BY usage_count DESC, t.name
       LIMIT 20"#,
)
.bind(&name)
.fetch_all(&state.db)
.await?;
```

Tags → fic_tags → fic_info: the tag vocabulary, the per-fic tag
assignments, the fics themselves. `COUNT(DISTINCT ft.url_id)` counts
how many *fics* use each tag (not how many tag rows exist — a fic could
theoretically have the same tag twice), restricted to tag types 1
(fandom) and 4 (freeform) — the two types a reader actually cares about
on an author page. Ordered by usage, then alphabetically as a
deterministic tiebreaker.

The whole response is assembled with the three parts — author stats,
canonical works, orphans:

```rust
Ok(Json(json!({
    "err": 0,
    "author": {
        "name": canonical_name,
        "work_count": stats.0,
        "total_words": stats.1.unwrap_or(0),
        "top_tags": top_tags.into_iter().map(|(tag_name, tag_type, count)| json!({
            "name": tag_name,
            "tag_type_id": tag_type,
            "usage_count": count,
        })).collect::<Vec<_>>(),
    },
    "works": works,
    "orphans": orphans,
})))
```

One page, three data shapes: the *canonical* works (the unified story
entries, each with its best linkable source), the *orphans* (everything
else the archive knows about), and the *aggregates* (stats + top tags
computed fresh on every request).

### 35.3 Wait — why is the author page in `series.rs` and not `authors.rs`?

Good question, and the answer is worth a paragraph because it teaches
you how to *read* a codebase: `authors.rs` is the *curator-facing*
module — profiles, socials, merge proposals, the workflow where humans
decide that "MsFigg" and "msfigg" are the same person. `series.rs`
holds the *reader-facing* endpoints for both series and the author
bibliography, because they're both "public read-only detail pages
assembled from the works tables". The file layout is a *design
statement*: public pages in one place, curation workflows in another,
even when both concern "authors". Don't fight this when you read a
codebase — note it, and let it tell you where new code should go.

### 35.4 The curator side: `authors.rs` and the merge workflow

`src/routes/authors.rs` (469 lines) is the other half of the author
story. Where `series.rs` is read-only, `authors.rs` is where author
identity gets *curated* — and it has a signature pattern: nearly every
handler starts with a role gate.

```rust
pub async fn update_author_profile(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
```

`role < 5` → 403. (The role tiers — 0 reader, 1 trusted, 5 curator, 10
admin — came from Part 7, Chapter 29.) This gate is *the* curator
pattern: it appears at the top of update, socials-add, socials-remove,
and every merge handler.

The interesting workflow here is the **merge proposal** — the
mechanism that eventually cleans up the "msfigg / MsFigg" chaos the
author page works around. Look at how `propose_merge` splits by
caller role:

```rust
let auto_approve = payload
    .get("auto_approve")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

if auto_approve && user.role >= 10 {
    // Admin auto-approve: create link directly
    sqlx::query(
        "INSERT INTO author_profile_links (profile_id, source_author, source_url, source_id) VALUES ($1, $2, $3, $4) ON CONFLICT (source_url) DO NOTHING",
    )
    .bind(target_profile_id)
    .bind(source_author)
    .bind(source_url)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "Author linked directly"})))
} else {
    sqlx::query(
        "INSERT INTO author_merge_proposals (source_author, source_url, target_profile_id, proposed_by) VALUES ($1, $2, $3, $4)",
    )
    .bind(source_author)
    .bind(source_url)
    .bind(target_profile_id)
    .bind(user.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "Merge proposal submitted"})))
}
```

⚠️ **Watch Out — authorization is a spectrum, not a boolean.** Most
endpoints are "allowed or not". Merges are three-tiered: *admins* can
auto-approve (role ≥ 10), *curators* can propose (role ≥ 5), and
everyone else is forbidden entirely. The condition `auto_approve &&
user.role >= 10` is a *compound* authorization decision — the caller
gets to *ask* for instant approval, but only an admin's request can be
honored. When you read an authorization check, don't just look for the
role threshold — look for how the *request itself* can modulate the
decision. And note the asymmetry: a curator proposing a merge creates
a row in `author_merge_proposals` with `status = 'pending'`; an admin
skips the queue entirely. The pending list is what `pending_merges`
serves:

```rust
let rows = sqlx::query_as::<_, (i32, String, String, i32, String, Option<String>, i32, String)>(
    r#"SELECT amp.id, amp.source_author, amp.source_url, amp.target_profile_id,
              ap.canonical_name, u.username, amp.proposed_by, amp.created_at::text
       FROM author_merge_proposals amp
       JOIN author_profiles ap ON ap.id = amp.target_profile_id
       LEFT JOIN users u ON u.id = amp.proposed_by
       WHERE amp.status = 'pending'
       ORDER BY amp.created_at DESC"#,
)
.fetch_all(&state.db)
.await?;
```

`LEFT JOIN users` — because `proposed_by` can be NULL (a system
process can propose merges too), and a plain JOIN would silently drop
those rows. And then the approval handler, which *does* the link
creation and the status flip in two statements:

```rust
let proposal = sqlx::query_as::<_, (String, String, i32)>(
    "SELECT source_author, source_url, target_profile_id FROM author_merge_proposals WHERE id = $1 AND status = 'pending'",
)
.bind(proposal_id)
.fetch_optional(&state.db)
.await?
.ok_or_else(|| AppError::NotFound("Proposal not found or already resolved".into()))?;

// Create the link
sqlx::query(
    "INSERT INTO author_profile_links (profile_id, source_author, source_url) VALUES ($1, $2, $3) ON CONFLICT (source_url) DO NOTHING",
)
.bind(proposal.2)
.bind(&proposal.0)
.bind(&proposal.1)
.execute(&state.db)
.await?;

// Update proposal status
sqlx::query(
    "UPDATE author_merge_proposals SET status = 'approved', approved_by = $1, resolved_at = NOW() WHERE id = $2",
)
.bind(user.user_id)
.bind(proposal_id)
.execute(&state.db)
.await?;
```

The `WHERE id = $1 AND status = 'pending'` on the read is a *guard*:
an already-resolved proposal can't be approved twice, and the error
message covers both cases ("not found or already resolved") without
leaking which one. The status flip sets `approved_by` and `resolved_at`
— the audit trail of who did what, ready for the modlog in Part 11.

The file also shows the *handler-level tests* pattern: `authors.rs`
ends with a battery of tests that need no database at all —
`test_forbidden_role_check` constructs `AuthUser` structs by hand and
asserts the role comparisons:

```rust
#[test]
fn test_forbidden_role_check() {
    // Users with role < 5 should get Forbidden
    let user = AuthUser {
        user_id: Some(1),
        username: Some("reader".into()),
        role: 0,
    };
    assert!(user.role < 5, "Reader should not pass curator check");

    let curator = AuthUser {
        user_id: Some(2),
        username: Some("curator".into()),
        role: 5,
    };
    assert!(curator.role >= 5, "Curator should pass curator check");

    let admin = AuthUser {
        user_id: Some(3),
        username: Some("admin".into()),
        role: 10,
    };
    assert!(admin.role >= 10, "Admin should pass admin check");
}
```

This looks almost silly — "the test proves that 5 ≥ 5" — and that's
exactly the point. The *policy* (who may curate) is pinned here as a
test, so a future refactor of the role system can't silently change
the boundary. Cheap tests that pin constants and comparisons are
valuable precisely because they're trivial.

🧪 **Try It Yourself — browse the public pages.**

```bash
# 1. A series page (works in reading order with next_in_series)
curl -s http://localhost:8080/api/series/1 | jq .
#    → look at works[0].next_in_series.work_id — the next in order

# 2. Walk the chain with jq
curl -s http://localhost:8080/api/series/1 \
  | jq -r '.works[] | "\(.canonical_title) → next: \(.next_in_series.canonical_title // "END")"'

# 3. An author bibliography, canonical name resolved for you
curl -s "http://localhost:8080/api/authors/by-name/msfigg" | jq .
#    → note "name" may come back as the canonical spelling, works + orphans arrays

# 4. Top tags per author
curl -s "http://localhost:8080/api/authors/by-name/msfigg" \
  | jq -r '.author.top_tags[] | "\(.name) (\(.usage_count) fics)"'

# 5. Curator side: role-gated (expect 403 without curator token)
curl -s -X POST http://localhost:8080/api/curator/authors/merge \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"source_author": "msfigg", "source_url": "https://archiveofourown.org/users/msfigg",
       "target_profile_id": 1}' | jq .
```

If your `$TOKEN` belongs to a plain reader, the merge call returns the
403 with "Curator access required" — the role gate working exactly as
the test in `authors.rs` promises.

---

Series pages and author pages are the read-side of the archive: no
writes, but the hardest queries in this part. You've now seen the
`fetch_optional` + `ok_or_else` idiom, the "default source, else best
source" fallback, the orphan-aware author query, and a role-gated
curation workflow with a pending-approval lifecycle. Next, we hand the
whole archive to the open internet — the feeds that let anyone
subscribe.


---

## Chapter 36 — RSS/Atom Feeds

Here's a truth about fanfiction readers: they don't check the archive
every day — they wait for the *notification*. Part 7 built the
in-app notification inbox, but that only works when you're logged into
FicHub. What about the rest of the open internet? The answer is the
humble, ancient, still-irreplaceable RSS/Atom feed — a URL that any
feed reader on the planet can poll, and that FicHub serves as a plain
XML document.

FicHub ships **three feeds**, all Atom 1.0, all in `src/routes/rss/`
— a module split into `mod.rs` (the contract), `build.rs` (pure XML
construction, 394 lines), and `handlers.rs` (the HTTP layer, 126
lines). The module doc comment in `mod.rs` is the whole spec:

```rust
//! RSS/Atom update feeds (program item: RSS feeds).
//!
//! Three feeds, all Atom 1.0 (`application/atom+xml; charset=utf-8`):
//!
//! * `GET /feed.xml` — New arrivals: the 20 most recently created works
//!   (ordered by `fic_info.created` DESC, falling back to `fic_updated`
//!   when created is NULL). Public.
//! * `GET /feed/follows.xml?token=<jwt>` — Updates for the works the
//!   authenticated user follows, ordered by `fic_updated` DESC. The token
//!   is a regular FicHub JWT (the same one the frontend keeps in
//!   `localStorage['fichub_token']` and sends as `Authorization: Bearer ...`);
//!   it may come from the `token` query param (feed readers can't send
//!   headers) or the Authorization header.
//! * `GET /feed/works/<url_id>.xml` — Update feed for a single fic,
//!   containing just that fic's entry (its `fic_updated` timestamp).
//!
//! Every entry carries an `<updated>` timestamp, an `<id>`, a
//! `<link rel="alternate">` back to the fic page, and per-format download
//! links. All text is XML-escaped.
```

And the two constants that govern everything:

```rust
/// Atom feed content type (RFC 4287).
pub const ATOM_CONTENT_TYPE: &str = "application/atom+xml; charset=utf-8";

/// Maximum entries in a feed.
pub const MAX_ENTRIES: i64 = 20;
```

The first thing you should notice is the module split. `build.rs` is
*XML with no database*; `handlers.rs` is *database with no XML*. That
separation exists for one reason, and the file comment says it
plainly: *"Kept separate from `handlers` so the XML-escaping /
feed-shape helpers are unit-testable without a database."* We're going
to see the payoff in the test section — a whole battery of `#[test]`
functions that need no Postgres at all.

### 36.1 The pure XML layer: escaping is security

Every feed is, at bottom, user-controlled strings (titles, authors,
descriptions) pasted into an XML document. And XML has the same
injection problem as SQL and HTML: a title containing `<script>` is
not a title, it's a *parser directive*. The defense is the tiny
function at the top of `build.rs`:

```rust
/// Escape a string for use inside XML text/attribute content.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

Five replacements. `&` first — always first, or you'd double-escape
the entities you just created. (If `&` were replaced last, an input of
`&lt;` would become `&amp;lt;` and the feed would show the literal
text `&lt;` instead of `<`.) Ordering matters, and there's a test for
it.

💡 **Key Concept — string interpolation into a structured format is an
injection surface.** `format!` is not a template engine — it pastes
text with no awareness of the surrounding syntax. When you build XML,
HTML, or SQL with `format!`, every interpolated value must be escaped
for *that* format. This is the same class of bug as SQL injection:
trusting the boundary. FicHub's `html_escape` is its XSS defense for
feeds — and its tests are explicit about the threat model:

```rust
#[test]
fn entry_escapes_specials_never_breaks_xml() {
    let e = entry_xml(
        "urn:id",
        "/fic/id",
        "<script>alert('x')</script> & \"",
        "A & B",
        "s <b>html</b> & co",
        "2024-01-01T00:00:00Z",
        1,
        1,
        "ongoing",
    );
    assert!(!e.contains("<script>"));
    assert!(e.contains("&lt;script&gt;"));
    assert!(e.contains("&amp; &quot;"));
}
```

The test *feeds the attack string in* and asserts the attack doesn't
survive. When you write a test that includes an XSS payload, you're
not being paranoid — you're pinning the security property so a future
refactor can't quietly drop the escaping.

The entry builder is where escaping and shape come together:

```rust
pub fn entry_xml(
    entry_id: &str,
    page_path: &str,
    title: &str,
    author: &str,
    summary: &str,
    updated: &str,
    words: i64,
    chapters: i32,
    status: &str,
) -> String {
    format!(
        r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>{id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <category term="{status}" label="{status}"/>
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <category term="words:{words}" label="{words} words"/>
    <link rel="alternate" href="{page}" type="text/html"/>
    <link rel="self" href="{page}.xml" type="{ct}"/>
    <link rel="enclosure" href="/cache/epub/{url}/{url}.epub" type="application/epub+zip" length="0"/>
    <link rel="enclosure" href="/cache/pdf/{url}/{url}.pdf" type="application/pdf" length="0"/>
    <link rel="enclosure" href="/cache/html/{url}/{url}.html" type="text/html" length="0"/>
  </entry>
"#,
        title = html_escape(title),
        author = html_escape(author),
        id = html_escape(entry_id),
        updated = updated,
        summary = html_escape(summary),
        status = html_escape(status),
        chapters = chapters,
        words = words,
        page = html_escape(page_path),
        ct = ATOM_CONTENT_TYPE,
        url = html_escape(url_of(page_path)),
    )
}
```

Look at what an entry *carries* — this is the Atom spec being used
thoughtfully. `<id>` is a stable URN (`urn:fichub:fic:...`) so feed
readers can dedupe across refreshes. `<updated>` drives "what's new".
The `<link rel="alternate">` goes back to the fic page; `<link
rel="self">` is the feed entry's own URL; and the three
`rel="enclosure"` links point at the **EPUB, PDF, and HTML downloads** —
meaning a feed reader like a podcatcher can fetch the whole book
directly from the feed. FicHub's export pipeline (Part 5) becomes a
subscription product with zero extra code.

The full document builder wraps entries in the Atom envelope:

```rust
pub fn atom_feed(
    feed_id: &str,
    title: &str,
    subtitle: &str,
    self_path: &str,
    updated: &str,
    entries: &str,
    base_url: Option<&str>,
) -> String {
    let self_abs = abs_url(self_path, base_url);
    let mut links = String::new();
    links.push_str(&format!(
        r#"  <link href="{self}" rel="self" type="{ct}"/>
"#,
        self = html_escape(&self_abs),
        ct = ATOM_CONTENT_TYPE,
    ));
    links.push_str(&format!(
        r#"  <link href="{alt}" rel="alternate" type="text/html"/>
"#,
        alt = html_escape(&abs_url("/", base_url)),
    ));

    let base_attr = match base_url {
        Some(b) if !b.is_empty() => format!(" xml:base=\"{}\"", html_escape(b)),
        _ => String::new(),
    };

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"{base}>
  <id>{id}</id>
  <title>{title}</title>
  <subtitle>{subtitle}</subtitle>
  <updated>{updated}</updated>
  <author><name>FicHub</name></author>
{links}{entries}</feed>
"#,
        base = base_attr,
        id = html_escape(feed_id),
        title = html_escape(title),
        subtitle = html_escape(subtitle),
        updated = updated,
        links = links,
        entries = entries,
    )
}
```

And note `xml:base` — an Atom feature that makes *every relative URL in
the feed* resolve against the configured base URL, so the same feed
works in dev (`localhost`) and in production. The base URL comes from
`state.config.opds_base_url` — the same config knob the OPDS feeds
(Part 5) use, so a deployer sets one value and all syndication
follows.

⚠️ **Watch Out — XML is not HTML; escaping is only half the battle.**
RSS/Atom feeds get parsed by *strict* XML parsers. An unescaped `&` in
a title is a hard feed-breaking error in most readers — and a feed
with broken XML is worse than no feed, because readers cache the
failure. Also note what the tests check beyond escaping: `rel="self"`
presence, the `xml:base` absolutization, and the *enclosure* links. A
feed that validates but lacks the right `<link>` elements is a feed
that works in no reader's UI. The tests in `build.rs` read like a
checklist for the Atom spec — that's intentional.

The response helper is one line, but it's the whole HTTP contract:

```rust
pub fn atom_response(body: String) -> ([(header::HeaderName, &'static str); 1], String) {
    ([(header::CONTENT_TYPE, ATOM_CONTENT_TYPE)], body)
}
```

`application/atom+xml; charset=utf-8` — without this Content-Type,
feed readers and browsers would treat the XML as generic text. Small
header, big compatibility difference.

### 36.2 The three handlers: public, authed, and per-fic

The handlers in `handlers.rs` are thin on purpose — fetch rows, build
entries, wrap the document, return. The new-arrivals feed is the
simplest:

```rust
/// GET /feed.xml — new arrivals (most recently created fics).
pub async fn new_arrivals_feed(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.effective_limit();
    let rows = fetch_new_arrivals(&state, limit).await?;
    let now = iso_now();

    let entries: String = rows.iter().map(entry_for_fic).collect();
    let body = atom_feed(
        "urn:fichub:feed:new",
        "FicHub — New Arrivals",
        "Recently added fanfiction",
        "/feed.xml",
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );
    Ok(atom_response(body))
}
```

Six lines of logic. `rows.iter().map(entry_for_fic).collect()` — one
`FeedFic` row becomes one Atom entry, and the entries string is just
concatenated XML. The feed id (`urn:fichub:feed:new`) is a *stable
URN*: it must never change, because feed readers use `<id>` to track
what they've already seen.

The limit handling lives in `build.rs` as a small method with a test
you should steal:

```rust
impl FeedQuery {
    pub fn effective_limit(&self) -> i64 {
        self.limit
            .or(self.per_page)
            .unwrap_or(MAX_ENTRIES)
            .clamp(1, MAX_ENTRIES)
    }
}
```

`limit` or `per_page` (two names for the same knob, accepted for
compatibility), default `MAX_ENTRIES` (20), clamped to at most 20.
No matter what a caller sends — `limit=1000`, `limit=-5`, garbage —
the feed returns between 1 and 20 entries. **Clamp user-controlled
numbers at the boundary**; an attacker asking for 1000 entries is
asking you to do 50x the work for free.

The follows feed is the interesting one, because of *how* it
authenticates. Feed readers (the apps and websites that poll feeds)
generally cannot send `Authorization` headers — they just GET a URL.
So the JWT has to travel in the query string:

```rust
/// Resolve the authenticated user id from a feed request.
///
/// Feed readers can't send `Authorization` headers, so the JWT may arrive
/// as a `token` query param (same token the web frontend stores in
/// `localStorage['fichub_token']`). Falls back to the Bearer header.
fn authed_user_id(headers: &HeaderMap, token: Option<&str>) -> Option<i32> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    if let Some(t) = token {
        if let Ok(claims) = crate::routes::auth::verify_token(t, &secret) {
            return Some(claims.sub);
        }
    }
    if let Some(header_val) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(val) = header_val.to_str() {
            if let Some(t) = val.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::routes::auth::verify_token(t, &secret) {
                    return Some(claims.sub);
                }
            }
        }
    }
    None
}
```

Two auth paths, same `verify_token` from Part 7's Chapter 28: the
query-param token first, then the classic `Authorization: Bearer`
header as a fallback. The `if let Ok(claims) = ...` pattern means a
garbage token just skips to the next path — and if both fail, the
handler rejects:

```rust
let user_id = authed_user_id(&headers, query.token.as_deref())
    .ok_or_else(|| AppError::BadRequest(401, "Feed token required (login)".into()))?;
```

⚠️ **Watch Out — tokens in URLs leak.** The `token` query param is a
compromise: feed readers need it, but URLs end up in browser history,
proxy logs, and `Referer` headers. FicHub accepts this because the
follows feed is *read-only metadata* (no downloads without a second
check — note the handler only lists fic metadata, not full EPUBs), and
because a leaked 30-day JWT is far less dangerous than leaked
credentials. Still, this is the kind of trade-off you should *notice*
when you read code: the comment says "feed readers can't send
headers", and the design accepts the leak for compatibility. When you
build an authenticated-by-URL endpoint, keep it read-only, keep it
scoped, and keep the token short-lived.

The follows feed query itself is a great example of *polymorphic
follows* — the `follows` table from Part 7 can track either a work or
an author, and the feed must respect both:

```rust
pub async fn fetch_followed_updates(
    state: &Arc<AppState>,
    user_id: i32,
    limit: i64,
) -> AppResult<Vec<FeedFic>> {
    let rows = sqlx::query_as::<_, FeedFic>(
        r#"SELECT fi.id, fi.title, fi.author, fi.description,
                  fi.fic_updated, fi.words, fi.chapters, fi.status,
                  fi.work_id, fi.created
           FROM follows f
           JOIN fic_info fi
             ON (f.work_id IS NOT NULL AND fi.work_id = f.work_id)
             OR (f.work_id IS NULL AND f.author_name IS NOT NULL
                 AND fi.author = f.author_name)
           WHERE f.follower_id = $1
           ORDER BY fi.fic_updated DESC
           LIMIT $2"#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}
```

💡 **Key Concept — a polymorphic JOIN is an OR of two shapes.** The
follow row is either "I follow work 42" (`work_id` set) or "I follow
the author 'MsFigg'" (`author_name` set). The JOIN handles both with
an `OR` of two conditions: match on work_id when it's a work follow,
match on author name when it's an author follow. Because the two
conditions are mutually exclusive by design (one side is NULL when the
other is set), the OR is unambiguous. This is how you JOIN against a
polymorphic key without redesigning the schema — and it's the same
trick the OPDS feeds and notifications use.

The third feed is the per-fic one — and it contains a subtle axum
lesson:

```rust
/// GET /feed/works/<url_id>.xml — updates for a single fic.
///
/// The route is registered as `/feed/works/{url_id}` (axum 0.8 forbids
/// mixed literal+param segments); a trailing `.xml` is stripped here so the
/// canonical `/feed/works/<url_id>.xml` URL works.
pub async fn work_feed(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let url_id = url_id.strip_suffix(".xml").unwrap_or(&url_id).to_string();
    let row = fetch_fic(&state, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {url_id}")))?;
```

The route is declared as `/feed/works/{url_id}` because axum 0.8
refuses a literal `.xml` suffix inside a route pattern; the handler
strips the suffix from the captured value. So the URL that *users*
bookmark — `/feed/works/abc123.xml` — works, and the routing system
never has to deal with mixed segments. It's a small compatibility
shim, documented in a comment precisely because it's non-obvious. When
you see a comment explaining *why the code is shaped this way*, that's
a senior developer's fingerprint.

The whole feed module ends with a test battery that needs no database:
`html_escape` round trips, the Atom envelope's required elements, the
`xml:base` absolutization, the escaping of hostile titles, and the
limit clamping. Every one of those is a property that would be painful
to test through HTTP and trivially cheap to test pure. That's why
`build.rs` exists as a separate module — the testability drove the
architecture.

🧪 **Try It Yourself — subscribe to FicHub.**

```bash
# 1. The public new-arrivals feed
curl -s http://localhost:8080/feed.xml | head -40
#    → the Atom envelope: <feed>, <id>, <title>, then <entry> blocks

# 2. Count entries and confirm the content type
curl -s http://localhost:8080/feed.xml | grep -c "<entry>"
curl -sI http://localhost:8080/feed.xml | grep -i content-type
#    → application/atom+xml; charset=utf-8

# 3. Try to abuse the limit (it clamps to 20)
curl -s "http://localhost:8080/feed.xml?limit=1000" | grep -c "<entry>"

# 4. A single fic's update feed
curl -s http://localhost:8080/feed/works/abc123.xml | head -30

# 5. The follows feed — without a token it's a 401
curl -s http://localhost:8080/feed/follows.xml | head -5

# 6. With a token (query-param auth, the feed-reader way)
curl -s "http://localhost:8080/feed/follows.xml?token=$TOKEN" | grep -c "<entry>"
```

Then paste `http://localhost:8080/feed.xml` into any feed reader
(Thunderbird, Feedly, or `newsboat` if you're feeling terminal-y) and
watch the archive come to you. And if your reader supports
enclosures, the EPUB download links are sitting right there in every
entry.

---

Three feeds, one XML builder, one escaping function — that's the whole
syndication layer. The module split (`build.rs` pure + `handlers.rs`
thin) is the pattern to copy: when you build any output format,
separate the *pure rendering* from the *database plumbing* so the
rendering can be tested without infrastructure. The feeds also closed
a loop: the follows table from Part 7 now powers an *external*
notification channel, and the export pipeline from Part 5 now powers
feed enclosures. Community features compose. Next, the last and most
ambitious feature of the part — the one where the community decides
what gets built next.


---

## Chapter 37 — The Roadmap Consensus Arena

Every community eventually asks the same dangerous question: *"What
should we build next?"* Dangerous, because the answers arrive as a
wall of overlapping, contradictory, unrankable suggestions. "Dark mode"
and "change the font to black" are the same idea from two people.
FicHub's answer to this chaos is the most interesting code in this
part: a **roadmap consensus engine** in `src/routes/roadmap.rs` that
uses *semantic clustering* to merge duplicate ideas, a *MaxDiff voting
arena* to gather preferences efficiently, and *Elo ratings* to produce
a global, evolving leaderboard.

The module comment is a spec in five sentences:

```rust
//! Roadmap Consensus Engine — semantic clustering + MaxDiff/Elo voting.
//!
//! Users submit feature ideas (`POST /api/roadmap/suggest`); the backend
//! embeds the text via Ollama (nomic-embed-text) and either joins the nearest
//! existing cluster (cosine distance < threshold) or spawns a new one.
//! The Arena (`GET /api/roadmap/arena`) serves 4 clusters; a MaxDiff vote
//! (`POST /api/roadmap/vote`, best+worst) is translated into 6 virtual 1v1
//! Elo matches to produce a global consensus ranking.
//!
//! Zero-PII: anonymous voters are keyed by client_id UUID, never IPs.
```

Three ideas from the world of machine learning and ranking theory,
packed into a single 468-line file. We'll take them one at a time —
and you'll be relieved to know the file itself is built so that each
piece is testable in isolation.

### 37.1 The schema: clusters, suggestions, and a vote log with teeth

Migration `011_roadmap_consensus.sql` defines the whole arena. The
star is `feature_clusters`, which is a `pgvector` table:

```sql
CREATE EXTENSION IF NOT EXISTS vector;

-- The grouped concepts (what users actually vote on) — created FIRST so the
-- FK in feature_suggestions below resolves cleanly.
CREATE TABLE IF NOT EXISTS feature_clusters (
    id                   SERIAL PRIMARY KEY,
    representative_text  TEXT NOT NULL,
    embedding            VECTOR(768) NOT NULL,
    elo_rating           REAL DEFAULT 1500.0,
    matches_played       INT DEFAULT 0,
    times_picked_best    INT DEFAULT 0,
    times_picked_worst   INT DEFAULT 0,
    created_at           TIMESTAMPTZ DEFAULT NOW(),
    status               TEXT DEFAULT 'open'  -- 'open' | 'implemented' | 'archived'
);
```

💡 **Key Concept — an embedding column is a semantic fingerprint.** The
`embedding VECTOR(768)` column stores each idea as a 768-dimensional
vector — a point in semantic space produced by Ollama's
`nomic-embed-text` model. The crucial property: *similar texts land
near each other*. "Dark mode" and "change the font to black" become
nearby points, even though they share almost no words. The migration
comment is worth quoting in full, because it records a decision
process: *"nomic-embed-text is 768-d, NOT 384 — the blueprint's 384
was for all-MiniLM-L6-v2; we use nomic-embed-text since it's
installed."* That's a real debugging story: someone planned for a
384-dimensional model, discovered the installed one produces 768, and
wrote the actual value into the migration so the next person doesn't
re-litigate it.

The table also *is* the leaderboard: `elo_rating` starts at 1500,
`matches_played` counts arena appearances, and
`times_picked_best`/`times_picked_worst` feed the controversy score.

The vote log has teeth — a `UNIQUE` constraint that enforces
no-double-voting *for logged-in users*:

```sql
-- Audit log for arena votes (prevents double voting on the same set)
CREATE TABLE IF NOT EXISTS arena_votes (
    id              SERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id) ON DELETE CASCADE,
    client_id       varchar(36),
    cluster_ids     INT[] NOT NULL,           -- the 4 clusters presented (sorted)
    best_cluster_id INT REFERENCES feature_clusters(id),
    worst_cluster_id INT REFERENCES feature_clusters(id),
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (user_id, cluster_ids)
);
```

And migration `016_roadmap_consensus_statuses.sql` turns the loose
status comment into a real constraint with four values, the fourth of
which matters for the consensus UI:

```sql
-- 016: Roadmap consensus statuses — 'shipped' | 'rejected' | 'deferred' | 'open'
ALTER TABLE feature_clusters
    ADD CONSTRAINT feature_clusters_status_check
    CHECK (status IN ('open', 'shipped', 'rejected', 'deferred'));
```

The migration comment explains the lifecycle: the seed bin marks
already-shipped features `shipped` (so the consensus page can show
them as done), rejected ones carry their reason in the text, deferred
ones are parked. The `CHECK` is the contract that keeps the UI badges
valid.

### 37.2 The clustering: embeddings, cosine distance, and a threshold

The constants at the top of `roadmap.rs` are the tuning knobs of the
whole system:

```rust
/// Clustering threshold: cosine distance below this means "same idea".
/// nomic-embed-text is well-suited to ~0.15-0.25 thresholds for short texts;
/// 0.22 is a reasonable middle ground (strict enough to avoid "dark mode" and
/// "change font to black" merging, loose enough to group paraphrases).
pub const CLUSTER_DISTANCE_THRESHOLD: f64 = 0.22;
/// K-factor for the Elo updates.
pub const ELO_K: f64 = 32.0;
/// Max submissions per day per identity (anti-spam).
pub const MAX_SUGGESTIONS_PER_DAY: i64 = 3;
```

Three constants, each with a *rationale* in its doc comment — the
threshold one even cites the trade-off it's balancing. When you write
magic numbers, write the reason next to them; someone (probably you, in
six months) will need it.

The suggest handler is where clustering happens. It embeds the text,
finds the nearest open cluster, and either joins it or spawns a new
one:

```rust
// Embed via Ollama (best-effort — on failure store unclustered).
let embedding: Option<Vec<f32>> = state
    .ollama
    .embed(&text)
    .await
    .map_err(|e| tracing::warn!("ollama embed failed: {e}"))
    .ok();

let cluster_id: i32 = if let Some(emb) = embedding {
    let emb_sql = format!("[{}]", emb.iter().map(|f| format!("{f:.6}")).collect::<Vec<_>>().join(","));
    // Nearest existing open cluster
    let nearest: Option<(i32, f64)> = sqlx::query_as(
        "SELECT id, (embedding <=> $1::vector) AS dist FROM feature_clusters WHERE status = 'open' ORDER BY embedding <=> $1::vector LIMIT 1",
    )
    .bind(&emb_sql)
    .fetch_optional(&state.db)
    .await?;

    match nearest {
        Some((cid, dist)) if dist < CLUSTER_DISTANCE_THRESHOLD => {
            // Join the existing cluster (representative text unchanged)
            sqlx::query("UPDATE feature_clusters SET matches_played = matches_played WHERE id = $1")
                .bind(cid)
                .execute(&state.db)
                .await?;
            cid
        }
        _ => {
            // Spawn a new cluster
            sqlx::query(
                "INSERT INTO feature_clusters (representative_text, embedding) VALUES ($1, $2::vector) RETURNING id",
            )
            .bind(&text)
            .bind(&emb_sql)
            .fetch_one(&state.db)
            .await
            .map(|row: sqlx::postgres::PgRow| row.get::<i32, _>(0))?
        }
    }
} else {
    // Ollama down — no cluster yet; the raw suggestion is still saved.
    0
};
```

Four craft points here:

1. **The operator is `<=>`** — pgvector's cosine-distance operator. The
   query `ORDER BY embedding <=> $1::vector LIMIT 1` is a nearest
   neighbor search: Postgres (with the ivfflat index from the
   migration) finds the closest cluster in semantic space.
2. **The embedding is serialized by hand** — `format!("[{:.6},...]")`
   turns the `Vec<f32>` into Postgres's vector literal syntax. This is
   the awkward seam between Rust and pgvector; it works, and the
   `{:.6}` formatting keeps the literals compact.
3. **The `if dist < threshold` guard** decides join vs. spawn. Below
   0.22 you're the same idea; above it you're new. The no-op
   `UPDATE ... SET matches_played = matches_played` in the join branch
   looks silly — it *is* a no-op — but it's a deliberate touchpoint
   where future "cluster touched" bookkeeping can hang. Comment says
   so: *"representative text unchanged"*.
4. **The graceful degradation is explicit**: if Ollama is down
   (`embedding: None`), the suggestion is still saved with
   `cluster_id = 0` → `None` in the insert. The idea is never lost —
   it just waits for an embedding that never comes. Compare this to
   the `candidates` endpoint in Chapter 33: *the community feature
   degrades, it doesn't fail.*

And the anti-spam guard runs *before* all of that:

```rust
// Anti-spam: max 3 suggestions/day per identity.
let since = chrono::Utc::now() - chrono::Duration::days(1);
let recent: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM feature_suggestions WHERE (user_id = $1 OR client_id = $2) AND created_at >= $3",
)
.bind(user_id_or(&user))
.bind(&client_id)
.fetch_one(&state.db)
.await?;
if recent >= MAX_SUGGESTIONS_PER_DAY {
    return Err(AppError::BadRequest(-1, "too many suggestions today (max 3)".into()));
}
```

`(user_id = $1 OR client_id = $2)` — the identity is *either* the
account *or* the anonymous client fingerprint. That's the Zero-PII
design in action: anonymous users still have a stable identity (a
client UUID, never an IP), so the spam limit holds for them too.

### 37.3 The arena: why MaxDiff, and how 4 picks become 6 matches

Here's the problem the arena solves. If you asked every user to rank
every feature, you'd drown them — and they'd stop voting. If you asked
them to vote yes/no on individual features, you'd get no *relative*
signal — a hundred features at "80% yes" tell you nothing about which
to build first. The classic answer is **pairwise comparison** (this vs.
that), but 100 features means ~5,000 pairs, which is also hopeless.

The design FicHub uses is **MaxDiff**: show the user *four* clusters
and ask only two questions — *which is the best? which is the worst?*
That one choice carries far more information than a binary vote, and
the math turns it into **six virtual 1v1 matches**:

```rust
/// Translate a MaxDiff choice (best, worst, two neutral) into 6 virtual
/// 1v1 Elo updates. Returns a map of cluster_id -> new_rating.
///
/// `ids` is the 4 presented cluster ids (any order); `best` and `worst` must
/// both be in `ids` and distinct. The match matrix:
///   best  beats worst + both neutrals (3 wins)
///   worst loses to best + both neutrals (3 losses)
///   each neutral beats worst, loses to best, draws the other neutral
pub fn maxdiff_elo(
    ids: &[i32],
    ratings: &[(i32, f64)],
    best: i32,
    worst: i32,
) -> Vec<(i32, f64)> {
    let rating_of = |id: i32| ratings.iter().find(|(i, _)| *i == id).map(|(_, r)| *r).unwrap_or(1500.0);
    let mut new_ratings: Vec<(i32, f64)> = ratings.to_vec();

    let mut update = |id: i32, opponent: i32, score: f64| {
        let idx = new_ratings.iter().position(|(i, _)| *i == id).expect("id present");
        new_ratings[idx].1 = elo_update(new_ratings[idx].1, rating_of(opponent), score);
    };

    for &id in ids {
        if id == best {
            // Best: wins vs the other three
            for &opp in ids {
                if opp != best {
                    update(id, opp, 1.0);
                }
            }
        } else if id == worst {
            // Worst: loses vs the other three
            for &opp in ids {
                if opp != worst {
                    update(id, opp, 0.0);
                }
            }
        } else {
            // Neutral: beats worst, loses to best, draws the other neutral
            update(id, worst, 1.0);
            update(id, best, 0.0);
            for &opp in ids {
                if opp != best && opp != worst && opp != id {
                    update(id, opp, 0.5);
                }
            }
        }
    }

    new_ratings
}
```

Read the match matrix carefully — it's the heart of the system:

- **Best** gets three wins (vs. worst and both neutrals).
- **Worst** gets three losses (vs. best and both neutrals).
- Each **neutral** beats worst, loses to best, and *draws* the other
  neutral (0.5 each) — because you didn't tell us which neutral you
  preferred, the honest assumption is a tie.

💡 **Key Concept — Elo is a transfer of rating, not a score.** The
formulas `expected_score` and `elo_update` are the standard chess
math:

```rust
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

pub fn elo_update(rating: f64, opponent_rating: f64, score: f64) -> f64 {
    let expected = expected_score(rating, opponent_rating);
    rating + ELO_K * (score - expected)
}
```

The *expected* score is a logistic curve: two equal ratings predict
0.5; a 400-point gap predicts ~0.91. The *update* is
`K × (actual − expected)`: beat someone you were expected to beat and
you gain little; beat someone rated higher and you gain a lot. The
rating difference between two clusters *is* the community's preference
signal — a global ranking emerges from thousands of tiny transfers,
and every cluster's rating is meaningful relative to every other's.

And notice the *elegance of the function signature*: `maxdiff_elo`
takes `ratings: &[(i32, f64)]` and returns new ratings — it's a *pure
function*. No database, no state. That's what makes the test battery
at the bottom of the file possible:

```rust
#[test]
fn maxdiff_best_soars_worst_tanks_neutrals_shift_little() {
    let ids = vec![1, 2, 3, 4];
    let ratings = vec![(1, 1500.0), (2, 1500.0), (3, 1500.0), (4, 1500.0)];
    let out = maxdiff_elo(&ids, &ratings, 1, 4);

    let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
    let r4 = out.iter().find(|(i, _)| *i == 4).unwrap().1;
    let r2 = out.iter().find(|(i, _)| *i == 2).unwrap().1;
    let r3 = out.iter().find(|(i, _)| *i == 3).unwrap().1;

    assert!(r1 > 1540.0, "best gains big: {r1}");
    assert!(r4 < 1460.0, "worst loses big: {r4}");
    assert!(r2 > r4 && r2 < r1, "neutral 2 between: {r2}");
    assert!(r3 > r4 && r3 < r1, "neutral 3 between: {r3}");
}
```

The test asserts the *invariants* of the system — best soars above
1540, worst sinks below 1460 (K=32 × 3 matches each way), neutrals end
up strictly between them. And the property tests keep going:

```rust
#[test]
fn maxdiff_rating_gap_reduces_transfer() {
    // A 1800-rated best vs a 1200-rated worst: the win transfers less Elo
    // than an even match would.
    let ids = vec![1, 2, 3, 4];
    let ratings = vec![(1, 1800.0), (2, 1500.0), (3, 1500.0), (4, 1200.0)];
    let out = maxdiff_elo(&ids, &ratings, 1, 4);
    let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
    assert!(r1 > 1800.0, "still gains");
    assert!(r1 - 1800.0 < 32.0 * 3.0, "gain bounded by K*matches");
}
```

The second test pins a subtler property: an upset transfers *less* the
bigger the gap — an 1800 beating a 1200 gains less than it would
beating an equal. That's Elo's self-correcting behavior, frozen into a
test so a future refactor can't break it silently.

⚠️ **Watch Out — Elo's baseline is arbitrary; only *differences* mean
anything.** 1500 is the starting rating, not a quality score. A
cluster at 1510 isn't "good" — it's "slightly above the initial
expectation". And Elo has quirks: early votes swing ratings wildly (few
data points), and a cluster that never appears in the arena keeps a
stale rating. That's why the arena query orders by `matches_played
ASC` — to get *fresh* clusters into the ring and establish their
baseline. When you adopt a ranking algorithm, adopt its caveats too.

### 37.4 The vote handler: validation, uniqueness, atomicity

`vote_handler` is where the pure math meets the messy world of HTTP
and concurrent users. First, validation of the request shape:

```rust
if req.cluster_ids.len() != 4 {
    return Err(AppError::BadRequest(-1, "arena must present exactly 4 clusters".into()));
}
if !req.cluster_ids.contains(&req.best_cluster_id) || !req.cluster_ids.contains(&req.worst_cluster_id) {
    return Err(AppError::BadRequest(-1, "best/worst must be among the presented clusters".into()));
}
if req.best_cluster_id == req.worst_cluster_id {
    return Err(AppError::BadRequest(-1, "best and worst must differ".into()));
}
```

Three checks, each catching a distinct way a client can lie. Then the
uniqueness check — the no-double-vote rule. This is where the
`user_id` vs `client_id` split gets subtle:

```rust
let mut sorted_ids = req.cluster_ids.clone();
sorted_ids.sort_unstable();

// Uniqueness: no double-vote on the same set. Logged-in users are keyed
// by user_id; anonymous clients (user_id None) by client_id (NULL user_id
// rows don't collide on the UNIQUE (user_id, cluster_ids) index, so we
// enforce per-client uniqueness here).
let uid = user_id_or(&user);
let exists: bool = if uid.is_some() {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE user_id = $1 AND cluster_ids = $2)",
    )
    .bind(uid)
    .bind(&sorted_ids)
    .fetch_one(&state.db)
    .await?
} else {
    match &client_id {
        Some(cid) => {
            sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE client_id = $1 AND cluster_ids = $2 AND user_id IS NULL)",
            )
            .bind(cid)
            .bind(&sorted_ids)
            .fetch_one(&state.db)
            .await?
        }
        None => false,
    }
};
if exists {
    return Err(AppError::BadRequest(-1, "you already voted on this set".into()));
}
```

Two details are worth slowing down on:

1. **`sorted_ids`** — the cluster ids are sorted *before* being stored
   and compared. `[4,1,3,2]` and `[1,2,3,4]` are the same arena set,
   and sorting makes the uniqueness check treat them as identical. The
   database `UNIQUE (user_id, cluster_ids)` also compares the stored
   array — sorted at insert time — so the constraint works.
2. **Why the manual check at all?** The `UNIQUE` constraint covers
   `(user_id, cluster_ids)` — but anonymous voters have `user_id =
   NULL`, and in Postgres, **NULLs don't collide in unique
   constraints**. Two anonymous votes with NULL user_id could both
   insert the same set. So the handler enforces per-client uniqueness
   *by hand* for the anonymous path — and the comment explains the
   whole trap. This is the same composite-key story from Chapter 33,
   with a NULL-shaped hole in it.

Then the transaction. The vote row, the Elo updates, and the counter
bumps all need to land together or not at all:

```rust
// Persist the vote + apply Elo + bump counters atomically.
let mut tx = state.db.begin().await?;
sqlx::query(
    "INSERT INTO arena_votes (user_id, client_id, cluster_ids, best_cluster_id, worst_cluster_id) VALUES ($1, $2, $3, $4, $5)",
)
.bind(user_id_or(&user))
.bind(&client_id)
.bind(&sorted_ids)
.bind(req.best_cluster_id)
.bind(req.worst_cluster_id)
.execute(&mut *tx)
.await?;

for (id, new_rating) in &new_ratings {
    let delta = new_rating - 1500.0; // only meaningful relative to baseline
    let _ = delta;
    sqlx::query(
        r#"
        UPDATE feature_clusters SET
            elo_rating = $1,
            matches_played = matches_played + 1,
            times_picked_best = times_picked_best + CASE WHEN $2 THEN 1 ELSE 0 END,
            times_picked_worst = times_picked_worst + CASE WHEN $3 THEN 1 ELSE 0 END
        WHERE id = $4
        "#,
    )
    .bind(new_rating)
    .bind(*id == req.best_cluster_id)
    .bind(*id == req.worst_cluster_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
}
tx.commit().await?;
```

💡 **Key Concept — a transaction is a promise of all-or-nothing.** If
the process dies between the INSERT and the UPDATEs, without the
transaction you'd have a recorded vote that never moved any Elo — the
leaderboard and the audit log out of sync, forever. `begin()` →
statements on `&mut *tx` → `commit()` makes the five writes one
indivisible unit. The `CASE WHEN $2 THEN 1 ELSE 0 END` trick is
SQL-boolean-to-integer: the boolean bind becomes 1 or 0, bumping the
right counter in the same UPDATE that sets the rating. One statement
per cluster, no second round trips.

The handler returns the applied ratings so the UI can animate the
change:

```rust
Ok(Json(json!({
    "err": 0,
    "applied": new_ratings.iter().map(|(id, r)| json!({ "cluster_id": id, "new_elo": r })).collect::<Vec<_>>(),
})))
```

### 37.5 The arena draw and the public consensus

The arena needs to present *four* clusters — and it wants to expose
fresh ones. The draw is a weighted-random-ish trick:

```rust
let rows: Vec<(i32, String, i32, f64)> = sqlx::query_as(
    r#"
    SELECT id, representative_text, matches_played, elo_rating::float8
    FROM feature_clusters
    WHERE status = 'open'
    ORDER BY matches_played ASC, random()
    LIMIT 4
    "#,
)
.fetch_all(&state.db)
.await?;

if rows.is_empty() {
    return Ok(Json(json!({ "err": 0, "clusters": [], "message": "No features yet — be the first to suggest one!" })));
}
```

⚠️ **Watch Out — "random" in SQL is a full-table scan.** `ORDER BY
random()` is famously expensive on big tables — Postgres must sort
everything to pick four random rows. Here it's acceptable because the
arena is *not* a hot path (a human clicks a button, a handful of times
a day) and the `ORDER BY matches_played ASC` prefix limits the sort
scope in practice. But keep this in your head: `ORDER BY random()` is
the first thing you delete when a query shows up in the slow-query log.
The comment in the code says as much: *"True weighted random in SQL is
awkward; a 'least played first' draw is a fine approximation and
cheap."*

And the empty case returns a *friendly message* — "No features yet —
be the first to suggest one!" — instead of an empty array with no
explanation. Small UX touch, easy to skip, and it's exactly the kind
of thing that makes a community feature feel alive on day one.

The public consensus endpoint (`consensus_handler`, no auth) produces
the leaderboard and the controversy metric:

```rust
let leaderboard = sqlx::query_as::<_, (i32, String, f64, i32, i32, i32, i64, String)>(
    r#"
    SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
           c.times_picked_best, c.times_picked_worst, COUNT(s.id)::bigint AS suggestions,
           c.status
    FROM feature_clusters c
    LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
    GROUP BY c.id
    ORDER BY c.elo_rating DESC
    LIMIT 100
    "#
)
.fetch_all(&state.db)
.await?;
```

`LEFT JOIN feature_suggestions` + `COUNT(s.id)` — the number of raw
suggestions that landed in each cluster, zero for clusters with none.
`ORDER BY c.elo_rating DESC` is the leaderboard. And the controversy
query is the *product insight* of the whole system:

```rust
let controversy = sqlx::query_as::<_, (i32, String, i32, i32, i32, i32, f64)>(
    r#"
    SELECT id, representative_text, matches_played,
           times_picked_best, times_picked_worst,
           (times_picked_best + times_picked_worst) AS controversy,
           elo_rating::float8
    FROM feature_clusters
    WHERE status = 'open'
    ORDER BY matches_played DESC
    LIMIT 100
    "#
)
.fetch_all(&state.db)
.await?;
```

💡 **Key Concept — controversy is a derived metric, computed not
stored.** A feature that's picked *both* best and worst a lot isn't
popular or unpopular — it's *divisive*. `times_picked_best +
times_picked_worst` computed in SQL is the controversy score: volume
(`matches_played`) times polarization. The roadmap page renders the
leaderboard *and* the controversy list side by side, so the team can
see not just what's wanted but what's *argued about*. Deriving it at
query time (rather than storing a `controversy` column) means the
definition can evolve without a migration — and it's the same
"derivable data, computed at the edge" principle from Chapter 35.

And the whole thing is deliberately public. `consensus_handler` takes
no `AuthUser` at all — the comment says *"Zero-PII: cluster texts +
aggregates only (no user/client mapping)"*. The leaderboard is a
community artifact, and it's served to the world.

Finally — the seed. The roadmap didn't start empty: `src/roadmap_seed.rs`
seeds the arena with the program's own feature list, so the first
visitors have something to vote on. It also records the system's
self-awareness — look at the shipped list:

```rust
pub const SHIPPED_FEATURES: &[&str] = &[
    "Send to Kindle: email an EPUB of any fic to your Kindle address",
    "Strict genre filtering: hard-scoped tag/domain exclusions in search",
    "Offline PWA: installable app shell that works without connectivity",
    "Boolean search: AND/OR/NOT operators, quoted phrases and field scopes",
    ...
    "Roadmap consensus engine: arena voting with Elo-ranked feature leaderboard",
    ...
];
```

The roadmap engine *votes on itself* — "Roadmap consensus engine" is
listed among the shipped features, because by the time you read this
chapter, it is. That's the kind of detail that makes a codebase feel
alive: the dogfooding is explicit, in the seed data.

🧪 **Try It Yourself — run a consensus cycle.**

```bash
# 1. The arena: get your four clusters (any visitor, no auth needed)
curl -s http://localhost:8080/api/roadmap/arena | jq .

# 2. Suggest a feature (anonymous; the client_id header is your identity)
curl -s -X POST http://localhost:8080/api/roadmap/suggest \
  -H "Content-Type: application/json" -H "x-client-id: demo-client-1" \
  -d '{"text": "Dark mode for the reader"}' | jq .
#    → {"cluster_id": N, "clustered": true} — joined or spawned

# 3. Suggest a paraphrase — watch it cluster into the SAME cluster
curl -s -X POST http://localhost:8080/api/roadmap/suggest \
  -H "Content-Type: application/json" -H "x-client-id: demo-client-2" \
  -d '{"text": "Change the reader font to black with a black background"}' | jq .

# 4. Vote on the arena set (best + worst from the /arena response)
curl -s -X POST http://localhost:8080/api/roadmap/vote \
  -H "Content-Type: application/json" -H "x-client-id: demo-client-1" \
  -d '{"cluster_ids": [1,2,3,4], "best_cluster_id": 1, "worst_cluster_id": 4}' | jq .
#    → "applied" shows the six Elo updates

# 5. Try the same vote twice — the uniqueness rule bites
curl -s -X POST http://localhost:8080/api/roadmap/vote \
  -H "Content-Type: application/json" -H "x-client-id: demo-client-1" \
  -d '{"cluster_ids": [1,2,3,4], "best_cluster_id": 1, "worst_cluster_id": 4}' | jq .
#    → "you already voted on this set"

# 6. The public leaderboard + controversy
curl -s http://localhost:8080/api/roadmap/consensus | jq '.leaderboard[0:5]'
curl -s http://localhost:8080/api/roadmap/consensus | jq '.controversy[0:3]'
```

If your two suggestions landed in the same `cluster_id`, you just
watched semantic clustering work with your own eyes: two sentences
with almost no shared words became one feature on the board.

---

That's the roadmap consensus engine: embeddings and cosine distance
for deduplication, MaxDiff for efficient preference capture, Elo for
an ever-evolving ranking, transactions for atomicity, NULL-aware
uniqueness for anonymous voters, and pure functions everywhere the
math could be tested without a database. And with it, the community
loop of Part 8 closes: readers ask for fics (Chapter 33), curate them
(Chapter 34), follow them (Chapter 35), subscribe to them (Chapter
36), and decide what the platform itself becomes next (this chapter).

---

## Part 8 Wrap-Up

Five chapters, five ways a community *does things together*:

- **Chapter 33** — the requests board: `create` → `answer` (with
  URL-ingest via the Part 4 scraper) → `vote` → `accept`, with the
  composite-primary-key vote table, soft-delete discipline, and
  best-effort notifications. The requester's prompt became a ranked,
  accepted, answered artifact.
- **Chapter 34** — two generations of organization: the flat
  hard-deleted shelf and the ordered, blurbed, soft-deleted, shareable
  reading list. You learned to read a design *conversation* in one
  codebase.
- **Chapter 35** — the archive's own pages: series in reading order
  with `next_in_series` computed at the edge, author bibliographies
  that handle canonical-name resolution and orphaned `fic_info` rows,
  and the role-gated curator merge workflow.
- **Chapter 36** — the syndication layer: three Atom feeds, a pure XML
  builder with injection-proof escaping, token-in-query-string auth,
  and a polymorphic follows JOIN.
- **Chapter 37** — the consensus engine: semantic clustering, MaxDiff
  voting, Elo ranking, and a public leaderboard with a controversy
  metric — the community deciding the platform's future, with tests
  pinning the math.

The pattern that runs through all of it: **community features are
database features**. The "one vote per user" rule is a primary key.
The "no duplicate answers" rule is a unique constraint. The "list
order" is a position column. The "no double-voting" rule is a sorted
array plus a unique index. When you build your next community feature,
start by asking *what does the database have to forbid?* — then make
it a constraint, and the app logic becomes the easy part.

Meanwhile, every vote you collected — on answers, on requests, on
features — is a *signal*. The requests board ranks answers by
community fit; the roadmap ranks features by community preference.
And the bookmarks, ratings, and follows from Part 7 are signals too —
hundreds of thousands of them, sitting in `work_ratings`, `follows`,
and `reading_stats`, waiting for someone to mine them.

In **Part 9 — The Recommendation Platform**, that's exactly what we
do. The strategy registry in `src/recommender/` wraps multiple
recommendation engines behind one `RecStrategy` trait — the same
engine the requests board's `candidates` endpoint reached into at the
end of Chapter 33. We'll build the co-occurrence engine that turns
"people who bookmarked this also bookmarked..." into recommendations,
watch how FicHub A/B-tests strategies safely in shadow mode before
showing them to users, and wire up the personal recommendation feed
that turns every reader's history into their next favorite story.
Your bookmarks are about to get a second job. See you there.

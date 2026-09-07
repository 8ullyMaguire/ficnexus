# Community Forum

The [forum](/forum) is where the community talks — fic discussion, meta,
announcements, and everything that doesn't fit in a comment thread.
Categories hold topics, topics hold posts, and the system remembers where
you left off in every thread.

You need an account to post. Reading is open to everyone (unless an admin
sets `FORUM_PUBLIC_READ=false`); mod-only categories stay hidden from
anonymous visitors.

## Reading

- **Categories** (`/forum`) — every visible category with topic and unread
  counts.
- **Topics in a category** (`/forum/{category}`) — newest activity first,
  cursor-paginated.
- **A topic** (`/forum/board/{slug}.{id}`) — the opening post plus replies.
  Every topic gets a stable slug URL at creation (e.g.
  `/forum/board/what-s-your-opinion-on-the-site.42`) generated from the
  title. The slug never changes, so links keep working even if the topic is
  renamed. The older `/forum/{category}/{id}` URLs still work.
- **Views that help you catch up** — `/forum/recent` (latest activity),
  `/forum/unread` (topics with posts you haven't read, login required),
  `/forum/popular` (most-viewed), and a per-topic RSS feed at
  `/api/forum/rss`.
- **Read state** — opening a topic marks it read; unread topics show a dot /
  badge. You can also mark a topic read explicitly
  (`POST /api/forum/topics/{id}/read`), e.g. to skip a long thread.
- **Preferences** — posts-per-page and topic sort order live in
  `GET/PUT /api/forum/preferences` and apply across your sessions.

## Posting

- **New topic** (`/forum/new` or `POST /api/forum/topics`) — pick a
  category, write a title (max 120 characters) and a body (max 20,000
  characters). The first post becomes the topic's opening post.
- **Reply** (`POST /api/forum/topics/{id}/posts`) — plain body, optional
  `quote_of` to quote another post (the topic page renders a preview of the
  quoted post).
- **Tags** — topics can carry free-text tags
  (`GET/POST /api/forum/topics/{id}/tags`), which feed the topic's
  full-text search entry so tag words help the topic surface in search.
- **Editing** — the topic/post author can edit directly, and every edit is
  recorded in immutable history (`GET /api/forum/edits/{topic|post}/{id}`).
  Non-authors (below curator level) can *propose* an edit instead; curators
  review the proposal queue (`GET /api/forum/edits/queue`,
  `POST /api/forum/edits/{id}/review`) and approve or reject it.
- **Deleting** — authors can soft-delete their own topics/posts; curators
  can delete anything. Deleted content is hidden, not purged.
- **Mentions** — write `@username` in a reply and the mentioned user gets a
  notification (deduped against users already notified as followers).

## Following & notifications

- **Follow a topic** (`POST /api/forum/topics/{id}/follow` toggles,
  `GET …/follow` reads state) to get a `forum_reply` notification whenever
  someone replies. The reply author is never notified of their own post.
- Notifications link straight back to the topic's board URL and carry the
  topic id, so clients can deep-link and badge correctly.

## Search

`/forum/search` (or `GET /api/forum/search?q=…&category=…`) runs full-text
search over topic titles/bodies *and* post bodies, ranked by relevance with
`<mark>`-highlighted snippets. Anonymous visitors only see public
categories. Limit is capped at 50 per page and `next_cursor` is always null
(deterministic ordering: rank, then post id, then topic id).

For fic search rather than forum search, see [Finding Great
Stories](./searching.md) — including the find-fic-by-name box and For-You
chips.

## Moderation points & metamoderation

Instead of a small fixed mod team, trusted members earn **moderation
points** and spend them from the queue:

- **Status** (`GET /api/forum/moderation/status`) — your points, window
  expiry, and eligibility.
- **Queue** (`GET /api/forum/moderation/queue`) — posts needing attention,
  reported and low-score first.
- **Moderate** (`POST /api/forum/posts/{postId}/moderate` with a `reason`) —
  spends one point; reasons map to score deltas server-side. You can't
  moderate your own posts or the same post twice.
- **History** (`GET /api/forum/posts/{postId}/moderations`) — the public
  moderation history of a post, mirroring the modlog.
- **Metamoderation** — moderator actions are audited anonymously:
  `GET /api/forum/metamod/queue` serves random under-reviewed actions with
  the moderator anonymized, and you vote `fair` / `unfair` / `unsure`
  (`POST /api/forum/metamod/{actionId}/vote`). Rolling unfair-rates and
  cooldowns keep the system honest. Full detail on a grant at
  `GET /api/forum/metamod/grants/{id}` (+ `/verdict` to resolve).
- **Curator fast-hide** (`POST /api/admin/forum/hide/{postId}`, auto-expires
  after 72h), **lock / pin topics**
  (`POST /api/admin/forum/topics/{id}/lock|pin`), and **scoped bans**
  (`POST /api/admin/forum/bans` with `scope: forum|category`) round out the
  toolkit. Bans block forum writes only — reading and fic features keep
  working. Your per-user grants are visible at
  `GET /api/forum/moderation/user/{userId}/grants`.

All of this is level-gated: your [level](./profile.md) (not just your role
label) decides whether you can moderate, fast-hide, or run admin actions.
Every action lands in the public [modlog](/modlog) — see
[Transparency](./transparency.md).

## Reactions

Posts accept emoji reactions (`POST /api/forum/posts/{id}/react`,
`GET …/reactions`, `DELETE …/reactions`) — a lightweight positive-only
signal alongside the moderation-points system.

---

*Next: [Getting Recommendations](./recommendations.md)*

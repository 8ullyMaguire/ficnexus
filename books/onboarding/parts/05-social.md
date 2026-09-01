## Part 5 — Exports & the Reader

### Chapter 16: From URL to EPUB

`src/routes/export.rs` is the export pipeline — the heart of the original
product. Follow a URL through the whole journey:

1. **Pick the scraper.** `find_specific_or_fff(url)` (Chapter 14) decides
   who handles this URL.
2. **Check the body cache.** `load_body(config, url_id)`. On a hit, use the
   cached chapters — no network.
3. **On a miss, scrape.** `scraper.lookup()` for metadata, then
   `scraper.fetch_chapters()` for the text.
4. **Persist to the body cache.** `save_body(...)` — the archive grows.
5. **Build the format.** The EPUB builder (pure Rust, `export/`), or HTML /
   TXT / MD. MOBI/PDF/AZW3 go through the Calibre sidecar (a Docker
   container).
6. **Stream the file back.** Set `Content-Type`, `Content-Disposition`
   filename, and return the bytes.

The export handler is also where the **self-healing telemetry** hooks in:
on a scrape failure, `state.heal.record_failure(...)` records the URL,
error, and a snapshot — feeding `scrape_failures` (migration 029) and the
classifier that decides whether the failure is transient, blocked, or
structural.

**The semaphore guard.** `cache::get_export_semaphore(&state.cache_semaphores,
url_id, etype)` prevents two concurrent exports of the same URL from
double-scraping. The map is *bounded* (a 10k cap) so it can't grow forever
— a deliberate fix for a real leak. In-flight requests hold an `Arc` to the
semaphore, so removing the key from the map after completion is safe.

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;   // only one export per url_id at a time
// ... scrape + build + stream ...
```

> 🧪 **Try it:** read the export handler and count the steps between "URL
> comes in" and "file goes out." Then look at where `save_body` is called —
> that's the archive growing.
>
> ⚠️ **Watch out:** the semaphore is per `(url_id, etype)` — EPUB and HTML
> exports of the same fic can run in parallel, but two EPUBs of the same
> fic cannot. That's the intended behavior.
>
> 💡 **Key concept:** the export pipeline is cache-first. The body cache
> turns a network scrape into an occasional cost instead of a per-request
> one.

### Chapter 17: The Reader + The SPA

The SPA talks to `/api/*` via `frontend/src/lib/api/client.ts`. Every
frontend feature is: a page under `frontend/src/routes/`, an API call
through the client, and (usually) a `page.test.ts` vitest file.

The reader page (`/read/[urlId]`) is the flagship frontend feature:

- Fetches chapter HTML from the API (cache-first, thanks to the body
  cache).
- Tracks scroll progress in `localStorage` (keyed `fichub:reader:state:{url_id}`)
  so you resume where you left off.
- Offers a "Next Up" panel: sequel → community pick → readers-also-
  bookmarked. That panel is powered by the recommendation engine (Part 7).
- Has typography preferences and chapter navigation.

The frontend is a standard SvelteKit 5 app: `+layout.svelte` for the shell
and nav, `+page.svelte` per route, `+layout.ts` for client-side
initialization (e.g., PWA registration). The i18n system (`src/lib/i18n/`)
provides the `t('nav.modlog')` style translation calls across six
dictionaries.

One quirk to know: **route tests must be named `page.test.ts`.** The vitest
setup picks up exactly that pattern. If you name a test anything else next
to a route, it won't run.

> 🧪 **Try it:** open `frontend/src/routes/read/` and find the reader page.
> Then open `client.ts` and find how it attaches the JWT (`Authorization:
> Bearer <token from localStorage['fichub_token']>`).
>
> ⚠️ **Watch out:** the API client reads the token from
> `localStorage['fichub_token']`. The auth store may hold it in a different
> place — a known mismatch to check if personalization seems "silently
> disabled."
>
> 💡 **Key concept:** the frontend is a thin client over a rich API. The
> interesting logic is backend; the frontend renders and calls.

---

## Part 6 — Social, Community & Governance

### Chapter 18: Auth, Users & Roles

`src/routes/auth.rs` issues JWTs (`create_token`/`verify_token`). The
`AuthUser` extractor reads `Authorization: Bearer <jwt>` and yields the
user's id + role.

```rust
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
    pub role: i16,
    // ...
}
```

Roles are integers:

- **0** = regular user
- **1** = curator-ish (can propose merges/fixes)
- **5** = moderator (modlog actions, comment moderation)
- **10** = admin (everything, analytics, users)

Handlers gate with `auth.user_id` and role checks. Two gate styles you'll
see:

```rust
// "any logged-in user"
if auth.user_id == 0 {
    return Err(AppError::Unauthorized);  // or BadRequest(-403, ...)
}

// "admin only"
if auth.role < 10 {
    return Err(AppError::BadRequest(-403, "Admin access required".into()));
}
```

Anonymous requests get `AuthUser::default()` (id 0, role 0) — so "logged
in?" is `user_id != 0`. The JWT Claims shape is
`{ sub: i32, username, role, exp, iat }`.

> ⚠️ **Watch out:** role checks use `>=` thresholds. Check the existing
> pattern before writing your own gate — some endpoints are
> `require_logged_in`, others `role >= 10`, and a few use custom codes like
> `-403`. Match the neighborhood.
>
> 💡 **Key concept:** auth is a JWT in a header + a role integer. "Logged
> in" = `user_id != 0`; "admin" = `role >= 10`.

### Chapter 19: Bookmark Identity — Two Systems, Don't Conflate

This is a genuine footgun, documented in the project's skill notes, and
worth internalizing before you build personalization features.

There are **two bookmark systems**:

1. **`bookmarks`** — the shipped one. Keyed by `(user_id, url_id)`, driven
   by `AuthUser` (the JWT user). This is what the UI uses, and what
   personalization reads.
2. **`fic_bookmarks`** — legacy/anonymous. Keyed by `user_hash` (the
   sha256 of a lowercased AO3 profile URL), fed ONLY by the recommender
   worker's AO3-profile import. No user linkage.

Why the split? The old anonymous system predates accounts. The recommender
worker still imports AO3 profiles by hashing the profile URL. But anything
new that personalizes must key on `bookmarks.user_id` — never on
`fic_bookmarks`/`user_hash`, or you'll silently mix anonymous imports with
real user data.

The personal recommendations endpoint
(`/api/recommendations/personal`) uses bookmarks + anonymous download
signals from `request_log` (last 90 days, `url_id IS NOT NULL AND etype IN
('download','export')`). The gate is `PERSONAL_RECS_MIN_SIGNAL = 3` — fewer
than 3 signals returns `{"enough_data": false, "recs": []}`.

> ⚠️ **Watch out:** `request_log` has no user linkage and no `path` column
> — downloads are identified by `url_id IS NOT NULL` + the etype/export
> filename. If you need "who downloaded what," that's a deliberate gap.
>
> 💡 **Key concept:** when in doubt, key on `bookmarks.user_id`. The
> legacy anonymous system is worker-only.

### Chapter 20: The Modlog — Radical Transparency

`src/modlog.rs` + migration 034. Every moderator, curator, and admin action
is recorded — bans, role changes, upload approvals, comment removals, tag
merges, body-fix votes, flag resolves, and more.

The `record` helper is deliberately **best-effort**: it inserts into
`modlog` and never fails the main action. If the log write fails, the
action still succeeds — transparency is a goal, not a bottleneck.

```rust
crate::modlog::record(
    &state.db,
    auth.user_id,
    auth.username.clone(),
    "ban_user",
    "user",
    &user_id.to_string(),
    serde_json::json!({"banned": true}),
).await;
```

Any logged-in user can read `GET /api/modlog` (the `/modlog` page). This is
a product decision: **moderation is completely transparent** in FicHub.
When you add an admin action, call `modlog::record(...)` after the
successful DB write — it's a requirement, not an afterthought.

> 🧪 **Try it:** open `src/modlog.rs` and count the actions the codebase
> records. Then open `/modlog` on the live site.
>
> ⚠️ **Watch out:** the modlog is readable by any logged-in user — never
> write PII or sensitive details into the `details` JSON.
>
> 💡 **Key concept:** transparency is a feature. Log every admin action;
> make the log public to logged-in users.

### Chapter 21: Usage Analytics — Zero PII

`usage_events` (migration 033) + middleware. Every request records a `view`
(browsing) or `action` (export, vote, comment, ...) tagged with an
anonymous `X-Client-ID` — a random UUID stored in the browser's
localStorage, **never an IP**.

The middleware (`from_fn_with_state`) classifies each path: health = view,
meta = action, etc. The `/admin/analytics` dashboard shows:

- **Unique visitors** — daily (30d), weekly (12w), monthly (12m).
- **Engagement** — active users (performed actions) vs view-only users,
  with action events + total events + action rate.
- **Recent activity timeline** — the last 50 events (anonymous client,
  path, type, time).
- **Search analytics** — zero-result queries, trope popularity, search
  volume, and search→export conversion (searchers vs exporters, joined by
  anonymous client ID).

The zero-PII rule is absolute: client IDs only, aggregates never expose IPs.
`request_log` retains IPs internally for anti-bot, but analytics never read
them.

> 🧪 **Try it:** open `/admin/analytics` (as admin) and look at the cards.
> Then read the middleware that classifies view vs action.
>
> ⚠️ **Watch out:** if you add a new endpoint, decide its classification
> (view vs action) in the middleware — don't leave it unclassified.
>
> 💡 **Key concept:** measure without identifying. Anonymous client IDs
> give you funnels and engagement without PII.

### Chapter 21A: Ratings, Reviews & Comments

The social layer is where the community lives. Three subsystems worth
knowing in detail.

**Ratings (`work_ratings`).** Users rate a work 1-5 stars. The table is
UNIQUE per `(user_id, work_id)` — one rating per user per work. The
aggregate shown publicly is derived from ratings + the constructive filter:
public lists emphasize constructive feedback, never a raw dislike count.

**Reviews (`reviews`).** In-depth reviews, UNIQUE per `(user_id,
work_id)`. Reviews feed the recommendation engine through
`work_feedback_signals()` — they're signal, not just content.

**Comments (`comments`).** Threaded comments on works, with a
`constructive` flag. Moderators can hide or delete comments (each action is
recorded in the modlog). The comment triage feature uses Ollama to help
moderators classify incoming comments as constructive vs spam/toxic.

The house pattern for "user-generated content with moderation":

1. Insert with the author's user_id.
2. Validate content (length, spam heuristics).
3. Make it visible, with moderation hooks (hide/delete).
4. Record moderation actions in the modlog.

> 🧪 **Try it:** open `comments.rs` and find the hide/delete handlers.
> Notice they call `modlog::record` — transparency by default.
>
> ⚠️ **Watch out:** comment triage uses an LLM to *help* classify — the
> human moderator makes the final call. Never auto-hide based on model
> output alone.
>
> 💡 **Key concept:** social content is moderated + transparent. Every
> moderation action is logged and public.

### Chapter 21B: Follows, Notifications & the Updates Feed

The engagement loop:

**Follows (`follows.rs`).** Users follow works, authors, or other users.
The `follows` table tracks the target type + id.

**Updates feed (`updates.rs` + `/api/v1/feed`).** When a followed fic is
re-scraped and changes (new chapter, edited metadata), the feed shows it.
The refresh-fic re-scrape triggers follower notifications.

**Notifications (`notifications.rs`).** User-scoped notification rows for
events: "your request got an answer," "a fic you follow updated," etc.
Fic Requests M3 (a starter task) is about wiring the request board into
this notification plumbing.

The pattern: follow → track changes → notify → show in feed. Each piece is
a simple table + endpoint.

> 🧪 **Try it:** follow a fic on dev, trigger a re-scrape, and watch the
> feed + notification appear.
>
> ⚠️ **Watch out:** notification events should be idempotent — don't
> create duplicate notifications when a job runs twice.
>
> 💡 **Key concept:** follows + updates + notifications form the
> engagement loop; the pieces are simple but the wiring must be careful.

### Chapter 21C: Series, Authors, Lists & Shelves

Four content-organization features round out the social layer:

- **Series (`series.rs`)** — ordered works. The reader's "next in series"
  comes from here.
- **Authors (`authors.rs`)** — author profiles, bibliography, linked
  accounts. Keyed by canonical name.
- **Lists (`lists.rs`)** — curated reading lists: title, description,
  positioned items, blurbs. Users build themed collections.
- **Shelves (`shelves.rs`)** — personal shelves (like bookshelves for
  fics).

All four follow the same CRUD pattern: list → detail → create/update →
delete, with auth for ownership and modlog for moderation actions.

> 🧪 **Try it:** create a reading list on dev, add a few fics, and look at
> how `lists.rs` stores positions.
>
> ⚠️ **Watch out:** shelf/list items have ORDER (position) — don't treat
> them as unordered sets.
>
> 💡 **Key concept:** content organization is standard CRUD with order +
> ownership. The novelty is in how the reader surfaces it ("Next Up").

---

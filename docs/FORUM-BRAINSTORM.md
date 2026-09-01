# Forum Brainstorm — Ideas Pool, Ranked

> Companion to `docs/SPEC-COMMUNITY-PLATFORM.md` (v2 spec: moderation points +
> metamoderation + leveling), `docs/FORUM-API-CONTRACT.md`, and
> `docs/THREADLIGHT-FEATURES.md` (F1–F8 inventory). This file is the idea pool
> for **what comes after F8** — everything the forum could become, ranked
> best → worst by leverage, fit with FicHub's product direction, and effort.
>
> **How to read the ranks:** Tier A = high-leverage, fits the forum-first,
> positive-only, self-hosted direction. Tier B = good, would make the forum
> genuinely better. Tier C = interesting but needs care (abuse surface or
> scope creep). Tier D = anti-patterns — things this project deliberately
> rejects (from the Threadlight fit analysis + product decisions).
>
> Each idea lists a rough effort (S = hours, M = days, L = weeks). Ideas
> marked **[user]** came from the maintainer's own brainstorm; the rest are
> extensions of the same design space.
>
> 2026-08-15 — initial brainstorm (forum live, F1–F7 shipped, F8 pending).

---

## Tier A — high leverage, fits the direction

### A1. Level-scaled posting rate limits **[user]**
Posting cadence scales with level: lvl 1 → 1 post/hour; lvl 100 → no limit.
All thresholds config (`FORUM_POST_MIN_INTERVAL_MINUTES_AT_LEVEL0`,
`FORUM_POST_MIN_INTERVAL_MINUTES_AT_LEVEL100`, linear or stepped between).
- Why: the cheapest spam/abuse defense that *rewards* exactly the behavior
  leveling already measures (participation). Doubles as a "slow mode" for the
  cold-start cohort without an admin doing anything.
- Effort: S (one check in `create_topic`/`create_post`, config read, tests).
- Concern: punishment should never feel arbitrary — surface the cooldown as a
  friendly countdown, not a 429.

### A2. Level-gated community creation **[user]**
`FORUM_CREATE_COMMUNITY_MIN_LEVEL` (config; default maybe 30). Below it, the
"create community" button is hidden with a "unlocks at level N" hint.
- Why: prevents community sprawl from day-one signups while making leveling a
  *capability* unlock, not just a number. Mirrors the "curator at level 50"
  pattern already in the spec.
- Effort: S (gate + UI hint + config + tests).

### A3. Level-gated community participation **[user]**
Per-community `min_level_to_post` (and optionally `min_level_to_read`). A
community owner sets it at creation; owner + admin can always bypass.
- Why: lets communities self-select their seriousness. A "meta/off-topic"
  lounge can stay chill; a "fics beta readers" board can require demonstrated
  writers. This is the lightest-weight version of private communities.
- Effort: S–M (column on communities, check in write paths, UI field,
  tests). Combine with A4's visibility tiers for the full matrix.

### A4. Visibility/participation matrix (public → invite-only) **[user]**
Every community gets a tier, owner-configurable:
1. **Public** — read + post open.
2. **Level-gated** — read open, post ≥ level N (A3).
3. **Members-only write** — anyone can read, only members post (join is
   open).
4. **Invite-only** — only invited members can read or post (join by invite
   or application, owner approves). This is the **developer board** use case:
   devs talk without noise, because nobody else is even in the room.
5. **Hidden** — invisible in listings, only direct link + membership.
- Why: one clean matrix covers all the "quiet place" instincts (dev board,
   council room, beta-reader circles) without building separate features for
  each.
- Effort: M (columns + join flow + membership checks + UI + tests).

### A5. Community-level moderation (delegated mods)
Community owner can appoint co-moderators (level ≥ `FORUM_CURATOR_LEVEL`,
configurable per community). Their actions land in the **community modlog**
(same modlog machinery, scoped by community) and are metamod-auditable.
- Why: a forum with many communities can't have one admin moderating all of
  them. Delegation + the existing transparency layer (modlog, metamod) keeps
  it honest. Owner remains able to dismiss mods.
- Effort: M (role column on membership, mod endpoints, scoped modlog
  queries).

### A6. Community RSS feeds
`/forum/feed/{community_slug}.xml` — new topics + replies as RSS/Atom, one
feed per community and one global.
- Why: FicHub already does RSS exceptionally well (fics, updates, series);
  this is the same pattern for the forum and costs almost nothing. Power
  users and devs can monitor the boards without visiting.
- Effort: S (reuse the existing RSS builder, add two routes).

### A7. Topic tags + cross-community tag navigation
Topics get freeform tags (curated suggestions + admin merge), and a tag page
shows matching topics **across communities**. Tag synonyms/merges reuse the
existing tag-wrangling admin tooling.
- Why: communities can fragment discussion ("FFN recs" in three boards);
  tags re-join them. This is the search/discovery lever the spec's
  `search_queries` model already points at.
- Effort: M (tag table on topics, filter in search, tag pages).

### A8. Fic-anchored topic cards
In the forum composer, typing `/fic {url_id}` (or a "Link a fic" button)
renders an inline card: cover, title, author, status, word count, rating —
pulled from the works table, updated live. Card is a deep link into the
reader.
- Why: the forum is *for* fanfiction. Making the fic the first-class citizen
  of a post is the single most FicHub-shaped feature on this list — no other
  forum has it.
- Effort: M (markdown extension + card component + endpoint for fic lookup).

### A9. Per-community read-state ("all caught up")
Extend the existing per-topic unread dots to a per-community summary row:
"3 unread topics · last activity 2h ago". One "mark community read" action.
- Why: with multiple communities, the unread model needs an aggregation or
  users drown. Cheap addition to the existing read-state machinery.
- Effort: S–M.

### A10. Community owners see member list + last-active
Owner-visible roster with join date, last post, level, exp. Not public.
- Why: community self-management needs basic sociology — who's active, who
  went quiet, who's sock-puppeting. Owner-level trust boundary, so no
  privacy leak.
- Effort: S (one query + owner-gated UI).

### A11. Posting budget with rollover (soft cap instead of hard cooldown)
Alternative/complement to A1: a daily post budget (e.g. 3 posts/day at lvl 1)
that rolls unused budget forward up to a cap. Hard cooldowns feel like
punishment; budgets feel like allowance.
- Why: same abuse defense as A1 but better UX for a small cohort where
  genuine conversation shouldn't stall.
- Effort: M (budget bookkeeping — a daily counter table or a decrementing
  per-user allowance; needs a janitor job or computed-on-read with a
  windowed COUNT).

---

## Tier B — genuinely good, needs care

### B1. One-shot Q&A topics with accepted answer
Like Fic Requests but general: the opener can mark one reply as "answered"
(pinned under the question). Encourages help threads (beta readers, site
usage, fic-finding) to reach resolution instead of dying open.
- Effort: M (accepted_post_id on topics + UI + notification).

### B2. Community spotlight / weekly digest
Weekly per-community digest: top posts (by score velocity), new members,
new tags. In-app notification + optional RSS. Ties into the existing
"weekly digest" P8#39 idea.
- Effort: M (a weekly aggregation query + a digest page/notification).

### B3. Topic templates
Per-community template picker when creating a topic: "Fic rec",
"Looking for…", "Discussion", "Meta". Templates prefill the composer
(required fields, helpful hints). Reduces low-effort posts and makes
scanning easier.
- Effort: S–M (frontend-only for the picker + per-community template
  strings; no schema change if templates are just prefilled markdown).

### B4. Slow mode per community (owner-enabled)
Owner toggles "slow mode": posts limited to one per N minutes for
non-moderators, surfaced as a visible banner. For spike events (fic
announcements, drama waves) without changing global policy.
- Effort: S (reuse A1's interval machinery scoped per community).

### B5. Community-owner content filters
Owner sets a word/URL regex filter for their community; matching posts go to
that community's mod queue (owner/mods approve). Reuses the F8-ish content
filter idea from THREADLIGHT-FEATURES §2.1#9.
- Effort: M (filter table per community + queue integration).

### B6. Editing window + "edited" stamp
Posts editable for N minutes (config), then locked; edits after that are
logged in the post's history and stamped "edited". Deletes are soft with a
short undo window for the author.
- Why: protects thread coherence (someone editing their posts to win an
  argument later is a known abuse vector) while keeping the forgiving
  author UX.
- Effort: M (edit history table + timestamps + UI).

### B7. Topic follows → updates feed integration
Followed topics already notify in the bell; add an option to also stream
them into the existing "updates" feed alongside fic updates. One feed for
the whole site.
- Effort: S (append forum events to the existing feed query).

### B8. New-member onboarding: suggested communities
On signup (cohort P7#24), a "pick your fandoms" step maps to suggested
communities (from body-cache favorites + existing fandom tags). Lands the
new member in a community, not a void.
- Effort: M (survey → match → subscribe; ties to P7#24 onboarding).

### B9. Community search scope filter
Search UI gains a community dropdown (already possible via the existing
search route — just wire the filter param into the frontend).
- Effort: S.

### B10. Bookmark/save topics
Add topics to a personal reading list (reuse the bookmarks/shelves
machinery). "Save for later" is the single most-requested forum UX.
- Effort: S (join table + button + list page).

### B11. Exp multiplier on mod-approved posts
Positive mods already grant +1 exp; add a configurable bonus for posts that
receive a positive mod *and* the author's level ≥ a threshold (e.g. ×2 at
level 50). Keeps quality signals feeding the same currency.
- Effort: S (config read in the existing award path).

### B12. Level-based flair in communities
Users pick a flair label within a community (owner-curated list); higher
levels unlock more flair options. Lightweight identity, no schema drama.
- Effort: S–M (flair column on membership + owner-managed list).

### B13. Auto-archival of dead communities
A community with zero posts for N days (config) gets flagged "dormant" —
read-only, owner gets a notification. Prevents zombie boards from collecting
spam later.
- Effort: S (janitor check + state flag + UI badge).

### B14. Community transfer/ownership handoff
Owner can transfer ownership to another member (with admin confirmation).
People leave; communities shouldn't die with them.
- Effort: S (one update + confirmation flow).

### B15. Report rate limiting + report status
Limit reports per user per day (config) and let reporters see status
("received → under review → actioned"). Stops report-spam and makes
reporting feel less like a black hole — both cheap.
- Effort: S (existing `user_reports` table + a status column + daily cap).

### B16. Ghost/absorbed accounts — free usernames of inactive users **[user]**
An "anonymous"/"ghost" account collects all activity from accounts that
haven't been used in months (configurable, e.g. 6 months), freeing the
username for a new user.
- How: a janitor job marks an account as dormant after N months of
  inactivity (no logins, no posts). All its content (forum posts, comments,
  bookmarks, reviews) is re-attributed to the single site-wide "ghost"
  account — a real, dedicated user row named e.g. "anonymous" with an
  "absorbed" badge. The username is then released and claimable.
- Why: fanfiction usernames are identity-heavy ("DarkHarry96" isn't
  negotiable); a dead account squatting a name is a real cost when the
  cohort grows. This is the standard forum "sweep inactive accounts"
  mechanic, done carefully: content survives (as ghost), identity is
  recycled, no data is deleted.
- Effort: M (janitor + re-attribution transaction + ghost account row +
  profile "absorbed from X" marker + admin review queue before sweep).
- Care: only sweep accounts with **no** authored content that others
  depend on (reviews/bookmarks are fine to absorb; a long-running fic
  discussion thread's author should be exempt or flagged for admin
  review). Never sweep an admin. Make the threshold config, default
  conservative (12 months), and notify the user before the sweep.

---

## Tier C — interesting, but needs care (abuse surface / scope creep)

### C1. Post attachments (images)
Upload images to posts with content scan + size caps + per-community
opt-out. Storage, moderation, and hotlinking abuse make this heavier than it
looks — defer until there's a real need (fic covers are already handled
elsewhere).
- Effort: L.

### C2. Polls in topics
Numeric or multi-option polls with positive-only result display. Useful for
community votes, but the consensus/roadmap machinery already exists
(MaxDiff/Elo) — a simple poll is a poor man's version. Build only if
communities ask for it.
- Effort: M.

### C3. Threaded replies within posts
True nested threading (currently flat + quote). Long topics become readable,
but the UI cost is real (indent depth, context jumping) and the current
quote-reply pattern is intentionally simple. Revisit at scale, not now.
- Effort: M–L.

### C4. Webhooks for communities (dev board → Telegram/GitHub)
Community-level webhook on new topics/posts. Excellent for the dev board
(devs get GitHub/Telegram noise-free), but that's a niche need — build as a
config-gated feature only after A4's invite-only boards prove useful.
- Effort: M.

### C5. Community quotas (max members, topics/day)
Owner-set caps. Prevents runaway growth, but at cohort scale this is
over-engineering — A1/A2 already damp the spam paths. Only build if a
community actually hits capacity.
- Effort: M.

### C6. Activity heatmap on community list
Sparkline of activity per community (posts/day). Nice glanceable signal,
but it's polish — the unread dots (A9) cover 90% of the need.
- Effort: S.

### C7. Bot-posted series threads
A topic that auto-tracks a fic's updates (bot posts new-chapter notes).
Clever but risks becoming notification spam and needs careful opt-in per
community. Ties to the existing update watcher (P8#6) — build the watcher
first, then decide if forum delivery makes sense.
- Effort: L.

### C8. "Council" appeals panel (level 90+ users review ban appeals)
Democratic appeals for mod actions. The site is small and the operator is
the admin — a jury adds ceremony without adding fairness yet. Revisit when
there's a real user base.
- Effort: L.

### C9. Community shadowban (mute within a community)
Poster's posts go invisible to everyone but themselves within a specific
community. Powerful anti-troll tool, but the "invisible to all but self"
UX is confusing and can be gamed (sock accounts). The existing global
shadowban + A4's invite-only tiers cover most cases.
- Effort: M.

### C10. Seasonal community events (writing challenges, fic bingo)
Gamified events with badges. Fun at real scale, but FicHub deliberately
avoided a credit/quest economy (see D1) — a light event system is a middle
ground worth *designing* for, not building now.
- Effort: L.

---

## Tier D — anti-patterns (deliberately rejected)

### D1. Credit/currency economy (coins, bounties, daily rewards)
Threadlight had a full credit service; the fit analysis explicitly excluded
it (THREADLIGHT-FEATURES §2.2#22–24). FicHub's leveling + exp is
positive-only and non-spendable — that's the point. Adding a spendable
currency invites farming, arbitrage, and a second economy to moderate.

### D2. Public downvotes
The positive-only public surface is a product decision (user preference:
"only positive/constructive shown publicly; no public downvotes"). A
downvote button contradicts it. Moderation points already express
negativity privately, through the metamod-audited path.

### D3. Private messages
Excluded in the Threadlight analysis as a non-goal. The forum's
invite-only communities (A4) give quiet, small-group space without a PM
inbox (which needs its own spam/abuse surface).

### D4. Real-time chat / websockets
At cohort scale, async forums are correct. Live chat is a different product
with its own moderation load; revisit only if the site ever outgrows async.

### D5. Trust-network graphs / affinity scores
Threadlight had stubs (trust.rs 28 lines); fit analysis excluded them.
Leveling + moderation points already encode trust *behaviorally* without
the graph complexity or its privacy surface.

### D6. Circles (invite-only friend groups)
Same exclusion from Threadlight. A4's invite-only communities *are* the
circle feature, with owner governance instead of clique dynamics.

### D7. Community forking
Excluded in the fit analysis. Forking fractures discussion and duplicates
content; the tag system (A7) is the sane way to regroup around a topic.

### D8. Feed plugins
Out of scope. FicHub is a self-contained platform; external feed
integration is what RSS (A6) already provides the open way.

### D9. Public per-user moderation scorecards
Making everyone's moderation history public breeds retaliation and
score-shopping. The modlog is transparent; metamod voters stay anonymous —
that's the balance.

---

## Suggested sequencing

1. **Now (S, mostly config + gates):** A1 (rate limits by level), A2
   (level-gated creation), A3 (level-gated posting), A6 (community RSS),
   A9 (community read-state), B9 (search scope), B10 (save topics),
   B15 (report status/caps).
2. **Next wave (M):** A4 (visibility matrix — the dev board), A5 (delegated
   community mods), A8 (fic cards), B1 (Q&A topics), B6 (edit window),
   B8 (onboarding suggested communities), B16 (ghost accounts — pairs
   naturally with onboarding: free names as the cohort grows).
3. **After cohort feedback:** A7 (tags), A10 (owner roster), B2 (digest),
   B3 (templates), then any C-tier items that real users actually ask for.
4. **Never (until explicitly re-decided):** D-tier.

> The through-line: FicHub's forum should feel like *the place fanfiction
> gets discussed*, not another generic BB. Tier A is the set that makes it
> that — leveling as capability, communities as self-organizing spaces,
> fics as first-class citizens, and RSS/notifications as the connective
> tissue.

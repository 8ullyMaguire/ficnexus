# Forum ↔ NodeBB Parity — Verdict & Plan

> Audit date: 2026-09-07 (revised: trust-system direction). Reference: NodeBB
> v4.15.1 at `~/code/js/NodeBB`. Ficnexus side: `src/routes/forum.rs`
> (66 handlers), 14 frontend pages under `frontend/src/routes/forum/`,
> DB-gated suite `tests/forum_api.rs` (49 tests + 2 trigger-regression
> tests, migration 089). Trust service: `src/services/trust.rs`
> (TL0–TL6, `flag_weight`, auto-promotion TL0→TL4, staff-granted TL5+).

## Verdict: no parity — by design

Ficnexus's forum is not a NodeBB clone. It covers NodeBB's **core discussion
loop** and the platform's trust system already governs who is safe; the
forum adopts that ladder instead of running a parallel moderation economy.
Items marked **divergence** are conscious product decisions, not gaps — do
not "fix" them.

### At parity or better (no action)

Categories CRUD · topics CRUD · replies + quotes · soft delete · topic
follow + reply/mention notifications · read state + unread · recent/popular/
unread views · full-text search with snippets · lock/pin/hide/scoped bans ·
public modlog · basic RSS · invites + registration applications · user blocks
· stable slug URLs · topic tags (text shape, migration 082).

### Deliberate divergences (not gaps)

- **Trust levels instead of Slashdot points + metamoderation** (see below).
- Site-wide levels (0–100) for coarse gates; trust (TL0–TL6) for safety.
- Edit proposals with immutable history instead of silent author edits.
- Positive-only signal (reactions; **no downvotes** — trust-weighted flags
  already cover "this is bad").
- Invites/applications instead of a registration queue.
- No built-in chat (separate surface), no polls (plugin territory in NodeBB
  too).

## Direction: trust runs forum moderation

The platform trust system (`src/services/trust.rs`) answers *who is safe*:
earned from reading/participation, auto-promoted TL0→TL4
(`works_read`/`words_read`/`days_active`/`forum_posts` thresholds in
`meets_level`), staff-granted TL5+ via the weekly digest. It already gates
publishing (TL2+) and report resolution (TL5+), scales flag weight by level
(TL1=1 … TL5=5, TL0 may not flag), and already counts `forum_posts` in its
metrics. The forum uses this ladder — no parallel grant/points economy.

### Mapping (replaces the old points/metamod design)

- **Who moderates → trust level.** TL4+ (Elder) gets queue access; TL5+
  (Community Moderator) resolves reports, fast-hides, locks/pins; TL6/admin
  bans. Replaces `FORUM_MOD_MIN_LEVEL`-style gates and points-grant
  eligibility with the one ladder users already see at `/settings/trust`.
- **Report triage → trust-weighted flags (already built).** `flag_weight` +
  auto-triage (`pending` / `needs_admin` / `auto_hidden`) sorts the forum
  mod queue — no separate points spend.
- **Rate limiting → thin daily cap.** A per-user daily moderation-action cap
  (anti-abuse) replaces the earned/spendable points currency. No grants, no
  refill windows.
- **Metamoderation → contested-resolution escalation.** A `needs_admin`
  report where the reporter disputes a resolution escalates to admins. The
  anonymized-audit-vote-cooldown apparatus policed a points economy that no
  longer exists — drop it.
- **What ports over unchanged (workflow, not identity):** queue UI, reason
  picker with score deltas, public mod history, curator fast-hide with 72h
  expiry, scoped bans, `needs_admin` escalation, modlog writes.

### Migration notes (old → new)

- `forum_mod_grants` / points-window tables: stop writing; read paths fall
  back to trust checks. Remove in a later cleanup migration once no code
  references them.
- `FORUM_POINTS_*` / `FORUM_META_*` env vars: superseded by trust thresholds
  + one `FORUM_MOD_ACTIONS_PER_DAY` cap. Keep parsing (ignore) for one
  release so old `.env` files don't break boot, then delete.
- `tests/forum_api.rs` `f5_*`/`f6_*` tests: rewrite against trust gates
  (TL4 queue access, TL5 resolve) rather than points balances.
- `docs/src/forum.md` "Moderation points & metamoderation" section: rewrite
  to trust tiers (already flagged; update when the code lands).

## Gaps worth building (ranked)

### P1 — breaks real moderator workflows

| # | Gap | NodeBB reference | Ficnexus starting point |
|---|-----|------------------|-------------------------|
| 1 | **Move topic between categories** | `topicTools.move` (`src/topics/tools.js:239`) | `update_topic` PATCH exists; needs `category_slug` in body + TL5 check |
| 2 | **Post queue (pre-moderation)** | `Posts.shouldQueue/addToQueue/submitFromQueue` (`src/posts/queue.js`), gates `postQueue`, `postQueueReputationThreshold`, `newbiePostDelay` | `forum_edit_proposals` table is the natural queue store; TL0 posts held for TL5 review |
| 3 | **Flood control + length floors** | `postDelay=10`, `newbiePostDelay=120`, `minimumPostLength=8`, `maximumTitleLength=255` (`install/data/defaults.json`) | No timing checks today; add `FORUM_POST_DELAY_SECS`, `FORUM_MIN_POST_LEN` (config-gated) |
| 4 | **Restore soft-deleted posts/topics** | `topicTools.restore`, `Diffs.restore` | `deleted_at` columns exist; needs undelete path + TL5 gate |
| 5 | **Trust-gate the mod surface** | — (new work) | TL4 → queue access; TL5 → resolve/fast-hide/lock/pin; TL6 → bans. Daily action cap. Retire points/metamod tables. |

### P2 — expected forum behavior, users will ask

| # | Gap | NodeBB reference | Notes |
|---|-----|------------------|-------|
| 6 | **Merge topics / fork posts to new topic** | `Topics.merge`, `Topics.createTopicFromPosts` (`src/topics/merge.js`, `fork.js`) | Rare ops; TL5-only v1 acceptable |
| 7 | **Category watch (notify on new topic)** | `Categories.watch`, `Topics.notifyFollowers` | Small: `forum_category_follows` table + hook in `create_topic` |
| 8 | **Topic ignore** | `Topics.ignore/isIgnoring` (`src/topics/follow.js`) | Small: reuse follows table with state column |
| 9 | **Tag autocomplete + tag search + tag pages** | `Topics.autocompleteTags/searchTags/getTagData` (`src/topics/tags.js:451+`) | `forum_topic_tags` is text-shape; autocomplete is a `SELECT DISTINCT … ILIKE` away |
| 10 | **Per-topic/per-category/recent-posts RSS** | `src/routes/feeds.js` (topic/category/topics/recent/recentposts feeds) | `forum_rss` today is one global feed with recent/popular modes |
| 11 | **Post history diff view** | `Diffs.list/get` (`src/posts/diffs.js`), `enablePostHistory=1` | Edit snapshots exist (`forum_edit_proposals`); needs a diff render |
| 12 | **Raw post view** | `SocketPosts.getRawPost` | One small read endpoint |

### P3 — nice, not load-bearing

| # | Gap | NodeBB reference |
|---|-----|------------------|
| 13 | Pin expiry (`setPinExpiry`, `checkPinExpiry`) | `src/topics/tools.js:121` |
| 14 | Scheduled topics | `src/topics/scheduled.js` |
| 15 | Per-post-index topic bookmarks (resume position) | `Topics.setUserBookmark` (`src/topics/bookmarks.js`) |
| 16 | Signatures | `maximumSignatureLength=255`, `min:rep:signature` |
| 17 | Online indicator / last-seen | `User.isOnline/updateLastOnlineTime` (`src/user/online.js`) |
| 18 | Per-user listings (posts/topics/best/upvoted…) | `controllers.accounts.posts.*` (`src/routes/user.js:28-42`) |
| 19 | GDPR export including forum data | `exportUsersCSV`; ficnexus `user_export.rs` anonymizes but keeps posts |
| 20 | Composer drafts/autosave/preview + image uploads | `controllers.composer.js`, `uploadsController.uploadPost` |
| 21 | Digest emails | `src/user/digest.js`, `dailyDigestFreq` |
| 22 | Forum URLs in sitemap | `src/sitemap.js` (`sitemapTopics=500`) |
| 23 | Full flag lifecycle (states/notes/history) | `src/flags.js` — trust-weighted triage covers the need today |
| 24 | Purge (hard delete) | `topicTools.purge`, `Flags.purge` — needs retention policy first |
| 25 | Realtime (sockets for live replies/presence) | `src/socket.io/*` — architectural decision, not a milestone |

## Suggested milestone order

- **M1 (moderator unblock):** P1 items 1–5. Each is small and DB-gated-testable
  in `tests/forum_api.rs` following the existing `f5_*` patterns (rewritten
  for trust gates).
- **M2 (community expectations):** P2 items 6–12.
- **M3 (polish):** P3 as drive-bys; 25 only if realtime ever becomes a goal.

## Regression guard (shipped 2026-09-07)

Migration 089 dropped a stale pair of `forum_topics` triggers whose function
body referenced the pre-082 `forum_topic_tags.tag_id` column — every topic
UPDATE (including the `view_count + 1` read bump) 500'd. Pinned by
`regression_no_stale_forum_topic_triggers` (no non-FK triggers on
`forum_topics`, no orphan function) and
`regression_topic_update_survives_view_count_bump` (UPDATE + handler read
returns `err:0`) in `tests/forum_api.rs`. Lesson for future migrations that
reshape a table: grep `pg_trigger`/`pg_proc` for functions referencing the
old columns — sqlx migrations don't track DB functions.

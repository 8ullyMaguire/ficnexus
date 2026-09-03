# SPEC: Community Platform v2 — Slashdot-Style Moderation Points + Metamoderation + Leveling

> **Status:** Draft for discussion (v2) · **Owner:** maintainer · **Area:** community platform
> **Supersedes:** v1 forum spec (jury/constitution machinery). This replaces the jury/constitution
> machinery entirely. v2 also introduces a **site-wide leveling system** (0-100 levels + exp)
> that replaces the 0/1/5/10 role ladder and reputation — applies to **both FicHub and the forum**.

---

## 0. Framing

Slashdot's system isn't democratic — it's a **meritocratic lottery with statistical audit**.
Eligible users get a small, expiring budget of moderation points; readers self-filter by score
threshold; a random sample of every mod action gets re-judged by veteran users, and
systematically unfair moderators lose eligibility. No deliberation, no quorum, no campaigning,
no latency problem — it's mechanical. For a self-hosted forum, that's a strictly better fit than
juries.

---

## 1. Concept mapping: Slashdot → FicHub

| Slashdot concept | FicHub implementation |
|---|---|
| Karma | **NEW: `users.exp` (replaces `users.reputation`) + `users.level` (0-100, replaces `users.role`)** + role-free gates. See §10. |
| Moderator eligibility | `level ≥ FORUM_MOD_MIN_LEVEL` (config, default 1), account age ≥ 14d, `exp ≥ FORUM_MOD_MIN_EXP` (default 20), not in a mod cooldown |
| 5 points / 3 days, expiring | `forum_mod_grants` — 5 points per 72h window, lazy-granted on first queue fetch, unused points expire |
| Comment score −1…+5 | `forum_posts.score`, starting 0 (+1 base if author `exp ≥ 100`) |
| Moderation reasons | Fixed enum with **fixed deltas** (removes one abuse axis vs. Slashdot's free score+reason) |
| Reader threshold | Per-user setting (localStorage, like `prefs.ts`); posts below threshold render collapsed with a "score X — show" affordance. **Moderation filters; it never deletes** |
| Metamoderation | Random sample of mod actions, moderator identity anonymized, rated fair / unfair / unsure by veterans; rolling audit per moderator |
| Public mod log | FicHub's existing `modlog` (migration 034) — every mod action logged, readable by any logged-in user. Transparency is inherited for free |

---

## 2. Governance layers

**Layer 0 — Floor (operator, unchanged).** Illegal content, doxxing, spam, ban evasion: instant
removal by operator/curator, no points spent, logged to modlog. Anti-bot stays infrastructure
(honeypot, tiered limits, shadowban, bot-scorer flags). A role ≥5 / level ≥ curator **fast-hide**
(72h, auto-expires) covers urgent-but-contested content while points trickle in.

**Scoped enforcement (board/forum-level, NOT account-level).** Bans and timeouts are scoped —
they never touch the account itself:
- **Scope levels:** `forum` (whole forum — can't post/reply anywhere), `category` (a single
  board/category), or `post`-level (fast-hide).
- **Timeout vs ban:** a timeout is a *bounded* scope restriction (`expires_at` set); a ban is
  an *unbounded* one (`expires_at` NULL) that can be lifted by a mod. Both are `forum_bans`
  rows distinguished by `expires_at`.
- A user banned from the forum is **still a normal FicHub account** — they can read/download/
  rate/review fics, use the library, etc. Only the forum scope is blocked. No account ban,
  no shadowban, no modlog noise on the account.
- **Enforcement:** write routes (topic/post create, reply, vote/mod) check the ban scope:
  `forum` scope blocks all; `category` scope blocks that category only. Read is never blocked
  (consistent with "moderation filters, it never deletes").
- **Modlog:** every scope restriction is logged (`forum_ban`, `forum_timeout` with scope +
  expiry + reason), visible on `/modlog`. Metamod audits scoped bans the same as point mods.
- **Moderator scope:** a moderator can only apply scope restrictions up to their own level
  (a category mod can't forum-ban; only level ≥ curator can forum-ban; level ≥ admin can
  ban from the account level if ever needed).

**Layer 1 — Moderation points (the daily driver).**
- Eligible users get 5 points per 72h window. One mod per post per user, never on your own post,
  hard cap of 5 mods per post (anti-pile-on at cohort scale).
- Reasons (fixed deltas):

| Positive | Δ | Negative | Δ |
|---|---|---|---|
| Insightful | +2 | Off-Topic | −1 |
| Informative | +1 | Redundant | −1 |
| Interesting | +1 | Flamebait | −2 |
| Funny | +1 | Troll | −2 |
| | | Abusive | −3 |

- `Abusive` (or score ≤ −2) auto-collapses the post for **all** thresholds — but still doesn't
  delete. Metamod reversal restores it. Deletion remains an operator floor action.
- **Reports become a queue signal, not a case flow.** `user_reports` (CHECK ALTER: add
  `'forum_post'`, `'forum_topic'` to the current `('comment','work','user')`) boosts a post's
  position in the mod queue. The points economy decides; there's no report→verdict pipeline to
  administer.
- No notifications on moderation (avoids down-mod drama); the score and the public modlog tell
  the story.

**Layer 2 — Metamoderation (the accountability layer).**
- Eligibility is a higher bar: `level ≥ FORUM_META_MIN_LEVEL` (default 1), age ≥ 90d,
  `exp ≥ FORUM_META_MIN_EXP` (default 50), ≥10 forum posts.
- Each mod action is sampled for **3 ratings**. Metamoderators see the post excerpt + the reason
  used, **never the moderator's identity** (prevents bias; audit aggregates are per-moderator only).
- Verdicts: fair / unfair / unsure.
- Audit: rolling window over the last 30 metamod ratings of a moderator's actions. ≥10 rated and
  unfair-rate > 30% → 30-day mod cooldown (grant revoked). The suspension is logged to modlog
  (`mod_privileges_suspended`, aggregate only — no metamod voter identities leak).
- Symmetrically: high fair-rate moderators get nothing but the continued privilege. **No
  reputation reward for moderating or metamoderating** — FicHub is deliberately not gamified, and
  paying for mod actions is exactly how you get mod-farming.

**Layer 3 — Reader thresholds.** Default threshold 0; user-adjustable −1…5. This is the part of
Slashdot that most reduces operator workload: content-quality disputes become personal filter
settings instead of moderation tickets.

---

## 3. Data model (migrations 041–047; 047 is the current high-water mark)

```
041_forum_core
  forum_categories (id, slug, name, description, position, is_archived)      -- level ≥ admin creates
  forum_topics     (id BIGSERIAL, category_id, author_id INT4→users, title,
                    status open|locked|pinned|removed, last_post_at)
  forum_posts      (id BIGSERIAL, topic_id, author_id INT4, body_md,
                    quoted_post_id, score INT DEFAULT 0, mod_count INT DEFAULT 0,
                    hidden_until TIMESTAMPTZ, created/updated/deleted_at)
  forum_follows    (user_id, topic_id, PK)
  forum_read_state (user_id, topic_id, last_read_post_id, PK)
  + tsvector column on forum_posts.body_md + GIN (body_text_search pattern, mig 040)

042_forum_moderation
  forum_mod_grants (user_id PK, points_left SMALLINT, granted_at, expires_at)
  forum_mod_actions(id BIGSERIAL, post_id, moderator_id, reason TEXT,
                    delta SMALLINT, score_after INT, created_at)
  forum_metamod_votes(mod_action_id, voter_id, verdict SMALLINT, PK(action,voter))
  ALTER user_reports: extend CHECK to include forum_post/forum_topic

043_leveling (site-wide — FicHub AND forum)
  ALTER users:
    ADD COLUMN level SMALLINT NOT NULL DEFAULT 0 CHECK (level BETWEEN 0 AND 100)
    ADD COLUMN exp BIGINT NOT NULL DEFAULT 0
    -- reputation stays (legacy read path) or is dropped after backfill;
    -- role stays (legacy read path) or is dropped after backfill
  Backfill: level = mapped from role for existing privileged users (see §10);
            exp = reputation for existing users.

044_forum_metamoderation / 045_leveling-gates / 046_subsystems  (see those migration files)

047_forum_topic_slug
  ALTER forum_topics: ADD COLUMN topic_slug TEXT NULL
  CREATE UNIQUE INDEX idx_forum_topics_slug ON forum_topics (topic_slug)
    WHERE topic_slug IS NOT NULL
  Canonical format `{slugified-title}-{id}`; generated at create; immutable.
  Legacy rows backfilled at server startup (backfill_topic_slugs, best-effort).
```

Everything else reuses existing FicHub machinery: notifications (free-text types — no
migration), modlog, and the new leveling system (`exp_events` reusing the `reputation_events`
pattern with a new event type `forum_post_moderated_up`, +1 exp to author on positive mods,
capped +3/day; negative mods recorded with delta 0, matching the positive-only public
philosophy).

---

## 4. API surface

Categories/topics/posts CRUD, follows, read state, `GET /api/forum/search?q=` (from v1). New:

```
GET  /api/forum/moderation/status            my points, window expiry, eligibility
GET  /api/forum/topics/by-slug/{topicSlug}   topic detail resolved by slug (same shape as {topicId})
GET  /api/forum/moderation/queue             mods-needed posts (reported + low-score first;
                                             excludes own/already-moded; requires points)
POST /api/forum/posts/{post_id}/moderate     {reason} — server maps reason→delta, spends 1 pt
GET  /api/forum/posts/{post_id}/moderations  public mod history (mirrors modlog)
GET  /api/forum/metamod/queue                N random under-rated actions, moderator anonymized
POST /api/forum/metamod/{action_id}/vote     {verdict: fair|unfair|unsure}
POST /api/admin/forum/hide/{post_id}         level ≥ curator fast-hide, 72h auto-expiry (floor path)
POST /api/admin/forum/topics/{id}/lock|pin   level ≥ curator; categories level ≥ admin
```

All auth-gated routes follow the 400-as-401 convention; level gates return 400 with
`{"err":403}`. Keep `{post_id}` / `{action_id}` param names consistent per path level (axum 0.8
forbids mixed names at one level).

---

## 5. Frontend

- `/forum/c/[slug]`, `/forum/t/[id]` — topic view: posts chronological, score badge,
  below-threshold collapsed ("score −1 — show"), mod button (opens reason picker, only when
  points > 0), report button for everyone.
- `/forum/moderate` — queue page with excerpts and reason buttons.
- `/forum/metamod` — audit cards: post excerpt, reason used, fair/unfair/unsure.
- Settings: threshold slider (−1…5), localStorage like reader prefs.
- `/forum/rules` — a **static** rules page (a `docs/src/` chapter + one route). The reasons enum
  *is* the operationalized constitution; a static page is zero machinery.
- Hard requirements: `/forum` prefixes added to `routePages` in `+layout.svelte` (or the SPA
  silently renders the home dashboard); test files named `page.test.ts`, never `+page.test.ts`.

---

## 6. Anti-gaming

- No self-mods, one mod per post per user, 5-mod cap per post, points expire (no hoarding).
- Moderator identity hidden from metamoderators; metamod voters never exposed.
- Alt rings: flagged via the existing bot-scorer `client_id` clustering in the admin view —
  flag, never auto-block.
- Exp impact is asymmetric (positive mods only) so down-mod brigades can't farm anything.
- Mods/meta-mods gated on account age + level; fresh accounts can post and report but not judge.

---

## 7. Cold start (the part Slashdot never had to solve)

Config-gated, checked dynamically:
- Eligible-moderator pool < 8 → moderation points dormant; level ≥ curator moderators directly
  (unlimited, all actions still logged and metamod-auditable later).
- Metamod pool < 8 → metamod dormant; the **public modlog is the audit** (skimmed weekly — it's
  already readable by any logged-in user, so the cohort can too).
- When pools cross thresholds, layers switch on automatically. No migration, no ceremony — a
  config read against a COUNT.

At 10–20 cohort users, metamoderation degenerates (everyone ends up auditing their own friends).
The min-pool gates are what keep the system honest instead of theatrical.

---

## 8. Milestones

| # | Unit | Migration |
|---|---|---|
| 1 | Crate skeleton + categories/topics read | 041 |
| 2 | Posts + follows + notifications (**no ±1 votes — mod points are the only score system**) | 041 |
| 3 | Read state + search | 041 |
| 4 | **Moderation points**: grants, reasons, queue, thresholds, reports-as-signal, curator fast-hide | 042 |
| 5 | **Metamoderation**: queue, anonymized audit, rolling unfair-rate, cooldowns | 042 |
| 6 | Invites + registration applications (moved up — entry curation is the highest-leverage moderation) | 043+ |
| 7 | Blocks, site info | — |

Each remains one worktree + one subagent, DB-gated `tests/forum_api.rs` with its own mutex
(`--test-threads=1`). Make every tuning constant config-driven (`FORUM_MOD_POOL_MIN`,
`FORUM_POINTS_PER_WINDOW`, `FORUM_WINDOW_HOURS`, `FORUM_META_RATINGS`, `FORUM_UNFAIR_RATE`,
`FORUM_AUDIT_WINDOW`, plus the leveling thresholds in §10) so tests can shrink the windows and
the live system can be retuned without redeploying logic.

---

## 9. The honest caveat

Slashdot moderation has one well-documented failure mode: **the hivemind**. Moderate-by-consensus
drifts toward conformity — mainstream takes get modded Insightful, dissenting ones get modded
Off-Topic, and over time the forum calcifies into its majority's opinions. Metamoderation polices
*process* abuse (unfair targeting, troll-flagging the innocent) but not *taste* conformity;
nothing mechanical can. At cohort scale this is muted (everyone knows everyone), but expect it
the moment the forum has factions. Defenses already in this spec: the fixed reasons list forces
mods to be *categorical* rather than just negative; thresholds mean down-modded content is
filtered rather than destroyed; the floor means the operator — not the mob — owns the hard calls.

If that drift ever becomes the dominant complaint, the fix isn't more machinery — it's the invite
queue. Admit people who argue well. That's the lever milestone 6 protects.

---

## 10. Leveling system (site-wide — applies to FicHub AND forum)

**Replaces the 0/1/5/10 role ladder with a 0-100 level + exp progression.** This is a FicHub-wide
refactor, not forum-only: every existing role gate (authors.rs, admin.rs, server.rs, frontend
auth store, admin nav visibility) is swept to level gates, and reputation becomes exp.

### 10.1 Design

- `users.level SMALLINT 0-100` (default 0) — the progression ladder.
- `users.exp BIGINT` (default 0) — the XP currency. `users.reputation` is retired (backfilled
  into `exp` on migration 043; the column stays for legacy read paths until swept).
- `users.role` is retired after backfill: privileged users get a mapped level
  (curator 5 → level 50, senior_curator 5 → 60, admin 10 → 100) so nothing regresses.
- **Level gates replace role gates everywhere:**
  - Curator privileges: `level ≥ FORUM_CURATOR_LEVEL` (config, default 50)
  - Admin privileges: `level ≥ FORUM_ADMIN_LEVEL` (config, default 100)
  - All existing `role >= 5` / `role >= 10` checks become level comparisons.
- **Exp sources (positive-only public philosophy):**
  - Forum post created: +2 exp
  - Positive mod received (Insightful/Informative/Interesting/Funny): +1 exp, capped +3/day
  - Negative mods: recorded with delta 0 (no exp loss — matches the site's positive-only surface)
  - (FicHub-side sources can be added later: reviews, downloads, etc. — out of scope for this milestone.)
- **Exp audit trail:** reuse the `reputation_events` table pattern (migration 001) as
  `exp_events` (or extend reputation_events with a new event_type) so level-ups are explainable.

### 10.2 Forum eligibility (config-driven, defaults shown)

| Role (old) | Level (new) | Mod eligibility | Metamod eligibility |
|---|---|---|---|
| reader (0) | 0-49 | — | — |
| curator (5) | ≥ 50 | ✓ (age ≥14d, exp ≥ 20) | ✓ (age ≥ 90d, exp ≥ 50, ≥10 posts) |
| admin (10) | 100 | ✓ | ✓ |

All thresholds are config: `FORUM_MOD_MIN_LEVEL`, `FORUM_MOD_MIN_EXP`, `FORUM_MOD_MIN_AGE_DAYS`,
`FORUM_META_MIN_LEVEL`, `FORUM_META_MIN_EXP`, `FORUM_META_MIN_AGE_DAYS`, `FORUM_META_MIN_POSTS`.

### 10.3 What changes in the codebase

- `migrations/043_leveling.sql` — ALTER users, backfill, exp_events.
- `AuthUser` carries `level` + `exp` (replacing `role` in the extractor).
- Sweep all `role >= N` checks in `src/` to `level >= FORUM_CURATOR_LEVEL` / `>= FORUM_ADMIN_LEVEL`
  (or a small `level_gate()` helper).
- Frontend: auth store exposes `level`/`exp`; admin nav visibility uses level gates.
- Tests: every suite referencing `role` fixtures updates to `level` fixtures.

### 10.4 Why this fits

FicHub is a fanfiction community; progression is engaging and matches the existing
badges/quests/leaderboard systems. Moderation privilege now **derives from demonstrated
participation** (the leveling system) rather than a static admin-assigned role — which is exactly
the trust model Slashdot-style moderation needs. Admin (level 100) remains the operator floor;
levels 50-99 are the trusted middle that curates and moderates.

---

## 11. Open questions (with recommended answers)

1. **`role` column**: drop after backfill vs keep for operator convenience? Recommend **keep as
   legacy read-only** for one release, then drop — avoids breaking anything while the sweep lands.
2. **Admin = level 100 only?** Recommend yes — level 100 is the admin tier; a separate
   `is_operator` flag is NOT needed at cohort scale (the site owner is the only one who can
   promote to 100).
3. **Exp sources beyond forum?** Recommend forum-only for now (posts + positive mods); extend
   later (reviews, downloads) without schema change (exp_events is generic).
4. **Level-up UX**: recommend a notification (`level_up` type) + small badge on profile; no
   fanfare beyond that.
5. **Threshold default**: recommend 0 (show everything, collapse only score ≤ −2 auto-collapse);
   power users can raise it.
6. **Points window**: 72h / 5 points per spec. Recommend keeping 72h (Slashdot's cadence) but
   make it config (`FORUM_WINDOW_HOURS`, `FORUM_POINTS_PER_WINDOW`) so it can be tightened at
   cohort scale.
7. **Metamod sample size**: 3 ratings per action per spec. Recommend config
   (`FORUM_META_RATINGS`) so a busier forum can raise it to 5.

---

## 12. Acceptance criteria (M1 done — milestones 1-3)

- [ ] `forum-core` crate builds standalone; unit tests green; zero FicHub types.
- [ ] Migration 041 applied; forum tables + tsvector GIN + `user_reports` CHECK extension verified.
- [ ] `/api/forum/categories|topics|posts|follows|read` + `/api/forum/search` all auth-gated;
      `qa/api-walk.js` passes for them.
- [ ] `tests/forum_api.rs` green (`--test-threads=1`): happy paths + level gates + rate limits.
- [ ] `/forum` UI: categories, topic list (unread dots), topic view (posts chronological, score
      badge, reply composer, follow, report), `/forum/search`.
- [ ] Notifications: `forum_reply` + `forum_mention` in the bell.
- [ ] Moderation points (M4) + metamoderation (M5) machinery present per §2-4 (acceptance for
      those milestones checked separately).
- [ ] Leveling (migration 043): `users.level`/`users.exp` live; existing role gates swept to
      level gates; admin nav + AuthUser carry level/exp; exp_events recorded.
- [ ] Frontend vitest (`page.test.ts` for forum routes + store tests) and Playwright e2e green;
      coverage gates hold.
- [ ] `+layout.svelte` `routePages` includes `/forum` prefixes; i18n strings added.
- [ ] `cargo build --release` + `systemctl restart fichub` + `node qa/run.js` + `hermes verify
      --save` green on ThinkCentre; live `/forum` reachable.
- [ ] Docs: `docs/src/forum.md` + `docs/src/rules.md` + SUMMARY entry; roadmap marked; cohort
      invite includes `/forum`.

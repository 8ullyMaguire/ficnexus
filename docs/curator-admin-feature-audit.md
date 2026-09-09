# FicNexus — Curator & Admin Feature Audit

> **Scope:** Every feature accessible to users with elevated privileges (curators, moderators, admins, trust level ≥ 2).
> **Role hierarchy:** User (TL0–TL2) → Curator (TL3–TL4) → Moderator (TL5) → Admin (role ≥ 10) → Owner (role ≥ 100).
> **Last verified:** 2026-09-09 against `src/routes/`, `src/services/`, and `migrations/`.

---

## 1. Trust & Reputation System

### 1.1 Trust Levels (TL0–TL6)

| Level | Name | Requirement | Abilities unlocked |
|-------|------|-------------|-------------------|
| TL0 | New User | Sign up | Read, bookmark |
| TL1 | Reader | 5+ works, 30k+ words | — |
| TL2 | Active Reader | 5+ days, 100k+ words, 1 forum post | Publish own works |
| TL3 | Contributor | 15+ days, 100+ works, 10+ posts | Curator queue access |
| TL4 | Regular | 400k+ words, 500+ works, 50+ posts | — |
| TL5 | Community Moderator | Admin-granted via digest | Resolve reports, metamod |
| TL6 | Staff | Admin-granted | Full moderation |

**Endpoints:**
- `GET /api/me/trust` — own level, metrics, next-level hints
- `GET /api/admin/trust` — distribution + per-user snapshot (admin only)
- `PUT /api/admin/trust/{id}` — set trust level (admin only)
- `GET /api/admin/digest` — weekly moderation digest (admin only)

**Pros:**
- Transparent progression — users see exactly what's needed to level up
- TL5 is earned via admin digest, not auto-promoted — prevents privilege escalation by gaming metrics
- `flag_weight(level)` gives higher-trust users' flags more weight in auto-moderation
- `assert_min_trust()` / `assert_staff_or_min_trust()` are reusable gates

**Cons:**
- TL3–TL4 gap is steep (50 posts, 500 works) — may discourage curation participation
- No automated TL3→TL4 promotion — relies on admin digest which is weekly
- `next_level()` hints are hardcoded strings, not localized
- `RESOLVE_MIN_TRUST = 5` means only community moderators+ can resolve reports — creates bottleneck

**Verdict:** Solid foundation. Consider auto-promoting TL3→TL4 (no admin bottleneck) and lowering `RESOLVE_MIN_TRUST` to 4 to reduce moderator load.

---

### 1.2 Reputation (per-work)

- Separate from trust — earned via kudos, bookmarks, reviews
- Displayed on user profile
- No admin endpoints — purely emergent

**Pros:** Decoupled from trust, so a user can be trusted but not "famous" and vice versa.

**Cons:** No decay — old reputation persists forever. No way to reset for bad behavior.

**Verdict:** Add optional reputation decay or admin reset for banned users.

---

## 2. Forum Moderation

### 2.1 Post/Reply Moderation

**Endpoints:**
- `GET /api/forum/moderation/queue` — posts flagged for review
- `POST /api/forum/moderation/{postId}` — approve/reject/edit
- `GET /api/forum/moderation/{postId}` — moderation history
- `POST /api/forum/posts/{postId}/hide` — admin hide (soft delete)
- `DELETE /api/forum/posts/{postId}` — admin hard delete

**Mechanics:**
- `require_mod()` gate (TL5+)
- `already_modded()` prevents double-moderation
- `daily_mod_usage()` tracks mod actions for rate limiting
- `mod_delta()` adjusts trust on positive/negative mod actions
- `ON CONFLICT DO NOTHING` on re-moderation attempts

**Pros:**
- Idempotent — safe to retry
- Audit trail via `mod_log`
- Trust impact on mod actions incentivizes careful moderation

**Cons:**
- No batch operations — moderators must act on posts one-by-one
- No "escalate to admin" path for edge cases
- `daily_mod_usage` cap is hardcoded, not configurable

**Verdict:** Add batch moderation (select N posts → approve/reject) and configurable daily caps.

---

### 2.2 Topic Management

**Endpoints:**
- `POST /api/forum/topics/{topicId}/lock` — lock/unlock
- `POST /api/forum/topics/{topicId}/pin` — pin/unpin
- `DELETE /api/forum/topics/{topicId}` — delete topic

**Pros:** Standard forum operations, well-gated.

**Cons:** No "move topic to category" — moderators can't reorganize.

**Verdict:** Add `POST /api/forum/topics/{topicId}/move` for category changes.

---

### 2.3 User Bans

**Endpoints:**
- `POST /api/forum/users/{userId}/ban` — ban from forum
- `DELETE /api/forum/users/{userId}/ban` — unban
- `GET /api/forum/users/{userId}/bans` — list bans

**Mechanics:**
- `is_banned()` checked on every forum write
- Bans are per-user, not per-IP
- No ban duration — permanent until manually lifted

**Pros:** Simple, effective.

**Cons:**
- No temporary bans (1 day, 1 week, etc.)
- No ban reason visible to the banned user
- No appeal mechanism
- Per-user only — no IP range bans for ban evasion

**Verdict:** Add temporary bans, ban reasons, and appeal workflow.

---

### 2.4 Meta-Moderation (TL5+)

**Endpoints:**
- `GET /api/forum/metamod/queue` — moderator actions to review
- `POST /api/forum/metamod/{modlogId}` — approve/reject a mod action
- `GET /api/forum/metamod/grants` — view moderator grants

**Pros:** Checks-and-balances on moderator power. Prevents rogue moderators.

**Cons:** Only TL5+ can metamod — if all TL5s are the same person, no oversight.
No notification to the original moderator when their action is overturned.

**Verdict:** Add notifications for overturned actions. Consider allowing admins to trigger metamod review.

---

## 3. Admin Tools

### 3.1 User Management

**Endpoints:**
- `GET /api/admin/users` — list users (paginated, filterable)
- `PUT /api/admin/users/{id}/role` — set role (admin/moderator/user)
- `POST /api/admin/users/{id}/ban` — site-wide ban
- `GET /api/admin/users/{id}/trust` — trust details

**Role system:**
- `role < 10` = user/curator
- `role >= 10` = admin
- `role >= 100` = owner

**Pros:** Simple numeric hierarchy.

**Cons:**
- No role inheritance — a moderator (TL5) with `role=0` can't access admin endpoints
- No audit log for role changes
- No "demote to user" confirmation — one click and they're done

**Verdict:** Add audit log for role changes and confirmation dialogs for demotions.

---

### 3.2 Content Blacklist

**Endpoints:**
- `POST /api/admin/blacklist/fic` — blacklist a work
- `POST /api/admin/blacklist/author` — blacklist all works by author
- `GET /api/admin/blacklist` — list blacklisted items

**Pros:** Quick way to remove problematic content.

**Cons:**
- No expiration — blacklisted forever unless manually removed
- No reason field — future admins won't know why something was blacklisted
- No notification to the work's author

**Verdict:** Add expiration, reason field, and author notification.

---

### 3.3 Content Scanning

**Endpoints:**
- `GET /api/admin/content-scan` — list scan results
- `POST /api/admin/content-scan/run` — trigger a scan
- `POST /api/admin/content-scan/{id}/review` — approve/reject scan finding

**Pros:** Automated detection of problematic content.

**Cons:** No details on what the scan checks (plagiarism? CSAM? spam?)
No integration with external hash-matching databases (e.g., PhotoDNA).

**Verdict:** Document what the scan checks. Consider external hash-matching integration.

---

### 3.4 Translation Management

**Endpoints:**
- `GET /api/admin/translations` — list pending translations
- `POST /api/admin/translations/{id}/approve` — approve
- `POST /api/admin/translations/{id}/reject` — reject
- `PUT /api/admin/translations/{id}` — edit translation

**Pros:** Quality control on community translations.

**Cons:** No versioning — once approved, no history of changes.
No "suggest edit" for already-approved translations.

**Verdict:** Add translation versioning and edit suggestions.

---

### 3.5 Bot Management

**Endpoints:**
- `GET /api/admin/bots` — list detected bots
- `POST /api/admin/bots/{id}/shadowban` — shadowban (content hidden from others)
- `DELETE /api/admin/bots/{id}/shadowban` — unshadowban

**Pros:** Shadowban is a gentle alternative to full ban — bot doesn't know it's banned.

**Cons:**
- Bot detection is not documented — unclear what triggers it
- No appeal path for false positives
- No rate limiting on bot detection (could accidentally flag power users)

**Verdict:** Document bot detection criteria. Add appeal path.

---

### 3.6 Search Analytics

**Endpoints:**
- `GET /api/admin/search-analytics` — popular searches, zero-result queries

**Pros:** Identifies content gaps and trending topics.

**Cons:** No date range filter — shows all-time only.
No export (CSV/JSON) for external analysis.

**Verdict:** Add date range filters and CSV export.

---

### 3.7 Realtime Dashboard

**Endpoints:**
- `GET /api/admin/realtime` — active WS/SSE connections, channel subscriptions

**Pros:** Live view of system load.

**Cons:** No historical data — can't see peak usage times.
No per-user connection tracking.

**Verdict:** Add connection history and per-user tracking for abuse detection.

---

### 3.8 Embedding Deduplication

**Endpoints:**
- `POST /api/admin/embedding-dedupe` — run embedding-based duplicate detection

**Pros:** Finds near-duplicate works (rewrites, slight title changes).

**Cons:** One-shot operation — no scheduling.
No threshold tuning — false positives/negatives can't be adjusted.

**Verdict:** Add scheduling (weekly auto-scan) and configurable similarity threshold.

---

### 3.9 Search Mining

**Endpoints:**
- `POST /api/admin/search-mining` — extract trending terms from search logs

**Pros:** Identifies emerging fandoms/tags.

**Cons:** No integration with auto-tagger — mined terms don't become tags automatically.

**Verdict:** Pipe mined terms into auto-tagger queue.

---

## 4. Curator Tools

### 4.1 Curator Queue (TL3+)

**Endpoints:**
- `GET /api/curator/flags` — list items flagged for curator review

**Gate:** `require_curator()` — TL3+

**Pros:** Separates curation from moderation — curators improve content, moderators enforce rules.

**Cons:**
- No priority ordering — oldest first, no "urgent" flag
- No assignment — all curators see the same queue, risk of duplicate work
- No "skip" button — curators must act or items pile up

**Verdict:** Add priority, assignment, and skip functionality.

---

### 4.2 Work Proposals

**Endpoints:**
- `POST /api/curator/proposals` — propose a metadata fix
- `POST /api/curator/proposals/{id}/vote` — vote on a proposal
- `GET /api/curator/proposals` — list open proposals

**Mechanics:**
- Proposals require N votes from TL3+ to auto-apply
- `propose_metadata_fix()` for title/fandom/tag changes
- `propose_fix()` for body/content changes

**Pros:** Democratic — no single curator can unilaterally change metadata.

**Cons:**
- No notification to the work's author about proposed changes
- No "discuss" phase — voting is binary (approve/reject)
- No minimum voter trust level — TL3 and TL6 have equal vote weight

**Verdict:** Add author notification, discussion phase, and weighted voting by trust level.

---

### 4.3 Body Management

**Endpoints:**
- `GET /api/curator/bodies/{id}` — get work body
- `DELETE /api/curator/bodies/{id}` — delete body (DMCA/compliance)

**Pros:** Necessary for legal compliance.

**Cons:**
- No "replace body" — can only delete, not fix
- No audit trail visible to admins
- No notification to downloaders that content was removed

**Verdict:** Add body replacement and downloader notification.

---

## 5. Gamification

### 5.1 XP System

**Ledger:** `xp_events` → `users.xx` (unified, migrated from `exp_events`/`users.exp`)

**XP Sources (from `xp_source_defs`):**
| Event | XP | Daily Cap |
|-------|-----|-----------|
| `post_created` | 2 | — |
| `post_reacted` | 5 | — |
| `poll_voted` | 3 | — |
| `message_sent` | 1 | — |
| `work_read` | 1 | — |
| `work_bookmarked` | 1 | — |
| `review_left` | 5 | — |
| `level_up` | 0 | — |

**Mechanics:**
- Cooldown per event type (prevents spam)
- Daily cap per event type
- Streak multiplier (consecutive days)
- Rate limit per period

**Pros:**
- Anti-gaming: cooldowns + daily caps + rate limits
- Streak multiplier rewards consistency
- `ON CONFLICT DO NOTHING` prevents double-award

**Cons:**
- XP amounts are hardcoded in `xp_source_defs` — no admin UI to adjust
- No "XP penalty" for negative actions (spam, reports)
- No XP leaderboard (only trust leaderboard)

**Verdict:** Add admin UI for XP tuning and XP penalty system.

---

### 5.2 Achievements

**20 achievement types** (see migration 091):
- `first_post`, `popular_post`, `forum_veteran`, `topic_creator`
- `reaction_giver`, `poll_master`, `social_butterfly`
- `level_5`, `level_10`, `level_25`, `level_50`, `level_100`
- `mod_action`, `loremaster`, `collector`, `reviewer`
- `daily_visitor`, `night_owl`, `early_bird`, `weekend_warrior`

**Endpoint:** `GET /api/forum/users/{userId}/achievements`

**Pros:**
- Backfilled for existing users
- `ON CONFLICT (user_id, feature_id) DO NOTHING` prevents duplicates
- `check_and_unlock_achievements()` is best-effort, never fails the caller

**Cons:**
- No achievement icons/badges in UI
- No "achievement unlocked" notification
- No rarity tracking (what % of users have this?)
- `daily_visitor`, `night_owl`, `early_bird`, `weekend_warrior` are defined but not wired into any handler

**Verdict:** Wire up time-based achievements. Add notification on unlock. Add rarity stats.

---

### 5.3 Leaderboard

**Endpoint:** `GET /api/leaderboard` — trust-based ranking

**Pros:** Public recognition for active community members.

**Cons:**
- Trust-only — no XP leaderboard, no achievement leaderboard
- No time filter (weekly/monthly/all-time)
- No category filter (most works read, most forum posts, etc.)

**Verdict:** Add XP and achievement leaderboards with time/category filters.

---

## 6. Collections & Challenges

### 6.1 Collections

**Endpoints:**
- `POST /api/collections` — create
- `GET /api/collections/{id}` — view
- `PUT /api/collections/{id}` — update
- `DELETE /api/collections/{id}` — delete
- `POST /api/collections/{id}/items` — add work
- `DELETE /api/collections/{id}/items/{workId}` — remove work
- `POST /api/collections/{id}/bookmark` — bookmark collection
- `GET /api/collections/{id}/bookmarkers` — list bookmarkers

**Pros:** Standard CRUD with bookmarking.

**Cons:**
- No collection categories (fandom, trope, etc.)
- No "featured collections" — all collections are equal
- No moderation queue for collection items

**Verdict:** Add categories, featured collections, and item moderation.

---

### 6.2 Challenges

**Endpoints:**
- `GET /api/challenges/{id}` — challenge info
- `POST /api/challenges/{id}/signup` — sign up
- `POST /api/challenges/{id}/assign` — assign prompts (curator+)

**Pros:** Community engagement through structured events.

**Cons:**
- No challenge creation endpoint — must be done via DB
- No progress tracking
- No winner selection

**Verdict:** Add challenge creation UI, progress tracking, and winner selection.

---

## 7. Reporting System

### 7.1 User Reports

**Endpoints:**
- `POST /api/reports` — create a report
- `GET /api/admin/reports` — list reports (admin+)
- `POST /api/admin/reports/{id}/resolve` — resolve

**Mechanics:**
- `run_report_triage()` auto-classifies reports
- `auto_hide_effect()` auto-hides content for clear violations
- Reports can target: work, post, comment, user

**Pros:**
- Auto-triage reduces moderator workload
- Auto-hide for clear violations (spam, CSAM)

**Cons:**
- No reporter feedback — user doesn't know if their report was acted upon
- No "false report" penalty — users can spam reports
- No priority scoring — all reports are equal

**Verdict:** Add reporter feedback, false report penalties, and priority scoring.

---

## 8. Feature Flags

### 8.1 Feature Gate

**Endpoints:**
- `GET /api/features` — list all features
- `GET /api/features/available` — features available to current user
- `POST /api/features/{slug}/enable` — enable for self
- `POST /api/features/{slug}/disable` — disable for self
- `PUT /api/features/{slug}` — update feature (admin+)
- `GET /api/features/stats` — usage stats (admin+)
- `GET /api/admin/features` — admin feature management

**Gate types:** `rank`, `trust`, `xp`, `none`

**Pros:**
- Gradual rollout — enable features for TL5+ first, then expand
- Per-user opt-in/out
- Admin override

**Cons:**
- No A/B testing — can't show feature to 50% of users
- No "forced on" — users can always opt out
- No feature dependencies (feature B requires feature A)

**Verdict:** Add A/B testing support and feature dependencies.

---

## 9. Notifications

### 9.1 Notification System

**Endpoints:**
- `GET /api/notifications` — list notifications
- `POST /api/notifications/{id}/read` — mark as read
- `POST /api/notifications/read-all` — mark all as read
- `GET /api/notifications/unread-count` — unread count
- `GET /api/notifications/preferences` — get preferences
- `PUT /api/notifications/preferences` — update preferences

**Notification types:**
- `message`, `forum_reply`, `forum_mention`, `report_resolved`
- `work_featured`, `achievement_unlocked`, `level_up`
- `collection_item_approved`, `collection_item_rejected`

**Pros:**
- Per-type preferences — users can mute specific types
- Unread count for badge display

**Cons:**
- No batch "mark as read" by type
- No "mute all" temporary (vacation mode)
- No email digest option

**Verdict:** Add batch mark-by-type, vacation mode, and email digest.

---

## 10. Subsystems

### 10.1 Invite System

**Endpoints:**
- `POST /api/invites` — create invite (admin+)
- `GET /api/invites` — list invites (admin+)
- Registration modes: `open`, `invite`, `application`

**Pros:** Three registration modes cover most use cases.

**Cons:**
- No invite expiration (invites are forever)
- No invite usage limit (one invite can be used N times)
- No "invite tree" — can't see who invited whom

**Verdict:** Add expiration, usage limits, and invite tree.

---

### 10.2 Registration Applications

**Endpoints:**
- `POST /api/registration-applications` — apply
- `GET /api/admin/registration-applications` — list (admin+)
- `POST /api/admin/registration-applications/{id}/review` — approve/reject

**Pros:** Vetting for closed communities.

**Cons:**
- No application questions — just a free-text field
- No "why do you want to join?" prompt
- No notification to applicant on decision

**Verdict:** Add structured questions and applicant notification.

---

### 10.3 User Blocking

**Endpoints:**
- `POST /api/users/{id}/block` — block user
- `DELETE /api/users/{id}/block` — unblock
- `GET /api/users/blocks` — list blocks

**Mechanics:**
- Blocked users can't DM you
- Blocked users' content is hidden from your feeds

**Pros:** Essential for user safety.

**Cons:**
- No "block + report" combined action
- No block reason (for admin review)
- No "blocked you" notification to the blocker when a blocked user tries to interact

**Verdict:** Add block+report combined action and block reasons.

---

## 11. Content Management

### 11.1 Work Deletion

**Endpoint:** `DELETE /api/works/{id}` — delete work (admin+ or author)

**Pros:** Necessary for DMCA and policy violations.

**Cons:**
- No "soft delete" — hard delete only
- No "delete and ban author" combined action
- No notification to downloaders

**Verdict:** Add soft delete, combined delete+ban, and downloader notification.

---

### 11.2 Work Metadata Update

**Endpoint:** `PUT /api/admin/works/{id}/metadata` — update title/fandom/tags (admin+)

**Pros:** Fix metadata errors without curator proposal.

**Cons:**
- No audit trail — changes are silent
- No author notification

**Verdict:** Add audit trail and author notification.

---

### 11.3 Tag Management

**Endpoints:**
- `GET /api/tags` — list tags
- `POST /api/tags` — create tag (curator+)
- `PUT /api/tags/{id}` — update tag (curator+)
- `DELETE /api/tags/{id}` — delete tag (admin+)

**Pros:** Controlled vocabulary for fandoms, characters, tropes.

**Cons:**
- No tag aliases (e.g., "Harry Potter" = "HP")
- No tag hierarchy (parent/child)
- No tag merging (duplicate tags can't be combined)

**Verdict:** Add aliases, hierarchy, and merge functionality.

---

## 12. Realtime (WS/SSE)

### 12.1 Channels

| Channel | Access | Events |
|---------|--------|--------|
| `user:{id}` | Owning session only | `message_new`, `notification` |
| `topic:{id}` | Public (if topic visible) | `topic_reply`, `poll_closed` |
| `room:{id}` | Room members only | `message_new` |

**Pros:**
- Channel-level authorization prevents information leakage
- `publish_event()` is best-effort — Redis failure doesn't break local delivery

**Cons:**
- No "presence" — can't see who's online in a room
- No "typing indicator"
- No message history replay on reconnect

**Verdict:** Add presence, typing indicators, and message history replay.

---

## 13. Summary & Recommendations

### Critical (fix now)

| # | Issue | Impact | Effort |
|---|-------|--------|--------|
| 1 | No temporary bans | Moderators can't issue time-outs | Low |
| 2 | No batch moderation | Moderator burnout on spam waves | Medium |
| 3 | No reporter feedback | Users spam reports when ignored | Low |
| 4 | No author notification on metadata changes | Trust erosion | Medium |

### High (next sprint)

| # | Issue | Impact | Effort |
|---|-------|--------|--------|
| 5 | No achievement notifications | Gamification feels dead | Low |
| 6 | No XP leaderboard | Less engagement | Medium |
| 7 | No collection moderation | Spam in collections | Medium |
| 8 | No invite expiration | Stale invites accumulate | Low |
| 9 | No "block + report" | Extra clicks for users | Low |

### Medium (backlog)

| # | Issue | Impact | Effort |
|---|-------|--------|--------|
| 10 | No tag merging | Duplicate tags | Medium |
| 11 | No A/B testing | Can't test features safely | High |
| 12 | No translation versioning | Can't revert bad translations | Medium |
| 13 | No challenge creation UI | Manual DB work for events | Medium |
| 14 | No realtime presence | Rooms feel empty | High |

### Low (nice-to-have)

| # | Issue | Impact | Effort |
|---|-------|--------|--------|
| 15 | No reputation decay | Stale reputation | Low |
| 16 | No search analytics export | Manual analysis | Low |
| 17 | No bot detection transparency | False positive confusion | Low |
| 18 | No email digest for notifications | Email-only users miss out | Medium |

---

## 14. Role Permission Matrix

| Feature | User | TL2 | TL3 (Curator) | TL5 (Mod) | Admin |
|---------|------|-----|---------------|-----------|-------|
| Read works | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bookmark | ✅ | ✅ | ✅ | ✅ | ✅ |
| Review | ✅ | ✅ | ✅ | ✅ | ✅ |
| Kudos | ✅ | ✅ | ✅ | ✅ | ✅ |
| Publish works | ❌ | ✅ | ✅ | ✅ | ✅ |
| Forum post | ❌ | ✅ | ✅ | ✅ | ✅ |
| Curator queue | ❌ | ❌ | ✅ | ✅ | ✅ |
| Propose metadata fix | ❌ | ❌ | ✅ | ✅ | ✅ |
| Moderate forum | ❌ | ❌ | ❌ | ✅ | ✅ |
| Resolve reports | ❌ | ❌ | ❌ | ✅ | ✅ |
| Meta-moderation | ❌ | ❌ | ❌ | ✅ | ✅ |
| Ban users | ❌ | ❌ | ❌ | ❌ | ✅ |
| Set trust levels | ❌ | ❌ | ❌ | ❌ | ✅ |
| Manage features | ❌ | ❌ | ❌ | ❌ | ✅ |
| Bot management | ❌ | ❌ | ❌ | ❌ | ✅ |
| Content scanning | ❌ | ❌ | ❌ | ❌ | ✅ |

---

*Generated from codebase inspection of `src/routes/`, `src/services/`, and `migrations/`.*
*For questions or corrections, open an issue or contact the dev team.*

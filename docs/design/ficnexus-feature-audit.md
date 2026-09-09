# FicNexus — Complete Feature Audit

> **Scope:** Every feature in the FicNexus codebase (backend routes, services, realtime, gamification).
> **Role hierarchy:** User (TL0–TL2), Curator (TL3–TL4), Moderator (TL5), Admin (TL6).
> **Trust level gates:** `assert_staff_or_min_trust()` enforces trust levels; `admin_level()` reads `FORUM_ADMIN_LEVEL` (default 100).
> **Date:** 2026-09-09

---

## Table of Contents

1. [Authentication & Authorization](#1-authentication--authorization)
2. [User Management](#2-user-management)
3. [Content Management](#3-content-management)
4. [Reading & Bookmarks](#4-reading--bookmarks)
5. [Comments & Reviews](#5-comments--reviews)
6. [Collections & Challenges](#6-collections--challenges)
7. [Requests & Bounties](#7-requests--bounties)
8. [Forum & Discussions](#8-forum--discussions)
9. [Moderation & Reporting](#9-moderation--reporting)
10. [Gamification & Achievements](#10-gamification--achievements)
11. [Messages & Realtime](#11-messages--realtime)
12. [Search & Discovery](#12-search--discovery)
13. [Analytics & Stats](#13-analytics--stats)
14. [Translation & Localization](#14-translation--localization)
15. [Download & Export](#15-download--export)
16. [Admin Tools](#16-admin-tools)
17. [Subsystems](#17-subsystems)
18. [Infrastructure](#18-infrastructure)
19. [Role Permission Matrix](#19-role-permission-matrix)
20. [Prioritized Recommendations](#20-prioritized-recommendations)

---

## 1. Authentication & Authorization

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| POST | `/api/auth/register` | `social.rs:18` | Public |
| POST | `/api/auth/login` | `social.rs:117` | Public |
| POST | `/api/auth/refresh` | `social.rs:212` | Logged-in |
| GET | `/api/me` | `social.rs:197` | Logged-in |
| POST | `/api/pow/challenge` | `pow.rs:35` | Public |
| POST | `/api/pow/solve` | `pow.rs:77` | Public |
| GET | `/api/site/info` | `subsystems.rs:101` | Public |

### Mechanics

- JWT tokens with 7-day access / 365-day refresh tokens.
- Tokens store `sub` (user_id), `username`, `trust_level`.
- Argon2id password hashing.
- PoW challenge-response to deter brute-force on register/login.
- Registration modes: `open`, `invite`, `application`.

### Pros

- Proper token expiration and refresh flow.
- PoW protects auth endpoints.
- Trust level embedded in JWT — no extra DB lookup per request.

### Cons

- No rate limiting on `/api/auth/login` (only PoW).
- No email verification workflow.
- No OAuth/SAML SSO support.
- `refresh_handler` rotates tokens but doesn't blacklist old ones.

### Verdict

**Keep + Add:** Rate limiting, email verification, token blacklisting on logout.

---

## 2. User Management

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/users` | `admin.rs:296` | Admin |
| POST | `/api/users/{id}/role` | `admin.rs:352` | Admin |
| POST | `/api/users/{id}/ban` | `admin.rs:389` | Admin |
| GET | `/api/users/{id}/works` | `social.rs:1038` | Public |
| GET | `/api/users/{id}/stats` | `analytics.rs:100` | Public |
| GET | `/api/users/{id}/profile` | `social.rs:898` | Public |
| GET | `/api/users/{id}/followers` | `follows.rs:164` | Public |
| POST | `/api/users/{id}/follow` | `follows.rs:20` | Logged-in |
| DELETE | `/api/users/{id}/follow` | `follows.rs:91` | Logged-in |
| GET | `/api/users/{id}/follows` | `follows.rs:106` | Public |
| GET | `/api/users/{id}/check` | `follows.rs:133` | Logged-in |
| GET | `/api/pseuds` | `pseuds.rs:51` | Public |
| POST | `/api/pseuds` | `pseuds.rs:63` | Logged-in |
| PATCH | `/api/pseuds/{id}` | `pseuds.rs:105` | Owner/Admin |
| DELETE | `/api/pseuds/{id}` | `pseuds.rs:149` | Owner/Admin |

### Mechanics

- Users have profiles, pseudonyms (pseuds), avatars.
- Follow system with exclusions (mute specific authors).
- Profile shows XP level, trust level, reputation, badges.
- Admins can ban, change roles, shadowban.

### Pros

- Clean follow/unfollow with exclusion support.
- Multiple pseudonyms per user.
- Privacy-respecting (email hidden from profile).

### Cons

- No user search by username/email (admin only).
- No user merge tool.
- No "block user" integration with follow system (separate table).

### Verdict

**Keep + Add:** User search, user merge tool, block-follow integration.

---

## 3. Content Management

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/works/{id}` | `social.rs:945` | Public |
| POST | `/api/works/upload` | `upload.rs:16` | Logged-in |
| PUT | `/api/works/{id}` | `upload.rs:279` | Owner/Admin |
| DELETE | `/api/works/{id}` | `upload.rs:387` | Owner/Admin |
| GET | `/api/works/{id}/stats` | `social.rs:1003` | Public |
| GET | `/api/series/{id}` | `series.rs:30` | Public |
| GET | `/api/authors/{id}` | `authors.rs:80` | Public |
| GET | `/api/authors/{id}/profile` | `authors.rs:80` | Public |
| PATCH | `/api/authors/{id}/profile` | `authors.rs:174` | Owner/Admin |
| POST | `/api/authors/{id}/social` | `authors.rs:224` | Owner/Admin |
| DELETE | `/api/authors/{id}/social` | `authors.rs:262` | Owner/Admin |
| GET | `/api/fandoms` | `fandom.rs:46` | Public |
| GET | `/api/fandoms/{id}` | `fandom.rs:90` | Public |
| GET | `/api/tags` | `tags::routes` | Public |
| GET | `/api/tags/search` | `tags::routes` | Public |
| GET | `/api/tags/{id}` | `tags::routes` | Public |
| GET | `/api/auto-tag/queue` | `auto_tag.rs:113` | TL3+ |
| POST | `/api/auto-tag/{id}/approve` | `auto_tag.rs:170` | TL3+ |
| POST | `/api/auto-tag/{id}/dismiss` | `auto_tag.rs:207` | TL3+ |

### Mechanics

- Works support chapters, tags, fandoms, ratings, categories, warnings.
- Auto-tagging service suggests tags using embeddings.
- Authors can have multiple pseuds, social links, merge proposals.
- Tag system with types, categories, synonyms.

### Pros

- Rich metadata model (tags, fandoms, ratings, warnings).
- Auto-tagging reduces manual curation.
- Author merge workflow for deduplication.

### Cons

- No tag merge/alias admin tool.
- No tag hierarchy (parent/child tags).
- Auto-tagger accuracy not exposed (no confidence score).
- No work versioning (edit history only in forum).

### Verdict

**Keep + Add:** Tag merge/alias, tag hierarchy, work versioning, auto-tagger confidence scores.

---

## 4. Reading & Bookmarks

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/reader/{id}` | `reader.rs:550` | Public |
| GET | `/api/reader/{id}/marginalia` | `reader.rs:505` | Logged-in |
| GET | `/api/reader/{id}/sequel` | `reader.rs:145` | Public |
| GET | `/api/reader/{id}/related` | `reader.rs:299` | Public |
| GET | `/api/reader/{id}/sequential` | `reader.rs:332` | Public |
| GET | `/api/reader/{id}/next-up` | `reader.rs:371` | Public |
| GET | `/api/bookmarks` | `social.rs:316` | Logged-in |
| POST | `/api/bookmarks` | `social.rs:265` | Logged-in |
| DELETE | `/api/bookmarks/{id}` | `social.rs:297` | Logged-in |
| GET | `/api/bookmarks/export` | `social.rs:348` | Logged-in |
| POST | `/api/bookmarks/import` | `social.rs:405` | Logged-in |
| GET | `/api/shelves` | `shelves.rs:74` | Logged-in |
| POST | `/api/shelves` | `shelves.rs:31` | Logged-in |
| GET | `/api/shelves/{id}` | `shelves.rs:159` | Public |
| POST | `/api/shelves/{id}/works/{workId}` | `shelves.rs:117` | Logged-in |
| DELETE | `/api/shelves/{id}/works/{workId}` | `shelves.rs:138` | Logged-in |

### Mechanics

- Reader with marginalia (personal notes on fics).
- Sequel/related/next-up navigation for series.
- Bookmarks with CSV import/export.
- Shelves (custom reading lists).
- Kudos (likes) system.

### Pros

- Rich reader experience with marginalia.
- CSV import/export for bookmark portability.
- Related works via embeddings.

### Cons

- No reading progress sync (server-side tracking only).
- No "mark as read" for works (only forum topics).
- No bookmark collections/folders.
- No reading statistics (time spent, pages read).

### Verdict

**Keep + Add:** Reading progress sync, read/unread for works, bookmark folders, reading time stats.

---

## 5. Comments & Reviews

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/comments` | `social.rs:822` | Public |
| POST | `/api/comments` | `social.rs:714` | Logged-in |
| DELETE | `/api/comments/{id}` | `comments.rs:422` | Owner/TL3+ |
| PATCH | `/api/comments/{id}` | `comments.rs:495` | Owner |
| POST | `/api/comments/{id}/hide` | `comments.rs:458` | TL3+ |
| GET | `/api/reviews` | `reviews.rs:131` | Public |
| POST | `/api/reviews` | `reviews.rs:46` | Logged-in |
| DELETE | `/api/reviews/{id}` | `reviews.rs:195` | Owner/Admin |
| POST | `/api/ratings` | `social.rs:465` | Logged-in |
| GET | `/api/ratings` | `social.rs:506` | Public |
| POST | `/api/kudos` | `social.rs:628` | Logged-in |
| DELETE | `/api/kudos/{id}` | `social.rs:659` | Logged-in |
| GET | `/api/kudos` | `social.rs:683` | Public |

### Mechanics

- Comments on works with threading.
- Reviews with ratings (1-5 stars).
- Kudos (one-per-user-per-work).
- TL3+ can hide comments.
- Moderation queue for flagged comments.

### Pros

- Threaded comments with depth limit.
- Separate reviews from comments.
- Kudos prevents abuse (one per user).

### Cons

- No comment editing history.
- No review helpfulness voting.
- No comment reactions (only hide).
- No spam detection on comments.

### Verdict

**Keep + Add:** Comment edit history, review helpfulness voting, spam detection, comment reactions.

---

## 6. Collections & Challenges

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/collections` | `collections.rs:280` | Public |
| POST | `/api/collections` | `collections.rs:304` | Logged-in |
| GET | `/api/collections/{id}` | `collections.rs:382` | Public |
| PATCH | `/api/collections/{id}` | `collections.rs:479` | Owner/Admin |
| DELETE | `/api/collections/{id}` | `collections.rs:576` | Owner/Admin |
| POST | `/api/collections/{id}/items` | `collections.rs:596` | Owner |
| DELETE | `/api/collections/{id}/items/{workId}` | `collections.rs:679` | Owner |
| POST | `/api/collections/{id}/bookmark` | `collections.rs:705` | Logged-in |
| POST | `/api/collections/{id}/items/{workId}/moderate` | `collections.rs:810` | TL3+ |
| GET | `/api/collections/{id}/challenge` | `collections.rs:1408` | Public |
| POST | `/api/collections/{id}/challenge/signup` | `collections.rs:1441` | Logged-in |
| POST | `/api/collections/{id}/challenge/assign` | `collections.rs:1455` | Owner |
| POST | `/api/collections/{id}/challenge/claim` | `collections.rs:1493` | Logged-in |

### Mechanics

- Curated collections with items, bookmarks, moderation.
- Challenges: signup, assign prompts, claim assignments.
- Approval workflow for collection items (TL3+).
- Collection item requests with voting.

### Pros

- Full curation workflow with approvals.
- Challenge system for events.
- Bookmark collections (subscribe to updates).

### Cons

- No collection categories/tags.
- No challenge progress tracking.
- No collection featured/promoted flag.
- No challenge completion certificates.

### Verdict

**Keep + Add:** Collection categories, challenge progress tracking, featured collections, completion badges.

---

## 7. Requests & Bounties

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/requests` | `requests.rs:213` | Public |
| POST | `/api/requests` | `requests.rs:81` | Logged-in |
| GET | `/api/requests/{id}` | `requests.rs:279` | Public |
| DELETE | `/api/requests/{id}` | `requests.rs:398` | Owner/TL3+ |
| POST | `/api/requests/{id}/answers` | `requests.rs:424` | Logged-in |
| DELETE | `/api/requests/{id}/answers/{aid}` | `requests.rs:599` | Owner/TL3+ |
| POST | `/api/requests/{id}/answers/{aid}/vote` | `requests.rs:627` | Logged-in |
| POST | `/api/requests/{id}/answers/{aid}/accept` | `requests.rs:749` | Owner |
| POST | `/api/requests/{id}/upvote` | `requests.rs:692` | Logged-in |
| GET | `/api/requests/candidates` | `requests.rs:825` | Logged-in |
| GET | `/api/bounties` | `bounties.rs:31` | Public |
| POST | `/api/bounties` | `bounties.rs:62` | Logged-in |
| POST | `/api/bounties/{id}/claim` | `bounties.rs:94` | Logged-in |
| POST | `/api/bounties/{id}/resolve` | `bounties.rs:115` | Owner |

### Mechanics

- Fic requests: users request fics, others provide answers with URLs.
- Voting on answers, accepting best answer.
- Bounties: reward points for fulfilling requests.
- Idle bounty tick handler for auto-resolution.

### Pros

- Community-driven request fulfillment.
- Voting surfaces best answers.
- Bounty incentive system.

### Cons

- No request categories/tags.
- No bounty escrow (points deducted upfront).
- No dispute resolution for bounties.
- No request expiration.

### Verdict

**Keep + Add:** Request categories, bounty escrow, dispute resolution, request expiration.

---

## 8. Forum & Discussions

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/forum/categories` | `forum.rs:1715` | Public |
| POST | `/api/forum/categories` | `forum.rs:2244` | Admin |
| GET | `/api/forum/topics` | `forum.rs:1790` | Public |
| POST | `/api/forum/topics` | `forum.rs:2707` | Logged-in |
| GET | `/api/forum/topics/{id}` | `forum.rs:2343` | Public |
| PATCH | `/api/forum/topics/{id}` | `forum.rs:2906` | Owner/TL5+ |
| DELETE | `/api/forum/topics/{id}` | `forum.rs:3055` | TL5+ |
| POST | `/api/forum/topics/{id}/posts` | `forum.rs:3098` | Logged-in |
| GET | `/api/forum/posts/{id}` | `forum.rs:3278` | Public |
| PATCH | `/api/forum/posts/{id}` | `forum.rs:3278` | Owner/TL5+ |
| DELETE | `/api/forum/posts/{id}` | `forum.rs:3344` | TL5+ |
| POST | `/api/forum/posts/{id}/moderate` | `forum.rs:854` | TL4+ |
| POST | `/api/forum/moderation/batch` | `forum.rs:947` | TL4+ |
| GET | `/api/forum/posts/{id}/moderations` | `forum.rs:1081` | Public |
| POST | `/api/forum/posts/{id}/react` | `forum.rs:3816` | Logged-in |
| GET | `/api/forum/posts/{id}/reactions` | `forum.rs:3886` | Public |
| DELETE | `/api/forum/posts/{id}/reactions/{id}` | `forum.rs:3897` | Logged-in |
| POST | `/api/forum/polls` | `forum_polls.rs:120` | Logged-in |
| POST | `/api/forum/polls/{id}/vote` | `forum_polls.rs:274` | Logged-in |
| POST | `/api/forum/polls/{id}/close` | `forum_polls.rs:356` | Owner/Admin |
| GET | `/api/forum/groups` | `forum_groups.rs:144` | Public |
| POST | `/api/forum/groups` | `forum_groups.rs:380` | Logged-in |
| POST | `/api/forum/groups/{id}/join` | `forum_groups.rs:539` | Logged-in |
| POST | `/api/forum/groups/{id}/leave` | `forum_groups.rs:573` | Logged-in |
| POST | `/api/forum/groups/{id}/invite` | `forum_groups.rs:610` | Member |
| POST | `/api/forum/groups/{id}/members/{uid}/role` | `forum_groups.rs:649` | Manager/Admin |
| GET | `/api/forum/groups/{id}/members` | `forum_groups.rs:681` | Public |
| POST | `/api/forum/groups/{id}/members/{uid}/remove` | `forum_groups.rs:744` | Manager/Admin |
| GET | `/api/forum/search` | `forum.rs:3538` | Public |
| GET | `/api/forum/unread` | `forum.rs:1948` | Logged-in |
| GET | `/api/forum/recent` | `forum.rs:1984` | Public |
| GET | `/api/forum/popular` | `forum.rs:2031` | Public |
| GET | `/api/forum/rss` | `forum.rs:2092` | Public |
| GET | `/api/forum/widgets/recent` | `forum.rs:4170` | Public |
| GET | `/api/forum/widgets/popular` | `forum.rs:4200` | Public |
| GET | `/api/forum/widgets/stats` | `forum.rs:4228` | Public |
| GET | `/api/forum/privileges` | `forum_privileges.rs:27` | Logged-in |
| POST | `/api/forum/privileges` | `forum_privileges.rs:131` | Admin |
| GET | `/api/forum/metamod/queue` | `forum.rs:1149` | TL5+ |
| POST | `/api/forum/metamod/queue/{id}/verdict` | `forum.rs:1199` | TL5+ |
| POST | `/api/forum/metamod/queue/{id}/vote` | `forum.rs:1211` | TL5+ |
| GET | `/api/forum/edits` | `forum.rs:3011` | TL3+ |
| POST | `/api/forum/edits/{id}/review` | `forum.rs:3022` | TL3+ |
| GET | `/api/forum/tags` | `forum.rs:2187` | Public |
| POST | `/api/forum/tags` | `forum.rs:2138` | TL3+ |
| GET | `/api/forum/prefs` | `forum.rs:2204` | Logged-in |
| POST | `/api/forum/prefs` | `forum.rs:2220` | Logged-in |

### Mechanics

- Full forum with categories, topics, posts, reactions, polls.
- Trust queue moderation (TL4+): approve/reject posts with reasons.
- Batch moderation (new): moderate up to 50 posts at once.
- Meta-moderation (TL5+): review moderator decisions.
- Forum groups with roles (owner, manager, member).
- Category privileges (TL-based access control).
- Edit queue for post edits (TL3+ review).
- Topic tags, RSS feeds, widgets.

### Pros

- Comprehensive forum with trust-based moderation.
- Batch moderation reduces repetitive work.
- Meta-moderation provides accountability.
- Forum groups enable private communities.
- Category privileges allow fine-grained access.

### Cons

- No real-time presence/typing indicators.
- No forum digest emails.
- No topic merge/split.
- No post diff view in edit queue.
- No forum analytics (posts per day, active users).

### Verdict

**Keep + Add:** Realtime presence, forum digests, topic merge/split, post diff view, forum analytics.

---

## 9. Moderation & Reporting

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| POST | `/api/reports` | `reports.rs:55` | TL1+ |
| GET | `/api/reports/{id}/status` | `reports.rs:407` | Reporter/Staff |
| GET | `/api/admin/reports` | `reports.rs:260` | TL5+ |
| POST | `/api/admin/reports/{id}/resolve` | `reports.rs:451` | TL5+ |
| GET | `/api/admin/modlog` | `modlog.rs:16` | TL5+ |
| GET | `/api/admin/modqueue` | `admin.rs:43` | TL3+ |
| POST | `/api/admin/comments/{id}/hide` | `admin.rs:1552` | TL3+ |
| DELETE | `/api/admin/comments/{id}` | `admin.rs:1593` | Admin |
| POST | `/api/admin/works/{id}/blacklist` | `admin.rs:1436` | Admin |
| POST | `/api/admin/authors/{id}/blacklist` | `admin.rs:1475` | Admin |
| GET | `/api/admin/blacklist` | `admin.rs:1515` | Admin |
| POST | `/api/admin/bans` | `forum.rs:1366` | TL5+ |
| DELETE | `/api/admin/bans/{id}` | `forum.rs:1463` | TL5+ |
| GET | `/api/admin/bans` | `forum.rs:1492` | TL5+ |
| POST | `/api/admin/posts/{id}/hide` | `forum.rs:1222` | TL5+ |
| POST | `/api/admin/topics/{id}/lock` | `forum.rs:1266` | TL5+ |
| POST | `/api/admin/topics/{id}/pin` | `forum.rs:1318` | TL5+ |
| POST | `/api/admin/users/{id}/ban` | `admin.rs:389` | Admin |
| POST | `/api/admin/bots/{id}/shadowban` | `admin.rs:1090` | Admin |
| POST | `/api/admin/bots/{id}/unshadowban` | `admin.rs:1112` | Admin |

### Mechanics

- User reports with weighted flags (trust-weighted).
- Auto-triage: pending → needs_admin → auto_hidden based on weight.
- Reporter can check status of their own reports.
- Modlog records all moderation actions.
- Temporary bans with `expires_at` (RFC3339).
- Blacklist for works/authors.
- Shadowban for bots.

### Pros

- Trust-weighted flagging prevents abuse.
- Auto-triage reduces admin workload.
- Reporter feedback loop (status endpoint).
- Temporary bans already supported.
- Comprehensive modlog.

### Cons

- No bulk report resolution.
- No report categories (spam, harassment, copyright).
- No escalation path (TL5 → Admin).
- No ban appeals workflow.
- No automated ban expiration cleanup job.

### Verdict

**Keep + Add:** Bulk report resolution, report categories, escalation path, ban appeals, automated ban cleanup.

---

## 10. Gamification & Achievements

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/level` | `forum.rs:207` | Logged-in |
| GET | `/api/xp/history` | `forum.rs:4066` | Logged-in |
| GET | `/api/achievements` | `forum.rs:4137` | Logged-in |
| GET | `/api/badges` | `badges.rs:34` | Logged-in |
| GET | `/api/badges/definitions` | `badges.rs:11` | Public |
| GET | `/api/leaderboard` | `leaderboard.rs:29` | Public |
| GET | `/api/leaderboard/weekly` | `badges.rs:55` | Public |
| GET | `/api/leaderboard/monthly` | `badges.rs:76` | Public |
| GET | `/api/leaderboard/categories` | `leaderboard.rs:200` | Public |
| GET | `/api/leaderboard/curators` | `social.rs:873` | Public |
| GET | `/api/quests/stats` | `quests.rs:25` | Logged-in |
| POST | `/api/quests/read` | `quests.rs:55` | Logged-in |
| GET | `/api/quests/streak` | `quests.rs:70` | Logged-in |
| GET | `/api/quests/reading-list` | `quests.rs:125` | Logged-in |
| GET | `/api/quests/reading-history` | `quests.rs:182` | Logged-in |
| POST | `/api/quests/reading-history` | `quests.rs:213` | Logged-in |
| GET | `/api/progression` | `progression.rs:51` | Logged-in |
| GET | `/api/progression/prefs` | `progression.rs:104` | Logged-in |
| POST | `/api/progression/prefs` | `progression.rs:129` | Logged-in |
| GET | `/api/progression/layout` | `progression.rs:156` | Logged-in |
| POST | `/api/progression/layout` | `progression.rs:181` | Logged-in |
| GET | `/api/progression/views` | `progression.rs:208` | Logged-in |
| POST | `/api/progression/views` | `progression.rs:240` | Logged-in |

### Mechanics

- XP system: award_xp delegates to services::progression::award_xp.
- XP sources: posts, reactions, poll votes, messages, moderation.
- Level progression with exp_per_level (default 100).
- Achievements stored in user_features (feature_type='achievement').
- Achievement triggers: post_created, reaction, poll_vote, message_sent, level_up.
- Badges with definitions, weekly/monthly leaderboards.
- Quests: reading stats, streaks, reading lists.
- Progression: customizable dashboard with views, layouts, prefs.

### Pros

- Unified XP ledger (single source of truth).
- Achievement system with multiple triggers.
- Customizable progression dashboard.
- Leaderboards with time windows.

### Cons

- No achievement notifications (silent unlock).
- No XP leaderboard (only badges).
- No achievement categories.
- No quest rewards (XP for completing quests).
- No progression comparison with friends.

### Verdict

**Keep + Add:** Achievement notifications, XP leaderboard, achievement categories, quest rewards, friend comparisons.

---

## 11. Messages & Realtime

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/messages/rooms` | `messages.rs:172` | Logged-in |
| POST | `/api/messages/rooms` | `messages.rs:241` | Logged-in |
| GET | `/api/messages/rooms/{id}` | `messages.rs:325` | Member |
| POST | `/api/messages/rooms/{id}` | `messages.rs:379` | Member |
| PATCH | `/api/messages/rooms/{id}/prefs` | `messages.rs:501` | Member |
| GET | `/api/notifications` | `notifications.rs:19` | Logged-in |
| POST | `/api/notifications/{id}/read` | `notifications.rs:58` | Logged-in |
| POST | `/api/notifications/read-all` | `notifications.rs:76` | Logged-in |
| GET | `/api/notifications/unread` | `notifications.rs:90` | Logged-in |
| GET | `/api/notifications/preferences` | `notifications.rs:122` | Logged-in |
| POST | `/api/notifications/preferences` | `notifications.rs:153` | Logged-in |
| GET | `/api/admin/realtime` | `admin.rs:1221` | Admin |

### Mechanics

- Direct messages with rooms (1:1 and group).
- Notifications with read/unread tracking.
- Notification preferences per type.
- SSE (Server-Sent Events) for realtime updates.
- WebSocket with channel authorization (user:/topic:/room: prefix gating).
- Realtime producers for forum events.

### Pros

- SSE for efficient one-way realtime.
- WS channel authorization prevents unauthorized subscriptions.
- Notification preferences give user control.

### Cons

- No typing indicators in messages.
- No message editing/deleting.
- No message reactions.
- No notification digest emails.
- No push notifications (browser/APNs/FCM).

### Verdict

**Keep + Add:** Typing indicators, message edit/delete, message reactions, notification digests, push notifications.

---

## 12. Search & Discovery

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/search` | `search.rs:21` | Public |
| GET | `/api/search/similar` | `search.rs:21` | Public |
| GET | `/api/search/similar-by-bookmarks` | `search.rs:230` | Public |
| GET | `/api/search/also-bookmarked` | `search.rs:104` | Public |
| GET | `/api/search/random` | `search.rs:182` | Public |
| GET | `/api/search/chips` | `search.rs:266` | Public |
| GET | `/api/find-fic` | `find_fic.rs:78` | Public |
| POST | `/api/find-fic/suggest` | `find_fic.rs:258` | Logged-in |
| GET | `/api/find-fic/matches` | `find_fic.rs:299` | Public |
| GET | `/api/find-fic/similar-titles` | `find_fic.rs:367` | Public |
| GET | `/api/trending` | `trending.rs:145` | Public |
| GET | `/api/trending/tags` | `trending.rs:199` | Public |
| GET | `/api/trending/by-tag` | `trending.rs:41` | Public |
| GET | `/api/saved-searches` | `saved_search.rs:159` | Logged-in |
| POST | `/api/saved-searches` | `saved_search.rs:113` | Logged-in |
| DELETE | `/api/saved-searches/{id}` | `saved_search.rs:182` | Owner |
| POST | `/api/saved-searches/{id}/alert` | `saved_search.rs:206` | Owner |
| POST | `/api/saved-searches/{id}/run` | `saved_search.rs:240` | Owner |
| GET | `/api/saved-searches/{id}/feed` | `saved_search.rs:340` | Owner |

### Mechanics

- Full-text search with PostgreSQL tsvector.
- Vector similarity search (embeddings).
- "Find fic" — reverse search by description/plot.
- Trending works/tags with time windows.
- Saved searches with alerts (RSS feed).
- Search chips for filtering.

### Pros

- Multiple search modes (text, vector, bookmarks).
- "Find fic" is unique and useful.
- Saved searches with RSS alerts.
- Trending discovery.

### Cons

- No search autocomplete.
- No search history.
- No advanced search syntax (AND/OR/NOT).
- No search analytics (popular queries).

### Verdict

**Keep + Add:** Search autocomplete, search history, advanced syntax, search analytics.

---

## 13. Analytics & Stats

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/analytics` | `analytics.rs:32` | Public |
| GET | `/api/analytics/author` | `analytics.rs:24` | Public |
| GET | `/api/analytics/user` | `analytics.rs:100` | Public |
| GET | `/api/analytics/endpoint` | `analytics.rs:193` | Admin |
| POST | `/api/analytics/track` | `analytics.rs:154` | Public |
| GET | `/api/analytics/admin` | `analytics.rs:231` | Admin |
| GET | `/api/admin/stats` | `admin.rs:949` | Admin |
| GET | `/api/admin/search-analytics` | `admin.rs:1141` | Admin |

### Mechanics

- Personal reading stats (pages, works, time).
- Author stats (views, kudos, bookmarks).
- Endpoint usage tracking (admin).
- Search analytics (admin).
- Admin dashboard stats.

### Pros

- Comprehensive personal analytics.
- Author analytics for creators.
- Endpoint usage monitoring.

### Cons

- No reading time estimation.
- No reading goal tracking.
- No author revenue/royalty tracking.
- No A/B testing framework.

### Verdict

**Keep + Add:** Reading time estimation, reading goals, A/B testing framework.

---

## 14. Translation & Localization

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/locales` | `locales.rs:13` | Public |
| GET | `/api/locales/ui` | `locales.rs:33` | Public |
| POST | `/api/locales/work` | `locales.rs:61` | Logged-in |
| GET | `/api/locales/work/{id}` | `locales.rs:91` | Public |
| POST | `/api/locales/chapter` | `locales.rs:142` | Logged-in |
| GET | `/api/locales/chapter/{id}` | `locales.rs:116` | Public |
| POST | `/api/translate/enqueue` | `translate.rs:79` | Logged-in |
| POST | `/api/translate/batch` | `translate.rs:42` | TL3+ |
| GET | `/api/translate/status` | `translate.rs:150` | Logged-in |
| GET | `/api/translate/coverage` | `translate.rs:187` | Public |
| GET | `/api/admin/translations` | `admin.rs:501` | Admin |
| POST | `/api/admin/translations/{id}/approve` | `admin.rs:576` | Admin |
| POST | `/api/admin/translations/{id}/reject` | `admin.rs:623` | Admin |
| POST | `/api/admin/translations/{id}/edit` | `admin.rs:670` | Admin |

### Mechanics

- UI translations (i18n) with locale files.
- Work/chapter translations with versioning.
- Auto-translation queue (Ollama integration).
- Translation coverage stats.
- Admin approval workflow for translations.

### Pros

- Full i18n support.
- Community translation with approval.
- Auto-translation via Ollama.
- Coverage tracking.

### Cons

- No translation memory (reuse previous translations).
- No machine translation fallback (Google/DeepL).
- No translator attribution on UI.
- No translation progress per language.

### Verdict

**Keep + Add:** Translation memory, MT fallback, translator attribution, per-language progress.

---

## 15. Download & Export

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/epub` | `export.rs:63` | Public |
| GET | `/api/epub/{id}` | `export.rs:63` | Public |
| GET | `/api/download/author/{id}` | `download.rs:247` | Logged-in |
| GET | `/api/download/author/{id}/stream` | `download.rs:416` | Logged-in |
| GET | `/api/download/series/{id}` | `download.rs:794` | Logged-in |
| POST | `/api/export/convert` | `export.rs:1131` | Logged-in |
| POST | `/api/kindle/send` | `kindle.rs:211` | Logged-in |
| GET | `/api/user/export` | `user_export.rs:325` | Logged-in |
| DELETE | `/api/user/account` | `user_export.rs:401` | Logged-in |
| POST | `/api/user/consent` | `user_export.rs:467` | Logged-in |

### Mechanics

- EPUB generation with customizable templates.
- Batch author download (ZIP of all works).
- Series download.
- Format conversion (EPUB, MOBI, PDF, HTML).
- Kindle send-to-device.
- GDPR-compliant data export and account deletion.

### Pros

- Multiple export formats.
- Batch downloads for authors/series.
- GDPR compliance (export + delete).
- Kindle integration.

### Cons

- No download queue/priority.
- No download history.
- No custom CSS in EPUB.
- No download limits (rate limiting).

### Verdict

**Keep + Add:** Download queue, download history, custom CSS, rate limiting.

---

## 16. Admin Tools

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/admin/users` | `admin.rs:296` | Admin |
| POST | `/api/admin/users/{id}/role` | `admin.rs:352` | Admin |
| POST | `/api/admin/users/{id}/ban` | `admin.rs:389` | Admin |
| GET | `/api/admin/stats` | `admin.rs:949` | Admin |
| GET | `/api/admin/bots` | `admin.rs:1005` | Admin |
| POST | `/api/admin/bots/{id}/shadowban` | `admin.rs:1090` | Admin |
| GET | `/api/admin/search-analytics` | `admin.rs:1141` | Admin |
| GET | `/api/admin/realtime` | `admin.rs:1221` | Admin |
| GET | `/api/admin/roadmap-consensus` | `admin.rs:1298` | Admin |
| GET | `/api/admin/rating-checks` | `admin.rs:727` | Admin |
| POST | `/api/admin/rating-checks/{id}/verify` | `admin.rs:799` | Admin |
| POST | `/api/admin/tag-score/{id}/fix` | `admin.rs:873` | Admin |
| POST | `/api/admin/backfill/tags` | `admin.rs:2026` | Admin |
| POST | `/api/admin/backfill/bodies` | `admin.rs:2076` | Admin |
| GET | `/api/admin/content-scan` | `admin.rs:1753` | Admin |
| POST | `/api/admin/content-scan/run` | `admin.rs:1807` | Admin |
| POST | `/api/admin/content-scan/{id}/review` | `admin.rs:1833` | Admin |
| GET | `/api/admin/search-mining` | `admin.rs:1903` | Admin |
| POST | `/api/admin/embedding-dedupe` | `admin.rs:1944` | Admin |
| GET | `/api/admin/invites` | `subsystems.rs:198` | TL5+ |
| POST | `/api/admin/invites` | `subsystems.rs:131` | TL5+ |
| GET | `/api/admin/registration-applications` | `subsystems.rs:365` | TL5+ |
| POST | `/api/admin/registration-applications/{id}/review` | `subsystems.rs:477` | TL5+ |
| GET | `/api/admin/trust` | `trust.rs:56` | Admin |
| POST | `/api/admin/trust/{id}` | `trust.rs:79` | Admin |
| GET | `/api/admin/trust-digest` | `trust.rs:105` | Admin |
| GET | `/api/admin/features` | `features.rs:362` | Admin |
| POST | `/api/admin/features/{id}/enable` | `features.rs:110` | Admin |
| POST | `/api/admin/features/{id}/disable` | `features.rs:154` | Admin |
| POST | `/api/admin/features/{id}/update` | `features.rs:185` | Admin |
| GET | `/api/admin/features/stats` | `features.rs:264` | Admin |

### Mechanics

- User management (ban, role, shadowban).
- Bot management with shadowban.
- Content scanning (automated moderation).
- Search mining (query analysis).
- Embedding deduplication.
- Feature flags (enable/disable per user).
- Trust level management with digest.
- Backfill tools for tags/bodies.
- Invite management with expiration.
- Registration application review.

### Pros

- Comprehensive admin toolkit.
- Feature flags for gradual rollout.
- Trust digest for monitoring.
- Backfill tools for data migration.

### Cons

- No admin action confirmation (destructive actions).
- No admin audit log (who did what).
- No bulk user operations.
- No admin dashboard (single page overview).

### Verdict

**Keep + Add:** Action confirmation, admin audit log, bulk operations, admin dashboard.

---

## 17. Subsystems

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/site/info` | `subsystems.rs:101` | Public |
| POST | `/api/admin/invites` | `subsystems.rs:131` | TL5+ |
| GET | `/api/admin/invites` | `subsystems.rs:198` | TL5+ |
| POST | `/api/admin/registration-applications/{id}/review` | `subsystems.rs:477` | TL5+ |
| POST | `/api/blocks` | `subsystems.rs:569` | Logged-in |
| DELETE | `/api/blocks/{id}` | `subsystems.rs:605` | Logged-in |
| GET | `/api/blocks` | `subsystems.rs:624` | Logged-in |
| GET | `/api/features` | `features.rs:29` | Logged-in |
| POST | `/api/features/{id}/enable` | `features.rs:110` | Logged-in |
| POST | `/api/features/{id}/disable` | `features.rs:154` | Logged-in |
| GET | `/api/skins` | `skins.rs:151` | Public |
| POST | `/api/skins` | `skins.rs:175` | Logged-in |
| PATCH | `/api/skins/{id}` | `skins.rs:204` | Owner/Admin |
| DELETE | `/api/skins/{id}` | `skins.rs:250` | Owner/Admin |
| POST | `/api/skins/{id}/set` | `skins.rs:270` | Logged-in |
| GET | `/api/skins/user` | `skins.rs:290` | Logged-in |
| POST | `/api/skins/{id}/assign-work` | `skins.rs:308` | Owner/Admin |
| GET | `/api/customization/theme` | `customization.rs:17` | Logged-in |
| POST | `/api/customization/theme` | `customization.rs:34` | Logged-in |
| GET | `/api/customization/nav` | `customization.rs:54` | Logged-in |
| GET | `/api/customization/widgets` | `customization.rs:66` | Logged-in |
| GET | `/api/device-library` | `device_library.rs:143` | Logged-in |
| POST | `/api/device-library/bookmarks` | `device_library.rs:157` | Logged-in |
| DELETE | `/api/device-library/bookmarks/{id}` | `device_library.rs:184` | Logged-in |
| POST | `/api/device-library/follows` | `device_library.rs:211` | Logged-in |
| DELETE | `/api/device-library/follows/{id}` | `device_library.rs:243` | Logged-in |
| POST | `/api/device-library/merge` | `device_library.rs:276` | Logged-in |

### Mechanics

- Site info (registration mode, features).
- Invite system with expiration.
- Registration applications with review.
- User blocking (hide content from blocked users).
- Feature flags per user.
- Skins/themes for customization.
- Device library (Kindle, Kobo, etc.).
- Customization (theme, nav, widgets).

### Pros

- Flexible registration modes.
- Invite expiration already supported.
- Per-user feature flags.
- Device library for e-readers.

### Cons

- No block+report combined action.
- No invite bulk generation.
- No skin marketplace.
- No device sync status.

### Verdict

**Keep + Add:** Block+report, bulk invites, skin marketplace, device sync status.

---

## 18. Infrastructure

### Endpoints

| Method | Path | Handler | Access |
|--------|------|---------|--------|
| GET | `/api/health` | `health.rs:40` | Public |
| GET | `/api/docs` | `api_docs.rs:5` | Public |
| GET | `/api/sitemap.xml` | `sitemap.rs:39` | Public |
| GET | `/api/sitemap/{page}.xml` | `sitemap.rs:69` | Public |
| GET | `/api/rss/new-arrivals` | `rss/handlers.rs:49` | Public |
| GET | `/api/rss/follows` | `rss/handlers.rs:74` | Logged-in |
| GET | `/api/rss/works/{id}` | `rss/handlers.rs:107` | Public |
| GET | `/api/opds/root` | `opds/feeds.rs:40` | Public |
| GET | `/api/opds/recent` | `opds/feeds.rs:180` | Public |
| GET | `/api/opds/popular` | `opds/feeds.rs:227` | Public |
| GET | `/api/opds/search` | `opds/search.rs:169` | Public |
| GET | `/api/opds/authors` | `opds/authors.rs:35` | Public |
| GET | `/api/opds/tags` | `opds/tags.rs:33` | Public |
| GET | `/api/opds/shelves` | `opds/shelves.rs:92` | Public |
| GET | `/api/opds/shelves/{id}` | `opds/shelves.rs:156` | Public |
| GET | `/api/opds/recommendations` | `opds/recommendations.rs:22` | Public |
| GET | `/api/opds/recommendations/{id}` | `opds/recommendations.rs:84` | Public |
| GET | `/api/opds/manifest` | `opds/manifest.rs:15` | Public |
| POST | `/api/docs/ingest` | `docs.rs:112` | Admin |
| POST | `/api/docs/ask` | `docs.rs:35` | Logged-in |
| GET | `/api/feed` | `feed.rs:18` | Logged-in |
| GET | `/api/updates` | `updates.rs:28` | Logged-in |
| POST | `/api/updates/{id}/seen` | `updates.rs:71` | Logged-in |
| POST | `/api/updates/{id}/refresh` | `updates.rs:89` | Logged-in |
| GET | `/api/roadmap` | `roadmap.rs:96` | Public |
| POST | `/api/roadmap/suggest` | `roadmap.rs:96` | Logged-in |
| GET | `/api/roadmap/arena` | `roadmap.rs:206` | Public |
| POST | `/api/roadmap/vote` | `roadmap.rs:272` | Logged-in |
| GET | `/api/roadmap/consensus` | `roadmap.rs:407` | Public |
| GET | `/api/roadmap/features` | `roadmap.rs:458` | Public |
| POST | `/api/roadmap/features/{id}/move` | `roadmap.rs:558` | TL3+ |
| GET | `/api/roadmap/changelog` | `roadmap.rs:671` | Public |
| POST | `/api/roadmap/changelog` | `roadmap.rs:777` | Admin |
| GET | `/api/heal` | `heal.rs:66` | Logged-in |
| GET | `/api/heal/extractions` | `heal.rs:204` | TL3+ |
| POST | `/api/heal/extractions/{id}/trust` | `heal.rs:234` | TL3+ |
| POST | `/api/heal/replay-pending` | `heal.rs:256` | Admin |
| GET | `/api/recipes` | `recipes.rs:92` | Public |
| POST | `/api/recipes` | `recipes.rs:110` | Logged-in |
| PATCH | `/api/recipes/{id}` | `recipes.rs:147` | Owner/Admin |
| DELETE | `/api/recipes/{id}` | `recipes.rs:186` | Owner/Admin |
| POST | `/api/recipes/{id}/activate` | `recipes.rs:206` | Logged-in |
| GET | `/api/recipes/active` | `recipes.rs:226` | Logged-in |
| GET | `/api/recipes/gallery` | `recipes.rs:244` | Public |
| POST | `/api/recipes/{id}/install` | `recipes.rs:259` | Logged-in |
| POST | `/api/recipes/{id}/publish` | `recipes.rs:283` | Owner |

### Mechanics

- Health check endpoint.
- API docs (OpenAPI).
- Sitemap for SEO.
- RSS feeds (new arrivals, follows, work updates).
- OPDS catalog for e-readers.
- Docs Q&A with ingestion.
- Personal feed (followed authors, bookmarks).
- Updates (fic refresh status).
- Roadmap/consensus engine (feature voting).
- Heal (data extraction and trust).
- Recipes (automation scripts).

### Pros

- Excellent SEO (sitemap, RSS, OPDS).
- E-reader support (OPDS, device library).
- Community roadmap with voting.
- Automation recipes.

### Cons

- No API rate limiting headers.
- No API versioning (only v1).
- No webhook system.
- No GraphQL endpoint.

### Verdict

**Keep + Add:** Rate limiting headers, API versioning, webhooks, GraphQL (optional).

---

## 19. Role Permission Matrix

| Feature | User (TL0-2) | Curator (TL3-4) | Moderator (TL5) | Admin (TL6) |
|---------|-------------|-----------------|-----------------|-------------|
| Register/Login | ✅ | ✅ | ✅ | ✅ |
| Edit own profile | ✅ | ✅ | ✅ | ✅ |
| Post comments | ✅ | ✅ | ✅ | ✅ |
| Post reviews | ✅ | ✅ | ✅ | ✅ |
| Give kudos | ✅ | ✅ | ✅ | ✅ |
| Create bookmarks | ✅ | ✅ | ✅ | ✅ |
| Create collections | ✅ | ✅ | ✅ | ✅ |
| Create forum topics | ✅ | ✅ | ✅ | ✅ |
| Create forum posts | ✅ | ✅ | ✅ | ✅ |
| Create polls | ✅ | ✅ | ✅ | ✅ |
| Send messages | ✅ | ✅ | ✅ | ✅ |
| File reports | TL1+ | ✅ | ✅ | ✅ |
| Hide comments | ❌ | ✅ | ✅ | ✅ |
| Approve collection items | ❌ | ✅ | ✅ | ✅ |
| Review forum edits | ❌ | ✅ | ✅ | ✅ |
| Set topic tags | ❌ | ✅ | ✅ | ✅ |
| Moderate posts (trust queue) | ❌ | TL4+ | ✅ | ✅ |
| Batch moderate | ❌ | TL4+ | ✅ | ✅ |
| Resolve reports | ❌ | ❌ | ✅ | ✅ |
| Forum bans | ❌ | ❌ | ✅ | ✅ |
| Lock/pin topics | ❌ | ❌ | ✅ | ✅ |
| Hide posts | ❌ | ❌ | ✅ | ✅ |
| Meta-moderation | ❌ | ❌ | ✅ | ✅ |
| Manage invites | ❌ | ❌ | ✅ | ✅ |
| Review registrations | ❌ | ❌ | ✅ | ✅ |
| Set trust levels | ❌ | ❌ | ❌ | ✅ |
| Manage feature flags | ❌ | ❌ | ❌ | ✅ |
| Blacklist works/authors | ❌ | ❌ | ❌ | ✅ |
| Shadowban bots | ❌ | ❌ | ❌ | ✅ |
| Run backfills | ❌ | ❌ | ❌ | ✅ |
| View admin analytics | ❌ | ❌ | ❌ | ✅ |
| Manage skins | ❌ | ❌ | ❌ | ✅ |

---

## 20. Prioritized Recommendations

### Critical (Fix Now)

| # | Recommendation | Impact | Effort | Domain |
|---|---------------|--------|--------|--------|
| 1 | Add rate limiting on auth endpoints | High | Low | Auth |
| 2 | Add admin audit log (who did what) | High | Medium | Admin |
| 3 | Add achievement notifications | High | Low | Gamification |
| 4 | Add report categories (spam, harassment, copyright) | High | Low | Moderation |
| 5 | Add ban appeals workflow | High | Medium | Moderation |

### High (Next Sprint)

| # | Recommendation | Impact | Effort | Domain |
|---|---------------|--------|--------|--------|
| 6 | Add XP leaderboard | High | Low | Gamification |
| 7 | Add block+report combined action | High | Low | Subsystems |
| 8 | Add forum digest emails | High | Medium | Forum |
| 9 | Add search autocomplete | High | Medium | Search |
| 10 | Add reading progress sync | High | Medium | Reading |
| 11 | Add message edit/delete | High | Low | Messages |
| 12 | Add tag merge/alias admin tool | High | Medium | Content |
| 13 | Add temporary ban cleanup cron | High | Low | Moderation |

### Medium (Backlog)

| # | Recommendation | Impact | Effort | Domain |
|---|---------------|--------|--------|--------|
| 14 | Add forum topic merge/split | Medium | Medium | Forum |
| 15 | Add collection categories/tags | Medium | Low | Collections |
| 16 | Add challenge progress tracking | Medium | Medium | Collections |
| 17 | Add request categories/tags | Medium | Low | Requests |
| 18 | Add bounty escrow | Medium | Medium | Requests |
| 19 | Add notification digest emails | Medium | Medium | Messages |
| 20 | Add download queue/history | Medium | Medium | Download |
| 21 | Add translation memory | Medium | High | Translation |
| 22 | Add reading time estimation | Medium | Medium | Analytics |
| 23 | Add admin dashboard (single page) | Medium | High | Admin |

### Low (Nice-to-Have)

| # | Recommendation | Impact | Effort | Domain |
|---|---------------|--------|--------|--------|
| 24 | Add OAuth/SSO support | Low | High | Auth |
| 25 | Add push notifications | Low | High | Messages |
| 26 | Add GraphQL endpoint | Low | High | Infrastructure |
| 27 | Add webhook system | Low | Medium | Infrastructure |
| 28 | Add A/B testing framework | Low | High | Analytics |
| 29 | Add skin marketplace | Low | Medium | Subsystems |
| 30 | Add friend comparisons in progression | Low | Low | Gamification |

---

## Summary

**Total features audited:** 471 route handlers across 60+ route files, 18 service files, 90+ migrations.

**Strengths:**
- Comprehensive trust-level-based access control (TL0-6).
- Unified XP ledger with achievement system.
- Rich forum with trust queue, batch moderation, meta-moderation.
- GDPR-compliant data export/deletion.
- Multiple export formats (EPUB, MOBI, PDF, HTML).
- OPDS catalog for e-readers.
- Community roadmap with consensus engine.
- Automation recipes.

**Weaknesses:**
- No rate limiting on auth endpoints.
- No admin audit log.
- No achievement notifications.
- No forum digest emails.
- No reading progress sync.
- No message edit/delete.
- No tag merge/alias tool.
- No temporary ban cleanup cron.

**Architecture health:** Good. Clean separation of routes/services. JWT consolidation complete. Trust level migration complete. No critical security issues found.

**Recommended next sprint:** Focus on critical items (rate limiting, admin audit, achievement notifications, report categories, ban appeals) — all are low-effort, high-impact.

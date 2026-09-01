# Transparency: Moderation Log & Usage Analytics

FicHub believes in **radical transparency** for moderation and **zero-PII**
privacy for analytics.

## Moderation Log (`/modlog`)

Every moderator, curator, and admin action is recorded in the `modlog` table
and readable by **any logged-in user** — you don't need to be staff to see
what staff did.

Recorded actions include:

| Action | What it means |
|---|---|
| `set_user_role` | A user's role was changed (e.g. to curator/admin) |
| `ban_user` / `unban_user` | A user was banned or unbanned |
| `approve_upload` / `reject_upload` | A manual fic submission was approved/rejected |
| `approve_translation` / `reject_translation` | An ML translation was approved/rejected |
| `hide_comment` / `delete_comment` | A comment was hidden or deleted |
| `forum_topic` / `forum_post` | Forum topic/post hidden, locked, pinned, deleted, or moderated (reason + scope recorded) |
| `forum_ban` | A forum scoped ban/timeout was applied or lifted (forum \| category scope) |
| `report_auto_hide` | A target was escalated to auto-hidden by overwhelming weighted reports (weight + reporter count recorded) |
| `trust_level_change` | A user's trust level was changed (from → to + reason recorded) |
| `blacklist_fic` / `blacklist_author` | A fic or author was blacklisted |
| `create_alias` / `merge_tags` / `delete_tag` | Tag curation actions |
| `resolve_flag` | A tag flag was resolved |
| `propose_fix` / `vote_fix` / `delete_body` | Curator body-fix workflow (peer-voted) |

Each entry shows the actor, the action, the target, when it happened, and any
details. Use the action filter to narrow the list.

API: `GET /api/modlog?limit=50&action=ban_user` (any logged-in user).

## Trust-Weighted Reports

User reports are community self-moderation with a safety valve:

- Every report is recorded with the reporter's **trust weight** (a
  Discourse-style 7-level axis — brand-new accounts cannot flag at all).
- A target's open reports are continuously re-aggregated into `pending`,
  `needs_admin` (contested), or `auto_hidden` (overwhelming weight).
  Escalations are logged to the modlog as `report_auto_hide` — nothing is
  hidden silently, and a human always makes the final call.
- **Community moderators** (trust level 5+, staff-designated from the weekly
  digest) can resolve reports, so moderation doesn't bottleneck on admins.
  All resolutions are attributed in the modlog.
- Admins get a **weekly moderation digest** (`GET /api/admin/digest`)
  surfacing contested targets and TL5 promotion candidates.

## Usage Analytics (`/admin/analytics` — admins only)

FicHub tracks **non-PII** usage: each browser gets a random anonymous client
ID (stored in localStorage — never an IP, never a cookie that identifies
you). Every request records a `view` (browsing) or `action` (download, vote,
comment, …).

Admins see:

- **Unique visitors** — daily (30d), weekly (12w), monthly (12m) bar charts.
- **Engagement** — active users (performed actions) vs view-only users, for
  today / 7d / 30d, with action events + total events + action rate.
- **Recent activity timeline** — the last 50 events (anonymous client, path,
  type, time).
- **Search analytics** — zero-result queries (search quality blind spots),
  trope popularity, search volume, and search→export conversion
  (searchers vs exporters, joined by anonymous client ID).

**Privacy:** nothing PII is stored or shown. The client ID is a random UUID
that cannot be tied to a person, and it's only used to count distinct
visitors and to join search→export funnels. Raw IPs are never recorded in
analytics.

API: `GET /api/admin/analytics` and `GET /api/admin/search-analytics`
(role ≥ 10).

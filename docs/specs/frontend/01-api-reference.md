# FicHub API Reference

Base URL: `http://localhost:8000`

All endpoints return JSON unless noted. Authenticated endpoints require `Authorization: Bearer <JWT>` header.

## Response Convention

Every response includes `"err": 0` on success. Error responses include `"err": <code>` and `"msg": "<message>"`.

## Authentication

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/register` | No | Register (honeypot-protected) |
| POST | `/api/auth/login` | No | Login, returns JWT + refresh token |
| POST | `/api/auth/refresh` | Refresh token | Exchange refresh token for new JWT |
| GET | `/api/auth/me` | Optional | Current user info |

**Register/Login body:** `{ username, password, email?, website?: "", form_opened_at?: "" }`
**Response:** `{ err: 0, user: { id, username, role, level, xp, rank, reputation, locale }, token, refresh_token }`

## Fic Metadata

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/epub?url=<url>&fmt=<epub\|mobi\|pdf\|azw3>` | No | Export fic as ebook |
| GET | `/api/epub/convert?url=<url>&fmt=<target>` | No | Convert cached fic |
| GET | `/api/meta?url=<url>` | No | Scrape/return fic metadata |
| GET | `/api/remote?url=<url>` | No | Proxy remote fetch |
| GET | `/api/health` | No | Health check (db, redis, version) |

## Search

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/search?q=<query>&page=<n>&limit=<n>` | No | Full-text search with facets |
| POST | `/api/search/ask` | No | AI-assisted search |
| GET | `/api/search/body?q=<query>` | No | Body/full-text search |
| GET | `/api/search/suggest?q=<query>` | No | Autocomplete suggestions |
| GET | `/api/search/similar/<work_id>` | No | Vector similarity search |
| GET | `/api/search/ask?q=<query>` | No | Ask-the-archive search |

**Search query supports:** `title:`, `author:`, `fandom:`, `tag:`, `rating:`, `status:`, `words:>5000`, `chapters:<100`, `NOT`, `AND`, `OR`, quoted phrases.

**Response:** `{ err: 0, results: [{ work_id, url_id, title, author, words, chapters, rating, status, ... }], total, page, limit }`

## Works & Fics

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/works/<id>` | Optional | Work detail (social features) |
| GET | `/api/works/random` | No | Random work |
| GET | `/api/works/<id>/similar` | No | Bookmark-based similar works |
| GET | `/api/works/<id>/reviews` | No | List reviews |
| GET | `/api/works/<id>/comments` | No | List comments |
| POST | `/api/works/<id>/comments` | Yes | Add comment |
| POST | `/api/upload` | Curator | Manual upload (multipart) |
| POST | `/api/upload/<url_id>/update` | Curator | Update fic |
| DELETE | `/api/upload/<url_id>` | Admin | Delete fic |

## Reader

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/reader/<url_id>` | No | Reader data (chapters, metadata) |
| POST | `/api/reading/status` | Yes | Update reading status |
| GET | `/api/reading/list` | Yes | Get reading list |
| POST | `/api/reading/record` | Yes | Record a read |

## Social Features

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/bookmarks` | Yes | Add bookmark (body: `{ url_id, shelf_id? }`) |
| GET | `/api/bookmarks` | Yes | List user bookmarks |
| DELETE | `/api/bookmarks/<work_id>` | Yes | Remove bookmark |
| GET | `/api/bookmarks/export.csv` | Yes | Export bookmarks CSV |
| POST | `/api/bookmarks/import` | Yes | Import bookmarks CSV |
| POST | `/api/ratings` | Yes | Rate work (body: `{ url_id, score }`) |
| GET | `/api/ratings/<work_id>` | Optional | Get ratings |
| POST | `/api/kudos/<work_id>` | Yes | Give kudos |
| DELETE | `/api/kudos/<work_id>` | Yes | Remove kudos |
| GET | `/api/kudos/<work_id>` | No | List kudos |
| POST | `/api/reviews` | Yes | Upsert review |
| DELETE | `/api/reviews/<id>` | Yes | Delete review |

## Recommendations

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/recommendations` | Optional | Generic trending recs |
| GET | `/api/recommendations/personal` | Yes | Personalized recs |
| GET | `/api/recommendations/strategies` | No | List available strategies |
| POST | `/api/recommendations/suggest` | Yes | Suggest a fic for recs |
| POST | `/api/recommendations/vote` | Yes | Vote on suggestion |
| GET | `/api/recommendations/votes` | Yes | List user votes |
| POST | `/api/recommendations/train` | Admin | Train rec engine |

## Tags & Fandoms

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tags?work_id=<id>&type=<type>` | No | Get tags for work |
| POST | `/api/tags/submit` | Yes | Submit new tag |
| POST | `/api/tags/vote` | Yes | Vote on tag |
| POST | `/api/tags/flag` | Yes | Flag inappropriate tag |
| GET | `/api/tags/search?q=<query>` | No | Search tags |
| GET | `/api/tags/autocomplete?q=<query>` | No | Tag autocomplete |
| GET | `/api/tags/resolve?q=<query>` | No | Resolve tag name to ID |
| GET | `/api/tags/<id>` | No | Tag detail |
| GET | `/api/forum/categories` | No | Forum categories |
| GET | `/api/forum/topics?category=&cursor=&limit=` | No | Forum topics (pinned first) |
| GET | `/api/forum/topics/{id}` | No | Forum topic detail |
| POST | `/api/forum/topics` | Yes (TL1+) | Create forum topic (title+body+tags, optional poll/scheduledAt/fic card) |
| POST | `/api/forum/topics/{id}/reply` | Yes | Reply to topic (quote/uploads/mentions) |
| GET | `/api/forum/search?q=&category=&tag=&author=&since=` | No | Forum FTS (Postgres tsvector, title-weighted) |
| GET | `/api/forum/tags/{tag}` | No | Topics by tag |
| GET | `/api/forum/groups` | No | Forum groups |
| POST | `/api/forum/flags` | Yes (TL2+) | Flag/report post (weighted by trust) |
| GET | `/ws/forum` | Yes (optional) | Forum WebSocket (live posts/typing/presence/notifs) |
| GET | `/api/forum/stream?channel=...` | No | Forum SSE fallback |
| GET | `/api/forum/messaging/rooms` | Yes | DM rooms |
| POST | `/api/forum/messaging/rooms/{id}/messages` | Yes | Send DM |
| GET | `/api/messages` | Yes | DM inbox (alias of forum messaging) |
| GET | `/api/users/{id}` | No | User profile — bibliography + forum topics/posts (unified at `/users/{id}`) |
| GET | `/api/fandoms` | No | List all fandoms |
| GET | `/api/fandoms/<slug>` | No | Fandom detail + works |

## User Profile

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/users/<id>` | Optional | Public profile |
| GET | `/api/users/<id>/badges` | No | User badges |
| GET | `/api/users/<id>/reading-stats` | No | Reading statistics |
| GET | `/api/users/<id>/streak` | No | Reading streak |
| GET | `/api/users/me/level` | Yes | Forum level + XP |
| GET | `/api/user/export` | Yes | Export user data |
| PUT | `/api/user/site-credentials` | Yes | Set site credentials |
| GET | `/api/user/site-credentials` | Yes | List credentials |
| DELETE | `/api/user/site-credentials/<domain>` | Yes | Delete credential |

## Progression & Customization

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/me/progression` | Yes | XP, level, unlocks |
| GET | `/api/me/prefs` | Yes | User preferences |
| PUT | `/api/me/prefs` | Yes | Update preferences |
| GET | `/api/me/layout/<page>` | Yes | Page layout config |
| PUT | `/api/me/layout/<page>` | Yes | Save page layout |
| GET | `/api/me/views` | Yes | Custom views |
| POST | `/api/me/views` | Yes | Create view |
| DELETE | `/api/me/views/<id>` | Yes | Delete view |
| PUT | `/api/me/views/<id>/toggle-pin` | Yes | Pin/unpin view |
| GET | `/api/me/theme` | Yes | Theme settings |
| PUT | `/api/me/theme` | Yes | Update theme |
| GET | `/api/nav` | Optional | Navigation items |
| GET | `/api/widgets` | Optional | Widget registry |

## Lists & Shelves

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/shelves` | Yes | Create shelf |
| GET | `/api/shelves` | Yes | List shelves |
| DELETE | `/api/shelves/<id>` | Yes | Delete shelf |
| POST | `/api/shelves/add` | Yes | Add work to shelf |
| DELETE | `/api/shelves/<shelf_id>/works/<work_id>` | Yes | Remove from shelf |
| GET | `/api/shelves/<shelf_id>/works` | No | List works in shelf |
| POST | `/api/lists` | Yes | Create list |
| GET | `/api/lists` | Yes | List user lists |
| GET | `/api/lists/<id>` | No | List detail |
| PATCH | `/api/lists/<id>` | Yes | Update list |
| DELETE | `/api/lists/<id>` | Yes | Delete list |
| POST | `/api/lists/<id>/items` | Yes | Add item |
| DELETE | `/api/lists/<id>/items/<work_id>` | Yes | Remove item |

## Feed & Follows

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/feed` | Yes | Personal feed |
| POST | `/api/follows` | Yes | Follow target |
| GET | `/api/follows` | Yes | List follows |
| DELETE | `/api/follows/<id>` | Yes | Unfollow |
| GET | `/api/follows/check/<target_type>/<target_id>` | Yes | Check follow state |
| GET | `/api/follows/followers/<user_id>` | No | List followers |

## Notifications

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/notifications` | Yes | List notifications |
| GET | `/api/notifications/unread-count` | Yes | Unread count |
| POST | `/api/notifications/read-all` | Yes | Mark all read |
| POST | `/api/notifications/<id>/read` | Yes | Mark one read |
| GET | `/api/notifications/preferences` | Yes | Get prefs |
| PUT | `/api/notifications/preferences` | Yes | Update prefs |

## Quests & Gamification

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/quests` | Yes | Daily/weekly quests |
| GET | `/api/badges` | No | Badge definitions |
| GET | `/api/leaderboard/curators` | No | Curator leaderboard |
| GET | `/api/leaderboard/curators/weekly` | No | Weekly leaderboard |
| GET | `/api/leaderboard/curators/monthly` | No | Monthly leaderboard |

## Requests Board

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/requests` | Yes | Create request |
| GET | `/api/requests` | No | List requests |
| GET | `/api/requests/<id>` | No | Request detail |
| DELETE | `/api/requests/<id>` | Yes | Delete request |
| POST | `/api/requests/<id>/answers` | Yes | Add answer |
| DELETE | `/api/requests/<id>/answers/<aid>` | Yes | Delete answer |
| POST | `/api/requests/<id>/answers/<aid>/vote` | Yes | Vote answer |
| POST | `/api/requests/<id>/upvote` | Yes | Upvote request |
| POST | `/api/requests/<id>/accept/<aid>` | Yes | Accept answer |
| GET | `/api/requests/<id>/candidates` | Yes | Candidate fics |

## Trending & Discovery

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/trending` | No | Trending fics |
| GET | `/api/trending/tags` | No | Trending tags |
| GET | `/api/trending/tag/<type_id>/<name>` | No | Trending by tag |
| GET | `/api/blind-date` | No | Random blind-date fic |
| GET | `/api/blind-date/reveal` | No | Reveal blind-date details |

## Updates

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/v1/updates` | Yes | Followed work updates |
| POST | `/api/v1/follows/<id>/seen` | Yes | Mark update seen |
| POST | `/api/v1/works/<url_id>/refresh` | Yes | Force refresh fic |

## Comments (v2)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/comment` | Yes | Add comment |
| DELETE | `/api/comment/<id>` | Yes/Mod | Delete comment |
| PATCH | `/api/comment/<id>/hide` | Mod | Hide comment |

## Translations & Locales

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/locales` | No | Available locales |
| GET | `/api/translations/<locale>` | No | UI translations |
| POST | `/api/works/<id>/translations` | Yes | Upsert translation |
| GET | `/api/works/<id>/translations/<locale>` | No | Get translation |
| GET | `/api/works/<id>/chapter-translations/<locale>` | No | Chapter translations |
| PUT | `/api/works/<id>/chapter-translations/<locale>` | Yes | Upsert chapters |

## Series & Authors

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/series/<id>` | No | Series detail |
| GET | `/api/authors/search?q=<query>` | No | Search authors |
| GET | `/api/authors/by-name/<name>` | No | Author by name |
| GET | `/api/authors/<id>` | No | Author profile |
| PUT | `/api/authors/<id>` | Curator | Update profile |
| POST | `/api/authors/<id>/socials` | Curator | Add social link |
| DELETE | `/api/authors/<id>/socials/<social_id>` | Curator | Remove social |

## Work Proposals (Merge/Split)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/work-proposals` | Yes | Create proposal |
| GET | `/api/work-proposals` | No | List proposals |
| GET | `/api/work-proposals/<id>` | No | Proposal detail |
| POST | `/api/work-proposals/<id>/vote` | Yes | Vote on proposal |

## Also Bookmarked

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/v1/works/<url_id>/also-bookmarked` | No | Users who bookmarked also bookmarked |

## Roadmap & Suggestions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/roadmap/suggest` | Yes | Suggest feature |
| GET | `/api/roadmap/arena` | Yes | Suggestion arena |
| POST | `/api/roadmap/vote` | Yes | Vote on suggestion |
| GET | `/api/roadmap/consensus` | No | Consensus results |
| GET | `/api/fic-suggestions` | Yes | List fic suggestions |

## Reports

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/reports` | Yes | Report content |
| GET | `/api/admin/reports` | Mod | List reports |

## Mod Log

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/modlog` | Yes | Moderation log |

## Analytics

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/analytics` | Optional | Site analytics |
| GET | `/api/analytics/user/<client_id>` | Optional | User analytics |

## Docs

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/docs/ask?q=<query>` | No | Search documentation |
| POST | `/api/admin/docs/ingest` | Admin | Ingest docs |

## Recipes (Export Templates)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/recipes` | No | List recipes |
| POST | `/api/recipes` | Yes | Create recipe |
| GET | `/api/recipes/active` | Yes | Get active recipe |
| GET | `/api/recipes/gallery` | No | Browse gallery |
| PUT | `/api/recipes/<id>` | Yes | Update recipe |
| DELETE | `/api/recipes/<id>` | Yes | Delete recipe |
| POST | `/api/recipes/<id>/activate` | Yes | Activate recipe |
| POST | `/api/recipes/<id>/install` | Yes | Install recipe |
| POST | `/api/recipes/<id>/publish` | Yes | Publish recipe |

## Kindle

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/send-to-kindle` | Yes | Send fic to Kindle |

## Blocks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/blocks` | Yes | Block user |
| GET | `/api/blocks` | Yes | List blocks |
| DELETE | `/api/blocks/<user_id>` | Yes | Unblock user |

## Features (Gating)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/features` | Optional | List features |
| GET | `/api/features/available` | Yes | Available for user |
| POST | `/api/features/<slug>/enable` | Yes | Enable feature |
| POST | `/api/features/<slug>/disable` | Yes | Disable feature |

## OPDS (Ebook Reader Catalog)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/opds` | No | Root catalog |
| GET | `/opds/new` | No | Recent fics |
| GET | `/opds/popular` | No | Popular fics |
| GET | `/opds/tags` | No | Tag types |
| GET | `/opds/tags/<type_id>` | No | Tags by type |
| GET | `/opds/tags/<type_id>/<name>` | No | Fics by tag |
| GET | `/opds/authors` | No | Author list |
| GET | `/opds/recommendations` | No | Recommendations |
| GET | `/opds/recommendations/popular` | No | Popular recs |
| GET | `/opds/search?q=<query>` | No | Search feed |
| GET | `/opds/shelves` | Yes | User shelves |
| GET | `/opds/shelf/<shelf_id>` | Yes | Shelf contents |

## RSS Feeds

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/feed.xml` | No | New arrivals |
| GET | `/feed/follows.xml` | Yes | Followed works |
| GET | `/feed/works/<url_id>` | No | Single work feed |

## Downloads

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/download/author?<params>` | Yes | Download all works by an author (AO3 + XenForo). Returns single file or ZIP. |
| GET | `/api/download/author/stream?<params>` | Yes | SSE stream of progress events during author batch download. |
| GET | `/api/download/series?<params>` | No | Download all works in an AO3 series |
| GET | `/cache/<type>/<url_id>` | No | Cached file download |

## Admin Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/admin/realtime` | Admin | Real-time stats |
| GET | `/api/admin/stats` | Admin | Site statistics |
| GET | `/api/admin/users` | Admin | User list |
| PUT | `/api/admin/users/<id>/role` | Admin | Set user role |
| PUT | `/api/admin/users/<id>/ban` | Admin | Toggle ban |
| GET | `/api/admin/bots` | Admin | Bot detection |
| POST | `/api/admin/bots/<id>/shadowban` | Admin | Shadowban bot |
| POST | `/api/admin/bots/<id>/unshadowban` | Admin | Unshadowban |
| GET | `/api/admin/content-scan` | Admin | Content scan results |
| POST | `/api/admin/content-scan/run` | Admin | Run scan |
| POST | `/api/admin/content-scan/<url_id>/review` | Admin | Review scan |
| POST | `/api/admin/dedupe/embeddings` | Admin | Run embedding dedupe |
| GET | `/api/admin/search-mining` | Admin | Search mining data |
| GET | `/api/admin/scraper-health` | Admin | Scraper health |
| GET | `/api/admin/moderation/queue` | Mod | Mod queue |
| GET | `/api/admin/moderation/comments` | Mod | Comment moderation |
| POST | `/api/admin/moderation/comments/<id>/hide` | Mod | Hide comment |
| POST | `/api/admin/moderation/comments/<id>/delete` | Mod | Delete comment |
| GET | `/api/admin/blacklist` | Admin | Blacklist |
| POST | `/api/admin/blacklist/fic` | Admin | Blacklist fic |
| POST | `/api/admin/blacklist/author` | Admin | Blacklist author |
| POST | `/api/admin/moderation/approve/<work_id>` | Mod | Approve upload |
| POST | `/api/admin/moderation/reject/<work_id>` | Mod | Reject upload |
| POST | `/api/admin/heal` | Admin | Heal extractions |
| GET | `/api/admin/analytics` | Admin | Admin analytics |
| GET | `/api/admin/search-analytics` | Admin | Search analytics |
| GET | `/api/admin/roadmap-consensus` | Admin | Roadmap consensus |
| GET | `/api/admin/translations` | Admin | Translation queue |
| POST | `/api/admin/translations/<id>/approve` | Admin | Approve translation |
| POST | `/api/admin/translations/<id>/reject` | Admin | Reject translation |
| POST | `/api/admin/translations/<id>/edit` | Admin | Edit translation |
| GET | `/api/admin/rating-checks` | Admin | Rating checks |
| POST | `/api/admin/rating-checks/<id>/verify` | Admin | Verify rating |
| PUT | `/api/admin/characters/<id>/score` | Admin | Fix tag score |
| POST | `/api/admin/auto-tag` | Admin | Auto-tag fic |
| POST | `/api/admin/auto-tag/backfill` | Admin | Backfill auto-tags |
| GET | `/api/admin/auto-tag/queue` | Admin | Auto-tag queue |
| POST | `/api/admin/auto-tag/approve/<url_id>/<tag_id>` | Admin | Approve auto-tag |
| POST | `/api/admin/auto-tag/dismiss/<url_id>/<tag_id>` | Admin | Dismiss auto-tag |
| POST | `/api/admin/bulk/refresh` | Admin | Bulk refresh |
| POST | `/api/admin/bulk/auto-tag` | Admin | Bulk auto-tag |
| GET | `/api/admin/fics/search` | Admin | Search fics for admin |
| POST | `/api/admin/invites` | Admin | Create invite |
| GET | `/api/admin/invites` | Admin | List invites |
| GET | `/api/admin/registration-applications` | Admin | List applications |
| POST | `/api/admin/docs/ingest` | Admin | Ingest docs |
| GET | `/api/admin/features` | Admin | Feature flags |
| PUT | `/api/admin/features/<slug>` | Admin | Update feature |
| GET | `/api/admin/features/stats` | Admin | Feature stats |

## Curator Endpoints (Level >= 50)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/curator/alias` | Curator | Create tag alias |
| POST | `/api/curator/merge` | Curator | Merge tags |
| DELETE | `/api/curator/tags/<id>` | Curator | Delete tag |
| PUT | `/api/curator/tags/<id>` | Curator | Update tag |
| GET | `/api/curator/flags` | Curator | List flags |
| POST | `/api/curator/flags/<id>/resolve` | Curator | Resolve flag |
| POST | `/api/curator/content/<url_id>/propose` | Curator | Propose content fix |
| GET | `/api/curator/content/proposals` | Curator | List proposals |
| POST | `/api/curator/content/proposals/<id>/vote` | Curator | Vote on fix |
| POST | `/api/curator/metadata/propose` | Curator | Propose metadata fix |
| GET | `/api/curator/metadata/proposals` | Curator | List metadata proposals |
| POST | `/api/curator/metadata/proposals/<id>/vote` | Curator | Vote on metadata fix |
| GET | `/api/curator/content/<url_id>` | Curator | Get fic body |
| DELETE | `/api/curator/content/<url_id>` | Curator | Delete fic body |
| POST | `/api/curator/authors/merge` | Curator | Propose author merge |
| GET | `/api/curator/authors/pending` | Curator | Pending merges |
| POST | `/api/curator/authors/approve/<id>` | Curator | Approve merge |
| POST | `/api/curator/authors/reject/<id>` | Curator | Reject merge |

## Proof-of-Work (Anti-Scrape)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/pow/challenge` | No | Get PoW challenge |
| POST | `/api/pow/solve` | No | Submit PoW solution |

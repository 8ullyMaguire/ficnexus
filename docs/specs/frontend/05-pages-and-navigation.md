# FicHub Pages & Navigation

## App Shell

The app is a **SPA (Single Page Application)** with client-side routing. No SSR, no prerendering. The SvelteKit static adapter outputs a single `index.html` with client-side JavaScript.

### Layout Structure
```
+layout.svelte (root)
├── TopNav (persistent navigation bar)
├── {slot} (page content)
├── BottomNav (mobile navigation)
├── CommandPalette (Cmd+K global search)
└── NotificationBell (floating, authenticated only)
```

### Navigation Items (TopNav)
- **Home** `/` — Dashboard with recommendations, trending, recent
- **Search** `/search` — Full-text search with facets
- **Download** `/download` — Export interface
- **Forum** `/forum` — Community discussions (trust TL1+ to post; TL0 sandboxed, forum is public-read)
- **Requests** `/requests` — Request board (level >= 40)
- **Admin** `/admin` — Admin panel (level >= 100)

### BottomNav (Mobile)
Simplified version of TopNav with icons.

## Page Catalog

### Public Pages (No Auth Required)

#### `/` — Home / Dashboard
- Personalized recommendations (if logged in)
- Trending fics
- Recent arrivals
- Quick actions (ask the archive, blind-date)
- Component: `HomeDashboard.svelte`

#### `/search` — Search
- Full-text search with query syntax
- Faceted filters (fandom, rating, status, words, chapters)
- Sort options (relevance, date, kudos, words)
- Search syntax help link
- Component: `SearchPage.svelte`

#### `/search/syntax` — Search Syntax Docs
- Visual guide to search operators
- Examples and tips

#### `/search/body` — Body Search
- Full-text search within fic content
- Longer queries, more results

#### `/works/[urlId]` — Work Detail (by source URL)
- Metadata (title, author, fandom, rating, status, words, chapters)
- Tags (clickable, voteable)
- Summary
- Actions (download, bookmark, rate, kudos)
- Similar fics
- Reviews section
- Comments section

#### `/read/[urlId]` — Web Reader
- Chapter-by-chapter reading
- Table of contents
- Reading progress tracking
- Font size/theme controls
- Component: `ReaderPage.svelte`

#### `/trending` — Trending
- Trending fics (overall, by fandom, by tag)
- Time-based filtering (day, week, month)

#### `/fandoms` — Browse Fandoms
- Alphabetical fandom list
- Work counts per fandom

#### `/fandom/[slug]` — Fandom Detail
- Fandom description
- Works in this fandom
- Related tags

#### `/blind-date` — Random Discovery
- Random fic reveal
- "I like it" / "Skip" interaction
- Reveal details on interest

#### `/leaderboard` — Rankings
- Curator leaderboard (by contributions)
- Reader leaderboard (by activity)
- Time filters (all-time, monthly, weekly)

#### `/ask` — Ask the Archive
- Submit a URL to scrape/import
- Paste fic URL, get it archived

#### `/recommendations` — Recommendations
- Strategy-based recommendations
- "People who liked X also liked Y"
- Personalized recs (if logged in)

### Authenticated Pages (Login Required)

#### `/download` — Export
- Format selection (EPUB, MOBI, PDF, AZW3)
- Customization options
- Download queue status

#### `/bookmarks` — My Bookmarks
- List of bookmarked fics
- Shelf organization
- Import/export (CSV)
- Component: `BookmarksPage.svelte`

#### `/feed` — Personal Feed
- Updates from followed works
- New chapters, new fics
- Component: `FeedPage.svelte`

#### `/requests` — Request Board
- Browse requests
- Submit request
- Add answers/votes
- Routes: `/requests`, `/requests/[id]`, `/requests/new`

#### `/settings` — Account Settings
- Profile (username, bio, locale)
- Preferences (recs personalization, notifications)
- Reading stats
- Streaks
- Kindle email
- Site credentials
- Routes: `/settings`, `/settings/recipes`

#### `/notifications` — Notifications
- Notification list
- Mark as read
- Preferences

#### `/users/[id]` — User Profile (bibliography + forum history + DM)
- Public profile info
- Badges
- Reading stats
- Reviews
- Books shelves (public)

#### `/work/[workId]` — Work Social
- Social features (reviews, comments, kudos)
- Reading progress
- Component: `WorkPage.svelte`

### Forum Pages

#### `/forum` — Forum Home
- Category listing
- Recent topics
- Component: `ForumLayout.svelte`

#### `/forum/[categorySlug]` — Category (topics, pinned first, paginated)
- Topic list within category
- Sort (recent, popular, unanswered)
- Component: `ForumCategory.svelte`

#### `/forum/board/[topicSlug].[topicId]` — Topic (canonical)
- Thread view (posts, replies) streamed live over `/ws/forum`
- Post actions (edit, delete, moderate), polls, tags, fic card `/works {id}`
- Reactions + typing + presence + unread (WS push)
- Component: `TopicThread.svelte`

#### `/forum/new` — New Topic
- Category selection
- Title + body editor
- Component: `NewTopic.svelte`

#### `/forum/[categorySlug]/[topicId]` — Topic (legacy, 301 → canonical `/forum/board/[topicSlug].[topicId]`)

### Admin Pages (Level >= 100)

#### `/admin` — Dashboard
- Site statistics
- Real-time metrics (30s poll)
- Recent activity

#### `/admin/stats` — Detailed Stats
- User growth
- Content metrics
- Search analytics

#### `/admin/users` — User Management
- User list with search
- Role assignment
- Ban/unban

#### `/admin/content-scan` — Content Scan
- Scan results
- Review actions

#### `/admin/moderation` — Moderation Queue
- Pending content
- Report queue

#### `/admin/comment-triage` — Comment Moderation
- Pending comments
- Hide/delete actions

#### `/admin/search-analytics` — Search Analytics
- Zero-result queries
- Popular searches

#### `/admin/scraper-health` — Scraper Status
- Per-site health checks
- Error rates

#### `/admin/auto-tag` — Auto-Tagging
- Tag queue
- Approve/dismiss actions

#### `/admin/bots` — Bot Detection
- Bot scores
- Shadowban management

#### `/admin/blacklist` — Blacklist
- Fic blacklist
- Author blacklist

#### `/admin/translations` — Translation Queue
- Pending translations
- Approve/reject/edit

#### `/admin/invites` — Invite Management
- Generate invites
- List pending invites

#### `/admin/registration-applications` — Applications
- Review registration applications

#### `/admin/roadmap-consensus` — Roadmap
- Feature consensus data

#### `/admin/rating-checks` — Rating Verification
- Pending rating checks

#### `/admin/reports` — Reports
- User reports
- Resolution actions

### Legacy Routes
- `/legacy/epub_export` → redirect to `/`
- `/changes` → redirect to `/`
- `/popular/` → redirect to `/`

## Routing Patterns

### Dynamic Routes
| Pattern | Example | Description |
|---------|---------|-------------|
| `/works/[urlId]` | `/works/ao3_12345` | Work detail by source URL (canonical) |
| `/read/[urlId]` | `/read/ao3_12345` | Reader by source ID |
| `/work/[workId]` | `/work/42` | Work by canonical ID |
| `/fandom/[slug]` | `/fandom/harry-potter` | Fandom by slug |
| `/forum/[categorySlug]` | `/forum/general` | Forum category |
| `/forum/board/[topicSlug].[topicId]` | `/forum/board/my-topic.123` | Forum topic (canonical) |
| `/users/[id]` | `/users/42` | User profile (bibliography + forum topics/posts + DM) |
| `/settings` | `/settings` | Settings (no params) |
| `/messages` | `/messages` | DM inbox (rooms) |
| `/users/[id]` | `/users/42` | (duplicate of profile — kept for table completeness) |

### Catch-All Route
`[...slug]/+page.ts` handles legacy/redirect routes. Any unmatched path gets redirected to `/`.

## Data Loading

### Page Loaders
Some pages use `+page.ts` for server-side data loading:
```typescript
// frontend/src/routes/works/[urlId]/+page.ts
export async function load({ params }) {
  const work = await getWorkDetail(params.urlId);
  return { work };
}
```

### Client-Side Loading
Most pages fetch data in `onMount` or `$effect`:
```svelte
<script>
  let data = $state(null);
  $effect(() => {
    loadData().then(d => data = d);
  });
</script>
```

### Caching
- **Browser cache:** Immutable hashed assets cached for 1 year
- **API cache:** No client-side caching layer (each page fetches fresh)
- **Service Worker:** Basic offline support via `sw.js`

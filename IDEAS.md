# FicHub — Feature Ideas

> Brain-dump of ideas for making FicHub indispensable to daily fic readers.
> Starred (⭐) items are high-confidence picks that 70%+ of users would genuinely use.

---

## 1. The Core Promise: Downloading Stories

- **⭐ Broader site support** – the holy grail. Wattpad, Quotev, Tumblr fics, fanfiktion.de, Webnovel… every new site brings a flood of users who currently have no good download tool.
- **⭐ Batch downloads** – select a whole series on AO3, an author's page, or tick boxes in your bookmarks and hit "download all as EPUB". One click, one zip file.
- **⭐ Send to e-reader** – email-to-Kindle (or Kobo's Dropbox integration) directly from FicHub. No more side-loading manually.
- **⭐ Download queue + progress** – for big batches, show what's happening, allow cancelling, resume after network hiccups.
- **Customisable EPUB** – choose cover style, font, include/exclude author notes, dedications, tags as chapter, etc. Power-users will tweak; casual users will just use a nice default.
- **"Download again" instantly** – once a fic is cached, re-download in any format without scraping again, and notify if the source updated.

---

## 2. Your Personal Library

- **⭐ Shelves / Collections** – not just one flat bookmark list. "Hurt/Comfort", "To Read", "Favourites", "MUST FINISH". Drag-and-drop between shelves.
- **⭐ Reading status** – Want to Read, Currently Reading, Completed, Dropped. Filterable.
- **⭐ Chapter tracking** – mark which chapter you're on, see a progress bar per fic. Sync this across devices (since you have accounts).
- **Private notes** – jot down why you liked it, a favourite quote. Stays inside FicHub, not on the public internet.
- **Import/Export bookmarks** – migrate from AO3 bookmarks, Goodreads shelves, or a CSV. No lock-in.
- **Search my library** – "what was that soulmate AU with the coffee shop I bookmarked last year?"

---

## 3. Discovery: Finding Your Next Obsession

- **⭐ Personalised recommendations** – "Because you loved *X*, you might like *Y*." Use your bookmarks and ratings, not just community votes.
- **⭐ "Similar fics" on every fic page** – algorithmically mined tags + user co-bookmarks.
- **⭐ Follow feeds** – follow an author, a fandom, a tag and get a feed of new fics. Even better: get a notification (email/web) when a tracked fic updates.
- **Public community shelves** – curated lists like "Best time-loop stories", "Found family with actual plot". High-quality, human-picked.
- **Trending / Popular this week** – what's getting bookmarked, rated, and commented on inside FicHub? Anonymised, of course.

---

## 4. Reading Inside FicHub (the "maybe I don't need an app" angle)

- **⭐ Built-in web reader** – some people just want to click and read, especially on mobile. A clean reader with font size, themes, and scroll-position memory.
- **⭐ Offline reading (PWA)** – let FicHub work as a progressive web app; open a downloaded fic, turn off wifi, keep reading.
- **Chapter navigation** – next/prev, dropdown, progress restoration on return.
- **Reading stats** – "you've read 300k words this month, 70% complete on your 'To Read' shelf." Gamify without the pressure.

---

## 5. Account & Sync (the glue)

- **⭐ True cross-device sync** – bookmarks, reading progress, shelves, notes. All instantly available whether you're on your phone, tablet, or desktop (as long as you're logged into the same FicHub instance).
- **Default preferences** – preferred download format, default tags to exclude, always download entire series, etc.
- **Data export/backup** – one button to download all your data, in case you ever move or the Pi dies.

---

## 6. Community (lightweight social)

- **⭐ Comments** – you already have threaded comments. That's huge. Maybe add simple likes on comments, and a notification when someone replies.
- **Following users** – see their public shelves and recommendations. Opt-in, privacy-first (nothing scary).
- **Ratings & short reviews** – beyond like/dislike, a 5-star system and a paragraph "why I recommend it" would help discoverability.

---

## 7. The Admin Side (for self-hosters)

- **Dashboard** – download stats, scraper health, cache size, user count.
- **Bulk actions** – re-scrape all fics by an author after a site change, or re-tag a batch.
- **Import from other FicHub instances / fichub.net** – if someone's moving from the old service, a migration tool would be killer.

---

## 8. Usage Analytics & Visitor Tracking

### Client ID System
- **Anonymous client IDs** – generate UUID on first visit, store in localStorage, send in X-Client-ID header. No login required.
- **Unique visitor tracking** – distinguish between total requests and actual unique users.
- **Return visitor rate** – track how many users come back within 7/30 days.

### Per-User Action Tracking
- **Download counts** – total EPUB/HTML/MOBI/PDF/AZW3/TXT/MD exports per user.
- **Search queries** – what users search for (anonymized).
- **Fic views** – which fic pages users visit.
- **Bookmark/rating/comment activity** – social engagement metrics.
- **Session duration** – approximate time spent on site (first to last request per session).

### Aggregate Analytics
- **Popular fics this week/month** – most viewed, most downloaded, most bookmarked.
- **Download format breakdown** – EPUB vs MOBI vs PDF preferences.
- **Site source breakdown** – AO3 vs FFN vs RoyalRoad traffic.
- **Peak usage times** – when users are most active.

### Admin Dashboard
- **Real-time metrics** – requests per minute, active users, cache hit rate.
- **Historical trends** – daily/weekly/monthly charts.
- **User retention** – cohort analysis of return visitors.
- **Export to CSV** – for manual analysis.

---

## Why these ideas hit the 70% mark

People come to FicHub to **get stories onto their devices**. The list above expands that core loop:

- **Get** → more sites, batch, queue, send to device
- **Organise** → shelves, status, notes, library search
- **Discover** → personalised recs, update notifications, similar fics
- **Read** → web reader, progress sync, offline PWA

Everything else (community, admin tools) sweetens the deal but isn't required for most. The starred items alone would turn FicHub from a handy converter into a central piece of someone's reading life.

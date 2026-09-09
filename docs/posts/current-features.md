# FicNexus — Current Features

> A comprehensive, descriptive list of everything FicNexus can do right now.
> Last updated: 2026-09-09

---

## Download & Reading

- **Multi-site downloader** — Download fanfiction from 107+ sites including AO3, FanFiction.net, RoyalRoad, Wattpad, Quotev, FimFiction, ScribbleHub, SpaceBattles, QuestionableCreative, and many more. Just paste a URL and the archivist fetches the story.
- **Multiple export formats** — Save stories as EPUB, HTML, MOBI, PDF, AZW3, plain text, or Markdown. Pick your format per download or set a default.
- **Built-in web reader** — Read directly in the browser with customizable fonts, themes, and layout. Remembers your place across sessions and suggests what to read next.
- **Reading progress sync** — Server-side tracking of which chapters you've read, so your progress follows you across devices.
- **Reading time estimation** — See how long a story will take to read based on word count and your average reading speed.
- **Reading goals** — Set daily or weekly reading targets and track your progress toward them.
- **Reading streaks** — Maintain a consecutive-days reading streak and see your longest streak in your stats.
- **Sequential reading mode** — Read through a series in order, with "next chapter" and "previous chapter" navigation.
- **Related works** — Discover similar stories based on embeddings, tags, and reading patterns.
- **Blind Date** — Hit the random-fic button for a surprise story from the archive, filtered by your preferences.
- **Offline reading (PWA)** — Install the site as a Progressive Web App and read downloaded stories offline.
- **Kindle send-to-device** — Send stories directly to your Kindle email address with one click.
- **Bookmarklet** — Install a one-click browser bookmarklet to download stories from any site page without copying URLs.

## Library & Organization

- **Bookmarks** — Save stories to your library with private notes, tags, and a "recommend" flag. Export/import bookmarks as CSV.
- **Shelves** — Create custom reading lists (e.g., "To Read," "Favorites," "Completed") and organize works across shelves.
- **Collections** — Curate public or private collections of works around a theme. Others can bookmark your collection to get updates.
- **Collection moderation** — Approve or reject items submitted to your collection. Set whether submissions require approval.
- **Challenges** — Run writing or reading challenges within a collection: signup, assign prompts, claim assignments, and track completions.
- **Kudos** — Give one-click "likes" to stories you enjoyed. One kudos per user per work.
- **Reviews** — Write in-depth reviews with 1-5 star ratings. Reviews appear on the work page and your profile.
- **Threaded comments** — Leave comments on works with nested threading. Hide your own comments or let moderators hide rule-breaking ones.
- **Favorites** — Mark authors or works as favorites to get notified of updates.
- **Follow authors** — Follow your favorite writers and see their new stories in your personal feed.
- **Follow works** — Subscribe to a story's updates (new chapters, edits) via your feed or RSS.

## Search & Discovery

- **Natural language search** — Search with plain English like "completed slow-burn Dramione over 50k, no major character death" and get relevant results.
- **Advanced search filters** — Use boolean operators (AND, OR, NOT), tag facets, word count ranges, completion status, rating, language, and more.
- **Full-text quote search** — Search across story bodies for exact quotes or phrases using PostgreSQL tsvector.
- **Typo tolerance** — Fuzzy matching catches misspelled tags and author names.
- **Search chips** — Save and reuse complex search queries as one-click chips.
- **Saved searches** — Save any search and get an RSS feed of new results matching your criteria.
- **Trending** — See what's popular right now across the whole site or within specific tags/fandoms.
- **Popular works** — Browse the most-read, most-kudos'd, and most-bookmarked stories.
- **Fandom landing pages** — Dedicated pages per fandom with top works, recent additions, and fandom stats.
- **Tag pages** — Browse all works tagged with a specific tag, with filtering and sorting.
- **Author pages** — View an author's full bibliography, social links, and stats.
- **Recommendations** — Get personalized suggestions based on your bookmarks, reading history, and similar users.
- **Also bookmarked** — See what other users who bookmarked a work also bookmarked.
- **Similar by bookmarks** — Find works with overlapping bookmark patterns.
- **Find fic** — Describe a plot or trope and let the AI archivist search for matching stories.
- **Random fic** — Get a random story from the archive, optionally filtered by your preferences.

## Gamification & Progression

- **XP system** — Earn experience points for posting, reacting, voting, moderating, and participating. XP accumulates toward level-ups.
- **100-level progression** — Advance through 100 levels, each requiring more XP than the last. Your level appears on your profile.
- **Achievements** — Unlock achievements for milestones like "first post," "100 reactions," "level 50," and more. Each unlock fires a notification.
- **Badges** — Earn weekly and monthly badges for top contributors in categories like posts, reactions, and bookmarks.
- **Leaderboards** — Compete on global, weekly, and monthly leaderboards for XP, badges, and curator activity.
- **Trust levels** — Advance from User (TL0) through Curator (TL3), Moderator (TL5), to Admin (TL6) based on participation and reputation.
- **Reputation** — Gain reputation from positive contributions. High reputation unlocks auto-promotion to curator.
- **Quests** — Complete reading quests (read 5 works this week, maintain a 7-day streak) for bonus XP.
- **Reading stats** — Track pages read, words consumed, works completed, and time spent reading.
- **Progression dashboard** — Customize your personal dashboard with draggable widgets showing your stats, quests, and recent activity.

## Community & Social

- **Forum** — Full discussion forum with categories, topics, posts, reactions, polls, and groups. Create topics, reply, react with emojis, and vote in polls.
- **Forum groups** — Create or join groups for private discussions. Groups have owners, managers, and members with different permissions.
- **Forum privileges** — Per-category access control based on trust level. Some categories are mod-only, others require TL3+ to post.
- **Forum moderation** — Trust queue (TL4+) for approving/rejecting posts. Batch moderation for handling multiple posts at once.
- **Meta-moderation** — TL5+ can review moderator decisions and vote on their fairness.
- **Forum tags** — Tag topics for discoverability. TL3+ can create and manage tags.
- **Forum search** — Full-text search across all forum content with tsvector indexing.
- **Forum RSS** — Subscribe to category or topic RSS feeds for updates.
- **Forum widgets** — Embeddable widgets showing recent topics, popular posts, and forum stats.
- **Direct messages** — Send private messages to other users, either 1:1 or in group rooms.
- **Message rooms** — Create group chat rooms with multiple members and shared preferences.
- **Notifications** — Get notified about replies, mentions, level-ups, achievements, and more. Mark as read or read-all.
- **Notification preferences** — Choose which notification types you want to receive and how.
- **Notification digests** — Get a daily or weekly digest of unread notifications instead of individual alerts.
- **User profiles** — Customizable profiles showing your bio, avatar, level, badges, trust level, and recent activity.
- **Pseudonyms** — Create multiple pseuds (pen names) for different fandoms or personas.
- **Follow system** — Follow users and see their updates in your feed. Exclude specific authors from your feed without unfollowing.
- **Block users** — Block other users to hide their content from your view.
- **Fic requests** — Post requests for specific stories you're looking for. Others can answer with URLs, vote on answers, and accept the best one.
- **Bounties** — Attach a point reward to a request. The answerer who fulfills it gets the bounty.
- **Request voting** — Upvote requests you want to see fulfilled, surfacing the most wanted stories.
- **Request candidates** — See which of your bookmarked works match open requests, so you can easily fulfill them.

## Curation & Moderation

- **Auto-tagging** — AI suggests tags for uploaded works based on content embeddings. Curators (TL3+) approve or dismiss suggestions.
- **Tag management** — Create, edit, and merge tags. Set tag types, categories, and synonyms.
- **Rating checks** — Automated checks flag works with mismatched ratings. Admins can verify and correct.
- **Content scanning** — Automated content scanning flags potentially problematic works for human review.
- **Work delete proposals** — Propose works for deletion with a reason. Community votes, and approved proposals auto-delete.
- **Author merge proposals** — Propose merging duplicate author profiles. Community votes on correctness.
- **Author social proposals** — Suggest social links (Twitter, Tumblr, etc.) for author profiles.
- **Reports** — Report posts, comments, or works with a category (spam, harassment, copyright, inappropriate, other) and reason.
- **Report status** — Check the status of your own reports (pending, needs_admin, resolved, dismissed).
- **Report queue** — TL5+ can view and resolve all reports. Auto-triage escalates high-weight reports.
- **Ban appeals** — Banned users can submit an appeal with a reason. Moderators review and accept or reject.
- **Modlog** — Public log of all moderation actions (bans, hides, role changes) with actor, action, and timestamp.
- **Shadowban** — Admins can shadowban bots or trolls, making their content invisible to others without their knowledge.
- **Blacklist** — Admins can blacklist works or authors, removing them from the archive.
- **Auto-moderation** — Configurable rules that automatically flag or hide content matching patterns (regex, keywords).
- **DMCA notices** — Handle takedown requests with a documented workflow and counter-notice process.
- **GDPR compliance** — Full data export (all your data in machine-readable format) and account deletion (right to be forgotten).
- **User consent** — Manage consent for data processing, cookies, and terms of service.

## Customization & Themes

- **Themes** — Choose from community-made themes or create your own with custom CSS.
- **Theme marketplace** — Browse, install, and publish themes in the extension marketplace.
- **Skins** — Apply different visual skins to your profile or the whole site.
- **Dashboard widgets** — Drag-and-drop customizable widgets on your personal dashboard.
- **Dashboard layouts** — Save and switch between different dashboard layouts.
- **Dashboard views** — Create multiple dashboard views for different purposes (reading, writing, moderating).
- **Navigation customization** — Customize your navigation bar with your most-used links.
- **Custom CSS** — Add custom CSS to your profile or globally (if admin).
- **Extensions** — Install community-built extensions for new functionality.
- **Recipes** — Create automation recipes (scripts) that run on triggers like "new work uploaded" or "level up."
- **Recipe gallery** — Browse and install recipes shared by the community.
- **Recipe marketplace** — Publish your recipes for others to use.
- **Command palette (Ctrl+K)** — Jump to any page, feature, or action with the keyboard-driven command palette.

## Translation & Localization

- **UI translations** — The interface is available in multiple languages, with community-contributed translations.
- **Work translations** — Translate works into other languages. Translations go through an approval workflow.
- **Auto-translation** — Ollama-powered machine translation for works, with human review.
- **Translation memory** — Reuse previous translations for consistency across works.
- **Translation coverage** — See which languages have the most translations and which works need translators.
- **Translator attribution** — Translators are credited on translated works.

## Infrastructure & Integrations

- **RSS/Atom feeds** — Subscribe to new arrivals, followed authors, specific works, or forum categories.
- **OPDS catalog** — Browse the archive as an OPDS catalog for e-reader apps (KyBook, Marvin, etc.).
- **ActivityPub** — Federate with the Fediverse. Follow FicNexus users from Mastodon, etc.
- **Webhooks** — Trigger external services on events (new work, new post, level up).
- **API docs** — Interactive OpenAPI documentation for the REST API.
- **Health check** — Public health endpoint for monitoring uptime and dependencies.
- **Sitemap** — XML sitemap for search engine indexing.
- **Analytics** — Personal reading stats, author stats, and site-wide analytics (admin).
- **Search analytics** — Admin view of popular search queries and zero-result searches.
- **Endpoint analytics** — Track API usage patterns and performance.
- **Anti-bot protection** — PoW challenge-response on registration and login to deter automated abuse.
- **Content warnings** — Works can have warnings (graphic violence, major character death, etc.) that users can filter by.
- **Positive-only surfaces** — Community areas are designed to be welcoming and constructive, with no downvoting or negativity features.
- **Rate limiting** — Redis-backed rate limiting on auth endpoints and API routes to prevent abuse.
- **Caching** — Multi-tier caching (in-memory, Redis, HTTP) for fast page loads.
- **Media uploads** — Upload images and files for forum posts, with size and type validation.
- **Drafts** — Save forum post drafts and resume editing later.
- **Scheduled posts** — Schedule forum posts to publish at a future time.
- **Import tool** — Import a NodeBB forum into FicNexus (users, categories, topics, posts, notifications) in a single atomic transaction.
- **Roadmap** — Public roadmap where users can suggest features, vote on priorities, and track progress.
- **Roadmap consensus** — Features with enough votes get promoted and scheduled for development.
- **Roadmap changelog** — See when features move from proposed → in progress → done.
- **Registration modes** — Open registration, invite-only, or application-based. Admins review applications.
- **Invite system** — Generate invite codes with expiration dates. Track who used your invite.
- **Feature flags** — Enable or disable features globally or per-user for gradual rollouts.
- **A/B testing** — Run experiments on feature variants to measure impact.
- **Multi-tenancy** — Run multiple FicNexus instances on a single deployment (if configured).
- **Self-hosted** — Everything runs on your own server. No cloud dependencies, no tracking, no ads.
- **Single binary** — The whole backend compiles to one executable for easy deployment.
- **Docker support** — Containerized deployment with Docker Compose.
- **Automated backups** — Scheduled database backups with retention policies.
- **TLS termination** — Built-in HTTPS support or behind reverse proxy.
- **Email notifications** — SMTP-based email for notifications, password resets, and digests.
- **Push notifications** — Browser push notifications for real-time alerts (requires configuration).
- **WebSocket** — Real-time updates for forum activity, notifications, and presence.
- **SSE** — Server-Sent Events for efficient one-way realtime streams.
- **Realtime presence** — See who's online and typing in forum rooms (when enabled).
- **Typing indicators** — Show when someone is typing in a direct message or forum room.

---

*This file is auto-generated from the codebase. For the authoritative feature audit with endpoint references, see `docs/ficnexus-feature-audit.md`.*

# Cool Features

Here's everything FicHub can do!

## Help & Keyboard Shortcuts ⌨

- **Command palette (`Ctrl+K` / `Cmd+K`)** — press anywhere on the site to
  jump to any page or search the docs. Type "dark harry" or "elo" and the
  palette surfaces the relevant docs section; hit Enter to open it.
- **`?` help links** — small `?` icons next to confusing things (search
  syntax, main-character attribute, the roadmap arena, Fic Requests answers,
  download formats) open the exact docs section in a popup without leaving
  the page.
- **Help modal** — every docs section opens in-app; use the **"Ask the
  docs"** box at the top to ask a question in plain English (e.g. *"how do
  I find dark harry fics?"*) and FicHub retrieves the relevant section for
  you. "Open full docs ↗" takes you to the whole page.
- **Docs link back** — every docs page has a **← Back to FicHub** bar plus
  contextual **▶ Try it** links that jump into the app with a prefilled
  action (e.g. a ready-made boolean search).

## Download Features

- **Multi-chapter support** — grabs all chapters at once
- **Multiple formats** — EPUB, HTML, MOBI, PDF, AZW3, TXT, Markdown
- **Works offline** — read without internet
- **Bookmarklet** — one-click download from any story page
- **107+ supported sites** — native adapters for every major fanfiction
  archive (AO3, FanFiction.net, RoyalRoad, ScribbleHub, FimFiction,
  Literotica, Wattpad, DeviantArt, SpaceBattles, and 95+ more), full
  FanFicFare adapter parity
- **Send-to-Kindle** — email an EPUB of any fic straight to your Kindle
  address
- **OPDS catalog** — subscribe to the archive from your e-reader app
  (`/opds/*`)
- **Site cache of gathered fics** — every scraped fic body is persisted on
  the attached drive (`BODY_CACHE_DIR`, e.g. `/public/literature/fichub/bodies`),
  so repeat exports are instant and the site is a growing fanfiction archive.
- **Your credentials for login-requiring sites** — on `/settings` you can
  optionally provide credentials for sites that require login (fanfics.me,
  fictionhunt, inkbunny, sofurry, dokuga) so downloads from those sites work.
  Opt-in only: you tick a consent box, credentials are encrypted at rest
  (AES-256-GCM) and auto-expire after 30 days. Your password is never
  returned by any endpoint and is only decrypted in memory at download time.

## Community & Transparency

- **Moderation Log** — every moderator, curator, and admin action (bans, role
  changes, upload approvals, comment removals, tag merges, body-fix votes,
  …) is recorded in the modlog and **visible to any logged-in user** at
  `/modlog`. Moderation is completely transparent.
- **Usage Analytics** — admins see non-PII aggregate usage (unique daily /
  weekly / monthly visitors, active vs view-only users, action timeline) at
  `/admin/analytics`. No IPs, no PII — anonymous client IDs only.

## Search Features (Better than AO3!)

- **Boolean operators** — `AND`, `OR`, `NOT`, and implicit AND between words
- **Quoted phrases** — `"slow burn"` matches the phrase as a whole
- **Exclusion** — `-angst` or `NOT angst` hides stories with that tag/term
- **Fielded search** — `title:harry`, `author:rowling`, `fandom:...`,
  `character:...`, `relationship:...`
- **Parentheses** — group terms like `(fluff OR humor) AND angst`
- **Faceted navigation** — click fandom/character/relationship/warning
  values in the sidebar to narrow results
- **Filter chips** — active filters shown as removable chips with × buttons
- **Relationship characters** — find fics whose relationships include the
  characters you name
- **Primary tag filter** — find stories where a tag is the MAIN tag
- **No warnings filter** — hide all stories with warnings
- **Comment count filter** — find stories people are talking about
- **Kudos filter** — find only popular stories
- **Tag ID search** — search by exact tag ID for precision
- **Chapter/word count filters** — find short or long stories
- **Full-text search** — searches titles, descriptions, AND tags
- **Search inside fic bodies** (`/search/body`) — quote search for lines of
  dialogue or prose ("fics where a character says X") across every cached
  fic body, with highlighted `<mark>` snippets in the results. No other
  scraper-only archive offers this.
- **Ask the Archive** — type a natural-language request like "completed
  slow-burn Dramione over 50k, no major character death" and the LLM turns
  it into the right search filters for you (see /ask). If an ask comes up
  empty, one click turns it into a Fic Request for the community to answer.

See [Finding Great Stories](./searching.md) for the full search syntax guide.

## Home Dashboard

- **Personalized recommendations** — sign in and bookmark (or download) a few
  fics; the home page shows "Recommended for you" stories with a
  "Because you bookmarked: …" label
- **Widget-composed dashboard** (rank 5+) — drag widgets around a CSS grid;
  edit mode with a widget registry; default layout factory for new users
- **Trending this week** — popular fics updated by reader activity
- **Compact download** — paste a story URL right on the home page to start
  a download (opens the full Download tab)

## Social Features

- **User accounts** — create a free account
- **Bookmarks** — save stories for later
- **5-star ratings** — rate stories from 1 to 5 stars; ratings feed your recommendations
- **In-depth reviews** — write a short review with your rating
- **Comments** — discuss stories with other readers
- **Community forum** — categories, topics, replies, follows, notifications, search, and moderation (see below)
- **Fic Requests** — post a prompt ("fics like X") and get works-only answers
  from the community — either a work id **or a pasted fic URL** (FicHub
  scrapes it and adds it to the archive). Upvote requests you'd like
  answered; you get notified when someone answers or you accept one.
  **Ask × Requests**: an Ask-the-Archive search that finds nothing turns
  into a request in one click, and every request page has an "Ask the
  Archive" box that finds matching fics already in the library — each with
  an "Add as answer" button.
### Fic Requests

Post a prompt ("fics like X") on the [requests board](/requests) and the
community answers with **works** — either a work id or a **pasted fic URL**
(FicHub scrapes it and adds it to the archive automatically). Upvote
requests you'd like answered; you get a notification when someone answers,
and the requester can accept the best answer to mark the request resolved.

The board is wired into **Ask the Archive**: an ask that finds no results
offers "Turn this into a request", prefilling the prompt; and a request
page's "Ask the Archive" box surfaces fics already in the library that fit
the prompt, each one click away from becoming an answer.

### Community Forum

The [forum](/forum) is where the community talks — categories, topics, and
replies, plus:

- **Follow topics** and get notified when someone replies
- **Search** the forum (full-text over topics + posts, with snippets)
- **Read state** — the forum remembers where you left off in each topic
  (unread dots/badges)
- **Moderation points** — trusted members earn points and can moderate
  posts from the queue (reports-as-signal, reason picker, curator
  fast-hide)
- **Metamoderation** — moderator actions are audited anonymously and the
  community votes on whether each was fair
- **Levels** — your level (see [Your Profile & Leaderboard](./profile.md))
  gates moderation powers
- **Stable topic URLs** — every topic gets a clean slug URL
  (`/forum/board/my-topic-title.42`) that doesn't change, plus invites +
  registration applications + user blocks

### Similar-fic suggestions

On any fic page, suggest other in-archive stories (or paste a link to be
scraped) that you think are similar; readers up/down-vote them so the best
matches rise to the top.

### Follows

Follow fics, authors, and users; get an updates feed.
- **Reading lists & shelves** — curate bundles and track your reading status
- **Series & author pages** — browse an author's bibliography (with an
  AO3-style filter sidebar: sort, complete/in-progress, word-count range,
  include/exclude tags) and series in order. Author pages show social links,
  which curators can propose via quorum vote rather than editing directly.
- **Reading time** — fic pages show an estimated reading time that accounts
  for dialogue density (dialogue reads faster than prose).
- **Smarter discovery** — Blind Date picks use your taste history, the reader
  suggests what to read next from community reading sequences, and fic
  requests without a seed work get semantic matches from the archive.
- **RSS/Atom feeds** — subscribe to new arrivals and your follows in your feed reader
- **Recommendations** — discover new stories
- **Leaderboards** — see top contributors
- **User profiles** — show your reading activity
- **Unified works** — the same story from different sites shows up as one page
- **Curator proposals** — community members can suggest merging duplicate works together
- **Work proposals** — community-driven merge/split proposals for duplicate
  works, peer-voted

## Reading Features

- **In-browser web reader** — read right in FicHub with font size/theme controls,
  chapter navigation, scroll-position memory, and a "Next Up" panel that shows
  the next story in a series (real next-in-series from `series_works`), the top
  community suggestion, and what readers also bookmarked.
- **Offline reading (PWA)** — open a story, then keep reading without internet
- **Blind Date** — let FicHub surprise you with a random story

## Technical Features

- **Fast** — built in Rust for speed
- **Private** — no tracking, no ads
- **Open source** — anyone can check the code
- **Self-hosted** — the community controls the server
- **Self-healing scrapers** — scrape failures are tracked per domain with
  health + retry (see [Self-Healing](./self-healing.md))

## Progression System

- **100 levels, 10 ranks** — earn XP by reading, downloading, bookmarking,
  rating, reviewing, commenting, and contributing to the community. Your
  level (0–100) rises with XP; your rank (Reader → Curator → Admin) gates
  access to features.
- **Ability Tree** (`/features`) — browse all 26+ gateable features, see
  which ones you've unlocked, activate/deactivate them individually.
  Onboarding banner for new users; level-up notifications.
- **Rank-gated features** — widgets (rank 5+), recipe builder (rank 4+),
  theme editor (rank 1+), custom views (rank 7+), advanced admin tools
  (rank 8+). Your rank unlocks access; you choose what to turn on.

## Extension Platform

- **Recipe Builder** (`/settings/recipes`) — compose custom recommendation
  blends by adjusting strategy weights (cooccur, tag_graph, embeddings, etc.),
  setting filters (min words, exclude warnings), and adding boosts (tag/author
  weight multipliers). Activate a recipe to override your personal rec
  engine. Publish recipes to the gallery for others to install.
- **Plugin Manifest** — unified `extensions` table for shareable recipes,
  themes, layouts, and views. Install from the gallery, version-tracked,
  level-gated.
- **Theme Design Tokens** (`/settings/theme`) — customize every visual
  aspect of FicHub: accent colour, background, surface, text, muted text,
  font family, corner radius, density. 5 presets: Default Dark, Default
  Light, High Contrast, Sepia, Dyslexia-Friendly. Reader theme is
  separate (font, width, line height). Import/export as JSON. Live preview.
- **Custom Saved Views** (rank 7+) — save any search as a named view, pin
  it to your dashboard, delete when you don't need it.

## Trust Levels

Trust measures *participation safety* — a Discourse-style axis separate from
your level and rank. It decides what you can write and how much your reports
weigh; it never grants moderation power.

- **7 levels** — TL0 New → TL1 Basic → TL2 Member → TL3 Regular → TL4 Elder →
  TL5 Community Moderator → TL6 Near-admin. Levels 0–4 are earned
  automatically from reading and participation (works/words read, days active,
  forum posts, reviews); TL5+ are granted by staff from the weekly digest.
- **What it gates** — TL0 (brand-new accounts) can read everything but cannot
  flag or publish; **TL2+ can publish** skins, recipes, and themes to the
  shared galleries; **TL5+ can resolve community reports**.
- **Your dashboard** (`/settings/trust`) — your current level, the metrics
  feeding it, exactly what the next level needs, and your capability gates.
  Also linked as **Trust** in the account bar.
- **Reports carry weight** — when you file a report, its weight scales with
  your trust. Reports are auto-triaged per target: `pending`, `needs_admin`
  (contested), or `auto_hidden` (overwhelming weight — always reviewed by a
  human, never applied silently).

## Extension Marketplace

One gallery (`/marketplace`) for every shareable customization object:
themes, skins, recipes, layouts, and views.

- **Browse & install** — filter by category, search by name, install in one
  click. Installs are deduplicated so popular extensions show real adoption.
- **Rate** — 1–5 stars per extension; the gallery ranks by rating and installs.
- **Remix** — copy any public extension into your own private draft and make
  it yours; slugs are unique per category so remixes never collide.
- **Publish (TL2+)** — share your own creations. Publishing is trust-gated so
  the gallery stays high-signal; drafts stay private until you publish.
### Roadmap consensus

Vote on which features to build next (see /roadmap): the public consensus
leaderboard shows how the community ranks every idea (Elo) plus the most
controversial proposals. Voting is **comparison-based** — you pick the
best (and worst) from a set of four ideas, and each comparison updates the
ideas' Elo ratings. Similar ideas are clustered automatically, so your
vote counts toward the whole cluster.
- **Transparent moderation** — the moderation log at `/modlog` shows every
  admin/curator action to any logged-in user
- **Curator/admin tooling** — auto-tag review (approve/dismiss ML tag
  suggestions), upload moderation, blacklist, bots, comment triage, scrapers,
  translation review (approve/reject/edit machine translations), and
  metadata correction (fix a work's title/author/status/description)

### LLM-powered curation tools (admin)

A local LLM (Ollama, nothing leaves the server) powers four admin tools that
keep the archive honest:

- **Content scan** (`/admin/content-scan`) — a nightly batch walks the cached
  fic bodies and classifies each as story prose vs. scrape noise, and detects
  violence/sexual/profanity the author didn't tag. Curators confirm or
  dismiss each flag; confirmed noise flags can be fixed via the body-cache
  curation flow. Keeps the `no_warnings` search filter honest.
- **Search demand mining** — zero-result search queries (last 30 days) are
  clustered into themes with suggestions, so curators see what readers want
  that the archive doesn't have yet ("people want Hermione-centric HP fics").
- **Crosspost dedupe** — finds same-author works with different titles and
  asks the LLM whether each pair is the same story renamed; confirmed
  matches become merge proposals for the curator vote queue.
- All three run on-demand from the admin UI or via API
  (`/api/admin/content-scan`, `/api/admin/search-mining`,
  `/api/admin/dedupe/run`).

### Chat bot (Discord / Telegram / Matrix)

**fanfic-archivist** brings FicHub into chat — Socrates-style recommendations
right where you already talk about fic:

- **Recommendations** — `!recs`, `!fresh`, `!gems`, `!roll` with pagination (`!next` / `!prev` / `!page 3`)
- **Cross-platform taste** — AO3, fanfiction.net and RoyalRoad unified into one works model; history on one site feeds recs across all of them
- **Slash commands** — `/search`, `/ask` (natural-language question about a fic's content), `/quote`, `/download` (EPUB/MOBI/PDF straight to chat)
- **Library** — bookmark, rate, kudos, block/unblock, updates on followed fics
- **Community** — fic requests board, roadmap voting, links back to your web profile

It's a thin client over the FicHub REST API — it never touches the archive
database directly. See [Integrations](./integrations.md) for invite links,
the linking flow, and the announcement post.

---

*Next: [FAQ](./faq.md)*

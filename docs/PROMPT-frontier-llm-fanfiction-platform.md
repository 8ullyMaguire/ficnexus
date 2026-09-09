# Prompt: Build the Ultimate Fanfiction Platform

You are building a complete, polished fanfiction archive and community platform from scratch. You have ONE day of access to a frontier LLM and want to produce:

1. **A complete design document** detailed enough for a junior developer to implement the entire site from scratch
2. **A clickable prototype specification** (detailed text descriptions of every page, every interaction, every state)

This is NOT a brainstorming exercise. You are producing a blueprint that a small development team could use to build the real thing.

## The Mission

Build a platform that **maximizes the amount of fanfiction written and shared** while maintaining quality and community health. Secondary goals:

1. **Steer users' tastes toward your own curatorial vision over time** — without making the site feel mono-thematic or niche
2. **Self-must be mostly self-governing via trust levels**
3. **Maximize fic creation** — every feature should ultimately serve getting more stories written and read
4. **Preserve creative freedom for pseudonymous authors** — multiple pseuds, privacy controls, compartmentalization
5. **Be more configurable than any existing platform** — users should be able to make it theirs

## What to Retain from the Existing Platform

The following features are battle-tested and should appear in your design:

### Core Content & Download
- Multi-site downloader (107+ fanfiction sites) — EPUB, MOBI, PDF, HTML, plain text
- Built-in web reader with customizable fonts, themes, reading progress sync
- Reading lists, shelves, bookmarks with tags and notes
- Multi-chapter support with sequential reading mode
- "Blind Date" random fic discovery
- "Find Fic" — reverse search by description/plot

### Community & Social
- Forum with categories, topics, posts, reactions, polls
- Forum groups with roles (owner, manager, member)
- Direct messages and group chat rooms
- Follow system with exclusions (mute specific authors)
- Collections (curated, with approval workflows)
- Challenges (signup, prompt assignment, claiming)
- Fic requests + bounties (reward points for fulfilling)

### Trust & Moderation
- Trust levels (TL0-TL6) — the primary gating mechanism
- Trust-weighted flagging (high-trust users' reports count more)
- Auto-triage for reports based on weight threshold
- Public modlog (transparency in moderation)
- Ban appeals workflow
- Meta-moderation (TL5+ review moderator decisions)

### Gamification & Discovery
- Reputation points (earned through participation)
- Achievements (milestone-based unlocks with notifications)
- Badges (weekly/monthly leaderboards)
- Natural language search with embeddings
- "Also bookmarked" and "Similar by bookmarks" discovery
- Trending/popular with time windows

### Infrastructure
- Self-hosted, single binary deployment
- Redis-backed rate limiting
- RSS/Atom feeds + OPDS catalog for e-readers
- ActivityPub federation
- GDPR-compliant data export + account deletion
- API with OpenAPI docs

## Critical Issues to Solve (Better Than AO3)

### Advanced Search System

AO3's tagging system has well-known limitations. Your search must solve ALL of these:

**1. Main vs. Side Character/Relationship Distinction**
- Tags should distinguish between "Character A is the protagonist" vs. "Character A appears in this fic"
- Relationship tags should distinguish "A/B is the central ship" vs. "A/B appears but isn't the focus"
- Users should be able to filter: "Show me fics where Character A is a MAIN character" vs. "Show me fics where Character A appears at all"
- Same for relationships: "A/B is the primary ship" vs. "A/B appears"

**2. Negative Ship Filtering Without Excluding Characters**
- Users should be able to filter OUT any relationship involving Character X WITHOUT filtering out Character X themselves
- Example: "I want fics with Character A but NOT any ship involving A" or "I want fics with A but not A/B specifically"
- This must work for both inclusion and exclusion

**3. Attribute-Based Main Character Search**
- Users should be able to search for attributes like "BAMF" that apply to the MAIN character, regardless of who that character is
- "Show me fics where the main character is BAMF" (not just any character)
- "Show me fics where the protagonist is a vampire" (regardless of which character)
- Combine with character filters: "Show me BAMF Character A fics where A is the protagonist"

**4. Common AO3 User Frustrations**
- **Tag wanking**: Too many tags make fics unfindable. Implement tag hierarchies and "primary tag" distinctions
- **Inconsistent tagging**: "Character A" vs "A (Character)" vs "A" — implement tag aliasing and canonicalization
- **Spoiler tags**: Users should be able to hide/show spoiler tags
- **Relationship vs. Character filters**: Currently conflated on AO3. Separate them.
- **Search by trope**: "Enemies to Lovers" should be searchable as a relationship dynamic, not just a tag
- **Search by completion status + word count + rating**: Should work together seamlessly
- **Search by "vibe"**: "Comfort fics," "angst," "fluff" — these should be first-class searchable dimensions
- **Search by character role**: "Mentor!Character A," "Parent!Character A" — role-based tagging
- **Search by setting**: "Coffee Shop AU," "Canon Divergence" — setting/trope combinations

**5. Tag Hierarchy and Canonicalization**
- Implement a tag taxonomy: Characters → Fandoms → Relationships → Tropes → Attributes
- Canonical tags with aliases: "Character A" = "A (Character)" = "A"
- Tag implications: Tagging "Enemies to Lovers" implies "Slow Burn" (configurable)
- Tag conflicts: "Major Character Death" vs. "Happy Ending" — warn but don't block
- Primary vs. secondary tags: Only primary tags affect search ranking

### Marketplace & Configurability

The site must be as configurable as possible:

**1. WASM Sandbox Limits Per Trust Level**
- Users can install extensions/widgets that run in a WASM sandbox
- Trust levels determine resource limits:
  - TL0: Minimal sandbox (basic widgets only)
  - TL2: Medium sandbox (custom CSS, simple scripts)
  - TL4: Full sandbox (complex widgets, API access)
  - TL6: Unrestricted (server-side mods)
- Each trust level has configurable CPU, memory, and API call limits
- Marketplace shows which trust level is required for each extension

**2. Trust/Reputation Perks (Without Limiting Free Users)**
- Free users get full functionality — perks are EXTRA, not restrictions
- Examples:
  - TL0: Basic profile, 5 reading lists, standard search
  - TL1: Custom CSS on profile, 10 reading lists
  - TL2: Custom CSS site-wide, 25 reading lists, priority support
  - TL3: Beta features, 50 reading lists, custom domains
  - TL4: Unlimited reading lists, API access, marketplace selling
  - TL5: Moderation tools, unlimited everything, revenue share
  - TL6: Admin tools, site configuration, profit sharing

**3. Marketplace Features**
- Extensions: New widgets, themes, search filters, reader modes
- Themes: Complete visual overhauls (not just CSS snippets)
- Integrations: Discord bots, RSS generators, e-reader sync
- Challenges: Premium challenge types with advanced features
- Writers' tools: Grammar checkers, plot generators, co-authoring tools

### Credit/Subscription System

For resource-intensive tasks when many users are active:

**1. Credit System**
- Users earn credits through participation (writing, reviewing, moderating)
- Credits can be spent on:
  - Priority scraping (download fics faster)
  - Priority rendering (generate EPUBs/MOBI faster)
  - Priority AI features (translation, summarization)
  - Extended storage (more works, larger files)
  - Compute resources (complex searches, custom reports)

**2. Subscription Tiers**
- **Free**: Full functionality, standard queue
- **Supporter ($3/month)**: Priority queue, 2x credits, custom CSS
- **Creator ($8/month)**: Full marketplace, 5x credits, custom domain
- **Patron ($15/month)**: Maximum priority, unlimited credits, revenue share

**3. Credit Economy**
- Credits regenerate over time (e.g., 10 credits/day for free users)
- Bonus credits for high-trust users
- Credits can be gifted/transferred between users
- No pay-to-win: credits only affect SPEED, not capability

## What You Have Freedom to Change

Everything else is up for redesign. Consider:

### Taste Steering Mechanism
How does the platform guide users toward your curatorial vision?
- Featured curator picks that rotate prominently
- "Taste match" scoring in recommendations
- Featured challenges that highlight themes you want to promote
- Curator's reading list as a discovery surface
- Algorithmic boost for content that matches your taste profile
- "Because you liked X" that bridges from popular to niche

### Fic Creation Maximization
What would actually get more people writing?
- Writing sprints with live counters
- Prompt generators (random trope + fandom + constraint)
- "First chapter" mentorship program
- Collaborative fic writing (multiple authors)
- Reading challenges that require writing a response fic
- "Fic in a weekend" events
- Lower-friction publishing (write in-browser, import from Google Docs)

### Trust Level Design
How should trust levels actually work?
- What actions earn trust? (writing, reviewing, moderating, flagging accurately)
- Should trust be global or per-category?
- How do you prevent trust farming?
- Should trust decay over time?
- Can trust be lost? How?

### Visual Design & UX
The existing platform is functional but not beautiful. You have freedom to:
- Design a completely different visual identity
- Choose different information hierarchies
- Prioritize different landing pages
- Invent new interaction patterns
- Target mobile-first vs desktop-first

## Deliverables

Produce a document with these sections:

### 1. Product Requirements Document (PRD)
- User personas (Reader, Writer, Moderator, Admin)
- User stories with acceptance criteria for EVERY feature
- Feature priority levels (P0 = must have, P1 = should have, P2 = nice to have)
- Success metrics for each feature

### 2. Information Architecture
- Complete site map (every page, every route)
- Navigation structure (primary, secondary, footer)
- User flows for key tasks:
  - Uploading a fic
  - Writing a fic in-browser
  - Searching for fics (all filter combinations)
  - Moderating (all moderation actions)
  - Installing an extension
  - Earning and spending credits

### 3. Trust Level Specification
- Detailed mechanics: how to earn, lose, and use trust levels
- Complete list of gates (what each TL unlocks)
- Moderation capabilities at each level
- Anti-farming measures
- Decay mechanics (if any)

### 4. Taste Steering Specification
- Algorithm details for recommendations
- How curator picks work
- How "taste match" scoring works
- How users can opt out of steering

### 5. Search System Specification
- Complete tag taxonomy
- All search filters and their interactions
- How main/side character distinction works
- How negative ship filtering works
- How attribute-based main character search works
- How tag canonicalization works
- Search result ranking algorithm

### 6. Database Schema
- ALL tables with columns, types, indexes, relationships
- Especially: users, works, tags, bookmarks, trust_levels, credits, extensions

### 7. API Design
- ALL endpoints with method, path, request body, response body
- Authentication and rate limiting
- Error codes and messages

### 8. UI Mockup Descriptions
For EACH of these pages, describe:
- Layout (header, sidebar, main content, footer)
- All interactive elements and their states
- All data displayed and its format
- Error states and empty states
- Mobile responsive behavior

Pages to describe:
- Home page (logged out)
- Home page (logged in)
- Search results page
- Advanced search builder
- Work page (reading)
- Work page (downloading)
- Writer dashboard
- New work form
- Forum board
- Forum topic
- User profile
- Moderation queue
- Extension marketplace
- Credit store
- Trust level dashboard
- Admin dashboard

### 9. Credit/Subscription Specification
- How credits are earned (complete list of actions and credit values)
- How credits are spent (complete list of services and costs)
- Subscription tier features and pricing
- Credit economy balance (inflation/deflation prevention)

### 10. Marketplace Specification
- Extension API surface
- WASM sandbox limits per trust level
- Extension review/approval process
- Revenue share model

### 11. Differentiation Section
- What you did differently from the existing platform and why
- What you did better than AO3 and why
- What tradeoffs you made and why

## Constraints

- Must be self-hostable (single server, no cloud dependencies)
- Must be implementable by a junior developer (or small team) in 6 months
- Must respect user privacy and pseudonymity
- Must be GDPR-compliant
- Must not require users to log in to read (reading is public)
- Must support at least 10,000 works and 1,000 users on modest hardware
- Must be mobile-responsive
- Must be accessible (WCAG 2.1 AA)

## Tone

Be opinionated. This is your chance to build the fanfiction platform you actually want to exist. If you think the existing platform got something wrong, say so and explain your alternative. If you think they got something right, say that too.

The output should be detailed enough that a junior developer could start implementing from it without asking clarifying questions. It should be structured so that you can later compare it feature-by-feature against the existing FicNexus platform.

---

**Begin your design document. Make it comprehensive. Make it implementable. Make it the best fanfiction platform imaginable.**

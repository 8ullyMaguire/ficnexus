# Prompt: Design the Ultimate Fanfiction Platform

You are designing a complete, polished fanfiction archive and community platform from scratch. You have ONE day of access to a frontier LLM and want to produce the most compelling, feature-complete design document + clickable prototype specification that you can compare against an existing open-source implementation.

## The Mission

Build a platform that **maximizes the amount of fanfiction written and shared** while maintaining quality and community health. Secondary goals:

1. **Steer users' tastes toward your own curatorial vision over time** — without making the site feel mono-thematic or niche. Users should discover your "taste" organically through recommendations, featured content, and community signals, but never feel like the platform is forcing a single genre/fandom down their throats.

2. **Self-moderation via trust levels** — The site should be mostly self-governing. Design a trust level system where the most active, constructive users naturally gain moderation capabilities, reducing the need for top-down admin intervention.

3. **Maximize fic creation** — Every feature should ultimately serve the goal of getting more stories written and read. This means writing prompts, challenges, community support, lowering barriers to publication, and making the reading experience so good that readers become writers.

4. **Preserve creative freedom for pseudonymous authors** — Multiple pseuds per user, privacy controls, the ability to compartmentalize fandoms/identities.

## What to Retain from the Existing Platform

The following features are battle-tested and should appear in your design in some form:

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
- RSS/Atom feeds + OPDD catalog for e-readers
- ActivityPub federation
- GDPR-compliant data export + account deletion
- API with OpenAPI docs

## What You Have Freedom to Change

Everything else is up for redesign. Specifically, consider:

### Taste Steering Mechanism
How does the platform guide users toward your curatorial vision without being heavy-handed? Ideas to explore:
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

### Monetization (Optional)
The existing platform is free and ad-free. You can keep it that way, or explore:
- Optional donations/tips
- Premium features (more shelves, custom CSS, etc.)
- Marketplace for writers (commission system)
- Print-on-demand for original works

### Visual Design & UX
The existing platform is functional but not beautiful. You have freedom to:
- Design a completely different visual identity
- Choose different information hierarchies
- Prioritize different landing pages
- Invent new interaction patterns
- Target mobile-first vs desktop-first

## Deliverables

Produce:

1. **Product Requirements Document (PRD)** — Complete feature specification with user stories, acceptance criteria, and priority levels

2. **Information Architecture** — Site map, navigation structure, user flows for key tasks (upload fic, write fic, moderate, discover)

3. **Trust Level Specification** — Detailed mechanics: how to earn, lose, and use trust levels. How moderation works at each level.

4. **Taste Steering Specification** — Exactly how the platform guides users toward your vision without being oppressive. Algorithm details if applicable.

5. **Database Schema** — Key tables and relationships (especially trust, reputation, content, moderation)

6. **API Design** — Core endpoints (REST or GraphQL), authentication, rate limiting

7. **UI Mockup Descriptions** — Detailed text descriptions of key pages (home, reader, upload, forum, profile, moderation queue) that a developer could implement from

8. **Differentiation Section** — What you did differently from the existing platform and why

## Constraints

- Must be self-hostable (single server, no cloud dependencies)
- Must be implementable by a small team (2-3 developers) in 6 months
- Must respect user privacy and pseudonymity
- Must be GDPR-compliant
- Must not require users to log in to read (reading is public)
- Must support at least 10,000 works and 1,000 users on modest hardware

## Tone

Be opinionated. This is your chance to build the fanfiction platform you actually want to exist. If you think the existing platform got something wrong, say so and explain your alternative. If you think they got something right, say that too.

The output should be detailed enough that a developer team could start implementing from it, and structured so that you can later compare it feature-by-feature against the existing FicNexus platform.

---

**Begin your design document.**

# Part 8: What's Next

## Chapter 26: Where to Go From Here

### You Did It

Take a deep breath. Look at how far you've come.

When you started this book, you had maybe a little Rust experience, or maybe none at all. You'd never built a web server. You'd never set up a database. You'd never written code that scraped a website, generated an EPUB file, or deployed a Docker container to a tiny ARM computer sitting on your desk.

And now? Now you've built all of that. Every single piece.

Let's look at what's inside you right now — the skills, the knowledge, the *understanding* of how real software is built:

**Chapter 1** — You learned what FicHub is and why we're building it. You set up your workspace, installed Rust, and ran your very first program.

**Chapters 2–3** — You set up your workshop. You wrote "Hello, World!" in Rust. You learned about `cargo`, about compiled languages, and about why Rust is special.

**Chapter 4** — You built your first web server with Axum. You sent HTTP requests, received responses, and understood how the web actually works.

**Chapter 5** — You created a configuration system that reads from environment variables. You learned why hardcoding secrets is a terrible idea.

**Chapter 6** — You built error handling that turns ugly Rust errors into friendly JSON responses. You learned the `?` operator, `thiserror`, and the art of making things fail gracefully.

**Chapter 7** — You learned the shared state pattern. Your database pool, your Redis client, your config — all available to every handler, all without global variables.

**Chapters 8–9** — You set up PostgreSQL. You wrote migrations. You connected SQLx to your database and saw compile-time checked queries catch mistakes before your code ever ran.

**Chapter 10** — You created models and queries. You read and wrote data. You learned how databases store stories, summaries, and export records.

**Chapter 11** — You added request logging so you can see every incoming request, track timing, and know exactly what's happening.

**Chapters 12–15** — This was the scraping core. You built a trait-based scraper design, implemented an AO3 scraper, created scrapers for FF.net and XenForo, and assembled them into a registry. You learned how the internet actually delivers web pages, and how to pull story content out of HTML like a pro.

**Chapters 16–19** — You built the export engine. EPUB generation from scratch, HTML bundle creation, a disk cache that prevents duplicate work, and a handler that ties it all together. Stories come in, files come out, cached and ready to serve.

**Chapters 20–22** — You set up Redis, built a token bucket rate limiter, and added middleware to block datacenter IPs. Your server protects itself from abuse without punishing real users.

**Chapters 23–25** — You containerized everything with Docker, cross-compiled for ARM64, and set up nginx as a reverse proxy with TLS termination. Your server is deployed, secure, and production-ready.

That's 26 chapters. 7 parts. One complete, working, deployed web server — built from scratch, with understanding at every level.

> 🎉 **You are no longer someone who "wants to learn Rust." You are someone who has built something real with Rust.**

That distinction matters. Reading about code is not the same as writing code. Watching tutorials is not the same than building projects. And now you've done it — you've built something that works, that serves real users, and that you understand inside and out.

### What We Didn't Cover

FicHub is a real project, and real projects are bigger than any one book can capture. There are features we didn't implement, techniques we didn't explore, and entire systems we only hinted at. Here are the big ones:

#### OPDS Catalog

OPDS stands for Open Publication Distribution System. It's a standard that e-readers like Kindle, Kobo, and apps like Calibre use to browse and download books. Think of it as an RSS feed for books — a machine-readable catalog that lets devices discover what's available.

Building an OPDS catalog for FicHub would mean your users could point their e-reader at your server and browse stories directly from the device. No downloading files manually, no transferring via USB. Just connect and read.

The nice thing is, an OPDS feed is just XML served at a specific URL. You've already built an XML-generating export engine (EPUB is just XML in a zip file). You've already built HTTP handlers that serve content. An OPDS catalog would be a natural extension — a new handler, a new template, a few new routes. The database queries you'd need are basically the same `SELECT` statements you wrote in Chapter 10, just returning different fields. This is the kind of project that feels massive until you realize it's just small pieces you already know, assembled in a new order.

#### Tagging System

Right now, FicHub stores stories and summaries. But what if you could tag stories? Add genres, characters, ratings, word counts, and let users filter by any combination? A tagging system would turn FicHub from a download tool into a personal library.

This would involve new database tables (a `tags` table, a `story_tags` junction table), new API endpoints for adding and searching tags, and probably a small frontend or CLI for managing them. You know how to build all of this — the database patterns are the same ones you learned in Part 3, and the API patterns are the same ones from Part 2.

#### Recommendation Engine

What if FicHub could recommend stories based on what you've already downloaded? "People who downloaded this story also downloaded..." That's a recommendation engine, and it's a surprisingly fun problem to solve.

The simplest approach is collaborative filtering — look at which stories are downloaded together and suggest the overlap. It requires a `downloads` table that tracks who downloaded what, plus some clever SQL queries or a small Rust function that computes similarity scores. No machine learning required for a basic version.

There's also content-based filtering — look at the tags, summaries, and metadata of stories you've liked, and find others with similar attributes. This one is simpler to implement and gives you a different kind of recommendation: "more of what you already enjoy." The combination of both approaches is how services like Netflix and Spotify work, scaled down to a manageable size.

#### Search System

If your FicHub instance has hundreds of stories, you need search. Full-text search — the kind that understands synonyms, handles typos, and ranks results by relevance.

You could use PostgreSQL's built-in full-text search (which is remarkably powerful), or add something like Meilisearch or Typesense as a separate service. Either way, search is a feature that transforms FicHub from a "download what you know" tool to a "discover what you love" platform.

The beauty of search is that it starts simple. A basic `WHERE title LIKE '%harry%'` query is search. Then you add full-text search with PostgreSQL's `tsvector` and `tsquery`. Then you add ranking. Then you add fuzzy matching for typos. Each step builds on the last, and each step makes the experience dramatically better. You've already built the data layer — search is just a new way to query it.

### Ideas for Extending FicHub

Beyond the missing features, here are some ideas for taking FicHub in new directions:

#### User Accounts

Right now, FicHub is public. Anyone can download anything. Adding user accounts would let you track per-user downloads, save reading lists, and customize the experience. You'd need:

- A `users` table with hashed passwords (never store plain-text passwords — ever)
- Authentication middleware (maybe JWT tokens, maybe sessions stored in Redis)
- A login flow and session management
- Per-user download history and reading lists

This is a great next project because it touches every layer of the stack — database, middleware, handlers, and configuration. You'll write new SQL migrations, new middleware, new handlers, and new tests. It's basically building a second, smaller project inside FicHub, and it's the kind of feature that makes a tool feel *complete* rather than *functional*.

#### Calibre Integration

Calibre is the Swiss Army knife of ebook management. It organizes your library, converts between formats, and syncs to e-readers. Integrating FicHub with Calibre would mean:

- An OPDS endpoint (so Calibre can browse your FicHub library)
- Metadata matching (so Calibre recognizes stories and fills in cover art and descriptions)
- A Calibre plugin that lets users search and download from FicHub without leaving Calibre

This is a more advanced project, but it connects your server to an ecosystem of millions of ebook readers. And the OPDS part of this is basically the first idea on our list, repurposed for a specific client. That's the power of building standards-compliant interfaces — one implementation serves many clients.

#### A Mobile App

A native mobile app — or even a progressive web app — that connects to your FicHub server and lets you browse and download stories on your phone. This would require:

- A frontend framework (React Native, Flutter, or even just a good responsive web app)
- API design for mobile consumption (maybe GraphQL instead of REST)
- Offline reading support (download stories for reading on the train)
- Push notifications for updates

Building a mobile app is a whole different discipline, but the backend skills you've learned here transfer directly. The API you've built is the API that a mobile app would consume. You already know how to design endpoints, return JSON, and handle errors. The mobile app is just a new skin on the same backend — and that's the beauty of building clean APIs.

#### A Web Frontend

Speaking of frontends — FicHub currently only has an API. A web frontend would make it accessible to anyone, not just people who know how to use `curl`. A simple frontend with a search bar, story cards, and download buttons could be built with:

- Svelte (lightweight, fast, fun to write)
- SvelteKit (for server-side rendering and routing)
- Tailwind CSS (for quick, good-looking styles)

You could even host the frontend inside the same nginx container that serves the API, making deployment a non-event. The static files would be just another set of files that nginx serves, with a catch-all route that sends unknown paths to `index.html` so the frontend can handle its own routing. It's a small change that makes the project feel like a complete product instead of a headless API.

### Contributing to Open Source

FicHub is a learning project, but the skills you've gained apply far beyond this book. The open source community is always looking for contributors, and now you're ready.

#### Finding Projects to Contribute To

Here's where to find projects that need help:

- **GitHub Explore** (`github.com/explore`) — browse trending projects, popular topics, and "good first issue" labels
- **Up For Grabs** (`up-for-grabs.net`) — curated list of projects with beginner-friendly issues
- **First Timers Only** (`firsttimersonly.com`) — specifically for people making their first open source contribution
- **Rust Project** (`github.com/rust-lang/rust`) — contribute to Rust itself
- **crates.io** — browse Rust libraries and find ones with open issues

#### Writing Good Pull Requests

A pull request (PR) is how you contribute code to someone else's project. Here's how to write one that gets merged:

1. **Read the contribution guidelines.** Every project has a `CONTRIBUTING.md` file. Read it. Follow it. This is the fastest way to have your PR accepted.

2. **Start with a small issue.** Don't try to rewrite someone's entire architecture on your first PR. Fix a typo. Add a test. Improve documentation. Build trust first.

3. **Write a clear PR description.** Explain what you changed, why you changed it, and how it works. Include screenshots if there are visual changes. Reference the issue number.

4. **Keep it focused.** One PR should do one thing. Don't mix bug fixes with feature additions. Reviewers hate PRs that try to do everything.

5. **Be patient.** Maintainers are busy. They might not review your PR for days or weeks. A polite ping after a week is fine. Demanding attention is not.

6. **Handle feedback gracefully.** If someone asks you to change something, change it. Don't argue. Don't take it personally. Code review is about making the code better, not about being right.

**The Ripple Effect**

When you contribute to open source, you learn from experienced developers, you get your code reviewed by people who are better than you (and you *want* people better than you reviewing your code), and you build a public track record that employers and collaborators can see.

Your first PR might be fixing a typo in a README file. That's okay. Everyone starts somewhere. The important thing is that you start. And once you've contributed one PR, the second one is ten times easier because you've already figured out the workflow — the fork, the branch, the commit, the PR, the review cycle. It's like learning any other skill: the first time is the hardest.

### Resources for Learning More

This book is a foundation, not the ceiling. Here's where to keep building:

#### Rust

- **The Rust Book** (`doc.rust-lang.org/book/`) — The official Rust book. If something in this book clicked but you want more depth, this is where to go. It covers Rust's ownership model, lifetimes, traits, and concurrency in detail.

- **Rust by Example** (`doc.rust-lang.org/rust-by-example/`) — Learn Rust through small, runnable examples. Great for reinforcing concepts.

- **Rustlings** (`github.com/rust-lang/rustlings`) — Small interactive exercises that help you practice Rust concepts. Do these alongside the book.

- **Exercism Rust Track** (`exercism.org/tracks/rust`) — Structured exercises with mentoring. Get your code reviewed by experienced Rust developers.

#### Web Development with Rust

- **Axum Documentation** (`docs.rs/axum`) — The web framework we used throughout this book. The docs include examples for authentication, websockets, and more.

- **SQLx Documentation** (`docs.rs/sqlx`) — The async database library. Learn about connection pooling, transactions, and advanced query patterns.

- **Tokio Documentation** (`tokio.rs`) — The async runtime that powers everything. Understanding Tokio means understanding how modern Rust web applications work.

#### Databases

- **PostgreSQL Tutorial** (`postgresqltutorial.com`) — Learn SQL from scratch or level up your existing skills.

- **The Art of PostgreSQL** (`theartofpostgresql.com`) — A deep dive into PostgreSQL for developers. Teaches you to think in SQL.

#### DevOps and Deployment

- **Docker Getting Started** (`docs.docker.com/get-started/`) — Learn more about containers and Docker Compose.

- **nginx Documentation** (`nginx.org/en/docs/`) — Understand the web server that sits in front of your application.

- **12-Factor App** (`12factor.net`) — A methodology for building modern web apps. You've already followed most of these principles without knowing it.

### The Bigger Picture

Here's something I want you to take away from this book: **FicHub is not the point.**

FicHub is a vehicle. It carried you through 26 chapters of learning, but the real cargo is everything you picked up along the way:

- How to design a system from scratch
- How to choose the right tool for the right job
- How to handle errors without panicking
- How to work with databases, caches, and message queues
- How to structure code so it's readable months later
- How to deploy software safely and monitor it in production
- How to think about security, performance, and maintainability

These skills apply to *every* software project you'll ever build. The next thing you build might not be a fanfiction server — maybe it's a personal blog engine, a chat application, a game server, or a tool for your school or job. But the patterns are the same:

- Handle requests with a web framework
- Store data in a database
- Cache expensive work
- Protect your server from abuse
- Deploy with containers
- Monitor what you've built

Once you see the pattern, you can't unsee it. Every web application is a variation on this theme. And now you know the theme.

### Advice for Your Next Project

You've got momentum. Don't lose it. Here's how to channel it:

**Build something you actually want to use.** The best learning projects are the ones you care about. If you don't care about fanfiction, build something you do care about. A tool for your hobby. A solution to a problem you have every day. Motivation matters more than technology. When you're stuck on a bug at 11 PM, the only thing that keeps you going is caring about the result.

**Start small and ship early.** Don't try to build the next Facebook. Build something that works, put it online, and use it. Then add features one at a time. The difference between "I have an idea" and "I have a deployed project" is shipping — getting it out the door, even if it's imperfect. FicHub shipped with just one scraper, no caching, and no rate limiting. It was still useful. Ship the MVP, then iterate.

**Don't be afraid to start over.** Your first version will be messy. That's fine. The second version will be cleaner because you'll know what matters and what doesn't. Many experienced developers expect to rewrite a project once before it becomes good. The second time around, you'll design the database differently, structure the code better, and make architectural decisions that you couldn't have made before you understood the problem.

**Read other people's code.** Open source is a library of solutions to every problem you'll face. Find a well-written Rust project and read it like a book. You'll pick up patterns, idioms, and techniques that no tutorial can teach. Some of the best Rust developers learned by reading the source code of libraries they used.

**Talk to people.** Join a Rust Discord server. Attend a local meetup. Post your project on Reddit. Other developers are incredibly generous with their time and knowledge, and having someone to ask questions to makes all the difference. You'll be surprised how many people are excited to see someone building things.

### Final Words

I want to tell you something, and I want you to hear it: **you built something real.**

Not a tutorial. Not an exercise. Not a toy. You built a web server that scrapes stories from the internet, packages them as beautiful ebook files, serves them through a secure HTTPS connection, and runs on a tiny ARM computer that costs less than a video game.

You did that.

Every `fn main()` you wrote, every `async fn` you composed, every `SELECT` query you ran, every Docker container you built, every nginx configuration line you wrote — that was you, learning, building, growing.

Software development is one of the most powerful skills a person can have. It's the skill of turning ideas into things that exist — things that work, things that help people, things that are real. And you have that skill now.

The code you wrote in this book is yours. It's in a repository on your machine. You can change it, extend it, break it, fix it, deploy it, and share it. It's not just code — it's proof that you can build something from nothing, and that's a superpower.

So what's next?

Whatever you want it to be.

Go build something. Go learn something. Go make something that didn't exist before you sat down and wrote it.

The world needs more builders. And now, you're one of them.

Welcome to the club. 🦀

### What We Built — Part 8 Summary

In this final part, we looked at where FicHub can go from here:

1. **Features We Didn't Cover** — OPDS catalogs, tagging systems, recommendation engines, and search systems are all natural extensions that you now have the skills to build.

2. **Ideas for Extending FicHub** — User accounts, Calibre integration, a mobile app, and a web frontend are all within reach.

3. **Contributing to Open Source** — How to find projects, write good pull requests, and join the community of people who build software together.

4. **Resources for Learning More** — The Rust Book, Axum docs, PostgreSQL tutorials, and dozens of other resources to continue your journey.

5. **The Bigger Picture** — FicHub was a vehicle for learning skills that apply to every software project. The patterns you learned — request handling, database design, caching, deployment — are universal.

---

**The complete FicHub architecture:**

```
Client → DNS → nginx (HTTPS + static) → FicHub (Rust/Axum)
                                            ↓
                                    PostgreSQL + Redis
                                            ↓
                                    AO3 / FF.net / XenForo
```

**The complete FicHub skill set you now have:**

- Rust programming (ownership, traits, async/await, error handling)
- Web frameworks (Axum, routing, middleware, state management)
- Database design (PostgreSQL, SQLx, migrations, models)
- Web scraping (HTML parsing, trait-based design, multiple scrapers)
- Export engines (EPUB generation, HTML bundles, disk caching)
- Security (rate limiting, datacenter IP blocking, TLS)
- DevOps (Docker, cross-compilation, systemd, nginx)
- Software architecture (configuration, logging, monitoring, deployment)

Thank you for reading. Now go build something amazing.

*— The FicHub Book, Part 8: What's Next*

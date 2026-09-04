# OUTLINE.md — Building FicHub: Full-Stack 50K Tutorial

> Learn to build FicHub from scratch, one feature at a time. Every part teaches a real feature: first the Rust/Axum backend route, then the SvelteKit frontend page that calls it. By the end you have a working fanfiction archive server.

**Target**: ~50,000 words, 12 parts, ~36 chapters. Kid-friendly. Real code from the FicHub repo.

**Stack**: Rust 2024 edition, Axum 0.8, SQLx 0.9, PostgreSQL, Redis, SvelteKit 5 (frontend), Vite, epub-builder, Tera.

---

## Part 1 — Welcome & Hello Server (5 chapters)

1.1 What is FicHub and why build it?  
1.2 Your first `cargo run`: the Axum server skeleton  
1.3 `src/main.rs` line by line — Tokio, Axum, router  
1.4 The `/health` endpoint: your first route  
1.5 Try It Yourself: add a `/ping` route and curl it  

## Part 2 — Database & Your First Model (4 chapters)

2.1 PostgreSQL + SQLx setup: `.env`, connection pool  
2.2 `migrations/001_initial.sql`: tables for works, authors, fic  
2.3 `src/db/models.rs`: SQLx `FromRow` structs  
2.4 `src/db/queries.rs`: your first query function  
2.5 Try It Yourself: write a query that returns a count  

## Part 3 — Getting a Single Work (5 chapters)

3.1 Backend: `GET /api/fic/:urlId` route in `src/routes/works.rs`  
3.2 SQL: lookup work by url_id, join author  
3.3 Frontend: SvelteKit `src/routes/fic/[urlId]/+page.svelte` shell  
3.4 Frontend: call the API with `fetch`, render work metadata  
3.5 Try It Yourself: add the work's synopsis to the page  

## Part 4 — Search Works (5 chapters)

4.1 Backend: `GET /api/search` — query params, SQLx `SELECT` with `LIKE`  
4.2 Backend: `src/search/parser.rs` — parsing query strings  
4.3 Backend: `src/search/builder.rs` — building SQL from parsed queries  
4.4 Frontend: `src/routes/search/+page.svelte` — search form + results list  
4.5 Try It Yourself: add a tag filter chip that toggles a URL param  

## Part 5 — Upload a Work (5 chapters)

5.1 Backend: `POST /api/upload` — Axum `Multipart` extraction  
5.2 Backend: validate the file, call fanfic-scrapers to parse metadata  
5.3 Backend: insert into `works` table, return the new url_id  
5.4 Frontend: `src/routes/upload/+page.svelte` — drop zone + progress  
5.5 Try It Yourself: show a success toast and link to the new work  

## Part 6 — Reading Lists & Bookmarks (4 chapters)

6.1 Backend: `GET/POST /api/saved-search` — list + create saved searches  
6.2 Database: `saved_searches` table, SQLx insert/select  
6.3 Frontend: `src/routes/follows/+page.svelte` — saved search cards  
6.4 Try It Yourself: delete a saved search with a confirmation dialog  

## Part 7 — EPUB Export (4 chapters)

7.1 Backend: `GET /api/export/:urlId` — epub-builder in Rust  
7.2 Backend: Tera template for the EPUB metadata cover  
7.3 Frontend: download button on the work page, link to `/cache/...`  
7.4 Try It Yourself: add a "TXT" export alongside EPUB  

## Part 8 — User Accounts & Login (5 chapters)

8.1 Backend: `POST /api/auth/register` — hash password, insert user  
8.2 Backend: `POST /api/auth/login` — session cookie, Redis rate limiter  
8.3 Backend: `src/routes/user_export.rs` — get current user profile  
8.4 Frontend: login/register page skeleton in SvelteKit  
8.5 Try It Yourself: protect a route so only logged-in users see it  

## Part 9 — Ratings & Reviews (5 chapters)

9.1 Backend: `POST /api/fic/:urlId/rate` — insert rating row  
9.2 Backend: `GET /api/fic/:urlId/reviews` — list reviews for a work  
9.3 Frontend: star rating widget on the work page  
9.4 Frontend: review form + review list below the work  
9.5 Try It Yourself: show the average rating as a number  

## Part 10 — Forum & Moderation (4 chapters)

10.1 Backend: `src/forum_core/` — forum engine crate overview  
10.2 Backend: `GET/POST /api/forum/:topicId/post` — create a post  
10.3 Frontend: `src/routes/forum/+page.svelte` — topic list + post thread  
10.4 Try It Yourself: add a "flag" button that calls the curator API  

## Part 11 — OPDS Catalog (3 chapters)

11.1 What is OPDS? Why FicHub speaks it.  
11.2 Backend: `src/routes/opds/` — XML catalog routes  
11.3 Try It Yourself: open an OPDS feed in a reader app  

## Part 12 — Deployment & Wrap-Up (3 chapters)

12.1 `justfile` recipes: build, migrate, run  
12.2 `Dockerfile` + `docker-compose.yml`: containerize the stack  
12.3 Where to go next: scrapers, recommender, curator tools  

---

*Each chapter ends with "Try It Yourself" so you type the code, not just read it. Backend chapters build the Rust route; frontend chapters build the SvelteKit page that calls it. You never build "all backend then all frontend" — every feature ships end-to-end before the next one starts.*

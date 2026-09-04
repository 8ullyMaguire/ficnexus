# FicHub Full-Stack 200K — Part-by-Part Inventory
Generated: 2026-08-26 21:38 CEST
Project root: ~/code/rust/fichub

## Tutorial Book Parts — Final Status

### Variant A: Full-stack (outdated structure)
These parts came from a different book structure and are EXCLUDED from the current assembly:
- 01-boot-server, 02-rust-backend, 03-scraping, 04-export-cache, 05-recommendations, 06-deployment, 07-frontend, 08-testing, 09-appendices, 10-advanced-backend, 11-advanced-scraping, 12-advanced-frontend, 13-advanced-db-security, 14-async-patterns, 15-frontend-deep-dive, 16-advanced-testing, 17-cross-cutting, 18-production, 19-scraping-export, 20-recommendations, 21-operations, 22-frontend-patterns, 23-router-middleware, 24-advanced-sql, 25-svelte-a11y, 26-testing-patterns, 27-export-cache-rate, 28-opds-tags, 29-router-advanced, 30-sql-patterns, 31-recommendations-advanced, 32-config-errors, 33-scraping-export, 34-deployment-security

### Variant B: Full-stack (canonical structure — 49 parts)
These are the parts included in BOOK_FULL_STACK_200K.md:

| # | Part | Words | Status |
|---|------|-------|--------|
| 01 | Welcome | 6363 | ✅ direct |
| 02 | Hello Route | 1742 | ✅ direct |
| 03 | Database Foundations | 2220 | ✅ direct |
| 04 | Get a Single Work | 1989 | ✅ direct |
| 05 | Search Works | 2492 | ✅ direct |
| 06 | Upload a Work | — | ❌ delegated (interrupted, never wrote) |
| 08 | User Accounts | 2499 | ✅ direct |
| 09 | Bookmarks | 1514 | ✅ direct |
| 10 | Ratings and Kudos | 1453 | ✅ direct |
| 11 | Reviews and Comments | 2321 | ✅ direct |
| 12 | Reading Lists | 1294 | ✅ direct |
| 13 | Follows and Feed | 2281 | ✅ direct |
| 14 | Series and Authors | 1450 | ✅ direct |
| 15 | Tags, Voting, Curation | 1486 | ✅ delegated (complete) |
| 16 | Forum | 2295 | ✅ direct |
| 17 | Ask the Archive | 2257 | ✅ direct |
| 18 | Bounties and Reputation | 1724 | ✅ direct |
| 19 | Reader | 1782 | ✅ direct |
| 20 | RSS and OPDS | 1622 | ✅ direct |
| 21 | — | — | ❌ delegated (never landed) |
| 22 | — | — | ❌ delegated (never landed) |
| 23 | Saved Searches | 3222 | ✅ delegated |
| 24 | Blind Date | 3535 | ✅ delegated |
| 25 | Fandoms/Trending | 3562 | ✅ delegated |
| 26 | Work Proposals | 3633 | ✅ delegated |
| 27 | Pseuds | 3501 | ✅ delegated |
| 28 | Skins/Customization | — | ⏳ re-dispatched (running) |
| 29 | Reading Quests | 4310 | ✅ delegated |
| 30 | Reading History | 2534 | ✅ delegated |
| 31 | Translations | — | ❌ delegated (never landed) |
| 32 | Send to Kindle | 3587 | ✅ delegated |
| 33 | Roadmap Consensus | 3709 | ✅ delegated |
| 34 | Ollama Recommender | — | ❌ delegated (truncated/503, not re-dispatched) |
| 35 | Recipe Builder | 5016 | ✅ delegated |
| 36 | Moderation/Admin | — | ❌ delegated (never landed) |
| 37 | Auth Deep Dive | 3721 | ✅ delegated |
| 38 | Rate Limiter/Redis | — | ⏳ re-dispatched (running) |
| 39 | Proof of Work | 3978 | ✅ delegated |
| 40 | Frontend Foundations | 3494 | ✅ delegated |
| 41 | Frontend Search UI | 3783 | ✅ delegated |
| 42 | Frontend Work/Fic Pages | 3420 | ✅ delegated |
| 43 | Frontend Dashboard/Profile | 4226 | ✅ delegated |
| 44 | Frontend Forum/Curator | — | ⏳ re-dispatched (running) |
| 45 | Frontend Social Features | 3873 | ✅ delegated |
| 46 | Frontend Collections/Shelves | 4036 | ✅ delegated |
| 47 | Testing Patterns | 3618 | ✅ delegated |
| 48 | Deployment | 4731 | ✅ delegated |
| 49 | Wrap-Up | 3633 | ✅ delegated (complete) |

### Summary
- On disk: 57 full-stack parts (canonical structure)
- Missing: Parts 06, 21, 22, 31, 34, 36 (never landed) + 28, 38, 44 (running)
- Skipped 07 in canonical sequence (no Part 7 in the canonical outline)

## Assembly State
- `BOOK_FULL_STACK_200K.md`: 132,711 words (50 parts, excluding the "numbered feature" variants)
- `FicHub_full-stack_200k.md`: 129,092 words (same content via pandoc)
- `FicHub_full-stack_200k.epub`: 913 KB, 184 internal files (35 XHTML chapters + assets) — TOC verified
- `PARTS_LOG.md`: this file

## EPUB Validation
- 35 XHTML chapter files in EPUB/text/ (covers Parts 01-49, minus missing 06, 21, 22, 31, 34, 36)
- nav.xhtml present with TOC
- mimetype, META-INF, content.opf all present
- Pandoc 3.1.3 used with --standalone --toc --css-style=block
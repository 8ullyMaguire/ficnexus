# Small-Scale AI Features for FicHub — Orange Pi 5 (4GB) Plan

> Target hardware: **Orange Pi 5, 4GB RAM**. Rules out request-time LLMs and
> heavy HuggingFace pipelines. Greenlit: `fastembed`/ONNX models under ~200MB,
> `pgvector` with binary quantization, traditional ML (k-means, bandits, Markov),
> and tiny LLMs (Qwen 1.5B/3B, Llama 3.2 3B via Ollama) **only in background
> batches**.
>
> Pair with: `docs/plans/recommendation-engine-plugin-system.md`,
> `docs/plans/search-v2-advanced-search.md`, `docs/plans/translation-everything.md`,
> `src/recommender/` (existing bandit/embeddings/sequential strategies),
> `src/server.rs` recommendation routes.

## The "Orange Pi AI Stack" — three rules

1. **At request time** — only `fastembed`/`ort` (ONNX) models under 200MB,
   traditional ML (k-means, bandits), and `pgvector` binary search. No LLM.
2. **Background batches** — heavier tasks (mood classification, tag wiki
   drafting, translation) run via nightly/hourly cron jobs. Load model into
   RAM → process queue → fully dispose of model before next job. Reuse the
   `qa/triage.js` pattern (ollama llama3.2:3b, single-shot, no daemon).
3. **Graceful degradation** — if Ollama sidecar is down or RAM spikes,
   features like *Ask-the-Archive* or *Tag Wiki Drafting* silently fall back
   to keyword search or blank templates. Core read/download loop never
   crashes.

---

## 🔍 Search & Discovery (Vector & Semantic)

### 1. Natural Language to Tag Translation (`/ask`)
Use a tiny Seq2Seq ONNX model or background 3B LLM (Ollama cron) to convert
conversational queries ("sad draco malfoy slow burn") into structured advanced
search facets (`include_tags`, `character`, `relationship`).

- **Request-time**: keyword fallback always works.
- **Background**: cron job pre-computes a query→facet cache (`ask_cache`
  table) keyed by normalized query string; cache hit = no LLM call.
- Reuses existing `POST /api/recommendations/suggest` route shape.

### 2. Semantic "More Like This" Search
Generate text embeddings for summaries/titles using `all-MiniLM-L6-v2` (via
`fastembed`, ~200MB RAM) and store them in `pgvector` with **binary
quantization** (48 bytes per row) for semantic search without blowing up RAM.

- Extends the existing `recommender/embeddings.rs` strategy.
- Migration: add `summary_embedding BIT(48)` column to `works`; backfill
  via background job (`recommendations/train` route already wired).
- Powers "fic-level: more like this" link on work pages.

### 3. Personalized Search Chip Re-ranking
Compute lightweight dot-product between logged-in user's recent bookmark
embeddings and global trending search queries to swap "Popular" chips for
"For You" chips in real-time.

- **Request-time only**: pure pgvector binary dot-product + bookmark
  in-memory summary (cached in Redis per-user, 1h TTL).
- New route: `GET /api/search/chips?user_id=…` — returns ordered chip list.

### 4. "Fic Requests" Auto-Matching
When user posts a request on the prompt board, use semantic similarity
(MiniLM embeddings) to instantly surface top 3 existing works that match
before any human archivist answers.

- Hooks into the existing fic-requests forum (`FORUM-API-CONTRACT.md`).
- Embed request body once on submit; `pgvector` returns top-k works.
- Archivist sees the matches pre-filled as "suggested answers".

### 5. Taste Clustering ("People Like You")
Run lightweight **k-means clustering** over the bookmark/tag affinity
graph to segment users into distinct "taste profiles" for collaborative
filtering. Negligible RAM vs deep learning.

- Nightly batch: rebuild `user_clusters` table from `bookmarks × tags`.
- Used by the existing `recommender/clusters.rs` strategy (currently
  dormant — wire it live with cluster IDs from this batch).
- New route: `GET /api/users/{id}/taste-cluster` returns cluster label
  + adjacent profiles.

### 6. Contextual Bandits for "Blind Date" Discovery
Use **Thompson sampling** (lightweight ML exploration algorithm) to
balance safe recommendations vs. random "hidden gems" in the Blind Date
widget, learning from click-through rates.

- Reuses the existing `recommender/bandit.rs` strategy (currently
  ε-greedy, opt-in). Swap to Thompson sampling + arm-keyed per
  Blind Date session.
- Add `blind_date_clicks` event to `usage_events`; bandit reward = click
  within 60s.

---

## 🛡️ Content Moderation & Curation

### 7. Content Shield & Spice Triage
Run lightweight, distilled BERT ONNX classifier over chapter text to
auto-detect and collapse explicit/gory passages behind a "tap to reveal"
warning, keeping public browsing safe.

- Background cron: scan new chapters on import, store
  `chapter_spice_spans` table (start_offset, end_offset, spice_score).
- Request-time: render UI with `<details>` collapses; "tap to reveal" is
  pure HTML, zero AI in the hot path.
- Opt-in per-work via a `content_shield_enabled` flag in `work_prefs`.

### 8. Auto-Tagging & Tag Wrangling Assistant
Use FastText or cosine similarity on tag embeddings to suggest canonical
tag merges, synonyms, or missing tags to admins during upload moderation.

- Nightly batch: compute tag co-occurrence + embedding similarity; emit
  `tag_merge_suggestions` table (tag_a, tag_b, score, kind).
- Hook into existing `curator/content` workflow — admins see a
  "Suggested merges" panel.
- Uses `fastembed` for tag text embeddings (single shared model).

### 9. Main-Character Attribute Extraction
Use lightweight Named Entity Recognition (NER) model to auto-extract
character roles and traits from summaries, populating the "Main-Character
Attribute" advanced filter.

- ONNX NER model (~150MB) loaded only during the nightly work-import
  job.
- Output: `work_character_attributes(character, role, traits[])` table;
  surfaces in the v3 advanced-search facets.

### 10. Tag Wiki Auto-Drafting
Use a tiny background LLM (Ollama cron) to read top 5 most-bookmarked
fics of a new trope and generate a 2-sentence draft definition for the
community Tag Wiki.

- Cron: `0 3 * * *` — process any new tag with >50 bookmarks but no wiki
  entry. Output goes to `tag_wiki_drafts` (status=`pending_review`).
- Admin UI at `/curator/tag-wiki` — approve/edit/reject.
- LLM is disposed (model unloaded) before the next cron tick.

### 11. Self-Healing Scraper Telemetry
Use a tiny pattern-matching ML model (or a tiny LLM fallback) to analyze
HTML DOM trees on failing scraper targets to auto-locate and extract
broken author-profile links.

- Augments the existing `AGENT_ENABLED` self-healing loop (M1/M2/M3 in
  `AGENTS.md`). When a structural scraper failure occurs, the agent
  loads the HTML snapshot + runs the tiny classifier to locate the
  author profile node before invoking the on-the-fly scraper creator.

---

## 📖 Reader Experience & Social

### 12. Quote Card Extraction
Use extractive summarization (TextRank or small BERT sentence scorer)
to auto-identify the most "quotable" lines in a chapter for rendering
as shareable Instagram/Twitter cards.

- Background batch per chapter (on scrape-complete).
- Output: `chapter_quotes(chapter_id, text, score, position)` — top 5.
- New endpoint: `GET /api/reader/{work_id}/ch/{n}/quotes` for the card
  generator UI.

### 13. Fic-Level Mood & Tone Classification
Classify emotional tone (angsty, fluffy, dark, hurt/comfort) of a work
by running lightweight sentiment/emotion ONNX model over tags,
description, and first chapter.

- Nightly batch (or on-import). Stores `work_mood(work_id, label,
  score)` — multi-label allowed.
- Powers mood chips on work pages + a "mood filter" on advanced search.

### 14. True Reading Time Estimation
Move beyond "words/minute" math by using a small regression model that
factors in dialogue density, vocabulary complexity, and formatting to
predict *actual* reading time.

- Linear regression or small decision tree — exported to ONNX (~5MB).
- Request-time safe. Replaces the simple heuristic on work cards.
- Trained nightly from `reading_sessions` table (user_id, work_id,
  start_ts, end_ts, pct_read).

### 15. Markov "Next Up" Sequencer
Use a simple **Markov chain** sequential model based on reader position,
prequels, and sequels to power the "Next Up" panel in the web reader.

- Reuses the dormant `recommender/sequential.rs` strategy. Wire it to
  the reader UI's "Next Up" panel instead of the recs route.
- Trains nightly on `(user_session: prev_work → next_work)` sequences.

### 16. Comment Reaction Suggestions
Analyze comment text with a lightweight NLP classifier to auto-highlight
the most appropriate emoji reaction from the custom dropdown picker
(reaction set in `REACTION-EMOJI-DESIGN.md`), reducing user friction.

- ONNX classifier (~100MB). Request-time: single forward pass per
  comment < 50ms on Orange Pi.
- Pure UI hint — never auto-posts.

---

## ⚙️ Platform Operations & Admin

### 17. Automated Translation Drafting
Use small, quantized Neural Machine Translation (NMT) ONNX model to draft
translations for metadata (titles/summaries), presented to admins in a
translation review UI for quick approve/edit.

- Pairs with `docs/plans/translation-everything.md`.
- Nightly batch: untranslated metadata → NMT draft → `translation_drafts`
  table. Admin sees side-by-side diff and approves.
- Source/target language pair selectable per-job.

### 18. Update Cadence Prediction
Use simple time-series forecasting (ARIMA or lightweight Prophet
equivalent) on an author's past update history to predict the next
chapter drop, fueling a "Chapter-release countdown" widget.

- Pure Python (statsmodels) or Rust (`statrs`) — both run on Pi.
- Background cron per active author (or hourly incremental update).
- Stored in `author_update_forecast(author_id, expected_at, ci_low,
  ci_high)`.

### 19. Spam & Bot Shadowbanning
Train an **Isolation Forest** or small XGBoost model (exported to ONNX)
on `usage_events` and `bot_scores` to silently shadowban scrapers and
forum spammers without human intervention.

- `Isolation Forest` is the right pick here — unsupervised, tiny
  footprint, retrainable in minutes.
- Nightly retrain. Request-time: score events on the fly; flag above
  threshold → invisible to other users, full content served to bot.
- Logs `shadowban_actions` table for admin audit.

### 20. QA Bug Triage & Summarization
Use a local 3B parameter LLM (via Ollama, disposed after batch) to read
nightly `qa/api-walk.js` failure logs, summarize the 5xx errors, and
auto-group them into categories for `qa/reports/ISSUES.md`.

- Extends `qa/triage.js` — which already uses ollama llama3.2:3b. The
  addition is *categorization* (group similar 5xx by endpoint +
  fingerprint) before writing ISSUES.md.
- LLM lifecycle: spawned per run, exits after group emit; never a
  daemon.

---

## Cross-Cutting Concerns

### Model lifecycle
- One shared `fastembed` singleton loaded at process start; ~200MB
  resident for the lifetime of the Rust process. Used by features #2,
  #3, #4, #8, #9, #12, #13.
- ONNX classifiers (features #7, #14, #16, #19): load-once-per-route,
  cache in-process via `OnceCell`. Per-feature footprint <50MB.
- Ollama 3B: **never resident**. Always cron + `ollama run … "prompt"`
  + dispose. Same pattern as the existing `qa/triage.js`.

### Storage budget
- `pgvector` binary quantization: 48 bytes/row × 100k works ≈ 5MB. Trivial.
- New tables for cached LLM output (ask_cache, tag_wiki_drafts,
  translation_drafts, mood, quote, etc.) all bounded by work count.

### Observability
- Each feature logs to `usage_events` with a `feature_tag` so we can
  measure adoption + degrade features that aren't pulling weight.
- Graceful-degradation events (LLM down → keyword fallback) log a
  counter `ai_fallback_total{feature=…}` — Grafana panel material.

### Privacy
- Embeddings are computed server-side from public metadata only (titles,
  summaries, tags). No user-generated content leaves the host.
- Ollama runs locally — no API keys, no external calls.

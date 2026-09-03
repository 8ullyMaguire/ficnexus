# Translate-Everything — Machine + Community + Consensus (Plan for approval)

> **Goal (user's vision):** anything a visitor sees — fic bodies, titles,
> requests, answers, forum posts, comments, reviews, UI strings, docs — is
> translated into their language. A fast local LLM translates on the fly
> when nothing better exists; the machine output is cached and shown;
> humans (any user, points if approved) submit better translations that go
> to the curator queue for consensus approval/dismissal; once an approved
> translation exists the LLM is never called again for that string+locale;
> curators (and users) can improve an approved translation, which restarts
> the consensus vote — and this improve→re-vote loop is a property of the
> WHOLE curator queue, not just translations.
>
> Plan only — no code until you approve/edit (decisions table §8).

## 1. What already exists (audited 2026-08-31)

- **Tables**: `work_translations` (title/summary per work+locale, status
  draft/approved/rejected), `chapter_translations` +
  `chapter_translation_versions` (whole-chapter JSONB, versioned, approved
  version is what readers get), `ui_translations` (namespace/key/value per
  locale — read by `GET /api/v1/translations/{locale}` but the SPA i18n
  system uses static TS dictionaries, not this table).
- **Routes** (`src/routes/locales.rs`): list locales, get UI strings,
  upsert/get work translations, get/put chapter translations (PUT →
  `chapter_translation_versions` as `draft`).
- **Reader** (`/read/[urlId]`): locale picker state, `loadTranslation()`,
  translated content switch — reads approved versions only.
- **Editor** (`/work/[workId]/translate`): chapter-by-chapter manual
  translation editor, submits draft "for curator review".
- **Approval today**: `POST /api/admin/translations/{id}/approve|reject|edit`
  and chapter-translation-versions approve/reject — **admin-only role ≥ 10,
  single actor, no consensus, no points.**
- **Consensus pattern exists**: `curator_content.rs` propose_fix/vote_fix
  (VOTE_QUORUM, no self-vote, net threshold → applied) and
  `forum_edit_proposals` (topic/post snapshots, pending/approved/rejected,
  curator review queue). These are the templates.
- **LLM**: `OllamaClient::generate(prompt, chat_model)` — same path as
  /ask and content_scan. No translation prompt exists yet.
- **Points**: `xp_source_defs` + `progression::award_xp` + `reputation_events`
  (reputation = leaderboard currency; trust 0–6 is moderation-only).
- **Body cache**: chapters as JSON blobs — source text for chapter
  translation without re-scraping.

**Gaps**: no LLM anywhere in the translation flow; approval is admin
dictatorship; no machine-translation cache; no points; nothing covers
requests/forum/comments/reviews/UI-strings/docs; no improve→re-vote;
`ui_translations` table is dead weight (nothing reads it from the SPA).

## 2. Design — one pipeline, three tiers

Fallback chain per (target, field, locale), used by every read endpoint:

```
approved  (human/curator consensus)  →  shown as "Community translation"
machine   (LLM, cached, unreviewed)  →  shown with "Machine translated" badge
original                             →  shown with a "Translate" affordance
```

### 2.1 New table: `translations` (unified store)

```sql
CREATE TABLE translations (
  id            bigserial PRIMARY KEY,
  target_type   text NOT NULL,   -- see registry below
  target_id     text NOT NULL,   -- id/url_id/slug-key, always text
  field         text NOT NULL,   -- 'title'|'body'|'content_html'|'pitch'|...
  locale        text NOT NULL,   -- 'es','de','fr','pt-BR','zh',...
  source_hash   text NOT NULL,   -- sha256 of source text; change ⇒ invalidate
  text          text NOT NULL,
  status        text NOT NULL DEFAULT 'machine'
                CHECK (status IN ('machine','pending','approved')),
  origin        text NOT NULL DEFAULT 'llm'
                CHECK (origin IN ('llm','user')),
  model         text,            -- machine rows: which model
  confidence    real,            -- optional self-assessed quality
  created_by    integer REFERENCES users(id) ON DELETE SET NULL,
  approved_proposal_id bigint,   -- link to proposal that made this approved
  created_at    timestamptz NOT NULL DEFAULT now(),
  updated_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (target_type, target_id, field, locale)
);
CREATE INDEX idx_translations_lookup ON translations (target_type, target_id, locale);
```

Target registry (validated enum in code, `TranslationType::as_str`):

| target_type | fields | source |
|---|---|---|
| work_meta | title, summary | fic_info |
| chapter | content_html | body cache / chapter_translations |
| request | title, body | fic_requests |
| request_answer | pitch | fic_request_answers |
| forum_topic | title | forum_topics |
| forum_post | body | forum_posts |
| comment | body | comments |
| review | title, body | reviews |
| ui_string | key → string | static dictionaries / ui_translations |
| doc | section | docs markdown |

`ui_translations` and `work_translations`/`chapter_translations` stay
authoritative where they already work (reader/editor shipped); the new
service wraps them with the same API surface, and ui_translations gets its
missing write path (LLM + proposals). (Alternative: migrate everything —
see D2.)

### 2.2 Machine translation, on demand + cached forever

Flow (request path stays fast; LLM never blocks a page):

1. SPA knows the reader's `readingLocale` pref (new, default =
   `users.locale` / browser language) and sends it: API endpoints accept
   `?locale=` or the client calls `POST /api/translate/batch` with
   `[{type,id,field}, ...]` for everything visible.
2. Server resolves the chain (§2): returns `{text, status}` or
   `{text: null}` for untranslated.
3. For every `null` (and every `machine` whose source_hash changed), the
   client posts `POST /api/translate/enqueue {items, locale}` — budgeted,
   deduped, per-IP/user (Write tier + `TRANSLATE_ENQUEUE_PER_HOUR`).
4. A tokio worker (pattern: bookmark_import_queue, Redis list
   `translate_queue`) pops jobs: skip if a row now exists; else chunk the
   source (`TRANSLATE_CHUNK_CHARS`, default ~3000, split on
   paragraph/`</p>`), call Ollama per chunk with the translation prompt,
   stitch, store as `status='machine'` row, bump modlog-free telemetry
   (`translate_runs`). Machine rows are **visible immediately** but always
   enqueue a curator review proposal (§3) unless `TRANSLATE_AUTO_APPROVE_MACHINE=true`.
5. Second visitor with same locale: cache hit, no LLM. Ever. The only
   re-trigger is a source edit (hash mismatch) or a human overriding it.

Prompt (same pipe format as content_scan for cheap parsing):

```
Translate the following fanfiction-community text from {lang} to {locale}.
Keep HTML tags, @mentions, #tags and URLs intact. Return ONLY the
translation, no commentary.
TEXT: ...
```

Ollama down or budget exhausted ⇒ client keeps showing original; queue
retry with backoff (existing heal/replay pattern).
### 2.3 Reader-visible signals (seamless integration)

- Every translated block renders a subtle affordance in AO3 style:
  `(Machine translated — report/improve)` or `(Community translation — by
  {user}, approved {date})`. Hover/long-press = "Show original" (toggle is
  per-block, persisted in localStorage). Works pages, request threads,
  forum threads, comments, reviews, docs — identical component
  `TranslatedText.svelte` (archive-skin + modern-skin aware).
- Locale picker in the reader already exists; promote it to a per-user
  **reading** locale pref, separate from the **interface** locale (i18n
  dictionaries). Interface follows the user's `locale`; reading follows
  `readingLocale`.
- "Translate this page/board": one button per surface that batch-enqueues
  all visible untranslated items (cheap UX, the real work is the queue).
- Language of the source is detected once per target (Ollama or the
  lightweight `lingua`-style heuristic via tag heuristics; stored on the
  source row or `translations` sibling) so the prompt always knows
  `{lang}`.

## 3. One curator queue for everything — proposals, consensus, improve→revote

This is the generalization of `curator_content::propose_fix` +
`forum_edit_proposals` + translation approval into **one** subsystem, and
it satisfies the user's "restart the consensus vote on improvement" for
everything that goes to the queue.

### 3.1 New table: `proposals`

```sql
CREATE TABLE proposals (
  id            bigserial PRIMARY KEY,
  kind          text NOT NULL,   -- 'translate'|'content_fix'|'metadata_fix'
                                 -- |'body_replace'|'work_deletion'|'collection_add'
                                 -- |'post_edit'|'request_edit'|'doc_edit'|...
  target_type   text NOT NULL,   -- 'translation','fic','forum_post',...
  target_id     text NOT NULL,
  payload       jsonb NOT NULL,  -- the change: {locale,text,field} for
                                 -- translate; {before,after} for edits
  proposer_id   integer REFERENCES users(id) ON DELETE SET NULL,
  source        text NOT NULL DEFAULT 'user'
                CHECK (source IN ('user','llm','moderator','system')),
  status        text NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','approved','dismissed','superseded')),
  supersedes    bigint REFERENCES proposals(id),  -- improve→revote chain
  decided_by    integer REFERENCES users(id),     -- last decision actor
  decided_at    timestamptz,
  note          text,
  created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_proposals_queue ON proposals (status, created_at)
  WHERE status = 'pending';

CREATE TABLE proposal_votes (
  proposal_id  bigint REFERENCES proposals(id) ON DELETE CASCADE,
  curator_id   integer NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  decision     text NOT NULL CHECK (decision IN ('approve','dismiss')),
  created_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (proposal_id, curator_id)
);
```

Machine-translation rows are proposals too: `source='llm'`, `proposer_id
NULL` — the queue is literally the human review of LLM output (which is
what "LLM translations would probably be bad so curators can improve"
means). Dismissing an LLM row hides it (back to original, re-queueable at
lower priority or marked `rejected` so the badge never re-appears).

### 3.2 Consensus rules (configurable per kind)

- Curator = role ≥ 5 or trust ≥ `MOD_TRUST_CURATOR` (matches existing
  gates). No self-vote on your own proposal (curator_content rule).
- Quorum by kind — reuse `VOTE_QUORUM`/`VOTE_NET_MIN` math from
  curator_content.rs:
  `PROPOSAL_QUORUM_translate=2`, `PROPOSAL_QUORUM_content_fix=2`,
  `PROPOSAL_QUORUM_work_deletion=2`, `PROPOSAL_QUORUM_post_edit=1`
  (single curator matches forum behavior — authors edit their own posts
  directly, non-author edits need one curator: the product rule already
  in MEMORY), `PROPOSAL_QUORUM_doc_edit=1`.
- Net score (approve − dismiss) ≥ 0 tie-break approve after quorum, else
  stays open; any curator can `decide=true` fast-path (existing forum
  fast-hide behavior) — recorded as single-curator decision in the audit.
- On approve: apply callback per kind (dispatch table in
  `src/services/proposals.rs`, like the moderation plan M4 §4.2):
  translate → upsert `translations` row status='approved'; content_fix →
  body replace (existing path); metadata → apply work fields;
  post_edit/request_edit → write new revision, **keep permanent history**
  (product rule: authors always, others via proposals); doc_edit → swap
  docs markdown; work_deletion → soft delete.
- Every decision modlogged (`crate::modlog::record`) — one filterable
  modlog page (product rule) covers all kinds via
  `?action=proposal_approve|proposal_dismiss`.

### 3.3 Improve → restart the vote (all kinds)

Any curator/user (subject to the same write rules as the original
proposal) submits a **competing proposal**: same
`(kind, target_type, target_id)`, status pending. Rules:

- While competing proposals exist, the old approved value keeps serving
  (`fallback to best current`), plus the queue shows them grouped
  ("3 competing translations for es — review").
- Voting: first proposal to reach its quorum with net ≥ 0 wins → old
  approved value is marked `superseded` (kept for history, linked via
  `supersedes` chain — permanent history preserved, product rule), new
  value applied, points paid to the winning proposer.
- All losers get `status='superseded'` — no points, no penalty (encourage
  effort, don't weaponize it).
- The apply callback re-points `translations.approved_proposal_id` and
  emits an update event so open pages re-fetch (WS/Redis pubsub already
  exists for the forum; reuse `notifications` broadcast pattern).

### 3.4 Points (reputation = leaderboard currency)

`xp_source_defs` entries (names final; amounts via
`TRANSLATE_POINTS_*` env, defaults proposed — user approves):

| event_type | base | paid when |
|---|---|---|
| translation_approved | 15 | a user proposal reaches approved |
| translation_improved | 25 | your proposal supersedes an existing approved one |
| translation_reviewed | 2 | curator cast a decision on a pending proposal (anti-idle mod incentive, daily cap) |
| machine_flag | 1 | first user to report a bad machine translation (dedupe per target) |

Award via `services::progression::award_xp` + the v3 reputation_events
write (the pattern used everywhere else), with per-user daily caps
(`TRANSLATE_POINTS_DAILY_CAP=100`) like existing sources. No XP/levels —
points only (progression scrapped 2026-08-31; MEMORY).
## 4. API surface

Read (all accept `?locale=`; response shape unchanged + `translation`
sidecar so existing clients don't break):

```
GET  /api/translate/batch?locale=es        body {items:[{type,id,field}]}
                                           → {translations:{key:{text,status,origin}}}
POST /api/translate/enqueue                body {items, locale}  (budgeted)
GET  /api/translate/status/{type}/{id}     per-locale coverage matrix (progress UI)
```

Existing endpoints that gain translation sidecars (field-by-field, not a
new proxy system — e.g. `list_requests` response items get
`title_t/…_t` only when a non-default locale is requested):

- `/api/requests` + `/api/requests/{id}` (title, body, answer pitches)
- `/api/forum/*` topic lists + thread (title, post bodies)
- `/api/comments`, `/api/reviews` bodies
- `/api/v1/works/{id}` meta + reader chapter content (already half-wired)
- `/api/v1/translations/{locale}` UI strings (now actually populated)

Proposals + queue:

```
POST /api/proposals                        submit (kind,payload) — Write tier,
                                           PROPOSALS_PER_USER_PER_DAY
GET  /api/proposals/{id}                   detail + votes + competing set
POST /api/proposals/{id}/vote              approve|dismiss (curator, no self-vote)
POST /api/proposals/{id}/decide            fast-path (curator, forum-mod-points
                                           rules apply where relevant)
GET  /api/curator/proposals?kind=&status=&locale=&grouped=   the queue
```

Translation improve/submit is `POST /api/proposals {kind:'translate',
target_type:'translation', target_id:'work:123:es:title',
payload:{locale,text,field,source_hash}}` — one entry point for the whole
queue (this is the product rule: requests+ask unified, curator consensus
on one filterable page).

## 5. Frontend

- **i18n loader**: `initI18n()` already merges dictionaries; extend to
  fetch `GET /api/v1/translations/{locale}` and override static keys
  (approved rows win; machine rows win when no approved; static TS dict is
  the floor). Keys become translatable strings like everything else.
- `TranslatedText.svelte` component (see §2.3 badges + show-original
  toggle) — used by WorkBlurb titles? **no** — blurbs stay original-size
  cheap: only list *titles* get batch-translated when
  `readingLocale != source lang`, deferred per-viewport via
  IntersectionObserver → `translate/batch` (don't LLM-scan whole boards
  eagerly).
- Reader: existing locale picker + translate page — add "Machine
  translate draft" button that enqueues + polls status, prefills the
  editor with the machine text so improving it is one pass of editing,
  not from scratch (this is the key friction-killer: the LLM output is
  the first draft for humans).
- Curator queue page `/curator/proposals` (or fold into
  `/curator/consensus` per product rule "one filterable modlog page"):
  grouped competing proposals, side-by-side diff view (before / approved /
  this proposal), vote buttons, "show original", points preview.
- Everything in all 6 dictionaries (`t('translate.…')` keys lockstep —
  skill rule).
- "Translate this page" button on requests list/detail, forum threads,
  work pages, docs, comments/reviews section headers.

## 6. Config (self-hosted per instance — every knob)

```
TRANSLATION_ENABLED=true               master switch (all tiers)
TRANSLATE_LLM_ENABLED=true             machine tier on/off (human tiers still work)
TRANSLATE_MODEL=<ollama_chat_model>    fast model for translation
TRANSLATE_CHUNK_CHARS=3000
TRANSLATE_GLOBAL_BUDGET_PER_HOUR=100   circuit breaker (shared with ask? separate key)
TRANSLATE_USER_BUDGET_PER_HOUR=10      enqueue spam guard (see guardrails plan)
TRANSLATE_AUTO_APPROVE_MACHINE=false   true = small-instance mode: LLM is trusted,
                                       queue skipped entirely (curators can still
                                       improve later)
TRANSLATE_BADGE_MACHINE=true           show "Machine translated" label
TRANSLATE_READING_LOCALE_ASK=true      prompt once to set reading locale
TRANSLATE_UI_STRINGS=true              SPA dictionary override from DB
PROPOSAL_QUORUM_TRANSLATE=2
PROPOSAL_QUORUM_CONTENT_FIX=2
PROPOSAL_QUORUM_POST_EDIT=1
PROPOSAL_QUORUM_WORK_DELETION=2
PROPOSAL_QUORUM_DOC_EDIT=1
PROPOSALS_PER_USER_PER_DAY=20
TRANSLATE_POINTS_APPROVED=15
TRANSLATE_POINTS_IMPROVED=25
TRANSLATE_POINTS_REVIEWED=2
TRANSLATE_POINTS_FLAG=1
TRANSLATE_POINTS_DAILY_CAP=100
TRANSLATE_LOCALES=en,de,es,fr,pt-BR,zh   supported target locales
TRANSLATE_KEEP_MACHINE_ON_DISMISS=false  dismissed LLM row → original, no badge
```

Single-admin private instances get: `TRANSLATE_AUTO_APPROVE_MACHINE=true`
+ `PROPOSAL_QUORUM_*=1` = "LLM translates everything, nobody votes" mode
without deleting code paths.

## 7. Milestones (each independently shippable)

1. **M1 — unified store + read sidecar**: `translations` table,
   `TranslationType` registry, `resolve_chain()`, batch + enqueue
   endpoints, queue worker + LLM prompt, `TranslatedText` component,
   reader + requests + forum threads show machine text. No proposals yet —
   machine rows are enough to prove the flywheel.
2. **M2 — proposals subsystem**: `proposals`/`proposal_votes` tables,
   service + apply dispatch, queue API, `/curator/proposals` page,
   improve→supersede→revote, modlog wiring, migrate the existing
   `curator_content` fix-proposals + `forum_edit_proposals` review flows
   onto it (or keep them as kinds — recommended: fold, one queue = product
   rule).
3. **M3 — machine rows enter the queue**: LLM rows auto-create
   `source='llm'` proposals; curators approve/dismiss/replace; points
   (xp_source_defs rows + awards + daily caps); report-a-bad-translation.
4. **M4 — UI strings + docs + everything else**: ui_translations
   populated via the same pipeline; i18n loader override; docs sections;
   comments/reviews/answer pitches; coverage-matrix progress UI
   ("requests board: 62% translated to es").
5. **M5 — polish**: "Translate this page" everywhere, side-by-side diff
   view, show-original default per pref, search hits the translated text
   too? (**no** — search stays on source text; translation is display-only;
   flagged here so the decision is explicit).

## 8. Decisions — pick before build (defaults in **bold**)

| # | Question | Options | Pick |
|---|---|---|---|
| D1 | Machine rows visible before any human review? | **A) yes, badged, plus queue entry** (instant value) · B) no, queue-only (AO3-purity) | |
| D2 | Existing `work_translations`/`chapter_translations` | **A) wrap them, new `translations` for everything else, unify in M5** · B) migrate all into `translations` now (reader churns, but one table) | |
| D3 | Who may submit translations | **A) any logged-in user** (points on approval, Write-tier limits) · B) trust ≥1 | |
| D4 | Curators editing/approving their own proposal | **A) no self-vote** (curator_content rule) · B) solo-admin instances override | |
| D5 | Translate on request (lazy, viewer-triggered) vs eager background crawl of hot content | **A) lazy + per-page button** (budget) · B) also nightly crawl of front-page items · C | |
| D6 | Machine translation of * fic bodies * (whole stories) | **A) only via existing reader button (per chapter, user-triggered)** · B) auto-crawl popular fics (cost) | |
| D7 | Competing proposals while none approved yet (race of 3 users) | **A) first-to-quorum wins, losers 'superseded'** · B) merge window (hold 24h, show all, then vote) | |
| D8 | Language detection | **A) Ollama one-liner, cached** · B) reqwest/`lingua` crate offline | |
| D9 | Points for machine-review dismissals (curator labor) | **A) reviewed=2 for both approve and dismiss** · B) approve-only | |

## 9. Risks

- **LLM cost/latency** — budgets (M1) + separate Redis circuit key;
  `TRANSLATE_GLOBAL_BUDGET_PER_HOUR` fail-closed to original.
- **Bad LLM text shown** — badge + report + queue; D1=B hides until
  reviewed if you prefer.
- **Vote collusion / points farming** — daily caps, no self-vote,
  `translation_reviewed` capped hard, modlog + metamod (forum pattern)
  can audit proposal decisions later.
- **Migration churn on `chapter_translation_versions`** — M1 leaves it
  untouched; D2=A.
- **Search/SEO serving machine text** — translation is display-layer only;
  `?locale=` never changes the indexed/canonical payload (robots see
  original).
- **Source edits invalidating translations** — `source_hash` mismatch ⇒
  approved row falls back to original + auto-re-enqueue +
  "translation is outdated" badge; history kept.


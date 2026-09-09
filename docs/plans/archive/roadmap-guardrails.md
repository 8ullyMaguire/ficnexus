# Roadmap Guardrails — Anti-Spam, Trust Gates & Curator Batch Moderation

> **Status:** plan 2026-08-31 (rev 2026-08-31c). Trigger: bogus suggestion spam on `/roadmap` (`ficnexus.polarisocial.xyz/roadmap`). No prod DB writes in this plan — only design + migration path.
> **Owner ask (base):** rate limits, trust-gated submit + vote, quarantine new ideas in the curator consensus queue before they enter the Arena, bulk for `trust_level > 3` (Elder+).
> **Rev b:** suggestions are `title + body`; cards/consensus show only `title` (open to see `title+body`); clustering is LLM-*suggested* only — curator approval required. Brainstorm related ideas (§13).
> **Rev c:** curators may polish `title` and `body` at any time, but every edit is kept as an immutable patch history visible to anyone at any point. Brainstorm ideas triaged — essentials promoted into the core plan, rest stay in §13.

## 1. Objective

Make the Roadmap consensus spam-resistant without hurting legitimate signal:

- **Spam can't reach voters.** New suggestions are invisible to the Arena / board / public consensus until a curator approves them.
- **Submit costs trust.** Low-trust and anonymous users are throttled or blocked.
- **Vote stays accessible but not zero-cost.** Lower gate than submit, still trust-gated.
- **Curators can clean bursts in seconds.** Batch review, merge, and rejection for `trust_level > 3` with audit trail.
- **Ideas are readable at a glance but rich on demand.** Every suggestion has a short `title` (what voters compare) and an optional `body` (why/how — only on detail open). Keeps Arena cards scannable while letting authors make a real pitch.
- **Clustering is advisory, not automatic.** The LLM may *suggest* "this looks like #123" but never auto-merges — only a curator merges.
- **Polishing never rewrites history.** Curators (and authors while `pending`) can edit `title`/`body`, but every version is retained and anyone can browse the patch history / diff at any point.

Preserve what works today: MaxDiff/Elo ranking (`fichub-consensus` crate), `idea → up_next → in_progress → finished → shipped` lifecycle, semantic clustering (`nomic-embed-text 768d`, threshold `0.22` as *suggestion* threshold), category + status filters.

## 2. Current state & threat model

**Tables** (`migrations/001_initial.sql:1107,1142`, `064`, `065`): `feature_clusters(id, representative_text text, embedding vector(768), elo_rating, matches_played, times_picked_best, times_picked_worst, status, category)`, `feature_suggestions(id, user_id, client_id, raw_text text, cluster_id, created_at)`, `arena_votes(id, user_id, client_id, cluster_ids int[], best, worst)`. Status check: `idea | long_term | medium_term | up_next | in_progress | finished | shipped | rejected`. Single free-text field (`representative_text` / `raw_text`, 1000-char limit) — no title/body split, no revision history.

**Handlers** (`src/routes/roadmap.rs`, `src/routes/consensus.rs`, `src/services/trust.rs`, `src/routes/auth.rs`):

- `POST /api/roadmap/suggest` — requires `AuthUser` but allows `user_id = None` (anon via `x-client-id`). Guards: `trim != ""`, `len <= 1000`, `MAX_SUGGESTIONS_PER_DAY = 3` via `COUNT(*) WHERE (user_id = $1 OR client_id = $2) AND created_at >= now()-1d`. On success: nearest `status='idea'` cluster `embedding <=> $1 < 0.22` → auto-join, else `INSERT ... status='idea'` directly. Auto-merge is the problem rev b fixes.
- `POST /api/roadmap/vote` — `user.level < 2` rejects (F7 `level 0-100`, not `trust_level 0-6`). Needs 4 `cluster_ids`, `best != worst`, uniqueness on `(user_id, cluster_ids)` or `(client_id, cluster_ids)` for anon.
- `GET /api/roadmap/arena` — `WHERE status IN (<fichub_config eligible>) ORDER BY matches_played ASC, random() LIMIT 4`. Today eligible = all except `rejected`. Returns `representative_text` as `text`.
- `PATCH /api/roadmap/features/:id` — `auth.level < 50` rejects (curator). Single-id move `{status, category}`. No content polish, no history.
- `GET /api/curator/consensus` — logged-in view over 6 kinds; `roadmap` kind lists `feature_clusters` ranked by `elo DESC` (no moderation filter). `GET /api/admin/roadmap-consensus` is read-only leaderboard.
- **No status `pending`**, no bulk endpoint, no soft-delete, no `title`/`body` split, no `suggested_cluster` column, no revision table.

**Threat observed:** burst of bogus suggestions creating many new `feature_clusters` (one per distinct embedding when `dist >= 0.22`) that immediately enter the Arena and pollute Elo + public consensus. Bypass paths: rotate `client_id` UUID (the `OR` anti-spam is bypassable), stay anonymous, submit 3/day per identity → unbounded with new identities. `POST /suggest` is cheaper than `POST /vote`.

**Trust system today** (`src/services/trust.rs`): 7 levels `0 New … 6 Near-admin`, `PUBLISH_MIN_TRUST=2`, `RESOLVE_MIN_TRUST=5`, `fetch_trust_level(db, Option<uid>) -> 0 for anon`, monotonic promotion to TL4 (TL3 revocable on spam). Roadmap currently gates on `level` (0-100), not `trust_level` — this plan unifies roadmap on `trust_level`.

**Forum precedent for history:** `forum_edit_proposals` keeps every snapshot as an immutable row (`target_type, target_id, author_id, snapshot jsonb, status, reviewed_by, created_at`) and `GET /api/forum/posts/{id}/moderations` + `GET .../history` expose it publicly. Roadmap will mirror that append-only pattern (§6–7) rather than in-place `UPDATE` without audit.

## 3. Design decisions

1. **Trust, not level, for roadmap.** Roadmap is a participation surface → gate on `trust_level` (0-6) via `trust::assert_min_trust`. Keeps `level` for progression/rank display. Map legacy `level < 2` (vote) → `trust_level >= 1` and `level < 50` (curator) → `trust_level >= 4` for bulk, `>= 3` for single moves.

2. **Kill anonymous submit.** Require authenticated user for `POST /suggest`. Anonymous votes can stay (keyed by `client_id`) but anonymous suggests go away — the cheapest spam vector. Return `401` with a login CTA if `user_id is None`.

3. **Quarantine new clusters.** New clusters spawn as `status='pending'` (new enum value), excluded from Arena, board default view, and public consensus until approved. Approval (`pending → idea`) is the curator action that makes Elo real. Rejection (`pending → rejected`) hides without hard delete (preserves forensics; hard delete is a separate bulk action).

4. **Tiered daily limits by trust.** `3/day` for everyone is both too generous for TL0 and too stingy for proven members. Scale: TL1 1/day, TL2 3/day, TL3 5/day, TL4+ 10/day. Counts are per `user_id` (no `client_id` fallback after auth requirement). Window is rolling 24h on `feature_suggestions.created_at`. A title+body pair counts as one submission (title is required, body optional).

5. **Title + body split.** Free-text → `title` (5–100 chars, the vote surface) + `body` (0–2000 chars, markdown, the detail). Vote/consensus/board cards show only `title` (+ Elo/votes/category); detail view shows `title + body` + suggestion history. Keeps cards scannable and forces authors to name the idea crisply. Embedding for clustering is `title + "\n\n" + body` (title weighted by duplication: `title + "\n" + title + "\n\n" + body` if you want title to dominate — default to plain concat and tune if needed).

6. **Clustering is suggestion-only.** Never auto-join. On submit we compute the nearest `status='idea'` cluster and store `(suggested_cluster_id, suggested_distance, suggested_at)` on the new `pending` cluster, but we still create a distinct `pending` cluster. The curator queue surfaces "Suggested duplicate of #123 (distance 0.18) [Merge] [Keep separate]". Bulk action `merge` is the only path that collapses duplicates. This preserves signal (curator sees LLM hint) without letting spam auto-attach to legit ideas.

7. **Batch is an admin primitive, not a frontend hack.** One transactional `POST /api/admin/roadmap/bulk` handling approve/reject/delete/merge + audit (merge now meaningful as "merge pending into suggested target"). Frontend reuses the bulk pattern from `src/routes/bulk.rs` but gated on `trust_level`.

8. **Board and consensus filter `pending` by default.** Curators see a dedicated "Pending review" column/filter; regular users never see pending unless they authored it (optional "My pending" hint via `GET /api/roadmap/my-pending` — promoted from brainstorm, see §7.2).

9. **No new infra.** Postgres + existing Redis (reused for IP sliding window if added later) only. No new crate.

10. **Polish is append-only with public patch history (rev c).** Any edit to `title`/`body` — whether curator polishing a live idea or an author fixing their pending draft — appends a new row to an immutable `feature_cluster_revisions` table and updates `feature_clusters.title/body` to the latest. The history endpoint is public (no auth) so anyone can see how the suggestion looked at any point (mirrors `forum_edit_proposals` history). No edit ever mutates a past revision. Diff is computed server-side or client-side from successive snapshots; the store keeps full snapshots (cheap: ≤2100 chars per revision).

11. **Promoted essentials (from §13 brainstorm).** The following brainstorm ideas are now core, not optional: search-before-submit (live duplicate chips while typing), confidence bands for suggested duplicates, side-by-side duplicate diff view, author's pending shelf + withdraw/edit-while-pending, structured rejection reasons, and alias preservation on merge. See §7/§8 for wiring.

## 4. Trust gates (final)

| Action | Gate | Why |
|---|---|---|
| `POST /api/roadmap/suggest` | `trust_level >= 1` (Basic). TL0 blocked. Staff `role >= 10` bypasses. | Suggest creates clusters — most abuse-prone. TL1 = entered 5 topics + 30 posts read + 10 min (cheap legitimately, hard to farm at scale if anon is closed). |
| Per-day suggestion cap | TL1 1, TL2 3, TL3 5, TL4+ 10 (rolling 24h) | Matches `PUBLISH_MIN_TRUST=2` intuition: publishing-tier members get headroom. One `title+body` = one count. |
| `POST /api/roadmap/vote` | `trust_level >= 1` (drop from current `level>=2`). | Voting is lower cost than authoring — per owner ask. Still blocks TL0/anon farms. Revisit to `>=2` if vote spam appears. |
| `PATCH /api/roadmap/features/:id` (single move) | `trust_level >= 3` (Regular) | Current `level>=50` roughly maps to Regular/Elder. Keep single moves slightly more open than bulk. |
| Polish `PATCH /api/roadmap/features/:id` title/body (curator) | `trust_level >= 3` (same as move); author polish while `pending` allowed for the author at `trust >= 1` | Polishing is a content edit — same bar as moving columns. Author self-correct before review shouldn't need Elder. |
| Bulk `POST /api/admin/roadmap/bulk` | `trust_level > 3` (i.e. `>= 4 Elder`), per explicit ask. Staff `role>=10` also passes. | Bulk can nuke/merge many clusters — restrict to proven members. |
| `GET /api/roadmap/arena` / `GET /api/roadmap/consensus` / `GET /api/roadmap/features` (lists) | No gate (public read) but server filters `status='pending'` and `status='rejected'` out. Detail `GET /api/roadmap/features/:id` for `pending` requires `trust >= 3` or author. History `GET .../:id/history` is **public** (no gate). | Readers don't need trust to see approved consensus; history is transparency. Pending body is curator-only. |
| Changelog create | stays `level >= 50` or `trust >= 4` — unchanged in this plan. |  |

Anonymous policy: `suggest` → 401. `vote`/`arena`/`consensus` → allowed (vote still requires `trust>=1`, so effective anon vote is blocked; anon can read arena/consensus/history).

Tune knobs without code changes later: move thresholds to `src/config.rs` (`roadmap_suggest_min_trust`, `roadmap_vote_min_trust`, `roadmap_bulk_min_trust`, limits map, `roadmap_title_max`, `roadmap_body_max`, `roadmap_suggest_distance_threshold`).

## 5. Intake flow after this plan

```
frontend: Title <input maxlength 100> + Body <textarea markdown maxlength 2000>
  + live "Similar existing ideas" chips (search-before-submit, promoted)
  + honeypot website + form_opened_at (mount timestamp)
  → POST /api/roadmap/suggest {title, body?, website?, form_opened_at?}

  → auth? no → 401 {msg: "Log in to suggest"}
  → trust fetch → < 1 → 403 "trust level 1 (Basic) required; keep reading…"
  → honeypot / dwell check → silent 200 {err:0, quarantined:true} if bot-like
  → daily cap by trust tier (per user_id, rolling 24h) → 429 if over
  → validate: title 5..100 chars (trim), body 0..2000, at least title non-empty
  → embed: text_for_embedding = title + "\n\n" + body
     call Ollama nomic-embed-text → vector
     search: SELECT id, embedding <=> $1 AS dist FROM feature_clusters
             WHERE status='idea' ORDER BY dist LIMIT 1
     if found and dist < 0.22 → suggested_cluster_id = id, suggested_distance = dist
     else → suggested = null
     (never auto-join — always create a new pending cluster)
  → INSERT feature_clusters (title, body, representative_text=title — compat,
        embedding, status='pending', category='general',
        suggested_cluster_id, suggested_distance) RETURNING id
  → INSERT feature_suggestions (user_id, title, body, raw_text=title — compat, cluster_id=id)
  → INSERT feature_cluster_revisions (cluster_id, rev_no=1, title, body, edited_by=user_id, reason='initial submission')  -- first patch history entry
  → response {cluster_id, quarantined:true, status:"pending",
              suggested_cluster_id, suggested_distance}
  → NOT visible in Arena/board/consensus until curator approves

curator (trust>3) reviews in /curator/consensus?kind=roadmap&status=pending or /roadmap/features/:id
  card shows: title, confidence band for suggested duplicate, body collapsed, revision count
  actions:
    Approve: pending → idea (now eligible for Arena, Elo 1500 seeded, keeps its embedding). No content change → no new revision unless curator also polished title/body in the same action.
    Approve & Merge: pending → merged into suggested target
        (re-point feature_suggestions.cluster_id → target, preserve source as alias (§7.2), delete pending cluster — revisions of the deleted pending are kept or moved to target as "merged from #X" note)
    Keep separate: pending → idea without merge (LLM was wrong). Optionally curator polishes title/body in the same request → creates a new revision before the status flip.
    Reject: pending → rejected with structured reason (spam/duplicate/out-of-scope/needs-clarification + free-text)
    Hard delete (spam burst): bulk delete
    Polish (any time, incl. on live idea): PATCH title/body → appends revision (§7.1) — history visible to anyone via GET .../:id/history

author (while pending): may PATCH own pending cluster's title/body (trust >=1, author check) → also appends a revision with edited_by=author and status stays pending. Can withdraw (→ rejected with reason "withdrawn by author").
```

Why search only `idea` (not `pending`) for suggestions: prevents a spam wave from forming its own gravity well — new pending spam doesn't get suggested as a target, so legit paraphrases aren't pulled toward junk.

## 6. Database migration

New migration `080_roadmap_guardrails.sql` (number = next free after 065; `066` is `plugin_marketplace_kinds` — confirm head before assigning):

```sql
-- widen status to include quarantine
ALTER TABLE feature_clusters DROP CONSTRAINT IF EXISTS feature_clusters_status_check;
ALTER TABLE feature_clusters ADD CONSTRAINT feature_clusters_status_check
  CHECK (status IN ('pending','idea','long_term','medium_term','up_next','in_progress','finished','shipped','rejected'));

-- title+body split (keep representative_text for compat; backfill from it)
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS title text;
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS body text NOT NULL DEFAULT '';
-- backfill existing rows: title = left(representative_text, 100)
UPDATE feature_clusters SET title = left(representative_text, 100) WHERE title IS NULL;
ALTER TABLE feature_clusters ALTER COLUMN title SET NOT NULL;
-- keep representative_text for 1-2 releases, then drop (or keep as alias)

ALTER TABLE feature_suggestions ADD COLUMN IF NOT EXISTS title text;
ALTER TABLE feature_suggestions ADD COLUMN IF NOT EXISTS body text NOT NULL DEFAULT '';
UPDATE feature_suggestions SET title = left(raw_text, 100) WHERE title IS NULL;
-- raw_text stays for compat; new writes fill both

-- LLM suggestion (advisory, not applied)
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS suggested_cluster_id integer REFERENCES feature_clusters(id) ON DELETE SET NULL;
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS suggested_distance real;
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS suggested_at timestamptz;

-- curator moderation audit on clusters
ALTER TABLE feature_clusters
  ADD COLUMN IF NOT EXISTS moderated_by integer REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN IF NOT EXISTS moderated_at timestamptz,
  ADD COLUMN IF NOT EXISTS rejection_reason text;
-- structured rejection kind (promoted from brainstorm)
ALTER TABLE feature_clusters ADD COLUMN IF NOT EXISTS rejection_kind text CHECK (rejection_kind IN ('spam','duplicate','out-of-scope','needs-clarification','withdrawn','other') OR rejection_kind IS NULL);

-- alias preservation on merge (promoted): keep source title as a searchable alias on the target
CREATE TABLE IF NOT EXISTS feature_cluster_aliases (
    id bigserial PRIMARY KEY,
    cluster_id integer NOT NULL REFERENCES feature_clusters(id) ON DELETE CASCADE,
    alias_title text NOT NULL,
    source_cluster_id integer REFERENCES feature_clusters(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_cluster_aliases_cluster ON feature_cluster_aliases(cluster_id);
CREATE INDEX IF NOT EXISTS idx_cluster_aliases_title_trgm ON feature_cluster_aliases USING gin (alias_title gin_trgm_ops);

-- make embedding nullable so Ollama-down suggests still create a reviewable pending (no vector)
ALTER TABLE feature_clusters ALTER COLUMN embedding DROP NOT NULL;

-- ── patch history (rev c, public) — append-only, mirrors forum_edit_proposals ──
CREATE TABLE IF NOT EXISTS feature_cluster_revisions (
    id bigserial PRIMARY KEY,
    cluster_id integer NOT NULL REFERENCES feature_clusters(id) ON DELETE CASCADE,
    rev_no integer NOT NULL,                                   -- 1..N per cluster, monotonic
    title text NOT NULL,
    body text NOT NULL,
    edited_by integer REFERENCES users(id) ON DELETE SET NULL, -- author or curator
    reason text,                                               -- optional: "polish title", "fix typo", null for initial
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (cluster_id, rev_no)
);
CREATE INDEX IF NOT EXISTS idx_cluster_revisions_cluster ON feature_cluster_revisions(cluster_id, rev_no);
CREATE INDEX IF NOT EXISTS idx_cluster_revisions_editor ON feature_cluster_revisions(edited_by);
-- backfill one rev per existing cluster so history is complete from day one
INSERT INTO feature_cluster_revisions (cluster_id, rev_no, title, body, edited_by, reason)
SELECT id, 1, title, body, NULL, 'backfill' FROM feature_clusters
ON CONFLICT DO NOTHING;

-- helpful indexes for the new read patterns
CREATE INDEX IF NOT EXISTS idx_feature_clusters_status_pending ON feature_clusters(status) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_feature_clusters_status_idea ON feature_clusters(status) WHERE status = 'idea';
CREATE INDEX IF NOT EXISTS idx_feature_clusters_suggested ON feature_clusters(suggested_cluster_id) WHERE suggested_cluster_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_feature_suggestions_user_created ON feature_suggestions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_feature_suggestions_cluster ON feature_suggestions(cluster_id);
CREATE INDEX IF NOT EXISTS idx_feature_clusters_title_trgm ON feature_clusters USING gin (title gin_trgm_ops);

-- optional tight FK (uncomment after verifying no orphans):
-- ALTER TABLE feature_suggestions DROP CONSTRAINT IF EXISTS feature_suggestions_cluster_id_fkey;
-- ALTER TABLE feature_suggestions ADD CONSTRAINT feature_suggestions_cluster_id_fkey
--   FOREIGN KEY (cluster_id) REFERENCES feature_clusters(id) ON DELETE CASCADE;
```

No `deleted_at` column — `rejected` is the soft-delete; hard delete is `DELETE`. For forensics after hard delete rely on `modlog` + `pg_dump` + `feature_cluster_revisions` (revisions of a deleted pending are retained until the cluster row is deleted; for hard-deleted spam we still keep the modlog + optionally keep the last revision as a tombstone — see §7.2).

API compat: keep `representative_text`/`raw_text` read paths for 1 release (return `title` as `representative_text` alias) so old clients don't break; write both columns on insert. Similarly, `feature_cluster_revisions` backfill ensures `GET .../history` never returns empty.

## 7. Backend changes

### 7.1 `src/routes/roadmap.rs` — suggest / vote / arena / detail / polish

- Imports: `use crate::services::trust;`
- Constants (or `config.rs`):
  ```rust
  pub const ROADMAP_SUGGEST_MIN_TRUST: i16 = 1;
  pub const ROADMAP_VOTE_MIN_TRUST: i16 = 1;
  pub const ROADMAP_MOVE_MIN_TRUST: i16 = 3;
  pub const ROADMAP_BULK_MIN_TRUST: i16 = 4; // >3 per ask
  pub const ROADMAP_POLISH_MIN_TRUST: i16 = 3; // curator polish; authors bypass while pending
  pub const ROADMAP_TITLE_MIN: usize = 5;
  pub const ROADMAP_TITLE_MAX: usize = 100;
  pub const ROADMAP_BODY_MAX: usize = 2000;
  pub const ROADMAP_SUGGEST_DISTANCE_THRESHOLD: f64 = 0.22;
  // confidence bands (promoted)
  // distance < 0.15 => Very likely duplicate, < 0.22 => Possible related
  fn confidence_band(dist: f64) -> &'static str {
      if dist < 0.15 { "very_likely" } else { "possible" }
  }
  fn daily_limit_for_trust(tl: i16) -> i64 {
      match tl { 1 => 1, 2 => 3, 3 => 5, _ => 10 }
  }
  fn text_for_embedding(title: &str, body: &str) -> String {
      if body.is_empty() { title.to_string() } else { format!("{title}\n\n{body}") }
  }
  ```
- `SuggestRequest`: replace `text: String` with `{ title: String, body: Option<String>, website: Option<String>, form_opened_at: Option<String> }`.
- `suggest_handler`:
  - `let uid = user.user_id.ok_or(Forbidden("login required"))?;`
  - `let tl = trust::fetch_trust_level(&state.db, Some(uid)).await;` + `assert_staff_or_min_trust(..., ROADMAP_SUGGEST_MIN_TRUST)`.
  - Honeypot: if `website.is_some_and(|s| !s.is_empty())` or `form_opened_at` missing / dwell < 1500ms → `Ok(Json(json!({"err":0,"quarantined":true})))` without insert.
  - Daily cap: `SELECT COUNT(*) FROM feature_suggestions WHERE user_id=$1 AND created_at >= $2` vs `daily_limit_for_trust(tl)`.
  - Validate title/body lengths (trim, 5..100, 0..2000).
  - Embed `text_for_embedding(&title, &body)`. On Ollama failure: `embedding = None`, `suggested = None` — still insert pending with `embedding=NULL`.
  - On embedding success: search `WHERE status='idea'` for nearest, store `suggested_cluster_id/distance` if `< threshold`; never join. Confidence band is derived at read time.
  - Insert cluster: `INSERT INTO feature_clusters (title, body, representative_text, embedding, status, category, suggested_cluster_id, suggested_distance, suggested_at) VALUES ($1,$2,$1,$3::vector,'pending','general',$4,$5,now()) RETURNING id` (compat: `representative_text = title`). If `embedding is None`, insert `NULL::vector`.
  - Insert suggestion: `INSERT INTO feature_suggestions (user_id, title, body, raw_text, cluster_id) VALUES ($1,$2,$3,$2,$4)` (`raw_text = title` compat).
  - Insert first revision: `INSERT INTO feature_cluster_revisions (cluster_id, rev_no, title, body, edited_by, reason) VALUES ($id, 1, $title, $body, $uid, 'initial submission')`.
  - Response: `{cluster_id, quarantined:true, status:"pending", suggested_cluster_id, title, body}`.
- `vote_handler`: replace `if user.level < 2` with `trust::assert_min_trust(..., ROADMAP_VOTE_MIN_TRUST, "voting")`.
- `arena_handler`: select `id, title, body, matches_played, elo` but return only `title` in the array (`text: title` compat). Include `has_body: body != ""` so UI can show an expand affordance without leaking body to voters pre-click.
- New `GET /api/roadmap/features/:id` (or extend existing `features_list` with `?id=`): returns `{id, title, body, category, status, elo, matches_played, times_picked_best/worst, suggestions: count, suggested_cluster_id, suggested_distance, confidence_band, created_at, moderated_by/at, revision_count}`. Gate: if `status='pending'` and requester is not author and `trust < 3`, return 404 (hide pending detail from low-trust users). Body is returned here (detail), not in lists.
- New `GET /api/roadmap/features/:id/history` — **public** (no auth), returns `{err:0, cluster_id, items:[{rev_no, title, body, edited_by, editor_username, reason, created_at}]}` ordered `rev_no ASC`. Paginate `?limit=&offset=` if needed, but default is all (revisions are small). Used by detail page "History" tab so anyone can see how the suggestion looked at any point. This is the rev c transparency guarantee.
- `PATCH /api/roadmap/features/:id` — polish + move (two concerns, one handler or split into `PATCH /:id` and `PATCH /:id/polish`):
  - Auth: if `title` or `body` is being changed → polish path. Check: `is_author && status=='pending'` → allow at `trust >= 1`; else require `trust >= ROADMAP_POLISH_MIN_TRUST` (or `role>=10`). Status/category changes still require `trust >= ROADMAP_MOVE_MIN_TRUST`.
  - Validate edited title/body (same 5..100 / 0..2000). If title/body unchanged, it's a pure status move (no revision).
  - Tx: `SELECT ... FOR UPDATE` the cluster, compute `next_rev = max(rev_no)+1`, `INSERT INTO feature_cluster_revisions (cluster_id, rev_no, title, body, edited_by, reason) VALUES ($id,$next,$new_title,$new_body,$uid,$reason)`, then `UPDATE feature_clusters SET title=$t, body=$b, status=COALESCE($s,status), category=COALESCE($c,category), moderated_by=$uid, moderated_at=now() WHERE id=$id`. If the edit also changes `status`, handle the status transition in the same tx (e.g. approve+polish atomically).
  - On success also `record` to modlog (`roadmap_polish` with `target_type='roadmap_cluster'`). Response includes `{id, rev_no, title, body}`.
  - Rate-limit polish: reuse daily cap? No — polish is curator-only, so a simple per-cluster cooldown (e.g. 1 polish per 30s via Redis or `created_at` check) to avoid accidental double-submit is enough.
- `consensus_handler` / `features_list_handler`:
  - Public default excludes `pending`/`rejected` (`WHERE status NOT IN ('pending','rejected')`). Keep admin/curator feed as-is or add `?include_pending=true` for curators.
  - Select `title, body` but public list serializes only `title` as `text` (compat alias `title` + `text`). Detail endpoint is the only place body is returned for pending/idea. Include `revision_count` and `has_body` in list rows so cards can show "edited" affordance without fetching history.
  - When `?status=pending` explicitly requested, require `trust >= 3` else 403.
- Search-before-submit (promoted): new `GET /api/roadmap/search?q=&limit=5` (public) — `SELECT id, title FROM feature_clusters WHERE status='idea' AND title ILIKE $1 ORDER BY elo DESC LIMIT $limit` plus an optional embedding search if `q` is long enough (reuse trigram + vector). Used by the submit box for live chips. Cheap, no auth.

### 7.2 New bulk handler + author's pending shelf + merge aliasing

```
POST /api/admin/roadmap/bulk
  Body: {
    ids: number[],                         // 1..100, deduped
    action: "approve" | "reject" | "delete" | "merge",
    reason?: string,                       // free-text
    rejection_kind?: "spam"|"duplicate"|"out-of-scope"|"needs-clarification"|"withdrawn"|"other", // structured, promoted
    target_id?: number,                    // required for merge (idea cluster to merge into)
    category?: string,                     // optional for approve (sets category at approval time)
    title?: string, body?: string,         // optional polish on approve (creates a revision — see §7.1)
    polish_reason?: string
  }
  Gate: trust_level > 3 (Elder+, i16 >=4) OR role >=10
  Tx per action:
    approve: for each id where status='pending': optionally polish first (insert revision), then UPDATE status='idea', moderated_by/at, rejection_kind=NULL, suggested_cluster_id=NULL (consumed), increment revision if polished
    reject:  UPDATE status='rejected', rejection_reason=$reason, rejection_kind=$kind, moderated_by/at
    delete:  DELETE FROM arena_votes WHERE cluster_ids && $ids then DELETE FROM feature_suggestions WHERE cluster_id=ANY($ids) then DELETE FROM feature_clusters WHERE id=ANY($ids) — revisions cascade via FK; for spam burst, keep a modlog entry with the last title/body snapshot so forensics survive hard delete
    merge:   for each source id (pending): INSERT INTO feature_cluster_aliases (cluster_id, alias_title, source_cluster_id) VALUES ($target, $source.title, $source.id) — preserves searchable alias (promoted); UPDATE feature_suggestions SET cluster_id=$target WHERE cluster_id=ANY($ids); INSERT INTO feature_cluster_revisions (cluster_id, rev_no, title, body, edited_by, reason) VALUES ($target, next, $target.title, $target.body || '\n\nAlso suggested as: ' || $source.title, $actor, 'merged from #'||$source.id) — optional body append for audit; DELETE FROM feature_clusters WHERE id=ANY($ids) (source pendings) — their revisions are deleted via CASCADE but the alias + target revision + modlog keep the trace
  INSERT INTO modlog (actor_id, action, target_type='roadmap_cluster', target_ids, reason, rejection_kind, created_at)
  Response: { err:0, updated: number, action, ids }
```

Add `GET /api/admin/roadmap/pending?limit=&q=` (curator list) if `/curator/consensus?kind=roadmap&status=pending` isn't enough — same query as `consensus_feed` but filtered `status='pending'` with title + body + author + suggested target join + confidence band + revision count.

Add `GET /api/roadmap/my-pending` (auth required, promoted): `SELECT id, title, body, status, created_at, suggested_cluster_id FROM feature_clusters WHERE id IN (SELECT cluster_id FROM feature_suggestions WHERE user_id=$uid) AND status='pending' ORDER BY created_at DESC`. Powers "Your pending ideas" shelf plus withdraw.

Add `POST /api/roadmap/features/:id/withdraw` (author only, `pending` only): `UPDATE feature_clusters SET status='rejected', rejection_kind='withdrawn', rejection_reason='withdrawn by author', moderated_by=$uid WHERE id=$id AND status='pending'` plus modlog. Alternative is `PATCH` with `status=rejected` by author, but a dedicated withdraw endpoint is clearer for the UI.

Wire in `src/server.rs`: `.route("/api/admin/roadmap/bulk", post(bulk_handler))`, `.route("/api/roadmap/features/{id}", get(feature_detail_handler).patch(feature_polish_handler))`, `.route("/api/roadmap/features/{id}/history", get(feature_history_handler))`, `.route("/api/roadmap/features/{id}/withdraw", post(withdraw_handler))`, `.route("/api/roadmap/my-pending", get(my_pending_handler))`, `.route("/api/roadmap/search", get(search_handler))` and the pending list.

### 7.3 Trust promotion interaction

No change to `services/trust.rs` promotion. Optionally log roadmap spam rejections as negative signal feeding TL3 revocability (future: increment a counter that the promoter reads). Out of scope for v1. Structured `rejection_kind` makes that future aggregation trivial (`WHERE rejection_kind='spam'`).

## 8. Frontend

- `/roadmap` submit box: two fields — `Title` (single-line input, 5–100, placeholder "Add dark mode to reader") + `Details` (textarea markdown, 0–2000, placeholder "Why, how, edge cases…" + live char counts). While typing the title (debounced 250ms), call `GET /api/roadmap/search?q=<title>` and show "Similar existing ideas" chips (promoted search-before-submit) — each chip links to the existing idea's detail so the author can upvote instead of duplicating. Hidden honeypot `website` + hidden `form_opened_at` on mount. On submit, send `{title, body, website, form_opened_at}`. On 401 → "Log in to suggest" CTA. On 403 trust gate → `TRUST_NAMES[level]` message from API error. On success with `quarantined:true`, show "Thanks — \"{title}\" is pending curator review before it enters the vote. You can track it in 'Your pending ideas'." Reset fields.
- Arena / board / public consensus cards: show only `title` (truncate 100 chars), plus `Elo · votes · category` badges and `has_body` indicator (e.g. "· details") and `revision_count` dot if edited. Clicking a card opens a detail drawer/modal (or navigates to `/roadmap/features/:id`) that shows `title + body` (markdown rendered via existing sanitizer — no raw HTML), suggestion history, and for pending: nothing extra for regular users; for curators, show the suggested-duplicate banner with confidence band.
- `/roadmap/features/:id` detail page (new or reuse board card expand): renders `title` as h1, `body` as markdown, metadata badges, related ideas graph (top 3 nearest `idea` by embedding — optional follow-up), and a **History tab** (public, rev c). History tab calls `GET .../:id/history` and renders a vertical timeline: `Rev 1 — initial submission by @alice — 2026-08-31 — title/body snapshot`, `Rev 2 — polished by @curator — "fix typo" — diff (added/removed) + full snapshot toggle`. Anyone can see every version; no auth required. Include a per-revision permalink `?rev=3` that scrolls to that snapshot.
- `/curator/consensus` (kind=roadmap): add `Roadmap: Pending` tab. Each row shows checkbox, `title`, truncated `body` preview, author, date, revision count, and if `suggested_cluster_id` exists: **confidence-banded banner** (promoted) — "Very likely duplicate of #123 (distance 0.12)" vs "Possible related to #123 (0.20)" + [View target] which opens a **side-by-side diff view** (promoted): left = new submission's `title+body`, right = suggested target's `title+body`, with diff highlights and category/Elo context, so the merge decision is one glance. Bulk bar: [Approve] [Approve & Merge into suggested] [Keep separate] [Reject…] [Delete hard]. On Approve, an inline polish popover lets the curator fix title/body before confirming — that polish creates a revision atomically with the approval. Merge target picker defaults to `suggested_cluster_id` but is searchable. Reuse the bulk UX from `/admin/bulk/*` and `/curator/consensus` page patterns. Reject opens a dialog with structured `rejection_kind` chips + optional free-text (promoted).
- Author's pending shelf (promoted): new section on `/roadmap` for logged-in users — "Your pending ideas" — lists `GET /api/roadmap/my-pending` (title, status, date, suggested hint). Each row has [Edit] (inline title/body edit → creates a revision, still pending) and [Withdraw] (→ rejected/withdrawn). Edits while pending are author-allowed and also append to the same public history.
- Optional `/admin/roadmap` page: same list but with Elo + `matches_played` + all suggestion bodies expanded + history timeline, for deeper review before merge.
- After bulk action, optimistic update + toast with count; log to modlog link.
- Keep API compat: frontend reads both `text` and `title` fields (`title ?? text`) during rollout. History UI reads `title`/`body` per revision.

## 9. Cleaning the current burst (no prod writes by this agent)

Do not hard-delete from prod without a dump. Recommended manual steps (owner runs via `psql` on ThinkCentre or via the new bulk API once shipped):

1. Snapshot: `pg_dump -t feature_clusters -t feature_suggestions -t feature_cluster_revisions -t feature_cluster_aliases -t arena_votes > /tmp/roadmap_pre_cleanup.sql`
2. Identify spam (now title-aware): `SELECT id, left(title,80), left(body,80), status, suggested_cluster_id, created_at, (SELECT count(*) FROM feature_suggestions s WHERE s.cluster_id=c.id) AS sugg, (SELECT count(*) FROM feature_cluster_revisions r WHERE r.cluster_id=c.id) AS revs FROM feature_clusters c WHERE created_at > now() - interval '14 days' ORDER BY id DESC LIMIT 100;` then `SELECT id, title, body, user_id, created_at FROM feature_suggestions WHERE created_at > now()-interval '14 days' ORDER BY id DESC LIMIT 100;`
3. If a single `user_id` burst: bulk reject via the new API or `UPDATE feature_clusters SET status='rejected', rejection_reason='spam burst 2026-08-31', rejection_kind='spam', moderated_by=$you WHERE id IN (...);`
4. Hard delete only if clearly junk: bulk `delete` action (also purges suggestions/votes referencing those clusters; revisions cascade but modlog keeps the last snapshot). Prefer `rejected` (reversible, history intact).
5. Verify: `GET /api/roadmap/consensus` no longer lists them; `GET /api/curator/consensus?kind=roadmap&status=rejected` shows them for audit; `GET /api/roadmap/features/:id/history` still shows the original spam text so the moderation was auditable.
6. Backfill titles + revisions: after migration, `UPDATE feature_clusters SET title=left(representative_text,100) WHERE title IS NULL` and the `INSERT ... SELECT` backfill already create rev 1 for each existing cluster.

## 10. Verification

- Unit: `cargo test` — new trust-gate tests for `suggest_handler` (TL0 blocked, anon 401, honeypot silent, daily cap per tier, title/body validation, suggested_cluster populated but not auto-merged), `vote_handler` TL1 pass, `arena_handler` excludes `pending` and returns title-only, polish handler (curator creates revision, author can polish only while pending, anon cannot read pending history beyond title?), history handler public read.
- Integration: `sqlx prepare --check` after migration; test `GET /api/roadmap/features/:id` gate for pending detail (author + TL3 pass, anon 404) and `GET .../history` is public and returns all revs in order; test search-before-submit returns trigram matches; test bulk merge creates alias + target revision + re-points suggestions.
- Manual QA against ThinkCentre staging: TL0 submit → 403, TL1 first submit → 200 pending with suggested hint, second same-day over cap → 429, typing title shows similar chips, TL3 bulk approve with polish → 200 + new revision + public history shows both versions, Arena no longer serves pending, `/curator/consensus?kind=roadmap` hides pending by default but shows with `?status=pending` for TL3+, detail page History tab shows diff and per-rev permalink, withdraw while pending → rejected/withdrawn + history.
- Load: burst 20 pending submits from one TL3 account → only 5/day pass (cap). Burst of near-duplicates → all land as separate pending with same `suggested_cluster_id`, single merge collapses them and leaves aliases + target revision.

## 11. Rollout order

1. Migration + backend gates + title/body + suggested-only clustering + revision table + history endpoint + Arena/consensus filters. Deploy. Spam stops accumulating; new submits are well-formed; history is live.
2. Detail endpoint `GET /api/roadmap/features/:id` + title-only list serialization + search-before-submit endpoint. Deploy.
3. Bulk API + modlog + merge aliasing + structured rejection + my-pending/withdraw. Deploy.
4. Frontend: honeypot + title/body form + live similar-ideas chips + title-only cards + detail drawer/page + public History tab with diff + curator side-by-side duplicate view + confidence bands + bulk UI with polish-on-approve + author's pending shelf (edit/withdraw). Deploy.
5. Run manual cleanup via bulk API; backfill titles/revisions (already in migration).
6. Follow-up (optional): IP sliding window in Redis (`suggest:<ip>:count` 1h), tighten `ROADMAP_SUGGEST_DISTANCE_THRESHOLD`, feed rejections into TL3 demotion signal, add related-ideas graph on detail, drop `representative_text`/`raw_text` columns after one release.

## 12. Risks & non-goals

- **False positives on `pending`:** legitimate ideas delayed until curator review. Mitigated by small curator pool + optimistic "pending" toast + author-visible "my pending" list + History tab showing the original pitch was preserved even after polish. Could add auto-approve for TL3+ authors (future, not v1).
- **Trust vs level confusion:** keep `level` visible as progression/rank; roadmap copy must say "trust level" not "level" to avoid user confusion (update `docs/frontend-design.md` strings).
- **Embedding fail path:** when Ollama down, new code creates `pending` with `embedding=NULL` and `suggested=NULL` and a rev 1 — still reviewable (no silent orphan).
- **Title quality:** short titles can be vague ("Fix it"). Mitigate with placeholder examples, 5-char minimum, live similar-ideas chips, and curator title-polish on approve (§7.1 creates a revision, history keeps the original). Could add LLM title suggestion from body (future, not committed — stays in §13).
- **Body markdown safety:** render via existing sanitizer (same as comments/reviews) — no raw HTML. History snapshots are stored as raw markdown; rendered on read.
- **History growth:** revisions are small (≤2100 chars + metadata). 10k ideas × 3 revs avg = ~60 MB — negligible. Index on `(cluster_id, rev_no)` keeps reads fast. No pruning; append-only.
- **Merge consents & search:** alias table ensures merged source titles remain searchable — avoids losing discoverability after dedup.
- Non-goals: full CAPTCHA, per-IP hard block, LLM spam classifier — defer unless burst recurs after these guardrails.

## 13. Brainstorm — related ideas (full list, essentials already promoted)

Answers the rev b prompt "brainstorm related ideas" — nothing dropped. Items marked **[Promoted]** are now core (see §7/§8); the rest stay as future options.

- **[Promoted] Title polish on approve.** Curator can edit title/body at approval time (already in bulk `title`/`body` fields). Rev c extends this to *any* polish at any time with immutable history (§6 `feature_cluster_revisions`, §7.1 polish handler, §8 History tab). Keeps Arena copy tight without rejecting a good idea for a bad title and without erasing the original.
- **[Promoted] Side-by-side duplicate view.** In the pending queue, show suggested target's `title+body` next to the new submission's `title+body` (diff-style) so the merge decision is one glance. Core in §8 curator queue.
- **[Promoted] Confidence bands.** Show suggestion strength as "Very likely duplicate (0.12)" / "Possible related (0.20)" / no hint, instead of a raw distance. Threshold bands: `<0.15` very likely, `<0.22` possible. Core in §7.1 `confidence_band` + §8 banner.
- **[Promoted] Search before submit.** As the user types the title, live-search existing `idea` titles (trigram `title ILIKE` + embedding) and show "Similar existing ideas" chips — reduces duplicates before they enter the queue. Core in §7.1 `GET /api/roadmap/search` + §8 live chips.
- **[Promoted] Author's pending shelf.** `GET /api/roadmap/my-pending` — "Your pending ideas" list so authors can withdraw/edit before review. Core in §7.2 + §8 shelf.
- **[Promoted] Withdraw / edit while pending.** Author can `PATCH` title/body only while `status='pending'` and they are the author (creates a revision). Curator edits always allowed (also revisioned). Core in §7.1 polish gate + §7.2 withdraw endpoint.
- **[Promoted] Merge consents / alias preservation.** When merging, keep the source title as an alias/alternative phrasing on the target (stored in `feature_cluster_aliases` side table and optionally appended to target body as "Also suggested as: …") so search finds it later. Core in §6 `feature_cluster_aliases` + §7.2 merge tx.
- **[Promoted] Rejection reasons as structured tags.** `rejection_kind` enum (`spam`, `duplicate`, `out-of-scope`, `needs-clarification`, `withdrawn`, `other`) + free-text `rejection_reason`, so the modlog and weekly digest can aggregate spam trends. Core in §6 `rejection_kind` + §7.2 bulk `rejection_kind`.
- **Category & tag suggestions.** LLM suggests `category` (search/scraper/social/reader/admin/recs/general) at submit time (advisory, curator confirms). Same pattern as clustering — never auto-sets. Future: add `suggested_category` column + show chip in pending queue.
- **Body templates.** Placeholder prompts in the body field: "Problem → Proposal → Alternatives considered" to nudge higher-quality pitches without mandating structure. Future: set as `placeholder` + helper text in the submit form; no backend change.
- **Auto-title from body.** If title is weak (e.g. <15 chars or generic), offer an LLM-generated title suggestion from the body before submit ("Did you mean: …?"). Future: call Ollama `generate` with a short prompt; keep it client-side suggestion only.
- **Related ideas graph on detail.** Detail page shows "Related ideas" (top 3 nearest `idea` clusters by embedding) — helps voters discover nuance and curators spot families to merge. Future: `GET /api/roadmap/features/:id/related` (nearest by vector).
- **Digest for curators.** Weekly DM/email "N pending ideas need review (3 suggested duplicates)" via the existing weekly-digest pipeline (`src/bin/trust_promote.rs` style). Future: extend digest query with `WHERE status='pending'`.
- **Reputation for good ideas.** On approval → small reputation bump; on merge → bump goes to the surviving cluster's authors (optional, ties into `reputation_events` + leaderboard without creating farm incentives). Future: award `reputation_events` with `event_type='roadmap_idea_approved'`.
- **Rate-limit by body length / link count.** Bodies with >2 links or >500 chars from TL1 get extra scrutiny (flag for curator, not block) — mirrors the existing trust-gate for posts with many links. Future: add a `flagged_for_review` boolean on pending if heuristics trip.
- **Future: two-stage clustering.** First pass trigram dedup (cheap, no Ollama) → only call Ollama if trigram similarity >0.3. Saves Ollama calls during spam bursts. Future: gate `embed` call behind `SELECT similarity(title, $1) > 0.3`.
- **Future (rev c): per-revision diff API.** `GET .../:id/history?diff=1` returns unified diffs between successive revs so the client doesn't have to compute them. Keep it simple for v1 (client diff), add server diff if history gets large.
- **Future (rev c): revert to revision.** Curator can `POST .../:id/revert {rev_no}` to append a new revision copying an old snapshot — "undo polish" without mutating history. Cheap to add after the revision table exists.

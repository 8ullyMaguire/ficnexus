## Part 7A — Build a Feature End to End (The Walkthrough You'll Reuse)

The single most valuable thing an onboarding guide can give you is a
complete, realistic feature build. Let's do one together. We'll add a tiny
feature that touches every layer: **`GET /api/ping` returning
`{"pong": true}`** — deliberately small, but it walks the entire pipeline
you'll repeat for every real feature.

### Step 1: The handler

In `src/routes/health.rs` (or a new file — let's reuse health since it's
the right neighborhood):

```rust
use axum::Json;
use serde_json::{json, Value};

/// GET /api/ping — trivial liveness probe
pub async fn ping_handler() -> Json<Value> {
    Json(json!({"pong": true}))
}
```

Notice: no `State` needed — this handler doesn't touch shared state. When
yours does, add `State(state): State<Arc<AppState>>`.

### Step 2: Register the route

In `src/server.rs`, inside `build_router()`, add:

```rust
.route("/api/ping", get(crate::routes::health::ping_handler))
```

Put it with the other health/liveness routes, BEFORE the ServeDir fallback.

### Step 3: A unit test (fast, no DB)

In `src/routes/health.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ping_returns_pong() {
        let body = ping_handler().await;
        assert_eq!(body.0["pong"], true);
    }
}
```

### Step 4: An integration test (DB-gated, optional for this size)

If the endpoint touches the DB or auth, add it to `tests/<area>_api.rs`
with the house pattern: `#[ignore]`, unique seed names, cleanup. For a pure
`/api/ping`, the unit test is enough.

### Step 5: Run the targeted suite

```bash
cargo test --lib health
```

### Step 6: The frontend (when the feature is user-visible)

Add a call in `frontend/src/lib/api/client.ts` if the page needs it, then
the page under `frontend/src/routes/`, then a `page.test.ts`.

### Step 7: Commit conventionally + verify

```bash
git add src/routes/health.rs src/server.rs
git commit -m "feat: add /api/ping liveness probe"
```

Then, for a real feature, run `hermes verify --save` for the canonical
gate.

That's it. Every feature — search filters, modlog entries, admin pages —
follows the same skeleton. The size of the feature changes what goes in
steps 1-6, not the order.

> 🧪 **Try it:** actually do the ping feature right now. It'll take ten
> minutes and permanently demystify the loop.
>
> ⚠️ **Watch out:** if your route path uses a param, use `{param}` (Axum
> 0.8), and remember `build_router` order matters vs the SPA fallback.
>
> 💡 **Key concept:** handler → route → test → frontend → commit. The
> loop is small; the discipline is doing it every time.

### Step 8: A bigger example — the modlog entry

Now let's make the loop concrete with a *real* feature shape: adding a
modlog entry to an admin action. Say we're adding a "delete user" admin
endpoint. The handler skeleton:

```rust
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,                      // JWT extractor
    Path(user_id): Path<i32>,
) -> AppResult<Json<Value>> {
    if auth.role < 10 {
        return Err(AppError::BadRequest(-403, "Admin access required".into()));
    }
    // ... actually delete the user (or soft-delete) ...
    sqlx::query("UPDATE users SET is_deleted = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    // TRANSPARENCY: record it in the modlog
    crate::modlog::record(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "delete_user",
        "user",
        &user_id.to_string(),
        serde_json::json!({"deleted": true}),
    ).await;
    Ok(Json(json!({"err": 0, "msg": "user deleted"})))
}
```

Notice the shape: **gate → mutate → record → respond.** That's the house
style for every admin/curator action. The modlog call is best-effort; even
if it fails, the action already happened.

> 🧪 **Try it:** pick an existing admin handler (e.g. in `admin.rs`) and
> identify the gate, the mutate, the record, and the respond steps. They're
> all there.
>
> ⚠️ **Watch out:** never log PII into the modlog `details` — it's
> readable by any logged-in user.
>
> 💡 **Key concept:** every admin action = gate → mutate → modlog →
> respond. The order is not arbitrary; the record is part of the product,
> not an afterthought.

### Chapter 26: Self-Healing Scraping (the Agent Loop)

FicHub has an ambitious self-healing system (`src/heal/`, migrations
029-030) for the scraper subsystem. When a scrape fails:

1. `state.heal.record_failure(url, error, snapshot)` writes to
   `scrape_failures` — including an HTML snapshot of the page when
   possible.
2. A pure-Rust classifier (`src/heal/classifier.rs`) labels the failure:
   **transient** (network blip), **blocked** (bot wall / 403), **structural**
   (the site changed its HTML so the parser breaks), or **systemic**.
3. Fingerprint + debounce dedupe repeated failures of the same URL.
4. An admin endpoint (`POST /api/admin/heal`) runs the diagnose-only loop
   today — it shows what it WOULD do.
5. The future (M2): an agent (Ollama or CommandCode) that CREATES a new
   scraper on the fly when the failure is structural, then the site uses it
   immediately.

Currently autonomy is OFF by default (`AGENT_ENABLED=false`) — the
diagnose-only loop is safe to run, and the agent loop is documented but
gated. This is a great example of "build the telemetry first, gate the
autonomy."

**Why the classification matters:** a transient failure (one network blip)
should be retried; a blocked failure (Cloudflare challenge) needs a
different strategy (user-supplied cookies, a proxy); a structural failure
(the site changed its HTML) means the scraper itself is broken and needs a
rewrite; a systemic failure (Postgres down) means nothing scraper-related
will help. The classifier's output drives what the healer does — and what
it records for human review.

> 🧪 **Try it:** look at `src/heal/classifier.rs` — the classification is
> pure Rust and unit-tested. Feed it a few failure strings and see what it
> labels them.
>
> ⚠️ **Watch out:** the agent loop is OFF. Don't enable it without the
> safety review documented in NEXT.md.
>
> 💡 **Key concept:** telemetry before autonomy. Record failures,
> classify them, dedupe, and only then consider automated healing.

### Chapter 27: Curator Content Fixes (Peer-Voted Bodies)

The body cache stores scraped content — but scrapes can be wrong (e.g. a
forum thread where the fic is post #2 and the "chapters" include the OP
announcement and comments). Curators fix this with a peer-voted workflow
(`src/routes/curator_content.rs`, migration 032):

1. `POST /api/curator/content/{url_id}/propose` — a curator proposes a
   corrected body.
2. Other curators vote up/down (`POST /api/curator/content/proposals/{id}/vote`,
   no self-vote).
3. At quorum (≥2 votes) with net ≥ 1, the fix is applied to the body cache
   (`save_body` + version bump) and recorded in the modlog.
4. `POST /api/curator/content/{url_id}/delete` removes a cached body.

This is the "human-in-the-loop" correction path for the cache: the scraper
doesn't guess (per product decision, "don't auto-guess fic content"), and
humans fix bodies post-hoc with votes.

**Why peer-voted and not "curator edits directly"?** Because the cache is
a public asset. One curator's mistake would corrupt the archive for
everyone; requiring a second curator's vote catches errors and keeps
quality high without making fixes slow (quorum is only 2).

> 🧪 **Try it:** read `curator_content.rs` and trace a proposal through to
> application. Note where it writes to the body cache vs the DB.
>
> ⚠️ **Watch out:** the quorum threshold is small (≥2 votes). Don't loosen
> it casually — peer review is the quality gate.
>
> 💡 **Key concept:** the cache is correctable. Propose → vote → apply
> gives human oversight to automated scraping.

### Chapter 28: The AI Features (Auto-Tagger & Translations)

Two more Ollama-backed features complete the picture:

**Auto-tagger** (`src/routes/auto_tag.rs`): given a fic's metadata/body,
Ollama suggests tags. Suggestions land as drafts; a curator approves or
rejects them (the review UI is a starter task). This directly improves
search quality — more accurate tags → better `main_char_attr` and trope
searches.

**Translations** (`src/routes/locales.rs`, migrations 023-025): ML
translation of fic metadata/content, with a human post-edit workflow.
Approved translations are stored and served in the reader.

Both follow the same pattern as Ask the Archive: LLM proposes, human
confirms, the result is stored deterministically. The LLM is never the
source of truth by itself.

**The pattern in code:**

```rust
// 1. Model proposes
let suggestion = ollama::suggest_tags(&state, &fic).await?;
// 2. Store as a DRAFT (never approved directly)
sqlx::query("INSERT INTO tag_suggestions (url_id, tag_name, status) VALUES ($1, $2, 'draft')")
    .bind(&fic.url_id).bind(&suggestion).execute(&state.db).await?;
// 3. Human approves later via the review endpoint (status → 'approved')
```

> 🧪 **Try it:** find the auto-tag endpoint and see how a suggestion flows
> to a draft. Then find where a human approves it.
>
> ⚠️ **Watch out:** ML outputs are drafts by design. Never let an LLM
> write directly to user-visible "approved" state without a human gate.
>
> 💡 **Key concept:** human-in-the-loop AI. The model proposes; a human
> disposes; the DB stores the human-confirmed result.

### Chapter 29: The Hidden Complexity — Worker Tasks & Background Jobs

Not everything happens in request handlers. FicHub has background workers
spawned in `server::run()`:

- **The recommender worker** (`src/recommender/worker.rs`) — maintains
  co-occurrence data, imports AO3 profiles (the legacy `fic_bookmarks`
  path), and refreshes rec-relevant statistics. It uses Redis for queues —
  including that BRPOP we met in Chapter 6.
- **The bookmark-import worker** — the one that parks a blocking BRPOP on
  the shared Redis connection. Its job is to watch a queue of AO3 profile
  URLs and import their bookmarks into `fic_bookmarks`.

When you add a background job, the pattern is: spawn it in `run()`, give it
`AppState` (or the specific connections it needs), log with tracing, and
make it resilient to failures (don't crash the server if a job errors).

> 🧪 **Try it:** find where workers are spawned in `server::run()` and list
> what each one does.
>
> ⚠️ **Watch out:** a worker that blocks forever on a connection can break
> health checks (the Redis story). Keep workers' blocking operations off
> shared connections.
>
> 💡 **Key concept:** request handlers are the front door; workers are the
> kitchen staff. Both matter, and both must be failure-tolerant.

---

## Where to Go Next

You've now seen: the one-binary architecture, the config struct, the
query module, the scraper registry, the body cache, the export pipeline,
the search parser, the rec platform, the anti-bot layers, the consensus
engine, self-healing, curation, and the AI features. That's the whole
product, end to end.

The best next step is to close this book and build something small — the
ping feature from the walkthrough, or one of the starter tasks from the
workflow chapter. The repo is well-commented; the docs
(`docs/ROADMAP.md`) are the living plan; and now you know the vocabulary
to ask good questions.

Welcome to FicHub. Happy building.


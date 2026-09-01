# Self-Healing & Scrape Failure Telemetry

> Status: milestone 1 (failure telemetry + diagnose-only agent trigger)
> implemented. Autonomy is OFF by default (`AGENT_ENABLED=false`).

FicHub records every scrape failure, classifies it, and (when enabled) asks
an LLM agent to diagnose the cause. This chapter is for operators running
the live instance.

## Site as a cache of gathered fanfiction

Every successfully scraped fic body is persisted on the attached drive
(`BODY_CACHE_DIR`, default `/public/literature/fichub/bodies`), NOT in the
database. Layout is two-level hex sharding on `url_id` (git-objects style),
versioned per fix:

- `bodies/{url_id[0:2]}/{url_id[2:4]}/{url_id}.v{version}.html` — the
  extracted body HTML (so a bad extraction can be re-done from saved
  content without re-scraping).
- `bodies/{url_id[0:2]}/{url_id[2:4]}/{url_id}.v{version}.json` — the
  structured chapters the exports render.

Exports reuse the cached body (no re-scrape). Version bumps GC old files
for the same url_id (bounded disk, no background job).

Curator fixes go through **peer voting** — a single curator cannot silently
replace a fic's content:

- `POST /api/curator/content/{url_id}/propose` — submit a fix proposal
  (body_html + reason) → `pending`.
- `POST /api/curator/content/proposals/{id}/vote` — other curators vote
  up/down (no self-vote). At quorum (≥2 votes) with net ≥ 1 the fix is
  **applied** (body blob written at the next version); net < 0 → rejected.
- `GET /api/curator/content/proposals?status=pending` — review queue.
- `GET /api/curator/content/{url_id}` — inspect what was gathered.
- `DELETE /api/curator/content/{url_id}` — delete the blob, force a
  re-scrape (safe: only triggers re-scrape, doesn't replace content).

All gated to role ≥ 10 (400-as-403 convention).

## Failure telemetry

Every failed scrape lands in the `scrape_failures` table:

| Column | Meaning |
|--------|---------|
| `url` / `url_id` | the requested fic URL |
| `domain` | host (e.g. `royalroad.com`) |
| `error_kind` | `blocked` / `timeout` / `parse` / `not_found` / `export` / `unknown` |
| `message` | scraper error text |
| `html_snapshot_path` | captured page (parse failures only, sanitized, ≤200KB) |
| `fingerprint` | stable hash — dedupes identical failures |
| `created_at` / `resolved_at` / `resolution` | lifecycle |

On parse failures the offending HTML is saved under
`tests/fixtures/scrape/<domain>/<fingerprint>.html` — it doubles as agent
context and as a future regression fixture.

## Classifier

Pure Rust (no LLM), maps an error to a healing class:

- **transient** — 429/timeouts/525/not-found: back off, no agent.
- **blocked** — 403 / Cloudflare from known-bad hosts (AO3, FFN,
  FictionPress): the agent cannot fix an IP-level block; surface to users.
- **structural** — selector/parse error: agent territory (site layout
  changed).
- **systemic** — 5xx / panic / build failure: service-level playbook.

Debounce: ≥3 same-domain structural failures in 15 min → one healing event.
Fingerprint: one heal per fingerprint per 24h unless it changes.

## Agent runtime

- Endpoint (OpenAI-compatible `chat/completions`): CommandCode
  `https://api.commandcode.ai/provider/v1` — key in
  `COMMANDCODE_API_KEY` / `AGENT_API_KEY`, model `poolside/laguna-s-2.1-free`
  (override `AGENT_MODEL`). Local fallback: `AGENT_OLLAMA_URL`.
- Diagnose-only this pass: `POST /api/admin/heal?domain=X` (role ≥ 10)
  returns recent failures + a plan, and if `AGENT_ENABLED=true` makes one
  diagnose call (30s timeout) recording the reply in `agent_runs`.
  It never writes files or modifies code.

## Config

| Env | Default | Meaning |
|-----|---------|---------|
| `AGENT_ENABLED` | `false` | master switch (healing loop + agent calls) |
| `AGENT_MODEL` | `deepseek/deepseek-v4-flash` | agent model (deploy uses `poolside/laguna-s-2.1-free`) |
| `AGENT_API_KEY` / `COMMANDCODE_API_KEY` | — | remote API key |
| `AGENT_API_URL` | `https://api.commandcode.ai/provider/v1` | OpenAI-compatible base |
| `AGENT_OLLAMA_URL` | `http://localhost:11434` | local fallback |
| `AGENT_MAX_RUNS_PER_DAY` | `6` | daily budget |
| `AGENT_COOLDOWN_DOMAIN_SECS` | `3600` | per-domain cooldown |
| `AGENT_USE_ON_FLY` | `false` | M2: on-the-fly extraction on structural failure (needs `AGENT_ENABLED=true`) |
| `AGENT_EXTRACT_MAX_SNAPSHOT_CHARS` | `120000` | max snapshot HTML sent to the extraction agent |

## What's next

See `docs/brainstorm-02-roadmap-status.md` — T1 auto-merge + deploy/rollback
harness, T2 service heal (health watcher → restart), T3 QA-driven site
regen, `/admin/agent` UI panel.

# Roadmap Implementation Plan — fanfic-archivist Bot

**Audience:** Junior developers contributing to the FicHub Discord companion bot.
**Scope:** Everything implemented in the 2026-08-26 session plus the remaining roadmap items still pending.
**Status:** Session work done (commands, API methods, store methods, registrations). Infra + ROADMAP updates + a few bot-only commands still to land.

---

## 1. Architecture Overview (read this first)

### Two repos, two roles

| Repo | Path | Role |
|------|------|------|
| **Bot repo (canonical)** | `/home/alvaro/.cache/fanfic-archivist-repo/` | Multi-platform bot source of truth. 8 crates, edition 2024. `archivist-core` holds shared API client + dispatch + intent; platform crates (Discord, Telegram, IRC, Slack, Matrix, Fediverse) are adapters. |
| **FicHub-side** | `/home/alvaro/code/rust/fichub/fanfic-archivist/` | Thinner Discord-only companion. Own `api.rs`, `store.rs`, `commands/`, `main.rs`. Mirrors the batch endpoints but is a separate client. |

**Key rule:** The FicHub-side `ROADMAP.md` is *stale* — it described a thin Discord bot. The canonical source of truth is the bot repo's `ROADMAP.md`. When in doubt, read the bot repo.

### Request flow (every command)

```
Discord message → poise command → require_token/require_level → FichubClient::method() → reqwest → FicHub REST API → JSON → user-facing embed/message
```

Key helpers in `commands/mod.rs` (bot repo) and `commands/mod.rs` (FicHub-side):
- `require_token(tokens, discord_id)` — looks up the stored JWT for a Discord user; returns `NotLinked` if missing.
- `require_level(tokens, discord_id, min_level)` — same but enforces a minimum FicHub trust level (0 = member, 1 = reader, 2 = reviewer, 3 = curator, 4 = moderator, 5+ = admin). Used by privileged commands.

### API client pattern

Both `api.rs` files follow the same shape:

```rust
impl FichubClient {
    async fn get<T: DeserializeOwned>(&self, path: &str, auth_token: Option<&str>) -> Result<T>;
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B, auth_token: Option<&str>) -> Result<T>;
    async fn delete<T: DeserializeOwned>(&self, path: &str, auth_token: Option<&str>) -> Result<T>;
    // ...one method per endpoint...
}
```

- Public endpoints (no auth): pass `None` as the token.
- Auth endpoints: pass `Some(token)`.
- The `parse_response` helper checks HTTP status + the `err` field (0 = ok) and returns typed JSON.

### Store pattern

`store.rs` holds Redis-backed state:
- `TokenStore` — `set`/`get`/`delete`/`touch` per Discord user id → JWT.
- `PendingLinkStore` — one-time `/link` codes (8-char alphanumeric, unambiguous alphabet).
- Preference methods (added in this session) — `set_filter_chips`, `get_filter_chips`, `set_track_state`, `get_track_state`, `set_full_favourites`, `get_full_favourites`, `set_recs_cutoff`, `get_recs_cutoff`, `set_recs_wordcount_range`, `get_recs_wordcount_range`, `set_recs_year`, `get_recs_year`.

### Command registration

Every new command must be registered in **two places**:
1. `commands/mod.rs` — `pub mod <name>;`
2. `main.rs` — `commands::<name>::<fn>(),` inside the `commands()` vec.

Both repos need this. The bot repo also has `command_list()` (used by `/commands`) and `require_level_for` (used by level-gated commands) — update these when adding new commands.

---

## 2. What Was Implemented This Session

### 2.1 Follow exclusions (`!exclude`)

**What it does:** Hides a work, series, or fandom from a user's follow updates feed. Server-side anti-join removes excluded items from `GET /api/v1/updates`.

**API methods added (bot repo `archivist-core/src/api.rs`):**
- `add_follow_exclusion(token, follow_id, body)` → `POST /api/v1/follows/{follow_id}/exclusions`
- `list_follow_exclusions(token, follow_id)` → `GET /api/v1/follows/{follow_id}/exclusions`
- `delete_follow_exclusion(token, follow_id, exclusion_id)` → `DELETE /api/v1/follows/{follow_id}/exclusions/{exclusion_id}`

**Body struct (`FollowExclusionBody`):**
```rust
pub struct FollowExclusionBody {
    pub exclude_type: String,  // "work" | "series" | "fandom"
    pub work_id: Option<i32>,
    pub series_id: Option<i32>,
    pub fandom: Option<String>,
}
```

**Model struct (`FollowExclusion` in `model.rs`):** Uses different field names (`exclude_work_id`/`exclude_series_id`/`exclude_fandom`/`created_at`/`err`) — this is the response shape, not the request body. Don't confuse the two.

**Command (`exclude.rs`):** Three subcommands:
- `/exclude <work_id>` — exclude a work
- `/exclude series <series_id>` — exclude a series
- `/exclude fandom <fandom>` — exclude a fandom

Level-gated to 1 (linked members). Uses `require_level`.

**Intent wiring:** `Intent::Exclude { work_id, series_id, fandom }` variant added to `intent.rs`, dispatched in `dispatch.rs` via `do_exclude_work`/`do_exclude_list`/`do_exclude_remove`, and wired into all 9 platform adapters (Discord, Telegram, IRC, Slack, Matrix, Fediverse Mastodon, Fediverse Bsky, WS, REPL).

**FicHub-side:** Same API methods in `api.rs`, same command in `commands/exclude.rs`, registered in `mod.rs` + `main.rs`.

### 2.2 Saved-search alerts (`/daily`)

**What it does:** Runs a user's saved searches and reports new matches. Three subcommands:
- `/daily [N]` — run all (or newest N) saved searches
- `/daily off <search_id>` — turn off daily alerts
- `/daily on <search_id>` — turn on RSS alerts

**API methods:**
- `create_saved_search(token, name, query_text)` → `POST /api/search/saved`
- `list_saved_searches(token)` → `GET /api/search/saved`
- `delete_saved_search(token, id)` → `DELETE /api/search/saved/{id}`
- `update_saved_search_alert(token, id, alert_mode)` → `PUT /api/search/saved/{id}/alert`
- `run_saved_search(token, id)` → `POST /api/search/saved/{id}/run`

**Implementation note:** The command re-runs each saved search and shows top 3 results per search. It does NOT rely on the server's `last_run_at` tracking — it just re-runs and displays. This is a deliberate simplification.

### 2.3 Community suggestion (`/suggest`)

**What it does:** Lets a reader propose a fic for the recommendation engine.

**API method:** `suggest_fic(token, url_id, comment)` → `POST /api/recommendations/suggest`

**Command:** Takes a URL, resolves it to a `url_id` via `fetch_export`, then calls `suggest_fic`. Returns the suggestion id + net votes.

### 2.4 Notifications (`/notify`)

**What it does:** Lists recent notifications + marks all as read.

**API methods:**
- `notifications(token, limit, offset)` → `GET /api/notifications`
- `unread_notification_count(token)` → `GET /api/notifications/unread-count`
- `mark_all_notifications_read(token)` → `POST /api/notifications/read-all`

**Important limitation:** Fic-update subscription webhooks (push notifications when followed fics get new chapters) are NOT available — the server has no `/api/notifications/subscribe-fic` or fic-update webhook endpoint. Only the notification history list + mark-read are implementable. The command file documents this clearly.

### 2.5 Curator (`/curator`)

**What it does:** Shows pending curator approvals queue + curator leaderboard.

**API methods:**
- `curator_approvals(token, status, item_type, curator)` → `GET /api/curator/approvals`
- `curator_leaderboard(token, limit)` → `GET /api/leaderboard/curators?limit=`

**Command:** Two subcommands:
- `/curator [status]` — show approvals (pending|approved|rejected, default: pending)
- `/curator leaderboard [limit]` — show top curators

Role-gated to 5 (curator+).

### 2.6 Roadmap voting (`/vote`)

**What it does:** Submits a MaxDiff vote on the roadmap arena.

**API methods:**
- `roadmap_arena()` → `GET /api/roadmap/arena` (fetches current 4-cluster arena)
- `vote_roadmap(token, cluster_ids, best, worst)` → `POST /api/roadmap/vote`

**Command:** Takes best + worst cluster ids, validates they're in the arena, submits the vote, shows Elo shifts.

### 2.7 Authors (`/authors`)

**What it does:** Shows an author's bibliography.

**API method:** `author_bibliography(author_name)` → `GET /api/authors/by-name/{name}`

**Command:** Takes an author name, fetches bibliography, shows up to 10 works as embed fields.

### 2.8 Stats (`/stats`)

**What it does:** Shows the user's reading stats.

**API method:** `my_reading_stats(token)` → `GET /api/users/me/reading-stats`

**Command:** Shows total words, books read, fandoms explored, avg words/book, top fandom, crossover ratio.

### 2.9 Filter chips (`!filters`/`!reset-filters`)

**What it does:** Bot-only client-side filter chips that affect recommendation behavior. No server endpoint needed.

**Store methods:** `set_filter_chips`, `get_filter_chips` — persisted in Redis as `archivist:filters:{discord_id}`.

**Command:** `!filters [chip1 chip2 ...]` — sets filter chips (valid: completed, hiatus, unpopular, classic, long, short, top-rated, new). `!reset-filters` — clears all.

### 2.10 Full favourites (`!full-favourites`)

**What it does:** Bot-only — paginates the user's full bookmarked list.

**Store methods:** `set_full_favourites`, `get_full_favourites` — cached in Redis as `archivist:fullfav:{discord_id}`.

**API method:** `list_bookmarks(token, limit, offset)` → `GET /api/bookmarks`

**Command:** Fetches + caches the full bookmark list, then paginates (10 per page).

### 2.11 Tracked-message state machine (`/track`)

**What it does:** Bot-only — tracks a message through states: unseen → seen → saved → archived.

**Store methods:** `set_track_state`, `get_track_state` — persisted in Redis as `archivist:track:{discord_id}:{url_id}`.

**Command:** `/track <url_id> [state]` — set a specific state, or "advance" to move to the next state.

### 2.12 Bot-only recs tuners (cutoff, wordcount, year)

**What they do:** Persist per-user recs preferences in Redis.
- `!cutoff <N>` — set minimum wordcount
- `!wordcount <min>-<max>` — set wordcount range
- `!year <year>` — filter by publication year

**Store methods:** `set_recs_cutoff`/`get_recs_cutoff`, `set_recs_wordcount_range`/`get_recs_wordcount_range`, `set_recs_year`/`get_recs_year`.

### 2.13 Upload (bot-only, not yet implemented)

**What it would do:** Direct EPUB upload via Discord attachment (`poise::CreateAttachment` from bytes), for files < 25 MiB. Server has `GET /api/works/{id}/download` returning EPUB but no upload endpoint yet.

---

## 3. Remaining Roadmap Items (still pending)

### 3.1 Blocked — no server endpoint

These cannot be implemented without server changes:

| Item | Why blocked |
|------|-------------|
| `/notify` fic-update subscription | No `/api/notifications/subscribe-fic` or webhook endpoint |
| `/mood` (fandom mood) | No `/api/fandoms/{fandom}/mood` or mood-tag endpoints |
| OPDS/PWA deep-link | No server OPDS endpoint |
| Guild-level allowlist | No server-side guild config endpoint (denylist on token is feasible; allowlist needs server) |
| `/download` direct upload | Server has download but no upload endpoint yet |

### 3.2 Partially implemented — needs completion

| Item | Status |
|------|------|
| Tracked-message state machines (`!sus`, `!stats`, `!filters`, `!reset-filters`, `!full-favourites`, `!cutoff`, `!wordcount`, `!year`) | Infrastructure added (store methods + `/track` command). `!sus` (suspicious-author) needs server rarity-tier weighting. |
| Author recs (`/authors`) | Implemented as author bibliography browse. Server has `GET /api/authors/by-name/{name}` but not a recommendation endpoint. |
| Filter-chip select menus | `!filters`/`!reset-filters` implemented as text commands. Discord select menus (UI) not yet built. |
| Guild-level denylist | Client-side deny-list on token is feasible; not yet implemented. |

### 3.3 Infra — needs writing

| Item | Status |
|------|------|
| Systemd unit file | Not written |
| Dockerfile | Not written |

### 3.4 ROADMAP.md updates — needs final pass

| File | Status |
|------|--------------|
| `/home/alvaro/code/rust/fichub/fanfic-archivist/ROADMAP.md` | Follow-exclusions marked `[x]`. Remaining `[ ]` items need final review. |
| `/home/alvaro/.cache/fanfic-archivist-repo/ROADMAP.md` | Canonical; needs sync with shipped state. |

---

## 4. Common Pitfalls (read before implementing anything)

1. **Two `FollowExclusionBody` definitions:** The struct must live at module scope in `api.rs`, NOT inside an `impl` block. If you see "struct-in-impl" errors, move it out.

2. **Field name mismatch:** The POST body uses `work_id`/`series_id`/`fandom` (serde `rename` to `exclude_type` for the type field). The response model uses `exclude_work_id`/`exclude_series_id`/`exclude_fandom`. Client methods must serialize the body with server field names and deserialize the response with model field names.

3. **Dereferencing `Option<i64>`:** `work_id` fields are `Option<i64>`. Using `*work_id as i32` fails — use `work_id.unwrap_or(0) as i32` or `w as i32` (when destructured by value).

4. **`PlatformMessage::Text` is a tuple variant:** Not a struct. Use `PlatformMessage::text(...)` and `PlatformMessage::ephemeral(...)` constructors, not struct patterns.

5. **`BotError::ApiError` doesn't exist:** Use `BotError::Status { status, body }` or `BotError::Api { err, msg }`.

6. **The lint tool uses Rust 2015:** It produces false-positive `async fn` errors across the entire file. The real toolchain is edition 2024. Ignore lint tool errors; verify with `cargo check -p <crate>` using `CARGO_TARGET_DIR=/media/alvaro/code-worktrees/.fichub-main-target`.

7. **Register in two places:** Every new command needs `pub mod <name>` in `commands/mod.rs` AND `commands::<name>::<fn>()` in `main.rs`. Both repos. Miss one and you get "unresolved import" or "unregistered command" errors.

8. **Slack test flake:** `config::tests::default_loads_env_tokens` fails due to pre-existing `SLACK_ALLOWED_CHANNELS` env pollution from another test. Passes under `--test-threads=1`. Not a roadmap regression.

9. **`mint_token` exists:** It's at `archivist-core/src/api.rs:350-369`. Don't reimplement it.

10. **FicHub-side `api.rs` is separate from bot repo's `archivist-core/src/api.rs`:** Both need the batch endpoint methods. When adding a method to one, add it to the other.

---

## 5. File Map (where everything lives)

### Bot repo (canonical)
```
crates/archivist-core/src/api.rs       — FichubClient methods (all endpoints)
crates/archivist-core/src/model.rs     — response/request structs
crates/archivist-core/src/intent.rs    — Intent enum + classification
crates/archivist-core/src/dispatch.rs  — do_* dispatch functions
crates/archivist-core/src/store.rs     — TokenStore + PendingLinkStore + pref methods
crates/fanfic-archivist/src/commands/  — Discord command files
crates/fanfic-archivist/src/main.rs    — command registration + event loop
```

### FicHub-side
```
src/api.rs           — FichubClient methods (mirrors batch endpoints)
src/model.rs         — response/request structs
src/store.rs         — TokenStore + PendingLinkStore + pref methods
src/commands/        — Discord command files
src/main.rs          — command registration + event loop
ROADMAP.md           — project roadmap (stale; update carefully)
```

---

## 6. How to Test Your Changes

```bash
# Bot repo
cd /home/alvaro/.cache/fanfic-archivist-repo
unset GIT_OBJECT_DIR GIT_ALTERNATE_OBJECT_DIRECTORIES
CARGO_TARGET_DIR=/media/alvaro/code-worktrees/.fichub-main-target cargo test --workspace -- --test-threads=1

# FicHub-side
cd /home/alvaro/code/rust/fichub/fanfic-archivist
CARGO_TARGET_DIR=/media/alvaro/code-worktrees/.fichub-main-target cargo check
```

**Always use `--test-threads=1`** — the Slack config test flakes under parallel execution due to env pollution.

---

## 7. Conventional Commits

This project uses conventional commits. Format:
```
feat: <description>
fix: <description>
docs: <description>
```

Examples from this session:
- `feat: follow-exclusion client methods, Intent::Exclude, and !exclude command`
- `feat: saved-search alerts command (/daily) + API methods`
- `feat: community suggestion, notification, curator, vote, authors, stats commands`
- `feat: filter chips, full-favourites, tracked-message state machine`
- `docs: mark follow-exclusions wired in bot (P1 2026-08-25 batch)`

---

*End of plan. Implementation tutorials for individual features follow in separate files.*

# Code-Along Tutorial: Implementing a New Discord Command

**Audience:** Junior developers adding a command to the fanfic-archivist bot.
**Prerequisites:** Read `docs/roadmap-implementation-plan.md` first.
**Expected time:** 15-30 minutes per command.

---

## Part 1: The CLI Companion (FicHub-side)

The FicHub-side bot at `/home/alvaro/code/rust/fichub/fanfic-archivist/` is a thinner Discord-only companion. This tutorial covers adding a command there.

### Step 1: Check if the server endpoint exists

Before writing any code, verify the server endpoint exists. The server is at `/home/alvaro/code/rust/fichub/src/server.rs` with route handlers in `src/routes/`.

```bash
cd /home/alvaro/code/rust/fichub
grep -n 'route.*/your/endpoint' src/server.rs
```

If the endpoint doesn't exist, you can only implement bot-only features (client-side state in Redis). Document the limitation in your command file.

**Example — checking for a hypothetical `/api/works/{id}/mood` endpoint:**

```bash
grep -n 'mood' src/server.rs src/routes/*.rs
# If nothing comes back, the endpoint doesn't exist → bot-only or blocked.
```

### Step 2: Add the API client method

Open `src/api.rs`. Find the `impl FichubClient` block. Add a method following the existing pattern:

```rust
// ── Your feature (your-section-name) ─────────────────────────────────
// Routes:
//   GET  /api/your/endpoint              (description)

/// `GET /api/your/endpoint` — description of what it does.
pub async fn your_method_name(
    &self,
    token: &str,           // auth token, or use None if public
    param: &str,           // some parameter
) -> Result<serde_json::Value> {
    self.get(
        &format!("/api/your/endpoint/{}", urlencoding::encode(param)),
        Some(token),        // or None if the endpoint is public
    )
    .await
}
```

**Rules:**
- Public endpoints → pass `None` as the token argument.
- Auth endpoints → pass `Some(token)`.
- If the endpoint takes a path parameter, URL-encode it with `urlencoding::encode`.
- If the endpoint takes a query parameter, build the query string manually.
- Return type: use `Result<serde_json::Value>` for flexible JSON, or a typed model struct from `model.rs` if one exists.
- Add a doc comment with the route and a description.

**Example — adding `roadmap_arena`:**

```rust
/// `GET /api/roadmap/arena` — current 4-cluster MaxDiff arena.
pub async fn roadmap_arena(&self) -> Result<serde_json::Value> {
    self.get("/api/roadmap/arena", None).await
}
```

**Example — adding a method that takes a path parameter:**

```rust
/// `GET /api/authors/by-name/{name}` — author bibliography.
pub async fn author_bibliography(&self, author_name: &str) -> Result<serde_json::Value> {
    let name = urlencoding::encode(author_name);
    self.get(&format!("/api/authors/by-name/{name}"), None).await
}
```

### Step 3: Add a response model (if needed)

Open `src/model.rs`. If the endpoint returns a structured JSON shape you'll use in multiple places, add a model struct:

```rust
/// Response from `GET /api/your/endpoint`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YourResponse {
    pub err: i32,
    #[serde(default)]
    pub data: Option<YourData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YourData {
    pub id: i64,
    pub name: String,
    // ...
}
```

**Rules:**
- Always include `err: i32` — the FicHub API uses this for error reporting (0 = ok).
- Use `#[serde(default)]` for optional fields.
- Add `Clone + Deserialize + Serialize` derives.
- If the shape is only used once in the command, skip the model and use `serde_json::Value` directly.

### Step 4: Create the command file

Create `src/commands/your_command.rs`:

```rust
//! Your command: `/your-command <args>` — brief description.
//!
//! Wraps `METHOD /api/your/endpoint` (section name).

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/your-command <arg>` — description.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("your-alias"),
    description_localized("en-US", "Human-readable description of the command"),
    ephemeral            // use this for personal actions (bookmark, rate, etc.)
                       // omit for public actions (search, recs, etc.)
)]
pub async fn your_command(
    ctx: Context<'_>,
    #[description = "Argument description"] arg: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // 1. Get the auth token (or handle unlinked case)
    let token = match require_token(&data.tokens, ctx.author().id.get()).await {
        Ok(t) => t,
        Err(_) => {
            ctx.say("Not linked. Run `/link` first.").await?;
            return Ok(());
        }
    };

    // 2. Touch the token (refresh TTL on successful use)
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    // 3. Call the API
    match data.client.your_method_name(&token, &arg).await {
        Ok(resp) => {
            // 4. Build the response
            let value = resp.get("field_name").and_then(|v| v.as_str()).unwrap_or("?");
            ctx.say(format!("Result: {}", value)).await?;
        }
        Err(e) => {
            ctx.say(format!("Error: {e}")).await?;
        }
    }

    Ok(())
}
```

**Rules:**
- Use `ephemeral` for personal actions (only the invoking user sees the response).
- Use `require_token` for auth commands; handle the `NotLinked` case gracefully.
- Always call `touch()` after successful auth to refresh the token TTL.
- Extract JSON fields using `.get("field").and_then(|v| v.as_type())`.
- Use `truncate()` from `crate::util` for long strings in embeds.

**Example — a command with no auth (public endpoint):**

```rust
/// `/authors <name>` — show an author's bibliography.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("authors"),
    description_localized("en-US", "Browse an author's bibliography"),
    ephemeral
)]
pub async fn authors(
    ctx: Context<'_>,
    #[description = "Author name (canonical)"] name: String,
) -> Result<(), Error> {
    let data = ctx.data();
    // No auth needed — public endpoint
    match data.client.author_bibliography(&name).await {
        Ok(resp) => {
            let works: Vec<serde_json::Value> = resp
                .get("works")
                .and_then(|v| v.as_array())
                .map_or_else(|| vec![], |a| a.to_vec());
            if works.is_empty() {
                ctx.say(format!("No works found for author `{}`.", name)).await?;
                return Ok(());
            }
            let mut msg = format!("**{}** — {} works\n", name, works.len());
            for w in works.iter().take(10) {
                let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                msg.push_str(&format!("- {}\n", truncate(title, 80)));
            }
            ctx.say(msg).await?;
        }
        Err(e) => ctx.say(format!("Could not fetch bibliography: {e}")).await?,
    }
    Ok(())
}
```

### Step 5: Register the command (two places)

**Place 1: `src/commands/mod.rs`** — add the module declaration:

```rust
pub mod your_command;   // add this line, alphabetically
```

**Place 2: `src/main.rs`** — add the command to the `commands()` vec:

```rust
fn commands() -> Vec<poise::Command<Data, BotError>> {
    vec![
        // ... existing commands ...
        commands::your_command::your_command(),
    ]
}
```

**Also update `command_list()` in `mod.rs`** if you want the command to appear in `/commands`:

```rust
pub fn command_list() -> Vec<(&'static str, &'static str)> {
    vec![
        // ...
        ("your-alias", "Human-readable description"),
    ]
}
```

### Step 6: Add a level gate (if needed)

If the command needs a minimum FicHub trust level, update `require_level_for` in `mod.rs`:

```rust
pub fn require_level_for(cmd: &str) -> i16 {
    match cmd {
        "your-command" => 1,   // 1 = reader level
        _ => 0,
    }
}
```

Then use `require_level` instead of `require_token` in your command:

```rust
let token = require_level(&data.tokens, ctx.author().id.get(), require_level_for("your-command")).await?;
```

### Step 7: Verify

```bash
cd /home/alvaro/code/rust/fichub/fanfic-archivist
CARGO_TARGET_DIR=/media/alvaro/code-worktrees/.fichub-main-target cargo check
```

If `cargo check` passes, the command compiles. The lint tool may report false-positive `async fn` errors (Rust 2015 false positives — the real toolchain is edition 2024). Ignore those.

### Common mistakes

| Symptom | Cause | Fix |
|---------|-------|-----|
| "unresolved import" | Forgot `pub mod your_command` in `mod.rs` | Add the module declaration |
| "unregistered command" | Forgot `commands::your_command::your_command()` in `main.rs` | Add to the `commands()` vec |
| "NotLinked" on every call | Forgot to handle the `Err` case from `require_token` | Add the match/if-let for the error |
| Token never refreshes | Forgot `touch()` after successful auth | Add `data.tokens.touch(...)` |
| Wrong JSON field | Used wrong field name in `.get("...")` | Check the server route handler to see the actual JSON keys |
| "async fn not permitted in Rust 2015" | Lint tool false positive | Ignore — verify with real `cargo check` |

---

## Part 2: The Bot Repo (Canonical, Multi-Platform)

The bot repo at `/home/alvaro/.cache/fanfic-archivist-repo/` is the canonical multi-platform source. The Discord adapter is at `crates/fanfic-archivist/`. Adding a command here follows the same pattern as Part 1, with these differences:

### Differences from the FicHub-side

1. **API methods go in `crates/archivist-core/src/api.rs`**, not `src/api.rs`.
2. **Store methods go in `crates/archivist-core/src/store.rs`**, not `src/store.rs`.
3. **Command files go in `crates/fanfic-archivist/src/commands/`**.
4. **The `impl FichubClient` block is much larger** (1500+ lines, 90+ methods). Find the right section by searching for existing related methods.
5. **Dispatch functions:** The bot repo has a `dispatch.rs` layer that maps `Intent` variants to `do_*` functions. If your command needs an `Intent` variant, you must add it to `intent.rs`, add a dispatch function to `dispatch.rs`, and wire it into all platform adapters.

### Adding an Intent variant (if needed)

**Step A: `crates/archivist-core/src/intent.rs`**

```rust
// Add to the Intent enum:
Intent::YourIntent { field: i64 },

// Add to describe():
Intent::YourIntent { field } => format!("your intent for {}", field),

// Add to ACTION_WHITELIST:
"your-intent-action" => true,

// Add to sanitize():
// (heuristic keyword matching — only if the LLM/ASK pipeline needs it)

// Add to from_json():
// (if the intent can come from a JSON payload)

// Add to the LLM prompt entry:
// (so the LLM can suggest this intent)
```

**Step B: `crates/archivist-core/src/dispatch.rs`**

```rust
pub async fn do_your_intent(core: &CoreCtx, token: &str, param: i64) -> Result<PlatformMessage> {
    // Call the API client method, build a PlatformMessage
    let result = core.client.your_method(&token, param).await?;
    Ok(PlatformMessage::text(format!("Result: {:?}", result)))
}
```

**Step C: Wire into each platform adapter**

Every adapter's `main.rs` or `handlers.rs` has a match on `Intent`. Add your variant:

```rust
Intent::YourIntent { field } => {
    // Get the token for this user (pattern varies by platform)
    let token = /* ... */;
    archivist_core::dispatch::do_your_intent(&core, &token, field).await
}
```

Platform-specific patterns:
- **Discord:** `data.tokens.get(&msg.author.id.get().to_string())` → `stored.token`
- **Telegram:** `state.token_for(user_id as i64)`
- **IRC:** `require_token(bot, incoming)`
- **Slack:** `authed(data, user, |t| async move { ... })`
- **Matrix:** `stored.token` from token-lookup match
- **Fediverse:** same pattern as Discord

### Adding a store method (bot repo)

If your command needs persistent per-user state, add methods to `crates/archivist-core/src/store.rs`:

```rust
impl TokenStore {
    pub async fn set_your_state(&self, discord_id: &str, value: &str) -> Result<()> {
        redis::cmd("SETEX")
            .arg(format!("archivist:yourstate:{discord_id}"))
            .arg(60 * 60 * 24 * 30)  // 30-day TTL
            .arg(value)
            .exec_async(&mut self.conn.clone())
            .await?;
        Ok(())
    }

    pub async fn get_your_state(&self, discord_id: &str) -> Result<Option<String>> {
        let raw: Option<Vec<u8>> = redis::cmd("GET")
            .arg(format!("archivist:yourstate:{discord_id}"))
            .query_async(&mut self.conn.clone())
            .await?;
        match raw {
            Some(b) => Ok(Some(String::from_utf8(b)?)),
            None => Ok(None),
        }
    }
}
```

**Redis key naming convention:** Use `archivist:{purpose}:{discord_id}` or `archivist:{purpose}:{discord_id}:{extra}` for multi-key states.

---

## Part 3: Bot-Only Features (No Server Endpoint)

When the server doesn't have an endpoint, you can still build bot-only features using Redis for persistence.

**Pattern:**
1. Add store methods for the state you need.
2. Build the command using only the store (no API call needed) + existing API methods for display data.
3. Document clearly that this is a bot-only feature.

**Examples from this session:**
- `!filters`/`!reset-filters` — filter chips stored in Redis, used to tune recs.
- `!full-favourites` — caches bookmark list in Redis, paginates locally.
- `/track` — state machine (unseen→seen→saved→archived) in Redis.
- `!cutoff`/`!wordcount`/`!year` — per-user recs tuners in Redis.
- `/download-direct` — fetch EPUB from `/cache/epub/{url_id}`, upload as Discord attachment <25MiB.
- `/opds` — deep-link to FicHub OPDS catalog + PWA offline reader.
- Filter-chip select menu — `CreateSelectMenu` attached to recs reply; `component_handler` persists via `set_filter_chips` on selection.

---

## Part 4: Blocked Features — How to Recognize Them

Before implementing, check if the server endpoint exists. If it doesn't, the feature is either:
- **Bot-only** (can use Redis + existing API methods for display).
- **Blocked** (needs server changes — document and move on).

**How to check:**

```bash
cd /home/alvaro/code/rust/fichub
# Search server.rs for the route
grep -n 'route.*/your/endpoint' src/server.rs
# Search route handlers
grep -rn 'your_endpoint' src/routes/
```

If nothing comes back, the endpoint doesn't exist. Document in your command file:

```rust
//! NOTE: This feature requires a server endpoint that doesn't exist yet.
//! The server has no `/api/your/endpoint` route. Only the bot-only
//! parts (Redis state) are implementable here.
```

**Blocked features from this session:**
- `/notify` fic-update subscription (no webhook endpoint)
- `/mood` (no mood endpoint — P8#96, blocked until FicHub ships fic-level moods)
- OPDS/PWA (no OPDS endpoint — `/opds` command added, points to existing /opds routes)
- `/download` direct upload (no upload endpoint — `/download-direct` command added, fetches from `/cache/epub/{url_id}`)

---

## Part 5: Writing the Commit Message

Use conventional commits:

```bash
git commit -m "feat: <brief description of what was added>"
```

Examples:
- `feat: add /authors command (author bibliography browse)`
- `feat: add /stats command (reading stats embed)`
- `feat: add filter chips store methods + !filters/!reset-filters commands`
- `docs: mark saved-search alerts wired in bot`

If you also updated docs/ROADMAP:
```bash
git commit -m "feat: <code changes>

docs: <doc changes>"
```

Or as two separate commits (preferred for clarity).

---

## Part 6: Checklist for Every New Command

- [ ] Server endpoint exists (or feature is bot-only by design)
- [ ] API method added to `api.rs` (both repos if applicable)
- [ ] Response model added to `model.rs` (if multi-use)
- [ ] Command file created in `commands/`
- [ ] Command has `#[poise::command(...)]` with `slash_command`, `prefix_command`
- [ ] `ephemeral` set for personal actions
- [ ] `require_token` or `require_level` used for auth
- [ ] `touch()` called after successful auth
- [ ] JSON fields extracted with correct names (check server handler)
- [ ] `pub mod your_command` added to `commands/mod.rs`
- [ ] `commands::your_command::your_command()` added to `main.rs`
- [ ] `command_list()` updated (if you want it in `/commands`)
- [ ] `require_level_for` updated (if level-gated)
- [ ] `cargo check` passes (ignore lint tool false positives)
- [ ] Committed with conventional commit message
- [ ] `docs/code-along-new-command.md` updated with implemented items

---

*End of tutorial.*

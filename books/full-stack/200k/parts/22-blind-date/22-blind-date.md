# Part 22 — Blind Date with a Fic

"What if I picked a fic to read just by its summary, without knowing the title or fandom?" That's the idea behind *Blind Date with a Fic* — a discovery game where you judge a fic by its tropes, word count, and description, and only reveal the title (and fandom!) when you're ready. This part builds the two backend endpoints that power it, plus the frontend page where you actually play.

---

## 22.1 Backend: `GET /api/blind-date` — `blind_date_handler`

Open `src/routes/blind.rs`. The handler for the discovery endpoint is near the top:

```rust
// src/routes/blind.rs (lines 51-134, excerpt)
pub async fn blind_date_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BlindDateParams>,
) -> Result<Json<Value>, AppError> {
    let exclude: Vec<String> = params
        .exclude
        .as_deref()
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    // 1. Pick a random eligible fic.
    let row: Option<(String, String, i64, i32, String)> = sqlx::query_as(
        r#"
        SELECT id, description, words, chapters, status
        FROM fic_info
        WHERE btrim(description) <> ''
          AND NOT EXISTS (SELECT 1 FROM fic_blacklist b WHERE b.url_id = fic_info.id)
          AND ($1::text[] IS NULL OR NOT (id = ANY ($1::text[]))
        ORDER BY random()
        LIMIT 1
        "#,
    )
    .bind(if exclude.is_empty() { None } else { Some(&exclude) })
    .fetch_optional(&state.db)
    .await?;
```

### What it returns

The full response JSON looks like this:

```json
{
  "err": 0,
  "fic": {
    "url_id": "abc123",
    "description": "A young hero discovers a hidden world...",
    "words": 87430,
    "chapters": 24,
    "status": "complete",
    "tropes": ["Angst", "Enemies to Lovers", "Found Family"],
    "reveal": {
      "nonce": "a1b2c3d4e5f6a7b8",
      "sig": "f3a9c7b2..."
    }
  }
}
```

Notice what's **missing**: no `title`, no `author`, no `fandom`, no `source`. The frontend only sees the spoiler-free description, the stats (word count, chapter count, status), and the top 3 "core tropes" — character and freeform tags ranked by community votes.

### The `BlindDateParams` struct

```rust
// src/routes/blind.rs (lines 21-33)
#[derive(Debug, Default, Deserialize)]
pub struct BlindDateParams {
    /// Comma-separated list of `url_id`s to skip (so the UI can avoid
    /// showing the same fic twice in a row).
    pub exclude: Option<String>,
}
```

The `exclude` parameter lets the frontend say "I've already seen these fics — don't show them again." The client builds a comma-separated list of `url_id`s and passes it as `?exclude=id1,id2,id3`.

### Picking the tropes: `top_tropes`

The handler fetches character (tag type 2) and freeform (tag type 4) tags, then picks the top 3 by score:

```rust
// src/routes/blind.rs (lines 21-42)
pub fn top_tropes(tags: Vec<(String, i32)>, n: usize) -> Vec<String> {
    let mut sorted = tags;
    // Higher score first; ties → lexicographic name so the result is stable.
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted.into_iter().take(n).map(|(name, _)| name).collect()
}
```

This pure function is unit-tested — you can see tests right below it in the same file that verify the ranking order, tie-breaking, and the cap at `n`.

### The random pick and the nonce

Unlike search (which uses relevance ranking), blind date uses Postgres' `ORDER BY random()` for a truly random fic. The `exclude` list is bound as a typed array (`$1::text[]`), so there's no SQL injection risk.

Before returning, the handler generates a random 128-bit nonce and signs it:

```rust
// src/routes/blind.rs (lines 114-133)
    let nonce = random_hex();

    Ok(Json(json!({
        "err": 0,
        "fic": {
            "url_id": url_id,
            "description": description,
            "words": words,
            "chapters": chapters,
            "status": status,
            "tropes": tropes,
            "reveal": {
                "nonce": nonce,
                "sig": reveal_signature(&url_id, &nonce),
            },
        },
    })))
```

The `reveal` object travels alongside the spoiler-free data. The frontend uses these values when the user clicks "Reveal" to ask for the hidden title and fandom.

### The `random_hex` function

```rust
// src/routes/blind.rs (lines 256-261)
fn random_hex() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
```

It uses `OsRng` (the OS's cryptographically secure random source) to fill 16 bytes, then hex-encodes them. The `rand` crate is already a dependency (used by the scraper), so no new crates are needed.

### HMAC without the `hmac` crate

The signature function implements HMAC-SHA256 from scratch using the project's existing `sha2` dependency:

```rust
// src/routes/blind.rs (lines 204-236)
fn reveal_signature(url_id: &str, nonce: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    hmac_sha256_hex(&secret, &format!("{url_id}:{nonce}"))
}

fn hmac_sha256_hex(key: &str, msg: &str) -> String {
    use sha2::{Digest, Sha256};
    let block_size = 64usize;
    let mut k = key.as_bytes().to_vec();
    if k.len() > block_size {
        k = Sha256::digest(&k).to_vec();
    }
    k.resize(block_size, 0);
    let mut ipad = vec![0x36u8; block_size];
    let mut opad = vec![0x5cu8; block_size];
    for i in 0..block_size {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(msg.as_bytes());
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(inner_hash);
    hex::encode(outer.finalize())
}
```

The signature is `HMAC-SHA256(secret, url_id || ":" || nonce)`, hex-encoded to 64 characters. The secret is the app's `JWT_SECRET` environment variable — the same one used for JWT tokens, so no new config is needed. If `JWT_SECRET` isn't set (like in development), it falls back to `"fichub-dev-secret"`.

> **Kid-friendly explanation**: Think of the signature like a wax seal on an envelope. The server writes `url_id:nonce`, signs it with its secret key (wax seal), and hands the sealed envelope to the client. The client can't forge the seal — it doesn't know the secret. When the client wants to "open" the envelope (reveal the fic), it presents the sealed `nonce` and `sig` back to the server, which checks: "Yep, I made that seal." Only then does it show the title.

The code includes a test that verifies the HMAC against the official RFC 4231 test vector (`"Hi There"` with a 0x0b key should produce `b0344c61...`), so you know the implementation is correct.

### Constant-time comparison

When the reveal endpoint checks the signature, it uses a careful comparison that doesn't leak timing info:

```rust
// src/routes/blind.rs (lines 242-253)
fn signatures_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
```

Instead of returning early on the first mismatch (which would leak how many characters matched), it XORs all bytes together and checks if the overall result is zero.

### Route registration

In `src/server.rs`, the two blind-date routes are registered together:

```rust
// src/server.rs (lines 369-371)
        // Blind Date with a Fic — random discovery with title/fandom hidden
        .route("/api/blind-date", get(crate::routes::blind::blind_date_handler))
        .route("/api/blind-date/reveal", get(crate::routes::blind::blind_date_reveal_handler))
```

And in `src/routes/mod.rs`:

```rust
// src/routes/mod.rs (line 8)
pub mod blind;
```

### Breakdown

- **Eligibility**: Only fics with a non-empty description and not on the blacklist. Status is intentionally not filtered — both complete and ongoing fics can be discovered.
- **No repeats**: The `exclude` parameter lets the frontend avoid showing the same fic twice in one session.
- **Spoiler-free discovery**: The payload contains description, stats, and tropes — but never the title, author, or fandom.
- **Signed reveal**: A per-fic nonce + HMAC signature ensures the frontend can only reveal the title after seeing the summary first.
- **No auth required**: Blind Date is a public endpoint — you don't need to be logged in to play.

---

## 22.2 Backend: `GET /api/blind-date/reveal` — `blind_date_reveal_handler`

Now for the reveal endpoint, which is the whole point of the "blind date" mechanic. Open `src/routes/blind.rs`:

```rust
// src/routes/blind.rs (lines 150-196)
pub async fn blind_date_reveal_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RevealParams>,
) -> Result<Json<Value>, AppError> {
    let expected = reveal_signature(&params.url_id, &params.nonce);
    if !signatures_equal(&expected, &params.sig) {
        return Err(AppError::BadRequest("invalid reveal signature".to_string()));
    }

    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, title, author, source FROM fic_info WHERE id = $1",
    )
    .bind(&params.url_id)
    .fetch_optional(&state.db)
    .await?;

    let Some((id, title, author, source)) = row else {
        return Err(AppError::NotFound("fic not found".into()));
    };

    // Fandom = the single strongest fandom tag (tag_type_id = 1) by score.
    let fandom: Option<String> = sqlx::query_scalar(
        r#"
        SELECT t.name
        FROM fic_tags ft
        JOIN tags t ON t.id = ft.tag_id
        WHERE ft.url_id = $1 AND t.tag_type_id = 1
        ORDER BY ft.score DESC, t.name ASC
        LIMIT 1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "fic": {
            "url_id": id,
            "title": title,
            "author": author,
            "source": source,
            "fandom": fandom,
        },
    })))
}
```

### How the reveal flow works

1. The discovery endpoint returns a fic with a `reveal` object: `{ nonce, sig }`.
2. The frontend shows the summary to the user.
3. When the user clicks "Reveal," the frontend calls `/api/blind-date/reveal?url_id=...&nonce=...&sig=...`.
4. The server re-derives the signature from the `url_id` and `nonce`, compares it to what the client sent, and **only if they match** returns the full metadata: title, author, source, and fandom.

The `sig` was generated server-side in step 1 and handed to the client. Since the client can't forge it (no access to `JWT_SECRET`), the server knows the client must have received the discovery payload first. It's a clever way to enforce "summary before title" without tracking state.

### The `RevealParams` struct

```rust
// src/routes/blind.rs (lines 137-142)
#[derive(Debug, Deserialize)]
pub struct RevealParams {
    pub url_id: String,
    pub nonce: String,
    pub sig: String,
}
```

All three are passed as query parameters. You could also POST them in a body, but GET with query params is fine here since the reveal is idempotent.

### Fandom lookup

The reveal endpoint does a second query to find the fandom. Tags are categorized by `tag_type_id`:

- `1` = fandom
- `2` = character
- `4` = freeform (ships, tropes, warnings, etc.)

It picks the highest-scored fandom tag (by `ft.score DESC, t.name ASC` for deterministic tie-breaking):

```sql
SELECT t.name
FROM fic_tags ft
JOIN tags t ON t.id = ft.tag_id
WHERE ft.url_id = $1 AND t.tag_type_id = 1
ORDER BY ft.score DESC, t.name ASC
LIMIT 1
```

If a fic has no fandom tags, `fandom` comes back as `null` — spoiler-light, just like the discovery payload.

### The reveal response

```json
{
  "err": 0,
  "fic": {
    "url_id": "abc123",
    "title": "The Last Ember",
    "author": "StarlightWriter",
    "source": "archiveofourown.org",
    "fandom": "Harry Potter - J.K. Rowling"
  }
}
```

Now the full reveal is shown. The frontend links the `url_id` to the work page so the user can start reading.

### Breakdown

- **Signature verification**: The server re-derives the expected signature and uses constant-time comparison. A fake or tampered `sig` gets a 400.
- **No auth required**: Like the discovery endpoint, reveal is public.
- **Single fandom**: The strongest fandom tag wins. Multi-fandom crossovers show the primary fandom.
- **Title/fandom separation**: Discovery never sends title/fandom; reveal never sends description/tropes. Each response is minimal and purpose-built.

---

## 22.3 Frontend: Blind Date page — guess, reveal, share

The frontend page lives at `frontend/src/routes/blind-date/+page.svelte`:

```svelte
<!-- frontend/src/routes/blind-date/+page.svelte (lines 1-36) -->
<script lang="ts">
  import BlindDateCard from '$lib/components/BlindDateCard.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Blind Date with a Fic</h1>
      <p class="archive-muted archive-sub">
        The title and fandom of each fic stay hidden until you hit Reveal —
        judge purely by summary, word count, and tropes. "Another!" draws a new fic and never
        repeats one you've already seen this session.
      </p>
    </header>

    <fieldset class="archive-fieldset">
      <legend>Today's Match</legend>
      <BlindDateCard />
    </fieldset>
  </div>
</main>
{:else}
<div class="blind-date-page">
  <BlindDateCard />
  <p class="muted hint">
    The title and fandom of each fic stay hidden until you hit <strong>Reveal</strong> —
    judge purely by summary, word count and tropes. "Another!" draws a new fic and never
    repeats one you've already seen this session.
  </p>
</div>
{/if}
```

### What the page does

The page is a thin shell — it just renders the `BlindDateCard` component. All the real logic (fetching, guessing, revealing, tracking seen fics) lives inside `BlindDateCard`. The page has two modes:

- **Archive mode** (AO3-style layout with serif fonts and a vintage border): for users who've enabled the archive UI preference.
- **Modern mode**: the default SvelteKit look with a centered card and a hint paragraph.

The `getPref('uiMode')` call reads the user's UI preference from localStorage — the same `getPref` helper used across the frontend, just like in Part 8 where the auth store read cached user data.

### The API client pattern

The `BlindDateCard` component uses the `request()` helper from `frontend/src/lib/api/social.ts` to call the backend. That helper is the central fetch wrapper used by every API call in the app — you saw it in Parts 8, 11, and 13. Here's how the blind-date API calls would look written against it:

```typescript
// frontend/src/lib/api/social.ts — conceptual additions (not yet wired in)
export interface BlindDateFic {
  url_id: string;
  description: string;
  words: number;
  chapters: number;
  status: string;
  tropes: string[];
  reveal: { nonce: string; sig: string };
}

export async function getBlindDate(exclude: string[] = []): Promise<{ err: number; fic: BlindDateFic | null }> {
  const qs = exclude.length ? `?exclude=${exclude.join(',')}` : '';
  return request(`/blind-date${qs}`);
}

export async function revealBlindDate(
  url_id: string, nonce: string, sig: string
): Promise<{ err: number; fic: { title: string; author: string; source: string; fandom: string | null } }> {
  return request(`/blind-date/reveal?url_id=${encodeURIComponent(url_id)}&nonce=${encodeURIComponent(nonce)}&sig=${encodeURIComponent(sig)}`);
}
```

The key things to notice:
- **Same `request()` helper**: Handles auth headers, 401 redirects, and JSON parsing — no need to reinvent it.
- **`exclude` as a query string**: The frontend builds a comma-separated list of `url_id`s it has already shown.
- **`encodeURIComponent`**: The `url_id`, `nonce`, and `sig` are hex strings, but `encodeURIComponent` is good practice.

### How the BlindDateCard component works

The actual `BlindDateCard.svelte` isn't in the repo's components directory yet — it's imported by the page but doesn't exist. That's the exercise for this part! Here's what it does, based on the page's hint text and the backend contract:

```svelte
<!-- frontend/src/lib/components/BlindDateCard.svelte (conceptual — your task to build) -->
<script lang="ts">
  import { getBlindDate, revealBlindDate } from '$lib/api/social';

  let fic = $state<BlindDateFic | null>(null);
  let revealed = $state(false);
  let revealedInfo = $state<{ title: string; author: string; source: string; fandom: string | null } | null>(null);
  let loading = $state(true);
  let error = $state('');
  let seen = $state<string[]>([]);

  async function drawFic() {
    loading = true;
    error = '';
    revealed = false;
    revealedInfo = null;
    try {
      const res = await getBlindDate(seen);
      if (res.err === 0 && res.fic) {
        fic = res.fic;
      } else {
        fic = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Something went wrong';
    } finally {
      loading = false;
    }
  }

  async function reveal() {
    if (!fic) return;
    revealed = true;
    seen = [...new Set([...seen, fic.url_id])];
    try {
      const res = await revealBlindDate(fic.url_id, fic.reveal.nonce, fic.reveal.sig);
      if (res.err === 0 && res.fic) {
        revealedInfo = res.fic;
      } else {
        error = 'Reveal failed';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Reveal failed';
    }
  }

  drawFic();
</script>

<div class="blind-date-card">
  {#if loading}
    <p class="loading">Drawing a mystery fic…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if !fic}
    <p class="empty">No eligible fics found. Try again later!</p>
  {:else}
    <div class="fic-summary">
      <p class="description">{fic.description}</p>
      <div class="meta">
        {fic.words.toLocaleString()} words · {fic.chapters} chapters · {fic.status}
      </div>
      {#if fic.tropes.length > 0}
        <div class="tropes">
          {#each fic.tropes as trope}
            <span class="trope-tag">{trope}</span>
          {/each}
        </div>
      {/if}
      {#if revealed && revealedInfo}
        <div class="revealed">
          <h3>{revealedInfo.title}</h3>
          <p class="byline">by {revealedInfo.author} · {revealedInfo.fandom ?? 'Unknown fandom'}</p>
          <a href={`/works/${fic.url_id}`} class="read-link">Read this fic →</a>
        </div>
      {:else}
        <button onclick={reveal} class="reveal-btn">Reveal</button>
      {/if}
      <button onclick={drawFic} class="another-btn">Another!</button>
    </div>
  {/if}
</div>
```

### The game loop

Here's how the user experience flows:

1. **"Draw"** — The page loads (or the user clicks "Another!"), and the `request()` helper calls `GET /api/blind-date`. The response has description, stats, and tropes — but no title.
2. **"Guess"** — The user reads the summary and thinks, "I bet this is from [fandom]!"
3. **"Reveal"** — The user clicks "Reveal." The component calls `GET /api/blind-date/reveal?url_id=...&nonce=...&sig=...`. The server verifies the signature, then returns the title, author, source, and fandom.
4. **"Another!"** — The `url_id` of the revealed fic is added to the `seen` array. The next `getBlindDate(seen)` call passes `?exclude=...` so Postgres skips it.

### The `seen` array and `exclude` param

The `seen` array lives in component state (not localStorage — it resets when you reload the page, keeping the game fresh each session). Each time you click "Another!", the current `url_id` is appended:

```typescript
seen = [...new Set([...seen, fic.url_id])];
```

Using `new Set` deduplicates. The list is then sent back to the server:

```
GET /api/blind-date?exclude=abc123,def456,ghi789
```

The backend parses this into a `Vec<String>` and wraps it in a Postgres array for the `NOT (id = ANY($1))` clause. When the list is empty, it binds `None` (PostgreSQL `NULL`), which the SQL handles with the `$1::text[] IS NULL OR ...` guard.

> **Kid-friendly tip**: The `exclude` list is like a "do not repeat" pile of cards. Each time you get a new card, you put the old one on the pile so the deck knows to skip it.

### Share feature

The hint text mentions "share" — the actual share mechanism would use the browser's native share API or a fallback. Since the title was hidden, sharing the *guessed* fandom with friends ("I thought this was Harry Potter but it was actually Marvel!") is the fun part.

### Styling

The page supports two modes. In archive mode, it uses the AO3-style CSS variables (`--archive-max-width`, `--archive-link`, `--archive-border`, etc.) with Georgia serif fonts and a red accent color. In modern mode, it uses a simple centered card with a muted hint paragraph.

### Breakdown

- **Two UI modes**: Archive (AO3-style) and modern. Switched via `getPref('uiMode')`.
- **`BlindDateCard` component**: Contains all the game logic — fetch, guess, reveal, track seen fics.
- **`request()` helper reuse**: All API calls go through the centralized `request()` from `social.ts`.
- **`exclude` param wiring**: The frontend builds a comma-separated list of seen `url_id`s and passes it as a query param.
- **Signature round-trip**: The `nonce` and `sig` from the discovery response are passed straight through to the reveal call — the client never sees the secret.
- **State management**: Svelte 5 `$state` for reactive updates, `$derived` for computed values.

---

## 22.4 Try It Yourself: play a blind date

### Step 1: Fetch a blind date fic

```bash
curl -s http://localhost:8000/api/blind-date | python3 -m json.tool
```

**Expected**: A JSON object with `err: 0` and a `fic` object containing `url_id`, `description`, `words`, `chapters`, `status`, `tropes`, and `reveal` — but **no title or fandom**.

### Step 2: Try excluding a fic

```bash
curl -s "http://localhost:8000/api/blind-date?exclude=YOUR_FIC_URL_ID" | python3 -m json.tool
```

Replace `YOUR_FIC_URL_ID` with the `url_id` from Step 1. **Expected**: A *different* fic.

### Step 3: Reveal the title

```bash
curl -s "http://localhost:8000/api/blind-date/reveal?url_id=YOUR_FIC_URL_ID&nonce=NONCE&sig=SIG" | python3 -m json.tool
```

Replace `YOUR_FIC_URL_ID`, `NONCE`, and `SIG` with the values from Step 1's `reveal` object. **Expected**: A JSON object with the `title`, `author`, `source`, and `fandom` for that fic.

### Step 4: Try a bad signature

```bash
curl -s "http://localhost:8000/api/blind-date/reveal?url_id=YOUR_FIC_URL_ID&nonce=NONCE&sig=deadbeef" | python3 -m json.tool
```

**Expected**: `400 Bad Request` with `"invalid reveal signature"`. The server rejected the forged signature.

### Step 5: Build the BlindDateCard component

Create `frontend/src/lib/components/BlindDateCard.svelte` based on the conceptual scaffold above. At minimum, it should:
- Call `getBlindDate()` on mount (use the `request()` helper pattern)
- Display the description, stats, and tropes
- Have a "Reveal" button that calls `revealBlindDate()`
- Show the title and fandom after reveal
- Track seen fics and pass `exclude` to avoid repeats
- Handle loading and error states

### Troubleshooting

**"No eligible fics found"**: The `fic_info` table has no rows with a non-empty description, or all fics are blacklisted. Check:
```sql
SELECT COUNT(*) FROM fic_info WHERE btrim(description) <> '';
SELECT COUNT(*) FROM fic_blacklist;
```

**"invalid reveal signature"**: The `nonce` or `sig` was modified, or they came from a different `url_id`. Make sure you're using the exact `nonce` and `sig` from the discovery response for the same `url_id`.

**"fic not found"**: The `url_id` doesn't exist in `fic_info`. This can happen if the fic was deleted between discovery and reveal.

**Frontend "Cannot find module" errors**: Make sure `BlindDateCard.svelte` is created in `frontend/src/lib/components/` before importing it in `+page.svelte`.

**Tests failing**: Run the unit tests for the pure functions:
```bash
cd /home/alvaro/code/rust/fichub && cargo test --lib blind
```
This runs `top_tropes`, `signatures_equal`, `reveal_signature`, and `hmac_sha256_hex` tests — all the logic that doesn't need a database.

---

## 22.5 What you have now

- You understand the blind-date discovery endpoint: `GET /api/blind-date` returns a random eligible fic with title and fandom hidden, only showing description, stats, and top 3 tropes.
- You understand the `exclude` parameter: a comma-separated list of `url_id`s to skip, bound safely as a Postgres array.
- You understand the `top_tropes` pure function: sorts character + freeform tags by score with deterministic tie-breaking, capped at 3.
- You understand the HMAC signature scheme: the discovery payload includes a `nonce` and `sig` (HMAC-SHA256 of `url_id:nonce` with the `JWT_SECRET`), implemented from scratch with `sha2`.
- You understand the reveal endpoint: verifies the signature before returning title, author, source, and fandom. Constant-time comparison prevents timing attacks.
- You understand the two-query pattern: discovery hides the title; reveal fetches it.
- You understand the frontend page: two UI modes (archive + modern), thin shell rendering `BlindDateCard`, with the game loop (draw → guess → reveal → another).
- You understand the API client pattern: reusing the centralized `request()` helper, with `exclude` as a query string and `url_id`/`nonce`/`sig` for the reveal.
- You tested both endpoints with curl and verified signature verification rejects forged signatures.

Next: Part 23 — Fandoms and Trending. You will build the trending fics endpoint and fandom landing pages.

---

*End of Part 22. On to [Part 23 — Fandoms and Trending](./23-fandoms-trending/23-fandoms-trending.md).*

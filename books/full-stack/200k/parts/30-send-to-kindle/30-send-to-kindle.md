# Part 30 — Send to Kindle

Most fanfic readers have a Kindle. But downloading an EPUB, emailing it to yourself, and opening your Kindle address book is a pain. Send to Kindle fixes that: one click on the fic page, and FicHub generates an EPUB and emails it straight to the user's `name@kindle.com` address. This part builds the whole pipeline — the backend email route, the SMTP mailer service, and the frontend button that triggers it.

---

## 30.1 Backend: `POST /api/send-to-kindle` — `send_to_kindle_handler`

Open `src/routes/kindle.rs` and `src/server.rs`. The route is registered at line 764 of `server.rs`:

```rust
// src/server.rs (line 763-764)
// Send-to-Kindle route
.route("/api/send-to-kindle", axum::routing::post(crate::routes::kindle::send_to_kindle_handler))
```

And the handler lives in its own module file:

```rust
// src/routes/kindle.rs (lines 1-27)
use axum::{
    extract::State,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Request body for POST /api/send-to-kindle.
/// Exactly one of `url` (scrape + export) or `url_id` (export from DB) is used.
#[derive(Debug, Deserialize)]
pub struct SendToKindleReq {
    pub url: Option<String>,
    pub url_id: Option<String>,
}
```

### The handler

```rust
// src/routes/kindle.rs (lines 208-272)
pub async fn send_to_kindle_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<SendToKindleReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Exactly one of url / url_id must be provided
    let resolved_url_id = match (&req.url, &req.url_id) {
        (Some(url), None) => {
            if !url.starts_with("http") {
                return Err(AppError::BadRequest("url must be an http(s) fic URL".to_string()));
            }
            let scraper = state
                .scraper_registry
                .find_specific_or_fff(url)
                .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {url}")))?;
            scraper
                .lookup(&state.http_client, url)
                .await
                .map_err(|e| AppError::ScrapeError(e.to_string()))?
                .url_id
        }
        (None, Some(url_id)) => {
            if url_id.is_empty() {
                return Err(AppError::BadRequest("url_id must not be empty".to_string()));
            }
            url_id.clone()
        }
        _ => {
            return Err(AppError::BadRequest("provide exactly one of url or url_id".to_string()));
        }
    };

    // Load the user's email + kindle_email from the DB
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT COALESCE(email, ''), kindle_email FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (account_email, kindle_email) = row
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let to = kindle_email
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(account_email.clone());

    if to.trim().is_empty() {
        return Err(AppError::BadRequest("no email on file — add an email or a Kindle address to your account".to_string()));
    }

    // Graceful invalid-email handling: reject before any EPUB work so the
    // user gets a clear 400 instead of a mid-send failure.
    if let Err(e) = crate::services::mailer::validate_email(&to) {
        return Err(AppError::BadRequest(format!("invalid email: {e}")));
    }

    let title = format!("Fic {resolved_url_id}");
    let resp = build_and_send(&state, &to, &resolved_url_id, &title).await?;
    Ok(Json(resp))
}
```

### Breakdown

**Auth**: The handler uses the `AuthUser` extractor (see Part 8). If the user isn't logged in, `user_id` is `None` and we return 401.

**Two ways to identify a fic**: The request body accepts either:
- `url` — a full source URL (e.g. an AO3 link). The handler scrapes it fresh to resolve the `url_id`.
- `url_id` — the short FicHub ID already known to the frontend (e.g. from the fic page). This skips the scrape step.

Exactly one must be provided; passing both or neither returns a 400.

**Email resolution**: The handler fetches the user's row from the `users` table:

```sql
SELECT COALESCE(email, ''), kindle_email FROM users WHERE id = $1
```

It prefers `kindle_email` (the user's `@kindle.com` address) but falls back to their account `email` if they haven't set one. The address is validated *before* any EPUB work happens — this is a "fail fast" design so users get a clear `400 invalid email` instead of a cryptic SMTP error 10 seconds later.

**The real work**: After resolving the `url_id` and email, the handler calls `build_and_send`, which we'll break down next.

---

## 30.2 Backend: `src/routes/kindle.rs` — Lettre SMTP, format selection

The `build_and_send` function is where the magic happens. It lives right above the handler in the same file:

```rust
// src/routes/kindle.rs (lines 21-206)
async fn build_and_send(
    state: &AppState,
    to: &str,
    url_id: &str,
    _title: &str,
) -> Result<Value, AppError> {
    // Resolve chapters: prefer a cached fic_info entry, else scrape live.
    let (meta, chapters) = if let Some(fic) =
        crate::db::queries::get_fic_info(&state.db, url_id).await?
    {
        let meta = crate::scrape::FicMetadata {
            url_id: fic.id.clone(),
            title: fic.title.clone(),
            author: fic.author.clone(),
            chapters: fic.chapters,
            words: fic.words,
            desc: fic.description,
            published: fic.fic_created.timestamp_millis(),
            updated: fic.fic_updated.timestamp_millis(),
            status: fic.status.clone(),
            source: fic.source.clone(),
            // ... (many fields elided for brevity)
        };
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(&meta.source)
            .ok_or_else(|| AppError::BadRequest(format!("no scraper for {}", meta.source)))?;
        let chapters = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => {
                state.heal.record_failure(...).await;
                return Err(AppError::ScrapeError(format!("failed to fetch chapters: {e}")));
            }
        };
        (meta, chapters)
    } else {
        // Not in DB: scrape fresh
        let scraper = state.scraper_registry.find_specific_or_fff(url_id)
            .ok_or_else(|| AppError::NotFound(format!("fic {url_id} not found")))?;
        let meta = scraper.lookup(&state.http_client, url_id).await...;
        let chapters = scraper.fetch_chapters(&state.http_client, &meta).await...;
        (meta, chapters)
    };

    // Generate the EPUB
    let (epub_path, epub_hash) =
        crate::export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
            .await
            .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Cache it (same flow as the export handler)
    let cache_dest = crate::cache::disk::cache_path(
        &state.config.cache_dir,
        &crate::cache::EType::Epub,
        &meta.url_id,
        &epub_hash,
    );
    if let Err(e) = crate::cache::disk::move_to_cache(&epub_path, &cache_dest) {
        tracing::warn!("send-to-kindle: failed to cache EPUB: {e}");
    }

    // Log the export (reuses the export_log table)
    let version = state.config.export_version;
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    crate::db::queries::insert_export_log(
        &state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash,
    ).await?;

    // Sanitized attachment name: <slug>.epub
    let slug = meta.title.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");
    let attachment_name = if slug.is_empty() {
        format!("{}.epub", meta.url_id)
    } else {
        format!("{}.epub", slug)
    };

    let subject = format!("{} by {}", meta.title, meta.author);
    let body = format!(
        "Your fic is attached.\n\nTitle: {}\nAuthor: {}\nURL: {}\n\nSent from FicHub.",
        meta.title, meta.author, meta.source
    );

    let mail = crate::services::mailer::KindleMail {
        to: to.to_string(),
        subject,
        body,
        attachment_path: epub_path.clone(),
        attachment_name,
    };

    let mailer = state.mailer.clone_box();
    tokio::task::spawn_blocking(move || mailer.send_kindle(&mail))
        .await
        .map_err(|e| AppError::Internal(format!("mailer task failed: {e}")))?
        .map_err(|e| AppError::Internal(format!("failed to send email: {e}")))?;

    let _ = std::fs::remove_file(&epub_path);

    Ok(json!({
        "err": 0,
        "url_id": meta.url_id,
        "to": to,
        "msg": "EPUB sent to your Kindle",
    }))
}
```

### Breakdown

**Caching before sending**: The `build_and_send` function checks whether the fic is already in the database (`get_fic_info`). If it is, it reuses the cached metadata and only fetches the latest chapter list from the source site. If the fic isn't in the DB, it scrapes fresh. This mirrors the flow used by the regular EPUB export handler (see Part 7), so the Kindle feature reuses the same well-tested path rather than inventing a new one.

**Format selection = EPUB**: The handler generates only one format for Kindle delivery — EPUB. (The regular export pipeline, covered in Part 7, supports HTML, MOBI, PDF, AZW3, DOCX, FB2, TXT, and Markdown via the Calibre-backed conversion service. But Kindle natively supports EPUB since 2022, so we hardcode it here. The `EPub` format is selected by calling `create_epub`.)

**Caching the result**: After generating the EPUB, the file is moved into the on-disk cache (`/cache/epub/<hashed path>`) so a second send-to-kindle for the same fic is instant. The cache path is computed by `crate::cache::disk::cache_path`, the same function the export handler uses.

**Export log**: Every send-to-kindle is logged via `insert_export_log` — the same table the regular export pipeline writes to. This means:
- It shows up in analytics (see `src/db/queries.rs` line 2541, where `/api/send-to-kindle` is bucketed into the `export` endpoint group).
- The self-heal system can detect repeated failures.
- The endpoint rate-limits under the "download tier" category (see `src/routes/analytics.rs` line 138).

**Sanitized attachment names**: The EPUB filename is derived from the fic title, with non-alphanumeric characters replaced by spaces, then joined with underscores. This avoids filesystem issues with titles like `"Harry Potter & the Chamber of Secrets?!"`. If the slug comes out empty (e.g. the title is all symbols), it falls back to the `url_id`.

**Mailer dispatch**: The email is sent through `state.mailer`, which is a trait object (`Box<dyn Mailer>`). The `clone_box()` method lets us hand it to `spawn_blocking`, which is important — SMTP sends are synchronous I/O and would block the async runtime otherwise.

---

## 30.3 The SMTP mailer service (`src/services/mailer.rs`)

The email-sending logic is isolated in a dedicated service module. It uses the `lettre` crate (declared in `Cargo.toml` with `smtp-transport`, `builder`, and `tokio1` features):

```toml
# Cargo.toml (line 96-97)
# Email (Send-to-Kindle)
lettre = { version = "0.11", default-features = false, features = ["smtp-transport", "builder", "tokio1"] }
```

### Configuration

SMTP settings come from environment variables. Open `src/config.rs` (lines 472-481):

```rust
// src/config.rs (lines 472-481)
// Send-to-Kindle (SMTP) — optional; if SMTP_HOST is unset the
// /api/send-to-kindle endpoint reports SMTP not configured.
let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
let smtp_port = std::env::var("SMTP_PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(587);
let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();
let smtp_from = std::env::var("SMTP_FROM").unwrap_or_default();
```

These five variables map to the `SmtpConfig` struct:

```rust
// src/services/mailer.rs (lines 19-27)
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}
```

### Building the MIME message

The `build_kindle_message` function constructs a multipart MIME email with a text body and an EPUB attachment. It's a pure function (no network), so it's unit-testable:

```rust
// src/services/mailer.rs (lines 21-79)
pub fn build_kindle_message(
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
    attachment_path: &Path,
    attachment_name: &str,
) -> Result<Message, String> {
    let from: Mailbox = from.parse()
        .map_err(|e| format!("invalid from address '{}': {}", from, e))?;
    let to: Mailbox = to.parse()
        .map_err(|e| format!("invalid to address '{}': {}", to, e))?;

    let attachment_bytes = std::fs::read(attachment_path)
        .map_err(|e| format!("failed to read attachment {}: {}", attachment_path.display(), e))?;

    let content_type = ContentType::parse("application/epub+zip")
        .unwrap_or_else(|_| ContentType::TEXT_PLAIN);

    Message::builder()
        .from(from)
        .to(to)
        .subject(subject.to_string())
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(body.to_string()),
                )
                .singlepart(
                    Attachment::new(attachment_name.to_string())
                        .body(attachment_bytes, content_type),
                ),
        )
        .map_err(|e| format!("failed to build email: {}", e))
}
```

The message is `multipart/mixed` — a text part for the human-readable body, and an attachment part with `Content-Type: application/epub+zip` (the official MIME type for EPUBs).

### Sending over SMTP

The `send_email` function creates a `lettre::transport::smtp::SmtpTransport` and sends:

```rust
// src/services/mailer.rs (lines 81-105)
pub fn send_email(cfg: &SmtpConfig, msg: &Message) -> Result<(), String> {
    let mut builder = SmtpTransport::builder_dangerous(&cfg.host).port(cfg.port);
    if !cfg.user.is_empty() {
        builder = builder.credentials(Credentials::new(cfg.user.clone(), cfg.pass.clone()));
    }
    let transport = builder.build();
    transport
        .send(msg)
        .map(|_| ())
        .map_err(|e| format!("SMTP send failed: {}", e))
}
```

**Note on TLS**: The Cargo features deliberately omit a TLS backend (`tokio1-rustls-tls` / `tokio1-native-tls` are NOT enabled). Instead, the transport uses `builder_dangerous`, which connects in plain mode. As the comment at the top of the file says (lines 6-10):

```rust
// NOTE: with the current Cargo features (`smtp-transport`, `builder`,
// `tokio1`) no TLS backend is compiled in, so the transport is built with
// `builder_dangerous` (plain SMTP). If the deployment relay requires TLS,
// enable `tokio1-rustls-tls` (or `tokio1-native-tls`) and switch to
// `SmtpTransport::starttls_relay` / `relay` instead.
```

This means SMTP relays that require opportunistic TLS on port 587 won't work out of the box — you'd need to add the feature flag. For dev/test, a local relay like `mailhog` or `smtp4dev` on port 25 (plain SMTP) works fine.

### The Mailer trait and testability

Rather than depending on `lettre` directly in the route, the code defines a `Mailer` trait. This is a classic dependency-injection pattern — the handler calls `state.mailer.send_kindle(...)`, and tests inject a mock:

```rust
// src/services/mailer.rs (lines 134-173)
pub trait Mailer: Send + Sync {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String>;
    fn clone_box(&self) -> Box<dyn Mailer>;
}

impl Mailer for Box<dyn Mailer> {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        (**self).send_kindle(mail)
    }
    fn clone_box(&self) -> Box<dyn Mailer> {
        (**self).clone_box()
    }
}

pub struct SmtpMailer { pub cfg: SmtpConfig }

impl Mailer for SmtpMailer {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        send_kindle_email(&self.cfg, mail)
    }
    fn clone_box(&self) -> Box<dyn Mailer> { Box::new(self.clone()) }
}
```

The `MockMailer` records every send for test assertions and can simulate failures:

```rust
// src/services/mailer.rs (lines 175-227)
pub struct MockMailer {
    pub sent: std::sync::Mutex<Vec<KindleMail>>,
    fail_count: std::sync::Mutex<u32>,
}

impl Mailer for MockMailer {
    fn send_kindle(&self, mail: &KindleMail) -> Result<(), String> {
        if self.take_failure() {
            return Err("mock SMTP failure".into());
        }
        self.sent.lock().unwrap().push(mail.clone());
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Mailer> {
        Box::new(MockMailer {
            sent: self.sent.lock().unwrap().clone().into(),
            fail_count: std::sync::Mutex::new(0),
        })
    }
}
```

### Email validation

Before any EPUB is generated, the handler validates the recipient address with a lightweight `validate_email` helper (no `lettre` parser needed — just a quick sanity check that catches empty strings, missing `@`, and domains without a dot):

```rust
// src/services/mailer.rs (lines 111-132)
pub fn validate_email(address: &str) -> Result<(), String> {
    let trimmed = address.trim();
    if trimmed.is_empty() { return Err("email address is empty".into()); }
    if trimmed.len() > 254 { return Err("email address is too long".into()); }
    let mut parts = trimmed.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if local.is_empty() || domain.is_empty() {
        return Err(format!("'{address}' is not a valid email address"));
    }
    if local.contains(' ') || domain.contains(' ') {
        return Err(format!("'{address}' is not a valid email address"));
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err(format!("'{address}' is not a valid email address"));
    }
    Ok(())
}
```

### The tests

The mailer has a thorough test suite. The `build_kindle_message` tests verify the recipient, subject, and attachment are all set correctly on the built `Message`. The `validate_email` tests check edge cases like empty strings, missing `@`, and domains without dots. The `MockMailer` tests confirm that sends are recorded and that the `fail_next` mechanism works for simulating relay errors.

The tests also include a DB-gated test at the end of `kindle.rs` that verifies the `users.kindle_email` column exists and can be updated — this checks that migration 013 was applied.

---

## 30.4 Frontend: API client (`frontend/src/lib/api/kindle.ts`)

The frontend talks to the backend through a small API client:

```typescript
// frontend/src/lib/api/kindle.ts (full file)
const BASE = '/api';
const TOKEN_KEY = 'fichub_token';

interface SendToKindleResponse {
  err: number;
  url_id?: string;
  to?: string;
  msg?: string;
}

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

export async function sendToKindle(body: {
  url?: string;
  url_id?: string;
}): Promise<SendToKindleResponse> {
  const token = getToken();
  if (!token) {
    throw new Error('You must be logged in to send to Kindle.');
  }
  const res = await fetch(`${BASE}/send-to-kindle`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as SendToKindleResponse;
}
```

**The client mirrors the backend contract exactly**:
- It reads the JWT from `localStorage` under the `fichub_token` key — the same convention as `social.ts`.
- If no token is present, it throws immediately (no network request). This is the client-side defense; the server is the real gate.
- It posts to `/api/send-to-kindle` with the body `{ url_id: "..." }` (or `{ url: "..." }`).
- It parses the JSON response, which returns `{ err: 0, url_id, to, msg }`.

### The tests

Open `frontend/src/lib/api/kindle.test.ts`. The tests mock `fetch` and `localStorage`, then verify:
1. **No token → throws** with "You must be logged in" — the client refuses before hitting the network.
2. **Token present → POSTs with Bearer header** — checks that the URL is `/api/send-to-kindle`, the method is `POST`, the Authorization header is `Bearer kindle-jwt`, and the body contains the correct `url`.
3. **`url_id` body** — confirms the body is `{ url_id: 'known-id' }` when no `url` is given.
4. **Error on non-OK** — a 401 response throws an `API error 401` message.

These tests are also echoed in the smoke-test file `frontend/src/lib/test/new-features.smoke.test.ts` (lines 9-58), which checks that the client sends the auth header when logged in and refuses to make a request without a token.

---

## 30.5 Frontend: Send to Kindle button on the work page

Open `frontend/src/routes/works/[urlId]/+page.svelte`. The button lives in the actions row of the fic page, right next to the download buttons:

```svelte
<!-- frontend/src/routes/works/[urlId]/+page.svelte (lines 761-779) -->
{#if auth.isLoggedIn}
<button
  class="btn btn-secondary kindle-btn"
  onclick={handleSendToKindle}
  disabled={kindleSending}
  title={t('fic.kindleTitle')}
>
  {#if kindleSending}
  <span class="spinner"></span> {t('fic.sending')}
  {:else if kindleSent}
  {t('fid.sentToKindle')}
  {:else}
  {t('fic.sendToKindle')}
  {/if}
</button>
{#if kindleError}
<p class="error-text kindle-error">{kindleError}</p>
{/if}
{/if}
```

The button is hidden entirely for anonymous users (`{#if auth.isLoggedIn}`). It shows three states: "Send to Kindle" (default), "Sending…" (with a spinner), and "Sent to Kindle" (success confirmation). Errors render in an `<p class="kindle-error">`.

The `handleSendToKindle` function is declared alongside the other state variables:

```typescript
// frontend/src/routes/works/[urlId]/+page.svelte (lines 54-57)
let kindleSending = $state(false);
let kindleSent = $state(false);
let kindleError = $state('');

// lines 441-458
async function handleSendToKindle() {
  if (!auth.isLoggedIn || kindleSending) return;
  kindleSending = true;
  kindleError = '';
  kindleSent = false;
  try {
    const res = await sendToKindle({ url_id: urlId });
    if (res.err !== 0) {
      kindleError = res.msg || 'Could not send to Kindle.';
    } else {
      kindleSent = true;
    }
  } catch (e) {
    kindleError = e instanceof Error ? e.message : 'Send to Kindle failed';
  } finally {
    kindleSending = false;
  }
}
```

It passes `url_id: urlId` (the fic's short ID from the URL params), which means the frontend never sends the full source URL — the `url_id` is already resolved from the page load. The `url` path is available in the API for callers that only have the original URL.

### Styling

The button uses a `.kindle-btn` CSS class (line 1205):

```css
.kindle-btn { text-decoration: none; }
.kindle-error { font-size: 0.82rem; max-width: 260px; }
```

And the i18n strings come from the locale dictionaries (see `frontend/src/lib/i18n/dictionaries/en.ts` lines 326-329):

```typescript
'fic.sendToKindle': 'Send to Kindle',
'fic.sentToKindle': 'Sent to Kindle',
'fic.sending': 'Sending…',
'fic.kindleTitle': 'Email this fic as an EPUB to your Kindle address',
```

---

## 30.6 Try It Yourself: send a fic to your Kindle

### Step 1: Configure SMTP (if you don't have a relay)

For local testing, start a dummy SMTP server:

```bash
# Install and run MailHog (catches emails without delivering)
go install github.com/mailhog/MailHog@latest 2>/dev/null || docker run -d -p 1025:1025 -p 8025:8025 mailhog/mailhog
```

Then set the environment variables before launching FicHub:

```bash
export SMTP_HOST=127.0.0.1
export SMTP_PORT=1025
export SMTP_USER=""
export SMTP_PASS=""
export SMTP_FROM="fichub@localhost"
```

### Step 2: Set your Kindle email address

Register or log in to FicHub, then set your `kindle_email` in your account settings. (Kindle addresses look like `yourname@kindle.com` — you can find yours at [Amazon's "Manage Your Content and Devices" page](https://www.amazon.com/hz/myx/#/tab1).) If you don't have a Kindle, use any email address — the SMTP relay will still send the EPUB as an attachment.

### Step 3: Send a fic

Once the dev server is running at `http://localhost:3000`, open any work page (`/works/<url_id>`), scroll to the download buttons, and click **"Send to Kindle"**:

![Send to Kindle button on the fic page](https://dummyimage.com/300x40/cccccc/000000&text=Send+to+Kindle)

Or test it directly with curl (using a real token from your login):

```bash
curl -s -X POST http://localhost:3000/api/send-to-kindle \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"url_id": "a1b2c3d4"}' | python3 -m json.tool
```

**Expected response**:

```json
{
  "err": 0,
  "url_id": "a1b2c3d4",
  "to": "yourname@kindle.com",
  "msg": "EPUB sent to your Kindle"
}
```

### Step 4: Verify the email

If you're using MailHog, open `http://localhost:8025` in your browser. You should see an email with the subject `"<Fic Title> by <Author>"`, a text body, and a `.epub` attachment.

### Step 5: Test error handling

Try sending without logging in:

```bash
curl -s -X POST http://localhost:3000/api/send-to-kindle \
  -H "Content-Type: application/json" \
  -d '{"url_id": "a1b2c3d4"}' | python3 -m json.tool
```

**Expected**: `{"err": 401, "msg": "Authentication required"}` — the server rejects the request because no Bearer token was sent.

---

## 30.7 Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `no email on file` | The user's `email` and `kindle_email` are both NULL/empty in the `users` table | Ensure the user has registered with an email, or set `kindle_email` in account settings |
| `invalid email` | The address in `kindle_email` or `email` doesn't pass the quick validation (missing `@` or domain dot) | Update the email to a valid address |
| `SMTP send failed: connection refused` | SMTP relay is not running or wrong host/port | Verify `SMTP_HOST`/`SMTP_PORT` and that the relay is reachable |
| `SMTP send failed: authentication failed` | Wrong `SMTP_USER`/`SMTP_PASS` credentials | Double-check the relay credentials |
| 502 Bad Gateway with `failed to fetch chapters` | The source site blocked the scrape or returned an error | The self-heal system records this failure; check the heal log and retry. See Part 21 for the self-healing pipeline. |
| EPUB generated but not received | SMTP relay swallowed the message (common with Gmail's "less secure apps" or Office365) | Use a relay that accepts plain SMTP (MailHog, SMTP4Dev, or a properly configured account like Gmail App Passwords) |

### Checking the config at startup

If SMTP is not configured, the server logs a notice (see `src/config.rs` line 472-473):

```rust
// Send-to-Kindle (SMTP) — optional; if SMTP_HOST is unset the
// /api/send-to-kindle endpoint reports SMTP not configured.
```

You can also check from the API. If `SMTP_HOST` is empty, the handler still runs — it will fail at the `build_and_send` step with an SMTP error, which propagates as a 500. So make sure your environment variables are set *before* starting the server.

---

## 30.8 What you have now

- You understand the full Send-to-Kindle pipeline: frontend button → `kindle.ts` client → `POST /api/send-to-kindle` → EPUB generation → SMTP email with attachment.
- You understand the backend handler's two entry paths: `url` (fresh scrape) or `url_id` (cached DB lookup).
- You understand the email resolution logic: `kindle_email` with `email` fallback, validated before any EPUB work.
- You understand the `build_and_send` function: scrape/fetch chapters → `create_epub` → cache → log → `spawn_blocking` mailer → cleanup.
- You understand the `Mailer` trait abstraction and how `SmtpMailer` vs `MockMailer` enable testing without a real relay.
- You understand the `lettre` 0.11 dependency configuration (no TLS by default; `builder_dangerous` for plain SMTP).
- You understand the frontend: the `sendToKindle` API client, the button's three states (idle/sending/sent), error display, auth gating, and i18n strings from the locale dictionary.
- You can test the feature end-to-end and diagnose common failures.

---

On to [Part 31 — Roadmap Consensus Engine](./31-roadmap-consensus/21-roadmap-consensus.md).

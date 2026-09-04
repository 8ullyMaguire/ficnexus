# Part 4 — The Scraper Subsystem

> **Part 4 of 13** — Part 3 took us underground: configuration, Postgres,
> Redis, and the anti-bot defenses. That was the *memory* of FicHub. But a
> memory isn't any good if the brain never receives new input. This part is
> the input: the scraper subsystem, the code that reaches out across the
> internet and drags fanfiction back home. We open `src/scrape/mod.rs` — the
> `FicMetadata` and `Chapter` shapes, the `ScrapeError` enum, and the
> `SiteScraper` trait that every site adapter implements — then spend a
> chapter inside the AO3 scraper, a chapter on the FanFicFare catch-all
> (plus FF.net, RoyalRoad, and the XenForo forums), a chapter on the
> registry's `find_specific_or_fff` trick, and finish with the body cache:
> the on-disk store that makes FicHub a *cache of all gathered fanfiction*
> rather than a mere proxy. By the end of Part 4, when someone asks "how
> does FicHub get a story out of a URL?" you'll be able to walk them
> through the entire pipeline, from the first regex to the last sharded
> JSON blob.

---

## Chapter 14 — How FicHub Talks to Other Sites: scrape/mod.rs

Part 3 ended with a promise: "The data's in place — now we make it answer."
Here we are, and the first thing that has to *answer* is the question every
user asks first: **"what is this fic?"** Give FicHub a URL —
`https://archiveofourown.org/works/21845264` — and it has to reply with a
title, an author, a word count, a status, a description, and eventually the
full text of every chapter.

That sounds simple. It is not. Here is the problem in one sentence:

> **Every fanfiction site on the internet is a different pile of HTML, and
> none of them publish an official API.**

AO3 has no public API. FanFiction.net has no public API. SpaceBattles is a
forum running XenForo with a different DOM layout than RoyalRoad's custom
framework. The only universal interface is: fetch the page, read the HTML,
and *extract* what you need with CSS selectors and regular expressions.

So before we can export a single EPUB, we need a subsystem that:

1. Recognizes which site a URL belongs to,
2. Fetches that site's pages politely (correct User-Agent, timeouts),
3. Extracts metadata (title, author, word count, status, dates…),
4. Extracts the chapter texts,
5. Fails *gracefully and specifically* when the site says "not found,"
   blocks us, or the HTML doesn't match what we expected.

That subsystem is `src/scrape/`. Let's look at the map before we dive in:

```text
src/scrape/
├── mod.rs              # the shapes: FicMetadata, Chapter, ScrapeError,
│                       # the SiteScraper trait, generate_url_id
├── registry.rs         # ScraperRegistry: find a scraper for a URL
├── sites/
│   ├── mod.rs          # declares the site modules
│   ├── ao3.rs          # Archive of Our Own
│   ├── ffnet.rs        # FanFiction.net (and FictionPress — same layout)
│   ├── royalroad.rs    # RoyalRoad
│   ├── xenforo.rs      # SpaceBattles, SufficientVelocity, QuestionableQuesting
│   ├── fanficfare.rs   # the catch-all: shells out to the FanFicFare tool
│   ├── fictionpress.rs # re-export of ffnet
│   ├── adultfanfiction.rs
│   └── hpfanfic.rs     # fanficauthors.net / hpfanficarchive.com
└── compat_fichub_net.rs # #[cfg(test)] compatibility tests vs fichub.net
```

One module per site. Every site module implements the same trait. That
uniformity is the whole architecture: **the rest of the codebase never
needs to know which site a fic came from.** It just calls `lookup()` and
`fetch_chapters()` and gets back typed structs.

### 14.1 The shapes: what a "fic" is to FicHub

Open `src/scrape/mod.rs`. The first thing you meet is a helper struct used
for search — `ExtractedTag`:

```rust
/// A tag extracted from a fanfiction site during scraping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
    /// Relative importance for "main" semantics: the first character in the
    /// metadata is the MAIN character (score 10), secondary characters 1;
    /// the first ship is the primary pairing (5), others 1; freeforms etc. 0.
    /// Search uses this to answer "attribute applies to the main character".
    pub score: i16,
}
```

We'll spend real time on tags in Chapter 16, because the `score` field
powers one of FicHub's most interesting search features (the "Dark Harry"
semantics from the outline — Part 6 territory). For now, just notice the
shape: every tag has a name, a numeric *type* (`tag_type_id`: 1 fandom,
2 character, 3 relationship, 4 freeform, 5 warning, 6 category), and a
score encoding *importance*. The convenience constructors right below make
creating tags readable:

```rust
impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1, score: 0 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2, score: 1 } }
    pub fn character_main(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2, score: 10 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3, score: 1 } }
    pub fn relationship_primary(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3, score: 5 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4, score: 0 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5, score: 0 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6, score: 0 } }
}
```

This is the *builder* pattern in miniature: `ExtractedTag::character_main("Harry Potter")`
is far more readable than constructing the struct by hand with
`tag_type_id: 2, score: 10` scattered across twenty call sites. And when
the search layer (Part 6) needs to answer "is this tag about the main
character?" it reads one field: `score >= 10` for characters, `>= 5` for
relationships.

💡 **Key Concept — A `tag_type_id` is a foreign key without a join.** The
scraper produces tags with numeric type IDs; the database (Part 3's
migrations) stores those same IDs in a `tag_types` table. The scraper
subsystem never queries that table — it just agrees on the numbers. That's
a *contract by convention*: two layers of the system share a vocabulary of
integers so the scraper can stay database-free and testable. When you read
code that uses bare `1`, `2`, `3` constants, hunt for the doc comment that
defines the convention — it's usually one comment away, and it *is* the
documentation.

Next, the star of the show. Every scraper in the codebase returns one of
these:

```rust
/// Metadata scraped from a fanfiction site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,         // unix millis
    pub updated: i64,           // unix millis
    pub status: String,         // ongoing, complete, hiatus, cancelled
    pub source: String,         // original URL
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

Read it like a checklist — this is the answer to "what is this fic?" in
seventeen fields. Some notes on the ones that aren't obvious:

- **`url_id`** — the fic's *identity* inside FicHub. Not the URL (URLs can
  change, sites get renamed); a hash we generate deterministically from
  `(source_id, story_id)`. We'll see `generate_url_id` in a moment. This is
  the value that ends up in database primary keys, cache filenames, and
  reader URLs like `/read/3fa2c891b04e`.
- **`published` / `updated`** are `i64` unix *milliseconds* — a raw number,
  not a formatted date. Why? Because serialization to JSON is trivial, the
  database stores `bigint`, and formatting is a presentation concern the
  frontend handles (Part 7's SvelteKit world). Scrapers that only get a
  human date like `"2008-09-16"` convert it with `parse_date_to_millis`
  (we'll meet it in Chapter 16).
- **`status`** is a plain `String` constrained by convention to
  `ongoing`, `complete`, `hiatus`, `cancelled` — the doc comment says so,
  and each site scraper maps its own vocabulary onto it ("In-Progress" →
  `ongoing`, "Completed" → `complete`).
- **`source`** is the *original* URL we scraped — kept so we can re-fetch
  later, and so the heal system (Part 9) knows where a failure happened.
- **`source_id`** — the numeric site ID shared with fichub.net. AO3 is 1,
  FFN is 2, FictionPress/fnac is 3, SpaceBattles/SufficientVelocity is 4,
  Wattpad is 5, RoyalRoad is 6, unknown is 99. These numbers are a
  *compatibility contract*: `compat_fichub_net.rs` contains tests asserting
  our mapping matches the public fichub.net instance, because if they ever
  diverge, the same story would get *different `url_id`s on different
  instances*.
- **`author_local_id`** — the author's identifier *on the source site*
  (for AO3 it's the work ID, for FFN the story ID, for FanFicFare the
  author ID string). Slightly oddly named, but it's the key scrapers stash
  site-specific identifiers in so `fetch_chapters` can rebuild URLs.
- **`content_hash`, `extra_meta`, `raw_extended_meta`** — all `Option`al;
  `None` in most scrapers. `extra_meta` is where RoyalRoad tucks its genre
  tags as a comma-joined string; the `Option` type says "this field is
  allowed to be absent" — which is exactly the *illegal states
  unrepresentable* philosophy from Part 3.

And the chapter:

```rust
/// A single chapter's content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String, // HTML content
}
```

Three fields. Note the comment: `content` is **HTML content**, not plain
text. That's a deliberate choice you'll see pay off in Part 5 (the EPUB
builder can embed the HTML directly, preserving italics, links, and
paragraph structure without any re-parsing). The scraper's job is to
*faithfully move* the site's markup into our struct, not to sanitize or
simplify it. Sanitization happens later, at the edge (rendering), where it
belongs.

### 14.2 Failing with style: ScrapeError

Scraping is the most failure-prone code in the entire platform: network
timeouts, HTTP 403 blocks, reCAPTCHA walls, renamed CSS classes, stories
deleted mid-scrape. A generic `Box<dyn Error>` would bury all of that
under one indistinguishable blob. FicHub instead defines exactly the
failure modes it cares about:

```rust
/// Scraper error types
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}
```

Four variants. That's the whole taxonomy, and each one means something
different to the system:

- **`NotFound`** — the fic isn't there (HTTP 404, or a page that parses to
  nothing). The user's URL is bad or the story was removed. *Friendly
  error, nothing to fix.*
- **`Blocked`** — the site refused us (HTTP 403/429, bot wall). The fic
  exists; *we* can't have it. Retrying harder is exactly wrong.
- **`Network(String)`** — DNS failure, connection refused, timeout. The
  string carries the underlying message. Transient; a retry might work.
- **`ParseError(String)`** — we got a page, but the HTML didn't match our
  expectations (a selector found nothing). The string says what went
  wrong. This is the *site changed under us* signal — the kind of failure
  the heal system (Part 9) fingerprints and dedupes.

Why does this granularity matter? Because the heal classifier in
`src/heal/classifier.rs` maps `ScrapeError` onto its own
`ErrorKind { Blocked, Timeout, Parse, NotFound, Export, Unknown }` and
uses that to decide whether a domain is worth an automated agent
diagnose-run. A `Blocked` is *structural* (site is hostile to scraping); a
`ParseError` is *transient* (HTML changed, retry in an hour). If every
failure were one blob, that whole self-healing loop would be blind. **Error
enums are information** — spend the extra ten minutes to make them precise.

The impls make the enum usable in the standard ecosystem:

```rust
impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

`Display` gives every variant a human-readable sentence (so `AppError`
can wrap it into a JSON error response — see `error.rs` from Part 2's
`ScrapeError(String)` variant), and the blanket `impl std::error::Error`
is Rust's way of saying "this type is a proper error" so it works with
`?`, `Box<dyn Error>`, and logging frameworks. Notice the *test-first*
discipline from Part 2 showing up again: the module ends with tests like
`test_scrape_error_display_blocked` asserting
`format!("{}", ScrapeError::Blocked) == "blocked by site"` — locking the
user-visible strings so nobody silently changes an error message users
might be matching on.

### 14.3 The contract: the SiteScraper trait

Now the heart of the subsystem. Every site adapter implements this trait:

```rust
/// Trait that all site scrapers must implement
#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;

    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;

    /// Extract structured tags from a fic URL (optional, default empty).
    /// Override for sites that expose tags (AO3, FF.net, etc.).
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

Let's unpack this, because it's the single most important abstraction in
the whole subsystem:

- **`can_handle(&self, url) -> bool`** — a cheap, synchronous check: "is
  this URL mine?" The AO3 scraper answers
  `url.contains("archiveofourown.org")`. It never touches the network; the
  registry calls it in a loop to pick a scraper (Chapter 17).
- **`lookup(&self, client, url) -> Result<FicMetadata, ScrapeError>`** —
  fetch the story page, parse the metadata, return the 17-field struct.
  The `client` parameter is *injected*, not created inside the scraper —
  that's dependency injection (you met the concept in Part 2's
  `AppState`). One shared `reqwest::Client` means one shared connection
  pool, one shared timeout config, and one place to set the User-Agent.
- **`fetch_chapters(&self, client, meta) -> Result<Vec<Chapter>, ...>`** —
  given the metadata (which carries `source`, `author_local_id`,
  `chapters`), fetch the chapter content. Note it takes `&FicMetadata` —
  the *lookup* and the *fetch* are two separate network passes, which lets
  the export pipeline (Part 5) cache metadata without re-fetching bodies,
  and vice versa.
- **`extract_tags(...)`** — the only method with a *default
  implementation*: `Ok(Vec::new())`. Most sites don't expose structured
  tags, so most scrapers don't override it. AO3's does (we'll see the
  tag-parsing in Chapter 16 via FanFicFare). This is the *interface
  evolution* pattern: when a new capability arrives, give it a sensible
  default so existing implementors keep compiling.

The `#[async_trait]` attribute deserves its own beat. Rust's `async fn`
in traits is only natively supported in recent editions; `async_trait`
(from the crate of the same name) is the classic macro that rewrites each
`async fn` into a method returning a `Pin<Box<dyn Future>>`. The
`Send + Sync` supertraits say: any scraper must be safe to share across
threads — which matters because Axum handlers run on a multithreaded
runtime and `ScraperRegistry` is wrapped in `Arc` (remember
`scraper_registry: Arc<ScraperRegistry>` in `AppState`, and
`Arc::new(ScraperRegistry::new())` in `server.rs`). Without `Send + Sync`,
you couldn't hold a registry in shared state at all.

💡 **Key Concept — The trait is the contract; the structs are the
postage.** `SiteScraper` is FicHub's answer to "the internet is a pile of
different HTML": it defines a tiny, fixed interface (recognize, lookup,
fetch chapters) and lets each site implement it however it likes. The rest
of the codebase — registry, routes, export pipeline, heal system — only
ever sees the trait. Adding a new site means writing one new module and
pushing it in the registry's constructor; nothing else changes. That's the
**strategy pattern**, and it's why FicHub can support eight-plus sites
with a single export pipeline. Whenever you find yourself writing
`if url.contains("ao3") { ... } else if url.contains("ffn") { ... }`,
you're one step away from needing a trait — extract the branch into an
implementor and let a registry pick.

### 14.4 The identity: generate_url_id

Remember `url_id`? Here's how it's born:

```rust
/// Generate a deterministic url_id from source_id and story_id
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    // Use first 12 hex chars for a compact but unique ID
    hex::encode(&result[..6])
}
```

A SHA-256 hash of the string `"{source_id}:{story_id}"`, truncated to the
first 6 bytes, rendered as 12 hex characters. Three properties make this
the right identity scheme:

1. **Deterministic.** The same fic scraped today, next week, or on another
   server produces the *same* `url_id`. No random UUIDs — which would
   duplicate a fic every time it was re-scraped. The tests
   (`test_generate_url_id_deterministic`) lock this property down.
2. **Site-safe.** Because `source_id` is part of the hash input, story ID
   `12345` on AO3 (source 1) and story ID `12345` on FFN (source 2) get
   *different* ids (`test_generate_url_id_different_source_id`). Two sites
   sharing numeric IDs can never collide.
3. **Compact and filesystem-safe.** 12 lowercase hex chars: no slashes, no
   dots, no case sensitivity, short enough for cache filenames and URLs.
   The `url_id_is_12_hex_chars` tests in `compat_fichub_net.rs` assert the
   exact format fichub.net uses — so two FicHub instances (say, yours and
   the public one) address the same fic by the same id. The `#[cfg(test)]`
   compat module replicates the algorithm *independently* and compares
   against known fics — a golden-file test, in the wild.

🧪 **Try It Yourself — watch the ids come out.** The url_id tests are pure
and fast — run them now:

```bash
cd /personal/documents/code/rust/fichub
cargo test generate_url_id
```

You should see all eight `test_generate_url_id_*` tests pass in a blink.
Then open `src/scrape/compat_fichub_net.rs` and read
`url_id_known_ao3_work` — it computes `generate_url_id(1, "21845264")` for
a real, well-known AO3 fic and asserts the hash matches a precomputed
value. That test is the whole compatibility story in one function: *if
this hash ever changes, every bookmarked fic id in the database changes
with it* — so the test exists to make that change loud and impossible to
ignore.

⚠️ **Watch Out — a hash prefix is a collision gamble, and it's a
deliberate one.** Twelve hex chars is 48 bits: with hundreds of thousands
of fics, birthday-paradox collisions are theoretically possible (roughly
when you approach ~16 million ids — the square root of 2^48 — collisions
become likely). FicHub accepts this: the id is compact, and a collision
would surface as a wrong fic in one lookup — detectable, rare, and far
cheaper than the 64-char alternative in every URL, filename, and database
row. When you design your own id scheme, write down *why* you picked the
length — future-you will want to know the tradeoff was made on purpose.

### 14.5 What Chapter 14 taught you

You now know the vocabulary every scraper speaks:

- **`FicMetadata`** — seventeen typed fields answering "what is this
  fic?", with `url_id` as the deterministic cross-site identity.
- **`Chapter`** — a numbered, titled HTML blob; the *content* payload
  exports render.
- **`ScrapeError`** — four precise failure modes (not found / blocked /
  network / parse) that the heal system classifies into self-healing
  decisions.
- **`SiteScraper`** — the four-method trait (recognize, lookup, fetch
  chapters, optional tags) that lets one export pipeline serve eight
  sites.
- **`generate_url_id`** — SHA-256 of `source_id:story_id`, 12 hex chars,
  deterministic and compatible across instances.

The shapes are the easy 20%. The hard 80% — actually *extracting* those
fields from real-world HTML — starts now, with the site that arguably does
the most to make scrapers' lives easy: Archive of Our Own.

---

## Chapter 15 — The AO3 Scraper: sites/ao3.rs

### 15.1 Why AO3 first

<!-- PART4-CONTINUES -->

### 15.2 The scraper's shape: `can_handle` and `lookup`

Open `scrapers/src/sites/ao3.rs` and look at the top of the impl block.
Every scraper in the crate implements the same `SiteScraper` trait, and
AO3 is the cleanest example to start with.

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains(&self.domain)
}

async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

    let fic_url = format!("{}/works/{work_id}?view_full_work=true", self.base_url());
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    // ... metadata parsing ...
}
```

`can_handle` is the *router's* question: "does this URL belong to you?"
AO3 answers by checking whether the URL contains its domain. That's it —
one line. The interesting work happens in `lookup`.

🧪 **Try It Yourself**: change `can_handle` to require the URL to contain
`/works/` too, and watch what happens when someone pastes an AO3 *author*
page URL. (Hint: you'd need `list_author_works` to step in — which is
exactly why `is_author_page` exists further down the file.)

`lookup` does four things, and it's worth naming them because every
scraper follows the same skeleton:

1. **Extract the ID** from the URL (`extract_work_id`). The ID is the
   stable thing you'll use everywhere else.
2. **Fetch the page** with a browser-ish User-Agent. AO3 works fine with
   `fichub.net/0.1.0` here; other sites are pickier (that's why the
   shared `http::fetch` helper exists).
3. **Check the status** — if AO3 returns anything that isn't success,
   the fic probably doesn't exist → `ScrapeError::NotFound`.
4. **Parse the HTML** into a `FicMetadata`.

💡 **Key Concept**: A scraper's `lookup` is a *pure read*: URL in,
`FicMetadata` out. It never writes to the database, never mutates shared
state. That makes scrapers trivially testable — give them a URL and a
client, and assert on the metadata they return.

⚠️ **Watch Out**: `extract_work_id` returns `Option<String>`. If the URL
is malformed, we *could* panic — but we don't. We map it to a
`ScrapeError::ParseError` so the caller can show a friendly message.
"Never panic on bad input" is the #1 rule of scraper code.

### 15.3 `view_full_work=true` — the scraper's best friend

The AO3 URL we build has a magic query parameter:

```rust
let fic_url = format!("{}/works/{work_id}?view_full_work=true", self.base_url());
```

AO3 normally splits a multi-chapter fic across *many* pages (one chapter
per page). But `?view_full_work=true` makes AO3 render **the whole work
on one page**. That single query parameter collapses the entire
chapter-fetching problem into: download one page, split on chapter
headers.

This is a great lesson in **reading the site before writing the
scraper**. The first version of this scraper probably fetched each
chapter separately — until someone noticed the site had a "show entire
work" toggle and the whole approach simplified.

🧪 **Try It Yourself**: open an AO3 multi-chapter fic in your browser and
append `?view_full_work=true` to the URL. Watch the address bar and see
the entire story render in one scroll. Now imagine writing a scraper
that had to handle 40 separate chapter pages instead.

### 15.4 Parsing metadata from the page

The title parsing shows the scraper's defensive style:

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| {
        // Fallback: try <title> tag
        let fallback = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        // Extract title from "Title - Author - Fandom | Archive" format
        let t = fallback.split(" - ").next().unwrap_or("").trim().to_string();
        if t.is_empty() || t.len() < 2 {
            tracing::warn!("AO3 title not found via h2.title.heading, HTML snippet: {:?}",
                html.chars().take(2000).collect::<String>());
            "Unknown Title".to_string()
        } else {
            tracing::info!("AO3 title fallback from <title> tag: {}", t);
            t
        }
    });
```

Three layers of defense in one expression:

1. **Primary selector** — `h2.title.heading`. This is where AO3 puts the
   title on a story page.
2. **Fallback** — the `<title>` tag, which has a predictable
   "Title - Author - Fandom | Archive" format. Split on `" - "` and take
   the first piece.
3. **Last resort** — `"Unknown Title"`, with a warning log including an
   HTML snippet so a human (or the self-healing agent!) can debug.

💡 **Key Concept**: a scraper must be *forgiving about markup but loud
about failure*. Sites change their HTML. When they do, you want (a) a
usable fallback, (b) a log you can grep, and (c) a clear error — in that
order. `"Unknown Title"` is the user-facing symptom; the `tracing::warn`
with an HTML snippet is the developer-facing breadcrumb.

⚠️ **Watch Out**: notice the `tracing::warn!` only logs the first 2000
characters of HTML. Logging an entire page could be megabytes. Always
truncate before logging HTML — it keeps logs readable and avoids
accidentally storing huge strings in memory.

## Chapter 16 — FanFicFare, the Catch-All Fallback

### 16.1 Why keep a catch-all at all?

The crate has native adapters for 107 real sites — but the long tail of
the internet is longer than that. Someone will paste a URL from a tiny
archive nobody's written an adapter for. That's where the
`FanFicFareScraper` comes in: it shells out to the FanFicFare CLI
(a battle-tested Python tool with adapters for hundreds of sites) and
reads its JSON output.

Look at its `can_handle`:

```rust
fn can_handle(&self, _url: &str) -> bool {
    true
}
```

It accepts **everything**. That's the definition of a fallback: it never
says "no", so the registry only reaches it when every native scraper has
already declined.

### 16.2 Turning CLI output into `FicMetadata`

```rust
async fn lookup(&self, _client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let meta = fetch_metadata(url)?;

    // Generate source_id from site abbreviation
    let source_id: i64 = match meta.site_abbrev.as_str() {
        "ao3" => 1,
        "ffn" => 2,
        "fnac" => 3,
        "sb" | "sv" => 4,  // SpaceBattles / Sufficient Velocity
        "Wattpad" => 5,
        "royalroad" => 6,
        _ => 99,
    };

    let url_id = generate_url_id(source_id, &meta.story_id);

    Ok(FicMetadata {
        url_id,
        title: meta.title,
        author: meta.author,
        chapters: meta.num_chapters,
        // ...
    })
}
```

The `site_abbrev` mapping is a mini-treaty: it maps FanFicFare's
site-abbreviation strings onto FicHub's numeric `source_id` table so
that `ao3_12345`-style URL ids stay stable no matter which path scraped
them.

🧪 **Try It Yourself**: `generate_url_id(source_id, story_id)` produces
the string `"1_12345"` for an AO3 work with id 12345. Try tracing where
`url_id` flows after this — search for `url_id` in `src/routes/download.rs`
and watch it become the key for the body cache and the bookmark table.

## Chapter 17 — `find_specific_or_fff`: preferring native scrapers

### 17.1 The registry's routing decision

Native adapters are better than the catch-all: they're faster (no CLI
process spawn), more reliable, and give richer metadata. So the registry
implements a simple policy: **native first, FanFicFare last**.

```rust
/// Find the best scraper for a URL: prefer a native (non-catch-all)
/// scraper when one can handle the URL, falling back to FanFicFare.
pub fn find_specific_or_fff(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.inner.find_specific_or_fallback(url)
}
```

The FicHub wrapper delegates to the crate's `find_specific_or_fallback`,
which iterates the registered native scrapers, returns the first whose
`can_handle` returns true, and only if *none* match falls back to the
catch-all.

💡 **Key Concept**: this is a classic **strategy selection** pattern. The
"best" scraper is chosen by a policy, not by the caller. The caller just
says "give me whatever can handle this URL" — the registry decides the
priority. If a new native adapter is registered tomorrow, every existing
call site automatically prefers it over the CLI fallback. That's the
power of a registry.

### 17.2 The login-aware path

Modern FicHub also has `lookup_authed`, which runs a login pre-pass
before the lookup when the site needs one:

```rust
pub async fn lookup_authed(
    &self,
    client: &reqwest::Client,
    url: &str,
    creds: &[fanfic_scrapers::SiteCredentials],
    heal: Option<&crate::heal::HealService>,
) -> Result<FicMetadata, ScrapeError> {
    match self.inner.find_scraper(url) {
        // ...
    }
}
```

Sites like fanfics.me or fictionhunt require a login for some stories.
The credentials come from environment variables
(`FANFICSCRAPER_<DOMAIN>_USER` / `_PASS`), and the client carries a
cookie store so the session persists across requests.

🧪 **Try It Yourself**: run `grep -rn "FANFICSCRAPER" src/` in the repo
and trace how an env var becomes a `SiteCredentials` entry, then flows
into `lookup_authed`. Follow the whole journey — env var → config →
registry → scraper → HTTP cookie.

## Chapter 18 — The body cache: every scraped fic saved to disk

### 18.1 Why cache entire bodies?

Scraping is expensive and fragile: the source site might be down, slow,
or blocking you. But a fic you've already scraped once doesn't change
every hour. So FicHub persists **the entire scraped body** to disk — the
site becomes a cache of all fanfiction it has gathered.

The design lives in `src/body_cache.rs`. The core idea: files, not
database rows. The body is a JSON blob on an attached drive, keyed by
`url_id`, sharded into subdirectories so no single directory gets huge.

```rust
pub fn save_html(config: &Config, url_id: &str, html: &str, version: i32) -> std::io::Result<PathBuf> {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    std::fs::create_dir_all(&dir)?;
    let path = html_path(&config.body_cache_dir, url_id, version);
    let tmp = dir.join(format!("{url_id}.v{version}.html.tmp.{}", std::process::id()));
    std::fs::write(&tmp, html)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}
```

### 18.2 Atomic writes and versioning

Two details in `save_html` are worth studying closely:

1. **Write to a temp file, then rename.** The `.tmp.{pid}` suffix means
   the write goes to a unique temp file first, and only once it's fully
   written does the atomic `rename` make it visible at the real path. If
   the process crashes mid-write, you never see a half-written file at
   the final path. This is the classic *atomic write* pattern.

2. **Versioning.** Files are named `{url_id}.v{version}.html`. When a
   fic is re-scraped, the version bumps — old versions stay on disk
   (so a curator can compare, and roll back if the new scrape is worse).

```rust
pub fn load_html(config: &Config, url_id: &str) -> Option<String> {
    let v = current_version(config, url_id);
    for ver in (1..=v).rev() {
        let path = html_path(&config.body_cache_dir, url_id, ver);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                return String::from_utf8(bytes).ok();
            }
        }
    }
    None
}
```

`load_html` walks versions *newest first* and returns the first one that
exists. This makes the cache resilient: if the newest version is corrupt
or missing, an older one is used automatically.

### 18.3 Structured bodies

The HTML cache is the raw material; the *structured* cache is what
exports actually consume:

```rust
pub fn save_body(config: &Config, url_id: &str, chapters: &[Chapter], source: Option<String>, version: i32) -> std::io::Result<PathBuf> {
    let dir = shard_dir(&config.body_cache_dir, url_id);
    std::fs::create_dir_all(&dir)?;
    let blob = BodyBlob {
        url_id: url_id.to_string(),
        chapters: chapters.to_vec(),
        saved_at_ms: chrono::Utc::now().timestamp_millis(),
        source,
    };
    // ...serialize to JSON and write...
}
```

A `BodyBlob` holds the ordered chapters plus metadata (`saved_at_ms`,
`source`). Exports read the blob and rebuild the EPUB/HTML/MD from it —
no re-scrape needed. This is why repeat downloads are instant: the
second export of a fic is just "read blob from disk → build EPUB".

💡 **Key Concept**: there are *two* caches with different jobs. The HTML
cache preserves exactly what the site served (debugging, curator
comparison, re-parsing after a scraper fix). The body cache preserves
what *we* extracted (fast export, offline resilience). Separating "raw"
from "structured" is a powerful pattern: you can rebuild one from the
other, and each stays clean for its own consumers.

### 18.4 Curator fixes and peer voting

Because the cache is authoritative for exports, a *wrong* cached body is
a real problem. That's why FicHub gives curators a fix path —
`/api/curator/content/{url_id}` — and why fixes require **peer voting**
(migration 032): propose → other curators vote → apply only at quorum
(≥2 votes, net ≥ 1, no self-vote). The body cache turned from a simple
speed optimization into a *trust layer*: anyone can fix a bad scrape,
but it takes consensus.

⚠️ **Watch Out**: a body cache makes scrapes idempotent, which is great —
but it also means a stale body can outlive a source-site change. When
the source site updates a fic and FicHub re-scrapes, the version bump
handles it. When it *doesn't* re-scrape, curators have the delete
endpoint to force it. Both tools exist because caches drift; the design
acknowledges that.

---

That's Part 4 done — the scraper subsystem is now your playground. You
understand how a URL becomes a `FicMetadata`, why the catch-all exists,
how the registry prefers native scrapers, and how every successful
scrape is preserved on disk for instant re-exports.

Next, in **Part 5 — Exports: From URL to EPUB**, we follow the journey
past metadata: the export pipeline, the EPUB builder, and how a single
URL turns into a file you can read on any device. See you there.

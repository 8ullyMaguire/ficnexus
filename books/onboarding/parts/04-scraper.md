## Part 4 — The Scraper Subsystem

### Chapter 13: How FicHub Talks to Other Sites

Open `src/scrape/mod.rs`. This is where the "fetch fanfiction from the
internet" magic lives. It defines the shapes every site adapter must honor:

```rust
pub struct FicMetadata {
    pub title: String,
    pub author: String,
    pub description: String,
    pub words: i64,
    pub status: String,
    // ... chapters, updated, etc.
}

pub struct Chapter {
    pub title: String,
    pub body_html: String,
    // ...
}

pub enum ScrapeError {
    Network(String),
    NotFound,
    UnsupportedSite,
    // ...
}

#[async_trait]
pub trait SiteScraper {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, url: &str)
        -> Result<Vec<Chapter>, ScrapeError>;
}
```

The contract is beautifully simple: **can you handle this URL? If yes, give
me metadata and chapters.** Every site adapter implements this trait, and
the rest of the system doesn't care *how* the site is scraped — HTML
parsing, JSON API, FanFicFare subprocess — it only cares about the
`SiteScraper` interface.

The `ScrapeError` enum matters because callers map errors to user-facing
messages and telemetry. A `NotFound` (URL is valid-looking but the fic
doesn't exist) is very different from a `Network` error (upstream site is
down or blocking us). The self-healing system (Part 6) classifies scrape
failures into transient/blocked/structural/systemic — the error variants
feed that classification.

> 💡 **Key concept:** interface over implementation. `SiteScraper` is the
> seam between the site-specific world (HTML, cookies, bot walls) and the
> site-agnostic world (metadata, chapters, exports). New sites = new
> adapter = no changes to the rest of the system.

### Chapter 14: The Registry + `find_specific_or_fff`

The registry (`src/scrape/registry.rs`) owns the list of site adapters.
Two functions matter:

```rust
pub fn find_scraper(&self, url: &str) -> &dyn SiteScraper {
    // the naive version: return the FIRST scraper whose can_handle(url) is true
}

pub fn find_specific_or_fff(&self, url: &str) -> &dyn SiteScraper {
    // 1. try each NON-catch-all native scraper whose can_handle(url) is true
    // 2. fall back to FanFicFare (the catch-all)
}
```

Why does `find_specific_or_fff` exist? This is a real debugging story from
the project's history, and it's worth telling because it explains a subtle
bug class you'll encounter.

**The shadowing problem.** FanFicFare is the catch-all scraper: it accepts
*every* URL and hands it to a Python CLI that knows 100+ sites. In the
registry, FanFicFare is pushed **first** — it's the safety net. But if
`find_scraper` just returned the first match, FanFicFare would *always*
win, and the native scrapers (AO3, XenForo, etc.) would never run. That's
exactly what happened when TheForce.net was added: the native XenForo
scraper was registered, but `find_scraper` kept returning FanFicFare, which
didn't know the site — `UnknownSite` errors for every request.

The fix was `find_specific_or_fff`: **prefer a real native scraper, fall
back to FanFicFare.** Every route that scrapes — meta, export, kindle,
updates, download, recommender, fic_suggestions — now calls this function
instead of `find_scraper`.

The rule for adding a new site:

1. Implement `SiteScraper` for your site (Chapter 13's trait).
2. Add it to the registry **before** the FanFicFare catch-all.
3. If it's a *specific* site (AO3, XenForo, ...), `find_specific_or_fff`
   will prefer it automatically. Don't make your scraper accept every URL
   unless it truly is a catch-all — that shadows everyone else.

> 🧪 **Try it:** read `find_specific_or_fff` and list the sites it would
> prefer over FanFicFare. Then read the registry's construction to see the
> order.
>
> ⚠️ **Watch out:** a new scraper with `can_handle` that returns `true`
> for everything is a landmine. Keep `can_handle` precise (domain + path
> patterns).
>
> 💡 **Key concept:** catch-alls are dangerous when they shadow specific
> adapters. Prefer specificity, fall back to the catch-all.

### Chapter 15: The Body Cache — the Archive on Disk

`src/body_cache.rs` implements the "site as a cache" idea — the most
important architectural decision in the current codebase. Let's unpack it.

**Sharding.** Bodies are stored as JSON files under `BODY_CACHE_DIR`,
sharded by the first characters of the `url_id`:

```
BODY_CACHE_DIR/<2ch>/<2ch>/<url_id>.json
# e.g. url_id "xenforo_50062326" → xe/nf/.../xenforo_50062326.json
```

Sharding keeps any single directory small (a few hundred files max) so
filesystem lookups stay fast even with hundreds of thousands of fics.

**Versioning.** Each blob has a `current_version`. When a curator fixes a
body (Part 6), the version bumps — readers and exports use the corrected
content, and the old version is preserved for audit.

**The API:**

```rust
pub fn save_body(config, url_id, body) -> Result<(), CacheError>;
pub fn load_body(config, url_id) -> Result<Option<CachedBody>, CacheError>;
pub fn delete_body(config, url_id) -> Result<(), CacheError>;
pub fn save_html(config, url_id, html) -> Result<(), CacheError>;
```

**The flow.** In the export pipeline (Part 5), the body cache is checked
FIRST. A cache hit skips the network scrape entirely. A miss scrapes,
persists, and serves. Repeat exports are instant, and the archive is
resilient: even if AO3 blocks the server tomorrow, everything already
gathered keeps serving.

This is the concrete meaning of "the site should serve as a cache of all
gathered fanfiction." The database holds metadata; the filesystem holds
content. They're kept in sync by convention: scrape → persist body →
upsert metadata.

> 🧪 **Try it:** export a fic twice and time both. The second call should
> be dramatically faster (cache hit). Then look in `BODY_CACHE_DIR` for the
> sharded blob — you'll see the `xe/nf/...` structure.
>
> ⚠️ **Watch out:** body blobs are gitignored and live on the attached
> drive, NOT in the repo. Never commit them. If `BODY_CACHE_DIR` is
> unset in a test, it falls back to a temp dir — the tests use
> `BODY_CACHE_DIR=/tmp/fichub-body-verify`.
>
> 💡 **Key concept:** DB = metadata, filesystem = content. The cache is
> what makes FicHub a durable archive instead of a scraper-only proxy.

### Chapter 15A: The Site Adapters in Detail

Let's look at the actual adapters to see the range of strategies.

**AO3 (`sites/ao3.rs`).** The cleanest adapter. AO3 pages are
well-structured HTML. The scraper:

1. Extracts the work id from the URL with a regex:
   `r"/works/(\d+)"` → `21845264`.
2. Fetches the page with a normal User-Agent.
3. Parses metadata with the `scraper` crate (CSS selectors): title, author,
   summary, stats.
4. Parses chapter links and fetches each chapter's body.

AO3 also has **series pages** — the adapter can list all works in a
series (`list_series_works`), which is how series exports work.

**FanFicFare (`sites/fanficfare.rs`).** The catch-all. It shells out to
the FanFicFare Python CLI, which knows 100+ sites. The wrapper:

1. Builds a FanFicFare command line for the URL.
2. Runs it as a subprocess.
3. Parses the resulting metadata/HTML.

FanFicFare is powerful but: (a) it's a Python dependency, (b) it can fail
with `UnknownSite` for newer/lesser-known sites, and (c) it's slow. That's
why native adapters are preferred — see Chapter 14's `find_specific_or_fff`.

**XenForo (`sites/xenforo.rs`).** Forums (SpaceBattles, SufficientVelocity,
TheForce.Net) run XenForo. The adapter:

1. Recognizes the domain (`boards.theforce.net`, etc.).
2. Parses thread pages: `h1.p-title-value` for the title, `a.username`
   for authors, `article.message-body` for post bodies.
3. Each forum post becomes a chapter.

**The force.net story** is instructive: when TheForce.Net was added, the
native XenForo adapter was registered, but the app kept returning
FanFicFare `UnknownSite` errors because `find_scraper` returned the
catch-all first. The fix — `find_specific_or_fff` — is why the registry
prefers specificity. Also: forum threads have an OP announcement and
comments mixed in; the scraper returns all message bodies as chapters (a
known caveat), and curators fix the body via the peer-voted workflow.

**FFN / RoyalRoad / others.** Each site adapter implements the same
`SiteScraper` trait with site-specific selectors and URL patterns.

> 🧪 **Try it:** open `sites/ao3.rs` and `sites/xenforo.rs` side by side.
> Notice both implement the same trait, but the HTML parsing differs
> completely. That's the interface seam doing its job.
>
> ⚠️ **Watch out:** site HTML changes frequently. When an adapter breaks,
> it's usually a selector change — the classifier will label it
> "structural" (Chapter 26).
>
> 💡 **Key concept:** every site is a `SiteScraper`; the registry prefers
> specific adapters over the catch-all; HTML parsing is adapter-local.

### Chapter 15B: The FanFicFare Wrapper in Depth

`src/scrape/sites/fanficfare.rs` deserves its own look because it's the
safety net and the source of a recurring failure mode. The wrapper runs
the FanFicFare CLI and parses its output:

```rust
// (simplified)
let output = tokio::process::Command::new("fanficfare")
    .arg("--json-meta").arg(url)
    .output().await?;
if !output.status.success() {
    // map stderr → ScrapeError (e.g. "UnknownSite" → UnsupportedSite)
}
```

Common failure modes you'll see in logs:

- `UnknownSite` — FanFicFare doesn't know the domain (the force.net case).
- Timeouts/network errors from the FanFicFare subprocess.
- Parse failures when the site's HTML doesn't match FanFicFare's
  expectations.

The lesson: **the catch-all is a last resort, not a first choice.** Native
adapters are faster and more controllable; FanFicFare covers the long tail
of sites nobody has written a native adapter for.

> 🧪 **Try it:** run the FanFicFare CLI manually on a URL:
> `fanficfare --json-meta <url>` and see its output shape.
>
> ⚠️ **Watch out:** the FanFicFare subprocess is a dependency — if it's
> missing or broken on a host, every catch-all scrape fails. Health checks
> don't cover it; watch logs.
>
> 💡 **Key concept:** FanFicFare is the safety net for the long tail.
> Native adapters win when they exist.

---

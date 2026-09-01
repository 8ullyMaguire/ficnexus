# Part 3: Scraping Fanfiction

*Building the system that reads stories from the web*

---

# Chapter 11: Understanding Fanfiction Sites

## The Landscape of Fanfiction

Imagine you're a librarian, but instead of organizing books on shelves, you're trying to catalog stories scattered across dozens of different websites—each with its own rules, its own layout, and its own way of presenting information. That's the challenge we face in Part 3 of this book: building scrapers that can read fanfiction from different sites and extract the information we need.

In Parts 1 and 2, we built the FicHub backend: an Axum server, a PostgreSQL database, CRUD endpoints for managing stories. But all that infrastructure is useless without data. Our scrapers are the bridge between the wild, messy world of fanfiction websites and our neatly organized database.

Think about what a scraper actually does at a high level. Someone gives it a URL like `https://archiveofourown.org/works/123456` and says, "What's the title of this story? Who wrote it? How many chapters does it have? Give me the content." The scraper has to visit that page, read the HTML, find the right elements, extract the text, and package it all into a neat data structure.

In this chapter, we'll tour the major fanfiction sites to understand how they work. By the end, you'll see why every site needs its own scraper—and why this is both the hardest and most interesting part of the project.

## Archive of Our Own (AO3)

Archive of Our Own, affectionately known as AO3, is the gold standard of modern fanfiction hosting. Run by the Organization for Transformative Works, it's a massive archive with millions of stories across thousands of fandoms. It's also one of the most scraper-friendly sites out there, with clean HTML and well-organized data.

**How AO3 organizes stories:**

Every story on AO3 is called a "work." Each work has:
- A **title** displayed prominently at the top
- An **author** with a profile link
- A **summary** (what they call the "summary" or description)
- **Tags**—lots and lots of tags. Fandoms, characters, relationships, warnings, ratings, and freeform tags
- **Stats**: word count, chapters, kudos, bookmarks, hits
- **Chapters**: each chapter is a separate section with its own title and content
- **Status**: whether the story is complete or still in progress
- **Dates**: published date and last update date

The URL structure is clean and predictable. A story lives at:
```
https://archiveofourown.org/works/123456
```

If the story has multiple chapters, you can link directly to a specific chapter:
```
https://archiveofourown.org/works/123456/chapters/789012
```

The work ID (123456) is a simple numeric identifier that's unique across the entire site. The chapter ID (789012) is also numeric. This makes extracting the ID from a URL straightforward—we just need a regex that matches `/works/(\d+)`.

Here's how the `Ao3Scraper` extracts the work ID:

```rust
fn extract_work_id(url: &str) -> Option<String> {
    // Matches /works/NUMBER or /works/NUMBER/chapters/NUMBER
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

This is pattern matching at its simplest: find `/works/`, grab the digits that follow. The `(\d+)` part is a capture group—it matches one or more digits and captures them as a group. The `?` after `ok()` handles the case where the regex fails to compile (which shouldn't happen with a hardcoded pattern, but Rust's API requires us to handle it).

And similarly, the chapter number extraction:

```rust
fn extract_chapter_number(url: &str) -> Option<i32> {
    let re = regex_lite::Regex::new(r"/chapters/(\d+)").ok()?;
    re.captures(url)?.get(1).and_then(|m| m.as_str().parse().ok())
}
```

Notice the difference: `extract_work_id` returns a `String` (the raw digits), while `extract_chapter_number` returns `Option<i32>` (the parsed integer). This is a design choice—we need the work ID as a string for the URL ID hash, but the chapter number as an integer for display.

**The HTML structure that scrapers look at:**

AO3 uses semantic HTML with clear CSS class names. This is one of the reasons it's so popular among developers—it's a pleasure to scrape (compared to some other sites we'll see shortly). The page structure for a story looks roughly like:

```html
<div id="workskin">
  <div class="preface group">
    <div class="title heading">
      <h2 class="title heading">My Amazing Fanfiction</h2>
      <h3 class="byline heading">
        <a href="/users/authorname/pseuds/authorname" rel="author">AuthorName</a>
      </h3>
    </div>

    <div class="stats group">
      <dl>
        <dt>Published:</dt>
        <dd class="published">2023-01-15</dd>
        <dt>Status:</dt>
        <dd class="status">Complete</dd>
        <dt>Chapters:</dt>
        <dd class="chapters">3 / 5</dd>
        <dt>Words:</dt>
        <dd class="words">42,156</dd>
      </dl>
    </div>

    <div class="summary module">
      <blockquote class="userstuff">
        <p>This is the summary of the story...</p>
      </blockquote>
    </div>
  </div>

  <div id="chapters">
    <div class="chapter">
      <h3 class="heading">
        <span class="chapter">1</span>
        <span class="chapter-title">The Beginning</span>
      </h3>
      <div class="userstuff module">
        <p>Once upon a time...</p>
      </div>
    </div>

    <div class="chapter">
      <h3 class="heading">
        <span class="chapter">2</span>
        <span class="chapter-title">The Middle</span>
      </h3>
      <div class="userstuff module">
        <p>And then things happened...</p>
      </div>
    </div>
  </div>
</div>
```

Notice how the class names are descriptive: `h2.title.heading` for the title, `a[rel='author']` for the author link, `dd.chapters` for chapter count, `dd.words` for word count. These selectors are what our CSS selector engine uses to find the right elements. The HTML is well-structured and predictable, which is exactly what a scraper needs.

**The magic query parameter:**

One of AO3's quirks is that by default, a story page only shows the first chapter. To get all chapters on a single page, you append `?view_full_work=true` to the URL. The AO3 scraper uses this trick:

```rust
let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
```

Without this parameter, fetching a 50-chapter story would require 50 separate HTTP requests—one for each chapter. With it, we get everything in a single request. AO3 is nice about this, and it's a huge efficiency win. The entire story content, including all chapters, metadata, and tags, loads on one page.

**AO3's tag structure:**

AO3 has the richest tagging system of any fanfiction site. Stories are tagged with:
- **Fandom tags** (Harry Potter, Marvel, Star Wars)
- **Character tags** (Harry Potter, Hermione Granger, Draco Malfoy)
- **Relationship tags** (Harry/Hermione, Steve/Tony)
- **Freeform tags** (time travel, fix-it, angst, fluff)
- **Warning tags** (major character death, non-con, underaged)
- **Rating** (General, Teen, Mature, Explicit)
- **Category** (M/M, F/M, F/F, Gen, Multi)

These tags are structured and searchable on the site itself. Our scrapers can extract them and store them in our database for advanced searching and filtering. More on this in Chapter 14.

🧪 **Try It Yourself:** Open your browser and navigate to an AO3 story. View the page source (Ctrl+U or right-click > View Page Source). Search for `class="title heading"` — can you find it? That's the element our scraper targets for the title. Now search for `rel="author"` — that's the author link. You're reading the page exactly like our scraper does.

## FanFiction.net (FF.net)

FanFiction.net is the granddaddy of fanfiction archives. It's been around since 1998, and it shows. The HTML is older, the layout is more complex, and the selectors are less intuitive. But it has an enormous catalog of stories that you won't find anywhere else—some stories have been on FF.net for over two decades.

**How FF.net organizes stories:**

Like AO3, stories are identified by numeric IDs. A story lives at:
```
https://www.fanfiction.net/s/1234567/1/
```

The `s/1234567` part identifies the story, and the `/1/` at the end is the chapter number. You change that number to navigate between chapters.

FF.net's HTML is denser and uses IDs more than classes. The metadata lives in a `#profile_top` section, and many elements share the same class name (`xcontrast_txt`), differentiated by their element type (bold tag for title, anchor tag for author, div tag for description).

```html
<div id="profile_top">
  <b class="xcontrast_txt">My Story Title</b>
  <a href="/u/12345/AuthorName" class="xcontrast_txt">AuthorName</a>
  <div class="xcontrast_txt">The summary text goes here...</div>
  <span class="xgray">Rated: T | Chapters: 3 | Words: 42,156 | Reviews: 52</span>
  <span data-xutitle="word count">42,156</span>
  <span data-xutitle="chapters">3 / 5</span>
</div>
```

Notice how the title is a bold tag (`b.xcontrast_txt`), the author is an anchor tag (`a.xcontrast_txt`), and the description is a div (`div.xcontrast_txt`). They all share the `xcontrast_txt` class, but our selectors distinguish them by element type.

The chapter content lives in a `div.storytext`:

```html
<div class="storytext xcontrast_txt">
  <p>The actual story content...</p>
  <p>More paragraphs...</p>
</div>
```

Notice the `xcontrast_txt` class everywhere? That's a quirk of FF.net's styling. The class seems to be about text contrast rather than semantic meaning. Our scraper needs to target these specific class names, which are less descriptive than AO3's.

**One request per chapter:**

Unlike AO3, FF.net doesn't have a "view full work" option. Each chapter is a separate page. So the FF.net scraper has to loop through all chapters:

```rust
for i in 1..=meta.chapters {
    let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
    // ... fetch and parse each chapter
}
```

This means a 20-chapter story requires 20 HTTP requests. We need to be mindful of rate limiting (more on that later). The trade-off is that each request is small and focused, which makes error handling easier—if chapter 3 fails, we can skip it and continue with chapter 4.

**FF.net's chapter title selector:**

One clever aspect of the FF.net scraper is how it gets chapter titles. FF.net has a dropdown menu (`select#chap_select`) that lists all chapters, with the currently selected option showing the current chapter's title:

```rust
let title = document
    .select(&Selector::parse("select#chap_select option[selected]").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| format!("Chapter {i}"));
```

This selector targets the `<option>` element inside the chapter dropdown that has the `selected` attribute. It's a neat trick—instead of looking for a heading element, we extract the title from the navigation UI.

**FictionPress: FF.net's twin:**

FictionPress.com is a sister site to FF.net—it was created for original fiction, but it uses the exact same codebase. The HTML structure is identical. In fact, the FicHub codebase handles this elegantly:

```rust
/// FictionPress scraper (site shares same structure as FF.net)
pub use FfNetScraper as FictionPressScraper;
```

That's the entire `fictionpress.rs` file. FictionPress is literally the same struct, just re-exported under a different name. The `can_handle` method checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

Smart, right? When two sites share the same structure, why write the same code twice? This is the DRY (Don't Repeat Yourself) principle in action. And because the registry calls `can_handle` first, there's no ambiguity—if a URL points to FictionPress, the same FF.net scraper handles it.

🧪 **Try It Yourself:** Visit a FF.net story and open the page source. Search for `#profile_top` — can you find the metadata section? Now search for `storytext` — that's where the actual story content lives. Compare the HTML structure to AO3's. Can you see why FF.net needs different CSS selectors?

## XenForo Forums (SpaceBattles, SufficientVelocity)

Now we get to the interesting case. SpaceBattles (spacebattles.com), SufficientVelocity (sufficientvelocity.com), and QuestionableQuesting (questionablequesting.com) are XenForo-based forums. They're not traditional fanfiction archives—they're general-purpose forums where people also write and share original fiction.

This creates a fundamental challenge: the sites weren't designed for fanfiction. They're designed for forum discussions. Stories are posted as forum threads, and each "chapter" is a forum post.

**How XenForo works:**

Stories are posted as forum threads. Each "chapter" is a forum post in the thread. The URL looks like:
```
https://forums.spacebattles.com/threads/story-title.12345/
```

That `12345` at the end is the thread ID. But the URL also contains a slug—the human-readable title of the thread. The regex for extracting the thread ID needs to handle this:

```rust
fn extract_thread_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The regex `threads/.*\.(\d+)/?` matches "threads/", then any characters (the slug), then a dot, then captures the digits before the optional trailing slash. The `.*` is greedy—it matches as many characters as possible, then backtracks to find the dot and digits.

**The HTML structure is very different:**

```html
<h1 class="p-title-value">Story Title by AuthorName</h1>

<article class="message" data-author="AuthorName">
  <div class="message-body">
    <div class="bbWrapper">
      <p>First post content (often the story intro or Chapter 1)...</p>
    </div>
  </div>
</article>

<article class="message" data-author="AuthorName">
  <div class="message-body">
    <div class="bbWrapper">
      <p>Second post content (often Chapter 2)...</p>
    </div>
  </div>
</article>
```

Each `article.message` is a forum post, and we treat each one as a chapter. The content lives inside `div.bbWrapper` (BBCode wrapper—XenForo uses BBCode internally).

**The author extraction:**

XenForo stores the author in two places: the `data-author` attribute on the article element, and in an `a.username` link:

```rust
let author_el = document
    .select(&Selector::parse("a.username").unwrap())
    .next();
let author = author_el
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The author URL needs special handling because XenForo uses relative URLs:

```rust
let author_url = author_el
    .and_then(|el| el.value().attr("href"))
    .map(|h| {
        if h.starts_with('/') {
            // Determine domain from the URL
            for domain in XENFORO_DOMAINS {
                if url.contains(domain) {
                    return format!("https://{domain}{h}");
                }
            }
        }
        h.to_string()
    })
    .unwrap_or_default();
```

This is more complex than AO3's author URL extraction because XenForo gives us a relative path like `/members/username.12345/`. We need to figure out which domain it belongs to and prepend it.

**The description extraction:**

For XenForo, the "description" is the first post's content:

```rust
let description = document
    .select(&Selector::parse("article.message-body").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

The first `article.message-body` is the opening post (OP) of the thread, which typically contains the story introduction or description. We use `inner_html()` to preserve any formatting.

**Pagination: XenForo's multi-page threads:**

Long XenForo threads are split across multiple pages. A 50-post thread might be spread across 5 pages (10 posts per page). The scraper currently only fetches the first page, which means it only gets the posts on that page.

The metadata always shows `chapters: 1` because we don't know the total chapter count without fetching all pages:

```rust
Ok(FicMetadata {
    // ...
    chapters: 1,
    words: 0,
    // ...
})
```

This is a known limitation. For a full implementation, the scraper would need to:
1. Parse the page navigation to find the total page count
2. Fetch each page sequentially
3. Combine posts from all pages
4. Handle rate limiting between page requests

For now, the scraper works well for threads that fit on a single page, and provides a reasonable approximation for longer threads by grabbing the first page's posts.

**Domain matching:**

XenForo scrapers need to know which domains to handle. The scraper uses a constant array:

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

And the `can_handle` method checks if the URL contains any of these domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    Self::is_xenforo_url(url)
}

fn is_xenforo_url(url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

If you wanted to add another XenForo forum (there are hundreds out there), you'd just add its domain to the array. The scraper logic is generic enough to handle any XenForo site.

⚠️ **Watch Out:** XenForo forums often have CAPTCHAs, anti-bot measures, and require cookies for certain content. Our basic scraper won't handle authentication-protected threads. If a thread requires a login to view, our scraper will get an error or an empty page.

## Story IDs: The Universal Identifier

Every site uses some form of unique identifier for its stories. Understanding these IDs is crucial because they form the basis of our URL ID system.

| Site | URL Pattern | ID Extraction |
|------|-------------|---------------|
| AO3 | `/works/\d+` | Regex captures digits after `/works/` |
| FF.net | `/s/\d+` | Regex captures digits after `/s/` |
| XenForo | `threads/*.\d+` | Regex captures digits after last dot |
| AdultFanFiction | Numeric segments | First numeric path segment |
| HPFanFic | URL slug | Last URL segment |

Our `generate_url_id` function creates a deterministic, site-independent ID by combining the source ID with the story ID:

```rust
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

The `source_id` is a numeric identifier for each site (AO3=1, FF.net=2, XenForo=3, AdultFanFiction=4, HPFanFic=5). The `story_id` is the site-specific identifier extracted from the URL. By hashing them together with a colon separator, we get a 12-character hex string that's unique across all sites.

The tests for this function are thorough:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}
```

The deterministic test verifies that the same inputs always produce the same output (essential for database lookups). The different-source-id test confirms that the source ID actually matters. The length and hex-char tests verify the output format.

🧪 **Try It Yourself:** Open a browser and navigate to three different fanfiction sites. Pick a story on each one. Look at the URL—can you spot the story ID in each URL? Try copying the URL into a text editor and highlighting the ID portion. That's exactly what our regex patterns are designed to extract.

## Being Polite: Robots.txt and Rate Limits

Web scraping comes with responsibilities. Just because we *can* fetch a page doesn't mean we should hammer the server with requests.

**User-Agent headers:**

Every request our scrapers make includes a User-Agent header that identifies us:

```rust
.header("User-Agent", "fichub.net/0.1.0")
```

This tells the target site who we are. It's basic etiquette—like introducing yourself before entering someone's house. Without a User-Agent, many sites will block your requests because they can't distinguish you from malicious bots.

The version number in the User-Agent (0.1.0) helps site administrators identify which version of our scraper is accessing their site. If we introduce a bug that sends too many requests, they can contact us and ask us to fix it.

**Rate limiting:**

AO3, FF.net, and other sites all have rate limits. If you make too many requests too quickly, you'll get blocked. Our scrapers don't implement rate limiting themselves (that's handled elsewhere in the system), but the architecture is designed to support it.

The `CollectionWorker` in our backend creates per-site rate limiters:

```rust
let mut rate_limiters = HashMap::new();
for fetcher in &fetchers {
    let domain = fetcher.site_domain().to_string();
    let delay = fetcher.rate_limit_delay(&config, &domain);
    rate_limiters.insert(domain, PerSiteRateLimiter::new(delay));
}
```

Each site gets its own delay configuration. AO3 might allow one request every 2 seconds, while FF.net might require 3 seconds between requests. This per-site configuration is essential because different sites have different tolerance levels.

**Timeouts:**

We set reasonable timeouts on our HTTP client:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

30 seconds is generous enough for slow sites but prevents our scraper from hanging indefinitely if a site is unresponsive. Without a timeout, a single slow response could block an entire async task.

**robots.txt:**

Most websites have a `robots.txt` file that tells automated tools which pages they're allowed to visit and how often. For example, AO3's robots.txt might say "don't scrape more than one page per second." Our scrapers should respect these guidelines.

In practice, we implement this through our rate limiter configuration rather than by parsing robots.txt directly. The rate limits are manually configured based on our understanding of each site's preferences.

**Error handling for blocked requests:**

Our `ScrapeError::Blocked` variant handles the case where a site refuses our request:

```rust
pub enum ScrapeError {
    NotFound,      // Story doesn't exist
    Blocked,       // Site blocked our request
    Network(String),  // HTTP/connection error
    ParseError(String),  // Couldn't parse HTML
}
```

If a site returns a 403 (Forbidden) or 429 (Too Many Requests), we map it to `ScrapeError::Blocked`. The caller can then decide whether to retry (after a delay) or give up.

⚠️ **Watch Out:** Some fanfiction sites explicitly prohibit scraping in their Terms of Service. Always check the ToS before scraping a site. AO3 has specific guidelines about automated access. Our scrapers are designed for personal use—building a public scraping service that hammers someone's server is a different matter entirely.

## The Challenge: Every Site Is Different

Here's the fundamental challenge: there's no standard way that fanfiction sites present their data. AO3 uses semantic HTML with clean class names. FF.net uses IDs and custom classes. XenForo uses its own templating system. Each site has evolved independently over years, and they all make different choices.

Consider the simple task of extracting a story title:

| Site | Selector | What it matches |
|------|----------|-----------------|
| AO3 | `h2.title.heading` | An `<h2>` with both `title` and `heading` classes |
| FF.net | `#profile_top b.xcontrast_txt` | A `<b>` inside the profile section |
| XenForo | `h1.p-title-value` | An `<h1>` with the XenForo title class |
| AdultFanFiction | `h1.story_title` | An `<h1>` with `story_title` class |
| HPFanFic | `h1` | Any `<h1>` on the page |

Each site uses a different element, different classes, and different hierarchy. There's no "one selector fits all" solution.

This is why we need a **trait-based architecture**. Instead of writing one monolithic scraper that handles everything, we define a contract (the `SiteScraper` trait) and let each site implement it differently:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

Each scraper only needs to know about its own site's HTML structure. AO3 doesn't care about FF.net's selectors, and FF.net doesn't care about XenForo's forum layout. They all speak the same language at the trait level.

This separation of concerns is what makes the system maintainable. If AO3 redesigns its page, we only update the AO3 scraper. If we want to support a new site, we add a new scraper. The rest of the system—database, API, EPUB generation—never changes.

In the next chapter, we'll dig into how to build these scrapers using Rust's HTTP and HTML parsing tools.

---

# Chapter 12: Building Web Scrapers

## What Is Scraping?

Let's use an analogy. Imagine you walk into a library and pick up a book. You open it, find the table of contents, and read the title page. That's what web scraping is—but the library is a website, the book is a web page, and you're a Rust program.

More precisely, web scraping is the process of:
1. **Fetching** a web page (making an HTTP request)
2. **Parsing** the HTML content (reading the page structure)
3. **Extracting** specific information (pulling out the title, author, content, etc.)

In Rust, we have excellent tools for each step. The `reqwest` crate handles HTTP requests. The `scraper` crate parses HTML and lets us use CSS selectors. And our `SiteScraper` trait ties everything together.

Let's build a scraper from the ground up, step by step.

## The reqwest Crate: Making HTTP Requests

`reqwest` is the de facto HTTP client for Rust async code. It's like `fetch()` in JavaScript, but built for Rust's async/await model. It handles connection pooling, TLS, redirects, and all the details of HTTP so we can focus on what we care about: getting the page content.

**Building a client:**

We create a reusable HTTP client when the server starts. The client is expensive to create (it sets up connection pools and TLS configuration), so we make it once and share it:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

The `Client::builder()` pattern lets us configure:
- **User-Agent**: who we identify as (required by most sites)
- **Timeout**: how long to wait for a response (30 seconds)
- **TLS**: we use `rustls` (not OpenSSL) for HTTPS—this is a Rust-native TLS implementation
- **Features**: we enable JSON support for API calls

The `build()` method returns a `Result<Client, Error>`. We use `.expect()` here because if the client can't be built, the server can't function at all, so a panic is appropriate.

**Making a GET request:**

Every scraper request follows the same pattern. Let's trace through a real request to AO3:

```rust
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

Let's break this down line by line:
1. `client.get(&fic_url)` creates a GET request builder targeting the URL
2. `.header("User-Agent", "fichub.net/0.1.0")` adds our identification header
3. `.send()` actually sends the request over the network (this is the async part!)
4. `.await` suspends the current task until the response arrives
5. `.map_err(|e| ScrapeError::Network(e.to_string()))?` converts any error into our error type and propagates it

The `?` operator is Rust's error propagation shorthand. If `map_err` produces an `Err`, the function returns early with that error. If it produces an `Ok`, the inner value is unwrapped and assigned to `response`.

**Reading the response body:**

Once we have the response, we need to read its body as text (HTML):

```rust
if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}

let html = response.text().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

First we check the HTTP status code using `response.status().is_success()`. This returns `true` for any 2xx status code (200 OK, 201 Created, etc.). If it's not a success, the story probably doesn't exist or has been deleted, so we return `ScrapeError::NotFound`.

Otherwise, `response.text().await` reads the entire response body as a UTF-8 string. This is where the actual HTML lives.

**Response status handling:**

Different status codes mean different things:
- **200 OK**: The story exists and we got the page
- **301/302 Redirect**: reqwest follows these automatically
- **403 Forbidden**: The site is blocking our request → `ScrapeError::Blocked`
- **404 Not Found**: The story doesn't exist → `ScrapeError::NotFound`
- **429 Too Many Requests**: We're rate-limited → `ScrapeError::Blocked`
- **500+ Server Error**: The site has a problem → `ScrapeError::Network`

Currently, our scrapers treat any non-2xx status as "not found." A more sophisticated approach would distinguish between 404 and 403, mapping them to different error variants.

⚠️ **Watch Out:** `response.text().await` reads the entire response body into memory. For a very large story with hundreds of chapters on a single page (like a full-work AO3 view), this could use significant memory. In practice, fanfiction pages rarely exceed a few megabytes, so this is fine. But if you were scraping a site that returns huge pages, you'd want to stream the response instead.

## The scraper Crate: Reading HTML with CSS Selectors

Once we have the HTML, we need to parse it and find specific elements. This is where the `scraper` crate comes in. It provides HTML parsing and CSS selector support, built on top of the `html5ever` parser and the `selectors` crate.

**Parsing HTML:**

```rust
let document = Html::parse_document(&html);
```

This takes our raw HTML string and turns it into a DOM-like tree structure that we can query. The `parse_document` function handles malformed HTML gracefully—it won't crash on unclosed tags or missing attributes.

**Using CSS selectors:**

CSS selectors are the language you use in stylesheets to target HTML elements. If you've ever written `div.content` or `#main-title` or `h2 a`, you've used CSS selectors. The `scraper` crate lets us use them to find elements in parsed HTML.

Here's a concrete example from the AO3 scraper:

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

Let's walk through this step by step:
1. `Selector::parse("h2.title.heading")` creates a selector that matches `<h2>` elements with both `title` and `heading` classes
2. `.unwrap()` is safe here because we know the selector string is valid
3. `document.select(...)` returns an iterator of all matching elements
4. `.next()` gets the first match (we expect only one)
5. `.map(|el| el.text().collect::<String>()...)` extracts all text content from the element and its children
6. `.unwrap_or_else(|| "Unknown Title".to_string())` provides a default if nothing was found

The `.text()` method returns an iterator over all text nodes in the element. We `.collect()` them into a single string. This means if the element contains nested elements (like `<h2>Story <em>Title</em></h2>`), we get "Story Title"—all the text, merged together.

**Common selector patterns in our scrapers:**

Here's a reference table of the selectors used across our scrapers:

| What we're looking for | Selector | Site |
|----------------------|----------|------|
| Story title | `h2.title.heading` | AO3 |
| Author link | `a[rel='author']` | AO3 |
| Chapter count | `dd.chapters` | AO3 |
| Word count | `dd.words` | AO3 |
| Story summary | `blockquote.userstuff` | AO3 |
| Chapter container | `div.chapter` | AO3 |
| Chapter content | `div.userstuff` | AO3 |
| Story title | `#profile_top b.xcontrast_txt` | FF.net |
| Author | `#profile_top a.xcontrast_txt` | FF.net |
| Description | `#profile_top div.xcontrast_txt` | FF.net |
| Chapter content | `div.storytext` | FF.net |
| Chapter title | `select#chap_select option[selected]` | FF.net |
| Thread title | `h1.p-title-value` | XenForo |
| Author | `a.username` | XenForo |
| Forum post | `article.message-body` | XenForo |
| Story title | `h1.story_title` | AdultFanFiction |
| Author | `a.author` | AdultFanFiction |
| Story description | `div.story_description` | AdultFanFiction |
| Story content | `div.story_content` | AdultFanFiction |
| Title | `h1` | HPFanFic |
| Author | `a[href*='author']` | HPFanFic |
| Content | `div.story-content, div.fic-content, article` | HPFanFic |

Notice the variety: ID selectors (`#profile_top`), class selectors (`.userstuff`), attribute selectors (`[rel='author']`, `[data-xutitle='word count']`), and even comma-separated fallback selectors (`div.story-content, div.fic-content, article`).

🧪 **Try It Yourself:** Open your browser's developer tools (F12 or Ctrl+Shift+I) and navigate to a fanfiction page. Use the "Inspector" or "Elements" tab to find the title element. Right-click it and select "Copy > Copy selector" to get the CSS selector. Then try typing that selector into the console with `document.querySelector('your-selector-here')`. You just did manually what our scraper does automatically!

**Selector strategies:**

When writing selectors, there's a trade-off between specificity and robustness:

- **Very specific** (e.g., `#profile_top b.xcontrast_txt`): Works perfectly until the site changes, then breaks completely
- **Very generic** (e.g., `h1`): More resilient to changes, but might match the wrong element
- **Fallback chains** (e.g., `div.story-content, div.fic-content, article`): Tries multiple specific selectors, falling back to more generic ones

Our scrapers generally use specific selectors because we're targeting known, stable sites. But the HPFanFic scraper demonstrates the fallback approach.

## The SiteScraper Trait: A Blueprint for Scrapers

In Rust, a **trait** is like a contract. It says: "Any type that implements this trait must provide these methods." It's how we achieve polymorphism—the ability to write code that works with any scraper, without knowing which specific scraper it is.

Think of it like a job description. The trait says "the applicant must be able to do X, Y, and Z." Different applicants (scrapers) implement those abilities differently, but they all satisfy the job description.

Here's our `SiteScraper` trait from `scrape/mod.rs`:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;

    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;

    /// Extract structured tags from a fic URL (optional, default empty).
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

Let's look at each method:

**`can_handle(&self, url: &str) -> bool`**

This is the routing method. Given a URL, does this scraper know how to handle it? For AO3, it simply checks if the URL contains "archiveofourown.org":

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

For XenForo, it checks against multiple domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    Self::is_xenforo_url(url)
}

fn is_xenforo_url(url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

The `can_handle` method is the first thing the registry calls. It's synchronous (not async) because it doesn't do any I/O—it's just string matching.

**`lookup(&self, client, url) -> Result<FicMetadata, ScrapeError>`**

This is the main extraction method. Given a story URL, it:
1. Extracts the story ID from the URL
2. Fetches the story page via HTTP
3. Parses the HTML
4. Extracts metadata (title, author, chapters, words, description, etc.)
5. Returns a `FicMetadata` struct

This is where most of the scraper's work happens. The `client` parameter is the shared HTTP client—we pass it in rather than creating one inside the scraper, which allows for connection pooling and shared configuration.

**`fetch_chapters(&self, client, meta) -> Result<Vec<Chapter>, ScrapeError>`**

Once we have metadata, we need to download the actual chapter content. This method takes the metadata (which includes the URL and chapter count) and fetches each chapter's HTML content.

The separation between `lookup` and `fetch_chapters` is intentional. You might want to look up metadata without downloading all the content (for example, to show a preview). Or you might want to fetch chapters separately (for example, to resume a failed download).

**`extract_tags(&self, client, url) -> Result<Vec<ExtractedTag>, ScrapeError>`**

This is an optional method with a default implementation that returns an empty vector. It lets scrapers that can extract structured tags (like AO3 with its rich tagging system) provide that data. Sites without rich tags simply don't override this method.

The default implementation is defined right in the trait:

```rust
async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    Ok(Vec::new())
}
```

Note the underscore-prefixed parameters (`_client`, `_url`). This tells Rust "I know these parameters exist but I don't use them in this default implementation." It prevents unused-variable warnings.

**Why `Send + Sync`?**

The `Send + Sync` bounds on the trait mean that any scraper must be safe to send between threads and share references across threads. This is essential because we wrap our scraper registry in `Arc` and share it across async tasks. Without these bounds, the compiler would refuse to let us share scrapers across the `tokio` runtime.

## The FicMetadata Struct: What We Extract

Every scraper returns a `FicMetadata` struct—the standardized representation of a fanfiction story's metadata. This is the bridge between the scraper world and the database world.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,           // Deterministic ID from source_id + story_id
    pub title: String,            // "My Amazing Story"
    pub author: String,           // "AuthorName"
    pub chapters: i32,            // Number of chapters published
    pub words: i64,               // Total word count
    pub desc: String,             // Story description/summary (HTML)
    pub published: i64,           // Unix timestamp (milliseconds) when published
    pub updated: i64,             // Unix timestamp (milliseconds) when last updated
    pub status: String,           // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,           // Original URL
    pub source_id: i64,           // Numeric site identifier (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Database ID for the author (0 if unknown)
    pub author_url: String,       // Link to author's profile page
    pub author_local_id: String,  // Site-specific identifier for the author/story
    pub content_hash: Option<String>,  // Hash of content for change detection
    pub extra_meta: Option<String>,    // Additional metadata (JSON)
    pub raw_extended_meta: Option<String>,  // Raw extended metadata
}
```

Let's look at some design decisions:

**Why `i64` for timestamps?**

We store timestamps as Unix milliseconds (the number of milliseconds since January 1, 1970). Using `i64` gives us enough precision and range to handle any date we'll encounter. Millisecond precision is more than enough for fanfiction—we rarely need sub-second precision for publish dates.

When a scraper doesn't know the exact publish date, it uses the current time:

```rust
let now = Utc::now().timestamp_millis();
```

**Why `String` for status?**

We use simple strings for status: "ongoing", "complete", "hiatus", "cancelled". An enum would be more type-safe, but the database stores this as text, and the string representation is easier to work with across different parts of the system. The API returns strings, the EPUB generator reads strings, and the database stores strings.

**Why `Option<String>` for `content_hash`?**

The `content_hash` field is used for change detection. When we scrape a story, we can hash its content and compare it to a previously stored hash. If the hash is different, the story has been updated. This field is `Option` because we don't always compute it—sometimes we just want the metadata without doing a full content comparison.

**Why `extra_meta` and `raw_extended_meta`?**

These are escape hatches for site-specific data that doesn't fit into the standard fields. For example, AO3 has kudos, bookmarks, and hits—data that's unique to AO3 and doesn't have equivalents on other sites. We can store this as JSON in `extra_meta` without polluting the standard fields.

**The derive macros:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

- `Debug`: Lets us print the struct for debugging (e.g., `println!("{:?}", metadata)`)
- `Clone`: Lets us create copies (needed because we sometimes pass metadata to multiple functions)
- `Serialize` / `Deserialize`: Lets us convert to/from JSON (needed for API responses and database storage)

## The Chapter Struct

Chapters are simpler than metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String, // HTML content
}
```

The `content` field stores HTML, not plain text. This is intentional—fanfiction often has formatting (bold, italic, blockquotes, images) that we want to preserve. When we generate EPUBs later, we'll convert this HTML to EPUB-compatible markup.

The `chapter_id` is 1-indexed (starts at 1, not 0) to match human numbering conventions.

## async_trait: Making Traits Work with Async

Rust's trait system has a limitation: trait methods can't natively be async. This is because async functions return a `Future`, and the compiler can't know what concrete future type a trait method will return at compile time.

The `async_trait` crate solves this by transforming async trait methods into regular methods that return a pinned boxed future. The `#[async_trait]` attribute macro handles the transformation:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
}
```

Without `async_trait`, this would need to be written much more verbosely:

```rust
// Without async_trait (for illustration only):
pub trait SiteScraper: Send + Sync {
    fn lookup<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<FicMetadata, ScrapeError>> + Send + 'a>>;
}
```

The `async_trait` version looks and feels like a regular async method, while the manual version requires explicit lifetime annotations, `Pin<Box<dyn Future>>` types, and `Send + 'a` bounds. The attribute macro hides all this complexity.

There's a small performance cost: every async trait method call allocates a `Box` on the heap. For our scraper system, this cost is negligible compared to the network I/O. But it's worth knowing about if you're building something performance-critical.

## Error Handling in Scrapers: The ScrapeError Type

Every scraper operation can fail, and we need to know *how* it failed. Our `ScrapeError` enum covers the common cases:

```rust
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,           // Story doesn't exist or was deleted
    Blocked,            // Site blocked our request
    Network(String),    // HTTP/connection error with message
    ParseError(String), // Couldn't parse the HTML with message
}
```

**Why four variants instead of one?**

Each variant represents a fundamentally different failure mode that the caller might want to handle differently:

- **NotFound**: The story was deleted or the URL is wrong. We should probably tell the user "this story doesn't exist" and not retry.
- **Blocked**: The site is rate-limiting us or blocking automated access. We should retry later, not immediately. Maybe after a 60-second delay.
- **Network**: Something went wrong with the HTTP request. Could be DNS failure, timeout, or connection refused. Maybe retry after a short delay, or maybe it's a permanent issue.
- **ParseError**: The HTML doesn't match our expectations. The site might have changed its layout. This needs developer attention, not a retry.

By distinguishing these cases, the caller can make smarter decisions about what to do next.

We also implement `Display` and `Error` for `ScrapeError` so it integrates with Rust's standard error handling:

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

The `Display` implementation gives us human-readable error messages. The `Error` implementation lets us use `?` to propagate `ScrapeError` in functions that return `Result<T, Box<dyn Error>>`.

**The `?` operator in action:**

When a scraper method returns an error, we use the `?` operator to propagate it up the call stack:

```rust
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

The `.map_err(...)` converts `reqwest::Error` into our `ScrapeError::Network`, and the `?` operator returns early from the function if there's an error. This keeps the error handling clean and consistent across all scrapers.

**Why `.map_err()` instead of `?` directly?**

We can't use `?` directly because `reqwest::Error` is a different type than `ScrapeError`. The `?` operator needs a conversion (via `From` trait) or we need to do it explicitly with `.map_err()`. Using `.map_err()` is explicit and makes it clear what type of error we're wrapping.

**Test coverage for error display:**

The tests verify that our error messages are correct:

```rust
#[test]
fn test_scrape_error_display_not_found() {
    assert_eq!(format!("{}", ScrapeError::NotFound), "fic not found");
}

#[test]
fn test_scrape_error_display_network() {
    let err = ScrapeError::Network("connection refused".into());
    assert_eq!(format!("{}", err), "network error: connection refused");
}
```

This might seem like overkill, but error messages are user-facing (they appear in API responses), so it's worth verifying they're correct.

## Putting It All Together

Here's the complete flow when a user asks FicHub to scrape a story:

1. **User submits a URL** through the API (e.g., `GET /api/v0/meta?url=https://archiveofourown.org/works/123456`)
2. **Registry finds the scraper** by calling `find_scraper(url)` which checks each scraper's `can_handle` method
3. **Scraper's `lookup`** extracts the work ID, fetches the page, parses the HTML, and returns metadata
4. **Scraper's `fetch_chapters`** downloads all chapter content (one request for AO3, N requests for FF.net)
5. **Scraper's `extract_tags`** (if available) pulls structured tags
6. **Server stores everything** in the PostgreSQL database
7. **EPUB is generated** from the stored data when the user requests a download

Each step is independent, testable, and replaceable. If AO3 changes its HTML, we only need to update the AO3 scraper. If we want to support a new site, we just add a new scraper that implements the trait.

The beauty of this architecture is that the rest of the system doesn't know or care which scraper was used. The API handler calls `state.scraper_registry.lookup(&client, &url)`, and the registry handles the routing. Whether the story is from AO3, FF.net, or XenForo, the response is the same `FicMetadata` struct.

🧪 **Try It Yourself:** Think about a fanfiction site that isn't currently supported. What would its scraper need? Write down: (1) the URL pattern for stories, (2) what metadata you'd want to extract, (3) a rough idea of what CSS selectors you'd use, and (4) what the `can_handle` method would check for. This mental exercise helps you understand the scraper architecture.

---

# Chapter 13: The Scraper Registry

## What Is a Registry?

Imagine you're at a hotel, and you need to reach a specific department. You call the front desk, and the receptionist says "Let me transfer you." They look up your request, find the right department, and connect you. That's exactly what our ScraperRegistry does—it's a phone book for scrapers.

When the server receives a URL like `https://archiveofourown.org/works/123456`, it doesn't know which scraper should handle it. The registry takes that URL, checks each scraper's `can_handle` method, and routes the request to the right one.

This is a classic **Strategy Pattern** in software design. We have a collection of strategies (scrapers), each capable of handling a specific type of input (URLs from different sites), and a registry that selects the appropriate strategy at runtime.

Let's look at the code.

## The ScraperRegistry Struct

The registry is defined in `scrape/registry.rs`:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

That's it. A vector of boxed trait objects. Each element is a scraper that implements `SiteScraper`. We use `Box<dyn SiteScraper>` (dynamic dispatch) instead of concrete types because the scrapers are different types (`Ao3Scraper`, `FfNetScraper`, `XenForoScraper`, etc.) and we need to store them all in the same collection.

**Why `Vec` instead of `HashMap`?**

You might think a `HashMap<String, Box<dyn SiteScraper>>` would be more efficient—mapping domain names to scrapers directly. But the problem is that some scrapers handle multiple domains (like FF.net handling both `fanfiction.net` and `fictionpress.com`), and the matching logic isn't always a simple domain check.

Consider: FF.net's `can_handle` checks `url.contains("fanfiction.net") || url.contains("fictionpress.com")`. If we used a HashMap keyed on domain, we'd need to register the same scraper twice—once for each domain. And for sites like XenForo where we maintain a list of domains, a HashMap wouldn't capture the full matching logic.

The `can_handle` method gives us the flexibility to implement any matching logic we want. Whether it's a simple domain check, a regex match on the URL path, or something more complex, the trait-based approach handles it.

**Why `Box<dyn SiteScraper>` (dynamic dispatch)?**

In Rust, you can't have a vector of different types directly. `Vec<Ao3Scraper>` can only hold `Ao3Scraper` values. But we want to store `Ao3Scraper`, `FfNetScraper`, `XenForoScraper`, and others in the same vector.

`Box<dyn SiteScraper>` solves this. The `dyn SiteScraper` part is a **trait object**—it says "any type that implements SiteScraper." The `Box` part puts it on the heap so it has a known size (trait objects are unsized by default).

The trade-off: dynamic dispatch adds a small overhead (one extra indirection) compared to static dispatch. For our use case, this overhead is completely negligible—we're doing HTTP requests that take hundreds of milliseconds, and the dispatch overhead is nanoseconds.

## Creating the Registry

The `new()` method registers all known scrapers:

```rust
impl ScraperRegistry {
    pub fn new() -> Self {
        let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
        scrapers.push(Box::new(sites::ao3::Ao3Scraper));
        scrapers.push(Box::new(sites::ffnet::FfNetScraper));
        scrapers.push(Box::new(sites::xenforo::XenForoScraper));
        scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
        scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
        scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
        ScraperRegistry { scrapers }
    }
}
```

Each scraper is a **unit struct** (a struct with no fields, like `struct Ao3Scraper;`). This is a design choice that makes sense because our scrapers don't carry any state—they're stateless functions that operate on the passed-in client and URL.

Each scraper is wrapped in `Box::new(...)` to heap-allocate it, then stored as a `Box<dyn SiteScraper>`. This is how Rust achieves runtime polymorphism—we have a collection of different types that all share the same interface.

**The order matters (a little):**

The scrapers are checked in order. The first scraper that returns `true` from `can_handle` wins. In practice, this rarely matters because each scraper handles a distinct set of domains. But if you had overlapping scrapers (say, two scrapers that both claim to handle AO3), the first one registered would be used.

Currently, the FictionPress scraper is a re-export of FfNetScraper:

```rust
// In sites/fictionpress.rs:
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

This means both the FF.net scraper and the FictionPress scraper will match on `fictionpress.com` URLs (since FfNetScraper's `can_handle` checks for both domains). The FF.net scraper is registered first, so it handles FictionPress URLs. This is fine—they use the exact same code.

**The Default implementation:**

```rust
impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

This is a Rust convention. Implementing `Default` lets you create an instance with `ScraperRegistry::default()` in addition to `ScraperRegistry::new()`. It's a small ergonomic improvement that makes the code more idiomatic.

## find_scraper: Matching a URL to the Right Scraper

The core method of the registry is `find_scraper`:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This is a simple linear search. It iterates through all registered scrapers, calls `can_handle` on each one, and returns the first match. If no scraper can handle the URL, it returns `None`.

Let's trace through what happens with an AO3 URL:
1. Check `Ao3Scraper.can_handle("https://archiveofourown.org/works/123456")` → `true` (URL contains "archiveofourown.org")
2. Return `Some(Ao3Scraper)` immediately

And for an unknown URL:
1. Check `Ao3Scraper.can_handle("https://wattpad.com/story/12345")` → `false`
2. Check `FfNetScraper.can_handle(...)` → `false`
3. Check `XenForoScraper.can_handle(...)` → `false`
4. Check `FictionPressScraper.can_handle(...)` → `false`
5. Check `AdultFanFictionScraper.can_handle(...)` → `false`
6. Check `HpFanFicScraper.can_handle(...)` → `false`
7. Return `None`

**Is linear search fast enough?**

With six scrapers, a linear search takes at most six comparisons. Each comparison is a string `contains` check, which is essentially a substring search. This is measured in nanoseconds—completely negligible compared to the hundreds of milliseconds a network request takes.

If we had hundreds of scrapers, we might want to optimize with a HashMap or a trie-based lookup. But for six scrapers, simplicity wins.

**Return type: `Option<&Box<dyn SiteScraper>>`**

The return type looks intimidating, but let's break it down:
- `Option<...>`: Might return `Some(value)` or `None`
- `&Box<dyn SiteScraper>`: A reference to a boxed trait object

We return a *reference* rather than an owned value because we don't want to move the scraper out of the vector. We're just borrowing it for the duration of the call.

## The Convenience Methods

The registry provides two convenience methods that combine the lookup with the scraper call:

```rust
pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    match self.find_scraper(url) {
        Some(scraper) => scraper.lookup(client, url).await,
        None => Err(ScrapeError::NotFound),
    }
}

pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    match self.find_scraper(&meta.source) {
        Some(scraper) => scraper.fetch_chapters(client, meta).await,
        None => Err(ScrapeError::NotFound),
    }
}
```

The `lookup` method finds the right scraper for a URL and calls its `lookup` method. If no scraper is found, it returns `ScrapeError::NotFound`.

The `fetch_chapters` method is a bit different: it uses the `source` field from the metadata (which is the original URL) to find the scraper, not the URL the caller originally provided. This is because `fetch_chapters` might be called later, after the metadata has been stored in the database. The `source` field preserves the original URL.

**Why not just pass the scraper directly?**

You might wonder: why have the registry look up the scraper when the caller could just call the scraper directly? The answer is **decoupling**. The route handler doesn't need to know which scraper handles which site. It just says "look up this URL" and the registry handles the rest. This means:

1. Route handlers are simpler—they don't need site-specific logic
2. Adding new sites doesn't require changing route handlers
3. The registry can apply middleware (logging, metrics, rate limiting) to all scraper calls

**A small helper:**

```rust
pub fn scraper_count(&self) -> usize {
    self.scrapers.len()
}
```

This returns the number of registered scrapers, which we log at startup:

```rust
tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
```

This is a simple but useful diagnostic. If you see "Registered 0 scrapers" at startup, something went wrong with the initialization.

## How the Server Uses the Registry

The registry is created once when the server starts and stored in the shared application state. Let's trace through the server initialization:

```rust
// In server.rs - the run() function
pub async fn run(config: Config) {
    // ... database and Redis connections ...

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // ... other initialization ...

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        // ... other fields
    });

    // Build router and start serving
    let app = build_router(state).await;
    // ...
}
```

The registry is wrapped in `Arc` (Atomic Reference Counted pointer) so it can be shared across multiple async tasks without cloning the data. The registry itself lives on the heap, and `Arc` provides shared ownership with thread-safe reference counting.

The `AppState` struct includes the registry:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

When a route handler needs to scrape a story, it accesses the registry through the state:

```rust
// In a route handler:
let metadata = state.scraper_registry.lookup(&state.http_client, &url).await?;
let chapters = state.scraper_registry.fetch_chapters(&state.http_client, &metadata).await?;
```

The route handler doesn't need to know whether the URL is from AO3, FF.net, or XenForo. The registry handles the routing transparently.

## The Arc<ScraperRegistry> Pattern

Let's take a moment to understand why we use `Arc`. In Rust, data can be owned by one owner at a time. But our registry needs to be accessed by multiple async tasks simultaneously—different HTTP handlers might be scraping different stories at the same time.

`Arc` solves this by providing shared ownership. Multiple `Arc` pointers can point to the same data, and the data is only freed when the last `Arc` is dropped. It's reference-counted garbage collection, but thread-safe.

```rust
// When the server starts:
let scraper_registry = Arc::new(ScraperRegistry::new());

// Arc is cloned when shared to other parts of the system:
let worker_registry = scraper_registry.clone();
// Now both scraper_registry and worker_registry point to the same data
// The reference count is now 2

// When we need to use it:
state.scraper_registry.lookup(&client, &url).await?;
```

The key insight: cloning an `Arc` is cheap—it just increments an atomic counter. The underlying data isn't copied. This is why `Arc` is the standard pattern for sharing expensive-to-create resources in async Rust.

**When the last Arc is dropped:**

When all `Arc` pointers to the registry are dropped (the server shuts down, for example), the reference count reaches zero and the registry is freed. This is automatic—you don't need to manually clean up the scrapers.

⚠️ **Watch Out:** `Arc` provides shared *read* access but not shared *write* access. If you needed mutable shared state (like adding a scraper at runtime), you'd use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`. Our registry is read-only after creation, so `Arc` alone is sufficient.

**Why not just pass the registry by value?**

You could theoretically pass the registry by value to each handler, but then you'd need to reconstruct it for each request. `Arc` lets us create the registry once and share it efficiently across all requests.

## The CollectionWorker Connection

The registry isn't just used by route handlers. The `CollectionWorker`—a background task that scrapes user favourites—also uses it:

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiterLimiter>,
}
```

The worker receives the same `Arc<ScraperRegistry>` that the server uses. This means both the HTTP handlers and the background worker share the same registry instance, with the same scrapers, and the same configuration. No duplication, no drift.

The `CollectionWorker` also maintains its own set of `SiteFetcher` objects and per-site rate limiters. These are separate from the `SiteScraper` trait—`SiteFetcher` is a different abstraction used for batch collection operations where rate limiting and progress tracking are more important.

## Adding a New Scraper: Step by Step

Let's walk through what it takes to add a new scraper to the registry. This is the most common maintenance task you'll do as a FicHub developer.

**Step 1: Create the scraper file**

Create a new file in `src/scrape/sites/`. Let's call it `newsite.rs`:

```rust
/// NewSite scraper
pub struct NewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl NewSiteScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for NewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("newsite.example.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;

        let response = client
            .get(url)
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

        let title = document
            .select(&Selector::parse("h1.title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("a.author-name").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let url_id = crate::scrape::generate_url_id(6, &story_id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 6,
            author_id: 0,
            author_url: String::new(),
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let response = client
            .get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content_sel = Selector::parse("div.story-content").unwrap();
        let mut chapters = Vec::new();

        for (i, div) in document.select(&content_sel).enumerate() {
            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: format!("Chapter {}", i + 1),
                content: div.inner_html(),
            });
        }

        Ok(chapters)
    }
}
```

Note the source ID `6`—each site gets a unique number. Our current assignments are: 1=AO3, 2=FF.net, 3=XenForo, 4=AdultFanFiction, 5=HPFanFic. The next available ID is 6.

**Step 2: Register the module**

In `src/scrape/sites/mod.rs`, add the module:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod newsite;  // Add this line
```

**Step 3: Add to the registry**

In `src/scrape/registry.rs`, push your new scraper into the vector:

```rust
pub fn new() -> Self {
    let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
    scrapers.push(Box::new(sites::ao3::Ao3Scraper));
    scrapers.push(Box::new(sites::ffnet::FfNetScraper));
    scrapers.push(Box::new(sites::xenforo::XenForoScraper));
    scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
    scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
    scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
    scrapers.push(Box::new(sites::newsite::NewSiteScraper));  // Add this line
    ScraperRegistry { scrapers }
}
```

**Step 4: Test it**

Run the server and try scraping a story from the new site. Check the logs for the "Registered 7 scrapers" message.

That's it. Three steps, and your new scraper is ready to go. The architecture is designed so that adding new sites requires minimal boilerplate.

## Testing the Registry

The registry is straightforward to test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_scrapers() {
        let registry = ScraperRegistry::new();
        assert_eq!(registry.scraper_count(), 6);
    }

    #[test]
    fn test_find_ao3_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://archiveofourown.org/works/123456";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_ffnet_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://www.fanfiction.net/s/1234567/1/";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_xenforo_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://forums.spacebattles.com/threads/story.12345/";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_unknown_site() {
        let registry = ScraperRegistry::new();
        let url = "https://wattpad.com/story/12345";
        assert!(registry.find_scraper(url).is_none());
    }
}
```

These tests verify that all scrapers are registered and that routing works correctly. The unknown-site test is particularly important—it confirms that we don't accidentally claim to handle sites we don't support.

🧪 **Try It Yourself:** Modify the `test_registry_has_scrapers` test to check for a different number. If you've added your own scraper (from the step-by-step guide above), update the expected count. What happens if you push the same scraper twice? Try it and observe.

---

# Chapter 14: AO3 Scraper (Deep Dive)

## The Most Important Scraper

If FicHub had to pick just one site to support, it would be AO3. It's the largest, most active fanfiction archive, it has the most structured data, and its community is the most engaged. Our AO3 scraper is the most fully-featured in the codebase, and it's a great template for understanding how scrapers work.

Let's go through it line by line. Every selector, every extraction, every fallback—explained.

## The AO3 Page Structure

Before we can extract data from AO3, we need to understand what the page looks like. AO3 uses a well-organized HTML structure with semantic elements and descriptive CSS classes. This is one of the reasons it's so popular among developers—it's a pleasure to scrape.

**The work page (view_full_work=true):**

When you visit `https://archiveofourown.org/works/123456?view_full_work=true`, you see the entire story on one page. The HTML structure looks roughly like this:

```html
<div id="workskin">
  <!-- Header section with metadata -->
  <div class="preface group">
    <div class="title heading">
      <h2 class="title heading">The Title of the Story</h2>
      <h3 class="byline heading">
        <a href="/users/authorname/pseuds/authorname" rel="author">AuthorName</a>
      </h3>
    </div>

    <!-- Stats section -->
    <div class="stats group">
      <dl>
        <dt>Published:</dt>
        <dd class="published">2023-01-15</dd>
        <dt>Status:</dt>
        <dd class="status">Complete</dd>
        <dt>Chapters:</dt>
        <dd class="chapters">5 / 5</dd>
        <dt>Words:</dt>
        <dd class="words">42,156</dd>
      </dl>
    </div>

    <!-- Summary -->
    <div class="summary module">
      <blockquote class="userstuff">
        <p>This is the summary of the story.</p>
      </blockquote>
    </div>
  </div>

  <!-- Chapters -->
  <div id="chapters">
    <div class="chapter" id="chapter-1">
      <h3 class="heading">
        <span class="chapter">1</span>
        <span class="chapter-title">The Beginning</span>
      </h3>
      <div class="userstuff module">
        <p>Chapter 1 content goes here...</p>
      </div>
    </div>

    <div class="chapter" id="chapter-2">
      <h3 class="heading">
        <span class="chapter">2</span>
        <span class="chapter-title">The Middle</span>
      </h3>
      <div class="userstuff module">
        <p>Chapter 2 content goes here...</p>
      </div>
    </div>
  </div>
</div>
```

Notice the pattern: AO3 wraps content in `div.userstuff` elements. The title is in `h2.title.heading`. The author is in `a[rel='author']`. These selectors are what our scraper targets.

## CSS Selectors That Find the Metadata

Let's walk through each piece of metadata extraction in the AO3 scraper.

**Title:**

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

The selector `h2.title.heading` matches an `<h2>` element that has both the `title` and `heading` classes. AO3 uses this specific combination for the story title. The `.text().collect::<String>()` call extracts all text nodes within the element (including text inside child elements), and `.trim()` removes leading/trailing whitespace.

Why `h2` and not `h1`? AO3 uses `h1` for the site navigation and `h2` for story titles. This is a quirk of AO3's HTML—you'd only know this by examining the page source.

**Author:**

```rust
let author = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The selector `a[rel='author']` is a CSS attribute selector. It matches any `<a>` element that has `rel="author"`. This is how AO3 marks author links—they use the `rel="author"` attribute to indicate that the link points to the author's profile.

This is a robust selector because `rel="author"` is semantically meaningful. AO3 is unlikely to remove it because it's used for accessibility and SEO purposes.

**Author URL:**

```rust
let author_url = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"))
    .map(|h| format!("{BASE_URL}{h}"))
    .unwrap_or_default();
```

This reuses the same selector as the author name, but instead of getting the text content, it gets the `href` attribute. AO3 uses relative URLs for author profiles (like `/users/authorname/pseuds/authorname`), so we prepend the base URL to make it absolute.

Note the `.and_then()` chain: it first gets the element, then gets the `href` attribute. If either step fails, the chain short-circuits to `None`.

**Description:**

```rust
let description = document
    .select(&Selector::parse("blockquote.userstuff").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

The summary lives in a `blockquote.userstuff` element. Note that we use `inner_html()` instead of `text()` here—we want to preserve any HTML formatting in the summary (like italic text, links, or paragraph breaks). This is important because AO3 summaries often contain rich formatting.

**Chapter count:**

```rust
let stats_text: String = document
    .select(&Selector::parse("dd.chapters").unwrap())
    .next()
    .map(|el| el.text().collect())
    .unwrap_or_default();

let chapters = if let Some(pos) = stats_text.find('/') {
    stats_text[..pos].trim().parse().unwrap_or(1)
} else {
    1
};
```

This is interesting. AO3's chapter count is displayed as "3 / 5" (3 chapters published out of 5 planned). We find the `/` character and take everything before it—the current chapter count. If there's no `/`, it's a one-shot story with a single chapter.

The `.parse().unwrap_or(1)` chain converts the string to an integer, defaulting to 1 if parsing fails. This handles edge cases like empty strings or unexpected formats.

**Word count:**

```rust
let words = document
    .select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

The word count is in a `dd.words` element. Note the `.replace(',', "")`—AO3 formats large numbers with commas (like "42,156"), and we need to remove them before parsing as an integer. The `.parse().ok()` converts the result to `Option<i64>`, which we default to 0 if parsing fails.

**Status:**

```rust
let status_text = document
    .select(&Selector::parse("dd.status").unwrap())
    .next()
    .map(|el| el.text().collect::<String>())
    .unwrap_or_default();
let status = if status_text.contains("Complete") {
    "complete".to_string()
} else {
    "ongoing".to_string()
};
```

AO3's status field says either "Complete" or "In Progress." We normalize these to our internal format: "complete" or "ongoing." If the status element isn't found, we default to "ongoing"—a safe assumption since most stories are in progress.

**Generating the URL ID:**

```rust
let url_id = crate::scrape::generate_url_id(1, &work_id);
```

The `1` is AO3's source ID. Combined with the work ID, this generates a deterministic 12-character hex hash. This hash is used as the story's identifier across the FicHub system.

## The lookup() Function

The `lookup` method ties all these extractions together:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
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

    // ... extract all metadata fields ...

    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters,
        words,
        desc: description,
        published: now,
        updated: now,
        status,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url,
        author_local_id: work_id.clone(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

The flow is:
1. Extract the work ID from the URL
2. Build the full URL with `?view_full_work=true`
3. Fetch the page
4. Check for errors
5. Parse the HTML
6. Extract each metadata field
7. Assemble and return the `FicMetadata` struct

**Note the `source` field:** We store the full URL with `?view_full_work=true` as the source. This means when `fetch_chapters` is called later, it can use this URL directly to get the full story in one request. We don't need to reconstruct the URL or add the parameter again.

**Note the `author_local_id` field:** We store the `work_id` as the `author_local_id`. This is a bit of a naming quirk—the field is used to store the site-specific identifier, which for AO3 is the work ID. We'll use this same field when fetching chapters.

## The fetch_chapters() Function

Once we have metadata, we need the actual chapter content:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();

    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }

    if chapters.is_empty() {
        // Fallback for one-shots: look for top-level content
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }

    Ok(chapters)
}
```

**The `?view_full_work=true` trick again:**

We use the same parameter to get all chapters on one page. This means for a 20-chapter story, we make 1 HTTP request instead of 20. Huge efficiency win. The alternative—fetching each chapter separately—would be 20x slower and much more likely to trigger rate limits.

**Iterating through chapters:**

The `div.chapter` selector finds all chapter containers on the page. For each one, we extract:
- The chapter title from `h3.title` (or fall back to "Chapter N")
- The chapter content from `div.userstuff` (using `inner_html()` to preserve formatting)

The `enumerate()` method gives us the index `i`, which we use for the chapter ID (`i + 1` since we want 1-indexed chapters) and as a fallback title.

**The fallback for one-shots:**

If no `div.chapter` elements are found (common for one-shot stories), the code looks for a top-level `div.userstuff` element. This handles stories with a single chapter that aren't wrapped in chapter divs.

The fallback uses `meta.title.clone()` as the chapter title—the story title serves as the chapter title for one-shots. This makes sense because there's only one chapter, so the title is the same as the story title.

⚠️ **Watch Out:** The fallback logic checks `if chapters.is_empty()` after the main loop. This means even multi-chapter stories where the selectors don't match would fall back to treating the entire page as one chapter. It's a safety net, not a perfect solution. If AO3 changes its HTML significantly, we might get incorrect chapter counts.

## extract_tags: Getting Structured Tags

AO3 has the richest tagging system of any fanfiction site. Our `ExtractedTag` struct represents each tag:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

The convenience methods (`fandom()`, `character()`, etc.) make it easy to construct tags with the right type ID. The type IDs map to database values: 1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category.

The `extract_tags` method is defined on the `SiteScraper` trait with a default implementation that returns an empty vector. AO3's scraper overrides this to extract tags from the page. The tag HTML on AO3 is organized by type, with each type in its own section:

```html
<div class="fandom tags">
  <h3 class="heading">Fandoms:</h3>
  <ul class="tags commas">
    <li><a href="/tags/Harry%20Potter">Harry Potter</a></li>
    <li><a href="/tags/Marvel">Marvel</a></li>
  </ul>
</div>

<div class="relationship tags">
  <h3 class="heading">Relationships:</h3>
  <ul class="tags commas">
    <li><a href="/tags/Harry%20Potter%2FDraco%20Malfoy">Harry/Draco</a></li>
  </ul>
</div>
```

Each tag type section uses the same HTML structure but different class names. The scraper targets each section with the appropriate selector and constructs `ExtractedTag` values with the correct type ID.

## Handling Different AO3 Page Formats

AO3 has a few page variants that our scraper needs to handle:

1. **Single chapter story**: No `div.chapter` elements, just a top-level `div.userstuff`. Handled by the fallback logic.
2. **Multi-chapter story**: Multiple `div.chapter` elements, each with content. The primary code path.
3. **Work with restricted chapters**: Some chapters might be marked as "restricted" (only visible to logged-in users). Our scraper gets an empty content div for these.
4. **Work with author notes**: Author notes appear before and after the story content, typically in `div.author` or `div.end` elements. Our current implementation doesn't extract these separately.

Our current implementation handles cases 1 and 2 well. Cases 3 and 4 are edge cases that might not work perfectly, but they don't crash—the scraper gracefully handles missing content.

**The `published` and `updated` fields:**

Notice that we set both `published` and `updated` to `Utc::now().timestamp_millis()`:

```rust
let now = Utc::now().timestamp_millis();

Ok(FicMetadata {
    // ...
    published: now,
    updated: now,
    // ...
})
```

This is a limitation. We're not currently extracting the actual publish and update dates from AO3's page. AO3 provides these in `dd.published` and `dd.updated` elements as date strings like "2023-01-15". A more complete scraper would parse these date strings into Unix timestamps using chrono's date parsing.

This is a common pattern in scraper development: start with the most important fields (title, author, chapters, words), and add refinements (exact dates, extended metadata) over time.

## Testing the AO3 Scraper

The AO3 scraper has some of the best tests in the codebase, particularly for the URL ID generation:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_different_story_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_456");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_generate_url_id_empty_story_id() {
    let id = generate_url_id(1, "");
    assert_eq!(id.len(), 12);
}
```

The deterministic test verifies that the same inputs always produce the same output (essential for database lookups). The different-source-id and different-story-id tests confirm that both inputs affect the output. The length and hex-char tests verify the output format.

The empty-story-id test is a boundary case—it confirms that even with an empty input, we get a valid 12-character hash. This is important for robustness.

🧪 **Try It Yourself:** Write a test for the `extract_work_id` method. Create test cases for these URLs:
- `https://archiveofourown.org/works/123456`
- `https://archiveofourown.org/works/123456/chapters/789012`
- `https://archiveofourown.org/works/123456?view_full_work=true`

What should the method return for each? (Hint: the `?view_full_work=true` parameter shouldn't affect the work ID extraction.)

---

# Chapter 15: Other Site Scrapers

## Beyond AO3: The Full Ecosystem

While AO3 is the most popular fanfiction site, it's far from the only one. FicHub supports six different scrapers, each tailored to a specific site's unique HTML structure. In this chapter, we'll tour the remaining scrapers and learn the patterns that make them work.

The key insight across all these scrapers is this: the **structure** of the scraper is always the same (extract ID, fetch page, parse HTML, extract metadata, fetch chapters), but the **specifics** change for each site. Different selectors, different URL patterns, different quirks.

## FF.net Scraper: Different HTML, Same Goal

FanFiction.net uses a completely different HTML structure than AO3. Where AO3 uses clean semantic HTML with descriptive classes, FF.net uses IDs and custom classes that are more about styling than meaning.

**Extracting the story ID:**

FF.net URLs follow the pattern `/s/NUMBER/CHAPTER/`. Our scraper extracts the story ID:

```rust
fn extract_story_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

Simple and clean. The `/s/` prefix is FF.net's convention for "story."

**FF.net metadata extraction:**

The metadata section on FF.net uses a `#profile_top` container. Unlike AO3's descriptive class names, FF.net uses the same class (`xcontrast_txt`) for multiple element types, distinguished by the element itself:

```rust
let title = document
    .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

let description = document
    .select(&Selector::parse("#profile_top div.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_default();
```

Notice the selectors:
- `b.xcontrast_txt` — the title is a `<b>` (bold) element
- `a.xcontrast_txt` — the author is an `<a>` (link) element
- `div.xcontrast_txt` — the description is a `<div>` element

All share the `xcontrast_txt` class, but the element type tells us what each one represents.

**Word count and chapter count via data attributes:**

FF.net uses `data-` attributes to store specific metadata values:

```rust
let words = document
    .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);

let chapters = document
    .select(&Selector::parse("#profile_top span[data-xutitle='chapters']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().trim().split('/').next()
            .and_then(|s| s.trim().parse().ok())
    })
    .unwrap_or(1);
```

The `data-xutitle` attribute is FF.net's custom way of marking specific data fields. The `xu` prefix is likely an internal FF.net convention. Our selectors target these attributes precisely with `[data-xutitle='word count']` and `[data-xutitle='chapters']`.

The chapter count uses the same "X / Y" format as AO3 (current / planned), so we apply the same parsing logic: split on `/`, take the first part.

**One request per chapter:**

Unlike AO3, FF.net doesn't support viewing the full work on one page. The scraper iterates through each chapter:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;

    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let chapter_sel = Selector::parse("div.storytext").unwrap();
        let content = document
            .select(&chapter_sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));

        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }

    Ok(chapters)
}
```

The chapter content selector `div.storytext` targets FF.net's story content div. The chapter title selector `select#chap_select option[selected]` is particularly clever—it extracts the title from the currently selected option in the chapter dropdown menu, which shows the chapter name.

⚠️ **Watch Out:** FF.net's rate limits are stricter than AO3's. A 50-chapter story requires 50 HTTP requests. If you're scraping multiple stories, you need to space out requests to avoid getting blocked. Our `PerSiteRateLimiter` handles this, but it's worth understanding the constraint.

## XenForo Scraper: Forum Posts as Chapters

XenForo forums are the most different scraper in our collection. They're not purpose-built for fanfiction, so we have to adapt. Stories are posted as forum threads, and each "chapter" is a forum post.

**The domain list:**

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

These three forums are the most popular XenForo-based sites for fanfiction. SpaceBattles focuses on sci-fi and action, SufficientVelocity is more general, and QuestionableQuesting allows mature content.

**Thread ID extraction:**

XenForo URLs follow a pattern: `threads/slug.12345/`. The thread ID is the number after the last dot:

```rust
fn extract_thread_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The `.*` in the regex matches the slug (human-readable title), and `\.(\d+)` captures the numeric thread ID. The `.*` is greedy by default, so it matches as many characters as possible, then backtracks to find the dot and digits.

**Forum posts as chapters:**

The most interesting part of the XenForo scraper is how it treats forum posts as chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let url = &meta.source;
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let article_sel = Selector::parse("article.message-body").unwrap();
    let mut chapter_idx = 0;

    for article in document.select(&article_sel) {
        chapter_idx += 1;
        let content = article.inner_html();
        let title = if chapter_idx == 1 {
            meta.title.clone()
        } else {
            format!("Chapter {chapter_idx}: Thread Page {chapter_idx}")
        };

        chapters.push(Chapter {
            chapter_id: chapter_idx,
            title,
            content,
        });
    }

    if chapters.is_empty() {
        return Err(ScrapeError::ParseError("no content found in XenForo thread".into()));
    }

    Ok(chapters)
}
```

Each `article.message-body` element is a forum post. The first post is treated as Chapter 1 (using the thread title), and subsequent posts get numbered chapters like "Chapter 2: Thread Page 2."

The first post is special because in XenForo, the OP (original poster) usually writes the story introduction and Chapter 1 in the first post. Subsequent posts by the same author are typically additional chapters.

**The pagination challenge:**

XenForo threads can span multiple pages. A thread with 50 posts might be spread across 5 pages (10 posts per page). Our current scraper only fetches the first page, which means it only gets the posts on that page.

This is a known limitation. To handle pagination properly, the scraper would need to:
1. Parse the page navigation to find the total page count
2. Fetch each page sequentially (with rate limiting)
3. Combine posts from all pages
4. Handle cases where non-story posts (replies from other users) are mixed in

For now, the scraper works well for threads that fit on a single page, and provides a reasonable approximation for longer threads.

**Author URL construction:**

XenForo uses relative URLs for member profiles:

```rust
let author_url = author_el
    .and_then(|el| el.value().attr("href"))
    .map(|h| {
        if h.starts_with('/') {
            for domain in XENFORO_DOMAINS {
                if url.contains(domain) {
                    return format!("https://{domain}{h}");
                }
            }
        }
        h.to_string()
    })
    .unwrap_or_default();
```

This is more complex than AO3's author URL extraction. XenForo gives us a relative path like `/members/username.12345/`, and we need to figure out which domain it belongs to by checking the original URL. We iterate through our known domains and match against the URL.

## FictionPress: The Clone

FictionPress is the simplest scraper because it doesn't exist as separate code:

```rust
// In sites/fictionpress.rs:
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

That's the entire file. FictionPress uses the exact same codebase as FF.net, so we just re-export the FF.net scraper under a different name. The `can_handle` method already checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

This is a great example of the DRY (Don't Repeat Yourself) principle. When two things are truly identical, don't duplicate—re-export. It also means any bug fix to the FF.net scraper automatically applies to FictionPress.

## AdultFanFiction Scraper

AdultFanFiction.org is a smaller archive with a simpler HTML structure:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("adult-fanfiction.org") || url.contains("adultfanfiction.net")
}
```

It checks for two domain variants. The scraper uses straightforward selectors:

```rust
let title = document
    .select(&Selector::parse("h1.story_title").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("a.author").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

let description = document
    .select(&Selector::parse("div.story_description").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_default();
```

The story ID extraction is a bit different—it finds the first numeric segment in the URL path:

```rust
let story_id = url.split('/').filter_map(|s| s.parse::<i64>().ok()).next()
    .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
```

This is less precise than the regex-based approaches used by other scrapers, but it works for AdultFanFiction's URL structure. It splits the URL by `/`, tries to parse each segment as a number, and takes the first one that succeeds.

The chapter content uses `div.story_content`:

```rust
let content_sel = Selector::parse("div.story_content").unwrap();
let mut chapters = Vec::new();

for (i, content_div) in document.select(&content_sel).enumerate() {
    chapters.push(Chapter {
        chapter_id: (i + 1) as i32,
        title: format!("Chapter {}", i + 1),
        content: content_div.inner_html(),
    });
}
```

This is the simplest chapter extraction of all our scrapers—it just finds all `div.story_content` elements and treats each as a chapter.

## HPFanFic Scraper

The Harry Potter Fan Fiction Archive (hpfanficarchive.com / fanficauthors.net) is a niche site for HP-specific fanfiction:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("hpfanficarchive.com") || url.contains("fanficauthors.net")
}
```

This scraper has the most generic selectors, reflecting the site's simpler and less standardized HTML structure:

```rust
let title = document
    .select(&Selector::parse("h1").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("a[href*='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The `a[href*='author']` selector uses a substring match on the `href` attribute—any link whose URL contains "author" is treated as the author link. This is more flexible but also more fragile. If a non-author link happens to contain "author" in its URL, it would be selected instead.

For content, it tries multiple selectors with a comma-separated fallback:

```rust
let content_sel = Selector::parse("div.story-content, div.fic-content, article").unwrap();
```

This comma-separated selector tries three different selectors in order. If the site uses `div.story-content`, great. If it uses `div.fic-content`, that works too. If neither exists, it falls back to `article`. This defensive approach helps when a site's exact HTML structure isn't perfectly known.

For the ID, it uses the last segment of the URL:

```rust
let id = format!("hpfanfic_{}", url.split('/').last().unwrap_or("unknown"));
```

This is a different approach from the other scrapers. Instead of a numeric ID, it uses a string-based ID prefixed with the site name. The `generate_url_id` function still hashes this into a 12-character hex string.

## The Challenge of Site Changes

Here's the uncomfortable truth about web scraping: **sites change their HTML**. When AO3 redesigns its page layout, our selectors break. When FF.net updates its CSS classes, our scrapers can't find the elements they're looking for.

**How do we detect when selectors break?**

If a selector returns no results, the scraper falls back to default values:
```rust
.unwrap_or_else(|| "Unknown Title".to_string())
```

So instead of crashing, the scraper returns "Unknown Title" and an empty description. The story is still saved, but with degraded metadata. This is a graceful degradation strategy—we prefer incomplete data over no data.

**How do we know the defaults are being used?**

If you're logging scrape results, you can monitor for stories with "Unknown Title" or zero word counts. These are signals that the selectors might be broken.

**How do we fix broken selectors?**

The process is:
1. Detect that metadata is missing (title is "Unknown", chapters is 1, words is 0)
2. Visit the site manually in a browser
3. Open the browser's developer tools (F12)
4. Use the Elements tab to find where the data now lives
5. Copy the new CSS selector
6. Update the scraper code
7. Deploy the fix
8. Optionally re-scrape affected stories

This is why the `SiteScraper` trait is so valuable. When a site changes, we only need to update one scraper file. The rest of the system—database, API, EPUB generation—is unaffected. The trait boundary isolates the impact of changes.

**Preventive measures:**

Some things we can do to make scrapers more resilient:
- Use multiple fallback selectors (like HPFanFic's comma-separated selector)
- Check for data quality after scraping (is the title empty? are words count 0?)
- Log warnings when default values are used
- Monitor scrape success rates over time
- Write tests that verify selector strings are valid CSS

⚠️ **Watch Out:** If a site changes its HTML and your scraper silently returns default values, you might not notice the problem until users complain about missing data. Consider adding metrics or alerts for scrape failures. A simple "percentage of stories with non-default titles" metric can catch selector breakage early.

## Adding a New Scraper: Step by Step

Let's put everything together with a concrete walkthrough. Suppose you want to add a scraper for a new fanfiction site.

**Step 1: Research the site**

Visit the site and examine:
- URL structure: What does a story URL look like?
- Story ID: How are stories identified in the URL?
- Page structure: Where is the title, author, summary, chapters?
- Content structure: How is the story content laid out?

Use your browser's developer tools to inspect the HTML. Look for patterns: consistent class names, predictable element hierarchies, data attributes.

**Step 2: Create the scraper file**

Create a new file in `src/scrape/sites/`. Use the template we showed in Chapter 13:

```rust
pub struct NewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl NewSiteScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for NewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("newsite.example.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Extract ID, fetch page, parse HTML, extract metadata
        todo!()
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Fetch chapters, parse content
        todo!()
    }
}
```

**Step 3: Register the module and registry**

In `src/scrape/sites/mod.rs`:
```rust
pub mod newsite;
```

In `src/scrape/registry.rs`:
```rust
scrapers.push(Box::new(sites::newsite::NewSiteScraper));
```

**Step 4: Test it**

Run the server and try scraping a story from the new site. Check:
- Does `can_handle` return true for the right URLs?
- Does `lookup` return correct metadata?
- Does `fetch_chapters` return the content?

**Step 5: Handle edge cases**

Consider:
- What if the story doesn't exist? (Return `ScrapeError::NotFound`)
- What if the site blocks us? (Return `ScrapeError::Blocked`)
- What if the HTML is different than expected? (Return `ScrapeError::ParseError`)
- What if the story has 0 chapters? (Handle gracefully)

## Comparison: All Scrapers Side by Side

| Feature | AO3 | FF.net | XenForo | AdultFF | HPFanFic |
|---------|-----|--------|---------|---------|----------|
| ID Extraction | Regex `\d+` | Regex `\d+` | Regex after dot | Numeric segment | URL slug |
| Requests per story | 1 | N (one per chapter) | 1 (first page) | 1 | 1 |
| Tag extraction | Yes | Basic | No | No | No |
| Chapter titles | From page | From dropdown | Post number | Numbered | Numbered |
| Content selector | `div.userstuff` | `div.storytext` | `article.message-body` | `div.story_content` | Multiple fallbacks |
| Source ID | 1 | 2 | 3 | 4 | 5 |

Notice the trade-offs: AO3 has the best data quality (rich tags, chapter titles, full work view) but requires careful selector maintenance. FF.net has good data quality but requires multiple requests. XenForo has the worst data quality (no tags, limited chapter info) but is the simplest to implement.

## Wrapping Up the Scraper System

Let's zoom out and look at the full picture of what we've built across these five chapters:

1. **The `SiteScraper` trait** defines the contract every scraper must follow
2. **Individual scrapers** implement the trait for their specific site
3. **The `ScraperRegistry`** routes URLs to the right scraper
4. **The `FicMetadata` struct** standardizes what we extract from every site
5. **The `ScrapeError` enum** handles failures gracefully
6. **The `ExtractedTag` struct** lets scrapers provide structured tags
7. **The `generate_url_id` function** creates deterministic IDs across sites
8. **The `Chapter` struct** holds per-chapter content

This architecture is extensible. Adding a new site requires three things: a new scraper file, a module declaration, and a registry entry. The rest of the system—database, API, EPUB export—works automatically.

The scrapers are stateless (unit structs with no fields), which makes them easy to test and reason about. They receive the HTTP client as a parameter, which allows for connection pooling and shared configuration. They return standardized types (`FicMetadata`, `Vec<Chapter>`), which makes the rest of the system site-agnostic.

In the next part of the book, we'll use this scraper system to build the API endpoints that let users submit URLs and get back beautiful EPUB files. The scrapers are the foundation; the API is the house we build on top of it.

🧪 **Try It Yourself:** Take everything you've learned in Part 3 and imagine building a scraper for Wattpad. Write down: (1) the regex for extracting the story ID, (2) the CSS selectors for title, author, and content, (3) how you'd handle chapters (one-shot vs multi-chapter), and (4) what source ID you'd assign. You now have all the knowledge you need to build a real scraper.

---

*In Part 4, we'll explore the API layer that ties everything together.*
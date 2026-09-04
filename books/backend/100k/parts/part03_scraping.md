# Part 3: The Scraping System

---

# Chapter 11: Fanfiction Sites

## The Landscape of Fanfiction

Fanfiction is a vast, vibrant community where millions of writers create stories based on existing universes — Harry Potter, Marvel, Star Wars, anime, video games, and thousands more. These stories are published on dedicated platforms, each with its own community, culture, and technical infrastructure.

Understanding these platforms is the first step in building FicHub's scraping system. Each site has unique URL patterns, HTML structures, rate limiting policies, and quirks that our scrapers must handle.

## Archive of Our Own (AO3)

AO3 is the largest and most popular fanfiction archive, operated by the Organization for Transformative Works (OTW), a nonprofit organization. It hosts millions of stories across hundreds of thousands of fandoms.

**URL Patterns:**
- Work page: `https://archiveofourown.org/works/12345678`
- Chapter page: `https://archiveofourown.org/works/12345678/chapters/87654321`
- Full work: `https://archiveofourown.org/works/12345678?view_full_work=true`

The work ID is a numeric identifier. The `?view_full_work=true` parameter is crucial — it tells AO3 to render all chapters on a single page, which dramatically simplifies scraping.

**HTML Structure:**

AO3 uses clean, semantic HTML with consistent class names:

```html
<div id="workskin">
  <div class="preface group">
    <h2 class="title heading">
      My Amazing Fanfiction
      <span class="chapter" title="Chapters: 10">Chapters: 10/10</span>
    </h2>
    <h3 class="byline heading">
      <a href="/users/SomeAuthor" rel="author">SomeAuthor</a>
    </h3>
    <div class="summary module">
      <blockquote class="userstuff">
        <p>A story about things happening...</p>
      </blockquote>
    </div>
    <dl class="stats">
      <dt>Words:</dt><dd class="words">125,000</dd>
      <dt>Chapters:</dt><dd class="chapters">10/10</dd>
      <dt>Status:</dt><dd class="status">Complete</dd>
      <dt>Published:</dt><dd class="published">2023-01-15</dd>
      <dt>Updated:</dt><dd class="updated">2023-06-20</dd>
    </dl>
  </div>
  <div id="chapters">
    <div class="chapter" id="chapter-1">
      <h3 class="heading">
        <a href="/works/12345678/chapters/11111111">Chapter 1</a>
      </h3>
      <div class="userstuff">
        <p>Once upon a time in a land far away...</p>
      </div>
    </div>
    <div class="chapter" id="chapter-2">
      <h3 class="heading">
        <a href="/works/12345678/chapters/22222222">Chapter 2</a>
      </h3>
      <div class="userstuff">
        <p>The adventure continued as they...</p>
      </div>
    </div>
  </div>
</div>
```

**Key Selectors:**
- Title: `h2.title.heading`
- Author: `a[rel='author']`
- Description: `blockquote.userstuff`
- Word count: `dd.words`
- Chapter count: `dd.chapters` (format: "3/10")
- Status: `dd.status` ("Complete" or "In Progress")
- Chapter content: `div.chapter` > `div.userstuff`
- Tags: `ul.tags li.fandom a.tag`, `ul.tags li.character a.tag`, etc.

**Rate Limiting:** AO3 is strict about rate limiting. FicHub identifies itself with a User-Agent header and includes reasonable delays. Excessive requests result in temporary blocks or CAPTCHA challenges.

**Content Warnings:** AO3 uses a warnings system (e.g., "No Archive Warnings Apply", "Graphic Depictions Of Violence"). These appear in the metadata and can be extracted as tags.

**Series and Collections:** Stories can be part of series or collections. FicHub currently doesn't scrape these, but they could be added in the future.

## FanFiction.net (FF.net)

FF.net is one of the oldest fanfiction sites, launched in 1998. It has a more complex HTML structure with many CSS classes and data attributes.

**URL Patterns:**
- Story page: `https://www.fanfiction.net/s/12345678/1/` (story ID / chapter number)
- Chapter page: `https://www.fanfiction.net/s/12345678/3/` (chapter 3)

The story ID and chapter number are both in the URL path. Unlike AO3, FF.net doesn't have a "view full work" option, so each chapter must be fetched individually.

**HTML Structure:**

FF.net uses a more complex structure with data attributes:

```html
<div id="profile_top">
  <b class="xcontrast_txt">My Amazing Fanfiction</b>
  <a class="xcontrast_txt" href="/u/123456/SomeAuthor">SomeAuthor</a>
  <div class="xcontrast_txt">A story about things happening...</div>
  <span class="xgray">
    Rated: T | English | Humor/Romance | Chapters: 10 | Words: 125,000
  </span>
  <span data-xutitle="word count">125,000</span>
  <span data-xutitle="chapters">10/10</span>
</div>
<div class="storytext" id="storytext">
  <p>Once upon a time in a land far away...</p>
</div>
```

**Key Selectors:**
- Title: `#profile_top b.xcontrast_txt`
- Author: `#profile_top a.xcontrast_txt`
- Description: `#profile_top div.xcontrast_txt`
- Word count: `#profile_top span[data-xutitle='word count']`
- Chapter count: `#profile_top span[data-xutitle='chapters']`
- Chapter content: `div.storytext`
- Chapter select: `select#chap_select option[selected]`

**Fandom Categories:** FF.net uses a category system (Anime, Books, Cartoons, etc.) with subcategories (Harry Potter, Lord of the Rigs, etc.). These appear in the page metadata.

**Favorites and Follows:** FF.net tracks favorites and follows, but these aren't exposed in the same way as AO3's kudos.

**Challenges:** FF.net has been around since 1998 and its HTML reflects that legacy. Selectors can be fragile, and the site occasionally makes changes that break scrapers.

## XenForo Forums

XenForo is forum software used by several fanfiction communities. The most notable are:
- SpaceBattles (`forums.spacebattles.com`)
- SufficientVelocity (`forums.sufficientvelocity.com`)
- QuestionableQuesting (`forum.questionablequesting.com`)

**URL Pattern:** `https://forums.spacebattles.com/threads/story-name.12345/`

Stories are posted as forum threads. Each post can be a chapter, a comment, or off-topic discussion.

**HTML Structure:**

```html
<h1 class="p-title-value">Story Title</h1>
<div class="message-userContent">
  <a class="username" href="/members/author.12345/">AuthorName</a>
</div>
<article class="message-body">
  <div class="bbWrapper">
    <p>Once upon a time...</p>
  </div>
</article>
```

**Key Selectors:**
- Thread title: `h1.p-title-value`
- Author: `a.username`
- Post content: `article.message-body`
- Post content inner: `div.bbWrapper`

**Challenges:**
1. **Not all posts are story content.** Posts can be comments, discussions, or off-topic. A production scraper needs to filter these.
2. **Pagination.** Long threads span multiple pages. FicHub currently only scrapes the first page.
3. **User formatting.** Forum posts can contain embedded images, videos, spoilers, and custom BBCode that doesn't translate cleanly to HTML.
4. **Thread prefixes.** Threads often have prefixes like "[Complete]" or "[WIP]" that indicate story status.

## FictionPress

FictionPress is owned by the same company as FF.net and uses an identical HTML structure. FicHub handles this by reusing the FF.net scraper:

```rust
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

This is a clean example of code reuse through Rust's type system.

## AdultFanFiction.org (AFF)

AFF is a smaller site for adult-rated fanfiction. It has a simpler HTML structure:

**URL Pattern:** `https://www.adult-fanfiction.org/story/12345`

**Key Selectors:**
- Title: `h1.story_title`
- Author: `a.author`
- Description: `div.story_description`
- Content: `div.story_content`

AFF requires age verification, but the story pages themselves are accessible without authentication.

## HP Fan Fiction Archive

This covers two related sites:
- hpfanficarchive.com
- fanficauthors.net

Both are Harry Potter-specific archives with similar structures:

**Key Selectors:**
- Title: `h1`
- Author: `a[href*='author']`
- Content: `div.story-content, div.fic-content, article`

The flexible content selector tries multiple possible structures, making the scraper more resilient to layout changes.

## Site Comparison

| Feature | AO3 | FF.net | XenForo | AFF |
|---------|-----|--------|---------|-----|
| Full work view | Yes | No | No | No |
| Chapter selection | URL param | URL path | N/A | N/A |
| Tag system | Rich | Categories | Thread prefix | Basic |
| Rate limiting | Strict | Moderate | Varies | Lenient |
| HTML quality | Good | Legacy | Forum | Simple |
| Authentication | Optional | None | None | Age gate |

## Anti-Scraping Considerations

Each site has different anti-scraping measures:

- **AO3:** Rate limiting, CAPTCHA after excessive requests
- **FF.net:** Aggressive bot detection, IP blocking
- **XenForo:** Varies by installation, some use Cloudflare
- **AFF:** Age verification gate
- **HP FanFic:** Minimal protections

FicHub's approach is respectful: we identify ourselves with a User-Agent header, include reasonable delays, and don't overload the sites. This is both ethical and practical — aggressive scraping leads to blocks.

## Summary

Each fanfiction site has its own personality, technical infrastructure, and challenges. FicHub handles this diversity with a pluggable scraper system where each site has its own implementation. In the next chapter, we'll see how these scrapers are built using the `reqwest` and `scraper` crates.

---

# Chapter 12: Building Scrapers

## The HTTP Client

Every scraper needs to make HTTP requests. FicHub uses `reqwest`, the most popular Rust HTTP client:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

The `user_agent` is crucial — many sites block requests without a recognized User-Agent. FicHub identifies itself as "fichub.net/0.1.0" so site administrators can contact us if there are issues.

The timeout prevents requests from hanging indefinitely if a site is slow or unresponsive.

## The HTML Parser

FicHub uses the `scraper` crate for parsing HTML:

```rust
use scraper::{Html, Selector};

let html = response.text().await?;
let document = Html::parse_document(&html);

// Parse a CSS selector
let selector = Selector::parse("h2.title.heading").unwrap();

// Find matching elements
for element in document.select(&selector) {
    let text: String = element.text().collect();
    println!("Found: {}", text);
}
```

The `scraper` crate is built on top of `html5ever`, which is the same HTML parser used by Firefox. It handles malformed HTML gracefully.

## The SiteScraper Trait

Every scraper implements this trait:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;

    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>;

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
        -> Result<Vec<Chapter>, ScrapeError>;

    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str)
        -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

### can_handle

This method determines if a scraper can handle a given URL:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

It's simple string matching. For more complex patterns, you could use regex:

```rust
fn can_handle(&self, url: &str) -> bool {
    regex_lite::Regex::new(r"https?://(www\.)?fanfiction\.net/s/\d+").unwrap().is_match(url)
}
```

### lookup

This method fetches metadata without downloading chapter content:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    // 1. Fetch the page
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    
    // 2. Parse HTML
    let document = Html::parse_document(&html);
    
    // 3. Extract data
    let title = extract_text(&document, "h2.title.heading");
    let author = extract_text(&document, "a[rel='author']");
    // ... more fields ...
    
    // 4. Build result
    Ok(FicMetadata { title, author, /* ... */ })
}
```

### fetch_chapters

This method downloads all chapter content:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
    let mut chapters = Vec::new();
    
    for i in 1..=meta.chapters {
        let url = format!("{}/s/{}/{}", BASE_URL, meta.author_local_id, i);
        let response = client.get(&url).send().await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        let content = extract_html(&document, "div.storytext");
        chapters.push(Chapter {
            chapter_id: i,
            title: format!("Chapter {}", i),
            content,
        });
    }
    
    Ok(chapters)
}
```

### extract_tags

This optional method extracts structured tags:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str)
    -> Result<Vec<ExtractedTag>, ScrapeError>
{
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    let mut tags = Vec::new();
    
    // Extract fandom tags
    for el in document.select(&Selector::parse("ul.tags li.fandom a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }
    
    // ... more tag types ...
    
    Ok(tags)
}
```

## Helper Functions

Let's define some helper functions that all scrapers can use:

```rust
/// Extract text content from the first matching element
fn extract_text(document: &Html, selector: &str) -> String {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default()
}

/// Extract HTML content from the first matching element
fn extract_html(document: &Html, selector: &str) -> String {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default()
}

/// Extract an attribute from the first matching element
fn extract_attr(document: &Html, selector: &str, attr: &str) -> Option<String> {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.to_string())
}

/// Extract text with a fallback default
fn extract_text_or(document: &Html, selector: &str, default: &str) -> String {
    let text = extract_text(document, selector);
    if text.is_empty() {
        default.to_string()
    } else {
        text
    }
}
```

## URL ID Generation

Every fic needs a unique, deterministic ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 6 bytes = 12 hex chars
}
```

The ID is derived from the source site's story ID, ensuring the same story always gets the same ID regardless of when it was first seen.

## The FicMetadata Struct

This is the universal representation of scraped metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,           // Deterministic ID (12 hex chars)
    pub title: String,            // Story title
    pub author: String,           // Author name
    pub chapters: i32,            // Number of chapters
    pub words: i64,               // Word count
    pub desc: String,             // Description/summary (HTML)
    pub published: i64,           // Publication date (unix millis)
    pub updated: i64,             // Last update date (unix millis)
    pub status: String,           // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,           // Original URL
    pub source_id: i64,           // Site identifier (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Author identifier (site-specific)
    pub author_url: String,       // Author's profile URL
    pub author_local_id: String,  // Site-specific story/author ID
    pub content_hash: Option<String>,  // Hash of story content (for cache invalidation)
    pub extra_meta: Option<String>,    // Additional metadata (JSON)
    pub raw_extended_meta: Option<String>,  // Raw extended metadata
}
```

The `content_hash` is particularly important — it changes when the story's content changes, which triggers cache invalidation.

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,  // HTML content
}
```

The `content` field contains HTML, not plain text. This is crucial because the EPUB generator needs HTML for proper formatting.

## Error Handling

Scrapers can fail in several ways:

```rust
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}

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
```

- **NotFound** — The story doesn't exist or has been deleted
- **Blocked** — The site is blocking our requests
- **Network** — A network error occurred (timeout, connection refused, etc.)
- **ParseError** — The HTML structure changed and we couldn't extract data

## The ExtractedTag Struct

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

These type IDs match the `tag_types` table in the database.

## Building a Complete Scraper: AO3

Let's build the AO3 scraper from scratch, step by step.

### Step 1: Define the struct

```rust
pub struct Ao3Scraper;
```

It's a unit struct — no fields needed.

### Step 2: Implement can_handle

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

### Step 3: Extract the work ID

```rust
fn extract_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

### Step 4: Implement lookup

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;

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

    let title = extract_text_or(&document, "h2.title.heading", "Unknown Title");
    let author = extract_text_or(&document, "a[rel='author']", "Unknown Author");
    let author_url = extract_attr(&document, "a[rel='author']", "href")
        .map(|h| format!("{BASE_URL}{h}"))
        .unwrap_or_default();
    let description = extract_html(&document, "blockquote.userstuff");

    // Extract word count (remove commas)
    let words_text = extract_text(&document, "dd.words");
    let words = words_text.replace(',', "").parse().unwrap_or(0);

    // Extract chapter count
    let chapters_text = extract_text(&document, "dd.chapters");
    let chapters = if let Some(pos) = chapters_text.find('/') {
        chapters_text[..pos].trim().parse().unwrap_or(1)
    } else {
        1
    };

    // Extract status
    let status_text = extract_text(&document, "dd.status");
    let status = if status_text.contains("Complete") {
        "complete".to_string()
    } else {
        "ongoing".to_string()
    };

    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();

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
        author_local_id: work_id,
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

### Step 5: Implement fetch_chapters

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

    // Handle single-chapter works
    if chapters.is_empty() {
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

## Watch Out!

**Selector fragility!** CSS selectors break when sites change their HTML. Use the most stable selectors possible. IDs are better than classes. Semantic elements are better than divs.

**Don't trust the HTML!** Fanfiction sites can have malformed HTML. The `scraper` crate is tolerant of this, but always handle missing elements gracefully.

**Respect robots.txt!** Check the site's robots.txt before scraping. FicHub identifies itself and requests reasonable rates.

**User-Agent matters!** Many sites block requests without a recognized User-Agent. Always set one.

## Summary

In this chapter, we learned:
- How `reqwest` and `scraper` work together for HTTP + HTML parsing
- The `SiteScraper` trait and its four methods
- How to extract text, HTML, and attributes with CSS selectors
- The `FicMetadata` and `Chapter` structs
- Error handling with `ScrapeError`
- How to build a complete scraper from scratch
- Helper functions for common extraction patterns

---

# Chapter 13: Scraper Registry

## The Registry Pattern

FicHub supports multiple fanfiction sites, each with its own scraper. The **ScraperRegistry** is a central lookup table that maps URLs to the appropriate scraper.

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

The registry holds a vector of boxed trait objects. Each scraper is boxed because they're different concrete types that all implement the same trait.

## Creating the Registry

The registry is created with all known scrapers:

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

The `Default` trait is also implemented:

```rust
impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

## Finding a Scraper

The `find_scraper` method iterates through all scrapers:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This returns a reference to the first scraper that can handle the URL.

## Convenience Methods

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

pub fn scraper_count(&self) -> usize {
    self.scrapers.len()
}
```

## Sharing the Registry

The registry is wrapped in `Arc` and shared across all handlers:

```rust
let scraper_registry = Arc::new(ScraperRegistry::new());
```

`Arc` (Atomic Reference Counting) allows multiple handlers to hold a reference to the same registry without cloning it.

## Adding a New Scraper

To add support for a new site:

1. Create `src/scrape/sites/mynewsite.rs`
2. Implement `SiteScraper` for your struct
3. Add `pub mod mynewsite;` to `src/scrape/sites/mod.rs`
4. Register it in `ScraperRegistry::new()`

```rust
// In registry.rs
scrapers.push(Box::new(sites::mynewsite::MyNewSiteScraper));
```

## Watch Out!

**Scraper order matters!** If two scrapers can handle the same URL, the first one wins. Make sure `can_handle` methods are specific.

**Dynamic dispatch has overhead!** Each method call goes through dynamic dispatch. For FicHub's use case, this is negligible — network I/O dominates.

## Summary

The ScraperRegistry provides a clean interface for managing multiple scrapers. It's easy to extend and efficiently shares state across handlers.

---

# Chapter 14: AO3 Deep Dive

## Understanding AO3's HTML

AO3 has relatively clean, semantic HTML. The key elements are:

**Title:** `h2.title.heading`
**Author:** `a[rel='author']`
**Description:** `blockquote.userstuff`
**Word count:** `dd.words`
**Chapter count:** `dd.chapters` (format: "3/10")
**Status:** `dd.status` ("Complete" or "In Progress")
**Chapter content:** `div.chapter` > `div.userstuff`
**Tags:** `ul.tags li.fandom a.tag`, `ul.tags li.character a.tag`, etc.

## The lookup Method

The lookup method extracts metadata from the page:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;

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
```

The `?view_full_work=true` parameter tells AO3 to show all chapters on a single page.

### Extracting the Title

```rust
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
```

### Extracting the Author

```rust
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());

    let author_url = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|h| format!("{BASE_URL}{h}"))
        .unwrap_or_default();
```

### Extracting Word Count

```rust
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
```

Word counts are formatted with commas (e.g., "125,000"), so we remove them before parsing.

### Extracting Chapter Count

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

The chapter count is in the format "10/10" (current/total).

### Extracting Status

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

## The fetch_chapters Method

This method downloads all chapter content:

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

The key insight: AO3 wraps each chapter in `div.chapter`. We iterate over all chapter divs and extract the title and content from each one.

**inner_html vs text:** We use `.inner_html()` for chapter content because the EPUB generator needs HTML structure for proper formatting. We use `.text()` for metadata where we only need plain text.

## Extracting Tags

AO3 provides structured tags:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("no work ID".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client.get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let mut tags = Vec::new();

    // Fandom tags
    for el in document.select(&Selector::parse("ul.tags li.fandom a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }

    // Character tags
    for el in document.select(&Selector::parse("ul.tags li.character a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::character(&name));
        }
    }

    // Relationship tags
    for el in document.select(&Selector::parse("ul.tags li.relationship a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::relationship(&name));
        }
    }

    // Freeform tags
    for el in document.select(&Selector::parse("ul.tags li.freeform a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::freeform(&name));
        }
    }

    // Warning tags
    for el in document.select(&Selector::parse("ul.tags li.warnings a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::warning(&name));
        }
    }

    Ok(tags)
}
```

## Watch Out!

**AO3 rate limits aggressively!** Excessive requests lead to CAPTCHAs or temporary blocks.

**Some works require authentication!** Mature-rated works may need login.

**HTML changes break scrapers!** AO3 occasionally updates its HTML structure.

## Summary

The AO3 scraper demonstrates the full scraping workflow: URL detection, HTTP fetching, HTML parsing, data extraction, and error handling. It also shows how to extract structured tags for the tagging system.

---

# Chapter 15: Other Scrapers

## FF.net Scraper

FF.net's scraper fetches chapters individually:

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

        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
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

The chapter title is extracted from the chapter selector dropdown.

## XenForo Scraper

XenForo treats forum posts as chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let url = &meta.source;
    let response = client.get(url)
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
            chapter_idx as i32,
            title,
            content,
        });
    }

    if chapters.is_empty() {
        return Err(ScrapeError::ParseError("no content found".into()));
    }

    Ok(chapters)
}
```

## AdultFanFiction Scraper

AFF has a simpler structure with flexible content selectors:

```rust
let content_sel = Selector::parse("div.story_content, div.content, div#story").unwrap();
```

The comma-separated selector tries multiple possible structures.

## HP FanFic Scraper

The HP FanFic scraper handles multiple domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("hpfanficarchive.com") || url.contains("fanficauthors.net")
}
```

## Adding a New Scraper: Complete Example

Here's a template for adding a hypothetical new site:

### Step 1: Create the file

Create `src/scrape/sites/mynewsite.rs`:

```rust
pub struct MyNewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

#[async_trait]
impl SiteScraper for MyNewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("mynewsite.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let response = client.get(url)
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
            .select(&Selector::parse("h1").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("a.author").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let id = url.split('/').last().unwrap_or("unknown").to_string();
        let url_id = crate::scrape::generate_url_id(6, &id);
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
            author_local_id: id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let response = client.get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content_sel = Selector::parse("div.story-content, div.content, article").unwrap();
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

### Step 2: Add the module

In `src/scrape/sites/mod.rs`:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod mynewsite;
```

### Step 3: Register the scraper

In `src/scrape/registry.rs`:

```rust
scrapers.push(Box::new(sites::mynewsite::MyNewSiteScraper));
```

### Step 4: Test

```bash
cargo build
cargo test
```

## Watch Out!

**Source IDs must be unique!** Each site needs a unique `source_id`.

**Selector specificity matters!** Use more specific selectors to avoid matching wrong elements.

**Handle missing data gracefully!** Always provide defaults for optional fields.

## Summary

Each scraper handles its site's unique quirks. Adding a new scraper follows a consistent four-step process: create the file, add the module, register the scraper, and test.

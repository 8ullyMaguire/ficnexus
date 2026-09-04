# Part 3: Web Scraping

---

# Chapter 11: Understanding Fanfiction Sites

Before we can scrape fanfiction sites, we need to understand how they're structured. Each site has its own HTML layout, URL patterns, and quirks. This chapter covers the major fanfiction sites FicHub supports and the challenges each one presents.

## The Fanfiction Landscape

Fanfiction is published on several major platforms, each with its own culture, technical infrastructure, and content policies. Understanding these differences is crucial for building reliable scrapers.

### Archive of Our Own (AO3)

AO3 is the largest and most modern fanfiction platform. It's run by the Organization for Transformative Works, a non-profit dedicated to preserving fanworks.

**URL Patterns:**
- Work page: `https://archiveofourown.org/works/123456`
- Chapter page: `https://archiveofourown.org/works/123456/chapters/789012`
- Full work view: `https://archiveofourown.org/works/123456?view_full_work=true`

**HTML Structure:** AO3 uses semantic HTML with CSS classes. Key selectors:
- Title: `h2.title.heading`
- Author: `a[rel='author']`
- Chapters: `div.chapter`
- Chapter content: `div.userstuff`
- Stats: `dd.words`, `dd.chapters`, `dd.status`

**Challenges:**
- Rate limiting (AO3 actively blocks aggressive scrapers)
- Some works require login (restricted works)
- Multi-chapter works may have different HTML structures
- Work skins can alter the HTML layout

**Real-world analogy:** AO3 is like a well-organized library with a catalog system, clear signage, and helpful librarians. The HTML is structured and predictable, making it easier to parse. But the librarians (rate limiters) will ask you to leave if you check out too many books too quickly.

### FanFiction.net (FF.net)

FF.net is one of the oldest fanfiction platforms. It has a loyal user base but an aging technical infrastructure.

**URL Patterns:**
- Story page: `https://www.fanfiction.net/s/123456/1/`
- Chapter URLs: `https://www.fanfiction.net/s/123456/1/` (chapter number after story ID)

**HTML Structure:** FF.net uses older HTML patterns with table-based layouts. Key selectors:
- Title: `#profile_top b.xcontrast_txt`
- Author: `#profile_top a.xcontrast_txt`
- Description: `#profile_top div.xcontrast_txt`
- Chapter content: `div.storytext`
- Chapter selector: `select#chap_select option[selected]`

**Challenges:**
- Aggressive anti-scraping measures (CAPTCHA, IP blocking)
- Inconsistent HTML across different story types
- Chapter-by-chapter fetching required (no full work view)
- JavaScript-dependent features

**Real-world analogy:** FF.net is like an old bookstore with hand-written labels, creaky shelves, and a grumpy owner who doesn't like strangers browsing too long. The information is there, but you need to be careful and respectful.

### XenForo Forums (SpaceBattles, SufficientVelocity, etc.)

Several popular fanfiction communities use XenForo forum software. Stories are posted as forum threads with one post per chapter.

**URL Patterns:**
- Thread page: `https://forums.spacebattles.com/threads/story-name.12345/`
- Chapter URLs: Post numbers within the thread

**Supported Sites:**
- SpaceBattles: `forums.spacebattles.com`
- SufficientVelocity: `forums.sufficientvelocity.com`
- QuestionableQuesting: `forum.questionablequesting.com`

**HTML Structure:** XenForo uses a consistent template across all sites:
- Thread title: `h1.p-title-value`
- Posts: `article.message`
- Post content: `div.bbWrapper`
- Thread markers: `span.label` (for "Complete", "Abandoned", etc.)

**Challenges:**
- Multi-page threads (stories can span many pages)
- Inline images and embedded content
- User signatures and forum-specific markup
- Thread rules and moderation posts mixed with story content

**Real-world analogy:** XenForo is like a community bulletin board where each story is pinned as a long thread. Finding the actual story content among the discussion requires careful parsing.

### FictionPress

FictionPress is a sister site of FanFiction.net for original fiction. It shares the same technical infrastructure.

**URL Patterns:** Same as FF.net: `https://www.fictionpress.com/s/123456/1/`

FicHub's `FictionPressScraper` is actually an alias for `FfNetScraper` because the sites share the same HTML structure:

```rust
pub use FfNetScraper as FictionPressScraper;
```

This is a great example of code reuse — one scraper handles two sites.

## Content Extraction Challenges

### Handling Different HTML Structures

Each site wraps story content in different HTML elements. A robust scraper must handle:

- **Paragraphs:** `<p>`, `<div>`, `<br>` tags
- **Inline formatting:** `<b>`, `<i>`, `<em>`, `<strong>` tags
- **Block elements:** `<blockquote>`, `<div class="quote">`
- **Links:** `<a href="...">` tags
- **Images:** `<img src="...">` tags (rare in text-based fanfiction)
- **Special characters:** HTML entities like `&amp;`, `&lt;`, `&gt;`

### Metadata Extraction

Beyond the story content, scrapers need to extract:
- **Title** — Sometimes includes special characters or unicode
- **Author** — May be a pen name or anonymous
- **Word count** — Sometimes displayed, sometimes needs calculation
- **Chapter count** — For multi-chapter works
- **Status** — Complete, ongoing, hiatus, abandoned
- **Tags/Categories** — Site-specific metadata
- **Published/Updated dates** — When the story was posted/modified

### URL Normalization

Different sites use different URL formats, but FicHub normalizes them:
- `https://archiveofourown.org/works/123456` → normalized to canonical form
- `https://www.fanfiction.net/s/123456/1/` → normalized to canonical form
- `https://forums.spacebattles.com/threads/story-name.12345/` → normalized to canonical form

The `generate_url_id` function creates a deterministic ID from the source site and story ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 12 hex chars
}
```

This ensures the same story always gets the same ID, regardless of how the URL is formatted.

## Ethical Scraping

Web scraping raises ethical questions. Here are FicHub's guidelines:

1. **Respect rate limits** — Don't overwhelm the upstream sites
2. **Identify yourself** — Use a descriptive User-Agent string
3. **Cache results** — Don't re-scrape what you've already fetched
4. **Handle errors gracefully** — If a site blocks you, back off
5. **Don't scrape private content** — Respect login requirements
6. **Give credit** — Attribution to authors and sites

## 📝 Practice Exercises

1. **Site Analysis:** Visit AO3, FF.net, and SpaceBattles. Inspect the HTML structure of a story page on each. What are the key CSS selectors for title, author, and content?

2. **URL Pattern Matching:** Write a function that takes a URL and determines which site it's from. Test it with URLs from AO3, FF.net, SpaceBattles, and SufficientVelocity.

3. **Metadata Extraction:** Manually extract metadata from a story on each site. How consistent is the data? What fields are available on some sites but not others?

4. **Ethical Considerations:** Research the robots.txt files for AO3 and FF.net. What do they allow and disallow? How does FicHub comply?

---

# Chapter 12: Building Web Scrapers

Now let's build the actual scrapers. This chapter covers the `SiteScraper` trait, HTML parsing with the `scraper` crate, and the patterns used throughout the codebase.

## The SiteScraper Trait

FicHub defines a trait that all scrapers must implement:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    
    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    
    /// Extract structured tags from a fic URL (optional)
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

**`Send + Sync`** — These are marker traits that ensure the scraper can be safely shared across threads. `Send` means the type can be moved to another thread. `Sync` means the type can be shared between threads.

**`async_trait`** — This attribute macro enables async methods in traits. Without it, Rust's async methods in traits are not object-safe (can't be used as trait objects).

**`can_handle`** — This synchronous method checks if a URL belongs to this scraper's site. It's fast because it just checks URL patterns.

**`lookup`** — This async method fetches the story page and extracts metadata. It's async because it makes HTTP requests.

**`fetch_chapters`** — This async method fetches all chapter content. It's async because it may make multiple HTTP requests.

**`extract_tags`** — This optional async method extracts structured tags. Not all sites expose tags, so the default implementation returns an empty vector.

## HTML Parsing with the `scraper` Crate

The `scraper` crate provides CSS selector-based HTML parsing. Here's how it works:

```rust
use scraper::{Html, Selector};

fn parse_html(html: &str) {
    // Parse the HTML document
    let document = Html::parse_document(html);
    
    // Create a CSS selector
    let selector = Selector::parse("h2.title.heading").unwrap();
    
    // Find all matching elements
    for element in document.select(&selector) {
        // Extract text content
        let text: String = element.text().collect();
        println!("Found: {}", text);
        
        // Extract attributes
        if let Some(href) = element.value().attr("href") {
            println!("Link: {}", href);
        }
        
        // Extract inner HTML (preserves child elements)
        let html = element.inner_html();
        println!("HTML: {}", html);
    }
}
```

**Real-world analogy:** CSS selectors are like GPS coordinates for HTML elements. `h2.title.heading` means "find an `h2` element with class `title` and class `heading`." Just like GPS coordinates pinpoint a location on a map, CSS selectors pinpoint elements in an HTML document.

### Common CSS Selectors

Here are the selectors used most frequently in FicHub's scrapers:

```rust
// Element by tag name
Selector::parse("h2").unwrap()

// Element with a class
Selector::parse("h2.title").unwrap()

// Element with multiple classes
Selector::parse("h2.title.heading").unwrap()

// Element by ID
Selector::parse("#profile_top").unwrap()

// Element with attribute
Selector::parse("a[rel='author']").unwrap()

// Child element
Selector::parse("div.chapter h3.title").unwrap()

// Descendant element
Selector::parse("div p").unwrap()

// Attribute value contains
Selector::parse("[data-xutitle='word count']").unwrap()
```

## Building the FF.net Scraper

Let's build the FanFiction.net scraper step by step:

```rust
pub struct FfNetScraper;

impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for FfNetScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiction.net") || url.contains("fictionpress.com")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // 1. Extract story ID from URL
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
        
        // 2. Fetch the story page
        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
        let response = client
            .get(&fic_url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        // 3. Parse the HTML
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // 4. Extract metadata using CSS selectors
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
        
        // ... more metadata extraction ...
        
        // 5. Generate url_id and return
        let url_id = crate::scrape::generate_url_id(2, &story_id);
        let now = Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            // ... other fields ...
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();
        let story_id = &meta.author_local_id;
        
        // FF.net requires chapter-by-chapter fetching
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
}
```

### Building the XenForo Scraper

The XenForo scraper handles multiple sites:

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];

impl XenForoScraper {
    fn extract_thread_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/threads/[^.]+\.(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for XenForoScraper {
    fn can_handle(&self, url: &str) -> bool {
        XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let thread_id = Self::extract_thread_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract thread ID".into()))?;
        
        // XenForo threads may be multi-page
        // Fetch the first page to get metadata
        let response = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // Extract thread title
        let title = document
            .select(&Selector::parse("h1.p-title-value").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        // Extract author from first post
        let author = document
            .select(&Selector::parse("article.message:first-child .username").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        
        // Count total pages for chapter estimation
        let page_count = document
            .select(&Selector::parse("a.pageNav-page-jump").unwrap())
            .count()
            .max(1);
        
        // ... more extraction ...
        
        let url_id = crate::scrape::generate_url_id(3, &thread_id);
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: page_count as i32,
            // ...
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();
        let thread_id = &meta.author_local_id;
        
        // Determine the base URL from the source
        let base_url = if meta.source.contains("spacebattles") {
            "https://forums.spacebattles.com"
        } else if meta.source.contains("sufficientvelocity") {
            "https://forums.sufficientvelocity.com"
        } else {
            "https://forum.questionablequesting.com"
        };
        
        // Fetch all pages
        for page in 1..=meta.chapters {
            let url = format!("{}/threads/{}.page-{}", base_url, thread_id, page);
            let response = client
                .get(&url)
                .header("User-Agent", "fichub.net/0.1.0")
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            
            let html = response.text().await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let document = Html::parse_document(&html);
            
            // Collect all post content on this page
            let mut page_content = String::new();
            for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
                let post_html = post.inner_html();
                page_content.push_str(&post_html);
                page_content.push_str("<hr>");
            }
            
            chapters.push(Chapter {
                chapter_id: page,
                title: format!("Page {}", page),
                content: page_content,
            });
        }
        
        Ok(chapters)
    }
    
    async fn extract_tags(&self, client: &reqwest::Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        let response = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut tags = Vec::new();
        
        // Extract prefix tags (like [Complete], [Harry Potter], etc.)
        if let Some(title_el) = document.select(&Selector::parse("h1.p-title-value").unwrap()).next() {
            let title_text = title_el.text().collect::<String>();
            
            // Look for tags in brackets
            let re = regex_lite::Regex::new(r"\[([^\]]+)\]").unwrap();
            for cap in re.captures_iter(&title_text) {
                if let Some(tag) = cap.get(1) {
                    let tag_name = tag.as_str().to_string();
                    // Determine tag type based on content
                    let tag_type_id = if tag_name == "Complete" {
                        5  // Warning type
                    } else {
                        4  // Freeform
                    };
                    tags.push(ExtractedTag {
                        name: tag_name,
                        tag_type_id,
                    });
                }
            }
        }
        
        // Extract prefix labels (like "Fandom: Harry Potter")
        for label in document.select(&Selector::parse("dl.pairs--justified dt").unwrap()) {
            let label_text = label.text().collect::<String>().trim().to_string();
            if let Some(dd) = label.next_sibling_element() {
                let value = dd.text().collect::<String>().trim().to_string();
                match label_text.as_str() {
                    "Fandom" => tags.push(ExtractedTag::fandom(&value)),
                    "Rating" => tags.push(ExtractedTag::warning(&value)),
                    _ => {}
                }
            }
        }
        
        Ok(tags)
    }
}
```

## Error Handling in Scrapers

Scrapers need to handle many error conditions:

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

The error types map to different HTTP responses:
- `NotFound` → 404 Not Found
- `Blocked` → 502 Bad Gateway (upstream blocked us)
- `Network` → 502 Bad Gateway (network issue)
- `ParseError` → 502 Bad Gateway (site changed its HTML)

## 📝 Practice Exercises

1. **Selector Practice:** Given this HTML, write CSS selectors to extract:
   - The title: `<h2 class="title heading">Story Title</h2>`
   - The author: `<a rel="author" href="/users/123">Author Name</a>`
   - Word count: `<dd class="words">50,000</dd>`
   - Chapter content: `<div class="userstuff">Story text here</div>`

2. **Error Handling:** Write a scraper function that handles three error cases: network timeout, HTML parse error, and missing element. Return appropriate `ScrapeError` variants.

3. **Tag Extraction:** Given a page with tags like `[Harry Potter][Complete][Angst]`, write a function that extracts each tag and determines its type.

4. **URL Normalization:** Write a function that normalizes different URL formats for the same story:
   - `https://archiveofourown.org/works/123456`
   - `https://archiveofourown.org/works/123456?view_adult=true`
   - `http://www.archiveofourown.org/works/123456`

---

# Chapter 13: The Scraper Registry

The scraper registry is a pattern that maps URLs to the appropriate scraper. It's a key architectural pattern in FicHub.

## The Registry Pattern

The registry holds a list of all available scrapers and finds the right one for a given URL:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
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
}
```

**Real-world analogy:** The ScraperRegistry is like a reception desk at a hotel. When a guest (URL) arrives, the receptionist checks which floor (site) they need to go to and directs them to the right elevator (scraper). The receptionist doesn't know how to clean rooms or cook food — they just know who handles what.

### Why Box<dyn SiteScraper>?

The scrapers are stored as `Box<dyn SiteScraper>` — trait objects. This means the registry doesn't know the concrete type of each scraper at compile time. It just knows they all implement the `SiteScraper` trait.

This is called "dynamic dispatch" — the decision of which method to call is made at runtime, not compile time. The alternative is "static dispatch" (using generics), but that would require the registry to be generic over all scraper types, which would make the code much more complex.

**Real-world analogy:** Dynamic dispatch is like a phone directory. When you call a business, you don't know exactly who will answer. The phone system (trait object) routes your call to the right person (scraper) based on the number you dialed (URL). You just know that whoever answers will speak the same language (implement the same trait).

### Adding a New Scraper

To add support for a new fanfiction site:

1. Create a new file in `src/scrape/sites/`
2. Implement the `SiteScraper` trait
3. Add the scraper to `ScraperRegistry::new()`

That's it! The rest of the codebase automatically supports the new site.

## 📝 Practice Exercises

1. **New Scraper:** Create a scraper for a hypothetical fanfiction site called "StoryArchive" with URL pattern `https://storyarchive.com/story/12345`. Implement `can_handle`, `lookup`, and `fetch_chapters`.

2. **Registry Extension:** Add your new scraper to the `ScraperRegistry`. Write a test that verifies `find_scraper` returns the correct scraper for a StoryArchive URL.

3. **Priority Ordering:** Modify the registry to support priority ordering. If two scrapers can handle the same URL, the higher-priority one should be used.

---

# Chapter 14: AO3 Scraper Deep Dive

Let's take a detailed look at the AO3 scraper — FicHub's most important and complex scraper.

## AO3 HTML Structure

AO3's HTML is well-structured with semantic elements and consistent CSS classes. Here's a simplified version of a work page:

```html
<div id="skin">
  <div class="chapter work meta group">
    <h2 class="title heading">
      <a href="/works/123456">My Story Title</a>
    </h2>
    <h3 class="byline heading">
      by <a rel="author" href="/users/author_name">Author Name</a>
    </h3>
    
    <div class="stats">
      <dl>
        <dt>Chapters:</dt>
        <dd class="chapters">10/10</dd>
        <dt>Words:</dt>
        <dd class="words">50,000</dd>
        <dt>Status:</dt>
        <dd class="status">Complete</dd>
      </dl>
    </div>
    
    <blockquote class="userstuff">
      <p>Story description here...</p>
    </blockquote>
    
    <div id="chapters">
      <div class="chapter" id="chapter-1">
        <h3 class="heading">
          <a href="/works/123456/chapters/789012">Chapter 1: The Beginning</a>
        </h3>
        <div class="userstuff">
          <p>Chapter content here...</p>
        </div>
      </div>
      <!-- More chapters... -->
    </div>
  </div>
</div>
```

## The AO3 Scraper Implementation

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";

impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
        
        // Fetch the full work view
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
        
        // Extract title
        let title = document
            .select(&Selector::parse("h2.title.heading").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        // Extract author
        let author = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        
        // Extract author URL
        let author_url = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("{BASE_URL}{h}"))
            .unwrap_or_default();
        
        // Extract description
        let description = document
            .select(&Selector::parse("blockquote.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        // Extract chapter count
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
        
        // Extract word count
        let words = document
            .select(&Selector::parse("dd.words").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);
        
        // Extract status
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
        
        // Generate url_id
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
            source_id: 1,  // AO3 source ID
            author_id: 0,
            author_url,
            author_local_id: work_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }
    
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
        
        // Fallback for single-chapter works
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
    
    async fn extract_tags(&self, client: &reqwest::Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
        
        let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
        let response = client
            .get(&fic_url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut tags = Vec::new();
        
        // Extract fandom tags
        for li in document.select(&Selector::parse("dd.fandoms ul li").unwrap()) {
            let name = li.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tags.push(ExtractedTag::fandom(&name));
            }
        }
        
        // Extract character tags
        for li in document.select(&Selector::parse("dd.characters ul li").unwrap()) {
            let name = li.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tags.push(ExtractedTag::character(&name));
            }
        }
        
        // Extract relationship tags
        for li in document.select(&Selector::parse("dd.relationships ul li").unwrap()) {
            let name = li.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tags.push(ExtractedTag::relationship(&name));
            }
        }
        
        // Extract freeform tags
        for li in document.select(&Selector::parse("dd.freeform ul li").unwrap()) {
            let name = li.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tags.push(ExtractedTag::freeform(&name));
            }
        }
        
        // Extract warning tags
        for li in document.select(&Selector::parse("dd.warnings ul li").unwrap()) {
            let name = li.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tags.push(ExtractedTag::warning(&name));
            }
        }
        
        Ok(tags)
    }
}
```

## AO3-Specific Challenges

### Rate Limiting

AO3 has strict rate limiting. FicHub handles this by:
1. Using a descriptive User-Agent string
2. Adding delays between requests
3. Caching results aggressively
4. Respecting robots.txt

### Login-Required Works

Some AO3 works require a login to view. FicHub currently doesn't support login-based scraping — these works are marked as "not found."

### Work Skins

AO3 allows authors to apply custom "work skins" (CSS) that can change the HTML structure. FicHub's scraper focuses on the core semantic elements, which are consistent across skins.

## 📝 Practice Exercises

1. **AO3 Tag Extraction:** Given an AO3 work page HTML, extract all tag categories (fandom, character, relationship, freeform, warning) and their values.

2. **Multi-Chapter Handling:** Write a function that handles both single-chapter and multi-chapter AO3 works. Test with both cases.

3. **Error Recovery:** What happens when AO3 returns a CAPTCHA page? Write a function that detects this and returns an appropriate error.

---

# Chapter 15: Other Site Scrapers

This chapter covers the remaining scrapers: AdultFanFiction and HPFanFic.

## AdultFanFiction Scraper

AdultFanFiction.net is a smaller site with a simpler structure:

```rust
pub struct AdultFanFictionScraper;

#[async_trait]
impl SiteScraper for AdultFanFictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("adultfanfiction.net")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Similar pattern to FF.net
        // Extract story ID, fetch page, parse metadata
        todo!()
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Chapter-by-chapter fetching like FF.net
        todo!()
    }
}
```

## HPFanFic Scraper

HPFanFic.com is a Harry Potter-specific site:

```rust
pub struct HpFanFicScraper;

#[async_trait]
impl SiteScraper for HpFanFicScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("hpfanficarchive.com")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Site-specific parsing
        todo!()
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Site-specific chapter fetching
        todo!()
    }
}
```

## Scraping Best Practices

### User-Agent Identification

Always identify yourself with a descriptive User-Agent:

```rust
let response = client
    .get(&url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await?;
```

This tells the upstream site who you are and what you're doing. It's polite and helps them contact you if there's an issue.

### Timeout Configuration

Set appropriate timeouts for HTTP requests:

```rust
let client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(10))
    .build()?;
```

**Real-world analogy:** Timeouts are like waiting in line at a store. If the line doesn't move for 30 seconds, you leave. If the store doesn't open within 10 seconds of you arriving, you leave. Timeouts prevent your application from hanging indefinitely.

### Retry Logic

For transient failures, implement retry logic with exponential backoff:

```rust
async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_retries: u32,
) -> Result<String, ScrapeError> {
    let mut delay = std::time::Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                return response.text().await
                    .map_err(|e| ScrapeError::Network(e.to_string()));
            }
            Ok(response) if response.status().as_u16() == 429 => {
                // Rate limited — wait and retry
                tokio::time::sleep(delay).await;
                delay *= 2;  // Exponential backoff
            }
            Ok(_) => {
                return Err(ScrapeError::Network("non-success status".into()));
            }
            Err(e) if attempt < max_retries - 1 => {
                // Transient error — retry
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => {
                return Err(ScrapeError::Network(e.to_string()));
            }
        }
    }
    
    Err(ScrapeError::Network("max retries exceeded".into()))
}
```

## 📝 Practice Exercises

1. **New Site:** Create a scraper for a hypothetical fanfiction site. Focus on getting the metadata extraction right.

2. **Retry Logic:** Implement retry logic with exponential backoff for a scraper. Test it with a mock server that returns 429 on the first two requests.

3. **User-Agent Testing:** Try scraping a site with and without a User-Agent header. What happens?

4. **robots.txt:** Write a function that fetches and parses a site's robots.txt file. What URLs does it disallow?


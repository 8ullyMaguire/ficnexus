# Part 4: Fetching Stories — Web Scraping, Traits, and the Scraper Registry

*Chapters 12–15*

Every piece of data flows through these functions. Request comes in → handler looks up source → logs the request → does the work → returns a result. The request_source table tells us WHERE the request came from. The request_log table tells us WHAT happened. And the fic_info table stores the STORY DATA we scraped from the web.

But we've been glossing over one crucial step: how does FicHub actually *get* that story data? How does it reach out to AO3, FanFiction.net, and other sites, download story content, and turn it into something useful?

That's what this part is all about. We're going to build the scraping engine — the part of FicHub that reads fanfiction websites, pulls out the good stuff, and feeds it into our database.

---

## Chapter 12: How Web Scraping Works

### What Is Web Scraping?

Let's start with the big picture. When you visit a fanfiction website in your browser, here's what happens behind the scenes:

1. Your browser sends a request to the website's server: "Hey, give me the HTML for this page."
2. The server sends back a giant text document full of HTML tags.
3. Your browser reads that HTML and turns it into the pretty page you see — with colors, images, formatting, and layout.

Web scraping is basically step 1 and 2, but instead of a browser rendering the page, we write code that reads the HTML and picks out the parts we care about — the title, the author, the story content.

Think of it like this: imagine someone hands you a giant recipe book (that's the HTML page). You don't want to read the whole book — you just want the ingredients list for chocolate cake. Web scraping is like quickly flipping through the book and copying down just the ingredients.

### HTTP Requests: Talking to Websites

Before we can scrape anything, we need to send HTTP requests. HTTP is the language that browsers and servers use to talk to each other. The two most important types of requests are:

- **GET** — "Give me this page." This is what your browser does when you type a URL and press Enter.
- **POST** — "Here's some data, do something with it." This is what happens when you fill out a form and click Submit.

For scraping, we almost always use GET requests. We're just asking for pages, not submitting anything.

When you send a GET request, the server responds with a **status code**. The most common ones:

| Code | Meaning | What it means for scraping |
|------|---------|---------------------------|
| 200 | OK | Success! We got the page. |
| 301/302 | Redirect | The page moved. Follow the new URL. |
| 403 | Forbidden | The server said "no." We might be blocked. |
| 404 | Not Found | The story doesn't exist. |
| 429 | Too Many Requests | We're being too aggressive. Slow down! |
| 500 | Server Error | The site is having problems. Try again later. |

In Rust, we use the `reqwest` crate to make HTTP requests. Here's what a basic GET request looks like:

```rust
use reqwest::Client;

let client = Client::new();

// Send a GET request and wait for the response
let response = client
    .get("https://archiveofourown.org/works/123456")
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await?;

// Check if the request succeeded
if response.status().is_success() {
    // Get the HTML content
    let html = response.text().await?;
    println!("Got {} bytes of HTML!", html.len());
} else {
    println!("Request failed with status: {}", response.status());
}
```

Notice the `User-Agent` header. That's like showing your ID card when you knock on the door — it tells the server who's asking for the page. We'll talk more about why this matters in the "Ethical Scraping" section later.

### HTML Parsing: Making Sense of the Mess

Once we have the HTML, it looks something like this:

```html
<html>
<body>
  <h2 class="title heading">The Best Story Ever</h2>
  <a rel="author" href="/users/JaneDoe">JaneDoe</a>
  <div class="words">50,000</div>
  <div class="userstuff">
    <p>It was a dark and stormy night...</p>
  </div>
</body>
</html>
```

That's not very fun to read, is it? The HTML is wrapped in a structure called the **DOM tree** (Document Object Model). Every tag is like a branch on a tree:

```
<html>
├── <body>
│   ├── <h2 class="title heading">The Best Story Ever</h2>
│   ├── <a rel="author">JaneDoe</a>
│   ├── <div class="words">50,000</div>
│   └── <div class="userstuff">
│       └── <p>It was a dark and stormy night...</p>
```

In Rust, we use the `scraper` crate to parse HTML into a DOM tree. Here's how:

```rust
use scraper::{Html, Selector};

// Parse the HTML string into a document
let document = Html::parse_document(&html);

// Now we can search for elements using CSS selectors
let title_selector = Selector::parse("h2.title.heading").unwrap();

// Find the first matching element
if let Some(title_el) = document.select(&title_selector).next() {
    let title_text: String = title_el.text().collect();
    println!("Title: {}", title_text.trim());
}
```

### CSS Selectors: Finding Needles in Haystacks

**CSS selectors** are patterns that describe which HTML elements you want to find. They're the same syntax you'd use in CSS to style web pages, but here we're using them to search through HTML.

Here are some common selector patterns:

| Selector | What it matches |
|----------|----------------|
| `h2` | All `<h2>` elements |
| `.title` | All elements with class "title" |
| `h2.title` | `<h2>` elements with class "title" |
| `#story` | The element with id "story" |
| `a[rel='author']` | `<a>` tags where the `rel` attribute equals "author" |
| `div.userstuff p` | `<p>` tags inside `<div class="userstuff">` |
| `dd.words` | `<dd>` elements with class "words" |

Let's try a more complete example:

```rust
use scraper::{Html, Selector};

let html = r#"
<html>
<body>
  <h2 class="title heading">The Best Story Ever</h2>
  <a rel="author" href="/users/JaneDoe">JaneDoe</a>
  <dd class="words">50,000</dd>
  <dd class="chapters">5 / 10</dd>
</body>
</html>
"#;

let doc = Html::parse_document(html);

// Extract the title
let title = doc.select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

// Extract the author
let author = doc.select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

// Extract word count (remove commas before parsing)
let words: i64 = doc.select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>()
            .replace(',', "")
            .parse()
            .ok()
    })
    .unwrap_or(0);

// Extract chapter count from "5 / 10" format
let chapters_text: String = doc.select(&Selector::parse("dd.chapters").unwrap())
    .next()
    .map(|el| el.text().collect())
    .unwrap_or_default();

let chapters = if let Some(pos) = chapters_text.find('/') {
    chapters_text[..pos].trim().parse().unwrap_or(1)
} else {
    1
};

println!("Title: {title}");
println!("Author: {author}");
println!("Words: {words}");
println!("Chapters: {chapters}");
```

Notice the pattern: every extraction follows the same three steps:

1. **Parse** the HTML with `Html::parse_document()`
2. **Select** elements with `document.select(&Selector::parse("...").unwrap())`
3. **Extract** text or attributes from the matched elements

> 💡 **Key Concept: The `unwrap()` on Selectors**
>
> You'll notice we call `.unwrap()` on `Selector::parse()`. This is safe because we know our selector strings are valid CSS. If we wrote `"h2..title"` (two dots — invalid), it would panic at runtime. Since we control the selector strings, this is fine. If you were accepting selectors from user input, you'd want to handle the error.

### The Scraper Crate for Rust

The `scraper` crate (version 0.27 in FicHub) is what makes all this possible. Add it to your `Cargo.toml`:

```toml
[dependencies]
scraper = "0.27"
```

The `scraper` crate gives us two main tools:

- **`Html::parse_document()`** — Turns a string of HTML into a searchable DOM tree
- **`Selector::parse()`** — Compiles a CSS selector string into a reusable pattern

The selector compilation step is important: parsing a selector string takes time, so we do it once and reuse the compiled selector. In FicHub's scrapers, you'll often see selectors created with `.unwrap()` inside function calls — they're compiled each time, which is fine for scraping but would be wasteful in a hot loop.

### Ethical Scraping: Don't Be a Jerk

Web scraping is powerful, but it comes with responsibilities. You're making your program read someone else's website, and that takes their server resources. Here are the rules:

**1. Always set a User-Agent.**

The User-Agent header tells the website who's making the request. Don't pretend to be a browser — be honest about who you are:

```rust
.header("User-Agent", "fichub.net/0.1.0")
```

If you send requests without a User-Agent, many websites will block you automatically.

**2. Respect rate limits.**

Don't hammer a website with hundreds of requests per second. If you get a 429 (Too Many Requests) response, slow down. Add delays between requests. Be a good citizen.

**3. Check robots.txt.**

Every major website has a file at `/robots.txt` that says which parts of the site crawlers are allowed to access. Respect it. If a site says "no bots in /admin/", don't scrape /admin/.

**4. Cache aggressively.**

If you've already scraped a story, don't scrape it again. Store the results and reuse them. This is why FicHub has a caching layer — more on that in a later part.

**5. Don't scrape sites that don't want to be scraped.**

If a site has measures to block scraping (like Cloudflare protection), that's their way of saying "please don't." Don't try to bypass those protections.

> ⚠️ **Watch Out: Cloudflare and CAPTCHAs**
>
> Some websites use Cloudflare or similar services to block automated requests. If you encounter a challenge page instead of the content you expected, that's the site telling you to back off. Don't try to solve CAPTCHAs programmatically — that crosses a line from "scraping" into "attacking."

### 🧪 Try It Yourself

1. Install the `reqwest` and `scraper` crates
2. Write a program that fetches any public webpage and prints its title
3. Try to extract all the links (`<a>` tags) from the page
4. What happens when you try to scrape a page that requires JavaScript? (Hint: it doesn't work with `reqwest` alone!)

---

## Chapter 13: The Scraper Trait

### Why We Need a Trait

FicHub doesn't just scrape one website — it scrapes many: AO3, FanFiction.net, XenForo forums, and more. Each site has different HTML structure, different URLs, and different ways of organizing content.

If we wrote all the scraping logic in one big function, it would be a tangled mess of if/else statements. Instead, we use a **trait** — Rust's way of saying "any scraper must follow this contract."

> 💡 **Key Concept: Traits as Contracts**
>
> A trait is like a job description. It says: "To be a scraper, you must be able to do these four things." Each scraper (AO3, FF.net, etc.) is a different employee that follows the same job description in their own way.

### The SiteScraper Trait

Here's the actual trait from FicHub's source code:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Extract metadata from a story URL
    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str
    ) -> Result<FicMetadata, ScrapeError>;

    /// Fetch all chapters given metadata
    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata
    ) -> Result<Vec<Chapter>, ScrapeError>;

    /// Extract structured tags from a fic URL
    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str
    ) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

Let's break down each method.

### `can_handle(url)` — Is This My Job?

```rust
fn can_handle(&self, url: &str) -> bool;
```

This method answers a simple question: "Can this scraper handle this URL?" Each scraper checks the URL to see if it matches the site it knows about.

For example, the AO3 scraper checks if the URL contains `archiveofourown.org`:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

The FF.net scraper checks for `fanfiction.net` or `fictionpress.com`:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

This is a synchronous method — it's just checking a string, no network calls needed. That's why it's not `async`.

### `lookup(client, url)` — Tell Me About This Story

```rust
async fn lookup(
    &self,
    client: &reqwest::Client,
    url: &str
) -> Result<FicMetadata, ScrapeError>;
```

This is the big one. When someone asks FicHub about a story URL, this method:

1. Sends an HTTP request to the website
2. Parses the HTML response
3. Extracts metadata: title, author, chapter count, word count, status
4. Returns a `FicMetadata` struct with all that information

The `client` parameter is a `reqwest::Client` — a reusable HTTP client that maintains connection pools and handles TLS. We pass it in instead of creating a new one each time, because creating HTTP clients is expensive.

### `fetch_chapters(client, meta)` — Download the Story

```rust
async fn fetch_chapters(
    &self,
    client: &reqwest::Client,
    meta: &FicMetadata
) -> Result<Vec<Chapter>, ScrapeError>;
```

Once we have the metadata, we need the actual story content. This method takes the metadata and downloads every chapter. For a single-chapter story, it makes one request. For a 50-chapter epic, it might make 50 requests.

### `extract_tags(client, url)` — What Are the Tags?

```rust
async fn extract_tags(
    &self,
    _client: &reqwest::Client,
    _url: &str
) -> Result<Vec<ExtractedTag>, ScrapeError> {
    Ok(Vec::new())
}
```

Notice that `extract_tags` has a **default implementation** — it returns an empty vector. Not every site has structured tags, so scrapers can choose to override this method if they can extract tags, or just use the default.

### The FicMetadata Struct

When `lookup` finishes its work, it returns a `FicMetadata` struct. Here's the actual struct from FicHub:

```rust
pub struct FicMetadata {
    pub url_id: String,           // Unique ID for this story
    pub title: String,            // "The Best Story Ever"
    pub author: String,           // "JaneDoe"
    pub chapters: i32,            // 12
    pub words: i64,               // 85000
    pub desc: String,             // Story description/summary
    pub published: i64,           // Unix timestamp (millis)
    pub updated: i64,             // Unix timestamp (millis)
    pub status: String,           // "complete", "ongoing", etc.
    pub source: String,           // Original URL
    pub source_id: i64,           // Which site (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Local author ID
    pub author_url: String,       // Author's profile URL
    pub author_local_id: String,  // Story ID on the original site
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

This is a rich struct with lots of fields. Let's focus on the important ones:

- **`url_id`** — A unique identifier we generate by hashing the source and story ID. More on this in Chapter 14.
- **`source_id`** — A number that tells us which site this came from (1 = AO3, 2 = FF.net, 3 = XenForo).
- **`author_local_id`** — The story's ID on the original site. For AO3, this is the work number (like `123456`).
- **`status`** — Is the story complete, ongoing, on hiatus, or cancelled?

### The Chapter Struct

Each chapter is simpler:

```rust
pub struct Chapter {
    pub chapter_id: i32,  // Chapter number (1, 2, 3...)
    pub title: String,    // Chapter title
    pub content: String,  // HTML content of the chapter
}
```

The `content` field stores raw HTML, not plain text. This is intentional — HTML preserves formatting like paragraphs, bold text, italics, and blockquotes. When FicHub converts stories to EPUB or other formats, it needs that formatting information.

### The ExtractedTag Struct

Tags tell us what a story is about — which fandom, characters, relationships, and themes:

```rust
pub struct ExtractedTag {
    pub name: String,        // "Harry Potter"
    pub tag_type_id: i16,    // 1=Fandom, 2=Character, 3=Relationship, etc.
}
```

FicHub has convenience constructors for each tag type:

```rust
impl ExtractedTag {
    pub fn fandom(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 1 }
    }
    pub fn character(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 2 }
    }
    pub fn relationship(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 3 }
    }
    pub fn freeform(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 4 }
    }
    pub fn warning(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 5 }
    }
}
```

This makes tag extraction readable and self-documenting:

```rust
let tags = vec![
    ExtractedTag::fandom("Harry Potter"),
    ExtractedTag::character("Harry Potter"),
    ExtractedTag::relationship("Harry Potter/Draco Malfoy"),
    ExtractedTag::freeform("Alternate Universe"),
    ExtractedTag::warning("Major Character Death"),
];
```

### The ScrapeError Enum

Not everything goes smoothly. FicHub defines four error types:

```rust
pub enum ScrapeError {
    NotFound,              // Story doesn't exist (404)
    Blocked,               // Site blocked our request (403)
    Network(String),       // Connection error
    ParseError(String),    // HTML didn't match expected structure
}
```

Each error tells us something different:

- **`NotFound`** — The story was deleted, or the URL is wrong. Nothing we can do.
- **`Blocked`** — The site said "no." We should back off.
- **`Network`** — DNS failed, connection timed out, something went wrong with the network. We could retry this.
- **`ParseError`** — We got the HTML but couldn't understand it. The site might have changed its layout. This is the scraper equivalent of "I can see the page but I'm confused."

### 🧪 Try It Yourself

1. Define your own `SiteScraper` trait with just `can_handle` and `lookup`
2. Create a `MySiteScraper` struct that implements it for a test URL
3. Try calling `lookup` with a URL that doesn't exist — what error do you get?
4. Add a method `supports_tags(&self) -> bool` to your trait. What should the default implementation return?

---

## Chapter 14: AO3 Scraper

### Archive of Our Own: The Big One

AO3 (Archive of Our Own) is the largest fanfiction archive on the internet. It hosts millions of stories across thousands of fandoms. If FicHub is going to be useful, it *has* to support AO3.

AO3 URLs follow a consistent pattern:

```
https://archiveofourown.org/works/123456
https://archiveofourown.org/works/123456/chapters/789012
```

The number after `/works/` is the **work ID** — a unique number that identifies every story on AO3. The optional `/chapters/` part points to a specific chapter.

### The AO3 Scraper Struct

Here's how we start:

```rust
pub struct Ao3Scraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

const BASE_URL: &str = "https://archiveofourown.org";
```

Notice that `Ao3Scraper` has no fields — it's a unit struct. All the data it needs comes from the parameters passed to its methods. This makes it stateless and simple.

### Extracting the Work ID

Before we can do anything, we need to pull the work ID out of the URL. AO3 uses a regex to match the `/works/NUMBER` pattern:

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

Let's trace through this:

1. `Regex::new(r"/works/(\d+)")` — Creates a regex that matches `/works/` followed by one or more digits. The parentheses `(\d+)` create a **capture group** — it captures the digits.
2. `re.captures(url)?` — Tries to match the regex against the URL. If it doesn't match, the `?` returns `None`.
3. `get(1)` — Gets the first capture group (the digits).
4. `map(|m| m.as_str().to_string())` — Converts the match to a `String`.

So for `https://archiveofourown.org/works/123456/chapters/789012`, it extracts `"123456"`.

### The can_handle Method

This is simple — just check the domain:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

### The lookup Method

This is where the real work happens. Let's walk through it step by step:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    // Step 1: Extract the work ID from the URL
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError(
            "could not extract work ID from AO3 URL".into()
        ))?;

    // Step 2: Fetch the full work page (all chapters at once)
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    // Step 3: Check for errors
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }

    // Step 4: Parse the HTML
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    // Step 5: Extract metadata using CSS selectors
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    // ... more extraction ...

    // Step 6: Build and return FicMetadata
    Ok(FicMetadata { /* ... */ })
}
```

Let's look at the interesting parts more closely.

**The `?` operator for error handling.** When we extract the work ID, we use `.ok_or_else()` to convert a missing ID into a `ScrapeError::ParseError`. The `?` then returns early if the error occurred. This keeps the code clean — no nested if/else blocks.

**The `view_full_work=true` parameter.** AO3 normally shows one chapter at a time. By adding `?view_full_work=true` to the URL, we tell AO3 to show us the entire story at once. This means we can grab all the chapter content in a single HTTP request instead of making separate requests for each chapter.

**The `map_err` pattern.** When `reqwest` fails, it returns its own error type. We use `.map_err()` to convert it into our `ScrapeError::Network` so everything uses the same error type.

### Extracting Specific Fields

Let's look at how AO3's HTML maps to our data:

```rust
// Title: lives in an <h2> with classes "title" and "heading"
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

// Author: lives in an <a> tag with rel="author"
let author = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

// Word count: lives in a <dd> with class "words"
let words = document
    .select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

Notice the pattern: every extraction ends with either `.unwrap_or_else(|| "default".to_string())` or `.unwrap_or(0)`. This handles the case where the HTML doesn't match what we expected — instead of crashing, we use a sensible default.

### Handling Multi-Chapter Works

AO3 displays chapters differently depending on whether `view_full_work=true` is used. When fetching the full work, chapters are wrapped in `<div class="chapter">` elements:

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
    // ... fallback for single-chapter works ...
}
```

The key here is the `.enumerate()` — it gives us both the index (`i`) and the element. We use the index to number chapters starting at 1.

Notice the fallback at the end. Some AO3 works don't have chapter divs (single-chapter stories). In that case, we look for a `div.userstuff` directly:

```rust
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
```

This is defensive programming — we handle the unexpected case instead of returning an empty result.

### The generate_url_id Function

Every story needs a unique ID in FicHub's database. Rather than using sequential numbers (1, 2, 3...) that could collide across sites, FicHub generates a deterministic ID by hashing the source and story ID together:

```rust
use sha2::{Sha256, Digest};

pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])
}
```

Let's break this down:

1. Create a SHA-256 hasher
2. Feed it the source ID (e.g., `1` for AO3) and the story ID (e.g., `123456`), separated by a colon
3. Get the hash result and take the first 6 bytes
4. Encode those 6 bytes as hex (12 characters)

Why 6 bytes? SHA-256 produces 32 bytes, but we only need enough to be unique. 12 hex characters gives us 2^48 (about 281 trillion) possible IDs — more than enough.

The colon separator is important: without it, `source_id=12, story_id="34"` would produce the same hash as `source_id=1, story_id="234"`. The colon prevents this collision.

```rust
// Same source + story = same ID (deterministic!)
let id1 = generate_url_id(1, "story_123");
let id2 = generate_url_id(1, "story_123");
assert_eq!(id1, id2);

// Different source = different ID
let id3 = generate_url_id(2, "story_123");
assert_ne!(id1, id3);
```

> ⚠️ **Watch Out: Full Work vs. Individual Chapters**
>
> The `?view_full_work=true` trick only works for AO3. FF.net requires you to fetch each chapter individually. When writing a new scraper, always check: does the site offer a way to get all chapters at once, or do we need to loop?

### 🧪 Try It Yourself

1. Manually visit an AO3 work URL in your browser and open the "View Source" (Ctrl+U)
2. Find the `<h2>` with the title — what are its exact classes?
3. Find the word count element — what tag and class does it use?
4. Write a Rust program that takes an AO3 work ID, fetches the page, and prints the title and word count
5. What happens if you try to scrape an AO3 work that requires authentication?

---

## Chapter 15: Other Scrapers and the Registry

### FF.net: A Different Beast

FanFiction.net (FF.net) is one of the oldest fanfiction sites — it's been around since 1998! Its HTML structure is very different from AO3, which is why we need a separate scraper. Each site has its own personality, its own HTML quirks, and its own way of organizing stories.

FF.net URLs look like `https://www.fanfiction.net/s/123456/1/` — the `/s/NUMBER` pattern holds the story ID, and the trailing `1/` is the chapter number. The scraper needs to extract that story ID:

```rust
pub struct FfNetScraper;

impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

FF.net metadata lives in a very specific part of the page — inside a `<div id="profile_top">`. The title lives in a bold tag, the author in a link, and the word count is buried in a span with a special data attribute:

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

let words = document
    .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

Notice how specific these selectors are. `#profile_top b.xcontrast_txt` means "find a bold tag with class `xcontrast_txt` inside the element with id `profile_top`." This precision is necessary because HTML pages often have multiple elements of the same type — we need to be exact.

The FF.net scraper works differently from AO3 in one key way: **it has to fetch each chapter separately**. AO3 gives us the full work with `?view_full_work=true`, but FF.net only shows one chapter at a time. So `fetch_chapters` loops through them:

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

Notice the CSS selectors are completely different from AO3:
- AO3 uses `div.userstuff` for content; FF.net uses `div.storytext`
- AO3 uses `h3.title` for chapter titles; FF.net uses a `<select>` dropdown with the active `<option>`

Each site has its own HTML dialect, and each scraper must speak that dialect fluently.

> 💡 **Key Concept: FictionPress**
>
> FictionPress.com is the original fiction-publishing site by the same people who created FF.net. It uses the exact same HTML structure! FicHub reuses the FF.net scraper for FictionPress with a single line of code:
>
> ```rust
> pub use FfNetScraper as FictionPressScraper;
> ```
>
> This is a great example of code reuse — no need to write a new scraper when the sites share the same structure.

### XenForo: Forum-Based Sites

Some fanfiction lives on forums rather than dedicated archives. SpaceBattles, SufficientVelocity, and QuestionableQuesting all run on XenForo — forum software that's been around for decades.

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];

impl XenForoScraper {
    fn is_xenforo_url(url: &str) -> bool {
        XENFORO_DOMAINS.iter().any(|d| url.contains(d))
    }

    fn extract_thread_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

XenForo URLs look like `https://forums.spacebattles.com/threads/some-story-title.123456/`. The thread ID is the number after the period in the last segment.

Forum-based scraping is different from archive scraping:

1. **Posts, not chapters.** A XenForo "story" is a thread, and each post is like a chapter. The first post usually contains the story content, but authors sometimes split stories across multiple posts.

2. **Thread pagination.** Long threads span multiple pages. The scraper would need to follow "Next Page" links.

3. **No metadata.** Forums don't track word counts, chapter counts, or completion status the way AO3 does. The XenForo scraper sets `chapters: 1` and `words: 0` as defaults — it knows it can't extract that information.

4. **Extra content.** Forum posts include signatures, quotes, and other non-story content. A more advanced scraper would need to filter those out.

### The ScraperRegistry: Finding the Right Scraper

Now we have multiple scrapers, each knowing how to handle different sites. But how does FicHub know which one to use for a given URL?

The answer is the **ScraperRegistry** — a collection that finds the right scraper automatically:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

This is a vector of **trait objects** — `Box<dyn SiteScraper>` means "a boxed thing that implements the SiteScraper trait." This is how we store different types (Ao3Scraper, FfNetScraper, etc.) in the same collection.

### Building the Registry

When FicHub starts up, it creates the registry and registers all known scrapers:

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

Each scraper is wrapped in `Box::new()` to put it on the heap, and stored in the vector. The registry doesn't care about the concrete types — it only knows that everything in the vector implements `SiteScraper`.

### Finding the Right Scraper

When a URL comes in, the registry iterates through its scrapers and finds the first one that claims it can handle the URL:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

The `.find()` method stops at the first match — it calls `can_handle()` on each scraper until one returns `true`. If no scraper claims the URL, it returns `None`.

### Convenience Methods

The registry also provides high-level methods that combine finding the scraper with calling its methods:

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

This is the **Registry Pattern** — one of the most useful patterns in software engineering. Instead of writing a big switch statement like:

```rust
// ❌ The bad way — hard to extend, messy
if url.contains("archiveofourown.org") {
    ao3_lookup(url).await
} else if url.contains("fanfiction.net") {
    ffnet_lookup(url).await
} else if url.contains("spacebattles.com") {
    xenforo_lookup(url).await
} else {
    Err(ScrapeError::NotFound)
}
```

We let the scrapers self-identify. Adding a new site means writing a new scraper and adding one line to `new()`. No giant if/else chain to maintain.

### Adding a New Scraper

Want to add support for a new site? Here's the recipe:

1. **Create a new file** in `src/scrape/sites/` (e.g., `wattpad.rs`)

2. **Define a struct** (it can be a unit struct with no fields):
   ```rust
   pub struct WattpadScraper;
   ```

3. **Implement `SiteScraper`** for your struct:
   ```rust
   #[async_trait]
   impl SiteScraper for WattpadScraper {
       fn can_handle(&self, url: &str) -> bool {
           url.contains("wattpad.com")
       }

       async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
           // Your scraping logic here
           todo!()
       }

       async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
           // Your chapter fetching logic here
           todo!()
       }
   }
   ```

4. **Add the module** to `src/scrape/sites/mod.rs`:
   ```rust
   pub mod wattpad;
   ```

5. **Register it** in `src/scrape/registry.rs`:
   ```rust
   scrapers.push(Box::new(sites::wattpad::WattpadScraper));
   ```

That's it! Five steps, and FicHub can now scrape a new website. The registry automatically picks up the new scraper, and all the existing code (the API handlers, the caching layer, the database) works without any changes.

> 💡 **Key Concept: Open/Closed Principle**
>
> The registry pattern follows the Open/Closed Principle: the code is *open for extension* (add new scrapers) but *closed for modification* (existing scrapers don't need to change). When you add a new scraper, you don't touch any existing code. This is a hallmark of good software design.

### The Module Structure

FicHub organizes scrapers into a clean module hierarchy:

```
src/scrape/
├── mod.rs           // Trait definition, error types, generate_url_id
├── registry.rs      // ScraperRegistry
└── sites/
    ├── mod.rs       // Re-exports all site modules
    ├── ao3.rs       // AO3 scraper
    ├── ffnet.rs     // FanFiction.net scraper
    ├── xenforo.rs   // XenForo forum scraper
    ├── fictionpress.rs  // FictionPress scraper (reuses FF.net)
    ├── adultfanfiction.rs
    └── hpfanfic.rs
```

The `mod.rs` files serve as the public interface. Other parts of FicHub don't reach into `scrape::sites::ao3::Ao3Scraper` directly — they use `scrape::registry::ScraperRegistry` which handles everything behind the scenes.

### Error Handling Across Scrapers

Remember our `ScrapeError` enum? Let's see how it maps to real-world scenarios:

```rust
pub enum ScrapeError {
    NotFound,              // Story deleted or URL invalid
    Blocked,               // Site returned 403
    Network(String),       // DNS failure, timeout, connection refused
    ParseError(String),    // HTML structure changed, can't parse
}
```

Each scraper converts errors into this common format. When `reqwest` fails, we wrap it in `ScrapeError::Network`. When the status code is unexpected, we return `ScrapeError::NotFound` or `ScrapeError::Blocked`. When the HTML doesn't match our selectors, we return `ScrapeError::ParseError`.

The `Display` implementation makes these errors human-readable:

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
```

This means the rest of FicHub doesn't need to know about individual site errors. The handlers just match on `ScrapeError` and respond appropriately:

- **NotFound** → Return a 404 to the user
- **Blocked** → Return a 503 (service unavailable) with a message about the site blocking us
- **Network** → Return a 502 (bad gateway) and log the error for retry
- **ParseError** → Return a 500 (internal server error) and alert the developer that a site might have changed its HTML structure

> ⚠️ **Watch Out: ParseError Means the Site Changed**
>
> If you start seeing `ParseError` for a site that used to work, it probably means the website changed its HTML layout. This happens more often than you'd think! Websites redesign their pages, and suddenly your carefully crafted CSS selectors stop matching. When this happens, you need to visit the site, inspect the new HTML structure, and update your selectors. This is why scrapers need maintenance — the web is always changing.

### Scaling to Many Sites

The beauty of this architecture is how well it scales. FicHub currently supports six scrapers, but the pattern works just as well for twenty or fifty. Each scraper is independent — changing how the AO3 scraper works doesn't affect the FF.net scraper or the XenForo scraper.

The registry is also efficient. `find_scraper()` is a linear scan, but with fewer than ten scrapers, that's negligible. If you ever had hundreds of scrapers, you could optimize with a HashMap keyed by domain, but for FicHub's scale, the simple approach is the best approach.

### 🧪 Try It Yourself

1. Look at the `sites/mod.rs` file — how many scrapers are registered?
2. Create a new scraper struct for a test site (even if it's just a placeholder)
3. Add it to the registry and call `scraper_count()` — does it return the right number?
4. What happens if two scrapers both claim `can_handle()` for the same URL? (Hint: `.find()` returns the first match)
5. Try writing a `find_scraper` test that verifies a known AO3 URL returns the `Ao3Scraper`

### What We Built

In these four chapters, we went from zero to a complete web scraping system:

- **Chapter 12**: We learned what web scraping is — sending HTTP requests to websites, receiving HTML, and parsing it to extract data. We covered HTTP basics (GET requests, status codes), HTML parsing with the `scraper` crate, and CSS selectors for finding elements. We also talked about ethical scraping — being honest about who you are, respecting rate limits, and caching aggressively.

- **Chapter 13**: We designed the `SiteScraper` trait — the contract that all scrapers must follow. We defined `FicMetadata`, `Chapter`, and `ExtractedTag` structs for the data scrapers produce, and `ScrapeError` for when things go wrong. The trait gives us a clean interface: `can_handle`, `lookup`, `fetch_chapters`, and `extract_tags`.

- **Chapter 14**: We built the AO3 scraper — the most important one. We extracted work IDs from URLs, fetched full work pages, parsed metadata with CSS selectors, and handled both single-chapter and multi-chapter works. We also learned about `generate_url_id`, which creates deterministic IDs using SHA-256 hashing.

- **Chapter 15**: We saw how other scrapers work (FF.net with per-chapter fetching, XenForo with forum posts), and built the `ScraperRegistry` — a collection that automatically finds the right scraper for any URL. The registry pattern makes FicHub extensible: adding a new site means writing one file and adding one line.

Here's the big picture: when a user sends a URL to FicHub, the request flows through `ScraperRegistry::lookup()` to find the right scraper, that scraper sends an HTTP request to the site, parses the HTML, and returns `FicMetadata`. Then `ScraperRegistry::fetch_chapters()` downloads the actual story content. All of this happens in a few hundred milliseconds, and the results get stored in the database so we never have to scrape the same story twice.

The scraping layer is also where FicHub's extensibility really shines. Fanfiction lives on dozens of sites across the internet, and each one has its own quirks. By using the trait + registry pattern, FicHub can support any new site without touching existing code. A contributor who wants to add Wattpad support just creates a new scraper file, implements `SiteScraper`, and adds one line to the registry. The rest of the system — the API handlers, the database, the export pipeline — all work automatically.

This is the heart of FicHub. The scrapers reach out to the internet and bring back the stories that readers love. In the next part, we'll build the caching layer — because once we've scraped a story, we shouldn't have to do it again.

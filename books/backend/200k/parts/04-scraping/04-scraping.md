# Part 4: Web Scraping — How FicHub Reads the Internet

> "We've now built the complete data layer: the connection pool, the Rust models, the database queries, blacklists, request logging, and cache versioning. Every piece of data flows through these functions. In the next part, we'll learn how scraping works."

This is it — the part where FicHub comes alive. Everything we've built so far (the database, the models, the connection pool) is just infrastructure. It's the stage. Now we need the actors.

The actors are **scrapers** — pieces of code that go out to fanfiction websites, read their pages, and pull down stories so we can store them, serve them as EPUBs, and make them searchable.

If Part 3 was the library where books are stored, Part 4 is the librarian who goes out and *finds* the books.

This part is where the rubber meets the road. We'll start with the basics of how the web works, learn about HTML parsing and CSS selectors, build a trait-based scraper system that can handle six different fanfiction sites, and tie it all together with a registry that automatically picks the right scraper for any URL.

---

## Chapter 20: How Web Scraping Works

### What Is Web Scraping?

Let's start with the basics. When you visit a website in your browser — say, `https://archiveofourown.org/works/1234567` — here's what actually happens behind the scenes:

1. Your browser sends an **HTTP request** to the AO3 server
2. The server sends back an **HTTP response** — a chunk of text called HTML
3. Your browser reads that text and turns it into a pretty webpage with colors, images, and layout

The text the server sends back is **HTML** — HyperText Markup Language. It looks something like this:

```html
<html>
<head><title>The Story of a Lifetime - AO3</title></head>
<body>
  <div id="workskin">
    <h2 class="title heading">The Story of a Lifetime</h2>
    <h3 class="byline">
      <a rel="author" href="/users/someauthor/pseuds/someauthor">SomeAuthor</a>
    </h3>
    <div class="chapters">
      <span class="chapter-heading">Chapters:</span>
      <dd class="chapters">3 / 5</dd>
    </div>
    <div class="words">
      <span class="chapter-heading">Words:</span>
      <dd class="words">42,000</dd>
    </div>
  </div>
</body>
</html>
```

That HTML contains everything we need: the title, the author, the chapter count, the word count. A **web scraper** is just a program that:

1. Sends the same HTTP request your browser would send
2. Reads the HTML that comes back
3. Pulls out the data we care about

Think of it like this: a browser is a translator that turns HTML into pictures. A scraper is a translator that turns HTML into *data*.

### Why Not Use the Browser?

You might wonder: "Can't we just automate a browser?" We could! Tools like Selenium or Puppeteer can control a real web browser programmatically. But there are good reasons to use direct HTTP requests instead:

1. **Speed**: A browser has to load CSS, JavaScript, images, fonts... We just need the HTML text. Our requests finish in milliseconds, not seconds.
2. **Memory**: Running a browser uses hundreds of megabytes of RAM. An HTTP client uses almost nothing.
3. **Simplicity**: No need to install Chrome, no need for driver binaries, no need to wait for JavaScript to execute.
4. **Reliability**: Browsers crash, update, and change APIs. HTTP is stable and well-understood.

The trade-off is that some websites require JavaScript to render their content. For fanfiction sites, this is rarely the case — they serve their story content as plain HTML. So direct HTTP scraping works perfectly.

### What Makes Scraping Hard?

Scraping sounds simple — fetch a page, read the HTML, pull out data. But in practice, there are a lot of things that can go wrong:

**Websites change their HTML.** A site might redesign its layout, renaming classes or reorganizing elements. Your carefully crafted CSS selectors suddenly stop working. This is the #1 source of maintenance headaches for scrapers.

**Sites have anti-scraping measures.** Rate limiting (returning 429 if you make too many requests), CAPTCHAs, IP blocking, and User-Agent checking are all common. You need to be a good citizen and respect these.

**HTML is messy.** Real-world HTML is full of quirks — unclosed tags, unexpected nesting, invisible characters, Unicode issues. The scraper crate handles most of this, but you'll still encounter edge cases.

**Data formats vary.** Word counts might have commas ("42,000") or not ("42000"). Dates might be ISO format, Unix timestamps, or human-readable strings. Chapter counts might be "3 / 5" or just "5" or "3 of 5".

**Error handling is critical.** A scraper that panics on bad HTML will crash your server. Every extraction needs fallbacks and defaults.

FicHub handles all of these by being defensive: every selector has a fallback, every parse has an `unwrap_or_else`, and every error is caught and converted to a `ScrapeError`.

### Ethical Scraping

Before we go further, let's talk about the ethics of web scraping. When you scrape a website, you're making requests that cost the site's servers money and bandwidth. There are some important rules:

1. **Identify yourself.** Always set a User-Agent header with your project name. Don't pretend to be a browser.
2. **Respect rate limits.** If a site says "slow down" (HTTP 429), slow down. Don't hammer a site with hundreds of requests per second.
3. **Don't scrape login-required content.** If a page requires authentication, you probably shouldn't be scraping it.
4. **Respect robots.txt.** Check if the site has a robots.txt file that says which pages are off-limits.
5. **Cache aggressively.** If you've scraped a page recently, don't scrape it again. Use local caching to minimize requests.

FicHub follows all of these principles. It identifies itself, uses reasonable request rates, and caches results. The goal is to make fanfiction more accessible, not to cause problems for the sites that host it.

### How Different Sites Use HTML

Before we dive into individual scrapers, it's worth understanding that every fanfiction site structures its HTML differently. Here's a quick comparison:

**AO3** uses semantic HTML with clean class names:
```html
<h2 class="title heading">Story Title</h2>
<a rel="author" href="/users/author">Author</a>
<dd class="chapters">3 / 5</dd>
```

**FF.net** uses data attributes and ID selectors:
```html
<div id="profile_top">
  <b class="xcontrast_txt">Story Title</b>
  <a class="xcontrast_txt" href="/u/12345">Author</a>
  <span data-xutitle="word count">42,000</span>
</div>
```

**XenForo** uses a class-heavy structure:
```html
<h1 class="p-title-value">Thread Title</h1>
<a class="username" href="/members/12345/">Author</a>
<article class="message-body">Content here...</article>
```

Each site has its own conventions, naming styles, and HTML patterns. This is why we need separate scrapers — there's no one-size-fits-all approach. But the pattern is always the same: find the right elements, extract the text, handle the edge cases.

### HTTP Requests: The Language of the Web

Before we can scrape anything, we need to understand how the web works at the most basic level. Every time you visit a webpage, your computer speaks a language called **HTTP** — HyperText Transfer Protocol.

There are two main types of HTTP requests:

- **GET**: "Give me this page." This is what your browser does when you type a URL and hit Enter.
- **POST**: "Here's some data — what do you think?" This is what happens when you submit a form.

For scraping, we almost always use GET requests. We're just reading pages, not submitting data.

Every HTTP response comes with a **status code** — a number that tells us what happened:

| Status Code | Meaning | What It Means for Scraping |
|-------------|---------|---------------------------|
| 200 | OK | Success! We got the page. |
| 301 | Moved Permanently | The page moved to a new URL. Follow the redirect. |
| 302 | Found (Redirect) | Temporary redirect. Follow it. |
| 403 | Forbidden | The site blocked us. Stop. |
| 404 | Not Found | The story doesn't exist (or was deleted). |
| 429 | Too Many Requests | We're being too aggressive. Slow down. |
| 500 | Server Error | The site is broken. Try again later. |
| 503 | Service Unavailable | The site is temporarily down. Retry later. |

When FicHub scrapes a site, it needs to handle all of these cases gracefully. A 403 doesn't mean our code is broken — it means the site doesn't want to be scraped right now. We log it, mark the site as "blocked" in our tracking, and move on.

### Making HTTP Requests in Rust

In Rust, we use the **reqwest** crate to make HTTP requests. Here's the simplest possible request:

```rust
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a reusable HTTP client
    let client = reqwest::Client::new();

    // Make a GET request
    let response = client
        .get("https://archiveofourown.org/works/1234567")
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await?;

    // Check the status
    println!("Status: {}", response.status());

    // Get the response body as text
    let html = response.text().await?;
    println!("Got {} bytes of HTML", html.len());

    Ok(())
}
```

Notice the `User-Agent` header. This is important — it tells the website *who* is making the request. Good scraping etiquette means identifying yourself. We use `"fichub.net/0.1.0"` so the website knows it's FicHub, not a malicious bot.

⚠️ **Watch Out**: Always set a User-Agent header. Some websites will block requests that don't have one. It's also polite — if the website admin wants to block your scraper, they know who to contact. Never pretend to be a browser when you're not.

### Request Headers: Talking to Websites Properly

Beyond the User-Agent, there are other headers that matter for scraping. Here's what FicHub sends with its requests:

```rust
let response = client
    .get(&url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

Some sites require additional headers:
- `Accept-Language`: Tells the site what language you prefer
- `Cookie`: Needed for age-restricted content (like on AO3)
- `Accept`: Tells the site what content types you can handle

FicHub keeps its headers minimal — just the User-Agent. This is enough for most sites, and adding more headers can sometimes make you look *more* like a bot, not less.

### The reqwest Client

Notice that in FicHub's code, we always receive a `&reqwest::Client` as a parameter rather than creating a new one. This is important because:

1. **Connection pooling**: The client reuses TCP connections to the same server. If you're fetching 50 chapters from FF.net, you don't want to open 50 separate connections.
2. **Configuration**: We can set timeouts, TLS settings, and proxy configurations in one place.
3. **Performance**: Creating a client has overhead. Creating it once and reusing it is much faster.

Here's how FicHub creates its client:

```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

The 30-second timeout is important. Some fanfiction sites are slow, especially under heavy load. We give them plenty of time, but not infinite time — if a site is truly down, we don't want to wait forever.

### HTML Parsing: Turning Text into Structure

Once we have the HTML as a big string, we need to *parse* it — turn it from flat text into a tree of elements we can search through. This is where the **scraper** crate comes in.

Here's the key idea: HTML is hierarchical. Elements contain other elements. The scraper crate turns this hierarchy into a tree structure:

The HTML:

```html
<div class="story">
  <h2 class="title">My Story</h2>
  <div class="meta">
    <span class="author">Written by Jane</span>
    <span class="words">10,000 words</span>
  </div>
  <div class="content">
    <p>Once upon a time...</p>
  </div>
</div>
```

Gets parsed into a tree structure:

```
div.story
├── h2.title
│   └── "My Story"
├── div.meta
│   ├── span.author
│   │   └── "Written by Jane"
│   └── span.words
│       └── "10,000 words"
└── div.content
    └── p
        └── "Once upon a time..."
```

Once it's a tree, we can use **CSS selectors** to find specific elements — just like you'd use CSS to style a webpage.

### CSS Selectors: Finding Needles in the Haystack

CSS selectors are a mini-language for describing which HTML elements you want. Here are some common patterns:

```rust
use scraper::{Html, Selector};

let html = "<div class='story'><h2>Title</h2><p>Content</p></div>";
let document = Html::parse_document(html);

// Find an element by tag name
let h2 = Selector::parse("h2").unwrap();

// Find by class
let story = Selector::parse("div.story").unwrap();

// Find by ID
let main = Selector::parse("#main").unwrap();

// Find by attribute
let link = Selector::parse("a[href]").unwrap();
```

The pattern is always: `Selector::parse("the-selector")` gives you a `Selector`, and then you use `document.select(&selector)` to get an iterator over matching elements.

💡 **Key Concept**: The `scraper` crate in Rust is named the same thing as the concept of "scraping" — but it's specifically about *parsing HTML*. It doesn't make HTTP requests. You use `reqwest` for the request and `scraper` for parsing the response. Two crates, two jobs, working together.

---

## Chapter 21: The scraper Crate

Now let's go deeper into the scraper crate. This is the tool we'll use for every single site scraper in FicHub. If you understand this crate well, you can build scrapers for any website.

### Setting It Up

Add the scraper crate to your `Cargo.toml`:

```toml
[dependencies]
scraper = "0.19"
```

The scraper crate wraps two things:
- **html5ever**: A parser that turns HTML strings into a document tree. It's written by the Servo team (the same people who built Mozilla's browser engine), so it's very reliable.
- **selectors**: A CSS selector engine that can search that tree efficiently.

### parse_document vs parse_fragment

The scraper crate gives you two ways to parse HTML:

```rust
use scraper::Html;

// Parse a full HTML document (has <html>, <head>, <body> tags)
let doc = Html::parse_document(
    "<html><body><p>Hello</p></body></html>"
);

// Parse just a fragment (a piece of HTML, not a full document)
let frag = Html::parse_fragment(
    "<p>Hello</p><p>World</p>"
);
```

For scraping, we almost always use `parse_document`. The websites we scrape return full HTML documents with `<!DOCTYPE html>`, `<html>`, `<head>`, and `<body>` tags. The fragment parser is useful when you've already extracted a piece of HTML and want to parse just that piece — for example, if you've pulled out the content of a single element and need to parse the HTML inside it.

⚠️ **Watch Out**: If you use `parse_document` on a fragment, html5ever might add extra `<html>` and `<body>` tags around your content. And if you use `parse_fragment` on a full document, it might not handle the DOCTYPE or head section correctly. Use the right one for your situation.

### CSS Selector Syntax in Depth

Let's go through the selector syntax you'll actually use in FicHub scrapers. This is the most important section — you'll use these patterns every day.

**Tag selectors — finding elements by name:**
```rust
Selector::parse("div")      // Any <div> element
Selector::parse("h2")       // Any <h2> element
Selector::parse("a")        // Any <a> (link) element
Selector::parse("p")        // Any <p> (paragraph) element
```

**Class selectors — finding elements by their CSS class:**
```rust
Selector::parse("div.story")      // <div class="story">
Selector::parse("h2.title")       // <h2 class="title">
Selector::parse("a.username")     // <a class="username">
```

Classes are the most common way to select elements on fanfiction sites. AO3 uses classes like `title.heading`, `userstuff`, and `byline`. FF.net uses `xcontrast_txt` and `storytext`.

**ID selectors — finding elements by their unique ID:**
```rust
Selector::parse("#main")          // <div id="main">
Selector::parse("#profile_top")   // <div id="profile_top">
Selector::parse("#chap_select")   // <select id="chap_select">
```

IDs are supposed to be unique on a page, so these selectors are very precise. FF.net uses ID selectors heavily.

**Attribute selectors — finding elements by their HTML attributes:**
```rust
Selector::parse("a[href]")        // Any <a> with an href attribute
Selector::parse("a[rel='author']") // <a rel="author">
Selector::parse("div[data-xutitle='word count']")
                                  // <div data-xutitle="word count">
```

Attribute selectors are powerful for sites that use data attributes or semantic HTML. AO3 uses `rel='author'` on author links, and FF.net uses custom `data-xutitle` attributes.

**Descendant selectors — finding elements inside other elements:**
```rust
Selector::parse("div.story p")     // <p> inside <div class="story">
Selector::parse("#profile_top a")  // <a> inside anything with id="profile_top"
Selector::parse("dd.chapters")     // <dd> with class "chapters"
```

**Combined selectors — the real power:**
```rust
// FF.net uses this: an <a> with class xcontrast_txt inside #profile_top
Selector::parse("#profile_top a.xcontrast_txt")

// AO3: a blockquote with class userstuff (the story summary)
Selector::parse("blockquote.userstuff")

// AO3: the chapter select dropdown's selected option
Selector::parse("select#chap_select option[selected]")
```

### Element Methods: Extracting Data

Once you've found an element, you need to get data out of it. The scraper crate gives you three main methods.

**text()** — Get all the text inside an element (and its children):
```rust
// For <h2 class="title">My Story Title</h2>
let title: String = element.text().collect::<String>();
// title = "My Story Title"

// For <span class="words">42,000 words</span>
let words: String = element.text().collect::<String>();
// words = "42,000 words"
```

The `text()` method returns an iterator over all text nodes inside the element. You collect them into a single string. This strips out all HTML tags and gives you just the text content.

**inner_html()** — Get the HTML inside an element (including child tags):
```rust
// For <blockquote class="userstuff"><p>A great <b>story</b>.</p></blockquote>
let html = element.inner_html();
// html = "<p>A great <b>story</b>.</p>"
```

Use `inner_html()` when you want to preserve HTML formatting. FicHub uses this for story content and descriptions — we want to keep bold, italics, paragraphs, and other formatting.

**attr()** — Get the value of an attribute:
```rust
// For <a href="/users/author123">Author Name</a>
let href = element.value().attr("href");
// href = Some("/users/author123")

// For <a> without href
let href = element.value().attr("href");
// href = None
```

Note that `attr()` returns an `Option<&str>` — it might be `None` if the attribute doesn't exist. Always handle this with `.unwrap_or_default()` or similar.

### The Pattern That Repeats Everywhere

Here's the most important thing to take away from this chapter. This exact pattern shows up hundreds of times in FicHub's scraping code:

```rust
document
    .select(&Selector::parse("some.selector").unwrap())
    .next()                           // Get the first match
    .map(|el| el.text().collect())     // Extract data
    .unwrap_or_else(|| default)        // Handle missing data
```

Or with `inner_html()`:

```rust
document
    .select(&Selector::parse("some.selector").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default()
```

This chain — select, take first, map, default — is the bread and butter of web scraping. You'll write this pattern hundreds of times. Let's trace through what each step does:

1. `.select(&selector)` — Returns an iterator over all matching elements
2. `.next()` — Takes just the first match (returns `Option<Element>`)
3. `.map(|el| ...)` — If we found an element, extract data from it
4. `.unwrap_or_else(...)` — If we didn't find anything, use a default value

### Why trim() Matters

You'll notice we call `.trim()` a lot. That's because HTML often has whitespace around text — newlines, spaces, indentation. The `text()` method gives you exactly what's in the HTML, whitespace included:

```rust
// The HTML might be:
// <h2>
//   My Story Title
// </h2>

// text().collect() gives you "\n  My Story Title\n"
// trim() gives you "My Story Title"
```

Without `trim()`, you'd end up with story titles that have leading newlines and spaces. It's a small detail that makes a big difference in the user experience.

### Building a Complete Parser

Let's put everything together and build a complete parser for a fictional fanfiction page. This shows the full workflow from HTML string to structured data:

```rust
use scraper::{Html, Selector};

struct StoryInfo {
    title: String,
    author: String,
    author_url: String,
    word_count: i64,
    chapter_count: i32,
    description: String,
    status: String,
}

fn parse_story_page(html: &str) -> StoryInfo {
    let document = Html::parse_document(html);

    // Title: might be in h2.title or h1.story-title
    let title = document
        .select(&Selector::parse("h2.title, h1.story-title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());

    // Author: link with rel="author"
    let author_el = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next();

    let author = author_el
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());

    let author_url = author_el
        .and_then(|el| el.value().attr("href"))
        .map(|h| {
            if h.starts_with("http") {
                h.to_string()
            } else {
                format!("https://example.com{h}")
            }
        })
        .unwrap_or_default();

    // Word count: strip commas and parse
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text()
                .collect::<String>()
                .replace(',', "")
                .trim()
                .parse()
                .ok()
        })
        .unwrap_or(0);

    // Chapter count: "3 / 5" → take first number
    let chapters = document
        .select(&Selector::parse("dd.chapters").unwrap())
        .next()
        .and_then(|el| {
            let text = el.text().collect::<String>();
            let before_slash = text.split('/').next()?;
            before_slash.trim().parse().ok()
        })
        .unwrap_or(1);

    // Description: keep HTML formatting
    let description = document
        .select(&Selector::parse("blockquote.description").unwrap())
        .next()
        .map(|el| el.inner_html().trim().to_string())
        .unwrap_or_default();

    // Status: check for completion
    let status = document
        .select(&Selector::parse("dd.status").unwrap())
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| if s.contains("Complete") { "complete" } else { "ongoing" })
        .unwrap_or("ongoing")
        .to_string();

    StoryInfo {
        title,
        author,
        author_url,
        word_count: words,
        chapter_count: chapters,
        description,
        status,
    }
}
```

Notice the pattern: every field follows the same approach — select, first, map, default. The only differences are the CSS selector and how we transform the text. This consistency makes scrapers easy to read and maintain.

Also notice the comma-separated selector `"h2.title, h1.story-title"` — this matches *either* element. It's useful when different page layouts use different tags for the same content.

### Error Recovery in Parsing

Real-world HTML is messy. Here's a more robust parser that handles missing elements gracefully:

```rust
fn extract_with_fallback(
    document: &Html,
    primary: &str,
    fallback: &str,
    attribute: Option<&str>,
) -> String {
    // Try primary selector first
    let result = document
        .select(&Selector::parse(primary).unwrap())
        .next()
        .or_else(|| {
            // Fall back to secondary selector
            document
                .select(&Selector::parse(fallback).unwrap())
                .next()
        })
        .map(|el| {
            if let Some(attr) = attribute {
                el.value().attr(attr).unwrap_or("").to_string()
            } else {
                el.text().collect::<String>().trim().to_string()
            }
        });

    result.unwrap_or_default()
}

// Usage:
let title = extract_with_fallback(
    &document,
    "h2.title.heading",     // Primary: AO3 style
    "h1.story-title",       // Fallback: generic style
    None,                    // No attribute — get text
);

let author_url = extract_with_fallback(
    &document,
    "a[rel='author']",       // Primary: semantic HTML
    "#profile_top a.xcontrast_txt",  // Fallback: FF.net style
    Some("href"),             // Get href attribute
);
```

This function tries one selector, then falls back to another, then falls back to an empty string. It's defensive programming — we always get *something*, even if it's empty.

🧪 **Try It Yourself**: Pick a fanfiction website and write a complete parser function that extracts title, author, word count, and description. Test it with at least three different stories on the same site. Does your parser work for all of them?

---

## Chapter 22: The Scraper Trait

We've learned how to make HTTP requests and parse HTML. Now we need to decide how to organize our scrapers. FicHub supports six different fanfiction sites, and each one works differently. We need a common interface — a contract that every scraper must follow.

That contract is a **trait**.

### What Is a Trait?

In Rust, a trait is like a promise. It says: "Any type that implements this trait must provide these methods." It's how we say "all scrapers must be able to do X, Y, and Z" without caring about the specific scraper.

Think of it like a job description. If you're hiring for "Web Scraper", you don't care if the candidate speaks Python or Rust — you care that they can scrape websites. The trait defines the skills, and each scraper brings its own implementation.

### The SiteScraper Trait

Here's the trait that every FicHub scraper implements:

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
        url: &str,
    ) -> Result<FicMetadata, ScrapeError>;

    /// Fetch all chapters given metadata
    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError>;

    /// Extract structured tags from a fic URL
    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str,
    ) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())  // Default: no tags
    }
}
```

There's a lot going on here, so let's break it down piece by piece.

### The async_trait Macro

Notice the `#[async_trait]` attribute. In Rust, you can't normally have async methods in traits. The `async_trait` crate solves this by transforming async methods into regular methods that return a boxed future. It's one of those things you just use without worrying too much about the implementation details.

The `Send + Sync` bound means scrapers can be shared between threads safely. This is important because FicHub runs multiple scraping tasks concurrently.

### can_handle — "Is This My Job?"

```rust
fn can_handle(&self, url: &str) -> bool;
```

This method takes a URL and returns `true` if the scraper knows how to handle it. For example, the AO3 scraper checks:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

The FF.net scraper checks two domains because FictionPress uses the same code:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

The XenForo scraper checks against a list of domains:

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];

fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

This is the first thing the registry calls — it walks through all scrapers, asking each one "can you handle this URL?" until one says yes.

### lookup — "Tell Me About This Story"

```rust
async fn lookup(
    &self,
    client: &reqwest::Client,
    url: &str,
) -> Result<FicMetadata, ScrapeError>;
```

This method visits the URL, parses the page, and returns a `FicMetadata` struct with everything we need to know about the story: title, author, chapter count, word count, description, and more.

It takes a `reqwest::Client` as a parameter so it can make HTTP requests. We pass the same client to every scraper — this lets us configure timeouts, connection pools, and headers in one place.

The lookup method is usually the simpler of the two — it just reads the first page and pulls out metadata. The hard work happens in `fetch_chapters`.

### fetch_chapters — "Get Me the Story"

```rust
async fn fetch_chapters(
    &self,
    client: &reqwest::Client,
    meta: &FicMetadata,
) -> Result<Vec<Chapter>, ScrapeError>;
```

After we know the metadata, we need the actual story content. This method downloads every chapter and returns them as a vector of `Chapter` structs.

Notice it takes `FicMetadata` as a parameter. This is important — `metadata` contains the story ID, the chapter count, and the source URL. The scraper uses this info to know *which* pages to fetch. For AO3, it constructs a URL with `?view_full_work=true`. For FF.net, it loops from chapter 1 to chapter N.

### extract_tags — "What Are the Tags?"

```rust
async fn extract_tags(
    &self,
    _client: &reqwest::Client,
    _url: &str,
) -> Result<Vec<ExtractedTag>, ScrapeError> {
    Ok(Vec::new())
}
```

This method is optional — it has a default implementation that returns an empty vector. Not all sites expose structured tags (XenForo forums don't, for example). But sites like AO3 have rich tagging systems, and we want to capture those.

By providing a default, we don't force every scraper to implement this. XenForo and AdultFanFiction just use the default. AO3 implements it fully.

### The Data Structs

The trait methods work with a few key data structures. Let's look at each one in detail.

**FicMetadata** — Everything we know about a story:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,          // Deterministic hash ID
    pub title: String,
    pub author: String,
    pub chapters: i32,           // Total chapter count
    pub words: i64,              // Total word count
    pub desc: String,            // Story description/synopsis
    pub published: i64,          // Unix milliseconds
    pub updated: i64,            // Unix milliseconds
    pub status: String,          // "ongoing", "complete", "hiatus"
    pub source: String,          // Original URL
    pub source_id: i64,          // Numeric ID for the source site
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String, // The story ID on the source site
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

This is a big struct, and that's intentional. Every site provides different amounts of information. AO3 gives us word counts and status; XenForo gives us almost nothing. The struct has room for all of it. Fields that a site doesn't provide get sensible defaults (empty strings, zero numbers, `None` for options).

The `url_id` field is the deterministic hash we'll learn about in the next chapter. The `source_id` field tells us which website the story came from (1 for AO3, 2 for FF.net, etc.). The `author_local_id` is the story's ID on the original website — we use this to reconstruct URLs for fetching chapters.

**Chapter** — A single chapter's content:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String, // HTML content
}
```

The `content` field is HTML, not plain text. We keep it as HTML because:
1. We need to convert it to EPUB later, and EPUB is HTML-based
2. Some sites use formatting (bold, italics, blockquotes) in their stories
3. Stripping HTML loses information we might need
4. It's the most faithful representation of the original content

**ExtractedTag** — A tag from a fanfiction site:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}
```

Tags have types: fandom (1), character (2), relationship (3), freeform (4), warning (5), and category (6). The `ExtractedTag` struct has handy constructors that make it easy to create tags:

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
    pub fn category(name: &str) -> Self {
        Self { name: name.to_string(), tag_type_id: 6 }
    }
}
```

Instead of writing `ExtractedTag { name: "Harry Potter".to_string(), tag_type_id: 2 }`, you just write `ExtractedTag::character("Harry Potter")`. Cleaner, less error-prone.

### ScrapeError — When Things Go Wrong

Scraping is fragile. Websites change their layout, go down, block scrapers, or just have weird HTML. We need to handle all these failures:

```rust
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}
```

- **NotFound**: The story doesn't exist (404) or the scraper can't find the story on the page
- **Blocked**: The site returned a 403 or similar — it doesn't want us scraping
- **Network(String)**: Something went wrong with the HTTP request (timeout, connection refused, DNS failure). The string describes the specific error.
- **ParseError(String)**: The HTML came back fine, but we couldn't parse it. Maybe the site changed its layout, or the regex failed to match.

Each variant has a nice `Display` implementation:

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

And it implements the standard `Error` trait:

```rust
impl std::error::Error for ScrapeError {}
```

This means it can be used anywhere Rust expects an error type — in `Result<T, E>`, in error logging, and in the API layer.

💡 **Key Concept**: By using an enum for errors instead of a generic `Box<dyn Error>`, we give our callers clear, structured information about what went wrong. The API layer can respond differently to a 404 (tell the user the story doesn't exist) versus a 503 (tell the user to try again later).

### Writing Your First Scraper from Scratch

Let's walk through creating a complete scraper from scratch. We'll build a scraper for a hypothetical fanfiction site called "StoryArchive." This shows the full thought process.

First, we need to understand the site:
- Stories live at `https://storyarchive.example.com/stories/12345`
- The page has a title in `<h1 class="story-title">`
- Author name is in `<a class="author" href="/authors/456">AuthorName</a>`
- Word count is in `<span class="word-count">42,000 words</span>`
- Story content is in `<div class="story-body">`

Now let's build the scraper:

```rust
pub struct StoryArchiveScraper;

const BASE_URL: &str = "https://storyarchive.example.com";

impl StoryArchiveScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/stories/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for StoryArchiveScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("storyarchive.example.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>
    {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError(
                "could not extract story ID".into()
            ))?;

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
            .select(&Selector::parse("h1.story-title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author_el = document
            .select(&Selector::parse("a.author").unwrap())
            .next();
        let author = author_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        let author_url = author_el
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("{BASE_URL}{h}"))
            .unwrap_or_default();

        let words = document
            .select(&Selector::parse("span.word-count").unwrap())
            .next()
            .and_then(|el| {
                let text = el.text().collect::<String>();
                let cleaned = text.replace(" words", "").replace(",", "");
                cleaned.trim().parse().ok()
            })
            .unwrap_or(0);

        let url_id = crate::scrape::generate_url_id(7, &story_id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words,
            desc: String::new(),
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 7,
            author_id: 0,
            author_url,
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
        -> Result<Vec<Chapter>, ScrapeError>
    {
        let response = client
            .get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content = document
            .select(&Selector::parse("div.story-body").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        if content.is_empty() {
            return Err(ScrapeError::ParseError(
                "no story content found".into()
            ));
        }

        Ok(vec![Chapter {
            chapter_id: 1,
            title: meta.title.clone(),
            content,
        }])
    }
}
```

This is about 100 lines of code. That's typical for a simple scraper. More complex sites (like AO3 with multi-chapter support and tag extraction) need more code, but the structure is always the same.

### When Things Go Wrong: Debugging Scrapers

When your scraper doesn't work, here's a systematic approach:

1. **Fetch the HTML and save it to a file.** Then open the file in a text editor and look at the actual HTML structure.
2. **Use the browser's "Inspect Element" tool.** Right-click on the data you want and look at the HTML. Copy the CSS selector from the browser's Elements panel.
3. **Test your selectors in the Rust console.** Create a small test that parses the saved HTML and tries your selectors.
4. **Check for dynamic content.** If the data isn't in the raw HTML (it's loaded by JavaScript), you'll need a different approach.

The most common mistakes:
- Using `text()` when you need `inner_html()` (or vice versa)
- Forgetting to `.trim()` whitespace
- Using a selector that's too specific (breaks when one class changes) or too broad (matches wrong elements)
- Not handling `Option` properly (`.unwrap()` on a `None` causes a panic)

---

## Chapter 23: URL ID Generation

Every story in FicHub needs a unique identifier. But we can't just use auto-incrementing integers — those are tied to our database. We need an ID that's **deterministic**: given the same story, we always get the same ID, even if we rebuild the database from scratch.

### The Problem

Say you have a story on AO3 with work ID `1234567`. On FF.net, there might be a story with the same numeric ID. And on XenForo forums, thread IDs could overlap with story IDs on other sites.

We need an ID that's unique across *all* sites. And we need it to be stable — if someone scrapes the same story twice (maybe to check for updates), it should get the same ID both times.

If we used auto-incrementing integers, the same story scraped twice would get two different IDs — and our database would have duplicates. That's a disaster for a system that's supposed to be a clean archive.

### The Solution: SHA256 Hashing

FicHub uses SHA256 hashing to generate deterministic IDs. Here's the function:

```rust
use sha2::{Sha256, Digest};

pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    // Use first 12 hex chars for a compact but unique ID
    hex::encode(&result[..6])
}
```

Let's break this down step by step:

1. **Create a hasher**: `Sha256::new()` creates a fresh SHA256 hasher. SHA256 is a cryptographic hash function — it takes any input and produces a 32-byte (256-bit) output that's essentially random-looking but deterministic.

2. **Feed the source ID**: `hasher.update(source_id.to_string().as_bytes())` adds the source ID (like `1` for AO3) to the hash input.

3. **Add a separator**: `hasher.update(b":")` adds a colon. This prevents collisions between "source 1, story 23" and "source 12, story 3" — without the separator, they'd both hash "123".

4. **Feed the story ID**: `hasher.update(story_id.as_bytes())` adds the story's ID on the original website.

5. **Finalize**: `hasher.finalize()` produces the 32-byte hash.

6. **Take first 6 bytes**: `&result[..6]` takes just the first 6 bytes (out of 32).

7. **Encode as hex**: `hex::encode(...)` converts those 6 bytes into 12 hex characters (each byte becomes 2 hex digits).

### Why 12 Hex Characters?

A hex character can be 0-9 or a-f, so there are 16 possibilities per character. With 12 hex characters, that's 16^12 = 281,474,976,710,656 possible IDs — about 281 trillion.

Could there be collisions? Technically yes — but the probability is incredibly small. For a fanfiction archive with millions of stories, the chance of two different stories getting the same 12-character ID is roughly 1 in 281 trillion. The probability of even one collision in a billion stories is about 1 in 281,000 — essentially zero.

Why not use the full 64-character hash? It would be even more unique, but:
- It's harder for humans to read and share
- It takes up more space in the database (12 bytes vs 32 bytes per ID)
- It's more than we need
- URLs and database keys are shorter with 12 characters

12 hex characters gives us a compact, human-friendly ID with essentially zero collision risk.

### How Different Scrapers Use It

Each scraper combines its `source_id` with the story's local ID:

```rust
// AO3: source_id = 1
let url_id = generate_url_id(1, &work_id);

// FF.net: source_id = 2
let url_id = generate_url_id(2, &story_id);

// XenForo: source_id = 3
let url_id = generate_url_id(3, &thread_id);

// AdultFanFiction: source_id = 4
let url_id = generate_url_id(4, &story_id.to_string());

// HP FanFic Archive: source_id = 5
let url_id = generate_url_id(5, &id);
```

The `source_id` ensures that the same numeric ID on different sites produces different hashes. Without it, AO3 story `1234567` and FF.net story `1234567` would have the same hash.

### Tests Prove It Works

The source code includes several tests that verify the hashing:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);  // Same input → same output
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);  // Different source → different ID
}

#[test]
fn test_generate_url_id_different_story_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_456");
    assert_ne!(id1, id2);  // Different story → different ID
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);  // Always 12 characters
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));  // Only 0-9, a-f
}
```

Notice the pattern: we test determinism (same input gives same output), uniqueness (different inputs give different outputs), and format (always 12 hex characters). These tests catch any changes to the hashing logic.

🧪 **Try It Yourself**: Can you calculate the URL ID for AO3 story `789`? You'd compute `generate_url_id(1, "789")`. If you run the function, you'll get a 12-character hex string. Try changing just the source_id to 2 — how does the output change? Try with an empty story ID — does it still produce 12 characters?

### Hash Collision Analysis

Let's think more carefully about collision probability. This is important to understand if you're designing a system that uses hashing for identifiers.

The birthday paradox tells us that with N possible IDs, the probability of at least one collision among K items is approximately:

```
P(collision) ≈ K² / (2N)
```

For our 12 hex character IDs:
- N = 16^12 = 281,474,976,710,656 (about 2.8 × 10^14)
- If we have K = 1,000,000 stories (a large archive): P ≈ 10^12 / (5.6 × 10^14) ≈ 0.0018 or 0.18%
- If we have K = 10,000,000 stories (a massive archive): P ≈ 10^14 / (5.6 × 10^14) ≈ 0.18 or 18%

That 18% for 10 million stories is actually non-trivial! But in practice, fanfiction archives don't have 10 million unique stories across all platforms. And even if there were a collision, we'd detect it when we try to insert a duplicate `url_id` into the database.

If we were worried about collisions at scale, we could increase to 16 or 20 hex characters. But for FicHub's use case, 12 is fine.

### Why Not Use the Story URL Directly?

You might wonder: why not just use the story URL as the ID? It's unique, after all.

The problem is that URLs can change. A site might restructure its URLs (like when FF.net moved from `fanfiction.net/d/12345/` to `fanfiction.net/s/12345/`). If the URL is the ID, a URL change creates a "new" story.

The hash-based ID is **stable** — even if the URL changes, as long as the `source_id` and `story_id` stay the same, the hash stays the same. We can update the URL in the metadata without creating a duplicate.

### Why Not Use Sequential Integers?

Auto-incrementing IDs are simple and fast. But they have problems:

1. **No cross-site uniqueness**: Story 1 on AO3 and story 1 on FF.net would collide
2. **Not deterministic**: If you rebuild the database, stories get different IDs
3. **Exposes information**: Sequential IDs reveal how many stories you have and in what order they were added
4. **Migration problems**: If you add a new site, you'd need to offset all its IDs

The SHA256 approach solves all of these problems at the cost of slightly longer IDs (12 characters vs. an integer).

---

## Chapter 24: AO3 Scraper

Archive of Our Own (AO3) is the biggest fanfiction archive on the internet. It's where most stories live, so getting this scraper right is crucial. Let's walk through how FicHub scrapes AO3.

### Understanding AO3 URLs

AO3 stories have URLs like:
```
https://archiveofourown.org/works/1234567
```

Multi-chapter works can also link to specific chapters:
```
https://archiveofourown.org/works/1234567/chapters/789012
```

The key piece of information is the **work ID** — the number after `/works/`. Every AO3 story has a unique numeric ID.

First, we need to extract that ID from the URL:

```rust
fn extract_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

This uses a regular expression to find `/works/` followed by digits, and captures those digits. It returns `None` if the URL doesn't match.

⚠️ **Watch Out**: AO3 URLs can come in many formats:
- `/works/1234567` (basic)
- `/works/1234567/chapters/789012` (specific chapter)
- `/works/1234567?view_adult=true` (adult content)
- `/works/1234567?view_full_work=true` (all chapters on one page)

Our regex just looks for `/works/(\d+)`, which works for all of these because the work ID always comes right after `/works/`.

### The AO3 Scraper Structure

The AO3 scraper is a simple unit struct:

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

It has no fields because it doesn't need any state. It's a stateless machine — give it a URL, it scrapes the story. The `BASE_URL` constant is used to construct URLs for author profiles and other AO3 pages.

### Scraping the AO3 Page

Here's the full lookup method for the AO3 scraper:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    // Step 1: Extract the work ID from the URL
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError(
            "could not extract work ID from AO3 URL".into()
        ))?;

    // Step 2: Fetch the full work page
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

    // Step 5: Extract metadata (see below)
    // ...
}
```

Notice the `?view_full_work=true` query parameter. This tells AO3 to show all chapters on one page instead of one chapter at a time. This is a huge efficiency win — one HTTP request instead of potentially hundreds.

### Parsing AO3's HTML

AO3 has well-structured HTML with semantic class names. Here's how we extract each piece of metadata:

**Title:**
```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

**Author:**
```rust
let author = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

**Author URL:**
```rust
let author_url = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"))
    .map(|h| format!("{BASE_URL}{h}"))
    .unwrap_or_default();
```

Notice how the author URL uses `.and_then()` instead of `.map()`. This is because `attr()` returns an `Option`, and we're chaining two `Option` operations. If either step returns `None`, the whole chain returns `None`.

**Description:**
```rust
let description = document
    .select(&Selector::parse("blockquote.userstuff").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

Note that for the description, we use `inner_html()` instead of `text()` — we want the HTML formatting (paragraphs, italics, etc.) to be preserved.

**Chapter Count:**
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

The chapter count on AO3 is displayed as "3 / 5" — meaning chapter 3 of 5. We take everything before the `/` to get the total count. If there's no `/`, it's a single-chapter work.

**Word Count:**
```rust
let words = document
    .select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

Notice the `.replace(',', "")`. Word counts on AO3 use commas as thousands separators: "42,000". We strip them before parsing. Also notice the `.ok()` — if parsing fails (because the text isn't a number), we return 0 instead of panicking.

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

### Generating the URL ID

After extracting all the metadata, we generate the deterministic URL ID:

```rust
let url_id = crate::scrape::generate_url_id(1, &work_id);
let now = Utc::now().timestamp_millis();
```

The `source_id` for AO3 is `1`. We also grab the current timestamp for the `published` and `updated` fields (AO3 doesn't expose exact timestamps in a format we can easily parse, so we use "now" as a placeholder).

### Fetching Chapter Content

After we have metadata, we need the actual story content. Here's how the AO3 scraper fetches chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
    let url = format!("{BASE_URL}/works/{}?view_full_work=true",
        meta.author_local_id);

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

    // Handle single-chapter works (no div.chapter found)
    if chapters.is_empty() {
        if let Some(body) = document
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
        {
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

There's a clever fallback at the end: if we don't find any `div.chapter` elements (meaning it's a single-chapter work), we look for `div.userstuff` directly. AO3 uses different HTML structure for one-shots versus multi-chapter works.

### Tag Extraction from AO3

AO3 has the richest tagging system of any fanfiction site. Stories can have fandoms, characters, relationships, warnings, and freeform tags. The AO3 scraper extracts these:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str)
    -> Result<Vec<ExtractedTag>, ScrapeError>
{
    // Fetch and parse the page...

    let mut tags = Vec::new();

    // Fandom tags
    for el in document.select(&Selector::parse("li.fandoms a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        tags.push(ExtractedTag::fandom(&name));
    }

    // Character tags
    for el in document.select(&Selector::parse("li.characters a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        tags.push(ExtractedTag::character(&name));
    }

    // Relationship tags
    for el in document.select(&Selector::parse("li.relationships a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        tags.push(ExtractedTag::relationship(&name));
    }

    // Freeform tags
    for el in document.select(&Selector::parse("li.freeforms a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        tags.push(ExtractedTag::freeform(&name));
    }

    Ok(tags)
}
```

Each tag type gets its own selector and constructor. The result is a list of `ExtractedTag` structs that we can store in the database and use for searching and filtering.

### AO3-Specific Challenges

The AO3 scraper faces some unique challenges:

**Rate limiting.** AO3 has a strict rate limit — too many requests and you'll get temporarily blocked. The scraper uses the `?view_full_work=true` parameter to minimize requests (one request for all chapters instead of N requests). This is a huge efficiency win.

**Age-restricted content.** Some AO3 stories are restricted to logged-in users or require age confirmation. The current scraper doesn't handle authentication, so these stories might return incomplete content. A production system might need cookie support.

**Work skins.** AO3 lets authors customize the appearance of their stories with "work skins" — custom CSS that changes how the story looks. This doesn't affect the HTML structure, but it means the same CSS selector might find different-looking content on different stories.

**Collaborative works.** Some AO3 stories have multiple authors. The scraper extracts only the first author link. For multi-author works, we'd need to handle additional author elements.

These are the kinds of edge cases that make scraping a real-world system more complex than a textbook example. The FicHub scraper handles the common case well and degrades gracefully for edge cases.

🧪 **Try It Yourself**: Pick a story on AO3. Open the page source and find the tags section. Can you identify the HTML class names for fandoms, characters, relationships, and freeform tags? Write the CSS selector for each. How many of each tag type does the story have?

---

## Chapter 25: FF.net and FictionPress

FanFiction.net (FF.net) is one of the oldest fanfiction archives, founded in 1998. FictionPress.com is its sister site for original fiction. They share the same codebase, so FicHub uses the same scraper for both.

### FF.net URL Structure

FF.net stories have URLs like:
```
https://www.fanfiction.net/s/1234567/1/
https://www.fanfiction.net/s/1234567/5/
```

The number after `/s/` is the story ID. The number after that is the chapter number. So `/s/1234567/1/` is chapter 1, and `/s/1234567/5/` is chapter 5.

This is different from AO3! On AO3, you can get all chapters with `?view_full_work=true`. On FF.net, you **must** fetch each chapter separately. A 50-chapter story means 50 HTTP requests.

### Extracting the Story ID

```rust
fn extract_story_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

Simple regex — find `/s/` followed by digits. We use `regex_lite` instead of the full `regex` crate because it's smaller and compiles faster. We don't need advanced regex features for these simple patterns.

### FF.net Lookup

The FF.net page has a different HTML structure than AO3. Here's how we extract the metadata:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError(
            "could not extract story ID from FF.net URL".into()
        ))?;

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

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    // FF.net uses #profile_top for the story info panel
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
    // ... more parsing ...
}
```

Notice FF.net uses ID selectors (`#profile_top`) and data attributes (`[data-xutitle='word count']`). Every site has its own HTML patterns — this is why we need separate scrapers.

FF.net also uses some unusual class names like `xcontrast_txt` and `xgray`. These are FF.net's custom CSS classes — they don't follow any standard naming convention. You just have to inspect the page and figure out what they mean.

### Word Count and Chapter Count on FF.net

FF.net has a clever way of displaying stats. The word count and chapter count are stored in data attributes:

```rust
let words = document
    .select(&Selector::parse(
        "#profile_top span[data-xutitle='word count']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);

let chapters = document
    .select(&Selector::parse(
        "#profile_top span[data-xutitle='chapters']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().trim().split('/').next()
            .and_then(|s| s.trim().parse().ok())
    })
    .unwrap_or(1);
```

The chapter count works similarly to AO3 — it's displayed as "3 / 5", and we take the first part.

### Per-Chapter Fetching

FF.net's biggest difference from AO3 is that you must fetch each chapter separately. Here's how FicHub handles that:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
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

        // Story content is in div.storytext
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        // Chapter title is in the chapter select dropdown
        let title = document
            .select(&Selector::parse(
                "select#chap_select option[selected]").unwrap())
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

⚠️ **Watch Out**: Fetching chapters one-by-one means:
1. **It's slower** — 50 chapters = 50 round-trips to the server
2. **It's more likely to be blocked** — sites notice when you make many requests quickly
3. **It costs more bandwidth** — each page has all the navigation, ads, and boilerplate HTML

This is a fundamental trade-off. We could save bandwidth by only requesting the story content, but FF.net doesn't offer that option. We work with what the site gives us.

### The FictionPress Clone Pattern

Here's one of the cleverest lines in the FicHub codebase:

```rust
/// FictionPress scraper (site shares same structure as FF.net)
pub use FfNetScraper as FictionPressScraper;
```

That's it. One line. FictionPress uses the exact same HTML structure as FF.net (they're sister sites run by the same people), so the same scraper works for both. We just create a type alias.

This means when the registry asks "who handles fictionpress.com?", the `FictionPressScraper` (which is just `FfNetScraper` under a different name) says "I do!"

💡 **Key Concept**: Code reuse through type aliases. When two things are identical, don't duplicate code — just give the same code a second name. This is a pattern you'll see throughout FicHub and in many Rust projects. It's clean, it's DRY, and it's maintainable.

### FF.net Quirks and Anti-Scraping

FF.net is notorious among scrapers for its quirks:

**Chapter titles come from a dropdown.** Unlike AO3 where chapter titles are in heading elements, FF.net stores them in a `<select>` dropdown. The currently selected chapter is marked with the `selected` attribute:

```html
<select id="chap_select">
  <option value="1">Chapter 1 - The Beginning</option>
  <option value="2" selected>Chapter 2 - The Journey</option>
  <option value="3">Chapter 3 - The End</option>
</select>
```

We use `option[selected]` to find the current chapter's title. If you're fetching a specific chapter (like chapter 3), the `selected` option tells you its title.

**Stats are in a specific format.** FF.net shows stats like "Words: 42,000 | Chapters: 5 | Reviews: 100 | Updated: 01/15/2024". The word count and chapter count are in data attributes, not plain text. Our selectors use `[data-xutitle='word count']` and `[data-xutitle='chapters']` to find them.

**FF.net can be slow.** The site often takes several seconds to respond, especially during peak hours. FicHub's 30-second timeout gives it plenty of room, but you might see timeout errors during high-traffic periods.

**Rate limiting is aggressive.** FF.net will temporarily block your IP if you make too many requests too quickly. FicHub's per-chapter fetching for long stories can trigger this. The 30-second timeout helps, but for very long stories, you might want to add delays between chapter requests.

**Content filtering.** FF.net has a mature content filter. Some stories are restricted to logged-in users or require an age confirmation. FicHub's scraper handles the basic case, but age-restricted stories might return incomplete content.

### Handling FF.net's Redirects

FF.net sometimes redirects story URLs. A story at `/s/1234567/` might redirect to `/s/1234567/1/` or vice versa. The reqwest client follows redirects automatically (up to a configurable limit), so this usually works transparently. But if you see unexpected 301/302 responses, check your redirect settings.

---

## Chapter 26: XenForo Forums

Here's where things get interesting. Not all fanfiction lives on dedicated archives. Some of the most popular stories are posted on **XenForo forums** — specifically SpaceBattles, SufficientVelocity, and QuestionableQuesting.

### Forum Posts as Chapters

On a forum, a "story" is a **thread**. The first post is usually the story content, and replies might contain more chapters (marked with special tags), discussion, or both.

This is fundamentally different from AO3 or FF.net, where chapters are clearly defined. On a forum:
- Chapters might span multiple posts
- There's no metadata like word count or tags
- The "title" is the thread title
- The "author" is the thread creator
- There might be discussion mixed in with story content

### XenForo URL Structure

XenForo forum URLs look like:
```
https://forums.spacebattles.com/threads/story-title-here.123456/
https://forums.sufficientvelocity.com/threads/another-story.789012/
https://forum.questionablequesting.com/threads/quest-story.345678/
```

The thread ID is the number after the period in the last segment. Our extraction:

```rust
fn extract_thread_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The regex matches `threads/` followed by any characters, then a dot, then digits (the thread ID). The `/?` at the end makes the trailing slash optional.

### The XenForo Scraper

The XenForo scraper needs to handle all three forums since they share the same XenForo software:

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];

fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

This is a nice pattern — instead of checking for each domain individually, we check against a list. Adding a new XenForo forum is just one line: add the domain to the array.

### Parsing XenForo HTML

XenForo has its own HTML structure, different from both AO3 and FF.net. The thread title is in an `h1`:

```rust
let title = document
    .select(&Selector::parse("h1.p-title-value").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

The author is in a link with class `username`:

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
            // Determine domain from the original URL
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

If the href starts with `/`, we prepend the domain. This turns `/members/author.12345/` into `https://forums.spacebattles.com/members/author.12345/`.

### Posts Become Chapters

The biggest difference with XenForo is how we handle content. Each forum post becomes a chapter:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
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
        return Err(ScrapeError::ParseError(
            "no content found in XenForo thread".into()
        ));
    }

    Ok(chapters)
}
```

The first post gets the thread title as its chapter title. Subsequent posts get numbered titles.

### The XenForo Lookup

The XenForo lookup is simpler than AO3 or FF.net because forums have less metadata:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    let thread_id = Self::extract_thread_id(url)
        .ok_or_else(|| ScrapeError::ParseError(
            "could not extract thread ID".into()
        ))?;

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
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());

    let url_id = crate::scrape::generate_url_id(3, &thread_id);
    let now = Utc::now().timestamp_millis();

    Ok(FicMetadata {
        url_id,
        title,
        author,          // extracted from a.username
        chapters: 1,     // unknown until we fetch
        words: 0,        // unknown
        desc: description,
        published: now,
        updated: now,
        status: "ongoing".to_string(),
        source: url.to_string(),
        source_id: 3,    // XenForo source ID
        author_id: 0,
        author_url,
        author_local_id: thread_id,
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

Notice `chapters: 1` and `words: 0`. We don't know the real values until we fetch the content. The chapters field gets updated as we scrape, and the word count can be calculated from the chapter content.

### Why XenForo Is Different

The XenForo scraper illustrates an important lesson: **every site is different**. There's no "universal scraper" that works everywhere. Each site has:

- Different URL patterns
- Different HTML structures
- Different metadata availability
- Different ways of organizing chapters
- Different levels of anti-scraping protection

This is exactly why we have the `SiteScraper` trait — it gives us a common interface while letting each scraper handle its site's quirks.

⚠️ **Watch Out**: This is a simplification. In reality, XenForo threads often mix story content with non-story posts (discussion, questions, feedback). A more sophisticated scraper would filter posts by the original author or look for story-specific tags like `[Chapter]` or `[Complete]`. The basic scraper just grabs everything — a trade-off between completeness and accuracy.

### Filtering Non-Story Posts

A more advanced XenForo scraper would filter posts to only include story content. Here's how you might do that:

```rust
// Find the thread creator's username
let thread_author = document
    .select(&Selector::parse("a.username").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string());

for article in document.select(&article_sel) {
    // Check if this post is from the thread author
    let post_author = article
        .select(&Selector::parse("a.username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    let is_author = match (&thread_author, &post_author) {
        (Some(ta), Some(pa)) => ta == pa,
        _ => true, // If we can't determine, include it
    };

    if is_author {
        // This is likely story content
        let content = article.inner_html();
        // ... add to chapters
    }
    // Skip posts from other users (discussion, feedback)
}
```

This filters out posts from other users, keeping only the story author's posts. It's not perfect — some authors post discussion in their own threads — but it's much better than including everything.

Another approach is to look for chapter markers. Many XenForo story authors mark their posts with tags like `[Chapter 1]` or `--- Chapter 2 ---`. You can search for these patterns and use them to split content into chapters.

### XenForo Pagination

XenForo threads can span many pages. The first page might have 20 posts, but a long story could have hundreds. To get all the content, you need to fetch additional pages:

```
https://forums.spacebattles.com/threads/story.123456/        (page 1)
https://forums.spacebattles.com/threads/story.123456/page-2   (page 2)
https://forums.spacebattles.com/threads/story.123456/page-3   (page 3)
```

A more complete XenForo scraper would:
1. Fetch the first page
2. Look for pagination links (`a.pageNav-page`)
3. Determine the total number of pages
4. Fetch each additional page
5. Combine all posts from all pages

The current FicHub scraper only fetches the first page — a limitation that means long stories might be incomplete. This is a known trade-off: completeness vs. simplicity and request count.

### XenForo Content Filters

XenForo forums use BBCode-like formatting inside posts. Authors typically wrap their story content in special tags:

```
[Chapter] Chapter 1: The Beginning
The story text goes here...

[Chapter] Chapter 2: The Journey
More story text...
```

A sophisticated scraper could parse these chapter markers and split the content accordingly. The current FicHub scraper doesn't do this — it just grabs the raw HTML of each post. This means chapter boundaries might not align with story chapter boundaries.

For a production system, you'd want to implement chapter detection:
1. Look for `[Chapter]` or `[CHAPTER]` tags in the text
2. Split the post content at these markers
3. Create a separate `Chapter` struct for each section

This is one of those "last 20% of the work takes 80% of the time" situations. The basic scraper works, but making it really good requires understanding the conventions of each community.

🧪 **Try It Yourself**: Visit a SpaceBattles thread with a story. Open the HTML source and find the `article.message-body` elements. How many are on the first page? What does the HTML inside them look like? Can you tell which posts are story content and which are discussion? Look for chapter markers in the text.

---

## Chapter 27: The Scraper Registry

We've built individual scrapers for AO3, FF.net, XenForo, and more. But how does the system know *which* scraper to use for a given URL? That's where the **Scraper Registry** comes in.

### The Registry Pattern

The Scraper Registry is a collection of all available scrapers. When someone asks FicHub to scrape a URL, the registry asks each scraper "can you handle this?" until one says yes.

This is a well-known software design pattern called the **Registry Pattern** (or sometimes the **Service Locator Pattern**). It's like a receptionist at a big office building — you say "I need to see the person who handles AO3 URLs," and the receptionist points you to the right desk.

Here's the complete registry:

```rust
use super::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use super::sites;

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
}
```

Let's unpack what's happening here:

1. We create a `Vec<Box<dyn SiteScraper>>` — a vector of boxed trait objects
2. We add each scraper by wrapping it in `Box::new()`
3. The `dyn SiteScraper` means "any type that implements SiteScraper"

💡 **Key Concept**: `Box<dyn SiteScraper>` is Rust's way of saying "I don't know the concrete type, but it implements the SiteScraper trait." This is called **dynamic dispatch** — the method to call is determined at runtime, not compile time. It's how we can store AO3Scraper and FfNetScraper in the same vector even though they're different types.

### Finding the Right Scraper

The most important method is `find_scraper`:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This walks through the vector, calling `can_handle()` on each scraper, and returns the first one that says yes. If none match, it returns `None`.

It's like a chain of responsibility — each scraper gets a chance to say "this is my job." The order matters: if two scrapers could handle the same URL, the first one in the vector wins. In practice, this doesn't happen because each scraper checks for specific domains.

### The Convenience Methods

The registry provides wrapper methods that combine finding a scraper with calling its methods:

```rust
pub async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    match self.find_scraper(url) {
        Some(scraper) => scraper.lookup(client, url).await,
        None => Err(ScrapeError::NotFound),
    }
}

pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
    match self.find_scraper(&meta.source) {
        Some(scraper) => scraper.fetch_chapters(client, meta).await,
        None => Err(ScrapeError::NotFound),
    }
}
```

The pattern is always:
1. Find the right scraper
2. Call its method
3. If no scraper found, return NotFound

Notice `fetch_chapters` uses `meta.source` (the original URL) to find the scraper, not the current URL. This is important because the metadata might have been modified, but we still need to know which scraper originally produced it.

### The Default Implementation

The registry also implements `Default`, which is a Rust convention for types that can be created with a sensible default configuration:

```rust
impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

This lets you write `ScraperRegistry::default()` instead of `ScraperRegistry::new()`. It's a small convenience, but it's idiomatic Rust.

### Adding a New Scraper

Adding support for a new fanfiction site is straightforward. Here's the step-by-step process:

**Step 1: Create the scraper file.** Add a new file in `src/scrape/sites/`, like `wattpad.rs`:

```rust
pub struct WattpadScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl WattpadScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        // Parse Wattpad URL to get story ID
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for WattpadScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("wattpad.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>
    {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError(
                "could not extract story ID".into()
            ))?;

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

        // Extract Wattpad-specific metadata...
        let title = document
            .select(&Selector::parse("h1.title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let url_id = crate::scrape::generate_url_id(6, &story_id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            // ... other fields ...
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
        -> Result<Vec<Chapter>, ScrapeError>
    {
        // Fetch Wattpad chapters...
        todo!("Implement Wattpad chapter fetching")
    }
}
```

**Step 2: Add it to the sites module.** In `src/scrape/sites/mod.rs`:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod wattpad;  // Add this line
```

**Step 3: Register it.** In `src/scrape/registry.rs`, add one line to `new()`:

```rust
scrapers.push(Box::new(sites::wattpad::WattpadScraper));
```

That's it. Four steps, and you've added a new site. The rest of the system — the API, the database, the EPUB generation — all work automatically because they only interact with `FicMetadata` and `Chapter`, not with site-specific code.

### The sites Module

All the individual scrapers live in the `sites` module:

```rust
// src/scrape/sites/mod.rs
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
```

Each file contains one scraper implementation. The module is just a list of `pub mod` declarations — each scraper file handles its own imports and trait implementation.

### Error Types in Detail

Let's look more closely at the error types and when each one occurs:

**NotFound** — The most common error:
```rust
// When the site returns a non-success status
if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}
```

This covers 404s (page not found), 410s (page removed), and any other non-success status. The caller can decide whether to retry or give up.

**Blocked** — The site actively prevents scraping:
```rust
// When we detect we're being blocked
if response.status().as_u16() == 403 {
    return Err(ScrapeError::Blocked);
}
```

Some sites return 403 for scrapers even though the page exists for browsers. This is a polite way for the site to say "stop scraping us." FicHub respects this and doesn't retry blocked requests.

**Network** — Something went wrong with the connection:
```rust
// Wrapping reqwest errors
let response = client.get(url).send().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

Network errors include timeouts, connection refused, DNS failures, TLS errors, and more. We wrap the error message in the enum so we can log it later.

**ParseError** — We got HTML but couldn't make sense of it:
```rust
// When a regex or CSS selector fails to find expected content
let work_id = Self::extract_work_id(url)
    .ok_or_else(|| ScrapeError::ParseError(
        "could not extract work ID from AO3 URL".into()
    ))?;
```

ParseError often indicates a site has changed its HTML structure. When FicHub starts returning parse errors for a site that used to work, it's a sign the site's layout has changed and the CSS selectors need updating.

⚠️ **Watch Out**: ParseError is the most common maintenance headache. Fanfiction sites change their layouts, add new features, or redesign entirely. When that happens, your CSS selectors stop matching, and you start getting ParseError. You'll need to inspect the new layout and update the selectors.

### The Full Picture

Let's trace a complete request through the system:

```
User sends URL: https://archiveofourown.org/works/1234567
        ↓
Registry.find_scraper("archiveofourown.org/works/1234567")
        ↓
  Ao3Scraper.can_handle() → true!
        ↓
  Ao3Scraper.lookup() → FicMetadata {
      title: "My Story",
      chapters: 5,
      words: 42000,
      ...
  }
        ↓
Registry.fetch_chapters(meta) → [
      Chapter { id: 1, title: "Chapter 1", content: "..." },
      Chapter { id: 2, title: "Chapter 2", content: "..." },
      Chapter { id: 3, title: "Chapter 3", content: "..." },
      Chapter { id: 4, title: "Chapter 4", content: "..." },
      Chapter { id: 5, title: "Chapter 5", content: "..." },
  ]
        ↓
Chapters stored in database
        ↓
EPUB generated on demand
```

The registry is the entry point. It's the dispatcher that connects URLs to scrapers. Without it, the API layer would need to know about every site — with it, the API just says "scrape this URL" and the registry figures out the rest.

### Why the Registry Pattern?

You might wonder: why not just use a match statement? Something like:

```rust
// DON'T do this
if url.contains("archiveofourown.org") {
    let scraper = Ao3Scraper;
    scraper.lookup(client, url).await
} else if url.contains("fanfiction.net") {
    let scraper = FfNetScraper;
    scraper.lookup(client, url).await
} else if url.contains("spacebattles.com") {
    let scraper = XenForoScraper;
    scraper.lookup(client, url).await
}
// ... and so on for every site ...
```

The problem is that this puts all the scraping logic in one place. Each scraper has hundreds of lines of code — CSS selectors, regex patterns, HTML parsing. You don't want all that mixed together in a giant match statement. And every time you add a new site, you'd have to modify this central function.

The registry pattern:
- **Separates concerns**: Each scraper lives in its own file
- **Is extensible**: Adding a site means adding one file and one line to the registry
- **Is testable**: You can test each scraper independently
- **Is maintainable**: When AO3 changes its HTML, you only touch `ao3.rs`
- **Is readable**: The registry is just a list — no nested conditionals

This is a fundamental software design principle: separate things that change independently. Sites change independently, so scrapers should be separate. The registry ties them together without coupling them.

### Scraper Count

The registry even has a utility method:

```rust
pub fn scraper_count(&self) -> usize {
    self.scrapers.len()
}
```

This returns how many scrapers are registered. It's useful for health checks and monitoring — if the count drops below expected, something went wrong.

### Testing Your Scrapers

Testing scrapers is tricky because they depend on external websites. You don't want your tests to fail because a website is down or has changed its layout. Here are three strategies:

**Strategy 1: Save HTML fixtures.** Fetch a page once, save the HTML to a file, and test against the saved file:

```rust
#[test]
fn test_ao3_metadata_parsing() {
    let html = std::fs::read_to_string("tests/fixtures/ao3_story.html")
        .expect("fixture file not found");
    let document = Html::parse_document(&html);

    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    assert_eq!(title, "The Story of a Lifetime");
}
```

**Strategy 2: Mock HTTP responses.** Use a mock HTTP server that returns saved HTML:

```rust
#[tokio::test]
async fn test_ao3_lookup() {
    let mock_server = mockito::Server::new();
    let html = std::fs::read_to_string("tests/fixtures/ao3_story.html").unwrap();

    let mock = mock_server.mock("GET", "/works/1234567")
        .with_status(200)
        .with_body(&html)
        .create();

    let client = reqwest::Client::new();
    let scraper = Ao3Scraper;
    let result = scraper.lookup(&client, &mock_server.url()).await;

    assert!(result.is_ok());
    mock.assert();
}
```

**Strategy 3: Integration tests.** Actually scrape a real website (only in CI, not in unit tests). This catches layout changes but is slow and flaky.

Most FicHub tests use Strategy 1 — HTML fixtures. It's fast, reliable, and catches parsing regressions.

### Performance Considerations

The registry itself is very lightweight — it's just a vector of trait objects. The performance bottleneck is always the HTTP requests, not the registry lookups.

A few performance tips:

**Connection pooling is automatic.** The `reqwest::Client` reuses TCP connections. If you're scraping multiple stories from the same site, the connections are reused.

**HTML parsing is fast.** The scraper crate can parse megabytes of HTML in milliseconds. It's not the bottleneck.

**The real cost is network I/O.** Each HTTP request takes 100ms-3000ms depending on the site. For FF.net stories with many chapters, this adds up quickly.

**Consider concurrent requests.** FicHub could fetch multiple chapters in parallel using `tokio::join!` or `futures::join_all`. The current implementation fetches chapters sequentially, which is simpler but slower.

**Cache aggressively.** If you've already scraped a story, don't scrape it again. Use the database to check if a story exists before hitting the website.

### The Registry as a Design Pattern

The registry pattern we've used here appears in many software systems:

- **Plugin systems**: A web server that loads authentication plugins from a directory
- **Database drivers**: A system that supports multiple databases through a common interface
- **Media format handlers**: A video player that supports different formats through plugins
- **Payment processors**: An e-commerce system that supports multiple payment providers

In each case, the pattern is the same:
1. Define a trait (the interface)
2. Implement the trait for each variant (the plugins)
3. Create a registry that collects all implementations
4. At runtime, find the right implementation for the current task

It's a powerful pattern because it separates *what* needs to be done from *how* it's done. The API layer says "scrape this URL" without knowing anything about AO3 or FF.net. The registry handles the routing. This makes the system extensible without modifying existing code.

🧪 **Try It Yourself**: Think about what sites you'd want to add to FicHub. What's a fanfiction site you use that isn't currently supported? Sketch out the `can_handle()` function and the CSS selectors you'd need for its story pages. How many scrapers would you need to support your favorite reading sites?

---

## Part 4 Summary

In this part, we learned how FicHub reads the internet. This is where the system comes alive — where raw URLs become structured data and stories get pulled from the web into our database.

Here's what we covered across eight chapters:

- **Web scraping basics (Chapter 20)**: What scraping is, how HTTP requests work, what status codes mean, and why we identify ourselves with User-Agent headers. We also talked about ethical scraping and what makes scraping hard in practice.

- **The scraper crate (Chapter 21)**: How to parse HTML into a searchable tree structure using `parse_document`, and how to find elements with CSS selectors. We learned the three essential element methods — `text()` for plain text, `inner_html()` for HTML content, and `attr()` for attributes. And we built a complete parser from scratch to practice the select-map-default pattern.

- **The SiteScraper trait (Chapter 22)**: The common interface every scraper must implement. We explored each method in depth — `can_handle` for URL routing, `lookup` for metadata extraction, `fetch_chapters` for content retrieval, and `extract_tags` for structured tagging. We also learned about the `FicMetadata`, `Chapter`, and `ExtractedTag` data structures, and the `ScrapeError` enum for structured error handling.

- **URL ID generation (Chapter 23)**: How deterministic SHA256 hashing creates unique 12-character IDs from source IDs and story IDs. We analyzed collision probability, discussed why we don't use URLs or sequential integers as IDs, and verified the hashing with comprehensive tests.

- **AO3 scraper (Chapter 24)**: The most complex scraper, handling Archive of Our Own's multi-chapter works, tag extraction, and metadata parsing. We learned about the `?view_full_work=true` parameter that fetches all chapters in one request, and how to extract fandom, character, relationship, and freeform tags.

- **FF.net and FictionPress (Chapter 25)**: Per-chapter fetching for sites that don't offer a "full work" view, the data-attribute selectors that FF.net uses, and the elegant `pub use` type alias that makes FictionPress reuse the FF.net scraper with zero code duplication.

- **XenForo forums (Chapter 26)**: How forum posts become chapters, how to handle multiple XenForo domains with a single scraper, and the challenges of filtering non-story posts from discussion. We explored pagination and content filtering as areas for future improvement.

- **The Scraper Registry (Chapter 27)**: The design pattern that ties everything together — a collection of trait objects that routes URLs to the right scraper. We learned about dynamic dispatch, the `Box<dyn SiteScraper>` pattern, testing strategies (HTML fixtures, mock servers, integration tests), and performance considerations.

### Key Takeaways

If you remember nothing else from this part, remember these three things:

1. **Every site is different.** There's no universal scraper. Each site has its own HTML structure, its own quirks, and its own anti-scraping measures. The trait-based architecture lets us handle this diversity cleanly.

2. **Be defensive.** Every selector might fail. Every parse might return None. Every request might time out. Always have fallbacks, defaults, and error handling.

3. **The registry pattern is powerful.** By separating "which scraper handles this URL" from "how the scraper works," we make the system extensible. Adding a new site is just one file and one line.

### By the Numbers

- **6 scrapers** currently registered: AO3, FF.net, FictionPress, XenForo, AdultFanFiction, HP FanFic Archive
- **4 error types**: NotFound, Blocked, Network, ParseError
- **12 hex characters** per URL ID (281 trillion possible IDs)
- **1 trait** with 4 methods (3 required, 1 optional)
- **1 registry** that routes URLs to scrapers

We now have a complete system for pulling stories from six different fanfiction sites. The registry makes it easy to add more — just implement the trait and register it.

In the next part, we'll learn how to take the HTML content our scrapers collect and transform it into beautiful EPUB files that readers can enjoy on any device.

---

> "We've now built the system that reads the internet — from HTTP requests to CSS selectors, from the SiteScraper trait to the registry that ties it all together. Every fanfiction site is just another scraper, and every story is just data flowing through our code. In the next part, we'll learn how to transform that raw HTML into polished EPUB files."

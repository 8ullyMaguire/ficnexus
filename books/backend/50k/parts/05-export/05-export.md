# Part 5: Turning Stories Into Books

You've built the scraping engine that pulls stories off the internet. You've got metadata — title, author, chapters, words — all neatly organized in `FicMetadata` structs. You even have the chapter content as HTML.

But here's the thing: raw HTML isn't great for reading on a Kindle. Nobody wants to open a web browser to read a fanfiction story on their e-reader. This is where the **export** subsystem comes in. It takes the scraped data and turns it into formats people actually want: EPUB for e-readers, HTML bundles for web browsers, and eventually MOBI and PDF for other devices.

Think of the export system as a bookbinder. You hand it a stack of pages (chapter content) and a cover sheet (metadata), and it puts everything together into a polished, readable book. In this part, we'll learn how FicHub does the binding.

Before we dive in, let's peek at what's inside the `export` module:

```rust
/// Export subsystem: generates EPUB, HTML bundles, and converts to MOBI/PDF
pub mod convert;
pub mod epub;
pub mod html_bundle;
```

Three submodules, three jobs. `epub` makes EPUB files, `html_bundle` makes HTML ZIPs, and `convert` handles converting between formats (like turning an EPUB into a MOBI file using Calibre). Each module is independent — they don't know about each other, they just produce their output. This is the **single responsibility principle** at work: each module does one thing and does it well.

We also have a set of error types that tie everything together:

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    TemplateError(String),
    EpubError(String),
    CalibreError(String),
    ZipError(String),
}

impl std::error::Error for ExportError {}
```

Each variant maps to a different kind of failure. If a file can't be written, that's `IoError`. If the EPUB builder chokes, that's `EpubError`. If the ZIP creation fails, that's `ZipError`. And if Calibre's command-line tool fails during format conversion, that's `CalibreError`. Having specific error types instead of a generic "something went wrong" message makes debugging much easier — you know exactly where the problem is.

Now, let's start with the star of the show: EPUB generation.

---

## Chapter 16: Generating EPUBs

### What Is EPUB?

EPUB stands for **Electronic Publication**. It's the standard format for ebooks — the kind you read on a Kindle, Nook, Kobo, or your phone's reading app. If you've ever downloaded an ebook from a library or bought one from a bookstore, chances are it was an EPUB file.

An EPUB is actually just a **ZIP file** with a special structure inside. Unzip one, and you'll find:

- A `META-INF/container.xml` file that tells readers where to find the table of contents
- An `OEBPS/content.opf` file with metadata (title, author, language)
- XHTML files for each chapter
- A CSS stylesheet
- An optional navigation document

The beauty of EPUB is that it's all plain text — HTML and CSS — so it renders beautifully on devices with tiny screens, big screens, and everything in between. Text reflows to fit the screen, readers can change the font size, and the experience is just... nice.

> 💡 **Key Concept: EPUB Is Just a ZIP of HTML**
>
> If you ever feel intimidated by the EPUB format, remember this: it's literally a ZIP file containing HTML pages and a CSS stylesheet. That's all. The rest is just metadata files that tell the e-reader how to navigate between chapters. If you can write an HTML page, you can understand EPUB.

### The epub-builder Crate

Writing EPUB files by hand would be tedious — you'd have to create all those XML manifest files, the container structure, and make sure every path and filename is exactly right. Fortunately, there's a Rust crate for that.

The `epub-builder` crate provides a high-level API for creating EPUB files. You tell it the metadata and the chapter contents, and it handles all the low-level EPUB structure for you. Add it to your `Cargo.toml`:

```toml
[dependencies]
epub-builder = "0.7"
```

The crate gives us two main pieces:

- **`EpubBuilder`** — A builder that accumulates metadata and content
- **`ZipLibrary`** — The ZIP backend that EPUB files need internally

Here's how you start building an EPUB:

```rust
use epub_builder::{EpubBuilder, ZipLibrary};

// Create the builder with the ZIP backend
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

// Set the metadata
builder.metadata("title", "The Best Story Ever")?;
builder.metadata("author", "JaneDoe")?;
builder.metadata("lang", "en")?;
```

That's it for metadata. The builder remembers everything you tell it, and when you call `generate()`, it assembles the complete EPUB structure.

### Adding Chapter Content

Each chapter is an XHTML page. The `epub-builder` crate expects you to pass chapters as bytes along with a title:

```rust
use epub_builder::EpubContent;

// Each chapter is an XHTML document
let chapter_html = r#"
<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body>
    <h2>Chapter 1: The Beginning</h2>
    <p>Once upon a time, in a land far away...</p>
</body>
</html>
"#;

builder.add_content(
    EpubContent::new("chapter_1.xhtml", chapter_html.as_bytes())
        .title("Chapter 1: The Beginning"),
)?;
```

The `.title()` call is important — it sets the chapter title that shows up in the e-reader's table of contents. Without it, your chapter would just show up as a blank entry.

### The create_epub Function

Now let's look at FicHub's actual EPUB generation function. It takes three inputs: the fic metadata, the chapters, and a temporary directory to work in:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;

pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Step 1: Create a temporary work directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // Step 2: Set up the builder with metadata
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;

    // Step 3: Add inline CSS for the reading experience
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0.5em 0;}"
    );
    builder.stylesheet(css.as_bytes())?;

    // Step 4: Add an introduction page with story metadata
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

    builder.add_content(
        EpubContent::new("introduction.xhtml", intro_html.as_bytes())
            .title("Introduction"),
    )?;

    // Step 5: Add each chapter
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

    // Step 6: Write the EPUB file
    let epub_path = work_dir.join("output.epub");
    let file = fs::File::create(&epub_path)?;
    builder.generate(file)?;

    // Step 7: Compute MD5 hash for cache key
    let epub_data = fs::read(&epub_path)?;
    let md5_hex = Md5::digest(&epub_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((epub_path, md5_hex))
}
```

That's a lot of code, so let's break it down.

### Why a UUID Work Directory?

Notice Step 1 creates a directory with a random UUID name. This is important because FicHub is a web server that handles many requests simultaneously. If two people ask for EPUBs at the same time, each one needs its own workspace. UUIDs guarantee no collisions — the odds of two identical UUIDs are so small that you'd win the lottery multiple times first.

The work directory is inside a temporary directory (`tmp_dir`) that gets cleaned up periodically. We're not hoarding temporary files forever.

### The Introduction Page

Step 4 creates a special first page — the introduction. This page isn't a story chapter; it's a summary page that shows the title, author, word count, and other metadata. When you open an EPUB on your Kindle, this is the first thing you see. It's like the title page of a physical book.

Notice the `escape_html` helper function. This is crucial for security and correctness:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

Without escaping, a story title like `He said "She's <amazing>"` would break the HTML. The `<amazing>` part would be interpreted as an HTML tag, not as text. The escaping function turns special characters into their HTML-safe equivalents.

This pattern — escaping user content before inserting it into HTML — is one of the most important security practices in web development. Without it, you're vulnerable to **XSS (Cross-Site Scripting)** attacks, where a malicious story title could inject JavaScript into the EPUB. Since EPUB readers render HTML, this is a real concern.

### The Stylesheet: Making It Look Like a Book

The EPUB includes a minimal CSS stylesheet:

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

This does three things:

1. **`font-family:serif`** — Uses a serif font (like Times New Roman) for a book-like feel
2. **`h2{text-align:center}`** — Centers chapter titles
3. **`p{margin:0.5em 0}`** — Adds spacing between paragraphs

It's minimal on purpose. E-readers have their own font settings, dark mode, and text size controls. We don't want to fight those features with aggressive styling. The CSS just provides sensible defaults that the reader can override.

The `concat!` macro joins the three CSS rules into a single string at compile time — no runtime cost at all. It's a tiny optimization, but it's the kind of thing Rust makes effortless.

### Why MD5 for the Hash?

Step 7 computes an MD5 hash of the generated EPUB file. This hash serves as a **cache key** — a unique identifier for this specific EPUB. If the same story is requested again with the same chapters, the hash will be identical, and we can skip regenerating the file.

MD5 is fast and produces consistent results. It's not cryptographically secure (don't use it for passwords!), but for caching purposes, it's perfect. The same input always produces the same hash, and even a tiny change in the content produces a completely different hash.

> ⚠️ **Watch Out: async and File I/O**
>
> Notice that `create_epub` is marked `async` but does synchronous file I/O with `fs::File::create` and `fs::read`. In a production server, you'd ideally use async I/O (`tokio::fs`) to avoid blocking the runtime. FicHub makes a pragmatic choice here — EPUB generation is fast enough that the blocking I/O doesn't cause noticeable delays. But if you're building something that needs to handle thousands of concurrent exports, consider switching to `tokio::fs`.

### The Return Value

The function returns a tuple: `(PathBuf, String)`. The `PathBuf` is the path to the generated EPUB file, and the `String` is the MD5 hex hash. The caller (the export handler) will use the hash to determine where to store the file in the cache.

> 🧪 **Try It Yourself**
>
> 1. Add the `epub-builder`, `md5`, and `uuid` crates to a new Rust project
> 2. Create a `FicMetadata` struct with a test title and author
> 3. Create a few `Chapter` structs with simple HTML content
> 4. Call `create_epub` and verify that the output file is a valid EPUB (rename it to `.zip` and unzip it!)
> 5. Look inside the ZIP — can you find the XHTML chapter files and the CSS?

---

## Chapter 17: HTML Bundles

### What Is an HTML Bundle?

Not everyone has a Kindle. Some people prefer reading in their web browser, or they want to share a story link with a friend who can open it on any device. An **HTML bundle** is a ZIP file containing one or more HTML files that make up a complete, self-contained story.

The idea is simple: put everything into a single ZIP file. Inside, you'll find:

- `index.html` — A complete HTML page with the entire story, including navigation, metadata, and all chapters

When someone downloads this ZIP and opens `index.html`, they get a beautifully formatted story page they can read in any web browser. No special apps needed. No e-reader required.

### The zip Crate for Rust

Just like EPUBs are ZIP files under the hood, HTML bundles are explicitly ZIP archives. The `zip` crate gives us Rust-native ZIP file creation:

```toml
[dependencies]
zip = "2"
```

Here's the basic usage:

```rust
use zip::ZipWriter;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use std::fs::File;
use std::io::Write;

let file = File::create("bundle.zip")?;
let mut zip = ZipWriter::new(file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

// Add a file to the archive
zip.start_file("index.html", options)?;
zip.write_all(b"<html>...</html>")?;

zip.finish()?;
```

The `Deflated` compression method reduces the file size without losing any data — it's the same algorithm used in regular ZIP files. The `unix_permissions` setting ensures the file is readable when extracted on Linux or macOS.

### Building the HTML Content

FicHub's HTML bundles create one big HTML page containing all chapters. Let's look at how the chapter navigation and content are assembled:

```rust
let mut chapters_nav = String::new();
let mut chapters_content = String::new();

for chapter in chapters {
    // Build a navigation link for each chapter
    chapters_nav.push_str(&format!(
        r##"<li><a href="#ch{ch}">{title}</a></li>"##,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
    ));

    // Build the chapter content section
    chapters_content.push_str(&format!(
        r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
        content = chapter.content,
    ));
}
```

Notice the `#ch{ch}` anchors in the navigation links. These are HTML **anchor links** — when you click a chapter in the navigation, the browser jumps to that chapter's heading on the same page. It's like a table of contents that scrolls you to the right spot.

### The create_html_bundle Function

Here's the full function that FicHub uses:

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Work directory (same UUID pattern as EPUB)
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // Build navigation and content (shown above)
    let mut chapters_nav = String::new();
    let mut chapters_content = String::new();

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

    // Assemble the full HTML page
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: Georgia, 'Times New Roman', serif;
            line-height: 1.7;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; }}
        h2 {{
            text-align: center;
            margin: 1.5em 0 0.5em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; }}
        .nav {{ background: #fff; border: 1px solid #ddd; padding: 1em; margin: 1.5em 0; }}
        .nav ul {{ list-style: none; columns: 2; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <div class="meta">
        <p>by <strong>{author}</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>{words}</td></tr>
            <tr><td>Chapters:</td><td>{chapters}</td></tr>
            <tr><td>Status:</td><td>{status}</td></tr>
            <tr><td>Published:</td><td>{published}</td></tr>
            <tr><td>Updated:</td><td>{updated}</td></tr>
        </table>
    </div>
    <hr/>
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>{nav}</ul>
    </div>
    <hr/>
    <div class="content">{content}</div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — {source}</p>
    </div>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        words = meta.words,
        chapters = meta.chapters,
        status = escape_html(&meta.status),
        published = format_timestamp(meta.published),
        updated = format_timestamp(meta.updated),
        desc_escaped = escape_html(&meta.desc),
        nav = chapters_nav,
        content = chapters_content,
        source = escape_html(&meta.source),
    );

    // Write the HTML file
    let html_path = work_dir.join("index.html");
    fs::write(&html_path, &html)?;

    // Bundle it into a ZIP
    let zip_path = work_dir.join("bundle.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip.start_file("index.html", options)?;
    zip.write_all(html.as_bytes())?;
    zip.finish()?;

    // Compute hash for caching
    let zip_data = fs::read(&zip_path)?;
    let md5_hex = Md5::digest(&zip_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((zip_path, md5_hex))
}
```

### Why ZIP the HTML?

You might wonder: if the goal is a single HTML page, why put it in a ZIP at all? Three reasons:

1. **Consistent download experience.** When a web browser downloads a `.zip` file, it automatically offers to save it. When it downloads a raw `.html` file, it might try to open it in a new tab instead, which isn't what we want.

2. **Smaller file size.** HTML is very compressible — a 500KB HTML page might compress to 100KB in a ZIP. This matters when stories have lots of content.

3. **Future-proofing.** The ZIP container makes it easy to add more files later — multiple HTML files, images, or metadata files — without changing the download format.

### The Reading Experience

The HTML bundle includes a carefully designed CSS stylesheet that makes the story comfortable to read. Let's look at a few key pieces:

```css
body {
    font-family: Georgia, 'Times New Roman', serif;
    line-height: 1.7;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}
```

The `max-width: 800px` with `margin: 0 auto` centers the content and prevents lines from getting too long. Long lines are hard to read — your eyes have to travel too far from the end of one line to the start of the next. The `line-height: 1.7` adds generous spacing between lines, making the text easier to scan.

The navigation section uses a two-column layout:

```css
.nav ul { list-style: none; columns: 2; }
```

This puts chapter links in two columns, so a 30-chapter story doesn't create a massive scrolling list. The chapter headings include `border-bottom` separators, and each chapter has a "back to top" link for easy navigation.

These details might seem minor, but they make the difference between a story that's pleasant to read and one that gives you a headache after five minutes. Good typography isn't about fancy fonts — it's about spacing, line length, and contrast.

> ⚠️ **Watch Out: Escaping in the HTML Bundle**
>
> Notice that the HTML bundle escapes metadata but **not** chapter content. This is intentional — the chapter content is already HTML that was scraped from the original site. If we escaped it, all the formatting (bold, italics, blockquotes, links) would be destroyed. But the metadata (title, author, description) comes from structured data that might contain characters like `<` or `&`, so it must be escaped.

> 💡 **Key Concept: The Double `{{` in Format Strings**
>
> Did you notice the `{{` and `}}` in the CSS string? That's because we're using Rust's `format!()` macro, which uses `{}` for substitution. If we want actual curly braces in the output (like CSS rules), we double them: `{{` becomes `{`. It's one of those "once you know it, you never forget it" things.

> 🧪 **Try It Yourself**
>
> 1. Create a simple HTML page with a title, some paragraphs, and a table of contents
> 2. Wrap it in a ZIP file using the `zip` crate
> 3. Extract the ZIP and verify the HTML renders correctly in a browser
> 4. Try adding a second HTML file to the ZIP — what happens when you open it?

---

## Chapter 18: The Disk Cache

### Why Cache?

Imagine someone asks FicHub for an EPUB of a popular story. The server scrapes the website, downloads every chapter, generates the EPUB, computes a hash, and returns the file. Great!

Now imagine 100 people ask for the same story. Without caching, FicHub would scrape that story 100 times, generate 100 identical EPUBs, and waste a ton of time and bandwidth. That's like going to the library to check out the same book 100 times instead of keeping a copy on your shelf.

A **cache** is a storage area where we keep previously generated files. When someone asks for a story we've already processed, we just hand them the cached file instead of regenerating it. It's the single biggest performance win in the entire system.

> 💡 **Key Concept: Cache Hit vs. Cache Miss**
>
> When someone requests a file that's already in the cache, that's a **cache hit** — fast and easy. When the file isn't cached yet, that's a **cache miss** — we have to generate it and store it. The goal is to make cache hits as fast as possible and minimize the cost of cache misses.

### The EType Enum

FicHub supports multiple output formats, and each format has its own cache. The `EType` enum represents these export types:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}
```

Each variant knows three things about itself:

```rust
impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",  // HTML bundles are ZIP files!
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }

    pub fn version(&self) -> i32 {
        match self {
            EType::Epub => 1,
            EType::Html => 1,
            EType::Mobi => 0,
            EType::Pdf => 0,
        }
    }
}
```

Notice something interesting: `EType::Html` has suffix `.zip`, not `.html`. That's because HTML bundles are ZIP archives containing HTML files — the actual file extension reflects the container, not the content.

The `version()` method returns a version number for each format. This is used in cache versioning — more on that shortly.

### Cache Path Structure

Where do cached files live? FicHub uses a hierarchical directory structure to avoid putting millions of files in one folder:

```
cache_root/
├── epub/
│   ├── abc/
│   │   ├── def/
│   │   │   ├── abcdef123/
│   │   │   │   ├── a1b2c3d4e5f6.epub
│   │   │   │   └── f6e5d4c3b2a1.epub
│   │   │   └── ...
│   │   └── ...
│   └── ...
├── html/
│   └── ...
└── mobi/
    └── ...
```

Each URL gets a unique ID (`url_id`), and the cache splits that ID into 3-character directory chunks. So for `url_id = "abcdefghijklm"`, the path becomes:

```
cache_root/epub/abc/def/ghi/abcdefghijklm/<hash>.epub
```

Here's the function that computes the cache path:

```rust
pub fn cache_path(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    hash: &str,
) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    // Split url_id into 3-char directory chunks
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    // Full url_id directory
    path = path.join(url_id);

    // File: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

The `.take(3)` ensures we create at most 3 levels of subdirectories (up to 9 characters of the ID). This keeps the directory tree manageable while preventing any single folder from having too many entries.

> ⚠️ **Watch Out: Filesystem Limits**
>
> Most filesystems get slow when a single directory has tens of thousands of files. The 3-character prefix splitting prevents this. If you're designing your own cache, always think about directory fanout — how many subdirectories and files end up in each folder.

### Moving Files to Cache

After generating an EPUB or HTML bundle, the file lives in the temporary work directory. We need to move it to its permanent cache location:

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    // Create any missing parent directories
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    // Move (rename) the file
    fs::rename(source, dest)?;
    Ok(())
}
```

This is simple but important. `fs::rename` is an atomic operation on most filesystems — the file either moves completely or not at all. There's no risk of a half-copied file. The `create_dir_all` ensures the cache directory structure exists before we try to move the file there.

### Cache Versioning

Sometimes the cache needs to be invalidated — when the export code changes, when a format's template is updated, or when the underlying story data changes. FicHub uses **version numbers** to handle this:

```rust
pub fn compute_version(
    export_version: i32,
    etype_version: i32,
    fic_version_bump: i32,
) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

Three components contribute to the version:

- **`export_version`** — Bumped when the export code itself changes (new templates, layout fixes). Think of it as the "EPUB generator version."
- **`etype_version`** — Bumped when a specific format's template changes. EPUB and HTML have their own version numbers.
- **`fic_version_bump`** — Bumped when the underlying story data is re-scraped and updated.

The version is stored alongside the cached file in the database. When FicHub checks the cache, it compares the stored version with the current version. If they don't match, the cached file is stale and needs to be regenerated.

> 💡 **Key Concept: Three-Layer Versioning**
>
> Imagine you change the EPUB font from Times New Roman to Georgia. That's an `export_version` bump — every cached EPUB is now stale. But the HTML bundles still look fine, so they don't need regeneration. By separating the version into three layers, you can invalidate exactly what needs to change without throwing away everything.

### Cache Semaphores

Here's a subtle but critical problem: what if two people request the same story at exactly the same time, and neither EPUB is cached yet? Without protection, both requests would start generating the EPUB simultaneously — wasting resources and potentially creating race conditions where both try to write the same file.

FicHub solves this with **semaphores** — a concurrency primitive that limits how many tasks can run at the same time:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

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

Each `(url_id, etype)` pair gets its own semaphore with a limit of 1. When a request wants to generate an EPUB, it "acquires" the semaphore. If another request is already generating the same EPUB, the second request waits. Once the first request finishes and releases the semaphore, the second request gets its turn — but by then, the cache is already populated, so it finds a cache hit!

Let's trace through this step by step:

1. **Request A** arrives for `url_id = "abc123"`, type `Epub`
2. It acquires the semaphore (now at 0 permits — locked)
3. It starts generating the EPUB (this takes a few seconds)
4. **Request B** arrives for the same `url_id = "abc123"`, type `Epub`
5. It tries to acquire the semaphore — but it's locked, so Request B waits
6. Request A finishes, moves the file to cache, releases the semaphore
7. Request B acquires the semaphore (now at 1 permit)
8. Request B checks the cache again — **cache hit!** Returns immediately

Notice that the semaphore is per `(url_id, etype)` pair. Request A generating an EPUB doesn't block Request C from generating an HTML bundle for a completely different story. The semaphores are fine-grained — they only block concurrent work on the same file.

This pattern is called **double-check locking**: check the cache, acquire the lock, check the cache again. It's a well-known concurrency pattern that prevents duplicate work while keeping lock contention low. The first cache check avoids locking entirely for cache hits (the common case). The lock is only acquired when there's actual work to do.

> 🧪 **Try It Yourself**
>
> 1. Create a test directory with subdirectories at `cache/epub/abc/def/abcdef/`
> 2. Write a file there with `fs::write`
> 3. Write a function `cache_exists(path) -> bool` that checks if a cached file exists
> 4. Try the `move_to_cache` function: create a temp file, move it to the cache path, and verify it moved correctly
> 5. What happens if you try to move a file that doesn't exist?

---

## Chapter 19: The Export Handler

### The Export Handler

Now it's time to wire them together into a handler — the function that Axum calls when someone hits the `/api/v0/epub` endpoint.

The flow looks like this:

```
User sends: GET /api/v0/epub?q=https://archiveofourown.org/works/123456
         │
         ▼
   ┌─────────────┐
   │ Validate URL │
   └──────┬──────┘
          │
          ▼
   ┌─────────────────┐
   │ Lookup metadata  │  ← scraper.lookup()
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Check cache      │  ← cache hit? Return URLs immediately
   └──────┬──────────┘
          │ cache miss
          ▼
   ┌─────────────────┐
   │ Acquire semaphore│  ← prevent duplicate work
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Check cache      │  ← double-check (someone else might have finished)
   └──────┬──────────┘
          │ still a miss
          ▼
   ┌─────────────────┐
   │ Fetch chapters   │  ← scraper.fetch_chapters()
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Generate EPUB    │  ← export::epub::create_epub()
   │ Generate HTML    │  ← export::html_bundle::create_html_bundle()
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Move to cache    │  ← cache::disk::move_to_cache()
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Return JSON      │  ← with download URLs and metadata
   └─────────────────┘
```

### The Export Handler Function

Here's the handler, simplified for teaching but faithful to the real code:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    // Step 1: Validate the query parameter
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        })));
    }
```

The handler starts by extracting and validating the `q` parameter — the URL of the story to export. If it's empty, we return an error immediately. Notice the `?` pattern isn't used here — we return a valid JSON response with `err: -1` instead of an HTTP error. This is a FicHub convention: API errors are communicated via the `err` field in the JSON, not HTTP status codes. The HTTP status is always 200, and the client checks `err` to see if something went wrong.

```rust
    // Step 2: Block automated requests
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({
            "err": -10,
            "msg": "automated requests blocked"
        })));
    }
```

The `automated` parameter is a bot-detection mechanism. If someone sends requests with `automated=true`, we reject them. This is a polite way to discourage scripts from hammering the server — real users don't set this flag.

```rust
    // Step 3: Find the right scraper for this URL
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(
            -5,
            format!("unsupported URL: {}", query),
        ))?;
```

Here's where the scraper registry from Part 4 comes in. We pass the URL to `find_scraper`, and it returns the appropriate scraper (AO3, FF.net, etc.) or `None` if the URL isn't supported. The `?` operator converts the `None` into an `AppError::BadRequest`, which Axum turns into an HTTP 400 response.

```rust
    // Step 4: Look up the story metadata
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Step 5: Check the cache
    let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
        .await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;
    let input_hash = meta.content_hash.clone()
        .unwrap_or_else(|| "upstream".to_string());

    let cached = queries::find_export_log(
        &state.db, &meta.url_id, version, "epub", &input_hash,
    ).await?;
```

Step 4 calls the scraper to get the story metadata. This is the first network request — fetching the story's title, author, word count, and other info from the source website.

Step 5 checks the cache. The cache lookup uses four pieces of information: the story's `url_id`, the current export version, the format ("epub"), and the content hash. The `content_hash` comes from the scraper — it's a hash of the story's content on the source site. If the content hasn't changed, the hash stays the same, and we can reuse the cached export.

The `version_bump` is fetched from the database — it's a per-story counter that gets incremented when the export code changes in a way that affects this specific story. Combined with `export_version` (the global version), this gives us a complete version for cache validation.
    if let Some(export_log) = cached {
        // Cache hit! Build URLs from the cached hash
        let epub_hash = &export_log.export_hash;
        let slug = generate_slug(&meta.title, &meta.url_id);
        let (info_str, notes) = build_info_string(&meta);

        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "info": info_str,
            "url_id": meta.url_id,
            "slug": slug,
            "meta": build_meta_json(&meta),
            "epub_url": format!(
                "/cache/epub/{}?h={}", meta.url_id, epub_hash
            ),
            "html_url": null,
        })));
    }

    // Cache miss — acquire semaphore for this story
    let sem = cache::get_export_semaphore(
        &state.cache_semaphores, &meta.url_id, &EType::Epub,
    ).await;
    let _permit = sem.acquire().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Double-check: another request might have finished while we waited
    let cached = queries::find_export_log(
        &state.db, &meta.url_id, version, "epub", &input_hash,
    ).await?;
    if let Some(export_log) = cached {
        let epub_hash = &export_log.export_hash;
        let slug = generate_slug(&meta.title, &meta.url_id);
        let (info_str, notes) = build_info_string(&meta);

        return Ok(Json(json!({
            "err": 0,
            "q": query,
            "info": info_str,
            "url_id": meta.url_id,
            "slug": slug,
            "meta": build_meta_json(&meta),
            "epub_url": format!(
                "/cache/epub/{}?h={}", meta.url_id, epub_hash
            ),
            "html_url": null,
        })));
    }

    // Still a cache miss — generate everything
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Generate EPUB
    let (epub_path, epub_hash) = export::epub::create_epub(
        &meta, &chapters, &state.config.tmp_dir,
    ).await
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Move EPUB to cache
    let cache_dest = cache::disk::cache_path(
        &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash,
    );
    cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    // Record in database so future requests find the cache
    queries::insert_export_log(
        &state.db, &meta.url_id, version, "epub",
        &input_hash, &epub_hash,
    ).await?;
```

The cache miss path is where all the real work happens. First, `fetch_chapters` downloads every chapter's HTML content from the source website. For a 50-chapter story, this might involve 50 HTTP requests — each one fetching a chapter page, parsing the HTML, and extracting the story text.

Then `create_epub` takes the metadata and chapters and produces the EPUB file. The result is a tuple: the path to the file and its MD5 hash. The hash becomes part of the cache key — it uniquely identifies this specific version of the EPUB.

`move_to_cache` moves the file from the temporary work directory to its permanent home in the cache directory. This is an atomic move (rename), so there's no risk of a partial file being served.

Finally, `insert_export_log` records the export in the database. This is the crucial step that makes the cache work for future requests. The log entry stores the `url_id`, version, format, input hash, and output hash. When the next request comes in, `find_export_log` will find this entry and return the cached URLs.

```rust
    // Generate HTML bundle (same chapters, different format)
    let (html_path, html_hash) = export::html_bundle::create_html_bundle(
        &meta, &chapters, &state.config.tmp_dir,
    ).await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    let html_cache_dest = cache::disk::cache_path(
        &state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash,
    );
    cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
```
    // Build and return the JSON response
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "info": info_str,
        "url_id": meta.url_id,
        "slug": slug,
        "meta": build_meta_json(&meta),
        "epub_url": format!(
            "/cache/epub/{}?h={}", meta.url_id, epub_hash
        ),
        "html_url": format!(
            "/cache/html/{}?h={}", meta.url_id, html_hash
        ),
    })))
}
```

### Building the JSON Response

Every response from FicHub's API is JSON. The `serde_json::json!` macro makes it easy to build structured responses:

```rust
use serde_json::{json, Value};

let response = json!({
    "err": 0,          // 0 means success, negative means error
    "q": query,        // The original URL that was requested
    "info": info_str,  // Human-readable summary
    "url_id": meta.url_id,  // Unique ID for this story
    "slug": slug,      // URL-friendly name for the story
    "meta": build_meta_json(&meta),  // Full metadata object
    "epub_url": "/cache/epub/abc123?h=def456",
    "html_url": "/cache/html/abc123?h=ghi789",
});
```

The `err` field is FicHub's convention: `0` means everything went fine, and negative numbers indicate different error types. This makes it easy for frontend clients to check if the request succeeded with a simple `if response["err"] == 0` comparison.

### The generate_slug Function

A **slug** is a URL-friendly version of the story title. It's the kind of thing you'd see in a URL:

```rust
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Collapse multiple underscores
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();

    format!("{}-{}", slug, url_id)
}
```

For a story titled `"The Best Story Ever (Part 1/2)"` with url_id `"abc123"`, this produces:

```
The_Best_Story_Ever_Part_1_2-abc123
```

The slug serves two purposes:

1. **Human readability.** Instead of remembering `abc123`, users see something that tells them what the story is.
2. **Download naming.** The slug becomes the suggested filename when downloading the EPUB, so users get a meaningful filename instead of a hash.

### The build_info_string Function

This function creates a friendly summary that's shown to users — like the text you'd see on a book's back cover:

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

    let updated_str = chrono::DateTime::from_timestamp_millis(meta.updated)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string());

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

This returns a tuple: the info string and a vector of notes. The notes are empty in the normal case, but for special situations (like greylisted fics), they can contain warning messages.

The relative time calculation (`"5 minutes ago"`, `"3 days ago"`) makes the information feel current and alive, rather than just showing a raw timestamp that nobody wants to decode.

### Putting It All Together

The export handler is where everything connects. Let's trace through a complete request:

1. Someone sends `GET /api/v0/epub?q=https://archiveofourown.org/works/123456`
2. The handler validates the query and finds the AO3 scraper from the registry
3. The scraper fetches the story metadata from AO3's website
4. The handler checks the database for a cached export at the current version
5. **Cache hit path:** If the export exists, build URLs and return JSON immediately
6. **Cache miss path:** Acquire the semaphore, double-check cache, then:
   - Fetch all chapter content from AO3
   - Generate an EPUB file with proper formatting
   - Move the EPUB to the permanent cache location
   - Generate an HTML bundle with the same chapters
   - Move the HTML bundle to its cache location
   - Record both exports in the database
   - Build and return the JSON response with download URLs

The response includes URLs like `/cache/epub/123456?h=abc123def`. The client (usually a web frontend) can then redirect the user to that URL to download the file. The cache serving layer (which we'll cover later) handles serving the actual file from disk.

> 🧪 **Try It Yourself**
>
> 1. Create a mock `FicMetadata` with a funny title and test the `generate_slug` function
> 2. Test edge cases: What about titles with only special characters? Empty titles?
> 3. Build a mock `build_info_string` call and verify the output format
> 4. Write a simplified version of the cache-hit path that just returns a JSON response with hardcoded URLs
> 5. What happens if the URL parameter is missing? Test the empty query path

### What We Built

In this part, we went from scraped chapter content to fully formatted, downloadable books:

- **EPUB generation** using the `epub-builder` crate, with metadata, CSS, and chapters wrapped in proper XHTML
- **HTML bundles** using the `zip` crate, with navigation anchors, responsive CSS, and a clean reading layout
- **Disk caching** with hierarchical paths, three-layer version-based invalidation, and semaphore-based concurrency control
- **The export handler** that ties it all together, handling cache hits, cache misses, and everything in between

The export pipeline follows a clean separation of concerns:

```
epub.rs        →  knows how to make EPUBs (but not where to store them)
html_bundle.rs →  knows how to make HTML bundles (but not where to store them)
cache/disk.rs  →  knows how to store files on disk (but not what they contain)
export.rs      →  orchestrates the flow (but delegates the actual work)
```

Each module has a single job. The EPUB module doesn't know about caching. The cache module doesn't know about EPUBs. The export handler knows about both, but only in terms of "call this function, get this result." This makes each piece independently testable and replaceable.

One pattern worth noticing is the **tuple return** `(PathBuf, String)` — path plus hash. This shows up in both `create_epub` and `create_html_bundle`. The hash serves double duty: it's used as the cache key for future lookups, and it's used as part of the filename on disk. By computing the hash from the file contents (not the metadata), we ensure that two identical EPUBs always get the same cache key, regardless of when or how they were generated.

In the next part, we'll look at how FicHub serves those cached files back to users, and how the frontend consumes the API we've just built. The export pipeline is complete — stories go in one end, and beautiful ebooks come out the other.

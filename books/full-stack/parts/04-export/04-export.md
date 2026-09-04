# Part 4: Export and Caching

*The export pipeline turns scraped story data into downloadable files—EPUBs for e-readers, HTML bundles for browsers—and stores them so we never have to do the work twice. This part covers file generation, content-addressable caching, Redis-backed rate limiting, and the full end-to-end export flow.*

---

## Chapter 16: Generating EPUB Files

### What Is an EPUB?

An EPUB file is the closest thing the internet has to a universal e-book standard. It's what you send to a Kindle (with a quick conversion), a Kobo, a Nook, an iPad, or any modern reading app. And here's the thing that makes it interesting from an engineering perspective: an EPUB file is just a ZIP archive with a specific structure inside.

That's it. No magic, no proprietary binary format. A `.epub` file is a ZIP containing XHTML files, CSS stylesheets, images, and an XML manifest that tells reading software how to put it all together. The International Digital Publishing Forum (IDPF) standardized this format in 2007, and now it's maintained by the W3C as EPUB 3.

An EPUB 3 archive contains:

- `META-INF/container.xml` — tells the reader where the package document is
- `content.opf` — the Open Packaging Format document listing all files and metadata
- `toc.ncx` — navigation control for backward compatibility with EPUB 2 readers
- `stylesheet.css` — our custom styling
- `introduction.xhtml` — the story's title page with metadata
- `chapter_1.xhtml` through `chapter_N.xhtml` — one file per chapter

For FicHub, EPUB generation is the core export feature. When someone asks for a fanfiction story, we need to hand them a beautifully formatted book they can load onto their e-reader. That means taking the HTML content we scraped from Archive of Our Own, FanFiction.net, or Royal Road and wrapping it in a proper EPUB structure with metadata, navigation, and styling.

Why EPUB over other formats? Because it's the standard. Amazon's MOBI format is proprietary. PDF doesn't adapt to different screen sizes. Plain HTML lacks navigation and metadata support. EPUB gives readers everything they need in a format every device understands.

### Why EPUB Over Other Formats?

Before diving into the implementation, it's worth understanding why FicHub chose EPUB as the primary export format. Each format has trade-offs:

- **EPUB**: Open standard, works on virtually every e-reader and reading app, supports metadata and navigation, renders with the reader's own styling. This is the gold standard for long-form reading.
- **PDF**: Fixed layout, doesn't adapt to screen sizes, poor accessibility. Good for print but terrible for reading on phones.
- **MOBI/AZW3**: Amazon's proprietary format. Requires conversion from EPUB. No reason to generate natively.
- **HTML**: Universal but lacks the navigation structure and metadata support that EPUB provides.
- **Plain text**: Loses all formatting. No chapter navigation, no metadata, no styling.

EPUB wins because it's the standard that every device supports. If a user doesn't have an e-reader, the HTML bundle serves as a fallback. By generating both EPUB and HTML, FicHub covers all reading scenarios.

### The epub-builder Crate: Making EPUBs in Rust

Rather than building the ZIP archive and XML manifests by hand, we use the `epub-builder` crate. It handles all the internal EPUB plumbing—the OPF package document, the NCX navigation file, the container.xml—and gives us a clean API to work with.

Here's the dependency in `Cargo.toml`:

```toml
epub-builder = "0.7"
```

The core API follows a builder pattern. You create an `EpubBuilder` with a zip library, set metadata, add content pages, and generate the file. The `ZipLibrary` type tells `epub-builder` how to create the ZIP archive—there are different backends available, but `ZipLibrary::new()` uses the standard `zip` crate.

Here's the dependency list for the full export system:

```toml
epub-builder = "0.7"      # EPUB generation
zip = "0.6"                # ZIP compression for HTML bundles
md5 = "0.7"                # Content hashing
uuid = "1.0"               # Unique workspace identifiers
chrono = "0.4"             # Timestamp formatting
hex = "0.4"                # Hash encoding
```

The `md5` and `uuid` crates are also used elsewhere in the application (for API keys and content identification), so they're shared dependencies. The `zip` crate is used both by `epub-builder` (internally) and by the HTML bundle generator (directly).

Let's look at the full signature of our EPUB creation function:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

/// Generate an EPUB file for the given fic metadata and chapters.
///
/// Creates a UUID-named subdirectory inside `tmp_dir`, writes the EPUB there,
/// computes its MD5 hash, and returns `(path_to_epub, md5_hex)`.
pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // ... implementation ...
}
```

Notice the function is `async`. While the EPUB generation itself is synchronous (CPU-bound file I/O), making it async allows the runtime to yield to other tasks during disk writes, and keeps the interface consistent with the rest of the async application.

### Creating a Temporary Directory with UUID

Every export starts with a clean workspace. We generate a UUID and create a temporary directory inside our configured `tmp_dir`:

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

Why UUID? Because when ten people request the same story at once (and they will—we've seen it happen with popular updates), each export needs its own isolated workspace. The UUID guarantees uniqueness without any coordination. No locks, no file existence checks, no race conditions.

The temporary directory structure looks like:

```
/tmp/fichub/
  a1b2c3d4-e5f6-7890-abcd-ef1234567890/
    output.epub
  b2c3d4e5-f6a7-8901-bcde-f12345678901/
    output.epub
```

Each UUID-named directory contains exactly one EPUB file. After the file is generated, it's moved to the cache and the temporary directory is cleaned up (or left for the OS to clean later).

### Setting Metadata: Title, Author, Language

An EPUB without metadata is like a book without a cover. The `epub-builder` crate lets us set metadata fields that e-readers use to display story information in libraries:

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

The `lang` field is required by the EPUB specification. Without it, some reading apps refuse to open the file or default to whatever language their OS uses. We default to English, but the scraper could be extended to detect the language from the content.

The `description` field is interesting—we set it to the story's description from the source site. Some reading apps show this in search results or on the book's info page.

The `FicMetadata` struct provides all the fields we need:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,       // Unique identifier for the story
    pub title: String,        // Story title
    pub author: String,       // Author name
    pub chapters: i32,        // Number of chapters
    pub words: i64,           // Total word count
    pub desc: String,         // Story description (HTML)
    pub published: i64,       // Publication date (unix millis)
    pub updated: i64,         // Last update date (unix millis)
    pub status: String,       // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,       // Original URL
    pub source_id: i64,       // Numeric ID of the source site
    pub author_id: i64,       // Numeric ID of the author
    pub author_url: String,   // Author's profile URL
    pub author_local_id: String, // Author's ID on the source site
    pub content_hash: Option<String>, // Hash of the story content
    pub extra_meta: Option<String>,   // Additional metadata
    pub raw_extended_meta: Option<String>, // Raw extended metadata
}
```

### Adding Inline CSS

Every chapter needs styling. Rather than including a separate CSS file (which adds complexity and risks broken file references), we inline it:

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

The `concat!` macro joins the three string literals at compile time, producing a single string. The `stylesheet()` method in `epub-builder` writes this as a file named `stylesheet.css` inside the EPUB and automatically links it to all content pages via `<link rel="stylesheet">`.

This gives readers a clean, readable layout: serif fonts for that bookish feel, centered chapter titles, and comfortable spacing between paragraphs. It's minimal, but it works across all reading apps—from Kindle's serif defaults to iBooks' more modern look.

### The Introduction Page: Showing Story Info

Before the first chapter, we add an introduction page that shows the story's metadata. This is the "title page" of our e-book:

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

builder.add_content(
    EpubContent::new("introduction.xhtml", intro_html.as_bytes())
        .title("Introduction"),
)?;
```

Notice two important things here.

First, we're using `escape_html()` on user-provided strings like the title and author. This prevents XSS attacks and EPUB parsing errors from special characters in story metadata. A story titled "He Said <she> & 'went' to the Shop" would break an XHTML document if the `<`, `>`, `&`, and `'` characters weren't escaped.

Second, we use `format_timestamp()` to convert Unix milliseconds to a human-readable date. The source sites store dates as Unix timestamps in milliseconds, but readers want to see "2024-03-15" not "1710460800000".

Here are those helper functions:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

The `escape_html` function replaces the five characters that have special meaning in HTML/XML. The order matters: we replace `&` first because the other replacements introduce `&` characters (like `&amp;`). If we replaced `<` first, then replaced `&`, we'd double-escape the ampersand.

### Adding Chapters as Separate XHTML Files

Each chapter becomes its own XHTML file inside the EPUB. This is how e-readers handle navigation—each chapter is a separate "page" that readers can flip through:

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

The `chapter_id` comes from the scraper and ensures chapters are uniquely named. The `EpubContent::new()` call takes a filename and the HTML content as bytes. The `.title()` call sets what the e-reader shows in its table of contents.

Important: the `chapter.content` field is already HTML from the scraper. We don't escape it because the scraper has already ensured it's valid HTML (with proper paragraph tags, line breaks, etc.). If we escaped it, the HTML tags would show up as literal text in the reader.

The `Chapter` struct is simple:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,   // Sequential chapter number (1, 2, 3, ...)
    pub title: String,     // Chapter title
    pub content: String,   // HTML content of the chapter
}
```

The `chapter_id` is typically the chapter number as assigned by the source site. This ensures chapters maintain their original order and numbering.

### The Spine: The Order of Chapters

You might wonder: how does the EPUB reader know which chapter comes first? That's what the "spine" is for. In EPUB terminology, the spine defines the reading order of all the content documents. With `epub-builder`, this happens automatically—the spine follows the order in which you call `add_content()`.

That's why we add the introduction first, then loop through chapters in order. If we added them out of order, the reader would present them in a jumbled mess.

Under the hood, `epub-builder` builds the OPF manifest like this:

```xml
<spine toc="ncx">
  <itemref idref="introduction"/>
  <itemref idref="chapter_1"/>
  <itemref idref="chapter_2"/>
  <!-- ... more chapters ... -->
</spine>
```

Each `<itemref>` points to a content document. The reader processes them in order. If a reader supports "go to next chapter," it follows this spine.

### Writing the EPUB File

After adding all the content, we write the EPUB to disk:

```rust
let epub_path = work_dir.join("output.epub");
let file = fs::File::create(&epub_path)?;
builder.generate(file)?;
```

The `generate()` method does all the heavy lifting: it creates the EPUB's internal ZIP structure, writes the OPF manifest, generates the NCX navigation document, and packages everything together. The resulting `output.epub` is a valid EPUB 3 file that any compliant reader can open.

The output file is always named `output.epub`—the actual identity comes from its hash and the cache path. The filename is irrelevant once we move it to the cache.

### Computing the MD5 Hash

After generating the file, we compute an MD5 hash. This hash serves two purposes: it's a content-addressable identifier for the cache, and it lets us detect if the content has changed:

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();

Ok((epub_path, md5_hex))
```

The `Md5::digest()` function returns a 16-byte array. We format each byte as two lowercase hex characters, producing a 32-character string like `ef92b24f0e9a8d3c12345678abcdef01`.

MD5 isn't cryptographically secure (there are known collision attacks), but it's perfectly fine for content-addressable caching. We don't need collision resistance—we just need a fast way to identify identical content. If two different stories ever produced the same MD5 hash (astronomically unlikely), the worst case is a cache collision, which the hash verification in the download handler would catch.

The function returns both the path to the EPUB and its hash. The caller will use the hash for cache storage and to give the client a download URL with integrity verification.

### The ExportError Type

All export functions return `Result<T, ExportError>`. This error enum wraps the various failure modes:

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),       // File system errors
    TemplateError(String), // Template formatting errors
    EpubError(String),     // epub-builder errors
    CalibreError(String),  // Calibre conversion errors
    ZipError(String),      // ZIP archive errors
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

impl std::error::Error for ExportError {}
```

The `From` implementations let us use the `?` operator to automatically convert library errors into our error type:

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

This pattern keeps the code clean—`fs::File::create(&epub_path)?` automatically wraps IO errors without any extra code.

### Version Tracking

The export module also tracks versions. Each export format has a version number that's used for cache invalidation:

```rust
pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m
}

pub fn compute_version(export_version: i32, etype_version: i32, fic_version_bump: i32) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

The total version is the sum of the export module version, the format-specific version, and any content-level version bumps. If we change the EPUB template (fix a CSS bug, add a new metadata field), we bump `ETYPE_EPUB` from 1 to 2. This invalidates all cached EPUBs without deleting them—they just won't match the new version number.

MOBI and PDF are at version 0 because they're generated by converting from EPUB using Calibre (covered in `convert.rs`). Their templates are the EPUB template—when it changes, MOBI and PDF are regenerated automatically because they depend on the EPUB hash.

### Converting EPUBs to Other Formats

FicHub doesn't just generate EPUBs—it can convert them to MOBI and PDF using Calibre's `ebook-convert` tool. This is handled by `src/export/convert.rs`:

```rust
const CONVERT_TIMEOUT_SECS: u64 = 300;

pub async fn convert_epub(
    epub_path: &Path,
    output_format: &str,
    calibre_container: &str,
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    let output_filename = format!("output.{output_format}");
    let output_path = work_dir.join(&output_filename);

    // First attempt
    let first = run_conversion(epub_path, &output_path, calibre_container).await;

    match first {
        Ok(()) => {}
        Err(e) => {
            // Retry once on failure
            tracing::warn!("First conversion attempt failed: {e}. Retrying once ...");
            run_conversion(epub_path, &output_path, calibre_container)
                .await
                .map_err(|retry_err| {
                    ExportError::CalibreError(format!(
                        "Calibre conversion failed after retry: {retry_err}"
                    ))
                })?;
        }
    }

    // Verify output exists
    if !output_path.exists() {
        return Err(ExportError::CalibreError(format!(
            "Output file was not created: {}",
            output_path.display()
        )));
    }

    // Compute hash
    let output_data = fs::read(&output_path)?;
    let md5_hex = Md5::digest(&output_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((output_path, md5_hex))
}
```

Calibre can run either directly on the host or inside a Docker container. The `calibre_container` parameter controls this:

```rust
async fn run_conversion(
    epub_path: &Path,
    output_path: &Path,
    calibre_container: &str,
) -> Result<(), ExportError> {
    let timeout_dur = Duration::from_secs(CONVERT_TIMEOUT_SECS);

    if calibre_container.is_empty() {
        run_direct(epub_path, output_path, timeout_dur).await
    } else {
        run_docker(calibre_container, epub_path, output_path, timeout_dur).await
    }
}
```

The Docker approach is useful when Calibre isn't installed on the host system but is available as a container. The conversion is wrapped in a 300-second timeout to prevent hung processes from blocking the export pipeline. The one-retry pattern handles transient Calibre errors—sometimes the first attempt fails due to temporary resource constraints but succeeds on the second try.

### CSS Design Philosophy

The CSS in both the EPUB and HTML bundle follows a deliberate design philosophy:

1. **Serif fonts for reading**: Georgia (HTML) and generic serif (EPUB) are chosen for readability. Sans-serif fonts are easier to scan but harder to read for long periods.

2. **Generous line height**: 1.5-1.7 line height reduces eye strain. The default in most browsers is 1.2, which is too tight for long-form reading.

3. **Centered layout with max-width**: 800px is the sweet spot for line length. Studies show 50-75 characters per line is optimal for reading comprehension. 800px at 16px font size gives roughly 70-80 characters.

4. **Minimal decoration**: No background images, no gradients, no animations. The content is the star. The styling exists only to make the content comfortable to read.

5. **High contrast**: #333 text on #fafafa background is easier on the eyes than pure black on pure white. The contrast ratio is still well above WCAG accessibility requirements.

These choices aren't arbitrary—they're based on typographic research about what makes text comfortable to read for extended periods.

> 🧪 **Try It Yourself**: Take any HTML file and create a minimal EPUB with `epub-builder`. Set the metadata, add a single chapter, and write it out. Rename the `.epub` to `.zip` and open it—you'll see the internal structure: a `META-INF` directory, `content.opf`, and your XHTML files. Try removing `stylesheet.css` and see if the EPUB still opens (it should, but without styling).

> ⚠️ **Watch Out**: The `epub-builder` crate doesn't validate your HTML. If you pass in broken XHTML (unclosed tags, invalid entities), the EPUB might generate successfully but fail to open on certain readers. Always escape user-provided content with `escape_html()` and trust that the scraper provides valid HTML for chapter content.

---

## Chapter 17: HTML Bundles

The HTML bundle generator (`src/export/html_bundle.rs`) builds a self-contained HTML file with all chapters, styled with CSS for comfortable reading, and packaged in a ZIP. It follows the same UUID-workspace pattern as the EPUB generator. While EPUB is the gold standard for dedicated e-readers, the HTML bundle is the universal fallback—readable on any device with a web browser.

### What Is an HTML Bundle?

Not everyone wants an EPUB. Some users want to read a story right in their browser, or they want a single portable file they can open on any computer without special software. That's what the HTML bundle is for.

An HTML bundle is exactly what it sounds like: a complete, self-contained HTML file with all the chapters, styling, and navigation baked in. We package it into a ZIP file so it downloads as a single file and takes up less space on disk.

The HTML bundle is simpler than EPUB in some ways (no XML manifests, no navigation documents, no OPF packaging) but more complex in others (we need to build a complete, well-styled web page from scratch). Where EPUB delegates rendering to the reading app, the HTML bundle has to provide all the styling and layout itself.

### Building a Complete HTML Document

Let's walk through the `create_html_bundle` function:

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Unique workspace
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;
```

Same pattern as the EPUB: create a UUID-named workspace. This consistency makes the code easier to understand and maintain. Both export functions follow the same lifecycle: create workspace, build output, compute hash, return results.

### Adding a Table of Contents with Links

The HTML bundle builds navigation and content in a single loop, but they go into different parts of the page. The navigation goes at the top; the full chapter content goes below. This separation is important—it lets us put the navigation in a fixed, easy-to-access location while the content scrolls freely:

```rust
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
```

Each chapter gets an anchor (`id="ch{ch}"`) that the table of contents links to. Click "Chapter 7" in the nav, and the browser jumps right to it. The `r##"..."##` syntax (a raw string with a delimiter) is used for the navigation links because the `#` in `href="#ch1"` would normally need escaping—but in raw strings, it's just a character.

The two-column layout for navigation uses CSS `columns: 2`, which automatically flows the list items across two columns. For a story with 20 chapters, this gives a compact navigation block at the top of the page.

### The Full HTML Structure

The complete HTML document includes styling, metadata, a chapter navigation section, and all the chapter content. It's designed to look good on both desktop and mobile:

```rust
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
            background: #fafafa;
        }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }}
        h2 {{
            text-align: center;
            margin: 1.5em 0 0.5em;
            font-size: 1.4em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }}
        .meta td {{ padding: 2px 8px; }}
        .desc {{
            margin: 1em 0;
            padding: 1em;
            background: #fff;
            border-radius: 4px;
            border: 1px solid #eee;
        }}
        .nav {{
            background: #fff;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1em;
            margin: 1.5em 0;
        }}
        .nav h3 {{ margin-bottom: 0.5em; }}
        .nav ul {{ list-style: none; columns: 2; }}
        .nav li {{ padding: 2px 0; }}
        .nav a {{ color: #1a5276; text-decoration: none; }}
        .nav a:hover {{ text-decoration: underline; }}
        .content p {{ margin: 0.5em 0; text-indent: 1.5em; }}
        .content p:first-of-type {{ text-indent: 0; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2em 0; }}
        .footer {{ text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }}
        a.back-to-top {{ display: block; text-align: right; font-size: 0.85em; color: #1a5276; }}
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
    <div class="desc">{desc_escaped}</div>
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
```

Let's break down the styling decisions:

- **`max-width: 800px`** — Limits line length for comfortable reading. Too-wide text is hard to read.
- **`font-family: Georgia`** — A serif font that's available on virtually every system. It's the safe default for reading-focused pages.
- **`line-height: 1.7`** — Generous line spacing reduces eye strain during long reading sessions.
- **`background: #fafafa`** — Off-white is easier on the eyes than pure white, especially at night.
- **`text-indent: 1.5em`** — Paragraph indentation follows traditional book formatting. The first paragraph after a heading isn't indented.
- **`border-bottom` on h2** — Visual separation between chapters without taking up too much vertical space.

The `viewport` meta tag ensures the page looks good on mobile devices. Without it, mobile browsers would render the page at desktop width and zoom out.

The `.desc` section uses a card-style design (white background, border, border-radius) to visually separate the story description from the content. This creates a clear hierarchy: title → metadata → description → navigation → chapters.

### The zip Crate: Packaging It Up

A raw HTML file would work, but ZIP compression significantly reduces file size (typically 60-80% for text-heavy content). Plus, the `.zip` extension is universally recognized:

```rust
use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

// Write the HTML file first
let html_path = work_dir.join("index.html");
fs::write(&html_path, &html)?;

// Bundle into ZIP
let zip_path = work_dir.join("bundle.zip");
let zip_file = fs::File::create(&zip_path)
    .map_err(|e| ExportError::IoError(e.to_string()))?;
let mut zip = ZipWriter::new(zip_file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)
    .map_err(|e| ExportError::ZipError(e.to_string()))?;
zip.write_all(html.as_bytes())
    .map_err(|e| ExportError::ZipError(e.to_string()))?;

zip.finish()
    .map_err(|e| ExportError::ZipError(e.to_string()))?;
```

The `Deflated` compression method is a good balance between speed and ratio. We set Unix permissions to `0o644` (readable by everyone, writable by owner) so the file behaves correctly when extracted on Unix systems.

Why ZIP the HTML? A 500KB HTML file might compress to 150KB in a ZIP. For popular stories with hundreds of thousands of words, the savings are significant. And since the cache stores the ZIP, we save disk space too.

The ZIP contains a single file: `index.html`. This is different from the EPUB, which contains many files. The simplicity is intentional—it keeps the download small and the extraction trivial.

### Computing the Hash

Same pattern as the EPUB—we read the ZIP file and compute its MD5:

```rust
let zip_data = fs::read(&zip_path)?;
let md5_hex = Md5::digest(&zip_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();

Ok((zip_path, md5_hex))
```

The hash is computed over the ZIP file, not the HTML. This means if we change the ZIP compression settings but keep the HTML identical, the hash would change. That's fine—the cache treats each format independently.

### The escape_html and format_timestamp Helpers

The HTML bundle uses the same helper functions as the EPUB generator:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

These are duplicated between the EPUB and HTML modules. In a larger codebase, you might extract them to a shared utility module. For now, the duplication is acceptable—it keeps each module self-contained and avoids tight coupling.

### Why the Double-Curly-Brace Syntax?

You might notice `{{ box-sizing: border-box; }}` in the Rust format string. This is a quirk of Rust's `format!()` macro: curly braces `{` and `}` are reserved for interpolation. To include literal braces in the output, you must double them: `{{` produces `{`, and `}}` produces `}`.

The CSS for the HTML bundle is significantly more elaborate than the EPUB stylesheet because:
1. EPUB styling is interpreted by the reading app—our CSS is a suggestion, not a command.
2. HTML styling is rendered directly by the browser—we control the full presentation.
3. The HTML bundle needs to work on mobile, desktop, and print—the CSS must handle all contexts.

### Responsive Design Considerations

The HTML bundle includes several responsive design patterns:

- **`max-width: 800px`** prevents lines from becoming too long on wide screens
- **`margin: 0 auto`** centers the content container
- **`padding: 20px`** provides breathing room on narrow screens
- **`meta viewport` tag** ensures mobile browsers don't zoom out to fit desktop width
- **Two-column navigation** uses CSS `columns: 2` which automatically reflows to one column on narrow screens

These patterns ensure the story is readable on a 4-inch phone, a 10-inch tablet, and a 27-inch monitor without any JavaScript.

### Performance Considerations

For very long stories (100+ chapters, millions of words), the HTML bundle can become quite large. A 2-million-word story might produce a 10MB HTML file. Compression typically reduces this to 2-3MB in the ZIP, but the uncompressed HTML still needs to be rendered by the browser.

Modern browsers handle large HTML files well, but there are limits. If a story exceeds about 5 million words, the HTML bundle might cause performance issues on mobile devices. For these cases, the EPUB format is better because reading apps handle pagination and lazy loading.

The EPUB format doesn't have this problem because reading apps load one chapter at a time. The HTML bundle loads everything at once.

### When to Use HTML vs EPUB

The export handler generates *both* formats for every request. But they serve different needs:

**EPUB is ideal for:**
- E-readers (Kindle, Kobo, Nook)
- Reading apps (Apple Books, Calibre, Moon+ Reader)
- Offline reading on mobile devices
- Large stories (better chapter navigation)
- Users who prefer adjustable font sizes and reader-controlled styling
- Preserving reading progress across sessions

**HTML bundle is ideal for:**
- Quick reading in a browser
- Sharing with others who might not have an e-reader
- Archiving (it's a single, self-contained file)
- Quick searches (Ctrl+F works on the entire story)
- Printing (the browser's print dialog works perfectly)
- Viewing on devices without reading apps

Both are cached independently, so subsequent requests for the same story (in either format) are instant. The choice doesn't affect performance—it's purely about user preference.

Some users prefer the HTML format because they can open it on any device without installing software. Others prefer EPUB because it syncs reading progress across devices via their reading app. By providing both, FicHub accommodates all preferences.

### Accessibility Considerations

The HTML bundle includes several accessibility features:

- **`lang="en"` on the `<html>` tag**: Tells screen readers what language to use for pronunciation.
- **Semantic HTML**: Uses `<h1>`, `<h2>`, `<table>`, `<ul>`, `<li>` elements correctly. Screen readers can navigate by heading level.
- **Sufficient color contrast**: The #333 on #fafafa combination exceeds WCAG AA requirements (4.5:1 ratio).
- **Responsive design**: Works on all screen sizes without horizontal scrolling.

The EPUB format inherits accessibility from the reading app—most modern readers support screen readers, adjustable fonts, and high-contrast modes. Our job is to provide valid XHTML and let the reader handle the rest.

### Testing the HTML Bundle

To verify the HTML bundle works correctly, try these tests:

1. **Open in Chrome/Firefox/Safari**: The page should render correctly in all major browsers.
2. **Check mobile rendering**: Resize the browser to 375px width (iPhone SE). The layout should adapt.
3. **Test the navigation**: Click each chapter link in the table of contents. The page should jump to the correct chapter.
4. **Search with Ctrl+F**: Type a word from the middle of the story. It should be found across all chapters.
5. **Extract the ZIP**: Use `unzip bundle.zip` and open `index.html`. It should work identically.
6. **Check the source**: View the HTML source. All special characters should be properly escaped.

> 🧪 **Try It Yourself**: Download an HTML bundle from FicHub, extract the ZIP, and open `index.html` in a browser. Try resizing the window—the layout should adapt. Now open the same ZIP on your phone's browser. It should be readable too. Try using Ctrl+F to search across all chapters—the HTML format excels at this.

> ⚠️ **Watch Out**: The HTML bundle uses `escape_html()` on the story description but not on chapter content. Chapter content is already HTML from the scraper. If a scraper returns raw, unescaped text instead of HTML, the bundle could break. Each scraper must ensure its chapter content is valid HTML. Always validate your scrapers' output before trusting it.

---

## Chapter 18: The Disk Cache

The disk cache module (`src/cache/disk.rs`) is the persistence layer for the export system. It provides content-addressable file storage with hash-based directory structures, atomic file moves, and cache integrity verification. Combined with the `export_log` database table, it ensures FicHub only generates each export once.

### Why Cache? Don't Redo Work You've Already Done

Generating an EPUB involves scraping a website (which can take seconds to minutes), parsing its content, building the EPUB structure, and writing it to disk. If someone requests the same story again—maybe they lost the file, or they're using a different device, or they want the HTML version this time—why redo all that work?

The answer is caching: store the generated file on disk and serve it directly on subsequent requests. FicHub's caching system is hash-based and content-addressable, which means if the story hasn't changed, we serve the same file. If it has, we generate a new one with a different hash.

The cache also serves another purpose: it provides integrity verification. When a client requests a cached file, we recompute the hash and compare it to the expected hash. If they don't match, we return an error instead of serving a potentially corrupted file.

Consider the math: without caching, a popular story with 100 chapters might take 60 seconds to scrape and generate. If 100 people request it in a day, that's 100 minutes of scraping. With caching, the first request takes 60 seconds, and all subsequent requests take milliseconds. The cache saves us 99+ minutes of work per day for a single story.

### Cache Invalidation Strategies

FicHub uses content-based cache invalidation. Instead of expiring cached files after a time period (TTL-based), we invalidate them when the content changes:

1. **Content hash from scraper**: When the scraper fetches metadata, it also computes a hash of the story's content (chapter count, word count, last update time). If this hash changes, we know the story has been updated.

2. **Export version numbers**: The export module has version numbers for each format. When we change the EPUB template (fix a CSS bug, add metadata), we bump the version. This invalidates all cached files without deleting them.

3. **Export log in PostgreSQL**: The `export_log` table records what we've generated. When a request comes in, we check the log for a matching version, format, and input hash. If it matches, we serve the cached file.

This triple-layered approach ensures we never serve stale content while maximizing cache hits.

### Hash-Based Directory Structure

The cache lives on disk with a carefully structured directory layout. The `cache_path` function computes where a file should go:

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

This creates a path like `/cache/epub/abc/def/abcdefghijklm/ef92b24f0e9a8d3c.epub`. The three-character directory chunks prevent any single directory from having thousands of entries—which would slow down filesystem operations on many systems.

Let's trace through a few examples to understand the algorithm:

**Short url_id (`"abc"`):**
1. `cache/epub`
2. Chunk at index 0-2: `abc` → `cache/epub/abc`
3. Full url_id: `cache/epub/abc/abc`
4. File: `cache/epub/abc/abc/<hash>.epub`

**Medium url_id (`"abcdef"`):**
1. `cache/html`
2. Chunk at index 0-2: `abc` → `cache/html/abc`
3. Chunk at index 3-5: `def` → `cache/html/abc/def`
4. Full url_id: `cache/html/abc/def/abcdef`
5. File: `cache/html/abc/def/abcdef/<hash>.zip`

**Long url_id (`"abcdefghijklm"`):**
1. `cache/mobi`
2. Chunk at index 0-2: `abc` → `cache/mobi/abc`
3. Chunk at index 3-5: `def` → `cache/mobi/abc/def`
4. Chunk at index 6-8: `ghi` → `cache/mobi/abc/def/ghi`
5. Full url_id: `cache/mobi/abc/def/ghi/abcdefghijklm`
6. File: `cache/mobi/abc/def/ghi/abcdefghijklm/<hash>.mobi`

The `.take(3)` ensures we create at most three chunk directories (9 characters of the url_id). Beyond that, the full url_id is used as the final directory name. This gives us a good balance between directory depth and spread.

### The EType Enum: epub, html, mobi, pdf

FicHub supports four export formats, modeled as an enum:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

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
            EType::Html => ".zip",     // HTML bundles are zipped
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

The `version()` method is particularly clever. Each format has a version number that gets baked into the cache key. If we change the EPUB template (fix a CSS bug, add a new metadata field), we bump the version from 1 to 2, and all cached EPUBs are effectively invalidated without us having to delete anything.

MOBI and PDF are at version 0 because they're generated by converting from EPUB—they don't have their own templates. When the EPUB version changes, MOBI and PDF are regenerated automatically because their cache keys depend on the EPUB hash.

The `FromStr` implementation lets us parse format strings:

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

Note the `.to_lowercase()` — this makes the parsing case-insensitive, so `"EPUB"`, `"Epub"`, and `"epub"` all parse to `EType::Epub`. This is important because the URL path might use different casing.

### move_to_cache: Moving Files to the Cache

After generating a file in a temporary directory, we move it to the cache:

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

This is a simple but important operation. We create the full directory structure (the three-character chunks, the url_id directory) and then move the file atomically. The `fs::rename` is an atomic operation on most filesystems—it either succeeds completely or fails completely. No half-written files in the cache.

We use `rename` instead of `copy` because the temporary file is no longer needed after moving. This saves disk space and avoids the overhead of copying large files.

The `create_dir_all` call is idempotent—if the directory already exists, it does nothing. This means concurrent requests can safely call `move_to_cache` without coordination.

### CacheSemaphores: Preventing Duplicate Work

Here's a subtle but critical problem: what if ten people request the same story simultaneously? Without coordination, we'd start ten concurrent exports—wasting CPU, bandwidth, and disk space—all generating the exact same file.

FicHub solves this with semaphores, one per (url_id, etype) pair:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Semaphore map to prevent duplicate concurrent exports per (url_id, etype)
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

A `Semaphore::new(1)` is a mutex—only one task can acquire it at a time. The outer `Mutex<HashMap<...>>` protects the map of semaphores itself. The `Arc` wrapping ensures the semaphore can be shared across async tasks.

When a request comes in for a story that needs exporting, it acquires the semaphore. Subsequent requests for the same story wait. The first request generates the file, puts it in the cache, and releases the semaphore. The waiting requests then wake up—and find the file already in the cache.

The key insight is that the semaphore is *per story*. Exports for different stories proceed in parallel—only duplicate exports for the same story are serialized.

### clear_stale_cache: Cleaning Up

When a story is updated on the source site, the scraper detects a new content hash. We generate a new EPUB with a different hash—but the old one is still on disk. The `clear_stale_cache` function handles this:

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

It scans the cache directory for a given story and removes any file whose hash doesn't match the current one. The `let _ =` on `remove_file` is intentional—if a file can't be deleted (permission error, race condition), we log it but don't fail the entire operation.

Note that `clear_stale_cache` looks in `cache_root/etype/url_id/` directly (without the 3-char chunk directories). This is a simplification—the full `cache_path` function includes chunks, but for cleanup, we scan the flat url_id directory. In practice, this works because the cache directory structure is consistent.

### file_md5: Verifying Cache Integrity

The cache download handler verifies file integrity by recomputing the MD5 hash:

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

When a client requests a cached file, we verify the hash matches before serving it. If the file was corrupted on disk (filesystem error, accidental modification), we return an error instead of serving garbage.

The `hex::encode` function is from the `hex` crate—it converts the byte array to a lowercase hex string. This is more idiomatic than the manual formatting used in the EPUB/HTML generators, but produces the same result.

### Cache Size and Disk Usage

Cached files can accumulate over time. A typical EPUB is 100KB-5MB depending on story length. An HTML bundle is similar after ZIP compression. For a service with 10,000 cached stories, the total disk usage might be 1-50GB.

The `clear_stale_cache` function removes old versions of stories, but it only cleans up when a new version is generated. Stories that are deleted from the source site will remain in the cache indefinitely until manually cleaned up or until their directory is garbage-collected.

A production deployment might add a cron job that scans the cache directory and removes files older than a certain age. But for FicHub's current scale, manual cleanup is sufficient.

### The Double-Check Pattern in Detail

The export handler implements a classic concurrency pattern: double-checked locking. Here's the full flow:

1. **First check** (fast, no lock): Query the database for a cached export. This is a simple `SELECT` query against the `export_log` table, which should complete in under 5ms.

2. **Cache hit**: Return immediately. Total time: ~50ms. No lock contention, no file generation.

3. **Cache miss**: Acquire the semaphore (wait if another request is exporting the same story).

4. **Second check** (after lock): Query the database again. Another request might have finished while we waited.

5. **Second cache hit**: Return the result another request generated. We avoided redundant work.

6. **Second cache miss**: We're the first—generate the file, store it, release the semaphore.

This pattern ensures that:
- Cache hits are fast (no lock contention).
- Duplicate exports are avoided (only one generates at a time).
- Waiting requests don't redo work (they check again after the semaphore is released).
- The semaphore is automatically released when the request handler returns (RAII pattern).

The key insight is that the semaphore is *per story and per format*. Exporting story A's EPUB doesn't block exporting story B's EPUB. And exporting story A's EPUB doesn't block exporting story A's HTML. The granularity is exactly right.

### Why Not Use a Distributed Lock?

You might wonder: why use local semaphores instead of Redis-based distributed locks? The answer is that FicHub currently runs as a single instance. Local semaphores are simpler, faster, and don't require Redis for locking.

If FicHub scales to multiple instances, the semaphore approach would need to change. Options include:
1. **Redis-based locks** (e.g., Redlock): More complex but works across instances.
2. **Optimistic locking**: Use the database as the coordination point. The `export_log` table already serves this purpose.
3. **Message queue**: Route export requests through a queue so only one worker processes each request.

For now, the local semaphore combined with the database-based double-check provides sufficient coordination.

### Cache Statistics and Monitoring

In production, you'd want to monitor cache performance. Key metrics include:

- **Cache hit rate**: What percentage of requests are served from cache? A healthy system should have 80%+ hit rate for popular stories.
- **Cache size**: How much disk space is used? Set alerts for when it exceeds thresholds.
- **Stale cache entries**: How many old versions are sitting around? Run `clear_stale_cache` periodically.
- **Export latency**: How long do cache misses take? Track the 95th percentile.

You can get these metrics by logging cache hits/misses and monitoring disk usage. A simple approach is to add a metric counter in the export handler:

```rust
if cached.is_some() {
    tracing::info!("cache_hit url_id={} etype={}", meta.url_id, "epub");
} else {
    tracing::info!("cache_miss url_id={} etype={}", meta.url_id, "epub");
}
```

These logs can be aggregated into dashboards using tools like Grafana or Prometheus.

> 🧪 **Try It Yourself**: Run `ls -R /cache/epub/` (or wherever your cache directory is) to see the directory structure. Count how many stories are cached and how many have multiple versions (different hashes). The structure should match the `cache_path` algorithm: `<etype>/<3-chars>/<3-chars>/<3-chars>/<url_id>/<hash>.<suffix>`.

> ⚠️ **Watch Out**: The `move_to_cache` function uses `fs::rename`, which only works within the same filesystem. If your temporary directory and cache directory are on different mounts or volumes, the rename will fail with an "Invalid cross-device link" error. Make sure `tmp_dir` and `cache_dir` are on the same filesystem, or implement a copy-then-delete fallback.

---

## Chapter 19: Rate Limiting with Redis

The rate limiter module (`src/limiter/`) implements a Redis-backed token bucket algorithm that coordinates request rates across all FicHub instances. It protects upstream fanfiction sites from overload while keeping FicHub responsive for legitimate users.

### Why Be Polite to Websites?

FicHub scrapes content from Archive of Our Own, FanFiction.net, Royal Road, and other sites. These sites provide a free service—hosting stories for authors. Flooding them with rapid-fire requests is not only rude, it's a fast way to get our IP addresses blocked.

Rate limiting serves two purposes: it protects our upstream sites from overload, and it keeps FicHub functional by avoiding bans and temporary blocks. The rate limiter sits between FicHub's scraper and the external websites, ensuring we never exceed a safe threshold.

Consider what happens without rate limiting: a popular story update triggers 500 simultaneous requests. FicHub sends 500 concurrent HTTP requests to Archive of Our Own. AO3's servers see a sudden spike, flag it as suspicious, and temporarily block FicHub's IP range. Now *no one* can request stories from AO3 until the block expires. Rate limiting prevents this cascade.

But rate limiting isn't just about being polite. It's also about resilience. If AO3 starts returning 503 errors (server busy), the rate limiter automatically backs off by consuming extra tokens on failure. This means we naturally adapt to upstream conditions without manual intervention.

### Rate Limiting as a Shared Resource

The rate limiter is shared across all FicHub instances. If we run multiple servers (for load balancing or high availability), they all check against the same Redis buckets. This prevents the combined request rate from exceeding what upstream sites can handle.

Without shared state, each server would independently allow 30 requests/second. With 3 servers, that's 90 requests/second total—three times what we intended. Redis ensures that the global bucket is decremented atomically across all servers.

### What Is Redis? A Fast In-Memory Data Store

Redis is an in-memory data store that's incredibly fast (microsecond response times) and supports atomic operations. It's the perfect backing store for a rate limiter because:

1. **Speed**: Rate limit checks happen on every request. They need to be fast.
2. **Atomicity**: The rate limit check and update must happen as a single operation. Redis ensures this with single-threaded command execution.
3. **Persistence**: Redis can persist data to disk (via RDB snapshots or AOF logs), so rate limit state survives restarts.
4. **Shared state**: Multiple FicHub instances (if we scale horizontally) share the same rate limit state through Redis.
5. **Rich data structures**: Redis supports hashes, sorted sets, and Lua scripting—all useful for rate limiting.

FicHub connects to Redis in the server setup:

```rust
let redis_client = redis::Client::open(config.redis_url.as_str())
    .expect("Invalid Redis URL");
let redis_conn = redis_client.get_multiplexed_async_connection()
    .await
    .expect("Failed to connect to Redis");
```

The `MultiplexedConnection` is a single connection that can handle multiple concurrent requests without blocking. This is important in an async application where many tasks might need to check rate limits simultaneously.

### Token Buckets: An Analogy with Candy

The rate limiter uses the "token bucket" algorithm. Imagine you have a jar of candy:

- The jar has a **capacity** (maximum candy it can hold).
- Candy is added at a steady **flow rate** (e.g., 2 candies per second).
- Every request takes one candy from the jar.
- If the jar is empty, you have to wait until enough candy accumulates.
- If the jar is full, extra candy spills over (unused capacity doesn't carry forward beyond the cap).

In FicHub's terms, the "jar" is a Redis key, "candy" is tokens, and each HTTP request to an upstream site consumes one token.

Here are FicHub's actual token bucket parameters:

```rust
RedisBucketLimiter {
    global_capacity: 150.0,    // Max 150 tokens in global bucket
    global_flow: 30.0,         // Refill at 30 tokens/second
    ip_capacity: 30.0,         // Max 30 tokens per IP
    ip_flow: 0.116,            // ~1 token every 8.6 seconds per IP
}
```

The global bucket allows bursts of up to 150 requests across all users, refilling at 30 per second. This means we can handle a sudden spike of 150 requests, then sustain about 30 requests per second.

The per-IP bucket is much more restrictive: each user can make roughly one request every 8.6 seconds. This prevents any single user from monopolizing the global bucket. Even if a user makes 100 requests simultaneously, their per-IP bucket only allows one every 8.6 seconds.

### The Lua Script for Rate Limiting

The heart of the rate limiter is a Lua script that runs atomically inside Redis. Lua scripts in Redis execute as a single atomic operation—no other command can interfere:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

Let's walk through this script step by step:

1. **Read the current state**: Get the token count (`value`) and the last time we drained (`last_drain`) from a Redis hash. A hash is like a dictionary—`HMGET` gets multiple fields at once.

2. **Get the current time**: Redis provides time with microsecond precision via the `TIME` command. We combine seconds and microseconds for floating-point precision.

3. **Initialize if needed**: If this is the first request (no state exists), fill the bucket to capacity. This gives new keys an immediate burst allowance.

4. **Calculate new tokens**: Based on how much time has elapsed since the last drain, add `elapsed * flow` tokens, capped at `capacity`. The `math.min` ensures we never exceed the bucket's capacity.

5. **Check if allowed**: Subtract the requested tokens from the new total.

6. **Decision**: If tokens remain (`allowed >= 0`), update the state and return `-1` (the convention for "allowed"). If not, calculate how many seconds to wait and return that value.

The script is loaded once when the rate limiter starts:

```rust
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;
```

Subsequent calls use `EVALSHA` (execute by SHA hash) instead of `EVAL` (execute by script text). This is faster because Redis caches the compiled script. The SHA is computed from the script text, so any change to the script would invalidate the SHA.

### The check_bucket Method

The `RedisBucketLimiter` uses the Lua script via this method:

```rust
async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) -> Result<f64, redis::RedisError> {
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)           // number of keys
        .arg(key)         // the Redis key
        .arg(1.0)         // requested tokens (1 per check)
        .arg(capacity)    // bucket capacity
        .arg(flow)        // refill rate
        .query_async(&mut conn)
        .await?;
    Ok(result)
}
```

The return value is either `-1` (allowed, no wait) or a positive float (wait this many seconds). The caller uses this to decide whether to proceed or sleep.

### Alternatives to Token Buckets

Other rate limiting algorithms exist, each with different trade-offs:

- **Fixed window**: Count requests in a time window (e.g., 100 requests per minute). Simple but has a burst problem—100 requests in the first second of the window is allowed.
- **Sliding window log**: Store timestamps of all requests and count those in the sliding window. Precise but memory-intensive.
- **Sliding window counter**: Combine the current and previous window counts. Good balance of accuracy and efficiency.
- **Leaky bucket**: Requests enter a queue and are processed at a fixed rate. Smooth but introduces latency.

Token buckets were chosen for FicHub because they allow bursts (important for legitimate traffic patterns) while maintaining a steady average rate. The burst capacity (150 global, 30 per-IP) absorbs normal traffic spikes without penalizing users, while the flow rate (30/second global, ~0.12/second per-IP) prevents sustained overload.

### Tuning the Rate Limits

The rate limit parameters in FicHub are carefully tuned:

```rust
global_capacity: 150.0,    // Burst allowance for the entire system
global_flow: 30.0,         // Sustained rate (30 req/sec)
ip_capacity: 30.0,         // Per-user burst allowance
ip_flow: 0.116,            // ~1 req per 8.6 seconds per user
```

The global flow of 30 requests/second is based on what AO3 and FFN can comfortably handle. The per-IP flow of 0.116 tokens/second means each user can sustain about 7 requests per minute—enough to export a 7-chapter story without hitting the limit, but slow enough that a single user can't monopolize the global bucket.

If upstream sites start blocking FicHub, the first adjustment is usually reducing `global_flow`. If a specific user is hammering the API, reducing `ip_flow` throttles them without affecting others.

### Monitoring Rate Limit State

Redis makes it easy to monitor rate limit state. You can inspect the current token count and last drain time:

```
redis-cli HMGET rate:global value last_drain
redis-cli HMGET rate:ip:192.168.1.1 value last_drain
```

This is useful for debugging—if users report slow response times, you can check whether the global bucket is depleted. If `value` is near zero, the system is running at capacity and requests are waiting.

You can also use `redis-cli MONITOR` to watch all rate limit operations in real time. This is invaluable for tuning the parameters and understanding traffic patterns.

### The Global vs Per-IP Rate Limits

The rate limiter checks two buckets in sequence:

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        // Simple static delay for development
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }

    // Check datacenter IPs first
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    // Check global bucket first
    let global_wait = self.check_bucket(
        "rate:global", self.global_capacity, self.global_flow
    ).await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Then check per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(
        &ip_key, self.ip_capacity, self.ip_flow
    ).await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

The global bucket protects the upstream sites overall. The per-IP bucket prevents any single user from consuming the entire global allocation. Both must pass for a request to proceed.

The `.unwrap_or(-1.0)` on the Redis calls is a fallback—if Redis is temporarily unavailable, we default to "allowed" rather than blocking all requests. This is a deliberate trade-off: we'd rather risk being too aggressive with upstream sites than block all FicHub users because Redis is down.

The `.ceil()` rounds the wait time up to the nearest whole second. We don't want to tell users "wait 0.3 seconds"—that's imprecise and could lead to premature retries.

### The RateLimiter Trait

FicHub defines a trait for rate limiters, making it easy to swap implementations:

```rust
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allowed,
    Wait(u64),     // Wait N seconds before retrying
    Blocked,        // IP is in datacenter blocklist
}
```

The `report_failure` method is called when a scraper encounters an error (HTTP 429, 503, connection reset). It penalizes the IP by consuming extra tokens:

```rust
async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)        // penalize with 1.5 extra tokens
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

Using 1.5 tokens instead of 1.0 means a failure costs 50% more than a success. This naturally backs off when a site starts returning errors. If a site gives us a 429, the next request will wait longer. The penalty decays over time as the bucket refills.

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}
```

Both the global and per-IP buckets are penalized. The `let _ =` ignores Redis errors—we don't want a failed penalty to crash the application.

### Handling Rate Limit Exceeded: 429 Responses

When the rate limiter returns `Wait(n)`, the scraper respects it by sleeping for the specified duration before retrying. In development mode, the limiter uses a simpler approach:

```rust
if !self.dynamic_rate_limit {
    // Simple static delay
    let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
    tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
    return RateLimitResult::Allowed;
}
```

When `dynamic_rate_limit` is disabled, the limiter just adds a small random delay (0.1 to 0.2 seconds). This is useful for local development when Redis isn't running or when you're testing scrapers and don't want to wait for real rate limits.

In production, the full token bucket algorithm handles everything dynamically. The delay adapts to the current load—if the upstream site is under heavy use, the global bucket depletes faster and requests wait longer.

### Datacenter IP Blocking

Some requests come from cloud servers, data centers, or known bots. The rate limiter maintains a set of datacenter IP addresses and blocks them outright:

```rust
pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
    for (file_path, _type, _tag) in sources {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let mut ips = self.datacenter_ips.write().await;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        if let Ok(ip) = line.parse::<IpAddr>() {
                            ips.insert(ip);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not load IP tag file {}: {}", file_path, e);
            }
        }
    }
    tracing::info!(
        "Loaded {} datacenter IPs",
        self.datacenter_ips.read().await.len()
    );
}
```

This prevents automated scrapers from using FicHub as a proxy to scrape upstream sites. The IP sets are loaded from external tag files that can be updated independently. The `RwLock<HashSet<IpAddr>>` allows concurrent reads (which is what `is_datacenter_ip` does) while ensuring exclusive access when updating the set.

Note that `is_datacenter_ip` is currently a placeholder:

```rust
fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
    // Synchronous check - this is a best-effort check
    // For production, use a proper prefix tree (ipnet crate)
    false
}
```

For production use, you'd want to use the `ipnet` crate to efficiently check CIDR ranges (like `104.16.0.0/12` for Cloudflare, `198.41.0.0/20` for Amazon AWS). A simple `HashSet` won't work because datacenter IPs are specified as ranges, not individual addresses.

> 🧪 **Try It Yourself**: Use Redis CLI to watch the rate limit buckets in real time. Run `redis-cli MONITOR` in one terminal and make a few requests to FicHub in another. You'll see `HMGET` and `HMSET` operations on the `rate:global` and `rate:ip:*` keys. Each request consumes tokens from both buckets.

> ⚠️ **Watch Out**: The `unwrap_or(-1.0)` fallback on Redis calls means the limiter defaults to "allowed" when Redis is unreachable. In production, you might want a stricter fallback—perhaps blocking all requests when Redis is down, or using a local in-memory limiter as a backup. The current approach prioritizes availability over safety.

### Redis Persistence and Recovery

Redis can persist rate limit state to disk in two ways:

1. **RDB snapshots**: Periodic snapshots of the entire dataset. Fast to load on restart, but might lose some data between snapshots.

2. **Append-only file (AOF)**: Every write operation is appended to a log file. More durable, but slower to load and uses more disk space.

For rate limiting, RDB snapshots are usually sufficient. If rate limit state is lost on restart, the worst case is a brief burst of requests that exceeds the intended limit. The buckets refill quickly enough that this is self-correcting.

If you need stricter persistence (e.g., for billing or abuse prevention), use AOF with `everysec` fsync policy. This gives you at most 1 second of data loss on a hard crash.

### Failure Modes and Edge Cases

The rate limiter handles several edge cases:

1. **Redis is down**: The `unwrap_or(-1.0)` fallback returns "allowed." This is a deliberate trade-off: we'd rather risk being too aggressive than blocking all users.

2. **Clock skew**: The Lua script uses Redis's `TIME` command, which returns the server's clock. If the Redis server's clock is wrong, the refill calculation will be inaccurate. This is rare in practice.

3. **Negative token counts**: The `math.min(capacity, value + elapsed * flow)` ensures tokens never exceed capacity. The `allowed >= 0` check ensures we never consume more tokens than available.

4. **Rapid key expiration**: Redis keys with no expiration never expire. The rate limit buckets persist until explicitly deleted. This is correct behavior—we want the state to survive for the lifetime of the application.

---

## Chapter 20: The Export Flow (End to End)

### Following a Request from Start to Finish

Now that we've explored each subsystem in isolation, let's follow a complete export request from the moment a URL hits the API to the moment a download link appears in the response. This is the `epub_handler` in `src/routes/export.rs`, and it orchestrates everything we've built—the scraper, the EPUB and HTML generators, the disk cache, the rate limiter, and the database.

The handler lives at `GET /api/v0/epub?q=<url>` and returns JSON containing download URLs for all available formats. It's the most complex endpoint in FicHub, touching nearly every subsystem. Let's trace through every step.

### Step 1: Receiving the Request

The request arrives and is immediately validated:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }
```

The `start` variable lets us track how long the entire export takes. We reject empty queries and explicitly automated requests. The `ExportQuery` struct captures the query parameters:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,          // The URL to export
    pub automated: Option<String>,  // Flag indicating bot usage
    pub format: Option<String>,     // Optional format preference
}
```

All parameters are `Option` because they're not all required. The `q` parameter is the only one that matters for the basic flow.

### Step 2: Finding the Right Scraper

FicHub supports multiple fanfiction sites, each with its own scraper. The scraper registry matches the URL to the right scraper:

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(
        -5,
        format!("unsupported URL: {}", query)
    ))?;
```

The `ScraperRegistry` contains all registered scrapers (AO3, FFN, Royal Road, etc.). The `find_scraper` method examines the URL and returns the appropriate scraper. If the URL doesn't match any known pattern, we return error code -5.

This is the first point where the rate limiter comes into play—though in the current code, the rate limiter is checked separately before this handler is called (via middleware or in the scraper itself).

### Step 3: Looking Up Metadata

Before generating anything, we need to know what the story is. The scraper makes a lightweight HTTP request to get just the metadata:

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;

let info_request_ms = start.elapsed().as_millis() as i32;
```

This is intentionally separate from the chapter fetch. Most requests will find this story in the cache and never need to fetch chapters at all. The metadata lookup is the fast path that determines whether we need to do any heavy work.

The `info_request_ms` timestamp lets us track how long the metadata lookup took—useful for performance monitoring.

The handler then upserts the metadata into the database:

```rust
let fic_info_row = FicInfo {
    id: meta.url_id.clone(),
    title: meta.title.clone(),
    author: meta.author.clone(),
    author_url: Some(meta.author_url.clone()),
    author_local_id: Some(meta.author_local_id.clone()),
    chapters: meta.chapters,
    words: meta.words,
    description: meta.desc.clone(),
    fic_created: chrono::DateTime::from_timestamp_millis(meta.published)
        .unwrap_or_default(),
    fic_updated: chrono::DateTime::from_timestamp_millis(meta.updated)
        .unwrap_or_default(),
    status: meta.status.clone(),
    source: meta.source.clone(),
    extra_meta: meta.extra_meta.clone(),
    raw_extended_meta: meta.raw_extended_meta.clone(),
    source_id: Some(meta.source_id),
    author_id: Some(meta.author_id),
    content_hash: meta.content_hash.clone(),
};
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

This keeps our database in sync with the source site. If the author changes their pen name or updates the description, we'll pick it up on the next request. The `upsert` operation (insert or update) handles both new and existing stories.

### Step 4: Auto-Populating Tags

The handler also extracts tags from the scraper:

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

Tags (genres, warnings, characters, etc.) are extracted from the source site and stored in our database. The `resolve_tag` function either finds an existing tag or creates a new one. The `UNSPECIFIED` IP address indicates this is an automated tag extraction, not a user-submitted tag.

### Step 5: Checking Blacklists

Before generating any files, the handler checks if the story or author is blacklisted:

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    // Greylist: show metadata but no download
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        return Ok(build_metadata_response(
            &meta, &[], &state.config.export_version, None, true
        ));
    }
    // Hard blacklist
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted", "q": query})));
    }
}

let author_blacklist = queries::check_author_blacklist(
    &state.db, meta.source_id, meta.author_id,
).await?;
if !author_blacklist.is_empty() {
    return Ok(Json(json!({"err": -7, "msg": "author is blacklisted", "q": query})));
}
```

Blacklists serve multiple purposes:
- **Reason 5**: DMCA takedown requests
- **Reason 6**: Greylist (show metadata, no download)
- **Reason 7**: Content policy violations
- **Reason 8**: Other legal requirements

The greylist response shows the story's metadata (title, author, description) but doesn't provide download links. This lets users know the story exists without facilitating distribution.

### Step 6: Computing the Cache Version

The cache version combines the export module's version with any content-level bumps:

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
    .await?.unwrap_or(0);
let version = state.config.export_version + version_bump;

let input_hash = meta.content_hash.clone()
    .unwrap_or_else(|| "upstream".to_string());
```

The `export_version` is a global version number for the export module. The `version_bump` is per-story—incremented when the story's content changes on the source site. The `input_hash` is the story's content hash from the scraper, or `"upstream"` if no hash is available.

### Step 7: Checking the Cache

The cache check queries the database for a matching export:

```rust
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;
```

The `export_log` table stores previously generated exports. We look for a match on url_id, version, format, and input hash. If all four match, the cached file is still valid.

### Step 8: Cache Hit — Return Immediately

If the cache check finds a match, the handler builds URLs for all available formats and returns:

```rust
if let Some(export_log) = cached {
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

    // Check other formats too
    for etype_str in &["html", "mobi", "pdf"] {
        if let Ok(Some(entry)) = queries::find_export_log(
            &state.db, &meta.url_id, version, etype_str,
            &format!("epub:{}", epub_hash),
        ).await {
            hashes.insert(etype_str.to_string(), entry.export_hash.clone());
            urls.insert(etype_str.to_string(),
                format!("/cache/{}/{}?h={}",
                    etype_str, meta.url_id, entry.export_hash));
        }
    }

    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    return Ok(Json(json!({
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
    })));
}
```

On cache hit, the total response time is typically under 50 milliseconds—a database lookup and a JSON response. No HTTP requests to upstream sites, no file generation, no disk I/O beyond reading cached files when the client downloads them.

The handler also looks up HTML, MOBI, and PDF versions. MOBI and PDF have `input_hash` of `epub:{epub_hash}`—they depend on the EPUB hash, not the raw content hash. This creates a dependency chain.

### Step 9: Cache Miss — Acquire Semaphore and Double-Check

When the cache misses, we enter the expensive path—but first, we protect against duplicate concurrent work:

```rust
let sem = cache::get_export_semaphore(
    &state.cache_semaphores, &meta.url_id, &EType::Epub
).await;
let _permit = sem.acquire().await
    .map_err(|e| AppError::Internal(e.to_string()))?;

// Double-check: did another request finish while we waited?
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;
if let Some(export_log) = cached {
    // Another request did the work — use their result
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    return Ok(Json(json!({
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
    })));
}
```

This is the double-check pattern from Chapter 18 in action. The first request to acquire the semaphore will proceed to generate the file. All others will wait, then find the file already cached when they check again.

The `_permit` variable is important—it holds the semaphore permit. When `_permit` goes out of scope (at the end of the function or the `return` statement), the semaphore is automatically released. This is RAII (Resource Acquisition Is Initialization) in action.

### Step 10: Fetching Chapters

Now we do the heavy lifting—actually scraping the chapter content:

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

This is where the scraper (covered in Part 3) does its work: making HTTP requests, parsing HTML, extracting chapter content, and returning a vector of `Chapter` structs. The rate limiter ensures we don't overwhelm the upstream site.

For a story with 50 chapters, this might take 30-60 seconds. The rate limiter's per-IP bucket (one request every 8.6 seconds) means we need to pace our requests carefully. The global bucket (30 requests/second) allows multiple FicHub users to scrape different stories simultaneously.

The `Chapter` struct contains everything we need for export:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,   // Sequential chapter number
    pub title: String,     // Chapter title (e.g., "Chapter 1: The Beginning")
    pub content: String,   // Full HTML content of the chapter
}
```

The `content` field is HTML, not plain text. The scraper converts the source site's markup to a consistent HTML format. This means we can use it directly in the EPUB and HTML templates without further processing.

If the scraper fails (network error, parse error, rate limit), it returns a `ScrapeError`. The `?` operator propagates this error, which Axum converts to an appropriate HTTP response. The client receives an error JSON indicating what went wrong.

### Step 11: Generating EPUB and HTML

With the chapters in hand, we generate both export formats:

```rust
// Generate EPUB
let (epub_path, epub_hash) = export::epub::create_epub(
    &meta, &chapters, &state.config.tmp_dir
).await.map_err(|e| AppError::ExportError(e.to_string()))?;

// Generate HTML bundle
let (html_path, html_hash) = export::html_bundle::create_html_bundle(
    &meta, &chapters, &state.config.tmp_dir
).await.map_err(|e| AppError::ExportError(e.to_string()))?;
```

Both functions follow the same pattern: create a UUID workspace, build the output file, compute the MD5 hash, and return the path and hash.

### Step 12: Saving to Cache

Move the generated files from temporary directories to the cache:

```rust
// Cache EPUB
let cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash
);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;

// Record in export_log
queries::insert_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash
).await?;

// Cache HTML
let html_cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash
);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;

let html_input_hash = format!("epub:{}", epub_hash);
queries::insert_export_log(
    &state.db, &meta.url_id, version, "html", &html_input_hash, &html_hash
).await?;
```

Notice that the HTML bundle's input hash is `epub:{epub_hash}`. This creates a dependency chain: if the EPUB changes, the HTML bundle is also regenerated, even if the HTML template hasn't changed. This ensures consistency—the HTML and EPUB always represent the same version of the story.

The `insert_export_log` call records the export in the database. This is what the cache check looks for on subsequent requests. Without this record, every request would be a cache miss.

### Atomicity of Cache Operations

The cache operations are designed to be atomic:

1. **move_to_cache** uses `fs::rename`, which is atomic on the same filesystem.
2. **insert_export_log** is a single database INSERT (or UPSERT).
3. The semaphore ensures only one request performs these operations for a given story.

If the server crashes between `move_to_cache` and `insert_export_log`, the file is in the cache but not recorded in the database. The next request will regenerate the file (generating a new hash) and record it. The orphaned file will remain until manually cleaned up. This is acceptable because orphaned files don't cause errors—they just waste disk space.

### Step 13: Building the Response

The final response includes everything the client needs:

```rust
hashes.insert("epub".to_string(), epub_hash.clone());
hashes.insert("html".to_string(), html_hash.clone());
urls.insert("epub".to_string(),
    format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
urls.insert("html".to_string(),
    format!("/cache/html/{}?h={}", meta.url_id, html_hash));

let export_ms = start.elapsed().as_millis() as i32;
let slug = generate_slug(&meta.title, &meta.url_id);
let (info_str, notes) = build_info_string(&meta);

// Log the request
let fic_json = serde_json::to_string(&meta).ok();
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;

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

The request log records everything: how long the metadata lookup took, how long the export took, the hashes, and the original query. This data powers analytics and helps identify slow scrapers or frequent requests.

### The Response Structure

The JSON response tells the client everything it needs:

| Field | Type | Description |
|-------|------|-------------|
| `err` | integer | 0 = success. Negative values indicate specific errors. |
| `q` | string | The original URL that was requested |
| `url_id` | string | Unique identifier for the story |
| `slug` | string | URL-safe title identifier for friendly URLs |
| `info` | string | Human-readable story summary |
| `meta` | object | Full story metadata (title, author, dates, etc.) |
| `hashes` | object | Format → hash mapping (e.g., `{"epub": "abc123"}`) |
| `urls` | object | Format → download URL mapping |
| `epub_url` | string | Direct download URL for EPUB (shorthand) |
| `html_url` | string | Direct download URL for HTML (shorthand) |
| `notes` | array | Warnings or notices about the story |

The `epub_url` and `html_url` fields are shorthand duplicates of what's in `urls`. They exist because many clients only need the EPUB and HTML URLs, and it's convenient to have them at the top level.

### The generate_slug Function

The slug makes download URLs human-friendly:

```rust
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

A title like "Harry Potter and the Methods of Rationality" becomes `Harry_Potter_and_the_Methods_of_Rationality-abc123`. Special characters are replaced with underscores, consecutive underscores are collapsed, and the url_id is appended for uniqueness.

The slug is purely cosmetic—it doesn't affect caching or downloads. But it makes the API response more human-readable and helps with debugging. When you see `slug: "My_Story-abc123"` in a log entry, you immediately know which story it refers to.

### The build_info_string Function

The info string is a human-readable summary:

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

The relative time ("3 hours ago", "2 days ago") makes it easy for users to see how recent the story is. The `Vec<String>` for notes is empty in the basic case—it's used for greylisting and other special conditions.

The function returns a tuple of `(String, Vec<String>)` because some stories have notes attached (e.g., "This fic is greylisted - download links are not available."). The notes array can contain multiple strings for different conditions.

### Request Source Tracking

The export handler tracks where requests come from:

```rust
let source_id = queries::insert_request_source(
    &state.db, false, "/api/v0/epub", "web request",
).await?;
```

This tells us whether requests are coming from the web interface, the API, or automated tools. The `automated` parameter in the query string is a hint, but the real source tracking happens at the database level. This data helps answer questions like:
- How many API requests do we get per day?
- Which stories are most popular?
- Are there any clients hammering the API?

### The Cache Download Path

When the client follows a download URL like `/cache/epub/abc123?h=ef92b24f`, the `download_with_hash` handler serves the file:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    let hash = match params.h {
        Some(h) => h,
        None => {
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
            };
            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition",
                            &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

The hash verification is a security measure. Without it, an attacker could guess valid cache paths and download files that don't belong to them. The hash in the URL acts as a secret token—only someone with the hash (which is returned in the export response) can download the file.

The `Content-Disposition` header tells the browser to download the file rather than display it. The filename includes the url_id and the format suffix, giving the user a meaningful filename.

### The download_or_export Fallback

There's also a simpler download endpoint that handles cases where the client doesn't have a hash:

```rust
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash
            );
            if cache_path.exists() {
                // ... serve file if hash matches ...
            }
        }
    }

    // No cached file - redirect to the main page
    Redirect::to(&format!("/?id={}", url_id)).into_response()
}
```

If the client has a hash and the file exists, serve it. Otherwise, redirect to the frontend with the story ID so the frontend can trigger an export. This handles the case where someone shares a direct link without the hash parameter.

The redirect pattern is important: instead of returning a 404 or an error, we redirect to a page that can handle the request. This provides a better user experience—the user sees a loading screen while the export runs, rather than an error message.

### The Request Source Tracking

The export handler tracks where requests come from using the `insert_request_source` function. This tells us whether requests are coming from the web interface, the API, or automated tools. The `automated` parameter in the query string is a hint, but the real source tracking happens at the database level.

This data helps answer important questions:
- How many API requests do we get per day?
- Which stories are most popular?
- Are there any clients hammering the API?
- What's the ratio of cache hits to cache misses?

### Request Logging

Every export request is logged for analytics:

```rust
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;
```

The log includes:
- The source (API vs. web interface)
- The original query
- Timing data (metadata lookup time, total export time)
- The story metadata (as JSON)
- The output file hash

This data helps identify popular stories, slow scrapers, and performance bottlenecks. It also helps with debugging—if a user reports a corrupt EPUB, we can look up the export log and check the hash.

### Error Codes Reference

The export handler returns specific error codes for different failure modes:

| Code | Message | Meaning |
|------|---------|---------|
| 0 | Success | Export completed successfully |
| -1 | No query / Invalid format | Missing or malformed request |
| -5 | Unsupported URL | No scraper matches the URL |
| -7 | Blacklisted | Story or author is blocked |
| -10 | Automated requests blocked | Bot detection triggered |

These codes are documented in the API but not formally specified. If FicHub were to publish a public API, these would be formalized with descriptions and suggested client handling.

### The Full Flow in Summary

Here's the complete flow for a cache miss:

1. **Receive request** → Validate query, reject automated requests
2. **Find scraper** → Match URL to the right scraper
3. **Lookup metadata** → Lightweight HTTP request for story info
4. **Upsert database** → Store/update metadata in PostgreSQL
5. **Extract tags** → Parse genres, warnings, characters from source
6. **Check blacklists** → Verify the story isn't blocked
7. **Compute version** → Combine export version with content version
8. **Check cache** → Database lookup for existing export
9. **Acquire semaphore** → Prevent duplicate concurrent exports
10. **Double-check cache** → Verify another request didn't finish first
11. **Fetch chapters** → Full scrape with rate limiting
12. **Generate EPUB** → Build the e-book file
13. **Generate HTML** → Build the browser-readable bundle
14. **Move to cache** → Atomic file move to permanent storage
15. **Record in database** → Store export log for future lookups
16. **Log request** → Record analytics data
17. **Return response** → JSON with download URLs and metadata

For a cache hit, steps 10-15 are skipped. The total time drops from minutes (scraping + generation) to milliseconds (database lookup + JSON response).

### Design Decisions and Trade-offs

The export flow makes several deliberate design decisions:

**Why generate both EPUB and HTML on cache miss?**
Generating both at once means the next request for either format is a cache hit. The marginal cost of generating HTML after EPUB is small (maybe 200ms) compared to the cost of scraping (30+ seconds). It's almost always worth it.

**Why store the export log in PostgreSQL instead of Redis?**
The export log is authoritative data. Redis could lose it on a crash. PostgreSQL with WAL (Write-Ahead Logging) guarantees durability. The log also powers analytics queries that are easier to write in SQL than Redis.

**Why use the MD5 hash as the cache key?**
Content-addressable storage is simple and correct. If the content changes, the hash changes, and the cache naturally serves the new version. There's no need for explicit cache invalidation—the hash handles it.

**Why not use a CDN for cached files?**
FicHub currently serves files from disk. A CDN would improve download speed for distant users but adds complexity and cost. For the current scale, disk I/O is fast enough.

### Future Improvements

Several improvements could enhance the export system:

1. **Streaming EPUB generation**: For very large stories, generate the EPUB incrementally instead of loading all chapters into memory.
2. **Background export queue**: Move export generation to a background worker so the API responds immediately with a "processing" status.
3. **Format-agnostic caching**: Store the raw chapter data in the cache and generate formats on demand, reducing initial cache miss cost.
4. **Compression optimization**: Try different ZIP compression levels to find the best size/speed trade-off for HTML bundles.
5. **Parallel chapter fetching**: Fetch multiple chapters simultaneously (within rate limits) to reduce total scrape time.

### Performance Breakdown

Here's a rough breakdown of how long each step takes for a typical 50-chapter story:

| Step | Time | Notes |
|------|------|-------|
| Metadata lookup | 1-3 seconds | Single HTTP request |
| Database upsert | 5-10 ms | PostgreSQL INSERT/UPDATE |
| Tag extraction | 1-3 seconds | Depends on scraper |
| Cache check | 1-5 ms | PostgreSQL SELECT |
| Chapter fetching | 30-90 seconds | Rate-limited HTTP requests |
| EPUB generation | 100-500 ms | CPU-bound file I/O |
| HTML generation | 50-200 ms | Simpler than EPUB |
| Cache write | 10-50 ms | Atomic rename |
| **Total (cache miss)** | **~1-2 minutes** | Mostly chapter fetching |
| **Total (cache hit)** | **~10-50 ms** | Database + JSON only |

The bottleneck is always the chapter fetching. Everything else is fast. This is why caching is so valuable—once a story is cached, subsequent requests are 1000x faster.

### Error Recovery

The export handler has several error recovery mechanisms:

1. **Semaphore timeout**: If the semaphore can't be acquired (e.g., a previous export crashed and didn't release it), the handler returns an internal error. The semaphore's permit is released when the `_permit` variable is dropped, even on error.

2. **Calibre retry**: The `convert_epub` function retries failed conversions once before giving up. This handles transient Calibre errors.

3. **Cache validation**: The download handler recomputes MD5 before serving. Corrupted files are rejected with a "hash mismatch" error.

4. **Graceful degradation**: If tag extraction fails, the export continues without tags. If blacklisting fails, the export proceeds (fail open for availability).

### The AppState: Everything Connected

The `AppState` struct holds all the shared resources that the export handler needs:

```rust
pub struct AppState {
    pub config: Config,                    // Cache dir, tmp dir, versions
    pub db: sqlx::PgPool,                  // PostgreSQL connection pool
    pub redis: redis::aio::MultiplexedConnection,  // Redis for rate limiting
    pub http_client: reqwest::Client,      // HTTP client for scraping
    pub scraper_registry: Arc<ScraperRegistry>,     // Scraper lookup
    pub cache_semaphores: CacheSemaphores,          // Export deduplication
    pub rate_limiter: Box<dyn limiter::RateLimiter>, // Rate limiting
    pub recommender_engine: RecommendationEngine,   // Story recommendations
    pub collection_worker: CollectionWorker,        // Background collection
}
```

Every handler receives `State(state): State<Arc<AppState>>` from Axum. The `Arc` (Atomic Reference Counting) allows multiple handlers to share the same state without copying. The connection pools (`db`, `redis`) are themselves concurrent—multiple requests can use them simultaneously.

The `rate_limiter` is boxed as a trait object (`Box<dyn RateLimiter>`). This allows swapping implementations at startup—the application could use `RedisBucketLimiter` in production and a mock limiter in tests.

The `config` field holds filesystem paths:

```rust
pub struct Config {
    pub cache_dir: PathBuf,   // Where cached exports live
    pub tmp_dir: PathBuf,     // Where temporary files are created
    pub export_version: i32,  // Current export module version
    // ... database URL, Redis URL, etc.
}
```

The `cache_dir` and `tmp_dir` paths must be on the same filesystem for `fs::rename` to work. If they're on different mounts, the move will fail. This is a common gotcha when deploying to containerized environments where `/tmp` might be a tmpfs mount.

> 🧪 **Try It Yourself**: Use `curl` to make an export request and trace the full flow:
> ```
> curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/123456" | python3 -m json.tool
> ```
> Note the `epub_url` and `html_url` in the response. Follow the URL to download the file. Now make the same request again—the second response should be much faster because it's a cache hit. Check the timing in the logs to see the difference.

> ⚠️ **Watch Out**: The export handler doesn't currently use the rate limiter for incoming requests—it only applies rate limiting to *outgoing* scraper requests. If FicHub becomes popular, you'd want to add incoming rate limiting to prevent abuse of the export endpoint itself. A simple middleware that checks the client's IP against a Redis-backed limit would work well here. The `RedisBucketLimiter` already has per-IP support—it just needs to be wired into the Axum middleware layer.

> ⚠️ **Watch Out**: The `Semaphore::acquire()` call can fail if the semaphore is closed (all permits have been dropped). In practice, this shouldn't happen with our code, but the `.map_err()` call handles it gracefully. If it ever does fail, it means the semaphore infrastructure has a bug—the application logs the error and returns an internal error response.

---

*Part 4 has shown how FicHub transforms raw scraped data into polished, downloadable files. The EPUB generator creates proper e-books with metadata, styling, and chapter navigation. The HTML bundle produces portable, browser-readable archives with responsive design. The disk cache ensures we only generate each file once, using content-addressable storage with hash-based directory structures. The Redis-backed token bucket rate limiter keeps our upstream sites happy with globally-coordinated, per-IP pacing. And the end-to-end export flow ties everything together with double-checked locking, semaphore-based deduplication, and comprehensive error handling. In Part 5, we'll explore how the database layer stores metadata, tracks exports, and powers the search features that help users discover new stories.*

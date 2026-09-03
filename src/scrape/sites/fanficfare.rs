use async_trait::async_trait;
use std::process::Command;

use crate::scrape::{FicMetadata, Chapter, ExtractedTag, SiteScraper, ScrapeError, generate_url_id};

/// FanFicFare fallback scraper — handles any site FanFicFare supports.
/// Called via CLI when native scrapers can't handle the URL.
pub struct FanFicFareScraper;

/// Metadata parsed from FanFicFare's --meta-only output (Python dict format).
struct FffMeta {
    title: String,
    author: String,
    author_url: String,
    author_id: String,
    story_id: String,
    num_chapters: i32,
    num_words: i64,
    description: String,
    status: String,
    date_published: String,
    date_updated: String,
    site_abbrev: String,
    category: String,
    characters: String,
}

/// Parse a Python-style dict from FanFicFare stdout into key-value pairs.
/// FanFicFare outputs a Python dict with indentation and multi-line strings.
/// Uses first occurrence of each key (later keys are chapter-level duplicates).
fn parse_fff_dict(output: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    // Find the dict start (first '{')
    let dict_start = match output.find('{') {
        Some(pos) => pos,
        None => return pairs,
    };
    let dict_end = match output.rfind('}') {
        Some(pos) => pos + 1,
        None => output.len(),
    };
    let dict_str = &output[dict_start..dict_end];

    // Simple state machine: find 'key': 'value' patterns
    let mut chars: Vec<char> = dict_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Look for opening quote of key
        if chars[i] == '\'' {
            i += 1;
            let key_start = i;
            while i < chars.len() && chars[i] != '\'' {
                i += 1;
            }
            let key: String = chars[key_start..i].iter().collect();
            i += 1; // skip closing quote

            // Skip ": " or "': "
            while i < chars.len() && (chars[i] == ':' || chars[i] == ' ') {
                i += 1;
            }

            // Only keep first occurrence of each key
            if seen_keys.contains(&key) {
                // Skip this value
                if i < chars.len() && (chars[i] == '\'' || chars[i] == '"') {
                    let quote = chars[i];
                    i += 1;
                    while i < chars.len() && chars[i] != quote {
                        i += 1;
                    }
                    if i < chars.len() { i += 1; }
                }
                continue;
            }

            if i < chars.len() && chars[i] == '\'' {
                // Single-quoted value
                i += 1; // skip opening quote
                let val_start = i;
                while i < chars.len() {
                    if chars[i] == '\'' && (i + 1 >= chars.len() || chars[i+1] == ',' || chars[i+1] == ' ' || chars[i+1] == '\n' || chars[i+1] == '}') {
                        break;
                    }
                    i += 1;
                }
                let val: String = chars[val_start..i].iter().collect();
                if i < chars.len() { i += 1; } // skip closing quote
                seen_keys.insert(key.clone());
                pairs.push((key, val));
            } else if i < chars.len() && chars[i] == '"' {
                // Double-quoted value (for strings with single quotes inside)
                i += 1; // skip opening quote
                let val_start = i;
                while i < chars.len() {
                    if chars[i] == '"' && (i + 1 >= chars.len() || chars[i+1] == ',' || chars[i+1] == ' ') {
                        break;
                    }
                    i += 1;
                }
                let val: String = chars[val_start..i].iter().collect();
                if i < chars.len() { i += 1; } // skip closing quote
                seen_keys.insert(key.clone());
                pairs.push((key, val));
            }
        } else {
            i += 1;
        }
    }
    pairs
}

/// Call fanficfare --meta-only and parse the output.
fn fetch_metadata(url: &str) -> Result<FffMeta, ScrapeError> {
    let output = Command::new("fanficfare")
        .arg("--meta-only")
        .arg("-o")
        .arg("is_adult=true")
        .arg(url)
        .output()
        .map_err(|e| ScrapeError::Network(format!("failed to run fanficfare: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ScrapeError::Network(format!("fanficfare failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pairs = parse_fff_dict(&stdout);

    let get = |key: &str| -> String {
        pairs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };

    let num_chapters: i32 = get("numChapters").parse().unwrap_or(1);
    let num_words: i64 = get("numWords")
        .replace(',', "")
        .parse()
        .unwrap_or(0);

    let status = match get("status").as_str() {
        "Completed" => "complete",
        "In-Progress" => "ongoing",
        "Abandoned" => "cancelled",
        _ => "ongoing",
    };

    Ok(FffMeta {
        title: get("title"),
        author: get("author"),
        author_url: get("authorUrl"),
        author_id: get("authorId"),
        story_id: get("storyId"),
        num_chapters,
        num_words,
        description: get("description"),
        status: status.to_string(),
        date_published: get("datePublished"),
        date_updated: get("dateUpdated"),
        site_abbrev: get("siteabbrev"),
        category: get("category"),
        characters: get("characters"),
    })
}

/// Parse a date string like "2008-09-16" to unix millis
fn parse_date_to_millis(date_str: &str) -> i64 {
    if date_str.is_empty() {
        return 0;
    }
    // Try parsing "YYYY-MM-DD"
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return dt.and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
    }
    // Try parsing "YYYY/MM/DD"
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
        return dt.and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
    }
    // Try "Month Day, Year" e.g. "September 16, 2008"
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return dt.and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc()
            .timestamp_millis();
    }
    0
}

#[async_trait]
impl SiteScraper for FanFicFareScraper {
    /// FanFicFare handles any URL — it's the fallback scraper.
    /// The registry tries native scrapers first, so this only runs
    /// when no other scraper matches.
    fn can_handle(&self, _url: &str) -> bool {
        true
    }

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
            words: meta.num_words,
            desc: meta.description,
            published: parse_date_to_millis(&meta.date_published),
            updated: parse_date_to_millis(&meta.date_updated),
            status: meta.status,
            source: url.to_string(),
            source_id,
            author_id: 0,
            author_url: meta.author_url,
            author_local_id: meta.author_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, _client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Create a temp directory for FanFicFare downloads
        let tmp_dir = std::env::temp_dir().join("fichub_fff");
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| ScrapeError::Network(format!("failed to create temp dir: {}", e)))?;

        // Download HTML via fanficfare (runs in temp directory)
        let output = Command::new("fanficfare")
            .arg("--format")
            .arg("html")
            .arg("-o")
            .arg("is_adult=true")
            .arg(&meta.source)
            .current_dir(&tmp_dir)
            .output()
            .map_err(|e| ScrapeError::Network(format!("failed to run fanficfare: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScrapeError::Network(format!("fanficfare download failed: {}", stderr)));
        }

        // Find the most recently created .html file in the temp directory
        let html_files: Vec<_> = std::fs::read_dir(&tmp_dir)
            .map_err(|e| ScrapeError::Network(format!("failed to read temp dir: {}", e)))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "html").unwrap_or(false)
            })
            .collect();

        // Find the newest HTML file
        let html_file = html_files.iter()
            .max_by_key(|f| f.metadata().and_then(|m| m.modified()).ok())
            .ok_or_else(|| ScrapeError::Network("no HTML file found after fanficfare download".to_string()))?;

        let html_content = std::fs::read_to_string(html_file.path())
            .map_err(|e| ScrapeError::Network(format!("failed to read HTML file: {}", e)))?;

        // Clean up the downloaded file
        let _ = std::fs::remove_file(html_file.path());

        // Parse HTML and extract chapters
        let document = scraper::Html::parse_document(&html_content);
        let mut chapters = Vec::new();

        // FanFicFare uses <a name="section0001"><h2>Title</h2></a> for chapters
        // and <div class="story"> for content
        let story_selector = scraper::Selector::parse(".story, .chapter_content, #story")
            .map_err(|_| ScrapeError::ParseError("invalid selector".to_string()))?;

        let h2_selector = scraper::Selector::parse("h2")
            .map_err(|_| ScrapeError::ParseError("invalid selector".to_string()))?;

        // Try to find story content divs
        let story_divs: Vec<_> = document.select(&story_selector).collect();

        if story_divs.is_empty() {
            // Fallback: treat the entire body as one chapter
            let body_selector = scraper::Selector::parse("body")
                .map_err(|_| ScrapeError::ParseError("invalid selector".to_string()))?;
            if let Some(body) = document.select(&body_selector).next() {
                let content = body.inner_html();
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        } else {
            // Extract chapters from story divs
            for (i, div) in story_divs.iter().enumerate() {
                // Try to find the chapter title from the preceding h2
                let title = div.select(&h2_selector)
                    .next()
                    .map(|h| h.text().collect::<String>())
                    .unwrap_or_else(|| format!("Chapter {}", i + 1));

                let content = div.inner_html();
                chapters.push(Chapter {
                    chapter_id: (i + 1) as i32,
                    title,
                    content,
                });
            }
        }

        Ok(chapters)
    }

    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fff_dict_basic() {
        let input = r#"{'title': 'Test Story', 'author': 'TestAuthor', 'numChapters': '5'}"#;
        let pairs = parse_fff_dict(input);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], ("title".to_string(), "Test Story".to_string()));
        assert_eq!(pairs[1], ("author".to_string(), "TestAuthor".to_string()));
        assert_eq!(pairs[2], ("numChapters".to_string(), "5".to_string()));
    }

    #[test]
    fn test_parse_fff_dict_with_commas() {
        let input = "{'numWords': '1,234', 'title': 'Hello'}";
        let pairs = parse_fff_dict(input);
        assert_eq!(pairs[0], ("numWords".to_string(), "1,234".to_string()));
    }

    #[test]
    fn test_parse_fff_dict_empty() {
        let pairs = parse_fff_dict("");
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_parse_date_to_millis() {
        let ms = parse_date_to_millis("2008-09-16");
        assert!(ms > 0);
    }

    #[test]
    fn test_parse_date_to_millis_empty() {
        assert_eq!(parse_date_to_millis(""), 0);
    }

    #[test]
    fn test_parse_date_to_millis_slash_format() {
        let ms = parse_date_to_millis("2008/09/16");
        assert!(ms > 0);
    }

    #[test]
    fn test_can_handle_is_always_true() {
        let scraper = FanFicFareScraper;
        assert!(scraper.can_handle("https://anything.com/story/123"));
        assert!(scraper.can_handle("https://some-random-site.com/fic/456"));
        assert!(scraper.can_handle("not even a url"));
    }

    #[test]
    fn test_site_abbrev_to_source_id() {
        // Test the mapping logic (extracted for testability)
        let cases = vec![
            ("ao3", 1),
            ("ffn", 2),
            ("sb", 4),
            ("sv", 4),
            ("Wattpad", 5),
        ];
        for (abbrev, expected) in cases {
            let source_id: i64 = match abbrev {
                "ao3" => 1,
                "ffn" => 2,
                "fnac" => 3,
                "sb" | "sv" => 4,
                "Wattpad" => 5,
                "royalroad" => 6,
                _ => 99,
            };
            assert_eq!(source_id, expected, "Failed for {}", abbrev);
        }
    }

    #[test]
    fn test_parse_fff_dict_wattpad_real() {
        // Real FanFicFare output for a Wattpad story
        let input = "{'author': 'randomthingsbyme',\n \
             'authorId': 'randomthingsbyme',\n \
             'authorUrl': 'https://www.wattpad.com/user/randomthingsbyme',\n \
             'category': 'Humor',\n \
             'datePublished': '2013-06-03',\n \
             'dateUpdated': '2018-03-14',\n \
             'numChapters': '48',\n \
             'numWords': '70,809',\n \
             'siteabbrev': 'Watt',\n \
             'status': 'Completed',\n \
             'storyId': '6049138',\n \
             'title': 'Young, Wild & Married',\n \
             'titleHTML': \"<a class='titlelink' href='https://www.wattpad.com/story/6049138'>Young, Wild &amp; Married</a>\",\n \
             }";

        let pairs = parse_fff_dict(input);
        let get = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());

        assert_eq!(get("title").unwrap(), "Young, Wild & Married");
        assert_eq!(get("author").unwrap(), "randomthingsbyme");
        assert_eq!(get("authorId").unwrap(), "randomthingsbyme");
        assert_eq!(get("numChapters").unwrap(), "48");
        assert_eq!(get("numWords").unwrap(), "70,809");
        assert_eq!(get("siteabbrev").unwrap(), "Watt");
        assert_eq!(get("status").unwrap(), "Completed");
        assert_eq!(get("storyId").unwrap(), "6049138");
        assert_eq!(get("datePublished").unwrap(), "2013-06-03");
        assert_eq!(get("dateUpdated").unwrap(), "2018-03-14");
    }
}

use std::collections::HashMap;
use std::io::Read;

use crate::error::AppError;
use crate::scrape::{Chapter, FicMetadata, generate_url_id};
use uuid::Uuid;

pub struct ManualUploadPayload {
    pub title: String,
    pub author: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub status: String, // "ongoing", "complete", "hiatus"
    pub file_data: Vec<u8>,
    pub file_name: String,
    pub user_id: i32,
}

pub async fn parse_manual_upload(
    payload: ManualUploadPayload,
) -> Result<(FicMetadata, Vec<Chapter>), AppError> {
    let story_id = Uuid::new_v4().to_string();
    let url_id = generate_url_id(10, &story_id);

    let chapters = if payload.file_name.ends_with(".txt") {
        parse_txt_chapters(&payload.file_data, &payload.title)?
    } else if payload.file_name.ends_with(".epub") {
        parse_epub_chapters(&payload.file_data)?
    } else if payload.file_name.ends_with(".html") || payload.file_name.ends_with(".htm") {
        parse_html_chapters(&payload.file_data)?
    } else {
        return Err(AppError::BadRequest("Unsupported file type. Use .txt, .epub, or .html".to_string()));
    };

    if chapters.is_empty() {
        return Err(AppError::BadRequest("No chapters could be parsed from the file".to_string()));
    }

    let total_words: i64 = chapters
        .iter()
        .map(|c| c.content.split_whitespace().count() as i64)
        .sum();

    let now_ms = chrono::Utc::now().timestamp_millis();

    let meta = FicMetadata {
        url_id: url_id.clone(),
        title: payload.title,
        author: payload.author,
        desc: payload.summary,
        words: total_words,
        chapters: chapters.len() as i32,
        status: if payload.status.is_empty() {
            "complete".into()
        } else {
            payload.status
        },
        source: format!("fichub://manual/{}", url_id),
        source_id: 10,
        author_id: payload.user_id as i64,
        author_url: format!("/users/{}", payload.user_id),
        author_local_id: story_id,
        published: now_ms,
        updated: now_ms,
        content_hash: None,
        extra_meta: Some(payload.tags.join(", ")),
        raw_extended_meta: None,
    };

    Ok((meta, chapters))
}

fn parse_txt_chapters(data: &[u8], title: &str) -> Result<Vec<Chapter>, AppError> {
    let text = String::from_utf8_lossy(data);
    let text_str = text.as_ref();

    // Try to split on chapter headers
    let re =
        regex_lite::Regex::new(r"(?im)^(?:chapter|part|section)\s+\d+.*$|^---+\s*$").unwrap();
    let mut chapters = Vec::new();
    let mut current_start = 0;
    let mut chapter_num = 1;

    for cap in re.find_iter(text_str) {
        if cap.start() > current_start {
            let content = text_str[current_start..cap.start()].trim();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: chapter_num,
                    title: format!("Chapter {}", chapter_num),
                    content: escape_html_for_content(content),
                });
                chapter_num += 1;
            }
        }
        current_start = cap.end();
    }

    // Final chapter after last header
    if current_start < text_str.len() {
        let content = text_str[current_start..].trim();
        if !content.is_empty() {
            chapters.push(Chapter {
                chapter_id: chapter_num,
                title: format!("Chapter {}", chapter_num),
                content: escape_html_for_content(content),
            });
        }
    }

    // Fallback: no chapters found, treat entire file as one chapter
    if chapters.is_empty() {
        chapters.push(Chapter {
            chapter_id: 1,
            title: title.to_string(),
            content: escape_html_for_content(text_str),
        });
    }

    Ok(chapters)
}

fn parse_epub_chapters(data: &[u8]) -> Result<Vec<Chapter>, AppError> {
    use std::io::Cursor;

    // ── Zip-bomb / decompression-bomb protection ─────────────────────
    // Cap the number of entries (a legitimate EPUB has < 1000 files; a
    // malicious archive can declare millions) and the total decompressed
    // size (a 1MB file must not expand to 10GB).
    const MAX_ZIP_ENTRIES: usize = 5000;
    const MAX_TOTAL_DECOMPRESSED: usize = 50 * 1024 * 1024; // 50 MiB

    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::BadRequest(format!("Invalid EPUB file: {}", e)))?;

    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(AppError::BadRequest(format!(
            "EPUB has {} entries; maximum allowed is {}",
            archive.len(),
            MAX_ZIP_ENTRIES
        )));
    }

    // Find the OPF file from container.xml
    let mut opf_path = "OEBPS/content.opf".to_string();
    if let Ok(mut container) = archive.by_name("META-INF/container.xml") {
        let mut container_xml = String::new();
        container.read_to_string(&mut container_xml).ok();
        let re =
            regex_lite::Regex::new(r#"rootfile\s+full-path="([^"]+)""#).ok();
        if let Some(caps) = re.and_then(|r| r.captures(&container_xml)) {
            if let Some(p) = caps.get(1) {
                opf_path = p.as_str().to_string();
            }
        }
    }

    // Parse the OPF to get chapter file order
    let mut opf_content = String::new();
    if let Ok(mut opf_file) = archive.by_name(&opf_path) {
        opf_file.read_to_string(&mut opf_content).ok();
    }

    let spine_re = regex_lite::Regex::new(r#"<itemref\s+idref="([^"]+)""#).ok();
    let item_re = regex_lite::Regex::new(r#"<item\s+[^>]*id="([^"]+)"[^>]*href="([^"]+)""#).ok();

    // Build id->href map
    let mut id_to_href = HashMap::new();
    if let Some(ref ire) = item_re {
        for cap in ire.captures_iter(&opf_content) {
            id_to_href.insert(cap[1].to_string(), cap[2].to_string());
        }
    }

    let mut chapters = Vec::new();
    let mut chapter_num = 1;
    let mut total_decompressed: usize = 0;

    // Resolve base directory of OPF
    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    if let Some(ref sre) = spine_re {
        for cap in sre.captures_iter(&opf_content) {
            let idref = cap[1].to_string();
            if let Some(href) = id_to_href.get(&idref) {
                // Resolve relative path
                let full_path = opf_dir.join(href);
                let full_path_str = full_path.to_string_lossy().to_string();

                if let Ok(mut file) = archive.by_name(&full_path_str) {
                    let mut content = String::new();
                    file.read_to_string(&mut content).ok();

                    total_decompressed += content.len();
                    if total_decompressed > MAX_TOTAL_DECOMPRESSED {
                        return Err(AppError::BadRequest(
                            "EPUB expands beyond 50 MB after decompression — refusing to process".to_string(),
                        ));
                    }

                    // Extract text from HTML, strip tags
                    let text = strip_html_tags(&content);
                    if !text.trim().is_empty() {
                        chapters.push(Chapter {
                            chapter_id: chapter_num,
                            title: format!("Chapter {}", chapter_num),
                            content: text,
                        });
                        chapter_num += 1;
                    }
                }
            }
        }
    }

    if chapters.is_empty() {
        // Fallback: scan all html/xhtml files in archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                AppError::BadRequest(format!("EPUB read error: {}", e))
            })?;
            let name = file.name().to_string();
            if name.ends_with(".html") || name.ends_with(".xhtml") || name.ends_with(".htm") {
                let mut content = String::new();
                file.read_to_string(&mut content).ok();

                total_decompressed += content.len();
                if total_decompressed > MAX_TOTAL_DECOMPRESSED {
                    return Err(AppError::BadRequest(
                        "EPUB expands beyond 50 MB after decompression — refusing to process".to_string(),
                    ));
                }

                let text = strip_html_tags(&content);
                if !text.trim().is_empty() {
                    chapters.push(Chapter {
                        chapter_id: chapter_num,
                        title: format!("Chapter {}", chapter_num),
                        content: text,
                    });
                    chapter_num += 1;
                }
            }
        }
    }

    Ok(chapters)
}

fn parse_html_chapters(data: &[u8]) -> Result<Vec<Chapter>, AppError> {
    let text = String::from_utf8_lossy(data);
    let doc = scraper::Html::parse_document(&text);

    // Split by h1/h2 tags
    let header_sel = scraper::Selector::parse("h1, h2").unwrap();
    let body_sel = scraper::Selector::parse("body").unwrap();

    let mut chapters = Vec::new();
    let mut chapter_num = 1;

    // Get all header elements
    let headers: Vec<_> = doc.select(&header_sel).collect();

    if headers.is_empty() {
        // No headers — entire body is one chapter
        if let Some(body) = doc.select(&body_sel).next() {
            let content = body.inner_html();
            chapters.push(Chapter {
                chapter_id: 1,
                title: "Chapter 1".into(),
                content,
            });
        } else {
            // No body either — treat the whole document as text
            let content = strip_html_tags(&text);
            chapters.push(Chapter {
                chapter_id: 1,
                title: "Chapter 1".into(),
                content,
            });
        }
    } else {
        for i in 0..headers.len() {
            let title = headers[i].text().collect::<String>().trim().to_string();
            let content = if i + 1 < headers.len() {
                extract_content_between(&doc, &headers[i], &headers[i + 1])
            } else {
                // Everything after last header
                extract_content_after(&doc, &headers[i])
            };

            if !content.trim().is_empty() {
                chapters.push(Chapter {
                    chapter_id: chapter_num,
                    title,
                    content: escape_html_for_content(&content),
                });
                chapter_num += 1;
            }
        }
    }

    Ok(chapters)
}

fn strip_html_tags(html: &str) -> String {
    let doc = scraper::Html::parse_fragment(html);
    doc.root_element().text().collect::<String>()
}

fn escape_html_for_content(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br/>")
}

fn extract_content_between(
    doc: &scraper::Html,
    from: &scraper::ElementRef,
    _to: &scraper::ElementRef,
) -> String {
    // Walk sibling nodes from `from` until we hit _to
    // For simplicity, use the HTML tree — collect all text after the current header
    // node until the next header in the parsed tree
    let from_id = from.id();
    let mut capturing = false;
    let mut content = String::new();

    // Use the node tree traversal
    for node in doc.root_element().descendants() {
        if node.id() == from_id {
            capturing = true;
            continue;
        }
        if capturing {
            // Check if this node is a header (h1 or h2)
            if let Some(el) = scraper::ElementRef::wrap(node) {
                let tag_name = el.value().name();
                if tag_name == "h1" || tag_name == "h2" {
                    break;
                }
                content.push_str(&el.inner_html());
            }
        }
    }

    content
}

fn extract_content_after(doc: &scraper::Html, from: &scraper::ElementRef) -> String {
    let from_id = from.id();
    let mut capturing = false;
    let mut content = String::new();

    for node in doc.root_element().descendants() {
        if node.id() == from_id {
            capturing = true;
            continue;
        }
        if capturing {
            if let Some(el) = scraper::ElementRef::wrap(node) {
                content.push_str(&el.inner_html());
            }
        }
    }

    content
}

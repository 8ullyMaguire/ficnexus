//! SpiritFanfiction.com native adapter (Spanish/Portuguese archive).
//!
//! Story page: `https://www.spiritfanfiction.com/historia/{id}`.
//! - title: `h1.tituloPrincipal strong`
//! - authors: `div.left span.usuario a` (multi-author)
//! - chapters: `table.listagemCapitulos tr.listagem-textoBg1 a` with
//!   `time[datetime]` publish dates
//! - summary: `div.texto` (after "Sinopse:")
//! - info box: `div.texto.espacamentoTop` — label rows (Iniciado,
//!   Atualizada, Idioma, Palavras, Concluído, Categorias, Personagens...)
//! - rating: `div.classificacao` class suffix
//! Chapter body: `div.texto-capitulo` + `div.texto-capitulo-notas`
//! (Notas do Autor / Notas Finais).

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct SpiritFanfictionScraper;

#[async_trait]
impl SiteScraper for SpiritFanfictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("spiritfanfiction.com/historia/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        // Title
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("h1.tituloPrincipal strong") {
            title = doc
                .select(&h1_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("spirit: no title".into()));
        }

        // Story id
        let story_id = url
            .split("/historia/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.rsplit('-').next())
            .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or("")
            .to_string();

        // Authors (multi-author supported)
        let mut authors: Vec<String> = Vec::new();
        let mut author_urls: Vec<String> = Vec::new();
        if let Ok(a_sel) = Selector::parse("div.left span.usuario a") {
            for a in doc.select(&a_sel) {
                authors.push(a.text().collect::<String>().trim().to_string());
                if let Some(h) = a.value().attr("href") {
                    author_urls.push(format!("https://www.spiritfanfiction.com{h}"));
                }
            }
        }
        let author = authors.join(", ");
        let author_url = author_urls.first().cloned().unwrap_or_default();
        let author_local_id = author_url.rsplit('/').next().unwrap_or("").to_string();

        // Description
        let mut desc = String::new();
        if let Ok(d_sel) = Selector::parse("div.clearfix div.texto") {
            if let Some(el) = doc.select(&d_sel).next() {
                desc = el.inner_html();
            }
        }

        // Status + words from the info box.
        let mut status = "ongoing".to_string();
        let mut words = 0i64;
        let mut rating = String::new();
        if let Ok(info_sel) = Selector::parse("div.texto.espacamentoTop") {
            for el in doc.select(&info_sel) {
                let text = el.text().collect::<String>();
                if text.contains("Concluído") {
                    status = if text.contains("Sim") {
                        "complete"
                    } else {
                        "ongoing"
                    }
                    .to_string();
                }
                if text.contains("Palavras") {
                    words = text
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0);
                }
            }
        }
        if let Ok(r_sel) = Selector::parse("div.classificacao") {
            if let Some(el) = doc.select(&r_sel).next() {
                if let Some(class) = el.value().attr("class") {
                    rating = class.rsplit('-').next().unwrap_or("").to_string();
                }
            }
        }

        // Chapters
        let mut chapters = 0i32;
        if let Ok(c_sel) = Selector::parse("table.listagemCapitulos tr.listagem-textoBg1 a") {
            chapters = doc.select(&c_sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("spi_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: now,
            updated: now,
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!("rating={};", rating)),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let html = http::fetch(client, &meta.source).await?;

        // Collect chapter links (title, href) into owned data first.
        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let c_sel = Selector::parse("table.listagemCapitulos tr.listagem-textoBg1 a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&c_sel)
                .filter_map(|el| {
                    let title = el.text().collect::<String>().trim().to_string();
                    let href = el.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.spiritfanfiction.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
                content,
            });
        }
        Ok(chapters)
    }

    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str,
    ) -> Result<Vec<crate::ExtractedTag>, ScrapeError> {
        Ok(vec![])
    }
}

/// Fetch a SpiritFanfiction chapter and extract `div.texto-capitulo` +
/// author notes.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    let mut out = String::new();

    // Author notes (Notas do Autor / Notas Finais) in texto-capitulo-notas.
    if let Ok(note_sel) = Selector::parse("div.texto.texto-capitulo-notas") {
        for el in doc.select(&note_sel) {
            out.push_str(&format!(
                "<div class=\"fff_chapter_notes\">{}</div>",
                el.inner_html()
            ));
        }
    }
    if let Ok(sel) = Selector::parse("div.texto-capitulo") {
        if let Some(el) = doc.select(&sel).next() {
            out.push_str(&el.inner_html());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = SpiritFanfictionScraper;
        assert!(s.can_handle("https://www.spiritfanfiction.com/historia/1234567/titulo"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id_from_slug() {
        let url = "https://www.spiritfanfiction.com/historia/titulo-de-historia-1234567/capitulo1";
        let id = url
            .split("/historia/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.rsplit('-').next())
            .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap();
        assert_eq!(id, "1234567");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="texto-capitulo"><p>Spanish story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div.texto-capitulo") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Spanish story text."));
    }
}

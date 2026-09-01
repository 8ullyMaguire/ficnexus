//! Search commands: `/search`, `/ask`, `/quote`, `/body`.
//!
//! `/search` maps to `GET /api/search` (advanced search with filters).
//! `/ask` maps to `POST /api/search/ask` (LLM-assisted natural-language
//! search — the "Ask the Archive" feature).
//! `/quote` fetches a fic's metadata + a snippet from body search.
//! `/body` runs full-text body search.

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::api::SearchParams;
use crate::cache::PageEntry;
use crate::commands::{search_embed, Context, Data, Error};
use crate::model::{AskResponse, BodySearchHit, SearchResult};
use crate::util::truncate;

/// `/search <query>` — advanced search across the archive.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("search"),
    description_localized("en-US", "Search the FicHub archive (advanced filters)")
)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Search query (title, author, fandom, tags)"] q: String,
    #[description = "Minimum word count"] min_words: Option<i64>,
    #[description = "Maximum word count"] max_words: Option<i64>,
    #[description = "Only completed works"] complete: Option<bool>,
    #[description = "Source site (e.g. archiveofourown.org)"] source: Option<String>,
    #[description = "Sort: relevance | kudos | words | updated | created"] sort: Option<String>,
    #[description = "Require at least N kudos"] min_kudos: Option<i64>,
) -> Result<(), Error> {
    let data = ctx.data();
    let params = SearchParams {
        q,
        min_words,
        max_words,
        complete,
        source,
        sort,
        min_kudos,
        page: Some(1),
        per_page: Some(data.config.api_page_size),
        ..Default::default()
    };
    let resp = data.client.search(&params).await?;
    render_search(ctx, data, resp.total, resp.results, &params.q).await
}

/// `/ask <natural language>` — LLM-assisted search.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("ask"),
    description_localized("en-US", "Ask the archive in natural language (LLM-assisted search)")
)]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "Natural-language request, e.g. 'dark harry potter complete over 50k'"] q: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let resp = data.client.ask(&q).await?;
    render_ask(ctx, data, &resp).await
}

/// `/quote <url>` — fetch metadata + a body snippet for a fic URL.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("quote"),
    description_localized("en-US", "Get metadata + a sample snippet for a fic")
)]
pub async fn quote(
    ctx: Context<'_>,
    #[description = "URL of the fic"] url: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let export = data.client.fetch_export(&url).await?;
    let meta = export.meta.ok_or_else(|| Error::Command("No metadata for that URL.".into()))?;
    let mut e = CreateEmbed::new()
        .title(truncate(&meta.title, 200))
        .url(&url)
        .description(truncate(&meta.description, 500))
        .color(0xf1c40f)
        .field("Author", truncate(&meta.author, 80), true)
        .field("Words", crate::util::format_words(meta.words), true)
        .field("Status", &meta.status, true)
        .field("Source", truncate(&meta.source, 60), true);
    if meta.chapters > 0 {
        e = e.field("Chapters", meta.chapters.to_string(), true);
    }
    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}

/// `/body <query>` — full-text body search (snippet highlighting).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("body"),
    description_localized("en-US", "Search the text of fic bodies (snippet highlighting)")
)]
pub async fn body(
    ctx: Context<'_>,
    #[description = "Text to search inside fic bodies"] q: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let resp = data
        .client
        .body_search(&q, 1, data.config.api_page_size)
        .await?;
    if resp.err != 0 {
        return Err(Error::Command(format!("Body search error: {}", resp.err)));
    }
    if resp.results.is_empty() {
        ctx.say("No body matches.").await?;
        return Ok(());
    }
    // Convert to search-result-like entries and render via pagination cache.
    let results: Vec<SearchResult> = resp
        .results
        .iter()
        .map(|h| search_result_from_hit(h))
        .collect();
    render_search(ctx, data, resp.total, results, &q).await
}

/// Convert a body-search hit to a `SearchResult` for shared rendering.
fn search_result_from_hit(h: &BodySearchHit) -> SearchResult {
    SearchResult {
        url_id: h.url_id.clone(),
        title: h.title.clone(),
        author: h.author.clone(),
        source: h.source.clone(),
        words: h.words,
        chapters: h.chapters,
        status: h.status.clone(),
        description: h.description.clone(),
        updated: None,
        rank: None,
        snippet: h.body_snippet.clone(),
        tags: Vec::new(),
        total_freeform: 0,
        comment_count: 0,
        kudos_count: 0,
    }
}

/// Shared: cache search results + render page 1.
pub(crate) async fn render_search(
    ctx: Context<'_>,
    data: &Data,
    total: i64,
    results: Vec<SearchResult>,
    query: &str,
) -> Result<(), Error> {
    if results.is_empty() {
        ctx.say(format!("No results for `{query}`.")).await?;
        return Ok(());
    }
    let title = format!("Search: {query}");
    let entries: Vec<PageEntry> = results
        .iter()
        .enumerate()
        .map(|(i, r)| PageEntry {
            index: i,
            item: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
        })
        .collect();
    let session_id = data
        .cache
        .store_for_user(ctx.author().id.get(), &title, entries)
        .await?;
    let total = total.max(results.len() as i64);

    let page_size = data.config.page_size;
    let shown: Vec<_> = results.iter().take(page_size).collect();
    let mut embeds: Vec<CreateEmbed> = shown
        .iter()
        .enumerate()
        .map(|(i, r)| search_embed(r, i, total as usize))
        .collect();

    // Footer: total count on the last embed.
    if let Some(last) = embeds.last_mut() {
        let footer = serenity::builder::CreateEmbedFooter::new(format!("{total} results · Page 1"));
        *last = last.clone().footer(footer);
    }

    let mut reply = poise::CreateReply::default().components(vec![crate::commands::recs::pagination_row(
        &session_id,
        total as usize,
        page_size,
        1,
    )]);
    for e in embeds {
        reply = reply.embed(e);
    }
    ctx.send(reply).await?;
    Ok(())
}

/// Render an `/ask` response (search results + translation metadata).
pub(crate) async fn render_ask(
    ctx: Context<'_>,
    data: &Data,
    resp: &AskResponse,
) -> Result<(), Error> {
    let mut msg = String::new();
    if resp.used_ask {
        msg.push_str("**Ask the Archive** interpreted your request");
        if let Some(t) = &resp.translation {
            msg.push_str(&format!(": `{}`", truncate(&t.to_string(), 200)));
        }
        msg.push('\n');
    }
    if resp.search.results.is_empty() {
        msg.push_str("No results.");
        ctx.say(msg).await?;
        return Ok(());
    }
    let entries: Vec<PageEntry> = resp
        .search
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| PageEntry {
            index: i,
            item: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
        })
        .collect();
    let title = format!("Ask: {}", resp.ask_query.clone().unwrap_or_default());
    let session_id = data
        .cache
        .store_for_user(ctx.author().id.get(), &title, entries)
        .await?;

    let page_size = data.config.page_size;
    let shown: Vec<_> = resp.search.results.iter().take(page_size).collect();
    let embeds: Vec<CreateEmbed> = shown
        .iter()
        .enumerate()
        .map(|(i, r)| search_embed(r, i, resp.search.total as usize))
        .collect();

    let mut reply = poise::CreateReply::default()
        .content(msg)
        .components(vec![crate::commands::recs::pagination_row(
            &session_id,
            resp.search.total as usize,
            page_size,
            1,
        )]);
    for e in embeds {
        reply = reply.embed(e);
    }
    ctx.send(reply).await?;
    Ok(())
}

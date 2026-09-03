//! Full favourites browse: `/full-favourites` — paginate your bookmarked list.
//!
//! Bot-only: caches your full bookmark list in Redis and lets you page through
//! it with `/full-favourites`, `/full-favourites page N`.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/full-favourites [page]` — browse your full bookmarked list (paginated).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("full-favourites", "fullfav"),
    description_localized("en-US", "Browse your full bookmarked list"),
    ephemeral
)]
pub async fn full_favourites(
    ctx: Context<'_>,
    #[description = "Page number (default: 1)"] page: Option<i64>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    // Fetch + cache the full bookmark list.
    let bookmarks = data.client.list_bookmarks(&token, None, None).await?;
    let items: Vec<serde_json::Value> = bookmarks
        .get("items")
        .and_then(|v| v.as_array())
        .map_or_else(|| vec![], |a| a.to_vec());

    if items.is_empty() {
        ctx.say("You have no bookmarks.").await?;
        return Ok(());
    }

    // Cache in Redis for quick re-access.
    let ids: Vec<i64> = items.iter().filter_map(|i| i.get("work_id").and_then(|v| v.as_i64())).collect();
    let _ = data.tokens.set_full_favourites(&ctx.author().id.get().to_string(), &ids).await;

    let page = page.unwrap_or(1).max(1);
    let per_page = 10i64;
    let total = items.len() as i64;
    let total_pages = (total + per_page - 1) / per_page;
    let page = page.min(total_pages).max(1);
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(total as usize);

    let mut msg = format!("**Your bookmarks — page {}/{} ({} total)**\n", page, total_pages.max(1), total);
    for item in &items[start..end] {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("?");
        let author = item.get("author").and_then(|v| v.as_str()).unwrap_or("?");
        let url_id = item.get("url_id").and_then(|v| v.as_str()).unwrap_or("?");
        msg.push_str(&format!("- /fic/{url_id} **{}** by {}\n", truncate(title, 60), truncate(author, 40)));
    }
    ctx.say(msg).await?;
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}

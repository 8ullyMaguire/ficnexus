//! Fandom directory commands: `/fandoms` and `/fandom <slug>`.
//!
//! Wraps `GET /api/fandoms` (returns `SearchResponse` with fandom facets)
//! and `GET /api/fandoms/{slug}` (returns a fandom detail JSON object).

use poise::serenity_prelude as serenity;

use crate::commands::Context;
use crate::util::truncate;

/// `/fandoms` — list all fandoms with fic counts.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("fandoms"),
    description_localized("en-US", "List all fandoms with fic counts"),
    ephemeral
)]
pub async fn fandoms(ctx: Context<'_>) -> Result<(), crate::error::DiscordBotError> {
    let data = ctx.data();
    let resp = data.client.list_fandoms().await?;
    let fandoms: Vec<serde_json::Value> = resp
        .facets
        .fandoms
        .iter()
        .cloned()
        .collect();
    if fandoms.is_empty() {
        ctx.say("No fandoms tagged yet.").await?;
        return Ok(());
    }
    let mut msg = format!("**{} fandoms**\n", fandoms.len());
    for f in fandoms.iter().take(20) {
        let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let slug = f.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
        let count = f.get("fic_count").and_then(|v| v.as_i64()).unwrap_or(0);
        msg.push_str(&format!(
            "**{}** — {} fics\n`/fandom {}`\n",
            truncate(name, 60),
            count,
            slug
        ));
    }
    if fandoms.len() > 20 {
        msg.push_str(&format!("...and {} more\n", fandoms.len() - 20));
    }
    ctx.say(msg).await?;
    Ok(())
}

/// `/fandom <slug>` — show fandom detail + top fics.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("fandom"),
    description_localized("en-US", "Show a specific fandom's details and top fics"),
    ephemeral
)]
pub async fn fandom(ctx: Context<'_>, #[description = "Fandom slug"] slug: String) -> Result<(), crate::error::DiscordBotError> {
    let data = ctx.data();
    let slug = slug.to_ascii_lowercase();
    let resp: serde_json::Value = data.client.get_fandom(&slug).await?;
    let fandom = resp.get("fandom").cloned().unwrap_or_default();
    let name = fandom.get("name").and_then(|v| v.as_str()).unwrap_or("?");
    let count = fandom.get("fic_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let words = fandom.get("total_words").and_then(|v| v.as_i64()).unwrap_or(0);
    let authors = fandom.get("active_authors").and_then(|v| v.as_i64()).unwrap_or(0);
    let mut msg = format!(
        "**{}** — {} fics, {} words, {} active authors\n",
        truncate(name, 60),
        count,
        truncate(&words.to_string(), 30),
        authors
    );
    let top_fics: Vec<serde_json::Value> = fandom
        .get("top_fics")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if top_fics.is_empty() {
        msg.push_str("No fics tagged yet.\n");
    } else {
        msg.push_str("**Top Fics:**\n");
        for fic in top_fics.iter().take(5) {
            let title = fic.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let author = fic.get("author").and_then(|v| v.as_str()).unwrap_or("?");
            let wid = fic.get("url_id").and_then(|v| v.as_str()).unwrap_or("?");
            let wc = fic.get("words").and_then(|v| v.as_i64()).unwrap_or(0);
            let ch = fic.get("chapters").and_then(|v| v.as_i64()).unwrap_or(0);
            let score = fic.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            msg.push_str(&format!(
                "  /fic/{} **{}** by {} — {} words, {} ch{}\n",
                wid,
                truncate(title, 60),
                truncate(author, 30),
                wc,
                ch,
                if score > 0.0 { format!(" ⭐ {:.1}", score) } else { String::new() }
            ));
        }
        if top_fics.len() > 5 {
            msg.push_str(&format!("  ...and {} more\n", top_fics.len() - 5));
        }
    }
    ctx.say(msg).await?;
    Ok(())
}

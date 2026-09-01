//! Author bibliography: `/authors <name>` — show an author's works.
//!
//! Wraps `GET /api/authors/by-name/{name}`.

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::commands::Context;
use crate::util::truncate;

/// `/authors <name>` — show an author's bibliography.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("authors"),
    description_localized("en-US", "Show an author's bibliography of works"),
    ephemeral
)]
pub async fn authors(
    ctx: Context<'_>,
    #[description = "Author name (or username)"] name: String,
) -> Result<(), crate::error::DiscordBotError> {
    let data = ctx.data();
    let encoded = urlencoding::encode(&name);
    let resp = data.client.author_bibliography(&encoded).await?;

    let works: Vec<serde_json::Value> = resp
        .get("works")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if works.is_empty() {
        ctx.say(format!("No works found for author `{name}`.")).await?;
        return Ok(());
    }

    let total_words: i64 = works.iter().map(|w| w.get("words").and_then(|v| v.as_i64()).unwrap_or(0)).sum();
    let total_fics = works.len() as i64;
    let mut msg = format!("**{name}** — {total_fics} fic(s), {} words\n", crate::util::format_words(total_words));

    for w in works.iter().take(10) {
        let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("?");
        let url_id = w.get("url_id").and_then(|v| v.as_str()).unwrap_or("?");
        let words = w.get("words").and_then(|v| v.as_i64()).unwrap_or(0);
        let status = w.get("status").and_then(|v| v.as_str()).unwrap_or("?");
        msg.push_str(&format!("  /fic/{url_id} **{title}** — {words} words · {status}\n"));
    }

    ctx.say(msg).await?;
    Ok(())
}

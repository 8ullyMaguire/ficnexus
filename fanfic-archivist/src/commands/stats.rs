//! User reading stats: `/stats` — show your reading profile.
//!
//! Wraps `GET /api/users/me/reading-stats`.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/stats` — show your reading stats (wordcount distribution, fandom diversity, etc.).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("stats"),
    description_localized("en-US", "Show your reading stats (wordcount, fandom diversity, etc.)"),
    ephemeral
)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let resp = data.client.my_reading_stats(&token).await?;
    let stats: serde_json::Value = resp.get("stats").cloned().unwrap_or_default();

    let total_words: i64 = stats.get("total_words").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_books: i64 = stats.get("total_books").and_then(|v| v.as_i64()).unwrap_or(0);
    let fandoms: i64 = stats.get("fandoms").and_then(|v| v.as_i64()).unwrap_or(0);
    let crossover_ratio: f64 = stats.get("crossover_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let avg_words: i64 = stats.get("avg_words").and_then(|v| v.as_i64()).unwrap_or(0);
    let longest: i64 = stats.get("longest").and_then(|v| v.as_i64()).unwrap_or(0);
    let complete_ratio: f64 = stats.get("complete_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0);

    ctx.say(format!(
        "**Your reading stats**\n\
         · {total_books} books · {total_words} total words (avg {avg_words}/fic)\n\
         · {fandoms} fandoms · crossover ratio {:.1}%\n\
         · {:.0}% complete · longest: {longest} words",
        (crossover_ratio * 100.0) as u64,
        (complete_ratio * 100.0) as u64,
    )).await?;
    Ok(())
}

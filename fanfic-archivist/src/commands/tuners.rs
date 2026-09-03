//! Recs range tuners: `!cutoff`, `!wordcount`, `!year`.
//!
//! Bot-only — persist per-user recs preferences in Redis (cutoff, wordcount
//! range, publication year). These feed the search params when the recs/search
//! commands build their `SearchParams`.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `!cutoff <N>` — set minimum wordcount for recommendations.
#[poise::command(
    prefix_command,
    slash_command,
    aliases("cutoff"),
    description_localized("en-US", "Set minimum wordcount for recs"),
    ephemeral
)]
pub async fn cutoff(
    ctx: Context<'_>,
    #[description = "Minimum word count (e.g. 50000)"] cutoff: i64,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    data.tokens.set_recs_cutoff(&ctx.author().id.get().to_string(), cutoff).await?;
    ctx.say(format!("Recs minimum wordcount cutoff set to {} words.", cutoff)).await?;
    Ok(())
}

/// `!wordcount <min>-<max>` — set wordcount range for recommendations.
#[poise::command(
    prefix_command,
    slash_command,
    aliases("wordcount"),
    description_localized("en-US", "Set wordcount range for recs (min-max, e.g. 50000-200000)"),
    ephemeral
)]
pub async fn wordcount(
    ctx: Context<'_>,
    #[description = "Wordcount range: min-max (e.g. 50000-200000)"] range: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        ctx.say("Usage: !wordcount <min>-<max> (e.g. 50000-200000)").await?;
        return Ok(());
    }
    let min: i64 = parts[0].parse().map_err(|_| {
        Error::Command("Invalid minimum wordcount. Use a number.".into())
    })?;
    let max: i64 = parts[1].parse().map_err(|_| {
        Error::Command("Invalid maximum wordcount. Use a number.".into())
    })?;
    if min > max {
        ctx.say("Minimum must be <= maximum.").await?;
        return Ok(());
    }
    data.tokens.set_recs_wordcount_range(&ctx.author().id.get().to_string(), min, max).await?;
    ctx.say(format!("Recs wordcount range set to {}–{} words.", min, max)).await?;
    Ok(())
}

/// `!year <year>` — filter recommendations by publication year.
#[poise::command(
    prefix_command,
    slash_command,
    aliases("year"),
    description_localized("en-US", "Filter recs by publication year"),
    ephemeral
)]
pub async fn year(
    ctx: Context<'_>,
    #[description = "Publication year (e.g. 2024)"] year: i64,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    data.tokens.set_recs_year(&ctx.author().id.get().to_string(), year).await?;
    ctx.say(format!("Recs publication year filter set to {}.", year)).await?;
    Ok(())
}

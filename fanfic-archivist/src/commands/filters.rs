//! Client-side recs filter chips: `!filters` / `!reset-filters`.
//!
//! Bot-only — modify persisted filter preferences used by `recs` to narrow
//! recommendations. No server endpoint needed.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `!filters [chip1 chip2 ...]` — set recs filter chips.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("filters"),
    description_localized("en-US", "Set recs filter chips (completed, hiatus, unpopular, classic, long, short, top-rated, new)"),
    ephemeral
)]
pub async fn filters(
    ctx: Context<'_>,
    #[description = "One or more filter chips to apply"] chips: Vec<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let valid_chips = ["completed", "hiatus", "unpopular", "classic", "long", "short", "top-rated", "new"];
    let mut invalid = Vec::new();
    let mut applied = Vec::new();
    for chip in &chips {
        let lower = chip.to_ascii_lowercase();
        if valid_chips.contains(&lower.as_str()) {
            applied.push(lower);
        } else {
            invalid.push(chip.clone());
        }
    }
    if !invalid.is_empty() {
        ctx.say(format!(
            "Unknown chips: {}. Valid: completed, hiatus, unpopular, classic, long, short, top-rated, new.",
            invalid.join(", ")
        )).await?;
        return Ok(());
    }
    data.tokens.set_filter_chips(&ctx.author().id.get().to_string(), &applied).await?;
    ctx.say(format!("Filter chips set to: {}", applied.join(", "))).await?;
    Ok(())
}

/// `!reset-filters` — clear all recs filter chips to default.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("reset-filters"),
    description_localized("en-US", "Reset all recs filter chips to default"),
    ephemeral
)]
pub async fn reset_filters(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    data.tokens.set_filter_chips(&ctx.author().id.get().to_string(), &vec![]).await?;
    ctx.say("Filter chips reset to default (none).").await?;
    Ok(())
}

//! OPFS/PWA deep-link: `/opds` — link to FicHub OPDS + PWA offline reader.
//! Bot-only feature — no server endpoint needed. Points users to FicHub's OPDS
//! catalog and PWA for offline reading.

use poise::serenity_prelude as serenity;

use crate::commands::Context;

/// `/opds` — link to FicHub OPDS catalog + PWA offline reader.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("opds"),
    description_localized("en-US", "Link to FicHub OPDS catalog and PWA offline reader"),
    ephemeral
)]
pub async fn opds(ctx: Context<'_>) -> Result<(), crate::error::DiscordBotError> {
    let _base = crate::config::BotConfig::default().url("");
    ctx.say(format!(
        "**FicHub OPDS + PWA**\n\
         · OPDS catalog: `/opds`\n\
         · PWA offline reader: `/reader`\n\
         · Add to home screen for offline reading."
    )).await?;
    Ok(())
}

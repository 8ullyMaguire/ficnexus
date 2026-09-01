//! Suggest a fic as a community recommendation.
//!
//! Wraps `POST /api/recommendations/suggest` — the 2026-08-25 batch endpoint
//! that lets readers propose fics for the recommendation engine.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/suggest <url> [comment]` — suggest a fic for the recommendation engine.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("suggest"),
    description_localized("en-US", "Suggest a fic for the community recommendation engine"),
    ephemeral
)]
pub async fn suggest(
    ctx: Context<'_>,
    #[description = "Fic URL to suggest"] url: String,
    #[description = "Optional comment (why this fic deserves recs)"] comment: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    // Normalize the URL and extract the url_id.
    let normalized = crate::util::normalize_url(&url);
    let export = data.client.fetch_export(&normalized).await?;
    let url_id = export
        .url_id
        .ok_or_else(|| Error::Command("Could not resolve that URL to a work id.".into()))?;

    match data.client.suggest_fic(&token, &url_id, comment.as_deref()).await {
        Ok(resp) => {
            let id = resp.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let net_votes = resp.get("net_votes").and_then(|v| v.as_i64()).unwrap_or(0);
            ctx.say(format!(
                "✅ Suggested **{url_id}**. Suggestion #{id} · current net votes: {net_votes}",
            ))
            .await?;
        }
        Err(e) => {
            ctx.say(format!("Could not suggest that fic: {e}")).await?;
        }
    }
    Ok(())
}

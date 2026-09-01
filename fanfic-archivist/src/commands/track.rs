//! Tracked-message state machine: `/track <url_id> [state]`.
//!
//! Bot-only feature — tracks a message (update, rec, search result) through
//! states: unseen → seen → saved → archived. Stores state transitions in
//! Redis per user. No server endpoint needed.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/track <url_id> [state]` — track message states.
/// States: unseen, seen, saved, archived, advance.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("track"),
    description_localized("en-US", "Track a message state: unseen, seen, saved, archived, or advance"),
    ephemeral
)]
pub async fn track(
    ctx: Context<'_>,
    #[description = "The url_id of the work/message to track"] url_id: String,
    #[description = "Target state: unseen | seen | saved | archived (default: advance)"] state: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let url_id = crate::util::normalize_url(&url_id);
    let target = state.as_deref().unwrap_or("advance");
    let key = format!("track:{}", url_id);

    match target {
        "unseen" => { data.tokens.set_track_state(&ctx.author().id.get().to_string(), &key, "unseen").await?; }
        "seen" => { data.tokens.set_track_state(&ctx.author().id.get().to_string(), &key, "seen").await?; }
        "saved" => { data.tokens.set_track_state(&ctx.author().id.get().to_string(), &key, "saved").await?; }
        "archived" => { data.tokens.set_track_state(&ctx.author().id.get().to_string(), &key, "archived").await?; }
        "advance" => {
            let current = data.tokens.get_track_state(&ctx.author().id.get().to_string(), &key).await?;
            let next = match current.as_deref() {
                Some("unseen") => "seen",
                Some("seen") => "saved",
                Some("saved") => "archived",
                _ => "seen",
            };
            data.tokens.set_track_state(&ctx.author().id.get().to_string(), &key, next).await?;
            ctx.say(format!("Tracked `{}`: {} → {}", url_id, current.as_deref().unwrap_or("unseen"), next)).await?;
            return Ok(());
        }
        _ => {
            ctx.say(format!("Unknown state `{}`. Use: unseen, seen, saved, archived, or advance.", target)).await?;
            return Ok(());
        }
    }
    ctx.say(format!("Tracked `{}` → {}", url_id, target)).await?;
    Ok(())
}

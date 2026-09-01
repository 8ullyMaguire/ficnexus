//! Exclude command: `/exclude <work_id>` / `!exclude <work_id>`.
//!
//! Hides a work (or series/fandom) from your follow's updates feed.
//! Server-side anti-join removes excluded items from `GET /api/v1/updates`.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// `/exclude <work_id>` — exclude a work from your follow's updates.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("exclude"),
    description_localized("en-US", "Exclude a work from your follow updates"),
    ephemeral
)]
pub async fn exclude(
    ctx: Context<'_>,
    #[description = "Work (fic) id to exclude from updates"] work_id: i64,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    match data.client.add_follow_exclusion(
        &token,
        0, // personal follow — server resolves from token
        &crate::api::FollowExclusionBody {
            exclude_type: "work".into(),
            work_id: Some(work_id as i32),
            series_id: None,
            fandom: None,
        },
    ).await {
        Ok(_) => {
            ctx.say(format!(
                "Excluded work `#{work_id}` from your updates. New chapters will no longer appear."
            )).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not exclude work: {e}")).await?;
        }
    }
    Ok(())
}

/// `/exclude series <series_id>` — exclude a series from updates.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("exclude-series"),
    description_localized("en-US", "Exclude a series from your follow updates"),
    ephemeral
)]
pub async fn exclude_series(
    ctx: Context<'_>,
    #[description = "Series id to exclude"] series_id: i64,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    match data.client.add_follow_exclusion(
        &token,
        0,
        &crate::api::FollowExclusionBody {
            exclude_type: "series".into(),
            work_id: None,
            series_id: Some(series_id as i32),
            fandom: None,
        },
    ).await {
        Ok(_) => {
            ctx.say(format!("Excluded series `#{series_id}` from your updates.")).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not exclude series: {e}")).await?;
        }
    }
    Ok(())
}

/// `/exclude fandom <fandom>` — exclude a fandom from updates.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("exclude-fandom"),
    description_localized("en-US", "Exclude a fandom from your follow updates"),
    ephemeral
)]
pub async fn exclude_fandom(
    ctx: Context<'_>,
    #[description = "Fandom name to exclude"] fandom: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let fandom_owned = fandom.clone();
    match data.client.add_follow_exclusion(
        &token,
        0,
        &crate::api::FollowExclusionBody {
            exclude_type: "fandom".into(),
            work_id: None,
            series_id: None,
            fandom: Some(fandom),
        },
    ).await {
        Ok(_) => {
            ctx.say(format!("Excluded fandom `{}` from your updates.", fandom_owned)).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not exclude fandom: {e}")).await?;
        }
    }
    Ok(())
}

//! Community commands: Fic Requests board + Roadmap consensus.
//!
//! `/request` — browse / create fic requests (the "Fic Requests" community
//! feature from the web app). `/roadmap` — browse the roadmap consensus
//! leaderboard + controversy.

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::commands::{require_token, Context, Error};
use crate::model::ClusterRow;
use crate::util::truncate;

/// `/request list [status]` — browse the Fic Requests board.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("requests"),
    description_localized("en-US", "Browse the Fic Requests board"),
    subcommands("request_list", "request_create"),
)]
pub async fn request(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// `!requests [status]` — text alias for listing requests.
#[poise::command(prefix_command, aliases("requests_list"))]
pub async fn request_list(
    ctx: Context<'_>,
    #[description = "Status: open | answered | closed"] status: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let status = status.unwrap_or_else(|| "open".to_string());
    let resp = data.client.list_requests(&status, 1).await?;
    if resp.items.is_empty() {
        ctx.say(format!("No {status} requests.")).await?;
        return Ok(());
    }
    let mut lines = format!("**Fic Requests — {status}**\n");
    for (_i, item) in resp.items.iter().take(10).enumerate() {
        lines.push_str(&format!(
            "`#{}` **{}** — {} answers, {} upvotes\n> {}\n",
            item.id,
            truncate(&item.title, 60),
            item.answer_count,
            item.upvotes,
            truncate(&item.body, 100)
        ));
    }
    ctx.say(lines).await?;
    Ok(())
}

/// `/request create <title> <body>` — submit a fic request (auth).
#[poise::command(slash_command, ephemeral)]
pub async fn request_create(
    ctx: Context<'_>,
    #[description = "Request title"] title: String,
    #[description = "Request body"] body: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;
    match data.client.create_request(&token, &title, &body).await {
        Ok(_) => {
            ctx.say(format!("📝 Request posted: **{title}**")).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not create request: {e}")).await?;
        }
    }
    Ok(())
}

/// `/roadmap` — roadmap consensus leaderboard + controversy.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("roadmap"),
    description_localized("en-US", "FicHub roadmap consensus leaderboard"),
    subcommands("roadmap_top", "roadmap_controversial"),
)]
pub async fn roadmap(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// `!roadmap` — text alias for the leaderboard.
#[poise::command(prefix_command, aliases("roadmap_top"))]
pub async fn roadmap_top(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let resp = data.client.consensus().await?;
    if resp.leaderboard.is_empty() {
        ctx.say("No consensus data yet.").await?;
        return Ok(());
    }
    let top: Vec<&ClusterRow> = resp.leaderboard.iter().take(10).collect();
    let mut e = CreateEmbed::new()
        .title("FicHub Roadmap — Consensus Leaderboard")
        .color(0x3498db)
        .description("Top roadmap ideas by Elo (community voting).");
    for (i, row) in top.iter().enumerate() {
        e = e.field(
            format!("{}. (Elo {:.0})", i + 1, row.elo_rating),
            truncate(&row.text, 150),
            false,
        );
    }
    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}

/// `/roadmap controversial` — most controversial ideas.
#[poise::command(slash_command)]
pub async fn roadmap_controversial(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let resp = data.client.consensus().await?;
    if resp.controversy.is_empty() {
        ctx.say("No controversy data yet.").await?;
        return Ok(());
    }
    let top: Vec<&ClusterRow> = resp.controversy.iter().take(10).collect();
    let mut e = CreateEmbed::new()
        .title("FicHub Roadmap — Most Controversial")
        .color(0xe74c3c)
        .description("Ideas with the biggest best/worst split.");
    for (i, row) in top.iter().enumerate() {
        e = e.field(
            format!("{}. (controversy {})", i + 1, row.controversy),
            truncate(&row.text, 150),
            false,
        );
    }
    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}

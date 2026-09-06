//! Roadmap commands: Kanban board, changelog, and suggest.
//!
//! Wraps the three migration-064/065 endpoints:
//! - `GET /api/roadmap/features` — Kanban board (8 statuses, categories)
//! - `GET /api/roadmap/changelog` — public changelog feed
//! - `POST /api/roadmap/suggest` — submit a feature idea (auth)

use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbed;

use crate::commands::{get_valid_token, require_token, Context, Error};
use crate::util::truncate;

/// Status emoji + label map for the Kanban board.
fn status_emoji(status: &str) -> &'static str {
    match status {
        "idea" => "💡",
        "long_term" => "🏛️",
        "medium_term" => "📅",
        "up_next" => "🔜",
        "in_progress" => "🚧",
        "finished" => "✅",
        "shipped" => "🚀",
        "rejected" => "❌",
        _ => "⚪",
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "idea" => "Idea",
        "long_term" => "Long Term",
        "medium_term" => "Medium Term",
        "up_next" => "Up Next",
        "in_progress" => "In Progress",
        "finished" => "Finished",
        "shipped" => "Shipped",
        "rejected" => "Rejected",
        _ => "Other",
    }
}

fn kind_emoji(kind: &str) -> &'static str {
    match kind {
        "new" => "🆕",
        "improved" => "✨",
        "fixed" => "🐛",
        _ => "📝",
    }
}

/// `/roadmap-board [status|category]` — browse the Kanban board.
#[poise::command(slash_command, prefix_command, aliases("roadmap-board", "roadmap_kanban"))]
pub async fn roadmap_board(
    ctx: Context<'_>,
    #[description = "Filter by status (idea|up_next|in_progress|shipped|...)"] status: Option<String>,
    #[description = "Filter by category"] category: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let resp = data
        .client
        .features(status.as_deref(), category.as_deref())
        .await
        .map_err(|e| crate::BotError::Other(format!("could not load features: {e}")))?;

    if resp.features.is_empty() {
        ctx.say("No features match that filter.").await?;
        return Ok(());
    }

    // Group by status for kanban-style output.
    let mut by_status: std::collections::HashMap<&str, Vec<&crate::model::FeatureCluster>> =
        std::collections::HashMap::new();
    for f in &resp.features {
        by_status.entry(f.status.as_str()).or_default().push(f);
    }

    let title = if let (Some(s), None) = (&status, &category) {
        format!("Roadmap — {}", status_label(s))
    } else if let (None, Some(c)) = (&status, &category) {
        format!("Roadmap — category: {}", c)
    } else {
        "Roadmap Kanban Board".to_string()
    };

    let mut e = CreateEmbed::new()
        .title(&title)
        .color(0x3498db)
        .description("Top ideas by community Elo rating.");

    // Show up to 10 per status in field order.
    let status_order = [
        "up_next", "in_progress", "finished", "shipped", "idea", "medium_term",
        "long_term", "rejected",
    ];
    for s in status_order {
        if let Some(rows) = by_status.get(s) {
            let label = format!("{} {}", status_emoji(s), status_label(s));
            let text: String = rows
                .iter()
                .take(5)
                .map(|f| {
                    format!(
                        "`#{}` {:.0} Elo — {}",
                        f.id,
                        f.elo_rating,
                        truncate(&f.representative_text, 80)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !text.is_empty() {
                e = e.field(label, text, false);
            }
        }
    }

    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}

/// `/roadmap-suggest <idea>` — submit a feature idea (auth required).
#[poise::command(slash_command, ephemeral)]
pub async fn roadmap_suggest(
    ctx: Context<'_>,
    #[description = "Your feature idea (max 1000 chars)"] idea: String,
) -> Result<(), Error> {
    if idea.trim().is_empty() {
        ctx.say("Please provide a feature idea.").await?;
        return Ok(());
    }
    if idea.len() > 1000 {
        ctx.say("Idea is too long (max 1000 characters).").await?;
        return Ok(());
    }

    let token = require_token(&ctx.data().tokens, ctx.author().id.get()).await?;
    ctx.data().tokens.touch(&ctx.author().id.get().to_string()).await?;

    match ctx.data().client.suggest(&token, &idea).await {
        Ok(resp) => {
            if resp.clustered {
                ctx.say(format!(
                    "💡 Idea submitted and grouped with cluster #{}.",
                    resp.cluster_id.unwrap_or(0)
                ))
                .await?;
            } else {
                ctx.say("💡 Idea submitted as a new cluster. You'll vote on it in the next arena!")
                    .await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Could not submit idea: {e}")).await?;
        }
    }
    Ok(())
}

/// `/changelog [new|improved|fixed] [limit]` — browse recent changelog entries.
#[poise::command(slash_command, prefix_command, aliases("changelog"))]
pub async fn changelog(
    ctx: Context<'_>,
    #[description = "Filter: new | improved | fixed"] kind: Option<String>,
    #[description = "Max entries (default 10, max 50)"] limit: Option<i64>,
) -> Result<(), Error> {
    let limit = limit.unwrap_or(10).min(50);
    let data = ctx.data();

    let resp = data
        .client
        .changelog(kind.as_deref(), None, Some(limit))
        .await
        .map_err(|e| crate::BotError::Other(format!("could not load changelog: {e}")))?;

    if resp.changelog.is_empty() {
        ctx.say("No changelog entries yet.").await?;
        return Ok(());
    }

    let filter_label = kind
        .as_ref()
        .map(|k| format!(" ({})", k))
        .unwrap_or_default();
    let mut e = CreateEmbed::new()
        .title(format!("Changelog{}", filter_label))
        .color(0x2ecc71);

    for entry in resp.changelog.iter().take(10) {
        let label = format!(
            "{} #{}",
            kind_emoji(&entry.kind),
            entry.id,
        );
        let text = format!(
            "**{}**\n{}\n_by user {} · {} ago_",
            truncate(&entry.title, 100),
            truncate(&entry.body, 200),
            entry.author_id,
            &entry.published_at[..10],
        );
        e = e.field(label, text, false);
    }

    ctx.send(poise::CreateReply::default().embed(e)).await?;
    Ok(())
}

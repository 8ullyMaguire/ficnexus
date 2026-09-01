//! `/work` command: show full work detail for Discord.
//!
//! Reuses the platform-agnostic `presenter::format_work_detail`
//! so the same logic powers `/work`, `fichub work <id>`, and future
//! Telegram/Matrix ports.

use poise::serenity_prelude as serenity;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};

use crate::commands::{Context, Error};
use crate::presenter::format_work_detail;
use crate::util::truncate;

/// `/work <work_id>` — show detailed info about a fic.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("work"),
    description_localized("en-US", "Show detailed info about a fic")
)]
pub async fn work(
    ctx: Context<'_>,
    #[description = "FicHub work ID (numeric)"] work_id: i64,
) -> Result<(), Error> {
    let data = ctx.data();

    let work_resp = data.client.work_detail(work_id).await?;
    let Some(work) = work_resp.work else {
        ctx.say(format!("No work found with ID **{work_id}**."))
            .await?;
        return Ok(());
    };

    let stats = data.client.work_stats(work_id).await.ok();
    let text = format_work_detail(&work, stats.as_ref());

    let title = truncate(&work.canonical_title, 200);
    let author_name = truncate(&work.canonical_author, 80);

    let mut embed = CreateEmbed::new()
        .title(format!("#{work_id} — {title}"))
        .description(truncate(&work.description, 400))
        .color(0x9b59b6)
        .footer(CreateEmbedFooter::new(format!(
            "_by {}_ · Updated {}",
            author_name,
            truncate(&work.updated_at, 20)
        )));

    // Engagement stats.
    let stats_line = stats.as_ref().map(|s| {
        let total = s.kudos_count + s.guest_count;
        format!(
            "❤️ {} total kudos ({} verified)\n📌 {} bookmarks\n⭐ {} ratings\n💬 {} comments\n👁️ {} views",
            total,
            s.kudos_count,
            s.total_bookmarks,
            s.total_ratings,
            s.total_comments,
            s.total_views.unwrap_or(0),
        )
    });

    if let Some(ref stats) = stats_line {
        embed = embed.field("Engagement", stats, false);
    }

    // Source details.
    if let Some(first) = work.sources.first() {
        if !first.source.is_empty() {
            embed = embed
                .field("Source", format!("[{}]({})", first.source.to_uppercase(), first.url), true)
                .field("Words", crate::util::format_words(first.words), true)
                .field("Chapters", format!("{} ({})", first.chapters, first.status), true);
        }
    }

    let _ = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed)
                .content(format!("\n{}", text))
                .ephemeral(true),
        )
        .await;

    Ok(())
}

/// Build a work-detail embed — reusable by auto-URL detection and other frontends.
pub fn work_embed(
    work: &crate::model::Work,
    stats: Option<&crate::model::WorkStatsResponse>,
) -> CreateEmbed {
    let text = format_work_detail(work, stats);
    CreateEmbed::new()
        .title(truncate(&work.canonical_title, 200))
        .description(truncate(&text, 1024))
        .color(0x9b59b6)
}

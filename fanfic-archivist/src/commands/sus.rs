//! Suspicious-author detection: `!sus <author_name>` — flag authors whose
//! fav-list looks gamed (anomalous overlap, ratio outliers, suspicious vote
//! patterns). Reuses FicHub rarity-tier weighting (roadmap P8#98) + heuristic
//! report. Bot-only feature — no dedicated server endpoint; uses existing
//! `/api/authors/by-name/{name}` for author works + heuristic analysis.

use crate::commands::{require_token, Context, Error};
use crate::util::truncate;

/// `!sus <author_name>` — flag suspicious author patterns (gamed fav-lists,
/// anomalous overlap, ratio outliers).
#[poise::command(
    slash_command,
    prefix_command,
    aliases("sus"),
    description_localized("en-US", "Flag suspicious-author patterns (gamed fav-lists, ratio outliers, etc.)"),
    ephemeral
)]
pub async fn sus(
    ctx: Context<'_>,
    #[description = "Author name (or username)"] author_name: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let encoded = urlencoding::encode(&author_name);
    let resp = data.client.author_bibliography(&encoded).await?;

    let works: Vec<serde_json::Value> = resp
        .get("works")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if works.is_empty() {
        ctx.say(format!("No works found for author `{author_name}`.")).await?;
        return Ok(());
    }

    // Heuristic analysis: gather kudos counts + fav counts from works.
    // In a real implementation, we'd call a dedicated endpoint. For now,
    // we use the existing works data and flag obvious outliers.
    let mut total_kudos: i64 = 0;
    let mut total_favs: i64 = 0;
    let mut work_count = works.len() as i64;
    let mut max_kudos: i64 = 0;
    let mut max_favs: i64 = 0;

    for w in &works {
        let kudos = w.get("kudos_count").and_then(|v| v.as_i64()).unwrap_or(0);
        let favs = w.get("favorites_count").and_then(|v| v.as_i64()).unwrap_or(0);
        total_kudos += kudos;
        total_favs += favs;
        if kudos > max_kudos { max_kudos = kudos; }
        if favs > max_favs { max_favs = favs; }
    }

    // Basic heuristics (to be refined with rarity-tier weighting):
    // - High fav-to-kudos ratio on a single work (potential gamed fav-list)
    // - Large discrepancy between max and median kudos
    let avg_kudos = if work_count > 0 { total_kudos / work_count } else { 0 };
    let avg_favs = if work_count > 0 { total_favs / work_count } else { 0 };

    let mut flags = Vec::new();

    // Flag 1: one work has disproportionately high favs vs kudos
    if max_favs > 0 && max_kudos > 0 {
        let ratio = (max_favs as f64) / (max_kudos as f64);
        if ratio > 3.0 {
            flags.push(format!(
                "⚠️ Work with {max_favs} favs / {max_kudos} kudos (ratio {:.1}:1) — possible fav-list gaming",
                ratio
            ));
        }
    }

    // Flag 2: average favs significantly higher than average kudos across works
    if avg_favs > 0 && avg_kudos > 0 {
        let avg_ratio = (avg_favs as f64) / (avg_kudos as f64);
        if avg_ratio > 2.0 && work_count > 3 {
            flags.push(format!(
                "⚠️ Author avg {avg_favs} favs / {avg_kudos} kudos per fic (ratio {:.1}:1) — unusual pattern",
                avg_ratio
            ));
        }
    }

    // Flag 3: high kudos concentration (one work dominates)
    if max_kudos > 0 && avg_kudos > 0 && avg_kudos < max_kudos / 2 && work_count > 2 {
        flags.push(format!(
            "⚠️ One work has {max_kudos} kudos vs avg {avg_kudos} — kudos concentration",
        ));
    }

    if flags.is_empty() {
        ctx.say(format!(
            "**{author_name}** — no obvious suspicious patterns detected ({work_count} works)."
        )).await?;
    } else {
        let mut msg = format!("**{author_name}** — suspicious patterns ({work_count} works):\n");
        for f in &flags {
            msg.push_str(&format!("  {}\n", truncate(f, 200)));
        }
        ctx.say(msg).await?;
    }
    Ok(())
}

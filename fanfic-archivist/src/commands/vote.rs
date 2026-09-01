//! Roadmap arena voting: `/vote` — submit a MaxDiff vote.
//!
//! Wraps `POST /api/roadmap/vote`. First fetch the arena from
//! `GET /api/roadmap/arena` to get the 4 cluster ids, then submit best+worst.

use poise::serenity_prelude as serenity;

use crate::commands::{require_token, Context, Error};

/// Arena vote request body for `/api/roadmap/vote`.
#[derive(Debug, Clone, serde::Serialize)]
struct ArenaVote {
    cluster_ids: Vec<i32>,
    best_cluster_id: i32,
    worst_cluster_id: i32,
}

/// `/vote <best> <worst> [<neutral1> <neutral2>]` — submit a roadmap arena vote.
#[poise::command(
    slash_command,
    prefix_command,
    aliases("vote"),
    description_localized("en-US", "Submit a roadmap arena vote (4 cluster ids, best + worst)"),
    ephemeral
)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "Best cluster id (the one you liked most)"] best: i32,
    #[description = "Worst cluster id (the one you liked least)"] worst: i32,
    #[description = "Neutral cluster id 1"] neutral1: Option<i32>,
    #[description = "Neutral cluster id 2"] neutral2: Option<i32>,
) -> Result<(), Error> {
    let data = ctx.data();
    let token = require_token(&data.tokens, ctx.author().id.get()).await?;
    data.tokens.touch(&ctx.author().id.get().to_string()).await?;

    let mut ids = vec![best, worst];
    for n in [neutral1, neutral2].into_iter().flatten() {
        if !ids.contains(&n) {
            ids.push(n);
        }
    }
    if ids.len() != 4 {
        ctx.say("Need exactly 4 cluster ids: best, worst, and 2 neutrals. Run `/roadmap` first to get the current arena.").await?;
        return Ok(());
    }

    let body = ArenaVote {
        cluster_ids: ids.clone(),
        best_cluster_id: best,
        worst_cluster_id: worst,
    };

    match data.client.vote_roadmap(&token, &[body.best_cluster_id, body.worst_cluster_id, 0, 0], body.best_cluster_id, body.worst_cluster_id).await {
        Ok(resp) => {
            let applied: Vec<serde_json::Value> = resp
                .get("applied")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if applied.is_empty() {
                ctx.say("Vote not applied (maybe you already voted on this set).").await?;
            } else {
                let mut msg = "✅ Vote applied:\n".to_string();
                for a in &applied {
                    let id = a.get("cluster_id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let elo = a.get("new_elo").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    msg.push_str(&format!("  cluster #{id}: new elo {elo:.0}\n"));
                }
                ctx.say(msg).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Could not submit vote: {e}")).await?;
        }
    }
    Ok(())
}

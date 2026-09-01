//! Backfill tag scores so `main_char_attr` works on pre-scoring content.
//!
//! Usage: `cargo run --bin backfill-scores` (with `DATABASE_URL` set, e.g.
//! `.env` loaded). Idempotent — re-running after a successful run is a no-op.
//!
//! What it does per fic:
//! - Character tags (type 2) that are ALL score 0: first-listed (by
//!   created_at, then tag_id) → 10, the rest → 1.
//! - Relationship tags (type 3) that are ALL score 0: first-listed → 5,
//!   the rest → 1 (mirrors scrape-time `relationship_primary`).
//! - Freeform/fandom/warning/category (4/1/5/6): left at 0 — that IS their
//!   correct score; `main_char_attr` checks freeform *presence*, not score.
//! - Fics whose character/ship tags already contain a nonzero score are
//!   skipped (backfill ran, or users voted) — never overwritten.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("error: DATABASE_URL must be set (run with .env loaded)");
            return ExitCode::FAILURE;
        }
    };

    let pool = match sqlx::PgPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("error: could not connect to database: {err}");
            return ExitCode::FAILURE;
        }
    };

    match fichub::tags::backfill::backfill_scores(&pool).await {
        Ok(summaries) => {
            if summaries.is_empty() {
                println!("No fics needed backfilling (all character/ship scores already set).");
                return ExitCode::SUCCESS;
            }
            for s in &summaries {
                let mut parts = vec![format!("url_id={}", s.url_id)];
                if let Some(mc) = &s.main_char {
                    parts.push(format!("main_char={mc}"));
                }
                if !s.other_chars.is_empty() {
                    parts.push(format!("other_chars=[{}]", s.other_chars.join(", ")));
                }
                if let Some(ps) = &s.primary_ship {
                    parts.push(format!("primary_ship={ps}"));
                }
                if !s.other_ships.is_empty() {
                    parts.push(format!("other_ships=[{}]", s.other_ships.join(", ")));
                }
                println!("{}", parts.join(" | "));
            }
            println!("Backfilled {} fic(s).", summaries.len());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: backfill failed: {err}");
            ExitCode::FAILURE
        }
    }
}

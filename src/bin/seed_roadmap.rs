//! Seed the roadmap consensus with the full program scope.
//!
//! Usage: `cargo run --bin seed-roadmap` (with `DATABASE_URL` set, e.g. `.env`
//! loaded). Connects to Postgres, embeds each feature text via Ollama
//! (nomic-embed-text, 768-d — the same client the suggest endpoint uses) and
//! upserts `feature_clusters` rows keyed by `representative_text`, so running
//! it against the 134 pre-existing clusters is a no-op for anything already
//! present.
//!
//! Statuses: already-shipped features get 'shipped' (shown as done in the
//! consensus UI), proposed-but-missing features get 'open' (votable in the
//! arena). Rejected/deferred ideas carry their decision in the text and stay
//! 'open' for votes unless explicitly marked in the seed list.
//!
//! Idempotent: run it as often as you like.

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

    let ollama_base = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let ollama_model = std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".into());
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("build reqwest client");
    let ollama = fichub::services::ollama::OllamaClient::new(ollama_base, ollama_model, http);

    println!("Embedding + upserting {} features…", fichub::roadmap_seed::seed_list().len());

    let report = match fichub::roadmap_seed::seed_roadmap_features(&pool, &ollama).await {
        Ok(report) => report,
        Err(err) => {
            eprintln!("error: seed failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    let (mut inserted, mut updated, mut unchanged) = (0usize, 0usize, 0usize);
    for (text, action, status) in &report {
        println!("  [{action:>9}] ({status:8}) {text}");
        match action.as_str() {
            "inserted" => inserted += 1,
            "updated" => updated += 1,
            _ => unchanged += 1,
        }
    }
    println!(
        "Done: {inserted} inserted, {updated} updated, {unchanged} unchanged (of {}).",
        report.len()
    );
    ExitCode::SUCCESS
}

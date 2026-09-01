//! FicHub CLI — search, download, rate, kudos from the terminal.

use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "fichub", about = "CLI for the FicHub fanfiction archive")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,

    /// FicHub API base URL. Default: http://localhost:8000
    #[arg(long, global = true)]
    base_url: Option<String>,

    /// FicHub auth token (JWT). Enables kudos toggle.
    #[arg(long, global = true)]
    token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show detailed info about a fic by work ID.
    Work {
        /// FicHub work ID (numeric)
        id: i64,
    },
    /// Toggle kudos on a fic, or show count.
    Kudos {
        /// FicHub work ID (numeric)
        id: i64,
        /// "count" to show kudos without toggling (no auth needed)
        action: Option<String>,
    },
    /// Search the archive.
    Search {
        /// Search query (or natural-language via LLM if --ask)
        query: String,
        /// Use LLM natural-language search
        #[arg(long)]
        ask: bool,
        /// Results per page
        #[arg(short, long, default_value_t = 10)]
        per_page: usize,
    },
    /// Download a fic in EPUB/MOBI/PDF/AZW3.
    Download {
        /// Fic URL or work ID
        url: String,
        /// Output format: epub | mobi | pdf | azw3
        #[arg(short, long, default_value = "epub")]
        format: String,
        /// Write to this path instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show your linked FicHub account (requires token).
    Whoami,
    /// Show the recommendation consensus leaderboard.
    Roadmap,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let base_url = cli.base_url.unwrap_or_else(|| {
        std::env::var("FANFIC_ARCHIVIST_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string())
    });

    let config = fanfic_archivist::config::BotConfig {
        base_url,
        ..Default::default()
    };
    let client = Arc::new(fanfic_archivist::api::FichubClient::new(Arc::new(config))?);

    match cli.cmd {
        Commands::Work { id } => {
            let resp = client.work_detail(id).await?;
            let Some(work) = resp.work else {
                println!("No work found with ID {}", id);
                return Ok(());
            };
            let stats = client.work_stats(id).await.ok();
            let text = fanfic_archivist::presenter::format_work_detail(&work, stats.as_ref());
            println!("{}", text);
        }

        Commands::Kudos { id, action } => {
            if action.as_deref() == Some("count") {
                let k = client.kudos(id).await?;
                let total = k.kudos_count + k.guest_count;
                println!("No. {} total kudos ({} guest)", total, k.guest_count);
            } else {
                let token = cli.token.ok_or("Kudos toggle requires --token")?;
                let resp = client.give_kudos(&token, id).await?;
                println!("Gave kudos. Total: {}", resp.total_kudos);
            }
        }

        Commands::Search { query, ask, per_page } => {
            if ask {
                let resp = client.ask(&query).await?;
                print_search_results(&resp.search.results);
            } else {
                let params = fanfic_archivist::api::SearchParams {
                    q: query.clone(),
                    per_page: Some(per_page),
                    ..Default::default()
                };
                let resp = client.search(&params).await?;
                print_search_results(&resp.results);
            }
        }

        Commands::Download { url, format, output } => {
            let fic_url = if url.parse::<i64>().is_ok() {
                format!("https://fichub.example.com/fic/{}", url)
            } else {
                url
            };
            let export = client.fetch_export(&fic_url).await?;
            let urls = export.urls.clone().unwrap_or_else(|| export.flat_urls());

            if let Some((fmt, u)) = urls.available().iter().find(|(f, _)| f == &format) {
                if let Some(path) = output {
                    println!("Download URL: {}", u);
                    println!("(Save to {} manually)", path);
                } else {
                    println!("{}: {}", fmt, u);
                }
            } else if ["mobi", "pdf", "azw3"].contains(&format.as_str()) {
                let cv = client.lazy_convert(&fic_url, &format).await?;
                if let Some(u) = &cv.url {
                    println!("{}: {}", format, u);
                } else {
                    println!("No {} available: {:?}", format, cv.msg);
                }
            } else {
                println!("Unknown format: {}. Try epub | mobi | pdf | azw3", format);
            }
        }

        Commands::Whoami => {
            let token = cli.token.ok_or("whoami requires --token")?;
            let me = client.me(&token).await?;
            if let Some(user) = me.user {
                println!("{} — user #{}", user.username, user.id);
            } else {
                println!("Token invalid or expired.");
            }
        }

        Commands::Roadmap => {
            let consensus = client.consensus().await?;
            for row in &consensus.leaderboard {
                println!("{}. {} Elo {}", row.id, row.text, row.elo_rating);
            }
        }
    }

    Ok(())
}

fn print_search_results(results: &[fanfic_archivist::model::SearchResult]) {
    for (i, r) in results.iter().enumerate() {
        println!(
            "{}. {} by {}\n   {} words • {} chapters • kudos: {} • comments: {}\n   {}",
            i + 1,
            r.title,
            r.author,
            r.words,
            r.chapters,
            r.kudos_count,
            r.comment_count,
            r.source,
        );
    }
}

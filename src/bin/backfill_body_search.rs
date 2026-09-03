//! Backfill the `body_text_search` tsvector column from the on-disk body
//! cache. Run manually (NOT part of CI) after deploying migration 040:
//!
//!   set -a; . ./.env; set +a   # DATABASE_URL + BODY_CACHE_DIR
//!   cargo run --bin backfill_body_search
//!
//! Walks BODY_CACHE_DIR recursively, collects every `<url_id>.v{N}.json`
//! blob, and re-computes body_text_search from the extracted chapters.
//! Idempotent (the UPDATE overwrites) and graceful (files that fail to
//! parse are skipped and counted, never fatal).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("error: DATABASE_URL must be set (run with .env loaded)");
            return ExitCode::FAILURE;
        }
    };
    let body_dir = match std::env::var("BODY_CACHE_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            eprintln!("error: BODY_CACHE_DIR must be set (run with .env loaded)");
            return ExitCode::FAILURE;
        }
    };
    if !body_dir.exists() {
        eprintln!("error: BODY_CACHE_DIR {} does not exist", body_dir.display());
        return ExitCode::FAILURE;
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to start tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };

    let ok = rt.block_on(async {
        let pool = match sqlx::PgPool::connect(&database_url).await {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("error: could not connect to database: {err}");
                return false;
            }
        };

        let config = fichub::config::Config::from_env();

        // Collect url_ids: every *.json body blob (extracted chapters).
        // The RAW html artifacts are `<url_id>.v{N}.html` — skipped.
        let mut url_ids: Vec<String> = Vec::new();
        collect_url_ids(&body_dir, &mut url_ids);
        url_ids.sort();
        url_ids.dedup();
        println!("Found {} body blob(s) under {}", url_ids.len(), body_dir.display());

        let mut indexed = 0usize;
        let mut skipped = 0usize;
        let mut missing = 0usize;
        for (i, url_id) in url_ids.iter().enumerate() {
            if fichub::body_cache::body_plain_text(&config, url_id).is_none() {
                skipped += 1;
                continue;
            }
            fichub::body_cache::index_body_text(&pool, &config, url_id).await;
            indexed += 1;
            if (i + 1) % 500 == 0 {
                println!("  progress: {}/{} indexed", i + 1, url_ids.len());
            }
        }

        // Fics whose blob exists in the cache dir but that have no
        // fic_info row are harmless (the UPDATE is a no-op) — count them
        // via a second pass against the DB to report accurately.
        if indexed > 0 {
            let rows: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM fic_info WHERE body_text_search IS NOT NULL",
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
            missing = indexed.saturating_sub(rows.len());
        }

        println!(
            "Backfill complete: {indexed} indexed, {skipped} skipped (no parseable body), {missing} without a fic_info row"
        );
        true
    });

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Recursively collect url_ids from all `<url_id>.v{N}.json` blobs under
/// the body cache dir (the two-level hex sharding nests them 2 deep).
fn collect_url_ids(dir: &std::path::Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_url_ids(&path, out);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // <url_id>.v{N}.json — url_id itself may contain dots/underscores
            if name.ends_with(".json") {
                if let Some(idx) = name.rfind(".v") {
                    let url_id = &name[..idx];
                    out.push(url_id.to_string());
                }
            }
        }
    }
}

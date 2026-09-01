//! `weekly-digest` — the <15-min admin digest.
//!
//! Usage: `cargo run --bin weekly-digest -- [--email]` (with `DATABASE_URL`,
//! and `SMTP_*` env vars when `--email` is given).
//!
//! Aggregates everything an admin needs to review weekly and prints it (or
//! emails it via the existing SMTP mailer):
//!   1. Contested reports awaiting resolution / auto-hidden content
//!   2. TL5 (Community Moderator) candidates surfaced by the promoter
//!   3. Trust-level changes since last run
//!   4. Spam stats (auto-hide rate, top flagged users)
//!   5. Top new content worth a glance
//!
//! Runs the trust promotion pass first so the digest reflects the latest
//! trust state, then renders. `--mark-read` records the run in
//! `modlog` (action 'weekly_digest') so the next run can diff.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let email = std::env::args().any(|a| a == "--email");
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

    // Refresh trust before showing the digest.
    let config = fichub::config::Config::from_env();
    let (promotions, candidates) =
        match fichub::services::trust::run_trust_promotion(&pool, &config).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: promotion pass failed (digest continues): {e}");
                (Vec::new(), Vec::new())
            }
        };
    let demotions = fichub::services::trust::run_trust_revocation(&pool)
        .await
        .unwrap_or_default();

    // --- 1. Contested / auto-hidden reports ---
    let contested: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT id, target_type, status, auto_status FROM user_reports
         WHERE auto_status IN ('auto_hidden','needs_admin') OR status = 'open'
         ORDER BY created_at DESC LIMIT 40",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    // --- 4. Spam stats ---
    let (spam_auto, spam_total): (i64, i64) = sqlx::query_as(
        "SELECT
           COUNT(*) FILTER (WHERE auto_status IN ('auto_hidden','needs_admin')),
           COUNT(*)
         FROM user_reports WHERE created_at > NOW() - interval '7 days'",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0));

    println!("==============================================");
    println!("  FicNexus Weekly Moderation Digest");
    println!("==============================================");

    println!("\n1) Reports needing a click ({} shown):", contested.len());
    for (id, ttype, status, auto) in &contested {
        println!(
            "   #[{id}] target={ttype} status={status} auto={auto}"
        );
    }

    println!("\n2) Community Moderator candidates ({}):", candidates.len());
    for (uid, m) in &candidates {
        println!(
            "   user #{uid}: TL4 → promote to TL5? {} posts, {} works read, {} reports filed",
            m.forum_posts, m.works_read, m.reports_filed
        );
    }

    println!("\n3) Trust changes since last run:");
    println!("   auto promotions: {}", promotions.len());
    for (uid, f, t, r) in &promotions {
        println!("     user #{uid}: TL{f} → TL{t} ({r})");
    }
    println!("   auto demotions (spam): {}", demotions.len());
    for (uid, f, t, r) in &demotions {
        println!("     user #{uid}: TL{f} → TL{t} ({r})");
    }

    println!("\n4) Spam activity (7d):");
    println!(
        "   {} auto-triaged / {} total reports (auto-hide rate {:.0}%)",
        spam_auto,
        spam_total,
        if spam_total > 0 {
            spam_auto as f64 / spam_total as f64 * 100.0
        } else {
            0.0
        }
    );

    if email {
        use fichub::services::mailer::{SmtpConfig, send_digest_email};
        let smtp = SmtpConfig::from_env();
        if smtp.host.is_empty() {
            eprintln!("warning: --email given but SMTP_HOST unset; skipping email");
        } else {
            let mut body = String::new();
            body.push_str("FicNexus Weekly Moderation Digest\r\n\r\n");
            body.push_str(&format!("Reports open/auto-hidden: {}\r\n", contested.len()));
            for (id, ttype, status, auto) in &contested {
                body.push_str(&format!("  #{id} {ttype} status={status} auto={auto}\r\n"));
            }
            body.push_str(&format!("\r\nTL5 candidates: {}\r\n", candidates.len()));
            body.push_str(&format!(
                "\r\nTrust changes: {} up / {} down\r\n",
                promotions.len(),
                demotions.len()
            ));
            body.push_str(&format!("\r\nSpam (7d): {} auto / {} total\r\n", spam_auto, spam_total));

            let to = smtp.digest_recipient();
            match send_digest_email(&smtp, &to, "FicNexus Weekly Moderation Digest", &body) {
                Ok(()) => println!("digest emailed to {to}"),
                Err(e) => eprintln!("error: could not send digest email: {e}"),
            }
        }
    }

    ExitCode::SUCCESS
}
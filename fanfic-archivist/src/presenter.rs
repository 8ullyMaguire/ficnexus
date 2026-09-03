//! Platform-agnostic presentation helpers for FicHub data.
//!
//! These functions format Work, Kudos, and other shared models into
//! plain text (Markdown or plain) that can be reused by any frontend:
//! Discord embeds, Telegram messages, CLI output, Matrix cards, etc.
//!
//! No platform-specific types (poise, serenity, tui) here — only
//! `&str` / `String` returns that each frontend wraps in its own
//! embed/message/widget.

use crate::model::{Work, WorkStatsResponse};

/// Format a work's display block — the canonical text representation
/// shared by `/work`, `fichub work <id>`, and any future Telegram/Matrix port.
///
/// Returns Markdown text. Each frontend wraps this in its own embed/card.
pub fn format_work_detail(work: &Work, stats: Option<&WorkStatsResponse>) -> String {
    let mut lines = Vec::new();

    // Title + author
    lines.push(format!(
        "**{}** _by {}_",
        work.canonical_title, work.canonical_author,
    ));

    // Source badge (first source domain)
    if let Some(first) = work.sources.first() {
        if !first.source.is_empty() {
            lines.push(format!("[{}]({})", first.source.to_uppercase(), first.url));
        }
    }

    // Description (truncated)
    if !work.description.is_empty() {
        let desc = if work.description.len() > 500 {
            format!("{}...", &work.description[..500])
        } else {
            work.description.clone()
        };
        lines.push(format!("\n{}", desc));
    }

    // Stats row
    let kudos = stats.map(|s| s.kudos_count).unwrap_or(0);
    let guests = stats.map(|s| s.guest_count).unwrap_or(0);
    let bookmarks = work.total_bookmarks;
    let ratings = work.total_ratings;
    let comments = work.total_comments;
    let views = stats.and_then(|s| s.total_views);

    let mut stat_parts = vec![
        format!("❤️ {} kudos", kudos + guests),
        format!("📌 {} bookmarks", bookmarks),
        format!("⭐ {} ratings", ratings),
        format!("💬 {} comments", comments),
    ];
    if let Some(v) = views {
        stat_parts.push(format!("👁️ {} views", v));
    }
    lines.push(format!("\n{}", stat_parts.join(" · ")));

    // Sources detail
    if !work.sources.is_empty() {
        let src_lines: Vec<String> = work
            .sources
            .iter()
            .map(|s| format!("• [{}]({}) — {} chapters, {} words", s.title, s.url, s.chapters, s.words))
            .collect();
        lines.push(format!("\n**Sources:**\n{}", src_lines.join("\n")));
    }

    lines.join("\n")
}

/// Format a kudos toggle response (after give/remove).
/// Returns a one-line string any frontend can send.
pub fn format_kudos_response(
    work: &Work,
    total_kudos: i64,
    my_kudos: bool,
) -> String {
    if my_kudos {
        format!(
            "❤️ You gave kudos to **{}** by {}.\n   Total kudos: {}",
            work.canonical_title, work.canonical_author, total_kudos,
        )
    } else {
        format!(
            "💔 Removed kudos from **{}** by {}.\n   Total kudos: {}",
            work.canonical_title, work.canonical_author, total_kudos,
        )
    }
}

/// Format a simple "not linked" message — used by every frontend
/// when auth is required but the user hasn't linked their account.
pub fn format_not_linked() -> String {
    "You're not linked — link your FicHub account first to use this command."
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Work, WorkSource, WorkStatsResponse};

    fn sample_work() -> Work {
        Work {
            id: 42,
            canonical_title: "The Marriage Contract".to_string(),
            canonical_author: "QuickThorne".to_string(),
            description: "A tale of marriage".to_string(),
            default_source_id: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-01T00:00:00Z".to_string(),
            sources: vec![WorkSource {
                id: "abc".to_string(),
                source: "ao3".to_string(),
                title: "The Marriage Contract".to_string(),
                author: "QuickThorne".to_string(),
                words: 84234,
                chapters: 12,
                status: "completed".to_string(),
                url: "https://ao3.org/works/12345".to_string(),
            }],
            total_bookmarks: 150,
            total_ratings: 87,
            total_comments: 32,
        }
    }

    fn sample_stats() -> WorkStatsResponse {
        WorkStatsResponse {
            err: 0,
            work_id: 42,
            kudos_count: 300,
            guest_count: 50,
            total_bookmarks: 150,
            total_ratings: 87,
            total_comments: 32,
            total_views: Some(2048),
            avg_rating: Some(4.5),
        }
    }

    #[test]
    fn format_work_includes_title_author_stats() {
        let work = sample_work();
        let stats = sample_stats();
        let text = format_work_detail(&work, Some(&stats));
        assert!(text.contains("**The Marriage Contract** _by QuickThorne_"));
        assert!(text.contains("❤️ 350 kudos"));
        assert!(text.contains("👁️ 2048 views"));
        assert!(text.contains("Sources"));
    }

    #[test]
    fn format_kudos_toggled_on() {
        let work = sample_work();
        let text = format_kudos_response(&work, 301, true);
        assert!(text.contains("You gave kudos"));
        assert!(text.contains("301"));
    }

    #[test]
    fn format_not_linked_is_string() {
        let msg = format_not_linked();
        assert!(msg.contains("linked"));
    }
}

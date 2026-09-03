//! Comment moderation triage — best-effort LLM classification of new
//! comments into an admin review queue.
//!
//! Every posted comment is classified by Ollama (llama3.1:8b via
//! `/api/generate`) into one of: `fine`, `constructive`, `non-constructive`,
//! `toxic`, `spam`. The verdict lands in the `comment_triage` table with a
//! short reason + confidence; curators review it in the admin moderation UI
//! and decide what to do. Nothing is ever auto-hidden: triage is advisory.
//!
//! The whole pipeline is deliberately best-effort. If Ollama is down, the
//! response is unparsable, or the insert fails, the comment post itself is
//! unaffected — [`classify_comment`] falls back to `Fine` with confidence
//! `0.0` and the caller logs and moves on.

use crate::services::ollama::{OllamaClient, OllamaError};

/// Triage categories. `Fine`/`Constructive` are publicly shown;
/// `NonConstructive`/`Toxic`/`Spam` land in the admin review queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriageCategory {
    Fine,
    Constructive,
    NonConstructive,
    Toxic,
    Spam,
}

/// Result of a triage pass over one comment.
#[derive(Debug, Clone, PartialEq)]
pub struct CommentTriage {
    pub category: TriageCategory,
    pub reason: String,
    pub confidence: f32,
}

impl CommentTriage {
    /// Default fallback used whenever Ollama is down or the response is
    /// unparsable — triage never fails the comment post.
    pub fn fallback() -> Self {
        Self {
            category: TriageCategory::Fine,
            reason: "triage unavailable".into(),
            confidence: 0.0,
        }
    }
}

impl TriageCategory {
    /// Parse a category name leniently (case/whitespace-insensitive).
    /// Anything unknown falls back to `Fine` so a weird model answer never
    /// creates a spurious moderation queue entry.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "constructive" => TriageCategory::Constructive,
            "non-constructive" | "nonconstructive" => TriageCategory::NonConstructive,
            "toxic" => TriageCategory::Toxic,
            "spam" => TriageCategory::Spam,
            _ => TriageCategory::Fine,
        }
    }

    /// Stable lowercase name used in the DB and the admin API.
    pub fn as_str(&self) -> &'static str {
        match self {
            TriageCategory::Fine => "fine",
            TriageCategory::Constructive => "constructive",
            TriageCategory::NonConstructive => "non-constructive",
            TriageCategory::Toxic => "toxic",
            TriageCategory::Spam => "spam",
        }
    }
}

/// Should this comment be shown publicly? `Fine`/`Constructive` yes;
/// `NonConstructive`/`Toxic`/`Spam` no (they belong in the review queue).
pub fn should_show_publicly(category: TriageCategory) -> bool {
    !matches!(
        category,
        TriageCategory::NonConstructive | TriageCategory::Toxic | TriageCategory::Spam
    )
}

/// Terse system-ish prompt for the classification call. The model is asked
/// for a machine-parseable one-liner: `category | reason | confidence`.
pub fn triage_prompt(text: &str) -> String {
    format!(
        "Classify this fanfiction comment as one of: fine, constructive, \
         non-constructive, toxic, spam. Reply with one word + | + short \
         reason + | + confidence 0-1.\n\nComment: {text}"
    )
}

/// Parse a model reply of the form `toxic | reason | 0.9`.
///
/// Defensive by design: unknown categories, missing fields, and out-of-range
/// or non-numeric confidence all degrade to `Fine`/`0.0` rather than erroring
/// — triage is best-effort and must never block a comment post.
pub fn parse_triage_response(s: &str) -> CommentTriage {
    let s = s.trim();
    if s.is_empty() {
        return CommentTriage::fallback();
    }

    // Take the first line only — models sometimes append explanations.
    let line = s.lines().next().unwrap_or(s);
    let mut parts = line.splitn(3, '|').map(|p| p.trim());

    let category = TriageCategory::parse(parts.next().unwrap_or(""));
    let reason = parts.next().unwrap_or("").to_string();
    let confidence = match parts.next() {
        Some(raw) => raw
            .trim()
            .trim_end_matches('.')
            .parse::<f32>()
            .ok()
            .filter(|c| c.is_finite() && (0.0..=1.0).contains(c))
            .unwrap_or(0.0),
        None => 0.0,
    };

    CommentTriage { category, reason, confidence }
}

/// Classify a comment body via Ollama and persist the result.
///
/// Best-effort: any failure (Ollama down, bad response, DB error) falls back
/// to [`CommentTriage::fallback`] and is logged — the caller's comment post
/// must never fail because of triage. Insert is `ON CONFLICT DO NOTHING` so
/// a retry can never duplicate a triage row.
pub async fn classify_and_store(
    db: &sqlx::PgPool,
    ollama: &OllamaClient,
    comment_id: i64,
    text: &str,
) -> CommentTriage {
    let triage = classify_comment(ollama, text).await;

    let result = sqlx::query(
        "INSERT INTO comment_triage (comment_id, category, reason, confidence)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (comment_id) DO NOTHING",
    )
    .bind(comment_id)
    .bind(triage.category.as_str())
    .bind(&triage.reason)
    .bind(triage.confidence)
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::warn!("comment triage insert failed for comment {comment_id}: {e}");
    }
    triage
}

/// Classify a comment body. Never fails: any error from the model call
/// degrades to [`CommentTriage::fallback`].
async fn classify_comment(ollama: &OllamaClient, text: &str) -> CommentTriage {
    let text = text.trim();
    if text.is_empty() {
        return CommentTriage::fallback();
    }
    match ollama.generate(&triage_prompt(text), "llama3.1:8b").await {
        Ok(reply) => parse_triage_response(&reply),
        Err(OllamaError(e)) => {
            tracing::debug!("comment triage skipped (ollama): {e}");
            CommentTriage::fallback()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_happy_paths() {
        let t = parse_triage_response("toxic | slurs directed at author | 0.92");
        assert_eq!(t.category, TriageCategory::Toxic);
        assert_eq!(t.reason, "slurs directed at author");
        assert!((t.confidence - 0.92).abs() < 1e-6);

        let t = parse_triage_response("spam | link to casino | 0.85");
        assert_eq!(t.category, TriageCategory::Spam);
        assert_eq!(t.reason, "link to casino");
        assert!((t.confidence - 0.85).abs() < 1e-6);

        let t = parse_triage_response("fine |  | 0.6");
        assert_eq!(t.category, TriageCategory::Fine);
        assert_eq!(t.reason, "");
        assert!((t.confidence - 0.6).abs() < 1e-6);
    }

    #[test]
    fn parse_is_case_and_whitespace_insensitive() {
        let t = parse_triage_response("  Toxic  |  Nasty  |  0.9 ");
        assert_eq!(t.category, TriageCategory::Toxic);
        assert_eq!(t.reason, "Nasty");
        assert!((t.confidence - 0.9).abs() < 1e-6);

        let t = parse_triage_response("Constructive | good point | 0.7");
        assert_eq!(t.category, TriageCategory::Constructive);
        assert_eq!(t.reason, "good point");
    }

    #[test]
    fn parse_handles_multi_line_replies() {
        // Models sometimes append extra sentences after the one-liner.
        let t = parse_triage_response(
            "non-constructive | just venting | 0.71\nThis comment expresses frustration without actionable feedback.",
        );
        assert_eq!(t.category, TriageCategory::NonConstructive);
        assert_eq!(t.reason, "just venting");
        assert!((t.confidence - 0.71).abs() < 1e-6);
    }

    #[test]
    fn parse_unknown_category_and_bad_confidence_fall_back_to_fine() {
        let t = parse_triage_response("maybe | unclear | 0.5");
        assert_eq!(t.category, TriageCategory::Fine);
        assert_eq!(t.reason, "unclear");
        assert!((t.confidence - 0.5).abs() < 1e-6);

        // Out-of-range confidence clamps to the 0.0 fallback.
        let t = parse_triage_response("toxic | slur | 42");
        assert_eq!(t.category, TriageCategory::Toxic);
        assert_eq!(t.confidence, 0.0);

        // Non-numeric confidence → 0.0.
        let t = parse_triage_response("spam | link | high");
        assert_eq!(t.category, TriageCategory::Spam);
        assert_eq!(t.confidence, 0.0);
    }

    #[test]
    fn parse_missing_fields_and_empty_input_fall_back() {
        let t = parse_triage_response("");
        assert_eq!(t.category, TriageCategory::Fine);
        assert_eq!(t.confidence, 0.0);

        let t = parse_triage_response("toxic");
        assert_eq!(t.category, TriageCategory::Toxic);
        assert_eq!(t.reason, "");
        assert_eq!(t.confidence, 0.0);
    }

    #[test]
    fn should_show_publicly_matches_expected_policy() {
        assert!(should_show_publicly(TriageCategory::Fine));
        assert!(should_show_publicly(TriageCategory::Constructive));
        assert!(!should_show_publicly(TriageCategory::NonConstructive));
        assert!(!should_show_publicly(TriageCategory::Toxic));
        assert!(!should_show_publicly(TriageCategory::Spam));
    }

    #[test]
    fn category_as_str_roundtrip() {
        for c in [
            TriageCategory::Fine,
            TriageCategory::Constructive,
            TriageCategory::NonConstructive,
            TriageCategory::Toxic,
            TriageCategory::Spam,
        ] {
            assert_eq!(TriageCategory::parse(c.as_str()), c);
        }
    }

    #[test]
    fn triage_prompt_embeds_text() {
        let p = triage_prompt("hello there");
        assert!(p.contains("hello there"));
        assert!(p.contains("fine, constructive, non-constructive, toxic, spam"));
    }
}

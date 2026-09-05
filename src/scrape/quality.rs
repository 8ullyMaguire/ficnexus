//! Quality filters for scraped work metadata.
//!
//! Per user feedback, some scraped works show up with `unknown` titles /
//! authors (page didn't load, 0 words, lorem ipsum placeholders, etc).
//! Rather than block the scrape at the source — which can hide legitimate
//! fresh works — we mark such candidates as `Suspicious` and let the
//! curator queue decide. The default behaviour is "auto-reject obvious
//! junk; quarantine the rest for review".
//!
//! All functions are pure and easy to test against [`FicMetadata`].

use fanfic_scrapers::FicMetadata;

/// A classification of how trustworthy a scraped work looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityVerdict {
    /// Looks legitimate: non-zero word count, real title and author.
    Accept,
    /// Clearly junk (zero words, placeholder title, lorem ipsum). Drop it.
    Reject(&'static str),
    /// Couldn't make a confident call — leave it for a curator to review.
    Suspicious(&'static str),
}

/// Strings that indicate the scraper failed to extract real content and
/// filled in a placeholder instead.
const PLACEHOLDER_TITLES: &[&str] = &["unknown", "untitled", "lorem ipsum", "placeholder", "n/a"];

/// Trim, lowercase, and collapse internal whitespace for fuzzy comparison.
fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Heuristic lorem-ipsum detection: a string made mostly of the
/// "lorem ipsum dolor sit amet" canonical filler.
fn is_lorem_ipsum(s: &str) -> bool {
    let s = s.to_lowercase();
    if s.len() < 20 {
        return false;
    }
    let hits = ["lorem", "ipsum", "dolor", "sit", "amet", "consectetur"]
        .iter()
        .filter(|w| s.contains(*w))
        .count();
    hits >= 3
}

/// Classify a scraped work. Pure function; safe to call from any context.
///
/// Decision tree:
/// 1. `words == 0` → `Suspicious` (scraper JS-render failure is the common
///    case, but a real 0-word work is also plausible, so don't auto-reject)
/// 2. Title or author is empty/placeholder → `Reject`
/// 3. Title or author is lorem-ipsum filler → `Reject`
/// 4. Otherwise → `Accept`
pub fn classify(fic: &FicMetadata) -> QualityVerdict {
    let title = normalize(&fic.title);
    let author = normalize(&fic.author);

    if title.is_empty() || author.is_empty() {
        return QualityVerdict::Reject("empty title or author");
    }
    if PLACEHOLDER_TITLES.iter().any(|p| title == *p) {
        return QualityVerdict::Reject("placeholder title");
    }
    if PLACEHOLDER_TITLES.iter().any(|p| author == *p) {
        return QualityVerdict::Reject("placeholder author");
    }
    if is_lorem_ipsum(&title) || is_lorem_ipsum(&author) {
        return QualityVerdict::Reject("lorem ipsum detected");
    }
    if fic.words <= 0 {
        return QualityVerdict::Suspicious("zero word count");
    }
    QualityVerdict::Accept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fic(title: &str, author: &str, words: i64) -> FicMetadata {
        FicMetadata {
            url_id: "x".into(),
            title: title.into(),
            author: author.into(),
            chapters: 1,
            words,
            desc: String::new(),
            published: 0,
            updated: 0,
            status: "complete".into(),
            source: "https://example.com".into(),
            source_id: 1,
            author_id: 0,
            author_url: String::new(),
            author_local_id: String::new(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }

    #[test]
    fn real_fic_accepts() {
        let m = fic("The Long Road Home", "alice42", 50_000);
        assert_eq!(classify(&m), QualityVerdict::Accept);
    }

    #[test]
    fn unknown_title_rejects() {
        let m = fic("unknown", "alice42", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn unknown_author_rejects() {
        let m = fic("My Fic", "n/a", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn empty_title_rejects() {
        let m = fic("   ", "alice42", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn empty_author_rejects() {
        let m = fic("My Fic", "  ", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn lorem_ipsum_title_rejects() {
        let m = fic("Lorem ipsum dolor sit amet consectetur", "alice42", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn zero_words_is_suspicious_not_reject() {
        let m = fic("Real Title", "real_author", 0);
        assert!(matches!(classify(&m), QualityVerdict::Suspicious(_)));
    }

    #[test]
    fn case_insensitive_placeholder_match() {
        let m = fic("UNKNOWN", "alice42", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn whitespace_normalized_for_match() {
        let m = fic("  untitled  ", "alice42", 5_000);
        assert!(matches!(classify(&m), QualityVerdict::Reject(_)));
    }

    #[test]
    fn reject_reason_is_descriptive() {
        let m = fic("unknown", "alice42", 5_000);
        if let QualityVerdict::Reject(reason) = classify(&m) {
            assert!(!reason.is_empty());
        } else {
            panic!("expected Reject");
        }
    }
}

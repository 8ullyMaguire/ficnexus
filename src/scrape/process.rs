//! Process scraped FicMetadata through the quality filter and route to
//! the appropriate destination (accept, quarantine for curator review, reject).
//!
//! Bridges `scrape::quality::classify` with the curator quorum system
//! (`curator_content` proposals). Pure decision logic — no DB or network.

use crate::scrape::quality::{QualityVerdict, classify};

/// Action to take for a scraped fic after quality classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    /// Insert normally into fic_info.
    Accept,
    /// Insert as hidden + create curator quorum proposal for review.
    Quarantine,
    /// Drop the work entirely (log only).
    Reject,
}

/// Map a scraped work's quality verdict to the appropriate processing action.
///
/// - `Accept` → `ProcessAction::Accept`
/// - `Suspicious` → `ProcessAction::Quarantine` (hidden insert + curator proposal)
/// - `Reject` → `ProcessAction::Reject` (drop with log)
pub fn process_scraped_fic(m: &fanfic_scrapers::FicMetadata) -> ProcessAction {
    match classify(m) {
        QualityVerdict::Accept => ProcessAction::Accept,
        QualityVerdict::Suspicious(_) => ProcessAction::Quarantine,
        QualityVerdict::Reject(_) => ProcessAction::Reject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrape::quality::QualityVerdict;

    fn fic(title: &str, author: &str, words: i64) -> fanfic_scrapers::FicMetadata {
        fanfic_scrapers::FicMetadata {
            url_id: "test_123".into(),
            title: title.into(),
            author: author.into(),
            chapters: 1,
            words,
            desc: "A test fic".into(),
            published: 0,
            updated: 0,
            status: "complete".into(),
            source: "https://example.com/123".into(),
            source_id: 1,
            author_id: 42,
            author_url: "https://example.com/user/42".into(),
            author_local_id: "42".into(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        }
    }

    #[test]
    fn real_fic_accepts() {
        let m = fic("The Long Road Home", "alice42", 50_000);
        assert_eq!(process_scraped_fic(&m), ProcessAction::Accept);
    }

    #[test]
    fn unknown_title_rejects() {
        let m = fic("unknown", "alice42", 5_000);
        assert_eq!(process_scraped_fic(&m), ProcessAction::Reject);
    }

    #[test]
    fn lorem_ipsum_rejects() {
        let m = fic("Lorem ipsum dolor sit amet consectetur", "alice42", 5_000);
        assert_eq!(process_scraped_fic(&m), ProcessAction::Reject);
    }

    #[test]
    fn zero_words_quarantines() {
        let m = fic("Real Title", "real_author", 0);
        assert_eq!(process_scraped_fic(&m), ProcessAction::Quarantine);
    }

    #[test]
    fn empty_title_rejects() {
        let m = fic("   ", "alice42", 5_000);
        assert_eq!(process_scraped_fic(&m), ProcessAction::Reject);
    }

    #[test]
    fn quality_verdict_reasons_are_descriptive() {
        let reject_fic = fic("unknown", "alice42", 5_000);
        match classify(&reject_fic) {
            QualityVerdict::Reject(reason) => assert!(!reason.is_empty()),
            _ => panic!("expected Reject"),
        }

        let suspicious_fic = fic("Real Title", "real_author", 0);
        match classify(&suspicious_fic) {
            QualityVerdict::Suspicious(reason) => assert!(!reason.is_empty()),
            _ => panic!("expected Suspicious"),
        }
    }

    #[test]
    fn process_action_matches_verdict() {
        // Exhaustive mapping check
        let accept_fic = fic("Good Title", "Good Author", 1000);
        let suspicious_fic = fic("Okay Title", "Okay Author", 0);
        let reject_fic = fic("unknown", "unknown", 100);

        assert_eq!(classify(&accept_fic), QualityVerdict::Accept);
        assert!(matches!(classify(&suspicious_fic), QualityVerdict::Suspicious(_)));
        assert!(matches!(classify(&reject_fic), QualityVerdict::Reject(_)));
    }
}

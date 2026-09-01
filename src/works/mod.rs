use sqlx::PgPool;
use crate::db::queries;
use crate::error::AppError;
use crate::scrape::FicMetadata;

/// Result of auto-merge check
#[derive(Debug)]
pub struct AutoMergeResult {
    /// The work ID this source was assigned to
    pub work_id: i32,
    /// Whether this was an auto-merge (true) or new work created (false)
    pub auto_merged: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Word count tolerance for auto-merge (±5%)
const WORD_COUNT_TOLERANCE: f64 = 0.05;

/// Minimum confidence for auto-merge (high confidence)
const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.9;

/// Default confidence when a title+author match exists but no existing source
/// carries a usable word count.
const DEFAULT_MERGE_CONFIDENCE: f64 = 0.8;

/// Medium confidence for a title+author match whose word count falls outside
/// the tolerance (used for proposals).
const MEDIUM_CONFIDENCE: f64 = 0.5;

/// Relative word-count difference between `actual` and `avg`, as a fraction of
/// `avg` (0.0 = identical). Returns 0.0 when `avg` is not positive.
fn word_count_diff(actual: i64, avg: f64) -> f64 {
    if avg <= 0.0 {
        0.0
    } else {
        (actual as f64 - avg).abs() / avg
    }
}

/// Whether `actual` is within ±[`WORD_COUNT_TOLERANCE`] of `avg`.
///
/// Returns `false` when `avg` is not positive (no meaningful average to
/// compare against — callers fall back to [`DEFAULT_MERGE_CONFIDENCE`]).
pub fn word_count_within_tolerance(actual: i64, avg: f64) -> bool {
    avg > 0.0 && word_count_diff(actual, avg) <= WORD_COUNT_TOLERANCE
}

/// Auto-merge confidence for a source with `actual` words vs. an existing
/// average of `avg` words.
///
/// Confidence is `1.0 - relative_diff`, clamped to at least
/// [`HIGH_CONFIDENCE_THRESHOLD`] so a close match never drops below "high".
/// Returns `None` when `avg` is not positive (no basis for comparison).
pub fn auto_merge_confidence(actual: i64, avg: f64) -> Option<f64> {
    if avg <= 0.0 {
        None
    } else {
        Some((1.0 - word_count_diff(actual, avg)).max(HIGH_CONFIDENCE_THRESHOLD))
    }
}

/// Outcome of the auto-merge heuristic for a scraped source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeDecision {
    /// High-confidence match: merge the source into the existing work.
    AutoMerge { confidence: f64 },
    /// Medium-confidence match: propose the merge for review instead of
    /// applying it automatically.
    Proposal { confidence: f64 },
    /// No title+author match found: create a new work.
    NewWork,
}

/// Decide how to handle a scraped source given whether it matched an existing
/// work by title+author and the average word count of that work's sources.
///
/// Mirrors the runtime heuristic in [`find_or_create_work`]:
/// 1. No title+author match → [`MergeDecision::NewWork`]
/// 2. Match with no usable average word count → auto-merge with
///    [`DEFAULT_MERGE_CONFIDENCE`]
/// 3. Match within ±[`WORD_COUNT_TOLERANCE`] → high-confidence auto-merge
/// 4. Match outside tolerance → [`MergeDecision::Proposal`] with
///    [`MEDIUM_CONFIDENCE`]
pub fn decide_merge(title_author_match: bool, words: i64, avg_words: f64) -> MergeDecision {
    if !title_author_match {
        return MergeDecision::NewWork;
    }
    // No word count from scraper (words=0) or no existing average — treat as
    // unknown and auto-merge with default confidence.  Zero words means the
    // scraper couldn't extract a count (e.g. RoyalRoad JS-rendered pages);
    // creating a proposal or separate work would just create noise.
    if words <= 0 || avg_words <= 0.0 {
        return MergeDecision::AutoMerge {
            confidence: DEFAULT_MERGE_CONFIDENCE,
        };
    }
    if word_count_within_tolerance(words, avg_words) {
        MergeDecision::AutoMerge {
            confidence: auto_merge_confidence(words, avg_words)
                .unwrap_or(DEFAULT_MERGE_CONFIDENCE),
        }
    } else {
        // Word count outside tolerance: medium confidence (proposal).
        MergeDecision::Proposal {
            confidence: MEDIUM_CONFIDENCE,
        }
    }
}

/// Find or create a work for a scraped source.
///
/// This runs the auto-merge heuristic:
/// 1. Exact title+author match with word count within ±5% → high confidence auto-merge
/// 2. Exact title+author match but word count outside tolerance → medium confidence (proposal)
/// 3. No match → create new work
pub async fn find_or_create_work(
    pool: &PgPool,
    meta: &FicMetadata,
) -> Result<AutoMergeResult, AppError> {
    let title = meta.title.trim();
    let author = meta.author.trim();
    let words = meta.words;

    // Step 1: Try exact title+author match
    if let Some(existing_work) = queries::find_work_by_title_author(pool, title, author).await? {
        // Get existing sources to check word count
        let sources = queries::get_work_sources(pool, existing_work.id).await?;

        // Calculate average word count across existing sources
        let avg_words = if sources.is_empty() {
            0.0
        } else {
            sources.iter().map(|s| s.words as f64).sum::<f64>() / sources.len() as f64
        };

        // Pure decision logic: word-count tolerance + confidence (no DB)
        let decision = decide_merge(true, words, avg_words);

        match decision {
            MergeDecision::AutoMerge { confidence } => {
                // High confidence: auto-merge
                if avg_words > 0.0 {
                    tracing::info!(
                        "Auto-merged source '{}' ({} words) into work '{}' (avg={:.0}, diff={:.1}%)",
                        title, words, existing_work.canonical_title, avg_words,
                        word_count_diff(words, avg_words) * 100.0
                    );
                }
                // Link source to existing work
                queries::link_source_to_work(pool, &meta.url_id, existing_work.id).await?;
                // Log the auto-merge
                queries::log_auto_merge(pool, &meta.source, existing_work.id, confidence).await?;

                return Ok(AutoMergeResult {
                    work_id: existing_work.id,
                    auto_merged: true,
                    confidence,
                });
            }
            MergeDecision::Proposal { confidence } => {
                // Word count outside tolerance: medium confidence. Do NOT
                // silently merge — that would glue a likely-different story
                // (or a very different version) onto the wrong work. Instead
                // create a NEW work for this source so the duplicate is
                // visible in search; curators can then merge it via the
                // existing /work-proposals flow.
                tracing::info!(
                    "Title+author match but word count outside tolerance: {} vs avg {:.0} (diff={:.1}%). Creating separate work instead of auto-merging.",
                    words, avg_words, word_count_diff(words, avg_words) * 100.0
                );

                let work_id = queries::create_work(
                    pool,
                    title,
                    author,
                    &meta.desc,
                    Some(&meta.url_id),
                )
                .await?;
                queries::link_source_to_work(pool, &meta.url_id, work_id).await?;

                return Ok(AutoMergeResult {
                    work_id,
                    auto_merged: false,
                    confidence,
                });
            }
            MergeDecision::NewWork => unreachable!(
                "decide_merge returned NewWork even though a title+author match exists"
            ),
        };
    }

    // Step 2: No match found — create new work
    let work_id = queries::create_work(
        pool,
        title,
        author,
        &meta.desc,
        Some(&meta.url_id),
    )
    .await?;

    // Link the source to the new work
    queries::link_source_to_work(pool, &meta.url_id, work_id).await?;

    tracing::info!(
        "Created new work '{}' by '{}' (id={})",
        title, author, work_id
    );

    Ok(AutoMergeResult {
        work_id,
        auto_merged: false,
        confidence: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- word-count tolerance ---

    #[test]
    fn tolerance_accepts_exact_match() {
        assert!(word_count_within_tolerance(100_000, 100_000.0));
    }

    #[test]
    fn tolerance_accepts_within_plus_minus_five_percent() {
        let avg = 100_000.0;
        assert!(word_count_within_tolerance(95_000, avg));
        assert!(word_count_within_tolerance(99_000, avg));
        assert!(word_count_within_tolerance(100_000, avg));
        assert!(word_count_within_tolerance(101_000, avg));
        assert!(word_count_within_tolerance(105_000, avg));
    }

    #[test]
    fn tolerance_rejects_outside_five_percent() {
        let avg = 100_000.0;
        assert!(!word_count_within_tolerance(94_999, avg));
        assert!(!word_count_within_tolerance(90_000, avg));
        assert!(!word_count_within_tolerance(105_001, avg));
        assert!(!word_count_within_tolerance(110_000, avg));
    }

    #[test]
    fn tolerance_boundary_at_exactly_five_percent_is_accepted() {
        // diff == WORD_COUNT_TOLERANCE exactly → still within tolerance
        let avg = 100_000.0;
        assert_eq!(word_count_diff(95_000, avg), 0.05);
        assert_eq!(word_count_diff(105_000, avg), 0.05);
        assert!(word_count_within_tolerance(95_000, avg));
        assert!(word_count_within_tolerance(105_000, avg));
    }

    #[test]
    fn tolerance_rejects_non_positive_average() {
        assert!(!word_count_within_tolerance(100_000, 0.0));
        assert!(!word_count_within_tolerance(100_000, -1.0));
    }

    // --- confidence calculation ---

    #[test]
    fn confidence_is_one_minus_diff() {
        // diff = 0.01 → confidence = 0.99 (no clamping needed)
        assert_eq!(auto_merge_confidence(99_000, 100_000.0), Some(0.99));
        assert_eq!(auto_merge_confidence(101_000, 100_000.0), Some(0.99));
        // diff = 0.05 → confidence = 0.95
        assert_eq!(auto_merge_confidence(95_000, 100_000.0), Some(0.95));
    }

    #[test]
    fn confidence_exact_match_is_one() {
        assert_eq!(auto_merge_confidence(100_000, 100_000.0), Some(1.0));
    }

    #[test]
    fn confidence_is_clamped_to_high_threshold() {
        // diff = 0.20 → raw 0.80 → clamped up to 0.90
        assert_eq!(
            auto_merge_confidence(80_000, 100_000.0),
            Some(HIGH_CONFIDENCE_THRESHOLD)
        );
        // diff = 0.50 → clamped
        assert_eq!(
            auto_merge_confidence(50_000, 100_000.0),
            Some(HIGH_CONFIDENCE_THRESHOLD)
        );
    }

    #[test]
    fn confidence_is_none_without_average() {
        assert_eq!(auto_merge_confidence(100_000, 0.0), None);
        assert_eq!(auto_merge_confidence(100_000, -5.0), None);
    }

    #[test]
    fn confidence_ordering_matches_diff() {
        let exact = auto_merge_confidence(100_000, 100_000.0).unwrap();
        let close = auto_merge_confidence(99_000, 100_000.0).unwrap();
        let far = auto_merge_confidence(95_000, 100_000.0).unwrap();
        assert!(exact > close);
        assert!(close > far);
        assert!(exact >= HIGH_CONFIDENCE_THRESHOLD);
    }

    // --- merge decision ---

    #[test]
    fn no_title_author_match_creates_new_work() {
        // Word counts don't matter without a title+author match.
        assert_eq!(decide_merge(false, 100_000, 100_000.0), MergeDecision::NewWork);
        assert_eq!(decide_merge(false, 100_000, 0.0), MergeDecision::NewWork);
        assert_eq!(decide_merge(false, 10, 0.0), MergeDecision::NewWork);
    }

    #[test]
    fn exact_match_auto_merges_with_high_confidence() {
        assert_eq!(
            decide_merge(true, 100_000, 100_000.0),
            MergeDecision::AutoMerge { confidence: 1.0 }
        );
    }

    #[test]
    fn match_within_tolerance_auto_merges_with_derived_confidence() {
        // diff = 0.01 → confidence = 1.0 - 0.01 = 0.99
        assert_eq!(
            decide_merge(true, 99_000, 100_000.0),
            MergeDecision::AutoMerge { confidence: 0.99 }
        );
        assert_eq!(
            decide_merge(true, 101_000, 100_000.0),
            MergeDecision::AutoMerge { confidence: 0.99 }
        );
    }

    #[test]
    fn match_at_exact_five_percent_boundary_auto_merges() {
        // Exactly ±5% → still within tolerance → high-confidence auto-merge
        assert_eq!(
            decide_merge(true, 95_000, 100_000.0),
            MergeDecision::AutoMerge { confidence: 0.95 }
        );
        assert_eq!(
            decide_merge(true, 105_000, 100_000.0),
            MergeDecision::AutoMerge { confidence: 0.95 }
        );
    }

    #[test]
    fn match_outside_tolerance_proposes_merge() {
        // Preserves current runtime semantics: outside tolerance → medium
        // confidence (proposal), which find_or_create_work still auto-merges
        // with under a TODO.
        assert_eq!(
            decide_merge(true, 90_000, 100_000.0),
            MergeDecision::Proposal { confidence: MEDIUM_CONFIDENCE }
        );
        assert_eq!(
            decide_merge(true, 110_000, 100_000.0),
            MergeDecision::Proposal { confidence: 0.5 }
        );
    }

    #[test]
    fn match_just_outside_boundary_proposes_merge() {
        // 94_999 → diff 0.05001 → just outside tolerance → proposal
        assert_eq!(
            decide_merge(true, 94_999, 100_000.0),
            MergeDecision::Proposal { confidence: MEDIUM_CONFIDENCE }
        );
    }

    #[test]
    fn match_without_word_count_average_uses_default_confidence() {
        // No existing sources with word counts → default 0.8 auto-merge
        assert_eq!(
            decide_merge(true, 100_000, 0.0),
            MergeDecision::AutoMerge { confidence: DEFAULT_MERGE_CONFIDENCE }
        );
        assert_eq!(
            decide_merge(true, 42, 0.0),
            MergeDecision::AutoMerge { confidence: 0.8 }
        );
    }

    #[test]
    fn zero_words_from_scraper_auto_merges() {
        // Scraper returned 0 words (e.g. RoyalRoad JS-rendered page).
        // Should auto-merge with default confidence, not create a proposal.
        assert_eq!(
            decide_merge(true, 0, 806_306.0),
            MergeDecision::AutoMerge { confidence: DEFAULT_MERGE_CONFIDENCE }
        );
        // Both zero — still auto-merge
        assert_eq!(
            decide_merge(true, 0, 0.0),
            MergeDecision::AutoMerge { confidence: DEFAULT_MERGE_CONFIDENCE }
        );
    }
}

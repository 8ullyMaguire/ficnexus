//! Leaderboard category definitions and scoring.
//!
//! Issue #12: multiple leaderboard categories beyond a single "all users" board.
//! Categories are defined as an enum — adding a new one is a single variant
//! + match arm, no DB schema change required.

use std::fmt;

/// Available leaderboard categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaderboardCategory {
    /// Authors ranked by works published.
    Authors,
    /// Curators ranked by moderation actions approved.
    Curators,
    /// Translators ranked by works translated.
    Translators,
    /// Developers ranked by marketplace plugins published.
    Developers,
    /// Readers ranked by kudos + comments given.
    Readers,
    /// Overall composite score across all categories.
    All,
}

impl LeaderboardCategory {
    /// All categories except `All` (used for the composite calculation).
    pub const fn variants() -> &'static [LeaderboardCategory] {
        &[
            LeaderboardCategory::Authors,
            LeaderboardCategory::Curators,
            LeaderboardCategory::Translators,
            LeaderboardCategory::Developers,
            LeaderboardCategory::Readers,
        ]
    }

    /// Parse from a query string parameter.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "authors" => Some(LeaderboardCategory::Authors),
            "curators" => Some(LeaderboardCategory::Curators),
            "translators" => Some(LeaderboardCategory::Translators),
            "developers" => Some(LeaderboardCategory::Developers),
            "readers" => Some(LeaderboardCategory::Readers),
            "all" => Some(LeaderboardCategory::All),
            _ => None,
        }
    }

    /// Display name for UI rendering.
    pub fn label(&self) -> &'static str {
        match self {
            LeaderboardCategory::Authors => "Authors",
            LeaderboardCategory::Curators => "Curators",
            LeaderboardCategory::Translators => "Translators",
            LeaderboardCategory::Developers => "Developers",
            LeaderboardCategory::Readers => "Readers",
            LeaderboardCategory::All => "Overall",
        }
    }

    /// Weight used in the `All` composite score.
    pub fn weight(&self) -> f64 {
        match self {
            LeaderboardCategory::Authors => 1.0,
            LeaderboardCategory::Curators => 5.0,
            LeaderboardCategory::Translators => 3.0,
            LeaderboardCategory::Developers => 10.0,
            LeaderboardCategory::Readers => 0.1,
            // `All` is itself the composite, weight N/A
            LeaderboardCategory::All => 0.0,
        }
    }

    /// SQL fragment that computes the raw score for this category.
    /// Returns `(user_id, score)` rows. Categories with no backing data
    /// fall back to 0 for all users.
    pub fn score_sql(&self) -> &'static str {
        match self {
            LeaderboardCategory::Authors => {
                "SELECT w.author_id as user_id, COUNT(*)::float8 as score
                 FROM works w
                 GROUP BY w.author_id"
            }
            LeaderboardCategory::Curators => {
                "SELECT p.proposed_by as user_id, COUNT(*)::float8 as score
                 FROM curator_fix_proposals p
                 WHERE p.status = 'approved'
                 GROUP BY p.proposed_by"
            }
            LeaderboardCategory::Translators => {
                "SELECT t.translator_id as user_id, COUNT(*)::float8 as score
                 FROM translations t
                 GROUP BY t.translator_id"
            }
            LeaderboardCategory::Developers => {
                "SELECT pl.developer_id as user_id, COUNT(*)::float8 as score
                 FROM plugins pl
                 GROUP BY pl.developer_id"
            }
            LeaderboardCategory::Readers => {
                "SELECT u.id as user_id, u.reputation::float8 as score
                 FROM users u"
            }
            LeaderboardCategory::All => {
                // Composite is computed in Rust from the other categories
                ""
            }
        }
    }
}

impl fmt::Display for LeaderboardCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A single leaderboard entry.
#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub user_id: i32,
    pub username: String,
    pub score: f64,
    pub rank: usize,
}

/// Compute the rank of a given user within a sorted leaderboard.
/// Returns None if the user is not present.
pub fn compute_user_rank(entries: &[LeaderboardEntry], user_id: i32) -> Option<usize> {
    entries
        .iter()
        .position(|e| e.user_id == user_id)
        .map(|pos| pos + 1)
}

/// Compute the composite "All" score from per-category scores.
/// Input: map of category → (user_id, score).
/// Output: sorted leaderboard entries.
pub fn compute_composite(
    category_scores: &std::collections::HashMap<LeaderboardCategory, Vec<(i32, f64)>>,
) -> Vec<(i32, f64)> {
    let mut composite: std::collections::HashMap<i32, f64> = std::collections::HashMap::new();

    for category in LeaderboardCategory::variants() {
        let weight = category.weight();
        if weight <= 0.0 {
            continue;
        }
        if let Some(scores) = category_scores.get(category) {
            for (user_id, score) in scores {
                *composite.entry(*user_id).or_insert(0.0) += score * weight;
            }
        }
    }

    let mut result: Vec<(i32, f64)> = composite.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_categories() {
        assert_eq!(
            LeaderboardCategory::parse("authors"),
            Some(LeaderboardCategory::Authors)
        );
        assert_eq!(
            LeaderboardCategory::parse("curators"),
            Some(LeaderboardCategory::Curators)
        );
        assert_eq!(
            LeaderboardCategory::parse("translators"),
            Some(LeaderboardCategory::Translators)
        );
        assert_eq!(
            LeaderboardCategory::parse("developers"),
            Some(LeaderboardCategory::Developers)
        );
        assert_eq!(
            LeaderboardCategory::parse("readers"),
            Some(LeaderboardCategory::Readers)
        );
        assert_eq!(
            LeaderboardCategory::parse("all"),
            Some(LeaderboardCategory::All)
        );
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(
            LeaderboardCategory::parse("AUTHORS"),
            Some(LeaderboardCategory::Authors)
        );
        assert_eq!(
            LeaderboardCategory::parse("Readers"),
            Some(LeaderboardCategory::Readers)
        );
    }

    #[test]
    fn parse_rejects_unknown() {
        assert_eq!(LeaderboardCategory::parse("promoters"), None);
        assert_eq!(LeaderboardCategory::parse(""), None);
    }

    #[test]
    fn labels_are_human_readable() {
        assert_eq!(LeaderboardCategory::Authors.label(), "Authors");
        assert_eq!(LeaderboardCategory::All.label(), "Overall");
    }

    #[test]
    fn weights_are_positive_for_non_all() {
        for cat in LeaderboardCategory::variants() {
            assert!(cat.weight() > 0.0, "{:?} should have positive weight", cat);
        }
        assert_eq!(LeaderboardCategory::All.weight(), 0.0);
    }

    #[test]
    fn score_sql_non_empty_for_all_categories() {
        for cat in LeaderboardCategory::variants() {
            assert!(
                !cat.score_sql().is_empty(),
                "{:?} should return non-empty SQL",
                cat
            );
        }
    }

    #[test]
    fn score_sql_all_is_empty() {
        // All is computed in Rust, not SQL
        assert_eq!(LeaderboardCategory::All.score_sql(), "");
    }

    #[test]
    fn compute_user_rank_finds_position() {
        let entries = vec![
            LeaderboardEntry {
                user_id: 1,
                username: "alice".into(),
                score: 100.0,
                rank: 1,
            },
            LeaderboardEntry {
                user_id: 2,
                username: "bob".into(),
                score: 80.0,
                rank: 2,
            },
            LeaderboardEntry {
                user_id: 3,
                username: "charlie".into(),
                score: 60.0,
                rank: 3,
            },
        ];
        assert_eq!(compute_user_rank(&entries, 1), Some(1));
        assert_eq!(compute_user_rank(&entries, 2), Some(2));
        assert_eq!(compute_user_rank(&entries, 99), None);
    }

    #[test]
    fn compute_composite_weights_correctly() {
        use std::collections::HashMap;

        let mut scores = HashMap::new();
        // User 1: 10 authors (weight 1.0) + 2 curators (weight 5.0) = 20.0
        scores.insert(LeaderboardCategory::Authors, vec![(1, 10.0), (2, 5.0)]);
        scores.insert(LeaderboardCategory::Curators, vec![(1, 2.0), (3, 8.0)]);

        let result = compute_composite(&scores);
        // User 1: 10*1 + 2*5 = 20
        // User 3: 0*1 + 8*5 = 40
        // User 2: 5*1 + 0*5 = 5
        assert_eq!(result[0].0, 3); // 40
        assert_eq!(result[0].1, 40.0);
        assert_eq!(result[1].0, 1); // 20
        assert_eq!(result[1].1, 20.0);
        assert_eq!(result[2].0, 2); // 5
        assert_eq!(result[2].1, 5.0);
    }

    #[test]
    fn compute_composite_empty_input() {
        let scores = std::collections::HashMap::new();
        let result = compute_composite(&scores);
        assert!(result.is_empty());
    }

    #[test]
    fn display_trait_works() {
        assert_eq!(format!("{}", LeaderboardCategory::Authors), "Authors");
        assert_eq!(format!("{}", LeaderboardCategory::All), "Overall");
    }
}

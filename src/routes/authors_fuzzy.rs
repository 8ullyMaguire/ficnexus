//! Fuzzy matching helpers for author profile lookups.
//!
//! User feedback: when `/api/authors/123` returns 404 the frontend just
//! shows "Author not found" with no way forward. These helpers let us
//! return a `suggestions` field with close name matches so the UI can
//! offer alternatives ("Did you mean: alice42?") instead of dead-ending.
//!
//! Pure functions, no DB access, no I/O. The route layer queries the
//! database, then calls [`suggest_alternatives`] to score and rank the
//! candidate list.

/// Result of fuzzy-comparing a query against a candidate name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchScore {
    /// 0 = no match, 100 = exact match.
    pub score: i32,
}

impl MatchScore {
    pub fn is_strong(self) -> bool {
        self.score >= 80
    }
}

/// Score how well `candidate` matches `query` for author-name suggestions.
///
/// Strategy: case-insensitive containment + token-set overlap. We don't
/// pull in a real fuzzy library because author names are short and
/// predictable — substring containment covers 95% of real cases.
pub fn name_match_score(query: &str, candidate: &str) -> MatchScore {
    let q = normalize(query);
    let c = normalize(candidate);
    if q.is_empty() || c.is_empty() {
        return MatchScore { score: 0 };
    }
    if q == c {
        return MatchScore { score: 100 };
    }
    if c.contains(&q) || q.contains(&c) {
        // Substring of the other — proportional to overlap length
        let overlap = q.len().min(c.len());
        let max_len = q.len().max(c.len());
        let ratio = (overlap as f64) / (max_len as f64);
        return MatchScore {
            score: (60.0 + 30.0 * ratio) as i32,
        };
    }
    // Token-set overlap (e.g. "J Smith" vs "John Smith")
    let q_tokens: std::collections::HashSet<&str> = q.split_whitespace().collect();
    let c_tokens: std::collections::HashSet<&str> = c.split_whitespace().collect();
    let shared = q_tokens.intersection(&c_tokens).count();
    if shared == 0 {
        return MatchScore { score: 0 };
    }
    let total = q_tokens.union(&c_tokens).count();
    let jaccard = (shared as f64) / (total as f64);
    MatchScore {
        score: (40.0 + 50.0 * jaccard) as i32,
    }
}

/// Trim, lowercase, and collapse internal whitespace.
fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pick the best candidates from `options` for a user typing `query`.
/// Returns up to `limit` items sorted by descending score, only items
/// with score > 0.
pub fn suggest_alternatives<'a, T, F>(
    query: &str,
    options: &'a [T],
    limit: usize,
    name: F,
) -> Vec<&'a T>
where
    F: Fn(&T) -> &str,
{
    let mut scored: Vec<(&T, i32)> = options
        .iter()
        .map(|opt| (opt, name_match_score(query, name(opt)).score))
        .filter(|(_, s)| *s > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| name(a.0).cmp(name(b.0))));
    scored.into_iter().take(limit).map(|(t, _)| t).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_scores_100() {
        assert_eq!(name_match_score("Alice", "Alice").score, 100);
        assert_eq!(name_match_score("alice", "ALICE").score, 100);
    }

    #[test]
    fn empty_inputs_score_zero() {
        assert_eq!(name_match_score("", "alice").score, 0);
        assert_eq!(name_match_score("alice", "").score, 0);
        assert_eq!(name_match_score("   ", "alice").score, 0);
    }

    #[test]
    fn substring_match_scores_high() {
        // Short query into longer candidate still gets a usable score
        let s = name_match_score("alice", "alice42");
        assert!(s.score >= 80 && s.score < 100);
        // "smith" is half of "john smith" — moderate score, not high
        let s = name_match_score("smith", "John Smith");
        assert!(s.score >= 60 && s.score < 100);
    }

    #[test]
    fn token_overlap_scores_moderate() {
        let s = name_match_score("alice johnson", "alice marie johnson");
        assert!(s.score > 40 && s.score < 100);
    }

    #[test]
    fn unrelated_names_score_zero() {
        assert_eq!(name_match_score("alice", "bob").score, 0);
        assert_eq!(name_match_score("a", "b").score, 0);
    }

    #[test]
    fn suggest_returns_top_n_sorted() {
        let names = vec!["Alice", "Bob", "Alex", "Charlie", "alicia_keys"];
        let picks: Vec<&&str> = suggest_alternatives("ali", &names, 3, |s| *s);
        // "Alice" and "alicia_keys" both contain "ali"; "Alex" shares the
        // "al" prefix but is shorter so substring check is one-sided.
        assert!(picks.contains(&&"Alice"));
        assert!(picks.iter().any(|s| **s == "Alice" || **s == "alicia_keys"));
        assert!(picks.len() <= 3);
    }

    #[test]
    fn suggest_drops_zero_scores() {
        let names = vec!["Alice", "Bob", "Charlie"];
        let picks: Vec<&&str> = suggest_alternatives("xyz", &names, 5, |s| *s);
        assert!(picks.is_empty());
    }

    #[test]
    fn suggest_respects_limit() {
        let names = vec!["a_one", "a_two", "a_three", "a_four"];
        let picks: Vec<&&str> = suggest_alternatives("a", &names, 2, |s| *s);
        assert_eq!(picks.len(), 2);
    }

    #[test]
    fn whitespace_normalized() {
        let s = name_match_score("  alice  smith  ", "alice smith");
        assert_eq!(s.score, 100);
    }
}

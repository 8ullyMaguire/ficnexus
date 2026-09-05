//! Heuristics for the "Ask the Archive" vs. plain search decision.
//!
//! `POST /api/search/ask` already gracefully falls back to plain full-text
//! search when Ollama is down, so the two endpoints aren't strictly
//! disjoint — Ask is a *frontend* choice. The question these helpers
//! answer is: given a user's typed query, should the UI nudge them to
//! the Ask endpoint instead?
//!
//! Rule of thumb:
//! - Plain keyword-style queries ("harry potter", "draco hermione") → search
//! - Natural-language questions / constraints in prose ("dark harry potter
//!   complete over 50k") → Ask
//!
//! These are deliberately lightweight — full NL detection is Ollama's job.
//! The goal is to give the UI a sane default, not to outsmart the user.

/// Lower bound for "this query has structure worth a translation". Below
/// this we don't even bother suggesting Ask — just go straight to FTS.
pub const ASK_SUGGEST_MIN_LEN: usize = 8;

/// Does `query` look like a natural-language request that Ask handles
/// better than the FTS parser?
///
/// We look for three signals:
/// 1. Length — too short to be NL
/// 2. Multiple word tokens (Ask works best with 4+ words)
/// 3. At least one "NL connector" — words that almost never appear in
///    keyword-style search ("over", "long", "complete", "wip", "short",
///    "finished", "with", "without", "oneshot", "english", "rated")
pub fn should_suggest_ask(query: &str) -> bool {
    let q = query.trim();
    if q.len() < ASK_SUGGEST_MIN_LEN {
        return false;
    }
    let tokens: Vec<&str> = q.split_whitespace().collect();
    if tokens.len() < 3 {
        return false;
    }
    let lower_tokens: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();
    let has_connector = lower_tokens
        .iter()
        .any(|t| NL_CONNECTORS.contains(&t.as_str()));
    if !has_connector {
        return false;
    }
    // Reject pure-operator strings ("-foo +bar" → just use search).
    !looks_like_operator_query(&lower_tokens)
}

const NL_CONNECTORS: &[&str] = &[
    "over",
    "under",
    "long",
    "short",
    "complete",
    "completed",
    "wip",
    "finished",
    "oneshot",
    "with",
    "without",
    "english",
    "rated",
    "explicit",
    "in",
    "and",
    "or",
    "but",
    "by",
    "from",
    "about",
    "the",
    "a",
    "an",
];

/// Bare heuristic: if the query is mostly `+/-` prefixed tokens, it's
/// already a structured operator string and Ask won't help.
fn looks_like_operator_query(tokens: &[String]) -> bool {
    let operators = tokens
        .iter()
        .filter(|t| t.starts_with('+') || t.starts_with('-'))
        .count();
    operators * 2 >= tokens.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_queries_dont_suggest_ask() {
        assert!(!should_suggest_ask("harry"));
        assert!(!should_suggest_ask("hp dm"));
        assert!(!should_suggest_ask("a"));
    }

    #[test]
    fn keyword_only_queries_dont_suggest_ask() {
        // No NL connectors — just topic words
        assert!(!should_suggest_ask("harry potter draco"));
    }

    #[test]
    fn nl_request_with_complete_suggests_ask() {
        assert!(should_suggest_ask("dark harry potter complete over 50k"));
        assert!(should_suggest_ask("harry potter oneshot english"));
    }

    #[test]
    fn operator_heavy_query_doesnt_suggest_ask() {
        // Even with connectors, the structure says "I know what I want"
        assert!(!should_suggest_ask("+harry -draco +romance -angst"));
    }

    #[test]
    fn whitespace_only_doesnt_suggest_ask() {
        assert!(!should_suggest_ask("   "));
        assert!(!should_suggest_ask(""));
    }

    #[test]
    fn mixed_case_works() {
        assert!(should_suggest_ask("Dark Harry Potter COMPLETE over 50k"));
    }
}

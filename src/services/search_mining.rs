//! Zero-result search mining.
//!
//! Users type queries that return nothing. Some are typos or dead ends, but
//! many are *demand signals*: real people looking for content we don't
//! have. This service clusters those zero-hit queries (last 30 days) into
//! themes with the LLM, so curators get a ranked "content to acquire" list
//! (e.g. "people want long Hermione-centric Harry Potter fics").
//!
//! Best-effort: an Ollama failure falls back to a plain SQL grouping (no
//! LLM clustering) so the endpoint always returns something useful.

use crate::config::Config;
use crate::services::ollama::OllamaClient;
use serde::{Deserialize, Serialize};

/// One mined theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinedTheme {
    pub theme: String,
    pub queries: Vec<String>,
    pub count: i32,
    /// Curator hint: what to acquire/ingest to serve this demand.
    pub suggestion: String,
}

/// Query zero-hit search_queries from the last `days` days, grouped by
/// normalized query with frequency. Returns (query, count) rows.
pub async fn zero_hit_queries(
    db: &sqlx::PgPool,
    days: i32,
    limit: usize,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT query, count(*) AS cnt
           FROM search_queries
           WHERE total_results = 0
             AND ts > NOW() - ($1 || ' days')::interval
             AND length(query) BETWEEN 2 AND 120
           GROUP BY query
           ORDER BY cnt DESC
           LIMIT $2"#,
    )
    .bind(days)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Ask the LLM to cluster zero-hit queries into themes.
fn build_prompt(queries: &[(String, i64)]) -> String {
    let list: String = queries
        .iter()
        .map(|(q, c)| format!("- {q} (x{c})"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"You are analyzing failed search queries on a fanfiction archive.
Group these queries into themes. For each theme return ONE line:
THEME | suggestion | query1;query2;query3

- theme: short label (e.g. "Hermione-centric Harry Potter").
- suggestion: a curator hint for content to acquire (e.g. "ingest Hermione
  main-char fics from AO3").
- queries: semicolon-separated original queries in this theme.

Only include themes with at least 2 queries. Ignore obvious typos/garbage.

QUERIES:
{list}"#,
        list = list
    )
}

/// Parse the model's theme lines. Never errors.
pub fn parse_themes(s: &str) -> Vec<MinedTheme> {
    let mut themes = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '|').map(|p| p.trim()).collect();
        if parts.len() < 3 {
            continue;
        }
        let queries: Vec<String> = parts[2]
            .split(';')
            .map(|q| q.trim().to_string())
            .filter(|q| !q.is_empty())
            .collect();
        if queries.is_empty() {
            continue;
        }
        let query_count = queries.len() as i32;
        themes.push(MinedTheme {
            theme: parts[0].to_string(),
            suggestion: parts[1].to_string(),
            queries,
            count: query_count,
        });
    }
    themes
}

/// Mine zero-hit queries: SQL group first, then LLM cluster. Falls back to
/// a plain per-query listing when Ollama is unavailable.
pub async fn mine(
    db: &sqlx::PgPool,
    ollama: &OllamaClient,
    config: &Config,
    days: i32,
    limit: usize,
) -> (Vec<MinedTheme>, Vec<(String, i64)>) {
    let Ok(queries) = zero_hit_queries(db, days, limit).await else {
        return (vec![], vec![]);
    };
    if queries.is_empty() {
        return (vec![], queries);
    }

    match ollama
        .generate(&build_prompt(&queries), &config.ollama_chat_model)
        .await
    {
        Ok(reply) => {
            let themes = parse_themes(&reply);
            if themes.is_empty() {
                (vec![], queries)
            } else {
                (themes, vec![])
            }
        }
        Err(e) => {
            tracing::warn!("search mining skipped (ollama): {e}");
            (vec![], queries)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_theme_lines() {
        let s = "Hermione-centric HP | ingest Hermione main-char AO3 fics | dark harry;hermione adventures\n\
                 RWBY crossover | acquire RWBY crossovers | rwby crossover;ruby and jaune";
        let themes = parse_themes(s);
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].theme, "Hermione-centric HP");
        assert_eq!(themes[0].queries, vec!["dark harry", "hermione adventures"]);
        assert_eq!(themes[1].count, 2);
    }

    #[test]
    fn ignores_garbage_lines() {
        let s = "garbage\njust a word\n";
        assert!(parse_themes(s).is_empty());
    }

    #[test]
    fn prompt_lists_queries_with_counts() {
        let p = build_prompt(&[("dark harry".into(), 3)]);
        assert!(p.contains("dark harry (x3)"));
    }
}

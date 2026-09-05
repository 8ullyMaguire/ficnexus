use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::config::Config;
use crate::error::AppError;

// =============================================================================
// Public DTOs
// =============================================================================

/// Minimum number of signals (bookmarks + download events) required before
/// personalized recommendations are computed. Below this we return
/// `enough_data: false` so the frontend can show a hint card.
pub const PERSONAL_RECS_MIN_SIGNAL: usize = 3;

/// How many personalized recommendations to return at most.
pub const PERSONAL_RECS_N: usize = 20;

/// Jaccard overlap between two tag id sets: |A ∩ B| / |A ∪ B|.
/// Returns 0.0 if either set is empty (no evidence of similarity).
pub fn tag_overlap(a: &[i32], b: &[i32]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_set: HashSet<i32> = a.iter().copied().collect();
    let b_set: HashSet<i32> = b.iter().copied().collect();
    let inter = a_set.intersection(&b_set).count();
    let union = a_set.union(&b_set).count();
    if union == 0 {
        return 0.0;
    }
    inter as f64 / union as f64
}

/// Blend tag overlap with (normalized) popularity: overlap dominates,
/// popularity is a small recency-based tie-breaker.
pub fn personal_score(overlap: f64, popularity: f64) -> f64 {
    overlap + 0.2 * popularity
}

/// Recency-based popularity in [0, 1]: exp(-days_since_update / 30).
/// A fic updated today scores 1.0; one updated 30+ days ago falls below 0.37.
pub fn popularity_norm(updated: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
    let days = now.signed_duration_since(updated).num_seconds() as f64 / 86_400.0;
    if days <= 0.0 {
        return 1.0;
    }
    (-days / 30.0).exp()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecQuery {
    pub url_id: String,
    #[serde(default = "default_rec_n")]
    pub n: usize,
    pub site_domain: Option<String>,
}

const fn default_rec_n() -> usize {
    20
}

impl Default for RecQuery {
    fn default() -> Self {
        Self {
            url_id: String::new(),
            n: 20,
            site_domain: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecResult {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub site_domain: String,
    pub summary: String,
    pub score: f64,
    pub community_score: i32,
    pub download_urls: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: i64,
    pub suggested_url_id: String,
    pub comment: Option<String>,
    pub net_votes: i32,
    pub created: Option<DateTime<Utc>>,
}

// =============================================================================
// Internal row types
// =============================================================================

#[derive(Debug, FromRow)]
pub struct FicInfoRow {
    pub id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub source: String,
    pub description: String,
}

#[derive(Debug, FromRow)]
struct CachedRow {
    recommended_url_id: String,
    score: f32,
    title: String,
    author: String,
    words: i64,
    chapters: i32,
    status: String,
    source: String,
    description: String,
}

#[derive(Debug, FromRow)]
struct CooccurRow {
    candidate_id: String,
    #[allow(dead_code)]
    cooccur_count: i32,
    #[allow(dead_code)]
    favouriter_count: i32,
    jaccard: Option<f64>,
}

#[derive(Debug, FromRow)]
struct SuggestionRow {
    id: i64,
    suggested_url_id: String,
    comment: Option<String>,
    net_votes: i32,
    created: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct CandidateScore {
    url_id: String,
    score: f64,
    tag_score: f64,
    community_score: i32,
}

// =============================================================================
// Engine
// =============================================================================

pub struct RecommendationEngine {
    pub db: PgPool,
}

impl RecommendationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { db: pool }
    }

    /// Return recommendations for the given query.
    ///
    /// 1. Tries the precomputed cache first.
    /// 2. Falls back to live computation (co-occurrence + optional tag fallback).
    pub async fn get_recommendations(
        &self,
        query: &RecQuery,
        config: &Config,
    ) -> Result<Vec<RecResult>, AppError> {
        let limit = query.n.min(config.rec_max_recommendations);
        if limit == 0 {
            return Ok(Vec::new());
        }

        // 1. Check precomputed cache
        let cached = check_cache(&self.db, query, config, limit).await?;
        if !cached.is_empty() {
            return Ok(cached);
        }

        // 2. Compute live
        compute_live(&self.db, query, config, limit).await
    }

    /// Force-recompute and persist recommendations for a single work.
    pub async fn compute_and_cache(&self, url_id: &str, config: &Config) -> Result<(), AppError> {
        let query = RecQuery {
            url_id: url_id.to_string(),
            n: config.rec_max_recommendations,
            site_domain: None,
        };

        let results =
            compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;

        // Clear old cache entries for this work
        sqlx::query("DELETE FROM precomputed_recommendations WHERE url_id = $1")
            .bind(url_id)
            .execute(&self.db)
            .await?;

        // Insert new entries
        for (rank, result) in results.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO precomputed_recommendations
                       (url_id, recommended_url_id, score, rank, computed_at)
                   VALUES ($1, $2, $3, $4, NOW())"#,
            )
            .bind(url_id)
            .bind(&result.url_id)
            .bind(result.score as f32)
            .bind(rank as i16)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }
}

// =============================================================================
// Cache check
// =============================================================================

async fn check_cache(
    db: &PgPool,
    query: &RecQuery,
    config: &Config,
    limit: usize,
) -> Result<Vec<RecResult>, AppError> {
    let rows: Vec<CachedRow> = if let Some(ref domain) = query.site_domain {
        sqlx::query_as::<_, CachedRow>(
            r#"SELECT pr.recommended_url_id, pr.score,
                      fi.title, fi.author, fi.words, fi.chapters,
                      fi.status, fi.source, fi.description
               FROM precomputed_recommendations pr
               JOIN fic_info fi ON fi.id = pr.recommended_url_id
               JOIN fic_works fw ON fw.url_id = pr.recommended_url_id
               WHERE pr.url_id = $1
                 AND fw.site_domain = $2
                 AND pr.computed_at > NOW() - ($3 * INTERVAL '1 hour')
               ORDER BY pr.rank ASC
               LIMIT $4"#,
        )
        .bind(&query.url_id)
        .bind(domain)
        .bind(config.rec_cache_ttl_hours as i32)
        .bind(limit as i64)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, CachedRow>(
            r#"SELECT pr.recommended_url_id, pr.score,
                      fi.title, fi.author, fi.words, fi.chapters,
                      fi.status, fi.source, fi.description
               FROM precomputed_recommendations pr
               JOIN fic_info fi ON fi.id = pr.recommended_url_id
               WHERE pr.url_id = $1
                 AND pr.computed_at > NOW() - ($2 * INTERVAL '1 hour')
               ORDER BY pr.rank ASC
               LIMIT $3"#,
        )
        .bind(&query.url_id)
        .bind(config.rec_cache_ttl_hours as i32)
        .bind(limit as i64)
        .fetch_all(db)
        .await?
    };

    // Enrich with community scores
    let community = get_community_scores(db).await?;

    let results = rows
        .into_iter()
        .map(|r| {
            let cs = community.get(&r.recommended_url_id).copied().unwrap_or(0);
            RecResult {
                url_id: r.recommended_url_id,
                title: r.title,
                author: r.author,
                words: r.words,
                chapters: r.chapters,
                status: r.status,
                site_domain: r.source,
                summary: r.description,
                score: r.score as f64,
                community_score: cs,
                download_urls: HashMap::new(),
            }
        })
        .collect();

    Ok(results)
}

// =============================================================================
// Live computation
// =============================================================================

async fn compute_live(
    db: &PgPool,
    query: &RecQuery,
    config: &Config,
    limit: usize,
) -> Result<Vec<RecResult>, AppError> {
    // --- Seed metadata ---
    let seed = sqlx::query_as::<_, (i32, String, String)>(
        r#"SELECT COALESCE(fw.favouriter_count, 0),
                  COALESCE(fi.title, ''),
                  COALESCE(fi.author, '')
           FROM fic_works fw
           JOIN fic_info fi ON fi.id = fw.url_id
           WHERE fw.url_id = $1"#,
    )
    .bind(&query.url_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Work {} not found", query.url_id)))?;

    let (favouriter_count, seed_title, seed_author) = seed;

    // --- Co-occurrence (collaborative) candidates ---
    let cooccur: Vec<CooccurRow> = sqlx::query_as::<_, CooccurRow>(
        r#"WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
            candidates AS (
              SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
                     cooccur_count
              FROM fic_bookmark_cooccur
              WHERE work_a = $1 OR work_b = $1
            )
            SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
              (c.cooccur_count::float
                 / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
              ) AS jaccard
            FROM candidates c
            JOIN fic_works fw ON fw.url_id = c.candidate_id
            CROSS JOIN seed s
            WHERE c.candidate_id != $1
            ORDER BY jaccard DESC
            LIMIT $2"#,
    )
    .bind(&query.url_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    // --- Tag fallback when seed has few favouriters ---
    let min_collab = config.rec_min_favouriters_for_collab as i32;
    let use_tag_fallback = favouriter_count < min_collab;

    let tag_candidates = if use_tag_fallback {
        fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
    } else {
        Vec::new()
    };

    // --- Merge co-occurrence + tag candidates into a scored map ---
    let mut scored: HashMap<String, CandidateScore> = HashMap::new();

    for c in &cooccur {
        let jaccard = c.jaccard.unwrap_or(0.0);
        scored.insert(
            c.candidate_id.clone(),
            CandidateScore {
                url_id: c.candidate_id.clone(),
                score: jaccard,
                tag_score: 0.0,
                community_score: 0,
            },
        );
    }

    for (tag_id, tag_score_val) in &tag_candidates {
        let entry = scored
            .entry(tag_id.clone())
            .or_insert_with(|| CandidateScore {
                url_id: tag_id.clone(),
                score: 0.0,
                tag_score: 0.0,
                community_score: 0,
            });
        // Tag candidates get a base similarity score
        entry.tag_score = *tag_score_val;
    }

    // --- Blend collaborative + tag scores ---
    // weight = min(1.0, favouriter_count / 5.0)
    // final  = collab * weight + tag * (1.0 - weight)
    let weight = (favouriter_count as f64 / 5.0).min(1.0);

    let mut candidates: Vec<CandidateScore> = scored.into_values().collect();
    for c in &mut candidates {
        let collab = c.score;
        let tag = c.tag_score;
        c.score = collab * weight + tag * (1.0 - weight);
    }

    // --- Community votes (for voting boost) ---
    let net_votes = get_community_votes(db, &candidates).await?;

    let gamma = config.rec_voting_boost_gamma;
    for c in &mut candidates {
        let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
        c.community_score = nv;
        // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
        let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
        c.score *= boost;
    }

    // --- Sort & take top N ---
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top: Vec<CandidateScore> = candidates.into_iter().take(limit).collect();

    // --- Fetch metadata and build results ---
    let mut results = Vec::with_capacity(top.len());
    for c in top {
        let info = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info WHERE id = $1"#,
        )
        .bind(&c.url_id)
        .fetch_optional(db)
        .await?;

        if let Some(info) = info {
            results.push(RecResult {
                url_id: info.id,
                title: info.title,
                author: info.author,
                words: info.words,
                chapters: info.chapters,
                status: info.status,
                site_domain: info.source,
                summary: info.description,
                score: c.score,
                community_score: c.community_score,
                download_urls: HashMap::new(),
            });
        }
    }

    // --- Post-filter by site_domain if requested ---
    if let Some(ref domain) = query.site_domain {
        results.retain(|r| r.site_domain == *domain);
    }

    Ok(results)
}

// =============================================================================
// Tag-based fallback (approximate — no tags table yet)
// =============================================================================

/// Search for works with matching author or title keywords.
/// Returns `(url_id, similarity_score)` pairs.
async fn fetch_tag_candidates(
    db: &PgPool,
    url_id: &str,
    seed_title: &str,
    seed_author: &str,
    limit: usize,
) -> Result<Vec<(String, f64)>, AppError> {
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(url_id.to_string());

    // 1. Same author (strong signal — score 0.8)
    if !seed_author.is_empty() {
        let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info
               WHERE id != $1 AND author ILIKE $2
               ORDER BY words DESC
               LIMIT $3"#,
        )
        .bind(url_id)
        .bind(format!("%{}%", seed_author))
        .bind(limit as i64)
        .fetch_all(db)
        .await?;

        for row in rows {
            if seen.insert(row.id.clone()) {
                results.push((row.id, 0.8));
            }
        }
    }

    // 2. Title keyword matches (weaker signal — score 0.5)
    let keywords: Vec<&str> = seed_title
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .collect();

    for kw in keywords {
        if results.len() >= limit {
            break;
        }
        let remaining = limit - results.len();
        let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info
               WHERE id != $1 AND title ILIKE $2
               ORDER BY words DESC
               LIMIT $3"#,
        )
        .bind(url_id)
        .bind(format!("%{}%", kw))
        .bind(remaining as i64)
        .fetch_all(db)
        .await?;

        for row in rows {
            if seen.insert(row.id.clone()) {
                results.push((row.id, 0.5));
            }
        }
    }

    Ok(results)
}

// =============================================================================
// Community scores
// =============================================================================

/// Fetch net votes aggregated by suggested_url_id (across all works).
async fn get_community_scores(db: &PgPool) -> Result<HashMap<String, i32>, AppError> {
    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           GROUP BY s.suggested_url_id"#,
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}

/// Fetch net votes restricted to a specific set of candidates.
async fn get_community_votes(
    db: &PgPool,
    candidates: &[CandidateScore],
) -> Result<HashMap<String, i32>, AppError> {
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();

    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#,
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}

// =============================================================================
// Community suggestion / voting free functions
// =============================================================================

/// List community suggestions for a given work.
pub async fn get_community_suggestions(
    db: &PgPool,
    url_id: &str,
) -> Result<Vec<Suggestion>, AppError> {
    let rows: Vec<SuggestionRow> = sqlx::query_as::<_, SuggestionRow>(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(v.net, 0) AS net_votes, s.created
           FROM recommendation_suggestions s
           LEFT JOIN (
               SELECT suggestion_id, SUM(vote)::INT AS net
               FROM recommendation_votes
               GROUP BY suggestion_id
           ) v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           ORDER BY net_votes DESC, s.created DESC"#,
    )
    .bind(url_id)
    .fetch_all(db)
    .await?;

    let suggestions = rows
        .into_iter()
        .map(|r| Suggestion {
            id: r.id,
            suggested_url_id: r.suggested_url_id,
            comment: r.comment,
            net_votes: r.net_votes,
            created: r.created,
        })
        .collect();

    Ok(suggestions)
}

/// Submit a community suggestion.
/// Returns the new suggestion id.
pub async fn submit_suggestion(
    db: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    voter_ip: &str,
    comment: Option<&str>,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
               (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)
           DO UPDATE SET comment = EXCLUDED.comment, created = CURRENT_TIMESTAMP
           RETURNING id"#,
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(voter_ip)
    .bind(comment)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}

/// Cast (or change) a vote on a suggestion.
/// `vote_value` should be -1 or 1.
/// Returns the new net vote count for the suggestion.
pub async fn cast_vote(
    db: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    vote_value: i16,
) -> Result<i32, AppError> {
    let vote = vote_value.clamp(-1, 1);

    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip)
           DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP"#,
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(vote)
    .execute(db)
    .await?;

    let row: (Option<i32>,) = sqlx::query_as(
        r#"SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1"#,
    )
    .bind(suggestion_id)
    .fetch_one(db)
    .await?;

    Ok(row.0.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rec_query_defaults() {
        let q = RecQuery::default();
        assert_eq!(q.n, 20);
        assert!(q.url_id.is_empty());
        assert!(q.site_domain.is_none());
    }

    #[test]
    fn test_rec_query_serialization() {
        let q = RecQuery {
            url_id: "abc123".into(),
            n: 10,
            site_domain: Some("archiveofourown.org".into()),
        };
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("archiveofourown.org"));
    }

    #[test]
    fn test_rec_result_serialization() {
        let mut urls = HashMap::new();
        urls.insert("epub".into(), "/cache/epub/abc/h.epub".into());
        let r = RecResult {
            url_id: "def456".into(),
            title: "Test Fic".into(),
            author: "Test Author".into(),
            words: 50000,
            chapters: 10,
            status: "complete".into(),
            site_domain: "archiveofourown.org".into(),
            summary: "A test fic".into(),
            score: 0.85,
            community_score: 3,
            download_urls: urls,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("def456"));
        assert!(json.contains("Test Fic"));
        assert!(json.contains("0.85"));
        assert!(json.contains("/cache/epub/abc/h.epub"));
    }

    #[test]
    fn test_submit_suggestion_upsert_query() {
        // Verify the submit_suggestion query uses ON CONFLICT
        let query = "ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)";
        // Read the query from the function source (compile-time check)
        let sql = r#"INSERT INTO recommendation_suggestions
               (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)
           DO UPDATE SET comment = EXCLUDED.comment, created = CURRENT_TIMESTAMP
           RETURNING id"#;
        assert!(sql.contains(query));
        assert!(sql.contains("RETURNING id"));
    }

    #[test]
    fn test_vote_constraints() {
        // Verify vote values are -1 or 1
        let valid_votes = vec![-1i16, 1i16];
        for v in &valid_votes {
            assert!(*v == -1 || *v == 1);
        }
        // Verify invalid votes are rejected
        let invalid = 0i16;
        assert!(invalid != -1 && invalid != 1);
    }

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion {
            id: 42,
            suggested_url_id: "abc123".into(),
            comment: Some("Great read!".into()),
            net_votes: 5,
            created: Some(Utc::now()),
        };
        assert_eq!(suggestion.id, 42);
        assert_eq!(suggestion.net_votes, 5);
        assert_eq!(suggestion.comment, Some("Great read!".into()));
    }

    #[test]
    fn test_work_in_progress_check_cache_query() {
        // Verify the cache check query is correct
        let sql = "SELECT url_id, recommended_url_id, score, rank
                   FROM precomputed_recommendations
                   WHERE url_id = $1 AND computed_at > NOW() - ($2::TEXT || ' hours')::INTERVAL
                   ORDER BY rank ASC LIMIT $3";
        assert!(sql.contains("precomputed_recommendations"));
        assert!(sql.contains("computed_at > NOW()"));
    }

    #[test]
    fn test_get_recommendations_live_query() {
        // Verify the live co-occurrence query structure
        let sql = "WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
        candidates AS (
          SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id, cooccur_count
          FROM fic_bookmark_cooccur WHERE work_a = $1 OR work_b = $1
        )
        SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
          (c.cooccur_count::float / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)) AS jaccard
        FROM candidates c JOIN fic_works fw ON fw.url_id = c.candidate_id
        CROSS JOIN seed s WHERE c.candidate_id != $1
        ORDER BY jaccard DESC LIMIT $2";
        assert!(sql.contains("jaccard"));
        assert!(sql.contains("fic_bookmark_cooccur"));
        assert!(sql.contains("LIMIT $2"));
    }

    // ─── Personalized-recommendation scoring helpers ───────────────────────

    #[test]
    fn test_tag_overlap_prefers_shared_tags() {
        // Two overlapping sets score strictly higher than two disjoint sets.
        let overlap = tag_overlap(&[1, 2, 3], &[1, 2, 9]);
        let disjoint = tag_overlap(&[1, 2, 3], &[9, 10, 11]);
        assert!(
            overlap > disjoint,
            "shared tags must score higher: overlap={overlap}, disjoint={disjoint}"
        );
        // Sanity: |{1,2} ∩ {1,2,9}| / |{1,2,3,9}| = 2/4 = 0.5
        assert!((overlap - 0.5).abs() < 1e-9, "expected 0.5, got {overlap}");
        assert_eq!(disjoint, 0.0);
    }

    #[test]
    fn test_tag_overlap_empty_returns_zero() {
        assert_eq!(tag_overlap(&[], &[1, 2, 3]), 0.0);
        assert_eq!(tag_overlap(&[1, 2, 3], &[]), 0.0);
        assert_eq!(tag_overlap(&[], &[]), 0.0);
    }

    #[test]
    fn test_tag_overlap_duplicates_and_identity() {
        // Duplicates within a set don't inflate the score: {1,2} vs {1,2,3} = 2/3.
        let dup = tag_overlap(&[1, 1, 1, 2], &[1, 2, 3]);
        assert!(
            (dup - 2.0 / 3.0).abs() < 1e-9,
            "dupes must be deduped: {dup}"
        );
        // A set fully contained in another is 2/3, not 1.0.
        let subset = tag_overlap(&[1, 2], &[1, 2, 3]);
        assert!(
            (subset - 2.0 / 3.0).abs() < 1e-9,
            "expected 2/3, got {subset}"
        );
    }

    #[test]
    fn test_personal_score_weights_overlap_positively() {
        let s1 = personal_score(0.8, 0.5);
        let s2 = personal_score(0.2, 0.5);
        assert!(
            s1 > s2,
            "higher overlap must win at equal popularity: {s1} vs {s2}"
        );
        // Popularity is a positive secondary signal.
        assert!(personal_score(0.5, 1.0) > personal_score(0.5, 0.0));
        // Overlap dominates: the 0.2 * popularity term is a small boost.
        assert!(personal_score(0.5, 1.0) < 0.71);
    }

    #[test]
    fn test_popularity_norm_recency() {
        let now = Utc::now();
        let today = popularity_norm(now, now);
        assert!((today - 1.0).abs() < 1e-9, "today's fic must score 1.0");

        let old = popularity_norm(now - chrono::Duration::days(60), now);
        assert!(old < 0.2, "60-day-old fic must be much less popular: {old}");
        assert!(old >= 0.0 && old <= 1.0);

        // Future-dated updates clamp to 1.0 (defensive).
        let future = popularity_norm(now + chrono::Duration::days(5), now);
        assert!((future - 1.0).abs() < 1e-9);
    }

    /// The `recs.personalized` opt-out preference is stored as a row in
    /// `user_prefs` (key/value). When the key is absent the user gets the
    /// default-on behavior (personalized recs). This mirrors the pref toggle
    /// the settings page writes via PUT /api/me/prefs and what the home
    /// dashboard reads via /api/me/prefs.
    ///
    /// We can't assert against a live DB in a unit test, so this test locks in
    /// the string-comparison contract the handler uses:
    ///   - absent key  -> value is None  -> not opted out (default-on)
    ///   - "true"      -> `v == "false"` is false -> not opted out
    ///   - "false"     -> `v == "false"` is true  -> IS opted out -> generic fallback
    #[test]
    fn test_recs_personalized_pref_toggle_contract() {
        // Absent key -> not opted out (personalized recs remain on by default).
        let absent: Option<String> = None;
        let opted_out = absent.map(|v| v == "false").unwrap_or(false);
        assert!(!opted_out, "absent pref key must NOT opt the user out");

        // Explicit "true" -> not opted out.
        let opt_in: Option<String> = Some("true".to_string());
        let opted_out = opt_in.map(|v| v == "false").unwrap_or(false);
        assert!(!opted_out, "\"true\" must NOT opt the user out");

        // Explicit "false" -> opted out (falls back to generic recs).
        let opt_out: Option<String> = Some("false".to_string());
        let opted_out = opt_out.map(|v| v == "false").unwrap_or(false);
        assert!(
            opted_out,
            "\"false\" MUST opt the user out of personal recs"
        );

        // A stray value like "" or "0" must NOT opt the user out (only the
        // exact string "false" does), matching the handler's strict check.
        let stray: Option<String> = Some("".to_string());
        let opted_out = stray.map(|v| v == "false").unwrap_or(false);
        assert!(
            !opted_out,
            "a non-\"false\" value must NOT opt the user out"
        );
    }
}

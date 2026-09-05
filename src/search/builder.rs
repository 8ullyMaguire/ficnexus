use chrono::{DateTime, Utc};
use sqlx::Postgres;
use sqlx::QueryBuilder;
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::search::parser::{Field, FieldOp, FieldQuery, RangeExpr};

/// `fic_tags.role_confidence` threshold for the `@` role modifier (main /
/// primary tag). The backfill in migration 003 maps score>=10 → 1.0, so a
/// main/primary tag carries confidence >= 1.0.
const ROLE_MAIN_CONFIDENCE: f32 = 1.0;

/// A single tag filter in the form (tag_type_id, tag_name)
#[derive(Debug, Clone)]
pub struct TagFilter {
    pub tag_type_id: i16,
    pub tag_name: String,
}

/// Parsed search parameters used by the query builder
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    /// Full-text search query
    pub q: Option<String>,
    /// Parsed tsquery string from the boolean query parser (to_tsquery, not plainto_tsquery)
    pub parsed_tsquery: Option<String>,
    /// Fuzzy fallback: when true, bare text queries also match title/author
    /// via ILIKE '%term%' (pg_trgm-backed). Used for typo tolerance when the
    /// exact full-text search returns nothing (see routes.rs).
    pub fuzzy: bool,
    /// Fielded terms extracted from query (title:, author:, etc.)
    pub fielded_terms: Vec<(String, String)>,
    /// Structured v2 field expressions extracted from query (`@char:X`,
    /// `romship:A/B`, `words:>50k`, `crossover:1`, `primary_fandom:F`…).
    pub field_queries: Vec<FieldQuery>,
    /// Explicit "Guided tier" helper: a main/primary character name
    /// (`char:Name` with main role). Backed by role_confidence.
    pub main_char: Option<String>,
    /// Explicit status filter mapping to `fic_info.status` (e.g. "complete").
    pub status: Option<String>,
    /// Explicit beta/editing status filter mapping to `works.beta_status`.
    pub beta_status: Option<String>,
    /// Explicit crossover filter (fandom count > 1).
    pub crossover: Option<bool>,
    /// Minimum / maximum hit count (works.hit_count).
    pub min_hits: Option<i64>,
    pub max_hits: Option<i64>,
    /// Comma-separated character names — finds relationships containing these characters
    pub relationship_characters: Option<String>,
    /// Tags that ALL must be present (AND) — name-based
    pub include_tags: Vec<TagFilter>,
    /// Tags that NONE must be present (AND NOT) — name-based
    pub exclude_tags: Vec<TagFilter>,
    /// Tag TYPE ids to exclude entirely: fics carrying ANY tag of these types
    /// are filtered out (NOT EXISTS on the type). E.g. exclude_tag_types=3
    /// hides every fic with a relationship tag ("no ships"); combined with
    /// exclude_tag_types=6 it also hides category tags ("strict gen").
    pub exclude_tag_types: Vec<i16>,
    /// Strict Gen mode: hide fics with relationship tags (type 3) or
    /// category tags (type 6, e.g. M/M, F/F). Shorthand for
    /// exclude_tag_types=3,6.
    pub strict_gen: bool,
    /// Tags where at least ONE must be present (OR) — name-based
    pub include_any_tags: Vec<TagFilter>,
    /// Pre-resolved tag IDs for include_tags (each inner vec has canonical + synonym IDs)
    /// If empty, the builder falls back to name-based filtering
    pub expanded_include_tag_ids: Vec<Vec<i32>>,
    /// Pre-resolved tag IDs for exclude_tags (each inner vec has canonical + synonym IDs)
    pub expanded_exclude_tag_ids: Vec<Vec<i32>>,
    /// Pre-resolved tag IDs for include_any_tags (each inner vec has canonical + synonym IDs)
    pub expanded_include_any_tag_ids: Vec<Vec<i32>>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    /// Maps to status = 'complete'
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    /// Primary tag: fic must have this tag as its FIRST tag
    pub primary_tag: Option<TagFilter>,
    /// Minimum comment count
    pub min_comments: Option<i64>,
    /// Minimum kudos/bookmark count
    pub min_kudos: Option<i64>,
    /// Maximum kudos count filter
    pub max_kudos: Option<i64>,
    /// Minimum bookmarks count filter
    pub min_bookmarks: Option<i64>,
    /// Maximum bookmarks count filter
    pub max_bookmarks: Option<i64>,
    /// Content rating filter (resolved to a tag with tag_type_id = 7)
    pub rating: Option<String>,
    /// Language filter (future-proof, inert for now)
    pub language: Option<String>,
    /// Exclude works with any archive warnings
    pub no_warnings: Option<bool>,
    /// Search by numeric tag ID
    pub tag_ids: Vec<i32>,
    /// Sort field: "-relevance", "-date", "-words", "-chapters", "-title".
    /// Default "-date" when no q, "-relevance" when q is present.
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    /// Hide works the signed-in user has marked read (reading_stats status
    /// = 'completed'). Anonymous users: no-op (no read data to filter on).
    pub hide_read: bool,
    /// Hide works the signed-in user has bookmarked.
    pub hide_bookmarked: bool,
    /// Library-only mode: return ONLY works the signed-in user bookmarked.
    /// Mutually exclusive with hide_bookmarked (library_only wins).
    pub library_only: bool,
    /// The signed-in user id, set by the route handler from the JWT.
    /// None/0 = anonymous — personal filters (hide_read, hide_bookmarked,
    /// library_only) are no-ops.
    pub user_id: Option<i32>,
}

/// A single search result item (fic metadata + optional rank + snippet)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchResult {
    pub url_id: String,
    pub work_id: Option<i32>,
    pub title: String,
    pub author: String,
    pub source: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub description: String,
    pub updated: Option<DateTime<Utc>>,
    pub rank: Option<f32>,
    /// ts_headline snippet with <b> highlight tags (None without a text q)
    pub snippet: Option<String>,
}

/// Internal DB row for search queries — matches fic_info columns + optional
/// rank + optional snippet (ts_headline)
#[derive(Debug, Clone)]
pub struct FicSearchRow {
    pub id: String,
    pub work_id: Option<i32>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
    pub rank: Option<f32>,
    /// ts_headline snippet: highlighted description excerpt, or None when no
    /// text query / no highlightable match.
    pub snippet: Option<String>,
}

/// Manual `FromRow` impl so `rank` can be optional (present only with FTS).
impl<'r> sqlx::FromRow<'r, PgRow> for FicSearchRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Column;
        let columns: Vec<String> = row.columns().iter().map(|c| c.name().to_string()).collect();

        let has_rank = columns.iter().any(|c| c == "rank");
        let has_work_id = columns.iter().any(|c| c == "work_id");
        let has_snippet = columns.iter().any(|c| c == "snippet");

        Ok(FicSearchRow {
            id: row.try_get("id")?,
            work_id: if has_work_id {
                row.try_get("work_id").ok()
            } else {
                None
            },
            created: row.try_get("created")?,
            updated: row.try_get("updated")?,
            title: row.try_get("title")?,
            author: row.try_get("author")?,
            author_url: row.try_get("author_url")?,
            author_local_id: row.try_get("author_local_id")?,
            chapters: row.try_get("chapters")?,
            words: row.try_get("words")?,
            description: row.try_get("description")?,
            fic_created: row.try_get("fic_created")?,
            fic_updated: row.try_get("fic_updated")?,
            status: row.try_get("status")?,
            source: row.try_get("source")?,
            extra_meta: row.try_get("extra_meta")?,
            raw_extended_meta: row.try_get("raw_extended_meta")?,
            source_id: row.try_get("source_id")?,
            author_id: row.try_get("author_id")?,
            content_hash: row.try_get("content_hash")?,
            rank: if has_rank {
                row.try_get("rank").ok()
            } else {
                None
            },
            snippet: if has_snippet {
                row.try_get("snippet").ok()
            } else {
                None
            },
        })
    }
}

impl From<FicSearchRow> for SearchResult {
    fn from(row: FicSearchRow) -> Self {
        SearchResult {
            url_id: row.id,
            work_id: row.work_id,
            title: row.title,
            author: row.author,
            source: row.source,
            words: row.words,
            chapters: row.chapters,
            status: row.status,
            description: row.description,
            updated: row.updated,
            rank: row.rank,
            snippet: row.snippet,
        }
    }
}

/// Builds dynamic SQL queries for fic search with tag filters.
pub struct SearchQueryBuilder {
    pub params: SearchParams,
    hidden_threshold: i16,
    has_comments: bool,
    has_ratings: bool,
    has_bookmarks: bool,
}

impl SearchQueryBuilder {
    pub fn new(
        params: SearchParams,
        hidden_threshold: i16,
        has_comments: bool,
        has_ratings: bool,
        has_bookmarks: bool,
    ) -> Self {
        Self {
            params,
            hidden_threshold,
            has_comments,
            has_ratings,
            has_bookmarks,
        }
    }

    /// Get the current page number (default 1).
    pub fn page(&self) -> usize {
        self.params.page.unwrap_or(1).max(1)
    }

    /// Build a query that returns the total count of matching rows.
    pub fn build_count_query(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM fic_info fi");
        self.push_works_join(&mut qb);
        qb.push(" WHERE 1=1");
        self.push_where_clauses(&mut qb);
        qb
    }

    /// Build a query that returns the paginated search results with rank.
    pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
        let has_q = self.params.q.is_some() || self.params.parsed_tsquery.is_some();

        // Build the SELECT clause with work_id from works table
        let mut qb = QueryBuilder::<Postgres>::new("SELECT fi.*, fi.work_id AS work_id, ");

        if let Some(ref tsq) = self.params.parsed_tsquery {
            qb.push("ts_rank(fi.text_search, to_tsquery('english', ");
            qb.push_bind(tsq);
            qb.push(")) AS rank, ts_headline('english', fi.description, to_tsquery('english', ");
            qb.push_bind(tsq);
            qb.push("), 'StartSel=<b>, StopSel=</b>, MaxWords=35, MinWords=15') AS snippet FROM fic_info fi");
        } else if let Some(ref q) = self.params.q {
            qb.push("ts_rank(fi.text_search, websearch_to_tsquery('english', ");
            qb.push_bind(q);
            qb.push(")) AS rank, ts_headline('english', fi.description, websearch_to_tsquery('english', ");
            qb.push_bind(q);
            qb.push("), 'StartSel=<b>, StopSel=</b>, MaxWords=35, MinWords=15') AS snippet FROM fic_info fi");
        } else {
            qb.push("NULL::real AS rank, NULL::text AS snippet FROM fic_info fi");
        }
        self.push_works_join(&mut qb);
        qb.push(" WHERE 1=1");

        self.push_where_clauses(&mut qb);

        // ORDER BY
        let sort =
            self.params
                .sort
                .as_deref()
                .unwrap_or(if has_q { "-relevance" } else { "-date" });

        qb.push(" ORDER BY ");
        // When the `works` join is present we can compose a relevance ranking
        // from full-text rank plus engagement weight; otherwise keep the
        // legacy rank ordering byte-for-byte (backward compatible).
        let can_compose = self.needs_works_join();
        match sort {
            "-relevance" | "relevance" => {
                if has_q {
                    if can_compose {
                        qb.push("rank DESC, COALESCE(w.kudos_count, 0) DESC, fi.fic_updated DESC");
                    } else {
                        qb.push("rank DESC, fi.fic_updated DESC");
                    }
                } else {
                    qb.push("fi.fic_updated DESC");
                }
            }
            // Frontend SORT_OPTIONS values ('updated', 'created', 'words',
            // 'kudos'), the URL-friendly '-field' descending forms, and the
            // bare-field ascending forms ('date' = oldest first, etc.).
            "-date" | "updated" | "newest" => {
                qb.push("fi.fic_updated DESC");
            }
            "date" | "-updated" | "oldest" => {
                qb.push("fi.fic_updated ASC");
            }
            "created" | "-created" => {
                qb.push("fi.fic_created DESC");
            }
            "words" | "-words" => {
                qb.push("fi.words DESC");
            }
            "chapters" => {
                qb.push("fi.chapters ASC");
            }
            "-chapters" => {
                qb.push("fi.chapters DESC");
            }
            "title" | "-title" => {
                qb.push("fi.title ASC");
            }
            "kudos" | "kudos_count" | "-kudos" => {
                if self.has_ratings {
                    qb.push("(SELECT COUNT(*) FROM kudos k WHERE k.work_id = fi.work_id AND k.user_id IS NOT NULL) DESC");
                } else {
                    qb.push("fi.fic_updated DESC");
                }
            }
            "bookmarks" | "bookmarks_count" | "-bookmarks" => {
                if self.has_bookmarks {
                    qb.push("(SELECT COUNT(*) FROM bookmarks b WHERE b.work_id = fi.work_id) DESC");
                } else {
                    qb.push("fi.fic_updated DESC");
                }
            }
            "comments" | "comments_count" | "-comments" => {
                if self.has_comments {
                    qb.push("(SELECT COUNT(*) FROM comments c WHERE c.work_id = fi.work_id) DESC");
                } else {
                    qb.push("fi.fic_updated DESC");
                }
            }
            "hits" | "-hits" => {
                // hits not tracked yet — fall back to recent
                qb.push("fi.fic_updated DESC");
            }
            _ => {
                qb.push("fi.fic_updated DESC");
            }
        }

        // LIMIT / OFFSET
        let page = self.params.page.unwrap_or(1).max(1);
        let per_page = self.params.per_page.unwrap_or(20).max(1);
        let offset = (page - 1) * per_page;
        qb.push(" LIMIT ");
        qb.push_bind(per_page as i64);
        qb.push(" OFFSET ");
        qb.push_bind(offset as i64);

        qb
    }

    /// Build a facet query that returns the top tags of the given type among
    /// fics matching the active search filters. The shared WHERE clauses are
    /// pushed via `push_where_clauses`; this query never inherits the
    /// LIMIT/OFFSET that `build_data_query` appends.
    pub fn build_facet_query(&self, tag_type_id: i16) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT t.name AS name, COUNT(*) AS count FROM fic_info fi",
        );
        self.push_works_join(&mut qb);
        qb.push(
            " \
             JOIN fic_tags ft ON ft.url_id = fi.id \
             JOIN tags t ON t.id = ft.tag_id \
             WHERE 1=1",
        );
        self.push_where_clauses(&mut qb);
        qb.push(" AND t.tag_type_id = ");
        qb.push_bind(tag_type_id);
        qb.push(" AND ft.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(" GROUP BY t.name ORDER BY COUNT(*) DESC LIMIT 20");
        qb
    }

    /// Build a facet query that returns status counts among fics matching the
    /// active search filters. No LIMIT/OFFSET from the data query is applied.
    pub fn build_status_facet_query(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT fi.status AS name, COUNT(*) AS count FROM fic_info fi",
        );
        self.push_works_join(&mut qb);
        qb.push(" WHERE 1=1");
        self.push_where_clauses(&mut qb);
        qb.push(" GROUP BY fi.status ORDER BY COUNT(*) DESC LIMIT 20");
        qb
    }

    /// Push WHERE clauses for all filters into the query builder.
    /// Shared by count, data, and facet queries. Does NOT add
    /// ORDER BY / LIMIT / OFFSET (those are applied per-query by callers).
    pub(crate) fn push_where_clauses(&self, qb: &mut QueryBuilder<Postgres>) {
        // Full-text search — use parsed tsquery if available (boolean
        // operators), else websearch_to_tsquery (native boolean syntax:
        // "quoted phrases", OR, -exclusion, implicit AND).
        let fuzzy_q: Option<&str> = if self.params.fuzzy {
            self.params.q.as_deref()
        } else {
            None
        };
        if let Some(ref tsq) = self.params.parsed_tsquery {
            // parsed_tsquery comes from the boolean parser (expr_to_tsquery),
            // which emits to_tsquery-native syntax (& | ! <->). Use
            // to_tsquery for it; websearch_to_tsquery is only for RAW user
            // text (the q branch below), which speaks websearch syntax
            // (OR, -exclusion, "phrases", implicit AND).
            qb.push(" AND fi.text_search @@ to_tsquery('english', ");
            qb.push_bind(tsq);
            qb.push(")");
            // Fuzzy fallback (pg_trgm): even with a parsed tsquery (which is
            // always set for bare text queries), a typo'd query matches nothing
            // via tsquery — add word_similarity ORs so "Hary Pottr" still finds
            // "Harry Potter". Kept with OR so exact matches rank higher.
            if let Some(q) = fuzzy_q {
                for term in q.split_whitespace().filter(|t| t.len() >= 3) {
                    qb.push(" OR word_similarity(");
                    qb.push_bind(term);
                    qb.push(", fi.title) > 0.35 OR word_similarity(");
                    qb.push_bind(term);
                    qb.push(", fi.author) > 0.35");
                }
            }
        } else if let Some(ref q) = self.params.q {
            qb.push(" AND (fi.text_search @@ websearch_to_tsquery('english', ");
            qb.push_bind(q);
            qb.push(")");
            // Fuzzy fallback (pg_trgm): when the exact full-text match fails,
            // also match title/author by trigram word-similarity. Kept inside
            // the same AND group with OR so exact matches still rank higher
            // (rank DESC ordering). word_similarity(term, text) measures how
            // well term appears inside text — 'hary' vs 'harry' ~ 0.6, while
            // ILIKE '%hary%' would match nothing. GIN gin_trgm_ops indexes
            // (migration 012) back the operator.
            if let Some(q) = fuzzy_q {
                for term in q.split_whitespace().filter(|t| t.len() >= 3) {
                    qb.push(" OR word_similarity(");
                    qb.push_bind(term);
                    qb.push(", fi.title) > 0.35 OR word_similarity(");
                    qb.push_bind(term);
                    qb.push(", fi.author) > 0.35");
                }
            }
            qb.push(")");
        }

        // Fielded search terms (title:, author:) — add ILIKE WHERE clauses
        for (field, value) in &self.params.fielded_terms {
            match field.as_str() {
                "title" => {
                    qb.push(" AND fi.title ILIKE ");
                    qb.push_bind(format!("%{}%", value));
                }
                "author" => {
                    qb.push(" AND fi.author ILIKE ");
                    qb.push_bind(format!("%{}%", value));
                }
                _ => {
                    // fandom, character, relationship are already converted to tag filters
                    // by the route handler; skip unknown fields
                }
            }
        }

        // ── v2 field expressions + explicit Guided-tier params ────────────
        // These resolved against the typed column/tag sets. Explicit params
        // are the "Guided" tier; `field_queries` carry the compact q= syntax.
        if let Some(ref status) = self.params.status {
            qb.push(" AND fi.status = ");
            qb.push_bind(status);
        }
        if let Some(ref lang) = self.params.language {
            qb.push(" AND w.language_code = ");
            qb.push_bind(lang.to_lowercase());
        }
        if let Some(ref beta) = self.params.beta_status {
            qb.push(" AND w.beta_status = ");
            qb.push_bind(beta.to_lowercase());
        }
        if self.params.main_char.is_some() {
            // Guide tier: a main/primary character (role_confidence >= 1.0).
            self.push_field_query(
                qb,
                &FieldQuery {
                    field: Field::Char,
                    value: self.params.main_char.clone(),
                    op: FieldOp::Contains,
                    role: true,
                    negated: false,
                },
            );
        }
        if let Some(cross) = self.params.crossover {
            self.push_field_query(
                qb,
                &FieldQuery {
                    field: Field::Crossover,
                    value: Some(if cross { "1".into() } else { "0".into() }),
                    op: FieldOp::Contains,
                    role: false,
                    negated: false,
                },
            );
        }
        if let Some(min) = self.params.min_hits {
            self.push_field_query(
                qb,
                &FieldQuery {
                    field: Field::Hits,
                    value: Some(min.to_string()),
                    op: FieldOp::Range(RangeExpr::Ge(min)),
                    role: false,
                    negated: false,
                },
            );
        }
        if let Some(max) = self.params.max_hits {
            self.push_field_query(
                qb,
                &FieldQuery {
                    field: Field::Hits,
                    value: Some(max.to_string()),
                    op: FieldOp::Range(RangeExpr::Le(max)),
                    role: false,
                    negated: false,
                },
            );
        }
        for fq in &self.params.field_queries {
            self.push_field_query(qb, fq);
        }

        // include_tags — ALL must match
        // Use expanded tag IDs when available (synonym-aware), fall back to name-based
        if !self.params.expanded_include_tag_ids.is_empty()
            && self.params.expanded_include_tag_ids.len() == self.params.include_tags.len()
        {
            for ids in &self.params.expanded_include_tag_ids {
                if ids.is_empty() {
                    // The tag name resolved to NO known tags — no fic can
                    // satisfy "must have this tag", so force the whole query
                    // to match nothing instead of silently dropping the AND.
                    qb.push(" AND 1=0");
                    continue;
                }
                qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
                qb.push(" WHERE ft.url_id = fi.id");
                qb.push(" AND ft.tag_id = ANY(");
                qb.push_bind(ids);
                qb.push(") AND ft.score >= ");
                qb.push_bind(self.hidden_threshold);
                qb.push(")");
            }
        } else {
            for tag in &self.params.include_tags {
                qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
                qb.push(" JOIN tags t ON t.id = ft.tag_id");
                qb.push(" WHERE ft.url_id = fi.id");
                qb.push(" AND t.name = ");
                qb.push_bind(&tag.tag_name);
                qb.push(" AND t.tag_type_id = ");
                qb.push_bind(tag.tag_type_id);
                qb.push(" AND ft.score >= ");
                qb.push_bind(self.hidden_threshold);
                qb.push(")");
            }
        }

        // exclude_tags — NONE must match
        // Use expanded tag IDs when available (synonym-aware), fall back to name-based
        if !self.params.expanded_exclude_tag_ids.is_empty()
            && self.params.expanded_exclude_tag_ids.len() == self.params.exclude_tags.len()
        {
            for ids in &self.params.expanded_exclude_tag_ids {
                if ids.is_empty() {
                    continue;
                }
                qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
                qb.push(" WHERE ft.url_id = fi.id");
                qb.push(" AND ft.tag_id = ANY(");
                qb.push_bind(ids);
                qb.push(") AND ft.score >= ");
                qb.push_bind(self.hidden_threshold);
                qb.push(")");
            }
        } else {
            for tag in &self.params.exclude_tags {
                qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
                qb.push(" JOIN tags t ON t.id = ft.tag_id");
                qb.push(" WHERE ft.url_id = fi.id");
                qb.push(" AND t.name = ");
                qb.push_bind(&tag.tag_name);
                qb.push(" AND t.tag_type_id = ");
                qb.push_bind(tag.tag_type_id);
                qb.push(" AND ft.score >= ");
                qb.push_bind(self.hidden_threshold);
                qb.push(")");
            }
        }

        // exclude_tag_types — fics carrying ANY tag of an excluded type are
        // dropped (e.g. "no ships": exclude type 3 = relationships).
        // strict_gen is shorthand for exclude_tag_types=3,6 (relationships
        // + categories). The types are merged and emitted as one NOT EXISTS.
        let mut excluded_types: Vec<i16> = self.params.exclude_tag_types.clone();
        if self.params.strict_gen {
            for t in [3i16, 6i16] {
                if !excluded_types.contains(&t) {
                    excluded_types.push(t);
                }
            }
        }
        if !excluded_types.is_empty() {
            qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
            qb.push(" JOIN tags t ON t.id = ft.tag_id");
            qb.push(" WHERE ft.url_id = fi.id AND t.tag_type_id = ANY(");
            qb.push_bind(&excluded_types);
            qb.push(") AND ft.score >= ");
            qb.push_bind(self.hidden_threshold);
            qb.push(")");
        }

        // include_any_tags — at least ONE must match
        // Use expanded tag IDs when available (synonym-aware), fall back to name-based OR
        if !self.params.expanded_include_any_tag_ids.is_empty()
            && self.params.expanded_include_any_tag_ids.len() == self.params.include_any_tags.len()
        {
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
            qb.push(" WHERE ft.url_id = fi.id");
            qb.push(" AND ft.score >= ");
            qb.push_bind(self.hidden_threshold);
            qb.push(" AND (");
            let mut first = true;
            for ids in &self.params.expanded_include_any_tag_ids {
                if ids.is_empty() {
                    continue;
                }
                if !first {
                    qb.push(" OR");
                }
                first = false;
                qb.push(" ft.tag_id = ANY(");
                qb.push_bind(ids);
                qb.push(")");
            }
            qb.push("))");
        } else if !self.params.include_any_tags.is_empty() {
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
            qb.push(" JOIN tags t ON t.id = ft.tag_id");
            qb.push(" WHERE ft.url_id = fi.id");
            qb.push(" AND ft.score >= ");
            qb.push_bind(self.hidden_threshold);
            qb.push(" AND (");
            let mut first = true;
            for tag in &self.params.include_any_tags {
                if !first {
                    qb.push(" OR");
                }
                first = false;
                qb.push(" (t.name = ");
                qb.push_bind(&tag.tag_name);
                qb.push(" AND t.tag_type_id = ");
                qb.push_bind(tag.tag_type_id);
                qb.push(")");
            }
            qb.push("))");
        }

        // min_words / max_words
        if let Some(min) = self.params.min_words {
            qb.push(" AND fi.words >= ");
            qb.push_bind(min);
        }
        if let Some(max) = self.params.max_words {
            qb.push(" AND fi.words <= ");
            qb.push_bind(max);
        }

        // min_chapters / max_chapters
        if let Some(min) = self.params.min_chapters {
            qb.push(" AND fi.chapters >= ");
            qb.push_bind(min);
        }
        if let Some(max) = self.params.max_chapters {
            qb.push(" AND fi.chapters <= ");
            qb.push_bind(max);
        }

        // complete -> status = 'complete'
        if let Some(true) = self.params.complete {
            qb.push(" AND fi.status = 'complete'");
        }
        if let Some(false) = self.params.complete {
            qb.push(" AND fi.status != 'complete'");
        }

        // source filter
        if let Some(ref source) = self.params.source {
            qb.push(" AND fi.source = ");
            qb.push_bind(source);
        }

        // date_from / date_to on fic_updated
        if let Some(ref dt) = self.params.date_from {
            qb.push(" AND fi.fic_updated >= ");
            qb.push_bind(dt);
        }
        if let Some(ref dt) = self.params.date_to {
            qb.push(" AND fi.fic_updated <= ");
            qb.push_bind(dt);
        }

        // primary_tag — fic must have this tag as its HIGHEST-scored fic_tag
        // (the tag with the max score among all the fic's tags must BE this
        // tag).
        if let Some(ref tag) = self.params.primary_tag {
            qb.push(" AND (SELECT ft2.tag_id FROM fic_tags ft2");
            qb.push(" WHERE ft2.url_id = fi.id");
            qb.push(" ORDER BY ft2.score DESC LIMIT 1) IN (");
            qb.push("SELECT t3.id FROM tags t3 WHERE t3.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(" AND t3.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(")");
        }

        // min_comments — only if comments table exists
        if let Some(min) = self.params.min_comments {
            if self.has_comments {
                qb.push(" AND (SELECT COUNT(*) FROM comments c WHERE c.url_id = fi.id) >= ");
                qb.push_bind(min);
            }
        }

        // min_kudos — only if kudos table exists. Real kudos count: signed-in
        // kudos only (user_id IS NOT NULL); guest kudos are excluded so the
        // filter is not gameable by anonymous taps.
        if let Some(min) = self.params.min_kudos {
            if self.has_ratings {
                qb.push(" AND (SELECT COUNT(*) FROM kudos k WHERE k.work_id = fi.work_id AND k.user_id IS NOT NULL) >= ");
                qb.push_bind(min);
            }
        }

        // max_kudos — same subquery, upper bound.
        if let Some(max) = self.params.max_kudos {
            if self.has_ratings {
                qb.push(" AND (SELECT COUNT(*) FROM kudos k WHERE k.work_id = fi.work_id AND k.user_id IS NOT NULL) <= ");
                qb.push_bind(max);
            }
        }

        // min_bookmarks / max_bookmarks — only if bookmarks table exists.
        if let Some(min) = self.params.min_bookmarks {
            if self.has_bookmarks {
                qb.push(" AND (SELECT COUNT(*) FROM bookmarks b WHERE b.url_id = fi.id) >= ");
                qb.push_bind(min);
            }
        }
        if let Some(max) = self.params.max_bookmarks {
            if self.has_bookmarks {
                qb.push(" AND (SELECT COUNT(*) FROM bookmarks b WHERE b.url_id = fi.id) <= ");
                qb.push_bind(max);
            }
        }

        // no_warnings — exclude works with Archive Warning tags (tag_type_id = 5)
        if self.params.no_warnings == Some(true) {
            qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
            qb.push(" JOIN tags t ON t.id = ft.tag_id");
            qb.push(" WHERE ft.url_id = fi.id AND t.tag_type_id = 5");
            qb.push(" AND ft.score >= ");
            qb.push_bind(self.hidden_threshold);
            qb.push(")");
        }

        // tag_ids — search by numeric tag IDs
        if !self.params.tag_ids.is_empty() {
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
            qb.push(" WHERE ft.url_id = fi.id AND ft.tag_id = ANY(");
            qb.push_bind(&self.params.tag_ids);
            qb.push("))");
        }

        // ── Personal filters (hide read / hide bookmarked / library-only) ──
        // All three are per-user filters keyed on the signing user's
        // reading_stats + bookmarks rows. They are no-ops for anonymous
        // searches (user_id 0), so the public search surface is unchanged.
        // library_only wins over hide_bookmarked when both are set (they are
        // contradictory), matching the "My library" semantics.
        if self.params.hide_read || self.params.hide_bookmarked || self.params.library_only {
            let user_id: i32 = self.params.user_id.unwrap_or(0);
            if user_id > 0 {
                if self.params.library_only {
                    qb.push(" AND EXISTS (SELECT 1 FROM bookmarks b");
                    qb.push(" WHERE b.user_id = ");
                    qb.push_bind(user_id);
                    qb.push(" AND b.url_id = fi.id)");
                } else {
                    if self.params.hide_bookmarked {
                        qb.push(" AND NOT EXISTS (SELECT 1 FROM bookmarks b");
                        qb.push(" WHERE b.user_id = ");
                        qb.push_bind(user_id);
                        qb.push(" AND b.url_id = fi.id)");
                    }
                    if self.params.hide_read {
                        qb.push(" AND NOT EXISTS (SELECT 1 FROM reading_stats rs");
                        qb.push(" WHERE rs.user_id = ");
                        qb.push_bind(user_id);
                        qb.push(" AND rs.work_id = fi.work_id AND rs.status = 'completed')");
                    }
                }
            }
        }
    }

    // ─── v2 field-expression helpers ─────────────────────────────────────

    /// Whether any active filter needs a `works` join (counters / language /
    /// beta live on the `works` table). Kept conservative so the default
    /// search surface emits no join.
    fn needs_works_join(&self) -> bool {
        self.params
            .field_queries
            .iter()
            .any(|q| q.field.needs_works())
            || self.params.beta_status.is_some()
            || self.params.language.is_some()
            || self.params.min_hits.is_some()
            || self.params.max_hits.is_some()
    }

    /// Append a `LEFT JOIN works w` (idempotent — callers invoke it exactly
    /// once on the base FROM) when a works-backed filter is present.
    fn push_works_join(&self, qb: &mut QueryBuilder<Postgres>) {
        if self.needs_works_join() {
            qb.push(" LEFT JOIN works w ON w.id = fi.work_id");
        }
    }

    /// Emit `AND <col> <op> <bound>` for a numeric comparator.
    fn push_cmp(&self, qb: &mut QueryBuilder<Postgres>, col: &'static str, op: &str, val: i64) {
        qb.push(" AND ");
        qb.push(col);
        qb.push(" ");
        qb.push(op);
        qb.push(" ");
        qb.push_bind(val);
    }

    /// Emit the WHERE clause for a numeric / date interval, honouring an
    /// (inverted) negation — `-words:>100` becomes `AND words <= 100`.
    fn push_range(
        &self,
        qb: &mut QueryBuilder<Postgres>,
        col: &'static str,
        expr: &RangeExpr,
        negated: bool,
    ) {
        match expr {
            RangeExpr::Eq(v) => self.push_cmp(qb, col, if negated { "<>" } else { "=" }, *v),
            RangeExpr::Ge(v) => self.push_cmp(qb, col, if negated { "<" } else { ">=" }, *v),
            RangeExpr::Gt(v) => self.push_cmp(qb, col, if negated { "<=" } else { ">" }, *v),
            RangeExpr::Le(v) => self.push_cmp(qb, col, if negated { ">" } else { "<=" }, *v),
            RangeExpr::Lt(v) => self.push_cmp(qb, col, if negated { ">=" } else { "<" }, *v),
            RangeExpr::Between(a, b) => {
                if negated {
                    qb.push(" AND (");
                    qb.push(col);
                    qb.push(" < ");
                    qb.push_bind(*a);
                    qb.push(" OR ");
                    qb.push(col);
                    qb.push(" > ");
                    qb.push_bind(*b);
                    qb.push(")");
                } else {
                    qb.push(" AND ");
                    qb.push(col);
                    qb.push(" >= ");
                    qb.push_bind(*a);
                    qb.push(" AND ");
                    qb.push(col);
                    qb.push(" <= ");
                    qb.push_bind(*b);
                }
            }
        }
    }

    /// ILIKE pattern for a text/tag field value given its operator.
    fn pattern_for_op(&self, op: &FieldOp, value: &str) -> String {
        match op {
            FieldOp::Contains => format!("%{}%", value),
            FieldOp::Prefix => format!("{}%", value),
            FieldOp::ContainsWide => format!("%{}", value),
            FieldOp::Range(_) => value.to_string(),
        }
    }

    /// SQL (aliased) column for a text field.
    fn text_column(&self, field: &Field) -> &'static str {
        match field {
            Field::Title => "fi.title",
            Field::Author => "fi.author",
            Field::Description => "fi.description",
            Field::Status => "fi.status",
            Field::Source => "fi.source",
            Field::Language => "w.language_code",
            Field::Beta => "w.beta_status",
            _ => "fi.title",
        }
    }

    /// SQL (aliased) expression for a numeric / date field.
    fn numeric_column(&self, field: &Field) -> &'static str {
        match field {
            Field::Words => "fi.words",
            Field::Chapters => "fi.chapters",
            Field::Kudos => "w.kudos_count",
            Field::Comments => "w.comments_count",
            Field::Bookmarks => "w.bookmarks_count",
            Field::Hits => "w.hit_count",
            Field::Published => "EXTRACT(YEAR FROM fi.fic_created)::int",
            Field::Updated => "EXTRACT(YEAR FROM fi.fic_updated)::int",
            _ => "fi.words",
        }
    }

    /// Emit the WHERE clause for a `crossover` (fandom count > 1) filter.
    fn push_crossover(&self, qb: &mut QueryBuilder<Postgres>, fq: &FieldQuery) {
        // value "0" disables (single-fandom wanted); negation flips the count.
        let on = !matches!(fq.value.as_deref(), Some("0"));
        let want_gt1 = on != fq.negated;
        qb.push(" AND (SELECT COUNT(DISTINCT t2.tag_id) FROM fic_tags t2");
        qb.push(" JOIN tags tt2 ON tt2.id = t2.tag_id");
        qb.push(" WHERE t2.url_id = fi.id AND tt2.tag_type_id = 1");
        qb.push(" AND t2.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(")");
        if want_gt1 {
            qb.push(" > 1");
        } else {
            qb.push(" <= 1");
        }
    }

    /// Emit the WHERE clause for a single structured v2 field expression.
    pub(crate) fn push_field_query(&self, qb: &mut QueryBuilder<Postgres>, fq: &FieldQuery) {
        let FieldQuery {
            field,
            value,
            op,
            role,
            negated,
        } = fq;
        if *field == Field::Crossover {
            self.push_crossover(qb, fq);
            return;
        }
        let v = value.as_deref().unwrap_or("");

        if field.is_tag() {
            let Some(type_id) = field.tag_type_id() else {
                return;
            };
            let pattern = self.pattern_for_op(op, v);
            if *negated {
                qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
            } else {
                qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
            }
            qb.push(" JOIN tags t ON t.id = ft.tag_id");
            qb.push(" WHERE ft.url_id = fi.id");
            qb.push(" AND t.tag_type_id = ");
            qb.push_bind(type_id);
            qb.push(" AND t.name ILIKE ");
            qb.push_bind(pattern);
            qb.push(" AND ft.score >= ");
            qb.push_bind(self.hidden_threshold);
            if let Some(pol) = field.rel_polarity() {
                qb.push(" AND t.rel_polarity = ");
                qb.push_bind(pol);
            }
            if *role {
                // `@`-prefixed tag field: require the tag to be main/primary.
                qb.push(" AND ft.role_confidence >= ");
                qb.push_bind(ROLE_MAIN_CONFIDENCE);
            }
            qb.push(")");
            return;
        }

        if field.is_text() {
            let col = self.text_column(field);
            let pattern = self.pattern_for_op(op, v);
            if *negated {
                qb.push(" AND NOT (");
            } else {
                qb.push(" AND (");
            }
            qb.push(col);
            qb.push(" ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
            return;
        }

        if field.is_count() || field.is_date() {
            let col = self.numeric_column(field);
            if let FieldOp::Range(expr) = op {
                self.push_range(qb, col, expr, *negated);
            }
        }
    }
}

/// A tag row returned from the database for building the response.
#[derive(Debug, Clone)]
pub struct FicTagRow {
    pub url_id: String,
    pub name: String,
    pub tag_type_id: i16,
    pub score: i16,
}

/// Parse a comma-separated "type_id:name" list into TagFilter vec.
/// e.g. "1:Harry Potter,2:Hermione Granger"
pub fn parse_tag_filters(input: &str) -> Result<Vec<TagFilter>, String> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let mut filters = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let colon_pos = part.find(':').ok_or_else(|| {
            format!(
                "Invalid tag filter format '{}': expected 'type_id:name'",
                part
            )
        })?;
        let type_id: i16 = part[..colon_pos]
            .parse()
            .map_err(|e| format!("Invalid tag type_id in '{}': {}", part, e))?;
        let name = part[colon_pos + 1..].to_string();
        filters.push(TagFilter {
            tag_type_id: type_id,
            tag_name: name,
        });
    }
    Ok(filters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_filters_empty() {
        let result = parse_tag_filters("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_tag_filters_single() {
        let result = parse_tag_filters("1:Harry Potter").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tag_type_id, 1);
        assert_eq!(result[0].tag_name, "Harry Potter");
    }

    #[test]
    fn test_parse_tag_filters_multiple() {
        let result = parse_tag_filters("1:Harry Potter,2:Hermione Granger,4:Angst").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].tag_type_id, 1);
        assert_eq!(result[0].tag_name, "Harry Potter");
        assert_eq!(result[1].tag_type_id, 2);
        assert_eq!(result[1].tag_name, "Hermione Granger");
        assert_eq!(result[2].tag_type_id, 4);
        assert_eq!(result[2].tag_name, "Angst");
    }

    #[test]
    fn test_parse_tag_filters_invalid_format() {
        let result = parse_tag_filters("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tag_filters_invalid_type_id() {
        let result = parse_tag_filters("abc:test");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tag_filters_trailing_comma() {
        let result = parse_tag_filters("1:test,").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tag_type_id, 1);
        assert_eq!(result[0].tag_name, "test");
    }

    #[test]
    fn test_parse_tag_filters_whitespace() {
        let result = parse_tag_filters(" 1:Harry , 2:Potter ").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tag_type_id, 1);
        assert_eq!(result[0].tag_name, "Harry");
        assert_eq!(result[1].tag_type_id, 2);
        assert_eq!(result[1].tag_name, "Potter");
    }

    #[test]
    fn test_parse_tag_filters_multiple_commas() {
        let result = parse_tag_filters(",,1:test,,2:foo,").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tag_type_id, 1);
        assert_eq!(result[0].tag_name, "test");
        assert_eq!(result[1].tag_type_id, 2);
        assert_eq!(result[1].tag_name, "foo");
    }

    #[test]
    fn test_search_result_from_row() {
        let row = FicSearchRow {
            id: "abc123".into(),
            work_id: Some(42),
            created: None,
            updated: None,
            title: "Test Fic".into(),
            author: "Author".into(),
            author_url: None,
            author_local_id: None,
            chapters: 10,
            words: 50000,
            description: "A test".into(),
            fic_created: Utc::now(),
            fic_updated: Utc::now(),
            status: "complete".into(),
            source: "ao3".into(),
            extra_meta: None,
            raw_extended_meta: None,
            source_id: None,
            author_id: None,
            content_hash: None,
            rank: Some(0.5),
            snippet: None,
        };
        let result: SearchResult = row.into();
        assert_eq!(result.url_id, "abc123");
        assert_eq!(result.work_id, Some(42));
        assert_eq!(result.title, "Test Fic");
        assert_eq!(result.rank, Some(0.5));
        assert_eq!(result.snippet, None);
    }

    // --- build_count_query tests ---

    #[test]
    fn test_build_count_query_basic() {
        let params = SearchParams::default();
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("SELECT COUNT(*) FROM fic_info fi WHERE 1=1"));
    }

    #[test]
    fn test_build_count_query_with_q() {
        let params = SearchParams {
            q: Some("test".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("websearch_to_tsquery"));
    }

    #[test]
    fn test_build_count_query_with_fuzzy_fallback() {
        // fuzzy=true should add ILIKE fallback terms for words >= 3 chars
        let params = SearchParams {
            q: Some("hary pottr".into()),
            fuzzy: true,
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        // 2 words -> 4 word_similarity clauses (title+author per word)
        let ws_count = sql.matches("word_similarity").count();
        assert_eq!(
            ws_count, 4,
            "2 words x (title+author) = 4 word_similarity: {sql}"
        );
        // The fallback lives inside the same AND group with OR so exact
        // full-text matches still rank higher under rank DESC.
        assert!(
            sql.contains("(fi.text_search @@ websearch_to_tsquery"),
            "OR group opens: {sql}"
        );
        // Exact full-text clause still present (websearch, not plainto)
        assert!(
            sql.contains("websearch_to_tsquery"),
            "exact clause preserved: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_uses_websearch_to_tsquery() {
        // Native boolean search: the FTS predicate must be websearch_to_tsquery
        // so users get 'OR', '-exclusion', '"quoted phrases"' and implicit AND
        // straight from PostgreSQL instead of the hand-rolled parser tsquery.
        let params = SearchParams {
            q: Some("\"exact phrase\" harry OR ron -draco".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("fi.text_search @@ websearch_to_tsquery('english', "),
            "FTS predicate must use websearch_to_tsquery: {sql}"
        );
        assert!(
            !sql.contains("plainto_tsquery"),
            "no plainto_tsquery: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_without_fuzzy_has_no_ilike() {
        // Default (fuzzy=false) must NOT add ILIKE fallback for a bare q
        let params = SearchParams {
            q: Some("harry potter".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("ILIKE"),
            "no fuzzy -> no ILIKE fallback: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_fuzzy_with_parsed_tsquery() {
        // The runtime path for bare text queries sets parsed_tsquery (always
        // non-empty for a text q), so the fuzzy fallback must be emitted in
        // that branch too — this is the path that was broken (typo queries
        // matched nothing because the tsquery branch skipped the OR fallback).
        let params = SearchParams {
            q: Some("hary pottr".into()),
            parsed_tsquery: Some("'hary' & 'pottr'".into()),
            fuzzy: true,
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        // 2 words -> 4 word_similarity clauses in the parsed_tsquery branch
        let ws_count = sql.matches("word_similarity").count();
        assert_eq!(
            ws_count, 4,
            "fuzzy + parsed_tsquery: 4 word_similarity: {sql}"
        );
        // parsed_tsquery comes from the boolean parser (expr_to_tsquery),
        // which emits to_tsquery-native syntax — so the parsed branch uses
        // to_tsquery, NOT websearch_to_tsquery (websearch is for raw text).
        assert!(
            sql.contains("to_tsquery"),
            "parsed path uses to_tsquery: {sql}"
        );
        // Fuzzy ORs are still present alongside the exact tsquery
        assert!(
            sql.contains("fi.text_search @@ to_tsquery"),
            "exact clause preserved: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_with_include_tags() {
        let params = SearchParams {
            include_tags: vec![TagFilter {
                tag_type_id: 1,
                tag_name: "Harry".into(),
            }],
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("EXISTS (SELECT 1 FROM fic_tags"));
    }

    #[test]
    fn test_build_count_query_with_all_filters() {
        let params = SearchParams {
            include_tags: vec![TagFilter {
                tag_type_id: 1,
                tag_name: "Harry".into(),
            }],
            exclude_tags: vec![TagFilter {
                tag_type_id: 2,
                tag_name: "Voldemort".into(),
            }],
            include_any_tags: vec![TagFilter {
                tag_type_id: 3,
                tag_name: "Angst".into(),
            }],
            min_words: Some(1000),
            max_words: Some(100000),
            complete: Some(true),
            source: Some("ao3".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("EXISTS (SELECT 1 FROM fic_tags"),
            "should contain include_tags clause"
        );
        assert!(
            sql.contains("NOT EXISTS (SELECT 1 FROM fic_tags"),
            "should contain exclude_tags clause"
        );
        assert!(
            sql.contains("AND (") || sql.contains("OR"),
            "should contain include_any_tags clause"
        );
        assert!(sql.contains("fi.words >="), "should contain min_words");
        assert!(sql.contains("fi.words <="), "should contain max_words");
        assert!(
            sql.contains("fi.status = 'complete'"),
            "should contain complete filter"
        );
        assert!(sql.contains("fi.source ="), "should contain source filter");
    }

    // --- build_data_query tests ---

    #[test]
    fn test_build_data_query_no_q() {
        let params = SearchParams::default();
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("NULL::real AS rank"));
        assert!(sql.contains("fi.fic_updated DESC"));
        assert!(sql.contains("LIMIT"));
        assert!(sql.contains("OFFSET"));
    }

    #[test]
    fn test_build_data_query_with_q() {
        let params = SearchParams {
            q: Some("test".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("ts_rank"));
        assert!(sql.contains("websearch_to_tsquery"));
        // ts_headline snippet must be selected alongside rank
        assert!(
            sql.contains("ts_headline('english', fi.description, websearch_to_tsquery('english', ")
        );
        assert!(sql.contains("'StartSel=<b>, StopSel=</b>, MaxWords=35, MinWords=15') AS snippet"));
    }

    #[test]
    fn test_build_data_query_sort_relevance() {
        let params = SearchParams {
            q: Some("test".into()),
            sort: Some("-relevance".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("rank DESC"));
    }

    #[test]
    fn test_build_data_query_sort_date() {
        let params = SearchParams {
            sort: Some("-date".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.fic_updated DESC"));
    }

    #[test]
    fn test_build_data_query_sort_words() {
        let params = SearchParams {
            sort: Some("-words".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.words DESC"));
    }

    #[test]
    fn test_build_data_query_sort_chapters() {
        let params = SearchParams {
            sort: Some("-chapters".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.chapters DESC"));
    }

    #[test]
    fn test_build_data_query_sort_title() {
        let params = SearchParams {
            sort: Some("-title".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.title ASC"));
    }

    #[test]
    fn test_build_data_query_sort_fallback() {
        let params = SearchParams {
            sort: Some("invalid".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("fi.fic_updated DESC"),
            "invalid sort should fall back to default"
        );
    }

    // --- page() tests ---

    #[test]
    fn test_page_default() {
        let params = SearchParams::default();
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        assert_eq!(qb.page(), 1);
    }

    #[test]
    fn test_page_custom() {
        let params = SearchParams {
            page: Some(3),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        assert_eq!(qb.page(), 3);
    }

    // --- Additional filter tests ---

    #[test]
    fn test_build_count_query_min_max_chapters() {
        let params = SearchParams {
            min_chapters: Some(5),
            max_chapters: Some(50),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("fi.chapters >="),
            "should contain min_chapters"
        );
        assert!(
            sql.contains("fi.chapters <="),
            "should contain max_chapters"
        );
    }

    #[test]
    fn test_build_count_query_date_filters() {
        let params = SearchParams {
            date_from: Some(Utc::now()),
            date_to: Some(Utc::now()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("fi.fic_updated >="),
            "should contain date_from"
        );
        assert!(sql.contains("fi.fic_updated <="), "should contain date_to");
    }

    #[test]
    fn test_build_count_query_primary_tag() {
        let params = SearchParams {
            primary_tag: Some(TagFilter {
                tag_type_id: 1,
                tag_name: "Harry Potter".into(),
            }),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("ORDER BY ft2.score DESC LIMIT 1"),
            "should have primary_tag subquery"
        );
    }

    #[test]
    fn test_build_count_query_min_comments_with_table() {
        let params = SearchParams {
            min_comments: Some(5),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM comments"),
            "should have comments subquery"
        );
    }

    #[test]
    fn test_build_count_query_min_comments_without_table() {
        let params = SearchParams {
            min_comments: Some(5),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, false, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("SELECT COUNT(*) FROM comments"),
            "should NOT have comments subquery"
        );
    }

    #[test]
    fn test_build_count_query_min_kudos_with_table() {
        let params = SearchParams {
            min_kudos: Some(10),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM kudos"),
            "should use the real kudos subquery: {sql}"
        );
        assert!(
            sql.contains("k.user_id IS NOT NULL"),
            "guest kudos must not count toward min_kudos: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_min_kudos_without_table() {
        let params = SearchParams {
            min_kudos: Some(10),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, false, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("SELECT COUNT(*) FROM kudos"),
            "should NOT have kudos subquery: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_no_warnings() {
        let params = SearchParams {
            no_warnings: Some(true),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("tag_type_id = 5"),
            "should filter archive warnings"
        );
        assert!(sql.contains("NOT EXISTS"), "should be NOT EXISTS");
    }

    #[test]
    fn test_build_count_query_exclude_tag_types() {
        let params = SearchParams {
            exclude_tag_types: vec![3],
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("NOT EXISTS (SELECT 1 FROM fic_tags"),
            "should be NOT EXISTS"
        );
        assert!(
            sql.contains("t.tag_type_id = ANY("),
            "should exclude by type: {sql}"
        );
        // The bound value is an array parameter ($1); the type ids ride in the
        // bind, so assert the placeholder form rather than literal digits.
        assert!(sql.contains("ANY($1)"), "should bind the type array: {sql}");
        // Without strict_gen the query must NOT mention type 6 (as a literal).
        assert!(
            !sql.contains("tag_type_id = 6"),
            "should NOT exclude type 6 by default: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_exclude_multiple_tag_types() {
        let params = SearchParams {
            exclude_tag_types: vec![3, 6],
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("t.tag_type_id = ANY("),
            "should exclude by type: {sql}"
        );
        assert!(sql.contains("ANY($1)"), "should bind the type array: {sql}");
    }

    #[test]
    fn test_build_count_query_strict_gen() {
        // strict_gen is shorthand for exclude_tag_types=3,6 — the builder
        // must emit a single NOT EXISTS over both relationship and category
        // tags, matching the merged-type semantics of exclude_tag_types.
        let params = SearchParams {
            strict_gen: true,
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("NOT EXISTS (SELECT 1 FROM fic_tags"),
            "should be NOT EXISTS"
        );
        assert!(
            sql.contains("t.tag_type_id = ANY("),
            "should exclude by type: {sql}"
        );
        assert!(
            sql.contains("ANY($1)"),
            "strict_gen binds both types: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_exclude_tag_types_default_empty() {
        // Default params must emit NO exclusion clause — existing search
        // behavior is unchanged.
        let params = SearchParams::default();
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("exclude_tag_types"),
            "no exclusion clause: {sql}"
        );
        // The only NOT EXISTS here would come from exclude_tags (none set).
        assert!(
            !sql.contains("NOT EXISTS"),
            "default has no NOT EXISTS: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_tag_ids() {
        let params = SearchParams {
            tag_ids: vec![1, 2, 3],
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("ft.tag_id = ANY("),
            "should have tag_ids filter"
        );
    }

    #[test]
    fn test_build_count_query_complete_false() {
        let params = SearchParams {
            complete: Some(false),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("fi.status != 'complete'"),
            "should filter incomplete fics"
        );
    }

    #[test]
    fn test_build_data_query_pagination() {
        let params = SearchParams {
            page: Some(3),
            per_page: Some(10),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("LIMIT"), "should contain LIMIT");
        assert!(sql.contains("OFFSET"), "should contain OFFSET");
        assert!(
            sql.contains("LIMIT $"),
            "LIMIT must be a bind, not a literal"
        );
    }

    #[test]
    fn test_build_data_query_sort_kudos_uses_real_kudos() {
        let params = SearchParams {
            sort: Some("kudos".to_string()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM kudos"),
            "kudos sort must use the real kudos table: {sql}"
        );
        assert!(
            sql.contains("k.user_id IS NOT NULL"),
            "guest kudos must not count toward the kudos sort: {sql}"
        );
    }

    #[test]
    fn test_build_data_query_sort_kudos_without_table_falls_back() {
        let params = SearchParams {
            sort: Some("kudos".to_string()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, false, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("SELECT COUNT(*) FROM kudos"),
            "kudos sort must fall back when the table is missing: {sql}"
        );
        assert!(sql.contains("fi.fic_updated DESC"), "fallback sort: {sql}");
    }

    #[test]
    fn test_hidden_threshold_used() {
        let params = SearchParams {
            include_tags: vec![TagFilter {
                tag_type_id: 1,
                tag_name: "Test".into(),
            }],
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 50, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("ft.score >="), "should use hidden_threshold");
    }

    // --- AO3-form parity: max_kudos, bookmarks bounds, rating, language ---

    #[test]
    fn test_build_count_query_max_kudos_with_table() {
        let params = SearchParams {
            max_kudos: Some(50),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM kudos"),
            "max_kudos should use kudos subquery: {sql}"
        );
        assert!(
            sql.contains("k.user_id IS NOT NULL"),
            "max_kudos must filter guest kudos: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_max_kudos_without_table() {
        let params = SearchParams {
            max_kudos: Some(50),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, false, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("SELECT COUNT(*) FROM kudos"),
            "max_kudos must NOT emit subquery when table absent: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_min_bookmarks_with_table() {
        let params = SearchParams {
            min_bookmarks: Some(5),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM bookmarks b WHERE b.url_id = fi.id) >="),
            "min_bookmarks should emit bookmarks subquery with >=: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_min_bookmarks_without_table() {
        let params = SearchParams {
            min_bookmarks: Some(5),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, false);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("bookmarks"),
            "min_bookmarks must NOT emit subquery when table absent: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_max_bookmarks_with_table() {
        let params = SearchParams {
            max_bookmarks: Some(100),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("SELECT COUNT(*) FROM bookmarks b WHERE b.url_id = fi.id) <="),
            "max_bookmarks should emit bookmarks subquery with <=: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_max_bookmarks_without_table() {
        let params = SearchParams {
            max_bookmarks: Some(100),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, false);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("bookmarks"),
            "max_bookmarks must NOT emit subquery when table absent: {sql}"
        );
    }

    #[test]
    fn test_build_count_query_min_and_max_bookmarks_together() {
        let params = SearchParams {
            min_bookmarks: Some(5),
            max_bookmarks: Some(100),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains(") >="), "min_bookmarks >= clause: {sql}");
        assert!(sql.contains(") <="), "max_bookmarks <= clause: {sql}");
    }

    #[test]
    fn test_build_count_query_language_maps_to_works_column() {
        // v2: language now maps to `works.language_code` (migration 004),
        // which requires the on-demand works join.
        let params = SearchParams {
            language: Some("en".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("LEFT JOIN works w"));
        assert!(sql.contains("w.language_code = $1"));
        // The value is lower-cased for a stable match against language codes.
        assert!(sql.contains("w.language_code = $1"));
    }

    // ─── v2 field-expression builder tests ───────────────────────────────

    /// Parse one structured field query of the given field from a compact
    /// query string (mirrors what the route handler extracts from q=).
    fn one_fq(query: &str, field: Field) -> FieldQuery {
        use crate::search::parser::{extract_field_queries, parse_query};
        extract_field_queries(&parse_query(query))
            .into_iter()
            .find(|q| q.field == field)
            .unwrap_or_else(|| panic!("no {:?} in '{query}'", field))
    }

    #[test]
    fn test_build_field_query_char_tag_exists() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("char:Harry", Field::Char)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("EXISTS (SELECT 1 FROM fic_tags ft"), "{sql}");
        assert!(sql.contains("t.tag_type_id = $"), "{sql}");
        assert!(sql.contains("t.name ILIKE $"), "{sql}");
    }

    #[test]
    fn test_build_field_query_at_char_role_confidence() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("@char:Harry", Field::Char)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("ft.role_confidence >= $"),
            "role threshold: {sql}"
        );
    }

    #[test]
    fn test_build_field_query_romship_rel_polarity() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("romship:Harry/Ginny", Field::Romship)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("t.rel_polarity = $"),
            "romship polarity: {sql}"
        );
    }

    #[test]
    fn test_build_field_query_platship_rel_polarity() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("platship:Harry&Ron", Field::Platship)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("t.rel_polarity = $"), "{sql}");
    }

    #[test]
    fn test_build_field_query_words_gt() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("words:>50k", Field::Words)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.words > $"), "{sql}");
    }

    #[test]
    fn test_build_field_query_words_range() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("words:10k-100k", Field::Words)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.words >= $"), "range lower: {sql}");
        assert!(sql.contains("fi.words <= $"), "range upper: {sql}");
    }

    #[test]
    fn test_build_field_query_words_negated_inverts() {
        // -words:>100 => AND words <= 100
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("-words:>100", Field::Words)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.words <= $"), "negated inversion: {sql}");
    }

    #[test]
    fn test_build_field_query_crossover_gt1() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("crossover:1", Field::Crossover)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("COUNT(DISTINCT t2.tag_id)"), "{sql}");
        assert!(sql.contains(") > 1"), "crossover wants >1 fandoms: {sql}");
    }

    #[test]
    fn test_build_field_query_kudos_uses_works_join() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("kudos:>=100", Field::Kudos)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("LEFT JOIN works w ON w.id = fi.work_id"),
            "{sql}"
        );
        assert!(sql.contains("w.kudos_count >= $"), "{sql}");
    }

    #[test]
    fn test_build_field_query_language_text() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("language:en", Field::Language)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("LEFT JOIN works w"), "{sql}");
        assert!(sql.contains("w.language_code ILIKE $"), "{sql}");
    }

    #[test]
    fn test_build_field_query_negated_text() {
        // description is a v2 field; `-description:harry` inverts the ILIKE.
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("-description:harry", Field::Description)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("AND NOT (fi.description ILIKE $"), "{sql}");
    }

    #[test]
    fn test_build_no_works_join_by_default() {
        let qb = SearchQueryBuilder::new(SearchParams::default(), 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            !sql.contains("LEFT JOIN works"),
            "default query has no works join: {sql}"
        );
    }

    #[test]
    fn test_build_default_uses_no_field_queries() {
        let qb = SearchQueryBuilder::new(SearchParams::default(), 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(!sql.contains("role_confidence"), "{sql}");
    }

    #[test]
    fn test_build_explicit_status_main_char_crossover() {
        let mut params = SearchParams::default();
        params.status = Some("complete".into());
        params.main_char = Some("Harry".into());
        params.crossover = Some(true);
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("fi.status = $"), "status: {sql}");
        assert!(
            sql.contains("EXISTS (SELECT 1 FROM fic_tags ft"),
            "main_char: {sql}"
        );
        assert!(sql.contains("ft.role_confidence >= $"), "main role: {sql}");
        assert!(sql.contains(") > 1"), "crossover: {sql}");
    }

    #[test]
    fn test_build_explicit_beta_and_min_max_hits() {
        let mut params = SearchParams::default();
        params.beta_status = Some("beta".into());
        params.min_hits = Some(100);
        params.max_hits = Some(1000);
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("LEFT JOIN works w"), "{sql}");
        assert!(sql.contains("w.beta_status = $"), "{sql}");
        assert!(sql.contains("w.hit_count >= $"), "{sql}");
        assert!(sql.contains("w.hit_count <= $"), "{sql}");
    }

    #[test]
    fn test_build_relevance_composes_kudos_when_works_present() {
        let mut params = SearchParams {
            q: Some("harry ginny".into()),
            sort: Some("-relevance".into()),
            ..Default::default()
        };
        // Any works-backed filter turns on the join AND the composite ranking.
        params.field_queries = vec![one_fq("kudos:>=50", Field::Kudos)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("COALESCE(w.kudos_count, 0) DESC"),
            "composite ranking: {sql}"
        );
    }

    #[test]
    fn test_build_relevance_keeps_legacy_order_without_works() {
        let params = SearchParams {
            q: Some("harry ginny".into()),
            sort: Some("-relevance".into()),
            ..Default::default()
        };
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_data_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("rank DESC, fi.fic_updated DESC"),
            "legacy relevance: {sql}"
        );
        assert!(!sql.contains("COALESCE(w."), "{sql}");
    }

    #[test]
    fn test_build_field_query_published_year_range() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("published:2018-2021", Field::Published)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(
            sql.contains("EXTRACT(YEAR FROM fi.fic_created)::int >= $"),
            "{sql}"
        );
        assert!(
            sql.contains("EXTRACT(YEAR FROM fi.fic_created)::int <= $"),
            "{sql}"
        );
    }

    #[test]
    fn test_build_field_query_primary_fandom_role() {
        let mut params = SearchParams::default();
        params.field_queries = vec![one_fq("primary_fandom:harry", Field::PrimaryFandom)];
        let qb = SearchQueryBuilder::new(params, 100, true, true, true);
        let sql = qb.build_count_query().into_sql().as_ref().to_owned();
        assert!(sql.contains("t.tag_type_id = $"), "fandom type {sql}");
        assert!(
            sql.contains("ft.role_confidence >= $"),
            "primary role {sql}"
        );
    }
}

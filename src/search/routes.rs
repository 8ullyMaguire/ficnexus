use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::search::builder::{
    FicSearchRow, SearchParams, SearchQueryBuilder, TagFilter, parse_tag_filters,
};
use crate::search::parser::{
    extract_excluded_terms, extract_field_queries, extract_fielded_terms, extract_text_tsquery,
    parse_query,
};
use crate::search::tags::get_matching_tag_ids;
use crate::server::AppState;

/// Raw query parameters from the HTTP request (stringly typed).
#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    /// Comma-separated "type_id:name" pairs
    pub include_tags: Option<String>,
    /// Comma-separated "type_id:name" pairs
    pub exclude_tags: Option<String>,
    /// Comma-separated tag TYPE ids to exclude entirely (e.g. "3" hides all
    /// relationship tags; "3,6" also hides category tags).
    pub exclude_tag_types: Option<String>,
    /// Strict Gen mode: hide fics with relationship or category tags.
    pub strict_gen: Option<bool>,
    /// Comma-separated "type_id:name" pairs
    pub include_any_tags: Option<String>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    /// ISO 8601 datetime string
    pub date_from: Option<String>,
    /// ISO 8601 datetime string
    pub date_to: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    /// Primary tag filter: "type_id:name"
    pub primary_tag: Option<String>,
    pub min_comments: Option<i64>,
    pub min_kudos: Option<i64>,
    pub max_kudos: Option<i64>,
    pub min_bookmarks: Option<i64>,
    pub max_bookmarks: Option<i64>,
    /// Content rating filter (e.g. "Explicit", "Teen And Up Audiences")
    pub rating: Option<String>,
    /// Language filter — maps to `works.language_code`.
    pub language: Option<String>,
    /// Main/primary character filter (Guide tier) — maps to a character tag
    /// with role_confidence >= 1.0 (`@char:Name`).
    pub main_char: Option<String>,
    /// The combined Guided filter: `"<main character>|<attribute>"`.
    ///
    /// Both halves are one filter, not two independent ones — the point is
    /// "fics where this character is the lead AND carries this attribute",
    /// which is not expressible as two separate params without letting a
    /// caller ask for a character who is main in one fic and an attribute it
    /// has elsewhere. The left side is a character tag (type 2) restricted to
    /// main role; the right side is a freeform tag (type 4).
    ///
    /// This parameter is named in src/bin/backfill_scores.rs, src/tags/backfill.rs
    /// and src/roadmap_seed.rs as though it had always existed; it had not —
    /// `SearchQueryParams` derives Deserialize without `deny_unknown_fields`, so
    /// it was silently dropped and the search ran unfiltered. See
    /// docs/specs/search-api-main-char-attr.md.
    pub main_char_attr: Option<String>,
    /// Status filter — maps to `fic_info.status` (e.g. "complete").
    pub status: Option<String>,
    /// Beta/editing status — maps to `works.beta_status`.
    pub beta_status: Option<String>,
    /// Crossover filter — fics with > 1 fandom tag.
    pub crossover: Option<bool>,
    /// Minimum / maximum hit count (works.hit_count).
    pub min_hits: Option<i64>,
    pub max_hits: Option<i64>,
    pub no_warnings: Option<bool>,
    /// Comma-separated tag IDs: "1,2,3"
    pub tag_ids: Option<String>,
    /// Comma-separated character names — finds relationships containing these characters
    pub relationship_characters: Option<String>,
    /// Hide works the signed-in user has marked read (status = 'completed').
    pub hide_read: Option<bool>,
    /// Hide works the signed-in user has bookmarked.
    pub hide_bookmarked: Option<bool>,
    /// Search only works the signed-in user has bookmarked ("My library").
    pub library_only: Option<bool>,
}

impl SearchQueryParams {
    /// Convert raw string parameters into typed SearchParams.
    /// `pub(super)` — reused by `search/ask.rs` (Ask the Archive) so the
    /// LLM-translated params go through the exact same parsing path as a
    /// hand-built query string.
    pub(super) fn into_search_params(self) -> AppResult<SearchParams> {
        let include_tags = self.include_tags.as_deref().unwrap_or("").to_string();
        let exclude_tags = self.exclude_tags.as_deref().unwrap_or("").to_string();
        let include_any_tags = self.include_any_tags.as_deref().unwrap_or("").to_string();

        let date_from = match self.date_from {
            Some(ref s) if !s.is_empty() => {
                let dt = DateTime::parse_from_rfc3339(s).map_err(|e| {
                    AppError::BadRequest(format!("Invalid date_from '{}': {}", s, e))
                })?;
                Some(dt.with_timezone(&Utc))
            }
            _ => None,
        };

        let date_to = match self.date_to {
            Some(ref s) if !s.is_empty() => {
                let dt = DateTime::parse_from_rfc3339(s)
                    .map_err(|e| AppError::BadRequest(format!("Invalid date_to '{}': {}", s, e)))?;
                Some(dt.with_timezone(&Utc))
            }
            _ => None,
        };

        // Parse primary_tag
        let primary_tag = match self.primary_tag {
            Some(ref s) if !s.is_empty() => {
                let filters = parse_tag_filters(s).map_err(|e| AppError::BadRequest(e))?;
                filters.into_iter().next()
            }
            _ => None,
        };

        // Parse tag_ids
        let tag_ids = match self.tag_ids {
            Some(ref s) if !s.is_empty() => s
                .split(',')
                .filter_map(|part| part.trim().parse::<i32>().ok())
                .collect(),
            _ => Vec::new(),
        };

        // Parse exclude_tag_types — comma-separated numeric tag type ids.
        // Unknown ids are silently dropped (a malformed param is a no-op,
        // never an error, keeping the search endpoint forgiving).
        let exclude_tag_types = match self.exclude_tag_types {
            Some(ref s) if !s.is_empty() => s
                .split(',')
                .filter_map(|part| part.trim().parse::<i16>().ok())
                .collect(),
            _ => Vec::new(),
        };

        Ok(SearchParams {
            q: self.q.filter(|s| !s.is_empty()),
            fuzzy: false,
            include_tags: parse_tag_filters(&include_tags).map_err(|e| AppError::BadRequest(e))?,
            exclude_tags: parse_tag_filters(&exclude_tags).map_err(|e| AppError::BadRequest(e))?,
            exclude_tag_types,
            strict_gen: self.strict_gen.unwrap_or(false),
            include_any_tags: parse_tag_filters(&include_any_tags)
                .map_err(|e| AppError::BadRequest(e))?,
            min_words: self.min_words,
            max_words: self.max_words,
            min_chapters: self.min_chapters,
            max_chapters: self.max_chapters,
            complete: self.complete,
            source: self.source.filter(|s| !s.is_empty()),
            date_from,
            date_to,
            sort: self.sort.filter(|s| !s.is_empty()),
            page: self.page,
            per_page: self.per_page,
            primary_tag,
            min_comments: self.min_comments,
            min_kudos: self.min_kudos,
            max_kudos: self.max_kudos,
            min_bookmarks: self.min_bookmarks,
            max_bookmarks: self.max_bookmarks,
            rating: self.rating.filter(|s| !s.is_empty()),
            language: self.language.filter(|s| !s.is_empty()),
            main_char: self.main_char.filter(|s| !s.is_empty()),
            main_char_attr: self.main_char_attr.filter(|s| !s.is_empty()),
            status: self.status.filter(|s| !s.is_empty()),
            beta_status: self.beta_status.filter(|s| !s.is_empty()),
            crossover: self.crossover,
            min_hits: self.min_hits,
            max_hits: self.max_hits,
            no_warnings: self.no_warnings,
            tag_ids,
            relationship_characters: self.relationship_characters.filter(|s| !s.is_empty()),
            // These are filled in by the query builder / handler
            parsed_tsquery: None,
            fielded_terms: Vec::new(),
            field_queries: Vec::new(),
            expanded_include_tag_ids: Vec::new(),
            expanded_exclude_tag_ids: Vec::new(),
            expanded_include_any_tag_ids: Vec::new(),
            hide_read: self.hide_read.unwrap_or(false),
            hide_bookmarked: self.hide_bookmarked.unwrap_or(false),
            library_only: self.library_only.unwrap_or(false),
            user_id: None,
        })
    }
}

/// Facet value with count
#[derive(Debug, Clone, Serialize)]
pub struct FacetValue {
    pub name: String,
    pub count: i64,
}

/// Apply the boolean-query parser to a `SearchParams.q`, filling in
/// `parsed_tsquery`, `fielded_terms`, `include_any_tags` (for `fandom:` /
/// `character:` / `relationship:` fields) and `exclude_tags` (for `-`-prefixed
/// terms). Shared by `run_search` (GET /api/search + Ask) and by the
/// saved-search run/watcher path so the saved query text is interpreted
/// exactly like a live search.
pub fn apply_query_parse(search_params: &mut SearchParams) {
    if let Some(ref q) = search_params.q.clone() {
        let parsed = parse_query(q);
        let text_tsquery = extract_text_tsquery(&parsed);
        let fielded_terms = extract_fielded_terms(&parsed);
        let field_queries = extract_field_queries(&parsed);
        let excluded_terms = extract_excluded_terms(&parsed);

        tracing::debug!(
            "Parsed query: q={:?}, text_tsquery={:?}, fielded={:?}, excluded={:?}",
            q,
            text_tsquery,
            fielded_terms,
            excluded_terms
        );

        // If we have a proper tsquery (non-empty), use it instead of plainto_tsquery
        if !text_tsquery.is_empty() {
            search_params.parsed_tsquery = Some(text_tsquery);
        } else if !fielded_terms.is_empty() || !field_queries.is_empty() {
            // Query is fully fielded (e.g. `title:harry` or `author:jk` or
            // `words:>50k` or `@char:Harry`): the raw q string would tokenize
            // poorly under plainto_tsquery (e.g. "title:harry" -> title &
            // harry), which would filter out every hit. Clear q so only the
            // fielded ILIKE/tag/clause filters apply.
            search_params.q = None;
        }

        // Convert fielded terms to tag/field filters
        for (field, value) in &fielded_terms {
            match field.as_str() {
                "title" => {
                    // title:something — We'll pass it to the builder for a title ILIKE clause
                    // For now, add it as a special note — the builder handles it
                }
                "author" => {
                    // author:something — same as title
                }
                "fandom" => {
                    search_params.include_any_tags.push(TagFilter {
                        tag_type_id: 1,
                        tag_name: value.clone(),
                    });
                }
                "character" => {
                    search_params.include_any_tags.push(TagFilter {
                        tag_type_id: 2,
                        tag_name: value.clone(),
                    });
                }
                "relationship" => {
                    search_params.include_any_tags.push(TagFilter {
                        tag_type_id: 3,
                        tag_name: value.clone(),
                    });
                }
                _ => {}
            }
        }
        // Add excluded terms as exclude_tags
        for term in &excluded_terms {
            search_params.exclude_tags.push(TagFilter {
                tag_type_id: 0, // 0 means any type
                tag_name: term.clone(),
            });
        }
        // Store fielded terms for the builder
        search_params.fielded_terms = fielded_terms;
        // Store the structured v2 field expressions (ranges, @-roles,
        // romship/platship, crossover, counters, etc.) for the builder.
        search_params.field_queries = field_queries;
    }
}

/// Run a query through the shared search pipeline and return the distinct
/// `works.id` values that match (ignoring pagination / facets). Used by the
/// saved-search `run` endpoint (to diff + render) and by the nightly
/// `saved_search_watcher` (to record newly-seen works). Reuses `apply_query_parse`
/// + the same builder as `run_search`, so a saved query matches exactly like
/// the live `/api/search` call.
pub async fn run_search_work_ids(
    pool: &PgPool,
    query_text: &str,
    per_page: usize,
    tag_hidden_threshold: i16,
) -> AppResult<Vec<i32>> {
    let mut search_params = SearchParams {
        q: Some(query_text.to_string()),
        page: Some(1),
        per_page: Some(per_page),
        ..Default::default()
    };
    apply_query_parse(&mut search_params);

    // Resolve tag names to expanded tag IDs for synonym-aware filtering.
    if !search_params.include_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &search_params.include_tags {
            let ids = get_matching_tag_ids(pool, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        search_params.expanded_include_tag_ids = expanded;
    }
    if !search_params.exclude_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &search_params.exclude_tags {
            let ids = get_matching_tag_ids(pool, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        search_params.expanded_exclude_tag_ids = expanded;
    }
    if !search_params.include_any_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &search_params.include_any_tags {
            let ids = get_matching_tag_ids(pool, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        search_params.expanded_include_any_tag_ids = expanded;
    }

    let has_comments: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='comments')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    let has_ratings: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='work_ratings')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    let has_kudos: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='kudos')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    let has_ratings = has_ratings || has_kudos;
    let has_bookmarks: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='bookmarks')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    let builder = SearchQueryBuilder::new(
        search_params,
        tag_hidden_threshold,
        has_comments,
        has_ratings,
        has_bookmarks,
    );

    let rows: Vec<FicSearchRow> = builder
        .build_data_query()
        .build_query_as()
        .fetch_all(pool)
        .await?;

    let mut ids: Vec<i32> = Vec::new();
    for row in rows {
        if let Some(wid) = row.work_id {
            if !ids.contains(&wid) {
                ids.push(wid);
            }
        }
    }
    Ok(ids)
}

/// Facet counts returned alongside search results
#[derive(Debug, Clone, Serialize)]
pub struct SearchFacets {
    pub fandoms: Vec<FacetValue>,
    pub characters: Vec<FacetValue>,
    pub relationships: Vec<FacetValue>,
    pub warnings: Vec<FacetValue>,
    pub categories: Vec<FacetValue>,
    pub freeforms: Vec<FacetValue>,
    pub statuses: Vec<FacetValue>,
}

/// GET /api/v0/search
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
    auth: crate::routes::auth::AuthUser,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    // Capture raw query for analytics logging BEFORE params are consumed.
    let analytics_query = params.q.clone();
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut search_params = params.into_search_params()?;

    // Personal filters (hide read / hide bookmarked / library-only) are
    // keyed on the signed-in user. Anonymous searches pass user_id 0 and
    // the builder treats them as no-ops, keeping the public surface intact.
    search_params.user_id = auth.user_id;

    // The GET handler logs the raw user-typed query to search_queries
    // after the search runs.
    let envelope = run_search(&state, search_params).await?;

    // ─── Search analytics (best-effort, non-blocking) ──────────────────
    // Log every search so admins can see zero-result queries and volume.
    // The search must never fail because logging failed, so any error here
    // is swallowed.
    let log_query = analytics_query.unwrap_or_default();
    if !log_query.is_empty() {
        if let Err(e) = crate::db::queries::insert_search_query(
            &state.db,
            &log_query,
            envelope.total,
            None,
            client_id.as_deref(),
            auth.user_id,
        )
        .await
        {
            tracing::warn!(error = %e, "failed to log search query for analytics");
        }
    }

    Ok(Json(json!({
        "total": envelope.total,
        "page": envelope.page,
        "per_page": envelope.per_page,
        "results": envelope.results,
        "facets": envelope.facets,
    })))
}

/// The shared search execution pipeline.
///
/// Both `GET /api/search` and `POST /api/search/ask` (Ask the Archive) run
/// through here — this is the single source of truth for query parsing,
/// typo-tolerant fuzzy fallback, result assembly and facet building, so the
/// two endpoints can never drift apart. Analytics logging stays in the
/// handlers (GET logs the raw user-typed query; Ask logs its own marked
/// query so the NL string + applied params are both captured).
pub async fn run_search(
    state: &Arc<AppState>,
    mut search_params: SearchParams,
) -> AppResult<SearchResponseEnvelope> {
    // ─── Parse query string with boolean parser ──────────────────────────
    apply_query_parse(&mut search_params);

    // ─── Handle relationship_characters parameter ────────────────────────
    if let Some(ref chars) = search_params.relationship_characters.clone() {
        if !chars.is_empty() {
            let char_list: Vec<&str> = chars
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            for c in &char_list {
                let pattern = format!("%{}%", c);
                let rel_tags = sqlx::query_scalar::<_, String>(
                    "SELECT name FROM tags WHERE tag_type_id = 3 AND name ILIKE $1 LIMIT 50",
                )
                .bind(&pattern)
                .fetch_all(&state.db)
                .await?;
                for tag_name in rel_tags {
                    search_params.include_any_tags.push(TagFilter {
                        tag_type_id: 3,
                        tag_name,
                    });
                }
            }
        }
    }

    // Cap per_page at the configured maximum
    let per_page = search_params
        .per_page
        .unwrap_or(20)
        .min(state.config.search_max_per_page)
        .max(1);

    let mut capped_params = SearchParams {
        per_page: Some(per_page),
        ..search_params.clone()
    };

// ── Resolve rating param to an include_tags entry ──────────────────
    // Content ratings are stored as tags (tag_type_id = 7), not columns.
    // If the user passes `rating=Explicit`, look up the tag and add it
    // to include_tags so the existing tag-filter machinery handles it.
    //
    // This mutates `capped_params`, not `search_params`, and must run BEFORE
    // the expanded_include_tag_ids resolution below. It used to push into
    // `search_params`, which had already been cloned into `capped_params`
    // above - so the filter was silently dropped and `rating=` was a no-op.
    // The test caught it only because it also sent `q=rating`, which put the
    // expected fic in the results for an unrelated reason.
    // See docs/specs/search-api-main-char-attr.md.
    if let Some(ref rating_name) = capped_params.rating.clone() {
        let rating_tag_id: Option<i32> = sqlx::query_scalar(
            "SELECT id FROM tags WHERE LOWER(name) = LOWER($1) AND tag_type_id = 7",
        )
        .bind(rating_name)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
        if let Some(tag_id) = rating_tag_id {
            // We need the canonical name for include_tags (the DB name).
            let canonical: String = sqlx::query_scalar("SELECT name FROM tags WHERE id = $1")
                .bind(tag_id)
                .fetch_one(&state.db)
                .await
                .unwrap_or_else(|_| rating_name.clone());
            // Append to include_tags so the builder emits the EXISTS clause.
            capped_params.include_tags.push(TagFilter {
                tag_type_id: 7,
                tag_name: canonical,
            });
        }
        // If the rating name doesn't match any tag, silently ignore —
        // the search will return results as if no rating filter was set.
    }


    // Resolve tag names to expanded tag IDs for synonym-aware filtering
    if !capped_params.include_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &capped_params.include_tags {
            let ids = get_matching_tag_ids(&state.db, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        capped_params.expanded_include_tag_ids = expanded;
    }
    if !capped_params.exclude_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &capped_params.exclude_tags {
            let ids = get_matching_tag_ids(&state.db, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        capped_params.expanded_exclude_tag_ids = expanded;
    }
    if !capped_params.include_any_tags.is_empty() {
        let mut expanded = Vec::new();
        for tag in &capped_params.include_any_tags {
            let ids = get_matching_tag_ids(&state.db, &tag.tag_name, tag.tag_type_id).await?;
            expanded.push(ids);
        }
        capped_params.expanded_include_any_tag_ids = expanded;
    }

    // Check if social tables exist (comments, work_ratings, kudos)
    let has_comments: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='comments')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);
    let has_ratings: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='work_ratings')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);
    // min_kudos + the kudos sort + result kudos_count now read the real
    // `kudos` table, so the builder needs that table to exist too. The
    // `has_ratings` field gates it; treat a present `kudos` table as
    // equivalent so fresh installs (which may skip 014-era tables) still
    // get the kudos filter once migration 048 is applied.
    let has_kudos: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='kudos')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);
    let has_ratings = has_ratings || has_kudos;

    // bookmarks table — needed for min_bookmarks / max_bookmarks filters.
    let has_bookmarks: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name='bookmarks')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

        let builder = SearchQueryBuilder::new(
        capped_params,
        state.config.tag_hidden_threshold,
        has_comments,
        has_ratings,
        has_bookmarks,
    );

    // --- Execute count query ---
    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query.build_query_as().fetch_one(&state.db).await?;
    let mut total = total.0;
    let mut fuzzy_builder: Option<SearchQueryBuilder> = None;

    // ─── Typo tolerance (NEXT.md item 5) ───────────────────────────────
    // Zero-result BARE text query? Retry with the builder's fuzzy fallback
    // (pg_trgm ILIKE on title/author) so "Hary Pottr" finds "Harry Potter".
    // Only for plain queries: advanced filters (source, tags, word/chapter
    // bounds, completion, dates) or boolean operators disable the fallback,
    // because fuzzy matching must never override an explicit filter.
    //
    // NOTE: parsed_tsquery is set for ALL text queries (plain AND boolean),
    // so it cannot gate the fallback — instead detect explicit boolean
    // operators in the raw query string (AND/OR/NOT). A plain typo'd query
    // ("Hary Pottr") has no operators and should get the fallback.
    // Detect "explicit boolean" syntax in the raw query so the fuzzy
    // fallback never overrides a user's boolean intent: bare words trigger
    // fuzzy on zero results, but `harry OR ron`, `-draco`, `"exact phrase"`
    // (or `NOT term`) are treated as explicit boolean and stay 0 when they
    // match nothing.
    let has_boolean_ops = search_params
        .q
        .as_deref()
        .map(|q| {
            q.split_whitespace().any(|w| {
                w.eq_ignore_ascii_case("AND")
                    || w.eq_ignore_ascii_case("OR")
                    || w.eq_ignore_ascii_case("NOT")
                    || w.starts_with('-')
                    || w.starts_with('"')
            }) || q.contains('"')
        })
        .unwrap_or(false);
    let has_advanced_filters = search_params.source.is_some()
        || !search_params.include_tags.is_empty()
        || !search_params.exclude_tags.is_empty()
        || !search_params.include_any_tags.is_empty()
        || !search_params.exclude_tag_types.is_empty()
        || search_params.strict_gen
        || search_params.min_words.is_some()
        || search_params.max_words.is_some()
        || search_params.min_chapters.is_some()
        || search_params.max_chapters.is_some()
        || search_params.complete.is_some()
        || search_params.date_from.is_some()
        || search_params.date_to.is_some()
        // Numeric bound filters also count as "advanced": a zero-result
        // search with min_kudos/min_comments must NOT fall back to fuzzy,
        // or the fics would be re-included regardless of the bound.
        || search_params.min_kudos.is_some()
        || search_params.max_kudos.is_some()
        || search_params.min_bookmarks.is_some()
        || search_params.max_bookmarks.is_some()
        || search_params.rating.is_some()
        || search_params.min_comments.is_some();
    if total == 0
        && search_params.q.is_some()
        && !has_advanced_filters
        && !has_boolean_ops
        && !search_params.fuzzy
    {
        let mut fuzzy_params = search_params.clone();
        fuzzy_params.fuzzy = true;
        let builder = SearchQueryBuilder::new(
            fuzzy_params,
            state.config.tag_hidden_threshold,
            has_comments,
            has_ratings,
            has_bookmarks,
        );
        let (fuzzy_total,): (i64,) = builder
            .build_count_query()
            .build_query_as()
            .fetch_one(&state.db)
            .await?;
        if fuzzy_total > 0 {
            total = fuzzy_total;
            // Swap in the fuzzy builder for the data query below.
            fuzzy_builder = Some(builder);
        } else {
            search_params.fuzzy = false;
        }
    }

    // --- Execute data query ---
    let mut data_query = if let Some(ref fb) = fuzzy_builder {
        fb.build_data_query()
    } else {
        builder.build_data_query()
    };
    let rows: Vec<FicSearchRow> = data_query.build_query_as().fetch_all(&state.db).await?;

    // --- Map rows to SearchResult ---
    let results: Vec<SearchResultData> = if rows.is_empty() {
        Vec::new()
    } else {
        // Collect all result url_ids for batch tag fetching
        let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();

        // Fetch tags for all results in one query
        let tag_rows: Vec<TagDbRow> = sqlx::query_as::<_, TagDbRow>(
            r#"SELECT ft.url_id, t.name, t.tag_type_id, tt.name AS type_name, ft.score
               FROM fic_tags ft
               JOIN tags t ON t.id = ft.tag_id
               JOIN tag_types tt ON tt.id = t.tag_type_id
               WHERE ft.url_id = ANY($1)
                 AND ft.score >= $2
               ORDER BY ft.url_id, ft.score DESC"#,
        )
        .bind(&url_ids)
        .bind(state.config.tag_hidden_threshold)
        .fetch_all(&state.db)
        .await?;

        // Fetch comment counts and kudos counts — graceful if tables don't exist yet
        let mut stats_by_fic: std::collections::HashMap<String, (i64, i64)> =
            std::collections::HashMap::new();
        if let Ok(stats_rows) = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT fi.id,
                      COALESCE((SELECT COUNT(*) FROM comments c WHERE c.url_id = fi.id), 0),
                      -- Real kudos count for the search results: signed-in kudos
                      -- only (user_id IS NOT NULL); guest kudos are counted
                      -- separately and never surface here.
                      COALESCE((SELECT COUNT(*) FROM kudos k WHERE k.work_id = fi.work_id AND k.user_id IS NOT NULL), 0)
               FROM fic_info fi WHERE fi.id = ANY($1)"#,
        )
        .bind(&url_ids)
        .fetch_all(&state.db)
        .await
        {
            for (id, c, k) in stats_rows {
                stats_by_fic.insert(id, (c, k));
            }
        }

        // Group tags by url_id — return ALL tags, no server-side limit
        use std::collections::HashMap;
        let mut tags_by_fic: HashMap<String, Vec<TagDbRow>> = HashMap::new();
        let mut freeform_total: HashMap<String, usize> = HashMap::new();
        for tag_row in tag_rows {
            let is_freeform = tag_row.tag_type_id == 4;
            if is_freeform {
                *freeform_total.entry(tag_row.url_id.clone()).or_insert(0) += 1;
            }
            tags_by_fic
                .entry(tag_row.url_id.clone())
                .or_default()
                .push(tag_row);
        }

        // Build response items — this is the else block value
        rows.into_iter()
            .map(|row| {
                let fic_tags = tags_by_fic.remove(&row.id).unwrap_or_default();
                let total_freeform = freeform_total.remove(&row.id).unwrap_or(0);
                let response_tags: Vec<Value> = fic_tags
                    .into_iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "type": t.type_name,
                            "type_id": t.tag_type_id,
                            "score": t.score,
                        })
                    })
                    .collect();
                let (comment_count, kudos_count) = stats_by_fic.remove(&row.id).unwrap_or((0, 0));
                SearchResultData {
                    url_id: row.id,
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
                    tags: response_tags,
                    total_freeform,
                    comment_count,
                    kudos_count,
                }
            })
            .collect()
    };

    // --- Build facet data (filtered by the active search filters) ---
    let mut facet_fandoms = builder.build_facet_query(1);
    let mut facet_characters = builder.build_facet_query(2);
    let mut facet_relationships = builder.build_facet_query(3);
    let mut facet_warnings = builder.build_facet_query(5);
    let mut facet_categories = builder.build_facet_query(6);
    let mut facet_freeforms = builder.build_facet_query(4);
    let mut facet_statuses = builder.build_status_facet_query();
    let facets = SearchFacets {
        fandoms: facet_fandoms
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        characters: facet_characters
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        relationships: facet_relationships
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        warnings: facet_warnings
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        categories: facet_categories
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        freeforms: facet_freeforms
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
        statuses: facet_statuses
            .build_query_as::<(String, i64)>()
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|(name, count)| FacetValue { name, count })
            .collect(),
    };

    let page = builder.page();

    Ok(SearchResponseEnvelope {
        total,
        page,
        per_page,
        results,
        facets,
    })
}

/// Serialisable search response — the shared shape returned by
/// `GET /api/search` and (wrapped with Ask metadata) by
/// `POST /api/search/ask`.
#[derive(Debug, Serialize)]
pub struct SearchResponseEnvelope {
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub results: Vec<SearchResultData>,
    pub facets: SearchFacets,
}

/// Internal struct for search result data including tags.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResultData {
    pub url_id: String,
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
    pub tags: Vec<Value>,
    pub total_freeform: usize,
    pub comment_count: i64,
    pub kudos_count: i64,
}

/// Raw tag row from the database for response building.
#[derive(Debug, Clone, sqlx::FromRow)]
struct TagDbRow {
    url_id: String,
    name: String,
    tag_type_id: i16,
    type_name: String,
    score: i16,
}

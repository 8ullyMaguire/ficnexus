use axum::{
    extract::{Query, State},
    http::{header, HeaderMap},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::search::builder::{
    parse_tag_filters, FicSearchRow, SearchParams, SearchQueryBuilder,
};
use crate::server::AppState;

use super::{
    build_feed, fic_entry, html_escape, iso_now, opds_response, FeedKind,
};

/// Raw query parameters for OPDS search (stringly typed)
#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    /// Comma-separated "type_id:name" pairs
    pub include_tags: Option<String>,
    /// Comma-separated "type_id:name" pairs
    pub exclude_tags: Option<String>,
    /// Comma-separated "type_id:name" pairs
    pub include_any_tags: Option<String>,
    /// Comma-separated tag TYPE ids to exclude entirely (e.g. "3" hides all
    /// relationship tags; "3,6" also hides category tags).
    pub exclude_tag_types: Option<String>,
    /// Strict Gen mode: hide fics with relationship or category tags.
    pub strict_gen: Option<bool>,
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
    pub rating: Option<String>,
    pub language: Option<String>,
    pub no_warnings: Option<bool>,
    /// Comma-separated tag IDs: "1,2,3"
    pub tag_ids: Option<String>,
    /// Comma-separated character names — finds relationships containing these characters
    pub relationship_characters: Option<String>,
}

/// Convert raw string params into typed SearchParams
fn into_search_params(raw: SearchQueryParams) -> AppResult<SearchParams> {
    let include_tags = raw.include_tags.as_deref().unwrap_or("").to_string();
    let exclude_tags = raw.exclude_tags.as_deref().unwrap_or("").to_string();
    let include_any_tags = raw.include_any_tags.as_deref().unwrap_or("").to_string();

    Ok(SearchParams {
        q: raw.q.filter(|s| !s.is_empty()),
        fuzzy: false,
        include_tags: parse_tag_filters(&include_tags)
            .map_err(|e| AppError::BadRequest(e))?,
        exclude_tags: parse_tag_filters(&exclude_tags)
            .map_err(|e| AppError::BadRequest(e))?,
        exclude_tag_types: raw
            .exclude_tag_types
            .as_deref()
            .map(|s| {
                s.split(',')
                    .filter_map(|part| part.trim().parse::<i16>().ok())
                    .collect()
            })
            .unwrap_or_default(),
        strict_gen: raw.strict_gen.unwrap_or(false),
        include_any_tags: parse_tag_filters(&include_any_tags)
            .map_err(|e| AppError::BadRequest(e))?,
        min_words: raw.min_words,
        max_words: raw.max_words,
        min_chapters: raw.min_chapters,
        max_chapters: raw.max_chapters,
        complete: raw.complete,
        source: raw.source.filter(|s| !s.is_empty()),
        date_from: raw
            .date_from
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        date_to: raw
            .date_to
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        primary_tag: raw
            .primary_tag
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|s| parse_tag_filters(s).ok())
            .and_then(|mut f| f.drain(..).next()),
        min_comments: raw.min_comments,
        min_kudos: raw.min_kudos,
        max_kudos: raw.max_kudos,
        min_bookmarks: raw.min_bookmarks,
        max_bookmarks: raw.max_bookmarks,
        rating: raw.rating.filter(|s| !s.is_empty()),
        language: raw.language.filter(|s| !s.is_empty()),
        no_warnings: raw.no_warnings,
        tag_ids: raw
            .tag_ids
            .as_deref()
            .map(|s| {
                s.split(',')
                    .filter_map(|part| part.trim().parse::<i32>().ok())
                    .collect()
            })
            .unwrap_or_default(),
        sort: raw.sort.filter(|s| !s.is_empty()),
        page: raw.page,
        per_page: raw.per_page,
        relationship_characters: raw.relationship_characters.filter(|s| !s.is_empty()),
        parsed_tsquery: None,
        fielded_terms: vec![],
        field_queries: vec![],
        main_char: None,
        status: None,
        beta_status: None,
        crossover: None,
        min_hits: None,
        max_hits: None,
        expanded_include_tag_ids: vec![],
        expanded_exclude_tag_ids: vec![],
        expanded_include_any_tag_ids: vec![],
        // OPDS feeds are public; no per-user personal filters.
        hide_read: false,
        hide_bookmarked: false,
        library_only: false,
        user_id: None,
    })
}

/// OpenSearch description document XML
fn opensearch_description() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/">
  <ShortName>FicNexus</ShortName>
  <Description>Search fanfiction on FicNexus</Description>
  <InputEncoding>UTF-8</InputEncoding>
  <Url type="application/atom+xml;profile=opds-catalog;kind=acquisition"
       template="/opds/search?q={{searchTerms}}&amp;page={{startPage}}"/>
  <Url type="application/opensearchdescription+xml"
       rel="self"
       template="/opds/search"/>
</OpenSearchDescription>"#,
    )
}

/// GET /opds/search — OpenSearch description or search results
///
/// If called with Accept: application/opensearchdescription+xml or no q param,
/// returns an OpenSearch description document.
/// Otherwise returns search results as acquisition feed.
pub async fn search_feed(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    // Check if the client wants the OpenSearch description
    let wants_opensearch = params.q.is_none()
        || headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/opensearchdescription+xml"))
            .unwrap_or(false);

    if wants_opensearch {
        let body = opensearch_description();
        return Ok((
            [(header::CONTENT_TYPE, "application/opensearchdescription+xml")],
            body,
        ));
    }

    // Parse search params
    let search_params = into_search_params(params)?;

    // Cap per_page
    let per_page = search_params
        .per_page
        .unwrap_or(20)
        .min(state.config.search_max_per_page)
        .max(1);

    let capped_params = SearchParams {
        per_page: Some(per_page),
        ..search_params
    };

    let builder =
        SearchQueryBuilder::new(capped_params, state.config.tag_hidden_threshold, true, true, true);

    // Execute count query
    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query
        .build_query_as()
        .fetch_one(&state.db)
        .await?;
    let total = total.0;

    // Execute data query
    let mut data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query
        .build_query_as()
        .fetch_all(&state.db)
        .await?;

    let now = iso_now();
    let query_str = builder.params.q.as_deref().unwrap_or("");
    let base_url = state.config.opds_base_url.as_deref();
    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let acq_map = super::acquisition_links_many(&state.db, base_url, &ids).await;

    let mut entries = String::new();
    for row in &rows {
        let updated = row
            .fic_updated
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let acq = acq_map.get(&row.id).map(|v| v.as_slice()).unwrap_or(&[]);
        entries.push_str(&fic_entry(
            &row.id,
            &row.title,
            &row.author,
            &row.description,
            &updated,
            row.words,
            row.chapters,
            &row.status,
            acq,
        ));
    }

    let page = builder.page();
    let query_encoded = urlencoding::encode(query_str);
    let pagi = super::PaginationInfo {
        // Keep URL encoding separate from XML escaping; build_feed escapes hrefs once.
        base_path: format!("/opds/search?q={}", query_encoded),
        page,
        per_page,
        total,
    };

    let self_link = format!(
        "/opds/search?q={}&page={}&per_page={}",
        query_encoded, page, per_page,
    );

    let feed_title = if query_str.is_empty() {
        "FicNexus — Search Results".to_string()
    } else {
        format!("FicNexus — Search: {}", query_str)
    };

    let feed_id = if query_str.is_empty() {
        "urn:ficnexus:search:all".to_string()
    } else {
        format!("urn:ficnexus:search:q={}", html_escape(query_str))
    };

    let body = build_feed(
        &feed_title,
        &feed_id,
        &entries,
        &now,
        Some(&self_link),
        FeedKind::Acquisition,
        Some(&pagi),
        state.config.opds_base_url.as_deref(),
    );

    Ok(opds_response(body, FeedKind::Acquisition))
}

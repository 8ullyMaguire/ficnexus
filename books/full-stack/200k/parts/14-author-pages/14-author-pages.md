# Part 14 — Author Pages and Series

> In this chapter you will learn how FicHub's author bibliographies and series pages work — canonical author name resolution (case-insensitive + most-common-spelling), orphan fic sources, the `next_in_series` link computation, and author profile management with badges and social links.

---

## Overview

Two page types, both keyed by the canonical author name:

- **Series page** (`GET /api/series/{id}`) — ordered works in reading order, each with a `next_in_series` link (the highest-conversion per-fic link in fanfiction).
- **Author page** (`GET /api/authors/{name}`) — bibliography with aggregate stats (work count, total words), top tags, badges, favorite tags, author profile (avatar/bio/badge_text).

Author profile management (search, create, update, social links, curator merge proposals) lives in `src/routes/authors.rs` (489 lines). Both use `works.canonical_author` as the canonical join key.

---

## Chapter 14.1 — Series Pages with next_in_series

### Goal

Build the series page that lists works in reading order, computing `next_in_series` for each work (the work that follows it).

### Actions

```rust
// src/routes/series.rs (lines 30-132)
pub async fn get_series(State(state): State<Arc<AppState>>,
    Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let series = sqlx::query_as::<_, (i32, String, String, DateTime, DateTime)>(
        "SELECT id, name, description, created_at, updated_at FROM series WHERE id = $1",
    ).bind(id).fetch_optional(&state.db).await?
      .ok_or_else(|| AppError::NotFound("Series not found".into()))?;

    // Ordered works (position ASC — the reading order).
    let rows: Vec<(i32, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT w.id, w.canonical_title, w.canonical_author,
                  w.default_source_id,
                  (SELECT fi.id::text FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1)
           FROM series_works sw JOIN works w ON w.id = sw.work_id
           WHERE sw.series_id = $1
           ORDER BY sw.position ASC, w.id ASC"#,
    ).bind(id).fetch_all(&state.db).await?;

    let mut works: Vec<Value> = Vec::with_capacity(rows.len());
    for (work_id, canonical_title, canonical_author, default_url, fallback_url) in rows {
        // Resolve url_id: prefer default_source_id, fall back to highest-word-count source
        let url_id = default_url.filter(|u| !u.is_empty()).or(fallback_url).unwrap_or_default();

        // Source metadata (best effort)
        let src = sqlx::query_as::<_, (String, String, i64, i32, String)>(
            "SELECT title, author, words, chapters, status FROM fic_info WHERE id = $1",
        ).bind(&url_id).fetch_optional(&state.db).await?.map(|(t, a, w, c, s)| {
            json!({ "title": t, "author": a, "words": w, "chapters": c, "status": s })
        }).unwrap_or_else(|| json!({ "title": canonical_title, "author": canonical_author, "words": 0, "chapters": 0, "status": "unknown" }));

        works.push(json!({ "work_id": work_id, "url_id": url_id, "canonical_title": canonical_title, ... }));
    }

    // Compute next_in_series: the i-th work's next is i+1; last has none.
    for i in 0..works.len() {
        if i + 1 < works.len() {
            let nxt = &works[i + 1];
            works[i]["next_in_series"] = json!({ "work_id": nxt["work_id"], "canonical_title": nxt["canonical_title"], "url_id": nxt["url_id"] });
        } else {
            works[i]["next_in_series"] = Value::Null;
        }
    }

    Ok(Json(json!({ "err": 0, "series": {...}, "works": works })))
}
```

> **💡 Key Concept**: `next_in_series` is the **highest-conversion per-fic link** — it tells the reader "what to read next" in a series, driving them to the next work's page. The computation is a simple index-based loop: `works[i].next = works[i+1]`. It's done in Rust, not SQL, because the position ordering is already resolved by the initial query.

> **⚠️ Watch Out**: `default_source_id` is preferred for the URL, but if it's empty/missing, the code falls back to the highest-word-count source via a correlated subquery. This ensures the card always links somewhere, even if the default source is missing.

### Try It Yourself

```bash
# Get series detail
curl /api/series/42

# Response includes:
# works[].next_in_series = { work_id, canonical_title, url_id } or null
```

### Check

- ✅ Works ordered by `sw.position ASC` (reading order).
- ✅ `url_id` resolved: default_source_id first, then highest-word-count fallback.
- ✅ `next_in_series` computed in Rust (index-based, last work = null).
- ✅ Fallback metadata when `fic_info` row is missing.
- ✅ 2 unit tests for the `next_in_series` loop logic.

### What you built

The series page — ordered works via `series_works.position`, URL resolution with fallback, `next_in_series` computation, and graceful metadata fallback.

---

## Chapter 14.2 — Author Bibliographies

### Goal

Build the author page that aggregates all works by an author, handles orphan sources, resolves canonical names, and shows stats + badges.

### Actions

```rust
// src/routes/series.rs (lines 140-339)
pub async fn get_author(State(state): State<Arc<AppState>>,
    Path(name): Path<String>) -> Result<Json<Value>, AppError> {
    let name = name.trim();
    if name.is_empty() { return Err(AppError::BadRequest("author name required".into())); }

    // Canonical author = most common works.canonical_author spelling (case-insensitive)
    let canonical: Option<String> = sqlx::query_scalar(
        r#"SELECT canonical_author FROM works
           WHERE LOWER(canonical_author) = LOWER($1) AND canonical_author <> ''
           GROUP BY canonical_author ORDER BY COUNT(*) DESC, canonical_author LIMIT 1"#,
    ).bind(&name).fetch_optional(&state.db).await?;
    let canonical_name = canonical.unwrap_or_else(|| name.to_string());

    // Works (canonical entries) by this author
    let works: Vec<Value> = sqlx::query_as::<_, (i32, String, String, String, Option<String>, Option<String>, i64, i32, String)>(
        r#"SELECT w.id, w.canonical_title, w.description, w.canonical_author,
                  w.default_source_id,
                  (SELECT fi.id::text FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1),
                  COALESCE((SELECT fi.words FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 0),
                  COALESCE((SELECT fi.chapters FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 0),
                  COALESCE((SELECT fi.status FROM fic_info fi WHERE fi.work_id = w.id ORDER BY fi.words DESC LIMIT 1), 'unknown')
           FROM works w
           WHERE LOWER(w.canonical_author) = LOWER($1)
           ORDER BY w.updated_at DESC, w.id DESC"#,
    ).bind(&name).fetch_all(&state.db).await?
    .into_iter().map(|(id, title, desc, author, default_url, fallback_url, words, chapters, status)| {
        // Same url_id resolution logic as series page
        json!({ "work_id": id, "url_id": url_id, "canonical_title": title, ... })
    }).collect();

    // Orphan sources: fic_info rows by this author NOT linked to a canonical work
    let orphans: Vec<Value> = sqlx::query_as::<_, (String, String, String, i64, i32, String, Option<i32>)>(
        r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, fi.status, fi.work_id
           FROM fic_info fi WHERE LOWER(fi.author) = LOWER($1) AND fi.work_id IS NULL
           ORDER BY fi.fic_updated DESC LIMIT 50"#,
    ).bind(&name).fetch_all(&state.db).await?
    .into_iter().map(|(url_id, title, author, words, chapters, status, _)| {
        json!({ "work_id": null, "url_id": url_id, "canonical_title": title, ... })
    }).collect();

    // Aggregate stats: work count + total words
    let stats = sqlx::query_as::<_, (i64, Option<i64>)>(
        r#"SELECT COUNT(*)::bigint, SUM(fi.words)::bigint FROM fic_info fi WHERE LOWER(fi.author) = LOWER($1)"#,
    ).bind(&name).fetch_one(&state.db).await?;

    // Author profile (avatar, bio, badge_text) — matched by canonical_name
    let author_profile = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        r#"SELECT avatar_url, bio, badge_text FROM author_profiles WHERE canonical_name = $1
           ORDER BY updated_at DESC LIMIT 1"#,
    ).bind(&canonical_name).fetch_optional(&state.db).await?;

    // Author's earned badges (if author_profile_links to a user)
    let author_id_opt = sqlx::query_scalar::<_, i32>(
        r#"SELECT user_id FROM author_profile_links apl
           JOIN author_profiles ap ON ap.id = apl.profile_id WHERE ap.canonical_name = $1 LIMIT 1"#,
    ).bind(&canonical_name).fetch_optional(&state.db).await?;

    // Top tags: freeform (type 4) + fandom (type 1) by usage count
    let top_tags = sqlx::query_as(
        r#"SELECT t.name, t.tag_type_id, COUNT(DISTINCT ft.url_id)::bigint AS usage_count
           FROM tags t JOIN fic_tags ft ON ft.tag_id = t.id JOIN fic_info fi ON fi.id = ft.url_id
           WHERE LOWER(fi.author) = LOWER($1) AND t.tag_type_id IN (1, 4)
           GROUP BY t.id ORDER BY usage_count DESC, t.name LIMIT 20"#,
    ).bind(&name).fetch_all(&state.db).await?;

    Ok(Json(json!({ "err": 0, "author": {...}, "works": works, "orphans": orphans })))
}
```

> **⚠️ Watch Out**: The canonical author name uses a **GROUP BY + COUNT(*) DESC** query — the most common spelling wins. This handles authors who appear with slightly different spellings across sources. Orphan sources (`fic_info.work_id IS NULL`) are surfaced separately so the bibliography is complete even for works not yet deduplicated into a `works` row.

### Try It Yourself

```bash
curl /api/authors/J.K.%20Rowling

# Returns: canonical_name, avatar_url, bio, badges, favorite_tags,
#           work_count, total_words, top_tags, works[], orphans[]
```

### Check

- ✅ Case-insensitive author matching (`LOWER(canonical_author) = LOWER($1)`).
- ✅ Canonical name = most common spelling (GROUP BY + COUNT DESC).
- ✅ Orphan sources surfaced (fic_info with work_id IS NULL).
- ✅ Stats computed from `fic_info` (sum of words across all sources).
- ✅ Badges joined via `author_profile_links → author_profiles → user_badges`.
- ✅ Top tags filtered to fandom (type 1) + freeform (type 4).

### What you built

The author bibliography — case-insensitive canonical name resolution, orphan source surfacing, aggregate stats, badge/profile display, and top tags. Plus the series page with `next_in_series` computation.

---

## Conclusion

You now understand FicHub's author-centric pages:

1. **Series page** — ordered works via `series_works.position`, `next_in_series` computation in Rust, URL resolution with fallback.
2. **Author bibliography** — case-insensitive canonical name, orphan source handling, aggregate stats (work count + total words), badge/profile display, top tags.
3. **Author profiles** (`src/routes/authors.rs`) — search, CRUD, social links, curator merge proposals with admin auto-approve.
# Part 21 — Saved Searches and Alerts

Imagine you find a search you run *all the time* — like "harry potter AND complete, sorted by kudos." Wouldn't it be nice to save that search so you can run it again with one click? And what if FicHub checked it for you every night and emailed you when new fics that match show up?

That's exactly what **saved searches and alerts** do. This part builds the backend endpoints that store your saved searches in the database, let you toggle nightly alerts on and off, and re-run your saved queries on demand. Then we look at the frontend that lets you manage your saved searches right from the Search page.

---

## 21.1 Backend: `POST/GET /api/search/saved` — saved searches

### Goal

Create an endpoint to **save** a search (POST) and an endpoint to **list** all of a user's saved searches (GET). Both require you to be logged in.

### Actions

Open `src/routes/saved_search.rs`. Every file in FicHub starts with a doc comment — this one explains the whole feature:

```rust
//! Saved searches + daily alerts.
//!
//! A logged-in user can save a search (a name + the raw query text + the JSON
//! AST of the parsed boolean query), re-run it on demand, toggle nightly alert
//! mode (`alert_mode` = `none` | `rss`), or delete it. The nightly
//! `saved_search_watcher` bin re-runs every alerting (`rss`) search, diffs the
//! current match set against `saved_search_matches`, and records newly-seen
//! works; those are exposed in a public per-search Atom feed at
//! `/feed/saved/{user_id}/{search_id}`.
```

So a saved search is really **three things**:
1. A **name** you give it (like "HP favorites").
2. The **query text** (like `harry potter AND complete`).
3. A **JSON AST** — the parsed boolean query tree, stored so the nightly watcher doesn't have to re-parse it.

Let's start with the data types. Right after the imports, there are the request/response structs:

```rust
// src/routes/saved_search.rs (lines 19-58, excerpt)
/// A saved-search row as returned by list/get queries.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct SavedSearchRow {
    pub id: i64,
    pub name: String,
    pub query_text: String,
    pub alert_mode: String,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_match_count: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request body for creating a saved search.
#[derive(Debug, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub query_text: String,
}
```

The `SavedSearchRow` struct uses `sqlx::FromRow` (so SQLx can map a database row to it) and `serde::Serialize` (so Axum can turn it into JSON). Every field maps directly to a column in the `saved_searches` table.

### Creating a saved search: `POST /api/search/saved`

Here's the `create_saved_search` handler:

```rust
// src/routes/saved_search.rs (lines 111-154)
/// POST /api/search/saved — create a saved search.
pub async fn create_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateSavedSearchRequest>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;
    validate_create_request(&body)?;

    let name = body.name.trim().to_string();
    let query_text = body.query_text.trim().to_string();
    let query_json = query_to_ast(&query_text);

    // The saved search must be runnable — fail fast if the query text does
    // not yield a meaningful search. Run the parser; an empty AST means the
    // query is effectively blank.
    if query_json == Value::Null && query_text.is_empty() {
        return Err(AppError::BadRequest(
            "Saved search query cannot be empty".to_string(),
        ));
    }

    let row: SavedSearchRow = sqlx::query_as(
        r#"INSERT INTO saved_searches (user_id, name, query_text, query_json)
           VALUES ($1, $2, $3, $4)
           RETURNING id, name, query_text, alert_mode, last_run_at, last_match_count, created_at"#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&query_text)
    .bind(&query_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::Conflict(format!("A saved search named '{}' already exists", name))
        }
        other => AppError::Database(other.to_string()),
    })?;

    Ok(Json(json!({
        "err": 0,
        "saved_search": row,
    })))
}
```

**Step by step:**

1. **Authentication** — `let user_id = require_user(&auth)?;` pulls the user ID out of the JWT. If the user isn't logged in, this returns 401 Unauthorized. The `require_user` helper is tiny:

```rust
// src/routes/saved_search.rs (lines 104-108)
/// Resolve the signed-in user's id or bail with 401.
fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}
```

2. **Validation** — `validate_create_request` checks three things: the name isn't blank, it's under 120 characters, and the query text isn't empty. If any of those fail, the user gets a 400 Bad Request with a helpful message:

```rust
// src/routes/saved_search.rs (lines 78-95)
/// Validate a create-saved-search request (name + non-empty query text).
pub fn validate_create_request(req: &CreateSavedSearchRequest) -> Result<(), AppError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Saved search name is required".to_string()));
    }
    if name.len() > 120 {
        return Err(AppError::BadRequest(
            "Saved search name must be 120 characters or fewer".to_string(),
        ));
    }
    if req.query_text.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Saved search query cannot be empty".to_string(),
        ));
    }
    Ok(())
}
```

3. **AST serialization** — `query_to_ast` runs the query through the same boolean parser that the live search uses, so the stored AST and the live `q` parameter always produce the same results:

```rust
// src/routes/saved_search.rs (lines 100-102)
/// Serialize a search query text into the JSON AST stored in `query_json`.
pub fn query_to_ast(query_text: &str) -> Value {
    serde_json::to_value(parse_query(query_text)).unwrap_or_else(|_| Value::Null)
}
```

The `parse_query` function lives in `src/search/parser.rs`. For a simple query like `harry potter`, it produces something like `{"Term": {"Word": "harry"}}`. For `coffee AND tea`, it becomes `{"And": [...]}`. The nightly watcher uses this AST instead of re-parsing the text — that way, even if the parser is updated, old saved searches keep their original parsed structure.

4. **Database insert** — The `INSERT` stores the user ID, name, query text, and JSON AST. The `RETURNING` clause gives us the full row back in one query (including `alert_mode` which defaults to `none`). If the user already has a saved search with that name, PostgreSQL raises a unique violation, which we turn into a 409 Conflict:

```rust
sqlx::Error::Database(ref de) if de.is_unique_violation() => {
    AppError::Conflict(format!("A saved search named '{}' already exists", name))
}
```

5. **Response** — Returns `{"err": 0, "saved_search": {...}}` with the full row.

### Listing saved searches: `GET /api/search/saved`

The list handler is much simpler — it just queries all saved searches for the current user, newest first:

```rust
// src/routes/saved_search.rs (lines 156-177)
/// GET /api/search/saved — list the current user's saved searches.
pub async fn list_saved_searches(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let rows: Vec<SavedSearchRow> = sqlx::query_as(
        r#"SELECT id, name, query_text, alert_mode, last_run_at, last_match_count, created_at
           FROM saved_searches
           WHERE user_id = $1
           ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "saved_searches": rows,
    })))
}
```

The `last_run_at` and `last_match_count` columns are updated every time a saved search is re-run (we'll see that in the "run" endpoint). The frontend uses these to show you when the search was last checked and how many matches it found.

### Route registration

In `src/server.rs`, the routes are registered right next to the other search endpoints (lines 378-383):

```rust
// src/server.rs (lines 378-383)
// Saved searches + daily alerts
.route("/api/search/saved", axum::routing::post(crate::routes::saved_search::create_saved_search))
.route("/api/search/saved", get(crate::routes::saved_search::list_saved_searches))
.route("/api/search/saved/{id}", axum::routing::delete(crate::routes::saved_search::delete_saved_search))
.route("/api/search/saved/{id}/alert", axum::routing::put(crate::routes::saved_search::update_saved_search_alert))
.route("/api/search/saved/{id}/run", axum::routing::post(crate::routes::saved_search::run_saved_search))
```

Notice that `POST` and `GET` both go to `/api/search/saved` — Axum dispatches based on the HTTP method. The `{id}` routes use the same pattern as bookmarks (from Part 9): the path parameter is extracted and the handler filters by both `id` *and* `user_id` for safety.

### The alert mode validator

Only two alert modes are valid: `"none"` (no nightly checking) and `"rss"` (nightly checking with results published to an Atom feed). The `validate_alert_mode` function enforces this:

```rust
// src/routes/saved_search.rs (lines 67-76)
/// Validate an alert-mode value: only `none` and `rss` are accepted.
pub fn validate_alert_mode(mode: &str) -> Result<(), AppError> {
    match mode {
        "none" | "rss" => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "Invalid alert_mode '{}': must be 'none' or 'rss'",
            mode
        ))),
    }
}
```

This function is unit-tested right in the same file — there are tests that confirm `"none"` and `"rss"` pass, while `"email"`, `""`, and `"rss-feed"` are rejected:

```rust
// src/routes/saved_search.rs (lines 416-427)
#[test]
fn alert_mode_accepts_valid_values() {
    assert!(validate_alert_mode("none").is_ok());
    assert!(validate_alert_mode("rss").is_ok());
}

#[test]
fn alert_mode_rejects_invalid_values() {
    assert!(validate_alert_mode("email").is_err());
    assert!(validate_alert_mode("").is_err());
    assert!(validate_alert_mode("rss-feed").is_err());
}
```

### Check

**Test 1: Create a saved search**

```bash
curl -s -X POST http://localhost:8000/api/search/saved \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"name": "My Favorite Search", "query_text": "harry potter AND complete"}' \
  | python3 -m json.tool
```

**Expected**: `{"err": 0, "saved_search": {"id": 1, "name": "My Favorite Search", "query_text": "harry potter AND complete", "alert_mode": "none", ...}}`

**Test 2: List saved searches**

```bash
curl -s http://localhost:8000/api/search/saved \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: `{"err": 0, "saved_searches": [...]}` — your newly created search should be in the list.

**Test 3: Try creating a duplicate name**

The second `POST` with the same name should return 409 Conflict.

### Troubleshooting

- **401 Unauthorized**: You're not logged in. Make sure your JWT token is valid. Log in first with `POST /api/auth/login`.
- **400 Bad Request**: Either the name is blank/too long or the query text is empty. Check the error message in the response.
- **409 Conflict**: A saved search with that name already exists. Pick a different name or delete the old one first (see Part 21.2).
- **"Invalid alert_mode"**: You're probably trying to set an alert mode directly — the `POST` endpoint always starts with `alert_mode = "none"`. Use the PUT endpoint (Part 21.3) to change it later.

---

## 21.2 Backend: `DELETE /api/search/saved/:id` — delete saved search

### Goal

Let a user **delete** one of their own saved searches. The `:id` in the URL is the saved search's database ID.

### Actions

The delete handler is short and follows the same safety pattern as bookmarks:

```rust
// src/routes/saved_search.rs (lines 179-203)
/// DELETE /api/search/saved/{id} — delete one of the current user's searches.
pub async fn delete_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let deleted = sqlx::query(
        "DELETE FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(path.id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Saved search {} not found",
            path.id
        )));
    }

    Ok(Json(json!({ "err": 0, "deleted": path.id })))
}
```

**Key details:**

1. **Path parameter** — `Path(path): Path<AlertPathParams>` extracts the `{id}` from the URL. The `AlertPathParams` struct is:

```rust
// src/routes/saved_search.rs (lines 55-58)
#[derive(Debug, Deserialize)]
pub struct AlertPathParams {
    pub id: i64,
}
```

It's a simple struct with just an `id` field. Axum deserializes the path parameter into it.

2. **Scoped delete** — The `WHERE` clause filters on **both** `id` *and* `user_id`. This is critical: without the `user_id` check, any logged-in user could delete any other user's saved search by guessing the ID. By requiring both, we ensure each user can only delete their own searches.

3. **Not found handling** — If the query affects 0 rows (meaning the ID doesn't exist or doesn't belong to this user), we return a 404 Not Found. This prevents an attacker from even confirming whether a search ID belongs to another user.

4. **Cascading deletes** — The `saved_searches` table has a foreign key to `saved_search_matches` (the table that stores which works each nightly search has seen). When you delete a saved search, the matches are cleaned up automatically by the database's `ON DELETE CASCADE` rule. You don't need to write a separate cleanup query.

### The `run_saved_search` endpoint (bonus)

While we're here, let's look at the fourth route registered in `server.rs`:

```rust
// src/server.rs line 383
.route("/api/search/saved/{id}/run", axum::routing::post(crate::routes::saved_search::run_saved_search))
```

This endpoint lets you **re-run a saved search on demand** — exactly what the "Run" button in the frontend triggers:

```rust
// src/routes/saved_search.rs (lines 241-288)
/// POST /api/search/saved/{id}/run — re-run the saved query and return the
/// same result shape as the normal search (so the frontend can render it
/// identically). Also refreshes `last_run_at` / `last_match_count`.
pub async fn run_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT name, query_text FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(path.id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (name, query_text) = row.ok_or_else(|| {
        AppError::NotFound(format!("Saved search {} not found", path.id))
    })?;

    // Run through the exact same pipeline as a live /api/search call so the
    // response shape is identical (total / page / per_page / results / facets).
    let params = SearchParams {
        q: Some(query_text.clone()),
        user_id: Some(user_id),
        ..Default::default()
    };
    let envelope = run_search(&state, params).await?;

    let last_match_count = envelope.total as i32;
    sqlx::query(
        "UPDATE saved_searches SET last_run_at = now(), last_match_count = $1 WHERE id = $2",
    )
    .bind(last_match_count)
    .bind(path.id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "saved_search_id": path.id,
        "name": name,
        "total": envelope.total,
        "page": envelope.page,
        "per_page": envelope.per_page,
        "results": envelope.results,
        "facets": envelope.facets,
    })))
}
```

The cool part is that it calls `run_search` — the exact same function used by `GET /api/search`. This means a saved search produces the **same results** as a live search with the same query text. The code comment says it best: "Run through the exact same pipeline as a live `/api/search` call so the response shape is identical."

After the search runs, it **updates** `last_run_at` (set to `now()`) and `last_match_count` (the total number of matching works). The frontend uses `last_match_count` to show you a badge like "42 matches" next to each saved search, and `last_run_at` to tell you when it was last checked.

The `run_search` function lives in `src/search/routes.rs` and is the shared search pipeline. It calls `apply_query_parse` to turn the query text into the boolean AST, then builds the full SQL query with all the filters (tags, word counts, sort order, etc.). You don't need to understand the whole pipeline — just know that saved searches and live searches use the same code path.

### Check

**Test: Delete a saved search**

```bash
curl -s -X DELETE http://localhost:8000/api/search/saved/1 \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: `{"err": 0, "deleted": 1}`

**Test: Confirm it's gone**

```bash
curl -s http://localhost:8000/api/search/saved \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: `{"err": 0, "saved_searches": []}` — the deleted search no longer appears.

### Troubleshooting

- **404 Not Found**: The search ID doesn't exist, or it belongs to another user. Double-check the ID by listing your saved searches first.
- **401 Unauthorized**: You need to be logged in. Remember the `Authorization: Bearer ***`?

---

## 21.3 Backend: `PUT /api/search/saved/:id/alert` — alert toggle

### Goal

Let a logged-in user **turn nightly alerts on or off** for one of their saved searches. When alerts are on (`alert_mode = "rss"`), a background watcher re-runs the search every night, finds new matches, and writes them to a public Atom feed.

### Actions

The alert toggle handler:

```rust
// src/routes/saved_search.rs (lines 205-236)
/// PUT /api/search/saved/{id}/alert — set the alert mode (none | rss).
pub async fn update_saved_search_alert(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
    Json(body): Json<UpdateAlertRequest>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;
    validate_alert_mode(&body.alert_mode)?;

    let updated = sqlx::query(
        "UPDATE saved_searches SET alert_mode = $1 WHERE id = $2 AND user_id = $3",
    )
    .bind(&body.alert_mode)
    .bind(path.id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Saved search {} not found",
            path.id
        )));
    }

    Ok(Json(json!({
        "err": 0,
        "id": path.id,
        "alert_mode": body.alert_mode,
    })))
}
```

The `UpdateAlertRequest` body is a simple struct:

```rust
// src/routes/saved_search.rs (lines 50-53)
/// Request body for updating the alert mode.
#[derive(Debug, Deserialize)]
pub struct UpdateAlertRequest {
    pub alert_mode: String,
}
```

**How it works:**

1. **Validate** — The `alert_mode` field must be exactly `"none"` or `"rss"`. The `validate_alert_mode` function (seen in Part 21.1) enforces this and returns 400 for anything else.

2. **Update** — The SQL `UPDATE` sets the new `alert_mode` on the row, scoped by both `id` and `user_id` — same safety pattern as delete.

3. **Not found** — If no rows were affected, we return 404. The user can't toggle the alert on a search that doesn't exist or doesn't belong to them.

### What "rss" alert mode actually does

When `alert_mode` is set to `"rss"`, here's the chain of events:

1. **The nightly watcher** (`saved_search_watcher` bin) runs every night. It queries the database for all searches where `alert_mode = 'rss'`.
2. For each one, it **re-runs the saved query** (using the stored `query_json` AST) and collects the list of matching `work_id`s.
3. It **diffs** the current match set against `saved_search_matches` — the table that stores which works each alerting search has already seen.
4. Any **new** matches (works that match now but weren't in `saved_search_matches`) get inserted into that table.
5. Those new matches become **entries in the public Atom feed** at `/feed/saved/{user_id}/{search_id}` (more on this below).

You don't need to write the watcher — it already exists as a separate binary. Your job in this part is just the HTTP API that lets users toggle the alert flag.

### The public Atom feed: `saved_search_feed`

At the bottom of `saved_search.rs` is the handler that generates the RSS feed for alerted searches:

```rust
// src/routes/saved_search.rs (lines 342-408)
/// GET /feed/saved/{user_id}/{search_id} — public per-search Atom feed of the
/// works that saved search has matched so far (newest match first).
///
/// The route is registered as `/feed/saved/{user_id}/{search_id}` because axum
/// 0.8 forbids mixed literal+param segments like `{search_id}.xml`; a trailing
/// `.xml` is stripped here so the canonical URL works.
pub async fn saved_search_feed(
    State(state): State<Arc<AppState>>,
    Path(params): Path<FeedPathParams>,
    query: Query<ayum::feed::FeedQuery>,
) -> AppResult<impl IntoResponse> {
    let search_id = params.search_id;

    // Verify the search exists and belongs to the given user.
    let (name,): (String,) = sqlx::query_as(
        "SELECT name FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(search_id)
    .bind(params.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("saved search not found".into()))?;

    let rows = matched_feed_rows(&state, search_id, super::rss::MAX_ENTRIES).await?;
    let now = iso_now();

    let entries: String = rows
        .iter()
        .map(|r| {
            let updated = r.first_seen.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            format!(
                r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>urn:fichub:saved-search:{search_id}:{work_id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <link rel="alternate" href="/fic/{url_id}" type="text/html"/>
    <category term="{status}" label="{status}"/>
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <category term="words:{words}" label="{words} words"/>
  </entry>
"#,
                title = html_escape(&r.title),
                author = html_escape(&r.author),
                search_id = search_id,
                work_id = r.work_id,
                updated = updated,
                summary = html_escape(&r.description),
                url_id = html_escape(&r.url_id),
                status = html_escape(&r.status),
                chapters = r.chapters,
                words = r.words,
            )
        })
        .collect();

    let body = atom_feed(
        &format!("urn:fichub:feed:saved:{}:{}", params.user_id, search_id),
        &format!("FicHub — Saved Search: {}", name),
        "New works matching this saved search",
        &format!(
            "/feed/saved/{}/{}{}",
            params.user_id,
            search_id,
            ".xml"
        ),
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );

    Ok(atom_response(body))
}
```

**Key points:**

- The feed is **public** — no auth required. Anyone with the URL can subscribe.
- The URL includes both `user_id` and `search_id` (like `/feed/saved/42/7.xml`). This is an unguessable URL pattern — an attacker would need to know your user ID and search ID.
- The `atom_feed` and `atom_response` helpers come from `src/routes/rss.rs` — the same module that builds the site-wide RSS feed (you saw this in Part 19).
- The `matched_feed_rows` function (lines 292-320) does a `JOIN` between `saved_search_matches`, `works`, and `fic_info` to pull the fic metadata for each matched work:

```rust
// src/routes/saved_search.rs (lines 290-320)
async fn matched_feed_rows(
    state: &Arc<AppState>,
    search_id: i64,
    limit: i64,
) -> AppResult<Vec<FeedRow>> {
    let rows: Vec<FeedRow> = sqlx::query_as(
        r#"SELECT fi.id AS url_id,
                  w.canonical_title AS title,
                  COALESCE(w.canonical_author, fi.author) AS author,
                  fi.description,
                  fi.words,
                  fi.chapters,
                  fi.status,
                  m.first_seen,
                  m.work_id
           FROM saved_search_matches m
           JOIN works w ON w.id = m.work_id
           LEFT JOIN fic_info fi ON fi.work_id = m.work_id
           WHERE m.search_id = $1
             AND fi.id IS NOT NULL
           ORDER BY m.first_seen DESC, m.work_id DESC
           LIMIT $2"#,
    )
    .bind(search_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}
```

The `first_seen` timestamp in the `WHERE ORDER BY` clause ensures the feed shows the **newest matches first** — so when the watcher adds 3 new fics to your search, they appear at the top of the feed.

### Route registration for the feed

The feed route isn't in `server.rs` lines 378-383 — it's registered with the other RSS/OPDS routes. Let's find it:<tool_call>terminal<arg_key>command</arg_key><arg_value>grep -n "feed/saved\|saved_search_feed\|saved_search" /home/alvaro/code/rust/fichub/src/server.rs
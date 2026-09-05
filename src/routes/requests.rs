//! Fic Requests — prompt board for fic requests (works-only answers).
//!
//! A request = a prompt ("fics like X", "dark harry, no bashing"). Answers
//! are WORKS (work_id) with an optional one-line pitch. Community up/down
//! votes rank fit (net score only, never public downvote counts); the
//! requester can accept one answer → status 'answered'.
//!
//! Votes: one per user per answer, toggle/retract allowed. No self-vote.
//! Answers per user per request capped at 3 (ANSWER_CAP).

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Max answers one user may post on a single request (user-approved cap).
pub const ANSWER_CAP: i64 = 3;

#[derive(Debug, Deserialize)]
pub struct CreateRequestBody {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub seed_work_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsParams {
    #[serde(default = "default_status")]
    pub status: String, // open|answered|closed
    #[serde(default = "default_sort")]
    pub sort: String, // top|new
    #[serde(default = "default_page")]
    pub page: i64,
}

fn default_status() -> String {
    "open".into()
}
fn default_sort() -> String {
    "new".into()
}
fn default_page() -> i64 {
    1
}

#[derive(Debug, Deserialize)]
pub struct AnswerBody {
    #[serde(default)]
    pub work_id: Option<i32>,
    #[serde(default)]
    pub url_id: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub pitch: String,
    /// Search-kind answer: a raw search query string instead of a work.
    /// Renders as a "Try this search" card on the request page.
    #[serde(default)]
    pub search_query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub vote: i16, // 1 | -1 | 0 (0 = retract)
}

#[derive(Deserialize)]
pub struct UpvoteBody {
    pub enabled: bool,
}

/// POST /api/requests — create a request (auth)
pub async fn create_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRequestBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_string()));
    }
    if title.chars().count() > 200 {
        return Err(AppError::BadRequest("title too long (max 200)".to_string()));
    }
    if body.body.chars().count() > 4000 {
        return Err(AppError::BadRequest("body too long (max 4000)".to_string()));
    }
    if let Some(sid) = body.seed_work_id {
        if queries::get_work(&state.db, sid).await?.is_none() {
            return Err(AppError::BadRequest("seed_work_id not found".to_string()));
        }
    }

    let id: i32 = sqlx::query_scalar(
        r#"INSERT INTO fic_requests (user_id, title, body, seed_work_id)
           VALUES ($1, $2, $3, $4) RETURNING id"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(body.body.trim())
    .bind(body.seed_work_id)
    .fetch_one(&state.db)
    .await?;

    // ── Archivist LLM answer (best-effort, never blocks creation) ──────
    // Translate the request into search params via the Ask pipeline
    // (`/ask` itself is untouched) and store the result as the first
    // answer, attributed "FicNexus Archivist". Ollama down → plain-query
    // fallback; total failure of this block just means no auto-answer.
    let state2 = state.clone();
    let nl = format!("{} {}", title, body.body.trim());
    let req_id = id;
    tokio::spawn(async move {
        let v2 = crate::search::ask::translate_for_request(&state2, &nl).await;
        // Build a frontend search URL from the v2 query string — the /search
        // box parses the same syntax, so the query rides straight into q=.
        let url = format!("/search?q={}", urlencode(&v2));
        let pitch = format!(
            "Here's a live search built from your request — tweak the filters as you like: {}",
            v2
        );
        let _ = sqlx::query(
            r#"INSERT INTO fic_request_answers (request_id, user_id, work_id, pitch, source, answer_kind, search_query)
               SELECT $1, r.user_id, NULL, $2, 'archivist', 'llm', $3 FROM fic_requests r WHERE r.id = $1"#,
        )
        .bind(req_id)
        .bind(pitch)
        .bind(&url)
        .execute(&state2.db)
        .await;
    });

    Ok(Json(
        json!({ "err": 0, "id": id, "msg": "Request created" }),
    ))
}

/// Minimal percent-encoding for search URLs (keep it readable; only encode
/// characters that break query strings or paths).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push('+'),
            '&' | '#' | '%' | '+' | '?' | '=' | '"' | '\'' | '<' | '>' => {
                out.push_str(&format!("%{:02X}", c as u32))
            }
            _ => out.push(c),
        }
    }
    out
}

/// GET /api/requests?status=&sort=&page= — list requests (public)
pub async fn list_requests(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListRequestsParams>,
) -> Result<Json<Value>, AppError> {
    let status = match params.status.as_str() {
        "open" | "answered" | "closed" => params.status.as_str(),
        _ => {
            return Err(AppError::BadRequest(
                "status must be open|answered|closed".to_string(),
            ));
        }
    };
    let per_page = 20i64;
    let offset = (params.page.max(1) - 1) * per_page;

    let rows: Vec<(i32, String, String, Option<i32>, String, String, i64, i64)> = match params.sort.as_str() {
        "top" => sqlx::query_as(
            r#"SELECT r.id, r.title, r.body, r.seed_work_id, r.status,
                      r.created_at::text,
                      (SELECT COUNT(*) FROM fic_request_answers a WHERE a.request_id = r.id AND a.deleted_at IS NULL)::bigint,
                      (SELECT COUNT(*) FROM fic_request_upvotes u WHERE u.request_id = r.id)::bigint
               FROM fic_requests r
               WHERE r.deleted_at IS NULL AND r.status = $1
               ORDER BY (SELECT COUNT(*) FROM fic_request_upvotes u WHERE u.request_id = r.id) DESC,
                        (SELECT COUNT(*) FROM fic_request_answers a WHERE a.request_id = r.id AND a.deleted_at IS NULL) DESC,
                        r.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        "new" => sqlx::query_as(
            r#"SELECT r.id, r.title, r.body, r.seed_work_id, r.status,
                      r.created_at::text,
                      (SELECT COUNT(*) FROM fic_request_answers a WHERE a.request_id = r.id AND a.deleted_at IS NULL)::bigint,
                      (SELECT COUNT(*) FROM fic_request_upvotes u WHERE u.request_id = r.id)::bigint
               FROM fic_requests r
               WHERE r.deleted_at IS NULL AND r.status = $1
               ORDER BY r.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?,
        _ => return Err(AppError::BadRequest("sort must be top|new".to_string())),
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, title, body, seed, st, created, answers, upvotes)| {
            json!({ "id": id, "title": title, "body": body, "seed_work_id": seed,
                    "status": st, "created_at": created, "answer_count": answers,
                    "upvotes": upvotes })
        })
        .collect();

    Ok(Json(
        json!({ "err": 0, "items": items, "page": params.page, "status": status }),
    ))
}

/// GET /api/requests/{id} — detail + answers with net scores + my_vote (auth optional)
pub async fn get_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let req_row: Option<(
        i32,
        i32,
        String,
        String,
        Option<i32>,
        String,
        String,
        Option<i32>,
    )> = sqlx::query_as(
        r#"SELECT id, user_id, title, body, seed_work_id, status,
                      created_at::text, accepted_answer_id
               FROM fic_requests WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let (rid, owner_id, title, body, seed, status, created, accepted) =
        req_row.ok_or_else(|| AppError::BadRequest("request not found".to_string()))?;

    let owner: Option<(String,)> = sqlx::query_as("SELECT username FROM users WHERE id = $1")
        .bind(owner_id)
        .fetch_optional(&state.db)
        .await?;
    let owner_name = owner.map(|o| o.0).unwrap_or_default();

    let seed_fic: Option<(String, String)> = if let Some(sid) = seed {
        sqlx::query_as(
            "SELECT fi.title, fi.author FROM fic_info fi JOIN works w ON w.id = fi.work_id WHERE w.id = $1 LIMIT 1",
        )
        .bind(sid)
        .fetch_optional(&state.db)
        .await?
    } else {
        None
    };

    let my_user_id = auth.user_id;
    // M3: request upvotes (count + my upvote)
    let (upvote_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*)::bigint FROM fic_request_upvotes WHERE request_id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;
    let (my_upvote,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM fic_request_upvotes WHERE request_id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(my_user_id)
    .fetch_one(&state.db)
    .await?;

    let answers: Vec<(i32, i32, Option<i32>, String, String, String, Option<String>, i64, Option<i16>)> = sqlx::query_as(
        r#"SELECT a.id, a.user_id, a.work_id, a.pitch, a.source, a.created_at::text,
                  a.search_query,
                  (SELECT COALESCE(SUM(v.vote), 0) FROM fic_request_answer_votes v WHERE v.answer_id = a.id)::bigint,
                  (SELECT v.vote FROM fic_request_answer_votes v WHERE v.answer_id = a.id AND v.user_id = $2)
           FROM fic_request_answers a
           WHERE a.request_id = $1 AND a.deleted_at IS NULL
           ORDER BY 8 DESC, a.created_at ASC"#,
    )
    .bind(id)
    .bind(my_user_id)
    .fetch_all(&state.db)
    .await?;

    let mut ans_items: Vec<Value> = Vec::new();
    for (aid, auid, wid, pitch, source, created, search_query, score, my_vote) in answers {
        let (uname,): (String,) = sqlx::query_as("SELECT username FROM users WHERE id = $1")
            .bind(auid)
            .fetch_one(&state.db)
            .await?;
        // Work answers carry fic metadata; search/LLM answers carry the
        // query string instead.
        let (answer_kind, fic_title, fic_author) = if let Some(wid) = wid {
            let (t, a): (String, String) =
                sqlx::query_as("SELECT title, author FROM fic_info WHERE work_id = $1 LIMIT 1")
                    .bind(wid)
                    .fetch_one(&state.db)
                    .await?;
            ("work".to_string(), t, a)
        } else if source == "archivist" {
            (
                "llm".to_string(),
                String::new(),
                "FicNexus Archivist".to_string(),
            )
        } else {
            ("search".to_string(), String::new(), uname.clone())
        };
        ans_items.push(json!({
            "id": aid, "user_id": auid, "username": uname, "work_id": wid,
            "fic_title": fic_title, "fic_author": fic_author,
            "pitch": pitch, "source": source, "created_at": created,
            "score": score, "my_vote": my_vote,
            "accepted": accepted == Some(aid),
            "answer_kind": answer_kind, "search_query": search_query,
        }));
    }

    Ok(Json(json!({
        "err": 0,
        "request": {
            "id": rid, "user_id": owner_id, "username": owner_name,
            "title": title, "body": body, "seed_work_id": seed,
            "seed_fic": seed_fic.map(|(t, a)| json!({ "title": t, "author": a })),
            "status": status, "created_at": created, "accepted_answer_id": accepted,
            "upvotes": upvote_count, "my_upvote": my_upvote,
        },
        "answers": ans_items,
    })))
}

/// DELETE /api/requests/{id} — soft delete (owner or curator >= 5)
pub async fn delete_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let role = auth.role;
    let req_row: Option<(i32,)> =
        sqlx::query_as("SELECT user_id FROM fic_requests WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let owner = req_row.ok_or_else(|| AppError::BadRequest("request not found".to_string()))?;
    if owner.0 != user_id && role < 5 {
        return Err(AppError::Forbidden("Not allowed".to_string()));
    }
    sqlx::query("UPDATE fic_requests SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "err": 0, "msg": "Request deleted" })))
}

/// POST /api/requests/{id}/answers — add a work answer (auth, cap 3)
pub async fn add_answer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<AnswerBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let req: Option<(String,)> =
        sqlx::query_as("SELECT status FROM fic_requests WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let (status,) = req.ok_or_else(|| AppError::BadRequest("request not found".to_string()))?;
    if status != "open" {
        return Err(AppError::BadRequest("request is not open".to_string()));
    }

    // Resolve the answer target. Three kinds:
    //   work   — work_id | url_id | scraped URL (unchanged behaviour)
    //   search — `search_query` set: a "try this search" answer (no work)
    let search_answer = body
        .search_query
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    if search_answer.is_some()
        && (body.work_id.is_some() || body.url_id.is_some() || !body.url.trim().is_empty())
    {
        return Err(AppError::BadRequest(
            "search_query answers cannot also name a work".to_string(),
        ));
    }
    let work_id: i32 = if let Some(sq) = &search_answer {
        if sq.chars().count() > 300 {
            return Err(AppError::BadRequest(
                "search_query too long (max 300)".to_string(),
            ));
        }
        0 // unused placeholder for the search branch; not inserted
    } else if let Some(wid) = body.work_id {
        if queries::get_work(&state.db, wid).await?.is_none() {
            return Err(AppError::BadRequest("work_id not found".to_string()));
        }
        wid
    } else if let Some(url_id) = body.url_id.as_deref().filter(|s| !s.is_empty()) {
        // url_id is the fic_info.id (already in the library — ask results,
        // search results). Resolve to the linked work without scraping.
        let work_id: Option<i32> = sqlx::query_scalar(
            "SELECT work_id FROM fic_info WHERE id = $1 AND work_id IS NOT NULL",
        )
        .bind(url_id)
        .fetch_optional(&state.db)
        .await?;
        work_id.ok_or_else(|| AppError::BadRequest("url_id not found in library".to_string()))?
    } else {
        let url = body.url.trim();
        if url.is_empty() {
            return Err(AppError::BadRequest("work_id or url required".to_string()));
        }
        // Scrape the URL with the registry (prefers native scrapers) and
        // find-or-create the work. Blacklisted URLs are rejected.
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(url)
            .ok_or_else(|| AppError::BadRequest("Unsupported URL".to_string()))?;
        let meta = scraper
            .lookup(&state.http_client, url)
            .await
            .map_err(|e| AppError::BadRequest(format!("Could not fetch URL: {e}")))?;
        let blacklisted = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
        if !blacklisted.is_empty() {
            return Err(AppError::BadRequest("This fic is blacklisted".to_string()));
        }
        crate::works::find_or_create_work(&state.db, &meta)
            .await
            .map_err(|e| AppError::BadRequest(format!("Could not save fic: {e}")))?
            .work_id
    };

    if body.pitch.chars().count() > 500 {
        return Err(AppError::BadRequest("pitch too long (max 500)".to_string()));
    }

    // +3 cap per user per request
    let (cnt,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM fic_request_answers WHERE request_id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    if cnt >= ANSWER_CAP {
        return Err(AppError::BadRequest(format!(
            "Max {ANSWER_CAP} answers per request"
        )));
    }

    // Duplicate (UNIQUE(request_id, work_id)) → friendly error. Search
    // answers (work_id NULL) skip the work-duplicate path; the cap above
    // still limits spam.
    let aid: i32 = if let Some(sq) = &search_answer {
        match sqlx::query_scalar(
            r#"INSERT INTO fic_request_answers (request_id, user_id, work_id, pitch, answer_kind, search_query)
               VALUES ($1, $2, NULL, $3, 'search', $4) RETURNING id"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(body.pitch.trim())
        .bind(sq)
        .fetch_one(&state.db)
        .await
        {
            Ok(aid) => aid,
            Err(e) => return Err(e.into()),
        }
    } else {
        match sqlx::query_scalar(
            r#"INSERT INTO fic_request_answers (request_id, user_id, work_id, pitch)
               VALUES ($1, $2, $3, $4) RETURNING id"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(work_id)
        .bind(body.pitch.trim())
        .fetch_one(&state.db)
        .await
        {
            Ok(aid) => aid,
            Err(e) if e.to_string().contains("duplicate key") => {
                return Err(AppError::BadRequest(
                    "This fic is already an answer".to_string(),
                ));
            }
            Err(e) => return Err(e.into()),
        }
    };

    sqlx::query("UPDATE fic_requests SET updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    // M3: notify the requester that their request got a new answer
    // (best-effort; never fail the main action). Get the requester id.
    if let Ok(Some((owner_id,))) = sqlx::query_as::<_, (i32,)>(
        "SELECT user_id FROM fic_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        if owner_id != user_id {
            let _ = queries::create_notification(
                &state.db,
                owner_id,
                "request_answer",
                "Your fic request got a new answer",
                Some("Someone suggested a work for your request."),
                Some(&format!("/requests/{id}")),
                Some("request"),
                Some(&id.to_string()),
            )
            .await;
        }
    }

    Ok(Json(
        json!({ "err": 0, "answer_id": aid, "msg": "Answer added" }),
    ))
}

/// DELETE /api/requests/{id}/answers/{aid} — soft delete (owner or curator >= 5)
pub async fn delete_answer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, aid)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let role = auth.role;
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT user_id FROM fic_request_answers WHERE id = $1 AND request_id = $2 AND deleted_at IS NULL",
    )
    .bind(aid)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let owner = row.ok_or_else(|| AppError::BadRequest("answer not found".to_string()))?;
    if owner.0 != user_id && role < 5 {
        return Err(AppError::Forbidden("Not allowed".to_string()));
    }
    sqlx::query("UPDATE fic_request_answers SET deleted_at = NOW() WHERE id = $1")
        .bind(aid)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "err": 0, "msg": "Answer deleted" })))
}

/// POST /api/requests/{id}/answers/{aid}/vote — vote 1|-1|0 (toggle/retract, no self-vote)
pub async fn vote_answer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, aid)): Path<(i32, i32)>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    if !matches!(body.vote, -1 | 0 | 1) {
        return Err(AppError::BadRequest("vote must be -1, 0, or 1".to_string()));
    }
    let row: Option<(i32, i32)> = sqlx::query_as(
        "SELECT request_id, user_id FROM fic_request_answers WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(aid)
    .fetch_optional(&state.db)
    .await?;
    let (rid, answer_owner) =
        row.ok_or_else(|| AppError::BadRequest("answer not found".to_string()))?;
    if rid != id {
        return Err(AppError::BadRequest(
            "answer does not belong to this request".to_string(),
        ));
    }
    if answer_owner == user_id {
        return Err(AppError::BadRequest(
            "Cannot vote on your own answer".to_string(),
        ));
    }

    if body.vote == 0 {
        sqlx::query("DELETE FROM fic_request_answer_votes WHERE answer_id = $1 AND user_id = $2")
            .bind(aid)
            .bind(user_id)
            .execute(&state.db)
            .await?;
    } else {
        sqlx::query(
            r#"INSERT INTO fic_request_answer_votes (answer_id, user_id, vote)
               VALUES ($1, $2, $3)
               ON CONFLICT (answer_id, user_id) DO UPDATE SET vote = EXCLUDED.vote, created_at = NOW()"#,
        )
        .bind(aid)
        .bind(user_id)
        .bind(body.vote)
        .execute(&state.db)
        .await?;
    }

    let (score,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(vote), 0)::bigint FROM fic_request_answer_votes WHERE answer_id = $1",
    )
    .bind(aid)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "score": score, "my_vote": if body.vote == 0 { Value::Null } else { json!(body.vote) } }),
    ))
}

/// POST /api/requests/{id}/upvote — request-level upvote (toggle). No self-vote.
/// One upvote per user per request (PRIMARY KEY (request_id, user_id)).
/// `enabled: true` upvotes, `enabled: false` removes the upvote.
pub async fn upvote_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(body): Json<UpvoteBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT user_id FROM fic_requests WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let owner = row.ok_or_else(|| AppError::BadRequest("request not found".to_string()))?;
    if owner.0 == user_id {
        return Err(AppError::BadRequest(
            "Cannot upvote your own request".to_string(),
        ));
    }

    if body.enabled {
        sqlx::query(
            r#"INSERT INTO fic_request_upvotes (request_id, user_id) VALUES ($1, $2)
               ON CONFLICT (request_id, user_id) DO NOTHING"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    } else {
        sqlx::query("DELETE FROM fic_request_upvotes WHERE request_id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&state.db)
            .await?;
    }

    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*)::bigint FROM fic_request_upvotes WHERE request_id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;
    let (mine,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM fic_request_upvotes WHERE request_id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "upvotes": count, "my_upvote": mine }),
    ))
}

/// POST /api/requests/{id}/accept/{aid} — requester only → status='answered'
pub async fn accept_answer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, aid)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let req: Option<(i32, String)> = sqlx::query_as(
        "SELECT user_id, status FROM fic_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let (owner, status) =
        req.ok_or_else(|| AppError::BadRequest("request not found".to_string()))?;
    if owner != user_id {
        return Err(AppError::Forbidden(
            "Only the requester can accept".to_string(),
        ));
    }
    if status == "closed" {
        return Err(AppError::BadRequest("request is closed".to_string()));
    }
    let ans: Option<(i32,)> = sqlx::query_as(
        "SELECT id FROM fic_request_answers WHERE id = $1 AND request_id = $2 AND deleted_at IS NULL",
    )
    .bind(aid)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    if ans.is_none() {
        return Err(AppError::BadRequest("answer not found".to_string()));
    }

    sqlx::query(
        r#"UPDATE fic_requests SET status = 'answered', accepted_answer_id = $2, closed_at = NOW(), updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(aid)
    .execute(&state.db)
    .await?;

    // M3: notify the answer author that their answer was accepted
    // (best-effort; never fail the main action).
    if let Ok(Some((answer_owner,))) =
        sqlx::query_as::<_, (i32,)>("SELECT user_id FROM fic_request_answers WHERE id = $1")
            .bind(aid)
            .fetch_optional(&state.db)
            .await
    {
        if answer_owner != user_id {
            let _ = queries::create_notification(
                &state.db,
                answer_owner,
                "request_accepted",
                "Your answer was accepted!",
                Some("The requester accepted your suggested work."),
                Some(&format!("/requests/{id}")),
                Some("request"),
                Some(&id.to_string()),
            )
            .await;
        }
    }

    Ok(Json(
        json!({ "err": 0, "msg": "Answer accepted", "status": "answered" }),
    ))
}

/// GET /api/requests/{id}/candidates — engine suggestions for answering a
/// request. When the request has a seed work, recommend works similar to it
/// (via the recommender engine); otherwise return an empty list (a future
/// search-based path can be added).
pub async fn candidates(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    crate::modlog::require_logged_in(&auth)?;

    // Load the request's seed work (if any).
    let seed: Option<i32> = sqlx::query_scalar(
        "SELECT seed_work_id FROM fic_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    if seed.is_none() {
        return Ok(Json(
            json!({ "err": 0, "request_id": id, "candidates": [] }),
        ));
    }
    let seed = seed.unwrap();

    // Find the seed work's canonical url_id (any source).
    let url_id: Option<String> = sqlx::query_scalar(
        r#"SELECT fi.id AS url_id
           FROM fic_info fi
           WHERE fi.work_id = $1
           ORDER BY fi.id
           LIMIT 1"#,
    )
    .bind(seed)
    .fetch_optional(&state.db)
    .await?;

    let Some(url_id) = url_id else {
        return Ok(Json(
            json!({ "err": 0, "request_id": id, "candidates": [] }),
        ));
    };

    // Ask the recommender engine for similar works.
    let rec_query = crate::recommender::RecQuery {
        url_id,
        n: 20,
        site_domain: None,
    };
    let recs = state
        .recommender_engine
        .get_recommendations(&rec_query, &state.config)
        .await?;

    let candidates: Vec<Value> = recs
        .into_iter()
        .map(|r| {
            json!({
                "url_id": r.url_id,
                "title": r.title,
                "author": r.author,
                "score": r.score,
            })
        })
        .collect();

    Ok(Json(
        json!({ "err": 0, "request_id": id, "candidates": candidates }),
    ))
}

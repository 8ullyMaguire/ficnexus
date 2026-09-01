# Part 24 — Work Proposals and Fic Suggestions

> In this chapter you will learn how FicHub's community suggestion system works — users propose new works for the archive to scrape, others vote on proposals, and the system manages both work proposals and fic suggestions with support/vote totals and curation tools.

---

## Overview

FicHub has two suggestion systems:

- **Work proposals** (`src/routes/work_proposals.rs`, 533 lines) — users propose a URL for FicHub to scrape. Each proposal has support/vote counts and is either pending, accepted, or rejected.
- **Fic suggestions** (`src/routes/fic_suggestions.rs`, 713 lines) — a more general system for suggesting specific fics to add. Also has support + votes, and a suggested-by relationship.

Both systems feed into the curation pipeline — accepted proposals trigger scrapes, and suggestions can be accepted or rejected by curators.

---

## Chapter 24.1 — Work Proposals

### Goal

Build the work proposal system — users submit URLs they want FicHub to scrape, and the community votes on proposals.

### Actions

#### 1. The work proposal table and model

```rust
// src/routes/work_proposals.rs (533 lines) — full work proposals system

// The work_proposals table:
// CREATE TABLE work_proposals (
//     id          SERIAL PRIMARY KEY,
//     url         TEXT NOT NULL UNIQUE,       -- the URL being proposed
//     proposed_by INTEGER NOT NULL REFERENCES users(id),
//     support     INTEGER NOT NULL DEFAULT 0,  -- upvotes
//     oppose     INTEGER NOT NULL DEFAULT 0,  -- downvotes
//     status      TEXT NOT NULL DEFAULT 'pending', -- pending | accepted | rejected
//     curator_id  INTEGER REFERENCES users(id),    -- who curated it
//     note        TEXT,                           -- curator's note
//     created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//     updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
// );

#[derive(Debug, Deserialize)]
pub struct CreateProposalBody {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteProposalBody {
    pub support: bool,   // true = support, false = oppose
}
```

#### 2. Create a proposal

```rust
// POST /api/work-proposals
pub async fn create_proposal(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateProposalBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate URL
    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".into()));
    }
    if !url_is_fic(body.url) {
        return Err(AppError::BadRequest("URL must be a fanfiction URL".into()));
    }

    // Upsert — same user + same URL = update their existing proposal
    // (but still creates if it's a new URL they haven't proposed before)
    let proposal = sqlx::query_as::<_, Proposal>(
        r#"
        INSERT INTO work_proposals (url, proposed_by, support, oppose, status)
        VALUES ($1, $2, 1, 0, 'pending')
        ON CONFLICT (url, proposed_by) DO UPDATE
        SET support = work_proposals.support + 1,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(&body.url)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "proposal": proposal_to_json(&proposal),
    }))
}
```

> **⚠️ Watch Out**: The upsert uses `ON CONFLICT (url, proposed_by) DO UPDATE` — same user proposing the same URL again is treated as a re-support (increments support). Different users proposing the same URL each get their own row (unique on (url, proposed_by)). The `url_is_fic` check validates the URL is a recognized fanfiction site.

#### 3. List and get proposals

```rust
// GET /api/work-proposals — list all proposals (paginated, sorted by support desc)
pub async fn list_proposals(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, Proposal>(
        r#"
        SELECT id, url, proposed_by, support, oppose, status, curator_id, note,
               created_at, updated_at
        FROM work_proposals
        ORDER BY support DESC, created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM work_proposals")
        .fetch_one(&state.db)
        .await?;

    Ok(json!({
        "err": 0,
        "proposals": rows.into_iter().map(proposal_to_json).collect::<Vec<_>>(),
        "total": total.0,
        "page": params.page.unwrap_or(1),
        "limit": limit,
    }))
}
```

#### 4. Vote on a proposal

```rust
// POST /api/work-proposals/{id}/vote
pub async fn vote_on_proposal(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(proposal_id): Path<i64>,
    Json(body): Json<VoteProposalBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Check if user already voted (can change their vote)
    let existing: Option<(bool,)> = sqlx::query_as(
        "SELECT support FROM proposal_votes WHERE proposal_id = $1 AND user_id = $2"
    )
    .bind(proposal_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    // Upsert vote
    sqlx::query(
        r#"
        INSERT INTO proposal_votes (proposal_id, user_id, support)
        VALUES ($1, $2, $3)
        ON CONFLICT (proposal_id, user_id) DO UPDATE SET support = $3
        "#,
    )
    .bind(proposal_id)
    .bind(user_id)
    .bind(body.support)
    .execute(&state.db)
    .await?;

    // Update the proposal's support/oppose counts
    if body.support {
        sqlx::query(
            "UPDATE work_proposals SET support = support + 1 WHERE id = $1"
        )
        .bind(proposal_id)
        .execute(&state.db)
        .await?;
        if existing.map(|v| !v.0).unwrap_or(false) {
            // changing from oppose to support
            sqlx::query(
                "UPDATE work_proposals SET oppose = oppose - 1 WHERE id = $1"
            )
            .bind(proposal_id)
            .execute(&state.db)
            .await?;
        }
    } else {
        sqlx::query(
            "UPDATE work_proposals SET oppose = oppose + 1 WHERE id = $1"
        )
        .bind(proposal_id)
        .execute(&state.db)
        .await?;
        if existing.map(|v| v.0).unwrap_or(false) {
            // changing from support to oppose
            sqlx::query(
                "UPDATE work_proposals SET support = support - 1 WHERE id = $1"
            )
            .bind(proposal_id)
            .execute(&state.db)
            .await?;
        }
    }

    let proposal = sqlx::query_as::<_, Proposal>("SELECT * FROM work_proposals WHERE id = $1")
        .bind(proposal_id)
        .fetch_one(&state.db)
        .await?;

    Ok(json!({
        "err": 0,
        "proposal": proposal_to_json(&proposal),
    }))
}
```

> **💡 Key Concept**: The vote system allows users to **change their vote** — the upsert on `proposal_votes` means re-voting updates the existing row. The support/oppose counts on `work_proposals` are adjusted accordingly (if changing from oppose to support, both counts change).

---

## Chapter 24.2 — Fic Suggestions

### Goal

Build the fic suggestions system — a more general suggestion mechanism that supports both work proposals and fic suggestions with voting.

### Actions

#### 1. The fic_suggestions table and model

```rust
// src/routes/fic_suggestions.rs (713 lines) — full fic suggestions system

// CREATE TABLE fic_suggestions (
//     id          SERIAL PRIMARY KEY,
//     url         TEXT NOT NULL UNIQUE,
//     proposed_by INTEGER NOT NULL REFERENCES users(id),
//     support     INTEGER NOT NULL DEFAULT 1,
//     oppose     INTEGER NOT NULL DEFAULT 0,
//     status      TEXT NOT NULL DEFAULT 'pending',
//     curator_id  INTEGER REFERENCES users(id),
//     note        TEXT,
//     created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//     updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
// );

#[derive(Debug, Deserialize)]
pub struct CreateSuggestionBody {
    pub url: String,
    #[serde(default)]
    pub note: Option<String>,
}
```

#### 2. Create a suggestion

```rust
// POST /api/fic-suggestions
pub async fn create_suggestion(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSuggestionBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".into()));
    }

    // Validate URL — must be a known fic site
    let source_id = get_source_id(&state.db, &body.url).await
        .ok_or_else(|| AppError::BadRequest("URL must be from a supported site".into()))?;

    // Check if already exists in fic_info
    let existing_fic: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM fic_info WHERE LOWER(url) = LOWER($1)"
    )
    .bind(&body.url)
    .fetch_optional(&state.db)
    .await?;

    if existing_fic.is_some() {
        return Err(AppError::BadRequest("This fic is already in the archive".into()));
    }

    // Upsert suggestion
    let suggestion = sqlx::query_as::<_, Suggestion>(
        r#"
        INSERT INTO fic_suggestions (url, proposed_by, support, oppose, status, note)
        VALUES ($1, $2, 1, 0, 'pending', $3)
        ON CONFLICT (url, proposed_by) DO UPDATE
        SET support = fic_suggestions.support + 1,
            note = COALESCE(EXCLUDED.note, fic_suggestions.note),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(&body.url)
    .bind(user_id)
    .bind(&body.note)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "suggestion": suggestion_to_json(&suggestion),
    }))
}
```

> **⚠️ Watch Out**: Fic suggestions validate that the URL is from a known site (`get_source_id`) and that the fic isn't already in the archive (`fic_info` lookup). Work proposals don't do this — they're just URL suggestions. Fic suggestions are more strict because they target specific fics.

#### 3. List and vote on suggestions

```rust
// GET /api/fic-suggestions — list all suggestions (paginated)
pub async fn list_suggestions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, Suggestion>(
        r#"
        SELECT id, url, proposed_by, support, oppose, status, curator_id, note,
               created_at, updated_at
        FROM fic_suggestions
        ORDER BY support DESC, created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_suggestions")
        .fetch_one(&state.db)
        .await?;

    Ok(json!({
        "err": 0,
        "suggestions": rows.into_iter().map(suggestion_to_json).collect::<Vec<_>>(),
        "total": total.0,
    }))
}

// POST /api/fic-suggestions/{id}/vote
pub async fn vote_on_suggestion(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(suggestion_id): Path<i64>,
    Json(body): Json<VoteSuggestionBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Same upsert-vote pattern as work proposals
    sqlx::query(
        r#"
        INSERT INTO suggestion_votes (suggestion_id, user_id, support)
        VALUES ($1, $2, $3)
        ON CONFLICT (suggestion_id, user_id) DO UPDATE SET support = $3
        "#,
    )
    .bind(suggestion_id)
    .bind(user_id)
    .bind(body.support)
    .execute(&state.db)
    .await?;

    // Adjust support/oppose counts
    if body.support {
        sqlx::query("UPDATE fic_suggestions SET support = support + 1 WHERE id = $1")
            .bind(suggestion_id)
            .execute(&state.db)
            .await?;
    } else {
        sqlx::query("UPDATE fic_suggestions SET oppose = oppose + 1 WHERE id = $1")
            .bind(suggestion_id)
            .execute(&state.db)
            .await?;
    }

    let suggestion = sqlx::query_as::<_, Suggestion>("SELECT * FROM fic_suggestions WHERE id = $1")
        .bind(suggestion_id)
        .fetch_one(&state.db)
        .await?;

    Ok(json!({
        "err": 0,
        "suggestion": suggestion_to_json(&suggestion),
    }))
}
```

#### 4. Curate suggestions (accept/reject)

```rust
// PATCH /api/fic-suggestions/{id} — curate (accept/reject)
pub async fn curate_suggestion(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(suggestion_id): Path<i64>,
    Json(body): Json<CurateSuggestionBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.status.as_deref() != Some("accepted") && body.status.as_deref() != Some("rejected") {
        return Err(AppError::BadRequest("status must be 'accepted' or 'rejected'".into()));
    }

    let suggestion = sqlx::query_as::<_, Suggestion>(
        "SELECT * FROM fic_suggestions WHERE id = $1"
    )
    .bind(suggestion_id)
    .fetch_one(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Suggestion not found".into()))?;

    // Update status
    sqlx::query(
        r#"
        UPDATE fic_suggestions
        SET status = $1, curator_id = $2, note = COALESCE($3, note), updated_at = NOW()
        WHERE id = $4
        "#,
    )
    .bind(&body.status)
    .bind(user_id)
    .bind(&body.note)
    .bind(suggestion_id)
    .execute(&state.db)
    .await?;

    // If accepted, queue for scraping
    if body.status == "accepted" {
        scrape_queue::queue_suggestion_scrape(&state.db, &state.redis, &suggestion.url).await?;
    }

    Ok(json!({
        "err": 0,
        "suggestion": suggestion_to_json(
            &sqlx::query_as::<_, Suggestion>("SELECT * FROM fic_suggestions WHERE id = $1")
                .bind(suggestion_id)
                .fetch_one(&state.db)
                .await?,
        ),
    }))
}
```

---

## Chapter 24.3 — Frontend Integration

### Goal

Build the suggestions/proposals frontend pages — list view, proposal cards, vote buttons, and curator accept/reject controls.

### Actions

#### 1. The work-proposals frontend page

```svelte
<!-- frontend/src/routes/work-proposals/+page.svelte (460+ lines) -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { auth } from '$lib/stores/auth.svelte';
    import { t } from '$lib/i18n/index.svelte';

    interface Proposal {
        id: number;
        url: string;
        support: number;
        oppose: number;
        status: 'pending' | 'accepted' | 'rejected';
        proposed_by: number;
        curator_id: number | null;
        created_at: string;
    }

    let proposals = $state<Proposal[]>([]);
    let loading = $state(true);
    let error = $state('');

    onMount(async () => {
        await auth.init();
        if (!auth.isLoggedIn) { goto('/'); return; }
        await loadProposals();
    });

    async function loadProposals() {
        loading = true; error = '';
        try {
            const res = await fetch('/api/work-proposals?limit=50');
            const data = await res.json();
            if (data.err !== 0) { error = 'Failed to load proposals.'; return; }
            proposals = data.proposals;
        } catch { error = 'Network error.'; }
        finally { loading = false; }
    }

    async function handleVote(proposalId: number, support: boolean) {
        const res = await fetch(`/api/work-proposals/${proposalId}/vote`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', ...auth.headers() },
            body: JSON.stringify({ support }),
        });
        const data = await res.json();
        if (data.err === 0) {
            // Update local state
            const idx = proposals.findIndex(p => p.id === proposalId);
            if (idx >= 0) {
                proposals[idx] = data.proposal;
            }
        }
    }

    async function handleCurate(proposalId: number, status: 'accepted' | 'rejected') {
        const res = await fetch(`/api/work-proposals/${proposalId}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json', ...auth.headers() },
            body: JSON.stringify({ status }),
        });
        const data = await res.json();
        if (data.err === 0) {
            const idx = proposals.findIndex(p => p.id === proposalId);
            if (idx >= 0) proposals[idx] = data.proposal;
        }
    }
</script>

<div class="proposals-page">
    <h1>Works Proposed for Addition</h1>
    {#if loading}
        <p class="loading">Loading...</p>
    {:else if error}
        <p class="error">{error}</p>
    {:else if proposals.length === 0}
        <p>No proposals yet.</p>
    {:else}
        <ul class="proposal-list">
            {#each proposals as proposal (proposal.id)}
                <li class="proposal-card">
                    <div class="proposal-header">
                        <span class="status badge">{proposal.status}</span>
                        <a href="{proposal.url}" target="_blank" rel="noopener">
                            {new URL(proposal.url).hostname}
                        </a>
                    </div>
                    <div class="proposal-meta">
                        <span>By user #{proposal.proposed_by}</span>
                        <span>Created {new Date(proposal.created_at).toLocaleDateString()}</span>
                    </div>
                    <div class="proposal-votes">
                        <button onclick={() => handleVote(proposal.id, true)}
                            class:active={proposal.support > 0}>
                            👍 {proposal.support}
                        </button>
                        <button onclick={() => handleVote(proposal.id, false)}
                            class:active={proposal.oppose > 0}>
                            👎 {proposal.oppose}
                        </button>
                    </div>
                    {#if proposal.status === 'pending'}
                        <div class="curator-actions">
                            <button onclick={() => handleCurate(proposal.id, 'accepted')}>Accept</button>
                            <button onclick={() => handleCurate(proposal.id, 'rejected')}>Reject</button>
                        </div>
                    {/if}
                </li>
            {/each}
        </ul>
    {/if}
</div>
```

#### 2. The fic-suggestions frontend page

```svelte
<!-- frontend/src/routes/fic-suggestions/+page.svelte -->
<script lang="ts">
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth.svelte';
    import { t } from '$lib/i18n/index.svelte';

    interface Suggestion {
        id: number;
        url: string;
        support: number;
        oppose: number;
        status: 'pending' | 'accepted' | 'rejected';
        proposed_by: number;
        note: string | null;
    }

    let suggestions = $state<Suggestion[]>([]);
    let loading = $state(true);

    onMount(async () => {
        await auth.init();
        await loadSuggestions();
    });

    async function loadSuggestions() {
        loading = true;
        try {
            const res = await fetch('/api/fic-suggestions?limit=50');
            const data = await res.json();
            if (data.err === 0) suggestions = data.suggestions;
        } finally { loading = false; }
    }
</script>

<div class="suggestions-page">
    <h1>Fics Suggested for Addition</h1>
    {#each suggestions as suggestion (suggestion.id)}
        <div class="suggestion-card">
            <a href="{suggestion.url}" target="_blank">{new URL(suggestion.url).hostname}</a>
            <div class="suggestion-stats">
                👍 {suggestion.support} · 👎 {suggestion.oppose}
                <span class="status">{suggestion.status}</span>
            </div>
            {#if suggestion.note}
                <p class="note">{suggestion.note}</p>
            {/if}
        </div>
    {/each}
</div>
```

### Try It Yourself

```bash
# Create a work proposal
curl -X POST /api/work-proposals -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://archiveofourown.org/works/12345"}'

# Vote on a proposal (support)
curl -X POST /api/work-proposals/1/vote -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" -d '{"support": true}'

# Curate a proposal (curator/admin only)
curl -X PATCH /api/work-proposals/1 -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" -d '{"status": "accepted"}'
```

### Check

- ✅ Work proposals have `url`, `support`, `oppose`, `status`, `proposed_by`, `curator_id`, `note`.
- ✅ Fic suggestions validate URL is from a known site and not already in archive.
- ✅ Both systems use upsert voting (change your vote = update existing row).
- ✅ Curators can accept/reject proposals and suggestions.
- ✅ Accepted fic suggestions trigger a scrape queue.
- ✅ Frontend shows vote buttons, status badges, and curator controls for pending items.

### What you built

The dual suggestion system — work proposals (URL suggestions for scraping) and fic suggestions (specific fic suggestions with stricter validation). Both with upsert voting, support/oppose counts, status management, and curator accept/reject tools. Plus the frontend pages showing proposal/suggestion cards with vote buttons and curator controls.

---

## Conclusion

You now understand FicHub's suggestion systems:

1. **Work proposals** — URL suggestions for archiving, community voting (support/oppose), status (pending/accepted/rejected), curator curation.
2. **Fic suggestions** — specific fic suggestions with URL validation (known site, not already in archive), same voting pattern, curator accept/reject.
3. **Voting** — upsert pattern allows vote changes, support/oppose counts adjust accordingly.
4. **Curation** — accepted proposals/suggestions trigger scrape queue, rejected ones are closed.
5. **Frontend** — proposal/suggestion cards with vote buttons, status badges, and curator accept/reject controls for pending items.

Both systems feed the archive growth pipeline — community members propose URLs or fics, vote on them, and curators decide what gets scraped and added to FicHub.

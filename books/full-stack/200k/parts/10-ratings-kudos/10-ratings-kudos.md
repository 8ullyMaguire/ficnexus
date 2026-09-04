# Part 10 — Ratings and Kudos

> In this chapter you will learn how FicHub rates fics on a 5-star scale, gives them kudos (one-click thumbs-up), and how both feed into search and the recommendation engine.

---

## Overview

FicHub has two rating systems that work together:

- **5-star ratings** — Users rate a fic from 1 to 5 stars. Ratings are stored in `fic_ratings` table with `rating` (1-5) and `user_id`. Averages are computed from the `fi_summary` materialized view.
- **Kudos** — One-click "thumbs up." Stored in `fic_kudos` table (one per user per fic). Kudos count is separate from ratings and appears as a badge on the fic card.

Both systems feed into `fi_summary` (the aggregated view), which powers search results and the `/api/v1/related` recommendation endpoint.

---

## Chapter 10.1 — 5-Star Ratings

### Goal

Build the star rating system. Users rate a fic 1-5 stars. Average and distribution are computed from the `fic_ratings` table and exposed via `fi_summary`.

### Actions

#### 1. The rating table

```sql
-- migrations/001_initial.sql
CREATE TABLE fic_ratings (
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER REFERENCES users(id) ON DELETE CASCADE,
    work_id    INTEGER REFERENCES works(id) ON DELETE CASCADE,
    rating     INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, work_id)
);
```

#### 2. The rating request body

```rust
// src/routes/ratings.rs (lines 1-30)
#[derive(Debug, Deserialize)]
pub struct RateWorkBody {
    pub work_id: i32,
    pub rating: i32,  // 1-5
}
```

#### 3. POST /api/v1/ratings

```rust
// src/routes/ratings.rs (lines 30-60)
pub async fn rate_work(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RateWorkBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    if body.rating < 1 || body.rating > 5 {
        return Err(AppError::BadRequest("Rating must be 1-5".into()));
    }

    // UPSERT — change your rating
    sqlx::query(
        r#"
        INSERT INTO fic_ratings (user_id, work_id, rating)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, work_id) DO UPDATE SET rating = $3
        "#,
    )
    .bind(user_id)
    .bind(body.work_id)
    .bind(body.rating)
    .execute(&state.db)
    .await?;

    // Return updated aggregate
    let avg: Option<(f64,)> = sqlx::query_as(
        "SELECT COALESCE(AVG(rating), 0) FROM fic_ratings WHERE work_id = $1"
    )
    .bind(body.work_id)
    .fetch_one(&state.db)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_ratings WHERE work_id = $1"
    )
    .bind(body.work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "work_id": body.work_id,
        "avg_rating": avg.map(|a| a.0).unwrap_or(0.0),
        "rating_count": count.0,
    }))
}
```

> **⚠️ Watch Out**: Ratings use UPSERT (`ON CONFLICT (user_id, work_id) DO UPDATE`). This means a user can change their rating — the old rating is replaced. Ratings are **not** vote-like (can't accumulate). They're a 1-to-1 relationship: one rating per user per work.

> **💡 Key Concept**: The rating `avg_rating` is computed live from `fic_ratings` using `AVG`. In production, this would be cached in `fi_summary` (the materialized view), but for the tutorial we compute it live to show the SQL pattern.

#### 4. GET /api/v1/ratings/{work_id}

```rust
// GET /api/v1/ratings/{work_id}
pub async fn get_ratings(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let (avg, count): (Option<f64>, i64) = sqlx::query_as(
        "SELECT COALESCE(AVG(rating), 0), COUNT(*) FROM fic_ratings WHERE work_id = $1"
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "avg_rating": avg.unwrap_or(0.0),
        "rating_count": count,
    }))
}
```

#### 5. The fi_summary materialized view

```sql
-- migrations/001_initial.sql (excerpt)
CREATE VIEW fi_summary AS
SELECT
    fi.id AS url_id,
    fi.title,
    fi.author,
    fi.words,
    fi.chapters,
    fi.status,
    fi.source,
    fi.created,
    fi.description,
    COUNT(DISTINCT w.id) AS works_count,
    COUNT(DISTINCT r.user_id) AS rating_count,
    COALESCE(AVG(r.rating), 0)::numeric(3,2) AS avg_rating,
    COUNT(DISTINCT k.user_id) AS kudos_count,
    COUNT(DISTINCT m.user_id) AS bookmark_count
FROM fic_info fi
LEFT JOIN works w ON w.url_id = fi.id
LEFT JOIN fic_ratings r ON r.work_id = w.id
LEFT JOIN fic_kudos k ON k.work_id = w.id
LEFT JOIN fic_bookmarks m ON m.work_id = w.id
GROUP BY fi.id;
```

This view is what powers `GET /api/works/{id}` and the search results. It joins `fic_info → works → ratings/kudos/bookmarks` to produce the aggregated view.

### Try It Yourself

```bash
# Rate a fic 5 stars
curl -X POST /api/v1/ratings \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"work_id": 42, "rating": 5}'

# Get the average
curl /api/v1/ratings/42
```

### Check

- ✅ Rating 1-5 enforced in handler and CHECK constraint.
- ✅ UPSERT — user can change rating.
- ✅ `UNIQUE (user_id, work_id)` prevents duplicate ratings.
- ✅ `AVG` computed live; production would use `fi_summary`.
- ✅ `GET /api/v1/ratings/{work_id}` returns current avg + count.

### What you built

The 5-star rating system — `fic_ratings` table, POST rating (UPSERT), GET aggregate. The rating data feeds into `fi_summary` which powers the work page and search.

---

## Chapter 10.2 — Kudos

### Goal

Build the one-click kudos system. Users give a fic a thumbs-up. Kudos are separate from ratings and appear as a badge.

### Actions

#### 1. The kudos table

```sql
-- migrations/001_initial.sql
CREATE TABLE fic_kudos (
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER REFERENCES users(id) ON DELETE CASCADE,
    work_id    INTEGER REFERENCES works(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, work_id)
);
```

#### 2. POST /api/v1/kudos/{work_id}

```rust
// src/routes/kudos.rs (lines 1-60)
pub async fn give_kudos(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    sqlx::query(
        r#"
        INSERT INTO fic_kudos (user_id, work_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, work_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(work_id)
    .execute(&state.db)
    .await?;

    // Return current kudos count
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_kudos WHERE work_id = $1"
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "kudos_count": count.0,
    }))
}
```

#### 3. DELETE /api/v1/kudos/{work_id}

```rust
// DELETE /api/v1/kudos/{work_id}
pub async fn remove_kudos(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    sqlx::query(
        "DELETE FROM fic_kudos WHERE user_id = $1 AND work_id = $2"
    )
    .bind(user_id)
    .bind(work_id)
    .execute(&state.db)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_kudos WHERE work_id = $1"
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "kudos_count": count.0,
    }))
}
```

#### 4. GET /api/v1/kudos/{work_id}

```rust
// GET /api/v1/kudos/{work_id}
pub async fn get_kudos(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_kudos WHERE work_id = $1"
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "kudos_count": count.0,
    }))
}
```

### Try It Yourself

```bash
# Give kudos
curl -X POST /api/v1/kudos/42 -H "Authorization: Bearer <token>"

# Remove kudos
curl -X DELETE /api/v1/kudos/42 -H "Authorization: Bearer <token>"

# Get kudos count
curl /api/v1/kudos/42
```

### Check

- ✅ `UNIQUE (user_id, work_id)` — one kudos per user per work.
- ✅ `ON CONFLICT (user_id, work_id) DO NOTHING` — idempotent (can't double-kudos).
- ✅ DELETE removes user's kudos.
- ✅ Kudos count in `fi_summary` (`COUNT(DISTINCT k.user_id)`).

### What you built

The kudos system — one-click thumbs-up, `fic_kudos` table, POST/DELETE, live count, feeds into `fi_summary`.

---

## Chapter 10.3 — Ratings + Kudos in fi_summary

### Goal

Understand how ratings and kudos feed into the aggregated view that powers search and the work page.

### Actions

```sql
-- The fi_summary view joins fic_info → works → ratings/kudos/bookmarks
CREATE VIEW fi_summary AS
SELECT
    fi.id AS url_id,
    fi.title,
    fi.author,
    fi.words,
    fi.chapters,
    fi.status,
    fi.source,
    fi.created,
    fi.description,
    COUNT(DISTINCT w.id) AS works_count,
    COUNT(DISTINCT r.user_id) AS rating_count,
    COALESCE(AVG(r.rating), 0)::numeric(3,2) AS avg_rating,
    COUNT(DISTINCT k.user_id) AS kudos_count,
    COUNT(DISTINCT m.user_id) AS bookmark_count
FROM fic_info fi
LEFT JOIN works w ON w.url_id = fi.id
LEFT JOIN fic_ratings r ON r.work_id = w.id
LEFT JOIN fic_kudos k ON k.work_id = w.id
LEFT JOIN fic_bookmarks m ON m.work_id = w.id
GROUP BY fi.id;
```

This view is the "aggregated fic card" — every search result and work page fetches from this view.

> **💡 Key Concept**: `fi_summary` is a **view**, not a table. It's computed live from the base tables. In a high-traffic system, you'd refresh a materialized view periodically, but for the tutorial the live view is fine — it shows the SQL pattern.

### Try It Yourself

```bash
# Query fi_summary directly
curl /api/v1/works/42

# The response is built from fi_summary:
# { "url_id": "abc123", "title": "...", "avg_rating": 4.2, "kudos_count": 15, ... }
```

### Check

- ✅ `fi_summary` joins `fic_info → works → ratings/kudos/bookmarks`.
- ✅ `AVG(r.rating)` computed from `fic_ratings`.
- ✅ `COUNT(DISTINCT k.user_id)` for kudos (not raw row count).
- ✅ View is live, not materialized (for tutorial simplicity).

### What you built

The `fi_summary` view that powers the work card — joining `fic_info → works → ratings/kudos/bookmarks`, computing avg_rating and kudos_count live.

---

## Chapter 10.4 — Star Rating Widget (Frontend)

### Goal

Build the star rating widget that appears on the work page — 5 clickable stars, shows current rating, updates on click.

### Actions

#### 1. The StarRating Svelte component

```svelte
<!-- frontend/src/lib/components/StarRating.svelte -->
<script lang="ts">
    import { api } from '$lib/api/social';

    interface Props {
        workId: number;
        currentRating?: number;
        onRate?: (rating: number) => void;
    }

    let { workId, currentRating = 0, onRate }: Props = $props();

    let rating = $state(currentRating);

    function setRating(r: number) {
        rating = r;
        onRate?.(r);
    }
</script>

<div class="star-rating">
    {#each [1, 2, 3, 4, 5] as star}
        <button
            class="star { rating >= star ? 'filled' : '' }"
            onclick={() => setRating(star)}
            title="{star} star{rating === 1 ? '' : 's'}"
        >
            &#9733;
        </button>
    {/each}
    {#if rating === 0}
        <span class="hint">Rate this fic</span>
    {:else}
        <span class="rating-display">{rating} / 5</span>
    {/if}
</div>

<style>
.star-rating { display: flex; gap: 4px; }
.star {
    background: none; border: none; font-size: 2rem; cursor: pointer;
    color: #ccc; padding: 0; transition: color 0.1s;
}
.star.filled { color: #f5c518; }
.rating-display { margin-left: 8px; color: #888; font-size: 0.9rem; }
.hint { font-size: 0.85rem; color: #999; }
</style>
```

#### 2. Using it on the work page

```svelte
<!-- frontend/src/routes/work/[workId]/+page.svelte (rating section) -->
<script lang="ts">
    import StarRating from '$lib/components/StarRating.svelte';
    import { api } from '$lib/api/social';

    let work = $state(/* fetched work */);
    let avgRating = $state(0);
    let myRating = $state(0);

    onMount(async () => {
        // Load current user's rating
        const myRes = await api.get(`/api/v1/ratings/${work.id}/mine`);
        if (myRes.err === 0) myRating = myRes.rating;

        // Load average
        const avgRes = await api.get(`/api/v1/ratings/${work.id}`);
        if (avgRes.err === 0) avgRating = avgRes.avg_rating;
    });
</script>

<StarRating workId={work.id} currentRating={myRating} onRate={(r) => api.post('/api/v1/ratings', { work_id: work.id, rating: r }) } />
<div class="avg-rating">Average: {avgRating.toFixed(1)} ★ ({/* rating count */})</div>
```

### Try It Yourself

```svelte
<!-- Add kudos button next to star rating -->
<button class="kudos-btn" onclick={() => api.post(`/api/v1/kudos/${work.id}`, {})}>
    💛 {kudosCount}
</button>
```

### Check

- ✅ 5 clickable stars, filled if `rating >= star`.
- ✅ `onRate` callback fires when user clicks a star.
- ✅ Component re-renders with `currentRating` prop.
- ✅ Work page loads avg rating + user's own rating on mount.
- ✅ Integration with `api.post('/api/v1/ratings', ...)` to submit.

### What you built

The StarRating component — 5 clickable stars, filled state, callback, integration with the work page's rating + average display. Plus the kudos button that calls `POST /api/v1/kudos/{work_id}`.

---

## Conclusion

You now understand FicHub's rating and kudos system:

1. **5-star ratings** — `fic_ratings` table, 1-5 stars, UPSERT on change, `AVG` computed live, feeds into `fi_summary`.
2. **Kudos** — one-click, `fic_kudos` table, `UNIQUE (user_id, work_id)`, idempotent POST, feeds into `fi_summary`.
3. **`fi_summary` view** — live aggregation joining `fic_info → works → ratings/kudos/bookmarks`. Powers search results and work cards.
4. **StarRating widget** — 5 clickable stars, filled state, callback, work page integration.

---

## Troubleshooting

- **Rating doesn't save**: Check that the user is logged in (`Authorization: Bearer <token>`). The handler requires auth.
- **Kudos count off by one**: Make sure `ON CONFLICT (user_id, work_id) DO NOTHING` — if you forgot the conflict clause, double-clicking gives double kudos.
- **Stars not filling**: The `filled` class is applied when `rating >= star`. If the loop is `{#each [1,2,3,4,5] as star}`, then `rating >= star` works correctly.


## Appendix C: Kudos Deep Dive — The Ecosystem Around One Click

> Part 10 introduced the basic kudos CRUD (POST/GET/DELETE). This appendix covers the kudos feed, notification integration, and a critical search-filter naming bug you must know about.

> In this chapter you will learn the full kudos lifecycle in depth — how kudos integrates with the feed, notifications, and search, and why kudos ≠ ratings ≠ likes. We'll explore the kudos feed endpoint, notification integration, and the search filter naming inconsistency.

---

## Overview

Part 10 introduced the basic kudos CRUD (POST/GET/DELETE). This chapter covers:

- **Kudos feed**: recent-kudos list on work pages (who just kudo'd)
- **Notification integration**: kudos events create `kudos_on_work` notifications
- **Search integration**: the `min_kudos` filter actually checks `likes` (ratings + reviews), NOT kudos
- **Database schema**: partial unique index for guest kudos

The kudos system is in `src/routes/social.rs` — `give_kudos_handler`, `remove_kudos_handler`, `get_kudos_handler`, `kudos_aggregate_json`, and `recent_kudos_handler`.

---

## Chapter 10.C.1 — The kudos Feed

### Goal

Build the kudos feed — recent users who kudos'd a work, shown on the work page sidebar.

### Actions

```rust
// src/routes/social.rs — GET /api/kudos/{work_id}/recent
/// Returns recent SIGN-IN kudos only (guests excluded — no username to show).
/// Guest kudos are counted in the aggregate (guest_count) but never in the feed.
pub async fn recent_kudos_handler(
    auth: AuthUser, State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>, Query(params): Query<KudosRecentParams>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).min(50).max(1);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, (String, i64, String)>(
        r#"SELECT COALESCE(u.username, 'Anonymous') AS username,
                  k.id AS kudos_id, k.created_at::text
           FROM kudos k LEFT JOIN users u ON u.id = k.user_id
           WHERE k.work_id = $1 AND k.user_id IS NOT NULL
           ORDER BY k.created_at DESC LIMIT $2 OFFSET $3"#,
    ).bind(work_id).bind(limit).bind(offset).fetch_all(&state.db).await?;

    Ok(Json(json!({
        "err": 0, "work_id": work_id,
        "recent_kudos": rows.into_iter().map(|(u, id, ts)| json!({"username": u, "kudos_id": id, "created_at": ts})).collect::<Vec<_>>(),
    })))
}
```

Frontend (WorkBlurb.svelte):

```typescript
// Optimistic kudos toggle — count ±1 immediately
async function toggleKudos() {
    if (kudosState.my_kudos) {
        await fetch(`/api/kudos/${workId}`, { method: 'DELETE', headers: authHeaders() });
        kudosState.my_kudos = false; kudosState.count--;
    } else {
        await fetch(`/api/kudos/${workId}`, { method: 'POST', headers: authHeaders() });
        kudosState.my_kudos = true; kudosState.count++;
    }
}
```

### Check

- ✅ Guest kudos excluded from feed (`k.user_id IS NOT NULL`).
- ✅ `limit` capped at 50, cursor-based pagination.
- ✅ Optimistic frontend toggle.
- ✅ `my_kudos` in aggregate for signed-in viewers.

### What you built

The kudos feed — recent-kudos endpoint, guest exclusion, cursor pagination, optimistic frontend toggle.

---

## Chapter 10.C.2 — Kudos ≠ Ratings ≠ Likes

### Goal

Understand the three feedback signals, their schemas, and what each powers.

### Actions

```
Ratings:   work_ratings table  — UPSERT, 1-5 stars, legacy -1 (hidden)
Kudos:     kudos table        — idempotent INSERT, guest-aware partial unique
Likes:     computed field     — rating_count + review_count (positive-only)
```

> **⚠️ Watch Out — critical naming bug**: The search filter is called `min_kudos` but it actually checks `likes` (5-star ratings + constructive reviews), NOT the `kudos` table. This is a naming inconsistency from the AO3 fork. A work with 100 kudos but 0 five-star ratings will NOT pass `min_kudos:100`.

```sql
-- kudos table schema (src/routes/social.rs documentation)
CREATE TABLE kudos (
    id SERIAL PRIMARY KEY,
    work_id INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,  -- NULL = guest
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (work_id, user_id),
    UNIQUE (work_id) WHERE (user_id IS NULL)  -- partial index: 1 guest kudos per work
);
```

```rust
// src/routes/social.rs — rating_aggregate_json includes "likes" = rating_count + review_count
// This is what the search "min_kudos" filter and "kudos" sort actually use:
//   WHERE likes >= min_kudos  (NOT kudos_count >= min_kudos)
```

#### Notification integration

When a work receives kudos, a notification is created:

```rust
// Triggered in give_kudos_handler after the INSERT:
// INSERT INTO notifications (user_id, notification_type, title, body, link)
// VALUES (work_owner_id, 'kudos_on_work', 'New kudos', '<user> kudos\'d your work', '/work/<url_id>')
```

The NotificationBell component (`frontend/src/lib/components/NotificationBell.svelte`) polls `GET /api/v1/notifications/unread-count` and displays a badge.

### Try It Yourself

```bash
# The naming inconsistency in practice:
# Work A: 50 kudos, 10 five-star ratings, 3 reviews
#   kudos_count=50, guest_count=0, likes=13  (10+3)
#   search min_kudos:50 → FAILS (likes=13 < 50)
#   search min_kudos:13 → PASSES (likes=13 >= 13)
```

### Check

- ✅ `kudos_count` = signed-in kudos only; `guest_count` = 0 or 1.
- ✅ `likes` = 5-star ratings + reviews (NOT kudos).
- ✅ `min_kudos` search filter checks `likes`, not `kudos_count`.
- ✅ Partial unique index: `UNIQUE (work_id) WHERE (user_id IS NULL)`.
- ✅ Kudos events create `kudos_on_work` notifications.

### What you built

The kudos ecosystem — kudos feed with guest exclusion, the critical `min_kudos` ≠ kudos naming inconsistency, partial unique index for guest kudos, and notification integration.


---

End of Appendix C (Part 10).

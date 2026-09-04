# Part 5: Recommendations

---

# Chapter 21: Collaborative Filtering

## The Recommendation Problem

When you read a great fanfiction, you often want to find similar stories. FicHub's recommendation engine solves this using **collaborative filtering** — the same fundamental technique Netflix, Spotify, and Amazon use for their "recommended for you" features.

The core insight is simple but powerful: people who liked the same things you liked will probably like other things you'll like too. If Alice and Bob both bookmarked Story X, and Alice also bookmarked Story Y, then Story Y might interest Bob.

There are two main approaches to collaborative filtering:

1. **User-based** — Find users similar to you, recommend what they liked
2. **Item-based** — Find items similar to what you liked, recommend those

FicHub uses item-based collaborative filtering because it scales better and works well with the data available from fanfiction sites.

## The Jaccard Coefficient

The similarity between two stories is measured using the **Jaccard coefficient**, a standard metric in information retrieval:

```
Jaccard(A, B) = |A ∩ B| / |A ∪ B|
```

Where:
- A = set of users who bookmarked Story A
- B = set of users who bookmarked Story B
- |A ∩ B| = number of users who bookmarked both (intersection)
- |A ∪ B| = total number of unique users who bookmarked either (union)

### Example

Suppose:
- 100 users bookmarked Story A
- 80 users bookmarked Story B
- 60 users bookmarked both A and B

```
Jaccard = 60 / (100 + 80 - 60) = 60 / 120 = 0.5
```

A Jaccard of 0.5 means the stories are moderately similar — half of the combined audience overlaps.

### Properties of Jaccard

- Range: 0 to 1
- 0 = no overlap (completely different audiences)
- 1 = perfect overlap (same audience)
- Symmetric: Jaccard(A, B) = Jaccard(B, A)
- Doesn't consider absolute sizes, only overlap proportions

### Why Not Cosine Similarity?

Cosine similarity is another popular metric, but Jaccard is better for binary data (bookmarked or not). Cosine would give too much weight to stories with many bookmarkers, even if the overlap proportion is small.

## The Co-occurrence Matrix

FicHub stores co-occurrence data in the `fic_bookmark_cooccur` table:

```sql
CREATE TABLE fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

The `CHECK (work_a < work_b)` constraint ensures each pair is stored only once. This is a common optimization — instead of storing both (A, B) and (B, A), we only store one and canonicalize the order.

The `cooccur_count` is the number of users who bookmarked both stories. When a new user's favourites are processed, this count is incremented.

## Computing Recommendations

The recommendation query computes Jaccard coefficients for all stories that co-occur with the seed story:

```sql
WITH seed AS (
    SELECT favouriter_count FROM fic_works WHERE url_id = $1
),
candidates AS (
    SELECT
        CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
        cooccur_count
    FROM fic_bookmark_cooccur
    WHERE work_a = $1 OR work_b = $1
)
SELECT
    c.candidate_id,
    c.cooccur_count,
    fw.favouriter_count,
    (c.cooccur_count::float
       / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
    ) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != $1
ORDER BY jaccard DESC
LIMIT $2
```

Let's break down this query:

1. **CTE `seed`** — Gets the seed story's favouriter count
2. **CTE `candidates`** — Finds all stories that co-occur with the seed, using CASE to normalize the order
3. **Main query** — Computes Jaccard = cooccur / (seed_count + candidate_count - cooccur)
4. **ORDER BY jaccard DESC** — Most similar first
5. **LIMIT** — Return top N results

## Tag Fallback

For new stories with few favouriters, collaborative filtering doesn't work well — there's simply not enough data. FicHub falls back to tag-based recommendations:

```rust
let min_collab = config.rec_min_favouriters_for_collab as i32;
let use_tag_fallback = favouriter_count < min_collab;

let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The `rec_min_favouriters_for_collab` config (default: 5) determines when to switch from tag-based to collaborative.

### Same Author (Strong Signal)

```rust
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
            results.push((row.id, 0.8));  // Score: 0.8
        }
    }
}
```

Stories by the same author get a high base similarity score of 0.8.

### Title Keywords (Weaker Signal)

```rust
let keywords: Vec<&str> = seed_title
    .split_whitespace()
    .filter(|w| w.len() >= 3)
    .collect();

for kw in keywords {
    if results.len() >= limit { break; }
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
            results.push((row.id, 0.5));  // Score: 0.5
        }
    }
}
```

Stories with matching title keywords get a lower base score of 0.5.

## Blending Scores

FicHub smoothly blends collaborative and tag-based scores:

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

for c in &mut candidates {
    let collab = c.score;
    let tag = c.tag_score;
    c.score = collab * weight + tag * (1.0 - weight);
}
```

The weight transitions based on data availability:
- 0 favouriters → weight = 0 (pure tag-based)
- 1 favouriter → weight = 0.2
- 2 favouriters → weight = 0.4
- 3 favouriters → weight = 0.6
- 4 favouriters → weight = 0.8
- 5+ favouriters → weight = 1.0 (pure collaborative)

This ensures smooth degradation as data becomes sparse.

## Community Voting Boost

FicHub considers community votes on recommendations:

```rust
let gamma = config.rec_voting_boost_gamma;  // Default: 0.2
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The `ln_1p` function ensures logarithmic scaling:
- 0 votes → boost = 1.0 (no change)
- 1 vote → boost ≈ 1.14
- 5 votes → boost ≈ 1.32
- 10 votes → boost ≈ 1.46
- 50 votes → boost ≈ 1.78

This means the first few votes have a big impact, but additional votes have diminishing returns — exactly what we want.

## Caching Recommendations

Computed recommendations are cached in the `precomputed_recommendations` table:

```sql
CREATE TABLE precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

The engine first checks this cache:

```rust
let cached = check_cache(&self.db, query, config, limit).await?;
if !cached.is_empty() {
    return Ok(cached);
}

compute_live(&self.db, query, config, limit).await
```

### Cache Check Query

```sql
SELECT pr.recommended_url_id, pr.score,
       fi.title, fi.author, fi.words, fi.chapters,
       fi.status, fi.source, fi.description
FROM precomputed_recommendations pr
JOIN fic_info fi ON fi.id = pr.recommended_url_id
WHERE pr.url_id = $1
  AND pr.computed_at > NOW() - ($2 * INTERVAL '1 hour')
ORDER BY pr.rank ASC
LIMIT $3
```

The cache expires after `rec_cache_ttl_hours` (default: 12 hours).

### Cache Population

```rust
pub async fn compute_and_cache(&self, url_id: &str, config: &Config) -> Result<(), AppError> {
    let results = compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;

    // Clear old cache
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
```

## Community Scores

The engine also fetches community scores for display:

```rust
async fn get_community_scores(db: &PgPool) -> Result<HashMap<String, i32>, AppError> {
    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           GROUP BY s.suggested_url_id"#,
    )
    .fetch_all(db)
    .await?;

    let mut map = HashMap::new();
    for (url_id, score) in rows {
        map.insert(url_id, score.unwrap_or(0));
    }
    Ok(map)
}
```

## Watch Out!

**Cold start problem!** New stories with no favouriters can't be recommended via collaborative filtering. The tag fallback helps but is less accurate.

**Popularity bias!** Stories with many favouriters dominate. Jaccard helps mitigate this by considering overlap proportion, not absolute counts.

**Site isolation!** FicHub can optionally filter recommendations by site domain, preventing cross-site recommendations that might not make sense.

## Summary

FicHub's recommendation engine uses collaborative filtering with Jaccard coefficients, tag fallback for new stories, community voting boosts, and a caching layer. The system smoothly transitions from tag-based to collaborative as more data becomes available.

---

# Chapter 22: Collection Worker

## The Background Worker

FicHub's recommendation engine needs data — specifically, it needs to know which users have bookmarked which stories. This data is collected by the **CollectionWorker**, a background task that scrapes user favourites from fanfiction sites.

## The SiteFetcher Trait

Each fanfiction site implements the `SiteFetcher` trait:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    fn site_domain(&self) -> &str;

    fn rate_limit_delay(&self, config: &Config, domain: &str) -> u64 {
        config.rec_site_rate_limits.get(domain).copied()
            .unwrap_or(config.rec_default_delay_secs)
    }

    async fn collect_favouriters(
        &self, client: &Client, work_url: &str, max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    async fn collect_user_favourites(
        &self, client: &Client, user_url: &str, max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

- `site_domain` — Canonical domain for the site
- `rate_limit_delay` — Per-site delay between requests
- `collect_favouriters` — Find users who bookmarked a story
- `collect_user_favourites` — Find all stories a user bookmarked
- `user_hash` — Create a deterministic, anonymous hash of a user's profile URL

The `user_hash` method uses SHA-256 to create a one-way hash. This preserves anonymity while allowing us to track user preferences.

## The Redis Queue

The worker uses Redis as a job queue:

```rust
pub async fn enqueue(&self, url_id: &str, site_domain: &str, site_work_id: &str)
    -> Result<(), redis::RedisError>
{
    let item = QueueItem {
        url_id: url_id.to_string(),
        site_domain: site_domain.to_string(),
        site_work_id: site_work_id.to_string(),
    };
    let json = serde_json::to_string(&item)?;
    let key = format!("collection_queue:{}", site_domain);

    let mut conn = self.redis.lock().await;
    redis::cmd("LPUSH")
        .arg(&[key.as_str(), json.as_str()])
        .query_async(&mut *conn)
        .await
}
```

Jobs are pushed with `LPUSH` and popped with `LPOP`.

## The Per-Site Rate Limiter

Each site has its own rate limiter using an atomic CAS loop:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

impl PerSiteRateLimiter {
    pub fn new(delay_secs: u64) -> Self {
        Self {
            last_request: AtomicI64::new(0),
            delay_secs,
        }
    }

    pub async fn wait_if_needed(&self) {
        let delay_nanos = (self.delay_secs as u64) * 1_000_000_000;

        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;

            let last = self.last_request.load(Ordering::Acquire);
            let elapsed = now.wrapping_sub(last);

            if elapsed >= delay_nanos as i64 {
                if self.last_request.compare_exchange(
                    last, now, Ordering::AcqRel, Ordering::Relaxed
                ).is_ok() {
                    return;
                }
            } else {
                let remaining = delay_nanos - elapsed as u64;
                tokio::time::sleep(Duration::from_nanos(remaining)).await;
            }
        }
    }
}
```

The CAS (compare-and-swap) loop atomically claims a time slot. Multiple concurrent callers are serialized so each waits the full delay.

## The Worker Loop

```rust
pub async fn run(&self) {
    info!("Collection worker started");

    loop {
        for fetcher in &self.fetchers {
            let domain = fetcher.site_domain();
            let key = format!("collection_queue:{}", domain);

            let item_str: Option<String> = {
                let mut conn = self.redis.lock().await;
                redis::cmd("LPOP")
                    .arg(&key)
                    .query_async(&mut *conn)
                    .await
                    .unwrap_or(None)
            };

            if let Some(item_str) = item_str {
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }

                match serde_json::from_str::<QueueItem>(&item_str) {
                    Ok(item) => {
                        if let Err(e) = self.process_work(item).await {
                            error!("Error processing work: {}", e);
                        }
                    }
                    Err(e) => warn!("Invalid queue item: {}", e),
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

## Processing a Work

The `process_work` method is the core collection procedure:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = self.fetchers.iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or("No fetcher for domain")?;

    let work_url = format!("https://{}/works/{}", item.site_domain, item.site_work_id);

    // 1. Fetch favouriters
    let favouriters = fetcher.collect_favouriters(
        &self.client, &work_url, self.config.rec_max_favourite_pages
    ).await?;

    let mut new_user_count: u32 = 0;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // 2. Skip if already recorded
        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)"
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .fetch_one(&self.db)
        .await?;

        if already_exists { continue; }
        new_user_count += 1;

        // 3. Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .execute(&self.db)
        .await?;

        // 4. Fetch user's favourites
        let user_favs = fetcher.collect_user_favourites(
            &self.client, user_url, self.config.rec_max_user_favourite_pages
        ).await?;

        // 5. Update co-occurrence
        for other_url_id in &user_favs {
            if other_url_id == &item.url_id { continue; }
            let (a, b) = if item.url_id < *other_url_id {
                (&item.url_id, other_url_id)
            } else {
                (other_url_id, &item.url_id)
            };

            sqlx::query(
                r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, site_domain, cooccur_count)
                   VALUES ($1, $2, $3, 1)
                   ON CONFLICT (work_a, work_b) DO UPDATE SET
                       cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                       last_updated = NOW()"#,
            )
            .bind(a)
            .bind(b)
            .bind(&item.site_domain)
            .execute(&self.db)
            .await?;
        }
    }

    // 6. Update favouriter count
    sqlx::query(
        r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count,
                                   first_favourite_scraped, last_favourite_scraped)
           VALUES ($1, $2, $3, $4, NOW(), NOW())
           ON CONFLICT (url_id) DO UPDATE SET
               favouriter_count = fic_works.favouriter_count + 1,
               last_favourite_scraped = NOW()"#,
    )
    .bind(&item.url_id)
    .bind(&item.site_domain)
    .bind(&item.site_work_id)
    .bind(favouriters.len() as i32)
    .execute(&self.db)
    .await?;

    info!("Processed {} from {}: {} favouriters, {} new users",
        item.url_id, item.site_domain, favouriters.len(), new_user_count);

    Ok(())
}
```

## Stub SiteFetcher Implementations

Currently, FicHub has stub implementations for all sites:

```rust
struct Ao3Fetcher;
struct FfNetFetcher;
struct XenForoFetcher;
struct FictionPressFetcher;
struct AdultFanFictionFetcher;
struct HpFanFicFetcher;

#[async_trait]
impl SiteFetcher for Ao3Fetcher {
    fn site_domain(&self) -> &str { "archiveofourown.org" }

    async fn collect_favouriters(&self, _client: &Client, _work_url: &str, _max_pages: u32)
        -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())  // Stub — implement later
    }

    async fn collect_user_favourites(&self, _client: &Client, _user_url: &str, _max_pages: u32)
        -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())  // Stub — implement later
    }
}
```

The architecture is the focus — real scraping logic can be added per-site without changing the worker.

## Watch Out!

**The worker is CPU and network intensive!** Run it on a separate thread to avoid blocking the main server.

**Rate limits are essential!** Without per-site rate limiting, the worker could get blocked.

**Privacy matters!** User hashes are one-way — you can't reverse them.

## Summary

The CollectionWorker is a background task that scrapes user favourites, populates the co-occurrence matrix, and updates favouriter counts. It uses Redis queues, per-site rate limiters, and atomic CAS loops.

---

# Chapter 23: Community Suggestions

## Beyond Algorithmic Recommendations

While collaborative filtering is powerful, it can't capture everything. Sometimes a human reader knows that two stories are similar in ways algorithms can't detect — maybe they share a specific plot device, or they're both part of the same crossover universe.

## The Suggestion Table

```sql
CREATE TABLE recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);
```

The `UNIQUE` constraint ensures each user can only suggest a specific pair once.

## Submitting a Suggestion

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    if body.url_id.is_empty() || body.suggested_url.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id and suggested_url required"})));
    }

    // Resolve the suggested URL
    let scraper = state.scraper_registry.find_scraper(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", body.suggested_url)))?;
    let meta = scraper.lookup(&state.http_client, &body.suggested_url).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let suggested_url_id = meta.url_id;

    // Validate both fics exist
    let seed_exists = queries::get_fic_info(&state.db, &body.url_id).await?.is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id).await?.is_some();

    if !seed_exists {
        state.collection_worker.enqueue(&body.url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "seed fic not found — enqueued"})));
    }

    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "suggested fic not collected — enqueued"})));
    }

    // Rate limit check
    check_tag_rate_limit(&mut redis, "suggest", ip, state.config.rec_suggest_limit_per_hour).await?;

    // Submit
    let suggestion_id = submit_suggestion(
        &state.db, &body.url_id, &suggested_url_id, "0.0.0.0", body.comment.as_deref(),
    ).await?;

    Ok(Json(json!({"err": 0, "suggestion_id": suggestion_id})))
}
```

## The submit_suggestion Function

```rust
pub async fn submit_suggestion(
    pool: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    submitted_by_ip: &str,
    comment: Option<&str>,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
           (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip) DO NOTHING
           RETURNING id"#,
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(submitted_by_ip)
    .bind(comment)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
```

## Rate Limiting

```rust
async fn check_tag_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    action: &str,
    ip: std::net::IpAddr,
    limit: u32,
) -> AppResult<()> {
    let key = format!("ratelimit:tag:{}:{}", action, ip);

    let count: Option<u32> = redis::cmd("GET")
        .arg(&key)
        .query_async(redis)
        .await
        .unwrap_or(None);

    if let Some(c) = count {
        if c >= limit {
            let ttl: u64 = redis::cmd("TTL")
                .arg(&key)
                .query_async(redis)
                .await
                .unwrap_or(3600);
            return Err(AppError::RateLimited(ttl));
        }
    }

    let new_count: u32 = redis::cmd("INCR")
        .arg(&key)
        .query_async(redis)
        .await?;

    if new_count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600i64)
            .query_async(redis)
            .await?;
    }

    Ok(())
}
```

The rate limit key has a 1-hour TTL, so the counter resets automatically.

## Watch Out!

**IP-based identification is imperfect!** Multiple users behind the same NAT share an IP.

**Suggestions require both fics in the database!** Missing fics are enqueued for background collection.

## Summary

Community suggestions let users contribute to the recommendation system. The system validates inputs, rate-limits submissions, and enqueues missing stories.

---

# Chapter 24: Voting

## The Vote Table

```sql
CREATE TABLE recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

## Casting a Vote

```rust
pub async fn cast_vote(
    pool: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    value: i16,
) -> AppResult<i32> {
    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip) DO UPDATE SET vote = EXCLUDED.vote"#,
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(value)
    .execute(pool)
    .await?;

    let row: (i32,) = sqlx::query_as(
        "SELECT COALESCE(SUM(vote)::INT, 0) FROM recommendation_votes WHERE suggestion_id = $1"
    )
    .bind(suggestion_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
```

The `ON CONFLICT DO UPDATE` lets users change their vote.

## Net Score

```sql
SELECT COALESCE(SUM(vote)::INT, 0)
FROM recommendation_votes
WHERE suggestion_id = $1
```

## Community Scores in Recommendations

```rust
let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

## Getting Community Suggestions

```rust
pub async fn get_community_suggestions(
    db: &PgPool,
    url_id: &str,
) -> AppResult<Vec<Suggestion>> {
    let rows: Vec<SuggestionRow> = sqlx::query_as::<_, SuggestionRow>(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(SUM(v.vote)::INT, 0) AS net_votes,
                  s.created
           FROM recommendation_suggestions s
           LEFT JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           GROUP BY s.id, s.suggested_url_id, s.comment, s.created
           ORDER BY net_votes DESC"#,
    )
    .bind(url_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|r| Suggestion {
        id: r.id,
        suggested_url_id: r.suggested_url_id,
        comment: r.comment,
        net_votes: r.net_votes,
        created: r.created,
    }).collect())
}
```

## Vote Validation

```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(json!({"err": -1, "msg": "vote must be 1 or -1"})));
    }

    let new_score = cast_vote(&state.db, body.suggestion_id, "0.0.0.0", body.vote as i16).await?;

    Ok(Json(json!({"err": 0, "new_score": new_score})))
}
```

## Watch Out!

**Single vote per user!** The PRIMARY KEY ensures each IP can only vote once per suggestion.

**No anonymous voting without accounts!** IP-based voting is the current approach.

## Summary

FicHub's voting system lets users upvote or downvote community suggestions with optimistic voting, net score computation, and logarithmic boosting for the recommendation engine.

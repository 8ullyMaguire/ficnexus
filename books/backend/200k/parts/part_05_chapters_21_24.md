# Part 5: Recommendation Engine

---

# Chapter 21: Collaborative Filtering

The recommendation engine suggests similar stories based on what other readers have bookmarked. This chapter covers the collaborative filtering algorithm, co-occurrence tables, and the scoring system.

## How Collaborative Filtering Works

Collaborative filtering is based on a simple idea: people who liked the same things in the past will probably like similar things in the future.

**Real-world analogy:** Imagine you and your friend both love the same 10 books. Your friend discovers a new book and loves it. You haven't read it yet, but based on your shared taste, you'd probably love it too. That's collaborative filtering.

### The Co-occurrence Matrix

When two stories appear in the same user's favourites, we increment a counter for that pair. Stories that co-occur frequently are likely similar.

```
Story A favourites: {Story B, Story C, Story D}
Story B favourites: {Story A, Story C, Story E}
Story C favourites: {Story A, Story B, Story F}

Co-occurrence:
  A-B: 2 (both favourited by users who like A and B)
  A-C: 2
  B-C: 2
  A-D: 1
  B-E: 1
  C-F: 1
```

### Jaccard Coefficient

The Jaccard coefficient measures similarity between two sets:

```
J(A, B) = |A ∩ B| / |A ∪ B|
```

Where:
- `|A ∩ B|` is the number of users who favourited both stories
- `|A ∪ B|` is the total number of users who favourited either story

A Jaccard coefficient of 1.0 means identical favourite lists. 0.0 means no overlap.

**Real-world analogy:** The Jaccard coefficient is like comparing two spice racks. If you have 5 spices and your friend has 5 spices, and 3 are the same, your Jaccard similarity is 3/7 ≈ 0.43. If you have the exact same 5 spices, it's 1.0.

### The Scoring Algorithm

FicHub combines co-occurrence counts with Jaccard similarity:

```rust
fn compute_score(
    cooccurrence_count: i32,
    story_a_favouriters: i32,
    story_b_favouriters: i32,
    voting_boost: f64,
) -> f64 {
    // Jaccard coefficient
    let union = (story_a_favouriters + story_b_favouriters - cooccurrence_count) as f64;
    if union == 0.0 {
        return 0.0;
    }
    let jaccard = cooccurrence_count as f64 / union;
    
    // Apply voting boost
    let boosted = jaccard * (1.0 + voting_boost);
    
    // Clamp to [0, 1]
    boosted.min(1.0)
}
```

## The Recommendation Engine

```rust
use sqlx::PgPool;
use crate::error::AppError;

pub struct RecommendationEngine {
    pool: PgPool,
}

impl RecommendationEngine {
    pub fn new(pool: PgPool) -> Self {
        RecommendationEngine { pool }
    }
    
    pub async fn get_recommendations(
        &self,
        url_id: &str,
        limit: usize,
    ) -> Result<Vec<Recommendation>, AppError> {
        // 1. Get co-occurring stories
        let cooccurrences = sqlx::query_as::<_, (String, i32)>(
            r#"SELECT story_b, count
               FROM rec_cooccurrence
               WHERE story_a = $1
               ORDER BY count DESC
               LIMIT $2"#
        )
        .bind(url_id)
        .bind((limit * 2) as i64)  // Fetch extra for filtering
        .fetch_all(&self.pool)
        .await?;
        
        if cooccurrences.is_empty() {
            return Ok(Vec::new());
        }
        
        // 2. Compute scores
        let mut recommendations = Vec::new();
        for (rec_url_id, count) in cooccurrences {
            // Get favouriter counts for Jaccard calculation
            let (a_count, b_count) = self.get_favouriter_counts(url_id, &rec_url_id).await?;
            
            let score = compute_score(count, a_count, b_count, 0.0);
            
            // Get story metadata
            if let Some(meta) = self.get_story_meta(&rec_url_id).await? {
                recommendations.push(Recommendation {
                    url_id: rec_url_id,
                    title: meta.title,
                    author: meta.author,
                    score,
                    reason: format!("{} readers also favourited this", count),
                });
            }
        }
        
        // 3. Sort by score and take top N
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(limit);
        
        Ok(recommendations)
    }
    
    async fn get_favouriter_counts(
        &self,
        story_a: &str,
        story_b: &str,
    ) -> Result<(i32, i32), AppError> {
        let a_count: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM rec_favourites WHERE url_id = $1"
        )
        .bind(story_a)
        .fetch_one(&self.pool)
        .await?;
        
        let b_count: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM rec_favourites WHERE url_id = $2"
        )
        .bind(story_b)
        .fetch_one(&self.pool)
        .await?;
        
        Ok((a_count.0, b_count.0))
    }
    
    async fn get_story_meta(
        &self,
        url_id: &str,
    ) -> Result<Option<StoryMeta>, AppError> {
        let meta = sqlx::query_as::<_, StoryMeta>(
            "SELECT title, author FROM fic_info WHERE id = $1"
        )
        .bind(url_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(meta)
    }
}
```

## Tag-Based Fallback

When co-occurrence data is insufficient, the engine falls back to tag-based similarity:

```rust
async fn tag_based_recommendations(
    &self,
    url_id: &str,
    limit: usize,
) -> Result<Vec<Recommendation>, AppError> {
    // Get tags for the source story
    let source_tags = sqlx::query_scalar::<_, String>(
        r#"SELECT t.name
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           AND ft.score >= 0"#
    )
    .bind(url_id)
    .fetch_all(&self.pool)
    .await?;
    
    if source_tags.is_empty() {
        return Ok(Vec::new());
    }
    
    // Find stories with the most overlapping tags
    let recommendations = sqlx::query_as::<_, (String, String, String, i64)>(
        r#"SELECT fi.id, fi.title, fi.author, COUNT(*) as tag_overlap
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           JOIN fic_info fi ON fi.id = ft.url_id
           WHERE t.name = ANY($1)
           AND ft.url_id != $2
           AND ft.score >= 0
           GROUP BY fi.id, fi.title, fi.author
           ORDER BY tag_overlap DESC
           LIMIT $3"#
    )
    .bind(&source_tags)
    .bind(url_id)
    .bind(limit as i64)
    .fetch_all(&self.pool)
    .await?
    .into_iter()
    .map(|(id, title, author, overlap)| Recommendation {
        url_id: id,
        title,
        author,
        score: overlap as f64 / source_tags.len() as f64,
        reason: format!("{} shared tags", overlap),
    })
    .collect();
    
    Ok(recommendations)
}
```

## 📝 Practice Exercises

1. **Jaccard Calculator:** Implement the Jaccard coefficient function and test it with sample sets.

2. **Co-occurrence Builder:** Write a function that builds the co-occurrence matrix from a list of user favourites.

3. **Score Tuning:** Experiment with different scoring formulas. How does adding a minimum co-occurrence threshold affect recommendation quality?

---

# Chapter 22: The Collection Worker

The collection worker is a background task that scrapes user favourite lists to build data for the recommendation engine.

## How the Worker Works

1. Find users whose favourites haven't been collected recently
2. Fetch their favourite list from the source site
3. Extract story IDs from the favourites
4. Update the co-occurrence matrix
5. Mark the user as processed

**Real-world analogy:** The collection worker is like a librarian who regularly checks which books patrons have checked out. By tracking what people read, the librarian can make better recommendations.

## The Worker Implementation

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct CollectionWorker {
    pool: PgPool,
    redis: MultiplexedConnection,
    http_client: Client,
    config: Config,
    scraper_registry: Arc<ScraperRegistry>,
    processed_count: AtomicI64,
}

impl CollectionWorker {
    pub fn new(
        pool: PgPool,
        redis: MultiplexedConnection,
        http_client: Client,
        config: Config,
        scraper_registry: Arc<ScraperRegistry>,
    ) -> Self {
        CollectionWorker {
            pool,
            redis,
            http_client,
            config,
            scraper_registry,
            processed_count: AtomicI64::new(0),
        }
    }
    
    pub async fn run(&self) {
        loop {
            match self.process_batch().await {
                Ok(count) => {
                    self.processed_count.fetch_add(count, Ordering::SeqCst);
                    tracing::info!(
                        processed = count,
                        total = self.processed_count.load(Ordering::SeqCst),
                        "Collection batch complete"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "Collection batch failed");
                }
            }
            
            // Wait before next batch
            let delay = Duration::from_secs(self.config.rec_default_delay_secs);
            tokio::time::sleep(delay).await;
        }
    }
    
    async fn process_batch(&self) -> Result<i64, AppError> {
        // Find users to process
        let users = self.get_users_to_process().await?;
        
        let mut processed = 0;
        for user in users {
            match self.process_user(&user).await {
                Ok(_) => {
                    self.mark_user_processed(&user.id).await?;
                    processed += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        user_id = %user.id,
                        error = %e,
                        "Failed to process user"
                    );
                }
            }
            
            // Rate limit between users
            let delay = Duration::from_millis(
                self.config.rec_default_delay_secs * 1000
            );
            tokio::time::sleep(delay).await;
        }
        
        Ok(processed)
    }
    
    async fn get_users_to_process(&self) -> Result<Vec<RecUser>, AppError> {
        let users = sqlx::query_as::<_, RecUser>(
            r#"SELECT id, source_site, last_collected, favourite_count
               FROM rec_users
               WHERE processed = FALSE
               OR last_collected < NOW() - INTERVAL '24 hours'
               ORDER BY last_collected ASC NULLS FIRST
               LIMIT 10"#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }
    
    async fn process_user(&self, user: &RecUser) -> Result<(), AppError> {
        // Fetch user's favourites from source site
        let favourites = self.fetch_favourites(user).await?;
        
        // Store favourites
        for fav_url_id in &favourites {
            self.store_favourite(&user.id, fav_url_id).await?;
        }
        
        // Update co-occurrence matrix
        self.update_cooccurrence(&favourites).await?;
        
        // Update user stats
        sqlx::query(
            r#"UPDATE rec_users
               SET last_collected = NOW(),
                   favourite_count = $1,
                   processed = TRUE
               WHERE id = $2"#
        )
        .bind(favourites.len() as i32)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn fetch_favourites(&self, user: &RecUser) -> Result<Vec<String>, AppError> {
        // Implementation depends on source site
        // This is a simplified version
        let mut favourites = Vec::new();
        
        for page in 1..=self.config.rec_max_user_favourite_pages {
            let url = self.build_favourite_url(user, page);
            let response = self.http_client
                .get(&url)
                .send()
                .await
                .map_err(|e| AppError::ScrapeError(e.to_string()))?;
            
            if !response.status().is_success() {
                break;
            }
            
            let html = response.text().await
                .map_err(|e| AppError::ScrapeError(e.to_string()))?;
            
            let page_favourites = self.parse_favourites(&html);
            if page_favourites.is_empty() {
                break;
            }
            
            favourites.extend(page_favourites);
        }
        
        Ok(favourites)
    }
    
    async fn update_cooccurrence(&self, favourites: &[String]) -> Result<(), AppError> {
        // For each pair of favourited stories, increment co-occurrence
        for i in 0..favourites.len() {
            for j in (i + 1)..favourites.len() {
                let story_a = &favourites[i];
                let story_b = &favourites[j];
                
                sqlx::query(
                    r#"INSERT INTO rec_cooccurrence (story_a, story_b, count)
                       VALUES ($1, $2, 1)
                       ON CONFLICT (story_a, story_b)
                       DO UPDATE SET count = rec_cooccurrence.count + 1"#
                )
                .bind(story_a)
                .bind(story_b)
                .execute(&self.pool)
                .await?;
                
                // Also update the reverse pair
                sqlx::query(
                    r#"INSERT INTO rec_cooccurrence (story_a, story_b, count)
                       VALUES ($2, $1, 1)
                       ON CONFLICT (story_a, story_b)
                       DO UPDATE SET count = rec_cooccurrence.count + 1"#
                )
                .bind(story_a)
                .bind(story_b)
                .execute(&self.pool)
                .await?;
            }
        }
        
        Ok(())
    }
}
```

## 📝 Practice Exercises

1. **Worker Scheduling:** Modify the worker to run at specific times (e.g., every 6 hours) instead of continuously.

2. **Error Recovery:** Add retry logic for failed user processing. How should the worker handle temporary failures vs. permanent ones?

3. **Metrics:** Add metrics to track worker performance: users processed per hour, favourites collected, co-occurrence updates.

---

# Chapter 23: Community Suggestions

Users can suggest stories for the recommendation engine. This chapter covers the suggestion system, including submission, voting, and approval.

## The Suggestion System

### Submitting a Suggestion

```rust
#[derive(Deserialize)]
struct SuggestRequest {
    url: String,
}

pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuggestRequest>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("suggest:{}", remote_addr.ip());
    queries::check_rate_limit(&mut state.redis.clone(), &key, 
        state.config.rec_suggest_limit_per_hour).await?;
    
    // Validate URL
    let scraper = state.scraper_registry.find_scraper(&payload.url)
        .ok_or_else(|| AppError::BadRequest(-1, "unsupported URL".into()))?;
    
    // Lookup metadata
    let meta = scraper.lookup(&state.http_client, &payload.url).await?;
    
    // Upsert fic info
    let fic_info = FicInfo::from_metadata(&meta);
    queries::upsert_fic_info(&state.db, &fic_info).await?;
    
    // Insert suggestion
    sqlx::query(
        r#"INSERT INTO rec_suggestions (url_id, source_url, submitted_ip)
           VALUES ($1, $2, $3::inet)
           ON CONFLICT DO NOTHING"#
    )
    .bind(&meta.url_id)
    .bind(&payload.url)
    .bind(remote_addr.ip().to_string())
    .execute(&state.db)
    .await?;
    
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "title": meta.title,
        "msg": "suggestion submitted"
    })))
}
```

### Voting on Suggestions

Users can vote on whether a suggestion should be included:

```rust
#[derive(Deserialize)]
struct VoteRequest {
    url_id: String,
    recommended_url_id: String,
    value: i16,  // 1 = upvote, -1 = downvote
}

pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VoteRequest>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("rec_vote:{}", remote_addr.ip());
    queries::check_rate_limit(&mut state.redis.clone(), &key,
        state.config.rec_vote_limit_per_hour).await?;
    
    // Validate value
    if payload.value != 1 && payload.value != -1 {
        return Err(AppError::BadRequest(-1, "value must be 1 or -1".into()));
    }
    
    // Upsert vote
    sqlx::query(
        r#"INSERT INTO rec_votes (url_id, recommended_url_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, recommended_url_id, voter_ip)
           DO UPDATE SET value = $4, created_at = NOW()"#
    )
    .bind(&payload.url_id)
    .bind(&payload.recommended_url_id)
    .bind(remote_addr.ip().to_string())
    .bind(payload.value)
    .execute(&state.db)
    .await?;
    
    // Compute vote totals
    let (upvotes, downvotes) = get_vote_totals(
        &state.db, &payload.url_id, &payload.recommended_url_id
    ).await?;
    
    Ok(Json(json!({
        "err": 0,
        "upvotes": upvotes,
        "downvotes": downvotes,
        "net_score": upvotes - downvotes
    })))
}
```

## 📝 Practice Exercises

1. **Suggestion Queue:** Build an admin interface that shows pending suggestions with vote counts.

2. **Auto-Approval:** Implement auto-approval for suggestions that reach a certain vote threshold.

3. **Spam Prevention:** Add rate limiting and duplicate detection to the suggestion system.

---

# Chapter 24: Voting and Scoring

This chapter covers the voting system for recommendations and how votes affect scores.

## The Voting System

### Vote Recording

```rust
pub async fn upsert_tag_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
    value: i16,
) -> AppResult<()> {
    // Try update first
    let affected = sqlx::query(
        r#"UPDATE fic_tag_votes SET value = $1, created_at = NOW()
           WHERE url_id = $2 AND tag_id = $3 AND voter_ip = $4::inet"#
    )
    .bind(value)
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .execute(pool)
    .await?
    .rows_affected();
    
    if affected == 0 {
        // Insert new vote
        sqlx::query(
            r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
               VALUES ($1, $2, $3::inet, $4)"#
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .bind(value)
        .execute(pool)
        .await?;
        
        // Ensure fic_tags row exists
        sqlx::query(
            r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
               VALUES ($1, $2, $3::inet, 0)
               ON CONFLICT DO NOTHING"#
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .execute(pool)
        .await?;
    }
    
    Ok(())
}
```

### Score Computation

The tag score is the sum of all votes:

```rust
pub async fn get_tag_score(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
) -> AppResult<i16> {
    let row: (i16,) = sqlx::query_as(
        r#"SELECT COALESCE(SUM(value), 0)
           FROM fic_tag_votes
           WHERE url_id = $1 AND tag_id = $2"#
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.0)
}
```

### Auto-Moderation

Tags with low scores are automatically hidden:

```rust
pub async fn apply_auto_moderation(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
    auto_delete_threshold: Option<i16>,
) -> AppResult<()> {
    // Get all tags for this fic
    let tags = sqlx::query_as::<_, (i32, String, i16)>(
        r#"SELECT t.id, t.name, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1"#
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    
    for (tag_id, tag_name, score) in tags {
        // Auto-delete tags below threshold
        if let Some(threshold) = auto_delete_threshold {
            if score <= threshold {
                sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
                    .bind(url_id)
                    .bind(tag_id)
                    .execute(pool)
                    .await?;
                
                tracing::info!(
                    url_id = %url_id,
                    tag = %tag_name,
                    score = score,
                    "Auto-deleted tag"
                );
            }
        }
    }
    
    Ok(())
}
```

## 📝 Practice Exercises

1. **Vote History:** Build a function that returns the vote history for a specific tag on a story.

2. **Score Decay:** Implement score decay so that older votes have less influence than newer ones.

3. **Anti-Manipulation:** Implement detection for vote manipulation (e.g., same IP voting on many tags).


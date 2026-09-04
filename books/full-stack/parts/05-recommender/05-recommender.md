# Part 5: The Recommendation Engine

*If the export pipeline is FicHub's factory, the recommendation engine is its brain. It watches what thousands of readers bookmark, learns the patterns of taste, and whispers to each visitor: "Based on what you're reading, you'll probably love this one too." This part covers collaborative filtering, background data collection, community suggestions, and voting—turning passive browsing data into genuinely useful recommendations.*

---

## Chapter 21: Collaborative Filtering Explained

### What Are Recommendations?

You know that friend who, after you finish a great book, immediately says "Oh, you'll love this next one"? They know your taste. They know what you just read. And they've read enough to connect the dots between your last five books and this new one you haven't heard of yet.

A recommendation engine does exactly the same thing, except it works at the scale of thousands of readers instead of one friend.

When a user opens a fanfiction on FicHub, we want to say: "Other people who read *this* story also loved *these* stories." That's the core promise. Not "here are stories with similar tags" (though that helps). Not "here's what's popular right now." But something deeper: "the community's reading patterns suggest you'll enjoy this."

The challenge is building that brain. And the technique we use is called **collaborative filtering**.

Why not just use tag matching? Because tags are noisy. Two stories might both be tagged "Harry Potter" but one is a 500-word comedy crack-fic and the other is a 500,000-word epic war drama. Tags alone can't capture the nuance of taste. But human behavior can. If 200 people bookmark both stories, there's a genuine connection there—regardless of what the tags say.

### Collaborative Filtering: Learning from What Others Liked

The word "collaborative" is the key. We're not analyzing the stories themselves—we're analyzing the *behavior* of readers. Specifically, we're looking at bookmarks.

Here's the intuition: if Reader A bookmarks Story X and Story Y, and Reader B also bookmarks Story X and Story Y, there's a connection between those two readers. If Reader B also bookmarks Story Z, then Story Z is a good recommendation for Reader A.

We don't need to know anything about the stories themselves. We don't need to read them. We don't need to understand their themes or plot points. We just need to know that the same people tend to bookmark the same things. That's the magic of collaborative filtering—the crowd's behavior teaches us about taste.

Think about how this works in practice. Imagine Alice bookmarks these stories:

- "Midnight Sun" by JaneAuthor
- "Eclipse Reimagined" by BobWriter
- "Bella's War" by CarolNerd
- "The Last Twilight" by DaveStories

And Bob bookmarks these:

- "Midnight Sun" by JaneAuthor
- "Eclipse Reimagined" by BobWriter
- "Phoenix Rising" by EveTales

Alice and Bob share two stories in common: "Midnight Sun" and "Eclipse Reimagined." That's a strong signal. When Alice discovers that Bob also liked "Phoenix Rising," that's a recommendation worth surfacing.

Now scale this up to thousands of readers. The patterns become even more powerful. If 500 readers bookmark both Story X and Story Y, that's not a coincidence—that's a genuine taste connection.

This is called **item-item collaborative filtering** because we're computing similarity between items (stories), not between users. "People who liked X also liked Y" is the item-item framing. We chose this approach because:

1. Items change more slowly than users—you write a story once, but a user's taste evolves. A story's bookmark set grows over time, but it doesn't fundamentally change. A user's preferences, on the other hand, shift constantly.
2. Item-item similarity is more stable and interpretable. "Stories A and B are similar because 50 readers bookmarked both" is easier to reason about than "Reader 123 and Reader 456 have similar tastes because they both liked 20 of the same stories."
3. We can precompute and cache item similarities efficiently. Story X's top 20 similar stories are the same for every reader who views Story X. Compute once, serve from cache.
4. New items get recommendations quickly as soon as a few people bookmark them. There's no cold-start problem for the recommendation *candidates*—only for the seed story.

There's also user-user collaborative filtering (finding similar readers), but item-item is better for our use case because we care about recommending *stories*, not finding *readers*.

### The Jaccard Coefficient: Overlap as Similarity

To measure how "similar" two stories are, we need a mathematical tool. Enter the **Jaccard coefficient**.

The Jaccard coefficient measures the overlap between two sets. If Story A was bookmarked by {Alice, Bob, Carol, Dave} and Story B was bookmarked by {Bob, Carol, Dave, Eve}, the overlap is {Bob, Carol, Dave}—that's 3 elements. The union is {Alice, Bob, Carol, Dave, Eve}—5 elements. The Jaccard coefficient is 3/5 = 0.6.

```
            Set A          Set B
          ┌───────┐     ┌───────┐
          │       │  ●●●│       │
          │  ●●   │●●●  │   ●●● │
          │       │  ●●●│       │
          └───────┘     └───────┘

          Jaccard(A, B) = |A ∩ B| / |A ∪ B|
                        = 3 / 5
                        = 0.6
```

A Jaccard coefficient of 1.0 means the stories have identical sets of bookmarkers (perfect overlap). A coefficient of 0.0 means no overlap at all. In practice, we see values between 0.0 and 0.3 for most story pairs—because most stories are bookmarked by different readers. A Jaccard of 0.2 is actually quite high and indicates a strong recommendation candidate.

Here's why Jaccard works well for us: it naturally handles stories with different numbers of bookmarks. A story with 100 bookmarks and a story with 50 bookmarks might share 10 readers. Raw count (10) seems low, but as a fraction of their combined reach, it might be very significant. Jaccard captures that significance.

Contrast this with a simpler metric like "shared bookmarkers count." Story A (1000 bookmarkers) and Story B (1000 bookmarkers) sharing 50 readers would score higher than Story C (10 bookmarkers) and Story D (10 bookmarkers) sharing 8 readers—even though the second pair is clearly more similar. Jaccard fixes this by normalizing by the union size.

Let's work through another example. Story X has 50 bookmarkers. Story Y has 30 bookmarkers. They share 15 bookmarkers. The Jaccard is 15 / (50 + 30 - 15) = 15/65 ≈ 0.23. That's a strong recommendation—23% of the combined audience overlaps. Now imagine Story Z also has 30 bookmarkers but shares only 3 with Story X. Jaccard = 3 / (50 + 30 - 3) = 3/77 ≈ 0.04. Much weaker. The co-occurrence count (15 vs. 3) directly translates to Jaccard similarity.

One subtlety: the Jaccard formula in the SQL query uses `cooccur_count::float` to ensure floating-point division. Without the `::float` cast, PostgreSQL would perform integer division, truncating the result to 0 for most pairs. This is a common gotcha in SQL—one that can silently produce incorrect rankings if you're not careful.

### Item-Item Collaborative Filtering

FicHub specifically uses **item-item** collaborative filtering rather than user-user. Here's why:

In user-user filtering, you'd find "readers similar to you" and recommend what they liked. But FicHub doesn't have user accounts—readers are anonymous (we only hash their profile URLs). So user-user filtering would require tracking individual reading histories, which raises privacy concerns and creates cold-start problems for new visitors.

Item-item filtering sidesteps this entirely. We don't need to know anything about the current reader. We just need to know: "This story is similar to that story, based on the community's bookmarking patterns." Then when someone views Story X, we show stories similar to Story X—regardless of who the reader is.

The item-item approach also has a practical advantage: we can precompute similarities and cache them. Story X's top 20 similar stories are the same for every reader who views Story X. So we compute once and serve from cache.

### The Co-occurrence Table: Tracking Which Fics Appear Together

The heart of the recommendation engine is the `fic_bookmark_cooccur` table. Every time a user bookmarks two stories, we increment the co-occurrence count for that pair.

```
work_a                  | work_b                  | cooccur_count
------------------------|-------------------------|---------------
"Midnight Sun"          | "Eclipse Reimagined"    | 47
"Midnight Sun"          | "Bella's War"           | 23
"Eclipse Reimagined"    | "Bella's War"           | 31
"Phoenix Rising"        | "The Last Twilight"     | 12
...
```

Notice the `CHECK (work_a < work_b)` constraint in the database schema. This ensures that each pair is stored only once—alphabetically ordered. Without this, we'd have both `(A, B)` and `(B, A)` and need to query both directions. The constraint guarantees that `(work_a, work_b)` is always in sorted order.

Let's trace through a concrete example. Say Alice bookmarks stories A, B, and C. Bob bookmarks stories A, B, D, and E. Carol bookmarks stories A, C, D, and F. After processing all three users, the co-occurrence table would contain:

```
work_a | work_b | cooccur_count
-------|--------|---------------
A      | B      | 2          (Alice + Bob)
A      | C      | 2          (Alice + Carol)
A      | D      | 1          (Bob + Carol)
A      | E      | 1          (Bob only)
A      | F      | 1          (Carol only)
B      | C      | 1          (Alice only)
B      | D      | 1          (Bob only)
B      | E      | 1          (Bob only)
C      | D      | 1          (Carol only)
C      | F      | 1          (Carol only)
```

Story A appears in the most pairs (5 pairs) because Alice, Bob, and Carol all bookmarked it. Stories B and C each appear in 4 pairs. The co-occurrence count tells us which stories are most connected to each other.

When someone views Story A, the engine looks up all rows where Story A appears (as either `work_a` or `work_b`), computes Jaccard for each candidate, and returns the top matches. In this example, Stories B and C would rank highest for Story A (both with co-occurrence of 2, and both having favorable Jaccard scores).

The full schema:

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
CREATE INDEX IF NOT EXISTS idx_cooccur_a ON fic_bookmark_cooccur(work_a);
CREATE INDEX IF NOT EXISTS idx_cooccur_b ON fic_bookmark_cooccur(work_b);
```

The indexes on `work_a` and `work_b` are crucial. When we look up recommendations for a story, we need to find all rows where the story appears as either `work_a` or `work_b`. The two indexes make both lookups fast. Without them, every recommendation request would require a full table scan—unacceptable at scale.

The co-occurrence count is a raw number: how many users bookmarked both stories. But raw counts aren't enough. A story with 10,000 bookmarks will have high co-occurrence with almost everything. We need to normalize by the total number of bookmarks for each story—which is where the Jaccard coefficient comes in.

### The FicWorks Table: Tracking Bookmark Counts

To compute Jaccard, we need to know how many people bookmarked each story. The `fic_works` table tracks this:

```sql
CREATE TABLE IF NOT EXISTS fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

The `favouriter_count` is incremented every time the collection worker discovers a new user who bookmarked this story. It's the `|A|` and `|B|` in the Jaccard formula.

### The SQL Query for Finding Similar Works

Here's the SQL that computes the Jaccard coefficient on the fly:

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

Let's break this down piece by piece:

1. **`seed` CTE**: Gets the favouriter count for the story we're recommending for (the "seed"). This is a single-row result that we'll use in the denominator.

2. **`candidates` CTE**: Finds all stories that share bookmarks with the seed. The `CASE WHEN work_a = $1 THEN work_b ELSE work_a END` expression is clever: since our seed could appear as either `work_a` or `work_b` (it's always stored alphabetically), the `CASE` picks out the *other* work in each pair. This gives us a clean list of candidate story IDs.

3. **The Jaccard calculation**: `cooccur_count::float / (s.favouriter_count + fw.favouriter_count - cooccur_count)`. This is the standard Jaccard formula. The denominator `|A| + |B| - |A ∩ B|` is mathematically equivalent to `|A ∪ B|`. The `::float` cast ensures we get decimal division instead of integer truncation.

4. **`JOIN fic_works fw`**: We need the candidate's favouriter count for the denominator.

5. **`CROSS JOIN seed s`**: Brings the seed's favouriter count into every row.

6. **`WHERE c.candidate_id != $1`**: Don't recommend a story to itself.

7. **`ORDER BY jaccard DESC`**: Most similar stories first.

8. **`LIMIT $2`**: Return the top N candidates.

This query is surprisingly efficient. The co-occurrence table has indexes on both `work_a` and `work_b`, so the `WHERE work_a = $1 OR work_b = $1` clause uses index scans. For a story with 50 bookmarkers, this query returns maybe 50-200 candidates and runs in under 50ms.

⚠️ **Watch Out**: The Jaccard formula can produce division by zero if both stories have zero bookmarkers. The `favouriter_count` column starts at 0, and the engine handles this by checking the count before doing collaborative filtering. If the seed has zero bookmarkers, we skip straight to tag-based fallback—there's nothing meaningful to compute.

### The RecQuery and RecResult Structs

The engine speaks through two clean data structures. `RecQuery` is what goes in:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecQuery {
    pub url_id: String,
    #[serde(default = "default_rec_n")]
    pub n: usize,
    pub site_domain: Option<String>,
}

const fn default_rec_n() -> usize {
    20
}

impl Default for RecQuery {
    fn default() -> Self {
        Self {
            url_id: String::new(),
            n: 20,
            site_domain: None,
        }
    }
}
```

`RecQuery` asks: "Give me N recommendations for this story, optionally filtered to a specific site." The `site_domain` filter is handy when someone only wants recommendations from the same platform—they're reading on AO3 and want more AO3 stories, not FanFiction.net ones. The default of 20 recommendations gives a good balance between variety and manageability.

`RecResult` is what comes out:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecResult {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub site_domain: String,
    pub summary: String,
    pub score: f64,
    pub community_score: i32,
    pub download_urls: HashMap<String, String>,
}
```

Each result carries the story's metadata (so the frontend can display it immediately without making another request) plus two scores: the engine's computed `score` (0.0 to ~1.0) and the `community_score` (net votes from users). The `download_urls` map is populated when the user has export-ready files, and left empty otherwise.

The `score` field is the blended result of collaborative filtering, tag matching, and voting boost. It's not a probability or a percentage—it's a relative ranking number. A score of 0.8 doesn't mean "80% likely to enjoy." It means "this candidate scored 0.8 on our blending formula." What matters is the *relative* ordering: a story with score 0.8 ranks above one with score 0.6.

### The RecommendationEngine Struct

The engine itself is beautifully simple—a single field:

```rust
pub struct RecommendationEngine {
    pub db: PgPool,
}

impl RecommendationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { db: pool }
    }
}
```

It's a database wrapper. All the intelligence lives in SQL queries and a bit of Rust glue code. No machine learning models, no TensorFlow, no GPU clusters. Just clever queries and well-structured data.

This is a deliberate design choice. Machine learning models require training data, periodic retraining, hyperparameter tuning, and infrastructure to serve predictions. SQL queries are self-contained, debuggable, and run on the same PostgreSQL server we already have. For a project like FicHub—where the data is relatively simple (bookmarks and votes) and the recommendation problem is straightforward (find similar stories)—SQL-based collaborative filtering is the right level of complexity.

### The get_recommendations Function: Step by Step

The `get_recommendations` function follows a two-tier strategy:

```rust
pub async fn get_recommendations(
    &self,
    query: &RecQuery,
    config: &Config,
) -> Result<Vec<RecResult>, AppError> {
    let limit = query.n.min(config.rec_max_recommendations);
    if limit == 0 {
        return Ok(Vec::new());
    }

    // 1. Check precomputed cache
    let cached = check_cache(&self.db, query, config, limit).await?;
    if !cached.is_empty() {
        return Ok(cached);
    }

    // 2. Compute live
    compute_live(&self.db, query, config, limit).await
}
```

This is the classic **cache-or-compute** pattern. Precomputed results are fast (one query), but they get stale. Live computation is thorough but costs more database time. The cache TTL (time-to-live) is configurable via `config.rec_cache_ttl_hours`—typically a few hours.

**The Cache Check** queries the `precomputed_recommendations` table:

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

The `computed_at > NOW() - ($2 * INTERVAL '1 hour')` clause ensures we don't serve stale results. If the cache is older than `rec_cache_ttl_hours` hours, we treat it as a miss and recompute. This is a simple TTL-based invalidation strategy—no complex cache coherence protocols, no event-driven invalidation. Just "recompute every few hours."

When the cache includes a `site_domain` filter, the query adds an extra condition:

```sql
JOIN fic_works fw ON fw.url_id = pr.recommended_url_id
WHERE pr.url_id = $1
  AND fw.site_domain = $2
  AND pr.computed_at > NOW() - ($3 * INTERVAL '1 hour')
```

This ensures that filtered recommendations only come from the requested site. The cache is stored *without* the site filter (all recommendations are cached together), and the filter is applied at query time. This means one cache entry serves all site-filtered requests—we just join with `fic_works` to filter.

After fetching from the cache, the results are enriched with community scores:

```rust
let community = get_community_scores(db).await?;

let results = rows
    .into_iter()
    .map(|r| {
        let cs = community.get(&r.recommended_url_id).copied().unwrap_or(0);
        RecResult {
            url_id: r.recommended_url_id,
            title: r.title,
            author: r.author,
            words: r.words,
            chapters: r.chapters,
            status: r.status,
            site_domain: r.source,
            summary: r.description,
            score: r.score as f64,
            community_score: cs,
            download_urls: HashMap::new(),
        }
    })
    .collect();
```

Notice that community scores are *not* cached—they're always fetched fresh. This is because votes change frequently (users can vote at any time), and we want the displayed vote counts to be current. The collaborative filtering scores are stable enough to cache, but vote counts should always reflect the latest state.

When the cache misses, `compute_live` does the heavy lifting. Let's walk through it step by step.

**Step 1: Get the seed story's metadata.**

```rust
let seed = sqlx::query_as::<_, (i32, String, String)>(
    r#"SELECT COALESCE(fw.favouriter_count, 0),
              COALESCE(fi.title, ''),
              COALESCE(fi.author, '')
       FROM fic_works fw
       JOIN fic_info fi ON fi.id = fw.url_id
       WHERE fw.url_id = $1"#,
)
.bind(&query.url_id)
.fetch_optional(db)
.await
.ok_or_else(|| AppError::NotFound(format!("Work {} not found", query.url_id)))?;
```

We need three things: the favouriter count (to compute Jaccard and decide whether to use tag fallback), the title (for keyword matching), and the author (for author matching). The `COALESCE` handles NULL values gracefully—defaulting to 0 for count and empty strings for text.

**Step 2: Find co-occurrence candidates.**

The big Jaccard query runs here, returning stories ranked by similarity. This is the core collaborative filtering signal.

**Step 3: Tag fallback if needed.**

If the seed story has very few bookmarkers (below `rec_min_favouriters_for_collab`), collaborative filtering won't have enough data. So we fall back to matching by author and title keywords. We'll cover this in detail in the "cold start" section.

**Step 4: Merge and blend scores.**

Collaborative and tag-based scores are combined using a weighted formula:

```rust
let mut scored: HashMap<String, CandidateScore> = HashMap::new();

for c in &cooccur {
    let jaccard = c.jaccard.unwrap_or(0.0);
    scored.insert(
        c.candidate_id.clone(),
        CandidateScore {
            url_id: c.candidate_id.clone(),
            score: jaccard,
            tag_score: 0.0,
            community_score: 0,
        },
    );
}

for (tag_id, tag_score_val) in &tag_candidates {
    let entry = scored.entry(tag_id.clone()).or_insert_with(|| CandidateScore {
        url_id: tag_id.clone(),
        score: 0.0,
        tag_score: 0.0,
        community_score: 0,
    });
    entry.tag_score = *tag_score_val;
}
```

The `HashMap` keyed by `url_id` ensures we don't have duplicate candidates. A story that appears in both collaborative and tag results gets both scores stored—collaborative in `score` and tag in `tag_score`.

**Step 5: Blend collaborative and tag scores.**

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

let mut candidates: Vec<CandidateScore> = scored.into_values().collect();
for c in &mut candidates {
    let collab = c.score;
    let tag = c.tag_score;
    c.score = collab * weight + tag * (1.0 - weight);
}
```

With 0 favouriters, `weight` is 0.0—pure tag-based. With 5+ favouriters, `weight` is 1.0—pure collaborative. This smooth transition means new stories still get reasonable recommendations while mature stories rely on real user behavior.

**Step 6: Apply voting boost.**

Community votes are fetched and applied as a logarithmic multiplier:

```rust
let net_votes = get_community_votes(db, &candidates).await?;

let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

**Step 7: Sort and take the top N.**

```rust
candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
let top: Vec<CandidateScore> = candidates.into_iter().take(limit).collect();
```

**Step 8: Fetch metadata and build results.**

Each candidate's full metadata (title, author, word count, etc.) is fetched from `fic_info` and wrapped in a `RecResult`.

**Step 9: Post-filter by site domain.**

```rust
if let Some(ref domain) = query.site_domain {
    results.retain(|r| r.site_domain == *domain);
}
```

This filter is applied *after* scoring so that the ranking is still based on the full dataset, but the user only sees stories from their preferred site.

### The Cold Start Problem

"What if nobody has bookmarked this story yet?"

This is the classic **cold start problem** in recommendation systems. Without bookmark data, collaborative filtering can't compute similarities. A brand-new story on a small site might have zero bookmarkers. Even a popular story might have only 2-3 bookmarkers, which isn't enough for meaningful Jaccard coefficients.

FicHub handles this with a **tag-based fallback**. When the seed has fewer than `rec_min_favouriters_for_collab` bookmarkers (configurable, typically 5), the engine switches strategies:

```rust
let min_collab = config.rec_min_favouriters_for_collab as i32;
let use_tag_fallback = favouriter_count < min_collab;

let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The `fetch_tag_candidates` function does two things:

**1. Find stories by the same author.** This is the strongest signal—if you liked one story by an author, you'll probably like their others. We use an `ILIKE` query with wildcard patterns:

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
            results.push((row.id, 0.8));
        }
    }
}
```

The `ILIKE` operator is PostgreSQL's case-insensitive `LIKE`. It matches "J.K. Rowling", "j.k. rowling", and "J.k. ROWLING" equally. The `%author%` wildcards catch partial matches too—important when authors have slightly different pen names across sites.

Same-author matches get a score of 0.8—high but not perfect. The author might write in very different styles or fandoms, so we can't assume identity.

**2. Find stories with matching title keywords.** This is a weaker signal, but it works surprisingly well for stories with distinctive words in their titles:

```rust
let keywords: Vec<&str> = seed_title
    .split_whitespace()
    .filter(|w| w.len() >= 3)
    .collect();

for kw in keywords {
    if results.len() >= limit {
        break;
    }
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
            results.push((row.id, 0.5));
        }
    }
}
```

The `filter(|w| w.len() >= 3)` skips tiny words like "the" or "a" that would match everything. We search one keyword at a time, stopping as soon as we have enough results. Title matches get a score of 0.5—meaningful but weaker than author matches.

The `seen` set prevents the same story from appearing twice. If a story matches both an author search and a title keyword search, it gets the higher score (0.8 from the author match, because author results are added first).

🧪 **Try It Yourself**: Think about the Jaccard coefficient. If Story A has 20 bookmarkers and Story B has 30 bookmarkers, and they share 8 bookmarkers, what's the Jaccard? Answer: 8 / (20 + 30 - 8) = 8/42 ≈ 0.19. That's actually a pretty strong similarity—19% overlap in a system with thousands of potential bookmarkers. In practice, anything above 0.1 is worth recommending.

### Blending Collaborative and Tag Scores

The blending formula is elegant and well-tuned:

```
final_score = collaborative × weight + tag × (1 - weight)
weight = min(1.0, favouriter_count / 5.0)
```

Think of it as a sliding scale. When a story is brand-new (0 favouriters), `weight` is 0.0 and we rely entirely on tag matching. As bookmarkers accumulate, the weight shifts toward collaborative filtering. By 5+ bookmarkers, collaborative data is fully trusted.

This transition is smooth and continuous—there's no hard cutoff where we suddenly switch from one strategy to another. A story with 2 bookmarkers gets 40% collaborative and 60% tag. A story with 3 gets 60% collaborative and 40% tag. It gracefully evolves as data accumulates.

Why 5 as the threshold? It's a heuristic, but it's based on the observation that Jaccard coefficients become meaningful once there are about 5 shared bookmarkers. Below that, the signal is too noisy to be reliable. Above that, collaborative filtering outperforms tag matching.

### The Voting Boost: Community Favorites

Beyond collaborative and tag signals, the engine incorporates community voting. When users upvote or downvote a recommendation, those votes create a `community_score` for each candidate.

The boost formula uses a logarithmic curve:

```
boost = 1.0 + gamma × ln(1 + max(0, net_votes))
```

Why logarithmic? Because a story with +100 votes is better than one with +10 votes, but not 10× better. The logarithm captures diminishing returns—the first few votes matter more than the hundredth.

The `gamma` parameter (from config) controls how much votes influence the final ranking. A higher `gamma` means votes matter more; a lower `gamma` means the engine relies more on collaborative filtering.

After all signals are blended and boosted, the candidates are sorted by final score, and the top N are returned. Each result includes both the computed score and the community vote count, giving the frontend everything it needs to display a rich recommendation card.

### Putting It All Together

Let's trace a complete recommendation request:

1. User A visits a story's page and clicks "Recommendations."
2. The frontend calls `GET /api/v0/recommendations?url_id=abc123&n=20`.
3. The handler resolves the URL, checks the database, and calls `get_recommendations`.
4. The engine checks the cache—miss.
5. `compute_live` runs: gets seed metadata, queries co-occurrence, runs Jaccard, applies tag fallback if needed, blends scores, applies voting boost, sorts, and fetches metadata.
6. 20 `RecResult` objects are returned as JSON.
7. The results are cached in `precomputed_recommendations` for next time.
8. The frontend renders 20 recommendation cards with titles, authors, word counts, and scores.

The whole thing takes under 100ms for a warm cache hit, and typically under 500ms for a live computation. That's fast enough to feel instant.

The cache is precomputed by the `compute_and_cache` method:

```rust
pub async fn compute_and_cache(
    &self,
    url_id: &str,
    config: &Config,
) -> Result<(), AppError> {
    let query = RecQuery {
        url_id: url_id.to_string(),
        n: config.rec_max_recommendations,
        site_domain: None,
    };

    let results = compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;

    // Clear old cache entries for this work
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

This method is called periodically (or on-demand) to refresh the cache. It deletes old entries and inserts new ones with the current timestamp. The `rank` column preserves the ordering, so the cache query can `ORDER BY rank ASC` without recomputing scores.

The `precomputed_recommendations` table schema:

```sql
CREATE TABLE IF NOT EXISTS precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
CREATE INDEX IF NOT EXISTS idx_precomputed_url ON precomputed_recommendations(url_id, rank);
```

The `score` column uses `REAL` (single-precision float) instead of `DOUBLE PRECISION` to save space. We don't need double precision for recommendation scores—a float has plenty of precision for ranking. The `rank` column is a `SMALLINT` (2 bytes) since we'll never have more than 32,767 recommendations for a single story.

The composite primary key `(url_id, recommended_url_id)` ensures no duplicate recommendations, and the index on `(url_id, rank)` makes the cache query fast—we can efficiently look up all recommendations for a story, ordered by rank.

⚠️ **Watch Out**: The cache stores recommendations *without* the site filter. If you want site-filtered recommendations, the filtering happens at query time by joining with `fic_works`. This means the cache might contain 100 precomputed recommendations, but only 15 are from the requested site. The `LIMIT` is applied *after* filtering, so you might get fewer results than requested. This is a trade-off: one cache entry serves all filter combinations, but you can't guarantee a specific number of results for filtered queries.

🧪 **Try It Yourself**: Write a SQL query that finds the top 5 stories most similar to a story with url_id `'abc123'`, using the Jaccard formula from this chapter. Hint: you'll need the `fic_bookmark_cooccur` and `fic_works` tables, and the formula is `cooccur_count / (favouriter_count_a + favouriter_count_b - cooccur_count)`.

---

## Chapter 22: The Collection Worker

### Why Collect Data?

The recommendation engine is only as good as its data. Without knowing which users bookmark which stories, collaborative filtering has nothing to work with. The co-occurrence table starts empty. The Jaccard coefficients are meaningless. It's like a restaurant with no reviews—the food might be great, but nobody knows.

We need a system that continuously collects bookmark data from fanfiction sites—a background worker that crawls user profiles, discovers which stories they've bookmarked, and updates the co-occurrence counts. This worker runs 24/7, quietly building the knowledge base that powers recommendations.

The worker answers a simple question: "If I visit this story's page on AO3 and look at who bookmarked it, and then visit each of those users' profiles to see what else they bookmarked—what can I learn about which stories go together?"

It's a lot of web scraping. And it needs to be done carefully—politely, efficiently, and with respect for user privacy.

Consider the scale: a popular AO3 story might have 5,000+ bookmarkers. Each bookmarker might have 50+ bookmarks of their own. That's potentially 250,000+ bookmark relationships to discover for a single story. We obviously can't crawl all of that—we'd be making requests for days. But we don't need to. Even crawling 5 pages of bookmarkers (maybe 250 users) and 3 pages of their favourites (maybe 150 works per user) gives us enough data to build meaningful co-occurrence patterns.

The key insight is that we don't need comprehensive data—we need *sufficient* data. A co-occurrence count of 5 between two stories is meaningful, even if the true count is 50. The relative ordering (which story pairs have higher co-occurrence) is what matters, and that ordering stabilizes with surprisingly few data points.

### The CollectionWorker: A Robot That Works While You Sleep

The `CollectionWorker` is FicHub's background data collector. It's not a web request handler—it's a long-running process that polls a queue and does work without anyone asking.

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiter>,
}
```

Let's unpack each field:

- **`db`**: The PostgreSQL connection pool for reading and writing bookmark data. This is how the worker interacts with the same database that the recommendation engine reads from.

- **`redis`**: A Redis connection wrapped in a `Mutex` (since we share it across async tasks). Redis serves as the work queue—a lightweight, fast message broker that doesn't require a separate service.

- **`client`**: An HTTP client for making requests to fanfiction sites. This is a `reqwest::Client` with sensible defaults (timeouts, redirect policies, connection pooling).

- **`config`**: Configuration values like rate limits, maximum page counts, and cache TTLs. These let operators tune the worker's behavior without changing code.

- **`registry`**: The scraper registry, so we can look up site-specific scrapers when needed.

- **`fetchers`**: A list of `SiteFetcher` implementations—one per supported site. This is the polymorphic core: each site has its own scraping logic, but the worker treats them all the same.

- **`rate_limiters`**: Per-domain rate limiters that ensure we don't hammer any single site. Stored in a `HashMap<String, PerSiteRateLimiter>` keyed by domain.

The constructor registers a fetcher for each known site:

```rust
pub fn new(
    db: PgPool,
    redis: MultiplexedConnection,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
) -> Self {
    let mut fetchers: Vec<Box<dyn SiteFetcher>> = Vec::new();
    fetchers.push(Box::new(Ao3Fetcher));
    fetchers.push(Box::new(FfNetFetcher));
    fetchers.push(Box::new(XenForoFetcher));
    fetchers.push(Box::new(FictionPressFetcher));
    fetchers.push(Box::new(AdultFanFictionFetcher));
    fetchers.push(Box::new(HpFanFicFetcher));

    let mut rate_limiters = HashMap::new();
    for fetcher in &fetchers {
        let domain = fetcher.site_domain().to_string();
        let delay = fetcher.rate_limit_delay(&config, &domain);
        rate_limiters.insert(domain, PerSiteRateLimiter::new(delay));
    }

    Self {
        db,
        redis: Mutex::new(redis),
        client,
        config,
        registry,
        fetchers,
        rate_limiters,
    }
}
```

Each fetcher knows how to collect bookmarks from its specific site. AO3's bookmark page structure is completely different from FanFiction.net's, so each site needs its own implementation. The `rate_limiters` map is pre-populated with the correct delay for each site.

### The SiteFetcher Trait: Collecting Bookmarks from Each Site

The `SiteFetcher` trait defines the interface that every site-specific scraper must implement:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    /// Canonical domain for this site.
    fn site_domain(&self) -> &str;

    /// Per-site rate-limit delay in seconds.
    fn rate_limit_delay(&self, config: &Config, domain: &str) -> u64 {
        config
            .rec_site_rate_limits
            .get(domain)
            .copied()
            .unwrap_or(config.rec_default_delay_secs)
    }

    /// Collect users who have favourited / bookmarked the given work URL.
    async fn collect_favouriters(
        &self,
        client: &Client,
        work_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Collect the set of works favourited by a user.
    async fn collect_user_favourites(
        &self,
        client: &Client,
        user_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Compute a deterministic hash for a user's profile URL.
    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

Two core methods:

1. **`collect_favouriters`**: Given a story URL, return a list of user profile URLs for everyone who bookmarked it. On AO3, this means paginating through the "Bookmarks" tab of a work. On FanFiction.net, it might mean checking the "Favorited by" page. Each site has its own way of presenting this information.

2. **`collect_user_favourites`**: Given a user profile URL, return a list of story URL IDs they've bookmarked. On AO3, this means paginating through a user's "Bookmarks" page. On FanFiction.net, it means checking their "Favorites" page.

The `max_pages` parameter limits how deep we crawl. We don't need *every* bookmarker of a popular story—that could be thousands of pages. We just need enough to build meaningful co-occurrence counts. Typically, 5-10 pages per query is plenty.

The `rate_limit_delay` method has a default implementation that checks `config.rec_site_rate_limits` for an override. If the config has a specific delay for this domain, use it. Otherwise, fall back to `config.rec_default_delay_secs`.

The current implementations are stubs—they return empty vectors:

```rust
struct Ao3Fetcher;

#[async_trait]
impl SiteFetcher for Ao3Fetcher {
    fn site_domain(&self) -> &str {
        "archiveofourown.org"
    }

    async fn collect_favouriters(
        &self,
        _client: &Client,
        _work_url: &str,
        _max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())
    }

    async fn collect_user_favourites(
        &self,
        _client: &Client,
        _user_url: &str,
        _max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

The architecture is the focus—the trait defines the contract, and real scraping logic can be added later per-site without changing the worker. This is the **strategy pattern** in action: the worker doesn't know how to scrape AO3, but it knows how to call a `SiteFetcher` that does.

### SHA-256 User Hashing for Privacy

Notice the `user_hash` method. We never store raw user profile URLs. Instead, we compute a SHA-256 hash:

```rust
fn user_hash(&self, user_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_url.to_lowercase().as_bytes());
    hex::encode(hasher.finalize())
}
```

The `user_hash` column in the database is 64 hex characters—a one-way hash. We can't reverse it to find the original URL. We can't look up a user by their profile URL (we'd need to hash it first). And if the database were ever compromised, the user identities would be protected.

The `to_lowercase()` ensures that `User123` and `user123` produce the same hash. Consistency matters more than case sensitivity here—we don't want the same user treated as two different people because of capitalization differences.

SHA-256 is a cryptographic hash function. While it's not strictly necessary for privacy (we're not trying to prevent brute-force attacks), it's the standard choice for deterministic hashing. It's fast, widely supported (via the `sha2` crate), and produces a uniform distribution of outputs.

The privacy model is important: FicHub collects reading behavior data but doesn't identify individuals. We know that "user hash abc123" bookmarked stories X, Y, and Z. We don't know who that person is, where they live, or what their AO3 username is. This is a privacy-by-design approach.

### The PerSiteRateLimiter: Being Polite to Websites

Fanfiction sites are not big tech companies. They're often run by volunteers on modest hardware. AO3, for example, runs on donations and has limited server capacity. Hammering it with rapid-fire requests would be rude at best, and get our IP banned at worst.

The `PerSiteRateLimiter` ensures we wait between requests to each site:

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
}
```

The implementation uses an atomic compare-and-swap (CAS) loop to coordinate concurrent access without locks:

```rust
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
            // Enough time has passed — try to claim this slot.
            if self.last_request
                .compare_exchange(last, now, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
            // CAS failed — another thread claimed a slot. Retry.
        } else {
            // Still within cooldown — sleep for the remainder.
            let remaining = delay_nanos - elapsed as u64;
            tokio::time::sleep(Duration::from_nanos(remaining)).await;
        }
    }
}
```

Here's how it works:

1. Read the current time and the last request timestamp.
2. If enough time has elapsed, try to atomically claim the slot by updating `last_request`.
3. If the CAS succeeds, we're good—proceed with the request.
4. If the CAS fails (another thread jumped in between our load and our CAS), retry the whole check with a fresh timestamp.
5. If not enough time has elapsed, sleep until the cooldown expires, then loop back to try claiming a slot.

The `last_request` field is initialized to 0, which represents "never requested." Since any real timestamp is much larger than 0, the first caller always passes the check immediately.

Why use atomic CAS instead of a Mutex? Because we want the check-and-update to be as fast as possible. A Mutex would block other threads for the entire duration of the critical section. The CAS loop only blocks briefly—just long enough to check and update a single integer.

⚠️ **Watch Out**: The rate limiter uses `AtomicI64` and `Ordering::AcqRel`. This is important—without the correct memory ordering, the CAS loop could see stale values and send requests too quickly. The `Acquire` load ensures we see the latest write from any other thread, and `AcqRel` ensures our write is visible to subsequent loads by other threads. Getting the ordering wrong could cause the rate limiter to fail silently.

### The Redis Queue: A To-Do List for the Collector

Redis provides a simple, fast queue using lists. Each site gets its own queue:

```
collection_queue:archiveofourown.org
collection_queue:fanfiction.net
collection_queue:forums.spacebattles.com
collection_queue:fictionpress.com
collection_queue:adult-fanfiction.org
collection_queue:hpfanfic.com
```

Why separate queues per site? Because different sites have different rate limits and processing characteristics. AO3's bookmark pages are structured and parse quickly. FanFiction.net's pages are older and sometimes inconsistent. SpaceBattles uses XenForo, which has its own quirks. By separating queues, we can process each site at its own pace without one slow site blocking the others.

The `QueueItem` struct represents a single unit of work:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub url_id: String,
    pub site_domain: String,
    pub site_work_id: String,
}
```

When a new story is scraped or requested, we enqueue it for collection. The worker picks it up and processes it.

Why Redis and not PostgreSQL for the queue? Redis is an in-memory data store—it's orders of magnitude faster for queue operations. A `LPUSH`/`LPOP` pair completes in microseconds, while a PostgreSQL `INSERT` + `SELECT FOR UPDATE` + `DELETE` cycle takes milliseconds. For a queue that processes hundreds of items per minute, that difference matters.

Redis also doesn't need persistence for the queue. If the worker restarts, it just re-enqueues anything that was in progress (or accepts the loss—these aren't critical tasks). And Redis's `LPUSH`/`LPOP` operations are atomic, so we don't need additional locking. The `Mutex<MultiplexedConnection>` in the worker is for sharing the connection across async tasks, not for queue atomicity.

The queue is intentionally simple. No priorities, no retries, no dead-letter queues. Items are processed in FIFO order, and failures are logged and forgotten. This simplicity is a feature—it makes the system easy to understand, debug, and operate. For FicHub's scale (hundreds of items per day, not millions per second), this simplicity is the right trade-off.

### The Enqueue Function: Adding Work to the Queue

Adding work to the queue is a single Redis command:

```rust
pub async fn enqueue(
    &self,
    url_id: &str,
    site_domain: &str,
    site_work_id: &str,
) -> Result<(), redis::RedisError> {
    let item = QueueItem {
        url_id: url_id.to_string(),
        site_domain: site_domain.to_string(),
        site_work_id: site_work_id.to_string(),
    };
    let json = serde_json::to_string(&item)
        .expect("QueueItem serialisation should not fail");

    let key = format!("collection_queue:{}", site_domain);
    let mut conn = self.redis.lock().await;

    redis::cmd("LPUSH")
        .arg(&[key.as_str(), json.as_str()])
        .query_async(&mut *conn)
        .await
}
```

`LPUSH` pushes to the left of a Redis list. The worker uses `LPOP` (pop from the left) to consume items in FIFO order. It's the simplest queue possible—no message brokers, no consumer groups, no complexity.

The `QueueItem` is serialized to JSON before pushing. This makes the queue inspectable (you can `LRANGE` the key to see pending items) and debuggable (the JSON is human-readable).

### The Run Function: The Main Loop

The worker's `run` method is an infinite loop that polls all site queues:

```rust
pub async fn run(&self) {
    info!("Collection worker started — polling Redis queues for all known sites");

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
                // Apply rate limit before making HTTP requests.
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }

                match serde_json::from_str::<QueueItem>(&item_str) {
                    Ok(item) => {
                        debug!("Processing {} from {}", item.url_id, domain);
                        if let Err(e) = self.process_work(item).await {
                            error!("Error processing work on {}: {}", domain, e);
                        }
                    }
                    Err(e) => {
                        warn!("Invalid queue item on {}: {}", domain, e);
                    }
                }
            }
        }

        // Brief sleep to avoid busy-looping Redis when queues are empty.
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

Notice the 100ms sleep at the end. Without it, the loop would spin as fast as possible, burning CPU while waiting for new items. The 100ms pause is short enough to feel responsive (a new item will be picked up within ~100ms of being enqueued) but long enough to avoid wasteful polling.

Each site's queue is checked independently. If AO3 has 5 items and FanFiction.net has 0, the worker processes all 5 AO3 items before moving on (well, interleaved—each loop iteration checks all queues once). This ensures we don't starve one site while another is busy.

Error handling is pragmatic: if a queue item fails to parse, we log a warning and skip it. If processing fails, we log an error and move on. We don't retry failed items immediately—they'd just fail again. In a production system, you might push failed items to a dead-letter queue for investigation, but for FicHub's scale, logging and moving on is sufficient.

### Processing a Work: Step by Step

The `process_work` method is where the real work happens. It's a multi-step procedure:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    // Find the right fetcher for this site
    let fetcher = self.fetchers
        .iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or_else(|| format!("No SiteFetcher for domain {}", item.site_domain))?;

    let work_url = format!("https://{}/works/{}", item.site_domain, item.site_work_id);

    // Step A: Fetch favouriters (who bookmarked this work?)
    let favouriters = fetcher
        .collect_favouriters(&self.client, &work_url, self.config.rec_max_favourite_pages)
        .await?;

    let mut new_user_count: u32 = 0;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // Skip users already recorded for this work
        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)",
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or(false);

        if already_exists {
            continue;
        }

        new_user_count += 1;

        // Ensure the fic_works row exists (or update counter)
        sqlx::query(
            r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count,
                                       first_favourite_scraped, last_favourite_scraped)
               VALUES ($1, $2, $3, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               ON CONFLICT (url_id) DO UPDATE SET
                   favouriter_count    = fic_works.favouriter_count + 1,
                   last_favourite_scraped = CURRENT_TIMESTAMP"#,
        )
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .bind(&item.site_work_id)
        .execute(&self.db)
        .await?;

        // Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .execute(&self.db)
        .await?;

        // Step B: Fetch this user's favourite works
        let user_favourites = fetcher
            .collect_user_favourites(
                &self.client,
                user_url,
                self.config.rec_max_user_favourite_pages,
            )
            .await?;

        // Step C: Update co-occurrence for all pairs in the set
        if user_favourites.len() >= 2 {
            self.update_cooccurrence(&item.site_domain, &user_favourites)
                .await?;
        }
    }

    // Step D: Ensure the fic_works row exists even with no new users
    if new_user_count == 0 {
        sqlx::query(
            r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count)
               VALUES ($1, $2, $3, 0)
               ON CONFLICT (url_id) DO UPDATE SET
                   last_favourite_scraped = CURRENT_TIMESTAMP
               WHERE fic_works.first_favourite_scraped IS NULL"#,
        )
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .bind(&item.site_work_id)
        .execute(&self.db)
        .await?;
    } else {
        info!(
            "Processed {} on {} — {} new user(s), {} favouriter(s) total",
            item.url_id, item.site_domain, new_user_count, favouriters.len(),
        );
    }

    Ok(())
}
```

Let's trace through the steps:

**Step A**: Call `collect_favouriters` to get user profile URLs from the site's bookmark page. The `max_pages` parameter from config limits how deep we crawl.

**For each new user** (one we haven't already recorded for this story):

**Step B**: Call `collect_user_favourites` to get the full list of stories this user has bookmarked. This is the expensive part—one HTTP request per page of their bookmarks, paginated.

**Step C**: Update co-occurrence counts for all pairs of works in that user's favourite set. If they bookmarked 10 stories, that's C(10,2) = 45 pairs to update.

**Step D**: If no new users were found (all already in the database), still ensure the `fic_works` row exists. This handles the case where a story was already fully processed—the row should still exist with accurate metadata.

The `ON CONFLICT DO NOTHING` on the bookmark insert prevents duplicate entries. The `ON CONFLICT DO UPDATE` on `fic_works` increments the favouriter count. These upsert patterns are essential—the worker might re-process a story if it's re-enqueued, and we need idempotency.

⚠️ **Watch Out**: The `process_work` method does a lot of database queries—one `EXISTS` check, one upsert, one insert, and one co-occurrence update *per user*. For a story with 200 bookmarkers, that's 800+ database queries. This is fine for a background worker running at moderate speed, but it would be problematic in a request handler. The worker's isolation from the API layer is important.

### Finding What Each User Bookmarked: The Co-occurrence Update

The `update_cooccurrence` method is a simple nested loop:

```rust
async fn update_cooccurrence(
    &self,
    site_domain: &str,
    works: &[String],
) -> Result<(), sqlx::Error> {
    for i in 0..works.len() {
        for j in (i + 1)..works.len() {
            let (work_a, work_b) = if works[i] < works[j] {
                (works[i].as_str(), works[j].as_str())
            } else {
                (works[j].as_str(), works[i].as_str())
            };

            sqlx::query(
                r#"INSERT INTO fic_bookmark_cooccur
                       (work_a, work_b, site_domain, cooccur_count)
                   VALUES ($1, $2, $3, 1)
                   ON CONFLICT (work_a, work_b) DO UPDATE SET
                       cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                       last_updated  = CURRENT_TIMESTAMP"#,
            )
            .bind(work_a)
            .bind(work_b)
            .bind(site_domain)
            .execute(&self.db)
            .await?;
        }
    }
    Ok(())
}
```

For a user who bookmarked 5 stories, this creates C(5,2) = 10 pairs. Each pair gets its co-occurrence count incremented by 1. The `work_a < work_b` ordering is enforced by the code (sorting the pair) and validated by the `CHECK` constraint in the database schema.

The `ON CONFLICT DO UPDATE` means: if this pair already exists, add 1 to the count. If it doesn't, insert with count 1. This is how the co-occurrence table grows over time—each user's bookmark set contributes to the pairwise counts.

The complexity is O(n²) for n bookmarked stories. For typical users (10-50 bookmarks), this is fast. For power users with hundreds of bookmarks, it could take a while—but that's acceptable for a background worker.

### Site-Specific Rate Limits

Different sites have different tolerance for scraping. AO3 is relatively robust but still has rate limits. FanFiction.net is older and more fragile. XenForo forums (SpaceBattles, Sufficient Velocity) have their own conventions.

The rate limits are configured per domain:

```toml
[recommender]
default_delay_secs = 5
site_rate_limits = { "archiveofourown.org" = 3, "fanfiction.net" = 10 }
```

AO3 gets 3 seconds between requests (it's a modern Rails app with decent capacity). FanFiction.net gets 10 seconds (it's an older PHP app that struggles under load). The default for unknown sites is 5 seconds.

The `rec_max_favourite_pages` config option limits how many pages of bookmarks we crawl per query. More pages means more data but slower collection. For most sites, 5-10 pages captures the majority of bookmarkers.

The `rec_max_user_favourite_pages` config option limits how many pages of a user's bookmarks we crawl. This is typically lower (3-5 pages) because we don't need a user's complete reading history—just enough to establish co-occurrence patterns.

⚠️ **Watch Out**: Rate limits are per-process, not per-IP. If FicHub runs multiple worker instances (e.g., in a Docker Compose setup), each one independently enforces its own rate limit. This could result in a site seeing faster request rates than intended—for example, two workers each waiting 3 seconds could send requests 1.5 seconds apart on average. For production deployments, consider using a distributed rate limiter (Redis-based token bucket) instead of the per-process atomic CAS approach.

### How the Worker Fits in the System

The worker is triggered in a few ways:

1. **On new story scrape**: When the scraper fetches a new story, it can enqueue it for collection. This is the most common trigger—new stories need bookmark data to appear in recommendations.

2. **On recommendation request**: If a user requests recommendations for a story not yet in the database, the handler enqueues it. This is a lazy-loading approach: we don't proactively scrape every story, only those that users actually ask about.

3. **On suggestion submission**: If a user suggests a recommendation involving a story not in the database, both stories are enqueued. This ensures that community suggestions can eventually be processed even for stories we haven't scraped yet.

4. **Periodic batch jobs**: An admin might enqueue popular stories for re-scraping to refresh their bookmark data. Bookmark counts change over time as new readers discover old stories, so periodic refreshes keep the data current.

This creates a self-reinforcing cycle: more requests → more stories enqueued → more data collected → better recommendations → more requests.

The worker's design also supports incremental updates. When we re-process a story, we don't re-scrape everything from scratch. The `ON CONFLICT DO NOTHING` on bookmark inserts and `ON CONFLICT DO UPDATE` on co-occurrence inserts mean we only process new data. Existing bookmarks are skipped, and co-occurrence counts are incremented (not reset). This makes refreshes efficient—you only pay for the new data.

### Error Handling and Resilience

The worker is designed to be resilient. If a single story fails to process (network error, parsing error, database error), the error is logged and the worker moves on. It doesn't crash, it doesn't retry endlessly, and it doesn't block other stories.

```rust
match serde_json::from_str::<QueueItem>(&item_str) {
    Ok(item) => {
        debug!("Processing {} from {}", item.url_id, domain);
        if let Err(e) = self.process_work(item).await {
            error!("Error processing work on {}: {}", domain, e);
        }
    }
    Err(e) => {
        warn!("Invalid queue item on {}: {}", domain, e);
    }
}
```

The `if let Err(e)` pattern means: if processing fails, log the error and continue. The next iteration of the loop will pick up the next item. Failed items are not re-queued automatically—they're lost. For a production system, you might want a dead-letter queue for failed items, but for FicHub's scale, logging and moving on is sufficient. The stories will eventually be re-enqueued by other triggers.

Network errors are the most common failure mode. Fanfiction sites occasionally go down, return errors, or serve unexpected HTML. The `SiteFetcher` implementations should handle these gracefully—returning empty results rather than panicking. The worker treats empty results as "no data available" rather than "error"—the story simply won't contribute to co-occurrence counts until it's re-processed successfully.

🧪 **Try It Yourself**: If a user has bookmarked 20 stories, how many co-occurrence pairs are created? The answer is C(20,2) = 20 × 19 / 2 = 190 pairs. That's 190 database updates for a single user. Now imagine a power user with 50 bookmarks—that's 1,225 pairs. This is why the rate limiter and background processing matter. These updates happen asynchronously, not in the API request path.

---

## Chapter 23: Community Suggestions

### Letting Users Suggest Recommendations

Collaborative filtering is powerful, but it's passive. It watches what people do, not what they think. Sometimes a reader finishes a story and *knows* exactly what to recommend next—but there's no bookmark data connecting the two.

Maybe the two stories are in different fandoms, so the same readers don't overlap. Maybe one story is brand-new and hasn't accumulated enough bookmarks yet. Maybe the connection is thematic—similar pacing, similar character dynamics, similar emotional arc—and that connection only a human reader would notice.

FicHub lets users submit recommendations manually. This is the community suggestions feature—a way for readers to say "I read this, and I think you'd love that."

### The Suggestion Table in the Database

The `recommendation_suggestions` table stores user-submitted links between stories:

```sql
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);
```

Key points:

- **`url_id`**: The story being recommended *for* (the seed). This is the story the user is currently reading.
- **`suggested_url_id`**: The story being recommended (the suggestion). This is the story they think others should read.
- **`submitted_by_ip`**: Who submitted it. We use IP addresses for identity, not user accounts (FicHub doesn't have a login system). This is a pragmatic choice for a tool that values simplicity. IP addresses aren't perfect identity—multiple people behind the same NAT share an IP—but they're good enough for a small community.
- **`comment`**: Optional text like "Great similar pacing" or "Same universe" or "If you liked the worldbuilding, this has even more." Comments add semantic context that the algorithm can't extract. They also help other voters decide whether to upvote the suggestion.
- **`UNIQUE(url_id, suggested_url_id, submitted_by_ip)`**: Prevents duplicate suggestions from the same IP. If someone tries to suggest the same story twice, it upserts (updates the comment and timestamp).

The `ON DELETE CASCADE` ensures that if a story is deleted from `fic_info`, all its suggestions and votes are automatically cleaned up. Referential integrity at the database level means we never have orphaned suggestions pointing to deleted stories.

The table is intentionally simple. There's no moderation status column, no visibility flag, no "approved" boolean. The philosophy is: let the community self-curate through voting. Bad suggestions don't need to be deleted—they just get downvoted and sink to the bottom. This keeps the system simple and avoids the need for a moderation queue.

### The Suggestion Struct

The `Suggestion` struct represents a suggestion in the API response:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: i64,
    pub suggested_url_id: String,
    pub comment: Option<String>,
    pub net_votes: i32,
    pub created: Option<DateTime<Utc>>,
}
```

It includes the suggestion's ID (for voting), the suggested story's ID, any comment, the net vote count (upvotes minus downvotes), and when it was created. The `net_votes` field is computed dynamically by joining with the votes table—it's not stored in the suggestions table itself.

### The Submit Suggestion Function

When a user submits a suggestion, the backend calls `submit_suggestion`:

```rust
pub async fn submit_suggestion(
    db: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    voter_ip: &str,
    comment: Option<&str>,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
               (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)
           DO UPDATE SET comment = EXCLUDED.comment, created = CURRENT_TIMESTAMP
           RETURNING id"#,
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(voter_ip)
    .bind(comment)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}
```

The `ON CONFLICT ... DO UPDATE` means: if this IP already suggested this pair, update the comment and timestamp rather than rejecting it. This is a nice UX touch—users can revise their suggestions without creating duplicates.

The `::inet` cast is important. PostgreSQL's `INET` type validates that the value is a proper IP address (IPv4 or IPv6). If someone passes garbage, the database will reject it with a clear error rather than storing invalid data.

The function returns the suggestion's ID, which the frontend uses to allow immediate voting on the new suggestion.

### Validating That Both Fics Exist

Before storing a suggestion, the API handler validates that both stories exist in the database. This is the `suggest_handler` in `routes.rs`:

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    if body.url_id.is_empty() || body.suggested_url.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id and suggested_url are required"
        })));
    }

    // Resolve the suggested_url to a url_id via the scraper
    let scraper = state.scraper_registry.find_scraper(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", body.suggested_url)))?;
    let meta = scraper.lookup(&state.http_client, &body.suggested_url).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let suggested_url_id = meta.url_id;

    let seed_exists = queries::get_fic_info(&state.db, &body.url_id).await?.is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id).await?.is_some();

    if !seed_exists {
        state.collection_worker.enqueue(&body.url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "seed fic not found in database — enqueued for collection"
        })));
    }

    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "suggested fic not yet collected — enqueued for processing"
        })));
    }

    // Submit the suggestion
    let suggestion_id = submit_suggestion(
        &state.db,
        &body.url_id,
        &suggested_url_id,
        "0.0.0.0",
        body.comment.as_deref(),
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "suggestion_id": suggestion_id,
    })))
}
```

If either story isn't in the database, we don't reject the suggestion outright. Instead, we **enqueue the missing story** for background collection. Once the worker scrapes it and stores its metadata, the suggestion can be processed. This is a graceful degradation—the user's intent is preserved even if the data isn't ready yet.

The `find_scraper` + `lookup` pattern resolves the URL to a canonical `url_id`. The user provides a URL (like `https://archiveofourown.org/works/12345`), and the scraper extracts the canonical ID (`12345` in this case, though the actual format is more complex).

### The Suggestion Modal in the Frontend

The suggestion flow in the frontend is designed to be frictionless. Here's the detailed user experience:

1. **User is on a story's page.** They've just read "Midnight Sun" and want to recommend "Eclipse Reimagined" to future readers.

2. **They click "Suggest a Recommendation."** This is a secondary action button—visible but not prominent. It's usually placed below the main recommendation list, so users only see it after browsing the existing recommendations.

3. **A modal appears** with:
   - A title: "Suggest a Recommendation for Midnight Sun"
   - An input field for the suggested story's URL. Placeholder text: "Paste the URL of a story you'd recommend..."
   - A text area for an optional comment. Placeholder text: "Why do you think this is a good match? (optional)"
   - A character counter for the comment (max 500 characters).
   - A "Submit" button (disabled until a URL is entered).
   - A "Cancel" link.

4. **The user pastes a URL.** The frontend might do quick client-side validation—checking that the URL looks like a valid fanfiction URL (contains "archiveofourown.org/works/" or "fanfiction.net/s/" etc.). If the URL doesn't look right, a gentle hint appears: "This doesn't look like a fanfiction URL. Please check and try again."

5. **They write a comment.** The comment is optional but encouraged. Good comments help other voters decide whether to upvote. The frontend might show a subtle prompt: "Tip: explain why this is a good match—it helps others vote."

6. **On submit, the frontend calls `POST /api/v0/recommendations/suggest`.** While waiting for the response, the submit button shows a loading spinner and is disabled to prevent double-submission.

7. **The backend resolves the URL to a `url_id`, validates both stories exist, and stores the suggestion.** If either story is missing from the database, the backend enqueues it for collection and returns an error message like "This story isn't in our database yet—we've queued it for collection."

8. **The modal shows a success message.** "Your suggestion has been submitted! Other readers can now vote on it." The modal auto-closes after 2 seconds.

9. **The suggestion appears in the community suggestions list.** The user can immediately see their suggestion with 0 votes and start voting on it themselves (yes, you can upvote your own suggestion—it's a feature, not a bug).

The frontend uses the `SuggestBody` struct:

```rust
pub struct SuggestBody {
    pub url_id: String,
    pub suggested_url: String,
    pub comment: Option<String>,
}
```

Note that the user provides the *URL* of the suggested story, not a `url_id`. The backend resolves the URL to a `url_id` using the scraper registry—the same mechanism used when someone requests an export. This means users don't need to know FicHub's internal ID system. They just paste a URL from wherever they found the story.

The URL resolution is important for user experience. Fanfiction URLs come in many forms:
- `https://archiveofourown.org/works/12345`
- `https://www.fanfiction.net/s/12345/1/`
- `https://forums.spacebattles.com/threads/story-name.12345/`
- `https://www.fictionpress.com/s/12345/1/`

The scraper registry knows how to parse each format and extract the canonical `url_id`. The user doesn't need to worry about URL normalization—they just paste whatever they have.

### Preventing Spam

Without user accounts, how do we prevent spam? Several layers work together:

1. **IP-based deduplication**: The `UNIQUE(url_id, suggested_url_id, submitted_by_ip)` constraint prevents the same IP from submitting the same pair twice. If someone wants to change their comment, the suggestion is updated rather than duplicated.

2. **Rate limiting** (in the frontend): The suggestion button can be throttled—e.g., one suggestion per 30 seconds. This prevents rapid-fire spam without being too restrictive for normal use.

3. **Vote-based filtering**: Low-voted suggestions appear at the bottom. High-voted ones rise to the top. Spammy suggestions that nobody upvotes simply sink into obscurity. The community self-curates.

4. **Manual moderation**: Admins can delete suggestions. The `ON DELETE CASCADE` handles cleanup—deleting a suggestion automatically removes its votes.

This isn't perfect, but it's sufficient for a small-to-medium community. The voting system does most of the heavy lifting—spam that nobody upvotes simply doesn't matter. And the IP-based deduplication prevents the most obvious abuse: one person flooding the system with hundreds of duplicate suggestions.

### How Suggestions Feed the Engine

Community suggestions don't just appear in a list—they feed back into the recommendation engine's scoring. When computing live recommendations, the engine queries `get_community_votes` to find net votes for each candidate:

```rust
async fn get_community_votes(
    db: &PgPool,
    candidates: &[CandidateScore],
) -> Result<HashMap<String, i32>, AppError> {
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();

    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#,
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}
```

The `ANY($1)` operator is PostgreSQL's way of doing `IN` with a dynamic list. The `JOIN` between suggestions and votes ensures we only count suggestions that have been voted on. The `GROUP BY` aggregates votes per suggestion.

This creates a feedback loop: good suggestions get upvoted → votes boost the recommendation's score → higher ranking → more visibility → more upvotes. Bad suggestions sink to the bottom naturally.

🧪 **Try It Yourself**: What happens if a user suggests a story that's already been recommended by the collaborative filtering engine? Nothing special—it's stored as a separate suggestion. But if other users upvote it, it gets a voting boost that might push it higher in the results. The collaborative and community signals reinforce each other.

---

## Chapter 24: Voting and Scoring

### Upvotes and Downvotes

The voting system lets the community curate recommendations. Anyone can upvote or downvote a suggestion, and those votes affect how prominently the suggestion appears in results.

It's a simple binary: **+1** for upvote, **-1** for downvote. No scales, no stars, no nuanced "3.5 out of 5" judgments. Binary voting is easy to implement, easy to understand, and produces clear signals.

Why binary? Because nuanced ratings are hard. Asking someone to rate a recommendation on a 5-star scale introduces decision fatigue—"Is this a 3 or a 4? What's the difference?" Binary is instant: thumbs up or thumbs down. The aggregate signal is clear, and the user experience is frictionless.

The voting system also has a subtle psychological effect: because votes are visible to everyone, they create social proof. A suggestion with +7 votes looks more trustworthy than one with +1. This helps voters make quick decisions—they can see what the community thinks before forming their own opinion. It's the same mechanism that makes Reddit upvotes and YouTube likes so influential.

There's also an important asymmetry in how we use votes. Upvotes *boost* recommendations—the boost formula uses `max(0, net_votes)`, so only positive votes increase the score. Downvotes *don't punish*—they just prevent the boost. A suggestion with -5 votes gets no boost (multiplier of 1.0), but it doesn't get penalized below its collaborative filtering baseline. This design choice reflects the philosophy that downvotes are a signal of "this isn't a good recommendation," not "this is a bad story."

### The Vote Table

The `recommendation_votes` table stores every vote:

```sql
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

Key design choices:

- **`PRIMARY KEY (suggestion_id, voter_ip)`**: Each IP can only have one vote per suggestion. If someone changes their mind, the vote is updated, not duplicated. This prevents vote-stuffing while allowing opinion changes.

- **`CHECK (vote IN (-1, 1))`**: The database rejects any value that isn't -1 or 1. No 0s, no 2s, no fractions. This is a hard constraint that prevents invalid data at the database level.

- **`ON DELETE CASCADE`**: If a suggestion is deleted, its votes are automatically removed. No orphaned votes.

- **`SMALLINT`**: The vote value is a 2-byte integer. Overkill for storing -1 or 1, but it's the smallest integer type PostgreSQL supports. The `SMALLINT` type saves space compared to `INT4` at scale—millions of votes add up.

The `INET` type for `voter_ip` ensures we store valid IP addresses. PostgreSQL's `INET` supports both IPv4 and IPv6, so it's future-proof.

### The Cast Vote Function

Casting a vote is a two-step operation:

```rust
pub async fn cast_vote(
    db: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    vote_value: i16,
) -> Result<i32, AppError> {
    let vote = vote_value.clamp(-1, 1);

    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip)
           DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP"#,
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(vote)
    .execute(db)
    .await?;

    let row: (Option<i32>,) = sqlx::query_as(
        r#"SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1"#,
    )
    .bind(suggestion_id)
    .fetch_one(db)
    .await?;

    Ok(row.0.unwrap_or(0))
}
```

The `clamp(-1, 1)` ensures only valid values are stored—defense in depth. The `ON CONFLICT ... DO UPDATE` means: if this IP already voted on this suggestion, replace the old vote with the new one. This is the "change your vote" behavior—users aren't locked into their first impression.

After storing the vote, the function immediately queries the new net score (`SUM(vote)`) and returns it. This gives the frontend the updated score in the same response, enabling optimistic UI updates.

The net score calculation is straightforward:

```sql
SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1
```

If 5 people upvoted (+1 each) and 2 downvoted (-1 each), the sum is 3. The `COALESCE` isn't needed here because the suggestion_id is guaranteed to exist (it's a foreign key), but the `unwrap_or(0)` handles the edge case of a suggestion with zero votes.

### Calculating Net Scores

The net score for a suggestion is simply:

```
net_votes = SUM of all votes for this suggestion
          = upvotes - downvotes
```

If 5 people upvoted and 2 downvoted, the net score is 3. If 3 upvoted and 7 downvoted, it's -4. The net score is always an integer, and it can be negative.

The net score is computed in two places:

1. **In `cast_vote`**: After storing a vote, we compute the new total and return it immediately.
2. **In `get_community_suggestions`**: When listing suggestions, we join with the votes table to get each suggestion's net score.

The `get_community_suggestions` function:

```rust
pub async fn get_community_suggestions(
    db: &PgPool,
    url_id: &str,
) -> Result<Vec<Suggestion>, AppError> {
    let rows: Vec<SuggestionRow> = sqlx::query_as::<_, SuggestionRow>(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(v.net, 0) AS net_votes, s.created
           FROM recommendation_suggestions s
           LEFT JOIN (
               SELECT suggestion_id, SUM(vote)::INT AS net
               FROM recommendation_votes
               GROUP BY suggestion_id
           ) v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           ORDER BY net_votes DESC, s.created DESC"#,
    )
    .bind(url_id)
    .fetch_all(db)
    .await?;

    let suggestions = rows
        .into_iter()
        .map(|r| Suggestion {
            id: r.id,
            suggested_url_id: r.suggested_url_id,
            comment: r.comment,
            net_votes: r.net_votes,
            created: r.created,
        })
        .collect();

    Ok(suggestions)
}
```

The `LEFT JOIN` is crucial—it ensures suggestions with zero votes still appear (with `net_votes = 0`). The `COALESCE(v.net, 0)` handles the case where no votes exist yet. And `ORDER BY net_votes DESC, s.created DESC` puts the most upvoted suggestions first, with newest as the tiebreaker.

The subquery `(SELECT suggestion_id, SUM(vote)::INT AS net FROM recommendation_votes GROUP BY suggestion_id)` pre-aggregates all votes, and the outer query joins it with the suggestions. This is more efficient than computing the sum for each suggestion individually.

### The Voting Boost in Recommendations

Community votes don't just affect the suggestion list—they feed into the recommendation engine's scoring.

When computing live recommendations, the engine fetches net votes for all candidate stories and applies a logarithmic boost:

```rust
let net_votes = get_community_votes(db, &candidates).await?;

let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The `get_community_votes` function aggregates votes across all suggestions for each candidate:

```rust
async fn get_community_votes(
    db: &PgPool,
    candidates: &[CandidateScore],
) -> Result<HashMap<String, i32>, AppError> {
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();

    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#,
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}
```

The logarithmic boost formula deserves detailed explanation:

```
boost = 1.0 + gamma × ln(1 + max(0, net_votes))
```

- **`max(0, net_votes)`**: Negative votes don't *reduce* the score—they just don't boost it. A story with -5 votes gets no boost (multiplier of 1.0). This prevents downvoted stories from being actively punished beyond their natural collaborative ranking. The philosophy is: downvotes are a signal that "this isn't a good recommendation," but they shouldn't make the story *worse* than its collaborative filtering score suggests.

- **`ln(1 + ...)`**: Natural logarithm of 1 + the vote count. This gives diminishing returns: going from 0 to 10 votes gives a bigger boost than going from 100 to 110. The `1 +` ensures we don't take `ln(0)` (which is undefined).

- **`gamma`**: A tuning parameter from the config. Higher values make votes matter more. Typical values are 0.3 to 0.7.

For example, with `gamma = 0.5`:

| Net Votes | Boost Multiplier |
|-----------|-----------------|
| 0         | 1.00            |
| 1         | 1.35            |
| 5         | 1.80            |
| 10        | 1.98            |
| 20        | 2.25            |
| 50        | 2.45            |
| 100       | 2.75            |
| 500       | 3.46            |

The first few votes have the biggest impact. Going from 0 to 5 votes increases the boost by 80%. Going from 100 to 105 votes increases it by less than 2%. This is the "early votes matter most" philosophy—it ensures that new suggestions can quickly rise if the community likes them, without allowing popular suggestions to become unstoppable.

### Voting UI in the Frontend

The voting interface is simple and immediate. Each suggestion card has an upvote button (▲) and a downvote button (▼), with the current net score displayed between them.

The design follows a common pattern seen in Reddit, Hacker News, and Stack Overflow:

```
┌─────────────────────────────────────────────┐
│  ▲                                          │
│  7     "Same author, similar pacing!"       │
│  ▼     — suggested for "Midnight Sun"       │
│                                              │
│  ▲                                          │
│  2     "If you liked the worldbuilding..."  │
│  ▼     — suggested for "Midnight Sun"       │
│                                              │
│  ▲                                          │
│ -1     "Better worldbuilding IMO"           │
│  ▼     — suggested for "Midnight Sun"       │
└─────────────────────────────────────────────┘
```

The vote buttons are large enough to tap on mobile (important—many users browse fanfiction on their phones). The score is prominently displayed between the buttons, using green for positive scores, red for negative, and gray for zero.

The API endpoint:

```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(json!({
            "err": -1,
            "msg": "vote must be 1 (upvote) or -1 (downvote)"
        })));
    }

    let new_score = cast_vote(
        &state.db,
        body.suggestion_id,
        "0.0.0.0",
        body.vote as i16,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "new_score": new_score,
    })))
}
```

The `VoteBody` struct:

```rust
pub struct VoteBody {
    pub suggestion_id: i64,
    pub vote: i32,
}
```

The validation at the top (`vote != 1 && vote != -1`) catches invalid values before they reach the database. This is a defense-in-depth measure—the database also has a `CHECK` constraint, but early rejection saves a round trip and provides a clearer error message.

The response includes `new_score`—the updated net vote count. This is essential for optimistic UI updates. When the frontend sends a vote request, it can immediately update the displayed score without waiting for the server. If the server returns a different score than expected (rare, but possible if someone else voted simultaneously), the frontend corrects to the server's value.

### Loading Suggestions: The votes_handler

The frontend loads suggestions and their current votes when the user views a story's recommendations:

```rust
pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VotesQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id is required"
        })));
    }

    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "suggestions": suggestions,
    })))
}
```

The `GET /api/v0/recommendations/votes` endpoint returns all suggestions and their net scores for a given story. The frontend uses this to populate the community suggestions section, showing suggestions sorted by votes. This is typically called once when the page loads, not on every vote—the frontend manages vote state locally after the initial load.

### Optimistic Voting: Immediate UI Feedback

When a user clicks the upvote button, the frontend doesn't wait for the server response. It immediately:

1. Increments the displayed score by 1.
2. Highlights the upvote button to show the user's choice.
3. Sends the vote request to the server in the background.

If the server returns an error (network failure, rate limit, database error), the frontend rolls back—decrementing the score and un-highlighting the button. This is **optimistic UI**—assuming success and handling failure as an edge case.

Why optimistic? Because voting is a low-stakes action. If the vote fails 1 in 100 times, it's better to show immediate feedback and occasionally roll back than to make every vote feel sluggish. The user sees instant results, and the rare failure is handled gracefully.

The flow:

```
User clicks ▲
  → Frontend: score += 1, highlight ▲
  → Background: POST /api/v0/recommendations/vote {suggestion_id: 42, vote: 1}
  → Server: stores vote, returns new_score = 7
  → Frontend: score = 7 (matches? great. no adjustment needed.)
  
  OR (failure path)
  
  → Server: returns error
  → Frontend: score -= 1, un-highlight ▲
  → Show subtle error toast: "Vote couldn't be recorded"
```

The key insight is that the frontend doesn't need the server's response to update the UI—it can calculate the new score locally. The server's response is just a confirmation. If the confirmation says something different (rare, but possible if someone else voted simultaneously), the frontend updates to match.

Here's how this looks in practice. The frontend maintains a local state for each suggestion:

```javascript
// Pseudocode for the voting logic
function handleVote(suggestionId, direction) {
    const suggestion = suggestions.find(s => s.id === suggestionId);
    const previousScore = suggestion.net_votes;
    
    // Optimistic update
    suggestion.net_votes += direction;
    suggestion.userVote = direction;
    
    // Send to server
    fetch('/api/v0/recommendations/vote', {
        method: 'POST',
        body: JSON.stringify({
            suggestion_id: suggestionId,
            vote: direction
        })
    })
    .then(res => res.json())
    .then(data => {
        // Reconcile with server state
        suggestion.net_votes = data.new_score;
    })
    .catch(err => {
        // Rollback on error
        suggestion.net_votes = previousScore;
        suggestion.userVote = 0;
        showToast("Vote couldn't be recorded. Please try again.");
    });
}
```

The reconciliation step is important. Even if the optimistic update is correct, the server's response is the source of truth. If another user voted between our optimistic update and the server response, the scores might not match. The frontend always adopts the server's value.

⚠️ **Watch Out**: Optimistic UI requires careful state management. If the user clicks upvote, then immediately clicks downvote before the first request completes, you could end up with race conditions. The frontend needs to cancel or queue the previous request. A simple approach is to track the pending vote and ignore stale responses. Another approach is to debounce the vote requests—only send the most recent vote to the server.

### The netScore Calculation

The net score is the backbone of the voting system. Let's trace how it flows through the system:

1. **Storage**: Each vote is stored as +1 or -1 in `recommendation_votes`.
2. **Aggregation**: `SUM(vote)` across all votes for a suggestion gives the net score.
3. **Display**: The suggestions list shows `net_votes` sorted descending.
4. **Engine integration**: `get_community_votes` aggregates votes for all candidates and feeds them into the boost formula.

The net score serves double duty: it's a social signal (this suggestion is well-liked) and an engine signal (this recommendation gets a boost). The same number drives both the UI and the algorithm.

```rust
pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VotesQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id is required"
        })));
    }

    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "suggestions": suggestions,
    })))
}
```

The `GET /api/v0/recommendations/votes` endpoint returns all suggestions and their net scores for a given story. The frontend uses this to populate the community suggestions section, showing suggestions sorted by votes.

### The Suggestion Struct in the API

When a user requests suggestions for a story, the API returns JSON like this:

```json
{
    "err": 0,
    "url_id": "abc123",
    "suggestions": [
        {
            "id": 42,
            "suggested_url_id": "def456",
            "comment": "Same author, similar pacing!",
            "net_votes": 7,
            "created": "2024-01-15T10:30:00Z"
        },
        {
            "id": 43,
            "suggested_url_id": "ghi789",
            "comment": null,
            "net_votes": 2,
            "created": "2024-01-16T14:20:00Z"
        },
        {
            "id": 44,
            "suggested_url_id": "jkl012",
            "comment": "Better worldbuilding IMO",
            "net_votes": -1,
            "created": "2024-01-17T09:00:00Z"
        }
    ]
}
```

The suggestions are sorted by `net_votes DESC, created DESC`—most upvoted first, newest first as tiebreaker. This means the best community recommendations appear at the top. A suggestion with -1 votes appears at the bottom—visible but clearly not favored by the community.

### How Voting Feeds the Engine

Let's trace the complete flow from a user voting to that vote affecting recommendations:

1. User upvotes suggestion #42 (linking Story A to Story B).
2. `cast_vote` stores `+1` in `recommendation_votes` for suggestion #42.
3. The net vote count for Story B increases by 1.
4. Next time someone requests recommendations for any story, and Story B is a candidate, `get_community_votes` picks up the new net score.
5. The boost formula applies: `score *= (1.0 + gamma × ln_1p(max(0, net_votes)))`.
6. Story B's final score is higher, so it ranks higher in the results.

This creates a virtuous cycle: good suggestions get upvoted, which boosts their ranking, which makes them more visible, which leads to more upvotes. Meanwhile, bad suggestions get downvoted and sink to the bottom.

The key insight is that votes influence *all* recommendations for a story, not just the specific suggestion. If Story B is upvoted as a recommendation for Story A, and Story B is also a collaborative filtering candidate for Story C, the vote boost applies to both recommendations. The voting system and the collaborative filtering system reinforce each other.

🧪 **Try It Yourself**: Trace through the math. A story has a collaborative score of 0.3. It has 8 upvotes and 2 downvotes (net = 6). With `gamma = 0.5`, the boost is `1.0 + 0.5 × ln(1 + 6) = 1.0 + 0.5 × 1.95 = 1.97`. The final score is `0.3 × 1.97 = 0.59`. That's nearly double—votes can significantly shift a recommendation's ranking.

### Putting It All Together

The recommendation system in FicHub is a blend of automated intelligence and human curation:

- **Collaborative filtering** discovers patterns from real reading behavior—what the crowd's bookmarks tell us about taste.
- **Tag fallback** handles new stories that don't have enough data yet—matching by author and title keywords.
- **Community suggestions** let readers contribute their expertise—their "you'll love this" moments.
- **Voting** surfaces the best suggestions and feeds signals back into the engine.

No single signal dominates. The blended approach means recommendations are robust: even if collaborative filtering is weak (cold start), tag matching and community votes can carry the day. Even if the community hasn't voted on a recommendation yet, collaborative filtering provides a solid baseline. Even if there's no collaborative data at all, community suggestions provide immediate value.

The result is a recommendation engine that feels alive—it learns from the community, surfaces human judgment, and adapts as new stories appear and reader tastes evolve. It's not just an algorithm; it's a conversation between readers, mediated by data.

The beauty of this architecture is its extensibility. The `SiteFetcher` trait makes adding new sites trivial—implement two methods and you're done. The Redis queue makes scaling the worker horizontal—just run more instances. The voting system makes community curation self-sustaining—no moderator intervention needed. And the recommendation engine itself—the heart of it all—is just SQL queries and a few blending formulas. No magic. Just math, data, and a community that cares about good stories.

### The API Surface: Connecting Everything

The recommendation system exposes three API endpoints that tie all these components together:

1. **`GET /api/v0/recommendations`** — The main endpoint. Accepts a `url_id` or URL, optionally a `site_domain` filter, and a count `n`. Returns up to `n` recommendations with scores and metadata. This endpoint calls `get_recommendations` on the engine, which handles cache lookup and live computation.

2. **`POST /api/v0/recommendations/suggest`** — Submit a community suggestion. Accepts a seed `url_id`, a suggested URL, and an optional comment. Resolves the URL, validates both stories exist, and stores the suggestion. Returns the suggestion ID.

3. **`POST /api/v0/recommendations/vote`** — Cast a vote on a suggestion. Accepts a `suggestion_id` and a `vote` value (1 or -1). Returns the new net score.

4. **`GET /api/v0/recommendations/votes`** — List all suggestions and their votes for a given story. Returns suggestions sorted by net votes.

These four endpoints cover the entire recommendation workflow: request recommendations, suggest new ones, vote on suggestions, and view community feedback. The frontend consumes all four, creating a rich, interactive recommendation experience.

The `AppState` struct holds references to all the components:

```rust
pub struct AppState {
    pub db: PgPool,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
    pub config: Config,
    pub http_client: Client,
    pub scraper_registry: Arc<ScraperRegistry>,
}
```

Each request handler accesses the shared state via Axum's `State` extractor. The `recommender_engine` handles recommendation computation, the `collection_worker` handles background data collection, and the `config` provides tuning parameters. This clean separation of concerns makes each component independently testable and replaceable.

---

## Summary

Part 5 built FicHub's recommendation engine from the ground up—a system that turns passive bookmark data into genuinely useful recommendations.

- **Chapter 21** explained collaborative filtering: the Jaccard coefficient measures similarity between stories based on shared bookmarkers. The co-occurrence table tracks which stories appear together, and the SQL-based Jaccard query efficiently ranks candidates. The engine blends collaborative scores with tag-based fallback for new stories (matching by author and title keywords when bookmark data is sparse), and applies a logarithmic voting boost for community favorites. The precomputed cache ensures fast response times for popular stories, while live computation handles cache misses.

- **Chapter 22** built the collection worker: a background process that scrapes bookmark data from fanfiction sites using per-site `SiteFetcher` implementations. The worker uses Redis queues for work distribution, per-site rate limiters (atomic CAS-based) to stay polite, and SHA-256 hashing to protect user privacy. The co-occurrence update uses a nested loop to increment pairwise counts for each user's bookmark set.

- **Chapter 23** added community suggestions: users can manually recommend stories by pasting a URL and adding a comment. The backend resolves URLs, validates both stories exist in the database, and stores the suggestion with IP-based deduplication. Missing stories are automatically queued for collection, creating a seamless bridge between community input and data gathering.

- **Chapter 24** implemented voting and scoring: a binary upvote/downvote system with optimistic UI feedback (instant score updates, server reconciliation). The logarithmic boost formula gives early votes outsized impact while preventing popular suggestions from becoming unstoppable. Votes feed directly into the recommendation engine's scoring pipeline, creating a virtuous cycle where good suggestions rise and bad ones sink.

The recommendation engine is a closed loop: collect data → compute similarity → surface recommendations → gather feedback → improve rankings. Each component feeds the others, creating a system that gets smarter as the community grows. The architecture is deliberately simple—SQL queries and a few blending formulas—but that simplicity is its strength. It's debuggable, fast, and extensible. New sites can be added by implementing a trait. New signals can be blended by adjusting weights. The community can contribute through suggestions and votes. And the whole thing runs on the same PostgreSQL server that powers the rest of FicHub.

---

## The Last 500 Words

The recommendation engine in FicHub represents a philosophy: that the best recommendations come from the community, not from algorithms alone. Collaborative filtering is powerful—it can discover connections that no human curator would spot—but it's fundamentally passive. It observes. It doesn't understand *why* a story is good, only that certain readers tend to bookmark certain combinations.

Community suggestions add the "why." When a reader says "you'll love this because it has the same slow-burn romance" or "this has the same world-building style," they're contributing semantic understanding that no bookmark-based algorithm can extract. And voting amplifies the signal: good suggestions rise, bad ones sink.

The collection worker is the unglamorous foundation. Without it, the engine would starve. Without data flowing in from AO3, FanFiction.net, SpaceBattles, and the other sites, there's nothing to compute. The worker's polite rate limiting, privacy-preserving hashing, and careful co-occurrence tracking ensure that data collection is sustainable and ethical.

The scoring formula—blending collaborative, tag-based, and community signals with a logarithmic voting boost—is deliberately simple. We could add machine learning models, neural collaborative filtering, or content-based NLP analysis. But simplicity has value: it's debuggable, it's fast, and it works. A reader shouldn't have to wait 2 seconds for recommendations. They should click and see results instantly.

As the system matures, the co-occurrence table grows denser, Jaccard coefficients become more meaningful, and the community's collective wisdom surfaces through votes. The engine doesn't need to be perfect—it just needs to be useful. And every time a reader discovers a new favorite through a recommendation, the system has done its job.

The beauty of this architecture is its extensibility. The `SiteFetcher` trait makes adding new sites trivial. The Redis queue makes scaling the worker horizontal. The voting system makes community curation self-sustaining. And the recommendation engine itself—the heart of it all—is just SQL queries and a few blending formulas. No magic. Just math, data, and a community that cares about good stories.

Looking ahead, the natural next step would be to add content-based features—using the story's summary text, tags, and metadata to compute similarity independent of bookmark data. Techniques like TF-IDF or even simple word embeddings could complement collaborative filtering, especially for brand-new stories that haven't accumulated any bookmarks. But that's a future chapter. For now, the collaborative + tag fallback + community voting blend serves FicHub's readers well. It's not perfect, but it's genuinely useful—and in a system built by and for fanfiction readers, that's exactly what matters.

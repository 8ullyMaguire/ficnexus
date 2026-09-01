# Part 31 — Roadmap Consensus Engine

> In this chapter you will learn how FicHub crowdsources product feedback using a MaxDiff voting arena powered by Ollama embeddings and Elo ratings. We'll build the semantic clustering system (nomic-embed-text), the MaxDiff-to-Elo vote translator, and the consensus leaderboard.

---

## Overview

FicHub's Roadmap page (`/roadmap`) lets users shape the platform's future. The system works in three stages:

1. **Suggest** — users submit feature ideas via `POST /api/roadmap/suggest`. The text is embedded via Ollama's `nomic-embed-text` model (768-dimensional vectors), then clustered: if the suggestion is similar (cosine distance < 0.22) to an existing open cluster, it joins that cluster; otherwise, a new cluster is created.
2. **Arena** — `GET /api/roadmap/arena` serves 4 clusters (selected to favor the least-voted-on). Users pick the **best** and **worst** in a MaxDiff vote.
3. **Vote → Elo** — the MaxDiff choice is translated into 6 virtual 1v1 Elo matches (best beats all three, worst loses to all three, neutrals beat worst + lose to best + draw each other). Ratings update on a standard Elo K=32 scale.
4. **Consensus** — `GET /api/roadmap/consensus` returns the public leaderboard (sorted by Elo) and a "most controversial" view.

**Zero-PII**: anonymous voters are keyed by a client-side UUID (`x-client-id` header), never by IP address.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database, getting a single work, searching, user accounts).
- You have completed Part 8 (User Accounts) — understand JWT auth and the `AuthUser` extractor.
- Ollama is running locally on `http://localhost:11434` with the `nomic-embed-text` model.
- The `feature_clusters`, `feature_suggestions`, and `arena_votes` tables exist in the database.

---

## Chapter 31.1 — The Ollama Embedding Client

### Goal

Understand how FicHub wraps Ollama's HTTP API to get text embeddings, with graceful degradation when Ollama is down.

### Actions

#### 1. The client struct

```rust
// src/services/ollama.rs (lines 37–42)
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,   // e.g. "http://localhost:11434"
    model: String,      // e.g. "nomic-embed-text"
    http: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String, http: reqwest::Client) -> Self {
        Self { base_url, model, http }
    }
}
```

#### 2. The `embed` method

```rust
/// Embed text via Ollama's /api/embeddings endpoint.
/// Returns a 768-d vector (nomic-embed-text). Errors are non-fatal.
pub async fn embed(&self, text: &str) -> Result<Vec<f32>, OllamaError> {
    let url = format!("{}/api/embeddings", self.base_url.trim_end_matches('/'));
    let resp = self
        .http
        .post(&url)
        .json(&json!({ "model": self.model, "prompt": text }))
        .send()
        .await
        .map_err(|e| OllamaError(format!("request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(OllamaError(format!("HTTP {}", resp.status())));
    }

    let body: EmbeddingResponse = resp.json().await
        .map_err(|e| OllamaError(format!("bad response: {e}")))?;

    if body.embedding.is_empty() {
        return Err(OllamaError("empty embedding".into()));
    }
    Ok(body.embedding)
}
```

> **⚠️ Watch Out**: If Ollama is down, `embed()` returns an `Err(OllamaError)`. The caller (suggest_handler) catches this with `.map_err(|e| tracing::warn!(...)).ok()` — the suggestion is still saved to the database, just without clustering. This is the "degrade gracefully" pattern.

#### 3. The `generate` and `generate_json` methods

The same client also wraps `/api/generate` for text completion — used by Ask the Archive (Part 17) for natural-language-to-search-filter conversion:

```rust
/// Generate text via Ollama's /api/generate. Best-effort, non-fatal.
pub async fn generate(&self, prompt: &str, chat_model: &str) -> Result<String, OllamaError> {
    // ... POST /api/generate with {"model": chat_model, "prompt": prompt, "stream": false}
    // Strips <thinking> blocks from thinking-model outputs
}

/// Same but with "format": "json" for constrained JSON output.
pub async fn generate_json(&self, prompt: &str, chat_model: &str) -> Result<String, OllamaError> {
    // ... same as generate but adds "format": "json" to the request body
}
```

#### 4. Tests

The test suite uses mock TCP servers to test error handling:

```rust
#[tokio::test]
async fn embed_error_when_ollama_down() {
    let client = OllamaClient::new("http://127.0.0.1:1".into(), ...);
    let err = client.embed("test").await.expect_err("should fail");
    assert!(err.to_string().starts_with("ollama:"));
}

#[tokio::test]
async fn generate_parses_response_field() {
    // Mock server returns {"model":"llama3.1:8b","response":"toxic | slur | 0.9","done":true}
    let out = client.generate("classify", "llama3.1:8b").await.expect("should succeed");
    assert_eq!(out, "toxic | slur | 0.9");
}
```

### Try It Yourself

```bash
# Run the Ollama tests
cargo test --lib ollama -- --nocapture

# Start Ollama locally and pull the embedding model
ollama pull nomic-embed-text
ollama serve  # on localhost:11434
```

### Check

- ✅ `OllamaClient::embed()` returns a `Vec<f32>` of 768 elements (nomic-embed-text).
- ✅ When Ollama is unreachable, `embed()` returns `Err(OllamaError)`.
- ✅ `generate_json()` adds `"format": "json"` to the request body.
- ✅ The thinking-block stripper handles `thinking`…`>` tags from reasoning models.

### What you built

A thin HTTP wrapper around Ollama's embeddings and text-generation APIs, with non-fatal error handling and unit tests using mock TCP servers.

---

## Chapter 31.2 — Semantic Clustering

### Goal

Build the `suggest_handler` endpoint that embeds feature text, finds the nearest existing cluster, and either joins it or creates a new one.

### Actions

#### 1. The database schema

```sql
-- feature_clusters: one row per unique idea (semantic group)
CREATE TABLE feature_clusters (
    id                  SERIAL PRIMARY KEY,
    representative_text TEXT NOT NULL,      -- the first/clearest suggestion
    embedding           vector(768),        -- nomic-embed-text embedding
    elo_rating          REAL DEFAULT 1500.0, -- starts at 1500 (standard Elo)
    matches_played      INTEGER DEFAULT 0,  -- how many arena votes involved this cluster
    times_picked_best   INTEGER DEFAULT 0,  -- MaxDiff "best" count
    times_picked_worst  INTEGER DEFAULT 0,  -- MaxDiff "worst" count
    status              TEXT DEFAULT 'open', -- open | shipped | rejected
    created_at          TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_feature_clusters_embedding ON feature_clusters USING hnsw (embedding);
CREATE INDEX idx_feature_clusters_status ON feature_clusters(status);
ALTER OWNER TO fichub;

-- feature_suggestions: raw submissions (cluster_id nullable)
CREATE TABLE feature_suggestions (
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER REFERENCES users(id) ON DELETE SET NULL,
    client_id  TEXT,                       -- anonymous UUID
    raw_text   TEXT NOT NULL,
    cluster_id INTEGER REFERENCES feature_clusters(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_feature_suggestions_client ON feature_suggestions(client_id);
ALTER OWNER TO fichub;
```

#### 2. The suggest handler

`POST /api/roadmap/suggest`:

```rust
use crate::ollama::OllamaClient;

const CLUSTER_DISTANCE_THRESHOLD: f64 = 0.22;
const MAX_SUGGESTIONS_PER_DAY: i64 = 3;

pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<SuggestRequest>,
) -> Result<Json<Value>, AppError> {
    let text = req.text.trim().to_string();
    if text.is_empty() || text.len() > 1000 {
        return Err(AppError::BadRequest("suggestion text required (max 1000 chars)".into()));
    }

    // Anti-spam: max 3 suggestions/day per identity
    let since = chrono::Utc::now() - chrono::Duration::days(1);
    let client_id = headers.get("x-client-id")
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feature_suggestions \
         WHERE (user_id = $1 OR client_id = $2) AND created_at >= $3"
    ).bind(user_id_or(&user)).bind(&client_id).bind(since)
     .fetch_one(&state.db).await?;

    if recent >= MAX_SUGGESTIONS_PER_DAY {
        return Err(AppError::BadRequest("too many suggestions today (max 3)".into()));
    }

    // Embed via Ollama — degrade gracefully if it's down
    let embedding: Option<Vec<f32>> = state.ollama.embed(&text).await
        .map_err(|e| tracing::warn!("ollama embed failed: {e}"))
        .ok();

    let cluster_id: i32 = if let Some(emb) = embedding {
        let emb_sql = format!("[{}]", emb.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>().join(","));

        // Find nearest existing open cluster using PostgreSQL's <=> operator
        let nearest: Option<(i32, f64)> = sqlx::query_as(
            "SELECT id, (embedding <=> $1::vector) AS dist \
             FROM feature_clusters WHERE status = 'open' \
             ORDER BY embedding <=> $1::vector LIMIT 1"
        ).bind(&emb_sql).fetch_optional(&state.db).await?;

        match nearest {
            Some((cid, dist)) if dist < CLUSTER_DISTANCE_THRESHOLD => cid,  // join cluster
            _ => {
                // Spawn new cluster
                sqlx::query(
                    "INSERT INTO feature_clusters (representative_text, embedding) \
                     VALUES ($1, $2::vector) RETURNING id"
                ).bind(&text).bind(&emb_sql).fetch_one(&state.db).await?
                 .get::<i32, _>(0)
            }
        }
    } else {
        // Ollama down — save raw suggestion without clustering
        0
    };

    sqlx::query(
        "INSERT INTO feature_suggestions (user_id, client_id, raw_text, cluster_id) \
         VALUES ($1, $2, $3, $4)"
    ).bind(user_id_or(&user)).bind(&client_id)
     .bind(&text).bind(if cluster_id > 0 { Some(cluster_id) } else { None })
     .execute(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "cluster_id": if cluster_id > 0 { Some(cluster_id) } else { None },
        "clustered": cluster_id > 0,
    })))
}

fn user_id_or(user: &AuthUser) -> Option<i32> { user.user_id }

#[derive(Debug, Deserialize)]
pub struct SuggestRequest { pub text: String }
```

> **💡 Key Concept**: The `<=>` operator is PostgreSQL's vector distance operator (from the `pgvector` extension). When `dist < 0.22`, the suggestion joins the existing cluster (its representative text is unchanged — only the suggestion is linked). When no cluster is close enough, a new cluster is spawned.

#### 3. Cosine distance threshold

The threshold of 0.22 is calibrated for `nomic-embed-text`:
- Below 0.15: too strict, creates duplicate clusters for paraphrases.
- 0.15–0.22: the sweet spot — groups "dark mode" and "dark theme" but separates "dark mode" from "change font to black".
- Above 0.25: too loose, merges unrelated ideas.

### Try It Yourself

```bash
# Submit a feature suggestion (anonymous)
curl -X POST "http://localhost:8000/api/roadmap/suggest" \
  -H "x-client-id: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{"text": "Add a dark mode toggle"}' | jq

# Submit a paraphrase (should join the same cluster)
curl -X POST "http://localhost:8000/api/roadmap/suggest" \
  -H "x-client-id: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{"text": "I want a dark theme option"}' | jq
```

### Check

- ✅ `POST /api/roadmap/suggest` embeds the text and clusters it.
- ✅ Paraphrased suggestions join the same cluster (distance < 0.22).
- ✅ Anti-spam limits to 3 suggestions per day per client_id/user_id.
- ✅ When Ollama is down, suggestions are saved without clustering (`cluster_id: null`).

### What you built

The semantic clustering system — feature suggestions are embedded via Ollama, compared against existing clusters using PostgreSQL's pgvector `<=>` operator, and grouped by semantic similarity.

---

## Chapter 31.3 — MaxDiff Voting and Elo Translation

### Goal

Build the `arena_handler` and `vote_handler` — the MaxDiff voting system that powers the Roadmap Arena.

### Actions

#### 1. Elo math (pure, unit-tested)

```rust
const ELO_K: f64 = 32.0;

/// Expected score of rating_a against rating_b (logistic, standard Elo).
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// New rating after a match: rating + K * (score - expected).
/// score: 1.0 (win), 0.5 (draw), 0.0 (loss).
pub fn elo_update(rating: f64, opponent_rating: f64, score: f64) -> f64 {
    let expected = expected_score(rating, opponent_rating);
    rating + ELO_K * (score - expected)
}

/// Translate a MaxDiff choice (best, worst, 2 neutrals) into 6 virtual 1v1 Elo updates.
pub fn maxdiff_elo(
    ids: &[i32],          // 4 cluster IDs in the arena
    ratings: &[(i32, f64)], // current (id, rating) pairs
    best: i32,            // best-rated cluster
    worst: i32,           // worst-rated cluster
) -> Vec<(i32, f64)> {
    let rating_of = |id: i32| {
        ratings.iter().find(|(i, _)| *i == id).map(|(_, r)| *r).unwrap_or(1500.0)
    };
    let mut new_ratings: Vec<(i32, f64)> = ratings.to_vec();

    let mut update = |id: i32, opponent: i32, score: f64| {
        let idx = new_ratings.iter().position(|(i, _)| *i == id).expect("id present");
        new_ratings[idx].1 = elo_update(new_ratings[idx].1, rating_of(opponent), score);
    };

    for &id in ids {
        if id == best {
            // Best: wins vs the other three
            for &opp in ids { if opp != best { update(id, opp, 1.0); } }
        } else if id == worst {
            // Worst: loses vs the other three
            for &opp in ids { if opp != worst { update(id, opp, 0.0); } }
        } else {
            // Neutral: beats worst, loses to best, draws the other neutral
            update(id, worst, 1.0);   // neutral > worst
            update(id, best, 0.0);     // neutral < best
            for &opp in ids {
                if opp != best && opp != worst && opp != id {
                    update(id, opp, 0.5);  // neutral ↔ neutral (draw)
                }
            }
        }
    }
    new_ratings
}
```

#### 2. Unit tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_score_midpoint_is_half() {
        assert!((expected_score(1500.0, 1500.0) - 0.5).abs() < 1e-9);
        assert!(expected_score(1600.0, 1400.0) > 0.75);
    }

    #[test]
    fn elo_update_win_raises_loser_lowers() {
        let winner_new = elo_update(1500.0, 1500.0, 1.0);
        let loser_new = elo_update(1500.0, 1500.0, 0.0);
        assert!(winner_new > 1500.0);
        assert!(loser_new < 1500.0);
        // Symmetry at equal ratings: winner gains exactly what loser loses
        assert!((winner_new - 1500.0 - (1500.0 - loser_new)).abs() < 1e-9);
    }

    #[test]
    fn maxdiff_best_soars_worst_tanks_neutrals_shift_little() {
        let ids = vec![1, 2, 3, 4];
        let ratings = vec![(1, 1500.0), (2, 1500.0), (3, 1500.0), (4, 1500.0)];
        let out = maxdiff_elo(&ids, &ratings, 1, 4);

        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        let r4 = out.iter().find(|(i, _)| *i == 4).unwrap().1;
        let r2 = out.iter().find(|(i, _)| *i == 2).unwrap().1;
        let r3 = out.iter().find(|(i, _)| *i == 3).unwrap().1;

        assert!(r1 > 1540.0, "best gains big: {r1}");      // ~+48 (3 wins at K=32)
        assert!(r4 < 1460.0, "worst loses big: {r4}");      // ~-48 (3 losses)
        assert!(r2 > r4 && r2 < r1, "neutral 2 between: {r2}");
        assert!(r3 > r4 && r3 < r1, "neutral 3 between: {r3}");
    }

    #[test]
    fn maxdiff_rating_gap_reduces_transfer() {
        // A 1800-rated best vs a 1200-rated worst: less Elo transfer than an even match
        let ids = vec![1, 2, 3, 4];
        let ratings = vec![(1, 1800.0), (2, 1500.0), (3, 1500.0), (4, 1200.0)];
        let out = maxdiff_elo(&ids, &ratings, 1, 4);
        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        assert!(r1 > 1800.0, "still gains");
        assert!(r1 - 1800.0 < 32.0 * 3.0, "gain bounded by K × matches");
    }
}
```

#### 3. The arena handler

`GET /api/roadmap/arena` — serves 4 clusters, favoring the least-played:

```rust
pub async fn arena_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    // Select 4 open clusters, ordered by least-played first (with random tiebreak).
    // This ensures fresh clusters get seen and their baseline Elo gets established.
    let rows: Vec<(i32, String, i32, f64)> = sqlx::query_as(
        r#"SELECT id, representative_text, matches_played, elo_rating::float8
           FROM feature_clusters
           WHERE status = 'open'
           ORDER BY matches_played ASC, random()
           LIMIT 4"#
    ).fetch_all(&state.db).await?;

    if rows.is_empty() {
        return Ok(Json(json!({
            "err": 0, "clusters": [],
            "message": "No features yet — be the first to suggest one!"
        })));
    }

    let clusters = rows.into_iter().map(|(id, text, played, elo)| json!({
        "id": id, "text": text,
        "matches_played": played, "elo_rating": elo,
    })).collect::<Vec<_>>();

    Ok(Json(json!({
        "err": 0, "clusters": clusters,
        "user_id": user_id_or(&user).unwrap_or(0),
    })))
}
```

#### 4. The vote handler

`POST /api/roadmap/vote` — validates the vote, translates it to Elo, and persists atomically:

```rust
#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub cluster_ids: Vec<i32>,   // exactly 4
    pub best_cluster_id: i32,
    pub worst_cluster_id: i32,
}

pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<VoteRequest>,
) -> Result<Json<Value>, AppError> {
    // Validate the arena shape
    if req.cluster_ids.len() != 4 {
        return Err(AppError::BadRequest("arena must present exactly 4 clusters".into()));
    }
    if !req.cluster_ids.contains(&req.best_cluster_id) ||
       !req.cluster_ids.contains(&req.worst_cluster_id) {
        return Err(AppError::BadRequest("best/worst must be among the presented clusters".into()));
    }
    if req.best_cluster_id == req.worst_cluster_id {
        return Err(AppError::BadRequest("best and worst must differ".into()));
    }

    let client_id = headers.get("x-client-id")
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    // Uniqueness: no double-vote on the same set
    let uid = user_id_or(&user);
    let sorted_ids = { let mut s = req.cluster_ids.clone(); s.sort_unstable(); s };
    let exists: bool = if uid.is_some() {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE user_id = $1 AND cluster_ids = $2)"
        ).bind(uid).bind(&sorted_ids).fetch_one(&state.db).await?
    } else {
        match &client_id {
            Some(cid) => sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE client_id = $1 AND cluster_ids = $2 AND user_id IS NULL)"
            ).bind(cid).bind(&sorted_ids).fetch_one(&state.db).await?,
            None => false,
        }
    };
    if exists {
        return Err(AppError::BadRequest("you already voted on this set".into()));
    }

    // Load current ratings, compute new ones via MaxDiff→Elo
    let mut ratings: Vec<(i32, f64)> = Vec::new();
    for id in &req.cluster_ids {
        let rating: Option<f64> = sqlx::query_scalar(
            "SELECT elo_rating::float8 FROM feature_clusters WHERE id = $1"
        ).bind(id).fetch_optional(&state.db).await?;
        ratings.push((*id, rating.unwrap_or(1500.0)));
    }
    let new_ratings = maxdiff_elo(&req.cluster_ids, &ratings,
        req.best_cluster_id, req.worst_cluster_id);

    // Persist atomically: vote record + Elo updates + counter bumps
    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO arena_votes (user_id, client_id, cluster_ids, best_cluster_id, worst_cluster_id) \
         VALUES ($1, $2, $3, $4, $5)"
    ).bind(uid).bind(&client_id).bind(&sorted_ids)
     .bind(req.best_cluster_id).bind(req.worst_cluster_id)
     .execute(&mut *tx).await?;

    for (id, new_rating) in &new_ratings {
        sqlx::query(
            r#"UPDATE feature_clusters SET
                 elo_rating = $1,
                 matches_played = matches_played + 1,
                 times_picked_best = times_picked_best + CASE WHEN $2 THEN 1 ELSE 0 END,
                 times_picked_worst = times_picked_worst + CASE WHEN $3 THEN 1 ELSE 0 END
               WHERE id = $4"#
        ).bind(new_rating)
         .bind(*id == req.best_cluster_id)
         .bind(*id == req.worst_cluster_id)
         .bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await?;

    Ok(Json(json!({
        "err": 0,
        "applied": new_ratings.iter().map(|(id, r)| json!({
            "cluster_id": id, "new_elo": r
        })).collect::<Vec<_>>(),
    })))
}
```

### Try It Yourself

```bash
# Run the Elo math unit tests
cargo test --lib roadmap -- --nocapture

# Expected output:
#   test expected_score_midpoint_is_half ... ok
#   test elo_update_win_raises_loser_lowers ... ok
#   test maxdiff_best_soars_worst_tanks_neutrals_shift_little ... ok
#   test maxdiff_rating_gap_reduces_transfer ... ok
```

### Check

- ✅ `maxdiff_elo` produces 3 wins for best, 3 losses for worst, mixed results for neutrals.
- ✅ When all clusters start at 1500 Elo, best gains ~48 points (3 × 16) and worst loses ~48.
- ✅ Against a 300-point rating gap, the transfer is bounded (< K × 3 = 96 points).
- ✅ The vote handler validates: 4 clusters, best ≠ worst, both among the 4.
- ✅ Double-votes are prevented via the `cluster_ids` uniqueness check.

### What you built

The MaxDiff-to-Elo voting engine — pure functions for Elo math and MaxDiff translation, a random-fair arena selector (least-played first), and a transactional vote handler with anti-double-vote protection.

---

## Chapter 31.4 — Consensus Leaderboard

### Goal

Build the public `GET /api/roadmap/consensus` endpoint that returns the leaderboard (sorted by Elo) and a "most controversial" view.

### Actions

#### 1. The consensus handler

```rust
pub async fn consensus_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    // Leaderboard: ranked by Elo DESC, with vote counters + suggestion count.
    let leaderboard = sqlx::query_as::<_, (i32, String, f64, i32, i32, i32, i64, String)>(
        r#"SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
                  c.times_picked_best, c.times_picked_worst,
                  COUNT(s.id) AS suggestions, c.status
           FROM feature_clusters c
           LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
           GROUP BY c.id
           ORDER BY c.elo_rating DESC
           LIMIT 100"#
    ).fetch_all(&state.db).await?;

    // Controversy: volume (matches) vs disagreement (best + worst picks).
    // A feature voted "best" AND "worst" often is the most polarizing.
    let controversy = sqlx::query_as::<_, (i32, String, i32, i32, i32, i32, f64)>(
        r#"SELECT id, representative_text, matches_played,
                  times_picked_best, times_picked_worst,
                  (times_picked_best + times_picked_worst) AS controversy,
                  elo_rating::float8
           FROM feature_clusters
           WHERE status = 'open'
           ORDER BY matches_played DESC
           LIMIT 100"#
    ).fetch_all(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "leaderboard": leaderboard.into_iter().map(|(id, text, elo, played, best, worst, sugg, status)| json!({
            "id": id, "text": text, "elo_rating": elo, "matches_played": played,
            "times_picked_best": best, "times_picked_worst": worst,
            "suggestions": sugg, "status": status,
        })).collect::<Vec<_>>(),
        "controversy": controversy.into_iter().map(|(id, text, played, best, worst, contr, elo)| json!({
            "id": id, "text": text, "matches_played": played,
            "times_picked_best": best, "times_picked_worst": worst,
            "controversy": contr, "elo_rating": elo,
        })).collect::<Vec<_>>(),
    })))
}
```

#### 2. The arena vote table

```sql
CREATE TABLE arena_votes (
    id              SERIAL PRIMARY KEY,
    user_id         INTEGER REFERENCES users(id) ON DELETE SET NULL,
    client_id       TEXT,                       -- anonymous UUID
    cluster_ids     INTEGER[],                  -- sorted array of the 4 presented IDs
    best_cluster_id INTEGER NOT NULL,
    worst_cluster_id INTEGER NOT NULL,
    voted_at        TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (user_id, cluster_ids),              -- logged-in users: one vote per set
    UNIQUE (client_id, cluster_ids)             -- anonymous: same rule via client_id
);
ALTER OWNER TO fichub;
```

> **💡 Key Concept**: The `UNIQUE` constraint on `(user_id, cluster_ids)` or `(client_id, cluster_ids)` prevents double-voting. Since anonymous users have `user_id = NULL` (which doesn't collide in PostgreSQL's unique constraint), the backend separately checks `client_id` uniqueness for anonymous votes.

### Try It Yourself

```bash
# Get the public consensus (no auth required)
curl "http://localhost:8000/api/roadmap/consensus" | jq '.leaderboard[0]'

# Check the controversy view
curl "http://localhost:8000/api/roadmap/consensus" | jq '.controversy[0]'
```

### Check

- ✅ `GET /api/roadmap/consensus` requires no authentication (public endpoint).
- ✅ The leaderboard returns clusters sorted by `elo_rating DESC`.
- ✅ The controversy view returns `(times_picked_best + times_picked_worst) AS controversy`.
- ✅ Both views are capped at 100 entries.

### What you built

The consensus leaderboard — a public endpoint that returns the global feature ranking (by Elo) and the most controversial features (by best+worst vote volume).

---

## Conclusion

You now understand FicHub's complete roadmap consensus system:

1. **Semantic clustering** — suggestions are embedded via Ollama's `nomic-embed-text`, then clustered using PostgreSQL's `pgvector` `<=>` operator with a 0.22 cosine distance threshold.
2. **MaxDiff arena** — 4 clusters are served (least-played first for fairness), users pick best + worst.
3. **Elo translation** — the MaxDiff vote is translated into 6 virtual 1v1 Elo matches (K=32), updating all 4 ratings.
4. **Consensus leaderboard** — the public `GET /api/roadmap/consensus` endpoint returns the global ranking by Elo and a controversy view.
5. **Zero-PII** — anonymous votes are keyed by client-side UUID, the 53-char hex JWT, never by IP.
6. **Degrade gracefully** — if Ollama is down, suggestions are saved without clustering (no hard failure).

The entire Elo/MaxDiff math is pure functions — fully unit-tested without a database. The clustering adds a small latency cost (one Ollama round-trip per suggestion), but the voting and consensus queries are pure SQL.

---

## On to the next part

In Part 19 we'll build [Ask the Archive] — how FicHub translates natural-language questions ("fics where Harry is the main character and the mood is dark") into structured search filters using Ollama's `generate_json` endpoint.

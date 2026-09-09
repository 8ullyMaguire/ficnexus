# StoryGraph-style Kanban Roadmap with Elo Consensus

## Goal
Polish FicHub roadmap like StoryGraph's, keep **Elo/MaxDiff consensus** as the ranking engine, extract it into a reusable Rust crate.

## Reference: StoryGraph (Noora) layout
- `/roadmap/` — Give feedback dropdown (Features/Bugs/Requests-Ideas with counts) then Kanban: Up Next, In Progress, Finished [Not Yet Shipped], Shipped, Medium-term, Long-term, Ideas
- `/features` (board) — vote-based consensus, tag filters per status, sort Newest/Popular, search bar
- `/changelog` — blog with posts, filter by New/Improved/Fixed

## What to preserve
- Elo ranking within every column (not simple upvotes)
- Arena only pulls from `idea` pool; moving to `up_next` freezes voting

## Two-crate split

### 1. `fichub-consensus` (new crate, standalone)
Goal: generic Elo/MaxDiff engine usable by any project.

**pub API:**
- ` EloRating::new(start: f64) ` 
- ` EloRating::record_match(winner: &str, loser: &str, k: f64) -> (f64, f64) ` 
- ` EloRating::batch_from_pairs(pairs: Vec<(String, String)>) -> HashMap<String, f64> ` 
- persistence: trait `RatingStore` with `load`, `save`, default `SqlxStore` backend
- serialization: feature id + elo + last_updated

**Cargo.toml:** no FicHub deps, sqlx optional behind feature flag `sqlx-store`.

**Why separate:** other projects can `cargo add fichub-consensus`, plug their own store, reuse arena pairing logic.

### 2. `fichub` (existing)
- uses `fichub_consensus` via Cargo workspace
- arena routes call `Consensus::get_pairs(status='idea')` (already filtered)
- board routes return features sorted by `elo DESC` per column

## Database migration (064_roadmap_kanban.sql)

```sql
ALTER TABLE feature_clusters DROP CONSTRAINT IF EXISTS feature_clusters_status_check;
ALTER TABLE feature_clusters ADD CONSTRAINT feature_clusters_status_check 
  CHECK (status IN ('idea','long_term','medium_term','up_next','in_progress','finished','shipped','rejected'));

UPDATE feature_clusters SET status = 'idea' WHERE status = 'open';
UPDATE feature_clusters SET status = 'long_term' WHERE status = 'deferred';

ALTER TABLE feature_clusters ADD COLUMN category TEXT DEFAULT 'general';
CREATE INDEX idx_feature_clusters_category ON feature_clusters(category);
```

Categories: search, scraper, social, reader, admin, recs, general.

## Backend changes

### routes/roadmap.rs

**Arena:**
- keep `WHERE status = 'idea'` (already done per plan)
- arena state returns elo per feature for display

**Features list:**
- `GET /api/roadmap/features` accepts `status?`, `category?`
- ORDER BY elo DESC always
- for board: fetch all statuses, group by status in Rust or via separate queries per column

**Curator promotion:**
- `PATCH /api/roadmap/features/:id` body `{ status, category? }`
- guard role >= curator (5)
- on move `idea -> up_next`: optionally freeze (already done by status filter)

## Frontend (SvelteKit 5)

New route `/roadmap/board` (or replace list view).

**Columns (left to right by priority/urgency, StoryGraph order):**
1. Up Next — yellow
2. In Progress — orange
3. Finished (not yet shipped) — green
4. Shipped — emerald
5. Medium-term — pink
6. Long-term — dark purple
7. Ideas — blue (with Arena CTA at top)

**Tag filter chips (StoryGraph style):**
All, Search, Scrapers, Social, Reader, Admin, Recs — filters across all columns.

**KanbanCard:**
- title
- category badge
- Elo score (rounded) — proves community drove it
- representative_text (line-clamp 3)
- curator dropdown: Move to... (statuses)

**Arena CTA in Ideas column header:**
"Features ranked by community votes. Enter the Arena to compare and push favorites up."

**Sort control:** per column elo DESC (default, non-negotiable). Optional: sort by title asc/desc. No "popular" since elo replaces it.

**Search:** search bar filters cards by title across all columns (like StoryGraph features board).

## Changelog (future, out of scope for first pass)
Mimic StoryGraph: posts with date, labels New/Improved/Fixed, linked back to roadmap feature. Can reuse changelog table + OPDS-like posts.

## Workflow lifecycle
1. suggestion -> embed via ollama, dedup, insert `status=idea`, `elo=1500`, `category`
2. arena serves pairs of `idea` features, elo updates
3. curator sees Ideas column sorted by elo, moves top to Up Next
4. arena stops serving it (status != idea)
5. move In Progress -> Finished -> Shipped on deploy
6. users filter by category, see shipped feature with final elo

## Crate extraction steps
1. create `crates/consensus/` with lib.rs, Cargo.toml
2. move elo math from current arena code into `fichub_consensus`
3. add `RatingStore` trait, sqlx impl behind feature flag
4. add workspace member in root Cargo.toml
5. update fichub Cargo.toml `fichub-consensus = { path = "crates/consensus" }`
6. feature flag `sqlx-store` default on for fichub, off for standalone usage

## Verification
- migration runs on dev db, old `open`/`deferred` still visible as `idea`/`long_term`
- arena only shows idea-status features
- board shows all statuses, each column sorted by elo desc
- curator can move card, elo preserved
- consensus crate compiles standalone with `--no-default-features`

## Configurable stages (crate-level)

The crate does not hardcode column names or stage order. Pass a `RoadmapConfig` at construction:

```rust
pub struct RoadmapConfig {
    pub stages: Vec<Stage>,
    pub default_category: String,
    pub allow_voting_on_non_idea_stages: bool, // default false
    pub min_trust_level_for_voting: Option<u8>, // None = open to all
    pub ranking: RankingMode, // Elo (default) or SimpleVote
    pub elo_start: f64,      // default 1500
    pub elo_k: f64,          // default 32
}
pub struct Stage {
    pub id: String,        // matches DB status enum value
    pub label: String,
    pub color: String,     // hex
    pub roadmaps_position: f64, // sort order on board (lower first)
    pub frozen_for_arena: bool, // if true, not served in arena pairs
}
```

**Default config (general-purpose):** 7 stages matching StoryGraph order, `frozen_for_arena` true for all except `idea`, `allow_voting_on_non_idea_stages=false`, `min_trust_level_for_voting=None`, `ranking=Elo`.

**FicHub config (project override):** same stages but `frozen_for_arena=false` for all stages so users can keep voting on planned/shipped features, `min_trust_level_for_voting=2` (L2+), `ranking=Elo`.

Stage order is configurable, so other projects can rename/reorder stages (e.g. drop "Finished not yet shipped" or add custom columns) without forking the crate.

## Trust gating

Crate exposes a policy trait for voting eligibility:

```rust
pub trait VoterPolicy: Send + Sync {
    fn can_vote(&self, voter: &VoterRef) -> bool;
}
pub struct VoterRef {
    pub id: Option<String>,
    pub trust_level: u8, // 0 = anonymous/guest
}
```

Default impl: `allow_all`. FicHub impl: `trust_level >= 2`. Crate does not know about FicHub roles; FicHub converts its role/trust system into `VoterRef` at the API boundary.

If `min_trust_level_for_voting` is set in config, crate provides a convenience impl `TrustLevelPolicy` that checks `voter.trust_level >= min`. Projects can also supply custom `VoterPolicy` (e.g. karma, account age, captcha).

## Ranking mode

Config field `ranking: RankingMode`:

```rust
pub enum RankingMode {
    Elo(EloConfig),
    SimpleVote,
}
```

**Elo (default):** use MaxDiff pairwise matches as before. `EloConfig` exposes start, k, optional decay, optional cap.

**SimpleVote:** each feature has up/down counts; ranking by `up - down` or `up` (configurable sort). When `SimpleVote`, arena still serves pairs but the vote is recorded as up/down on the chosen feature rather than an Elo update; board sort uses vote score.

FicHub uses `Elo`. Other projects can set `SimpleVote` if they want StoryGraph-style upvote counts instead of consensus.

## Voting on planned features

Config `allow_voting_on_non_idea_stages`:
- **false (default):** arena only serves `frozen_for_arena == false` stages. Matches original plan: moving to `up_next` removes from arena.
- **true (FicHub override):** arena serves all stages; voters can still vote on features already planned or shipped. Elo still updates. Board still sorts by elo desc per column.

FicHub wants this on so consensus keeps accumulating even after planning. Crate default is off so generic users get the "freeze after planning" behavior out of the box.

## Crate config loading

- `RoadmapConfig` can be built programmatically or deserialized from serde (JSON/YAML) so each project can ship its own config file.
- FicHub can store config in DB or config file; crate does not require a specific source, only that a `RoadmapConfig` is passed to `Consensus::new(config)`.

## Backend wiring (FicHub)

### routes/roadmap.rs
- Arena: `WHERE status IN ($idea_stage_ids)` if `allow_voting_on_non_idea_stages` false, else no status filter beyond voter policy
- before recording a vote, check `VoterPolicy::can_vote`; return 403 if below L2
- ranking mode already Elo; if future project switches to SimpleVote, arena records up/down and board sorts by vote score

### config
- FicHub builds `RoadmapConfig` with frozen=false across stages, `min_trust_level_for_voting=2`, `ranking=Elo`
- stages list matches DB enum; DB CHECK constraint still required so crate and DB agree on allowed status values

## Migration note

Same 064 migration. Stage ids in crate config must match DB `status` enum values: `idea, long_term, medium_term, up_next, in_progress, finished, shipped, rejected`. If a project customizes stages, they must also alter the CHECK constraint and migration accordingly — crate does not auto-migrate DB schema, only validates in-memory config.

## Verification additions
- crate tests: default config freezes non-idea stages; FicHub config does not
- crate tests: `TrustLevelPolicy` with min=2 rejects L0/L1, allows L2+
- crate tests: `SimpleVote` records up/down and ranks by vote score; `Elo` records pairwise and ranks by elo
- FicHub config compiles, arena returns 403 for sub-L2 voters, arena serves all stages
- consensus crate compiles standalone with `--no-default-features` and without any DB
# fichub-consensus

Generic Elo/MaxDiff consensus engine for feature ranking, extracted from FicHub.

## Features

- Pure Elo math (no external deps)
- MaxDiff → virtual 1v1 Elo translation
- Configurable stages, categories, ranking modes
- Pluggable `RatingStore` trait with optional SQLx backend
- Voter policy trait for trust-gated voting
- Standalone: compiles with `--no-default-features`

## Usage

```rust
use fichub_consensus::{Consensus, ConsensusConfig, EloConfig, RankingMode, SqlxStore};

let config = ConsensusConfig::fichub(); // FicHub preset
let store = SqlxStore::new(pool.clone());
let consensus = Consensus::new(config, store);

// Record a MaxDiff vote (best vs worst among 4 options)
consensus.record_maxdiff(&cluster_ids, best_id, worst_id).await?;

// Get leaderboard sorted by Elo DESC
let leaderboard = consensus.leaderboard(None).await?;
```

## Config presets

```rust
// Generic default (freezes non-idea stages)
let config = ConsensusConfig::default();

// FicHub override (votes on all stages, L2+ trust gate)
let config = ConsensusConfig::fichub();
```
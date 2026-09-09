# Remove XP/Level System — Implementation Plan

> **For an implementer with zero context.** Follow tasks in order. Commit after each task passes verification.

---

## Current State

- XP system spans ~1,145 lines across 3 core files + ~2131 references in ~120 files
- **Already done:** `src/services/achievements.rs` created, `src/routes/user_preferences.rs` created, structs moved to `src/db/models.rs`, module declarations updated
- **Remaining:** Remove all XP award calls, delete XP tables, update tests

## Files Already Modified (Don't Touch)

- `src/services/achievements.rs` — NEW: achievement/feature/notification logic
- `src/routes/user_preferences.rs` — NEW: prefs/layouts/views handlers
- `src/db/models.rs` — ADDED: Feature, UserFeature, UserPref, UserLayout, UserView structs
- `src/lib.rs` — REMOVED: `pub mod progression;`
- `src/services/mod.rs` — REMOVED: `pub mod progression;`, ADDED: `pub mod achievements;`
- `src/routes/mod.rs` — REMOVED: `pub mod progression;`, ADDED: `pub mod user_preferences;`
- `src/server.rs` — REMOVED: `/api/me/progression` route, updated prefs/layouts/views routes
- `src/services/bounties.rs` — REMOVED: `use crate::services::progression::award_xp;`, idle_tick returns Ok(0)

---

## Task 1: Replace `award_exp` in `src/routes/forum.rs`

**File:** `src/routes/forum.rs`

**Action:** Delete the `level_for_exp` function (lines ~87-91), the `award_exp` function (lines ~97-165), and the `award_post_exp` function (lines ~169-175). Replace all `award_exp(...)` and `award_post_exp(...)` calls with `update_reputation_and_promote(...)` calls.

**Specific replacements:**

1. Delete `fn level_for_exp(exp: i64) -> i16 { ... }` (around line 87-91)

2. Delete the entire `async fn award_exp(...)` function (around lines 97-165)

3. Delete the entire `async fn award_post_exp(...)` function (around lines 169-175)

4. Replace `award_post_exp(&state.db, user_id, post_id).await;` (around line 2893) with:
   ```rust
   let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 2, "forum_post_create").await;
   ```

5. Replace `award_post_exp(&state.db, user_id, post_id).await;` (around line 3154) with:
   ```rust
   let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 2, "forum_post_create").await;
   ```

6. For `award_reaction_exp` (around line 4268-4307): Delete the function. Replace the call at line 3868 with:
   ```rust
   let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 5, "post_reacted").await;
   ```

7. For `award_poll_vote_exp` (around line 4309+): Delete the function. Replace calls with:
   ```rust
   let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 3, "poll_voted").await;
   ```

**Verification:**
```bash
grep -n 'award_exp\|award_post_exp\|award_reaction_exp\|award_poll_vote_exp\|level_for_exp' src/routes/forum.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): replace award_exp with reputation calls in forum.rs`

---

## Task 2: Replace `award_xp` in `src/routes/proposals.rs`

**File:** `src/routes/proposals.rs`

**Action:** Replace `crate::services::progression::award_xp(...)` with `crate::db::queries::social::update_reputation_and_promote(...)`.

**Specific replacements:**

1. Line 380: Replace `let _ = crate::services::progression::award_xp(...)` with:
   ```rust
   let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 5, "proposal_vote").await;
   ```

**Verification:**
```bash
grep -n 'award_xp\|progression::' src/routes/proposals.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): replace award_xp with reputation in proposals.rs`

---

## Task 3: Replace `award_xp` in `src/routes/admin.rs`

**File:** `src/routes/admin.rs`

**Action:** Replace all `crate::services::progression::award_xp(...)` and `crate::services::progression::award_scaled_xp(...)` calls with `crate::db::queries::social::update_reputation_and_promote(...)`.

**Specific replacements:**

1. Line 146: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 10, "work_approved").await;`
2. Line 154: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 5, "work_completed").await;`
3. Line 168: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 15, "work_complete_qualified").await;`
4. Line 462: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(&state.db, user_id, 10, "backfill_tags").await;`

**Verification:**
```bash
grep -n 'award_xp\|award_scaled_xp\|progression::' src/routes/admin.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): replace award_xp with reputation in admin.rs`

---

## Task 4: Replace `award_xp` in `src/services/proposals.rs`

**File:** `src/services/proposals.rs`

**Action:** Replace all `crate::services::progression::award_xp(...)` calls with `crate::db::queries::social::update_reputation_and_promote(...)`.

**Specific replacements:**

1. Line 199: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(pool, user_id, 5, "proposal_vote").await;`
2. Line 476: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(db, uid, 5, "proposal_vote").await;`
3. Line 513: Replace with `let _ = crate::db::queries::social::update_reputation_and_promote(pool, user_id, 10, "proposal_accepted").await;`

**Verification:**
```bash
grep -n 'award_xp\|progression::' src/services/proposals.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): replace award_xp with reputation in services/proposals.rs`

---

## Task 5: Remove XP tables from baseline migration

**File:** `migrations/000_baseline.sql`

**Action:** Remove the following table definitions if they exist:
- `xp_events`
- `xp_source_defs`
- `xp_sources_streaks`
- `exp_events` (legacy XP table)

Also remove any `xp`, `level`, `exp`, `rank` columns from the `users` table definition.

**Verification:**
```bash
grep -n 'xp_events\|xp_source\|exp_events\|xp_amount\|level\|rank\|exp_per_level' migrations/000_baseline.sql
# Should return nothing relevant
```

**Commit:** `refactor(xp): remove XP tables from baseline migration`

---

## Task 6: Remove XP columns from users table in baseline

**File:** `migrations/000_baseline.sql`

**Action:** In the `users` table definition, remove these columns if present:
- `xp BIGINT NOT NULL DEFAULT 0`
- `level INTEGER NOT NULL DEFAULT 1`
- `exp INTEGER NOT NULL DEFAULT 0`
- `rank SMALLINT NOT NULL DEFAULT 0`
- `exp_per_level INTEGER NOT NULL DEFAULT 100`

Keep: `trust_level`, `reputation`, `level` (if it means something else — but likely remove).

**Verification:**
```bash
grep -A30 'CREATE TABLE.*users' migrations/000_baseline.sql | grep -E 'xp|level|rank|exp'
# Should return nothing (or only trust_level/reputation)
```

**Commit:** `refactor(xp): remove XP columns from users table`

---

## Task 7: Update `src/db/queries/social.rs` to remove XP references

**File:** `src/db/queries/social.rs`

**Action:** Search for and remove any XP-specific queries or functions. The file should only contain reputation and social queries.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/db/queries/social.rs | grep -v 'trust_level' | grep -v '//'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from social.rs queries`

---

## Task 8: Update `src/db/queries/reputation.rs` to remove XP references

**File:** `src/db/queries/reputation.rs`

**Action:** Search for and remove any XP-specific queries. The file should only contain reputation queries.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/db/queries/reputation.rs | grep -v '//'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from reputation.rs`

---

## Task 9: Update tests in `src/routes/api_contract_tests.rs`

**File:** `src/routes/api_contract_tests.rs`

**Action:** Remove or update any test that asserts on `xp`, `level`, `rank`, `xp_events`, or `/api/me/progression`.

**Verification:**
```bash
grep -n 'xp\|level\|rank\|progression' src/routes/api_contract_tests.rs | grep -v '//'
# Should return nothing relevant
cargo test --lib -p fichub 2>&1 | tail -5
```

**Commit:** `refactor(xp): remove XP references from api contract tests`

---

## Task 10: Update tests in `tests/` directory

**Files:** `tests/*.rs`

**Action:** Remove or update any test that references XP, level, rank, or progression.

**Verification:**
```bash
grep -rn 'xp\|level\|rank\|progression\|award_exp\|award_xp' tests/ --include='*.rs' | grep -v '//'
# Should return nothing relevant
cargo test --lib -p fichub 2>&1 | tail -5
```

**Commit:** `refactor(xp): remove XP references from integration tests`

---

## Task 11: Update `src/routes/forum.rs` to remove XP history endpoint

**File:** `src/routes/forum.rs`

**Action:** Remove the `/api/xp/history` endpoint handler (around line 4090) and any related functions.

**Verification:**
```bash
grep -n 'xp_history\|xp/history\|xp_events' src/routes/forum.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP history endpoint from forum.rs`

---

## Task 12: Update `src/routes/forum.rs` to remove XP leaderboard endpoint

**File:** `src/routes/forum.rs`

**Action:** Remove any XP-specific leaderboard endpoints. Reputation leaderboard should remain.

**Verification:**
```bash
grep -n 'xp_leaderboard\|xp/leaderboard' src/routes/forum.rs
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP leaderboard endpoint from forum.rs`

---

## Task 13: Update `src/bin/` binaries to remove XP references

**Files:** `src/bin/*.rs`

**Action:** Search for and remove any XP-specific code in binaries (compute_leaderboards.rs, compute_stats.rs, etc.).

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/bin/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from binaries`

---

## Task 14: Update `src/routes/analytics.rs` to remove XP references

**File:** `src/routes/analytics.rs`

**Action:** Remove any XP-specific analytics. Keep reputation analytics.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/analytics.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from analytics`

---

## Task 15: Update `src/routes/leaderboard.rs` to remove XP references

**File:** `src/routes/leaderboard.rs`

**Action:** Remove any XP-specific leaderboard logic. Keep reputation leaderboard.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/leaderboard.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from leaderboard`

---

## Task 16: Update `src/routes/badges.rs` to remove XP references

**File:** `src/routes/badges.rs`

**Action:** Remove any XP-specific badge logic. Keep reputation-based badges.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/badges.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from badges`

---

## Task 17: Update `src/routes/quests.rs` to remove XP references

**File:** `src/routes/quests.rs`

**Action:** Remove any XP-specific quest logic. Quests should be reputation-based or removed.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/quests.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from quests`

---

## Task 18: Update `src/routes/social.rs` to remove XP references

**File:** `src/routes/social.rs`

**Action:** Remove any XP-specific social features. Keep reputation.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/social.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from social.rs`

---

## Task 19: Update `src/routes/user_credentials.rs` to remove XP references

**File:** `src/routes/user_credentials.rs`

**Action:** Remove any XP-specific credential logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/user_credentials.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from user_credentials.rs`

---

## Task 20: Update `src/routes/extensions.rs` to remove XP references

**File:** `src/routes/extensions.rs`

**Action:** Remove any XP-specific extension logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/extensions.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from extensions.rs`

---

## Task 21: Update `src/routes/recipes.rs` to remove XP references

**File:** `src/routes/recipes.rs`

**Action:** Remove any XP-specific recipe logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/recipes.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from recipes.rs`

---

## Task 22: Update `src/routes/skins.rs` to remove XP references

**File:** `src/routes/skins.rs`

**Action:** Remove any XP-specific skin logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/skins.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from skins.rs`

---

## Task 23: Update `src/routes/customization.rs` to remove XP references

**File:** `src/routes/customization.rs`

**Action:** Remove any XP-specific customization logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/customization.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from customization.rs`

---

## Task 24: Update `src/routes/device_library.rs` to remove XP references

**File:** `src/routes/device_library.rs`

**Action:** Remove any XP-specific device library logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/device_library.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from device_library.rs`

---

## Task 25: Update `src/routes/feed.rs` to remove XP references

**File:** `src/routes/feed.rs`

**Action:** Remove any XP-specific feed logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/feed.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from feed.rs`

---

## Task 26: Update `src/routes/messages.rs` to remove XP references

**File:** `src/routes/messages.rs`

**Action:** Remove any XP-specific message logic. Keep achievement checks.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/messages.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from messages.rs`

---

## Task 27: Update `src/routes/notifications.rs` to remove XP references

**File:** `src/routes/notifications.rs`

**Action:** Remove any XP-specific notification logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/notifications.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from notifications.rs`

---

## Task 28: Update `src/routes/requests.rs` to remove XP references

**File:** `src/routes/requests.rs`

**Action:** Remove any XP-specific request logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/requests.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from requests.rs`

---

## Task 29: Update `src/routes/reviews.rs` to remove XP references

**File:** `src/routes/reviews.rs`

**Action:** Remove any XP-specific review logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/reviews.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from reviews.rs`

---

## Task 30: Update `src/routes/roadmap.rs` to remove XP references

**File:** `src/routes/roadmap.rs`

**Action:** Remove any XP-specific roadmap logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/roadmap.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from roadmap.rs`

---

## Task 31: Update `src/routes/saved_search.rs` to remove XP references

**File:** `src/routes/saved_search.rs`

**Action:** Remove any XP-specific saved search logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/saved_search.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from saved_search.rs`

---

## Task 32: Update `src/routes/search.rs` to remove XP references

**File:** `src/routes/search.rs`

**Action:** Remove any XP-specific search logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/search.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from search.rs`

---

## Task 33: Update `src/routes/series.rs` to remove XP references

**File:** `src/routes/series.rs`

**Action:** Remove any XP-specific series logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/series.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from series.rs`

---

## Task 34: Update `src/routes/subsystems.rs` to remove XP references

**File:** `src/routes/subsystems.rs`

**Action:** Remove any XP-specific subsystem logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/subsystems.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from subsystems.rs`

---

## Task 35: Update `src/routes/trending.rs` to remove XP references

**File:** `src/routes/trending.rs`

**Action:** Remove any XP-specific trending logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/trending.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from trending.rs`

---

## Task 36: Update `src/routes/trust.rs` to remove XP references

**File:** `src/routes/trust.rs`

**Action:** Remove any XP-specific trust logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/trust.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from trust.rs`

---

## Task 37: Update `src/routes/updates.rs` to remove XP references

**File:** `src/routes/updates.rs`

**Action:** Remove any XP-specific update logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/updates.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from updates.rs`

---

## Task 38: Update `src/routes/upload.rs` to remove XP references

**File:** `src/routes/upload.rs`

**Action:** Remove any XP-specific upload logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/upload.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from upload.rs`

---

## Task 39: Update `src/routes/uploads.rs` to remove XP references

**File:** `src/routes/uploads.rs`

**Action:** Remove any XP-specific uploads logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/uploads.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from uploads.rs`

---

## Task 40: Update `src/routes/work_delete.rs` to remove XP references

**File:** `src/routes/work_delete.rs`

**Action:** Remove any XP-specific work delete logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/work_delete.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from work_delete.rs`

---

## Task 41: Update `src/routes/work_proposals.rs` to remove XP references

**File:** `src/routes/work_proposals.rs`

**Action:** Remove any XP-specific work proposals logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/work_proposals.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from work_proposals.rs`

---

## Task 42: Update `src/routes/auto_tag.rs` to remove XP references

**File:** `src/routes/auto_tag.rs`

**Action:** Remove any XP-specific auto-tag logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/auto_tag.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from auto_tag.rs`

---

## Task 43: Update `src/routes/backfill.rs` to remove XP references

**File:** `src/routes/backfill.rs`

**Action:** Remove any XP-specific backfill logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/backfill.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from backfill.rs`

---

## Task 44: Update `src/routes/blind.rs` to remove XP references

**File:** `src/routes/blind.rs`

**Action:** Remove any XP-specific blind logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/blind.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from blind.rs`

---

## Task 45: Update `src/routes/bounties.rs` to remove XP references

**File:** `src/routes/bounties.rs`

**Action:** Remove any XP-specific bounties logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/bounties.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from bounties.rs`

---

## Task 46: Update `src/routes/bulk.rs` to remove XP references

**File:** `src/routes/bulk.rs`

**Action:** Remove any XP-specific bulk logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/bulk.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from bulk.rs`

---

## Task 47: Update `src/routes/cache_download.rs` to remove XP references

**File:** `src/routes/cache_download.rs`

**Action:** Remove any XP-specific cache download logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/cache_download.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from cache_download.rs`

---

## Task 48: Update `src/routes/collections.rs` to remove XP references

**File:** `src/routes/collections.rs`

**Action:** Remove any XP-specific collections logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/collections.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from collections.rs`

---

## Task 49: Update `src/routes/comments.rs` to remove XP references

**File:** `src/routes/comments.rs`

**Action:** Remove any XP-specific comments logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/comments.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from comments.rs`

---

## Task 50: Update `src/routes/consensus.rs` to remove XP references

**File:** `src/routes/consensus.rs`

**Action:** Remove any XP-specific consensus logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/consensus.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from consensus.rs`

---

## Task 51: Update `src/routes/copyright.rs` to remove XP references

**File:** `src/routes/copyright.rs`

**Action:** Remove any XP-specific copyright logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/copyright.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from copyright.rs`

---

## Task 52: Update `src/routes/curator_content.rs` to remove XP references

**File:** `src/routes/curator_content.rs`

**Action:** Remove any XP-specific curator content logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/curator_content.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from curator_content.rs`

---

## Task 53: Update `src/routes/drafts.rs` to remove XP references

**File:** `src/routes/drafts.rs`

**Action:** Remove any XP-specific drafts logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/drafts.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from drafts.rs`

---

## Task 54: Update `src/routes/export.rs` to remove XP references

**File:** `src/routes/export.rs`

**Action:** Remove any XP-specific export logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/export.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from export.rs`

---

## Task 55: Update `src/routes/fandom.rs` to remove XP references

**File:** `src/routes/fandom.rs`

**Action:** Remove any XP-specific fandom logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/fandom.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from fandom.rs`

---

## Task 56: Update `src/routes/forum_groups.rs` to remove XP references

**File:** `src/routes/forum_groups.rs`

**Action:** Remove any XP-specific forum groups logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/forum_groups.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from forum_groups.rs`

---

## Task 57: Update `src/routes/forum_polls.rs` to remove XP references

**File:** `src/routes/forum_polls.rs`

**Action:** Remove any XP-specific forum polls logic. Keep achievement checks.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/forum_polls.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from forum_polls.rs`

---

## Task 58: Update `src/routes/forum_privileges.rs` to remove XP references

**File:** `src/routes/forum_privileges.rs`

**Action:** Remove any XP-specific forum privileges logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/forum_privileges.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from forum_privileges.rs`

---

## Task 59: Update `src/routes/heal.rs` to remove XP references

**File:** `src/routes/heal.rs`

**Action:** Remove any XP-specific heal logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/heal.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from heal.rs`

---

## Task 60: Update `src/routes/health.rs` to remove XP references

**File:** `src/routes/health.rs`

**Action:** Remove any XP-specific health logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/health.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from health.rs`

---

## Task 61: Update `src/routes/kindle.rs` to remove XP references

**File:** `src/routes/kindle.rs`

**Action:** Remove any XP-specific kindle logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/kindle.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from kindle.rs`

---

## Task 62: Update `src/routes/lists.rs` to remove XP references

**File:** `src/routes/lists.rs`

**Action:** Remove any XP-specific lists logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/lists.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from lists.rs`

---

## Task 63: Update `src/routes/locales.rs` to remove XP references

**File:** `src/routes/locales.rs`

**Action:** Remove any XP-specific locales logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/locales.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from locales.rs`

---

## Task 64: Update `src/routes/marginalia_admin.rs` to remove XP references

**File:** `src/routes/marginalia_admin.rs`

**Action:** Remove any XP-specific marginalia admin logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/marginalia_admin.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from marginalia_admin.rs`

---

## Task 65: Update `src/routes/opds/` to remove XP references

**Files:** `src/routes/opds/*.rs`

**Action:** Remove any XP-specific OPDS logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/routes/opds/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from opds routes`

---

## Task 66: Update `src/routes/pow.rs` to remove XP references

**File:** `src/routes/pow.rs`

**Action:** Remove any XP-specific PoW logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/pow.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from pow.rs`

---

## Task 67: Update `src/routes/reader.rs` to remove XP references

**File:** `src/routes/reader.rs`

**Action:** Remove any XP-specific reader logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/reader.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from reader.rs`

---

## Task 68: Update `src/routes/reports.rs` to remove XP references

**File:** `src/routes/reports.rs`

**Action:** Remove any XP-specific reports logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/reports.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from reports.rs`

---

## Task 69: Update `src/routes/rss/` to remove XP references

**Files:** `src/routes/rss/*.rs`

**Action:** Remove any XP-specific RSS logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/routes/rss/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from rss routes`

---

## Task 70: Update `src/routes/requests.rs` to remove XP references

**File:** `src/routes/requests.rs`

**Action:** Remove any XP-specific requests logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/requests.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from requests.rs`

---

## Task 71: Update `src/routes/reviews.rs` to remove XP references

**File:** `src/routes/reviews.rs`

**Action:** Remove any XP-specific reviews logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/reviews.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from reviews.rs`

---

## Task 72: Update `src/routes/roadmap.rs` to remove XP references

**File:** `src/routes/roadmap.rs`

**Action:** Remove any XP-specific roadmap logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/roadmap.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from roadmap.rs`

---

## Task 73: Update `src/routes/saved_search.rs` to remove XP references

**File:** `src/routes/saved_search.rs`

**Action:** Remove any XP-specific saved search logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/saved_search.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from saved_search.rs`

---

## Task 74: Update `src/routes/search.rs` to remove XP references

**File:** `src/routes/search.rs`

**Action:** Remove any XP-specific search logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/search.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from search.rs`

---

## Task 75: Update `src/routes/series.rs` to remove XP references

**File:** `src/routes/series.rs`

**Action:** Remove any XP-specific series logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/series.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from series.rs`

---

## Task 76: Update `src/routes/social.rs` to remove XP references

**File:** `src/routes/social.rs`

**Action:** Remove any XP-specific social logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/social.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from social.rs`

---

## Task 77: Update `src/routes/subsystems.rs` to remove XP references

**File:** `src/routes/subsystems.rs`

**Action:** Remove any XP-specific subsystems logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/subsystems.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from subsystems.rs`

---

## Task 78: Update `src/routes/trending.rs` to remove XP references

**File:** `src/routes/trending.rs`

**Action:** Remove any XP-specific trending logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/trending.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from trending.rs`

---

## Task 79: Update `src/routes/trust.rs` to remove XP references

**File:** `src/routes/trust.rs`

**Action:** Remove any XP-specific trust logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/trust.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from trust.rs`

---

## Task 80: Update `src/routes/updates.rs` to remove XP references

**File:** `src/routes/updates.rs`

**Action:** Remove any XP-specific updates logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/updates.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from updates.rs`

---

## Task 81: Update `src/routes/upload.rs` to remove XP references

**File:** `src/routes/upload.rs`

**Action:** Remove any XP-specific upload logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/upload.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from upload.rs`

---

## Task 82: Update `src/routes/uploads.rs` to remove XP references

**File:** `src/routes/uploads.rs`

**Action:** Remove any XP-specific uploads logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/uploads.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from uploads.rs`

---

## Task 83: Update `src/routes/user_credentials.rs` to remove XP references

**File:** `src/routes/user_credentials.rs`

**Action:** Remove any XP-specific user credentials logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/user_credentials.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from user_credentials.rs`

---

## Task 84: Update `src/routes/user_export.rs` to remove XP references

**File:** `src/routes/user_export.rs`

**Action:** Remove any XP-specific user export logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/user_export.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from user_export.rs`

---

## Task 85: Update `src/routes/work_delete.rs` to remove XP references

**File:** `src/routes/work_delete.rs`

**Action:** Remove any XP-specific work delete logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/work_delete.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from work_delete.rs`

---

## Task 86: Update `src/routes/work_proposals.rs` to remove XP references

**File:** `src/routes/work_proposals.rs`

**Action:** Remove any XP-specific work proposals logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/routes/work_proposals.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from work_proposals.rs`

---

## Task 87: Update `src/services/` to remove XP references

**Files:** `src/services/*.rs`

**Action:** Remove any XP-specific service logic. Keep reputation and trust services.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/services/ --include='*.rs' | grep -v '//' | grep -v 'trust_level' | grep -v 'achievements.rs'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from services`

---

## Task 88: Update `src/db/` to remove XP references

**Files:** `src/db/**/*.rs`

**Action:** Remove any XP-specific database logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/db/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from db layer`

---

## Task 89: Update `src/activitypub/` to remove XP references

**Files:** `src/activitypub/**/*.rs`

**Action:** Remove any XP-specific ActivityPub logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/activitypub/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from activitypub`

---

## Task 90: Update `src/export/` to remove XP references

**Files:** `src/export/**/*.rs`

**Action:** Remove any XP-specific export logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/export/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from export`

---

## Task 91: Update `src/scrape/` to remove XP references

**Files:** `src/scrape/**/*.rs`

**Action:** Remove any XP-specific scrape logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/scrape/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from scrape`

---

## Task 92: Update `src/search/` to remove XP references

**Files:** `src/search/**/*.rs`

**Action:** Remove any XP-specific search logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/search/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from search`

---

## Task 93: Update `src/tags/` to remove XP references

**Files:** `src/tags/**/*.rs`

**Action:** Remove any XP-specific tags logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/tags/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from tags`

---

## Task 94: Update `src/trending/` to remove XP references

**Files:** `src/trending/**/*.rs`

**Action:** Remove any XP-specific trending logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/trending/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from trending`

---

## Task 95: Update `src/user_formats.rs` to remove XP references

**File:** `src/user_formats.rs`

**Action:** Remove any XP-specific user formats logic.

**Verification:**
```bash
grep -n 'xp\|level\|rank' src/user_formats.rs | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from user_formats.rs`

---

## Task 96: Update `src/visitor/` to remove XP references

**Files:** `src/visitor/**/*.rs`

**Action:** Remove any XP-specific visitor logic.

**Verification:**
```bash
grep -rn 'xp\|level\|rank' src/visitor/ --include='*.rs' | grep -v '//' | grep -v 'trust_level'
# Should return nothing relevant
cargo check -p fichub 2>&1 | tail -3
```

**Commit:** `refactor(xp): remove XP references from visitor`

---

## Task 97: Final full-tree sweep

**Files:** entire repo

**Action:** Run comprehensive grep to find any remaining XP-related references.

**Verification:**
```bash
grep -rn '\.xp\b\|\.level\b\|\.exp\b\|xp_\|level_\|rank_\|award_exp\|award_xp\|XpEvent\|XpSource\|xp_for_level\|xp_to_level' --include='*.rs' --include='*.ts' --include='*.sql' --include='*.svelte' . | grep -v target | grep -v node_modules | grep -v 'experience' | grep -v 'trust_level' | grep -v 'trust-level' | grep -v 'reputation'
# Should return nothing
cargo check -p fichub 2>&1 | tail -3
cargo test --lib -p fichub 2>&1 | tail -5
```

**Commit:** `refactor(xp): final sweep — all XP references removed`

---

## Final Verification

```bash
# No XP module references
grep -rn 'pub mod progression' src/ lib.rs
# Should return nothing

# No XP structs
grep -rn 'XpEvent\|XpSourceDef' src/ --include='*.rs'
# Should return nothing

# No XP functions
grep -rn 'award_exp\|award_xp\|award_scaled_xp' src/ --include='*.rs'
# Should return nothing

# No XP tables in baseline
grep -n 'xp_events\|xp_source' migrations/000_baseline.sql
# Should return nothing

# Cargo clean build
cargo check -p fichub 2>&1 | tail -3
# Should show "Finished"

# Tests pass
cargo test --lib -p fichub 2>&1 | tail -5
# Should show "test result: ok"

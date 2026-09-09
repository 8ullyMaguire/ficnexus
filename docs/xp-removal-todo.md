# TODO — XP/Level System Removal (Remaining Tasks)

## High Priority (Compilation Blockers)

- [ ] **Fix `admin.rs`**: Remove `crate::services::progression::award_xp` and `award_scaled_xp` calls (lines 146, 154, 168, 462, 453, 430)
- [ ] **Fix `proposals.rs`**: Remove `crate::services::progression::award_xp` calls (line 380)
- [ ] **Fix `services/proposals.rs`**: Remove `crate::services::progression::award_xp` calls (lines 199, 476, 513)
- [ ] **Fix `config.rs`**: Remove `forum_exp_per_level` field and env var parsing
- [ ] **Remove `xp_events`, `xp_source_defs`, `exp_events` tables from `migrations/000_baseline.sql`**
- [ ] **Remove XP columns from users table in baseline: `xp`, `level`, `exp`, `rank`, `exp_per_level`**

## Medium Priority (Clean Code)

- [ ] **Remove `modlog.rs` XP test data (lines 142, 153)**
- [ ] **Remove `authors.rs` XP test data (lines 736, 744, 752, 764, 772, 783)**
- [ ] **Remove `social.rs` XP test data (lines 240, 241, 348, 1090, 1091, 1108)**
- [ ] **Remove `forum_groups.rs` XP level checks (lines 68, 153, 308, 506, 627, 663, 699)**
- [ ] **Remove `forum_polls.rs` XP level checks (lines 162, 372)**
- [ ] **Remove `uploads.rs` XP level check (line 186)**
- [ ] **Remove `roadmap.rs` XP level references (lines 58, 296, 297, 565, 782)**
- [ ] **Remove `backfill.rs` XP level check (line 35)**
- [ ] **Remove `leaderboard.rs` XP rank reference (line 144)**

## Low Priority (Tests & Frontend)

- [ ] **Update tests in `tests/` directory**
- [ ] **Update tests in `src/routes/api_contract_tests.rs`**
- [ ] **Remove XP from frontend types/API calls**
- [ ] **Remove XP from `src/bin/` binaries**
- [ ] **Remove XP from `src/routes/analytics.rs`**
- [ ] **Remove XP from `src/routes/badges.rs`**
- [ ] **Remove XP from `src/routes/quests.rs`**

## Deployment

- [ ] **Deploy to ThinkCentre: `ssh thinkcentre 'cd /home/alvaro/code/rust/ficnexus && git pull && cargo build --release && sudo systemctl restart fichub'`**

---

## Summary

**Completed:**
- ✅ Deleted `src/progression.rs`, `src/services/progression.rs`, `src/routes/progression.rs`
- ✅ Created `src/services/achievements.rs` with achievement/feature/notification logic
- ✅ Created `src/routes/user_preferences.rs` with prefs/layouts/views handlers
- ✅ Moved Feature/UserFeature/UserPref/UserLayout/UserView to `src/db/models.rs`
- ✅ Updated module declarations in `lib.rs`, `services/mod.rs`, `routes/mod.rs`
- ✅ Updated `server.rs` routes
- ✅ Cleaned up `forum.rs` (removed all XP functions and calls)
- ✅ Cleaned up `bounties.rs`

**Remaining:** ~20 files with XP references, mostly test data and level checks that need to be replaced with trust_level checks.

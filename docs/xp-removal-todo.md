# TODO — XP/Level System Removal

## Status: PARTIAL — Compilation errors remain

**Completed commits:**
- ✅ `dd39d47` — Deleted progression modules, created achievements.rs, user_preferences.rs
- ✅ `10cf8b5` — Cleaned up forum.rs XP functions
- ✅ `f36f122` — Added this TODO list

## Remaining Work (Compilation Blockers)

### `src/routes/forum.rs`
- [ ] Remove `award_exp` function (lines ~97-165)
- [ ] Remove `level_for_exp` function (lines ~89-91)
- [ ] Remove `exp_per_level` function (lines ~80-87)
- [ ] Remove `award_post_exp` function (lines ~169-175)
- [ ] Remove `award_mod_received_exp` function (lines ~182-205)
- [ ] Remove `my_level` function (lines ~207-240)
- [ ] Remove `user_xp_history` function (lines ~3950-4010)
- [ ] Remove `award_reaction_exp` function (lines ~4268-4290)
- [ ] Remove `award_poll_vote_exp` function (lines ~4293-4315)
- [ ] Replace all `award_exp(...)` calls with `update_reputation_and_promote(...)`
- [ ] Remove XP fields from user profile query (line ~3890)
- [ ] Remove XP history query (line ~3951)
- [ ] Remove today's XP by type query (line ~3960)

### `src/routes/admin.rs`
- [ ] Replace `crate::services::progression::award_xp` (line 146, 154, 462)
- [ ] Replace `crate::services::progression::award_scaled_xp` (line 168)
- [ ] Remove `xp_source_defs` query (line 453)

### `src/services/proposals.rs`
- [ ] Replace `crate::services::progression::award_xp` (lines 199, 476, 513)

### `src/config.rs`
- [ ] Remove `forum_exp_per_level` field (line 349)
- [ ] Remove `forum_exp_topic_create` field (line 351)
- [ ] Remove `forum_exp_post_create` field (line 353)
- [ ] Remove `forum_exp_mod_received` field (line 355)
- [ ] Remove `forum_exp_mod_daily_cap` field (line 357)
- [ ] Remove env var parsing (lines 1211-1230)
- [ ] Remove test assertions (lines 1745-1749)

### `src/main.rs`
- [ ] Remove `pub mod progression;` (line 17)

### Database
- [ ] Remove `xp_events` table from `migrations/000_baseline.sql`
- [ ] Remove `xp_source_defs` table from `migrations/000_baseline.sql`
- [ ] Remove `xp_sources_streaks` table from `migrations/000_baseline.sql`
- [ ] Remove `exp_events` table from `migrations/000_baseline.sql`
- [ ] Remove `xp`, `level`, `exp`, `rank` columns from users table in baseline

### Tests & Frontend
- [ ] Update tests in `tests/` directory
- [ ] Update tests in `src/routes/api_contract_tests.rs`
- [ ] Remove XP from frontend types/API calls
- [ ] Remove XP from `src/bin/` binaries

## Deployment

**BLOCKED** — Cannot deploy until compilation errors are fixed.

```bash
# Deploy command (run after fixes):
ssh thinkcentre 'cd /home/alvaro/code/rust/ficnexus && git pull && cargo build --release && sudo systemctl restart fichub'
```

## Summary

~20 files need XP references removed. Most are mechanical replacements of `award_exp`/`award_xp` with `update_reputation_and_promote`. The main risk is breaking compilation if function signatures don't match.

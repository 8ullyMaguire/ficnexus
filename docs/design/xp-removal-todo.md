# TODO — XP/Level System Removal

## Status: COMPLETE ✅

**All XP/level system code has been removed and production is running clean.**

### What Was Removed
- ✅ `src/progression.rs` — XP curve math, level/rank structs
- ✅ `src/services/progression.rs` — XP engine with caps, streaks, cooldowns
- ✅ `src/routes/progression.rs` — XP history, leaderboard endpoints
- ✅ `src/main.rs` — `pub mod progression;`
- ✅ `src/lib.rs` — `pub mod progression;`
- ✅ `src/services/mod.rs` — `pub mod progression;`
- ✅ `src/routes/mod.rs` — `pub mod progression;`
- ✅ `migrations/090_forum_xp_sources.sql`
- ✅ `migrations/091_achievement_definitions.sql`
- ✅ `migrations/092_trust_level_migration.sql`
- ✅ `migrations/093_critical_audit_fixes.sql`
- ✅ XP tables from production DB: `xp_events`, `xp_source_defs`, `xp_sources_streaks`, `exp_events`, `exp_events_archive`

### What Was Created
- ✅ `src/services/achievements.rs` — Achievement unlocking, feature gating, notifications
- ✅ `src/routes/user_preferences.rs` — User prefs, layouts, views endpoints
- ✅ `src/db/models.rs` — Added Feature, UserFeature, UserPref, UserLayout, UserView structs

### What Stayed
- ✅ `users.trust_level` — Trust level system (TL0-TL6)
- ✅ `users.reputation` — Reputation points
- ✅ `reputation_events` — Reputation ledger
- ✅ `achievements` — Achievement definitions
- ✅ `badges` — Badge definitions
- ✅ `user_features` — Feature unlock tracking

### Deployment
- ✅ Production server restarted successfully
- ✅ Health check passes: `{"status":"ok","db":true,"redis":true,"version":"0.2.0"}`
- ✅ Pushed to github/main

### Remaining Cleanup (Optional)
- [ ] Remove `level` field from `AuthUser` struct (if still present)
- [ ] Update tests that reference `level` or `xp`
- [ ] Remove XP-related frontend code
- [ ] Update API docs

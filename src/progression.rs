//! V3 progression, feature-gating, and user-customization models.
//!
//! XP is earned via tracked events, accumulated into a level, and
//! that level maps to a rank.  Features are gated by rank or trust
//! score; users can toggle them, pin favourites, and customise their
//! layout / preferences / saved views.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ── XP ───────────────────────────────────────────────────────────────────────

/// A single XP-award event stored in the `xp_events` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XpEvent {
    pub id: i64,
    pub user_id: i32,
    pub event_type: String,
    pub xp: i32,
    pub source_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Canonical XP award definition stored in `xp_source_defs`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XpSourceDef {
    pub event_type: String,
    pub xp_amount: i32,
    pub daily_cap: Option<i32>,
    pub description: Option<String>,
}

// ── Features ─────────────────────────────────────────────────────────────────

/// A feature that can be gated, toggled, and pinned by users.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feature {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub long_help: String,
    pub icon: Option<String>,
    pub category: String,
    pub gate_type: String,
    pub gate_value: i32,
    pub requires_feature: Option<String>,
    pub is_default: bool,
    pub is_revocable: bool,
    pub admin_only: bool,
    pub widget_component: Option<String>,
    pub nav_target: Option<String>,
    pub sort_hint: i32,
    pub created_at: DateTime<Utc>,
}

/// A user's relationship to a feature: unlocked, enabled, pinned.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserFeature {
    pub user_id: i32,
    pub feature_id: i32,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub enabled_at: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub sort_order: i32,
}

// ── Preferences & Layout ─────────────────────────────────────────────────────

/// A single user preference stored as a JSONB value.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPref {
    pub user_id: i32,
    pub key: String,
    pub value: serde_json::Value,
}

/// A per-page dashboard layout stored as JSONB.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLayout {
    pub user_id: i32,
    pub page: String,
    pub layout: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// A saved search / filter view that the user can pin and switch between.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserView {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub query: serde_json::Value,
    pub pinned: bool,
}

// ── Progression helpers ──────────────────────────────────────────────────────

/// Cumulative XP required to reach a given level.
///
/// The cost curve is `50 * level^1.6`, summed from 1..=level.
/// Level 0 always costs 0 XP.
pub fn xp_for_level(level: i32) -> i64 {
    if level <= 0 {
        return 0;
    }
    let mut total: f64 = 0.0;
    for l in 1..=level {
        total += 50.0 * (l as f64).powf(1.6);
    }
    total.round() as i64
}

/// Invert the XP curve: given a total XP, return the current level.
///
/// Iterates from 1 upward until the cumulative cost exceeds `xp`,
/// then backs up by one level.  Capped at 200.
pub fn xp_to_level(xp: i64) -> i32 {
    let mut cumulative: f64 = 0.0;
    let xp_f = xp as f64;
    for level in 1..=200 {
        cumulative += 50.0 * (level as f64).powf(1.6);
        if cumulative > xp_f {
            return level - 1;
        }
    }
    200
}

/// Map a level (0–200) to a rank number (1–10).
///
/// ```text
/// L01–L10  → R1    (Reader)
/// L11–L20  → R2    (Visitor)
/// L21–L30  → R3    (Regular)
/// L31–L40  → R4    (Enthusiast)
/// L41–L50  → R5    (Collector)
/// L51–L60  → R6    (Contributor)
/// L61–L70  → R7    (Voice)
/// L71–L80  → R8    (Power User)
/// L81–L90  → R9    (Archivist)
/// L91–L100 → R10   (Curator Emeritus)
/// ```
pub fn xp_to_rank(level: i32) -> i32 {
    match level {
        0..=10 => 1,
        11..=20 => 2,
        21..=30 => 3,
        31..=40 => 4,
        41..=50 => 5,
        51..=60 => 6,
        61..=70 => 7,
        71..=80 => 8,
        81..=90 => 9,
        _ => 10, // 91+
    }
}

/// Human-readable title for a rank number (1–10).
pub fn rank_title(rank: i32) -> &'static str {
    match rank {
        1 => "Reader",
        2 => "Visitor",
        3 => "Regular",
        4 => "Enthusiast",
        5 => "Collector",
        6 => "Contributor",
        7 => "Voice",
        8 => "Power User",
        9 => "Archivist",
        10 => "Curator Emeritus",
        _ => "Reader",
    }
}

/// XP still needed to reach `level + 1`.
///
/// Returns 0 if the user is already at level 200.
pub fn xp_for_next_level(level: i32) -> i64 {
    if level >= 200 {
        return 0;
    }
    xp_for_level(level + 1) - xp_for_level(level)
}

/// Convenience: given a total XP, compute both the level and rank.
pub fn calculate_level_and_rank(xp: i64) -> (i32, i32) {
    let level = xp_to_level(xp);
    let rank = xp_to_rank(level);
    (level, rank)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_zero_costs_zero() {
        assert_eq!(xp_for_level(0), 0);
    }

    #[test]
    fn xp_to_level_inverts_xp_for_level() {
        // Level 0 → 0 XP
        assert_eq!(xp_to_level(0), 0);

        // Level 1 boundary
        let cost1 = xp_for_level(1);
        assert_eq!(xp_to_level(cost1), 1);
        assert_eq!(xp_to_level(cost1 - 1), 0);

        // Level 10 boundary
        let cost10 = xp_for_level(10);
        assert_eq!(xp_to_level(cost10), 10);
        assert_eq!(xp_to_level(cost10 - 1), 9);
    }

    #[test]
    fn rank_mapping() {
        assert_eq!(xp_to_rank(0), 1);
        assert_eq!(xp_to_rank(10), 1);
        assert_eq!(xp_to_rank(11), 2);
        assert_eq!(xp_to_rank(50), 5);
        assert_eq!(xp_to_rank(100), 10);
        assert_eq!(xp_to_rank(200), 10);
    }

    #[test]
    fn rank_titles() {
        assert_eq!(rank_title(1), "Reader");
        assert_eq!(rank_title(5), "Collector");
        assert_eq!(rank_title(10), "Curator Emeritus");
    }

    #[test]
    fn next_level_cost_positive() {
        assert!(xp_for_next_level(0) > 0);
        assert!(xp_for_next_level(50) > 0);
    }

    #[test]
    fn calculate_level_and_rank_consistent() {
        let xp = 5000;
        let (level, rank) = calculate_level_and_rank(xp);
        assert_eq!(rank, xp_to_rank(level));
    }
}

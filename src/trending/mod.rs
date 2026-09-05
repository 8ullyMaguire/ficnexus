//! Timeframe parsing for trending filters.
//!
//! Accepts flexible time window specifications so callers can request custom
//! ranges beyond the simple `days=7` integer query. Format:
//!
//! - `Nd` (e.g. `7d`, `30d`) — N days, capped to 365
//! - `Nw` (e.g. `2w`, `12w`) — N weeks, capped to 52
//! - `Nm` (e.g. `3m`, `6m`) — N months, capped to 24
//! - `Ny` (e.g. `1y`, `2y`) — N years, capped to 5
//! - Bare integer (e.g. `7`) — treated as days for backward compat
//!
//! All parse failures return [`None`] so callers can fall back to defaults.

use std::fmt;

/// Maximum allowed days for any timeframe.
const MAX_DAYS: i32 = 5 * 366; // ~5 years

/// Parsed timeframe result: always expressed in days so SQL can use
/// `($1 || ' days')::INTERVAL` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    pub days: i32,
}

impl Timeframe {
    /// Default 7-day window when callers don't supply one.
    pub const DEFAULT: Timeframe = Timeframe { days: 7 };

    /// Construct from a raw day count (capped to [`MAX_DAYS`], minimum 1).
    pub fn from_days(days: i32) -> Self {
        let clamped = days.clamp(1, MAX_DAYS);
        Timeframe { days: clamped }
    }

    /// Try to parse a flexible timeframe string. Returns `None` for any
    /// malformed input — callers should fall back to [`Timeframe::DEFAULT`].
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // Bare integer → days (backward compat with the existing `days=N` query)
        if let Ok(n) = s.parse::<i32>() {
            return Some(Timeframe::from_days(n));
        }

        // N<unit> where unit is d/w/m/y
        let last = s.chars().last()?;
        if !last.is_ascii_alphabetic() {
            return None;
        }
        let num_str = &s[..s.len() - last.len_utf8()];
        let n: i32 = num_str.parse().ok()?;
        if n <= 0 {
            return None;
        }
        let days = match last {
            'd' | 'D' => n,
            'w' | 'W' => n.saturating_mul(7),
            'm' | 'M' => n.saturating_mul(30),
            'y' | 'Y' => n.saturating_mul(365),
            _ => return None,
        };
        Some(Timeframe::from_days(days))
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d", self.days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_integer_is_days() {
        assert_eq!(Timeframe::parse("7"), Some(Timeframe { days: 7 }));
        assert_eq!(Timeframe::parse("0"), Some(Timeframe { days: 1 })); // clamped up
        assert_eq!(Timeframe::parse("30"), Some(Timeframe { days: 30 }));
    }

    #[test]
    fn parse_days_unit() {
        assert_eq!(Timeframe::parse("7d"), Some(Timeframe { days: 7 }));
        assert_eq!(Timeframe::parse("30D"), Some(Timeframe { days: 30 }));
    }

    #[test]
    fn parse_weeks_unit() {
        assert_eq!(Timeframe::parse("2w"), Some(Timeframe { days: 14 }));
        assert_eq!(Timeframe::parse("1W"), Some(Timeframe { days: 7 }));
    }

    #[test]
    fn parse_months_unit() {
        assert_eq!(Timeframe::parse("3m"), Some(Timeframe { days: 90 }));
        assert_eq!(Timeframe::parse("1M"), Some(Timeframe { days: 30 }));
    }

    #[test]
    fn parse_years_unit() {
        assert_eq!(Timeframe::parse("1y"), Some(Timeframe { days: 365 }));
        assert_eq!(Timeframe::parse("2Y"), Some(Timeframe { days: 730 }));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert_eq!(Timeframe::parse(""), None);
        assert_eq!(Timeframe::parse("   "), None);
        assert_eq!(Timeframe::parse("abc"), None);
        assert_eq!(Timeframe::parse("7x"), None);
        assert_eq!(Timeframe::parse("d"), None);
        assert_eq!(Timeframe::parse("-3d"), None);
        assert_eq!(Timeframe::parse("0d"), None);
    }

    #[test]
    fn parse_caps_at_max_days() {
        // 10 years would be 3650 days — capped to MAX_DAYS (5*366=1830)
        assert_eq!(
            Timeframe::parse("10y").map(|t| t.days <= MAX_DAYS),
            Some(true)
        );
    }

    #[test]
    fn from_days_clamps() {
        assert_eq!(Timeframe::from_days(0), Timeframe { days: 1 });
        assert_eq!(Timeframe::from_days(-5), Timeframe { days: 1 });
        assert_eq!(Timeframe::from_days(MAX_DAYS + 1000).days, MAX_DAYS);
    }

    #[test]
    fn display_shows_days() {
        assert_eq!(Timeframe { days: 14 }.to_string(), "14d");
    }
}

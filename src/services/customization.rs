//! Customization service: theme preferences, home layout, widget registry,
//! and navigation-item derivation from activated features.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::error::AppError;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct NavItem {
    pub slug: String,
    pub name: String,
    pub nav_target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetInfo {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub min_rank: i32,
    pub icon: String,
}

// ── Service ──────────────────────────────────────────────────────────────────

pub struct CustomizationService;

impl CustomizationService {
    /// Return the default theme configuration.
    pub fn default_theme() -> Value {
        json!({
            "mode": "dark",
            "accent": "#7c3aed",
            "accent_name": "violet",
            "font": "inter",
            "density": "comfortable",
            "font_size": "base",
            "reader_theme": "dark",
            "reader_font": "serif",
            "reader_width": "medium",
            "reader_line_height": 1.8
        })
    }

    /// Fetch the user's theme preferences, falling back to defaults for
    /// any key that the user has not explicitly set.
    pub async fn get_theme(pool: &PgPool, user_id: i32) -> Result<Value, AppError> {
        let defaults = Self::default_theme();

        let rows = sqlx::query_as::<_, (String, Value)>(
            "SELECT key, value FROM user_prefs WHERE user_id = $1 AND key LIKE 'theme.%'",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut result = defaults;
        for (key, value) in rows {
            // Strip the "theme." prefix — the UI works with bare keys
            // inside the theme object (e.g. "theme.mode" → "mode").
            if let Some(bare) = key.strip_prefix("theme.") {
                result[bare] = value;
            }
        }

        Ok(result)
    }

    /// Upsert a set of theme preferences under the `theme.*` namespace.
    pub async fn set_theme(pool: &PgPool, user_id: i32, theme: Value) -> Result<(), AppError> {
        if let Some(obj) = theme.as_object() {
            for (k, v) in obj {
                sqlx::query(
                    "INSERT INTO user_prefs (user_id, key, value) VALUES ($1, $2, $3)
                     ON CONFLICT (user_id, key) DO UPDATE SET value = $3",
                )
                .bind(user_id)
                .bind(format!("theme.{k}"))
                .bind(v)
                .execute(pool)
                .await
                .map_err(|e| AppError::Database(format!("Failed to save theme pref: {e}")))?;
            }
        }
        Ok(())
    }

    /// Derive navigation items from the `features` table, filtered by the
    /// user's enabled state (via `user_features`).
    ///
    /// - Authenticated users see every feature whose `nav_target` is set
    ///   **and** which is enabled for them (or is a default feature).
    /// - Anonymous users see only default features with a nav_target.
    pub async fn get_nav_items(
        pool: &PgPool,
        user_id: Option<i32>,
    ) -> Result<Vec<NavItem>, AppError> {
        let rows: Vec<(String, String, Option<String>)> = match user_id {
            Some(uid) => {
                sqlx::query_as(
                    "SELECT f.slug, f.name, f.nav_target
                     FROM features f
                     LEFT JOIN user_features uf
                       ON f.id = uf.feature_id AND uf.user_id = $1
                     WHERE f.nav_target IS NOT NULL
                       AND (uf.enabled = true OR f.is_default = true)
                     ORDER BY f.sort_hint",
                )
                .bind(uid)
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    "SELECT slug, name, nav_target
                     FROM features
                     WHERE nav_target IS NOT NULL AND is_default = true
                     ORDER BY sort_hint",
                )
                .fetch_all(pool)
                .await?
            }
        };

        let items = rows
            .into_iter()
            .filter_map(|(slug, name, nav_target)| {
                nav_target.map(|nt| NavItem {
                    slug,
                    name,
                    nav_target: nt,
                })
            })
            .collect();

        Ok(items)
    }

    /// Return the user's home-page layout, falling back to a sensible
    /// default when no custom layout has been saved.
    pub async fn get_home_layout(pool: &PgPool, user_id: i32) -> Result<Value, AppError> {
        let row = sqlx::query_as::<_, (Value,)>(
            "SELECT layout FROM user_layouts WHERE user_id = $1 AND page = 'home'",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((layout,)) => Ok(layout),
            None => Ok(json!([
                { "widget": "DownloadInput", "size": "full" },
                { "widget": "Trending",      "size": "full" }
            ])),
        }
    }

    /// Return the registry of available widgets.
    pub fn available_widgets() -> Vec<WidgetInfo> {
        vec![
            WidgetInfo {
                slug: "DownloadInput".into(),
                name: "Download a Fic".into(),
                description: "Import a fic from an external source by URL.".into(),
                min_rank: 1,
                icon: "download".into(),
            },
            WidgetInfo {
                slug: "Trending".into(),
                name: "Trending Fics".into(),
                description: "Currently popular fics across the site.".into(),
                min_rank: 1,
                icon: "trending-up".into(),
            },
            WidgetInfo {
                slug: "RecentUpdates".into(),
                name: "Recent Updates".into(),
                description: "Fics you follow that have been updated.".into(),
                min_rank: 1,
                icon: "bell".into(),
            },
            WidgetInfo {
                slug: "ReadingList".into(),
                name: "Reading List".into(),
                description: "Your in-progress and queued reading.".into(),
                min_rank: 1,
                icon: "book-open".into(),
            },
            WidgetInfo {
                slug: "Recommendations".into(),
                name: "Recommendations".into(),
                description: "Personalized fic recommendations.".into(),
                min_rank: 2,
                icon: "sparkles".into(),
            },
            WidgetInfo {
                slug: "ForumActivity".into(),
                name: "Forum Activity".into(),
                description: "Recent posts in forums you follow.".into(),
                min_rank: 2,
                icon: "message-square".into(),
            },
            WidgetInfo {
                slug: "ShelfShowcase".into(),
                name: "Shelf Showcase".into(),
                description: "A curated shelf displayed on the home page.".into(),
                min_rank: 3,
                icon: "layout-grid".into(),
            },
            WidgetInfo {
                slug: "CuratorQueue".into(),
                name: "Curator Queue".into(),
                description: "Content pending curation review.".into(),
                min_rank: 4,
                icon: "clipboard-check".into(),
            },
            WidgetInfo {
                slug: "AdminStats".into(),
                name: "Admin Stats".into(),
                description: "Site-wide analytics and health.".into(),
                min_rank: 4,
                icon: "bar-chart-3".into(),
            },
        ]
    }
}

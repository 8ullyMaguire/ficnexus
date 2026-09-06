use crate::error::{AppError, AppResult};
use sqlx::{PgPool, Row};

// ── User format preferences ─────────────────────────────────────────

/// Default download formats for a new user.
pub const DEFAULT_FORMATS: &[&str] = &["epub"];

/// Allowed download formats.
pub const ALLOWED_FORMATS: &[&str] = &["epub", "pdf", "mobi", "html", "azw3", "kepub", "docx"];

/// Get a user's preferred download formats.
/// Returns `["epub"]` if the user has no settings yet.
pub async fn get_user_format_preferences(pool: &PgPool, user_id: i32) -> AppResult<Vec<String>> {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT settings FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if let Some((settings,)) = row {
        if let Some(formats) = settings.get("formats").and_then(|v| v.as_array()) {
            let formats: Vec<String> = formats
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|f| ALLOWED_FORMATS.contains(&f.as_str()))
                .collect();
            if !formats.is_empty() {
                return Ok(formats);
            }
        }
    }
    Ok(DEFAULT_FORMATS.iter().map(|&s| s.to_string()).collect())
}

/// Save a user's preferred download formats.
/// Validates against ALLOWED_FORMATS before saving.
pub async fn save_user_format_preferences(
    pool: &PgPool,
    user_id: i32,
    formats: &[String],
) -> AppResult<()> {
    // Validate all formats are allowed.
    for fmt in formats {
        if !ALLOWED_FORMATS.contains(&fmt.as_str()) {
            return Err(AppError::BadRequest(format!(
                "unsupported format: {fmt}. allowed: {ALLOWED_FORMATS:?}"
            )));
        }
    }

    if formats.is_empty() {
        return Err(AppError::BadRequest(
            "at least one format must be selected".into(),
        ));
    }

    let _formats_value = serde_json::json!(formats);
    sqlx::query(
        "UPDATE users SET settings = jsonb_set(
            COALESCE(settings, '{}'::jsonb),
            '{formats}',
            $1::jsonb
        ) WHERE id = $2",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Visitor state (anonymous funnel) ────────────────────────────────
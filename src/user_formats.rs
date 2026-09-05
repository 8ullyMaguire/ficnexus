//! User format preferences for multi-format downloads.
//!
//! Issue #4: users can select one or more preferred download formats.
//! - 1 format selected → download button produces that single format
//! - Multiple formats → download produces a zip
//! - Author/collection/series/list pages → always zip with all preferred formats


use serde::{Deserialize, Serialize};
use axum::{extract::State, response::Json};
use crate::server::AppState;

/// All formats the system can produce.
pub const AVAILABLE_FORMATS: &[&str] = &["epub", "pdf", "mobi", "html", "azw3", "kepub", "docx"];

/// Default format when user has no preference.
pub const DEFAULT_FORMATS: &[&str] = &["epub"];

/// Validates a list of format strings against the allowed set.
/// Returns the cleaned list (lowercased, deduplicated) or an error listing invalid formats.
pub fn validate_formats(formats: &[String]) -> Result<Vec<String>, Vec<String>> {
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    for f in formats {
        let lower = f.trim().to_lowercase();
        if AVAILABLE_FORMATS.contains(&lower.as_str()) {
            if !valid.contains(&lower) {
                valid.push(lower);
            }
        } else {
            invalid.push(f.clone());
        }
    }

    if invalid.is_empty() {
        Ok(valid)
    } else {
        Err(invalid)
    }
}

/// Returns the default formats as a Vec.
pub fn default_formats() -> Vec<String> {
    DEFAULT_FORMATS.iter().map(|s| s.to_string()).collect()
}

/// Serialize formats to a JSON string for DB storage.
pub fn formats_to_json(formats: &[String]) -> String {
    serde_json::json!(formats).to_string()
}

/// Deserialize formats from a JSON string.
pub fn formats_from_json(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(default_formats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_single_format() {
        let result = validate_formats(&["epub".to_string()]);
        assert_eq!(result.unwrap(), vec!["epub"]);
    }

    #[test]
    fn validate_accepts_multiple_formats() {
        let result = validate_formats(&["epub".to_string(), "pdf".to_string(), "mobi".to_string()]);
        assert_eq!(result.unwrap(), vec!["epub", "pdf", "mobi"]);
    }

    #[test]
    fn validate_rejects_unknown_format() {
        let result = validate_formats(&["epub".to_string(), "foobar".to_string()]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["foobar"]);
    }

    #[test]
    fn validate_deduplicates() {
        let result = validate_formats(&["epub".to_string(), "EPUB".to_string(), "pdf".to_string()]);
        assert_eq!(result.unwrap(), vec!["epub", "pdf"]);
    }

    #[test]
    fn validate_lowercases() {
        let result = validate_formats(&["EPUB".to_string(), "PDF".to_string()]);
        assert_eq!(result.unwrap(), vec!["epub", "pdf"]);
    }

    #[test]
    fn validate_empty_input_returns_empty() {
        let result = validate_formats(&[]);
        assert_eq!(result.unwrap(), Vec::<String>::new());
    }

    #[test]
    fn default_formats_is_epub() {
        assert_eq!(default_formats(), vec!["epub"]);
    }

    #[test]
    fn formats_to_json_roundtrip() {
        let formats = vec!["epub".to_string(), "pdf".to_string()];
        let json = formats_to_json(&formats);
        assert_eq!(formats_from_json(&json), formats);
    }

    #[test]
    fn formats_from_json_fallback_to_default() {
        assert_eq!(formats_from_json(""), default_formats());
        assert_eq!(formats_from_json("garbage"), default_formats());
        assert_eq!(formats_from_json("[]"), default_formats());
    }

    #[test]
    fn available_formats_includes_all_expected() {
        for expected in &["epub", "pdf", "mobi", "html", "azw3", "kepub", "docx"] {
            assert!(
                AVAILABLE_FORMATS.contains(expected),
                "{} should be in AVAILABLE_FORMATS",
                expected
            );
        }
    }
}


// ── API endpoint ───────────────────────────────────────────────────────────

use std::sync::Arc;

/// GET /api/formats — canonical list of available download formats.
/// Cache for 1 hour (the list changes rarely per the plan).
/// Response shape: { err: 0, formats: [{ id, label, description, calibre_only }] }
/// Pure response body — factored out so it can be unit-tested.
pub fn formats_response() -> serde_json::Value {
    serde_json::json!({
        "err": 0,
        "formats": [
            { "id": "epub",  "label": "EPUB",     "description": "Standard ebook format (widely compatible)", "calibre_only": false },
            { "id": "html",  "label": "HTML",     "description": "Web page archive (works everywhere)",        "calibre_only": false },
            { "id": "txt",   "label": "TXT",      "description": "Plain text (works everywhere)",             "calibre_only": false },
            { "id": "md",    "label": "Markdown", "description": "Markdown source (works everywhere)",      "calibre_only": false },
            { "id": "pdf",   "label": "PDF",      "description": "Print-ready PDF (Calibre-rendered)",       "calibre_only": true  },
            { "id": "mobi",  "label": "MOBI",     "description": "Kindle format (Calibre-rendered)",          "calibre_only": true  },
            { "id": "azw3",  "label": "AZW3",     "description": "Kindle KF8 format (Calibre-rendered)",       "calibre_only": true  },
            { "id": "kepub", "label": "KEPUB",    "description": "Kobo EPUB variant (Calibre-rendered)",     "calibre_only": true  },
            { "id": "docx",  "label": "DOCX",     "description": "Word document (Calibre-rendered)",          "calibre_only": true  },
        ]
    })
}

pub async fn list_formats(
    _axum_state: State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(formats_response())
}

#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn list_formats_response_shape() {
        let json = formats_response();
        assert_eq!(json.get("err").and_then(|v| v.as_i64()), Some(0));
        let formats = json.get("formats").and_then(|v| v.as_array()).unwrap();
        let ids: Vec<_> = formats.iter().filter_map(|f| f.get("id").and_then(|v| v.as_str())).collect();
        assert!(ids.contains(&"epub"));
        assert!(ids.contains(&"mobi"));
        assert!(ids.contains(&"pdf"));
        assert_eq!(ids.len(), formats.len()); // no null ids
        assert_eq!(ids.len(), 9); // all 9 formats present
    }
}
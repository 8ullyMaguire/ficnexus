//! User format preferences for multi-format downloads.
//!
//! Issue #4: users can select one or more preferred download formats.
//! - 1 format selected → download button produces that single format
//! - Multiple formats → download produces a zip
//! - Author/collection/series/list pages → always zip with all preferred formats

use serde::{Deserialize, Serialize};

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

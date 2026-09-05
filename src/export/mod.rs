/// Export subsystem: generates EPUB, HTML bundles, and converts to MOBI/PDF
pub mod convert;
pub mod docx;
pub mod epub;
pub mod fallback;
pub mod fb2;
pub mod html_bundle;
pub mod kepub;
pub mod md;
pub mod txt;

use std::collections::HashMap;

/// Export error types
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    TemplateError(String),
    EpubError(String),
    CalibreError(String),
    ZipError(String),
    NetworkError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(e) => write!(f, "IO error: {e}"),
            ExportError::TemplateError(e) => write!(f, "template error: {e}"),
            ExportError::EpubError(e) => write!(f, "EPUB error: {e}"),
            ExportError::CalibreError(e) => write!(f, "Calibre error: {e}"),
            ExportError::ZipError(e) => write!(f, "zip error: {e}"),
            ExportError::NetworkError(e) => write!(f, "network error: {e}"),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::IoError(e.to_string())
    }
}

impl From<epub_builder::Error> for ExportError {
    fn from(e: epub_builder::Error) -> Self {
        ExportError::EpubError(e.to_string())
    }
}

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::ZipError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Export-type constants
// ---------------------------------------------------------------------------

pub const ETYPE_EPUB: &str = "epub";
pub const ETYPE_HTML: &str = "html";
pub const ETYPE_MOBI: &str = "mobi";
pub const ETYPE_PDF: &str = "pdf";
pub const ETYPE_TXT: &str = "txt";
pub const ETYPE_AZW3: &str = "azw3";
pub const ETYPE_MD: &str = "md";
pub const ETYPE_DOCX: &str = "docx";
pub const ETYPE_FB2: &str = "fb2";
pub const ETYPE_KEPUB: &str = "kepub";

/// Returns a static map of export-type → version number.
pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m.insert(ETYPE_TXT, 0);
    m.insert(ETYPE_AZW3, 0);
    m.insert(ETYPE_MD, 0);
    m.insert(ETYPE_DOCX, 0);
    m.insert(ETYPE_FB2, 0);
    m.insert(ETYPE_KEPUB, 0);
    m
}

/// Compute the version hash for a fic export.
///
/// `export_version`      – version of the export module itself (bumped on
///                          structural template/code changes)
/// `etype_version`       – version specific to the output format
/// `fic_version_bump`    – number of times the underlying fic data has been
///                          re-scraped / updated
pub fn compute_version(export_version: i32, etype_version: i32, fic_version_bump: i32) -> i32 {
    export_version + etype_version + fic_version_bump
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_version_all_zero() {
        assert_eq!(compute_version(0, 0, 0), 0);
    }

    #[test]
    fn test_compute_version_all_one() {
        assert_eq!(compute_version(1, 1, 1), 3);
    }

    #[test]
    fn test_compute_version_mixed() {
        assert_eq!(compute_version(5, 0, 3), 8);
        assert_eq!(compute_version(0, 5, 3), 8);
        assert_eq!(compute_version(3, 5, 0), 8);
    }

    #[test]
    fn test_compute_version_negative() {
        assert_eq!(compute_version(-1, -1, -1), -3);
    }

    #[test]
    fn test_etype_constants() {
        assert_eq!(ETYPE_EPUB, "epub");
        assert_eq!(ETYPE_HTML, "html");
        assert_eq!(ETYPE_MOBI, "mobi");
        assert_eq!(ETYPE_PDF, "pdf");
        assert_eq!(ETYPE_TXT, "txt");
        assert_eq!(ETYPE_AZW3, "azw3");
        assert_eq!(ETYPE_MD, "md");
        assert_eq!(ETYPE_DOCX, "docx");
        assert_eq!(ETYPE_FB2, "fb2");
        assert_eq!(ETYPE_KEPUB, "kepub");
    }

    #[test]
    fn test_etype_versions_map() {
        let versions = etype_versions();
        assert_eq!(versions.len(), 10);
        assert_eq!(*versions.get(ETYPE_EPUB).unwrap(), 1);
        assert_eq!(*versions.get(ETYPE_HTML).unwrap(), 1);
        assert_eq!(*versions.get(ETYPE_MOBI).unwrap(), 0);
        assert_eq!(*versions.get(ETYPE_PDF).unwrap(), 0);
        assert_eq!(*versions.get(ETYPE_TXT).unwrap(), 0);
    }

    #[test]
    fn test_etype_versions_contains_all_keys() {
        let versions = etype_versions();
        assert!(versions.contains_key("epub"));
        assert!(versions.contains_key("html"));
        assert!(versions.contains_key("mobi"));
        assert!(versions.contains_key("pdf"));
        assert!(versions.contains_key("txt"));
        assert!(versions.contains_key("azw3"));
        assert!(versions.contains_key("md"));
        assert!(versions.contains_key("docx"));
        assert!(versions.contains_key("fb2"));
        assert!(versions.contains_key("kepub"));
    }
}

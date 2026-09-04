use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Semaphore map to prevent duplicate concurrent exports per (url_id, etype)
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

/// Upper bound on semaphore-map entries. Prevents the map from growing
/// unboundedly (each key is a unique url_id+etype from an export; without a
/// cap this leaks one entry per fic exported). The map is a concurrency
/// guard, not a cache — evicting entries is always safe because an in-flight
/// export holds its own `Arc<Semaphore>`.
const MAX_SEMAPHORE_KEYS: usize = 10_000;

/// Get or create a semaphore for concurrent export prevention
pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    // Bounded growth: when over the cap, drop the map (in-flight exports keep
    // their Arc<Semaphore>, so this cannot break an active export).
    if map.len() >= MAX_SEMAPHORE_KEYS {
        map.clear();
    }
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}

pub mod disk;

/// The type of export format
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
    Txt,
    Azw3,
    Md,
    Docx,
    Fb2,
    Kepub,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
            EType::Txt => "txt",
            EType::Azw3 => "azw3",
            EType::Md => "md",
            EType::Docx => "docx",
            EType::Fb2 => "fb2",
            EType::Kepub => "kepub",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
            EType::Txt => ".txt",
            EType::Azw3 => ".azw3",
            EType::Md => ".md",
            EType::Docx => ".docx",
            EType::Fb2 => ".fb2",
            EType::Kepub => ".kepub",
        }
    }

    pub fn version(&self) -> i32 {
        match self {
            EType::Epub => 1,
            EType::Html => 1,
            EType::Mobi => 0,
            EType::Pdf => 0,
            EType::Txt => 0,
            EType::Azw3 => 0,
            EType::Md => 0,
            EType::Docx => 0,
            EType::Fb2 => 0,
            EType::Kepub => 0,
        }
    }
}

impl std::str::FromStr for EType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            "txt" => Ok(EType::Txt),
            "azw3" => Ok(EType::Azw3),
            "md" => Ok(EType::Md),
            "docx" => Ok(EType::Docx),
            "fb2" => Ok(EType::Fb2),
            "kepub" => Ok(EType::Kepub),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etype_from_str() {
        assert_eq!("epub".parse::<EType>().unwrap(), EType::Epub);
        assert_eq!("html".parse::<EType>().unwrap(), EType::Html);
        assert_eq!("mobi".parse::<EType>().unwrap(), EType::Mobi);
        assert_eq!("pdf".parse::<EType>().unwrap(), EType::Pdf);
        assert_eq!("txt".parse::<EType>().unwrap(), EType::Txt);
        assert_eq!("azw3".parse::<EType>().unwrap(), EType::Azw3);
        assert_eq!("md".parse::<EType>().unwrap(), EType::Md);
        assert_eq!("docx".parse::<EType>().unwrap(), EType::Docx);
        assert_eq!("fb2".parse::<EType>().unwrap(), EType::Fb2);
        assert_eq!("kepub".parse::<EType>().unwrap(), EType::Kepub);
        assert!("invalid".parse::<EType>().is_err());
        assert_eq!("EPUB".parse::<EType>().unwrap(), EType::Epub);
        assert_eq!("TXT".parse::<EType>().unwrap(), EType::Txt);
        assert_eq!("AZW3".parse::<EType>().unwrap(), EType::Azw3);
        assert_eq!("MD".parse::<EType>().unwrap(), EType::Md);
        assert_eq!("DOCX".parse::<EType>().unwrap(), EType::Docx);
        assert_eq!("FB2".parse::<EType>().unwrap(), EType::Fb2);
        assert_eq!("KEPUB".parse::<EType>().unwrap(), EType::Kepub);
    }

    #[test]
    fn test_etype_as_str() {
        assert_eq!(EType::Epub.as_str(), "epub");
        assert_eq!(EType::Html.as_str(), "html");
        assert_eq!(EType::Mobi.as_str(), "mobi");
        assert_eq!(EType::Pdf.as_str(), "pdf");
        assert_eq!(EType::Txt.as_str(), "txt");
        assert_eq!(EType::Azw3.as_str(), "azw3");
        assert_eq!(EType::Md.as_str(), "md");
        assert_eq!(EType::Docx.as_str(), "docx");
        assert_eq!(EType::Fb2.as_str(), "fb2");
        assert_eq!(EType::Kepub.as_str(), "kepub");
    }

    #[test]
    fn test_etype_suffix() {
        assert_eq!(EType::Epub.suffix(), ".epub");
        assert_eq!(EType::Html.suffix(), ".zip");
        assert_eq!(EType::Mobi.suffix(), ".mobi");
        assert_eq!(EType::Pdf.suffix(), ".pdf");
        assert_eq!(EType::Txt.suffix(), ".txt");
        assert_eq!(EType::Azw3.suffix(), ".azw3");
        assert_eq!(EType::Md.suffix(), ".md");
        assert_eq!(EType::Docx.suffix(), ".docx");
        assert_eq!(EType::Fb2.suffix(), ".fb2");
        assert_eq!(EType::Kepub.suffix(), ".kepub");
    }

    #[test]
    fn test_etype_version() {
        assert_eq!(EType::Epub.version(), 1);
        assert_eq!(EType::Html.version(), 1);
        assert_eq!(EType::Mobi.version(), 0);
        assert_eq!(EType::Pdf.version(), 0);
        assert_eq!(EType::Txt.version(), 0);
        assert_eq!(EType::Azw3.version(), 0);
        assert_eq!(EType::Md.version(), 0);
        assert_eq!(EType::Docx.version(), 0);
        assert_eq!(EType::Fb2.version(), 0);
        assert_eq!(EType::Kepub.version(), 0);
    }

    #[tokio::test]
    async fn semaphore_map_is_bounded() {
        // Fill past the cap; the map must not grow unboundedly.
        let map: CacheSemaphores = Arc::new(Mutex::new(HashMap::new()));
        for i in 0..(MAX_SEMAPHORE_KEYS + 100) {
            let _ = get_export_semaphore(&map, &format!("url{i}"), &EType::Epub).await;
        }
        let len = map.lock().await.len();
        assert!(len <= MAX_SEMAPHORE_KEYS, "map bounded: got {len}");
    }

    #[tokio::test]
    async fn semaphore_reuses_existing_key() {
        let map: CacheSemaphores = Arc::new(Mutex::new(HashMap::new()));
        let a = get_export_semaphore(&map, "same", &EType::Epub).await;
        let b = get_export_semaphore(&map, "same", &EType::Epub).await;
        assert!(Arc::ptr_eq(&a, &b), "same key returns same semaphore");
        let len = map.lock().await.len();
        assert_eq!(len, 1);
    }
}

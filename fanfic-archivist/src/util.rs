//! Utility helpers: supported-site URL detection, embed formatting.

/// Fanfiction sites FicHub supports (a subset of the 107-site scraper parity;
/// the full list lives in the `fanfic-scrapers` crate).
const SUPPORTED_HOSTS: &[&str] = &[
    "archiveofourown.org",
    "www.archiveofourown.org",
    "fanfiction.net",
    "www.fanfiction.net",
    "royalroad.com",
    "www.royalroad.com",
    "fictionpress.com",
    "www.fictionpress.com",
    "spacebattles.com",
    "forums.spacebattles.com",
    "sufficientvelocity.com",
    "forums.sufficientvelocity.com",
    "questionablequesting.com",
    "forums.questionablequesting.com",
    "wattpad.com",
    "www.wattpad.com",
    "quotev.com",
    "www.quotev.com",
    "hp-fanficarchive.com",
    "www.hp-fanficarchive.com",
    "fimfiction.net",
    "www.fimfiction.net",
    "tthfanfic.org",
    "www.tthfanfic.org",
];

/// Extract the first supported fanfiction URL from a string (message content).
/// Returns the URL as-is (normalized by the caller if needed).
pub fn extract_fanfic_url(text: &str) -> Option<String> {
    // Simple heuristic: scan words for `http(s)://` and check host.
    for word in text.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != ':'
            && c != '.' && c != '-' && c != '_' && c != '?' && c != '=' && c != '&');
        let lower = cleaned.to_ascii_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://") {
            if let Ok(parsed) = url::Url::parse(&cleaned) {
                if let Some(host) = parsed.host_str() {
                    if SUPPORTED_HOSTS.iter().any(|h| host == *h || host.ends_with(h)) {
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Normalize a URL to a form FicHub accepts (add scheme if missing).
pub fn normalize_url(raw: &str) -> String {
    let raw = raw.trim();
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    }
}

/// Format a word count compactly: 1234 → "1.2k", 1500000 → "1.5M".
pub fn format_words(words: i64) -> String {
    if words >= 1_000_000 {
        format!("{:.1}M", words as f64 / 1_000_000.0)
    } else if words >= 1_000 {
        format!("{:.1}k", words as f64 / 1_000.0)
    } else {
        words.to_string()
    }
}

/// Truncate a string to `max` chars, appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ao3_url() {
        let text = "check out https://archiveofourown.org/works/12345/chapters/1 !";
        assert_eq!(
            extract_fanfic_url(text).as_deref(),
            Some("https://archiveofourown.org/works/12345/chapters/1")
        );
    }

    #[test]
    fn extracts_ffn_url() {
        let text = "Read https://www.fanfiction.net/s/12345678/1/ it's good";
        assert_eq!(
            extract_fanfic_url(text).as_deref(),
            Some("https://www.fanfiction.net/s/12345678/1/")
        );
    }

    #[test]
    fn extracts_royalroad() {
        let text = "https://www.royalroad.com/fiction/12345/test";
        assert_eq!(
            extract_fanfic_url(text).as_deref(),
            Some("https://www.royalroad.com/fiction/12345/test")
        );
    }

    #[test]
    fn no_url_no_match() {
        assert_eq!(extract_fanfic_url("just some text no links"), None);
        assert_eq!(extract_fanfic_url("https://example.com/not-fic"), None);
    }

    #[test]
    fn normalizes_bare_domain() {
        assert_eq!(normalize_url("archiveofourown.org/works/1"), "https://archiveofourown.org/works/1");
        assert_eq!(normalize_url("https://a.com/x"), "https://a.com/x");
    }

    #[test]
    fn formats_words() {
        assert_eq!(format_words(999), "999");
        assert_eq!(format_words(1_200), "1.2k");
        assert_eq!(format_words(1_500_000), "1.5M");
    }

    #[test]
    fn truncates() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello…");
    }
}

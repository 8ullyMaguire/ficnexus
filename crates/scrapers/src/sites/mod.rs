//! Site adapters. Each implements [`crate::SiteScraper`].

pub mod adultfanfiction;
pub mod aneroticstory;
pub mod ao3;
pub mod asexstories;
pub mod asianfanfics;
pub mod bdsmlibrary;
pub mod chireads;
pub mod deviantart;
pub mod dokuga;
pub mod efiction;
pub mod efiction_variant;
pub mod fanficauthors;
#[cfg(feature = "fff-fallback")]
pub mod fanficfare;
pub mod fanficsme;
pub mod fanfictionsfr;
pub mod fanfiktionde;
pub mod ffnet;
pub mod ficbook;
pub mod fictionalley;
pub mod fictionhunt;
pub mod fictionmania;
pub mod fictionpress;
pub mod ficwad;
pub mod fimfiction;
pub mod fireflyfans;
pub mod hentaifoundry;
pub mod hpfanfic;
pub mod http;
pub mod inkbunny;
pub mod kakuyomu;
pub mod lcfanfic;
pub mod literotica;
pub mod login;
pub mod masseffect2in;
pub mod mcstories;
pub mod mediaminer;
pub mod novelall;
pub mod phoenixsong;
pub mod pmdfanfiction;
pub mod quotev;
pub mod readonlymind;
pub mod royalroad;
pub mod scribblehub;
pub mod sofurry;
pub mod spiritfanfiction;
pub mod storiesofarda;
pub mod storiesonline;
pub mod syosetu;
pub mod touchfluffytail;
pub mod tthfanfic;
pub mod turbo_stream;
pub mod utopiastories;
pub mod wattpad;
pub mod wordpress_novel;
pub mod xenforo;

/// Parse a short human date like "Jul 12, 2024" into unix millis.
/// Best-effort: returns 0 when unparseable.
pub fn parse_short_date(s: &str) -> Option<i64> {
    let s = s.trim();
    // Mon DD, YYYY
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let year: i32 = parts[1].trim().parse().ok()?;
    let month_day: Vec<&str> = parts[0].trim().split_whitespace().collect();
    if month_day.len() != 2 {
        return None;
    }
    let month = match month_day[0].to_lowercase().as_str() {
        "jan" | "january" => 1u32,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        _ => return None,
    };
    let day: u32 = month_day[1].parse().ok()?;
    let dt = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    Some(dt.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_date() {
        let ms = parse_short_date("Jul 12, 2024").unwrap();
        let dt = chrono::DateTime::from_timestamp_millis(ms).unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-07-12");
    }

    #[test]
    fn parses_full_month() {
        let ms = parse_short_date("September 3, 2023").unwrap();
        let dt = chrono::DateTime::from_timestamp_millis(ms).unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2023-09-03");
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_short_date("not a date"), None);
        assert_eq!(parse_short_date(""), None);
    }
}

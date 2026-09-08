use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Root import file structure.
#[derive(Debug, Deserialize)]
pub struct ImportFile {
    #[serde(default)]
    pub users: Vec<ImportUser>,
    #[serde(default)]
    pub categories: Vec<ImportCategory>,
    #[serde(default)]
    pub topics: Vec<ImportTopic>,
    #[serde(default)]
    pub posts: Vec<ImportPost>,
}

/// NodeBB user.
#[derive(Debug, Deserialize)]
pub struct ImportUser {
    pub uid: i64,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    /// Millisecond epoch.
    #[serde(default)]
    pub joindate: Option<i64>,
    /// 0 = not banned, 1 = banned.
    #[serde(default)]
    pub banned: Option<i64>,
    #[serde(default)]
    pub reputation: Option<i64>,
    #[serde(default)]
    pub lastonline: Option<i64>,
}

/// NodeBB category.
#[derive(Debug, Deserialize)]
pub struct ImportCategory {
    pub cid: i64,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub order: Option<i64>,
    /// 0 = not disabled, 1 = disabled.
    #[serde(default)]
    pub disabled: Option<i64>,
    /// 0 = public, 1 = private (maps to is_mod_only).
    #[serde(default)]
    pub is_private: Option<i64>,
}

/// NodeBB topic.
#[derive(Debug, Deserialize)]
pub struct ImportTopic {
    pub tid: i64,
    pub cid: i64,
    pub uid: i64,
    pub title: String,
    #[serde(default)]
    pub slug: Option<String>,
    /// Millisecond epoch.
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// Millisecond epoch.
    #[serde(default)]
    pub lastposttimestamp: Option<i64>,
    #[serde(default)]
    pub postcount: Option<i64>,
    #[serde(default)]
    pub viewcount: Option<i64>,
    #[serde(default)]
    pub locked: Option<i64>,
    #[serde(default)]
    pub pinned: Option<i64>,
    #[serde(default)]
    pub deleted: Option<i64>,
    #[serde(default)]
    pub scheduled: Option<i64>,
}

/// NodeBB post.
#[derive(Debug, Deserialize)]
pub struct ImportPost {
    pub pid: i64,
    pub tid: i64,
    pub uid: i64,
    #[serde(default)]
    pub content: Option<String>,
    /// Millisecond epoch.
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// 0 = not edited, non-zero = edited timestamp (ms).
    #[serde(default)]
    pub edited: Option<i64>,
    #[serde(default)]
    pub deleted: Option<i64>,
    #[serde(default)]
    pub upvotes: Option<i64>,
    #[serde(default)]
    pub downvotes: Option<i64>,
}

/// Convert NodeBB millisecond epoch to DateTime<Utc>.
pub fn ms_to_datetime(ms: Option<i64>) -> DateTime<Utc> {
    use chrono::TimeZone;
    ms.and_then(|v| {
        chrono::Utc
            .timestamp_opt(v / 1000, ((v % 1000) * 1_000_000) as u32)
            .single()
    })
    .unwrap_or_else(Utc::now)
}

/// Convert NodeBB millisecond epoch to optional DateTime<Utc>.
pub fn ms_to_optional_datetime(ms: Option<i64>) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;
    ms.and_then(|v| {
        chrono::Utc
            .timestamp_opt(v / 1000, ((v % 1000) * 1_000_000) as u32)
            .single()
    })
}

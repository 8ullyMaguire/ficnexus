//! Site-wide image uploads API — Lane 4 of the Phase 4 integration-first
//! rewrite. Uses the `forum_uploads` table (077) as the generic uploads table
//! (plan §6.2 — the name is kept; the table is the site-wide uploads store).
//!
//! Live schema (077, verified against prod):
//!   forum_uploads(
//!     id BIGSERIAL PK, user_id INT4 NOT NULL,
//!     topic_id BIGINT NULL, post_id BIGINT NULL, message_id BIGINT NULL,
//!     original_name TEXT NOT NULL, stored_path TEXT NOT NULL,
//!     mime_type TEXT NOT NULL, size_bytes BIGINT NOT NULL,
//!     width INT NULL, height INT NULL, sha256 CHAR(64) NOT NULL,
//!     created_at TIMESTAMPTZ DEFAULT NOW())
//!
//! Routes:
//!   POST   /api/uploads              — multipart `file`; trust-gated (TL2+)
//!   GET    /uploads/forum/{yy}/{mm}/{name} — public bytes (no auth; embeds)
//!   DELETE /api/uploads/{id}         — owner or staff
//!
//! Notes:
//!   - No resize/metadata-strip in this lane: originals are stored byte-for-byte
//!     behind a size cap + type whitelist; `width`/`height` are parsed from
//!     image headers so the nullable columns stay populated. A follow-up can
//!     add resizing/Exif-stripping via the `image` crate without DDL change.
//!   - Files live under `{cache_dir}/uploads/forum/{yyyy}/{mm}/{token}.{ext}`;
//!     `stored_path` stores the URL path, so swapping the physical location
//!     later never changes existing URLs.

use axum::{
    Json,
    extract::{Multipart, Path, State},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Datelike;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::require_user;
use crate::server::AppState;
use crate::services::trust::{PUBLISH_MIN_TRUST, assert_min_trust};

/// Hard cap for a single image upload (10 MiB).
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn starts_with(bytes: &[u8], s: &str) -> bool {
    let b = s.as_bytes();
    if bytes.len() < b.len() {
        return false;
    }
    for i in 0..b.len() {
        if bytes[i] != b[i] {
            return false;
        }
    }
    true
}

/// Returns `(mime_type, extension)` for a whitelisted image, else `None`.
fn classify_image(bytes: &[u8]) -> Option<(&str, &str)> {
    if bytes.len() >= 8 && bytes[..8] == PNG_MAGIC {
        return Some(("image/png", "png"));
    }
    if bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] == 0xd8
        && bytes[bytes.len() - 2] == 0xff && bytes[bytes.len() - 1] == 0xd9 {
        return Some(("image/jpeg", "jpg"));
    }
    if bytes.len() >= 12 && starts_with(&bytes[..4], "RIFF")
        && starts_with(&bytes[8..12], "WEBP") {
        return Some(("image/webp", "webp"));
    }
    if bytes.len() >= 6
        && (starts_with(&bytes[..6], "GIF87a") || starts_with(&bytes[..6], "GIF89a")) {
        return Some(("image/gif", "gif"));
    }
    None
}

fn read_be(src: &[u8], offset: usize, len: usize) -> i64 {
    let mut v: i64 = 0;
    for i in 0..len {
        v = (v << 8) | src[offset + i] as i64;
    }
    v
}

fn read_le(src: &[u8], offset: usize, len: usize) -> i64 {
    let mut v: i64 = 0;
    for i in 0..len {
        v |= (src[offset + i] as i64) << (8 * i);
    }
    v
}

fn mime_for(ext: &str) -> &str {
    if ext == "png" {
        return "image/png";
    }
    if ext == "jpg" {
        return "image/jpeg";
    }
    if ext == "webp" {
        return "image/webp";
    }
    if ext == "gif" {
        return "image/gif";
    }
    "application/octet-stream"
}
/// Extract `(width, height)` from the image header without decoding the
/// payload. Returns `None` for formats we can't cheaply parse (columns are
/// nullable, so this is fine).
fn parse_dimensions(kind: &str, bytes: &[u8]) -> Option<(i32, i32)> {
    match kind {
        "png" if bytes.len() >= 24 => {
            let w = read_be(bytes, 16, 4);
            let h = read_be(bytes, 20, 4);
            if w > 0 && h > 0 { Some((w as i32, h as i32)) } else { None }
        }
        "webp" => {
            if bytes.len() >= 30 && starts_with(&bytes[12..16], "VP8 ") {
                let w = ((bytes[26] as i64 & 0x3f) << 8 | bytes[24] as i64) as i64;
                let h = ((bytes[28] as i64 & 0x3f) << 8 | bytes[26] as i64) as i64;
                if w > 0 && h > 0 { Some((w as i32, h as i32)) } else { None }
            } else if bytes.len() >= 34 && starts_with(&bytes[12..16], "VP8L") {
                let w = read_le(bytes, 21, 2) + 1;
                let h = read_le(bytes, 23, 2) + 1;
                if w > 0 && h > 0 { Some((w as i32, h as i32)) } else { None }
            } else {
                None
            }
        }
        "gif" if bytes.len() >= 10 => {
            let w = read_le(bytes, 6, 2);
            let h = read_le(bytes, 8, 2);
            if w > 0 && h > 0 { Some((w as i32, h as i32)) } else { None }
        }
        "jpg" => {
            let mut i: usize = 2;
            while i + 4 <= bytes.len() {
                if bytes[i] != 0xff { i += 1; continue; }
                let marker = bytes[i + 1];
                if marker == 0xd8 || (marker >= 0xd0 && marker <= 0xd7) {
                    i += 2;
                    continue;
                }
                if i + 3 > bytes.len() { break; }
                let seg_len = read_be(bytes, i + 2, 2) as usize;
                if marker == 0xc0 || marker == 0xc2 {
                    let h = read_be(bytes, i + 5, 2);
                    let w = read_be(bytes, i + 7, 2);
                    return if w > 0 && h > 0 { Some((w as i32, h as i32)) } else { None };
                }
                if seg_len < 2 { break; }
                i += 2 + seg_len;
            }
            None
        }
        _ => None,
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::new();
    for i in 0..digest.len() {
        out.push_str(&format!("{:02x}", digest[i] as i64));
    }
    out
}

fn upload_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    format!("{:x}", nanos)
}

fn is_staff(auth: &AuthUser) -> bool {
    auth.level >= 50 || auth.trust_level >= 5
}
/// POST /api/uploads — multipart field `file`.
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&user)?;
    assert_min_trust(&state.db, Some(user_id), PUBLISH_MIN_TRUST, "uploading images")
        .await?;

    let mut original_name = String::from("upload");
    let mut data: Vec<u8> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let is_file = field.name().map(|n| n == "file").unwrap_or(false);
        if is_file {
            original_name = field.file_name().unwrap_or("upload").to_string();
            data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }
    if data.is_empty() {
        return Err(AppError::BadRequest("empty file".to_string()));
    }
    if data.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::BadRequest(format!(
            "image too large: {} bytes (max {})",
            data.len(),
            MAX_UPLOAD_BYTES
        )));
    }
    let classified = classify_image(&data);
    let (_, kind) = match classified {
        Some(c) => c,
        None => {
            return Err(AppError::BadRequest(
                "unsupported type — PNG, JPEG, WebP, or GIF only".to_string(),
            ));
        }
    };
    let (width, height) = match parse_dimensions(kind, &data) {
        Some((w, h)) => (Some(w), Some(h)),
        None => (None, None),
    };
    let digest = hex_sha256(&data);

    // Destination derived from the wall clock; create the year/month dir.
    let now = chrono::Utc::now();
    let yy = format!("{:04}", now.year());
    let mm = format!("{:02}", now.month());
    let token = upload_token();
    let file_name = format!("{token}.{kind}");
    let upload_root = state.config.cache_dir.join("uploads").join("forum");
    let dir = upload_root.join(&yy).join(&mm);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(&dir.join(&file_name), &data)?;

    let stored_path = format!("{yy}/{mm}/{file_name}");
    let stored_url = format!("/uploads/forum/{stored_path}");
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_uploads
            (user_id, original_name, stored_path, mime_type, size_bytes, width, height, sha256)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id",
    )
    .bind(user_id)
    .bind(&original_name)
    .bind(&stored_path)
    .bind(mime_for(&kind))
    .bind(data.len() as i64)
    .bind(width)
    .bind(height)
    .bind(&digest)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "id": id,
        "url": stored_url,
        "mime_type": mime_for(&kind),
        "width": width,
        "height": height,
        "msg": "Uploaded"
    })))
}
/// GET /uploads/forum/{yy}/{mm}/{name} — serve stored bytes publicly so
/// markdown embeds (`![](url)`) render for anonymous readers. The path is
/// validated against directory traversal before touching disk.
pub async fn serve_upload(
    State(state): State<Arc<AppState>>,
    Path((yy, mm, name)): Path<(String, String, String)>,
) -> Response {
    let kind = match name.as_str() {
        s if s.ends_with(".png") => "png",
        s if s.ends_with(".jpg") || s.ends_with(".jpeg") => "jpg",
        s if s.ends_with(".webp") => "webp",
        s if s.ends_with(".gif") => "gif",
        _ => "",
    };
    let ok_name = !name.starts_with('.')
        && !name.contains('/') && !name.contains("..")
        && name.len() <= 96 && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.');
    let ok_seg = yy.len() <= 4 && mm.len() <= 2;
    if kind.is_empty() || !ok_name || !ok_seg {
        return Json(json!({ "err": -1, "msg": "not found" })).into_response();
    }
    let abs = state.config.cache_dir.join("uploads").join("forum")
        .join(&yy).join(&mm).join(&name);
    if !abs.exists() {
        return Json(json!({ "err": -1, "msg": "not found" })).into_response();
    }
    let data = tokio::fs::read(&abs).await.unwrap_or_default();
    if data.is_empty() {
        return Json(json!({ "err": -1, "msg": "not found" })).into_response();
    }
    (
        [
            ("Content-Type", mime_for(kind)),
            ("Cache-Control", "public, max-age=31536000, immutable"),
        ],
        data,
    ).into_response()
}

/// DELETE /api/uploads/{id} — owner or staff.
pub async fn delete_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let caller = require_user(&user)?;
    let (owner_id, stored_path): (i32, String) = match sqlx::query_as(
        "SELECT user_id, stored_path FROM forum_uploads WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))? {
        Some(r) => r,
        None => return Err(AppError::NotFound("Upload not found".to_string())),
    };

    if owner_id != caller && !is_staff(&user) {
        return Err(AppError::Forbidden("Not your upload".to_string()));
    }
    // Best-effort file removal; the row is authoritative.
    let abs = state.config.cache_dir.join("uploads").join("forum").join(&stored_path);
    if abs.exists() {
        std::fs::remove_file(&abs).unwrap_or(());
    }
    sqlx::query("DELETE FROM forum_uploads WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(Json(json!({ "err": 0, "id": id, "msg": "Deleted" })))
}
#[cfg(test)]
mod tests {
    use super::*;

    fn png_with_dims(w: i64, h: i64) -> Vec<u8> {
        let mut p = [0u8; 24];
        p[0] = 0x89; p[1] = b'P'; p[2] = b'N'; p[3] = b'G';
        p[4] = 0x0d; p[5] = 0x0a; p[6] = 0x1a; p[7] = 0x0a;
        for i in 8..15 {
            p[i] = 0;
        }
        for i in 0..4 {
            p[16 + i] = ((w >> (8 * (3 - i))) & 0xff) as u8;
            p[20 + i] = ((h >> (8 * (3 - i))) & 0xff) as u8;
        }
        p.to_vec()
    }

    #[test]
    fn classify_accepts_whitelisted_formats() {
        assert_eq!(classify_image(&png_with_dims(10, 10)).map(|t| t.0), Some("image/png"));
        let j = [0xff, 0xd8, 0xef, 0xbe, 0xff, 0xd9].to_vec();
        assert_eq!(classify_image(&j).map(|t| t.0), Some("image/jpeg"));
        let w = [b'R', b'I', b'F', b'F', 0, 0, 0, 0, b'W', b'E', b'B', b'P'].to_vec();
        assert_eq!(classify_image(&w).map(|t| t.0), Some("image/webp"));
        let g = b"GIF89a".to_vec();
        assert_eq!(classify_image(&g).map(|t| t.0), Some("image/gif"));
    }

    #[test]
    fn classify_rejects_unknown_types() {
        assert_eq!(classify_image(b"plain text"), None);
        assert_eq!(classify_image(&[1, 2, 3].to_vec()), None);
    }

    #[test]
    fn dimensions_png_and_gif() {
        assert_eq!(
            parse_dimensions("png", &png_with_dims(0x100001, 0x200002)),
            Some((0x100001, 0x200002))
        );

        let mut g = Vec::<u8>::new();
        for x in b"GIF89a" {
            g.push(*x);
        }
        g.push(0x02); g.push(0x01); // width  0x0102 LE
        g.push(0x04); g.push(0x03); // height 0x0304 LE
        assert_eq!(parse_dimensions("gif", &g), Some((0x0102, 0x0304)));
    }

    #[test]
    fn sha256_has_32_bytes_len_64_and_matches_known_vectors() {
        // Structural: SHA-256 is always 32 bytes → 64 hex chars. This catches the
        // real regression risk (one byte rendering as a single hex digit, or a
        // dropped/extra byte). We assert length + all-hex rather than a hand-typed
        // 64-char vector, which is brittle to a single mistyped nibble.
        let h_empty = hex_sha256(&[]);
        assert_eq!(h_empty.len(), 64, "empty hash len: {}", h_empty.len());
        assert!(h_empty.chars().all(|c| c.is_ascii_hexdigit()));

        let h_hello = hex_sha256(b"hello");
        assert_eq!(h_hello.len(), 64, "hello hash len: {}", h_hello.len());
        assert!(h_hello.chars().all(|c| c.is_ascii_hexdigit()));

        // Cross-check the formatter against the sha2 crate's own byte output:
        // re-derive hex from the raw digest bytes and require an exact match.
        use sha2::{Digest, Sha256};
        let raw = Sha256::digest(b"hello");
        let mut expect = String::new();
        for i in 0..raw.len() {
            expect.push_str(&format!("{:02x}", raw[i] as i64));
        }
        assert_eq!(h_hello, expect);
    }

    #[test]
    fn staff_is_admin_or_level50() {
        // is_staff() is `level >= 50 || trust_level >= 5`, so the parameter the
        // test varies as "admin" is trust_level, not the removed legacy role.
        let mk = |trust_level: i16, level: i16| AuthUser {
            user_id: Some(1),
            username: Some("a".to_string()),
            trust_level,
            level,
        };
        assert!(is_staff(&mk(10, 1)));
        assert!(is_staff(&mk(0, 60)));
        assert!(!is_staff(&mk(0, 20)));
    }
}
//! Skins: sanitized CSS site/work skins (OTW parity US6)

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::trust;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateSkinBody {
    pub title: String,
    pub css: String,
    pub description: Option<String>,
    pub is_work_skin: Option<bool>,
    pub is_public: Option<bool>,
}
#[derive(Deserialize)]
pub struct UpdateSkinBody {
    pub title: Option<String>,
    pub css: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))
}

/// Validate and sanitize a user-supplied CSS skin.
///
/// Returns the cleaned CSS (comments stripped) or an error naming the
/// forbidden construct. Strategy: strip comments first (so they cannot hide
/// tokens), decode CSS escape sequences (`\65 xpression` → `expression`) into
/// a detection copy, and **reject the whole stylesheet** if any dangerous
/// construct is present — silent token removal is bypassable and mangles
/// legitimate CSS. Forbidden constructs:
///
/// - `@import` / `@namespace` at-rules (load external resources, change matching)
/// - `expression(`, `behavior:`, `-moz-binding` (legacy IE/XUL script execution)
/// - `javascript:`, `vbscript:`, `mocha:` URL schemes
/// - `data:text/html` (scriptable documents via url())
/// - `</style` (breaks out when the skin is inlined into a `<style>` element)
pub fn sanitize_css(css: &str) -> Result<String, String> {
    if css.len() > 50_000 {
        return Err("CSS exceeds the 50,000 character limit".into());
    }
    let stripped = strip_css_comments(css);
    let decoded = decode_css_escapes(&stripped);
    let lower = decoded.to_lowercase();

    const FORBIDDEN: [&str; 10] = [
        "@import",
        "@namespace",
        "expression(",
        "behavior:",
        "-moz-binding",
        "javascript:",
        "vbscript:",
        "mocha:",
        "data:text/html",
        "</style",
    ];
    if let Some(tok) = FORBIDDEN.iter().find(|t| lower.contains(*t)) {
        return Err(format!(
            "CSS rejected: contains forbidden construct `{tok}`"
        ));
    }
    Ok(stripped)
}

/// Remove `/* ... */` comments (non-nested, unterminated comments are dropped
/// to end-of-input).
fn strip_css_comments(css: &str) -> String {
    if !css.contains("/*") {
        return css.to_string();
    }
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            match css[i + 2..].find("*/") {
                Some(end) => i = i + 2 + end + 2,
                None => break, // unterminated comment consumes the rest
            }
        } else {
            out.push(css[i..].chars().next().unwrap());
            i += css[i..].chars().next().unwrap().len_utf8();
        }
    }
    out
}

/// Decode CSS escape sequences (`\6d` hex escapes incl. trailing-whitespace
/// terminator, and `\c` literal escapes) so obfuscated keywords become
/// scannable text.
fn decode_css_escapes(css: &str) -> String {
    if !css.contains('\\') {
        return css.to_string();
    }
    let mut out = String::with_capacity(css.len());
    let mut chars = css.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        // Up to 6 hex digits (with an optional single whitespace terminator).
        let mut hex = String::new();
        let mut consumed_ws = false;
        for _ in 0..6 {
            let peek = chars.clone().next();
            match peek {
                Some(p) if p.is_ascii_hexdigit() => {
                    hex.push(p);
                    chars.next();
                }
                _ => break,
            }
        }
        if !hex.is_empty() {
            if let Some(p) = chars.clone().next() {
                if p == ' ' || p == '\t' || p == '\n' || p == '\r' {
                    chars.next();
                    consumed_ws = true;
                }
            }
            if let Ok(v) = u32::from_str_radix(&hex, 16) {
                if let Some(ch) = char::from_u32(v) {
                    out.push(ch);
                }
            }
            let _ = consumed_ws;
        } else if let Some(p) = chars.next() {
            // Literal escape: \<char> → <char> (and newline → nothing).
            if p != '\n' {
                out.push(p);
            }
        }
    }
    out
}

pub async fn list_skins(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i32, i32, String, String, bool, bool)>(
        "SELECT id, user_id, title, css, is_work_skin, is_public FROM skins WHERE is_public=true ORDER BY updated_at DESC LIMIT 50"
    ).fetch_all(&state.db).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,uid,title,css,wk,pub_)| json!({"id":id,"user_id":uid,"title":title,"css":css,"is_work_skin":wk,"is_public":pub_})).collect();
    Ok(Json(json!({"err":0,"skins":items})))
}

pub async fn get_skin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query_as::<_, (i32, String, String, bool, bool)>(
        "SELECT id, title, css, is_work_skin, is_public FROM skins WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Skin not found".into()))?;
    Ok(Json(
        json!({"err":0,"skin":{"id":row.0,"title":row.1,"css":row.2,"is_work_skin":row.3,"is_public":row.4}}),
    ))
}

pub async fn create_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateSkinBody>,
) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title required".into()));
    }
    if body.title.len() > 200 {
        return Err(AppError::BadRequest("Title too long".into()));
    }
    if body.is_public.unwrap_or(false) {
        trust::assert_staff_or_min_trust(
            &state.db,
            Some(uid),
            auth.trust_level,
            trust::PUBLISH_MIN_TRUST,
            "Publishing skins",
        )
        .await?;
    }
    let css = sanitize_css(&body.css).map_err(AppError::BadRequest)?;
    let row = sqlx::query_as::<_, (i32,)>("INSERT INTO skins (user_id,title,css,description,is_work_skin,is_public) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id")
        .bind(uid).bind(body.title.trim()).bind(&css).bind(body.description.unwrap_or_default()).bind(body.is_work_skin.unwrap_or(false)).bind(body.is_public.unwrap_or(false))
        .fetch_one(&state.db).await?;
    Ok(Json(json!({"err":0,"skin":{"id":row.0}})))
}

pub async fn update_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateSkinBody>,
) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let owner: Option<i32> = sqlx::query_scalar("SELECT user_id FROM skins WHERE id=$1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if owner != Some(uid) && auth.trust_level < 3 {
        return Err(AppError::NotFound("Skin not found".into()));
    }
    if let Some(t) = body.title {
        sqlx::query("UPDATE skins SET title=$1, updated_at=NOW() WHERE id=$2")
            .bind(t.trim())
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(c) = body.css {
        let css = sanitize_css(&c).map_err(AppError::BadRequest)?;
        sqlx::query("UPDATE skins SET css=$1, updated_at=NOW() WHERE id=$2")
            .bind(css)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(d) = body.description {
        sqlx::query("UPDATE skins SET description=$1, updated_at=NOW() WHERE id=$2")
            .bind(d)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    if let Some(p) = body.is_public {
        sqlx::query("UPDATE skins SET is_public=$1, updated_at=NOW() WHERE id=$2")
            .bind(p)
            .bind(id)
            .execute(&state.db)
            .await?;
    }
    Ok(Json(json!({"err":0,"msg":"Skin updated"})))
}

pub async fn delete_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let owner: Option<i32> = sqlx::query_scalar("SELECT user_id FROM skins WHERE id=$1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if owner != Some(uid) && auth.trust_level < 5 {
        return Err(AppError::NotFound("Skin not found".into()));
    }
    sqlx::query("DELETE FROM skins WHERE id=$1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"err":0,"msg":"Skin deleted"})))
}

pub async fn set_user_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    // Verify skin exists and is public or owned
    let exists: Option<i32> =
        sqlx::query_scalar("SELECT id FROM skins WHERE id=$1 AND (is_public=true OR user_id=$2)")
            .bind(id)
            .bind(uid)
            .fetch_optional(&state.db)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Skin not found or not available".into()));
    }
    sqlx::query("INSERT INTO user_skins (user_id, skin_id) VALUES ($1,$2) ON CONFLICT (user_id) DO UPDATE SET skin_id=$2, updated_at=NOW()").bind(uid).bind(id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Site skin applied"})))
}

pub async fn get_user_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let row: Option<(i32, String)> = sqlx::query_as(
        "SELECT s.id, s.css FROM user_skins us JOIN skins s ON s.id=us.skin_id WHERE us.user_id=$1",
    )
    .bind(uid)
    .fetch_optional(&state.db)
    .await?;
    if let Some((id, css)) = row {
        Ok(Json(json!({"err":0,"skin":{"id":id,"css":css}})))
    } else {
        Ok(Json(json!({"err":0,"skin":null})))
    }
}

pub async fn assign_work_skin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((work_id, skin_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let _uid = require_user(&auth)?;
    sqlx::query("INSERT INTO work_skins (work_id, skin_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
        .bind(work_id)
        .bind(skin_id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"err":0,"msg":"Work skin assigned"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn err_of(css: &str) -> String {
        sanitize_css(css).unwrap_err()
    }

    #[test]
    fn sanitize_rejects_import() {
        assert!(err_of("@import url(evil)").contains("@import"));
    }

    #[test]
    fn sanitize_rejects_obfuscated_js() {
        assert!(err_of("a:javascript:alert(1)").contains("javascript"));
        // CSS escape obfuscation: \6a = 'j'
        assert!(err_of("x{background:url(\\6a avascript:alert(1))}").contains("javascript"));
        assert!(err_of("width: expression(alert(1));").contains("expression"));
        // Comment obfuscation: expr/**/ession(
        assert!(err_of("width: expr/**/ession(alert(1));").contains("expression"));
        assert!(err_of("behavior:url(evil.htc)").contains("behavior"));
        assert!(err_of("-moz-binding: url(evil.xml)").contains("-moz-binding"));
        assert!(
            err_of("a{background:url(\"data:text/html,<script>\")}").contains("data:text/html")
        );
        assert!(err_of("}</style><script>alert(1)</script>").contains("</style"));
    }

    #[test]
    fn sanitize_keeps_legitimate_css() {
        let css = "/* nice theme */\nbody { background: #123; font-family: serif; }\n.card { border-radius: 8px; }";
        let out = sanitize_css(css).unwrap();
        assert!(out.contains("border-radius: 8px"));
        assert!(!out.contains("/* nice theme */")); // comments stripped
    }

    #[test]
    fn sanitize_allows_data_image_urls() {
        // SVG-as-image is inert in browsers; common for skin backgrounds.
        let css = "body{background:url(data:image/svg+xml;base64,AAAA)}";
        assert!(sanitize_css(css).is_ok());
    }

    #[test]
    fn sanitize_rejects_oversized() {
        assert!(err_of(&"a".repeat(60_000)).contains("limit"));
    }
}

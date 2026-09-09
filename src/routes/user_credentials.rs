//! User site credentials — encrypted storage for external site logins
//! (AO3, FFN, etc.) used by the scraper to fetch works on behalf of users.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx;
use std::sync::Arc;

use crate::crypto::encrypt;
use crate::db::models::UserSiteCredential;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct SetSiteCredentialsRequest {
    pub domain: String,
    pub username: String,
    pub password: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SiteCredentialResponse {
    pub id: i64,
    pub domain: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserSiteCredential> for SiteCredentialResponse {
    fn from(c: UserSiteCredential) -> Self {
        Self {
            id: c.id,
            domain: c.domain,
            username: c.username,
            created_at: c.created_at,
            expires_at: c.expires_at,
        }
    }
}

/// PUT /api/user/site-credentials — store/update credentials for a site
pub async fn set_site_credentials_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<SetSiteCredentialsRequest>,
) -> Result<Json<SiteCredentialResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let expires_at = req
        .expires_at
        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(365));

    // Simple encryption (in production, use proper key management)
    let jwt_secret = state.jwt_secret.clone();
    let password_enc = encrypt(&req.password, &jwt_secret);

    let cred = sqlx::query_as::<_, UserSiteCredential>(
        r#"
        INSERT INTO user_site_credentials (user_id, domain, username, password_enc, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, domain) DO UPDATE SET
            username = EXCLUDED.username,
            password_enc = EXCLUDED.password_enc,
            expires_at = EXCLUDED.expires_at,
            created_at = now()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&req.domain)
    .bind(&req.username)
    .bind(&password_enc)
    .bind(expires_at)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(cred.into()))
}

/// GET /api/user/site-credentials — list stored credentials (no passwords)
pub async fn list_site_credentials_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<SiteCredentialResponse>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let creds = sqlx::query_as::<_, UserSiteCredential>(
        "SELECT * FROM user_site_credentials WHERE user_id = $1 ORDER BY domain",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(creds.into_iter().map(Into::into).collect()))
}

/// DELETE /api/user/site-credentials/{domain} — remove credentials for a site
pub async fn delete_site_credentials_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(domain): Path<String>,
) -> Result<(), AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let result =
        sqlx::query("DELETE FROM user_site_credentials WHERE user_id = $1 AND domain = $2")
            .bind(user_id)
            .bind(&domain)
            .execute(&state.db)
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Credentials not found".into()));
    }

    Ok(())
}

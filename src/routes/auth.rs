use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,       // user id
    pub username: String,
    pub role: i16, // legacy: 0=regular, 1=trusted, 5=curator, 10=admin (read-only after F7)
    /// F7 site-wide level (0-100). The active gate for forum mod/admin.
    pub level: i16,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: i16,
    pub reputation: i32,
    pub email: Option<String>,
    /// F7 site-wide level (0-100).
    pub level: i16,
    /// F7 experience points (drives level).
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    /// Honeypot — hidden CSS-invisible field that real users never see.
    /// Any non-empty value means a bot filled it; the handler silently
    /// swallows the registration.
    #[serde(default)]
    pub website: Option<String>,
    /// Form-open timestamp (epoch ms) set by the frontend JS on mount.
    /// Missing or impossibly fast submissions are silently rejected.
    #[serde(default)]
    pub form_opened_at: Option<String>,
    /// F7: single-use invite code (required when REGISTRATION_MODE=invite,
    /// optional otherwise — a valid code is still consumed).
    #[serde(default)]
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// Create a JWT token for a user.
pub fn create_token(user: &User, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role,
        level: user.level,
        exp: (now + Duration::days(30)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
}

/// Create a long-lived refresh token (365 days).
pub fn create_refresh_token(user_id: i32, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        username: String::new(),
        role: 0,
        level: 0,
        exp: (now + Duration::days(365)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
}

/// Verify a JWT token and return claims.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AppError::BadRequest(format!("Invalid token: {}", e)))
}

/// Register a new user.
pub async fn register_user(db: &PgPool, req: RegisterRequest, secret: &str) -> Result<AuthResponse, AppError> {
    if req.username.len() < 2 || req.username.len() > 32 {
        return Err(AppError::BadRequest("Username must be 2-32 characters".to_string()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Hash error: {}", e)))?;

    let row = sqlx::query_as::<_, (i32, String, i16, i32, Option<String>, i16, i64)>(
        "INSERT INTO users (username, password_hash, email) VALUES ($1, $2, COALESCE($3, ''))
         RETURNING id, username, role, reputation, email, level, exp",
    )
    .bind(&req.username)
    .bind(&hash)
    .bind(&req.email)
    .fetch_one(db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::BadRequest("Username or email already taken".to_string())
        }
        _ => AppError::Database(e.to_string()),
    })?;

    let user = User {
        id: row.0,
        username: row.1.clone(),
        role: row.2,
        reputation: row.3,
        email: row.4,
        level: row.5,
        exp: row.6,
    };
    let token = create_token(&user, secret)?;
    Ok(AuthResponse { token, user })
}

/// Login a user with username + password.
pub async fn login_user(db: &PgPool, req: LoginRequest, secret: &str) -> Result<AuthResponse, AppError> {
    let row = sqlx::query_as::<_, (i32, String, String, i16, i32, Option<String>, i16, i64)>(
        "SELECT id, username, password_hash, role, reputation, email, level, exp FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let (id, username, hash, role, reputation, email, level, exp) = row;

    let valid = bcrypt::verify(&req.password, &hash)
        .map_err(|e| AppError::Internal(format!("Verify error: {}", e)))?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    let user = User { id, username, role, reputation, email, level, exp };
    let token = create_token(&user, secret)?;
    Ok(AuthResponse { token, user })
}

/// Axum extractor that reads the Bearer token from the Authorization header.
/// If no token is present or it's invalid, user_id is None (anonymous).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub role: i16,
    /// F7 site-wide level (0-100); 0 for anonymous.
    pub level: i16,
}

impl Default for AuthUser {
    fn default() -> Self {
        Self { user_id: None, username: None, role: 0, level: 0 }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get the Authorization header
        let auth_header = parts.headers.get("Authorization").and_then(|v| v.to_str().ok());

        if let Some(header_val) = auth_header {
            if let Some(token) = header_val.strip_prefix("Bearer ") {
                // We need the secret — read from env at request time (cheap, env is cached by OS)
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
                if let Ok(claims) = verify_token(token, &secret) {
                    return Ok(AuthUser {
                        user_id: Some(claims.sub),
                        username: Some(claims.username),
                        role: claims.role,
                        level: claims.level,
                    });
                }
            }
        }

        Ok(AuthUser::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let user = User {
            id: 1,
            username: "testuser".into(),
            role: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
        };
        let secret = "test-secret";
        let token = create_token(&user, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, 0);
        assert_eq!(claims.level, 0);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_token("invalid.token.here", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_secret() {
        let user = User {
            id: 1,
            username: "testuser".into(),
            role: 0,
            reputation: 0,
            email: None,
            level: 0,
            exp: 0,
        };
        let token = create_token(&user, "secret1").unwrap();
        let result = verify_token(&token, "secret2");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "mypass123";
        let hash = bcrypt::hash(password, 4).unwrap();
        assert!(bcrypt::verify(password, &hash).unwrap());
        assert!(!bcrypt::verify("wrong", &hash).unwrap());
    }
}

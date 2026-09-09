# Permanent /link Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Make `/link` permanent by implementing refresh tokens so the bot never needs re-linking.

**Architecture:** Server-side refresh token endpoint + bot-side auto-refresh. JWT stays short-lived (30d) for security; refresh token is long-lived (365d). Bot detects expired JWT and silently refreshes.

**Tech Stack:** Rust, Axum, Redis, jsonwebtoken, sqlx

---

## Problem Analysis

Current flow:
1. User runs `/link username password`
2. Bot calls `POST /api/auth/login` → gets JWT (30-day expiry)
3. JWT stored in Redis (`archivist:token:<id>`) with 30-day TTL
4. `touch()` extends Redis TTL but NOT the JWT itself
5. After 30 days, JWT expires, user must re-link

Root cause: No refresh token mechanism. JWT expiry is hardcoded to 30 days.

---

## Task 1: Add refresh token endpoint to server

**Objective:** Create `POST /api/auth/refresh` that exchanges a refresh token for a new JWT.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/src/routes/auth.rs`
- Modify: `/home/alvaro/code/rust/fichub/src/routes/social.rs`
- Modify: `/home/alvaro/code/rust/fichub/src/server.rs`

**Step 1: Add RefreshRequest struct to auth.rs**

```rust
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
```

**Step 2: Add create_refresh_token function to auth.rs**

```rust
/// Create a long-lived refresh token (365 days).
pub fn create_refresh_token(user_id: i32, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        username: String::new(), // Not needed for refresh
        role: 0,
        level: 0,
        exp: (now + Duration::days(365)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
}
```

**Step 3: Add refresh_handler to social.rs**

```rust
/// POST /api/auth/refresh — exchange refresh token for new JWT
pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<auth::RefreshRequest>,
) -> Result<Json<Value>, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    
    // Verify refresh token
    let claims = auth::verify_token(&body.refresh_token, &secret)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".into()))?;
    
    // Check if refresh token is expired (365 days)
    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err(AppError::Unauthorized("Refresh token expired".into()));
    }
    
    // Fetch user from DB to get current role/level
    let user = sqlx::query_as::<_, auth::User>(
        "SELECT id, username, role, reputation, email, level, exp FROM users WHERE id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;
    
    // Create new JWT
    let token = auth::create_token(&user, &secret)?;
    
    // Create new refresh token (rotation)
    let new_refresh = auth::create_refresh_token(user.id, &secret)?;
    
    Ok(Json(json!({
        "err": 0,
        "token": token,
        "refresh_token": new_refresh,
        "user": user,
    })))
}
```

**Step 4: Register route in server.rs**

Add after the login route:
```rust
.route("/api/auth/refresh", axum::routing::post(crate::routes::social::refresh_handler))
```

**Step 5: Update login/register to return refresh token**

In `social.rs`, update `login_handler` and `register_handler` to include refresh token in response:

```rust
// After creating token
let refresh = auth::create_refresh_token(result.user.id, &secret)?;

Ok(Json(json!({
    "err": 0,
    "token": result.token,
    "refresh_token": refresh,
    "user": result.user,
})))
```

**Step 6: Test manually**

```bash
# Login and get tokens
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass"}'

# Use refresh token
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"<token_from_login>"}'
```

**Step 7: Commit**

```bash
git add src/routes/auth.rs src/routes/social.rs src/server.rs
git commit -m "feat: add refresh token endpoint for permanent bot linking"
```

---

## Task 2: Update bot TokenStore to store refresh token

**Objective:** Store refresh token alongside JWT in Redis.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/store.rs`

**Step 1: Update StoredToken struct**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub token: String,
    pub refresh_token: Option<String>, // NEW
    pub fichub_user_id: i64,
    pub username: String,
    pub linked_at: String,
}
```

**Step 2: Update TOKEN_TTL_SECS**

```rust
/// Default TTL for stored tokens (365 days — refresh token keeps us alive).
pub const TOKEN_TTL_SECS: u64 = 60 * 60 * 24 * 365;
```

**Step 3: Add refresh method to TokenStore**

```rust
/// Refresh an expired JWT using the stored refresh token.
pub async fn refresh(&self, discord_id: &str, client: &crate::api::FichubClient) -> Result<Option<StoredToken>> {
    let key = Self::key(discord_id);
    let raw: Option<Vec<u8>> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut self.conn.clone())
        .await?;
    
    match raw {
        Some(bytes) => {
            let stored: StoredToken = serde_json::from_slice(&bytes)?;
            if let Some(refresh) = &stored.refresh_token {
                match client.refresh(refresh).await {
                    Ok(auth) => {
                        let new_stored = StoredToken {
                            token: auth.token,
                            refresh_token: Some(auth.refresh_token),
                            fichub_user_id: stored.fichub_user_id,
                            username: stored.username,
                            linked_at: chrono::Utc::now().to_rfc3339(),
                        };
                        // Store new tokens
                        redis::cmd("SETEX")
                            .arg(&key)
                            .arg(TOKEN_TTL_SECS)
                            .arg(serde_json::to_vec(&new_stored)?)
                            .exec_async(&mut self.conn.clone())
                            .await?;
                        Ok(Some(new_stored))
                    }
                    Err(_) => Ok(None), // Refresh failed
                }
            } else {
                Ok(None) // No refresh token
            }
        }
        None => Ok(None),
    }
}
```

**Step 4: Update tests**

```rust
#[test]
fn stored_token_roundtrip() {
    let st = StoredToken {
        token: "abc".into(),
        refresh_token: Some("refresh_abc".into()), // NEW
        fichub_user_id: 42,
        username: "tester".into(),
        linked_at: "now".into(),
    };
    let bytes = serde_json::to_vec(&st).unwrap();
    let back: StoredToken = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(back.token, "abc");
    assert_eq!(back.refresh_token, Some("refresh_abc".into()));
    assert_eq!(back.fichub_user_id, 42);
    assert_eq!(back.username, "tester");
}
```

**Step 5: Commit**

```bash
git add fanfic-archivist/src/store.rs
git commit -m "feat: add refresh token storage to bot TokenStore"
```

---

## Task 3: Add refresh method to bot API client

**Objective:** Add `refresh()` method to FichubClient.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/api.rs`

**Step 1: Add AuthResponseWithRefresh struct**

```rust
#[derive(Debug, Deserialize)]
pub struct AuthResponseWithRefresh {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}
```

**Step 2: Add refresh method**

```rust
/// Exchange a refresh token for a new JWT + refresh token.
pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponseWithRefresh> {
    let url = format!("{}/api/auth/refresh", self.base_url);
    let resp = self.client
        .post(&url)
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(Error::Api(format!("Refresh failed: {}", resp.status())));
    }
    
    let body: serde_json::Value = resp.json().await?;
    if body["err"].as_i64().unwrap_or(-1) != 0 {
        return Err(Error::Api(body["msg"].as_str().unwrap_or("Unknown error").into()));
    }
    
    Ok(AuthResponseWithRefresh {
        token: body["token"].as_str().unwrap_or("").into(),
        refresh_token: body["refresh_token"].as_str().unwrap_or("").into(),
        user: serde_json::from_value(body["user"].clone())?,
    })
}
```

**Step 3: Commit**

```bash
git add fanfic-archivist/src/api.rs
git commit -m "feat: add refresh method to FichubClient"
```

---

## Task 4: Update /link command to store refresh token

**Objective:** Store refresh token when user links.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/commands/link.rs`

**Step 1: Update link command**

The login endpoint now returns `refresh_token`. Update the link command to store it:

```rust
pub async fn link(
    ctx: Context<'_>,
    #[description = "FicHub username"] username: String,
    #[description = "FicHub password (sent once, never stored)"] password: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Exchange credentials for a JWT via the login endpoint.
    let auth = data.client.login(&username, &password).await?;
    let user = auth.user;

    let stored = StoredToken {
        token: auth.token,
        refresh_token: auth.refresh_token, // NEW
        fichub_user_id: user.id,
        username: user.username.clone(),
        linked_at: chrono::Utc::now().to_rfc3339(),
    };
    data.tokens
        .set(ctx.author().id.get(), &stored)
        .await?;

    ctx.say(format!(
        "✅ Linked **{}** (FicHub user #{}) to your Discord account.\n\
         Personalized recommendations and library commands are now available.",
        user.username, user.id
    ))
    .await?;
    Ok(())
}
```

**Step 2: Update whoami to auto-refresh**

```rust
pub async fn whoami(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    match data
        .tokens
        .get(&ctx.author().id.get().to_string())
        .await?
    {
        Some(stored) => {
            // Try current token first
            data.tokens.touch(&ctx.author().id.get().to_string()).await?;
            let valid = data.client.me(&stored.token).await;
            match valid {
                Ok(me) => {
                    let name = me
                        .user
                        .as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_else(|| stored.username.clone());
                    ctx.say(format!("Linked as **{name}** (FicHub user #{})", stored.fichub_user_id))
                        .await?;
                }
                Err(_) => {
                    // Token expired — try refresh
                    if let Some(refreshed) = data.tokens.refresh(
                        &ctx.author().id.get().to_string(),
                        &data.client
                    ).await? {
                        let name = refreshed.username.clone();
                        ctx.say(format!(
                            "✅ Token refreshed! Linked as **{name}** (FicHub user #{})",
                            refreshed.fichub_user_id
                        ))
                        .await?;
                    } else {
                        ctx.say(format!(
                            "Linked as **{}** but the token expired — run `/link` again.",
                            stored.username
                        ))
                        .await?;
                    }
                }
            }
        }
        None => {
            ctx.say("Not linked. Run `/link` to connect your FicHub account.")
                .await?;
        }
    }
    Ok(())
}
```

**Step 3: Commit**

```bash
git add fanfic-archivist/src/commands/link.rs
git commit -m "feat: store refresh token on /link, auto-refresh on expiry"
```

---

## Task 5: Add auto-refresh to authed commands

**Objective:** Automatically refresh token when any authed command fails.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/commands/library.rs`
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/commands/recs.rs`

**Step 1: Create helper function in commands/mod.rs**

```rust
/// Get a valid token, refreshing if necessary.
pub async fn get_valid_token(ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let data = ctx.data();
    let discord_id = ctx.author().id.get().to_string();
    
    if let Some(stored) = data.tokens.get(&discord_id).await? {
        // Try current token
        if data.client.me(&stored.token).await.is_ok() {
            return Ok(Some(stored.token));
        }
        
        // Try refresh
        if let Some(refreshed) = data.tokens.refresh(&discord_id, &data.client).await? {
            return Ok(Some(refreshed.token));
        }
    }
    
    Ok(None)
}
```

**Step 2: Update library and recs commands**

Replace direct token usage with `get_valid_token()`:

```rust
// Before
let token = match data.tokens.get(&discord_id).await? {
    Some(stored) => stored.token,
    None => { ctx.say("Not linked.").await?; return Ok(()); }
};

// After
let token = match get_valid_token(&ctx).await? {
    Some(t) => t,
    None => { ctx.say("Not linked. Run `/link` first.").await?; return Ok(()); }
};
```

**Step 3: Commit**

```bash
git add fanfic-archivist/src/commands/mod.rs fanfic-archivist/src/commands/library.rs fanfic-archivist/src/commands/recs.rs
git commit -m "feat: auto-refresh token in all authed bot commands"
```

---

## Task 6: Update link.rs comment

**Objective:** Update documentation to reflect new flow.

**Files:**
- Modify: `/home/alvaro/code/rust/fichub/fanfic-archivist/src/commands/link.rs`

**Step 1: Update top comment**

```rust
//! Account linking: `/link` + `/unlink`.
//!
//! Two flows:
//! 1. **API-token flow** (recommended once FicHub P8 #88 "API tokens" ships):
//!    user pastes a token from their profile → stored in Redis, verified via
//!    `/api/auth/me`.
//! 2. **Password flow** (today): user passes username+password;
//!    the bot exchanges them for a JWT + refresh token via `/api/auth/login`.
//!    JWT is short-lived (30d) for security; refresh token is long-lived (365d).
//!    Bot auto-refreshes when JWT expires — no re-linking needed.
//!
//! Tokens stored keyed by platform user id in Redis (`archivist:token:<id>`),
//! TTL 365 days, refreshed on each authed call.
```

**Step 2: Commit**

```bash
git add fanfic-archivist/src/commands/link.rs
git commit -m "docs: update link.rs documentation for refresh token flow"
```

---

## Task 7: Deploy and verify

**Objective:** Deploy changes and verify /link works permanently.

**Steps:**

1. Build server:
```bash
cd /home/alvaro/code/rust/fichub
cargo build --release
```

2. Deploy server to ThinkCentre

3. Build bot:
```bash
cd /home/alvaro/code/rust/fichub/fanfic-archivist
cargo build --release
```

4. Deploy bot

5. Test /link:
```
/link yourusername yourpassword
```

6. Verify response includes success message

7. Wait or mock JWT expiry, verify /whoami auto-refreshes

---

## Verification Checklist

- [ ] `/api/auth/refresh` endpoint works
- [ ] Login returns refresh_token
- [ ] Register returns refresh_token
- [ ] Bot stores refresh_token in Redis
- [ ] Bot auto-refreshes when JWT expires
- [ ] /whoami shows "Token refreshed!" after auto-refresh
- [ ] All authed commands work with refreshed token
- [ ] No re-linking needed after 30 days

---

## Security Notes

- Refresh tokens rotate on each use (new refresh token returned)
- Refresh tokens have 365-day expiry (configurable)
- JWT stays short-lived (30d) for security
- Password never stored — only exchanged once for tokens
- Refresh tokens stored in Redis (ephemeral, not persistent DB)

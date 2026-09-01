# Part 35 — Authentication Deep Dive

> In this chapter you will learn how FicHub's authentication system works end-to-end: JWT tokens, Bearer auth, bcrypt password hashing, the two-token system (19-day JWT + 365-day refresh), the honeypot trap that blocks bot signups, rate-limited auth routes, and the AuthUser extractor that powers anonymous-friendly authorization.

---

## Overview

FicHub's auth system uses a **two-token pattern**:

- **JWT access token** — 30 days, sent as a `Bearer` header on every authenticated request.
- **Refresh token** — 365 days, long-lived, exchanged for a new JWT when the old one expires.

The JWT carries a `Claims` struct with `sub` (user id), `username`, `role`, and `level`. The `AuthUser` extractor parses the Bearer header and returns `user_id: None` (anonymous) on any failure — meaning routes can degrade gracefully for unauthenticated users without special-casing.

Bot protection is handled by a **honeypot + timing trap** (hidden CSS field + form-open timestamp). A bot that fills the hidden `website` field or submits in under 500ms gets a success-looking response but **no account is created and no token is returned**.

Auth routes are rate-limited at **10 per minute per IP** (the `auth` tier in the rate limiter).

---

## Chapter 35.1 — JWT Claims and Token Creation

### Goal

Build the JWT token system: Claims struct, 19-day access token, 365-day refresh token, and verification.

### Actions

#### 1. The Claims struct

```rust
// src/routes/auth.rs (lines 10–84)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,       // user id
    pub username: String,
    pub role: i16,      // 0=regular, 1=trusted, 5=curator, 10=admin
    pub level: i16,     // F7 site-wide level (0-100) — drives frontend role gates
    pub exp: usize,     // expiration (unix timestamp)
    pub iat: usize,     // issued-at
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: i16,
    pub reputation: i32,
    pub email: Option<String>,
    pub level: i16,     // F7 site-wide level (0-100)
    pub exp: i64,       // F7 experience points (drives level)
}

/// Create JWT access token (30 days)
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

/// Create long-lived refresh token (365 days)
pub fn create_refresh_token(user_id: i32, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        username: String::new(),  // refresh tokens don't carry username
        role: 0,                   // role is always re-read from DB on refresh
        level: 0,
        exp: (now + Duration::days(365)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
}

/// Verify a JWT token and return claims
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AppError::BadRequest(format!("Invalid token: {}", e)))
}
```

> **💡 Key Concept**: The access token and refresh token are **both JWTs** with the same structure but different TTLs. This means the refresh endpoint can validate the token using the same `verify_token` function — no separate refresh-token store needed. The downside: you can't revoke a refresh token server-side without a denylist.

> **⚠️ Watch Out**: The refresh token carries `role: 0` and `username: ""`. The refresh handler **always re-reads the user from the database** and re-issues a token with the current role. This prevents stale roles in old refresh tokens.

#### 2. Unit tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_create_and_verify_token() {
        let user = User { id: 1, username: "testuser", role: 0, reputation: 0, email: None, level: 0, exp: 0 };
        let secret = "test-secret";
        let token = create_token(&user, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, 0);
    }

    #[test]
    fn test_verify_invalid_token() {
        assert!(verify_token("invalid.token.here", "secret").is_err());
    }

    #[test]
    fn test_verify_wrong_secret() {
        let token = create_token(&user, "secret1").unwrap();
        assert!(verify_token(&token, "secret2").is_err());
    }

    #[test]
    fn test_password_hashing() {
        let hash = bcrypt::hash("mypass123", 4).unwrap();
        assert!(bcrypt::verify("mypass123", &hash).unwrap());
        assert!(!bcrypt::verify("wrong", &hash).unwrap());
    }
}
```

### Try It Yourself

```bash
# Run the auth unit tests (no DB needed)
cargo test --lib routes::auth::tests -- --nocapture
```

### Check

- ✅ Access token expires in 30 days; refresh token in 365 days.
- ✅ Refresh token carries `role: 0` — role is always re-fetched from DB.
- ✅ `verify_token` returns `AppError::BadRequest` on invalid/expired tokens.
- ✅ Password hashing uses bcrypt with `DEFAULT_COST` = 12 rounds.
- ✅ `test_password_hashing` uses cost factor 4 (fast for tests).

### What you built

The JWT token system — a `Claims` struct carrying user id, username, role, and level; a 19-day access token; a 365-day refresh token; and a `verify_token` function that validates against the shared secret. The refresh token deliberately carries a stale role (0) so the refresh handler always re-reads the user from the DB.

---

## Chapter 35.2 — The AuthUser Extractor

### Goal

Build the Axum `FromRequestParts` extractor that reads the Bearer token, making every route auth-aware — without forcing a 401 on anonymous users.

### Actions

```rust
// src/routes/auth.rs (lines 172–216)
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub role: i16,       // 0 for anonymous
    pub level: i16,      // 0 for anonymous
}

impl Default for AuthUser {
    fn default() -> Self {
        Self { user_id: None, username: None, role: 0, level: 0 }
    }
}

/// Reads Bearer token from Authorization header. Returns AuthUser::default()
/// (anonymous) if the header is missing, the token is invalid, or the secret
/// is unreadable. The route itself decides what to do with an anonymous user.
impl<S> FromRequestParts<S> for AuthUser
where S: Send + Sync,
{
    type Rejection = ();  // never rejects — always returns Ok

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get("Authorization").and_then(|v| v.to_str().ok()) {
            if let Some(token) = header_val.strip_prefix("Bearer ") {
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
        Ok(AuthUser::default())  // anonymous — no error
    }
}
```

#### How routes use it

```rust
// Anonymous-friendly (returns different data, never 401):
pub async fn personal_recommendations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Ok(empty_personal_response());  // enough_data: false
    };
    // ... personalized recs
}

// Auth-required (401 on anonymous):
pub async fn add_bookmark_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BookmarkBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    // ...
}

// Admin-only (403 on anonymous + non-admin):
pub async fn admin_pending_uploads(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 { return Err(AppError::Forbidden("Admin access required".into())); }
    // ...
}
```

> **💡 Key Concept**: `AuthUser` is the **identity object** for every route. Because `Rejection = ()` and it always returns `Ok`, routes never fail just because a user isn't logged in. The route logic decides: anonymous-friendly routes return a degraded response; auth-required routes return 401; admin routes return 403. This makes the same `AuthUser` extractor serve all three tiers uniformly.

### Try It Yourself

```bash
# No auth header → 401 on auth-required endpoint
curl "http://localhost:8000/api/v1/bookmarks" | jq  # {"err": 401, "msg": "Login required"}

# With valid JWT → 200 with data
curl "http://localhost:8000/api/v1/bookmarks" \
  -H "Authorization: Bearer <jwt>" | jq

# Invalid token → still anonymous (AuthUser::default, not 401)
curl "http://localhost:8000/api/recommendations/personal" \
  -H "Authorization: Bearer garbage" | jq  # {"enough_data": false}
```

### Check

- ✅ `AuthUser` always returns `Ok` (never `Rejection`) — anonymous on any auth failure.
- ✅ `JWT_SECRET` env var with fallback to `"fichub-dev-secret"` for local dev.
- ✅ `user.role` defaults to `0` for anonymous (not `-1` or `None`).
- ✅ Admin routes check `user.role < 10` → `403 Forbidden`.

### What you built

The `AuthUser` extractor — an Axum `FromRequestParts` impl that reads Bearer tokens, returns anonymous on any failure, and lets each route decide how to handle unauthenticated users. This single pattern powers all three tiers: anonymous-friendly, auth-required, and admin-only.

---

## Chapter 35.3 — Registration with Honeypot Trap

### Goal

Understand how FicHub blocks bot signups using a honeypot field and form-timing trap, and the invite-code gate for restricted signup.

### Actions

#### 1. The honeypot trap

```rust
// src/routes/honeypot.rs (lines 14–88)
/// Hidden fields the frontend renders invisibly; bots that fill them are caught.
pub const HONEYPOT_FIELD: &str = "website";
pub const OPENED_AT_FIELD: &str = "form_opened_at";
pub const MIN_FORM_MS: u64 = 500;     // too fast → bot
pub const MAX_FORM_AGE_MS: u64 = 60 * 60 * 1000;  // >1h old → stale/bot

pub enum TrapVerdict { Clean, RejectSilently }

/// Silent rejection: bot gets a success-looking response, nothing is persisted.
pub fn inspect_submission(
    website: Option<&str>,           // hidden field — any non-empty = bot
    form_opened_at: Option<&str>,     // epoch-ms timestamp set by frontend JS
    now_ms: u64,
) -> TrapVerdict {
    // 1. Honeypot: any non-empty value (whitespace-only is OK)
    if website.map(|w| !w.trim().is_empty()).unwrap_or(false) {
        return TrapVerdict::RejectSilently;
    }
    // 2. Missing form_opened_at → not from real form (curl/scripted)
    let opened_at = match form_opened_at { Some(r) => r.parse().else { return RejectSilently }, None => return RejectSilently };
    // 3. Absurdly old (>1h) → stale/cached/replayed
    if now_ms.saturating_sub(opened_at) > MAX_FORM_AGE_MS { return TrapVerdict::RejectSilently; }
    // 4. Too fast (<500ms) → bot can't type that quickly
    if now_ms.saturating_sub(opened_at) < MIN_FORM_MS { return TrapVerdict::RejectSilently; }
    TrapVerdict::Clean
}
```

> **⚠️ Watch Out**: A bare `curl -X POST /api/auth/register` fails silently — the bot gets a `200 OK` with an empty token and no account. The bot never knows it was caught. This is by design: tipping off the bot wastes no resources, and the silent rejection means no false negatives.

#### 2. The register handler

```rust
// src/routes/social.rs (lines 18–92)
/// POST /api/v1/auth/register
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Rate limit: auth tier (10/min per IP)
    enforce_auth_rate_limit(&state).await?;

    // 2. Honeypot + timing trap
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("registration silently rejected (honeypot/timing trap)");
        return Ok(Json(json!({
            "err": 0,
            "token": "",        // ← empty token, no account created
            "user": { "id": 0, "username": body.username, "role": 0, "reputation": 0, "email": null },
        })));
    }

    // 3. Registration mode gate
    // * "invite": invite code is required
    // * "application": registration disabled — apply via /api/registration-applications
    // * "open" (default): no gate; optional valid code is still consumed
    let invite_code = body.invite_code.clone().unwrap_or_default();
    match crate::routes::subsystems::registration_mode() {
        "invite" => { crate::routes::subsystems::validate_invite_code(&state.db, &invite_code).await?; }
        "application" => { return Err(AppError::Forbidden("Registration by application only — apply via /api/registration-applications".into())); }
        _ => {}
    }

    // 4. Create the user (bcrypt hash, INSERT INTO users)
    let result = auth::register_user(&state.db, body, &secret).await?;
    let refresh = auth::create_refresh_token(result.user.id, &secret)?;

    // 5. Consume invite code (idempotent, any mode)
    if !invite_code.trim().is_empty() {
        crate::routes::subsystems::consume_invite_code(&state.db, &invite_code, result.user.id).await;
    }

    Ok(Json(json!({
        "err": 0,
        "token": result.token,
        "refresh_token": refresh,
        "user": result.user,
    })))
}
```

### Try It Yourself

```bash
# Register through the real form (frontend) — sets form_opened_at + empty website
# Bot curl attempt — gets success-looking response but no token
curl -X POST "http://localhost:8000/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username": "bot", "password": "123456", "email": null}'  # missing form_opened_at → trap
# Response: {"err": 0, "token": "", "user": {"id": 0, ...}}

# Proper registration with invite code (invite mode)
curl -X POST "http://localhost:8000/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username": "newuser", "password": "securepass", "email": "u@example.com", "invite_code": "abc123", "form_opened_at": "1700000000000"}' | jq
```

### Check

- ✅ Honeypot fires on any non-empty `website` field value (whitespace-only is ignored).
- ✅ Missing `form_opened_at` → `RejectSilently` (bare API scripting can't register).
- ✅ Timestamp < 500ms ago → too fast for a human → `RejectSilently`.
- ✅ Timestamp > 1h old → stale → `RejectSilently`.
- ✅ Silently rejected responses return `"token": ""` and `"id": 0`.
- ✅ Invite codes are consumed idempotently via `ON CONFLICT`.

### What you built

The registration honeypot system — invisible trap fields, timing checks, silent rejection, and the invite-code gate. A bot that tries to register via the API directly (without the frontend's JS-set timestamp) gets a success-like response but never creates an account.

---

## Chapter 35.4 — Login, Refresh, and Logout

### Goal

Complete the auth lifecycle: login with failed-auth logging, token refresh, and session termination.

### Actions

#### 1. Login handler

```rust
// src/routes/social.rs (lines 95–172)
/// POST /api/v1/auth/login
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Rate limit (auth tier: 10/min per IP)
    enforce_auth_rate_limit(&state).await?;

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    match auth::login_user(&state.db, body, &secret).await {
        Ok(result) => {
            let refresh = auth::create_refresh_token(result.user.id, &secret)?;
            Ok(Json(json!({
                "err": 0,
                "token": result.token,
                "refresh_token": refresh,
                "user": result.user,
            })))
        }
        Err(e) => {
            // Log failed auth to bot-scorer (x-client-id + user-agent + IP)
            let client_id = headers.get("x-client-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            let ip = crate::limiter::client_ip_from_headers(
                headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
                "127.0.0.1".parse().expect("static ip"),
            );
            // Insert into request_log with source_id for '/api/auth/login'
            // etype = 'auth_failed' — aggregated into bot_scores.failed_auths
            crate::db::queries::insert_request_log(...).await;
            Err(e)
        }
    }
}
```

> **💡 Key Concept**: Failed login attempts are logged to the **bot-scorer pipeline**. The `etype='auth_failed'` entries feed into `bot_scores.failed_auths`, which contributes to the composite `bot_score` in `GET /api/admin/bots` (the "stuffing" flag = failed_auths > 3). This means brute-force attacks are automatically detected and surfaced to admins.

#### 2. Refresh handler

```rust
/// POST /api/auth/refresh
/// Exchange a 365-day refresh token for a fresh 19-day JWT + new refresh token.
pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<Value>, AppError> {
    let claims = auth::verify_token(&body.refresh_token, &secret)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".into()))?;

    // Check expiry
    let now = Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err(AppError::Unauthorized("Refresh token expired".into()));
    }

    // Re-read the user from DB to get current role/level (not the stale claims)
    let user = sqlx::query_as::<_, (i32, String, i16, i32, Option<String>, i16, i64)>(
        "SELECT id, username, role, reputation, email, level, exp FROM users WHERE id = $1"
    ).bind(claims.sub).fetch_optional(&state.db).await?
     .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    let user = auth::User { id: user.0, username: user.1, role: user.2,
        reputation: user.3, email: user.4, level: user.5, exp: user.6 };
    let token = auth::create_token(&user, &secret)?;
    let new_refresh = auth::create_refresh_token(user.id, &secret)?;

    // ROTATING refresh tokens: client gets both a new JWT and a new refresh.
    // The old refresh token is NOT stored server-side (no denylist → can't revoke
    // mid-life, but the 365-day TTL limits exposure).
    Ok(Json(json!({
        "err": 0,
        "token": token,
        "refresh_token": new_refresh,
        "user": user,
    })))
}
```

#### 3. Frontend AuthStore

```typescript
// src/lib/stores/auth.svelte.ts
const JWT_TTL = 30 * 24 * 60 * 60 * 1000;  // 30 days in ms
const REFRESH_THRESHOLD = 5 * 24 * 60 * 60 * 1000;  // refresh 5 days before expiry

class AuthStore {
    user: User | null = null;
    token: string | null = null;
    refresh_token: string | null = localStorage.getItem('refresh_token');

    private getTokenExpiry(token: string): number | null {
        const payload = JSON.parse(atob(token.split('.')[1]));
        return payload.exp * 1000;
    }

    private shouldAutoRefresh(): boolean {
        if (!this.token) return false;
        const exp = this.getTokenExpiry(this.token);
        if (!exp) return false;
        return (exp - Date.now()) < REFRESH_THRESHOLD;
    }

    async apiFetch(url: string, opts: RequestInit = {}) {
        if (this.shouldAutoRefresh() && this.refresh_token) {
            await this.refresh();  // silent refresh before expiry
        }
        opts.headers = { ...opts.headers, Authorization: `Bearer ${this.token}` };
        return fetch(url, opts);
    }

    async login(username: string, password: string) {
        const { token, refresh_token, user } = await api.post('/api/auth/login', { username, password });
        this.token = token;
        this.refresh_token = refresh_token;
        this.user = user;
        localStorage.setItem('refresh_token', refresh_token);
    }

    async refresh() {
        const { token, refresh_token, user } = await api.post('/api/auth/refresh', {
            refresh_token: this.refresh_token,
        });
        this.token = token;
        this.refresh_token = refresh_token;
        this.user = user;
        localStorage.setItem('refresh_token', refresh_token);
    }

    logout() {
        this.token = null; this.refresh_token = null; this.user = null;
        localStorage.removeItem('refresh_token');
    }
}
```

> **⚠️ Watch Out**: The frontend must call `shouldAutoRefresh()` before every authenticated request and silently refresh the JWT 5 days before expiry. If the user closes the tab and comes back within 365 days, the refresh token in `localStorage` still works — but if localStorage was cleared, they'll need to log in again.

#### 4. The /api/auth/me endpoint

```rust
/// GET /api/v1/auth/me — return current user from JWT claims
pub async fn me_handler(auth: AuthUser) -> Result<Json<Value>, AppError> {
    match auth.user_id {
        Some(id) => Ok(Json(json!({
            "err": 0,
            "user": { "id", "username", "role" }
        }))),
        None => Ok(Json(json!({ "err": 401, "msg": "Not authenticated" }))),
    }
}
```

### Try It Yourself

```bash
# Login
TOKEN=$(curl -s -X POST "http://localhost:8000/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secret123"}' | jq -r .token)

# Refresh (get a new JWT + new refresh token)
curl -s -X POST "http://localhost:8000/api/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" | jq

# Check who's logged in
curl "http://localhost:8000/api/auth/me" -H "Authorization: Bearer $TOKEN" | jq

# Logout (client-side: just clear localStorage)
# No server endpoint — refresh tokens can't be revoked server-side without a denylist
```

### Check

- ✅ Login handler logs `auth_failed` to `request_log` with `x-client-id` and IP for bot scoring.
- ✅ Refresh handler re-reads user from DB — old JWT role/level staleness is impossible.
- ✅ Refresh returns both a new JWT AND a new refresh token (rotating tokens).
- ✅ `/me` returns `{ "err": 401 }` for anonymous users (not 403).
- ✅ Logout is client-side only (clear localStorage) — no server-side revoke endpoint.

### What you built

The complete auth lifecycle — login with bot-score signal logging, JWT refresh with rotating tokens and DB re-read of role, a `/me` endpoint for identity checks, and client-side logout. Failed login attempts feed into the bot detection pipeline seen in Part 19.

---

## Chapter 35.5 — Rate-Limited Auth Routes

### Goal

Understand the auth-tier rate limiting that protects login/register from brute-force attacks.

### Actions

The auth routes (`register`, `login`) are rate-limited by the rate limiter to **10 requests per minute per IP**. This is the `auth` tier in `limiter/mod.rs`:

```rust
// src/limiter/mod.rs
pub enum Tier {
    Auth,      // 10/min per IP (login, register)
    ReadOnly,  // 60/min per IP (search, exports, RSS)
    Standard,  // 120/min with client_id (general API)
    Export,    // 5/hour per user (EPUB/MOBI/PDF generation)
}

pub fn tier_for_path(path: &str) -> Tier {
    if path.starts_with("/api/auth/register") || path.starts_with("/api/auth/login") {
        Tier::Auth
    } else if path.starts_with("/api/search") || path.starts_with("/api/epub") {
        Tier::ReadOnly
    } else if path.starts_with("/api/export") {
        Tier::Export
    } else {
        Tier::Standard
    }
}
```

The `enforce_auth_rate_limit` function in `social.rs` checks the token bucket for the client's IP against the `Auth` tier limits. Exceeding 10/min returns `AppError::RateLimited(secs)`.

### Try It Yourself

```bash
# Hammer login 11 times in a minute → 429
for i in $(seq 1 11); do
  curl -s -o /dev/null -w "%{http_code}" -X POST "http://localhost:8000/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username": "test", "password": "wrong"}'; echo
done
# First 10: 401 (wrong password). 11th: 429 (rate limited)
```

### Check

- ✅ Auth tier: 10 requests per minute per IP.
- ✅ Exceeding returns `429 Too Many Requests` with `Retry-After` header.
- ✅ Shadowbanned clients get stricter download buckets (Part 19).
- ✅ Rate limit is per-IP, not per-username (prevents user enumeration via timing).

### What you built

The auth rate-limiting layer — a tiered token bucket system that applies stricter limits to login/register (10/min) and feeds rate-limit violations into the same bot-scoring pipeline as failed auths.

---

## Chapter 35.6 — Frontend AuthStore with Role Mapping

### Goal

Understand how the frontend stores JWT tokens, auto-refreshes before expiry, and maps the legacy role ladder (0/1/5/10) onto the F7 site-wide level (0–100).

### Actions

```typescript
// src/lib/stores/auth.svelte.ts (lines 1–31)
const CACHED_USER_KEY = 'fichub_cached_user';

/// Legacy role → level fallback (F7). Fresh auth responses carry `level`;
/// a cached/legacy user may not. Map the old role ladder onto the site-wide
/// level scale so role gates (curator ≥ 50, admin ≥ 100) keep working.
export function roleToLevel(role: number | undefined): number {
  if (!role) return 0;
  if (role >= 10) return 100;  // admin → 100
  if (role >= 5) return 50;    // curator → 50
  if (role >= 1) return 1;     // trusted → 1
  return 0;                    // regular → 0
}

/// Resolve a user's effective site-wide level (0-100).
export function userLevel(user: Pick<User, 'role'> & { level?: number } | null): number {
  if (!user) return 0;
  if (typeof user.level === 'number' && user.level > 0) return user.level;
  return roleToLevel(user.role);
}
```

The AuthStore persists `refresh_token` to `localStorage` (survives tab close/reopen) and caches the last-known `user` object for offline use. The JWT is kept in memory only (never persisted to localStorage — XSS mitigation).

### Try It Yourself

```typescript
// The store automatically refreshes JWT 5 days before expiry
const auth = new AuthStore();
await auth.login("alice", "secret123");  // sets token + refresh_token
auth.userLevel(auth.user);               // → 100 for admin, 50 for curator
auth.apiFetch("/api/v1/bookmarks");      // auto-refreshes if needed
```

### Check

- ✅ `roleToLevel(10)` → 100; `roleToLevel(5)` → 50; `roleToLevel(1)` → 1.
- ✅ `userLevel` prefers the explicit `level` field from fresh auth responses, falls back to `roleToLevel`.
- ✅ `refresh_token` is persisted to `localStorage`; JWT `token` is memory-only.
- ✅ Cached user is stored under `CACHED_USER_KEY` for offline reader access.

### What you built

The frontend auth integration — a Svelte 5 store that manages JWT/refresh tokens, auto-refreshes before expiry, maps legacy roles to F7 levels, and persists only the refresh token (not the JWT) for XSS resistance.

---

## Conclusion

You now understand FicHub's complete authentication system:

1. **Two-token JWT** — 19-day access token + 365-day rotating refresh token, both signed with the same secret.
2. **AuthUser extractor** — always `Ok`, returns anonymous on any failure, lets routes decide auth tiers.
3. **Honeypot trap** — invisible `website` field + `form_opened_at` timestamp check, silent rejection, no account created.
4. **Rate-limited auth** — 10/min per IP on register/login, with failed attempts feeding bot scoring.
5. **Login → refresh → logout** — rotating refresh tokens, DB re-read of role on refresh, client-side logout.
6. **Role mapping** — legacy roles (0/1/5/10) map to F7 levels (0/1/50/100) for frontend gates.

The system is anonymous-friendly (routes degrade gracefully), bot-resistant (honeypot + rate limits + failed-auth scoring), and refresh-aware (rotating tokens, auto-refresh on the frontend).

Up next: Part 19 covers [Rate Limiter and Redis — already written above in this section].

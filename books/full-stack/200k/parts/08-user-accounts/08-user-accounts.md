# Part 8 — User Accounts

Every community site needs users. This part builds registration, login, JWT tokens, and the auth state on the frontend. You see how passwords are hashed, how tokens work, and how the frontend knows you're logged in.

---

## 8.1 Backend: `POST /api/auth/register` — `register_handler`

Open `src/routes/social.rs`. The registration handler is the first function:

```rust
// src/routes/social.rs (lines 17-92, excerpt)
use crate::routes::auth::{self, AuthUser, LoginRequest, RegisterRequest, RefreshRequest};
use crate::routes::honeypot::{self, TrapVerdict};

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    // Rate limit: 10 requests per minute per IP for auth endpoints
    enforce_auth_rate_limit(&state).await?;

    // Honeypot + timing trap: silently reject bots
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("registration silently rejected (honeypot/timing trap)");
        // Return a success-looking response with no token
        return Ok(Json(json!({
            "err": 0,
            "token": "",
            "user": {
                "id": 0,
                "username": body.username,
                "role": 0,
                "reputation": 0,
                "email": null,
            },
        })));
    }

    // Get JWT secret from environment
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());

    // F7: registration mode gate (invite, application, or open)
    let invite_code = body.invite_code.clone().unwrap_or_default();
    match crate::routes::subsystems::registration_mode() {
        "invite" => {
            crate::routes::subsystems::validate_invite_code(&state.db, &invite_code).await?;
        }
        "application" => {
            return Err(AppError::Forbidden(
                "Registration by application only — apply via /api/registration-applications".to_string(),
            ));
        }
        _ => {}
    }

    // Register the user
    let result = auth::register_user(&state.db, body, &secret).await?;

    // Create refresh token
    let refresh = auth::create_refresh_token(result.user.id, &secret)?;

    // Consume invite code if provided
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

### Breakdown

**Rate limiting**: Auth endpoints are rate-limited to prevent brute-force attacks. The `enforce_auth_rate_limit` function checks the Redis rate limiter.

**Honeypot + timing trap**: The handler checks two fields that real users don't see:
- `website` — a hidden CSS-invisible field. If it's filled, a bot filled it.
- `form_opened_at` — a timestamp set by the frontend JavaScript when the form mounts. If it's missing or the submission is too fast, it's a bot.

If the honeypot triggers, the handler returns a success-looking response with no token. The bot thinks it registered, but no account was created.

**Registration mode gate (F7)**: The site can be in one of three modes:
- `open` — anyone can register (default).
- `invite` — an invite code is required.
- `application` — users must apply via a separate endpoint.

**`auth::register_user`**: This is where the actual registration happens. Let's look at it.

---

## 8.2 Backend: `auth::register_user` — bcrypt, insert, token

Open `src/routes/auth.rs`:

```rust
// src/routes/auth.rs (lines 108-147, excerpt)
use bcrypt;
use sqlx::PgPool;

pub async fn register_user(
    db: &PgPool,
    req: RegisterRequest,
    secret: &str,
) -> Result<AuthResponse, AppError> {
    // Validate username length
    if req.username.len() < 2 || req.username.len() > 32 {
        return Err(AppError::BadRequest("Username must be 2-32 characters".to_string()));
    }
    // Validate password length
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    // Hash the password with bcrypt
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Hash error: {}", e)))?;

    // Insert the user into the database
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

    // Build the User struct
    let user = User {
        id: row.0,
        username: row.1.clone(),
        role: row.2,
        reputation: row.3,
        email: row.4,
        level: row.5,
        exp: row.6,
    };

    // Create a JWT token
    let token = create_token(&user, secret)?;

    Ok(AuthResponse { token, user })
}
```

### Breakdown

**Validation**: Username must be 2-32 characters. Password must be at least 6 characters.

**bcrypt hashing**: The password is hashed with bcrypt. The `DEFAULT_COST` is 10, which means the hash takes about 100ms to compute. This is intentional — it makes brute-force attacks expensive.

**Database insert**: The user is inserted into the `users` table. The `RETURNING` clause returns the inserted row, so we get the user's ID, username, role, reputation, email, level, and exp without a second query.

**Unique violation handling**: If the username or email already exists, PostgreSQL returns a unique violation error. The handler catches this and returns a user-friendly "Username or email already taken" error.

**JWT token**: The `create_token` function creates a JWT with the user's claims. The token expires in 30 days.

---

## 8.3 Backend: JWT tokens — `create_token`, `verify_token`

The JWT handling is in `src/routes/auth.rs`:

```rust
// src/routes/auth.rs (lines 10-19, excerpt)
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,       // user id
    pub username: String,
    pub role: i16,      // legacy: 0=regular, 1=trusted, 5=curator, 10=admin
    pub level: i16,     // F7 site-wide level (0-100)
    pub exp: usize,     // expiration time (Unix timestamp)
    pub iat: usize,     // issued at (Unix timestamp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: i16,
    pub reputation: i32,
    pub email: Option<String>,
    pub level: i16,
    pub exp: i64,
}

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

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AppError::BadRequest(format!("Invalid token: {}", e)))
}
```

### Breakdown

**`Claims`**: The JWT claims. `sub` is the user ID (standard JWT field). `exp` is the expiration time. `iat` is when the token was issued. The custom fields are `username`, `role`, `level`, and `exp` (experience points).

**`create_token`**: Creates a JWT with a 30-day expiration. The `Header::default()` uses the HS256 algorithm (HMAC with SHA-256). The secret comes from the `JWT_SECRET` environment variable.

**`verify_token`**: Decodes and verifies a JWT. If the token is invalid or expired, it returns an error.

**Why JWT?**: JWTs are stateless — the server doesn't need to store them. The token contains all the information needed to authenticate the user. This makes scaling easier (no session store to share across servers).

**Why 30-day expiration?**: Short-lived tokens are more secure, but they require frequent re-authentication. 30 days is a balance between security and convenience.

---

## 8.4 Backend: `POST /api/auth/login` — `login_handler`

```rust
// src/routes/social.rs (lines 94-172, excerpt)
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
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
            // Log failed auth attempts for bot detection
            // ... (omitted for brevity)
            Err(e)
        }
    }
}
```

### Breakdown

**`auth::login_user`**: Looks up the user by username, verifies the password with bcrypt, and creates a JWT token.

**Refresh token**: In addition to the 30-day access token, a 365-day refresh token is created. When the access token expires, the frontend can use the refresh token to get a new access token without re-entering the password.

**Failed auth logging**: Failed login attempts are logged for bot detection. The `bot_scorer` crate uses this data to identify automated attacks.

---

## 8.5 Backend: `GET /api/auth/me` — `me_handler`

```rust
// src/routes/social.rs (lines 174-180, excerpt)
pub async fn me_handler(
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    match auth.user_id {
        Some(id) => Ok(Json(json!({
            "err": 0,
            "user": {
                "id": id,
                "username": auth.username.unwrap_or_default(),
                "role": auth.role,
                "level": auth.level,
            },
        }))),
        None => Err(AppError::Unauthorized("Not logged in".to_string())),
    }
}
```

### Breakdown

**`AuthUser` extractor**: This is a custom Axum extractor that reads the Bearer token from the Authorization header and extracts the user's ID, username, role, and level. If no token is present, `user_id` is `None` (anonymous).

**The handler**: If the user is logged in, returns their info. If not, returns 401 Unauthorized.

---

## 8.6 The `AuthUser` extractor

```rust
// src/routes/auth.rs (lines 172-210, excerpt)
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub role: i16,
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
        let auth_header = parts.headers.get("Authorization").and_then(|v| v.to_str().ok());

        if let Some(header_val) = auth_header {
            if let Some(token) = header_val.strip_prefix("Bearer ") {
                // Verify the token and extract claims
                // ...
            }
        }

        Ok(AuthUser::default())
    }
}
```

### Breakdown

**Custom extractor**: Axum allows you to define custom extractors by implementing `FromRequestParts`. This extractor reads the Authorization header, verifies the JWT, and returns the user's info.

**Anonymous by default**: If no token is present, the extractor returns `AuthUser::default()` (anonymous). Handlers that require auth check `auth.user_id` and return 401 if it's `None`.

**This is how every protected endpoint works**: Every handler that needs auth just adds `auth: AuthUser` as a parameter. Axum extracts the user automatically.

---

## 8.7 Frontend: auth state in SvelteKit

Open `frontend/src/lib/stores/auth.svelte.ts`:

```typescript
// frontend/src/lib/stores/auth.svelte.ts (excerpt)
import { getMe, login, logout, register, setToken } from '$lib/api/social';
import type { User } from '$lib/api/social-types';

const CACHED_USER_KEY = 'fichub_cached_user';

export function roleToLevel(role: number | undefined): number {
  if (!role) return 0;
  if (role >= 10) return 100;   // admin
  if (role >= 5) return 50;     // curator
  if (role >= 1) return 1;      // trusted
  return 0;                       // regular
}

export function userLevel(user: Pick<User, 'role'> & { level?: number } | null | undefined): number {
  if (!user) return 0;
  if (typeof user.level === 'number' && user.level > 0) return user.level;
  return roleToLevel(user.role);
}

function cacheUser(user: User): void {
  try {
    localStorage.setItem(CACHED_USER_KEY, JSON.stringify(user));
  } catch { /* storage full/blocked — non-fatal */ }
}

function readCachedUser(): User | null {
  try {
    const raw = localStorage.getItem(CACHED_USER_KEY);
    return raw ? (JSON.parse(raw) as User) : null;
  } catch {
    return null;
  }
}

class AuthStore {
  user = $state<User | null>(null);
  loading = $state(false);
  initialized = $state(false);

  get isLoggedIn(): boolean {
    return this.user !== null;
  }

  get username(): string | null {
    return this.user?.username ?? null;
  }

  get level(): number {
    const lvl = userLevel(this.user);
    return Math.min(100, Math.max(0, lvl));
  }

  async init() {
    if (this.initialized) return;
    this.loading = true;
    try {
      // Try to restore from cached user
      const cached = readCachedUser();
      if (cached) {
        this.user = cached;
      }
      // Refresh from server if we have a token
      const token = getToken();
      if (token) {
        try {
          const me = await getMe();
          if (me.err === 0 && me.user) {
            this.user = me.user;
            cacheUser(me.user);
          }
        } catch {
          // Token expired or invalid — clear it
          setToken(null);
          this.user = null;
        }
      }
    } finally {
      this.loading = false;
      this.initialized = true;
    }
  }

  async login(username: string, password: string) {
    const res = await login(username, password);
    if (res.err === 0 && res.user) {
      this.user = res.user;
      cacheUser(res.user);
      return true;
    }
    return false;
  }

  async register(username: string, password: string, email?: string) {
    const res = await register(username, password, email);
    if (res.err === 0 && res.user) {
      this.user = res.user;
      cacheUser(res.user);
      return true;
    }
    return false;
  }

  logout() {
    logout();
    this.user = null;
  }
}

export const auth = new AuthStore();
```

### Breakdown

**Svelte 5 store**: The `AuthStore` class uses Svelte 5 runes (`$state`) for reactive state. The `.svelte.ts` extension tells SvelteKit to treat it as a Svelte module.

**Caching**: The user is cached in `localStorage` so the app can show the cached user even when offline. The cache is only written when a real auth response succeeds.

**Level fallback**: Fresh auth responses carry `level`, but cached/legacy users might not. The `userLevel` function falls back to the legacy role ladder: admin (role 10) → level 100, curator (role 5) → level 50, trusted (role 1) → level 1, regular → level 0.

**`init()`**: Called on app startup. Restores the cached user, then tries to refresh from the server. If the token is expired, it clears the token and sets the user to null.

**`login()` / `register()`**: Call the API, set the user on success, cache the user.

**`logout()`**: Clears the token and user.

---

## 8.8 Frontend: login and register pages

The login and register pages are in `frontend/src/routes/`. There are pages for:
- `/login` — login form
- `/register` — registration form

These pages use the `auth` store to handle form submission and show success/error states.

The key pattern in the login page:

```svelte
<!-- frontend/src/routes/login/+page.svelte (conceptual) -->
<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import { login } from '$lib/api/social';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit() {
    loading = true;
    error = '';
    try {
      const success = await auth.login(username, password);
      if (!success) {
        error = 'Invalid username or password';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<form onsubmit={handleSubmit}>
  <input type="text" bind:value={username} placeholder="Username" />
  <input type="password" bind:value={password} placeholder="Password" />
  <button type="submit" disabled={loading}>Log In</button>
  {#if error}
    <p class="error">{error}</p>
  {/if}
</form>
```

---

## 8.9 Try It Yourself: protect a route

Add a simple protected endpoint that only logged-in users can access.

### Step 1: Add the handler

In `src/routes/social.rs`, add:

```rust
/// GET /api/protected — only logged-in users can see this
pub async fn protected_handler(
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| {
        AppError::Unauthorized("You must be logged in to access this resource".to_string())
    })?;
    Ok(Json(json!({
        "err": 0,
        "message": format!("Hello, user {}!", user_id),
        "user_id": user_id,
    })))
}
```

### Step 2: Register the route

In `src/server.rs`, add:

```rust
.route("/api/protected", get(routes::social::protected_handler))
```

### Step 3: Test without auth

```bash
curl -s http://localhost:8000/api/protected
```

**Expected**: 401 Unauthorized.

### Step 4: Test with auth

First, register a user and get a token:

```bash
curl -s -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass123"}' | python3 -m json.tool
```

Copy the `token` from the response, then:

```bash
curl -s http://localhost:8000/api/protected \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" | python3 -m json.tool
```

**Expected**:

```json
{
    "err": 0,
    "message": "Hello, user 1!",
    "user_id": 1
}
```

### What you learned

- How the `AuthUser` extractor works.
- How to protect an endpoint.
- How to test auth with curl.

---

## 8.10 What you have now

- You understand registration: validation, bcrypt hashing, database insert, JWT creation.
- You understand honeypots and timing traps: silent bot rejection.
- You understand JWT: claims, encoding, verification, 30-day expiration.
- You understand refresh tokens: 365-day tokens for re-authentication.
- You understand the `AuthUser` extractor: custom Axum extractor, anonymous by default, Bearer token parsing.
- You understand the frontend auth store: Svelte 5 runes, localStorage caching, level fallback.
- You understand login/register pages: form submission, error handling.
- You added a protected endpoint and tested it with and without auth.

Next: Part 9 — Bookmarks. You will build bookmark creation, listing, deletion, and CSV import/export.

---

*End of Part 8. On to [Part 9 — Bookmarks](./09-bookmarks/09-bookmarks.md).*

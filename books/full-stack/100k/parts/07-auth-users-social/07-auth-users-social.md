# Part 7 — Authentication, Users & Social

> **Part 7 of 13** — Parts 1 through 6 built FicHub's guts: the scraper
> that reads stories off the internet, the export pipeline that turns
> them into EPUBs, the search engine that finds them, and the reader
> that displays them. Every one of those features treated every visitor
> as anonymous — the same URL in, the same files out, for everyone.
> This part adds the *people*.
>
> We cover the JWT session system in `src/routes/auth.rs` with its
> `Claims`, `create_token`/`verify_token` pair, and the `AuthUser`
> extractor that makes "is this request logged in?" a one-line question
> (Chapter 28); the users table, the role tiers 0/1/5/10, and the
> reputation-and-promotion system that grows a reader into a curator
> (Chapter 29); bookmarks, the 5-star `work_ratings`, and in-depth
> written reviews (Chapter 30); threaded comments and the three-layer
> moderation stack — heuristic, LLM triage, and the transparent modlog
> (Chapter 31); and finally the follows table, the updates feed, and the
> notifications inbox that turn a download tool into a community
> (Chapter 32).
>
> By the end of this part, you'll be able to trace one "like" from the
> frontend button all the way down to a `work_ratings` row, understand
> why FicHub keeps negative comments in the database while hiding them
> from the public, and know exactly which SQL guarantees that a user can
> never be spammed by a duplicate follow.

---

## Chapter 28 — Auth: JWT, `AuthUser`, and `create_token`/`verify_token`

Let's start with the single most important question in any web app:
**who is this request from?** Until Part 7, FicHub didn't care. The
meta endpoint, the export pipeline, the search engine — they all served
whoever showed up. But bookmarks, ratings, comments, follows, and
notifications are all *personal* data. A bookmark means nothing if we
can't tell whose shelf it's on. So before we build any of the social
features, we need identity.

FicHub's answer is the file `src/routes/auth.rs` — 230 lines that pack
in a lot of craft. Let's read it top to bottom, the way you'd read any
new module in a codebase you've just joined.

### 28.1 The contract: what does "logged in" mean?

First, the shape of the session itself:

```rust
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
    pub role: i16, // 0=regular, 1=trusted, 5=curator, 10=admin (see admin/users UI)
    pub exp: usize,
    pub iat: usize,
}
```

This is a *JWT claims struct* — the payload that travels inside every
token. Notice the comment right on the `role` field: `0=regular,
1=trusted, 5=curator, 10=admin`. We'll spend all of Chapter 29 on what
those numbers mean; for now, just note that the role is *baked into the
token itself*, not looked up from the database on every request. That's
a classic JWT trade-off that we'll come back to.

Also note the fields: `sub` (the JWT-standard "subject" — here, the
user id), `username` (so the API can greet you without a second query),
`exp` (expiry, seconds since epoch) and `iat` (issued-at, same units).
The two timestamps are `usize` — Unix seconds — because that's what
the `jsonwebtoken` crate expects.

💡 **Key Concept — JWT is a signed envelope, not a session cookie.**
A JSON Web Token is three base64url chunks joined by dots: a header, a
payload (our `Claims`), and a signature. The header and payload are
*not encrypted* — anyone can decode them and read `sub`, `username`,
and `role`. The signature is the security: it's computed over the
header + payload using a secret key, so a token whose payload was
tampered with fails verification instantly. That's why JWTs work on
stateless servers: FicHub doesn't need to store "who's logged in"
anywhere — the client carries the proof, and the server just checks the
signature. The catch, as we'll see, is that you can't revoke a token
before it expires. FicHub's tokens live 30 days — a deliberate balance
between convenience and the blast radius of a leaked token.

### 28.2 `create_token` and `verify_token`: the mint and the check

Next in the file come the two functions that the whole auth system
hinges on. First, the mint:

```rust
/// Create a JWT token for a user.
pub fn create_token(user: &User, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role,
        exp: (now + Duration::days(30)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
}
```

Five lines of claim-building, one `encode` call. The token lives for 30
days from right now; `iat` records when it was born. The `secret` comes
from the caller — we'll see in a moment that handlers pull it from the
`JWT_SECRET` environment variable.

And the check:

```rust
/// Verify a JWT token and return claims.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AppError::BadRequest(401, format!("Invalid token: {}", e)))
}
```

`decode` does three jobs at once: it checks the signature against the
secret, it checks `exp` (the `Validation::default()` enables expiry
validation), and it deserializes the payload into our `Claims` struct.
Any failure — wrong secret, expired token, malformed string — becomes
an HTTP 401 "Invalid token" error. There's a test for exactly that in
the same file:

```rust
#[test]
fn test_verify_wrong_secret() {
    let user = User {
        id: 1,
        username: "testuser".into(),
        role: 0,
        reputation: 0,
        email: None,
    };
    let token = create_token(&user, "secret1").unwrap();
    let result = verify_token(&token, "secret2");
    assert!(result.is_err());
}
```

⚠️ **Watch Out — `Validation::default()` is not "no validation".**
Some tutorials write custom `Validation` structs to skip checks, and
juniors often assume "default" means "lax". The opposite is true:
jsonwebtoken's `Validation::default()` validates the *exp* claim
(required), requires the `sub` claim for the HS algorithms, and enables
leeway. FicHub gets 30-day sessions *and* expiry enforcement from one
default. If you ever see code doing `Validation { validate_exp: false,
..Default::default() }`, a big red flag should go up — that's how
"logged in forever" tokens happen. Also note the secret handling: the
same secret must be used to encode and decode, and FicHub falls back to
a hard-coded `"fichub-dev-secret"` when `JWT_SECRET` is unset. That
fallback is fine for local development and dangerous in production —
one of the very first things you'd harden when deploying for real (we
visit this in Part 13).

### 28.3 The `User` struct and the registration SQL

The next chunk of the file defines what a user *is* on the wire, plus
the request/response shapes for the two auth flows:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: i16,
    pub reputation: i32,
    pub email: Option<String>,
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
}
```

This is where FicHub shows its battle scars. The `RegisterRequest` has
the two fields you'd expect — `username` and `password` — and then two
fields *no legitimate user will ever fill in*: `website` and
`form_opened_at`. These are the honey trap and the timing trap, and
they're your first real lesson in **defending against bots at the
application layer** rather than the network layer.

Then registration itself:

```rust
/// Register a new user.
pub async fn register_user(db: &PgPool, req: RegisterRequest, secret: &str) -> Result<AuthResponse, AppError> {
    if req.username.len() < 2 || req.username.len() > 32 {
        return Err(AppError::BadRequest(-1, "Username must be 2-32 characters".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::BadRequest(-1, "Password must be at least 6 characters".into()));
    }

    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Hash error: {}", e)))?;

    let row = sqlx::query_as::<_, (i32, String, i16, i32, Option<String>)>(
        "INSERT INTO users (username, password_hash, email) VALUES ($1, $2, COALESCE($3, ''))
         RETURNING id, username, role, reputation, email",
    )
    .bind(&req.username)
    .bind(&hash)
    .bind(&req.email)
    .fetch_one(db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::BadRequest(-1, "Username or email already taken".into())
        }
        _ => AppError::Database(e.to_string()),
    })?;

    let user = User {
        id: row.0,
        username: row.1.clone(),
        role: row.2,
        reputation: row.3,
        email: row.4,
    };
    let token = create_token(&user, secret)?;
    Ok(AuthResponse { token, user })
}
```

Read that `INSERT ... RETURNING` line carefully — it's a pattern
you'll see everywhere in FicHub. The database inserts the row *and*
hands back the generated columns (`id`, plus the defaults for `role`,
`reputation`, `email`) in one round trip. No second `SELECT` needed.
The `COALESCE($3, '')` is a nice defensive touch: the column is
nullable, so a `None` email becomes an empty string instead of NULL.

💡 **Key Concept — never store a password, store a hash.**
`bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)` runs the password
through bcrypt with the default cost factor (12 iterations), producing
a salted, one-way digest. "One-way" is the whole point: even if the
`users` table leaks, the stored `password_hash` values cannot be
reversed into passwords. Every serious password framework has this
built in (Argon2id and scrypt are the modern alternatives); what you
must *never* do is store raw passwords, unsalted MD5/SHA1 digests, or
your own homemade "encryption". Notice also the error mapping: a
`unique_violation` from the database (the `username`/`email` UNIQUE
constraints we met in migration 001) becomes a friendly
"Username or email already taken" instead of a 500.

### 28.4 Login: same shape, different SQL

Login is registration's mirror image — fetch the row by username, then
verify:

```rust
/// Login a user with username + password.
pub async fn login_user(db: &PgPool, req: LoginRequest, secret: &str) -> Result<AuthResponse, AppError> {
    let row = sqlx::query_as::<_, (i32, String, String, i16, i32, Option<String>)>(
        "SELECT id, username, password_hash, role, reputation, email FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::BadRequest(401, "Invalid username or password".into()))?;

    let (id, username, hash, role, reputation, email) = row;

    let valid = bcrypt::verify(&req.password, &hash)
        .map_err(|e| AppError::Internal(format!("Verify error: {}", e)))?;
    if !valid {
        return Err(AppError::BadRequest(401, "Invalid username or password".into()));
    }

    let user = User { id, username, role, reputation, email };
    let token = create_token(&user, secret)?;
    Ok(AuthResponse { token, user })
}
```

Two details worth your attention. First, `fetch_optional` + `ok_or_else`
turns "no such user" into the exact same 401 as "wrong password" — the
classic **user-enumeration defense**. The error message is identical
either way, so an attacker can't probe which usernames exist by
comparing responses. Second, the SELECT includes `password_hash` but the
response `User` struct never carries it — it's destructured into a
local `hash` variable that dies at the end of the function. The hash
travels from the database to `bcrypt::verify` and nowhere else.

⚠️ **Watch Out — the timing leak you can't see.**
The user-enumeration message is identical, but the *time* differs:
looking up a nonexistent user is a fast query, while verifying a
password against a real hash takes ~100ms of bcrypt work. A determined
attacker can measure that. Real systems paper over it by always running
a dummy bcrypt verify when the user doesn't exist. FicHub doesn't do
that here — and that's okay, because auth requests are rate-limited to
10/minute/IP, which we'll see in the handler. Layered defenses: the
message hides existence, the rate limit makes timing attacks impractical.

### 28.5 The `AuthUser` extractor: auth as a type

Now the piece that makes all of Part 7's handlers pleasant to read.
Every handler that needs to know "who is this?" declares an `AuthUser`
parameter, and Axum fills it in:

```rust
/// Axum extractor that reads the Bearer token from the Authorization header.
/// If no token is present or it's invalid, user_id is None (anonymous).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub role: i16,
}

impl Default for AuthUser {
    fn default() -> Self {
        Self { user_id: None, username: None, role: 0 }
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
                    });
                }
            }
        }

        Ok(AuthUser::default())
    }
}
```

This is Axum's extractor pattern, and it's the cleanest "optional auth"
design I know. The key insight is `type Rejection = ();` — the extractor
*never fails*. A request with no header, a garbage header, or an expired
token all produce the same result: `AuthUser::default()` with
`user_id: None`. The handler decides what "anonymous" means for its
route.

💡 **Key Concept — extractors turn plumbing into type signatures.**
In Axum, an extractor is any type implementing `FromRequestParts` (for
things taken from the request head) or `FromRequest` (for bodies). By
implementing it for `AuthUser`, we've taught Axum to run the whole
token-verification dance *before our handler body executes* — and any
future handler gets authentication by merely listing the type in its
parameter list. This is dependency injection, Rust style: the framework
constructs your arguments; you just describe what you need. If FicHub
ever wanted a *required* login variant, it would be a second struct
with `type Rejection = AuthError` whose `from_request_parts` errors out
when the user is anonymous.

There's a subtle performance note in the code, too: `JWT_SECRET` is
read from the environment *on every request* rather than cached in
`AppState`. The comment explains why that's fine — environment reads
are cheap (the OS caches them) — and it means the secret can be rotated
by restarting with a new env var, no code change. A reasonable call for
a small app; for high-traffic services you'd cache it in state and add
key rotation logic.

### 28.6 The handlers: rate limits, honeypots, and the auth tier

`auth.rs` contains the pure functions; the HTTP handlers live in
`src/routes/social.rs` (the file that will also host bookmarks and
ratings). Registration looks like this:

```rust
/// POST /api/v1/auth/register
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    // ── Tiered rate limit (auth tier: 10/min per IP) ──────────────────
    enforce_auth_rate_limit(&state).await?;

    // ── Honeypot + timing trap (silent rejection) ──────────────────────
    // A bot that fills the hidden `website` field, or submits without the
    // JS-set `form_opened_at` (or impossibly fast), gets a success-looking
    // response and no account. See `crate::routes::honeypot`.
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
        // Success-looking response with no token — the bot "registered"
        // without creating anything.
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

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    let result = auth::register_user(&state.db, body, &secret).await?;
    Ok(Json(json!({
        "err": 0,
        "token": result.token,
        "user": result.user,
    })))
}
```

Read that silent-rejection branch once more, because it's a genuinely
clever pattern. A bot that fills the hidden `website` field (honeypot)
or submits too fast (timing trap) gets... a *successful-looking*
response. `err: 0`, a fake user object, an empty token. The bot
believes it registered. Nothing was created. The bot learns nothing,
because the response looks exactly like success — a real user who
trips the timing trap (say, by autofilling and submitting in under
500ms) sees the same shape and just retries. This is the opposite of
CAPTCHAs: instead of challenging bots, you *absorb* them. The
`MIN_FORM_MS` constant lives in `src/routes/honeypot.rs`:

```rust
/// rejected silently — a human cannot type a comment or fill a
/// registration form that fast. The frontend forms never submit this
/// early, so no legitimate user is ever affected.
pub const MIN_FORM_MS: u64 = 500;
```

And the rate limiter behind it all:

```rust
/// Enforce the auth-tier rate limit (10/min per IP by default) for
/// login/register. In test mode (`dynamic_rate_limit == false`) this is a
/// no-op static delay, so existing honeypot/auth integration tests are
/// unaffected. Redis errors fail open — a Redis hiccup never blocks auth.
async fn enforce_auth_rate_limit(state: &Arc<AppState>) -> Result<(), AppError> {
    let ip = crate::limiter::client_ip_from_headers(
        None,
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    match state
        .rate_limiter
        .check(ip, None, Tier::Auth)
        .await
    {
        crate::limiter::TieredRateLimitResult::Wait(secs) => {
            Err(AppError::RateLimited(secs))
        }
        crate::limiter::TieredRateLimitResult::Allowed => Ok(()),
    }
}
```

`Tier::Auth` is one rung of a **tiered rate limiter** — different
routes get different budgets, and the limiter itself lives in
`src/limiter/` (Redis-backed, which is why the comment mentions
fail-open behavior). Login/register is capped at 10 attempts per minute
per IP — enough for a human retyping a password, nowhere near enough
for a credential-stuffing bot.

⚠️ **Watch Out — fail-open vs. fail-closed.**
`Redis errors fail open — a Redis hiccup never blocks auth.` That's a
deliberate availability-vs-security trade-off. If the rate limiter's
backing store is down and we *fail closed* (reject everything), a Redis
outage becomes a full site outage for logins. FicHub chooses to let
requests through instead — authentication still works because bcrypt
and JWT verification don't need Redis at all. The cost: during a Redis
outage, brute-force protection is temporarily offline. For a fanfic
platform, that's the right call; for a bank, you'd flip it.

The login handler is where things get interesting in a different way —
it *logs its own failures*. When `auth::login_user` returns an error,
the handler records the attempt into `request_log` before re-returning
the error:

```rust
    match auth::login_user(&state.db, body, &secret).await {
        Ok(result) => Ok(Json(json!({
            "err": 0,
            "token": result.token,
            "user": result.user,
        }))),
        Err(e) => {
            // Log the failed auth (bot stuffing signal) — best-effort, never
            // masks the real error. etype='auth_failed' is aggregated by
            // bot-scorer into bot_scores.failed_auths.
            let client_id = headers
                .get("x-client-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let user_agent = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let ip = crate::limiter::client_ip_from_headers(
                headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
                "127.0.0.1".parse().expect("static ip"),
            );
            // request_log.source_id has an FK to request_source; ensure the
            // 'auth' source row exists (idempotent) before logging.
            let auth_source_id: i64 = match sqlx::query_scalar::<_, i64>(
                r#"INSERT INTO request_source (is_automated, route, description)
                   SELECT false, '/api/auth/login', 'auth login attempts'
                   WHERE NOT EXISTS (SELECT 1 FROM request_source WHERE route = '/api/auth/login')
                   RETURNING id"#,
            )
            .fetch_optional(&state.db)
            .await
            {
                Ok(Some(id)) => id,
                _ => sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM request_source WHERE route = '/api/auth/login' LIMIT 1",
                )
                .fetch_one(&state.db)
                .await
                .unwrap_or(1),
            };
            if let Err(e) = crate::db::queries::insert_request_log(
                &state.db,
                auth_source_id,
                "auth_failed",
                "login",
                0,
                None, None, None, None, None, None,
                client_id.as_deref(),
                user_agent.as_deref(),
                Some(ip),
            )
            .await
            {
                tracing::warn!("failed to log auth_failed: {e}");
            }
            Err(e)
        }
    }
```

There's a lot to unpack here, but the through-line is: **failures are
data**. Every failed login becomes a `request_log` row with
`etype='auth_failed'`, tagged with client id, user agent, and IP. Later
(Part 11, the analytics part) that data feeds a bot-scorer that computes
`bot_scores.failed_auths` and decides who gets shadowbanned or a
proof-of-work challenge. The handler even shows you the idempotency
dance for `request_source` — an `INSERT ... WHERE NOT EXISTS` that
creates the "auth login attempts" source row once, then reuses it.

The last endpoint of the trio is the delightful `me` handler, which
shows exactly how cheap "who am I?" becomes once the extractor exists:

```rust
/// GET /api/v1/auth/me — return the current user from token
pub async fn me_handler(
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    match auth.user_id {
        Some(id) => Ok(Json(json!({
            "err": 0,
            "user": {
                "id": id,
                "username": auth.username,
                "role": auth.role,
            }
        }))),
        None => Ok(Json(json!({ "err": 401, "msg": "Not authenticated" }))),
    }
}
```

No database, no secret lookup — the extractor already did the work.
That's the entire auth flow, end to end: register (hash + insert +
token), login (lookup + verify + token), and every subsequent request
carries `Authorization: Bearer <token>`, which any handler can inspect
by declaring `auth: AuthUser`.

### 28.7 The test suite: tokens and passwords, no database needed

`auth.rs` closes with a test module that's a great model for testing
crypto without a database:

```rust
#[test]
fn test_create_and_verify_token() {
    let user = User {
        id: 1,
        username: "testuser".into(),
        role: 0,
        reputation: 0,
        email: None,
    };
    let secret = "test-secret";
    let token = create_token(&user, secret).unwrap();
    let claims = verify_token(&token, secret).unwrap();
    assert_eq!(claims.sub, 1);
    assert_eq!(claims.username, "testuser");
    assert_eq!(claims.role, 0);
}

#[test]
fn test_expired_token_rejected() {
    // Create a token with a past expiry by crafting claims manually
    let claims = super::super::auth::Claims {
        sub: 1,
        username: "test".into(),
        role: 0,
        exp: 1, // Jan 1 1970 — expired
        iat: 1,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"test-secret"),
    ).unwrap();
    let result = verify_token(&token, "test-secret");
    assert!(result.is_err());
}
```

`test_expired_token_rejected` is my favorite: instead of waiting 30
days for a token to expire, it *crafts* an already-expired token by
building `Claims` directly with `exp: 1`. That's the whole trick of
testing time-based logic — don't sleep, manufacture the condition.

🧪 **Try It Yourself — the auth round trip with curl.**
Start FicHub locally (`cargo run` with a dev database, Part 3), then
open a second terminal and watch identity happen:

```bash
# Register — you should get back a token and a user object
curl -s -X POST http://localhost:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username": "reader1", "password": "hunter22"}'

# Login — same shape
curl -s -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username": "reader1", "password": "hunter22"}'

# Now the money shot: /api/auth/me with and without the token
curl -s http://localhost:8080/api/auth/me                      # err: 401
curl -s http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <TOKEN_FROM_LOGIN>"                # your user
```

Then decode the token to see the claims with your own eyes:

```bash
# Split the JWT on dots and base64-decode the middle chunk
echo '<TOKEN>' | cut -d. -f2 | base64 -d 2>/dev/null; echo
# → {"sub":1,"username":"reader1","role":0,"exp":... ,"iat":...}
```

You'll see your `sub`, `username`, `role` — all readable, all signed.
Now edit one character of the token and retry `me`; you'll get a 401,
because the signature no longer matches. That's JWT in a nutshell.

---

Let's check our progress before moving on. We have identity: a token
that proves "you are user 7, role 0, username reader1" for 30 days.
But so far every user is identical — role 0, zero reputation. The next
chapter turns that flat table of users into a community with a ladder:
the role tiers that decide who can hide a comment, and the reputation
system that decides who climbs.

---

## Chapter 29 — Users, Roles, and Reputation: From Reader to Curator

In Chapter 28 we minted tokens for users — but all users were born
equal: `role: 0`, `reputation: 0`. That's fine for a bookmark feature,
but FicHub is a *community*. Communities need trust, and trust needs
tiers: a brand-new account probably shouldn't be able to hide other
people's comments or approve uploads. This chapter is about the ladder
that separates a lurker from a curator, and the mechanics that let
people climb it.

### 29.1 The users table: where identity lives

Everything about a user's standing is stored in one row of the `users`
table, created way back in migration 001:

```sql
-- User roles: 0=reader, 1=curator, 2=senior_curator, 3=admin
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE,
    role SMALLINT NOT NULL DEFAULT 0,
    reputation INT4 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    curator_since TIMESTAMPTZ,
    curator_status TEXT DEFAULT 'active' CHECK (curator_status IN ('active','suspended'))
);

CREATE INDEX IF NOT EXISTS idx_users_reputation ON users(reputation DESC);
```

I want you to notice two things about this table. First, the comment
above it describes the role system as `0=reader, 1=curator,
2=senior_curator, 3=admin` — but by the time of the code we read in
Chapter 28, the comment on `Claims.role` says `0=regular, 1=trusted,
5=curator, 10=admin`. **The schema drifted from the code.** The
database constraint never enforced the meaning of the numbers — `role`
is just a `SMALLINT` — so the *semantics* moved (a 3-tier ladder became
a 4-tier ladder with wider spacing) without a migration. That's the
reality of a real codebase: comments are documentation, the code is the
truth, and a lint pass for "comment says X, code says Y" is a luxury
most projects never build. The important part is that the *tier numbers
grow with power*: any check in the code is `if role >= N`, never
`if role == N`. That's what lets new tiers be inserted later without
rewriting every guard.

Second, the `CHECK (curator_status IN ('active','suspended'))` — the
database itself refuses bad values. We'll come back to this pattern
again and again in FicHub: **constraints are the last line of defense**
between the app and garbage data.

### 29.2 The role ladder: 0, 1, 5, 10

Let's look at how the code actually uses these numbers. From
`src/routes/comments.rs`, the delete-comment guard:

```rust
    // Check ownership or curator role
    let comment: Option<(Option<i32>, i16)> = sqlx::query_as(
        "SELECT c.user_id, u.role FROM comments c
         LEFT JOIN users u ON u.id = $2
         WHERE c.id = $1",
    )
    .bind(comment_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match comment {
        Some((Some(uid), _)) if uid == user_id => {}
        Some((_, role)) if role >= 1 => {} // Curator or above
        _ => return Err(AppError::BadRequest(403, "Not authorized".into())),
    }
```

This is the classic **ownership-or-privilege** pattern: you may delete a
comment if you wrote it, OR if you're a curator or above (`role >= 1`).
The query is doing something sneaky — it fetches *your* role (via
`LEFT JOIN users u ON u.id = $2`) in the same query that fetches the
comment's owner. One round trip, two answers.

And the hide-comment guard, one step stricter:

```rust
pub async fn hide_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(comment_id): Path<i64>,
    Json(body): Json<HideCommentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    // Check curator role
    let role: Option<i16> = sqlx::query_scalar(
        "SELECT role FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match role {
        Some(r) if r >= 1 => {} // Curator or above
        _ => return Err(AppError::BadRequest(403, "Curator role required".into())),
    }
```

Deleting your own comment is a right; hiding someone else's is a
*privilege*. Notice the interesting difference between the two checks:
the delete check reads `role` from the token indirectly (the JOIN),
while the hide check does a fresh `SELECT role FROM users WHERE id = $1`.
Why the inconsistency? The `Claims` struct *embeds* the role in the
JWT, so the `AuthUser` extractor already has it... but a role baked
into a 30-day token goes stale. If a user is promoted (or demoted!) by
an admin, their old token still claims the old role for up to 30 days.
For a *privileged* action like hiding, FicHub re-checks the database —
the fresh truth. For lower-stakes checks it trusts the token. That's a
really important security nuance:

💡 **Key Concept — token claims are cached; the database is truth.**
JWTs make `role` available in 0 round trips, which is great for
performance, but any claim baked into a token is a snapshot of the
moment it was minted. The pattern in FicHub: *use the token's role for
convenience (UI hints, low-stakes gates), re-query the database for
anything that actually changes trust*. A demoted curator with an old
token can still *see* curator UI for a while, but cannot *perform*
curator actions. If you take one security lesson from this chapter,
take this one: never let a stale token authorize an action you couldn't
undo.

The admin tier (role 10) gates the whole `src/routes/admin.rs` module.
Every handler there opens with the same two lines:

```rust
/// GET /api/admin/moderation/queue — list pending manual uploads
pub async fn mod_queue(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
```

`if user.role < 10` — not `if user.role != 10`. Again the
greater-than-or-equal philosophy. And note that admin handlers read
`user.role` straight from the `AuthUser` extractor — the token's copy —
which for an admin-only module is a slight relaxation of the
"database is truth" rule. In practice, admin tokens are short-lived
enough and the admin surface is logged enough (remember the modlog from
Chapter 28's handler? we'll see it in Chapter 31) that FicHub accepts
the staleness window. Every design is a trade.

Admin actions go through the modlog — let's watch role changes in
action, because this is the *only* place roles are ever changed by
hand:

```rust
/// PUT /api/admin/users/{id}/role — change user role
pub async fn set_user_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let new_role: i16 = payload
        .get("role")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16)
        .ok_or_else(|| AppError::BadRequest(-1, "role field required (0,1,5,10)".into()))?;

    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(new_role)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "set_user_role",
        "user",
        &user_id.to_string(),
        vec![("role", json!(new_role))],
    )
    .await;

    Ok(Json(json!({"err": 0, "msg": "Role updated"})))
}
```

Three things to note. First, the error message itself documents the
vocabulary: `"role field required (0,1,5,10)"`. Second, the update is
just `UPDATE users SET role = $1 WHERE id = $2` — one line — because
the *policy* (who may change roles) lives in the handler guard and the
*record* (who changed what) lives in the modlog call. Third, `record_json`
passes the new role as detail data — so the audit trail shows *what*
the role was changed to. That's the transparency pattern we'll explore
fully in Chapter 31.

⚠️ **Watch Out — privilege checks belong in the handler, not the
frontend.** `if user.role < 10` appears in *every* admin handler even
though the frontend already hides admin buttons from non-admins. This
is defense in depth: the UI hiding a button is a courtesy, not a
security boundary. Anyone can curl the API. The server must enforce
every gate, every time. When you're writing your own admin features,
remember: the frontend's job is to make the API *friendly*; the
backend's job is to make it *safe*. Never skip the backend check
"because the button is hidden anyway."

### 29.3 Reputation: the currency of trust

Roles are granted by admins, but FicHub also grows curators *organically*
— through reputation. The schema has a dedicated audit table for it:

```sql
-- Reputation audit trail
CREATE TABLE IF NOT EXISTS reputation_events (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    points INT4 NOT NULL,
    reference_type TEXT,
    reference_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rep_events_user ON reputation_events(user_id, created_at DESC);
```

Reputation is a number on the user row, but *reputation_events* is the
story of how it got there. Every point has a provenance: `event_type`
(like `upload_approved` or `curation_approve`), a reference to what
caused it, and a timestamp. That's the difference between a "score" and
an *auditable* score — and later we'll see badges and quests built on
exactly this table.

The core function lives in `src/db/queries.rs`, and its name tells the
whole story:

```rust
// ── Reputation & Auto-Promotion ─────────────────────────────────────

/// Award reputation to a user and check for auto-promotion to curator
pub async fn update_reputation_and_promote(
    pool: &PgPool,
    user_id: i32,
    delta: i32,
    event_type: &str,
) -> AppResult<()> {
    // Update reputation
    sqlx::query("UPDATE users SET reputation = reputation + $1 WHERE id = $2")
        .bind(delta)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Record reputation event
    sqlx::query(
        "INSERT INTO reputation_events (user_id, event_type, delta)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(delta)
    .execute(pool)
    .await?;

    // Check for auto-promotion to curator (reputation >= 100, role = user)
    let row: Option<(i32, String)> = sqlx::query_as(
        "SELECT reputation, role FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some((reputation, role)) = row {
        if reputation >= 100 && role == "user" {
            // Promote to curator
            sqlx::query(
                "UPDATE users SET role = 'curator', curator_since = NOW() WHERE id = $1",
            )
            .bind(user_id)
            .execute(pool)
            .await?;

            // Record promotion event
            sqlx::query(
                "INSERT INTO reputation_events (user_id, event_type, delta)
                 VALUES ($1, 'curator_promoted', 0)",
            )
            .bind(user_id)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
```

This function is a beautiful little state machine disguised as a
utility. Note the drift again: it checks `role == "user"` and sets
`role = 'curator'` — *string* roles — while the rest of the codebase
compares numeric roles. Another artifact of the schema evolution we saw
in 29.1; this function predates the numeric tier rework. In a fresh
codebase you'd fix this; in a real one you'd file it and move on,
knowing the numeric system wins where it matters (the guards).

💡 **Key Concept — idempotent promotion, one event per promotion.**
The promotion guard `reputation >= 100 && role == "user"` only fires
*once*: after promotion, the role is no longer `"user"`, so re-running
the function never double-promotes or double-logs. That's the
idempotency pattern for state transitions: *make the condition
self-consuming*. If the guard were `reputation >= 100` alone, every
subsequent reputation award would re-trigger the promotion branch and
spam `curator_promoted` events. Whenever you write a "fire when X"
block, ask: what stops it from firing again on the next call? If the
answer is "nothing", it's a bug waiting for a customer.

Where does the reputation actually *come* from? Look at the admin
approve-upload flow — a perfect example of the reward loop:

```rust
    sqlx::query("UPDATE works SET is_visible = TRUE WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    // Award uploader XP for approval
    let _ = crate::db::queries::update_reputation_and_promote(&state.db, uploader_id, 25, "upload_approved").await;
    let _ = crate::db::queries::check_and_award_badges(&state.db, uploader_id, "upload_approved").await;

    crate::modlog::record(&state.db, user.user_id, user.username.clone(), "approve_upload", "work", &work_id.to_string(), serde_json::json!({})).await;
```

Upload a fic → admin approves → **+25 reputation** and a badge check,
with every step logged to the modlog. The `let _ =` prefix is a
deliberate choice: the approval itself must succeed even if the
reputation write fails (the upload is already visible at that point —
failing the whole request would be wrong). Best-effort rewards.

### 29.4 Badges: reputation's visible skin

Reputation is a number; badges are a story. The badge definitions live
seeded in migration 003:

```sql
INSERT INTO badge_definitions (badge_type, name, description, icon, category, threshold, event_type) VALUES
    ('curator_apprentice', 'Curator Apprentice', 'Had 10 curation proposals approved', '🔰', 'curation', 10, 'curation_approve'),
    ('senior_curator', 'Senior Curator', 'Had 50 curation proposals approved', '⭐', 'curation', 50, 'curation_approve'),
    ('master_curator', 'Master Curator', 'Had 200 curation proposals approved', '👑', 'curation', 200, 'curation_approve'),
    ('reader_10', 'Bookworm', 'Read 10 works', '📖', 'reading', 10, 'work_read'),
    ('reader_100', 'Bibliophile', 'Read 100 works', '📚', 'reading', 100, 'work_read'),
    ('commenter_10', 'Chatty', 'Posted 10 comments', '💬', 'social', 10, 'comment_post'),
    ('first_bookmark', 'First Favourite', 'Bookmarked your first work', '💝', 'social', 1, 'bookmark_add'),
    ('collector_50', 'Collector', 'Bookmarked 50 works', '📑', 'social', 50, 'bookmark_add'),
ON CONFLICT (badge_type) DO NOTHING;
```

Look at the shape of a badge definition: a `threshold` and an
`event_type`. That's the entire badge logic in data form — **a badge
definition is just "count events of type X; when the count passes
threshold T, award it."** The code that consumes it is generic and
reusable:

```rust
/// Check and award badges based on event count
pub async fn check_and_award_badges(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
) -> AppResult<Vec<String>> {
    let mut awarded = Vec::new();

    // Get badge definitions that track this event type
    let badges = sqlx::query_as::<_, BadgeDefinition>(
        "SELECT badge_type, name, description, icon, category, threshold, event_type, created_at
         FROM badge_definitions WHERE event_type = $1 ORDER BY threshold",
    )
    .bind(event_type)
    .fetch_all(pool)
    .await?;

    if badges.is_empty() {
        return Ok(awarded);
    }

    // Count user's events of this type
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM reputation_events WHERE user_id = $1 AND event_type = $2",
    )
    .bind(user_id)
    .bind(event_type)
    .fetch_one(pool)
    .await?;

    for badge in &badges {
        if count.0 >= badge.threshold as i64 {
            if award_badge(pool, user_id, &badge.badge_type).await? {
                awarded.push(badge.badge_type.clone());
            }
        }
    }

    Ok(awarded)
}
```

Two queries, a loop, done. Adding a brand-new badge is a *data* change
— insert a row into `badge_definitions` — not a code change. That's the
payoff of pushing policy into the database. (And the `reputation_events`
table does double duty: it's both the audit trail *and* the badge
counter. One source of truth.)

⚠️ **Watch Out — the events table is the counter, so events must be
append-only.** Badge logic counts `reputation_events` rows. If any code
path ever *deleted* or *updated* an event row, badge counts would
silently change. That's why `reputation_events` has no UPDATE path in
the codebase — it's a ledger. When you design an audit table, design
for append-only from day one: no updates, no deletes, and any "fix a
mistake" operation becomes a *compensating event* (+25 then −25)
rather than an edit. Ledgers work because they can't lie about the
past.

### 29.5 The leaderboard and the public profile

Reputation becomes visible in two places. The first is the curator
leaderboard — a single query with an ORDER BY, the whole feature:

```rust
/// GET /api/v1/leaderboard/curators — top curators by reputation
pub async fn leaderboard_curators_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT id, username, reputation FROM users
         WHERE reputation > 0 ORDER BY reputation DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await?;

    let entries: Vec<Value> = rows.into_iter().map(|(id, username, reputation)| {
        json!({
            "id": id,
            "username": username,
            "reputation": reputation,
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "leaderboard": entries })))
}
```

And the second is the public profile page, which shows how a profile
handler composes several small queries (the user row, then a badge
count) into one response:

```rust
/// GET /api/v1/users/{id} — get a user's public profile
pub async fn user_profile_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query_as::<_, (i32, String, i16, i32, Option<String>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, username, role, reputation, email, created_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let (id, username, role, reputation, email, created_at) = row;

    // Get badge count
    let badge_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_badges WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "user": {
            "id": id,
            "username": username,
            "role": role,
            "reputation": reputation,
            "email": email,
            "created_at": created_at.to_rfc3339(),
            "badges": badge_count.0,
        }
    })))
}
```

Note the shape of the response — the `user` object carries `reputation`
and `badges` but no `password_hash` (good) and no internal flags like
`curator_status` (also good, though `email` is arguably a privacy
question — a public profile endpoint probably shouldn't leak emails,
but FicHub chose to include it; privacy is a product decision, not a
code accident, and you should make it *deliberately* in your own apps).

The admin user listing gives us the full row-level picture of what
admins see — and notice it reads the columns we've met across three
migrations (role, reputation, `total_words_read` from migration 003,
`is_banned` and `locale` from later ones):

```rust
    let rows = sqlx::query_as::<_, (i32, String, i16, i32, Option<i64>, bool, Option<String>)>(
        r#"SELECT id, username, role, reputation, total_words_read, is_banned, locale
           FROM users ORDER BY id LIMIT $1 OFFSET $2"#
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;
```

And the search variant uses `ILIKE` for case-insensitive username
search — a pattern you'll use constantly for admin "find the user"
boxes:

```rust
    if let Some(ref q) = params.q {
        let rows = sqlx::query_as::<_, (i32, String, i16, i32, Option<i64>, bool, Option<String>)>(
            r#"SELECT id, username, role, reputation, total_words_read, is_banned, locale
               FROM users
               WHERE username ILIKE $1
               ORDER BY id
               LIMIT $2 OFFSET $3"#
        )
        .bind(format!("%{}%", q))
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?;
```

💡 **Key Concept — ILIKE: the case-insensitive LIKE.**
`WHERE username ILIKE '%mor%'` matches "Morgan", "morgana", and
"MORIARTY" alike. PostgreSQL's `ILIKE` is `LIKE` with case folding —
and the `%` wildcards on *both* sides make it a substring match (a
"contains" search) rather than a prefix search. It's the workhorse of
admin search UIs. The cost: `%x%` can't use a normal btree index, so
it's a full scan — fine for an admin tool on a few thousand users,
painful on millions. That's why FicHub limits it to admin endpoints and
paginates hard (LIMIT/OFFSET everywhere).

### 29.6 Banning: the role system's emergency brake

The last piece of the user-management puzzle is the ban toggle — the
one role-related state *not* encoded in the role number:

```rust
/// PUT /api/admin/users/{id}/ban — toggle ban
pub async fn toggle_ban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let banned: bool = payload
        .get("is_banned")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::BadRequest(-1, "is_banned field required".into()))?;

    sqlx::query("UPDATE users SET is_banned = $1 WHERE id = $2")
        .bind(banned)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        if banned { "ban_user" } else { "unban_user" },
        "user",
        &user_id.to_string(),
        vec![],
    )
    .await;

    Ok(Json(json!({"err": 0, "msg": if banned { "User banned" } else { "User unbanned" }})))
}
```

Interesting design choice: `is_banned` is a boolean *column*, separate
from `role`. A banned user keeps their role (so the ban can be lifted
without re-granting anything) but is flagged. The handler even picks
the modlog action name dynamically: `if banned { "ban_user" } else
{ "unban_user" }`. One handler, two auditable actions.

🧪 **Try It Yourself — walk the ladder.**
With FicHub running and a JWT in hand from Chapter 28:

```bash
# 1. Register two users and note their roles (both 0)
curl -s -X POST http://localhost:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username": "curator_wannabe", "password": "secret123"}'

# 2. Check your own role via /api/auth/me — role is 0
curl -s http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer <your_token>"

# 3. Try a curator action with role 0 — expect 403
curl -s -X PATCH http://localhost:8080/api/comment/1/hide \
  -H "Authorization: Bearer <your_token>" \
  -H 'Content-Type: application/json' -d '{"hidden": true}'
# → {"err":403,"msg":"Curator role required"}

# 4. As the admin user (role 10, seeded in dev), promote yourself:
curl -s -X PUT http://localhost:8080/api/admin/users/1/role \
  -H "Authorization: Bearer <admin_token>" \
  -H 'Content-Type: application/json' -d '{"role": 1}'
# → {"err":0,"msg":"Role updated"}

# 5. Peek at the modlog to see the promotion recorded:
curl -s http://localhost:8080/api/modlog -H "Authorization: Bearer <admin_token>"
```

You just watched the entire trust ladder in action: the 403 proves the
gate works, the role update proves the gate is openable, and the
modlog proves every step is remembered. That's role management done
right.

---

We now have users who can be trusted — or at least, whose trust level
is a number we can check. Time to give them something to *do* with
that identity. Bookmarks, ratings, and reviews are the first social
features that use everything we've built: the token proves who you are,
and the role tells us what you're allowed to do. Let's build the
shelf.

---

## Chapter 30 — Bookmarks, Ratings, and Reviews: Building the Shelf

Now we get to build the features that make FicHub feel like *yours*.
Three related concepts, three different granularities:

- **Bookmarks** — a binary "I saved this" signal, with optional private
  notes. (Do I want to read this? Is this on my list?)
- **Ratings** — a 1–5 star number. (How good was it, in one click?)
- **Reviews** — a written, titled, rated essay. (Here's *why* it's
  good, in 8,000 characters or less.)

All three live around the `works` table we built in Part 4, all three
require an authenticated user, and all three share one fascinating
schema decision that we need to understand before writing any code:
**everything is keyed by `url_id` AND `work_id`.**

### 30.1 The schema: why `url_id` and `work_id` both?

Here's the bookmark table from migration 001:

```sql
-- Bookmarks (user favorites)
CREATE TABLE IF NOT EXISTS bookmarks (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL,
    work_id INTEGER REFERENCES works(id),
    notes TEXT DEFAULT '',
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, url_id)
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_user ON bookmarks(user_id, created_at DESC);
```

And the ratings table right below it:

```sql
-- Ratings (like/dislike)
CREATE TABLE IF NOT EXISTS work_ratings (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL,
    work_id INTEGER REFERENCES works(id),
    rating SMALLINT NOT NULL,  -- 1 = like, -1 = dislike
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, url_id)
);
```

Remember the mental model from Part 4: a **work** is the canonical
story (one row in `works`), and a **url_id** is a *specific copy* of
that story on a *specific platform* (one row in `fic_info`, keyed by
its 12-hex hash). FicHub merges "the same fic on AO3 and on FFN" into
one work — but a reader might have bookmarked the AO3 copy *and* the
FFN copy before they were merged.

That's why the unique constraint is `UNIQUE(user_id, url_id)`: one
bookmark per user per *copy*. The `work_id` column then links the
bookmark to the canonical story so queries like "all my bookmarks"
(which don't care about copies) can join through. It's a
denormalization — both identifiers stored because each serves a
different query — and it's the kind of schema you only design after
living with a real merge problem.

💡 **Key Concept — one row, two identifiers, two jobs.**
`user_id + url_id` is the *identity* of the bookmark (what makes it
unique — the UNIQUE constraint enforces this). `work_id` is the
*meaning* of the bookmark (what story it points at, across platforms).
When a schema stores both, you're saying: "this user bookmarked this
specific copy, and that copy belongs to this canonical story." The
handler code has to keep both in sync, which is why every social insert
first *resolves* the url_id for a given work_id:

```rust
/// Resolve the `url_id` (fic_info.id) for a work id, preferring the work's
/// default source. Exposed for handlers that insert bookmarks directly
/// (`bookmarks.url_id` is NOT NULL).
pub async fn resolve_url_id_for_work(pool: &PgPool, work_id: i32) -> AppResult<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT COALESCE(
              (SELECT fi.id FROM fic_info fi WHERE fi.work_id = w.id AND fi.id = w.default_source_id LIMIT 1),
              (SELECT fi.id FROM fic_info fi WHERE fi.work_id = w.id LIMIT 1)
          )
          FROM works w WHERE w.id = $1"#,
    )
    .bind(work_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}
```

This function is a two-level fallback: prefer the work's *default*
source copy; if that doesn't exist, grab *any* copy of the work. The
`COALESCE` over two subqueries is exactly the kind of SQL that looks
scary until you read it as "first choice, else second choice."

### 30.2 Add a bookmark: the upsert pattern

The add-bookmark handler in `src/routes/social.rs` is the pattern every
"save this thing" endpoint in FicHub follows:

```rust
/// POST /api/v1/bookmarks — add a bookmark
pub async fn add_bookmark_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BookmarkBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    // Resolve the url_id (fic_info.id) for this work — the `bookmarks`
    // table requires `url_id NOT NULL` with `UNIQUE(user_id, url_id)`.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, url_id) DO UPDATE SET notes = $4, is_private = $5, work_id = $3",
    )
    .bind(user_id)
    .bind(&url_id)
    .bind(body.work_id)
    .bind(body.notes.as_deref().unwrap_or(""))
    .bind(body.is_private.unwrap_or(false))
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "err": 0, "msg": "Bookmarked" })))
}
```

Line by line, this handler is a masterclass in the **upsert** pattern:

1. **Auth first.** `auth.user_id.ok_or_else(|| ...401...)` — if you're
   anonymous, you get "Login required" before any SQL runs. This
   `ok_or_else` on the `Option` is FicHub's standard idiom for
   "required login", and you'll see it in every handler from here on.
2. **Resolve the url_id.** One query turns the frontend's `work_id`
   into the copy-level identifier the table needs.
3. **Insert or update in one statement.** `ON CONFLICT (user_id,
   url_id) DO UPDATE` — if you already bookmarked this fic, the second
   bookmark *updates* your notes and privacy flag instead of erroring.
   Clicking "bookmark" again is an edit, not an error.
4. **Defaults at the SQL boundary.** `body.notes.as_deref().unwrap_or("")`
   and `body.is_private.unwrap_or(false)` — optional JSON fields become
   explicit values, never NULL surprises.

And removal is the mirror image — note that it deletes by `work_id`
(the canonical story), which works because `work_id` was populated at
insert time:

```rust
/// DELETE /api/v1/bookmarks/{work_id} — remove a bookmark
pub async fn remove_bookmark_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 AND work_id = $2")
        .bind(user_id)
        .bind(work_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Removed" })))
}
```

⚠️ **Watch Out — upsert is not magic; the conflict target must match
reality.** `ON CONFLICT (user_id, url_id)` only works because the
schema has `UNIQUE(user_id, url_id)`. If the unique constraint and the
conflict target disagree, PostgreSQL errors out with
"there is no unique or exclusion constraint matching the ON CONFLICT
specification" — a runtime error, not a compile error. When you copy
this pattern, copy the constraint too. (And yes, `ON CONFLICT DO
NOTHING` vs `ON CONFLICT DO UPDATE`: nothing for "at most once",
update for "latest write wins".)

### 30.3 Listing bookmarks, and the CSV escape hatch

The list endpoint is the plainest query in the file — and notice how it
exposes the *copy-level* shape even though the UI mostly cares about
works:

```rust
/// GET /api/v1/bookmarks — list current user's bookmarks
pub async fn list_bookmarks_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let rows = sqlx::query_as::<_, (i32, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT b.work_id, b.notes, b.is_private, b.created_at
         FROM bookmarks b WHERE b.user_id = $1 ORDER BY b.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let bookmarks: Vec<Value> = rows.into_iter().map(|(work_id, notes, is_private, created_at)| {
        json!({
            "work_id": work_id,
            "notes": notes,
            "is_private": is_private,
            "created_at": created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "bookmarks": bookmarks })))
}
```

But the *export* endpoint is where the craft shows. "Export my
bookmarks as CSV" sounds trivial until you remember that CSV has no
escaping standard and user data contains commas, quotes, and newlines:

```rust
/// GET /api/bookmarks/export — export bookmarks as CSV
pub async fn export_bookmarks_csv(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let rows = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT w.canonical_title, w.canonical_author, fi.source, b.created_at
           FROM bookmarks b
           JOIN works w ON w.id = b.work_id
           JOIN fic_info fi ON fi.work_id = w.id AND fi.id = COALESCE(w.default_source_id, (SELECT id FROM fic_info WHERE work_id = w.id LIMIT 1))
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    // Build CSV manually (avoid adding a csv crate dep)
    let mut csv = String::from("title,author,source,bookmarked_at\n");
    for (title, author, source, created) in &rows {
        // Escape double quotes and commas
        let esc = |s: &str| -> String {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        csv.push_str(&format!(
            "{},{},{},{}\n",
            esc(title),
            esc(author),
            esc(source),
            created.format("%Y-%m-%d %H:%M:%S UTC"),
        ));
    }

    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"fichub-bookmarks.csv\""),
    ];

    Ok((headers, csv).into_response())
}
```

That little `esc` closure is the whole CSV-injection defense: values
containing commas, quotes, or newlines get wrapped in double quotes,
and embedded quotes get doubled (`"` → `""`). The `CONTENT_DISPOSITION`
header tells the browser "this is a download called
fichub-bookmarks.csv", not a page to render. And the comment
"(avoid adding a csv crate dep)" is a lovely window into real-world
engineering: for one endpoint, hand-rolled wins over a dependency.

💡 **Key Concept — data-export features are the difference between a
service and a prison.** FicHub can import *and* export bookmarks —
users can always take their data elsewhere. That's data portability,
and it's both an ethical choice and a practical one: services that
offer frictionless export build more trust, and trust is what makes
people invest years of bookmarks in you. The import side is even
smarter: `import_bookmarks_csv` doesn't parse the CSV in the request
handler at all. It validates the upload (10 MB cap), enqueues a job,
and returns immediately:

```rust
    let job = crate::services::bookmark_import::ImportJob {
        user_id,
        csv: csv_str,
    };
    crate::services::bookmark_import::enqueue_import(&state.db, &state.redis, &job)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue import job: {e}")))?;

    Ok(Json(json!({
        "err": 0,
        "status": "queued",
        "msg": "Import queued — you'll be notified when it finishes.",
    })))
```

The heavy lifting happens in a background worker (we'll peek at that
pattern in Chapter 32 when notifications arrive), and the user gets a
`bookmark_import` notification when it's done. That's the
**request/queue/notify** pattern: never block a web request on
potentially-slow work you can do later.

### 30.4 Ratings: from like/dislike to a 5-star scale

The ratings table was born in migration 001 as a *binary* like/dislike
(`-- 1 = like, -1 = dislike`). Then the product team decided stars were
better. Migration 014 shows exactly how you evolve a schema without
breaking existing data — and it's a genuinely beautiful piece of
migration-writing:

```sql
-- 014: Feedback rework — 5-star ratings, public downvote hiding, in-depth reviews.
--
-- DESIGN NOTES (see commit body of feat/feedback-rework for the full rationale):
--
-- 1. 5-STAR RATINGS
--    `work_ratings.rating` is smallint; existing rows are 1 (like) / -1 (dislike).
--    We keep legacy rows untouched (rec-engine value) and enforce 1..=5 for NEW
--    writes via a CHECK constraint that admits both namespaces:
--      * rating IN (1,2,3,4,5)  →  new 5-star scale (1 = worst ... 5 = best)
--      * rating = -1            →  legacy dislike (internal rec-engine signal only,
--                                  never shown publicly; kept for backward compat)
--    The old "1 = like" semantic is intentionally REMAPPED to 1 = worst star so
--    the 5-star scale is consistent and the aggregate (avg_rating) is meaningful.
--    Migrated legacy rows: -1 → -1 (unchanged), 1 → 5 (a legacy "like" is a
--    full-throated positive signal, so it maps to 5 stars).

-- ── 1. Ratings: admit the 5-star scale + legacy -1 ─────────────────────────
ALTER TABLE work_ratings DROP CONSTRAINT IF EXISTS work_ratings_rating_check;
ALTER TABLE work_ratings ADD CONSTRAINT work_ratings_rating_check
    CHECK (rating IN (1, 2, 3, 4, 5) OR rating = -1);

-- Legacy binary rows: -1 stays -1 (internal signal); 1 ("like") becomes 5 stars
-- (a like is a strong positive, and 1 now means "worst" on the new scale).
UPDATE work_ratings SET rating = 5 WHERE rating = 1;
```

Read the design notes like a novel. The migration:
1. **Drops the old constraint** (there wasn't a named one in 001, hence
   `IF EXISTS` — `DROP CONSTRAINT IF EXISTS` is the polite way).
2. **Adds a new CHECK admitting both namespaces**: `1..=5` for new
   stars AND `-1` for legacy dislikes.
3. **Remaps data**: legacy `1` (which meant "like") becomes `5` (the
   best star) — because a like is a full-throated positive, and `1` now
   means "worst".
4. **Keeps `-1` rows forever** as an internal rec-engine signal, never
   shown publicly.

This is schema evolution done with respect for your users' data: no
destructive delete, every old row means something meaningful in the new
system, and the CHECK constraint *documents* the allowed values in the
database itself.

The handler that writes ratings shows the same dual-namespace care:

```rust
/// POST /api/v1/ratings — rate a work (5-star scale).
///
/// `rating` must be 1..=5. The legacy value -1 (old "dislike") is still
/// accepted for backward compatibility and stored as an internal-only
/// rec-engine signal — it is never returned by the aggregate endpoints, which
/// expose only positive feedback (stars + like count).
pub async fn rate_work_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RatingBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    if !(1..=5).contains(&body.rating) && body.rating != -1 {
        return Err(AppError::BadRequest(-1, "Rating must be 1..=5 stars (legacy -1 accepted)".into()));
    }

    // Resolve the url_id (fic_info.id) for this work — `work_ratings` has
    // UNIQUE(user_id, url_id), mirroring the bookmark handler.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    sqlx::query(
        "INSERT INTO work_ratings (user_id, work_id, url_id, rating) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, url_id) DO UPDATE SET rating = $4, work_id = $2",
    )
    .bind(user_id)
    .bind(body.work_id)
    .bind(&url_id)
    .bind(body.rating)
    .execute(&state.db)
    .await?;

    Ok(Json(rating_aggregate_json(&state, body.work_id).await?))
}
```

Two details stand out. The validation is an *inclusive* OR: valid is
`1..=5` OR `-1`. And the response isn't a bare "ok" — it returns the
*updated aggregate* (`rating_aggregate_json`), so the frontend can
re-render the star display from the same round trip. Return the thing
the UI needs next; don't make it re-fetch.

### 30.5 The aggregate: positive-only, always

The star display on a fic page comes from `rating_aggregate_json`.
Read it with the migration's design notes fresh in your mind:

```rust
/// Build the positive-only rating aggregate for a work.
async fn rating_aggregate_json(state: &Arc<AppState>, work_id: i32) -> Result<Value, AppError> {
    // Star-scale aggregate (1..=5 only — legacy -1 rows are internal).
    let (rating_count, sum): (i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN 1 ELSE 0 END), 0)::bigint,
            COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN rating ELSE 0 END), 0)::bigint
         FROM work_ratings WHERE work_id = $1",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    let dist_rows: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT rating, COUNT(*)::bigint FROM work_ratings
         WHERE work_id = $1 AND rating BETWEEN 1 AND 5 GROUP BY rating",
    )
    .bind(work_id)
    .fetch_all(&state.db)
    .await?;

    let mut distribution = [0i64; 5];
    for (star, count) in dist_rows {
        if (1..=5).contains(&star) {
            distribution[(star - 1) as usize] = count;
        }
    }

    let (review_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM reviews WHERE work_id = $1 AND deleted_at IS NULL AND constructive = TRUE",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    let avg_rating = if rating_count > 0 {
        f64::round((sum as f64 / rating_count as f64) * 100.0) / 100.0
    } else {
        0.0
    };

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "avg_rating": avg_rating,
        "rating_count": rating_count,
        // Positive-only public signal: a "like" = a 5-star rating (legacy
        // 1 rows were migrated to 5 in migration 014) or a written review.
        "likes": rating_count + review_count,
        "review_count": review_count,
        "rating_distribution": {
            "1": distribution[0],
            "2": distribution[1],
            "3": distribution[2],
            "4": distribution[3],
            "5": distribution[4],
        },
    }))
}
```

The `CASE WHEN rating BETWEEN 1 AND 5` filters are the whole story:
legacy `-1` rows exist in the table, but *this endpoint never counts
them*. The sum, the count, the distribution, the average — all
positive-only. And the `"likes"` field is the product decision made
visible: a "like" is a 5-star rating OR a written review. Two tables,
one sentiment. The fixed-size `[0i64; 5]` array with `(star - 1) as
usize` indexing is the classic star-distribution idiom: star 1 → index
0, star 5 → index 4.

⚠️ **Watch Out — legacy data is forever; your queries must admit it.**
The `-1` rows will exist in FicHub's database for as long as the site
runs. Every future query against `work_ratings` must remember the dual
namespace — `BETWEEN 1 AND 5` for public math, `-1` handled
explicitly for the rec engine. This is the cost of the "never destroy
data" philosophy: migrations that remap (like `1 → 5`) reduce the
damage, but the schema's history lives in every query you write. When
you design your own migrations, ask: *what will every future query have
to remember about my old data?* The answer is often "add a column
instead of overloading one."

### 30.6 Reviews: the long-form cousin

Reviews get their own table (migration 014) and their own module,
`src/routes/reviews.rs` — because a review is not a comment. It's a
*first-class signal*: one per user per work, carrying a star rating,
feeding the recommendation engine:

```sql
CREATE TABLE IF NOT EXISTS reviews (
    id          BIGSERIAL PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    work_id     INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    url_id      VARCHAR(128),
    rating      SMALLINT NOT NULL CHECK (rating IN (1, 2, 3, 4, 5)),
    title       VARCHAR(200),
    body        TEXT NOT NULL,
    constructive BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ,
    deleted_at  TIMESTAMPTZ,
    CONSTRAINT reviews_user_work_unique UNIQUE (user_id, work_id)
);
```

Note the difference from bookmarks: the unique constraint is
`(user_id, work_id)` — **one review per user per canonical story**, not
per copy. You can't review the same story twice just because it exists
on two platforms. That's a product decision encoded in a constraint.

The upsert handler shows the "create or edit" pattern at full length:

```rust
pub async fn upsert_review_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    if !(1..=5).contains(&body.rating) {
        return Err(AppError::BadRequest(-1, "Review rating must be 1..=5 stars".into()));
    }

    let title = body.title.as_deref().unwrap_or("").trim().to_string();
    if title.len() > 200 {
        return Err(AppError::BadRequest(-1, "Review title too long (max 200 chars)".into()));
    }

    let text = body.body.as_deref().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err(AppError::BadRequest(-1, "Review body cannot be empty".into()));
    }
    if text.len() > 8000 {
        return Err(AppError::BadRequest(-1, "Review too long (max 8000 chars)".into()));
    }

    // Resolve url_id like the other work-scoped social endpoints.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    // Constructive heuristic (same as comments): negative-toned review bodies
    // are still stored but flagged, and never shown by the public list.
    let constructive = crate::routes::comments::constructive_score(&text);

    let (id, created_at, updated_at, constructive): (i64, String, String, bool) =
        sqlx::query_as(
            r#"INSERT INTO reviews (user_id, work_id, url_id, rating, title, body, constructive)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (user_id, work_id) DO UPDATE SET
                   rating = EXCLUDED.rating,
                   title = EXCLUDED.title,
                   body = EXCLUDED.body,
                   constructive = EXCLUDED.constructive,
                   updated_at = NOW()
               RETURNING id, created_at::text, COALESCE(updated_at::text, created_at::text), constructive"#,
        )
        .bind(user_id)
        .bind(body.work_id)
        .bind(&url_id)
        .bind(body.rating)
        .bind(title.as_str())
        .bind(text.as_str())
        .bind(constructive)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "review": {
            "id": id,
            "work_id": body.work_id,
            "rating": body.rating,
            "title": if title.is_empty() { Value::Null } else { Value::String(title) },
            "body": text,
            "constructive": constructive,
            "created_at": created_at,
            "updated_at": updated_at,
        }
    })))
}
```

Every input is validated *before* touching the database: rating range,
title length, body non-empty, body length. Then the insert-or-update
uses `EXCLUDED` — PostgreSQL's keyword for "the row I tried to insert"
— so an edit replaces rating/title/body and bumps `updated_at` while
keeping the original `created_at` and `id`. And the `RETURNING` clause
hands back the final state in one round trip.

💡 **Key Concept — `EXCLUDED` is your upsert's best friend.**
In `INSERT ... ON CONFLICT DO UPDATE`, the `DO UPDATE` branch can
reference the would-be-inserted row via the `EXCLUDED` pseudo-table.
`SET body = EXCLUDED.body` means "set the existing row's body to
whatever the insert tried to put in". Without it you'd have to bind
every value twice — once for `VALUES`, once for `SET`. And `RETURNING`
lets you read the post-upsert row (including defaults and `NOW()`-style
updates) in the same statement. Three features, one round trip —
this is why the upsert pattern scales to every "save" button in your
app.

### 30.7 The reviews list and the delete guard

The public review list is deliberately strict about what it shows:

```rust
pub async fn list_reviews_handler(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let rows: Vec<(i64, String, String, i16, Option<String>, String, String, bool)> =
        sqlx::query_as(
            r#"SELECT r.id, u.username, r.body, r.rating, r.title,
                      r.created_at::text, COALESCE(r.updated_at::text, r.created_at::text),
                      r.constructive
               FROM reviews r
               JOIN users u ON u.id = r.user_id
               WHERE r.work_id = $1
                 AND r.deleted_at IS NULL
                 AND r.constructive = TRUE
               ORDER BY r.created_at DESC"#,
        )
        .bind(work_id)
        .fetch_all(&state.db)
        .await?;
```

`deleted_at IS NULL` (soft-deleted reviews vanish) AND `constructive =
TRUE` (negative-toned reviews are stored but hidden). We'll dive into
the constructive heuristic in Chapter 31 — it's one of FicHub's most
interesting product decisions — but notice how it's *shared*: reviews
call `crate::routes::comments::constructive_score`, the same function
comments use. One policy, two features.

And delete follows the ownership-or-curator pattern we learned in
Chapter 29:

```rust
    let row: Option<(Option<i32>, i16)> = sqlx::query_as(
        "SELECT r.user_id, u.role FROM reviews r
         LEFT JOIN users u ON u.id = $2
         WHERE r.id = $1",
    )
    .bind(review_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some((Some(uid), _)) if uid == user_id => {}
        Some((_, role)) if role >= 1 => {}
        _ => return Err(AppError::BadRequest(403, "Not authorized".into())),
    }

    let updated = sqlx::query(
        "UPDATE reviews SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(review_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("Review not found".into()));
    }
```

The delete is a *soft* delete (`SET deleted_at = NOW()`), the WHERE
clause makes it idempotent (`AND deleted_at IS NULL` — deleting twice
affects zero rows), and the `rows_affected()` check converts "already
gone" into a clean 404. That's the complete lifecycle: **insert with
upsert, read with filters, delete by tombstone.**

🧪 **Try It Yourself — the social round trip.**
With a token from Chapter 28, build a shelf for one work. You'll need
a real work id — grab one from `/api/works/random` (yes, FicHub has a
"surprise me" endpoint!):

```bash
# Get a work to play with
WORK=$(curl -s http://localhost:8080/api/works/random | python3 -c 'import sys,json; print(json.load(sys.stdin)["work"]["id"])')
echo "Playing with work $WORK"

# Bookmark it, with a private note
curl -s -X POST http://localhost:8080/api/bookmarks \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"work_id\": $WORK, \"notes\": \"start this one on the weekend\", \"is_private\": true}"
# → {"err":0,"msg":"Bookmarked"}

# Bookmark it AGAIN with different notes — upsert, not error:
curl -s -X POST http://localhost:8080/api/bookmarks \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"work_id\": $WORK, \"notes\": \"no wait, tonight\"}"
# → {"err":0,"msg":"Bookmarked"}  (the note was updated)

# Rate it 5 stars and see the aggregate change shape
curl -s -X POST http://localhost:8080/api/ratings \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"work_id\": $WORK, \"rating\": 5}"
# → {"err":0,"work_id":...,"avg_rating":5.0,"rating_count":1,"likes":1,...}

# Write a review
curl -s -X POST http://localhost:8080/api/reviews \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"work_id\": $WORK, \"rating\": 5, \"title\": \"Hidden gem\", \"body\": \"The pacing is perfect.\"}"

# Now export your bookmark as CSV — open it in a spreadsheet:
curl -s http://localhost:8080/api/bookmarks/export -H "Authorization: Bearer <TOKEN>"
```

Watch what the second bookmark POST does: it doesn't 400, it *updates*.
That single behavior — upsert — is what makes bookmark buttons feel
responsive instead of error-prone.

---

Your shelf is built: saved works, star ratings, written reviews. But a
shelf is only half of a community — the other half is *conversation*.
Comments are where readers talk about a fic, and moderation is where
that conversation stays kind. Chapter 31 takes us into the most
opinionated code in FicHub: the constructive-comment heuristic, the
LLM triage pipeline, and the transparent modlog.

---

## Chapter 31 — Comments and Moderation: Keeping the Conversation Kind

Every community platform eventually faces the same question: *what do
we do with the mean comments?* FicHub's answer is opinionated, layered,
and — I think — genuinely interesting. It's three lines of defense:

1. **At insert time**: a cheap heuristic flags negative-toned comments,
   so they never appear publicly in the first place.
2. **In the background**: an LLM (Ollama) classifies every comment into
   a moderation queue for curators.
3. **In the record**: every moderator action lands in a public modlog —
   moderation is transparent by design.

Let's look at each layer, then at the threaded-comment engine they
protect.

### 31.1 Layer 1: the constructive-comment heuristic

The comments table was born in migration 001 with the classic
soft-delete columns:

```sql
-- Threaded comments
CREATE TABLE IF NOT EXISTS comments (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    user_id INT4 REFERENCES users(id) ON DELETE SET NULL,
    parent_id BIGINT REFERENCES comments(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,
    work_id INTEGER REFERENCES works(id)
);
```

Then migration 014 (the feedback rework we met in Chapter 30) added
the `constructive` column — the hinge of the whole moderation design:

```sql
-- ── 2. Comments: constructive flag ─────────────────────────────────────────
ALTER TABLE comments ADD COLUMN IF NOT EXISTS constructive boolean NOT NULL DEFAULT TRUE;
CREATE INDEX IF NOT EXISTS idx_comments_work_constructive
    ON comments (work_id) WHERE deleted_at IS NULL AND constructive = TRUE;
```

And the heuristic itself, at the top of `src/routes/comments.rs`:

```rust
// ── Constructive-comment heuristic ────────────────────────────────────────
//
// Feedback rework (program item 1): the site only surfaces positive /
// constructive feedback publicly. Downvotes and negative-toned comments are
// *kept in the DB* (the rec engine can still use them) but hidden from public
// view. At insert time every comment is scored by `is_negative_comment`; a
// match sets `constructive = FALSE` so the public list endpoints skip it.
// Curators can still flip the flag / soft-delete via the existing endpoints.

const NEGATIVE_MARKERS: &[&str] = &[
    "terrible", "awful", "horrible", "garbage", "trash", "rubbish", "waste of time",
    "disappointing", "disappointment", "hate", "hating", "stupid", "idiotic",
    "dumb", "ridiculous", "nonsense", "boring", "bored", "cringe", "worst",
    "worse", "sucks", "sucked", "suck", "useless", "pointless", "pathetic",
    "annoying", "disgusting", "crap", "shit", "fuck", "piss", "abandon",
    "quit reading", "couldn't finish", "could not finish", "drops this", "dropped",
];

/// Heuristic: is this comment text negative-toned (i.e. not constructive)?
/// Cheap substring match on lowercased text — no NLP, deliberate. A comment
/// that contains a negation marker is flagged non-constructive at insert.
pub fn is_negative_comment(body: &str) -> bool {
    let lower = body.to_lowercase();
    NEGATIVE_MARKERS.iter().any(|m| lower.contains(m))
}

/// Score a comment body for constructiveness; public helper used by both
/// comment insertion paths (v2 `social.rs` + threaded `comments.rs`).
pub fn constructive_score(body: &str) -> bool {
    !is_negative_comment(body)
}
```

Read the doc comment very carefully, because the *philosophy* is more
important than the code: **"the site only surfaces positive /
constructive feedback publicly. Downvotes and negative-toned comments
are *kept in the DB* (the rec engine can still use them) but hidden
from public view."** FicHub made a product decision — this is a
feel-good reading space, and the UI shows appreciation, not
denigration — and implemented it as *a filter on read*, not *a delete
on write*. Nothing is destroyed. The rec engine can still use the
sentiment signal. The data stays; only the public view changes.

And the heuristic is *deliberately* crude: a substring match over a
marker list, on lowercased text, with a doc comment that says "no NLP,
deliberate". The tests even document the trade-off:

```rust
#[test]
fn test_constructive_score_allows_negated_markers() {
    // The heuristic is a cheap substring match — "not terrible" still
    // contains the marker, so it is flagged non-constructive. This
    // documents the deliberate limitation (no NLP).
    assert!(!constructive_score("It is not terrible at all."));
    assert!(!constructive_score("Nothing boring about this one!"));
}

#[test]
fn test_constructive_score_rejects_negative_markers() {
    assert!(!constructive_score("This is terrible."));
    assert!(!constructive_score("I hate this fic."));
    assert!(!constructive_score("What a waste of time."));
    assert!(!constructive_score("It sucks."));
}
```

💡 **Key Concept — a false positive you can see beats a false negative
you can't.** "It is not terrible at all" gets flagged non-constructive
— that's a false positive, a genuinely kind comment hidden from public
view. FicHub *documents* that failure in a test rather than trying to
fix it with smarter matching. Why? Because the cost structure favors
it: a falsely-hidden comment is recoverable (a curator sees it in
triage and flips it), while a falsely-shown hateful comment costs real
harm and reputation. When you build any classifier, ask which error
direction is cheaper, and bias the threshold accordingly. (We'll see
the exact same bias in the LLM triage layer next.)

⚠️ **Watch Out — substring matching is not sentiment analysis.**
"Nothing boring about this one!" is *praise* — and it's hidden,
because "boring" is in the list. "This fic is not garbage, it's the
best I've read" — hidden too. Any junior reading this will be tempted
to "fix" it with a negation parser or an ML model. Don't. The
deliberate crudeness is a *feature*: it's deterministic, free, runs in
microseconds on every comment insert with zero infrastructure, and its
limitations are documented and tested. If you replace it with a smarter
model, you inherit the model's failure modes *plus* its latency and
cost. The layered design — heuristic at insert, LLM triage in the
background — exists precisely so the cheap layer can be dumb.

### 31.2 Layer 2: LLM triage in the background

The heuristic catches obvious negativity. For everything else, FicHub
runs a *triage pipeline*: every posted comment is classified by Ollama
into one of five categories. The service lives in
`src/services/comment_triage.rs`:

```rust
//! Comment moderation triage — best-effort LLM classification of new
//! comments into an admin review queue.
//!
//! Every posted comment is classified by Ollama (llama3.1:8b via
//! `/api/generate`) into one of: `fine`, `constructive`, `non-constructive`,
//! `toxic`, `spam`. The verdict lands in the `comment_triage` table with a
//! short reason + confidence; curators review it in the admin moderation UI
//! and decide what to do. Nothing is ever auto-hidden: triage is advisory.
//!
//! The whole pipeline is deliberately best-effort. If Ollama is down, the
//! response is unparsable, or the insert fails, the comment post itself is
//! unaffected — [`classify_comment`] falls back to `Fine` with confidence
//! `0.0` and the caller logs and moves on.

/// Triage categories. `Fine`/`Constructive` are publicly shown;
/// `NonConstructive`/`Toxic`/`Spam` land in the admin review queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriageCategory {
    Fine,
    Constructive,
    NonConstructive,
    Toxic,
    Spam,
}
```

Three things to notice in that module doc. First, the model: llama3.1
8B, called through Ollama's `/api/generate`. FicHub runs a *local* LLM
for moderation — no external API, no per-comment cost, no data leaving
the server. Second, the outcome: **"Nothing is ever auto-hidden:
triage is advisory."** The LLM queues, the human decides. That's a
profoundly important design choice — the machine never has the final
word over a community member's words. Third, the failure mode: if
Ollama is down, classification falls back to `Fine` with confidence
0.0. **The comment post must never fail because the moderator bot is
unavailable.**

The insert path wires it together as fire-and-forget — in
`src/routes/social.rs`, right after the comment row is created:

```rust
    // ── Comment moderation triage (best-effort, fire-and-forget) ─────
    // The comment post must succeed even if Ollama is down: classification
    // runs in its own task and every failure inside it falls back to a
    // `Fine` verdict with confidence 0.0 (never an error).
    {
        let db = state.db.clone();
        let ollama = state.ollama.clone();
        let body = body.body.clone();
        tokio::spawn(async move {
            let triage =
                crate::services::comment_triage::classify_and_store(&db, &ollama, id, &body).await;
            tracing::debug!("comment {id} triaged as {:?}", triage.category);
        });
    }
```

`tokio::spawn` — the comment is already in the database, the user has
already gotten their 200 OK; the classification happens *concurrently*,
in its own task. This is the pattern for any "nice-to-have" work that
must never slow down or break the primary action.

💡 **Key Concept — fire-and-forget with cloned state.**
`tokio::spawn` requires its future to be `'static` — it can't borrow
`state`. So the handler clones the pieces the task needs (`db`,
`ollama`, and the comment text) and moves them into the task. That
clone of an `Arc<PgPool>` is cheap (a refcount bump), which is exactly
why `AppState` is wrapped in `Arc` everywhere. The rule of thumb:
if a piece of work is (a) optional, (b) potentially slow, and (c)
must not fail the request — spawn it. If it must *never* be lost, put
it on a queue instead (like the bookmark import we saw in Chapter 30,
which uses Redis + a worker so jobs survive restarts).

And the triage category parsing is deliberately lenient — a weird
model answer can never create a spurious queue entry:

```rust
impl TriageCategory {
    /// Parse a category name leniently (case/whitespace-insensitive).
    /// Anything unknown falls back to `Fine` so a weird model answer never
    /// creates a spurious moderation queue entry.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "constructive" => TriageCategory::Constructive,
            "non-constructive" | "nonconstructive" => TriageCategory::NonConstructive,
            "toxic" => TriageCategory::Toxic,
            "spam" => TriageCategory::Spam,
            _ => TriageCategory::Fine,
        }
    }
```

### 31.3 The threaded comment engine: one query per depth

The comments themselves are *threaded* — replies to replies to replies.
The listing handler in `src/routes/comments.rs` is the most
SQL-interesting part of this chapter, because fetching an arbitrary
depth of nested comments in one round trip needs a **recursive CTE**:

```rust
    // Fetch all replies for these top-level comments (recursive CTE)
    let all_replies: Vec<(i64, String, Option<i64>, String, Option<i32>, Option<String>, String, Option<String>, bool, bool, i64)> =
        if top_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_as(
                "WITH RECURSIVE thread AS (
                    SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                           u.username, c.created_at::text, c.updated_at::text,
                           c.deleted_at IS NOT NULL AS deleted, c.is_hidden, c.parent_id AS root_id
                    FROM comments c
                    LEFT JOIN users u ON u.id = c.user_id
                    WHERE c.parent_id = ANY($1)
                    AND c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE

                    UNION ALL
                    SELECT c.id, c.url_id, c.parent_id, c.body, c.user_id,
                           u.username, c.created_at::text, c.updated_at::text,
                           c.deleted_at IS NOT NULL AS deleted, c.is_hidden, t.root_id
                    FROM comments c
                    LEFT JOIN users u ON u.id = c.user_id
                    JOIN thread t ON c.parent_id = t.id
                    WHERE c.deleted_at IS NULL AND c.is_hidden = FALSE AND c.constructive = TRUE
                )
                SELECT id, url_id, parent_id, body, user_id, username,
                       created_at, updated_at, deleted, is_hidden, root_id
                FROM thread
                ORDER BY root_id, created_at ASC",
            )
            .bind(&top_ids)
            .fetch_all(&state.db)
            .await?
        };
```

Let's decode the CTE, because it's a tool you'll use for the rest of
your career:

- **The base term** (before `UNION ALL`): grab the *direct children*
  of the top-level comments — `WHERE c.parent_id = ANY($1)`. Each row
  also records which top-level comment it ultimately belongs to:
  `c.parent_id AS root_id`.
- **The recursive term** (after `UNION ALL`): join the comments table
  against the CTE itself — `JOIN thread t ON c.parent_id = t.id` —
  finding *children of children*. The `root_id` is carried down
  unchanged, so a great-grandchild still knows which thread it belongs
  to.
- **The termination**: PostgreSQL keeps recursing until a join produces
  no new rows. A comment tree of any depth collapses into one result
  set.

Then the handler does the tree assembly in Rust: `reply_counts` maps
each root id to its reply total, `replies_by_parent` groups replies
under their parent, and the response walks top-level rows emitting
each comment followed by its replies:

```rust
    // Build reply count map
    use std::collections::HashMap;
    let mut reply_counts: HashMap<i64, i64> = HashMap::new();
    for row in &all_replies {
        *reply_counts.entry(row.10).or_insert(0) += 1;
    }

    // Group replies by parent
    let mut replies_by_parent: HashMap<i64, Vec<&(i64, String, Option<i64>, String, Option<i32>, Option<String>, String, Option<String>, bool, bool, i64)>> = HashMap::new();
    for row in &all_replies {
        if let Some(parent) = row.2 {
            replies_by_parent.entry(parent).or_default().push(row);
        }
    }
```

💡 **Key Concept — recursive CTEs are graph traversal in SQL.**
Any "parent/child" structure with unknown depth — comment threads,
category trees, org charts — is a recursive CTE's home turf. The
syntax is always the same skeleton: a base `SELECT` that seeds the
first generation, `UNION ALL`, a recursive `SELECT` that joins the
table to the CTE name, and (implicitly) the loop ends when nothing new
matches. Two things trip everyone up: the recursive term must not
re-seed rows the base term already grabbed (your WHERE clauses must
shrink the candidate set each generation), and without `UNION ALL`'s
dedup the query can loop forever on cyclic data. Comment threads are
acyclic (a comment's parent is older than it), so FicHub is safe.

The rendering side shows the *display* rules — this is where
`deleted_at` and `is_hidden` become visible strings:

```rust
fn build_comment_json(
    id: i64, url_id: &str, parent_id: Option<i64>, body: &str,
    user_id: Option<i32>, username: Option<&str>,
    created_at: &str, updated_at: Option<&str>,
    deleted: bool, hidden: bool, reply_count: i64,
) -> Value {
    let user = user_id.map(|uid| {
        json!({
            "id": uid,
            "username": username.unwrap_or("unknown"),
        })
    });

    let display_body = if deleted {
        "[deleted]".to_string()
    } else if hidden {
        "[hidden]".to_string()
    } else {
        body.to_string()
    };

    json!({
        "id": id,
        "url_id": url_id,
        "parent_id": parent_id,
        "body": display_body,
        "user": user,
        "created_at": created_at,
        "updated_at": updated_at,
        "deleted": deleted,
        "hidden": hidden,
        "reply_count": reply_count,
    })
}
```

Note the precedence: `deleted` wins over `hidden` (a deleted-and-hidden
comment shows `[deleted]`), and `username.unwrap_or("unknown")` handles
the `ON DELETE SET NULL` case where a user's account is gone but their
comments remain. There's a test pinning that exact precedence:

```rust
#[test]
fn test_build_comment_json_deleted_and_hidden_ignored() {
    // deleted takes priority over hidden
    let json = build_comment_json(
        60, "work-xyz", None, "Gone",
        Some(1), Some("Mod"),
        "2026-07-27T16:00:00Z", None,
        true, true, 0,
    );
    assert_eq!(json["body"], "[deleted]");
    assert_eq!(json["deleted"], true);
    assert_eq!(json["hidden"], true);
}
```

### 31.4 Posting a comment: validation and parenting

The post handler is where all of Chapter 30's lessons come together —
and it adds one new check worth studying: **parent verification**.
When you reply to a comment, FicHub makes sure the parent actually
belongs to the same work:

```rust
    // Verify parent belongs to same work if provided
    if let Some(parent_id) = body.parent_id {
        let parent: Option<(String,)> = sqlx::query_as(
            "SELECT url_id FROM comments WHERE id = $1",
        )
        .bind(parent_id)
        .fetch_optional(&state.db)
        .await?;

        match parent {
            Some((parent_url,)) if parent_url == url_id => {}
            Some(_) => return Err(AppError::BadRequest(-1, "Parent comment belongs to different work".into())),
            None => return Err(AppError::BadRequest(-1, "Parent comment not found".into())),
        }
    }
```

This is a **cross-row integrity check** — the database's foreign keys
can't express "a reply's parent must live on the same work", because
that's a relationship between *two columns of two rows*, not a key
reference. So the application enforces it. The match arms tell three
stories: same work (proceed), different work (reject with a specific
message), or nonexistent parent (reject with a different message).
Specific errors, not a blanket "invalid".

⚠️ **Watch Out — business rules that FKs can't express live in
handlers.** Foreign keys handle "this id must exist". They cannot
handle "this id must exist AND belong to the same story AND not be a
reply to yourself". Those rules are application logic — and the moment
you write them, write the *specific* error messages too. A junior's
instinct is a single `Err(AppError::BadRequest(-1, "invalid"))` for
all three cases; FicHub's three-way match is what lets the frontend
tell the user *what actually went wrong*.

The insert itself reuses the constructive heuristic as the value for
the `constructive` column, and the response tells the client whether
its comment is publicly visible:

```rust
    let (id, created_at, constructive): (i64, String, bool) = sqlx::query_as(
        "INSERT INTO comments (url_id, user_id, parent_id, body, constructive)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, created_at::text, constructive",
    )
    .bind(&url_id)
    .bind(user_id)
    .bind(body.parent_id)
    .bind(&body.body)
    .bind(crate::routes::comments::constructive_score(&body.body))
    .fetch_one(&state.db)
    .await?;
```

### 31.5 Moderation endpoints and the transparent modlog

The two moderation actions — delete and hide — are the role-gate
examples we already dissected in Chapter 29 (ownership-or-curator for
delete, curator-only for hide). The new ingredient is what happens
*around* them: the modlog.

The modlog table (migration 034) was designed for transparency from
the start:

```sql
-- Moderation log (modlog): a transparent, public-by-default record of every
-- moderator / curator / admin action. ANY logged-in user can read it — the
-- point is that moderation is completely transparent.
CREATE TABLE IF NOT EXISTS modlog (
    id            BIGSERIAL PRIMARY KEY,
    actor_id      INTEGER,
    actor_username TEXT,
    action        TEXT NOT NULL,
    target_type   TEXT NOT NULL DEFAULT '',
    target_id     TEXT NOT NULL DEFAULT '',
    details       JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

"ANY logged-in user can read it" — not just admins. The modlog endpoint
is served by `/api/modlog` and requires only *being logged in*, not
being a curator. Moderation actions are public record. That's a
deliberate anti-tyranny design: the people with power over the
conversation are themselves accountable to the conversation.

The writer side is best-effort by construction (`src/modlog.rs`):

```rust
/// Record a moderation action. Never fails the caller (errors are logged).
pub async fn record(
    db: &sqlx::PgPool,
    actor_id: Option<i32>,
    actor_username: Option<String>,
    action: &str,
    target_type: &str,
    target_id: &str,
    details: Value,
) {
    let res = sqlx::query(
        "INSERT INTO modlog (actor_id, actor_username, action, target_type, target_id, details)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(actor_id)
    .bind(actor_username)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(details)
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!("modlog record failed for action '{action}': {e}");
    }
}
```

💡 **Key Concept — audit logging must never fail the audited action.**
`record` swallows its own errors (`tracing::warn!` and move on). Think
about the alternative: if hiding a comment *depended* on the modlog
insert succeeding, then a modlog outage would mean curators can't hide
comments — which means abusive comments stay up *because the audit
system is down*. That's backwards. The primary action (hide/ban/delete)
must always succeed; the audit is a best-effort side effect. This
"log never blocks the logged thing" principle applies to every
observability feature you'll ever write: metrics, audit trails, error
reporting. They're all side effects, and side effects must be
optional.

The reader side (`modlog::list`) shows the whole pattern for a
filterable audit view:

```rust
/// Recent modlog entries, newest first.
pub async fn list(
    db: &sqlx::PgPool,
    limit: i64,
    action_filter: Option<&str>,
) -> sqlx::Result<Vec<ModlogEntry>> {
    let rows = if let Some(act) = action_filter {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             WHERE action = $2
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .bind(act)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(db)
        .await?
    };
    Ok(rows)
}
```

The `if let Some(act)` pattern — one query with a filter, one without —
is the classic "optional WHERE" idiom. (You can also do it with
dynamic SQL, but two explicit queries is clearer and keeps every query
static and indexable. sqlx loves static queries: they're checked at
compile time.)

The `ModlogEntry` struct shows the payload shape — including the
`details` JSONB that carries context like the new role value from
Chapter 29:

```rust
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ModlogEntry {
    pub id: i64,
    pub actor_id: Option<i32>,
    pub actor_username: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub details: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

And there's a tiny helper that shows the "any logged-in user" policy in
one line:

```rust
/// Simple guard for "any logged-in user" (not admin/curator-only).
pub fn require_logged_in(auth: &crate::routes::auth::AuthUser) -> Result<(), crate::error::AppError> {
    if auth.user_id.is_none() {
        return Err(crate::error::AppError::BadRequest(401, "Login required".into()));
    }
    Ok(())
}
```

🧪 **Try It Yourself — the full moderation story.**
Post a comment, watch the layers react:

```bash
# Post a nice comment (constructive = true)
curl -s -X POST "http://localhost:8080/api/works/<url_id>/comments" \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d '{"body": "The character work in chapter 3 is fantastic!"}'
# → constructive: true

# Post a nasty one — it still "succeeds"...
curl -s -X POST "http://localhost:8080/api/works/<url_id>/comments" \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d '{"body": "This is garbage, what a waste of time"}'
# → constructive: false   ← the flag, right in the response

# ...but the public list won't show it:
curl -s "http://localhost:8080/api/works/<url_id>/comments" | python3 -m json.tool
# (only the nice comment appears)

# As a curator: hide a comment and watch the modlog
curl -s -X PATCH http://localhost:8080/api/comment/<id>/hide \
  -H "Authorization: Bearer <CURATOR_TOKEN>" \
  -H 'Content-Type: application/json' -d '{"hidden": true}'

# The public modlog now records the action — anyone logged in can see it:
curl -s http://localhost:8080/api/modlog -H "Authorization: Bearer <TOKEN>"
```

The magic moment is the second comment: the API returns
`"constructive": false` — the system *tells you* your comment was
flagged, stores it anyway, and quietly keeps it out of the public
conversation. No shadowban theatrics, no angry modmail. Just a flag,
a filter, and a record.

---

The conversation is now moderated, threaded, and accountable. One
social feature remains — the one that turns a library into a
*community*: follows, the updates feed, and the notifications that
pull readers back. Chapter 32 is the payoff — the reason all those
bookmarks, ratings, and comments exist. See you there.

---

## Chapter 32 — Follows, the Updates Feed, and Notifications

This is the chapter where FicHub stops being a download tool and
becomes a community. Everything we've built so far — identity, roles,
shelves, conversations — points at one moment: a reader follows an
author, follows a work, and the next time that story updates, FicHub
*remembers to tell them*. That's the follow system, and it's built on
three pieces:

1. The `follows` table — a single table that follows *users*, *works*,
   *or authors-by-name* using a clever nullable-column trick.
2. The updates feed — "what did the works I follow do lately?", with a
   per-follow `last_seen` that powers a little NEW badge.
3. The notifications inbox — per-user rows with per-user preferences,
   written by every feature in the app.

### 32.1 The follows table: three targets, one table, one CHECK

Here's the schema from migration 003 — read it twice, because the
constraint at the bottom is the whole trick:

```sql
CREATE TABLE IF NOT EXISTS follows (
    id BIGSERIAL PRIMARY KEY,
    follower_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followee_id INT4 REFERENCES users(id) ON DELETE CASCADE,      -- follow a user
    work_id INT4 REFERENCES works(id) ON DELETE CASCADE,          -- follow a work
    author_name TEXT,                                              -- follow an author by name
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT follows_target CHECK (
        num_nonnulls(followee_id, work_id, author_name) = 1
    )
);

-- Expression-based unique index for follows (PG16 supports this as an index, not a table constraint)
CREATE UNIQUE INDEX IF NOT EXISTS idx_follows_unique ON follows(follower_id, COALESCE(followee_id, 0), COALESCE(work_id, 0), COALESCE(author_name, ''));

CREATE INDEX IF NOT EXISTS idx_follows_follower ON follows(follower_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_follows_followee ON follows(followee_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_follows_work ON follows(work_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_follows_author ON follows(author_name, created_at DESC);
```

One row per follow; three possible targets; exactly one must be set.
Two mechanisms enforce that:

- **`CONSTRAINT follows_target CHECK (num_nonnulls(followee_id,
  work_id, author_name) = 1)`** — PostgreSQL's `num_nonnulls()` counts
  how many of the listed columns are non-NULL, and the CHECK demands
  exactly one. A row with no target is rejected. A row with *two*
  targets is rejected. The database itself refuses ambiguous follows.
- **The expression unique index** — `UNIQUE (follower_id, COALESCE(...))`
  — turns the three nullable columns into one comparable value
  (NULL becomes 0 or ''), so the database can enforce "one follow per
  (follower, target)" even though the target column varies by row
  type. This is the trick that makes follow/unfollow *idempotent*:
  following the same work twice is a no-op instead of a duplicate.

💡 **Key Concept — `num_nonnulls` and COALESCE-in-index: modeling an
"exclusive OR" in SQL.** "Exactly one of these three columns" is a
constraint that SQL has no native keyword for — so FicHub composes it
from `num_nonnulls(...) = 1` (validity) and a COALESCE-ized unique
index (uniqueness). Every time you see this pattern, it's a *polymorphic
relationship*: one table standing in for three. The alternative —
separate `follow_users`, `follow_works`, `follow_authors` tables —
triples the code and makes "list everything I follow" a three-table
UNION. The single-table design costs a bit of cleverness up front and
pays off in every query you write afterward.

⚠️ **Watch Out — indexes on expressions vs. columns.** Note the
difference between the unique index (which uses `COALESCE(...)` — an
*expression*) and the four plain indexes (`idx_follows_follower`,
`idx_follows_followee`, ...). Plain column indexes serve the common
query patterns ("whose follows are these?" / "who follows this?");
the expression index only serves the uniqueness guarantee. If you ever
write `WHERE author_name = 'x'`, PostgreSQL uses `idx_follows_author`;
it does *not* use `idx_follows_unique` (different expression tree).
Indexes are per-query tools, not a pile you can dump and forget.

### 32.2 The follow handler: one endpoint, three branches

`src/routes/follows.rs` dispatches on `target_type` — the frontend
says "follow a user", "follow a work", or "follow an author", and the
handler routes to one of three query functions:

```rust
#[derive(Debug, Deserialize)]
pub struct FollowBody {
    pub target_type: String,  // "user", "work", "author"
    pub target_id: Option<i32>,
    pub author_name: Option<String>,
}

/// POST /api/v1/follows — follow a user, work, or author
pub async fn follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<FollowBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    match body.target_type.as_str() {
        "user" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest(-1, "target_id required for user follow".into()))?;
            if target == user_id {
                return Err(AppError::BadRequest(-1, "Cannot follow yourself".into()));
            }
            queries::follow_user(&state.db, user_id, target).await?;
        }
        "work" => {
            let target = body.target_id.ok_or_else(|| AppError::BadRequest(-1, "target_id required for work follow".into()))?;
            // Only works that exist can be followed.
            if queries::get_work(&state.db, target).await?.is_none() {
                return Err(AppError::BadRequest(-1, "work not found".into()));
            }
            queries::follow_work(&state.db, user_id, target).await?;
        }
        "author" => {
            let name = body.author_name.as_deref().ok_or_else(|| AppError::BadRequest(-1, "author_name required for author follow".into()))?;
            if name.trim().is_empty() {
                return Err(AppError::BadRequest(-1, "author_name must not be empty".into()));
            }
            queries::follow_author(&state.db, user_id, name).await?;
        }
        _ => return Err(AppError::BadRequest(-1, "target_type must be 'user', 'work', or 'author'".into())),
    }

    // Return the follow id so the client can unfollow without re-listing.
    let follow_id = match body.target_type.as_str() {
        "work" => queries::find_follow_for_work(&state.db, user_id, body.target_id.unwrap_or(0)).await?,
        "author" => queries::find_follow_for_author(&state.db, user_id, body.author_name.as_deref().unwrap_or("")).await?,
        _ => None,
    };

    Ok(Json(json!({
        "err": 0,
        "msg": "Following",
        "follow_id": follow_id,
    })))
}
```

Three branches, each with *its own validation*: users can't follow
themselves, works must exist, author names must be non-empty. Each
branch maps to a query function — and each of those is the same
idempotent INSERT we learned in Chapter 30:

```rust
/// Follow a work
pub async fn follow_work(pool: &PgPool, follower_id: i32, work_id: i32) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO follows (follower_id, work_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(follower_id)
    .bind(work_id)
    .execute(pool)
    .await?;
    Ok(())
}
```

`ON CONFLICT DO NOTHING` — the expression unique index from 32.1 makes
a duplicate follow *impossible*, and this clause makes it *harmless*.
Following something you already follow is a quiet no-op. (Notice the
follow-user branch adds the extra "cannot follow yourself" guard — a
business rule the schema can't express, exactly like the parent-check
we saw in Chapter 31.)

The rest of the module is the follow *read* surface. The listing and
the check endpoints are built on the same query layer:

```rust
/// GET /api/v1/follows — list who I follow
pub async fn list_follows_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let follows = queries::list_follows(&state.db, user_id).await?;

    let items: Vec<Value> = follows.into_iter().map(|f| {
        json!({
            "id": f.id,
            "followee_id": f.followee_id,
            "work_id": f.work_id,
            "author_name": f.author_name,
            "created_at": f.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "follows": items })))
}
```

And the check endpoint — the one the frontend calls to decide whether
to render "Follow" or "Following" — returns both the boolean *and* the
follow id, so the UI can unfollow without a second lookup:

```rust
/// GET /api/v1/follows/check/{target_type}/{target_id}
pub async fn check_follow_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let is_following = match target_type.as_str() {
        "user" => queries::is_following_user(&state.db, user_id, target_id).await?,
        "work" => queries::is_following_work(&state.db, user_id, target_id).await?,
        _ => false,
    };

    // For works, also return the follow id so the client can mark seen /
    // unfollow without an extra listing call.
    let follow_id = if target_type == "work" {
        queries::find_follow_for_work(&state.db, user_id, target_id).await?
    } else {
        None
    };

    Ok(Json(json!({
        "err": 0,
        "is_following": is_following,
        "follow_id": follow_id,
    })))
}
```

### 32.3 The updates feed: one `last_seen` column, one NEW badge

Following a work is only half of it — the *point* of following is
knowing when it updates. Migration 017 added the piece that makes the
updates feed possible:

```sql
-- 017: Follow updates — per-follow "last seen" tracking for the update feed.
--
-- The `updates` feed (GET /api/v1/updates) lists works the user follows,
-- ordered by fic_updated DESC. To show a NEW badge ("updated since you last
-- looked"), each follow needs a last_seen timestamp: the value is written
-- when the user opens the fic page (POST /api/v1/follows/{id}/seen) or when
-- they view the updates feed.

ALTER TABLE follows
    ADD COLUMN IF NOT EXISTS last_seen TIMESTAMPTZ;

-- Index so "follows with updates since last_seen" scans are cheap.
CREATE INDEX IF NOT EXISTS idx_follows_last_seen
    ON follows (follower_id, last_seen DESC)
    WHERE work_id IS NOT NULL;
```

The whole "NEW" badge feature is one nullable timestamp compared
against `fic_updated`. The query that powers the feed lives in
`src/db/queries.rs`:

```rust
#[derive(Debug, sqlx::FromRow)]
pub struct FollowedWorkUpdate {
    pub follow_id: i64,
    pub work_id: i32,
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub fic_updated: chrono::DateTime<chrono::Utc>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_followed_work_updates(pool: &PgPool, user_id: i32) -> AppResult<Vec<FollowedWorkUpdate>> {
    let rows = sqlx::query_as::<_, FollowedWorkUpdate>(
        r#"SELECT f.id AS follow_id, f.work_id, fi.id AS url_id, fi.title, fi.author,
                  fi.words, fi.chapters, fi.status, fi.fic_updated, f.last_seen
           FROM follows f
           JOIN fic_info fi ON fi.work_id = f.work_id
           WHERE f.follower_id = $1 AND f.work_id IS NOT NULL
           ORDER BY fi.fic_updated DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

Note the JOIN: `follows f JOIN fic_info fi ON fi.work_id = f.work_id`.
The follow stores a `work_id`; the feed needs the story's *fresh
metadata* — title, author, word count, chapter count, and the
`fic_updated` timestamp that decides recency. One join, everything the
card needs. (And `f.work_id IS NOT NULL` filters to work-follows only —
author-follows need a different expansion, which the feed handler
doesn't do yet. Another "file it and move on" moment in a real
codebase.)

The handler turns those rows into the feed with the `is_new`
comparison:

```rust
pub async fn updates_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let updates = queries::list_followed_work_updates(&state.db, user_id).await?;
    let unseen = queries::count_unseen_followed_updates(&state.db, user_id).await?;

    let now = chrono::Utc::now();
    let items: Vec<Value> = updates
        .into_iter()
        .map(|u| {
            let is_new = match u.last_seen {
                Some(last) => u.fic_updated > last,
                None => true,
            };
            json!({
                "follow_id": u.follow_id,
                "work_id": u.work_id,
                "url_id": u.url_id,
                "title": u.title,
                "author": u.author,
                "words": u.words,
                "chapters": u.chapters,
                "status": u.status,
                "fic_updated": u.fic_updated.to_rfc3339(),
                "updated_ago": relative_ago(now, u.fic_updated),
                "is_new": is_new,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "unseen_count": unseen,
    })))
}
```

The `is_new` logic is a one-liner: no `last_seen` at all → NEW (you
followed it and never looked); `fic_updated` after `last_seen` → NEW
(it changed since you looked); otherwise → seen. And the response
carries a human-readable `updated_ago` string built server-side by a
tiny helper — the same "X minutes ago" text the frontend would
otherwise have to compute:

```rust
/// Human "updated X ago" string (mirrors the frontend util, server-side).
fn relative_ago(now: chrono::DateTime<chrono::Utc>, then: chrono::DateTime<chrono::Utc>) -> String {
    let diff = now.signed_duration_since(then);
    if diff.num_seconds() < 60 {
        "less than a minute ago".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hours ago", diff.num_hours())
    } else {
        format!("{} days ago", diff.num_days())
    }
}
```

💡 **Key Concept — relative time: one function, four branches, unit
tests.** "3 minutes ago" is a tiny feature with a surprisingly sharp
edge — the units change at 60s, 60m, 24h, and each boundary is off-by-
one bait. That's why it has a test:

```rust
#[test]
fn relative_ago_handles_units() {
    let now = chrono::Utc::now();
    assert!(relative_ago(now, now).contains("less than a minute"));
    assert!(relative_ago(now, now - chrono::Duration::minutes(5)).contains("5 minutes"));
    assert!(relative_ago(now, now - chrono::Duration::hours(3)).contains("3 hours"));
    assert!(relative_ago(now, now - chrono::Duration::days(2)).contains("2 days"));
}
```

Every time you write a time-formatting function, write the boundary
tests in the same commit — time math is where "it works on my
machine" goes to die.

Marking a follow "seen" is the idempotent UPDATE we know by heart now:

```rust
/// Mark a follow's `last_seen` as now (idempotent; no row → no-op).
pub async fn mark_follow_seen(pool: &PgPool, follow_id: i64, user_id: i32) -> AppResult<bool> {
    let result = sqlx::query("UPDATE follows SET last_seen = NOW() WHERE id = $1 AND follower_id = $2")
        .bind(follow_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
```

Note the `AND follower_id = $2` — you can only mark *your own* follows
as seen. The ownership check is in the WHERE clause, not in a separate
SELECT. (And the route for this lives in `src/routes/updates.rs` as
`POST /api/v1/follows/{id}/seen`, wired up in `server.rs` right next
to the feed.)

The unseen *count* — the little number in the nav — is the same
comparison as an aggregate:

```rust
/// Number of followed works whose fic_updated is newer than last_seen
/// (used for the nav "Updates" badge count).
pub async fn count_unseen_followed_updates(pool: &PgPool, user_id: i32) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*)
           FROM follows f
           JOIN fic_info fi ON fi.work_id = f.work_id
           WHERE f.follower_id = $1
             AND f.work_id IS NOT NULL
             AND (f.last_seen IS NULL OR fi.fic_updated > f.last_seen)"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

That `(f.last_seen IS NULL OR fi.fic_updated > f.last_seen)` is the
same `is_new` logic in SQL form — "never looked, or changed since
looked". One concept, two implementations (Rust and SQL), kept in
sync by being written within sight of each other.

### 32.4 Refresh: where the update actually comes from

The updates feed is only useful if works *get* updated. FicHub's
refresh endpoint (`POST /api/v1/works/{url_id}/refresh`, in
`src/routes/updates.rs`) is the mechanism: it re-scrapes a fic, and if
anything changed, it bumps the version and *notifies the followers*.
The change detection is a three-way comparison:

```rust
    let changed = meta.updated > existing.fic_updated.timestamp_millis()
        || meta.chapters != existing.chapters
        || meta.words != existing.words;

    if !changed {
        return Ok(Json(json!({
            "err": 0,
            "status": "no_change",
            "url_id": url_id,
        })));
    }
```

New update timestamp, more chapters, more words — any of the three
means "something happened". If nothing changed, the endpoint answers
`no_change` and stops: no version bump, no notifications, no waste.

If something *did* change, the handler persists the fresh metadata,
bumps the export-cache version (the `fic_version_bump` mechanism from
Part 5 — the next EPUB export regenerates), and then does the
community thing:

```rust
    // Notify followers (respects their work_update preference).
    let notified = queries::notify_work_followers(&state.db, work_id, &meta.title).await?;

    Ok(Json(json!({
        "err": 0,
        "status": "ok",
        "url_id": url_id,
        "version_bump": bump,
        "notified": notified,
    })))
```

### 32.5 The notifications table: an inbox per user

The notification insert is where all of Part 7's threads finally come
together. The schema (migration 003) is a classic per-user inbox:

```sql
CREATE TABLE IF NOT EXISTS notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL,  -- 'comment_reply', 'follow_update', 'work_update', 'badge_earned', 'curator_promotion', 'recommendation'
    title TEXT NOT NULL,
    body TEXT,
    link TEXT,                        -- URL to the relevant resource
    reference_type TEXT,              -- 'comment', 'work', 'user', 'badge'
    reference_id TEXT,                -- ID of the referenced resource
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, created_at DESC) WHERE is_read = FALSE;
CREATE INDEX IF NOT EXISTS idx_notifications_user_all ON notifications(user_id, created_at DESC);
```

Notice the *partial index*: `idx_notifications_user` indexes only
unread rows — the hot query ("how many unread?") never touches read
notifications. That's a classic PostgreSQL optimization: `WHERE
is_read = FALSE` on the index means the unread-count scan is tiny, and
the "all" index serves the full inbox view.

The insert function is the boring-but-beautiful foundation:

```rust
/// Create a notification
pub async fn create_notification(
    pool: &PgPool,
    user_id: i32,
    notification_type: &str,
    title: &str,
    body: Option<&str>,
    link: Option<&str>,
    reference_type: Option<&str>,
    reference_id: Option<&str>,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO notifications (user_id, notification_type, title, body, link, reference_type, reference_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .bind(title)
    .bind(body)
    .bind(link)
    .bind(reference_type)
    .bind(reference_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

And the fan-out — `notify_work_followers` — shows the pattern every
"tell everyone" feature uses: query the audience, check preferences,
insert one row per recipient:

```rust
pub async fn notify_work_followers(
    pool: &PgPool,
    work_id: i32,
    work_title: &str,
) -> AppResult<i64> {
    let followers = sqlx::query_scalar::<_, i32>(
        "SELECT follower_id FROM follows WHERE work_id = $1",
    )
    .bind(work_id)
    .fetch_all(pool)
    .await?;

    for follower_id in &followers {
        // Check preferences
        let prefs = get_notification_preferences(pool, *follower_id).await.ok();
        if prefs.map(|p| p.work_update).unwrap_or(true) {
            create_notification(
                pool,
                *follower_id,
                "work_update",
                &format!("\"{}\" has been updated", work_title),
                None,
                Some(&format!("/api/works/{}", work_id)),
                Some("work"),
                Some(&work_id.to_string()),
            ).await.ok();
        }
    }

    Ok(followers.len() as i64)
}
```

Deconstruct this — it's a miniature event system:

1. **Query the audience**: `SELECT follower_id FROM follows WHERE
   work_id = $1` — one query, the whole fan-out list.
2. **Respect preferences**: `get_notification_preferences(...).ok()`
   — if preferences can't be loaded, the `.ok()` turns the error into
   `None`, and `unwrap_or(true)` *defaults to sending*. **Fail-open
   again** — a preference lookup failure must not silence a useful
   update. Notice the subtle default: `work_update` defaults to TRUE
   in the schema, so "default to sending" matches the schema's
   contract.
3. **Insert per recipient**: one `create_notification` per follower.
   N+1 queries, sure — but the N is small (followers of one fic), and
   each insert is trivial. For a fanfic platform, this is the right
   call; for a site with 100k-follower accounts, you'd move to a
   single `INSERT ... SELECT` or a queue.
4. **Best-effort everywhere**: `create_notification(...).await.ok()` —
   a failed notification insert is logged and ignored. The refresh
   already succeeded; notifications are a side effect.

The preferences table gives each user the kill-switches (migration
003):

```sql
CREATE TABLE IF NOT EXISTS notification_preferences (
    user_id INT4 PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    comment_reply BOOLEAN NOT NULL DEFAULT TRUE,
    follow_update BOOLEAN NOT NULL DEFAULT TRUE,
    work_update BOOLEAN NOT NULL DEFAULT TRUE,
    badge_earned BOOLEAN NOT NULL DEFAULT TRUE,
    curator_promotion BOOLEAN NOT NULL DEFAULT TRUE,
    recommendation BOOLEAN NOT NULL DEFAULT FALSE,
    email_digest TEXT NOT NULL DEFAULT 'never' CHECK (email_digest IN ('instant', 'daily', 'weekly', 'never')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Five booleans plus a digest frequency — and note `recommendation`
defaults to **FALSE**: marketing-flavored notifications are opt-in
while user-valuable ones (replies, updates) are opt-out. That's a
privacy/product principle encoded directly in schema defaults.

### 32.6 The notifications API: read, mark, and the preference editor

The endpoints in `src/routes/notifications.rs` are the calmest code in
this part — everything hard was already built. Listing with paging:

```rust
/// GET /api/v1/notifications — list notifications for current user
pub async fn list_notifications_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<NotifQueryParams>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;
    let limit = params.limit.unwrap_or(20).min(50).max(1);
    let offset = params.offset.unwrap_or(0).max(0);

    let notifs = queries::list_notifications(&state.db, user_id, limit, offset).await?;
    let unread_count = queries::get_unread_notification_count(&state.db, user_id).await?;

    let items: Vec<Value> = notifs.into_iter().map(|n| {
        json!({
            "id": n.id,
            "notification_type": n.notification_type,
            "title": n.title,
            "body": n.body,
            "link": n.link,
            "reference_type": n.reference_type,
            "reference_id": n.reference_id,
            "is_read": n.is_read,
            "created_at": n.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "notifications": items,
        "unread_count": unread_count,
    })))
}
```

The `limit.unwrap_or(20).min(50).max(1)` chain is the FicHub paging
idiom — default, cap, floor, in one expression. And the response packs
the unread count into *every* listing call, so the nav badge updates
without a second request. Mark-read is the ownership-scoped UPDATE we
now recognize instantly:

```rust
/// Mark notification as read
pub async fn mark_notification_read(pool: &PgPool, user_id: i32, notification_id: i64) -> AppResult<bool> {
    let result = sqlx::query(
        "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2",
    )
    .bind(notification_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
```

And the preference editor shows the *partial-update* pattern — PUT
with only the fields you want to change:

```rust
/// PUT /api/v1/notifications/preferences
pub async fn update_preferences_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PrefsBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::BadRequest(401, "Login required".into()))?;

    let mut prefs = queries::get_notification_preferences(&state.db, user_id).await?;

    if let Some(v) = body.comment_reply { prefs.comment_reply = v; }
    if let Some(v) = body.follow_update { prefs.follow_update = v; }
    if let Some(v) = body.work_update { prefs.work_update = v; }
    if let Some(v) = body.badge_earned { prefs.badge_earned = v; }
    if let Some(v) = body.curator_promotion { prefs.curator_promotion = v; }
    if let Some(v) = body.recommendation { prefs.recommendation = v; }
    if let Some(ref d) = body.email_digest {
        if !["instant", "daily", "weekly", "never"].contains(&d.as_str()) {
            return Err(AppError::BadRequest(-1, "email_digest must be one of: instant, daily, weekly, never".into()));
        }
        prefs.email_digest = d.clone();
    }

    queries::update_notification_preferences(&state.db, user_id, &prefs).await?;

    Ok(Json(json!({ "err": 0, "msg": "Preferences updated" })))
}
```

💡 **Key Concept — read-modify-write for partial updates.**
`PrefsBody` fields are all `Option<bool>` — the client sends *only*
the toggles it wants to change. The handler loads the current prefs
into a local struct, applies the `Some(...)` fields, validates the
enumerated `email_digest` value, and writes the whole row back. That's
read-modify-write, and it's the cleanest way to do PATCH-style
semantics without a pile of `CASE WHEN` SQL. The race risk (two
concurrent updates clobbering each other) is real but acceptable for a
settings page; if it ever mattered, you'd do a single `UPDATE ... SET
field = COALESCE($1, field)` per field instead.

🧪 **Try It Yourself — the community loop, end to end.**
Now the full story: follow a work, fake an update, and watch the
notification arrive. You'll need *two* accounts to see the whole loop
(or just watch your own):

```bash
# 1. Follow a work you bookmarked in Chapter 30
curl -s -X POST http://localhost:8080/api/follows \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"target_type\": \"work\", \"target_id\": $WORK}"
# → {"err":0,"msg":"Following","follow_id":1}

# 2. Try following it again — idempotent, no error:
curl -s -X POST http://localhost:8080/api/follows \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d "{\"target_type\": \"work\", \"target_id\": $WORK}"

# 3. Check your follow status:
curl -s "http://localhost:8080/api/follows/check/work/$WORK" \
  -H "Authorization: Bearer <TOKEN>"
# → {"err":0,"is_following":true,"follow_id":1}

# 4. Look at the updates feed — brand new follows have no last_seen, so
#    everything is NEW:
curl -s http://localhost:8080/api/v1/updates -H "Authorization: Bearer <TOKEN>"

# 5. Mark it seen, then check the unseen count dropped:
curl -s -X POST http://localhost:8080/api/v1/follows/1/seen \
  -H "Authorization: Bearer <TOKEN>"
curl -s http://localhost:8080/api/v1/updates -H "Authorization: Bearer <TOKEN>"

# 6. Open the notifications inbox:
curl -s http://localhost:8080/api/notifications -H "Authorization: Bearer <TOKEN>"
curl -s http://localhost:8080/api/notifications/unread-count -H "Authorization: Bearer <TOKEN>"

# 7. Flip your notification preferences:
curl -s -X PUT http://localhost:8080/api/notifications/preferences \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d '{"work_update": false, "email_digest": "daily"}'
```

If you can get a second account to `refresh` a work you follow, the
`notify_work_followers` fan-out will drop a `work_update` notification
into your inbox — the entire loop of Part 7 running in one curl
sequence. That moment — a row in `follows`, a row in `notifications`,
joined by a re-scrape — is the community working.

---

That's the follow system: a polymorphic table with one CHECK, a
`last_seen` timestamp doing the work of a badge, and a notification
fan-out that respects each reader's preferences. Which means Part 7 is
complete — the download tool now knows its users, trusts them in
tiers, lets them build shelves, talk about stories, and keep up with
the works they love.

But a community doesn't just *talk* — it *recommends*. The bookmarks,
ratings, and follows you created in these five chapters aren't just
shelf furniture: they're *signals*. In **Part 8 — Recommendations &
Discovery**, we'll see how FicHub mines co-bookmarking patterns,
builds the collaborative filtering matrix from the exact
`work_ratings` rows you just wrote, and turns "people who bookmarked
this also bookmarked..." into a recommendation engine that learns what
you like. Your bookmarks are about to get a second job. See you there.

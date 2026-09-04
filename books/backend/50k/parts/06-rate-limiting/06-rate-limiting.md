# Part 6: Putting Up the Bouncers

The export handler is the conductor of the orchestra. It coordinates every subsystem — the scraper registry to find the right scraper, the scraper itself to fetch metadata and chapters, the EPUB and HTML generators to create downloadable files, the disk cache to store them, the database to record everything, and the JSON response to tell the user where to find their downloads. It all happens in one request, and most of the time, the cache hit path means it takes under 100 milliseconds.

But there's a problem we haven't solved yet. What happens when someone sends thousands of requests per minute? What if a bot scrapes our server aggressively? We need to limit how often each IP address can make requests. That's where rate limiting comes in.

Rate limiting is like having a bouncer at the door of a nightclub. Regular guests walk right in. Someone trying to push through with a crowd of friends? The bouncer holds up a hand and says, "Not so fast." A bot hammering our endpoint thousands of times per second? "You're out."

Without rate limiting, a single misbehaving client could consume all of FicHub's CPU, memory, and network bandwidth — starving every other user. The scraper would time out, the database would fill up with useless entries, and legitimate fans would see error pages. Rate limiting keeps the system fair: everyone gets a turn, nobody gets too many, and the server stays healthy.

To build this bouncer, we need a fast, shared memory store that all our handlers can access simultaneously. That's Redis.

---

## Chapter 20: Redis Basics

### What Is Redis?

Redis stands for **RE**mote **DI**ctionary **S**erver. It's an in-memory data store — basically a giant key-value dictionary that lives in RAM instead of on disk. Because everything is in memory, Redis is incredibly fast. We're talking about getting or setting a value in under a millisecond, even with thousands of concurrent connections.

Think of Redis like a massive whiteboard in a shared office. Anyone in the office can walk up to the whiteboard, write something down (`SET key value`), read what's on it (`GET key`), or erase it (`DEL key`). Because it's a shared whiteboard, everyone sees the same data at the same time.

Redis was created by Salvatore Sanfilippo in 2009 and has become one of the most popular databases in the world. It's used by Twitter, GitHub, Stack Overflow, and thousands of other companies. The reason is simple: Redis is fast, reliable, and easy to use. For our rate limiting use case, it's the perfect tool.

> 💡 **Key Concept: In-Memory Means Fast**
>
> Databases like PostgreSQL store data on disk, which is persistent but slower. Redis stores everything in RAM, which is blazing fast but means data disappears when Redis restarts. That's why Redis is perfect for temporary, high-speed data like rate limit counters, session tokens, and caches — things you need right now, not forever.
>
> There's a saying in computer science: "There are only two hard things: cache invalidation and naming things." Redis is the first part of that problem. It's a cache — and for rate limiting, we don't need the data to survive a restart. If Redis loses all its counters, people just get a fresh set of tokens. No harm done.

### Installing Redis

Let's get Redis running on your machine. On most Linux distributions:

```bash
# Arch Linux
sudo pacman -S redis

# Ubuntu/Debian
sudo apt install redis-server

# macOS
brew install redis
```

Start the Redis server:

```bash
redis-server --daemonize yes
```

Test that it's running:

```bash
$ redis-cli ping
PONG
```

That `PONG` response means Redis is alive and listening. By default, Redis listens on port 6379. You can verify with:

```bash
$ redis-cli INFO server | head -5
# Server
redis_version:7.2.0
tcp_port:6379
```

Redis is a single-threaded server — it processes commands one at a time, but because it's all in memory and each command is so fast, it can handle over 100,000 commands per second on a modest machine. That's more than enough for FicHub's rate limiting needs.

### Redis as a Key/Value Store

Redis can do more than just store strings, but let's start with the basics. The three fundamental commands are:

```bash
# Set a key to a value
SET visitor_count 0

# Get the value
GET visitor_count
# "0"

# Increment it atomically
INCR visitor_count
# (integer) 1

# Delete a key
DEL visitor_count
```

The `INCR` command is special — it atomically adds 1 to the current value and returns the new value. No race conditions, no "read-modify-write" bugs. Redis does the read, the increment, and the write as one indivisible operation.

In Rust, using the `redis` crate:

```rust
use redis::AsyncCommands;

// SET a value
redis::cmd("SET")
    .arg("visitor_count")
    .arg("0")
    .query_async(&mut conn)
    .await?;

// GET the value
let count: String = redis::cmd("GET")
    .arg("visitor_count")
    .query_async(&mut conn)
    .await?;

// INCR the value (atomic increment)
let new_count: u32 = redis::cmd("INCR")
    .arg("visitor_count")
    .query_async(&mut conn)
    .await?;

// DEL the value
redis::cmd("DEL")
    .arg("visitor_count")
    .query_async(&mut conn)
    .await?;
```

The `redis` crate gives us a `MultiplexedConnection` — a single TCP connection that can handle multiple concurrent requests without blocking. This is important for our async Axum server, where multiple handlers might need Redis at the same time. One handler checking a rate limit doesn't block another handler from reading a cache key.

> ⚠️ **Watch Out: Don't Share Regular Connections Across Tasks**
>
> A regular `async connection` is bound to a single task at a time. If two handlers try to use it simultaneously, one will wait. That's why FicHub uses `MultiplexedConnection` — it lets multiple async tasks use the same connection concurrently. You get it by calling `client.get_multiplexed_async_connection().await`.
>
> Think of a regular connection like a single-lane bridge — only one car at a time. A `MultiplexedConnection` is like a multi-lane highway — many cars can cross simultaneously without waiting for each other.

### Redis Data Types

Redis isn't just strings. It has five core data types, each useful for different patterns:

**Strings** — The basic building block. Store numbers, text, serialized JSON, anything up to 512 MB. This is what you use for simple counters and flags.

**Lists** — Ordered sequences. Push items to the front (`LPUSH`) or back (`RPUSH`), pop from either end (`LPOP`, `RPOP`). Great for queues, activity feeds, and message lists.

**Sets** — Unordered collections of unique values. Add items (`SADD`), remove them (`SREM`), check if something exists (`SISMEMBER`). Perfect for "is this IP in our blocklist?" or "what tags does this story have?"

**Hashes** — Maps of key-value pairs within a single key. Like a mini-dictionary with field names. We'll use these for our token buckets — storing `value` and `last_drain` together under one key. Use `HMSET` to set multiple fields and `HMGET` to read them.

**Sorted Sets** — Sets with scores. Automatically sorted by score. Useful for leaderboards, time-ordered data, and priority queues.

For rate limiting, we'll use **hashes** for the token buckets and **strings** for simple counters. Each hash key represents a bucket (like `rate:ip:203.0.113.42`) and stores the current token count and the last time it was checked.

### Connecting to Redis in Rust

Add the `redis` crate to your `Cargo.toml`:

```toml
[dependencies]
redis = { version = "1.4", features = ["aio", "tokio-comp"] }
```

The `aio` feature enables async support, and `tokio-comp` makes it work with Tokio (which Axum uses under the hood). Without these features, you'd get a blocking Redis client, which would freeze your async handlers every time they talk to Redis.

Here's how FicHub connects to Redis in `server.rs`:

```rust
// Create a Redis client from the URL
let redis_client = redis::Client::open(config.redis_url.as_str())
    .expect("Invalid Redis URL");

// Get a multiplexed async connection
let redis_conn = redis_client.get_multiplexed_async_connection()
    .await
    .expect("Failed to connect to Redis");
```

The `config.redis_url` comes from the `REDIS_URL` environment variable — something like `redis://127.0.0.1:6379`. The `Client::open` method parses this URL and creates a client that knows where Redis is. The `get_multiplexed_async_connection` method creates that multi-lane highway we talked about.

That `redis_conn` is now shared across the entire application. Every handler that needs Redis can clone it and use it concurrently. We store it in `AppState`:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    // ... other fields
}
```

### Why Redis for Rate Limiting?

You might wonder: why not just keep a counter in a Rust `HashMap`? Why add Redis to the mix?

Three reasons:

**Atomicity.** Redis commands are atomic. When two requests hit our server at the exact same millisecond, Redis guarantees that `INCR` happens one after the other, not both reading the same value and writing back the wrong count. A `HashMap` in Rust would need explicit locking (like a `Mutex` or `RwLock`) to get the same guarantee, and locks add overhead and complexity.

**Shared state.** If you run multiple FicHub instances behind a load balancer, each instance has its own Rust memory. A counter in one instance's `HashMap` is invisible to the others. Redis lives outside the application — all instances share the same counters. Rate limits work across your entire server fleet. User X hitting Instance A and Instance B will share the same token bucket.

**Speed.** Redis operations take under a millisecond. That's fast enough to check on every single request without slowing down response times. A PostgreSQL query might take 5-50 milliseconds. Redis is 10-100 times faster for simple key lookups.

> 🧪 **Try It Yourself: Play with Redis**
>
> Open a terminal with `redis-cli` and try these commands:
>
> ```bash
> # Create a counter for your IP
> SET rate:ip:127.0.0.1 '{"value":100,"last_drain":0}'
>
> # Read it back
> GET rate:ip:127.0.0.1
>
> # Use a hash for structured data (this is what we'll actually use)
> HMSET bucket:demo value 50 last_drain 1000.0
> HMGET bucket:demo value last_drain
>
> # Set a TTL — auto-delete after 60 seconds
> EXPIRE bucket:demo 60
> TTL bucket:demo
>
> # Delete it
> DEL bucket:demo
> ```
>
> Notice how `HMSET` stores two fields under one key. This is exactly how FicHub's token buckets work — `value` holds the current token count, and `last_drain` holds the last time we checked. The `EXPIRE` command is neat too — it automatically deletes a key after a number of seconds, which is handy for temporary data.

---

## Chapter 21: Token Bucket Rate Limiter

### What Is Rate Limiting?

Rate limiting is the art of saying "no" — politely — when someone asks for too much too fast. Without rate limiting, a single user (or bot) could send a million requests per minute, overwhelming your server and blocking everyone else.

Think of it like a highway on-ramp. During rush hour, traffic lights at the on-ramp let cars enter the highway one at a time, spaced a few seconds apart. Without the light, hundreds of cars would try to merge at once, causing a traffic jam. Rate limiting is the traffic light for your server.

There are many algorithms for rate limiting. Some are simple (count requests in a time window), some are complex (sliding window logs). FicHub uses the **token bucket** algorithm. It's elegant, efficient, and easy to implement in Redis.

> 💡 **Key Concept: The Token Bucket Algorithm**
>
> Imagine a bucket that holds tokens. Every request costs one token. Tokens refill at a steady rate. If you have tokens, your request goes through. If the bucket is empty, you have to wait until more tokens arrive.
>
> The bucket has two parameters:
> - **Capacity** — The maximum number of tokens the bucket can hold
> - **Flow rate** — How many tokens are added per second
>
> Think of it like a water bucket under a faucet. Water (tokens) drips in at a steady rate. You can take water out whenever you want, but if the bucket is dry, you have to wait for the faucet to refill it. The bucket's size limits how much water you can take at once.

This gives us natural burst tolerance. If someone hasn't made requests in a while, their bucket fills up to capacity. They can make a few rapid requests (the "burst"), but once the bucket drains, they have to wait for the flow rate to refill it. This is exactly what you want for a web server — allow occasional bursts of activity, but prevent sustained high-volume abuse.

### Implementing the Bucket in Redis

FicHub's token bucket lives in Redis as a hash with two fields:

```
KEY: rate:ip:203.0.113.42
FIELDS:
  value     → current number of tokens (float)
  last_drain → timestamp when we last checked (float)
```

When a request comes in, we:
1. Read `value` and `last_drain` from the hash
2. Calculate how many new tokens have arrived since `last_drain`
3. Add them to `value` (but never exceed capacity)
4. Check if there are enough tokens for the request
5. If yes, subtract the tokens and allow the request
6. If no, calculate how long to wait and tell the user

The tricky part is doing all of this atomically. If two requests read the same `value` and both try to subtract tokens, we'd lose track. Redis solves this with **Lua scripts**.

A Lua script runs entirely inside Redis. No other command can interrupt it mid-execution. It's like a mini-program that runs inside the database itself. Redis reads the script once, compiles it, and executes it as a single operation. No other command can sneak in between the read and the write.

Here's FicHub's token bucket Lua script, found in `src/limiter/redis_bucket.rs`:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

-- Read current bucket state
local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

-- Get current time with microsecond precision
local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

-- Initialize bucket if this is the first request
if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

-- Calculate new tokens based on elapsed time
local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)

-- Check if we have enough tokens
local allowed = new_tokens - requested

if allowed >= 0 then
    -- Enough tokens! Subtract and update
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1  -- -1 means "allowed"
else
    -- Not enough tokens. Calculate wait time
    local wait = (requested - new_tokens) / flow
    return wait  -- seconds until enough tokens arrive
end
```

Let's walk through this step by step:

1. **Read the current state** — We fetch `value` (how many tokens are in the bucket) and `last_drain` (when we last checked). `HMGET` reads both fields in one command.

2. **Get the current time** — Redis's `TIME` command returns seconds and microseconds as two separate values. We combine them for sub-second precision. This matters — if our clock granularity is only in seconds, two requests in the same second would see the same timestamp and not refill any tokens.

3. **Initialize on first request** — If this IP has never been seen before, we give them a full bucket. That's the fair thing — give newcomers a chance. No one should be rate limited on their very first request.

4. **Refill tokens** — We calculate how much time has passed since the last check, multiply by the flow rate, and add to the current value. The `math.min` ensures we never exceed capacity. If someone waits a long time between requests, their bucket fills to the max and stops — you can't "save up" unlimited tokens.

5. **Check and subtract** — If there are enough tokens, subtract one and return `-1` (meaning "allowed"). If not, calculate how many seconds until enough tokens arrive. The client can use this number to know when to retry.

> ⚠️ **Watch Out: The Wait Time Calculation**
>
> The formula `(requested - new_tokens) / flow` gives the exact number of seconds until enough tokens will have arrived. If you requested 5 tokens but only 2 are available, and the flow rate is 1 token per second, you'd wait `(5 - 2) / 1 = 3 seconds`. This is precise to the millisecond thanks to our sub-second timestamps.

### Loading the Lua Script

When the `RedisBucketLimiter` starts, it loads this Lua script into Redis and gets back a SHA hash — a fingerprint of the script. Later, instead of sending the entire script on every request, we just send the SHA. Redis recognizes it and runs the cached script.

```rust
pub async fn new(
    redis_conn: redis::aio::MultiplexedConnection,
    dynamic_rate_limit: bool,
) -> Result<Self, redis::RedisError> {
    // The Lua script (shown above)
    let lua_script = r#"
        -- ... (the full script from above)
    "#;

    let mut conn = redis_conn.clone();

    // Load the script into Redis, get back a SHA
    let lua_sha: String = redis::cmd("SCRIPT")
        .arg("LOAD")
        .arg(lua_script)
        .query_async(&mut conn)
        .await?;

    Ok(RedisBucketLimiter {
        redis: redis_conn,
        lua_sha,
        dynamic_rate_limit,
        static_delay_base: 0.1,
        datacenter_ips: Arc::new(RwLock::new(HashSet::new())),
        global_capacity: 150.0,
        global_flow: 30.0,
        ip_capacity: 30.0,
        ip_flow: 0.116,
    })
}
```

Notice the two sets of parameters: **global** and **ip**. FicHub has two levels of rate limiting:

- **Global bucket** — Limits the total request rate across ALL users. Capacity: 150 tokens, refills at 30 tokens/second. This protects the server from being overwhelmed even if each individual user is polite. If 100 users all request at once, the global bucket drains and forces everyone to wait.
- **Per-IP bucket** — Limits each individual IP address. Capacity: 30 tokens, refills at about 0.116 tokens/second (roughly one request every 8.6 seconds). This prevents any single user from hogging all the global tokens.

> ⚠️ **Watch Out: Token Refill Rate Math**
>
> The per-IP flow rate of 0.116 means about 7 tokens per minute, or roughly one request every 8.6 seconds. This is intentionally slow — FicHub doesn't need to handle thousands of rapid requests from a single user. If someone wants a different story, they wait a few seconds. That's fine for a download server. A social media API might need 100 requests per second per user, but a fanfiction download server? One every 8.6 seconds is plenty.

### The RedisBucketLimiter Struct

Let's put the whole picture together. Here's the `RedisBucketLimiter`:

```rust
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,

    // Datacenter IP cache — a set of known bot IPs
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,

    // Global rate limit config
    global_capacity: f64,
    global_flow: f64,

    // Per-IP rate limit config
    ip_capacity: f64,
    ip_flow: f64,
}
```

The `datacenter_ips` field uses `Arc<RwLock<HashSet<IpAddr>>>` — a reference-counted, read-write locked hash set. The `Arc` lets multiple async tasks share ownership (each task gets a pointer to the same data). The `RwLock` lets many tasks read simultaneously but only one task write at a time. This is the standard pattern for shared mutable state in async Rust. We need this because `load_datacenter_ips` writes to the set at startup, while `is_datacenter_ip` reads from it on every request — and those can happen concurrently.

### Checking a Bucket

The `check_bucket` method sends the Lua script to Redis:

```rust
async fn check_bucket(
    &self,
    key: &str,
    capacity: f64,
    flow: f64,
) -> Result<f64, redis::RedisError> {
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)            // number of keys
        .arg(key)           // the Redis key
        .arg(1.0)           // requested tokens (one per request)
        .arg(capacity)      // bucket capacity
        .arg(flow)          // refill rate
        .query_async(&mut conn)
        .await?;
    Ok(result)
}
```

`EVALSHA` runs the Lua script we loaded earlier. The arguments pass through to the script: `KEYS[1]` is the bucket key, `ARGV[1]` is how many tokens we're requesting, `ARGV[2]` is capacity, and `ARGV[3]` is the flow rate. It returns `-1.0` if the request is allowed, or a positive number (seconds to wait) if the bucket is empty.

### The check_ip Method

This is the main entry point. It checks both buckets in sequence:

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    // If dynamic rate limiting is off, just add a small random delay
    if !self.dynamic_rate_limit {
        let delay = self.static_delay_base
            + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(
            tokio::time::Duration::from_secs_f64(delay)
        ).await;
        return RateLimitResult::Allowed;
    }

    // Block datacenter IPs immediately
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    // Check the global bucket first
    let global_wait = self.check_bucket(
        "rate:global",
        self.global_capacity,
        self.global_flow,
    ).await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Check the per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(
        &ip_key,
        self.ip_capacity,
        self.ip_flow,
    ).await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

The method returns a `RateLimitResult` enum:

```rust
pub enum RateLimitResult {
    Allowed,       // Go ahead!
    Wait(u64),     // Wait this many seconds
    Blocked,       // No way. You're a datacenter IP.
}
```

Notice the order: **datacenter check → global bucket → per-IP bucket**. We block the worst offenders first (datacenter IPs), then check the overall server capacity, then check the individual user. It's like a bouncer checking your ID at the door, then checking the club's capacity, then checking if you've been to this club too many times today.

Also notice the `.unwrap_or(-1.0)` — if Redis is down or the connection fails, we treat it as "allowed." This is a deliberate design choice. We'd rather let a few extra requests through during a Redis outage than block everyone. Rate limiting should protect the server, not become a single point of failure.

The `ceil()` call rounds the wait time up to the nearest whole number. We don't want to tell a client "wait 0.3 seconds" — that's too precise. "Wait 1 second" is better.

### Penalizing Failures

When a request fails (the scraper errors out, the upstream site returns a 500), we penalize the IP by requesting extra tokens. This is the `report_failure` method:

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize(
        "rate:global",
        self.global_capacity,
        self.global_flow,
    ).await;

    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(
        &ip_key,
        self.ip_capacity,
        self.ip_flow,
    ).await;
}

async fn penalize(
    &self,
    key: &str,
    capacity: f64,
    flow: f64,
) -> Result<(), redis::RedisError> {
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)        // Request 1.5 extra tokens (more than normal!)
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

The penalty requests **1.5 tokens** instead of the usual 1.0. This means a failed request costs 50% more than a successful one. If someone keeps hitting endpoints that error out, their bucket drains faster, and they have to wait longer before their next request. It's a gentle nudge: "Maybe try again later instead of hammering the server."

Notice that `report_failure` penalizes both the global and per-IP buckets. A bot causing lots of failures doesn't just hurt itself — it also eats into the global token pool, slowing everyone down. The penalty reflects this: if you're causing trouble, both your personal budget and the shared budget take a hit.

The `let _ =` pattern on the `penalize` calls means we're ignoring errors. If Redis is down during a failure report, that's okay — we just skip the penalty. We don't want a Redis failure to cause a cascade of problems.

> 🧪 **Try It Yourself: Simulate Rate Limiting**
>
> Here's a pure-Rust token bucket that matches the Lua script logic. Create a new Rust project and try it:
>
> ```rust
> struct TokenBucket {
>     value: f64,
>     last_drain: f64,
>     capacity: f64,
>     flow: f64,
> }
>
> impl TokenBucket {
>     fn new(capacity: f64, flow: f64, now: f64) -> Self {
>         TokenBucket {
>             value: capacity,
>             last_drain: now,
>             capacity,
>             flow,
>         }
>     }
>
>     /// Returns -1.0 if allowed, or wait seconds if denied
>     fn request(&mut self, requested: f64, now: f64) -> f64 {
>         let elapsed = now - self.last_drain;
>         let new_tokens = (self.value + elapsed * self.flow)
>             .min(self.capacity);
>         let allowed = new_tokens - requested;
>
>         if allowed >= 0.0 {
>             self.value = allowed;
>             self.last_drain = now;
>             -1.0  // allowed
>         } else {
>             (requested - new_tokens) / self.flow  // wait seconds
>         }
>     }
> }
>
> fn main() {
>     let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
>
>     // First request: full bucket, allowed
>     let r = bucket.request(1.0, 0.0);
>     assert_eq!(r, -1.0);
>
>     // Drain the bucket
>     for _ in 0..10 {
>         bucket.request(1.0, 0.0);
>     }
>
>     // Bucket empty, request 1 token → need to wait 0.5s
>     let wait = bucket.request(1.0, 0.0);
>     assert_eq!(wait, 0.5);
>
>     // After 3 seconds, 6 tokens have refilled
>     let r = bucket.request(1.0, 3.0);
>     assert_eq!(r, -1.0);
> }
> ```
>
> Run it, then tweak the `capacity` and `flow` values. What happens when you set flow really high? What about really low? Playing with these numbers helps you feel how the algorithm behaves. Try setting capacity to 1.0 and flow to 0.001 — that's a very strict limiter!

---

## Chapter 22: Datacenter IP Blocking and Middleware

### Why Block Datacenter IPs?

Not all traffic is created equal. A request from someone's home internet connection is probably a real person reading fanfiction on their phone. A request from an AWS EC2 instance or a Google Cloud server? That's almost certainly a bot — a scraper, a crawler, or someone trying to automate downloads at scale.

Datacenter IPs belong to cloud providers, hosting companies, and VPN services. They're not residential. Real people rarely browse fanfiction sites from a server in a data center. By blocking datacenter IPs, we eliminate a huge chunk of bot traffic before it even reaches our rate limiter.

Think of it this way: if you're running a small café and someone walks in wearing a full hazmat suit, you'd probably have questions before seating them. A datacenter IP is the hazmat suit of the internet — it's not always malicious, but it's unusual enough to warrant a second look.

The benefits are significant:
- **Reduced load** — Fewer requests means less CPU, less bandwidth, less database writes
- **Better rate limiting** — Real users aren't competing with bots for token bucket capacity
- **Fewer error logs** — Bots tend to hit endpoints incorrectly, generating noise in your logs
- **Fairness** — Real users get served first; bots have to go through extra hurdles

FicHub keeps a list of known datacenter IP addresses. When a request comes in, we check if the IP is on that list. If it is, we reject the request immediately with `RateLimitResult::Blocked`.

### Loading IP Lists from Config

FicHub loads its datacenter IP lists from files specified in the environment:

```rust
pub async fn load_datacenter_ips(
    &self,
    sources: &[(String, String, String)],
) {
    for (file_path, _type, _tag) in sources {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let mut ips = self.datacenter_ips.write().await;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        if let Ok(ip) = line.parse::<IpAddr>() {
                            ips.insert(ip);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Could not load IP tag file {}: {}",
                    file_path, e
                );
            }
        }
    }
    tracing::info!(
        "Loaded {} datacenter IPs",
        self.datacenter_ips.read().await.len()
    );
}
```

Each source is a tuple of `(file_path, type, tag)`. The files contain one IP address per line. Lines starting with `#` are comments. The `type` and `tag` fields describe the source (we ignore them for now — they're useful for categorizing IPs by severity in a more advanced setup).

Notice the `.write().await` and `.read().await` calls on the `RwLock`. When we're loading IPs, we need exclusive write access — only one task should modify the set at a time. When we're checking if an IP is in the set, we only need read access, and many tasks can do that simultaneously. This is the beauty of `RwLock` — it maximizes concurrency for the common case (reading).

The config comes from the `IP_TAG_SOURCES` environment variable:

```bash
IP_TAG_SOURCES=/etc/fichub/ips/datacenter.txt,tag,datacenter
```

If you have multiple source files, separate them with newlines:

```bash
IP_TAG_SOURCES=/etc/fichub/ips/datacenter.txt,tag,datacenter
/etc/fichub/ips/vpn.txt,tag,vpn
```

FicHub parses this in `config.rs`:

```rust
let ip_tag_sources = std::env::var("IP_TAG_SOURCES")
    .unwrap_or_default()
    .lines()
    .filter_map(|line| {
        let parts: Vec<&str> = line.splitn(3, ',').collect();
        if parts.len() == 3 {
            Some((
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts[2].trim().to_string(),
            ))
        } else {
            None
        }
    })
    .collect();
```

Each line is split into three comma-separated parts. The `splitn(3, ',')` ensures we only split on the first two commas — in case the file path contains commas (unlikely, but defensive programming is good programming).

### The is_datacenter_ip Check

The actual check is straightforward — look up the IP in the hash set:

```rust
fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
    // Best-effort check using the loaded set
    // For production, use a proper prefix tree (ipnet crate)
    false
}
```

> ⚠️ **Watch Out: Exact IP Matching vs. CIDR Ranges**
>
> The simplified version above always returns `false` because we're comparing individual IP addresses. In a real production system, datacenter IPs come in CIDR ranges (like `198.51.100.0/24`), which cover thousands of addresses. You'd use the `ipnet` crate to build a prefix tree and check if an IP falls within any of the loaded ranges.
>
> A CIDR range like `198.51.100.0/24` means "any IP from 198.51.100.0 to 198.51.100.255." That's 256 addresses covered by a single rule. Cloud providers publish their IP ranges in CIDR notation, so you load them as `IpNet` objects and use `ipnet.contains(ip)` to check. The code structure is the same — just swap the `HashSet` for an `IpNet` lookup.
>
> For now, the `HashSet` approach works for individual IPs. A production deployment would use a `PrefixSet` or similar structure for efficient range lookups.

### Integrating Rate Limiting into Handlers

Now that we have the rate limiter, how do we use it in our handlers? FicHub uses Axum's `ConnectInfo` extractor to get the client's IP address, then calls the rate limiter at the top of each handler:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<std::net::SocketAddr>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    // Check rate limit FIRST
    match state.rate_limiter.check_ip(ip).await {
        RateLimitResult::Allowed => {}
        RateLimitResult::Wait(seconds) => {
            return Err(AppError::RateLimited(seconds));
        }
        RateLimitResult::Blocked => {
            return Err(AppError::RateLimited(3600));
        }
    }

    // ... rest of the handler
}
```

When the rate limiter returns `Wait`, we convert it to an `AppError::RateLimited` error. Axum catches this and returns a 429 Too Many Requests response:

```json
{
    "err": -429,
    "msg": "rate limited",
    "retry_after": 12
}
```

The `retry_after` field tells the client how many seconds to wait. A well-behaved client will sleep for that duration before retrying. A not-so-well-behaved client will get rate limited again. The system is self-correcting.

When the rate limiter returns `Blocked`, we return a `RateLimited(3600)` — essentially "come back in an hour." That's generous, honestly. Some services just flat-out block you forever. FicHub gives datacenter IPs a chance to try again later, in case they're legitimate automated tools that just need to slow down.

### The RateLimiter Trait

FicHub defines a trait for rate limiters, so you could swap implementations if needed:

```rust
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize the IP)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}
```

Three methods. `check_ip` is the main gate — it runs the token bucket checks. `report_failure` penalizes IPs that cause errors. `is_datacenter_ip` checks the blocklist.

The `RedisBucketLimiter` implements this trait:

```rust
#[async_trait::async_trait]
impl RateLimiter for RedisBucketLimiter {
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
        // ... (the implementation we covered above)
    }

    async fn report_failure(&self, ip: IpAddr) {
        let _ = self.penalize(
            "rate:global", self.global_capacity, self.global_flow
        ).await;
        let ip_key = format!("rate:ip:{}", ip);
        let _ = self.penalize(
            &ip_key, self.ip_capacity, self.ip_flow
        ).await;
    }

    fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
        false  // simplified
    }
}
```

Because it's a trait, you could write a `MemoryBucketLimiter` for testing (no Redis needed), or a `RedisSlidingWindowLimiter` for a different algorithm. The handlers don't care which implementation you use — they just call `check_ip` and follow the result. This is the power of trait-based design in Rust: define a behavior, implement it multiple ways, and let the consumer choose.

In `AppState`, the rate limiter is stored as a trait object:

```rust
pub struct AppState {
    // ...
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    // ...
}
```

The `Box<dyn RateLimiter>` means "I don't care which concrete type implements this — I just need something that implements the `RateLimiter` trait." This is dynamic dispatch, and it's how FicHub keeps its handlers decoupled from the specific rate limiting implementation.

### Configuring Rate Limits

FicHub has a `dynamic_rate_limit` flag in its configuration:

```rust
let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
    .unwrap_or_else(|_| "true".to_string())
    .parse::<bool>()
    .unwrap_or(true);
```

When `dynamic_rate_limit` is `true`, the full token bucket algorithm runs — Redis, Lua scripts, all of it. When it's `false`, FicHub falls back to a simple random delay:

```rust
if !self.dynamic_rate_limit {
    let delay = self.static_delay_base
        + rand::random::<f64>() * self.static_delay_base;
    tokio::time::sleep(
        tokio::time::Duration::from_secs_f64(delay)
    ).await;
    return RateLimitResult::Allowed;
}
```

This is useful for local development. When you're testing FicHub on your laptop and don't have Redis running, you can set `DYNAMIC_RATE_LIMIT=false` and everything still works — you just get a small random delay instead of real rate limiting. The random component (between 0.1 and 0.2 seconds) prevents request storms even without Redis, which is a nice safety net.

### Putting It All Together

Let's trace a request from start to finish:

**1. Request arrives.** A client sends `GET /api/v0/epub?url=https://fanfiction.net/s/12345` to FicHub.

**2. IP extraction.** Axum extracts the client's IP address from the TCP connection using `ConnectInfo`.

**3. Rate limit check.** The handler calls `state.rate_limiter.check_ip(ip)`.

**4. Datacenter check.** First, the limiter checks if the IP is on the datacenter blocklist. If yes, return `Blocked`. The handler returns a 429 immediately — no scraping, no database queries, nothing. This is the cheapest path through the system.

**5. Global bucket check.** The limiter runs the Lua script against the `rate:global` key. If the global bucket is empty (too many total requests), return `Wait(seconds)`.

**6. Per-IP bucket check.** The limiter runs the Lua script against `rate:ip:{ip}`. If this specific user has been too aggressive, return `Wait(seconds)`.

**7. Request proceeds.** If both buckets have tokens, return `Allowed`. The handler continues: scraper lookup, metadata fetch, EPUB generation, cache storage, database upsert, JSON response.

**8. Failure penalty.** If the scraper fails (upstream site down, timeout, parse error), the handler calls `state.rate_limiter.report_failure(ip)`. The global and per-IP buckets each lose 1.5 extra tokens. The next request from this IP will have to wait a bit longer.

Here's the flow in pseudocode:

```
Request → Extract IP
         → RateLimiter.check_ip(ip)
            → is_datacenter_ip? → Blocked → 429
            → check_bucket("rate:global") → Wait? → 429
            → check_bucket("rate:ip:{ip}") → Wait? → 429
            → Allowed
         → Scraper Lookup
         → Fetch Metadata
         → Generate EPUB/HTML
         → Store in Cache
         → Upsert Database
         → Return JSON
         → (on error) RateLimiter.report_failure(ip)
```

> 💡 **Key Concept: Defense in Depth**
>
> Rate limiting is just one layer of defense. FicHub also blocks `automated=true` query parameters, uses `is_datacenter_ip` checks, and applies rate limiting at multiple levels (global + per-IP + per-tag). No single layer is perfect, but together they make automated abuse much harder. This is the principle of **defense in depth** — never rely on just one security measure.
>
> Think of it like a medieval castle. The moat stops some attackers. The outer wall stops more. The inner wall catches anyone who got past the outer one. The tower has archers watching for anyone who got past both walls. Each layer catches what the previous one missed.

### What We Built

In this part, we added three major pieces to FicHub:

1. **Redis integration** — A fast, shared data store for rate limit counters, powered by the `redis` crate with `MultiplexedConnection` for concurrent async access. Redis gives us atomic operations, sub-millisecond response times, and shared state across multiple server instances.

2. **Token bucket rate limiter** — An algorithm that gives users a bucket of tokens that refill over time. Two levels (global and per-IP) protect both the server as a whole and individual users from overusing the system. Lua scripts ensure atomicity — no race conditions, no lost tokens.

3. **Datacenter IP blocking** — A blocklist of known bot/cloud provider IPs that gets checked before rate limiting. Datacenter IPs are rejected immediately, saving the rate limiter work and blocking the most obvious abuse. The blocklist is loaded from files at startup and stored in a `RwLock<HashSet>` for concurrent-safe access.

Together with the `RateLimiter` trait, these pieces are modular and testable. You could swap in a different algorithm, a different storage backend, or different blocking rules — the handlers don't care. They just call `check_ip` and follow the result. The trait-based design means adding new rate limiting strategies is as simple as implementing three methods.

The export handler is now protected. Bots get blocked, abusive users get slowed down, and legitimate readers get smooth, fast downloads. The bouncer is at the door, the club stays fun, and everyone can enjoy their fanfiction in peace.

But there's still more to build. In the next part, we'll tackle the recommendation engine — a system that reads your favorites and suggests stories you'll love. It's the "if you liked this, try that" feature, and it's one of FicHub's most beloved tools. Ready? Let's go.

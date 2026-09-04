# Part 6: Rate Limiting — Protecting FicHub from Abuse

*The export pipeline is where all the pieces come together. Scrapers fetch the data, the database remembers it, the cache avoids redundant work, and the export functions turn it want. But what happens when someone decides to send a thousand requests per second to FicHub? What if a bot starts hammering the same URL over and over? That's where rate limiting comes in — FicHub's immune system that protects it from being overwhelmed.*

*In this part, we'll learn about Redis (the in-memory database that stores rate limit state), the token bucket algorithm (the mathematical model behind rate limiting), and how FicHub uses Lua scripts running inside Redis to make atomic rate-limiting decisions. We'll also explore why FicHub blocks datacenter IPs and how all these pieces integrate with the request handlers.*

---

## Chapter 38: Redis Basics

### What Is Redis?

If PostgreSQL is FicHub's long-term memory — the place where it stores fanfics, export logs, and tags — then Redis is its short-term working memory. Redis stores data in RAM, which makes it incredibly fast. While a PostgreSQL query might take 5-20 milliseconds, a Redis command typically completes in under 1 millisecond.

Think of it like the difference between looking something up in a library catalog (PostgreSQL, reliable but takes a moment) versus remembering something you just heard (Redis, instant but only works while the data is still "hot" in your mind).

Redis stands for **RE**mote **DI**ctionary **S**erver. Despite the name, it's much more than a dictionary. It's a versatile in-memory data store that supports strings, lists, sets, sorted sets, hashes, and more. FicHub uses Redis for several things:

- **Rate limiting state** — tracking how many tokens each IP has used
- **Caching hot data** — quick lookups for frequently requested fics
- **Semaphore coordination** — preventing duplicate work across requests
- **Session storage** — temporary state for operations that span multiple requests

Redis was created by Salvatore Sanfilippo in 2009 and has become one of the most popular data stores in the world. It's used by Twitter, GitHub, Stack Overflow, and thousands of other high-traffic sites. The reason is simple: it's fast, reliable, and easy to use.

💡 **Key Concept:** Redis keeps all data in memory by default. This means it's fast, but if the server crashes and you haven't configured persistence, data can be lost. For rate limiting, this is actually fine — a lost rate limit just means a few extra requests slip through. For critical data (like fic content), PostgreSQL is the source of truth. Redis is a fast cache, not a durable store.

### Installing Redis

On most Linux distributions, Redis is available through the package manager:

```bash
# Arch Linux / Manjaro
sudo pacman -S redis

# Ubuntu / Debian
sudo apt install redis-server

# macOS
brew install redis
```

Starting Redis is simple:

```bash
# Start the Redis server (foreground)
redis-server

# Or as a systemd service (background, auto-start on boot)
sudo systemctl start redis
sudo systemctl enable redis  # Start on boot
```

You can verify Redis is running by connecting to it with the `redis-cli` tool:

```bash
$ redis-cli ping
PONG
```

If you see `PONG`, Redis is ready to go. The `redis-cli` tool is incredibly useful for debugging — you can use it to inspect rate limit keys, check connection status, and experiment with commands.

### The Basic Commands: SET, GET, DEL

Redis's fundamental operations are like a key-value store. You set values, get them back, and delete them when you're done. These are the building blocks that everything else is built on.

**SET** stores a value under a key:

```bash
$ SET user:123:name "Alice"
OK
```

The `OK` confirms the value was stored. You can also set expiration times:

```bash
$ SET session:abc123 "user_data" EX 3600  # Expires in 3600 seconds (1 hour)
```

**GET** retrieves a value:

```bash
$ GET user:123:name
"Alice"
```

If the key doesn't exist, you get `(nil)`:

```bash
$ GET nonexistent
(nil)
```

**DEL** removes a key:

```bash
$ DEL user:123:name
(integer) 1
```

The `1` means one key was deleted. If the key didn't exist, it returns `0`.

**EXISTS** checks if a key exists:

```bash
$ EXISTS user:123:name
(integer) 1  # 1 = exists, 0 = doesn't exist
```

**INCR** and **DECR** atomically increment or decrement numeric values:

```bash
$ SET counter 0
$ INCR counter
(integer) 1
$ INCR counter
(integer) 2
$ INCRBY counter 5
(integer) 7
$ DECR counter
(integer) 6
```

These are atomic operations — even if two clients try to `INCR` the same key simultaneously, the count is always correct. This is one of Redis's superpowers.

Redis keys follow a naming convention using colons as separators. You'll see this pattern throughout FicHub:

```bash
rate:global           # Global rate limit bucket
rate:ip:192.168.1.1   # Per-IP rate limit bucket
cache:epub:abc123     # Cached EPUB data
```

The colon-separated format isn't special to Redis — it's just a convention that makes keys readable. But `redis-cli` can use it for namespace browsing with the `KEYS` command:

```bash
$ KEYS rate:*
1) "rate:global"
2) "rate:ip:192.168.1.1"
3) "rate:ip:10.0.0.5"
```

⚠️ **Watch Out:** The `KEYS` command scans all keys in the database. In a production Redis instance with millions of keys, this can block all other operations for several seconds. Use `SCAN` instead for production monitoring, or check specific keys directly. The `SCAN` command iterates through keys incrementally without blocking:

```bash
# Bad: blocks everything
$ KEYS rate:*

# Good: scans incrementally
$ SCAN 0 MATCH rate:* COUNT 100
```

### Redis Expiration

One of Redis's most useful features is automatic key expiration. You can set a time-to-live (TTL) on any key:

```bash
$ SET session:abc123 "user_data" EX 3600  # Expires in 3600 seconds
$ TTL session:abc123
(integer) 3597
$ EXISTS session:abc123
(integer) 1
# Wait 3600 seconds...
$ EXISTS session:abc123
(integer) 0  # Key has been automatically deleted
```

This is incredibly useful for rate limiting. Instead of manually cleaning up old rate limit keys, you can set them to expire after a reasonable period (e.g., 24 hours). Redis handles the cleanup automatically.

FicHub doesn't currently use expiration on rate limit keys, which means the keys accumulate over time. In a production deployment, you might want to add expiration to prevent unbounded memory growth:

```bash
# Set a rate limit key with 24-hour expiration
$ HMSET rate:ip:192.168.1.1 value 30 last_drain 1690000000
$ EXPIRE rate:ip:10.0.0.1 86400  # 24 hours
```

The `EXPIRE` command sets a TTL on an existing key. After 86400 seconds (24 hours), Redis automatically deletes the key. The next request from that IP will create a fresh bucket with full tokens.

### Redis Data Types

Redis supports more than just strings. Understanding these data types is important because FicHub uses several of them, and each type has its own set of efficient operations.

**Strings** — The simplest type. A key maps to a string value. Numbers are stored as strings too, but Redis has special commands for numeric operations:

```bash
$ SET counter 0
$ INCR counter
(integer) 1
$ INCRBY counter 5
(integer) 6
$ GET counter
"6"
```

Redis strings can hold up to 512 MB of data. That's enough to store an entire chapter of a fanfiction if you wanted to (though you probably shouldn't — that's what PostgreSQL is for).

**Hashes** — A key maps to a dictionary of field-value pairs. This is what FicHub's token bucket uses. Hashes are perfect for storing related data under one key:

```bash
$ HMSET rate:ip:192.168.1.1 value 28.5 last_drain 1690000000.123
OK
$ HMGET rate:ip:192.168.1.1 value last_drain
1) "28.5"
2) "1690000000.123"
```

A hash stores multiple related values under one key. This is more efficient than using separate keys for each field, because you can read all fields in one operation. Redis hashes can hold up to 4 billion field-value pairs, and each field can be up to 512 MB.

You can also increment individual fields:

```bash
$ HINCRBY rate:ip:192.168.1.1 value -1
(integer) 27
```

**Lists** — Ordered collections. Items can be pushed and popped from both ends:

```bash
$ RPUSH queue "task1" "task2"
(integer) 2
$ LPOP queue
"task1"
$ LRANGE queue 0 -1  # Get all items
1) "task2"
```

Lists are useful for job queues, message passing, and activity feeds. Redis lists can hold up to 4 billion elements.

**Sets** — Unordered collections of unique values:

```bash
$ SADD tags "romance" "adventure" "romance"
(integer) 2    # "romance" only counted once
$ SISMEMBER tags "romance"
(integer) 1    # 1 = exists, 0 = doesn't exist
$ SMEMBERS tags
1) "romance"
2) "adventure"
```

Sets are great for tracking unique visitors, tags, or categories. Redis sets support set operations like intersection, union, and difference.

**Sorted Sets** — Like sets, but each member has a score for ordering:

```bash
$ ZADD leaderboard 100 "player1" 200 "player2"
(integer) 2
$ ZREVRANGE leaderboard 0 -1 WITHSCORES
1) "player2"
2) "200"
3) "player1"
4) "100"
$ ZINCRBY leaderboard 50 "player1"
"150"
```

Sorted sets are incredibly versatile. FicHub could use them for ranking fics by popularity, implementing leaderboards, or scheduling delayed tasks.

💡 **Key Concept:** FicHub's token bucket uses **Hashes** to store two values together: the current number of tokens and the last time the bucket was drained. This ensures both values are always consistent — you can't read a stale token count with an up-to-date timestamp, because they're stored and updated together.

### Connecting with the redis Crate

In Rust, FicHub connects to Redis using the `redis` crate. Here's the setup from `Cargo.toml`:

```toml
redis = { version = "1.4", features = ["aio", "tokio-comp"] }
```

The `aio` feature enables async support, and `tokio-comp` integrates with Tokio's async runtime. These are essential for an axum web server, which needs to handle many concurrent connections without blocking.

Creating a Redis client and connection:

```rust
// From src/server.rs
let redis_client = redis::Client::open(config.redis_url.as_str())
    .expect("Invalid Redis URL");
let redis_conn = redis_client.get_multiplexed_async_connection()
    .await
    .expect("Failed to connect to Redis");
```

The `Client::open()` call creates a client from a URL. The default Redis URL is `redis://localhost:6379`, but in production it might be something like `redis://:password@redis-host:6379/0`.

The URL format is: `redis://[user:password@]host[:port][/db-number]`

- `user:password` — Optional authentication credentials
- `host` — The Redis server hostname or IP
- `port` — The Redis port (default 6379)
- `/db-number` — The Redis database number (0-15, default 0)

The `get_multiplexed_async_connection()` method is the key call here. It creates a **multiplexed** connection, which is different from a regular connection.

### MultiplexedConnection

When you use a regular Redis connection, only one command can be in flight at a time. You send a command, wait for the response, then send the next one. This is called a "pipeline" of sequential commands.

A `MultiplexedConnection` is smarter. It sends multiple commands over a single TCP connection without waiting for each response. When the responses come back, Redis attaches a unique ID to each one so the client can match them to the original commands. This is called **multiplexing**.

Think of it like a multi-lane highway versus a single-lane road. With a regular connection, cars (commands) have to wait in line. With multiplexing, multiple cars can travel simultaneously.

```rust
// Cloning the connection is cheap — it's an Arc internally
let mut conn = self.redis.clone();

// Send a command
let result: f64 = redis::cmd("EVALSHA")
    .arg(&self.lua_sha[..])
    .arg(1)
    .arg(key)
    .arg(1.0)
    .query_async(&mut conn)
    .await?;
```

Notice the `self.redis.clone()` at the top. Cloning a `MultiplexedConnection` doesn't create a new TCP connection — it creates a new handle to the same underlying connection. This is safe to do from multiple async tasks simultaneously, which is essential for a web server handling concurrent requests.

The `redis::cmd("EVALSHA")` creates a command builder. You chain `.arg()` calls to add arguments, then `.query_async()` to execute and parse the response. The generic type (`f64` in this case) tells Redis how to deserialize the result.

⚠️ **Watch Out:** Never use `get_async_connection()` instead of `get_multiplexed_async_connection()`. The non-multiplexed version creates a separate TCP connection per call, which wastes resources and can lead to "too many connections" errors on the Redis server. The multiplexed version shares a single connection across all callers. In a busy web server, this difference is enormous — hundreds of concurrent tasks sharing one connection vs. hundreds of separate connections.

### Testing Your Redis Connection

Let's try some Redis commands interactively using `redis-cli`:

```bash
# Start a Redis server
$ redis-server &

# Connect to it
$ redis-cli

# Store some data
127.0.0.1:6379> SET mykey "hello"
OK
127.0.0.1:6379> GET mykey
"hello"

# Try a hash (what FicHub's token bucket uses)
127.0.0.1:6379> HMSET rate:ip:10.0.0.1 value 30 last_drain 1690000000.0
OK
127.0.0.1:6379> HMGET rate:ip:10.0.0.1 value last_drain
1) "30"
2) "1690000000.0"

# Update a field
127.0.0.1:6379> HMSET rate:ip:10.0.0.1 value 29.0 last_drain 1690000001.0
OK

# Check how much memory Redis is using
127.0.0.1:6379> INFO memory
# Memory section shows used_memory_human: 500K (approximately)

# Clean up
127.0.0.1:6379> DEL rate:ip:10.0.0.1
(integer) 1
```

The `INFO memory` command is useful for monitoring how much RAM Redis is consuming. For rate limiting, Redis typically uses very little memory — each rate limit key is just a hash with two fields, and keys expire automatically.

You can also use `redis-cli` to monitor commands in real-time:

```bash
$ redis-cli MONITOR
1690000000.123456 [0 127.0.0.1:12345] "HMGET" "rate:ip:10.0.0.1" "value" "last_drain"
1690000000.123789 [0 127.0.0.1:12345] "HMSET" "rate:ip:10.0.0.1" "value" "29.0" "last_drain" "1690000001.0"
```

This is invaluable for debugging — you can see exactly what commands FicHub is sending to Redis.

### Redis Persistence

By default, Redis stores everything in memory. But what happens when Redis restarts? The data is gone. For rate limiting, this is usually fine — a lost rate limit means a few extra requests slip through. But for other uses (like caching), you might want data to survive restarts.

Redis offers two persistence options:

1. **RDB snapshots** — Periodically saves the entire dataset to disk. Fast to load on restart, but you might lose data from the last snapshot.
2. **AOF (Append-Only File)** — Logs every write operation to disk. More durable, but the file grows over time and restarts are slower.

You can configure both in `redis.conf`:

```bash
# Save every 60 seconds if at least 1000 keys changed
save 60 1000

# Enable AOF
appendonly yes
```

For FicHub's rate limiting use case, the default configuration (RDB snapshots every few minutes) is sufficient. If Redis crashes, the rate limits from the last few minutes are lost, and users get a brief window of unrestricted access. This is acceptable because:
- The window is short (at most a few minutes)
- Rate limiting is defense-in-depth, not the only protection
- Real users are unlikely to notice, and bots only get a brief reprieve

### Redis Pub/Sub

Redis also supports publish/subscribe messaging, which can be useful for coordinating between multiple FicHub instances. While FicHub doesn't use this for rate limiting (it uses the shared Redis store instead), it's worth knowing about:

```bash
# Terminal 1: Subscribe to a channel
$ redis-cli SUBSCRIBE rate-limit-alerts

# Terminal 2: Publish a message
$ redis-cli PUBLISH rate-limit-alerts "IP 192.168.1.1 exceeded global limit"
```

In a multi-instance FicHub deployment, you could use Pub/Sub to broadcast rate limit alerts between instances. But for the token bucket algorithm, the shared Redis store is simpler and more reliable — every instance reads and writes to the same keys.

### Redis Transactions

Redis supports transactions through `MULTI`/`EXEC`, but they work differently from database transactions. A Redis transaction guarantees that all commands in the batch are executed sequentially, but it doesn't provide rollback capabilities:

```bash
$ MULTI
OK
$ SET key1 "value1"
QUEUED
$ SET key2 "value2"
QUEUED
$ EXEC
1) OK
2) OK
```

FicHub doesn't use transactions for rate limiting — the Lua script provides stronger atomicity guarantees. But transactions are useful for other Redis operations where you need to batch commands together.

🧪 **Try It Yourself:** Install Redis locally and experiment with hashes. Create a hash called `rate:global` with fields `value` (set to 100.0) and `last_drain` (set to the current Unix timestamp). Then read both fields back. Try using `HINCRBY` to decrement the `value` field by 1, then decrement it by 1 five more times. What happens when you try to decrement below zero? (Hint: it works fine — Redis doesn't know it's supposed to be a token count.) Try using `MULTI`/`EXEC` to batch a `GET` and `SET` together. Does the `GET` return the old or new value?

---

## Chapter 39: The Token Bucket Algorithm

### What Is Rate Limiting?

Rate limiting is the practice of controlling how many requests a client can make to a server within a given time period. It's like a bouncer at a club — the bouncer lets people in at a reasonable pace, but if too many people rush the door at once, they get turned away (or told to wait).

For FicHub, rate limiting serves two purposes:

1. **Protection** — Preventing bots or misbehaving clients from overwhelming the server
2. **Fairness** — Ensuring one user can't hog all the server's resources

Without rate limiting, a single bot could send 10,000 requests per second, consuming all of FicHub's CPU, memory, and bandwidth. The server would become slow or unresponsive for everyone else. Rate limiting ensures that each user gets a fair share of the server's capacity.

There are several common rate limiting algorithms:

- **Fixed window** — Count requests per minute, block when over threshold. Simple but allows bursts at window boundaries.
- **Sliding window** — Like fixed window but with a rolling time period. Smoother but more complex to implement.
- **Leaky bucket** — Requests queue up and are processed at a fixed rate. Smooth but doesn't allow bursts.
- **Token bucket** — Tokens accumulate at a fixed rate, requests consume tokens. Allows bursts while limiting sustained rate.

FicHub uses the **token bucket algorithm** because it's the best balance of simplicity, burst-friendliness, and fairness.

### The Token Bucket Algorithm

Imagine a bucket that holds tokens. Tokens drip into the bucket at a constant rate (say, 1 token per second). The bucket has a maximum capacity — it can't hold more than, say, 30 tokens.

When a request comes in, it needs to grab 1 token from the bucket. If there are tokens available, the request is allowed. If the bucket is empty, the request is rejected (or told to wait).

This is elegant because:

- **Bursts are allowed** — If no one has made requests for a while, the bucket fills up. A user can burst through many requests in quick succession by draining the accumulated tokens.
- **Sustained rate is limited** — The refill rate determines the sustainable throughput. If tokens drip at 1/second, a user can average at most 1 request per second over time.
- **The cap prevents abuse** — Even if a user waits a long time, they can't accumulate unlimited tokens. The bucket's capacity is the burst limit.

### Visual Walkthrough

Let's trace through what happens with a bucket of capacity 30 and a flow rate of 0.116 tokens/second (which is FicHub's per-IP configuration — approximately 1 request every 8.6 seconds).

```
Time 0: Bucket starts full
┌─────────────────┐
│ ■■■■■■■■■■■■■■■ │  30 tokens (full)
│ ■■■■■■■■■■■■■■■ │
│ ■■■■■■■■■■■■■■■ │
└─────────────────┘

User makes 10 requests quickly (time 0 to 10):
┌─────────────────┐
│                 │  20 tokens remaining
│ ■■■■■■■■■■■■■■ │  (30 - 10 = 20)
│ ■■■■■■■■■■■■■■ │
└─────────────────┘

Time 100 (about 1.6 minutes later):
Refill: 100 × 0.116 = 11.6 tokens added
New total: min(30, 20 + 11.6) = 30 (capped at capacity)
┌─────────────────┐
│ ■■■■■■■■■■■■■■■ │  30 tokens (refilled to capacity)
│ ■■■■■■■■■■■■■■■ │
│ ■■■■■■■■■■■■■■■ │
└─────────────────┘

User tries 31 requests at time 100:
First 30: Allowed (bucket has 30 tokens)
31st: Rejected — bucket is empty
  Wait time = (31 - 0) / 0.116 = 267 seconds
  ≈ 4.5 minutes
```

The `math.min(capacity, value + elapsed * flow)` calculation is the heart of the algorithm. It calculates how many tokens would be available after accounting for the time that's passed, but never exceeds the bucket's capacity.

Let's trace a few more scenarios:

**Scenario: Rapid burst followed by pause**

```
t=0:   value=30, request 5 → value=25, allowed
t=1:   value=25.116, request 5 → value=20.116, allowed
t=2:   value=20.232, request 5 → value=15.232, allowed
... (user stops for 100 seconds)
t=102: value = min(30, 15.232 + 100 × 0.116) = min(30, 26.832) = 26.832
        request 5 → value=21.832, allowed
```

**Scenario: Exhausted bucket, user retries too soon**

```
t=0:   value=0 (bucket drained), request 1
        new_tokens = 0 + 0 × 0.116 = 0
        allowed = 0 - 1 = -1 (denied)
        wait = (1 - 0) / 0.116 = 8.62 seconds

t=5:   still waiting
        new_tokens = 0 + 5 × 0.116 = 0.58
        allowed = 0.58 - 1 = -0.42 (still denied)
        wait = (1 - 0.58) / 0.116 = 3.62 seconds

t=8.62: enough tokens accumulated
        new_tokens = 0 + 8.62 × 0.116 = 1.0
        allowed = 1.0 - 1 = 0 (exactly enough, allowed!)
```

### Lua Scripts for Atomicity

Here's the tricky part: the token bucket state lives in Redis, and multiple FicHub workers might be checking the same bucket simultaneously. If two workers both read "28 tokens available" at the same time, they might both try to consume tokens, resulting in a race condition where 2 tokens are consumed but the bucket only had 1 to spare.

The solution is to run the entire token bucket calculation as a **single atomic operation** inside Redis. Redis is single-threaded — it processes one command at a time. If we could put the entire check-and-update logic into one command, race conditions would be impossible.

That's exactly what **Lua scripts** do. Redis supports embedded Lua scripting. When you send a Lua script to Redis, it executes atomically — no other commands can run while the script is executing. This is guaranteed by Redis's single-threaded architecture.

Here's FicHub's token bucket Lua script:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

We'll dissect this script line by line in Chapter 43. For now, understand that this script:

1. Reads the current bucket state (tokens + last drain time)
2. Calculates how many new tokens have arrived since the last check
3. Tries to consume the requested number of tokens
4. Returns either -1 (allowed) or a wait time in seconds

Because this entire script runs atomically in Redis, two concurrent requests can't interfere with each other. The second request will always see the updated state from the first.

💡 **Key Concept:** The Lua script uses `redis.call('TIME')` to get the current time from Redis, not from the FicHub server. This prevents time-skew issues where different servers have different system clocks. Since Redis is the single source of truth for rate limit state, it should also be the source of truth for time.

### Two Levels: Global and Per-IP

FicHub doesn't just use one rate limit — it uses two:

1. **Global bucket** — Controls the total request rate across ALL users. This prevents FicHub's scrapers from overwhelming source websites (like AO3 or FanFiction.net).
2. **Per-IP bucket** — Controls how fast each individual IP address can make requests. This prevents any single user from hogging the global bucket.

The global bucket is like a parking garage with a limited number of spaces. The per-IP bucket is like each driver having their own personal parking permit that limits how many spaces they can use at once.

FicHub's default configuration:

```rust
global_capacity: 150.0,   // Max 150 tokens in the global bucket
global_flow: 30.0,        // 30 tokens per second refill rate
ip_capacity: 30.0,        // Max 30 tokens per IP
ip_flow: 0.116,           // ~1 token every 8.6 seconds
```

The global bucket allows 30 requests per second on average (with bursts up to 150). The per-IP bucket allows about 1 request every 8.6 seconds, with bursts up to 30.

Let's break down what these numbers mean in practice:

**Global capacity of 150:** If no requests have been made for a while, the global bucket fills up to 150 tokens. This means a burst of 150 simultaneous requests would all be allowed — useful when FicHub first starts up after a maintenance window and many users come back at once.

**Global flow of 30:** After a burst drains the bucket, it refills at 30 tokens per second. This means FicHub can sustain 30 requests per second indefinitely. If upstream sites (AO3, FFN) can handle 30 requests per second from FicHub, this is safe.

**IP capacity of 30:** Each user can burst up to 30 requests without waiting. This handles the case where someone opens 15 fics in new tabs — each tab loads metadata (1 request) and the EPUB (1 request), totaling 30 requests.

**IP flow of 0.116:** This means each user gets about 1 token every 8.6 seconds. In practice, a real user would never make requests this fast — they'd download a fic, read it for hours, then come back. The 8.6-second limit only kicks in for bots or extremely enthusiastic readers.

Each request must pass BOTH checks:

```
Request arrives for IP 192.168.1.1
  │
  ├── Check global bucket (rate:global)
  │     Tokens available? → No → Wait 5 seconds
  │     Tokens available? → Yes → Continue
  │
  ├── Check per-IP bucket (rate:ip:192.168.1.1)
  │     Tokens available? → No → Wait 12 seconds
  │     Tokens available? → Yes → Allow request
  │
  └── Request proceeds
```

If either check fails, the request is delayed. The user sees the longer of the two wait times.

The two-level approach has an important property: even if a single IP isn't hitting its personal limit, the global limit can still slow it down. This protects the upstream websites. Conversely, even if the global bucket has plenty of tokens, a single IP that's hammering the server will be slowed down by its per-IP limit. This protects other users.

```rust
// From RedisBucketLimiter::check_ip()
let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
    .await.unwrap_or(-1.0);

if global_wait > 0.0 {
    return RateLimitResult::Wait(global_wait.ceil() as u64);
}

// Check per-IP bucket
let ip_key = format!("rate:ip:{}", ip);
let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
    .await.unwrap_or(-1.0);
```

Notice the global check happens first. This is intentional — if the system is under global pressure, we want to reject fast without even checking the per-IP bucket.

🧪 **Try It Yourself:** Open a Redis CLI and create two hashes: `rate:global` (with value=150, last_drain=current time) and `rate:ip:127.0.0.1` (with value=30, last_drain=current time). Simulate 50 rapid requests by running a loop that decrements the `value` field by 1 for each request. Watch how quickly the per-IP bucket empties compared to the global one. Try writing a small Lua script that does the token bucket calculation — you can use `EVAL` to test it directly in `redis-cli`.

---

## Chapter 40: The Redis Rate Limiter

### The RedisBucketLimiter Struct

Now that we understand Redis and the token bucket algorithm, let's look at how FicHub combines them into a working rate limiter. The main structure lives in `src/limiter/redis_bucket.rs`:

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,        // SHA of loaded Lua script
    dynamic_rate_limit: bool,
    static_delay_base: f64, // seconds

    // Datacenter IP cache
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,

    // Configuration
    global_capacity: f64,
    global_flow: f64,
    ip_capacity: f64,
    ip_flow: f64,
}
```

Let's examine each field:

- **`redis`** — The multiplexed Redis connection, shared across all concurrent requests. This is the same connection created in `server.rs` and passed in during construction.
- **`lua_sha`** — The SHA hash of the Lua script, loaded once at startup and reused for every check. This avoids sending the full 600-byte script on every request.
- **`dynamic_rate_limit`** — Whether to use the sophisticated token bucket or a simple static delay. When `false`, FicHub uses a basic random sleep instead of Redis.
- **`static_delay_base`** — The base delay in seconds when dynamic rate limiting is disabled. Set to 0.1 seconds.
- **`datacenter_ips`** — A set of known datacenter IP addresses, wrapped in `Arc<RwLock<...>>` for thread-safe access.
- **`global_capacity`** and **`global_flow`** — Parameters for the global token bucket. Capacity is 150 tokens, flow is 30 tokens/second.
- **`ip_capacity`** and **`ip_flow`** — Parameters for the per-IP token bucket. Capacity is 30 tokens, flow is 0.116 tokens/second.

The `Arc<RwLock<HashSet<IpAddr>>>` pattern deserves explanation. `Arc` is the atomically reference-counted pointer that allows shared ownership across threads. `RwLock` is a reader-writer lock — multiple readers can access the data simultaneously, but writers get exclusive access. `HashSet<IpAddr>` is the actual set of datacenter IPs.

This pattern is used because the datacenter IP list is loaded once at startup (a write operation) and then read on every request (many read operations). RwLock is optimal here because reads vastly outnumber writes.

### Loading the Lua Script

When `RedisBucketLimiter::new()` is called, it loads the Lua script into Redis:

```rust
pub async fn new(
    redis_conn: redis::aio::MultiplexedConnection,
    dynamic_rate_limit: bool,
) -> Result<Self, redis::RedisError> {
    // Load the token bucket Lua script
    let lua_script = r#"
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
"#;

    let mut conn = redis_conn.clone();
    let lua_sha: String = redis::cmd("SCRIPT")
        .arg("LOAD")
        .arg(lua_script)
        .query_async(&mut conn)
        .await?;
```

The `SCRIPT LOAD` command sends the Lua script to Redis and gets back a SHA hash (a 40-character hexadecimal string). Later, instead of sending the full script text, FicHub sends just the SHA — this saves bandwidth on every request.

The `tonumber()` calls are necessary because Redis passes all arguments as strings, even numbers. Without `tonumber()`, Lua would treat them as strings and do string concatenation instead of math. For example, `20 + "30"` in Lua would fail, but `20 + tonumber("30")` gives you `50`.

The `r#"..."#` syntax is a Rust raw string literal. The `#` characters delimit the string, allowing the Lua code to contain both single and double quotes without escaping. This makes the embedded Lua much more readable.

### The check_bucket Method

The private `check_bucket` method executes the Lua script against a specific Redis key:

```rust
/// Check a token bucket and return wait time in seconds
async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) 
    -> Result<f64, redis::RedisError> 
{
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.0)        // requested tokens
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(result)
}
```

The `EVALSHA` command executes a previously loaded Lua script. The arguments are:

1. **`self.lua_sha`** — The SHA hash of the script to execute
2. **`1`** — Number of keys (Redis needs this to know how many KEYS arguments follow)
3. **`key`** — The Redis key to operate on (e.g., `"rate:global"` or `"rate:ip:192.168.1.1"`)
4. **`1.0`** — Number of tokens requested (1 token per request)
5. **`capacity`** — The bucket's maximum capacity
6. **`flow`** — The token refill rate (tokens per second)

The return value is either `-1` (request allowed) or a positive number indicating how many seconds to wait.

The `self.redis.clone()` at the top is essential. Each async task needs its own handle to the Redis connection. Cloning is cheap (it's just incrementing an Arc reference count), and each clone can independently send commands.

### The penalize Method

When a request fails (e.g., a scraper encounters an error), FicHub penalizes the IP by consuming extra tokens:

```rust
/// Penalize by requesting extra tokens (on failure)
async fn penalize(&self, key: &str, capacity: f64, flow: f64) 
    -> Result<(), redis::RedisError> 
{
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)        // penalize with 1.5 tokens
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

Notice the `1.5` instead of `1.0` — failed requests cost 1.5 tokens. This is a deliberate design choice. If a scraper keeps failing (maybe the source site is blocking FicHub), it should be slowed down faster than a normal request. The penalty discourages repeated failures.

The return value is ignored (stored in `_: f64`) because we don't need to know whether the penalization was accepted or not — it's best-effort. If Redis is down, the penalty is simply skipped.

The penalty uses the same Lua script as a normal token check. This means a penalty can also deny requests — if the bucket is already empty, consuming 1.5 extra tokens will result in a positive wait time, which gets returned but is ignored. The actual effect is that the next request will find fewer tokens available.

Why 1.5 tokens specifically? It's a balance:
- **1.0 tokens** — Same cost as a normal request. This means a failed request doesn't hurt any more than a successful one. A bot could hammer the server with failing requests at the same rate as normal requests.
- **1.5 tokens** — 50% more cost. After 20 failures, the IP has consumed 30 extra tokens — equivalent to a full per-IP bucket. This effectively locks out the IP for about 4 minutes.
- **2.0+ tokens** — Too aggressive. A single transient error (like a brief network hiccup) would severely penalize a legitimate user. Better to be conservative.

The 1.5 value was chosen through trial and error. It's aggressive enough to slow down repeated failures but gentle enough to avoid punishing transient errors.

### The check_ip Method

The `check_ip` method is the main entry point, implementing the `RateLimiter` trait:

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    // Step 1: Static delay mode
    if !self.dynamic_rate_limit {
        let delay = self.static_delay_base 
            + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(
            tokio::time::Duration::from_secs_f64(delay)
        ).await;
        return RateLimitResult::Allowed;
    }

    // Step 2: Datacenter IP check
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    // Step 3: Global bucket check
    let global_wait = self.check_bucket(
        "rate:global", 
        self.global_capacity, 
        self.global_flow
    ).await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Step 4: Per-IP bucket check
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(
        &ip_key, 
        self.ip_capacity, 
        self.ip_flow
    ).await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    // Step 5: All checks passed
    RateLimitResult::Allowed
}
```

The method follows a clear decision tree:

1. **Static mode** — If `dynamic_rate_limit` is false, just sleep for a random delay. This is the simple fallback that doesn't need Redis.
2. **Datacenter check** — If the IP is in the datacenter blocklist, return `Blocked` immediately. No token bucket check needed — we know this IP shouldn't be making requests at all.
3. **Global bucket** — Check the global rate limit. If we need to wait, return `Wait` with the wait time.
4. **Per-IP bucket** — Check the IP-specific rate limit. If we need to wait, return `Wait`.
5. **Allow** — If both checks pass, return `Allowed`.

The `.unwrap_or(-1.0)` handles Redis errors gracefully. If Redis is unreachable, the limiter defaults to allowing the request (wait time of -1.0 means "don't wait"). This is the fail-open strategy discussed below.

The `.ceil()` method rounds up the wait time to the nearest whole second. You don't want to tell a user "wait 0.3 seconds" — that's essentially no wait at all. Rounding up ensures users get meaningful feedback in the `Retry-After` header.

⚠️ **Watch Out:** The `rand::random::<f64>()` call in the static delay mode returns a value between 0.0 and 1.0. Combined with `static_delay_base` of 0.1, the total delay is between 0.1 and 0.2 seconds. This randomization is crucial — without it, all clients would retry at exactly the same time, causing synchronized bursts (the "thundering herd" problem).

### The RateLimitResult Enum

The return type is defined in `src/limiter/mod.rs`:

```rust
#[derive(Debug)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Wait this many seconds before retrying
    Wait(u64),
    /// Blocked (datacenter IP, etc.)
    Blocked,
}
```

Three possible outcomes:

- **`Allowed`** — The request can proceed immediately. The handler continues with its normal logic.
- **`Wait(u64)`** — The request should be retried after this many seconds. Handlers typically return a `429 Too Many Requests` response with a `Retry-After` header set to this value.
- **`Blocked`** — The request should be rejected entirely, typically with a `403 Forbidden` response. No retry is expected.

The `#[derive(Debug)]` allows us to print the result for debugging. The `Wait` variant carries the wait time as a `u64`, which is a 64-bit unsigned integer — more than enough for any practical wait time (up to 18 quintillion seconds).

The enum is deliberately simple. It gives the caller all the information needed to make a decision without being overly specific. The handler decides what HTTP status code to return, not the rate limiter. This separation of concerns makes the rate limiter reusable across different contexts.

The `RateLimitResult` enum also makes testing easier. You can pattern-match on the result to verify that rate limiting is working correctly:

```rust
let result = limiter.check_ip("192.168.1.1".parse().unwrap()).await;
match result {
    RateLimitResult::Allowed => println!("Request allowed"),
    RateLimitResult::Wait(secs) => println!("Wait {} seconds", secs),
    RateLimitResult::Blocked => println!("IP is blocked"),
}
```

This is much cleaner than returning a raw integer or boolean. The enum makes the code self-documenting — you can see all possible outcomes without reading the implementation.

### The Fail-Open Strategy

One of the most important design decisions in FicHub's rate limiter is the **fail-open** strategy. Look at this line again:

```rust
let global_wait = self.check_bucket(
    "rate:global", 
    self.global_capacity, 
    self.global_flow
).await.unwrap_or(-1.0);
```

If the Redis call fails (connection timeout, Redis server down, network error), `unwrap_or(-1.0)` returns -1.0. A wait time of -1.0 means "don't wait" — the request is allowed through.

This is the **fail-open** approach: when the rate limiter itself fails, requests are still allowed. The alternative is **fail-closed**, where a rate limiter failure blocks all requests.

Why fail-open?

1. **Availability over protection** — If Redis is down, you'd rather have FicHub serve requests without rate limiting than refuse all requests entirely. A few extra requests without rate limiting is less harmful than a complete outage.
2. **External dependency** — Redis is an external dependency. Rate limiting should enhance FicHub's reliability, not become a single point of failure.
3. **Self-healing** — When Redis comes back online, rate limiting resumes automatically. No manual intervention needed.
4. **Graceful degradation** — The server continues to function, just with reduced protection. This is much better than the server going down entirely.

The fail-open pattern appears throughout FicHub's codebase. The `report_failure` method also uses `let _ = ...` to ignore errors:

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}
```

If Redis is unreachable during a failure report, we just skip the penalty. The IP will still be rate-limited by the normal token bucket logic when it recovers — the penalty just makes it more aggressive.

⚠️ **Watch Out:** The fail-open strategy means that if Redis is down, FicHub is vulnerable to abuse. In production, you'd monitor Redis health and alert on failures. A brief outage (a few seconds) is acceptable; a prolonged one (minutes or hours) needs attention. Consider setting up Redis Sentinel or Redis Cluster for high availability.

### The report_failure Method

When a scraper request fails, the rate limiter penalizes both the global and per-IP buckets:

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize(
        "rate:global", 
        self.global_capacity, 
        self.global_flow
    ).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(
        &ip_key, 
        self.ip_capacity, 
        self.ip_flow
    ).await;
}
```

Both `let _ = ...` patterns deliberately ignore errors. If Redis is unreachable during a failure report, we just skip the penalty. The IP will still be rate-limited by the normal token bucket logic — the penalty just makes it more aggressive.

Notice that the global bucket is also penalized. This prevents a failing scraper from consuming global capacity that other requests could use. If a scraper is hitting errors (maybe AO3 is returning 503), it should back off and leave room for healthy requests.

The penalty amount of 1.5 tokens means each failure costs 50% more than a normal request. After 20 failures, the IP has consumed 30 extra tokens — equivalent to a full per-IP bucket. This effectively locks out the IP for the time it takes to refill (about 4 minutes with the default flow rate).

🧪 **Try It Yourself:** Write a test that creates a `RedisBucketLimiter`, calls `check_ip()` three times in rapid succession for the same IP, and verifies the results. The first should be `Allowed`, the second might be `Allowed` (depending on timing), and eventually you should see `Wait`. Then call `report_failure()` and verify the next request has a longer wait time. Try varying the `ip_flow` rate to see how it affects the recovery time.

---

## Chapter 41: Datacenter IP Blocking

### Why Block Datacenter IPs?

Not all traffic is created equal. When someone visits FicHub from their home internet connection, the request comes from a residential IP address — the kind assigned by ISPs like Comcast, BT, or Deutsche Telekom. These are the users FicHub is built for.

But when a bot or scraper sends requests, it typically comes from a datacenter IP address — the kind used by cloud providers like AWS, Google Cloud, DigitalOcean, or OVH. Datacenter IPs are also used by VPN providers, corporate networks, and proxy services.

Why should FicHub care? Because datacenter traffic is almost never from real users browsing fanfiction:

- **Scraping bots** — Automated tools that download fic content for aggregation sites. These bots might download thousands of fics per hour, consuming FicHub's resources without benefiting real users. They often use datacenter IPs because cloud servers are fast, cheap, and can be replaced if blocked.
- **API abusers** — Programs that bypass the web interface and hit FicHub's API directly. These might be trying to build competing services, collect training data for AI models, or just experiment with the API without caring about the impact.
- **Spam bots** — Automated systems that submit spam content, fake tags, or abusive recommendations. These typically run on cheap cloud instances and use datacenter IPs.
- **DDoS attacks** — Distributed denial-of-service attacks originating from cloud instances. These are designed to take FicHub offline by overwhelming it with requests.
- **Web scrapers** — Tools that crawl FicHub's content to build mirrors or archives. While some of these serve legitimate purposes, uncontrolled scraping can overwhelm the server.

Blocking datacenter IPs doesn't affect real users (who have residential IPs) but significantly reduces abuse. It's the first line of defense, before rate limiting even comes into play.

This is an imperfect solution — some legitimate users might be on VPNs (which use datacenter IPs) or corporate networks. But for a free service like FicHub, the trade-off is worth it: blocking datacenter traffic reduces abuse by 80-90% while affecting less than 1% of real users. The 1% who are affected (VPN users) can typically disable their VPN temporarily to access FicHub.

The cost of NOT blocking datacenter IPs is significant:
- Server resources wasted on bot traffic (CPU, memory, bandwidth)
- Upstream sites (AO3, FFN) getting extra requests from FicHub's scrapers
- Database queries for storing bot activity
- Cache pollution from bot-requested fics

By blocking datacenter IPs at the door, FicHub avoids all of these costs with minimal impact on legitimate users.

💡 **Key Concept:** Datacenter IP blocking isn't about being mean to cloud users. It's about resource allocation. FicHub's resources are limited. Every request from a bot is one less request that can be served to a real fanfiction reader. Blocking known bot IPs is like closing the back door to prevent shoplifters — it doesn't affect honest customers.

### Loading IP Lists from Config

FicHub loads datacenter IP addresses from external files. The configuration specifies these files via the `IP_TAG_SOURCES` environment variable:

```bash
# Format: path,type,tag (one per line)
IP_TAG_SOURCES=/data/dc-ips-aws.txt,dc,aws
/data/dc-ips-gcloud.txt,dc,gcloud
/data/dc-ips-digitalocean.txt,dc,digitalocean
```

Each line has three fields separated by commas:

1. **path** — The file path containing IP addresses
2. **type** — The type of IP list (e.g., "dc" for datacenter)
3. **tag** — A label for the source (e.g., "aws", "gcloud")

The IP files themselves are simple text files, one IP per line:

```
# AWS IP ranges
52.0.0.0/8
54.0.0.0/8
# Comments start with #
18.0.0.0/8
```

Where do these IP lists come from? Cloud providers publish their IP ranges:

- **AWS** — https://ip-ranges.amazonaws.com/ip-ranges.json
- **Google Cloud** — https://www.gstatic.com/ipranges/cloud.json
- **DigitalOcean** — https://www.digitalocean.com/community/questions/what-are-the-ip-ranges-for-digitalocean

You can download these lists, extract the IP addresses, and save them as plain text files. Some services also provide ready-made lists of datacenter IPs, updated regularly.

The config loading code in `src/config.rs` parses these into tuples:

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

The `splitn(3, ',')` ensures that even if a filename contains a comma (unlikely but possible), only the first two commas are used as separators. The `filter_map` skips malformed lines gracefully — a line with only one comma is silently ignored.

### The RwLock<HashSet> Pattern

The actual IP loading happens in `RedisBucketLimiter::load_datacenter_ips()`:

```rust
pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
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

Several important design choices here:

1. **`tokio::fs::read_to_string`** — Async file reading. We don't want to block the Tokio runtime while reading potentially large IP files. If the files are on a network filesystem, blocking could stall the entire server.

2. **`self.datacenter_ips.write().await`** — We acquire a write lock on the RwLock. This gives us exclusive access to the HashSet, allowing us to insert multiple IPs. The `.await` is necessary because RwLock's `write()` returns a future in Tokio (it might need to wait for other readers to finish).

3. **Error handling** — If a file can't be read, we log a warning and continue. We don't crash the server because of a missing IP file. This is graceful degradation — the server starts with whatever IPs it could load.

4. **Line filtering** — Empty lines and comment lines (starting with `#`) are skipped. This makes the IP files human-readable and easy to maintain. You can add notes, organize by section, and comment out IPs you want to temporarily allow.

5. **`line.parse::<IpAddr>()`** — Parses the line as an IP address. Invalid lines (like comments or empty strings) are silently skipped via the `if let Ok(...)` pattern.

The `Arc<RwLock<HashSet<IpAddr>>>` pattern is extremely common in async Rust:

- **`Arc`** — Allows the HashSet to be shared across async tasks. When a new request handler task starts, it gets a clone of the `Arc`, which just increments a reference count. The actual data lives on the heap, shared by all clones.
- **`RwLock`** — Allows concurrent reads (which is what happens on every request) but exclusive writes (which only happens at startup). This is better than `Mutex`, which would force all operations to be sequential.
- **`HashSet<IpAddr>`** — O(1) membership testing. Checking if an IP is in the set is constant time, regardless of how many IPs are in the set. This is important — with tens of thousands of datacenter IPs, you need fast lookups.

```rust
// Checking if an IP is in the set (synchronous, but fast)
fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
    // Synchronous check - this is a best-effort check
    // For production, use a proper prefix tree (ipnet crate)
    false
}
```

⚠️ **Watch Out:** The current `is_datacenter_ip()` implementation is a placeholder that always returns `false`. In production, you'd use a proper prefix tree or the `ipnetwork` crate (which is already in FicHub's dependencies) to handle CIDR ranges like `52.0.0.0/8`. A CIDR range covers all IPs from 52.0.0.0 to 52.255.255.255 — you can't just store individual IPs; you need range matching.

The `ipnetwork` crate provides an efficient way to check if an IP falls within any of several CIDR ranges:

```rust
use ipnetwork::IpNetwork;

fn is_in_datacenter(ip: IpAddr, networks: &[IpNetwork]) -> bool {
    networks.iter().any(|net| net.contains(ip))
}
```

For better performance with many networks, you'd build a prefix tree (trie) that allows O(prefix-length) lookups instead of O(n) linear scans.

### Integration at Startup

The loading happens once during server startup, in `src/server.rs`:

```rust
// Load datacenter IPs if configured
if !config.ip_tag_sources.is_empty() {
    rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
}
```

This runs after the `RedisBucketLimiter` is created but before the server starts accepting requests. The `if` check avoids unnecessary work when no IP sources are configured.

Once loaded, the IP set lives in memory for the lifetime of the server. Since writes (loading) happen only at startup and reads happen on every request, the RwLock contention is minimal. In practice, the lock is almost never held by a writer.

For IP lists that change frequently, you could add a periodic reload task that re-reads the files every hour or day. But for most deployments, loading once at startup is sufficient.

### The RateLimiter Trait

The `RateLimiter` trait in `src/limiter/mod.rs` defines the interface that all rate limiters must implement:

```rust
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}
```

The `#[async_trait::async_trait]` macro enables async methods in traits. Without it, Rust's trait system can't handle async methods (as of the 2024 edition). The macro transforms the async methods into a form that can be stored in a vtable.

The `Send + Sync` bounds ensure the trait object can be shared across threads:

- **`Send`** — The type can be moved between threads
- **`Sync`** — The type can be referenced from multiple threads simultaneously

These bounds are required because `Box<dyn RateLimiter>` is stored in `AppState`, which is shared across all handler tasks via `Arc<AppState>`. Without `Send + Sync`, the compiler would refuse to share the rate limiter across async tasks.

The trait-based design means FicHub could swap out the rate limiter implementation without changing any handler code. Want a local (non-Redis) rate limiter for development? Implement `RateLimiter` for a `LocalBucketLimiter` struct. Want to use Memcached instead? Implement the trait for a `MemcacheLimiter`. Want to skip rate limiting entirely in tests? Implement `NullRateLimiter` that always returns `Allowed`.

In `src/server.rs`, the rate limiter is stored as a trait object:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

The `Box<dyn limiter::RateLimiter>` is a fat pointer — it stores both a pointer to the `RedisBucketLimiter` data and a pointer to the vtable (virtual function table) for the `RateLimiter` trait. This allows dynamic dispatch: the handler code calls `rate_limiter.check_ip()` without knowing which concrete implementation is behind it.

This indirection has a tiny performance cost (one extra pointer dereference), but the flexibility is worth it. You can swap implementations without recompiling handlers.

🧪 **Try It Yourself:** Write a `LocalBucketLimiter` struct that implements the `RateLimiter` trait using only in-memory state (no Redis). Use a `Mutex<HashMap<IpAddr, TokenBucket>>` to store per-IP state. This would be useful for local development when you don't have Redis running. Test it by creating several instances and verifying that rate limiting works correctly. Try running it under concurrent load to see if your in-memory implementation has any race conditions.

---

## Chapter 42: Rate Limiting Integration

### Integrating with Handlers

So far, we've built a rate limiter that can check IPs and manage token buckets. But how does it actually connect to FicHub's request handlers? The integration happens through Axum's state extraction system.

Every FicHub handler has access to the shared `AppState`, which contains the rate limiter:

```rust
pub struct AppState {
    // ...
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    // ...
}
```

Handlers access the rate limiter by extracting `State<Arc<AppState>>`:

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    // Check rate limit
    let result = state.rate_limiter.check_ip(remote_addr.ip()).await;
    
    match result {
        RateLimitResult::Allowed => {
            // Proceed with the request
        }
        RateLimitResult::Wait(seconds) => {
            // Return 429 Too Many Requests
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", seconds.to_string())],
                format!("Rate limit exceeded. Retry after {} seconds.", seconds),
            );
        }
        RateLimitResult::Blocked => {
            // Return 403 Forbidden
            return (
                StatusCode::FORBIDDEN,
                "Access denied.",
            );
        }
    }
    
    // ... handle the actual request
}
```

The `ConnectInfo(remote_addr)` extractor is how Axum passes the client's IP address to the handler. This works because the server is started with `into_make_service_with_connect_info()`:

```rust
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
.await
.expect("Server error");
```

Without `into_make_service_with_connect_info`, the `ConnectInfo` extractor would fail. This is a common source of confusion when setting up Axum — you need to opt into connection info extraction explicitly.

### Extracting the Client IP

The IP address comes from the TCP connection. When a client connects to FicHub, the OS creates a `SocketAddr` containing the client's IP and port. The `remote_addr.ip()` call extracts just the IP address.

But what if FicHub is behind a reverse proxy (like Nginx or Caddy)? In that case, the TCP connection is from the proxy, not the client. The client's real IP is in the `X-Forwarded-For` or `X-Real-IP` header.

FicHub handles this with trusted proxy configuration:

```rust
pub trusted_proxies: Vec<String>,
```

When processing requests from trusted proxies, FicHub reads the client IP from the forwarded header instead of the TCP connection. This is essential for rate limiting — without it, all requests would appear to come from the proxy's IP, and the rate limiter would treat everyone as a single user.

The `TRUSTED_PROXIES` environment variable specifies which proxy IPs to trust:

```bash
TRUSTED_PROXIES=10.0.0.1, 10.0.0.2
```

Only requests from these IPs will have their `X-Forwarded-For` header honored. This prevents malicious clients from spoofing their IP address by adding a fake `X-Forwarded-For` header.

Here's why IP spoofing is dangerous for rate limiting: if a client can fake their IP in the `X-Forwarded-For` header, they can create a new "virtual" IP for every request, completely bypassing the per-IP rate limit. By only trusting headers from known proxies, FicHub ensures that only the proxy's IP is used for rate limiting.

The proxy chain looks like this:

```
Client (192.168.1.100)
  → Nginx proxy (10.0.0.1, trusted)
    → FicHub server
```

FicHub sees the connection from 10.0.0.1 (the proxy). It checks the `X-Forwarded-For` header, which contains `192.168.1.100` (the real client). Since 10.0.0.1 is in the trusted proxies list, FicHub uses 192.168.1.100 for rate limiting.

If a malicious client sends a request directly (bypassing the proxy) with a fake `X-Forwarded-For: 1.2.3.4`, FicHub would use the TCP connection's IP (the client's real IP) for rate limiting, because the client's IP isn't in the trusted proxies list.

### Configuring Rate Limits

FicHub's rate limits are configured via the `dynamic_rate_limit` environment variable:

```bash
# Enable dynamic rate limiting (token bucket)
DYNAMIC_RATE_LIMIT=true

# Or use simple static delays
DYNAMIC_RATE_LIMIT=false
```

When `dynamic_rate_limit` is `true` (the default), FicHub uses the full token bucket algorithm with Redis. When `false`, it falls back to a simple random delay:

```rust
if !self.dynamic_rate_limit {
    // Simple static delay
    let delay = self.static_delay_base 
        + rand::random::<f64>() * self.static_delay_base;
    tokio::time::sleep(
        tokio::time::Duration::from_secs_f64(delay)
    ).await;
    return RateLimitResult::Allowed;
}
```

The static delay adds a random component (`rand::random::<f64>() * self.static_delay_base`) to prevent thundering herd problems. If all users got the exact same delay, they'd all retry at the same time, causing synchronized bursts.

With `static_delay_base = 0.1`, the delay is between 0.1 and 0.2 seconds — just enough to prevent bot-like behavior without noticeably affecting real users.

### The dynamic_rate_limit Flag

The `dynamic_rate_limit` flag serves several purposes:

1. **Development mode** — When developing locally, you might not have Redis running. Setting `DYNAMIC_RATE_LIMIT=false` lets the server start without relying on Redis for rate limiting (though it still needs a Redis connection for other features like caching).

2. **Graceful degradation** — If Redis is having issues, you can flip this flag to false to maintain basic protection without Redis. The server doesn't need to be restarted if the config is reloaded.

3. **Testing** — During integration tests, you might want to skip rate limiting entirely to test other functionality. Setting this to false with a very low `static_delay_base` makes tests run fast.

4. **Migration** — When first deploying rate limiting, you might start with `false` to verify the integration, then switch to `true` once you're confident everything works.

The flag is checked at the beginning of `check_ip()`, before any Redis calls. This means even if Redis is completely unreachable, the static delay still provides basic protection. The server never crashes because of a rate limiter failure.

### Monitoring Rate Limit Hits

FicHub uses the `tracing` crate for structured logging. Rate limit events can be observed through tracing spans and events. When a rate limit is hit, the handler typically returns a `429` status, which can be monitored through access logs.

Key metrics to watch:

```bash
# Count rate limit hits in the last hour
grep "429" /var/log/fichub/access.log | wc -l

# See which IPs are hitting rate limits most
grep "429" /var/log/fichub/access.log | \
  awk '{print $1}' | sort | uniq -c | sort -rn | head -10

# Check Redis memory usage (rate limit keys consume memory)
redis-cli INFO memory | grep used_memory_human

# List all rate limit keys in Redis
redis-cli KEYS "rate:*" | wc -l

# Check a specific IP's bucket
redis-cli HMGET rate:ip:192.168.1.1 value last_drain
```

The `tracing::info!` call in `load_datacenter_ips` gives visibility into the datacenter IP loading:

```rust
tracing::info!(
    "Loaded {} datacenter IPs",
    self.datacenter_ips.read().await.len()
);
```

When the server starts, you'll see a log line like:

```
INFO Loaded 14523 datacenter IPs
```

This confirms that the IP lists loaded correctly. If the count is 0, check your `IP_TAG_SOURCES` configuration.

### Error Handling in the Integration

The rate limiter integration follows FicHub's overall philosophy of graceful degradation. If anything goes wrong with rate limiting, the request should still be served:

```rust
// From check_ip - unwrap_or(-1.0) handles Redis errors
let global_wait = self.check_bucket(
    "rate:global", 
    self.global_capacity, 
    self.global_flow
).await.unwrap_or(-1.0);
```

The `unwrap_or(-1.0)` means: if the Redis call fails, default to "no wait needed." This is better than crashing the handler and returning a 500 error.

Similarly, the `report_failure` method ignores errors:

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize(
        "rate:global", 
        self.global_capacity, 
        self.global_flow
    ).await;
    let _ = self.penalize(
        &ip_key, 
        self.ip_capacity, 
        self.ip_flow
    ).await;
}
```

The `let _ = ...` pattern deliberately discards the Result. If the penalty fails, the IP is still rate-limited by the normal token bucket — the penalty just makes it more aggressive.

⚠️ **Watch Out:** The `unwrap_or` pattern means rate limiting is best-effort. In a production environment, you should monitor for Redis failures and alert operations staff. A few minutes without rate limiting is fine; hours without it is a problem. Consider setting up Redis monitoring with tools like Redis Exporter for Prometheus.

### Rate Limiting and the Cache Pipeline

Rate limiting interacts with FicHub's caching system. The flow is:

```
Request arrives
  │
  ├── Rate limit check
  │     Allowed → Continue
  │     Wait → Return 429
  │     Blocked → Return 403
  │
  ├── Cache lookup
  │     Hit → Serve cached file (fast)
  │     Miss → Scrape + export
  │
  └── Response
```

Rate limiting sits at the front of this pipeline. This means:

- **Every request is rate-limited** — Even cached requests go through rate limiting. This is a deliberate choice: even cached requests consume server resources (HTTP parsing, cache lookup, file serving). A bot hammering a cached URL still wastes resources.
- **Failed scrapers are penalized** — When a scraper fails, the IP's token bucket is penalized, slowing down future requests. This prevents abusive clients from wasting scrape resources.
- **Global rate limiting protects upstream sites** — The global bucket ensures FicHub doesn't overwhelm AO3, FanFiction.net, or other source sites with too many requests. If FicHub is processing 30 requests per second globally, and each request triggers a scrape, the upstream site sees 30 requests per second — which is usually acceptable.

The two-level rate limiting creates a natural flow control:

1. **Global level** — Controls total outbound traffic to upstream sites. If FicHub gets busy, the global bucket slows down all requests proportionally.
2. **Per-IP level** — Controls individual user behavior. Even if the global bucket is underutilized, a single user can't hog all the capacity.

This layered approach is common in production systems. Each layer handles a different concern, and each can be tuned independently.

### Rate Limit Headers

When a request is rate-limited, FicHub should tell the client when to retry. The standard way is via HTTP headers:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 5
Content-Type: text/plain

Rate limit exceeded. Retry after 5 seconds.
```

The `Retry-After` header tells the client exactly how many seconds to wait before retrying. Well-behaved HTTP clients (like web browsers and API clients) respect this header and automatically wait before sending the next request.

For the `Blocked` case (datacenter IPs), there's no `Retry-After` header because the client should not retry:

```http
HTTP/1.1 403 Forbidden
Content-Type: text/plain

Access denied.
```

💡 **Key Concept:** The `Retry-After` header is part of the HTTP specification (RFC 7231). Using it correctly means that clients don't need to guess when to retry — they get an exact number from the server. This prevents "thundering herd" problems where all blocked clients retry at the same time.

### Performance Impact

Let's quantify the performance impact of rate limiting:

- **Static delay mode** — Adds 100-200ms of latency to every request (the random sleep). This is noticeable but acceptable for a download service.
- **Dynamic rate limiting** — Adds 1-10ms of latency (one Redis round-trip). This is negligible compared to the 50-500ms it takes to process a typical request.
- **Datacenter IP check** — Adds <1ms (HashSet lookup). This is so fast it's effectively free.

For a normal user making one request every few seconds, the rate limiter has zero practical impact. For a user making rapid requests, the rate limiter adds appropriate delays. For a bot trying to hammer the server, the rate limiter provides strong protection.

The key insight is that rate limiting is a **preemptive** optimization. It prevents problems before they happen, rather than reacting to them after the fact. A server without rate limiting might handle 100 requests per second fine — until a bot sends 10,000 requests per second and crashes it. Rate limiting ensures the server stays within its capacity limits.

In production, you'd want to monitor these metrics:

1. **Rate limit hit rate** — What percentage of requests are being rate-limited? If it's more than 1%, you might need to tune the limits.
2. **Redis latency** — How long does each rate limit check take? If Redis is slow, the rate limiter becomes a bottleneck.
3. **Datacenter IP block rate** — How many requests are being blocked by the IP list? A high number suggests your list needs updating.
4. **Memory usage** — How many rate limit keys are in Redis? If the number grows unbounded, you need key expiration.

These metrics help you tune the rate limiting parameters for your specific deployment. A high-traffic server might need higher global capacity; a server with many bots might need lower per-IP limits.

🧪 **Try It Yourself:** Write a simple Axum handler that extracts the client IP and returns it as JSON. Add rate limiting to the handler using the `RateLimiter` trait. Test it with `curl` by sending multiple rapid requests. Verify that you eventually get a `429` response with a `Retry-After` header. Try measuring the latency difference between static delay mode and dynamic rate limiting mode using `time curl ...`.

---

## Chapter 43: Rate Limiting Deep Dive

### Lua Script Line-by-Line Analysis

Now that we've seen the token bucket rate limiter in action, let's dissect the Lua script line by line. Understanding this script is crucial because it's the mathematical heart of FicHub's rate limiting.

Here's the complete script again:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

### Lines 1-4: Parameter Extraction

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])
```

Redis passes keys and arguments to Lua scripts as arrays of strings. `KEYS[1]` is the first key (e.g., `"rate:ip:192.168.1.1"`). `ARGV[1]` through `ARGV[3]` are the arguments.

The `tonumber()` calls are critical. Redis always passes arguments as strings, even when they represent numbers. Without `tonumber()`, Lua would treat `"30"` as the string `"30"`, and `value + "30"` would do string concatenation (producing `"9030"` if value is `"90"`) instead of addition. The `tonumber()` conversions ensure all math operations work on floating-point numbers.

The key naming convention (`rate:global`, `rate:ip:192.168.1.1`) is defined by FicHub's Rust code. The Lua script doesn't care what the key name is — it just uses whatever key it's given.

### Lines 6-8: Reading Current State

```lua
local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])
```

`HMGET` reads two fields from the hash in a single operation. This is more efficient than two separate `HGET` calls because it involves only one network round-trip.

If the key doesn't exist yet, `bucket[1]` and `bucket[2]` will be `nil`. The `tonumber(nil)` call returns `nil` in Lua, which is how we detect a first-time bucket.

The `redis.call()` function is Redis's way of executing commands from within Lua. It's similar to `redis.pcall()` but propagates errors instead of catching them. If the `HMGET` fails (which is unlikely for a hash operation), the entire script fails and returns an error to the caller.

### Lines 10-11: Getting Current Time

```lua
local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000
```

The `TIME` command returns `[seconds, microseconds]` as an array of two strings. `now[1]` is Unix seconds (e.g., `1690000000`), and `now[2]` is microseconds (e.g., `500000` for 0.5 seconds).

The formula `now[1] + now[2]/1000000` converts to a single floating-point number: `1690000000.5`.

Using Redis time instead of the server's system time is important in a distributed setup. If FicHub runs on multiple servers (for load balancing), their system clocks might differ by a few seconds. Redis, being the single source of truth for rate limit state, should also be the source of truth for time.

The microsecond precision ensures that rapid successive requests (within the same second) still see meaningful time differences. Without microseconds, two requests in the same second would see `elapsed = 0`, which would incorrectly calculate zero new tokens.

### Lines 13-17: First-Time Initialization

```lua
if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end
```

When an IP makes its very first request, no bucket exists in Redis. This block creates one with the bucket full to capacity. This is the "burst" allowance — new users can immediately make several requests without waiting.

Without this block, a new IP would have `value = nil` and `last_drain = nil`, causing all subsequent math operations to fail (Lua treats `nil` in arithmetic as an error).

The initialization sets `last_drain = now_sec`, which means the elapsed time since "last drain" is zero. This is correct — the bucket was just created, so no tokens should have refilled yet.

### Lines 19-20: Calculating New Tokens

```lua
local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
```

The `elapsed` calculation is how many seconds have passed since the last time this bucket was accessed. The `new_tokens` calculation is the core of the algorithm:

- `value + elapsed * flow` — Add new tokens based on elapsed time and flow rate
- `math.min(capacity, ...)` — Never exceed the bucket's capacity

For example, if `value = 20`, `elapsed = 10`, `flow = 0.116`, and `capacity = 30`:

```
new_tokens = math.min(30, 20 + 10 * 0.116)
           = math.min(30, 20 + 1.16)
           = math.min(30, 21.16)
           = 21.16
```

The `math.min` cap is crucial. Without it, a user who doesn't make requests for a long time could accumulate unlimited tokens. If `value = 0` and `elapsed = 1000` seconds (about 16 minutes), the uncapped value would be `0 + 1000 * 0.116 = 116` — almost 4x the bucket's capacity. The cap at 30 ensures the burst limit is respected.

💡 **Key Concept:** The `math.min` function is Lua's built-in minimum function. It takes two arguments and returns the smaller one. This single line of code enforces the fundamental constraint of the token bucket: you can never have more tokens than the bucket can hold.

### Lines 21-27: The Allow/Deny Decision

```lua
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

The `allowed` variable is how many tokens would remain after the request. If positive or zero, the request is allowed. If negative, we need to wait.

When allowed, we update the hash with the new token count and current time. The `-1` return value signals "allowed" to the Rust code. We choose -1 because wait times are always positive (or zero), so -1 can never be confused with a real wait time.

When denied, we calculate the wait time:

```
wait = (requested - new_tokens) / flow
```

This is simple algebra. If we need `requested` tokens but only have `new_tokens`, we need to wait for `(requested - new_tokens)` more tokens to arrive. At a rate of `flow` tokens per second, the wait time is `(requested - new_tokens) / flow`.

For example, if `requested = 1`, `new_tokens = 0.5`, and `flow = 0.116`:

```
wait = (1 - 0.5) / 0.116
     = 0.5 / 0.116
     = 4.31 seconds
```

The user would need to wait about 4.31 seconds before retrying.

Notice that when denied, we **don't** update the hash. The token count and last drain time stay the same. This means the next request (after waiting) will see the same starting state, with more tokens available due to the elapsed time. This is correct — we don't want to penalize a denied request by updating the timestamp.

### Redis EVALSHA vs EVAL

Redis provides two ways to execute Lua scripts:

1. **`EVAL script numkeys key [key ...] arg [arg ...]`** — Sends the full script text every time
2. **`EVALSHA sha1 numkeys key [key ...] arg [arg ...]`** — Sends just the SHA1 hash of a previously loaded script

FicHub uses `EVALSHA` because:

```rust
// Load once at startup
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;

// Use SHA on every request (much smaller payload)
let result: f64 = redis::cmd("EVALSHA")
    .arg(&self.lua_sha[..])
    .arg(1)
    .arg(key)
    .arg(1.0)
    .arg(capacity)
    .arg(flow)
    .query_async(&mut conn)
    .await?;
```

The Lua script is about 600 bytes. Sending 600 bytes on every request wastes bandwidth. The SHA1 hash is 40 characters. That's a 93% reduction in payload size.

But there's a catch: Redis doesn't guarantee the script stays in memory. If Redis restarts or runs out of memory, the script is lost, and `EVALSHA` will return a `NOSCRIPT` error. For this reason, some implementations fall back to `EVAL` if `EVALSHA` fails:

```rust
// Robust pattern (not used in FicHub for simplicity)
match try_evalsha(sha, args).await {
    Ok(result) => result,
    Err(NOSCRIPT) => {
        // Script was evicted — reload it
        let new_sha = script_load(script).await?;
        evalsha(new_sha, args).await?
    }
    Err(e) => return Err(e),
}
```

FicHub's approach is simpler — it assumes the script stays in memory. This is usually fine for a small, frequently-used script. The Redis `maxmemory-policy` setting can prevent scripts from be scripts are stored in a separate memory space from data keys, so they're not affected by `maxmemory` evictions. However, if Redis restarts, the scripts are gone and need to be reloaded.

The `SCRIPT LOAD` command is idempotent — loading the same script twice returns the same SHA. This means if FicHub needed to reload the script (e.g., after a Redis restart), it could safely call `SCRIPT LOAD` again without worrying about duplicates.

💡 **Key Concept:** The `SCRIPT LOAD` command returns a SHA1 hash of the script content. This hash is deterministic — the same script always produces the same hash. If you modify even one character of the script, the hash changes completely. This means if you update the rate limiting logic, you need to restart the server to load the new script. Redis doesn't cache multiple versions of the same script — it's one SHA per script content.

### Performance Characteristics

Let's analyze the performance of this rate limiting system:

**Redis round-trip:** Each `EVALSHA` call requires one network round-trip to Redis. On the same machine, this is typically 0.1-0.5 milliseconds. Over a network (e.g., to a Redis server in the same datacenter), it's 1-5 milliseconds.

**Script execution:** The Lua script does one `HMGET`, one `TIME`, and either one `HMSET` (allowed) or no writes (denied). This is extremely fast — sub-millisecond on modern hardware.

**Memory usage per bucket:** Each rate limit key is a Redis hash with two fields. A hash with two fields uses approximately 200-300 bytes of memory. For 10,000 unique IPs, that's about 2-3 MB — negligible.

**Concurrency:** Because the Lua script runs atomically, there's no contention between concurrent requests. Each request waits only for its own script execution, not for other requests.

The entire rate limiting check — from receiving the HTTP request to making the allow/deny decision — adds about 1-10 milliseconds of latency. For a web server that already takes 50-500ms to process a request, this overhead is minimal.

Let's compare this to alternative approaches:

| Approach | Latency | Atomicity | Memory |
|---|---|---|---|
| Redis Lua script (FicHub) | 1-10ms | Guaranteed | ~300 bytes/IP |
| PostgreSQL row-level lock | 5-20ms | Guaranteed | ~1KB/IP |
| In-memory HashMap (single server) | <1ms | Not concurrent-safe | ~500 bytes/IP |
| File-based (shared filesystem) | 10-50ms | Depends on filesystem | ~1KB/IP |

FicHub's approach is a good balance: low latency, guaranteed atomicity, and reasonable memory usage. The in-memory approach is faster but doesn't work across multiple servers. PostgreSQL is more durable but slower. File-based is simple but unreliable under concurrency.

💡 **Key Concept:** The 1-10ms overhead of rate limiting is often negligible compared to other parts of the request lifecycle. Network latency to the client is typically 50-200ms. Database queries take 5-20ms. File I/O takes 1-10ms. The rate limiter adds one more step in a chain of many, and it's usually one of the fastest.

### When Things Go Wrong

Here are some common failure modes and how the fail-open strategy handles them:

**Redis connection lost:** `check_bucket()` returns an error, `unwrap_or(-1.0)` returns -1.0, request is allowed. Rate limiting is temporarily disabled until Redis reconnects. The connection pool in the `redis` crate handles reconnection automatically.

**Redis out of memory:** If Redis can't allocate memory for a new rate limit key, the `HMSET` call fails. The Lua script propagates the error, `check_bucket()` returns an error, and the request is allowed. In production, you'd set `maxmemory` and `maxmemory-policy` to prevent this.

**Clock skew:** If Redis and the FicHub server have different clocks, the `elapsed` calculation might be slightly off. This could allow a few extra requests or deny a few legitimate ones. Using `redis.call('TIME')` inside the Lua script eliminates this problem — the time comes from the same Redis instance that stores the state.

**Script eviction:** If Redis restarts, the Lua script is gone. The next `EVALSHA` call fails with `NOSCRIPT`. In FicHub's current implementation, this error is caught by `unwrap_or(-1.0)` and the request is allowed. A more robust implementation would fall back to `EVAL`.

**Network partition:** If the network between FicHub and Redis is flaky, some requests succeed and some fail. The fail-open strategy means failed checks are allowed, while successful checks are properly rate-limited. This is the best behavior during partial failures.

The key insight is that rate limiting is a **defense layer**, not a security boundary. It's designed to prevent abuse and ensure fairness, but it's not meant to be bulletproof. The fail-open strategy prioritizes availability over perfect rate limiting.

### Tuning the Parameters

The token bucket parameters (capacity and flow rate) directly affect user experience. Let's think about what reasonable values look like:

**Global bucket (capacity=150, flow=30):**
- Sustained rate: 30 requests/second across all users
- Burst capacity: 150 requests in one burst
- This means FicHub can serve about 30 scrapes per second to upstream sites

**Per-IP bucket (capacity=30, flow=0.116):**
- Sustained rate: ~1 request every 8.6 seconds
- Burst capacity: 30 requests in one burst
- A real user would never hit the sustained rate — they'd download a fic, read it, come back hours later

If you wanted to be more generous to real users, you could increase `ip_flow` to allow faster sustained rates. If you wanted stricter protection, you could decrease `ip_capacity` to limit burst size.

The key trade-off is between user experience and protection:

| Configuration | User Experience | Protection |
|---|---|---|
| High capacity, high flow | Very permissive, real users never notice | Weaker, bots can do more damage |
| Low capacity, low flow | Users might hit limits during normal use | Stronger, but frustrated users |
| High capacity, low flow | Bursty, then slow | Moderate, catches sustained abuse |

The current values (30 capacity, 0.116 flow) are tuned for fanfiction downloads — users make a request every few minutes, not every few seconds. The burst capacity of 30 handles the case where someone opens multiple fics in new tabs simultaneously.

### What We've Built

In this part, we've built a complete rate limiting system:

1. **Redis** as the fast, shared state store for token bucket data
2. **The token bucket algorithm** providing smooth, burst-friendly rate limiting
3. **Lua scripts** ensuring atomicity in a concurrent environment
4. **Two-level limiting** (global + per-IP) for both system protection and fairness
5. **Datacenter IP blocking** filtering out bot traffic before rate limiting
6. **Fail-open design** maintaining availability when Redis is unhealthy
7. **Trait-based architecture** allowing future rate limiter implementations
8. **Configuration-driven** behavior with the `dynamic_rate_limit` flag
9. **Penalty system** that slows down failing scrapers more aggressively
10. **Proxy-aware IP extraction** handling reverse proxy deployments correctly

The result is a rate limiting system that's fast (sub-millisecond overhead), fair (per-IP and global limits), robust (fails open), and maintainable (clean separation of concerns through traits).

What makes this system elegant is how the pieces fit together. Redis provides the shared state. Lua scripts provide atomicity. The token bucket algorithm provides smooth rate limiting. The trait abstraction allows swapping implementations. And the fail-open strategy ensures the system degrades gracefully.

Each piece is simple on its own. Redis is just a key-value store. The token bucket is just basic math. Lua scripting is just code that runs inside Redis. But combined, they form a sophisticated rate limiting system that protects FicHub from abuse while remaining invisible to legitimate users.

This is a common pattern in systems engineering: complex behavior emerging from simple, well-designed components. The key is getting each component right and connecting them cleanly.

---

*In the next part, we'll explore FicHub's recommendation engine — how it suggests new fics based on reading history, and how the voting system lets readers shape the suggestions for everyone.*

# Part 11: Database Performance

---

# Chapter 51: PostgreSQL Internals for Rust Developers

Understanding how PostgreSQL works under the hood helps you write better queries and optimize performance.

## How PostgreSQL Processes Queries

1. **Parse** — SQL text is parsed into a parse tree
2. **Analyze** — The query planner generates possible execution plans
3. **Plan** — The cheapest plan is selected based on statistics
4. **Execute** — The plan is executed and results are returned

## The Query Planner

PostgreSQL uses a cost-based optimizer. It estimates the cost of each execution plan and chooses the cheapest one:

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM fic_info WHERE words > 10000 ORDER BY fic_updated DESC LIMIT 20;
```

### Understanding EXPLAIN Output

```
Limit  (cost=0.00..1234.56 rows=20 width=123) (actual time=0.123..4.567 rows=20 loops=1)
  ->  Index Scan using idx_fic_info_words on fic_info  (cost=0.00..123456.78 rows=10000 width=123) (actual time=0.123..4.567 rows=20 loops=1)
        Filter: (words > 10000)
Planning Time: 0.123 ms
Execution Time: 4.690 ms
```

Key metrics:
- **cost** — Estimated cost (lower is better)
- **rows** — Estimated number of rows
- **width** — Estimated row width in bytes
- **actual time** — Real execution time
- **loops** — Number of times the node was executed

## Table Statistics

PostgreSQL maintains statistics about table contents:

```sql
-- View statistics for a table
SELECT * FROM pg_stats WHERE tablename = 'fic_info';

-- Update statistics
ANALYZE fic_info;
```

## Connection Architecture

PostgreSQL uses a process-per-connection model:
- Each connection gets a separate OS process
- Processes communicate through shared memory
- The postmaster (main process) manages connections

**Real-world analogy:** PostgreSQL is like a bank with multiple tellers. Each teller (connection) handles one customer at a time. The bank manager (postmaster) assigns customers to available tellers. If all tellers are busy, customers wait in line.

## 📝 Practice Exercises

1. **EXPLAIN Analysis:** Run EXPLAIN ANALYZE on 5 different queries and interpret the output.

2. **Statistics:** Check the statistics for the `fic_info` table. Are they up to date?

3. **Connection Limits:** Test what happens when you exceed PostgreSQL's max_connections limit.

---

# Chapter 52: Connection Pool Tuning

Connection pool tuning is critical for performance. This chapter covers how to optimize SQLx's connection pool.

## Pool Configuration

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(2)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(database_url)
    .await?;
```

### Tuning Parameters

| Parameter | Description | Small Server | Large Server |
|-----------|-------------|--------------|--------------|
| max_connections | Max pool size | 10-20 | 50-100 |
| min_connections | Min idle connections | 2-5 | 10-20 |
| acquire_timeout | Wait for connection | 30s | 10s |
| idle_timeout | Close idle connections | 600s | 300s |
| max_lifetime | Connection age limit | 1800s | 900s |

### Monitoring Pool Usage

```rust
let stats = pool.stats();
tracing::info!(
    active = stats.active_connections(),
    idle = stats.idle_connections(),
    waiting = stats.waiting(),
    max = pool.options().max_connections(),
    "Pool statistics"
);
```

### Connection Pool Sizing Formula

A common formula is:

```
connections = (2 × num_cpu_cores) + num_disks
```

For a 4-core server with SSD:
```
connections = (2 × 4) + 1 = 9
```

For I/O-heavy workloads (like FicHub), you can increase this:
```
connections = (2 × 4) + 4 = 12
```

## Connection Pool Anti-Patterns

### Creating Pools Per Request

```rust
// BAD: Creates a new pool for every request
async fn handler() -> Result<Response, Error> {
    let pool = PgPool::connect(database_url).await?;
    // ... use pool ...
    pool.close().await;
}
```

### Sharing Pool Incorrectly

```rust
// BAD: Creating pool inside async task
tokio::spawn(async {
    let pool = PgPool::connect(database_url).await?;
    // Pool is dropped when task completes
});
```

### Best Practice

```rust
// GOOD: Create pool once, share via Arc
let pool = PgPool::connect(database_url).await?;
let state = Arc::new(AppState { pool, /* ... */ });
```

## 📝 Practice Exercises

1. **Pool Monitoring:** Add pool monitoring that logs statistics every 60 seconds.

2. **Load Testing:** Use `pgbench` to test PostgreSQL performance with different pool sizes.

3. **Connection Leaks:** Write a test that verifies connections are properly returned to the pool.

---

# Chapter 53: Query Optimization with EXPLAIN

This chapter covers advanced EXPLAIN techniques for optimizing queries.

## EXPLAIN Options

```sql
-- Basic explanation
EXPLAIN SELECT * FROM fic_info WHERE words > 10000;

-- With actual execution times
EXPLAIN (ANALYZE) SELECT * FROM fic_info WHERE words > 10000;

-- With buffer usage
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM fic_info WHERE words > 10000;

-- With timing
EXPLAIN (ANALYZE, TIMING) SELECT * FROM fic_info WHERE words > 10000;

-- JSON format
EXPLAIN (ANALYZE, FORMAT JSON) SELECT * FROM fic_info WHERE words > 10000;
```

## Common Query Patterns

### Sequential Scan vs Index Scan

```sql
-- Sequential scan (slow for large tables)
EXPLAIN SELECT * FROM fic_info WHERE title LIKE '%harry%';
-- Output: Seq Scan on fic_info

-- Index scan (fast)
CREATE INDEX idx_fic_info_title ON fic_info USING gin(to_tsvector('english', title));
EXPLAIN SELECT * FROM fic_info WHERE to_tsvector('english', title) @@ plainto_tsquery('english', 'harry');
-- Output: Index Scan using idx_fic_info_title
```

### Join Optimization

```sql
-- Nested Loop (good for small result sets)
EXPLAIN SELECT fi.*, ft.score
FROM fic_info fi
JOIN fic_tags ft ON fi.id = ft.url_id
WHERE ft.tag_id = 1;

-- Hash Join (good for large result sets)
EXPLAIN SELECT fi.*, ft.score
FROM fic_info fi
JOIN fic_tags ft ON fi.id = ft.url_id;

-- Merge Join (good for sorted data)
EXPLAIN SELECT fi.*
FROM fic_info fi
WHERE fi.id IN (SELECT url_id FROM fic_tags WHERE tag_id = 1);
```

### Subquery Optimization

```sql
-- EXISTS is often faster than IN
EXPLAIN SELECT * FROM fic_info fi
WHERE EXISTS (SELECT 1 FROM fic_tags ft WHERE ft.url_id = fi.id AND ft.tag_id = 1);

-- LATERAL JOIN for correlated subqueries
EXPLAIN SELECT fi.*, ft.score
FROM fic_info fi
CROSS JOIN LATERAL (
    SELECT score FROM fic_tags
    WHERE url_id = fi.id
    ORDER BY score DESC
    LIMIT 1
) ft;
```

## Index Types

### B-Tree Index

```sql
-- Default index type, good for equality and range queries
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_status ON fic_info(status);
```

### GIN Index

```sql
-- Good for full-text search and arrays
CREATE INDEX idx_fic_info_title_search ON fic_info
    USING gin(to_tsvector('english', title));
```

### GiST Index

```sql
-- Good for geometric data and full-text search
CREATE INDEX idx_fic_info_description ON fic_info
    USING gist(to_tsvector('english', description));
```

### Partial Index

```sql
-- Index only active stories
CREATE INDEX idx_fic_info_active ON fic_info(fic_updated)
    WHERE status = 'ongoing';
```

## 📝 Practice Exercises

1. **Index Analysis:** Run EXPLAIN on 10 queries and determine which ones would benefit from indexes.

2. **Index Creation:** Create indexes for the 5 most common query patterns in FicHub.

3. **Partial Index:** Create a partial index for stories updated in the last 7 days.

---

# Chapter 54: Indexing Strategies

This chapter covers when and how to create indexes.

## When to Create an Index

Create an index when:
- A column is frequently used in WHERE clauses
- A column is used in JOIN conditions
- A column is used in ORDER BY or GROUP BY
- A query returns a small fraction of the table

Don't create an index when:
- The table is small (< 10,000 rows)
- The column has low cardinality (few unique values)
- The column is rarely queried

## Index Design Principles

### Prefix Indexes

```sql
-- Index only the first 10 characters
CREATE INDEX idx_fic_info_title_prefix ON fic_info(left(title, 10));
```

### Composite Indexes

```sql
-- Index for queries filtering on status AND sorting by words
CREATE INDEX idx_fic_info_status_words ON fic_info(status, words);
```

The order matters: put the most selective column first.

### Covering Indexes

```sql
-- Include all columns needed by the query
CREATE INDEX idx_fic_info_covering ON fic_info(status, words)
    INCLUDE (title, author, chapters);
```

This allows PostgreSQL to answer the query from the index alone, without touching the table.

## Index Maintenance

```sql
-- Rebuild an index
REINDEX INDEX idx_fic_info_words;

-- Check index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;

-- Find unused indexes
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0
AND schemaname = 'public';
```

## 📝 Practice Exercises

1. **Index Audit:** Analyze FicHub's queries and recommend indexes.

2. **Index Bloat:** Check for index bloat and rebuild bloated indexes.

3. **Index Usage:** Monitor index usage over a week and remove unused indexes.

---

# Chapter 55: Batch Operations and Bulk Inserts

Batch operations are much faster than individual inserts.

## Bulk Insert with sqlx

```rust
async fn bulk_insert_fic_tags(
    pool: &PgPool,
    tags: &[(String, i32, i16)],
) -> Result<(), sqlx::Error> {
    let mut query = String::from(
        "INSERT INTO fic_tags (url_id, tag_id, score) VALUES "
    );
    
    for (i, _) in tags.iter().enumerate() {
        if i > 0 {
            query.push(',');
        }
        query.push_str(&format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3));
    }
    
    query.push_str(" ON CONFLICT DO NOTHING");
    
    let mut q = sqlx::query(&query);
    for (url_id, tag_id, score) in tags {
        q = q.bind(url_id).bind(tag_id).bind(score);
    }
    
    q.execute(pool).await?;
    Ok(())
}
```

## COPY Protocol

For maximum performance, use PostgreSQL's COPY protocol:

```rust
use sqlx::Copy;

async fn bulk_copy(pool: &PgPool, data: Vec<FicInfo>) -> Result<(), sqlx::Error> {
    let mut copy = pool.copy()
        .table("fic_info")
        .columns(&["id", "title", "author", "words"])
        .finish()
        .await?;
    
    for fic in &data {
        copy.send(&fic.id).await?;
        copy.send(&fic.title).await?;
        copy.send(&fic.author).await?;
        copy.send(fic.words).await?;
    }
    
    copy.finish().await?;
    Ok(())
}
```

## Batch Size Guidelines

| Operation | Recommended Batch Size |
|-----------|----------------------|
| INSERT | 1,000 - 10,000 rows |
| UPDATE | 100 - 1,000 rows |
| DELETE | 1,000 - 10,000 rows |
| COPY | No limit (streaming) |

## 📝 Practice Exercises

1. **Bulk Insert Benchmark:** Compare individual inserts vs bulk inserts for 10,000 rows.

2. **Batch Update:** Write a function that updates 1,000 records in a single transaction.

3. **COPY Implementation:** Use PostgreSQL's COPY protocol to import 100,000 rows.

---

# Chapter 56: Connection Pooling Deep Dive

This chapter covers advanced connection pooling techniques.

## PgBouncer

PgBouncer is a lightweight connection pooler for PostgreSQL:

```ini
[databases]
fichub = host=localhost port=5432 dbname=fichub

[pgbouncer]
listen_port = 6432
listen_addr = *
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 20
```

### Pool Modes

- **Session** — Connection is assigned for the entire session
- **Transaction** — Connection is returned after each transaction
- **Statement** — Connection is returned after each statement (no multi-statement transactions)

## Connection Pool Monitoring

```sql
-- Check active connections
SELECT
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query_start,
    now() - query_start AS duration
FROM pg_stat_activity
WHERE datname = 'fichub';

-- Check connection count
SELECT
    count(*) as total,
    state
FROM pg_stat_activity
WHERE datname = 'fichub'
GROUP BY state;
```

## Connection Pool in Rust

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)
    .before_connect(|conn, _meta| {
        Box::pin(async move {
            // Set connection-level parameters
            sqlx::query("SET statement_timeout = '30s'")
                .execute(conn)
                .await?;
            Ok(())
        })
    })
    .connect(database_url)
    .await?;
```

## 📝 Practice Exercises

1. **PgBouncer Setup:** Set up PgBouncer in front of PostgreSQL and compare performance.

2. **Connection Monitoring:** Write a query that shows connection usage over time.

3. **Pool Exhaustion:** Test what happens when all connections are busy.

---

# Chapter 57: Database Migrations at Scale

Managing migrations in production requires care and planning.

## Migration Best Practices

### Always Test Migrations

```bash
# Run migrations on a test database
DATABASE_URL=postgres://localhost/fichub_test sqlx migrate run --source migrations

# Revert and re-run
sqlx migrate revert --source migrations
sqlx migrate run --source migrations
```

### Backward-Compatible Migrations

```sql
-- Step 1: Add new column (nullable)
ALTER TABLE fic_info ADD COLUMN new_field TEXT;

-- Step 2: Backfill data
UPDATE fic_info SET new_field = 'default' WHERE new_field IS NULL;

-- Step 3: Add NOT NULL constraint
ALTER TABLE fic_info ALTER COLUMN new_field SET NOT NULL;
```

### Zero-Downtime Migrations

```sql
-- Use CREATE INDEX CONCURRENTLY to avoid locking
CREATE INDEX CONCURRENTLY idx_fic_info_new ON fic_info(new_field);

-- Use ALTER TABLE ... ADD COLUMN with DEFAULT for fast adds
ALTER TABLE fic_info ADD COLUMN new_field TEXT DEFAULT 'value';
```

## Migration Versioning

```sql
-- migrations/005_add_search_index.sql
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_fic_info_search
    ON fic_info USING gin(to_tsvector('english', title || ' ' || author));
```

## Rollback Strategy

Always write rollback migrations:

```sql
-- migrations/005_add_search_index.sql (rollback)
DROP INDEX CONCURRENTLY IF EXISTS idx_fic_info_search;
```

## 📝 Practice Exercises

1. **Migration Testing:** Create a migration, test it, and verify it can be rolled back.

2. **Zero-Downtime Migration:** Perform a migration on a large table without locking.

3. **Migration Scripts:** Write a script that applies migrations with rollback support.


# Extended Content: Database Schema Complete Reference

---

# FicHub Database Schema Complete Reference

## Overview

FicHub uses PostgreSQL as its primary database. The schema is managed through SQLx migrations and includes tables for story metadata, exports, tags, recommendations, and more.

## Table: fic_info

The core table storing fanfiction story metadata.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | VARCHAR(128) | NO | — | Unique story identifier |
| created | TIMESTAMPTZ | YES | NOW() | Record creation time |
| updated | TIMESTAMPTZ | YES | NOW() | Record update time |
| title | TEXT | NO | — | Story title |
| author | TEXT | NO | — | Author name |
| author_url | TEXT | YES | — | Author profile URL |
| author_local_id | TEXT | YES | — | Author ID on source site |
| chapters | INT4 | NO | 1 | Number of chapters |
| words | INT8 | NO | 0 | Word count |
| description | TEXT | NO | '' | Story description |
| fic_created | TIMESTAMPTZ | NO | — | Story creation date on source |
| fic_updated | TIMESTAMPTZ | NO | — | Story update date on source |
| status | TEXT | NO | 'ongoing' | Story status: ongoing, complete, hiatus, cancelled |
| source | TEXT | NO | — | Original URL |
| source_id | INT8 | YES | — | Source site identifier (1=AO3, 2=FF.net, etc.) |
| author_id | INT8 | YES | — | Author ID on source site |
| content_hash | TEXT | YES | — | Hash of story content for change detection |
| extra_meta | JSONB | YES | — | Additional metadata |
| raw_extended_meta | JSONB | YES | — | Raw extended metadata |

### Indexes

```sql
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_source ON fic_info(source);
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX idx_fic_info_search ON fic_info
    USING gin(to_tsvector('english', title || ' ' || author));
```

### Example Queries

```sql
-- Find all complete stories with more than 10,000 words
SELECT * FROM fic_info 
WHERE status = 'complete' AND words > 10000
ORDER BY fic_updated DESC;

-- Full-text search
SELECT * FROM fic_info
WHERE to_tsvector('english', title) @@ plainto_tsquery('english', 'harry potter');

-- Stories by author
SELECT * FROM fic_info WHERE author = 'J.K. Rowling';

-- Recent stories
SELECT * FROM fic_info
WHERE fic_updated > NOW() - INTERVAL '7 days'
ORDER BY fic_updated DESC;
```

---

## Table: export_log

Tracks cached export files.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| url_id | VARCHAR(128) | NO | — | Story identifier |
| version | INT4 | NO | 1 | Export version |
| etype | VARCHAR(32) | NO | — | Export type: epub, html |
| input_hash | VARCHAR(64) | NO | — | MD5 hash of input metadata |
| export_hash | VARCHAR(64) | NO | — | MD5 hash of generated file |
| created | TIMESTAMPTZ | YES | NOW() | Export creation time |

### Primary Key

```sql
PRIMARY KEY (url_id, version, etype, input_hash)
```

### Example Queries

```sql
-- Find cached export
SELECT * FROM export_log
WHERE url_id = 'abc123' AND version = 1 AND etype = 'epub' AND input_hash = 'xyz';

-- Insert new export
INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)
VALUES ('abc123', 1, 'epub', 'input_hash', 'export_hash')
ON CONFLICT (url_id, version, etype, input_hash)
DO UPDATE SET export_hash = EXCLUDED.export_hash, created = NOW();

-- Find all exports for a story
SELECT * FROM export_log WHERE url_id = 'abc123';
```

---

## Table: tags

Canonical tags for stories.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT4 | NO | GENERATED | Tag identifier |
| name | TEXT | NO | — | Tag name (unique) |
| tag_type_id | INT2 | NO | — | Tag type: 1=Fandom, 2=Character, 3=Relationship, 4=Freeform, 5=Warning, 6=Category |
| created | TIMESTAMPTZ | YES | NOW() | Creation time |

### Indexes

```sql
CREATE UNIQUE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_type ON tags(tag_type_id);
```

### Tag Types

| ID | Name | Description |
|----|------|-------------|
| 1 | Fandom | Source material (Harry Potter, Star Wars, etc.) |
| 2 | Character | Individual characters (Harry Potter, Hermione Granger) |
| 3 | Relationship | Romantic pairings (Harry/Hermione) |
| 4 | Freeform | Any other tags (Angst, Humor, Slow Burn) |
| 5 | Warning | Content warnings (Violence, Major Character Death) |
| 6 | Category | Gen, Slash, F/M, Multi, Other |

### Example Queries

```sql
-- Find tag by name
SELECT id, name, tag_type_id FROM tags WHERE name = 'Harry Potter';

-- Find all tags of a type
SELECT * FROM tags WHERE tag_type_id = 1;

-- Create new tag
INSERT INTO tags (name, tag_type_id) VALUES ('New Tag', 4) RETURNING id;

-- Search tags
SELECT * FROM tags WHERE name ILIKE '%harry%';
```

---

## Table: tag_aliases

Alternative names that map to canonical tags.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| alias_name | TEXT | NO | — | Alternative name (primary key) |
| canonical_tag_id | INT4 | NO | — | Reference to canonical tag |
| created | TIMESTAMPTZ | YES | NOW() | Creation time |

### Foreign Key

```sql
FOREIGN KEY (canonical_tag_id) REFERENCES tags(id)
```

### Example Queries

```sql
-- Lookup alias
SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = 'HP';

-- Create alias
INSERT INTO tag_aliases (alias_name, canonical_tag_id) 
VALUES ('HP', 42)
ON CONFLICT DO NOTHING;

-- Find all aliases for a tag
SELECT * FROM tag_aliases WHERE canonical_tag_id = 42;
```

---

## Table: fic_tags

Associates stories with tags.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| url_id | VARCHAR(128) | NO | — | Story identifier |
| tag_id | INT4 | NO | — | Tag identifier |
| added_by_ip | INET | YES | — | IP that added the tag |
| score | INT2 | YES | 0 | Tag score (sum of votes) |
| created | TIMESTAMPTZ | YES | NOW() | Association time |

### Primary Key

```sql
PRIMARY KEY (url_id, tag_id)
```

### Foreign Keys

```sql
FOREIGN KEY (tag_id) REFERENCES tags(id)
```

### Indexes

```sql
CREATE INDEX idx_fic_tags_url_id ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag_id ON fic_tags(tag_id);
```

### Example Queries

```sql
-- Get all tags for a story
SELECT t.id, t.name, t.tag_type_id, ft.score
FROM fic_tags ft
JOIN tags t ON t.id = ft.tag_id
WHERE ft.url_id = 'abc123'
ORDER BY t.tag_type_id, ft.score DESC;

-- Get all stories with a tag
SELECT fi.id, fi.title, fi.author
FROM fic_info fi
JOIN fic_tags ft ON fi.id = ft.url_id
WHERE ft.tag_id = 42 AND ft.score >= 0;

-- Add tag to story
INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
VALUES ('abc123', 42, '192.168.1.1', 0)
ON CONFLICT (url_id, tag_id) DO UPDATE SET added_by_ip = EXCLUDED.added_by_ip;

-- Update tag score
UPDATE fic_tags SET score = score + 1 WHERE url_id = 'abc123' AND tag_id = 42;
```

---

## Table: fic_tag_votes

Tracks votes on tags.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| url_id | VARCHAR(128) | NO | — | Story identifier |
| tag_id | INT4 | NO | — | Tag identifier |
| voter_ip | INET | NO | — | Voter's IP address |
| value | INT2 | NO | — | Vote value: 1=upvote, -1=downvote |
| created_at | TIMESTAMPTZ | YES | NOW() | Vote time |

### Primary Key

```sql
PRIMARY KEY (url_id, tag_id, voter_ip)
```

### Example Queries

```sql
-- Check if user has voted
SELECT value FROM fic_tag_votes
WHERE url_id = 'abc123' AND tag_id = 42 AND voter_ip = '192.168.1.1';

-- Get vote counts
SELECT SUM(CASE WHEN value = 1 THEN 1 ELSE 0 END) as upvotes,
       SUM(CASE WHEN value = -1 THEN 1 ELSE 0 END) as downvotes,
       SUM(value) as score
FROM fic_tag_votes
WHERE url_id = 'abc123' AND tag_id = 42;

-- Upsert vote
INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
VALUES ('abc123', 42, '192.168.1.1', 1)
ON CONFLICT (url_id, tag_id, voter_ip)
DO UPDATE SET value = 1, created_at = NOW();
```

---

## Table: tag_flags

Flags tags for moderation review.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT8 | NO | GENERATED | Flag identifier |
| url_id | VARCHAR(128) | NO | — | Story identifier |
| tag_id | INT4 | NO | — | Tag identifier |
| flagged_by_ip | INET | NO | — | IP that flagged |
| reason | TEXT | YES | — | Reason for flagging |
| resolved | BOOLEAN | YES | FALSE | Whether flag has been resolved |
| created_at | TIMESTAMPTZ | YES | NOW() | Flag time |

### Example Queries

```sql
-- List unresolved flags
SELECT f.id, f.url_id, f.tag_id, t.name, f.reason
FROM tag_flags f
JOIN tags t ON t.id = f.tag_id
WHERE f.resolved = FALSE
ORDER BY f.created_at DESC;

-- Resolve flag
UPDATE tag_flags SET resolved = TRUE WHERE id = 1;

-- Insert flag
INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason)
VALUES ('abc123', 42, '192.168.1.1', 'Incorrect tag')
ON CONFLICT (url_id, tag_id, flagged_by_ip) DO NOTHING;
```

---

## Table: tag_types

Defines tag categories.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT2 | NO | — | Tag type identifier |
| name | TEXT | NO | — | Tag type name (unique) |

### Default Data

```sql
INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

## Table: rec_users

Users processed by the collection worker.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | VARCHAR(256) | NO | — | User identifier |
| source_site | VARCHAR(32) | NO | — | Source site |
| last_collected | TIMESTAMPTZ | YES | — | Last collection time |
| favourite_count | INT4 | YES | 0 | Number of favourites |
| processed | BOOLEAN | YES | FALSE | Whether user has been processed |

### Example Queries

```sql
-- Find users to process
SELECT * FROM rec_users
WHERE processed = FALSE
OR last_collected < NOW() - INTERVAL '24 hours'
ORDER BY last_collected ASC NULLS FIRST
LIMIT 10;

-- Mark user as processed
UPDATE rec_users
SET last_collected = NOW(), favourite_count = 50, processed = TRUE
WHERE id = 'user_123';
```

---

## Table: rec_favourites

User favourites (bookmarks).

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| user_id | VARCHAR(256) | NO | — | User identifier |
| url_id | VARCHAR(128) | NO | — | Story identifier |
| created | TIMESTAMPTZ | YES | NOW() | Favourite time |

### Primary Key

```sql
PRIMARY KEY (user_id, url_id)
```

### Example Queries

```sql
-- Get user's favourites
SELECT fi.id, fi.title, fi.author
FROM rec_favourites rf
JOIN fic_info fi ON fi.id = rf.url_id
WHERE rf.user_id = 'user_123';

-- Store favourite
INSERT INTO rec_favourites (user_id, url_id)
VALUES ('user_123', 'abc123')
ON CONFLICT DO NOTHING;
```

---

## Table: rec_cooccurrence

Co-occurrence counts for recommendations.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| story_a | VARCHAR(128) | NO | — | First story identifier |
| story_b | VARCHAR(128) | NO | — | Second story identifier |
| count | INT4 | NO | 1 | Co-occurrence count |

### Primary Key

```sql
PRIMARY KEY (story_a, story_b)
```

### Example Queries

```sql
-- Get co-occurring stories
SELECT story_b, count
FROM rec_cooccurrence
WHERE story_a = 'abc123'
ORDER BY count DESC
LIMIT 10;

-- Update co-occurrence
INSERT INTO rec_cooccurrence (story_a, story_b, count)
VALUES ('abc123', 'def456', 1)
ON CONFLICT (story_a, story_b)
DO UPDATE SET count = rec_cooccurrence.count + 1;
```

---

## Table: rec_results

Precomputed recommendations.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| url_id | VARCHAR(128) | NO | — | Story identifier |
| recommended_url_id | VARCHAR(128) | NO | — | Recommended story identifier |
| score | REAL | NO | — | Recommendation score |
| created | TIMESTAMPTZ | YES | NOW() | Computation time |

### Primary Key

```sql
PRIMARY KEY (url_id, recommended_url_id)
```

---

## Table: rec_suggestions

Story suggestions for the recommendation engine.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT8 | NO | GENERATED | Suggestion identifier |
| url_id | VARCHAR(128) | NO | — | Story identifier |
| source_url | TEXT | NO | — | Original URL |
| submitted_ip | INET | YES | — | Submitter's IP |
| created | TIMESTAMPTZ | YES | NOW() | Submission time |
| status | VARCHAR(32) | YES | 'pending' | Status: pending, approved, rejected |

---

## Table: rec_votes

Votes on recommendations.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT8 | NO | GENERATED | Vote identifier |
| url_id | VARCHAR(128) | NO | — | Story identifier |
| recommended_url_id | VARCHAR(128) | NO | — | Recommended story identifier |
| voter_ip | INET | NO | — | Voter's IP |
| value | INT2 | NO | — | Vote value: 1=upvote, -1=downvote |
| created | TIMESTAMPTZ | YES | NOW() | Vote time |

### Unique Constraint

```sql
UNIQUE(url_id, recommended_url_id, voter_ip)
```

---

## Table: request_source

Tracks request sources.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT8 | NO | GENERATED | Source identifier |
| created | TIMESTAMPTZ | YES | NOW() | Creation time |
| is_automated | BOOLEAN | YES | FALSE | Whether request is automated |
| route | TEXT | YES | — | API route |
| description | TEXT | YES | — | Source description |

---

## Table: request_log

Logs all API requests.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | INT8 | NO | GENERATED | Log identifier |
| created | TIMESTAMPTZ | YES | NOW() | Request time |
| source_id | INT8 | YES | — | Reference to request_source |
| etype | VARCHAR(32) | NO | — | Request type |
| query | TEXT | NO | — | Request query/URL |
| info_request_ms | INT4 | YES | — | Metadata request time (ms) |
| url_id | VARCHAR(128) | YES | — | Story identifier |
| fic_info | JSONB | YES | — | Story metadata |
| export_ms | INT4 | YES | — | Export generation time (ms) |
| export_file_name | TEXT | YES | — | Generated file name |
| export_file_hash | TEXT | YES | — | Generated file hash |
| url | TEXT | YES | — | Request URL |

### Example Queries

```sql
-- Recent requests
SELECT * FROM request_log ORDER BY created DESC LIMIT 10;

-- Average export time
SELECT AVG(export_ms) FROM request_log WHERE export_ms IS NOT NULL;

-- Requests by hour
SELECT date_trunc('hour', created) as hour, COUNT(*) as count
FROM request_log
GROUP BY hour
ORDER BY hour DESC;
```

---

## Table: fic_blacklist

Blacklisted stories.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| url_id | VARCHAR(128) | NO | — | Story identifier |
| created | TIMESTAMPTZ | YES | NOW() | Blacklist time |
| updated | TIMESTAMPTZ | YES | NOW() | Update time |
| reason | INT4 | NO | 0 | Reason code |

---

## Table: author_blacklist

Blacklisted authors.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| source_id | INT8 | NO | — | Source site identifier |
| author_id | INT8 | NO | — | Author identifier |
| created | TIMESTAMPTZ | YES | NOW() | Blacklist time |
| updated | TIMESTAMPTZ | YES | NOW() | Update time |
| reason | INT4 | NO | 0 | Reason code |

---

## Table: fic_version_bump

Manual cache invalidation.

### Columns

| Column | Type | Nullable | Default | Description |
|--------|------|----------|---------|-------------|
| id | VARCHAR(128) | NO | — | Story identifier |
| value | INT4 | YES | — | Version bump value |

---

## Migration History

| Migration | Description |
|-----------|-------------|
| 001_initial_schema.sql | Core tables: fic_info, export_log, request_source, request_log, blacklists, version bumps |
| 002_recommender.sql | Recommendation engine tables: rec_users, rec_favourites, rec_cooccurrence, rec_results, rec_suggestions, rec_votes |
| 003_tagging.sql | Tag system tables: tags, tag_aliases, fic_tags, fic_tag_votes, tag_flags, tag_types |
| 004_shelves.sql | OPDS shelf tables for user shelf functionality |

---

## Performance Considerations

### Index Strategy

FicHub uses indexes strategically:

1. **Primary keys** — All tables have primary keys for fast lookups
2. **Foreign keys** — Join columns are indexed for efficient joins
3. **Search columns** — Full-text search uses GIN indexes
4. **Filter columns** — Common filter columns (status, source, words) are indexed
5. **Date columns** — Date columns used for sorting are indexed

### Query Optimization

1. **Use EXPLAIN ANALYZE** — Always check query plans for slow queries
2. **Avoid SELECT *** — Only select columns you need
3. **Use LIMIT** — Always limit result sets
4. **Batch operations** — Use bulk inserts instead of individual inserts
5. **Connection pooling** — Use connection pools for database connections

### Monitoring

```sql
-- Check table sizes
SELECT pg_size_pretty(pg_total_relation_size('fic_info'));

-- Check index usage
SELECT indexrelname, idx_scan, idx_tup_read, idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;

-- Check slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```


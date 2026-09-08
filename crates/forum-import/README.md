# forum-import

NodeBB JSON export → FicNexus `forum_*` tables importer.

## Input Schema

The importer reads a JSON file with the following structure:

```json
{
  "users": [
    {
      "uid": 1,
      "username": "alice",
      "email": "alice@example.com",
      "joindate": 1672531200000,
      "banned": 0,
      "reputation": 42,
      "postcount": 128,
      "lastonline": 1672617600000
    }
  ],
  "categories": [
    {
      "cid": 1,
      "name": "General Discussion",
      "description": "Talk about anything",
      "slug": "general-discussion",
      "order": 0,
      "parentCid": 0,
      "disabled": 0,
      "isPrivate": 0
    }
  ],
  "topics": [
    {
      "tid": 100,
      "cid": 1,
      "uid": 1,
      "title": "Welcome to FicNexus",
      "slug": "100/welcome-to-ficnexus",
      "timestamp": 1672531200000,
      "lastposttimestamp": 1672617600000,
      "postcount": 5,
      "viewcount": 128,
      "locked": 0,
      "pinned": 0,
      "deleted": 0,
      "scheduled": 0,
      "titleSlug": "welcome-to-ficnexus"
    }
  ],
  "posts": [
    {
      "pid": 1000,
      "tid": 100,
      "uid": 1,
      "content": "Hello world!",
      "timestamp": 1672531200000,
      "edited": 0,
      "deleted": 0,
      "upvotes": 5,
      "downvotes": 1,
      "bookmarks": 0,
      "anon": 0
    }
  ]
}
```

### Field Mapping

| NodeBB Field | FicNexus Column | Notes |
|---|---|---|
| `users.uid` | `users.id` | Mapped via lookup table |
| `users.username` | `users.username` | Preserved as-is |
| `users.email` | `users.email` | Nullable |
| `users.joindate` | `users.created_at` | Millisecond epoch → timestamptz |
| `users.banned` | `users.is_banned` | 0/1 → false/true |
| `users.reputation` | `users.reputation` | Direct mapping |
| `categories.cid` | `forum_categories.id` | Preserved if possible |
| `categories.name` | `forum_categories.title` | Direct mapping |
| `categories.slug` | `forum_categories.slug` | Direct mapping |
| `topics.tid` | `forum_topics.id` | Preserved if possible |
| `topics.cid` | `forum_topics.category_id` | FK to forum_categories |
| `topics.uid` | `forum_topics.author_id` | FK via users lookup |
| `topics.title` | `forum_topics.title` | Direct mapping |
| `topics.timestamp` | `forum_topics.created_at` | Millisecond epoch → timestamptz |
| `posts.pid` | `forum_posts.id` | Preserved if possible |
| `posts.tid` | `forum_posts.topic_id` | FK to forum_topics |
| `posts.uid` | `forum_posts.author_id` | FK via users lookup |
| `posts.content` | `forum_posts.body` | Direct mapping |
| `posts.timestamp` | `forum_posts.created_at` | Millisecond epoch → timestamptz |
| `posts.upvotes - posts.downvotes` | `forum_posts.score` | Net score |

### Notifications

Import creates a single `forum_import` notification per user:

```sql
INSERT INTO notifications (user_id, notification_type, title, body, link, is_read, created_at)
VALUES ($1, 'forum_import', 'Forum Import Complete', 'Your NodeBB content has been imported.', '/forum', true, NOW())
```

### Rules

- **NEVER** write to `forum_notifications` (if it exists) — only `notifications`
- User IDs are mapped: NodeBB `uid` → FicNexus `users.id` via a lookup table
- Categories, topics, posts preserve original IDs when possible
- Duplicate detection: skip if ID already exists in target table
- Timestamps: NodeBB stores millisecond epoch; convert to PostgreSQL `timestamptz`

## Usage

```bash
# Import from NodeBB JSON export
forum-import --database-url postgres://user:pass@localhost/db --input nodebb-export.json

# Dry run (validate only, no writes)
forum-import --database-url postgres://user:pass@localhost/db --input nodebb-export.json --dry-run

# Verbose logging
RUST_LOG=info forum-import --database-url postgres://user:pass@localhost/db --input nodebb-export.json
```

## Testing

```bash
cargo test -p forum-import
```

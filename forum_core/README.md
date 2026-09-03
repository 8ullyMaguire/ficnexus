# forum-core

A generic, embeddable forum engine core for self-hosted community
platforms: **categories → topics → posts**, with follows,
per-user read state and bans — as a clean async Rust domain model with a
feature-gated Postgres store. The optional `Vote<A>` model is available
for embedders that want ±1 scoring; the reference embedding (FicHub)
deliberately does **not** surface votes — moderation points are its only
score system.

Mirrors the `fanfic-scrapers` playbook: a standalone, publishable crate
with **zero application types**. Embed it in any app; the embedding layer
provides auth, moderation hooks, notifications and content policy.

## Features

- **Generic domain model** — `Category<A>`, `Topic<A>`, `Post<A>`,
  `Vote<A>`, `Follow<A>`, `ReadState<A>`, `Ban<A>` generic over the actor
  (user) id type (`i32` in FicHub; `i64` elsewhere).
- **Opaque payloads** — topic/post `payload` is `serde_json::Value`
  (JSONB): the crate never interprets it. The embedding app renders link
  cards, embeds, etc. from it.
- **`Store<A>` trait** — full async surface (category/topic/post CRUD with
  cursor pagination, votes, follows, read state, bans), with a sqlx/Postgres
  implementation behind the `postgres` feature (default). Compile the crate
  without the feature and the pure domain has **zero database dependency**.
- **`ModerationHook<A>`** — async pre-write content check and post-action
  modlog hook; pure status-transition logic (open → locked/pinned/archived,
  no posts into locked) with no DB.
- **FTS helper** — Postgres `tsvector`/`ts_rank`/`ts_headline` SQL fragment
  builder (pure, parameterized — user input is never interpolated).
- **Self-contained migrations** — `migrations/` inside the crate
  (`sqlx::migrate!()` embedded), so any embedding app gets the schema with
  the dependency.

## Crate layout

```
forum_core/
├── Cargo.toml
├── README.md
├── migrations/            # sqlx migrations (embedded via sqlx::migrate!())
└── src/
    ├── lib.rs             # crate docs + re-exports
    ├── model.rs           # Category, Topic, Post, Vote, Follow, ReadState, Ban, Status
    ├── store.rs           # Store<A> trait, Cursor/Page, StoreError
    ├── store/pg.rs        # PgStore (sqlx/Postgres, behind `postgres` feature)
    ├── moderation.rs      # ModerationHook<A>, ModAction<A>, status state machine
    └── search.rs          # FTS query builder (PG dialect)
```

## Usage

```toml
[dependencies]
forum-core = { version = "0.1", features = ["postgres"] }
```

```rust
use forum_core::{Store, model::Status};

let store = forum_core::store::pg::PgStore::connect("postgres://...").await?;
store.run_migrations().await?; // applies crate migrations (idempotent)

let cat = store.create_category("general", "General", "Anything goes", 0, false).await?;
let topic = store.create_topic(cat.id, 42, "Hello", "World", serde_json::Value::Null).await?;
let reply = store.create_post(topic.id, 7, "First!", serde_json::Value::Null, None).await?;

// Moderation hooks are app-side:
#[async_trait::async_trait]
impl forum_core::moderation::ModerationHook<i32> for MyHook {
    async fn check_post(&self, _actor: &i32, body: &str) -> Result<(), forum_core::moderation::ModReject> {
        if body.contains("spam") { Err(forum_core::moderation::ModReject::new("spam")) } else { Ok(()) }
    }
    async fn on_mod_action(&self, action: forum_core::moderation::ModAction<i32>) { /* modlog */ }
}
```

## Integration notes (FicHub)

The crate is generic; the embedding app adds its user-table foreign keys
(`author_id`/`user_id`/`banned_by` → `users(id)`) in its own migration —
the crate's migration deliberately leaves them out so it applies
standalone. The `user_reports` CHECK extension at the end of the migration
is guarded with `IF EXISTS` (no-op on a fresh database; FicHub's existing
`user_reports` gets the extended constraint).

## Running tests

Pure-domain unit tests only (no database required):

```sh
cargo test            # all features (default: postgres)
cargo test --no-default-features   # pure domain without sqlx
cargo doc --no-deps
```

DB-gated integration tests (scratch Postgres) live in the embedding app's
suite, mirroring the repo's existing `tests/*_api.rs` pattern.

## License

AGPL-3.0-or-later

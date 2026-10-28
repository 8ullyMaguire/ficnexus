# Preserved forum-core migrations

These SQL files created the forum tables (`forum_topics`, `forum_posts`,
`forum_messages`, …) that still exist in deployed databases. ficnexus no
longer has a forum — `crates/forum-core` was deleted and the forum belongs to
darkhold — but migrations are a historical record of how the schema got its
shape, so they are kept here rather than deleted.

Nothing applies them any more: the application's own migrations live in
`/migrations` and are the only set sqlx runs at start-up. No table was
dropped; see `docs/EXPORTER-SCOPE.md`.

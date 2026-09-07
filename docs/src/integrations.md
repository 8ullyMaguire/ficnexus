# Integrations

FicHub is more than a website — it ships companion tools so you can browse,
search and download stories from where you already hang out.

## Discord / Telegram / Matrix bot (fanfic-archivist)

**fanfic-archivist** is a companion chat bot for FicHub. It is a *thin client*
over the FicHub REST API — it never touches the archive database directly, and
it shares FicHub's Redis only for a transient pagination cache so `!next` /
`!prev` don't hammer the DB.

### Where to use it

- **Discord channel** — <https://discord.com/channels/1390467506577346782/1538281679402180718> in [this server](https://discord.gg/AmE93m23dW)
- **Add the Discord bot to your own server** — <https://discord.com/oauth2/authorize?client_id=1427681624459317278&permissions=116800&scope=bot+applications.commands>
- **Telegram group** — <https://t.me/+khDXd6IKQYRjZmQ0>
- **Matrix room** — <https://matrix.to/#/#archivist-bot:matrix.org>
- **Code** — [`fanfic-archivist/`](../fanfic-archivist/) crate, standalone (not in the FicHub workspace)

### What it does

Socrates-style recommendations right in chat:

- **Recommendations from your reading history** — `!recs`, `!fresh`, `!gems`, `!roll`, with pagination (`!next` / `!prev` / `!page 3`)
- **Cross-platform taste** — AO3, fanfiction.net and RoyalRoad are unified into one works model, so your history on one site feeds recommendations across all of them. No "500 favs limit" wall.
- **Slash commands** — `/search`, `/ask` (ask a question about a fic's content in natural language), `/quote`, `/download` (EPUB/MOBI/PDF straight to Discord)
- **Library commands** — bookmark, rate, kudos, block/unblock, updates on followed fics
- **Community** — fic requests board, roadmap voting, links back to your web profile

### Auth / linking

- `/link <username> <password>` — exchanges credentials **once** via `POST /api/auth/login`, stores only the JWT in Redis (`archivist:token:<discord_id>`, TTL 30d, refreshed on use). Passwords are never persisted.
- `/unlink`, `/whoami`.

## Lemmy / ActivityPub adapter (fanfic_archivist)

The same bot also federates as `@fanfic_archivist@ficnexus.polarisocial.xyz`
— a first-class ActivityPub actor served by ficnexus itself
(`src/activitypub/`), so Lemmy instances (e.g. reddthat.com) and the wider
Fediverse can follow it, mention it, and receive its posts.

### Configuration

```env
ACTIVITYPUB_ENABLED=true
ACTIVITYPUB_DOMAIN=ficnexus.polarisocial.xyz
ACTIVITYPUB_ALLOW_LOOPBACK=false   # true only for local testing
ACTIVITYPUB_BOT_USERNAME=fanfic_archivist
```

(The `BOT_USERNAME` is separate from `NODE_NAME` on purpose — node names
feed agent/exporter pipelines and must not be repurposed.) Keys are
auto-generated into the `ap_keys` table (RSA-2048) on first run.

### Endpoints (served by ficnexus)

| Endpoint | Purpose |
|---|---|
| `GET /actor` | Actor document (`preferredUsername: fanfic_archivist`) |
| `POST /actor/inbox` | Receives `Create` / `Like` / `Follow` / `Announce` (HTTP-signature required — unsigned POSTs get 401) |
| `GET /api/activitypub/status` | `{"enabled":true,"domain":…}` health check |

Inbound activities validate HTTP signatures + `Digest` headers
(`src/activitypub/signatures.rs`) and land in `ap_inbox_log`.

### Mention monitoring (Lemmy communities)

Point the bot at a Lemmy community (e.g. the fanfiction community on
reddthat.com) and it watches its inbox for mentions. The Python reference
bots under `~/code/python/lemmy/` show the complementary half of the
pattern: `Reddit-Lemmy-Repost-bot/bot.py` (Reddit→Lemmy reposting with PRAW)
and `lemmy-rss-pybot/` (RSS→Lemmy with keyword filtering). The Rust side
posts back out via `src/activitypub/outbox.rs`, signing with the actor key.

### Operator notes

- Credentials and instance URLs live in `/opt/ficnexus/.env` on the server —
  never in the repo.
- Verify live: `curl https://ficnexus.polarisocial.xyz/actor` should return
  the actor JSON; `curl -X POST …/actor/inbox` without a signature should
  return 401 (not 404).

### Honest limitation

The recommendation engine learns from what people like on the archive. Until a
few users like a few fics, "personalized" recs won't be proper — the more it's
used, the better it gets.

### Announcement

See the forum announcement: [The FicHub Discord bot — Socrates-style recommendations from a self-hosted archive](https://fichub.polarisocial.xyz/forum/board/the-fichub-discord-bot-socrates-style-recommendations-from-a-2.2)

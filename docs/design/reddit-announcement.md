# r/rust — Announcing fanfic-archivist: a multi-platform FicHub bot in Rust

**Title:** I built a multi-platform fanfiction bot in Rust — one platform-neutral core, six chat adapters, a CLI, and a TUI

**Body:**

I run a self-hosted fanfiction archive ([FicHub](https://fichub.polarisocial.xyz/)) and I
wanted a companion bot that works everywhere — Discord, Matrix, Telegram,
Slack, IRC, Mastodon/Bluesky/Piefed — plus a terminal interface. The
interesting engineering decision: instead of writing N bots, I wrote
**one platform-neutral core** and made every adapter a thin translation
layer. (Note: this is the self-hosted FicHub rewrite — the public
[fichub.net](https://fichub.polarisocial.xyz/) is a separate legacy Python codebase with
a different API; the bot targets the Rust API.)

**Architecture**

```
Discord ─┐
Matrix ──┤
Telegram─┼──► archivist-core ──► FicHub REST API
Slack ───┤        │                  + Redis (shared)
IRC ─────┤   do_* → PlatformMessage
Fediverse┘
CLI/TUI ─┘
```

- `archivist-core` contains **zero platform SDKs** — no discord-rs, no
  teloxide, nothing. Every command is a `do_*` function returning a
  platform-neutral `PlatformMessage` IR (`Text`/`Rich`/`File`/`Ephemeral`).
  An adapter's only job: translate platform events → core calls, and render
  the IR back natively.
- New commands land once in the core and every platform gets them for free.
- Shared Redis for pagination cache, a token store, and cross-platform rate
  limiting.

**The CLI + TUI** ([docs](https://opencommit.eu/MagicZhang/fanfic-archivist/src/branch/main/docs/CLI-TUI.md))

- Batch download parity with the Python CLI: `--infile`, `--format
  epub,mobi,pdf,html`, `--out-dir`, `--force`, `--changelog`, `--config-init`.
- Forum subcommands: `forum categories | topics | show | new | reply |
  search | follow | read | mod | metamod`.
- A [ratatui](https://github.com/ratatui/ratatui) three-pane TUI
  (search/forum browse on the left, detail on the right, input/status at
  the bottom) with an async worker over `tokio::sync::mpsc` — the state
  machine is testable without a terminal.
- An interactive REPL with rustyline history: bare URLs, command words,
  `next`/`prev` pagination, free-form text → intent classification.

**Testing** — 256 tests across 8 crates, all offline (no Redis/FicHub/Ollama
needed to run the suite).

**Published crates** — `fanfic-archivist` (Discord), `archivist-core`
(platform-neutral core), `forum-core`, `fanfic-scrapers` on crates.io.
Source: [Forgejo](https://opencommit.eu/MagicZhang/fanfic-archivist),
AGPL-3.0-or-later.

The core is the piece I think is reusable beyond fanfiction — if you're
building a multi-platform bot, the "core + thin adapters" split is worth
stealing. Happy to answer questions about the adapter pattern, the TUI
architecture, or the Redis-backed pagination.

---

# r/cli — Announcing fichub-cli: a terminal client + TUI for a fanfiction archive

**Title:** I wrote a terminal client + TUI for a fanfiction archive — batch download, forum browsing, REPL, and a ratatui panel UI

**Body:**

For the past few months I've been building [FicHub](https://fichub.polarisocial.xyz/), a
self-hosted fanfiction archive. The terminal client (`fichub-cli`, Rust)
has grown into something I genuinely enjoy using, and I think it's a good
example of "CLI-first" for a content-heavy service:

**What it does**

- **Batch download** — one URL, comma-separated URLs, or a list file:
  ```
  fichub-cli download urls.txt --format epub,mobi,pdf --out-dir ./fics --changelog
  ```
  Output files are `{title} by {author}.{ext}` with a filename sanitizer,
  `--force` to overwrite, a deduplicated `output.log` changelog, and a
  3-attempt retry loop with backoff on 429/5xx (mirrors the Python CLI's
  retry semantics).
- **Forum** — browse categories/topics, read threads, create topics, reply
  (with `$EDITOR` for long posts), follow, mark-read, search:
  ```
  fichub-cli forum topics --category general
  fichub-cli forum new --title "..." --category general --editor
  ```
- **TUI** — a [ratatui](https://github.com/ratatui/ratatui) three-pane
  interface: left pane browses search results or forum threads, right pane
  shows fic metadata / post bodies, bottom has a REPL-grammar input line.
  Keybindings: `j/k` move, `Tab` switch pane, `/` focus input, `d`
  download, `b` bookmark, `f` follow, `R` reply, `r` refresh.
- **REPL** — rustyline history, bare fanfic URLs are auto-detected (URL
  wins over command words), `next`/`prev` pagination, free-form text
  classified as intent.

**Config** — `~/.config/fichub/config.toml` (generated via
`fichub-cli config-init`), token resolution chain: `--token` → env →
config → Redis token store.

Source: [Forgejo](https://opencommit.eu/MagicZhang/fanfic-archivist) ·
[docs](https://opencommit.eu/MagicZhang/fanfic-archivist/src/branch/main/docs/CLI-TUI.md) ·
AGPL-3.0-or-later.

If you've built a CLI/TUI for a content service, I'd love to hear what UX
patterns worked for you — especially around long-form reading in the
terminal.

---

# r/tui — Announcing a ratatui three-pane TUI for a fanfiction archive

**Title:** I built a ratatui TUI for browsing and reading fanfiction — three panes, async worker, testable state machine

**Body:**

I've been building [FicHub](https://fichub.polarisocial.xyz/), a self-hosted fanfiction
archive, and recently added a terminal UI on top of its CLI
(`fichub-cli tui`, Rust + [ratatui](https://github.com/ratatui/ratatui) +
crossterm).

**Layout** (mutt/neomutt-inspired):

```
┌──────────────────────┬──────────────────────────────┐
│ LEFT: browse         │ RIGHT: content               │
│  • Search | Forum    │  • fic metadata + chapter    │
│  • results/topics    │  • forum thread (posts)      │
│  • filtering         │  • action hints at bottom    │
├──────────────────────┴──────────────────────────────┤
│ BOTTOM: status + input (REPL grammar: /cmds, URLs)  │
└─────────────────────────────────────────────────────┘
```

**Design choices I'm happy with**

- **Async worker over `tokio::sync::mpsc`** — the UI thread owns
  crossterm/ratatui, a background task owns the HTTP client and sends
  `TuiMsg::Results(...)` / `TuiMsg::Thread(...)` / `TuiMsg::Error(...)`
  messages. No blocking in the draw loop.
- **Testable without a terminal** — the `AppState` enum + transitions are
  pure Rust, so 13 state-machine tests run without crossterm. Rendering is
  separate from state.
- **Debounced search** (~300 ms) — type-ahead search like a live web UI.
- **REPL-grammar input box** — bare URL → metadata, `/`-prefixed commands,
  forum words (`forum cats`, `forum show 3`), so the TUI and the CLI share
  one command language.

**Keybindings:** `q`/`Ctrl-C` quit · `Esc` back · `Tab` switch pane ·
`↑↓←→`/`hjkl` move · `/` focus input · `r` refresh · `d` download · `b`
bookmark · `f` follow · `R` reply in thread ($EDITOR).

Source: [Forgejo](https://opencommit.eu/MagicZhang/fanfic-archivist) ·
[docs](https://opencommit.eu/MagicZhang/fanfic-archivist/src/branch/main/docs/CLI-TUI.md) ·
AGPL-3.0-or-later.

The TUI is one adapter in a multi-platform bot — Discord, Matrix, Telegram,
Slack, IRC and fediverse adapters all share the same core, so `forum`,
`download`, `search`, etc. behave identically everywhere. Happy to talk
about the worker/channel pattern or anything else.
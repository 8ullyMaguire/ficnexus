# FicHub — Pi Agent Context

## Project
FicHub is a Rust + Svelte web application for managing fanfiction libraries.
- **Stack:** Rust (Axum), SvelteKit, PostgreSQL, Redis, Ollama
- **Deploy:** ThinkCentre (fichub.service, ebook-convert live for MOBI/PDF/AZW3)
- **Dev:** gamingpc (Ryzen 7 5700G), worktrees at /media/alvaro/code-worktrees

## Build & Test
```bash
cargo build --release
cargo test
cargo sqlx prepare --check
justfile recipes
```

## Code Style
- Use `pathlib.Path` not `os.path` in Python
- Prefer `anyhow` for error handling in Rust
- No `unwrap()` in production code — use `?` or `.expect("reason")`
- Commit messages: conventional commits (feat/fix/chore/docs)

## Key Files
- `docs/brainstorm-02-roadmap-status.md` — canonical roadmap
- `docs/USER-ACTIONS.md` — user-action items runbook
- `justfile` — build recipes
- `src/` — Rust source
- `frontend/` — SvelteKit frontend

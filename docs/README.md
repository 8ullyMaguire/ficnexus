# FicHub Docs

> This folder was reorganized on 2026-09-09. Specs moved to `../specs/`;
> design docs, plans, sessions, posts, and reference docs were bucketed into
> subdirectories. See `REPOSITORY-LAYOUT.md` for the full migration map.

## Structure

```
docs/
├── README.md              ← you are here
├── REPOSITORY-LAYOUT.md   ← migration map + what was moved where
├── book.toml              ← mdbook config
├── build.sh               ← mdbook build + copy to frontend/static/docs/
├── build_docs_map.py      ← sitemap generator for docs site
├── fichub-header.js       ← mdbook additional JS (header injection)
├── src/                   ← mdbook source (SUMMARY.md, *.md chapters)
├── theme/                 ← mdbook theme (CSS)
├── design/                ← design docs, brainstorms, audits, UX riffs
│   ├── ideas/             ← brainstorm idea notes
│   └── ...
├── plans/                 ← time-stamped implementation plans
│   └── archive/           ← fully-executed or superseded plans
├── sessions/              ← session summaries (local only, gitignored)
├── posts/                 ← long-form posts (announcements, feature spotlights)
└── reference/             ← operator/contributor reference docs
```

## What lives elsewhere

- **Specs** — `../specs/` (top-level, one folder per feature with `spec.md`, `plan.md`, `data-model.md`, `quickstart.md`, `research.md`, `tasks.md`, `contracts/`, `checklists/`)
- **mdbook output** — gitignored (`docs/book/`, `docs/FicHub_Docs.epub`)
- **Books** (LLM-generated tutorials) — gitignored (`books/`)
- **Deployment infra** — `../infra/docker/` + `../infra/systemd/`

## Appendix — Quick Reference

### The Commands You'll Type Daily

**Backend:**

```bash
cargo check                        # fast type-check
cargo test --lib <module>          # unit tests for a module
cargo test --test <suite> -- --include-ignored --test-threads=1   # DB-gated
cargo build --release --bin fichub # release binary
cargo run                          # dev server on :8000
```

**Frontend:**

```bash
cd frontend
npm run dev                        # SPA on :5173 (proxies /api)
npm run build                      # static build → build/
npx vitest run src/routes/<x>/page.test.ts   # one page test
```

**Database:**

```bash
psql -h localhost -U fichub -d fichub        # dev DB
sudo -u postgres psql -d fichub              # on the deploy box
```

**Deploy (deploy box):**

```bash
sudo systemctl restart fichub.service
journalctl -u fichub.service -n 100
curl https://fichub.polarisocial.xyz/api/health
```

**Git (with the NFS quirk):**

```bash
# before commit, when refs dangle on NFS:
export GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-<name>
git add ... && git commit ...
cp -rn /home/alvaro/.cache/git-objects-<name>/* .git/objects/ 2>/dev/null
unset GIT_OBJECT_DIRECTORY
git cat-file -t <sha>              # ALWAYS verify
```

### Fast Facts

| Fact | Value |
|------|-------|
| Backend | Rust + Axum 0.8, one binary |
| Frontend | SvelteKit 5, static adapter |
| Database | PostgreSQL 16 + pgvector |
| Cache/queues | Redis |
| LLM | Ollama, `lfm2.5:8b` default on deploy |
| Migrations | 34 (1-34), applied at boot |
| Roles | 0 user, 1 curator, 5 mod, 10 admin |
| Body cache | `/public/literature/fichub/bodies` (sharded JSON) |
| Rec mode | `legacy` live; `pluggable` with shadow mode |
| Self-healing | M1 telemetry live; agent loop OFF |
| Modlog | migration 034, public to logged-in users |
| Analytics | migration 033, zero-PII (X-Client-ID) |

### The File You Should Print

If you remember ONE file, remember **`src/server.rs`** — specifically
`build_router()`. It's the map of the whole product. Every route, grouped
by area. When you're lost, open it and find your feature.

Second place: **`src/config.rs`** — every knob. Third:
**`src/db/queries.rs`** — every query. Together those three files
are 80% of "where does X live."

### A Final Note on Asking for Help

When you get stuck, the fastest path to an answer in this codebase:

1. `grep` for the term in `src/` (route names, function names).
2. Read the route file for the area.
3. Check the docs (ROADMAP, SPECIFICATION, skill references).
4. If still stuck, ask with specifics: "In `src/routes/requests.rs`, the
   candidates endpoint returns empty for a request without a seed work —
   is that expected?"

Specific questions get fast answers. The codebase is well-organized;
the answer to most questions is a grep away.

---

*This onboarding guide was generated for the FicHub codebase on
2026-08-11. The repo evolves; if a path or behavior here has changed,
trust the code and update the book.*

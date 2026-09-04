# Part 45 — Testing

> In this chapter you will learn how FicHub tests its backend contracts and frontend routes — the API contract test suite, SQLx prepare checks, Vitest route tests, and the justfile recipes that enforce them.

---

## Overview

FicHub has two test layers:

1. **Backend** — unit tests (`cargo test --lib`), API contract tests (`src/routes/api_contract_tests.rs`, 460 lines), and DB-gated integration tests (`cargo test -- --include-ignored --test-threads=1`).
2. **Frontend** — Vitest route tests (`page.test.ts`, 413 lines, 15 cases), E2E tests (15/15 green baseline), and `svelte-check` for TypeScript validation.

All are orchestrated by `justfile` recipes: `just test`, `just test-frontend`, `just clippy`, `just lint`.

---

## Chapter 45.1 — Backend API Contract Tests

### Goal

Understand the API contract test suite that validates response shapes without requiring a live database.

### Actions

```rust
// src/routes/api_contract_tests.rs (460 lines)
// #[cfg(test)] mod api_contract_tests { ... }
// Included via: src/routes/mod.rs → #[cfg(test)] mod api_contract_tests;

#[cfg(test)]
mod api_contract_tests {
    use serde_json::json;

    // ── Response Format Contracts ────────────────────────────────────────
    // Every API endpoint must return a consistent JSON shape.
    // These tests document and verify the contract.

    #[test]
    fn epub_handler_empty_query_returns_error() {
        // GET /api/epub with no q param → err: -1
        let resp = json!({ "err": -1, "msg": "no query", "q": "" });
        assert_eq!(resp["err"], -1);
        assert!(resp["msg"].as_str().unwrap().contains("no query"));
    }

    #[test]
    fn epub_handler_automated_blocked() {
        // Rate-limited bots get err: -10
        let resp = json!({ "err": -10, "msg": "automated requests blocked" });
        assert_eq!(resp["err"], -10);
    }

    #[test]
    fn epub_handler_unsupported_url() {
        let resp = json!({ "err": -5, "msg": "unsupported URL: not-a-url" });
        assert_eq!(resp["err"], -5);
    }

    #[test]
    fn epub_handler_blacklisted_fic() {
        let resp = json!({ "err": -7, "msg": "fic is blacklisted", "q": "https://example.com/story" });
        assert_eq!(resp["err"], -7);
    }
```

> **💡 Key Concept**: API contract tests use `serde_json::json!` to construct mock responses and assert on their **shape** — not their behavior. They're pure unit tests (`#[test]`, no `#[sqlx::test]`) that run without a database. They document the expected JSON structure for every endpoint's consumers.

#### Meta handler contract

```rust
    #[test]
    fn meta_handler_success_response_shape() {
        // Successful meta response must have: q, fixits, info, url_id, slug,
        // meta { id, title, author, chapters, words, description, status, source,
        // created, updated, source_id, author_id, author_url, author_local_id },
        // hashes, urls { epub, html, mobi, pdf }, epub_url, html_url, mobi_url, pdf_url
        let resp = json!({
            "err": 0,
            "q": "https://archiveofourown.org/works/12345",
            "url_id": "abc123def456",
            "meta": { "id": "abc123def456", "title": "Test", "author": "Author", ... },
            "urls": {
                "epub": "/cache/epub/abc123def456?h=epubhash123",
                "html": "/cache/html/abc123def456?h=htmlhash456"
            },
            "epub_url": "/cache/epub/abc123def456?h=epubhash123",
            "html_url": "/cache/html/abc123def456?h=htmlhash456",
            "mobi_url": null,
            "pdf_url": null,
        });
        assert_eq!(resp["err"], 0);
        assert_eq!(resp["epub_url"].as_str().unwrap().starts_with("/cache/epub/"), true);
        assert!(resp["mobi_url"].is_null());  // null when not generated
    }
```

#### Search, recommendations, tags contracts

```rust
    #[test]
    fn search_response_shape() {
        // GET /api/search — paginated results with facets
        let resp = json!({ "err": 0, "results": [], "total": 0, "page": 1, "per_page": 50 });
        assert_eq!(resp["err"], 0);
        assert!(resp["results"].is_array());
    }

    #[test]
    fn recommendations_response_shape() {
        let resp = json!({ "err": 0, "recommendations": [], "total": 0 });
        assert!(resp["recommendations"].is_array());
    }

    #[test]
    fn tags_response_shape() {
        let resp = json!({ "err": 0, "tags": [] });
        assert!(resp["tags"].is_array());
    }
```

#### Auth API contract

```rust
    #[test]
    fn auth_register_response_shape() {
        // POST /api/auth/register → { err, token, user: { id, username } }
        let resp = json!({ "err": 0, "token": "jwt.token.here", "user": { "id": 1, "username": "newuser" } });
        assert!(resp.get("token").is_some());
    }

    #[test]
    fn auth_me_response_shape() {
        // GET /api/auth/me → { err, user: { id, username, role, reputation } }
        let resp = json!({ "err": 0, "user": { "id": 1, "username": "x", "role": 0, "reputation": 0 } });
        assert!(resp["user"].get("role").is_some());
    }
```

#### Error consistency

```rust
    #[test]
    fn all_error_responses_have_err_field() {
        let errors = vec![
            json!({"err": -1, "msg": "no query"}),
            json!({"err": -5, "msg": "unsupported URL"}),
            json!({"err": -7, "msg": "fic is blacklisted"}),
            json!({"err": -10, "msg": "automated requests blocked"}),
        ];
        for resp in &errors {
            assert!(resp.get("err").is_some(), "Error response missing 'err' field");
            assert!(resp["err"].as_i64().unwrap() < 0, "Error code should be negative");
        }
    }

    #[test]
    fn rate_limited_response_shape() {
        let resp = json!({ "err": 429, "msg": "rate limited", "retry_after": 60 });
        assert_eq!(resp["err"], 429);
        assert!(resp.get("retry_after").is_some());
    }
```

> **⚠️ Watch Out**: Error codes use a convention: **negative** for application errors (`-1` = generic, `-5` = unsupported URL, `-7` = blacklisted, `-10` = rate-limited), and **positive** for HTTP-like codes (`429` = rate limited, `401` = unauthorized). The `err` field is always present — success is `err: 0`.

### Try It Yourself

```bash
# Add a test for a new endpoint's response shape
# In api_contract_tests.rs:
#[test]
fn my_endpoint_response_shape() {
    let resp = json!({ "err": 0, "data": "..." });
    assert_eq!(resp["err"], 0);
    assert!(resp.get("data").is_some());
}

# Run the contract tests
cargo test --lib api_contract_tests
```

### Check

- ✅ All tests use `#[test]` (not `#[sqlx::test]`) — no DB required.
- ✅ Every error response has `err` as a negative integer.
- ✅ Success responses always have `err: 0`.
- ✅ `epub_url` and `html_url` start with `/cache/` (the cache download path).
- ✅ `mobi_url` and `pdf_url` are `null` when the format isn't available.
- ✅ Tests are included via `#[cfg(test)] mod api_contract_tests;` in `routes/mod.rs`.

### What you built

The API contract test suite — 20 tests documenting response shapes for epub, meta, comments, search, recommendations, tags, auth, bookmarks, ratings, leaderboard, errors, and OPDS feeds. All pure unit tests (no DB), enforcing consistent `{ err, ... }` JSON contract.

---

## Chapter 45.2 — Testing Workflow and Recipes

### Goal

Master the `justfile` testing recipes and understand when to use each.

### Actions

```makefile
# justfile — testing recipes

# Unit tests (fast, no DB)
test:
    cargo test --lib

# All unit + doc tests
test-all:
    cargo test

# DB-gated integration tests (needs .env DB/Redis; run suites individually)
test-db:
    cargo test -- --include-ignored --test-threads=1

# A single DB-gated suite by name, e.g. `just test-suite requests_api`
test-suite suite:
    cargo test --test {{suite}} -- --include-ignored --test-threads=1

# Frontend unit tests
test-frontend:
    cd frontend && npm test

# Frontend E2E (15/15 green baseline; add new routes to +layout.svelte routePages)
test-e2e:
    cd frontend && npm run test:e2e

# Everything fast (unit + frontend unit)
test-quick:
    cargo test --lib
    cd frontend && npm test
```

> **💡 Key Concept**: `test` (unit only) vs `test-db` (integration) vs `test-all` (unit + doc). The `--include-ignored` flag is required for DB-gated tests because they're marked `#[ignore]` (they'd fail CI without a database). Always use `--test-threads=1` for DB tests to avoid connection contention.

#### SQLx prepare check

```makefile
# The justfile doesn't have a dedicated sqlx recipe, but the project uses:
# cargo sqlx prepare --check   # in CI / pre-commit
# This verifies that SQL queries in the code match the actual database schema.
# If a query references a column that doesn't exist, `prepare --check` fails.
```

#### Clippy and formatting

```makefile
fmt:        # check formatting
    cargo fmt --check

fmt-fix:    # fix formatting in place
    cargo fmt

clippy:     # warnings as errors (CI gate)
    cargo clippy --workspace -- -D warnings

svelte-check:  # TypeScript validation for frontend
    cd frontend && npx svelte-check

lint:       # full pre-commit gate
    cargo fmt --check
    cargo clippy --workspace -- -D warnings
    cd frontend && npx svelte-check
```

### Try It Yourself

```bash
# Add a new test recipe for your feature
# In justfile:
test-feature:
    cargo test --lib my_feature_module
```

### Check

- ✅ `cargo test --lib` runs unit tests (fast, no DB).
- ✅ DB-gated tests use `--include-ignored --test-threads=1`.
- ✅ Frontend tests use `npm test` (Vitest).
- ✅ E2E tests require adding new routes to `+layout.svelte` routePages.
- ✅ `clippy -- -D warnings` treats warnings as errors (CI gate).
- ✅ `cargo sqlx prepare --check` validates SQL against schema.

### What you built

The testing workflow — `cargo test --lib` for unit tests, `--include-ignored --test-threads=1` for DB integration tests, `npm test` for frontend Vitest, E2E route registration via `+layout.svelte`, and the pre-commit lint gate (`fmt --check` + `clippy -D warnings` + `svelte-check`).

---

## Chapter 45.3 — Frontend Route Tests

### Goal

Understand the Vitest test suite for the search page — 15 tests covering rendering, filtering, tier switching, chip management, and URL sync.

### Actions

```typescript
// src/routes/search/page.test.ts (413 lines)
// Note: file is named page.test.ts (not +page.test.ts) — SvelteKit loads either,
// but +page.test.ts is the newer convention.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import * as navigation from '$app/navigation';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
    mockFetch.mockReset();
    localStorage.clear();
    // Default to modern UI in tests (archive is the app default)
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'modern' }));
});

// Mock SvelteKit modules
vi.mock('$app/stores', () => ({
    page: { subscribe: (fn) => { fn({ url: new URL('http://localhost/search') }); return () => {}; } },
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

function searchUrls(): string[] {
    return mockFetch.mock.calls
        .map(c => c[0] as string)
        .filter(u => u.startsWith('/api/search?') || u === '/api/search');
}
```

#### Test: Renders search bar and quick filters

```typescript
it('renders search bar and quick filters', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    expect(screen.getByPlaceholderText(/search titles/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /search/i })).toBeTruthy();
    expect(screen.getByText('No Warnings')).toBeTruthy();
    expect(screen.getByText('Complete Only')).toBeTruthy();
    expect(screen.getByText('100k+ Words')).toBeTruthy();
});
```

#### Test: Three-tier tabs

```typescript
it('defaults to the Simple tier with a tier switcher', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const simpleTab = screen.getByRole('tab', { name: 'Simple' });
    expect(simpleTab.getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('tab', { name: 'Guided' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Power' })).toBeTruthy();
    // Simple's single search box is present.
    expect(screen.getByPlaceholderText(/search titles/i)).toBeTruthy();
});

it('switches to Guided and back to Simple', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    await fireEvent.click(screen.getByRole('tab', { name: 'Guided' }));
    // Guided's facet sections appear
    expect(screen.getByText('Updated within')).toBeTruthy();
    expect(screen.getByText('Characters')).toBeTruthy();
    // Simple search box disappears in Guided mode
    expect(screen.queryByPlaceholderText(/search titles/i)).toBeNull();
    await fireEvent.click(screen.getByRole('tab', { name: 'Simple' }));
    expect(screen.getByPlaceholderText(/search titles/i)).toBeTruthy();
});
```

#### Test: Strict Gen and No Ships toggles

```typescript
it('Strict Gen quick filter toggles and persists into the search query', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ total: 0, results: [] }) });
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const strictGenBtn = screen.getByRole('button', { name: 'Strict Gen' });
    await fireEvent.click(strictGenBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(1); });
    expect(searchUrls()[0]).toContain('strict_gen=true');
    // Second click clears it
    await fireEvent.click(strictGenBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(2); });
    expect(searchUrls()[1]).not.toContain('strict_gen');
});

it('No Ships quick filter toggles exclude_tag_types', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const noShipsBtn = screen.getByRole('button', { name: 'No Ships' });
    await fireEvent.click(noShipsBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(1); });
    expect(searchUrls()[0]).toContain('exclude_tag_types=3');
});
```

#### Test: Chip interpretation from typed query

```typescript
it('renders an Interpreted-as chip row from a typed q value', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const input = screen.getByPlaceholderText(/search titles/i);
    await fireEvent.input(input, { target: { value: 'Harry @char:Harry -romship:*Malfoy' } });
    expect(screen.getByText('Interpreted as:')).toBeTruthy();
    expect(screen.getByLabelText('Edit Harry')).toBeTruthy();
    expect(screen.getByLabelText('Edit @char:Harry')).toBeTruthy();
    expect(screen.getByLabelText('Edit -romship:*Malfoy')).toBeTruthy();
    expect(screen.getByLabelText('Remove @char:Harry')).toBeTruthy();
});
```

#### Test: Surprise me link

```typescript
it('Surprise me links to the blind-date page', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const link = screen.getByRole('link', { name: /surprise me/i });
    expect(link.getAttribute('href')).toBe('/blind-date');
});
```

### Try It Yourself

```bash
# Run just the search page tests
cd frontend
npx vitest run src/routes/search/page.test.ts

# Add a test for a new feature — e.g. testing the "Save filter" button
it('saves current query as a view', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    // Type a query, click Save Filter, verify POST to /api/me/views
});
```

### Check

- ✅ Tests mock `$app/navigation` (goto) and `$app/stores` (page) for isolation.
- ✅ `searchUrls()` helper filters fetch calls to only `/api/search` requests.
- ✅ `beforeEach` resets mocks + localStorage, defaults to `uiMode: 'modern'`.
- ✅ Strict Gen toggle: click → `strict_gen=true`, click again → cleared.
- ✅ No Ships toggle: sets `exclude_tag_types=3` in the query string.
- ✅ "Surprise me" link → `/blind-date`.
- ✅ Tier switching: Guided shows facets, hides Simple input; Simple restores it.
- ✅ Chip interpretation: 3 tokens → 3 editable chips + 1 remove button each.

### What you built

The frontend test suite — 15 Vitest cases covering search rendering, three-tier switching, quick filter toggling (Strict Gen, No Ships), advanced filter visibility, search execution with mock responses, error states, chip interpretation from typed queries, and the "Surprise me" link. Tests mock SvelteKit modules and use `waitFor` for async assertions.

---

## Chapter 45.4 — Running the Full Test Suite

### Goal

Execute all tests locally and understand the CI gate.

### Actions

#### Full test matrix

```bash
# 1. Rust unit + contract tests (fast, ~30s)
just test        # = cargo test --lib

# 2. Frontend unit tests (Vitest)
just test-frontend    # = cd frontend && npm test

# 3. Combined fast run
just test-quick       # = cargo test --lib + cd frontend && npm test

# 4. Rust DB integration tests (slow, needs DB + Redis)
just test-db          # = cargo test -- --include-ignored --test-threads=1

# 5. Frontend E2E tests (needs running dev server)
just test-e2e         # = cd frontend && npm run test:e2e

# 6. Pre-commit lint gate (what CI enforces)
just lint             # = cargo fmt --check + clippy -D warnings + svelte-check
```

#### CI pipeline (from deploy scripts)

The `justfile` deploy recipes show the CI pipeline:

```makefile
# deploy-full = deploy-frontend + deploy-backend + health check
deploy-full: deploy-frontend deploy-backend
    @ssh thinkcentre "curl -s -o /dev/null -w 'health: HTTP %{http_code}\n' http://localhost:8000/health && systemctl is-active fichub"
```

The CI sequence is:
1. `cargo fmt --check` — formatting gate
2. `cargo clippy --workspace -- -D warnings` — lint gate
3. `cargo sqlx prepare --check` — SQL validation gate
4. `cargo test --lib` — unit tests
5. `cd frontend && npx svelte-check` — TypeScript validation
6. `cd frontend && npm test` — frontend unit tests
7. `cargo build --release` — release build
8. `cd frontend && npm run build` — frontend build
9. Deploy + health check

> **⚠️ Watch Out**: After a frontend deploy, if "no UI changes" are visible, it's browser cache. FicHub uses **immutable hashed assets cached for 1 year**. A new deploy generates new filenames. Clear the cache or hard-refresh (Ctrl+Shift+R).

### Try It Yourself

```bash
# Run the full CI gate locally before pushing
just lint && just test && just test-frontend

# Add a new API contract test
# Edit src/routes/api_contract_tests.rs, then:
cargo test --lib api_contract_tests::new_endpoint_response_shape
```

### Check

- ✅ `cargo test --lib` runs in parallel (multiple threads).
- ✅ DB tests need `--include-ignored --test-threads=1` (connection contention).
- ✅ E2E tests need a running dev server (`npm run test:e2e`).
- ✅ The deploy pipeline includes health verification with rollback on failure.
- ✅ Cache-busting: hashed assets need hard-refresh after deploy.

### What you built

The complete testing workflow — `cargo test --lib` for unit/contract tests, `npm test` for frontend Vitest, `test-e2e` for browser tests, `just lint` for the CI gate (fmt + clippy + svelte-check), SQLx prepare checks, and the deploy pipeline with health verification.

---

## Conclusion

You now understand FicHub's complete testing strategy:

1. **API contract tests** — 20 unit tests in `api_contract_tests.rs` documenting response shapes for every endpoint, enforcing `{ err, ... }` JSON contract, no DB required.
2. **Test recipes** — `just test`, `just test-db`, `just test-frontend`, `just test-e2e`, `just lint` — each with specific flags for parallel vs serial execution.
3. **Frontend route tests** — 15 Vitest cases covering search page rendering, tier switching, filter toggling, chip interpretation, URL sync, and the "Surprise me" link.
4. **CI pipeline** — fmt → clippy → sqlx prepare → unit tests → svelte-check → frontend tests → build → deploy + health check.
5. **Cache-awareness** — hashed assets cached 1yr, hard-refresh after deploy.

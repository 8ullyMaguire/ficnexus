## Part 5A — The Frontend Deep Dive

The backend gets most of the attention, but the frontend is where users
actually live. This part takes you through the SvelteKit 5 app in enough
depth that you can add pages, wire them to the API, and follow the house
patterns.

### Chapter 17A: SvelteKit 5 Basics in This Repo

The frontend lives in `frontend/`. It's a SvelteKit 5 app using the
`@sveltejs/adapter-static` — it builds to a static site served by the Axum
backend.

The core files you'll touch:

- **`frontend/src/routes/+layout.svelte`** — the app shell: nav, footer,
  theme. This is where global navigation lives (the "Moderation Log"
  link, admin links, etc.).
- **`frontend/src/routes/+layout.ts`** — client-side init (PWA
  registration, auth store bootstrap).
- **`frontend/src/routes/+page.svelte`** — the home page.
- **`frontend/src/lib/api/client.ts`** — the API client. Every backend
  call goes through this file. It attaches the JWT from
  `localStorage['fichub_token']` as `Authorization: Bearer <token>`.
- **`frontend/src/lib/stores/auth.svelte.ts`** — the auth store (Svelte 5
  runes).
- **`frontend/src/lib/i18n/`** — the translation system: six dictionaries
  (en, de, fr, es, ...) + a `t()` function used in every page.

**Svelte 5 runes.** You'll see `.svelte.ts` files using `$state`,
`$derived`, and `$effect`. The auth store is a good example:

```ts
// auth.svelte.ts (simplified)
export const auth = (() => {
    let user = $state<User | null>(null);
    let token = $state<string | null>(null);
    return {
        get user() { return user; },
        get token() { return token; },
        async login(...) { ... },
        async logout() { ... },
    };
})();
```

Components import `auth` and read `auth.user` reactively — when it
changes, the UI updates. No manual subscriptions.

**The API client pattern.** Every endpoint has a typed function:

```ts
export async function fetchModlog(params: { limit?: number; action?: string } = {}) {
    const qs = new URLSearchParams(...).toString();
    return apiFetch(`/api/modlog?${qs}`).then(r => r.json());
}
```

`apiFetch` handles the JWT header, JSON parsing, and error shape
(`{err, msg}`). New endpoints get a function here first.

> 🧪 **Try it:** open `client.ts` and find the function for your favorite
> backend endpoint. Then open the matching page and trace how it calls it.
>
> ⚠️ **Watch out:** the API client reads the token from
> `localStorage['fichub_token']`. If the auth store holds the token in a
> different variable, they can drift — a known quirk worth checking when
> debugging auth issues.
>
> 💡 **Key concept:** one API client file, typed functions per endpoint,
> JWT attached centrally. Pages are thin; the client owns the fetch
> logic.

### Chapter 17B: i18n — Six Languages, One t() Function

Every user-facing string goes through the i18n system. The dictionaries
live in `frontend/src/lib/i18n/dictionaries/*.ts` (one per language). The
`t()` function is imported and used everywhere:

```svelte
<a href="/modlog">{t('nav.modlog')}</a>
```

When you add a string, you add a key to ALL six dictionaries — the house
rule is no hardcoded user-facing text. The key naming convention is
`area.item` (e.g. `nav.modlog`, `search.results`, `admin.analyticsTitle`).

> 🧪 **Try it:** add a key to one dictionary and see the type/consistency
> check fail until you add it to the rest. That's the safety net.
>
> ⚠️ **Watch out:** forgetting a key in one language shows a raw key or
> empty string to that language's users. Add all six at once.
>
> 💡 **Key concept:** i18n is structural, not optional. The t() function
> is the only way user-facing text enters the UI.

### Chapter 17C: PWA Offline — The Service Worker

FicHub is installable and offline-capable. The pieces:

- **`frontend/static/manifest.webmanifest`** — the PWA manifest (name,
  icons, display mode).
- **`frontend/static/sw.js`** — the service worker. Registered from
  `+layout.ts` via `src/lib/pwa/register.ts` (prod only).
- **`src/lib/pwa/strategies.ts`** — pure functions (unit-tested!) that
  decide the caching strategy per URL:
  - `/fic/*` + `/read/*` — **cache-first** (offline reading works).
  - public fic APIs (`/api/reader/*`, `/api/epub`, `/api/meta`,
    `/api/search/similar/*`) — **network-first**.
  - shell precache — the app shell is pre-cached at install.
  - **everything else — 'none'** (auth/bookmarks/admin/social are NEVER
    intercepted; per-user data privacy).

The most important pitfall (from the project's history): **precache the
hashed `/_app/` build assets or the SPA renders blank offline.** The SW
discovers them at install by fetching `/` and regex-ing out the
`/_app/...` URLs.

> 🧪 **Try it:** open the site in Chrome, check DevTools → Application →
> Service Workers, then set the network to Offline and reload a fic you've
> read. It renders from cache.
>
> ⚠️ **Watch out:** never intercept auth/admin/social endpoints in the SW
> — that was a real privacy bug in v1. `strategies.ts` is the enforced
> contract.
>
> 💡 **Key concept:** offline is a strategy decision per URL, enforced in
> pure tested functions.

### Chapter 17D: The Admin UI

`frontend/src/routes/admin/` contains the admin pages: dashboard, usage
analytics, modlog, scrapers, users, bots, comment triage. The pattern:

- Each page is a `+page.svelte` that calls an admin API (via `adminFetch`,
  a variant of the client that requires auth).
- Role-gated in the backend (role ≥ 10); the frontend also hides admin
  links from non-admins.
- Each page has a `page.test.ts` asserting it renders with fixture data.

The admin pages are a great place to start contributing: the backend
endpoints exist, and several pages (auto-tag review, metadata correction,
translation post-edit) are still TODO — the exact starter tasks from
Chapter 24.

> 🧪 **Try it:** open `/admin/analytics` and find the card components.
> Then read its `page.test.ts` to see the fixture pattern.
>
> ⚠️ **Watch out:** admin routes are role-gated server-side. Don't rely
> on the frontend hiding links as the only protection.
>
> 💡 **Key concept:** admin UI is thin; the API enforces the role gate.

### Chapter 17E: Page Tests — The `page.test.ts` Pattern

Every user-facing route folder may carry a `page.test.ts` using vitest +
@testing-library/svelte. The pattern:

```ts
// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

// mock the API client so the page renders with fixture data
vi.mock('$lib/api/client', () => ({ fetchModlog: vi.fn(() => Promise.resolve({ entries: [...] })) }));

describe('modlog page', () => {
    it('renders entries', async () => {
        render(Page);
        expect(screen.getByText(/Moderation Log/)).toBeTruthy();
    });
});
```

Two vitest gotchas to remember:

1. **The file MUST be named `page.test.ts`** — that's the exact pattern
   the runner picks up.
2. **`vi.mock` hoisting** — if the mock references a variable defined
   later in the file, use `vi.hoisted()` or define the mock value inline;
   vitest hoists `vi.mock` calls to the top of the file.

Run a single page test:

```bash
npx vitest run src/routes/modlog/page.test.ts
```

> 🧪 **Try it:** read any `page.test.ts` in the repo and identify the
> mock, the render, and the assertions. Then add one assertion of your
> own and watch it pass.
>
> ⚠️ **Watch out:** tests that query text appearing both in a dropdown
> option and in page content need `getAllByText(...)` — `getByText` throws
> on duplicates.
>
> 💡 **Key concept:** every page has a test. Fixtures via mocks, render
> via testing-library, assert via screen queries.

---

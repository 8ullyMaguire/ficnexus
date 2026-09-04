# Part 29 — Translations and Localization

> In this chapter you will learn how FicHub supports six languages (English, German, Spanish, French, Brazilian Portuguese, Chinese) and switches between AO3-style and modern UI layouts. We'll build the i18n store, the locale detection flow, the dictionary system, and the preference persistence layer.

---

## Overview

FicHub's localization system has three layers:

1. **Dictionaries** — flat key→value maps in `src/lib/i18n/dictionaries/{locale}.ts` (6 files: `en.ts`, `de.ts`, `es.ts`, `fr.ts`, `pt-BR.ts`, `zh.ts`). Each contains ~923 keys covering every UI string.
2. **i18n store** — a reactive singleton in `src/lib/i18n/index.svelte.ts` that loads the right dictionary, interpolates `{variables}`, and persists the user's language choice to `localStorage`.
3. **UI preferences** — the `UserPrefs` blob in `src/lib/prefs.ts` that stores `uiMode` (`'archive'` or `'modern'`), `readerTheme`, `defaultFormat`, and other settings.

The backend also stores a `locale` column on the `users` table (set via `PUT /api/auth/locale`), but the frontend's `localStorage` choice always wins at startup.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database, getting a single work, searching, user accounts).
- You have completed Part 8 (User Accounts) — the auth store and JWT flow.
- Familiarity with SvelteKit runes (`$derived`, `$state`, `$props`).

---

## Chapter 29.1 — The Dictionary System

### Goal

Understand how FicHub stores and loads translations — flat key maps where every key must exist in every locale.

### Actions

#### 1. The directory structure

```
frontend/src/lib/i18n/
├── dictionaries/
│   ├── en.ts          ← source of truth (always has all keys)
│   ├── de.ts
│   ├── es.ts
│   ├── fr.ts
│   ├── pt-BR.ts
│   ├── zh.ts
│   └── index.ts       ← re-export dictionary objects + LOCALES constant
├── index.svelte.ts    ← reactive i18n store (I18nStore class)
└── index.test.ts      ← translation tests
```

#### 2. The dictionary format

Each locale file exports a flat object with namespaced keys:

```typescript
// frontend/src/lib/i18n/dictionaries/en.ts (excerpt, 44.5K, 923 keys)
export const en = {
  // ── Navigation ────────────────────────────────────────────────────────
  'nav.discover': 'Discover',
  'nav.library': 'Library',
  'nav.account': 'Account',
  'nav.trending': 'Trending',
  'nav.rankings': 'Rankings',
  'nav.forum': 'Forum',
  'nav.askTheArchive': 'Ask the Archive',
  'nav.login': 'Login',
  'nav.logout': 'Logout',

  // ── Download tab ──────────────────────────────────────────────────────
  'dl.pasteUrl': 'Paste a fanfiction URL (AO3, FanFiction.net, forums…)',
  'dl.download': 'Download',
  'dl.notAvailable': 'This fic is not available for download right now.',
  'dl.bookmark': 'Bookmark',
  'dl.bookmarked': 'Bookmarked',

  // ── Search ────────────────────────────────────────────────────────────
  'search.title': 'Advanced Search',
  'search.placeholder': 'Search titles, descriptions, tags…',
  'search.results': '{count} results',

  // ── Reader ────────────────────────────────────────────────────────────
  'reader.back': '← Back',
  'reader.next': 'Next ▸',
  'reader.markCompleted': 'Mark as Completed',

  // ── Forum ────────────────────────────────────────────────────────────
  'forum.title': 'Forum',
  'forum.subtitle': 'Community discussions — categories curated by the team.',
  'forum.newTopic': '+ New topic',
  'forum.reply': 'Reply',

  // ── ... and ~870 more keys for every UI string ...
}
```

> **💡 Key Concept**: Every key in `en.ts` MUST exist in `de.ts`, `es.ts`, `fr.ts`, `pt-BR.ts`, and `zh.ts`. Missing keys fall back to English at runtime (see `t()` below), but the CI `index.test.ts` checks for key parity and fails if a locale is missing keys.

#### 3. The locale registry

```typescript
// frontend/src/lib/i18n/dictionaries/index.ts
export const LOCALES = ['en', 'de', 'es', 'fr', 'pt-BR', 'zh'] as const;
export type LocaleCode = (typeof LOCALES)[number];

export const dictionaries: Record<LocaleCode, Record<string, string>> = {
  en: en, de: de, es: es, fr: fr, 'pt-BR': ptBR, zh: zh,
};

export type Dictionary = Record<string, string>;
// TranslationKey is a helper type — the full list of keys from en.ts
export type TranslationKey = keyof typeof en;
```

#### 4. Adding a new translation

1. Add the key to `en.ts` (source of truth).
2. Add the translated string to each of `de.ts`, `es.ts`, `fr.ts`, `pt-BR.ts`, `zh.ts`.
3. Use it in a component: `$derived(t('my.new.key'))` or `{t('my.new.key')}`.

> **⚠️ Watch Out**: Never remove keys from `en.ts` — old translations will show the raw key name. Deprecate keys by leaving them in `en.ts` with a `// deprecated` comment.

### Try It Yourself

```bash
# Run the i18n tests (checks key parity across all locales)
cd frontend && npm test -- index.test.ts

# Count keys per locale
for f in src/lib/i18n/dictionaries/*.ts; do
  echo "$f: $(grep -c \"': '\" "$f")"
done
```

### Check

- ✅ All 6 locale files exist in `dictionaries/`.
- ✅ `LOCALES` constant lists exactly: `'en', 'de', 'es', 'fr', 'pt-BR', 'zh'`.
- ✅ Every key in `en.ts` exists in all other locale files.
- ✅ The `t()` function falls back to English, then to the raw key.

### What you built

The dictionary architecture — flat key maps, a locale registry, and a build-time guarantee that every locale has the same keys.

---

## Chapter 29.2 — The Reactive i18n Store

### Goal

Understand how the `I18nStore` singleton loads the right dictionary, interpolates variables, and syncs with `localStorage` + `<html lang>`.

### Actions

#### 1. The store class

```typescript
// frontend/src/lib/i18n/index.svelte.ts (lines 25–120)
export const DEFAULT_LOCALE: LocaleCode = 'en';
export const STORAGE_KEY = 'fichub_locale';

class I18nStore {
  /** Currently active locale — components read this reactively. */
  locale = $state<LocaleCode>(DEFAULT_LOCALE);
  initialized = $state(false);

  /** Current dictionary (falls back to English if locale unknown). */
  get dictionary(): Dictionary {
    return dictionaries[this.locale] ?? dictionaries[DEFAULT_LOCALE];
  }

  /**
   * Look up a translation key with fallback chain:
   * current locale → English → raw key.
   * Supports {var} interpolation.
   */
  t(key: string, vars?: Record<string, string | number>): string {
    const dict = this.dictionary;
    let value: string | undefined = dict[key];
    if (value === undefined) value = dictionaries.en[key];
    if (value === undefined) value = key;
    if (vars && Object.keys(vars).length > 0) {
      for (const [k, v] of Object.entries(vars)) {
        value = value!.replaceAll(`{${k}}`, String(v));
      }
    }
    return value!;
  }

  /**
   * Switch language immediately, persist to localStorage,
   * and sync <html lang>.
   */
  setLocale(code: string): boolean {
    if (!isSupportedLocale(code)) return false;
    this.locale = code;
    try {
      localStorage.setItem(STORAGE_KEY, code);
    } catch { /* storage unavailable — in-memory still works */ }
    this.syncHtmlLang();
    return true;
  }

  /** Map browser language tags to closest supported locale. */
  detectBrowserLocale(languages?: readonly string[]): LocaleCode {
    // 'en-US' → 'en', 'pt-PT' → 'pt-BR', 'zh-CN' → 'zh', etc.
    // Full mapping in the skill file: lines 74–93
    }
  }

  /** Keep <html lang="..."> in sync with the active locale. */
  syncHtmlLang(): void {
    if (typeof document !== 'undefined') {
      document.documentElement.lang = this.locale;
    }
  }

  /**
   * Resolve the initial locale:
   * 1. localStorage ('fichub_locale') — explicit user choice
   * 2. auth.user.locale — from the backend
   * 3. detectBrowserLocale() — from navigator.languages
   */
  initI18n(options?: { userLocale?: string | null; browserLanguages?: readonly string[] }): LocaleCode {
    const stored = typeof localStorage !== 'undefined'
      ? localStorage.getItem(STORAGE_KEY) : null;
    if (isSupportedLocale(stored)) {
      this.setLocale(stored);
    } else if (isSupportedLocale(options?.userLocale)) {
      this.setLocale(options.userLocale);
    } else {
      this.setLocale(this.detectBrowserLocale(options?.browserLanguages));
    }
    this.initialized = true;
    return this.locale;
  }
}

export const i18n = new I18nStore();
```

#### 2. The reactive `t()` helper

```typescript
/** Reactive translation helper for components:
 *  In template:  {$derived(t('nav.discover'))}
 *  In script:    const label = $derived(t('search.results', { count: 42 }));
 */
export function t(key: string, vars?: Record<string, string | number>): string {
  return i18n.t(key, vars);
}
```

> **💡 Key Concept**: In SvelteKit components, use `$derived(t('key'))` so the text re-renders when the locale changes. In `<script>` context, use `const label = $derived(t('key', { count: n }))`.

#### 3. Variable interpolation

```typescript
// t('search.results', { count: 42 }) → "42 results" (en)
// t('search.results', { count: 1 })  → "1 result"  (en)
// t('nav.unread', { count: 5 })      → "New (5)"   (en)
```

The interpolation uses `String.replaceAll('{key}', value)` — simple but covers all FicHub's use cases.

#### 4. The locale selector component

The `LocaleSelector` dropdown (used in the settings/theme page) calls `setLocale` and also syncs to the backend:

```typescript
// In the theme settings page (src/routes/settings/theme/+page.svelte)
function changeLocale(newLocale: string) {
  i18n.setLocale(newLocale);
  // Also persist to the backend (user.locale)
  fetch('/api/auth/locale', {
    method: 'PUT',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ locale: newLocale }),
  });
}
```

### Try It Yourself

```bash
# Run the i18n tests
cd frontend && npm test

# In browser console:
i18n.locale          // → "en"
i18n.t('search.results', {count: 42})  // → "42 results"
i18n.setLocale('es')  // switches to Spanish + persists
```

### Check

- ✅ `i18n` is a singleton (only one instance across the app).
- ✅ `t('missing.key')` returns `'missing.key'` (graceful fallback).
- ✅ `setLocale('es')` updates `localStorage['fichub_locale']` and `<html lang>`.
- ✅ `detectBrowserLocale(['pt-PT'])` returns `'pt-BR'` (regional fallback).

### What you built

A reactive i18n store with fallback chains, variable interpolation, browser detection, and localStorage persistence — plus the locale selector integration.

---

## Chapter 29.3 — UI Mode Preferences

### Goal

Understand how `UserPrefs` stores UI preferences (modern vs. AO3 archive mode, reader theme, default format) in `localStorage` and syncs server-side settings.

### Actions

#### 1. The UserPrefs type

```typescript
// frontend/src/lib/prefs.ts (lines 8–36)
export type ArchiveSkin = 'zerafina' | 'ao3';
export type VisibilityLevel = 'public' | 'followers' | 'private';

export interface UserPrefs {
  /** Download format: 'epub' | 'html' | 'txt' | 'md' | 'mobi' | 'pdf' | 'azw3' | ... */
  defaultFormat?: string;
  /** Hide works you've finished reading in search */
  hideRead?: boolean;
  /** Hide bookmarked works in search */
  hideBookmarked?: boolean;
  /** Only show bookmarked works ("My Library") */
  libraryOnly?: boolean;
  /** Reader theme: 'light' | 'sepia' | 'dark' */
  readerTheme?: 'light' | 'sepia' | 'dark';
  /** Interface style: 'archive' (AO3-style) or 'modern' (SvelteKit UI) */
  uiMode?: 'archive' | 'modern';
  /** Archive skin: 'zerafina' (rich styling) or 'ao3' (minimal) */
  archiveSkin?: ArchiveSkin;
  /** Timezone preference (IANA, e.g. 'America/New_York') */
  timezone?: string;
  /** Profile visibility settings */
  profileVisibility?: {
    profile: VisibilityLevel;
    works: VisibilityLevel;
    reading_history: VisibilityLevel;
  };
}
```

#### 2. Defaults and storage

```typescript
const STORAGE_KEY = 'fichub_prefs_v1';
const DEFAULTS: UserPrefs = {
  defaultFormat: 'epub',
  hideRead: false,
  hideBookmarked: false,
  libraryOnly: false,
  readerTheme: 'light',
  uiMode: 'archive',
  archiveSkin: 'zerafina',
  timezone: '',
  profileVisibility: {
    profile: 'public',
    works: 'public',
    reading_history: 'private',
  },
};

/// Read the prefs blob; never throws (corrupt JSON → defaults).
export function loadPrefs(): UserPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as UserPrefs;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

/// Persist the whole blob.
export function savePrefs(prefs: UserPrefs): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch { /* Storage full — prefs are a nicety, never fatal */ }
}

/// Read one pref with its default.
export function getPref<K extends keyof UserPrefs>(key: K): NonNullable<UserPrefs[K]> {
  const prefs = loadPrefs();
  const value = prefs[key];
  return (value === undefined ? DEFAULTS[key] : value) as NonNullable<UserPrefs[K]>;
}

/// Set one pref and persist.
export function setPref<K extends keyof UserPrefs>(key: K, value: UserPrefs[K]): void {
  const prefs = loadPrefs();
  prefs[key] = value;
  savePrefs(prefs);
}
```

#### 3. `?ui=archive` URL parameter

The root layout checks for a `ui` query parameter on every navigation and applies it:

```typescript
/// Applied by the root layout on every navigation.
export function applyUiParam(url: URL): void {
  const ui = url.searchParams.get('ui');
  if (ui === 'archive' || ui === 'modern') {
    setPref('uiMode', ui);
  }
}
```

So a user can force archive mode by visiting `https://fichub.example.com/?ui=archive`.

#### 4. Server-side persistence

For settings that should survive across devices (timezone, profile visibility), the frontend calls `PUT /api/me/prefs` which persists to the DB:

```typescript
/// Persist a single preference to both localStorage AND the backend.
export async function persistPref<K extends keyof UserPrefs>(
  key: K, value: UserPrefs[K],
): Promise<boolean> {
  setPref(key, value);  // localStorage
  try {
    const res = await fetch('/api/me/prefs', {
      method: 'PUT',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify([{ key, value: JSON.stringify(value) }]),
    });
    return res.ok;
  } catch { return false; }
}
```

#### 5. Using `uiMode` in components

Every page checks `uiMode` to decide between AO3-style table and modern card rendering:

```svelte
<script>
  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
  <!-- AO3-style bordered table with .archive-* classes -->
  <main class="archive-main">
    <table class="archive-list">
      <thead><tr><th>Name</th><th>Stories</th></tr></thead>
      ...
    </table>
  </main>
{:else}
  <!-- Modern card grid with CSS variables -->
  <div class="fandom-grid">
    ...
  </div>
{/if}
```

### Try It Yourself

1. Open FicHub in your browser.
2. Go to Settings → Theme.
3. Switch `uiMode` to `archive` — the page should reload with AO3-style tables.
4. Check `localStorage.getItem('fichub_prefs_v1')` — it should contain `{"uiMode":"archive",...}`.

### Check

- ✅ `getPref('uiMode')` returns `'archive'` or `'modern'` (never undefined).
- ✅ `?ui=archive` in the URL overrides `uiMode` on load.
- ✅ `persistPref` sends a `PUT /api/me/prefs` request with `[{key, value}]`.
- ✅ Corrupt `localStorage` JSON falls back to `DEFAULTS` without throwing.
- ✅ `uiMode` defaults to `'archive'` (AO3-style).

### What you built

The UI preferences system — a localStorage blob with sensible defaults, a `?ui=` URL override, server-side sync for cross-device settings, and reactive `$derived` access in components.

---

## Chapter 29.4 — Backend Locale Persistence

### Goal

The backend stores the user's preferred locale in the `users` table. The frontend syncs its choice here; the backend's `locale` column is just one input to `initI18n()`.

### Actions

#### 1. The locale column

The `users` table has a `locale` column (nullable, defaults to NULL):

```sql
ALTER TABLE users ADD COLUMN locale TEXT;
ALTER OWNER TO fichub;
```

When NULL, the frontend uses browser detection.

#### 2. The PUT /api/auth/locale endpoint

```rust
// Registered in server.rs as a user-preference route
// PUT /api/auth/locale — update the user's locale preference
pub async fn update_locale(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<LocaleBody>,
) -> Result<Json<Value>, AppError> {
    let uid = auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if !is_valid_locale(&body.locale) {
        return Err(AppError::BadRequest(format!("Unsupported locale: {}", body.locale)));
    }

    sqlx::query("UPDATE users SET locale = $1 WHERE id = $2")
        .bind(&body.locale)
        .bind(uid)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "locale": body.locale })))
}

fn is_valid_locale(locale: &str) -> bool {
    matches!(locale, "en" | "de" | "es" | "fr" | "pt-BR" | "zh")
}

#[derive(Deserialize)]
struct LocaleBody {
    locale: String,
}
```

#### 3. Sending locale on login

When the user logs in, the `User` struct includes the `locale` field. The auth store uses this to seed `initI18n` before showing the UI:

```typescript
// In the auth store (src/lib/stores/auth.svelte.ts)
async init() {
  const res = await fetch('/api/auth/session', { credentials: 'include' });
  const data = await res.json();
  if (data.user) {
    this.user = data.user;
    // Seed i18n with the user's stored locale
    initI18n({ userLocale: data.user.locale, browserLanguages: navigator?.languages });
  }
}
```

The resolution order in `initI18n`:
1. `localStorage['fichub_locale']` — explicit choice, wins
2. `user.locale` — backend preference from the DB
3. `detectBrowserLocale()` — browser language detection

> **⚠️ Watch Out**: The backend locale is only a fallback. If the user has never set a locale in localStorage, the backend value is used. But if the user switches language in the UI, that only updates localStorage — the backend is NOT updated unless the `LocaleSelector` component explicitly calls `PUT /api/auth/locale`.

### Try It Yourself

```bash
# Change your locale on the backend
curl -X PUT "http://localhost:8000/api/auth/locale" \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{"locale": "es"}' | jq
```

### Check

- ✅ `PUT /api/auth/locale` requires authentication (401 without JWT).
- ✅ The locale is validated against the 6 supported codes.
- ✅ `locale` is NULL by default (frontend relies on browser detection).
- ✅ `initI18n` prioritizes localStorage → user.locale → browser detection.

### What you built

The backend locale persistence — a simple `locale` column on `users`, a `PUT /api/auth/locale` endpoint, and the three-tier resolution chain that the frontend's `initI18n` follows.

---

## Conclusion

You now understand FicHub's complete localization system:

- **6 dictionaries** (`en`, `de`, `es`, `fr`, `pt-BR`, `zh`) with ~923 keys each, validated for key parity.
- **Reactive i18n store** with fallback chain (current → English → raw key), variable interpolation, browser detection, and localStorage persistence.
- **UI preferences** — `UserPrefs` blob with `uiMode` (archive/modern), `readerTheme`, `defaultFormat`, and more, with `?ui=` URL override and server-side sync.
- **Backend locale** — `PUT /api/auth/locale` persists the user's choice; `initI18n` resolves localStorage → backend → browser.

The key insight: the frontend's `uiMode` preference (`'archive'` or `'modern'`) drives the dual rendering system that makes FicHub look like AO3 (archive mode) or a modern SvelteKit app. All the archive-style components are in `frontend/src/lib/ui/archive/`.

---

## On to the next part

In Part 21 we'll build [Roadmap Consensus Engine] — how FicHub uses Ollama embeddings to cluster similar feature suggestions and ranks them via an Elo-style algorithm.

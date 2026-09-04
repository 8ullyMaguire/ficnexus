# Part 1: Welcome & Architecture

## Chapter 1: What Is the Archive Interface?

Welcome to the FicHub Archive Frontend tutorial! If you're reading this, you're probably about to dive into a large, opinionated codebase that powers a fanfiction archive — and you want to understand how it works from the inside out.

Let's start with the big picture.

### Two Faces of One App

FicHub has **two distinct user interfaces** baked into a single SvelteKit application:

1. **Archive Mode** — a text-dense, tag-heavy layout inspired by [Archive of Our Own (AO3)](https://archiveofourown.org). Think compact rows, maroon accents, serif headings, and a sidebar packed with search filters.

2. **Modern Mode** — a clean, contemporary card-based UI with generous whitespace, hover effects, and a more "startup dashboard" feel.

Here's the surprise: **Archive Mode is the default**. Every new user lands on the Archive interface first. If you've used AO3, FicHub will feel immediately familiar. If you haven't, you'll still appreciate the density — when you're browsing thousands of stories, less padding means more content on screen.

> **💡 Key Concept — Why Two Modes?**
>
> Building two complete UIs in one app might sound wasteful, but it's a deliberate trade-off. The fanfiction community is deeply attached to AO3's visual language. A modern redesign loses users who associate that look with trust and usability. Keeping both modes lets FicHub attract new users with a polished modern feel *and* retain power users who want the classic archive experience.

### What We Copied from AO3 — and What We Changed

The Archive interface borrows several design patterns from AO3:

- **Two-row header** — logo on top, red navigation bar below
- **Work blurbs** — compact list items showing title, author, word count, tags, and summary in a single row
- **Tag soup** — freeform tags displayed as inline pills with category coloring
- **Sidebar search filters** — collapsible fieldsets for fandom, character, rating, completion status, etc.
- **Dense footer** — four-column link grid on a maroon background

What we *diverged* on:

- **Chips feature** — personalized "commonly added" tag suggestions above search results (AO3 has nothing like this)
- **Two-mode search form** — the same component renders as either a full page or a collapsible sidebar, depending on context
- **URL-driven UI switching** — you can force Archive or Modern mode by adding `?ui=archive` or `?ui=modern` to any URL
- **Theme presets** — Archive Classic (light) and Archive Noir (dark) with CSS custom properties

### How Users Switch Between Modes

There are **four ways** a user can switch their interface:

1. **Settings page** — Navigate to Settings → Interface Style, pick Archive or Modern
2. **Footer link** — In Archive mode, the footer says "Switch to modern interface"; in Modern mode, it says "Prefer the AO3 look? Switch interface"
3. **User dropdown menu** — Click your username, select the switch option
4. **URL parameter** — Add `?ui=archive` or `?ui=modern` to any page URL

The URL parameter is especially useful for sharing: you can send someone a link like `https://fichub.net/search?q=harry+potter&ui=archive` and they'll see results in Archive mode regardless of their saved preference.

> **🧪 Try It Yourself**
>
> Open FicHub in your browser, then add `?ui=modern` to the URL. The page reloads in Modern mode. Now add `?ui=archive` and it flips back. Your preference is saved to localStorage automatically — visit any other page and it sticks.

---

## Chapter 2: Project Structure

Now that you know *what* the Archive interface is, let's look at *where* it lives in the codebase.

### Where Archive Code Lives

All Archive-specific components live under a single directory:

```
frontend/src/lib/ui/archive/
```

This is inside SvelteKit's `$lib` directory — the convention for library code that isn't a route page. The `ui/archive/` subdirectory keeps everything cleanly separated from the rest of the application.

### The Full File Inventory

There are **17 files** in total. Here they are, grouped by type:

#### Svelte Components (14 files)

| File | Lines | Purpose |
|------|-------|---------|
| `ArchiveWorkSearchForm.svelte` | 853 | The main search form — full page and sidebar modes |
| `ArchiveHome.svelte` | 470 | Home page content for Archive mode |
| `ArchiveFilters.svelte` | 456 | Sidebar filter panel with collapsible fieldsets |
| `ArchiveWork.svelte` | 407 | Individual work detail page |
| `ArchiveHeader.svelte` | 347 | Two-row header (logo + red navbar) |
| `ArchiveSearch.svelte` | 308 | Search results page layout |
| `ArchiveListPage.svelte` | 216 | Generic list page wrapper |
| `WorkBlurb.svelte` | 208 | Compact work listing row |
| `ArchiveFooter.svelte` | 124 | Four-column maroon footer |
| `TagSoup.svelte` | 117 | Inline tag pills with category coloring |
| `ArchiveLayout.svelte` | 63 | Shell: wraps header + content + footer |
| `ArchiveButton.svelte` | 58 | Polymorphic button/link component |
| `StatsLine.svelte` | 52 | Word count, chapters, kudos stats row |
| `ArchiveNavLink.svelte` | 49 | Navigation link with active state indicator |

#### TypeScript Modules (2 files)

| File | Lines | Purpose |
|------|-------|---------|
| `searchForm.ts` | 480 | Search form state management, query building, field parsing |
| `rating.ts` | 118 | Rating display logic (G, T, M, E ratings with icons) |

#### Test Files (1 file)

| File | Lines | Purpose |
|------|-------|---------|
| `searchForm.test.ts` | 418 | Unit tests for search form parsing and query building |

**Total: 4,744 lines of code** — that's a substantial codebase for a single interface mode.

### How Archive Components Relate to Route Pages

Here's a key thing to understand: **Archive components are not routes themselves**. They're UI components that get *rendered by* the same SvelteKit routes that power Modern mode. The switching happens at the root layout level (more on this in Chapter 3).

The relationship looks like this:

```
Route: /search
  └─ Modern mode: renders ModernSearch.svelte
  └─ Archive mode: renders ArchiveSearch.svelte (which uses ArchiveWorkSearchForm)

Route: /work/[id]
  └─ Modern mode: renders ModernWork.svelte
  └─ Archive mode: renders ArchiveWork.svelte
```

This means you'll never find an `archive/search/+page.svelte` route file. The routing is shared — only the *rendering* switches.

### The Shared Layer

Archive components don't exist in a vacuum. They depend on shared infrastructure:

- **API clients** (`$lib/api/`) — HTTP calls to the FicHub backend
- **Auth store** (`$lib/stores/auth.svelte`) — reactive authentication state
- **Prefs store** (`$lib/prefs`) — user preferences including `uiMode`
- **Theme system** (`$lib/themes/`) — CSS custom property management
- **i18n** (`$lib/i18n/`) — internationalization for multi-language support

Every Archive component imports from this shared layer. The Archive code is *pure UI* — it contains no API logic, no auth logic, no persistence logic. That's all handled by the shared modules.

> **⚠️ Watch Out**
>
> When working on Archive components, be careful not to accidentally import from Modern-mode-specific modules. The components in `$lib/ui/archive/` should only depend on shared infrastructure, not on components in `$lib/components/` (those are Modern-mode-specific). The one exception is `CommandPalette.svelte` and `HelpModal.svelte`, which are shared modals imported into both layouts.

---

## Chapter 3: The Dual-Shell Architecture

This is the most important architectural concept in the entire codebase. If you understand how the two shells work, everything else falls into place.

### The Root Layout: Where the Split Happens

The entry point is `frontend/src/routes/+layout.svelte`. This is SvelteKit's root layout — it wraps every page in the application. Here's the critical decision it makes:

```svelte
{#if uiMode === 'archive'}
  <ArchiveLayout>
    {@render children()}
  </ArchiveLayout>
{:else}
  <div class="app">
    <!-- Modern mode header, nav, content, footer -->
    ...
  </div>
{/if}
```

That's it. One `{#if}` block. The entire interface switch happens right here at the top level.

When `uiMode` is `'archive'`, the root layout wraps all page content in `<ArchiveLayout>`. When it's `'modern'`, it renders the standard Modern shell with its own header, navigation, and footer.

### How the Mode Is Determined

The `uiMode` variable comes from the prefs store:

```svelte
<script lang="ts">
  import { getPref, setPref, applyUiParam } from '$lib/prefs';

  let uiMode = $derived(getPref('uiMode'));

  onMount(async () => {
    applyUiParam($page.url);
    applyTheme(loadTheme());
    // ... auth init, locale init
  });
</script>
```

Let's break this down:

1. **`getPref('uiMode')`** reads the user's saved preference from localStorage
2. **`$derived(...)`** makes it reactive — if the preference changes, Svelte re-renders
3. **`applyUiParam($page.url)`** checks for `?ui=archive` or `?ui=modern` in the URL and updates the pref if found

### The applyUiParam() Function

This is defined in `frontend/src/lib/prefs.ts` (line 73):

```typescript
export function applyUiParam(url: URL): void {
  const ui = url.searchParams.get('ui');
  if (ui === 'archive' || ui === 'modern') {
    setPref('uiMode', ui);
  }
}
```

It runs on every navigation. If someone visits `/search?q=hello&ui=modern`, the `ui` parameter gets parsed, the pref is updated to `'modern'`, and the layout re-renders in Modern mode. Simple, deterministic, no surprises.

> **💡 Key Concept — URL-First Design**
>
> The `?ui=` parameter acts as a "URL override" that takes precedence over the saved preference. This is powerful for sharing links: you can force someone into a specific mode regardless of their settings. It's also useful for testing — developers can flip modes without opening Settings.

### How the Modern Shell Stays 100% Intact

Here's the elegant part: **Modern mode code has no idea Archive mode exists**. The `{:else}` branch in the root layout renders the complete Modern shell — its own header (`<header class="topbar">`), its own navigation (`<nav class="main-nav">`), its own footer (`<footer class="footer muted">`). None of those components import anything from `$lib/ui/archive/`.

The only Modern-mode code that references Archive is the `switchToArchive()` function in the root layout:

```typescript
function switchToArchive() {
  setPref('uiMode', 'archive');
  window.location.reload();
}
```

And this is called from the footer button: "Prefer the AO3 look? Switch interface." That's the *only* bridge from Modern to Archive.

Similarly, `ArchiveFooter.svelte` has the reverse:

```svelte
<script lang="ts">
  import { setPref } from '$lib/prefs';

  function switchToModern() {
    setPref('uiMode', 'modern');
    window.location.reload();
  }
</script>
```

Both shells call `window.location.reload()` after switching. This is intentional — a full page reload ensures CSS, state, and component trees are cleanly reset. It's not elegant, but it's *reliable*.

### The Preferences Store

The entire mode-switching system rests on `frontend/src/lib/prefs.ts` — a tiny 78-line file. Let's examine it:

```typescript
export interface UserPrefs {
  defaultFormat?: string;
  hideRead?: boolean;
  hideBookmarked?: boolean;
  libraryOnly?: boolean;
  readerTheme?: 'light' | 'sepia' | 'dark';
  uiMode?: 'archive' | 'modern';
}

const STORAGE_KEY = 'fichub_prefs_v1';

const DEFAULTS: UserPrefs = {
  defaultFormat: 'epub',
  hideRead: false,
  hideBookmarked: false,
  libraryOnly: false,
  readerTheme: 'light',
  uiMode: 'archive',    // ← Archive is the default!
};
```

Notice that `uiMode` defaults to `'archive'`. This means new users — anyone who has never explicitly set a preference — land on the Archive interface.

The store uses a **single JSON blob** in localStorage under the key `fichub_prefs_v1`. Every field is optional, so old or partial blobs degrade gracefully:

```typescript
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
```

The spread pattern `{ ...DEFAULTS, ...parsed }` means: start with all defaults, then override with whatever the user has saved. If a key is missing from the saved blob, it falls back to the default. If the JSON is corrupt, you get clean defaults. If localStorage is unavailable (private browsing, storage full), you get clean defaults. **The store never throws.**

> **🧪 Try It Yourself**
>
> Open your browser's developer console on FicHub and run:
>
> ```javascript
> JSON.parse(localStorage.getItem('fichub_prefs_v1'))
> ```
>
> You'll see your saved preferences. Now try:
>
> ```javascript
> localStorage.removeItem('fichub_prefs_v1');
> location.reload();
> ```
>
> The page reloads with Archive mode (the default) since you cleared your preferences.

---

## Chapter 4: Design Tokens & Theme System

Now let's talk about how the Archive interface actually *looks*. The visual identity is controlled by CSS custom properties — also known as CSS variables — which are fed by TypeScript "theme tokens."

### CSS Custom Properties: The Visual Language

Throughout the Archive components, you'll see references like `var(--archive-bg, #ffffff)`. These are CSS custom properties with fallback values. The key variables used across the archive shell are:

| Variable | Purpose | Default (Light) |
|----------|---------|-----------------|
| `--archive-bg` | Page background | `#ffffff` |
| `--archive-text` | Primary text color | `#2a2a2a` |
| `--archive-muted` | Secondary/muted text | `#666666` |
| `--archive-link` | Link color | `#990000` |
| `--archive-link-visited` | Visited link color | `#660066` |
| `--archive-border` | Border color | `#dddddd` |
| `--archive-bg-raised` | Elevated surface (cards, dropdowns) | `#f5f5f5` |
| `--archive-accent-line` | Navbar/footer accent color | `#990000` |

These variables are set on the root element by the theme system. Components reference them with inline fallbacks, so even if a variable is missing, the component still renders correctly.

Look at how `ArchiveLayout.svelte` applies them:

```svelte
<style>
  .archive-shell {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    font-family: Georgia, 'Times New Roman', serif;
  }

  :global(.archive-shell a) {
    color: var(--archive-link, #990000);
  }

  :global(.archive-shell a:visited) {
    color: var(--archive-link-visited, #660066);
  }
</style>
```

The `:global(.archive-shell a)` selector is important — it applies the link color to *all* anchor tags inside the archive shell, not just ones directly inside `ArchiveLayout`. This is how the entire interface gets consistent link styling without touching every component.

### Two Archive Themes

FicHub ships two Archive-specific theme presets: **Archive Classic** (light) and **Archive Noir** (dark). These are defined in `frontend/src/lib/themes/presets.ts`:

```typescript
export const archiveClassic: ThemeTokens = {
  name: 'Archive Classic',
  accent: '#990000',
  bg: '#ffffff',
  surface: '#f5f5f5',
  text: '#2a2a2a',
  muted: '#666666',
  radius: '2px',
  font: "Georgia, 'Times New Roman', serif",
  density: 'compact',
  readerFont: "Georgia, 'Times New Roman', serif",
  readerWidth: '680px',
  readerLineHeight: '1.8',
};

export const archiveNoir: ThemeTokens = {
  name: 'Archive Noir',
  accent: '#990000',
  bg: '#1e1e1e',
  surface: '#262626',
  text: '#e0e0e0',
  muted: '#999999',
  radius: '2px',
  font: "Georgia, 'Times New Roman', serif",
  density: 'compact',
  readerFont: "Georgia, 'Times New Roman', serif",
  readerWidth: '680px',
  readerLineHeight: '1.8',
};
```

Both share the same maroon accent (`#990000`), the same Georgia serif font, the same tight 2px radius, and the same compact density. The only differences are background and text colors — light vs. dark.

### The ThemeTokens Interface

Every theme preset implements the `ThemeTokens` interface:

```typescript
export interface ThemeTokens {
  name: string;
  accent: string;
  bg: string;
  surface: string;
  text: string;
  muted: string;
  radius: string;
  font: string;
  density: 'compact' | 'comfortable' | 'spacious';
  readerFont: string;
  readerWidth: string;
  readerLineHeight: string;
}
```

This interface is the contract. Each field maps to a CSS custom property when the theme is applied. The `density` field doesn't map to a single CSS variable — it's used programmatically to adjust spacing scales across components.

### The Design DNA

Let's look at what makes the Archive theme *feel* like an archive:

1. **Georgia serif headings** — `font: "Georgia, 'Times New Roman', serif"`. This is the single most distinctive visual choice. The moment you see serif text, you know you're in Archive mode.

2. **Maroon accent** — `accent: '#990000'`. This is a deep, scholarly red that AO3 uses for its brand. It appears in links, the navbar, and the footer.

3. **2px radius** — `radius: '2px'`. Almost no rounding at all. Cards, buttons, and inputs have sharp corners. This creates a "no-nonsense, information-first" feel.

4. **Compact density** — `density: 'compact'`. Less padding, tighter line heights, more content per screen.

5. **Dark visited links** — `#660066` (a deep purple). This is borrowed directly from AO3 and signals "you've been here before" in a visually distinct way.

### How Themes Are Applied

The theme system has two parts:

1. **Theme definition** (`presets.ts`) — TypeScript objects with token values
2. **Theme application** (`apply.js`) — reads the current theme and sets CSS custom properties on the root element

When the page loads, `applyTheme(loadTheme())` is called in the root layout's `onMount`. This reads the saved theme from localStorage, finds the matching preset, and sets CSS variables. The Archive components then pick up those variables via `var(--archive-*)` references.

> **⚠️ Watch Out**
>
> The Archive components hard-code fallback values inline (e.g., `var(--archive-bg, #ffffff)`). This means they always render correctly even if the theme system fails. But it also means if you change a theme preset, you need to make sure the *variable names* stay consistent across all components. The fallback values are safety nets, not the source of truth.

### All Seven Presets

While we focused on the Archive presets, FicHub actually ships **seven** theme presets total:

| Preset | Slug | Type |
|--------|------|------|
| Default Dark | `default-dark` | Modern |
| Default Light | `default-light` | Modern |
| High Contrast | `high-contrast` | Accessibility |
| Sepia | `sepia` | Reader-focused |
| Dyslexia Friendly | `dyslexia` | Accessibility |
| **Archive Classic** | `archive-classic` | Archive |
| **Archive Noir** | `archive-noir` | Archive |

The presets are indexed by slug in a single record:

```typescript
export const presets: Record<string, ThemeTokens> = {
  'default-dark': defaultDark,
  'default-light': defaultLight,
  'high-contrast': highContrast,
  'sepia': sepia,
  'dyslexia': dyslexia,
  'archive-classic': archiveClassic,
  'archive-noir': archiveNoir,
};
```

This makes it trivial to add a new theme — just create a `ThemeTokens` object and add it to the record.

> **🧪 Try It Yourself**
>
> Open FicHub, go to Settings → Theme, and switch between Archive Classic and Archive Noir. Watch how the entire interface recolors instantly — the header, links, borders, and content all update. Then open dev tools and inspect the `<html>` element to see the CSS custom properties changing.
>
> Now try this: search for `--archive` in your dev tools' Styles panel. You'll see every component that references these variables, giving you a complete map of the visual system.

---

*End of Part 1. In Part 2, we'll dive into the Archive components themselves — starting with the layout shell and header, then working through the work blurbs, tag soup, and search system.*

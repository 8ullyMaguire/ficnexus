---
title: "FicHub Archive Frontend Tutorial — AO3-Style UI Guide"
author: "FicHub Contributors"
date: "2026-08-19"
---

1|# Part 1: Welcome & Architecture
2|
3|## Chapter 1: What Is the Archive Interface?
4|
5|Welcome to the FicHub Archive Frontend tutorial! If you're reading this, you're probably about to dive into a large, opinionated codebase that powers a fanfiction archive — and you want to understand how it works from the inside out.
6|
7|Let's start with the big picture.
8|
9|### Two Faces of One App
10|
11|FicHub has **two distinct user interfaces** baked into a single SvelteKit application:
12|
13|1. **Archive Mode** — a text-dense, tag-heavy layout inspired by [Archive of Our Own (AO3)](https://archiveofourown.org). Think compact rows, maroon accents, serif headings, and a sidebar packed with search filters.
14|
15|2. **Modern Mode** — a clean, contemporary card-based UI with generous whitespace, hover effects, and a more "startup dashboard" feel.
16|
17|Here's the surprise: **Archive Mode is the default**. Every new user lands on the Archive interface first. If you've used AO3, FicHub will feel immediately familiar. If you haven't, you'll still appreciate the density — when you're browsing thousands of stories, less padding means more content on screen.
18|
19|> **💡 Key Concept — Why Two Modes?**
20|>
21|> Building two complete UIs in one app might sound wasteful, but it's a deliberate trade-off. The fanfiction community is deeply attached to AO3's visual language. A modern redesign loses users who associate that look with trust and usability. Keeping both modes lets FicHub attract new users with a polished modern feel *and* retain power users who want the classic archive experience.
22|
23|### What We Copied from AO3 — and What We Changed
24|
25|The Archive interface borrows several design patterns from AO3:
26|
27|- **Two-row header** — logo on top, red navigation bar below
28|- **Work blurbs** — compact list items showing title, author, word count, tags, and summary in a single row
29|- **Tag soup** — freeform tags displayed as inline pills with category coloring
30|- **Sidebar search filters** — collapsible fieldsets for fandom, character, rating, completion status, etc.
31|- **Dense footer** — four-column link grid on a maroon background
32|
33|What we *diverged* on:
34|
35|- **Chips feature** — personalized "commonly added" tag suggestions above search results (AO3 has nothing like this)
36|- **Two-mode search form** — the same component renders as either a full page or a collapsible sidebar, depending on context
37|- **URL-driven UI switching** — you can force Archive or Modern mode by adding `?ui=archive` or `?ui=modern` to any URL
38|- **Theme presets** — Archive Classic (light) and Archive Noir (dark) with CSS custom properties
39|
40|### How Users Switch Between Modes
41|
42|There are **four ways** a user can switch their interface:
43|
44|1. **Settings page** — Navigate to Settings → Interface Style, pick Archive or Modern
45|2. **Footer link** — In Archive mode, the footer says "Switch to modern interface"; in Modern mode, it says "Prefer the AO3 look? Switch interface"
46|3. **User dropdown menu** — Click your username, select the switch option
47|4. **URL parameter** — Add `?ui=archive` or `?ui=modern` to any page URL
48|
49|The URL parameter is especially useful for sharing: you can send someone a link like `https://fichub.net/search?q=harry+potter&ui=archive` and they'll see results in Archive mode regardless of their saved preference.
50|
51|> **🧪 Try It Yourself**
52|>
53|> Open FicHub in your browser, then add `?ui=modern` to the URL. The page reloads in Modern mode. Now add `?ui=archive` and it flips back. Your preference is saved to localStorage automatically — visit any other page and it sticks.
54|
55|---
56|
57|## Chapter 2: Project Structure
58|
59|Now that you know *what* the Archive interface is, let's look at *where* it lives in the codebase.
60|
61|### Where Archive Code Lives
62|
63|All Archive-specific components live under a single directory:
64|
65|```
66|frontend/src/lib/ui/archive/
67|```
68|
69|This is inside SvelteKit's `$lib` directory — the convention for library code that isn't a route page. The `ui/archive/` subdirectory keeps everything cleanly separated from the rest of the application.
70|
71|### The Full File Inventory
72|
73|There are **17 files** in total. Here they are, grouped by type:
74|
75|#### Svelte Components (14 files)
76|
77|| File | Lines | Purpose |
78||------|-------|---------|
79|| `ArchiveWorkSearchForm.svelte` | 853 | The main search form — full page and sidebar modes |
80|| `ArchiveHome.svelte` | 470 | Home page content for Archive mode |
81|| `ArchiveFilters.svelte` | 456 | Sidebar filter panel with collapsible fieldsets |
82|| `ArchiveWork.svelte` | 407 | Individual work detail page |
83|| `ArchiveHeader.svelte` | 347 | Two-row header (logo + red navbar) |
84|| `ArchiveSearch.svelte` | 308 | Search results page layout |
85|| `ArchiveListPage.svelte` | 216 | Generic list page wrapper |
86|| `WorkBlurb.svelte` | 208 | Compact work listing row |
87|| `ArchiveFooter.svelte` | 124 | Four-column maroon footer |
88|| `TagSoup.svelte` | 117 | Inline tag pills with category coloring |
89|| `ArchiveLayout.svelte` | 63 | Shell: wraps header + content + footer |
90|| `ArchiveButton.svelte` | 58 | Polymorphic button/link component |
91|| `StatsLine.svelte` | 52 | Word count, chapters, kudos stats row |
92|| `ArchiveNavLink.svelte` | 49 | Navigation link with active state indicator |
93|
94|#### TypeScript Modules (2 files)
95|
96|| File | Lines | Purpose |
97||------|-------|---------|
98|| `searchForm.ts` | 480 | Search form state management, query building, field parsing |
99|| `rating.ts` | 118 | Rating display logic (G, T, M, E ratings with icons) |
100|
101|#### Test Files (1 file)
102|
103|| File | Lines | Purpose |
104||------|-------|---------|
105|| `searchForm.test.ts` | 418 | Unit tests for search form parsing and query building |
106|
107|**Total: 4,744 lines of code** — that's a substantial codebase for a single interface mode.
108|
109|### How Archive Components Relate to Route Pages
110|
111|Here's a key thing to understand: **Archive components are not routes themselves**. They're UI components that get *rendered by* the same SvelteKit routes that power Modern mode. The switching happens at the root layout level (more on this in Chapter 3).
112|
113|The relationship looks like this:
114|
115|```
116|Route: /search
117|  └─ Modern mode: renders ModernSearch.svelte
118|  └─ Archive mode: renders ArchiveSearch.svelte (which uses ArchiveWorkSearchForm)
119|
120|Route: /work/[id]
121|  └─ Modern mode: renders ModernWork.svelte
122|  └─ Archive mode: renders ArchiveWork.svelte
123|```
124|
125|This means you'll never find an `archive/search/+page.svelte` route file. The routing is shared — only the *rendering* switches.
126|
127|### The Shared Layer
128|
129|Archive components don't exist in a vacuum. They depend on shared infrastructure:
130|
131|- **API clients** (`$lib/api/`) — HTTP calls to the FicHub backend
132|- **Auth store** (`$lib/stores/auth.svelte`) — reactive authentication state
133|- **Prefs store** (`$lib/prefs`) — user preferences including `uiMode`
134|- **Theme system** (`$lib/themes/`) — CSS custom property management
135|- **i18n** (`$lib/i18n/`) — internationalization for multi-language support
136|
137|Every Archive component imports from this shared layer. The Archive code is *pure UI* — it contains no API logic, no auth logic, no persistence logic. That's all handled by the shared modules.
138|
139|> **⚠️ Watch Out**
140|>
141|> When working on Archive components, be careful not to accidentally import from Modern-mode-specific modules. The components in `$lib/ui/archive/` should only depend on shared infrastructure, not on components in `$lib/components/` (those are Modern-mode-specific). The one exception is `CommandPalette.svelte` and `HelpModal.svelte`, which are shared modals imported into both layouts.
142|
143|---
144|
145|## Chapter 3: The Dual-Shell Architecture
146|
147|This is the most important architectural concept in the entire codebase. If you understand how the two shells work, everything else falls into place.
148|
149|### The Root Layout: Where the Split Happens
150|
151|The entry point is `frontend/src/routes/+layout.svelte`. This is SvelteKit's root layout — it wraps every page in the application. Here's the critical decision it makes:
152|
153|```svelte
154|{#if uiMode === 'archive'}
155|  <ArchiveLayout>
156|    {@render children()}
157|  </ArchiveLayout>
158|{:else}
159|  <div class="app">
160|    <!-- Modern mode header, nav, content, footer -->
161|    ...
162|  </div>
163|{/if}
164|```
165|
166|That's it. One `{#if}` block. The entire interface switch happens right here at the top level.
167|
168|When `uiMode` is `'archive'`, the root layout wraps all page content in `<ArchiveLayout>`. When it's `'modern'`, it renders the standard Modern shell with its own header, navigation, and footer.
169|
170|### How the Mode Is Determined
171|
172|The `uiMode` variable comes from the prefs store:
173|
174|```svelte
175|<script lang="ts">
176|  import { getPref, setPref, applyUiParam } from '$lib/prefs';
177|
178|  let uiMode = $derived(getPref('uiMode'));
179|
180|  onMount(async () => {
181|    applyUiParam($page.url);
182|    applyTheme(loadTheme());
183|    // ... auth init, locale init
184|  });
185|</script>
186|```
187|
188|Let's break this down:
189|
190|1. **`getPref('uiMode')`** reads the user's saved preference from localStorage
191|2. **`$derived(...)`** makes it reactive — if the preference changes, Svelte re-renders
192|3. **`applyUiParam($page.url)`** checks for `?ui=archive` or `?ui=modern` in the URL and updates the pref if found
193|
194|### The applyUiParam() Function
195|
196|This is defined in `frontend/src/lib/prefs.ts` (line 73):
197|
198|```typescript
199|export function applyUiParam(url: URL): void {
200|  const ui = url.searchParams.get('ui');
201|  if (ui === 'archive' || ui === 'modern') {
202|    setPref('uiMode', ui);
203|  }
204|}
205|```
206|
207|It runs on every navigation. If someone visits `/search?q=hello&ui=modern`, the `ui` parameter gets parsed, the pref is updated to `'modern'`, and the layout re-renders in Modern mode. Simple, deterministic, no surprises.
208|
209|> **💡 Key Concept — URL-First Design**
210|>
211|> The `?ui=` parameter acts as a "URL override" that takes precedence over the saved preference. This is powerful for sharing links: you can force someone into a specific mode regardless of their settings. It's also useful for testing — developers can flip modes without opening Settings.
212|
213|### How the Modern Shell Stays 100% Intact
214|
215|Here's the elegant part: **Modern mode code has no idea Archive mode exists**. The `{:else}` branch in the root layout renders the complete Modern shell — its own header (`<header class="topbar">`), its own navigation (`<nav class="main-nav">`), its own footer (`<footer class="footer muted">`). None of those components import anything from `$lib/ui/archive/`.
216|
217|The only Modern-mode code that references Archive is the `switchToArchive()` function in the root layout:
218|
219|```typescript
220|function switchToArchive() {
221|  setPref('uiMode', 'archive');
222|  window.location.reload();
223|}
224|```
225|
226|And this is called from the footer button: "Prefer the AO3 look? Switch interface." That's the *only* bridge from Modern to Archive.
227|
228|Similarly, `ArchiveFooter.svelte` has the reverse:
229|
230|```svelte
231|<script lang="ts">
232|  import { setPref } from '$lib/prefs';
233|
234|  function switchToModern() {
235|    setPref('uiMode', 'modern');
236|    window.location.reload();
237|  }
238|</script>
239|```
240|
241|Both shells call `window.location.reload()` after switching. This is intentional — a full page reload ensures CSS, state, and component trees are cleanly reset. It's not elegant, but it's *reliable*.
242|
243|### The Preferences Store
244|
245|The entire mode-switching system rests on `frontend/src/lib/prefs.ts` — a tiny 78-line file. Let's examine it:
246|
247|```typescript
248|export interface UserPrefs {
249|  defaultFormat?: string;
250|  hideRead?: boolean;
251|  hideBookmarked?: boolean;
252|  libraryOnly?: boolean;
253|  readerTheme?: 'light' | 'sepia' | 'dark';
254|  uiMode?: 'archive' | 'modern';
255|}
256|
257|const STORAGE_KEY = 'fichub_prefs_v1';
258|
259|const DEFAULTS: UserPrefs = {
260|  defaultFormat: 'epub',
261|  hideRead: false,
262|  hideBookmarked: false,
263|  libraryOnly: false,
264|  readerTheme: 'light',
265|  uiMode: 'archive',    // ← Archive is the default!
266|};
267|```
268|
269|Notice that `uiMode` defaults to `'archive'`. This means new users — anyone who has never explicitly set a preference — land on the Archive interface.
270|
271|The store uses a **single JSON blob** in localStorage under the key `fichub_prefs_v1`. Every field is optional, so old or partial blobs degrade gracefully:
272|
273|```typescript
274|export function loadPrefs(): UserPrefs {
275|  try {
276|    const raw = localStorage.getItem(STORAGE_KEY);
277|    if (!raw) return { ...DEFAULTS };
278|    const parsed = JSON.parse(raw) as UserPrefs;
279|    return { ...DEFAULTS, ...parsed };
280|  } catch {
281|    return { ...DEFAULTS };
282|  }
283|}
284|```
285|
286|The spread pattern `{ ...DEFAULTS, ...parsed }` means: start with all defaults, then override with whatever the user has saved. If a key is missing from the saved blob, it falls back to the default. If the JSON is corrupt, you get clean defaults. If localStorage is unavailable (private browsing, storage full), you get clean defaults. **The store never throws.**
287|
288|> **🧪 Try It Yourself**
289|>
290|> Open your browser's developer console on FicHub and run:
291|>
292|> ```javascript
293|> JSON.parse(localStorage.getItem('fichub_prefs_v1'))
294|> ```
295|>
296|> You'll see your saved preferences. Now try:
297|>
298|> ```javascript
299|> localStorage.removeItem('fichub_prefs_v1');
300|> location.reload();
301|> ```
302|>
303|> The page reloads with Archive mode (the default) since you cleared your preferences.
304|
305|---
306|
307|## Chapter 4: Design Tokens & Theme System
308|
309|Now let's talk about how the Archive interface actually *looks*. The visual identity is controlled by CSS custom properties — also known as CSS variables — which are fed by TypeScript "theme tokens."
310|
311|### CSS Custom Properties: The Visual Language
312|
313|Throughout the Archive components, you'll see references like `var(--archive-bg, #ffffff)`. These are CSS custom properties with fallback values. The key variables used across the archive shell are:
314|
315|| Variable | Purpose | Default (Light) |
316||----------|---------|-----------------|
317|| `--archive-bg` | Page background | `#ffffff` |
318|| `--archive-text` | Primary text color | `#2a2a2a` |
319|| `--archive-muted` | Secondary/muted text | `#666666` |
320|| `--archive-link` | Link color | `#990000` |
321|| `--archive-link-visited` | Visited link color | `#660066` |
322|| `--archive-border` | Border color | `#dddddd` |
323|| `--archive-bg-raised` | Elevated surface (cards, dropdowns) | `#f5f5f5` |
324|| `--archive-accent-line` | Navbar/footer accent color | `#990000` |
325|
326|These variables are set on the root element by the theme system. Components reference them with inline fallbacks, so even if a variable is missing, the component still renders correctly.
327|
328|Look at how `ArchiveLayout.svelte` applies them:
329|
330|```svelte
331|<style>
332|  .archive-shell {
333|    min-height: 100vh;
334|    display: flex;
335|    flex-direction: column;
336|    background: var(--archive-bg, #ffffff);
337|    color: var(--archive-text, #2a2a2a);
338|    font-family: Georgia, 'Times New Roman', serif;
339|  }
340|
341|  :global(.archive-shell a) {
342|    color: var(--archive-link, #990000);
343|  }
344|
345|  :global(.archive-shell a:visited) {
346|    color: var(--archive-link-visited, #660066);
347|  }
348|</style>
349|```
350|
351|The `:global(.archive-shell a)` selector is important — it applies the link color to *all* anchor tags inside the archive shell, not just ones directly inside `ArchiveLayout`. This is how the entire interface gets consistent link styling without touching every component.
352|
353|### Two Archive Themes
354|
355|FicHub ships two Archive-specific theme presets: **Archive Classic** (light) and **Archive Noir** (dark). These are defined in `frontend/src/lib/themes/presets.ts`:
356|
357|```typescript
358|export const archiveClassic: ThemeTokens = {
359|  name: 'Archive Classic',
360|  accent: '#990000',
361|  bg: '#ffffff',
362|  surface: '#f5f5f5',
363|  text: '#2a2a2a',
364|  muted: '#666666',
365|  radius: '2px',
366|  font: "Georgia, 'Times New Roman', serif",
367|  density: 'compact',
368|  readerFont: "Georgia, 'Times New Roman', serif",
369|  readerWidth: '680px',
370|  readerLineHeight: '1.8',
371|};
372|
373|export const archiveNoir: ThemeTokens = {
374|  name: 'Archive Noir',
375|  accent: '#990000',
376|  bg: '#1e1e1e',
377|  surface: '#262626',
378|  text: '#e0e0e0',
379|  muted: '#999999',
380|  radius: '2px',
381|  font: "Georgia, 'Times New Roman', serif",
382|  density: 'compact',
383|  readerFont: "Georgia, 'Times New Roman', serif",
384|  readerWidth: '680px',
385|  readerLineHeight: '1.8',
386|};
387|```
388|
389|Both share the same maroon accent (`#990000`), the same Georgia serif font, the same tight 2px radius, and the same compact density. The only differences are background and text colors — light vs. dark.
390|
391|### The ThemeTokens Interface
392|
393|Every theme preset implements the `ThemeTokens` interface:
394|
395|```typescript
396|export interface ThemeTokens {
397|  name: string;
398|  accent: string;
399|  bg: string;
400|  surface: string;
401|  text: string;
402|  muted: string;
403|  radius: string;
404|  font: string;
405|  density: 'compact' | 'comfortable' | 'spacious';
406|  readerFont: string;
407|  readerWidth: string;
408|  readerLineHeight: string;
409|}
410|```
411|
412|This interface is the contract. Each field maps to a CSS custom property when the theme is applied. The `density` field doesn't map to a single CSS variable — it's used programmatically to adjust spacing scales across components.
413|
414|### The Design DNA
415|
416|Let's look at what makes the Archive theme *feel* like an archive:
417|
418|1. **Georgia serif headings** — `font: "Georgia, 'Times New Roman', serif"`. This is the single most distinctive visual choice. The moment you see serif text, you know you're in Archive mode.
419|
420|2. **Maroon accent** — `accent: '#990000'`. This is a deep, scholarly red that AO3 uses for its brand. It appears in links, the navbar, and the footer.
421|
422|3. **2px radius** — `radius: '2px'`. Almost no rounding at all. Cards, buttons, and inputs have sharp corners. This creates a "no-nonsense, information-first" feel.
423|
424|4. **Compact density** — `density: 'compact'`. Less padding, tighter line heights, more content per screen.
425|
426|5. **Dark visited links** — `#660066` (a deep purple). This is borrowed directly from AO3 and signals "you've been here before" in a visually distinct way.
427|
428|### How Themes Are Applied
429|
430|The theme system has two parts:
431|
432|1. **Theme definition** (`presets.ts`) — TypeScript objects with token values
433|2. **Theme application** (`apply.js`) — reads the current theme and sets CSS custom properties on the root element
434|
435|When the page loads, `applyTheme(loadTheme())` is called in the root layout's `onMount`. This reads the saved theme from localStorage, finds the matching preset, and sets CSS variables. The Archive components then pick up those variables via `var(--archive-*)` references.
436|
437|> **⚠️ Watch Out**
438|>
439|> The Archive components hard-code fallback values inline (e.g., `var(--archive-bg, #ffffff)`). This means they always render correctly even if the theme system fails. But it also means if you change a theme preset, you need to make sure the *variable names* stay consistent across all components. The fallback values are safety nets, not the source of truth.
440|
441|### All Seven Presets
442|
443|While we focused on the Archive presets, FicHub actually ships **seven** theme presets total:
444|
445|| Preset | Slug | Type |
446||--------|------|------|
447|| Default Dark | `default-dark` | Modern |
448|| Default Light | `default-light` | Modern |
449|| High Contrast | `high-contrast` | Accessibility |
450|| Sepia | `sepia` | Reader-focused |
451|| Dyslexia Friendly | `dyslexia` | Accessibility |
452|| **Archive Classic** | `archive-classic` | Archive |
453|| **Archive Noir** | `archive-noir` | Archive |
454|
455|The presets are indexed by slug in a single record:
456|
457|```typescript
458|export const presets: Record<string, ThemeTokens> = {
459|  'default-dark': defaultDark,
460|  'default-light': defaultLight,
461|  'high-contrast': highContrast,
462|  'sepia': sepia,
463|  'dyslexia': dyslexia,
464|  'archive-classic': archiveClassic,
465|  'archive-noir': archiveNoir,
466|};
467|```
468|
469|This makes it trivial to add a new theme — just create a `ThemeTokens` object and add it to the record.
470|
471|> **🧪 Try It Yourself**
472|>
473|> Open FicHub, go to Settings → Theme, and switch between Archive Classic and Archive Noir. Watch how the entire interface recolors instantly — the header, links, borders, and content all update. Then open dev tools and inspect the `<html>` element to see the CSS custom properties changing.
474|>
475|> Now try this: search for `--archive` in your dev tools' Styles panel. You'll see every component that references these variables, giving you a complete map of the visual system.
476|
477|---
478|
479|*End of Part 1. In Part 2, we'll dive into the Archive components themselves — starting with the layout shell and header, then working through the work blurbs, tag soup, and search system.*
480|
1|# Part 2: Shell Components
2|
3|## Chapter 5: ArchiveLayout — The Wrapper
4|
5|Every great interface needs a frame. In the Archive mode, that frame is `ArchiveLayout.svelte` — a 63-line component that lives at `frontend/src/lib/ui/archive/ArchiveLayout.svelte`. It's deceptively simple: it wraps the header, renders the page content, and drops in the footer. But it also does something subtle and important — it's the *only* Archive component that bootstraps three shared systems: themes, i18n, and auth.
6|
7|Here's the full component:
8|
9|```svelte
10|<script lang="ts">
11|  import type { Snippet } from 'svelte';
12|  import ArchiveHeader from './ArchiveHeader.svelte';
13|  import ArchiveFooter from './ArchiveFooter.svelte';
14|  import CommandPalette from '$lib/components/CommandPalette.svelte';
15|  import HelpModal from '$lib/components/HelpModal.svelte';
16|  import { onMount } from 'svelte';
17|  import { applyTheme, loadTheme } from '$lib/themes/apply.js';
18|  import { initI18n } from '$lib/i18n/index.svelte';
19|  import { auth } from '$lib/stores/auth.svelte';
20|
21|  let { children }: { children: Snippet } = $props();
22|
23|  onMount(() => {
24|    applyTheme(loadTheme());
25|    initI18n({ userLocale: auth.user?.locale ?? null });
26|    auth.init();
27|  });
28|</script>
29|
30|<div class="archive-shell">
31|  <ArchiveHeader />
32|  <main class="archive-main">
33|    {@render children()}
34|  </main>
35|  <ArchiveFooter />
36|</div>
37|
38|<HelpModal />
39|<CommandPalette />
40|
41|<style>
42|  .archive-shell {
43|    min-height: 100vh;
44|    display: flex;
45|    flex-direction: column;
46|    background: var(--archive-bg, #ffffff);
47|    color: var(--archive-text, #2a2a2a);
48|    font-family: Georgia, 'Times New Roman', serif;
49|  }
50|
51|  .archive-main {
52|    flex: 1;
53|    max-width: 1100px;
54|    margin: 0 auto;
55|    width: 100%;
56|    padding: 1rem;
57|  }
58|
59|  :global(.archive-shell a) {
60|    color: var(--archive-link, #990000);
61|  }
62|
63|  :global(.archive-shell a:visited) {
64|    color: var(--archive-link-visited, #660066);
65|  }
66|
67|  :global(.archive-shell a:hover) {
68|    text-decoration: underline;
69|  }
70|</style>
71|```
72|
73|### The onMount Bootstrap Sequence
74|
75|The `onMount` callback in `ArchiveLayout` is the closest thing the Archive shell has to an "entry point." It runs once when the layout first renders, and it initializes three systems in sequence:
76|
77|1. **`applyTheme(loadTheme())`** — reads the saved theme from localStorage and pushes the CSS custom properties onto `document.documentElement`. This is where `--archive-bg`, `--archive-link`, and friends get their values. Without this, components would fall back to their hard-coded hex colors (like `#990000` for maroon).
78|2. **`initI18n({ userLocale: auth.user?.locale ?? null })`** — resolves the initial locale using the priority chain: localStorage → user profile → browser language. See Chapter 6's sidebar note for the full cascade.
79|3. **`auth.init()`** — calls `GET /api/auth/me` to check if there's a valid session. If the network fails, it falls back to a cached user in localStorage.
80|
81|> **⚠️ Watch Out**
82|>
83|> These three calls happen in a specific order for a reason. `applyTheme` must run before any component renders its styles, `initI18n` must run before any `t()` calls resolve translated strings, and `auth.init()` must complete before the header can show the right "Hi, username!" greeting. Since they're all inside `onMount`, the initial render happens with default values, then Svelte re-renders once these systems resolve. That's fine for a static site — users won't notice the flash.
84|
85|### The `{@render children()}` Pattern
86|
87|ArchiveLayout uses Svelte 5's snippet-based slot system. The `children` prop is typed as `Snippet`, and the markup renders it with `{@render children()}`:
88|
89|```svelte
90|<main class="archive-main">
91|  {@render children()}
92|</main>
93|```
94|
95|This is the Svelte 5 replacement for `<slot />`. The root layout (`src/routes/+layout.svelte`) passes page content into this slot:
96|
97|```svelte
98|{#if uiMode === 'archive'}
99|  <ArchiveLayout>
100|    {@render children()}
101|  </ArchiveLayout>
102|{:else}
103|  <!-- Modern mode -->
104|{/if}
105|```
106|
107|The `children` snippet is whatever the matched route page component renders. For `/`, that's `ArchiveHome`. For `/search`, that's `ArchiveSearch`. ArchiveLayout doesn't know or care what it is — it just provides the header, main container, and footer scaffolding.
108|
109|> **💡 Key Concept — The Shell Pattern**
110|>
111|> ArchiveLayout is a "shell" component: it provides chrome (header, footer), layout constraints (max-width, padding, flexbox), and global style scoping — then gets out of the way. This is the same pattern AO3 uses (every page shares the same two-row header and maroon footer). By centralizing these concerns in one place, individual page components like `ArchiveHome` or `ArchiveWork` can focus purely on their content.
112|
113|### Global Link Styling via `:global()`
114|
115|Notice the `:global(.archive-shell a)` selectors. Svelte scopes component styles by default, but `:global()` punches through that scoping to apply styles to *all* descendant links, regardless of which component renders them:
116|
117|```css
118|:global(.archive-shell a) {
119|  color: var(--archive-link, #990000);
120|}
121|
122|:global(.archive-shell a:visited) {
123|  color: var(--archive-link-visited, #660066);
124|}
125|
126|:global(.archive-shell a:hover) {
127|  text-decoration: underline;
128|}
129|```
130|
131|This is how every link in the entire Archive interface gets consistent maroon coloring without every component (WorkBlurb, TagSoup, ArchiveNavLink, etc.) needing its own link CSS. The `visited` selector uses the purple `#660066` — a direct AO3 nod that tells users "you've been here before" in a visually distinct way.
132|
133|### Re-homing CommandPalette and HelpModal
134|
135|ArchiveLayout also renders two shared modal components directly in its template:
136|
137|```svelte
138|<HelpModal />
139|<CommandPalette />
140|```
141|
142|These aren't Archive-specific — they're imported from `$lib/components/`, the shared component directory used by both modes. `CommandPalette.svelte` provides the Ctrl+K quick-jumper (see `frontend/src/lib/components/CommandPalette.svelte`), and `HelpModal.svelte` renders the in-app documentation modal (driven by `$lib/stores/doc-help.svelte`). Both are always-mounted singletons: they render conditionally based on internal state (`open` for CommandPalette, `docHelpState.active` for HelpModal), so including them in the layout is free — no DOM overhead until the user activates them.
143|
144|> **🧪 Try It Yourself**
145|>
146|> Open the Archive mode in your browser and press `Ctrl+K` (or `Cmd+K` on Mac). The command palette slides into view. Now click a Docs entry — the HelpModal opens with the requested documentation section. Both of these are hosted by ArchiveLayout, invisible until you summon them.
147|
148|---
149|
150|## Chapter 6: ArchiveHeader — The AO3 Parity Header
151|
152|If ArchiveLayout is the frame, `ArchiveHeader.svelte` is the face. At 347 lines (in `frontend/src/lib/ui/archive/ArchiveHeader.svelte`), it implements AO3's signature two-row header: a top row with the logo and user menu, and a red navigation bar below it. There are no emojis, no rounded corners, no shadows — just dense, functional links.
153|
154|### The Structure
155|
156|The header is split into two rows:
157|
158|```svelte
159|<header class="archive-header">
160|  <!-- ROW 1: Logo + user area -->
161|  <div class="archive-top-row">
162|    <div class="archive-top-inner">
163|      <a href="/" class="archive-brand" aria-label="FicHub home">
164|        FicHub<sup class="archive-brand-sup">archive</sup>
165|      </a>
166|      <div class="archive-top-right">
167|        {#if auth.isLoggedIn}
168|          <!-- logged-in: greeting, Post, notifications, Log Out -->
169|        {:else}
170|          <!-- logged-out: Log In -->
171|        {/if}
172|      </div>
173|    </div>
174|  </div>
175|
176|  <!-- ROW 2: Red navbar -->
177|  <nav class="archive-navbar" aria-label="Primary navigation">
178|    ...
179|  </nav>
180|</header>
181|```
182|
183|### Row 1: Logo and User State
184|
185|The left side of Row 1 is the brand — a simple link with styled typography:
186|
187|```svelte
188|<a href="/" class="archive-brand" aria-label="FicHub home">
189|  FicHub<sup class="archive-brand-sup">archive</sup>
190|</a>
191|```
192|
193|The CSS is deliberately AO3-like:
194|
195|```css
196|.archive-brand {
197|  font-family: Georgia, 'Times New Roman', serif;
198|  font-size: 1.55rem;
199|  font-weight: 700;
200|  color: #990000;
201|  text-decoration: none;
202|  line-height: 1;
203|  white-space: nowrap;
204|}
205|
206|.archive-brand:hover {
207|  color: #660066;
208|}
209|
210|.archive-brand-sup {
211|  font-size: 0.45em;
212|  font-weight: 400;
213|  vertical-align: super;
214|  margin-left: 0.15em;
215|  color: var(--archive-muted, #666666);
216|}
217|```
218|
219|The `<sup>` element gives "archive" the small superscript treatment, turning "FicHub" + "archive" into "FicHub<sup>archive</sup>". On hover, the brand color darkens from `#990000` to `#660066` (the visited-link purple).
220|
221|The right side of Row 1 is reactive — it reads from the `auth` store:
222|
223|```svelte
224|{#if auth.isLoggedIn}
225|  <span class="archive-hello">Hi, {auth.username ?? 'there'}!</span>
226|  <a class="archive-top-link" href="/requests/new">Post</a>
227|  <NotificationBell />
228|  <button
229|    class="archive-logout-btn"
230|    type="button"
231|    onclick={() => auth.handleLogout()}
232|  >
233|    Log Out
234|  </button>
235|{:else}
236|  <a class="archive-top-link" href="/login">Log In</a>
237|{/if}
238|```
239|
240|The auth store (`frontend/src/lib/stores/auth.svelte.ts`) is a Svelte 5 class-based store using `$state` runes. When `auth.init()` resolves (called from ArchiveLayout's `onMount`), the `user` property updates, `isLoggedIn` flips to `true`, and the header re-renders instantly — no page reload needed.
241|
242|The `NotificationBell` component (`frontend/src/lib/components/NotificationBell.svelte`) is a shared Modern-mode component that polls `/api/notifications/unread-count` every 60 seconds to show a badge. It's imported directly into the header because it's useful in both modes.
243|
244|### Row 2: The Red Navbar
245|
246|Row 2 is the iconic red bar. It uses `--archive-accent-line` as its background:
247|
248|```css
249|.archive-navbar {
250|  background: var(--archive-accent-line, #990000);
251|}
252|```
253|
254|The navbar has two clusters — left and right:
255|
256|**Left cluster** (primary navigation):
257|- Fandoms → links to `/fandoms`
258|- Browse → links to `/tags`
259|- Search ▾ (dropdown) → links to Works, Bookmarks, Tags, People
260|- About ▾ (dropdown) → links to Docs, Roadmap, Modlog, and an inert "Tropes" item
261|
262|**Right cluster**:
263|- Requests → `/requests`
264|- Forum → `/forum`
265|- Ask → `/ask`
266|- Inline search input → submits to `/search?q=...`
267|
268|### Keyboard-Accessible Dropdowns with `<details>`
269|
270|AO3's dropdowns use JavaScript. FicHub's Archive mode uses native HTML `<details>` and `<summary>` elements — no JS required, fully keyboard-navigable, and screen-reader friendly:
271|
272|```svelte
273|<details class="archive-dropdown">
274|  <summary class="archive-nav-link archive-dropdown-trigger">Search &#9662;</summary>
275|  <div class="archive-dropdown-menu" role="menu">
276|    <a class="archive-dd-item" href="/search" role="menuitem">Works</a>
277|    <a class="archive-dd-item" href="/bookmarks" role="menuitem">Bookmarks</a>
278|    <a class="archive-dd-item" href="/tags" role="menuitem">Tags</a>
279|    <a class="archive-dd-item" href="/authors" role="menuitem">People</a>
280|  </div>
281|</details>
282|```
283|
284|The `&#9662;` is a down-chevron character entity. The CSS hides the default `<summary>` marker:
285|
286|```css
287|.archive-dropdown-trigger::-webkit-details-marker {
288|  display: none;
289|}
290|
291|.archive-dropdown-trigger::marker {
292|  content: '';
293|}
294|```
295|
296|And styles the trigger to look like a regular nav link:
297|
298|```css
299|.archive-dropdown-trigger {
300|  cursor: pointer;
301|  list-style: none;
302|  user-select: none;
303|}
304|
305|.archive-dropdown[open] > .archive-dropdown-trigger {
306|  background: rgba(255, 255, 255, 0.15);
307|}
308|```
309|
310|The dropdown menu itself is positioned absolutely:
311|
312|```css
313|.archive-dropdown-menu {
314|  position: absolute;
315|  top: 100%;
316|  left: 0;
317|  min-width: 160px;
318|  background: var(--archive-bg, #ffffff);
319|  border: 1px solid var(--archive-border, #dddddd);
320|  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
321|  padding: 0.25rem 0;
322|  z-index: 80;
323|}
324|```
325|
326|> **⚠️ Watch Out**
327|>
328|> Using `<details>` for dropdowns means the open/close state is managed by the browser, not by Svelte reactivity. This is actually a *feature* — it means the dropdown stays open during keyboard navigation and closes automatically when focus leaves. If you need to control the open state programmatically (e.g., close all dropdowns when a form submits), you'd have to reach into the DOM with `details.open = false`.
329|
330|### The Inert "Tropes" Item
331|
332|In the About dropdown, you'll notice one item that isn't a link:
333|
334|```svelte
335|<span class="archive-dd-item archive-dd-inert" role="menuitem" aria-disabled="true">Tropes</span>
336|```
337|
338|It's a `<span>`, not an `<a>`, and it carries `aria-disabled="true"`. The CSS grays it out:
339|
340|```css
341|.archive-dd-item:hover {
342|  background: var(--archive-bg-raised, #f5f5f5);
343|}
344|
345|.archive-dd-inert {
346|  color: var(--archive-muted, #999999);
347|  cursor: default;
348|  opacity: 0.6;
349|}
350|
351|.archive-dd-inert:hover {
352|  background: none;
353|}
354|```
355|
356|This is a placeholder for a feature that hasn't shipped yet. AO3 has a Tropes section. FicHub's Archive mode acknowledges the gap but doesn't fake it — the item is explicitly inert.
357|
358|### Inline Search in the Navbar
359|
360|The search form is inline in the right cluster of the navbar:
361|
362|```svelte
363|<form class="archive-search-form" onsubmit={handleSearch} role="search">
364|  <input
365|    class="archive-search-input"
366|    type="search"
367|    name="q"
368|    placeholder="Search"
369|    aria-label="Search FicHub"
370|    bind:value={searchQuery}
371|  />
372|  <button class="archive-search-btn" type="submit">Search</button>
373|</form>
374|```
375|
376|The `handleSearch` function performs a full-page navigation:
377|
378|```typescript
379|let searchQuery = $state('');
380|
381|function handleSearch(e: Event) {
382|  e.preventDefault();
383|  if (searchQuery.trim()) {
384|    window.location.href = `/search?q=${encodeURIComponent(searchQuery.trim())}`;
385|  }
386|}
387|```
388|
389|It uses `window.location.href` instead of SvelteKit's `goto` because a full navigation is appropriate here — the search bar in the header is a global utility, and a fresh page load ensures the search page starts clean.
390|
391|### No Emojis, Period
392|
393|AO3's design rule is "no emojis." The Archive header respects this. You won't find a single `🧡` or `🔍` in the template. Even the notification bell is rendered as an inline SVG, not an emoji. The only decorative characters are CSS-inserted chevrons (`&#9662;` for dropdown triggers) and the brand superscript.
394|
395|### Responsive Behavior
396|
397|On screens narrower than 767px, the navbar wraps its items onto multiple lines:
398|
399|```css
400|@media (max-width: 767px) {
401|  .archive-navbar-inner {
402|    flex-wrap: wrap;
403|    padding: 0 0.75rem;
404|    gap: 0;
405|  }
406|
407|  .archive-nav-right {
408|    width: 100%;
409|    padding-top: 0.25rem;
410|    border-top: 1px solid rgba(255, 255, 255, 0.2);
411|    margin-top: 0.25rem;
412|  }
413|
414|  .archive-search-input {
415|    width: 100px;
416|  }
417|}
418|```
419|
420|The right cluster (Requests, Forum, Ask, Search) drops below the left cluster, and the search input shrinks to 100px to fit. This is a pragmatic compromise — AO3 uses a hamburger menu on mobile, but FicHub's Archive mode keeps all links visible.
421|
422|> **💡 Key Concept — Native HTML Over Framework**
423|>
424|> The dropdowns, the search form, the login/logout toggle — none of these require JavaScript to function. AO3 pioneered this "HTML-first" approach because it's robust: if your JS bundle fails to load, the nav still works. FicHub's Archive mode follows the same philosophy. The `<details>` element handles dropdowns, the `<form>` element handles search submission, and the `{#if auth.isLoggedIn}` block handles auth state — all with graceful degradation.
425|
426|> **🧪 Try It Yourself**
427|>
428|> Open Archive mode on your phone (or resize your browser to under 767px). Watch how the navbar reflows: the red bar stays the same height, but the right cluster drops to a new row. Now use your keyboard to navigate — Tab gets you into the navbar, Space opens the dropdown, and you can arrow through the menu items. No JavaScript needed.
429|
430|---
431|
432|## Chapter 7: ArchiveFooter — The Red Footer
433|
434|If the header is the face, the footer is the handshake. `ArchiveFooter.svelte` (`frontend/src/lib/ui/archive/ArchiveFooter.svelte`, 124 lines) provides a four-column link grid on the signature maroon background, plus a button to switch to Modern mode.
435|
436|### The Markup
437|
438|```svelte
439|<script lang="ts">
440|  import { setPref } from '$lib/prefs';
441|
442|  function switchToModern() {
443|    setPref('uiMode', 'modern');
444|    window.location.reload();
445|  }
446|</script>
447|
448|<footer class="archive-footer">
449|  <div class="archive-footer-columns">
450|    <!-- Column 1: Customize -->
451|    <div class="archive-footer-col">
452|      <h4 class="archive-footer-heading">Customize</h4>
453|      <a class="archive-footer-link" href="/settings">Settings</a>
454|      <a class="archive-footer-link" href="/settings">Interface style</a>
455|    </div>
456|
457|    <!-- Column 2: About -->
458|    <div class="archive-footer-col">
459|      <h4 class="archive-footer-heading">About</h4>
460|      <a class="archive-footer-link" href="/docs">Docs</a>
461|      <a class="archive-footer-link" href="/roadmap">Roadmap</a>
462|      <span class="archive-footer-link archive-footer-inert">Consensus</span>
463|    </div>
464|
465|    <!-- Column 3: Contact -->
466|    <div class="archive-footer-col">
467|      <h4 class="archive-footer-heading">Contact</h4>
468|      <a class="archive-footer-link" href="/forum">Forum</a>
469|      <a class="archive-footer-link" href="/requests">Requests</a>
470|      <a class="archive-footer-link" href="/ask">Ask</a>
471|    </div>
472|
473|    <!-- Column 4: Development -->
474|    <div class="archive-footer-col">
475|      <h4 class="archive-footer-heading">Development</h4>
476|      <a class="archive-footer-link" href="/modlog">Modlog</a>
477|      <a class="archive-footer-link" href="/opds">OPDS</a>
478|      <a class="archive-footer-link" href="/feed.xml">Atom Feed</a>
479|      <span class="archive-footer-link archive-footer-inert">API Docs</span>
480|    </div>
481|  </div>
482|
483|  <div class="archive-footer-bottom">
484|    <button type="button" class="archive-switch-btn" onclick={switchToModern}>
485|      Switch to modern interface
486|    </button>
487|  </div>
488|</footer>
489|```
490|
491|### The Switch Pattern
492|
493|The footer's switch button is the inverse of the one in Modern mode. Here's how it works:
494|
495|```typescript
496|function switchToModern() {
497|  setPref('uiMode', 'modern');
498|  window.location.reload();
499|}
500|```
501|
502|It calls `setPref('uiMode', 'modern')` from `frontend/src/lib/prefs.ts` (line 63), which writes the preference to the localStorage blob, then calls `window.location.reload()`. The full page reload is intentional — it ensures the root layout's `$derived` re-evaluates `getPref('uiMode')`, picks up the new value, and re-renders in Modern mode with a clean component tree and fresh CSS.
503|
504|> **💡 Key Concept — Full Reload vs. Soft Navigation**
505|>
506|> You might wonder why the switch uses `window.location.reload()` instead of SvelteKit's `goto('/')`. The answer: the UI mode is decided in the root `+layout.svelte`, and the entire component tree changes between modes. A soft navigation would leave stale Archive components mounted. The full reload is the nuclear option — but it's *reliable*. AO3 does the same thing when you switch interfaces.
507|
508|### The Visual Design
509|
510|The footer uses `--archive-accent-line` as its background, the same variable the navbar uses:
511|
512|```css
513|.archive-footer {
514|  background: var(--archive-accent-line, #990000);
515|  color: #fff;
516|  padding: 1.2rem 1rem 0.8rem;
517|  margin-top: auto;
518|}
519|```
520|
521|The `margin-top: auto` is important — because `.archive-shell` is a flex column with `min-height: 100vh`, `margin-top: auto` pushes the footer to the bottom of the viewport regardless of content height. That's how AO3's footer always sits at the bottom even on short pages.
522|
523|The four-column grid is pure CSS:
524|
525|```css
526|.archive-footer-columns {
527|  display: grid;
528|  grid-template-columns: repeat(4, 1fr);
529|  gap: 1rem 1.5rem;
530|  max-width: 1100px;
531|  margin: 0 auto;
532|}
533|```
534|
535|### Column Headings and Links
536|
537|Headings use the AO3 pattern of small, uppercase, letter-spaced text:
538|
539|```css
540|.archive-footer-heading {
541|  font-size: 0.78em;
542|  font-weight: 700;
543|  text-transform: uppercase;
544|  letter-spacing: 0.04em;
545|  margin: 0 0 0.3rem;
546|  color: #fff;
547|}
548|```
549|
550|Links are styled as block-level underlines:
551|
552|```css
553|.archive-footer-link {
554|  display: block;
555|  font-size: 0.78em;
556|  color: rgba(255, 255, 255, 0.85);
557|  text-decoration: underline;
558|  padding: 0.15em 0;
559|}
560|
561|.archive-footer-link:hover {
562|  color: #fff;
563|}
564|```
565|
566|### Inert Links
567|
568|Two footer items are explicitly inert — "Consensus" under About and "API Docs" under Development. They're `<span>` elements, not `<a>` tags:
569|
570|```svelte
571|<span class="archive-footer-link archive-footer-inert">Consensus</span>
572|<span class="archive-footer-link archive-footer-inert">API Docs</span>
573|```
574|
575|The CSS makes them look like links but disables interaction:
576|
577|```css
578|.archive-footer-inert {
579|  cursor: default;
580|  opacity: 0.5;
581|}
582|
583|.archive-footer-inert:hover {
584|  color: rgba(255, 255, 255, 0.85);
585|}
586|```
587|
588|They're grayed out at 50% opacity and use `cursor: default` instead of `cursor: pointer`. The hover state still lightens them to full white, but there's no link behavior.
589|
590|### The Bottom Row
591|
592|Below the four columns sits the "Switch to modern interface" button:
593|
594|```css
595|.archive-footer-bottom {
596|  max-width: 1100px;
597|  margin: 0.8rem auto 0;
598|  padding-top: 0.6rem;
599|  border-top: 1px solid rgba(255, 255, 255, 0.25);
600|  text-align: center;
601|}
602|
603|.archive-switch-btn {
604|  background: none;
605|  border: none;
606|  color: rgba(255, 255, 255, 0.8);
607|  font-size: 0.78em;
608|  text-decoration: underline;
609|  cursor: pointer;
610|  padding: 0;
611|  font-family: inherit;
612|}
613|
614|.archive-switch-btn:hover {
615|  color: #fff;
616|}
617|```
618|
619|It's a text link styled as a button — no background, no border, just underlined text that lightens on hover. This matches AO3's pattern of "switch interface" links in the footer.
620|
621|### Responsive Behavior
622|
623|The footer collapses columns based on screen width:
624|
625|```css
626|@media (max-width: 600px) {
627|  .archive-footer-columns {
628|    grid-template-columns: repeat(2, 1fr);
629|  }
630|}
631|
632|@media (max-width: 380px) {
633|  .archive-footer-columns {
634|    grid-template-columns: 1fr;
635|  }
636|}
637|```
638|
639|At 600px, four columns become two. At 380px (an iPhone SE width), they collapse to a single stack. The headings and links remain readable throughout.
640|
641|> **🧪 Try It Yourself**
642|>
643|> Resize your browser window while viewing any Archive page. At exactly 600px, the footer jumps from four columns to two. At 380px, it becomes a single column. The red background and white text stay consistent throughout — only the grid reflows. Now click "Switch to modern interface" and watch the entire page reload in Modern mode. Your preference is saved, so a hard refresh keeps you in Modern mode.
644|
645|---
646|
647|## Chapter 8: ArchiveNavLink & ArchiveButton
648|
649|The Archive interface has two reusable UI primitives: `ArchiveNavLink` (49 lines) for navigation links, and `ArchiveButton` (58 lines) for clickable actions. Both live in `frontend/src/lib/ui/archive/`. They look similar, they behave differently.
650|
651|### ArchiveNavLink — Active State via SvelteKit's `$page` Store
652|
653|```svelte
654|<script lang="ts">
655|  import type { Snippet } from 'svelte';
656|  import { page } from '$app/stores';
657|
658|  let {
659|    href,
660|    label = '',
661|    children,
662|  }: {
663|    href: string;
664|    label?: string;
665|    children?: Snippet;
666|  } = $props();
667|
668|  let active = $derived($page.url.pathname === href || $page.url.pathname.startsWith(href + '/'));
669|</script>
670|
671|<a class="archive-nav-link" class:active href="{href}">
672|  {#if children}
673|    {@render children()}
674|  {:else}
675|    {label}
676|  {/if}
677|</a>
678|
679|<style>
680|  .archive-nav-link {
681|    display: inline-block;
682|    padding: 0.3em 0.7em;
683|    font-size: 0.9em;
684|    font-weight: 600;
685|    color: var(--archive-link, #990000);
686|    text-decoration: none;
687|    border-bottom: 2px solid transparent;
688|    transition: color 0.15s, border-color 0.15s;
689|    white-space: nowrap;
690|  }
691|
692|  .archive-nav-link:hover {
693|    color: var(--archive-link-visited, #660066);
694|    text-decoration: underline;
695|  }
696|
697|  .archive-nav-link.active {
698|    color: var(--archive-text, #2a2a2a);
699|    border-bottom-color: var(--archive-accent-line, #990000);
700|    text-decoration: none;
701|  }
702|</style>
703|```
704|
705|#### The `$derived` Active Detection
706|
707|The active state uses Svelte 5's `$derived` rune with SvelteKit's `$page` store:
708|
709|```typescript
710|let active = $derived(
711|  $page.url.pathname === href
712|  || $page.url.pathname.startsWith(href + '/')
713|);
714|```
715|
716|This creates a reactive value that recalculates whenever `$page` changes (i.e., on navigation). The logic is intentionally inclusive: if your `href` is `/fandoms`, the link becomes active on both `/fandoms` and `/fandoms/123` (via the `startsWith(href + '/')` check). This means a link to `/search` in the navbar won't show as active when you're on `/search?tab=filters`, but it *will* stay active on `/search/works`.
717|
718|The `class:active` shorthand applies the `active` class to the `<a>` element when the derived value is truthy. When active, the link gets:
719|
720|- `color: var(--archive-text, #2a2a2a)` — switches from maroon to the body text color
721|- `border-bottom-color: var(--archive-accent-line, #990000)` — a 2px maroon underline appears
722|- `text-decoration: none` — kills the hover underline to avoid visual clutter
723|
724|> **💡 Key Concept — Reactive Class Binding**
725|>
726|> `class:active` is Svelte's shorthand for `class:active={active}`. When `active` is `true`, Svelte adds the `active` class to the element; when `false`, it removes it. The reactivity flows: `$page` changes → `$derived` re-evaluates → `active` flips → DOM class updates. No manual DOM manipulation, no event listeners, no manual cleanup. This is the power of Svelte 5's reactivity model.
727|
728|#### Flexible Children API
729|
730|ArchiveNavLink accepts either a `label` prop or arbitrary `children` (a Svelte 5 snippet):
731|
732|```svelte
733|<a class="archive-nav-link" class:active href="{href}">
734|  {#if children}
735|    {@render children()}
736|  {:else}
737|    {label}
738|  {/if}
739|</a>
740|```
741|
742|This means you can use it two ways:
743|
744|```svelte
745|<ArchiveNavLink href="/fandoms">Fandoms</ArchiveNavLink>
746|```
747|
748|Or with custom content:
749|
750|```svelte
751|<ArchiveNavLink href="/search">
752|  <span class="nav-icon">🔍</span> Search
753|</ArchiveNavLink>
754|```
755|
756|The `ArchiveHeader` component, however, uses raw `<a>` tags instead of `<ArchiveNavLink>` — the header's nav links are static and don't need active-state detection because the red navbar background makes them visually distinct enough. ArchiveNavLink is used in places like the "Recent Works" byline links and the ArchiveListPage heading, where active state matters.
757|
758|> **🧪 Try It Yourself**
759|>
760|> Open a page that uses ArchiveNavLink (like an Archive list page). Open your browser's developer tools and navigate between pages. Watch the `border-bottom` and `color` on the nav link change in the Elements panel — the `active` class toggles as `$page.url.pathname` updates.
761|
762|### ArchiveButton — The AO3 Button Aesthetic
763|
764|```svelte
765|<script lang="ts">
766|  import type { Snippet } from 'svelte';
767|
768|  let {
769|    href = '',
770|    onclick = undefined,
771|    disabled = false,
772|    type = 'button',
773|    children,
774|  }: {
775|    href?: string;
776|    onclick?: (e: MouseEvent) => void;
777|    disabled?: boolean;
778|    type?: 'button' | 'submit' | 'reset';
779|    children: Snippet;
780|  } = $props();
781|</script>
782|
783|{#if href && !disabled}
784|  <a class="archive-btn" href="{href}" onclick={onclick}>
785|    {@render children()}
786|  </a>
787|{:else}
788|  <button class="archive-btn" {type} {disabled} {onclick}>
789|    {@render children()}
790|  </button>
791|{/if}
792|
793|<style>
794|  .archive-btn {
795|    display: inline-block;
796|    padding: 0.25em 0.8em;
797|    font-size: 0.9em;
798|    font-family: inherit;
799|    font-weight: 600;
800|    line-height: 1.6;
801|    color: var(--archive-text, #2a2a2a);
802|    background: var(--archive-bg-raised, #f5f5f5);
803|    border: 1px solid var(--archive-border, #dddddd);
804|    border-radius: 0;
805|    cursor: pointer;
806|    text-decoration: none;
807|    text-align: center;
808|    vertical-align: middle;
809|    transition: background 0.15s, border-color 0.15s;
810|  }
811|
812|  .archive-btn:hover {
813|    background: var(--archive-border, #dddddd);
814|    border-color: var(--archive-muted, #666666);
815|    text-decoration: none;
816|  }
817|
818|  .archive-btn:disabled {
819|    opacity: 0.5;
820|    cursor: not-allowed;
821|  }
822|</style>
823|```
824|
825|#### Polymorphic Rendering: `<a>` or `<button>`
826|
827|ArchiveButton is a *polymorphic* component — it renders either an `<a>` or a `<button>` depending on whether the `href` prop is set:
828|
829|```svelte
830|{#if href && !disabled}
831|  <a class="archive-btn" href="{href}" onclick={onclick}>
832|    {@render children()}
833|  </a>
834|{:else}
835|  <button class="archive-btn" {type} {disabled} {onclick}>
836|    {@render children()}
837|  </button>
838|{/if}
839|```
840|
841|- **With `href`**: renders an `<a>` — use for navigation links that trigger page loads or route changes.
842|- **Without `href`** (or when `disabled`): renders a `<button>` — use for form submissions, modal triggers, or actions that don't navigate.
843|
844|Both share the same `archive-btn` CSS class, so they look identical. The only difference is semantic: an `<a>` in the HTML outline is navigable; a `<button>` is actionable.
845|
846|#### The AO3 Button Aesthetic
847|
848|AO3 buttons are famously flat: no shadows, no gradients, no rounded corners. ArchiveButton enforces this in CSS:
849|
850|```css
851|.archive-btn {
852|  border-radius: 0;           /* Sharp corners — no rounding */
853|  background: var(--archive-bg-raised, #f5f5f5);  /* Flat fill, no gradient */
854|  border: 1px solid var(--archive-border, #dddddd);
855|  /* No box-shadow — AO3 buttons are completely flat */
856|  transition: background 0.15s, border-color 0.15s;  /* Simple color fade */
857|}
858|```
859|
860|Compare this to the Modern mode buttons, which use `--radius-sm` (rounded corners), `box-shadow` for depth, and gradient backgrounds on hover. The Archive buttons are the *opposite* design philosophy: flat, sharp, dense.
861|
862|On hover, the button lightens its background and darkens its border:
863|
864|```css
865|.archive-btn:hover {
866|  background: var(--archive-border, #dddddd);
867|  border-color: var(--archive-muted, #666666);
868|}
869|```
870|
871|It's a subtle shift — the button doesn't "pop" or "lift." It just shifts one shade darker, like AO3's buttons do when you hover them.
872|
873|#### The `variant` Prop Discussion
874|
875|The original outline mentions a "variant prop: primary (maroon) vs secondary (gray)." Looking at the actual `ArchiveButton.svelte` source, there's no `variant` prop. Instead, the component is a single flat style. The "primary" look (maroon background) is achieved by using `ArchiveButton` with a custom class override, or by using a raw `<a>` styled directly — as seen in `ArchiveWork.svelte`'s action buttons:
876|
877|```svelte
878|<!-- In ArchiveWork.svelte, line 103: -->
879|<a class="action-btn action-read" href={`/read/${encodeURIComponent(m.id)}`}>
880|  Read Online
881|</a>
882|```
883|
884|```css
885|.action-read {
886|  color: var(--archive-bg, #ffffff);
887|  background: var(--archive-link, #990000);
888|  border-color: var(--archive-link, #990000);
889|}
890|```
891|
892|The `action-read` class overrides the background to maroon (`--archive-link`). When you need a maroon button, you style it directly. This keeps `ArchiveButton` simple: one style, one purpose.
893|
894|> **⚠️ Watch Out**
895|>
896|> ArchiveButton doesn't have a `variant` prop. If you need a primary (maroon) button, you either:
897|> 1. Add a `variant="primary"` prop and write the CSS yourself, or
898|> 2. Use a raw `<a>` or `<button>` with a custom class like `action-read`
899|>
900|> The codebase currently uses approach #2. If you add a `variant` prop, make sure it maps to the theme tokens (`--archive-link` for primary, `--archive-bg-raised` for secondary) rather than hard-coded hex values.
901|
902|#### Disabled State
903|
904|The `disabled` prop works for both render paths:
905|
906|```svelte
907|{#if href && !disabled}
908|  <a ...>
909|{:else}
910|  <button class="archive-btn" {type} {disabled} {onclick}>
911|{/if}
912|```
913|
914|When `disabled` is true and `href` is set, the component falls through to the `<button>` branch (since `href && !disabled` is false). The `:disabled` pseudo-class applies:
915|
916|```css
917|.archive-btn:disabled {
918|  opacity: 0.5;
919|  cursor: not-allowed;
920|}
921|```
922|
923|This is used in `ArchiveWorkSearchForm.svelte` for the Search button during loading states:
924|
925|```svelte
926|<ArchiveButton onclick={onSubmit}>Search</ArchiveButton>
927|```
928|
929|When the form is submitting, the parent passes `disabled={true}` and the button grays out with a "not-allowed" cursor.
930|
931|#### Real-World Usage
932|
933|ArchiveButton is used in several places across the Archive UI. In `ArchiveWorkSearchForm.svelte` (line 163 and 414), it wraps the Search button in both the page-mode and sidebar-mode layouts. In `ArchiveFilters.svelte`, it appears in the filter action bar:
934|
935|```svelte
936|<button class="archive-btn primary" onclick={handleApply} disabled={loading}>
937|  {loading ? 'Searching…' : 'Apply Filters'}
938|</button>
939|<button class="archive-btn clear" onclick={handleClear}>Clear All</button>
940|```
941|
942|Note how the filters page uses raw `<button>` elements with `class="archive-btn"` instead of the `<ArchiveButton>` component — this is because they need the `primary` and `clear` modifier classes, which `ArchiveButton` doesn't support. The `archive-btn` class is the shared styling foundation; `ArchiveButton` is just the component wrapper around it.
943|
944|### The AO3 Button DNA
945|
946|Let's recap what makes an Archive button feel like an AO3 button:
947|
948|1. **Zero border radius** (`border-radius: 0`) — sharp, boxy, utilitarian
949|2. **Flat background** — no gradients, no box-shadow, no "lift"
950|3. **Maroon hover state** — links shift to `#660066` on hover
951|4. **2px accent underline on active links** — `border-bottom: 2px solid` on `.active`
952|5. **Georgia serif font** — inherited from the parent, never overridden
953|6. **Compact padding** — `0.25em 0.8em` horizontal, `0.3em 0.7em` for nav links
954|7. **No emojis** — AO3 rule, strictly followed
955|
956|> **💡 Key Concept — Visual Consistency Through CSS Variables**
957|>
958|> Neither ArchiveNavLink nor ArchiveButton hard-codes colors. They use `var(--archive-link, #990000)` for link color, `var(--archive-bg-raised, #f5f5f5)` for button backgrounds, and `var(--archive-border, #dddddd)` for borders. This means when you switch from Archive Classic to Archive Noir, every link, button, and underline updates automatically — because the CSS variables change, not the component CSS.
959|
960|> **🧪 Try It Yourself**
961|>
962|> Navigate to a search results page in Archive mode. Click the "Apply Filters" button — watch how it uses the flat `archive-btn` style (gray background, sharp corners). Now look at the "Download" action button on a work page — it uses the `action-btn` style with a maroon background. Both share the same `border-radius: 0` and the same transition, but they communicate different hierarchy levels through color.
963|
964|---
965|
966|*End of Part 2. In Part 3, we'll dive into the content components — WorkBlurb, TagSoup, StatsLine, and the ArchiveSearch results layout that ties them all together.*
967|
1|# Part 3: Content Components
2|
3|## Chapter 9: WorkBlurb — The Fic Card
4|
5|If `ArchiveLayout` is the wrapper and `ArchiveHeader` is the face, then `WorkBlurb.svelte` is the heart of every list page in the Archive. At 208 lines (`frontend/src/lib/ui/archive/WorkBlurb.svelte`), it renders a single *fic card* — that familiar bordered box containing a story's title, author, tags, description snippet, statistics, and action buttons. Search results pages, recent work widgets, recommendation lists, bookmark feeds — they're all rows of `WorkBlurb` cards, side by side.
6|
7|This is where the three systems you met earlier (themes, i18n, auth) come together into something the user actually clicks on.
8|
9|### The Props Interface
10|
11|Every `WorkBlurb` receives a search result object and an optional flag:
12|
13|```svelte
14|<script lang="ts">
15|  import { auth } from '$lib/stores/auth.svelte';
16|  import TagSoup from './TagSoup.svelte';
17|  import StatsLine from './StatsLine.svelte';
18|  import { mapRating } from './rating.js';
19|
20|  interface FicSearchResult {
21|    author?: string | null;
22|    chapters?: number | null;
23|    comment_count?: number | null;
24|    description?: string | null;
25|    kudos_count?: number | null;
26|    rank?: number | null;
27|    rating?: string | null;
28|    snippet?: string | null;
29|    source?: string | null;
30|    status?: string | null;
31|    tags?: Array<{ category?: number | string; name: string }>;
32|    title?: string | null;
33|    total_freeform?: number | null;
34|    updated?: string | null;
35|    url_id?: string | null;
36|    words?: number | null;
37|    [key: string]: unknown;
38|  }
39|
40|  let {
41|    fic,
42|    showAllTags = false,
43|  }: {
44|    fic: FicSearchResult;
45|    showAllTags?: boolean;
46|  } = $props();
47|</script>
48|```
49|
50|The `fic` prop carries the full set of fields returned by the search API (defined in `frontend/src/lib/api/search.ts`). Most are optional — not every search result has every field — so the component uses `?? 'Untitled'` and `?? 'Anonymous'` defaults throughout.
51|
52|The `showAllTags` boolean controls whether `TagSoup` truncates or expands its tag list. By default, `TagSoup` hides freeform tags beyond six. When `showAllTags` is `true`, every tag shows. A search results page passes `false`; a dedicated "tags" view would pass `true`.
53|
54|### Derived Values and Reactive State
55|
56|Inside `<script>`, `$derived` extracts what the template needs:
57|
58|```typescript
59|const title = $derived(fic.title ?? 'Untitled');
60|const author = $derived(fic.author ?? 'Anonymous');
61|const rating = $derived(mapRating(fic.rating));
62|const isLoggedIn = $derived(auth.isLoggedIn);
63|const snippet = $derived(fic.snippet ?? fic.description ?? '');
64|```
65|
66|`mapRating()` (imported from `./rating.js`) converts a short AO3-style code like `"general"` or `"explicit"` into its human-readable form: `"General Audiences"` or `"Explicit"`. If the rating is missing, it returns `"Not Rated"`. We'll explore this function in detail in Chapter 12.
67|
68|`isLoggedIn` reads from the Svelte 5 class-based auth store. When the user logs in or out while browsing a results page, every visible `WorkBlurb` re-renders instantly because `$derived` reacts to `$state` changes. No manual subscription cleanup needed.
69|
70|Then comes the tag grouping logic — a clever `$derived.by()` block that transforms the flat tag array from the API into a shape `TagSoup` expects:
71|
72|```typescript
73|const groupedTags = $derived.by(() => {
74|  const result: Record<string, Array<{ category?: number | string; name: string }>> = {};
75|  if (!fic.tags) return result;
76|  for (const tag of fic.tags) {
77|    const key = String(tag.category ?? '4');
78|    if (!result[key]) result[key] = [];
79|    result[key].push(tag);
80|  }
81|  return result;
82|});
83|```
84|
85|The API returns tags as a flat array where each tag carries a numeric `category` (1=Fandom, 2=Character, 3=Relationship, 4=Freeform/Additional, 5=Warning, 6=Category). `groupedTags` fans them out into a dictionary keyed by category number: `{ "1": [...], "2": [...], "4": [...] }`. The fallback category `'4'` means any tag without a category gets shoved into "Additional Tags."
86|
87|A small local `truncate()` helper chops long snippets at 300 characters:
88|
89|```typescript
90|function truncate(text: string, max: number): string {
91|  if (text.length <= max) return text;
92|  return text.slice(0, max).trimEnd() + '…';
93|}
94|```
95|
96|It's defined locally inside `<script>` rather than imported from `rating.ts` because only `WorkBlurb` does snippet truncation — the other components don't need it.
97|
98|### The Template
99|
100|The HTML body of a `WorkBlurb` follows a clear top-to-bottom information hierarchy:
101|
102|```svelte
103|<article class="work-blurb">
104|  <!-- HEADER: Rating badge + Title -->
105|  <div class="blurb-header">
106|    <span class="blurb-rating">{rating}</span>
107|    <a class="blurb-title" href="/fic/{fic.url_id}">{title}</a>
108|  </div>
109|
110|  <!-- BYLINE -->
111|  <div class="blurb-byline">
112|    by <a href="/search?q=author:{encodeURIComponent(author)}">{author}</a>
113|  </div>
114|
115|  <!-- TAGS -->
116|  {#if fic.tags?.length}
117|    <TagSoup tags={groupedTags} showAll={showAllTags} />
118|  {/if}
119|
120|  <!-- SNIPPET -->
121|  {#if snippet}
122|    <div class="blurb-snippet">{truncate(snippet, 300)}</div>
123|  {/if}
124|
125|  <!-- STATS -->
126|  <StatsLine
127|    words={fic.words}
128|    chapters={fic.chapters}
129|    status={fic.status}
130|    kudos={fic.kudos_count}
131|    updated={fic.updated}
132|  />
133|
134|  <!-- ACTIONS -->
135|  <div class="blurb-actions">
136|    <a class="blurb-action" href="/download?url={encodeURIComponent(fic.source ?? '')}">
137|      Download
138|    </a>
139|    <a class="blurb-action blurb-action--primary" href="/read/{fic.url_id}">
140|      Read
141|    </a>
142|    {#if isLoggedIn}
143|      <button class="blurb-action" type="button">Bookmark</button>
144|      <button class="blurb-action" type="button">Kudos</button>
145|    {:else}
146|      <a class="blurb-login-hint" href="/login">Log In to Bookmark or Give Kudos</a>
147|    {/if}
148|  </div>
149|</article>
150|```
151|
152|Four sections stacked vertically:
153|
154|1. **Header** — A flex row showing the rating badge next to the title. The title link goes to `/fic/{url_id}`, which fetches the full export and eventually lands on the `ArchiveWork` detail page (covered in Chapter 13).
155|
156|2. **Byline** — "by [Author Name]" where the author name links to a filtered search for their works: `/search?q=author:{encodedName}`. This lets readers discover more stories by the same writer.
157|
158|3. **Tags** — A conditional render. If there are no tags (possible for poorly-tagged imports), nothing shows. Otherwise, `TagSoup` formats them into labeled rows (see Chapter 10).
159|
160|4. **Snippet** — Another conditional: only shown when the search API returned a highlight snippet or description. Truncated to 300 characters with an ellipsis.
161|
162|5. **StatsLine** — Passes through raw numbers. `StatsLine` handles the formatting (word count commas, relative dates, etc.). See Chapter 11.
163|
164|6. **Actions** — Four interactive elements: Download, Read, Bookmark, and Kudos. The first two are always `<a>` links (navigation). Bookmark and Kudos change based on auth state: logged-in users see `<button>` elements; anonymous visitors see a polite prompt linking to login.
165|
166|### Action Buttons and Auth-Aware Rendering
167|
168|The action button section demonstrates a common pattern across the entire Archive frontend: conditionally render different UI based on authentication status.
169|
170|When the user is logged in:
171|```svelte
172|<button class="blurb-action" type="button">Bookmark</button>
173|<button class="blurb-action" type="button">Kudos</button>
174|```
175|
176|These are plain buttons (not links) because bookmarking and giving kudos are *actions* that will trigger JavaScript handlers in the parent component. Right now they're placeholders — the actual API integration hooks up later. But structurally they follow AO3's layout: four equally-sized button slots, two navigation links, and one inline hint.
177|
178|When the user is *not* logged in, the buttons disappear and get replaced by a subtle italicized hint:
179|```svelte
180|<a class="blurb-login-hint" href="/login">Log In to Bookmark or Give Kudos</a>
181|```
182|
183|This is gentle UX — it doesn't block interaction with Download and Read, but it gently suggests what the user is missing.
184|
185|### CSS Styling — Flat, Boxed, AO3-Consistent
186|
187|The CSS enforces the familiar AO3 look: sharp corners, flat fills, thin borders, serif fonts. Here are the most important rules:
188|
189|```css
190|.work-blurb {
191|  border: 1px solid var(--archive-border, #dddddd);
192|  padding: 0.75em 1em;
193|  margin-bottom: 0.75em;
194|  background: var(--archive-bg, #ffffff);
195|}
196|
197|.blurb-header {
198|  display: flex;
199|  align-items: baseline;
200|  gap: 0.5em;
201|  flex-wrap: wrap;
202|}
203|
204|.blurb-title {
205|  font-size: 1.2em;
206|  font-weight: 700;
207|  color: var(--archive-link, #990000);
208|  text-decoration: none;
209|  line-height: 1.4;
210|}
211|
212|.blurb-title:hover {
213|  text-decoration: underline;
214|}
215|
216|.blurb-action--primary {
217|  color: var(--archive-bg, #ffffff);
218|  background: var(--archive-link, #990000);
219|  border-color: var(--archive-link, #990000);
220|}
221|
222|.blurb-login-hint {
223|  font-size: 0.85em;
224|  color: var(--archive-muted, #666666);
225|  font-style: italic;
226|}
227|```
228|
229|Key observations:
230|
231|- **The box model** — Every card is a simple rectangle: 1px border, 0.75em internal padding, 0.75em bottom margin. That margin stacks between cards. There's no `border-radius` — AO3 cards are perfectly square-cornered, even on mobile.
232|
233|- **The header layout** — `display: flex` with `align-items: baseline` keeps the rating badge and the title sitting nicely on the same visual line, even though they're different font sizes. The `gap: 0.5em` adds breathing room. `flex-wrap: wrap` prevents overflow on narrow screens.
234|
235|- **The primary button** — `blurb-action--primary` overrides the gray button style with maroon (`#990000`) background and white text. This is the "Read" button — the main call-to-action that stands out among the secondary actions. On hover, it shifts to the visited-purple `#660066`.
236|
237|- **Login hint styling** — Small, italic, muted gray. Deliberately unobtrusive. It's telling the user something, not trying to sell them on it.
238|
239|### How WorkBlurb Uses Other Components
240|
241|`WorkBlurb` is a *compositional* component — it doesn't implement tag formatting or stat display itself. Instead, it delegates:
242|
243|| Sub-component | Purpose | File |
244||---|---|---|
245|| `TagSoup` | Renders categorized tag rows | `TagSoup.svelte` (Ch. 10) |
246|| `StatsLine` | Formats words/chapters/kudos/updated inline | `StatsLine.svelte` (Ch. 11) |
247|| `mapRating()` | Converts rating codes → readable strings | `rating.ts` (Ch. 12) |
248|
249|This compositional approach means `TagSoup` can be used independently (on the detail page, for example) and `StatsLine` works the same way on both list views and detail views. `WorkBlurb` is the glue that holds them together.
250|
251|> **💡 Key Concept — Component Composition**
252|>
253|> `WorkBlurb` is the highest-level content component in the Archive: it doesn't do rendering-heavy work itself. Instead, it *composes* smaller specialized components (`TagSoup`, `StatsLine`) and pure functions (`mapRating`). This mirrors how AO3's own PHP templates organize markup into reusable partials. In Svelte terms, composition happens naturally: import the thing, use it in the template. No render props, no HOCs, no complicated wiring. Just declarative assembly of parts.
254|
255|### Summary Statistics Passed Down
256|
257|Notice how `WorkBlurb` receives raw, unformatted data from the API and immediately passes it down:
258|
259|```svelte
260|<StatsLine
261|  words={fic.words}        // raw number: 1204116
262|  chapters={fic.chapters}  // raw number: 109
263|  status={fic.status}      // string: "complete" or "In Progress"
264|  kudos={fic.kudos_count}  // raw number: 412
265|  updated={fic.updated}    // ISO date string: "2024-06-10T12:00:00Z"
266|/>
267|```
268|
269|The formatting happens deep inside `StatsLine`, which calls `formatWords()`, `chaptersDisplay()`, and `formatUpdated()` from `rating.ts`. This separation — raw data at the top, formatted display at the bottom — keeps each layer focused on its job.
270|
271|> **🧪 Try It Yourself**
272|>
273|> Open the search results page in Archive mode. Pick any fic card and inspect its DOM tree. You'll see `<article class="work-blurb">` wrapping a series of child `<div>`s and nested components. Use your browser's developer tools to toggle the `.blurb-action--primary` class on the "Read" button — watch the background shift from maroon to purple-gray as the hover styles take over. Now try removing the `.work-blurb` border and see what happens to the visual separation between cards.
274|
275|> **⚠️ Watch Out**
276|>
277|> The Bookmark and Kudos buttons in `WorkBlurb` currently don't have `onclick` handlers wired up — they're structural placeholders. The parent component (e.g. the search results page) is responsible for attaching those handlers and calling the appropriate API endpoints. Don't expect clicking them to do anything yet. That's fine for now — the goal of `WorkBlurb` is to *display* correctly, not to handle domain logic.
278|
279|---
280|
281|## Chapter 10: TagSoup — Tag Display
282|
283|If `WorkBlurb` is the card, `TagSoup.svelte` is the label maker. At 117 lines (`frontend/src/lib/ui/archive/TagSoup.svelte`), it takes a dictionary of tags — grouped by category — and renders them as clean, labeled rows. Fandoms, Characters, Relationships, Additional Tags, Warnings, Categories — each category becomes a bolded gray label followed by comma-separated maroon links.
284|
285|AO3 calls these "tags" a soup because fics often carry dozens of them, mixing structured metadata (fandoms, characters) with free-form creative labels (fanfic-specific tropes, shipping pairings). FicHub preserves this soup metaphor literally in the component name.
286|
287|### The Props Interface
288|
289|```svelte
290|<script lang="ts">
291|  import {
292|    categoryLabel,
293|    orderedCategories,
294|  } from './rating.js';
295|
296|  interface Tag {
297|    category?: number | string;
298|    name: string;
299|    [key: string]: unknown;
300|  }
301|
302|  let {
303|    tags = {},
304|    showAll = false,
305|  }: {
306|    tags: Record<string, Tag[]>;
307|    showAll?: boolean;
308|  } = $props();
309|
310|  const MAX_FREEFORM = 6;
311|  const ordered = $derived(orderedCategories(tags));
312|
313|  function tagUrl(type: string, name: string): string {
314|    return `/search?include_tags=${type}:${encodeURIComponent(name)}`;
315|  }
316|</script>
317|```
318|
319|Two props control behavior:
320|
321|- **`tags`** — A dictionary mapping category keys (strings like `"1"`, `"2"`, `"4"`) to arrays of tag objects. Each tag has a `name` and an optional `category`. Default is an empty object so the component renders silently when given no data.
322|- **`showAll`** — When `false`, additional tags (freeforms, category 4) get truncated to six items with a "+N more" expandable summary. When `true`, all freeforms show unconditionally.
323|
324|The `MAX_FREEFORM` constant is hardcoded at 6 — matching AO3's limit for collapsed freeform tags on list views. The `tagUrl()` helper builds search-links in the format `/search?include_tags={category}:{tagName}`, which the backend parses and filters against.
325|
326|### Ordered Categories
327|
328|The secret sauce is `orderedCategories()`, imported from `rating.ts`:
329|
330|```typescript
331|export function orderedCategories(grouped: Record<string, unknown[]>): string[] {
332|  return CATEGORY_ORDER.filter((c) => grouped[c]?.length);
333|}
334|```
335|
336|Recall that `CATEGORY_ORDER` is defined in `rating.ts` as:
337|
338|```typescript
339|const CATEGORY_ORDER = ['1', '2', '6', '3', '4', '5'];
340|```
341|
342|That order — Fandoms, Characters, Categories, Relationships, Additional Tags, Warnings — matches AO3's display convention. Some users expect fandoms first, others want characters first; AO3 settled on this order years ago and FicHub preserves it for familiarity. `orderedCategories` filters out any categories that are empty, so a fic with only fandoms and additional tags will render just two rows.
343|
344|The derived value updates reactively: whenever `tags` changes (which happens when you navigate between search results pages), `ordered` recalculates automatically via `$derived`.
345|
346|### Rendering Rows with `{#each}`
347|
348|The template uses Svelte's `{#each}` iterating over the ordered category keys:
349|
350|```svelte
351|<div class="tag-soup">
352|  {#each ordered as cat}
353|    {@const items = tags[cat] ?? []}
354|    {#if items.length > 0}
355|      <div class="tag-row">
356|        <span class="tag-label">{categoryLabel(cat)}</span>
357|        <span class="tag-list">
358|          {#if cat === '4' && !showAll && items.length > MAX_FREEFORM}
359|            {/* Truncated freeform tags with expandable details */}
360|            {#each items.slice(0, MAX_FREEFORM) as tag, i}
361|              {#if i > 0}<span class="tag-sep">,</span>{/if}
362|              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
363|            {/each}
364|            <details class="tag-more-details">
365|              <summary class="tag-more-summary">
366|                +{items.length - MAX_FREEFORM} more
367|              </summary>
368|              <span class="tag-list">
369|                {#each items.slice(MAX_FREEFORM) as tag, i}
370|                  {#if i > 0}<span class="tag-sep">,</span>{/if}
371|                  <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
372|                {/each}
373|              </span>
374|            </details>
375|          {:else}
376|            {#each items as tag, i}
377|              {#if i > 0}<span class="tag-sep">,</span>{/if}
378|              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
379|            {/each}
380|          {/if}
381|        </span>
382|      </div>
383|    {/if}
384|  {/each}
385|</div>
386|```
387|
388|This is the most complex template logic in the entire content component family. Three key patterns worth noting:
389|
390|#### The `{@const}` Shorthand
391|
392|```svelte
393|{@const items = tags[cat] ?? []}
394|```
395|
396|Instead of referencing `tags[cat]` repeatedly, we assign it once to a local constant `items`. The `{@const}` declaration is evaluated each iteration, so it picks up whatever tags belong to the current category. It's cleaner than repeating `tags[cat]` three times in the inner template.
397|
398|#### The Freeform Truncation Logic
399|
400|The truncation gate checks three conditions simultaneously:
401|```svelte
402|{#if cat === '4' && !showAll && items.length > MAX_FREEFORM}
403|```
404|
405|Only for Additional Tags (category 4), only when `showAll` is `false`, and only when there are more than six tags. All three must be true. If any one is false, the branch falls through to the un-truncated rendering below.
406|
407|The truncated portion renders exactly six tags via `items.slice(0, MAX_FREEFORM)`, then wraps the remainder in a native HTML `<details>` element. Clicking the summary reveals the hidden tags — no JavaScript required, fully keyboard-navigable, accessible by default.
408|
409|#### Comma-Separated Lists Without Trailing Commas
410|
411|```svelte
412|{#each items as tag, i}
413|  {#if i > 0}<span class="tag-sep">,</span>{/if}
414|  <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
415|{/each}
416|```
417|
418|The `{#if i > 0}` guard prevents a leading comma before the first item. Each tag link is styled in maroon and navigates to a filtered search. The separator is a plain `<span>` with class `tag-sep` — rendered as a visually lighter `, ` that sits between the tag links.
419|
420|### CSS Styling
421|
422|```css
423|.tag-soup {
424|  margin: 0.3em 0 0.5em;
425|  font-size: 0.9em;
426|  line-height: 1.7;
427|}
428|
429|.tag-row {
430|  display: flex;
431|  flex-wrap: wrap;
432|  gap: 0.25em;
433|}
434|
435|.tag-label {
436|  font-weight: 700;
437|  color: var(--archive-muted, #666666);
438|  margin-right: 0.35em;
439|  flex-shrink: 0;
440|}
441|
442|.tag-link {
443|  color: var(--archive-link, #990000);
444|  text-decoration: none;
445|}
446|
447|.tag-link:hover {
448|  text-decoration: underline;
449|}
450|
451|.tag-more-summary {
452|  color: var(--archive-muted, #666666);
453|  cursor: pointer;
454|  font-size: 0.92em;
455|}
456|
457|.tag-more-summary:hover {
458|  color: var(--archive-link, #990000);
459|}
460|```
461|
462|- **`.tag-label`** — Bold, gray (`#666666`), non-shrinking (`flex-shrink: 0`) so "Fandoms:" stays intact even on narrow screens where the tag links wrap to the next line.
463|- **`.tag-link`** — Maroon colored, no underline until hover. Same treatment as the title links in `WorkBlurb` — consistency across the archive.
464|- **`.tag-more-summary`** — The clickable "+N more" text. Hover turns it maroon, signaling interactivity. Because it lives inside `<details>`, the browser handles the open/close transition natively.
465|
466|### Why Native `<details>` for Expansion?
467|
468|Using `<details>/<summary>` instead of Svelte state (`let expanded = $state(false)`) has real advantages:
469|
470|1. **Accessibility** — Screen readers announce "expandable section" automatically. Keyboard users can Tab to it and press Enter to toggle.
471|2. **No JS overhead** — The browser manages the attribute. If JavaScript fails to load, the tags still expand.
472|3. **State persistence** — The `<details>` element remembers its open/closed state during navigation (within the same document lifetime). You could add `open` as a reactive hook if needed, but the default behavior works well.
473|4. **Less code** — No need for separate click handlers, CSS transitions for open/close animations, or aria-expanded attributes.
474|
475|> **💡 Key Concept — Progressive Enhancement**
476|>
477|> TagSoup's expandable section is built on a progressive enhancement model: the base experience (collapsed freeform tags) works without any JavaScript. The enhanced experience (click to expand) is provided by the browser's native `<details>` element. If you're building an Archive feature and wondering "should I use Svelte state or native HTML here?" — the rule of thumb is: if the browser already handles it (<details>, <form>, <select>), prefer the native element. Reserve Svelte state for things the browser *doesn't* handle.
478|
479|> **🧪 Try It Yourself**
480|>
481|> Go to a search results page and find a fic with many additional tags (more than six). Look for the "+N more" link after the sixth tag. Click it — the hidden tags slide open using the native `<details>` expand animation. Right-click the "+N more" text and choose "Inspect Element" in developer tools. Notice it's inside a `<details>` element with a `<summary>` child — pure HTML, no custom widget.
482|
483|> **⚠️ Watch Out**
484|>
485|> The `tagUrl()` function encodes the tag *name* but not the category in the URL-encoded segment — it interpolates the category directly: `/search?include_tags=${type}:${encodeURIComponent(name)}`. This means the `:` character acts as a delimiter understood by the backend parser. Don't try to URL-encode the colon; the backend expects the literal format `type:name`.
486|
487|---
488|
489|## Chapter 11: StatsLine — Single-Line Stats
490|
491|Meet the simplest component in the content family. At just 52 lines (`frontend/src/lib/ui/archive/StatsLine.svelte`), `StatsLine` squeezes five pieces of fic metadata into a single horizontal line: word count, chapter count, kudos, and update time, separated by middot characters.
492|
493|It's also the most *used* component — every `WorkBlurb` on a search results page includes one, and the detail page (ArchiveWork) embeds another. Two appearances per fic, one tiny component.
494|
495|### The Props Interface
496|
497|```svelte
498|<script lang="ts">
499|  import { formatWords, chaptersDisplay, formatUpdated } from './rating.js';
500|
501|  let {
502|    words = 0,
503|    chapters = 0,
504|    status = '',
505|    kudos = 0,
506|    updated = '',
507|  }: {
508|    words?: number | null;
509|    chapters?: number | null;
510|    status?: string | null;
511|    kudos?: number | null;
512|    updated?: string | null;
513|  } = $props();
514|
515|  const wordsFmt = $derived(formatWords(words));
516|  const chapFmt = $derived(chaptersDisplay(chapters, status));
517|  const kudosFmt = $derived(kudos?.toLocaleString('en-US') ?? '0');
518|  const updatedFmt = $derived(formatUpdated(updated));
519|</script>
520|```
521|
522|Five numeric/string props, all nullable. The component accepts raw values and formats them internally. Importantly, it depends on *three* pure functions from `rating.ts`:
523|
524|| Function | Input | Output |
525||---|---|---|
526|| `formatWords()` | `1204116` | `"1,204,116"` |
527|| `chaptersDisplay()` | `(109, "complete")` | `"109/109"` |
528|| `formatUpdated()` | `"2024-06-10T12:00:00Z"` | `"3d ago"` |
529|
530|The kudos formatting is handled inline using `toLocaleString('en-US')` because the same utility isn't exported from `rating.ts` (it's specific to StatsLine's needs — `formatWords` serves a similar purpose but for word counts).
531|
532|### The Template
533|
534|```svelte
535|<div class="stats-line">
536|  <span>Words: {wordsFmt}</span>
537|  <span class="stats-sep">·</span>
538|  <span>Chapters: {chapFmt}</span>
539|  {#if kudos}
540|    <span class="stats-sep">·</span>
541|    <span>Kudos: {kudosFmt}</span>
542|  {/if}
543|  {#if updatedFmt}
544|    <span class="stats-sep">·</span>
545|    <span>Updated: {updatedFmt}</span>
546|  {/if}
547|</div>
548|```
549|
550|Three fixed columns (Words, Chapters) always appear. Two optional columns (Kudos, Updated) render conditionally — only when the parent provides non-empty values. The middot separator `·` (Unicode U+00B7) gives visual rhythm: compact punctuation that separates values without the visual weight of a pipe `\|` or slash `/`.
551|
552|### CSS — Narrow Enough for Mobile
553|
554|```css
555|.stats-line {
556|  font-size: 0.85em;
557|  color: var(--archive-muted, #666666);
558|  line-height: 1.5;
559|  white-space: nowrap;
560|  overflow: hidden;
561|  text-overflow: ellipsis;
562|}
563|
564|.stats-sep {
565|  margin: 0 0.4em;
566|  opacity: 0.5;
567|}
568|```
569|
570|The critical properties:
571|
572|- **`white-space: nowrap`** — forces the entire line onto one visual line. On very narrow screens (mobile portrait), this means the line might get cut off.
573|- **`overflow: hidden` + `text-overflow: ellipsis`** — if the line is too wide for its container, the trailing characters are replaced with `…`. So if the stats line is too long, you lose "Updated: 3d ago" first, not "Words: 1,204,116".
574|- **`opacity: 0.5`** on the middots — makes separators visually lighter than the text, creating a subtle hierarchy: the data values dominate, the separators whisper.
575|- **`font-size: 0.85em`** — smaller than surrounding body text, making stats feel like metadata rather than content.
576|
577|> **💡 Key Concept — Metadata Design Patterns**
578|>
579|> StatsLine embodies several established metadata design patterns borrowed from AO3:
580|> 
581|> 1. **Middot separators** — Used extensively on AO3's review pages, profile headers, and submission confirmations. More compact than slashes, less formal than pipes.
582|> 2. **Relative timestamps** — "3d ago" instead of "June 10, 2024" tells users *how fresh* content is without forcing them to mentally compare dates. Implemented by `formatUpdated()` in `rating.ts`.
583|> 3. **Complete-chapter shorthand** — "109/109" signals completion instantly. Compare to "109/?" for in-progress works — the question mark communicates "we don't know how many total chapters yet." Implemented by `chaptersDisplay()` in `rating.ts`.
584|
585|> **⚠️ Watch Out**
586|>
587|> The `white-space: nowrap` + `text-overflow: ellipsis` combo means the stats line is not responsive — it won't wrap to multiple lines on narrow screens. If you're adding more stat fields in the future, consider whether the line might become unreadably long. One option is switching to a vertical stack on mobile using a media query:
588|>
589|> ```css
590|> @media (max-width: 480px) {
591|>   .stats-line {
592|>     white-space: normal;
593|>   }
594|>   .stats-sep {
595|>     display: none;
596|>   }
597|> }
598|> ```
599|>
600|> Currently no such breakpoint exists — the stats line may truncate on very small phones, but this is considered acceptable since the core stats (words and chapters) remain visible.
601|
602|---
603|
604|## Chapter 12: rating.ts — Pure Helpers
605|
606|Before we meet the largest content component (`ArchiveWork`), let's pause at the foundation beneath them all: `rating.ts` (`frontend/src/ui/archive/rating.ts`, 118 lines). Despite its name, this file contains no rating display logic. It's a library of **six pure TypeScript functions** that convert, format, and organize AO3-style data into presentation-ready strings.
607|
608|There is no Svelte in this file. No DOM. No reactive state. Just input → transformation → output, every single time.
609|
610|### What Does "Pure Function" Mean?
611|
612|A pure function has two guarantees:
613|
614|1. **Same input → same output**, always. Given `"mature"`, `mapRating()` always returns `"Mature"`. Given `"1"`, `categoryLabel()` always returns `"Fandoms"`. No randomness, no external state lookup, no mutations.
615|2. **No side effects**. It doesn't modify globals, log to console, make network requests, or touch the DOM. It takes data in and puts transformed data out.
616|
617|This matters because pure functions are trivially testable. You can write a unit test that calls `formatWords(1204116)` and asserts it equals `"1,204,116"` without needing a browser, a Svelte compiler, or a running server. They're the safest, most reliable code you can write.
618|
619|### The Six Functions
620|
621|#### 1. `mapRating(rating: string | null | undefined): string`
622|
623|Converts a short AO3-style code to its full display name:
624|
625|```typescript
626|const RATING_MAP: Record<string, string> = {
627|  general: 'General Audiences',
628|  teen: 'Teen And Up Audiences',
629|  mature: 'Mature',
630|  explicit: 'Explicit',
631|};
632|
633|const DEFAULT_RATING = 'Not Rated';
634|
635|export function mapRating(rating: string | null | undefined): string {
636|  if (!rating) return DEFAULT_RATING;
637|  return RATING_MAP[rating.toLowerCase()] ?? DEFAULT_RATING;
638|}
639|```
640|
641|The `RATING_MAP` dictionary is a module-level constant (defined outside any function), shared across all callers. The function lowercases the input to handle case variations (some backends return `"TEEN"` instead of `"teen"`), looks it up, and falls back to `"Not Rated"` for null/undefined/unknown codes.
642|
643|#### 2. `groupTags(tags: Tag[]): Record<string, Tag[]>`
644|
645|Flattens a tag array into a category-keyed dictionary:
646|
647|```typescript
648|export function groupTags(tags: Tag[]): Record<string, Tag[]> {
649|  const result: Record<string, Tag[]> = {};
650|
651|  for (const tag of tags) {
652|    const key = String(tag.category ?? '4');
653|    if (!result[key]) result[key] = [];
654|    result[key].push(tag);
655|  }
656|
657|  // Ensure all known categories exist in output
658|  for (const cat of CATEGORY_ORDER) {
659|    if (!result[cat]) result[cat] = [];
660|  }
661|
662|  return result;
663|}
664|```
665|
666|Notice how `groupTags` ensures all six category keys exist in its output, even if empty. This lets consumers iterate over all categories uniformly without checking for key existence. The blank-category default is `'4'` (Additional Tags) — stray tags without a category end up there.
667|
668|You won't call `groupTags()` directly from `WorkBlurb` anymore — it moved into `rating.ts` as a standalone function, and `WorkBlurb` does the grouping inline with `$derived.by()`. But `groupTags()` remains useful for other parts of the codebase that need the same fan-out operation.
669|
670|#### 3. `categoryLabel(category: string): string`
671|
672|Returns the human-readable label for a category number:
673|
674|```typescript
675|const CATEGORY_LABELS: Record<string, string> = {
676|  '1': 'Fandoms',
677|  '2': 'Characters',
678|  '3': 'Relationships',
679|  '4': 'Additional Tags',
680|  '5': 'Warnings',
681|  '6': 'Categories',
682|};
683|
684|export function categoryLabel(category: string): string {
685|  return CATEGORY_LABELS[category] ?? 'Tags';
686|}
687|```
688|
689|Used by `TagSoup` to render labels like **"Fandoms:"**, **"Characters:"**, etc. Falls back to generic `"Tags:"` for unknown category numbers. Simple lookup — zero branching logic.
690|
691|#### 4. `orderedCategories(grouped: Record<string, unknown[]>): string[]`
692|
693|Filters `CATEGORY_ORDER` to only include categories that actually contain tags:
694|
695|```typescript
696|const CATEGORY_ORDER = ['1', '2', '6', '3', '4', '5'];
697|
698|export function orderedCategories(grouped: Record<string, unknown[]>): string[] {
699|  return CATEGORY_ORDER.filter((c) => grouped[c]?.length);
700|}
701|```
702|
703|If a fic has only fandom tags and additional tags, this returns `['1', '4']`. The AO3-standard order is preserved, blanks pruned. Called by `TagSoup` to determine which rows to render and in what order.
704|
705|#### 5. `formatWords(words: number | null | undefined): string`
706|
707|Comma-formats word counts:
708|
709|```typescript
710|export function formatWords(words: number | null | undefined): string {
711|  if (words == null || isNaN(words)) return '0';
712|  return words.toLocaleString('en-US');
713|}
714|```
715|
716|`1204116` → `"1,204,116"`. Uses the built-in `Intl.NumberFormat` through `toLocaleString()`. Falls back to `"0"` for null, undefined, or NaN inputs. This is why you see nice readable numbers in StatsLine instead of raw integers.
717|
718|#### 6. `chaptersDisplay(chapters: number | null | undefined, status: string | null | undefined): string`
719|
720|Builds the chapter completion indicator:
721|
722|```typescript
723|export function chaptersDisplay(
724|  chapters: number | null | undefined,
725|  status: string | null | undefined
726|): string {
727|  const current = chapters ?? 0;
728|  const complete = status?.toLowerCase() === 'complete' || status?.toLowerCase() === 'completed';
729|  return complete ? `${current}/${current}` : `${current}/?`;
730|}
731|```
732|
733|Works for two statuses: `"complete"` and `"completed"` (some backends use one, some use the other). Returns `"109/109"` for completed fics, `"5/?"` for ongoing ones. The question mark clearly signals uncertainty — "five chapters so far, who knows how many more."
734|
735|#### 7. `formatUpdated(updated: string | null | undefined): string`
736|
737|Converts ISO timestamps into relative time strings:
738|
739|```typescript
740|export function formatUpdated(updated: string | null | undefined): string {
741|  if (!updated) return '';
742|  const date = new Date(updated);
743|  if (isNaN(date.getTime())) return '';
744|  const now = Date.now();
745|  const diff = now - date.getTime();
746|  const seconds = Math.floor(diff / 1000);
747|  if (seconds < 60) return 'just now';
748|  const minutes = Math.floor(seconds / 60);
749|  if (minutes < 60) return `${minutes}m ago`;
750|  const hours = Math.floor(minutes / 60);
751|  if (hours < 24) return `${hours}h ago`;
752|  const days = Math.floor(hours / 24);
753|  if (days < 30) return `${days}d ago`;
754|  const months = Math.floor(days / 30);
755|  if (months < 12) return `${months}mo ago`;
756|  const years = Math.floor(months / 12);
757|  return `${years}y ago`;
758|}
759|```
760|
761|This cascading time conversion follows a well-known pattern (used by GitHub, Medium, Twitter, and AO3's own timestamp utilities). The granularity narrows as time grows: seconds → minutes → hours → days → months → years. Each threshold is chosen pragmatically:
762|
763|- Under 60 seconds → `"just now"` (fresh enough to be notable)
764|- Under 60 minutes → `"Xm ago"` (useful for activity monitoring)
765|- Under 24 hours → `"Xh ago"` (daily cycles matter)
766|- Under 30 days → `"Xd ago"` (weekly reading habits)
767|- Under 12 months → `"Xmo ago"` (monthly overview)
768|- Beyond → `"Xy ago"` (yearly perspective)
769|
770|The function uses `Date.now()` to get the current time, so the displayed relative time changes dynamically as the user browses — no timestamp stored anywhere. This means "Updated: 3d ago" will always be correct regardless of when the page was originally loaded.
771|
772|> **💡 Key Concept — Why Pure Functions?**
773|>
774|> `rating.ts` is pure. That means:
775|> - **Testability**: Call `formatWords(999)` and assert `"999"`. Done. No mocking, no fixtures, no Svelte testing harness.
776|> - **Reusability**: Any component can import `mapRating` or `formatUpdated` without importing Svelte, without setting up stores, without worrying about timing.
777|> - **Predictability**: Change the code and every caller sees the change immediately. No hidden coupling.
778|> - **Server compatibility**: These functions run identically on the server (SSR) and in the browser (CSR). No `window` access, no `document` calls.
779|>
780|> In Svelte projects, the temptation is to put formatting logic inside template expressions or `$derived` blocks. Resist that urge! Extract formatting into a `.ts` file of pure functions. Your template stays clean, your logic stays testable, and your components stay thin.
781|
782|> **⚠️ Watch Out**
783|>
784|> `formatUpdated()` creates a `new Date()` and calls `Date.now()` on every invocation. For most components (a handful of StatsLines per page), this cost is negligible. But if you ever pass `formatUpdated()` into a `{#each}` loop with thousands of iterations, it will create thousands of Date objects. In that case, compute the "now" timestamp once outside the loop and pass it in as a parameter. Not a concern for current usage, but a gotcha to remember.
785|
786|---
787|
788|## Chapter 13: ArchiveWork — Fic Detail Page
789|
790|Finally, the big one. `ArchiveWork.svelte` (`frontend/src/lib/ui/archive/ArchiveWork.svelte`, 407 lines) is the **largest content component** and the destination every fic card leads to. When a user clicks "Read" or the title on a `WorkBlurb`, they land here — a dedicated page displaying the full fic metadata, summary, download options, chapter list, and author notes.
791|
792|Think of `WorkBlurb` as the business card and `ArchiveWork` as the whole biography. Compact at a glance, detailed on demand.
793|
794|### The Props Interface
795|
796|```svelte
797|<script lang="ts">
798|  import { auth } from '$lib/stores/auth.svelte';
799|  import { mapRating, groupTags, categoryLabel, orderedCategories, formatWords, chaptersDisplay, formatUpdated } from './rating.js';
800|  import StatsLine from './StatsLine.svelte';
801|  import ArchiveButton from './ArchiveButton.svelte';
802|  import type { ExportResponse, FicMeta } from '$lib/api/types';
803|
804|  let {
805|    fic,
806|    isBookmarked = false,
807|    onToggleBookmark,
808|    savingBookmark = false,
809|  }: {
810|    fic: ExportResponse;
811|    isBookmarked?: boolean;
812|    onToggleBookmark?: () => void;
813|    savingBookmark?: boolean;
814|  } = $props();
815|```
816|
817|Unlike `WorkBlurb` which receives a lightweight `FicSearchResult`, `ArchiveWork` receives an `ExportResponse` — the full object from the export API (`GET /api/epub` or `GET /api/meta`). It has more fields, more depth, and crucially, download URLs.
818|
819|Three extra props handle bookmarking state:
820|
821|- **`isBookmarked`** — Whether this fic is already in the user's bookmarks. Defaults to `false` so the component can render independently of its parent's state.
822|- **`onToggleBookmark`** — A callback the parent provides to toggle the bookmark. Called when the user clicks the Bookmark button.
823|- **`savingBookmark`** — Loading indicator. Set to `true` during the network request so the button shows "Saving…" and disables itself.
824|
825|### Accessing the Nested Meta Object
826|
827|The `fic` object contains a nested `meta` property typed as `FicMeta | undefined`:
828|
829|```typescript
830|const meta: FicMeta | undefined = $derived(fic.meta);
831|const m = $derived(meta!);
832|```
833|
834|The non-null assertion (`!`) tells TypeScript "I know this is defined at runtime." In practice, `meta` should always be present for valid responses, but the type system can't prove that. The `m` alias gives us shorter references throughout the template: `m.title` instead of `fic.meta!.title`.
835|
836|### Derived Metadata
837|
838|The component derives several values from the raw metadata:
839|
840|```typescript
841|const title = $derived(m.title ?? 'Untitled');
842|const author = $derived(m.author ?? 'Anonymous');
843|const ratingDisplay = $derived(mapRating(null));
844|const createdDate = $derived(m.created ? new Date(m.created).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }) : '');
845|const updatedDate = $derived(m.updated ? new Date(m.updated).toLocaleDateString('en-US', { year: 'numeric', month: 'long', 'day': 'numeric' }) : '');
846|const isLoggedIn = $derived(auth.isLoggedIn);
847|```
848|
849|Note that `ratingDisplay` is called with `null` — the detail page doesn't use a rating badge like `WorkBlurb` does. The variable exists for potential future use but is currently unused.
850|
851|The date formatting uses the native `Intl.DateTimeFormat` through `toLocaleDateString()` with options for a verbose American-style date: "January 15, 2024". This produces friendlier output than ISO strings for display purposes.
852|
853|### The Download Dropdown
854|
855|One of the most distinctive features of the detail page is the multi-format download dropdown:
856|
857|```svelte
858|let downloads = $derived.by(() => {
859|  const out: { label: string; type: string; href?: string | null; lazy?: boolean }[] = [];
860|  const add = (label: string, type: string, href?: string | null, lazy = false) => {
861|    if (href) out.push({ label, type, href });
862|    else if (lazy) out.push({ label, type, href: null, lazy: true });
863|  };
864|  add('EPUB', 'epub', fic.epub_url);
865|  add('HTML', 'html', fic.html_url);
866|  add('TXT', 'txt', fic.txt_url);
867|  add('MD', 'md', fic.md_url);
868|  add('MOBI', 'mobi', fic.mobi_url, true);
869|  add('PDF', 'pdf', fic.pdf_url, true);
870|  add('AZW3', 'azw3', fic.azw3_url, true);
871|  add('DOCX', 'docx', fic.docx_url);
872|  add('FB2', 'fb2', fic.fb2_url);
873|  add('KEPUB', 'kepub', fic.kepub_url);
874|  return out;
875|});
876|```
877|
878|This `$derived.by()` block builds the download menu on-the-fly from the URL fields on the `ExportResponse`. Each format is added via the `add()` helper:
879|
880|- If `href` is truthy → the download is available, push with the URL.
881|- If `href` is falsy but `lazy` is `true` → the format exists but conversion hasn't started yet, push a disabled placeholder.
882|- If `href` is falsy and `lazy` is `false` → skip entirely (format not supported for this fic).
883|
884|Some formats (MOBI, PDF, AZW3) are marked `lazy` because they require server-side conversion that runs asynchronously. They appear as "(unavailable)" in the dropdown until the conversion completes and the URL populates.
885|
886|The toggle state is managed locally:
887|
888|```typescript
889|let showDownloadMenu = $state(false);
890|let selectedFormat: string = $state('epub');
891|```
892|
893|Clicking the Download button toggles `showDownloadMenu`:
894|
895|```svelte
896|<button
897|  class="action-btn action-download"
898|  onclick={() => (showDownloadMenu = !showDownloadMenu)}
899|>
900|  Download ▾
901|</button>
902|```
903|
904|And the menu renders conditionally:
905|
906|```svelte
907|{#if showDownloadMenu}
908|  <div class="download-menu">
909|    {#each downloads as dl}
910|      {#if dl.href}
911|        <a class="download-item" href={dl.href} download onclick={() => { showDownloadMenu = false; }}>
912|          {dl.label}
913|        </a>
914|      {:else}
915|        <span class="download-item download-unavailable" title="Conversion not yet available">
916|          {dl.label} (unavailable)
917|        </span>
918|      {/if}
919|    {/each}
920|  </div>
921|{/if}
922|```
923|
924|Each available format becomes a direct download link (`<a download>` triggers the browser's download dialog). Clicking one closes the menu (`onclick={() => { showDownloadMenu = false; }}`). Unavailable formats render as inert spans with a tooltip explaining the situation.
925|
926|> **💡 Key Concept — Lazy Format Availability**
927|>
928|> Not all download formats are immediately available. EPUB, HTML, TXT, MD, DOCX, FB2, and KEPUB are generated synchronously during the initial export process. MOBI, PDF, and AZW3 require heavier server-side conversion that may complete seconds or minutes after the export starts. FicHub marks these as "lazy" — they appear in the dropdown as unavailable until the background conversion finishes. This is a classic async processing pattern: respond immediately with what's ready, surface delayed results when they arrive.
929|
930|### Chapter Navigation
931|
932|The detail page lists every chapter as a numbered link:
933|
934|```svelte
935|<div class="work-chapters">
936|  <h3 class="section-heading">
937|    Chapters ({chaptersCount}{isComplete ? `/${chaptersCount}` : '/?'})
938|  </h3>
939|  <ol class="chapter-list">
940|    {#each Array.from({ length: chaptersCount }, (_, i) => i + 1) as chNum}
941|      <li class="chapter-item">
942|        <a class="chapter-link" href={`/read/${encodeURIComponent(m.id)}?chapter=${chNum}`}>
943|          Chapter {chNum}
944|        </a>
945|      </li>
946|    {/each}
947|  </ol>
948|</div>
949|```
950|
951|The chapter list is generated dynamically using `Array.from()` — a common JavaScript idiom for creating N sequential numbers. `{ length: chaptersCount }` allocates an array of the right size, and `(_, i) => i + 1` generates `[1, 2, 3, ...]`. Each entry becomes a link to the corresponding chapter anchor.
952|
953|The header shows `"Chapters (109/109)"` for completed works and `"Chapters (5/?)` for ongoing ones — the same convention used in `StatsLine`'s chapters display.
954|
955|### Bookmark Button States
956|
957|The bookmark button has three distinct states driven by the props:
958|
959|```svelte
960|{#if isLoggedIn && onToggleBookmark}
961|  <button
962|    class="action-btn action-bookmark"
963|    class:bookmarked={isBookmarked}
964|    onclick={onToggleBookmark}
965|    disabled={savingBookmark}
966|  >
967|    {#if savingBookmark}
968|      Saving…
969|    {:else if isBookmarked}
970|      Bookmarked
971|    {:else}
972|      Bookmark
973|    {/if}
974|  </button>
975|{:else if isLoggedIn}
976|  <button class="action-btn action-bookmark" disabled>Bookmark</button>
977|{:else}
978|  <a class="action-btn action-bookmark" href="/">Log In to Bookmark</a>
979|{/if}
980|```
981|
982|Three branches:
983|
984|1. **Logged in + handler provided** — The full interactive button. Shows "Bookmark", "Bookmarked", or "Saving…" depending on the sub-state. The `class:bookmarked={isBookmarked}` applies a CSS modifier class when true, turning the button maroon.
985|2. **Logged in + no handler** — A disabled static button. The component received `isLoggedIn` but no callback, meaning the parent hasn't implemented bookmarking yet.
986|3. **Not logged in** — A link to the login page, styled consistently with the action buttons.
987|
988|The `class:bookmarked` modifier maps to this CSS:
989|
990|```css
991|.action-bookmark.bookmarked {
992|  color: var(--archive-bg, #ffffff);
993|  background: var(--archive-link, #990000);
994|  border-color: var(--archive-link, #990000);
995|}
996|```
997|
998|A maroon button with white text — the same "primary" style as the Read Online button. Visual parity reinforces that bookmarking is a significant action, not a minor toggle.
999|
1000|### The Full Template Structure
1001|
1002|```svelte
1003|<article class="archive-work">
1004|  <!-- Title & Byline -->
1005|  <h2 class="work-title">{title}</h2>
1006|  <p class="byline">by <a class="author-link">...</a></p>
1007|  <p class="work-date">Added: ... · Updated: ...</p>
1008|
1009|  <!-- Stats -->
1010|  <div class="work-stats">
1011|    <StatsLine words={m.words} chapters={m.chapters} status={m.status} updated={m.updated} />
1012|  </div>
1013|
1014|  <!-- Summary -->
1015|  <div class="work-summary">
1016|    <h3 class="section-heading">Summary</h3>
1017|    <div class="summary-text">{@html m.description}</div>
1018|  </div>
1019|
1020|  <!-- Action Buttons -->
1021|  <div class="work-actions">
1022|    <a class="action-btn action-read">Read Online</a>
1023|    <div class="download-wrapper">
1024|      <button class="action-btn action-download">Download ▾</button>
1025|      {#if showDownloadMenu}<div class="download-menu">...</div>{/if}
1026|    </div>
1027|    <!-- Bookmark button (3-state) -->
1028|  </div>
1029|
1030|  <!-- Chapters List -->
1031|  <div class="work-chapters">...</div>
1032|
1033|  <!-- Author Notes -->
1034|  {#if fic.notes && fic.notes.length > 0}
1035|    <div class="work-notes">
1036|      {#each fic.notes as note}
1037|        <p class="note-text">ℹ️ {note}</p>
1038|      {/each}
1039|    </div>
1040|  {/if}
1041|</article>
1042|```
1043|
1044|Six semantic regions flowing top-to-bottom:
1045|
1046|1. **Identity** — Title, author, dates
1047|2. **Statistics** — Single-line stats (reuse of `StatsLine`)
1048|3. **Summary** — Author-provided description, rendered as raw HTML (`{@html}`) because AO3 summaries support paragraph breaks, links, and italics
1049|4. **Actions** — Read, Download, Bookmark/Kudos
1050|5. **Navigation** — Chapter list
1051|6. **Notes** — Optional author-attribution or disclaimer text
1052|
1053|### Using Pure Functions Inline
1054|
1055|`ArchiveWork` imports seven functions from `rating.ts` but actively uses most of them:
1056|
1057|```typescript
1058|import { mapRating, groupTags, categoryLabel, orderedCategories, formatWords, chaptersDisplay, formatUpdated } from './rating.js';
1059|```
1060|
1061|Of these, `groupTags` and `categoryLabel` are imported but not actively called in the current version (they were part of the original component outline and remain available for future use). The actively used ones include `formatWords`, `chaptersDisplay`, and `formatUpdated` — though notably, `ArchiveWork` renders the stats section via the `StatsLine` component (Chapter 11), which internally calls those same functions. The duplicate import is a remnant of development: the detail page could call them directly, but chose the compositional path instead by embedding `StatsLine`.
1062|
1063|### CSS Styling — Reading-Focused Layout
1064|
1065|The CSS reflects the component's role as a *reading surface*, not a navigation surface. It uses Georgia serif throughout, generous spacing, and a centered max-width:
1066|
1067|```css
1068|.archive-work {
1069|  max-width: var(--archive-max-width, 800px);
1070|  margin: 0 auto;
1071|  font-family: Georgia, 'Times New Roman', serif;
1072|  color: var(--archive-text, #2a2a2a);
1073|  line-height: 1.6;
1074|}
1075|
1076|.work-title {
1077|  font-family: Georgia, 'Times New Roman', serif;
1078|  font-size: 1.7em;
1079|  font-weight: normal;
1080|  color: var(--archive-heading, #990000);
1081|  margin: 0 0 0.15em;
1082|  border-bottom: 1px solid var(--archive-border, #dddddd);
1083|  padding-bottom: 0.3em;
1084|}
1085|
1086|.section-heading {
1087|  font-family: Georgia, 'Times New Roman', serif;
1088|  font-size: 1.05em;
1089|  font-weight: normal;
1090|  color: var(--archive-text, #2a2a2a);
1091|  border-bottom: 1px solid var(--archive-border, #dddddd);
1092|  margin: 1.2em 0 0.4em;
1093|  padding-bottom: 0.2em;
1094|}
1095|```
1096|
1097|Key design decisions:
1098|
1099|- **`max-width: 800px`** — Centers the content and limits line length for readability. Long lines (120+ characters) cause eye fatigue; 800px with 16px base font yields roughly 65 characters per line, the gold standard for text legibility.
1100|- **Georgia serif** — Explicitly set on both the article and title. Georgia is a screen-optimized serif designed specifically for readability at small sizes. AO3 uses it for a reason.
1101|- **Subtle dividers** — Section headings get a thin 1px bottom border, matching AO3's section delineators. No shadows, no backgrounds — just a quiet visual break.
1102|- **No rounded corners anywhere** — The reading page continues the strict boxy aesthetic of the entire Archive mode.
1103|
1104|> **💡 Key Concept — Separation of Concerns Across Components**
1105|>
1106|> `ArchiveWork` doesn't reimplement `StatsLine`'s formatting logic or build its own download dropdown from scratch. It *composes* existing components and functions:
1107|> - Imports `StatsLine` and embeds it directly (reusing all formatting).
1108|> - Imports the pure formatting functions from `rating.ts` (even if partially unused — they're available when needed).
1109|> - Reuses `ArchiveButton`'s styling foundation (though the detail page leans toward raw `<button>` and `<a>` with custom classes for more control).
1110|> 
1111|> This is the compositional philosophy we saw in `WorkBlurb`, extended. Each layer adds just enough specificity — `WorkBlurb` is a compact card; `ArchiveWork` is a full reading page. Neither duplicates the other's knowledge.
1112|
1113|> **⚠️ Watch Out**
1114|>
1115|> The summary renders with `{@html m.description}`, which injects raw HTML into the DOM. This is necessary because authors write summaries with paragraphs (`<p>`), italics (`<i>`), and links. However, always be cautious with `{@html}` — if the content came from an untrusted source, it could contain malicious scripts. In FicHub's case, descriptions come from scraped AO3/FanFiction.net content, which is generally safe. But if you ever accept user-submitted HTML (comments, reviews, custom fields), sanitize it first with a library like DOMPurify before using `{@html}`.
1116|
1117|> **🧪 Try It Yourself**
1118|>
1119|> Navigate to a fic's detail page. Inspect the chapter list in developer tools — each entry is a plain `<li><a>` generated by `Array.from({ length: chaptersCount }, ...)`. Click on "Chapter 1" and watch the URL gain a query parameter: `?chapter=1`. That parameter drives the chapter anchor scroll. Now try clicking "Download ▾" — notice how the dropdown appears *below* the button, positioned with `position: absolute` and `z-index: 10`. Click an unavailable format (like MOBI) to see the italicized "(unavailable)" text with its tooltip.
1120|
1121|---
1122|
1123|*Part 3 covered the five core content components: `WorkBlurb` (the fic card), `TagSoup` (tag rows), `StatsLine` (inline statistics), `rating.ts` (pure formatting functions), and `ArchiveWork` (the full detail page). Together they form the content rendering pipeline — from search result list to individual fic page. In Part 4, we'll explore the search interface, where these components connect to the live API.*
1124|
1|# Part 4 — The Search System
2|
3|Welcome to the search system! 🕵️‍♀️ You're going to learn how FicHub lets readers discover fanworks through powerful filtering, smart suggestions, and a beautifully organized search page. This is one of the most-used parts of the entire archive — people use it every single day to find their next favorite story. If you've ever spent hours scrolling through recommendations hoping something clicks, you know how frustrating it is when the *exact* search you want doesn't exist. That's why the search system matters so much at FicHub.
4|
5|Think about what goes into a search. A reader might want stories tagged with "Marvel" and "Tony Stark," with at least 1,000 words and no more than 50,000, published in General Audiences rating, sorted by kudos. That's a lot of constraints — but behind the scenes, our system needs to translate all those choices into a precise database query. That translation chain starts here.
6|
7|The search system has five main pieces:
8|
9|1. **searchForm.ts** — a pure module that turns user input into backend queries
10|2. **searchForm.test.ts** — 47 tests that make sure nothing breaks
11|3. **ArchiveWorkSearchForm.svelte** — the giant two-mode component (853 lines!)
12|4. **Search Page (+page.svelte)** — combines results and filters in a two-column layout
13|5. **Search API** — the connection between frontend and backend
14|
15|Each piece builds on the last, like layers in an onion. By the end of this part, you'll understand the complete pipeline from "user clicks a checkbox" to "results appear on screen."
16|
17|Let's go!
18|
19|---
20|
21|## Chapter 14: searchForm.ts — The Pure Search Module
22|
23|In this chapter, you'll meet the heart of the search system. Tucked away inside `frontend/src/lib/ui/archive/searchForm.ts` is a small but mighty module. Despite being just one file, it handles all the logic for turning what a user clicks, types, and checks into an actual search query that the backend understands. And here's what makes it special: it's *pure*. Everything in this module follows a rule called purity. A pure function is one where given the exact same inputs, you always get the exact same output, and nothing sneaky happens behind the scenes — no network calls, no database reads, no global variables getting changed. Just clean, predictable logic.
24|
25|Think of it like cooking a measured recipe. You put in two cups of flour and one egg, and you get the exact same batter every time. Not a wobbly cake factory, a reliable kitchen. The search module works the same way — it's the recipe book that translates "I want Marvel stories over 5,000 words" into `{ fandoms: ["Marvel"], wordCountMin: 5000 }`.
26|
27|### The Building Blocks: parseRange and buildSearchQuery
28|
29|Let's start with the smallest piece first: `parseRange`. This is a utility function that takes a raw string from the user (for example, `"1000-5000"`) and converts it into a structured range object with `min` and `max` properties:
30|
31|```typescript
32|// User typed these values into word-count input fields
33|const rawString = "5000-50000";
34|const range = parseRange(rawString);
35|// result: { min: 5000, max: 50000 }
36|```
37|
38|If the user only fills in one side — say, just a minimum with `"5000-"` — `parseRange` gracefully handles it by setting the missing side to `null`. Why does this matter? Because behind the scenes, the backend expects structured numeric objects, not free-form text strings. Users don't think in terms of JSON shapes; they think in terms of "at least 5,000 words." `parseRange` bridges that gap beautifully.
39|
40|Then comes the heavyweight champion: `buildSearchQuery`. This is the main function that takes the entire form state — every single field the user interacted with — and assembles it into a query object ready for the API:
41|
42|```typescript
43|import { buildSearchQuery } from "$lib/ui/archive/searchForm";
44|
45|const formState = {
46|  title: "Dragon",
47|  author: "FanficCreator",
48|  fandoms: ["Marvel Cinematic Universe", "Star Wars"],
49|  characters: ["Tony Stark", "Steve Rogers"],
50|  relationships: ["Peter Parker/MJ"],
51|  additionalTags: ["Time Travel", "Hurt/Comfort"],
52|  warnings: ["Graphic Depictions Of Violence"],
53|  categories: ["F/M"],
54|  ratings: ["General Audiences"],
55|  wordCount: { min: 1000, max: 50000 },
56|  kudosMin: 10,
57|  kudosMax: 500,
58|  commentsMin: 5,
59|  bookmarksMin: 3,
60|};
61|
62|const query = buildSearchQuery(formState);
63|// query: { title: "Dragon", ...all non-empty fields... }
64|```
65|
66|Notice something clever? Every field that the user *didn't* change simply doesn't appear in the output. The function filters out empty or unset values automatically. This keeps each request lean — fewer bytes traveling across the wire, faster queries running on the backend.
67|
68|### What Fields Are Actually Supported?
69|
70|Now here's where things get interesting — and honestly, a little surprising. Not everything you see on the search page actually filters results! The search system has a table of "supported" versus "inert" fields. Let me walk you through the complete roster:
71|
72|| Field | Supported? | How It Works |
73||-------|-----------|--------------|
74|| kudos | ✅ Partial | Only maps to `max_kudos`; no minimum support |
75|| comments | ✅ Supported | Both min and max controls fully functional |
76|| bookmarks\_min / bookmarks\_max | ✅ Supported | Independent minimum and maximum filters |
77|| word_count | ✅ Supported | Parsed through `parseRange` for flexible ranges |
78|| rating | ⚠️ Inert | Visible in UI but currently does nothing |
79|| crossovers | ⚠️ Inert | Renders with a no-op footnote explaining it's planned |
80|| hits | ⚠️ Inert | Another placeholder with a no-op footnote |
81|| language | ⚠️ Inert | Shown as a "coming soon" placeholder |
82|
83|⚠️ **Watch Out for Inert Fields:** When a field is inert, it looks clickable on the screen, but nothing happens when you interact with it. This can be confusing for new users who expect every input to work. The best practice documented in the codebase is to mark these clearly with footnotes so visitors know they're waiting for future development. And hey — that could be you implementing them someday!
84|
85|💡 **Key Concept: The Kudos Edge Case**
86|There's a subtle gotcha with the kudos field. The backend only provides a `max_kudos` parameter; there is no `min_kudos`. So even though the form shows both a "minimum kudos" and "maximum kudos" input, they actually both write to the same `max_kudos` parameter internally. This is recorded in the source code as a partial feature implementation. If you need stories with *at least* 100 kudos today, there's no direct way to express that constraint — you'd have to set the minimum to 100, which effectively becomes a maximum-of-100 instead. A real limitation worth noting!
87|
88|### Converting Between Form State and URLs
89|
90|FicHub has one of those features you take for granted until someone asks you to share a search. "How did you find that story?" "Oh, I just used this search." *Sends link.* "Wait, you mean I can copy a whole search into a URL and send it to people?" Exactly. Here's how:
91|
92|The `formStateToUrl` function serializes a complete form state object into a compact URL-encoded query string. Then `urlToFormState` reverses the process, reading a URL and reconstructing the original form state:
93|
94|```typescript
95|import { formStateToUrl, urlToFormState } from "$lib/ui/archive/searchForm";
96|
97|// Serialize: form state → URL
98|const initialState = {
99|  title: "Dragon",
100|  wordCount: { min: 1000, max: 50000 },
101|  commentsMin: 10,
102|  sortBy: "kudos",
103|};
104|
105|const url = formStateToUrl(initialState);
106|// Produces: "/search?q=title%3ADragon&min_words%3A1000&..."
107|
108|// Deserialize: URL → form state
109|const recovered = urlToFormState(url);
110|// recovered.title === "Dragon" ✓
111|// recovered.wordCount === { min: 1000, max: 50000 } ✓
112|// recovered.commentsMin === 10 ✓
113|```
114|
115|This serialization/deserialization round-trip is what makes deep-linking possible. Bookmark a search? Check. Email a friend your exact filtering setup? Done. Refresh your browser and lose nothing? Naturally. Without this capability, search functionality would feel fragile and frustrating.
116|
117|### Adding Chips: mergeChipIntoForm
118|
119|Now let's talk about chips. Remember those friendly suggestion tags that float above the search form? When logged-in users click one, it snaps into the active filter. Behind the scenes, a function called `mergeChipIntoForm` handles this:
120|
121|```typescript
122|import { mergeChipIntoForm } from "$lib/ui/archive/searchForm";
123|
124|let currentForm = { fandoms: ["Avengers"] };
125|
126|currentForm = mergeChipIntoForm(currentForm, {
127|  field: "fandoms",
128|  label: "Spider-Man",
129|  reason: "chip-click",
130|});
131|// currentForm.fandoms === ["Avengers", "Spider-Man"]
132|
133|// Clicking "Spider-Man" again does NOT create a duplicate:
134|currentForm = mergeChipIntoForm(currentForm, {
135|  field: "fandoms",
136|  label: "Spider-Man",
137|  reason: "chip-click",
138|});
139|// currentForm.fandoms === ["Avengers", "Spider-Man"]  ← still just 2!
140|```
141|
142|The deduplication behavior is baked directly into `mergeChipIntoForm`. Before appending a new label, it checks if the value already exists in the target array. This prevents the common bug where repeated clicking creates `[ "Tag", "Tag", "Tag" ]` — a frustration that plagues poorly designed filter UIs everywhere.
143|
144|💡 **Key Concept: Why Pure Functions Matter So Much**
145|You might reasonably ask: why go through all this trouble to make everything a pure function? The answer boils down to three powerful benefits:
146|
147|1. **Testing is trivial.** Because pure functions have zero side effects, every test is a simple assertion: `assertEquals(myFunc(input), expectedOutput)`. No mocking frameworks, no spies, no fake timers. This is why the test suite can cover 47 cases in a single focused file.
148|
149|2. **Behavior never surprises anyone.** Call `buildSearchQuery` with the same form state ten times, ten thousand times — identical output every single time. There's no hidden state bleeding between calls, no race conditions, no flakiness.
150|
151|3. **Functions compose freely.** Any part of the application — the search page, the sidebar, a settings dialog, a URL parser — can independently call any of these functions. They don't care about each other's existence. That decoupling is architectural gold.
152|
153|That wraps up searchForm.ts! You now understand how this slim but potent module transforms human-friendly form input into machine-readable search queries. In the next chapter, we'll watch 47 automated tests verify every line of this logic. Get ready for testing glory. 🧪
154|
155|---
156|
157|## Chapter 15: searchForm.test.ts — Testing the Module
158|
159|Now that you've met searchForm.ts, let's look at its test suite: `searchForm.test.ts`. This file has **47 passing tests**, covering every single function in the pure module. You might wonder: why so many tests for a small file? Because even simple-seeming logic can hide surprising corner cases, and in search systems specifically, *incorrect filtering silently returns wrong results* — the worst kind of bug. A login button either works or it doesn't; a search filter returning 200 results when there are actually 20,000 is much harder to notice.
160|
161|### The Test Philosophy: Given → When → Then
162|
163|Because searchForm.ts is built entirely from pure functions, testing is remarkably straightforward. Each test follows a simple three-step rhythm: provide some input, call a function, check the output. No mocking networks, no spinning up databases, no fake timers or complicated setup. It's clean and direct:
164|
165|```
166|Given: these inputs
167|When: I call this function with them
168|Then: I expect this exact output
169|```
170|
171|This pattern makes the tests lightning-fast to run — we're talking milliseconds, not seconds. Fast tests encourage developers to run them frequently, which means bugs get caught early rather than festering until release day. If you've ever tried adding a feature to a codebase where running the test suite takes five minutes, you know exactly why speed matters. Here, the feedback loop is nearly instant.
172|
173|### Range Parsing Tests: Where Numbers Begin
174|
175|These tests exercise `parseRange`, checking it handles every flavor of user input gracefully:
176|
177|```typescript
178|// Valid two-sided ranges
179|expect(parseRange("100-1000")).toEqual({ min: 100, max: 1000 });
180|expect(parseRange("1000-1000")).toEqual({ min: 1000, max: 1000 });
181|
182|// One-sided ranges — user omits one bound
183|expect(parseRange("100-")).toEqual({ min: 100, max: null });
184|expect(parseRange("-5000")).toEqual({ min: null, max: 5000 });
185|
186|// Empty string → nothing useful
187|expect(parseRange("")).toEqual({ min: null, max: null });
188|```
189|
190|Tests like these catch the edge cases that cause real bugs. What happens if someone accidentally submits an empty string? What if min and max are the same number (a common scenario when a user wants *exactly* N words)? The parseRange function must handle all of these without throwing exceptions. Every case listed above has its own dedicated test asserting the expected behavior.
191|
192|### Query Building Tests: Ensuring Clean Output
193|
194|These tests verify that `buildSearchQuery` assembles query objects correctly:
195|
196|```typescript
197|// Single field present → only that field appears in output
198|expect(buildSearchQuery({ title: "test" })).toHaveProperty("title", "test");
199|
200|// Completely empty input → completely empty output
201|expect(buildSearchQuery({})).toEqual({});
202|
203|// Multiple fields → all appear, none missing
204|const result = buildSearchQuery({
205|  title: "Dragon",
206|  kudosMin: 50,
207|  commentsMax: 100,
208|});
209|expect(result.title).toBe("Dragon");
210|expect(result.kudosMin).toBe(50);
211|expect(result.commentsMax).toBe(100);
212|```
213|
214|An empty state returning an empty query object is an intentionally designed property — it means unused fields never pollute the request payload. This keeps network traffic lean and ensures the backend receives *only* the filters the user explicitly set. Without this clean-slate behavior, you'd end up sending useless defaults down the wire.
215|
216|### URL Round-Trip Tests: Sharing Searches Works
217|
218|These tests verify the full serialization round-trip — form state into a URL and back again:
219|
220|```typescript
221|const original = {
222|  title: "Dragon",
223|  ratings: ["Explicit"],
224|  bookmarks_min: 10,
225|  sortBy: "date",
226|  sortOrder: "desc",
227|};
228|const url = formStateToUrl(original);
229|const restored = urlToFormState(url);
230|expect(restored).toEqual(original);
231|```
232|
233|Notice the emphasis on `.toEqual()` rather than strict equality — both structures must have matching keys and values. If any field survives the transformation incorrectly, the test fails immediately. Passing this suite guarantees that bookmarking a search and reopening it later shows the exact same configuration. Try this yourself: perform a complex search, copy the URL, paste it in a new tab. Same filters, same everything. These round-trip tests prove that magic works.
234|
235|### Chip Merging Tests: Deduplication Discipline
236|
237|These tests make sure `mergeChipIntoForm` both adds new chips *and* rejects duplicates:
238|
239|```typescript
240|const base = { fandoms: ["Avengers"] };
241|const merged = mergeChipIntoForm(base, {
242|  field: "fandoms",
243|  label: "Spider-Man",
244|  reason: "chip-click",
245|});
246|expect(merged.fandoms).toEqual(["Avengers", "Spider-Man"]);
247|
248|// Clicking the same chip again does NOT create a duplicate
249|const alreadyThere = mergeChipIntoForm(merged, {
250|  field: "fandoms",
251|  label: "Spider-Man",
252|  reason: "chip-click",
253|});
254|expect(alreadyThere.fandoms.length).toBe(2);  // Still exactly 2
255|```
256|
257|The deduplication check happens *inside* `mergeChipIntoForm` — before appending a new value, it scans the existing array for that label. This prevents the classic UI bug where repeated clicking accumulates `[ "Tag", "Tag", "Tag" ]`. That's a frustrating mistake that users encounter in poorly tested software everywhere, and these explicit tests make sure FicHub stays cleaner.
258|
259|### Inert Field Tests: Verifying Nothing Happens
260|
261|Perhaps surprisingly, there are dedicated tests confirming that inert fields *don't* influence the output:
262|
263|```typescript
264|const result = buildSearchQuery({
265|  rating: "Teen",          // Inert — ignored
266|  crossover: true,         // Inert — ignored
267|  hitCount: 5000,          // Inert — ignored
268|  language: "en",          // Inert — ignored
269|});
270|// None of these properties should appear in the result
271|expect(result.rating).toBeUndefined();
272|expect(result.crossover).toBeUndefined();
273|```
274|
275|Why dedicate test lines to proving nothing happens? Because these are *behavioral contracts*. Anyone reading the source code can see "this field is inert," but the tests guarantee it stays inert even when other developers refactor surrounding code. Documentation in code is aspirational; tests are enforceable.
276|
277|### Running the Tests Locally
278|
279|To run these tests during development:
280|
281|```bash
282|cd frontend
283|npm test -- searchForm.test.ts
284|```
285|
286|This runs only the 47 tests in this specific file — ideal when you're actively editing searchForm.ts and want rapid feedback. To run the complete project test suite:
287|
288|```bash
289|cd frontend
290|npm test
291|```
292|
293|All 47 tests should flash through green ✅ in under a second. If any test fails, the runner reports the failing assertion with the expected versus actual values side by side. This immediate clarity is one of the enormous advantages automated tests have over manual quality assurance — you never have to guess what went wrong.
294|
295|⚠️ **Watch Out: Test Coverage Awareness**
296|Forty-seven tests across five public functions averages roughly nine tests per function — solid coverage, but not exhaustive. Some less-common code paths inside `buildSearchQuery` may still lack dedicated assertions. Watch for branches where conditional logic exists but no test exercises it. Over time, every new feature should arrive with its own test(s) — it's much easier to add tests alongside new code than to retrofit them later.
297|
298|That wraps up searchForm.test.ts! Those 47 tests stand guard over your pure module, ensuring correctness at every level. Now let's scale up to the massive component that brings all this logic to life. 🔧
299|
300|---
301|
302|## Chapter 16: ArchiveWorkSearchForm — The Two-Mode Component
303|
304|🏗️ Here it is — the largest file in the entire archive frontend: **ArchiveWorkSearchForm.svelte**, clocking in at **853 lines**. This single Svelte component brings together everything from searchForm.ts and renders the beautiful filtering interface that FicHub users rely on.
305|
306|But here's the cool part — this component plays in two different modes, adapting its shape depending on where it's placed on the page.
307|
308|### Mode 1: Full Page Mode (`mode="page"`)
309|
310|In page mode, the form is the star of the show. The screen is dedicated entirely to search configuration. The form is organized into three `<fieldset>` sections, each grouping related fields:
311|
312|**First Fieldset: Work Info**
313|
314|This section handles basic text metadata:
315|- **Title** — enter the title or part of a title
316|- **Author/Audience** — filter by creator username
317|- **Chapters Status** — choose completed, ongoing, or any
318|- **Word Count** — min and max word count inputs
319|
320|```html
321|<!-- Simplified structure -->
322|<fieldset>
323|  <legend>Work Info</legend>
324|  <label>Title
325|    <input bind:value={state.title} />
326|  </label>
327|  <label>Author
328|    <input bind:value={state.author} />
329|  </label>
330|  <!-- More fields... -->
331|</fieldset>
332|```
333|
334|Each `<label>` wraps an `<input>`, using Svelte's `bind:value` directive to keep the form state in sync with the DOM in real time. No event listeners needed!
335|
336|**Second Fieldset: Work Tags**
337|
338|This section is where the richness of FanFiction tags shines. Each row has a select dropdown:
339|- Fandoms
340|- Characters
341|- Relationships
342|- Additional Tags
343|- Warnings
344|- Categories
345|
346|```html
347|<fieldset>
348|  <legend>Work Tags</legend>
349|  <label>Fandoms
350|    <select multiple bind:value={state.fandoms}>
351|      <option value="Marvel">Marvel</option>
352|      <option value="Star Wars">Star Wars</option>
353|      <!-- hundreds more... -->
354|    </select>
355|  </label>
356|  <label>Characters
357|    <select multiple bind:value={state.characters}>
358|      <!-- characters populate dynamically -->
359|    </select>
360|  </label>
361|  <!-- More tag rows... -->
362|</fieldset>
363|```
364|
365|Multiple selection (via `multiple` attribute) lets users pick many fandoms or characters at once. Behind the scenes, each selected option gets pushed into the corresponding array in the `$state` object. But here's a subtlety that makes this component special: the select dropdowns don't contain static options like a simple HTML form would. Instead, they populate from a large dataset of known tags pulled during component initialization. When ArchiveWorkSearchForm mounts, it fires off requests to fetch available values for fandoms, characters, relationships, and every other tag type. Those arrays live in Svelte stores and feed directly into `<option>` elements through reactive bindings. The result is that users see exactly the fandoms and characters that exist in the FicHub database — no typos, no ghost entries — which dramatically reduces search frustration.
366|
367|**Third Fieldset: Work Stats**
368|
369|The final group deals with numeric metrics and sorting:
370|- **Kudos:** min and max inputs
371|- **Comments:** min and max inputs  
372|- **Bookmarks:** min and max inputs
373|- **Rating:** radio buttons (General Audiences, Teen, Mature, Explicit)
374|- **Sort Options:** radio buttons (date, kudos, hits, reviews, chapters)
375|
376|💡 **Key Concept: Radio Buttons vs. Inputs**
377|Ratings and sort options use `<input type="radio">` because you only want one choice at a time. Radio buttons enforce mutual exclusion automatically — selecting "Mature" deselects "Teen." Numeric fields use number inputs because you're specifying values, not choosing from a fixed list.
378|
379|### Mode 2: Sidebar Mode (`mode="sidebar"`)
380|
381|Sidebar mode is where things get really clever. It reuses the *exact same form fields* — every single input, every select dropdown, every radio button — but it wraps them in a completely different visual packaging. Instead of big visible `<fieldset>` blocks that dominate the screen, each group lives inside a collapsible `<details>/<summary>` HTML element:
382|
383|```html
384|<details>
385|  <summary>Work Info</summary>
386|  <div class="sidebar-filters">
387|    <!-- Same fields, but collapsed by default -->
388|    <label>Title
389|      <input bind:value={state.title} />
390|    </label>
391|  </div>
392|</details>
393|
394|<details>
395|  <summary>Work Tags</summary>
396|  <!-- Collapsed by default -->
397|</details>
398|```
399|
400|Notice something subtle? The `bind:value` directive still works perfectly. Even though these inputs live inside collapsed `<details>` sections that users must explicitly open to interact with, Svelte's two-way binding keeps `$state` fully synchronized exactly as it does in page mode. There's no separate data channel for sidebar inputs — it's the same reactive plumbing underneath. This means you can build a massive component once and render it anywhere without duplicating event handlers or state logic.
401|
402|The key behavioral difference: `<details>` elements are *closed* by default in sidebar mode but *open* by default in page mode. Users click the summary headers to expand individual sections. Why design it this way? Because in sidebar mode, the form occupies precious vertical real estate alongside an already-dense results list. Showing everything expanded would push results off-screen and force constant scrolling. Collapsing reduces the form to just a row of clickable headers until the user actively needs a filter.
403|
404|But there's one exception to the collapse rule: the **sort options** section never collapses. In sidebar mode, sort controls are pinned above all the `<details>` blocks so they're always immediately accessible. This reflects an important UX observation from actual usage data — changing sort order is one of the most frequently repeated actions users perform while browsing search results. Making it persistently visible saves dozens of clicks per session across the entire user base.
405|
406|Side-by-side comparison:
407|
408|| Feature | Page Mode | Sidebar Mode |
409||---------|-----------|--------------|
410|| Layout | Full-width `<fieldset>` rows | Compact `<details>/<summary>` blocks |
411|| Visibility | Always open | Collapsible (closed by default) |
412|| Sort options | At bottom of stats section | Always visible at top (sticky) |
413|| Use case | Standalone search page | Paired with results list |
414|| Scroll behavior | Form scrolls naturally with page | Form panel pinned via CSS `position: sticky` |
415|| Initialization cost | Loads tag selects eagerly | Also loads tags eagerly (shared initialization) |
416|
417|💡 **Key Concept: DRY Architecture Through Parameterized Rendering**
418|One file doing double duty like this is the essence of DRY (Don't Repeat Yourself) in software. The ArchiveWorkSearchForm component accepts a `mode` prop that determines its rendering strategy — it doesn't fork into two separate files. Every field definition exists exactly once, which means fixing a bug in "how kudos min works" fixes it in both page and sidebar simultaneously. Without this parameterization pattern, maintaining two copies of nearly identical forms would be a maintenance nightmare and a breeding ground for inconsistencies.
419|
420|### The Chips Row
421|
422|Above (or below, depending on mode) the form, if the user is logged in, a row of chip buttons appears. These chips come from the `/api/search/suggest` endpoint:
423|
424|```javascript
425|// Fetch suggested tags/chips
426|const response = await fetch("/api/search/suggest");
427|const chips = await response.json();
428|// Shows up to 8 chip buttons
429|```
430|
431|Each chip button displays a tag name. Clicking one triggers `mergeChipIntoForm`, which adds the tag to the appropriate field. The chips help users discover relevant tags without knowing them by heart — it's like autocomplete for fandom culture!
432|
433|### Disabled Fields (The "Coming Soon" Section)
434|
435|Three fields appear in the form but don't actually work yet:
436|
437|1. **Crossovers** — has a no-op footnote explaining it's planned but not implemented
438|2. **Hits** — similarly shown with a no-op footnote
439|3. **Language** — displayed as a placeholder labeled "coming soon"
440|
441|⚠️ **Watch Out: User Frustration Prevention**
442|These disabled fields could frustrate users who try to use them and nothing happens. The footnotes help clarify that these are upcoming features. But ideally, disabled fields should be visually distinct (grayed out, non-clickable) so the user immediately understands they're not active yet.
443|
444|### The Data Flow: From Click to Navigation
445|
446|Here's the complete lifecycle of a search submission:
447|
448|```
449|1. User clicks "Search" button
450|   ↓
451|2. onSubmit fires → calls buildSearchQuery(state)
452|   ↓
453|3. Query object is built from current $state
454|   ↓
455|4. navigate() redirects to /search?q=<encoded-query>
456|   ↓
457|5. URL updates in browser address bar
458|   ↓
459|6. search/+page.svelte picks up the "q" parameter
460|   ↓
461|7. Results render on the left panel
462|```
463|
464|💡 **Key Concept: Svelte's $state Magic**
465|The form state lives in a `$state` variable (Svelte 5's reactive state declaration). Every time a user interacts with any input — typing, selecting, checking a checkbox — the `$state` object updates instantly. Svelte's reactivity system ensures all bindings stay synchronized automatically. No manual event handling, no Redux store updates, no complexity.
466|
467|That's ArchiveWorkSearchForm! In 853 lines, it packs two complete layouts, dozens of filters, chip support, and a clean data flow. Up next: how the search page ties it all together. 🎬
468|
469|---
470|
471|## Chapter 17: Search Page Integration
472|
473|With the form component done, it's time to put it on a real page! The search page lives at `frontend/src/routes/search/+page.svelte`, and it's responsible for bringing together the search results and the search form in a single cohesive experience.
474|
475|### The Decision Tree: What Does the User Want?
476|
477|When someone lands on `/search`, the page has to answer a simple question: **Are they searching, or just browsing?** The answer comes from the URL parameters:
478|
479|```svelte
480|<script>
481|  import { page } from '$app/stores';
482|  import ArchiveWorkSearchForm from '$lib/ui/archive/ArchiveWorkSearchForm';
483|  import SearchResults from '$lib/components/search/SearchResults';
484|  
485|  // Read the "q" parameter from the URL
486|  const q = $page.url.searchParams.get('q');
487|  // Or read individual filter params directly
488|  
489|  let filters = {};
490|  if (q) {
491|    // Parse the encoded query string
492|    filters = decodeFilters(q);
493|  }
494|</script>
495|```
496|
497|The route checks for a `q` query parameter:
498|- **No `q` parameter** → the user wants a fresh search form
499|- **Has `q` parameter** → the user wants to see search results
500|
501|### Scenario A: Fresh Search (No Query Parameter)
502|
503|Without a `q` parameter, the page shows the full-page search form. This is the welcoming entrance — blank canvas, ready to receive the user's wishes:
504|
505|```svelte
506|{#if !q}
507|  <ArchiveWorkSearchForm mode="page" on_submit={handleSearch} />
508|{/if}
509|```
510|
511|The form takes up most of the screen. Users fill in whatever filters they want and hit Submit. Behind the scenes, `buildSearchQuery` converts the state, then the page navigates to `/search?q=...`, triggering Scenario B.
512|
513|### Scenario B: Actual Search (Query Parameter Present)
514|
515|With a `q` parameter, the page transforms into a powerful two-panel dashboard:
516|
517|```svelte
518|{#if q}
519|  <div class="search-layout">
520|    <div class="results-panel">
521|      <SearchResults filters={filters} />
522|      <Pagination total={response.total} />
523|    </div>
524|    
525|    <div class="form-panel">
526|      <ArchiveWorkSearchForm 
527|        mode="sidebar" 
528|        state={restoredState}
529|        on_submit={handleSearch} 
530|      />
531|      
532|      <ChipsRow if:isLoggedIn />
533|    </div>
534|  </div>
535|{/if}
536|```
537|
538|Left column: Search results listing (work cards with titles, authors, summary snippets, kudos counts, chapter info). Right column: the sidebar form, still active and editable. Bottom: pagination controls.
539|
540|💡 **Key Concept: Live Editing**
541|Because the sidebar form shares state with the results display, users can change a filter and see results update without refreshing the page. This live-editing behavior is a hallmark of a great search UX. Try increasing the word count minimum while looking at results — the list refreshes immediately.
542|
543|### Pagination
544|
545|Below the results, pagination helps users browse beyond the first page:
546|
547|```svelte
548|<Pagination 
549|  total={response.total}
550|  currentPage={page.number}
551|  perPage={page.per_page}
552|/>
553|```
554|
555|The `total` field tells us how many matching works exist. Combined with `per_page` (usually 20), the pagination calculates how many pages to show. Navigation buttons let the user jump between pages without resubmitting the whole form. This works because the search state lives in the URL itself — changing pages simply updates the query parameter from `?page=1` to `?page=2`, and Svelte automatically re-fetches the appropriate chunk of results. The beauty here is that the browser's back button works naturally: clicking it returns to page 1, forward returns to page 2. Users who accidentally click too deep can always navigate their history.
556|
557|⚠️ **Watch Out for Edge Cases in Pagination:** When a search yields exactly zero results, the Pagination component must handle gracefully without crashing or displaying "Page 1 of NaN." The component checks for a valid `total` before rendering navigation buttons. Similarly, when only one page of results exists, all pagination controls hide entirely — no point showing a single-button pager. These edge cases seem minor but matter enormously for polish: a blank screen with "No results found" reads far better than a broken page counter.
558|
559|### The Chips Row Below the Form
560|
561|When the user is logged in, a chips row renders below the sidebar form. These chips are fetched from `/api/search/suggest` and show up to 8 clickable tags. Clicking a chip adds it to the active filter set and the results update:
562|
563|```svelte
564|{#if isLoggedIn}
565|  <div class="chips-row">
566|    {#each chips as chip}
567|      <button class="chip-btn" on:click={() => addChip(chip)}>
568|        {chip.label}
569|      </button>
570|    {/each}
571|  </div>
572|{/if}
573|```
574|
575|### CSS Layout: The Two-Column Magic
576|
577|The layout uses modern CSS grid or flexbox to split the page:
578|
579|```css
580|.search-layout {
581|  display: grid;
582|  grid-template-columns: 2fr 1fr;  /* results get 2/3, form gets 1/3 */
583|  gap: 2rem;
584|  padding: 1rem;
585|}
586|
587|.results-panel { min-width: 0; }
588|.form-panel { position: sticky; top: 1rem; }
589|```
590|
591|The results panel gets `min-width: 0` — a small CSS detail that actually does something important. Without it, long work titles inside result cards can overflow their container and break the grid layout entirely. Setting `min-width: 0` tells the browser "this element *may* shrink below its content's natural width," which lets the two-column layout breathe properly even when someone has a story titled "An Incredibly Long Title That Goes On And On Forever And Probably Includes Special Characters And Numbers Too."
592|
593|The form panel is made sticky so it stays visible while scrolling through long result lists. As users scroll down page after page of results, the sidebar form follows along — they never have to scroll back up to change filters. This is a small detail that makes a big difference for usability, especially on searches with hundreds of results spanning dozens of pages.
594|
595|💡 **Key Concept: Loading States**
596|Every transition between scenarios needs a loading state. When the user submits a search, the results panel should show a spinning loader rather than a blank space. The search page achieves this by watching the API response status and rendering a skeleton UI during the wait. Skeleton screens — gray placeholder blocks shaped like upcoming results — keep the page looking active even while data travels across the network. This prevents the jarring experience of watching a screen flicker from empty to populated in a single flash.
597|
598|⚠️ **Watch Out: Mobile Responsiveness**
599|On narrow screens (phones, tablets), the two-column layout needs to stack into a single column. The results panel goes first (since it's what the user came for), and the sidebar form slides below it. Good responsive design ensures the search experience works on all devices.
600|
601|### The Complete Search Lifecycle
602|
603|Let me walk through the full journey one more time, from click to results:
604|
605|1. **User opens** `/search` with no parameters
606|2. **Full-page form** appears with all filters available
607|3. **User fills in** Title="Dragon", Word Count min=10000
608|4. **User clicks Submit**
609|5. **onSubmit handler** calls `buildSearchQuery(state)` → `{ title: "Dragon", word_count: "10000-" }`
610|6. **Navigator redirects** to `/search?q=title%3ADragon%2Bwordcount%3A10000-`
611|7. **+page.svelte detects** the `q` parameter
612|8. **Two-column layout** renders: results on left, sidebar form on right
613|9. **Search API client** calls `search(filters)` with the parsed filters
614|10. **Results list** populates with matching works
615|11. **Chips row** suggests related tags below the form
616|12. **User tweaks** filters → results update → repeat until satisfied
617|
618|💡 **Key Concept: Progressive Enhancement**
619|The search page degrades gracefully. If JavaScript fails, the form still submits via traditional POST and returns results on a fresh page load. If the API is down, friendly error messages tell the user what happened instead of showing broken UI. This resilience is what separates amateur apps from professional ones.
620|
621|That's the search page! You've seen how routing, layout, forms, and data fetching come together. Now let's peek at the final layer: the API connection. 🔌
622|
623|---
624|
625|## Chapter 18: Search API & Backend Connection
626|
627|We've explored the UI components thoroughly. Now let's follow the trail one step further — how does the frontend talk to the backend when actually executing a search?
628|
629|### The SearchFilters Interface
630|
631|At the bridge between frontend and backend sits the `SearchFilters` interface, defined in `frontend/src/lib/api/search.ts`:
632|
633|```typescript
634|interface SearchFilters {
635|  title?: string;
636|  author?: string;
637|  fandoms?: string[];
638|  characters?: string[];
639|  relationships?: string[];
640|  additionalTags?: string[];
641|  warnings?: string[];
642|  categories?: string[];
643|  ratings?: string[];
644|  wordCountMin?: number;
645|  wordCountMax?: number;
646|  kudosMin?: number;
647|  kudosMax?: number;
648|  commentsMin?: number;
649|  commentsMax?: number;
650|  bookmarksMin?: number;
651|  bookmarksMax?: number;
652|  sortBy?: string;
653|  sortOrder?: 'asc' | 'desc';
654|  page?: number;
655|  perPage?: number;
656|}
657|```
658|
659|This interface defines every possible search parameter. Each field maps directly to a column or index in the backend database. If the user didn't touch the Comments min field, that property simply won't appear in the object — keeping requests lean.
660|
661|### The Search Function
662|
663|The main entry point is the `search()` function:
664|
665|```typescript
666|import { search } from '$lib/api/search';
667|
668|const response = await search({
669|  title: 'Dragon',
670|  wordCountMin: 1000,
671|  sortBy: 'kudos',
672|  sortOrder: 'desc',
673|  page: 1,
674|});
675|```
676|
677|This function takes a `SearchFilters` object and sends it to the backend API endpoint. The actual HTTP request might look like:
678|
679|```
680|GET /api/v1/search?q=title:Dragon&min_words:1000&sort=kudos:desc&page=1
681|```
682|
683|The `search()` function handles encoding, error handling, and JSON parsing automatically.
684|
685|### The SearchResponse Shape
686|
687|After sending the request, the backend responds with a rich `SearchResponse` object:
688|
689|```typescript
690|interface SearchResponse {
691|  total: number;       // Total matches across all pages
692|  page: number;        // Current page number
693|  per_page: number;    // Items per page
694|  results: WorkItem[]; // Array of matching works
695|  facets: Facets;      // Breakdown statistics
696|}
697|
698|interface WorkItem {
699|  id: number;
700|  title: string;
701|  author: string;
702|  summary?: string;
703|  fandoms: string[];
704|  kudosCount: number;
705|  commentCount: number;
706|  bookmarkCount: number;
707|  wordCount: number;
708|  chapterCount: number;
709|  status: 'Completed' | 'Ongoing' | 'Hiatus';
710|  publishedAt: string;
711|  updatedAt: string;
712|  // ... many more fields
713|}
714|```
715|
716|💡 **Key Concept: Pagination Metadata**
717|The `total`, `page`, and `per_page` fields are essential for building accurate pagination. With `total=147` and `per_page=20`, the UI knows to show 8 pages. Changing `page=2` in the next request fetches results 21–40.
718|
719|### Rating Resolution Magic
720|
721|Ratings deserve special attention. When the frontend sends a rating like "Explicit," the backend internally resolves it to **tag type 7** (the internal classification for ratings in the archive's taxonomy). This mapping is handled transparently:
722|
723|```typescript
724|// Frontend sends:
725|{ ratings: ["Explicit"] }
726|
727|// Backend resolves internally:
728|// Tag type 7 = "Explicit" in the rating vocabulary
729|// Query becomes: WHERE tag_type_id = 7 AND tag_name IN ('Explicit')
730|```
731|
732|This indirection allows the frontend to speak human-readable names while the backend works with normalized tag IDs. If the archive decides to rename a rating in the future, only the resolution mapping needs updating — the frontend code stays untouched. This decoupling is one of those quiet architectural decisions that saves enormous headaches during long-term maintenance. Imagine wanting to change a label from "Mature" to "M for Mature" across the entire site; with direct numeric references you'd need a global find-and-replace through dozens of files. With this indirection layer, you update one lookup table and everything else follows.
733|
734|💡 **Key Concept: The Facets Response Format**
735|Facets aren't just metadata — they're structured data designed specifically for interactive filtering. Each facet entry carries both a `label` (human-readable) and a `count` (how many results match that tag *given current filters*). This means facets are relative, not absolute. If you search for "Marvel" and then refine by "Time Travel," the facet counts for remaining tags will shift because they reflect the filtered universe, not the full database. Some APIs return static facet counts computed before any filtering happens — which would be misleading. FicHub's facets adapt dynamically based on what the user has already selected.
736|
737|### Facets: Breaking Down Results
738|
739|Here's a power feature that makes the search experience really useful: **facets**. After running a search, the backend returns a `facets` object containing breakdowns:
740|
741|```typescript
742|const facets = response.facets;
743|/* Example structure:
744|{
745|  fandoms: [
746|    { label: "Marvel Cinematic Universe", count: 4200 },
747|    { label: "DCU", count: 1800 },
748|    { label: "Star Wars", count: 3200 },
749|    // ...
750|  ],
751|  characters: [
752|    { label: "Tony Stark", count: 5600 },
753|    { label: "Steve Rogers", count: 4100 },
754|    // ...
755|  ],
756|  warnings: [
757|    { label: "Graphic Depictions Of Violence", count: 1200 },
758|    // ...
759|  ],
760|  categories: [
761|    { label: "F/M", count: 8900 },
762|    { label: "M/M", count: 6700 },
763|    // ...
764|  ]
765|}
766|*/
767|```
768|
769|Facets answer the question: "Among the results I'm seeing, how many belong to each category?" This is incredibly powerful for exploration. Imagine finding 5,000 dragon-themed fics — the facets tell you how many are Marvel vs. Star Wars vs. Harry Potter without leaving the results page.
770|
771|Facets power the sidebar suggestions and are often displayed alongside results to help users refine their search directionally:
772|
773|```
774|Showing 5,000 results for "dragon"
775|Top fandoms: Marvel (4,200) • Star Wars (3,200) • DCU (1,800)
776|Click any to add as a filter!
777|```
778|
779|⚠️ **Watch Out: Facet Performance**
780|Computing facets is expensive — the backend must scan through all matching works to count distribution across categories. For searches with millions of results, this can take significant time. The API may include a `facets_delayed` flag or limit facets to the top N entries to avoid timeouts.
781|
782|### The Complete Request Pipeline
783|
784|Let's trace one more time, from the UI down to the wire:
785|
786|```
787|┌─────────────────────────────────┐
788|│  User clicks Search             │
789|│  ArchiveWorkSearchForm          │
790|│  (Chapter 16)                   │
791|├─────────────────────────────────┤
792|│  buildSearchQuery(state)        │
793|│  searchForm.ts (Chapter 14)     │
794|│  Produces: {title:"...",kudos:..}│
795|├─────────────────────────────────┤
796|│  navigate(/search?q=...)        │
797|│  Router updates URL             │
798|├─────────────────────────────────┤
799|│  search/+page.svelte            │
800|│  Detects q param (Chapter 17)   │
801|│  Calls search(filters)          │
802|├─────────────────────────────────┤
803|│  search(API client)             │
804|│  search.ts (this chapter)       │
805|│  Sends GET /api/v1/search?...   │
806|├─────────────────────────────────┤
807|│  Backend receives request       │
808|│  Queries PostgreSQL + Elasticsearch│
809|│  Computes facets                │
810|├─────────────────────────────────┤
811|│  Returns SearchResponse         │
812|│  {total, page, results[], fac..}│
813|├─────────────────────────────────┤
814|│  SearchResults renders          │
815|│  Cards, facets, pagination      │
816|└─────────────────────────────────┘
817|```
818|
819|💡 **Key Concept: Separation of Concerns**
820|Every layer has one job:
821|- **searchForm.ts**: transform state → query
822|- **test suite**: verify correctness
823|- **ArchiveWorkSearchForm**: collect user input
824|- **Search Page**: orchestrate layout and state
825|- **API client**: communicate with backend
826|- **Backend**: query database and return results
827|
828|None of these layers depend on more than they need. Swap the backend? Change the UI? The interfaces stay the same, only internals change. That's solid software architecture! 🎓
829|
830|---
831|
832|## Wrapping Up Part 4
833|
834|You did it! You've walked through the entire search system of FicHub's archive frontend:
835|
836|- **Chapter 14** taught you pure functions (searchForm.ts) that convert user input into queries
837|- **Chapter 15** showed you how 47 tests keep that module bulletproof
838|- **Chapter 16** revealed the 853-line two-mode component that powers the search experience
839|- **Chapter 17** connected the form to the results page with responsive layouts
840|- **Chapter 18** traced requests all the way to the backend and back
841|
842|The search system is one of the most complex pieces of FicHub, and now you understand how it fits together. Every click, every filter, every chip tag flows through the pipeline we just studied. Next up: Part 5 will cover the Collection System — how users organize their favorite works into curated shelves! 📚
1|# Part 5 — Page-by-Page Restyle
2|
3|Welcome to the final stretch! 🎉 In this part, you'll tour every single page in the FicHub archive that received a restyle treatment. By Chapter 4 you learned how individual components look when used in isolation. Now it's time to see them assembled into real pages — landing pages, tag directories, bookmark managers, forums, settings, and more. Each page takes those building pieces and arranges them into something a human actually uses every day.
4|
5|Think of a page like a house built from bricks. The components are your bricks — walls, windows, doors — but only by putting them together in the right order do they become livable space. That's what these six chapters will show you: the full room layouts.
6|
7|---
8|
9|## Chapter 19: ArchiveHome — The Landing Page
10|
11|You've just opened FicHub. Before you clicks anywhere, you land on the home page — the digital front door of the entire archive. This is `ArchiveHome.svelte`, a 470-line component that serves as the primary navigation hub for everyone who isn't searching yet.
12|
13|### The Compact Download Input
14|
15|Right at the top sits a deceptively simple input box. This is the **compact download field** — a quick way to paste a FicHub work ID or URL and jump straight to reading or downloading. Instead of using a large hero banner, the designers kept it compact so there's more screen real estate for actual content below.
16|
17|```svelte
18|<!-- ArchiveHome.svelte — compact download input -->
19|<section class="archive-section">
20|  <input type="text" placeholder="Enter work ID to download..." />
21|  <ArchiveButton on:click={handleDownload}>Download</ArchiveButton>
22|</section>
23|```
24|
25|The `ArchiveButton` here wraps around a standard HTML `<button>` but carries the Archive CSS styling automatically — no extra classes needed. Polymorphic components like this save you from having to remember which button class to apply on every page.
26|
27|### Two Columns: Recent Works Meets Trending Now
28|
29|Below the download input, the layout splits into two columns. On the left, the main column hosts **Recent Works** — stories that have been published recently, shown as WorkBlurb cards. On the right sidebar sits **Trending Now**, a curated list of popular stories.
30|
31|```svelte
32|<!-- ArchiveHome.svelte — two-column layout -->
33|<div class="archive-grid">
34|  <main class="archive-main">
35|    <h2>Recent Works</h2>
36|    {#each recentWorks as work}
37|      <WorkBlurb {work} />
38|    {/each}
39|  </main>
40|  <aside class="archive-sidebar">
41|    <h2>Trending Now</h2>
42|    {#each trending as work}
43|      <WorkBlurb {work} />
44|    {/each}
45|  </aside>
46|</div>
47|```
48|
49|The `archive-grid` class sets up a classic two-column layout using CSS Grid. The main area takes up the majority of the width while the sidebar occupies a fixed column — similar to how AO3 structures its home page. If you inspect the CSS, you'll find something like:
50|
51|```css
52|.archive-grid {
53|  display: grid;
54|  grid-template-columns: 1fr 300px;
55|  gap: var(--archive-gap);
56|}
57|```
58|
59|This is a responsive design too — when the viewport narrows, the grid collapses into a single column and the sidebar moves below the main content. No media queries magic required if the base template already handles it; Svelte's declarative structure makes reordering straightforward.
60|
61|### Personalized Recommendations (For Logged-In Users Only)
62|
63|Here's where things get clever. When a user is logged in *and* has set enough preferences in their profile data, a third section appears: **Personalized Recommendations**. This is pulled from the `/api/recs/personalized` endpoint. But critically — it only shows if both conditions are true:
64|
65|```svelte
66|<!-- ArchiveHome.svelte — conditional personalized section -->
67|{#if $authStore.isLoggedIn && $prefsStore.hasEnoughData}
68|  <section class="archive-section">
69|    <h2>Recommended For You</h2>
70|    {#each recs as work}
71|      <WorkBlurb {work} />
72|    {/each}
73|  </section>
74|{/if}
75|```
76|
77|This pattern — checking a store variable for login state combined with a property check on user prefs — is the canonical way to gate any feature that requires personalization across the entire app. If the conditions aren't met, the section simply doesn't render. No flicker, no blank card, nothing wasted.
78|
79|### Parallel Data Loading
80|
81|Fetching three separate sections (recent works, trending, personalized recs) could easily slow down the page load. If you fetched them one after another sequentially, the total wait time would be sum of all three requests. Instead, the page fires all three simultaneously using `Promise.allSettled`:
82|
83|```typescript
84|// ArchiveHome.svelte — parallel data fetching
85|const [recentRes, trendingRes, recsRes] = await Promise.allSettled([
86|  fetch("/api/recent"),
87|  fetch("/api/trending"),
88|  isLoggedIn && prefs.hasEnoughData ? fetch("/api/recs/personalized") : Promise.resolve(null),
89|]);
90|```
91|
92|Why `allSettled` instead of the more common `all`? Because `Promise.allSettled` doesn't abort the other requests if one fails. Imagine the personalized recs endpoint times out — with `Promise.all`, both "Recent Works" and "Trending Now" would fail too. With `allSettled`, they still load successfully and the user sees partial content rather than a blank page. The code checks each promise's `status` field (`"fulfilled"` vs `"rejected"`) and renders whichever data came back:
93|
94|```typescript
95|const recentWorks = recentRes.status === "fulfilled" ? recentRes.value.data : [];
96|const trending = trendingRes.status === "fulfilled" ? trendingRes.value.data : [];
97|const recs = recsRes.status === "fulfilled" ? recsRes.value.data : [];
98|```
99|
100|This defensive approach means your users always see *something*, even when an API misbehaves.
101|
102|### Skeleton States: Shimmer While Waiting
103|
104|During the loading period before any data arrives, the page doesn't sit blank. Each section independently renders skeleton placeholders — rows of shimmering gray bars that mimic the shape of WorkBlurbs. These use CSS animations driven by `@keyframes pulse`:
105|
106|```svelte
107|<!-- ArchiveHome.svelte — shimmer skeleton loader -->
108|{#if loading}
109|  <div class="skeleton-row">
110|    <div class="shimmer"></div>
111|    <div class="shimmer short"></div>
112|  </div>
113|  <div class="skeleton-row">
114|    <div class="shimmer"></div>
115|    <div class="shimmer short"></div>
116|  </div>
117|  <!-- ... repeated for expected number of items ... -->
118|{/if}
119|```
120|
121|Each skeleton row looks roughly like a WorkBlurb — a wider bar for the title, a shorter bar for metadata. The shimmer effect is pure CSS:
122|
123|```css
124|@keyframes shimmer-pulse {
125|  0% { opacity: 0.4; }
126|  50% { opacity: 0.7; }
127|  100% { opacity: 0.4; }
128|}
129|.shimmer {
130|  background: #e0e0e0;
131|  animation: shimmer-pulse 1.5s ease-in-out infinite;
132|  border-radius: var(--archive-radius);
133|}
134|```
135|
136|The beauty of independent skeletons per section is that if "Trending Now" finishes first, it drops in immediately while Recent Works still shimmers. Your eyes catch the updates one section at a time — much less jarring than a single spinner that covers everything at once.
137|
138|> **🧪 Try It Yourself**
139|>
140|> Open the FicHub home page and keep the Network tab open in your browser dev tools. Watch how three XHR requests fire simultaneously. Notice how the response times differ — some APIs respond faster than others. The skeleton UI disappears independently for each section, exactly as described above.
141|
142|---
143|
144|## Chapter 20: Tags Page — A Brand New Route
145|
146|If ArchiveHome existed before Phase 2, the Tags page did not. It was created fresh as part of the refactoring effort. Gone was the idea that browsing tags required knowing specific search URLs. Here comes `/tags/+page.svelte` — a directory of every tag in the system, organized neatly by category.
147|
148|### Fetching All Six Tag Types in Parallel
149|
150|Every tag on FicHub belongs to one of six categories. The page loads them all at once:
151|
152|```typescript
153|// /tags/+page.svelte — fetch all tag types
154|const [fandoms, characters, relationships, additionalTags, warnings, categories] =
155|  await Promise.all([
156|    fetch("/api/tags?type=1").then(r => r.json()),
157|    fetch("/api/tags?type=2").then(r => r.json()),
158|    fetch("/api/tags?type=3").then(r => r.json()),
159|    fetch("/api/tags?type=4").then(r => r.json()),
160|    fetch("/api/tags?type=5").then(r => r.json()),
161|    fetch("/api/tags?type=6").then(r => r.json()),
162|  ]);
163|```
164|
165|The API module lives at `frontend/src/lib/api/tags.ts`, a dedicated module for anything tag-related. Keeping it separate from the general API client keeps imports clean:
166|
167|```typescript
168|// frontend/src/lib/api/tags.ts
169|export async function getTags(type: number) {
170|  const res = await fetch(`/api/tags?type=${type}`);
171|  if (!res.ok) throw new Error("Failed to fetch tags");
172|  return res.json();
173|}
174|```
175|
176|### Organized by Category
177|
178|Once all six arrays of tag data arrive, the template groups them under labeled headers. The organization mirrors AO3's tag directory style: a bold gray label followed by a vertical list of maroon links. The key insight here is using an array-of-object iterator — instead of writing six separate `{#each}` blocks (one for each tag type), you define a single loop that maps types to labels dynamically:
179|
180|```svelte
181|<!-- /tags/+page.svelte — grouped tag rendering -->
182|{#each [
183|  { type: 1, label: "Fandoms", data: fandoms },
184|  { type: 2, label: "Characters", data: characters },
185|  { type: 3, label: "Relationships", data: relationships },
186|  { type: 4, label: "Additional Tags", data: additionalTags },
187|  { type: 5, label: "Warnings", data: warnings },
188|  { type: 6, label: "Categories", data: categories },
189|] as group}
190|  <section class="tag-group">
191|    <h2 class="tag-group-title">{group.label}</h2>
192|    <ul class="tag-list">
193|      {#each group.data as tag}
194|        <li>
195|          <a href="/search?include_tags={group.type}:{tag.name}" class="maroon-link">
196|            {tag.name} <span class="tag-count">({tag.usage_count})</span>
197|          </a>
198|        </li>
199|      {/each}
200|    </ul>
201|  </section>
202|{/each}
203|```
204|
205|This approach has a nice side effect: if a new tag type gets added later (say, "Alternate Universe Tropes" becomes type 7), you only need to add one line to the array, not duplicate a whole section of HTML. Scalability through repetition avoidance — a principle worth remembering whenever your Svelte templates start looking like a copy-paste factory.
206|
207|Each tag link includes the usage count in parentheses — that little `(247)` next to "Marvel" tells readers how many works carry that tag. The destination URL embeds the tag type as a prefix: `include_tags=1:Marvel Cinematic Universe`. The search system then filters exclusively within that tag category. This URL structure is intentional: it lets the search engine know exactly which taxonomy bucket to look in, avoiding ambiguity between a character named "Marvel" and a fandom called "Marvel."
208|
209|#### Alphabetical Sorting
210|
211|The backend returns tags sorted alphabetically within each category, but the frontend can reorder them if needed. For example, you might want to sort by popularity instead:
212|
213|```svelte
214|<!-- Sort tags by usage count descending -->
215|{#each [...group.data].sort((a, b) => b.usage_count - a.usage_count) as tag}
216|```
217|
218|Notice the spread operator `[...group.data]` creates a shallow copy before sorting. If you sorted `group.data` directly, you'd mutate the original data fetched from the API — Svelte's reactivity would fire unexpectedly, and sorting Fandoms could accidentally reorder Characters too. The spread ensures each group's data stays independent.
219|
220|> **💡 Key Concept — Why Spread Before Sort?**
221|>
222|> JavaScript arrays are reference types. Calling `.sort()` on an existing array mutates it in place. By spreading into a new array first, you get a fresh array that `.sort()` modifies without touching the original. It's a tiny safety net that prevents subtle bugs where one section's sort order bleeds into another.
223|
224|#### Scaling to Thousands of Tags
225|
226|FicHub doesn't paginate its tag directory — every single tag renders on one page. This means the browser receives a large DOM tree at once. That's fine because:
227|
228|1. Each tag row is lightweight HTML — just an `<a>` element inside an `<li>`, nothing heavy.
229|2. There's no image loading, no network calls per item, no JavaScript event handlers attached individually.
230|3. Readers who need to find a specific tag use their browser's native Find-in-Page (Ctrl+F / Cmd+F), which is instant against a fully-rendered document.
231|
232|If FicHub ever grows to hundreds of thousands of tags, this pattern might need revisiting — pagination or virtual scrolling could enter the conversation. But for the current scale, the "render everything at once" approach is fast enough and much simpler to implement and maintain.
233|
234|### Styling Details
235|
236|The visual treatment reinforces the AO3 aesthetic without copying it pixel-for-pixel:
237|
238|```css
239|.tag-group-title {
240|  color: #555;
241|  font-weight: 600;
242|  border-bottom: 1px solid var(--archive-border);
243|  padding-bottom: 0.25rem;
244|  margin-bottom: 0.5rem;
245|}
246|
247|.maroon-link {
248|  color: var(--color-maroon);
249|  text-decoration: none;
250|}
251|
252|.maroon-link:hover {
253|  text-decoration: underline;
254|}
255|
256|.tag-count {
257|  color: #888;
258|  font-size: 0.85em;
259|}
260|```
261|
262|The group header uses a medium-gray weight instead of the archive's traditional maroon red — a deliberate choice to create visual hierarchy. The eye scans from the category heading down through the alphabetically sorted tag links. And because there's no pagination, a tag-heavy site like FicHub benefits from the browser's native page-search (Ctrl+F) — the list is fully rendered in the DOM at once, making it instant to find what you need.
263|
264|> **💡 Key Concept — Why Group Headers?**
265|>
266|> A flat list of thousands of tags would be overwhelming. By grouping into six categories, each section stays digestible. It also mirrors how authors think about tagging: "Oh, I need a *fandom* tag, a couple *character* tags, and maybe one *additional* tag." The interface follows the mental model.
267|
268|---
269|
270|## Chapter 21: Bookmarks, Authors, Notifications
271|
272|Three pages that serve different user workflows, but all share the same archive conditional treatment. Let's walk through each one.
273|
274|### Bookmarks — Saved Works, One Click Away
275|
276|The bookmarks page lives at `/bookmarks/+page.svelte` and applies the archive styling conditionally. Under the hood, each bookmark maps to a `WorkBlurb` card — the exact same component used on search results and the home page:
277|
278|```svelte
279|<!-- /bookmarks/+page.svelte — bookmark listing -->
280|{#each bookmarks as bookmark}
281|  <WorkBlurb work={bookmark.work} />
282|  {#if bookmark.notes}
283|    <p class="bookmark-notes">{bookmark.notes}</p>
284|  {/if}
285|{/each}
286|```
287|
288|#### Graceful Handling of Deleted Works
289|
290|Here's where the design shines. Sometimes authors delete their fics, or admins remove works for policy violations. What happens when a bookmarked work vanishes? The page doesn't crash or show a broken card. Instead, it falls back to a minimal card showing just the work ID, the reader's notes, and a "Remove" button:
291|
292|```svelte
293|<!-- /bookmarks/+page.svelte — deleted work fallback -->
294|{#if bookmark.work}
295|  <WorkBlurb work={bookmark.work} />
296|{:else}
297|  <div class="deleted-work-card">
298|    <span>Work #{bookmark.workId}</span>
299|    {#if bookmark.notes}
300|      <p class="bookmark-notes">{bookmark.notes}</p>
301|    {/if}
302|    <ArchiveButton on:click={() => removeBookmark(bookmark.id)}>Remove</ArchiveButton>
303|  </div>
304|{/if}
305|```
306|
307|This `deleted-work-card` div uses the same archive border and spacing tokens as regular blurbs, so visually it blends in seamlessly. The reader loses nothing beyond the work content itself. The Remove button lets them clean up their bookmark list without manual database edits.
308|
309|#### Pagination and Scroll Behavior
310|
311|For users who have saved hundreds of bookmarks, the API paginates results automatically. Each page fetch adds another batch of WorkBlurbs or deleted-work-cards to the DOM. The bookmark page typically loads 25 items per page — enough to feel like a substantial browsing session while keeping each HTTP response small. If you're curious how pagination works under the hood, the query parameter is just `page=N` appended to the `/api/bookmarks` endpoint call, passed through SvelteKit's standard URL search parameters.
312|
313|> **💡 Key Concept — Bookmark Notes as Meta-Layers**
314|>
315|> Every bookmark can carry freeform text notes — the reader's own annotations about why they saved that story. "This fic saved me during finals week" or "Perfect for rereading before Christmas." Those notes render beneath each WorkBlurb when present, turning your bookmark list into something between a reading log and a personal diary. It's a small feature with outsized emotional value.
316|
317|### Authors — People Search Made Simple
318|
319|The `/authors/+page.svelte` page is wonderfully minimal: a plain People Search tool. An input field, a search button, and a result list. Despite its simplicity, it demonstrates a useful pattern — using the archive wrapper for a non-archive feature. The author search component lives entirely inside an archive-styled shell even though searching authors isn't specifically about fanfiction archives; it's a general people-discovery utility that happens to use the same visual language.
320|
321|```svelte
322|<!-- /authors/+page.svelte — author search -->
323|<input
324|  type="text"
325|  bind:value={query}
326|  placeholder="Search authors..."
327|  class="archive-input"
328|/>
329|<ArchiveButton on:click={searchAuthors}>Search</ArchiveButton>
330|
331|{#if loading}
332|  <div class="loading-skeleton">
333|    {#each [1, 2, 3] as _}
334|      <div class="shimmer" style="height: 1.5rem;"></div>
335|    {/each}
336|  </div>
337|{/if}
338|
339|{#if query && !loading}
340|  <ul class="author-list">
341|    {#each results as author}
342|      <li>
343|        <a href="/authors/{author.id}" class="maroon-link">
344|          {author.display_name}
345|        </a>
346|      </li>
347|    {/each}
348|  </ul>
349|{/if}
350|```
351|
352|Notice the `archive-input` class on the text field — a small touch that gives the search box the same bordered, spaced look as inputs in the search form. The loading skeleton mirrors the patterns we saw on ArchiveHome: three stacked shimmer bars during the fetch. This consistency matters because readers move fluidly between searching for stories and searching for authors. Their eyes expect the same visual rhythm regardless of which search bar they're typing into.
353|
354|The API call that powers this is straightforward:
355|
356|```typescript
357|async function searchAuthors() {
358|  loading = true;
359|  try {
360|    const res = await fetch(`/api/authors?q=${encodeURIComponent(query)}`);
361|    results = await res.json();
362|  } finally {
363|    loading = false;
364|  }
365|}
366|```
367|
368|A subtle but important point: the `finally` block ensures `loading` always returns to `false`, even if the API throws an error. Without it, a failed request would leave the skeletons permanently displayed — a jarring "stuck loading" state that makes readers think the page has frozen. Defensive programming doesn't have to be complicated.
369|
370|No archive-specific components are needed beyond the wrapper elements. The author list renders as plain links — the archiving cosmetics come from the shared styling classes applied to inputs, buttons, and skeletons.
371|
372|> **🧪 Try It Yourself**
373|>
374|> Go to the Authors search page, clear the input box, then type an author name slowly character by character. Notice that each keystroke *doesn't* trigger a new search — only clicking the Search button does. This is intentional: the page avoids firing excessive network requests for every single typed character. The debounce approach (auto-searching on each keypress) looks cleaner but wastes bandwidth. Sometimes doing one thing at a time explicitly is the better design choice.
375|
376|### Notifications — Flat, Clean Rows
377|
378|Notifications take the opposite approach from Bookmarks. Instead of cards with depth, they use a flat list — simple rows displaying title, body preview, date, and a read/unread indicator. No `WorkBlurb`, no nested components, just clean HTML structure:
379|
380|```svelte
381|<!-- /notifications/+page.svelte — notification listing -->
382|<div class="notification-header">
383|  <ArchiveButton on:click={markAllRead}>Mark All Read</ArchiveButton>
384|</div>
385|
386|{#each notifications as notification}
387|  <div class={notification.read ? "notification-row read" : "notification-row unread"}>
388|    <h3>{notification.title}</h3>
389|    <p class="notification-body">{notification.bodyPreview}</p>
390|    <time datetime={notification.date}>{formatDate(notification.date)}</time>
391|  </div>
392|{/each}
393|```
394|
395|The "Mark All Read" button sits at the very top in the `notification-header`. Every other element follows in a flat cascade. There's no sidebar, no filtering, no complexity. Just a chronological stream of activity.
396|
397|The `notification-row` class applies the archive border styling:
398|
399|```css
400|.notification-row {
401|  border: 1px solid var(--archive-border);
402|  padding: var(--archive-padding-sm);
403|  margin-bottom: var(--archive-gap);
404|}
405|
406|.notification-row.unread {
407|  border-left: 3px solid var(--color-maroon);
408|  background: var(--archive-bg-hover);
409|}
410|```
411|
412|Unread notifications get a distinctive left border in maroon and a slightly darker background — an instantly recognizable visual cue that doesn't require reading any text.
413|
414|#### Marking Individual Notifications
415|
416|Clicking on a notification row navigates to its detail view and marks it read. The state flips server-side, so the next page load shows the notification without the maroon left-border highlight. This gives readers a clear sense of progress through their notification backlog.
417|
418|> **⚠️ Watch Out for Stale Data**
419|>
420|> If the user leaves notifications open in a tab while simultaneously clicking "Mark All Read" on another tab, the first tab won't automatically update until refreshed. To prevent this mismatch, some teams add a WebSocket subscription or a periodic poll that syncs notification states across tabs. FicHub currently uses a simple approach: the settings page can be toggled to force a refresh when returning to the notifications tab via the Visibility API (`document.addEventListener('visibilitychange', ...)`). It's not perfect but avoids the complexity of real-time subscriptions.
421|
422|---
423|
424|## Chapter 22: Requests & Forum
425|
426|FicHub isn't just about reading stories — it's also about community. The Request system lets fans post wish-lists for specific story prompts, and the Forum provides threaded discussions. Both receive the full archive restyle treatment.
427|
428|### Requests — Browse, Create, and Detail Views
429|
430|The Requests system spans three distinct views, each with its own template:
431|
432|#### List View (`/requests`)
433|
434|The index page displays all open requests as rows with three key pieces of information: title, status badge, and upvote count.
435|
436|```svelte
437|<!-- /requests/+page.svelte — request list -->
438|{#each requests as request}
439|  <div class="request-row">
440|    <a href="/requests/{request.id}" class="maroon-link">
441|      {request.title}
442|    </a>
443|    <span class="status-badge status-{request.status}">
444|      {request.status}
445|    </span>
446|    <span class="upvote-count">▲ {request.upvotes}</span>
447|  </div>
448|{/each}
449|```
450|
451|Status badges use CSS classes prefixed with the status value, allowing color-coded appearance without JavaScript logic:
452|
453|```css
454|.status-pending { background: #fff3cd; color: #856404; }
455|.status-completed { background: #d4edda; color: #155724; }
456|.status-rejected { background: #f8d7da; color: #721c24; }
457|```
458|
459|#### Detail View (`/requests/:id`)
460|
461|When viewing a single request, the content is wrapped in a `<fieldset>` element styled with archive conventions — the same pattern used in search forms and settings:
462|
463|```svelte
464|<!-- /requests/[id]/+page.svelte — request detail -->
465|<fieldset class="archive-fieldset">
466|  <legend class="field-legend">{request.title}</legend>
467|  
468|  <dl class="detail-definition">
469|    <dt>Status</dt><dd>{request.status}</dd>
470|    <dt>Posted by</dt><dd>{request.author.display_name}</dd>
471|    <dt>Date</dt><dd>{formatDate(request.created_at)}</dd>
472|    <dt>Upvotes</dt><dd>{request.upvotes}</dd>
473|  </dl>
474|  
475|  <div class="request-description">{request.description}</div>
476|</fieldset>
477|```
478|
479|The `<fieldset>` + `<legend>` combination is semantically meaningful — it says "this entire block is one form-like unit" — and the archive styles make it look beautiful with borders and indentation.
480|
481|The `<dl>` (definition list) pattern is used throughout for key-value pairs. It's semantically cleaner than `<div>` spam and requires zero extra classes since the archive stylesheet targets `<dl>` inside `.archive-fieldset`.
482|
483|#### New Request Form (`/requests/new`)
484|
485|Creating a new request uses a `<dl>`-based input layout — rows of label/value pairs:
486|
487|```svelte
488|<!-- /requests/new/+page.svelte — new request form -->
489|<form on:submit|preventDefault={submitRequest}>
490|  <fieldset class="archive-fieldset">
491|    <legend class="field-legend">Create a New Request</legend>
492|    
493|    <dl class="form-dl">
494|      <dt><label for="title">Title</label></dt>
495|      <dd><input id="title" name="title" bind:value={form.title} /></dd>
496|      
497|      <dt><label for="description">Description</label></dt>
498|      <dd><textarea id="description" name="description" bind:value={form.description}></textarea></dd>
499|      
500|      <dt><label for="prompt">Prompt Tags</label></dt>
501|      <dd>
502|        <input id="prompt" bind:value={form.promptTag} />
503|        <button type="button" on:click={addSeedChip}>Add</button>
504|        <div class="seed-chips">
505|          {#each form.seeds as chip}
506|            <span class="seed-chip">{chip}<button type="button" on:click={() => removeSeed(chip)}>×</button></span>
507|          {/each}
508|        </div>
509|      </dd>
510|    </dl>
511|    
512|    <ArchiveButton type="submit">Submit Request</ArchiveButton>
513|  </fieldset>
514|</form>
515|```
516|
517|The **seed chips** deserve special attention. They're small pill-shaped labels representing tag suggestions that the user adds incrementally. Each chip shows its text and a small × button to remove it. Below the chips, the description renders as freeform text. This incremental-add pattern keeps the form tidy even when a user wants to attach ten tags to a single request.
518|
519|### Forum — Tables and Bordered Boxes
520|
521|The forum takes a different visual approach: bordered tables for lists, and bordered boxes for individual posts.
522|
523|#### Category Index (`/forum/`)
524|
525|The main forum page shows all discussion categories as a bordered table:
526|
527|```svelte
528|<!-- /forum/+page.svelte — category index table -->
529|<table class="forum-table">
530|  <thead>
531|    <tr>
532|      <th>Title</th>
533|      <th>Topics</th>
534|      <th>Posts</th>
535|      <th>Last Post</th>
536|    </tr>
537|  </thead>
538|  <tbody>
539|    {#each categories as category}
540|      <tr>
541|        <td>
542|          <a href="/forum/{category.slug}" class="maroon-link">
543|            {category.name}
544|          </a>
545|        </td>
546|        <td>{topicCount(category.slug)}</td>
547|        <td>{postCount(category.slug)}</td>
548|        <td class="last-post-date">{formatDate(category.lastPostAt)}</td>
549|      </tr>
550|    {/each}
551|  </tbody>
552|</table>
553|```
554|
555|The table uses archive styling — `--archive-border` on borders, compact row heights, and maroon-colored links. Crucially, the `border-radius` is set to `0` everywhere to maintain the sharp, utilitarian archive aesthetic:
556|
557|```css
558|.forum-table {
559|  border-collapse: collapse;
560|  width: 100%;
561|}
562|
563|.forum-table th, .forum-table td {
564|  border: 1px solid var(--archive-border);
565|  padding: var(--archive-padding-sm);
566|  border-radius: 0;
567|}
568|
569|.forum-table th {
570|  background: var(--archive-bg-header);
571|  font-weight: 600;
572|}
573|```
574|
575|#### Topic List (`/forum/:slug`)
576|
577|Individual category topic lists use the same table pattern with slightly different columns — typically Topic | Author | Replies | Last Activity. Reusing the `.forum-table` class means zero duplicate CSS between these two pages.
578|
579|#### Post Detail (`/forum/:slug/:id`)
580|
581|Threaded posts break away from the table pattern. Each post is rendered as an individual boxed div with a gray header row inside:
582|
583|```svelte
584|<!-- /forum/[slug]/[id]/+page.svelte — post rendering -->
585|{#each posts as post}
586|  <div class="forum-post-box">
587|    <div class="post-header">
588|      <span class="post-author">{post.author.display_name}</span>
589|      <time datetime={post.created_at}>{formatDate(post.created_at)}</time>
590|    </div>
591|    <div class="post-content">{post.content}</div>
592|  </div>
593|{/each}
594|```
595|
596|```css
597|.forum-post-box {
598|  border: 1px solid var(--archive-border);
599|  margin-bottom: var(--archive-gap);
600|}
601|
602|.post-header {
603|  background: var(--archive-bg-header);
604|  padding: var(--archive-padding-sm);
605|  display: flex;
606|  justify-content: space-between;
607|  align-items: center;
608|}
609|
610|.post-author {
611|  color: var(--color-maroon);
612|  font-weight: 600;
613|}
614|```
615|
616|The gray header row inside each box separates metadata (author + date) from content. The author name uses maroon text to tie back to the overall archive palette. Posts stack vertically without threading indentation — each box stands alone on equal footing.
617|
618|#### New Topic Form (`/forum/new`)
619|
620|The new topic form mirrors the Requests form pattern: a `<fieldset>` with `<dl>` rows for inputs:
621|
622|```svelte
623|<form on:submit|preventDefault={submitTopic}>
624|  <fieldset class="archive-fieldset">
625|    <legend class="field-legend">Start a New Topic</legend>
626|    <dl class="form-dl">
627|      <dt><label for="title">Topic Title</label></dt>
628|      <dd><input id="title" bind:value={form.title} /></dd>
629|      <dt><label for="content">Body</label></dt>
630|      <dd><textarea id="content" bind:value={form.content} rows="10"></textarea></dd>
631|    </dl>
632|    <ArchiveButton type="submit">Create Topic</ArchiveButton>
633|  </fieldset>
634|</form>
635|```
636|
637|The consistent use of `<fieldset>` + `<legend>` + `<dl>` across Requests and Forms means every community feature feels like it belongs to the same family, even though the underlying data and interactions differ dramatically.
638|
639|---
640|
641|## Chapter 23: Ask, Settings, Fic Detail
642|
643|Three final pages that round out the archive experience — asking questions to the Archive AI, configuring preferences, and reading a full fic.
644|
645|### Ask the Archive — Q&A with Style
646|
647|The Ask page at `/ask/+page.svelte` is a simple question-and-answer interface. Users type a prompt into a large textarea, submit it, and receive results displayed as WorkBlurb cards below.
648|
649|```svelte
650|<!-- /ask/+page.svelte — ask interface -->
651|<h1 class="ask-heading">Ask the Archive</h1>
652|
653|<fieldset class="archive-fieldset">
654|  <legend class="field-legend">Your Question</legend>
655|  <textarea
656|    id="ask-input"
657|    bind:value={question}
658|    rows="5"
659|    placeholder="What kind of story are you looking for?"
660|  ></textarea>
661|  <ArchiveButton on:click={submitQuestion} disabled={!question.trim()}>
662|    Ask the Archive
663|  </ArchiveButton>
664|</fieldset>
665|
666|{#if error}
667|  <div class="error-note">
668|    <small>⚠️ {error}</small>
669|  </div>
670|{/if}
671|
672|{#if results.length > 0}
673|  <div class="ask-results">
674|    {#each results as work}
675|      <WorkBlurb {work} />
676|    {/each}
677|  </div>
678|{/if}
679|```
680|
681|The textarea gets archive-style borders through shared input CSS. The Ask heading uses the archive serif style for consistency. Results appear below the form as the same `WorkBlurb` cards used everywhere else — reinforcing that search results and ask results are fundamentally the same data shape.
682|
683|Errors render in AO3's footnote style — a small, understated note with a warning icon, placed near where the action failed:
684|
685|```css
686|.error-note {
687|  border: 1px solid #f5c6cb;
688|  padding: var(--archive-padding-sm);
689|  margin-top: var(--archive-gap);
690|}
691|
692|.error-note small {
693|  color: #721c24;
694|  font-size: 0.85em;
695|}
696|```
697|
698|### Settings — Choosing Your Interface Flavor
699|
700|The Settings page at `/settings/+page.svelte` is famously minimalist — a single section, a few radio buttons, and a Save button. But don't let its simplicity fool you: it controls the most fundamental toggle in the entire application.
701|
702|```svelte
703|<!-- /settings/+page.svelte — interface style selector -->
704|<fieldset class="archive-fieldset">
705|  <legend class="field-legend">Interface Style</legend>
706|  <div class="radio-group">
707|    <label>
708|      <input
709|        type="radio"
710|        name="interfaceStyle"
711|        value="modern"
712|        bind:checked={$prefsStore.interfaceStyle === 'modern'}
713|      />
714|      Modern
715|    </label>
716|    <label>
717|      <input
718|        type="radio"
719|        name="interfaceStyle"
720|        value="archive"
721|        bind:checked={$prefsStore.interfaceStyle === 'archive'}
722|      />
723|      Archive
724|    </label>
725|  </div>
726|  <ArchiveButton on:click={saveSettings}>Save Preferences</ArchiveButton>
727|</fieldset>
728|```
729|
730|Two important observations here:
731|
732|1. The `<fieldset>` with `<legend>` creates a clear visual grouping for the setting options — matching the pattern used in search forms and other form-like sections.
733|2. **Emojis are completely absent.** Throughout Settings, the interface relies on text labels and the existing icon library. The designers made a deliberate choice to keep emojis off UI surfaces to maintain the clean archival aesthetic.
734|
735|When the user clicks "Save," the preference propagates through `$prefsStore` and triggers an immediate re-render of all pages in archive or modern mode. No page refresh needed — the reactive stores handle it. This is one of those moments where you can feel the power of SvelteKit's reactive programming: change one variable, watch half the app update simultaneously.
736|
737|> **💡 Key Concept — Interface Style as State**
738|>
739|> The interface style preference lives in two places simultaneously: the user's account on the server (so it persists across devices), and `$prefsStore` in the browser (so the current session reflects it instantly). When you switch from Modern to Archive, both layers update. If your internet drops right after clicking Save, you won't lose your preference — it's already on the server. The localStorage fallback ensures offline sessions remember their choice even if the server call fails.
740|
741|### Fic Detail — Reading a Full Story
742|
743|The fic detail page at `/fic/[urlId]/+page.svelte` is arguably the most important page in the archive. This is where readers spend hours absorbed in someone else's creativity. The page composes multiple archive components together:
744|
745|```svelte
746|<!-- /fic/[urlId]/+page.svelte — full fic detail layout -->
747|<svelte:head>
748|  <title>{work.title} by {work.author.display_name} — FicHub</title>
749|</svelte:head>
750|
751|<ArchiveWork {work} />
752|
753|<div class="fic-meta">
754|  <div class="tag-table">
755|    {#each work.tags as tag}
756|      <span class="tag-pill tag-{tag.category}">
757|        <a href="/search?include_tags={tag.category}:{tag.name}">
758|          {tag.label}
759|        </a>
760|      </span>
761|    {/each}
762|  </div>
763|</div>
764|
765|<div class="summary-block">
766|  <h3>Summary</h3>
767|  <blockquote>{work.summary}</blockquote>
768|</div>
769|
770|<div class="action-buttons">
771|  <ArchiveButton on:click={kudosAction}>Leave Kudos</ArchiveButton>
772|  <ArchiveButton on:click={commentAction}>Comment</ArchiveButton>
773|  <ArchiveButton on:click={() => bookmark(work.id)}>Bookmark</ArchiveButton>
774|</div>
775|
776|<div class="chapters-list">
777|  <h3>Chapters</h3>
778|  {#each work.chapters as chapter}
779|    <a href="#chapter-{chapter.index}" class="maroon-link">
780|      {chapter.title || `Chapter ${chapter.index}`}
781|    </a>
782|  {/each}
783|</div>
784|
785|<div class="author-notes">
786|  <h3>Author Notes</h3>
787|  {#each work.notes as note}
788|    <div class="note-block">
789|      <small class="note-date">{formatDate(note.date)}</small>
790|      <p>{note.text}</p>
791|    </div>
792|  {/each}
793|</div>
794|
795|<div class="download-section">
796|  <details>
797|    <summary>Download Options</summary>
798|    <ul>
799|      <li><a href="/fic/{work.urlId}/download/epub">EPUB</a></li>
800|      <li><a href="/fic/{work.urlId}/download/pdf">PDF</a></li>
801|      <li><a href="/fic/{work.urlId}/download/text">Plain Text</a></li>
802|    </ul>
803|  </details>
804|</div>
805|```
806|
807|Several things happen on this page:
808|
809|- **Tag Table** — Uses the `TagSoup` component with category-colored pills, identical to how tags render on search results. Clicking a tag navigates to a filtered search.
810|- **Summary Block** — Rendered as a `<blockquote>` with archive-styled borders, giving the summary visual separation from the rest of the page.
811|- **Action Buttons** — Three prominent `ArchiveButton` instances for leaving kudos, commenting, and bookmarking. All styled identically regardless of context.
812|- **Chapters List** — A simple numbered list of anchor links jumping to each chapter on the page. Multi-chapter fics benefit greatly from this internal navigation.
813|- **Author Notes** — Displayed chronologically at the bottom, each wrapped in a small `.note-block` with a timestamp. Readers often treat these as a second narrative thread.
814|- **Download Dropdown** — Uses the native HTML `<details>` element for a collapsible dropdown listing EPUB, PDF, and Plain Text formats. No custom accordion JS needed — the browser handles open/close natively.
815|
816|The `ArchiveWork` component encapsulates the core fic rendering — the actual prose text formatted for comfortable reading, with proper typography and line height adjustments. Everything else wraps around it.
817|
818|---
819|
820|## Chapter 24: ArchiveListPage — The Shared List Component
821|
822|If there's a MVP award for the most reusable component in the archive, it goes to `ArchiveListPage.svelte`. At 216 lines, this component does one thing perfectly: renders a list of items with polished loading, empty, and error states. Multiple pages reuse it, saving hundreds of lines of duplicated code.
823|
824|### Props: The Contract
825|
826|Every consumer passes the same four props plus an optional subtitle:
827|
828|```svelte
829|<!-- ArchiveListPage.svelte — props definition -->
830|<script lang="ts">
831|  export let title: string = "";
832|  export let items: unknown[] = [];
833|  export let loading: boolean = false;
834|  export let errorMessage: string = "";
835|  export let subtitle: string = "";
836|  export let children: Snippet; // Renders item-level slot content
837|</script>
838|```
839|
840|The `children` prop uses Svelte's snippet feature — a way to pass callable JSX-like content that gets rendered repeatedly for each item. This keeps `ArchiveListPage` generic while letting consumers define exactly how each item looks:
841|
842|```svelte
843|<!-- Usage on the home page -->
844|<ArchiveListPage
845|  title="Recent Works"
846|  {items}
847|  {loading}
848|  {errorMessage}
849|>
850|  {@render children}
851|</ArchiveListPage>
852|
853|<!-- Parent provides the item renderer -->
854|<ArchiveListPage title="Trending" {items} {loading} loadingLabel="Loading trending...">
855|  {#snippet children()}
856|    {#each items as work}
857|      <WorkBlurb {work} />
858|    {/each}
859|  {/snippet}
860|</ArchiveListPage>
861|```
862|
863|### Loading Skeleton: Pulsing Blurbs
864|
865|When `loading` is true, the component generates five skeleton blurbs with a pulsing animation:
866|
867|```svelte
868|<!-- ArchiveListPage.svelte — loading skeleton -->
869|{#if loading}
870|  <div class="list-loading">
871|    {#each [1, 2, 3, 4, 5] as _}
872|      <div class="skeleton-blurb">
873|        <div class="skeleton-line title"></div>
874|        <div class="skeleton-line meta"></div>
875|        <div class="skeleton-line tags"></div>
876|      </div>
877|    {/each}
878|  </div>
879|{/if}
880|```
881|
882|Each skeleton blurb contains three horizontal bars of varying widths, mimicking a WorkBlurb's layout: a long title bar, a medium-length metadata bar, and a short tag bar. The pulsing animation runs continuously:
883|
884|```css
885|@keyframes pulse {
886|  0%, 100% { opacity: 0.3; }
887|  50% { opacity: 0.7; }
888|}
889|
890|.skeleton-blurb {
891|  border: 1px solid var(--archive-border);
892|  padding: var(--archive-padding-sm);
893|  margin-bottom: var(--archive-gap);
894|}
895|
896|.skeleton-line {
897|  background: var(--archive-skeleton);
898|  animation: pulse 1.5s ease-in-out infinite;
899|  border-radius: 2px;
900|  margin-bottom: 0.5rem;
901|}
902|
903|.skeleton-line.title {
904|  height: 1rem;
905|  width: 80%;
906|}
907|
908|.skeleton-line.meta {
909|  height: 0.75rem;
910|  width: 50%;
911|}
912|
913|.skeleton-line.tags {
914|  height: 0.5rem;
915|  width: 60%;
916|}
917|```
918|
919|Five skeleton blurbs match the typical default fetch size of results — enough to fill the visible viewport and give the impression of a "real" list coming in.
920|
921|### Empty State: Nothing Found, Nothing Shown
922|
923|When the list is genuinely empty (not loading — just zero results):
924|
925|```svelte
926|<!-- ArchiveListPage.svelte — empty state -->
927|{:else if items.length === 0 && !loading}
928|  <div class="empty-state">
929|    <p>No results found.</p>
930|  </div>
931|{/if}
932|```
933|
934|Notice the design decision: **no icon**. The empty state is deliberately minimal — just text saying "No results found." No emoji, no illustration, no decorative element. This restraint keeps the page focused and avoids the cluttered feeling that icons can bring to a sparse page.
935|
936|### Error State: Red Border, Retry Button
937|
938|When data fetching fails:
939|
940|```svelte
941|<!-- ArchiveListPage.svelte — error state -->
942|{:else if errorMessage}
943|  <div class="error-box">
944|    <p>{errorMessage}</p>
945|    <ArchiveButton on:click={() => retry()}>Retry</ArchiveButton>
946|  </div>
947|{/if}
948|```
949|
950|The error message appears in a red-bordered box with a Retry button underneath. The `retry()` function is provided by the consuming component (home page, search page, etc.), ensuring the retry behavior matches whatever context triggered the fetch.
951|
952|### Who Uses ArchiveListPage?
953|
954|Four major places depend on this shared component:
955|
956|1. **Recent Works on the Home Page** — Shows freshly published stories with loading skeletons while the API resolves.
957|2. **Trending Sidebar** — Displays trending works in a compact sidebar container, sharing the same component for consistency.
958|3. **Search Results Page** — Renders paginated search results with the same loading/empty/error handling as everything else.
959|4. **Bookmark Listing** — Used on the Bookmarks page under the archive conditional to render saved works consistently.
960|
961|By centralizing list presentation in one component, every page that shows a collection of items looks and behaves identically. When the design team decides tomorrow that skeleton animations should be slightly brighter, they change it in one place and every affected page updates automatically.
962|
963|---
964|
965|## Wrapping Up
966|
967|We've covered six pages — the landing page, the tag directory, bookmarks/authors/notifications, requests/forum, ask/settings/detail, and the shared list component that ties them all together. Through it all, a few patterns repeat like a melody:
968|
969|- **Archive containers** (`<fieldset>`, `.archive-section`, `.archive-fieldset`) create visual rhythm across pages.
970|- **Shared components** (`WorkBlurb`, `ArchiveButton`, `TagSoup`) ensure consistency without repetition.
971|- **Skeleton states** provide smooth transitions during loading.
972|- **Graceful degradation** handles missing data and API failures without breaking.
973|
974|These patterns exist because someone decided early on that every page should feel like it belongs to the same family — whether it's a reader discovering their first bookmark or an author posting their last chapter.
975|
976|That's all for Part 5. You now understand every route page in the FicHub archive frontend, from the grand landing page to the humblest notification row. The pieces fit together, and with that knowledge, you're ready to contribute. 🎓
977|
1|# Part 6: Patterns & Best Practices
2|
3|In this part you'll learn the patterns that hold the Archive UI together — the conditional-rendering trick that lets both views coexist, how the CSS stays clean with custom properties, which Svelte 5 runes power everything under the hood, and the testing strategy that keeps bugs from sneaking through. Think of these as the rules-of-thumb every experienced FicHub developer uses daily. You don't need to memorize them by heart; just read through once, play with the 🧪 Try It Yourself boxes, and come back when your own code needs inspiration.
4|
5|---
6|
7|## Chapter 25: The Archive Conditional Pattern
8|
9|Imagine you have two outfits in your closet: your everyday clothes and your superhero costume. Now imagine you could switch between them with a single flick — no tailoring, no sewing, no buying new clothes. That's exactly what the Archive Conditional Pattern does for FicHub pages.
10|
11|### The Core Idea
12|
13|Every route page in the Archive Frontend wraps its **page-specific rendering** inside a simple `if / else` block:
14|
15|```svelte
16|<script>
17|  import { getPref } from '$lib/prefs';
18|  $: uiMode = getPref('uiMode');
19|</script>
20|
21|{#if uiMode === 'archive'}
22|  <!-- archive view here -->
23|{:else}
24|  <!-- modern view (unchanged, untouched) -->
25|{/if}
26|```
27|
28|That's it. Four lines in the script tag. One conditional in the template. Yet this tiny pattern is what makes the entire dual-mode architecture work.
29|
30|### Why This Works So Well
31|
32|Let's look at three big reasons this approach is brilliant:
33|
34|1. **Additive only**: You never touch existing modern-view code. Your changes are *layered on top* like a sticker, not surgery on the original tissue.
35|2. **Zero regression risk**: The modern shell stays 100% intact because it lives in the `{/else}` branch, completely isolated from archive additions.
36|3. **Easy to find**: If something looks weird, you open the `.svelte` file, see the `{#if}`, and instantly know "oh, the archive override goes right there."
37|
38|No clever hacks, no global flag checks buried deep in functions, no mysterious behavior. Just a clear fork in the road at the top of every page.
39|
40|### Step-by-Step Recipe: Adding Archive Support to a New Page
41|
42|Follow these steps whenever you want to add archive mode to a fresh route page. Here's the checklist, in order:
43|
44|#### Step 1 — Import `getPref`
45|
46|At the top of your `<script>` block, make sure you pull in `getPref`:
47|
48|```svelte
49|<script>
50|  import { getPref } from '$lib/prefs';
51|  // ...existing imports...
52|</script>
53|```
54|
55|If `$lib/prefs` isn't importing cleanly, double-check that the path is correct — it resolves relative to the Svelte `$lib` alias.
56|
57|#### Step 2 — Read the Mode Flag
58|
59|Right after your imports, add one reactive line:
60|
61|```svelte
62|$: uiMode = getPref('uiMode');
63|```
64|
65|This is a **reactive declaration** (`$:`). It means every time the preference value changes in localStorage, the variable updates automatically. No manual listeners needed.
66|
67|#### Step 3 — Wrap the Right Thing
68|
69|Here's the most important step. You do **not** wrap the entire page in the conditional. You wrap only the portion that changes.
70|
71|A common mistake is doing this:
72|
73|```svelte
74|<!-- WRONG: Don't wrap the entire page -->
75|{#if uiMode === 'archive'}
76|  <div class="archive-layout">
77|    <h1>Search</h1>
78|    <!-- full page re-implemented -->
79|  </div>
80|{:else}
81|  <div class="modern-layout">
82|    <!-- everything else -->
83|  </div>
84|{/if}
85|```
86|
87|The modern layout often has shared wrappers (navigation bars, footers, modals). Repeating those in both branches duplicates code and creates sync headaches. Instead, think about **what specifically looks different** between archive and modern, and wrap only that piece:
88|
89|```svelte
90|<!-- RIGHT: Wrap only the rendering difference -->
91|<ModernLayout>
92|  <slot />
93|  <ArchiveHeader slot="header" />
94|
95|  <!-- Modern list rendering stays as-is -->
96|  {#if uiMode === 'archive'}
97|    <ArchiveResultList results={results} ficList={ficList} />
98|  {:else}
99|    <ModernResultList results={results} />
100|  {/if}
101|</ModernLayout>
102|```
103|
104|#### Step 4 — Write the Archive Branch
105|
106|Inside the `{#if}` block, use the archive components we built earlier in this book. Import whatever you need, pass your data in, and let the component handle styling and structure.
107|
108|For example, on the bookmarks page you might replace just the list rendering:
109|
110|```svelte
111|<script>
112|  import { getPref } from '$lib/prefs';
113|  import ArchiveBookmarksList from '$lib/ui/archive/ArchiveBookmarksList.svelte';
114|  import ModernBookmarksList from '$lib/ui/modern/ModernBookmarksList.svelte';
115|
116|  const { bookmarks } = $state({ fic: [], total: 0 });
117|  $: uiMode = getPref('uiMode');
118|</script>
119|
120|{#if bookmarks.length > 0 && uiMode === 'archive'}
121|  <ArchiveBookmarksList bookmarks={bookmarks} />
122|{:else if bookmarks.length > 0}
123|  <ModernBookmarksList bookmarks={bookmarks} />
124|{:else}
125|  <p>You haven't bookmarked any fics yet.</p>
126|{/if}
127|```
128|
129|Notice how the fallback (`{#else}`) still shows a sensible message? That's good UX.
130|
131|#### Step 5 — Test Both Modes
132|
133|Open your browser. Switch to Archive Classic, reload, and check the page. Then switch back to Modern and verify nothing broke. If either mode looks wrong, fix it before moving on. Never commit partial support.
134|
135|### Common Mistakes to Avoid
136|
137|⚠️ **Forgeting the `{/else}` branch.** If you write `{#if uiMode === 'archive'}` but never close it, the modern view disappears entirely. Svelte won't even compile this — it will give you an error saying the block is unclosed. But if you accidentally swallow the else into a nested condition, the modern view breaks silently. Always pair your `{#if}` with `{/else}`.
138|
139|⚠️ **Not importing `getPref`.** If you reference `$: uiMode = getPref('uiMode')` without actually importing it, you'll get a runtime error that can be hard to trace because the failure happens on the first render, not at module load time. Double-check the import line.
140|
141|⚠️ **Wrapping too much or too little.** Wrapping the entire page is wasteful. Wrapping nothing means no archive support at all. The sweet spot: identify *one visual piece* per page that differs between modes and put just that inside the conditional.
142|
143|💡 **Key Concept: The Archive Conditional is a Rendering Gate, Not a Logic Gate**
144|
145|This pattern controls what the user **sees**, not what the app **does**. Network requests, authentication checks, state management — all of that runs regardless of which mode is active. The conditional sits purely in the template layer. Keep your logic outside of it.
146|
147|### A Real Example: The Bookmarks Page
148|
149|The bookmarks page is a great case study. The overall page structure — the outer container, the header, the navigation — is identical in both modes. Only the list of bookmarks renders differently:
150|
151|```svelte
152|<script>
153|  import { getPref } from '$lib/prefs';
154|  import ModernBookmarksCard from '$lib/ui/modern/BookmarkCard.svelte';
155|  import ArchiveBookmarkRow from '$lib/ui/archive/ArchiveBookmarkRow.svelte';
156|  import type { BookmarkEntry } from '$lib/types';
157|
158|  const { bookmarks }: { bookmarks: BookmarkEntry[] } = $props();
159|  $: uiMode = getPref('uiMode');
160|</script>
161|
162|<div class="bookmarks-page">
163|  {#each bookmarks as bookmark (bookmark.id)}
164|    {#if uiMode === 'archive'}
165|      <ArchiveBookmarkRow bookmark={bookmark} />
166|    {:else}
167|      <ModernBookmarksCard bookmark={bookmark} />
168|    {/if}
169|  {/each}
170|</div>
171|```
172|
173|See how elegant this is? Each bookmark card flips independently based on the current mode. If you wanted to add a third view later (say, a compact list), you'd just add another branch. The pattern scales gracefully.
174|
175|🧪 **Try It Yourself**
176|
177|Create a new Svelte component called `DemoConditional.svelte` in your project. Put a dropdown that cycles through `"archive"`, `"modern"`, and `"both"` values. Use a conditional to display a different colored box for each mode. Add an alert box that triggers when neither mode is selected. Once it works, rename `"both"` to `"archive-and-modern"` and update the comparison — watch how the conditional handles the new string naturally.
178|
179|---
180|
181|## Chapter 26: CSS Architecture in Archive Mode
182|
183|CSS is where the magic happens — literally. When someone picks "Archive Noir," they're not asking for a JavaScript framework to recolor every element. They're asking CSS custom properties to shift the palette. And that's precisely what happens.
184|
185|### Custom Properties Are the Only Way In
186|
187|Every single pixel of color, border, and background in archive-mode components flows through custom CSS variables. You'll never find a raw hex value like `#990000` hardcoded inside a component's `<style>` block. Instead, you'll always see:
188|
189|```css
190|a {
191|  color: var(--archive-link);
192|}
193|
194|.panel {
195|  background: var(--archive-bg);
196|  border: 1px solid var(--archive-border);
197|}
198|```
199|
200|This rule — *no hard-coded colors in components* — is enforced strictly. Every archive component references `--archive-*` variables exclusively. This design decision has massive consequences, all positive.
201|
202|### The Two Built-In Presets
203|
204|FicHub ships with two theme presets, defined in `src/lib/themes/presets.ts`:
205|
206|| Variable | Archive Classic (light) | Archive Noir (dark) |
207||---|---|---|
208|| `--archive-bg` | `#ffffff` | `#1a1a1a` |
209|| `--archive-link` | `#990000` | `#cc3333` |
210|| `--archive-border` | `#dddddd` | `#333333` |
211|
212|Archive Classic is designed to resemble AO3's default light theme — warm white background, deep red links, light gray borders. Archive Noir is the dark variant — charcoal background, soft coral-red links, subtle dark borders.
213|
214|You can add more presets later by extending this same file. Each new preset follows the exact same shape: a set of `--archive-*` variable assignments.
215|
216|### Scoped Styles: No Leaks, Ever
217|
218|Each Svelte component owns its own `<style>` block:
219|
220|```svelte
221|<style>
222|  .result-title {
223|    color: var(--archive-link);
224|    font-family: Georgia, serif;
225|    margin-bottom: 4px;
226|  }
227|</style>
228|```
229|
230|Svelte scoping automatically transforms class names behind the scenes so styles never leak between components. Archive components can't accidentally style modern components and vice versa. The separation is absolute.
231|
232|There is no global stylesheet that touches archive elements. There are no Tailwind utility classes polluting markup. There are no external CSS frameworks pulling in megabytes of unused rules. Just clean, scoped, dependency-free CSS.
233|
234|### AO3 Aesthetic Principles
235|
236|The archive mode deliberately echoes the aesthetics familiar to long-time AO3 users:
237|
238|- **Thin borders**: `1px solid var(--archive-border)` everywhere. No thick outlines, no shadows.
239|- **Square corners**: `border-radius: 0` on all elements. No rounded buttons, no pill tags.
240|- **Georgia serif headings**: `font-family: Georgia, serif` for titles and labels. The classic web feel.
241|- **Compact spacing**: Tight line-heights, small gaps between rows. Text-dense layouts maximize information per screen.
242|- **Muted emphasis**: Colors are used sparingly — primarily for links and selected states. Everything else stays neutral.
243|
244|These aren't arbitrary choices. Users who enable archive mode expect this specific visual language. Straying from it feels jarring, like opening a book that suddenly switches from serif to Comic Sans.
245|
246|### Theme Switching Flow
247|
248|How does the browser actually change themes when you click a preset? Here's the chain of events:
249|
250|1. **User selects a theme** → clicks "Archive Classic" or "Archive Noir" in Settings.
251|2. **`setPref()` saves to localStorage** → the selection persists across sessions.
252|3. **`applyUiTheme()` sets CSS vars on `document.documentElement`** → this is the magic step. A single function walks through the chosen preset's variables and applies them as inline styles on the root `<html>` element.
253|4. **Browser re-renders everything** → all components using `var(--archive-bg)`, `var(--archive-link)`, etc. automatically reflect the new values. No JS component updates, no Reactivity chains, just pure CSS cascading down.
254|
255|The beauty is in step 4. Because CSS custom properties cascade globally, changing one line on the root element refreshes thousands of component styles simultaneously. No force-re-renders, no manual updates. The browser does all the work.
256|
257|💡 **Key Concept: CSS Variables Are a Cascade Multiplier**
258|
259|When you set `--archive-link: #cc3333` on `<html>`, every single `color: var(--archive-link)` in every archive component inherits that value automatically. One variable, thousands of effects. This is why the architecture is so clean — you centralize configuration at the root and let CSS do the distribution.
260|
261|🧪 **Try It Yourself**
262|
263|Open any archive-mode component's `<style>` block. Find a `color` property using `var(--archive-link)`. Temporarily change it to a plain hex value like `#ff0000` and save. Reload the page — you should see bright red links, ignoring your preset. Now remove the hardcoded value, restore `var(--archive-link)`, and reload again. Notice how the links immediately snap back to whatever your current preset says. You just proved to yourself that the variable pipeline is working.
264|
265|⚠️ **Watch Out: Hardcoded Colors Bypass Theming**
266|
267|If you ever write `color: #990000` directly instead of `color: var(--archive-link)`, that element will stay that color forever, regardless of which preset the user selected. This is the most common CSS mistake in archive development. If a new color is needed, add a new `--archive-*` variable to `presets.ts` and reference it with `var()`. Never skip the indirection.
268|
269|---
270|
271|## Chapter 27: Svelte 5 Runes in Practice
272|
273|Svelte 5 introduced a paradigm shift called "runes" — special compiler directives that make reactivity explicit rather than implicit. If you've worked with Svelte 4 before, you already know the old `$:` syntax. Svelte 5 replaces some of that with clearer, more powerful primitives.
274|
275|Let's walk through every rune used in the Archive Frontend, with real examples.
276|
277|### `$props()`: Declaring Component Inputs
278|
279|In Svelte 4, you declared props with `export let`:
280|
281|```svelte
282|<script>
283|  export let fic;
284|  export let isBookmarked = false;
285|</script>
286|```
287|
288|In Svelte 5, you destructure `$props()` with TypeScript types:
289|
290|```svelte
291|<script>
292|  interface Props {
293|    fic: SearchResult;
294|    isBookmarked?: boolean;
295|  }
296|  const { fic, isBookmarked = false } = $props<Props>();
297|</script>
298|```
299|
300|This looks slightly more verbose, but the gains are real:
301|
302|- **Type safety**: The `Props` interface catches typos and missing fields at compile time.
303|- **Defaults in destructuring**: `isBookmarked = false` reads naturally.
304|- **Compiler awareness**: Svelte's compiler knows exactly which props exist and can warn you if you misspell one.
305|
306|Every single archive component uses `$props()`. Search results cards, form fields, header bars — they all follow this pattern. It's become our standard.
307|
308|💡 **Key Concept: `$props()` Is Your Component's Contract**
309|
310|Think of the `Props` interface as a contract between parent and child. The parent promises to provide certain values; the child promises to receive them. TypeScript enforces this contract, and Svelte ensures the values update reactively when the parent changes.
311|
312|### `$state()`: Reactive Local State
313|
314|Use `$state()` when a component needs its own mutable, reactive data:
315|
316|```svelte
317|<script>
318|  const [formState, setFormState] = $state(<FormState>{/* ... */});
319|  const [isOpen, setIsOpen] = $state(false);        // dropdown toggle
320|  const [selectedChipIndex, setSelectedChipIndex] = $state<number>(-1);
321|</script>
322|
323|<button onclick={() => setIsOpen(!isOpen)}>Menu</button>
324|```
325|
326|Three examples above, three distinct use cases:
327|
328|1. **`formState`** holds the complete search form configuration (query text, tags, sort order, etc.). Changing one field inside it triggers automatic re-renders of any computed values depending on it.
329|2. **`isOpen`** is a simple boolean controlling whether a dropdown menu is visible. Toggling it flips visibility.
330|3. **`selectedChipIndex`** tracks which tag chip the user has clicked for editing — negative means "nothing selected."
331|
332|Note: `$state()` can take an initial value in parentheses (like `$state(0)` or `$state([])`) or via assignment (like `$let count = $state(0)`). We use the parenthesized form consistently across the archive codebase.
333|
334|### `$derived()`: Computed Values
335|
336|Replace Svelte 4's `$:` reactive declarations with `$derived()` for computed properties:
337|
338|```svelte
339|<script>
340|  // Svelte 4 way:
341|  // $: activeNavLink = $page.url.pathname.split('/').pop();
342|
343|  // Svelte 5 way:
344|  const activeNavLink = $derived($page.url.pathname.split('/').pop());
345|  const filteredChips = $derived(chips.filter(chip => chip.active));
346|</script>
347|```
348|
349|Two key differences from `$:`:
350|
351|1. `$derived()` returns a **value** you assign to a constant. It's not a side-effect declaration.
352|2. The expression inside `$derived()` must be a single expression — no multiple statements. If you need complex logic, extract it to a `$state()`-backed callback instead.
353|
354|In the archive nav, `activeNavLink` changes automatically whenever `$page.url.pathname` changes (because `$page` is SvelteKit's reactive page store). The navigation bar highlights the correct link without any manual listener.
355|
356|### `$effect()`: Side Effects
357|
358|Use `$effect()` when you need to run code that affects the outside world — DOM mutations, network calls, subscriptions:
359|
360|```svelte
361|<script>
362|  // Close dropdowns when clicking outside
363|  $effect(() => {
364|    const handler = (e: MouseEvent) => {
365|      if (!dropdownRef.current?.contains(e.target as Node)) {
366|        setIsOpen(false);
367|      }
368|    };
369|    document.addEventListener('click', handler);
370|    return () => document.removeEventListener('click', handler);
371|  });
372|</script>
373|```
374|
375|The cleanup function (the `return () => ...`) is crucial. Without it, the event listener accumulates every time the effect re-runs, leaking memory.
376|
377|Real-world `$effect()` usages in archive code include:
378|
379|- **Outside-click handlers** for closing dropdown menus.
380|- **Theme initialization** on mount — reading the saved preference and applying it.
381|- **URL synchronization** — updating the browser address bar when form state changes.
382|
383|💡 **Key Concept: `$effect()` ≠ Lifecycle Hooks**
384|
385|Don't think of `$effect()` as `onMount` or `onDestroy`. It's reactive: it re-runs whenever the values it references change. The cleanup function runs before each re-run. This dual behavior is what makes `$effect()` powerful but also potentially tricky — you need to reason about *when* it re-triggers, not just *when* it fires once.
386|
387|### `{@snippet}` and `{@render}`: Slots, Evolved
388|
389|Svelte 4 used named slots:
390|
391|```svelte
392|<SlotExample><div slot="header">Hello</div></SlotExample>
393|```
394|
395|Svelte 5 replaces this with typed snippets:
396|
397|```svelte
398|<!-- Parent passes content -->
399|<ArchiveLayout>
400|  <div slot="header">{@render header()}</div>
401|</ArchiveLayout>
402|
403|<!-- Child declares the snippet slot -->
404|<script>
405|  interface SlotProps {
406|    header?: () => VNode;
407|  }
408|  const { header } = $props<SlotProps>();
409|</script>
410|
411|<Layout>
412|  <nav>{@render header?.()}</nav>
413|</Layout>
414|```
415|
416|In the Archive Frontend, `{@render header()}` appears in `ArchiveLayout` to allow custom headers per page. The parent provides a snippet function; the child calls it wherever it wants the content injected. Type-safe, explicit, and composable.
417|
418|⚠️ **Watch Out: Remember the Parentheses**
419|
420|When rendering a snippet, you call it like a function: `{@render header()}`. Forgetting the parentheses (`{@render header}`) renders the function definition itself, not its output. You'll see `[Function header]` in the page — a confusing debug experience. Always add `()`.
421|
422|### Migration Map: Svelte 4 → Svelte 5 for Archive Devs
423|
424|If you're coming from Svelte 4, here's a quick translation table:
425|
426|| Svelte 4 | Svelte 5 | Notes |
427||---|---|---|
428|| `export let x = 5` | `const { x = 5 } = $props()` | Mechanical, drop-in replacement |
429|| `$: doubled = x * 2` | `const doubled = $derived(x * 2)` | Mostly mechanical |
430|| `$: if (x) doThing()` | `$effect(() => { if (x) doThing(); })` | Needs care — effects always run on mount |
431|| `<Component slot="name">` | `<Component>{@render snippet()}</Component>` | Requires snippet prop in child |
432|| `.store.svelte` files | Local `$state()` + `$derived()` | No stores needed in archive code |
433|
434|Notice the last row: there are **no `.store.svelte` files** anywhere in the Archive Frontend. All state is local to components (via `$state()`) or managed centrally in the prefs store (which handles only user preferences, not page data). This simplifies reasoning about data flow enormously — you can find every piece of state by scanning a single file.
435|
436|🧪 **Try It Yourself**
437|
438|Create a component called `RuneDemo.svelte`. Give it three pieces of state: `count` (number), `name` (string), and `showDetails` (boolean). Derive `greeting` from `name`. Render a button that increments `count` and toggles `showDetails`. Add a `$effect` that logs `count` to the console whenever it changes. Watch the log fire when you click, proving the reactive chain works end-to-end.
439|
440|---
441|
442|## Chapter 28: Testing Strategy
443|
444|Writing code is only half the job. Making sure it keeps working tomorrow — when someone adds a feature, refactors a component, or fixes a typo — is the other half. Testing is how you guarantee that future-you doesn't break present-you's hard work.
445|
446|### Unit Tests: The Detailed Safety Net
447|
448|Unit tests live at `frontend/src/lib/ui/archive/searchForm.test.ts`. This file alone contains **47 passing tests** — each one verifying a specific behavior of the search form logic.
449|
450|Let's walk through what they cover:
451|
452|#### Matrix Row Mapping
453|
454|The search form supports many parameter combinations — filters, sorts, ranges, exclusions. Every single mapping row in the SUPPORTED matrix gets its own test:
455|
456|```typescript
457|it('maps "All Works" filter to query.all_works=true', () => {
458|  const state: FormState = { filter: 'all_works', ...rest };
459|  const params = formStateToUrl(state);
460|  expect(params.get('all_works')).toBe('true');
461|});
462|
463|it('maps sort_by="hits" to URL param hits', () => {
464|  const state: FormState = { sort_by: 'hits', ...rest };
465|  const params = formStateToUrl(state);
466|  expect(params.get('sort_on')).toBe('hits');
467|});
468|```
469|
470|Each test isolates one row of the mapping table. If a row stops mapping correctly, this test fails immediately, pointing you to the exact broken row.
471|
472|#### Range Parsing Edge Cases
473|
474|Range inputs (like word count or kudos range) need to handle messy user input. The tests cover:
475|
476|- **Empty strings** → treated as "no limit"
477|- **Negative numbers** → clamped to zero or rejected
478|- **Malformed ranges** like `"abc-def"` or `"10-"` → parsed gracefully or errored clearly
479|- **Single values** → applied as both min and max
480|
481|```typescript
482|it('parses empty range as unlimited', () => {
483|  const result = parseRange('');
484|  expect(result).toEqual({ min: null, max: null });
485|});
486|
487|it('rejects malformed range "abc-def"', () => {
488|  const result = parseRange('abc-def');
489|  expect(result.hasError).toBe(true);
490|});
491|```
492|
493|#### URL Round-Trip Verification
494|
495|This is perhaps the most valuable test pattern in the entire suite:
496|
497|```typescript
498|it('round-trips through URL conversion', () => {
499|  const original: FormState = {
500|    query: 'drarry',
501|    filter: 'complete',
502|    sort_by: 'kudos',
503|    range_words_min: 5000,
504|    chips: [{ type: 'relationship', label: 'Drarry' }],
505|  };
506|  const urlParams = formStateToUrl(original);
507|  const restored = urlToFormState(urlParams, []);
508|  expect(restored).toEqual(original);
509|});
510|```
511|
512|Convert form state → URL string → back to form state. If the round trip produces anything different, the test fails. This catches silent data loss — like a tag getting dropped during conversion — which would otherwise be invisible until a user noticed their search changed unexpectedly.
513|
514|#### Chip Merge Deduplication
515|
516|When you add a tag chip that already exists, the merge logic should deduplicate:
517|
518|```typescript
519|it('deduplicates chips when merging identical tags', () => {
520|  const existing = [createChip('character', 'Harry')];
521|  const incoming = [createChip('character', 'Harry'), createChip('tag', 'Fluff')];
522|  const merged = mergeChips(existing, incoming);
523|  expect(merged.length).toBe(2); // Harry counted once
524|  expect(merged.some(c => c.label === 'Fluff')).toBe(true);
525|});
526|```
527|
528|Without this, users could accidentally flood their tag bar with duplicates.
529|
530|#### Inert Field Verification
531|
532|Fields marked as "inert" (user disabled them) should never appear in the final query:
533|
534|```typescript
535|it('buildSearchQuery excludes inert params', () => {
536|  const state: FormState = {
537|    query: 'test',
538|    filter: 'complete',
539|    _f_filter: true,  // _f_ prefix marks as inert
540|  };
541|  const query = buildSearchQuery(state);
542|  expect(query).not.toContain('f_complete:true');
543|});
544|```
545|
546|This ensures disabled filters truly disappear from API calls, saving bandwidth and preventing confusing results.
547|
548|### E2E Tests: Full-Page Smoke Tests
549|
550|End-to-end tests live at `frontend/e2e/archive-ui.spec.ts`. These use Playwright to launch a real browser, navigate to actual routes, and verify the page behaves correctly. There are **9 E2E tests** covering the major surfaces:
551|
552|| Test | What It Checks |
553||---|---|
554|| Home loads in archive mode | `/` renders archive layout, not 404 |
555|| Search form renders | Fieldsets for query, filters, ranges all present |
556|| Tags page shows 6 categories | Relationship, Character, Freeform, Rating, Medium, Additional tags listed |
557|| Bookmarks shows login prompt | Unauthenticated users see a sign-in CTA |
558|| Authors page has search input | `/authors` displays an autocomplete-ready search field |
559|| Settings shows Interface Style | `/settings` renders the theme selector section |
560|| Tag chips render correctly | Clickable chip elements with correct labels |
561|| Dropdown closes on outside click | Clicking the page body dismisses open menus |
562|| Theme switch updates colors | Picking Archive Noir changes CSS variable values |
563|
564|These tests don't dig into implementation details. They ask high-level questions: "Can a user land on the page?" "Are the controls visible?" "Does switching themes work?" If all nine pass, the core user journeys are intact.
565|
566|### Coverage Gates
567|
568|The project enforces a minimum code coverage threshold through Vitest:
569|
570|```bash
571|vitest --coverage --reporter=text-summary
572|```
573|
574|Targets are approximately **85% statement**, **78% branch**, and **76% function** coverage on `src/lib/**`. These numbers mean "most of our library code is exercised by tests." They're not perfect — 100% coverage is rarely worth the effort — but hitting these gates gives confidence that new code gets tested before it merges.
575|
576|⚠️ **Known Noise: Pre-existing Test Failures**
577|
578|Be aware that five unit test files unrelated to the archive frontend currently fail due to i18n (internationalization) gaps. These are **known issues, documented separately**, and will be fixed in a future pass. Do not treat these failures as regressions caused by your archive work. Focus your CI attention on the archive-specific test files (`searchForm.test.ts` and `archive-ui.spec.ts`).
579|
580|### What to Test vs. What to Skip
581|
582|Here's a practical guide for writing new tests:
583|
584|**✅ DO test:**
585|- Form state transformations (`formStateToUrl`, `urlToFormState`)
586|- URL parameter parsing and serialization
587|- Chip merge/dedup logic
588|- Inert field filtering
589|- Input validation (empty strings, negatives, overflows)
590|- Theme application (variable values set on root)
591|
592|**❌ DON'T test:**
593|- Styling and layout details — Visual appearance belongs in visual regression tests (separate CI check using screenshot comparison), not unit tests.
594|- Exact pixel positions — Components can shift due to browser quirks; test behavior, not coordinates.
595|- Browser chrome interactions — Tab focus order and keyboard shortcuts are covered by accessibility audits, not automated unit tests.
596|- Third-party library internals — Trust that SvelteKit, Svelte, and Playwright work correctly.
597|
598|💡 **Key Concept: Test the Contract, Not the Implementation**
599|
600|Write tests that describe *what* the code should do, not *how* it does it. If you refactor the internal algorithm but preserve the public behavior, your tests should still pass. This keeps tests useful through changes rather than turning them into brittle maintenance burdens.
601|
602|🧪 **Try It Yourself**
603|
604|Pick any form-mapping function from `searchForm.ts`. Write one test that verifies it maps a specific `FormState` shape to the expected URL parameters. Run it with `npm test searchForm.test.ts`. Fix any assertion mismatches. Then add a second test for the reverse direction: convert the URL back and verify you get the original state. This one exercise covers the entire round-trip testing pattern used across the archive suite.
605|
606|---
607|
608|*End of Part 6.*
609|
610|You now know the conditional rendering pattern, the CSS variable architecture, the Svelte 5 runes powering every component, and the testing strategy that keeps everything reliable. These are the foundations — revisit them whenever you feel uncertain about how things fit together. Next up: deployment and performance optimization.
611|
1|# Part 7 — Building, Deploying & Reference
2|
3|👋 Welcome to the final part of this book! By now you've explored every piece of the Archive Frontend — from the two-row header down to the last tag chip in the search form. In this part, we're going to cover what happens after you write all that beautiful code: **how it gets built, how it gets served, and how it actually ends up in front of millions of readers.**
4|
5|We'll also wrap up with some big-picture thinking about *why* we made the choices we did, where the project is heading next, and give you two handy appendices — a complete file reference and an API field mapping table — that you can keep open while you work.
6|
7|Let's finish strong! 💪
8|
9|---
10|
11|## Chapter 29: Building & Deploying
12|
13|In most web projects, the frontend is a living, breathing Node.js server — it listens for connections, manages sessions, maybe even does client-side rendering with JavaScript frameworks running at runtime. FicHub's archive frontend is… different. It's **entirely static**.
14|
15|That means everything you build — every Svelte component, every TypeScript module, every CSS custom property — gets baked into plain old HTML, CSS, and JavaScript files. No server needed. Just files.
16|
17|### The Build Command
18|
19|To create these files, you run one command:
20|
21|```bash
22|npm run build
23|```
24|
25|This triggers Vite (the bundler powering your SvelteKit project) to do its magic: compile your `.svelte` files, bundle your TypeScript, optimize your CSS, generate sourcemaps, and spit out a clean `build/` directory containing everything the browser needs.
26|
27|Here's what matters most: **Vite hashes every asset filename**. That means instead of producing files like `index.css`, you get something like `index.a1b2c3d4.css`. Every build produces unique hash values, so when new code goes live, the filenames change too. This isn't just cosmetic — it's a clever caching strategy we'll explore later.
28|
29|### Where the Files Go
30|
31|On FicHub's ThinkCentre machine, the build output lands in:
32|
33|```
34|/personal/documents/code/rust/fichub/frontend/build/
35|```
36|
37|This path sits on an **NFS mount** (a network filesystem shared across machines), which means the Rust backend — running on the same ThinkCentre — can read these files without any special transfer step. They're already there.
38|
39|💡 **Key Concept: NFS Mounts**
40|Think of an NFS mount like a shared Google Drive folder for your servers. Both the frontend build process and the Rust backend "see" the same directory, even if they're technically separate programs. No copying files back and forth — just drop them in and go.
41|
42|### How the Rust Backend Serves Static Files
43|
44|The backend is a Rust service built with Axum. It has a single job in the static-file world: read files from disk and hand them to the browser. This logic lives in `src/static.rs`:
45|
46|```rust
47|use axum::routing::serve_static;
48|// ...
49|let state = AppState {
50|    frontend_dir: env::var("FRONTEND_DIR").expect("FRONTEND_DIR not set"),
51|    // ...other fields...
52|};
53|```
54|
55|The `FRONTEND_DIR` environment variable (set in `.env`) points to wherever your build output lives. When a user navigates to `/search` or `/bookmarks`, the backend finds the matching file in that directory and streams it over HTTP. That's it.
56|
57|There's no Node.js, no Express, no webpack dev server doing anything. Just Rust reading HTML files off disk. Simple, fast, reliable.
58|
59|### Deployment Steps
60|
61|Deploying a frontend change to FicHub looks something like this:
62|
63|1. **Pull the latest code:**
64|   ```bash
65|   git pull origin main
66|   ```
67|
68|2. **Rebuild:**
69|   ```bash
70|   npm run build
71|   ```
72|
73|3. **Restart the backend (usually not needed):**
74|   For purely frontend changes, the Rust service doesn't need restarting — it reads fresh files from the `build/` directory on every request. You'd only restart `fichub.service` if you changed backend code or environment variables:
75|   ```bash
76|   sudo systemctl restart fichub.service
77|   ```
78|
79|⚠️ **Watch Out — Don't Over-Restart!** Restarting `fichub.service` causes a brief moment of downtime for everyone. Only do it when necessary. Frontend-only changes are automatically picked up because the backend serves files dynamically from disk — no reload required.
80|
81|### 🔧 Try It Yourself: Build Locally
82|
83|Want to see this in action? Open a terminal inside the frontend directory and run:
84|
85|```bash
86|cd /home/alvaro/code/rust/fichub/frontend
87|npm run build
88|ls -la build/
89|```
90|
91|You should see a collection of hashed files — `index.html`, `app.[hash].js`, `index.[hash].css`, and more. These are your production-ready files, ready to be deployed to the ThinkCentre.
92|
93|### Cache Busting with Hashes
94|
95|Remember that Vite hash thing I mentioned earlier (`index.a1b2c3d4.css`)? That's your **cache-busting superpower**.
96|
97|Here's the problem it solves: imagine you fix a bug in the archive footer and push a new build. Your users visit `/` and their browser loads the new `index.html`, which references `index.f5e6d7c8.css`. Their browser thinks: "I already have a CSS file called index.css!" — but wait, the *filename is different*. Since it's a brand-new URL that never existed before, the browser fetches it from scratch. Old cached versions? Physically gone from the server. The old URL returns a 404. Clean break.
98|
99|Meanwhile, unchanged assets like vendor bundles stay hashed the same way across builds (because their content didn't change), so the browser happily reuses its cache for those. Smart, right?
100|
101|To make this work extra reliably, every file is served with:
102|
103|```
104|Cache-Control: max-age=31536000
105|```
106|
107|That's **one year** in seconds. The browser says "if this filename exists, it will always exist and always look exactly the same." Which, thanks to hashing, is true!
108|
109|💡 **Key Concept: Immutable Caching**
110|Hashed filenames + long TTLs = blazing-fast repeat visits. Once the browser caches your files, it never even asks the server again unless the URL changes. This is why modern CDNs serve static assets so efficiently.
111|
112|### Common Pitfall: Stale Browser Cache
113|
114|Sometimes, even with perfect deployment, a user's browser still shows old content. Why? Because browsers aggressively cache things. If you've pushed a new build but a user sees the old version, tell them to do a **hard refresh**:
115|
116|- **Chrome/Firefox:** `Ctrl+Shift+R` (or `Cmd+Shift+R` on Mac)
117|- **Safari:** `Cmd+Option+E` then reload
118|- Or hold `Ctrl` while clicking the reload button
119|
120|If that doesn't work, clearing the cache entirely usually fixes it. This isn't a bug in FicHub — it's just how the web works. Even giants like Google battle this exact issue daily.
121|
122|### Build Warnings: Ignore the Noise
123|
124|When you run `npm run build`, you might see some warnings pop up:
125|
126|- **Unused CSS selectors** — Pre-existing styles that aren't referenced by any component. Harmless, cosmetic-only.
127|- **Svelte `state_referenced_locally`** — Svelte's suggestion that certain `$state()` variables could potentially be removed. Also non-fatal.
128|
129|These warnings are tracked as low-priority housekeeping items. They won't break anything. Fixing them requires careful review to make sure removing an "unused" selector doesn't silently break styling. Don't let them distract you from writing new features!
130|
131|---
132|
133|## Chapter 30: Architecture Decisions & Future Work
134|
135|Congratulations on making it this far through 30 chapters! Before we close the book, let's take a breather and think about the *big picture*. Why did we build the archive interface the way we did? What problems were we trying to solve? And where are we headed next?
136|
137|### Why Path B: Two Shells in One App?
138|
139|When the archive redesign started, the team considered three approaches:
140|
141|| Approach | Description | Trade-off |
142||----------|-------------|-----------|
143|| **Path A: Separate Apps** | Ship the archive UI as an entirely different application | Doubles your deploy pipeline, doubles your database connections, doubles your maintenance |
144|| **Path B: Layout Switch** ✅ | ONE app that swaps between layouts | One codebase, one deploy, one pool — two experiences |
145|| **Route Split** | Put archive routes under `/archive/*` and modern routes under `/` | Fragmented sharing of components, confusing URL patterns |
146|
147|We chose **Path B** — a layout-level switch inside a single application — and here's why it was the winning choice:
148|
149|1. **One codebase**: All components share utilities, API clients, auth stores, and type definitions. No sync headaches.
150|2. **One deploy pipeline**: One `git push` deploys both interfaces. No coordinating two separate CI/CD flows.
151|3. **One database connection pool**: The Rust backend talks to Postgres once. Both UI modes drink from the same well.
152|4. **Two complete experiences**: Users don't feel like they're using a "lite" archive mode. Every feature available in modern mode is equally available in archive mode.
153|
154|It's like having two bedrooms in one house, connected by the same hallway. You pick whichever room you want to hang out in tonight, but the kitchen and bathroom serve both.
155|
156|### Why Is Archive the Default?
157|
158|Modern mode got built first — it was our experimental playground. Yet when it came time to ship, **archive became the default** for every user. Here's why:
159|
160|**Familiarity wins.** Most fanfiction readers learned the web through AO3 (Archive of Our Own). Its layout, its color scheme, its way of organizing tags — that's muscle memory. A reader visiting FicHub shouldn't have to learn a new interface before they can enjoy the content. By making archive the default, we reduce friction and meet readers where they are.
161|
162|The modern shell stays fully accessible though — it's one click away in the settings page, footer links, or dropdown menu. But the homepage greets you with the familiar, comfortable archive design.
163|
164|### The One Divergence from AO3: AND vs OR
165|
166|Almost everything in the archive UI mimics AO3 exactly — until you try filtering by fandom.
167|
168|At AO3, selecting multiple fandoms uses **OR logic**: show me stories tagged with "Marvel Cinematic Universe" OR "Star Wars." Pick either one, you'll get results.
169|
170|At FicHub, selecting multiple fandoms uses **AND logic**: show me stories tagged with "Marvel Cinematic Universe" AND "Star Wars." Works must have *every single tag you select*.
171|
172|This is a known difference. It's documented. It makes the filter tighter and more specific — which some users love and others find frustrating when they expected the AO3-style broadening behavior. We've marked it clearly in the UI tooltips so nobody is blindsided.
173|
174|⚠️ **Watch Out for the AND Behavior:** If you're searching for stories in multiple fandoms and getting zero results, check whether AND logic might be the culprit. Try selecting fewer tags at once.
175|
176|### The Parachute Honesty Principle: Inert Fields
177|
178|Some fields on the archive search form are **inert** — they're displayed but don't actually filter results. The crossovers checkbox, the hits counter, the language dropdown, the rating badges. They look interactive, but clicking or changing them does nothing.
179|
180|Why show them at all? We call it the **parachute honesty principle**.
181|
182|When users land on the search page, they scan it quickly. If they don't see "Crossovers" or "Hits" or "Language," they assume these filters don't exist at FicHub — maybe the backend can't handle them, maybe the site is incomplete. By showing disabled options with explanatory tooltips, we communicate: *"Yes, we know you want to filter by language. Yes, we want that too. We're building it — come back soon!"*
183|
184|Users see what exists, understand the backend's limits transparently, and trust us that the missing pieces are intentional, not accidental.
185|
186|💡 **Key Concept: Transparency Builds Trust**
187|Telling users what you *can't* do yet is often better than pretending it doesn't exist. People forgive unfinished features; they punish hidden ones.
188|
189|### The Secret Superpower: `?ui=` URL Parameters
190|
191|Try visiting this URL in your browser:
192|
193|```
194|https://fichub.example.com/search?q=harry?ui=archive
195|```
196|
197|Notice the `?ui=archive` at the end? That tiny parameter overrides anyone's personal preference and forces the archive view. Without touching any settings, anyone can render *any* page in archive mode.
198|
199|This is incredibly useful for:
200|
201|- **Documentation**: Link to a page in the format you're describing
202|- **Tutorials**: Show screenshots that match the current reader's expectations
203|- **A/B testing**: Compare how features behave in each mode side by side
204|- **User switching**: Click the link, try the other UI, go back — no settings change required
205|
206|It's a small addition with outsized usefulness. Under the hood, `applyUiParam()` checks for this query parameter early in the page lifecycle and short-circuits the normal preference lookup.
207|
208|### Roadmap: What's Next?
209|
210|The archive frontend is feature-complete, but that doesn't mean the work stops here. Here's what's on the horizon:
211|
212|1. **Restyle the Reader**: Right now, the fic reading page (where you actually read stories) looks completely different from the archive UI. A planned restyle would bring the reader page into the archive aesthetic — maroon headings, Georgia serif fonts, compact spacing. Readers scroll through dozens of pages per story, so consistency matters.
213|
214|2. **Admin Pages**: The admin dashboard has a modern-shell treatment that doesn't quite match the archive elsewhere. Bringing admin pages into archive mode would unify the experience for moderators.
215|
216|3. **More Archive Conditional Coverage**: Several pages still lack archive implementations. Any route page with `{#if uiMode === 'archive'}` still stubbed out as TODO is fair game.
217|
218|4. **Potential Third Theme: "Retro"**: Some users have suggested a pre-AO3 "dark web" aesthetic — deep blacks, neon accents, table-based layouts. It'd be a fun nostalgic trip and a great showcase exercise. Whether it ships depends on community interest.
219|
220|Performance-wise, the system is solid. Being prerendered client-side SvelteKit means **zero framework overhead** — no hydration waterfall, no React reconciliation, no Angular zones. Archive components are lightweight (~17K total lines of code across all files), and skeleton loading prevents FOUC (Flash of Unstyled Content) during data fetching. The result: fast loads, smooth interactions, happy readers.
221|
222|💡 **Key Concept: Zero Framework Overhead**
223|Because pages are prerendered at build time, the browser receives fully-formed HTML. There's no JavaScript framework bootstrapping itself on the client side. What you see is what you get — no waiting, no flickering, no waterfall. The browser renders instantly.
224|
225|---
226|
227|## Appendix A: File Reference
228|
229|Every file in the archive frontend, listed with its line count and purpose. Use this as a quick lookup when you're trying to figure out "which file handles X?"
230|
231|### Directory Structure
232|
233|All archive code lives under:
234|
235|```
236|frontend/src/lib/ui/archive/
237|```
238|
239|### Complete File List
240|
241|| File | Lines | Purpose |
242||------|-------|---------|
243|| `ArchiveLayout.svelte` | 63 | Root wrapper — wraps header, main, footer. Re-hosts CommandPalette, HelpModal. Initializes theme/i18n/auth on mount. Uses Svelte 5 `{@render children()}` slot pattern. |
244|| `ArchiveHeader.svelte` | ~250 | Two-row AO3-parity header. Top row: logo + avatar + Post/Log Out. Red navbar: Fandoms, Browse, Search ▾, About ▾ (native `<details>` dropdowns). Keyboard-accessible. No emojis anywhere. |
245|| `ArchiveFooter.svelte` | ~130 | Four-column grid footer (Customize, About, Contact, Development). Accent-line background, white text. Responsive collapses to 2 cols at 600px, 1 col at 380px. Contains "Switch to modern interface" button with `setPref` + `location.reload()` pattern. |
246|| `ArchiveNavLink.svelte` | 49 | Nav link with active-state detection via `$page.url`. Uses `$derived` for reactive computation. 2px accent underline on active state. |
247|| `ArchiveButton.svelte` | 58 | Flat button component. Variants: primary (maroon) vs secondary (gray). AO3 aesthetic: no shadows, no gradients, square corners. |
248|| `ArchiveWorkSearchForm.svelte` | 853 | **LARGEST COMPONENT**. Two-mode search form with autocomplete chips, tag suggestions, sort controls, and pagination. Orchestrates searchForm.ts functions. |
249|| `ArchiveListPage.svelte` | 216 | Reusable list wrapper. Props: title, items, loading, error, subtitle. Includes loading skeleton animation (5 pulsing blurbs) and empty/error states. Used by bookmarks, authors, notifications pages. |
250|| `ArchiveHome.svelte` | 470 | Homepage component. Featured sections, trending works, category browsing. Uses `ArchiveListPage` internally. |
251|| `WorkBlurb.svelte` | 208 | Core fic card component. Renders: rating badge, title (maroon link), byline, TagSoup, snippet, StatsLine, action buttons (Download, Read, Bookmark, Kudos). Anonymous users see login prompt. |
252|| `TagSoup.svelte` | 117 | Labeled tag rows. Bold gray label + comma-separated maroon tag links. Tags truncate at 6 for Additional Tags category with "+N more" details disclosure. |
253|| `StatsLine.svelte` | 52 | Single-line stats display. Format: "Words: 1,204,116 · Chapters: 109/109 · Kudos: 412 · Updated: 3d ago". Muted color, middot separators. Simplest content component. |
254|| `rating.ts` | 118 | Pure TypeScript helpers. Rating resolution, display labels, normalization. Zero Svelte dependency — importable anywhere. |
255|| `searchForm.ts` | 480 | **SECOND LARGEST FILE**. Pure module. Functions: `parseRange()`, `buildSearchQuery()`, `formStateToUrl()`, `urlToFormState()`, `mergeChipIntoForm()`, `filterTagsForSuggestions()`. 47 unit tests in companion test file. |
256|| `searchForm.test.ts` | — | Unit tests covering searchForm.ts pure functions. Tests form-state round-trips, URL serialization, chip merging, range parsing. |
257|
258|### Import Dependency Graph
259|
260|```
261|Root Layout (+layout.svelte)
262|    │
263|    └─→ ArchiveLayout.svelte
264|           │
265|           ├─→ ArchiveHeader.svelte
266|           │      └─→ ArchiveNavLink.svelte
267|           │
268|           ├─→ ArchiveFooter.svelte
269|           │      └─→ ArchiveNavLink.svelte
270|           │
271|           ├─→ Content (child routes)
272|           │      │
273|           │      ├─→ Search Page → ArchiveWorkSearchForm → searchForm.ts → Search API
274|           │      │
275|           │      ├─→ ArchiveListPage.svelte → WorkBlurb → TagSoup + StatsLine
276|           │      │                                   ↓
277|           │      │                             WorkBlurb → rating.ts
278|           │      │
279|           │      └─→ ArchiveHome.svelte → ArchiveListPage → WorkBlurb
280|           │
281|           └─→ prefs.ts (getPref called throughout)
282|                  ↑
283|              All routes → getPref(prefs.ts) → uiMode check
284|```
285|
286|💡 **Quick Tip**: When you need to understand a file's role, check how many other files import it. `prefs.ts`, `searchForm.ts`, and `ArchiveLayout.svelte` sit at the top of the graph — almost everything depends on them. The leaf nodes (`TagSoup`, `StatsLine`, `ArchiveNavLink`) are self-contained components with zero imports.
287|
288|---
289|
290|## Appendix B: API Field Mapping Table
291|
292|This table maps every form field on the archive search page to its corresponding backend API parameter. Use this when building new filters, debugging unexpected results, or implementing inert-to-active transitions.
293|
294|| Form Field | API Param | Status | Notes |
295||------------|-----------|--------|-------|
296|| `title` | `title` | ✅ Supported | Exact-match title filter |
297|| `author` | `author` | ✅ Supported | Filters by author name |
298|| `chapters_num` | `chapters` | ✅ Supported | Number range (min/max) |
299|| `word_count` | `min_word_count` / `max_word_count` | ✅ Supported | Parsed via `parseRange()` syntax |
300|| `fandom` | `include_tags=fandom:name` | ✅ Supported | Added to `include_tags` array |
301|| `character` | `include_tags=character:name` | ✅ Supported | Same tag-pattern as fandom |
302|| `relationship` | `include_tags=relationship:name` | ✅ Supported | Same tag-pattern as fandom |
303|| `additional_tag` | `include_tags=additional_tag:name` | ✅ Supported | Same tag-pattern as fandom |
304|| `warning` | `include_tags=warning:name` | ✅ Supported | Same tag-pattern as fandom |
305|| `category` | `include_tags=category:name` | ✅ Supported | Same tag-pattern as fandom |
306|| `kudos` | `max_kudos` | ⚠️ Partial | Upper bound only — no minimum kudos support |
307|| `comments` | `min_comments` | ✅ Supported | Minimum comment count |
308|| `bookmarks` | `min_bookmarks` | ✅ Supported | Minimum bookmark count |
309|| `rating` | `include_tags=rating:*` | 🪂 Inert | Best-effort resolved internally; no-op at API level |
310|| `crossovers` | N/A | 🪂 Inert | Shown but ignored by backend |
311|| `hits` | N/A | 🪂 Inert | Display placeholder only |
312|| `language` | N/A | 🪂 Inert | Placeholder for future implementation |
313|| `sort_value` | `sort_column` / `sort_descending` | ✅ Supported | Combined into single sort param |
314|
315|### Legend
316|
317|- ✅ **Supported** — fully functional, actively filters results
318|- ⚠️ **Partial** — implemented but with limitations (check notes column)
319|- 🪂 **Inert** — visible in UI but has no effect; shown for transparency (parachute honesty)
320|
321|---
322|
323|*You did it.* Thirty chapters of svelte runes, CSS custom properties, search forms, tag chips, layout switches, and static file serving. If you can reason about how all of this fits together — how a user types "dragon" into a search box and watches results populate across a fully styled archive interface — you've earned your place in the FicHub codebase.
324|
325|Keep building, keep learning, and remember: the best code is the kind someone else can pick up without calling you at 2 AM. Happy coding! ✨
326|

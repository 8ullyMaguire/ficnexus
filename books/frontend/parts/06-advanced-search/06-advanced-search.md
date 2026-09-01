# Part 6: Advanced Search and Navigation

*Where we build the app shell, the search page, the filter system, and teach users a query language that makes them feel like power users.*

---

# Chapter 25: The Layout and Navigation

## The App Shell

Every great house has a foundation. Every great app has a layout.

In SvelteKit, the layout is defined by a special file: `+layout.svelte`. This file wraps *every* page in your application. It's the frame around the picture, the scaffolding around the building, the shell around the pearl. Whatever page the user navigates to, the layout stays constant.

Think about the apps you use every day. Gmail always has that sidebar with your inbox, sent, drafts. Twitter always has the top navigation bar. YouTube always has the search bar at the top. These persistent elements are layouts — they provide context and navigation no matter where you are in the app.

FicHub follows the same pattern. The layout provides:

1. **A sticky topbar** — always visible at the top of the screen, no matter how far you scroll. It contains the brand, tabs, and search field.
2. **Tab-based content switching** — the main area of the page renders different components depending on which tab is active: Download, Recommendations, or Suggestions.
3. **A footer** — the quiet little line at the bottom listing supported sites.

This separation is important. The layout is the **container** — it knows *where* things go. The pages and components are the **content** — they know *what* goes there. This separation makes the codebase easier to understand, easier to modify, and less likely to have bugs.

Let's look at the actual code. This is the entire `+layout.svelte` file:

```svelte
<script lang="ts">
  import '../app.css';
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import RecommendationsTab from '$lib/components/RecommendationsTab.svelte';
  import SuggestionsTab from '$lib/components/SuggestionsTab.svelte';
  import { goto } from '$app/navigation';

  let { children } = $props();

  type Tab = 'download' | 'recs' | 'sugg';
  let activeTab = $state<Tab>('download');

  const tabs: { id: Tab; label: string; icon: string }[] = [
    { id: 'download', label: 'Download', icon: '⬇' },
    { id: 'recs', label: 'Recommendations', icon: '★' },
    { id: 'sugg', label: 'Suggestions', icon: '💡' },
  ];

  let searchQuery = $state('');

  function onSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && searchQuery.trim()) {
      goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  }
</script>
```

That's the entire script section. Notice how clean it is — just imports, a type definition, state variables, and one function. It's about 25 lines of actual logic for the entire app shell. Let's break down each piece.

### Imports

```svelte
import '../app.css';
```

The first line imports the global stylesheet. `app.css` defines CSS custom properties like `--color-primary`, `--color-surface`, `--color-border`, and `--radius-sm`. Every component in the app can use these variables, so the whole app shares a consistent look. If you want to change the primary color from blue to purple, you change it in one place and the entire app updates.

The next three imports bring in the three tab components. Each is a self-contained Svelte file in `$lib/components/`. The layout doesn't need to know what's inside them — it just renders the right one based on the active tab. This is a key principle: **the layout is a container, not a content creator**. It knows *which* component to show, but not *what* that component does.

```svelte
import { goto } from '$app/navigation';
```

`goto` is SvelteKit's client-side navigation function. When we call `goto('/search?q=hello')`, the browser doesn't do a full page reload — SvelteKit just swaps the page component and updates the URL. It's fast, smooth, and it keeps the app feeling like a single-page app even though it's technically using URLs.

Behind the scenes, `goto` calls `history.pushState` to update the URL, then triggers SvelteKit's internal routing to unmount the current page and mount the new one. No network request, no white flash, no lost scroll position. It's the magic that makes SvelteKit SPAs feel so snappy.

### Props and Children

```svelte
let { children } = $props();
```

In Svelte 5, `$props()` replaces the old `export let` pattern. The layout receives `children` as a prop — this is the content of whichever page is currently active. If you're on the home page, `children` is the `+page.svelte` from the root route. If you're on `/search`, `children` is the search page. The layout decides *where* to put the children (in the `<main>` element), but the children decide *what* gets rendered there.

Think of it like a theater stage. The layout is the stage itself — it has lights, curtains, and a backdrop. The `children` prop is the play being performed. Different plays (pages) can use the same stage (layout), but the stage doesn't care which play is running.

> **Try It Yourself:** Think of `children` as a slot. The layout says "I have a space for content right here," and each page fills that space with its own markup. This is the fundamental layout pattern — the frame stays the same, the picture changes.

### The Tab Type

```svelte
type Tab = 'download' | 'recs' | 'sugg';
let activeTab = $state<Tab>('download');
```

We define a union type `Tab` with three possible values. The `activeTab` variable holds which tab is currently selected. By default, it's `'download'` — the first thing users see is the download form.

Using `$state<Tab>` means Svelte 5's reactivity system tracks this variable. When `activeTab` changes, Svelte automatically re-renders any `{#if}` blocks that depend on it. The `<Tab>` type parameter is just TypeScript — it tells the compiler that `activeTab` can only be one of three strings. If you accidentally wrote `activeTab = 'banana'`, TypeScript would catch the error before the app even runs.

### The Tabs Array

```svelte
const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: 'download', label: 'Download', icon: '⬇' },
  { id: 'recs', label: 'Recommendations', icon: '★' },
  { id: 'sugg', label: 'Suggestions', icon: '💡' },
];
```

Instead of writing three separate buttons, we define the tabs as data. Each tab has an `id` (matching the `Tab` type), a `label` (the text shown to the user), and an `icon` (the emoji). This makes it trivial to add a new tab later — just add an entry to the array. No need to copy-paste button markup, remember to add the right click handler, or worry about matching class names.

This is the "data-driven UI" pattern: instead of writing UI markup for each item, you describe the items as data and let a loop generate the UI. It's DRY (Don't Repeat Yourself), maintainable, and it scales beautifully.

If we wanted to add a fourth tab — say, "History" — we'd just add `{ id: 'history', label: 'History', icon: '📜' }` to the array and add a new `{:else if}` branch in the template. Two changes, no copy-paste.

### The Search Function

```svelte
let searchQuery = $state('');

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && searchQuery.trim()) {
    goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
  }
}
```

The navbar has a search field. When the user types a query and presses Enter, we navigate to `/search?q=...` using SvelteKit's `goto`. The `encodeURIComponent` makes sure special characters in the query don't break the URL — for example, a query like `Harry & Hermione` becomes `Harry%20%26%20Hermione`.

Notice we use `trim()` to remove leading/trailing whitespace, and we check `e.key === 'Enter'` to only trigger on Enter — not on every keystroke. We also check `searchQuery.trim()` to avoid navigating with an empty search. These small guards prevent confusing behavior.

> **Watch Out:** The `searchQuery` is local state — it's not shared with the search page. When the user navigates to `/search`, the search page reads its own `queryInput` from URL params. This means the navbar search field clears after navigation, which is fine — the user can always type a new query there. If you wanted the search field to persist across navigation, you'd need to lift state to a store or use SvelteKit's `page` state.

## The Template: Building the Topbar

Now let's look at the HTML structure:

```svelte
<div class="app">
  <header class="topbar">
    <div class="brand">
      <span class="logo">📚</span>
      <span class="title">FicHub</span>
    </div>
    <nav class="tab-bar" role="tablist">
      {#each tabs as t}
        <button
          class="tab"
          class:active={activeTab === t.id}
          onclick={() => (activeTab = t.id)}
          role="tab"
          aria-selected={activeTab === t.id}
        >
          <span class="tab-icon">{t.icon}</span>
          <span class="tab-label">{t.label}</span>
        </button>
      {/each}
    </nav>
    <div class="search-area">
      <input
        class="nav-search"
        type="search"
        placeholder="Search…"
        bind:value={searchQuery}
        onkeydown={onSearchKeydown}
        aria-label="Search fanfiction"
      />
      <a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>
        Advanced
      </a>
    </div>
  </header>
```

Let's walk through this piece by piece.

### The Brand

```svelte
<div class="brand">
  <span class="logo">📚</span>
  <span class="title">FicHub</span>
</div>
```

The leftmost element in the topbar. The 📚 emoji serves as the logo, and "FicHub" is the title. Using an emoji instead of an image file means we don't need to worry about image loading, alt text, vector formats, or retina displays. It's universally rendered, always crisp, and it gives the app a friendly, approachable feel.

### The Tab Bar

```svelte
<nav class="tab-bar" role="tablist">
  {#each tabs as t}
    <button
      class="tab"
      class:active={activeTab === t.id}
      onclick={() => (activeTab = t.id)}
      role="tab"
      aria-selected={activeTab === t.id}
    >
      <span class="tab-icon">{t.icon}</span>
      <span class="tab-label">{t.label}</span>
    </button>
  {/each}
</nav>
```

The `{#each tabs as t}` loop renders one button per tab. Each button has several important attributes:

- **`class="tab"`** — the base CSS class for all tab buttons.
- **`class:active={activeTab === t.id}`** — Svelte's conditional class syntax. When `activeTab` matches this tab's ID, the `active` class is added. This is the mechanism that highlights the currently selected tab.
- **`onclick={() => (activeTab = t.id)}`** — clicking the button sets the active tab. The arrow function creates a closure that captures the current tab's `id`.
- **`role="tab"` and `aria-selected`** — accessibility attributes so screen readers understand these are tabs and which one is selected. This is not just nice to have — it's essential for users who rely on assistive technology.

Inside each button, we render both the icon and the label. The icon is always visible. The label gets hidden on mobile via CSS (we'll see that later), showing just the icons on small screens.

> **Watch Out:** The `class:active` syntax is Svelte-specific. It's syntactic sugar for `class:active={expression}` — when the expression is truthy, the class is added. You could also write `class={activeTab === t.id ? 'tab active' : 'tab'}`, but the Svelte way is cleaner and less error-prone. It also composes well — you can have multiple `class:name` directives on the same element.

### The Search Area

```svelte
<div class="search-area">
  <input
    class="nav-search"
    type="search"
    placeholder="Search…"
    bind:value={searchQuery}
    onkeydown={onSearchKeydown}
    aria-label="Search fanfiction"
  />
  <a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>
    Advanced
  </a>
</div>
```

The search area sits on the right side of the topbar (thanks to `margin-left: auto` in the CSS). It has a text input and a small "Advanced" link.

The `bind:value={searchQuery}` creates two-way binding — when the user types, `searchQuery` updates. When `searchQuery` changes programmatically, the input updates. This is one of Svelte's signature features: two-way binding without the boilerplate of event handlers and manual state updates.

The `type="search"` attribute gives us semantic HTML and a native clear button in some browsers. The `placeholder="Search…"` shows hint text that disappears when the user starts typing.

The "Advanced" link navigates to `/search`. The `onclick` handler also sets `activeTab = 'download'` so that when the user returns from the search page, the download tab is active. This prevents a confusing state where the user returns to a tab they didn't expect.

### The Main Content

```svelte
  <main class="container">
    {#if activeTab === 'download'}
      <DownloadTab />
    {:else if activeTab === 'recs'}
      <RecommendationsTab />
    {:else if activeTab === 'sugg'}
      <SuggestionsTab />
    {/if}
  </main>
```

This is where the magic happens. The `{#if}/{:else if}` chain renders exactly one component based on `activeTab`. When the user clicks a different tab, `activeTab` changes, Svelte removes the old component and mounts the new one.

One important detail: each tab component is completely independent. `DownloadTab` doesn't know about `RecommendationsTab`, and neither knows about `SuggestionsTab`. They share the layout's topbar and footer, but they don't share any state with each other. This isolation makes each component easier to understand, test, and modify.

Another detail: the components are conditionally rendered, not hidden with CSS. When `activeTab` is `'download'`, the `RecommendationsTab` and `SuggestionsTab` components don't exist in the DOM at all. They're not just hidden — they're unmounted. This means they don't consume memory or run any JavaScript. When the user switches tabs, the old component is destroyed and the new one is created from scratch.

This is different from using `display: none` to hide tabs. With conditional rendering, switching tabs resets the component's internal state. If the user was scrolling through recommendations and switches to downloads, the recommendations scroll position is lost. For FicHub, this is fine — each tab is a fresh start. But if you needed to preserve scroll position across tab switches, you'd want to use CSS visibility instead.

Note that the root `+page.svelte` is intentionally empty — it contains only a comment explaining that the tabbed UI lives in the layout. This means the root page doesn't compete with the layout; the layout handles everything. The root page is just a placeholder that SvelteKit requires for the `/` route.

An alternative approach would be to use SvelteKit's file-based routing with separate pages for each tab (e.g., `/download`, `/recs`, `/sugg`). But for FicHub, tab-based UI within a single layout is simpler and faster — no page transitions, no URL changes, no loading states. The tabs just swap content instantly.

### The Footer

```svelte
  <footer class="footer muted">
    <span>FicHub — download & discover fanfiction.</span>
    <span class="sites">AO3 · FanFiction.net · FictionPress · Forums</span>
  </footer>
</div>
```

The footer is simple: a tagline on the left and a list of supported sites on the right. The `muted` class uses the `--color-muted` CSS variable for a lighter text color. The footer uses `justify-content: space-between` to push the two spans to opposite ends.

The footer is wrapped inside the `.app` container, which is a flex column. This is important for the layout's height behavior — the `.app` container stretches to at least the full viewport height, and the `main` element grows to fill the remaining space. This means the footer always sticks to the bottom, even when there's little content. No awkward gaps, no footer floating in the middle of the page.

## The CSS: Making It Beautiful

Now let's look at the styles that make this layout look polished:

```css
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
```

The app container is a flex column that takes up at least the full viewport height. `min-height: 100vh` ensures it's at least as tall as the viewport. `flex-direction: column` stacks the topbar, main content, and footer vertically. Together with `flex: 1` on the `main` element (which we'll see in a moment), this creates a sticky footer layout — the footer always stays at the bottom.

### The Topbar

```css
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  padding: 0.6rem 1rem;
  display: flex;
  align-items: center;
  gap: 1.2rem;
  flex-wrap: wrap;
}
```

`position: sticky` is the key. The topbar sticks to the top of the viewport when you scroll. `top: 0` means it sticks at the very top. `z-index: 10` ensures it stays above other content — without this, search results or cards might scroll over the topbar.

`background: var(--color-surface)` gives it a solid background so content scrolls *behind* it, not *through* it. `border-bottom` provides a subtle separator between the topbar and the content below.

`display: flex` with `align-items: center` creates a horizontal row with everything vertically centered. `gap: 1.2rem` adds consistent spacing between the brand, tabs, and search area. `flex-wrap: wrap` lets the topbar wrap onto multiple lines on very narrow screens — the tabs might wrap below the brand, and the search area might wrap below the tabs.

### Tab Styling

```css
.tab-bar {
  display: flex;
  gap: 0.3rem;
}
.tab {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: transparent;
  border: 1px solid transparent;
  color: var(--color-muted);
  padding: 0.45rem 0.9rem;
  border-radius: var(--radius-sm);
  font-weight: 600;
  font-size: 0.92rem;
  transition: all 0.15s;
}
.tab:hover {
  color: var(--color-text);
  background: var(--color-surface-2);
}
.tab.active {
  color: white;
  background: var(--color-primary);
}
```

The tab bar is a flex row with small gaps between tabs. Each inactive tab is transparent with muted text — it blends into the topbar. On hover, the tab gets a subtle background color and darker text, giving visual feedback.

The active tab gets a bold primary-color background with white text. This high contrast makes the selected tab immediately obvious. The `transition: all 0.15s` makes the hover and active transitions smooth — no jarring snap.

The `border: 1px solid transparent` on inactive tabs is a subtle trick: it ensures the tab doesn't "jump" when it gains a border on active state. The border is always there; it's just transparent when inactive. This maintains consistent sizing across all tabs.

### Search Area Positioning

```css
.search-area {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.nav-search {
  width: 180px;
  padding: 0.4rem 0.7rem;
  font-size: 0.88rem;
  border-radius: var(--radius-sm);
}
.adv-link {
  color: var(--color-muted);
  font-size: 0.82rem;
  white-space: nowrap;
}
.adv-link:hover {
  color: var(--color-text);
  text-decoration: none;
}
```

`margin-left: auto` is a classic CSS flexbox trick — in a flex row, applying `margin-left: auto` to the last item pushes it to the far right. This is how we get the search area aligned to the right without any absolute positioning or floats. It's elegant and responsive — the search area naturally stays on the right regardless of how wide the screen is.

The search input is 180 pixels wide — wide enough for typical queries but not so wide it dominates the topbar. The "Advanced" link is small and muted, sitting quietly next to the search field. It only highlights on hover, keeping the visual noise low.

### Responsive Design

```css
@media (max-width: 700px) {
  .tab-label {
    display: none;
  }
  .search-area {
    width: 100%;
    margin-left: 0;
  }
  .nav-search {
    flex: 1;
  }
}
```

On screens narrower than 700 pixels, we make two key changes:

1. **Hide tab labels** — only the emoji icons remain visible. This saves horizontal space on mobile. The icons are universally recognizable — ⬇ for download, ★ for recommendations, 💡 for suggestions — so the labels aren't essential.

2. **Full-width search area** — the search field takes the entire remaining width. `flex: 1` on the input makes it grow to fill the container. `margin-left: 0` overrides the desktop `margin-left: auto` so the search area doesn't try to push to the right of a non-existent space.

This means on a phone, the topbar shows: `📚 FicHub ⬇ ★ 💡 [Search field stretching to the right edge]`. Clean, compact, and functional.

> **Try It Yourself:** Open FicHub in your browser and shrink the window below 700px wide. Watch the tab labels disappear and the search field expand. Resize it back — the labels reappear. This is responsive design in action, and it's just 8 lines of CSS.

### The Footer CSS

```css
.footer {
  border-top: 1px solid var(--color-border);
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0.5rem;
  font-size: 0.82rem;
}
```

The footer uses `justify-content: space-between` to push the tagline to the left and the sites list to the right. On very narrow screens, `flex-wrap: wrap` allows them to stack vertically. The small font size keeps the footer unobtrusive — it's there if you need it, but it doesn't compete with the main content.

### The Main Element

```css
main {
  flex: 1;
}
```

This single line is crucial. `flex: 1` tells the main element to grow and fill all available vertical space. Combined with the `.app` container's `min-height: 100vh`, this pushes the footer to the bottom of the viewport (or the bottom of the content, whichever is lower).

Without `flex: 1`, the main element would only be as tall as its content. If the content is short (like an empty download form), the footer would float up to the middle of the page. With `flex: 1`, the footer always stays at the bottom.

## Accessibility

A good layout isn't just about looks — it's about usability for everyone. The layout includes several accessibility features:

- **`role="tablist"`** on the tab bar tells screen readers "this is a group of tabs."
- **`role="tab"`** on each button tells screen readers "this is a tab."
- **`aria-selected`** tells screen readers which tab is currently active.
- **`aria-label="Search fanfiction"`** on the search input gives screen readers a description of what the input is for.
- **Semantic HTML** — `<header>`, `<nav>`, `<main>`, `<footer>` — gives the page structure that screen readers can navigate.

These attributes don't change the visual appearance at all. They're invisible to sighted users but essential for users who rely on screen readers. Good accessibility is good engineering.

### Keyboard Navigation

The layout also supports keyboard navigation. Users can:

1. **Tab through the topbar** — pressing Tab moves focus from the brand to the first tab, then to each subsequent tab, then to the search input.
2. **Activate tabs with Enter or Space** — once a tab has focus, pressing Enter or Space selects it.
3. **Search with Enter** — typing in the search field and pressing Enter navigates to the search page.

The `<button>` elements for tabs are natively focusable and activatable with keyboard. The `<a>` element for the Advanced link is also natively focusable. This means the entire topbar is usable without a mouse.

### Focus Management

SvelteKit handles focus management during navigation. When the user clicks a tab, focus stays on the clicked button. When the user navigates to the search page, focus moves to the search input. This prevents the confusing "focus is lost" state where the user has to click somewhere to regain keyboard control.

## Responsive Design Deep Dive

Let's take a closer look at how the responsive design works. The layout uses two breakpoints:

1. **700px** — the topbar breakpoint. Below this, tab labels hide and the search area goes full-width.
2. **600px** — the search page breakpoint. Below this, the filter grid collapses to one column and result cards stack vertically.

These breakpoints are chosen based on common device widths:

- **Phones** (320px–428px) — both breakpoints apply. Topbar shows icons only, search page shows stacked layouts.
- **Tablets** (768px–1024px) — neither breakpoint applies. Full layout with labels and two-column grids.
- **Desktops** (1200px+) — neither breakpoint applies. Full layout with generous spacing.

The breakpoints aren't arbitrary — they're based on where the layout starts to feel cramped. At 700px, the three tabs plus the search field plus the brand don't fit in one row without wrapping. At 600px, the two-column filter grid becomes too narrow for comfortable input sizing.

## The Full Layout: How It All Fits Together

Here's the complete flow when a user interacts with FicHub:

1. **The user loads the app.** SvelteKit renders `+layout.svelte`. The topbar appears with brand, tabs, and search field. The `{#if}` chain renders `DownloadTab` by default because `activeTab` starts as `'download'`.

2. **The user clicks "Recommendations."** The click handler sets `activeTab = 'recs'`. Svelte removes `DownloadTab` from the DOM and mounts `RecommendationsTab`. The URL doesn't change — this is a client-side state transition, not a navigation. The transition is instant.

3. **The user types in the navbar search.** They type `Harry Potter` and press Enter. The `onSearchKeydown` handler fires, encoding the query and calling `goto('/search?q=Harry%20Potter')`. SvelteKit's router kicks in — it unmounts the current page content and mounts the search page.

4. **The search page renders inside `<main>`.** It's the "children" of the layout. The layout's topbar and footer remain visible. The search page reads `q=Harry Potter` from the URL and auto-executes the search.

5. **The user clicks "Advanced" in the navbar.** This navigates to `/search` (no query). The search page renders with its full filter UI, ready for the user to build a complex search.

6. **The user clicks a result's "Download" button.** This navigates to `/?q=...` with the fic's URL. The layout renders the `DownloadTab` (since we set `activeTab = 'download'` in the Advanced link's onclick).

The layout is the constant. The tabs and search page are the variables. This separation of concerns is what makes SvelteKit layouts so powerful — you write the shell once, and every page benefits from it.

---

# Chapter 26: The Advanced Search Page

## The Search Route

When a user clicks "Advanced" in the navbar (or searches from the navbar and gets redirected), they land on the `/search` route. This route has two files:

- **`+page.ts`** — the page loader that reads URL parameters.
- **`+page.svelte`** — the page component with the search UI.

Together, they create a self-contained search experience. The loader handles data fetching (reading URL params), and the component handles presentation (rendering the UI and handling interactions).

### The Page Loader

```typescript
import type { PageLoad } from './$types';

// Read URL search params and pass them to the component.
export const load: PageLoad = async ({ url }) => {
  return {
    q: url.searchParams.get('q') ?? '',
    tab: url.searchParams.get('tab') ?? 'work',
  };
};
```

This is beautifully simple. The loader reads two URL parameters: `q` (the search query) and `tab` (which search tab to show). If they're not provided, it defaults to an empty string and `'work'` respectively.

The `load` function runs before the page component renders. It passes its return value as the `data` prop to `+page.svelte`. This means the component always knows what URL parameters the user arrived with.

The `PageLoad` type comes from SvelteKit's generated `$types` module. It ensures the return type matches what the component expects. If you change the return shape, TypeScript will tell you if the component needs updating.

> **Why use a loader instead of reading `window.location` directly?** Because SvelteKit's loader runs both on the server (during SSR) and on the client (during navigation). If you used `window.location`, it would break during server-side rendering. The loader pattern is the portable, SvelteKit-approved way to access URL data. Even though FicHub runs in SPA mode (`export const ssr = false`), using the loader keeps the code consistent and future-proof.

### SPA Mode

FicHub runs in SPA (Single Page Application) mode. The root `+layout.ts` file sets:

```typescript
// SPA mode: no SSR, no prerender. The backend serves the static build.
export const ssr = false;
export const prerender = false;
```

This means SvelteKit doesn't do server-side rendering — the Rust backend serves the static build, and the browser handles all routing. This simplifies deployment (no Node.js server needed) and means the `load` function always runs in the browser.

## The Page Component Structure

Now let's look at the search page itself. It's a substantial component — about 540 lines — so we'll walk through it in sections.

### Script Section: Imports and Types

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { search } from '$lib/api/search';
  import type { SearchFilters, SearchResult, SearchResponse } from '$lib/api/search';
  import {
    SORT_OPTIONS,
    COMPLETE_OPTIONS,
    SOURCE_OPTIONS,
    defaultFilters,
  } from '$lib/api/search';
  import { parseSearchQuery } from '$lib/search/syntax';
  import {
    formatWords,
    relativeTime,
    detectSite,
    stripHtml,
  } from '$lib/util';

  let { data } = $props();
```

The imports bring in everything we need:

- **`page`** from SvelteKit's app state — gives us access to the current page URL and params.
- **`goto`** for client-side navigation.
- **`search`** — our API client function that hits the backend at `/api/v0/search`.
- **Types** — `SearchFilters`, `SearchResult`, `SearchResponse` for TypeScript safety. These types mirror the backend's response format.
- **Constants** — `SORT_OPTIONS`, `COMPLETE_OPTIONS`, `SOURCE_OPTIONS` are arrays of `{value, label}` objects for dropdown menus. They're defined once in the search API module and shared across components.
- **`defaultFilters`** — returns a fresh `SearchFilters` object with all defaults (empty strings, null values, page 1).
- **`parseSearchQuery`** — the syntax parser that converts query strings like `fandom:Harry Potter words:>10000` into structured `SearchFilters`.
- **Utility functions** — `formatWords` adds commas to numbers, `relativeTime` shows "3 days ago," `detectSite` identifies the fanfiction platform, and `stripHtml` removes HTML tags from summaries.

The `{ data }` prop comes from the page loader. It contains `q` and `tab` from the URL.

### Search Tab State

```svelte
  type SearchTab = 'work' | 'people' | 'bookmark' | 'tag';
  let activeTab = $state<SearchTab>((data.tab as SearchTab) || 'work');

  const searchTabs: { id: SearchTab; label: string }[] = [
    { id: 'work', label: 'Work Search' },
    { id: 'people', label: 'People Search' },
    { id: 'bookmark', label: 'Bookmark Search' },
    { id: 'tag', label: 'Tag Search' },
  ];
```

The search page has its own tab system — separate from the layout's tabs. This is a common pattern: the layout handles app-level navigation (Download/Recs/Sugg), while individual pages handle their own sub-navigation.

The `activeTab` is initialized from the URL parameter (`data.tab`), so if the user arrives with `?tab=tag`, the tag tab is pre-selected. If no tab is specified, it defaults to `'work'`.

The `searchTabs` array follows the same data-driven pattern as the layout's tabs: an array of objects that the template loops over. This consistency makes the codebase predictable — every tab system in the app works the same way.

### Query and Filter State

```svelte
  let queryInput = $state(data.q ?? '');
  let filters = $state<SearchFilters>(defaultFilters());
  let loading = $state(false);
  let error = $state('');
  let results = $state<SearchResult[]>([]);
  let total = $state(0);
  let currentPage = $state(1);
  let searched = $state(false);
```

These variables hold the entire search state:

- **`queryInput`** — the raw text the user types in the search box. This might contain syntax like `fandom:Harry Potter words:>10000`. It's initialized from the URL's `q` parameter.
- **`filters`** — the parsed `SearchFilters` object sent to the API. This is what the backend actually receives. Updated whenever the user searches.
- **`loading`** — true while a search request is in flight. Used to show a spinner and disable the Search button.
- **`error`** — error message if the search fails. Empty string when there's no error.
- **`results`** — the array of search results from the API. Each result has a title, author, tags, word count, and more.
- **`total`** — total number of matching results (for pagination). The API returns this even though we only fetch 20 results per page.
- **`currentPage`** — which page of results we're on. Starts at 1.
- **`searched`** — whether the user has performed at least one search. This prevents showing "No results found" before the user has even searched.

### Advanced Filter Fields

```svelte
  let filterComplete = $state('');
  let filterSource = $state('');
  let filterMinWords = $state('');
  let filterMaxWords = $state('');
  let filterMinChapters = $state('');
  let filterMaxChapters = $state('');
  let filterDateFrom = $state('');
  let filterDateTo = $state('');
  let filterSort = $state('');
  let filterIncludeTags = $state('');
  let filterExcludeTags = $state('');
```

These are the individual filter fields shown in the Work Search tab. Each one maps to a specific filter in the `SearchFilters` object. When the user selects "Complete Only" from the dropdown, `filterComplete` becomes `'true'`. When the user types "5000" in Min Words, `filterMinWords` becomes `'5000'`.

All filters start as empty strings, meaning "not set." In `doSearch()`, empty filter fields defer to the syntax-parsed values. This lets users combine the query syntax with the form UI seamlessly.

### Auto-Search on Mount

```svelte
  let initialized = $state(false);

  $effect(() => {
    if (data.q && !initialized) {
      initialized = true;
      queryInput = data.q;
      doSearch();
    }
  });
```

This is a clever pattern. If the user arrives with a query in the URL (e.g., from the navbar search), we automatically execute the search on mount. The `initialized` guard ensures we only auto-search once — not every time the effect re-runs.

The `$effect` is Svelte 5's reactivity primitive. It runs whenever its dependencies change. Since it reads `data.q`, it runs when the component mounts (or when `data` changes during navigation). Without the `initialized` guard, the effect would run again every time `data` changed, potentially causing an infinite search loop.

> **Try It Yourself:** Open a new tab and go to `http://localhost:5173/search?q=Harry+Potter`. The search executes automatically — you don't need to click the Search button. That's the auto-search on mount doing its job. The `initialized` flag prevents it from running again if you navigate away and come back.

## The doSearch Function

This is the heart of the search page. When called, it:

1. Sets loading state and clears errors.
2. Parses the query syntax into filters.
3. Merges with advanced filter fields.
4. Updates the URL to reflect the current search.
5. Calls the search API.
6. Updates the result state with the response.

Let's look at it piece by piece:

```svelte
  async function doSearch() {
    loading = true;
    error = '';
    searched = true;
```

The function starts by setting `loading` to true (which shows a spinner), clearing any previous error, and marking that a search has been performed.

```typescript
    // Parse syntax from query input.
    const syntaxFilters = parseSearchQuery(queryInput);
```

This calls our syntax parser (from `$lib/search/syntax`). It converts a query like `fandom:Harry Potter words:>10000 complete:true` into a `SearchFilters` object with `include_tags: '1:Harry Potter'`, `min_words: 10000`, and `complete: true`. We covered the syntax parser in detail in earlier chapters — this is where it gets used.

```typescript
    // Merge with advanced filter fields (advanced overrides syntax).
    filters = {
      ...syntaxFilters,
      q: filterComplete === '' ? syntaxFilters.q : syntaxFilters.q,
      min_words: filterMinWords ? Number(filterMinWords) : syntaxFilters.min_words,
      max_words: filterMaxWords ? Number(filterMaxWords) : syntaxFilters.max_words,
      min_chapters: filterMinChapters ? Number(filterMinChapters) : syntaxFilters.min_chapters,
      max_chapters: filterMaxChapters ? Number(filterMaxChapters) : syntaxFilters.max_chapters,
      complete: filterComplete === 'true'
        ? true
        : filterComplete === 'false'
          ? false
          : syntaxFilters.complete,
      source: filterSource || syntaxFilters.source,
      date_from: filterDateFrom || syntaxFilters.date_from,
      date_to: filterDateTo || syntaxFilters.date_to,
      sort: filterSort || syntaxFilters.sort,
      include_tags: filterIncludeTags || syntaxFilters.include_tags,
      exclude_tags: filterExcludeTags || syntaxFilters.exclude_tags,
      page: currentPage,
      per_page: 20,
    };
```

This is the merge step. The spread operator `...syntaxFilters` copies all the parsed syntax values as the base. Then each advanced filter field overrides its corresponding value — *but only if the user filled it in*.

For example, if the user typed `words:>10000` in the search bar AND typed "5000" in the Min Words field, the form field wins (5000). If the Min Words field is empty, the syntax value wins (10000).

The pattern is consistent: `filterField ? Number(filterField) : syntaxFilters.field` for numbers, and `filterField || syntaxFilters.field` for strings. The `||` operator returns the left side if it's truthy (non-empty string), otherwise the right side.

This means the user can use both the query syntax AND the filter dropdowns simultaneously. The form fields always win when set. The syntax provides the "power user" path, and the form provides the "visual" path.

> **Watch Out:** The `q` line is interesting: `q: filterComplete === '' ? syntaxFilters.q : syntaxFilters.q`. This looks like a no-op — it's the same either way! That's intentional: the `q` field (the free-text search term) always comes from the syntax parser. The advanced form doesn't have its own text search field for the free-text query; it relies on the query input for that. The form fields only control filters like completion status, word count, and tags. The free-text query always flows through the syntax parser.

### URL Sync

```typescript
    // Update URL without navigation.
    const qs = new URLSearchParams();
    if (queryInput) qs.set('q', queryInput);
    if (activeTab !== 'work') qs.set('tab', activeTab);
    goto(`/search?${qs.toString()}`, { replaceState: true, keepFocus: true });
```

After parsing the query, we update the browser URL to reflect the current search. This is important for two reasons:

1. **Bookmarkability** — the user can bookmark the search URL and come back to it later.
2. **Shareability** — the user can copy the URL and share it with someone else.

The `replaceState: true` option means we use `history.replaceState` instead of `history.pushState` — so the URL updates without creating a new entry in the browser history. The user can still hit Back to leave the search page entirely, but pressing Back within a search doesn't create a chain of identical URLs.

`keepFocus: true` ensures the search input stays focused after the URL update, so the user doesn't lose their typing position. This is a small UX detail that makes a big difference — without it, the user would have to click back into the input after every search.

### The API Call

```typescript
    try {
      const res: SearchResponse = await search(filters);
      total = res.total;
      results = res.results;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed.';
      results = [];
      total = 0;
    } finally {
      loading = false;
    }
```

The `search()` function (from `$lib/api/search`) builds a query string from the filters and fetches `/api/v0/search`. The backend parses the filters, queries the database (using Tantivy for full-text search and PostgreSQL for structured queries), and returns the results.

The `try/catch` handles network errors, server errors, and unexpected responses. The `finally` block always sets `loading = false`, even if the request fails. This ensures the spinner always stops, even on error.

The `instanceof Error` check ensures we get a meaningful error message. If the error is a standard `Error` object, we use its `.message`. Otherwise, we fall back to a generic "Search failed" message.

### Pagination Helpers

```svelte
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      currentPage = 1;
      doSearch();
    }
  }

  function nextPage() {
    currentPage++;
    doSearch();
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      doSearch();
    }
  }

  let totalPages = $derived(Math.ceil(total / 20));
```

When the user presses Enter in the search box, we reset to page 1 and search — pressing Enter always starts a new search from page 1. Next/Previous buttons increment/decrement the page number and re-search. The `prevPage` function has a guard to prevent going below page 1.

`totalPages` is a derived value — Svelte 5's `$derived` automatically recomputes it whenever `total` changes. It divides the total result count by 20 (the results per page) and rounds up with `Math.ceil`. So if there are 45 results, `totalPages` is 3 (45/20 = 2.25, rounded up to 3).

## The Template: Query Row and Tabs

```svelte
<div class="search-page">
  <h1>Advanced Search</h1>

  <div class="query-row">
    <input
      type="search"
      class="query-input"
      placeholder='Try: fandom:"Harry Potter" tag:Fluff complete:true sort:updated'
      bind:value={queryInput}
      onkeydown={onKeydown}
      aria-label="Search query"
    />
    <button class="btn" onclick={() => { currentPage = 1; doSearch(); }} disabled={loading}>
      {#if loading}<span class="spinner"></span> Searching…{:else}Search{/if}
    </button>
    <a class="syntax-link muted" href="/search/syntax" target="_blank">Syntax guide</a>
  </div>
```

The query row has three elements side by side:

1. **The search input** — a wide text field with a carefully chosen placeholder. The placeholder `fandom:"Harry Potter" tag:Fluff complete:true sort:updated` demonstrates four different syntax features in one query: a tag filter, a freeform filter, a boolean filter, and a sort option. It's an advertisement for the syntax system.

2. **The Search button** — shows a spinner while loading, and is disabled during search to prevent double-clicks. The `{#if}` block switches between "Searching..." with a spinner and a plain "Search" label.

3. **The Syntax guide link** — opens the syntax guide page in a new tab. We'll cover that page in Chapter 28. The `target="_blank"` means the user doesn't lose their current search.

### The Search Tabs

```svelte
  <div class="search-tabs">
    {#each searchTabs as t}
      <button
        class="stab"
        class:active={activeTab === t.id}
        onclick={() => { activeTab = t.id; }}
      >
        {t.label}
      </button>
    {/each}
  </div>
```

Similar to the layout's tab bar, but these are the search-specific tabs. The CSS class is `stab` (short for "search tab") to avoid conflicts with the layout's `.tab` class. Each tab switches the filter panel below it.

## Understanding Search Results

Before we look at how results are displayed, let's understand what a search result looks like. The `SearchResult` type defines the data structure for each fic:

```typescript
export interface SearchResult {
  url_id: string;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  updated: string | null;
  rank: number | null;
  tags: SearchTag[];
  total_freeform: number;
}
```

Each field tells a story:

- **`url_id`** — the unique identifier for this fic. Used as a React/Svelte key for efficient rendering.
- **`title`** — the fic's title, exactly as stored in the database.
- **`author`** — the author's name.
- **`source`** — the full URL to the original fic on its host site.
- **`words`** — the word count as a number (not formatted — formatting happens in the template with `formatWords`).
- **`chapters`** — the chapter count.
- **`status`** — "Complete" or "In Progress."
- **`description`** — the fic's summary, which may contain HTML tags.
- **`updated`** — ISO timestamp of the last update, or `null` if unknown.
- **`rank`** — the relevance score from Tantivy, or `null` if not applicable.
- **`tags`** — an array of `SearchTag` objects attached to this fic.
- **`total_freeform`** — the total number of freeform tags (we only display the first 12).

Each tag in the `tags` array has its own structure:

```typescript
export interface SearchTag {
  name: string;
  type: string;
  type_id: number;
  score: number;
}
```

The `type` is a human-readable name like "Fandom" or "Freeform." The `type_id` is the numeric ID (1–4). The `score` is how relevant this tag is to the search query — higher means more relevant.

## Results Display

### The Results Header

```svelte
  {#if results.length > 0}
    <div class="results-header">
      <span class="muted">{total.toLocaleString()} results</span>
      <span class="muted">Page {currentPage} of {totalPages}</span>
    </div>
```

A simple header showing the total count and current page. `toLocaleString()` adds commas to large numbers (e.g., 12,345). The muted class keeps it subtle — it's informational, not the main attraction.

### Result Cards

```svelte
    <div class="results">
      {#each results as r (r.url_id)}
        <div class="card result-card">
          <div class="result-main">
            <h3>
              <a href={r.source} target="_blank" rel="noopener">{r.title}</a>
            </h3>
            <p class="muted">by {r.author} · <span class="tag">{detectSite(r.source)}</span></p>
            <p class="meta-line">
              {formatWords(r.words)} words · {r.chapters} chapters · {r.status}
            </p>
            {#if r.description}
              <p class="desc">{stripHtml(r.description).slice(0, 250)}</p>
            {/if}
```

Each result card shows:

- **Title** — linked to the original fic URL, opening in a new tab. The `rel="noopener"` prevents the new tab from having access to the original page's `window` object (a security measure).
- **Author and site** — the author name and a detected site badge (AO3, FF.net, etc.). The `detectSite` utility inspects the URL to identify the platform.
- **Metadata** — word count (formatted with commas), chapter count, and completion status. This gives users a quick overview without clicking into the fic.
- **Description** — the first 250 characters of the summary, with HTML stripped. The `stripHtml` function removes `<p>`, `<br>`, and other tags, converting HTML entities like `&amp;` to their plain-text equivalents.

The `{#each results as r (r.url_id)}` uses the `url_id` as a key. This tells Svelte how to identify each result card, which helps with efficient DOM updates when results change.

### Tag Pills

```svelte
            {#if r.tags.length > 0}
              <div class="tags">
                {#each r.tags.slice(0, 12) as t}
                  <span class="tag-pill" title="{t.type}: score {t.score}">
                    {t.name}
                  </span>
                {/each}
                {#if r.total_freeform > 12}
                  <span class="tag-pill muted">+{r.total_freeform - 12} more</span>
                {/if}
              </div>
            {/if}
```

We show up to 12 tags per result. If there are more, we display a "+N more" indicator. Each tag pill shows the tag name and has a tooltip (via `title` attribute) showing the tag type and relevance score.

The `total_freeform` field tells us how many freeform tags the fic has in total — not just the 12 we're showing. This lets us display an accurate count of hidden tags.

> **Why limit to 12 tags?** Some fics have dozens of freeform tags. Showing all of them would make each result card take up the entire screen. The 12-tag limit keeps results scannable while still showing the most relevant tags. Users who need more detail can click through to the original fic.

### The Right Sidebar

```svelte
          <div class="result-meta">
            {#if r.rank}
              <span class="rank">#{Math.round(r.rank * 10) / 10}</span>
            {/if}
            {#if r.updated}
              <span class="muted">{relativeTime(r.updated)}</span>
            {/if}
            <a class="btn btn-secondary sm" href="/?q={encodeURIComponent(r.source)}">
              Download
            </a>
          </div>
```

On the right side of each result card, we show:

- **Search rank** — how well the fic matched the query, rounded to one decimal place. A lower rank means a better match. The `Math.round(r.rank * 10) / 10` formula rounds to one decimal — for example, 3.76 becomes 3.8.
- **Relative update time** — "3 days ago," "2 weeks ago," etc. The `relativeTime` function calculates this from the ISO timestamp.
- **Download button** — a small secondary button that navigates to the home page with the fic's URL pre-filled. The user can then choose their export format and download.

### Pagination Controls

```svelte
    <div class="pagination">
      <button class="btn btn-secondary" onclick={prevPage} disabled={currentPage <= 1}>
        ← Previous
      </button>
      <span class="muted">Page {currentPage} of {totalPages}</span>
      <button class="btn btn-secondary" onclick={nextPage} disabled={currentPage >= totalPages}>
        Next →
      </button>
    </div>
  {/if}
```

Simple Previous/Next buttons with the current page indicator. The buttons are disabled when there are no more pages to go to — Previous is disabled on page 1, and Next is disabled on the last page.

> **Try It Yourself:** Search for `fandom:Harry Potter complete:true` and look at the results. Click "Next" to page 2. Notice the URL updates with the new page number. Now hit your browser's Back button — you go back to page 1. Hit Back again — you leave the search page entirely. That's the `replaceState` option working correctly: it creates one history entry for the search, not one per page.

---

# Chapter 27: Search Filters and Results

## The Work Search Filter Grid

Below the search tabs, the Work Search tab shows a grid of filter controls. This is where users who prefer point-and-click over query syntax can build their searches visually.

### The Complete Filter Grid

```svelte
  {#if activeTab === 'work'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Title / Any Field
          <input type="text" bind:value={queryInput}
            placeholder="Search in title or any field…" />
        </label>

        <label class="filter-item">
          Completion Status
          <select bind:value={filterComplete}>
            {#each COMPLETE_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Site
          <select bind:value={filterSource}>
            {#each SOURCE_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Sort By
          <select bind:value={filterSort}>
            {#each SORT_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Min Words
          <input type="number" bind:value={filterMinWords} placeholder="0" min="0" />
        </label>

        <label class="filter-item">
          Max Words
          <input type="number" bind:value={filterMaxWords} placeholder="∞" min="0" />
        </label>

        <label class="filter-item">
          Min Chapters
          <input type="number" bind:value={filterMinChapters} placeholder="0" min="1" />
        </label>

        <label class="filter-item">
          Max Chapters
          <input type="number" bind:value={filterMaxChapters} placeholder="∞" min="1" />
        </label>

        <label class="filter-item">
          Date After
          <input type="date" bind:value={filterDateFrom} />
        </label>

        <label class="filter-item">
          Date Before
          <input type="date" bind:value={filterDateTo} />
        </label>

        <label class="filter-item full">
          Include Tags (type_id:name format)
          <input type="text" bind:value={filterIncludeTags}
            placeholder="1:Harry Potter,4:Fluff" />
        </label>

        <label class="filter-item full">
          Exclude Tags
          <input type="text" bind:value={filterExcludeTags}
            placeholder="4:Major Character Death" />
        </label>
      </div>
    </div>
  {/if}
```

Let's break down each filter and explain what it does and how it connects to the backend.

### Completion Status

```svelte
<label class="filter-item">
  Completion Status
  <select bind:value={filterComplete}>
    {#each COMPLETE_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `COMPLETE_OPTIONS` array (from `$lib/api/search`) defines three choices:

| Value | Label |
|-------|-------|
| `''` | All Works |
| `'true'` | Complete Only |
| `'false'` | In Progress Only |

When the user selects "Complete Only," `filterComplete` becomes `'true'`. In `doSearch()`, this gets converted to the boolean `true` in the `SearchFilters` object. The backend then adds `WHERE completed = true` to its SQL query.

The dropdown uses `bind:value` for two-way binding — when the user selects an option, the state updates immediately. No click handlers needed. This is one of Svelte's most loved features — form elements just work with minimal boilerplate.

### Site Filter

```svelte
<label class="filter-item">
  Site
  <select bind:value={filterSource}>
    {#each SOURCE_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `SOURCE_OPTIONS` array maps site abbreviations to their full domain names:

| Value | Label |
|-------|-------|
| `''` | All Sites |
| `'archiveofourown.org'` | Archive of Our Own |
| `'fanfiction.net'` | FanFiction.net |
| `'fictionpress.com'` | FictionPress |
| `'forums.spacebattles.com'` | SpaceBattles |
| `'forums.sufficientvelocity.com'` | SufficientVelocity |

The backend queries work with the full domain names, so the filter value is the domain, not the abbreviation. This keeps the API clean — it doesn't need to know about abbreviations or nicknames. The frontend handles the translation from human-friendly labels to machine-friendly domain names.

### Sort Options

```svelte
<label class="filter-item">
  Sort By
  <select bind:value={filterSort}>
    {#each SORT_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `SORT_OPTIONS` array:

| Value | Label |
|-------|-------|
| `''` | Relevance |
| `'updated'` | Date Updated |
| `'created'` | Date Published |
| `'words'` | Word Count |
| `'kudos'` | Kudos Count |

An empty string means "sort by relevance" — the default. The backend uses Tantivy's relevance scoring when no explicit sort is specified. Relevance is based on how well the fic matches the search terms — fics with more matching terms and higher term frequency rank higher.

### Word Count Range

```svelte
<label class="filter-item">
  Min Words
  <input type="number" bind:value={filterMinWords} placeholder="0" min="0" />
</label>

<label class="filter-item">
  Max Words
  <input type="number" bind:value={filterMaxWords} placeholder="∞" min="0" />
</label>
```

Two number inputs for the word count range. The placeholder "∞" suggests "no limit" when the field is empty. In `doSearch()`, empty fields are treated as `null` (no constraint).

The `type="number"` attribute gives us a numeric input with increment/decrement buttons in some browsers. The `min="0"` attribute prevents negative numbers. You can leave either field blank — an empty Min Words means "no minimum," and an empty Max Words means "no maximum."

> **Try It Yourself:** Leave Min Words empty and set Max Words to 1000. Search for `fandom:Harry Potter`. You'll see only short fics — one-shots and flash fiction. Now set Min Words to 50000 and clear Max Words. You'll see only epic-length fics. The word count filters are great for finding fics that match your reading time.

### Chapter Range

```svelte
<label class="filter-item">
  Min Chapters
  <input type="number" bind:value={filterMinChapters} placeholder="0" min="1" />
</label>

<label class="filter-item">
  Max Chapters
  <input type="number" bind:value={filterMaxChapters} placeholder="∞" min="1" />
</label>
```

Same pattern as word count, but for chapters. Note `min="1"` on the inputs — you can't have zero chapters (a fic with no chapters isn't a fic). The chapter filter is useful for finding multi-chapter epics or single-chapter one-shots.

### Date Range

```svelte
<label class="filter-item">
  Date After
  <input type="date" bind:value={filterDateFrom} />
</label>

<label class="filter-item">
  Date Before
  <input type="date" bind:value={filterDateTo} />
</label>
```

The `<input type="date">` gives us a native date picker. The browser shows a calendar widget when the user clicks the field. The value is stored as a `YYYY-MM-DD` string.

The `date_from` and `date_to` fields in `SearchFilters` accept ISO timestamps. In `doSearch()`, we pass them directly — the backend handles the date parsing. If you need more precise control (e.g., "after January 2024"), you can use the query syntax: `after:2024-01`.

### Include/Exclude Tags

```svelte
<label class="filter-item full">
  Include Tags (type_id:name format)
  <input type="text" bind:value={filterIncludeTags}
    placeholder="1:Harry Potter,4:Fluff" />
</label>

<label class="filter-item full">
  Exclude Tags
  <input type="text" bind:value={filterExcludeTags}
    placeholder="4:Major Character Death" />
</label>
```

These text fields use the `type_id:name` format. The type IDs are:

| ID | Type |
|----|------|
| 1 | Fandom |
| 2 | Character |
| 3 | Relationship |
| 4 | Freeform |

So `1:Harry Potter` means "include the Harry Potter fandom tag," and `4:Fluff` means "include the Fluff freeform tag." Multiple tags are comma-separated: `1:Harry Potter,4:Fluff` means "fics in the Harry Potter fandom tagged Fluff."

> **Watch Out:** The tag format is technical. Regular users will probably use the query syntax (`fandom:Harry Potter tag:Fluff`) instead. These fields are more useful for developers testing specific filter combinations, or for users who want to paste in a complex tag filter. The syntax guide (Chapter 28) teaches the friendlier syntax.

### The Grid Layout

```css
.filter-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.8rem;
}
.filter-item {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.85rem;
  color: var(--color-muted);
}
.filter-item.full {
  grid-column: 1 / -1;
}
.filter-item select,
.filter-item input {
  font-size: 0.88rem;
}
```

The filter grid is a two-column CSS Grid. Each `filter-item` is a label with a child input/select, stacked vertically with `flex-direction: column`. The label text is muted, and the input/select below it is slightly larger.

Items with the `full` class span both columns (`grid-column: 1 / -1`). This is used for the title field (which needs width) and the tag fields (which can be long strings). This pattern — full-width items mixed with half-width items — is common in form layouts.

### The SearchFilters Type

Behind every filter is the `SearchFilters` type, defined in `$lib/api/search.ts`:

```typescript
export interface SearchFilters {
  q: string;
  include_tags: string;
  exclude_tags: string;
  include_any_tags: string;
  min_words: number | null;
  max_words: number | null;
  min_chapters: number | null;
  max_chapters: number | null;
  complete: boolean | null;
  source: string;
  date_from: string;
  date_to: string;
  sort: string;
  page: number;
  per_page: number;
}
```

Every filter in the UI maps to a field in this type:

- **`q`** — the free-text search query. Comes from the query input or the syntax parser.
- **`include_tags`** — comma-separated `type_id:name` pairs for tags to include.
- **`exclude_tags`** — comma-separated `type_id:name` pairs for tags to exclude.
- **`min_words` / `max_words`** — word count range. `null` means no constraint.
- **`min_chapters` / `max_chapters`** — chapter count range. `null` means no constraint.
- **`complete`** — `true` for completed only, `false` for in-progress only, `null` for all.
- **`source`** — the site domain. Empty string means all sites.
- **`date_from` / `date_to`** — ISO timestamp strings. Empty means no constraint.
- **`sort`** — sort order. Empty string means relevance.
- **`page` / `per_page`** — pagination. Default is page 1, 20 results per page.

The `defaultFilters()` function returns a fresh instance with all defaults:

```typescript
export function defaultFilters(): SearchFilters {
  return {
    q: '',
    include_tags: '',
    exclude_tags: '',
    include_any_tags: '',
    min_words: null,
    max_words: null,
    min_chapters: null,
    max_chapters: null,
    complete: null,
    source: '',
    date_from: '',
    date_to: '',
    sort: '',
    page: 1,
    per_page: 20,
  };
}
```

This is the starting point for every search. The syntax parser and form fields fill in the values, and the `search()` function sends them to the backend.

### The buildSearchQuery Function

The `buildSearchQuery` function converts a `SearchFilters` object into a URL query string:

```typescript
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();
  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  if (filters.exclude_tags) params.set('exclude_tags', filters.exclude_tags);
  if (filters.min_words !== null) params.set('min_words', String(filters.min_words));
  // ... etc for all fields
  if (filters.page > 1) params.set('page', String(filters.page));
  if (filters.per_page !== 20) params.set('per_page', String(filters.per_page));
  return params.toString();
}
```

Notice that it only includes non-default values. If `min_words` is `null`, it's not added to the query string. If `page` is 1 (the default), it's not included. This keeps the URL clean — no unnecessary parameters.

The `search()` function then uses this query string to fetch `/api/v0/search`:

```typescript
export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const qs = buildSearchQuery(filters);
  const res = await fetch(`/api/v0/search${qs ? '?' + qs : ''}`);
  if (!res.ok) throw new Error(`Search failed (${res.status})`);
  return (await res.json()) as SearchResponse;
}
```

The `SearchResponse` type contains the results, total count, and pagination info:

```typescript
export interface SearchResponse {
  total: number;
  page: number;
  per_page: number;
  results: SearchResult[];
}
```

On mobile, the grid collapses to one column:

```css
@media (max-width: 600px) {
  .filter-grid {
    grid-template-columns: 1fr;
  }
}
```

## The Placeholder Tabs

FicHub's search supports four types of searches, but only Work Search has a fully functional filter UI. The other three tabs show placeholder forms with a "coming soon" message.

### People Search

```svelte
  {#if activeTab === 'people'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Author Name
          <input type="text" placeholder="Search by author name…" />
        </label>
        <label class="filter-item full">
          Fandom
          <input type="text" placeholder="Filter by fandom…" />
        </label>
      </div>
      <p class="muted">People search coming soon — use syntax: author:Name</p>
    </div>
  {/if}
```

The People Search tab shows two input fields (author name and fandom) plus a message saying it's coming soon. These fields aren't wired up yet — they're placeholders for a future feature.

The hint "use syntax: author:Name" tells users they can achieve the same result today by using the query syntax. The backend already supports author searches — only the form UI is missing.

### Bookmark Search

```svelte
  {#if activeTab === 'bookmark'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Bookmarked Item
          <input type="text" placeholder="Title or author of bookmarked fic…" />
        </label>
        <label class="filter-item">
          Word Count
          <input type="text" placeholder="e.g. >10000" />
        </label>
        <label class="filter-item">
          Has Rec
          <select>
            <option value="">Any</option>
            <option value="true">Rec only</option>
          </select>
        </label>
      </div>
      <p class="muted">Bookmark search coming soon — use syntax: words:>10000</p>
    </div>
  {/if}
```

Bookmark Search lets you search through your bookmarked fics. Like People Search, it's a placeholder. The form shows fields for the bookmarked item name, word count range, and whether it's a recommendation.

### Tag Search

```svelte
  {#if activeTab === 'tag'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Tag Name
          <input type="text" placeholder="Search tags…" />
        </label>
        <label class="filter-item">
          Tag Type
          <select>
            <option value="">Any</option>
            <option value="1">Fandom</option>
            <option value="2">Character</option>
            <option value="3">Relationship</option>
            <option value="4">Freeform</option>
          </select>
        </label>
      </div>
      <p class="muted">Tag search coming soon — use syntax: fandom:Harry Potter</p>
    </div>
  {/if}
```

Tag Search lets you search the tag database directly — find tags by name, filter by type (fandom, character, relationship, freeform). Again, a placeholder with a syntax hint.

> **Why placeholders?** Building the full UI for every search type takes time. The backend supports all these searches through the query syntax (`author:Name`, `words:>10000`, `fandom:Harry Potter`). The placeholders show users what's coming while the syntax gives them a way to use these features now. It's a pragmatic approach — ship what works, plan what's next. The placeholders also serve as a design sketch — they show what the form will look like when it's built. And they give users something to click on, which is better than a blank page with "Coming Soon."

## Error and Empty States

```svelte
  {#if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {/if}

  {#if searched && !loading && results.length === 0 && !error}
    <div class="card empty">
      <p class="muted">No results found. Try different keywords or filters.</p>
    </div>
  {/if}
```

Two important states:

1. **Error** — shown when the API call fails. The error message is displayed in a red-bordered card with a warning emoji. The `error-card` class adds a red border using `var(--color-error)`.

2. **Empty** — shown only after a search completes with zero results. The `searched && !loading` guard ensures we don't show "No results" before the first search or while loading. The `!error` guard ensures we don't show both the error and empty state simultaneously.

The empty state message is helpful: "Try different keywords or filters" — it gives the user a concrete next step instead of just saying "nothing found." Good error and empty states are a mark of polished UX.

## The Results CSS

```css
.results {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}
.result-card {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
}
.result-card h3 {
  margin: 0 0 0.3rem;
  font-size: 1.1rem;
}
.result-card h3 a {
  color: var(--color-text);
}
.meta-line {
  margin: 0.3rem 0;
  font-size: 0.9rem;
}
.desc {
  color: var(--color-muted);
  font-size: 0.85rem;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin-top: 0.5rem;
}
.tag-pill {
  background: var(--color-surface-2);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  padding: 0.1rem 0.5rem;
  font-size: 0.75rem;
  color: var(--color-muted);
}
.result-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.3rem;
  flex-shrink: 0;
}
.rank {
  font-weight: 700;
  font-size: 0.9rem;
  color: var(--color-primary);
}
```

The results list is a flex column with cards stacked vertically. Each card is a flex row with the main content on the left and metadata (rank, time, download button) on the right.

The tag pills use `border-radius: 999px` — a huge border radius that makes any rectangle into a pill shape. This is a common CSS trick for rounded badges. The pills have a subtle background and border, making them look like clickable chips.

The `flex-shrink: 0` on `.result-meta` prevents the right column from shrinking when the left column's content is wide. Without this, a long title could squeeze the download button into invisibility. This is a common flex layout gotcha — always set `flex-shrink: 0` on fixed-width elements.

The `.rank` class uses the primary color to make the rank number stand out. This is the search engine's relevance score — it tells users how well the fic matches their query. A rank of `#1.2` means it's a very strong match; `#8.5` means it's a weaker match.

### Mobile Responsive

```css
@media (max-width: 600px) {
  .query-row {
    flex-direction: column;
  }
  .result-card {
    flex-direction: column;
  }
  .result-meta {
    flex-direction: row;
    align-items: center;
  }
}
```

On mobile, the result card stacks vertically (title on top, metadata below), and the query row stacks too (search input above the button). The result metadata switches to a horizontal layout so the rank, time, and download button sit in a row instead of stacking.

---

# Chapter 28: The Syntax Guide

## Why a Syntax Guide?

The query syntax is FicHub's power feature. It lets you express complex search queries in a single line — queries that would take many clicks with dropdown menus alone. But here's the thing: **nobody knows a syntax they haven't been taught**.

Think about it: if you've never seen `fandom:Harry Potter words:>10000`, you'd have no idea that was a valid query. You'd use the dropdown menus, which are fine for simple searches but tedious for complex ones. The syntax guide bridges the gap between "I know what I want" and "I know how to ask for it."

The syntax guide exists to:

1. **Teach users the keywords** — what `fandom:`, `words:`, `complete:` mean.
2. **Show the short aliases** — `f:`, `w:`, `s:` for power users.
3. **Demonstrate combinations** — how to chain multiple filters in one query.
4. **Provide examples** — real queries that users can copy and modify.
5. **Reduce support requests** — instead of asking "how do I search for completed fics?", users can check the guide.

The guide lives at `/search/syntax` and is linked from the search page's "Syntax guide" link. It opens in a new tab, so users don't lose their search.

## The Guide Page Structure

The syntax guide is a static page — no dynamic data, no API calls. It's pure HTML and CSS. This makes it fast to load and easy to maintain. No JavaScript means no loading states, no error states, no interactivity that could break.

```svelte
<svelte:head>
  <title>Search Syntax Guide — FicHub</title>
</svelte:head>

<div class="syntax-page">
  <h1>Search Syntax Guide</h1>
  <p class="subtitle">
    FicHub's search bar supports a powerful query syntax.
    Mix and match these keywords to find exactly what you're looking for.
  </p>
```

The `<svelte:head>` block sets the page title — this appears in the browser tab and search engine results. The subtitle sets the tone: this is a friendly guide, not a dry reference manual.

## Full-Text Search

The first section covers the basics — searching by words:

```svelte
  <section>
    <h2>Full-Text Search</h2>
    <p>Type any words to search across titles, summaries, and tags.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">Harry Potter</td>
        <td>Finds fics mentioning "Harry Potter" in any field</td>
      </tr>
      <tr>
        <td class="code">title:Dragon</td>
        <td>Searches only in the title</td>
      </tr>
      <tr>
        <td class="code">author:cleo</td>
        <td>Searches only in the author name</td>
      </tr>
      <tr>
        <td class="code">author:cleo fandom:Harry Potter</td>
        <td>Author search combined with fandom filter</td>
      </tr>
    </table>
  </section>
```

The table format is consistent throughout the guide: code on the left, description on the right. This makes it easy to scan — you can visually separate "what to type" from "what it does."

### How It Works Under the Hood

When you type `Harry Potter` (bare words with no prefix), the syntax parser collects those words into the `q` field of `SearchFilters`. The backend's Tantivy search engine then searches across the title, summary, and tag fields for those words. This is a full-text search — it finds fics where those words appear anywhere.

When you type `title:Dragon`, the parser extracts "Dragon" and adds it to the `q` field. The `title:` prefix tells the backend to search only in the title field. This is more precise — it won't match fics that mention "Dragon" in the summary but don't have it in the title.

The `author:` prefix works the same way — it narrows the search to the author field. You can combine these with other keywords: `author:cleo fandom:Harry Potter` finds fics by "cleo" in the Harry Potter fandom.

> **Try It Yourself:** Type `author:cleo fandom:Harry Potter` in the search bar and press Enter. The parser splits this into two parts: the author name "cleo" and the fandom tag "Harry Potter." The backend searches for fics where the author matches AND the fandom tag matches. Try removing the `author:` prefix and see how the results change — now "cleo" is searched as a full-text term, which might match fics that mention "cleo" in the summary.

## Tag Filters

The next section covers tag-specific searches:

```svelte
  <section>
    <h2>Tag Filters</h2>
    <p>Filter by specific tag types. These are the building blocks of precise searches.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">fandom:Harry Potter</td>
        <td>Only fics in the Harry Potter fandom</td>
      </tr>
      <tr>
        <td class="code">char:Draco Malfoy</td>
        <td>Fics featuring Draco Malfoy</td>
      </tr>
      <tr>
        <td class="code">rel:Harry/Draco</td>
        <td>Fics with the Harry/Draco relationship</td>
      </tr>
      <tr>
        <td class="code">tag:Fluff</td>
        <td>Fics tagged with "Fluff" (freeform tag)</td>
      </tr>
      <tr>
        <td class="code">-fandom:Naruto</td>
        <td>Exclude the Naruto fandom</td>
      </tr>
      <tr>
        <td class="code">-tag:Angst</td>
        <td>Exclude fics tagged "Angst"</td>
      </tr>
    </table>
  </section>
```

The key insight here is the **exclusion operator**: prefix any tag filter with a minus sign (`-`) to exclude it. `-fandom:Naruto` means "not in the Naruto fandom." You can combine inclusions and exclusions freely: `fandom:Harry Potter -tag:Angst` finds Harry Potter fics without the Angst tag.

Under the hood, the parser calls `appendTag(filters.exclude_tags, 1, 'Naruto')` for `-fandom:Naruto`, which produces the string `1:Naruto` in the `exclude_tags` field. The backend then excludes fics with that tag.

The `appendTag` function is simple:

```typescript
function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}
```

If there are no existing tags, it returns `1:Naruto`. If there's already a tag, it appends with a comma: `1:Harry Potter,1:Naruto`.

### Multi-Word Tag Values

Tags with spaces work automatically:

```svelte
  <table class="syntax-table">
    <tr>
      <td class="code">fandom:Harry Potter</td>
      <td>The parser greedy-eats everything after the colon until the next keyword</td>
    </tr>
    <tr>
      <td class="code">fandom:"Harry Potter"</td>
      <td>Quoted strings also work for clarity</td>
    </tr>
  </table>
```

The parser's "greedy tokenize" mode is the secret sauce. When it sees `fandom:Harry Potter tag:Fluff`, it knows that "Harry Potter" belongs to the `fandom:` key because "tag:" is a recognized key that starts a new token. Without this greedy behavior, "Harry" would be assigned to fandom and "Potter" would be treated as bare words.

The `KNOWN_KEYS` set defines what tokens the parser recognizes:

```typescript
const KNOWN_KEYS = new Set([
  'title', 't',
  'author', 'creator', 'a',
  'fandom', 'f',
  'char', 'character', 'c',
  'rel', 'relationship', 'r',
  'tag', 'freeform',
  'words', 'w',
  'chapters', 'ch',
  'complete', 'comp',
  'site', 's',
  'sort',
  'after',
  'before',
]);
```

> **Watch Out:** If you use a keyword that isn't recognized (like `something:fancy`), the parser treats the whole thing as bare words for full-text search. Only recognized keys trigger the special parsing behavior. This means typos in keywords default to full-text search, which is usually what you want — a typo won't break your search, it'll just search for the literal text.

## Numeric Filters

For filtering by word count and chapter count:

```svelte
  <section>
    <h2>Numeric Filters</h2>
    <p>Filter by word count or chapter count using ranges.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">words:10000-50000</td>
        <td>Between 10,000 and 50,000 words</td>
      </tr>
      <tr>
        <td class="code">words:>1000</td>
        <td>More than 1,000 words</td>
      </tr>
      <tr>
        <td class="code">words:<500</td>
        <td>Fewer than 500 words</td>
      </tr>
      <tr>
        <td class="code">chapters:>10</td>
        <td>More than 10 chapters</td>
      </tr>
      <tr>
        <td class="code">chapters:3-20</td>
        <td>Between 3 and 20 chapters</td>
      </tr>
    </table>
  </section>
```

The numeric parser supports three formats:

1. **Range**: `words:10000-50000` → sets both `min_words` and `max_words`. The dash separates the two values.
2. **Greater than**: `words:>1000` → sets `min_words`. The `>` symbol is intuitive — "more than."
3. **Less than**: `words:<500` → sets `max_words`. The `<` symbol means "fewer than."

The parser handles edge cases gracefully. If you type `words:abc`, it returns `{min: null, max: null}` — no constraint. If you type `words:5000-`, it only sets the minimum. These partial ranges are useful for "at least X words" or "at most Y chapters" queries.

The `parseRange` function in the syntax parser handles all three formats:

```typescript
function parseRange(value: string): { min: number | null; max: number | null } {
  if (value.startsWith('>')) {
    const n = Number(value.slice(1));
    return { min: isNaN(n) ? null : n, max: null };
  }
  if (value.startsWith('<')) {
    const n = Number(value.slice(1));
    return { min: null, max: isNaN(n) ? null : n };
  }
  const parts = value.split('-');
  if (parts.length === 2) {
    const min = Number(parts[0]);
    const max = Number(parts[1]);
    return {
      min: isNaN(min) ? null : min,
      max: isNaN(max) ? null : max,
    };
  }
  const n = Number(value);
  return { min: isNaN(n) ? null : n, max: null };
}
```

> **Try It Yourself:** Try `fandom:Harry Potter words:>50000 complete:true` — this finds long, completed Harry Potter fics. Or try `tag:Fluff words:<1000` for short fluff one-shots. The numeric filters are great for narrowing down fic length to match your available reading time.

## Status and Site

Filter by completion status and source site:

```svelte
  <section>
    <h2>Status & Site</h2>

    <table class="syntax-table">
      <tr>
        <td class="code">complete:true</td>
        <td>Only completed fics</td>
      </tr>
      <tr>
        <td class="code">complete:false</td>
        <td>Only works in progress</td>
      </tr>
      <tr>
        <td class="code">site:ao3</td>
        <td>Only from Archive of Our Own</td>
      </tr>
      <tr>
        <td class="code">site:ffn</td>
        <td>Only from FanFiction.net</td>
      </tr>
      <tr>
        <td class="code">site:sb</td>
        <td>Only from SpaceBattles</td>
      </tr>
      <tr>
        <td class="code">site:sv</td>
        <td>Only from SufficientVelocity</td>
      </tr>
    </table>
  </section>
```

The site shortcuts are mapped in the `SITE_MAP` object in the syntax parser:

```typescript
const SITE_MAP: Record<string, string> = {
  ao3: 'archiveofourown.org',
  ff: 'fanfiction.net',
  ffn: 'fanfiction.net',
  fp: 'fictionpress.com',
  sb: 'forums.spacebattles.com',
  sv: 'forums.sufficientvelocity.com',
};
```

So `site:ao3` becomes `source=archiveofourown.org` in the filters. The backend then queries only fics from that source. Note that both `ff` and `ffn` map to FanFiction.net — two shortcuts for the same site. This flexibility means users don't have to remember the "right" abbreviation.

### The complete Keyword

The `complete` keyword accepts several truthy/falsy values:

```typescript
if (value === 'true' || value === 'yes' || value === '1') {
  filters.complete = true;
} else if (value === 'false' || value === 'no' || value === '0') {
  filters.complete = false;
}
```

So `complete:yes`, `complete:1`, and `complete:true` all work the same way. This flexibility means users don't have to remember the exact spelling — you can type whatever feels natural. The parser is forgiving, which is important for a syntax that users are learning by trial and error.

> **Try It Yourself:** Try `site:ao3 complete:true sort:kudos` — this finds the most-kudoed completed fics on AO3. Or try `site:ffn words:>100000` for epic-length fics on FanFiction.net. The site filter is great for users who prefer one platform over another.

## Sorting and Dates

Control sort order and date ranges:

```svelte
  <section>
    <h2>Sorting & Dates</h2>

    <table class="syntax-table">
      <tr>
        <td class="code">sort:updated</td>
        <td>Sort by date last updated</td>
      </tr>
      <tr>
        <td class="code">sort:created</td>
        <td>Sort by date published</td>
      </tr>
      <tr>
        <td class="code">sort:words</td>
        <td>Sort by word count</td>
      </tr>
      <tr>
        <td class="code">sort:kudos</td>
        <td>Sort by kudos count</td>
      </tr>
      <tr>
        <td class="code">after:2024-01-01</td>
        <td>Only fics updated after Jan 1, 2024</td>
      </tr>
      <tr>
        <td class="code">before:2023-12-31</td>
        <td>Only fics updated before Dec 31, 2023</td>
      </tr>
    </table>
  </section>
```

The date parser is flexible about formats:

```typescript
function normalizeDate(value: string, bound: 'start' | 'end'): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}T00:00:00Z` : `${value}T23:59:59Z`;
  }
  if (/^\d{4}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}-01T00:00:00Z` : `${value}-28T23:59:59Z`;
  }
  if (/^\d{4}$/.test(value)) {
    return bound === 'start' ? `${value}-01-01T00:00:00Z` : `${value}-12-31T23:59:59Z`;
  }
  return value;
}
```

You can use `YYYY-MM-DD`, `YYYY-MM`, or just `YYYY`. The parser automatically expands partial dates to full timestamps. For `before:`, it uses the end of the period (23:59:59), and for `after:`, it uses the start (00:00:00).

This means:
- `after:2024` → "after January 1, 2024 at midnight"
- `before:2023-12` → "before December 28, 2023 at 11:59 PM"
- `after:2024-06-15` → "after June 15, 2024 at midnight"

The flexibility is intentional — users think in different levels of precision. Some want "fics from this year," others want "fics from this specific week." The parser accommodates all of them.

> **Try It Yourself:** Try `fandom:Harry Potter after:2023-01-01 before:2024-01-01 sort:updated` — this finds Harry Potter fics updated during 2023, sorted by most recently updated. Great for finding recent activity in an old fandom. Or try `after:2024` with no other filters to see everything updated this year.

## Examples

The guide includes complex multi-filter examples that show how to combine multiple keywords:

```svelte
  <section>
    <h2>Examples</h2>
    <p>Here are some real-world search queries that combine multiple filters.</p>

    <div class="example">
      <code>fandom:Harry Potter tag:Fluff words:>10000 complete:true</code>
      <p>Long, completed Harry Potter fluff fics</p>
    </div>

    <div class="example">
      <code>-fandom:Naruto -tag:Angst words:5000-20000 site:ao3</code>
      <p>Mid-length AO3 fics, not Naruto, not angst</p>
    </div>

    <div class="example">
      <code>author:cleo fandom:Harry Potter sort:kudos</code>
      <p>All fics by author "cleo" in Harry Potter, sorted by kudos</p>
    </div>

    <div class="example">
      <code>rel:Harry/Draco tag:Slow Burn chapters:>20 after:2022-01-01</code>
      <p>Long Harry/Draco slow burns updated since 2022</p>
    </div>
  </section>
```

Each example shows a complete query and explains what it finds. These are real queries that users would actually type — not toy examples. They demonstrate:

1. **Combining tags with numeric filters** — `tag:Fluff words:>10000`
2. **Using exclusions** — `-fandom:Naruto -tag:Angst`
3. **Sorting by popularity** — `sort:kudos`
4. **Using short aliases** — `rel:`, `ch:`
5. **Combining date filters with other criteria** — `after:2022-01-01`

The examples are carefully chosen to cover different use cases: finding completed fics, filtering by site, sorting by popularity, and combining multiple criteria.

## Short Keys

For the impatient (or the frequent user), the guide lists all the shorthand aliases:

```svelte
  <section>
    <h2>Short Keys</h2>
    <p>Every keyword has a short alias. Use whichever you prefer.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">t:</td>
        <td>Alias for <code>title:</code></td>
      </tr>
      <tr>
        <td class="code">a:</td>
        <td>Alias for <code>author:</code> (also <code>creator:</code>)</td>
      </tr>
      <tr>
        <td class="code">f:</td>
        <td>Alias for <code>fandom:</code></td>
      </tr>
      <tr>
        <td class="code">c:</td>
        <td>Alias for <code>char:</code> (also <code>character:</code>)</td>
      </tr>
      <tr>
        <td class="code">r:</td>
        <td>Alias for <code>rel:</code> (also <code>relationship:</code>)</td>
      </tr>
      <tr>
        <td class="code">w:</td>
        <td>Alias for <code>words:</code></td>
      </tr>
      <tr>
        <td class="code">ch:</td>
        <td>Alias for <code>chapters:</code></td>
      </tr>
      <tr>
        <td class="code">s:</td>
        <td>Alias for <code>site:</code></td>
      </tr>
    </table>
  </section>
```

Both `title:` and `t:` are recognized. Both `fandom:` and `f:` work. This means power users can type ultra-short queries like:

```
f:HP w:>10k ch:true s:ao3
```

Which the parser handles just as well as the verbose version.

The short keys are implemented in the `KNOWN_KEYS` set in the syntax parser. Both the long and short forms are members of the set, so the parser treats them identically. You don't need to remember which aliases exist — if you type a single letter followed by a colon, the parser checks if it's a known key.

> **Try It Yourself:** Try typing the same search using both long and short forms. For example: `fandom:Harry Potter words:>10000` vs `f:Harry Potter w:>10000`. Both should return the same results. Pick whichever feels more natural to you. Most users start with the long forms and gradually switch to short keys as they get comfortable.

## Common Mistakes

Learning a new syntax takes practice. Here are the most common mistakes users make, and how to avoid them:

### 1. Forgetting the Colon

**Wrong:** `fandom Harry Potter`
**Right:** `fandom:Harry Potter`

Without the colon, `fandom` is treated as a bare word — the parser searches for the text "fandom" in titles and summaries. The colon is what tells the parser "this is a keyword, not a search term."

### 2. Using the Wrong Keyword

**Wrong:** `relationship:Harry/Draco`
**Right:** `rel:Harry/Draco` (or `relationship:Harry/Draco`)

Both `rel:` and `relationship:` work — they're aliases. But `relationship:` is the long form. If you type `ship:Harry/Draco`, the parser doesn't recognize `ship` as a keyword, so it treats the whole thing as a bare word search.

### 3. Case Sensitivity in Values

**Wrong:** `fandom:harry potter` (if the tag is stored as "Harry Potter")
**Right:** `fandom:Harry Potter`

Keywords are case-insensitive (`fandom:` works the same as `FANDOM:`), but tag values are case-sensitive. The database stores tags with their original capitalization. If you're not sure of the exact spelling, try the dropdown filters first to see what tags exist.

### 4. Confusing `words:` with `chapters:`

**Wrong:** `chapters:>50000` (trying to filter by word count)
**Right:** `words:>50000`

`chapters:` filters by chapter count, not word count. A 50,000-word fic might have 10 chapters, not 50,000. Use `words:` for word count and `chapters:` for chapter count.

### 5. Missing Quotes for Multi-Word Values

**Wrong:** `fandom:Harry Potter tag:Fluff` (this actually works due to greedy parsing!)
**Right:** `fandom:"Harry Potter" tag:Fluff` (also works, and is clearer)

The greedy parser handles unquoted multi-word values correctly, so this isn't really a mistake — but quotes make your intent clearer. Use them when a tag name might be confused with a keyword.

### 6. Overloading the Query

**Wrong:** `fandom:Harry Potter char:Harry rel:Harry/Ginny tag:Fluff words:>10000 complete:true site:ao3 sort:kudos after:2023-01-01`
**Right:** Start simple, add filters gradually.

Complex queries are powerful, but they can return zero results if the combination is too restrictive. Start with one or two filters, see what comes back, and add more if needed. The search engine is fast — you can iterate quickly.

### 7. Excluding Everything

**Wrong:** `-tag:Angst -tag:Major Character Death -tag:Character Death -tag:Slow Burn -tag:Hurt/Comfort`
**Right:** Exclude only what you truly don't want.

Every exclusion narrows the results. If you exclude too many tags, you might filter out fics you'd actually enjoy. Be selective with exclusions — use them for hard "no" preferences, not soft "meh" ones.

## Linking from the Search Bar

The syntax guide is linked from the search page's query row:

```svelte
<a class="syntax-link muted" href="/search/syntax" target="_blank">Syntax guide</a>
```

The `target="_blank"` opens it in a new tab. This is intentional — the user shouldn't lose their current search when they check the syntax. The guide is a reference, not a destination.

In the future, we could add inline tooltips to the search input that show syntax hints as the user types. For example, typing `fandom:` could show a small popup: "Enter a fandom name. Example: fandom:Harry Potter." But for now, the static guide page is the simplest and most reliable approach. It loads instantly, works offline, and can be updated without touching the search component.

## Utility Functions

The search results display uses several utility functions from `$lib/util.ts`. Let's look at the ones that make the results readable:

### formatWords

```typescript
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}
```

Converts a number like `1234567` into `"1,234,567"`. This makes large word counts scannable — instead of counting digits, you instantly see "1.2 million." The `'en-US'` locale ensures commas are used as thousand separators.

### relativeTime

```typescript
export function relativeTime(iso: string): string {
  if (!iso) return '';
  const then = new Date(iso).getTime();
  if (isNaN(then)) return '';
  const diff = Date.now() - then;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return 'less than a minute ago';
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} minute${min > 1 ? 's' : ''} ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} hour${hr > 1 ? 's' : ''} ago`;
  const day = Math.floor(hr / 24);
  return `${day} day${day > 1 ? 's' : ''} ago`;
}
```

Converts an ISO timestamp into a human-readable relative time: "3 days ago," "2 hours ago," "less than a minute ago." This is much more intuitive than showing "2024-01-15T14:30:00Z" — users care about recency, not exact dates.

### detectSite

```typescript
export function detectSite(url: string): string {
  if (url.includes('archiveofourown.org')) return 'AO3';
  if (url.includes('fanfiction.net')) return 'FanFiction.net';
  if (url.includes('fictionpress.com')) return 'FictionPress';
  if (url.includes('xenforo') || url.includes('spacebattles') || url.includes('sufficientvelocity'))
    return 'Forum';
  return 'Unknown';
}
```

Inspects the fic's URL to determine which platform it's from. The result is displayed as a small badge next to the author name. This helps users quickly identify which site a fic is from without clicking the link.

### stripHtml

```typescript
export function stripHtml(html: string): string {
  if (!html) return '';
  return html
    .replace(/<br\s*\/?>/gi, ' ')
    .replace(/<\/(p|div)>/gi, ' ')
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')
    .trim();
}
```

Fic summaries often contain HTML tags (`<p>`, `<br>`, `<b>`, etc.). This function strips them all, converting `<br>` to spaces and decoding HTML entities. The result is clean plain text that displays correctly in the result card. We then `slice(0, 250)` to show only the first 250 characters — enough to give a taste of the summary without overwhelming the card.

## What You Learned

In this part, you learned how to:

1. **Build a SvelteKit layout** with sticky navigation, tab switching, and responsive design.
2. **Create a search page** that reads URL parameters, parses query syntax, and displays paginated results.
3. **Implement filter controls** with dropdown menus, number inputs, date pickers, and tag fields.
4. **Write a query syntax** that converts human-readable keywords into structured search filters.
5. **Design result cards** with title, author, tags, metadata, and action buttons.
6. **Handle responsive layouts** that adapt from desktop to mobile with CSS Grid and Flexbox.
7. **Write a syntax guide** that teaches users a powerful query language.
8. **Use utility functions** to format numbers, dates, and HTML content.

These skills transfer to any web application with search functionality. The patterns you learned — data-driven UI, URL-synced state, syntax parsing, and responsive design — are universal.

## Practice: 5 Different Search Queries

Here are five queries to try, ranging from simple to complex. Type them in the search bar and press Enter to see the results. Each query demonstrates different features of the syntax.

1. **Simple keyword search:**
   ```
   time travel
   ```
   Finds fics mentioning "time travel" in any field. This is the simplest possible query — just bare words, no syntax required. The search engine looks for these words in titles, summaries, and tags.

2. **Fandom + completion:**
   ```
   fandom:Lord of the Rings complete:true
   ```
   Completed Lord of the Rings fics. The fandom filter ensures you only see LotR content, and the completion filter eliminates works in progress. Great for when you want a finished story.

3. **Author + word count:**
   ```
   author:snowqueens ibigdragon words:50000-200000
   ```
   Long fics by a specific author. The word count range filters for epic-length works — perfect for when you want a long reading session and don't want to run out of story.

4. **Exclusion + sorting:**
   ```
   -fandom:My Hero Academia -tag:Major Character Death sort:kudos
   ```
   Top-kudos fics excluding MHA and major character death. The exclusions remove content you don't want, and `sort:kudos` brings the most popular fics to the top.

5. **The kitchen sink:**
   ```
   fandom:Harry Potter rel:Harry/Hermione words:>20000 complete:true -tag:Angst -tag:Infidelity after:2022-01-01 sort:updated
   ```
   Long, completed Harry/Hermione fics from 2022 onwards, no angst or infidelity, sorted by most recently updated. This query combines eight different filters in a single line — try building the same search with dropdown menus!

## Bonus Challenges

Ready for more? Try these advanced challenges:

1. **Find fics shorter than 1000 words in a specific fandom, sorted by kudos:**
   ```
   fandom:Naruto words:<1000 sort:kudos
   ```
   This finds the most popular short fics — perfect for a quick read during a break.

2. **Find recently updated fics from a specific author on a specific site:**
   ```
   author:snowqueens ibigdragon site:ao3 after:2024-01-01
   ```
   This finds fics by a specific author updated this year on AO3. Great for following your favorite writers.

3. **Find completed fics with a specific relationship, excluding certain tags:**
   ```
   rel:Drarry complete:true -tag:Angst -tag:Major Character Death
   ```
   This finds happy Drarry fics — completed, no angst, no character deaths. Perfect for when you want a feel-good read.

4. **Find long fics in a fandom you don't usually read, sorted by word count:**
   ```
   fandom:My Hero Academia words:>100000 sort:words
   ```
   This finds the longest MHA fics. Sometimes you want to dive deep into a new fandom with an epic story.

5. **Find fics updated this week in any fandom, sorted by recent updates:**
   ```
   after:2024-01-01 sort:updated
   ```
   Without any fandom filter, this shows recently updated fics across all fandoms. Great for discovering new content.

These challenges help you practice combining different filters. The more you experiment, the more intuitive the syntax becomes.

# Quick Reference

Here's a condensed reference card for everything covered in this part. Pin it to your wall, tape it to your monitor, or just keep it in your back pocket.

## Layout Files

| File | Purpose |
|------|---------|
| `src/routes/+layout.svelte` | App shell: topbar, tabs, footer |
| `src/routes/+layout.ts` | SPA mode config (ssr=false) |
| `src/routes/+page.svelte` | Root page (intentionally empty) |
| `src/routes/+page.ts` | Root page loader (if any) |

## Search Files

| File | Purpose |
|------|---------|
| `src/routes/search/+page.svelte` | Search page UI (540 lines) |
| `src/routes/search/+page.ts` | Page loader (reads URL params) |
| `src/routes/search/syntax/+page.svelte` | Syntax guide (static) |
| `src/lib/api/search.ts` | Search API client + types |
| `src/lib/search/syntax.ts` | Query syntax parser |
| `src/lib/util.ts` | Formatting utilities |

## Syntax Quick Reference

| Keyword | Short | Example | Effect |
|---------|-------|---------|--------|
| `title:` | `t:` | `t:Dragon` | Search title only |
| `author:` | `a:` | `a:cleo` | Search author only |
| `fandom:` | `f:` | `f:HP` | Include fandom tag |
| `char:` | `c:` | `c:Harry` | Include character tag |
| `rel:` | `r:` | `r:Harry/Hermione` | Include relationship tag |
| `tag:` | — | `tag:Fluff` | Include freeform tag |
| `-fandom:` | `-f:` | `-f:Naruto` | Exclude fandom tag |
| `-tag:` | — | `-tag:Angst` | Exclude freeform tag |
| `words:` | `w:` | `w:>10000` | Word count range |
| `chapters:` | `ch:` | `ch:3-20` | Chapter count range |
| `complete:` | — | `comp:true` | Completion status |
| `site:` | `s:` | `s:ao3` | Source site |
| `sort:` | — | `sort:kudos` | Sort order |
| `after:` | — | `after:2024-01-01` | Date range start |
| `before:` | — | `before:2023-12-31` | Date range end |

## Key CSS Classes

| Class | Element | Purpose |
|-------|---------|---------|
| `.topbar` | `<header>` | Sticky navigation bar |
| `.tab-bar` | `<nav>` | Tab button container |
| `.tab` | `<button>` | Individual tab button |
| `.tab.active` | `<button>` | Currently selected tab |
| `.search-area` | `<div>` | Right-side search field |
| `.nav-search` | `<input>` | Navbar search input |
| `.filter-grid` | `<div>` | Two-column filter layout |
| `.filter-item` | `<label>` | Individual filter control |
| `.result-card` | `<div>` | Search result card |
| `.tag-pill` | `<span>` | Tag badge in results |
| `.pagination` | `<div>` | Page navigation controls |

## Search API

The search API endpoint is `GET /api/v0/search`. It accepts query parameters matching the `SearchFilters` fields and returns a `SearchResponse` JSON object.

**Example request:**
```
GET /api/v0/search?q=Harry+Potter&fandom=1:Harry+Potter&words_min=10000&complete=true&page=1
```

**Example response:**
```json
{
  "total": 42,
  "page": 1,
  "per_page": 20,
  "results": [
    {
      "url_id": "ao3-12345678",
      "title": "Harry Potter and the Methods of Rationality",
      "author": "Less Wrong",
      "source": "https://archiveofourown.org/works/10000000",
      "words": 661000,
      "chapters": 122,
      "status": "Complete",
      "description": "A rationalist reimagining of Harry Potter...",
      "updated": "2024-01-15T14:30:00Z",
      "rank": 1.2,
      "tags": [
        { "name": "Harry Potter - J.K. Rowling", "type": "Fandom", "type_id": 1, "score": 0.95 }
      ],
      "total_freeform": 15
    }
  ]
}
```

The `rank` field is the Tantivy relevance score. Lower values mean better matches. A rank of `1.0` means the fic matched very strongly; `5.0` means a weaker match.

## Common Patterns

Throughout this part, we used several patterns that appear in many web applications:

1. **Data-driven UI** — instead of writing separate markup for each tab, we define tabs as data and loop over them. This pattern appears in navigation bars, dropdown menus, and form builders.

2. **URL-synced state** — we update the URL when the search changes, so searches can be bookmarked and shared. This pattern is essential for any page with filterable content.

3. **Syntax parsing** — we convert a human-readable query string into structured data. This pattern appears in SQL query builders, cron job schedulers, and configuration languages.

4. **Responsive design** — we use CSS media queries to adapt layouts for different screen sizes. This pattern is mandatory for any public-facing web application.

5. **Error handling** — we wrap API calls in try/catch and display user-friendly error messages. This pattern prevents confusing blank screens when things go wrong.

# Summary

In this part, we built the navigation and search systems that make FicHub feel like a real application:

- **The layout** (`+layout.svelte`) provides the app shell — a sticky topbar with brand, tabs, and search field, plus a footer. Tabs switch between Download, Recommendations, and Suggestions. The search field navigates to the advanced search page. The layout is defined once and wraps every page, ensuring consistent navigation and appearance. The flex layout with `min-height: 100vh` and `flex: 1` on main creates a sticky footer that stays at the bottom.

- **The search page** (`/search`) is a full-featured search interface with query syntax parsing, advanced filter controls, paginated results, and four search tabs. The `doSearch()` function parses syntax, merges with form filters, syncs the URL, and calls the API. The page loader reads URL parameters so searches can be bookmarked and shared. The auto-search on mount pattern ensures users arriving from the navbar search see results immediately.

- **The filter system** provides dropdown menus for completion status, site, sort order, word count range, chapter range, date range, and tag inclusion/exclusion. The Work Search tab has a two-column grid of these filters. People Search, Bookmark Search, and Tag Search are placeholders for future development, with syntax hints guiding users to the query syntax. The merge logic in `doSearch()` ensures form fields override syntax values when both are set.

- **The syntax guide** (`/search/syntax`) teaches users a powerful query language with keywords like `fandom:`, `words:`, `complete:`, and `sort:`. It supports short aliases (`f:`, `w:`, `s:`), exclusion with `-`, and flexible date formats. The guide turns curious users into power users, and the short keys make complex searches fast to type. The consistent table format makes it easy to scan and reference.

Together, these components create a search experience that's both accessible (point-and-click filters) and powerful (query syntax). Users can start simple and gradually learn the syntax as they need more precision. The layout provides a consistent shell, the search page provides the tools, and the syntax guide provides the knowledge. That's a complete search experience.

The patterns we covered in this part — data-driven UI, URL-synced state, syntax parsing, responsive design, and error handling — are not specific to FicHub. They appear in every well-built web application. Whether you're building an e-commerce site with product filters, a documentation site with search, or a dashboard with data tables, these patterns will serve you well.

In the next part, we'll explore how FicHub handles the actual download process — taking a fic URL, fetching its content, and converting it to an ebook format. But that's a story for another chapter.

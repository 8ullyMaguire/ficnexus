# Part 2: Shell Components

## Chapter 5: ArchiveLayout — The Wrapper

Every great interface needs a frame. In the Archive mode, that frame is `ArchiveLayout.svelte` — a 63-line component that lives at `frontend/src/lib/ui/archive/ArchiveLayout.svelte`. It's deceptively simple: it wraps the header, renders the page content, and drops in the footer. But it also does something subtle and important — it's the *only* Archive component that bootstraps three shared systems: themes, i18n, and auth.

Here's the full component:

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';
  import ArchiveHeader from './ArchiveHeader.svelte';
  import ArchiveFooter from './ArchiveFooter.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import { onMount } from 'svelte';
  import { applyTheme, loadTheme } from '$lib/themes/apply.js';
  import { initI18n } from '$lib/i18n/index.svelte';
  import { auth } from '$lib/stores/auth.svelte';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    applyTheme(loadTheme());
    initI18n({ userLocale: auth.user?.locale ?? null });
    auth.init();
  });
</script>

<div class="archive-shell">
  <ArchiveHeader />
  <main class="archive-main">
    {@render children()}
  </main>
  <ArchiveFooter />
</div>

<HelpModal />
<CommandPalette />

<style>
  .archive-shell {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    font-family: Georgia, 'Times New Roman', serif;
  }

  .archive-main {
    flex: 1;
    max-width: 1100px;
    margin: 0 auto;
    width: 100%;
    padding: 1rem;
  }

  :global(.archive-shell a) {
    color: var(--archive-link, #990000);
  }

  :global(.archive-shell a:visited) {
    color: var(--archive-link-visited, #660066);
  }

  :global(.archive-shell a:hover) {
    text-decoration: underline;
  }
</style>
```

### The onMount Bootstrap Sequence

The `onMount` callback in `ArchiveLayout` is the closest thing the Archive shell has to an "entry point." It runs once when the layout first renders, and it initializes three systems in sequence:

1. **`applyTheme(loadTheme())`** — reads the saved theme from localStorage and pushes the CSS custom properties onto `document.documentElement`. This is where `--archive-bg`, `--archive-link`, and friends get their values. Without this, components would fall back to their hard-coded hex colors (like `#990000` for maroon).
2. **`initI18n({ userLocale: auth.user?.locale ?? null })`** — resolves the initial locale using the priority chain: localStorage → user profile → browser language. See Chapter 6's sidebar note for the full cascade.
3. **`auth.init()`** — calls `GET /api/auth/me` to check if there's a valid session. If the network fails, it falls back to a cached user in localStorage.

> **⚠️ Watch Out**
>
> These three calls happen in a specific order for a reason. `applyTheme` must run before any component renders its styles, `initI18n` must run before any `t()` calls resolve translated strings, and `auth.init()` must complete before the header can show the right "Hi, username!" greeting. Since they're all inside `onMount`, the initial render happens with default values, then Svelte re-renders once these systems resolve. That's fine for a static site — users won't notice the flash.

### The `{@render children()}` Pattern

ArchiveLayout uses Svelte 5's snippet-based slot system. The `children` prop is typed as `Snippet`, and the markup renders it with `{@render children()}`:

```svelte
<main class="archive-main">
  {@render children()}
</main>
```

This is the Svelte 5 replacement for `<slot />`. The root layout (`src/routes/+layout.svelte`) passes page content into this slot:

```svelte
{#if uiMode === 'archive'}
  <ArchiveLayout>
    {@render children()}
  </ArchiveLayout>
{:else}
  <!-- Modern mode -->
{/if}
```

The `children` snippet is whatever the matched route page component renders. For `/`, that's `ArchiveHome`. For `/search`, that's `ArchiveSearch`. ArchiveLayout doesn't know or care what it is — it just provides the header, main container, and footer scaffolding.

> **💡 Key Concept — The Shell Pattern**
>
> ArchiveLayout is a "shell" component: it provides chrome (header, footer), layout constraints (max-width, padding, flexbox), and global style scoping — then gets out of the way. This is the same pattern AO3 uses (every page shares the same two-row header and maroon footer). By centralizing these concerns in one place, individual page components like `ArchiveHome` or `ArchiveWork` can focus purely on their content.

### Global Link Styling via `:global()`

Notice the `:global(.archive-shell a)` selectors. Svelte scopes component styles by default, but `:global()` punches through that scoping to apply styles to *all* descendant links, regardless of which component renders them:

```css
:global(.archive-shell a) {
  color: var(--archive-link, #990000);
}

:global(.archive-shell a:visited) {
  color: var(--archive-link-visited, #660066);
}

:global(.archive-shell a:hover) {
  text-decoration: underline;
}
```

This is how every link in the entire Archive interface gets consistent maroon coloring without every component (WorkBlurb, TagSoup, ArchiveNavLink, etc.) needing its own link CSS. The `visited` selector uses the purple `#660066` — a direct AO3 nod that tells users "you've been here before" in a visually distinct way.

### Re-homing CommandPalette and HelpModal

ArchiveLayout also renders two shared modal components directly in its template:

```svelte
<HelpModal />
<CommandPalette />
```

These aren't Archive-specific — they're imported from `$lib/components/`, the shared component directory used by both modes. `CommandPalette.svelte` provides the Ctrl+K quick-jumper (see `frontend/src/lib/components/CommandPalette.svelte`), and `HelpModal.svelte` renders the in-app documentation modal (driven by `$lib/stores/doc-help.svelte`). Both are always-mounted singletons: they render conditionally based on internal state (`open` for CommandPalette, `docHelpState.active` for HelpModal), so including them in the layout is free — no DOM overhead until the user activates them.

> **🧪 Try It Yourself**
>
> Open the Archive mode in your browser and press `Ctrl+K` (or `Cmd+K` on Mac). The command palette slides into view. Now click a Docs entry — the HelpModal opens with the requested documentation section. Both of these are hosted by ArchiveLayout, invisible until you summon them.

---

## Chapter 6: ArchiveHeader — The AO3 Parity Header

If ArchiveLayout is the frame, `ArchiveHeader.svelte` is the face. At 347 lines (in `frontend/src/lib/ui/archive/ArchiveHeader.svelte`), it implements AO3's signature two-row header: a top row with the logo and user menu, and a red navigation bar below it. There are no emojis, no rounded corners, no shadows — just dense, functional links.

### The Structure

The header is split into two rows:

```svelte
<header class="archive-header">
  <!-- ROW 1: Logo + user area -->
  <div class="archive-top-row">
    <div class="archive-top-inner">
      <a href="/" class="archive-brand" aria-label="FicHub home">
        FicHub<sup class="archive-brand-sup">archive</sup>
      </a>
      <div class="archive-top-right">
        {#if auth.isLoggedIn}
          <!-- logged-in: greeting, Post, notifications, Log Out -->
        {:else}
          <!-- logged-out: Log In -->
        {/if}
      </div>
    </div>
  </div>

  <!-- ROW 2: Red navbar -->
  <nav class="archive-navbar" aria-label="Primary navigation">
    ...
  </nav>
</header>
```

### Row 1: Logo and User State

The left side of Row 1 is the brand — a simple link with styled typography:

```svelte
<a href="/" class="archive-brand" aria-label="FicHub home">
  FicHub<sup class="archive-brand-sup">archive</sup>
</a>
```

The CSS is deliberately AO3-like:

```css
.archive-brand {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 1.55rem;
  font-weight: 700;
  color: #990000;
  text-decoration: none;
  line-height: 1;
  white-space: nowrap;
}

.archive-brand:hover {
  color: #660066;
}

.archive-brand-sup {
  font-size: 0.45em;
  font-weight: 400;
  vertical-align: super;
  margin-left: 0.15em;
  color: var(--archive-muted, #666666);
}
```

The `<sup>` element gives "archive" the small superscript treatment, turning "FicHub" + "archive" into "FicHub<sup>archive</sup>". On hover, the brand color darkens from `#990000` to `#660066` (the visited-link purple).

The right side of Row 1 is reactive — it reads from the `auth` store:

```svelte
{#if auth.isLoggedIn}
  <span class="archive-hello">Hi, {auth.username ?? 'there'}!</span>
  <a class="archive-top-link" href="/requests/new">Post</a>
  <NotificationBell />
  <button
    class="archive-logout-btn"
    type="button"
    onclick={() => auth.handleLogout()}
  >
    Log Out
  </button>
{:else}
  <a class="archive-top-link" href="/login">Log In</a>
{/if}
```

The auth store (`frontend/src/lib/stores/auth.svelte.ts`) is a Svelte 5 class-based store using `$state` runes. When `auth.init()` resolves (called from ArchiveLayout's `onMount`), the `user` property updates, `isLoggedIn` flips to `true`, and the header re-renders instantly — no page reload needed.

The `NotificationBell` component (`frontend/src/lib/components/NotificationBell.svelte`) is a shared Modern-mode component that polls `/api/notifications/unread-count` every 60 seconds to show a badge. It's imported directly into the header because it's useful in both modes.

### Row 2: The Red Navbar

Row 2 is the iconic red bar. It uses `--archive-accent-line` as its background:

```css
.archive-navbar {
  background: var(--archive-accent-line, #990000);
}
```

The navbar has two clusters — left and right:

**Left cluster** (primary navigation):
- Fandoms → links to `/fandoms`
- Browse → links to `/tags`
- Search ▾ (dropdown) → links to Works, Bookmarks, Tags, People
- About ▾ (dropdown) → links to Docs, Roadmap, Modlog, and an inert "Tropes" item

**Right cluster**:
- Requests → `/requests`
- Forum → `/forum`
- Ask → `/ask`
- Inline search input → submits to `/search?q=...`

### Keyboard-Accessible Dropdowns with `<details>`

AO3's dropdowns use JavaScript. FicHub's Archive mode uses native HTML `<details>` and `<summary>` elements — no JS required, fully keyboard-navigable, and screen-reader friendly:

```svelte
<details class="archive-dropdown">
  <summary class="archive-nav-link archive-dropdown-trigger">Search &#9662;</summary>
  <div class="archive-dropdown-menu" role="menu">
    <a class="archive-dd-item" href="/search" role="menuitem">Works</a>
    <a class="archive-dd-item" href="/bookmarks" role="menuitem">Bookmarks</a>
    <a class="archive-dd-item" href="/tags" role="menuitem">Tags</a>
    <a class="archive-dd-item" href="/authors" role="menuitem">People</a>
  </div>
</details>
```

The `&#9662;` is a down-chevron character entity. The CSS hides the default `<summary>` marker:

```css
.archive-dropdown-trigger::-webkit-details-marker {
  display: none;
}

.archive-dropdown-trigger::marker {
  content: '';
}
```

And styles the trigger to look like a regular nav link:

```css
.archive-dropdown-trigger {
  cursor: pointer;
  list-style: none;
  user-select: none;
}

.archive-dropdown[open] > .archive-dropdown-trigger {
  background: rgba(255, 255, 255, 0.15);
}
```

The dropdown menu itself is positioned absolutely:

```css
.archive-dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 160px;
  background: var(--archive-bg, #ffffff);
  border: 1px solid var(--archive-border, #dddddd);
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
  padding: 0.25rem 0;
  z-index: 80;
}
```

> **⚠️ Watch Out**
>
> Using `<details>` for dropdowns means the open/close state is managed by the browser, not by Svelte reactivity. This is actually a *feature* — it means the dropdown stays open during keyboard navigation and closes automatically when focus leaves. If you need to control the open state programmatically (e.g., close all dropdowns when a form submits), you'd have to reach into the DOM with `details.open = false`.

### The Inert "Tropes" Item

In the About dropdown, you'll notice one item that isn't a link:

```svelte
<span class="archive-dd-item archive-dd-inert" role="menuitem" aria-disabled="true">Tropes</span>
```

It's a `<span>`, not an `<a>`, and it carries `aria-disabled="true"`. The CSS grays it out:

```css
.archive-dd-item:hover {
  background: var(--archive-bg-raised, #f5f5f5);
}

.archive-dd-inert {
  color: var(--archive-muted, #999999);
  cursor: default;
  opacity: 0.6;
}

.archive-dd-inert:hover {
  background: none;
}
```

This is a placeholder for a feature that hasn't shipped yet. AO3 has a Tropes section. FicHub's Archive mode acknowledges the gap but doesn't fake it — the item is explicitly inert.

### Inline Search in the Navbar

The search form is inline in the right cluster of the navbar:

```svelte
<form class="archive-search-form" onsubmit={handleSearch} role="search">
  <input
    class="archive-search-input"
    type="search"
    name="q"
    placeholder="Search"
    aria-label="Search FicHub"
    bind:value={searchQuery}
  />
  <button class="archive-search-btn" type="submit">Search</button>
</form>
```

The `handleSearch` function performs a full-page navigation:

```typescript
let searchQuery = $state('');

function handleSearch(e: Event) {
  e.preventDefault();
  if (searchQuery.trim()) {
    window.location.href = `/search?q=${encodeURIComponent(searchQuery.trim())}`;
  }
}
```

It uses `window.location.href` instead of SvelteKit's `goto` because a full navigation is appropriate here — the search bar in the header is a global utility, and a fresh page load ensures the search page starts clean.

### No Emojis, Period

AO3's design rule is "no emojis." The Archive header respects this. You won't find a single `🧡` or `🔍` in the template. Even the notification bell is rendered as an inline SVG, not an emoji. The only decorative characters are CSS-inserted chevrons (`&#9662;` for dropdown triggers) and the brand superscript.

### Responsive Behavior

On screens narrower than 767px, the navbar wraps its items onto multiple lines:

```css
@media (max-width: 767px) {
  .archive-navbar-inner {
    flex-wrap: wrap;
    padding: 0 0.75rem;
    gap: 0;
  }

  .archive-nav-right {
    width: 100%;
    padding-top: 0.25rem;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    margin-top: 0.25rem;
  }

  .archive-search-input {
    width: 100px;
  }
}
```

The right cluster (Requests, Forum, Ask, Search) drops below the left cluster, and the search input shrinks to 100px to fit. This is a pragmatic compromise — AO3 uses a hamburger menu on mobile, but FicHub's Archive mode keeps all links visible.

> **💡 Key Concept — Native HTML Over Framework**
>
> The dropdowns, the search form, the login/logout toggle — none of these require JavaScript to function. AO3 pioneered this "HTML-first" approach because it's robust: if your JS bundle fails to load, the nav still works. FicHub's Archive mode follows the same philosophy. The `<details>` element handles dropdowns, the `<form>` element handles search submission, and the `{#if auth.isLoggedIn}` block handles auth state — all with graceful degradation.

> **🧪 Try It Yourself**
>
> Open Archive mode on your phone (or resize your browser to under 767px). Watch how the navbar reflows: the red bar stays the same height, but the right cluster drops to a new row. Now use your keyboard to navigate — Tab gets you into the navbar, Space opens the dropdown, and you can arrow through the menu items. No JavaScript needed.

---

## Chapter 7: ArchiveFooter — The Red Footer

If the header is the face, the footer is the handshake. `ArchiveFooter.svelte` (`frontend/src/lib/ui/archive/ArchiveFooter.svelte`, 124 lines) provides a four-column link grid on the signature maroon background, plus a button to switch to Modern mode.

### The Markup

```svelte
<script lang="ts">
  import { setPref } from '$lib/prefs';

  function switchToModern() {
    setPref('uiMode', 'modern');
    window.location.reload();
  }
</script>

<footer class="archive-footer">
  <div class="archive-footer-columns">
    <!-- Column 1: Customize -->
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Customize</h4>
      <a class="archive-footer-link" href="/settings">Settings</a>
      <a class="archive-footer-link" href="/settings">Interface style</a>
    </div>

    <!-- Column 2: About -->
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">About</h4>
      <a class="archive-footer-link" href="/docs">Docs</a>
      <a class="archive-footer-link" href="/roadmap">Roadmap</a>
      <span class="archive-footer-link archive-footer-inert">Consensus</span>
    </div>

    <!-- Column 3: Contact -->
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Contact</h4>
      <a class="archive-footer-link" href="/forum">Forum</a>
      <a class="archive-footer-link" href="/requests">Requests</a>
      <a class="archive-footer-link" href="/ask">Ask</a>
    </div>

    <!-- Column 4: Development -->
    <div class="archive-footer-col">
      <h4 class="archive-footer-heading">Development</h4>
      <a class="archive-footer-link" href="/modlog">Modlog</a>
      <a class="archive-footer-link" href="/opds">OPDS</a>
      <a class="archive-footer-link" href="/feed.xml">Atom Feed</a>
      <span class="archive-footer-link archive-footer-inert">API Docs</span>
    </div>
  </div>

  <div class="archive-footer-bottom">
    <button type="button" class="archive-switch-btn" onclick={switchToModern}>
      Switch to modern interface
    </button>
  </div>
</footer>
```

### The Switch Pattern

The footer's switch button is the inverse of the one in Modern mode. Here's how it works:

```typescript
function switchToModern() {
  setPref('uiMode', 'modern');
  window.location.reload();
}
```

It calls `setPref('uiMode', 'modern')` from `frontend/src/lib/prefs.ts` (line 63), which writes the preference to the localStorage blob, then calls `window.location.reload()`. The full page reload is intentional — it ensures the root layout's `$derived` re-evaluates `getPref('uiMode')`, picks up the new value, and re-renders in Modern mode with a clean component tree and fresh CSS.

> **💡 Key Concept — Full Reload vs. Soft Navigation**
>
> You might wonder why the switch uses `window.location.reload()` instead of SvelteKit's `goto('/')`. The answer: the UI mode is decided in the root `+layout.svelte`, and the entire component tree changes between modes. A soft navigation would leave stale Archive components mounted. The full reload is the nuclear option — but it's *reliable*. AO3 does the same thing when you switch interfaces.

### The Visual Design

The footer uses `--archive-accent-line` as its background, the same variable the navbar uses:

```css
.archive-footer {
  background: var(--archive-accent-line, #990000);
  color: #fff;
  padding: 1.2rem 1rem 0.8rem;
  margin-top: auto;
}
```

The `margin-top: auto` is important — because `.archive-shell` is a flex column with `min-height: 100vh`, `margin-top: auto` pushes the footer to the bottom of the viewport regardless of content height. That's how AO3's footer always sits at the bottom even on short pages.

The four-column grid is pure CSS:

```css
.archive-footer-columns {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem 1.5rem;
  max-width: 1100px;
  margin: 0 auto;
}
```

### Column Headings and Links

Headings use the AO3 pattern of small, uppercase, letter-spaced text:

```css
.archive-footer-heading {
  font-size: 0.78em;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin: 0 0 0.3rem;
  color: #fff;
}
```

Links are styled as block-level underlines:

```css
.archive-footer-link {
  display: block;
  font-size: 0.78em;
  color: rgba(255, 255, 255, 0.85);
  text-decoration: underline;
  padding: 0.15em 0;
}

.archive-footer-link:hover {
  color: #fff;
}
```

### Inert Links

Two footer items are explicitly inert — "Consensus" under About and "API Docs" under Development. They're `<span>` elements, not `<a>` tags:

```svelte
<span class="archive-footer-link archive-footer-inert">Consensus</span>
<span class="archive-footer-link archive-footer-inert">API Docs</span>
```

The CSS makes them look like links but disables interaction:

```css
.archive-footer-inert {
  cursor: default;
  opacity: 0.5;
}

.archive-footer-inert:hover {
  color: rgba(255, 255, 255, 0.85);
}
```

They're grayed out at 50% opacity and use `cursor: default` instead of `cursor: pointer`. The hover state still lightens them to full white, but there's no link behavior.

### The Bottom Row

Below the four columns sits the "Switch to modern interface" button:

```css
.archive-footer-bottom {
  max-width: 1100px;
  margin: 0.8rem auto 0;
  padding-top: 0.6rem;
  border-top: 1px solid rgba(255, 255, 255, 0.25);
  text-align: center;
}

.archive-switch-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.8);
  font-size: 0.78em;
  text-decoration: underline;
  cursor: pointer;
  padding: 0;
  font-family: inherit;
}

.archive-switch-btn:hover {
  color: #fff;
}
```

It's a text link styled as a button — no background, no border, just underlined text that lightens on hover. This matches AO3's pattern of "switch interface" links in the footer.

### Responsive Behavior

The footer collapses columns based on screen width:

```css
@media (max-width: 600px) {
  .archive-footer-columns {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 380px) {
  .archive-footer-columns {
    grid-template-columns: 1fr;
  }
}
```

At 600px, four columns become two. At 380px (an iPhone SE width), they collapse to a single stack. The headings and links remain readable throughout.

> **🧪 Try It Yourself**
>
> Resize your browser window while viewing any Archive page. At exactly 600px, the footer jumps from four columns to two. At 380px, it becomes a single column. The red background and white text stay consistent throughout — only the grid reflows. Now click "Switch to modern interface" and watch the entire page reload in Modern mode. Your preference is saved, so a hard refresh keeps you in Modern mode.

---

## Chapter 8: ArchiveNavLink & ArchiveButton

The Archive interface has two reusable UI primitives: `ArchiveNavLink` (49 lines) for navigation links, and `ArchiveButton` (58 lines) for clickable actions. Both live in `frontend/src/lib/ui/archive/`. They look similar, they behave differently.

### ArchiveNavLink — Active State via SvelteKit's `$page` Store

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/stores';

  let {
    href,
    label = '',
    children,
  }: {
    href: string;
    label?: string;
    children?: Snippet;
  } = $props();

  let active = $derived($page.url.pathname === href || $page.url.pathname.startsWith(href + '/'));
</script>

<a class="archive-nav-link" class:active href="{href}">
  {#if children}
    {@render children()}
  {:else}
    {label}
  {/if}
</a>

<style>
  .archive-nav-link {
    display: inline-block;
    padding: 0.3em 0.7em;
    font-size: 0.9em;
    font-weight: 600;
    color: var(--archive-link, #990000);
    text-decoration: none;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
    white-space: nowrap;
  }

  .archive-nav-link:hover {
    color: var(--archive-link-visited, #660066);
    text-decoration: underline;
  }

  .archive-nav-link.active {
    color: var(--archive-text, #2a2a2a);
    border-bottom-color: var(--archive-accent-line, #990000);
    text-decoration: none;
  }
</style>
```

#### The `$derived` Active Detection

The active state uses Svelte 5's `$derived` rune with SvelteKit's `$page` store:

```typescript
let active = $derived(
  $page.url.pathname === href
  || $page.url.pathname.startsWith(href + '/')
);
```

This creates a reactive value that recalculates whenever `$page` changes (i.e., on navigation). The logic is intentionally inclusive: if your `href` is `/fandoms`, the link becomes active on both `/fandoms` and `/fandoms/123` (via the `startsWith(href + '/')` check). This means a link to `/search` in the navbar won't show as active when you're on `/search?tab=filters`, but it *will* stay active on `/search/works`.

The `class:active` shorthand applies the `active` class to the `<a>` element when the derived value is truthy. When active, the link gets:

- `color: var(--archive-text, #2a2a2a)` — switches from maroon to the body text color
- `border-bottom-color: var(--archive-accent-line, #990000)` — a 2px maroon underline appears
- `text-decoration: none` — kills the hover underline to avoid visual clutter

> **💡 Key Concept — Reactive Class Binding**
>
> `class:active` is Svelte's shorthand for `class:active={active}`. When `active` is `true`, Svelte adds the `active` class to the element; when `false`, it removes it. The reactivity flows: `$page` changes → `$derived` re-evaluates → `active` flips → DOM class updates. No manual DOM manipulation, no event listeners, no manual cleanup. This is the power of Svelte 5's reactivity model.

#### Flexible Children API

ArchiveNavLink accepts either a `label` prop or arbitrary `children` (a Svelte 5 snippet):

```svelte
<a class="archive-nav-link" class:active href="{href}">
  {#if children}
    {@render children()}
  {:else}
    {label}
  {/if}
</a>
```

This means you can use it two ways:

```svelte
<ArchiveNavLink href="/fandoms">Fandoms</ArchiveNavLink>
```

Or with custom content:

```svelte
<ArchiveNavLink href="/search">
  <span class="nav-icon">🔍</span> Search
</ArchiveNavLink>
```

The `ArchiveHeader` component, however, uses raw `<a>` tags instead of `<ArchiveNavLink>` — the header's nav links are static and don't need active-state detection because the red navbar background makes them visually distinct enough. ArchiveNavLink is used in places like the "Recent Works" byline links and the ArchiveListPage heading, where active state matters.

> **🧪 Try It Yourself**
>
> Open a page that uses ArchiveNavLink (like an Archive list page). Open your browser's developer tools and navigate between pages. Watch the `border-bottom` and `color` on the nav link change in the Elements panel — the `active` class toggles as `$page.url.pathname` updates.

### ArchiveButton — The AO3 Button Aesthetic

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    href = '',
    onclick = undefined,
    disabled = false,
    type = 'button',
    children,
  }: {
    href?: string;
    onclick?: (e: MouseEvent) => void;
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    children: Snippet;
  } = $props();
</script>

{#if href && !disabled}
  <a class="archive-btn" href="{href}" onclick={onclick}>
    {@render children()}
  </a>
{:else}
  <button class="archive-btn" {type} {disabled} {onclick}>
    {@render children()}
  </button>
{/if}

<style>
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
    text-align: center;
    vertical-align: middle;
    transition: background 0.15s, border-color 0.15s;
  }

  .archive-btn:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none;
  }

  .archive-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

#### Polymorphic Rendering: `<a>` or `<button>`

ArchiveButton is a *polymorphic* component — it renders either an `<a>` or a `<button>` depending on whether the `href` prop is set:

```svelte
{#if href && !disabled}
  <a class="archive-btn" href="{href}" onclick={onclick}>
    {@render children()}
  </a>
{:else}
  <button class="archive-btn" {type} {disabled} {onclick}>
    {@render children()}
  </button>
{/if}
```

- **With `href`**: renders an `<a>` — use for navigation links that trigger page loads or route changes.
- **Without `href`** (or when `disabled`): renders a `<button>` — use for form submissions, modal triggers, or actions that don't navigate.

Both share the same `archive-btn` CSS class, so they look identical. The only difference is semantic: an `<a>` in the HTML outline is navigable; a `<button>` is actionable.

#### The AO3 Button Aesthetic

AO3 buttons are famously flat: no shadows, no gradients, no rounded corners. ArchiveButton enforces this in CSS:

```css
.archive-btn {
  border-radius: 0;           /* Sharp corners — no rounding */
  background: var(--archive-bg-raised, #f5f5f5);  /* Flat fill, no gradient */
  border: 1px solid var(--archive-border, #dddddd);
  /* No box-shadow — AO3 buttons are completely flat */
  transition: background 0.15s, border-color 0.15s;  /* Simple color fade */
}
```

Compare this to the Modern mode buttons, which use `--radius-sm` (rounded corners), `box-shadow` for depth, and gradient backgrounds on hover. The Archive buttons are the *opposite* design philosophy: flat, sharp, dense.

On hover, the button lightens its background and darkens its border:

```css
.archive-btn:hover {
  background: var(--archive-border, #dddddd);
  border-color: var(--archive-muted, #666666);
}
```

It's a subtle shift — the button doesn't "pop" or "lift." It just shifts one shade darker, like AO3's buttons do when you hover them.

#### The `variant` Prop Discussion

The original outline mentions a "variant prop: primary (maroon) vs secondary (gray)." Looking at the actual `ArchiveButton.svelte` source, there's no `variant` prop. Instead, the component is a single flat style. The "primary" look (maroon background) is achieved by using `ArchiveButton` with a custom class override, or by using a raw `<a>` styled directly — as seen in `ArchiveWork.svelte`'s action buttons:

```svelte
<!-- In ArchiveWork.svelte, line 103: -->
<a class="action-btn action-read" href={`/read/${encodeURIComponent(m.id)}`}>
  Read Online
</a>
```

```css
.action-read {
  color: var(--archive-bg, #ffffff);
  background: var(--archive-link, #990000);
  border-color: var(--archive-link, #990000);
}
```

The `action-read` class overrides the background to maroon (`--archive-link`). When you need a maroon button, you style it directly. This keeps `ArchiveButton` simple: one style, one purpose.

> **⚠️ Watch Out**
>
> ArchiveButton doesn't have a `variant` prop. If you need a primary (maroon) button, you either:
> 1. Add a `variant="primary"` prop and write the CSS yourself, or
> 2. Use a raw `<a>` or `<button>` with a custom class like `action-read`
>
> The codebase currently uses approach #2. If you add a `variant` prop, make sure it maps to the theme tokens (`--archive-link` for primary, `--archive-bg-raised` for secondary) rather than hard-coded hex values.

#### Disabled State

The `disabled` prop works for both render paths:

```svelte
{#if href && !disabled}
  <a ...>
{:else}
  <button class="archive-btn" {type} {disabled} {onclick}>
{/if}
```

When `disabled` is true and `href` is set, the component falls through to the `<button>` branch (since `href && !disabled` is false). The `:disabled` pseudo-class applies:

```css
.archive-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

This is used in `ArchiveWorkSearchForm.svelte` for the Search button during loading states:

```svelte
<ArchiveButton onclick={onSubmit}>Search</ArchiveButton>
```

When the form is submitting, the parent passes `disabled={true}` and the button grays out with a "not-allowed" cursor.

#### Real-World Usage

ArchiveButton is used in several places across the Archive UI. In `ArchiveWorkSearchForm.svelte` (line 163 and 414), it wraps the Search button in both the page-mode and sidebar-mode layouts. In `ArchiveFilters.svelte`, it appears in the filter action bar:

```svelte
<button class="archive-btn primary" onclick={handleApply} disabled={loading}>
  {loading ? 'Searching…' : 'Apply Filters'}
</button>
<button class="archive-btn clear" onclick={handleClear}>Clear All</button>
```

Note how the filters page uses raw `<button>` elements with `class="archive-btn"` instead of the `<ArchiveButton>` component — this is because they need the `primary` and `clear` modifier classes, which `ArchiveButton` doesn't support. The `archive-btn` class is the shared styling foundation; `ArchiveButton` is just the component wrapper around it.

### The AO3 Button DNA

Let's recap what makes an Archive button feel like an AO3 button:

1. **Zero border radius** (`border-radius: 0`) — sharp, boxy, utilitarian
2. **Flat background** — no gradients, no box-shadow, no "lift"
3. **Maroon hover state** — links shift to `#660066` on hover
4. **2px accent underline on active links** — `border-bottom: 2px solid` on `.active`
5. **Georgia serif font** — inherited from the parent, never overridden
6. **Compact padding** — `0.25em 0.8em` horizontal, `0.3em 0.7em` for nav links
7. **No emojis** — AO3 rule, strictly followed

> **💡 Key Concept — Visual Consistency Through CSS Variables**
>
> Neither ArchiveNavLink nor ArchiveButton hard-codes colors. They use `var(--archive-link, #990000)` for link color, `var(--archive-bg-raised, #f5f5f5)` for button backgrounds, and `var(--archive-border, #dddddd)` for borders. This means when you switch from Archive Classic to Archive Noir, every link, button, and underline updates automatically — because the CSS variables change, not the component CSS.

> **🧪 Try It Yourself**
>
> Navigate to a search results page in Archive mode. Click the "Apply Filters" button — watch how it uses the flat `archive-btn` style (gray background, sharp corners). Now look at the "Download" action button on a work page — it uses the `action-btn` style with a maroon background. Both share the same `border-radius: 0` and the same transition, but they communicate different hierarchy levels through color.

---

*End of Part 2. In Part 3, we'll dive into the content components — WorkBlurb, TagSoup, StatsLine, and the ArchiveSearch results layout that ties them all together.*

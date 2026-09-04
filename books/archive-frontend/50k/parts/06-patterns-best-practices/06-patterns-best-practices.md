# Part 6: Patterns & Best Practices

In this part you'll learn the patterns that hold the Archive UI together — the conditional-rendering trick that lets both views coexist, how the CSS stays clean with custom properties, which Svelte 5 runes power everything under the hood, and the testing strategy that keeps bugs from sneaking through. Think of these as the rules-of-thumb every experienced FicHub developer uses daily. You don't need to memorize them by heart; just read through once, play with the 🧪 Try It Yourself boxes, and come back when your own code needs inspiration.

---

## Chapter 25: The Archive Conditional Pattern

Imagine you have two outfits in your closet: your everyday clothes and your superhero costume. Now imagine you could switch between them with a single flick — no tailoring, no sewing, no buying new clothes. That's exactly what the Archive Conditional Pattern does for FicHub pages.

### The Core Idea

Every route page in the Archive Frontend wraps its **page-specific rendering** inside a simple `if / else` block:

```svelte
<script>
  import { getPref } from '$lib/prefs';
  $: uiMode = getPref('uiMode');
</script>

{#if uiMode === 'archive'}
  <!-- archive view here -->
{:else}
  <!-- modern view (unchanged, untouched) -->
{/if}
```

That's it. Four lines in the script tag. One conditional in the template. Yet this tiny pattern is what makes the entire dual-mode architecture work.

### Why This Works So Well

Let's look at three big reasons this approach is brilliant:

1. **Additive only**: You never touch existing modern-view code. Your changes are *layered on top* like a sticker, not surgery on the original tissue.
2. **Zero regression risk**: The modern shell stays 100% intact because it lives in the `{/else}` branch, completely isolated from archive additions.
3. **Easy to find**: If something looks weird, you open the `.svelte` file, see the `{#if}`, and instantly know "oh, the archive override goes right there."

No clever hacks, no global flag checks buried deep in functions, no mysterious behavior. Just a clear fork in the road at the top of every page.

### Step-by-Step Recipe: Adding Archive Support to a New Page

Follow these steps whenever you want to add archive mode to a fresh route page. Here's the checklist, in order:

#### Step 1 — Import `getPref`

At the top of your `<script>` block, make sure you pull in `getPref`:

```svelte
<script>
  import { getPref } from '$lib/prefs';
  // ...existing imports...
</script>
```

If `$lib/prefs` isn't importing cleanly, double-check that the path is correct — it resolves relative to the Svelte `$lib` alias.

#### Step 2 — Read the Mode Flag

Right after your imports, add one reactive line:

```svelte
$: uiMode = getPref('uiMode');
```

This is a **reactive declaration** (`$:`). It means every time the preference value changes in localStorage, the variable updates automatically. No manual listeners needed.

#### Step 3 — Wrap the Right Thing

Here's the most important step. You do **not** wrap the entire page in the conditional. You wrap only the portion that changes.

A common mistake is doing this:

```svelte
<!-- WRONG: Don't wrap the entire page -->
{#if uiMode === 'archive'}
  <div class="archive-layout">
    <h1>Search</h1>
    <!-- full page re-implemented -->
  </div>
{:else}
  <div class="modern-layout">
    <!-- everything else -->
  </div>
{/if}
```

The modern layout often has shared wrappers (navigation bars, footers, modals). Repeating those in both branches duplicates code and creates sync headaches. Instead, think about **what specifically looks different** between archive and modern, and wrap only that piece:

```svelte
<!-- RIGHT: Wrap only the rendering difference -->
<ModernLayout>
  <slot />
  <ArchiveHeader slot="header" />

  <!-- Modern list rendering stays as-is -->
  {#if uiMode === 'archive'}
    <ArchiveResultList results={results} ficList={ficList} />
  {:else}
    <ModernResultList results={results} />
  {/if}
</ModernLayout>
```

#### Step 4 — Write the Archive Branch

Inside the `{#if}` block, use the archive components we built earlier in this book. Import whatever you need, pass your data in, and let the component handle styling and structure.

For example, on the bookmarks page you might replace just the list rendering:

```svelte
<script>
  import { getPref } from '$lib/prefs';
  import ArchiveBookmarksList from '$lib/ui/archive/ArchiveBookmarksList.svelte';
  import ModernBookmarksList from '$lib/ui/modern/ModernBookmarksList.svelte';

  const { bookmarks } = $state({ fic: [], total: 0 });
  $: uiMode = getPref('uiMode');
</script>

{#if bookmarks.length > 0 && uiMode === 'archive'}
  <ArchiveBookmarksList bookmarks={bookmarks} />
{:else if bookmarks.length > 0}
  <ModernBookmarksList bookmarks={bookmarks} />
{:else}
  <p>You haven't bookmarked any fics yet.</p>
{/if}
```

Notice how the fallback (`{#else}`) still shows a sensible message? That's good UX.

#### Step 5 — Test Both Modes

Open your browser. Switch to Archive Classic, reload, and check the page. Then switch back to Modern and verify nothing broke. If either mode looks wrong, fix it before moving on. Never commit partial support.

### Common Mistakes to Avoid

⚠️ **Forgeting the `{/else}` branch.** If you write `{#if uiMode === 'archive'}` but never close it, the modern view disappears entirely. Svelte won't even compile this — it will give you an error saying the block is unclosed. But if you accidentally swallow the else into a nested condition, the modern view breaks silently. Always pair your `{#if}` with `{/else}`.

⚠️ **Not importing `getPref`.** If you reference `$: uiMode = getPref('uiMode')` without actually importing it, you'll get a runtime error that can be hard to trace because the failure happens on the first render, not at module load time. Double-check the import line.

⚠️ **Wrapping too much or too little.** Wrapping the entire page is wasteful. Wrapping nothing means no archive support at all. The sweet spot: identify *one visual piece* per page that differs between modes and put just that inside the conditional.

💡 **Key Concept: The Archive Conditional is a Rendering Gate, Not a Logic Gate**

This pattern controls what the user **sees**, not what the app **does**. Network requests, authentication checks, state management — all of that runs regardless of which mode is active. The conditional sits purely in the template layer. Keep your logic outside of it.

### A Real Example: The Bookmarks Page

The bookmarks page is a great case study. The overall page structure — the outer container, the header, the navigation — is identical in both modes. Only the list of bookmarks renders differently:

```svelte
<script>
  import { getPref } from '$lib/prefs';
  import ModernBookmarksCard from '$lib/ui/modern/BookmarkCard.svelte';
  import ArchiveBookmarkRow from '$lib/ui/archive/ArchiveBookmarkRow.svelte';
  import type { BookmarkEntry } from '$lib/types';

  const { bookmarks }: { bookmarks: BookmarkEntry[] } = $props();
  $: uiMode = getPref('uiMode');
</script>

<div class="bookmarks-page">
  {#each bookmarks as bookmark (bookmark.id)}
    {#if uiMode === 'archive'}
      <ArchiveBookmarkRow bookmark={bookmark} />
    {:else}
      <ModernBookmarksCard bookmark={bookmark} />
    {/if}
  {/each}
</div>
```

See how elegant this is? Each bookmark card flips independently based on the current mode. If you wanted to add a third view later (say, a compact list), you'd just add another branch. The pattern scales gracefully.

🧪 **Try It Yourself**

Create a new Svelte component called `DemoConditional.svelte` in your project. Put a dropdown that cycles through `"archive"`, `"modern"`, and `"both"` values. Use a conditional to display a different colored box for each mode. Add an alert box that triggers when neither mode is selected. Once it works, rename `"both"` to `"archive-and-modern"` and update the comparison — watch how the conditional handles the new string naturally.

---

## Chapter 26: CSS Architecture in Archive Mode

CSS is where the magic happens — literally. When someone picks "Archive Noir," they're not asking for a JavaScript framework to recolor every element. They're asking CSS custom properties to shift the palette. And that's precisely what happens.

### Custom Properties Are the Only Way In

Every single pixel of color, border, and background in archive-mode components flows through custom CSS variables. You'll never find a raw hex value like `#990000` hardcoded inside a component's `<style>` block. Instead, you'll always see:

```css
a {
  color: var(--archive-link);
}

.panel {
  background: var(--archive-bg);
  border: 1px solid var(--archive-border);
}
```

This rule — *no hard-coded colors in components* — is enforced strictly. Every archive component references `--archive-*` variables exclusively. This design decision has massive consequences, all positive.

### The Two Built-In Presets

FicHub ships with two theme presets, defined in `src/lib/themes/presets.ts`:

| Variable | Archive Classic (light) | Archive Noir (dark) |
|---|---|---|
| `--archive-bg` | `#ffffff` | `#1a1a1a` |
| `--archive-link` | `#990000` | `#cc3333` |
| `--archive-border` | `#dddddd` | `#333333` |

Archive Classic is designed to resemble AO3's default light theme — warm white background, deep red links, light gray borders. Archive Noir is the dark variant — charcoal background, soft coral-red links, subtle dark borders.

You can add more presets later by extending this same file. Each new preset follows the exact same shape: a set of `--archive-*` variable assignments.

### Scoped Styles: No Leaks, Ever

Each Svelte component owns its own `<style>` block:

```svelte
<style>
  .result-title {
    color: var(--archive-link);
    font-family: Georgia, serif;
    margin-bottom: 4px;
  }
</style>
```

Svelte scoping automatically transforms class names behind the scenes so styles never leak between components. Archive components can't accidentally style modern components and vice versa. The separation is absolute.

There is no global stylesheet that touches archive elements. There are no Tailwind utility classes polluting markup. There are no external CSS frameworks pulling in megabytes of unused rules. Just clean, scoped, dependency-free CSS.

### AO3 Aesthetic Principles

The archive mode deliberately echoes the aesthetics familiar to long-time AO3 users:

- **Thin borders**: `1px solid var(--archive-border)` everywhere. No thick outlines, no shadows.
- **Square corners**: `border-radius: 0` on all elements. No rounded buttons, no pill tags.
- **Georgia serif headings**: `font-family: Georgia, serif` for titles and labels. The classic web feel.
- **Compact spacing**: Tight line-heights, small gaps between rows. Text-dense layouts maximize information per screen.
- **Muted emphasis**: Colors are used sparingly — primarily for links and selected states. Everything else stays neutral.

These aren't arbitrary choices. Users who enable archive mode expect this specific visual language. Straying from it feels jarring, like opening a book that suddenly switches from serif to Comic Sans.

### Theme Switching Flow

How does the browser actually change themes when you click a preset? Here's the chain of events:

1. **User selects a theme** → clicks "Archive Classic" or "Archive Noir" in Settings.
2. **`setPref()` saves to localStorage** → the selection persists across sessions.
3. **`applyUiTheme()` sets CSS vars on `document.documentElement`** → this is the magic step. A single function walks through the chosen preset's variables and applies them as inline styles on the root `<html>` element.
4. **Browser re-renders everything** → all components using `var(--archive-bg)`, `var(--archive-link)`, etc. automatically reflect the new values. No JS component updates, no Reactivity chains, just pure CSS cascading down.

The beauty is in step 4. Because CSS custom properties cascade globally, changing one line on the root element refreshes thousands of component styles simultaneously. No force-re-renders, no manual updates. The browser does all the work.

💡 **Key Concept: CSS Variables Are a Cascade Multiplier**

When you set `--archive-link: #cc3333` on `<html>`, every single `color: var(--archive-link)` in every archive component inherits that value automatically. One variable, thousands of effects. This is why the architecture is so clean — you centralize configuration at the root and let CSS do the distribution.

🧪 **Try It Yourself**

Open any archive-mode component's `<style>` block. Find a `color` property using `var(--archive-link)`. Temporarily change it to a plain hex value like `#ff0000` and save. Reload the page — you should see bright red links, ignoring your preset. Now remove the hardcoded value, restore `var(--archive-link)`, and reload again. Notice how the links immediately snap back to whatever your current preset says. You just proved to yourself that the variable pipeline is working.

⚠️ **Watch Out: Hardcoded Colors Bypass Theming**

If you ever write `color: #990000` directly instead of `color: var(--archive-link)`, that element will stay that color forever, regardless of which preset the user selected. This is the most common CSS mistake in archive development. If a new color is needed, add a new `--archive-*` variable to `presets.ts` and reference it with `var()`. Never skip the indirection.

---

## Chapter 27: Svelte 5 Runes in Practice

Svelte 5 introduced a paradigm shift called "runes" — special compiler directives that make reactivity explicit rather than implicit. If you've worked with Svelte 4 before, you already know the old `$:` syntax. Svelte 5 replaces some of that with clearer, more powerful primitives.

Let's walk through every rune used in the Archive Frontend, with real examples.

### `$props()`: Declaring Component Inputs

In Svelte 4, you declared props with `export let`:

```svelte
<script>
  export let fic;
  export let isBookmarked = false;
</script>
```

In Svelte 5, you destructure `$props()` with TypeScript types:

```svelte
<script>
  interface Props {
    fic: SearchResult;
    isBookmarked?: boolean;
  }
  const { fic, isBookmarked = false } = $props<Props>();
</script>
```

This looks slightly more verbose, but the gains are real:

- **Type safety**: The `Props` interface catches typos and missing fields at compile time.
- **Defaults in destructuring**: `isBookmarked = false` reads naturally.
- **Compiler awareness**: Svelte's compiler knows exactly which props exist and can warn you if you misspell one.

Every single archive component uses `$props()`. Search results cards, form fields, header bars — they all follow this pattern. It's become our standard.

💡 **Key Concept: `$props()` Is Your Component's Contract**

Think of the `Props` interface as a contract between parent and child. The parent promises to provide certain values; the child promises to receive them. TypeScript enforces this contract, and Svelte ensures the values update reactively when the parent changes.

### `$state()`: Reactive Local State

Use `$state()` when a component needs its own mutable, reactive data:

```svelte
<script>
  const [formState, setFormState] = $state(<FormState>{/* ... */});
  const [isOpen, setIsOpen] = $state(false);        // dropdown toggle
  const [selectedChipIndex, setSelectedChipIndex] = $state<number>(-1);
</script>

<button onclick={() => setIsOpen(!isOpen)}>Menu</button>
```

Three examples above, three distinct use cases:

1. **`formState`** holds the complete search form configuration (query text, tags, sort order, etc.). Changing one field inside it triggers automatic re-renders of any computed values depending on it.
2. **`isOpen`** is a simple boolean controlling whether a dropdown menu is visible. Toggling it flips visibility.
3. **`selectedChipIndex`** tracks which tag chip the user has clicked for editing — negative means "nothing selected."

Note: `$state()` can take an initial value in parentheses (like `$state(0)` or `$state([])`) or via assignment (like `$let count = $state(0)`). We use the parenthesized form consistently across the archive codebase.

### `$derived()`: Computed Values

Replace Svelte 4's `$:` reactive declarations with `$derived()` for computed properties:

```svelte
<script>
  // Svelte 4 way:
  // $: activeNavLink = $page.url.pathname.split('/').pop();

  // Svelte 5 way:
  const activeNavLink = $derived($page.url.pathname.split('/').pop());
  const filteredChips = $derived(chips.filter(chip => chip.active));
</script>
```

Two key differences from `$:`:

1. `$derived()` returns a **value** you assign to a constant. It's not a side-effect declaration.
2. The expression inside `$derived()` must be a single expression — no multiple statements. If you need complex logic, extract it to a `$state()`-backed callback instead.

In the archive nav, `activeNavLink` changes automatically whenever `$page.url.pathname` changes (because `$page` is SvelteKit's reactive page store). The navigation bar highlights the correct link without any manual listener.

### `$effect()`: Side Effects

Use `$effect()` when you need to run code that affects the outside world — DOM mutations, network calls, subscriptions:

```svelte
<script>
  // Close dropdowns when clicking outside
  $effect(() => {
    const handler = (e: MouseEvent) => {
      if (!dropdownRef.current?.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  });
</script>
```

The cleanup function (the `return () => ...`) is crucial. Without it, the event listener accumulates every time the effect re-runs, leaking memory.

Real-world `$effect()` usages in archive code include:

- **Outside-click handlers** for closing dropdown menus.
- **Theme initialization** on mount — reading the saved preference and applying it.
- **URL synchronization** — updating the browser address bar when form state changes.

💡 **Key Concept: `$effect()` ≠ Lifecycle Hooks**

Don't think of `$effect()` as `onMount` or `onDestroy`. It's reactive: it re-runs whenever the values it references change. The cleanup function runs before each re-run. This dual behavior is what makes `$effect()` powerful but also potentially tricky — you need to reason about *when* it re-triggers, not just *when* it fires once.

### `{@snippet}` and `{@render}`: Slots, Evolved

Svelte 4 used named slots:

```svelte
<SlotExample><div slot="header">Hello</div></SlotExample>
```

Svelte 5 replaces this with typed snippets:

```svelte
<!-- Parent passes content -->
<ArchiveLayout>
  <div slot="header">{@render header()}</div>
</ArchiveLayout>

<!-- Child declares the snippet slot -->
<script>
  interface SlotProps {
    header?: () => VNode;
  }
  const { header } = $props<SlotProps>();
</script>

<Layout>
  <nav>{@render header?.()}</nav>
</Layout>
```

In the Archive Frontend, `{@render header()}` appears in `ArchiveLayout` to allow custom headers per page. The parent provides a snippet function; the child calls it wherever it wants the content injected. Type-safe, explicit, and composable.

⚠️ **Watch Out: Remember the Parentheses**

When rendering a snippet, you call it like a function: `{@render header()}`. Forgetting the parentheses (`{@render header}`) renders the function definition itself, not its output. You'll see `[Function header]` in the page — a confusing debug experience. Always add `()`.

### Migration Map: Svelte 4 → Svelte 5 for Archive Devs

If you're coming from Svelte 4, here's a quick translation table:

| Svelte 4 | Svelte 5 | Notes |
|---|---|---|
| `export let x = 5` | `const { x = 5 } = $props()` | Mechanical, drop-in replacement |
| `$: doubled = x * 2` | `const doubled = $derived(x * 2)` | Mostly mechanical |
| `$: if (x) doThing()` | `$effect(() => { if (x) doThing(); })` | Needs care — effects always run on mount |
| `<Component slot="name">` | `<Component>{@render snippet()}</Component>` | Requires snippet prop in child |
| `.store.svelte` files | Local `$state()` + `$derived()` | No stores needed in archive code |

Notice the last row: there are **no `.store.svelte` files** anywhere in the Archive Frontend. All state is local to components (via `$state()`) or managed centrally in the prefs store (which handles only user preferences, not page data). This simplifies reasoning about data flow enormously — you can find every piece of state by scanning a single file.

🧪 **Try It Yourself**

Create a component called `RuneDemo.svelte`. Give it three pieces of state: `count` (number), `name` (string), and `showDetails` (boolean). Derive `greeting` from `name`. Render a button that increments `count` and toggles `showDetails`. Add a `$effect` that logs `count` to the console whenever it changes. Watch the log fire when you click, proving the reactive chain works end-to-end.

---

## Chapter 28: Testing Strategy

Writing code is only half the job. Making sure it keeps working tomorrow — when someone adds a feature, refactors a component, or fixes a typo — is the other half. Testing is how you guarantee that future-you doesn't break present-you's hard work.

### Unit Tests: The Detailed Safety Net

Unit tests live at `frontend/src/lib/ui/archive/searchForm.test.ts`. This file alone contains **47 passing tests** — each one verifying a specific behavior of the search form logic.

Let's walk through what they cover:

#### Matrix Row Mapping

The search form supports many parameter combinations — filters, sorts, ranges, exclusions. Every single mapping row in the SUPPORTED matrix gets its own test:

```typescript
it('maps "All Works" filter to query.all_works=true', () => {
  const state: FormState = { filter: 'all_works', ...rest };
  const params = formStateToUrl(state);
  expect(params.get('all_works')).toBe('true');
});

it('maps sort_by="hits" to URL param hits', () => {
  const state: FormState = { sort_by: 'hits', ...rest };
  const params = formStateToUrl(state);
  expect(params.get('sort_on')).toBe('hits');
});
```

Each test isolates one row of the mapping table. If a row stops mapping correctly, this test fails immediately, pointing you to the exact broken row.

#### Range Parsing Edge Cases

Range inputs (like word count or kudos range) need to handle messy user input. The tests cover:

- **Empty strings** → treated as "no limit"
- **Negative numbers** → clamped to zero or rejected
- **Malformed ranges** like `"abc-def"` or `"10-"` → parsed gracefully or errored clearly
- **Single values** → applied as both min and max

```typescript
it('parses empty range as unlimited', () => {
  const result = parseRange('');
  expect(result).toEqual({ min: null, max: null });
});

it('rejects malformed range "abc-def"', () => {
  const result = parseRange('abc-def');
  expect(result.hasError).toBe(true);
});
```

#### URL Round-Trip Verification

This is perhaps the most valuable test pattern in the entire suite:

```typescript
it('round-trips through URL conversion', () => {
  const original: FormState = {
    query: 'drarry',
    filter: 'complete',
    sort_by: 'kudos',
    range_words_min: 5000,
    chips: [{ type: 'relationship', label: 'Drarry' }],
  };
  const urlParams = formStateToUrl(original);
  const restored = urlToFormState(urlParams, []);
  expect(restored).toEqual(original);
});
```

Convert form state → URL string → back to form state. If the round trip produces anything different, the test fails. This catches silent data loss — like a tag getting dropped during conversion — which would otherwise be invisible until a user noticed their search changed unexpectedly.

#### Chip Merge Deduplication

When you add a tag chip that already exists, the merge logic should deduplicate:

```typescript
it('deduplicates chips when merging identical tags', () => {
  const existing = [createChip('character', 'Harry')];
  const incoming = [createChip('character', 'Harry'), createChip('tag', 'Fluff')];
  const merged = mergeChips(existing, incoming);
  expect(merged.length).toBe(2); // Harry counted once
  expect(merged.some(c => c.label === 'Fluff')).toBe(true);
});
```

Without this, users could accidentally flood their tag bar with duplicates.

#### Inert Field Verification

Fields marked as "inert" (user disabled them) should never appear in the final query:

```typescript
it('buildSearchQuery excludes inert params', () => {
  const state: FormState = {
    query: 'test',
    filter: 'complete',
    _f_filter: true,  // _f_ prefix marks as inert
  };
  const query = buildSearchQuery(state);
  expect(query).not.toContain('f_complete:true');
});
```

This ensures disabled filters truly disappear from API calls, saving bandwidth and preventing confusing results.

### E2E Tests: Full-Page Smoke Tests

End-to-end tests live at `frontend/e2e/archive-ui.spec.ts`. These use Playwright to launch a real browser, navigate to actual routes, and verify the page behaves correctly. There are **9 E2E tests** covering the major surfaces:

| Test | What It Checks |
|---|---|
| Home loads in archive mode | `/` renders archive layout, not 404 |
| Search form renders | Fieldsets for query, filters, ranges all present |
| Tags page shows 6 categories | Relationship, Character, Freeform, Rating, Medium, Additional tags listed |
| Bookmarks shows login prompt | Unauthenticated users see a sign-in CTA |
| Authors page has search input | `/authors` displays an autocomplete-ready search field |
| Settings shows Interface Style | `/settings` renders the theme selector section |
| Tag chips render correctly | Clickable chip elements with correct labels |
| Dropdown closes on outside click | Clicking the page body dismisses open menus |
| Theme switch updates colors | Picking Archive Noir changes CSS variable values |

These tests don't dig into implementation details. They ask high-level questions: "Can a user land on the page?" "Are the controls visible?" "Does switching themes work?" If all nine pass, the core user journeys are intact.

### Coverage Gates

The project enforces a minimum code coverage threshold through Vitest:

```bash
vitest --coverage --reporter=text-summary
```

Targets are approximately **85% statement**, **78% branch**, and **76% function** coverage on `src/lib/**`. These numbers mean "most of our library code is exercised by tests." They're not perfect — 100% coverage is rarely worth the effort — but hitting these gates gives confidence that new code gets tested before it merges.

⚠️ **Known Noise: Pre-existing Test Failures**

Be aware that five unit test files unrelated to the archive frontend currently fail due to i18n (internationalization) gaps. These are **known issues, documented separately**, and will be fixed in a future pass. Do not treat these failures as regressions caused by your archive work. Focus your CI attention on the archive-specific test files (`searchForm.test.ts` and `archive-ui.spec.ts`).

### What to Test vs. What to Skip

Here's a practical guide for writing new tests:

**✅ DO test:**
- Form state transformations (`formStateToUrl`, `urlToFormState`)
- URL parameter parsing and serialization
- Chip merge/dedup logic
- Inert field filtering
- Input validation (empty strings, negatives, overflows)
- Theme application (variable values set on root)

**❌ DON'T test:**
- Styling and layout details — Visual appearance belongs in visual regression tests (separate CI check using screenshot comparison), not unit tests.
- Exact pixel positions — Components can shift due to browser quirks; test behavior, not coordinates.
- Browser chrome interactions — Tab focus order and keyboard shortcuts are covered by accessibility audits, not automated unit tests.
- Third-party library internals — Trust that SvelteKit, Svelte, and Playwright work correctly.

💡 **Key Concept: Test the Contract, Not the Implementation**

Write tests that describe *what* the code should do, not *how* it does it. If you refactor the internal algorithm but preserve the public behavior, your tests should still pass. This keeps tests useful through changes rather than turning them into brittle maintenance burdens.

🧪 **Try It Yourself**

Pick any form-mapping function from `searchForm.ts`. Write one test that verifies it maps a specific `FormState` shape to the expected URL parameters. Run it with `npm test searchForm.test.ts`. Fix any assertion mismatches. Then add a second test for the reverse direction: convert the URL back and verify you get the original state. This one exercise covers the entire round-trip testing pattern used across the archive suite.

---

*End of Part 6.*

You now know the conditional rendering pattern, the CSS variable architecture, the Svelte 5 runes powering every component, and the testing strategy that keeps everything reliable. These are the foundations — revisit them whenever you feel uncertain about how things fit together. Next up: deployment and performance optimization.

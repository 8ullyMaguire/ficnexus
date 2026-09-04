# Part 3: Content Components

## Chapter 9: WorkBlurb — The Fic Card

If `ArchiveLayout` is the wrapper and `ArchiveHeader` is the face, then `WorkBlurb.svelte` is the heart of every list page in the Archive. At 208 lines (`frontend/src/lib/ui/archive/WorkBlurb.svelte`), it renders a single *fic card* — that familiar bordered box containing a story's title, author, tags, description snippet, statistics, and action buttons. Search results pages, recent work widgets, recommendation lists, bookmark feeds — they're all rows of `WorkBlurb` cards, side by side.

This is where the three systems you met earlier (themes, i18n, auth) come together into something the user actually clicks on.

### The Props Interface

Every `WorkBlurb` receives a search result object and an optional flag:

```svelte
<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import TagSoup from './TagSoup.svelte';
  import StatsLine from './StatsLine.svelte';
  import { mapRating } from './rating.js';

  interface FicSearchResult {
    author?: string | null;
    chapters?: number | null;
    comment_count?: number | null;
    description?: string | null;
    kudos_count?: number | null;
    rank?: number | null;
    rating?: string | null;
    snippet?: string | null;
    source?: string | null;
    status?: string | null;
    tags?: Array<{ category?: number | string; name: string }>;
    title?: string | null;
    total_freeform?: number | null;
    updated?: string | null;
    url_id?: string | null;
    words?: number | null;
    [key: string]: unknown;
  }

  let {
    fic,
    showAllTags = false,
  }: {
    fic: FicSearchResult;
    showAllTags?: boolean;
  } = $props();
</script>
```

The `fic` prop carries the full set of fields returned by the search API (defined in `frontend/src/lib/api/search.ts`). Most are optional — not every search result has every field — so the component uses `?? 'Untitled'` and `?? 'Anonymous'` defaults throughout.

The `showAllTags` boolean controls whether `TagSoup` truncates or expands its tag list. By default, `TagSoup` hides freeform tags beyond six. When `showAllTags` is `true`, every tag shows. A search results page passes `false`; a dedicated "tags" view would pass `true`.

### Derived Values and Reactive State

Inside `<script>`, `$derived` extracts what the template needs:

```typescript
const title = $derived(fic.title ?? 'Untitled');
const author = $derived(fic.author ?? 'Anonymous');
const rating = $derived(mapRating(fic.rating));
const isLoggedIn = $derived(auth.isLoggedIn);
const snippet = $derived(fic.snippet ?? fic.description ?? '');
```

`mapRating()` (imported from `./rating.js`) converts a short AO3-style code like `"general"` or `"explicit"` into its human-readable form: `"General Audiences"` or `"Explicit"`. If the rating is missing, it returns `"Not Rated"`. We'll explore this function in detail in Chapter 12.

`isLoggedIn` reads from the Svelte 5 class-based auth store. When the user logs in or out while browsing a results page, every visible `WorkBlurb` re-renders instantly because `$derived` reacts to `$state` changes. No manual subscription cleanup needed.

Then comes the tag grouping logic — a clever `$derived.by()` block that transforms the flat tag array from the API into a shape `TagSoup` expects:

```typescript
const groupedTags = $derived.by(() => {
  const result: Record<string, Array<{ category?: number | string; name: string }>> = {};
  if (!fic.tags) return result;
  for (const tag of fic.tags) {
    const key = String(tag.category ?? '4');
    if (!result[key]) result[key] = [];
    result[key].push(tag);
  }
  return result;
});
```

The API returns tags as a flat array where each tag carries a numeric `category` (1=Fandom, 2=Character, 3=Relationship, 4=Freeform/Additional, 5=Warning, 6=Category). `groupedTags` fans them out into a dictionary keyed by category number: `{ "1": [...], "2": [...], "4": [...] }`. The fallback category `'4'` means any tag without a category gets shoved into "Additional Tags."

A small local `truncate()` helper chops long snippets at 300 characters:

```typescript
function truncate(text: string, max: number): string {
  if (text.length <= max) return text;
  return text.slice(0, max).trimEnd() + '…';
}
```

It's defined locally inside `<script>` rather than imported from `rating.ts` because only `WorkBlurb` does snippet truncation — the other components don't need it.

### The Template

The HTML body of a `WorkBlurb` follows a clear top-to-bottom information hierarchy:

```svelte
<article class="work-blurb">
  <!-- HEADER: Rating badge + Title -->
  <div class="blurb-header">
    <span class="blurb-rating">{rating}</span>
    <a class="blurb-title" href="/fic/{fic.url_id}">{title}</a>
  </div>

  <!-- BYLINE -->
  <div class="blurb-byline">
    by <a href="/search?q=author:{encodeURIComponent(author)}">{author}</a>
  </div>

  <!-- TAGS -->
  {#if fic.tags?.length}
    <TagSoup tags={groupedTags} showAll={showAllTags} />
  {/if}

  <!-- SNIPPET -->
  {#if snippet}
    <div class="blurb-snippet">{truncate(snippet, 300)}</div>
  {/if}

  <!-- STATS -->
  <StatsLine
    words={fic.words}
    chapters={fic.chapters}
    status={fic.status}
    kudos={fic.kudos_count}
    updated={fic.updated}
  />

  <!-- ACTIONS -->
  <div class="blurb-actions">
    <a class="blurb-action" href="/download?url={encodeURIComponent(fic.source ?? '')}">
      Download
    </a>
    <a class="blurb-action blurb-action--primary" href="/read/{fic.url_id}">
      Read
    </a>
    {#if isLoggedIn}
      <button class="blurb-action" type="button">Bookmark</button>
      <button class="blurb-action" type="button">Kudos</button>
    {:else}
      <a class="blurb-login-hint" href="/login">Log In to Bookmark or Give Kudos</a>
    {/if}
  </div>
</article>
```

Four sections stacked vertically:

1. **Header** — A flex row showing the rating badge next to the title. The title link goes to `/fic/{url_id}`, which fetches the full export and eventually lands on the `ArchiveWork` detail page (covered in Chapter 13).

2. **Byline** — "by [Author Name]" where the author name links to a filtered search for their works: `/search?q=author:{encodedName}`. This lets readers discover more stories by the same writer.

3. **Tags** — A conditional render. If there are no tags (possible for poorly-tagged imports), nothing shows. Otherwise, `TagSoup` formats them into labeled rows (see Chapter 10).

4. **Snippet** — Another conditional: only shown when the search API returned a highlight snippet or description. Truncated to 300 characters with an ellipsis.

5. **StatsLine** — Passes through raw numbers. `StatsLine` handles the formatting (word count commas, relative dates, etc.). See Chapter 11.

6. **Actions** — Four interactive elements: Download, Read, Bookmark, and Kudos. The first two are always `<a>` links (navigation). Bookmark and Kudos change based on auth state: logged-in users see `<button>` elements; anonymous visitors see a polite prompt linking to login.

### Action Buttons and Auth-Aware Rendering

The action button section demonstrates a common pattern across the entire Archive frontend: conditionally render different UI based on authentication status.

When the user is logged in:
```svelte
<button class="blurb-action" type="button">Bookmark</button>
<button class="blurb-action" type="button">Kudos</button>
```

These are plain buttons (not links) because bookmarking and giving kudos are *actions* that will trigger JavaScript handlers in the parent component. Right now they're placeholders — the actual API integration hooks up later. But structurally they follow AO3's layout: four equally-sized button slots, two navigation links, and one inline hint.

When the user is *not* logged in, the buttons disappear and get replaced by a subtle italicized hint:
```svelte
<a class="blurb-login-hint" href="/login">Log In to Bookmark or Give Kudos</a>
```

This is gentle UX — it doesn't block interaction with Download and Read, but it gently suggests what the user is missing.

### CSS Styling — Flat, Boxed, AO3-Consistent

The CSS enforces the familiar AO3 look: sharp corners, flat fills, thin borders, serif fonts. Here are the most important rules:

```css
.work-blurb {
  border: 1px solid var(--archive-border, #dddddd);
  padding: 0.75em 1em;
  margin-bottom: 0.75em;
  background: var(--archive-bg, #ffffff);
}

.blurb-header {
  display: flex;
  align-items: baseline;
  gap: 0.5em;
  flex-wrap: wrap;
}

.blurb-title {
  font-size: 1.2em;
  font-weight: 700;
  color: var(--archive-link, #990000);
  text-decoration: none;
  line-height: 1.4;
}

.blurb-title:hover {
  text-decoration: underline;
}

.blurb-action--primary {
  color: var(--archive-bg, #ffffff);
  background: var(--archive-link, #990000);
  border-color: var(--archive-link, #990000);
}

.blurb-login-hint {
  font-size: 0.85em;
  color: var(--archive-muted, #666666);
  font-style: italic;
}
```

Key observations:

- **The box model** — Every card is a simple rectangle: 1px border, 0.75em internal padding, 0.75em bottom margin. That margin stacks between cards. There's no `border-radius` — AO3 cards are perfectly square-cornered, even on mobile.

- **The header layout** — `display: flex` with `align-items: baseline` keeps the rating badge and the title sitting nicely on the same visual line, even though they're different font sizes. The `gap: 0.5em` adds breathing room. `flex-wrap: wrap` prevents overflow on narrow screens.

- **The primary button** — `blurb-action--primary` overrides the gray button style with maroon (`#990000`) background and white text. This is the "Read" button — the main call-to-action that stands out among the secondary actions. On hover, it shifts to the visited-purple `#660066`.

- **Login hint styling** — Small, italic, muted gray. Deliberately unobtrusive. It's telling the user something, not trying to sell them on it.

### How WorkBlurb Uses Other Components

`WorkBlurb` is a *compositional* component — it doesn't implement tag formatting or stat display itself. Instead, it delegates:

| Sub-component | Purpose | File |
|---|---|---|
| `TagSoup` | Renders categorized tag rows | `TagSoup.svelte` (Ch. 10) |
| `StatsLine` | Formats words/chapters/kudos/updated inline | `StatsLine.svelte` (Ch. 11) |
| `mapRating()` | Converts rating codes → readable strings | `rating.ts` (Ch. 12) |

This compositional approach means `TagSoup` can be used independently (on the detail page, for example) and `StatsLine` works the same way on both list views and detail views. `WorkBlurb` is the glue that holds them together.

> **💡 Key Concept — Component Composition**
>
> `WorkBlurb` is the highest-level content component in the Archive: it doesn't do rendering-heavy work itself. Instead, it *composes* smaller specialized components (`TagSoup`, `StatsLine`) and pure functions (`mapRating`). This mirrors how AO3's own PHP templates organize markup into reusable partials. In Svelte terms, composition happens naturally: import the thing, use it in the template. No render props, no HOCs, no complicated wiring. Just declarative assembly of parts.

### Summary Statistics Passed Down

Notice how `WorkBlurb` receives raw, unformatted data from the API and immediately passes it down:

```svelte
<StatsLine
  words={fic.words}        // raw number: 1204116
  chapters={fic.chapters}  // raw number: 109
  status={fic.status}      // string: "complete" or "In Progress"
  kudos={fic.kudos_count}  // raw number: 412
  updated={fic.updated}    // ISO date string: "2024-06-10T12:00:00Z"
/>
```

The formatting happens deep inside `StatsLine`, which calls `formatWords()`, `chaptersDisplay()`, and `formatUpdated()` from `rating.ts`. This separation — raw data at the top, formatted display at the bottom — keeps each layer focused on its job.

> **🧪 Try It Yourself**
>
> Open the search results page in Archive mode. Pick any fic card and inspect its DOM tree. You'll see `<article class="work-blurb">` wrapping a series of child `<div>`s and nested components. Use your browser's developer tools to toggle the `.blurb-action--primary` class on the "Read" button — watch the background shift from maroon to purple-gray as the hover styles take over. Now try removing the `.work-blurb` border and see what happens to the visual separation between cards.

> **⚠️ Watch Out**
>
> The Bookmark and Kudos buttons in `WorkBlurb` currently don't have `onclick` handlers wired up — they're structural placeholders. The parent component (e.g. the search results page) is responsible for attaching those handlers and calling the appropriate API endpoints. Don't expect clicking them to do anything yet. That's fine for now — the goal of `WorkBlurb` is to *display* correctly, not to handle domain logic.

---

## Chapter 10: TagSoup — Tag Display

If `WorkBlurb` is the card, `TagSoup.svelte` is the label maker. At 117 lines (`frontend/src/lib/ui/archive/TagSoup.svelte`), it takes a dictionary of tags — grouped by category — and renders them as clean, labeled rows. Fandoms, Characters, Relationships, Additional Tags, Warnings, Categories — each category becomes a bolded gray label followed by comma-separated maroon links.

AO3 calls these "tags" a soup because fics often carry dozens of them, mixing structured metadata (fandoms, characters) with free-form creative labels (fanfic-specific tropes, shipping pairings). FicHub preserves this soup metaphor literally in the component name.

### The Props Interface

```svelte
<script lang="ts">
  import {
    categoryLabel,
    orderedCategories,
  } from './rating.js';

  interface Tag {
    category?: number | string;
    name: string;
    [key: string]: unknown;
  }

  let {
    tags = {},
    showAll = false,
  }: {
    tags: Record<string, Tag[]>;
    showAll?: boolean;
  } = $props();

  const MAX_FREEFORM = 6;
  const ordered = $derived(orderedCategories(tags));

  function tagUrl(type: string, name: string): string {
    return `/search?include_tags=${type}:${encodeURIComponent(name)}`;
  }
</script>
```

Two props control behavior:

- **`tags`** — A dictionary mapping category keys (strings like `"1"`, `"2"`, `"4"`) to arrays of tag objects. Each tag has a `name` and an optional `category`. Default is an empty object so the component renders silently when given no data.
- **`showAll`** — When `false`, additional tags (freeforms, category 4) get truncated to six items with a "+N more" expandable summary. When `true`, all freeforms show unconditionally.

The `MAX_FREEFORM` constant is hardcoded at 6 — matching AO3's limit for collapsed freeform tags on list views. The `tagUrl()` helper builds search-links in the format `/search?include_tags={category}:{tagName}`, which the backend parses and filters against.

### Ordered Categories

The secret sauce is `orderedCategories()`, imported from `rating.ts`:

```typescript
export function orderedCategories(grouped: Record<string, unknown[]>): string[] {
  return CATEGORY_ORDER.filter((c) => grouped[c]?.length);
}
```

Recall that `CATEGORY_ORDER` is defined in `rating.ts` as:

```typescript
const CATEGORY_ORDER = ['1', '2', '6', '3', '4', '5'];
```

That order — Fandoms, Characters, Categories, Relationships, Additional Tags, Warnings — matches AO3's display convention. Some users expect fandoms first, others want characters first; AO3 settled on this order years ago and FicHub preserves it for familiarity. `orderedCategories` filters out any categories that are empty, so a fic with only fandoms and additional tags will render just two rows.

The derived value updates reactively: whenever `tags` changes (which happens when you navigate between search results pages), `ordered` recalculates automatically via `$derived`.

### Rendering Rows with `{#each}`

The template uses Svelte's `{#each}` iterating over the ordered category keys:

```svelte
<div class="tag-soup">
  {#each ordered as cat}
    {@const items = tags[cat] ?? []}
    {#if items.length > 0}
      <div class="tag-row">
        <span class="tag-label">{categoryLabel(cat)}</span>
        <span class="tag-list">
          {#if cat === '4' && !showAll && items.length > MAX_FREEFORM}
            {/* Truncated freeform tags with expandable details */}
            {#each items.slice(0, MAX_FREEFORM) as tag, i}
              {#if i > 0}<span class="tag-sep">,</span>{/if}
              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
            {/each}
            <details class="tag-more-details">
              <summary class="tag-more-summary">
                +{items.length - MAX_FREEFORM} more
              </summary>
              <span class="tag-list">
                {#each items.slice(MAX_FREEFORM) as tag, i}
                  {#if i > 0}<span class="tag-sep">,</span>{/if}
                  <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
                {/each}
              </span>
            </details>
          {:else}
            {#each items as tag, i}
              {#if i > 0}<span class="tag-sep">,</span>{/if}
              <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
            {/each}
          {/if}
        </span>
      </div>
    {/if}
  {/each}
</div>
```

This is the most complex template logic in the entire content component family. Three key patterns worth noting:

#### The `{@const}` Shorthand

```svelte
{@const items = tags[cat] ?? []}
```

Instead of referencing `tags[cat]` repeatedly, we assign it once to a local constant `items`. The `{@const}` declaration is evaluated each iteration, so it picks up whatever tags belong to the current category. It's cleaner than repeating `tags[cat]` three times in the inner template.

#### The Freeform Truncation Logic

The truncation gate checks three conditions simultaneously:
```svelte
{#if cat === '4' && !showAll && items.length > MAX_FREEFORM}
```

Only for Additional Tags (category 4), only when `showAll` is `false`, and only when there are more than six tags. All three must be true. If any one is false, the branch falls through to the un-truncated rendering below.

The truncated portion renders exactly six tags via `items.slice(0, MAX_FREEFORM)`, then wraps the remainder in a native HTML `<details>` element. Clicking the summary reveals the hidden tags — no JavaScript required, fully keyboard-navigable, accessible by default.

#### Comma-Separated Lists Without Trailing Commas

```svelte
{#each items as tag, i}
  {#if i > 0}<span class="tag-sep">,</span>{/if}
  <a class="tag-link" href={tagUrl(cat, tag.name)}>{tag.name}</a>
{/each}
```

The `{#if i > 0}` guard prevents a leading comma before the first item. Each tag link is styled in maroon and navigates to a filtered search. The separator is a plain `<span>` with class `tag-sep` — rendered as a visually lighter `, ` that sits between the tag links.

### CSS Styling

```css
.tag-soup {
  margin: 0.3em 0 0.5em;
  font-size: 0.9em;
  line-height: 1.7;
}

.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25em;
}

.tag-label {
  font-weight: 700;
  color: var(--archive-muted, #666666);
  margin-right: 0.35em;
  flex-shrink: 0;
}

.tag-link {
  color: var(--archive-link, #990000);
  text-decoration: none;
}

.tag-link:hover {
  text-decoration: underline;
}

.tag-more-summary {
  color: var(--archive-muted, #666666);
  cursor: pointer;
  font-size: 0.92em;
}

.tag-more-summary:hover {
  color: var(--archive-link, #990000);
}
```

- **`.tag-label`** — Bold, gray (`#666666`), non-shrinking (`flex-shrink: 0`) so "Fandoms:" stays intact even on narrow screens where the tag links wrap to the next line.
- **`.tag-link`** — Maroon colored, no underline until hover. Same treatment as the title links in `WorkBlurb` — consistency across the archive.
- **`.tag-more-summary`** — The clickable "+N more" text. Hover turns it maroon, signaling interactivity. Because it lives inside `<details>`, the browser handles the open/close transition natively.

### Why Native `<details>` for Expansion?

Using `<details>/<summary>` instead of Svelte state (`let expanded = $state(false)`) has real advantages:

1. **Accessibility** — Screen readers announce "expandable section" automatically. Keyboard users can Tab to it and press Enter to toggle.
2. **No JS overhead** — The browser manages the attribute. If JavaScript fails to load, the tags still expand.
3. **State persistence** — The `<details>` element remembers its open/closed state during navigation (within the same document lifetime). You could add `open` as a reactive hook if needed, but the default behavior works well.
4. **Less code** — No need for separate click handlers, CSS transitions for open/close animations, or aria-expanded attributes.

> **💡 Key Concept — Progressive Enhancement**
>
> TagSoup's expandable section is built on a progressive enhancement model: the base experience (collapsed freeform tags) works without any JavaScript. The enhanced experience (click to expand) is provided by the browser's native `<details>` element. If you're building an Archive feature and wondering "should I use Svelte state or native HTML here?" — the rule of thumb is: if the browser already handles it (<details>, <form>, <select>), prefer the native element. Reserve Svelte state for things the browser *doesn't* handle.

> **🧪 Try It Yourself**
>
> Go to a search results page and find a fic with many additional tags (more than six). Look for the "+N more" link after the sixth tag. Click it — the hidden tags slide open using the native `<details>` expand animation. Right-click the "+N more" text and choose "Inspect Element" in developer tools. Notice it's inside a `<details>` element with a `<summary>` child — pure HTML, no custom widget.

> **⚠️ Watch Out**
>
> The `tagUrl()` function encodes the tag *name* but not the category in the URL-encoded segment — it interpolates the category directly: `/search?include_tags=${type}:${encodeURIComponent(name)}`. This means the `:` character acts as a delimiter understood by the backend parser. Don't try to URL-encode the colon; the backend expects the literal format `type:name`.

---

## Chapter 11: StatsLine — Single-Line Stats

Meet the simplest component in the content family. At just 52 lines (`frontend/src/lib/ui/archive/StatsLine.svelte`), `StatsLine` squeezes five pieces of fic metadata into a single horizontal line: word count, chapter count, kudos, and update time, separated by middot characters.

It's also the most *used* component — every `WorkBlurb` on a search results page includes one, and the detail page (ArchiveWork) embeds another. Two appearances per fic, one tiny component.

### The Props Interface

```svelte
<script lang="ts">
  import { formatWords, chaptersDisplay, formatUpdated } from './rating.js';

  let {
    words = 0,
    chapters = 0,
    status = '',
    kudos = 0,
    updated = '',
  }: {
    words?: number | null;
    chapters?: number | null;
    status?: string | null;
    kudos?: number | null;
    updated?: string | null;
  } = $props();

  const wordsFmt = $derived(formatWords(words));
  const chapFmt = $derived(chaptersDisplay(chapters, status));
  const kudosFmt = $derived(kudos?.toLocaleString('en-US') ?? '0');
  const updatedFmt = $derived(formatUpdated(updated));
</script>
```

Five numeric/string props, all nullable. The component accepts raw values and formats them internally. Importantly, it depends on *three* pure functions from `rating.ts`:

| Function | Input | Output |
|---|---|---|
| `formatWords()` | `1204116` | `"1,204,116"` |
| `chaptersDisplay()` | `(109, "complete")` | `"109/109"` |
| `formatUpdated()` | `"2024-06-10T12:00:00Z"` | `"3d ago"` |

The kudos formatting is handled inline using `toLocaleString('en-US')` because the same utility isn't exported from `rating.ts` (it's specific to StatsLine's needs — `formatWords` serves a similar purpose but for word counts).

### The Template

```svelte
<div class="stats-line">
  <span>Words: {wordsFmt}</span>
  <span class="stats-sep">·</span>
  <span>Chapters: {chapFmt}</span>
  {#if kudos}
    <span class="stats-sep">·</span>
    <span>Kudos: {kudosFmt}</span>
  {/if}
  {#if updatedFmt}
    <span class="stats-sep">·</span>
    <span>Updated: {updatedFmt}</span>
  {/if}
</div>
```

Three fixed columns (Words, Chapters) always appear. Two optional columns (Kudos, Updated) render conditionally — only when the parent provides non-empty values. The middot separator `·` (Unicode U+00B7) gives visual rhythm: compact punctuation that separates values without the visual weight of a pipe `\|` or slash `/`.

### CSS — Narrow Enough for Mobile

```css
.stats-line {
  font-size: 0.85em;
  color: var(--archive-muted, #666666);
  line-height: 1.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stats-sep {
  margin: 0 0.4em;
  opacity: 0.5;
}
```

The critical properties:

- **`white-space: nowrap`** — forces the entire line onto one visual line. On very narrow screens (mobile portrait), this means the line might get cut off.
- **`overflow: hidden` + `text-overflow: ellipsis`** — if the line is too wide for its container, the trailing characters are replaced with `…`. So if the stats line is too long, you lose "Updated: 3d ago" first, not "Words: 1,204,116".
- **`opacity: 0.5`** on the middots — makes separators visually lighter than the text, creating a subtle hierarchy: the data values dominate, the separators whisper.
- **`font-size: 0.85em`** — smaller than surrounding body text, making stats feel like metadata rather than content.

> **💡 Key Concept — Metadata Design Patterns**
>
> StatsLine embodies several established metadata design patterns borrowed from AO3:
> 
> 1. **Middot separators** — Used extensively on AO3's review pages, profile headers, and submission confirmations. More compact than slashes, less formal than pipes.
> 2. **Relative timestamps** — "3d ago" instead of "June 10, 2024" tells users *how fresh* content is without forcing them to mentally compare dates. Implemented by `formatUpdated()` in `rating.ts`.
> 3. **Complete-chapter shorthand** — "109/109" signals completion instantly. Compare to "109/?" for in-progress works — the question mark communicates "we don't know how many total chapters yet." Implemented by `chaptersDisplay()` in `rating.ts`.

> **⚠️ Watch Out**
>
> The `white-space: nowrap` + `text-overflow: ellipsis` combo means the stats line is not responsive — it won't wrap to multiple lines on narrow screens. If you're adding more stat fields in the future, consider whether the line might become unreadably long. One option is switching to a vertical stack on mobile using a media query:
>
> ```css
> @media (max-width: 480px) {
>   .stats-line {
>     white-space: normal;
>   }
>   .stats-sep {
>     display: none;
>   }
> }
> ```
>
> Currently no such breakpoint exists — the stats line may truncate on very small phones, but this is considered acceptable since the core stats (words and chapters) remain visible.

---

## Chapter 12: rating.ts — Pure Helpers

Before we meet the largest content component (`ArchiveWork`), let's pause at the foundation beneath them all: `rating.ts` (`frontend/src/ui/archive/rating.ts`, 118 lines). Despite its name, this file contains no rating display logic. It's a library of **six pure TypeScript functions** that convert, format, and organize AO3-style data into presentation-ready strings.

There is no Svelte in this file. No DOM. No reactive state. Just input → transformation → output, every single time.

### What Does "Pure Function" Mean?

A pure function has two guarantees:

1. **Same input → same output**, always. Given `"mature"`, `mapRating()` always returns `"Mature"`. Given `"1"`, `categoryLabel()` always returns `"Fandoms"`. No randomness, no external state lookup, no mutations.
2. **No side effects**. It doesn't modify globals, log to console, make network requests, or touch the DOM. It takes data in and puts transformed data out.

This matters because pure functions are trivially testable. You can write a unit test that calls `formatWords(1204116)` and asserts it equals `"1,204,116"` without needing a browser, a Svelte compiler, or a running server. They're the safest, most reliable code you can write.

### The Six Functions

#### 1. `mapRating(rating: string | null | undefined): string`

Converts a short AO3-style code to its full display name:

```typescript
const RATING_MAP: Record<string, string> = {
  general: 'General Audiences',
  teen: 'Teen And Up Audiences',
  mature: 'Mature',
  explicit: 'Explicit',
};

const DEFAULT_RATING = 'Not Rated';

export function mapRating(rating: string | null | undefined): string {
  if (!rating) return DEFAULT_RATING;
  return RATING_MAP[rating.toLowerCase()] ?? DEFAULT_RATING;
}
```

The `RATING_MAP` dictionary is a module-level constant (defined outside any function), shared across all callers. The function lowercases the input to handle case variations (some backends return `"TEEN"` instead of `"teen"`), looks it up, and falls back to `"Not Rated"` for null/undefined/unknown codes.

#### 2. `groupTags(tags: Tag[]): Record<string, Tag[]>`

Flattens a tag array into a category-keyed dictionary:

```typescript
export function groupTags(tags: Tag[]): Record<string, Tag[]> {
  const result: Record<string, Tag[]> = {};

  for (const tag of tags) {
    const key = String(tag.category ?? '4');
    if (!result[key]) result[key] = [];
    result[key].push(tag);
  }

  // Ensure all known categories exist in output
  for (const cat of CATEGORY_ORDER) {
    if (!result[cat]) result[cat] = [];
  }

  return result;
}
```

Notice how `groupTags` ensures all six category keys exist in its output, even if empty. This lets consumers iterate over all categories uniformly without checking for key existence. The blank-category default is `'4'` (Additional Tags) — stray tags without a category end up there.

You won't call `groupTags()` directly from `WorkBlurb` anymore — it moved into `rating.ts` as a standalone function, and `WorkBlurb` does the grouping inline with `$derived.by()`. But `groupTags()` remains useful for other parts of the codebase that need the same fan-out operation.

#### 3. `categoryLabel(category: string): string`

Returns the human-readable label for a category number:

```typescript
const CATEGORY_LABELS: Record<string, string> = {
  '1': 'Fandoms',
  '2': 'Characters',
  '3': 'Relationships',
  '4': 'Additional Tags',
  '5': 'Warnings',
  '6': 'Categories',
};

export function categoryLabel(category: string): string {
  return CATEGORY_LABELS[category] ?? 'Tags';
}
```

Used by `TagSoup` to render labels like **"Fandoms:"**, **"Characters:"**, etc. Falls back to generic `"Tags:"` for unknown category numbers. Simple lookup — zero branching logic.

#### 4. `orderedCategories(grouped: Record<string, unknown[]>): string[]`

Filters `CATEGORY_ORDER` to only include categories that actually contain tags:

```typescript
const CATEGORY_ORDER = ['1', '2', '6', '3', '4', '5'];

export function orderedCategories(grouped: Record<string, unknown[]>): string[] {
  return CATEGORY_ORDER.filter((c) => grouped[c]?.length);
}
```

If a fic has only fandom tags and additional tags, this returns `['1', '4']`. The AO3-standard order is preserved, blanks pruned. Called by `TagSoup` to determine which rows to render and in what order.

#### 5. `formatWords(words: number | null | undefined): string`

Comma-formats word counts:

```typescript
export function formatWords(words: number | null | undefined): string {
  if (words == null || isNaN(words)) return '0';
  return words.toLocaleString('en-US');
}
```

`1204116` → `"1,204,116"`. Uses the built-in `Intl.NumberFormat` through `toLocaleString()`. Falls back to `"0"` for null, undefined, or NaN inputs. This is why you see nice readable numbers in StatsLine instead of raw integers.

#### 6. `chaptersDisplay(chapters: number | null | undefined, status: string | null | undefined): string`

Builds the chapter completion indicator:

```typescript
export function chaptersDisplay(
  chapters: number | null | undefined,
  status: string | null | undefined
): string {
  const current = chapters ?? 0;
  const complete = status?.toLowerCase() === 'complete' || status?.toLowerCase() === 'completed';
  return complete ? `${current}/${current}` : `${current}/?`;
}
```

Works for two statuses: `"complete"` and `"completed"` (some backends use one, some use the other). Returns `"109/109"` for completed fics, `"5/?"` for ongoing ones. The question mark clearly signals uncertainty — "five chapters so far, who knows how many more."

#### 7. `formatUpdated(updated: string | null | undefined): string`

Converts ISO timestamps into relative time strings:

```typescript
export function formatUpdated(updated: string | null | undefined): string {
  if (!updated) return '';
  const date = new Date(updated);
  if (isNaN(date.getTime())) return '';
  const now = Date.now();
  const diff = now - date.getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  const years = Math.floor(months / 12);
  return `${years}y ago`;
}
```

This cascading time conversion follows a well-known pattern (used by GitHub, Medium, Twitter, and AO3's own timestamp utilities). The granularity narrows as time grows: seconds → minutes → hours → days → months → years. Each threshold is chosen pragmatically:

- Under 60 seconds → `"just now"` (fresh enough to be notable)
- Under 60 minutes → `"Xm ago"` (useful for activity monitoring)
- Under 24 hours → `"Xh ago"` (daily cycles matter)
- Under 30 days → `"Xd ago"` (weekly reading habits)
- Under 12 months → `"Xmo ago"` (monthly overview)
- Beyond → `"Xy ago"` (yearly perspective)

The function uses `Date.now()` to get the current time, so the displayed relative time changes dynamically as the user browses — no timestamp stored anywhere. This means "Updated: 3d ago" will always be correct regardless of when the page was originally loaded.

> **💡 Key Concept — Why Pure Functions?**
>
> `rating.ts` is pure. That means:
> - **Testability**: Call `formatWords(999)` and assert `"999"`. Done. No mocking, no fixtures, no Svelte testing harness.
> - **Reusability**: Any component can import `mapRating` or `formatUpdated` without importing Svelte, without setting up stores, without worrying about timing.
> - **Predictability**: Change the code and every caller sees the change immediately. No hidden coupling.
> - **Server compatibility**: These functions run identically on the server (SSR) and in the browser (CSR). No `window` access, no `document` calls.
>
> In Svelte projects, the temptation is to put formatting logic inside template expressions or `$derived` blocks. Resist that urge! Extract formatting into a `.ts` file of pure functions. Your template stays clean, your logic stays testable, and your components stay thin.

> **⚠️ Watch Out**
>
> `formatUpdated()` creates a `new Date()` and calls `Date.now()` on every invocation. For most components (a handful of StatsLines per page), this cost is negligible. But if you ever pass `formatUpdated()` into a `{#each}` loop with thousands of iterations, it will create thousands of Date objects. In that case, compute the "now" timestamp once outside the loop and pass it in as a parameter. Not a concern for current usage, but a gotcha to remember.

---

## Chapter 13: ArchiveWork — Fic Detail Page

Finally, the big one. `ArchiveWork.svelte` (`frontend/src/lib/ui/archive/ArchiveWork.svelte`, 407 lines) is the **largest content component** and the destination every fic card leads to. When a user clicks "Read" or the title on a `WorkBlurb`, they land here — a dedicated page displaying the full fic metadata, summary, download options, chapter list, and author notes.

Think of `WorkBlurb` as the business card and `ArchiveWork` as the whole biography. Compact at a glance, detailed on demand.

### The Props Interface

```svelte
<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import { mapRating, groupTags, categoryLabel, orderedCategories, formatWords, chaptersDisplay, formatUpdated } from './rating.js';
  import StatsLine from './StatsLine.svelte';
  import ArchiveButton from './ArchiveButton.svelte';
  import type { ExportResponse, FicMeta } from '$lib/api/types';

  let {
    fic,
    isBookmarked = false,
    onToggleBookmark,
    savingBookmark = false,
  }: {
    fic: ExportResponse;
    isBookmarked?: boolean;
    onToggleBookmark?: () => void;
    savingBookmark?: boolean;
  } = $props();
```

Unlike `WorkBlurb` which receives a lightweight `FicSearchResult`, `ArchiveWork` receives an `ExportResponse` — the full object from the export API (`GET /api/epub` or `GET /api/meta`). It has more fields, more depth, and crucially, download URLs.

Three extra props handle bookmarking state:

- **`isBookmarked`** — Whether this fic is already in the user's bookmarks. Defaults to `false` so the component can render independently of its parent's state.
- **`onToggleBookmark`** — A callback the parent provides to toggle the bookmark. Called when the user clicks the Bookmark button.
- **`savingBookmark`** — Loading indicator. Set to `true` during the network request so the button shows "Saving…" and disables itself.

### Accessing the Nested Meta Object

The `fic` object contains a nested `meta` property typed as `FicMeta | undefined`:

```typescript
const meta: FicMeta | undefined = $derived(fic.meta);
const m = $derived(meta!);
```

The non-null assertion (`!`) tells TypeScript "I know this is defined at runtime." In practice, `meta` should always be present for valid responses, but the type system can't prove that. The `m` alias gives us shorter references throughout the template: `m.title` instead of `fic.meta!.title`.

### Derived Metadata

The component derives several values from the raw metadata:

```typescript
const title = $derived(m.title ?? 'Untitled');
const author = $derived(m.author ?? 'Anonymous');
const ratingDisplay = $derived(mapRating(null));
const createdDate = $derived(m.created ? new Date(m.created).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }) : '');
const updatedDate = $derived(m.updated ? new Date(m.updated).toLocaleDateString('en-US', { year: 'numeric', month: 'long', 'day': 'numeric' }) : '');
const isLoggedIn = $derived(auth.isLoggedIn);
```

Note that `ratingDisplay` is called with `null` — the detail page doesn't use a rating badge like `WorkBlurb` does. The variable exists for potential future use but is currently unused.

The date formatting uses the native `Intl.DateTimeFormat` through `toLocaleDateString()` with options for a verbose American-style date: "January 15, 2024". This produces friendlier output than ISO strings for display purposes.

### The Download Dropdown

One of the most distinctive features of the detail page is the multi-format download dropdown:

```svelte
let downloads = $derived.by(() => {
  const out: { label: string; type: string; href?: string | null; lazy?: boolean }[] = [];
  const add = (label: string, type: string, href?: string | null, lazy = false) => {
    if (href) out.push({ label, type, href });
    else if (lazy) out.push({ label, type, href: null, lazy: true });
  };
  add('EPUB', 'epub', fic.epub_url);
  add('HTML', 'html', fic.html_url);
  add('TXT', 'txt', fic.txt_url);
  add('MD', 'md', fic.md_url);
  add('MOBI', 'mobi', fic.mobi_url, true);
  add('PDF', 'pdf', fic.pdf_url, true);
  add('AZW3', 'azw3', fic.azw3_url, true);
  add('DOCX', 'docx', fic.docx_url);
  add('FB2', 'fb2', fic.fb2_url);
  add('KEPUB', 'kepub', fic.kepub_url);
  return out;
});
```

This `$derived.by()` block builds the download menu on-the-fly from the URL fields on the `ExportResponse`. Each format is added via the `add()` helper:

- If `href` is truthy → the download is available, push with the URL.
- If `href` is falsy but `lazy` is `true` → the format exists but conversion hasn't started yet, push a disabled placeholder.
- If `href` is falsy and `lazy` is `false` → skip entirely (format not supported for this fic).

Some formats (MOBI, PDF, AZW3) are marked `lazy` because they require server-side conversion that runs asynchronously. They appear as "(unavailable)" in the dropdown until the conversion completes and the URL populates.

The toggle state is managed locally:

```typescript
let showDownloadMenu = $state(false);
let selectedFormat: string = $state('epub');
```

Clicking the Download button toggles `showDownloadMenu`:

```svelte
<button
  class="action-btn action-download"
  onclick={() => (showDownloadMenu = !showDownloadMenu)}
>
  Download ▾
</button>
```

And the menu renders conditionally:

```svelte
{#if showDownloadMenu}
  <div class="download-menu">
    {#each downloads as dl}
      {#if dl.href}
        <a class="download-item" href={dl.href} download onclick={() => { showDownloadMenu = false; }}>
          {dl.label}
        </a>
      {:else}
        <span class="download-item download-unavailable" title="Conversion not yet available">
          {dl.label} (unavailable)
        </span>
      {/if}
    {/each}
  </div>
{/if}
```

Each available format becomes a direct download link (`<a download>` triggers the browser's download dialog). Clicking one closes the menu (`onclick={() => { showDownloadMenu = false; }}`). Unavailable formats render as inert spans with a tooltip explaining the situation.

> **💡 Key Concept — Lazy Format Availability**
>
> Not all download formats are immediately available. EPUB, HTML, TXT, MD, DOCX, FB2, and KEPUB are generated synchronously during the initial export process. MOBI, PDF, and AZW3 require heavier server-side conversion that may complete seconds or minutes after the export starts. FicHub marks these as "lazy" — they appear in the dropdown as unavailable until the background conversion finishes. This is a classic async processing pattern: respond immediately with what's ready, surface delayed results when they arrive.

### Chapter Navigation

The detail page lists every chapter as a numbered link:

```svelte
<div class="work-chapters">
  <h3 class="section-heading">
    Chapters ({chaptersCount}{isComplete ? `/${chaptersCount}` : '/?'})
  </h3>
  <ol class="chapter-list">
    {#each Array.from({ length: chaptersCount }, (_, i) => i + 1) as chNum}
      <li class="chapter-item">
        <a class="chapter-link" href={`/read/${encodeURIComponent(m.id)}?chapter=${chNum}`}>
          Chapter {chNum}
        </a>
      </li>
    {/each}
  </ol>
</div>
```

The chapter list is generated dynamically using `Array.from()` — a common JavaScript idiom for creating N sequential numbers. `{ length: chaptersCount }` allocates an array of the right size, and `(_, i) => i + 1` generates `[1, 2, 3, ...]`. Each entry becomes a link to the corresponding chapter anchor.

The header shows `"Chapters (109/109)"` for completed works and `"Chapters (5/?)` for ongoing ones — the same convention used in `StatsLine`'s chapters display.

### Bookmark Button States

The bookmark button has three distinct states driven by the props:

```svelte
{#if isLoggedIn && onToggleBookmark}
  <button
    class="action-btn action-bookmark"
    class:bookmarked={isBookmarked}
    onclick={onToggleBookmark}
    disabled={savingBookmark}
  >
    {#if savingBookmark}
      Saving…
    {:else if isBookmarked}
      Bookmarked
    {:else}
      Bookmark
    {/if}
  </button>
{:else if isLoggedIn}
  <button class="action-btn action-bookmark" disabled>Bookmark</button>
{:else}
  <a class="action-btn action-bookmark" href="/">Log In to Bookmark</a>
{/if}
```

Three branches:

1. **Logged in + handler provided** — The full interactive button. Shows "Bookmark", "Bookmarked", or "Saving…" depending on the sub-state. The `class:bookmarked={isBookmarked}` applies a CSS modifier class when true, turning the button maroon.
2. **Logged in + no handler** — A disabled static button. The component received `isLoggedIn` but no callback, meaning the parent hasn't implemented bookmarking yet.
3. **Not logged in** — A link to the login page, styled consistently with the action buttons.

The `class:bookmarked` modifier maps to this CSS:

```css
.action-bookmark.bookmarked {
  color: var(--archive-bg, #ffffff);
  background: var(--archive-link, #990000);
  border-color: var(--archive-link, #990000);
}
```

A maroon button with white text — the same "primary" style as the Read Online button. Visual parity reinforces that bookmarking is a significant action, not a minor toggle.

### The Full Template Structure

```svelte
<article class="archive-work">
  <!-- Title & Byline -->
  <h2 class="work-title">{title}</h2>
  <p class="byline">by <a class="author-link">...</a></p>
  <p class="work-date">Added: ... · Updated: ...</p>

  <!-- Stats -->
  <div class="work-stats">
    <StatsLine words={m.words} chapters={m.chapters} status={m.status} updated={m.updated} />
  </div>

  <!-- Summary -->
  <div class="work-summary">
    <h3 class="section-heading">Summary</h3>
    <div class="summary-text">{@html m.description}</div>
  </div>

  <!-- Action Buttons -->
  <div class="work-actions">
    <a class="action-btn action-read">Read Online</a>
    <div class="download-wrapper">
      <button class="action-btn action-download">Download ▾</button>
      {#if showDownloadMenu}<div class="download-menu">...</div>{/if}
    </div>
    <!-- Bookmark button (3-state) -->
  </div>

  <!-- Chapters List -->
  <div class="work-chapters">...</div>

  <!-- Author Notes -->
  {#if fic.notes && fic.notes.length > 0}
    <div class="work-notes">
      {#each fic.notes as note}
        <p class="note-text">ℹ️ {note}</p>
      {/each}
    </div>
  {/if}
</article>
```

Six semantic regions flowing top-to-bottom:

1. **Identity** — Title, author, dates
2. **Statistics** — Single-line stats (reuse of `StatsLine`)
3. **Summary** — Author-provided description, rendered as raw HTML (`{@html}`) because AO3 summaries support paragraph breaks, links, and italics
4. **Actions** — Read, Download, Bookmark/Kudos
5. **Navigation** — Chapter list
6. **Notes** — Optional author-attribution or disclaimer text

### Using Pure Functions Inline

`ArchiveWork` imports seven functions from `rating.ts` but actively uses most of them:

```typescript
import { mapRating, groupTags, categoryLabel, orderedCategories, formatWords, chaptersDisplay, formatUpdated } from './rating.js';
```

Of these, `groupTags` and `categoryLabel` are imported but not actively called in the current version (they were part of the original component outline and remain available for future use). The actively used ones include `formatWords`, `chaptersDisplay`, and `formatUpdated` — though notably, `ArchiveWork` renders the stats section via the `StatsLine` component (Chapter 11), which internally calls those same functions. The duplicate import is a remnant of development: the detail page could call them directly, but chose the compositional path instead by embedding `StatsLine`.

### CSS Styling — Reading-Focused Layout

The CSS reflects the component's role as a *reading surface*, not a navigation surface. It uses Georgia serif throughout, generous spacing, and a centered max-width:

```css
.archive-work {
  max-width: var(--archive-max-width, 800px);
  margin: 0 auto;
  font-family: Georgia, 'Times New Roman', serif;
  color: var(--archive-text, #2a2a2a);
  line-height: 1.6;
}

.work-title {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 1.7em;
  font-weight: normal;
  color: var(--archive-heading, #990000);
  margin: 0 0 0.15em;
  border-bottom: 1px solid var(--archive-border, #dddddd);
  padding-bottom: 0.3em;
}

.section-heading {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 1.05em;
  font-weight: normal;
  color: var(--archive-text, #2a2a2a);
  border-bottom: 1px solid var(--archive-border, #dddddd);
  margin: 1.2em 0 0.4em;
  padding-bottom: 0.2em;
}
```

Key design decisions:

- **`max-width: 800px`** — Centers the content and limits line length for readability. Long lines (120+ characters) cause eye fatigue; 800px with 16px base font yields roughly 65 characters per line, the gold standard for text legibility.
- **Georgia serif** — Explicitly set on both the article and title. Georgia is a screen-optimized serif designed specifically for readability at small sizes. AO3 uses it for a reason.
- **Subtle dividers** — Section headings get a thin 1px bottom border, matching AO3's section delineators. No shadows, no backgrounds — just a quiet visual break.
- **No rounded corners anywhere** — The reading page continues the strict boxy aesthetic of the entire Archive mode.

> **💡 Key Concept — Separation of Concerns Across Components**
>
> `ArchiveWork` doesn't reimplement `StatsLine`'s formatting logic or build its own download dropdown from scratch. It *composes* existing components and functions:
> - Imports `StatsLine` and embeds it directly (reusing all formatting).
> - Imports the pure formatting functions from `rating.ts` (even if partially unused — they're available when needed).
> - Reuses `ArchiveButton`'s styling foundation (though the detail page leans toward raw `<button>` and `<a>` with custom classes for more control).
> 
> This is the compositional philosophy we saw in `WorkBlurb`, extended. Each layer adds just enough specificity — `WorkBlurb` is a compact card; `ArchiveWork` is a full reading page. Neither duplicates the other's knowledge.

> **⚠️ Watch Out**
>
> The summary renders with `{@html m.description}`, which injects raw HTML into the DOM. This is necessary because authors write summaries with paragraphs (`<p>`), italics (`<i>`), and links. However, always be cautious with `{@html}` — if the content came from an untrusted source, it could contain malicious scripts. In FicHub's case, descriptions come from scraped AO3/FanFiction.net content, which is generally safe. But if you ever accept user-submitted HTML (comments, reviews, custom fields), sanitize it first with a library like DOMPurify before using `{@html}`.

> **🧪 Try It Yourself**
>
> Navigate to a fic's detail page. Inspect the chapter list in developer tools — each entry is a plain `<li><a>` generated by `Array.from({ length: chaptersCount }, ...)`. Click on "Chapter 1" and watch the URL gain a query parameter: `?chapter=1`. That parameter drives the chapter anchor scroll. Now try clicking "Download ▾" — notice how the dropdown appears *below* the button, positioned with `position: absolute` and `z-index: 10`. Click an unavailable format (like MOBI) to see the italicized "(unavailable)" text with its tooltip.

---

*Part 3 covered the five core content components: `WorkBlurb` (the fic card), `TagSoup` (tag rows), `StatsLine` (inline statistics), `rating.ts` (pure formatting functions), and `ArchiveWork` (the full detail page). Together they form the content rendering pipeline — from search result list to individual fic page. In Part 4, we'll explore the search interface, where these components connect to the live API.*

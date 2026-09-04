# Part 4 — The Search System

Welcome to the search system! 🕵️‍♀️ You're going to learn how FicHub lets readers discover fanworks through powerful filtering, smart suggestions, and a beautifully organized search page. This is one of the most-used parts of the entire archive — people use it every single day to find their next favorite story. If you've ever spent hours scrolling through recommendations hoping something clicks, you know how frustrating it is when the *exact* search you want doesn't exist. That's why the search system matters so much at FicHub.

Think about what goes into a search. A reader might want stories tagged with "Marvel" and "Tony Stark," with at least 1,000 words and no more than 50,000, published in General Audiences rating, sorted by kudos. That's a lot of constraints — but behind the scenes, our system needs to translate all those choices into a precise database query. That translation chain starts here.

The search system has five main pieces:

1. **searchForm.ts** — a pure module that turns user input into backend queries
2. **searchForm.test.ts** — 47 tests that make sure nothing breaks
3. **ArchiveWorkSearchForm.svelte** — the giant two-mode component (853 lines!)
4. **Search Page (+page.svelte)** — combines results and filters in a two-column layout
5. **Search API** — the connection between frontend and backend

Each piece builds on the last, like layers in an onion. By the end of this part, you'll understand the complete pipeline from "user clicks a checkbox" to "results appear on screen."

Let's go!

---

## Chapter 14: searchForm.ts — The Pure Search Module

In this chapter, you'll meet the heart of the search system. Tucked away inside `frontend/src/lib/ui/archive/searchForm.ts` is a small but mighty module. Despite being just one file, it handles all the logic for turning what a user clicks, types, and checks into an actual search query that the backend understands. And here's what makes it special: it's *pure*. Everything in this module follows a rule called purity. A pure function is one where given the exact same inputs, you always get the exact same output, and nothing sneaky happens behind the scenes — no network calls, no database reads, no global variables getting changed. Just clean, predictable logic.

Think of it like cooking a measured recipe. You put in two cups of flour and one egg, and you get the exact same batter every time. Not a wobbly cake factory, a reliable kitchen. The search module works the same way — it's the recipe book that translates "I want Marvel stories over 5,000 words" into `{ fandoms: ["Marvel"], wordCountMin: 5000 }`.

### The Building Blocks: parseRange and buildSearchQuery

Let's start with the smallest piece first: `parseRange`. This is a utility function that takes a raw string from the user (for example, `"1000-5000"`) and converts it into a structured range object with `min` and `max` properties:

```typescript
// User typed these values into word-count input fields
const rawString = "5000-50000";
const range = parseRange(rawString);
// result: { min: 5000, max: 50000 }
```

If the user only fills in one side — say, just a minimum with `"5000-"` — `parseRange` gracefully handles it by setting the missing side to `null`. Why does this matter? Because behind the scenes, the backend expects structured numeric objects, not free-form text strings. Users don't think in terms of JSON shapes; they think in terms of "at least 5,000 words." `parseRange` bridges that gap beautifully.

Then comes the heavyweight champion: `buildSearchQuery`. This is the main function that takes the entire form state — every single field the user interacted with — and assembles it into a query object ready for the API:

```typescript
import { buildSearchQuery } from "$lib/ui/archive/searchForm";

const formState = {
  title: "Dragon",
  author: "FanficCreator",
  fandoms: ["Marvel Cinematic Universe", "Star Wars"],
  characters: ["Tony Stark", "Steve Rogers"],
  relationships: ["Peter Parker/MJ"],
  additionalTags: ["Time Travel", "Hurt/Comfort"],
  warnings: ["Graphic Depictions Of Violence"],
  categories: ["F/M"],
  ratings: ["General Audiences"],
  wordCount: { min: 1000, max: 50000 },
  kudosMin: 10,
  kudosMax: 500,
  commentsMin: 5,
  bookmarksMin: 3,
};

const query = buildSearchQuery(formState);
// query: { title: "Dragon", ...all non-empty fields... }
```

Notice something clever? Every field that the user *didn't* change simply doesn't appear in the output. The function filters out empty or unset values automatically. This keeps each request lean — fewer bytes traveling across the wire, faster queries running on the backend.

### What Fields Are Actually Supported?

Now here's where things get interesting — and honestly, a little surprising. Not everything you see on the search page actually filters results! The search system has a table of "supported" versus "inert" fields. Let me walk you through the complete roster:

| Field | Supported? | How It Works |
|-------|-----------|--------------|
| kudos | ✅ Partial | Only maps to `max_kudos`; no minimum support |
| comments | ✅ Supported | Both min and max controls fully functional |
| bookmarks\_min / bookmarks\_max | ✅ Supported | Independent minimum and maximum filters |
| word_count | ✅ Supported | Parsed through `parseRange` for flexible ranges |
| rating | ⚠️ Inert | Visible in UI but currently does nothing |
| crossovers | ⚠️ Inert | Renders with a no-op footnote explaining it's planned |
| hits | ⚠️ Inert | Another placeholder with a no-op footnote |
| language | ⚠️ Inert | Shown as a "coming soon" placeholder |

⚠️ **Watch Out for Inert Fields:** When a field is inert, it looks clickable on the screen, but nothing happens when you interact with it. This can be confusing for new users who expect every input to work. The best practice documented in the codebase is to mark these clearly with footnotes so visitors know they're waiting for future development. And hey — that could be you implementing them someday!

💡 **Key Concept: The Kudos Edge Case**
There's a subtle gotcha with the kudos field. The backend only provides a `max_kudos` parameter; there is no `min_kudos`. So even though the form shows both a "minimum kudos" and "maximum kudos" input, they actually both write to the same `max_kudos` parameter internally. This is recorded in the source code as a partial feature implementation. If you need stories with *at least* 100 kudos today, there's no direct way to express that constraint — you'd have to set the minimum to 100, which effectively becomes a maximum-of-100 instead. A real limitation worth noting!

### Converting Between Form State and URLs

FicHub has one of those features you take for granted until someone asks you to share a search. "How did you find that story?" "Oh, I just used this search." *Sends link.* "Wait, you mean I can copy a whole search into a URL and send it to people?" Exactly. Here's how:

The `formStateToUrl` function serializes a complete form state object into a compact URL-encoded query string. Then `urlToFormState` reverses the process, reading a URL and reconstructing the original form state:

```typescript
import { formStateToUrl, urlToFormState } from "$lib/ui/archive/searchForm";

// Serialize: form state → URL
const initialState = {
  title: "Dragon",
  wordCount: { min: 1000, max: 50000 },
  commentsMin: 10,
  sortBy: "kudos",
};

const url = formStateToUrl(initialState);
// Produces: "/search?q=title%3ADragon&min_words%3A1000&..."

// Deserialize: URL → form state
const recovered = urlToFormState(url);
// recovered.title === "Dragon" ✓
// recovered.wordCount === { min: 1000, max: 50000 } ✓
// recovered.commentsMin === 10 ✓
```

This serialization/deserialization round-trip is what makes deep-linking possible. Bookmark a search? Check. Email a friend your exact filtering setup? Done. Refresh your browser and lose nothing? Naturally. Without this capability, search functionality would feel fragile and frustrating.

### Adding Chips: mergeChipIntoForm

Now let's talk about chips. Remember those friendly suggestion tags that float above the search form? When logged-in users click one, it snaps into the active filter. Behind the scenes, a function called `mergeChipIntoForm` handles this:

```typescript
import { mergeChipIntoForm } from "$lib/ui/archive/searchForm";

let currentForm = { fandoms: ["Avengers"] };

currentForm = mergeChipIntoForm(currentForm, {
  field: "fandoms",
  label: "Spider-Man",
  reason: "chip-click",
});
// currentForm.fandoms === ["Avengers", "Spider-Man"]

// Clicking "Spider-Man" again does NOT create a duplicate:
currentForm = mergeChipIntoForm(currentForm, {
  field: "fandoms",
  label: "Spider-Man",
  reason: "chip-click",
});
// currentForm.fandoms === ["Avengers", "Spider-Man"]  ← still just 2!
```

The deduplication behavior is baked directly into `mergeChipIntoForm`. Before appending a new label, it checks if the value already exists in the target array. This prevents the common bug where repeated clicking creates `[ "Tag", "Tag", "Tag" ]` — a frustration that plagues poorly designed filter UIs everywhere.

💡 **Key Concept: Why Pure Functions Matter So Much**
You might reasonably ask: why go through all this trouble to make everything a pure function? The answer boils down to three powerful benefits:

1. **Testing is trivial.** Because pure functions have zero side effects, every test is a simple assertion: `assertEquals(myFunc(input), expectedOutput)`. No mocking frameworks, no spies, no fake timers. This is why the test suite can cover 47 cases in a single focused file.

2. **Behavior never surprises anyone.** Call `buildSearchQuery` with the same form state ten times, ten thousand times — identical output every single time. There's no hidden state bleeding between calls, no race conditions, no flakiness.

3. **Functions compose freely.** Any part of the application — the search page, the sidebar, a settings dialog, a URL parser — can independently call any of these functions. They don't care about each other's existence. That decoupling is architectural gold.

That wraps up searchForm.ts! You now understand how this slim but potent module transforms human-friendly form input into machine-readable search queries. In the next chapter, we'll watch 47 automated tests verify every line of this logic. Get ready for testing glory. 🧪

---

## Chapter 15: searchForm.test.ts — Testing the Module

Now that you've met searchForm.ts, let's look at its test suite: `searchForm.test.ts`. This file has **47 passing tests**, covering every single function in the pure module. You might wonder: why so many tests for a small file? Because even simple-seeming logic can hide surprising corner cases, and in search systems specifically, *incorrect filtering silently returns wrong results* — the worst kind of bug. A login button either works or it doesn't; a search filter returning 200 results when there are actually 20,000 is much harder to notice.

### The Test Philosophy: Given → When → Then

Because searchForm.ts is built entirely from pure functions, testing is remarkably straightforward. Each test follows a simple three-step rhythm: provide some input, call a function, check the output. No mocking networks, no spinning up databases, no fake timers or complicated setup. It's clean and direct:

```
Given: these inputs
When: I call this function with them
Then: I expect this exact output
```

This pattern makes the tests lightning-fast to run — we're talking milliseconds, not seconds. Fast tests encourage developers to run them frequently, which means bugs get caught early rather than festering until release day. If you've ever tried adding a feature to a codebase where running the test suite takes five minutes, you know exactly why speed matters. Here, the feedback loop is nearly instant.

### Range Parsing Tests: Where Numbers Begin

These tests exercise `parseRange`, checking it handles every flavor of user input gracefully:

```typescript
// Valid two-sided ranges
expect(parseRange("100-1000")).toEqual({ min: 100, max: 1000 });
expect(parseRange("1000-1000")).toEqual({ min: 1000, max: 1000 });

// One-sided ranges — user omits one bound
expect(parseRange("100-")).toEqual({ min: 100, max: null });
expect(parseRange("-5000")).toEqual({ min: null, max: 5000 });

// Empty string → nothing useful
expect(parseRange("")).toEqual({ min: null, max: null });
```

Tests like these catch the edge cases that cause real bugs. What happens if someone accidentally submits an empty string? What if min and max are the same number (a common scenario when a user wants *exactly* N words)? The parseRange function must handle all of these without throwing exceptions. Every case listed above has its own dedicated test asserting the expected behavior.

### Query Building Tests: Ensuring Clean Output

These tests verify that `buildSearchQuery` assembles query objects correctly:

```typescript
// Single field present → only that field appears in output
expect(buildSearchQuery({ title: "test" })).toHaveProperty("title", "test");

// Completely empty input → completely empty output
expect(buildSearchQuery({})).toEqual({});

// Multiple fields → all appear, none missing
const result = buildSearchQuery({
  title: "Dragon",
  kudosMin: 50,
  commentsMax: 100,
});
expect(result.title).toBe("Dragon");
expect(result.kudosMin).toBe(50);
expect(result.commentsMax).toBe(100);
```

An empty state returning an empty query object is an intentionally designed property — it means unused fields never pollute the request payload. This keeps network traffic lean and ensures the backend receives *only* the filters the user explicitly set. Without this clean-slate behavior, you'd end up sending useless defaults down the wire.

### URL Round-Trip Tests: Sharing Searches Works

These tests verify the full serialization round-trip — form state into a URL and back again:

```typescript
const original = {
  title: "Dragon",
  ratings: ["Explicit"],
  bookmarks_min: 10,
  sortBy: "date",
  sortOrder: "desc",
};
const url = formStateToUrl(original);
const restored = urlToFormState(url);
expect(restored).toEqual(original);
```

Notice the emphasis on `.toEqual()` rather than strict equality — both structures must have matching keys and values. If any field survives the transformation incorrectly, the test fails immediately. Passing this suite guarantees that bookmarking a search and reopening it later shows the exact same configuration. Try this yourself: perform a complex search, copy the URL, paste it in a new tab. Same filters, same everything. These round-trip tests prove that magic works.

### Chip Merging Tests: Deduplication Discipline

These tests make sure `mergeChipIntoForm` both adds new chips *and* rejects duplicates:

```typescript
const base = { fandoms: ["Avengers"] };
const merged = mergeChipIntoForm(base, {
  field: "fandoms",
  label: "Spider-Man",
  reason: "chip-click",
});
expect(merged.fandoms).toEqual(["Avengers", "Spider-Man"]);

// Clicking the same chip again does NOT create a duplicate
const alreadyThere = mergeChipIntoForm(merged, {
  field: "fandoms",
  label: "Spider-Man",
  reason: "chip-click",
});
expect(alreadyThere.fandoms.length).toBe(2);  // Still exactly 2
```

The deduplication check happens *inside* `mergeChipIntoForm` — before appending a new value, it scans the existing array for that label. This prevents the classic UI bug where repeated clicking accumulates `[ "Tag", "Tag", "Tag" ]`. That's a frustrating mistake that users encounter in poorly tested software everywhere, and these explicit tests make sure FicHub stays cleaner.

### Inert Field Tests: Verifying Nothing Happens

Perhaps surprisingly, there are dedicated tests confirming that inert fields *don't* influence the output:

```typescript
const result = buildSearchQuery({
  rating: "Teen",          // Inert — ignored
  crossover: true,         // Inert — ignored
  hitCount: 5000,          // Inert — ignored
  language: "en",          // Inert — ignored
});
// None of these properties should appear in the result
expect(result.rating).toBeUndefined();
expect(result.crossover).toBeUndefined();
```

Why dedicate test lines to proving nothing happens? Because these are *behavioral contracts*. Anyone reading the source code can see "this field is inert," but the tests guarantee it stays inert even when other developers refactor surrounding code. Documentation in code is aspirational; tests are enforceable.

### Running the Tests Locally

To run these tests during development:

```bash
cd frontend
npm test -- searchForm.test.ts
```

This runs only the 47 tests in this specific file — ideal when you're actively editing searchForm.ts and want rapid feedback. To run the complete project test suite:

```bash
cd frontend
npm test
```

All 47 tests should flash through green ✅ in under a second. If any test fails, the runner reports the failing assertion with the expected versus actual values side by side. This immediate clarity is one of the enormous advantages automated tests have over manual quality assurance — you never have to guess what went wrong.

⚠️ **Watch Out: Test Coverage Awareness**
Forty-seven tests across five public functions averages roughly nine tests per function — solid coverage, but not exhaustive. Some less-common code paths inside `buildSearchQuery` may still lack dedicated assertions. Watch for branches where conditional logic exists but no test exercises it. Over time, every new feature should arrive with its own test(s) — it's much easier to add tests alongside new code than to retrofit them later.

That wraps up searchForm.test.ts! Those 47 tests stand guard over your pure module, ensuring correctness at every level. Now let's scale up to the massive component that brings all this logic to life. 🔧

---

## Chapter 16: ArchiveWorkSearchForm — The Two-Mode Component

🏗️ Here it is — the largest file in the entire archive frontend: **ArchiveWorkSearchForm.svelte**, clocking in at **853 lines**. This single Svelte component brings together everything from searchForm.ts and renders the beautiful filtering interface that FicHub users rely on.

But here's the cool part — this component plays in two different modes, adapting its shape depending on where it's placed on the page.

### Mode 1: Full Page Mode (`mode="page"`)

In page mode, the form is the star of the show. The screen is dedicated entirely to search configuration. The form is organized into three `<fieldset>` sections, each grouping related fields:

**First Fieldset: Work Info**

This section handles basic text metadata:
- **Title** — enter the title or part of a title
- **Author/Audience** — filter by creator username
- **Chapters Status** — choose completed, ongoing, or any
- **Word Count** — min and max word count inputs

```html
<!-- Simplified structure -->
<fieldset>
  <legend>Work Info</legend>
  <label>Title
    <input bind:value={state.title} />
  </label>
  <label>Author
    <input bind:value={state.author} />
  </label>
  <!-- More fields... -->
</fieldset>
```

Each `<label>` wraps an `<input>`, using Svelte's `bind:value` directive to keep the form state in sync with the DOM in real time. No event listeners needed!

**Second Fieldset: Work Tags**

This section is where the richness of FanFiction tags shines. Each row has a select dropdown:
- Fandoms
- Characters
- Relationships
- Additional Tags
- Warnings
- Categories

```html
<fieldset>
  <legend>Work Tags</legend>
  <label>Fandoms
    <select multiple bind:value={state.fandoms}>
      <option value="Marvel">Marvel</option>
      <option value="Star Wars">Star Wars</option>
      <!-- hundreds more... -->
    </select>
  </label>
  <label>Characters
    <select multiple bind:value={state.characters}>
      <!-- characters populate dynamically -->
    </select>
  </label>
  <!-- More tag rows... -->
</fieldset>
```

Multiple selection (via `multiple` attribute) lets users pick many fandoms or characters at once. Behind the scenes, each selected option gets pushed into the corresponding array in the `$state` object. But here's a subtlety that makes this component special: the select dropdowns don't contain static options like a simple HTML form would. Instead, they populate from a large dataset of known tags pulled during component initialization. When ArchiveWorkSearchForm mounts, it fires off requests to fetch available values for fandoms, characters, relationships, and every other tag type. Those arrays live in Svelte stores and feed directly into `<option>` elements through reactive bindings. The result is that users see exactly the fandoms and characters that exist in the FicHub database — no typos, no ghost entries — which dramatically reduces search frustration.

**Third Fieldset: Work Stats**

The final group deals with numeric metrics and sorting:
- **Kudos:** min and max inputs
- **Comments:** min and max inputs  
- **Bookmarks:** min and max inputs
- **Rating:** radio buttons (General Audiences, Teen, Mature, Explicit)
- **Sort Options:** radio buttons (date, kudos, hits, reviews, chapters)

💡 **Key Concept: Radio Buttons vs. Inputs**
Ratings and sort options use `<input type="radio">` because you only want one choice at a time. Radio buttons enforce mutual exclusion automatically — selecting "Mature" deselects "Teen." Numeric fields use number inputs because you're specifying values, not choosing from a fixed list.

### Mode 2: Sidebar Mode (`mode="sidebar"`)

Sidebar mode is where things get really clever. It reuses the *exact same form fields* — every single input, every select dropdown, every radio button — but it wraps them in a completely different visual packaging. Instead of big visible `<fieldset>` blocks that dominate the screen, each group lives inside a collapsible `<details>/<summary>` HTML element:

```html
<details>
  <summary>Work Info</summary>
  <div class="sidebar-filters">
    <!-- Same fields, but collapsed by default -->
    <label>Title
      <input bind:value={state.title} />
    </label>
  </div>
</details>

<details>
  <summary>Work Tags</summary>
  <!-- Collapsed by default -->
</details>
```

Notice something subtle? The `bind:value` directive still works perfectly. Even though these inputs live inside collapsed `<details>` sections that users must explicitly open to interact with, Svelte's two-way binding keeps `$state` fully synchronized exactly as it does in page mode. There's no separate data channel for sidebar inputs — it's the same reactive plumbing underneath. This means you can build a massive component once and render it anywhere without duplicating event handlers or state logic.

The key behavioral difference: `<details>` elements are *closed* by default in sidebar mode but *open* by default in page mode. Users click the summary headers to expand individual sections. Why design it this way? Because in sidebar mode, the form occupies precious vertical real estate alongside an already-dense results list. Showing everything expanded would push results off-screen and force constant scrolling. Collapsing reduces the form to just a row of clickable headers until the user actively needs a filter.

But there's one exception to the collapse rule: the **sort options** section never collapses. In sidebar mode, sort controls are pinned above all the `<details>` blocks so they're always immediately accessible. This reflects an important UX observation from actual usage data — changing sort order is one of the most frequently repeated actions users perform while browsing search results. Making it persistently visible saves dozens of clicks per session across the entire user base.

Side-by-side comparison:

| Feature | Page Mode | Sidebar Mode |
|---------|-----------|--------------|
| Layout | Full-width `<fieldset>` rows | Compact `<details>/<summary>` blocks |
| Visibility | Always open | Collapsible (closed by default) |
| Sort options | At bottom of stats section | Always visible at top (sticky) |
| Use case | Standalone search page | Paired with results list |
| Scroll behavior | Form scrolls naturally with page | Form panel pinned via CSS `position: sticky` |
| Initialization cost | Loads tag selects eagerly | Also loads tags eagerly (shared initialization) |

💡 **Key Concept: DRY Architecture Through Parameterized Rendering**
One file doing double duty like this is the essence of DRY (Don't Repeat Yourself) in software. The ArchiveWorkSearchForm component accepts a `mode` prop that determines its rendering strategy — it doesn't fork into two separate files. Every field definition exists exactly once, which means fixing a bug in "how kudos min works" fixes it in both page and sidebar simultaneously. Without this parameterization pattern, maintaining two copies of nearly identical forms would be a maintenance nightmare and a breeding ground for inconsistencies.

### The Chips Row

Above (or below, depending on mode) the form, if the user is logged in, a row of chip buttons appears. These chips come from the `/api/search/suggest` endpoint:

```javascript
// Fetch suggested tags/chips
const response = await fetch("/api/search/suggest");
const chips = await response.json();
// Shows up to 8 chip buttons
```

Each chip button displays a tag name. Clicking one triggers `mergeChipIntoForm`, which adds the tag to the appropriate field. The chips help users discover relevant tags without knowing them by heart — it's like autocomplete for fandom culture!

### Disabled Fields (The "Coming Soon" Section)

Three fields appear in the form but don't actually work yet:

1. **Crossovers** — has a no-op footnote explaining it's planned but not implemented
2. **Hits** — similarly shown with a no-op footnote
3. **Language** — displayed as a placeholder labeled "coming soon"

⚠️ **Watch Out: User Frustration Prevention**
These disabled fields could frustrate users who try to use them and nothing happens. The footnotes help clarify that these are upcoming features. But ideally, disabled fields should be visually distinct (grayed out, non-clickable) so the user immediately understands they're not active yet.

### The Data Flow: From Click to Navigation

Here's the complete lifecycle of a search submission:

```
1. User clicks "Search" button
   ↓
2. onSubmit fires → calls buildSearchQuery(state)
   ↓
3. Query object is built from current $state
   ↓
4. navigate() redirects to /search?q=<encoded-query>
   ↓
5. URL updates in browser address bar
   ↓
6. search/+page.svelte picks up the "q" parameter
   ↓
7. Results render on the left panel
```

💡 **Key Concept: Svelte's $state Magic**
The form state lives in a `$state` variable (Svelte 5's reactive state declaration). Every time a user interacts with any input — typing, selecting, checking a checkbox — the `$state` object updates instantly. Svelte's reactivity system ensures all bindings stay synchronized automatically. No manual event handling, no Redux store updates, no complexity.

That's ArchiveWorkSearchForm! In 853 lines, it packs two complete layouts, dozens of filters, chip support, and a clean data flow. Up next: how the search page ties it all together. 🎬

---

## Chapter 17: Search Page Integration

With the form component done, it's time to put it on a real page! The search page lives at `frontend/src/routes/search/+page.svelte`, and it's responsible for bringing together the search results and the search form in a single cohesive experience.

### The Decision Tree: What Does the User Want?

When someone lands on `/search`, the page has to answer a simple question: **Are they searching, or just browsing?** The answer comes from the URL parameters:

```svelte
<script>
  import { page } from '$app/stores';
  import ArchiveWorkSearchForm from '$lib/ui/archive/ArchiveWorkSearchForm';
  import SearchResults from '$lib/components/search/SearchResults';
  
  // Read the "q" parameter from the URL
  const q = $page.url.searchParams.get('q');
  // Or read individual filter params directly
  
  let filters = {};
  if (q) {
    // Parse the encoded query string
    filters = decodeFilters(q);
  }
</script>
```

The route checks for a `q` query parameter:
- **No `q` parameter** → the user wants a fresh search form
- **Has `q` parameter** → the user wants to see search results

### Scenario A: Fresh Search (No Query Parameter)

Without a `q` parameter, the page shows the full-page search form. This is the welcoming entrance — blank canvas, ready to receive the user's wishes:

```svelte
{#if !q}
  <ArchiveWorkSearchForm mode="page" on_submit={handleSearch} />
{/if}
```

The form takes up most of the screen. Users fill in whatever filters they want and hit Submit. Behind the scenes, `buildSearchQuery` converts the state, then the page navigates to `/search?q=...`, triggering Scenario B.

### Scenario B: Actual Search (Query Parameter Present)

With a `q` parameter, the page transforms into a powerful two-panel dashboard:

```svelte
{#if q}
  <div class="search-layout">
    <div class="results-panel">
      <SearchResults filters={filters} />
      <Pagination total={response.total} />
    </div>
    
    <div class="form-panel">
      <ArchiveWorkSearchForm 
        mode="sidebar" 
        state={restoredState}
        on_submit={handleSearch} 
      />
      
      <ChipsRow if:isLoggedIn />
    </div>
  </div>
{/if}
```

Left column: Search results listing (work cards with titles, authors, summary snippets, kudos counts, chapter info). Right column: the sidebar form, still active and editable. Bottom: pagination controls.

💡 **Key Concept: Live Editing**
Because the sidebar form shares state with the results display, users can change a filter and see results update without refreshing the page. This live-editing behavior is a hallmark of a great search UX. Try increasing the word count minimum while looking at results — the list refreshes immediately.

### Pagination

Below the results, pagination helps users browse beyond the first page:

```svelte
<Pagination 
  total={response.total}
  currentPage={page.number}
  perPage={page.per_page}
/>
```

The `total` field tells us how many matching works exist. Combined with `per_page` (usually 20), the pagination calculates how many pages to show. Navigation buttons let the user jump between pages without resubmitting the whole form. This works because the search state lives in the URL itself — changing pages simply updates the query parameter from `?page=1` to `?page=2`, and Svelte automatically re-fetches the appropriate chunk of results. The beauty here is that the browser's back button works naturally: clicking it returns to page 1, forward returns to page 2. Users who accidentally click too deep can always navigate their history.

⚠️ **Watch Out for Edge Cases in Pagination:** When a search yields exactly zero results, the Pagination component must handle gracefully without crashing or displaying "Page 1 of NaN." The component checks for a valid `total` before rendering navigation buttons. Similarly, when only one page of results exists, all pagination controls hide entirely — no point showing a single-button pager. These edge cases seem minor but matter enormously for polish: a blank screen with "No results found" reads far better than a broken page counter.

### The Chips Row Below the Form

When the user is logged in, a chips row renders below the sidebar form. These chips are fetched from `/api/search/suggest` and show up to 8 clickable tags. Clicking a chip adds it to the active filter set and the results update:

```svelte
{#if isLoggedIn}
  <div class="chips-row">
    {#each chips as chip}
      <button class="chip-btn" on:click={() => addChip(chip)}>
        {chip.label}
      </button>
    {/each}
  </div>
{/if}
```

### CSS Layout: The Two-Column Magic

The layout uses modern CSS grid or flexbox to split the page:

```css
.search-layout {
  display: grid;
  grid-template-columns: 2fr 1fr;  /* results get 2/3, form gets 1/3 */
  gap: 2rem;
  padding: 1rem;
}

.results-panel { min-width: 0; }
.form-panel { position: sticky; top: 1rem; }
```

The results panel gets `min-width: 0` — a small CSS detail that actually does something important. Without it, long work titles inside result cards can overflow their container and break the grid layout entirely. Setting `min-width: 0` tells the browser "this element *may* shrink below its content's natural width," which lets the two-column layout breathe properly even when someone has a story titled "An Incredibly Long Title That Goes On And On Forever And Probably Includes Special Characters And Numbers Too."

The form panel is made sticky so it stays visible while scrolling through long result lists. As users scroll down page after page of results, the sidebar form follows along — they never have to scroll back up to change filters. This is a small detail that makes a big difference for usability, especially on searches with hundreds of results spanning dozens of pages.

💡 **Key Concept: Loading States**
Every transition between scenarios needs a loading state. When the user submits a search, the results panel should show a spinning loader rather than a blank space. The search page achieves this by watching the API response status and rendering a skeleton UI during the wait. Skeleton screens — gray placeholder blocks shaped like upcoming results — keep the page looking active even while data travels across the network. This prevents the jarring experience of watching a screen flicker from empty to populated in a single flash.

⚠️ **Watch Out: Mobile Responsiveness**
On narrow screens (phones, tablets), the two-column layout needs to stack into a single column. The results panel goes first (since it's what the user came for), and the sidebar form slides below it. Good responsive design ensures the search experience works on all devices.

### The Complete Search Lifecycle

Let me walk through the full journey one more time, from click to results:

1. **User opens** `/search` with no parameters
2. **Full-page form** appears with all filters available
3. **User fills in** Title="Dragon", Word Count min=10000
4. **User clicks Submit**
5. **onSubmit handler** calls `buildSearchQuery(state)` → `{ title: "Dragon", word_count: "10000-" }`
6. **Navigator redirects** to `/search?q=title%3ADragon%2Bwordcount%3A10000-`
7. **+page.svelte detects** the `q` parameter
8. **Two-column layout** renders: results on left, sidebar form on right
9. **Search API client** calls `search(filters)` with the parsed filters
10. **Results list** populates with matching works
11. **Chips row** suggests related tags below the form
12. **User tweaks** filters → results update → repeat until satisfied

💡 **Key Concept: Progressive Enhancement**
The search page degrades gracefully. If JavaScript fails, the form still submits via traditional POST and returns results on a fresh page load. If the API is down, friendly error messages tell the user what happened instead of showing broken UI. This resilience is what separates amateur apps from professional ones.

That's the search page! You've seen how routing, layout, forms, and data fetching come together. Now let's peek at the final layer: the API connection. 🔌

---

## Chapter 18: Search API & Backend Connection

We've explored the UI components thoroughly. Now let's follow the trail one step further — how does the frontend talk to the backend when actually executing a search?

### The SearchFilters Interface

At the bridge between frontend and backend sits the `SearchFilters` interface, defined in `frontend/src/lib/api/search.ts`:

```typescript
interface SearchFilters {
  title?: string;
  author?: string;
  fandoms?: string[];
  characters?: string[];
  relationships?: string[];
  additionalTags?: string[];
  warnings?: string[];
  categories?: string[];
  ratings?: string[];
  wordCountMin?: number;
  wordCountMax?: number;
  kudosMin?: number;
  kudosMax?: number;
  commentsMin?: number;
  commentsMax?: number;
  bookmarksMin?: number;
  bookmarksMax?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  page?: number;
  perPage?: number;
}
```

This interface defines every possible search parameter. Each field maps directly to a column or index in the backend database. If the user didn't touch the Comments min field, that property simply won't appear in the object — keeping requests lean.

### The Search Function

The main entry point is the `search()` function:

```typescript
import { search } from '$lib/api/search';

const response = await search({
  title: 'Dragon',
  wordCountMin: 1000,
  sortBy: 'kudos',
  sortOrder: 'desc',
  page: 1,
});
```

This function takes a `SearchFilters` object and sends it to the backend API endpoint. The actual HTTP request might look like:

```
GET /api/v1/search?q=title:Dragon&min_words:1000&sort=kudos:desc&page=1
```

The `search()` function handles encoding, error handling, and JSON parsing automatically.

### The SearchResponse Shape

After sending the request, the backend responds with a rich `SearchResponse` object:

```typescript
interface SearchResponse {
  total: number;       // Total matches across all pages
  page: number;        // Current page number
  per_page: number;    // Items per page
  results: WorkItem[]; // Array of matching works
  facets: Facets;      // Breakdown statistics
}

interface WorkItem {
  id: number;
  title: string;
  author: string;
  summary?: string;
  fandoms: string[];
  kudosCount: number;
  commentCount: number;
  bookmarkCount: number;
  wordCount: number;
  chapterCount: number;
  status: 'Completed' | 'Ongoing' | 'Hiatus';
  publishedAt: string;
  updatedAt: string;
  // ... many more fields
}
```

💡 **Key Concept: Pagination Metadata**
The `total`, `page`, and `per_page` fields are essential for building accurate pagination. With `total=147` and `per_page=20`, the UI knows to show 8 pages. Changing `page=2` in the next request fetches results 21–40.

### Rating Resolution Magic

Ratings deserve special attention. When the frontend sends a rating like "Explicit," the backend internally resolves it to **tag type 7** (the internal classification for ratings in the archive's taxonomy). This mapping is handled transparently:

```typescript
// Frontend sends:
{ ratings: ["Explicit"] }

// Backend resolves internally:
// Tag type 7 = "Explicit" in the rating vocabulary
// Query becomes: WHERE tag_type_id = 7 AND tag_name IN ('Explicit')
```

This indirection allows the frontend to speak human-readable names while the backend works with normalized tag IDs. If the archive decides to rename a rating in the future, only the resolution mapping needs updating — the frontend code stays untouched. This decoupling is one of those quiet architectural decisions that saves enormous headaches during long-term maintenance. Imagine wanting to change a label from "Mature" to "M for Mature" across the entire site; with direct numeric references you'd need a global find-and-replace through dozens of files. With this indirection layer, you update one lookup table and everything else follows.

💡 **Key Concept: The Facets Response Format**
Facets aren't just metadata — they're structured data designed specifically for interactive filtering. Each facet entry carries both a `label` (human-readable) and a `count` (how many results match that tag *given current filters*). This means facets are relative, not absolute. If you search for "Marvel" and then refine by "Time Travel," the facet counts for remaining tags will shift because they reflect the filtered universe, not the full database. Some APIs return static facet counts computed before any filtering happens — which would be misleading. FicHub's facets adapt dynamically based on what the user has already selected.

### Facets: Breaking Down Results

Here's a power feature that makes the search experience really useful: **facets**. After running a search, the backend returns a `facets` object containing breakdowns:

```typescript
const facets = response.facets;
/* Example structure:
{
  fandoms: [
    { label: "Marvel Cinematic Universe", count: 4200 },
    { label: "DCU", count: 1800 },
    { label: "Star Wars", count: 3200 },
    // ...
  ],
  characters: [
    { label: "Tony Stark", count: 5600 },
    { label: "Steve Rogers", count: 4100 },
    // ...
  ],
  warnings: [
    { label: "Graphic Depictions Of Violence", count: 1200 },
    // ...
  ],
  categories: [
    { label: "F/M", count: 8900 },
    { label: "M/M", count: 6700 },
    // ...
  ]
}
*/
```

Facets answer the question: "Among the results I'm seeing, how many belong to each category?" This is incredibly powerful for exploration. Imagine finding 5,000 dragon-themed fics — the facets tell you how many are Marvel vs. Star Wars vs. Harry Potter without leaving the results page.

Facets power the sidebar suggestions and are often displayed alongside results to help users refine their search directionally:

```
Showing 5,000 results for "dragon"
Top fandoms: Marvel (4,200) • Star Wars (3,200) • DCU (1,800)
Click any to add as a filter!
```

⚠️ **Watch Out: Facet Performance**
Computing facets is expensive — the backend must scan through all matching works to count distribution across categories. For searches with millions of results, this can take significant time. The API may include a `facets_delayed` flag or limit facets to the top N entries to avoid timeouts.

### The Complete Request Pipeline

Let's trace one more time, from the UI down to the wire:

```
┌─────────────────────────────────┐
│  User clicks Search             │
│  ArchiveWorkSearchForm          │
│  (Chapter 16)                   │
├─────────────────────────────────┤
│  buildSearchQuery(state)        │
│  searchForm.ts (Chapter 14)     │
│  Produces: {title:"...",kudos:..}│
├─────────────────────────────────┤
│  navigate(/search?q=...)        │
│  Router updates URL             │
├─────────────────────────────────┤
│  search/+page.svelte            │
│  Detects q param (Chapter 17)   │
│  Calls search(filters)          │
├─────────────────────────────────┤
│  search(API client)             │
│  search.ts (this chapter)       │
│  Sends GET /api/v1/search?...   │
├─────────────────────────────────┤
│  Backend receives request       │
│  Queries PostgreSQL + Elasticsearch│
│  Computes facets                │
├─────────────────────────────────┤
│  Returns SearchResponse         │
│  {total, page, results[], fac..}│
├─────────────────────────────────┤
│  SearchResults renders          │
│  Cards, facets, pagination      │
└─────────────────────────────────┘
```

💡 **Key Concept: Separation of Concerns**
Every layer has one job:
- **searchForm.ts**: transform state → query
- **test suite**: verify correctness
- **ArchiveWorkSearchForm**: collect user input
- **Search Page**: orchestrate layout and state
- **API client**: communicate with backend
- **Backend**: query database and return results

None of these layers depend on more than they need. Swap the backend? Change the UI? The interfaces stay the same, only internals change. That's solid software architecture! 🎓

---

## Wrapping Up Part 4

You did it! You've walked through the entire search system of FicHub's archive frontend:

- **Chapter 14** taught you pure functions (searchForm.ts) that convert user input into queries
- **Chapter 15** showed you how 47 tests keep that module bulletproof
- **Chapter 16** revealed the 853-line two-mode component that powers the search experience
- **Chapter 17** connected the form to the results page with responsive layouts
- **Chapter 18** traced requests all the way to the backend and back

The search system is one of the most complex pieces of FicHub, and now you understand how it fits together. Every click, every filter, every chip tag flows through the pipeline we just studied. Next up: Part 5 will cover the Collection System — how users organize their favorite works into curated shelves! 📚
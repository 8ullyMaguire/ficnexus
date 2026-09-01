# Part 9: Svelte 5 Deep Dive

*Going beyond the basics — mastering the runes, understanding the compiler, and writing idiomatic Svelte 5.*

---

## Chapter 37: Advanced Runes and Reactivity

### Beyond the Basics: When Simple Isn't Enough

In Chapter 7, we learned the fundamentals of Svelte 5 runes: `$state`, `$derived`, `$effect`, `$props`, and `$bindable`. You know how to create reactive variables, compute derived values, and run side effects. That's enough to build most applications.

But FicHub has some tricky parts. The search syntax parser needs to handle complex user input. The recommendations tab sorts results by a combined score that depends on multiple factors. The suggestions tab does optimistic updates — updating the UI immediately before the server confirms the change. These patterns require a deeper understanding of how runes work under the hood.

In this chapter, we're going to explore advanced rune patterns that will make you a Svelte 5 power user.

### $state and Deep Reactivity

One of the most important things to understand about `$state` is how it handles objects and arrays. When you create a reactive object, Svelte creates a **deeply reactive proxy**. This means changes to nested properties are automatically tracked.

```svelte
<script>
  let user = $state({
    name: 'Alice',
    preferences: {
      theme: 'dark',
      fontSize: 14,
      notifications: {
        email: true,
        push: false
      }
    }
  });
</script>

<!-- All of these trigger updates: -->
<p>{user.name}</p>
<p>{user.preferences.theme}</p>
<p>Email notifications: {user.preferences.notifications.email}</p>

<button onclick={() => user.preferences.theme = 'light'}>
  Toggle Theme
</button>
<button onclick={() => user.preferences.notifications.email = false}>
  Disable Email
</button>
```

When you click "Toggle Theme", only the parts of the template that use `user.preferences.theme` re-render. Svelte is smart enough to track exactly which properties you read, so it can be surgical about updates. This is called **fine-grained reactivity**, and it's one of Svelte's greatest strengths.

Here's a real example from FicHub. The search filters are a deeply nested object:

```typescript
// lib/api/search.ts
export interface SearchFilters {
  q: string;
  include_tags: string;
  exclude_tags: string;
  min_words: number | null;
  max_words: number | null;
  complete: boolean | null;
  source: string;
  sort: string;
  page: number;
  per_page: number;
}
```

When a user changes just the sort order, only the part of the UI that displays the sort dropdown needs to update. The search results list doesn't need to re-render. The word count filter doesn't need to update. Svelte tracks the dependency graph automatically.

### $state and Structured Clone

When you need to create a fresh copy of a deeply nested reactive object, use `$state.snapshot()`:

```svelte
<script>
  let filters = $state({
    q: '',
    min_words: null,
    tags: ['Harry Potter', 'Draco Malfoy']
  });

  function resetFilters() {
    // This creates a plain (non-reactive) snapshot
    const snapshot = $state.snapshot(filters);
    console.log(snapshot); // { q: '', min_words: null, tags: [...] }
    
    // Reset to defaults
    filters.q = '';
    filters.min_words = null;
    filters.tags = [];
  }
</script>
```

The snapshot is useful when you need to pass the current state to a function that expects a plain object — like sending it to an API or storing it in a log. It's also useful when you need to compare the current state with a previous state.

### $derived: Computing with Confidence

The `$derived` rune is perfect for simple computations, but sometimes you need to compute something complex — multiple steps, conditionals, loops. That's where `$derived.by()` comes in.

```svelte
<script>
  let recs = $state([]);
  let communityWeight = $state(0.1);
  let sortBy = $state('score');

  // Complex derived computation
  let sortedRecs = $derived.by(() => {
    const scored = recs.map(rec => ({
      ...rec,
      combinedScore: rec.score + rec.community_score * communityWeight
    }));

    switch (sortBy) {
      case 'score':
        return scored.sort((a, b) => b.combinedScore - a.combinedScore);
      case 'words':
        return scored.sort((a, b) => b.words - a.words);
      case 'title':
        return scored.sort((a, b) => a.title.localeCompare(b.title));
      default:
        return scored;
    }
  });
</script>
```

This is exactly how FicHub's Recommendations tab works. The `$derived.by` rune recomputes whenever `recs`, `communityWeight`, or `sortBy` changes. The result is automatically kept in sync.

**Key insight:** `$derived` values are **lazily evaluated** — they're only recomputed when something they depend on changes, and only when something reads them. This means you can have expensive derived computations that only run when needed.

### $effect: Side Effects Done Right

Side effects are operations that interact with the outside world: DOM manipulation, API calls, timers, localStorage. The `$effect` rune is how you handle them in Svelte 5.

```svelte
<script>
  let url = $state('');
  let debounceTimer;

  // Debounced URL validation
  $effect(() => {
    // Clear any existing timer when url changes
    clearTimeout(debounceTimer);
    
    if (url) {
      debounceTimer = setTimeout(() => {
        // Validate the URL after 500ms of no typing
        validateUrl(url);
      }, 500);
    }
    
    // Cleanup function — runs before the next effect
    return () => clearTimeout(debounceTimer);
  });

  function validateUrl(url) {
    try {
      new URL(url);
      console.log('Valid URL');
    } catch {
      console.log('Invalid URL');
    }
  }
</script>
```

The cleanup function is crucial. When `url` changes, Svelte runs the cleanup function from the *previous* effect before running the new one. This prevents stale timers, memory leaks, and other bugs.

Here's another pattern — syncing state to localStorage:

```svelte
<script>
  let theme = $state(
    typeof localStorage !== 'undefined' 
      ? localStorage.getItem('theme') ?? 'dark'
      : 'dark'
  );

  // Persist theme to localStorage whenever it changes
  $effect(() => {
    localStorage.setItem('theme', theme);
  });

  // Apply theme to document
  $effect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  });
</script>
```

Notice that we have two separate `$effect` blocks. Each one tracks different dependencies. The first tracks `theme` and writes to localStorage. The second also tracks `theme` and updates the DOM. Svelte runs each effect independently and only re-runs the ones whose dependencies changed.

### $effect vs $effect.pre

Sometimes you need to run code *before* the DOM updates, not after. For example, scrolling to a position before the user sees the change. The `$effect.pre` rune runs synchronously before the DOM is updated:

```svelte
<script>
  let messages = $state([]);
  let autoScroll = $state(true);
  let chatContainer;

  // Before DOM update, save scroll position
  let previousScrollHeight;
  $effect.pre(() => {
    if (chatContainer) {
      previousScrollHeight = chatContainer.scrollHeight;
    }
    // Track that we read messages and autoScroll
    messages.length;
    autoScroll;
  });

  // After DOM update, adjust scroll position
  $effect(() => {
    if (chatContainer && autoScroll && previousScrollHeight !== undefined) {
      const newScrollHeight = chatContainer.scrollHeight;
      const heightDiff = newScrollHeight - previousScrollHeight;
      chatContainer.scrollTop += heightDiff;
    }
    messages.length;
    autoScroll;
  });
</script>

<div bind:this={chatContainer} class="chat">
  {#each messages as msg}
    <div class="message">{msg.text}</div>
  {/each}
</div>
```

This is a common pattern for chat applications — keeping the user at the bottom of the message list as new messages arrive, while still allowing them to scroll up to read history.

### $effect.root: Effects Outside Components

Most effects live inside components, but sometimes you need an effect in a utility function or a store. The `$effect.root` rune creates a root-level effect that doesn't belong to any component:

```typescript
// lib/stores/theme.ts
export function createThemeStore() {
  let theme = $state('dark');

  // This effect runs outside any component
  $effect.root(() => {
    $effect(() => {
      localStorage.setItem('theme', theme);
      document.documentElement.setAttribute('data-theme', theme);
    });
  });

  return {
    get theme() { return theme; },
    set theme(t) { theme = t; },
    toggle() { theme = theme === 'dark' ? 'light' : 'dark'; }
  };
}
```

### $state and Unowned State

By default, `$state` creates owned state — it's tied to a component and gets cleaned up when the component is destroyed. But sometimes you need state that lives longer than any component:

```svelte
<script>
  // This state is "unowned" — it persists across component lifecycles
  let globalCount = $state.unsafe_unowned(0);
</script>
```

In practice, you rarely need `unsafe_unowned`. It's mainly used for stores and shared state. If you're building a store, consider using `$state` inside a module-level variable instead:

```typescript
// lib/stores/counter.ts
let count = $state(0);

export const counter = {
  get value() { return count; },
  increment() { count++; },
  decrement() { count--; },
  reset() { count = 0; }
};
```

### The $props Rune: Advanced Patterns

We covered `$props` basics in Chapter 8. Here are some advanced patterns.

**Extracting and forwarding props:**

```svelte
<script>
  // Receive some props, forward the rest to an inner element
  let { variant = 'primary', size = 'medium', class: className, ...rest } = $props();
</script>

<button class="btn btn-{variant} btn-{size} {className}" {...rest}>
  <slot />
</button>
```

The `...rest` spread captures any props not explicitly destructured. This lets callers pass any standard HTML attribute (like `aria-label`, `data-testid`, or `onkeydown`) and they'll be forwarded to the button element.

**Type-safe props with TypeScript:**

```svelte
<script lang="ts">
  interface Props {
    title: string;
    items: Array<{ id: number; label: string; active?: boolean }>;
    onSelect: (id: number) => void;
    maxVisible?: number;
  }

  let { title, items, onSelect, maxVisible = 10 }: Props = $props();
  
  let visibleItems = $derived(items.slice(0, maxVisible));
  let hasMore = $derived(items.length > maxVisible);
</script>

<h3>{title}</h3>
<ul>
  {#each visibleItems as item (item.id)}
    <li 
      class:active={item.active}
      onclick={() => onSelect(item.id)}
      role="button"
      tabindex="0"
    >
      {item.label}
    </li>
  {/each}
</ul>
{#if hasMore}
  <p class="muted">And {items.length - maxVisible} more...</p>
{/if}
```

This is a pattern used throughout FicHub's UI — components that display lists of items with a configurable maximum visible count.

**$bindable for two-way binding:**

```svelte
<!-- Slider.svelte -->
<script>
  let { value = $bindable(0), min = 0, max = 100, label = '' } = $props();
</script>

<div class="slider">
  {#if label}
    <label>{label}: {value}</label>
  {/if}
  <input 
    type="range" 
    {min} {max} 
    bind:value
  />
</div>
```

```svelte
<!-- Parent.svelte -->
<script>
  import Slider from './Slider.svelte';
  let volume = $state(50);
</script>

<Slider bind:value={volume} min={0} max={100} label="Volume" />
<p>Current volume: {volume}</p>
```

When the slider moves, `volume` in the parent updates automatically. This is two-way binding — the child can read and write the parent's state.

### Reactive Statements vs $derived

In Svelte 4, you could write reactive statements with `$:`:

```svelte
<!-- Svelte 4 (still works but not recommended) -->
<script>
  let count = 0;
  $: doubled = count * 2;
  $: console.log('Count is', count);
</script>
```

In Svelte 5, prefer `$derived` for computed values and `$effect` for side effects:

```svelte
<!-- Svelte 5 (recommended) -->
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);
  
  $effect(() => {
    console.log('Count is', count);
  });
</script>
```

The reason is clarity. `$:` was ambiguous — was it a computation or a side effect? `$derived` and `$effect` make your intent explicit. The compiler also generates more efficient code when it knows whether you're computing a value or running a side effect.

### Practical Example: FicHub Search Filters

Let's build a complete reactive search filter component that demonstrates multiple rune patterns:

```svelte
<script lang="ts">
  import type { SearchFilters } from '$lib/api/search';
  import { defaultFilters, SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS } from '$lib/api/search';

  let filters = $state(defaultFilters());
  let onSearch = $props<{ (filters: SearchFilters): void }>();

  // Derived: is the form dirty (has non-default values)?
  let isDirty = $derived(
    filters.q !== '' ||
    filters.include_tags !== '' ||
    filters.exclude_tags !== '' ||
    filters.min_words !== null ||
    filters.max_words !== null ||
    filters.complete !== null ||
    filters.source !== '' ||
    filters.sort !== ''
  );

  // Derived: count active filters
  let activeFilterCount = $derived(
    [
      filters.q,
      filters.include_tags,
      filters.exclude_tags,
      filters.min_words !== null,
      filters.max_words !== null,
      filters.complete !== null,
      filters.source,
      filters.sort
    ].filter(Boolean).length
  );

  // Effect: log filter changes (for debugging)
  $effect(() => {
    console.log('Filters changed:', $state.snapshot(filters));
  });

  function handleSubmit() {
    onSearch(filters);
  }

  function handleReset() {
    filters = defaultFilters();
    onSearch(filters);
  }

  function handleSortChange(e: Event) {
    filters.sort = (e.target as HTMLSelectElement).value;
  }

  function handleSourceChange(e: Event) {
    filters.source = (e.target as HTMLSelectElement).value;
  }

  function handleCompleteChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    filters.complete = val === 'true' ? true : val === 'false' ? false : null;
  }
</script>

<form onsubmit|preventDefault={handleSubmit} class="filters">
  <div class="filter-row">
    <input
      type="text"
      placeholder="Search fics..."
      bind:value={filters.q}
      aria-label="Search query"
    />
  </div>

  <div class="filter-row">
    <label>
      Sort by
      <select onchange={handleSortChange}>
        {#each SORT_OPTIONS as opt}
          <option value={opt.value} selected={filters.sort === opt.value}>
            {opt.label}
          </option>
        {/each}
      </select>
    </label>

    <label>
      Status
      <select onchange={handleCompleteChange}>
        {#each COMPLETE_OPTIONS as opt}
          <option 
            value={opt.value} 
            selected={filters.complete === (opt.value === 'true' ? true : opt.value === 'false' ? false : null)}
          >
            {opt.label}
          </option>
        {/each}
      </select>
    </label>

    <label>
      Site
      <select onchange={handleSourceChange}>
        {#each SOURCE_OPTIONS as opt}
          <option value={opt.value} selected={filters.source === opt.value}>
            {opt.label}
          </option>
        {/each}
      </select>
    </label>
  </div>

  {#if activeFilterCount > 0}
    <div class="filter-status">
      <span class="badge">{activeFilterCount} filter{activeFilterCount > 1 ? 's' : ''} active</span>
      <button type="button" class="btn-link" onclick={handleReset}>
        Clear all
      </button>
    </div>
  {/if}

  <button type="submit" class="btn" disabled={!isDirty && !filters.q}>
    Search
  </button>
</form>
```

This component demonstrates:
- `$state` for the filter object
- `$derived` for computing the dirty state and active filter count
- `$effect` for debugging
- `$props` for the callback
- Reactive bindings with `bind:value`
- Event handling with `$state` mutations

### Practice Exercises

1. **Reactive Timer:** Create a `Countdown.svelte` component that counts down from a given number. Use `$state` for the remaining time, `$derived` for the formatted time string (MM:SS), and `$effect` for the interval timer. Include start, pause, and reset buttons.

2. **Reactive Form:** Build a registration form with `$state` for each field. Use `$derived` to compute whether the form is valid (all required fields filled, email matches regex, passwords match). Disable the submit button when the form is invalid.

3. **Deep Reactivity Test:** Create a nested state object three levels deep. Modify a deeply nested property and verify that only the parts of the template that read that property re-render. Use `console.log` statements in the template to count re-renders.

4. **Effect Cleanup:** Build a component that subscribes to a WebSocket. Use `$effect` to connect and `$effect`'s cleanup function to disconnect. Verify that changing the URL reconnects without leaking the old connection.

---

## Chapter 38: Svelte 5 Event Handling and Transitions

### Event Handling: Beyond onclick

We've already seen basic event handling — `onclick={handleClick}`. But Svelte 5 gives you a rich set of event modifiers and patterns. Let's explore them all.

### Event Modifiers

Svelte 5 uses the `on:` directive for events with modifiers:

```svelte
<!-- Prevent default form submission -->
<form on:submit|preventDefault={handleSubmit}>
  <input type="text" bind:value={name} />
  <button type="submit">Submit</button>
</form>

<!-- Stop event propagation -->
<div on:click|stopPropagation={handleClick}>
  <button>Click me</button>
</div>

<!-- Once — handler runs only once -->
<button on:click|once={handleClick}>
  Click me (only works once)
</button>

<!-- Passive — tells the browser this handler won't prevent scrolling -->
<div on:touchmove|passive={handleTouch}>
  Scrollable area
</div>

<!-- Self — only triggers if event.target is the element itself -->
<div on:click|self={handleClick}>
  <button>Child button</button>
</div>

<!-- Capture — fires during the capture phase -->
<div on:click|capture={handleClick}>
  <button>Child button</button>
</div>
```

You can chain modifiers: `on:submit|preventDefault|once={handleSubmit}`.

### Custom Events with createEventDispatcher

When a child component needs to notify its parent of something, you use custom events:

```svelte
<!-- SearchBar.svelte -->
<script>
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  let query = $state('');

  function handleSubmit() {
    dispatch('search', { query: query.trim() });
  }
</script>

<form on:submit|preventDefault={handleSubmit}>
  <input type="text" bind:value={query} placeholder="Search..." />
  <button type="submit">Search</button>
</form>
```

```svelte
<!-- Parent.svelte -->
<script>
  import SearchBar from './SearchBar.svelte';
  
  function handleSearch(event) {
    console.log('Searching for:', event.detail.query);
  }
</script>

<SearchBar on:search={handleSearch} />
```

However, in Svelte 5, the preferred pattern is to pass callbacks as props:

```svelte
<!-- SearchBar.svelte (Svelte 5 style) -->
<script>
  let query = $state('');
  let onSearch = $props();
  
  function handleSubmit() {
    onSearch(query.trim());
  }
</script>

<form on:submit|preventDefault={handleSubmit}>
  <input type="text" bind:value={query} placeholder="Search..." />
  <button type="submit">Search</button>
</form>
```

```svelte
<!-- Parent.svelte -->
<script>
  import SearchBar from './SearchBar.svelte';
</script>

<SearchBar onSearch={(q) => console.log('Searching for:', q)} />
```

This is simpler, more TypeScript-friendly, and doesn't require importing `createEventDispatcher`.

### Keyboard Events: Building Accessible Interactions

Keyboard handling is essential for accessibility. Let's build a complete keyboard-navigable component:

```svelte
<script>
  let items = $state([
    { id: 1, label: 'Download' },
    { id: 2, label: 'Recommendations' },
    { id: 3, label: 'Suggestions' }
  ]);
  
  let selectedIndex = $state(0);
  let onSelect = $props();

  function handleKeydown(e) {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        onSelect(items[selectedIndex].id);
        break;
      case 'Home':
        e.preventDefault();
        selectedIndex = 0;
        break;
      case 'End':
        e.preventDefault();
        selectedIndex = items.length - 1;
        break;
    }
  }
</script>

<div 
  class="nav-list" 
  role="listbox" 
  aria-label="Navigation"
  on:keydown={handleKeydown}
>
  {#each items as item, i (item.id)}
    <div
      class="nav-item"
      class:selected={i === selectedIndex}
      role="option"
      aria-selected={i === selectedIndex}
      tabindex={i === selectedIndex ? 0 : -1}
      onclick={() => { selectedIndex = i; onSelect(item.id); }}
    >
      {item.label}
    </div>
  {/each}
</div>
```

This pattern is used in FicHub's tab navigation. It supports arrow keys, Home/End, Enter/Space, and proper ARIA attributes for screen readers.

### Mouse Events: Drag and Drop

FicHub's bookmarklet uses a drag event. Let's build a drag-and-drop component:

```svelte
<script>
  let draggable = $state(false);
  let isDragging = $state(false);

  function handleDragStart(e) {
    isDragging = true;
    e.dataTransfer.setData('text/plain', 'fichub-bookmarklet');
    e.dataTransfer.effectAllowed = 'copy';
  }

  function handleDragEnd() {
    isDragging = false;
  }
</script>

<div 
  class="draggable-item"
  class:dragging={isDragging}
  {draggable}
  on:dragstart={handleDragStart}
  on:dragend={handleDragEnd}
>
  <slot />
</div>

<style>
  .draggable-item {
    cursor: grab;
    transition: opacity 0.2s ease;
  }
  .draggable-item:active {
    cursor: grabbing;
  }
  .draggable-item.dragging {
    opacity: 0.5;
  }
</style>
```

### Transitions: Making UI Changes Beautiful

When elements enter or leave the DOM, transitions make the change smooth instead of jarring. Svelte has built-in transitions that you can use with the `transition:` directive.

**Fade — Gentle opacity transitions:**

```svelte
<script>
  import { fade } from 'svelte/transition';
  
  let showDetails = $state(false);
</script>

<button onclick={() => showDetails = !showDetails}>
  {showDetails ? 'Hide' : 'Show'} Details
</button>

{#if showDetails}
  <div transition:fade={{ duration: 300 }}>
    <p>Here are the details!</p>
  </div>
{/if}
```

**Slide — Elements that slide in and out:**

```svelte
<script>
  import { slide } from 'svelte/transition';
  
  let expanded = $state(false);
</script>

<button onclick={() => expanded = !expanded}>
  {expanded ? 'Collapse' : 'Expand'} Section
</button>

{#if expanded}
  <div transition:slide={{ duration: 200 }}>
    <p>This content slides in smoothly.</p>
  </div>
{/if}
```

**Fly — Elements that fly in from a direction:**

```svelte
<script>
  import { fly } from 'svelte/transition';
  
  let showNotification = $state(false);
  
  function triggerNotification() {
    showNotification = true;
    setTimeout(() => showNotification = false, 3000);
  }
</script>

<button onclick={triggerNotification}>Show Notification</button>

{#if showNotification}
  <div 
    class="notification"
    transition:fly={{ y: -20, duration: 300 }}
  >
    ✅ Story downloaded successfully!
  </div>
{/if}

<style>
  .notification {
    position: fixed;
    top: 20px;
    right: 20px;
    background: var(--color-surface);
    border: 1px solid var(--color-success);
    border-radius: var(--radius);
    padding: 1rem;
    z-index: 100;
  }
</style>
```

**Scale — Elements that scale in from a point:**

```svelte
<script>
  import { scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  
  let showModal = $state(false);
</script>

<button onclick={() => showModal = true}>Open Modal</button>

{#if showModal}
  <div class="overlay" onclick={() => showModal = false}>
    <div 
      class="modal"
      transition:scale={{ duration: 200, easing: quintOut, opacity: 0.5 }}
      onclick={(e) => e.stopPropagation()}
    >
      <h2>Modal Title</h2>
      <p>Modal content goes here.</p>
      <button onclick={() => showModal = false}>Close</button>
    </div>
  </div>
{/if}
```

**Custom Transitions:**

You can create your own transitions by implementing the `transition` interface:

```svelte
<script>
  import { cubicOut } from 'svelte/easing';
  
  function typewriter(node, { speed = 1 }) {
    const text = node.textContent;
    const duration = text.length / (speed * 0.01);

    return {
      duration,
      tick: (t) => {
        const i = Math.trunc(text.length * t);
        node.textContent = text.slice(0, i);
      }
    };
  }
</script>

<p transition:typewriter={{ speed: 0.5 }}>
  This text appears one character at a time!
</p>
```

### Transitions with {#each} Blocks

When transitioning items in a list, use `animate:` for reordering:

```svelte
<script>
  import { flip } from 'svelte/animate';
  import { fade } from 'svelte/transition';
  
  let items = $state([
    { id: 1, text: 'First' },
    { id: 2, text: 'Second' },
    { id: 3, text: 'Third' }
  ]);

  function shuffle() {
    items = items.sort(() => Math.random() - 0.5);
  }
  
  function remove(id) {
    items = items.filter(item => item.id !== id);
  }
</script>

<button onclick={shuffle}>Shuffle</button>

{#each items as item (item.id)}
  <div
    transition:fade={{ duration: 200 }}
    animate:flip={{ duration: 300 }}
    class="item"
  >
    {item.text}
    <button onclick={() => remove(item.id)}>×</button>
  </div>
{/each}
```

The `animate:flip` directive makes items smoothly animate to their new positions when the list is reordered. Combined with `transition:fade`, items fade in/out and the remaining items slide to fill the gap.

### In/Out Transitions

Sometimes you want different transitions for entering and leaving:

```svelte
<script>
  import { fly, fade } from 'svelte/transition';
  
  let show = $state(false);
</script>

<button onclick={() => show = !show}>Toggle</button>

{#if show}
  <div 
    in:fly={{ y: 20, duration: 300 }}
    out:fade={{ duration: 200 }}
  >
    I fly in but fade out!
  </div>
{/if}
```

### CSS Animations vs Svelte Transitions

CSS animations and Svelte transitions serve different purposes:

- **CSS animations** are great for continuous or looped animations (loading spinners, pulsing effects, ambient background animations)
- **Svelte transitions** are great for one-shot enter/exit animations (modals appearing, notifications sliding in, content fading)

Use them together for the best results:

```svelte
<script>
  import { fly } from 'svelte/transition';
  
  let loading = $state(false);
</script>

{#if loading}
  <div transition:fly={{ y: -10 }}>
    <span class="spinner"></span>
    Loading...
  </div>
{/if}

<style>
  /* CSS animation for the spinner */
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .spinner {
    animation: spin 1s linear infinite;
  }
</style>
```

### Practice Exercises

1. **Animated Toast Notifications:** Build a toast notification system. Notifications fly in from the right, stay for 3 seconds, then fade out. Support multiple simultaneous notifications stacked vertically.

2. **Accordion:** Create an accordion component using `transition:slide` for expanding/collapsing sections. Support only-one-open-at-a-time behavior.

3. **Sortable List:** Build a sortable list with `animate:flip` for reordering and `transition:fade` for adding/removing items. Include add, remove, and shuffle buttons.

4. **Modal with Focus Trap:** Build an accessible modal that traps focus inside it when open. Use `transition:scale` for the modal and `transition:fade` for the backdrop. Handle Escape key to close.

---

## Chapter 39: Svelte 5 Lifecycle and Component Communication

### Component Lifecycle: Birth, Life, and Death

Every Svelte component goes through three phases: creation, interaction, and destruction. Understanding these phases helps you manage resources, set up event listeners, and clean up after yourself.

### onMount: When the Component Appears

The `onMount` function runs after the component is first rendered to the DOM. This is where you set up things that need the DOM to exist — measuring elements, adding event listeners, starting API calls, initializing third-party libraries.

```svelte
<script>
  import { onMount } from 'svelte';
  
  let chartContainer;
  let chart;

  onMount(async () => {
    // Dynamic import — only load the chart library when needed
    const { Chart } = await import('chart.js');
    
    chart = new Chart(chartContainer, {
      type: 'bar',
      data: {
        labels: ['AO3', 'FFN', 'FP', 'SB', 'SV'],
        datasets: [{
          label: 'Stories',
          data: [1200, 800, 200, 150, 120]
        }]
      }
    });

    // Return cleanup function
    return () => {
      chart?.destroy();
    };
  });
</script>

<canvas bind:this={chartContainer}></canvas>
```

The return value of `onMount` is a cleanup function. Svelte calls it when the component is destroyed. This is crucial for preventing memory leaks — always clean up event listeners, timers, WebSocket connections, and third-party library instances.

### onDestroy: Cleanup on Destruction

If you don't want to use the return value of `onMount`, you can use `onDestroy`:

```svelte
<script>
  import { onMount, onDestroy } from 'svelte';
  
  let interval;
  let count = $state(0);

  onMount(() => {
    interval = setInterval(() => {
      count++;
    }, 1000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<p>Count: {count}</p>
```

In Svelte 5, the preferred pattern is to use the return value of `onMount` for cleanup, as it keeps the setup and teardown logic together. But `onDestroy` still works and is useful when cleanup logic is defined elsewhere.

### AfterUpdate and BeforeUpdate

These lifecycle functions let you run code before and after the DOM updates:

```svelte
<script>
  import { beforeUpdate, afterUpdate } from 'svelte';
  
  let div;
  let autoscroll;

  beforeUpdate(() => {
    if (div) {
      const { scrollTop, scrollHeight, offsetHeight } = div;
      // Check if user was scrolled to near bottom
      autoscroll = scrollHeight - scrollTop - offsetHeight < 50;
    }
  });

  afterUpdate(() => {
    if (div && autoscroll) {
      div.scrollTop = div.scrollHeight;
    }
  });
</script>

<div bind:this={div} class="chat-messages">
  <!-- messages here -->
</div>
```

In Svelte 5, you can achieve the same thing with `$effect.pre` (before update) and `$effect` (after update), which we covered in Chapter 37.

### Context API: Sharing Data Without Props Drilling

When you have data that many components need — like the current user, theme settings, or feature flags — passing props through every level of the component tree is tedious. The Context API lets you provide data to all descendant components.

```svelte
<!-- App.svelte -->
<script>
  import { setContext } from 'svelte';
  
  let theme = $state('dark');
  
  setContext('theme', {
    get theme() { return theme; },
    set theme(t) { theme = t; },
    toggle() { theme = theme === 'dark' ? 'light' : 'dark'; }
  });
</script>

<div class="app" data-theme={theme}>
  <slot />
</div>
```

```svelte
<!-- ThemeToggle.svelte -->
<script>
  import { getContext } from 'svelte';
  
  const theme = getContext('theme');
</script>

<button onclick={() => theme.toggle()}>
  {theme.theme === 'dark' ? '☀️ Light' : '🌙 Dark'}
</button>
```

Any component nested inside `App.svelte` can call `getContext('theme')` to access the theme. No props drilling needed!

### Store Pattern: Reactive State Outside Components

Svelte 5 still supports the store pattern for shared state. Here's how to create a reactive store:

```typescript
// lib/stores/favorites.ts
let favorites = $state<string[]>([]);

export const favoritesStore = {
  get all() { return favorites; },
  
  add(urlId: string) {
    if (!favorites.includes(urlId)) {
      favorites = [...favorites, urlId];
      saveToStorage();
    }
  },
  
  remove(urlId: string) {
    favorites = favorites.filter(id => id !== urlId);
    saveToStorage();
  },
  
  has(urlId: string) {
    return favorites.includes(urlId);
  },
  
  toggle(urlId: string) {
    if (favorites.includes(urlId)) {
      this.remove(urlId);
    } else {
      this.add(urlId);
    }
  }
};

function saveToStorage() {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('favorites', JSON.stringify(favorites));
  }
}

// Initialize from localStorage
if (typeof localStorage !== 'undefined') {
  const saved = localStorage.getItem('favorites');
  if (saved) {
    try {
      favorites = JSON.parse(saved);
    } catch {
      favorites = [];
    }
  }
}
```

```svelte
<!-- FavoriteButton.svelte -->
<script>
  import { favoritesStore } from '$lib/stores/favorites';
  
  let { urlId } = $props();
  
  let isFavorited = $derived(favoritesStore.has(urlId));
  
  function toggle() {
    favoritesStore.toggle(urlId);
  }
</script>

<button 
  class="fav-btn" 
  class:active={isFavorited}
  onclick={toggle}
  aria-label={isFavorited ? 'Remove from favorites' : 'Add to favorites'}
>
  {isFavorited ? '❤️' : '🤍'}
</button>
```

### Props Injection Pattern

Another way to share data is through props injection. A parent provides a function, and descendants call it:

```svelte
<!-- Parent.svelte -->
<script>
  let logs = $state([]);
  
  function addLog(message) {
    logs = [...logs, { message, time: new Date() }];
  }
</script>

<div class="app">
  <slot {addLog} />
  
  <div class="log-panel">
    {#each logs as log}
      <p>{log.message}</p>
    {/each}
  </div>
</div>
```

```svelte
<!-- Child.svelte -->
<script>
  let { addLog } = $props();
  
  function handleClick() {
    addLog('Button was clicked!');
  }
</script>

<button onclick={handleClick}>Click me</button>
```

### Component Bindings: Two-Way Communication

We've seen `$bindable` for simple two-way binding. Here's a more complex example — a tabbed interface:

```svelte
<!-- Tabs.svelte -->
<script>
  let { tabs, activeTab = $bindable(0) } = $props();
</script>

<div class="tabs">
  <div class="tab-bar" role="tablist">
    {#each tabs as tab, i (tab.id)}
      <button
        role="tab"
        aria-selected={i === activeTab}
        class:active={i === activeTab}
        onclick={() => activeTab = i}
      >
        {tab.label}
      </button>
    {/each}
  </div>
  
  <div class="tab-content">
    <slot />
  </div>
</div>
```

```svelte
<!-- Parent.svelte -->
<script>
  import Tabs from './Tabs.svelte';
  import DownloadTab from './DownloadTab.svelte';
  import RecommendationsTab from './RecommendationsTab.svelte';
  import SuggestionsTab from './SuggestionsTab.svelte';
  
  let activeTab = $state(0);
  
  const tabs = [
    { id: 'download', label: 'Download' },
    { id: 'recommendations', label: 'Recommendations' },
    { id: 'suggestions', label: 'Suggestions' }
  ];
</script>

<Tabs {tabs} bind:activeTab>
  {#if activeTab === 0}
    <DownloadTab />
  {:else if activeTab === 1}
    <RecommendationsTab />
  {:else}
    <SuggestionsTab />
  {/if}
</Tabs>
```

### The Slot Pattern: Composition with Content Projection

Slots let parent components inject content into child components:

```svelte
<!-- Card.svelte -->
<script>
  let { title, variant = 'default' } = $props();
</script>

<div class="card card-{variant}">
  {#if title}
    <div class="card-header">
      <h3>{title}</h3>
    </div>
  {/if}
  
  <div class="card-body">
    <slot />
  </div>
  
  {#if $$slots.footer}
    <div class="card-footer">
      <slot name="footer" />
    </div>
  {/if}
</div>
```

```svelte
<!-- Usage -->
<Card title="Story Details" variant="highlighted">
  <p>Harry Potter discovers he's a wizard...</p>
  
  <svelte:fragment slot="footer">
    <button>Download EPUB</button>
    <button>Add to Favorites</button>
  </svelte:fragment>
</Card>
```

Named slots (`slot="footer"`) let you inject content into specific areas. The `{#if $$slots.footer}` check lets you conditionally render the footer only when content is provided.

### Practice Exercises

1. **Modal with Lifecycle:** Build a modal component that traps focus when open, restores focus when closed, and handles the Escape key. Use `onMount` and `onDestroy` for event listeners.

2. **Infinite Scroll:** Create a component that loads more items as the user scrolls to the bottom. Use `onMount` for the IntersectionObserver and `onDestroy` for cleanup.

3. **Context-Based Theme System:** Build a complete theme system using Context API. Include light/dark themes, CSS variables, and a toggle button. The theme should persist to localStorage.

4. **Component Library:** Create a set of reusable components (Button, Card, Input, Modal) that all share a consistent theme via Context API.

---

## Chapter 40: Svelte 5 Stores and Global State

### Understanding the State Management Landscape

As your application grows, you'll find that some state needs to be shared across many components. The current user, theme preferences, notification counts, cached API responses — these don't belong to any single component.

Svelte 5 offers several ways to manage global state:
1. **Module-level `$state`** — Simple, no boilerplate
2. **Context API** — Scoped to a component subtree
3. **Stores** — Traditional Svelte stores (still supported)
4. **External stores** — Libraries like Zustand or Jotai

Let's explore each approach and understand when to use which.

### Module-Level State: The Simplest Approach

The simplest way to share state is to put it in a module file:

```typescript
// lib/state/counter.ts
let count = $state(0);

export const counter = {
  get value() { return count; },
  increment() { count++; },
  decrement() { count--; },
  reset() { count = 0; }
};
```

```svelte
<!-- AnyComponent.svelte -->
<script>
  import { counter } from '$lib/state/counter';
</script>

<p>Count: {counter.value}</p>
<button onclick={() => counter.increment()}>+</button>
```

This works because JavaScript modules are singletons — no matter how many files import `counter`, they all get the same instance. The `$state` rune makes the variable reactive, and the getter/setter pattern exposes it safely.

**Pros:** Simple, no imports needed beyond the module, full TypeScript support.
**Cons:** Global scope (everything sees everything), no scoping to component subtrees.

### Stores: The Traditional Svelte Way

Svelte has built-in store support. Stores are objects with a `subscribe` method. Svelte auto-subscribes to stores when you use the `$` prefix:

```typescript
// lib/stores/searchHistory.ts
import { writable } from 'svelte/store';

export const searchHistory = writable<string[]>([]);

export function addToHistory(query: string) {
  searchHistory.update(history => {
    const newHistory = [query, ...history.filter(h => h !== query)];
    return newHistory.slice(0, 20); // Keep last 20
  });
}

export function clearHistory() {
  searchHistory.set([]);
}
```

```svelte
<!-- SearchHistory.svelte -->
<script>
  import { searchHistory, clearHistory } from '$lib/stores/searchHistory';
</script>

{#if $searchHistory.length > 0}
  <div class="history">
    <h3>Recent Searches</h3>
    <ul>
      {#each $searchHistory as query}
        <li>{query}</li>
      {/each}
    </ul>
    <button onclick={clearHistory}>Clear History</button>
  </div>
{/if}
```

The `$` prefix auto-subscribes to the store. When the store's value changes, the component re-renders. When the component is destroyed, the subscription is automatically cleaned up.

### Derived Stores: Computing from Other Stores

```typescript
// lib/stores/filteredRecs.ts
import { writable, derived } from 'svelte/store';
import type { RecResult } from '$lib/api/types';

export const allRecs = writable<RecResult[]>([]);
export const sortBy = writable<'score' | 'words' | 'title'>('score');

export const filteredRecs = derived(
  [allRecs, sortBy],
  ([$allRecs, $sortBy]) => {
    return [...$allRecs].sort((a, b) => {
      switch ($sortBy) {
        case 'score': return b.score - a.score;
        case 'words': return b.words - a.words;
        case 'title': return a.title.localeCompare(b.title);
      }
    });
  }
);

export const stats = derived(allRecs, ($allRecs) => ({
  total: $allRecs.length,
  totalWords: $allRecs.reduce((sum, r) => sum + r.words, 0),
  avgWords: $allRecs.length > 0 
    ? Math.round($allRecs.reduce((sum, r) => sum + r.words, 0) / $allRecs.length)
    : 0
}));
```

### Custom Stores: Encapsulating Logic

You can create stores that encapsulate complex logic:

```typescript
// lib/stores/asyncStore.ts
import { writable } from 'svelte/store';

export function asyncStore<T>(fetcher: () => Promise<T>) {
  const state = writable<{
    data: T | null;
    loading: boolean;
    error: string | null;
  }>({
    data: null,
    loading: false,
    error: null
  });

  async function load() {
    state.update(s => ({ ...s, loading: true, error: null }));
    try {
      const data = await fetcher();
      state.set({ data, loading: false, error: null });
    } catch (e) {
      state.update(s => ({
        ...s,
        loading: false,
        error: e instanceof Error ? e.message : 'Unknown error'
      }));
    }
  }

  return {
    subscribe: state.subscribe,
    load,
    get data() { return state; }
  };
}
```

```svelte
<script>
  import { asyncStore } from '$lib/stores/asyncStore';
  import { fetchRecommendations } from '$lib/api/client';

  const recommendations = asyncStore(() => 
    fetchRecommendations('https://ao3.org/works/12345')
      .then(res => res.recommendations)
  );

  onMount(() => recommendations.load());
</script>

{#if $recommendations.loading}
  <p>Loading...</p>
{:else if $recommendations.error}
  <p class="error">{$recommendations.error}</p>
{:else if $recommendations.data}
  {#each $recommendations.data as rec}
    <div class="rec">{rec.title}</div>
  {/each}
{/if}
```

### Writable, Readable, and Derived Stores

Svelte provides three store types:

**Writable** — Can be read and written:
```typescript
import { writable } from 'svelte/store';
const count = writable(0);
count.set(5);
count.update(n => n + 1);
```

**Readable** — Can only be read, not written:
```typescript
import { readable } from 'svelte/store';
const time = readable(new Date(), function start(set) {
  const interval = setInterval(() => set(new Date()), 1000);
  return function stop() { clearInterval(interval); };
});
```

**Derived** — Computed from other stores:
```typescript
import { derived } from 'svelte/store';
const elapsed = derived(time, ($time) => {
  // computed value
});
```

### Combining Stores with Module-Level State

In Svelte 5, you can combine module-level `$state` with the store pattern:

```typescript
// lib/state/user.ts
interface User {
  id: string;
  name: string;
  email: string;
  preferences: {
    theme: 'dark' | 'light';
    fontSize: number;
  };
}

let currentUser = $state<User | null>(null);
let isLoading = $state(false);
let error = $state<string | null>(null);

export const userStore = {
  get current() { return currentUser; },
  get loading() { return isLoading; },
  get error() { return error; },
  get isLoggedIn() { return currentUser !== null; },
  
  async login(email: string, password: string) {
    isLoading = true;
    error = null;
    try {
      const res = await fetch('/api/v0/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      if (!res.ok) throw new Error('Login failed');
      currentUser = await res.json();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Login failed';
    } finally {
      isLoading = false;
    }
  },
  
  logout() {
    currentUser = null;
  },
  
  updatePreferences(prefs: Partial<User['preferences']>) {
    if (currentUser) {
      currentUser.preferences = { ...currentUser.preferences, ...prefs };
    }
  }
};
```

### When to Use What

Here's a decision guide:

| Scenario | Approach |
|----------|----------|
| Simple shared value (counter, flag) | Module-level `$state` |
| Complex state with mutations | Module-level `$state` with store object |
| Scoped to component subtree | Context API |
| Need store operators (derived, writable) | Svelte stores |
| Async data fetching | Custom async store |
| Third-party library integration | Svelte stores (most libraries use this) |

### Practice Exercises

1. **Shopping Cart Store:** Create a cart store with add, remove, update quantity, and clear functions. Include derived values for total items and total price. Persist to localStorage.

2. **Auth Store:** Build an authentication store with login, logout, and token refresh. Use module-level `$state` for the user object and derived values for `isLoggedIn` and `userRole`.

3. **Notification Store:** Create a notification queue store that manages toast notifications. Include auto-dismiss after a timeout and manual dismiss. Support different notification types (success, error, warning, info).

4. **Search History Store:** Build a search history store that tracks recent searches, deduplicates them, and limits the history to 20 items. Include a derived store that returns the 5 most recent unique searches.

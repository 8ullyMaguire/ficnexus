# Part 11: State Management Patterns

*Managing complex state in real applications — from simple stores to advanced patterns.*

---

## Chapter 44: Local Component State Patterns

### When Simple State Isn't Enough

We've covered `$state` basics in earlier chapters. But real applications have state that's more complex than a single variable. Forms have validation state, loading states, and error states. Lists have selection state, pagination state, and filter state. Modals have open/close state, animation state, and form state.

In this chapter, we'll explore patterns for managing complex local component state.

### The State Machine Pattern

Instead of juggling multiple boolean flags, use a single state variable with a defined set of states:

```svelte
<script lang="ts">
  // Instead of this:
  // let loading = false;
  // let error = '';
  // let result = null;
  // let submitted = false;

  // Use a state machine:
  type DownloadState = 
    | { status: 'idle' }
    | { status: 'loading' }
    | { status: 'success'; result: ExportResponse }
    | { status: 'error'; message: string };

  let state = $state<DownloadState>({ status: 'idle' });
  let url = $state('');

  async function handleDownload() {
    if (!url.trim()) {
      state = { status: 'error', message: 'Please enter a URL.' };
      return;
    }

    state = { status: 'loading' };

    try {
      const res = await fetchExport(url.trim());
      if (res.err !== 0) {
        state = { status: 'error', message: res.msg || 'Something went wrong.' };
        return;
      }
      state = { status: 'success', result: res };
    } catch (e) {
      state = { 
        status: 'error', 
        message: e instanceof Error ? e.message : 'Network error.' 
      };
    }
  }

  function reset() {
    state = { status: 'idle' };
    url = '';
  }
</script>

<div class="download-tab">
  <form onsubmit|preventDefault={handleDownload}>
    <input 
      type="url" 
      bind:value={url}
      placeholder="Paste a fanfiction URL..."
      disabled={state.status === 'loading'}
    />
    <button type="submit" disabled={state.status === 'loading'}>
      {state.status === 'loading' ? 'Downloading...' : 'Download'}
    </button>
  </form>

  {#if state.status === 'error'}
    <div class="error-card">
      <p>⚠️ {state.message}</p>
      <button onclick={reset}>Try Again</button>
    </div>
  {/if}

  {#if state.status === 'success'}
    <div class="result-card">
      <h2>{state.result.meta?.title}</h2>
      <p>by {state.result.meta?.author}</p>
      {#if state.result.epub_url}
        <a href={state.result.epub_url} download>Download EPUB</a>
      {/if}
      <button onclick={reset}>Download Another</button>
    </div>
  {/if}
</div>
```

The state machine pattern has several advantages:
- **No impossible states:** You can't have `loading = true` AND `error = 'something'` at the same time
- **Clear transitions:** Every state change is explicit
- **Easy to test:** You can test each state independently
- **Pattern matching:** The template uses `{#if state.status === '...'}` which is exhaustive

### The Form State Pattern

Forms are one of the most common sources of complex state. Here's a pattern for managing form state:

```typescript
// lib/forms/formState.ts
interface FieldState<T> {
  value: T;
  error: string;
  touched: boolean;
  dirty: boolean;
}

interface FormState<T extends Record<string, any>> {
  fields: { [K in keyof T]: FieldState<T[K]> };
  isValid: boolean;
  isDirty: boolean;
  isSubmitting: boolean;
  submitError: string;
}

export function createFormState<T extends Record<string, any>>(
  initialValues: T,
  validators?: { [K in keyof T]?: (value: T[K]) => string }
) {
  const fields = {} as any;
  
  for (const [key, value] of Object.entries(initialValues)) {
    fields[key] = {
      value,
      error: '',
      touched: false,
      dirty: false
    };
  }

  let form = $state<FormState<T>>({
    fields,
    isValid: true,
    isDirty: false,
    isSubmitting: false,
    submitError: ''
  });

  function validateField(key: keyof T) {
    const validator = validators?.[key];
    if (!validator) return '';
    return validator(form.fields[key].value);
  }

  function setValue(key: keyof T, value: any) {
    form.fields[key].value = value;
    form.fields[key].dirty = value !== initialValues[key];
    form.fields[key].error = validateField(key);
    updateFormState();
  }

  function setTouched(key: keyof T) {
    form.fields[key].touched = true;
    form.fields[key].error = validateField(key);
    updateFormState();
  }

  function updateFormState() {
    const errors = Object.values(form.fields).map(f => f.error).filter(Boolean);
    form.isValid = errors.length === 0;
    form.isDirty = Object.values(form.fields).some(f => f.dirty);
  }

  function reset() {
    for (const [key, value] of Object.entries(initialValues)) {
      form.fields[key] = { value, error: '', touched: false, dirty: false };
    }
    form.isSubmitting = false;
    form.submitError = '';
    updateFormState();
  }

  return {
    get state() { return form; },
    setValue,
    setTouched,
    reset,
    getValues(): T {
      const values = {} as any;
      for (const key of Object.keys(form.fields)) {
        values[key] = form.fields[key].value;
      }
      return values;
    }
  };
}
```

```svelte
<!-- SuggestForm.svelte -->
<script lang="ts">
  import { createFormState } from '$lib/forms/formState';
  import { submitSuggestion } from '$lib/api/client';

  let { urlId, onSuccess } = $props();

  const form = createFormState(
    { ficUrl: '', comment: '' },
    {
      ficUrl: (value) => {
        if (!value) return 'URL is required';
        try { new URL(value); return ''; }
        catch { return 'Please enter a valid URL'; }
      },
      comment: (value) => {
        if (value.length > 500) return 'Comment must be under 500 characters';
        return '';
      }
    }
  );

  async function handleSubmit() {
    // Validate all fields
    form.setTouched('ficUrl');
    form.setTouched('comment');
    
    if (!form.state.isValid) return;

    form.state.isSubmitting = true;
    form.state.submitError = '';

    try {
      const values = form.getValues();
      const res = await submitSuggestion(
        urlId, 
        values.ficUrl, 
        values.comment || undefined
      );
      
      if (res.err !== 0) {
        form.state.submitError = res.msg || 'Failed to submit.';
        return;
      }
      
      form.reset();
      onSuccess?.();
    } catch (e) {
      form.state.submitError = 'Network error. Please try again.';
    } finally {
      form.state.isSubmitting = false;
    }
  }
</script>

<form onsubmit|preventDefault={handleSubmit}>
  <div class="field" class:error={form.state.fields.ficUrl.error && form.state.fields.ficUrl.touched}>
    <label for="fic-url">Fic URL</label>
    <input
      id="fic-url"
      type="url"
      value={form.state.fields.ficUrl.value}
      oninput={(e) => form.setValue('ficUrl', e.target.value)}
      onblur={() => form.setTouched('ficUrl')}
      placeholder="https://archiveofourown.org/works/..."
    />
    {#if form.state.fields.ficUrl.error && form.state.fields.ficUrl.touched}
      <p class="error-text">{form.state.fields.ficUrl.error}</p>
    {/if}
  </div>

  <div class="field">
    <label for="comment">Comment (optional)</label>
    <textarea
      id="comment"
      value={form.state.fields.comment.value}
      oninput={(e) => form.setValue('comment', e.target.value)}
      onblur={() => form.setTouched('comment')}
      placeholder="Why do you recommend this?"
      rows="3"
    ></textarea>
    {#if form.state.fields.comment.error}
      <p class="error-text">{form.state.fields.comment.error}</p>
    {/if}
  </div>

  {#if form.state.submitError}
    <p class="error-text">{form.state.submitError}</p>
  {/if}

  <button 
    type="submit" 
    disabled={!form.state.isValid || form.state.isSubmitting}
  >
    {form.state.isSubmitting ? 'Submitting...' : 'Submit Suggestion'}
  </button>
</form>
```

### The Optimistic Update Pattern

Optimistic updates improve perceived performance by updating the UI immediately, before the server confirms the change. FicHub's Suggestions tab uses this pattern for voting:

```svelte
<script lang="ts">
  let suggestions = $state<Suggestion[]>([]);
  let localVotes = $state<Record<number, number>>({});

  async function handleVote(id: number, vote: 1 | -1) {
    // 1. Save the previous state for rollback
    const prev = localVotes[id] ?? 0;
    
    // 2. Apply optimistic update immediately
    localVotes[id] = prev + vote;
    
    try {
      // 3. Send the actual request
      const res = await castVote(id, vote);
      
      if (res.err !== 0) {
        // 4a. Rollback on failure
        localVotes[id] = prev;
        return;
      }
      
      // 4b. Optionally refresh to get the true server state
      await loadSuggestions();
    } catch {
      // 4c. Rollback on network error
      localVotes[id] = prev;
    }
  }

  function netScore(s: Suggestion): number {
    return s.net_votes + (localVotes[s.id] ?? 0);
  }
</script>
```

The key insight is that `localVotes` is separate from the server data. The UI shows `netScore()` which combines both. When the server responds, we refresh and `localVotes` gets cleared.

### The Derived State Pattern

Sometimes the most complex state management is simply computing derived values correctly:

```svelte
<script lang="ts">
  let items = $state([
    { id: 1, name: 'Story A', words: 50000, status: 'complete', site: 'ao3' },
    { id: 2, name: 'Story B', words: 120000, status: 'in-progress', site: 'ffn' },
    { id: 3, name: 'Story C', words: 80000, status: 'complete', site: 'ao3' },
  ]);

  let filter = $state({ status: 'all', site: 'all', minWords: 0 });
  let sortBy = $state('words');
  let searchQuery = $state('');

  // Derived: filtered items
  let filteredItems = $derived.by(() => {
    let result = [...items];

    // Apply search
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      result = result.filter(i => i.name.toLowerCase().includes(q));
    }

    // Apply status filter
    if (filter.status !== 'all') {
      result = result.filter(i => i.status === filter.status);
    }

    // Apply site filter
    if (filter.site !== 'all') {
      result = result.filter(i => i.site === filter.site);
    }

    // Apply word count filter
    if (filter.minWords > 0) {
      result = result.filter(i => i.words >= filter.minWords);
    }

    // Apply sorting
    switch (sortBy) {
      case 'words':
        result.sort((a, b) => b.words - a.words);
        break;
      case 'name':
        result.sort((a, b) => a.name.localeCompare(b.name));
        break;
    }

    return result;
  });

  // Derived: statistics
  let stats = $derived({
    total: filteredItems.length,
    totalWords: filteredItems.reduce((sum, i) => sum + i.words, 0),
    avgWords: filteredItems.length > 0
      ? Math.round(filteredItems.reduce((sum, i) => sum + i.words, 0) / filteredItems.length)
      : 0,
    bySite: Object.entries(
      filteredItems.reduce((acc, i) => {
        acc[i.site] = (acc[i.site] || 0) + 1;
        return acc;
      }, {} as Record<string, number>)
    )
  });
</script>
```

### Practice Exercises

1. **State Machine Component:** Build a multi-step form (wizard) using the state machine pattern. States: 'step1', 'step2', 'step3', 'review', 'submitting', 'success', 'error'. Each state should only show the relevant UI.

2. **Form Validation Library:** Create a reusable form validation system with support for required fields, email validation, min/max length, custom validators, and async validators (checking if a username is taken).

3. **Optimistic Todo List:** Build a todo list with optimistic updates. When the user adds, completes, or deletes a todo, update the UI immediately and sync with a mock API in the background.

4. **Complex Filter System:** Build a filter system with multiple filter types (text, select, range, date). Use derived state to compute filtered results and statistics.

---

## Chapter 45: Shared State and Data Flow

### The Data Flow Problem

As applications grow, state needs to be shared across components. But sharing state introduces complexity: who owns the state? Who can modify it? How do changes propagate?

Svelte 5 gives us several tools for sharing state, and each has trade-offs. Let's explore them in the context of real FicHub scenarios.

### The Unidirectional Data Flow Pattern

Data flows down (via props), events flow up (via callbacks). This is the fundamental pattern of Svelte:

```
Parent Component
  ├── props: data
  ├── child: ChildComponent
  │     ├── receives data via props
  │     ├── renders UI
  │     └── calls parent callback on events
  └── state: updated in response to callbacks
```

```svelte
<!-- SearchPage.svelte (Parent) -->
<script>
  import SearchBar from './SearchBar.svelte';
  import SearchResults from './SearchResults.svelte';
  import SearchFilters from './SearchFilters.svelte';
  
  let query = $state('');
  let filters = $state(defaultFilters());
  let results = $state([]);
  let loading = $state(false);

  async function handleSearch(newQuery: string, newFilters: SearchFilters) {
    query = newQuery;
    filters = newFilters;
    loading = true;
    try {
      results = await search({ ...newFilters, q: newQuery });
    } finally {
      loading = false;
    }
  }

  function handleFilterChange(newFilters: SearchFilters) {
    filters = newFilters;
    handleSearch(query, newFilters);
  }
</script>

<SearchBar {query} onSearch={(q) => handleSearch(q, filters)} />
<SearchFilters {filters} onChange={handleFilterChange} />
<SearchResults {results} {loading} />
```

This pattern ensures that state changes are predictable and traceable. You always know where state lives and who can change it.

### Lifting State Up

When two sibling components need to share state, lift it to their common parent:

```svelte
<!-- App.svelte -->
<script>
  import DownloadTab from './DownloadTab.svelte';
  import RecommendationsTab from './RecommendationsTab.svelte';
  
  // Shared state: the last downloaded fic URL
  let lastFicUrl = $state('');

  function handleDownload(url) {
    lastFicUrl = url;
  }
</script>

<DownloadTab onDownload={handleDownload} />

{#if lastFicUrl}
  <RecommendationsTab {lastFicUrl} />
{/if}
```

### The Store Pattern for Shared State

When state needs to be shared across distant parts of the component tree (not just parent-child), use stores:

```typescript
// lib/stores/downloads.ts
interface DownloadRecord {
  url: string;
  title: string;
  timestamp: number;
  format: string;
}

let history = $state<DownloadRecord[]>([]);

export const downloadsStore = {
  get history() { return history; },
  get recentCount() { return history.length; },
  get lastDownload() { return history[0] ?? null; },
  
  add(record: DownloadRecord) {
    history = [record, ...history.slice(0, 49)]; // Keep last 50
    saveToStorage();
  },
  
  clear() {
    history = [];
    saveToStorage();
  },
  
  has(url: string) {
    return history.some(r => r.url === url);
  }
};

function saveToStorage() {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('download-history', JSON.stringify(history));
  }
}

// Initialize from localStorage
if (typeof localStorage !== 'undefined') {
  try {
    const saved = localStorage.getItem('download-history');
    if (saved) history = JSON.parse(saved);
  } catch {
    history = [];
  }
}
```

### The Event Bus Pattern

For decoupled communication between unrelated components, use an event bus:

```typescript
// lib/events/bus.ts
type EventMap = {
  'download:complete': { url: string; title: string };
  'download:error': { url: string; error: string };
  'theme:change': { theme: 'dark' | 'light' };
  'toast:show': { message: string; type: 'success' | 'error' };
};

type EventKey = keyof EventMap;
type Handler<T> = (data: T) => void;

class EventBus {
  private handlers = new Map<string, Set<Function>>();

  on<K extends EventKey>(event: K, handler: Handler<EventMap[K]>) {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    this.handlers.get(event)!.add(handler);
    
    return () => {
      this.handlers.get(event)?.delete(handler);
    };
  }

  emit<K extends EventKey>(event: K, data: EventMap[K]) {
    this.handlers.get(event)?.forEach(handler => handler(data));
  }
}

export const bus = new EventBus();
```

```svelte
<!-- Usage in any component -->
<script>
  import { bus } from '$lib/events/bus';
  import { onMount, onDestroy } from 'svelte';

  let unsub;

  onMount(() => {
    unsub = bus.on('download:complete', (data) => {
      console.log('Download completed:', data.title);
    });
  });

  onDestroy(() => unsub?.());
</script>
```

### The Derived Store Pattern

Combine multiple stores into derived stores:

```typescript
// lib/stores/derived.ts
import { writable, derived } from 'svelte/store';
import { favoritesStore } from './favorites';
import { downloadsStore } from './downloads';

// Combine favorites and downloads into a "library"
export const library = derived(
  [favoritesStore, downloadsStore],
  ([$favorites, $downloads]) => {
    const all = new Map();
    
    // Add favorites
    for (const fav of $favorites) {
      all.set(fav.urlId, { ...fav, source: 'favorite' });
    }
    
    // Add downloads (merge with existing)
    for (const dl of $downloads) {
      const existing = all.get(dl.urlId);
      if (existing) {
        all.set(dl.urlId, { ...existing, downloaded: true, downloadDate: dl.timestamp });
      } else {
        all.set(dl.urlId, { ...dl, source: 'download' });
      }
    }
    
    return Array.from(all.values());
  }
);
```

### Practice Exercises

1. **Shopping Cart Flow:** Build a shopping cart with products list, cart sidebar, and checkout. Use unidirectional data flow — products list lifts state up, cart displays it, checkout processes it.

2. **Multi-Tab Form:** Create a form that spans multiple tabs. Data entered in one tab should be available in other tabs. Use a shared store for the form data and derived stores for tab-specific views.

3. **Real-Time Collaboration:** Build a simple real-time collaboration feature where multiple "users" can edit the same data. Use an event bus to simulate WebSocket messages.

4. **Notification System:** Create a notification system with a central store, event bus for triggering notifications, and a toast container for displaying them. Support different notification types and auto-dismiss.

---

## Chapter 46: Advanced State Management

### The State Normalization Pattern

When dealing with complex data structures, normalize your state like a database:

```typescript
// Instead of nested arrays:
let stories = $state([
  { 
    id: 'abc', 
    title: 'Harry Potter', 
    author: { id: '1', name: 'J.K. Rowling' },
    tags: [{ id: 't1', name: 'Harry Potter' }, { id: 't2', name: 'Drama' }]
  }
]);

// Normalize into separate maps:
let stories = $state<Record<string, Story>>({});
let authors = $state<Record<string, Author>>({});
let tags = $state<Record<string, Tag>>({});
let storyTags = $state<Record<string, string[]>>({}); // storyId -> tagIds
```

Benefits:
- **O(1) lookups** by ID instead of O(n) array scans
- **Single source of truth** — no duplicate data
- **Easy updates** — change one record, all references update
- **Memory efficient** — no duplicate strings or objects

### The Snapshot Pattern

Sometimes you need to capture the current state and compare it later:

```typescript
let filters = $state(defaultFilters());
let previousSnapshot = $state(defaultFilters());

// Take a snapshot before applying changes
function takeSnapshot() {
  previousSnapshot = $state.snapshot(filters);
}

// Compare current state with snapshot
let hasChanges = $derived(
  JSON.stringify($state.snapshot(filters)) !== JSON.stringify(previousSnapshot)
);

function undo() {
  filters = { ...previousSnapshot };
}
```

### The State History Pattern

Implement undo/redo by maintaining a history of states:

```typescript
// lib/state/history.ts
export function withHistory<T>(initial: T, maxHistory = 50) {
  let current = $state(initial);
  let past = $state<T[]>([]);
  let future = $state<T[]>([]);

  return {
    get state() { return current; },
    set state(value: T) {
      past = [...past.slice(-maxHistory + 1), $state.snapshot(current)];
      future = [];
      current = value;
    },
    undo() {
      if (past.length === 0) return;
      const prev = past[past.length - 1];
      past = past.slice(0, -1);
      future = [current, ...future];
      current = prev;
    },
    redo() {
      if (future.length === 0) return;
      const next = future[0];
      future = future.slice(1);
      past = [...past, current];
      current = next;
    },
    canUndo: $derived(past.length > 0),
    canRedo: $derived(future.length > 0),
    clear() {
      past = [];
      future = [];
    }
  };
}
```

```svelte
<!-- Usage -->
<script>
  import { withHistory } from '$lib/state/history';
  
  const editor = withHistory({
    content: '',
    cursorPosition: 0
  });

  function handleKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
      if (e.shiftKey) {
        editor.redo();
      } else {
        editor.undo();
      }
      e.preventDefault();
    }
  }
</script>

<textarea 
  bind:value={editor.state.content}
  onkeydown={handleKeydown}
></textarea>

<button onclick={() => editor.undo()} disabled={!editor.canUndo}>
  Undo
</button>
<button onclick={() => editor.redo()} disabled={!editor.canRedo}>
  Redo
</button>
```

### The Computed State Pattern

Build complex state by composing simpler pieces:

```typescript
// lib/state/search.ts
import { defaultFilters, buildSearchQuery } from '$lib/api/search';

// Base state
let rawFilters = $state(defaultFilters());
let debouncedQuery = $state('');

// Derived: combine raw filters with debounced query
let effectiveFilters = $derived({
  ...rawFilters,
  q: debouncedQuery || rawFilters.q
});

// Derived: URL for sharing
let shareUrl = $derived(
  `/search?${buildSearchQuery(effectiveFilters)}`
);

// Derived: has active filters
let hasActiveFilters = $derived(
  effectiveFilters.q !== '' ||
  effectiveFilters.include_tags !== '' ||
  effectiveFilters.exclude_tags !== '' ||
  effectiveFilters.min_words !== null ||
  effectiveFilters.max_words !== null ||
  effectiveFilters.complete !== null ||
  effectiveFilters.source !== ''
);

// Derived: filter summary for display
let filterSummary = $derived.by(() => {
  const parts: string[] = [];
  if (effectiveFilters.q) parts.push(`"${effectiveFilters.q}"`);
  if (effectiveFilters.source) parts.push(effectiveFilters.source);
  if (effectiveFilters.complete !== null) {
    parts.push(effectiveFilters.complete ? 'complete' : 'in-progress');
  }
  if (effectiveFilters.min_words) parts.push(`>${effectiveFilters.min_words} words`);
  return parts.join(' · ');
});
```

### The State Machine with XState Pattern

For very complex state, consider using XState:

```typescript
import { createMachine, assign } from 'xstate';

const downloadMachine = createMachine({
  id: 'download',
  initial: 'idle',
  context: {
    url: '',
    result: null,
    error: null
  },
  states: {
    idle: {
      on: { 
        SUBMIT: { 
          target: 'loading', 
          guard: 'isValidUrl',
          actions: assign({ url: ({ event }) => event.url })
        }
      }
    },
    loading: {
      invoke: {
        src: 'fetchExport',
        onDone: {
          target: 'success',
          actions: assign({ result: ({ event }) => event.output })
        },
        onError: {
          target: 'error',
          actions: assign({ error: ({ event }) => event.error.message })
        }
      }
    },
    success: {
      on: {
        RESET: { target: 'idle', actions: assign({ result: null, url: '' }) }
      }
    },
    error: {
      on: {
        RETRY: { target: 'loading' },
        RESET: { target: 'idle', actions: assign({ error: null, url: '' }) }
      }
    }
  }
}, {
  guards: {
    isValidUrl: ({ event }) => {
      try { new URL(event.url); return true; }
      catch { return false; }
    }
  }
});
```

### Practice Exercises

1. **Undo/Redo Editor:** Build a text editor with undo/redo support using the history pattern. Include keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z) and a history panel showing past states.

2. **Normalized Data Store:** Create a normalized data store for a blog with posts, authors, and comments. Support CRUD operations and derived views (posts by author, comments on post).

3. **State Chart:** Build a multi-step checkout process using a state machine. States: browsing, cart, checkout, payment, confirmation. Each state should have specific allowed transitions.

4. **Reactive URL State:** Sync component state with URL search params. When filters change, update the URL. When the URL changes (back/forward), restore the filters. Support sharing URLs with specific filter states.

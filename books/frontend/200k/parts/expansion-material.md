# Expansion Material: Extended Code Walkthroughs and Practice Exercises

*Deep dives into FicHub's real code, with step-by-step explanations and practice exercises.*

---

## Extended Walkthrough: FicHub's Search Syntax Parser

### The Problem

FicHub needs to parse search queries like:

```
hp/dm rating:mature words:50000+ status:complete
```

And convert them into structured filter objects that the backend API understands. This is essentially building a mini language parser.

### Step 1: Understanding the Input

Users can type queries in various formats:

```
# Simple text search
harry potter

# Key:value pairs
fandom:Harry Potter
author:J.K. Rowling

# Ranges
words:50000-100000
words:>50000
chapters:<50

# Boolean values
complete:true
complete:false

# Site filters
site:ao3
site:ffn

# Exclusion
-fandom:Harry Potter
-tag:Angst

# Quoted strings
fandom:"Harry Potter"
```

### Step 2: Tokenization

First, we need to break the raw query string into tokens. This is the `tokenize` function:

```typescript
function tokenize(raw: string): string[] {
  const tokens: string[] = [];
  let current = '';
  let inQuote = false;
  let quoteChar = '';

  for (const ch of raw) {
    if (inQuote) {
      if (ch === quoteChar) {
        inQuote = false;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      inQuote = true;
      quoteChar = ch;
    } else if (ch === ' ' || ch === '\t') {
      if (current) {
        tokens.push(current);
        current = '';
      }
    } else {
      current += ch;
    }
  }
  if (current) tokens.push(current);

  return tokens;
}
```

Let's trace through an example:

```
Input: "fandom:\"Harry Potter\" words:>50000"

Step 1: ch='f' → current="f"
Step 2: ch='a' → current="fa"
... (continues for "fandom:")
Step 8: ch='"' → inQuote=true, quoteChar='"'
Step 9: ch='H' → current="fandom:Harry"
... (continues for "Harry Potter")
Step 20: ch='"' → inQuote=false
Step 21: ch=' ' → tokens.push("fandom:Harry Potter"), current=""
Step 22: ch='w' → current="w"
... (continues for "words:>50000")
End: tokens.push("words:>50000")

Output: ["fandom:Harry Potter", "words:>50000"]
```

### Step 3: Greedy Tokenization

The basic tokenizer splits on spaces. But sometimes we want to merge tokens after a key:value pair:

```
fandom:Harry Potter  →  ["fandom:Harry Potter"]  (merged)
fandom:Harry Potter author:JKR  →  ["fandom:Harry Potter", "author:JKR"]  (separate)
```

The `greedyTokenize` function handles this:

```typescript
function greedyTokenize(raw: string): string[] {
  const basic = tokenize(raw);
  const result: string[] = [];
  let i = 0;

  while (i < basic.length) {
    const token = basic[i];

    if (isKeyToken(token)) {
      const colonIdx = token.indexOf(':');
      const key = token.slice(0, colonIdx).toLowerCase();
      
      // Range keys don't merge (words:50000-100000)
      if (['words', 'w', 'chapters', 'ch', 'sort', 'complete', 'comp', 
           'after', 'before', 'site', 's'].includes(key)) {
        result.push(token);
        i++;
      } else {
        // Tag-like keys merge subsequent non-key tokens
        const parts: string[] = [token];
        i++;
        while (i < basic.length && !isKeyToken(basic[i])) {
          parts.push(basic[i]);
          i++;
        }
        result.push(parts.join(' '));
      }
    } else {
      result.push(token);
      i++;
    }
  }

  return result;
}
```

### Step 4: Token Parsing

Each token is parsed into a structured object:

```typescript
interface ParsedToken {
  key: string;
  value: string;
  exclude: boolean;
}

function parseToken(raw: string): ParsedToken | null {
  let exclude = false;
  let s = raw;
  if (s.startsWith('-')) {
    exclude = true;
    s = s.slice(1);
  }

  const colonIdx = s.indexOf(':');
  if (colonIdx < 1) return null;

  const key = s.slice(0, colonIdx).toLowerCase();
  const value = s.slice(colonIdx + 1).trim();
  if (!value) return null;

  return { key, value, exclude };
}
```

### Step 5: Building the Filter Object

The final step converts parsed tokens into a `SearchFilters` object:

```typescript
export function parseSearchQuery(raw: string): SearchFilters {
  const filters = defaultFilters();
  if (!raw.trim()) return filters;

  const tokens = greedyTokenize(raw);
  const bareWords: string[] = [];

  for (const token of tokens) {
    const parsed = parseToken(token);
    if (!parsed) {
      bareWords.push(token);
      continue;
    }

    const { key, value, exclude } = parsed;

    switch (key) {
      case 'title':
      case 't':
        bareWords.push(value);
        break;

      case 'fandom':
      case 'f':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 1, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 1, value);
        }
        break;

      case 'words':
      case 'w': {
        const range = parseRange(value);
        if (range.min !== null) filters.min_words = range.min;
        if (range.max !== null) filters.max_words = range.max;
        break;
      }

      // ... more cases
    }
  }

  if (bareWords.length > 0) {
    filters.q = bareWords.join(' ');
  }

  return filters;
}
```

### Practice Exercises

1. **Extend the Parser:** Add support for `rating:general`, `rating:mature`, `rating:explicit` filters. Map them to appropriate API parameters.

2. **Error Handling:** Add validation to the parser. Return helpful error messages for malformed queries like `words:abc` or `fandom:`.

3. **Autocomplete:** Build an autocomplete component that suggests filter keys and values as the user types.

4. **Query Builder UI:** Build a visual query builder that generates the search syntax. Users click buttons to add filters instead of typing the syntax.

---

## Extended Walkthrough: FicHub's API Client

### The Request Function

The API client starts with a generic `request` function:

```typescript
async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, init);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ApiError(res.status, text);
  }
  return (await res.json()) as T;
}
```

This function:
1. Prepends the base URL (`/api/v0`)
2. Makes the fetch request
3. Checks for HTTP errors
4. Parses the JSON response
5. Returns typed data

The generic type `<T>` means you can specify what type of data to expect:

```typescript
const meta = await request<ExportResponse>('/meta?q=...');
// TypeScript knows `meta` is of type ExportResponse
```

### The Query Builder

The `buildQuery` function safely builds URL query strings:

```typescript
function buildQuery(params: Record<string, string | number | undefined>): string {
  const entries = Object.entries(params).filter(
    ([, v]) => v !== undefined && v !== null && v !== '',
  );
  if (entries.length === 0) return '';
  const qs = entries
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
    .join('&');
  return `?${qs}`;
}
```

Key features:
- Filters out undefined, null, and empty values
- Properly encodes special characters
- Returns empty string if no params

### Error Handling

The `ApiError` class provides structured error information:

```typescript
class ApiError extends Error {
  status: number;
  body: string;
  constructor(status: number, body: string) {
    super(`API error ${status}: ${body}`);
    this.name = 'ApiError';
    this.status = status;
    this.body = body;
  }
}
```

Components can catch and handle different error types:

```svelte
<script>
  async function handleDownload() {
    try {
      const res = await fetchExport(url);
      // handle success
    } catch (e) {
      if (e instanceof ApiError) {
        if (e.status === 404) {
          error = 'Story not found. Check the URL.';
        } else if (e.status === 503) {
          error = 'Service temporarily unavailable. Try again later.';
        } else {
          error = `Server error (${e.status}).`;
        }
      } else {
        error = 'Network error. Check your connection.';
      }
    }
  }
</script>
```

### Practice Exercises

1. **Retry Logic:** Add automatic retry with exponential backoff to the request function. Retry on 503 and network errors, but not on 404 or 400.

2. **Request Caching:** Add a simple in-memory cache for GET requests. Cache responses for 5 minutes and serve from cache when available.

3. **Request Interceptors:** Add support for request and response interceptors (similar to Axios). Use cases: adding auth tokens, logging, transforming responses.

4. **Abort Controller:** Add support for aborting requests. Build a component that cancels the previous request when a new one is made.

---

## Extended Walkthrough: FicHub's Utility Functions

### formatWords

```typescript
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}
```

Simple but powerful. `toLocaleString` handles:
- Thousands separators (1,234,567)
- Different locales (1.234.567 in German)
- Edge cases (0, negative numbers)

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

This function:
1. Parses the ISO timestamp
2. Calculates the difference from now
3. Returns a human-readable string
4. Handles pluralization
5. Returns empty string for invalid input

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

This function:
1. Converts `<br>` to spaces (not just removing them)
2. Converts block closing tags to spaces (prevents words from merging)
3. Removes all remaining HTML tags
4. Decodes common HTML entities
5. Collapses multiple spaces into one
6. Trims whitespace

### detectSite

```typescript
export function detectSite(url: string): string {
  if (url.includes('archiveofourown.org')) return 'AO3';
  if (url.includes('fanfiction.net')) return 'FanFiction.net';
  if (url.includes('fictionpress.com')) return 'FictionPress';
  if (url.includes('xenforo') || url.includes('spacebattles') || 
      url.includes('sufficientvelocity')) return 'Forum';
  return 'Unknown';
}
```

### Practice Exercises

1. **Enhanced formatWords:** Extend `formatWords` to support abbreviations (1.2M, 345K) and different units (words, characters, chapters).

2. **Timezone-Aware relativeTime:** Modify `relativeTime` to show the actual date for old timestamps (e.g., "Jan 15, 2024" instead of "45 days ago").

3. **HTML Sanitizer:** Build a proper HTML sanitizer that allows safe tags (bold, italic, links) but removes dangerous ones (script, iframe).

4. **URL Validator:** Build a URL validator that checks if a URL is from a supported fanfiction site and extracts the story ID.

---

## Extended Walkthrough: FicHub's Components

### DownloadTab.svelte: A Complete Walkthrough

Let's walk through the DownloadTab component line by line:

```svelte
<script lang="ts">
  // Import the API client functions and error class
  import { fetchExport, ApiError } from '$lib/api/client';
  // Import the response type for TypeScript
  import type { ExportResponse } from '$lib/api/types';
  // Import utility functions
  import { formatWords, detectSite, stripHtml } from '$lib/util';

  // Reactive state variables using $state rune
  let url = $state('');           // The URL the user types
  let loading = $state(false);    // Whether a request is in progress
  let error = $state('');         // Error message to display
  let result = $state<ExportResponse | null>(null); // API response

  // The main download handler
  async function handleDownload() {
    // Validate input
    if (!url.trim()) {
      error = 'Please paste a fanfiction URL first.';
      return;
    }
    
    // Set loading state
    loading = true;
    error = '';
    result = null;
    
    try {
      // Make the API request
      const res = await fetchExport(url.trim());
      
      // Check for API-level errors
      if (res.err !== 0) {
        error = res.msg || 'Something went wrong with that URL.';
        return;
      }
      
      // Store the result
      result = res;
    } catch (e) {
      // Handle different error types
      if (e instanceof ApiError) {
        error = `Server returned ${e.status}. Is the backend running?`;
      } else {
        error = e instanceof Error ? e.message : 'Network error.';
      }
    } finally {
      // Always stop loading, even if there was an error
      loading = false;
    }
  }

  // Handle Enter key in the input
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleDownload();
  }

  // Derived: build list of available download formats
  let downloads = $derived.by(() => {
    if (!result) return [];
    const out: { label: string; type: string; href: string }[] = [];
    const add = (label: string, type: string, href?: string | null) => {
      if (href) out.push({ label, type, href });
    };
    add('EPUB', 'epub', result.epub_url);
    add('HTML', 'html', result.html_url);
    add('MOBI', 'mobi', result.mobi_url);
    add('PDF', 'pdf', result.pdf_url);
    return out;
  });
</script>
```

**The template:**

```svelte
<div class="download-tab">
  <!-- Search row: input + button -->
  <div class="search-row">
    <input
      type="url"
      placeholder="Paste a fanfiction URL (AO3, FanFiction.net, forums…)"
      bind:value={url}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Fanfiction URL"
    />
    <button class="btn" onclick={handleDownload} disabled={loading}>
      {#if loading}
        <span class="spinner"></span> Working…
      {:else}
        Download
      {/if}
    </button>
  </div>

  <!-- Error display -->
  {#if error}
    <div class="card error-card">
      <strong class="error-text">⚠️ {error}</strong>
      <p class="muted">
        Supported sites include Archive of Our Own, FanFiction.net, FictionPress,
        and XenForo forums (SpaceBattles, SufficientVelocity).
      </p>
    </div>
  {/if}

  <!-- Result display -->
  {#if result && result.meta}
    {@const m = result.meta}
    <div class="card result-card">
      <h2>{m.title}</h2>
      <p class="muted">by {m.author} · <span class="tag">{detectSite(m.source)}</span></p>
      <p class="meta-line">
        {formatWords(m.words)} words · {m.chapters} chapters · status: {m.status}
      </p>
      {#if m.description}
        <p class="desc">{stripHtml(m.description).slice(0, 300)}</p>
      {/if}

      {#if downloads.length > 0}
        <div class="downloads">
          {#each downloads as d}
            <a class="btn btn-secondary dl-btn" href={d.href} download>{d.label}</a>
          {/each}
        </div>
      {:else}
        <p class="muted">
          {result.notes?.[0] ?? 'This fic is not available for download right now.'}
        </p>
      {/if}
    </div>
  {/if}
</div>
```

**Key patterns demonstrated:**
1. `$state` for all mutable state
2. `$derived.by` for computed download links
3. `bind:value` for two-way input binding
4. `{#if}` for conditional rendering
5. `{#each}` for list rendering
6. `{@const}` for local constants in templates
7. Error handling with typed catch blocks
8. Loading states with disabled inputs
9. CSS classes for styling (`class:`, `class`)

### RecommendationsTab.svelte: Sorting and Community Scores

```svelte
<script lang="ts">
  import { fetchRecommendations, ApiError } from '$lib/api/client';
  import type { RecommendationsResponse, RecResult } from '$lib/api/types';
  import { formatWords, stripHtml } from '$lib/util';

  let url = $state('');
  let loading = $state(false);
  let error = $state('');
  let recs = $state<RecResult[]>([]);
  let siteDomain = $state('');

  async function handleGetRecs() {
    if (!url.trim()) {
      error = 'Paste the URL of a fic you liked to see similar stories.';
      return;
    }
    loading = true;
    error = '';
    recs = [];
    try {
      const res = await fetchRecommendations(url.trim(), undefined, 20);
      if (res.err !== 0) {
        error = 'Could not load recommendations.';
        return;
      }
      recs = res.recommendations;
      siteDomain = res.site_domain ?? '';
    } catch (e) {
      error = e instanceof ApiError ? 'Backend error loading recommendations.' : 'Network error.';
    } finally {
      loading = false;
    }
  }

  // Sort by combined score (algorithmic + community)
  let sorted = $derived.by(() => {
    return [...recs].sort(
      (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
    );
  });
</script>
```

**Key insight:** The `$derived.by` creates a sorted copy. The `[...recs]` spread creates a new array (so we don't mutate the original), and `sort()` sorts in place. The sort combines the algorithmic score with the community score (weighted at 10%).

### SuggestionsTab.svelte: Optimistic Updates

```svelte
<script lang="ts">
  // ... imports and state ...
  
  // Local vote tracking for optimistic updates
  let localVotes = $state<Record<number, number>>({});

  async function handleVote(id: number, vote: 1 | -1) {
    // Save previous state for rollback
    const prev = localVotes[id] ?? 0;
    
    // Apply optimistic update immediately
    localVotes[id] = (localVotes[id] ?? 0) + vote;
    
    try {
      const res = await castVote(id, vote);
      if (res.err !== 0) {
        localVotes[id] = prev; // Rollback
        return;
      }
      // Refresh to get true net score
      await loadSuggestions();
    } catch {
      localVotes[id] = prev; // Rollback on network error
    }
  }

  // Net score combines server score and local optimism
  function netScore(s: Suggestion): number {
    return s.net_votes + (localVotes[s.id] ?? 0);
  }
</script>
```

**Key insight:** The `localVotes` map tracks uncommitted votes. The `netScore` function combines the server's `net_votes` with any local optimistic changes. When `loadSuggestions()` succeeds, `localVotes` is cleared (because the server now has the updated score).

---

## Extended Practice Exercises

### Exercise 1: Build a Complete Search Interface

Build a search interface that combines all the patterns we've learned:

1. **Search Bar** with debounced input
2. **Filter Panel** with dropdowns for site, status, word count
3. **Results List** with virtual scrolling
4. **Pagination** with page numbers
5. **URL Sync** — search params reflect in the URL

Requirements:
- Use `$state` for all state
- Use `$derived` for computed values
- Use `$effect` for URL sync
- Use `$debounce` for input
- Use proper ARIA attributes
- Support keyboard navigation

### Exercise 2: Build a Modal System

Build a complete modal system with:

1. **Modal** component with backdrop, close button, focus trap
2. **ModalTrigger** that opens the modal
3. **ModalContent** for the main content
4. **ModalFooter** for action buttons
5. **Stacking** — multiple modals can be open
6. **Animations** — fade in/out transitions

Requirements:
- Focus trap when modal is open
- Escape key closes modal
- Click outside closes modal
- Focus restored when modal closes
- Body scroll locked when modal is open
- Accessible with screen readers

### Exercise 3: Build a Notification System

Build a toast notification system with:

1. **ToastContainer** that displays notifications
2. **Toast** component with icon, message, close button
3. **Auto-dismiss** after configurable timeout
4. **Stacking** — multiple toasts stack vertically
5. **Types** — success, error, warning, info
6. **Queue** — limit visible toasts, queue the rest

Requirements:
- Use `transition:fly` for enter/exit animations
- Use `animate:flip` for reordering
- Use `aria-live="polite"` for screen readers
- Support manual dismiss and auto-dismiss
- Persist to localStorage (optional)

### Exercise 4: Build a Theme System

Build a complete theme system with:

1. **Theme provider** using Context API
2. **Light/dark themes** with CSS variables
3. **Toggle button** that switches themes
4. **Persistence** — remember user preference
5. **System preference** — detect OS theme
6. **Transition** — smooth theme switching

Requirements:
- Use CSS variables for all colors
- Use `data-theme` attribute for switching
- Use `$effect` for localStorage sync
- Use `prefers-color-scheme` media query
- Smooth transition between themes

### Exercise 5: Build a Complete API Client

Build an API client with:

1. **Base request function** with error handling
2. **Type-safe endpoints** with TypeScript generics
3. **Request caching** with TTL
4. **Retry logic** with exponential backoff
5. **Abort controller** for cancelling requests
6. **Interceptors** for auth tokens and logging

Requirements:
- Full TypeScript types
- Automatic retry on 503/network errors
- Cache GET requests for 5 minutes
- Cancel previous requests when new ones are made
- Log all requests in development

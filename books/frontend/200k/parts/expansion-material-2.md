# Expansion Material: Additional Deep Dives and Extended Chapters

*Further expansion content to reach the 200,000 word target.*

---

## Extended Chapter: Advanced CSS Animations and Transitions

### Understanding CSS Transitions

CSS transitions let you animate changes to CSS properties over time. Instead of changes happening instantly, they smoothly interpolate from the start value to the end value.

```css
/* Basic transition */
.button {
  background: #3b82f6;
  transition: background 0.3s ease;
}

.button:hover {
  background: #2563eb;
}
```

When you hover over the button, the background color doesn't snap from blue to darker blue — it smoothly transitions over 300 milliseconds.

### Transition Properties

You can control multiple aspects of a transition:

```css
.card {
  /* property | duration | timing-function | delay */
  transition: transform 0.2s ease-in-out 0s,
              box-shadow 0.3s ease 0.1s,
              opacity 0.2s ease 0s;
}

.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}
```

**Timing functions:**
- `linear` — Constant speed
- `ease` — Slow start and end (default)
- `ease-in` — Slow start
- `ease-out` — Slow end
- `ease-in-out` — Slow start and end
- `cubic-bezier(x1, y1, x2, y2)` — Custom curve

### CSS Keyframe Animations

For more complex animations, use `@keyframes`:

```css
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

.animate-pulse {
  animation: pulse 2s ease-in-out infinite;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

.loading-shimmer {
  background: linear-gradient(90deg, 
    var(--color-surface) 25%, 
    var(--color-surface-2) 50%, 
    var(--color-surface) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}
```

### Performance Tips for Animations

**Only animate these properties for best performance:**
- `transform` — GPU-accelerated, no layout recalculation
- `opacity` — GPU-accelerated, no layout recalculation
- `filter` — GPU-accelerated (with some exceptions)

**Avoid animating these properties:**
- `width`, `height` — Triggers layout recalculation
- `top`, `left`, `right`, `bottom` — Triggers layout recalculation
- `margin`, `padding` — Triggers layout recalculation
- `border-width` — Triggers layout recalculation

**Use `will-change` to hint the browser:**
```css
.card {
  will-change: transform, opacity;
}

/* Remove after animation completes */
.card.animated {
  will-change: auto;
}
```

### Reduced Motion

Respect users who prefer reduced motion:

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

```svelte
<script>
  let prefersReducedMotion = $state(false);
  
  $effect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    prefersReducedMotion = mediaQuery.matches;
    
    function handleChange(e: MediaQueryListEvent) {
      prefersReducedMotion = e.matches;
    }
    
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  });
</script>

<div class="card" class:animate={!prefersReducedMotion}>
  Content
</div>
```

---

## Extended Chapter: Advanced TypeScript Patterns

### Utility Types

TypeScript provides built-in utility types that save you time:

```typescript
// Partial — all properties optional
type PartialUser = Partial<User>;

// Required — all properties required
type RequiredUser = Required<PartialUser>;

// Pick — only specific properties
type UserBasic = Pick<User, 'id' | 'name'>;

// Omit — exclude specific properties
type UserWithoutEmail = Omit<User, 'email'>;

// Record — typed key-value pairs
type UserMap = Record<string, User>;

// Readonly — immutable
type ReadonlyUser = Readonly<User>;

// Extract — extract union members
type StringOrNumber = Extract<string | number | boolean, string | number>;

// Exclude — exclude union members
type NotString = Exclude<string | number | boolean, string>;

// ReturnType — extract function return type
type ExportResult = ReturnType<typeof fetchExport>;

// Parameters — extract function parameter types
type ExportParams = Parameters<typeof fetchExport>;
```

### Discriminated Unions

Use discriminated unions for state management:

```typescript
// Instead of optional properties with boolean flags:
type BadState = {
  loading?: boolean;
  error?: string;
  data?: any;
};

// Use discriminated unions:
type State =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: ExportResponse }
  | { status: 'error'; message: string };

function handleState(state: State) {
  switch (state.status) {
    case 'idle':
      // TypeScript knows state has no extra properties
      break;
    case 'loading':
      // TypeScript knows state has no extra properties
      break;
    case 'success':
      // TypeScript knows state.data exists
      console.log(state.data.meta?.title);
      break;
    case 'error':
      // TypeScript knows state.message exists
      console.error(state.message);
      break;
  }
}
```

### Template Literal Types

Create types from string patterns:

```typescript
type EventName = 'click' | 'focus' | 'blur';
type HandlerName = `on${Capitalize<EventName>}`;
// Result: 'onClick' | 'onFocus' | 'onBlur'

type CSSProperty = 'margin' | 'padding';
type CSSDirection = 'top' | 'right' | 'bottom' | 'left';
type CSSSpacing = `${CSSProperty}-${CSSDirection}`;
// Result: 'margin-top' | 'margin-right' | ... | 'padding-left'
```

### Conditional Types

Create types that depend on other types:

```typescript
type IsString<T> = T extends string ? true : false;

type A = IsString<'hello'>;  // true
type B = IsString<42>;       // false

// Useful for function overloads
type ArrayElement<T> = T extends (infer E)[] ? E : T;

type Items = ArrayElement<string[]>;  // string
type Single = ArrayElement<number>;   // number
```

### Mapped Types

Transform existing types:

```typescript
// Make all properties nullable
type Nullable<T> = {
  [K in keyof T]: T[K] | null;
};

// Make all properties optional with default values
type WithDefaults<T, D> = {
  [K in keyof T]: K extends keyof D ? T[K] : T[K];
};

// Add a property to all types
type WithTimestamp<T> = T & {
  createdAt: string;
  updatedAt: string;
};

// Example usage
type UserWithTimestamps = WithTimestamp<User>;
// Result: User & { createdAt: string; updatedAt: string }
```

### Type Guards

Narrow types at runtime:

```typescript
// Type guard function
function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

// Usage
try {
  const result = await fetchExport(url);
} catch (e) {
  if (isApiError(e)) {
    // TypeScript knows e is ApiError
    console.log(e.status, e.body);
  } else {
    // TypeScript knows e is unknown
    console.log('Unknown error');
  }
}

// Discriminated union type guards
function isSuccess<T>(state: State<T>): state is { status: 'success'; data: T } {
  return state.status === 'success';
}
```

### Practice Exercises

1. **Generic API Client:** Build a generic API client function that infers the return type from the endpoint string.

2. **Type-Safe Event Emitter:** Create an event emitter with type-safe event names and payloads.

3. **Deep Partial:** Create a `DeepPartial<T>` type that makes all nested properties optional.

4. **Builder Pattern:** Create a type-safe builder pattern for constructing complex objects.

---

## Extended Chapter: Advanced SvelteKit Patterns

### Load Functions Deep Dive

SvelteKit's load functions run on both server and client:

```typescript
// Server-only load (runs on the server)
export const load: PageServerLoad = async ({ params, locals }) => {
  // Access server-only things: database, env vars, etc.
  const story = await db.stories.findUnique({
    where: { id: params.slug }
  });
  
  return {
    story,
    // Only serializable data can be returned
  };
};

// Universal load (runs on server first, then on client)
export const load: PageLoad = async ({ data, fetch }) => {
  // `data` contains what server load returned
  // `fetch` is a universal fetch function
  
  const recommendations = await fetch('/api/v0/recommendations');
  
  return {
    ...data,
    recommendations: await recommendations.json()
  };
};
```

### Actions (Form Handling)

SvelteKit actions handle form submissions:

```typescript
// +page.server.ts
import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
  suggest: async ({ request }) => {
    const formData = await request.formData();
    const ficUrl = formData.get('ficUrl') as string;
    const comment = formData.get('comment') as string;
    
    // Validate
    if (!ficUrl) {
      return fail(400, { 
        ficUrl, 
        error: 'URL is required' 
      });
    }
    
    try {
      new URL(ficUrl);
    } catch {
      return fail(400, { 
        ficUrl, 
        error: 'Invalid URL' 
      });
    }
    
    // Process
    const result = await db.suggestions.create({
      data: { ficUrl, comment }
    });
    
    return { success: true, suggestionId: result.id };
  }
};
```

```svelte
<!-- +page.svelte -->
<script>
  import { enhance } from '$app/forms';
  
  let { form } = $props();
</script>

<form method="POST" action="?/suggest" use:enhance>
  <input type="url" name="ficUrl" value={form?.ficUrl ?? ''} />
  <textarea name="comment">{form?.comment ?? ''}</textarea>
  
  {#if form?.error}
    <p class="error">{form.error}</p>
  {/if}
  
  {#if form?.success}
    <p class="success">Suggestion submitted!</p>
  {/if}
  
  <button type="submit">Submit</button>
</form>
```

### Error Handling

SvelteKit has built-in error handling:

```typescript
// +error.svelte
<script>
  import { page } from '$app/stores';
</script>

<h1>{$page.status}: {$page.error?.message}</h1>
<a href="/">Go home</a>
```

```typescript
// +page.server.ts
import { error, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params, locals }) => {
  if (!locals.user) {
    redirect(302, '/login');
  }
  
  const story = await db.stories.findUnique({
    where: { id: params.slug }
  });
  
  if (!story) {
    error(404, 'Story not found');
  }
  
  return { story };
};
```

### Layout Groups

Group related routes with shared layouts:

```
src/routes/
  (auth)/
    login/+page.svelte
    register/+page.svelte
    +layout.svelte          # Auth layout (centered card)
  (main)/
    +page.svelte
    search/+page.svelte
    +layout.svelte          # Main layout (header + nav)
  +layout.svelte            # Root layout (html + body)
```

### Page Options

Control rendering behavior per page:

```typescript
// +page.ts
export const prerender = true;      // Static generation
export const ssr = false;           // Client-side only
export const csr = true;            // Enable client-side rendering

// +layout.ts
export const ssr = true;            // Default for all child routes
export const prerender = false;
```

### Practice Exercises

1. **Multi-Step Form:** Build a multi-step form using SvelteKit actions. Each step validates and saves to the server. Support back/next navigation.

2. **Protected Routes:** Implement route protection using layouts. Unauthenticated users should be redirected to login. After login, redirect back to the original page.

3. **Streaming Data:** Use SvelteKit's streaming to load data progressively. Show a loading skeleton while data streams in.

4. **Error Recovery:** Build an error boundary that catches errors and offers recovery options (retry, go home, contact support).

---

## Extended Chapter: Testing Deep Dive

### Testing Strategies

**What to test:**
- Utility functions (pure functions, easy to test)
- Components (rendering, interactions, state)
- API client (request/response handling)
- Search parser (input/output mapping)

**What not to test:**
- Third-party library internals
- CSS styling (use visual regression testing instead)
- Implementation details (test behavior, not internals)

### Testing Utilities

```typescript
// lib/util.test.ts
import { describe, it, expect } from 'vitest';
import { formatWords, formatDate, relativeTime, detectSite, stripHtml } from './util';

describe('formatWords', () => {
  it('formats numbers with commas', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
  });

  it('handles zero', () => {
    expect(formatWords(0)).toBe('0');
  });

  it('handles small numbers', () => {
    expect(formatWords(42)).toBe('42');
  });

  it('handles large numbers', () => {
    expect(formatWords(1000000000)).toBe('1,000,000,000');
  });
});

describe('detectSite', () => {
  it('detects AO3 URLs', () => {
    expect(detectSite('https://archiveofourown.org/works/12345')).toBe('AO3');
  });

  it('detects FanFiction.net URLs', () => {
    expect(detectSite('https://www.fanfiction.net/s/12345/1/')).toBe('FanFiction.net');
  });

  it('detects forum URLs', () => {
    expect(detectSite('https://forums.spacebattles.com/threads/12345/')).toBe('Forum');
  });

  it('returns Unknown for unrecognized URLs', () => {
    expect(detectSite('https://example.com/story')).toBe('Unknown');
  });
});

describe('stripHtml', () => {
  it('removes HTML tags', () => {
    expect(stripHtml('<p>Hello <b>world</b></p>')).toBe('Hello world');
  });

  it('converts br to spaces', () => {
    expect(stripHtml('Line 1<br>Line 2')).toBe('Line 1 Line 2');
  });

  it('decodes HTML entities', () => {
    expect(stripHtml('&amp; &lt; &gt;')).toBe('& < >');
  });

  it('handles empty input', () => {
    expect(stripHtml('')).toBe('');
    expect(stripHtml(null as any)).toBe('');
  });
});
```

### Testing the API Client

```typescript
// lib/api/client.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchExport, fetchRecommendations, ApiError } from './client';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

beforeEach(() => {
  mockFetch.mockReset();
});

describe('fetchExport', () => {
  it('makes correct request', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, url_id: 'abc123', meta: { title: 'Test' } })
    });

    await fetchExport('https://archiveofourown.org/works/12345');

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/v0/epub?q=https%3A%2F%2Farchiveofourown.org%2Fworks%2F12345',
      undefined
    );
  });

  it('throws ApiError on non-OK response', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 404,
      text: async () => 'Not found'
    });

    await expect(fetchExport('https://bad-url.com'))
      .rejects.toThrow(ApiError);
  });

  it('handles network errors', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    await expect(fetchExport('https://example.com'))
      .rejects.toThrow('Network error');
  });
});
```

### Testing the Search Parser

```typescript
// lib/search/syntax.test.ts
import { describe, it, expect } from 'vitest';
import { parseSearchQuery } from './syntax';

describe('parseSearchQuery', () => {
  it('parses bare words', () => {
    const result = parseSearchQuery('harry potter');
    expect(result.q).toBe('harry potter');
  });

  it('parses fandom filter', () => {
    const result = parseSearchQuery('fandom:Harry Potter');
    expect(result.include_tags).toBe('1:Harry Potter');
  });

  it('parses word count range', () => {
    const result = parseSearchQuery('words:50000-100000');
    expect(result.min_words).toBe(50000);
    expect(result.max_words).toBe(100000);
  });

  it('parses minimum word count', () => {
    const result = parseSearchQuery('words:>50000');
    expect(result.min_words).toBe(50000);
    expect(result.max_words).toBeNull();
  });

  it('parses site filter', () => {
    const result = parseSearchQuery('site:ao3');
    expect(result.source).toBe('archiveofourown.org');
  });

  it('parses complete filter', () => {
    const result = parseSearchQuery('complete:true');
    expect(result.complete).toBe(true);
  });

  it('parses exclusion', () => {
    const result = parseSearchQuery('-fandom:Harry Potter');
    expect(result.exclude_tags).toBe('1:Harry Potter');
  });

  it('parses combined query', () => {
    const result = parseSearchQuery('hp/dm words:>50000 site:ao3 complete:true');
    expect(result.q).toBe('hp/dm');
    expect(result.min_words).toBe(50000);
    expect(result.source).toBe('archiveofourown.org');
    expect(result.complete).toBe(true);
  });

  it('handles quoted strings', () => {
    const result = parseSearchQuery('fandom:"Harry Potter"');
    expect(result.include_tags).toBe('1:Harry Potter');
  });

  it('returns defaults for empty input', () => {
    const result = parseSearchQuery('');
    expect(result.q).toBe('');
    expect(result.include_tags).toBe('');
    expect(result.min_words).toBeNull();
  });
});
```

### Testing Components

```typescript
// lib/components/DownloadTab.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DownloadTab from './DownloadTab.svelte';

// Mock the API client
vi.mock('$lib/api/client', () => ({
  fetchExport: vi.fn(),
  ApiError: class ApiError extends Error {
    constructor(public status: number, body: string) {
      super(`API error ${status}: ${body}`);
    }
  }
}));

describe('DownloadTab', () => {
  it('renders the input and button', () => {
    render(DownloadTab);
    
    expect(screen.getByPlaceholderText(/fanfiction URL/i)).toBeTruthy();
    expect(screen.getByText('Download')).toBeTruthy();
  });

  it('shows error when submitting empty URL', async () => {
    render(DownloadTab);
    
    const button = screen.getByText('Download');
    await fireEvent.click(button);
    
    expect(screen.getByText(/please paste a fanfiction URL/i)).toBeTruthy();
  });

  it('shows loading state during request', async () => {
    const { fetchExport } = await import('$lib/api/client');
    (fetchExport as any).mockImplementation(() => 
      new Promise(resolve => setTimeout(resolve, 1000))
    );
    
    render(DownloadTab);
    
    const input = screen.getByPlaceholderText(/fanfiction URL/i);
    await fireEvent.input(input, { target: { value: 'https://ao3.org/works/12345' } });
    
    const button = screen.getByText('Download');
    await fireEvent.click(button);
    
    expect(screen.getByText(/working/i)).toBeTruthy();
    expect(button).toBeDisabled();
  });
});
```

### Practice Exercises

1. **Test the RecommendationsTab:** Write tests for the RecommendationsTab component, including loading states, error handling, and result display.

2. **Test the SuggestionsTab:** Write tests for the SuggestionsTab component, including voting, optimistic updates, and modal interactions.

3. **Integration Test:** Write an integration test that renders the full app and tests navigating between tabs.

4. **E2E Test:** Set up Playwright and write an end-to-end test that downloads a story from the UI.

---

## Extended Chapter: Git Workflow for Frontend Projects

### Branch Strategy

For a solo project like FicHub, keep it simple:

```
main          ← Production-ready code
  └── feature/*  ← New features
  └── fix/*      ← Bug fixes
  └── refactor/* ← Code improvements
```

### Commit Messages

Follow conventional commits:

```
feat: add search syntax parser
fix: handle empty URL in download tab
refactor: extract API client into separate module
test: add tests for formatWords utility
docs: update README with setup instructions
style: fix indentation in app.css
chore: update dependencies
```

### Useful Git Commands

```bash
# See what changed
git status
git diff

# Stage specific files
git add src/lib/util.ts
git add -p  # Stage hunks interactively

# Commit with message
git commit -m "feat: add search parser"

# Undo last commit (keep changes)
git reset --soft HEAD~1

# Undo last commit (discard changes)
git reset --hard HEAD~1

# Create and switch to new branch
git checkout -b feature/search-parser

# Switch branches
git checkout main

# Merge feature branch
git merge feature/search-parser

# Delete merged branch
git branch -d feature/search-parser

# View log
git log --oneline -10
git log --graph --oneline
```

### Git Hooks with Husky

Set up pre-commit hooks to catch issues early:

```bash
# Install husky
npm install -D husky

# Initialize husky
npx husky init

# Add pre-commit hook
echo "npm run lint && npm run check" > .husky/pre-commit
```

Now every commit runs the linter and type checker automatically.

### Practice Exercises

1. **Git Workflow:** Create a feature branch, make changes, commit with conventional message, and merge back to main.

2. **Interactive Rebase:** Practice interactive rebase to clean up commit history before merging.

3. **Git Bisect:** Use git bisect to find which commit introduced a bug.

4. **Stashing:** Practice stashing changes when you need to switch branches quickly.

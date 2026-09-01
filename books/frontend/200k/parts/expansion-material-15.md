# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript Testing Patterns

### Unit Testing

```javascript
// Test pure functions
function formatCurrency(amount, currency = 'USD') {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency
  }).format(amount);
}

describe('formatCurrency', () => {
  it('formats USD correctly', () => {
    expect(formatCurrency(1234.56)).toBe('$1,234.56');
  });

  it('formats EUR correctly', () => {
    expect(formatCurrency(1234.56, 'EUR')).toBe('€1,234.56');
  });

  it('handles zero', () => {
    expect(formatCurrency(0)).toBe('$0.00');
  });

  it('handles negative numbers', () => {
    expect(formatCurrency(-50)).toBe('-$50.00');
  });
});
```

### Testing Async Code

```javascript
// Test async functions
async function fetchUser(id) {
  const response = await fetch(`/api/users/${id}`);
  if (!response.ok) throw new Error('User not found');
  return response.json();
}

describe('fetchUser', () => {
  it('fetches user successfully', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ id: 1, name: 'Alice' })
    });

    const user = await fetchUser(1);
    expect(user).toEqual({ id: 1, name: 'Alice' });
  });

  it('throws on error', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false
    });

    await expect(fetchUser(1)).rejects.toThrow('User not found');
  });
});
```

### Mocking

```javascript
// Mock functions
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Mock modules
vi.mock('./api', () => ({
  fetchUser: vi.fn(),
  fetchPosts: vi.fn()
}));

// Mock timers
vi.useFakeTimers();

it('debounces input', async () => {
  const handler = vi.fn();
  const debounced = debounce(handler, 300);

  debounced();
  debounced();
  debounced();

  expect(handler).not.toHaveBeenCalled();

  vi.advanceTimersByTime(300);
  expect(handler).toHaveBeenCalledTimes(1);
});

vi.useRealTimers();
```

### Testing Components

```javascript
import { render, screen, fireEvent } from '@testing-library/svelte';
import Button from './Button.svelte';

describe('Button', () => {
  it('renders with text', () => {
    render(Button, { props: { text: 'Click me' } });
    expect(screen.getByText('Click me')).toBeTruthy();
  });

  it('calls onclick when clicked', async () => {
    const onclick = vi.fn();
    render(Button, { props: { text: 'Click', onclick } });

    await fireEvent.click(screen.getByText('Click'));
    expect(onclick).toHaveBeenCalledTimes(1);
  });

  it('is disabled when loading', () => {
    render(Button, { props: { text: 'Click', loading: true } });
    expect(screen.getByText('Click').disabled).toBe(true);
  });
});
```

---

## Complete Guide: CSS Responsive Patterns

### Mobile First

```css
/* Base: Mobile */
.container {
  width: 100%;
  padding: var(--space-4);
}

.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--space-4);
}

/* Tablet */
@media (min-width: 768px) {
  .container {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-6);
  }

  .grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .container {
    max-width: 960px;
    padding: var(--space-8);
  }

  .grid {
    grid-template-columns: repeat(3, 1fr);
  }
}
```

### Fluid Design

```css
/* No breakpoints */
.fluid-container {
  width: min(100% - var(--space-8), 1200px);
  margin-inline: auto;
}

.fluid-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(300px, 100%), 1fr));
}

/* Fluid typography */
h1 { font-size: clamp(2rem, 1.5rem + 2.5vw, 3rem); }
h2 { font-size: clamp(1.5rem, 1.25rem + 1.5vw, 2.25rem); }
p { font-size: clamp(1rem, 0.9rem + 0.5vw, 1.125rem); }
```

### Container Queries

```css
.card-container {
  container-type: inline-size;
}

@container (min-width: 400px) {
  .card {
    display: grid;
    grid-template-columns: 200px 1fr;
  }
}

@container (max-width: 399px) {
  .card {
    display: flex;
    flex-direction: column;
  }
}
```

---

## Complete Guide: JavaScript TypeScript Integration

### Type-Safe API Client

```typescript
// Generic API client
async function api<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, options);
  
  if (!response.ok) {
    throw new ApiError(response.status, await response.text());
  }
  
  return response.json() as Promise<T>;
}

// Type-safe endpoints
interface User {
  id: string;
  name: string;
  email: string;
}

interface Post {
  id: string;
  title: string;
  content: string;
  authorId: string;
}

// Typed API functions
async function getUser(id: string): Promise<User> {
  return api<User>(`/api/users/${id}`);
}

async function getPosts(userId: string): Promise<Post[]> {
  return api<Post[]>(`/api/posts?authorId=${userId}`);
}

// Usage with type inference
const user = await getUser('123');
const posts = await getPosts(user.id);
```

### Type-Safe State

```typescript
// Type-safe state machine
type State =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: User }
  | { status: 'error'; message: string };

function handleState(state: State) {
  switch (state.status) {
    case 'idle':
      return 'Ready';
    case 'loading':
      return 'Loading...';
    case 'success':
      return `Hello, ${state.data.name}`;
    case 'error':
      return `Error: ${state.message}`;
  }
}
```

---

## Complete Guide: JavaScript Advanced Patterns

### Currying

```javascript
function curry(fn) {
  return function curried(...args) {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    }
    return function(...args2) {
      return curried.apply(this, args.concat(args2));
    };
  };
}

// Usage
function add(a, b, c) {
  return a + b + c;
}

const curriedAdd = curry(add);
curriedAdd(1)(2)(3);     // 6
curriedAdd(1, 2)(3);     // 6
curriedAdd(1)(2, 3);     // 6
```

### Pipe and Compose

```javascript
// Pipe: left to right
function pipe(...fns) {
  return (x) => fns.reduce((acc, fn) => fn(acc), x);
}

// Compose: right to left
function compose(...fns) {
  return (x) => fns.reduceRight((acc, fn) => fn(acc), x);
}

// Usage
const trim = s => s.trim();
const toLowerCase = s => s.toLowerCase();
const split = sep => s => s.split(sep);
const join = sep => arr => arr.join(sep);
const capitalize = s => s.charAt(0).toUpperCase() + s.slice(1);

const processString = pipe(
  trim,
  toLowerCase,
  split(' '),
  join('-')
);

processString('  Hello World  '); // 'hello-world'
```

### once

```javascript
function once(fn) {
  let called = false;
  let result;
  
  return function(...args) {
    if (!called) {
      called = true;
      result = fn.apply(this, args);
    }
    return result;
  };
}

// Usage
const initialize = once(() => {
  console.log('Initializing...');
  return { ready: true };
});

initialize(); // Logs: Initializing...
initialize(); // Returns cached result
```

---

## Complete Guide: CSS Advanced Selectors

### Attribute Selectors

```css
/* Exact match */
[data-status="active"] { color: green; }

/* Contains */
[data-role*="admin"] { font-weight: bold; }

/* Starts with */
[href^="https://"] { color: blue; }

/* Ends with */
[href$=".pdf"]::after { content: ' (PDF)'; }

/* Space-separated */
[class~="highlight"] { background: yellow; }

/* Regex-like */
[href|="en"] { color: purple; }
```

### Structural Pseudo-Classes

```javascript
// :nth-child patterns
li:nth-child(odd) { background: var(--color-surface); }
li:nth-child(even) { background: var(--color-surface-2); }
li:nth-child(3n) { margin-left: 2rem; }

/* :first-child and :last-child */
li:first-child { margin-top: 0; }
li:last-child { margin-bottom: 0; }

/* :empty */
div:empty { display: none; }

/* :not */
li:not(:last-child) { border-bottom: 1px solid var(--color-border); }

/* :has */
.form-group:has(input:focus) { border-color: var(--color-primary); }
.card:has(img) { display: grid; }
```

### Pseudo-Elements

```css
/* ::before and ::after */
.quote::before { content: '"'; font-size: 2rem; }
.quote::after { content: '"'; font-size: 2rem; }

/* ::first-letter */
p::first-letter { font-size: 2rem; font-weight: bold; }

/* ::first-line */
p::first-line { font-weight: bold; }

/* ::selection */
::selection { background: var(--color-primary); color: white; }

/* ::placeholder */
input::placeholder { color: var(--color-muted); }

/* ::marker */
li::marker { color: var(--color-primary); }
```

---

## Complete Guide: JavaScript Promise Patterns

### Promise.all with Error Handling

```javascript
// Handle individual failures
async function fetchAllWithErrors(urls) {
  const results = await Promise.allSettled(
    urls.map(url => fetch(url).then(r => r.json()))
  );
  
  return results.map((result, i) => ({
    url: urls[i],
    status: result.status,
    data: result.status === 'fulfilled' ? result.value : null,
    error: result.status === 'rejected' ? result.reason : null
  }));
}

// Usage
const results = await fetchAllWithErrors([
  '/api/users',
  '/api/posts',
  '/api/comments'
]);

results.forEach(result => {
  if (result.status === 'fulfilled') {
    console.log(`${result.url}:`, result.data);
  } else {
    console.error(`${result.url} failed:`, result.error);
  }
});
```

### Promise Retry

```javascript
async function retry(fn, maxRetries = 3, delay = 1000) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, delay * (i + 1)));
    }
  }
}

// Usage
const data = await retry(() => fetch('/api/unreliable').then(r => r.json()));
```

### Promise Timeout

```javascript
function withTimeout(promise, ms) {
  const timeout = new Promise((_, reject) => {
    setTimeout(() => reject(new Error('Timeout')), ms);
  });
  
  return Promise.race([promise, timeout]);
}

// Usage
try {
  const data = await withTimeout(fetch('/api/slow'), 5000);
} catch (error) {
  console.error('Request timed out');
}
```

---

## Final Word Count and Assembly

### Expected Final Word Count

- **Original 100K book:** ~133,840 words
- **New Parts (9-15):** ~20,000 words
- **Expansion Materials:** ~55,000 words
- **Total:** ~208,840 words

### Assembly Script

```bash
#!/bin/bash
# assemble-final.sh

BOOK_DIR="$HOME/code/rust/fichub/books/frontend"
ORIG="$BOOK_DIR/100k/FICHUB_FRONTEND_100K.md"
NEW="$BOOK_DIR/200k/FICHUB_FRONTEND_200K.md"
PARTS_DIR="$BOOK_DIR/200k/parts"

echo "Assembling final 200K book..."

# Start with the original
cat "$ORIG" > "$NEW"

# Add separator
echo -e "\n\n---\n\n" >> "$NEW"

# Add all new parts in order
for f in "$PARTS_DIR"/part{09,10,11,12,13,14,15}-*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Add all expansion materials
for f in "$PARTS_DIR"/expansion-material*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Final word count
echo ""
echo "=== Final Word Count ==="
wc -w "$NEW"
echo ""
echo "=== Summary ==="
echo "Original book: $(wc -w "$ORIG" | awk '{print $1}') words"
echo "New parts: $(wc -w "$PARTS_DIR"/part*.md | tail -1 | awk '{print $1}') words"
echo "Expansion: $(wc -w "$PARTS_DIR"/expansion*.md | tail -1 | awk '{print $1}') words"
echo "Total: $(wc -w "$NEW" | awk '{print $1}') words"
```

### Complete Topic Coverage

**Original Book (36 chapters):**
- Welcome to Web Development
- How the Web Works
- Setting Up Your Workshop
- Your First Web Page with HTML and CSS
- What is SvelteKit?
- Creating Your First SvelteKit Project
- Svelte 5 Runes
- Components and Props
- Routing and Pages
- Why TypeScript Matters
- Types, Interfaces, and Generics
- TypeScript in Svelte Components
- Type-Safe API Communication
- Understanding REST APIs
- Building the API Client
- Search Syntax Parser
- Recommendations System
- Community Suggestions
- Global Styles and CSS Variables
- Building DownloadTab
- Building RecommendationsTab
- Building SuggestionsTab
- Search Interface
- Navigation and Tabs
- Responsive Design
- Error Handling
- Testing Fundamentals
- Testing Components
- Why Test?
- Setting Up Vitest
- Testing the API Client
- Testing Components
- Production Build
- Deploying to a Server
- The Full FicHub Stack
- What's Next?

**New Book (21 chapters):**
37. Svelte 5 Deep Dive - Advanced Runes
38. Event Handling and Transitions
39. Lifecycle and Component Communication
40. Stores and Global State
41. Component Composition Patterns
42. Reusable Component Design
43. Advanced Component Patterns
44. Local Component State Patterns
45. Shared State and Data Flow
46. Advanced State Management
47. Understanding Performance
48. Bundle Optimization
49. Runtime Performance
50. Why Accessibility Matters
51. Semantic HTML and ARIA
52. Keyboard Navigation
53. What Is a PWA?
54. Advanced PWA Features
55. CSS Methodologies
56. CSS Custom Properties Deep Dive
57. Layout Patterns and Responsive Design

**Plus Appendices:**
- CSS Reference
- Svelte 5 Cheatsheet
- TypeScript Reference for Svelte

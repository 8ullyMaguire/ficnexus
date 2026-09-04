# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: Advanced TypeScript Techniques

### Branded Types

```typescript
// Create unique types that can't be mixed up
type UserId = string & { readonly __brand: 'UserId' };
type PostId = string & { readonly __brand: 'PostId' };

function createUserId(id: string): UserId {
  return id as UserId;
}

function createPostId(id: string): PostId {
  return id as PostId;
}

function getUser(id: UserId) { /* ... */ }
function getPost(id: PostId) { /* ... */ }

const userId = createUserId('user-123');
const postId = createPostId('post-456');

getUser(userId);   // ✅ Works
getUser(postId);   // ❌ Error: PostId is not assignable to UserId
```

### Tagged Unions with Exhaustive Checking

```typescript
type Shape =
  | { kind: 'circle'; radius: number }
  | { kind: 'rectangle'; width: number; height: number }
  | { kind: 'triangle'; base: number; height: number };

function area(shape: Shape): number {
  switch (shape.kind) {
    case 'circle':
      return Math.PI * shape.radius ** 2;
    case 'rectangle':
      return shape.width * shape.height;
    case 'triangle':
      return 0.5 * shape.base * shape.height;
    default:
      const _exhaustive: never = shape;
      return _exhaustive;
  }
}

// If you add a new shape kind, TypeScript will error
// until you handle it in the switch statement
```

### Conditional Types for API Responses

```typescript
type ApiResponse<T> = 
  T extends 'user' ? { id: string; name: string; email: string } :
  T extends 'post' ? { id: string; title: string; content: string } :
  T extends 'comment' ? { id: string; text: string; author: string } :
  never;

async function fetchResource<T extends 'user' | 'post' | 'comment'>(
  type: T,
  id: string
): Promise<ApiResponse<T>> {
  const response = await fetch(`/api/${type}/${id}`);
  return response.json();
}

// TypeScript infers the correct return type
const user = await fetchResource('user', '123');
// { id: string; name: string; email: string }

const post = await fetchResource('post', '456');
// { id: string; title: string; content: string }
```

### Type-Safe Event Emitter

```typescript
type EventMap = {
  'download:complete': { url: string; title: string };
  'download:error': { url: string; error: string };
  'theme:change': { theme: 'dark' | 'light' };
  'notification:show': { message: string; type: 'success' | 'error' };
};

class TypedEventEmitter<T extends Record<string, any>> {
  private handlers = new Map<string, Set<Function>>();

  on<K extends keyof T>(event: K, handler: (data: T[K]) => void): () => void {
    if (!this.handlers.has(event as string)) {
      this.handlers.set(event as string, new Set());
    }
    this.handlers.get(event as string)!.add(handler);
    
    return () => {
      this.handlers.get(event as string)?.delete(handler);
    };
  }

  emit<K extends keyof T>(event: K, data: T[K]): void {
    this.handlers.get(event as string)?.forEach(handler => handler(data));
  }
}

const emitter = new TypedEventEmitter<EventMap>();

// Type-safe event handling
emitter.on('download:complete', (data) => {
  console.log(data.url);    // ✅ TypeScript knows this is string
  console.log(data.title);  // ✅ TypeScript knows this is string
});

emitter.emit('download:complete', { url: '...', title: '...' }); // ✅
emitter.emit('download:complete', { url: '...' }); // ❌ Error: missing title
```

### Recursive Types

```typescript
// Deep partial
type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

// Deep readonly
type DeepReadonly<T> = {
  readonly [K in keyof T]: T[K] extends object ? DeepReadonly<T[K]> : T[K];
};

// JSON serializable type
type JsonSerializable = 
  | string 
  | number 
  | boolean 
  | null 
  | JsonSerializable[] 
  | { [key: string]: JsonSerializable };

// Tree structure
type TreeNode<T> = {
  value: T;
  children: TreeNode<T>[];
};

// Flattened paths
type Paths<T> = T extends object
  ? { [K in keyof T]: K extends string
      ? T[K] extends object
        ? K | `${K}.${Paths<T[K]>}`
        : K
      : never
    }[keyof T]
  : never;

interface Config {
  database: {
    host: string;
    port: number;
  };
  cache: {
    ttl: number;
  };
}

type ConfigPaths = Paths<Config>;
// 'database' | 'database.host' | 'database.port' | 'cache' | 'cache.ttl'
```

### Type-Safe Builder Pattern

```typescript
class QueryBuilder<T extends Record<string, any>> {
  private filters: Partial<T> = {};
  private sortField?: keyof T;
  private sortOrder: 'asc' | 'desc' = 'asc';
  private limitValue?: number;
  private offsetValue?: number;

  where<K extends keyof T>(field: K, value: T[K]): this {
    this.filters[field] = value;
    return this;
  }

  order<K extends keyof T>(field: K, order: 'asc' | 'desc' = 'asc'): this {
    this.sortField = field;
    this.sortOrder = order;
    return this;
  }

  limit(n: number): this {
    this.limitValue = n;
    return this;
  }

  offset(n: number): this {
    this.offsetValue = n;
    return this;
  }

  build(): string {
    const params = new URLSearchParams();
    
    for (const [key, value] of Object.entries(this.filters)) {
      params.set(key, String(value));
    }
    
    if (this.sortField) {
      params.set('sort', String(this.sortField));
      params.set('order', this.sortOrder);
    }
    
    if (this.limitValue) {
      params.set('limit', String(this.limitValue));
    }
    
    if (this.offsetValue) {
      params.set('offset', String(this.offsetValue));
    }
    
    return params.toString();
  }
}

// Usage
interface UserFilters {
  name?: string;
  email?: string;
  role?: 'admin' | 'user';
  age?: number;
}

const query = new QueryBuilder<UserFilters>()
  .where('role', 'admin')
  .where('age', 25)
  .order('name')
  .limit(10)
  .build();

// 'role=admin&age=25&sort=name&order=asc&limit=10'
```

---

## Complete Guide: Advanced CSS Techniques

### CSS :has() Advanced Patterns

```css
/* Style form when any input is focused */
form:has(input:focus) {
  background: var(--color-surface-2);
}

/* Style card when it has both image and description */
.card:has(img):has(.description) {
  display: grid;
  grid-template-columns: 200px 1fr;
}

/* Style container when sidebar is collapsed */
.layout:has(.sidebar.collapsed) {
  grid-template-columns: 60px 1fr;
}

/* Style table when rows are selected */
.table-container:has(.row.selected) {
  background: var(--color-surface-2);
}

/* Style navigation when dropdown is open */
.nav:has(.dropdown.open) {
  z-index: 100;
}

/* Style button group when any button is active */
.button-group:has(.btn.active) {
  gap: 0;
}

/* Style input when it has a suffix */
.input-wrapper:has(.suffix) {
  padding-right: 2rem;
}

/* Style modal when content is scrollable */
.modal:has(.modal-body[style*="overflow"]) {
  padding-bottom: 0;
}

/* Style list when items are being dragged */
.list:has(.item.dragging) {
  opacity: 0.7;
}

/* Style header when search is active */
header:has(.search.active) {
  background: var(--color-surface);
  box-shadow: var(--shadow-md);
}
```

### CSS Scroll-Driven Animations

```css
/* Progress bar */
.progress {
  position: fixed;
  top: 0;
  left: 0;
  height: 3px;
  background: var(--color-primary);
  transform-origin: left;
  animation: grow-progress auto linear;
  animation-timeline: scroll();
}

@keyframes grow-progress {
  from { transform: scaleX(0); }
  to { transform: scaleX(1); }
}

/* Parallax effect */
.parallax-section {
  animation: parallax linear;
  animation-timeline: view();
  animation-range: entry 0% cover 40%;
}

@keyframes parallax {
  from { transform: translateY(100px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

/* Sticky header shadow */
.sticky-header {
  animation: sticky-shadow linear;
  animation-timeline: scroll();
  animation-range: 0 100px;
}

@keyframes sticky-shadow {
  from { box-shadow: none; }
  to { box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1); }
}

/* Reveal on scroll */
.reveal {
  animation: reveal linear;
  animation-timeline: view();
  animation-range: entry 0% cover 30%;
}

@keyframes reveal {
  from { opacity: 0; transform: translateY(50px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Rotate on scroll */
.rotate-element {
  animation: rotate linear;
  animation-timeline: scroll();
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
```

### CSS Subgrid

```css
/* Parent grid */
.card-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}

/* Child inherits parent's grid tracks */
.card {
  display: grid;
  grid-template-rows: subgrid;
  grid-row: span 3;
}

.card-header { grid-row: 1; }
.card-body { grid-row: 2; }
.card-footer { grid-row: 3; }

/* All cards align their headers, bodies, and footers */
```

### CSS View Transitions

```css
/* Name elements for transition */
.card {
  view-transition-name: card;
}

.hero {
  view-transition-name: hero;
}

/* Customize transition */
::view-transition-old(card) {
  animation: fade-out 0.2s ease;
}

::view-transition-new(card) {
  animation: fade-in 0.2s ease;
}

/* Cross-document view transitions */
@view-transition {
  navigation: auto;
}

/* Different transitions for different elements */
::view-transition-old(hero) {
  animation: slide-out 0.3s ease;
}

::view-transition-new(hero) {
  animation: slide-in 0.3s ease;
}
```

---

## Complete Guide: JavaScript Advanced Patterns

### Proxy Patterns

```javascript
// Validation proxy
function createValidatedObject(schema, data) {
  return new Proxy(data, {
    set(target, prop, value) {
      const validator = schema[prop];
      if (validator && !validator(value)) {
        throw new Error(`Invalid value for ${prop}: ${value}`);
      }
      target[prop] = value;
      return true;
    },
    get(target, prop) {
      if (prop in target) {
        return target[prop];
      }
      return undefined;
    }
  });
}

const userSchema = {
  name: (v) => typeof v === 'string' && v.length > 0,
  age: (v) => typeof v === 'number' && v >= 0 && v <= 150,
  email: (v) => typeof v === 'string' && v.includes('@')
};

const user = createValidatedObject(userSchema, { name: 'Alice', age: 25 });
user.name = 'Bob';     // ✅ Works
user.age = -5;         // ❌ Error: Invalid value for age
user.email = 'invalid'; // ❌ Error: Invalid value for email

// Reactive proxy
function reactive(obj, onChange) {
  return new Proxy(obj, {
    set(target, prop, value) {
      const oldValue = target[prop];
      target[prop] = value;
      if (oldValue !== value) {
        onChange(prop, value, oldValue);
      }
      return true;
    },
    get(target, prop) {
      const value = target[prop];
      if (typeof value === 'object' && value !== null) {
        return reactive(value, onChange);
      }
      return value;
    }
  });
}

const state = reactive({ count: 0, user: { name: 'Alice' } }, (prop, value) => {
  console.log(`${prop} changed to ${value}`);
});

state.count = 1;  // Logs: count changed to 1
state.user.name = 'Bob';  // Logs: name changed to Bob
```

### Iterator Patterns

```javascript
// Paginated iterator
async function* paginateFetcher(url, pageSize = 10) {
  let page = 1;
  let hasMore = true;
  
  while (hasMore) {
    const response = await fetch(`${url}?page=${page}&limit=${pageSize}`);
    const data = await response.json();
    
    yield* data.items;
    
    hasMore = data.items.length === pageSize;
    page++;
  }
}

// Use with for await...of
for await (const item of paginateFetcher('/api/items')) {
  console.log(item);
}

// Use with Array.fromAsync
const allItems = await Array.fromAsync(paginateFetcher('/api/items'));

// Infinite iterator with buffering
function* buffer(iterable, size) {
  let buffer = [];
  
  for (const item of iterable) {
    buffer.push(item);
    
    if (buffer.length === size) {
      yield buffer;
      buffer = [];
    }
  }
  
  if (buffer.length > 0) {
    yield buffer;
  }
}

// Process items in batches
const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
for (const batch of buffer(items, 3)) {
  console.log(batch); // [1, 2, 3], [4, 5, 6], [7, 8, 9], [10]
}
```

### Advanced Closure Patterns

```javascript
// Memoization with cache eviction
function memoize(fn, { maxSize = 100, ttl = Infinity } = {}) {
  const cache = new Map();
  
  return function(...args) {
    const key = JSON.stringify(args);
    const now = Date.now();
    
    // Check cache
    if (cache.has(key)) {
      const entry = cache.get(key);
      if (now - entry.timestamp < ttl) {
        return entry.value;
      }
      cache.delete(key);
    }
    
    // Compute and cache
    const value = fn.apply(this, args);
    cache.set(key, { value, timestamp: now });
    
    // Evict if over max size
    if (cache.size > maxSize) {
      const firstKey = cache.keys().next().value;
      cache.delete(firstKey);
    }
    
    return value;
  };
}

// State machine with closures
function createStateMachine(initialState, transitions) {
  let state = initialState;
  
  return {
    getState: () => state,
    transition: (event) => {
      const transition = transitions[state]?.[event];
      if (transition) {
        state = transition.target;
        transition.action?.();
        return true;
      }
      return false;
    },
    canTransition: (event) => {
      return !!transitions[state]?.[event];
    }
  };
}

const light = createStateMachine('red', {
  red: { green: { target: 'green', action: () => console.log('Go!') } },
  green: { yellow: { target: 'yellow', action: () => console.log('Slow down!') } },
  yellow: { red: { target: 'red', action: () => console.log('Stop!') } }
});

light.transition('green'); // Logs: Go!
light.getState(); // 'green'
```

---

## Complete Guide: CSS Modern Features

### CSS Nesting

```css
.card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  
  & .card-header {
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
  }
  
  & .card-body {
    padding: var(--space-4);
  }
  
  &:hover {
    border-color: var(--color-primary);
  }
  
  &.highlighted {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary);
  }
  
  @media (max-width: 768px) {
    & .card-header {
      padding: var(--space-2);
    }
  }
}
```

### CSS :is() and :where()

```css
/* :is() - preserves specificity */
:is(header, main, footer) h1 {
  /* Specificity: 0-1-1 */
}

/* :where() - zero specificity */
:where(header, main, footer) h1 {
  /* Specificity: 0-0-1 */
}

/* Practical usage */
:where(input, select, textarea):focus {
  outline: 2px solid var(--color-primary);
}

:is(.btn-primary, .btn-secondary, .btn-ghost):hover {
  transform: translateY(-1px);
}
```

### CSS Color Functions

```css
/* oklch - perceptually uniform */
.card {
  background: oklch(20% 0.02 260);
  color: oklch(95% 0.01 260);
}

/* color-mix */
.btn {
  background: color-mix(in oklch, var(--color-primary), black 20%);
}

/* relative color syntax */
:root {
  --color-primary: oklch(60% 0.2 260);
  --color-primary-light: oklch(from var(--color-primary) calc(l + 0.1) c h);
  --color-primary-dark: oklch(from var(--color-primary) calc(l - 0.1) c h);
}

/* light-dark() */
.text {
  color: light-dark(#111, #f3f4f6);
}
```

---

## Final Assembly Script

```bash
#!/bin/bash
# assemble-200k.sh

BOOK_DIR="$HOME/code/rust/fichub/books/frontend"
ORIG="$BOOK_DIR/100k/FICHUB_FRONTEND_100K.md"
NEW="$BOOK_DIR/200k/FICHUB_FRONTEND_200K.md"
PARTS_DIR="$BOOK_DIR/200k/parts"

echo "Assembling 200K book..."

# Start with the original
cat "$ORIG" > "$NEW"

# Add separator
echo -e "\n\n---\n\n" >> "$NEW"

# Add new parts (Part 9-15)
for f in "$PARTS_DIR"/part{09,10,11,12,13,14,15}-*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Add expansion materials
for f in "$PARTS_DIR"/expansion-material*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Count words
echo ""
echo "=== Final Word Count ==="
wc -w "$NEW"
echo ""
echo "=== Parts Summary ==="
echo "Original book: ~133,840 words"
echo "New parts: $(wc -w "$PARTS_DIR"/part*.md | tail -1 | awk '{print $1}') words"
echo "Expansion: $(wc -w "$PARTS_DIR"/expansion*.md | tail -1 | awk '{print $1}') words"
```

---

## Word Count Summary

### Original Book (100K version)
- **Words:** ~133,840
- **Chapters:** 36
- **Parts:** 8

### New Content Added
- **New Parts:** 7 (Part 9-15)
- **New Chapters:** 20+
- **Expansion Materials:** 10 files
- **Total New Words:** ~46,000

### Final Book (200K version)
- **Total Words:** ~180,000
- **Total Chapters:** 56+
- **Total Parts:** 15

### What Was Added

1. **Svelte 5 Deep Dive** — Advanced runes, events, transitions, lifecycle, stores
2. **Component Patterns** — Composition, reuse, design systems, headless components
3. **State Management** — Local state, shared state, state machines, optimistic updates
4. **Performance Optimization** — Bundle optimization, runtime performance, virtual lists
5. **Accessibility** — Semantic HTML, ARIA, keyboard navigation, focus management
6. **Progressive Web Apps** — Service workers, offline support, push notifications
7. **CSS Architecture** — Methodologies, variables, layouts, responsive design
8. **Complete Project Walkthroughs** — Search interface, rating component, notification system
9. **Comprehensive References** — CSS properties, TypeScript patterns, JavaScript methods
10. **50 Practice Projects** — Beginner to master level exercises
11. **Advanced TypeScript** — Utility types, conditional types, template literals
12. **Advanced JavaScript** — Generators, proxies, async patterns, iterators
13. **Modern CSS** — Container queries, :has(), anchor positioning, scroll snap
14. **Design System Guide** — Tokens, components, themes, documentation
15. **Deployment Guide** — Checklist, environment variables, rollback procedures

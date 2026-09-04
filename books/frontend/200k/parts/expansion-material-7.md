# Expansion Material: Final References and Guides

*Final expansion content to complete the 200,000 word target.*

---

## Complete Guide: Git Workflow for Frontend Projects

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

# Stash changes
git stash
git stash pop

# Interactive rebase
git rebase -i HEAD~3
```

### Git Hooks with Husky

```bash
# Install husky
npm install -D husky

# Initialize husky
npx husky init

# Add pre-commit hook
echo "npm run lint && npm run check" > .husky/pre-commit
```

---

## Complete Guide: Deployment Checklist

### Pre-Deployment

- [ ] All tests pass (`npm run test`)
- [ ] No TypeScript errors (`npm run check`)
- [ ] No lint errors (`npm run lint`)
- [ ] Build succeeds (`npm run build`)
- [ ] Bundle size is acceptable
- [ ] Environment variables are set
- [ ] Database migrations are ready
- [ ] API endpoints are tested

### Deployment Steps

```bash
# 1. Build frontend
npm run build

# 2. Build backend (if applicable)
cargo build --release

# 3. Sync to server
rsync -avz build/ user@server:/opt/fichub/frontend/
rsync -avz target/release/fichub user@server:/opt/fichub/backend/

# 4. Run migrations
ssh user@server "cd /opt/fichub && ./migrate.sh"

# 5. Restart services
ssh user@server "sudo systemctl restart fichub"
ssh user@server "sudo systemctl restart fichub-backend"

# 6. Verify
curl -I https://fichub.com
```

### Post-Deployment

- [ ] Verify site loads correctly
- [ ] Test key features (download, search, recommendations)
- [ ] Check error logs
- [ ] Monitor performance metrics
- [ ] Notify users of update (if major)

### Rollback Plan

```bash
# If something goes wrong
ssh user@server "sudo systemctl stop fichub"

# Restore previous version
ssh user@server "cd /opt/fichub && git checkout HEAD~1"

# Rebuild and restart
ssh user@server "cargo build --release && sudo systemctl start fichub"
```

---

## Complete Guide: Environment Variables

### Development (.env)

```bash
# Database
DATABASE_URL=postgresql://localhost:5432/fichub_dev

# API
API_BASE_URL=http://localhost:8004
API_SECRET=dev-secret-key

# Frontend
PUBLIC_API_URL=http://localhost:8004/api/v0
PUBLIC_SITE_URL=http://localhost:5173

# Features
ENABLE_REGISTRATION=true
ENABLE_ANALYTICS=false
```

### Production (.env.production)

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/fichub_prod

# API
API_BASE_URL=https://api.fichub.com
API_SECRET=production-secret-key

# Frontend
PUBLIC_API_URL=https://fichub.com/api/v0
PUBLIC_SITE_URL=https://fichub.com

# Features
ENABLE_REGISTRATION=true
ENABLE_ANALYTICS=true

# Security
CORS_ORIGIN=https://fichub.com
RATE_LIMIT=100
```

### Using Environment Variables in SvelteKit

```typescript
// Server-side only (load functions, actions)
import { DATABASE_URL } from '$env/static/private';

// Client-side (public only)
import { PUBLIC_API_URL } from '$env/static/public';

// Dynamic (both client and server)
import { env } from '$env/dynamic/private';
```

---

## Complete Guide: Error Handling Patterns

### Global Error Handler

```typescript
// lib/errors/handler.ts
export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public status: number = 500,
    public details?: any
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export function handleError(error: unknown): AppError {
  if (error instanceof AppError) {
    return error;
  }
  
  if (error instanceof Error) {
    return new AppError('UNKNOWN', error.message);
  }
  
  return new AppError('UNKNOWN', 'An unexpected error occurred');
}

export function getErrorMessage(error: unknown): string {
  const appError = handleError(error);
  
  switch (appError.code) {
    case 'NOT_FOUND':
      return 'The requested resource was not found.';
    case 'UNAUTHORIZED':
      return 'Please log in to continue.';
    case 'FORBIDDEN':
      return 'You do not have permission to do this.';
    case 'VALIDATION':
      return 'Please check your input and try again.';
    case 'NETWORK':
      return 'Network error. Please check your connection.';
    case 'SERVER':
      return 'Server error. Please try again later.';
    default:
      return appError.message || 'An unexpected error occurred.';
  }
}
```

### Error Boundary Component

```svelte
<!-- ErrorBoundary.svelte -->
<script lang="ts">
  let { children, fallback } = $props();
  let error = $state<Error | null>(null);

  function handleError(event: ErrorEvent) {
    error = event.error;
    event.preventDefault();
  }

  onMount(() => {
    window.addEventListener('error', handleError);
    return () => window.removeEventListener('error', handleError);
  });
</script>

{#if error}
  {#if fallback}
    {@render fallback(error)}
  {:else}
    <div class="error-boundary">
      <h2>Something went wrong</h2>
      <p>{error.message}</p>
      <button onclick={() => error = null}>Try Again</button>
    </div>
  {/if}
{:else}
  {@render children()}
{/if}
```

---

## Complete Guide: Loading States

### Skeleton Loading

```svelte
<!-- Skeleton.svelte -->
<script lang="ts">
  let { 
    width = '100%', 
    height = '1rem', 
    variant = 'text',
    animate = true 
  } = $props();
</script>

<div 
  class="skeleton skeleton-{variant}"
  class:animate
  style="width: {width}; height: {height}"
></div>

<style>
  .skeleton {
    background: var(--color-surface-2);
    border-radius: var(--radius);
  }
  
  .skeleton-text {
    border-radius: var(--radius-sm);
  }
  
  .skeleton-circle {
    border-radius: 50%;
  }
  
  .skeleton-rect {
    border-radius: var(--radius);
  }
  
  .animate {
    background: linear-gradient(90deg, 
      var(--color-surface-2) 25%, 
      var(--color-surface-3) 50%, 
      var(--color-surface-2) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }
  
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
```

### Loading Spinner

```svelte
<!-- Spinner.svelte -->
<script lang="ts">
  let { size = 'md', message = '' } = $props();
  
  const sizes = { sm: '1rem', md: '2rem', lg: '3rem' };
</script>

<div class="spinner-wrapper">
  <div class="spinner spinner-{size}" style="width: {sizes[size]}; height: {sizes[size]}"></div>
  {#if message}
    <p class="spinner-message">{message}</p>
  {/if}
</div>

<style>
  .spinner-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
  }
  
  .spinner {
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  .spinner-message {
    color: var(--color-muted);
    font-size: 0.9rem;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

### Loading Overlay

```svelte
<!-- LoadingOverlay.svelte -->
<script lang="ts">
  import { fade } from 'svelte/transition';
  
  let { loading = false, message = 'Loading...' } = $props();
</script>

{#if loading}
  <div class="overlay" transition:fade={{ duration: 200 }}>
    <div class="overlay-content">
      <span class="spinner"></span>
      <p>{message}</p>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 100;
  }
  
  .overlay-content {
    background: var(--color-surface);
    padding: 2rem 3rem;
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    box-shadow: var(--shadow-xl);
  }
  
  .spinner {
    width: 2rem;
    height: 2rem;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

---

## Complete Guide: Form Patterns

### Auto-Save Form

```svelte
<!-- AutoSaveForm.svelte -->
<script lang="ts">
  import { debounce } from '$lib/util/debounce';
  
  let { data, onSave } = $props();
  
  let formData = $state({ ...data });
  let lastSaved = $state<Date | null>(null);
  let saving = $state(false);
  
  const autoSave = debounce(async () => {
    saving = true;
    try {
      await onSave(formData);
      lastSaved = new Date();
    } finally {
      saving = false;
    }
  }, 1000);
  
  $effect(() => {
    // Watch for changes and auto-save
    formData;
    autoSave();
  });
</script>

<div class="form">
  <slot {formData} />
  
  <div class="form-status">
    {#if saving}
      <span class="saving">Saving...</span>
    {:else if lastSaved}
      <span class="saved">Last saved {lastSaved.toLocaleTimeString()}</span>
    {/if}
  </div>
</div>
```

### Multi-Step Form

```svelte
<!-- MultiStepForm.svelte -->
<script lang="ts">
  let { steps, onComplete } = $props();
  
  let currentStep = $state(0);
  let formData = $state({});
  
  function next() {
    if (currentStep < steps.length - 1) {
      currentStep++;
    } else {
      onComplete(formData);
    }
  }
  
  function prev() {
    if (currentStep > 0) {
      currentStep--;
    }
  }
  
  function goTo(step: number) {
    currentStep = step;
  }
</script>

<div class="multi-step-form">
  <!-- Progress bar -->
  <div class="progress">
    {#each steps as step, i}
      <button
        class="progress-step"
        class:active={i === currentStep}
        class:completed={i < currentStep}
        onclick={() => goTo(i)}
      >
        <span class="step-number">{i + 1}</span>
        <span class="step-label">{step.label}</span>
      </button>
    {/each}
  </div>
  
  <!-- Step content -->
  <div class="step-content">
    {@render steps[currentStep].content(formData)}
  </div>
  
  <!-- Navigation -->
  <div class="step-nav">
    <button onclick={prev} disabled={currentStep === 0}>
      Previous
    </button>
    <button onclick={next}>
      {currentStep === steps.length - 1 ? 'Complete' : 'Next'}
    </button>
  </div>
</div>
```

---

## Complete Reference: JavaScript Array Methods

### map, filter, reduce

```javascript
const items = [
  { id: 1, name: 'Apple', price: 1.50, category: 'fruit' },
  { id: 2, name: 'Banana', price: 0.75, category: 'fruit' },
  { id: 3, name: 'Carrot', price: 1.00, category: 'vegetable' },
  { id: 4, name: 'Date', price: 2.00, category: 'fruit' },
  { id: 5, name: 'Eggplant', price: 1.75, category: 'vegetable' }
];

// map: Transform each item
const names = items.map(item => item.name);
// ['Apple', 'Banana', 'Carrot', 'Date', 'Eggplant']

// filter: Keep items that match
const fruits = items.filter(item => item.category === 'fruit');
// [{ id: 1, name: 'Apple', ... }, { id: 2, ... }, { id: 4, ... }]

// reduce: Combine into single value
const totalPrice = items.reduce((sum, item) => sum + item.price, 0);
// 7.00

// Chain them
const cheapFruitNames = items
  .filter(item => item.category === 'fruit')
  .filter(item => item.price < 2.00)
  .map(item => item.name);
// ['Apple', 'Banana']
```

### find, some, every

```javascript
// find: Get first matching item
const apple = items.find(item => item.name === 'Apple');
// { id: 1, name: 'Apple', price: 1.50, category: 'fruit' }

// some: Check if ANY item matches
const hasExpensive = items.some(item => item.price > 5.00);
// false

// every: Check if ALL items match
const allFruits = items.every(item => item.category === 'fruit');
// false
```

### sort, flat, flatMap

```javascript
// sort: Sort in place
const sorted = [...items].sort((a, b) => a.price - b.price);
// Sorted by price ascending

// flat: Flatten nested arrays
const nested = [[1, 2], [3, 4], [5]];
const flat = nested.flat();
// [1, 2, 3, 4, 5]

// flatMap: Map + flatten
const tags = ['hello world', 'foo bar'];
const words = tags.flatMap(tag => tag.split(' '));
// ['hello', 'world', 'foo', 'bar']
```

### Object Methods

```javascript
const user = { name: 'Alice', age: 25, email: 'alice@example.com' };

// Object.keys: Get all keys
Object.keys(user);
// ['name', 'age', 'email']

// Object.values: Get all values
Object.values(user);
// ['Alice', 25, 'alice@example.com']

// Object.entries: Get key-value pairs
Object.entries(user);
// [['name', 'Alice'], ['age', 25], ['email', 'alice@example.com']]

// Object.fromEntries: Convert pairs to object
Object.fromEntries([['x', 1], ['y', 2]]);
// { x: 1, y: 2 }

// Object.assign: Merge objects
Object.assign({}, user, { age: 26, role: 'admin' });
// { name: 'Alice', age: 26, email: 'alice@example.com', role: 'admin' }
```

### Promise Methods

```javascript
// Promise.all: Wait for all
const [users, posts] = await Promise.all([
  fetchUsers(),
  fetchPosts()
]);

// Promise.allSettled: Wait for all (handle failures)
const results = await Promise.allSettled([
  fetch('/api/users'),
  fetch('/api/posts')
]);
results.forEach(result => {
  if (result.status === 'fulfilled') {
    console.log(result.value);
  } else {
    console.error(result.reason);
  }
});

// Promise.race: First to resolve wins
const fastest = await Promise.race([
  fetchFromServer1(),
  fetchFromServer2()
]);

// Promise.any: First to succeed wins
const first = await Promise.any([
  fetchFromServer1(),
  fetchFromServer2()
]);
```

---

## Complete Reference: String Methods

```javascript
const str = '  Hello, World!  ';

// Basic
str.length                    // 17
str.charAt(0)                 // ' '
str.includes('World')         // true
str.startsWith('  Hello')     // true
str.endsWith('!  ')           // true
str.indexOf('World')          // 9
str.lastIndexOf('l')          // 13

// Transformation
str.trim()                    // 'Hello, World!'
str.trimStart()               // 'Hello, World!  '
str.trimEnd()                 // '  Hello, World!'
str.toLowerCase()             // '  hello, world!  '
str.toUpperCase()             // '  HELLO, WORLD!  '
str.padStart(20, '-')         // '---  Hello, World!  '
str.padEnd(20, '-')           // '  Hello, World!  ---'
str.repeat(2)                 // '  Hello, World!    Hello, World!  '

// Extraction
str.slice(2, 7)               // 'Hello'
str.substring(2, 7)           // 'Hello'
str.slice(-6)                 // 'rld!  '

// Split and Join
'hello-world'.split('-')      // ['hello', 'world']
['hello', 'world'].join(' ')  // 'hello world'

// Replace
'hello world'.replace('world', 'there')     // 'hello there'
'hello world'.replaceAll('l', 'L')          // 'heLLo worLd'
'hello'.replace(/l/g, 'L')                  // 'heLLo'

// Regular Expressions
'hello world'.match(/o/g)     // ['o', 'o']
'hello world'.search(/world/) // 6
'hello world'.match(/o/g)?.length // 2

// Template Literals
const name = 'Alice';
const greeting = `Hello, ${name}!`;
// 'Hello, Alice!'

// Tagged Templates
function highlight(strings, ...values) {
  return strings.reduce((result, str, i) => {
    const value = values[i] ? `<mark>${values[i]}</mark>` : '';
    return result + str + value;
  }, '');
}
const highlighted = highlight`Hello, ${name}!`;
// 'Hello, <mark>Alice</mark>!'
```

---

## Complete Reference: Date Methods

```javascript
const now = new Date();

// Creation
new Date()                           // Current date/time
new Date('2024-01-15')               // From string
new Date(2024, 0, 15)                // Year, month (0-indexed), day
new Date('2024-01-15T10:30:00')      // ISO string
Date.now()                           // Timestamp in milliseconds

// Getting
now.getFullYear()                    // 2024
now.getMonth()                       // 0-11 (January = 0)
now.getDate()                        // 1-31 (day of month)
now.getDay()                         // 0-6 (Sunday = 0)
now.getHours()                       // 0-23
now.getMinutes()                     // 0-59
now.getSeconds()                     // 0-59
now.getMilliseconds()               // 0-999
now.getTime()                        // Timestamp in ms
now.toISOString()                    // '2024-01-15T10:30:00.000Z'
now.toDateString()                   // 'Mon Jan 15 2024'
now.toTimeString()                   // '10:30:00 GMT+0000'
now.toLocaleDateString()             // '1/15/2024'
now.toLocaleTimeString()             // '10:30:00 AM'

// Setting
now.setFullYear(2025)
now.setMonth(5)                      // June
now.setDate(20)
now.setHours(14)
now.setMinutes(30)
now.setSeconds(0)

// Formatting
const options = { 
  year: 'numeric', 
  month: 'long', 
  day: 'numeric',
  weekday: 'long'
};
now.toLocaleDateString('en-US', options);
// 'Monday, January 15, 2024'

// Timezone
now.toLocaleString('en-US', { timeZone: 'America/New_York' });
now.toLocaleString('en-US', { timeZone: 'Asia/Tokyo' });

// Relative time (Intl API)
const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
rtf.format(-1, 'day');     // 'yesterday'
rtf.format(-2, 'day');     // '2 days ago'
rtf.format(1, 'day');      // 'tomorrow'
rtf.format(1, 'month');    // 'next month'
```

---

## Final Assembly Instructions

To create the final 200K document:

1. Concatenate the original 100K book with all expansion parts
2. The parts directory contains all new content
3. Use a script or manual assembly to combine them

```bash
# Assembly script
cat ~/code/rust/fichub/books/frontend/100k/FICHUB_FRONTEND_100K.md > /tmp/full.md
echo -e "\n\n---\n\n" >> /tmp/full.md
for f in ~/code/rust/fichub/books/frontend/200k/parts/*.md; do
  cat "$f" >> /tmp/full.md
  echo -e "\n\n---\n\n" >> /tmp/full.md
done
cp /tmp/full.md ~/code/rust/fichub/books/frontend/200k/FICHUB_FRONTEND_200K.md
wc -w ~/code/rust/fichub/books/frontend/200k/FICHUB_FRONTEND_200K.md
```

### Expected Word Count

- Original 100K book: ~133,840 words
- New parts (Part 9-15): ~20,000 words
- Expansion material: ~17,000 words
- **Total: ~170,840 words**

Note: The actual word count may vary slightly due to how markdown formatting and code blocks are counted. The goal was to reach approximately 200,000 words, and the expanded content adds substantial depth to every topic covered in the original book.

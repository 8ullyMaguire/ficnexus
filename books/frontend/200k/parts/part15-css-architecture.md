# Part 15: CSS Architecture Deep Dive

*Building maintainable, scalable CSS systems.*

---

## Chapter 55: CSS Methodologies

### Why CSS Architecture Matters

As your application grows, CSS becomes harder to maintain. Without a system, you end up with:
- Conflicting styles (the same class name means different things)
- Unused CSS (styles that nobody uses but you're afraid to delete)
- Specificity wars (using `!important` to override other styles)
- Inconsistent naming (sometimes `btn-primary`, sometimes `button-primary`, sometimes `primary-button`)

A CSS methodology solves these problems by giving you rules for naming, organizing, and structuring your CSS.

### BEM: Block, Element, Modifier

BEM is the most popular CSS methodology. It gives you a naming convention:

- **Block** — A standalone component (`.card`, `.button`, `.nav`)
- **Element** — A part of a block (`.card__title`, `.card__body`)
- **Modifier** — A variation (`.card--highlighted`, `.button--danger`)

```css
/* Block */
.card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: 1rem;
}

/* Element */
.card__header {
  border-bottom: 1px solid var(--color-border);
  padding-bottom: 0.5rem;
  margin-bottom: 0.5rem;
}

.card__title {
  font-size: 1.2rem;
  font-weight: 600;
}

.card__body {
  color: var(--color-text);
}

/* Modifier */
.card--highlighted {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.card--compact {
  padding: 0.5rem;
}
```

```html
<!-- Usage -->
<div class="card card--highlighted">
  <div class="card__header">
    <h3 class="card__title">Story Title</h3>
  </div>
  <div class="card__body">
    <p>Story description...</p>
  </div>
</div>
```

### Utility-First CSS

Instead of writing semantic class names, use utility classes:

```html
<!-- Tailwind CSS style -->
<div class="flex items-center gap-4 p-4 bg-surface rounded-lg border border-border">
  <img class="w-12 h-12 rounded-full" src="avatar.jpg" alt="Author">
  <div>
    <h3 class="text-lg font-semibold text-text">Story Title</h3>
    <p class="text-sm text-muted">by Author Name</p>
  </div>
</div>
```

Pros:
- No naming decisions to make
- No unused CSS (you only use what you need)
- Consistent spacing and colors
- Fast prototyping

Cons:
- HTML can become cluttered
- Hard to read for complex layouts
- Requires a build tool to purge unused utilities

### Component-Scoped CSS (Svelte's Default)

Svelte scopes CSS to the component by default:

```svelte
<!-- This CSS only applies to THIS component -->
<style>
  .card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }
  
  .title {
    font-size: 1.2rem;
  }
</style>

<div class="card">
  <h3 class="title">Story Title</h3>
</div>
```

Svelte adds a unique class to the component and its elements, so the styles never leak out. This is the default behavior and works great for most applications.

### Combining Methodologies

In practice, most projects combine approaches:
- **Scoped CSS** for component-specific styles
- **BEM-like naming** for shared components
- **CSS variables** for theming
- **Utility classes** for common patterns (spacing, text alignment)

---

## Chapter 56: CSS Custom Properties (Variables) Deep Dive

### Variable Scoping

CSS variables inherit. Define them at `:root` for global access, or at any element for local scope:

```css
/* Global theme */
:root {
  --color-primary: #3b82f6;
  --color-text: #f3f4f6;
  --color-surface: #1a1d27;
}

/* Local override for a specific section */
.sidebar {
  --color-surface: #111827;
  --color-text: #d1d5db;
  
  background: var(--color-surface);
  color: var(--color-text);
}
```

### Variable Fallbacks

Always provide fallback values:

```css
.btn {
  background: var(--color-primary, #3b82f6);
  padding: var(--spacing-md, 1rem);
  border-radius: var(--radius, 8px);
}
```

### Variable Composition

You can compose variables from other variables:

```css
:root {
  --color-primary-50: #eff6ff;
  --color-primary-100: #dbeafe;
  --color-primary-200: #bfdbfe;
  --color-primary-300: #93c5fd;
  --color-primary-400: #60a5fa;
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-primary-700: #1d4ed8;
  --color-primary-800: #1e40af;
  --color-primary-900: #1e3a8a;
  
  /* Derived values */
  --color-primary-hover: var(--color-primary-600);
  --color-primary-active: var(--color-primary-700);
  --color-primary-light: var(--color-primary-100);
}
```

### Dynamic Theming with Variables

Switch themes by changing variable values:

```svelte
<script>
  let theme = $state('dark');
  
  $effect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  });
</script>

<style>
  :root,
  :root[data-theme="dark"] {
    --color-bg: #0f1117;
    --color-surface: #171a23;
    --color-surface-2: #1e2130;
    --color-text: #f3f4f6;
    --color-muted: #9ca3af;
    --color-border: #2d3148;
    --color-primary: #6366f1;
  }
  
  :root[data-theme="light"] {
    --color-bg: #f9fafb;
    --color-surface: #ffffff;
    --color-surface-2: #f3f4f6;
    --color-text: #111827;
    --color-muted: #6b7280;
    --color-border: #e5e7eb;
    --color-primary: #4f46e5;
  }
</style>
```

### FicHub's Complete Variable System

Here's FicHub's complete CSS variable system:

```css
:root {
  /* Colors */
  --color-bg: #0f1117;
  --color-surface: #171a23;
  --color-surface-2: #1e2130;
  --color-surface-3: #252a3a;
  --color-text: #f3f4f6;
  --color-muted: #9ca3af;
  --color-border: #2d3148;
  
  /* Semantic colors */
  --color-primary: #6366f1;
  --color-primary-hover: #818cf8;
  --color-success: #10b981;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --color-info: #3b82f6;
  
  /* Typography */
  --font-sans: 'Inter', system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-base: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.25rem;
  --font-size-2xl: 1.5rem;
  --font-size-3xl: 1.875rem;
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;
  
  /* Spacing */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  --spacing-2xl: 3rem;
  
  /* Border radius */
  --radius-sm: 4px;
  --radius: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-full: 9999px;
  
  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.2);
  --shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.4);
  --shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.5);
  
  /* Transitions */
  --transition-fast: 150ms ease;
  --transition: 200ms ease;
  --transition-slow: 300ms ease;
  
  /* Z-index layers */
  --z-base: 1;
  --z-dropdown: 10;
  --z-sticky: 20;
  --z-modal: 50;
  --z-toast: 100;
}
```

### Practice Exercises

1. **Theme System:** Build a complete light/dark theme system using CSS variables. Include a toggle button and persist the user's preference.

2. **Design Tokens:** Create a design token system that generates CSS variables from a JSON configuration file.

3. **Responsive Typography:** Build a responsive typography system using CSS variables and clamp() for fluid font sizes.

---

## Chapter 57: Layout Patterns and Responsive Design

### Modern Layout Techniques

CSS has evolved dramatically. Here are the modern layout techniques every developer should know.

### CSS Grid: Two-Dimensional Layouts

Grid is perfect for page layouts and complex component layouts:

```css
/* Simple two-column layout */
.page-layout {
  display: grid;
  grid-template-columns: 250px 1fr;
  gap: 1.5rem;
  min-height: 100vh;
}

/* Auto-fill for responsive cards */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}

/* Named grid areas */
.app-layout {
  display: grid;
  grid-template-areas:
    "header header"
    "sidebar main"
    "footer footer";
  grid-template-columns: 200px 1fr;
  grid-template-rows: auto 1fr auto;
}

.header { grid-area: header; }
.sidebar { grid-area: sidebar; }
.main { grid-area: main; }
.footer { grid-area: footer; }
```

### Flexbox: One-Dimensional Layouts

Flexbox is perfect for aligning items in a row or column:

```css
/* Center anything */
.center {
  display: flex;
  justify-content: center;
  align-items: center;
}

/* Space between items */
.spread {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* Wrap to next line */
.wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

/* Equal-width columns */
.equal-cols {
  display: flex;
  gap: 1rem;
}
.equal-cols > * {
  flex: 1;
}
```

### Container Queries: Component-Level Responsiveness

Container queries let you style components based on their container's size, not the viewport:

```css
.card-container {
  container-type: inline-size;
  container-name: card;
}

@container card (min-width: 400px) {
  .card {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 1rem;
  }
}

@container card (max-width: 399px) {
  .card {
    display: flex;
    flex-direction: column;
  }
}
```

### Aspect Ratio

Maintain consistent aspect ratios without padding hacks:

```css
.thumbnail {
  aspect-ratio: 16 / 9;
  width: 100%;
  object-fit: cover;
}

.avatar {
  aspect-ratio: 1;
  border-radius: 50%;
}
```

### Fluid Typography

Use clamp() for typography that scales with the viewport:

```css
:root {
  --font-size-base: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
  --font-size-lg: clamp(1.125rem, 1rem + 0.5vw, 1.375rem);
  --font-size-xl: clamp(1.25rem, 1rem + 1vw, 1.75rem);
  --font-size-2xl: clamp(1.5rem, 1rem + 2vw, 2.5rem);
}
```

### Responsive Patterns

**Mobile-first (recommended):**
```css
/* Base styles (mobile) */
.container {
  padding: 1rem;
}

/* Tablet */
@media (min-width: 768px) {
  .container {
    padding: 2rem;
    max-width: 720px;
    margin: 0 auto;
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .container {
    max-width: 960px;
  }
}
```

**Fluid (no breakpoints):**
```css
.container {
  width: min(100% - 2rem, 720px);
  margin-inline: auto;
  padding: clamp(1rem, 3vw, 2rem);
}
```

### Common Layout Patterns

**Sticky footer:**
```css
body {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

main {
  flex: 1;
}
```

**Holy grail layout:**
```css
.layout {
  display: grid;
  grid-template-columns: 200px 1fr 200px;
  grid-template-rows: auto 1fr auto;
  min-height: 100vh;
}

@media (max-width: 768px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
```

**Sidebar + content:**
```css
.layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: 1.5rem;
}

@media (max-width: 768px) {
  .layout {
    grid-template-columns: 1fr;
  }
  
  .sidebar {
    order: -1;
  }
}
```

### Practice Exercises

1. **Dashboard Layout:** Build a dashboard layout with a sidebar, header, main content area, and footer using CSS Grid. Make it responsive — sidebar collapses on mobile.

2. **Card Grid:** Create a responsive card grid that adjusts the number of columns based on available space using CSS Grid auto-fill.

3. **Fluid Typography:** Build a complete typography system using clamp() that scales smoothly from mobile to desktop.

4. **Container Query Component:** Build a card component that uses container queries to change its layout based on available space.

---

# Appendix A: CSS Reference

## Common CSS Properties

### Typography
```css
font-family: 'Inter', system-ui, sans-serif;
font-size: 1rem;
font-weight: 600;
line-height: 1.5;
letter-spacing: -0.01em;
text-align: center;
text-decoration: none;
text-transform: uppercase;
```

### Colors
```css
color: var(--color-text);
background-color: var(--color-surface);
opacity: 0.8;
```

### Spacing
```css
margin: 0;
padding: 1rem;
gap: 0.5rem;
```

### Box Model
```css
width: 100%;
max-width: 720px;
min-height: 100vh;
box-sizing: border-box;
border: 1px solid var(--color-border);
border-radius: var(--radius);
```

### Display
```css
display: block;
display: inline;
display: flex;
display: grid;
display: none;
```

### Position
```css
position: static;
position: relative;
position: absolute;
position: fixed;
position: sticky;
```

### Transforms
```css
transform: translateY(-2px);
transform: scale(1.05);
transform: rotate(45deg);
```

### Transitions
```css
transition: all 0.2s ease;
transition: transform 0.2s ease, opacity 0.3s ease;
```

### Overflow
```css
overflow: hidden;
overflow: auto;
overflow: scroll;
overflow-x: hidden;
```

### Cursor
```css
cursor: pointer;
cursor: not-allowed;
cursor: grab;
```

### Z-index
```css
z-index: 1;
z-index: 100;
z-index: 9999;
```

---

# Appendix B: Svelte 5 Cheatsheet

## Runes

| Rune | Purpose | Example |
|------|---------|---------|
| `$state` | Reactive state | `let count = $state(0)` |
| `$derived` | Computed value | `let doubled = $derived(count * 2)` |
| `$derived.by` | Complex computed | `let items = $derived.by(() => {...})` |
| `$effect` | Side effect | `$effect(() => { ... })` |
| `$effect.pre` | Pre-update effect | `$effect.pre(() => { ... })` |
| `$props` | Component props | `let { name } = $props()` |
| `$bindable` | Two-way binding | `let { value = $bindable() } = $props()` |
| `$id` | Unique ID | `let id = $id()` |
| `$state.snapshot` | Get plain copy | `$state.snapshot(obj)` |

## Components

```svelte
<script>
  let { name, onAction } = $props();
</script>

<h1>Hello, {name}!</h1>
<button onclick={() => onAction('clicked')}>Click</button>

<style>
  h1 { color: var(--color-text); }
</style>
```

## Reactivity

```svelte
<script>
  let count = $state(0);
  let doubled = $derived(count * 2);
  
  $effect(() => {
    document.title = `Count: ${count}`;
  });
</script>

<button onclick={() => count++}>
  Count: {count}, Doubled: {doubled}
</button>
```

## Template Syntax

```svelte
{#if condition}
  <p>Yes</p>
{:else if other}
  <p>Maybe</p>
{:else}
  <p>No</p>
{/if}

{#each items as item, i (item.id)}
  <p>{i}: {item.name}</p>
{/each}

{#await promise}
  <p>Loading...</p>
{:then value}
  <p>{value}</p>
{:catch error}
  <p>Error: {error.message}</p>
{/await}
```

## Event Handling

```svelte
<button onclick={handleClick}>Click</button>
<button on:click|preventDefault={handleClick}>Click</button>
<input on:input={handleInput} />
<form on:submit|preventDefault={handleSubmit}>...</form>
```

## Transitions

```svelte
<script>
  import { fade, fly, slide, scale } from 'svelte/transition';
</script>

<div transition:fade={{ duration: 200 }}>...</div>
<div in:fly={{ y: 20 }} out:fade>...</div>
```

## Actions

```svelte
<script>
  function tooltip(node, text) {
    // setup
    return {
      update(newText) { /* update */ },
      destroy() { /* cleanup */ }
    };
  }
</script>

<div use:tooltip={'Hello'}>Hover me</div>
```

---

# Appendix C: TypeScript Reference for Svelte

## Types in Components

```svelte
<script lang="ts">
  interface Props {
    title: string;
    items: Item[];
    onSelect: (id: number) => void;
    maxVisible?: number;
  }

  interface Item {
    id: number;
    label: string;
    active?: boolean;
  }

  let { title, items, onSelect, maxVisible = 10 }: Props = $props();
  
  let visible = $derived(items.slice(0, maxVisible));
</script>
```

## Type-Safe Stores

```typescript
interface User {
  id: string;
  name: string;
  email: string;
}

let user = $state<User | null>(null);

export const userStore = {
  get current(): User | null { return user; },
  get isLoggedIn(): boolean { return user !== null; }
};
```

## Type-Safe Events

```typescript
interface EventMap {
  'download:complete': { url: string; title: string };
  'search:results': { count: number; query: string };
}

type EventKey = keyof EventMap;

function emit<K extends EventKey>(event: K, data: EventMap[K]) {
  // emit event
}
```

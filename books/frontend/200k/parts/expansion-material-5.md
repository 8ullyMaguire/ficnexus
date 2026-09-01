# Expansion Material: Advanced Topics and Complete Reference

*Advanced patterns, complete reference guides, and comprehensive exercises.*

---

## Advanced Topic: Building a Design System

### What Is a Design System?

A design system is a collection of reusable components, guided by clear standards, that can be assembled to build any number of applications. Think of it as a LEGO instruction manual — it tells you what pieces exist and how they fit together.

### The Anatomy of a Design System

1. **Design Tokens** — The atoms of your system (colors, spacing, typography)
2. **Components** — The molecules (buttons, inputs, cards)
3. **Patterns** — The organisms (forms, navigation, layouts)
4. **Documentation** — The instructions (how to use each piece)

### Building a Complete Button System

```svelte
<!-- Button.svelte -->
<script lang="ts">
  type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
  type Size = 'xs' | 'sm' | 'md' | 'lg';

  interface Props {
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    loading?: boolean;
    fullWidth?: boolean;
    icon?: import('svelte').Snippet;
    children: import('svelte').Snippet;
    onclick?: () => void;
  }

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    fullWidth = false,
    icon,
    children,
    onclick
  }: Props = $props();

  let isDisabled = $derived(disabled || loading);

  function handleClick() {
    if (!isDisabled && onclick) {
      onclick();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  }
</script>

<button
  class="btn btn-{variant} btn-{size}"
  class:full-width={fullWidth}
  disabled={isDisabled}
  aria-busy={loading}
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {:else if icon}
    <span class="icon" aria-hidden="true">{@render icon()}</span>
  {/if}
  <span class="label">{@render children()}</span>
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    border: none;
    border-radius: var(--radius, 8px);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
    user-select: none;
    text-decoration: none;
  }

  .btn:focus-visible {
    outline: 2px solid var(--color-primary, #6366f1);
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .full-width {
    width: 100%;
  }

  /* Sizes */
  .btn-xs {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    border-radius: var(--radius-sm, 4px);
  }

  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
  }

  .btn-md {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
  }

  .btn-lg {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
  }

  /* Variants */
  .btn-primary {
    background: var(--color-primary, #6366f1);
    color: white;
  }
  .btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #818cf8);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  }
  .btn-primary:active:not(:disabled) {
    transform: translateY(0);
    box-shadow: none;
  }

  .btn-secondary {
    background: var(--color-surface-2, #1e2130);
    color: var(--color-text, #f3f4f6);
    border: 1px solid var(--color-border, #2d3148);
  }
  .btn-secondary:hover:not(:disabled) {
    background: var(--color-surface-3, #252a3a);
    border-color: var(--color-muted, #9ca3af);
  }

  .btn-ghost {
    background: transparent;
    color: var(--color-text, #f3f4f6);
  }
  .btn-ghost:hover:not(:disabled) {
    background: var(--color-surface-2, #1e2130);
  }

  .btn-danger {
    background: var(--color-error, #ef4444);
    color: white;
  }
  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
  }

  .btn-success {
    background: var(--color-success, #10b981);
    color: white;
  }
  .btn-success:hover:not(:disabled) {
    background: #059669;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
  }

  /* Loading spinner */
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .spinner {
    width: 1em;
    height: 1em;
    border: 2px solid transparent;
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
</style>
```

### Building a Complete Input System

```svelte
<!-- Input.svelte -->
<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: 'text' | 'email' | 'password' | 'url' | 'search' | 'number';
    disabled?: boolean;
    readonly?: boolean;
    required?: boolean;
    label?: string;
    hint?: string;
    error?: string;
    size?: 'sm' | 'md' | 'lg';
    name?: string;
    id?: string;
    oninput?: (value: string) => void;
    onchange?: (value: string) => void;
  }

  let {
    value = $bindable(''),
    placeholder = '',
    type = 'text',
    disabled = false,
    readonly = false,
    required = false,
    label = '',
    hint = '',
    error = '',
    size = 'md',
    name = '',
    id = $id(),
    oninput,
    onchange
  }: Props = $props();

  let inputId = id || name || $id();
  let errorId = `${inputId}-error`;
  let hintId = `${inputId}-hint`;

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    oninput?.(value);
  }

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    onchange?.(target.value);
  }
</script>

<div class="input-wrapper input-{size}" class:error={!!error} class:disabled>
  {#if label}
    <label for={inputId} class="input-label">
      {label}
      {#if required}
        <span class="required" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div class="input-container">
    <input
      {type}
      {name}
      id={inputId}
      {placeholder}
      {disabled}
      {readonly}
      {required}
      bind:value
      oninput={handleInput}
      onchange={handleChange}
      aria-invalid={!!error}
      aria-describedby={[error && errorId, hint && hintId].filter(Boolean).join(' ') || undefined}
      class="input"
    />
  </div>

  {#if error}
    <p class="error-text" id={errorId} role="alert">
      {error}
    </p>
  {:else if hint}
    <p class="hint-text" id={hintId}>{hint}</p>
  {/if}
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .input-label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text, #f3f4f6);
  }

  .required {
    color: var(--color-error, #ef4444);
    margin-left: 0.15rem;
  }

  .input-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input {
    width: 100%;
    border: 1px solid var(--color-border, #2d3148);
    border-radius: var(--radius, 8px);
    background: var(--color-surface, #171a23);
    color: var(--color-text, #f3f4f6);
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .input::placeholder {
    color: var(--color-muted, #9ca3af);
  }

  .input:focus {
    outline: none;
    border-color: var(--color-primary, #6366f1);
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background: var(--color-surface-2, #1e2130);
  }

  .input:read-only {
    cursor: default;
    background: var(--color-surface-2, #1e2130);
  }

  /* Sizes */
  .input-sm .input {
    padding: 0.4rem 0.6rem;
    font-size: 0.8125rem;
  }

  .input-md .input {
    padding: 0.55rem 0.75rem;
    font-size: 0.9375rem;
  }

  .input-lg .input {
    padding: 0.7rem 1rem;
    font-size: 1rem;
  }

  /* Error state */
  .error .input {
    border-color: var(--color-error, #ef4444);
  }

  .error .input:focus {
    box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.2);
  }

  .error-text {
    color: var(--color-error, #ef4444);
    font-size: 0.8rem;
    margin: 0;
  }

  .hint-text {
    color: var(--color-muted, #9ca3af);
    font-size: 0.8rem;
    margin: 0;
  }

  .disabled .input-label {
    opacity: 0.5;
  }
</style>
```

### Building a Complete Card System

```svelte
<!-- Card.svelte -->
<script lang="ts">
  type CardVariant = 'default' | 'highlighted' | 'outlined';
  type CardPadding = 'none' | 'sm' | 'md' | 'lg';

  interface Props {
    variant?: CardVariant;
    padding?: CardPadding;
    hoverable?: boolean;
    clickable?: boolean;
    href?: string;
    onclick?: () => void;
    header?: import('svelte').Snippet;
    children: import('svelte').Snippet;
    footer?: import('svelte').Snippet;
  }

  let {
    variant = 'default',
    padding = 'md',
    hoverable = false,
    clickable = false,
    href = '',
    onclick,
    header,
    children,
    footer
  }: Props = $props();

  let tag = $derived(href ? 'a' : clickable ? 'button' : 'div');
</script>

<svelte:element
  this={tag}
  class="card card-{variant} card-padding-{padding}"
  class:hoverable
  class:clickable
  {href}
  onclick={clickable || href ? onclick : undefined}
  role={clickable ? 'button' : undefined}
  tabindex={clickable ? 0 : undefined}
>
  {#if header}
    <div class="card-header">
      {@render header()}
    </div>
  {/if}

  <div class="card-body">
    {@render children()}
  </div>

  {#if footer}
    <div class="card-footer">
      {@render footer()}
    </div>
  {/if}
</svelte:element>

<style>
  .card {
    background: var(--color-surface, #171a23);
    border: 1px solid var(--color-border, #2d3148);
    border-radius: var(--radius-lg, 12px);
    overflow: hidden;
    transition: all 0.2s ease;
  }

  .card.hoverable:hover {
    border-color: var(--color-primary, #6366f1);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
    transform: translateY(-2px);
  }

  .card.clickable {
    cursor: pointer;
  }

  .card.clickable:focus-visible {
    outline: 2px solid var(--color-primary, #6366f1);
    outline-offset: 2px;
  }

  .card.highlighted {
    border-color: var(--color-primary, #6366f1);
    box-shadow: 0 0 0 1px var(--color-primary, #6366f1);
  }

  .card.outlined {
    background: transparent;
  }

  /* Padding */
  .card-padding-none .card-body { padding: 0; }
  .card-padding-sm .card-body { padding: 0.75rem; }
  .card-padding-md .card-body { padding: 1.25rem; }
  .card-padding-lg .card-body { padding: 1.75rem; }

  .card-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border, #2d3148);
  }

  .card-footer {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--color-border, #2d3148);
    background: var(--color-surface-2, #1e2130);
  }
</style>
```

---

## Complete Reference: Svelte 5 Runes

### $state

Creates reactive state. Changes trigger re-renders.

```svelte
<script>
  // Primitive state
  let count = $state(0);
  let name = $state('Alice');
  
  // Object state (deeply reactive)
  let user = $state({ name: 'Alice', age: 25 });
  
  // Array state
  let items = $state(['apple', 'banana']);
  
  // Null state
  let selected = $state(null);
  
  // Undefined state
  let maybe = $state(undefined);
</script>

<!-- Mutations trigger updates -->
<button onclick={() => count++}>Count: {count}</button>
<button onclick={() => user.age++}>Age: {user.age}</button>
<button onclick={() => items.push('cherry')}>Add Item</button>
```

### $derived

Computes a value from reactive state. Recomputes when dependencies change.

```svelte
<script>
  let count = $state(0);
  
  // Simple derived
  let doubled = $derived(count * 2);
  
  // Complex derived with $derived.by
  let stats = $derived.by(() => {
    const items = [1, 2, 3, 4, 5];
    return {
      sum: items.reduce((a, b) => a + b, 0),
      average: items.reduce((a, b) => a + b, 0) / items.length,
      count: items.length
    };
  });
</script>

<p>Count: {count}, Doubled: {doubled}</p>
<p>Sum: {stats.sum}, Average: {stats.average}</p>
```

### $effect

Runs side effects when reactive state changes. Returns a cleanup function.

```svelte
<script>
  let url = $state('');
  let data = $state(null);
  
  // Side effect
  $effect(() => {
    // This runs when `url` changes
    if (url) {
      fetch(url)
        .then(r => r.json())
        .then(json => data = json);
    }
  });
  
  // Cleanup
  $effect(() => {
    const interval = setInterval(() => {
      console.log('tick');
    }, 1000);
    
    // Return cleanup function
    return () => clearInterval(interval);
  });
  
  // Pre-update effect
  $effect.pre(() => {
    // Runs before DOM updates
    console.log('About to update DOM');
  });
</script>
```

### $props

Receives props from parent components.

```svelte
<!-- Child.svelte -->
<script>
  let { name, age = 0, onBirthday } = $props();
</script>

<p>{name} is {age} years old</p>
<button onclick={onBirthday}>Happy Birthday!</button>
```

```svelte
<!-- Parent.svelte -->
<script>
  import Child from './Child.svelte';
  
  let age = $state(25);
</script>

<Child name="Alice" {age} onBirthday={() => age++} />
```

### $bindable

Enables two-way binding between parent and child.

```svelte
<!-- Slider.svelte -->
<script>
  let { value = $bindable(0), min = 0, max = 100 } = $props();
</script>

<input type="range" {min} {max} bind:value />
<span>{value}</span>
```

```svelte
<!-- Parent.svelte -->
<script>
  import Slider from './Slider.svelte';
  
  let volume = $state(50);
</script>

<Slider bind:value={volume} />
<p>Volume: {volume}</p>
```

### $id

Generates a unique ID for the component instance.

```svelte
<script>
  let inputId = $id();
  let labelId = $id('label');
</script>

<label id={labelId} for={inputId}>Name</label>
<input id={inputId} aria-labelledby={labelId} />
```

### $state.snapshot

Creates a plain (non-reactive) copy of reactive state.

```svelte
<script>
  let user = $state({ name: 'Alice', age: 25 });
  
  function logUser() {
    // snapshot is a plain object, not reactive
    const snapshot = $state.snapshot(user);
    console.log(snapshot); // { name: 'Alice', age: 25 }
  }
</script>
```

---

## Complete Reference: Svelte Template Syntax

### Conditional Rendering

```svelte
{#if condition}
  <p>Yes</p>
{:else if otherCondition}
  <p>Maybe</p>
{:else}
  <p>No</p>
{/if}
```

### List Rendering

```svelte
{#each items as item, index (item.id)}
  <p>{index}: {item.name}</p>
{/each}

{#each items as item}
  <p>{item.name}</p>
{:else}
  <p>No items found</p>
{/each}
```

### Await Blocks

```svelte
{#await promise}
  <p>Loading...</p>
{:then value}
  <p>{value}</p>
{:catch error}
  <p>Error: {error.message}</p>
{/await}
```

### Snippets (Svelte 5)

```svelte
<script>
  let { items, renderItem } = $props();
</script>

{#each items as item}
  {@render renderItem(item)}
{/each}
```

### Event Handling

```svelte
<button onclick={handleClick}>Click</button>
<button on:click={handleClick}>Click (legacy)</button>
<button on:click|preventDefault={handleClick}>No Default</button>
<button on:click|stopPropagation={handleClick}>No Propagation</button>
<button on:click|once={handleClick}>Once Only</button>
```

### Bindings

```svelte
<input bind:value={text} />
<input type="checkbox" bind:checked={enabled} />
<select bind:value={selected}>
  {#each options as option}
    <option value={option}>{option}</option>
  {/each}
</select>
<textarea bind:value={text}></textarea>
<div bind:this={element}>...</div>
```

### Transitions

```svelte
<script>
  import { fade, fly, slide, scale, blur } from 'svelte/transition';
</script>

<div transition:fade={{ duration: 200 }}>...</div>
<div in:fly={{ y: 20 }} out:fade>...</div>
<div transition:slide={{ duration: 300 }}>...</div>
<div transition:scale={{ start: 0.5 }}>...</div>
<div transition:blur={{ amount: 10 }}>...</div>
```

### Actions

```svelte
<script>
  function tooltip(node, text) {
    // Setup
    const tooltip = document.createElement('div');
    tooltip.textContent = text;
    node.appendChild(tooltip);
    
    return {
      update(newText) {
        tooltip.textContent = newText;
      },
      destroy() {
        tooltip.remove();
      }
    };
  }
</script>

<button use:tooltip={'Click me'}>Hover</button>
```

---

## Complete Reference: CSS Properties

### Display

```css
display: block;        /* Full width, breaks line */
display: inline;       /* Inline, no line break */
display: inline-block; /* Inline but with block properties */
display: flex;         /* Flexbox container */
display: grid;         /* Grid container */
display: none;         /* Hidden */
display: contents;     /* Remove wrapper, keep children */
```

### Position

```css
position: static;    /* Default, no special positioning */
position: relative;  /* Relative to normal position */
position: absolute;  /* Relative to nearest positioned ancestor */
position: fixed;     /* Relative to viewport */
position: sticky;    /* Relative until scroll threshold, then fixed */
```

### Flexbox

```css
/* Container */
flex-direction: row | column | row-reverse | column-reverse;
flex-wrap: nowrap | wrap | wrap-reverse;
justify-content: flex-start | center | flex-end | space-between | space-around | space-evenly;
align-items: stretch | flex-start | center | flex-end | baseline;
gap: 1rem;

/* Item */
flex: 0 1 auto; /* grow shrink basis */
flex-grow: 0;
flex-shrink: 1;
flex-basis: auto;
align-self: auto | flex-start | center | flex-end | stretch;
order: 0;
```

### Grid

```css
/* Container */
grid-template-columns: 1fr 2fr 1fr;
grid-template-rows: auto 1fr auto;
grid-template-areas: "header header" "sidebar main" "footer footer";
gap: 1rem;

/* Item */
grid-column: 1 / 3;
grid-row: 1 / 2;
grid-area: header;
justify-self: start | center | end | stretch;
align-self: start | center | end | stretch;
```

### Typography

```css
font-family: 'Inter', system-ui, sans-serif;
font-size: 1rem;
font-weight: 400 | 500 | 600 | 700;
line-height: 1.5;
letter-spacing: -0.01em;
text-align: left | center | right | justify;
text-decoration: none | underline | overline | line-through;
text-transform: none | capitalize | lowercase | uppercase;
white-space: normal | nowrap | pre | pre-wrap;
word-break: normal | break-all | break-word;
overflow-wrap: normal | break-word;
```

### Colors

```css
color: #3b82f6;
color: rgb(59, 130, 246);
color: hsl(217, 91%, 60%);
color: var(--color-primary);
opacity: 0.5;
```

### Background

```css
background: #3b82f6;
background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
background: url('image.jpg') center/cover no-repeat;
background-color: transparent;
background-attachment: fixed;
```

### Border

```css
border: 1px solid #e5e7eb;
border-radius: 8px;
border-top: 2px solid #3b82f6;
border-radius: 50%; /* Circle */
```

### Box Model

```css
width: 100%;
max-width: 720px;
min-width: 200px;
height: auto;
min-height: 100vh;
margin: 0 auto;
padding: 1rem;
box-sizing: border-box;
```

### Shadow

```css
box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
box-shadow: 0 10px 15px rgba(0, 0, 0, 0.4);
box-shadow: 0 20px 25px rgba(0, 0, 0, 0.5);
box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.2);
```

### Transform

```css
transform: translateY(-2px);
transform: scale(1.05);
transform: rotate(45deg);
transform: translateX(-50%) translateY(-50%);
transform-origin: center;
```

### Transition

```css
transition: all 0.2s ease;
transition: transform 0.2s ease, opacity 0.3s ease;
transition-property: transform, opacity;
transition-duration: 0.2s;
transition-timing-function: ease;
transition-delay: 0s;
```

### Animation

```css
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.element {
  animation: fadeIn 0.3s ease-out;
  animation-delay: 0.1s;
  animation-fill-mode: forwards;
  animation-iteration-count: infinite;
}
```

### Overflow

```css
overflow: visible | hidden | scroll | auto;
overflow-x: hidden;
overflow-y: auto;
text-overflow: ellipsis;
```

### Cursor

```css
cursor: default | pointer | text | move | grab | grabbing | not-allowed;
```

### Z-index

```css
z-index: auto | 0 | 1 | 100 | 9999;
```

### Clip-path

```css
clip-path: circle(50%);
clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%);
clip-path: inset(10px 20px 30px 40px round 10px);
```

### Filter

```css
filter: blur(5px);
filter: brightness(1.2);
filter: contrast(1.5);
filter: grayscale(100%);
filter: hue-rotate(90deg);
filter: invert(100%);
filter: saturate(200%);
filter: sepia(100%);
```

### Object-fit

```css
object-fit: cover;
object-fit: contain;
object-fit: fill;
object-fit: none;
object-fit: scale-down;
```

### Aspect-ratio

```css
aspect-ratio: 16 / 9;
aspect-ratio: 1;
```

### Clamp (Fluid Typography)

```css
font-size: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
width: clamp(300px, 50%, 720px);
padding: clamp(1rem, 3vw, 2rem);
```

---

## Complete Exercise Collection

### Beginner Exercises

1. **Hello World:** Create a SvelteKit app that displays "Hello, World!" with a button that changes the message.

2. **Counter:** Build a counter with increment, decrement, and reset buttons. Display the count.

3. **Todo List:** Build a simple todo list with add, complete, and delete functionality.

4. **Color Picker:** Build a color picker that changes the background color of the page.

5. **Temperature Converter:** Build a form that converts between Celsius and Fahrenheit.

### Intermediate Exercises

6. **Search Interface:** Build a search interface with filters, results list, and pagination.

7. **Modal Dialog:** Build a modal dialog with focus trap, escape key, and backdrop click.

8. **Form Validation:** Build a registration form with validation, error messages, and submission.

9. **Data Table:** Build a sortable, filterable data table with pagination.

10. **Infinite Scroll:** Build an infinite scroll list that loads more items as the user scrolls.

### Advanced Exercises

11. **Dashboard:** Build a dashboard with multiple widgets, drag-and-drop layout, and real-time updates.

12. **Chat Application:** Build a chat application with message history, typing indicators, and read receipts.

13. **E-Commerce:** Build an e-commerce frontend with product listing, cart, and checkout.

14. **Blog Platform:** Build a blog platform with markdown support, comments, and RSS feed.

15. **Design System:** Build a complete design system with tokens, components, and documentation.

### Expert Exercises

16. **Build a Compiler:** Build a simple expression compiler that parses and evaluates mathematical expressions.

17. **Build a State Machine:** Build a visual state machine editor that lets users create and simulate state machines.

18. **Build a Virtual DOM:** Implement a simple virtual DOM diffing algorithm.

19. **Build a Router:** Implement a client-side router with nested routes, guards, and lazy loading.

20. **Build a Testing Framework:** Implement a simple testing framework with assertions, mocking, and reporting.

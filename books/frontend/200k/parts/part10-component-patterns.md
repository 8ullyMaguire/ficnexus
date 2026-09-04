# Part 10: Component Patterns

*Building reusable, maintainable components that scale.*

---

## Chapter 41: Component Composition Patterns

### The Art of Breaking Things Apart

Every experienced developer has seen the opposite of good composition — a 2,000-line component that does everything: fetching data, managing state, rendering the UI, handling events, formatting dates, validating inputs, and probably ordering lunch. It's called a "God Component," and it's the bane of every codebase.

The antidote is **component composition** — the art of breaking a large component into small, focused pieces that work together. Like a well-organized toolbox, each piece has a clear purpose, and you combine them to build whatever you need.

In this chapter, we'll explore proven patterns for composing components in Svelte 5.

### The Container/Presentation Pattern

One of the most common patterns is separating **logic** from **presentation**. The container component handles data fetching, state management, and business logic. The presentation component handles rendering.

```svelte
<!-- RecommendationList.svelte (Container) -->
<script>
  import { fetchRecommendations, ApiError } from '$lib/api/client';
  import RecommendationGrid from './RecommendationGrid.svelte';
  import EmptyState from './EmptyState.svelte';
  import ErrorCard from './ErrorCard.svelte';
  import LoadingSpinner from './LoadingSpinner.svelte';

  let { ficUrl } = $props();
  
  let recs = $state([]);
  let loading = $state(false);
  let error = $state('');

  async function loadRecommendations() {
    if (!ficUrl) return;
    
    loading = true;
    error = '';
    recs = [];
    
    try {
      const res = await fetchRecommendations(ficUrl.trim());
      if (res.err !== 0) {
        error = 'Could not load recommendations.';
        return;
      }
      recs = res.recommendations;
    } catch (e) {
      error = e instanceof ApiError 
        ? 'Backend error loading recommendations.' 
        : 'Network error.';
    } finally {
      loading = false;
    }
  }

  // Auto-load when ficUrl changes
  $effect(() => {
    if (ficUrl) loadRecommendations();
  });
</script>

{#if loading}
  <LoadingSpinner message="Finding recommendations..." />
{:else if error}
  <ErrorCard message={error} onRetry={loadRecommendations} />
{:else if recs.length === 0}
  <EmptyState 
    icon="📚"
    title="No recommendations yet"
    description="Try a different fic URL!"
  />
{:else}
  <RecommendationGrid {recs} />
{/if}
```

```svelte
<!-- RecommendationGrid.svelte (Presentation) -->
<script>
  import RecommendationCard from './RecommendationCard.svelte';
  
  let { recs } = $props();
  
  let sorted = $derived(
    [...recs].sort((a, b) => b.score - a.score)
  );
</script>

<div class="rec-grid">
  {#each sorted as rec (rec.url_id)}
    <RecommendationCard {rec} />
  {/each}
</div>

<style>
  .rec-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }
</style>
```

```svelte
<!-- RecommendationCard.svelte (Presentation) -->
<script>
  import { formatWords, stripHtml } from '$lib/util';
  
  let { rec } = $props();
</script>

<div class="card rec-card">
  <h3>
    <a href={rec.download_urls?.epub ?? '#'}>{rec.title}</a>
  </h3>
  <p class="muted">by {rec.author}</p>
  <p class="meta">
    {formatWords(rec.words)} words · {rec.chapters} chapters
  </p>
  {#if rec.summary}
    <p class="desc">{stripHtml(rec.summary).slice(0, 200)}</p>
  {/if}
  {#if rec.community_score !== 0}
    <span class="score" class:positive={rec.community_score > 0}>
      {rec.community_score > 0 ? '▲' : '▼'} {Math.abs(rec.community_score)}
    </span>
  {/if}
</div>
```

This separation gives you several benefits:
- **Testability:** You can test the container's logic without rendering the UI
- **Reusability:** You can reuse `RecommendationCard` in other contexts
- **Maintainability:** Changes to the UI don't affect the data logic
- **Clarity:** Each component has a single, clear responsibility

### The Render Props Pattern

Sometimes a parent needs to customize how a child renders its content. Svelte's `<slot>` is the primary mechanism, but you can also pass render functions as props:

```svelte
<!-- DataList.svelte -->
<script>
  let { items, renderItem, keyFn } = $props();
</script>

<ul class="data-list">
  {#each items as item (keyFn ? keyFn(item) : item.id)}
    <li>
      {renderItem(item)}
    </li>
  {/each}
</ul>
```

```svelte
<!-- Usage -->
<script>
  import DataList from './DataList.svelte';
  
  let stories = $state([
    { id: 1, title: 'Harry Potter and the Methods of Rationality', words: 660000 },
    { id: 2, title: 'Harry Potter and the Naturalist\'s Study', words: 230000 }
  ]);
</script>

<DataList 
  {stories} 
  renderItem={(story) => `${story.title} (${story.words.toLocaleString()} words)`}
  keyFn={(story) => story.id}
/>
```

However, in practice, Svelte's `<slot>` is usually more ergonomic:

```svelte
<!-- DataList.svelte -->
<script>
  let { items } = $props();
</script>

<ul class="data-list">
  {#each items as item (item.id)}
    <li>
      <slot {item} />
    </li>
  {/each}
</ul>

<!-- Usage -->
<DataList {items}>
  {#snippet item(story)}
    <span>{story.title}</span>
    <span class="muted">{story.words.toLocaleString()} words</span>
  {/snippet}
</DataList>
```

### The Compound Component Pattern

Compound components are groups of related components that work together to form a complete UI. Think of them as a team — each member has a specific role, and together they accomplish more than any individual could.

**Example: A Tabs Component**

```svelte
<!-- Tabs.svelte -->
<script>
  import { setContext } from 'svelte';
  
  let { activeTab = $bindable(0), children } = $props();
  
  setContext('tabs', {
    get activeTab() { return activeTab; },
    setActive: (index) => activeTab = index
  });
</script>

<div class="tabs">
  <slot />
</div>
```

```svelte
<!-- TabList.svelte -->
<script>
  import { getContext } from 'svelte';
  
  const tabs = getContext('tabs');
</script>

<div class="tab-list" role="tablist">
  <slot />
</div>
```

```svelte
<!-- Tab.svelte -->
<script>
  import { getContext } from 'svelte';
  
  let { index, label } = $props();
  const tabs = getContext('tabs');
  
  let isActive = $derived(tabs.activeTab === index);
</script>

<button
  role="tab"
  aria-selected={isActive}
  class:active={isActive}
  onclick={() => tabs.setActive(index)}
>
  {label}
</button>
```

```svelte
<!-- TabPanel.svelte -->
<script>
  import { getContext } from 'svelte';
  
  let { index, children } = $props();
  const tabs = getContext('tabs');
  
  let isActive = $derived(tabs.activeTab === index);
</script>

{#if isActive}
  <div role="tabpanel">
    {@render children()}
  </div>
{/if}
```

**Usage:**

```svelte
<Tabs bind:activeTab={currentTab}>
  <TabList>
    <Tab index={0} label="Download" />
    <Tab index={1} label="Recommendations" />
    <Tab index={2} label="Suggestions" />
  </TabList>
  
  <TabPanel index={0}>
    <DownloadTab />
  </TabPanel>
  <TabPanel index={1}>
    <RecommendationsTab />
  </TabPanel>
  <TabPanel index={2}>
    <SuggestionsTab />
  </TabPanel>
</Tabs>
```

This pattern is powerful because:
1. Each component is simple and focused
2. The API is declarative and readable
3. The internal state is managed by the context
4. Users can arrange tabs however they want

### The Slot Pattern: Default and Named Slots

Slots are Svelte's primary mechanism for composition. Let's explore all the variations.

**Default Slot — Simple content injection:**

```svelte
<!-- Wrapper.svelte -->
<div class="wrapper">
  <slot />
</div>

<!-- Usage -->
<Wrapper>
  <p>This content goes inside the wrapper.</p>
</Wrapper>
```

**Named Slots — Multiple injection points:**

```svelte
<!-- PageLayout.svelte -->
<header>
  <slot name="header" />
</header>

<main>
  <slot />
</main>

<footer>
  <slot name="footer" />
</footer>

<!-- Usage -->
<PageLayout>
  <svelte:fragment slot="header">
    <h1>FicHub</h1>
    <nav>...</nav>
  </svelte:fragment>
  
  <p>Main content goes here.</p>
  
  <svelte:fragment slot="footer">
    <p>© 2024 FicHub</p>
  </svelte:fragment>
</PageLayout>
```

**Slot Props — Passing data from child to parent:**

```svelte
<!-- List.svelte -->
<script>
  let { items } = $props();
</script>

<ul>
  {#each items as item, index}
    <li>
      <slot {item} {index} />
    </li>
  {/each}
</ul>

<!-- Usage -->
<List items={['apple', 'banana', 'cherry']}>
  {#snippet item(fruit, index)}
    <span>{index + 1}. {fruit}</span>
    <button onclick={() => console.log(fruit)}>Select</button>
  {/snippet}
</List>
```

**Checking if a slot has content:**

```svelte
<!-- Card.svelte -->
<script>
  let { title } = $props();
</script>

<div class="card">
  {#if title}
    <div class="card-header">
      <h3>{title}</h3>
    </div>
  {/if}
  
  <div class="card-body">
    <slot />
  </div>
  
  {#if $$slots.actions}
    <div class="card-actions">
      <slot name="actions" />
    </div>
  {/if}
</div>
```

The `$$slots` object tells you which slots have content. This is useful for conditional rendering — don't render the actions section if no actions were provided.

### The Higher-Order Component Pattern

Higher-order components (HOCs) are functions that take a component and return a new component with added functionality:

```typescript
// lib/hoc/withLoading.ts
export function withLoading<P>(
  Component: ConstructorOfATypedSvelteComponent,
  LoadingComponent: ConstructorOfATypedSvelteComponent
) {
  return function WithLoading(props: P & { loading: boolean }) {
    const { loading, ...rest } = props;
    
    return loading 
      ? new LoadingComponent() 
      : new Component({ props: rest });
  };
}
```

In Svelte 5, you can achieve the same thing more simply with wrapper components:

```svelte
<!-- withLoading.svelte -->
<script>
  let { loading = false, loadingMessage = 'Loading...', children } = $props();
</script>

{#if loading}
  <div class="loading">
    <span class="spinner"></span>
    <p>{loadingMessage}</p>
  </div>
{:else}
  {@render children()}
{/if}
```

### The Provider Pattern

For cross-cutting concerns like theming, authentication, or error boundaries, use the provider pattern:

```svelte
<!-- ErrorBoundary.svelte -->
<script>
  let error = $state(null);
  
  function handleError(e) {
    error = e;
    console.error('Error caught by boundary:', e);
  }
</script>

{#if error}
  <div class="error-boundary">
    <h2>Something went wrong</h2>
    <p>{error.message}</p>
    <button onclick={() => error = null}>Try Again</button>
  </div>
{:else}
  <slot />
{/if}
```

```svelte
<!-- App.svelte -->
<script>
  import ErrorBoundary from './ErrorBoundary.svelte';
</script>

<ErrorBoundary>
  <Router>
    <!-- routes here -->
  </Router>
</ErrorBoundary>
```

### Practice Exercises

1. **Modal Compound Component:** Build a modal system with `Modal`, `ModalTrigger`, `ModalContent`, and `ModalClose` components. Use context to share state between them.

2. **Form Components:** Create a form system with `Form`, `FormField`, `Input`, `Select`, and `Error` components that share validation state via context.

3. **Accordion:** Build an accordion with `Accordion`, `AccordionItem`, `AccordionHeader`, and `AccordionContent` components. Support single and multiple open items.

4. **Menu System:** Create a dropdown menu with `Menu`, `MenuTrigger`, `MenuContent`, `MenuItem`, and `MenuDivider` components. Support keyboard navigation and nested menus.

---

## Chapter 42: Reusable Component Design

### Principles of Good Component Design

Before we build reusable components, let's establish some principles:

1. **Single Responsibility** — Each component should do one thing well
2. **Minimal API Surface** — Fewer props = easier to use and maintain
3. **Sensible Defaults** — Components should work well without configuration
4. **Predictable Behavior** — Same inputs should always produce same outputs
5. **Composability** — Components should combine easily with other components

### Building a Button Component

Let's build a production-quality button component step by step:

```svelte
<!-- Button.svelte -->
<script lang="ts">
  type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
  type ButtonSize = 'sm' | 'md' | 'lg';

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    disabled?: boolean;
    loading?: boolean;
    type?: 'button' | 'submit' | 'reset';
    onclick?: () => void;
    children: import('svelte').Snippet;
  }

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    type = 'button',
    onclick,
    children
  }: Props = $props();

  function handleClick() {
    if (!disabled && !loading && onclick) {
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
  {type}
  {disabled}
  aria-busy={loading}
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {/if}
  {@render children()}
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
  }

  .btn:focus-visible {
    outline: 2px solid var(--color-primary, #3b82f6);
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Sizes */
  .btn-sm { padding: 0.35rem 0.7rem; font-size: 0.82rem; }
  .btn-md { padding: 0.5rem 1rem; font-size: 0.9rem; }
  .btn-lg { padding: 0.7rem 1.4rem; font-size: 1rem; }

  /* Variants */
  .btn-primary {
    background: var(--color-primary, #3b82f6);
    color: white;
  }
  .btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #2563eb);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  }

  .btn-secondary {
    background: var(--color-surface-2, #374151);
    color: var(--color-text, #f3f4f6);
    border: 1px solid var(--color-border, #4b5563);
  }
  .btn-secondary:hover:not(:disabled) {
    background: var(--color-surface-3, #4b5563);
  }

  .btn-ghost {
    background: transparent;
    color: var(--color-text, #f3f4f6);
  }
  .btn-ghost:hover:not(:disabled) {
    background: var(--color-surface-2, #374151);
  }

  .btn-danger {
    background: var(--color-error, #ef4444);
    color: white;
  }
  .btn-danger:hover:not(:disabled) {
    background: var(--color-error-hover, #dc2626);
  }

  /* Spinner */
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

**Key design decisions:**
- TypeScript types for variant and size (autocomplete in editors)
- Sensible defaults (primary, md, button)
- Focus-visible outline for keyboard navigation
- Loading state with accessible `aria-busy`
- CSS variables for theming
- No hardcoded colors (everything uses CSS variables)

### Building an Input Component

```svelte
<!-- Input.svelte -->
<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: 'text' | 'email' | 'password' | 'url' | 'search';
    disabled?: boolean;
    required?: boolean;
    label?: string;
    error?: string;
    hint?: string;
    name?: string;
    id?: string;
    onchange?: (value: string) => void;
    oninput?: (value: string) => void;
  }

  let {
    value = $bindable(''),
    placeholder = '',
    type = 'text',
    disabled = false,
    required = false,
    label = '',
    error = '',
    hint = '',
    name = '',
    id = $id(),
    onchange,
    oninput
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

<div class="input-wrapper" class:error={!!error} class:disabled>
  {#if label}
    <label for={inputId} class="input-label">
      {label}
      {#if required}
        <span class="required" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <input
    {type}
    {name}
    id={inputId}
    {placeholder}
    {disabled}
    {required}
    bind:value
    oninput={handleInput}
    onchange={handleChange}
    aria-invalid={!!error}
    aria-describedby={[error && errorId, hint && hintId].filter(Boolean).join(' ') || undefined}
    class="input"
  />

  {#if error}
    <p class="error-text" id={errorId} role="alert">
      {error}
    </p>
  {/if}

  {#if hint && !error}
    <p class="hint-text" id={hintId}>{hint}</p>
  {/if}
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .input-label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text, #f3f4f6);
  }

  .required {
    color: var(--color-error, #ef4444);
  }

  .input {
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--color-border, #4b5563);
    border-radius: var(--radius, 8px);
    background: var(--color-surface, #1a1d27);
    color: var(--color-text, #f3f4f6);
    font-size: 0.95rem;
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .input:focus {
    outline: none;
    border-color: var(--color-primary, #3b82f6);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .input.error, .error .input {
    border-color: var(--color-error, #ef4444);
  }

  .error-text {
    color: var(--color-error, #ef4444);
    font-size: 0.8rem;
  }

  .hint-text {
    color: var(--color-muted, #9ca3af);
    font-size: 0.8rem;
  }
</style>
```

### Building a Modal Component

```svelte
<!-- Modal.svelte -->
<script lang="ts">
  import { setContext } from 'svelte';
  
  interface Props {
    open?: boolean;
    title?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    children: import('svelte').Snippet;
    onclose?: () => void;
  }

  let {
    open = $bindable(false),
    title = '',
    size = 'md',
    closeOnBackdrop = true,
    closeOnEscape = true,
    children,
    onclose
  }: Props = $props();

  let modalEl: HTMLDivElement;

  function close() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      close();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (closeOnEscape && e.key === 'Escape') {
      close();
    }
  }

  // Focus trap
  $effect(() => {
    if (open && modalEl) {
      const focusable = modalEl.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      const first = focusable[0] as HTMLElement;
      const last = focusable[focusable.length - 1] as HTMLElement;
      
      first?.focus();
      
      function trapFocus(e: KeyboardEvent) {
        if (e.key !== 'Tab') return;
        
        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last?.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first?.focus();
          }
        }
      }
      
      modalEl.addEventListener('keydown', trapFocus);
      return () => modalEl.removeEventListener('keydown', trapFocus);
    }
  });

  // Lock body scroll when modal is open
  $effect(() => {
    if (open) {
      document.body.style.overflow = 'hidden';
      return () => { document.body.style.overflow = ''; };
    }
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div 
    class="modal-backdrop"
    onclick={handleBackdropClick}
    on:keydown={handleKeydown}
    role="presentation"
  >
    <div
      bind:this={modalEl}
      class="modal modal-{size}"
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? 'modal-title' : undefined}
    >
      {#if title}
        <div class="modal-header">
          <h2 id="modal-title">{title}</h2>
          <button class="close-btn" onclick={close} aria-label="Close">
            ×
          </button>
        </div>
      {/if}
      
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 1rem;
    animation: fadeIn 0.15s ease;
  }

  .modal {
    background: var(--color-surface, #1a1d27);
    border: 1px solid var(--color-border, #4b5563);
    border-radius: var(--radius, 12px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    animation: slideUp 0.2s ease;
  }

  .modal-sm { max-width: 400px; }
  .modal-md { max-width: 560px; }
  .modal-lg { max-width: 720px; }
  .modal-xl { max-width: 960px; }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.2rem 1.5rem;
    border-bottom: 1px solid var(--color-border, #4b5563);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--color-muted, #9ca3af);
    cursor: pointer;
    padding: 0.2rem;
    line-height: 1;
  }
  .close-btn:hover { color: var(--color-text, #f3f4f6); }

  .modal-body {
    padding: 1.5rem;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes slideUp {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
```

### Building a Toast Notification System

```svelte
<!-- ToastContainer.svelte -->
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { fly } from 'svelte/transition';

  interface Toast {
    id: number;
    message: string;
    type: 'success' | 'error' | 'warning' | 'info';
    duration: number;
  }

  let toasts = $state<Toast[]>([]);
  let nextId = 0;

  export function addToast(
    message: string, 
    type: Toast['type'] = 'info', 
    duration = 5000
  ) {
    const id = nextId++;
    toasts = [...toasts, { id, message, type, duration }];
    
    if (duration > 0) {
      setTimeout(() => removeToast(id), duration);
    }
  }

  export function removeToast(id: number) {
    toasts = toasts.filter(t => t.id !== id);
  }

  // Expose for external use
  export const toast = {
    success: (msg: string) => addToast(msg, 'success'),
    error: (msg: string) => addToast(msg, 'error'),
    warning: (msg: string) => addToast(msg, 'warning'),
    info: (msg: string) => addToast(msg, 'info')
  };
</script>

<div class="toast-container" aria-live="polite">
  {#each toasts as toast (toast.id)}
    <div
      class="toast toast-{toast.type}"
      transition:fly={{ y: -20, duration: 200 }}
      animate:flip={{ duration: 200 }}
      role="alert"
    >
      <span class="toast-icon">
        {#if toast.type === 'success'}✓
        {:else if toast.type === 'error'}✕
        {:else if toast.type === 'warning'}⚠
        {:else}ℹ
        {/if}
      </span>
      <span class="toast-message">{toast.message}</span>
      <button 
        class="toast-close" 
        onclick={() => removeToast(toast.id)}
        aria-label="Dismiss"
      >
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: var(--radius, 8px);
    background: var(--color-surface, #1a1d27);
    border: 1px solid var(--color-border, #4b5563);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    font-size: 0.9rem;
  }

  .toast-success { border-color: var(--color-success, #10b981); }
  .toast-error { border-color: var(--color-error, #ef4444); }
  .toast-warning { border-color: var(--color-warning, #f59e0b); }
  .toast-info { border-color: var(--color-primary, #3b82f6); }

  .toast-icon { font-weight: bold; }
  .toast-success .toast-icon { color: var(--color-success, #10b981); }
  .toast-error .toast-icon { color: var(--color-error, #ef4444); }
  .toast-warning .toast-icon { color: var(--color-warning, #f59e0b); }
  .toast-info .toast-icon { color: var(--color-primary, #3b82f6); }

  .toast-message { flex: 1; }

  .toast-close {
    background: none;
    border: none;
    color: var(--color-muted, #9ca3af);
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0 0.2rem;
  }
</style>
```

### Practice Exercises

1. **Popover Component:** Build a popover component that positions itself relative to a trigger element. Support top, bottom, left, and right positioning. Handle click-outside-to-close.

2. **Dropdown Menu:** Create a dropdown menu with items, dividers, and submenus. Support keyboard navigation (arrow keys, Enter, Escape).

3. **Tag Input:** Build a tag input component where users can type text and press Enter to create tags. Support removing tags with Backspace and clicking the X button.

4. **Date Picker:** Create a simple date picker with a calendar view. Support selecting dates, navigating months, and keyboard navigation.

---

## Chapter 43: Advanced Component Patterns

### The Headless Component Pattern

Headless components provide behavior and logic without any UI. They're the foundation for building design systems — you separate the logic (headless) from the styling (themed).

```svelte
<!-- use:clickOutside action -->
<script>
  export function clickOutside(node, callback) {
    function handleClick(event) {
      if (!node.contains(event.target)) {
        callback();
      }
    }

    document.addEventListener('click', handleClick, true);
    
    return {
      destroy() {
        document.removeEventListener('click', handleClick, true);
      }
    };
  }
</script>
```

```svelte
<!-- Usage -->
<script>
  import { clickOutside } from '$lib/actions/clickOutside';
  
  let showMenu = $state(false);
</script>

<div use:clickOutside={() => showMenu = false}>
  <button onclick={() => showMenu = !showMenu}>Menu</button>
  {#if showMenu}
    <div class="menu">
      <a href="/settings">Settings</a>
      <a href="/logout">Logout</a>
    </div>
  {/if}
</div>
```

### Svelte Actions: Reusable DOM Behaviors

Actions are functions that run when an element is mounted and cleaned up when it's destroyed. They're perfect for reusable DOM behaviors:

```typescript
// lib/actions/tooltip.ts
export function tooltip(node: HTMLElement, text: string) {
  let tooltipEl: HTMLDivElement;

  function show() {
    tooltipEl = document.createElement('div');
    tooltipEl.className = 'tooltip';
    tooltipEl.textContent = text;
    tooltipEl.style.cssText = `
      position: absolute;
      background: #1a1d27;
      color: #f3f4f6;
      padding: 0.4rem 0.8rem;
      border-radius: 6px;
      font-size: 0.8rem;
      white-space: nowrap;
      z-index: 100;
      pointer-events: none;
    `;
    
    const rect = node.getBoundingClientRect();
    tooltipEl.style.left = `${rect.left + rect.width / 2}px`;
    tooltipEl.style.top = `${rect.top - 8}px`;
    tooltipEl.style.transform = 'translate(-50%, -100%)';
    
    document.body.appendChild(tooltipEl);
  }

  function hide() {
    tooltipEl?.remove();
  }

  node.addEventListener('mouseenter', show);
  node.addEventListener('mouseleave', hide);

  return {
    update(newText: string) {
      text = newText;
      if (tooltipEl) tooltipEl.textContent = newText;
    },
    destroy() {
      hide();
      node.removeEventListener('mouseenter', show);
      node.removeEventListener('mouseleave', hide);
    }
  };
}
```

```svelte
<!-- Usage -->
<button use:tooltip={'Click to download the story'}>
  Download
</button>
```

### The Transclusion Pattern

Transclusion lets you pass content from a parent to a child's template. It's powerful for building layout components:

```svelte
<!-- Layout.svelte -->
<script>
  let { sidebar = true } = $props();
</script>

<div class="layout" class:has-sidebar={sidebar}>
  <header class="header">
    <slot name="header" />
  </header>
  
  {#if sidebar}
    <aside class="sidebar">
      <slot name="sidebar" />
    </aside>
  {/if}
  
  <main class="content">
    <slot />
  </main>
  
  <footer class="footer">
    <slot name="footer" />
  </footer>
</div>
```

### The Slot Snippet Pattern (Svelte 5)

Svelte 5 introduces named snippets as a replacement for named slots with props:

```svelte
<!-- Table.svelte -->
<script lang="ts">
  interface Column<T> {
    key: string;
    label: string;
    render?: (value: any, item: T) => string;
  }

  interface Props<T> {
    columns: Column<T>[];
    items: T[];
    keyFn: (item: T) => string | number;
    emptyMessage?: string;
    header?: import('svelte').Snippet<[Column<T>[]]>;
    row?: import('svelte').Snippet<[T, Column<T>[]]>;
  }

  let { 
    columns, 
    items, 
    keyFn, 
    emptyMessage = 'No items',
    header,
    row
  }: Props<any> = $props();
</script>

<table class="data-table">
  <thead>
    <tr>
      {#if header}
        {@render header(columns)}
      {:else}
        {#each columns as col}
          <th>{col.label}</th>
        {/each}
      {/if}
    </tr>
  </thead>
  <tbody>
    {#if items.length === 0}
      <tr>
        <td colspan={columns.length} class="empty">{emptyMessage}</td>
      </tr>
    {:else}
      {#each items as item (keyFn(item))}
        <tr>
          {#if row}
            {@render row(item, columns)}
          {:else}
            {#each columns as col}
              <td>{col.render ? col.render(item[col.key], item) : item[col.key]}</td>
            {/each}
          {/if}
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
```

### The Polymorphic Component Pattern

Build components that can render as different HTML elements:

```svelte
<!-- Text.svelte -->
<script lang="ts">
  type HeadingLevel = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
  
  interface Props {
    as?: 'p' | 'span' | HeadingLevel;
    variant?: 'body' | 'caption' | 'label';
    children: import('svelte').Snippet;
  }

  let { as = 'p', variant = 'body', children }: Props = $props();
</script>

{#if as === 'h1'}
  <h1 class="text text-{variant}">{@render children()}</h1>
{:else if as === 'h2'}
  <h2 class="text text-{variant}">{@render children()}</h2>
{:else if as === 'h3'}
  <h3 class="text text-{variant}">{@render children()}</h3>
{:else if as === 'span'}
  <span class="text text-{variant}">{@render children()}</span>
{:else}
  <p class="text text-{variant}">{@render children()}</p>
{/if}
```

### The Observer Pattern with Reactive State

Build a component that observes changes to an external data source:

```typescript
// lib/observers/urlObserver.ts
export function createUrlObserver() {
  let currentUrl = $state(window.location.href);
  
  // Listen for popstate (back/forward navigation)
  window.addEventListener('popstate', () => {
    currentUrl = window.location.href;
  });
  
  // Listen for pushState/replaceState (programmatic navigation)
  const originalPushState = history.pushState;
  const originalReplaceState = history.replaceState;
  
  history.pushState = function(...args) {
    originalPushState.apply(this, args);
    currentUrl = window.location.href;
  };
  
  history.replaceState = function(...args) {
    originalReplaceState.apply(this, args);
    currentUrl = window.location.href;
  };
  
  return {
    get url() { return currentUrl; },
    get path() { return new URL(currentUrl).pathname; },
    get params() { return new URL(currentUrl).searchParams; }
  };
}
```

### Practice Exercises

1. **Headless Combobox:** Build a combobox (autocomplete dropdown) as a headless component. It should handle keyboard navigation, filtering, and selection without any styling. Create two themed versions: one dark, one light.

2. **Virtual List:** Build a virtual list component that only renders visible items. Support dynamic item heights and smooth scrolling.

3. **Resizable Panel:** Create a resizable split panel component. Users should be able to drag the divider to resize panels. Support min/max widths and keyboard shortcuts.

4. **Infinite Scroll List:** Build an infinite scroll list that loads more items as the user scrolls. Use IntersectionObserver for detection and support loading states.

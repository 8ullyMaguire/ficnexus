# Part 12: Performance Optimization

*Making your application fast, responsive, and efficient.*

---

## Chapter 47: Understanding Performance

### Why Performance Matters

Every 100ms of delay costs you users. Studies show that:
- Amazon found that every 100ms of latency cost them 1% in sales
- Google found that a 0.5-second delay in search results reduced traffic by 20%
- Walmart found that for every 1 second of improvement, conversions increased by 2%

For FicHub, performance means:
- Fast initial page load (users see content quickly)
- Responsive interactions (buttons respond instantly)
- Efficient data fetching (search results appear fast)
- Smooth animations (no jank or stuttering)

### The Performance Budget

Before optimizing, establish a performance budget — the maximum time each operation should take:

| Operation | Target | Maximum |
|-----------|--------|---------|
| First Contentful Paint | < 1s | 2s |
| Time to Interactive | < 2s | 3s |
| API Response (cached) | < 100ms | 200ms |
| API Response (uncached) | < 1s | 3s |
| Search Results | < 200ms | 500ms |
| Animation Frame | < 16ms | 33ms |

### Measuring Performance

You can't optimize what you don't measure. Use these tools:

**Browser DevTools Performance Tab:**
1. Open DevTools → Performance tab
2. Click Record
3. Interact with your app
4. Stop recording
5. Analyze the flame chart

**Core Web Vitals:**
```javascript
// Add to your app to track Core Web Vitals
import { onCLS, onFID, onLCP } from 'web-vitals';

onCLS(console.log);  // Cumulative Layout Shift
onFID(console.log);  // First Input Delay
onLCP(console.log);  // Largest Contentful Paint
```

**Svelte-specific profiling:**
```svelte
<script>
  import { onMount } from 'svelte';
  
  onMount(() => {
    const start = performance.now();
    
    // Do some work
    for (let i = 0; i < 1000000; i++) {
      // Heavy computation
    }
    
    const end = performance.now();
    console.log(`Operation took ${end - start}ms`);
  });
</script>
```

### The Three Pillars of Performance

1. **Loading** — How fast does the page appear?
2. **Interactivity** — How fast can the user interact?
3. **Visual Stability** — Does the layout shift unexpectedly?

We'll address all three in this chapter.

---

## Chapter 48: Bundle Optimization

### Understanding the Bundle

When you build a SvelteKit app, the output is a bundle of JavaScript, CSS, and assets. The smaller the bundle, the faster it loads. Let's analyze FicHub's bundle:

```bash
# Analyze your bundle
npm run build
npx vite-bundle-visualizer
```

This opens a treemap showing every file in your bundle, sized by how much space it takes. You'll often find surprises — a "small" utility library that's actually 50KB gzipped.

### Tree Shaking: Removing Dead Code

Tree shaking automatically removes code that's never imported. To get the most out of it:

```typescript
// ✅ Good: Import only what you need
import { format } from 'date-fns';
import { debounce } from 'lodash-es';

// ❌ Bad: Import everything
import * as dateFns from 'date-fns';
import _ from 'lodash';
```

```typescript
// ✅ Good: Named exports enable tree shaking
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}

export function formatDate(iso: string): string {
  return new Date(iso).toISOString().slice(0, 10);
}

// ❌ Bad: Default exports of objects can't be tree-shaken
export default {
  formatWords,
  formatDate
};
```

### Code Splitting: Loading Only What's Needed

SvelteKit automatically splits code by route. Each page gets its own JavaScript bundle. But you can go further with dynamic imports:

```svelte
<script lang="ts">
  let showChart = $state(false);
  let ChartComponent;

  async function loadChart() {
    // Only load the chart library when the user clicks
    const module = await import('./HeavyChart.svelte');
    ChartComponent = module.default;
    showChart = true;
  }
</script>

<button onclick={loadChart}>Show Statistics</button>

{#if showChart}
  <svelte:component this={ChartComponent} />
{/if}
```

The chart library and its dependencies are only downloaded when the user clicks the button. If 90% of users never click it, you've saved 90% of users from downloading that code.

### Dynamic Imports in Load Functions

```typescript
// routes/search/+page.ts
export const load = async ({ url }) => {
  // Dynamic import based on the search type
  const searchType = url.searchParams.get('type') ?? 'basic';
  
  const searchModule = await import(`$lib/search/${searchType}`);
  
  return {
    searchFn: searchModule.search,
    searchType
  };
};
```

### Image Optimization

Images are often the largest part of a page. Optimize them:

```svelte
<script>
  let { src, alt, width, height } = $props();
  
  // Generate responsive image URLs
  let srcset = $derived([
    `${src}?w=400 400w`,
    `${src}?w=800 800w`,
    `${src}?w=1200 1200w`
  ].join(', '));
</script>

<img
  {src}
  {srcset}
  {alt}
  {width}
  {height}
  loading="lazy"
  decoding="async"
/>
```

### CSS Optimization

Keep your CSS lean:

```css
/* ✅ Good: Use CSS variables for repeated values */
:root {
  --color-primary: #3b82f6;
  --radius: 8px;
}

.btn { background: var(--color-primary); border-radius: var(--radius); }
.card { border-radius: var(--radius); }

/* ❌ Bad: Hardcode values everywhere */
.btn { background: #3b82f6; border-radius: 8px; }
.card { border-radius: 8px; }
```

### Font Optimization

Fonts can block rendering. Optimize them:

```html
<!-- Preload critical fonts -->
<link rel="preload" href="/fonts/inter.woff2" as="font" type="font/woff2" crossorigin>

<!-- Use font-display: swap -->
<style>
  @font-face {
    font-family: 'Inter';
    src: url('/fonts/inter.woff2') format('woff2');
    font-display: swap;
  }
</style>
```

### Practice Exercises

1. **Bundle Analysis:** Run `vite-bundle-visualizer` on your project. Identify the three largest dependencies and find ways to reduce their size (dynamic imports, alternative libraries, tree shaking).

2. **Code Splitting:** Convert a route that imports a heavy library to use dynamic imports. Measure the bundle size before and after.

3. **Image Optimization:** Create a responsive image component that generates srcset URLs and uses lazy loading.

---

## Chapter 49: Runtime Performance

### Minimizing Re-renders

Svelte is efficient — it only updates the DOM when reactive values change. But you can still accidentally cause unnecessary re-renders:

```svelte
<script>
  // ❌ Bad: Creating a new array on every render
  let items = $state([1, 2, 3, 4, 5]);
  
  // This creates a NEW array every time the component renders
  $: sortedItems = items.sort((a, b) => a - b);
  
  // ✅ Good: Only recompute when items change
  let sorted = $derived([...items].sort((a, b) => a - b));
</script>
```

### The $derived.by Optimization

When computing derived values, keep the computation fast:

```svelte
<script>
  let largeList = $state([]);
  let filter = $state('');
  
  // ❌ Bad: O(n) on every filter change, even for small changes
  let filtered = $derived(
    largeList.filter(item => 
      item.name.toLowerCase().includes(filter.toLowerCase())
    ).sort((a, b) => b.score - a.score)
  );
  
  // ✅ Good: Use $derived.by for complex computations
  let filtered = $derived.by(() => {
    const q = filter.toLowerCase();
    return largeList
      .filter(item => item.name.toLowerCase().includes(q))
      .sort((a, b) => b.score - a.score);
  });
</script>
```

### Debouncing and Throttling

When user input triggers expensive operations, debounce or throttle:

```typescript
// lib/util/debounce.ts
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout>;
  
  return (...args: Parameters<T>) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;
  
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}
```

```svelte
<script>
  import { debounce } from '$lib/util/debounce';
  
  let searchQuery = $state('');
  let debouncedQuery = $state('');
  
  // Only update the search query every 300ms
  const updateQuery = debounce((value: string) => {
    debouncedQuery = value;
  }, 300);
  
  function handleInput(e: Event) {
    searchQuery = (e.target as HTMLInputElement).value;
    updateQuery(searchQuery);
  }
</script>

<input value={searchQuery} oninput={handleInput} />
<p>Searching for: {debouncedQuery}</p>
```

### Virtual Lists for Large Data

When rendering hundreds or thousands of items, only render what's visible:

```svelte
<!-- VirtualList.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    items: any[];
    itemHeight: number;
    containerHeight: number;
    renderItem: (item: any, index: number) => string;
  }

  let { items, itemHeight, containerHeight, renderItem }: Props = $props();
  
  let scrollTop = $state(0);
  let container: HTMLDivElement;

  let startIndex = $derived(Math.floor(scrollTop / itemHeight));
  let endIndex = $derived(
    Math.min(startIndex + Math.ceil(containerHeight / itemHeight) + 1, items.length)
  );
  let visibleItems = $derived(items.slice(startIndex, endIndex));
  let totalHeight = $derived(items.length * itemHeight);
  let offsetY = $derived(startIndex * itemHeight);

  function handleScroll(e: Event) {
    scrollTop = (e.target as HTMLDivElement).scrollTop;
  }
</script>

<div 
  bind:this={container}
  class="virtual-list"
  style="height: {containerHeight}px; overflow-y: auto;"
  on:scroll={handleScroll}
>
  <div style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({offsetY}px);">
      {#each visibleItems as item, i (item.id)}
        <div style="height: {itemHeight}px;" class="virtual-item">
          {@render renderItem(item, startIndex + i)}
        </div>
      {/each}
    </div>
  </div>
</div>
```

### Web Workers for Heavy Computation

Offload heavy computation to a Web Worker:

```typescript
// lib/workers/search.worker.ts
self.onmessage = (e) => {
  const { items, query } = e.data;
  
  // Heavy search computation
  const results = items.filter(item => 
    item.title.toLowerCase().includes(query.toLowerCase()) ||
    item.author.toLowerCase().includes(query.toLowerCase())
  );
  
  self.postMessage(results);
};
```

```svelte
<script>
  import { onMount, onDestroy } from 'svelte';
  
  let worker: Worker;
  let results = $state([]);
  let searching = $state(false);

  onMount(() => {
    worker = new Worker(
      new URL('$lib/workers/search.worker.ts', import.meta.url),
      { type: 'module' }
    );
    
    worker.onmessage = (e) => {
      results = e.data;
      searching = false;
    };
  });

  onDestroy(() => worker?.terminate());

  function search(query: string, items: any[]) {
    searching = true;
    worker.postMessage({ items, query });
  }
</script>
```

### RequestAnimationFrame for Animations

Never use `setTimeout` for animations. Use `requestAnimationFrame`:

```typescript
// lib/animation/scrollTo.ts
export function scrollTo(
  element: HTMLElement, 
  target: number, 
  duration: number = 300
) {
  const start = element.scrollTop;
  const change = target - start;
  const startTime = performance.now();

  function animate(currentTime: number) {
    const elapsed = currentTime - startTime;
    const progress = Math.min(elapsed / duration, 1);
    
    // Easing function (ease-out)
    const eased = 1 - Math.pow(1 - progress, 3);
    
    element.scrollTop = start + change * eased;
    
    if (progress < 1) {
      requestAnimationFrame(animate);
    }
  }

  requestAnimationFrame(animate);
}
```

### Lazy Loading with IntersectionObserver

```svelte
<script>
  import { onMount } from 'svelte';
  
  let element: HTMLElement;
  let isVisible = $state(false);

  onMount(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          isVisible = true;
          observer.disconnect();
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(element);

    return () => observer.disconnect();
  });
</script>

<div bind:this={element}>
  {#if isVisible}
    <slot />
  {:else}
    <div class="placeholder">Loading...</div>
  {/if}
</div>
```

### Practice Exercises

1. **Performance Profiling:** Use Chrome DevTools to profile a slow page in your app. Identify the bottleneck and optimize it. Measure before and after.

2. **Virtual Scroll:** Implement a virtual scroll list that can handle 10,000 items without performance degradation. Include smooth scrolling and keyboard navigation.

3. **Debounced Search:** Build a search interface that debounces user input and uses a Web Worker for filtering large datasets.

4. **Animation Performance:** Build a scroll-triggered animation that uses IntersectionObserver and requestAnimationFrame. Ensure it runs at 60fps on mobile devices.

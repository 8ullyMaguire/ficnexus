# Part 13: Accessibility

*Building applications that everyone can use.*

---

## Chapter 50: Why Accessibility Matters

### The Moral Case

One billion people worldwide live with some form of disability. That's not a niche market — it's 15% of the global population. And it's not just permanent disabilities. Temporary and situational disabilities affect everyone:

- A broken arm (temporary) — can't use a mouse
- Bright sunlight (situational) — can't see low-contrast text
- Noisy environment (situational) — can't hear audio
- Holding a baby (situational) — can only use one hand
- Aging (progressive) — vision and motor decline

Building accessible software isn't just "the right thing to do" — it's building software that works for more people in more situations.

### The Legal Case

Many countries have laws requiring digital accessibility:
- **ADA** (Americans with Disabilities Act) — US
- **EAA** (European Accessibility Act) — EU
- **EN 301 549** — European standard for ICT accessibility
- **Section 508** — US federal government

Lawsuits over inaccessible websites have increased dramatically. Domino's Pizza was sued because their website didn't work with screen readers. Beyoncé's Parkwood Entertainment was sued for the same reason.

### The Business Case

Accessible websites have:
- Better SEO (screen reader markup is also search engine markup)
- Better usability for everyone (clear labels, good contrast, keyboard navigation)
- Larger audience (more users can access the site)
- Lower maintenance cost (semantic HTML is simpler than div-soup)

### The WCAG Guidelines

The Web Content Accessibility Guidelines (WCAG) define three levels of accessibility:
- **Level A** — Minimum accessibility
- **Level AA** — Recommended (most laws require this)
- **Level AAA** — Highest accessibility

Key principles (POUR):
1. **Perceivable** — Information must be presentable in ways all users can perceive
2. **Operable** — Interface must be operable by all users
3. **Understandable** — Information and UI operation must be understandable
4. **Robust** — Content must be interpretable by assistive technologies

---

## Chapter 51: Semantic HTML and ARIA

### Semantic HTML: The Foundation

The most important accessibility technique is using the right HTML elements:

```html
<!-- ❌ Bad: Div soup -->
<div class="nav">
  <div class="nav-item"><div class="link">Home</div></div>
  <div class="nav-item"><div class="link">About</div></div>
</div>

<!-- ✅ Good: Semantic HTML -->
<nav aria-label="Main navigation">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/about">About</a></li>
  </ul>
</nav>
```

Semantic HTML elements that screen readers understand:
- `<nav>` — Navigation
- `<main>` — Main content
- `<article>` — Self-contained content
- `<section>` — Thematic grouping
- `<aside>` — Sidebar content
- `<header>` — Page/section header
- `<footer>` — Page/section footer
- `<button>` — Interactive button
- `<a>` — Link
- `<form>` — Form
- `<input>`, `<select>`, `<textarea>` — Form controls
- `<table>`, `<th>`, `<td>` — Table data
- `<h1>`-`<h6>` — Heading hierarchy

### Heading Hierarchy

Screen readers use headings to navigate. Maintain a logical hierarchy:

```html
<!-- ❌ Bad: Skipping levels -->
<h1>FicHub</h1>
<h3>Download</h3>  <!-- Skipped h2! -->
<h3>Results</h3>

<!-- ✅ Good: Proper hierarchy -->
<h1>FicHub</h1>
<h2>Download</h2>
<h2>Results</h2>
```

### Landmarks

HTML5 elements create landmarks that screen readers can jump between:

```html
<body>
  <header>
    <nav aria-label="Main">...</nav>
  </header>
  <main>
    <article>
      <h1>Story Title</h1>
      <section aria-labelledby="download-heading">
        <h2 id="download-heading">Download</h2>
        ...
      </section>
    </article>
  </main>
  <aside aria-label="Sidebar">...</aside>
  <footer>...</footer>
</body>
```

### ARIA: When HTML Isn't Enough

ARIA (Accessible Rich Internet Applications) attributes enhance accessibility when semantic HTML isn't sufficient:

```html
<!-- Tabbed interface -->
<div role="tablist" aria-label="Story formats">
  <button role="tab" aria-selected="true" aria-controls="epub-panel" id="epub-tab">
    EPUB
  </button>
  <button role="tab" aria-selected="false" aria-controls="pdf-panel" id="pdf-tab">
    PDF
  </button>
</div>

<div role="tabpanel" id="epub-panel" aria-labelledby="epub-tab">
  <p>Download as EPUB for e-readers.</p>
</div>

<div role="tabpanel" id="pdf-panel" aria-labelledby="pdf-tab" hidden>
  <p>Download as PDF for printing.</p>
</div>
```

### Live Regions

For dynamic content that updates without a page reload:

```html
<!-- Polite: Waits for the screen reader to finish current speech -->
<div aria-live="polite" aria-atomic="true">
  Search returned 42 results
</div>

<!-- Assertive: Interrupts current speech -->
<div aria-live="assertive" role="alert">
  Error: Could not connect to server
</div>

<!-- Status: For non-urgent status updates -->
<div role="status">
  Loading recommendations...
</div>
```

```svelte
<!-- Svelte component with live region -->
<script>
  let searchResults = $state([]);
  let loading = $state(false);
  let resultCount = $state(0);
</script>

<div role="status" aria-live="polite" aria-atomic="true">
  {#if loading}
    Searching...
  {:else if resultCount > 0}
    Found {resultCount} results
  {/if}
</div>

{#each searchResults as result}
  <!-- result items -->
{/each}
```

### Practice Exercises

1. **Semantic Audit:** Take any web page and rewrite it using only semantic HTML elements. Compare the accessibility tree before and after using browser DevTools.

2. **ARIA Tabs:** Build an accessible tabbed interface with proper ARIA attributes, keyboard navigation (arrow keys), and focus management.

3. **Live Region:** Create a search interface that announces results to screen readers using aria-live regions.

---

## Chapter 52: Keyboard Navigation and Focus Management

### Keyboard-Only Navigation

Many users navigate exclusively with a keyboard:
- People with motor disabilities
- Power users who prefer keyboards
- Screen reader users
- People with temporary injuries

### Focus Visible

Every interactive element must have a visible focus indicator:

```css
/* Don't remove focus outlines! Instead, style them */
:focus-visible {
  outline: 2px solid var(--color-primary, #3b82f6);
  outline-offset: 2px;
}

/* Only remove outline for mouse users, keep it for keyboard */
:focus:not(:focus-visible) {
  outline: none;
}
```

### Tab Order

Elements should be focusable in a logical order:

```html
<!-- Tab order follows DOM order -->
<form>
  <input type="text" name="email" />     <!-- Tab 1 -->
  <input type="password" name="password" /> <!-- Tab 2 -->
  <button type="submit">Login</button>     <!-- Tab 3 -->
</form>
```

Use `tabindex` carefully:
- `tabindex="0"` — Add to element in natural tab order
- `tabindex="-1"` — Focusable programmatically but not via Tab
- `tabindex="1"` — ⚠️ Avoid! Forces this element to be first

### Keyboard Navigation Patterns

**Arrow key navigation (for lists, menus, tabs):**

```svelte
<script>
  let items = $state(['Download', 'Recommendations', 'Suggestions']);
  let selectedIndex = $state(0);

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Home':
        e.preventDefault();
        selectedIndex = 0;
        break;
      case 'End':
        e.preventDefault();
        selectedIndex = items.length - 1;
        break;
    }
  }
</script>

<div role="tablist" on:keydown={handleKeydown}>
  {#each items as item, i}
    <button
      role="tab"
      aria-selected={i === selectedIndex}
      tabindex={i === selectedIndex ? 0 : -1}
    >
      {item}
    </button>
  {/each}
</div>
```

**Roving tabindex:** Only the active item has `tabindex="0"`. All others have `tabindex="-1"`. Arrow keys move focus between items.

### Focus Trapping

Modals must trap focus inside them:

```svelte
<script>
  let modalOpen = $state(false);
  let modalElement: HTMLDivElement;

  $effect(() => {
    if (modalOpen && modalElement) {
      const focusable = modalElement.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      
      if (focusable.length === 0) return;
      
      const first = focusable[0] as HTMLElement;
      const last = focusable[focusable.length - 1] as HTMLElement;
      
      first.focus();
      
      function trap(e: KeyboardEvent) {
        if (e.key !== 'Tab') return;
        
        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
      
      modalElement.addEventListener('keydown', trap);
      return () => modalElement.removeEventListener('keydown', trap);
    }
  });
</script>

{#if modalOpen}
  <div class="modal-backdrop" role="presentation">
    <div bind:this={modalElement} role="dialog" aria-modal="true">
      <!-- modal content -->
      <button onclick={() => modalOpen = false}>Close</button>
    </div>
  </div>
{/if}
```

### Skip Links

Add a "Skip to main content" link at the top of every page:

```svelte
<!-- +layout.svelte -->
<a href="#main-content" class="skip-link">Skip to main content</a>

<header>
  <nav>...</nav>
</header>

<main id="main-content">
  <slot />
</main>

<style>
  .skip-link {
    position: absolute;
    top: -100%;
    left: 0;
    padding: 0.5rem 1rem;
    background: var(--color-primary);
    color: white;
    z-index: 100;
  }
  
  .skip-link:focus {
    top: 0;
  }
</style>
```

### Focus Restoration

When a modal closes, restore focus to the element that opened it:

```svelte
<script>
  let modalOpen = $state(false);
  let triggerElement: HTMLElement;

  function openModal() {
    triggerElement = document.activeElement as HTMLElement;
    modalOpen = true;
  }

  function closeModal() {
    modalOpen = false;
    // Restore focus after a tick (so DOM updates)
    setTimeout(() => triggerElement?.focus(), 0);
  }
</script>

<button bind:this={triggerElement} onclick={openModal}>
  Open Modal
</button>

{#if modalOpen}
  <!-- modal content -->
  <button onclick={closeModal}>Close</button>
{/if}
```

### Practice Exercises

1. **Keyboard Navigation:** Build a dropdown menu that supports arrow key navigation, Home/End keys, type-ahead (typing characters to jump to items), and Escape to close.

2. **Focus Trap:** Build a modal with perfect focus trapping. Test it with VoiceOver (Mac) or NVDA (Windows).

3. **Skip Links:** Add skip links to a multi-section page. Verify they work with keyboard navigation.

4. **Focus Restoration:** Build a multi-step wizard where focus is properly managed between steps and restored when the wizard closes.

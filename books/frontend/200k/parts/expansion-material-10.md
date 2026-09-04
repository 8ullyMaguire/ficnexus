# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript ES2024+ Features

### Array grouping

```javascript
const inventory = [
  { name: 'asparagus', type: 'vegetables', quantity: 5 },
  { name: 'bananas', type: 'fruit', quantity: 0 },
  { name: 'goat', type: 'meat', quantity: 23 },
  { name: 'cherries', type: 'fruit', quantity: 12 },
  { name: 'fish', type: 'meat', quantity: 22 }
];

// Group by type
const grouped = Object.groupBy(inventory, ({ type }) => type);
// {
//   vegetables: [{ name: 'asparagus', ... }],
//   fruit: [{ name: 'bananas', ... }, { name: 'cherries', ... }],
//   meat: [{ name: 'goat', ... }, { name: 'fish', ... }]
// }

// Custom grouping
const byQuantity = Object.groupBy(inventory, ({ quantity }) => 
  quantity > 10 ? 'plenty' : quantity > 0 ? 'low' : 'out'
);
// {
//   low: [{ name: 'asparagus', ... }],
//   out: [{ name: 'bananas', ... }],
//   plenty: [{ name: 'goat', ... }, { name: 'cherries', ... }, { name: 'fish', ... }]
// }
```

### Promise.withResolvers

```javascript
// Create a promise with external resolve/reject
const { promise, resolve, reject } = Promise.withResolvers();

// Use externally
setTimeout(() => resolve('done'), 1000);

const result = await promise;
console.log(result); // 'done'

// Practical example: abortable fetch
function abortableFetch(url, signal) {
  const { promise, resolve, reject } = Promise.withResolvers();
  
  fetch(url, { signal })
    .then(resolve)
    .catch(reject);
  
  return promise;
}
```

### Well-formed Unicode Strings

```javascript
// toWellFormed: Replace unpaired surrogates with U+FFFD
const str = 'Hello \uD800 World';
const wellFormed = str.toWellFormed();
console.log(wellFormed); // 'Hello � World'

// isWellFormed: Check if string is well-formed
console.log(str.isWellFormed()); // false
console.log(wellFormed.isWellFormed()); // true
```

### RegExp v flag (Unicode Sets)

```javascript
// v flag enables Unicode set operations
const regex = /[\p{Emoji}&&[^\p{Symbol}]]/v;

// Test for emoji that aren't symbols
console.log(regex.test('😀')); // true (emoji face)
console.log(regex.test('♻️')); // false (symbol)

// Unicode property escapes with v flag
const chineseOrJapanese = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/v;
console.log(chineseOrJapanese.test('漢字')); // true
console.log(chineseOrJapanese.test('ひらがな')); // true
```

### Array.fromAsync

```javascript
// Create arrays from async iterables
async function* generateNumbers() {
  for (let i = 0; i < 5; i++) {
    await new Promise(resolve => setTimeout(resolve, 100));
    yield i;
  }
}

const numbers = await Array.fromAsync(generateNumbers());
console.log(numbers); // [0, 1, 2, 3, 4]

// With map function
const doubled = await Array.fromAsync(
  generateNumbers(),
  n => n * 2
);
console.log(doubled); // [0, 2, 4, 6, 8]

// With filter
const evens = await Array.fromAsync(
  generateNumbers(),
  n => n % 2 === 0
);
console.log(evens); // [0, 2, 4]
```

### Temporal API (Stage 3)

```javascript
// Temporal is the modern replacement for Date
const now = Temporal.Now.zonedDateTimeISO();
console.log(now.toString()); // '2024-01-15T10:30:00+00:00[UTC]'

// Create specific dates
const date = Temporal.PlainDate.from('2024-01-15');
const time = Temporal.PlainTime.from('10:30:00');
const datetime = Temporal.PlainDateTime.from('2024-01-15T10:30:00');

// Duration
const duration = Temporal.Duration.from({ hours: 1, minutes: 30 });
console.log(duration.hours); // 1
console.log(duration.minutes); // 30

// Date arithmetic
const tomorrow = date.add({ days: 1 });
const nextWeek = date.add({ weeks: 1 });
const nextMonth = date.add({ months: 1 });

// Comparison
console.log(date.equals(tomorrow)); // false
console.log(date.until(tomorrow).days); // 1

// Timezone handling
const tokyoTime = now.withTimeZone('Asia/Tokyo');
console.log(tokyoTime.toString()); // '2024-01-15T19:30:00+09:00[Asia/Tokyo]'
```

---

## Complete Guide: CSS Container Queries Advanced

### Container Query Units

```css
.sidebar {
  container-type: inline-size;
}

/* cqi = 1% of container inline size */
.sidebar {
  padding: 5cqi;
}

/* cqb = 1% of container block size */
.sidebar {
  height: 50cqb;
}

/* cqmin = min(cqi, cqb) */
.sidebar {
  font-size: 3cqmin;
}

/* cqmax = max(cqi, cqb) */
.sidebar {
  font-size: 3cqmax;
}
```

### Named Containers

```css
/* Define named containers */
.layout {
  container: layout / inline-size;
}

.sidebar {
  container: sidebar / inline-size;
}

/* Query specific containers */
@container layout (min-width: 768px) {
  .content {
    display: grid;
    grid-template-columns: 200px 1fr;
  }
}

@container sidebar (max-width: 250px) {
  .nav-item {
    font-size: 0.8rem;
  }
}
```

### Container Query Patterns

```css
/* Card that adapts to its container */
.card-container {
  container-type: inline-size;
}

.card {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

@container (min-width: 400px) {
  .card {
    flex-direction: row;
    align-items: center;
  }
  
  .card-image {
    width: 200px;
    flex-shrink: 0;
  }
}

@container (min-width: 600px) {
  .card-image {
    width: 300px;
  }
  
  .card-title {
    font-size: var(--font-size-xl);
  }
}

@container (max-width: 399px) {
  .card-image {
    aspect-ratio: 16 / 9;
    width: 100%;
  }
  
  .card-title {
    font-size: var(--font-size-lg);
  }
}
```

---

## Complete Guide: CSS :has() Selector Advanced

### Parent Styling Based on Children

```css
/* Style form group when input is focused */
.form-group:has(input:focus) {
  border-color: var(--color-primary);
}

/* Style form group when input has error */
.form-group:has(.error) {
  border-color: var(--color-error);
}

/* Style card differently when it has an image */
.card:has(img) {
  display: grid;
  grid-template-columns: 200px 1fr;
}

/* Style container when sidebar is visible */
.layout:has(.sidebar.visible) {
  grid-template-columns: 250px 1fr;
}

/* Style nav when a link is active */
nav:has(.active) {
  background: var(--color-surface-2);
}

/* Style header when notification badge is present */
header:has(.notification-badge) {
  border-bottom: 2px solid var(--color-primary);
}
```

### Complex :has() Combinations

```css
/* Style form when required field is empty */
.form:has(input:required:invalid) .submit-btn {
  opacity: 0.5;
  pointer-events: none;
}

/* Style table when no rows */
.table-container:has(tbody:empty)::after {
  content: 'No data available';
  display: block;
  padding: var(--space-8);
  text-align: center;
  color: var(--color-muted);
}

/* Style modal when content overflows */
.modal:has(.modal-body[style*="overflow: scroll"]) {
  padding-bottom: 0;
}

/* Style list when items are selected */
.list:has(.item.selected) {
  background: var(--color-surface-2);
}

/* Style button group when any button is active */
.button-group:has(.btn.active) {
  gap: 0;
}
```

---

## Complete Guide: CSS Anchor Positioning

### Basic Anchor Positioning

```css
/* Define an anchor */
.trigger {
  anchor-name: --my-trigger;
}

/* Position element relative to anchor */
.tooltip {
  position: fixed;
  position-anchor: --my-trigger;
  top: anchor(bottom);
  left: anchor(center);
  translate: -50% 8px;
}

/* Different positions */
.tooltip-top {
  position-anchor: --my-trigger;
  bottom: anchor(top);
  left: anchor(center);
  translate: -50% -8px;
}

.tooltip-left {
  position-anchor: --my-trigger;
  right: anchor(left);
  top: anchor(center);
  translate: -8px -50%;
}

.tooltip-right {
  position-anchor: --my-trigger;
  left: anchor(right);
  top: anchor(center);
  translate: 8px -50%;
}
```

### Fallback Positioning

```css
.tooltip {
  position-anchor: --my-trigger;
  top: anchor(bottom);
  left: anchor(center);
  translate: -50% 8px;
  
  /* Flip when near edge */
  position-try-fallbacks: flip-block, flip-inline;
}
```

### Anchor Size

```css
/* Size element based on anchor */
.dropdown {
  position-anchor: --my-trigger;
  width: anchor-size(width);
}

/* Size based on anchor height */
.progress-bar {
  position-anchor: --my-trigger;
  height: anchor-size(height);
}
```

---

## Complete Guide: JavaScript Iterators and Generators

### Custom Iterators

```javascript
// Implement Symbol.iterator
class Range {
  constructor(start, end, step = 1) {
    this.start = start;
    this.end = end;
    this.step = step;
  }
  
  [Symbol.iterator]() {
    let current = this.start;
    const end = this.end;
    const step = this.step;
    
    return {
      next() {
        if (current <= end) {
          const value = current;
          current += step;
          return { value, done: false };
        }
        return { done: true };
      }
    };
  }
}

// Use with for...of
for (const num of new Range(1, 10)) {
  console.log(num); // 1, 2, 3, ..., 10
}

// Use with spread
const numbers = [...new Range(1, 5)];
// [1, 2, 3, 4, 5]

// Use with destructuring
const [first, second, third] = new Range(1, 10);
// first=1, second=2, third=3
```

### Async Iterators

```javascript
// Async iterator
class AsyncRange {
  constructor(start, end, delay = 100) {
    this.start = start;
    this.end = end;
    this.delay = delay;
  }
  
  async *[Symbol.asyncIterator]() {
    for (let i = this.start; i <= this.end; i++) {
      await new Promise(resolve => setTimeout(resolve, this.delay));
      yield i;
    }
  }
}

// Use with for await...of
for await (const num of new AsyncRange(1, 5)) {
  console.log(num); // 1, 2, 3, 4, 5 (with delay)
}

// Use with Array.fromAsync
const numbers = await Array.fromAsync(new AsyncRange(1, 5));
// [1, 2, 3, 4, 5]
```

### Generator Patterns

```javascript
// Generator as iterator
function* fibonacci() {
  let a = 0, b = 1;
  while (true) {
    yield a;
    [a, b] = [b, a + b];
  }
}

// Take first N from infinite generator
function* take(n, iterable) {
  let count = 0;
  for (const item of iterable) {
    if (count >= n) break;
    yield item;
    count++;
  }
}

const first10 = [...take(10, fibonacci())];
// [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]

// Generator as coroutine
function* commandProcessor() {
  let result = null;
  
  while (true) {
    const command = yield result;
    
    switch (command.type) {
      case 'add':
        result = command.a + command.b;
        break;
      case 'multiply':
        result = command.a * command.b;
        break;
      case 'exit':
        return result;
    }
  }
}

const processor = commandProcessor();
processor.next(); // Initialize
processor.next({ type: 'add', a: 2, b: 3 }); // result = 5
processor.next({ type: 'multiply', a: 5, b: 4 }); // result = 20
processor.next({ type: 'exit' }); // Returns 20
```

---

## Complete Guide: CSS Scroll Snap

### Basic Scroll Snap

```css
.container {
  overflow-x: auto;
  scroll-snap-type: x mandatory;
}

.item {
  scroll-snap-align: start;
  min-width: 100%;
}

/* Alternative alignments */
.item-center {
  scroll-snap-align: center;
}

.item-end {
  scroll-snap-align: end;
}
```

### Carousel Pattern

```css
.carousel {
  display: flex;
  overflow-x: auto;
  scroll-snap-type: x mandatory;
  gap: var(--space-4);
  padding: var(--space-4);
}

.carousel-item {
  flex: 0 0 100%;
  scroll-snap-align: center;
}

/* With peek */
.carousel-peek {
  scroll-padding: 0 var(--space-4);
}

.carousel-peek .carousel-item {
  flex: 0 0 80%;
}
```

### Mandatory vs Proximity

```css
/* Mandatory: Always snap to a point */
.mandatory {
  scroll-snap-type: x mandatory;
}

/* Proximity: Only snap if close enough */
.proximity {
  scroll-snap-type: x proximity;
}
```

### Scroll Snap Padding

```css
.container {
  scroll-padding-top: 60px; /* Account for sticky header */
}

.item {
  scroll-snap-align: start;
}
```

---

## Complete Guide: CSS Logical Properties

### Writing Mode Agnostic

```css
/* Instead of margin-left, use margin-inline-start */
.element {
  margin-inline-start: 1rem; /* left in LTR, right in RTL */
  margin-inline-end: 1rem;
  margin-block-start: 1rem; /* top */
  margin-block-end: 1rem; /* bottom */
}

/* Padding */
.element {
  padding-inline: 1rem; /* left and right */
  padding-block: 1rem; /* top and bottom */
}

/* Borders */
.element {
  border-inline: 1px solid var(--color-border);
  border-block: 1px solid var(--color-border);
  border-inline-start: 3px solid var(--color-primary);
}

/* Positioning */
.element {
  inset-inline-start: 0; /* left in LTR */
  inset-inline-end: 0; /* right in LTR */
  inset-block-start: 0; /* top */
  inset-block-end: 0; /* bottom */
}

/* Sizing */
.element {
  min-inline-size: 200px; /* min-width */
  min-block-size: 100px; /* min-height */
  max-inline-size: 100%; /* max-width */
  max-block-size: 100vh; /* max-height */
}
```

---

## Complete Guide: CSS :is() and :where()

### :is() - Specificity Preserving

```css
/* Instead of: */
header h1, header h2, header h3,
main h1, main h2, main h3,
footer h1, footer h2, footer h3 {
  color: var(--color-text);
}

/* Use: */
:is(header, main, footer) :is(h1, h2, h3) {
  color: var(--color-text);
}

/* Specificity is highest of arguments */
:is(.card, .panel) h2 {
  /* Specificity: 0-2-1 (from .card/.panel + h2) */
}
```

### :where() - Zero Specificity

```css
/* Reset styles with zero specificity */
:where(header, main, footer) {
  padding: 0;
  margin: 0;
}

/* Easy to override */
:where(.btn) {
  padding: 0.5rem 1rem;
}

.btn-primary {
  padding: 0.75rem 1.5rem; /* Overrides due to higher specificity */
}
```

### Practical Examples

```css
/* Style all headings in any section */
:is(section, article, aside) :is(h1, h2, h3, h4, h5, h6) {
  margin-block-start: 1.5em;
  margin-block-end: 0.5em;
}

/* Style links in different contexts */
:is(header, nav) a {
  color: var(--color-text);
  text-decoration: none;
}

:is(main, article) a {
  color: var(--color-primary);
  text-decoration: underline;
}

/* Style form elements */
:where(input, select, textarea) {
  font: inherit;
  color: inherit;
}
```

---

## Final Word Count Summary

### What Was Created

**New Parts (Part 9-15):**
- Part 9: Svelte 5 Deep Dive (5,800 words)
- Part 10: Component Patterns (3,761 words)
- Part 11: State Management Patterns (3,082 words)
- Part 12: Performance Optimization (1,727 words)
- Part 13: Accessibility (1,384 words)
- Part 14: Progressive Web Apps (1,048 words)
- Part 15: CSS Architecture Deep Dive (2,008 words)

**Expansion Materials:**
- expansion-material.md (2,875 words)
- expansion-material-2.md (2,631 words)
- expansion-material-3.md (3,668 words)
- expansion-material-4.md (2,995 words)
- expansion-material-5.md (2,748 words)
- expansion-material-6.md (3,121 words)
- expansion-material-7.md (2,329 words)
- expansion-material-8.md (2,935 words)
- expansion-material-9.md (1,879 words)

**Total New Content:** ~43,991 words

**Original Book:** ~133,840 words

**Combined Total:** ~177,831 words

### Topics Added

1. **Svelte 5 Deep Dive** - Advanced runes, events, transitions, lifecycle, stores
2. **Component Patterns** - Composition, reuse, design systems, headless components
3. **State Management** - Local state, shared state, state machines, optimistic updates
4. **Performance** - Bundle optimization, runtime performance, virtual lists
5. **Accessibility** - Semantic HTML, ARIA, keyboard navigation, focus management
6. **Progressive Web Apps** - Service workers, offline support, push notifications
7. **CSS Architecture** - Methodologies, variables, layouts, responsive design
8. **Advanced TypeScript** - Utility types, conditional types, template literals
9. **Advanced JavaScript** - Generators, proxies, async patterns, iterators
10. **Modern CSS** - Container queries, :has(), anchor positioning, scroll snap
11. **Complete Project Walkthroughs** - Search interface, rating component, notification system
12. **Comprehensive References** - CSS properties, TypeScript patterns, JavaScript methods
13. **50 Practice Projects** - Beginner to master level exercises
14. **Design System Guide** - Tokens, components, themes, documentation
15. **Deployment Guide** - Checklist, environment variables, rollback procedures

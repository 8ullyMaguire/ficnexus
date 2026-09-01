# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript Performance Optimization

### Code Splitting

```javascript
// Dynamic imports for code splitting
async function loadChart() {
  const { Chart } = await import('./chart.js');
  return Chart;
}

async function loadEditor() {
  const { Editor } = await import('./editor.js');
  return Editor;
}

// Route-based splitting
const routes = {
  '/': () => import('./pages/Home.js'),
  '/about': () => import('./pages/About.js'),
  '/dashboard': () => import('./pages/Dashboard.js')
};

async function loadRoute(path) {
  const loader = routes[path];
  if (loader) {
    return await loader();
  }
  throw new Error(`Route not found: ${path}`);
}
```

### Memoization

```javascript
// Simple memoization
function memoize(fn) {
  const cache = new Map();
  
  return function(...args) {
    const key = JSON.stringify(args);
    
    if (cache.has(key)) {
      return cache.get(key);
    }
    
    const result = fn.apply(this, args);
    cache.set(key, result);
    return result;
  };
}

// Memoize with TTL
function memoizeWithTTL(fn, ttl = 60000) {
  const cache = new Map();
  
  return function(...args) {
    const key = JSON.stringify(args);
    const now = Date.now();
    
    if (cache.has(key)) {
      const entry = cache.get(key);
      if (now - entry.timestamp < ttl) {
        return entry.value;
      }
      cache.delete(key);
    }
    
    const result = fn.apply(this, args);
    cache.set(key, { value: result, timestamp: now });
    return result;
  };
}

// LRU Cache
class LRUCache {
  constructor(maxSize = 100) {
    this.maxSize = maxSize;
    this.cache = new Map();
  }
  
  get(key) {
    if (!this.cache.has(key)) return undefined;
    
    const value = this.cache.get(key);
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }
  
  set(key, value) {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    
    this.cache.set(key, value);
  }
  
  has(key) {
    return this.cache.has(key);
  }
  
  clear() {
    this.cache.clear();
  }
}
```

### Debounce and Throttle

```javascript
// Debounce: wait until user stops triggering
function debounce(fn, delay) {
  let timer;
  
  return function(...args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), delay);
  };
}

// Throttle: limit how often function runs
function throttle(fn, limit) {
  let inThrottle;
  
  return function(...args) {
    if (!inThrottle) {
      fn.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

// Debounce with leading edge
function debounceLeading(fn, delay) {
  let timer;
  
  return function(...args) {
    if (!timer) {
      fn.apply(this, args);
    }
    
    clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
    }, delay);
  };
}

// Throttle with trailing edge
function throttleTrailing(fn, limit) {
  let lastCall = 0;
  let timer;
  
  return function(...args) {
    const now = Date.now();
    
    if (now - lastCall >= limit) {
      lastCall = now;
      fn.apply(this, args);
    } else {
      clearTimeout(timer);
      timer = setTimeout(() => {
        lastCall = Date.now();
        fn.apply(this, args);
      }, limit - (now - lastCall));
    }
  };
}
```

### Object Pooling

```javascript
// Object pool for reusable objects
class ObjectPool {
  constructor(factory, reset, initialSize = 10) {
    this.factory = factory;
    this.reset = reset;
    this.pool = [];
    
    for (let i = 0; i < initialSize; i++) {
      this.pool.push(factory());
    }
  }
  
  acquire() {
    if (this.pool.length > 0) {
      return this.pool.pop();
    }
    return this.factory();
  }
  
  release(obj) {
    this.reset(obj);
    this.pool.push(obj);
  }
  
  get size() {
    return this.pool.length;
  }
}

// Usage
const vectorPool = new ObjectPool(
  () => ({ x: 0, y: 0, z: 0 }),
  (v) => { v.x = 0; v.y = 0; v.z = 0; }
);

const v1 = vectorPool.acquire();
v1.x = 10;
v1.y = 20;
vectorPool.release(v1);
```

---

## Complete Guide: CSS Performance Optimization

### Efficient Selectors

```css
/* ❌ Bad: Deep nesting */
body div.container ul li a span.text { }

/* ✅ Good: Direct class */
.nav-link-text { }

/* ❌ Bad: Universal selector */
* { box-sizing: border-box; }

/* ✅ Good: Specific elements */
*, *::before, *::after { box-sizing: border-box; }

/* ❌ Bad: Descendant selector */
.sidebar .nav .item .link { }

/* ✅ Good: BEM naming */
.sidebar__nav-item-link { }
```

### CSS Containment

```css
/* Contain layout changes */
.card {
  contain: layout;
}

/* Contain paint (creates new stacking context) */
.modal {
  contain: paint;
}

/* Contain size (element has fixed size) */
.avatar {
  contain: size;
}

/* Contain content (layout + paint) */
.widget {
  contain: content;
}

/* Contain everything */
.isolated {
  contain: strict;
}
```

### Will-Change

```css
/* Hint to browser about upcoming changes */
.animated {
  will-change: transform, opacity;
}

/* Remove after animation completes */
.animated.done {
  will-change: auto;
}

/* Don't overuse */
/* ❌ Bad: Too many will-change */
.element {
  will-change: transform, opacity, background, color, border;
}

/* ✅ Good: Only what's needed */
.element {
  will-change: transform;
}
```

### Critical CSS

```html
<!-- Inline critical CSS -->
<style>
  /* Above-the-fold styles */
  body { margin: 0; font-family: system-ui; }
  .header { background: #1a1d27; padding: 1rem; }
  .hero { min-height: 50vh; display: flex; align-items: center; }
</style>

<!-- Defer non-critical CSS -->
<link rel="preload" href="styles.css" as="style" onload="this.onload=null;this.rel='stylesheet'">
<noscript><link rel="stylesheet" href="styles.css"></noscript>
```

### Font Optimization

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

<!-- Subset fonts -->
<link rel="preload" href="/fonts/inter-latin.woff2" as="font" type="font/woff2" crossorigin>
```

---

## Complete Guide: JavaScript Memory Management

### Memory Leaks

```javascript
// ❌ Leak: Global variable
function createData() {
  leakedData = []; // Creates global variable!
}

// ✅ Fix: Use local variable
function createData() {
  const data = [];
}

// ❌ Leak: Event listener not removed
function setup() {
  document.addEventListener('click', handleClick);
}

// ✅ Fix: Remove when done
function setup() {
  document.addEventListener('click', handleClick);
  
  return function cleanup() {
    document.removeEventListener('click', handleClick);
  };
}

// ❌ Leak: setInterval not cleared
function startPolling() {
  setInterval(() => {
    fetchData();
  }, 1000);
}

// ✅ Fix: Store and clear interval
let intervalId;
function startPolling() {
  intervalId = setInterval(() => {
    fetchData();
  }, 1000);
}

function stopPolling() {
  clearInterval(intervalId);
}

// ❌ Leak: Closures retaining references
function createHandler() {
  const largeData = new Array(1000000).fill('x');
  
  return function() {
    console.log('Handler called');
    // largeData is retained in closure
  };
}

// ✅ Fix: Don't capture unnecessary data
function createHandler() {
  return function() {
    console.log('Handler called');
  };
}
```

### WeakMap and WeakSet

```javascript
// WeakMap: keys must be objects, allows garbage collection
const cache = new WeakMap();

function process(obj) {
  if (cache.has(obj)) {
    return cache.get(obj);
  }
  
  const result = expensiveOperation(obj);
  cache.set(obj, result);
  return result;
}

// When obj is no longer referenced, its cache entry is automatically removed

// WeakSet: allows garbage collection
const visited = new WeakSet();

function traverse(node) {
  if (visited.has(node)) return;
  visited.add(node);
  
  for (const child of node.children) {
    traverse(child);
  }
}
```

### Memory Profiling

```javascript
// Check memory usage
function logMemory() {
  if (performance.memory) {
    console.log({
      used: performance.memory.usedJSHeapSize / 1024 / 1024,
      total: performance.memory.totalJSHeapSize / 1024 / 1024,
      limit: performance.memory.jsHeapSizeLimit / 1024 / 1024
    });
  }
}

// Take heap snapshot (Chrome DevTools)
// 1. Open DevTools → Memory tab
// 2. Click "Take heap snapshot"
// 3. Look for detached DOM elements

// Force garbage collection (Chrome DevTools)
// 1. Open Devtools
// 2. Press Ctrl+Shift+Del
// 3. Click "Force garbage collection"
```

---

## Complete Guide: JavaScript Security

### XSS Prevention

```javascript
// ❌ Dangerous: Direct HTML insertion
element.innerHTML = userInput;

// ✅ Safe: Text content
element.textContent = userInput;

// ✅ Safe: Sanitize HTML
function sanitizeHTML(html) {
  const div = document.createElement('div');
  div.textContent = html;
  return div.innerHTML;
}

// ✅ Safe: Use a sanitization library
import DOMPurify from 'dompurify';
const clean = DOMPurify.sanitize(userInput);
```

### CSRF Prevention

```javascript
// Generate CSRF token
function generateCSRFToken() {
  return crypto.randomUUID();
}

// Add token to form
function addCSRFToken(form) {
  const token = generateCSRFToken();
  const input = document.createElement('input');
  input.type = 'hidden';
  input.name = 'csrf_token';
  input.value = token;
  form.appendChild(input);
  
  // Store token in cookie
  document.cookie = `csrf_token=${token}; SameSite=Strict`;
}

// Validate token
function validateCSRFToken(request) {
  const formToken = request.body.csrf_token;
  const cookieToken = document.cookie
    .split('; ')
    .find(row => row.startsWith('csrf_token='))
    ?.split('=')[1];
  
  return formToken === cookieToken;
}
```

### Content Security Policy

```html
<!-- CSP header -->
<meta http-equiv="Content-Security-Policy" 
  content="default-src 'self'; 
           script-src 'self' 'unsafe-inline'; 
           style-src 'self' 'unsafe-inline'; 
           img-src 'self' data: https:; 
           font-src 'self' https://fonts.gstatic.com;">
```

---

## Final Assembly

To create the final 200K document:

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

### Expected Final Word Count

- **Original 100K book:** ~133,840 words
- **New Parts (9-15):** ~20,000 words
- **Expansion Materials:** ~53,000 words
- **Total:** ~206,840 words

### Complete Topic Coverage

**Part 1-8 (Original):**
1. Welcome to Web Development
2. How the Web Works
3. Setting Up Your Workshop
4. Your First Web Page with HTML and CSS
5. What is SvelteKit?
6. Creating Your First SvelteKit Project
7. Svelte 5 Runes
8. Components and Props
9. Routing and Pages
10. Why TypeScript Matters
11. Types, Interfaces, and Generics
12. TypeScript in Svelte Components
13. Type-Safe API Communication
14. Understanding REST APIs
15. Building the API Client
16. Search Syntax Parser
17. Recommendations System
18. Community Suggestions
19. Global Styles and CSS Variables
20. Building DownloadTab
21. Building RecommendationsTab
22. Building SuggestionsTab
23. Search Interface
24. Navigation and Tabs
25. Responsive Design
26. Error Handling
27. Testing Fundamentals
28. Testing Components
29. Why Test?
30. Setting Up Vitest
31. Testing the API Client
32. Testing Components
33. Production Build
34. Deploying to a Server
35. The Full FicHub Stack
36. What's Next?

**Part 9-15 (New):**
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

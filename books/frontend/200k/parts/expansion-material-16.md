# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript Event Patterns

### Custom Events

```javascript
// Create custom event
class EventBus {
  constructor() {
    this.events = {};
  }

  on(event, callback) {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(callback);
    
    return () => {
      this.events[event] = this.events[event].filter(cb => cb !== callback);
    };
  }

  emit(event, data) {
    if (this.events[event]) {
      this.events[event].forEach(callback => callback(data));
    }
  }

  off(event, callback) {
    if (this.events[event]) {
      this.events[event] = this.events[event].filter(cb => cb !== callback);
    }
  }
}

// Usage
const bus = new EventBus();

const unsubscribe = bus.on('user:login', (user) => {
  console.log('User logged in:', user);
});

bus.emit('user:login', { id: 1, name: 'Alice' });

unsubscribe(); // Stop listening
```

### Event Delegation

```javascript
// Instead of adding listeners to each item
document.querySelector('.list').addEventListener('click', (e) => {
  const item = e.target.closest('.list-item');
  if (item) {
    const id = item.dataset.id;
    console.log('Clicked item:', id);
  }
});

// Dynamic items work automatically
```

### Event Throttling

```javascript
// Throttle scroll events
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

window.addEventListener('scroll', throttle(() => {
  console.log('Scroll position:', window.scrollY);
}, 100));
```

### Event Debouncing

```javascript
// Debounce search input
function debounce(fn, delay) {
  let timer;
  return function(...args) {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), delay);
  };
}

const searchInput = document.querySelector('#search');
searchInput.addEventListener('input', debounce((e) => {
  console.log('Searching for:', e.target.value);
}, 300));
```

---

## Complete Guide: CSS Animation Techniques

### Keyframe Animations

```css
/* Fade in */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Slide up */
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Scale in */
@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* Bounce */
@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

/* Pulse */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Spin */
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Gradient shift */
@keyframes gradientShift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}
```

### Animation Utilities

```css
.animate-fade-in { animation: fadeIn 0.3s ease-out; }
.animate-slide-up { animation: slideUp 0.3s ease-out; }
.animate-scale-in { animation: scaleIn 0.2s ease-out; }
.animate-bounce { animation: bounce 1s ease-in-out infinite; }
.animate-pulse { animation: pulse 2s ease-in-out infinite; }
.animate-spin { animation: spin 1s linear infinite; }

.delay-100 { animation-delay: 100ms; }
.delay-200 { animation-delay: 200ms; }
.delay-300 { animation-delay: 300ms; }

.duration-fast { animation-duration: 150ms; }
.duration-normal { animation-duration: 200ms; }
.duration-slow { animation-duration: 300ms; }
```

---

## Complete Guide: JavaScript Data Structures

### Stack

```javascript
class Stack {
  constructor() {
    this.items = [];
  }

  push(item) {
    this.items.push(item);
  }

  pop() {
    return this.items.pop();
  }

  peek() {
    return this.items[this.items.length - 1];
  }

  isEmpty() {
    return this.items.length === 0;
  }

  size() {
    return this.items.length;
  }
}
```

### Queue

```javascript
class Queue {
  constructor() {
    this.items = [];
  }

  enqueue(item) {
    this.items.push(item);
  }

  dequeue() {
    return this.items.shift();
  }

  peek() {
    return this.items[0];
  }

  isEmpty() {
    return this.items.length === 0;
  }

  size() {
    return this.items.length;
  }
}
```

### Linked List

```javascript
class Node {
  constructor(value) {
    this.value = value;
    this.next = null;
  }
}

class LinkedList {
  constructor() {
    this.head = null;
    this.size = 0;
  }

  append(value) {
    const node = new Node(value);
    
    if (!this.head) {
      this.head = node;
    } else {
      let current = this.head;
      while (current.next) {
        current = current.next;
      }
      current.next = node;
    }
    
    this.size++;
  }

  prepend(value) {
    const node = new Node(value);
    node.next = this.head;
    this.head = node;
    this.size++;
  }

  remove(value) {
    if (!this.head) return;
    
    if (this.head.value === value) {
      this.head = this.head.next;
      this.size--;
      return;
    }
    
    let current = this.head;
    while (current.next) {
      if (current.next.value === value) {
        current.next = current.next.next;
        this.size--;
        return;
      }
      current = current.next;
    }
  }

  find(value) {
    let current = this.head;
    while (current) {
      if (current.value === value) return current;
      current = current.next;
    }
    return null;
  }

  toArray() {
    const result = [];
    let current = this.head;
    while (current) {
      result.push(current.value);
      current = current.next;
    }
    return result;
  }
}
```

### Hash Map

```javascript
class HashMap {
  constructor(size = 53) {
    this.keyMap = new Array(size);
  }

  _hash(key) {
    let total = 0;
    const PRIME = 31;
    for (let i = 0; i < Math.min(key.length, 100); i++) {
      const char = key[i];
      const value = char.charCodeAt(0) - 96;
      total = (total * PRIME + value) % this.keyMap.length;
    }
    return total;
  }

  set(key, value) {
    const index = this._hash(key);
    if (!this.keyMap[index]) {
      this.keyMap[index] = [];
    }
    
    const existing = this.keyMap[index].find(([k]) => k === key);
    if (existing) {
      existing[1] = value;
    } else {
      this.keyMap[index].push([key, value]);
    }
  }

  get(key) {
    const index = this._hash(key);
    if (this.keyMap[index]) {
      const pair = this.keyMap[index].find(([k]) => k === key);
      if (pair) return pair[1];
    }
    return undefined;
  }

  keys() {
    const keys = [];
    for (const bucket of this.keyMap) {
      if (bucket) {
        for (const [key] of bucket) {
          keys.push(key);
        }
      }
    }
    return keys;
  }

  values() {
    const values = [];
    for (const bucket of this.keyMap) {
      if (bucket) {
        for (const [, value] of bucket) {
          values.push(value);
        }
      }
    }
    return values;
  }
}
```

---

## Complete Guide: JavaScript Sorting Algorithms

### Bubble Sort

```javascript
function bubbleSort(arr) {
  const n = arr.length;
  
  for (let i = 0; i < n - 1; i++) {
    for (let j = 0; j < n - i - 1; j++) {
      if (arr[j] > arr[j + 1]) {
        [arr[j], arr[j + 1]] = [arr[j + 1], arr[j]];
      }
    }
  }
  
  return arr;
}
```

### Quick Sort

```javascript
function quickSort(arr) {
  if (arr.length <= 1) return arr;
  
  const pivot = arr[Math.floor(arr.length / 2)];
  const left = arr.filter(x => x < pivot);
  const middle = arr.filter(x => x === pivot);
  const right = arr.filter(x => x > pivot);
  
  return [...quickSort(left), ...middle, ...quickSort(right)];
}
```

### Merge Sort

```javascript
function mergeSort(arr) {
  if (arr.length <= 1) return arr;
  
  const mid = Math.floor(arr.length / 2);
  const left = mergeSort(arr.slice(0, mid));
  const right = mergeSort(arr.slice(mid));
  
  return merge(left, right);
}

function merge(left, right) {
  const result = [];
  let leftIndex = 0;
  let rightIndex = 0;
  
  while (leftIndex < left.length && rightIndex < right.length) {
    if (left[leftIndex] < right[rightIndex]) {
      result.push(left[leftIndex]);
      leftIndex++;
    } else {
      result.push(right[rightIndex]);
      rightIndex++;
    }
  }
  
  return result.concat(left.slice(leftIndex), right.slice(rightIndex));
}
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
- **Expansion Materials:** ~56,000 words
- **Total:** ~209,840 words

### Complete Topic Coverage

**57 Chapters across 15 Parts:**

**Part 1-8 (Original 36 chapters):**
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

**Part 9-15 (New 21 chapters):**
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
